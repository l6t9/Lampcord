use std::cell::Cell;
use std::mem;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use requiem_ui::signal::{Signal, signal};

use super::{Action, Store};
use crate::models::Id;
use crate::models::id::marker::{ChannelMarker, GuildMarker, UserMarker};

#[derive(Clone, Debug)]
pub enum NavEvent {
    FriendRequestSent {
        username: String,
        error: Option<String>,
    },

    DmOpened {
        channel_id: Id<ChannelMarker>,
        recipients: Vec<Id<UserMarker>>,
    },

    GuildFlow {
        create: bool,
        guild_id: Option<Id<GuildMarker>>,
        error: Option<String>,
    },

    ChannelCreated {
        channel_id: Option<Id<ChannelMarker>>,
        error: Option<String>,
    },

    OpenChannel(Id<ChannelMarker>),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FriendsTab {
    #[default]
    Online,
    All,
    Pending,
    Blocked,
    AddFriend,
}

#[derive(Clone)]
pub struct NavStore {
    pub home: Signal<bool>,
    pub friends_tab: Signal<FriendsTab>,

    pub friends_search: Signal<String>,

    pub add_friend_result: Signal<Option<(String, bool)>>,
    inbox: Arc<Mutex<Vec<NavEvent>>>,

    pub pending_guild: Rc<Cell<Option<Id<GuildMarker>>>>,
}

impl Default for NavStore {
    fn default() -> Self {
        Self::new()
    }
}

impl NavStore {
    pub fn new() -> Self {
        Self {
            home: signal(true),
            friends_tab: signal(FriendsTab::default()),
            friends_search: signal(String::new()),
            add_friend_result: signal(None),
            inbox: Arc::new(Mutex::new(Vec::new())),
            pending_guild: Rc::new(Cell::new(None)),
        }
    }

    pub fn inbox(&self) -> Arc<Mutex<Vec<NavEvent>>> {
        self.inbox.clone()
    }

    pub fn take_inbox(&self) -> Vec<NavEvent> {
        mem::take(&mut *self.inbox.lock().expect("nav inbox poisoned"))
    }
}

impl Store for NavStore {
    fn handle(&self, _action: &Action) {}

    fn reset(&self) {
        self.home.set(true);
        self.friends_tab.set(FriendsTab::default());
        self.friends_search.set(String::new());
        self.add_friend_result.set(None);
        self.take_inbox();
        self.pending_guild.set(None);
    }
}
