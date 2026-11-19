use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use lampcord_model::{
    Snowflake,
    channel::{Channel, ChannelType},
    guild::{Guild, UnavailableGuild},
    message::Message,
    user::User,
};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{event_dispatch::GatewayEventHandler, read_state::ReadStateStore};

pub const MAX_MESSAGES_PER_CHANNEL: usize = 300;

const HANDLED_EVENTS: &[&str] = &[
    "READY",
    "MESSAGE_CREATE",
    "GUILD_CREATE",
    "GUILD_UPDATE",
    "GUILD_DELETE",
    "CHANNEL_CREATE",
    "CHANNEL_UPDATE",
    "CHANNEL_DELETE",
    "USER_UPDATE",
];

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
    channels: HashMap<Snowflake, Channel>,
    private_channel_ids: Vec<Snowflake>,
    messages: HashMap<Snowflake, Vec<Message>>,
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

    pub fn channels_for_guild(&self, guild_id: Snowflake) -> Vec<&Channel> {
        let mut channels: Vec<&Channel> = self
            .channels
            .values()
            .filter(|channel| channel.guild_id == Some(guild_id))
            .collect();
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

    pub fn read_state(&self) -> &ReadStateStore {
        &self.read_state
    }

    pub fn handle_event(
        &mut self,
        event_type: &str,
        data: Option<&Value>,
    ) -> Result<(), CacheError> {
        match event_type {
            "READY" => self.handle_ready(data),
            "MESSAGE_CREATE" => self.handle_message_create(data),
            "GUILD_CREATE" => self.handle_guild_create(data),
            "GUILD_UPDATE" => self.handle_guild_update(data),
            "GUILD_DELETE" => self.handle_guild_delete(data),
            "CHANNEL_CREATE" | "CHANNEL_UPDATE" => self.handle_channel(data),
            "CHANNEL_DELETE" => self.handle_channel_delete(data),
            "USER_UPDATE" => self.handle_user_update(data),
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

        let mut guild_order = Vec::new();
        if let Some(guilds) = data.get("guilds").and_then(Value::as_array) {
            for value in guilds {
                let unavailable: UnavailableGuild = decode(value)?;
                self.merge_guild(Guild {
                    id: unavailable.id,
                    name: None,
                    icon: None,
                    splash: None,
                    banner: None,
                    owner_id: None,
                    permissions: None,
                    member_count: None,
                    unavailable: Some(unavailable.unavailable),
                });
                if !guild_order.contains(&unavailable.id) {
                    guild_order.push(unavailable.id);
                }
            }
        }
        self.guild_order = guild_order;

        self.private_channel_ids.clear();
        for key in ["channels", "private_channels"] {
            if let Some(channels) = data.get(key).and_then(Value::as_array) {
                for value in channels {
                    let channel: Channel = decode(value)?;
                    self.update_channel(channel);
                }
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

        let mention_count = self.mention_count(&message);
        self.read_state
            .record_message(channel_id, message.id, mention_count);
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

        let channel_ids: Vec<Snowflake> = self
            .channels
            .values()
            .filter(|channel| channel.guild_id == Some(guild_id))
            .map(|channel| channel.id)
            .collect();
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
        let channel = match self.channels.get(&incoming.id) {
            Some(existing) => merge_channel(existing, incoming),
            None => incoming,
        };
        self.channels.insert(channel.id, channel.clone());
        if channel.guild_id.is_none() && !self.private_channel_ids.contains(&channel.id) {
            self.private_channel_ids.push(channel.id);
        }
    }

    fn remove_channel(&mut self, channel_id: Snowflake) {
        self.channels.remove(&channel_id);
        self.private_channel_ids.retain(|id| *id != channel_id);
        self.messages.remove(&channel_id);
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
        self.channels.clear();
        self.private_channel_ids.clear();
        self.messages.clear();
        self.read_state.clear();
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
