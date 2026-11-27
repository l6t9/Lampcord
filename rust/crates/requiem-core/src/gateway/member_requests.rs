use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use serde_json::{Value, json};

use super::opcodes;

const CHUNK: usize = 100;

#[derive(Default)]
pub struct Requester {
    unacknowledged: HashMap<u64, Vec<u64>>,

    sent: HashMap<u64, HashSet<u64>>,
}

impl Requester {
    pub fn request(
        &mut self,
        guild: u64,
        users: impl IntoIterator<Item = u64>,
        known: impl Fn(u64) -> bool,
    ) -> Vec<Value> {
        let mut wanted: Vec<u64> = Vec::new();
        for user in users {
            if known(user)
                || self.sent.get(&guild).is_some_and(|s| s.contains(&user))
                || wanted.contains(&user)
            {
                continue;
            }
            wanted.push(user);
        }
        for user in &wanted {
            self.sent.entry(guild).or_default().insert(*user);
            self.unacknowledged.entry(guild).or_default().push(*user);
        }
        frames(guild, &wanted)
    }

    pub fn acknowledge(&mut self, guild: u64, users: impl IntoIterator<Item = u64>) {
        if let Some(list) = self.unacknowledged.get_mut(&guild) {
            for user in users {
                list.retain(|u| *u != user);
            }
        }
    }

    pub fn resend_unacknowledged(&mut self, known: impl Fn(u64, u64) -> bool) -> Vec<Value> {
        let mut out = Vec::new();
        let guilds: Vec<u64> = self.unacknowledged.keys().copied().collect();
        for guild in guilds {
            let list = self.unacknowledged.entry(guild).or_default();
            list.retain(|user| !known(guild, *user));
            let users = list.clone();
            out.extend(frames(guild, &users));
        }
        out
    }

    pub fn reset(&mut self) {
        self.unacknowledged.clear();
        self.sent.clear();
    }
}

fn frames(guild: u64, users: &[u64]) -> Vec<Value> {
    users
        .chunks(CHUNK)
        .map(|chunk| json!({ "op": opcodes::REQUEST_GUILD_MEMBERS, "d": { "guild_id": [guild.to_string()], "user_ids": chunk.iter().map(u64::to_string).collect::<Vec<_>>(), "presences": false } }))
        .collect()
}

static STATE: Mutex<Option<Requester>> = Mutex::new(None);

fn with<R>(f: impl FnOnce(&mut Requester) -> R) -> R {
    f(STATE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_or_insert_with(Requester::default))
}

pub fn request(guild: u64, users: impl IntoIterator<Item = u64>, known: impl Fn(u64) -> bool) {
    let frames = with(|r| r.request(guild, users, known));
    super::guild_subscriptions::send_frames(frames);
}

pub fn acknowledge(guild: u64, users: impl IntoIterator<Item = u64>) {
    with(|r| r.acknowledge(guild, users));
}

pub fn resend_unacknowledged(known: impl Fn(u64, u64) -> bool) {
    let frames = with(|r| r.resend_unacknowledged(known));
    super::guild_subscriptions::send_frames(frames);
}

pub fn request_query(guild: u64, query: &str) {
    super::guild_subscriptions::send_frames(vec![
        json!({ "op": opcodes::REQUEST_GUILD_MEMBERS, "d": { "guild_id": [guild.to_string()], "query": query, "limit": 20, "presences": true } }),
    ]);
}

pub fn reset() {
    with(Requester::reset);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_frame_is_the_one_the_real_client_sent() {
        let mut r = Requester::default();
        let out = r.request(1527668330054295552, [1527666663510511616], |_| false);
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].to_string(),
            r#"{"op":8,"d":{"guild_id":["1527668330054295552"],"user_ids":["1527666663510511616"],"presences":false}}"#
        );
    }

    #[test]
    fn known_members_and_users_already_asked_for_are_skipped() {
        let mut r = Requester::default();
        assert_eq!(
            r.request(1, [10, 11, 10], |u| u == 11).len(),
            1,
            "11 is a member, 10 appears once"
        );
        assert!(r.request(1, [10], |_| false).is_empty(), "10 was asked for");
        assert_eq!(
            r.request(2, [10], |_| false).len(),
            1,
            "another guild is another question"
        );
    }

    #[test]
    fn a_hundred_users_to_a_frame() {
        let mut r = Requester::default();
        let out = r.request(1, 1..=250u64, |_| false);
        assert_eq!(
            out.iter()
                .map(|f| f["d"]["user_ids"].as_array().unwrap().len())
                .collect::<Vec<_>>(),
            [100, 100, 50]
        );
    }

    #[test]
    fn a_resume_asks_again_only_for_the_unanswered() {
        let mut r = Requester::default();
        r.request(1, [10, 11, 12], |_| false);
        r.acknowledge(1, [10]);
        let out = r.resend_unacknowledged(|_, user| user == 12);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0]["d"]["user_ids"], json!(["11"]));
    }

    #[test]
    fn a_new_connection_forgets_what_was_asked() {
        let mut r = Requester::default();
        r.request(1, [10], |_| false);
        r.reset();
        assert_eq!(r.request(1, [10], |_| false).len(), 1);
    }
}
