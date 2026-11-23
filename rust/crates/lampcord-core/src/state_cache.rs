use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use lampcord_model::{
    Snowflake,
    channel::{Channel, ChannelType},
    guild::Guild,
    message::Message,
    user::User,
};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{event_dispatch::GatewayEventHandler, read_state::ReadStateStore};

pub const MAX_MESSAGES_PER_CHANNEL: usize = 150;
const MAX_CACHED_MESSAGE_CHANNELS: usize = 40;
const MAX_CACHED_USERS: usize = 2000;

const HANDLED_EVENTS: &[&str] = &[
    "READY",
    "MESSAGE_CREATE",
    "MESSAGE_UPDATE",
    "MESSAGE_DELETE",
    "GUILD_CREATE",
    "GUILD_UPDATE",
    "GUILD_DELETE",
    "CHANNEL_CREATE",
    "CHANNEL_UPDATE",
    "CHANNEL_DELETE",
    "USER_UPDATE",
    "GUILD_MEMBER_ADD",
    "GUILD_MEMBER_UPDATE",
    "GUILD_MEMBER_REMOVE",
    "GUILD_MEMBERS_CHUNK",
    "PRESENCE_UPDATE",
    "USER_SETTINGS_UPDATE",
];

/// A server folder from the user's settings, in rail order. Discord also
/// stores lone servers as nameless folders without an id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuildFolder {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub color: Option<u32>,
    pub guild_ids: Vec<Snowflake>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedMember {
    pub guild_id: Snowflake,
    pub user_id: Snowflake,
    pub username: String,
    pub avatar: Option<String>,
    pub nickname: Option<String>,
    pub roles: Vec<Snowflake>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedPresence {
    pub user_id: Snowflake,
    pub status: String,
    /// Names of everything but the custom status (type 4).
    pub activities: Vec<String>,
    /// The custom status text, when one is set.
    pub custom_status: Option<String>,
}

/// Parses a presence from PRESENCE_UPDATE, READY `presences` or
/// `merged_presences` (which carry `user_id` instead of a `user` object).
pub fn presence_from_value(value: &Value) -> Option<CachedPresence> {
    let user_id = snowflake(
        value
            .get("user")
            .and_then(|user| user.get("id"))
            .or_else(|| value.get("user_id")),
    )?;
    let status = value
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("offline")
        .to_owned();
    let activities: Vec<&Value> = value
        .get("activities")
        .and_then(Value::as_array)
        .map(|activities| activities.iter().collect())
        .unwrap_or_default();
    let is_custom = |activity: &Value| activity.get("type").and_then(Value::as_u64) == Some(4);
    let custom_status = activities
        .iter()
        .find(|activity| is_custom(activity))
        .and_then(|activity| activity.get("state").and_then(Value::as_str))
        .filter(|state| !state.is_empty())
        .map(str::to_owned);
    let activities = activities
        .iter()
        .filter(|activity| !is_custom(activity))
        .filter_map(|activity| {
            activity
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .collect();
    Some(CachedPresence {
        user_id,
        status,
        activities,
        custom_status,
    })
}

#[derive(Debug)]
pub enum CacheError {
    MissingData(&'static str),
    InvalidPayload(serde_json::Error),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingData(field) => write!(formatter, "missing {field} in gateway payload"),
            Self::InvalidPayload(error) => write!(formatter, "invalid cache payload: {error}"),
        }
    }
}

impl std::error::Error for CacheError {}

impl From<serde_json::Error> for CacheError {
    fn from(error: serde_json::Error) -> Self {
        Self::InvalidPayload(error)
    }
}

#[derive(Debug, Clone, Default)]
pub struct StateCache {
    token: Option<String>,
    current_user: Option<User>,
    users: HashMap<Snowflake, User>,
    guilds: HashMap<Snowflake, Guild>,
    guild_order: Vec<Snowflake>,
    guild_folders: Vec<GuildFolder>,
    own_status: Option<String>,
    own_custom_status: Option<String>,
    channels: HashMap<Snowflake, Channel>,
    guild_channels: HashMap<Snowflake, Vec<Snowflake>>,
    private_channel_ids: Vec<Snowflake>,
    messages: HashMap<Snowflake, Vec<Message>>,
    message_channel_order: VecDeque<Snowflake>,
    members: HashMap<Snowflake, HashMap<Snowflake, CachedMember>>,
    presences: HashMap<Snowflake, CachedPresence>,
    read_state: ReadStateStore,
}

impl StateCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn switch_token(&mut self, token: &str) {
        if self.token.as_deref() == Some(token) {
            return;
        }
        self.clear_data();
        self.token = Some(token.to_owned());
    }

    pub fn clear(&mut self) {
        self.clear_data();
        self.token = None;
    }

    pub fn current_user(&self) -> Option<&User> {
        self.current_user.as_ref()
    }

    pub fn user(&self, user_id: Snowflake) -> Option<&User> {
        self.users.get(&user_id)
    }

    pub fn ordered_guilds(&self) -> Vec<&Guild> {
        self.guild_order
            .iter()
            .filter_map(|guild_id| self.guilds.get(guild_id))
            .collect()
    }

    pub fn guild_folders(&self) -> &[GuildFolder] {
        &self.guild_folders
    }

    /// The status picked in settings (online, idle, dnd, invisible).
    pub fn own_status(&self) -> Option<&str> {
        self.own_status.as_deref()
    }

    pub fn own_custom_status(&self) -> Option<&str> {
        self.own_custom_status.as_deref()
    }

    /// Mirrors a status change made through the REST settings endpoint
    /// before Discord echoes it back.
    pub fn set_own_status(&mut self, status: &str) {
        self.own_status = Some(status.to_owned());
    }

    pub fn guild(&self, guild_id: Snowflake) -> Option<&Guild> {
        self.guilds.get(&guild_id)
    }

    pub fn channel(&self, channel_id: Snowflake) -> Option<&Channel> {
        self.channels.get(&channel_id)
    }

    pub fn guild_count(&self) -> usize {
        self.guilds.len()
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn cached_message_count(&self) -> usize {
        self.messages.values().map(Vec::len).sum()
    }

    pub fn channels(&self) -> impl Iterator<Item = &Channel> {
        self.channels.values()
    }

    pub fn channels_for_guild(&self, guild_id: Snowflake) -> Vec<&Channel> {
        let mut channels: Vec<&Channel> = self
            .guild_channels
            .get(&guild_id)
            .map(|channel_ids| {
                channel_ids
                    .iter()
                    .filter_map(|channel_id| self.channels.get(channel_id))
                    .collect()
            })
            .unwrap_or_default();
        channels.sort_by(|left, right| {
            left.position
                .unwrap_or(u16::MAX)
                .cmp(&right.position.unwrap_or(u16::MAX))
                .then_with(|| left.name.cmp(&right.name))
                .then_with(|| left.id.cmp(&right.id))
        });
        channels
    }

    pub fn private_channels(&self) -> Vec<&Channel> {
        let mut channels: Vec<&Channel> = self
            .private_channel_ids
            .iter()
            .filter_map(|channel_id| self.channels.get(channel_id))
            .collect();
        channels.sort_by(|left, right| {
            right
                .last_message_id
                .cmp(&left.last_message_id)
                .then_with(|| left.id.cmp(&right.id))
        });
        channels
    }

    pub fn messages(&self, channel_id: Snowflake) -> &[Message] {
        self.messages
            .get(&channel_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn member(&self, guild_id: Snowflake, user_id: Snowflake) -> Option<&CachedMember> {
        self.members.get(&guild_id)?.get(&user_id)
    }

    pub fn members_for_guild(&self, guild_id: Snowflake) -> Vec<&CachedMember> {
        let mut members: Vec<&CachedMember> = self
            .members
            .get(&guild_id)
            .map(|guild| guild.values().collect())
            .unwrap_or_default();
        members.sort_by(|left, right| {
            left.username
                .cmp(&right.username)
                .then_with(|| left.user_id.cmp(&right.user_id))
        });
        members
    }

    pub fn presence(&self, user_id: Snowflake) -> Option<&CachedPresence> {
        self.presences.get(&user_id)
    }

    pub fn presences(&self) -> Vec<&CachedPresence> {
        let mut presences: Vec<&CachedPresence> = self.presences.values().collect();
        presences.sort_by_key(|presence| presence.user_id);
        presences
    }

    pub fn read_state(&self) -> &ReadStateStore {
        &self.read_state
    }

    pub fn read_state_mut(&mut self) -> &mut ReadStateStore {
        &mut self.read_state
    }

    pub fn handle_event(
        &mut self,
        event_type: &str,
        data: Option<&Value>,
    ) -> Result<(), CacheError> {
        match event_type {
            "READY" => self.handle_ready(data),
            "MESSAGE_CREATE" => self.handle_message_create(data),
            "MESSAGE_UPDATE" => self.handle_message_update(data),
            "MESSAGE_DELETE" => self.handle_message_delete(data),
            "GUILD_CREATE" => self.handle_guild_create(data),
            "GUILD_UPDATE" => self.handle_guild_update(data),
            "GUILD_DELETE" => self.handle_guild_delete(data),
            "CHANNEL_CREATE" | "CHANNEL_UPDATE" => self.handle_channel(data),
            "CHANNEL_DELETE" => self.handle_channel_delete(data),
            "USER_UPDATE" => self.handle_user_update(data),
            "GUILD_MEMBER_ADD" | "GUILD_MEMBER_UPDATE" => self.handle_guild_member(data),
            "GUILD_MEMBER_REMOVE" => self.handle_guild_member_remove(data),
            "GUILD_MEMBERS_CHUNK" => self.handle_member_chunk(data),
            "PRESENCE_UPDATE" => self.handle_presence(data),
            "USER_SETTINGS_UPDATE" => {
                self.apply_user_settings(data);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Reads folders and the user's status from READY's `user_settings` or
    /// a USER_SETTINGS_UPDATE; keys a payload leaves out stay as they were.
    fn apply_user_settings(&mut self, settings: Option<&Value>) {
        let Some(settings) = settings else { return };
        if let Some(status) = settings.get("status").and_then(Value::as_str) {
            self.own_status = Some(status.to_owned());
        }
        if let Some(custom) = settings.get("custom_status") {
            self.own_custom_status = custom
                .get("text")
                .and_then(Value::as_str)
                .filter(|text| !text.is_empty())
                .map(str::to_owned);
        }
        let Some(folders) = settings.get("guild_folders").and_then(Value::as_array) else {
            return;
        };
        self.guild_folders = folders
            .iter()
            .map(|folder| GuildFolder {
                id: folder.get("id").and_then(|id| match id {
                    Value::String(text) => text.parse().ok(),
                    other => other.as_i64(),
                }),
                name: folder
                    .get("name")
                    .and_then(Value::as_str)
                    .filter(|name| !name.is_empty())
                    .map(str::to_owned),
                color: folder
                    .get("color")
                    .and_then(Value::as_u64)
                    .and_then(|color| u32::try_from(color).ok()),
                guild_ids: folder
                    .get("guild_ids")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|id| snowflake(Some(id)))
                    .collect(),
            })
            .collect();
    }

    fn handle_ready(&mut self, data: Option<&Value>) -> Result<(), CacheError> {
        let data = data.ok_or(CacheError::MissingData("READY"))?;

        if let Some(user) = data.get("user") {
            let user: User = decode(user)?;
            self.cache_user(user.clone());
            self.current_user = Some(user);
        }

        // With DEDUPE_USER_OBJECTS, users arrive once here and channels
        // only carry their ids.
        if let Some(users) = data.get("users").and_then(Value::as_array) {
            for value in users {
                if let Ok(user) = decode::<User>(value) {
                    self.cache_user(user);
                }
            }
        }
        self.apply_user_settings(data.get("user_settings"));

        let mut guild_order = Vec::new();
        if let Some(guilds) = data.get("guilds").and_then(Value::as_array) {
            for value in guilds {
                if let Some(guild_id) = snowflake(value.get("id")) {
                    if let Some(members) = value.get("members").and_then(Value::as_array) {
                        for member in members {
                            self.cache_member_value(guild_id, member);
                        }
                    }
                }
                let guild: Guild = decode(value)?;
                if !guild_order.contains(&guild.id) {
                    guild_order.push(guild.id);
                }
                self.merge_guild(guild);
            }
        }
        self.guild_order = guild_order;

        self.private_channel_ids.clear();
        for key in ["channels", "private_channels"] {
            if let Some(channels) = data.get(key).and_then(Value::as_array) {
                for value in channels {
                    let mut channel: Channel = decode(value)?;
                    if channel.recipients.is_empty() {
                        channel.recipients = value
                            .get("recipient_ids")
                            .and_then(Value::as_array)
                            .into_iter()
                            .flatten()
                            .filter_map(|id| snowflake(Some(id)))
                            .filter_map(|id| self.users.get(&id).cloned())
                            .collect();
                    }
                    self.update_channel(channel);
                }
            }
        }

        if let Some(presences) = data.get("presences").and_then(Value::as_array) {
            for presence in presences {
                self.cache_presence_value(presence);
            }
        }
        // User accounts get friends' and servers' presences here instead.
        if let Some(merged) = data.get("merged_presences") {
            let friends = merged
                .get("friends")
                .and_then(Value::as_array)
                .into_iter()
                .flatten();
            let guilds = merged
                .get("guilds")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_array)
                .flatten();
            for presence in friends.chain(guilds) {
                self.cache_presence_value(presence);
            }
        }

        self.read_state.handle_ready(data);
        Ok(())
    }

    fn handle_message_create(&mut self, data: Option<&Value>) -> Result<(), CacheError> {
        let data = data.ok_or(CacheError::MissingData("MESSAGE_CREATE"))?;
        let message: Message = decode(data)?;
        self.cache_user(message.author.clone());

        let channel_id = message.channel_id;
        let messages = self.messages.entry(channel_id).or_default();
        match messages.iter().position(|cached| cached.id == message.id) {
            Some(index) => messages[index] = message.clone(),
            None => {
                let index = messages.partition_point(|cached| cached.id > message.id);
                messages.insert(index, message.clone());
            }
        }
        if messages.len() > MAX_MESSAGES_PER_CHANNEL {
            messages.truncate(MAX_MESSAGES_PER_CHANNEL);
        }
        self.touch_message_channel(channel_id);

        let mention_count = self.mention_count(&message);
        self.read_state
            .record_message(channel_id, message.id, mention_count);
        Ok(())
    }

    fn handle_message_update(&mut self, data: Option<&Value>) -> Result<(), CacheError> {
        let data = data.ok_or(CacheError::MissingData("MESSAGE_UPDATE"))?;
        let channel_id = snowflake(data.get("channel_id"))
            .ok_or(CacheError::MissingData("MESSAGE_UPDATE.channel_id"))?;
        let message_id =
            snowflake(data.get("id")).ok_or(CacheError::MissingData("MESSAGE_UPDATE.id"))?;

        if let Ok(updated) = decode::<Message>(data) {
            self.cache_user(updated.author.clone());
            let messages = self.messages.entry(channel_id).or_default();
            match messages.iter().position(|cached| cached.id == message_id) {
                Some(index) => messages[index] = updated,
                None => {
                    let index = messages.partition_point(|cached| cached.id > updated.id);
                    messages.insert(index, updated);
                    if messages.len() > MAX_MESSAGES_PER_CHANNEL {
                        messages.truncate(MAX_MESSAGES_PER_CHANNEL);
                    }
                }
            }
            self.touch_message_channel(channel_id);
            return Ok(());
        }

        let mut touched = false;
        if let Some(messages) = self.messages.get_mut(&channel_id) {
            if let Some(existing) = messages.iter_mut().find(|m| m.id == message_id) {
                if let Some(content) = data.get("content").and_then(Value::as_str) {
                    existing.content = content.to_owned();
                }
                if data.get("edited_timestamp").is_some() {
                    existing.edited_timestamp = data
                        .get("edited_timestamp")
                        .and_then(Value::as_str)
                        .map(str::to_owned);
                }
                if let Some(attachments) = data.get("attachments") {
                    if let Ok(v) = serde_json::from_value(attachments.clone()) {
                        existing.attachments = v;
                    }
                }
                if let Some(embeds) = data.get("embeds").and_then(Value::as_array) {
                    existing.embeds = embeds.clone();
                }
                if let Some(pinned) = data.get("pinned").and_then(Value::as_bool) {
                    existing.pinned = pinned;
                }
                if let Some(mention_everyone) =
                    data.get("mention_everyone").and_then(Value::as_bool)
                {
                    existing.mention_everyone = mention_everyone;
                }
                touched = true;
            }
        }
        if touched {
            self.touch_message_channel(channel_id);
        }
        Ok(())
    }

    fn handle_message_delete(&mut self, data: Option<&Value>) -> Result<(), CacheError> {
        let data = data.ok_or(CacheError::MissingData("MESSAGE_DELETE"))?;
        let channel_id = snowflake(data.get("channel_id"))
            .ok_or(CacheError::MissingData("MESSAGE_DELETE.channel_id"))?;
        let message_id =
            snowflake(data.get("id")).ok_or(CacheError::MissingData("MESSAGE_DELETE.id"))?;
        let mut remove_entry = false;
        if let Some(messages) = self.messages.get_mut(&channel_id) {
            messages.retain(|m| m.id != message_id);
            remove_entry = messages.is_empty();
        }
        if remove_entry {
            self.messages.remove(&channel_id);
            self.message_channel_order.retain(|id| *id != channel_id);
        }
        Ok(())
    }

    fn handle_guild_create(&mut self, data: Option<&Value>) -> Result<(), CacheError> {
        let data = data.ok_or(CacheError::MissingData("GUILD_CREATE"))?;
        let guild: Guild = decode(data)?;
        self.merge_guild(guild.clone());

        if let Some(channels) = data.get("channels").and_then(Value::as_array) {
            for value in channels {
                let mut channel: Channel = decode(value)?;
                channel.guild_id = Some(guild.id);
                self.update_channel(channel);
            }
        }
        if let Some(members) = data.get("members").and_then(Value::as_array) {
            for member in members {
                self.cache_member_value(guild.id, member);
            }
        }
        if let Some(presences) = data.get("presences").and_then(Value::as_array) {
            for presence in presences {
                self.cache_presence_value(presence);
            }
        }
        Ok(())
    }

    fn handle_guild_update(&mut self, data: Option<&Value>) -> Result<(), CacheError> {
        let data = data.ok_or(CacheError::MissingData("GUILD_UPDATE"))?;
        let guild: Guild = decode(data)?;
        self.merge_guild(guild);
        Ok(())
    }

    fn handle_guild_delete(&mut self, data: Option<&Value>) -> Result<(), CacheError> {
        let data = data.ok_or(CacheError::MissingData("GUILD_DELETE"))?;
        let guild_id = snowflake(data.get("guild_id").or_else(|| data.get("id")))
            .ok_or(CacheError::MissingData("guild id"))?;
        self.guilds.remove(&guild_id);
        self.guild_order.retain(|id| *id != guild_id);

        let channel_ids = self.guild_channels.remove(&guild_id).unwrap_or_default();
        for channel_id in channel_ids {
            self.remove_channel(channel_id);
        }
        Ok(())
    }

    fn handle_channel(&mut self, data: Option<&Value>) -> Result<(), CacheError> {
        let data = data.ok_or(CacheError::MissingData("CHANNEL"))?;
        let channel: Channel = decode(data)?;
        self.update_channel(channel);
        Ok(())
    }

    fn handle_channel_delete(&mut self, data: Option<&Value>) -> Result<(), CacheError> {
        let data = data.ok_or(CacheError::MissingData("CHANNEL_DELETE"))?;
        let channel_id = snowflake(data.get("id")).ok_or(CacheError::MissingData("channel id"))?;
        self.remove_channel(channel_id);
        Ok(())
    }

    fn handle_user_update(&mut self, data: Option<&Value>) -> Result<(), CacheError> {
        let data = data.ok_or(CacheError::MissingData("USER_UPDATE"))?;
        let user: User = decode(data)?;
        let is_current_user = self
            .current_user
            .as_ref()
            .is_some_and(|current| current.id == user.id);
        self.cache_user(user.clone());
        if is_current_user {
            self.current_user = Some(user);
        }
        Ok(())
    }

    fn cache_user(&mut self, user: User) {
        self.users.insert(user.id, user);
        if self.users.len() > MAX_CACHED_USERS {
            let current_id = self.current_user.as_ref().map(|u| u.id);
            let mut to_remove = Vec::new();
            for (id, _) in self.users.iter() {
                if self.users.len().saturating_sub(to_remove.len()) <= MAX_CACHED_USERS {
                    break;
                }
                if Some(*id) != current_id {
                    to_remove.push(*id);
                }
            }
            for id in to_remove {
                self.users.remove(&id);
            }
            // Fallback: if still over limit (e.g. only current user remains but limit tiny), trim arbitrarily.
            if self.users.len() > MAX_CACHED_USERS {
                let keys: Vec<Snowflake> = self
                    .users
                    .keys()
                    .copied()
                    .filter(|id| Some(*id) != current_id)
                    .take(self.users.len().saturating_sub(MAX_CACHED_USERS))
                    .collect();
                for id in keys {
                    self.users.remove(&id);
                }
            }
        }
    }

    fn merge_guild(&mut self, incoming: Guild) {
        let merged = match self.guilds.get(&incoming.id) {
            Some(existing) => Guild {
                id: incoming.id,
                name: incoming.name.or_else(|| existing.name.clone()),
                icon: incoming.icon.or_else(|| existing.icon.clone()),
                splash: incoming.splash.or_else(|| existing.splash.clone()),
                banner: incoming.banner.or_else(|| existing.banner.clone()),
                owner_id: incoming.owner_id.or(existing.owner_id),
                permissions: incoming
                    .permissions
                    .or_else(|| existing.permissions.clone()),
                member_count: incoming.member_count.or(existing.member_count),
                unavailable: incoming.unavailable.or(existing.unavailable),
            },
            None => incoming,
        };
        self.guilds.insert(merged.id, merged.clone());
        if !self.guild_order.contains(&merged.id) {
            self.guild_order.push(merged.id);
        }
    }

    fn update_channel(&mut self, incoming: Channel) {
        let previous_guild_id = self
            .channels
            .get(&incoming.id)
            .and_then(|channel| channel.guild_id);
        let channel = match self.channels.get(&incoming.id) {
            Some(existing) => merge_channel(existing, incoming),
            None => incoming,
        };
        self.channels.insert(channel.id, channel.clone());
        if previous_guild_id != channel.guild_id {
            if let Some(previous_guild_id) = previous_guild_id {
                let empty = self
                    .guild_channels
                    .get_mut(&previous_guild_id)
                    .map(|channel_ids| {
                        channel_ids.retain(|id| *id != channel.id);
                        channel_ids.is_empty()
                    })
                    .unwrap_or(false);
                if empty {
                    self.guild_channels.remove(&previous_guild_id);
                }
            }
        }
        if let Some(guild_id) = channel.guild_id {
            let channel_ids = self.guild_channels.entry(guild_id).or_default();
            if !channel_ids.contains(&channel.id) {
                channel_ids.push(channel.id);
            }
        }
        if channel.guild_id.is_none() && !self.private_channel_ids.contains(&channel.id) {
            self.private_channel_ids.push(channel.id);
        }
    }

    fn remove_channel(&mut self, channel_id: Snowflake) {
        let removed = self.channels.remove(&channel_id);
        if let Some(guild_id) = removed.and_then(|channel| channel.guild_id) {
            let empty = self
                .guild_channels
                .get_mut(&guild_id)
                .map(|channel_ids| {
                    channel_ids.retain(|id| *id != channel_id);
                    channel_ids.is_empty()
                })
                .unwrap_or(false);
            if empty {
                self.guild_channels.remove(&guild_id);
            }
        }
        self.private_channel_ids.retain(|id| *id != channel_id);
        self.messages.remove(&channel_id);
        self.message_channel_order.retain(|id| *id != channel_id);
    }

    fn cache_member_value(&mut self, guild_id: Snowflake, value: &Value) {
        let user = value.get("user");
        let Some(user_id) = snowflake(
            user.and_then(|user| user.get("id"))
                .or_else(|| value.get("user_id")),
        ) else {
            return;
        };
        let username = user
            .and_then(|user| user.get("username"))
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_owned();
        let avatar = user
            .and_then(|user| user.get("avatar"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        let nickname = value
            .get("nick")
            .or_else(|| value.get("nickname"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        let roles = value
            .get("roles")
            .and_then(Value::as_array)
            .map(|roles| {
                roles
                    .iter()
                    .filter_map(|role| snowflake(Some(role)))
                    .collect()
            })
            .unwrap_or_default();
        self.members.entry(guild_id).or_default().insert(
            user_id,
            CachedMember {
                guild_id,
                user_id,
                username,
                avatar,
                nickname,
                roles,
            },
        );
    }

    fn cache_presence_value(&mut self, value: &Value) {
        if let Some(presence) = presence_from_value(value) {
            self.presences.insert(presence.user_id, presence);
        }
    }

    fn handle_guild_member(&mut self, data: Option<&Value>) -> Result<(), CacheError> {
        let data = data.ok_or(CacheError::MissingData("GUILD_MEMBER"))?;
        let guild_id = snowflake(data.get("guild_id"))
            .ok_or(CacheError::MissingData("guild_id in GUILD_MEMBER"))?;
        self.cache_member_value(guild_id, data);
        Ok(())
    }

    fn handle_guild_member_remove(&mut self, data: Option<&Value>) -> Result<(), CacheError> {
        let data = data.ok_or(CacheError::MissingData("GUILD_MEMBER_REMOVE"))?;
        let guild_id = snowflake(data.get("guild_id"))
            .ok_or(CacheError::MissingData("guild_id in GUILD_MEMBER_REMOVE"))?;
        let user_id = snowflake(
            data.get("user")
                .and_then(|user| user.get("id"))
                .or_else(|| data.get("user_id")),
        )
        .ok_or(CacheError::MissingData("user id in GUILD_MEMBER_REMOVE"))?;
        if let Some(members) = self.members.get_mut(&guild_id) {
            members.remove(&user_id);
        }
        Ok(())
    }

    fn handle_member_chunk(&mut self, data: Option<&Value>) -> Result<(), CacheError> {
        let data = data.ok_or(CacheError::MissingData("GUILD_MEMBERS_CHUNK"))?;
        let guild_id = snowflake(data.get("guild_id"))
            .ok_or(CacheError::MissingData("guild_id in GUILD_MEMBERS_CHUNK"))?;
        if let Some(members) = data.get("members").and_then(Value::as_array) {
            for member in members {
                self.cache_member_value(guild_id, member);
            }
        }
        Ok(())
    }

    fn handle_presence(&mut self, data: Option<&Value>) -> Result<(), CacheError> {
        let data = data.ok_or(CacheError::MissingData("PRESENCE_UPDATE"))?;
        self.cache_presence_value(data);
        Ok(())
    }

    fn mention_count(&self, message: &Message) -> u32 {
        let Some(current_user) = self.current_user.as_ref() else {
            return 0;
        };
        if message.author.id == current_user.id {
            return 0;
        }
        if message.mention_everyone {
            return 1;
        }
        u32::from(
            message
                .mentions
                .iter()
                .filter_map(|mention| snowflake(mention.get("id")))
                .any(|id| id == current_user.id),
        )
    }

    fn clear_data(&mut self) {
        self.current_user = None;
        self.users.clear();
        self.guilds.clear();
        self.guild_order.clear();
        self.guild_folders.clear();
        self.own_status = None;
        self.own_custom_status = None;
        self.channels.clear();
        self.guild_channels.clear();
        self.private_channel_ids.clear();
        self.messages.clear();
        self.message_channel_order.clear();
        self.members.clear();
        self.presences.clear();
        self.read_state.clear();
    }

    fn touch_message_channel(&mut self, channel_id: Snowflake) {
        if let Some(pos) = self
            .message_channel_order
            .iter()
            .position(|id| *id == channel_id)
        {
            self.message_channel_order.remove(pos);
        }
        self.message_channel_order.push_front(channel_id);

        while self.message_channel_order.len() > MAX_CACHED_MESSAGE_CHANNELS {
            if let Some(oldest) = self.message_channel_order.pop_back() {
                self.messages.remove(&oldest);
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct StateCacheHandler {
    cache: Arc<Mutex<StateCache>>,
}

impl StateCacheHandler {
    pub fn new(cache: Arc<Mutex<StateCache>>) -> Self {
        Self { cache }
    }

    pub fn cache(&self) -> &Arc<Mutex<StateCache>> {
        &self.cache
    }
}

impl GatewayEventHandler for StateCacheHandler {
    fn name(&self) -> &str {
        "state_cache"
    }

    fn supported_events(&self) -> &'static [&'static str] {
        HANDLED_EVENTS
    }

    fn handle(&self, event_type: &str, data: Option<&Value>) -> Result<(), String> {
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| "state cache lock poisoned".to_owned())?;
        cache
            .handle_event(event_type, data)
            .map_err(|error| error.to_string())
    }
}

fn decode<T: DeserializeOwned>(value: &Value) -> Result<T, CacheError> {
    serde_json::from_value(value.clone()).map_err(CacheError::InvalidPayload)
}

fn snowflake(value: Option<&Value>) -> Option<Snowflake> {
    match value? {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn is_direct_message(kind: ChannelType) -> bool {
    matches!(kind.0, 1 | 3)
}

fn merge_channel(existing: &Channel, incoming: Channel) -> Channel {
    let kind = if incoming.kind == ChannelType::default() {
        existing.kind
    } else {
        incoming.kind
    };
    let guild_id = if is_direct_message(kind) {
        None
    } else {
        incoming.guild_id.or(existing.guild_id)
    };
    Channel {
        id: incoming.id,
        kind,
        guild_id,
        name: incoming.name.or_else(|| existing.name.clone()),
        position: incoming.position.or(existing.position),
        parent_id: incoming.parent_id.or(existing.parent_id),
        last_message_id: incoming.last_message_id.or(existing.last_message_id),
        topic: incoming.topic.or_else(|| existing.topic.clone()),
        nsfw: incoming.nsfw.or(existing.nsfw),
        bitrate: incoming.bitrate.or(existing.bitrate),
        user_limit: incoming.user_limit.or(existing.user_limit),
        permission_overwrites: incoming
            .permission_overwrites
            .or_else(|| existing.permission_overwrites.clone()),
        recipients: if incoming.recipients.is_empty() {
            existing.recipients.clone()
        } else {
            incoming.recipients
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn user() -> Value {
        json!({"id": "42", "username": "lamp", "global_name": "Lamp"})
    }

    #[test]
    fn member_and_presence_events_are_cached() {
        let mut cache = StateCache::new();
        cache
            .handle_event(
                "GUILD_MEMBER_ADD",
                Some(&json!({
                    "guild_id": "7",
                    "user": {"id": "11", "username": "member", "avatar": "hash"},
                    "nick": "Nick",
                    "roles": ["3", "4"]
                })),
            )
            .expect("member add");
        cache
            .handle_event(
                "PRESENCE_UPDATE",
                Some(&json!({
                    "user": {"id": "11"},
                    "status": "online",
                    "activities": [{"name": "Compose"}]
                })),
            )
            .expect("presence update");

        let member = cache.member(7, 11).expect("cached member");
        assert_eq!(member.nickname.as_deref(), Some("Nick"));
        assert_eq!(member.roles, vec![3, 4]);
        assert_eq!(cache.members_for_guild(7).len(), 1);
        let presence = cache.presence(11).expect("cached presence");
        assert_eq!(presence.status, "online");
        assert_eq!(presence.activities, vec!["Compose".to_owned()]);

        cache
            .handle_event(
                "GUILD_MEMBER_REMOVE",
                Some(&json!({"guild_id": "7", "user": {"id": "11"}})),
            )
            .expect("member remove");
        assert!(cache.member(7, 11).is_none());
    }

    #[test]
    fn ready_seeds_user_guilds_channels_and_read_state() {
        let mut cache = StateCache::new();
        cache
            .handle_event(
                "READY",
                Some(&json!({
                    "user": user(),
                    "guilds": [
                        {"id": "7", "unavailable": true},
                        {"id": "8", "unavailable": false}
                    ],
                    "channels": [
                        {"id": "9", "type": 1, "recipients": [user()], "last_message_id": "100"}
                    ],
                    "read_state": [
                        {"channel_id": "9", "last_message_id": "99", "mention_count": 1}
                    ]
                })),
            )
            .expect("ready event");

        assert_eq!(cache.current_user().map(|user| user.id), Some(42));
        assert_eq!(
            cache
                .ordered_guilds()
                .iter()
                .map(|guild| guild.id)
                .collect::<Vec<_>>(),
            vec![7, 8]
        );
        assert_eq!(
            cache.guild(7).and_then(|guild| guild.unavailable),
            Some(true)
        );
        assert_eq!(cache.private_channels()[0].id, 9);
        assert_eq!(cache.read_state().mention_count(9), 1);
    }

    #[test]
    fn guild_channel_index_tracks_updates_and_deletes() {
        let mut cache = StateCache::new();
        for (id, position) in [(10, 1), (11, 0)] {
            cache
                .handle_event(
                    "CHANNEL_CREATE",
                    Some(&json!({
                        "id": id.to_string(),
                        "guild_id": "7",
                        "type": 0,
                        "name": "channel",
                        "position": position,
                    })),
                )
                .expect("channel create");
        }
        assert_eq!(
            cache
                .channels_for_guild(7)
                .iter()
                .map(|channel| channel.id)
                .collect::<Vec<_>>(),
            vec![11, 10]
        );

        cache
            .handle_event("CHANNEL_DELETE", Some(&json!({"id": "11"})))
            .expect("channel delete");
        assert_eq!(
            cache
                .channels_for_guild(7)
                .iter()
                .map(|channel| channel.id)
                .collect::<Vec<_>>(),
            vec![10]
        );

        cache
            .handle_event("GUILD_DELETE", Some(&json!({"guild_id": "7"})))
            .expect("guild delete");
        assert!(cache.channels_for_guild(7).is_empty());
    }

    #[test]
    fn message_create_keeps_messages_sorted_and_counts_mentions() {
        let mut cache = StateCache::new();
        cache
            .handle_event("READY", Some(&json!({"user": user(), "guilds": []})))
            .expect("ready event");

        cache
            .handle_event(
                "MESSAGE_CREATE",
                Some(&json!({
                    "id": "10",
                    "channel_id": "9",
                    "author": {"id": "5", "username": "sender"},
                    "content": "hello <@42>",
                    "mentions": [{"id": "42"}],
                    "timestamp": "2026-01-01T00:00:00Z"
                })),
            )
            .expect("first message");
        cache
            .handle_event(
                "MESSAGE_CREATE",
                Some(&json!({
                    "id": "20",
                    "channel_id": "9",
                    "author": user(),
                    "content": "newest",
                    "timestamp": "2026-01-01T00:00:01Z"
                })),
            )
            .expect("second message");

        assert_eq!(
            cache
                .messages(9)
                .iter()
                .map(|message| message.id)
                .collect::<Vec<_>>(),
            vec![20, 10]
        );
        assert_eq!(
            cache.user(5).map(|user| user.username.as_str()),
            Some("sender")
        );
        assert_eq!(cache.read_state().mention_count(9), 1);
        assert!(cache.read_state().is_unread(9, Some(20), None));
    }

    #[test]
    fn guild_events_merge_preserve_and_remove_channels() {
        let mut cache = StateCache::new();
        cache
            .handle_event(
                "GUILD_CREATE",
                Some(&json!({
                    "id": "7",
                    "name": "Guild",
                    "icon": "icon-1",
                    "member_count": 3,
                    "channels": [
                        {"id": "11", "type": 0, "name": "general", "position": 1}
                    ]
                })),
            )
            .expect("guild create");

        cache
            .handle_event("GUILD_UPDATE", Some(&json!({"id": "7", "name": "Renamed"})))
            .expect("guild update");

        let guild = cache.guild(7).expect("guild");
        assert_eq!(guild.name.as_deref(), Some("Renamed"));
        assert_eq!(guild.icon.as_deref(), Some("icon-1"));
        assert_eq!(cache.channels_for_guild(7)[0].id, 11);

        cache
            .handle_event(
                "GUILD_DELETE",
                Some(&json!({"id": "7", "unavailable": true})),
            )
            .expect("guild delete");

        assert!(cache.guild(7).is_none());
        assert!(cache.channels_for_guild(7).is_empty());
        assert!(cache.ordered_guilds().is_empty());
    }

    #[test]
    fn switching_tokens_discards_cached_account_data() {
        let mut cache = StateCache::new();
        cache
            .handle_event("READY", Some(&json!({"user": user(), "guilds": []})))
            .expect("ready event");
        assert!(cache.current_user().is_some());

        cache.switch_token("token-1");
        assert!(cache.current_user().is_none());

        cache
            .handle_event("READY", Some(&json!({"user": user(), "guilds": []})))
            .expect("ready event");
        cache.switch_token("token-1");
        assert!(cache.current_user().is_some());
    }

    #[test]
    fn ready_resolves_deduped_recipients_and_folders() {
        let mut cache = StateCache::new();
        cache
            .handle_event(
                "READY",
                Some(&json!({
                    "user": {"id": "1", "username": "me"},
                    "users": [{"id": "2", "username": "friend", "global_name": "Friend"}],
                    "user_settings": {"status": "invisible", "custom_status": {"text": "as above"}, "guild_folders": [
                        {"id": null, "guild_ids": ["10"]},
                        {"id": 7, "name": "Games", "color": 5793266, "guild_ids": ["11", "12"]}
                    ]},
                    "private_channels": [{"id": "9", "type": 1, "recipient_ids": ["2"]}],
                    "merged_presences": {"friends": [{"user_id": "2", "status": "online", "activities": [
                        {"type": 4, "name": "Custom Status", "state": "as above, so below"},
                        {"type": 0, "name": "Blender"}
                    ]}]}
                })),
            )
            .unwrap();

        assert_eq!(cache.own_status(), Some("invisible"));
        assert_eq!(cache.own_custom_status(), Some("as above"));
        let dm = cache.private_channels()[0];
        assert_eq!(dm.recipients[0].global_name.as_deref(), Some("Friend"));
        let folders = cache.guild_folders();
        assert_eq!(folders.len(), 2);
        assert_eq!(folders[0].id, None);
        assert_eq!(folders[1].name.as_deref(), Some("Games"));
        assert_eq!(folders[1].guild_ids, vec![11, 12]);
        let friend = cache.presence(2).unwrap();
        assert_eq!(friend.status, "online");
        assert_eq!(friend.custom_status.as_deref(), Some("as above, so below"));
        assert_eq!(friend.activities, vec!["Blender".to_owned()]);

        cache
            .handle_event(
                "USER_SETTINGS_UPDATE",
                Some(&json!({"guild_folders": [{"id": "8", "guild_ids": ["12"]}]})),
            )
            .unwrap();
        assert_eq!(cache.guild_folders()[0].id, Some(8));
    }
}
