use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

pub trait SecretStore: Send + Sync {
    fn load(&self) -> Option<String>;
    fn save(&self, value: &str);
    fn clear(&self);
}

#[derive(Debug, Clone)]
pub struct FileSecretStore {
    path: PathBuf,
}

impl FileSecretStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl SecretStore for FileSecretStore {
    fn load(&self) -> Option<String> {
        std::fs::read_to_string(&self.path).ok()
    }

    fn save(&self, value: &str) {
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&self.path, value);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&self.path, std::fs::Permissions::from_mode(0o600));
        }
    }

    fn clear(&self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[derive(Debug, Default)]
pub struct MemorySecretStore {
    value: Mutex<Option<String>>,
}

impl MemorySecretStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SecretStore for MemorySecretStore {
    fn load(&self) -> Option<String> {
        self.value.lock().expect("secret store lock").clone()
    }

    fn save(&self, value: &str) {
        *self.value.lock().expect("secret store lock") = Some(value.to_owned());
    }

    fn clear(&self) {
        *self.value.lock().expect("secret store lock") = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_round_trip_and_clear() {
        let store = MemorySecretStore::new();
        assert_eq!(store.load(), None);
        store.save("secret");
        assert_eq!(store.load().as_deref(), Some("secret"));
        store.clear();
        assert_eq!(store.load(), None);
    }

    #[test]
    fn file_store_round_trip_and_clear() {
        let path = std::env::temp_dir().join(format!(
            "lampcord-secret-store-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let store = FileSecretStore::new(&path);
        assert_eq!(store.load(), None);
        store.save("secret");
        assert_eq!(store.load().as_deref(), Some("secret"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path)
                .expect("secret file metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600, "secret files must not be world-readable");
        }
        store.clear();
        assert_eq!(store.load(), None);
    }
}
