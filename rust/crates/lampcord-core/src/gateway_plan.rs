use serde_json::Value;

pub const OP_DISPATCH: i64 = 0;
pub const OP_HEARTBEAT: i64 = 1;
pub const OP_IDENTIFY: i64 = 2;
pub const OP_RESUME: i64 = 6;
pub const OP_RECONNECT: i64 = 7;
pub const OP_INVALID_SESSION: i64 = 9;
pub const OP_HELLO: i64 = 10;
pub const OP_HEARTBEAT_ACK: i64 = 11;
pub const OP_HEARTBEAT_DEFAULT_MS: u64 = 41_250;
pub const HELLO_TIMEOUT_MS: u64 = 20_000;
pub const INVALID_SESSION_DELAY_MS: u64 = 1_000;
pub const MAX_RECONNECT_DELAY_MS: u64 = 30_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlAction {
    Hello { heartbeat_interval_ms: u64 },
    Heartbeat,
    Reconnect,
    InvalidSession { resumable: bool },
    HeartbeatAck,
    AuthFailed { token: String },
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeDecision<'a> {
    Resume { session_id: &'a str, sequence: u32 },
    Identify,
}

pub fn heartbeat_interval_ms(payload: &Value) -> u64 {
    payload
        .get("d")
        .and_then(|data| data.get("heartbeat_interval"))
        .and_then(|value| {
            value
                .as_u64()
                .or_else(|| value.as_f64().map(|value| value as u64))
        })
        .unwrap_or(OP_HEARTBEAT_DEFAULT_MS)
}

pub fn gateway_url(base: &str, resume_url: Option<&str>) -> String {
    let base = resume_url.filter(|url| !url.is_empty()).unwrap_or(base);
    format!("{}/?v=9&encoding=json", base.trim_end_matches('/'))
}

pub fn control_action(
    op: i64,
    data: Option<&Value>,
    close_code: Option<u16>,
    token: &str,
) -> ControlAction {
    if close_code.is_some_and(is_auth_failed_close) {
        return ControlAction::AuthFailed {
            token: token.to_owned(),
        };
    }

    match op {
        OP_HELLO => ControlAction::Hello {
            heartbeat_interval_ms: data
                .map(heartbeat_interval_ms)
                .unwrap_or(OP_HEARTBEAT_DEFAULT_MS),
        },
        OP_HEARTBEAT => ControlAction::Heartbeat,
        OP_RECONNECT => ControlAction::Reconnect,
        OP_INVALID_SESSION => ControlAction::InvalidSession {
            resumable: data.and_then(Value::as_bool).unwrap_or(false),
        },
        OP_HEARTBEAT_ACK => ControlAction::HeartbeatAck,
        _ => ControlAction::None,
    }
}

pub fn resume_decision<'a>(resume_info: Option<(&'a str, u32, &'a str)>) -> ResumeDecision<'a> {
    match resume_info {
        Some((session_id, sequence, _)) => ResumeDecision::Resume {
            session_id,
            sequence,
        },
        None => ResumeDecision::Identify,
    }
}

pub fn reconnect_delay_ms(attempt: u32) -> u64 {
    if attempt == 0 {
        return 1_000;
    }
    let shift = (attempt - 1).min(5);
    (1_000u64 << shift).min(MAX_RECONNECT_DELAY_MS)
}

pub fn is_auth_failed_close(code: u16) -> bool {
    code == 4003 || code == 4004
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hello_interval_as_integer_or_float() {
        let integer = serde_json::json!({ "d": { "heartbeat_interval": 41250 } });
        let float = serde_json::json!({ "d": { "heartbeat_interval": 41250.0 } });
        assert_eq!(heartbeat_interval_ms(&integer), 41_250);
        assert_eq!(heartbeat_interval_ms(&float), 41_250);
        assert_eq!(heartbeat_interval_ms(&serde_json::json!({})), 41_250);
    }

    #[test]
    fn prefers_resume_url_and_normalizes_suffix() {
        assert_eq!(
            gateway_url("wss://gateway.discord.gg/", None),
            "wss://gateway.discord.gg/?v=9&encoding=json"
        );
        assert_eq!(
            gateway_url("wss://gateway.discord.gg", Some("wss://resume.example/")),
            "wss://resume.example/?v=9&encoding=json"
        );
    }

    #[test]
    fn close_codes_4003_and_4004_become_auth_failed() {
        assert_eq!(
            control_action(OP_DISPATCH, None, Some(4004), "token-1"),
            ControlAction::AuthFailed {
                token: "token-1".to_owned()
            }
        );
        assert_eq!(
            control_action(
                OP_HELLO,
                Some(&serde_json::json!({})),
                Some(1000),
                "token-1"
            ),
            ControlAction::Hello {
                heartbeat_interval_ms: OP_HEARTBEAT_DEFAULT_MS
            }
        );
    }

    #[test]
    fn resume_decision_requires_session_and_sequence() {
        assert_eq!(
            resume_decision(Some(("session-1", 7, "wss://resume"))),
            ResumeDecision::Resume {
                session_id: "session-1",
                sequence: 7
            }
        );
        assert_eq!(resume_decision(None), ResumeDecision::Identify);
    }

    #[test]
    fn reconnect_delay_is_capped() {
        assert_eq!(reconnect_delay_ms(0), 1_000);
        assert_eq!(reconnect_delay_ms(1), 1_000);
        assert_eq!(reconnect_delay_ms(5), 16_000);
        assert_eq!(reconnect_delay_ms(20), 30_000);
    }
}
