use crate::rest::endpoints::user_settings;
use crate::utils::time;
use requiem_ui::signal::{Signal, signal};

use super::{Action, Store};
use crate::presence::{CustomStatus, OwnStatus};

#[derive(Clone)]
pub struct SelfPresenceStore {
    pub status: Signal<OwnStatus>,

    pub custom_status: Signal<Option<CustomStatus>>,
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
        }
    }
}

impl Store for SelfPresenceStore {
    fn handle(&self, action: &Action) {
        if let Action::ConnectionOpen {
            user_settings_proto,
            ..
        } = action
        {
            let proto = user_settings_proto.as_deref();
            let stored = proto.and_then(user_settings::status_from_base64);
            self.status.set(stored.unwrap_or_default());
            let custom = proto
                .and_then(user_settings::custom_status_from_base64)
                .filter(|c| !c.is_expired(time::now_ms()));
            self.custom_status.set(custom);
        }
    }

    fn reset(&self) {
        self.status.set(OwnStatus::default());
        self.custom_status.set(None);
    }
}
