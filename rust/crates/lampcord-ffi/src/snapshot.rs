use serde::{Deserialize, Serialize};

use crate::types::ConnectionState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ClientSnapshot {
    pub connection: ConnectionState,
    pub has_token: bool,
    pub current_user_id: Option<u64>,
    pub current_user: Option<String>,
    pub selected_guild_id: Option<u64>,
    pub selected_channel_id: Option<u64>,
    pub guilds: Vec<GuildSnapshot>,
    pub channels: Vec<ChannelSnapshot>,
    pub messages: Vec<MessageSnapshot>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuildSnapshot {
    pub id: u64,
    pub name: String,
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelSnapshot {
    pub id: u64,
    pub guild_id: Option<u64>,
    pub name: String,
    pub kind: u8,
    pub parent_id: Option<u64>,
    pub position: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageSnapshot {
    pub id: u64,
    pub channel_id: u64,
    pub author_id: Option<u64>,
    pub author_name: String,
    pub content: String,
    pub timestamp: String,
    pub edited_timestamp: Option<String>,
}
