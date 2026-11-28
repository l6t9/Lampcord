use crate::models::channel as models_channel;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;
use std::any;
use std::fmt as std_fmt;

use crate::models::id::marker::{
    ChannelMarker, EmojiMarker, GuildMarker, MessageMarker, RoleMarker, UserMarker,
};
use crate::models::{
    Channel, GuildMember, Id, PermissionOverwrite, User,
    user::{AvatarDecorationData, Collectibles, DisplayNameStyles},
};

use super::dm_store::DmChannel;

pub fn nullable<'de, D: Deserializer<'de>, T: Deserialize<'de> + Default>(
    deserializer: D,
) -> Result<T, D::Error> {
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

/// Discord sends snowflakes as numbers on the gateway and as strings in some
/// REST payloads. An activity should never be dropped over the difference.
#[derive(Deserialize)]
#[serde(untagged)]
enum SnowflakeText {
    Text(String),
    Number(u64),
}

fn snowflake_text<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Ok(Option::<SnowflakeText>::deserialize(deserializer)?.map(|value| match value {
        SnowflakeText::Text(text) => text,
        SnowflakeText::Number(number) => number.to_string(),
    }))
}

fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(deserializer).map(Some)
}

pub struct Lenient<T>(pub Option<T>);

impl<'de, T: DeserializeOwned> Deserialize<'de> for Lenient<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = Box::<RawValue>::deserialize(deserializer)?;
        Ok(Self(match serde_json::from_str(raw.get()) {
            Ok(value) => Some(value),
            Err(err) => {
                let shown: String = raw.get().chars().take(300).collect();
                log::warn!(
                    "gateway: skipped a {} that didn't parse: {err}; json: {shown}",
                    any::type_name::<T>().rsplit("::").next().unwrap_or("item"),
                );
                None
            }
        }))
    }
}

pub fn lenient_vec<'de, D: Deserializer<'de>, T: DeserializeOwned>(
    deserializer: D,
) -> Result<Vec<T>, D::Error> {
    Ok(Option::<Vec<Lenient<T>>>::deserialize(deserializer)?
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| item.0)
        .collect())
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct Activity {
    #[serde(rename = "type", default, deserialize_with = "nullable")]
    pub kind: u8,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub emoji: Option<ActivityEmoji>,
    #[serde(default, deserialize_with = "snowflake_text")]
    pub application_id: Option<String>,
    #[serde(default)]
    pub timestamps: Option<ActivityTimestamps>,
    #[serde(default)]
    pub assets: Option<ActivityAssets>,
    #[serde(default)]
    pub party: Option<ActivityParty>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct ActivityTimestamps {
    #[serde(default, deserialize_with = "nullable")]
    pub start: u64,
    #[serde(default, deserialize_with = "nullable")]
    pub end: u64,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct ActivityAssets {
    #[serde(default)]
    pub large_image: Option<String>,
    #[serde(default)]
    pub large_text: Option<String>,
    #[serde(default)]
    pub small_image: Option<String>,
    #[serde(default)]
    pub small_text: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct ActivityParty {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub size: Option<Vec<u32>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ActivityEmoji {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub id: Option<Id<EmojiMarker>>,
    #[serde(default, deserialize_with = "nullable")]
    pub animated: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PresenceUser {
    pub id: Id<UserMarker>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PresenceWire {
    pub user: PresenceUser,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default, deserialize_with = "lenient_vec")]
    pub activities: Vec<Activity>,
}

#[derive(Clone, Debug)]
pub struct FriendPresence {
    pub presence: PresenceWire,
    pub user: Option<User>,
}

#[derive(Deserialize)]
struct FriendPresenceRaw {
    /// Older payloads carry the whole user, newer ones only the id.
    #[serde(default)]
    user: Option<Box<RawValue>>,
    #[serde(default)]
    user_id: Option<Id<UserMarker>>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default, deserialize_with = "lenient_vec")]
    activities: Vec<Activity>,
}

#[derive(Deserialize)]
struct MergedPresences {
    #[serde(default, deserialize_with = "lenient_vec")]
    friends: Vec<FriendPresenceRaw>,
    /// Presences of the members of each guild, one list per guild.
    #[serde(default, deserialize_with = "lenient_vec")]
    guilds: Vec<Vec<FriendPresenceRaw>>,
}

#[derive(Deserialize)]
pub struct ReadySupplementalWire {
    #[serde(default)]
    merged_presences: Option<MergedPresences>,
}

impl ReadySupplementalWire {
    pub fn into_friends(self) -> Vec<FriendPresence> {
        let Some(merged) = self.merged_presences else {
            return Vec::new();
        };
        // Friends first so a guild's copy of someone can't hide their friend entry's user.
        merged
            .friends
            .into_iter()
            .chain(merged.guilds.into_iter().flatten())
            .filter_map(|friend| {
                let user: Option<User> = friend
                    .user
                    .as_ref()
                    .and_then(|raw| serde_json::from_str(raw.get()).ok());
                let id = friend.user_id.or_else(|| {
                    let raw = friend.user.as_ref()?;
                    serde_json::from_str::<PresenceUser>(raw.get()).ok().map(|u| u.id)
                })?;
                Some(FriendPresence {
                    presence: PresenceWire {
                        user: PresenceUser { id },
                        status: friend.status,
                        activities: friend.activities,
                    },
                    user,
                })
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ListOpKind {
    Sync,
    Insert,
    Update,
    Delete,
    Invalidate,
    #[default]
    #[serde(other)]
    Other,
}

/// A member list section: the two presence buckets, or a hoisted role (a number on the wire).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "GroupIdWire")]
pub enum GroupId {
    Online,
    Offline,
    Role(Id<RoleMarker>),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum GroupIdWire {
    Role(Id<RoleMarker>),
    Named(String),
}

impl TryFrom<GroupIdWire> for GroupId {
    type Error = String;

    fn try_from(wire: GroupIdWire) -> Result<Self, String> {
        match wire {
            GroupIdWire::Role(id) => Ok(Self::Role(id)),
            GroupIdWire::Named(name) => match name.as_str() {
                "online" => Ok(Self::Online),
                "offline" => Ok(Self::Offline),
                other => Err(format!("unknown member list group {other:?}")),
            },
        }
    }
}

impl std_fmt::Display for GroupId {
    fn fmt(&self, f: &mut std_fmt::Formatter<'_>) -> std_fmt::Result {
        match self {
            Self::Online => f.write_str("online"),
            Self::Offline => f.write_str("offline"),
            Self::Role(id) => id.fmt(f),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GroupWire {
    pub id: GroupId,
    #[serde(default, deserialize_with = "nullable")]
    pub count: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MemberPresence {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default, deserialize_with = "lenient_vec")]
    pub activities: Vec<Activity>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MemberItem {
    pub user: User,
    #[serde(default)]
    pub nick: Option<String>,
    #[serde(default, deserialize_with = "nullable")]
    pub roles: Vec<Id<RoleMarker>>,
    #[serde(default)]
    pub presence: Option<MemberPresence>,
    #[serde(default, rename = "avatar_decoration_data")]
    pub avatar_decoration_data: Option<AvatarDecorationData>,
    #[serde(default)]
    pub collectibles: Option<Collectibles>,
    #[serde(default, rename = "display_name_styles")]
    pub display_name_styles: Option<DisplayNameStyles>,
}

impl MemberItem {
    pub fn guild_member(&self) -> GuildMember {
        GuildMember {
            user: self.user.clone(),
            nick: self.nick.clone(),
            roles: self.roles.clone(),
            avatar_decoration_data: self.avatar_decoration_data.clone(),
            collectibles: self.collectibles.clone(),
            display_name_styles: self.display_name_styles.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ListItem {
    #[serde(default)]
    pub group: Option<GroupWire>,
    #[serde(default)]
    pub member: Option<MemberItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ListOp {
    #[serde(default)]
    pub op: ListOpKind,
    #[serde(default)]
    pub index: Option<usize>,

    #[serde(default)]
    pub range: Option<Vec<usize>>,
    #[serde(default, deserialize_with = "lenient_vec")]
    pub items: Vec<ListItem>,
    #[serde(default)]
    pub item: Option<ListItem>,
}

impl ListOp {
    pub fn range_start(&self) -> usize {
        self.range
            .as_ref()
            .and_then(|range| range.first().copied())
            .unwrap_or(0)
    }

    pub fn members(&self) -> impl Iterator<Item = &MemberItem> {
        self.items
            .iter()
            .chain(self.item.iter())
            .filter_map(|item| item.member.as_ref())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MembersChunk {
    pub guild_id: Id<GuildMarker>,
    #[serde(default, deserialize_with = "lenient_vec")]
    pub members: Vec<MemberItem>,
    #[serde(default, deserialize_with = "nullable")]
    pub not_found: Vec<Id<UserMarker>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MemberListUpdate {
    pub guild_id: Id<GuildMarker>,

    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub member_count: Option<u64>,
    #[serde(default)]
    pub online_count: Option<u64>,
    #[serde(default, deserialize_with = "lenient_vec")]
    pub ops: Vec<ListOp>,

    #[serde(default, deserialize_with = "lenient_vec")]
    pub groups: Vec<GroupWire>,
}

impl MemberListUpdate {
    pub fn is_sync(&self) -> bool {
        self.ops.iter().any(|op| op.op == ListOpKind::Sync)
    }

    pub fn members(&self) -> impl Iterator<Item = &MemberItem> {
        self.ops.iter().flat_map(ListOp::members)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChannelWire {
    pub id: Id<ChannelMarker>,
    #[serde(default)]
    pub guild_id: Option<Id<GuildMarker>>,
    #[serde(rename = "type")]
    pub kind: u8,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub position: Option<i32>,
    #[serde(default)]
    pub parent_id: Option<Id<ChannelMarker>>,

    #[serde(default)]
    pub permission_overwrites: Option<Vec<PermissionOverwrite>>,
    #[serde(default, deserialize_with = "nullable")]
    pub flags: u64,
    #[serde(default, deserialize_with = "nullable")]
    pub nsfw: bool,
    #[serde(default)]
    pub last_message_id: Option<Id<MessageMarker>>,
    #[serde(default)]
    pub owner_id: Option<Id<UserMarker>>,
    #[serde(default)]
    pub thread_metadata: Option<models_channel::ThreadMetadata>,
    #[serde(default, deserialize_with = "nullable")]
    pub message_count: u32,
    #[serde(default, deserialize_with = "nullable")]
    pub member_count: u32,
    #[serde(default, deserialize_with = "nullable")]
    pub total_message_sent: u32,
    #[serde(default)]
    pub member: Option<models_channel::ThreadMember>,
    #[serde(default, deserialize_with = "nullable")]
    pub newly_created: bool,
    #[serde(default)]
    pub recipient_ids: Option<Vec<Id<UserMarker>>>,
    #[serde(default, deserialize_with = "lenient_vec")]
    pub recipients: Vec<User>,
}

impl ChannelWire {
    pub fn channel(&self) -> Channel {
        Channel {
            id: self.id,
            guild_id: self.guild_id,
            kind: self.kind,
            name: self.name.clone(),
            position: self.position,
            parent_id: self.parent_id,
            permission_overwrites: self.permission_overwrites.clone().unwrap_or_default(),
            flags: self.flags,
            nsfw: self.nsfw,
            last_message_id: self.last_message_id,
            owner_id: self.owner_id,
            thread_metadata: self.thread_metadata.clone(),
            message_count: self.message_count,
            member_count: self.member_count,
            total_message_sent: self.total_message_sent,
            member: self.member.clone(),
            newly_created: self.newly_created,
        }
    }

    pub fn dm(&self) -> Option<DmChannel> {
        if !matches!(self.kind, 1 | 3) {
            return None;
        }
        let recipients = match &self.recipient_ids {
            Some(ids) => ids.clone(),
            None => self.recipients.iter().map(|user| user.id).collect(),
        };
        Some(DmChannel {
            id: self.id,
            kind: self.kind,
            name: self.name.clone(),
            recipients,
            last_message_id: self.last_message_id,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChannelDeleteWire {
    pub id: Id<ChannelMarker>,
    #[serde(default)]
    pub guild_id: Option<Id<GuildMarker>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ThreadMemberUpdateWire {
    pub guild_id: Id<GuildMarker>,
    #[serde(flatten)]
    pub member: models_channel::ThreadMember,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ThreadMembersUpdateWire {
    pub id: Id<ChannelMarker>,
    pub guild_id: Id<GuildMarker>,
    #[serde(default)]
    pub member_count: Option<u32>,
    #[serde(default, deserialize_with = "lenient_vec")]
    pub added_members: Vec<models_channel::ThreadMember>,
    #[serde(default)]
    pub removed_member_ids: Vec<Id<UserMarker>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChannelUnreadWire {
    #[serde(default, deserialize_with = "lenient_vec")]
    pub channel_unread_updates: Vec<ChannelUnread>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChannelUnread {
    pub id: Id<ChannelMarker>,
    pub last_message_id: Id<MessageMarker>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RelationshipWire {
    #[serde(default)]
    pub id: Option<Id<UserMarker>>,
    #[serde(rename = "type", default)]
    pub kind: Option<u8>,

    #[serde(default, deserialize_with = "present")]
    pub nickname: Option<Option<String>>,
    #[serde(default)]
    pub user: Option<User>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct MuteConfigWire {
    #[serde(default)]
    pub end_time: Option<String>,

    #[serde(default, deserialize_with = "lenient_seconds")]
    pub selected_time_window: Option<i64>,
}

fn lenient_seconds<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<i64>, D::Error> {
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(value.and_then(|v| {
        v.as_i64()
            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
    }))
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChannelOverrideWire {
    pub channel_id: Id<ChannelMarker>,
    #[serde(default, deserialize_with = "nullable")]
    pub muted: bool,
    #[serde(default)]
    pub mute_config: Option<MuteConfigWire>,
    #[serde(default)]
    pub message_notifications: Option<u8>,
    #[serde(default, deserialize_with = "nullable")]
    pub flags: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UserGuildSettingsEntry {
    #[serde(default)]
    pub guild_id: Option<Id<GuildMarker>>,
    #[serde(default, deserialize_with = "nullable")]
    pub muted: bool,
    #[serde(default)]
    pub mute_config: Option<MuteConfigWire>,
    #[serde(default)]
    pub message_notifications: Option<u8>,
    #[serde(default, deserialize_with = "nullable")]
    pub suppress_everyone: bool,
    #[serde(default, deserialize_with = "nullable")]
    pub suppress_roles: bool,
    #[serde(default, deserialize_with = "nullable")]
    pub hide_muted_channels: bool,
    #[serde(default, deserialize_with = "lenient_vec")]
    pub channel_overrides: Vec<ChannelOverrideWire>,

    #[serde(default)]
    pub mobile_push: Option<bool>,
    #[serde(default, deserialize_with = "nullable")]
    pub mute_scheduled_events: bool,

    #[serde(default, deserialize_with = "nullable")]
    pub notify_highlights: u8,
    #[serde(default, deserialize_with = "nullable")]
    pub flags: u64,

    #[serde(default, deserialize_with = "nullable")]
    pub version: u64,
}

#[derive(Default, Deserialize)]
pub struct ReadStateWire {
    #[serde(default, deserialize_with = "nullable")]
    pub partial: bool,
    #[serde(default, deserialize_with = "lenient_vec")]
    pub entries: Vec<ReadStateEntry>,
}

#[derive(Deserialize)]
pub struct ReadStateEntry {
    pub id: Option<Id<ChannelMarker>>,

    #[serde(default, deserialize_with = "nullable")]
    pub read_state_type: u64,
    pub last_message_id: Option<Id<MessageMarker>>,
    #[serde(default, deserialize_with = "nullable")]
    pub mention_count: u64,

    #[serde(default, deserialize_with = "nullable")]
    pub flags: u64,
}

#[derive(Default, Deserialize)]
pub struct UserGuildSettingsWire {
    #[serde(default, deserialize_with = "lenient_vec")]
    pub entries: Vec<UserGuildSettingsEntry>,
}

#[derive(Deserialize)]
pub struct MergedMember {
    pub guild_id: Option<Id<GuildMarker>>,
    pub roles: Option<Vec<Id<RoleMarker>>>,

    #[serde(
        default,
        deserialize_with = "crate::models::member::deserialize_timeout_until"
    )]
    pub communication_disabled_until: Option<i64>,
}

/// A folder from the older JSON user settings (`guild_folders`). A null `id` with a single
/// server is only that server's place in the list.
#[derive(Debug, Clone, Deserialize)]
pub struct LegacyFolderWire {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub color: Option<u64>,
    #[serde(default)]
    pub guild_ids: Vec<Id<GuildMarker>>,
}

/// `TYPING_START`: someone began typing in a channel.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct TypingWire {
    pub channel_id: Id<ChannelMarker>,
    pub user_id: Id<UserMarker>,
}

/// The user-wide notification settings: READY's `notification_settings` and the update event.
/// Its `USE_NEW_NOTIFICATIONS` flag decides whether unreads follow the notification level.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub struct NotificationFlagsWire {
    #[serde(default, deserialize_with = "nullable")]
    pub flags: u64,
}

#[derive(Default)]
pub struct ReadyExtras {
    pub legacy_folders: Option<Vec<LegacyFolderWire>>,
    pub users: Vec<User>,
    pub relationships: Vec<RelationshipWire>,
    pub private_channels: Vec<ChannelWire>,
    pub read_state: Option<ReadStateWire>,
    pub user_guild_settings: Option<UserGuildSettingsWire>,
    pub notification_flags: Option<u64>,

    pub merged_members: Vec<Vec<MergedMember>>,
}

#[cfg(test)]
mod tests {
    use super::Activity;

    #[test]
    fn rich_presence_keeps_the_fields_an_activity_card_draws() {
        let activity: Activity = serde_json::from_value(serde_json::json!({
            "type": 2,
            "name": "Spotify",
            "details": "Song",
            "state": "Artist",
            "application_id": "1234567890",
            "timestamps": { "start": 1_700_000_000_000u64, "end": null },
            "assets": {
                "large_image": "spotify:ab:cover",
                "large_text": "Album",
                "small_image": "small",
                "small_text": "Small"
            },
            "party": { "id": "party", "size": [1, 4] },
            "emoji": null
        }))
        .unwrap();

        assert_eq!(activity.kind, 2);
        assert_eq!(activity.name.as_deref(), Some("Spotify"));
        assert_eq!(activity.details.as_deref(), Some("Song"));
        assert_eq!(activity.state.as_deref(), Some("Artist"));
        assert_eq!(activity.application_id.as_deref(), Some("1234567890"));
        assert_eq!(activity.timestamps.unwrap().start, 1_700_000_000_000);
        let assets = activity.assets.unwrap();
        assert_eq!(assets.large_image.as_deref(), Some("spotify:ab:cover"));
        assert_eq!(assets.small_image.as_deref(), Some("small"));
        assert_eq!(activity.party.unwrap().size, Some(vec![1, 4]));
    }

    #[test]
    fn a_numeric_activity_application_id_is_kept_as_a_snowflake() {
        let activity: Activity = serde_json::from_value(serde_json::json!({
            "type": 0,
            "name": "Your Only Move Is HUSTLE",
            "application_id": 1_080_735_574_593_380_382u64,
            "assets": { "large_image": "1080814736456826932" }
        }))
        .unwrap();

        assert_eq!(
            activity.application_id.as_deref(),
            Some("1080735574593380382")
        );
        assert_eq!(
            activity.assets.unwrap().large_image.as_deref(),
            Some("1080814736456826932")
        );
    }
}
