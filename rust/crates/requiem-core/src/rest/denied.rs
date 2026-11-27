use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const REMEMBER_FOR: Duration = Duration::from_secs(10 * 60);

const MAX_ENTRIES: usize = 512;

#[derive(Clone)]
pub(super) struct Refusal {
    pub status: wreq::StatusCode,
    pub body: String,
}

#[derive(Clone, Default)]
pub(super) struct Denied {
    entries: Arc<Mutex<HashMap<String, (Refusal, Instant)>>>,
}

fn permanent(status: wreq::StatusCode) -> bool {
    matches!(status.as_u16(), 400 | 403 | 404 | 410)
}

impl Denied {
    fn key(method: &wreq::Method, path: &str) -> Option<String> {
        (method == wreq::Method::GET).then(|| path.to_string())
    }

    pub fn note(&self, method: &wreq::Method, path: &str, status: wreq::StatusCode, body: &str) {
        let (Some(key), true) = (Self::key(method, path), permanent(status)) else {
            return;
        };
        let mut entries = self.entries.lock().expect("denied set poisoned");
        if entries.len() >= MAX_ENTRIES {
            if let Some(oldest) = entries
                .iter()
                .min_by_key(|(_, (_, at))| *at)
                .map(|(k, _)| k.clone())
            {
                entries.remove(&oldest);
            }
        }
        entries.insert(
            key,
            (
                Refusal {
                    status,
                    body: body.to_string(),
                },
                Instant::now(),
            ),
        );
    }

    pub fn check(&self, method: &wreq::Method, path: &str) -> Option<Refusal> {
        let key = Self::key(method, path)?;
        let mut entries = self.entries.lock().expect("denied set poisoned");
        match entries.get(&key) {
            Some((refusal, at)) if at.elapsed() < REMEMBER_FOR => Some(refusal.clone()),
            Some(_) => {
                entries.remove(&key);
                None
            }
            None => None,
        }
    }

    pub fn forget(&self, needle: &str) {
        self.entries
            .lock()
            .expect("denied set poisoned")
            .retain(|path, _| !path.contains(needle));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_refused_read_is_answered_from_memory_until_asked_for_again() {
        let denied = Denied::default();
        let path = "/channels/42/messages?limit=10";
        assert!(denied.check(&wreq::Method::GET, path).is_none());
        denied.note(
            &wreq::Method::GET,
            path,
            wreq::StatusCode::FORBIDDEN,
            "{\"code\": 50001}",
        );
        let refusal = denied.check(&wreq::Method::GET, path).expect("remembered");
        assert_eq!(refusal.status, wreq::StatusCode::FORBIDDEN);
        assert!(
            denied
                .check(&wreq::Method::GET, "/channels/43/messages?limit=10")
                .is_none(),
            "other requests are unaffected"
        );
        denied.forget("/channels/42/");
        assert!(
            denied.check(&wreq::Method::GET, path).is_none(),
            "an explicit retry goes to the server"
        );
    }

    #[test]
    fn only_permanent_refusals_of_reads_are_remembered() {
        let denied = Denied::default();
        denied.note(
            &wreq::Method::GET,
            "/a",
            wreq::StatusCode::INTERNAL_SERVER_ERROR,
            "",
        );
        denied.note(
            &wreq::Method::GET,
            "/b",
            wreq::StatusCode::TOO_MANY_REQUESTS,
            "",
        );
        denied.note(&wreq::Method::POST, "/c", wreq::StatusCode::FORBIDDEN, "");
        for path in ["/a", "/b"] {
            assert!(denied.check(&wreq::Method::GET, path).is_none());
        }
        assert!(denied.check(&wreq::Method::POST, "/c").is_none());
    }
}
