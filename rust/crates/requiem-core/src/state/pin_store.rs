use std::collections::HashMap;

use requiem_ui::signal::{Signal, signal};

use super::{Action, Store};
use crate::models::id::marker::{ChannelMarker, MessageMarker};
use crate::models::{Id, Message};

#[derive(Clone, Debug)]
pub struct Pin {
    pub pinned_at: String,
    pub message: Message,
}

#[derive(Clone, Debug, Default)]
pub struct ChannelPins {
    pub items: Vec<Pin>,

    pub has_more: bool,

    pub loading: bool,

    pub stale: bool,

    pub failed: bool,

    pub known: bool,
}

#[derive(Clone)]
pub struct PinStore {
    pub by_channel: Signal<HashMap<Id<ChannelMarker>, ChannelPins>>,
}

impl PinStore {
    pub fn new() -> Self {
        Self {
            by_channel: signal(HashMap::new()),
        }
    }

    pub fn of(&self, channel_id: Id<ChannelMarker>) -> ChannelPins {
        self.by_channel
            .with(|all| all.get(&channel_id).cloned())
            .unwrap_or_default()
    }

    pub fn needs_fetch(&self, channel_id: Id<ChannelMarker>) -> bool {
        self.by_channel.with_untracked(|all| {
            all.get(&channel_id)
                .is_none_or(|pins| !pins.loading && !pins.failed && (!pins.known || pins.stale))
        })
    }

    pub fn start_loading(&self, channel_id: Id<ChannelMarker>) {
        self.by_channel.update(|all| {
            let pins = all.entry(channel_id).or_default();
            pins.loading = true;
            pins.failed = false;
        });
    }

    pub fn loaded(
        &self,
        channel_id: Id<ChannelMarker>,
        page: Vec<Pin>,
        has_more: bool,
        reset: bool,
    ) {
        self.by_channel.update(|all| {
            let pins = all.entry(channel_id).or_default();
            if reset {
                pins.items = page;
            } else {
                pins.items.extend(page);
            }
            pins.has_more = has_more;
            pins.loading = false;
            pins.stale = false;
            pins.failed = false;
            pins.known = true;
        });
    }

    pub fn failed(&self, channel_id: Id<ChannelMarker>) {
        self.by_channel.update(|all| {
            let pins = all.entry(channel_id).or_default();
            pins.loading = false;
            pins.failed = true;
        });
    }

    pub fn remove(&self, channel_id: Id<ChannelMarker>, message_id: Id<MessageMarker>) {
        self.by_channel.update(|all| {
            if let Some(pins) = all.get_mut(&channel_id) {
                pins.items.retain(|pin| pin.message.id != message_id);
            }
        });
    }

    fn mark_stale(&self, channel_id: Id<ChannelMarker>) {
        if self
            .by_channel
            .with_untracked(|all| all.contains_key(&channel_id))
        {
            self.by_channel.update(|all| {
                if let Some(pins) = all.get_mut(&channel_id) {
                    pins.stale = true;
                }
            });
        }
    }
}

impl Default for PinStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for PinStore {
    fn handle(&self, action: &Action) {
        match action {
            Action::ChannelPinsUpdate { channel_id, .. } => self.mark_stale(*channel_id),
            Action::MessageUpdate(message) => {
                let found = self.by_channel.with_untracked(|all| {
                    all.get(&message.channel_id).is_some_and(|pins| {
                        pins.items.iter().any(|pin| pin.message.id == message.id)
                    })
                });
                if found {
                    self.by_channel.update(|all| {
                        if let Some(pin) = all.get_mut(&message.channel_id).and_then(|pins| {
                            pins.items
                                .iter_mut()
                                .find(|pin| pin.message.id == message.id)
                        }) {
                            pin.message = message.clone();
                        }
                    });
                }
            }
            Action::MessageDelete { id, channel_id }
                if self.by_channel.with_untracked(|all| {
                    all.get(channel_id)
                        .is_some_and(|pins| pins.items.iter().any(|pin| pin.message.id == *id))
                }) => {
                    self.remove(*channel_id, *id);
                }
            _ => {}
        }
    }

    fn reset(&self) {
        self.by_channel.set(HashMap::new());
    }
}
