use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ClientConfig {
    pub api_base: String,
    pub gateway_url: Option<String>,
    pub user_agent: Option<String>,
    pub storage_path: Option<String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_base: "https://discord.com/api/v9".to_owned(),
            gateway_url: None,
            user_agent: None,
            storage_path: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "type", content = "value")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Ready,
    Resuming,
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum ClientCommand {
    ConnectToken {
        token: String,
    },
    ConnectStoredToken,
    SelectGuild {
        guild_id: Option<u64>,
    },
    SelectChannel {
        channel_id: Option<u64>,
    },
    FetchMessages {
        channel_id: u64,
        before: Option<u64>,
        limit: Option<u8>,
    },
    SendMessage {
        channel_id: u64,
        content: String,
        reply_to_message_id: Option<u64>,
        reply_to_guild_id: Option<u64>,
    },
    EditMessage {
        channel_id: u64,
        message_id: u64,
        content: String,
    },
    DeleteMessage {
        channel_id: u64,
        message_id: u64,
    },
    SendTyping {
        channel_id: u64,
    },
    AckMessage {
        channel_id: u64,
        message_id: u64,
    },
    Logout,
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum ClientEvent {
    SnapshotUpdated,
    ConnectionChanged(ConnectionState),
    Error { message: String },
    LoggedOut,
}
