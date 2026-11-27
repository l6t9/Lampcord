use std::collections::HashMap;

use requiem_ui::signal::{Signal, signal};

use super::{Action, Store};
use crate::models::Id;
use crate::models::id::marker::ChannelMarker;

#[derive(Clone)]
pub struct DraftStore {
    drafts: Signal<HashMap<Id<ChannelMarker>, String>>,
}

impl Default for DraftStore {
    fn default() -> Self {
        Self::new()
    }
}

impl DraftStore {
    pub fn new() -> Self {
        Self {
            drafts: signal(HashMap::new()),
        }
    }

    pub fn text(&self, channel: Id<ChannelMarker>) -> String {
        self.drafts
            .with_untracked(|drafts| drafts.get(&channel).cloned().unwrap_or_default())
    }

    pub fn set(&self, channel: Id<ChannelMarker>, text: &str) {
        let unchanged = self
            .drafts
            .with_untracked(|drafts| drafts.get(&channel).map_or(text.is_empty(), |d| d == text));
        if unchanged {
            return;
        }
        self.drafts.update(|drafts| {
            if text.is_empty() {
                drafts.remove(&channel);
            } else {
                drafts.insert(channel, text.to_string());
            }
        });
    }

    pub fn clear(&self, channel: Id<ChannelMarker>) {
        self.set(channel, "");
    }
}

impl Store for DraftStore {
    fn handle(&self, action: &Action) {
        if let Action::ChannelDelete(channel) = action {
            self.clear(channel.id);
        }
    }

    fn reset(&self) {
        self.drafts.set(HashMap::new());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drafts_are_per_channel() {
        let store = DraftStore::new();
        let (a, b) = (Id::new(1), Id::new(2));
        store.set(a, "```rust\nfn a() {}");
        store.set(b, "hello");
        assert_eq!(store.text(a), "```rust\nfn a() {}");
        assert_eq!(store.text(b), "hello");
        store.clear(a);
        assert_eq!(store.text(a), "");
        assert_eq!(store.text(b), "hello");
        store.reset();
        assert_eq!(store.text(b), "");
    }
}
