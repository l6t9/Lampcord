use std::collections::HashMap;

use serde_json::Value;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChannelSettings {
    pub muted: bool,
    pub message_notifications: Option<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GuildSettings {
    pub muted: bool,
    pub message_notifications: Option<u8>,
    pub channel_overrides: HashMap<u64, ChannelSettings>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UserGuildSettingsStore {
    version: Option<i64>,
    guilds: HashMap<u64, GuildSettings>,
}

impl UserGuildSettingsStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn version(&self) -> Option<i64> {
        self.version
    }

    pub fn guild(&self, guild_id: u64) -> Option<&GuildSettings> {
        self.guilds.get(&guild_id)
    }

    pub fn is_channel_muted(&self, guild_id: u64, channel_id: u64) -> bool {
        let Some(guild) = self.guilds.get(&guild_id) else {
            return false;
        };
        guild
            .channel_overrides
            .get(&channel_id)
            .map(|settings| settings.muted)
            .unwrap_or(guild.muted)
    }

    pub fn handle_ready(&mut self, ready: &Value) {
        let Some(value) = ready.get("user_guild_settings") else {
            return;
        };
        let entries = value
            .as_array()
            .or_else(|| value.get("entries")?.as_array());
        let Some(entries) = entries else {
            return;
        };
        self.version = value.get("version").and_then(Value::as_i64);
        self.guilds = entries.iter().filter_map(parse_guild).collect();
    }

    pub fn handle_update(&mut self, data: Option<&Value>) {
        let Some((guild_id, settings)) = data.and_then(parse_guild) else {
            return;
        };
        self.guilds.insert(guild_id, settings);
    }

    pub fn clear(&mut self) {
        self.version = None;
        self.guilds.clear();
    }
}

fn parse_guild(value: &Value) -> Option<(u64, GuildSettings)> {
    let guild_id = parse_u64(value.get("guild_id"))?;
    let channel_overrides = value
        .get("channel_overrides")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(parse_channel)
        .collect();
    Some((
        guild_id,
        GuildSettings {
            muted: value.get("muted").and_then(Value::as_bool).unwrap_or(false),
            message_notifications: parse_notifications(value),
            channel_overrides,
        },
    ))
}

fn parse_channel(value: &Value) -> Option<(u64, ChannelSettings)> {
    let channel_id = parse_u64(value.get("channel_id"))?;
    Some((
        channel_id,
        ChannelSettings {
            muted: value.get("muted").and_then(Value::as_bool).unwrap_or(false),
            message_notifications: parse_notifications(value),
        },
    ))
}

fn parse_notifications(value: &Value) -> Option<u8> {
    value
        .get("message_notifications")
        .and_then(Value::as_u64)
        .and_then(|level| u8::try_from(level).ok())
}

fn parse_u64(value: Option<&Value>) -> Option<u64> {
    let value = value?;
    value
        .as_u64()
        .or_else(|| value.as_str()?.parse::<u64>().ok())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn seeds_versioned_settings_from_ready() {
        let ready = json!({
            "user_guild_settings": {
                "version": 4,
                "entries": [
                    {
                        "guild_id": "10",
                        "muted": true,
                        "message_notifications": 1,
                        "channel_overrides": [
                            {"channel_id": "20", "muted": false, "message_notifications": 0}
                        ]
                    },
                    {"guild_id": null, "muted": false}
                ]
            }
        });
        let mut store = UserGuildSettingsStore::new();
        store.handle_ready(&ready);

        assert_eq!(store.version(), Some(4));
        assert!(store.is_channel_muted(10, 30));
        assert!(!store.is_channel_muted(10, 20));
        assert_eq!(store.guild(10).map(|guild| guild.muted), Some(true));
    }

    #[test]
    fn applies_single_guild_updates() {
        let mut store = UserGuildSettingsStore::new();
        store.handle_update(Some(&json!({"guild_id": "10", "muted": true})));
        assert!(store.is_channel_muted(10, 1));

        store.handle_update(Some(&json!({"guild_id": "10", "muted": false})));
        assert!(!store.is_channel_muted(10, 1));
    }
}
