use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::wire::PresenceWire;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::primitives::PresenceStatus;

use super::{Action, Store};
use crate::models::Id;
use crate::models::id::marker::UserMarker;
use crate::presence::CustomStatus;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UserPresence {
    pub status: PresenceStatus,
    pub custom_status: Option<CustomStatus>,
}

#[derive(Clone)]
pub struct PresenceStore {
    pub by_user: Signal<HashMap<Id<UserMarker>, UserPresence>>,

    /// One signal per user someone is watching, so a presence change only wakes that user's readers.
    watched: Rc<RefCell<HashMap<Id<UserMarker>, Signal<UserPresence>>>>,
}

fn parse(entry: &PresenceWire) -> (Id<UserMarker>, UserPresence) {
    let status = PresenceStatus::from_gateway(entry.status.as_deref().unwrap_or("offline"));
    let custom_status = CustomStatus::from_activities(&entry.activities)
        .filter(|_| status != PresenceStatus::Offline);
    (
        entry.user.id,
        UserPresence {
            status,
            custom_status,
        },
    )
}

impl Default for PresenceStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PresenceStore {
    pub fn new() -> Self {
        Self {
            by_user: signal(HashMap::new()),
            watched: Rc::default(),
        }
    }

    pub fn of(&self, user_id: Id<UserMarker>) -> UserPresence {
        self.watch(user_id).get()
    }

    fn watch(&self, user_id: Id<UserMarker>) -> Signal<UserPresence> {
        if let Some(existing) = self.watched.borrow().get(&user_id) {
            return existing.clone();
        }
        let current = self
            .by_user
            .with_untracked(|all| all.get(&user_id).cloned().unwrap_or_default());
        let watched = signal(current);
        self.watched.borrow_mut().insert(user_id, watched.clone());
        watched
    }

    fn set(&self, id: Id<UserMarker>, presence: UserPresence) {
        if self.by_user.with_untracked(|all| {
            all.get(&id) != Some(&presence)
                && (presence.status != PresenceStatus::Offline || all.contains_key(&id))
        }) {
            let watched = self.watched.borrow().get(&id).cloned();
            self.by_user.update(|all| {
                all.insert(id, presence.clone());
            });
            if let Some(watched) = watched {
                watched.set(presence);
            }
        }
    }
}

impl Store for PresenceStore {
    fn handle(&self, action: &Action) {
        match action {
            Action::ReadySupplemental(friends) => {
                log::debug!("presences: {} from READY_SUPPLEMENTAL", friends.len());
                for (id, presence) in friends.iter().map(|friend| parse(&friend.presence)) {
                    self.set(id, presence);
                }
            }
            Action::PresenceUpdate(data) => {
                let (id, presence) = parse(data);
                self.set(id, presence);
            }
            _ => {}
        }
    }

    fn reset(&self) {
        self.by_user.set(HashMap::new());
        for watched in self.watched.borrow().values() {
            watched.set(UserPresence::default());
        }
    }
}
