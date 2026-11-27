use std::collections::HashMap;

use requiem_ui::signal::{Signal, signal};

use super::{Action, GuildData, Store};
use crate::models::id::marker::{EmojiMarker, GuildMarker};
use crate::models::{CustomEmoji, Id};

#[derive(Clone)]
pub struct EmojiStore {
    pub by_guild: Signal<HashMap<Id<GuildMarker>, Vec<CustomEmoji>>>,
}

impl Default for EmojiStore {
    fn default() -> Self {
        Self::new()
    }
}

impl EmojiStore {
    pub fn new() -> Self {
        Self {
            by_guild: signal(HashMap::new()),
        }
    }

    pub fn find(&self, id: Id<EmojiMarker>) -> Option<(Id<GuildMarker>, CustomEmoji)> {
        self.by_guild.with_untracked(|by_guild| {
            by_guild.iter().find_map(|(guild, emojis)| {
                emojis
                    .iter()
                    .find(|e| e.id == id)
                    .map(|e| (*guild, e.clone()))
            })
        })
    }

    fn set(&self, guild_id: Id<GuildMarker>, emojis: Vec<CustomEmoji>) {
        self.by_guild.update(|by_guild| {
            by_guild.insert(guild_id, emojis);
        });
    }

    fn set_from(&self, data: &GuildData) {
        self.set(data.guild.id, data.emojis.clone());
    }
}

impl Store for EmojiStore {
    fn handle(&self, action: &Action) {
        match action {
            Action::ConnectionOpen { guilds, .. } => guilds.iter().for_each(|g| self.set_from(g)),
            Action::GuildCreate(data) => self.set_from(data),
            Action::GuildEmojisUpdate { guild_id, emojis } => self.set(*guild_id, emojis.clone()),
            _ => {}
        }
    }

    fn reset(&self) {
        self.by_guild.set(HashMap::new());
    }
}
