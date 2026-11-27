pub mod codec;
pub mod etf;
pub mod events;
pub mod guild_subscriptions;
pub mod identify;
pub mod member_requests;
pub mod opcodes;
pub mod qos;
pub mod session;
pub mod stack;

use crate::rest;
use serde_json::value;
use std::time as std_time;
use std::time::Duration;
use std::time::Instant as TimeInstant;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tokio::time;

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;
use wreq::ws::message::{CloseFrame, Message as WsMessage};

use crate::config::BackendConfig;
use crate::properties::SuperProperties;
use events::GatewayEvent;
pub use session::Session;

pub(crate) use guild_subscriptions::send_frames;

const CLOSE_CODE_AUTHENTICATION_FAILED: u16 = 4004;

const CLOSE_CODE_DECODE_ERROR: u16 = 4002;

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("websocket error: {0}")]
    WebSocket(#[from] wreq::Error),
    #[error("failed to parse gateway payload: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("undecodable frame: {0}")]
    Codec(#[from] codec::CodecError),
    #[error("connection closed before HELLO")]
    ClosedBeforeHello,

    #[error("the server does not speak this wire format: {0}")]
    StackRejected(String),
    #[error("connection closed")]
    Closed,

    #[error("server asked us to reconnect")]
    ReconnectRequested,
    #[error("connection is zombied (missed heartbeat ack)")]
    ZombiedConnection,

    #[error("authentication failed (token is no longer valid)")]
    AuthenticationFailed,
}

fn reject_if_negotiating(negotiating: bool, err: GatewayError) -> GatewayError {
    match err {
        GatewayError::ClosedBeforeHello
        | GatewayError::Closed
        | GatewayError::Codec(_)
        | GatewayError::Parse(_)
            if negotiating =>
        {
            GatewayError::StackRejected(err.to_string())
        }
        other => other,
    }
}

fn close_frame_error(frame: Option<CloseFrame>, session: &mut Session) -> GatewayError {
    let reason = frame.as_ref().map(|f| f.reason.as_str().to_string());
    let code = frame.map(|f| u16::from(f.code));
    if code.is_some_and(session::close_code_ends_session) {
        session.clear();
    }
    match code {
        Some(CLOSE_CODE_AUTHENTICATION_FAILED) => GatewayError::AuthenticationFailed,
        _ => {
            match (code, reason) {
                (Some(code), Some(reason)) => {
                    log::info!("gateway closed by server: code {code} reason {reason:?}")
                }
                (Some(code), None) => log::info!("gateway closed by server: code {code}"),
                (None, _) => log::info!("gateway connection dropped without a close frame"),
            }
            GatewayError::Closed
        }
    }
}

#[derive(Deserialize)]
struct RawPayload {
    op: u8,

    #[serde(default)]
    d: Option<Box<value::RawValue>>,
    #[serde(default)]
    s: Option<u64>,
    #[serde(default)]
    t: Option<String>,
}

fn payload_value(payload: &RawPayload) -> serde_json::Value {
    payload
        .d
        .as_deref()
        .and_then(|raw| serde_json::from_str(raw.get()).ok())
        .unwrap_or_default()
}

#[tracing::instrument(skip_all, fields(gateway = %config.gateway_url))]
pub async fn run(
    client: &wreq::Client,
    config: &BackendConfig,
    token: &str,
    super_properties: &SuperProperties,
    session: &mut Session,
    tx: UnboundedSender<GatewayEvent>,
    outgoing_rx: &mut mpsc::UnboundedReceiver<serde_json::Value>,
) -> Result<(), GatewayError> {
    if stack::is_fixed(&config.gateway_url) {
        return run_url(
            client,
            config,
            &config.gateway_url,
            None,
            token,
            super_properties,
            session,
            tx,
            outgoing_rx,
        )
        .await;
    }
    let base = config.gateway_url.as_str();
    let candidates = stack::candidates(base);
    let remembered_first = candidates.first().copied();
    let mut result = Err(GatewayError::ClosedBeforeHello);
    for (index, wire) in candidates.iter().copied().enumerate() {
        let url = stack::url_for(base, wire);
        result = run_url(
            client,
            config,
            &url,
            Some((base, wire)),
            token,
            super_properties,
            session,
            tx.clone(),
            outgoing_rx,
        )
        .await;
        match &result {
            Err(GatewayError::StackRejected(why)) if index + 1 < candidates.len() => {
                log::info!(
                    "gateway: {base} did not take {wire:?} ({why}); trying the next wire format"
                );
                if Some(wire) == remembered_first {
                    stack::forget(base);
                }
            }
            _ => break,
        }
    }
    result
}

#[allow(clippy::too_many_arguments)]
async fn run_url(
    client: &wreq::Client,
    config: &BackendConfig,
    gateway_url: &str,
    negotiating: Option<(&str, stack::Stack)>,
    token: &str,
    super_properties: &SuperProperties,
    session: &mut Session,
    tx: UnboundedSender<GatewayEvent>,
    outgoing_rx: &mut mpsc::UnboundedReceiver<serde_json::Value>,
) -> Result<(), GatewayError> {
    let _ = config;

    let mut upgraded = None;
    let mut last_error = None;
    let mut connected_url = String::new();

    for url in session.urls(gateway_url) {
        let attempt = rest::websocket(client, url.as_str()).send().await;
        match attempt {
            Ok(upgrade) => {
                upgraded = Some(upgrade);
                connected_url = url;
                break;
            }
            Err(err) => {
                log::warn!("gateway: could not connect to {url}: {err}");
                last_error = Some(err);
            }
        }
    }
    guild_subscriptions::set_established(false);
    session.set_established(false);
    let upgrade = match upgraded {
        Some(upgrade) => upgrade,
        None => {
            return Err(match last_error {
                Some(err)
                    if negotiating.is_some()
                        && (err.is_upgrade() || err.is_status() || err.is_websocket()) =>
                {
                    GatewayError::StackRejected(err.to_string())
                }
                Some(err) => GatewayError::WebSocket(err),
                None => GatewayError::ClosedBeforeHello,
            });
        }
    };
    let mut codec = codec::Codec::for_url(&connected_url);
    let ws_stream = upgrade.into_websocket().await?;
    let (mut write, mut read) = ws_stream.split();

    let hello_text = time::timeout(HELLO_TIMEOUT, async {
        loop {
            match read.next().await.ok_or(GatewayError::ClosedBeforeHello)?? {
                WsMessage::Close(frame) => return Err(close_frame_error(frame, session)),
                message => {
                    if let Some(text) = codec.decode(message)? {
                        return Ok(text);
                    }
                }
            }
        }
    })
    .await
    .map_err(|_| GatewayError::ClosedBeforeHello)
    .and_then(|result| result)
    .map_err(|err| reject_if_negotiating(negotiating.is_some(), err))?;
    let hello: RawPayload = serde_json::from_str(&hello_text)
        .map_err(|err| reject_if_negotiating(negotiating.is_some(), err.into()))?;
    if hello.op != opcodes::HELLO {
        return Err(reject_if_negotiating(
            negotiating.is_some(),
            GatewayError::ClosedBeforeHello,
        ));
    }
    if let Some((base, wire)) = negotiating {
        stack::remember(base, wire);
    }
    let heartbeat_interval_ms = payload_value(&hello)
        .get("heartbeat_interval")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(41250);

    let mut last_ack: Option<std_time::Instant> = None;
    send(
        &mut write,
        &codec,
        &begin_session(token, super_properties, session, &mut last_ack),
    )
    .await?;

    let interval = Duration::from_millis(heartbeat_interval_ms);
    let first_delay = Duration::from_millis(
        (uuid::Uuid::new_v4().as_u128() % u128::from(heartbeat_interval_ms.max(1))) as u64,
    );
    let mut next_beat = Instant::now() + first_delay;
    let mut qos_state = qos::QosState::default();
    qos_state.handle_active_state_change(qos::Qos::for_foreground(qos::is_foreground()));
    let launch_id = super_properties.gateway_value()["client_launch_id"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    let mut next_client_heartbeat: Option<Instant> =
        qos::is_foreground().then(Instant::now);
    let mut established = false;

    let mut awaiting_ack = false;

    loop {
        tokio::select! {
            _ = time::sleep_until(next_beat) => {
                if awaiting_ack {

                    close_with_4000(&mut write).await;
                    return Err(GatewayError::ZombiedConnection);
                }
                next_beat = Instant::now() + interval;
                send(&mut write, &codec, &qos::heartbeat(session.seq(), qos_state.consume().as_ref())).await?;
                awaiting_ack = true;
            }
            _ = qos::foreground_changed() => {
                let foregrounded = qos::is_foreground();
                let send_now = qos_state.handle_active_state_change(qos::Qos::for_foreground(foregrounded));
                if foregrounded {
                    next_client_heartbeat.get_or_insert_with(Instant::now);
                } else {
                    next_client_heartbeat = None;
                }
                if send_now && established {
                    send(&mut write, &codec, &qos::heartbeat(session.seq(), qos_state.consume().as_ref())).await?;
                }
            }
            _ = async { time::sleep_until(next_client_heartbeat.unwrap_or_else(Instant::now)).await }, if next_client_heartbeat.is_some() && established => {
                next_client_heartbeat = Some(Instant::now() + qos::CLIENT_HEARTBEAT_INTERVAL);
                if let (Some(session_now), true) = qos::touch_session(true) {
                    send(&mut write, &codec, &session_now.announcement(&launch_id)).await?;
                    send(&mut write, &codec, &qos::heartbeat(session.seq(), qos_state.consume().as_ref())).await?;
                }
            }
            outgoing = outgoing_rx.recv() => {
                let Some(payload) = outgoing else { continue };
                send(&mut write, &codec, &payload).await?;
            }
            msg = read.next() => {
                let Some(msg) = msg else { return Err(GatewayError::Closed) };
                let msg = msg?;
                if let WsMessage::Close(frame) = msg {

                    if negotiating.is_some() && frame.as_ref().is_some_and(|f| u16::from(f.code.clone()) == CLOSE_CODE_DECODE_ERROR) {
                        return Err(GatewayError::StackRejected("close 4002 (decode error)".into()));
                    }
                    return Err(close_frame_error(frame, session));
                }
                let Some(text) = codec.decode(msg)? else { continue };
                let payload: RawPayload = serde_json::from_str(&text)?;
                if let Some(s) = payload.s {
                    session.observe_seq(s);
                }

                match payload.op {
                    op if op == opcodes::DISPATCH => {
                        if !events::is_wanted(payload.t.as_deref().unwrap_or("")) {
                            if let Some(name) = payload.t.as_deref() {
                                log::debug!("gateway: dropped {name} (not in the handled set)");
                            }
                        }
                        if let (Some(name), Some(data)) = (payload.t.filter(|name| events::is_wanted(name)), payload.d) {
                            log::debug!(
                                "gateway: dispatch {name} seq={:?} {}",
                                payload.s,
                                events::summarize(&name, data.get())
                            );
                            if name == "RESUMED" {

                                established = true;
                                session.set_established(true);
                                guild_subscriptions::set_established(true);
                                guild_subscriptions::connection_open(false);
                            }
                            if name == "READY" {
                                session.on_ready_raw(data.get());
                                established = true;
                                session.set_established(true);

                                guild_subscriptions::set_established(true);
                                guild_subscriptions::connection_open(true);

                                if let (Some(current), _) = qos::touch_session(qos::is_foreground()) {
                                    send(&mut write, &codec, &current.announcement(&launch_id)).await?;
                                    send(&mut write, &codec, &qos::heartbeat(session.seq(), qos_state.consume().as_ref())).await?;
                                }
                            }
                            let _ = tx.send(GatewayEvent::from_raw(name, data));
                        }
                    }
                    op if op == opcodes::HEARTBEAT_ACK => {
                        awaiting_ack = false;
                        last_ack = Some(TimeInstant::now());
                    }

                    op if op == opcodes::RECONNECT => {
                        close_with_4000(&mut write).await;
                        return Err(GatewayError::ReconnectRequested);
                    }

                    op if op == opcodes::INVALID_SESSION => {
                        let can_resume = payload_value(&payload).as_bool() == Some(true);
                        if !can_resume {
                            session.clear();
                        }
                        established = false;
                        session.set_established(false);
                        guild_subscriptions::set_established(false);
                        send(&mut write, &codec, &begin_session(token, super_properties, session, &mut last_ack)).await?;
                    }
                    _ => {}
                }
            }
        }
    }
}

const HELLO_TIMEOUT: Duration = Duration::from_secs(20);

const MAX_RESUME_AGE: Duration = Duration::from_secs(3 * 60);

fn begin_session(
    token: &str,
    super_properties: &SuperProperties,
    session: &mut Session,
    last_ack: &mut Option<std_time::Instant>,
) -> serde_json::Value {
    let young = last_ack.is_none_or(|at| at.elapsed() <= MAX_RESUME_AGE);

    *last_ack = Some(TimeInstant::now());
    match young.then(|| session.resume_payload(token)).flatten() {
        Some(resume) => {
            log::info!(
                "gateway: resuming the session after sequence {:?}",
                session.seq()
            );
            resume
        }
        None => {
            session.clear();
            identify::build(token, super_properties)
        }
    }
}

async fn send<S>(
    write: &mut S,
    codec: &codec::Codec,
    payload: &serde_json::Value,
) -> Result<(), GatewayError>
where
    S: futures_util::Sink<WsMessage, Error = wreq::Error> + Unpin,
{
    write.send(codec.encode(payload)).await?;
    Ok(())
}

async fn close_with_4000<S>(write: &mut S)
where
    S: futures_util::Sink<WsMessage, Error = wreq::Error> + Unpin,
{
    let _ = write
        .send(WsMessage::Close(Some(CloseFrame {
            code: 4000u16.into(),
            reason: "".into(),
        })))
        .await;
}
