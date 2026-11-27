use serde_json::{Value, json};
use std::mem;
use std::sync::RwLock;

pub fn close_code_ends_session(code: u16) -> bool {
    matches!(code, 4003 | 4004 | 4005 | 4007 | 4009 | 4010..=4014)
}

static CURRENT_ID: RwLock<Option<String>> = RwLock::new(None);

pub fn current_id() -> Option<String> {
    CURRENT_ID.read().ok().and_then(|id| id.clone())
}

fn publish_id(id: Option<&str>) {
    if let Ok(mut slot) = CURRENT_ID.write() {
        *slot = id.map(str::to_owned);
    }
}

#[derive(Debug, Default, Clone)]
pub struct Session {
    established: bool,
    id: Option<String>,
    resume_url: Option<String>,

    seq: Option<u64>,
}

impl Session {
    pub fn set_established(&mut self, established: bool) {
        self.established = established;
    }

    pub fn take_established(&mut self) -> bool {
        mem::take(&mut self.established)
    }

    pub fn can_resume(&self) -> bool {
        self.id.is_some() && self.seq.is_some()
    }

    pub fn seq(&self) -> Option<u64> {
        self.seq
    }

    pub fn clear(&mut self) {
        publish_id(None);
        *self = Self::default();
    }

    pub fn observe_seq(&mut self, seq: u64) {
        self.seq = Some(seq);
    }

    pub fn on_ready_raw(&mut self, ready: &str) {
        #[derive(serde::Deserialize)]
        struct MinimalReady {
            session_id: Option<String>,
            resume_gateway_url: Option<String>,
        }
        let ready: MinimalReady = serde_json::from_str(ready).unwrap_or(MinimalReady {
            session_id: None,
            resume_gateway_url: None,
        });
        publish_id(ready.session_id.as_deref());
        self.id = ready.session_id;
        self.resume_url = ready.resume_gateway_url.filter(|url| !url.is_empty());
    }

    pub fn urls(&self, default_url: &str) -> Vec<String> {
        let mut urls = Vec::with_capacity(2);
        if self.can_resume() {
            if let Some(resume) = &self.resume_url {
                let query = default_url.split_once('?').map(|(_, query)| query);
                let url = match (resume.contains('?'), query) {
                    (false, Some(query)) => format!("{}/?{query}", resume.trim_end_matches('/')),
                    _ => resume.clone(),
                };
                if url != default_url {
                    urls.push(url);
                }
            }
        }
        urls.push(default_url.to_string());
        urls
    }

    pub fn resume_payload(&self, token: &str) -> Option<Value> {
        let (id, seq) = (self.id.as_ref()?, self.seq?);
        Some(
            json!({ "op": super::opcodes::RESUME, "d": { "token": token, "session_id": id, "seq": seq } }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready() -> Value {
        json!({ "session_id": "abc", "resume_gateway_url": "wss://resume.example" })
    }

    #[test]
    fn a_session_can_resume_once_it_has_an_id_and_a_sequence() {
        let mut session = Session::default();
        assert!(!session.can_resume());
        session.on_ready_raw(&ready().to_string());
        assert!(
            !session.can_resume(),
            "READY itself carries a sequence, but until one was seen there is nothing to resume from"
        );
        session.observe_seq(1);
        assert!(session.can_resume());
        session.clear();
        assert!(!session.can_resume());
        assert_eq!(session.seq(), None);
    }

    #[test]
    fn the_resume_payload_is_op_6_with_token_session_and_seq() {
        let mut session = Session::default();
        assert!(session.resume_payload("t").is_none());
        session.on_ready_raw(&ready().to_string());
        session.observe_seq(41);
        session.observe_seq(42);
        assert_eq!(
            session.resume_payload("tok").unwrap(),
            json!({ "op": 6, "d": { "token": "tok", "session_id": "abc", "seq": 42 } })
        );
    }

    #[test]
    fn a_resume_url_gets_the_query_of_the_configured_one_and_the_configured_one_is_the_fallback() {
        let mut session = Session::default();
        let default = "wss://gateway.example/?v=9&encoding=json";
        assert_eq!(
            session.urls(default),
            [default],
            "no session: just the configured URL"
        );
        session.on_ready_raw(&ready().to_string());
        session.observe_seq(5);
        assert_eq!(
            session.urls(default),
            ["wss://resume.example/?v=9&encoding=json", default]
        );

        session.on_ready_raw(
            &json!({ "session_id": "abc", "resume_gateway_url": default }).to_string(),
        );
        assert_eq!(session.urls(default), [default]);
    }

    #[test]
    fn only_codes_that_end_the_session_stop_a_resume() {
        for code in [4003, 4004, 4005, 4007, 4009, 4010, 4014] {
            assert!(close_code_ends_session(code), "{code}");
        }

        for code in [1000, 1001, 1006, 4000, 4001, 4002, 4008] {
            assert!(!close_code_ends_session(code), "{code}");
        }
    }
}
