use crate::snowflake::Snowflake;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ChannelType(pub u8);

impl ChannelType {
    pub const GUILD_TEXT: Self = Self(0);
    pub const DM: Self = Self(1);
    pub const GUILD_VOICE: Self = Self(2);
    pub const GROUP_DM: Self = Self(3);
    pub const GUILD_CATEGORY: Self = Self(4);
    pub const GUILD_ANNOUNCEMENT: Self = Self(5);
    pub const ANNOUNCEMENT_THREAD: Self = Self(10);
    pub const PUBLIC_THREAD: Self = Self(11);
    pub const PRIVATE_THREAD: Self = Self(12);
    pub const GUILD_STAGE_VOICE: Self = Self(13);
    pub const GUILD_DIRECTORY: Self = Self(14);
    pub const GUILD_FORUM: Self = Self(15);

    pub const fn is_voice(self) -> bool {
        matches!(self.0, 2 | 13)
    }

    pub const fn is_thread(self) -> bool {
        matches!(self.0, 10 | 11 | 12)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Channel {
    #[serde(with = "crate::snowflake::serde")]
    pub id: Snowflake,
    #[serde(default, rename = "type")]
    pub kind: ChannelType,
    #[serde(default, with = "crate::snowflake::serde::option")]
    pub guild_id: Option<Snowflake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<u16>,
    #[serde(default, with = "crate::snowflake::serde::option")]
    pub parent_id: Option<Snowflake>,
    #[serde(default, with = "crate::snowflake::serde::option")]
    pub last_message_id: Option<Snowflake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_limit: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission_overwrites: Option<Vec<serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recipients: Vec<crate::user::User>,
}
