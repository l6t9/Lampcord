use std::collections::BTreeMap;

use crate::id::{
    ApplicationId, AttachmentId, ChannelId, GuildId, MessageId, RoleId, UserId, WebhookId,
};

pub type Timestamp = i64;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UserData {
    pub id: UserId,
    pub username: String,
    pub global_name: Option<String>,

    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub accent_color: Option<u32>,
    pub bot: bool,
    pub system: bool,
    pub badges: Vec<String>,
}

impl UserData {
    pub fn display_name(&self) -> String {
        self.global_name
            .clone()
            .unwrap_or_else(|| self.username.clone())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MemberData {
    pub guild_id: GuildId,
    pub user: UserData,
    pub nick: Option<String>,
    pub role_ids: Vec<RoleId>,
    pub color: Option<u32>,
    pub joined_at: Option<Timestamp>,
    pub timeout_until: Option<Timestamp>,
    pub guild_avatar_url: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RoleData {
    pub id: RoleId,
    pub guild_id: GuildId,
    pub name: String,
    pub color: Option<u32>,
    pub position: i32,
    pub hoist: bool,
    pub mentionable: bool,
    pub icon_url: Option<String>,
    pub permissions: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GuildData {
    pub id: GuildId,
    pub name: String,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub owner_id: UserId,
    pub member_count: Option<u32>,
    pub online_count: Option<u32>,
    pub features: Vec<String>,
    pub description: Option<String>,
    pub boost_tier: u32,
    pub boost_count: u32,

    pub splash_url: Option<String>,
    pub discovery_splash_url: Option<String>,

    pub emojis: Vec<(String, String)>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChannelData {
    pub id: ChannelId,
    pub guild_id: Option<GuildId>,
    pub name: Option<String>,
    pub kind: String,
    pub parent_id: Option<ChannelId>,
    pub position: Option<i32>,
    pub topic: Option<String>,
    pub nsfw: bool,
    pub recipient_ids: Vec<UserId>,
    pub last_message_id: Option<MessageId>,

    pub my_permissions: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AttachmentData {
    pub id: AttachmentId,
    pub filename: String,
    pub content_type: Option<String>,
    pub size: u64,
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub description: Option<String>,
    pub spoiler: bool,
    pub duration_secs: Option<f32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EmbedData {
    pub kind: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub color: Option<u32>,
    pub author_name: Option<String>,
    pub provider_name: Option<String>,
    pub footer_text: Option<String>,
    pub image_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub video_url: Option<String>,
    pub fields: Vec<(String, String, bool)>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReactionData {
    pub emoji: String,
    pub count: u32,
    pub me: bool,
    pub burst: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MessageData {
    pub id: MessageId,
    pub channel_id: ChannelId,
    pub guild_id: Option<GuildId>,
    pub author: UserData,
    pub member: Option<MemberData>,
    pub content: String,
    pub timestamp: Timestamp,
    pub edited_timestamp: Option<Timestamp>,
    pub kind: String,
    pub flags: Vec<String>,
    pub attachments: Vec<AttachmentData>,
    pub embeds: Vec<EmbedData>,
    pub reactions: Vec<ReactionData>,
    pub stickers: Vec<String>,
    pub mention_ids: Vec<UserId>,
    pub mention_role_ids: Vec<RoleId>,
    pub mentions_everyone: bool,
    pub mentions_me: bool,
    pub reply_to: Option<MessageId>,
    pub referenced: Option<Box<MessageData>>,
    pub pinned: bool,
    pub webhook_id: Option<WebhookId>,

    pub raw_json: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActivityData {
    pub kind: String,
    pub name: String,
    pub details: Option<String>,
    pub state: Option<String>,
    pub application_id: Option<ApplicationId>,
    pub url: Option<String>,
    pub start: Option<Timestamp>,
    pub end: Option<Timestamp>,
    pub large_image: Option<String>,
    pub large_text: Option<String>,
    pub small_image: Option<String>,
    pub small_text: Option<String>,
    pub emoji: Option<String>,
    pub buttons: Vec<(String, String)>,
    pub party_id: Option<String>,
    pub party_size: Option<(u32, u32)>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PresenceData {
    pub user_id: UserId,
    pub status: String,
    pub platforms: Vec<String>,
    pub activities: Vec<ActivityData>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VoiceStateData {
    pub user_id: UserId,
    pub channel_id: Option<ChannelId>,
    pub guild_id: Option<GuildId>,
    pub self_mute: bool,
    pub self_deaf: bool,
    pub server_mute: bool,
    pub server_deaf: bool,
    pub streaming: bool,
    pub video: bool,
    pub suppressed: bool,
    pub joined_at: Option<Timestamp>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RelationshipData {
    pub user: UserData,
    pub kind: String,
    pub nickname: Option<String>,
    pub since: Option<Timestamp>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReadStateData {
    pub channel_id: ChannelId,
    pub unread: bool,
    pub mention_count: u32,
    pub last_read_message_id: Option<MessageId>,
    pub notification_level: String,
    pub muted: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProfileData {
    pub user: UserData,
    pub bio: Option<String>,
    pub pronouns: Option<String>,
    pub connections: Vec<(String, String, bool)>,
    pub member: Option<MemberData>,
    pub mutual_guild_ids: Vec<GuildId>,
    pub mutual_friend_ids: Vec<UserId>,
    pub premium_since: Option<Timestamp>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OutgoingFile {
    pub filename: String,
    pub content_type: Option<String>,
    pub size: u64,

    pub data: Option<Vec<u8>>,
    pub spoiler: bool,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OutgoingDraft {
    pub channel_id: ChannelId,
    pub guild_id: Option<GuildId>,
    pub content: String,
    pub reply_to: Option<MessageId>,
    pub reply_mention: bool,
    pub flags: Vec<String>,
    pub stickers: Vec<String>,
    pub files: Vec<OutgoingFile>,

    pub sent_by_plugin: Option<String>,

    pub split: Option<Vec<String>>,

    pub edit_instead: Option<MessageId>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditDraft {
    pub channel_id: ChannelId,
    pub guild_id: Option<GuildId>,
    pub message_id: MessageId,
    pub original_content: String,
    pub content: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IncomingSource {
    GatewayCreate,
    GatewayUpdate,
    GatewayDelete,
    History,
    Search,
    Jump,
}

impl IncomingSource {
    pub fn as_str(self) -> &'static str {
        match self {
            IncomingSource::GatewayCreate => "gateway-create",
            IncomingSource::GatewayUpdate => "gateway-update",
            IncomingSource::GatewayDelete => "gateway-delete",
            IncomingSource::History => "history",
            IncomingSource::Search => "search",
            IncomingSource::Jump => "jump",
        }
    }
}

#[derive(Clone, Debug)]
pub struct IncomingItem {
    pub message: MessageData,

    pub previous: Option<MessageData>,
    pub source: IncomingSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ByPlugin<T> {
    pub plugin: String,
    pub value: T,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
    pub kind: String,

    pub options: String,
}

#[derive(Clone, Debug)]
pub struct OutgoingOutcome<D> {
    pub draft: D,

    pub cancelled: Option<ByPlugin<String>>,

    pub modified_by: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IncomingOutcome {
    pub hidden: Option<ByPlugin<String>>,

    pub kept_deleted: Option<ByPlugin<String>>,
    pub display_content: Option<ByPlugin<String>>,
    pub annotations: Vec<ByPlugin<String>>,
    pub spans: Vec<ByPlugin<Span>>,

    pub decorations: Vec<ByPlugin<(String, String)>>,

    pub embed_states: Vec<ByPlugin<(u32, String)>>,
    pub attachment_states: Vec<ByPlugin<(u32, String)>>,

    pub author_display: Vec<ByPlugin<(String, Option<String>)>>,
}

impl IncomingOutcome {
    pub fn is_unchanged(&self) -> bool {
        *self == IncomingOutcome::default()
    }

    pub fn retain_plugins(&mut self, keep: impl Fn(&str) -> bool) {
        if self.hidden.as_ref().is_some_and(|h| !keep(&h.plugin)) {
            self.hidden = None;
        }
        if self.kept_deleted.as_ref().is_some_and(|h| !keep(&h.plugin)) {
            self.kept_deleted = None;
        }
        if self
            .display_content
            .as_ref()
            .is_some_and(|d| !keep(&d.plugin))
        {
            self.display_content = None;
        }
        self.annotations.retain(|a| keep(&a.plugin));
        self.spans.retain(|a| keep(&a.plugin));
        self.decorations.retain(|a| keep(&a.plugin));
        self.embed_states.retain(|a| keep(&a.plugin));
        self.attachment_states.retain(|a| keep(&a.plugin));
        self.author_display.retain(|a| keep(&a.plugin));
    }

    pub fn plugins(&self) -> impl Iterator<Item = &str> {
        self.hidden
            .iter()
            .map(|h| h.plugin.as_str())
            .chain(self.kept_deleted.iter().map(|h| h.plugin.as_str()))
            .chain(self.display_content.iter().map(|d| d.plugin.as_str()))
            .chain(self.annotations.iter().map(|a| a.plugin.as_str()))
            .chain(self.spans.iter().map(|a| a.plugin.as_str()))
            .chain(self.decorations.iter().map(|a| a.plugin.as_str()))
            .chain(self.embed_states.iter().map(|a| a.plugin.as_str()))
            .chain(self.attachment_states.iter().map(|a| a.plugin.as_str()))
            .chain(self.author_display.iter().map(|a| a.plugin.as_str()))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NotificationDraft {
    pub kind: String,
    pub channel_id: Option<ChannelId>,
    pub guild_id: Option<GuildId>,
    pub message: Option<MessageData>,
    pub title: String,
    pub body: String,
    pub silent: bool,
    pub bypass_dnd: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PresenceDraft {
    pub status: String,
    pub activities: Vec<ActivityData>,
}

#[derive(Clone, Debug, Default)]
pub struct EventData {
    pub kind: String,
    pub channel_id: Option<ChannelId>,
    pub guild_id: Option<GuildId>,
    pub user_id: Option<UserId>,
    pub message_id: Option<MessageId>,
    pub message: Option<MessageData>,
    pub previous_message: Option<MessageData>,
    pub presence: Option<PresenceData>,
    pub voice_state: Option<VoiceStateData>,

    pub data: String,
}

impl EventData {
    pub fn new(kind: impl Into<String>) -> Self {
        EventData {
            kind: kind.into(),
            data: "{}".to_string(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OverlayUpdate {
    DisplayContent {
        channel_id: ChannelId,
        message_id: MessageId,
        content: Option<String>,
    },
    Annotation {
        channel_id: ChannelId,
        message_id: MessageId,
        text: Option<String>,
    },
    Decoration {
        channel_id: ChannelId,
        message_id: MessageId,
        slot: String,
        view_id: Option<String>,
    },
    Span {
        channel_id: ChannelId,
        message_id: MessageId,
        span: Span,
    },
    Hidden {
        channel_id: ChannelId,
        message_id: MessageId,
        hidden: bool,
    },
    Clear {
        channel_id: ChannelId,
        message_id: MessageId,
    },

    Display {
        target: &'static str,
        id: u64,
        scope: Option<u64>,
        field: String,
        value: Option<String>,
    },
    Decorate {
        target: &'static str,
        id: u64,
        slot: String,
        view_id: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug, Default)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    pub timeout_ms: u32,
}

#[derive(Clone, Debug, Default)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct SendOptions {
    pub reply_to: Option<u64>,
    pub reply_mention: bool,
    pub flags: Vec<String>,
    pub sticker_ids: Vec<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SlotItemData {
    pub slot: String,
    pub id: String,
    pub icon: Option<String>,
    pub label: String,
    pub active: bool,
    pub disabled: bool,
    pub badge: u32,
    pub danger: bool,
    pub visible_in: Vec<String>,
    pub hidden: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Registrations {
    pub events: Vec<String>,
    pub commands: Vec<CommandData>,
    pub menu_items: Vec<MenuItemData>,
    pub shortcuts: Vec<(String, String, String)>,
    pub autocomplete_triggers: Vec<String>,

    pub search_providers: Vec<(String, Option<String>, String)>,

    pub markdown_rules: Vec<(String, String, bool)>,
    pub audio_streams: Vec<String>,
    pub slot_items: BTreeMap<(String, String), SlotItemData>,

    pub slot_contents: BTreeMap<(String, String), (Vec<String>, String)>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CommandData {
    pub name: String,
    pub description: String,
    pub options: Vec<CommandOption>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CommandOption {
    pub name: String,
    pub description: String,
    pub kind: String,
    pub required: bool,

    pub choices: Vec<(String, String)>,
    pub autocomplete: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MenuItemData {
    pub menu: String,
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub danger: bool,
    pub checkable: bool,
    pub parent: Option<String>,
    pub dynamic: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SuggestionData {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub insert_text: Option<String>,
    pub icon: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MenuStateData {
    pub id: String,
    pub label: Option<String>,
    pub checked: Option<bool>,
    pub disabled: bool,
}
