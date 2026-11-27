use crate::net_profile;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use tokio::sync::Mutex as AsyncMutex;

use crate::auth::TokenStore;
use crate::config::BackendConfig;

const TOKEN_STORE_ACCOUNT: &str = "default";

#[derive(Clone)]
pub struct AuthCoordinator {
    token: Arc<RwLock<String>>,

    reauth_lock: Arc<AsyncMutex<()>>,

    token_account: String,

    logged_out: Arc<AtomicBool>,
}

impl AuthCoordinator {
    pub fn new(config: BackendConfig, initial_token: String) -> Self {
        net_profile::set_origin(&config.web_base_url());
        Self {
            token: Arc::new(RwLock::new(initial_token)),
            reauth_lock: Arc::new(AsyncMutex::new(())),
            token_account: TOKEN_STORE_ACCOUNT.to_string(),
            logged_out: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn mark_logged_out(&self) {
        self.logged_out.store(true, Ordering::SeqCst);
    }

    pub fn with_token_account(mut self, account: impl Into<String>) -> Self {
        self.token_account = account.into();
        self
    }

    pub fn current_token(&self) -> String {
        self.token.read().expect("token lock poisoned").clone()
    }

    pub async fn recover_from(&self, stale_token: &str) -> Result<String, String> {
        let _guard = self.reauth_lock.lock().await;
        if self.logged_out.load(Ordering::SeqCst) {
            return Err("logged out".to_string());
        }
        let current = self.current_token();
        if current != stale_token {
            return Ok(current);
        }
        let token_store = TokenStore::new(self.token_account.clone());
        if let Err(err) = token_store.clear() {
            log::error!("failed to clear the now-invalid stored token: {err}");
        }
        Err("stored token is no longer valid".to_string())
    }
}
