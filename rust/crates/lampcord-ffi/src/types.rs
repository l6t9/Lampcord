use serde::{Deserialize, Serialize};

use crate::{error::FfiErrorKind, snapshot::MessageSnapshot};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum AuthFailureKind {
    InvalidCredentials,
    Suspended,
    RateLimited,
    ServerError,
    VerificationRequired,
    AccountDisabled,
    ScheduledForDeletion,
    CaptchaRejected,
    CaptchaCancelled,
    PasswordUpdateRequired,
    Network,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum AuthChallenge {
    Success {
        token: String,
    },
    MfaRequired {
        ticket: String,
        methods: Vec<String>,
    },
    CaptchaRequired {
        sitekey: String,
        service: String,
        rqdata: Option<String>,
        session_id: Option<String>,
        invisible: bool,
    },
    PasswordUpdateRequired,
    SmsSent {
        phone: String,
    },
    ResetSent {
        phone_verification_required: bool,
    },
    Failed {
        kind: AuthFailureKind,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "type", content = "value")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum RemoteAuthState {
    #[default]
    Idle,
    Connecting,
    QrReady {
        url: String,
    },
    UserScanned {
        user_id: String,
        username: String,
    },
    Finished {
        token: String,
    },
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
    LoginPassword {
        login: String,
        password: String,
        captcha_key: Option<String>,
    },
    SubmitMfa {
        code: String,
        ticket: String,
        method: String,
    },
    SendMfaSms {
        ticket: String,
    },
    ForgotPassword {
        login: String,
    },
    StartRemoteAuth,
    CancelRemoteAuth,
    SelectGuild {
        guild_id: Option<u64>,
    },
    SelectChannel {
        channel_id: Option<u64>,
    },
    SetMemberListRanges {
        ranges: Vec<Vec<u32>>,
    },
    FetchMessages {
        channel_id: u64,
        before: Option<u64>,
        limit: Option<u8>,
    },
    FetchMentions {
        limit: Option<u8>,
        before: Option<u64>,
    },
    SendMessage {
        channel_id: u64,
        content: String,
        reply_to_message_id: Option<u64>,
        reply_to_guild_id: Option<u64>,
    },
    /// Loads a profile into `ClientSnapshot.profile`.
    FetchProfile {
        user_id: u64,
        guild_id: Option<u64>,
    },
    /// Sets (or with `None`, clears) the custom status text.
    SetCustomStatus {
        text: Option<String>,
    },
    /// Loads the channel's pinned messages into `ClientSnapshot.pinned_messages`.
    FetchPins {
        channel_id: u64,
    },
    /// Uploads local files and sends them with `content` as one message.
    SendAttachments {
        channel_id: u64,
        content: String,
        file_paths: Vec<String>,
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
    AddReaction {
        channel_id: u64,
        message_id: u64,
        emoji: String,
    },
    RemoveReaction {
        channel_id: u64,
        message_id: u64,
        emoji: String,
    },
    SendTyping {
        channel_id: u64,
    },
    SetStatus {
        status: String,
    },
    AckMessage {
        channel_id: u64,
        message_id: u64,
    },
    MarkGuildRead {
        guild_id: u64,
    },
    PinMessage {
        channel_id: u64,
        message_id: u64,
    },
    UnpinMessage {
        channel_id: u64,
        message_id: u64,
    },
    JoinGuild {
        invite_code: String,
    },
    LeaveGuild {
        guild_id: u64,
    },
    SendFriendRequest {
        username: String,
    },
    AcceptFriendRequest {
        user_id: u64,
    },
    RemoveRelationship {
        user_id: u64,
    },
    BlockUser {
        user_id: u64,
    },
    OpenDm {
        user_id: u64,
    },
    SearchMessages {
        guild_id: Option<u64>,
        channel_id: Option<u64>,
        query: String,
    },
    FetchThreads {
        guild_id: u64,
        channel_id: u64,
    },
    CreateThread {
        channel_id: u64,
        name: String,
        message_id: Option<u64>,
    },
    FetchGuildSettings {
        guild_id: u64,
    },
    CreateGuildRole {
        guild_id: u64,
        name: String,
        color: u32,
    },
    DeleteGuildRole {
        guild_id: u64,
        role_id: u64,
    },
    DeleteGuildEmoji {
        guild_id: u64,
        emoji_id: u64,
    },
    UnbanMember {
        guild_id: u64,
        user_id: u64,
    },
    CreateChannel {
        guild_id: u64,
        name: String,
        kind: u8,
        parent_id: Option<u64>,
    },
    DeleteChannel {
        channel_id: u64,
    },
    SwitchAccount {
        user_id: String,
    },
    RemoveSavedAccount {
        user_id: String,
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
    AuthChanged(AuthChallenge),
    RemoteAuthChanged(RemoteAuthState),
    MessageUpdated(MessageSnapshot),
    MessageDeleted {
        channel_id: u64,
        message_id: u64,
    },
    ReactionChanged {
        channel_id: u64,
        message_id: u64,
        emoji: String,
        added: bool,
    },
    TypingStarted {
        channel_id: u64,
        user_id: u64,
    },
    PresenceUpdated(crate::snapshot::PresenceSnapshot),
    NotificationReceived(MessageSnapshot),
    MentionsLoaded {
        count: u32,
    },
    CommandFailed {
        kind: FfiErrorKind,
        message: String,
    },
    Error {
        message: String,
    },
    LoggedOut,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ClientEventEnvelope {
    pub sequence: u64,
    pub emitted_at_ms: u64,
    pub event: ClientEvent,
}
