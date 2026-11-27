use serde::{Deserialize, Serialize};

use super::id::{
    Id,
    marker::{ChannelMarker, GuildMarker, MessageMarker, UserMarker},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Channel {
    pub id: Id<ChannelMarker>,
    pub guild_id: Option<Id<GuildMarker>>,
    #[serde(rename = "type")]
    pub kind: u8,
    pub name: Option<String>,
    pub position: Option<i32>,

    #[serde(default)]
    pub parent_id: Option<Id<ChannelMarker>>,

    #[serde(default)]
    pub permission_overwrites: Vec<super::PermissionOverwrite>,

    #[serde(default)]
    pub flags: u64,

    #[serde(default)]
    pub nsfw: bool,

    #[serde(default)]
    pub last_message_id: Option<Id<MessageMarker>>,

    #[serde(default)]
    pub owner_id: Option<Id<UserMarker>>,
    #[serde(default)]
    pub thread_metadata: Option<ThreadMetadata>,
    #[serde(default)]
    pub message_count: u32,
    #[serde(default)]
    pub member_count: u32,
    #[serde(default)]
    pub total_message_sent: u32,

    #[serde(default)]
    pub member: Option<ThreadMember>,

    #[serde(default)]
    pub newly_created: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ThreadMetadata {
    #[serde(default)]
    pub archived: bool,
    #[serde(default = "default_auto_archive")]
    pub auto_archive_duration: u32,
    #[serde(default)]
    pub archive_timestamp: Option<String>,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub invitable: Option<bool>,
    #[serde(default)]
    pub create_timestamp: Option<String>,
}

fn default_auto_archive() -> u32 {
    1440
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThreadMember {
    /// Absent on the member embedded in a thread, which is the thread's own.
    #[serde(default, alias = "thread_id")]
    pub id: Option<Id<ChannelMarker>>,
    #[serde(default)]
    pub user_id: Option<Id<UserMarker>>,
    #[serde(default, alias = "joined_at")]
    pub join_timestamp: Option<String>,
    #[serde(default)]
    pub flags: u64,
    #[serde(default)]
    pub muted: bool,
}

impl Channel {
    pub const OBFUSCATED: u64 = 1 << 17;

    pub fn is_obfuscated(&self) -> bool {
        self.flags & Self::OBFUSCATED != 0 || self.name.as_deref() == Some("___hidden___")
    }

    pub const fn is_thread(&self) -> bool {
        matches!(self.kind, 10..=12)
    }

    pub fn is_archived_thread(&self) -> bool {
        self.thread_metadata
            .as_ref()
            .is_some_and(|metadata| metadata.archived)
    }

    pub fn is_locked_thread(&self) -> bool {
        self.thread_metadata
            .as_ref()
            .is_some_and(|metadata| metadata.locked)
    }
}
