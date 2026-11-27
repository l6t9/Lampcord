use crate::utils::prefs;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::time;
use tokio::sync::Notify;

use serde_json::{Value, json};

use super::opcodes;

pub const QOS_VERSION: u32 = 31;

pub const CLIENT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15 * 60);

pub const SESSION_LIFETIME_MS: u64 = 30 * 60 * 1000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Qos {
    pub active: bool,
    pub reasons: Vec<String>,
}

impl Qos {
    pub fn for_foreground(foregrounded: bool) -> Self {
        let reasons: Vec<String> = if foregrounded {
            vec!["foregrounded".to_string()]
        } else {
            Vec::new()
        };
        Self {
            active: !reasons.is_empty(),
            reasons,
        }
    }

    fn to_json(&self) -> Value {
        json!({ "active": self.active, "ver": QOS_VERSION, "reasons": self.reasons })
    }
}

#[derive(Debug, Default)]
pub struct QosState {
    current: Option<Qos>,
    upcoming: Option<Qos>,
}

impl QosState {
    pub fn handle_active_state_change(&mut self, change: Qos) -> bool {
        let was_inactive = !self.current.as_ref().is_some_and(|c| c.active);
        let send_now = change.active && was_inactive;
        let current = self.current.get_or_insert_with(|| change.clone());
        if change.active {
            current.active = true;
            let mut reasons: Vec<String> = current
                .reasons
                .iter()
                .chain(change.reasons.iter())
                .cloned()
                .collect();
            reasons.sort();
            reasons.dedup();
            current.reasons = reasons;
        }
        self.upcoming = Some(change);
        send_now
    }

    pub fn consume(&mut self) -> Option<Qos> {
        let sent = self.current.clone();
        if let Some(next) = self.upcoming.take() {
            self.current = Some(next);
        }
        sent
    }
}

pub fn heartbeat(seq: Option<u64>, qos: Option<&Qos>) -> Value {
    json!({ "op": opcodes::QOS_HEARTBEAT, "d": { "seq": seq, "qos": qos.map(Qos::to_json) } })
}

static FOREGROUND: AtomicBool = AtomicBool::new(true);
static CHANGED: Notify = Notify::const_new();

pub fn set_foreground(foregrounded: bool) {
    if FOREGROUND.swap(foregrounded, Ordering::Relaxed) != foregrounded {
        CHANGED.notify_one();
    }
}

pub fn is_foreground() -> bool {
    FOREGROUND.load(Ordering::Relaxed)
}

pub async fn foreground_changed() {
    CHANGED.notified().await;
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClientSession {
    pub uuid: String,
    pub created_ms: u64,
    pub last_used_ms: u64,
}

impl ClientSession {
    fn expired(&self, now_ms: u64) -> bool {
        SESSION_LIFETIME_MS + self.last_used_ms <= now_ms
    }

    pub fn announcement(&self, client_launch_id: &str) -> Value {
        json!({ "op": opcodes::UPDATE_TIME_SPENT_SESSION_ID, "d": { "initialization_timestamp": self.created_ms, "session_id": self.uuid, "client_launch_id": client_launch_id } })
    }
}

#[derive(Debug, Default)]
pub struct ClientSessions {
    loaded: bool,
    session: Option<ClientSession>,
}

impl ClientSessions {
    pub fn touch(
        &mut self,
        stored: Option<ClientSession>,
        active: bool,
        now_ms: u64,
        new_uuid: impl FnOnce() -> String,
    ) -> (Option<ClientSession>, bool, bool) {
        let previous = if self.loaded {
            self.session.as_ref().map(|s| s.uuid.clone())
        } else {
            None
        };
        let mut session = if self.loaded {
            self.session.clone()
        } else {
            stored
        };
        let mut write = false;
        if active {
            if session.as_ref().is_none_or(|s| s.expired(now_ms)) {
                session = Some(ClientSession {
                    uuid: new_uuid(),
                    created_ms: now_ms,
                    last_used_ms: now_ms,
                });
            }
            if let Some(s) = session.as_mut() {
                s.last_used_ms = now_ms;
            }
            write = true;
        } else if session.as_ref().is_some_and(|s| s.expired(now_ms)) {
            session = None;
        }
        self.loaded = true;
        self.session = session.clone();
        let changed = session
            .as_ref()
            .is_some_and(|s| previous.as_deref() != Some(s.uuid.as_str()));
        (session, write, changed)
    }
}

static SESSIONS: Mutex<Option<ClientSessions>> = Mutex::new(None);
static LAST_WRITE: Mutex<Option<Instant>> = Mutex::new(None);

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn touch_session(active: bool) -> (Option<ClientSession>, bool) {
    let mut guard = SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
    let sessions = guard.get_or_insert_with(ClientSessions::default);
    let stored = if sessions.loaded {
        None
    } else {
        prefs::load().client_heartbeat_session
    };
    let (session, write, changed) = sessions.touch(stored, active, now_ms(), || {
        uuid::Uuid::new_v4().to_string()
    });
    let due = LAST_WRITE
        .lock()
        .map(|last| last.is_none_or(|t| t.elapsed() >= Duration::from_secs(1)))
        .unwrap_or(true);
    if write && due {
        let mut prefs = prefs::load();
        prefs.client_heartbeat_session = session.clone();
        prefs::save(&prefs);
        if let Ok(mut last) = LAST_WRITE.lock() {
            *last = Some(Instant::now());
        }
    }
    (session, changed)
}

pub fn session_id() -> String {
    touch_session(true)
        .0
        .map(|s| s.uuid)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fg(on: bool) -> Qos {
        Qos::for_foreground(on)
    }

    #[test]
    fn the_idle_heartbeat_is_what_the_real_client_sent() {
        let mut state = QosState::default();
        state.handle_active_state_change(fg(false));
        assert_eq!(
            heartbeat(Some(14), state.consume().as_ref()).to_string(),
            r#"{"op":40,"d":{"seq":14,"qos":{"active":false,"ver":31,"reasons":[]}}}"#
        );
    }

    #[test]
    fn foregrounding_heartbeats_at_once_with_the_reason() {
        let mut state = QosState::default();
        assert!(!state.handle_active_state_change(fg(false)));
        state.consume();
        assert!(
            state.handle_active_state_change(fg(true)),
            "inactive -> active sends now"
        );
        assert_eq!(
            heartbeat(Some(23), state.consume().as_ref()).to_string(),
            r#"{"op":40,"d":{"seq":23,"qos":{"active":true,"ver":31,"reasons":["foregrounded"]}}}"#
        );
        assert!(
            !state.handle_active_state_change(fg(true)),
            "already active: no extra heartbeat"
        );
    }

    #[test]
    fn a_blur_between_heartbeats_is_still_reported_active_once() {
        let mut state = QosState::default();
        state.handle_active_state_change(fg(false));
        state.consume();
        state.handle_active_state_change(fg(true));
        state.handle_active_state_change(fg(false));
        let first = state.consume().unwrap();
        assert!(
            first.active,
            "the beat after a focus blip still says active"
        );
        assert!(
            !state.consume().unwrap().active,
            "then the state catches up"
        );
    }

    #[test]
    fn before_any_state_the_qos_is_null() {
        assert_eq!(
            heartbeat(None, None).to_string(),
            r#"{"op":40,"d":{"seq":null,"qos":null}}"#
        );
    }

    #[test]
    fn a_session_lives_thirty_minutes_after_its_last_use() {
        let mut sessions = ClientSessions::default();
        let (first, write, changed) = sessions.touch(None, true, 1_000, || "a".into());
        assert_eq!(
            (first.as_ref().map(|s| s.uuid.as_str()), write, changed),
            (Some("a"), true, true),
            "the first use announces a new session"
        );
        let (same, _, changed) = sessions.touch(None, true, 1_000 + 10 * 60 * 1000, || "b".into());
        assert_eq!((same.unwrap().uuid, changed), ("a".to_string(), false));
        let (renewed, _, changed) = sessions.touch(
            None,
            true,
            1_000 + 10 * 60 * 1000 + SESSION_LIFETIME_MS,
            || "b".into(),
        );
        assert_eq!(
            (renewed.unwrap().uuid, changed),
            ("b".to_string(), true),
            "expired: a new session"
        );
    }

    #[test]
    fn a_stored_session_from_the_last_launch_is_reused_if_young_and_dropped_if_not() {
        let stored = ClientSession {
            uuid: "old".into(),
            created_ms: 0,
            last_used_ms: 5_000,
        };
        let mut young = ClientSessions::default();
        let (session, _, _) = young.touch(Some(stored.clone()), false, 6_000, || "new".into());
        assert_eq!(
            session.map(|s| s.uuid),
            Some("old".to_string()),
            "backgrounded: it keeps what it had"
        );
        let mut stale = ClientSessions::default();
        let (session, _, _) = stale.touch(Some(stored), false, 5_000 + SESSION_LIFETIME_MS, || {
            "new".into()
        });
        assert_eq!(session, None, "backgrounded and expired: none");
    }

    #[test]
    fn the_announcement_has_the_real_keys() {
        let session = ClientSession {
            uuid: "u".into(),
            created_ms: 7,
            last_used_ms: 9,
        };
        assert_eq!(
            session.announcement("launch").to_string(),
            r#"{"op":41,"d":{"initialization_timestamp":7,"session_id":"u","client_launch_id":"launch"}}"#
        );
    }
}
