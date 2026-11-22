pub use crate::rest::{API_BASE, DESKTOP_USER_AGENT};

#[derive(Debug, Clone)]
pub struct Session {
    token: String,
    api_base: String,
    heartbeat_session_id: String,
}

impl Session {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            api_base: API_BASE.to_owned(),
            heartbeat_session_id: new_uuid(),
        }
    }

    pub fn with_api_base(mut self, api_base: impl Into<String>) -> Self {
        self.api_base = api_base.into();
        self
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn api_base(&self) -> &str {
        &self.api_base
    }

    pub fn authorization(&self) -> &str {
        &self.token
    }

    pub fn heartbeat_session_id(&self) -> &str {
        &self.heartbeat_session_id
    }

    pub fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.api_base.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    pub fn rotate_token(&mut self, token: impl Into<String>) {
        self.token = token.into();
    }
}

fn new_uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("{:016x}", nanos)
}
