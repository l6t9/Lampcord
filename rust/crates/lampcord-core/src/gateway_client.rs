use std::sync::{Arc, Mutex};
use std::time::Duration;

use lampcord_model::snowflake::Snowflake;
use serde_json::{Value, json};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    event_dispatch::Dispatcher,
    gateway_plan::{
        HELLO_TIMEOUT_MS, OP_DISPATCH, OP_HEARTBEAT, OP_IDENTIFY, OP_RESUME, control_action,
        gateway_url, resume_decision,
    },
    gateway_session::GatewaySession,
    rest::DESKTOP_USER_AGENT,
    state_cache::{StateCache, StateCacheHandler},
    traits::{GatewayMessage, GatewayTransport, GatewayUrlProvider, TransportError},
};

pub const DEFAULT_GATEWAY_URL: &str = "wss://gateway.discord.gg";

pub const OP_LAZY_REQUEST: i64 = 14;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayIdentityMode {
    Identify,
    Resume { session_id: String, sequence: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayEvent {
    Hello {
        heartbeat_interval_ms: u64,
        mode: GatewayIdentityMode,
    },
    HeartbeatSent,
    HeartbeatAck,
    Ready {
        session_id: String,
        resume_gateway_url: Option<String>,
        user_id: Option<String>,
    },
    Resumed,
    Dispatch {
        event_type: String,
        data: Option<Arc<serde_json::Value>>,
    },
    InvalidSession {
        resumable: bool,
    },
    ReconnectRequested,
    AuthFailed {
        token: String,
    },
    Closed {
        code: Option<u16>,
    },
    HandlerErrors {
        errors: Vec<crate::event_dispatch::HandlerError>,
    },
    Failed {
        message: String,
    },
}

#[derive(Debug)]
pub enum GatewayError {
    Transport(TransportError),
    InvalidPayload(serde_json::Error),
    StateCache(String),
}

impl std::fmt::Display for GatewayError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(error) => write!(formatter, "gateway transport failed: {error}"),
            Self::InvalidPayload(error) => write!(formatter, "invalid gateway payload: {error}"),
            Self::StateCache(error) => write!(formatter, "state cache failed: {error}"),
        }
    }
}

impl std::error::Error for GatewayError {}

impl From<TransportError> for GatewayError {
    fn from(error: TransportError) -> Self {
        Self::Transport(error)
    }
}

impl From<serde_json::Error> for GatewayError {
    fn from(error: serde_json::Error) -> Self {
        Self::InvalidPayload(error)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GatewayIdentity {
    pub properties: Value,
    pub client_state: Value,
    pub capabilities: u64,
    pub large_threshold: u8,
    pub compress: bool,
    /// Used for op 41 time-spent/session updates (matches identify properties).
    pub client_heartbeat_session_id: String,
}

impl Default for GatewayIdentity {
    fn default() -> Self {
        Self::desktop(None)
    }
}

impl GatewayIdentity {
    /// Build a desktop identify payload matching Lampcord/Kotlin as closely as practical.
    pub fn desktop(device_vendor_id: Option<String>) -> Self {
        use rand::RngCore;
        fn random_uuid() -> String {
            let mut bytes = [0u8; 16];
            rand::rngs::OsRng.fill_bytes(&mut bytes);
            bytes[6] = (bytes[6] & 0x0f) | 0x40;
            bytes[8] = (bytes[8] & 0x3f) | 0x80;
            format!(
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                bytes[0],
                bytes[1],
                bytes[2],
                bytes[3],
                bytes[4],
                bytes[5],
                bytes[6],
                bytes[7],
                bytes[8],
                bytes[9],
                bytes[10],
                bytes[11],
                bytes[12],
                bytes[13],
                bytes[14],
                bytes[15]
            )
        }

        let client_launch_id = random_uuid();
        let client_heartbeat_session_id = random_uuid();
        let launch_signature = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos())
        .to_string();

        let cpu_cores = std::thread::available_parallelism()
            .map(|n| n.get() as i64)
            .unwrap_or(0);
        let mem_kb = std::fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("MemTotal:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|v| v.parse::<i64>().ok())
            })
            .unwrap_or(0);

        let os_version = std::process::Command::new("uname")
            .arg("-r")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                std::fs::read_to_string("/etc/os-release")
                    .ok()
                    .and_then(|s| {
                        s.lines()
                            .find_map(|line| line.strip_prefix("VERSION_ID="))
                            .map(|v| v.trim_matches('"').to_owned())
                    })
            })
            .unwrap_or_default();

        let mut properties = json!({
            "os": "Linux",
            "browser": "Discord Desktop",
            "device": "",
            "system_locale": "en-US",
            "has_client_mods": false,
            "client_version": "0.0.309",
            "release_channel": "stable",
            "design_id": 0,
            "browser_user_agent": "",
            "browser_version": "",
            "os_version": os_version,
            "client_build_number": 309000,
            "client_event_source": Value::Null,
            "client_launch_id": client_launch_id,
            "launch_signature": launch_signature,
            "client_heartbeat_session_id": client_heartbeat_session_id,
            "client_app_state": "active",
            "accessibility_features": 0,
            "accessibility_support_enabled": false,
            "client_performance_cpu": cpu_cores,
            "client_performance_memory": mem_kb,
            "cpu_core_count": cpu_cores,
        });
        if let Some(id) = device_vendor_id {
            if let Some(obj) = properties.as_object_mut() {
                obj.insert("device_vendor_id".to_owned(), Value::String(id));
            }
        }

        Self {
            properties,
            client_state: json!({
                "guild_hashes": {},
                "highest_last_message_id": 0,
                "read_state_version": 0,
                "user_guild_settings_version": -1,
            }),
            capabilities: 351,
            large_threshold: 100,
            compress: false,
            client_heartbeat_session_id,
        }
    }
}

impl GatewayIdentity {
    pub fn identify_payload(&self, token: &str) -> Value {
        json!({
            "op": OP_IDENTIFY,
            "d": {
                "token": token,
                "properties": self.properties,
                "compress": self.compress,
                "large_threshold": self.large_threshold,
                "capabilities": self.capabilities,
                "client_state": self.client_state,
            }
        })
    }

    pub fn resume_payload(token: &str, session_id: &str, sequence: u32) -> Value {
        json!({
            "op": OP_RESUME,
            "d": {
                "token": token,
                "session_id": session_id,
                "seq": sequence,
            }
        })
    }
}

pub fn heartbeat_payload(sequence: Option<u32>) -> Value {
    json!({
        "op": OP_HEARTBEAT,
        "d": sequence,
    })
}

pub fn lazy_request_payload(
    guild_id: Snowflake,
    channel_id: Snowflake,
    ranges: &[[u32; 2]],
) -> Value {
    let mut channels = serde_json::Map::new();
    channels.insert(channel_id.to_string(), json!(ranges));
    json!({
        "op": OP_LAZY_REQUEST,
        "d": {
            "guild_id": guild_id.to_string(),
            "typing": true,
            "activities": true,
            "threads": true,
            "members": [],
            "channels": channels,
        }
    })
}

pub fn time_spent_payload(client_heartbeat_session_id: &str) -> Value {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    json!({
        "op": 41,
        "d": {
            "initialization_timestamp": now_ms,
            "session_id": client_heartbeat_session_id,
            "client_launch_id": client_heartbeat_session_id,
        }
    })
}

pub fn gateway_headers() -> Vec<(String, String)> {
    vec![
        ("User-Agent".to_owned(), DESKTOP_USER_AGENT.to_owned()),
        ("Origin".to_owned(), "https://discord.com".to_owned()),
        ("Accept-Language".to_owned(), "en-US,en;q=0.9".to_owned()),
        ("Cache-Control".to_owned(), "no-cache".to_owned()),
    ]
}

pub struct GatewayClient<T: GatewayTransport, U: GatewayUrlProvider> {
    transport: T,
    url_provider: U,
    session: GatewaySession,
    identity: GatewayIdentity,
    dispatcher: Dispatcher,
    cache: Arc<Mutex<StateCache>>,
    outbound: Option<UnboundedReceiver<Value>>,
}

impl<T: GatewayTransport, U: GatewayUrlProvider> GatewayClient<T, U> {
    pub fn new(transport: T, url_provider: U) -> Self {
        let cache = Arc::new(Mutex::new(StateCache::new()));
        let mut dispatcher = Dispatcher::new();
        dispatcher.register(Arc::new(StateCacheHandler::new(cache.clone())));
        Self {
            transport,
            url_provider,
            session: GatewaySession::new(),
            identity: GatewayIdentity::default(),
            dispatcher,
            cache,
            outbound: None,
        }
    }

    pub fn with_identity(mut self, identity: GatewayIdentity) -> Self {
        self.identity = identity;
        self
    }

    pub fn with_outbound(mut self, outbound: UnboundedReceiver<Value>) -> Self {
        self.outbound = Some(outbound);
        self
    }

    pub fn session(&self) -> &GatewaySession {
        &self.session
    }

    pub fn cache(&self) -> &Arc<Mutex<StateCache>> {
        &self.cache
    }

    /// Runs the gateway connection until it closes and returns all emitted events.
    /// Suitable for tests and short-lived scripted runs.
    pub async fn run(&mut self, token: &str) -> Result<Vec<GatewayEvent>, GatewayError> {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let run_result = self.run_stream(token, tx).await;
        let mut events = Vec::new();
        while let Ok(event) = rx.try_recv() {
            events.push(event);
        }
        run_result?;
        Ok(events)
    }

    /// Runs the gateway connection, emitting events incrementally to `tx`.
    /// Maintains periodic heartbeats once a HELLO interval is known.
    /// Returns the terminal gateway event when the connection ends normally.
    pub async fn run_stream(
        &mut self,
        token: &str,
        tx: UnboundedSender<GatewayEvent>,
    ) -> Result<Option<GatewayEvent>, GatewayError> {
        self.session.switch_token(token);
        {
            let mut cache = self
                .cache
                .lock()
                .map_err(|error| GatewayError::StateCache(error.to_string()))?;
            cache.switch_token(token);
        }

        let base = self
            .url_provider
            .gateway_url()
            .await
            .unwrap_or_else(|| DEFAULT_GATEWAY_URL.to_owned());
        let url = gateway_url(
            &base,
            self.session
                .resume_info()
                .map(|(_, _, resume_url)| resume_url),
        );
        self.transport.connect(&url, &gateway_headers()).await?;

        let mut heartbeat_interval_ms: Option<u64> = None;
        let mut next_heartbeat_at =
            tokio::time::Instant::now() + Duration::from_millis(HELLO_TIMEOUT_MS);

        loop {
            let now = tokio::time::Instant::now();
            let wait = if heartbeat_interval_ms.is_some() {
                next_heartbeat_at.saturating_duration_since(now)
            } else {
                // Wait for HELLO, but don't block forever in live runs.
                (next_heartbeat_at.max(now)).saturating_duration_since(now)
            };

            let outbound = &mut self.outbound;
            let outbound_next = async move {
                match outbound.as_mut() {
                    Some(rx) => rx.recv().await,
                    None => std::future::pending().await,
                }
            };
            let received = tokio::select! {
                biased;
                Some(payload) = outbound_next => {
                    self.transport.send(&payload.to_string()).await?;
                    continue;
                }
                result = tokio::time::timeout(wait, self.transport.receive()) => result,
            };
            match received {
                Ok(received) => match received? {
                    GatewayMessage::Text(text) => {
                        let mut payload: Value = serde_json::from_str(&text)?;
                        let op = payload
                            .get("op")
                            .and_then(Value::as_i64)
                            .unwrap_or_default();

                        if let Some(sequence) = payload.get("s").and_then(Value::as_u64) {
                            self.session.record_sequence(sequence as u32);
                        }

                        let action = control_action(
                            op,
                            payload.get("d"),
                            self.transport.close_code(),
                            token,
                        );

                        match action {
                            crate::gateway_plan::ControlAction::Hello {
                                heartbeat_interval_ms: interval,
                            } => {
                                heartbeat_interval_ms = Some(interval);
                                next_heartbeat_at =
                                    tokio::time::Instant::now() + Duration::from_millis(interval);
                                let mode = match resume_decision(self.session.resume_info()) {
                                    crate::gateway_plan::ResumeDecision::Resume {
                                        session_id,
                                        sequence,
                                    } => {
                                        let payload = GatewayIdentity::resume_payload(
                                            token, session_id, sequence,
                                        );
                                        self.transport.send(&payload.to_string()).await?;
                                        GatewayIdentityMode::Resume {
                                            session_id: session_id.to_owned(),
                                            sequence,
                                        }
                                    }
                                    crate::gateway_plan::ResumeDecision::Identify => {
                                        let payload = self.identity.identify_payload(token);
                                        self.transport.send(&payload.to_string()).await?;
                                        GatewayIdentityMode::Identify
                                    }
                                };
                                let _ = tx.send(GatewayEvent::Hello {
                                    heartbeat_interval_ms: interval,
                                    mode,
                                });
                            }
                            crate::gateway_plan::ControlAction::Heartbeat => {
                                let payload = heartbeat_payload(self.session.last_sequence());
                                self.transport.send(&payload.to_string()).await?;
                                let _ = tx.send(GatewayEvent::HeartbeatSent);
                                if let Some(interval) = heartbeat_interval_ms {
                                    next_heartbeat_at = tokio::time::Instant::now()
                                        + Duration::from_millis(interval);
                                }
                            }
                            crate::gateway_plan::ControlAction::HeartbeatAck => {
                                let _ = tx.send(GatewayEvent::HeartbeatAck);
                            }
                            crate::gateway_plan::ControlAction::InvalidSession { resumable } => {
                                if !resumable {
                                    self.session.clear_resume();
                                }
                                let event = GatewayEvent::InvalidSession { resumable };
                                let _ = tx.send(event.clone());
                                // Let supervisor decide whether/how to reconnect.
                                let _ = self.transport.close(1000, "invalid session").await;
                                return Ok(Some(event));
                            }
                            crate::gateway_plan::ControlAction::Reconnect => {
                                self.transport
                                    .close(1000, "server requested reconnect")
                                    .await?;
                                let event = GatewayEvent::ReconnectRequested;
                                let _ = tx.send(event.clone());
                                return Ok(Some(event));
                            }
                            crate::gateway_plan::ControlAction::AuthFailed { token } => {
                                let event = GatewayEvent::AuthFailed { token };
                                let _ = tx.send(event.clone());
                                return Ok(Some(event));
                            }
                            crate::gateway_plan::ControlAction::None => {
                                if op == OP_DISPATCH {
                                    let event_type = payload
                                        .get("t")
                                        .and_then(Value::as_str)
                                        .unwrap_or_default()
                                        .to_owned();
                                    let data = payload
                                        .as_object_mut()
                                        .and_then(|object| object.remove("d"));
                                    let errors =
                                        self.dispatcher.dispatch(Some(&event_type), data.as_ref());
                                    if !errors.is_empty() {
                                        let _ = tx.send(GatewayEvent::HandlerErrors { errors });
                                    }
                                    match event_type.as_str() {
                                        "READY" => {
                                            let data = data.as_ref();
                                            let session_id = data
                                                .and_then(|data| data.get("session_id"))
                                                .and_then(Value::as_str)
                                                .unwrap_or_default()
                                                .to_owned();
                                            let resume_gateway_url = data
                                                .and_then(|data| data.get("resume_gateway_url"))
                                                .and_then(Value::as_str)
                                                .map(str::to_owned);
                                            let user_id = data
                                                .and_then(|data| data.get("user"))
                                                .and_then(|user| user.get("id"))
                                                .and_then(Value::as_str)
                                                .map(str::to_owned);
                                            self.session.record_ready(
                                                session_id.clone(),
                                                resume_gateway_url.clone(),
                                            );
                                            // Match Lampcord/Kotlin: send time-spent/session update on READY.
                                            let op41 = time_spent_payload(
                                                &self.identity.client_heartbeat_session_id,
                                            );
                                            let _ = self.transport.send(&op41.to_string()).await;
                                            let _ = tx.send(GatewayEvent::Ready {
                                                session_id,
                                                resume_gateway_url,
                                                user_id,
                                            });
                                        }
                                        "RESUMED" => {
                                            let op41 = time_spent_payload(
                                                &self.identity.client_heartbeat_session_id,
                                            );
                                            let _ = self.transport.send(&op41.to_string()).await;
                                            let _ = tx.send(GatewayEvent::Resumed);
                                        }
                                        _ => {
                                            let _ = tx.send(GatewayEvent::Dispatch {
                                                event_type: event_type.clone(),
                                                data: data.map(Arc::new),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    GatewayMessage::Closed => {
                        let event = if self
                            .transport
                            .close_code()
                            .is_some_and(crate::gateway_plan::is_auth_failed_close)
                        {
                            GatewayEvent::AuthFailed {
                                token: token.to_owned(),
                            }
                        } else {
                            GatewayEvent::Closed {
                                code: self.transport.close_code(),
                            }
                        };
                        let _ = tx.send(event.clone());
                        return Ok(Some(event));
                    }
                },
                Err(_elapsed) => {
                    if let Some(interval) = heartbeat_interval_ms {
                        let payload = heartbeat_payload(self.session.last_sequence());
                        self.transport.send(&payload.to_string()).await?;
                        let _ = tx.send(GatewayEvent::HeartbeatSent);
                        next_heartbeat_at =
                            tokio::time::Instant::now() + Duration::from_millis(interval);
                    } else {
                        // Still waiting for HELLO; keep waiting.
                        next_heartbeat_at =
                            tokio::time::Instant::now() + Duration::from_millis(HELLO_TIMEOUT_MS);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, str::FromStr, sync::Mutex};

    use async_trait::async_trait;
    use serde_json::{Value, json};

    use super::*;
    use crate::traits::{GatewayMessage, GatewayTransport, GatewayUrlProvider, TransportError};

    const CLOSED: &str = "__closed__";

    #[derive(Default)]
    struct ScriptedGatewayTransport {
        frames: Mutex<VecDeque<String>>,
        sent: Mutex<Vec<Value>>,
        connected_url: Mutex<Option<String>>,
        close_code: Mutex<Option<u16>>,
    }

    impl ScriptedGatewayTransport {
        fn push(&self, payload: Value) {
            self.frames
                .lock()
                .expect("frames lock")
                .push_back(payload.to_string());
        }

        fn push_close(&self, code: u16) {
            *self.close_code.lock().expect("close code lock") = Some(code);
            self.frames
                .lock()
                .expect("frames lock")
                .push_back(CLOSED.to_owned());
        }
    }

    #[async_trait]
    impl GatewayTransport for ScriptedGatewayTransport {
        async fn connect(
            &mut self,
            url: &str,
            _headers: &[(String, String)],
        ) -> Result<(), TransportError> {
            *self.connected_url.lock().expect("connected url lock") = Some(url.to_owned());
            Ok(())
        }

        async fn receive(&mut self) -> Result<GatewayMessage, TransportError> {
            match self.frames.lock().expect("frames lock").pop_front() {
                Some(frame) if frame == CLOSED => Ok(GatewayMessage::Closed),
                Some(frame) => Ok(GatewayMessage::Text(frame)),
                None => Ok(GatewayMessage::Closed),
            }
        }

        async fn send(&mut self, text: &str) -> Result<(), TransportError> {
            let payload =
                Value::from_str(text).map_err(|error| TransportError(error.to_string()))?;
            self.sent.lock().expect("sent lock").push(payload);
            Ok(())
        }

        async fn close(&mut self, code: u16, _reason: &str) -> Result<(), TransportError> {
            *self.close_code.lock().expect("close code lock") = Some(code);
            Ok(())
        }

        fn close_code(&self) -> Option<u16> {
            *self.close_code.lock().expect("close code lock")
        }
    }

    struct StaticUrl;

    #[async_trait]
    impl GatewayUrlProvider for StaticUrl {
        async fn gateway_url(&self) -> Option<String> {
            Some("wss://gateway.discord.gg".to_owned())
        }
    }

    fn hello(interval: u64) -> Value {
        json!({"op": 10, "d": {"heartbeat_interval": interval}})
    }

    #[tokio::test]
    async fn identifies_and_records_ready_state() {
        let transport = ScriptedGatewayTransport::default();
        transport.push(hello(41_250));
        transport.push(json!({
            "op": 0,
            "s": 7,
            "t": "READY",
            "d": {
                "session_id": "session-1",
                "resume_gateway_url": "wss://resume.example",
                "user": {"id": "42", "username": "lamp"},
                "guilds": []
            }
        }));
        transport.push_close(1000);
        let mut client = GatewayClient::new(transport, StaticUrl);

        let events = client.run("token-1").await.expect("gateway run");

        assert!(
            !events
                .iter()
                .any(|event| matches!(event, GatewayEvent::HandlerErrors { .. }))
        );
        let sent = client.transport.sent.lock().expect("sent lock");
        assert_eq!(sent[0]["op"], OP_IDENTIFY);
        assert_eq!(sent[0]["d"]["token"], "token-1");
        assert!(events.contains(&GatewayEvent::Ready {
            session_id: "session-1".to_owned(),
            resume_gateway_url: Some("wss://resume.example".to_owned()),
            user_id: Some("42".to_owned()),
        }));
        assert_eq!(
            client.session().resume_info(),
            Some(("session-1", 7, "wss://resume.example"))
        );
    }

    #[tokio::test]
    async fn writes_queued_outbound_payloads_to_socket() {
        let transport = ScriptedGatewayTransport::default();
        transport.push(hello(41_250));
        transport.push_close(1000);
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        tx.send(lazy_request_payload(10, 11, &[[0, 99]]))
            .expect("queue");
        let mut client = GatewayClient::new(transport, StaticUrl).with_outbound(rx);

        client.run("token-1").await.expect("gateway run");

        let sent = client.transport.sent.lock().expect("sent lock");
        assert!(sent.iter().any(|payload| payload["op"] == OP_LAZY_REQUEST
            && payload["d"]["guild_id"] == "10"));
    }

    #[tokio::test]
    async fn dispatch_updates_state_cache() {
        let transport = ScriptedGatewayTransport::default();
        transport.push(hello(41_250));
        transport.push(json!({
            "op": 0,
            "s": 1,
            "t": "READY",
            "d": {
                "session_id": "session-cache",
                "resume_gateway_url": "wss://resume.example",
                "user": {"id": "42", "username": "lamp"},
                "guilds": [],
                "channels": []
            }
        }));
        transport.push(json!({
            "op": 0,
            "s": 2,
            "t": "MESSAGE_CREATE",
            "d": {
                "id": "100",
                "channel_id": "9",
                "author": {"id": "5", "username": "sender"},
                "content": "hello",
                "timestamp": "2026-01-01T00:00:00Z"
            }
        }));
        transport.push(json!({
            "op": 0,
            "s": 3,
            "t": "GUILD_CREATE",
            "d": {
                "id": "7",
                "name": "Guild",
                "channels": [{"id": "9", "type": 0, "name": "general"}]
            }
        }));
        transport.push_close(1000);
        let mut client = GatewayClient::new(transport, StaticUrl);

        let events = client.run("token-cache").await.expect("gateway run");

        assert!(
            !events
                .iter()
                .any(|event| matches!(event, GatewayEvent::HandlerErrors { .. }))
        );
        let cache = client.cache().lock().expect("cache lock");
        assert_eq!(cache.current_user().map(|user| user.id), Some(42));
        assert_eq!(cache.messages(9)[0].id, 100);
        assert_eq!(
            cache.guild(7).and_then(|guild| guild.name.as_deref()),
            Some("Guild")
        );
        assert_eq!(cache.channels_for_guild(7)[0].id, 9);
    }

    #[tokio::test]
    async fn resumes_existing_session_from_hello() {
        let transport = ScriptedGatewayTransport::default();
        transport.push(hello(41_250));
        transport.push(json!({"op": 0, "t": "RESUMED", "d": {}}));
        transport.push_close(1000);
        let mut client = GatewayClient::new(transport, StaticUrl);
        client.session.switch_token("token-1");
        client
            .session
            .record_ready("session-1", Some("wss://old.example".to_owned()));
        client.session.record_sequence(99);

        let events = client.run("token-1").await.expect("gateway run");

        let sent = client.transport.sent.lock().expect("sent lock");
        assert_eq!(sent[0]["op"], OP_RESUME);
        assert_eq!(sent[0]["d"]["session_id"], "session-1");
        assert_eq!(sent[0]["d"]["seq"], 99);
        assert!(events.contains(&GatewayEvent::Resumed));
        assert_eq!(
            client
                .transport
                .connected_url
                .lock()
                .expect("connected url lock")
                .as_deref(),
            Some("wss://old.example/?v=9&encoding=json")
        );
    }

    #[tokio::test]
    async fn auth_failed_close_emits_token() {
        let transport = ScriptedGatewayTransport::default();
        transport.push_close(4004);
        let mut client = GatewayClient::new(transport, StaticUrl);

        let events = client.run("token-1").await.expect("gateway run");
        assert_eq!(
            events,
            vec![GatewayEvent::AuthFailed {
                token: "token-1".to_owned()
            }]
        );
    }
}
