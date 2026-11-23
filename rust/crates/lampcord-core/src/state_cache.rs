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
    "GUILD_MEMBER_LIST_UPDATE",
    "PRESENCE_UPDATE",
    "USER_SETTINGS_UPDATE",
    "RELATIONSHIP_ADD",
    "RELATIONSHIP_UPDATE",
    "RELATIONSHIP_REMOVE",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemberListRow {
    Pending,
    Group { id: String, count: Option<u32> },
    Member(Snowflake),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MemberListState {
    pub rows: Vec<MemberListRow>,
    pub online_count: Option<u32>,
    pub member_count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    pub activities: Vec<String>,
}

#[derive(Debug)]
pub enum CacheError {
    MissingData(&'static str),
    InvalidPayload(serde_json::Error),
    SkippedEntries { event: &'static str, count: usize },
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingData(field) => write!(formatter, "missing {field} in gateway payload"),
            Self::InvalidPayload(error) => write!(formatter, "invalid cache payload: {error}"),
            Self::SkippedEntries { event, count } => {
                write!(formatter, "skipped {count} undecodable entries in {event}")
            }
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
    channels: HashMap<Snowflake, Channel>,
    guild_channels: HashMap<Snowflake, Vec<Snowflake>>,
    private_channel_ids: Vec<Snowflake>,
    messages: HashMap<Snowflake, Vec<Message>>,
    message_channel_order: VecDeque<Snowflake>,
    members: HashMap<Snowflake, HashMap<Snowflake, CachedMember>>,
    presences: HashMap<Snowflake, CachedPresence>,
    read_state: ReadStateStore,
    guild_order_hint: Vec<Snowflake>,
    member_lists: HashMap<Snowflake, MemberListState>,
    dirty: bool,
    relationships: HashMap<Snowflake, CachedRelationship>,
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
        let rank: HashMap<Snowflake, usize> = self
            .guild_order_hint
            .iter()
            .enumerate()
            .map(|(index, id)| (*id, index))
            .collect();
        let mut keyed: Vec<(usize, usize, Snowflake)> = self
            .guild_order
            .iter()
            .enumerate()
            .map(|(arrival, id)| (rank.get(id).copied().unwrap_or(usize::MAX), arrival, *id))
            .collect();
        keyed.sort_unstable();
        keyed
            .into_iter()
            .filter_map(|(_, _, guild_id)| self.guilds.get(&guild_id))
            .collect()
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

    pub fn member_list(&self, guild_id: Snowflake) -> Option<&MemberListState> {
        self.member_lists.get(&guild_id)
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
        if !matches!(event_type, "PRESENCE_UPDATE" | "TYPING_START") {
            self.dirty = true;
        }
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
            "GUILD_MEMBER_LIST_UPDATE" => self.handle_member_list_update(data),
            "USER_SETTINGS_UPDATE" => self.handle_user_settings(data),
            "RELATIONSHIP_ADD" | "RELATIONSHIP_UPDATE" => self.handle_relationship_upsert(data),
            "RELATIONSHIP_REMOVE" => self.handle_relationship_remove(data),
            _ => Ok(()),
        }
    }

    fn handle_ready(&mut self, data: Option<&Value>) -> Result<(), CacheError> {
        let data = data.ok_or(CacheError::MissingData("READY"))?;

        if let Some(user) = data.get("user") {
            let user: User = decode(user)?;
            self.cache_user(user.clone());
            self.current_user = Some(user);
        }

        if let Some(users) = data.get("users").and_then(Value::as_array) {
            for value in users {
                if let Ok(user) = decode::<User>(value) {
                    self.cache_user(user);
                }
            }
        }

        if let Some(settings) = data.get("user_settings") {
            let hint = guild_order_from_settings(settings);
            if !hint.is_empty() {
                self.guild_order_hint = hint;
            }
        }

        let mut skipped = 0usize;
        let mut guild_order = Vec::new();
        if let Some(guilds) = data.get("guilds").and_then(Value::as_array) {
            let merged_members = data.get("merged_members").and_then(Value::as_array);
            for (index, value) in guilds.iter().enumerate() {
                if let Some(guild_id) = snowflake(value.get("id")) {
                    if let Some(members) = value.get("members").and_then(Value::as_array) {
                        for member in members {
                            self.cache_member_value(guild_id, member);
                        }
                    }
                }
                let guild: Guild = match decode(value) {
                    Ok(guild) => guild,
                    Err(_) => {
                        skipped += 1;
                        continue;
                    }
                };
                if let Some(members) = merged_members
                    .and_then(|lists| lists.get(index))
                    .and_then(Value::as_array)
                {
                    for member in members {
                        self.cache_member_value(guild.id, member);
                    }
                }
                if !guild_order.contains(&guild.id) {
                    guild_order.push(guild.id);
                }
                self.merge_guild(guild);
            }
        }
        self.guild_order = guild_order;
        self.guilds.retain(|id, _| self.guild_order.contains(id));
        self.guild_channels
            .retain(|guild_id, _| self.guilds.contains_key(guild_id));
        let stale_channels: Vec<Snowflake> = self
            .channels
            .values()
            .filter(|channel| {
                channel
                    .guild_id
                    .is_some_and(|guild_id| !self.guilds.contains_key(&guild_id))
            })
            .map(|channel| channel.id)
            .collect();
        for channel_id in stale_channels {
            self.channels.remove(&channel_id);
        }

        self.private_channel_ids.clear();
        self.member_lists.clear();
        self.relationships.clear();
        if let Some(relationships) = data.get("relationships").and_then(Value::as_array) {
            for value in relationships {
                self.ingest_relationship(value);
            }
        }
        for key in ["channels", "private_channels"] {
            if let Some(channels) = data.get(key).and_then(Value::as_array) {
                for value in channels {
                    match decode::<Channel>(value) {
                        Ok(channel) => self.update_channel(channel),
                        Err(_) => skipped += 1,
                    }
                }
            }
        }

        if let Some(presences) = data.get("presences").and_then(Value::as_array) {
            for presence in presences {
                self.cache_presence_value(presence);
            }
        }

        self.read_state.handle_ready(data);
        if skipped > 0 {
            return Err(CacheError::SkippedEntries {
                event: "READY",
                count: skipped,
            });
        }
        Ok(())
    }

    fn handle_relationship_upsert(&mut self, data: Option<&Value>) -> Result<(), CacheError> {
        let data = data.ok_or(CacheError::MissingData("RELATIONSHIP"))?;
        self.ingest_relationship(data);
        Ok(())
    }

    fn handle_relationship_remove(&mut self, data: Option<&Value>) -> Result<(), CacheError> {
        let data = data.ok_or(CacheError::MissingData("RELATIONSHIP_REMOVE"))?;
        if let Some(user_id) = snowflake(data.get("id")) {
            self.relationships.remove(&user_id);
        }
        Ok(())
    }

    fn ingest_relationship(&mut self, value: &Value) {
        let Some(user_id) = snowflake(value.get("id")) else {
            return;
        };
        let kind = value.get("type").and_then(Value::as_u64).unwrap_or(0) as u8;
        if let Some(user) = value.get("user").and_then(|user| decode::<User>(user).ok()) {
            self.cache_user(user);
        }
        self.relationships
            .insert(user_id, CachedRelationship { user_id, kind });
    }

    fn handle_user_settings(&mut self, data: Option<&Value>) -> Result<(), CacheError> {
        let data = data.ok_or(CacheError::MissingData("USER_SETTINGS_UPDATE"))?;
        let hint = guild_order_from_settings(data);
        if !hint.is_empty() {
            self.guild_order_hint = hint;
        }
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

        let mut skipped = 0usize;
        if let Some(channels) = data.get("channels").and_then(Value::as_array) {
            for value in channels {
                match decode::<Channel>(value) {
                    Ok(mut channel) => {
                        channel.guild_id = Some(guild.id);
                        self.update_channel(channel);
                    }
                    Err(_) => skipped += 1,
                }
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
        if skipped > 0 {
            return Err(CacheError::SkippedEntries {
                event: "GUILD_CREATE",
                count: skipped,
            });
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
        let merged = match self.users.remove(&user.id) {
            Some(existing) => User {
                avatar: user.avatar.clone().or_else(|| existing.avatar.clone()),
                avatar_decoration_data: user
                    .avatar_decoration_data
                    .clone()
                    .or_else(|| existing.avatar_decoration_data.clone()),
                global_name: user
                    .global_name
                    .clone()
                    .or_else(|| existing.global_name.clone()),
                ..user
            },
            None => user,
        };
        self.users.insert(merged.id, merged);
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
                roles: if incoming.roles.is_empty() {
                    existing.roles.clone()
                } else {
                    incoming.roles
                },
            },
            None => incoming,
        };
        self.guilds.insert(merged.id, merged.clone());
        if !self.guild_order.contains(&merged.id) {
            self.guild_order.push(merged.id);
        }
    }

    pub fn upsert_channels(&mut self, channels: Vec<Channel>) {
        self.dirty = true;
        for channel in channels {
            self.update_channel(channel);
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
        let cached = self.users.get(&user_id);
        let username = user
            .and_then(|user| user.get("username"))
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| cached.map(|cached| cached.username.clone()))
            .unwrap_or_else(|| "unknown".to_owned());
        let avatar = user
            .and_then(|user| user.get("avatar"))
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| cached.and_then(|cached| cached.avatar.clone()));
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
        let user = value.get("user");
        let Some(user_id) = snowflake(
            user.and_then(|user| user.get("id"))
                .or_else(|| value.get("user_id")),
        ) else {
            return;
        };
        let status = value
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("offline")
            .to_owned();
        let activities = value
            .get("activities")
            .and_then(Value::as_array)
            .map(|activities| {
                activities
                    .iter()
                    .filter_map(|activity| {
                        activity
                            .get("name")
                            .and_then(Value::as_str)
                            .map(str::to_owned)
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.presences.insert(
            user_id,
            CachedPresence {
                user_id,
                status,
                activities,
            },
        );
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

    fn handle_member_list_update(&mut self, data: Option<&Value>) -> Result<(), CacheError> {
        let data = data.ok_or(CacheError::MissingData("GUILD_MEMBER_LIST_UPDATE"))?;
        let guild_id = snowflake(data.get("guild_id")).ok_or(CacheError::MissingData(
            "guild_id in GUILD_MEMBER_LIST_UPDATE",
        ))?;
        let ops = data
            .get("ops")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        let mut state = self.member_lists.remove(&guild_id).unwrap_or_default();
        if let Some(count) = u32_of(data.get("online_count")) {
            state.online_count = Some(count);
        }
        if let Some(count) = u32_of(data.get("member_count")) {
            state.member_count = Some(count);
        }
        let rows = &mut state.rows;
        for op in &ops {
            match op.get("op").and_then(Value::as_str).unwrap_or_default() {
                "SYNC" => {
                    let Some((start, _end)) = index_pair(op.get("range")) else {
                        continue;
                    };
                    let items = op
                        .get("items")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default();
                    if rows.len() < start + items.len() {
                        rows.resize(start + items.len(), MemberListRow::Pending);
                    }
                    for (offset, item) in items.iter().enumerate() {
                        rows[start + offset] = self.cache_list_item(guild_id, item);
                    }
                }
                "INSERT" => {
                    let (Some(index), Some(item)) = (index_of(op.get("index")), op.get("item"))
                    else {
                        continue;
                    };
                    let row = self.cache_list_item(guild_id, item);
                    if index <= rows.len() {
                        rows.insert(index, row);
                    }
                }
                "UPDATE" => {
                    let (Some(index), Some(item)) = (index_of(op.get("index")), op.get("item"))
                    else {
                        continue;
                    };
                    let row = self.cache_list_item(guild_id, item);
                    if index < rows.len() {
                        rows[index] = row;
                    }
                }
                "DELETE" => {
                    let Some(index) = index_of(op.get("index")) else {
                        continue;
                    };
                    if index < rows.len() {
                        if let MemberListRow::Member(user_id) = rows[index] {
                            if let Some(members) = self.members.get_mut(&guild_id) {
                                members.remove(&user_id);
                            }
                        }
                        rows.remove(index);
                    }
                }
                "INVALIDATE" => {
                    let Some((start, end)) = index_pair(op.get("range")) else {
                        continue;
                    };
                    let end = end.min(rows.len().saturating_sub(1));
                    for row in rows.iter_mut().take(end + 1).skip(start) {
                        *row = MemberListRow::Pending;
                    }
                }
                _ => {}
            }
        }
        self.member_lists.insert(guild_id, state);
        Ok(())
    }

    fn cache_list_item(&mut self, guild_id: Snowflake, item: &Value) -> MemberListRow {
        if let Some(group) = item.get("group") {
            return MemberListRow::Group {
                id: group
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                count: u32_of(group.get("count")),
            };
        }
        let Some(member) = item.get("member") else {
            return MemberListRow::Pending;
        };
        let Some(user_id) = snowflake(
            member
                .get("user")
                .and_then(|user| user.get("id"))
                .or_else(|| member.get("user_id")),
        ) else {
            return MemberListRow::Pending;
        };
        self.cache_member_value(guild_id, member);
        if let Some(presence) = member.get("presence").and_then(Value::as_object) {
            let mut presence = presence.clone();
            presence
                .entry("user".to_owned())
                .or_insert_with(|| serde_json::json!({ "id": user_id.to_string() }));
            self.cache_presence_value(&Value::Object(presence));
        }
        MemberListRow::Member(user_id)
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
        self.guild_order_hint.clear();
        self.member_lists.clear();
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CachedRelationship {
    pub user_id: Snowflake,
    pub kind: u8,
}

const PERSISTED_CHANNELS: usize = 20;
const PERSISTED_MEMBERS_PER_GUILD: usize = 500;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersistedCache {
    current_user: Option<User>,
    users: Vec<User>,
    guilds: Vec<Guild>,
    guild_order: Vec<Snowflake>,
    guild_order_hint: Vec<Snowflake>,
    channels: Vec<Channel>,
    members: Vec<(Snowflake, Vec<CachedMember>)>,
    messages: Vec<(Snowflake, Vec<Message>)>,
    relationships: Vec<CachedRelationship>,
}

const PERSISTED_MESSAGES_PER_CHANNEL: usize = 50;

impl PersistedCache {
    pub fn to_bytes(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec_named(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        rmp_serde::from_slice(bytes).ok()
    }
}

impl StateCache {
    pub fn relationship_entries(&self) -> Vec<CachedRelationship> {
        self.relationships.values().cloned().collect()
    }

    pub fn take_persisted_if_dirty(&mut self) -> Option<PersistedCache> {
        if !self.dirty {
            return None;
        }
        self.dirty = false;
        Some(self.persisted())
    }

    pub fn persisted(&self) -> PersistedCache {
        PersistedCache {
            current_user: self.current_user.clone(),
            users: self.users.values().cloned().collect(),
            guilds: self.guilds.values().cloned().collect(),
            guild_order: self.guild_order.clone(),
            guild_order_hint: self.guild_order_hint.clone(),
            channels: self.channels.values().cloned().collect(),
            members: self
                .members
                .iter()
                .map(|(guild_id, members)| {
                    (
                        *guild_id,
                        members
                            .values()
                            .take(PERSISTED_MEMBERS_PER_GUILD)
                            .cloned()
                            .collect(),
                    )
                })
                .collect(),
            messages: self
                .message_channel_order
                .iter()
                .rev()
                .take(PERSISTED_CHANNELS)
                .filter_map(|channel_id| {
                    self.messages.get(channel_id).map(|messages| {
                        let start = messages
                            .len()
                            .saturating_sub(PERSISTED_MESSAGES_PER_CHANNEL);
                        (*channel_id, messages[start..].to_vec())
                    })
                })
                .collect(),
            relationships: self.relationships.values().cloned().collect(),
        }
    }

    pub fn restore(&mut self, persisted: &PersistedCache) {
        self.current_user = persisted.current_user.clone();
        for user in &persisted.users {
            self.users.insert(user.id, user.clone());
        }
        for guild in &persisted.guilds {
            self.guilds.insert(guild.id, guild.clone());
        }
        self.guild_order = persisted.guild_order.clone();
        self.guild_order_hint = persisted.guild_order_hint.clone();
        self.upsert_channels(persisted.channels.clone());
        for (guild_id, members) in &persisted.members {
            let entry = self.members.entry(*guild_id).or_default();
            for member in members {
                entry.insert(member.user_id, member.clone());
            }
        }
        for (channel_id, messages) in &persisted.messages {
            self.messages.insert(*channel_id, messages.clone());
            self.message_channel_order.push_back(*channel_id);
        }
        for relationship in &persisted.relationships {
            self.relationships
                .insert(relationship.user_id, relationship.clone());
        }
        self.dirty = false;
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

fn guild_order_from_settings(settings: &Value) -> Vec<Snowflake> {
    let from_folders: Vec<Snowflake> = settings
        .get("guild_folders")
        .and_then(Value::as_array)
        .map(|folders| {
            folders
                .iter()
                .filter_map(|folder| folder.get("guild_ids").and_then(Value::as_array))
                .flatten()
                .filter_map(|id| snowflake(Some(id)))
                .collect()
        })
        .unwrap_or_default();
    if !from_folders.is_empty() {
        return from_folders;
    }
    settings
        .get("guild_positions")
        .and_then(Value::as_array)
        .map(|ids| ids.iter().filter_map(|id| snowflake(Some(id))).collect())
        .unwrap_or_default()
}

fn u32_of(value: Option<&Value>) -> Option<u32> {
    value?.as_u64().and_then(|value| u32::try_from(value).ok())
}

fn index_of(value: Option<&Value>) -> Option<usize> {
    value?.as_u64().map(|index| index as usize)
}

fn index_pair(value: Option<&Value>) -> Option<(usize, usize)> {
    let pair = value?.as_array()?;
    Some((
        pair.first()?.as_u64()? as usize,
        pair.get(1)?.as_u64()? as usize,
    ))
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
        icon: incoming.icon.or_else(|| existing.icon.clone()),
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
}

#[cfg(test)]
mod ordering_and_members_tests {
    use super::*;
    use serde_json::json;

    fn guild(id: u64) -> Value {
        json!({"id": id.to_string(), "name": format!("g{id}"), "icon": format!("icon{id}")})
    }

    #[test]
    fn ready_applies_folder_order_from_user_settings() {
        let mut cache = StateCache::new();
        cache
            .handle_event(
                "READY",
                Some(&json!({
                    "user": {"id": "1", "username": "me"},
                    "guilds": [guild(10), guild(20), guild(30)],
                    "user_settings": {
                        "guild_folders": [
                            {"id": null, "guild_ids": ["30"]},
                            {"id": 5, "guild_ids": ["20", "10"]}
                        ]
                    }
                })),
            )
            .expect("ready");
        let ids: Vec<u64> = cache.ordered_guilds().iter().map(|g| g.id).collect();
        assert_eq!(ids, vec![30, 20, 10]);
    }

    #[test]
    fn unlisted_guilds_follow_listed_ones_and_positions_are_used_as_fallback() {
        let mut cache = StateCache::new();
        cache
            .handle_event(
                "READY",
                Some(&json!({
                    "user": {"id": "1", "username": "me"},
                    "guilds": [guild(10), guild(20), guild(30)],
                    "user_settings": {"guild_positions": ["20", "10"]}
                })),
            )
            .expect("ready");
        let ids: Vec<u64> = cache.ordered_guilds().iter().map(|g| g.id).collect();
        assert_eq!(ids, vec![20, 10, 30]);
    }

    #[test]
    fn user_settings_update_reorders_without_clearing_on_partial_payload() {
        let mut cache = StateCache::new();
        cache
            .handle_event(
                "READY",
                Some(&json!({
                    "user": {"id": "1", "username": "me"},
                    "guilds": [guild(10), guild(20)],
                    "user_settings": {"guild_positions": ["10", "20"]}
                })),
            )
            .expect("ready");
        cache
            .handle_event(
                "USER_SETTINGS_UPDATE",
                Some(&json!({"guild_positions": ["20", "10"]})),
            )
            .expect("settings");
        let ids: Vec<u64> = cache.ordered_guilds().iter().map(|g| g.id).collect();
        assert_eq!(ids, vec![20, 10]);
        cache
            .handle_event("USER_SETTINGS_UPDATE", Some(&json!({"theme": "dark"})))
            .expect("partial settings");
        let ids: Vec<u64> = cache.ordered_guilds().iter().map(|g| g.id).collect();
        assert_eq!(ids, vec![20, 10]);
    }

    #[test]
    fn ready_caches_merged_members_with_users_from_users_array() {
        let mut cache = StateCache::new();
        cache
            .handle_event(
                "READY",
                Some(&json!({
                    "user": {"id": "1", "username": "me"},
                    "guilds": [guild(10)],
                    "users": [{"id": "55", "username": "alice", "avatar": "av"}],
                    "merged_members": [[{"user_id": "55", "nick": "Ali", "roles": []}]]
                })),
            )
            .expect("ready");
        let member = cache.member(10, 55).expect("merged member cached");
        assert_eq!(member.username, "alice");
        assert_eq!(member.avatar.as_deref(), Some("av"));
        assert_eq!(member.nickname.as_deref(), Some("Ali"));
    }

    #[test]
    fn member_list_sync_insert_update_delete_track_members() {
        let mut cache = StateCache::new();
        cache
            .handle_event(
                "READY",
                Some(&json!({"user": {"id": "1", "username": "me"}, "guilds": [guild(10)]})),
            )
            .expect("ready");
        let member = |id: u64, name: &str| {
            json!({"member": {"user": {"id": id.to_string(), "username": name}, "roles": [],
                   "presence": {"status": "online", "activities": []}}})
        };
        cache
            .handle_event(
                "GUILD_MEMBER_LIST_UPDATE",
                Some(&json!({"guild_id": "10", "ops": [{"op": "SYNC", "range": [0, 2],
                    "items": [{"group": {"id": "online", "count": 2}}, member(1, "a"), member(2, "b")]}]})),
            )
            .expect("sync");
        assert!(cache.member(10, 1).is_some());
        assert!(cache.member(10, 2).is_some());
        assert_eq!(cache.presence(2).map(|p| p.status.as_str()), Some("online"));

        cache
            .handle_event(
                "GUILD_MEMBER_LIST_UPDATE",
                Some(&json!({"guild_id": "10", "ops": [{"op": "INSERT", "index": 1, "item": member(3, "c")}]})),
            )
            .expect("insert");
        assert!(cache.member(10, 3).is_some());

        cache
            .handle_event(
                "GUILD_MEMBER_LIST_UPDATE",
                Some(&json!({"guild_id": "10", "ops": [{"op": "DELETE", "index": 2}]})),
            )
            .expect("delete");
        assert!(cache.member(10, 1).is_none());
        assert!(cache.member(10, 2).is_some());
    }

    #[test]
    fn channels_without_guild_appear_as_private_channels_with_recipient_names() {
        let mut cache = StateCache::new();
        cache
            .handle_event(
                "READY",
                Some(&json!({
                    "user": {"id": "1", "username": "me"},
                    "guilds": [],
                    "private_channels": [{
                        "id": "900", "type": 1, "last_message_id": null,
                        "recipients": [{"id": "77", "username": "bob", "global_name": "Bob"}]
                    }]
                })),
            )
            .expect("ready");
        let dms = cache.private_channels();
        assert_eq!(dms.len(), 1);
        assert_eq!(dms[0].id, 900);
        assert_eq!(dms[0].recipients[0].global_name.as_deref(), Some("Bob"));
    }

    #[test]
    fn member_list_rows_keep_groups_members_and_counts() {
        let mut cache = StateCache::new();
        cache
            .handle_event(
                "READY",
                Some(&json!({"user": {"id": "1", "username": "me"}, "guilds": [guild(10)]})),
            )
            .expect("ready");
        cache
            .handle_event(
                "GUILD_MEMBER_LIST_UPDATE",
                Some(&json!({
                    "guild_id": "10", "online_count": 3, "member_count": 9,
                    "ops": [{"op": "SYNC", "range": [0, 2], "items": [
                        {"group": {"id": "online", "count": 3}},
                        {"member": {"user": {"id": "5", "username": "eve"}, "roles": []}},
                        {"member": {"user": {"id": "6", "username": "fox"}, "roles": []}}
                    ]}]
                })),
            )
            .expect("sync");
        let state = cache.member_list(10).expect("member list");
        assert_eq!(state.online_count, Some(3));
        assert_eq!(state.member_count, Some(9));
        assert_eq!(
            state.rows,
            vec![
                MemberListRow::Group {
                    id: "online".to_owned(),
                    count: Some(3)
                },
                MemberListRow::Member(5),
                MemberListRow::Member(6),
            ]
        );
    }

    #[test]
    fn one_undecodable_channel_does_not_hide_the_rest_of_the_guild() {
        let mut cache = StateCache::new();
        cache
            .handle_event(
                "READY",
                Some(&json!({"user": {"id": "1", "username": "me"}, "guilds": [guild(10)]})),
            )
            .expect("ready");
        let result = cache.handle_event(
            "GUILD_CREATE",
            Some(&json!({
                "id": "10", "name": "g10",
                "channels": [
                    {"id": "100", "type": 0, "name": "general", "position": 0},
                    {"type": 0, "name": "broken, no id"},
                    {"id": "102", "type": 0, "name": "after-the-bad-one", "position": 2}
                ]
            })),
        );
        assert!(matches!(
            result,
            Err(CacheError::SkippedEntries {
                event: "GUILD_CREATE",
                count: 1
            })
        ));
        let names: Vec<String> = cache
            .channels_for_guild(10)
            .iter()
            .filter_map(|channel| channel.name.clone())
            .collect();
        assert_eq!(
            names,
            vec!["general".to_owned(), "after-the-bad-one".to_owned()]
        );
    }

    #[test]
    fn persisted_cache_round_trips_guilds_channels_and_users() {
        let mut cache = StateCache::new();
        cache
            .handle_event(
                "READY",
                Some(&json!({
                    "user": {"id": "1", "username": "me"},
                    "guilds": [guild(10)],
                    "users": [{"id": "55", "username": "alice", "avatar": "av"}]
                })),
            )
            .expect("ready");
        cache
            .handle_event(
                "GUILD_CREATE",
                Some(&json!({"id": "10", "name": "g10", "channels": [{"id": "100", "type": 0, "name": "general"}]})),
            )
            .expect("guild create");
        let bytes = cache.persisted().to_bytes().expect("serialize");
        let persisted = PersistedCache::from_bytes(&bytes).expect("deserialize");

        let mut restored = StateCache::new();
        restored.restore(&persisted);
        assert_eq!(restored.guild(10).map(|guild| guild.id), Some(10));
        assert_eq!(restored.channels_for_guild(10).len(), 1);
        assert_eq!(
            restored
                .user(55)
                .and_then(|user| user.avatar.clone())
                .as_deref(),
            Some("av")
        );
    }

    #[test]
    fn relationships_survive_a_persist_and_restore() {
        let mut cache = StateCache::new();
        cache
            .handle_event(
                "READY",
                Some(&json!({
                    "user": {"id": "1", "username": "me"},
                    "guilds": [],
                    "relationships": [{"id": "55", "type": 1, "user": {"id": "55", "username": "alice"}}]
                })),
            )
            .expect("ready");
        cache
            .handle_event("RELATIONSHIP_REMOVE", Some(&json!({"id": "55"})))
            .expect("remove");
        cache
            .handle_event("RELATIONSHIP_ADD", Some(&json!({"id": "66", "type": 1})))
            .expect("add");
        let bytes = cache.persisted().to_bytes().expect("serialize");
        let persisted = PersistedCache::from_bytes(&bytes).expect("deserialize");
        let mut restored = StateCache::new();
        restored.restore(&persisted);
        assert_eq!(
            restored
                .relationships
                .get(&66)
                .map(|relationship| relationship.kind),
            Some(1)
        );
        assert!(restored.relationships.get(&55).is_none());
    }

    #[test]
    fn partial_user_update_keeps_the_cached_avatar() {
        let mut cache = StateCache::new();
        cache
            .handle_event(
                "READY",
                Some(&json!({
                    "user": {"id": "1", "username": "me"},
                    "guilds": [guild(10)],
                    "users": [{"id": "55", "username": "alice", "avatar": "av"}]
                })),
            )
            .expect("ready");
        cache
            .handle_event(
                "READY",
                Some(&json!({
                    "user": {"id": "1", "username": "me"},
                    "guilds": [guild(10)],
                    "users": [{"id": "55", "username": "alice"}]
                })),
            )
            .expect("second ready");
        assert_eq!(
            cache
                .user(55)
                .and_then(|user| user.avatar.clone())
                .as_deref(),
            Some("av")
        );
    }

    #[test]
    fn lazy_request_payload_uses_opcode_14_and_string_ids() {
        let payload = crate::gateway_client::lazy_request_payload(10, 11, &[[0, 99]]);
        assert_eq!(payload["op"], 14);
        assert_eq!(payload["d"]["guild_id"], "10");
        assert_eq!(payload["d"]["channels"]["11"], json!([[0, 99]]));
    }
}
