use requiem_ui::signal::{Signal, signal};
use std::cmp;

use super::wire::ChannelWire;
use super::{Action, Store};
use crate::models::id::marker::{ChannelMarker, MessageMarker, UserMarker};
use crate::models::{Channel, Id};

#[derive(Clone, Debug)]
pub struct DmChannel {
    pub id: Id<ChannelMarker>,
    pub kind: u8,

    pub name: Option<String>,

    pub recipients: Vec<Id<UserMarker>>,
    pub last_message_id: Option<Id<MessageMarker>>,
}

impl DmChannel {
    pub fn is_group(&self) -> bool {
        self.kind == 3
    }

    pub fn as_channel(&self) -> Channel {
        Channel {
            id: self.id,
            guild_id: None,
            kind: self.kind,
            name: self.name.clone(),
            position: None,
            parent_id: None,
            permission_overwrites: Vec::new(),
            nsfw: false,
            flags: 0,
            last_message_id: self.last_message_id,
            owner_id: None,
            thread_metadata: None,
            message_count: 0,
            member_count: 0,
            total_message_sent: 0,
            member: None,
            newly_created: false,
        }
    }
}

#[derive(Clone)]
pub struct DmStore {
    pub list: Signal<Vec<DmChannel>>,
}

impl Default for DmStore {
    fn default() -> Self {
        Self::new()
    }
}

impl DmStore {
    pub fn new() -> Self {
        Self {
            list: signal(Vec::new()),
        }
    }

    pub fn find(&self, id: Id<ChannelMarker>) -> Option<DmChannel> {
        self.list
            .with_untracked(|list| list.iter().find(|c| c.id == id).cloned())
    }

    pub fn with_user(&self, user_id: Id<UserMarker>) -> Option<DmChannel> {
        self.list.with_untracked(|list| {
            list.iter()
                .find(|c| c.kind == 1 && c.recipients == [user_id])
                .cloned()
        })
    }

    fn sort(list: &mut [DmChannel]) {
        list.sort_by_key(|c| cmp::Reverse(c.last_message_id.map_or(c.id.get(), |m| m.get())));
    }

    fn upsert(&self, channel: DmChannel) {
        self.list.update(|list| {
            match list.iter_mut().find(|c| c.id == channel.id) {
                Some(slot) => *slot = channel,
                None => list.push(channel),
            }
            Self::sort(list);
        });
    }
}

impl Store for DmStore {
    fn handle(&self, action: &Action) {
        match action {
            Action::ConnectionOpen { ready, .. } => {
                let mut list: Vec<DmChannel> = ready
                    .private_channels
                    .iter()
                    .filter_map(ChannelWire::dm)
                    .collect();
                Self::sort(&mut list);
                self.list.set(list);
            }
            Action::ChannelCreate(data) | Action::ChannelUpdate(data) => {
                if let Some(channel) = data.dm() {
                    self.upsert(channel);
                }
            }
            Action::ChannelDelete(data) => {
                let id = data.id;
                self.list.update(|list| list.retain(|c| c.id != id));
            }
            Action::MessageCreate(message) => {
                if let Some(mut channel) = self.find(message.channel_id) {
                    channel.last_message_id = Some(message.id);
                    self.upsert(channel);
                }
            }
            _ => {}
        }
    }

    fn reset(&self) {
        self.list.set(Vec::new());
    }
}
