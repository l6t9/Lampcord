use crate::gateway::opcodes;
use crate::gateway;
use crate::utils::time;
use std::collections::VecDeque;
use std::sync::Mutex;

use serde_json::{Value, json};

use crate::presence::{CustomStatus, OwnStatus};

pub const IDLE_DURATION_MS: u64 = 10 * 60 * 1000;

const RATE_LIMIT_COUNT: usize = 5;
const RATE_LIMIT_WINDOW_MS: u64 = 20_000;

#[derive(Clone, Debug)]
pub struct IdleState {
    idle_since: u64,
    idle: bool,
    afk: bool,
    system_suspended: bool,
    system_locked: bool,
}

impl IdleState {
    pub fn new(now: u64) -> Self {
        Self {
            idle_since: now,
            idle: false,
            afk: false,
            system_suspended: false,
            system_locked: false,
        }
    }

    fn system_idle(&self) -> bool {
        self.system_suspended || self.system_locked
    }

    fn check(&mut self, now: u64) {
        let quiet = now.saturating_sub(self.idle_since);
        self.idle = quiet > IDLE_DURATION_MS || self.system_idle();

        self.afk = quiet > IDLE_DURATION_MS || self.system_idle();
    }

    pub fn note_activity(&mut self, now: u64) {
        self.idle_since = now;
        self.check(now);
    }

    pub fn note_system_idle(&mut self, now: u64, idle_ms: u64) {
        self.idle_since = self.idle_since.max(now.saturating_sub(idle_ms));
        self.check(now);
    }

    pub fn set_system_suspended(&mut self, suspended: bool, now: u64) {
        self.system_suspended = suspended;
        self.note_activity(now);
    }

    pub fn set_system_locked(&mut self, locked: bool, now: u64) {
        self.system_locked = locked;
        self.note_activity(now);
    }

    pub fn tick(&mut self, now: u64) {
        self.check(now);
    }

    pub fn since(&self) -> u64 {
        if self.idle { self.idle_since } else { 0 }
    }

    pub fn afk(&self) -> bool {
        self.afk
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Presence {
    pub status: &'static str,
    pub since: u64,
    pub activities: Vec<Value>,
    pub afk: bool,
}

impl Presence {
    fn to_payload(&self) -> Value {
        json!({ "op": opcodes::PRESENCE_UPDATE, "d": { "status": self.status, "since": self.since, "activities": self.activities, "afk": self.afk } })
    }

    fn initial() -> Self {
        Self {
            status: "online",
            since: 0,
            activities: Vec::new(),
            afk: false,
        }
    }
}

pub struct LocalPresence {
    status: OwnStatus,
    custom: Option<CustomStatus>,
    idle: IdleState,
    last_sent: Presence,
    sent_at: VecDeque<u64>,
}

impl LocalPresence {
    pub fn new(now: u64) -> Self {
        Self {
            status: OwnStatus::Online,
            custom: None,
            idle: IdleState::new(now),
            last_sent: Presence::initial(),
            sent_at: VecDeque::new(),
        }
    }

    pub fn current(&self, now: u64) -> Presence {
        let since = self.idle.since();
        let status = if self.status == OwnStatus::Online && since > 0 {
            OwnStatus::Idle
        } else {
            self.status
        };

        let activities: Vec<Value> = if status == OwnStatus::Invisible {
            Vec::new()
        } else {
            self.custom
                .iter()
                .filter(|c| !c.is_empty() && !c.is_expired(now))
                .map(CustomStatus::to_activity)
                .collect()
        };
        Presence {
            status: status.as_str(),
            since,
            activities,
            afk: self.idle.afk(),
        }
    }

    pub fn emit_if_changed(&mut self, now: u64) -> Option<Value> {
        let current = self.current(now);
        if current == self.last_sent {
            return None;
        }
        while self
            .sent_at
            .front()
            .is_some_and(|t| now.saturating_sub(*t) >= RATE_LIMIT_WINDOW_MS)
        {
            self.sent_at.pop_front();
        }
        if self.sent_at.len() >= RATE_LIMIT_COUNT {
            return None;
        }
        self.sent_at.push_back(now);
        let payload = current.to_payload();
        self.last_sent = current;
        Some(payload)
    }

    pub fn connection_open(
        &mut self,
        status: OwnStatus,
        custom: Option<CustomStatus>,
        now: u64,
    ) -> Option<Value> {
        self.status = status;
        self.custom = custom;
        self.last_sent = Presence::initial();
        self.sent_at.clear();
        self.emit_if_changed(now)
    }

    pub fn set_status(&mut self, status: OwnStatus, now: u64) -> Option<Value> {
        self.status = status;
        self.emit_if_changed(now)
    }

    pub fn set_custom(&mut self, custom: Option<CustomStatus>, now: u64) -> Option<Value> {
        self.custom = custom;
        self.emit_if_changed(now)
    }

    pub fn note_activity(&mut self, now: u64) -> Option<Value> {
        self.idle.note_activity(now);
        self.emit_if_changed(now)
    }

    pub fn note_system_idle(&mut self, now: u64, idle_ms: u64) -> Option<Value> {
        self.idle.note_system_idle(now, idle_ms);
        self.emit_if_changed(now)
    }

    pub fn tick(&mut self, now: u64) -> Option<Value> {
        self.idle.tick(now);
        self.emit_if_changed(now)
    }
}

static STATE: Mutex<Option<LocalPresence>> = Mutex::new(None);

fn with(f: impl FnOnce(&mut LocalPresence, u64) -> Option<Value>) {
    let now = time::now_ms();
    let payload = {
        let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
        f(guard.get_or_insert_with(|| LocalPresence::new(now)), now)
    };
    if let Some(payload) = payload {
        gateway::send_frames(vec![payload]);
    }
}

pub fn connection_open(status: OwnStatus, custom: Option<CustomStatus>) {
    with(|p, now| p.connection_open(status, custom, now));
}

pub fn set_status(status: OwnStatus) {
    with(|p, now| p.set_status(status, now));
}

pub fn set(status: OwnStatus, custom: Option<CustomStatus>) {
    with(|p, now| {
        p.status = status;
        p.custom = custom;
        p.emit_if_changed(now)
    });
}

pub fn set_custom_status(custom: Option<CustomStatus>) {
    with(|p, now| p.set_custom(custom, now));
}

pub fn note_activity() {
    with(|p, now| p.note_activity(now));
}

pub fn note_system_idle(idle_ms: u64) {
    with(|p, now| p.note_system_idle(now, idle_ms));
}

pub fn tick() {
    with(|p, now| p.tick(now));
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN: u64 = 60_000;

    #[test]
    fn a_default_account_sends_nothing_on_connect() {
        let mut p = LocalPresence::new(0);
        assert_eq!(p.connection_open(OwnStatus::Online, None, 1), None);
    }

    #[test]
    fn a_stored_dnd_status_is_sent_on_connect() {
        let mut p = LocalPresence::new(0);
        let payload = p.connection_open(OwnStatus::Dnd, None, 1).unwrap();
        assert_eq!(
            payload.to_string(),
            r#"{"op":3,"d":{"status":"dnd","since":0,"activities":[],"afk":false}}"#
        );
    }

    #[test]
    fn ten_quiet_minutes_make_an_online_account_idle_and_afk() {
        let mut p = LocalPresence::new(5_000);
        p.connection_open(OwnStatus::Online, None, 5_001);
        assert_eq!(
            p.tick(5_000 + 10 * MIN),
            None,
            "exactly ten minutes is not yet more than ten"
        );
        let payload = p.tick(5_000 + 10 * MIN + 1).unwrap();
        assert_eq!(
            payload.to_string(),
            r#"{"op":3,"d":{"status":"idle","since":5000,"activities":[],"afk":true}}"#
        );
    }

    #[test]
    fn input_ends_idleness_and_says_so_at_once() {
        let mut p = LocalPresence::new(1_000);
        p.connection_open(OwnStatus::Online, None, 1_001);
        p.tick(1_000 + 11 * MIN);

        let back = p.note_activity(1_000 + 12 * MIN).unwrap();
        assert_eq!(
            back.to_string(),
            r#"{"op":3,"d":{"status":"online","since":0,"activities":[],"afk":false}}"#
        );
        assert_eq!(
            p.note_activity(1_000 + 12 * MIN + 5),
            None,
            "nothing changed"
        );
    }

    #[test]
    fn dnd_stays_dnd_while_idle_but_the_clock_still_reports() {
        let mut p = LocalPresence::new(0);
        p.connection_open(OwnStatus::Dnd, None, 1);
        let payload = p.tick(11 * MIN).unwrap();
        assert_eq!(
            (
                payload["d"]["status"].as_str(),
                payload["d"]["afk"].as_bool(),
                payload["d"]["since"].as_u64()
            ),
            (Some("dnd"), Some(true), Some(0))
        );
    }

    #[test]
    fn the_os_idle_time_moves_the_start_of_the_quiet_time_forward_only() {
        let mut p = LocalPresence::new(0);
        p.connection_open(OwnStatus::Online, None, 1);

        assert_eq!(p.note_system_idle(5 * MIN, 2 * MIN), None);

        assert_eq!(p.tick(13 * MIN), None);
        let payload = p.tick(13 * MIN + 1).unwrap();
        assert_eq!(payload["d"]["since"].as_u64(), Some(3 * MIN));

        p.note_system_idle(14 * MIN, 12 * MIN);
        assert_eq!(p.current(14 * MIN).since, 3 * MIN);
    }

    #[test]
    fn no_more_than_five_updates_in_twenty_seconds() {
        let mut p = LocalPresence::new(0);
        p.connection_open(OwnStatus::Online, None, 1);
        let mut sent = 0;
        for i in 0..8u64 {
            let status = if i % 2 == 0 {
                OwnStatus::Dnd
            } else {
                OwnStatus::Online
            };
            sent += usize::from(p.set_status(status, 100 + i).is_some());
        }
        assert_eq!(sent, 5);

        assert!(
            p.set_status(OwnStatus::Online, 100 + RATE_LIMIT_WINDOW_MS + 100)
                .is_some(),
            "the window moves on"
        );
    }

    #[test]
    fn a_custom_status_is_the_activity_and_expires() {
        let mut p = LocalPresence::new(0);
        let custom = CustomStatus {
            text: "busy".into(),
            emoji: None,
            expires_at_ms: Some(5_000),
        };
        let first = p
            .connection_open(OwnStatus::Online, Some(custom), 1_000)
            .unwrap();
        assert_eq!(first["d"]["activities"][0]["state"], "busy");
        let expired = p.tick(6_000).unwrap();
        assert_eq!(expired["d"]["activities"], json!([]));
    }
}
