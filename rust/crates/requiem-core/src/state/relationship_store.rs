use super::wire::RelationshipWire;
use requiem_ui::signal::{Signal, signal};

use super::{Action, Store};
use crate::models::id::marker::UserMarker;
use crate::models::{Id, User};
use crate::rest::endpoints::relationships::{BLOCKED, FRIEND, PENDING_INCOMING, PENDING_OUTGOING};

#[derive(Clone, Debug)]
pub struct Relationship {
    pub id: Id<UserMarker>,
    pub kind: u8,
    pub user: User,
    pub nickname: Option<String>,
}

impl Relationship {
    pub fn is_friend(&self) -> bool {
        self.kind == FRIEND
    }
    pub fn is_incoming(&self) -> bool {
        self.kind == PENDING_INCOMING
    }
    pub fn is_outgoing(&self) -> bool {
        self.kind == PENDING_OUTGOING
    }
    pub fn is_blocked(&self) -> bool {
        self.kind == BLOCKED
    }

    /// READY dedupes user objects: a relationship then carries only the user's `id`, and the
    /// user is in the payload's `users`. `known` is that list.
    fn from_wire(wire: &RelationshipWire, known: &[User]) -> Option<Self> {
        let user = wire.user.clone().or_else(|| {
            let id = wire.id?;
            known.iter().find(|user| user.id == id).cloned()
        })?;
        Some(Self {
            id: user.id,
            kind: wire.kind?,
            nickname: wire.nickname.clone().flatten(),
            user,
        })
    }
}

#[derive(Clone)]
pub struct RelationshipStore {
    pub list: Signal<Vec<Relationship>>,
}

impl Default for RelationshipStore {
    fn default() -> Self {
        Self::new()
    }
}

impl RelationshipStore {
    pub fn new() -> Self {
        Self {
            list: signal(Vec::new()),
        }
    }

    pub fn kind_of(&self, user_id: Id<UserMarker>) -> Option<u8> {
        self.list
            .with_untracked(|list| list.iter().find(|r| r.id == user_id).map(|r| r.kind))
    }

    fn upsert(&self, rel: Relationship) {
        self.list
            .update(|list| match list.iter_mut().find(|r| r.id == rel.id) {
                Some(slot) => *slot = rel,
                None => list.push(rel),
            });
    }
}

impl Store for RelationshipStore {
    fn handle(&self, action: &Action) {
        match action {
            Action::ConnectionOpen { ready, .. } => {
                let list = ready
                    .relationships
                    .iter()
                    .filter_map(|wire| Relationship::from_wire(wire, &ready.users))
                    .collect();
                self.list.set(list);
            }
            Action::RelationshipAdd(data) => {
                if let Some(rel) = Relationship::from_wire(data, &[]) {
                    self.upsert(rel);
                }
            }

            Action::RelationshipUpdate(data) => {
                let Some(id) = data.id else { return };
                let kind = data.kind;
                let nickname = data.nickname.clone();
                self.list.update(|list| {
                    if let Some(rel) = list.iter_mut().find(|r| r.id == id) {
                        rel.kind = kind.unwrap_or(rel.kind);
                        if let Some(nickname) = nickname {
                            rel.nickname = nickname;
                        }
                    }
                });
            }
            Action::RelationshipRemove(data) => {
                if let Some(id) = data.id {
                    self.list.update(|list| list.retain(|r| r.id != id));
                }
            }
            _ => {}
        }
    }

    fn reset(&self) {
        self.list.set(Vec::new());
    }
}
