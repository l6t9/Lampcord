use std::collections::HashMap;
use std::mem;

use requiem_ui::signal::KeyedSignal;

use super::{Action, GuildData, Store, UserStore};

use crate::models::channel::ThreadMember;
use crate::models::id::marker::{ChannelMarker, GuildMarker};
use crate::models::{Channel, Id};

#[derive(Clone)]
pub struct ChannelStore {
    /// Follow one guild with `by_guild.watch(&guild_id)`; reading the whole map wakes on every guild.
    pub by_guild: KeyedSignal<Id<GuildMarker>, Vec<Channel>>,
    users: UserStore,
}

impl ChannelStore {
    pub fn new(users: UserStore) -> Self {
        Self {
            by_guild: KeyedSignal::new(),
            users,
        }
    }

    fn update_thread(
        &self,
        guild_id: Id<GuildMarker>,
        thread_id: Id<ChannelMarker>,
        apply: impl FnOnce(&mut Channel),
    ) {
        self.by_guild.update_key(&guild_id, |by_guild| {
            if let Some(thread) = by_guild
                .get_mut(&guild_id)
                .and_then(|channels| channels.iter_mut().find(|c| c.id == thread_id))
            {
                apply(thread);
            }
        });
    }

    fn set_guild_channels(&self, data: &GuildData) {
        self.by_guild.update_key(&data.guild.id, |by_guild| {
            by_guild.insert(data.guild.id, data.channels.clone());
        });
    }

    pub fn upsert(&self, mut channel: Channel, keep_overwrites: bool) {
        let Some(guild_id) = channel.guild_id else {
            return;
        };
        self.by_guild.update_key(&guild_id, |by_guild| {
            let channels = by_guild.entry(guild_id).or_default();
            match channels.iter_mut().find(|c| c.id == channel.id) {
                Some(existing) => {
                    if keep_overwrites {
                        channel.permission_overwrites =
                            mem::take(&mut existing.permission_overwrites);
                    }
                    if channel.member.is_none() {
                        channel.member = existing.member.clone();
                    }
                    *existing = channel;
                }
                None => channels.push(channel),
            }
        });
    }
}

impl Store for ChannelStore {
    fn handle(&self, action: &Action) {
        match action {
            Action::ConnectionOpen { guilds, .. } => {
                guilds.iter().for_each(|g| self.set_guild_channels(g))
            }
            Action::GuildCreate(data) => self.set_guild_channels(data),
            Action::GuildDelete(id) => self.by_guild.update_key(id, |by_guild| {
                by_guild.remove(id);
            }),

            Action::ChannelCreate(data) | Action::ChannelUpdate(data) => {
                let channel = data.channel();
                if channel.guild_id.is_some() {
                    self.upsert(channel, data.permission_overwrites.is_none());
                }
            }
            Action::ThreadCreate(channel) | Action::ThreadUpdate(channel) => {
                self.upsert((**channel).clone(), false)
            }
            Action::ThreadMemberUpdate(data) => {
                let member = data.member.clone();
                let Some(thread_id) = member.id else { return };
                self.update_thread(data.guild_id, thread_id, |thread| {
                    thread.member = Some(member);
                });
            }
            Action::ThreadMembersUpdate(data) => {
                let me = self
                    .users
                    .current
                    .with_untracked(|user| user.as_ref().map(|user| user.id));
                let added: Option<ThreadMember> = data
                    .added_members
                    .iter()
                    .find(|member| member.user_id.is_some() && member.user_id == me)
                    .cloned();
                let removed = me.is_some_and(|me| data.removed_member_ids.contains(&me));
                let count = data.member_count;
                self.update_thread(data.guild_id, data.id, |thread| {
                    if let Some(count) = count {
                        thread.member_count = count;
                    }
                    if removed {
                        thread.member = None;
                    } else if let Some(member) = added {
                        thread.member = Some(member);
                    }
                });
            }
            Action::ChannelDelete(data) | Action::ThreadDelete(data) => {
                if let Some(guild_id) = data.guild_id {
                    let id = data.id;
                    self.by_guild.update_key(&guild_id, |by_guild| {
                        if let Some(channels) = by_guild.get_mut(&guild_id) {
                            channels.retain(|c| c.id != id);
                        }
                    });
                }
            }
            _ => {}
        }
    }

    fn reset(&self) {
        self.by_guild.set(HashMap::new());
    }
}
