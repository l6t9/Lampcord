pub mod paths;

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::result;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Duration;

pub use rusqlite::{Connection, Error, Result, params};
use serde_json::{Map, Value};

const MIGRATIONS: &[&str] = &[
    "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL) WITHOUT ROWID;
     CREATE TABLE prefs (key TEXT PRIMARY KEY, value TEXT NOT NULL) WITHOUT ROWID;
     CREATE TABLE instances (
         id TEXT PRIMARY KEY,
         position INTEGER NOT NULL,
         name TEXT NOT NULL,
         base_url TEXT NOT NULL,
         overrides TEXT NOT NULL
     ) WITHOUT ROWID;
     CREATE TABLE alt_svc (origin TEXT PRIMARY KEY, expires INTEGER NOT NULL) WITHOUT ROWID;
     CREATE TABLE intermediates (der BLOB PRIMARY KEY) WITHOUT ROWID;
     CREATE TABLE plugin_kv (
         plugin TEXT NOT NULL,
         key TEXT NOT NULL,
         value BLOB NOT NULL,
         PRIMARY KEY (plugin, key)
     ) WITHOUT ROWID;
     CREATE TABLE documents (name TEXT PRIMARY KEY, body TEXT NOT NULL) WITHOUT ROWID;",
];

#[derive(Clone)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

impl fmt::Debug for Db {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Db")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceRow {
    pub id: String,
    pub name: String,
    pub base_url: String,

    pub overrides: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum KvError {
    InvalidKey,

    Quota(usize),
    Db(String),
}

pub const MAX_KEY_LEN: usize = 512;

impl Db {
    pub fn config_dir() -> Option<PathBuf> {
        paths::data_root()
    }

    pub fn shared() -> &'static Db {
        static SHARED: OnceLock<Db> = OnceLock::new();
        SHARED.get_or_init(|| {
            Self::config_dir()
                .map(|dir| dir.join("app.db"))
                .and_then(|path| match Db::open(&path) {
                    Ok(db) => Some(db),
                    Err(err) => {
                        log::warn!(
                            "could not open {}: {err}; settings will not be kept",
                            path.display()
                        );
                        None
                    }
                })
                .unwrap_or_else(|| Db::open_in_memory().expect("an in-memory database opens"))
        })
    }

    pub fn open(path: &Path) -> Result<Db> {
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        Self::configure(Connection::open(path)?, true)
    }

    pub fn open_in_memory() -> Result<Db> {
        Self::configure(Connection::open_in_memory()?, false)
    }

    fn configure(mut conn: Connection, file: bool) -> Result<Db> {
        conn.busy_timeout(Duration::from_secs(5))?;
        if file {
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "NORMAL")?;
        }
        migrate(&mut conn)?;
        Ok(Db {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn with<R>(&self, f: impl FnOnce(&mut Connection) -> Result<R>) -> Result<R> {
        f(&mut self.lock())
    }

    fn lock(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn meta(&self, key: &str) -> Option<String> {
        self.lock()
            .query_row("SELECT value FROM meta WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .ok()
    }

    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.lock().execute("INSERT INTO meta (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value", [key, value]).map(drop)
    }

    pub fn import_once(&self, name: &str, import: impl FnOnce(&Db) -> result::Result<(), String>) {
        let key = format!("imported:{name}");
        if self.meta(&key).is_some() {
            return;
        }
        match import(self) {
            Ok(()) => {
                let _ = self.set_meta(&key, "1");
            }
            Err(err) => log::warn!("importing {name}: {err}"),
        }
    }

    pub fn prefs(&self) -> Map<String, Value> {
        let conn = self.lock();
        let Ok(mut statement) = conn.prepare("SELECT key, value FROM prefs") else {
            return Map::new();
        };
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        });
        let mut map = Map::new();
        for (key, value) in rows.into_iter().flatten().flatten() {
            if let Ok(value) = serde_json::from_str(&value) {
                map.insert(key, value);
            }
        }
        map
    }

    pub fn set_prefs(&self, map: &Map<String, Value>) -> Result<()> {
        let mut conn = self.lock();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM prefs", [])?;
        for (key, value) in map.iter().filter(|(_, v)| !v.is_null()) {
            tx.execute(
                "INSERT INTO prefs (key, value) VALUES (?1, ?2)",
                params![key, value.to_string()],
            )?;
        }
        tx.commit()
    }

    pub fn instances(&self) -> (Vec<InstanceRow>, Option<String>) {
        let selected = self.meta("selected_instance");
        let conn = self.lock();
        let Ok(mut statement) =
            conn.prepare("SELECT id, name, base_url, overrides FROM instances ORDER BY position")
        else {
            return (Vec::new(), selected);
        };
        let rows = statement
            .query_map([], |row| {
                Ok(InstanceRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    base_url: row.get(2)?,
                    overrides: row.get(3)?,
                })
            })
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default();
        (rows, selected)
    }

    pub fn set_instances(&self, rows: &[InstanceRow], selected: &str) -> Result<()> {
        let mut conn = self.lock();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM instances", [])?;
        for (position, row) in rows.iter().enumerate() {
            tx.execute("INSERT INTO instances (id, position, name, base_url, overrides) VALUES (?1, ?2, ?3, ?4, ?5)", params![row.id, position as i64, row.name, row.base_url, row.overrides])?;
        }
        tx.execute("INSERT INTO meta (key, value) VALUES ('selected_instance', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value", [selected])?;
        tx.commit()
    }

    pub fn alt_svc(&self) -> Vec<(String, u64)> {
        let conn = self.lock();
        let Ok(mut statement) = conn.prepare("SELECT origin, expires FROM alt_svc") else {
            return Vec::new();
        };
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get::<_, i64>(1)? as u64)))
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default()
    }

    pub fn set_alt_svc(&self, entries: &[(String, u64)]) -> Result<()> {
        let mut conn = self.lock();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM alt_svc", [])?;
        for (origin, expires) in entries {
            tx.execute(
                "INSERT INTO alt_svc (origin, expires) VALUES (?1, ?2)",
                params![origin, *expires as i64],
            )?;
        }
        tx.commit()
    }

    pub fn intermediates(&self) -> Vec<Vec<u8>> {
        let conn = self.lock();
        let Ok(mut statement) = conn.prepare("SELECT der FROM intermediates") else {
            return Vec::new();
        };
        statement
            .query_map([], |row| row.get(0))
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default()
    }

    pub fn add_intermediates(&self, certs: &[Vec<u8>]) -> Result<()> {
        let mut conn = self.lock();
        let tx = conn.transaction()?;
        for der in certs {
            tx.execute(
                "INSERT OR IGNORE INTO intermediates (der) VALUES (?1)",
                [der],
            )?;
        }
        tx.commit()
    }

    pub fn document(&self, name: &str) -> Option<String> {
        self.lock()
            .query_row(
                "SELECT body FROM documents WHERE name = ?1",
                [name],
                |row| row.get(0),
            )
            .ok()
    }

    pub fn set_document(&self, name: &str, body: &str) -> Result<()> {
        self.lock().execute("INSERT INTO documents (name, body) VALUES (?1, ?2) ON CONFLICT(name) DO UPDATE SET body = excluded.body", [name, body]).map(drop)
    }

    pub fn kv_get(&self, plugin: &str, key: &str) -> Option<Vec<u8>> {
        self.lock()
            .query_row(
                "SELECT value FROM plugin_kv WHERE plugin = ?1 AND key = ?2",
                [plugin, key],
                |row| row.get(0),
            )
            .ok()
    }

    pub fn kv_keys(&self, plugin: &str, prefix: &str) -> Vec<String> {
        let conn = self.lock();
        let Ok(mut statement) = conn.prepare("SELECT key FROM plugin_kv WHERE plugin = ?1 AND substr(key, 1, length(?2)) = ?2 ORDER BY key") else { return Vec::new() };
        statement
            .query_map([plugin, prefix], |row| row.get(0))
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default()
    }

    pub fn kv_used(&self, plugin: &str) -> usize {
        self.lock().query_row("SELECT COALESCE(SUM(length(CAST(key AS BLOB)) + length(value)), 0) FROM plugin_kv WHERE plugin = ?1", [plugin], |row| row.get::<_, i64>(0)).unwrap_or(0) as usize
    }

    pub fn kv_set(
        &self,
        plugin: &str,
        key: &str,
        value: &[u8],
        quota: usize,
    ) -> result::Result<(), KvError> {
        if key.is_empty() || key.len() > MAX_KEY_LEN {
            return Err(KvError::InvalidKey);
        }
        let mut conn = self.lock();
        let db = |e: Error| KvError::Db(e.to_string());
        let tx = conn.transaction().map_err(db)?;
        let used: i64 = tx.query_row("SELECT COALESCE(SUM(length(CAST(key AS BLOB)) + length(value)), 0) FROM plugin_kv WHERE plugin = ?1", [plugin], |row| row.get(0)).map_err(db)?;
        let old: i64 = tx.query_row("SELECT COALESCE(length(CAST(key AS BLOB)) + length(value), 0) FROM plugin_kv WHERE plugin = ?1 AND key = ?2", [plugin, key], |row| row.get(0)).unwrap_or(0);
        let would_use = used as usize - old as usize + key.len() + value.len();
        if would_use > quota {
            return Err(KvError::Quota(would_use));
        }
        tx.execute("INSERT INTO plugin_kv (plugin, key, value) VALUES (?1, ?2, ?3) ON CONFLICT(plugin, key) DO UPDATE SET value = excluded.value", params![plugin, key, value]).map_err(db)?;
        tx.commit().map_err(db)
    }

    pub fn kv_delete(&self, plugin: &str, key: &str) -> Result<()> {
        self.lock()
            .execute(
                "DELETE FROM plugin_kv WHERE plugin = ?1 AND key = ?2",
                [plugin, key],
            )
            .map(drop)
    }

    pub fn kv_clear(&self, plugin: &str) -> Result<()> {
        self.lock()
            .execute("DELETE FROM plugin_kv WHERE plugin = ?1", [plugin])
            .map(drop)
    }

    pub fn kv_has_any(&self, plugin: &str) -> bool {
        self.lock()
            .query_row(
                "SELECT 1 FROM plugin_kv WHERE plugin = ?1 LIMIT 1",
                [plugin],
                |_| Ok(()),
            )
            .is_ok()
    }
}

fn migrate(conn: &mut Connection) -> Result<()> {
    let current: usize = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(current) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", index + 1)?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, process};

    #[test]
    fn migrations_apply_once_and_a_reopened_file_keeps_its_data() {
        let dir = env::temp_dir().join(format!("requiem-db-test-{}", process::id()));
        let path = dir.join("app.db");
        let db = Db::open(&path).unwrap();
        db.set_meta("a", "1").unwrap();
        drop(db);
        let db = Db::open(&path).unwrap();
        assert_eq!(db.meta("a").as_deref(), Some("1"));
        assert_eq!(
            db.with(|c| c.pragma_query_value(None, "user_version", |r| r.get::<_, usize>(0)))
                .unwrap(),
            MIGRATIONS.len()
        );
        drop(db);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn prefs_round_trip_as_rows_and_nulls_disappear() {
        let db = Db::open_in_memory().unwrap();
        let mut map = Map::new();
        map.insert("language".into(), Value::String("de".into()));
        map.insert("developer_mode".into(), Value::Bool(true));
        map.insert("time_format".into(), Value::Null);
        db.set_prefs(&map).unwrap();
        let loaded = db.prefs();
        assert_eq!(loaded.get("language"), Some(&Value::String("de".into())));
        assert_eq!(loaded.get("developer_mode"), Some(&Value::Bool(true)));
        assert!(!loaded.contains_key("time_format"));
    }

    #[test]
    fn instances_keep_their_order_and_the_selection() {
        let db = Db::open_in_memory().unwrap();
        let rows = vec![
            InstanceRow {
                id: "custom-2".into(),
                name: "B".into(),
                base_url: "b.example".into(),
                overrides: "{}".into(),
            },
            InstanceRow {
                id: "custom-1".into(),
                name: "A".into(),
                base_url: "a.example".into(),
                overrides: "{\"cdn\":\"x\"}".into(),
            },
        ];
        db.set_instances(&rows, "custom-1").unwrap();
        let (loaded, selected) = db.instances();
        assert_eq!(loaded, rows);
        assert_eq!(selected.as_deref(), Some("custom-1"));
    }

    #[test]
    fn plugin_storage_enforces_its_quota_and_counts_replacements_once() {
        let db = Db::open_in_memory().unwrap();
        db.kv_set("p", "a", b"hello", 1024).unwrap();
        db.kv_set("p", "b/c", &[0; 10], 1024).unwrap();
        assert_eq!(
            db.kv_set("p", "big", &[0; 2000], 1024),
            Err(KvError::Quota(19 + 3 + 2000))
        );
        assert_eq!(db.kv_used("p"), 1 + 5 + 3 + 10);
        db.kv_set("p", "a", b"hello world", 1024).unwrap();
        assert_eq!(db.kv_used("p"), 1 + 11 + 3 + 10);
        assert_eq!(db.kv_keys("p", "b/"), vec!["b/c".to_string()]);
        assert_eq!(db.kv_set("p", "", b"x", 1024), Err(KvError::InvalidKey));
        assert_eq!(db.kv_get("other", "a"), None);
        db.kv_delete("p", "a").unwrap();
        assert_eq!(db.kv_get("p", "a"), None);
    }

    #[test]
    fn import_once_runs_one_time() {
        let db = Db::open_in_memory().unwrap();
        let mut runs = 0;
        db.import_once("x", |_| {
            runs += 1;
            Ok(())
        });
        db.import_once("x", |_| {
            runs += 1;
            Ok(())
        });
        assert_eq!(runs, 1);

        db.import_once("y", |_| Err("nope".into()));
        assert!(db.meta("imported:y").is_none());
    }
}
