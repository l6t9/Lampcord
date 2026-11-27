use crate::models::message::message_type;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use requiem_ui::signal::{Signal, signal};

use super::{Action, Store};
use crate::models::id::marker::UserMarker;
use crate::models::id::marker::{ChannelMarker, MessageMarker};
use crate::models::{Id, Message};
use crate::models::{Reaction, ReactionEmoji};
use crate::rest::endpoints::messages::{INITIAL_PAGE_SIZE, OLDER_PAGE_SIZE};

pub const MAX_LOADED_MESSAGES: usize = 200;
pub const TRUNCATED_VIEW_SIZE: usize = MAX_LOADED_MESSAGES / 2;

#[derive(Clone)]
pub struct MessageStore {
    pub by_channel: Signal<HashMap<Id<ChannelMarker>, Vec<Message>>>,

    pub exhausted: Signal<HashSet<Id<ChannelMarker>>>,

    pub detached: Signal<HashSet<Id<ChannelMarker>>>,

    me: Signal<Option<Id<UserMarker>>>,

    revisions: Rc<RefCell<HashMap<Id<ChannelMarker>, Signal<u64>>>>,

    snapshots: Rc<RefCell<HashMap<Id<ChannelMarker>, (u64, Rc<Vec<Message>>)>>>,
}

impl Default for MessageStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageStore {
    pub fn new() -> Self {
        Self {
            by_channel: signal(HashMap::new()),
            exhausted: signal(HashSet::new()),
            detached: signal(HashSet::new()),
            me: signal(None),
            revisions: Rc::default(),
            snapshots: Rc::default(),
        }
    }

    pub fn channel_revision(&self, channel_id: Id<ChannelMarker>) -> Signal<u64> {
        self.revisions
            .borrow_mut()
            .entry(channel_id)
            .or_insert_with(|| signal(0))
            .clone()
    }

    pub fn in_channel(&self, channel_id: Id<ChannelMarker>) -> Rc<Vec<Message>> {
        let revision = self.channel_revision(channel_id).get();
        if let Some((at, snapshot)) = self.snapshots.borrow().get(&channel_id) {
            if *at == revision {
                return snapshot.clone();
            }
        }
        let snapshot =
            Rc::new(self.by_channel.with_untracked(|by_channel| {
                by_channel.get(&channel_id).cloned().unwrap_or_default()
            }));
        self.snapshots
            .borrow_mut()
            .insert(channel_id, (revision, snapshot.clone()));
        snapshot
    }

    fn bump(&self, channel_id: Id<ChannelMarker>) {
        if let Some(revision) = self.revisions.borrow().get(&channel_id) {
            revision.update(|n| *n += 1);
        }
    }

    pub fn insert(&self, message: Message) {
        if self
            .detached
            .with_untracked(|detached| detached.contains(&message.channel_id))
        {
            return;
        }
        if self.by_channel.with_untracked(|by_channel| {
            by_channel
                .get(&message.channel_id)
                .is_some_and(|list| list.iter().any(|m| m.id == message.id))
        }) {
            return;
        }
        let channel_id = message.channel_id;
        self.by_channel.update(|by_channel| {
            by_channel.entry(channel_id).or_default().push(message);
        });
        self.bump(channel_id);
    }

    /// True while the channel shows a window that isn't the latest page (a jump target or a
    /// truncated view), which a late initial fetch must not replace.
    pub fn is_detached(&self, channel_id: Id<ChannelMarker>) -> bool {
        self.detached
            .with_untracked(|detached| detached.contains(&channel_id))
    }

    pub fn set_history(&self, channel_id: Id<ChannelMarker>, mut messages: Vec<Message>) {
        if self.detached.with_untracked(|d| d.contains(&channel_id)) {
            self.detached.update(|d| {
                d.remove(&channel_id);
            });
        }
        self.note_page_size(channel_id, messages.len(), INITIAL_PAGE_SIZE);
        messages.reverse();
        self.by_channel.update(|by_channel| {
            by_channel.insert(channel_id, messages);
        });
        self.bump(channel_id);
    }

    pub fn replace_window(&self, channel_id: Id<ChannelMarker>, mut page: Vec<Message>) {
        page.reverse();
        self.exhausted.update(|set| {
            set.remove(&channel_id);
        });
        self.detached.update(|set| {
            set.insert(channel_id);
        });
        self.by_channel.update(|by_channel| {
            by_channel.insert(channel_id, page);
        });
        self.bump(channel_id);
    }

    pub fn prepend_history(&self, channel_id: Id<ChannelMarker>, page: Vec<Message>) {
        self.note_page_size(channel_id, page.len(), OLDER_PAGE_SIZE);
        let mut truncated = false;
        self.by_channel.update(|by_channel| {
            let existing = by_channel.entry(channel_id).or_default();
            let known: HashSet<_> = existing.iter().map(|m| m.id).collect();
            let mut older: Vec<Message> = page
                .into_iter()
                .rev()
                .filter(|m| !known.contains(&m.id))
                .collect();
            older.append(existing);

            if older.len() > MAX_LOADED_MESSAGES {
                older.truncate(TRUNCATED_VIEW_SIZE);
                truncated = true;
            }
            *existing = older;
        });
        self.bump(channel_id);
        if truncated {
            self.detached.update(|set| {
                set.insert(channel_id);
            });
        }
    }

    fn note_page_size(&self, channel_id: Id<ChannelMarker>, len: usize, asked: u8) {
        if len < usize::from(asked) {
            self.exhausted.update(|set| {
                set.insert(channel_id);
            });
        }
    }

    fn replace(&self, message: &Message) {
        self.by_channel.update(|by_channel| {
            if let Some(existing) = by_channel
                .get_mut(&message.channel_id)
                .and_then(|messages| messages.iter_mut().find(|m| m.id == message.id))
            {
                *existing = message.clone();
            }
        });
        self.bump(message.channel_id);
    }

    fn change_reactions(
        &self,
        channel_id: Id<ChannelMarker>,
        id: Id<MessageMarker>,
        change: impl FnOnce(&mut Vec<Reaction>),
    ) {
        self.by_channel.update(|by_channel| {
            if let Some(message) = by_channel
                .get_mut(&channel_id)
                .and_then(|messages| messages.iter_mut().find(|m| m.id == id))
            {
                change(&mut message.reactions);
            }
        });
        self.bump(channel_id);
    }

    fn react(
        &self,
        channel_id: Id<ChannelMarker>,
        id: Id<MessageMarker>,
        user_id: Id<UserMarker>,
        emoji: ReactionEmoji,
    ) {
        let mine = self.me.peek() == Some(user_id);
        self.change_reactions(channel_id, id, move |reactions| {
            match reactions.iter_mut().find(|r| r.emoji.same_as(&emoji)) {
                Some(existing) => {
                    existing.count += 1;
                    existing.me |= mine;
                }
                None => reactions.push(Reaction {
                    count: 1,
                    me: mine,
                    emoji,
                }),
            }
        });
    }

    fn unreact(
        &self,
        channel_id: Id<ChannelMarker>,
        id: Id<MessageMarker>,
        user_id: Id<UserMarker>,
        emoji: ReactionEmoji,
    ) {
        let mine = self.me.peek() == Some(user_id);
        self.change_reactions(channel_id, id, move |reactions| {
            if let Some(existing) = reactions.iter_mut().find(|r| r.emoji.same_as(&emoji)) {
                existing.count = existing.count.saturating_sub(1);
                if mine {
                    existing.me = false;
                }
            }
            reactions.retain(|r| r.count > 0);
        });
    }

    pub fn is_pinned(&self, channel_id: Id<ChannelMarker>, id: Id<MessageMarker>) -> Option<bool> {
        self.by_channel.with_untracked(|all| {
            all.get(&channel_id)
                .and_then(|list| list.iter().find(|m| m.id == id))
                .map(|m| m.pinned)
        })
    }

    pub fn set_pinned(
        &self,
        channel_id: Id<ChannelMarker>,
        pinned: &[Id<MessageMarker>],
        unpinned: &[Id<MessageMarker>],
        complete: bool,
    ) {
        let mut changed = false;
        self.by_channel.update(|by_channel| {
            for message in by_channel.get_mut(&channel_id).into_iter().flatten() {
                let now = if pinned.contains(&message.id) {
                    true
                } else if unpinned.contains(&message.id) || complete {
                    false
                } else {
                    message.pinned
                };
                if now != message.pinned {
                    message.pinned = now;
                    changed = true;
                }
            }
        });
        if changed {
            self.bump(channel_id);
        }
    }

    fn remove(&self, channel_id: Id<ChannelMarker>, id: Id<MessageMarker>) {
        self.by_channel.update(|by_channel| {
            if let Some(messages) = by_channel.get_mut(&channel_id) {
                messages.retain(|m| m.id != id);
            }
        });
        self.bump(channel_id);
    }
}

impl Store for MessageStore {
    fn handle(&self, action: &Action) {
        match action {
            Action::MessageCreate(message) => {
                if message.kind == message_type::CHANNEL_PINNED_MESSAGE {
                    if let Some(pinned) = message
                        .message_reference
                        .as_ref()
                        .and_then(|r| r.message_id)
                    {
                        self.set_pinned(message.channel_id, &[pinned], &[], false);
                    }
                }
                self.insert(message.clone());
            }

            Action::MessageUpdate(message) => self.replace(message),
            Action::MessageDelete { id, channel_id } => self.remove(*channel_id, *id),
            Action::ConnectionOpen { user, .. } => self.me.set(Some(user.id)),
            Action::ReactionAdd {
                channel_id,
                message_id,
                user_id,
                emoji,
            } => self.react(*channel_id, *message_id, *user_id, emoji.clone()),
            Action::ReactionRemove {
                channel_id,
                message_id,
                user_id,
                emoji,
            } => self.unreact(*channel_id, *message_id, *user_id, emoji.clone()),
            Action::ReactionRemoveAll {
                channel_id,
                message_id,
            } => self.change_reactions(*channel_id, *message_id, Vec::clear),
            Action::ReactionRemoveEmoji {
                channel_id,
                message_id,
                emoji,
            } => self.change_reactions(*channel_id, *message_id, |reactions| {
                reactions.retain(|r| !r.emoji.same_as(emoji))
            }),
            _ => {}
        }
    }

    fn reset(&self) {
        self.by_channel.set(HashMap::new());
        let watched: Vec<_> = self.revisions.borrow().keys().copied().collect();
        for channel_id in watched {
            self.bump(channel_id);
        }
        self.exhausted.set(HashSet::new());
        self.detached.set(HashSet::new());
        self.me.set(None);
    }
}
