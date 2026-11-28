use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use requiem_ui::signal::{KeyedSignal, Signal, signal};

use super::{Action, Store};
use crate::models::id::marker::UserMarker;
use crate::models::user_profile::{MutualFriend, ProfileUser};
use crate::models::{Id, Message, User};

#[derive(Clone)]
pub struct UserStore {
    pub current: Signal<Option<User>>,
    known: Rc<RefCell<HashMap<Id<UserMarker>, Entry>>>,

    version: Signal<u64>,
    watched: KeyedSignal<Id<UserMarker>, ()>,

    /// Who you and each user both have as a friend, kept from the profiles that ask for it.
    mutual_friends: KeyedSignal<Id<UserMarker>, Vec<MutualFriend>>,
    mutual_friends_tried: Rc<RefCell<HashSet<Id<UserMarker>>>>,
}

struct Entry {
    user: User,
    authoritative: bool,
}

fn is_placeholder(user: &User) -> bool {
    user.username == "unknown" && user.global_name.is_none() && user.avatar.is_none()
}

fn same_identity(a: &User, b: &User) -> bool {
    a.primary_guild == b.primary_guild
        && a.username == b.username
        && a.global_name == b.global_name
        && a.avatar == b.avatar
        && a.discriminator == b.discriminator
        && a.bot == b.bot
        && a.avatar_decoration_data == b.avatar_decoration_data
        && a.collectibles == b.collectibles
        && a.display_name_styles == b.display_name_styles
}

impl Default for UserStore {
    fn default() -> Self {
        Self::new()
    }
}

impl UserStore {
    pub fn new() -> Self {
        Self {
            current: signal(None),
            known: Rc::new(RefCell::new(HashMap::new())),
            version: signal(0),
            watched: KeyedSignal::new(),
            mutual_friends: KeyedSignal::new(),
            mutual_friends_tried: Rc::default(),
        }
    }

    /// The stored list, or `None` if no profile has brought one.
    pub fn mutual_friends_of(&self, user_id: Id<UserMarker>) -> Option<Vec<MutualFriend>> {
        self.mutual_friends.watch(&user_id);
        self.mutual_friends
            .with_untracked(|all| all.get(&user_id).cloned())
    }

    /// Whether the list for `user_id` should be fetched: the profile reports friends and the
    /// stored list is missing or a different length, and it hasn't been tried this session.
    /// Marks the user as tried.
    pub fn should_fetch_mutual_friends(&self, user_id: Id<UserMarker>, count: u32) -> bool {
        if count == 0 {
            return false;
        }
        let complete = self.mutual_friends.with_untracked(|all| {
            all.get(&user_id)
                .is_some_and(|friends| friends.len() == count as usize)
        });
        !complete && self.mutual_friends_tried.borrow_mut().insert(user_id)
    }

    pub fn set_mutual_friends(&self, user_id: Id<UserMarker>, friends: &[MutualFriend]) {
        let unchanged = self.mutual_friends.with_untracked(|all| {
            all.get(&user_id).is_some_and(|known| {
                known.len() == friends.len()
                    && known.iter().zip(friends).all(|(a, b)| a.id == b.id)
            })
        });
        if !unchanged {
            self.mutual_friends.update_key(&user_id, |all| {
                all.insert(user_id, friends.to_vec());
            });
        }
    }

    pub fn observe(&self, user: &User) {
        let (changed, first_sighting) = {
            let mut known = self.known.borrow_mut();
            match known.get_mut(&user.id) {
                None => {
                    known.insert(
                        user.id,
                        Entry {
                            user: user.clone(),
                            authoritative: false,
                        },
                    );
                    (false, true)
                }
                Some(entry) => {
                    let before = entry.user.clone();
                    if entry.authoritative {
                        entry.user.global_name = entry
                            .user
                            .global_name
                            .take()
                            .or_else(|| user.global_name.clone());
                        entry.user.avatar =
                            entry.user.avatar.take().or_else(|| user.avatar.clone());
                        entry.user.primary_guild = user
                            .primary_guild
                            .clone()
                            .or_else(|| entry.user.primary_guild.take());
                        entry.user.avatar_decoration_data = user
                            .avatar_decoration_data
                            .clone()
                            .or_else(|| entry.user.avatar_decoration_data.take());
                        entry.user.collectibles = user
                            .collectibles
                            .clone()
                            .or_else(|| entry.user.collectibles.take());
                        entry.user.display_name_styles = user
                            .display_name_styles
                            .clone()
                            .or_else(|| entry.user.display_name_styles.take());
                    } else if !is_placeholder(user) {
                        let merged = User {
                            global_name: user
                                .global_name
                                .clone()
                                .or_else(|| entry.user.global_name.take()),
                            avatar: user.avatar.clone().or_else(|| entry.user.avatar.take()),
                            ..user.clone()
                        };
                        entry.user = if is_placeholder(&before) {
                            user.clone()
                        } else {
                            merged
                        };
                    }
                    (!same_identity(&before, &entry.user), false)
                }
            }
        };
        if changed {
            self.version.update(|v| *v += 1);
        }
        if changed || first_sighting {
            self.watched.update_key(&user.id, |_| {});
        }
    }

    fn observe_authoritative(&self, user: &User) {
        let (changed, first_sighting) = {
            let mut known = self.known.borrow_mut();
            let previous = known.get(&user.id);
            let changed = previous.is_some_and(|entry| !same_identity(&entry.user, user));
            let first_sighting = previous.is_none();
            known.insert(
                user.id,
                Entry {
                    user: user.clone(),
                    authoritative: true,
                },
            );
            (changed, first_sighting)
        };
        if changed {
            self.version.update(|v| *v += 1);
        }
        if changed || first_sighting {
            self.watched.update_key(&user.id, |_| {});
        }
    }

    pub fn get_watched(&self, id: Id<UserMarker>) -> Option<User> {
        self.watched.watch(&id);
        self.known.borrow().get(&id).map(|entry| entry.user.clone())
    }

    pub fn observe_profile(&self, profile: &ProfileUser) {
        self.observe_authoritative(&User {
            id: profile.id,
            username: profile.username.clone(),
            discriminator: profile.discriminator.clone(),
            global_name: profile.global_name.clone(),
            avatar: profile.avatar.clone(),
            email: None,
            phone: None,
            verified: None,
            mfa_enabled: None,
            bot: profile.bot,
            premium_type: None,
            primary_guild: profile.primary_guild.clone(),
            avatar_decoration_data: profile.avatar_decoration_data.clone(),
            collectibles: profile.collectibles.clone(),
            display_name_styles: profile.display_name_styles.clone(),
        });
    }

    fn observe_all<'a>(&self, users: impl IntoIterator<Item = &'a User>) {
        for user in users {
            self.observe(user);
        }
    }

    pub fn get(&self, id: Id<UserMarker>) -> Option<User> {
        self.version.with(|_| ());
        self.known.borrow().get(&id).map(|entry| entry.user.clone())
    }

    pub fn observe_messages<'a>(&self, messages: impl IntoIterator<Item = &'a Message>) {
        for message in messages {
            self.observe(&message.author);
            message.mentions.iter().for_each(|user| self.observe(user));
        }
    }

    pub fn resolve(&self, user: &User) -> User {
        self.version.with(|_| ());
        self.known
            .borrow()
            .get(&user.id)
            .map_or_else(|| user.clone(), |entry| entry.user.clone())
    }

    pub fn display_name(&self, user: &User) -> String {
        self.resolve(user).display_name(None)
    }
}

impl Store for UserStore {
    fn handle(&self, action: &Action) {
        match action {
            Action::ConnectionOpen { user, ready, .. } => {
                self.observe_authoritative(user);
                self.current.set(Some(user.clone()));

                self.observe_all(&ready.users);
                self.observe_all(ready.relationships.iter().filter_map(|r| r.user.as_ref()));
            }
            Action::ReadySupplemental(friends) => {
                self.observe_all(friends.iter().filter_map(|friend| friend.user.as_ref()))
            }
            Action::RelationshipAdd(data) => self.observe_all(data.user.as_ref()),
            Action::ChannelCreate(data) | Action::ChannelUpdate(data) => {
                self.observe_all(&data.recipients)
            }
            Action::UserUpdate(user) => {
                self.observe_authoritative(user);

                let merged = self
                    .current
                    .peek()
                    .filter(|current| current.id == user.id)
                    .map(|current| User {
                        email: user.email.clone().or_else(|| current.email.clone()),
                        phone: user.phone.clone().or_else(|| current.phone.clone()),
                        verified: user.verified.or(current.verified),
                        mfa_enabled: user.mfa_enabled.or(current.mfa_enabled),
                        premium_type: user.premium_type.or(current.premium_type),
                        ..user.clone()
                    });
                if let Some(merged) = merged {
                    let unchanged = self.current.peek().is_some_and(|current| {
                        same_identity(&current, &merged)
                            && current.premium_type == merged.premium_type
                            && current.email == merged.email
                            && current.phone == merged.phone
                            && current.verified == merged.verified
                            && current.mfa_enabled == merged.mfa_enabled
                    });
                    if !unchanged {
                        self.current.set(Some(merged));
                    }
                }
            }
            Action::MessageCreate(message) | Action::MessageUpdate(message) => {
                self.observe_messages([message])
            }
            Action::MemberListUpdate(update) => {
                self.observe_all(update.members().map(|member| &member.user))
            }
            _ => {}
        }
    }

    fn reset(&self) {
        self.current.set(None);
        self.known.borrow_mut().clear();
        self.mutual_friends.set(HashMap::new());
        self.mutual_friends_tried.borrow_mut().clear();
        self.version.update(|v| *v += 1);
        self.watched.update(|_| {});
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(id: u64, username: &str, global: Option<&str>, avatar: Option<&str>) -> User {
        User {
            id: Id::new(id),
            username: username.into(),
            discriminator: "0".into(),
            global_name: global.map(Into::into),
            avatar: avatar.map(Into::into),
            email: None,
            phone: None,
            verified: None,
            mfa_enabled: None,
            bot: false,
            premium_type: None,
            primary_guild: None,
            avatar_decoration_data: None,
            collectibles: None,
            display_name_styles: None,
        }
    }

    fn profile(id: u64, username: &str, global: Option<&str>, avatar: Option<&str>) -> ProfileUser {
        serde_json::from_value(serde_json::json!({ "id": id.to_string(), "username": username, "global_name": global, "avatar": avatar })).unwrap()
    }

    #[test]
    fn a_placeholder_never_overwrites_real_data_and_real_data_replaces_a_placeholder() {
        let store = UserStore::new();
        store.observe(&user(1, "unknown", None, None));
        assert_eq!(
            store.resolve(&user(1, "unknown", None, None)).username,
            "unknown"
        );
        store.observe(&user(1, "ada", Some("Ada"), Some("hash")));
        let resolved = store.resolve(&user(1, "unknown", None, None));
        assert_eq!(
            (
                resolved.username.as_str(),
                resolved.global_name.as_deref(),
                resolved.avatar.as_deref()
            ),
            ("ada", Some("Ada"), Some("hash"))
        );
        store.observe(&user(1, "unknown", None, None));
        assert_eq!(store.resolve(&user(1, "x", None, None)).username, "ada");
    }

    #[test]
    fn a_snapshot_missing_display_name_or_avatar_keeps_what_is_known() {
        let store = UserStore::new();
        store.observe(&user(1, "ada", Some("Ada"), Some("hash")));
        store.observe(&user(1, "ada", None, None));
        let resolved = store.resolve(&user(1, "ada", None, None));
        assert_eq!(
            (resolved.global_name.as_deref(), resolved.avatar.as_deref()),
            (Some("Ada"), Some("hash"))
        );
    }

    #[test]
    fn a_profile_is_authoritative_and_later_snapshots_cannot_regress_it() {
        let store = UserStore::new();
        store.observe(&user(1, "ada", None, None));
        store.observe_profile(&profile(1, "ada", Some("Ada L."), Some("new")));

        store.observe(&user(1, "ada", Some("Old Name"), Some("old")));
        let resolved = store.resolve(&user(1, "ada", None, None));
        assert_eq!(
            (resolved.global_name.as_deref(), resolved.avatar.as_deref()),
            (Some("Ada L."), Some("new"))
        );
        assert_eq!(store.display_name(&user(1, "ada", None, None)), "Ada L.");
    }

    #[test]
    fn version_bumps_only_when_a_known_user_visibly_changes() {
        let store = UserStore::new();
        store.observe(&user(1, "unknown", None, None));
        assert_eq!(store.version.peek(), 0, "first sighting is silent");
        store.observe(&user(1, "unknown", None, None));
        assert_eq!(store.version.peek(), 0, "no change");
        store.observe(&user(1, "ada", Some("Ada"), None));
        assert_eq!(store.version.peek(), 1, "placeholder corrected");
        store.observe_profile(&profile(1, "ada", Some("Ada"), None));
        assert_eq!(
            store.version.peek(),
            1,
            "profile agreeing with what's known changes nothing visible"
        );
        store.observe_profile(&profile(1, "ada", Some("Ada"), None));
        assert_eq!(
            store.version.peek(),
            1,
            "re-observing the same profile every frame is free"
        );
    }
}
