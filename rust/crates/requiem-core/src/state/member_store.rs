use crate::gateway::member_requests;
use crate::presence;
use requiem_ui::widget::primitives;
use std::collections::HashMap;
use std::rc::Rc;

use requiem_ui::signal::{Signal, signal};

use super::member_list::MemberList;
use super::wire::{ListOpKind, MemberListUpdate};
use super::{Action, Store};
use crate::models::id::marker::{GuildMarker, UserMarker};
use crate::models::{GuildMember, Id};

#[derive(Clone)]
pub struct MemberStore {
    pub by_guild: Signal<HashMap<Id<GuildMarker>, HashMap<Id<UserMarker>, GuildMember>>>,

    pub lists: Signal<HashMap<Id<GuildMarker>, HashMap<String, Rc<MemberList>>>>,

    active_ids: Signal<HashMap<Id<GuildMarker>, String>>,
}

impl Default for MemberStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemberStore {
    pub fn new() -> Self {
        Self {
            by_guild: signal(HashMap::new()),
            lists: signal(HashMap::new()),
            active_ids: signal(HashMap::new()),
        }
    }

    pub fn active_list(&self, guild_id: Id<GuildMarker>) -> Option<Rc<MemberList>> {
        let id = self
            .active_ids
            .with(|active| active.get(&guild_id).cloned())?;
        self.lists.with(|lists| {
            lists
                .get(&guild_id)
                .and_then(|lists| lists.get(&id))
                .cloned()
        })
    }

    pub fn nick_of(&self, guild_id: Id<GuildMarker>, user_id: Id<UserMarker>) -> Option<String> {
        self.by_guild
            .with_untracked(|all| all.get(&guild_id)?.get(&user_id)?.nick.clone())
    }

    pub fn presence_of(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> Option<primitives::PresenceStatus> {
        self.lists.with_untracked(|lists| {
            lists
                .get(&guild_id)?
                .values()
                .find_map(|list| list.status_of(user_id))
        })
    }

    pub fn custom_status_of(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> Option<presence::CustomStatus> {
        self.lists.with_untracked(|lists| {
            lists
                .get(&guild_id)?
                .values()
                .find_map(|list| list.custom_status_of(user_id))
        })
    }

    fn apply_list_update(&self, update: &MemberListUpdate) {
        let guild_id = update.guild_id;
        if let Some(list_id) = update.id.as_deref() {
            self.lists.update(|lists| {
                let slot = lists
                    .entry(guild_id)
                    .or_default()
                    .entry(list_id.to_string())
                    .or_insert_with(|| {
                        Rc::new(MemberList {
                            id: list_id.to_string(),
                            ..Default::default()
                        })
                    });

                Rc::make_mut(slot).apply(update);
            });
            if update.is_sync()
                && self.active_ids.with_untracked(|active| {
                    active.get(&guild_id).map(String::as_str) != Some(list_id)
                })
            {
                self.active_ids.update(|active| {
                    active.insert(guild_id, list_id.to_string());
                });
            }
        }
        if update.ops.is_empty() {
            return;
        }

        self.by_guild.update(|all_members| {
            let members = all_members.entry(guild_id).or_default();

            for op in update.ops.iter().filter(|op| {
                matches!(
                    op.op,
                    ListOpKind::Sync | ListOpKind::Insert | ListOpKind::Update
                )
            }) {
                for member in op.members() {
                    members.insert(member.user.id, member.guild_member());
                }
            }
        });
    }
}

impl MemberStore {
    pub fn is_member(&self, guild: Id<GuildMarker>, user: Id<UserMarker>) -> bool {
        self.by_guild.with_untracked(|all| {
            all.get(&guild)
                .is_some_and(|members| members.contains_key(&user))
        })
    }

    fn apply_chunk(&self, chunk: &super::wire::MembersChunk) {
        self.by_guild.update(|all| {
            let members = all.entry(chunk.guild_id).or_default();
            for member in &chunk.members {
                members.insert(member.user.id, member.guild_member());
            }
        });
        member_requests::acknowledge(
            chunk.guild_id.get(),
            chunk
                .members
                .iter()
                .map(|m| m.user.id.get())
                .chain(chunk.not_found.iter().map(|u| u.get())),
        );
    }
}

impl Store for MemberStore {
    fn handle(&self, action: &Action) {
        match action {
            Action::MemberListUpdate(update) => self.apply_list_update(update),
            Action::GuildMembersChunk(chunk) => self.apply_chunk(chunk),

            Action::ConnectionOpen { .. } => member_requests::reset(),
            Action::Resumed => {
                member_requests::resend_unacknowledged(|guild, user| {
                    self.is_member(Id::new(guild), Id::new(user))
                })
            }
            _ => {}
        }
    }

    fn reset(&self) {
        self.by_guild.set(HashMap::new());
        self.lists.set(HashMap::new());
        self.active_ids.set(HashMap::new());
    }
}
