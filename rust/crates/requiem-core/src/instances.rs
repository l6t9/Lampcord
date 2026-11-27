use crate::utils::time;
use requiem_db::paths;
use std::fs;
use std::iter;
use std::path::PathBuf;
use std::path;

use serde::{Deserialize, Serialize};

use crate::config::BackendConfig;

const API_PATH: &str = "/api/v9";

pub const OFFICIAL_ID: &str = "discord";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceOverrides {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cdn: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub name: String,

    pub base_url: String,
    #[serde(default)]
    pub overrides: InstanceOverrides,
}

struct ParsedBase {
    secure: bool,
    authority: String,
}

fn parse_base(base_url: &str) -> Option<ParsedBase> {
    let trimmed = base_url.trim();
    let (secure, rest) = if let Some(rest) = trimmed.strip_prefix("https://") {
        (true, rest)
    } else if let Some(rest) = trimmed.strip_prefix("http://") {
        (false, rest)
    } else {
        (true, trimmed)
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("").trim();
    if authority.is_empty() || authority.contains(char::is_whitespace) {
        return None;
    }
    Some(ParsedBase {
        secure,
        authority: authority.to_string(),
    })
}

pub fn derived_urls(base_url: &str) -> Option<InstanceOverrides> {
    let base = parse_base(base_url)?;
    let (http, ws) = if base.secure {
        ("https", "wss")
    } else {
        ("http", "ws")
    };
    Some(InstanceOverrides {
        api: Some(format!("{http}://{}{API_PATH}", base.authority)),
        gateway: Some(format!("{ws}://gateway.{}", base.authority)),
        cdn: Some(format!("{http}://cdn.{}", base.authority)),
    })
}

pub fn is_valid_base_url(base_url: &str) -> bool {
    parse_base(base_url).is_some()
}

impl Instance {
    pub fn official() -> Self {
        Self {
            id: OFFICIAL_ID.to_string(),
            name: "Discord".to_string(),
            base_url: "https://discord.com".to_string(),
            overrides: InstanceOverrides {
                api: Some(format!("https://discord.com{API_PATH}")),

                gateway: Some(
                    "wss://gateway.discord.gg/?encoding=etf&v=9&compress=zstd-stream".to_string(),
                ),
                cdn: Some("https://cdn.discordapp.com".to_string()),
            },
        }
    }

    pub fn is_official(&self) -> bool {
        self.id == OFFICIAL_ID
    }

    pub fn backend(&self) -> Option<BackendConfig> {
        let derived = derived_urls(&self.base_url).unwrap_or_default();
        let pick = |over: &Option<String>, derived: Option<String>| {
            over.clone().filter(|s| !s.trim().is_empty()).or(derived)
        };
        Some(BackendConfig {
            rest_base_url: pick(&self.overrides.api, derived.api)?,
            gateway_url: pick(&self.overrides.gateway, derived.gateway)?,
            cdn_base_url: pick(&self.overrides.cdn, derived.cdn)?,
        })
    }

    pub fn token_account(&self) -> String {
        format!("instance:{}", self.id)
    }

    pub fn display_host(&self) -> String {
        parse_base(&self.base_url)
            .map(|b| b.authority)
            .unwrap_or_else(|| self.base_url.clone())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct StoredInstances {
    #[serde(default)]
    selected: Option<String>,
    #[serde(default)]
    instances: Vec<Instance>,
}

#[derive(Clone, Debug)]
pub struct InstanceStore {
    db: Option<requiem_db::Db>,
    pub custom: Vec<Instance>,
    pub selected: String,

    pub existed: bool,
}

impl InstanceStore {
    pub fn default_path() -> Option<PathBuf> {
        paths::data_root().map(|dir| dir.join("instances.json"))
    }

    pub fn load(path: Option<PathBuf>) -> Self {
        let db = path.as_deref().and_then(database_beside);
        if let (Some(db), Some(path)) = (&db, &path) {
            db.import_once("instances.json", |db| import_legacy(db, path));
        }
        let (rows, selected) = db.as_ref().map(|db| db.instances()).unwrap_or_default();
        let existed = !rows.is_empty() || selected.is_some();
        let custom: Vec<Instance> = rows
            .into_iter()
            .map(|row| Instance {
                overrides: serde_json::from_str(&row.overrides).unwrap_or_else(|err| {
                    log::warn!("ignoring unreadable overrides of {}: {err}", row.id);
                    InstanceOverrides::default()
                }),
                id: row.id,
                name: row.name,
                base_url: row.base_url,
            })
            .collect();
        let mut store = Self {
            db,
            custom,
            selected: selected.unwrap_or_else(|| OFFICIAL_ID.to_string()),
            existed,
        };
        if store.get(&store.selected).is_none() {
            store.selected = OFFICIAL_ID.to_string();
        }
        store
    }

    pub fn all(&self) -> Vec<Instance> {
        iter::once(Instance::official())
            .chain(self.custom.iter().cloned())
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<Instance> {
        if id == OFFICIAL_ID {
            return Some(Instance::official());
        }
        self.custom.iter().find(|i| i.id == id).cloned()
    }

    pub fn selected_instance(&self) -> Instance {
        self.get(&self.selected).unwrap_or_else(Instance::official)
    }

    pub fn add(
        &mut self,
        name: String,
        base_url: String,
        overrides: InstanceOverrides,
    ) -> Instance {
        let millis = time::now_ms();
        let instance = Instance {
            id: format!("custom-{millis}"),
            name,
            base_url: base_url.trim().to_string(),
            overrides,
        };
        self.custom.push(instance.clone());
        self.selected = instance.id.clone();
        self.save();
        instance
    }

    pub fn select(&mut self, id: &str) {
        if self.get(id).is_some() {
            self.selected = id.to_string();
            self.save();
        }
    }

    pub fn save(&self) {
        let Some(db) = &self.db else { return };
        let rows: Vec<requiem_db::InstanceRow> = self
            .custom
            .iter()
            .map(|i| requiem_db::InstanceRow {
                id: i.id.clone(),
                name: i.name.clone(),
                base_url: i.base_url.clone(),
                overrides: serde_json::to_string(&i.overrides).unwrap_or_else(|_| "{}".into()),
            })
            .collect();
        if let Err(err) = db.set_instances(&rows, &self.selected) {
            log::warn!("failed to save the instance list: {err}");
        }
    }
}

fn database_beside(legacy: &path::Path) -> Option<requiem_db::Db> {
    if InstanceStore::default_path().as_deref() == Some(legacy) {
        return Some(requiem_db::Db::shared().clone());
    }
    requiem_db::Db::open(&legacy.parent()?.join("app.db"))
        .map_err(|err| log::warn!("could not open the app database: {err}"))
        .ok()
}

fn import_legacy(db: &requiem_db::Db, path: &path::Path) -> Result<(), String> {
    let Ok(raw) = fs::read_to_string(path) else {
        return Ok(());
    };
    let stored: StoredInstances = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let rows: Vec<requiem_db::InstanceRow> = stored
        .instances
        .iter()
        .map(|i| requiem_db::InstanceRow {
            id: i.id.clone(),
            name: i.name.clone(),
            base_url: i.base_url.clone(),
            overrides: serde_json::to_string(&i.overrides).unwrap_or_else(|_| "{}".into()),
        })
        .collect();
    db.set_instances(&rows, stored.selected.as_deref().unwrap_or(OFFICIAL_ID))
        .map_err(|e| e.to_string())?;
    let _ = fs::rename(path, path.with_extension("json.imported"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_discord_layout_from_bare_host() {
        let urls = derived_urls("mee6isgoinggreat.com").unwrap();
        assert_eq!(
            urls.api.as_deref(),
            Some("https://mee6isgoinggreat.com/api/v9")
        );
        assert_eq!(
            urls.gateway.as_deref(),
            Some("wss://gateway.mee6isgoinggreat.com")
        );
        assert_eq!(
            urls.cdn.as_deref(),
            Some("https://cdn.mee6isgoinggreat.com")
        );
    }

    #[test]
    fn keeps_port_and_plain_http() {
        let urls = derived_urls("http://localhost:8080/app").unwrap();
        assert_eq!(urls.api.as_deref(), Some("http://localhost:8080/api/v9"));
        assert_eq!(urls.gateway.as_deref(), Some("ws://gateway.localhost:8080"));
    }

    #[test]
    fn overrides_win_over_derivation() {
        let instance = Instance {
            id: "x".into(),
            name: "x".into(),
            base_url: "example.com".into(),
            overrides: InstanceOverrides {
                gateway: Some("wss://ws.example.com".into()),
                ..Default::default()
            },
        };
        let backend = instance.backend().unwrap();
        assert_eq!(backend.gateway_url, "wss://ws.example.com");
        assert_eq!(backend.rest_base_url, "https://example.com/api/v9");
    }

    #[test]
    fn rejects_empty_base() {
        assert!(!is_valid_base_url("   "));
        assert!(!is_valid_base_url("https://"));
    }
}
