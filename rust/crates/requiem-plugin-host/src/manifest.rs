use std::collections::BTreeMap;

use serde::Deserialize;

use crate::Error;

pub const API_VERSION: (u32, u32) = (1, 0);

pub const KNOWN_PERMISSIONS: &[(&str, &str)] = &[
    ("guilds.read", "normal"),
    ("members.read", "normal"),
    ("presence.read", "normal"),
    ("voice.read", "normal"),
    ("relationships.read", "sensitive"),
    ("read-states.read", "normal"),
    ("read-states.write", "normal"),
    ("messages.read", "sensitive"),
    ("messages.intercept-incoming", "sensitive"),
    ("messages.intercept-outgoing", "sensitive"),
    ("attachments.read", "sensitive"),
    ("messages.send", "dangerous"),
    ("messages.manage", "dangerous"),
    ("reactions", "sensitive"),
    ("composer", "sensitive"),
    ("clipboard.write", "normal"),
    ("clipboard.read", "sensitive"),
    ("presence.write", "sensitive"),
    ("voice.control", "sensitive"),
    ("voice.moderate", "dangerous"),
    ("notifications", "normal"),
    ("notifications.intercept", "normal"),
    ("audio.play", "normal"),
    ("audio.replace", "normal"),
    ("audio.process", "sensitive"),
    ("tweaks", "normal"),
    ("files", "normal"),
    ("display.users", "normal"),
    ("display.channels", "normal"),
    ("display.guilds", "normal"),
    ("ui.local-messages", "normal"),
    ("ui.modal", "normal"),
    ("ui.slots", "normal"),
    ("ui.pages", "normal"),
    ("ui.notices", "normal"),
    ("ui.overlay", "normal"),
    ("data.raw", "sensitive"),
    ("network.localhost", "dangerous"),
];

pub const KNOWN_HOOKS: &[(&str, Option<&str>)] = &[
    ("incoming", Some("messages.intercept-incoming")),
    ("outgoing", Some("messages.intercept-outgoing")),
    ("presence", Some("presence.write")),
    ("notifications", Some("notifications.intercept")),
    ("message-click", Some("messages.read")),
    ("settings-ui", None),
];

pub fn permission_tier(permission: &str) -> Option<&'static str> {
    if let Some((_, tier)) = KNOWN_PERMISSIONS
        .iter()
        .find(|(name, _)| *name == permission)
    {
        return Some(tier);
    }
    if let Some(host) = permission.strip_prefix("network:") {
        let valid = !host.is_empty()
            && host
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '*' | ':'));
        return valid.then_some(if host == "*" { "dangerous" } else { "normal" });
    }
    if let Some(route) = permission.strip_prefix("discord-api:") {
        let (method, path) = route.split_once(' ')?;
        let valid =
            matches!(method, "GET" | "POST" | "PUT" | "PATCH" | "DELETE") && path.starts_with('/');
        return valid.then_some("dangerous");
    }
    None
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Manifest {
    pub id: String,
    pub name: String,

    pub version: String,

    pub api: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub source: Option<String>,

    #[serde(default)]
    pub permissions: BTreeMap<String, String>,

    #[serde(default)]
    pub optional_permissions: BTreeMap<String, String>,
    #[serde(default)]
    pub hooks: BTreeMap<String, HookConfig>,
    #[serde(default)]
    pub limits: Limits,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct HookConfig {
    #[serde(default)]
    pub priority: i32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct Limits {
    pub memory_mib: u32,
    pub storage_kib: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            memory_mib: 64,
            storage_kib: 1024,
        }
    }
}

const MAX_MEMORY_MIB: u32 = 512;
const MAX_STORAGE_KIB: u32 = 64 * 1024;

impl Manifest {
    pub fn parse(text: &str) -> Result<Manifest, Error> {
        let manifest: Manifest =
            toml::from_str(text).map_err(|err| Error::Manifest(err.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    fn validate(&self) -> Result<(), Error> {
        let bad = |message: String| Err(Error::Manifest(message));
        let id_ok = self.id.len() <= 128
            && self.id.split('.').count() >= 2
            && self.id.split('.').all(|part| {
                !part.is_empty()
                    && part
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            });
        if !id_ok {
            return bad(format!(
                "id `{}` must be reverse-DNS: lowercase letters, digits and `-`, separated by dots",
                self.id
            ));
        }
        if self.name.trim().is_empty() {
            return bad("name is empty".to_string());
        }
        let (major, minor) = self
            .api_version()
            .ok_or_else(|| Error::Manifest(format!("api `{}` must look like `1.0`", self.api)))?;
        if major != API_VERSION.0 || minor > API_VERSION.1 {
            return Err(Error::NeedsNewerClient(format!("plugin API {}", self.api)));
        }
        for permission in self
            .permissions
            .keys()
            .chain(self.optional_permissions.keys())
        {
            if permission_tier(permission).is_none() {
                return Err(Error::NeedsNewerClient(format!(
                    "permission `{permission}`"
                )));
            }
        }
        if let Some(both) = self
            .permissions
            .keys()
            .find(|p| self.optional_permissions.contains_key(*p))
        {
            return bad(format!("permission `{both}` is both required and optional"));
        }
        for hook in self.hooks.keys() {
            let Some((_, needs)) = KNOWN_HOOKS.iter().find(|(name, _)| name == hook) else {
                return Err(Error::NeedsNewerClient(format!("hook `{hook}`")));
            };
            if let Some(permission) = needs {
                if !self.asks_for(permission) {
                    return bad(format!("hook `{hook}` needs the `{permission}` permission"));
                }
            }
        }
        if self.limits.memory_mib == 0 || self.limits.memory_mib > MAX_MEMORY_MIB {
            return bad(format!("limits.memory-mib must be 1..={MAX_MEMORY_MIB}"));
        }
        if self.limits.storage_kib > MAX_STORAGE_KIB {
            return bad(format!(
                "limits.storage-kib must be at most {MAX_STORAGE_KIB}"
            ));
        }
        Ok(())
    }

    pub fn api_version(&self) -> Option<(u32, u32)> {
        let (major, minor) = self.api.split_once('.')?;
        Some((major.parse().ok()?, minor.parse().ok()?))
    }

    pub fn asks_for(&self, permission: &str) -> bool {
        self.permissions.contains_key(permission)
            || self.optional_permissions.contains_key(permission)
    }

    pub fn is_required(&self, permission: &str) -> bool {
        self.permissions.contains_key(permission)
    }

    pub fn has_hook(&self, hook: &str) -> bool {
        self.hooks.contains_key(hook)
    }

    pub fn priority(&self, hook: &str) -> i32 {
        self.hooks.get(hook).map(|h| h.priority).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"
        id = "dev.example.thing"
        name = "Thing"
        version = "0.1.0"
        api = "1.0"
        [permissions]
        "messages.intercept-incoming" = "because"
        [hooks]
        incoming = { priority = 3 }
    "#;

    #[test]
    fn parses_a_valid_manifest() {
        let manifest = Manifest::parse(GOOD).unwrap();
        assert_eq!(manifest.priority("incoming"), 3);
        assert!(manifest.has_hook("incoming"));
        assert_eq!(manifest.limits.memory_mib, 64);
    }

    #[test]
    fn rejects_a_hook_without_its_permission() {
        let text = GOOD.replace(r#""messages.intercept-incoming" = "because""#, "");
        assert!(matches!(Manifest::parse(&text), Err(Error::Manifest(_))));
    }

    #[test]
    fn newer_api_and_unknown_permissions_need_a_newer_client() {
        assert!(matches!(
            Manifest::parse(&GOOD.replace("api = \"1.0\"", "api = \"1.7\"")),
            Err(Error::NeedsNewerClient(_))
        ));
        let text = GOOD.replace("[hooks]", "\"actions.teleport\" = \"x\"\n[hooks]");
        assert!(matches!(
            Manifest::parse(&text),
            Err(Error::NeedsNewerClient(_))
        ));
    }

    #[test]
    fn optional_permissions() {
        let text = GOOD.replace(
            "[hooks]",
            "[optional-permissions]
\"clipboard.write\" = \"copy\"
[hooks]",
        );
        let manifest = Manifest::parse(&text).unwrap();
        assert!(manifest.asks_for("clipboard.write") && !manifest.is_required("clipboard.write"));
        assert!(manifest.is_required("messages.intercept-incoming"));
        let both = text.replace(
            "\"clipboard.write\" = \"copy\"",
            "\"messages.intercept-incoming\" = \"x\"",
        );
        assert!(matches!(Manifest::parse(&both), Err(Error::Manifest(_))));
    }

    #[test]
    fn rejects_bad_ids() {
        for id in ["thing", "Dev.example", "dev..x", "dev.ex ample"] {
            assert!(
                Manifest::parse(&GOOD.replace("dev.example.thing", id)).is_err(),
                "{id}"
            );
        }
    }

    #[test]
    fn pattern_permissions() {
        assert_eq!(permission_tier("network:api.deepl.com"), Some("normal"));
        assert_eq!(permission_tier("network:*"), Some("dangerous"));
        assert_eq!(permission_tier("network:bad host"), None);
        assert_eq!(
            permission_tier("discord-api:PATCH /users/@me/guilds/*/settings"),
            Some("dangerous")
        );
        assert_eq!(permission_tier("discord-api:FLY /x"), None);
        assert_eq!(permission_tier("messages.send"), Some("dangerous"));
    }
}
