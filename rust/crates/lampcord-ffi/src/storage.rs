use std::sync::Arc;

use lampcord_core::secret_store::SecretStore;

#[cfg_attr(feature = "uniffi", uniffi::export(with_foreign))]
pub trait PlatformSecretStore: Send + Sync {
    fn load(&self) -> Option<String>;
    fn save(&self, value: String);
    fn clear(&self);
}

pub struct ForeignSecretStore {
    inner: Arc<dyn PlatformSecretStore>,
}

impl ForeignSecretStore {
    pub fn new(inner: Arc<dyn PlatformSecretStore>) -> Self {
        Self { inner }
    }
}

impl SecretStore for ForeignSecretStore {
    fn load(&self) -> Option<String> {
        self.inner.load()
    }

    fn save(&self, value: &str) {
        self.inner.save(value.to_owned());
    }

    fn clear(&self) {
        self.inner.clear();
    }
}
