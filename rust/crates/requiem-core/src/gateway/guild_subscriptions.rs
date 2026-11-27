use std::collections::HashSet;
use std::mem;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use tokio::sync::mpsc;

use serde_json::{Map, Value, json};

use super::opcodes;

pub const MINIMUM_RANGE: u32 = 100;

pub const MEMBER_ROW_HEIGHT: f32 = 44.0;

const MAX_CHANNELS: usize = 5;

const FRAME_LIMIT: usize = 15_360;

pub type Ranges = Vec<(u32, u32)>;

pub fn default_ranges() -> Ranges {
    vec![(0, MINIMUM_RANGE - 1)]
}

pub fn ranges_for_dimensions(y: f32, height: f32, row_height: f32) -> Ranges {
    let to_index =
        |n: f32, buffer: i64| -> i64 { ((n / row_height).ceil() as i64 + buffer).max(0) };
    let mut ranges = Ranges::new();
    let mut push_range = |start: i64| -> i64 {
        let end = start + i64::from(MINIMUM_RANGE) - 1;
        ranges.push((start as u32, end as u32));
        end + 1
    };
    let buffer = to_index(height * 0.5, 0);
    let mut next = to_index(y, -buffer);
    let last = to_index(y + height, buffer);

    if next > 0 {
        next = push_range(0).max(next);
    }
    next = next / i64::from(MINIMUM_RANGE) * i64::from(MINIMUM_RANGE);
    while next <= last {
        next = push_range(next);
    }
    ranges
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selected {
    pub guild: u64,
    pub channel: u64,
}

#[derive(Default)]
pub struct GuildSubscriptions {
    typing: HashSet<u64>,
    threads: HashSet<u64>,
    activities: HashSet<u64>,

    channels: Vec<(u64, Vec<(u64, Ranges)>)>,

    subscribed: Vec<u64>,

    pending: Vec<(u64, Map<String, Value>)>,
    selected: Option<Selected>,
    members_visible: bool,
}

fn ranges_json(ranges: &Ranges) -> Value {
    Value::Array(ranges.iter().map(|(a, b)| json!([a, b])).collect())
}

impl GuildSubscriptions {
    fn enqueue(&mut self, guild: u64, key: &str, value: Value) {
        let entry = match self.pending.iter().position(|(g, _)| *g == guild) {
            Some(i) => &mut self.pending[i].1,
            None => {
                self.pending.push((guild, Map::new()));
                &mut self.pending.last_mut().expect("just pushed").1
            }
        };
        entry.insert(key.to_string(), value);
    }

    fn channels_of(&self, guild: u64) -> Option<&Vec<(u64, Ranges)>> {
        self.channels
            .iter()
            .find(|(g, _)| *g == guild)
            .map(|(_, c)| c)
    }

    fn channels_json(&self, guild: u64) -> Value {
        let mut map = Map::new();
        for (channel, ranges) in self.channels_of(guild).into_iter().flatten() {
            map.insert(channel.to_string(), ranges_json(ranges));
        }
        Value::Object(map)
    }

    pub fn subscribe_to_guild(&mut self, guild: u64) {
        if self.typing.insert(guild) {
            self.enqueue(guild, "typing", json!(true));
        }
        if self.activities.insert(guild) {
            self.enqueue(guild, "activities", json!(true));
        }
        if self.threads.insert(guild) {
            self.enqueue(guild, "threads", json!(true));
        }
    }

    pub fn subscribe_channel(&mut self, guild: u64, channel: u64, ranges: Ranges) -> bool {
        let index = match self.channels.iter().position(|(g, _)| *g == guild) {
            Some(i) => i,
            None => {
                self.channels.push((guild, Vec::new()));
                self.channels.len() - 1
            }
        };
        let list = &mut self.channels[index].1;
        let existing = list
            .iter()
            .position(|(c, _)| *c == channel)
            .map(|i| list.remove(i));
        if let Some((_, current)) = &existing {
            let unchanged = *current == ranges;
            list.insert(0, (channel, current.clone()));
            if unchanged {
                return false;
            }
        }
        list.retain(|(c, _)| *c != channel);
        list.insert(0, (channel, ranges));
        list.truncate(MAX_CHANNELS);
        let value = self.channels_json(guild);
        self.enqueue(guild, "channels", value);
        true
    }

    fn subscribe(&mut self, guild: u64, channel: u64) {
        self.subscribe_to_guild(guild);
        if self.members_visible {
            self.subscribe_channel(guild, channel, default_ranges());
        }
    }

    fn get(&self, guild: u64) -> Value {
        json!({
            "typing": self.typing.contains(&guild),
            "threads": self.threads.contains(&guild),
            "activities": self.activities.contains(&guild),
            "members": [],
            "member_updates": false,
            "channels": self.channels_json(guild),
            "thread_member_lists": [],
        })
    }

    fn clear_without_flushing(&mut self, guild: u64) {
        self.subscribed.retain(|g| *g != guild);
        self.pending.retain(|(g, _)| *g != guild);
        self.channels.retain(|(g, _)| *g != guild);
        self.typing.remove(&guild);
        self.threads.remove(&guild);
        self.activities.remove(&guild);
    }

    pub fn flush(&mut self) -> Vec<Value> {
        let pending = mem::take(&mut self.pending);
        for (guild, _) in &pending {
            if !self.subscribed.contains(guild) {
                self.subscribed.push(*guild);
            }
        }
        frames(
            pending
                .into_iter()
                .map(|(g, m)| (g.to_string(), Value::Object(m)))
                .collect(),
        )
    }

    pub fn select_channel(&mut self, guild: u64, channel: u64) -> Vec<Value> {
        self.selected = Some(Selected { guild, channel });
        self.subscribe(guild, channel);
        self.flush()
    }

    pub fn set_members_visible(&mut self, visible: bool) -> Vec<Value> {
        if self.members_visible == visible {
            return Vec::new();
        }
        self.members_visible = visible;
        match self.selected {
            Some(s) => {
                self.subscribe(s.guild, s.channel);
                self.flush()
            }
            None => Vec::new(),
        }
    }

    pub fn member_list_scrolled(&mut self, y: f32, height: f32) -> Vec<Value> {
        let Some(s) = self.selected else {
            return Vec::new();
        };
        if !self.members_visible {
            return Vec::new();
        }
        self.subscribe_channel(
            s.guild,
            s.channel,
            ranges_for_dimensions(y, height, MEMBER_ROW_HEIGHT),
        );
        self.flush()
    }

    pub fn connection_open(&mut self, fresh: bool) -> Vec<Value> {
        if fresh {
            let keep = self.selected.map(|s| s.guild);
            for guild in self.subscribed.clone() {
                if Some(guild) != keep {
                    self.clear_without_flushing(guild);
                }
            }
        }
        if let Some(s) = self.selected {
            self.subscribe(s.guild, s.channel);
        }
        let full: Vec<(String, Value)> = self
            .subscribed
            .iter()
            .map(|g| (g.to_string(), self.get(*g)))
            .collect();
        let mut out = frames(full);
        out.extend(self.flush());
        out
    }
}

fn frames(guilds: Vec<(String, Value)>) -> Vec<Value> {
    let mut out = Vec::new();
    let mut current = Map::new();
    let mut size = 0;
    for (guild, value) in guilds {
        let item = json!([guild, value]).to_string().len();
        if size + item > FRAME_LIMIT && !current.is_empty() {
            out.push(frame(mem::take(&mut current)));
            size = 0;
        }
        current.insert(guild, value);
        size += item;
    }
    if !current.is_empty() {
        out.push(frame(current));
    }
    out
}

fn frame(subscriptions: Map<String, Value>) -> Value {
    json!({ "op": opcodes::GUILD_SUBSCRIPTIONS_BULK, "d": { "subscriptions": subscriptions } })
}

static STATE: Mutex<Option<GuildSubscriptions>> = Mutex::new(None);
static OUTGOING: Mutex<Option<mpsc::UnboundedSender<Value>>> = Mutex::new(None);
static ESTABLISHED: AtomicBool = AtomicBool::new(false);

pub fn set_outgoing(tx: mpsc::UnboundedSender<Value>) {
    *OUTGOING.lock().unwrap_or_else(|e| e.into_inner()) = Some(tx);
}

pub(crate) fn set_established(up: bool) {
    ESTABLISHED.store(up, Ordering::Relaxed);
}

fn with<R>(f: impl FnOnce(&mut GuildSubscriptions) -> R) -> R {
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(GuildSubscriptions::default))
}

pub(crate) fn send_frames(frames: Vec<Value>) {
    send(frames);
}

fn send(frames: Vec<Value>) {
    if frames.is_empty() || !ESTABLISHED.load(Ordering::Relaxed) {
        return;
    }
    if let Some(tx) = OUTGOING.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
        for frame in frames {
            let _ = tx.send(frame);
        }
    }
}

pub fn select_channel(guild: impl Into<u64>, channel: impl Into<u64>) {
    send(with(|s| s.select_channel(guild.into(), channel.into())));
}

pub fn set_members_visible(visible: bool) {
    send(with(|s| s.set_members_visible(visible)));
}

pub fn member_list_scrolled(y: f32, height: f32) {
    send(with(|s| s.member_list_scrolled(y, height)));
}

pub(crate) fn connection_open(fresh: bool) {
    send(with(|s| s.connection_open(fresh)));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(frames: &[Value]) -> String {
        assert_eq!(frames.len(), 1, "{frames:?}");
        frames[0].to_string()
    }

    #[test]
    fn the_ranges_of_an_unscrolled_list_are_the_first_hundred() {
        assert_eq!(ranges_for_dimensions(0.0, 800.0, 44.0), vec![(0, 99)]);
        assert_eq!(ranges_for_dimensions(0.0, 800.0, 44.0), default_ranges());
    }

    #[test]
    fn a_scrolled_list_keeps_the_top_and_covers_what_is_visible() {
        assert_eq!(
            ranges_for_dimensions(5000.0, 800.0, 44.0),
            vec![(0, 99), (100, 199)]
        );

        assert_eq!(
            ranges_for_dimensions(50_000.0, 800.0, 44.0),
            vec![(0, 99), (1100, 1199)]
        );

        assert_eq!(
            ranges_for_dimensions(3500.0, 1600.0, 44.0),
            vec![(0, 99), (100, 199)]
        );
    }

    #[test]
    fn opening_a_guild_sends_the_first_frame_the_real_client_sent() {
        let mut s = GuildSubscriptions::default();
        s.members_visible = true;
        let out = s.select_channel(1527668330054295552, 1542594557797994496);
        assert_eq!(
            one(&out),
            r#"{"op":37,"d":{"subscriptions":{"1527668330054295552":{"typing":true,"activities":true,"threads":true,"channels":{"1542594557797994496":[[0,99]]}}}}}"#
        );
    }

    #[test]
    fn the_same_channel_and_ranges_again_send_nothing() {
        let mut s = GuildSubscriptions::default();
        s.members_visible = true;
        s.select_channel(1, 2);
        assert!(
            s.select_channel(1, 2).is_empty(),
            "already subscribed to everything"
        );
        assert!(
            s.select_channel(1, 3)
                .iter()
                .all(|f| !f.to_string().contains("typing")),
            "the guild features go out once"
        );
    }

    #[test]
    fn scrolling_sends_only_the_changed_channels_object() {
        let mut s = GuildSubscriptions::default();
        s.members_visible = true;
        s.select_channel(1, 2);
        let out = s.member_list_scrolled(5000.0, 800.0);
        assert_eq!(
            one(&out),
            r#"{"op":37,"d":{"subscriptions":{"1":{"channels":{"2":[[0,99],[100,199]]}}}}}"#
        );
        assert!(
            s.member_list_scrolled(5000.0, 800.0).is_empty(),
            "same ranges"
        );
    }

    #[test]
    fn only_five_channels_are_kept_most_recent_first() {
        let mut s = GuildSubscriptions::default();
        s.members_visible = true;
        for channel in 1..=6 {
            s.select_channel(1, channel);
        }
        let last = s.get(1);
        let keys: Vec<&String> = last["channels"].as_object().unwrap().keys().collect();
        assert_eq!(keys, ["6", "5", "4", "3", "2"]);
    }

    #[test]
    fn nothing_is_asked_for_the_channel_while_the_member_list_is_hidden() {
        let mut s = GuildSubscriptions::default();
        let out = s.select_channel(1, 2);
        assert_eq!(
            one(&out),
            r#"{"op":37,"d":{"subscriptions":{"1":{"typing":true,"activities":true,"threads":true}}}}"#
        );
        assert!(s.member_list_scrolled(0.0, 800.0).is_empty());
        let shown = s.set_members_visible(true);
        assert_eq!(
            one(&shown),
            r#"{"op":37,"d":{"subscriptions":{"1":{"channels":{"2":[[0,99]]}}}}}"#
        );
    }

    #[test]
    fn a_reconnect_sends_the_whole_state_in_discords_key_order() {
        let mut s = GuildSubscriptions::default();
        s.members_visible = true;
        s.select_channel(1, 2);
        let out = s.connection_open(false);
        assert_eq!(
            one(&out),
            r#"{"op":37,"d":{"subscriptions":{"1":{"typing":true,"threads":true,"activities":true,"members":[],"member_updates":false,"channels":{"2":[[0,99]]},"thread_member_lists":[]}}}}"#
        );
    }

    #[test]
    fn a_fresh_connection_forgets_every_guild_but_the_open_one() {
        let mut s = GuildSubscriptions::default();
        s.members_visible = true;
        s.select_channel(1, 2);
        s.select_channel(3, 4);
        let out = s.connection_open(true);
        assert!(
            out.iter().all(|f| !f.to_string().contains(r#""1":"#)),
            "guild 1 was dropped: {out:?}"
        );
    }

    #[test]
    fn big_states_are_split_into_frames() {
        let guilds: Vec<(String, Value)> = (0..400u64)
            .map(|g| {
                (
                    g.to_string(),
                    json!({ "channels": { "1": [[0, 99]] }, "pad": "x".repeat(100) }),
                )
            })
            .collect();
        let out = frames(guilds);
        assert!(out.len() > 1);
        assert!(
            out.iter()
                .all(|f| f["d"]["subscriptions"].to_string().len() < FRAME_LIMIT + 400)
        );
    }
}
