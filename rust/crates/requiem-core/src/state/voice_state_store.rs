use crate::sounds::Sound;
use crate::sounds;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::Instant;

use requiem_ui::signal::{Signal, signal};
use serde::Deserialize;

use super::wire::MemberItem;
use super::{Action, Store};
use crate::models::Id;
use crate::models::id::marker::{ChannelMarker, GuildMarker, UserMarker};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RtcState {
    #[default]
    Disconnected,

    AwaitingEndpoint,

    Authenticating,

    RtcConnecting,

    RtcConnected,

    NoRoute,

    RtcDisconnected,
}

impl RtcState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disconnected => "DISCONNECTED",
            Self::AwaitingEndpoint => "AWAITING_ENDPOINT",
            Self::Authenticating => "AUTHENTICATING",
            Self::RtcConnecting => "RTC_CONNECTING",
            Self::RtcConnected => "RTC_CONNECTED",
            Self::NoRoute => "NO_ROUTE",
            Self::RtcDisconnected => "RTC_DISCONNECTED",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "AWAITING_ENDPOINT" => Self::AwaitingEndpoint,
            "AUTHENTICATING" | "CONNECTING" => Self::Authenticating,
            "RTC_CONNECTING" | "ICE_CHECKING" | "DTLS_CONNECTING" => Self::RtcConnecting,
            "RTC_CONNECTED" => Self::RtcConnected,
            "NO_ROUTE" => Self::NoRoute,
            "RTC_DISCONNECTED" => Self::RtcDisconnected,
            _ => Self::Disconnected,
        }
    }

    pub fn is_connected(self) -> bool {
        self == Self::RtcConnected
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct VoiceState {
    pub user_id: Id<UserMarker>,
    #[serde(default)]
    pub guild_id: Option<Id<GuildMarker>>,
    #[serde(default)]
    pub channel_id: Option<Id<ChannelMarker>>,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub self_mute: bool,
    #[serde(default)]
    pub self_deaf: bool,

    #[serde(default)]
    pub mute: bool,
    #[serde(default)]
    pub deaf: bool,
    #[serde(default)]
    pub self_video: bool,
    #[serde(default)]
    pub self_stream: bool,
    #[serde(default)]
    pub suppress: bool,
    #[serde(default)]
    pub request_to_speak_timestamp: Option<String>,

    #[serde(default, skip_serializing)]
    pub member: Option<MemberItem>,
}

impl VoiceState {
    pub fn is_muted(&self) -> bool {
        self.self_mute || self.mute || self.suppress
    }

    pub fn is_deafened(&self) -> bool {
        self.self_deaf || self.deaf
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChannelVoiceInfo {
    pub status: Option<String>,

    pub voice_start_time: Option<i64>,
}

#[derive(Clone, Debug, Default)]
pub struct Rtc {
    pub state: RtcState,
    pub guild_id: Option<Id<GuildMarker>>,
    pub channel_id: Option<Id<ChannelMarker>>,

    pub ping: Option<u64>,

    pub pings: Vec<u64>,

    pub connected_at: Option<Instant>,
}

const PING_HISTORY: usize = 60;

#[derive(Clone)]
pub struct VoiceStateStore {
    pub by_channel: Signal<HashMap<Id<ChannelMarker>, Vec<VoiceState>>>,
    pub channel_info: Signal<HashMap<Id<ChannelMarker>, ChannelVoiceInfo>>,
    pub rtc: Signal<Rtc>,

    /// Per guild: is anyone in one of its voice channels. A guild's flag only changes when it gains its first or loses its last participant, and only wakes that guild's readers.
    voice_flags: Rc<RefCell<HashMap<Id<GuildMarker>, Signal<bool>>>>,

    pub speaking: Signal<HashSet<Id<UserMarker>>>,

    pub video: Signal<HashSet<Id<UserMarker>>>,

    me: Signal<Option<Id<UserMarker>>>,
}

impl Default for VoiceStateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl VoiceStateStore {
    pub fn new() -> Self {
        Self {
            by_channel: signal(HashMap::new()),
            channel_info: signal(HashMap::new()),
            rtc: signal(Rtc::default()),
            voice_flags: Rc::default(),
            speaking: signal(HashSet::new()),
            video: signal(HashSet::new()),
            me: signal(None),
        }
    }

    pub fn in_channel(&self, channel_id: Id<ChannelMarker>) -> Vec<VoiceState> {
        self.by_channel
            .with(|all| all.get(&channel_id).cloned())
            .unwrap_or_default()
    }

    pub fn of_user(
        &self,
        guild_id: Option<Id<GuildMarker>>,
        user_id: Id<UserMarker>,
    ) -> Option<VoiceState> {
        self.by_channel.with(|all| {
            all.values()
                .flatten()
                .find(|s| s.user_id == user_id && s.guild_id == guild_id)
                .cloned()
        })
    }

    pub fn mine(&self) -> Option<VoiceState> {
        let me = self.me.get()?;
        self.by_channel
            .with(|all| all.values().flatten().find(|s| s.user_id == me).cloned())
    }

    pub fn guild_has_voice(&self, guild_id: Id<GuildMarker>) -> bool {
        self.voice_flag(guild_id).get()
    }

    fn voice_flag(&self, guild_id: Id<GuildMarker>) -> Signal<bool> {
        if let Some(existing) = self.voice_flags.borrow().get(&guild_id) {
            return existing.clone();
        }
        let present = self.by_channel.with_untracked(|all| {
            all.values().flatten().any(|s| s.guild_id == Some(guild_id))
        });
        let flag = signal(present);
        self.voice_flags.borrow_mut().insert(guild_id, flag.clone());
        flag
    }

    fn refresh_guilds_with_voice(&self) {
        let now: HashSet<_> = self.by_channel.with_untracked(|all| {
            all.values().flatten().filter_map(|s| s.guild_id).collect()
        });
        for (guild_id, flag) in self.voice_flags.borrow().iter() {
            let present = now.contains(guild_id);
            if flag.with_untracked(|before| *before != present) {
                flag.set(present);
            }
        }
    }

    pub fn is_speaking(&self, user_id: Id<UserMarker>) -> bool {
        self.speaking.with(|s| s.contains(&user_id))
    }

    pub fn info(&self, channel_id: Id<ChannelMarker>) -> ChannelVoiceInfo {
        self.channel_info
            .with(|all| all.get(&channel_id).cloned())
            .unwrap_or_default()
    }

    fn upsert(&self, state: VoiceState) {
        let me = self.me.with_untracked(|me| *me);
        let ours = Some(state.user_id) == me;

        let mine = self
            .rtc
            .with_untracked(|rtc| rtc.channel_id.filter(|_| rtc.state.is_connected()));
        let was = self.by_channel.with_untracked(|all| {
            all.iter()
                .find(|(_, states)| {
                    states
                        .iter()
                        .any(|s| s.user_id == state.user_id && s.guild_id == state.guild_id)
                })
                .map(|(id, _)| *id)
        });
        if !ours && mine.is_some() && was != state.channel_id {
            if state.channel_id == mine {
                sounds::play(Sound::UserJoin);
            } else if was == mine {
                sounds::play(Sound::UserLeave);
            }
        }
        self.by_channel.update(|all| {
            for states in all.values_mut() {
                states.retain(|s| !(s.user_id == state.user_id && s.guild_id == state.guild_id));
            }
            all.retain(|_, states| !states.is_empty());
            if let Some(channel_id) = state.channel_id {
                all.entry(channel_id).or_default().push(state.clone());
            }
        });
        if state.channel_id.is_none() {
            self.speaking.update(|s| {
                s.remove(&state.user_id);
            });
            self.video.update(|v| {
                v.remove(&state.user_id);
            });
        }
        if ours && state.channel_id.is_some() {
            self.rtc.update(|rtc| {
                rtc.channel_id = state.channel_id;
                rtc.guild_id = state.guild_id;
            });
        }
    }

    fn set_rtc_state(
        &self,
        state: RtcState,
        guild_id: Option<Id<GuildMarker>>,
        channel_id: Option<Id<ChannelMarker>>,
    ) {
        let before = self.rtc.with_untracked(|rtc| (rtc.state, rtc.channel_id));

        match state {
            RtcState::RtcConnected if before.0 != RtcState::RtcConnected => sounds::play(
                if before.1.is_some() && channel_id.is_some() && before.1 != channel_id {
                    Sound::UserMoved
                } else {
                    Sound::UserJoin
                },
            ),
            RtcState::Disconnected if before.0 != RtcState::Disconnected => {
                sounds::play(Sound::Disconnect)
            }
            _ => {}
        }
        self.rtc.update(|rtc| {
            let was = rtc.state;
            rtc.state = state;
            if channel_id.is_some() {
                rtc.channel_id = channel_id;
                rtc.guild_id = guild_id;
            }
            match state {
                RtcState::RtcConnected if was != RtcState::RtcConnected => {
                    rtc.connected_at = Some(Instant::now())
                }
                RtcState::RtcConnected => {}
                RtcState::Disconnected => *rtc = Rtc::default(),
                _ => rtc.connected_at = None,
            }
        });
        if !state.is_connected() {
            self.speaking.set(HashSet::new());
            self.video.set(HashSet::new());
        }
    }
}

impl Store for VoiceStateStore {
    fn handle(&self, action: &Action) {
        self.apply(action);
        self.refresh_guilds_with_voice();
    }

    fn reset(&self) {
        self.by_channel.set(HashMap::new());
        self.refresh_guilds_with_voice();
        self.channel_info.set(HashMap::new());
        self.rtc.set(Rtc::default());
        self.speaking.set(HashSet::new());
        self.video.set(HashSet::new());
    }
}

impl VoiceStateStore {
    fn apply(&self, action: &Action) {
        match action {
            Action::ConnectionOpen { user, guilds, .. } => {
                self.me.set(Some(user.id));
                let mut by_channel: HashMap<Id<ChannelMarker>, Vec<VoiceState>> = HashMap::new();
                for guild in guilds {
                    for state in &guild.voice_states {
                        if let Some(channel_id) = state.channel_id {
                            by_channel
                                .entry(channel_id)
                                .or_default()
                                .push(state.clone());
                        }
                    }
                }
                self.by_channel.set(by_channel);
            }
            Action::GuildCreate(guild) => {
                let guild_id = guild.guild.id;
                self.by_channel.update(|all| {
                    for states in all.values_mut() {
                        states.retain(|s| s.guild_id != Some(guild_id));
                    }
                    all.retain(|_, states| !states.is_empty());
                    for state in &guild.voice_states {
                        if let Some(channel_id) = state.channel_id {
                            all.entry(channel_id).or_default().push(state.clone());
                        }
                    }
                });
            }
            Action::GuildDelete(guild_id) => {
                let guild_id = *guild_id;
                self.by_channel.update(|all| {
                    for states in all.values_mut() {
                        states.retain(|s| s.guild_id != Some(guild_id));
                    }
                    all.retain(|_, states| !states.is_empty());
                });
            }
            Action::ChannelDelete(deleted) => {
                let id = deleted.id;
                self.by_channel.update(|all| {
                    all.remove(&id);
                });
            }
            Action::VoiceStateUpdate(state) => self.upsert((**state).clone()),
            Action::VoiceChannelInfo { channels } => self.channel_info.update(|all| {
                for (id, info) in channels {
                    let entry = all.entry(*id).or_default();
                    if let Some(status) = &info.status {
                        entry.status = status.clone().filter(|s| !s.is_empty());
                    }
                    if let Some(start) = info.voice_start_time {
                        entry.voice_start_time = start;
                    }
                }
            }),
            Action::RtcConnectionState {
                state,
                guild_id,
                channel_id,
            } => self.set_rtc_state(*state, *guild_id, *channel_id),
            Action::RtcPing(ping) => self.rtc.update(|rtc| {
                rtc.ping = Some(*ping);
                rtc.pings.push(*ping);
                if rtc.pings.len() > PING_HISTORY {
                    rtc.pings.remove(0);
                }
            }),
            Action::Speaking { user_id, speaking } => self.speaking.update(|s| {
                if *speaking {
                    s.insert(*user_id);
                } else {
                    s.remove(user_id);
                }
            }),
            Action::VoiceVideo { user_id, active } => self.video.update(|v| {
                if *active {
                    v.insert(*user_id);
                } else {
                    v.remove(user_id);
                }
            }),
            _ => {}
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ChannelInfoPatch {
    pub status: Option<Option<String>>,
    pub voice_start_time: Option<Option<i64>>,
}

#[cfg(test)]
mod tests {
    use crate::models::Id as ModelsId;
    use serde_json::json;

    use crate::gateway::events::GatewayEvent;
    use crate::state::AppState;

    fn user(id: u64) -> serde_json::Value {
        json!({ "id": id.to_string(), "username": format!("u{id}"), "discriminator": "0", "global_name": null, "avatar": null })
    }

    fn ready(state: &AppState) {
        state.apply_event(&GatewayEvent::new(
            "READY",
            json!({
                "user": user(1),
                "session_id": "s1",
                "guilds": [{ "id": "40", "properties": { "name": "G", "owner_id": "1" }, "channels": [{ "id": "41", "type": 2, "name": "General" }, { "id": "42", "type": 2, "name": "Other" }],
                    "voice_states": [{ "user_id": "2", "channel_id": "41", "session_id": "x", "self_mute": true, "member": { "user": user(2), "nick": "Two" } }] }],
            }),
        ));
    }

    #[test]
    fn ready_states_get_their_guild_and_moves_replace() {
        let state = AppState::new();
        ready(&state);
        let general = ModelsId::new(41);
        let in_general = state.voice.in_channel(general);
        assert_eq!(in_general.len(), 1);
        assert_eq!(
            in_general[0].guild_id,
            Some(ModelsId::new(40)),
            "stamped from the guild"
        );
        assert!(in_general[0].is_muted());
        assert!(state.voice.guild_has_voice(ModelsId::new(40)));

        state.apply_event(&GatewayEvent::new(
            "VOICE_STATE_UPDATE",
            json!({ "user_id": "2", "guild_id": "40", "channel_id": "42", "session_id": "x" }),
        ));
        assert!(state.voice.in_channel(general).is_empty());
        assert_eq!(state.voice.in_channel(ModelsId::new(42)).len(), 1);

        state.apply_event(&GatewayEvent::new(
            "VOICE_STATE_UPDATE",
            json!({ "user_id": "2", "guild_id": "40", "channel_id": null, "session_id": "x" }),
        ));
        assert!(!state.voice.guild_has_voice(ModelsId::new(40)));
    }

    #[test]
    fn rtc_state_speaking_and_clock() {
        let state = AppState::new();
        ready(&state);
        state.apply_event(&GatewayEvent::new(
            "RTC_CONNECTION_STATE",
            json!({ "state": "RTC_CONNECTED", "guild_id": "40", "channel_id": "41" }),
        ));
        assert!(
            state
                .voice
                .rtc
                .with_untracked(|r| r.clone())
                .state
                .is_connected()
        );
        state.apply_event(&GatewayEvent::new(
            "SPEAKING",
            json!({ "user_id": "2", "speaking": 1 }),
        ));
        assert!(state.voice.is_speaking(ModelsId::new(2)));
        state.apply_event(&GatewayEvent::new(
            "RTC_CONNECTION_PING",
            json!({ "ping": 42 }),
        ));
        assert_eq!(state.voice.rtc.with_untracked(|r| r.clone()).ping, Some(42));
        state.apply_event(&GatewayEvent::new(
            "RTC_CONNECTION_STATE",
            json!({ "state": "DISCONNECTED" }),
        ));
        assert!(
            !state.voice.is_speaking(ModelsId::new(2)),
            "speaking clears with the call"
        );

        state.apply_event(&GatewayEvent::new(
            "VOICE_CHANNEL_START_TIME_UPDATE",
            json!({ "id": "41", "guild_id": "40", "voice_start_time": 1700000000 }),
        ));
        assert_eq!(
            state
                .voice
                .info(ModelsId::new(41))
                .voice_start_time,
            Some(1700000000)
        );
        state.apply_event(&GatewayEvent::new("CHANNEL_INFO", json!({ "guild_id": "40", "channels": [{ "id": "42", "status": "gaming", "voice_start_time": null }] })));
        assert_eq!(
            state
                .voice
                .info(ModelsId::new(42))
                .status
                .as_deref(),
            Some("gaming")
        );
    }
}
