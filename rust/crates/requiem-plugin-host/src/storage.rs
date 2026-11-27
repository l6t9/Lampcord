use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use requiem_db::{Db, KvError, MAX_KEY_LEN};

pub(crate) struct KvStore {
    db: Db,
    plugin: String,
    quota: usize,
}

impl KvStore {
    pub fn open(db: Db, plugin: &str, legacy: PathBuf, quota_kib: u32) -> KvStore {
        import_legacy(&db, plugin, &legacy);
        KvStore {
            db,
            plugin: plugin.to_string(),
            quota: quota_kib as usize * 1024,
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.db.kv_get(&self.plugin, key)
    }

    pub fn keys(&self, prefix: &str) -> Vec<String> {
        self.db.kv_keys(&self.plugin, prefix)
    }

    pub fn set(&mut self, key: String, value: Vec<u8>) -> Result<(), (&'static str, String)> {
        self.db
            .kv_set(&self.plugin, &key, &value, self.quota)
            .map_err(|err| match err {
                KvError::InvalidKey => ("invalid", format!("keys must be 1..={MAX_KEY_LEN} bytes")),
                KvError::Quota(used) => (
                    "quota-exceeded",
                    format!("storage would use {used} of {} bytes", self.quota),
                ),
                KvError::Db(err) => ("unavailable", err),
            })
    }

    pub fn delete(&mut self, key: &str) {
        if let Err(err) = self.db.kv_delete(&self.plugin, key) {
            log::warn!("plugin storage of {}: {err}", self.plugin);
        }
    }

    pub fn usage(&self) -> (u64, u64) {
        (self.db.kv_used(&self.plugin) as u64, self.quota as u64)
    }
}

fn import_legacy(db: &Db, plugin: &str, path: &Path) {
    db.import_once(&format!("plugin-storage:{plugin}"), |db| {
        let Ok(bytes) = fs::read(path) else {
            return Ok(());
        };
        let Some(entries) = decode(&bytes) else {
            return Err(format!("{} is unreadable", path.display()));
        };
        for (key, value) in entries {
            db.kv_set(plugin, &key, &value, usize::MAX)
                .map_err(|e| format!("{e:?}"))?;
        }
        let _ = fs::rename(path, path.with_extension("bin.imported"));
        Ok(())
    });
}

fn decode(mut bytes: &[u8]) -> Option<BTreeMap<String, Vec<u8>>> {
    fn chunk<'a>(bytes: &mut &'a [u8]) -> Option<&'a [u8]> {
        let len = u32::from_le_bytes(bytes.get(..4)?.try_into().ok()?) as usize;
        let data = bytes.get(4..4 + len)?;
        *bytes = &bytes[4 + len..];
        Some(data)
    }
    let mut entries = BTreeMap::new();
    while !bytes.is_empty() {
        let key = String::from_utf8(chunk(&mut bytes)?.to_vec()).ok()?;
        let value = chunk(&mut bytes)?.to_vec();
        entries.insert(key, value);
    }
    Some(entries)
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::process;
    use super::*;

    #[test]
    fn round_trips_and_enforces_the_quota() {
        let db = Db::open_in_memory().unwrap();
        let legacy = env::temp_dir()
            .join("plugin-kv-test-none")
            .join("storage.bin");
        let mut kv = KvStore::open(db.clone(), "p", legacy.clone(), 1);
        kv.set("a".into(), b"hello".to_vec()).unwrap();
        kv.set("b/c".into(), vec![0; 10]).unwrap();
        assert_eq!(
            kv.set("big".into(), vec![0; 2000]).unwrap_err().0,
            "quota-exceeded"
        );
        let reopened = KvStore::open(db, "p", legacy, 1);
        assert_eq!(reopened.get("a").as_deref(), Some(&b"hello"[..]));
        assert_eq!(reopened.keys("b/"), vec!["b/c".to_string()]);
    }

    #[test]
    fn an_old_storage_file_is_imported_once() {
        let dir = env::temp_dir().join(format!("plugin-kv-import-{}", process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("storage.bin");
        let mut file = Vec::new();
        for (key, value) in [("k", &b"v"[..]), ("n", &b"12"[..])] {
            file.extend_from_slice(&(key.len() as u32).to_le_bytes());
            file.extend_from_slice(key.as_bytes());
            file.extend_from_slice(&(value.len() as u32).to_le_bytes());
            file.extend_from_slice(value);
        }
        fs::write(&path, file).unwrap();
        let db = Db::open_in_memory().unwrap();
        let kv = KvStore::open(db.clone(), "p", path.clone(), 4);
        assert_eq!(kv.get("k").as_deref(), Some(&b"v"[..]));
        assert_eq!(kv.get("n").as_deref(), Some(&b"12"[..]));
        assert!(!path.exists(), "the old file was set aside");
        let _ = fs::remove_dir_all(dir);
    }
}
