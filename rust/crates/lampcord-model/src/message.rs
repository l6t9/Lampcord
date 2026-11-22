use crate::snowflake::Snowflake;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessageType(pub u8);

impl MessageType {
    pub const DEFAULT: Self = Self(0);
    pub const REPLY: Self = Self(19);
    pub const APPLICATION_COMMAND: Self = Self(20);
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attachment {
    #[serde(with = "crate::snowflake::serde")]
    pub id: Snowflake,
    pub filename: String,
    pub size: u64,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageReference {
    #[serde(default, with = "crate::snowflake::serde::option")]
    pub message_id: Option<Snowflake>,
    #[serde(default, with = "crate::snowflake::serde::option")]
    pub channel_id: Option<Snowflake>,
    #[serde(default, with = "crate::snowflake::serde::option")]
    pub guild_id: Option<Snowflake>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    #[serde(with = "crate::snowflake::serde")]
    pub id: Snowflake,
    #[serde(with = "crate::snowflake::serde")]
    pub channel_id: Snowflake,
    #[serde(default, with = "crate::snowflake::serde::option")]
    pub guild_id: Option<Snowflake>,
    pub author: crate::user::User,
    #[serde(default)]
    pub content: String,
    pub timestamp: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edited_timestamp: Option<String>,
    #[serde(default)]
    pub tts: bool,
    #[serde(default)]
    pub mention_everyone: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mentions: Vec<serde_json::Value>,
    #[serde(default, with = "crate::snowflake::serde::vec")]
    pub mention_roles: Vec<Snowflake>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub embeds: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reactions: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_reference: Option<MessageReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub referenced_message: Option<Box<Message>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sticker_items: Vec<serde_json::Value>,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default, rename = "type")]
    pub kind: MessageType,
}
