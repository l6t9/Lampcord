use std::collections::HashMap;

use serde_json::Value;

pub const DISCORD_EPOCH_MS: i64 = 1_420_070_400_000;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReadState {
    pub last_message_id: Option<u64>,
    pub mention_count: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReadStateStore {
    states: HashMap<u64, ReadState>,
    recent: HashMap<u64, u64>,
    local_acks: HashMap<u64, u64>,
}

impl ReadStateStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_ready(&mut self, ready: &Value) {
        let states = ready
            .get("read_state")
            .or_else(|| ready.get("read_states"))
            .and_then(Value::as_array);

        let Some(states) = states else {
            return;
        };

        for state in states {
            let Some(channel_id) = parse_u64(state.get("channel_id")) else {
                continue;
            };
            let last_message_id = parse_u64(state.get("last_message_id"));
            let mention_count = state
                .get("mention_count")
                .and_then(Value::as_u64)
                .unwrap_or_default()
                .min(u32::MAX as u64) as u32;
            let entry = self.states.entry(channel_id).or_default();
            entry.last_message_id = max_option(entry.last_message_id, last_message_id);
            entry.mention_count = mention_count;
        }
    }

    pub fn with_ack(&mut self, channel_id: u64, message_id: u64, mention_count: u32) {
        let state = self.states.entry(channel_id).or_default();
        state.last_message_id = max_option(state.last_message_id, Some(message_id));
        state.mention_count = mention_count;
        self.record_message(channel_id, message_id, 0);
    }

    pub fn record_message(&mut self, channel_id: u64, message_id: u64, mention_count: u32) {
        self.recent
            .entry(channel_id)
            .and_modify(|recent| *recent = (*recent).max(message_id))
            .or_insert(message_id);
        if mention_count > 0 {
            let state = self.states.entry(channel_id).or_default();
            state.mention_count = state.mention_count.saturating_add(mention_count);
        }
    }

    pub fn ack(&mut self, channel_id: u64, message_id: u64) {
        self.local_acks
            .entry(channel_id)
            .and_modify(|ack| *ack = (*ack).max(message_id))
            .or_insert(message_id);
        if let Some(state) = self.states.get_mut(&channel_id) {
            state.last_message_id = max_option(state.last_message_id, Some(message_id));
            state.mention_count = 0;
        }
    }

    pub fn is_unread(
        &self,
        channel_id: u64,
        channel_last_message_id: Option<u64>,
        join_timestamp_ms: Option<i64>,
    ) -> bool {
        let newest = max_option(
            channel_last_message_id,
            self.recent.get(&channel_id).copied(),
        );
        let Some(newest) = newest else {
            return false;
        };
        let ack = max_option(
            self.local_acks.get(&channel_id).copied(),
            self.states
                .get(&channel_id)
                .and_then(|state| state.last_message_id),
        );
        let ack = max_option(
            ack,
            join_timestamp_ms
                .map(guild_join_ack_id)
                .filter(|id| *id > 0),
        );
        ack.is_none_or(|ack| newest > ack)
    }

    pub fn most_recent_message_id(&self, channel_id: u64) -> Option<u64> {
        max_option(
            self.recent.get(&channel_id).copied(),
            self.states
                .get(&channel_id)
                .and_then(|state| state.last_message_id),
        )
    }

    pub fn last_read_message_id(&self, channel_id: u64) -> Option<u64> {
        max_option(
            self.local_acks.get(&channel_id).copied(),
            self.states
                .get(&channel_id)
                .and_then(|state| state.last_message_id),
        )
    }

    pub fn mention_count(&self, channel_id: u64) -> u32 {
        self.states
            .get(&channel_id)
            .map(|state| state.mention_count)
            .unwrap_or_default()
    }

    pub fn clear(&mut self) {
        self.states.clear();
        self.recent.clear();
        self.local_acks.clear();
    }
}

pub fn guild_join_ack_id(timestamp_ms: i64) -> u64 {
    let elapsed = timestamp_ms.saturating_sub(DISCORD_EPOCH_MS).max(0) as u64;
    elapsed.saturating_mul(1 << 22)
}

fn parse_u64(value: Option<&Value>) -> Option<u64> {
    let value = value?;
    value
        .as_u64()
        .or_else(|| value.as_str()?.parse::<u64>().ok())
}

fn max_option(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_states_from_ready() {
        let ready = serde_json::json!({
            "read_state": [
                {"channel_id": "1", "last_message_id": "10", "mention_count": 2},
                {"channel_id": "2", "last_message_id": 20, "mention_count": 0}
            ]
        });
        let mut store = ReadStateStore::new();
        store.handle_ready(&ready);

        assert_eq!(store.most_recent_message_id(1), Some(10));
        assert_eq!(store.most_recent_message_id(2), Some(20));
        assert_eq!(
            store.states.get(&1).map(|state| state.mention_count),
            Some(2)
        );
    }

    #[test]
    fn acks_are_monotonic_and_clear_unread() {
        let mut store = ReadStateStore::new();
        store.with_ack(1, 10, 2);
        store.ack(1, 5);
        assert!(store.is_unread(1, Some(11), None));
        store.ack(1, 12);
        assert!(!store.is_unread(1, Some(11), None));
    }

    #[test]
    fn record_message_tracks_recent_without_acking() {
        let mut store = ReadStateStore::new();
        store.record_message(1, 10, 1);
        assert_eq!(store.most_recent_message_id(1), Some(10));
        assert_eq!(store.mention_count(1), 1);
        assert!(store.is_unread(1, Some(10), None));
    }

    #[test]
    fn guild_join_ack_marks_old_messages_as_read() {
        let join_ack = guild_join_ack_id(DISCORD_EPOCH_MS + 1_000);
        assert!(join_ack > 0);
        let store = ReadStateStore::new();
        assert!(!store.is_unread(1, Some(join_ack - 1), Some(DISCORD_EPOCH_MS + 1_000)));
        assert!(store.is_unread(1, Some(join_ack + 1), Some(DISCORD_EPOCH_MS + 1_000)));
    }
}
