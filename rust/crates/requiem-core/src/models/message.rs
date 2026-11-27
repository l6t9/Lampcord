use chrono::{DateTime, Local, TimeZone};
use crate::utils::time;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync;

use super::embed::{Attachment, Embed};
use super::id::{
    Id,
    marker::{ChannelMarker, MessageMarker},
};
use super::user::User;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: Id<MessageMarker>,
    pub channel_id: Id<ChannelMarker>,
    pub author: User,
    pub content: String,
    pub timestamp: String,

    #[serde(default)]
    pub edited_timestamp: Option<String>,

    #[serde(skip)]
    pub(crate) timestamp_ms: sync::OnceLock<Option<i64>>,

    #[serde(default)]
    pub mentions: Vec<User>,

    #[serde(default)]
    pub mention_everyone: bool,

    #[serde(default)]
    pub mention_roles: Vec<Id<super::id::marker::RoleMarker>>,

    #[serde(default)]
    pub embeds: Vec<Embed>,

    #[serde(default)]
    pub attachments: Vec<Attachment>,

    #[serde(default)]
    pub components: Vec<super::Component>,

    #[serde(default)]
    pub nonce: Option<serde_json::Value>,

    #[serde(default)]
    pub message_reference: Option<MessageReference>,

    #[serde(default)]
    pub referenced_message: Option<Box<Message>>,

    #[serde(default)]
    pub thread: Option<Box<super::Channel>>,

    #[serde(default)]
    pub reactions: Vec<Reaction>,

    #[serde(default, rename = "type")]
    pub kind: u8,

    #[serde(default)]
    pub flags: u64,

    #[serde(default)]
    pub pinned: bool,

    #[serde(default)]
    pub webhook_id: Option<Id<super::id::marker::WebhookMarker>>,

    #[serde(default)]
    pub interaction: Option<serde_json::Value>,
}

pub mod message_type {
    pub const DEFAULT: u8 = 0;

    pub const CHANNEL_PINNED_MESSAGE: u8 = 6;

    pub const USER_JOIN: u8 = 7;

    pub const THREAD_CREATED: u8 = 18;
    pub const REPLY: u8 = 19;
    pub const CHAT_INPUT_COMMAND: u8 = 20;

    pub const THREAD_STARTER_MESSAGE: u8 = 21;
    pub const CONTEXT_MENU_COMMAND: u8 = 23;
}

pub mod message_flags {
    pub const HAS_THREAD: u64 = 1 << 5;
    pub const EPHEMERAL: u64 = 1 << 6;
    pub const IS_COMPONENTS_V2: u64 = 1 << 15;
}

pub const MESSAGE_GROUP_INTERVAL_MS: i64 = 7 * 60 * 1000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReactionEmoji {
    #[serde(default)]
    pub id: Option<Id<super::id::marker::EmojiMarker>>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub animated: bool,
}

impl ReactionEmoji {
    pub fn unicode(glyph: &str) -> Self {
        Self {
            id: None,
            name: Some(glyph.to_string()),
            animated: false,
        }
    }

    pub fn same_as(&self, other: &Self) -> bool {
        match (&self.id, &other.id) {
            (Some(a), Some(b)) => a == b,
            (None, None) => self.name == other.name,
            _ => false,
        }
    }

    pub fn path_segment(&self) -> String {
        let raw = match (&self.id, &self.name) {
            (Some(id), Some(name)) => format!("{name}:{id}"),
            (Some(id), None) => format!("_:{id}"),
            (None, name) => name.clone().unwrap_or_default(),
        };
        percent_encoding::utf8_percent_encode(&raw, percent_encoding::NON_ALPHANUMERIC).to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Reaction {
    #[serde(default)]
    pub count: u32,

    #[serde(default)]
    pub me: bool,
    pub emoji: ReactionEmoji,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageReference {
    #[serde(default, rename = "type")]
    pub kind: Option<u8>,
    #[serde(default)]
    pub message_id: Option<Id<MessageMarker>>,
    #[serde(default)]
    pub channel_id: Option<Id<ChannelMarker>>,
}

impl Message {
    pub fn has_flag(&self, flag: u64) -> bool {
        self.flags & flag != 0
    }

    pub fn is_command(&self) -> bool {
        matches!(
            self.kind,
            message_type::CHAT_INPUT_COMMAND | message_type::CONTEXT_MENU_COMMAND
        )
    }

    pub fn is_system(&self) -> bool {
        !matches!(self.kind, message_type::DEFAULT | message_type::REPLY)
            && (!self.is_command() || self.interaction.is_none())
    }

    pub fn starts_group_after(&self, last: &Message) -> bool {
        if self.has_flag(message_flags::HAS_THREAD) || self.is_command() {
            return true;
        }
        if self.kind > message_type::DEFAULT {
            return !last.is_system();
        }
        let within_interval = match (last.timestamp_millis(), self.timestamp_millis()) {
            (Some(last), Some(this)) => (this - last).abs() < MESSAGE_GROUP_INTERVAL_MS,

            _ => true,
        };
        last.is_system()
            || last.author.id != self.author.id
            || last.has_flag(message_flags::EPHEMERAL) != self.has_flag(message_flags::EPHEMERAL)
            || (self.webhook_id.is_some() && last.author.username != self.author.username)
            || !within_interval
    }

    pub fn is_reply(&self) -> bool {
        self.message_reference
            .as_ref()
            .is_some_and(|r| r.kind.unwrap_or(0) == 0 && r.message_id.is_some())
    }
}

impl Message {
    pub fn nonce_str(&self) -> Option<String> {
        match self.nonce.as_ref()? {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            _ => None,
        }
    }

    /// Unix seconds of the last edit, `None` for a message never edited.
    pub fn edited_unix(&self) -> Option<i64> {
        DateTime::parse_from_rfc3339(self.edited_timestamp.as_deref()?)
            .ok()
            .map(|dt| dt.timestamp())
    }

    pub fn timestamp_millis(&self) -> Option<i64> {
        match self.timestamp_ms.get() {
            Some(ms) => *ms,
            None => {
                let ms = DateTime::parse_from_rfc3339(&self.timestamp)
                    .ok()
                    .map(|dt| dt.timestamp_millis());

                let _ = self.timestamp_ms.set(ms);
                ms
            }
        }
    }

    pub fn format_header_timestamp(&self) -> Option<String> {
        self.format_header_timestamp_in(Local::now())
    }

    pub fn format_short_time(&self) -> Option<String> {
        let local = DateTime::parse_from_rfc3339(&self.timestamp)
            .ok()?
            .with_timezone(&Local);
        Some(time::clock(&local))
    }

    fn format_header_timestamp_in<Tz: TimeZone>(&self, now: DateTime<Tz>) -> Option<String>
    where
        Tz::Offset: fmt::Display,
    {
        let local = DateTime::parse_from_rfc3339(&self.timestamp)
            .ok()?
            .with_timezone(&now.timezone());
        let day_diff = local
            .date_naive()
            .signed_duration_since(now.date_naive())
            .num_days();
        let time = time::clock(&local);
        Some(match day_diff {
            0 => time,
            -1 => requiem_i18n::t!("time-yesterday-at", time = time),
            _ => requiem_i18n::t!(
                "time-date-time",
                date = time::date_numeric(&local),
                time = time
            ),
        })
    }
}

pub fn embed_timestamp(timestamp: &str) -> Option<String> {
    let now = Local::now();
    let local = DateTime::parse_from_rfc3339(timestamp)
        .ok()?
        .with_timezone(&Local);
    let time = time::clock(&local);
    Some(
        match local
            .date_naive()
            .signed_duration_since(now.date_naive())
            .num_days()
        {
            0 => requiem_i18n::t!("time-today-at", time = time),
            -1 => requiem_i18n::t!("time-yesterday-at", time = time),
            _ => requiem_i18n::t!(
                "time-date-time",
                date = time::date_numeric(&local),
                time = time
            ),
        },
    )
}

#[cfg(test)]
mod timestamp_tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    fn msg_with_timestamp(timestamp: &str) -> Message {
        Message {
            pinned: false,
            id: Id::new(1),
            channel_id: Id::new(1),
            author: User {
                id: Id::new(1),
                username: "x".to_string(),
                discriminator: "0".to_string(),
                global_name: None,
                avatar: None,

                email: None,
                phone: None,
                verified: None,
                mfa_enabled: None,
                bot: false,
                premium_type: None,
                primary_guild: None,
            },
            content: String::new(),
            timestamp: timestamp.to_string(),
            edited_timestamp: None,
            timestamp_ms: sync::OnceLock::new(),
            mentions: Vec::new(),
            mention_everyone: false,
            mention_roles: Vec::new(),
            embeds: Vec::new(),
            attachments: Vec::new(),
            components: Vec::new(),
            nonce: None,
            message_reference: None,
            referenced_message: None,
            thread: None,
            reactions: Vec::new(),
            kind: 0,
            flags: 0,
            webhook_id: None,
            interaction: None,
        }
    }

    #[test]
    fn epoch_parses_to_zero() {
        assert_eq!(
            msg_with_timestamp("1970-01-01T00:00:00.000Z").timestamp_millis(),
            Some(0)
        );
    }

    #[test]
    fn known_real_timestamp_matches_independently_computed_value() {
        assert_eq!(
            msg_with_timestamp("2026-07-17T13:00:00.000Z").timestamp_millis(),
            Some(1_784_293_200_000)
        );
    }

    #[test]
    fn milliseconds_are_parsed() {
        let a = msg_with_timestamp("2026-07-17T13:00:00.000Z")
            .timestamp_millis()
            .unwrap();
        let b = msg_with_timestamp("2026-07-17T13:00:00.500Z")
            .timestamp_millis()
            .unwrap();
        assert_eq!(b - a, 500);
    }

    #[test]
    fn malformed_timestamp_returns_none() {
        assert_eq!(
            msg_with_timestamp("not-a-timestamp").timestamp_millis(),
            None
        );
    }

    #[test]
    fn same_day_header_timestamp_is_bare_time_no_prefix() {
        let msg = msg_with_timestamp("2026-07-23T15:29:00.000Z");
        let now = Utc.with_ymd_and_hms(2026, 7, 23, 20, 0, 0).unwrap();
        assert_eq!(
            msg.format_header_timestamp_in(now),
            Some("3:29 PM".to_string())
        );
    }

    #[test]
    fn previous_day_gets_yesterday_prefix() {
        let msg = msg_with_timestamp("2026-07-23T21:51:26.000Z");
        let now = Utc.with_ymd_and_hms(2026, 7, 24, 6, 0, 0).unwrap();
        assert_eq!(
            msg.format_header_timestamp_in(now),
            Some("Yesterday at 9:51 PM".to_string())
        );
    }

    #[test]
    fn older_day_header_timestamp_includes_short_date() {
        let msg = msg_with_timestamp("2026-07-17T15:29:00.000Z");
        let now = Utc.with_ymd_and_hms(2026, 7, 23, 20, 0, 0).unwrap();
        assert_eq!(
            msg.format_header_timestamp_in(now),
            Some("7/17/26, 3:29 PM".to_string())
        );
    }

    #[test]
    fn midnight_and_noon_use_twelve_not_zero() {
        let midnight = msg_with_timestamp("2026-07-23T00:00:00.000Z");
        let noon = msg_with_timestamp("2026-07-23T12:00:00.000Z");
        let now = Utc.with_ymd_and_hms(2026, 7, 23, 20, 0, 0).unwrap();
        assert_eq!(
            midnight.format_header_timestamp_in(now),
            Some("12:00 AM".to_string())
        );
        assert_eq!(
            noon.format_header_timestamp_in(now),
            Some("12:00 PM".to_string())
        );
    }
}

#[cfg(test)]
mod spacebar_payload_tests {
    use super::*;

    #[test]
    fn a_spacebar_message_with_attachments_decodes() {
        let json = r#"[{"id":"1149808101717955003","channel_id":"1145194895242273720","thread_id":null,"thread":null,"guild_id":"1006649183970562092","author":{"id":"1006660181259576584","username":"Puyodead1","discriminator":"7970","avatar":"8a7e","avatar_decoration_data":null,"bot":false,"system":false,"banner":"72","accent_color":1,"public_flags":0},"content":"Added user join messages","timestamp":"2023-09-08T20:46:53.845Z","edited_timestamp":null,"tts":false,"mention_everyone":false,"mentions":[],"mention_roles":[],"mention_channels":[],"sticker_items":[],"attachments":[{"id":"1","filename":"s.png","size":112503,"height":856,"width":817,"content_type":"image/png","message_id":"1149808101717955003","channel_id":"1145194895242273720","description":null,"flags":null,"content_scan_version":null,"placeholder_version":null,"placeholder":null,"duration_secs":null,"waveform":null,"title":null,"clip_created_at":null,"url":"https://c/s.png","proxy_url":"https://c/s.png"}],"embeds":[],"reactions":[],"nonce":"1149808095535300609","pinned_at":null,"type":0,"flags":0,"components":[],"username":null,"avatar":null,"message_snapshots":[],"pinned":false}]"#;
        let messages: Vec<Message> = serde_json::from_str(json).expect("spacebar payload decodes");
        assert_eq!(messages[0].attachments.len(), 1);
    }
}

#[cfg(test)]
mod grouping_tests {
    use super::*;

    fn message(author: u64, name: &str, minute: u32, kind: u8) -> Message {
        let mut message: Message = serde_json::from_value(serde_json::json!({
            "id": "1", "channel_id": "1", "content": "", "timestamp": format!("2026-10-08T10:{minute:02}:00.000Z"),
            "author": { "id": author.to_string(), "username": name, "discriminator": "0" },
            "type": kind,
        }))
        .unwrap();
        message.flags = 0;
        message
    }

    #[test]
    fn the_same_author_within_seven_minutes_continues_the_group() {
        let first = message(1, "ana", 0, 0);
        assert!(!message(1, "ana", 6, 0).starts_group_after(&first));
        assert!(
            message(1, "ana", 7, 0).starts_group_after(&first),
            "seven minutes is out (the interval is exclusive)"
        );
        assert!(
            message(2, "bob", 1, 0).starts_group_after(&first),
            "another author"
        );
    }

    #[test]
    fn replies_and_commands_start_a_group_and_system_messages_group_together() {
        let user = message(1, "ana", 0, 0);
        assert!(
            message(1, "ana", 1, message_type::REPLY).starts_group_after(&user),
            "a reply after a normal message"
        );
        assert!(
            message(1, "ana", 1, message_type::CHAT_INPUT_COMMAND).starts_group_after(&user),
            "a command"
        );

        let joined = message(2, "bob", 1, 7);
        assert!(joined.is_system());
        assert!(
            joined.starts_group_after(&user),
            "a system message after a normal one starts a group"
        );
        assert!(
            !message(3, "cy", 2, 7).starts_group_after(&joined),
            "system messages follow each other in one group, whoever sent them"
        );
        assert!(
            message(1, "ana", 2, 0).starts_group_after(&joined),
            "a normal message after a system one"
        );
    }

    #[test]
    fn a_threads_starter_ephemeral_state_and_webhook_names_split_groups() {
        let first = message(1, "ana", 0, 0);
        let mut thread = message(1, "ana", 1, 0);
        thread.flags = message_flags::HAS_THREAD;
        assert!(thread.starts_group_after(&first));
        let mut ephemeral = message(1, "ana", 1, 0);
        ephemeral.flags = message_flags::EPHEMERAL;
        assert!(ephemeral.starts_group_after(&first));

        let mut hook = message(9, "hook", 0, 0);
        hook.webhook_id = Some(Id::new(5));
        let mut renamed = message(9, "hook2", 1, 0);
        renamed.webhook_id = Some(Id::new(5));
        assert!(
            renamed.starts_group_after(&hook),
            "a webhook's other username"
        );
        assert!(
            message(9, "hook", 1, 0).starts_group_after(&first),
            "another author splits"
        );
    }

    #[test]
    fn a_commands_result_with_an_interaction_counts_as_a_normal_message() {
        let mut result = message(1, "bot", 0, message_type::CHAT_INPUT_COMMAND);
        assert!(result.is_system(), "no interaction: a system message");
        result.interaction = Some(serde_json::json!({ "id": "1" }));
        assert!(!result.is_system());
    }
}
