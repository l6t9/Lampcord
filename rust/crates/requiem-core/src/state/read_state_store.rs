use std::cell::{Cell, RefCell};
use std::cmp;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use requiem_ui::signal::{KeyedSignal, Signal, signal};
use requiem_ui::timer::{TimeoutHandle, clear_timeout, set_timeout};

use super::{
    Action, ChannelStore, DmStore, GuildSettingsStore, GuildStore, MessageStore, PermissionStore,
    RelationshipStore, Store, UserStore,
};
use crate::models::id::marker::{
    ChannelMarker, GuildMarker, MessageMarker, RoleMarker, UserMarker,
};
use crate::models::{Channel, Guild, Id, Message, Permissions};
use crate::rest::endpoints::relationships::BLOCKED;
use crate::utils::time::{now_ms, snowflake_at};

type ChannelId = Id<ChannelMarker>;
type GuildId = Id<GuildMarker>;

const ACK_DELAY: Duration = Duration::from_millis(3000);
const DISCORD_EPOCH_MS: i64 = 1_420_070_400_000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChannelBadge {
    pub unread: bool,

    pub important: bool,

    pub mentions: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GuildBadge {
    pub unread: bool,

    pub mentions: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AckRequest {
    Message {
        channel_id: ChannelId,
        message_id: MessageId,

        last_viewed: u32,

        flags: Option<u8>,
    },

    Bulk(Vec<(ChannelId, MessageId)>),
}

type MessageId = Id<MessageMarker>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NewMarker {
    pub first_unread: Option<u64>,

    pub count: u32,
    pub estimated: bool,

    pub acked_at_open: u64,
}

#[derive(Clone, Debug, Default)]
struct ReadState {
    last_message_id: Option<u64>,
    ack_message_id: Option<u64>,
    mention_count: u32,
    unread_count: u32,
    estimated: bool,
    oldest_unread: Option<u64>,

    ack_at_select: Option<u64>,

    is_manual_ack: bool,

    loaded: bool,

    server_flags: u8,

    outgoing_ack: Option<u64>,
}

impl ReadState {
    fn has_unread(&self, fallback_ack: u64) -> bool {
        self.last_message_id
            .is_some_and(|last| last > self.ack_message_id.unwrap_or(fallback_ack))
    }

    fn ack_local(&mut self, message_id: u64) {
        self.ack_message_id = Some(message_id);
        self.mention_count = 0;
        self.unread_count = 0;
        self.estimated = false;
        self.is_manual_ack = false;
    }
}

fn is_readable_kind(kind: u8) -> bool {
    matches!(kind, 0 | 1 | 2 | 3 | 5 | 10 | 11 | 12 | 13 | 15 | 16)
}

struct Inner {
    states: RefCell<HashMap<ChannelId, ReadState>>,
    guild_of: RefCell<HashMap<ChannelId, GuildId>>,
    joined_at: RefCell<HashMap<GuildId, i64>>,
    badges: RefCell<HashMap<ChannelId, Signal<ChannelBadge>>>,
    guild_badges: RefCell<HashMap<GuildId, Signal<GuildBadge>>>,

    guild_unread: RefCell<HashMap<GuildId, Signal<bool>>>,

    published_mentions: RefCell<HashMap<ChannelId, u32>>,
    dm_mentions: Signal<u32>,

    dm_unread: Signal<bool>,
    total_mentions: Signal<u32>,

    marker_version: Signal<u64>,

    /// Whether any channel is unread or pinged. Changes far less often than `marker_version`.
    any_unread: Signal<bool>,

    marker_revisions: RefCell<HashMap<ChannelId, Signal<u64>>>,
    last_seen: KeyedSignal<ChannelId, ()>,
    viewing: Cell<Option<ChannelId>>,
    at_bottom: Cell<bool>,
    focused: Cell<bool>,
    timers: RefCell<HashMap<ChannelId, (TimeoutHandle, bool)>>,
    sender: RefCell<Option<Rc<dyn Fn(AckRequest)>>>,

    live_mentions: RefCell<Vec<Message>>,
}

const LIVE_MENTIONS_KEPT: usize = 100;

#[derive(Clone)]
pub struct ReadStateStore {
    users: UserStore,
    permissions: PermissionStore,
    settings: GuildSettingsStore,
    channels: ChannelStore,
    dms: DmStore,
    guilds: GuildStore,
    messages: MessageStore,
    relationships: RelationshipStore,
    inner: Rc<Inner>,
}

#[allow(clippy::too_many_arguments)]
impl ReadStateStore {
    pub fn new(
        users: UserStore,
        permissions: PermissionStore,
        settings: GuildSettingsStore,
        channels: ChannelStore,
        dms: DmStore,
        guilds: GuildStore,
        messages: MessageStore,
        relationships: RelationshipStore,
    ) -> Self {
        Self {
            users,
            permissions,
            settings,
            channels,
            dms,
            guilds,
            messages,
            relationships,
            inner: Rc::new(Inner {
                states: RefCell::new(HashMap::new()),
                guild_of: RefCell::new(HashMap::new()),
                joined_at: RefCell::new(HashMap::new()),
                badges: RefCell::new(HashMap::new()),
                guild_badges: RefCell::new(HashMap::new()),
                guild_unread: RefCell::new(HashMap::new()),
                published_mentions: RefCell::new(HashMap::new()),
                dm_mentions: signal(0),
                dm_unread: signal(false),
                total_mentions: signal(0),
                marker_version: signal(0),
                any_unread: signal(false),
                marker_revisions: RefCell::new(HashMap::new()),
                last_seen: KeyedSignal::new(),
                viewing: Cell::new(None),
                at_bottom: Cell::new(true),
                focused: Cell::new(true),
                timers: RefCell::new(HashMap::new()),
                sender: RefCell::new(None),
                live_mentions: RefCell::new(Vec::new()),
            }),
        }
    }

    pub fn badge(&self, channel_id: ChannelId) -> Signal<ChannelBadge> {
        if let Some(existing) = self.inner.badges.borrow().get(&channel_id) {
            return existing.clone();
        }
        let signal = signal(self.compute_badge(channel_id));
        self.inner
            .badges
            .borrow_mut()
            .insert(channel_id, signal.clone());
        signal
    }

    pub fn guild_badge(&self, guild_id: GuildId) -> Signal<GuildBadge> {
        if let Some(existing) = self.inner.guild_badges.borrow().get(&guild_id) {
            return existing.clone();
        }
        let signal = signal(self.compute_guild_badge(guild_id));
        self.inner
            .guild_badges
            .borrow_mut()
            .insert(guild_id, signal.clone());
        signal
    }

    pub fn guild_unread(&self, guild_id: GuildId) -> Signal<bool> {
        if let Some(existing) = self.inner.guild_unread.borrow().get(&guild_id) {
            return existing.clone();
        }
        let signal = signal(self.compute_guild_badge(guild_id).unread);
        self.inner
            .guild_unread
            .borrow_mut()
            .insert(guild_id, signal.clone());
        signal
    }

    pub fn dm_unread(&self) -> Signal<bool> {
        self.inner.dm_unread.clone()
    }

    pub fn dm_mentions(&self) -> Signal<u32> {
        self.inner.dm_mentions.clone()
    }

    pub fn total_mentions(&self) -> Signal<u32> {
        self.inner.total_mentions.clone()
    }

    pub fn any_unread(&self) -> Signal<bool> {
        self.inner.any_unread.clone()
    }

    pub fn marker_version(&self) -> Signal<u64> {
        self.inner.marker_version.clone()
    }

    pub fn marker_revision(&self, channel_id: ChannelId) -> Signal<u64> {
        self.inner
            .marker_revisions
            .borrow_mut()
            .entry(channel_id)
            .or_insert_with(|| signal(0))
            .clone()
    }

    fn bump_channel_marker(&self, channel_id: ChannelId) {
        let revision = self
            .inner
            .marker_revisions
            .borrow()
            .get(&channel_id)
            .cloned();
        if let Some(revision) = revision {
            revision.update(|n| *n += 1);
        }
    }

    pub fn new_marker(&self, channel_id: ChannelId) -> Option<NewMarker> {
        let states = self.inner.states.borrow();
        let rs = states.get(&channel_id)?;
        let unread_at_open = rs
            .ack_at_select
            .is_some_and(|ack| rs.last_message_id.is_some_and(|last| last > ack));
        (rs.oldest_unread.is_some() || (rs.unread_count > 0) || unread_at_open).then_some(
            NewMarker {
                first_unread: rs.oldest_unread,
                count: rs.unread_count,
                estimated: rs.estimated,
                acked_at_open: rs
                    .ack_at_select
                    .or(rs.ack_message_id)
                    .unwrap_or_else(|| self.fallback_ack(channel_id)),
            },
        )
    }

    pub fn acked_through(&self, channel_id: ChannelId) -> u64 {
        self.inner
            .states
            .borrow()
            .get(&channel_id)
            .and_then(|rs| rs.ack_message_id)
            .unwrap_or_else(|| self.fallback_ack(channel_id))
    }

    pub fn last_message_id(&self, channel_id: ChannelId) -> Option<u64> {
        self.inner
            .states
            .borrow()
            .get(&channel_id)
            .and_then(|rs| rs.last_message_id)
    }

    pub fn viewing_guild(&self) -> Option<GuildId> {
        self.inner
            .viewing
            .get()
            .and_then(|channel| self.inner.guild_of.borrow().get(&channel).copied())
    }

    pub fn live_mentions(&self) -> Vec<Message> {
        self.inner
            .live_mentions
            .borrow()
            .iter()
            .rev()
            .cloned()
            .collect()
    }

    pub fn mention_count(&self, channel_id: ChannelId) -> u32 {
        self.inner
            .states
            .borrow()
            .get(&channel_id)
            .map_or(0, |rs| rs.mention_count)
    }

    fn candidate_unread_channels(&self) -> Vec<ChannelId> {
        let states = self.inner.states.borrow();
        states
            .iter()
            .filter(|(id, rs)| rs.mention_count > 0 || rs.has_unread(self.fallback_ack(**id)))
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn has_unread_channels(&self) -> bool {
        self.candidate_unread_channels().into_iter().any(|id| {
            let badge = self.compute_badge(id);
            badge.unread || badge.mentions > 0
        })
    }

    pub fn unread_channels(&self) -> Vec<ChannelId> {
        let ids = self.candidate_unread_channels();
        let mut unread: Vec<(u64, ChannelId)> = ids
            .into_iter()
            .filter_map(|id| {
                let badge = self.compute_badge(id);
                (badge.unread || badge.mentions > 0).then(|| {
                    (
                        self.inner
                            .states
                            .borrow()
                            .get(&id)
                            .and_then(|rs| rs.last_message_id)
                            .unwrap_or(0),
                        id,
                    )
                })
            })
            .collect();
        unread.sort_by_key(|(last, _)| cmp::Reverse(*last));
        unread.into_iter().map(|(_, id)| id).collect()
    }

    pub fn set_ack_sender(&self, sender: Rc<dyn Fn(AckRequest)>) {
        *self.inner.sender.borrow_mut() = Some(sender);
    }

    pub fn select_channel(&self, channel_id: Option<ChannelId>) {
        if self.inner.viewing.get() == channel_id {
            if let Some(channel_id) = channel_id {
                self.try_auto_ack(channel_id);
            }
            return;
        }
        let previous = self.inner.viewing.replace(channel_id);
        let mut touched = Vec::new();
        if let Some(previous) = previous {
            if let Some(rs) = self.inner.states.borrow_mut().get_mut(&previous) {
                rs.is_manual_ack = false;
                rs.oldest_unread = None;
                rs.ack_at_select = None;
                touched.push(previous);
            }
        }
        if let Some(channel_id) = channel_id {
            let fallback = self.fallback_ack(channel_id);
            let mut states = self.inner.states.borrow_mut();
            let rs = states.entry(channel_id).or_default();
            rs.ack_at_select = Some(rs.ack_message_id.unwrap_or(fallback));
            touched.push(channel_id);
        }
        touched.into_iter().for_each(|id| self.publish_channel(id));
        self.bump_markers();
        if let Some(channel_id) = channel_id {
            self.try_auto_ack(channel_id);
        }
    }

    pub fn set_at_bottom(&self, at_bottom: bool) {
        if self.inner.at_bottom.replace(at_bottom) != at_bottom && at_bottom {
            if let Some(channel_id) = self.inner.viewing.get() {
                self.try_auto_ack(channel_id);
            }
        }
    }

    pub fn set_window_focused(&self, focused: bool) {
        if self.inner.focused.replace(focused) != focused && focused {
            if let Some(channel_id) = self.inner.viewing.get() {
                self.try_auto_ack(channel_id);
            }
        }
    }

    pub fn on_history(&self, channel_id: ChannelId) {
        let fallback = self.fallback_ack(channel_id);
        let loaded = self.messages.by_channel.with_untracked(|all| {
            all.get(&channel_id)
                .map(|list| list.iter().map(|m| m.id.get()).collect::<Vec<u64>>())
        });
        let Some(ids) = loaded else { return };
        let mut raised = false;
        {
            let mut states = self.inner.states.borrow_mut();
            let rs = states.entry(channel_id).or_default();
            rs.loaded = true;

            if let Some(newest) = ids.iter().copied().max() {
                if rs.last_message_id.is_none_or(|last| newest > last) {
                    rs.last_message_id = Some(newest);
                    raised = true;
                }
            }
            let ack = rs.ack_message_id.unwrap_or(fallback);
            let unread: Vec<u64> = ids.iter().copied().filter(|id| *id > ack).collect();
            rs.oldest_unread = rs
                .oldest_unread
                .filter(|oldest| *oldest > ack)
                .or_else(|| unread.iter().copied().min());
            if rs.has_unread(fallback) {
                rs.unread_count = unread.len() as u32;

                rs.estimated = ids.iter().all(|id| *id > ack) && !ids.is_empty();
            } else {
                rs.unread_count = 0;
                rs.estimated = false;
            }
        }
        if raised {
            self.inner.last_seen.update_key(&channel_id, |_| {});
        }
        self.publish_channel(channel_id);
        self.bump_markers();
        self.try_auto_ack(channel_id);
    }

    pub fn try_auto_ack(&self, channel_id: ChannelId) -> bool {
        if self.inner.viewing.get() != Some(channel_id)
            || !self.inner.focused.get()
            || !self.inner.at_bottom.get()
        {
            return false;
        }
        let fallback = self.fallback_ack(channel_id);
        let can_ack = {
            let states = self.inner.states.borrow();
            states.get(&channel_id).is_some_and(|rs| {
                rs.loaded && !rs.is_manual_ack && (rs.has_unread(fallback) || rs.mention_count > 0)
            })
        };
        can_ack && self.can_track(channel_id) && self.ack(channel_id, None, false)
    }

    pub fn mark_channel_read(&self, channel_id: ChannelId) {
        self.ack(channel_id, None, true);
    }

    pub fn mark_guild_read(&self, guild_id: GuildId) {
        let ids: Vec<ChannelId> = self.channels.by_guild.with_untracked(|all| {
            all.get(&guild_id)
                .map(|list| {
                    list.iter()
                        .filter(|c| is_readable_kind(c.kind))
                        .map(|c| c.id)
                        .collect()
                })
                .unwrap_or_default()
        });
        self.ack_many(ids);
    }

    pub fn mark_all_read(&self) {
        let ids = self.unread_channels();
        self.ack_many(ids);
    }

    fn ack_many(&self, ids: Vec<ChannelId>) {
        let mut sent = Vec::new();
        for id in ids {
            let fallback = self.fallback_ack(id);
            let message = {
                let mut states = self.inner.states.borrow_mut();
                let Some(rs) = states.get_mut(&id) else {
                    continue;
                };
                if !(rs.has_unread(fallback) || rs.mention_count > 0) {
                    continue;
                }
                let Some(last) = rs.last_message_id else {
                    continue;
                };
                rs.ack_local(last);
                rs.oldest_unread = None;
                rs.outgoing_ack = None;
                last
            };
            if let Some((handle, _)) = self.inner.timers.borrow_mut().remove(&id) {
                clear_timeout(handle);
            }
            self.publish_channel(id);
            sent.push((id, Id::new(message)));
        }
        if !sent.is_empty() {
            self.bump_markers();
            self.send(AckRequest::Bulk(sent));
        }
    }

    fn ack(&self, channel_id: ChannelId, message_id: Option<u64>, explicit: bool) -> bool {
        let had_mentions = {
            let mut states = self.inner.states.borrow_mut();
            let Some(rs) = states.get_mut(&channel_id) else {
                return false;
            };
            if rs.is_manual_ack && !explicit {
                return false;
            }
            let Some(target) = message_id.or(rs.last_message_id) else {
                return false;
            };
            let had_mentions = rs.mention_count > 0;
            rs.ack_local(target);

            if explicit {
                rs.oldest_unread = None;
            }
            rs.outgoing_ack = Some(target);
            had_mentions
        };
        self.publish_channel(channel_id);
        self.bump_markers();
        self.schedule_ack(
            channel_id,
            if had_mentions || explicit {
                Duration::ZERO
            } else {
                ACK_DELAY
            },
        );
        true
    }

    fn schedule_ack(&self, channel_id: ChannelId, delay: Duration) {
        let immediate = delay.is_zero();
        {
            let timers = self.inner.timers.borrow();
            if let Some((handle, pending_immediate)) = timers.get(&channel_id).copied() {
                if pending_immediate || !immediate {
                    return;
                }
                clear_timeout(handle);
            }
        }
        let store = self.clone();
        let handle = set_timeout(delay, move || store.flush_ack(channel_id));
        self.inner
            .timers
            .borrow_mut()
            .insert(channel_id, (handle, immediate));
    }

    fn flush_ack(&self, channel_id: ChannelId) {
        self.inner.timers.borrow_mut().remove(&channel_id);
        let message = self
            .inner
            .states
            .borrow_mut()
            .get_mut(&channel_id)
            .and_then(|rs| rs.outgoing_ack.take());
        let Some(message) = message else { return };
        let wanted = u8::from(self.inner.guild_of.borrow().contains_key(&channel_id));

        let flags = self
            .inner
            .states
            .borrow_mut()
            .get_mut(&channel_id)
            .and_then(|rs| {
                let changed = rs.server_flags != wanted;
                rs.server_flags = wanted;
                changed.then_some(wanted)
            });
        self.send(AckRequest::Message {
            channel_id,
            message_id: Id::new(message),
            last_viewed: ((now_ms() as i64 - DISCORD_EPOCH_MS) / 86_400_000).max(0) as u32,
            flags,
        });
    }

    fn send(&self, request: AckRequest) {
        let sender = self.inner.sender.borrow().clone();
        match sender {
            Some(sender) => sender(request),
            None => log::debug!("read state: no ack sender installed, dropping {request:?}"),
        }
    }

    fn fallback_ack(&self, channel_id: ChannelId) -> u64 {
        let joined = self
            .inner
            .guild_of
            .borrow()
            .get(&channel_id)
            .and_then(|guild| self.inner.joined_at.borrow().get(guild).copied());
        joined.map_or(channel_id.get(), snowflake_at)
    }

    fn can_track(&self, channel_id: ChannelId) -> bool {
        let Some(guild_id) = self.inner.guild_of.borrow().get(&channel_id).copied() else {
            return true;
        };
        let channel = self.channels.by_guild.with_untracked(|all| {
            all.get(&guild_id)
                .and_then(|list| list.iter().find(|c| c.id == channel_id).cloned())
        });
        match channel {
            Some(channel) => self.can_view(&channel, self.guilds.find(guild_id).as_ref()),
            None => true,
        }
    }

    fn can_view(&self, channel: &Channel, guild: Option<&Guild>) -> bool {
        self.permissions
            .channel_permissions(guild, channel)
            .contains(Permissions::VIEW_CHANNEL)
    }

    fn compute_badge(&self, channel_id: ChannelId) -> ChannelBadge {
        let guild_id = self.inner.guild_of.borrow().get(&channel_id).copied();
        let channel = guild_id.and_then(|g| {
            self.channels.by_guild.with_untracked(|all| {
                all.get(&g)
                    .and_then(|list| list.iter().find(|c| c.id == channel_id).cloned())
            })
        });
        self.badge_of(
            channel_id,
            channel.as_ref(),
            guild_id.and_then(|g| self.guilds.find(g)).as_ref(),
        )
    }

    fn badge_of(
        &self,
        channel_id: ChannelId,
        channel: Option<&Channel>,
        guild: Option<&Guild>,
    ) -> ChannelBadge {
        let states = self.inner.states.borrow();
        let Some(rs) = states.get(&channel_id) else {
            return ChannelBadge::default();
        };
        if let Some(channel) = channel {
            if !is_readable_kind(channel.kind) || !self.can_view(channel, guild) {
                return ChannelBadge::default();
            }
        }
        let parent = channel.and_then(|c| c.parent_id);
        // Muting a server suppresses notifications, but Discord still shows unread
        // indicators there. Only a channel or category override hides the dot and
        // bright name.
        let muted = self.settings.is_channel_individually_muted(
            guild.map(|g| g.id),
            channel_id,
            parent,
        );
        let important_unreads = self.settings.unreads_are_important(
            guild.map(|g| g.id),
            channel_id,
            channel.map(|c| c.kind),
            parent,
            guild.map_or(0, |g| g.default_message_notifications),
        );
        let unread = !muted && rs.has_unread(self.fallback_ack(channel_id));
        ChannelBadge {
            unread,
            important: unread && (important_unreads || rs.mention_count > 0),
            mentions: rs.mention_count,
        }
    }

    fn compute_guild_badge(&self, guild_id: GuildId) -> GuildBadge {
        let guild = self.guilds.find(guild_id);
        let channels = self
            .channels
            .by_guild
            .with_untracked(|all| all.get(&guild_id).cloned().unwrap_or_default());
        // A muted guild hides the rail pill, but mentions still break through.
        // Channel indicators are deliberately unaffected: those are governed by
        // channel/category mutes only.
        let guild_muted = self.settings.is_guild_muted(guild_id);
        let mut badge = GuildBadge::default();
        for channel in &channels {
            let channel_badge = self.badge_of(channel.id, Some(channel), guild.as_ref());
            if !guild_muted {
                badge.unread |= channel_badge.unread;
            }
            badge.mentions += channel_badge.mentions;
        }
        badge
    }

    fn compute_dms(&self) -> (u32, bool) {
        let ids: Vec<ChannelId> = self
            .dms
            .list
            .with_untracked(|list| list.iter().map(|dm| dm.id).collect());
        let badges: Vec<ChannelBadge> = ids
            .into_iter()
            .map(|id| self.badge_of(id, None, None))
            .collect();
        (
            badges.iter().map(|badge| badge.mentions).sum(),
            badges.iter().any(|badge| badge.unread),
        )
    }

    fn compute_dm_mentions(&self) -> u32 {
        self.compute_dms().0
    }

    fn publish_dms(&self) {
        let (mentions, unread) = self.compute_dms();
        Self::set_if_changed(&self.inner.dm_mentions, mentions);
        Self::set_if_changed(&self.inner.dm_unread, unread);
    }

    fn publish_guild(&self, guild_id: GuildId) {
        let badge = self.compute_guild_badge(guild_id);
        let signal = self.inner.guild_badges.borrow().get(&guild_id).cloned();
        if let Some(signal) = signal {
            Self::set_if_changed(&signal, badge);
        }
        let flag = self.inner.guild_unread.borrow().get(&guild_id).cloned();
        if let Some(flag) = flag {
            Self::set_if_changed(&flag, badge.unread);
        }
    }

    fn set_if_changed<T: Clone + PartialEq + 'static>(signal: &Signal<T>, value: T) -> bool {
        if signal.with_untracked(|current| *current == value) {
            return false;
        }
        signal.set(value);
        true
    }

    fn bump_markers(&self) {
        self.inner.marker_version.update(|version| *version += 1);
        Self::set_if_changed(&self.inner.any_unread, self.has_unread_channels());
    }

    fn publish_channel(&self, channel_id: ChannelId) {
        self.bump_channel_marker(channel_id);
        let badge = self.compute_badge(channel_id);
        let existing = self.inner.badges.borrow().get(&channel_id).cloned();
        if let Some(signal) = existing {
            if Self::set_if_changed(&signal, badge) && badge.unread {
                log::debug!(
                    "unread badge: channel={channel_id} important={} mentions={} guild={:?}",
                    badge.important,
                    badge.mentions,
                    self.inner.guild_of.borrow().get(&channel_id)
                );
            }
        }

        let previous = self
            .inner
            .published_mentions
            .borrow_mut()
            .insert(channel_id, badge.mentions)
            .unwrap_or(0);
        let mentions_changed = previous != badge.mentions;
        match self.inner.guild_of.borrow().get(&channel_id).copied() {
            Some(guild_id) => self.publish_guild(guild_id),
            None => self.publish_dms(),
        }
        if mentions_changed {
            self.publish_totals();
        }
    }

    fn publish_all(&self) {
        let badges: Vec<(ChannelId, Signal<ChannelBadge>)> = self
            .inner
            .badges
            .borrow()
            .iter()
            .map(|(id, signal)| (*id, signal.clone()))
            .collect();
        for (id, signal) in badges {
            Self::set_if_changed(&signal, self.compute_badge(id));
        }
        let guilds: Vec<GuildId> = self
            .inner
            .guild_badges
            .borrow()
            .keys()
            .chain(self.inner.guild_unread.borrow().keys())
            .copied()
            .collect();
        for id in guilds {
            self.publish_guild(id);
        }
        self.publish_dms();
        self.publish_totals();
        self.bump_markers();
        let watched: Vec<ChannelId> = self
            .inner
            .marker_revisions
            .borrow()
            .keys()
            .copied()
            .collect();
        watched
            .into_iter()
            .for_each(|id| self.bump_channel_marker(id));
    }

    fn publish_totals(&self) {
        let guild_ids: Vec<GuildId> = self
            .guilds
            .list
            .with_untracked(|list| list.iter().map(|g| g.id).collect());
        let guilds: u32 = guild_ids
            .into_iter()
            .map(|id| self.compute_guild_badge(id).mentions)
            .sum();
        Self::set_if_changed(
            &self.inner.total_mentions,
            guilds + self.compute_dm_mentions(),
        );
    }

    fn remember_channel(&self, channel: &Channel, guild_id: Option<GuildId>) {
        if let Some(guild_id) = guild_id.or(channel.guild_id) {
            self.inner
                .guild_of
                .borrow_mut()
                .insert(channel.id, guild_id);
        }
        if let Some(last) = channel.last_message_id.map(|id| id.get()) {
            self.raise_last_message(channel.id, last);
        }
    }

    fn raise_last_message(&self, channel_id: ChannelId, last: u64) {
        let raised = {
            let mut states = self.inner.states.borrow_mut();
            let rs = states.entry(channel_id).or_default();
            let raised = rs.last_message_id.is_none_or(|known| last > known);
            if raised {
                rs.last_message_id = Some(last);
            }
            raised
        };
        if raised {
            self.inner.last_seen.update_key(&channel_id, |_| {});
        }
    }

    pub fn last_message_id_watched(&self, channel_id: ChannelId) -> Option<u64> {
        self.inner.last_seen.watch(&channel_id);
        self.last_message_id(channel_id)
    }

    fn on_connection_open(&self, guilds: &[super::GuildData], ready: &super::wire::ReadyExtras) {
        let partial = ready.read_state.as_ref().is_some_and(|state| state.partial);
        if !partial {
            self.inner.states.borrow_mut().clear();
            self.inner.guild_of.borrow_mut().clear();
            self.inner.joined_at.borrow_mut().clear();
        }
        for data in guilds {
            if let Some(joined) = data.joined_at_ms {
                self.inner
                    .joined_at
                    .borrow_mut()
                    .insert(data.guild.id, joined);
            }
            data.channels
                .iter()
                .for_each(|channel| self.remember_channel(channel, Some(data.guild.id)));
        }
        for dm in &ready.private_channels {
            if let Some(last) = dm.last_message_id {
                self.raise_last_message(dm.id, last.get());
            }
        }

        for entry in ready
            .read_state
            .iter()
            .flat_map(|state| state.entries.iter())
        {
            if entry.read_state_type != 0 {
                continue;
            }
            let Some(id) = entry.id else { continue };
            let ack = entry
                .last_message_id
                .map(|id| id.get())
                .filter(|id| *id != 0);
            let mut states = self.inner.states.borrow_mut();
            let rs = states.entry(id).or_default();
            rs.ack_message_id = ack;
            rs.mention_count = entry.mention_count as u32;
            rs.server_flags = entry.flags as u8;
        }
        self.inner.last_seen.update(|_| {});
        self.publish_all();
    }

    fn me(&self) -> Option<Id<UserMarker>> {
        self.users
            .current
            .with_untracked(|user| user.as_ref().map(|user| user.id))
    }

    pub fn pings_me(&self, message: &Message, guild_id: Option<GuildId>) -> bool {
        let Some(me) = self.me() else { return false };
        if message.mentions.iter().any(|user| user.id == me) {
            return true;
        }
        if message.mention_everyone && !self.settings.suppresses_everyone(guild_id) {
            return true;
        }
        if !message.mention_roles.is_empty() && !self.settings.suppresses_roles(guild_id) {
            let mine: Vec<Id<RoleMarker>> = guild_id
                .map(|guild| self.permissions.my_role_ids(guild))
                .unwrap_or_default();
            return message.mention_roles.iter().any(|role| mine.contains(role));
        }
        false
    }

    fn on_message_create(&self, message: &Message) {
        let channel_id = message.channel_id;
        let message_id = message.id.get();
        let guild_id = self.inner.guild_of.borrow().get(&channel_id).copied();
        let is_dm = guild_id.is_none();

        let previous_last = self
            .inner
            .states
            .borrow()
            .get(&channel_id)
            .and_then(|rs| rs.last_message_id);
        self.raise_last_message(channel_id, message_id);

        if previous_last.is_some_and(|last| last >= message_id) {
            self.publish_channel(channel_id);
            return;
        }

        if self.me() == Some(message.author.id) {
            if let Some(rs) = self.inner.states.borrow_mut().get_mut(&channel_id) {
                rs.outgoing_ack = None;
                rs.ack_local(message_id);
            }
            self.publish_channel(channel_id);
            self.bump_markers();
            return;
        }

        if self.relationships.kind_of(message.author.id) != Some(BLOCKED)
            && self.pings_me(message, guild_id)
        {
            let mut kept = self.inner.live_mentions.borrow_mut();
            if !kept.iter().any(|m| m.id == message.id) {
                kept.push(message.clone());
                let extra = kept.len().saturating_sub(LIVE_MENTIONS_KEPT);
                kept.drain(..extra);
            }
        }

        let reading = self.inner.viewing.get() == Some(channel_id)
            && self.inner.focused.get()
            && self.inner.at_bottom.get();
        let loaded = self
            .inner
            .states
            .borrow()
            .get(&channel_id)
            .is_some_and(|rs| rs.loaded);
        if reading && loaded && self.can_track(channel_id) {
            let manual = self
                .inner
                .states
                .borrow()
                .get(&channel_id)
                .is_some_and(|rs| rs.is_manual_ack);
            if !manual && self.ack(channel_id, Some(message_id), false) {
                return;
            }
        }

        let silent = self.relationships.kind_of(message.author.id) == Some(BLOCKED);
        let pings = !silent
            && (self.pings_me(message, guild_id)
                || (is_dm && !self.settings.is_channel_muted(None, channel_id, None)));
        {
            let mut states = self.inner.states.borrow_mut();
            let rs = states.entry(channel_id).or_default();
            if rs.oldest_unread.is_none() {
                rs.oldest_unread = Some(message_id);
            }
            rs.unread_count += 1;
            if pings {
                rs.mention_count += 1;
            }
        }
        self.publish_channel(channel_id);
        self.bump_markers();
    }

    fn on_message_delete(&self, channel_id: ChannelId, message_id: u64) {
        let newest_remaining = self.messages.by_channel.with_untracked(|all| {
            all.get(&channel_id).and_then(|list| {
                list.iter()
                    .map(|m| m.id.get())
                    .filter(|id| *id != message_id)
                    .max()
            })
        });
        {
            let mut states = self.inner.states.borrow_mut();
            let Some(rs) = states.get_mut(&channel_id) else {
                return;
            };

            if rs.last_message_id == Some(message_id) {
                rs.last_message_id = newest_remaining.or(rs.ack_message_id);
            }
            if rs.oldest_unread == Some(message_id) {
                rs.oldest_unread = None;
            }
        }
        self.on_history(channel_id);
    }

    fn on_message_ack(
        &self,
        channel_id: ChannelId,
        message_id: u64,
        mention_count: Option<u32>,
        manual: bool,
    ) {
        let loaded: Vec<u64> = self.messages.by_channel.with_untracked(|all| {
            all.get(&channel_id)
                .map(|list| list.iter().map(|m| m.id.get()).collect())
                .unwrap_or_default()
        });
        {
            let mut states = self.inner.states.borrow_mut();
            let rs = states.entry(channel_id).or_default();
            if manual {
                rs.ack_message_id = Some(message_id);
                rs.is_manual_ack = true;
                rs.oldest_unread = loaded.iter().copied().filter(|id| *id > message_id).min();
                rs.unread_count = loaded.iter().filter(|id| **id > message_id).count() as u32;
                rs.estimated = false;
                rs.mention_count = mention_count.unwrap_or(0);
            } else if rs.ack_message_id != Some(message_id) {
                rs.ack_local(message_id);
                rs.oldest_unread = None;
            }

            rs.outgoing_ack = None;
        }
        if let Some((handle, _)) = self.inner.timers.borrow_mut().remove(&channel_id) {
            clear_timeout(handle);
        }
        self.publish_channel(channel_id);
        self.bump_markers();
    }
}

impl Store for ReadStateStore {
    fn handle(&self, action: &Action) {
        match action {
            Action::ConnectionOpen { guilds, ready, .. } => self.on_connection_open(guilds, ready),
            Action::GuildCreate(data) => {
                if let Some(joined) = data.joined_at_ms {
                    self.inner
                        .joined_at
                        .borrow_mut()
                        .insert(data.guild.id, joined);
                }
                data.channels
                    .iter()
                    .for_each(|channel| self.remember_channel(channel, Some(data.guild.id)));
                self.publish_all();
            }
            Action::GuildDelete(guild_id) => {
                let gone: Vec<ChannelId> = self
                    .inner
                    .guild_of
                    .borrow()
                    .iter()
                    .filter(|(_, guild)| *guild == guild_id)
                    .map(|(channel, _)| *channel)
                    .collect();
                for channel in gone {
                    self.inner.states.borrow_mut().remove(&channel);
                    self.inner.guild_of.borrow_mut().remove(&channel);
                    self.inner
                        .badges
                        .borrow()
                        .get(&channel)
                        .map(|signal| Self::set_if_changed(signal, ChannelBadge::default()));
                }
                self.inner
                    .guild_badges
                    .borrow()
                    .get(guild_id)
                    .map(|signal| Self::set_if_changed(signal, GuildBadge::default()));
                self.inner
                    .guild_unread
                    .borrow()
                    .get(guild_id)
                    .map(|signal| Self::set_if_changed(signal, false));
                self.publish_totals();
            }
            Action::ChannelCreate(data) | Action::ChannelUpdate(data) => {
                let channel = data.channel();
                let new = !self.inner.states.borrow().contains_key(&channel.id);
                self.remember_channel(&channel, None);

                if new && matches!(action, Action::ChannelCreate(_)) {
                    if let Some(rs) = self.inner.states.borrow_mut().get_mut(&channel.id) {
                        rs.ack_message_id = rs.last_message_id;
                    }
                }
                self.publish_channel(channel.id);
                self.publish_all();
            }
            // A thread belongs to its guild; without that its messages read as DMs and every
            // one of them would count as a mention.
            Action::ThreadCreate(thread) | Action::ThreadUpdate(thread) => {
                self.remember_channel(thread, None);
                self.publish_channel(thread.id);
            }
            Action::ChannelDelete(data) => {
                let id = data.id;
                self.inner.states.borrow_mut().remove(&id);
                let guild = self.inner.guild_of.borrow_mut().remove(&id);
                if let Some(signal) = self.inner.badges.borrow().get(&id) {
                    Self::set_if_changed(signal, ChannelBadge::default());
                }
                if let Some(guild) = guild {
                    self.publish_guild(guild);
                }
                self.publish_totals();
            }
            Action::ChannelUnreadUpdate(data) => {
                for update in &data.channel_unread_updates {
                    self.raise_last_message(update.id, update.last_message_id.get());
                    self.publish_channel(update.id);
                }
            }
            Action::MessageCreate(message) => self.on_message_create(message),
            Action::MessageDelete { id, channel_id } => {
                self.on_message_delete(*channel_id, id.get())
            }
            Action::MessageAck {
                channel_id,
                message_id,
                mention_count,
                manual,
            } => self.on_message_ack(*channel_id, message_id.get(), *mention_count, *manual),

            Action::UserGuildSettingsUpdate(_)
            | Action::NotificationSettingsUpdate(_)
            | Action::GuildRoleUpsert { .. }
            | Action::GuildRoleDelete { .. }
            | Action::GuildMemberUpdate { .. }
            | Action::GuildUpdate(_)
            | Action::UserUpdate(_) => self.publish_all(),

            Action::RelationshipAdd(_)
            | Action::RelationshipUpdate(_)
            | Action::RelationshipRemove(_) => {}
            _ => {}
        }
    }

    fn reset(&self) {
        self.inner.states.borrow_mut().clear();
        self.inner.guild_of.borrow_mut().clear();
        self.inner.joined_at.borrow_mut().clear();
        for (_, (handle, _)) in self.inner.timers.borrow_mut().drain() {
            clear_timeout(handle);
        }
        self.inner.viewing.set(None);
        self.inner.at_bottom.set(true);
        self.inner.published_mentions.borrow_mut().clear();
        self.inner
            .badges
            .borrow()
            .values()
            .for_each(|signal| signal.set(ChannelBadge::default()));
        self.inner
            .guild_badges
            .borrow()
            .values()
            .for_each(|signal| signal.set(GuildBadge::default()));
        self.inner
            .guild_unread
            .borrow()
            .values()
            .for_each(|signal| signal.set(false));
        self.inner.dm_mentions.set(0);
        self.inner.dm_unread.set(false);
        self.inner.total_mentions.set(0);
        self.bump_markers();
        let watched: Vec<ChannelId> = self
            .inner
            .marker_revisions
            .borrow()
            .keys()
            .copied()
            .collect();
        watched
            .into_iter()
            .for_each(|id| self.bump_channel_marker(id));
    }
}

#[cfg(test)]
mod tests {
    use requiem_ui::timer;
    use serde_json::{Value, json};

    use super::*;
    use crate::gateway::events::GatewayEvent;
    use crate::state::AppState;

    const JOINED: &str = "2026-01-01T00:00:00+00:00";
    fn joined_ms() -> i64 {
        chrono::DateTime::parse_from_rfc3339(JOINED)
            .unwrap()
            .timestamp_millis()
    }

    fn before_join() -> u64 {
        snowflake_at(joined_ms() - 60_000)
    }
    fn after_join(offset: u64) -> u64 {
        snowflake_at(joined_ms() + 60_000) + offset
    }

    fn event(name: &str, data: Value) -> GatewayEvent {
        GatewayEvent::new(name, data)
    }

    fn user(id: u64) -> Value {
        json!({ "id": id.to_string(), "username": format!("u{id}"), "discriminator": "0", "global_name": null, "avatar": null })
    }

    fn message(id: u64, channel: u64, author: u64) -> Value {
        json!({ "id": id.to_string(), "channel_id": channel.to_string(), "author": user(author), "content": "hi", "timestamp": "2026-02-01T00:00:00.000Z" })
    }

    fn parsed(id: u64, channel: u64, author: u64) -> Message {
        serde_json::from_value(message(id, channel, author)).unwrap()
    }

    struct Setup {
        state: AppState,
        sent: Rc<RefCell<Vec<AckRequest>>>,
    }

    fn setup(last: u64, ack: Option<u64>, mentions: u32) -> Setup {
        let state = AppState::new();
        let sent = Rc::new(RefCell::new(Vec::new()));
        let sink = sent.clone();
        state
            .read_states
            .set_ack_sender(Rc::new(move |request| sink.borrow_mut().push(request)));
        let entries = ack.map(|ack| vec![json!({ "id": "11", "last_message_id": ack.to_string(), "mention_count": mentions, "last_viewed": 1 })]).unwrap_or_default();
        state.apply_event(&event(
            "READY",
            json!({
                "user": user(1),
                "guilds": [{
                    "id": "10", "joined_at": JOINED,
                    "properties": { "name": "G", "icon": null, "owner_id": "1" },
                    "channels": [
                        { "id": "11", "type": 0, "name": "general", "position": 0, "last_message_id": last.to_string() },
                        { "id": "12", "type": 0, "name": "other", "position": 1 },
                    ],
                }],
                "private_channels": [{ "id": "50", "type": 1, "recipient_ids": ["2"], "last_message_id": null }],
                "read_state": { "version": 1, "partial": false, "entries": entries },
                "user_guild_settings": { "version": 1, "partial": false, "entries": [] },
            }),
        ));
        Setup { state, sent }
    }

    fn badge(s: &Setup, channel: u64) -> ChannelBadge {
        s.state.read_states.badge(Id::new(channel)).peek()
    }
    fn guild(s: &Setup) -> GuildBadge {
        s.state.read_states.guild_badge(Id::new(10)).peek()
    }

    #[test]
    fn newer_than_what_was_acknowledged_is_unread_and_the_servers_ping_count_shows() {
        let s = setup(after_join(10), Some(after_join(0)), 2);
        assert_eq!(
            badge(&s, 11),
            ChannelBadge {
                unread: true,
                important: true,
                mentions: 2
            }
        );
        assert_eq!(
            guild(&s),
            GuildBadge {
                unread: true,
                mentions: 2
            }
        );
        assert_eq!(s.state.read_states.total_mentions().peek(), 2);
        assert_eq!(badge(&s, 12), ChannelBadge::default(), "no messages at all");
    }

    #[test]
    fn acknowledged_up_to_the_newest_is_read() {
        let s = setup(after_join(10), Some(after_join(10)), 0);
        assert_eq!(badge(&s, 11), ChannelBadge::default());
        assert_eq!(guild(&s), GuildBadge::default());
    }

    #[test]
    fn a_channel_never_opened_is_unread_only_for_messages_after_you_joined() {
        let old = setup(before_join(), None, 0);
        assert!(
            !badge(&old, 11).unread,
            "history from before you joined isn't yours to catch up on"
        );
        let new = setup(after_join(0), None, 0);
        assert!(badge(&new, 11).unread);
    }

    #[test]
    fn a_message_from_someone_else_is_unread_and_pings_when_it_mentions_you() {
        let s = setup(after_join(0), Some(after_join(0)), 0);
        s.state
            .apply_event(&event("MESSAGE_CREATE", message(after_join(5), 11, 2)));
        assert_eq!(
            badge(&s, 11),
            ChannelBadge {
                unread: true,
                important: true,
                mentions: 0
            }
        );

        let mut ping = message(after_join(6), 11, 2);
        ping["mentions"] = json!([user(1)]);
        s.state.apply_event(&event("MESSAGE_CREATE", ping));
        assert_eq!(
            badge(&s, 11),
            ChannelBadge {
                unread: true,
                important: true,
                mentions: 1
            }
        );
        assert_eq!(guild(&s).mentions, 1);
        assert_eq!(s.state.read_states.total_mentions().peek(), 1);
    }

    #[test]
    fn messages_that_ping_you_are_kept_newest_first_for_the_mentions_tab_even_while_you_read() {
        let s = setup(after_join(0), Some(after_join(0)), 0);
        s.state
            .apply_event(&event("MESSAGE_CREATE", message(after_join(4), 11, 2)));
        let ids = |s: &Setup| {
            s.state
                .read_states
                .live_mentions()
                .iter()
                .map(|m| m.id.get())
                .collect::<Vec<_>>()
        };
        assert!(ids(&s).is_empty(), "no ping, nothing kept");

        for at in [5, 6] {
            let mut ping = message(after_join(at), 11, 2);
            ping["mentions"] = json!([user(1)]);
            s.state.apply_event(&event("MESSAGE_CREATE", ping));
        }
        assert_eq!(ids(&s), vec![after_join(6), after_join(5)]);

        let mut again = message(after_join(6), 11, 2);
        again["mentions"] = json!([user(1)]);
        s.state.apply_event(&event("MESSAGE_CREATE", again));
        assert_eq!(ids(&s).len(), 2);
    }

    #[test]
    fn everyone_and_your_roles_ping_unless_suppressed() {
        let s = setup(after_join(0), Some(after_join(0)), 0);
        s.state.apply_event(&event("GUILD_ROLE_CREATE", json!({ "guild_id": "10", "role": { "id": "500", "name": "mods", "permissions": "0" } })));
        s.state.apply_event(&event(
            "GUILD_MEMBER_UPDATE",
            json!({ "guild_id": "10", "user": { "id": "1" }, "roles": ["500"] }),
        ));

        let mut everyone = message(after_join(1), 11, 2);
        everyone["mention_everyone"] = json!(true);
        s.state.apply_event(&event("MESSAGE_CREATE", everyone));
        let mut role = message(after_join(2), 11, 2);
        role["mention_roles"] = json!(["500"]);
        s.state.apply_event(&event("MESSAGE_CREATE", role));
        let mut other_role = message(after_join(3), 11, 2);
        other_role["mention_roles"] = json!(["501"]);
        s.state.apply_event(&event("MESSAGE_CREATE", other_role));
        assert_eq!(
            badge(&s, 11).mentions,
            2,
            "@everyone and your role, not someone else's"
        );

        s.state.apply_event(&event(
            "USER_GUILD_SETTINGS_UPDATE",
            json!({ "guild_id": "10", "suppress_everyone": true, "suppress_roles": true }),
        ));
        let mut again = message(after_join(4), 11, 2);
        again["mention_everyone"] = json!(true);
        again["mention_roles"] = json!(["500"]);
        s.state.apply_event(&event("MESSAGE_CREATE", again));
        assert_eq!(badge(&s, 11).mentions, 2, "suppressed");
    }

    #[test]
    fn every_dm_message_pings_unless_the_dm_is_muted_and_blocked_people_never_do() {
        let s = setup(after_join(0), Some(after_join(0)), 0);
        s.state
            .apply_event(&event("MESSAGE_CREATE", message(after_join(1), 50, 2)));
        assert_eq!(
            badge(&s, 50),
            ChannelBadge {
                unread: true,
                important: true,
                mentions: 1
            }
        );
        assert_eq!(s.state.read_states.dm_mentions().peek(), 1);

        s.state.apply_event(&event("USER_GUILD_SETTINGS_UPDATE", json!({ "guild_id": null, "channel_overrides": [{ "channel_id": "50", "muted": true }] })));
        s.state
            .apply_event(&event("MESSAGE_CREATE", message(after_join(2), 50, 2)));
        assert_eq!(badge(&s, 50).mentions, 1, "a muted DM no longer pings");
        assert!(!badge(&s, 50).unread, "and shows no dot");

        s.state.apply_event(&event(
            "RELATIONSHIP_ADD",
            json!({ "id": "3", "type": 2, "nickname": null, "user": user(3) }),
        ));
        s.state.apply_event(&event(
            "USER_GUILD_SETTINGS_UPDATE",
            json!({ "guild_id": null, "channel_overrides": [] }),
        ));
        s.state
            .apply_event(&event("MESSAGE_CREATE", message(after_join(3), 50, 3)));
        assert_eq!(
            badge(&s, 50).mentions,
            1,
            "a blocked person's message doesn't ping"
        );
    }

    #[test]
    fn your_own_message_acknowledges_the_channel() {
        let s = setup(after_join(0), Some(after_join(0)), 0);
        s.state
            .apply_event(&event("MESSAGE_CREATE", message(after_join(1), 11, 2)));
        assert!(badge(&s, 11).unread);
        s.state
            .apply_event(&event("MESSAGE_CREATE", message(after_join(2), 11, 1)));
        assert_eq!(badge(&s, 11), ChannelBadge::default());
        assert!(
            s.sent.borrow().is_empty(),
            "the server acknowledges your own message itself"
        );
    }

    #[test]
    fn a_replayed_or_older_message_changes_nothing() {
        let s = setup(after_join(0), Some(after_join(0)), 0);
        let mut ping = message(after_join(5), 11, 2);
        ping["mentions"] = json!([user(1)]);
        s.state.apply_event(&event("MESSAGE_CREATE", ping.clone()));
        s.state.apply_event(&event("MESSAGE_CREATE", ping));
        s.state
            .apply_event(&event("MESSAGE_CREATE", message(after_join(3), 11, 2)));
        assert_eq!(
            badge(&s, 11),
            ChannelBadge {
                unread: true,
                important: true,
                mentions: 1
            },
            "counted once, and an older message doesn't count"
        );
    }

    #[test]
    fn an_individually_muted_channel_has_no_dot_and_a_muted_server_hides_only_the_rail_pill() {
        let s = setup(after_join(10), Some(after_join(0)), 3);
        s.state.apply_event(&event("USER_GUILD_SETTINGS_UPDATE", json!({ "guild_id": "10", "muted": false, "channel_overrides": [{ "channel_id": "11", "muted": true }] })));
        assert_eq!(
            badge(&s, 11),
            ChannelBadge {
                unread: false,
                important: false,
                mentions: 3
            }
        );
        assert_eq!(
            guild(&s),
            GuildBadge {
                unread: false,
                mentions: 3
            }
        );
        s.state.apply_event(&event(
            "USER_GUILD_SETTINGS_UPDATE",
            json!({ "guild_id": "10", "muted": true, "channel_overrides": [] }),
        ));
        assert_eq!(
            guild(&s),
            GuildBadge {
                unread: false,
                mentions: 3
            },
            "a muted server hides the rail unread pill; pings still show"
        );
        assert!(badge(&s, 11).unread, "server mute is not a channel mute");
    }

    #[test]
    fn a_channel_you_cannot_view_is_always_read() {
        let state = AppState::new();
        state.apply_event(&event(
            "READY",
            json!({
                "user": user(1),
                "guilds": [{
                    "id": "10", "joined_at": JOINED, "properties": { "name": "G", "icon": null, "owner_id": "9" },
                    "roles": [{ "id": "10", "name": "@everyone", "permissions": "1024" }],
                    "channels": [{ "id": "11", "guild_id": "10", "type": 0, "name": "secret", "position": 0, "last_message_id": after_join(5).to_string(),
                                   "permission_overwrites": [{ "id": "10", "type": 0, "allow": "0", "deny": "1024" }] }],
                    "members": [{ "user": { "id": "1" }, "roles": [] }],
                }],
                "read_state": { "version": 1, "partial": false, "entries": [{ "id": "11", "last_message_id": after_join(0).to_string(), "mention_count": 4 }] },
            }),
        ));
        let channels = state.channels.by_guild.peek();
        assert_eq!(
            channels.get(&Id::new(10)).map(Vec::len),
            Some(1),
            "the guild parsed"
        );
        let guild = state.guilds.find(Id::new(10)).expect("guild");
        let hidden = &channels[&Id::new(10)][0];
        assert!(
            !state
                .permissions
                .channel_permissions(Some(&guild), hidden)
                .contains(Permissions::VIEW_CHANNEL),
            "the permission store hides it: {hidden:?}"
        );
        assert_eq!(
            state.read_states.badge(Id::new(11)).peek(),
            ChannelBadge::default()
        );
        assert_eq!(
            state.read_states.guild_badge(Id::new(10)).peek(),
            GuildBadge::default()
        );
    }

    #[test]
    fn auto_ack_needs_the_channel_open_focused_at_the_bottom_and_loaded_and_waits_before_telling_the_server()
     {
        let s = setup(after_join(10), Some(after_join(0)), 0);
        let rs = &s.state.read_states;
        let channel = Id::new(11);

        rs.select_channel(Some(channel));
        assert!(!rs.try_auto_ack(channel), "nothing loaded yet");
        s.state
            .set_history(channel, vec![parsed(after_join(10), 11, 2)]);
        assert!(!badge(&s, 11).unread, "loaded, open, at the bottom: read");
        assert!(
            s.sent.borrow().is_empty(),
            "the server hears about it after a pause"
        );
        rs.flush_ack(channel);
        let sent = s.sent.borrow().clone();
        assert!(
            matches!(&sent[..], [AckRequest::Message { channel_id, message_id, flags: Some(1), .. }] if channel_id.get() == 11 && message_id.get() == after_join(10)),
            "{sent:?}"
        );

        rs.set_at_bottom(false);
        s.state
            .apply_event(&event("MESSAGE_CREATE", message(after_join(11), 11, 2)));
        assert!(badge(&s, 11).unread);
        rs.set_at_bottom(true);
        assert!(!badge(&s, 11).unread, "reaching the bottom reads it");
        rs.set_window_focused(false);
        s.state
            .apply_event(&event("MESSAGE_CREATE", message(after_join(12), 11, 2)));
        assert!(
            badge(&s, 11).unread,
            "not while the window is in the background"
        );
        rs.set_window_focused(true);
        assert!(!badge(&s, 11).unread);
    }

    #[test]
    fn a_message_arriving_while_you_read_is_acknowledged_and_goes_to_the_server_after_the_pause() {
        let s = setup(after_join(0), Some(after_join(0)), 0);
        let rs = &s.state.read_states;
        let channel = Id::new(11);
        rs.select_channel(Some(channel));
        s.state
            .set_history(channel, vec![parsed(after_join(0), 11, 2)]);
        let mut ping = message(after_join(1), 11, 2);
        ping["mentions"] = json!([user(1)]);
        s.state.apply_event(&event("MESSAGE_CREATE", ping));
        assert_eq!(
            badge(&s, 11),
            ChannelBadge::default(),
            "read as it arrived: no dot, no ping"
        );
        timer::run_due();
        assert!(
            s.sent.borrow().is_empty(),
            "the server hears of it after the pause, as the client does"
        );
        rs.flush_ack(channel);
        assert_eq!(s.sent.borrow().len(), 1);
    }

    #[test]
    fn opening_a_channel_with_pings_waiting_acknowledges_it_at_once() {
        let s = setup(after_join(5), Some(after_join(0)), 2);
        let rs = &s.state.read_states;
        let channel = Id::new(11);
        rs.select_channel(Some(channel));
        s.state
            .set_history(channel, vec![parsed(after_join(5), 11, 2)]);
        assert_eq!(badge(&s, 11), ChannelBadge::default());

        timer::run_due();
        assert_eq!(s.sent.borrow().len(), 1);
    }

    #[test]
    fn another_session_reading_clears_it_here_and_an_explicit_unread_sets_the_servers_count() {
        let s = setup(after_join(10), Some(after_join(0)), 2);
        s.state.apply_event(&event("MESSAGE_ACK", json!({ "channel_id": "11", "message_id": after_join(10).to_string(), "mention_count": 0, "manual": false })));
        assert_eq!(badge(&s, 11), ChannelBadge::default());
        s.state.apply_event(&event("MESSAGE_ACK", json!({ "channel_id": "11", "message_id": after_join(3).to_string(), "mention_count": 5, "manual": true })));
        assert_eq!(
            badge(&s, 11),
            ChannelBadge {
                unread: true,
                important: true,
                mentions: 5
            }
        );

        s.state.read_states.select_channel(Some(Id::new(11)));
        s.state
            .set_history(Id::new(11), vec![parsed(after_join(10), 11, 2)]);
        assert!(badge(&s, 11).unread);
    }

    #[test]
    fn mark_as_read_acknowledges_a_server_in_one_request_and_everything_for_mark_all() {
        let s = setup(after_join(10), Some(after_join(0)), 1);
        s.state
            .apply_event(&event("MESSAGE_CREATE", message(after_join(20), 50, 2)));
        s.state
            .apply_event(&event("MESSAGE_CREATE", message(after_join(21), 12, 2)));
        let rs = &s.state.read_states;
        rs.mark_guild_read(Id::new(10));
        assert_eq!(guild(&s), GuildBadge::default());
        assert_eq!(badge(&s, 50).mentions, 1, "DMs aren't part of the server");
        let sent = s.sent.borrow().clone();
        assert_eq!(sent.len(), 1);
        let AckRequest::Bulk(channels) = &sent[0] else {
            panic!("expected a bulk request")
        };
        let mut ids: Vec<u64> = channels.iter().map(|(channel, _)| channel.get()).collect();
        ids.sort_unstable();
        assert_eq!(ids, [11, 12]);

        rs.mark_all_read();
        assert_eq!(rs.total_mentions().peek(), 0);
        assert_eq!(badge(&s, 50), ChannelBadge::default());
        assert_eq!(s.sent.borrow().len(), 2);
    }

    #[test]
    fn history_finds_the_first_unread_message_and_counts_the_unread_ones() {
        let s = setup(after_join(30), Some(after_join(10)), 0);
        let channel = Id::new(11);

        s.state.set_history(
            channel,
            vec![
                parsed(after_join(30), 11, 2),
                parsed(after_join(20), 11, 2),
                parsed(after_join(10), 11, 2),
                parsed(after_join(5), 11, 2),
            ],
        );
        let marker = s
            .state
            .read_states
            .new_marker(channel)
            .expect("unread, so a marker");
        assert_eq!(
            (marker.first_unread, marker.count, marker.estimated),
            (Some(after_join(20)), 2, false)
        );
    }

    #[test]
    fn a_page_made_only_of_unread_messages_may_not_be_all_of_them() {
        let s = setup(after_join(30), Some(after_join(0)), 0);
        s.state.set_history(
            Id::new(11),
            vec![parsed(after_join(30), 11, 2), parsed(after_join(20), 11, 2)],
        );
        let marker = s.state.read_states.new_marker(Id::new(11)).unwrap();
        assert_eq!((marker.count, marker.estimated), (2, true));
    }

    #[test]
    fn reset_forgets_everything() {
        let s = setup(after_join(10), Some(after_join(0)), 2);
        let _ = badge(&s, 11);
        s.state.reset();
        assert_eq!(badge(&s, 11), ChannelBadge::default());
        assert_eq!(s.state.read_states.total_mentions().peek(), 0);
    }
}
