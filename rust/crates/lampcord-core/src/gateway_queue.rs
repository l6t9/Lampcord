use std::collections::{BTreeMap, VecDeque};

use serde_json::Value;

pub const SEND_WINDOW_MS: i64 = 60_000;
pub const SEND_LIMIT: usize = 110;
pub const MAX_QUEUE_SIZE: usize = 500;
pub const LAZY_DELAY_MS: i64 = 100;

#[derive(Debug)]
pub struct GatewayQueue {
    queue: VecDeque<Value>,
    lazy: BTreeMap<String, (Value, i64)>,
    send_times: VecDeque<i64>,
    max_queue_size: usize,
    send_limit: usize,
}

impl Default for GatewayQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl GatewayQueue {
    pub fn new() -> Self {
        Self::with_limits(MAX_QUEUE_SIZE, SEND_LIMIT)
    }

    pub fn with_limits(max_queue_size: usize, send_limit: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            lazy: BTreeMap::new(),
            send_times: VecDeque::new(),
            max_queue_size,
            send_limit,
        }
    }

    pub fn enqueue(&mut self, payload: Value, now_ms: i64) {
        while self.queue.len() >= self.max_queue_size && self.max_queue_size > 0 {
            let index = self
                .queue
                .iter()
                .position(|value| op(value) == Some(3))
                .unwrap_or(0);
            self.queue.remove(index);
        }
        self.queue.push_back(payload);
        self.prune(now_ms);
    }

    pub fn enqueue_lazy(&mut self, guild_id: impl Into<String>, payload: Value, now_ms: i64) {
        self.lazy
            .insert(guild_id.into(), (payload, now_ms + LAZY_DELAY_MS));
    }

    pub fn next(&mut self, authenticated: bool, now_ms: i64) -> Option<Value> {
        if authenticated {
            return self.queue.pop_front();
        }

        let ready_key = self
            .lazy
            .iter()
            .find(|(_, (_, ready_at))| *ready_at <= now_ms)
            .map(|(key, _)| key.clone());
        let key = ready_key?;
        self.lazy.remove(&key).map(|(payload, _)| payload)
    }

    pub fn can_send(&self, now_ms: i64) -> bool {
        let mut send_times = self.send_times.clone();
        let cutoff = now_ms.saturating_sub(SEND_WINDOW_MS);
        while send_times.front().is_some_and(|time| *time <= cutoff) {
            send_times.pop_front();
        }
        send_times.len() < self.send_limit
    }

    pub fn record_send(&mut self, now_ms: i64) {
        self.prune(now_ms);
        self.send_times.push_back(now_ms);
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.lazy.clear();
        self.send_times.clear();
    }

    pub fn queued(&self) -> usize {
        self.queue.len()
    }

    fn prune(&mut self, now_ms: i64) {
        let cutoff = now_ms.saturating_sub(SEND_WINDOW_MS);
        while self.send_times.front().is_some_and(|time| *time <= cutoff) {
            self.send_times.pop_front();
        }
    }
}

fn op(value: &Value) -> Option<i64> {
    value.get("op")?.as_i64()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(op: i64, id: &str) -> Value {
        serde_json::json!({ "op": op, "d": id })
    }

    #[test]
    fn evicts_presence_updates_before_oldest_payload() {
        let mut queue = GatewayQueue::with_limits(3, 10);
        queue.enqueue(payload(14, "sub-1"), 0);
        queue.enqueue(payload(3, "presence-1"), 0);
        queue.enqueue(payload(1, "heartbeat"), 0);
        queue.enqueue(payload(8, "members"), 0);

        let queued: Vec<_> = (0..3)
            .map(|_| queue.next(true, 0).expect("payload"))
            .collect();
        assert_eq!(queued[0], payload(14, "sub-1"));
        assert_eq!(queued[1], payload(1, "heartbeat"));
        assert_eq!(queued[2], payload(8, "members"));
        assert!(!queued.contains(&payload(3, "presence-1")));
    }

    #[test]
    fn lazy_payloads_are_last_write_wins_and_delayed() {
        let mut queue = GatewayQueue::new();
        queue.enqueue_lazy("guild-1", payload(14, "old"), 0);
        queue.enqueue_lazy("guild-1", payload(14, "new"), 10);

        assert_eq!(queue.next(false, 50), None);
        assert_eq!(queue.next(false, 110), Some(payload(14, "new")));
        assert_eq!(queue.next(false, 200), None);
    }

    #[test]
    fn send_limit_uses_sliding_window() {
        let mut queue = GatewayQueue::with_limits(10, 2);
        assert!(queue.can_send(0));
        queue.record_send(0);
        queue.record_send(1);
        assert!(!queue.can_send(100));
        assert!(queue.can_send(SEND_WINDOW_MS + 2));
    }
}
