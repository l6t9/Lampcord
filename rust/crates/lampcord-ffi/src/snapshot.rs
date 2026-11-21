use serde::{Deserialize, Serialize};

use crate::types::{AuthChallenge, ConnectionState, RemoteAuthState};

pub const SNAPSHOT_SCHEMA_VERSION: u32 = 2;

pub const MAX_SNAPSHOT_MEMBERS: usize = 200;
pub const MAX_SNAPSHOT_PRESENCES: usize = 200;
pub const MAX_SNAPSHOT_READ_STATES: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ClientSnapshot {
    pub schema_version: u32,
    pub seq: u64,
    pub connection: ConnectionState,
    pub has_token: bool,
    pub current_user_id: Option<u64>,
    pub current_user: Option<String>,
    pub selected_guild_id: Option<u64>,
    pub selected_channel_id: Option<u64>,
    pub guilds: Vec<GuildSnapshot>,
    pub channels: Vec<ChannelSnapshot>,
    pub messages: Vec<MessageSnapshot>,
    pub members: Vec<MemberSnapshot>,
    pub presences: Vec<PresenceSnapshot>,
    pub read_states: Vec<ReadStateSnapshot>,
    pub auth: AuthStateSnapshot,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct AuthStateSnapshot {
    pub challenge: Option<AuthChallenge>,
    pub remote: RemoteAuthState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct GuildSnapshot {
    pub id: u64,
    pub name: String,
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ChannelSnapshot {
    pub id: u64,
    pub guild_id: Option<u64>,
    pub name: String,
    pub kind: u8,
    pub parent_id: Option<u64>,
    pub position: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct MessageSnapshot {
    pub id: u64,
    pub channel_id: u64,
    pub author_id: Option<u64>,
    pub author_name: String,
    pub content: String,
    pub timestamp: String,
    pub edited_timestamp: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct MemberSnapshot {
    pub guild_id: u64,
    pub user_id: u64,
    pub username: String,
    pub avatar: Option<String>,
    pub nickname: Option<String>,
    pub roles: Vec<u64>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PresenceSnapshot {
    pub user_id: u64,
    pub status: String,
    pub activities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ReadStateSnapshot {
    pub channel_id: u64,
    pub last_message_id: Option<u64>,
    pub last_read_message_id: Option<u64>,
    pub mention_count: u32,
    pub unread: bool,
}
