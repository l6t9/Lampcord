use serde::{Deserialize, Serialize};

use crate::types::{AuthChallenge, ConnectionState, RemoteAuthState};

pub const SNAPSHOT_SCHEMA_VERSION: u32 = 4;

pub const MAX_SNAPSHOT_MEMBERS: usize = 200;
pub const MAX_SNAPSHOT_PRESENCES: usize = 200;
pub const MAX_SNAPSHOT_READ_STATES: usize = 200;
pub const MAX_SNAPSHOT_PRIVATE_CHANNELS: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ClientSnapshot {
    pub schema_version: u32,
    pub seq: u64,
    pub connection: ConnectionState,
    pub has_token: bool,
    pub current_user_id: Option<u64>,
    pub current_user: Option<String>,
    #[serde(default)]
    pub current_user_username: Option<String>,
    #[serde(default)]
    pub current_user_email: Option<String>,
    #[serde(default)]
    pub current_user_mfa_enabled: bool,
    pub current_user_avatar: Option<String>,
    pub selected_guild_id: Option<u64>,
    pub selected_channel_id: Option<u64>,
    pub guilds: Vec<GuildSnapshot>,
    pub channels: Vec<ChannelSnapshot>,
    pub messages: Vec<MessageSnapshot>,
    pub members: Vec<MemberSnapshot>,
    pub presences: Vec<PresenceSnapshot>,
    pub read_states: Vec<ReadStateSnapshot>,
    pub member_list: MemberListSnapshot,
    pub relationships: Vec<RelationshipSnapshot>,
    pub auth: AuthStateSnapshot,
    pub error: Option<String>,
    /// Schema 3+. DMs and group DMs, most recent first; versioned with `channels`.
    pub private_channels: Vec<PrivateChannelSnapshot>,
    /// Schema 3+. The user's server folders in rail order; versioned with `guilds`.
    pub guild_folders: Vec<GuildFolderSnapshot>,
    /// Schema 3+. Status picked in settings (online, idle, dnd, invisible); part of `meta`.
    pub current_user_status: Option<String>,
    /// Schema 3+. Custom status text; part of `meta`.
    pub current_user_custom_status: Option<String>,
    /// Schema 3+. Your avatar frame and nameplate assets; part of `meta`.
    pub current_user_decoration: Option<String>,
    pub current_user_nameplate: Option<String>,
    /// Schema 3+. The last profile fetched with FetchProfile; part of `meta`.
    pub profile: Option<ProfileSnapshot>,
    /// Schema 3+. Pins of the selected channel once fetched; versioned with `messages`.
    pub pinned_messages: Vec<MessageSnapshot>,
    /// Schema 4+. The last mentions page fetched by the client; versioned with `messages`.
    #[serde(default)]
    pub mentions: Vec<MessageSnapshot>,
    /// Schema 4+. Last search results fetched by the client; versioned with `messages`.
    #[serde(default)]
    pub search_results: Vec<MessageSnapshot>,
    /// Schema 4+. Custom emojis for the selected guild; versioned with `guilds`.
    #[serde(default)]
    pub guild_emojis: Vec<EmojiSnapshot>,
    /// Schema 4+. Roles for the selected guild; versioned with `guilds`.
    #[serde(default)]
    pub guild_roles: Vec<RoleSnapshot>,
    /// Schema 4+. Active/archived threads for the selected channel/guild; versioned with `channels`.
    #[serde(default)]
    pub threads: Vec<ThreadSnapshot>,
    /// Schema 4+. Bans for the selected guild (loaded on demand); versioned with `guilds`.
    #[serde(default)]
    pub guild_bans: Vec<BanSnapshot>,
    /// Schema 4+. Saved accounts from the local token store; versioned with `meta`.
    #[serde(default)]
    pub saved_accounts: Vec<SavedAccountSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct SnapshotVersions {
    pub seq: u64,
    pub meta: u64,
    pub guilds: u64,
    pub channels: u64,
    pub messages: u64,
    pub members: u64,
    pub presences: u64,
    pub read_states: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct SnapshotMetaPatch {
    pub connection: ConnectionState,
    pub has_token: bool,
    pub current_user_id: Option<u64>,
    pub current_user: Option<String>,
    #[serde(default)]
    pub current_user_username: Option<String>,
    #[serde(default)]
    pub current_user_email: Option<String>,
    #[serde(default)]
    pub current_user_mfa_enabled: bool,
    pub current_user_avatar: Option<String>,
    pub current_user_status: Option<String>,
    pub current_user_custom_status: Option<String>,
    pub current_user_decoration: Option<String>,
    pub current_user_nameplate: Option<String>,
    pub profile: Option<ProfileSnapshot>,
    pub selected_guild_id: Option<u64>,
    pub selected_channel_id: Option<u64>,
    pub auth: AuthStateSnapshot,
    pub error: Option<String>,
    #[serde(default)]
    pub saved_accounts: Vec<SavedAccountSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct SnapshotPatch {
    pub seq: u64,
    pub schema_version: u32,
    pub versions: SnapshotVersions,
    pub meta: Option<SnapshotMetaPatch>,
    pub guilds: Option<Vec<GuildSnapshot>>,
    pub channels: Option<Vec<ChannelSnapshot>>,
    pub messages: Option<Vec<MessageSnapshot>>,
    pub pinned_messages: Option<Vec<MessageSnapshot>>,
    pub mentions: Option<Vec<MessageSnapshot>>,
    #[serde(default)]
    pub search_results: Option<Vec<MessageSnapshot>>,
    #[serde(default)]
    pub guild_emojis: Option<Vec<EmojiSnapshot>>,
    #[serde(default)]
    pub guild_roles: Option<Vec<RoleSnapshot>>,
    #[serde(default)]
    pub threads: Option<Vec<ThreadSnapshot>>,
    #[serde(default)]
    pub guild_bans: Option<Vec<BanSnapshot>>,
    pub members: Option<Vec<MemberSnapshot>>,
    pub presences: Option<Vec<PresenceSnapshot>>,
    pub read_states: Option<Vec<ReadStateSnapshot>>,
    pub private_channels: Option<Vec<PrivateChannelSnapshot>>,
    pub guild_folders: Option<Vec<GuildFolderSnapshot>>,
    pub member_list: Option<MemberListSnapshot>,
    pub relationships: Vec<RelationshipSnapshot>,
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
    #[serde(default)]
    pub owner_id: Option<u64>,
    #[serde(default)]
    pub unread: bool,
    #[serde(default)]
    pub mention_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PrivateChannelSnapshot {
    pub id: u64,
    /// 1 for a DM, 3 for a group DM.
    pub kind: u8,
    /// Only group DMs can have a name.
    pub name: Option<String>,
    pub recipients: Vec<RecipientSnapshot>,
    pub last_message_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RecipientSnapshot {
    pub id: u64,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
    /// Avatar frame (decoration preset) asset.
    pub decoration: Option<String>,
    /// Nameplate collectible asset.
    pub nameplate: Option<String>,
    #[serde(default)]
    pub system: bool,
    #[serde(default)]
    pub bot: bool,
}

/// A user profile as shown on profile cards. Server-specific values (guild
/// member profile) already override the account's.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ProfileSnapshot {
    pub user_id: u64,
    pub guild_id: Option<u64>,
    pub username: String,
    pub global_name: Option<String>,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
    /// Banner image URL, server banner first.
    pub banner_url: Option<String>,
    pub accent_color: Option<u32>,
    /// The two Nitro profile theme colours, when set.
    pub theme_colors: Vec<u32>,
    pub bio: Option<String>,
    pub pronouns: Option<String>,
    pub decoration: Option<String>,
    pub badges: Vec<BadgeSnapshot>,
    pub mutual_guild_ids: Vec<u64>,
    pub mutual_friends_count: Option<u32>,
    /// ISO timestamp of joining the server the profile was opened from.
    pub joined_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct BadgeSnapshot {
    pub id: String,
    pub description: String,
    pub icon: String,
    pub link: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct GuildFolderSnapshot {
    /// `None` for the nameless wrapper Discord puts around a lone server.
    pub id: Option<i64>,
    pub name: Option<String>,
    pub color: Option<u32>,
    pub guild_ids: Vec<u64>,
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
    pub can_view: bool,
    pub can_send: bool,
    pub icon: Option<String>,
    pub recipients: Vec<RecipientSnapshot>,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub nsfw: bool,
    #[serde(default)]
    pub muted: bool,
    #[serde(default)]
    pub unobtrusive_unread: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ReactionSnapshot {
    pub emoji_id: Option<String>,
    pub emoji_name: String,
    pub animated: bool,
    pub count: u32,
    pub me: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct StickerSnapshot {
    pub id: u64,
    pub name: String,
    pub format_type: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct EmojiSnapshot {
    pub id: u64,
    pub name: String,
    pub animated: bool,
    pub guild_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RoleSnapshot {
    pub id: u64,
    pub name: String,
    pub color: u32,
    pub position: i32,
    pub permissions: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ThreadSnapshot {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub name: String,
    pub message_count: u32,
    pub member_count: u32,
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct BanSnapshot {
    pub user_id: u64,
    pub username: String,
    pub avatar: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct SavedAccountSnapshot {
    pub user_id: String,
    pub username: Option<String>,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct MessageSnapshot {
    pub id: u64,
    pub channel_id: u64,
    pub author_id: Option<u64>,
    pub author_name: String,
    pub author_avatar: Option<String>,
    pub content: String,
    pub timestamp: String,
    pub edited_timestamp: Option<String>,
    /// Schema 3+. Discord message type (0 default, 3 call, 6 pin, 7 join, 19 reply, ...).
    pub kind: u8,
    /// Schema 3+. The author's avatar frame asset.
    pub author_decoration: Option<String>,
    /// Schema 3+.
    pub attachments: Vec<AttachmentSnapshot>,
    /// Schema 3+. A short view of the message this one replies to.
    pub reply: Option<ReplySnapshot>,
    #[serde(default)]
    pub reactions: Vec<ReactionSnapshot>,
    #[serde(default)]
    pub stickers: Vec<StickerSnapshot>,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub embeds_json: String,
    #[serde(default)]
    pub forwarded_json: String,
    #[serde(default)]
    pub author_clan_tag: Option<String>,
    #[serde(default)]
    pub author_clan_badge: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct AttachmentSnapshot {
    pub id: u64,
    pub filename: String,
    pub url: String,
    pub proxy_url: Option<String>,
    pub content_type: Option<String>,
    pub size: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ReplySnapshot {
    pub message_id: u64,
    pub author_id: u64,
    pub author_name: String,
    pub author_avatar: Option<String>,
    pub content: String,
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
    /// Schema 3+.
    pub global_name: Option<String>,
    /// Schema 3+. Avatar frame asset (server-specific first).
    pub decoration: Option<String>,
    /// Schema 3+. Nameplate collectible asset.
    pub nameplate: Option<String>,
    #[serde(default)]
    pub clan_tag: Option<String>,
    #[serde(default)]
    pub clan_badge: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RelationshipSnapshot {
    pub user_id: u64,
    pub kind: u8,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
    pub avatar_decoration: Option<String>,
}

pub const MEMBER_LIST_ROW_PENDING: u8 = 0;
pub const MEMBER_LIST_ROW_GROUP: u8 = 1;
pub const MEMBER_LIST_ROW_MEMBER: u8 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct MemberListRowSnapshot {
    pub kind: u8,
    pub group_id: Option<String>,
    pub group_count: Option<u32>,
    pub member: Option<MemberSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct MemberListSnapshot {
    pub rows: Vec<MemberListRowSnapshot>,
    pub online_count: Option<u32>,
    pub member_count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PresenceSnapshot {
    pub user_id: u64,
    pub status: String,
    pub activities: Vec<String>,
    /// Schema 3+. Custom status text, kept out of `activities`.
    pub custom_status: Option<String>,
    /// Schema 3+. Online only from a phone.
    pub mobile: bool,
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
