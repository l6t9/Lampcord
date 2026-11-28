use crate::gateway::events;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::value::RawValue;
use std::collections;

use super::wire::*;
use crate::gateway::events::GatewayEvent;
use crate::models::id::marker::{
    ChannelMarker, GuildMarker, MessageMarker, RoleMarker, UserMarker,
};
use crate::models::{Channel, CustomEmoji, Guild, Id, Message, ReactionEmoji, Role, User};

pub struct GuildData {
    pub guild: Guild,
    pub channels: Vec<Channel>,
    pub emojis: Vec<CustomEmoji>,

    pub roles: Vec<Role>,

    pub members: Vec<MemberRoles>,

    pub joined_at_ms: Option<i64>,

    pub voice_states: Vec<super::VoiceState>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MemberRoles {
    pub user: MemberUserId,
    #[serde(default)]
    pub roles: Vec<Id<RoleMarker>>,

    #[serde(
        default,
        deserialize_with = "crate::models::member::deserialize_timeout_until"
    )]
    pub communication_disabled_until: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MemberUserId {
    pub id: Id<UserMarker>,
}

pub enum Action {
    ConnectionOpen {
        user: User,
        guilds: Vec<GuildData>,
        auth_session_id_hash: Option<String>,
        user_settings_proto: Option<String>,
        ready: ReadyExtras,
    },

    ReadySupplemental(Vec<FriendPresence>),

    UserSettingsProtoUpdate {
        proto: String,
    },

    GuildFoldersUpdate(Vec<LegacyFolderWire>),

    Resumed,
    RelationshipAdd(RelationshipWire),
    RelationshipUpdate(RelationshipWire),
    RelationshipRemove(RelationshipWire),

    PresenceUpdate(PresenceWire),
    ChannelCreate(Box<ChannelWire>),
    ChannelUpdate(Box<ChannelWire>),
    ChannelDelete(ChannelDeleteWire),
    ThreadCreate(Box<Channel>),
    ThreadUpdate(Box<Channel>),
    ThreadDelete(ChannelDeleteWire),
    ThreadMemberUpdate(ThreadMemberUpdateWire),
    ThreadMembersUpdate(ThreadMembersUpdateWire),
    GuildCreate(GuildData),

    GuildDelete(Id<GuildMarker>),

    GuildUpdate(Guild),

    GuildRoleUpsert {
        guild_id: Id<GuildMarker>,
        role: Role,
    },
    GuildRoleDelete {
        guild_id: Id<GuildMarker>,
        role_id: Id<RoleMarker>,
    },

    GuildMemberUpdate {
        guild_id: Id<GuildMarker>,
        member: MemberRoles,
    },

    UserUpdate(User),
    MessageCreate(Message),

    MessageUpdate(Message),
    MessageDelete {
        id: Id<MessageMarker>,
        channel_id: Id<ChannelMarker>,
    },

    ChannelPinsUpdate {
        channel_id: Id<ChannelMarker>,
        last_pin_timestamp: Option<String>,
    },

    MessageAck {
        channel_id: Id<ChannelMarker>,
        message_id: Id<MessageMarker>,
        mention_count: Option<u32>,
        manual: bool,
    },

    ChannelUnreadUpdate(ChannelUnreadWire),

    UserGuildSettingsUpdate(UserGuildSettingsEntry),
    NotificationSettingsUpdate(NotificationFlagsWire),
    TypingStart(TypingWire),

    GuildEmojisUpdate {
        guild_id: Id<GuildMarker>,
        emojis: Vec<CustomEmoji>,
    },

    ReactionAdd {
        channel_id: Id<ChannelMarker>,
        message_id: Id<MessageMarker>,
        user_id: Id<UserMarker>,
        emoji: ReactionEmoji,
    },
    ReactionRemove {
        channel_id: Id<ChannelMarker>,
        message_id: Id<MessageMarker>,
        user_id: Id<UserMarker>,
        emoji: ReactionEmoji,
    },

    ReactionRemoveAll {
        channel_id: Id<ChannelMarker>,
        message_id: Id<MessageMarker>,
    },

    ReactionRemoveEmoji {
        channel_id: Id<ChannelMarker>,
        message_id: Id<MessageMarker>,
        emoji: ReactionEmoji,
    },
    MemberListUpdate(Box<MemberListUpdate>),

    GuildMembersChunk(Box<MembersChunk>),

    VoiceStateUpdate(Box<super::VoiceState>),

    VoiceChannelInfo {
        channels: Vec<(
            Id<ChannelMarker>,
            super::voice_state_store::ChannelInfoPatch,
        )>,
    },

    RtcConnectionState {
        state: super::RtcState,
        guild_id: Option<Id<GuildMarker>>,
        channel_id: Option<Id<ChannelMarker>>,
    },

    RtcPing(u64),

    Speaking {
        user_id: Id<UserMarker>,
        speaking: bool,
    },

    VoiceVideo {
        user_id: Id<UserMarker>,
        active: bool,
    },
}

#[derive(Deserialize)]
struct LegacySettingsWire {
    #[serde(default)]
    guild_folders: Option<Vec<LegacyFolderWire>>,
}

#[derive(Deserialize)]
struct SettingsProtoUpdateWire {
    settings: SettingsProtoWire,
}

#[derive(Deserialize)]
struct SettingsProtoWire {
    #[serde(default)]
    proto: String,
    #[serde(rename = "type", default)]
    kind: u8,
}

#[derive(Deserialize)]
struct ReadyWire {
    user: User,

    #[serde(default, deserialize_with = "nullable")]
    guilds: Vec<Box<RawValue>>,
    #[serde(default)]
    auth_session_id_hash: Option<String>,
    #[serde(default)]
    user_settings_proto: Option<String>,
    #[serde(default)]
    user_settings: Option<LegacySettingsWire>,
    #[serde(default, deserialize_with = "lenient_vec")]
    users: Vec<User>,
    #[serde(default, deserialize_with = "lenient_vec")]
    relationships: Vec<RelationshipWire>,
    #[serde(default, deserialize_with = "lenient_vec")]
    private_channels: Vec<ChannelWire>,
    #[serde(default)]
    read_state: Option<ReadStateWire>,
    #[serde(default)]
    user_guild_settings: Option<UserGuildSettingsWire>,
    #[serde(default)]
    notification_settings: Option<NotificationFlagsWire>,
    #[serde(default, deserialize_with = "nullable")]
    merged_members: Vec<Vec<MergedMember>>,
}

#[derive(Deserialize)]
struct GuildProperties {
    #[serde(default)]
    id: Option<Id<GuildMarker>>,
    name: String,
    #[serde(default)]
    icon: Option<String>,
    #[serde(default)]
    owner_id: Option<Id<UserMarker>>,
    #[serde(default)]
    banner: Option<String>,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    premium_tier: u8,
    #[serde(default)]
    premium_subscription_count: u32,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    splash: Option<String>,
    #[serde(default)]
    discovery_splash: Option<String>,
    #[serde(default)]
    default_message_notifications: u8,
    #[serde(default)]
    vanity_url_code: Option<String>,
}

#[derive(Deserialize)]
struct GuildCreateData {
    #[serde(default)]
    id: Option<Id<GuildMarker>>,
    #[serde(default)]
    properties: Option<GuildProperties>,
    #[serde(default, deserialize_with = "lenient_vec")]
    channels: Vec<Channel>,

    #[serde(default, deserialize_with = "lenient_vec")]
    threads: Vec<Channel>,
    #[serde(default, deserialize_with = "lenient_vec")]
    emojis: Vec<CustomEmoji>,
    #[serde(default, deserialize_with = "lenient_vec")]
    roles: Vec<Role>,
    #[serde(default, deserialize_with = "lenient_vec")]
    members: Vec<MemberRoles>,
    #[serde(default)]
    joined_at: Option<String>,
    #[serde(default, deserialize_with = "lenient_vec")]
    voice_states: Vec<super::VoiceState>,
    #[serde(default)]
    unavailable: bool,
}

impl GuildData {
    fn from_wire(data: GuildCreateData) -> Option<Self> {
        if data.unavailable {
            return None;
        }
        let p = data.properties?;
        let id = data.id.or(p.id)?;
        let guild = Guild {
            id,
            name: p.name,
            icon: p.icon,
            owner_id: p.owner_id,
            banner: p.banner,
            features: p.features,
            premium_tier: p.premium_tier,
            premium_subscription_count: p.premium_subscription_count,
            description: p.description,
            splash: p.splash,
            discovery_splash: p.discovery_splash,
            default_message_notifications: p.default_message_notifications,
            vanity_url_code: p.vanity_url_code,
        };
        let joined_at_ms = data
            .joined_at
            .as_deref()
            .and_then(|time| chrono::DateTime::parse_from_rfc3339(time).ok())
            .map(|time| time.timestamp_millis());

        let mut channels = data.channels;
        channels.extend(data.threads);
        for channel in &mut channels {
            channel.guild_id.get_or_insert(id);
        }
        let mut voice_states = data.voice_states;
        for state in &mut voice_states {
            state.guild_id.get_or_insert(id);
        }
        Some(Self {
            guild,
            channels,
            emojis: data.emojis,
            roles: data.roles,
            members: data.members,
            joined_at_ms,
            voice_states,
        })
    }

    fn from_raw(raw: &RawValue) -> Option<Self> {
        match serde_json::from_str::<GuildCreateData>(raw.get()) {
            Ok(data) => Self::from_wire(data),
            Err(err) => {
                let shown: String = raw.get().chars().take(200).collect();
                log::warn!("gateway: skipped a guild that didn't parse: {err}; starts: {shown}");
                None
            }
        }
    }
}

#[derive(Deserialize)]
struct VoiceChannelInfoWire {
    #[serde(default, deserialize_with = "lenient_vec")]
    channels: Vec<VoiceChannelEntryWire>,
}

#[derive(Deserialize)]
struct VoiceChannelEntryWire {
    id: Id<ChannelMarker>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    voice_start_time: Option<i64>,
}

#[derive(Deserialize)]
struct RtcStateWire {
    state: String,
    #[serde(default)]
    guild_id: Option<Id<GuildMarker>>,
    #[serde(default)]
    channel_id: Option<Id<ChannelMarker>>,
}

#[derive(Deserialize)]
struct RtcPingWire {
    ping: u64,
}

#[derive(Deserialize)]
struct SpeakingWire {
    user_id: Id<UserMarker>,
    speaking: u8,
}

#[derive(Deserialize)]
struct VoiceVideoWire {
    user_id: Id<UserMarker>,
    active: bool,
}

#[derive(Deserialize)]
struct GuildDeleteWire {
    id: Id<GuildMarker>,
    #[serde(default)]
    unavailable: bool,
}

#[derive(Deserialize)]
struct RoleUpsertWire {
    guild_id: Id<GuildMarker>,
    role: Role,
}

#[derive(Deserialize)]
struct RoleDeleteWire {
    guild_id: Id<GuildMarker>,
    role_id: Id<RoleMarker>,
}

#[derive(Deserialize)]
struct MemberUpdateWire {
    guild_id: Id<GuildMarker>,
    user: MemberUserId,
    #[serde(default)]
    roles: Vec<Id<RoleMarker>>,
    #[serde(
        default,
        deserialize_with = "crate::models::member::deserialize_timeout_until"
    )]
    communication_disabled_until: Option<i64>,
}

#[derive(Deserialize)]
struct ChannelPinsUpdateWire {
    channel_id: Id<ChannelMarker>,
    #[serde(default)]
    last_pin_timestamp: Option<String>,
}

#[derive(Deserialize)]
struct MessageDeleteWire {
    id: Id<MessageMarker>,
    channel_id: Id<ChannelMarker>,
}

#[derive(Deserialize)]
struct MessageAckWire {
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    #[serde(default)]
    mention_count: Option<u32>,
    #[serde(default)]
    manual: bool,
}

#[derive(Deserialize)]
struct EmojisUpdateWire {
    guild_id: Id<GuildMarker>,
    emojis: Vec<CustomEmoji>,
}

#[derive(Deserialize)]
struct ReactionWire {
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    user_id: Id<UserMarker>,
    emoji: ReactionEmoji,
}

#[derive(Deserialize)]
struct ReactionRemoveAllWire {
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
}

#[derive(Deserialize)]
struct ReactionRemoveEmojiWire {
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    emoji: ReactionEmoji,
}

/// The text around byte `column` of `json`, for a parse error that only gives a position.
fn error_context(json: &str, column: usize) -> String {
    let mut start = column.saturating_sub(160).min(json.len());
    while !json.is_char_boundary(start) {
        start -= 1;
    }
    let mut end = (column + 40).min(json.len());
    while !json.is_char_boundary(end) {
        end += 1;
    }
    json[start..end].to_string()
}

fn parse<T: DeserializeOwned>(event: &GatewayEvent) -> Option<T> {
    match event.parse() {
        Ok(parsed) => Some(parsed),
        Err(err) => {
            log::warn!(
                "gateway: ignored {} that didn't parse: {err}; {}; near: {}",
                event.name,
                events::summarize(&event.name, event.json()),
                error_context(event.json(), err.column())
            );
            None
        }
    }
}

impl Action {
    pub const HANDLED_EVENTS: &'static [&'static str] = &[
        "READY",
        "READY_SUPPLEMENTAL",
        "USER_SETTINGS_PROTO_UPDATE",
        "USER_SETTINGS_UPDATE",
        "VOICE_STATE_UPDATE",
        "CHANNEL_INFO",
        "VOICE_CHANNEL_START_TIME_UPDATE",
        "VOICE_CHANNEL_STATUS_UPDATE",
        "RTC_CONNECTION_STATE",
        "RTC_CONNECTION_PING",
        "SPEAKING",
        "VOICE_VIDEO_STATE",
        "RESUMED",
        "RELATIONSHIP_ADD",
        "CHANNEL_PINS_UPDATE",
        "RELATIONSHIP_UPDATE",
        "RELATIONSHIP_REMOVE",
        "PRESENCE_UPDATE",
        "CHANNEL_CREATE",
        "CHANNEL_UPDATE",
        "CHANNEL_DELETE",
        "THREAD_CREATE",
        "THREAD_UPDATE",
        "THREAD_DELETE",
        "THREAD_MEMBER_UPDATE",
        "THREAD_MEMBERS_UPDATE",
        "GUILD_CREATE",
        "GUILD_DELETE",
        "GUILD_UPDATE",
        "GUILD_ROLE_CREATE",
        "GUILD_ROLE_UPDATE",
        "GUILD_ROLE_DELETE",
        "GUILD_MEMBER_UPDATE",
        "USER_UPDATE",
        "MESSAGE_CREATE",
        "MESSAGE_UPDATE",
        "MESSAGE_DELETE",
        "MESSAGE_ACK",
        "CHANNEL_UNREAD_UPDATE",
        "USER_GUILD_SETTINGS_UPDATE",
        "USER_NOTIFICATION_SETTINGS_UPDATE",
        "TYPING_START",
        "GUILD_EMOJIS_UPDATE",
        "MESSAGE_REACTION_ADD",
        "MESSAGE_REACTION_REMOVE",
        "MESSAGE_REACTION_REMOVE_ALL",
        "MESSAGE_REACTION_REMOVE_EMOJI",
        "GUILD_MEMBER_LIST_UPDATE",
        "GUILD_MEMBERS_CHUNK",
    ];

    pub fn from_gateway(event: &GatewayEvent) -> Option<Self> {
        match event.name.as_str() {
            "READY" => {
                let ready: ReadyWire = match event.parse() {
                    Ok(ready) => ready,
                    Err(err) => {
                        log::error!(
                            "gateway: READY didn't parse: {err}; {}",
                            events::summarize("READY", event.json())
                        );
                        return None;
                    }
                };
                let guilds: Vec<GuildData> = ready
                    .guilds
                    .iter()
                    .filter_map(|raw| GuildData::from_raw(raw))
                    .collect();
                log::info!(
                    "gateway: READY classified {} guilds, {} private channels ({} are DMs)",
                    guilds.len(),
                    ready.private_channels.len(),
                    ready
                        .private_channels
                        .iter()
                        .filter(|channel| channel.dm().is_some())
                        .count()
                );
                for guild in &guilds {
                    log::debug!(
                        "gateway: guild {} stored {} channels ({})",
                        guild.guild.id,
                        guild.channels.len(),
                        channel_kind_counts(&guild.channels)
                    );
                }
                // READY's merged_members runs parallel to `guilds` and its entries carry no guild_id.
                let mut merged_members = ready.merged_members;
                for (raw, entries) in ready.guilds.iter().zip(merged_members.iter_mut()) {
                    #[derive(Deserialize)]
                    struct IdOnly {
                        id: Id<GuildMarker>,
                    }
                    if let Ok(IdOnly { id }) = serde_json::from_str(raw.get()) {
                        for entry in entries.iter_mut().filter(|e| e.guild_id.is_none()) {
                            entry.guild_id = Some(id);
                        }
                    }
                }
                let extras = ReadyExtras {
                    legacy_folders: ready.user_settings.and_then(|s| s.guild_folders),
                    users: ready.users,
                    relationships: ready.relationships,
                    private_channels: ready.private_channels,
                    read_state: ready.read_state,
                    user_guild_settings: ready.user_guild_settings,
                    notification_flags: ready.notification_settings.map(|n| n.flags),
                    merged_members,
                };
                Some(Self::ConnectionOpen {
                    user: ready.user,
                    guilds,
                    auth_session_id_hash: ready.auth_session_id_hash,
                    user_settings_proto: ready.user_settings_proto,
                    ready: extras,
                })
            }
            "READY_SUPPLEMENTAL" => parse::<ReadySupplementalWire>(event)
                .map(|d| Self::ReadySupplemental(d.into_friends())),
            "USER_SETTINGS_UPDATE" => parse::<LegacySettingsWire>(event)
                .and_then(|d| d.guild_folders)
                .map(Self::GuildFoldersUpdate),
            "USER_SETTINGS_PROTO_UPDATE" => parse::<SettingsProtoUpdateWire>(event)
                .filter(|d| d.settings.kind == 1)
                .map(|d| Self::UserSettingsProtoUpdate {
                    proto: d.settings.proto,
                }),
            "RESUMED" => Some(Self::Resumed),
            "RELATIONSHIP_ADD" => parse(event).map(Self::RelationshipAdd),
            "RELATIONSHIP_UPDATE" => parse(event).map(Self::RelationshipUpdate),
            "RELATIONSHIP_REMOVE" => parse(event).map(Self::RelationshipRemove),
            "PRESENCE_UPDATE" => parse(event).map(Self::PresenceUpdate),
            "CHANNEL_CREATE" => parse::<ChannelWire>(event).map(|channel| {
                log_channel_classification("CHANNEL_CREATE", &channel);
                Self::ChannelCreate(Box::new(channel))
            }),
            "CHANNEL_UPDATE" => parse::<ChannelWire>(event).map(|channel| {
                log_channel_classification("CHANNEL_UPDATE", &channel);
                Self::ChannelUpdate(Box::new(channel))
            }),
            "CHANNEL_DELETE" => parse::<ChannelDeleteWire>(event).map(|channel| {
                log::debug!(
                    "gateway: CHANNEL_DELETE id={} guild={}",
                    channel.id,
                    channel
                        .guild_id
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| "<none>".into())
                );
                Self::ChannelDelete(channel)
            }),
            "THREAD_CREATE" => parse::<Channel>(event).map(|mut channel| {
                channel.newly_created = false;
                Self::ThreadCreate(Box::new(channel))
            }),
            "THREAD_UPDATE" => {
                parse::<Channel>(event).map(|channel| Self::ThreadUpdate(Box::new(channel)))
            }
            "THREAD_DELETE" => parse(event).map(Self::ThreadDelete),
            "THREAD_MEMBER_UPDATE" => parse(event).map(Self::ThreadMemberUpdate),
            "THREAD_MEMBERS_UPDATE" => parse(event).map(Self::ThreadMembersUpdate),
            "GUILD_CREATE" => match event.parse::<GuildCreateData>() {
                Ok(data) => {
                    let wire_channels = data.channels.len();
                    let without_guild = data
                        .channels
                        .iter()
                        .filter(|c| c.guild_id.is_none())
                        .count();
                    match GuildData::from_wire(data) {
                        Some(guild) => {
                            log::info!(
                                "gateway: GUILD_CREATE {} stored {} channels (wire had {wire_channels}, {without_guild} without guild_id; {})",
                                guild.guild.id,
                                guild.channels.len(),
                                channel_kind_counts(&guild.channels)
                            );
                            Some(Self::GuildCreate(guild))
                        }
                        None => {
                            log::info!(
                                "gateway: GUILD_CREATE skipped (unavailable, no properties, or no id)"
                            );
                            None
                        }
                    }
                }
                Err(err) => {
                    log::warn!(
                        "gateway: GUILD_CREATE didn't parse: {err}; {}",
                        events::summarize("GUILD_CREATE", event.json())
                    );
                    None
                }
            },

            "GUILD_DELETE" => parse::<GuildDeleteWire>(event)
                .filter(|d| !d.unavailable)
                .map(|d| Self::GuildDelete(d.id)),
            "GUILD_UPDATE" => parse::<Guild>(event).map(Self::GuildUpdate),
            "GUILD_ROLE_CREATE" | "GUILD_ROLE_UPDATE" => {
                parse::<RoleUpsertWire>(event).map(|d| Self::GuildRoleUpsert {
                    guild_id: d.guild_id,
                    role: d.role,
                })
            }
            "GUILD_ROLE_DELETE" => parse::<RoleDeleteWire>(event).map(|d| Self::GuildRoleDelete {
                guild_id: d.guild_id,
                role_id: d.role_id,
            }),
            "GUILD_MEMBER_UPDATE" => {
                parse::<MemberUpdateWire>(event).map(|d| Self::GuildMemberUpdate {
                    guild_id: d.guild_id,
                    member: MemberRoles {
                        user: d.user,
                        roles: d.roles,
                        communication_disabled_until: d.communication_disabled_until,
                    },
                })
            }
            "USER_UPDATE" => parse::<User>(event).map(Self::UserUpdate),
            "MESSAGE_CREATE" => parse::<Message>(event).map(Self::MessageCreate),
            "MESSAGE_UPDATE" => parse::<Message>(event).map(Self::MessageUpdate),

            "CHANNEL_PINS_UPDATE" => {
                parse::<ChannelPinsUpdateWire>(event).map(|d| Self::ChannelPinsUpdate {
                    channel_id: d.channel_id,
                    last_pin_timestamp: d.last_pin_timestamp,
                })
            }
            "MESSAGE_DELETE" => parse::<MessageDeleteWire>(event).map(|d| Self::MessageDelete {
                id: d.id,
                channel_id: d.channel_id,
            }),
            "MESSAGE_ACK" => parse::<MessageAckWire>(event).map(|d| Self::MessageAck {
                channel_id: d.channel_id,
                message_id: d.message_id,
                mention_count: d.mention_count,
                manual: d.manual,
            }),
            "CHANNEL_UNREAD_UPDATE" => parse(event).map(Self::ChannelUnreadUpdate),
            "USER_GUILD_SETTINGS_UPDATE" => parse(event).map(Self::UserGuildSettingsUpdate),
            "TYPING_START" => parse(event).map(Self::TypingStart),
            "USER_NOTIFICATION_SETTINGS_UPDATE" => {
                // The settings object itself, or wrapped as `settings`.
                #[derive(Deserialize)]
                struct Update {
                    #[serde(default)]
                    settings: Option<NotificationFlagsWire>,
                    #[serde(flatten)]
                    direct: NotificationFlagsWire,
                }
                parse::<Update>(event)
                    .map(|update| Self::NotificationSettingsUpdate(update.settings.unwrap_or(update.direct)))
            }
            "GUILD_EMOJIS_UPDATE" => {
                parse::<EmojisUpdateWire>(event).map(|d| Self::GuildEmojisUpdate {
                    guild_id: d.guild_id,
                    emojis: d.emojis,
                })
            }
            "MESSAGE_REACTION_ADD" => parse::<ReactionWire>(event).map(|d| Self::ReactionAdd {
                channel_id: d.channel_id,
                message_id: d.message_id,
                user_id: d.user_id,
                emoji: d.emoji,
            }),
            "MESSAGE_REACTION_REMOVE" => {
                parse::<ReactionWire>(event).map(|d| Self::ReactionRemove {
                    channel_id: d.channel_id,
                    message_id: d.message_id,
                    user_id: d.user_id,
                    emoji: d.emoji,
                })
            }
            "MESSAGE_REACTION_REMOVE_ALL" => {
                parse::<ReactionRemoveAllWire>(event).map(|d| Self::ReactionRemoveAll {
                    channel_id: d.channel_id,
                    message_id: d.message_id,
                })
            }
            "MESSAGE_REACTION_REMOVE_EMOJI" => {
                parse::<ReactionRemoveEmojiWire>(event).map(|d| Self::ReactionRemoveEmoji {
                    channel_id: d.channel_id,
                    message_id: d.message_id,
                    emoji: d.emoji,
                })
            }
            "GUILD_MEMBER_LIST_UPDATE" => parse(event).map(|u| Self::MemberListUpdate(Box::new(u))),
            "GUILD_MEMBERS_CHUNK" => parse(event).map(|c| Self::GuildMembersChunk(Box::new(c))),
            "VOICE_STATE_UPDATE" => parse(event).map(|s| Self::VoiceStateUpdate(Box::new(s))),
            "CHANNEL_INFO" => {
                parse::<VoiceChannelInfoWire>(event).map(|d| Self::VoiceChannelInfo {
                    channels: d
                        .channels
                        .into_iter()
                        .map(|c| {
                            (
                                c.id,
                                super::voice_state_store::ChannelInfoPatch {
                                    status: Some(c.status),
                                    voice_start_time: Some(c.voice_start_time),
                                },
                            )
                        })
                        .collect(),
                })
            }
            "VOICE_CHANNEL_START_TIME_UPDATE" => {
                parse::<VoiceChannelEntryWire>(event).map(|c| Self::VoiceChannelInfo {
                    channels: vec![(
                        c.id,
                        super::voice_state_store::ChannelInfoPatch {
                            status: None,
                            voice_start_time: Some(c.voice_start_time),
                        },
                    )],
                })
            }
            "VOICE_CHANNEL_STATUS_UPDATE" => {
                parse::<VoiceChannelEntryWire>(event).map(|c| Self::VoiceChannelInfo {
                    channels: vec![(
                        c.id,
                        super::voice_state_store::ChannelInfoPatch {
                            status: Some(c.status),
                            voice_start_time: None,
                        },
                    )],
                })
            }
            "RTC_CONNECTION_STATE" => {
                parse::<RtcStateWire>(event).map(|d| Self::RtcConnectionState {
                    state: super::RtcState::parse(&d.state),
                    guild_id: d.guild_id,
                    channel_id: d.channel_id,
                })
            }
            "RTC_CONNECTION_PING" => parse::<RtcPingWire>(event).map(|d| Self::RtcPing(d.ping)),
            "SPEAKING" => parse::<SpeakingWire>(event).map(|d| Self::Speaking {
                user_id: d.user_id,
                speaking: d.speaking != 0,
            }),
            "VOICE_VIDEO_STATE" => parse::<VoiceVideoWire>(event).map(|d| Self::VoiceVideo {
                user_id: d.user_id,
                active: d.active,
            }),
            _ => None,
        }
    }
}

fn channel_kind_counts(channels: &[Channel]) -> String {
    let mut counts = collections::BTreeMap::<u8, usize>::new();
    for channel in channels {
        *counts.entry(channel.kind).or_default() += 1;
    }
    if counts.is_empty() {
        return "no channels".into();
    }
    counts
        .into_iter()
        .map(|(kind, n)| format!("type {kind}: {n}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn log_channel_classification(name: &str, channel: &ChannelWire) {
    let guild = channel.guild_id.is_some();
    let dm = channel.dm().is_some();
    let bucket = match (guild, dm) {
        (true, false) => "guild",
        (false, true) => "dm",
        (true, true) => "guild-and-dm",
        (false, false) => "unclassified",
    };
    log::info!(
        "gateway: {name} id={} type={} guild={} parent={} classified={bucket}",
        channel.id,
        channel.kind,
        channel
            .guild_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "<none>".into()),
        channel
            .parent_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "<none>".into())
    );
    if bucket == "unclassified" {
        log::warn!(
            "gateway: {name} {} type {} has no guild_id and is not a DM; it will not appear as a server channel",
            channel.id,
            channel.kind
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_accepts_numeric_cosmetic_sku_ids() {
        let event = GatewayEvent::new(
            "READY",
            serde_json::json!({
                "user": {
                    "id": "1",
                    "username": "me",
                    "discriminator": "0",
                    "avatar": null,
                    "avatar_decoration_data": {
                        "asset": "a_1",
                        "expires_at": null,
                        "sku_id": 1341506443865489408u64
                    }
                },
                "guilds": [],
                "users": [{
                    "id": "2",
                    "username": "them",
                    "discriminator": "0",
                    "avatar": null,
                    "avatar_decoration_data": {
                        "asset": "a_2",
                        "expires_at": null,
                        "sku_id": 1459194821255237723u64
                    }
                }]
            }),
        );
        assert!(
            matches!(Action::from_gateway(&event), Some(Action::ConnectionOpen { .. })),
            "READY with numeric SKU ids must parse"
        );
    }

    #[test]
    fn handled_events_match_from_gateway() {
        let source = include_str!("action.rs");
        let body = &source[source.find("match event.name.as_str()").unwrap()..];
        let body = &body[..body.find("_ => None").unwrap()];
        let mut arms: Vec<&str> = body
            .split('"')
            .skip(1)
            .step_by(2)
            .filter(|s| s.chars().all(|c| c.is_ascii_uppercase() || c == '_'))
            .collect();
        arms.sort_unstable();
        arms.dedup();
        let mut listed = Action::HANDLED_EVENTS.to_vec();
        listed.sort_unstable();
        assert_eq!(arms, listed);
    }

    #[test]
    fn a_guild_channel_without_guild_id_is_not_stored_as_a_server_channel() {
        let event = GatewayEvent::new(
            "CHANNEL_CREATE",
            serde_json::json!({ "id": "41", "type": 0, "name": "general" }),
        );
        let Some(Action::ChannelCreate(channel)) = Action::from_gateway(&event) else {
            panic!("CHANNEL_CREATE should parse");
        };
        assert!(
            channel.guild_id.is_none(),
            "real discord omits guild_id on nested channels"
        );
        assert!(channel.dm().is_none(), "type 0 is not a DM");
        assert!(
            events::summarize("CHANNEL_CREATE", event.json())
                .contains("guild=<none>"),
            "the dispatch summary has to show the missing guild id"
        );
    }

    #[test]
    fn a_ready_guild_whose_id_lives_only_in_properties_is_kept() {
        let event = GatewayEvent::new(
            "READY",
            serde_json::json!({
                "user": user("1"),
                "guilds": [{
                    "properties": { "id": 20, "name": "Lazy" },
                    "channels": [{ "id": 21, "type": 0, "name": "general" }]
                }]
            }),
        );
        let Some(Action::ConnectionOpen { guilds, .. }) = Action::from_gateway(&event) else {
            panic!("READY should parse");
        };
        assert_eq!(guilds.len(), 1);
        assert_eq!(guilds[0].guild.id.get(), 20);
        assert_eq!(guilds[0].guild.name, "Lazy");
        assert_eq!(guilds[0].channels[0].guild_id.map(|id| id.get()), Some(20));
    }

    #[test]
    fn message_create_accepts_integer_snowflakes() {
        let event = GatewayEvent::new(
            "MESSAGE_CREATE",
            serde_json::json!({
                "id": 1561803622440767658u64,
                "channel_id": 30,
                "author": user("1"),
                "content": "hi",
                "timestamp": "2026-01-01T00:00:00+00:00",
                "webhook_id": 5,
                "attachments": [{ "id": 9, "filename": "a.png", "size": 1, "url": "https://cdn.example/a.png" }]
            }),
        );
        let Some(Action::MessageCreate(message)) = Action::from_gateway(&event) else {
            panic!("MESSAGE_CREATE with integer snowflakes should parse");
        };
        assert_eq!(message.id.get(), 1561803622440767658);
        assert_eq!(message.webhook_id.map(|id| id.get()), Some(5));
        assert_eq!(message.attachments[0].id.get(), 9);
    }

    fn user(id: &str) -> serde_json::Value {
        serde_json::json!({ "id": id, "username": "me", "discriminator": "0", "global_name": null, "avatar": null })
    }

    #[test]
    fn ready_parses_from_text_tolerating_nulls_stubs_and_one_bad_guild() {
        let good = serde_json::json!({ "id": "10", "properties": { "name": "Home" }, "channels": [], "joined_at": "2026-01-01T00:00:00+00:00" });
        let stub = serde_json::json!({ "id": "11", "unavailable": true });
        let bad = serde_json::json!({ "id": "12", "properties": { "name": 5 } });
        let event = GatewayEvent::new(
            "READY",
            serde_json::json!({
                "user": user("1"), "guilds": [good, stub, bad], "users": null, "relationships": [{ "id": "2", "type": 1, "user": user("2") }],
                "private_channels": null, "merged_members": [[{ "guild_id": "10", "roles": ["7"] }]],
                "read_state": { "partial": false, "entries": [{ "id": "20", "last_message_id": "30", "mention_count": 2 }] },
                "some_huge_field_nobody_reads": { "a": [1, 2, 3] },
            }),
        );
        let Some(Action::ConnectionOpen {
            user,
            guilds,
            ready,
            ..
        }) = Action::from_gateway(&event)
        else {
            panic!("READY should parse")
        };
        assert_eq!(user.id.get(), 1);
        assert_eq!(
            guilds.iter().map(|g| g.guild.id.get()).collect::<Vec<_>>(),
            [10],
            "the stub and the malformed guild are skipped alone"
        );
        assert!(guilds[0].joined_at_ms.is_some());
        assert!(
            ready.users.is_empty() && ready.private_channels.is_empty(),
            "null reads as empty"
        );
        assert_eq!(ready.relationships.len(), 1);
        assert_eq!(
            ready.merged_members[0][0].roles.as_deref().map(<[_]>::len),
            Some(1)
        );
        let entry = &ready.read_state.unwrap().entries[0];
        assert_eq!(
            (
                entry.id.unwrap().get(),
                entry.last_message_id.unwrap().get(),
                entry.mention_count
            ),
            (20, 30, 2)
        );
    }

    #[test]
    fn typed_events_parse_from_text_and_unavailable_deletes_are_ignored() {
        let delete = |unavailable: bool| {
            Action::from_gateway(&GatewayEvent::new(
                "GUILD_DELETE",
                serde_json::json!({ "id": "10", "unavailable": unavailable }),
            ))
            .is_some()
        };
        assert!(delete(false));
        assert!(!delete(true));
        let ack = GatewayEvent::new(
            "MESSAGE_ACK",
            serde_json::json!({ "channel_id": "5", "message_id": "9", "manual": true }),
        );
        assert!(matches!(
            Action::from_gateway(&ack),
            Some(Action::MessageAck {
                manual: true,
                mention_count: None,
                ..
            })
        ));

        let unfurl = GatewayEvent::new(
            "MESSAGE_UPDATE",
            serde_json::json!({ "id": "9", "channel_id": "5", "embeds": [] }),
        );
        assert!(Action::from_gateway(&unfurl).is_none());
    }

    #[test]
    fn presences_channels_relationships_and_member_lists_parse_from_text() {
        let presence = GatewayEvent::new(
            "PRESENCE_UPDATE",
            serde_json::json!({ "user": { "id": "2", "username": "x" }, "status": "idle", "activities": [{ "type": 4, "state": "hi", "emoji": { "name": "x", "id": "5" } }, { "type": "bogus" }] }),
        );
        let Some(Action::PresenceUpdate(p)) = Action::from_gateway(&presence) else {
            panic!("presence")
        };
        assert_eq!(
            (p.user.id.get(), p.status.as_deref(), p.activities.len()),
            (2, Some("idle"), 1),
            "the odd activity is dropped alone"
        );

        let dm = GatewayEvent::new(
            "CHANNEL_CREATE",
            serde_json::json!({ "id": "50", "type": 1, "recipients": [user("2")], "last_message_id": "9" }),
        );
        let Some(Action::ChannelCreate(c)) = Action::from_gateway(&dm) else {
            panic!("channel")
        };
        let dm = c.dm().expect("a DM");
        assert_eq!(
            (
                dm.recipients.iter().map(|r| r.get()).collect::<Vec<_>>(),
                dm.last_message_id.map(|m| m.get())
            ),
            (vec![2], Some(9))
        );

        let update = |value: serde_json::Value| match Action::from_gateway(&GatewayEvent::new(
            "RELATIONSHIP_UPDATE",
            value,
        )) {
            Some(Action::RelationshipUpdate(r)) => r.nickname,
            _ => panic!("relationship"),
        };
        assert_eq!(
            update(serde_json::json!({ "id": "2", "type": 1 })),
            None,
            "absent nickname: unchanged"
        );
        assert_eq!(
            update(serde_json::json!({ "id": "2", "nickname": null })),
            Some(None),
            "null nickname: cleared"
        );
        assert_eq!(
            update(serde_json::json!({ "id": "2", "nickname": "Ada" })),
            Some(Some("Ada".to_string()))
        );

        let list = GatewayEvent::new(
            "GUILD_MEMBER_LIST_UPDATE",
            serde_json::json!({ "guild_id": "10", "id": "everyone", "ops": [{ "op": "SYNC", "range": [0, 9], "items": [{ "group": { "id": "online", "count": 1 } }, { "member": { "user": 7 } }, { "member": { "user": user("3"), "presence": { "status": "online" } } }] }, { "op": "SOMETHING_NEW" }] }),
        );
        let Some(Action::MemberListUpdate(u)) = Action::from_gateway(&list) else {
            panic!("member list")
        };
        assert!(u.is_sync());
        assert_eq!(
            u.ops[0].items.len(),
            2,
            "the malformed member item is dropped alone"
        );
        assert_eq!(
            u.members().map(|m| m.user.id.get()).collect::<Vec<_>>(),
            [3]
        );
    }
}
