use std::cell::RefCell;
use std::cmp;
use std::collections::HashMap;
use std::env;
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::Duration;
use std::time::Instant;

use requiem_ui::signal::{KeyedSignal, Signal, signal};
use requiem_ui::timer::{TimeoutHandle, clear_timeout, set_deadline};

use super::{Action, GuildData, Store};
use crate::models::id::marker::{GuildMarker, RoleMarker, UserMarker};
use crate::models::member::is_timed_out;
use crate::models::permissions::{
    MemberContext, compute_base, compute_channel, compute_channel_of_kind, everyone_can,
    is_thread_kind,
};
use crate::models::{Channel, Guild, Id, Permissions, Role};

#[derive(Clone)]
pub struct PermissionStore {
    roles: KeyedSignal<Id<GuildMarker>, HashMap<Id<RoleMarker>, Role>>,
    my_roles: KeyedSignal<Id<GuildMarker>, Vec<Id<RoleMarker>>>,
    me: Signal<Option<Id<UserMarker>>>,

    timeouts: KeyedSignal<Id<GuildMarker>, i64>,

    timeout_tick: Signal<u64>,

    expiry_timers: Rc<RefCell<HashMap<Id<GuildMarker>, TimeoutHandle>>>,

    bits_cache: Rc<RefCell<HashMap<Id<GuildMarker>, Rc<HashMap<Id<RoleMarker>, Permissions>>>>>,
}

fn spoofed() -> bool {
    static SPOOF: OnceLock<bool> = OnceLock::new();
    *SPOOF.get_or_init(|| env::var_os("REQUIEM_SPOOF_PERMISSIONS").is_some())
}

impl Default for PermissionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionStore {
    pub fn new() -> Self {
        Self {
            roles: KeyedSignal::new(),
            my_roles: KeyedSignal::new(),
            me: signal(None),
            timeouts: KeyedSignal::new(),
            timeout_tick: signal(0),
            expiry_timers: Rc::default(),
            bits_cache: Rc::default(),
        }
    }

    pub fn my_role_ids(&self, guild_id: Id<GuildMarker>) -> Vec<Id<RoleMarker>> {
        self.my_roles.watch(&guild_id);
        self.my_roles
            .with_untracked(|all| all.get(&guild_id).cloned().unwrap_or_default())
    }

    pub fn guild_roles(&self, guild_id: Id<GuildMarker>) -> Vec<Role> {
        self.roles.watch(&guild_id);
        let mut roles: Vec<Role> = self.roles.with_untracked(|all| {
            all.get(&guild_id)
                .map(|roles| roles.values().cloned().collect())
                .unwrap_or_default()
        });
        roles.sort_by_key(|role| cmp::Reverse(role.position));
        roles
    }

    /// A role by id alone, for places that only carry the id (a mention in a
    /// message, a reply preview). Not tracked: roles change rarely.
    pub fn role(&self, role_id: Id<RoleMarker>) -> Option<Role> {
        self.roles
            .with_untracked(|all| all.values().find_map(|roles| roles.get(&role_id).cloned()))
    }

    pub fn role_in(&self, guild_id: Id<GuildMarker>, role_id: Id<RoleMarker>) -> Option<Role> {
        self.roles.watch(&guild_id);
        self.roles
            .with_untracked(|all| all.get(&guild_id)?.get(&role_id).cloned())
    }

    fn bits(&self, guild_id: Id<GuildMarker>) -> Rc<HashMap<Id<RoleMarker>, Permissions>> {
        self.roles.watch(&guild_id);
        if let Some(cached) = self.bits_cache.borrow().get(&guild_id) {
            return cached.clone();
        }
        let bits: Rc<HashMap<_, _>> = Rc::new(self.roles.with_untracked(|all| {
            all.get(&guild_id)
                .map(|roles| {
                    roles
                        .iter()
                        .map(|(id, role)| (*id, role.permissions))
                        .collect()
                })
                .unwrap_or_default()
        }));
        self.bits_cache.borrow_mut().insert(guild_id, bits.clone());
        bits
    }

    fn invalidate_bits(&self, guild_id: Id<GuildMarker>) {
        self.bits_cache.borrow_mut().remove(&guild_id);
    }

    fn with_context<R>(
        &self,
        guild_id: Id<GuildMarker>,
        owner_id: Option<Id<UserMarker>>,
        f: impl FnOnce(&MemberContext) -> R,
    ) -> Option<R> {
        let user_id = self.me.get()?;
        let roles = self.bits(guild_id);
        self.my_roles.watch(&guild_id);
        let member_roles = self
            .my_roles
            .with_untracked(|all| all.get(&guild_id).cloned().unwrap_or_default());
        let timed_out = self.is_timed_out(guild_id);
        Some(f(&MemberContext {
            guild_id,
            owner_id,
            user_id,
            roles: &roles,
            member_roles: &member_roles,
            timed_out,
        }))
    }

    pub fn is_timed_out(&self, guild_id: Id<GuildMarker>) -> bool {
        self.timeout_tick.with(|_| ());
        self.timeouts.watch(&guild_id);
        let until = self.timeouts.with_untracked(|all| all.get(&guild_id).copied());
        is_timed_out(until, chrono::Utc::now().timestamp_millis())
    }

    pub fn timeout_until(&self, guild_id: Id<GuildMarker>) -> Option<i64> {
        self.timeouts.watch(&guild_id);
        self.timeouts.with_untracked(|all| all.get(&guild_id).copied())
    }

    fn set_timeout_until(&self, guild_id: Id<GuildMarker>, until: Option<i64>) {
        if self
            .timeouts
            .with_untracked(|all| all.get(&guild_id).copied())
            != until
        {
            self.timeouts.update_key(&guild_id, |all| match until {
                Some(until) => {
                    all.insert(guild_id, until);
                }
                None => {
                    all.remove(&guild_id);
                }
            });
        }
        if let Some(handle) = self.expiry_timers.borrow_mut().remove(&guild_id) {
            clear_timeout(handle);
        }

        let now = chrono::Utc::now().timestamp_millis();
        if let Some(until) = until.filter(|until| *until > now) {
            let delay = Duration::from_millis((until - now) as u64 + 50);
            let (tick, timers) = (self.timeout_tick.clone(), self.expiry_timers.clone());
            let handle = set_deadline(Instant::now() + delay, move || {
                timers.borrow_mut().remove(&guild_id);
                tick.update(|n| *n += 1);
            });
            self.expiry_timers.borrow_mut().insert(guild_id, handle);
        }
    }

    pub fn guild_permissions(&self, guild: &Guild) -> Permissions {
        if spoofed() {
            return Permissions::ALL;
        }
        self.with_context(guild.id, guild.owner_id, compute_base)
            .unwrap_or(Permissions::ALL)
    }

    pub fn channel_permissions(&self, guild: Option<&Guild>, channel: &Channel) -> Permissions {
        let Some(guild) = guild.filter(|_| channel.guild_id.is_some()) else {
            return Permissions::ALL;
        };
        if spoofed() {
            return Permissions::ALL;
        }

        if channel.is_obfuscated() {
            return Permissions::NONE;
        }
        self.with_context(guild.id, guild.owner_id, |ctx| {
            compute_channel_of_kind(ctx, &channel.permission_overwrites, channel.kind)
        })
        .unwrap_or(Permissions::ALL)
    }

    pub fn channel_permissions_for(
        &self,
        guild_id: Id<GuildMarker>,
        owner_id: Option<Id<UserMarker>>,
        channel: &Channel,
    ) -> Permissions {
        self.channel_permissions_in(guild_id, owner_id, channel, None)
    }

    pub fn channel_permissions_in(
        &self,
        guild_id: Id<GuildMarker>,
        owner_id: Option<Id<UserMarker>>,
        channel: &Channel,
        parent: Option<&Channel>,
    ) -> Permissions {
        if spoofed() {
            return Permissions::ALL;
        }
        let source = parent
            .filter(|_| is_thread_kind(channel.kind))
            .unwrap_or(channel);
        if channel.is_obfuscated() || source.is_obfuscated() {
            return Permissions::NONE;
        }
        self.with_context(guild_id, owner_id, |ctx| {
            compute_channel_of_kind(ctx, &source.permission_overwrites, channel.kind)
        })
        .unwrap_or(Permissions::ALL)
    }

    pub fn role_required(&self, guild_id: Id<GuildMarker>, channel: &Channel) -> bool {
        if channel.guild_id.is_none() || channel.kind == 4 {
            return false;
        }
        let roles = self.bits(guild_id);
        let everyone = |permission| {
            everyone_can(
                &roles,
                guild_id,
                &channel.permission_overwrites,
                permission,
            )
        };
        (matches!(channel.kind, 2 | 13) && !everyone(Permissions::CONNECT))
            || !everyone(Permissions::VIEW_CHANNEL)
    }

    pub fn roles_that_can_view(&self, guild_id: Id<GuildMarker>, channel: &Channel) -> Vec<Role> {
        let bits = self.bits(guild_id);
        self.guild_roles(guild_id)
            .into_iter()
            .filter(|role| role.id.get() != guild_id.get())
            .filter(|role| {
                let ctx = MemberContext {
                    guild_id,
                    owner_id: None,
                    user_id: Id::new(0),
                    roles: &bits,
                    member_roles: &[role.id],
                    timed_out: false,
                };
                compute_channel(&ctx, &channel.permission_overwrites)
                    .contains(Permissions::VIEW_CHANNEL)
            })
            .collect()
    }

    pub fn highest_role_position(&self, guild: &Guild) -> i32 {
        if self.me.get() == guild.owner_id && guild.owner_id.is_some() {
            return i32::MAX;
        }
        self.my_roles.watch(&guild.id);
        self.roles.watch(&guild.id);
        let mine = self
            .my_roles
            .with_untracked(|all| all.get(&guild.id).cloned().unwrap_or_default());
        self.roles.with_untracked(|all| {
            all.get(&guild.id).map_or(0, |roles| {
                mine.iter()
                    .filter_map(|id| roles.get(id))
                    .map(|r| r.position)
                    .max()
                    .unwrap_or(0)
            })
        })
    }

    pub fn can_manage_role(&self, guild: &Guild, role: &Role) -> bool {
        if spoofed() {
            return true;
        }
        if !self
            .guild_permissions(guild)
            .contains(Permissions::MANAGE_ROLES)
        {
            return false;
        }
        role.id.get() != guild.id.get() && self.highest_role_position(guild) > role.position
            || self.me.get() == guild.owner_id && guild.owner_id.is_some()
    }

    pub fn can_assign_any_role(
        &self,
        guild: &Guild,
        roles: &[Role],
        assigned: &[Id<RoleMarker>],
    ) -> bool {
        roles.iter().any(|role| {
            role.id.get() != guild.id.get()
                && self.can_manage_role(guild, role)
                && (assigned.contains(&role.id) || !role.managed)
        })
    }

    fn set_roles(&self, guild_id: Id<GuildMarker>, roles: &[Role]) {
        self.roles.update_key(&guild_id, |all| {
            all.insert(
                guild_id,
                roles.iter().map(|role| (role.id, role.clone())).collect(),
            );
        });
        self.invalidate_bits(guild_id);
    }

    fn set_my_roles(&self, guild_id: Id<GuildMarker>, roles: Vec<Id<RoleMarker>>) {
        self.my_roles.update_key(&guild_id, |all| {
            all.insert(guild_id, roles);
        });
    }

    fn ingest_guild(&self, data: &GuildData) {
        self.set_roles(data.guild.id, &data.roles);
        if let Some(me) = self.me.peek() {
            if let Some(member) = data.members.iter().find(|m| m.user.id == me) {
                self.set_my_roles(data.guild.id, member.roles.clone());
                self.set_timeout_until(data.guild.id, member.communication_disabled_until);
            }
        }
    }
}

impl Store for PermissionStore {
    fn handle(&self, action: &Action) {
        match action {
            Action::ConnectionOpen {
                user,
                guilds,
                ready,
                ..
            } => {
                self.me.set(Some(user.id));
                guilds.iter().for_each(|g| self.ingest_guild(g));

                for entry in ready.merged_members.iter().flatten() {
                    if let (Some(guild_id), Some(roles)) = (entry.guild_id, &entry.roles) {
                        self.set_my_roles(guild_id, roles.clone());
                        self.set_timeout_until(guild_id, entry.communication_disabled_until);
                    }
                }
            }
            Action::GuildCreate(data) => self.ingest_guild(data),
            Action::GuildDelete(id) => {
                self.roles.update_key(id, |all| {
                    all.remove(id);
                });
                self.my_roles.update_key(id, |all| {
                    all.remove(id);
                });
                self.set_timeout_until(*id, None);
                self.invalidate_bits(*id);
            }
            Action::GuildRoleUpsert { guild_id, role } => {
                self.roles.update_key(guild_id, |all| {
                    all.entry(*guild_id)
                        .or_default()
                        .insert(role.id, role.clone());
                });
                self.invalidate_bits(*guild_id);
            }
            Action::GuildRoleDelete { guild_id, role_id } => {
                self.roles.update_key(guild_id, |all| {
                    if let Some(roles) = all.get_mut(guild_id) {
                        roles.remove(role_id);
                    }
                });
                self.invalidate_bits(*guild_id);
            }
            Action::GuildMemberUpdate { guild_id, member }
                if self.me.peek() == Some(member.user.id) => {
                    self.set_my_roles(*guild_id, member.roles.clone());
                    self.set_timeout_until(*guild_id, member.communication_disabled_until);
                }
            _ => {}
        }
    }

    fn reset(&self) {
        self.roles.set(HashMap::new());
        self.my_roles.set(HashMap::new());
        self.me.set(None);
        self.timeouts.set(HashMap::new());
        for (_, handle) in self.expiry_timers.borrow_mut().drain() {
            clear_timeout(handle);
        }
        self.bits_cache.borrow_mut().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::permissions::{OverwriteKind, PermissionOverwrite};

    fn role(id: u64, permissions: Permissions) -> Role {
        Role {
            id: Id::new(id),
            colors: Default::default(),
            name: format!("r{id}"),
            color: 0,
            position: 0,
            icon: None,
            unicode_emoji: None,
            managed: false,
            hoist: false,
            mentionable: false,
            permissions,
        }
    }

    fn guild() -> Guild {
        Guild::new(Id::new(100), "g", None, Some(Id::new(1)))
    }

    fn channel(overwrites: Vec<PermissionOverwrite>) -> Channel {
        Channel {
            id: Id::new(5),
            guild_id: Some(Id::new(100)),
            kind: 0,
            name: Some("c".into()),
            position: None,
            parent_id: None,
            permission_overwrites: overwrites,
            nsfw: false,
            flags: 0,
            last_message_id: None,
            owner_id: None,
            thread_metadata: None,
            message_count: 0,
            member_count: 0,
            total_message_sent: 0,
            member: None,
            newly_created: false,
        }
    }

    #[test]
    fn a_role_change_reaches_the_channel_decision() {
        let store = PermissionStore::new();
        store.me.set(Some(Id::new(7)));
        store.set_roles(
            Id::new(100),
            &[
                role(100, Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES),
                role(200, Permissions::NONE),
            ],
        );
        let locked = channel(vec![
            PermissionOverwrite {
                id: 100,
                kind: OverwriteKind::Role,
                allow: Permissions::NONE,
                deny: Permissions::VIEW_CHANNEL,
            },
            PermissionOverwrite {
                id: 200,
                kind: OverwriteKind::Role,
                allow: Permissions::VIEW_CHANNEL,
                deny: Permissions::NONE,
            },
        ]);
        let guild = guild();
        assert!(
            !store
                .channel_permissions(Some(&guild), &locked)
                .contains(Permissions::VIEW_CHANNEL)
        );
        store.handle(&Action::GuildMemberUpdate {
            guild_id: Id::new(100),
            member: super::super::action::MemberRoles {
                user: super::super::action::MemberUserId { id: Id::new(7) },
                roles: vec![Id::new(200)],
                communication_disabled_until: None,
            },
        });
        assert!(
            store
                .channel_permissions(Some(&guild), &locked)
                .contains(Permissions::VIEW_CHANNEL)
        );
        assert!(store.role_required(Id::new(100), &locked));
    }

    #[test]
    fn a_masked_channel_grants_nothing_and_dms_grant_everything() {
        let store = PermissionStore::new();
        store.me.set(Some(Id::new(7)));
        store.set_roles(Id::new(100), &[role(100, Permissions::ALL)]);
        let mut hidden = channel(Vec::new());
        hidden.flags = Channel::OBFUSCATED;
        assert_eq!(
            store.channel_permissions(Some(&guild()), &hidden),
            Permissions::NONE
        );
        let mut dm = channel(Vec::new());
        dm.guild_id = None;
        assert_eq!(store.channel_permissions(None, &dm), Permissions::ALL);
    }
}
