use std::cell::Cell;

use crate::rest::endpoints::user_settings;
use crate::utils::time;
use requiem_ui::signal::{Signal, signal};

use super::wire::Activity;
use super::{Action, Store};
use crate::models::Id;
use crate::models::id::marker::UserMarker;
use crate::presence::{CustomStatus, OwnStatus};

#[derive(Clone)]
pub struct SelfPresenceStore {
    pub status: Signal<OwnStatus>,

    pub custom_status: Signal<Option<CustomStatus>>,

    /// Rich presence activities from our own PRESENCE_UPDATE, without the
    /// custom status activity that is stored separately.
    pub activities: Signal<Vec<Activity>>,

    self_id: Cell<Option<Id<UserMarker>>>,
}

impl Default for SelfPresenceStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfPresenceStore {
    pub fn new() -> Self {
        Self {
            status: signal(OwnStatus::default()),
            custom_status: signal(None),
            activities: signal(Vec::new()),
            self_id: Cell::new(None),
        }
    }
}

impl Store for SelfPresenceStore {
    fn handle(&self, action: &Action) {
        match action {
            Action::ConnectionOpen {
                user,
                user_settings_proto,
                ..
            } => {
                self.self_id.set(Some(user.id));
                let proto = user_settings_proto.as_deref();
                let stored = proto.and_then(user_settings::status_from_base64);
                self.status.set(stored.unwrap_or_default());
                let custom = proto
                    .and_then(user_settings::custom_status_from_base64)
                    .filter(|c| !c.is_expired(time::now_ms()));
                self.custom_status.set(custom);
                self.activities.set(Vec::new());
            }
            Action::PresenceUpdate(presence) => {
                if self.self_id.get() != Some(presence.user.id) {
                    return;
                }
                let activities: Vec<Activity> = presence
                    .activities
                    .iter()
                    .filter(|activity| activity.kind != 4)
                    .cloned()
                    .collect();
                self.activities.set(activities);
            }
            _ => {}
        }
    }

    fn reset(&self) {
        self.self_id.set(None);
        self.status.set(OwnStatus::default());
        self.custom_status.set(None);
        self.activities.set(Vec::new());
    }
}
