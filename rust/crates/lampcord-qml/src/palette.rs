//! System-derived Material colours, ported from the Compose desktop
//! `PlatformTheme`: the seed is the dominant colour of the current
//! wallpaper (Celebi quantiser + score, like material-color-utilities), and
//! the scheme is tonal spot on the 2025 spec, as materialkolor builds it.
//! Falls back to Lampcord's own seed when no wallpaper can be found.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(QString, scheme_json, READ, NOTIFY)]
        type Palette = super::PaletteRust;

        /// Rebuilds the scheme in the background when the wallpaper
        /// changed. Cheap enough to call on a timer.
        #[qinvokable]
        fn refresh(self: Pin<&mut Palette>);
    }

    impl cxx_qt::Threading for Palette {}
}

use std::fs;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Command;
use std::time::SystemTime;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use material_colors::color::Rgb;
use material_colors::dynamic_color::{DynamicScheme, Platform, Role, SpecVersion, Variant};
use material_colors::quantize::{Quantizer, QuantizerCelebi};
use material_colors::score::Score;
use serde_json::{Map, Value};

/// `ColorLightPrimary`, the Compose fallback seed.
const DEFAULT_SEED: Rgb = Rgb::new(0x67, 0x50, 0xA4);
const GOOGLE_BLUE: Rgb = Rgb::new(0x1B, 0x6E, 0xF3);
const MAX_EXTRACTION_SIZE: u32 = 128;

const ROLES: &[Role] = &[
    Role::Primary,
    Role::OnPrimary,
    Role::PrimaryContainer,
    Role::OnPrimaryContainer,
    Role::Secondary,
    Role::OnSecondary,
    Role::SecondaryContainer,
    Role::OnSecondaryContainer,
    Role::Tertiary,
    Role::OnTertiary,
    Role::TertiaryContainer,
    Role::OnTertiaryContainer,
    Role::Error,
    Role::OnError,
    Role::ErrorContainer,
    Role::OnErrorContainer,
    Role::Background,
    Role::Surface,
    Role::SurfaceContainerLowest,
    Role::SurfaceContainerLow,
    Role::SurfaceContainer,
    Role::SurfaceContainerHigh,
    Role::SurfaceContainerHighest,
    Role::SurfaceVariant,
    Role::OnSurface,
    Role::OnSurfaceVariant,
    Role::Outline,
    Role::OutlineVariant,
    Role::InverseSurface,
    Role::InverseOnSurface,
    Role::InversePrimary,
];

pub struct PaletteRust {
    scheme_json: QString,
    /// Wallpaper path + mtime the current scheme was built from.
    source: Option<(PathBuf, Option<SystemTime>)>,
    busy: bool,
}

impl Default for PaletteRust {
    fn default() -> Self {
        // Something sensible for the very first frame; the wallpaper scheme
        // replaces it as soon as the extraction thread finishes.
        Self {
            scheme_json: QString::from(scheme_json(DEFAULT_SEED).as_str()),
            source: None,
            busy: false,
        }
    }
}

impl qobject::Palette {
    pub fn refresh(mut self: Pin<&mut Self>) {
        if self.busy {
            return;
        }

        let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
        let wallpaper = current_wallpaper(&home);
        let source = wallpaper
            .as_ref()
            .map(|path| (path.clone(), fs::metadata(path).and_then(|meta| meta.modified()).ok()));
        if source == self.source && self.source.is_some() {
            return;
        }

        let mut rust = self.as_mut().rust_mut();
        rust.source = source;
        rust.busy = true;
        let qt_thread = self.qt_thread();
        std::thread::spawn(move || {
            let seed = wallpaper
                .as_deref()
                .and_then(wallpaper_seed)
                .unwrap_or(DEFAULT_SEED);
            let json = scheme_json(seed);
            let _ = qt_thread.queue(move |mut palette| {
                palette.as_mut().rust_mut().busy = false;
                if json != palette.scheme_json.to_string() {
                    palette.as_mut().rust_mut().scheme_json = QString::from(json.as_str());
                    palette.as_mut().scheme_json_changed();
                }
            });
        });
    }
}

fn hex(rgb: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", rgb.red, rgb.green, rgb.blue)
}

/// `{"dark": {...}, "light": {...}}` with camelCase M3 role names.
fn scheme_json(seed: Rgb) -> String {
    let mode = |is_dark: bool| {
        let scheme = DynamicScheme::from_spec(
            seed.into(),
            Variant::TonalSpot,
            is_dark,
            None,
            Platform::Phone,
            SpecVersion::Spec2025,
        );
        let roles: Map<String, Value> = ROLES
            .iter()
            .map(|role| (camel(role.name()), Value::from(hex(scheme.get_rgb((*role).into())))))
            .collect();
        Value::Object(roles)
    };
    serde_json::json!({ "seed": hex(seed), "dark": mode(true), "light": mode(false) }).to_string()
}

fn camel(snake: &str) -> String {
    let mut out = String::with_capacity(snake.len());
    let mut upper = false;
    for ch in snake.chars() {
        if ch == '_' {
            upper = true;
        } else if upper {
            out.extend(ch.to_uppercase());
            upper = false;
        } else {
            out.push(ch);
        }
    }
    out
}

/// Same ranking as the Compose `extractWallpaperSeed`: downscale to 128px,
/// drop transparent pixels, quantise to 128 colours, score for 4, and retry
/// unfiltered when only the fallback survives.
fn wallpaper_seed(path: &Path) -> Option<Rgb> {
    let image = image::open(path).ok()?;
    let thumb = image
        .resize(MAX_EXTRACTION_SIZE, MAX_EXTRACTION_SIZE, image::imageops::FilterType::Triangle)
        .to_rgba8();
    let pixels: Vec<Rgb> = thumb
        .pixels()
        .filter(|pixel| pixel[3] >= 128)
        .map(|pixel| Rgb::new(pixel[0], pixel[1], pixel[2]))
        .collect();
    if pixels.is_empty() {
        return None;
    }

    let quantized = QuantizerCelebi::quantize(&pixels, 128).color_to_count;
    let mut ranked = Score::score(&quantized, Some(4), Some(GOOGLE_BLUE), Some(true));
    if ranked.len() == 1 && ranked[0] == GOOGLE_BLUE {
        let unfiltered = Score::score(&quantized, Some(4), Some(GOOGLE_BLUE), Some(false));
        if !unfiltered.is_empty() && (unfiltered.len() > 1 || unfiltered[0] != GOOGLE_BLUE) {
            ranked = unfiltered;
        } else if let Some((color, _)) = quantized.iter().max_by_key(|(_, count)| **count) {
            ranked = vec![*color];
        }
    }
    ranked.first().copied()
}

fn xdg(var: &str, fallback: &str, home: &Path) -> PathBuf {
    std::env::var_os(var)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(fallback))
}

fn normalize(raw: &str) -> Option<PathBuf> {
    let value = raw.trim().trim_matches(|c| c == '\'' || c == '"');
    let value = value.strip_prefix("file://").or_else(|| value.strip_prefix("file:")).unwrap_or(value);
    let path = PathBuf::from(value);
    path.is_file().then_some(path)
}

fn run(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .filter(|text| !text.is_empty())
}

/// The wallpaper lookup order of the Compose client, minus the rarer
/// desktops (XFCE, MATE, Cinnamon, sway, nitrogen) for now.
fn current_wallpaper(home: &Path) -> Option<PathBuf> {
    let config = xdg("XDG_CONFIG_HOME", ".config", home);
    let cache = xdg("XDG_CACHE_HOME", ".cache", home);

    if let Ok(text) = fs::read_to_string(cache.join("kiite/active_wallpaper.json")) {
        let path = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|root| root["path"].as_str().and_then(normalize));
        if path.is_some() {
            return path;
        }
    }

    if let Some(path) = fs::read_to_string(cache.join("wal/wal")).ok().as_deref().and_then(normalize) {
        return Some(path);
    }

    let gnome_dark = run("gsettings", &["get", "org.gnome.desktop.interface", "color-scheme"])
        .is_some_and(|scheme| scheme.to_lowercase().contains("dark"));
    let keys: &[&str] = if gnome_dark { &["picture-uri-dark", "picture-uri"] } else { &["picture-uri"] };
    for key in keys {
        if let Some(path) = run("gsettings", &["get", "org.gnome.desktop.background", key]).as_deref().and_then(normalize) {
            return Some(path);
        }
    }

    if let Ok(text) = fs::read_to_string(config.join("plasma-org.kde.plasma.desktop-appletsrc")) {
        let path = text
            .lines()
            .find_map(|line| line.strip_prefix("Image=file:").and_then(normalize));
        if path.is_some() {
            return path;
        }
    }

    if let Ok(text) = fs::read_to_string(config.join("hypr/hyprpaper.conf")) {
        let path = text.lines().rev().find_map(|line| {
            let value = line.split_once('=')?.1.trim();
            normalize(value.rsplit(',').next()?)
        });
        if path.is_some() {
            return path;
        }
    }

    run("swww", &["query"]).and_then(|output| {
        output
            .lines()
            .find_map(|line| line.split_once("image:").and_then(|(_, path)| normalize(path)))
    })
}
