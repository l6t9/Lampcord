use crate::guild::UnavailableGuild;
use crate::user::User;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GatewayOp(pub u8);

impl GatewayOp {
    pub const DISPATCH: Self = Self(0);
    pub const HEARTBEAT: Self = Self(1);
    pub const IDENTIFY: Self = Self(2);
    pub const PRESENCE_UPDATE: Self = Self(3);
    pub const VOICE_STATE_UPDATE: Self = Self(4);
    pub const RESUME: Self = Self(6);
    pub const RECONNECT: Self = Self(7);
    pub const REQUEST_GUILD_MEMBERS: Self = Self(8);
    pub const INVALID_SESSION: Self = Self(9);
    pub const HELLO: Self = Self(10);
    pub const HEARTBEAT_ACK: Self = Self(11);
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayPayload<T = serde_json::Value> {
    pub op: GatewayOp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d: Option<T>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub t: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hello {
    pub heartbeat_interval: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shard {
    pub shard_id: u16,
    pub num_shards: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ready {
    pub v: u8,
    pub user: User,
    pub guilds: Vec<UnavailableGuild>,
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume_gateway_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shard: Option<Shard>,
}
