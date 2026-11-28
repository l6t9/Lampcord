use crate::utils::time;
use std::cell;
use std::collections::HashMap;
use std::rc;

use requiem_ui::signal::{Signal, signal};

use super::wire::{MuteConfigWire, UserGuildSettingsEntry};
use super::{Action, Store};
use crate::models::Id;
use crate::models::id::marker::{ChannelMarker, GuildMarker};

#[derive(Clone, Debug, PartialEq)]
pub struct GuildSettings {
    pub muted: Mute,

    pub message_notifications: u8,
    pub suppress_everyone: bool,
    pub suppress_roles: bool,
    pub hide_muted_channels: bool,
    pub mobile_push: bool,
    pub mute_scheduled_events: bool,

    pub notify_highlights: u8,
    pub flags: u64,

    pub version: u64,
    pub overrides: HashMap<Id<ChannelMarker>, ChannelOverride>,
}

impl Default for GuildSettings {
    fn default() -> Self {
        Self {
            muted: Mute::default(),
            message_notifications: 3,
            suppress_everyone: false,
            suppress_roles: false,
            hide_muted_channels: false,
            mobile_push: true,
            mute_scheduled_events: false,
            notify_highlights: 0,
            flags: 0,
            version: 0,
            overrides: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChannelOverride {
    pub muted: Mute,

    pub message_notifications: u8,

    pub flags: u64,
}

impl Default for ChannelOverride {
    fn default() -> Self {
        Self {
            muted: Mute::default(),
            message_notifications: 3,
            flags: 0,
        }
    }
}

/// Channel override and guild flags that choose what counts as an unread, regardless of
/// the notification level (docs.discord.food, User Guild Settings).
const CHANNEL_UNREADS_ONLY_MENTIONS: u64 = 1 << 9;
const CHANNEL_UNREADS_ALL_MESSAGES: u64 = 1 << 10;
const GUILD_UNREADS_ALL_MESSAGES: u64 = 1 << 11;
const GUILD_UNREADS_ONLY_MENTIONS: u64 = 1 << 12;

impl ChannelOverride {
    pub fn is_set(&self) -> bool {
        self.message_notifications != 3 || self.muted.muted
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mute {
    pub muted: bool,
    pub until_ms: Option<i64>,

    pub window: Option<i64>,
}

impl Mute {
    fn from_wire(muted: bool, config: Option<&MuteConfigWire>) -> Self {
        let until_ms = config
            .and_then(|config| config.end_time.as_deref())
            .and_then(|time| chrono::DateTime::parse_from_rfc3339(time).ok())
            .map(|time| time.timestamp_millis());
        Self {
            muted,
            until_ms,
            window: config.and_then(|c| c.selected_time_window),
        }
    }

    pub fn is_active(self, now_ms: i64) -> bool {
        self.muted && self.until_ms.is_none_or(|until| until > now_ms)
    }
}

impl GuildSettings {
    fn from_wire(entry: &UserGuildSettingsEntry) -> Self {
        let overrides = entry
            .channel_overrides
            .iter()
            .map(|item| {
                (
                    item.channel_id,
                    ChannelOverride {
                        muted: Mute::from_wire(item.muted, item.mute_config.as_ref()),
                        message_notifications: item.message_notifications.unwrap_or(3),
                        flags: item.flags,
                    },
                )
            })
            .collect();
        Self {
            muted: Mute::from_wire(entry.muted, entry.mute_config.as_ref()),
            message_notifications: entry.message_notifications.unwrap_or(3),
            suppress_everyone: entry.suppress_everyone,
            suppress_roles: entry.suppress_roles,
            hide_muted_channels: entry.hide_muted_channels,
            mobile_push: entry.mobile_push.unwrap_or(true),
            mute_scheduled_events: entry.mute_scheduled_events,
            notify_highlights: entry.notify_highlights,
            flags: entry.flags,
            version: entry.version,
            overrides,
        }
    }
}

#[derive(Clone)]
pub struct GuildSettingsStore {
    pub by_guild: Signal<HashMap<Option<Id<GuildMarker>>, GuildSettings>>,

    /// The user-wide `USE_NEW_NOTIFICATIONS` flag: unreads are separate from the notification level.
    new_notifications: rc::Rc<cell::Cell<bool>>,
}

const USE_NEW_NOTIFICATIONS: u64 = 1 << 4;

impl Default for GuildSettingsStore {
    fn default() -> Self {
        Self::new()
    }
}

impl GuildSettingsStore {
    pub fn new() -> Self {
        Self {
            by_guild: signal(HashMap::new()),
            new_notifications: Default::default(),
        }
    }

    pub fn set_new_notifications(&self, on: bool) {
        self.new_notifications.set(on);
    }

    pub fn is_guild_muted(&self, guild_id: Id<GuildMarker>) -> bool {
        let now = time::now_ms() as i64;
        self.by_guild.with(|all| {
            all.get(&Some(guild_id))
                .is_some_and(|settings| settings.muted.is_active(now))
        })
    }

    pub fn is_channel_muted(
        &self,
        guild_id: Option<Id<GuildMarker>>,
        channel_id: Id<ChannelMarker>,
        parent_id: Option<Id<ChannelMarker>>,
    ) -> bool {
        let now = time::now_ms() as i64;
        self.by_guild.with(|all| {
            let Some(settings) = all.get(&guild_id) else {
                return false;
            };
            let own = |id: Id<ChannelMarker>| {
                settings
                    .overrides
                    .get(&id)
                    .is_some_and(|o| o.muted.is_active(now))
            };
            (guild_id.is_some() && settings.muted.is_active(now))
                || own(channel_id)
                || parent_id.is_some_and(own)
        })
    }

    /// Muting a server suppresses notifications there, but it must not visually dim every
    /// channel. Only a channel or its category override gets that treatment.
    pub fn is_channel_individually_muted(
        &self,
        guild_id: Option<Id<GuildMarker>>,
        channel_id: Id<ChannelMarker>,
        parent_id: Option<Id<ChannelMarker>>,
    ) -> bool {
        let now = time::now_ms() as i64;
        self.by_guild.with(|all| {
            let Some(settings) = all.get(&guild_id) else {
                return false;
            };
            let own = |id: Id<ChannelMarker>| {
                settings
                    .overrides
                    .get(&id)
                    .is_some_and(|o| o.muted.is_active(now))
            };
            own(channel_id) || parent_id.is_some_and(own)
        })
    }

    /// Whether an unread here is shown as important (the bright pill and name). The unread
    /// flags win; without them unreads follow the notification level.
    pub fn unreads_are_important(
        &self,
        guild_id: Option<Id<GuildMarker>>,
        channel_id: Id<ChannelMarker>,
        kind: Option<u8>,
        parent_id: Option<Id<ChannelMarker>>,
        guild_default: u8,
    ) -> bool {
        // Discord's `resolveUnreadSetting`: threads and DMs always count every message, and so
        // does everything until the user turns on "new notifications".
        if !self.new_notifications.get() || kind.is_some_and(|kind| matches!(kind, 1 | 3 | 10..=12)) {
            return true;
        }
        let flagged = self.by_guild.with(|all| {
            let settings = all.get(&guild_id)?;
            let channel_flags = |id: Id<ChannelMarker>| {
                let flags = settings.overrides.get(&id).map_or(0, |o| o.flags);
                if flags & CHANNEL_UNREADS_ALL_MESSAGES != 0 {
                    Some(true)
                } else if flags & CHANNEL_UNREADS_ONLY_MENTIONS != 0 {
                    Some(false)
                } else {
                    None
                }
            };
            channel_flags(channel_id)
                .or_else(|| parent_id.and_then(channel_flags))
                .or(if settings.flags & GUILD_UNREADS_ALL_MESSAGES != 0 {
                    Some(true)
                } else if settings.flags & GUILD_UNREADS_ONLY_MENTIONS != 0 {
                    Some(false)
                } else {
                    None
                })
        });
        flagged.unwrap_or_else(|| {
            self.resolved_message_notifications(guild_id, channel_id, parent_id, guild_default) == 0
        })
    }

    pub fn resolved_message_notifications(
        &self,
        guild_id: Option<Id<GuildMarker>>,
        channel_id: Id<ChannelMarker>,
        parent_id: Option<Id<ChannelMarker>>,
        guild_default: u8,
    ) -> u8 {
        self.by_guild.with(|all| {
            let Some(settings) = all.get(&guild_id) else {
                return guild_default;
            };
            let set = |level: u8| (level != 3).then_some(level);
            settings
                .overrides
                .get(&channel_id)
                .and_then(|o| set(o.message_notifications))
                .or_else(|| {
                    parent_id
                        .and_then(|parent| settings.overrides.get(&parent))
                        .and_then(|o| set(o.message_notifications))
                })
                .or_else(|| set(settings.message_notifications))
                .unwrap_or(guild_default)
        })
    }

    pub fn suppresses_everyone(&self, guild_id: Option<Id<GuildMarker>>) -> bool {
        self.by_guild.with(|all| {
            all.get(&guild_id)
                .is_some_and(|settings| settings.suppress_everyone)
        })
    }

    pub fn suppresses_roles(&self, guild_id: Option<Id<GuildMarker>>) -> bool {
        self.by_guild.with(|all| {
            all.get(&guild_id)
                .is_some_and(|settings| settings.suppress_roles)
        })
    }

    pub fn hides_muted_channels(&self, guild_id: Id<GuildMarker>) -> bool {
        self.by_guild.with(|all| {
            all.get(&Some(guild_id))
                .is_some_and(|settings| settings.hide_muted_channels)
        })
    }

    pub fn of(&self, guild_id: Id<GuildMarker>) -> GuildSettings {
        self.by_guild
            .with(|all| all.get(&Some(guild_id)).cloned().unwrap_or_default())
    }

    pub fn guild_mute(&self, guild_id: Id<GuildMarker>) -> Option<Mute> {
        let now = time::now_ms() as i64;
        self.by_guild.with(|all| {
            all.get(&Some(guild_id))
                .map(|s| s.muted)
                .filter(|m| m.is_active(now))
        })
    }

    pub fn channel_override(
        &self,
        guild_id: Id<GuildMarker>,
        channel_id: Id<ChannelMarker>,
    ) -> ChannelOverride {
        self.by_guild.with(|all| {
            all.get(&Some(guild_id))
                .and_then(|s| s.overrides.get(&channel_id))
                .cloned()
                .unwrap_or_default()
        })
    }

    pub fn channel_mute(
        &self,
        guild_id: Id<GuildMarker>,
        channel_id: Id<ChannelMarker>,
    ) -> Option<Mute> {
        let now = time::now_ms() as i64;
        Some(self.channel_override(guild_id, channel_id).muted).filter(|m| m.is_active(now))
    }

    pub fn guild_level(&self, guild_id: Id<GuildMarker>, guild_default: u8) -> u8 {
        let level = self.of(guild_id).message_notifications;
        if level == 3 { guild_default } else { level }
    }

    pub fn apply_local(&self, guild_id: Id<GuildMarker>, change: impl FnOnce(&mut GuildSettings)) {
        self.by_guild
            .update(|all| change(all.entry(Some(guild_id)).or_default()));
    }

    pub fn replace(&self, entry: &UserGuildSettingsEntry) {
        self.by_guild.update(|all| {
            if all
                .get(&entry.guild_id)
                .is_some_and(|current| current.version > entry.version)
            {
                return;
            }
            all.insert(entry.guild_id, GuildSettings::from_wire(entry));
        });
    }
}

impl Store for GuildSettingsStore {
    fn handle(&self, action: &Action) {
        match action {
            Action::ConnectionOpen { ready, .. } => {
                let entries = ready
                    .user_guild_settings
                    .iter()
                    .flat_map(|settings| settings.entries.iter());
                let all = entries
                    .map(|entry| {
                        log::debug!(
                            "guild settings: guild={:?} notifications={:?} flags={} muted={} overrides={} (flagged: {})",
                            entry.guild_id,
                            entry.message_notifications,
                            entry.flags,
                            entry.muted,
                            entry.channel_overrides.len(),
                            entry.channel_overrides.iter().filter(|o| o.flags != 0).count()
                        );
                        (entry.guild_id, GuildSettings::from_wire(entry))
                    })
                    .collect();
                self.by_guild.set(all);
                self.set_new_notifications(
                    ready
                        .notification_flags
                        .is_some_and(|flags| flags & USE_NEW_NOTIFICATIONS != 0),
                );
            }
            Action::NotificationSettingsUpdate(settings) => {
                self.set_new_notifications(settings.flags & USE_NEW_NOTIFICATIONS != 0)
            }
            Action::UserGuildSettingsUpdate(entry) => self.replace(entry),
            _ => {}
        }
    }

    fn reset(&self) {
        self.by_guild.set(HashMap::new());
    }
}

#[cfg(test)]
mod tests {
    use crate::models;
    use crate::state::wire;
    use serde_json::json;

    use super::*;

    fn entry(value: serde_json::Value) -> UserGuildSettingsEntry {
        serde_json::from_str(&value.to_string()).unwrap()
    }

    fn store_with(entries: serde_json::Value) -> GuildSettingsStore {
        let store = GuildSettingsStore::new();
        let ready = wire::ReadyExtras {
            user_guild_settings: Some(wire::UserGuildSettingsWire {
                entries: entries
                    .as_array()
                    .into_iter()
                    .flatten()
                    .cloned()
                    .map(entry)
                    .collect(),
            }),
            ..Default::default()
        };
        let user = models::User {
            id: Id::new(1),
            username: "me".into(),
            discriminator: "0".into(),
            global_name: None,
            avatar: None,
            email: None,
            phone: None,
            verified: None,
            mfa_enabled: None,
            bot: false,
            premium_type: None,
            primary_guild: None,
            avatar_decoration_data: None,
            collectibles: None,
            display_name_styles: None,
        };
        store.handle(&Action::ConnectionOpen {
            user,
            guilds: vec![],
            auth_session_id_hash: None,
            user_settings_proto: None,
            ready,
        });
        store
    }

    fn id<T>(n: u64) -> Id<T> {
        Id::new(n)
    }

    #[test]
    fn a_muted_server_mutes_its_channels_and_a_channel_override_mutes_just_that_one() {
        let store = store_with(json!([
            { "guild_id": "10", "muted": true, "channel_overrides": [] },
            { "guild_id": "20", "muted": false, "channel_overrides": [{ "channel_id": "21", "muted": true }, { "channel_id": "22", "muted": false }] },
        ]));
        assert!(store.is_guild_muted(id(10)));
        assert!(store.is_channel_muted(Some(id(10)), id(11), None));
        assert!(!store.is_guild_muted(id(20)));
        assert!(store.is_channel_muted(Some(id(20)), id(21), None));
        assert!(!store.is_channel_muted(Some(id(20)), id(22), None));
        assert!(
            !store.is_channel_muted(Some(id(30)), id(31), None),
            "a server with no entry mutes nothing"
        );
    }

    #[test]
    fn a_muted_server_does_not_individually_mute_its_channels() {
        let store = store_with(json!([
            { "guild_id": "10", "muted": true, "channel_overrides": [] },
            { "guild_id": "20", "muted": false, "channel_overrides": [{ "channel_id": "21", "muted": true }, { "channel_id": "50", "muted": true }] },
        ]));
        assert!(store.is_guild_muted(id(10)));
        assert!(store.is_channel_muted(Some(id(10)), id(11), None));
        assert!(!store.is_channel_individually_muted(Some(id(10)), id(11), None));

        assert!(store.is_channel_individually_muted(Some(id(20)), id(21), None));
        assert!(store.is_channel_individually_muted(Some(id(20)), id(22), Some(id(50))));
        assert!(!store.is_channel_individually_muted(Some(id(20)), id(22), Some(id(51))));
    }

    #[test]
    fn a_muted_category_mutes_the_channels_under_it() {
        let store = store_with(
            json!([{ "guild_id": "20", "muted": false, "channel_overrides": [{ "channel_id": "50", "muted": true }] }]),
        );
        assert!(store.is_channel_muted(Some(id(20)), id(21), Some(id(50))));
        assert!(!store.is_channel_muted(Some(id(20)), id(21), Some(id(51))));
    }

    #[test]
    fn a_timed_mute_stops_once_its_time_has_passed() {
        let store = store_with(json!([
            { "guild_id": "10", "muted": true, "mute_config": { "end_time": "2000-01-01T00:00:00+00:00", "selected_time_window": 3600 } },
            { "guild_id": "20", "muted": true, "mute_config": { "end_time": "2999-01-01T00:00:00+00:00", "selected_time_window": 3600 } },
        ]));
        assert!(!store.is_guild_muted(id(10)), "ran out");
        assert!(store.is_guild_muted(id(20)));
    }

    #[test]
    fn dm_settings_have_no_guild_and_a_dm_is_only_muted_by_its_own_override() {
        let store = store_with(
            json!([{ "guild_id": null, "muted": true, "channel_overrides": [{ "channel_id": "50", "muted": true }] }]),
        );
        assert!(store.is_channel_muted(None, id(50), None));
        assert!(
            !store.is_channel_muted(None, id(51), None),
            "the DM entry's own muted flag isn't a server mute"
        );
    }

    #[test]
    fn the_notification_level_comes_from_the_channel_then_its_category_then_the_server_then_the_servers_default()
     {
        let store = store_with(json!([{
            "guild_id": "10", "message_notifications": 1,
            "channel_overrides": [{ "channel_id": "11", "message_notifications": 0 }, { "channel_id": "50", "message_notifications": 2 }, { "channel_id": "13", "message_notifications": 3 }],
        }, { "guild_id": "20", "message_notifications": 3 }]));
        let level = |guild: u64, channel: u64, parent: Option<u64>, default: u8| {
            store.resolved_message_notifications(
                Some(id(guild)),
                id(channel),
                parent.map(id),
                default,
            )
        };
        assert_eq!(level(10, 11, None, 0), 0, "the channel's own");
        assert_eq!(level(10, 12, Some(50), 0), 2, "its category's");
        assert_eq!(level(10, 12, None, 0), 1, "the server's");
        assert_eq!(
            level(10, 13, None, 0),
            1,
            "an override of 3 means no setting of its own"
        );
        assert_eq!(
            level(20, 21, None, 1),
            1,
            "the server's default when it set none"
        );
        assert_eq!(level(30, 31, None, 0), 0, "a server with no entry");
    }

    #[test]
    fn suppression_and_updates() {
        let store = store_with(
            json!([{ "guild_id": "10", "suppress_everyone": true, "suppress_roles": false }]),
        );
        assert!(store.suppresses_everyone(Some(id(10))));
        assert!(!store.suppresses_roles(Some(id(10))));
        store.handle(&Action::UserGuildSettingsUpdate(entry(json!({ "guild_id": "10", "suppress_everyone": false, "suppress_roles": true, "muted": true }))));
        assert!(!store.suppresses_everyone(Some(id(10))));
        assert!(store.suppresses_roles(Some(id(10))));
        assert!(store.is_guild_muted(id(10)));
        store.reset();
        assert!(!store.is_guild_muted(id(10)));
    }
}
