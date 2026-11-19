#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GatewaySession {
    token: Option<String>,
    session_id: Option<String>,
    last_sequence: Option<u32>,
    resume_gateway_url: Option<String>,
}

impl GatewaySession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn switch_token(&mut self, token: &str) {
        if self.token.as_deref() != Some(token) {
            self.session_id = None;
            self.last_sequence = None;
            self.resume_gateway_url = None;
            self.token = Some(token.to_owned());
        }
    }

    pub fn record_ready(
        &mut self,
        session_id: impl Into<String>,
        resume_gateway_url: Option<String>,
    ) {
        self.session_id = Some(session_id.into());
        self.resume_gateway_url = resume_gateway_url;
    }

    pub fn record_sequence(&mut self, sequence: u32) {
        self.last_sequence = Some(sequence);
    }

    pub fn record_resume_url(&mut self, url: impl Into<String>) {
        self.resume_gateway_url = Some(url.into());
    }

    pub fn resume_info(&self) -> Option<(&str, u32, &str)> {
        let session_id = self.session_id.as_deref()?;
        let sequence = self.last_sequence?;
        let url = self.resume_gateway_url.as_deref()?;
        Some((session_id, sequence, url))
    }

    pub fn clear_resume(&mut self) {
        self.session_id = None;
        self.last_sequence = None;
        self.resume_gateway_url = None;
    }

    pub fn clear(&mut self) {
        self.token = None;
        self.clear_resume();
    }
}

#[cfg(test)]
mod tests {
    use super::GatewaySession;

    #[test]
    fn switching_token_discards_resume_state() {
        let mut session = GatewaySession::new();
        session.switch_token("old");
        session.record_ready("session-1", Some("wss://old".to_owned()));
        session.record_sequence(99);
        assert_eq!(session.resume_info(), Some(("session-1", 99, "wss://old")));

        session.switch_token("new");
        assert_eq!(session.resume_info(), None);
    }

    #[test]
    fn same_token_keeps_resume_state() {
        let mut session = GatewaySession::new();
        session.switch_token("same");
        session.record_ready("session-1", Some("wss://same".to_owned()));
        session.record_sequence(7);

        session.switch_token("same");
        assert_eq!(session.resume_info(), Some(("session-1", 7, "wss://same")));
    }
}
