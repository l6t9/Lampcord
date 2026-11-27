use serde::{Deserialize, Serialize};

use crate::models::Id;
use crate::models::id::marker::{ChannelMarker, GuildMarker};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamKey {
    Guild {
        guild_id: u64,
        channel_id: u64,
        owner_id: u64,
    },
    Call {
        channel_id: u64,
        owner_id: u64,
    },
}

impl StreamKey {
    pub fn parse(s: &str) -> Option<StreamKey> {
        let mut parts = s.split(':');
        match (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) {
            (Some("guild"), Some(guild), Some(channel), Some(owner), None) => {
                Some(StreamKey::Guild {
                    guild_id: guild.parse().ok()?,
                    channel_id: channel.parse().ok()?,
                    owner_id: owner.parse().ok()?,
                })
            }
            (Some("call"), Some(channel), Some(owner), None, _) => Some(StreamKey::Call {
                channel_id: channel.parse().ok()?,
                owner_id: owner.parse().ok()?,
            }),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match *self {
            StreamKey::Guild {
                guild_id,
                channel_id,
                owner_id,
            } => format!("guild:{guild_id}:{channel_id}:{owner_id}"),
            StreamKey::Call {
                channel_id,
                owner_id,
            } => format!("call:{channel_id}:{owner_id}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamType {
    Guild,
    Call,
}

impl StreamType {
    pub fn as_str(self) -> &'static str {
        match self {
            StreamType::Guild => "guild",
            StreamType::Call => "call",
        }
    }
}

#[derive(Serialize)]
pub struct StreamCreateRequest<'a> {
    #[serde(rename = "type")]
    pub kind: &'a str,
    pub guild_id: u64,
    pub channel_id: u64,
    pub preferred_region: &'a str,
}

#[derive(Serialize)]
pub struct StreamDeleteRequest<'a> {
    pub stream_key: &'a str,
}

#[derive(Serialize)]
pub struct StreamWatchRequest<'a> {
    pub stream_key: &'a str,
}

#[derive(Serialize)]
pub struct StreamPingRequest<'a> {
    pub stream_key: &'a str,
}

#[derive(Serialize)]
pub struct StreamSetPausedRequest<'a> {
    pub stream_key: &'a str,
    pub paused: bool,
}

#[derive(Deserialize)]
pub struct StreamCreate {
    pub stream_key: String,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub viewer_ids: Vec<String>,
    pub rtc_server_id: Id<GuildMarker>,
    pub rtc_channel_id: Id<ChannelMarker>,
    #[serde(default)]
    pub paused: bool,
}

#[derive(Deserialize)]
pub struct StreamServerUpdate {
    pub stream_key: String,
    pub endpoint: String,
    pub token: String,
}

#[derive(Deserialize)]
pub struct StreamUpdate {
    pub stream_key: String,
}

#[derive(Deserialize)]
pub struct StreamDelete {
    pub stream_key: String,
    #[serde(default)]
    pub unavailable: bool,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Deserialize)]
pub struct StreamClose {
    pub stream_key: String,
}

pub const STREAM_FULL: &str = "STREAM_FULL";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guild_key_roundtrip() {
        let key = StreamKey::parse("guild:111:222:333").unwrap();
        assert_eq!(
            key,
            StreamKey::Guild {
                guild_id: 111,
                channel_id: 222,
                owner_id: 333
            }
        );
        assert_eq!(key.to_string(), "guild:111:222:333");
    }

    #[test]
    fn call_key_roundtrip() {
        let key = StreamKey::parse("call:444:555").unwrap();
        assert_eq!(
            key,
            StreamKey::Call {
                channel_id: 444,
                owner_id: 555
            }
        );
        assert_eq!(key.to_string(), "call:444:555");
    }

    #[test]
    fn rejects_malformed() {
        assert!(StreamKey::parse("guild:1:2").is_none());
        assert!(StreamKey::parse("guild:1:2:3:4").is_none());
        assert!(StreamKey::parse("guild:a:b:c").is_none());
        assert!(StreamKey::parse("").is_none());
        assert!(StreamKey::parse("foo:1:2").is_none());
    }
}
