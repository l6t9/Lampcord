use crate::gateway::events;
use crate::models::Permissions;
use crate::models;
use crate::state;
use parking_lot::Mutex;
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::atomic;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use crate::gateway::events::GatewayEvent;
use crate::gateway::opcodes as gw;
use crate::models::Id;
use crate::models::id::marker::{ChannelMarker, GuildMarker, UserMarker};

use self::manager::{ConnectionConfig, VoiceFlags};
use self::status::RtcState;
use self::stream::{StreamCreate, StreamServerUpdate, StreamType};

pub mod attenuation;
pub mod audio;
pub mod capture;
pub mod dave;
pub mod diag;
pub mod engine;
pub mod gateway;
pub mod local_mute;
pub mod manager;
pub mod opcodes;
pub mod opus;
pub mod ptt;
pub mod sdp;
pub mod status;
pub mod stream;
pub mod udp;
pub mod video;
pub mod webrtc;

fn voice_state_update_payload(
    guild_id: Option<u64>,
    channel_id: Option<u64>,
    flags: &VoiceFlags,
) -> Value {
    json!({
        "guild_id": guild_id.map(|g| g.to_string()),
        "channel_id": channel_id.map(|c| c.to_string()),

        "self_mute": flags.mute.load(Ordering::Relaxed) || flags.deaf.load(Ordering::Relaxed),
        "self_deaf": flags.deaf.load(Ordering::Relaxed),
        "self_video": flags.camera.load(Ordering::Relaxed),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Call {
    pub guild: Option<u64>,
    pub channel: u64,
}

struct State {
    outgoing: Option<UnboundedSender<Value>>,

    client: Option<wreq::Client>,

    user_id: Option<u64>,

    gateway_session: Option<String>,

    target: Option<Call>,

    session_id: Option<String>,

    endpoint: Option<String>,
    token: Option<String>,

    current: Option<(String, String, String)>,

    flags: Arc<VoiceFlags>,

    cancel: Option<UnboundedSender<()>>,

    stream: Option<PendingStream>,
}

struct PendingStream {
    key: String,
    server_id: u64,
    channel_id: u64,
    endpoint: Option<String>,
    token: Option<String>,
    cancel: Option<UnboundedSender<()>>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    let mut guard = STATE.lock();
    let state = guard.get_or_insert_with(State::default);
    f(state)
}

impl Default for State {
    fn default() -> Self {
        Self {
            outgoing: None,
            client: None,
            user_id: None,
            gateway_session: None,
            target: None,
            session_id: None,
            endpoint: None,
            token: None,
            current: None,
            flags: Arc::new(VoiceFlags::default()),
            cancel: None,
            stream: None,
        }
    }
}

pub fn init(outgoing: UnboundedSender<Value>, client: wreq::Client) {
    with(|s| {
        s.outgoing = Some(outgoing);
        s.client = Some(client);
    });
}

pub fn set_user_id(user_id: u64) {
    with(|s| s.user_id = Some(user_id));
}

pub fn set_gateway_session(session_id: String) {
    with(|s| s.gateway_session = Some(session_id));
}

pub fn join(guild_id: u64, channel_id: u64) {
    join_call(Call {
        guild: Some(guild_id),
        channel: channel_id,
    });
}

pub fn join_call(call: Call) {
    with(|s| {
        s.target = Some(call);

        s.endpoint = None;
        s.token = None;
        let payload = voice_state_update_payload(call.guild, Some(call.channel), &s.flags);
        log::info!(
            "voice: join requested guild={:?} channel={} user={:?} session={} outgoing={}",
            call.guild,
            call.channel,
            s.user_id,
            s.gateway_session.as_deref().unwrap_or("<none>"),
            s.outgoing.is_some()
        );
        if s.outgoing.is_none() {
            log::warn!("voice: join has no gateway sender; op 4 will not be sent");
        }
        if s.user_id.is_none() {
            log::warn!("voice: join before READY user id; voice server identify will wait");
        }
        send_op(s, 4, payload);
    });
    status::rtc_state(RtcState::AwaitingEndpoint, call.guild, Some(call.channel));
}

pub fn join_channel(
    state: &state::AppState,
    guild_id: u64,
    channel: &models::Channel,
) -> bool {
    if !may_join(state, channel) {
        log::info!("voice: not joining {} (no CONNECT)", channel.id);
        return false;
    }
    join(guild_id, channel.id.get());
    true
}

pub fn may_join(state: &state::AppState, channel: &models::Channel) -> bool {
    state.can(channel, Permissions::CONNECT)
}

pub fn leave() {
    let left = with(|s| {
        let left = s.target.take();
        forget_connection(s);
        let payload = voice_state_update_payload(None, None, &s.flags);
        send_op(s, 4, payload);
        left
    });
    if left.is_some() {
        status::rtc_state(RtcState::Disconnected, None, None);
    }
}

pub fn set_mute(mute: bool) {
    set_flag(|f| &f.mute, mute);
}

pub fn set_deaf(deaf: bool) {
    set_flag(|f| &f.deaf, deaf);
}

pub fn set_camera(on: bool) {
    set_flag(|f| &f.camera, on);
}

fn set_flag(flag: impl Fn(&VoiceFlags) -> &atomic::AtomicBool, on: bool) {
    with(|s| {
        flag(&s.flags).store(on, Ordering::Relaxed);

        let (guild, channel) = s
            .target
            .map_or((None, None), |t| (t.guild, Some(t.channel)));
        send_op(s, 4, voice_state_update_payload(guild, channel, &s.flags));
    });
}

pub fn is_self_mute() -> bool {
    with(|s| s.flags.mute.load(Ordering::Relaxed))
}

pub fn is_self_deaf() -> bool {
    with(|s| s.flags.deaf.load(Ordering::Relaxed))
}

pub fn is_camera_on() -> bool {
    with(|s| s.flags.camera.load(Ordering::Relaxed))
}

pub fn current_channel() -> Option<(u64, u64)> {
    with(|s| s.target.and_then(|t| Some((t.guild?, t.channel))))
}

pub fn current_call() -> Option<Call> {
    with(|s| s.target)
}

pub fn start_stream(guild_id: u64, channel_id: u64, preferred_region: &str) {
    with(|s| {
        send_op(
            s,
            gw::STREAM_CREATE,
            json!({
                "type": StreamType::Guild.as_str(),
                "guild_id": guild_id.to_string(),
                "channel_id": channel_id.to_string(),
                "preferred_region": preferred_region,
            }),
        );
    });
}

pub fn stop_stream(stream_key: &str) {
    with(|s| {
        if let Some(stream) = s.stream.as_mut() {
            if let Some(cancel) = stream.cancel.take() {
                let _ = cancel.send(());
            }
        }
        s.stream = None;
        send_op(s, gw::STREAM_DELETE, json!({ "stream_key": stream_key }));
    });
}

pub fn own_stream_key() -> Option<String> {
    with(|s| s.stream.as_ref().map(|stream| stream.key.clone()))
}

pub fn watch_stream(stream_key: &str) {
    with(|s| {
        send_op(s, gw::STREAM_WATCH, json!({ "stream_key": stream_key }));
    });
}

pub fn stream_ping(stream_key: &str) {
    with(|s| {
        send_op(s, gw::STREAM_PING, json!({ "stream_key": stream_key }));
    });
}

pub fn set_stream_paused(stream_key: &str, paused: bool) {
    with(|s| {
        send_op(
            s,
            gw::STREAM_SET_PAUSED,
            json!({ "stream_key": stream_key, "paused": paused }),
        );
    });
}

pub fn request_channel_info(guild_id: u64) {
    with(|s| {
        send_op(
            s,
            gw::REQUEST_CHANNEL_INFO,
            json!({ "guild_id": guild_id.to_string(), "fields": ["status", "voice_start_time"] }),
        );
    });
}

pub(crate) const HANDLED_EVENTS: &[&str] = &[
    "VOICE_STATE_UPDATE",
    "VOICE_SERVER_UPDATE",
    "STREAM_CREATE",
    "STREAM_SERVER_UPDATE",
    "STREAM_CLOSE",
    "STREAM_DELETE",
];

pub fn note_event(event: &GatewayEvent) {
    match event.name.as_str() {
        "READY" => note_ready(event),
        "VOICE_STATE_UPDATE" => note_voice_state(event),
        "VOICE_SERVER_UPDATE" => note_voice_server(event),
        "STREAM_CREATE" => note_stream_create(event),
        "STREAM_SERVER_UPDATE" => note_stream_server(event),
        "STREAM_CLOSE" | "STREAM_DELETE" => note_stream_closed(event),
        _ => {}
    }
}

fn note_ready(event: &GatewayEvent) {
    #[derive(serde::Deserialize)]
    struct W {
        session_id: Option<String>,
    }
    if let Ok(W {
        session_id: Some(session_id),
    }) = event.parse::<W>()
    {
        set_gateway_session(session_id);
    }
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct VoiceStateUpdate {
    user_id: Id<UserMarker>,
    #[serde(default)]
    guild_id: Option<Id<GuildMarker>>,
    session_id: Option<String>,
    channel_id: Option<Id<ChannelMarker>>,
}

fn note_voice_state(event: &GatewayEvent) {
    let Ok(update) = event.parse::<VoiceStateUpdate>() else {
        log::warn!(
            "voice: VOICE_STATE_UPDATE didn't parse; {}",
            events::summarize("VOICE_STATE_UPDATE", event.json())
        );
        return;
    };
    let disconnected = with(|s| {
        let mine = Some(u64::from(update.user_id)) == s.user_id;
        log::debug!(
            "voice: VOICE_STATE_UPDATE user={} guild={:?} channel={:?} session={} mine={mine} target={:?}",
            update.user_id,
            update.guild_id,
            update.channel_id,
            update.session_id.as_deref().unwrap_or("<none>"),
            s.target
        );
        if !mine {
            return None;
        }
        let ours = match (&update.session_id, &s.gateway_session) {
            (Some(theirs), Some(mine)) => theirs == mine,

            _ => true,
        };
        if !ours {
            log::info!(
                "voice: VOICE_STATE_UPDATE is another session (channel={:?}, target={:?})",
                update.channel_id,
                s.target
            );
            if update.channel_id.is_some() && s.target.is_some() {
                log::info!("voice: another session joined voice, disconnecting here");
                s.target = None;
                forget_connection(s);
                return Some(());
            }
            return None;
        }
        match update.channel_id {
            Some(channel_id) => {
                s.session_id = update.session_id.clone();

                let channel = u64::from(channel_id);
                if let Some(target) = s.target.as_mut() {
                    target.channel = channel;
                    if update.guild_id.is_some() {
                        target.guild = update.guild_id.map(u64::from);
                    }
                    log::info!(
                        "voice: own voice state guild={:?} channel={channel} session={}",
                        target.guild,
                        s.session_id.as_deref().unwrap_or("<none>")
                    );
                } else {
                    log::info!(
                        "voice: own voice state for channel {channel} arrived with no join target; not connecting"
                    );
                }
                None
            }
            None => {
                let was = s.target.take();
                log::info!(
                    "voice: own voice state cleared channel (had_target={})",
                    was.is_some()
                );
                s.session_id = None;
                forget_connection(s);
                was.map(|_| ())
            }
        }
    });
    if disconnected.is_some() {
        status::rtc_state(RtcState::Disconnected, None, None);
    }
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct VoiceServerUpdate {
    token: String,

    endpoint: Option<String>,
    guild_id: Option<Id<GuildMarker>>,
}

fn note_voice_server(event: &GatewayEvent) {
    let Ok(update) = event.parse::<VoiceServerUpdate>() else {
        log::warn!(
            "voice: VOICE_SERVER_UPDATE didn't parse; {}",
            events::summarize("VOICE_SERVER_UPDATE", event.json())
        );
        return;
    };
    log::info!(
        "voice: VOICE_SERVER_UPDATE guild={:?} endpoint={}",
        update.guild_id,
        update.endpoint.as_deref().unwrap_or("<none>")
    );
    let waiting = with(|s| {
        s.token = Some(update.token);
        match update.endpoint {
            Some(endpoint) => {
                s.endpoint = Some(endpoint);
                maybe_connect(s);
                None
            }
            None => {
                log::warn!("voice: VOICE_SERVER_UPDATE has no endpoint; staying disconnected");
                s.endpoint = None;
                disconnect_inner(s);
                s.target
            }
        }
    });
    if let Some(call) = waiting {
        status::rtc_state(RtcState::AwaitingEndpoint, call.guild, Some(call.channel));
    }
}

fn note_stream_create(event: &GatewayEvent) {
    let Ok(create) = event.parse::<StreamCreate>() else {
        return;
    };
    with(|s| {
        let server_id = u64::from(create.rtc_server_id);
        let channel_id = u64::from(create.rtc_channel_id);
        let key = create.stream_key.clone();
        let stream = s.stream.get_or_insert_with(|| PendingStream {
            key,
            server_id,
            channel_id,
            endpoint: None,
            token: None,
            cancel: None,
        });
        stream.key = create.stream_key;
        stream.server_id = server_id;
        stream.channel_id = channel_id;
        maybe_connect_stream(s);
    });
}

fn note_stream_server(event: &GatewayEvent) {
    let Ok(update) = event.parse::<StreamServerUpdate>() else {
        return;
    };
    with(|s| {
        let Some(stream) = s.stream.as_mut() else {
            return;
        };
        if stream.key != update.stream_key {
            return;
        }
        stream.endpoint = Some(update.endpoint);
        stream.token = Some(update.token);
        maybe_connect_stream(s);
    });
}

fn note_stream_closed(event: &GatewayEvent) {
    #[derive(serde::Deserialize)]
    struct W {
        stream_key: String,
    }
    let Ok(w) = event.parse::<W>() else { return };
    with(|s| {
        if s.stream
            .as_ref()
            .is_some_and(|stream| stream.key == w.stream_key)
        {
            if let Some(stream) = s.stream.take() {
                if let Some(cancel) = stream.cancel {
                    let _ = cancel.send(());
                }
            }
        }
    });
}

fn maybe_connect_stream(s: &mut State) {
    let Some(user_id) = s.user_id else { return };
    let Some(client) = s.client.clone() else {
        return;
    };
    let scope_guild = s.target.and_then(|t| t.guild);
    let Some(stream) = s.stream.as_mut() else {
        return;
    };
    let (Some(endpoint), Some(token)) = (stream.endpoint.clone(), stream.token.clone()) else {
        return;
    };
    if stream.cancel.is_some() {
        return;
    }

    let (cancel_tx, cancel_rx) = unbounded_channel();
    let config = ConnectionConfig {
        client,
        guild_id: stream.server_id,
        channel_id: stream.channel_id,
        user_id,
        session_id: s
            .session_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().simple().to_string()),
        token,
        endpoint,
        scope_guild,
        video: manager::VideoMode::Screen,
    };
    tokio::spawn(manager::run(
        config,
        Arc::new(VoiceFlags::default()),
        cancel_rx,
    ));
    stream.cancel = Some(cancel_tx);
    log::info!(
        "voice: broadcasting stream {} (endpoint {})",
        stream.key,
        stream.endpoint.as_deref().unwrap_or("")
    );
}

fn maybe_connect(s: &mut State) {
    let Some(user_id) = s.user_id else {
        log::warn!("voice: not connecting yet (no user id)");
        return;
    };
    let Some(call) = s.target else {
        log::warn!("voice: not connecting (no join target; server update arrived without a join)");
        return;
    };
    let Some(session_id) = s.session_id.clone() else {
        log::info!("voice: not connecting yet (waiting for own VOICE_STATE_UPDATE session)");
        return;
    };
    let Some(endpoint) = s.endpoint.clone() else {
        log::info!("voice: not connecting yet (waiting for VOICE_SERVER_UPDATE endpoint)");
        return;
    };
    let Some(token) = s.token.clone() else {
        log::info!("voice: not connecting yet (waiting for voice token)");
        return;
    };
    let Some(client) = s.client.clone() else {
        log::warn!("voice: not connecting (voice client was never initialized)");
        return;
    };

    if !creds_changed(&s.current, &session_id, &endpoint, &token) {
        log::debug!("voice: credentials unchanged, keeping the current connection");
        return;
    }

    disconnect_inner(s);

    let (cancel_tx, cancel_rx) = unbounded_channel();
    let server_id = call.guild.unwrap_or(call.channel);
    log::info!(
        "voice: connecting server_id={server_id} channel={} scope_guild={:?} endpoint={endpoint}",
        call.channel,
        call.guild
    );
    let config = ConnectionConfig {
        client,

        guild_id: server_id,
        channel_id: call.channel,
        user_id,
        session_id: session_id.clone(),
        token: token.clone(),
        endpoint: endpoint.clone(),
        scope_guild: call.guild,

        video: manager::VideoMode::Camera,
    };
    tokio::spawn(manager::run(config, s.flags.clone(), cancel_rx));
    s.cancel = Some(cancel_tx);
    s.current = Some((session_id, endpoint.clone(), token));
    log::info!(
        "voice: joined channel {} in {:?} (endpoint {endpoint})",
        call.channel,
        call.guild
    );
}

fn disconnect_inner(s: &mut State) {
    if let Some(cancel) = s.cancel.take() {
        let _ = cancel.send(());
    }
    s.current = None;
}

fn forget_connection(s: &mut State) {
    disconnect_inner(s);
    s.session_id = None;
    s.endpoint = None;
    s.token = None;
}

fn creds_changed(
    current: &Option<(String, String, String)>,
    session_id: &str,
    endpoint: &str,
    token: &str,
) -> bool {
    let incoming = (
        session_id.to_string(),
        endpoint.to_string(),
        token.to_string(),
    );
    current.as_ref() != Some(&incoming)
}

fn send_op(s: &mut State, op: u8, d: Value) {
    if let Some(outgoing) = &s.outgoing {
        let _ = outgoing.send(json!({ "op": op, "d": d }));
    }
}

#[cfg(test)]
mod tests {
    use crate::models::Permissions as P;
    use super::*;

    #[test]
    fn move_is_detected_when_creds_change() {
        let connected = Some(("sess-1".into(), "endpoint-a".into(), "token-1".into()));

        assert!(!creds_changed(
            &connected,
            "sess-1",
            "endpoint-a",
            "token-1"
        ));

        assert!(creds_changed(&None, "sess-1", "endpoint-a", "token-1"));

        assert!(creds_changed(&connected, "sess-2", "endpoint-a", "token-1"));

        assert!(creds_changed(&connected, "sess-1", "endpoint-b", "token-2"));
    }

    #[test]
    fn leave_sends_null_guild() {
        let flags = VoiceFlags::default();
        let leave = voice_state_update_payload(None, None, &flags);
        assert!(leave["guild_id"].is_null());
        assert!(leave["channel_id"].is_null());
        assert_eq!(leave["self_mute"], json!(false));
        assert_eq!(leave["self_deaf"], json!(false));

        let join = voice_state_update_payload(Some(1), Some(2), &flags);
        assert_eq!(join["guild_id"], "1");
        assert_eq!(join["channel_id"], "2");
    }

    #[test]
    fn the_gateway_filter_forwards_voice_dispatches() {
        for name in HANDLED_EVENTS {
            assert!(
                events::is_wanted(name),
                "{name} should be forwarded"
            );
        }
        assert!(!events::is_wanted(
            "MESSAGE_CREATE_BUT_FOR_VOICE"
        ));
    }

    #[test]
    fn joining_a_voice_channel_without_connect_is_refused() {
        let state = state::AppState::new();
        let user = json!({ "id": "1", "username": "me", "discriminator": "0", "global_name": null, "avatar": null });
        state.apply_event(&GatewayEvent::new(
            "READY",
            json!({ "user": user, "guilds": [] }),
        ));
        let everyone = (P::VIEW_CHANNEL | P::CONNECT | P::SPEAK).0.to_string();
        let no_connect =
            json!([{ "id": "40", "type": 0, "allow": "0", "deny": P::CONNECT.0.to_string() }]);
        state.apply_event(&GatewayEvent::new(
            "GUILD_CREATE",
            json!({ "id": "40", "properties": { "name": "V", "owner_id": "99" }, "roles": [{ "id": "40", "name": "@everyone", "permissions": everyone }], "channels": [{ "id": "41", "type": 2, "name": "locked", "permission_overwrites": no_connect }, { "id": "42", "type": 2, "name": "open" }] }),
        ));
        let channels = state
            .channels
            .by_guild
            .with_untracked(|all| all[&Id::new(40)].clone());
        let locked = channels.iter().find(|c| c.id.get() == 41).unwrap();
        assert!(state.can(locked, P::VIEW_CHANNEL), "still listed");
        assert!(!may_join(&state, locked));
        assert!(!join_channel(&state, 40, locked));
        assert_ne!(
            with(|s| s.target),
            Some(Call {
                guild: Some(40),
                channel: 41
            }),
            "no join was recorded (no op 4)"
        );
        assert!(may_join(
            &state,
            channels.iter().find(|c| c.id.get() == 42).unwrap()
        ));
    }
}
