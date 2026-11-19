use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

use crate::{
    event_dispatch::Dispatcher,
    gateway_plan::{
        OP_DISPATCH, OP_HEARTBEAT, OP_IDENTIFY, OP_RESUME, control_action, gateway_url,
        resume_decision,
    },
    gateway_session::GatewaySession,
    rest::DESKTOP_USER_AGENT,
    state_cache::{StateCache, StateCacheHandler},
    traits::{GatewayMessage, GatewayTransport, GatewayUrlProvider, TransportError},
};

pub const DEFAULT_GATEWAY_URL: &str = "wss://gateway.discord.gg";

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
}

impl Default for GatewayIdentity {
    fn default() -> Self {
        Self {
            properties: json!({
                "os": "Linux",
                "browser": "Discord Desktop",
                "device": "Lampcord",
                "system_locale": "en-US",
                "has_client_mods": false,
                "client_version": "0.0.309",
                "release_channel": "stable",
                "design_id": 0,
                "os_version": "",
                "client_build_number": 309000,
                "client_event_source": null,
            }),
            client_state: json!({
                "guild_hashes": {},
                "highest_last_message_id": "0",
                "read_state_version": 0,
                "user_guild_settings_version": -1,
            }),
            capabilities: 351,
            large_threshold: 100,
            compress: false,
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
        }
    }

    pub fn with_identity(mut self, identity: GatewayIdentity) -> Self {
        self.identity = identity;
        self
    }

    pub fn session(&self) -> &GatewaySession {
        &self.session
    }

    pub fn cache(&self) -> &Arc<Mutex<StateCache>> {
        &self.cache
    }

    pub async fn run(&mut self, token: &str) -> Result<Vec<GatewayEvent>, GatewayError> {
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

        let mut events = Vec::new();
        loop {
            match self.transport.receive().await? {
                GatewayMessage::Text(text) => {
                    let payload: Value = serde_json::from_str(&text)?;
                    let op = payload
                        .get("op")
                        .and_then(Value::as_i64)
                        .unwrap_or_default();

                    if let Some(sequence) = payload.get("s").and_then(Value::as_u64) {
                        self.session.record_sequence(sequence as u32);
                    }

                    let action =
                        control_action(op, payload.get("d"), self.transport.close_code(), token);

                    match action {
                        crate::gateway_plan::ControlAction::Hello {
                            heartbeat_interval_ms,
                        } => {
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
                            events.push(GatewayEvent::Hello {
                                heartbeat_interval_ms,
                                mode,
                            });
                        }
                        crate::gateway_plan::ControlAction::Heartbeat => {
                            let payload = heartbeat_payload(self.session.last_sequence());
                            self.transport.send(&payload.to_string()).await?;
                            events.push(GatewayEvent::HeartbeatSent);
                        }
                        crate::gateway_plan::ControlAction::HeartbeatAck => {
                            events.push(GatewayEvent::HeartbeatAck);
                        }
                        crate::gateway_plan::ControlAction::InvalidSession { resumable } => {
                            if !resumable {
                                self.session.clear_resume();
                            }
                            events.push(GatewayEvent::InvalidSession { resumable });
                        }
                        crate::gateway_plan::ControlAction::Reconnect => {
                            self.transport
                                .close(1000, "server requested reconnect")
                                .await?;
                            events.push(GatewayEvent::ReconnectRequested);
                            break;
                        }
                        crate::gateway_plan::ControlAction::AuthFailed { token } => {
                            events.push(GatewayEvent::AuthFailed { token });
                            break;
                        }
                        crate::gateway_plan::ControlAction::None => {
                            if op == OP_DISPATCH {
                                let event_type =
                                    payload.get("t").and_then(Value::as_str).unwrap_or_default();
                                let errors =
                                    self.dispatcher.dispatch(Some(event_type), payload.get("d"));
                                if !errors.is_empty() {
                                    events.push(GatewayEvent::HandlerErrors { errors });
                                }
                                match event_type {
                                    "READY" => {
                                        let data = payload.get("d");
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
                                        events.push(GatewayEvent::Ready {
                                            session_id,
                                            resume_gateway_url,
                                            user_id,
                                        });
                                    }
                                    "RESUMED" => events.push(GatewayEvent::Resumed),
                                    _ => events.push(GatewayEvent::Dispatch {
                                        event_type: event_type.to_owned(),
                                    }),
                                }
                            }
                        }
                    }
                }
                GatewayMessage::Closed => {
                    if self
                        .transport
                        .close_code()
                        .is_some_and(crate::gateway_plan::is_auth_failed_close)
                    {
                        events.push(GatewayEvent::AuthFailed {
                            token: token.to_owned(),
                        });
                    } else {
                        events.push(GatewayEvent::Closed {
                            code: self.transport.close_code(),
                        });
                    }
                    break;
                }
            }
        }

        Ok(events)
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
