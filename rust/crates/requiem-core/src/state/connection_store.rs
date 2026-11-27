use requiem_ui::signal::{Signal, signal};

use super::{Action, Store};

#[derive(Clone)]
pub struct ConnectionStore {
    pub ready: Signal<bool>,

    pub auth_session_id_hash: Signal<Option<String>>,
}

impl Default for ConnectionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionStore {
    pub fn new() -> Self {
        Self {
            ready: signal(false),
            auth_session_id_hash: signal(None),
        }
    }
}

impl Store for ConnectionStore {
    fn handle(&self, action: &Action) {
        match action {
            Action::ConnectionOpen {
                auth_session_id_hash,
                ..
            } => {
                self.auth_session_id_hash.set(auth_session_id_hash.clone());
                self.ready.set(true);
            }

            Action::Resumed => self.ready.set(true),
            _ => {}
        }
    }

    fn reset(&self) {
        self.auth_session_id_hash.set(None);
        self.ready.set(false);
    }
}
