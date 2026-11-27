use crate::net_profile;
use crate::rest;
use std::time as std_time;
use std::time::Duration;
use std::time::SystemTime;
use tokio::time::MissedTickBehavior;
use tokio::time;
use wreq::ws;

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use wreq::ws::message::{CloseFrame, Message as WsMessage};

use super::opcodes::{self, EncryptionMode};

const VOICE_GATEWAY_VERSION: u8 = 10;

#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("websocket: {0}")]
    WebSocket(#[from] wreq::Error),
    #[error("voice gateway closed before HELLO")]
    ClosedBeforeHello,
    #[error("bad voice gateway payload: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("connection closed")]
    Closed,
}

pub struct VoiceGatewayConfig {
    pub endpoint: String,
    pub token: String,
    pub server_id: u64,
    pub channel_id: u64,
    pub user_id: u64,
    pub session_id: String,

    pub rtc_connection_id: String,

    pub video: bool,
}

#[derive(Debug)]
pub enum VoiceEvent {
    Hello {
        v: u8,
        heartbeat_interval: u64,
    },

    Ready {
        ssrc: u32,
        ip: String,
        port: u16,
        modes: Vec<EncryptionMode>,
        video_ssrc: Option<u32>,
    },
    SessionDescription {
        sdp: Option<String>,
        mode: Option<EncryptionMode>,
        secret_key: Vec<u8>,
        dave_protocol_version: u16,
    },
    Speaking {
        user_id: u64,
        ssrc: u32,
        speaking: u8,
    },
    ClientsConnect(Vec<u64>),
    ClientDisconnect(u64),
    Video {
        user_id: u64,
        audio_ssrc: u32,
        video_ssrc: u32,
        rtx_ssrc: u32,
    },
    SessionUpdate {
        audio_codec: Option<String>,
        video_codec: Option<String>,
        media_session_id: Option<String>,
    },
    VoiceBackendVersion {
        voice: Option<String>,
        rtc_worker: Option<String>,
    },
    DavePrepareTransition {
        transition_id: u16,
        protocol_version: u16,
    },
    DaveExecuteTransition {
        transition_id: u16,
    },
    DavePrepareEpoch {
        epoch: u32,
        protocol_version: u16,
    },

    DaveBinary {
        seq: u16,
        opcode: u8,
        payload: Vec<u8>,
    },

    HeartbeatAck {
        rtt_ms: Option<u64>,
    },
    Resumed,
    Closed {
        code: Option<u16>,
    },
}

enum Outgoing {
    Json { op: u8, d: Value },
    Binary(Vec<u8>),
    Close,
}

pub struct VoiceGateway {
    tx: UnboundedSender<Outgoing>,
    rtc_connection_id: String,
}

impl VoiceGateway {
    pub fn connect(
        client: &wreq::Client,
        config: VoiceGatewayConfig,
    ) -> Result<(Self, UnboundedReceiver<VoiceEvent>), GatewayError> {
        let rtc_connection_id = config.rtc_connection_id.clone();
        let (tx, rx) = unbounded_channel();
        let (event_tx, events) = unbounded_channel();
        tokio::spawn(run(client.clone(), config, event_tx, rx));
        Ok((
            Self {
                tx,
                rtc_connection_id,
            },
            events,
        ))
    }

    pub fn select_protocol_webrtc(&self, sdp_fragment: &str) {
        log::info!("voice: SELECT_PROTOCOL fragment={sdp_fragment:?}");
        self.send_json(
            opcodes::op::SELECT_PROTOCOL,
            json!({
                "protocol": "webrtc",
                "data": sdp_fragment,
                "sdp": sdp_fragment,
                "codecs": [
                    { "name": "opus", "type": "audio", "priority": 1000, "payload_type": 120 }
                ],
                "rtc_connection_id": self.rtc_connection_id,
            }),
        );
    }

    pub fn select_protocol_udp(&self, address: &str, port: u16, mode: EncryptionMode) {
        log::info!("voice: SELECT_PROTOCOL udp {address}:{port} mode={mode:?}");
        self.send_json(
            opcodes::op::SELECT_PROTOCOL,
            json!({
                "protocol": "udp",
                "data": { "address": address, "port": port, "mode": mode },
                "address": address,
                "port": port,
                "mode": mode,
                "codecs": [
                    { "name": "opus", "type": "audio", "priority": 1000, "payload_type": super::udp::OPUS_PAYLOAD_TYPE },
                    { "name": "VP8", "type": "video", "priority": 1000, "payload_type": super::video::VP8_PAYLOAD_TYPE, "rtx_payload_type": super::video::VP8_PAYLOAD_TYPE + 1, "encode": true, "decode": true },
                ],
                "rtc_connection_id": self.rtc_connection_id,
                "experiments": [],
            }),
        );
    }

    pub fn video(&self, audio_ssrc: u32, video_ssrc: u32, rtx_ssrc: u32) {
        self.send_json(
            opcodes::op::VIDEO,
            json!({ "audio_ssrc": audio_ssrc, "video_ssrc": video_ssrc, "rtx_ssrc": rtx_ssrc, "streams": [] }),
        );
    }

    pub fn speaking(&self, speaking: u8, ssrc: u32, delay: u32) {
        self.send_json(
            opcodes::op::SPEAKING,
            json!({ "speaking": speaking, "ssrc": ssrc, "delay": delay }),
        );
    }

    pub fn dave_transition_ready(&self, transition_id: u16) {
        self.send_json(
            opcodes::op::DAVE_TRANSITION_READY,
            json!({ "transition_id": transition_id }),
        );
    }

    pub fn dave_invalid_commit_welcome(&self, transition_id: u16) {
        self.send_json(
            opcodes::op::DAVE_MLS_INVALID_COMMIT_WELCOME,
            json!({ "transition_id": transition_id }),
        );
    }

    pub fn dave_binary(&self, opcode: u8, payload: Vec<u8>) {
        let mut frame = Vec::with_capacity(1 + payload.len());
        frame.push(opcode);
        frame.extend_from_slice(&payload);
        let _ = self.tx.send(Outgoing::Binary(frame));
    }

    pub fn no_route(&self) {
        self.send_json(opcodes::op::NO_ROUTE, json!({}));
    }

    pub fn send_json(&self, op: u8, d: Value) {
        let _ = self.tx.send(Outgoing::Json { op, d });
    }

    pub fn close(&self) {
        let _ = self.tx.send(Outgoing::Close);
    }
}

fn voice_websocket(client: &wreq::Client, url: &str) -> ws::WebSocketRequestBuilder {
    let mut emulation = rest::emulation_with_weight(220);
    if let Some(tls) = emulation.tls_options.as_mut() {
        tls.alps_protocols = None;
    }
    let (mut headers, order) = net_profile::websocket_handshake();
    headers.remove("sec-websocket-extensions");
    client
        .websocket(url)
        .emulation(emulation)
        .headers(headers)
        .orig_headers(order)
}

async fn run(
    client: wreq::Client,
    config: VoiceGatewayConfig,
    event_tx: UnboundedSender<VoiceEvent>,
    mut rx: UnboundedReceiver<Outgoing>,
) {
    let url = format!("wss://{}/?v={VOICE_GATEWAY_VERSION}", config.endpoint);
    if let Err(err) = run_inner(&client, &config, &url, &event_tx, &mut rx).await {
        log::warn!("voice: gateway ended: {err}");
    }
    let _ = event_tx.send(VoiceEvent::Closed { code: None });
}

async fn run_inner(
    client: &wreq::Client,
    config: &VoiceGatewayConfig,
    url: &str,
    event_tx: &UnboundedSender<VoiceEvent>,
    rx: &mut UnboundedReceiver<Outgoing>,
) -> Result<(), GatewayError> {
    let upgrade = voice_websocket(client, url).send().await?;
    let ws = upgrade.into_websocket().await?;
    let (mut write, mut read) = ws.split();

    let hello_text = time::timeout(Duration::from_secs(20), async {
        loop {
            match read.next().await.ok_or(GatewayError::ClosedBeforeHello)?? {
                WsMessage::Close(_) => return Err(GatewayError::ClosedBeforeHello),
                WsMessage::Text(text) => return Ok(text.to_string()),
                _ => continue,
            }
        }
    })
    .await
    .map_err(|_| GatewayError::ClosedBeforeHello)??;

    let hello: RawPayload = serde_json::from_str(&hello_text)?;
    if hello.op != opcodes::recv::HELLO {
        return Err(GatewayError::ClosedBeforeHello);
    }
    let hello_d = hello.d.unwrap_or_default();
    let heartbeat_interval = hello_d
        .get("heartbeat_interval")
        .and_then(Value::as_u64)
        .unwrap_or(13_750);
    let version = hello_d.get("v").and_then(Value::as_u64).unwrap_or(0) as u8;
    let _ = event_tx.send(VoiceEvent::Hello {
        v: version,
        heartbeat_interval,
    });

    send_json(&mut write, opcodes::op::IDENTIFY, &identify_payload(config)).await?;

    let mut last_seq: Option<u64> = None;
    let mut beat = time::interval(Duration::from_millis(heartbeat_interval));
    beat.set_missed_tick_behavior(MissedTickBehavior::Skip);

    beat.tick().await;

    loop {
        tokio::select! {
            _ = beat.tick() => {
                let t = unix_millis();
                let payload = heartbeat_payload(version, t, last_seq);
                log::debug!("voice: heartbeat {payload}");
                send_json(&mut write, opcodes::op::HEARTBEAT, &payload).await?;
            }
            outgoing = rx.recv() => {
                match outgoing {
                    Some(Outgoing::Json { op, d }) => send_json(&mut write, op, &d).await?,
                    Some(Outgoing::Binary(payload)) => {
                        let _ = write.send(WsMessage::Binary(payload.into())).await;
                    }
                    Some(Outgoing::Close) | None => {
                        let _ = write.send(WsMessage::Close(Some(CloseFrame { code: 4000u16.into(), reason: "".into() }))).await;
                        return Err(GatewayError::Closed);
                    }
                }
            }
            msg = read.next() => {
                match msg {
                    None => return Err(GatewayError::Closed),
                    Some(Ok(WsMessage::Close(frame))) => {
                        let code = frame.as_ref().map(|f| u16::from(f.code.clone()));
                        let reason = frame.as_ref().map(|f| f.reason.as_str().to_string());
                        log::info!("voice: gateway closed by server: code {:?} reason {:?}", code, reason);
                        let _ = event_tx.send(VoiceEvent::Closed { code });
                        return Err(GatewayError::Closed);
                    }
                    Some(Ok(WsMessage::Text(text))) => handle_text(text.as_str(), &mut last_seq, event_tx),
                    Some(Ok(WsMessage::Binary(bytes))) => handle_binary(&bytes, &mut last_seq, event_tx),
                    Some(Ok(_)) => {}
                    Some(Err(err)) => {
                        log::warn!("voice: gateway read error: {err}");
                        return Err(GatewayError::Closed);
                    }
                }
            }
        }
    }
}

#[derive(Deserialize)]
struct RawPayload {
    op: u8,
    #[serde(default)]
    d: Option<Value>,
    #[serde(default)]
    s: Option<u64>,
}

fn identify_payload(config: &VoiceGatewayConfig) -> Value {
    json!({
        "server_id": config.server_id.to_string(),
        "channel_id": config.channel_id.to_string(),
        "user_id": config.user_id.to_string(),
        "session_id": config.session_id,
        "token": config.token,
        "max_dave_protocol_version": super::dave::MAX_DAVE_PROTOCOL_VERSION,
        "video": config.video,

        "streams": [{ "type": "video", "rid": "100", "quality": 100 }]
    })
}

fn unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(std_time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn heartbeat_payload(version: u8, t: u64, last_seq: Option<u64>) -> Value {
    if version >= 8 {
        json!({ "t": t, "seq_ack": last_seq.map(|seq| seq as i64).unwrap_or(-1) })
    } else {
        json!(t)
    }
}

fn handle_text(text: &str, last_seq: &mut Option<u64>, tx: &UnboundedSender<VoiceEvent>) {
    let Ok(raw) = serde_json::from_str::<RawPayload>(text) else {
        log::warn!("voice: gateway text was not a payload ({}B)", text.len());
        return;
    };
    if let Some(s) = raw.s {
        note_seq(last_seq, s);
    }
    log::debug!(
        "voice: opcode {} ({}) seq={:?}",
        raw.op,
        opcode_name(raw.op),
        raw.s
    );
    let Some(d) = raw.d else {
        log::debug!("voice: opcode {} has no data", raw.op);
        return;
    };
    match raw.op {
        opcodes::recv::READY => {
            #[derive(Deserialize)]
            struct S {
                ssrc: u32,
            }
            #[derive(Deserialize)]
            struct W {
                ssrc: u32,
                ip: String,
                port: u16,
                #[serde(default)]
                modes: Vec<EncryptionMode>,
                #[serde(default)]
                streams: Vec<S>,
            }
            if let Ok(w) = serde_json::from_value::<W>(d.clone()) {
                let video_ssrc = w.streams.first().map(|s| s.ssrc);
                let _ = tx.send(VoiceEvent::Ready {
                    ssrc: w.ssrc,
                    ip: w.ip,
                    port: w.port,
                    modes: w.modes,
                    video_ssrc,
                });
            } else {
                log::warn!("voice: READY didn't parse: {}", keys_of(&d));
            }
        }
        opcodes::recv::SESSION_DESCRIPTION => {
            #[derive(Deserialize)]
            struct W {
                sdp: Option<String>,
                mode: Option<EncryptionMode>,
                #[serde(default)]
                secret_key: Vec<u8>,
                dave_protocol_version: Option<u16>,
            }
            if let Ok(w) = serde_json::from_value::<W>(d.clone()) {
                let _ = tx.send(VoiceEvent::SessionDescription {
                    sdp: w.sdp,
                    mode: w.mode,
                    secret_key: w.secret_key,
                    dave_protocol_version: w.dave_protocol_version.unwrap_or(0),
                });
            } else {
                log::warn!("voice: SESSION_DESCRIPTION didn't parse: {}", keys_of(&d));
            }
        }
        opcodes::recv::SPEAKING => {
            if let Ok(w) = serde_json::from_value::<opcodes::Speaking>(d.clone()) {
                if let Some(user_id) = w.user_id {
                    let _ = tx.send(VoiceEvent::Speaking {
                        user_id: user_id.get(),
                        ssrc: w.ssrc,
                        speaking: w.speaking,
                    });
                } else {
                    log::debug!("voice: SPEAKING without a user id (ssrc={})", w.ssrc);
                }
            } else {
                log::warn!("voice: SPEAKING didn't parse: {}", keys_of(&d));
            }
        }
        opcodes::recv::CLIENTS_CONNECT => {
            if let Ok(w) = serde_json::from_value::<opcodes::ClientsConnect>(d) {
                let ids: Vec<u64> = w.user_ids.iter().map(|id| id.get()).collect();
                let _ = tx.send(VoiceEvent::ClientsConnect(ids));
            }
        }
        opcodes::recv::CLIENT_DISCONNECT => {
            if let Ok(w) = serde_json::from_value::<opcodes::ClientDisconnect>(d) {
                let _ = tx.send(VoiceEvent::ClientDisconnect(w.user_id.get()));
            }
        }
        opcodes::recv::VIDEO => {
            if let Ok(w) = serde_json::from_value::<opcodes::Video>(d) {
                if let Some(user_id) = w.user_id {
                    let _ = tx.send(VoiceEvent::Video {
                        user_id: user_id.get(),
                        audio_ssrc: w.audio_ssrc,
                        video_ssrc: w.video_ssrc,
                        rtx_ssrc: w.rtx_ssrc,
                    });
                }
            }
        }
        opcodes::recv::SESSION_UPDATE => {
            if let Ok(w) = serde_json::from_value::<opcodes::SessionUpdate>(d) {
                let _ = tx.send(VoiceEvent::SessionUpdate {
                    audio_codec: w.audio_codec,
                    video_codec: w.video_codec,
                    media_session_id: w.media_session_id,
                });
            }
        }
        opcodes::recv::VOICE_BACKEND_VERSION => {
            if let Ok(w) = serde_json::from_value::<opcodes::VoiceBackendVersion>(d) {
                let _ = tx.send(VoiceEvent::VoiceBackendVersion {
                    voice: w.voice,
                    rtc_worker: w.rtc_worker,
                });
            }
        }
        opcodes::recv::HEARTBEAT_ACK => {
            let t = d.get("t").and_then(Value::as_u64).or_else(|| d.as_u64());
            let rtt_ms = t.map(|t| unix_millis().saturating_sub(t));
            log::debug!("voice: heartbeat ack rtt={rtt_ms:?}");
            let _ = tx.send(VoiceEvent::HeartbeatAck { rtt_ms });
        }
        opcodes::recv::RESUMED => {
            let _ = tx.send(VoiceEvent::Resumed);
        }
        opcodes::recv::DAVE_PREPARE_TRANSITION => {
            if let Ok(w) = serde_json::from_value::<opcodes::DavePrepareTransition>(d) {
                let _ = tx.send(VoiceEvent::DavePrepareTransition {
                    transition_id: w.transition_id,
                    protocol_version: w.protocol_version,
                });
            }
        }
        opcodes::recv::DAVE_EXECUTE_TRANSITION => {
            if let Ok(w) = serde_json::from_value::<opcodes::DaveExecuteTransition>(d) {
                let _ = tx.send(VoiceEvent::DaveExecuteTransition {
                    transition_id: w.transition_id,
                });
            }
        }
        opcodes::recv::DAVE_PREPARE_EPOCH => {
            if let Ok(w) = serde_json::from_value::<opcodes::DavePrepareEpoch>(d) {
                let _ = tx.send(VoiceEvent::DavePrepareEpoch {
                    epoch: w.epoch,
                    protocol_version: w.protocol_version,
                });
            }
        }
        opcodes::recv::HELLO => {}
        other => log::debug!("voice: unhandled opcode {other}"),
    }
}

fn handle_binary(bytes: &[u8], last_seq: &mut Option<u64>, tx: &UnboundedSender<VoiceEvent>) {
    let Some(frame) = opcodes::BinaryFrame::parse(bytes) else {
        log::debug!("voice: binary frame didn't parse ({}B)", bytes.len());
        return;
    };
    log::debug!(
        "voice: binary opcode {} seq={} ({}B)",
        frame.opcode,
        frame.seq,
        frame.payload.len()
    );
    note_seq(last_seq, u64::from(frame.seq));
    let _ = tx.send(VoiceEvent::DaveBinary {
        seq: frame.seq,
        opcode: frame.opcode,
        payload: frame.payload.to_vec(),
    });
}
fn note_seq(last_seq: &mut Option<u64>, seq: u64) {
    if last_seq.is_none_or(|seen| seq > seen) {
        *last_seq = Some(seq);
    }
}

fn opcode_name(op: u8) -> &'static str {
    match op {
        opcodes::recv::READY => "READY",
        opcodes::recv::HEARTBEAT => "HEARTBEAT",
        opcodes::recv::SESSION_DESCRIPTION => "SESSION_DESCRIPTION",
        opcodes::recv::SPEAKING => "SPEAKING",
        opcodes::recv::HEARTBEAT_ACK => "HEARTBEAT_ACK",
        opcodes::recv::HELLO => "HELLO",
        opcodes::recv::RESUMED => "RESUMED",
        opcodes::recv::CLIENTS_CONNECT => "CLIENTS_CONNECT",
        opcodes::recv::VIDEO => "VIDEO",
        opcodes::recv::CLIENT_DISCONNECT => "CLIENT_DISCONNECT",
        opcodes::recv::SESSION_UPDATE => "SESSION_UPDATE",
        opcodes::recv::MEDIA_SINK_WANTS => "MEDIA_SINK_WANTS",
        opcodes::recv::VOICE_BACKEND_VERSION => "VOICE_BACKEND_VERSION",
        opcodes::recv::FLAGS => "FLAGS",
        opcodes::recv::PLATFORM => "PLATFORM",
        opcodes::recv::DAVE_PREPARE_TRANSITION => "DAVE_PREPARE_TRANSITION",
        opcodes::recv::DAVE_EXECUTE_TRANSITION => "DAVE_EXECUTE_TRANSITION",
        opcodes::recv::DAVE_PREPARE_EPOCH => "DAVE_PREPARE_EPOCH",
        opcodes::recv::SFU_UPDATE => "SFU_UPDATE",
        _ => "unknown",
    }
}

fn keys_of(value: &Value) -> String {
    value
        .as_object()
        .map(|map| map.keys().map(String::as_str).collect::<Vec<_>>().join(","))
        .unwrap_or_else(|| value.to_string())
}

async fn send_json<S>(write: &mut S, op: u8, d: &Value) -> Result<(), GatewayError>
where
    S: futures_util::Sink<WsMessage, Error = wreq::Error> + Unpin,
{
    let payload = json!({ "op": op, "d": d });
    write
        .send(WsMessage::Text(payload.to_string().into()))
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> VoiceGatewayConfig {
        VoiceGatewayConfig {
            endpoint: "example.discord.media".into(),
            token: "tok".into(),
            server_id: 123,
            channel_id: 456,
            user_id: 789,
            session_id: "sess".into(),
            rtc_connection_id: "rtc".into(),
            video: true,
        }
    }

    #[test]
    fn identify_uses_string_snowflakes_and_dave() {
        let d = identify_payload(&config());
        assert_eq!(d["server_id"], "123");
        assert_eq!(d["channel_id"], "456");
        assert_eq!(d["user_id"], "789");
        assert_eq!(d["session_id"], "sess");
        assert_eq!(d["token"], "tok");
        assert_eq!(
            d["max_dave_protocol_version"],
            super::super::dave::MAX_DAVE_PROTOCOL_VERSION
        );
        assert_eq!(d["video"], true);
        assert_eq!(
            d["streams"],
            json!([{ "type": "video", "rid": "100", "quality": 100 }])
        );
    }

    #[test]
    fn heartbeat_acks_the_last_sequence_on_v8() {
        assert_eq!(
            heartbeat_payload(8, 123, None),
            json!({ "t": 123, "seq_ack": -1 })
        );
        assert_eq!(
            heartbeat_payload(8, 123, Some(6)),
            json!({ "t": 123, "seq_ack": 6 })
        );
        assert_eq!(heartbeat_payload(4, 123, Some(6)), json!(123));
    }

    #[test]
    fn parses_ready() {
        let (tx, mut rx) = unbounded_channel();
        let mut seq = None;
        handle_text(
            r#"{"op":2,"s":1,"d":{"ssrc":999,"ip":"10.0.0.1","port":50000,"modes":["aead_aes256_gcm_rtpsize","aead_xchacha20_poly1305_rtpsize"]}}"#,
            &mut seq,
            &tx,
        );
        assert_eq!(seq, Some(1));
        match rx.try_recv().unwrap() {
            VoiceEvent::Ready {
                ssrc,
                ip,
                port,
                modes,
                ..
            } => {
                assert_eq!(ssrc, 999);
                assert_eq!(ip, "10.0.0.1");
                assert_eq!(port, 50000);
                assert_eq!(modes.len(), 2);
            }
            other => panic!("expected Ready, got {other:?}"),
        }
    }

    #[test]
    fn parses_session_description() {
        let (tx, mut rx) = unbounded_channel();
        let mut seq = None;
        handle_text(
            r#"{"op":4,"d":{"sdp":"v=0\r\nm=audio 9 UDP/TLS/RTP/SAVPF 120\r\n","mode":"aead_aes256_gcm_rtpsize","secret_key":[1,2,3],"dave_protocol_version":1}}"#,
            &mut seq,
            &tx,
        );
        match rx.try_recv().unwrap() {
            VoiceEvent::SessionDescription {
                sdp,
                mode,
                secret_key,
                dave_protocol_version,
            } => {
                assert!(sdp.is_some());
                assert_eq!(mode, Some(EncryptionMode::AeadAes256GcmRtpsize));
                assert_eq!(secret_key, vec![1, 2, 3]);
                assert_eq!(dave_protocol_version, 1);
            }
            other => panic!("expected SessionDescription, got {other:?}"),
        }
    }

    #[test]
    fn parses_webrtc_session_description_without_secret_key() {
        let (tx, mut rx) = unbounded_channel();
        let mut seq = None;
        handle_text(
            r#"{"op":4,"d":{"sdp":"m=audio 50000 ICE/SDP\r\n","audio_codec":"opus","video_codec":"H264"}}"#,
            &mut seq,
            &tx,
        );
        match rx.try_recv().unwrap() {
            VoiceEvent::SessionDescription {
                sdp, secret_key, ..
            } => {
                assert!(sdp.is_some());
                assert!(secret_key.is_empty());
            }
            other => panic!("expected SessionDescription, got {other:?}"),
        }
    }

    #[test]
    fn parses_speaking_and_clients_connect() {
        let (tx, mut rx) = unbounded_channel();
        let mut seq = None;
        handle_text(
            r#"{"op":5,"d":{"user_id":"789","ssrc":999,"speaking":1}}"#,
            &mut seq,
            &tx,
        );
        handle_text(r#"{"op":11,"d":{"user_ids":["111","222"]}}"#, &mut seq, &tx);
        match rx.try_recv().unwrap() {
            VoiceEvent::Speaking {
                user_id,
                ssrc,
                speaking,
            } => {
                assert_eq!((user_id, ssrc, speaking), (789, 999, 1));
            }
            other => panic!("expected Speaking, got {other:?}"),
        }
        match rx.try_recv().unwrap() {
            VoiceEvent::ClientsConnect(ids) => assert_eq!(ids, vec![111, 222]),
            other => panic!("expected ClientsConnect, got {other:?}"),
        }
    }

    #[test]
    fn parses_dave_binary_frame() {
        let (tx, mut rx) = unbounded_channel();
        let mut seq = None;

        let frame = vec![0x00, 0x07, 25, 0xAA, 0xBB];
        handle_binary(&frame, &mut seq, &tx);
        match rx.try_recv().unwrap() {
            VoiceEvent::DaveBinary {
                seq,
                opcode,
                payload,
            } => {
                assert_eq!(seq, 7);
                assert_eq!(opcode, 25);
                assert_eq!(payload, vec![0xAA, 0xBB]);
            }
            other => panic!("expected DaveBinary, got {other:?}"),
        }
    }

    #[test]
    fn binary_seq_feeds_the_heartbeat_ack() {
        let (tx, _rx) = unbounded_channel();
        let mut seq = Some(4);
        handle_binary(&[0x00, 0x06, 27, 0x01], &mut seq, &tx);
        assert_eq!(seq, Some(6));
        handle_binary(&[0x00, 0x05, 27, 0x01], &mut seq, &tx);
        assert_eq!(seq, Some(6));
    }

    #[test]
    fn ignores_truncated_binary_frame() {
        let (tx, rx) = unbounded_channel();
        let mut seq = None;

        handle_binary(&[0x00, 0x07], &mut seq, &tx);
        assert!(rx.is_empty());
        assert_eq!(seq, None);
    }
}
