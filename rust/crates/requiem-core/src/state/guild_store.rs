use requiem_ui::signal::{Signal, signal};

use super::{Action, GuildData, Store};
use crate::models::id::marker::{GuildMarker, UserMarker};
use crate::models::{Guild, Id};

#[derive(Clone)]
pub struct GuildStore {
    pub list: Signal<Vec<Guild>>,
}

impl Default for GuildStore {
    fn default() -> Self {
        Self::new()
    }
}

impl GuildStore {
    pub fn new() -> Self {
        Self {
            list: signal(Vec::new()),
        }
    }

    pub fn find(&self, id: Id<GuildMarker>) -> Option<Guild> {
        self.list
            .with_untracked(|guilds| guilds.iter().find(|g| g.id == id).cloned())
    }

    pub fn owner_of(&self, id: Id<GuildMarker>) -> Option<Id<UserMarker>> {
        self.list
            .with_untracked(|guilds| guilds.iter().find(|g| g.id == id).and_then(|g| g.owner_id))
    }

    fn add(&self, data: &GuildData) {
        self.list.update(|guilds| {
            if !guilds.iter().any(|g| g.id == data.guild.id) {
                guilds.push(data.guild.clone());
            }
        });
    }
}

impl Store for GuildStore {
    fn handle(&self, action: &Action) {
        match action {
            Action::ConnectionOpen { guilds, .. } => guilds.iter().for_each(|g| self.add(g)),
            Action::GuildCreate(data) => self.add(data),

            Action::GuildDelete(id) => self.list.update(|guilds| guilds.retain(|g| g.id != *id)),
            Action::GuildUpdate(updated) => {
                let changed = self.list.with_untracked(|guilds| {
                    guilds
                        .iter()
                        .find(|g| g.id == updated.id)
                        .is_some_and(|guild| guild != &*updated)
                });
                if changed {
                    self.list.update(|guilds| {
                        if let Some(guild) = guilds.iter_mut().find(|g| g.id == updated.id) {
                            *guild = updated.clone();
                        }
                    });
                }
            }
            _ => {}
        }
    }

    fn reset(&self) {
        self.list.set(Vec::new());
    }
}
