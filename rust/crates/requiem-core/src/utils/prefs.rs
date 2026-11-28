use crate::gateway::qos;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::SystemTime;
use std::time;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Prefs {
    pub language: Option<String>,

    pub time_format: Option<String>,

    pub requiem_emoji: Option<String>,

    pub emoji_skin_tone: Option<u8>,

    pub developer_mode: Option<bool>,

    /// Hosts whose masked links open without the leaving-the-app warning.
    pub trusted_link_domains: Option<Vec<String>>,

    pub theme: Option<String>,

    /// Use Material-inspired tinted colors instead of Discord's flat greys.
    pub dynamic_color: Option<bool>,

    /// Follow the desktop light/dark preference while the theme is `auto`.
    pub sync_theme: Option<bool>,

    /// Disable interface motion effects.
    pub reduce_motion: Option<bool>,

    /// `cozy` (default) or `compact` message spacing.
    pub message_display: Option<String>,

    pub client_heartbeat_session: Option<qos::ClientSession>,

    pub installation_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_input_device: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_output_device: Option<String>,

    pub voice_settings: Option<super::voice_settings::VoiceSettings>,

    pub notification_settings: Option<super::notification_settings::NotificationSettings>,
}

pub fn config_dir() -> Option<PathBuf> {
    requiem_db::Db::config_dir()
}

fn legacy_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("prefs.json"))
}

fn db() -> &'static requiem_db::Db {
    let db = requiem_db::Db::shared();
    db.import_once("prefs.json", |db| {
        let Some(path) = legacy_path() else {
            return Ok(());
        };
        let Ok(raw) = fs::read_to_string(&path) else {
            return Ok(());
        };
        let serde_json::Value::Object(map) =
            serde_json::from_str(&raw).map_err(|e| e.to_string())?
        else {
            return Err("not an object".into());
        };
        db.set_prefs(&map).map_err(|e| e.to_string())?;
        let _ = fs::rename(&path, path.with_extension("json.imported"));
        Ok(())
    });
    db
}

static DEVELOPER_MODE: OnceLock<AtomicBool> =
    OnceLock::new();

fn developer_mode_cell() -> &'static AtomicBool {
    DEVELOPER_MODE
        .get_or_init(|| AtomicBool::new(load().developer_mode.unwrap_or(false)))
}

pub fn developer_mode() -> bool {
    developer_mode_cell().load(Ordering::Relaxed)
}

pub fn set_developer_mode(on: bool) {
    developer_mode_cell().store(on, Ordering::Relaxed);
    let saved = Prefs {
        developer_mode: Some(on),
        ..load()
    };
    save(&saved);
}

pub fn theme() -> String {
    load().theme.unwrap_or_else(|| "dark".to_string())
}

pub fn set_theme(theme: &str) {
    let saved = Prefs {
        theme: Some(theme.to_string()),
        ..load()
    };
    save(&saved);
}

/// Select a theme swatch. `auto` is a real pref value; manual choices turn
/// system following off so the raw pref stays compatible with older installs.
pub fn select_theme(theme: &str) {
    match theme {
        "auto" => {
            set_sync_theme(true);
            set_theme("auto");
        }
        manual => {
            set_sync_theme(false);
            set_theme(manual);
        }
    }
}

pub fn sync_theme() -> bool {
    load().sync_theme.unwrap_or(false)
}

pub fn set_sync_theme(on: bool) {
    let saved = Prefs {
        sync_theme: Some(on),
        ..load()
    };
    save(&saved);
}

pub fn dynamic_color() -> bool {
    load().dynamic_color.unwrap_or(false)
}

pub fn set_dynamic_color(on: bool) {
    let saved = Prefs {
        dynamic_color: Some(on),
        ..load()
    };
    save(&saved);
}

pub fn reduce_motion() -> bool {
    load().reduce_motion.unwrap_or(false)
}

pub fn set_reduce_motion(on: bool) {
    requiem_ui::scroll_animation::set_smooth_scrolling(!on);
    let saved = Prefs {
        reduce_motion: Some(on),
        ..load()
    };
    save(&saved);
}

pub fn message_display() -> String {
    load()
        .message_display
        .unwrap_or_else(|| "cozy".to_string())
}

pub fn set_message_display(value: &str) {
    let saved = Prefs {
        message_display: Some(value.to_string()),
        ..load()
    };
    save(&saved);
}

fn gsettings_value(schema: &str, key: &str) -> Option<String> {
    let output = Command::new("gsettings")
        .args(["get", schema, key])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .trim_matches('\'')
            .trim_matches('"')
            .to_ascii_lowercase(),
    )
}

/// Follow the desktop light/dark preference on Linux. `color-scheme` is the
/// modern key; the GTK theme name is the fallback for older desktops.
pub fn system_prefers_dark() -> bool {
    if let Some(scheme) = gsettings_value("org.gnome.desktop.interface", "color-scheme") {
        if scheme.contains("dark") {
            return true;
        }
        if scheme.contains("light") {
            return false;
        }
    }
    gsettings_value("org.gnome.desktop.interface", "gtk-theme")
        .is_some_and(|theme| theme.contains("dark"))
}

pub fn resolved_theme() -> &'static str {
    let raw = theme();
    if raw == "auto" {
        return if sync_theme() && system_prefers_dark() {
            "dark"
        } else if sync_theme() {
            "light"
        } else {
            "dark"
        };
    }
    match raw.as_str() {
        "light" => "light",
        "ash" => "ash",
        "onyx" => "onyx",
        _ => "dark",
    }
}

/// The CSS class suffix used by `requiem_ui::set_theme_name`. Density is part
/// of the theme name so a density change re-resolves the message row tokens.
pub fn theme_class() -> &'static str {
    let compact = message_display() == "compact";
    let dynamic = dynamic_color();
    match (dynamic, resolved_theme(), compact) {
        (false, "light", false) => "light",
        (false, "light", true) => "light-compact",
        (false, "ash", false) => "ash",
        (false, "ash", true) => "ash-compact",
        (false, "dark", false) => "dark",
        (false, "dark", true) => "dark-compact",
        (false, "onyx", false) => "onyx",
        (false, "onyx", true) => "onyx-compact",
        (true, "light", false) => "dynamic-light",
        (true, "light", true) => "dynamic-light-compact",
        (true, "ash", false) => "dynamic-ash",
        (true, "ash", true) => "dynamic-ash-compact",
        (true, "dark", false) => "dynamic-dark",
        (true, "dark", true) => "dynamic-dark-compact",
        (true, "onyx", false) => "dynamic-onyx",
        (true, "onyx", true) => "dynamic-onyx-compact",
        _ => "dark",
    }
}

pub fn apply_theme() {
    requiem_ui::set_theme_name(theme_class());
    requiem_ui::scroll_animation::set_smooth_scrolling(!reduce_motion());
}

pub fn log_dir() -> Option<PathBuf> {
    let dir = config_dir()?.join("logs");
    fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

pub fn export_logs() -> Option<PathBuf> {
    let dir = log_dir()?;
    let stamp = SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let dest = dir.join(format!("requiem-logs-{stamp}.txt"));
    let mut out = String::new();
    for path in [
        dir.join("requiem.log"),
        config_dir()?.join("voice-logs").join("voice.log"),
    ] {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        out.push_str(&format!("===== {} =====\n{text}\n", path.display()));
    }
    if out.is_empty() {
        out.push_str("no logs yet\n");
    }
    fs::write(&dest, out).ok()?;
    Some(dest)
}
pub fn load() -> Prefs {
    serde_json::from_value(serde_json::Value::Object(db().prefs())).unwrap_or_else(|err| {
        log::warn!("ignoring unreadable preferences: {err}");
        Prefs::default()
    })
}

pub fn save(prefs: &Prefs) {
    match serde_json::to_value(prefs) {
        Ok(serde_json::Value::Object(map)) => {
            if let Err(err) = db().set_prefs(&map) {
                log::warn!("could not save preferences: {err}");
            }
        }
        Ok(_) => {}
        Err(err) => log::warn!("could not encode prefs: {err}"),
    }
}
