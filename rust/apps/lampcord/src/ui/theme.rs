use std::{fs, path::PathBuf};

use gpui::Rgba;
use material_colors::{
    color::Argb,
    hct::Hct,
    scheme::{
        Scheme,
        variant::{
            SchemeContent, SchemeExpressive, SchemeFidelity, SchemeFruitSalad, SchemeMonochrome,
            SchemeNeutral, SchemeRainbow, SchemeTonalSpot, SchemeVibrant,
        },
    },
};

pub use gpui_material::theme::{M3Palette, dark_scheme, light_scheme};

/// App theme palette (M3 Expressive), shared with gpui-material components.
pub type M3Theme = M3Palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaletteStyle {
    TonalSpot,
    #[default]
    Expressive,
    Vibrant,
    Neutral,
    Rainbow,
    FruitSalad,
    Content,
    Fidelity,
    Monochrome,
}

impl PaletteStyle {
    fn from_setting(s: &str) -> Self {
        match s.trim().to_ascii_uppercase().as_str() {
            "TONAL_SPOT" | "TONALSPOT" => Self::TonalSpot,
            "EXPRESSIVE" => Self::Expressive,
            "VIBRANT" => Self::Vibrant,
            "NEUTRAL" | "SPRITZ" => Self::Neutral,
            "RAINBOW" => Self::Rainbow,
            "FRUIT_SALAD" | "FRUITSALAD" => Self::FruitSalad,
            "CONTENT" => Self::Content,
            "FIDELITY" => Self::Fidelity,
            "MONOCHROME" | "MONOCHROMATIC" => Self::Monochrome,
            _ => Self::Expressive,
        }
    }
}

#[must_use]
pub fn dark_theme() -> M3Theme {
    // Fallback to a pleasant baseline, not the vendored Rust-orange scheme.
    generate_palette(Argb::new(255, 208, 188, 255), PaletteStyle::TonalSpot, true)
        .unwrap_or_else(dark_scheme)
}

#[must_use]
pub fn light_theme() -> M3Theme {
    generate_palette(Argb::new(255, 103, 80, 164), PaletteStyle::TonalSpot, false)
        .unwrap_or_else(light_scheme)
}

fn lampcord_settings_path() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(dir)
            .join("lampcord")
            .join("settings.properties");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("lampcord")
            .join("settings.properties");
    }
    PathBuf::from("settings.properties")
}

fn kiite_active_theme_path() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CACHE_HOME") {
        return PathBuf::from(dir).join("kiite").join("active_theme.json");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".cache")
            .join("kiite")
            .join("active_theme.json");
    }
    PathBuf::from("active_theme.json")
}

fn load_kiite_synced_theme() -> Option<M3Theme> {
    let path = kiite_active_theme_path();
    let contents = fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&contents).ok()?;
    let palette = value.get("palette")?;
    serde_json::from_value::<M3Palette>(palette.clone()).ok()
}

fn unescape_java_properties_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('t') => out.push('\t'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('f') => out.push('\x0C'),
                Some('\\') => out.push('\\'),
                Some(other) => out.push(other), // handles \#, \=, \:, spaces, unicode escapes simplified
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn parse_properties(contents: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let mut logical = String::new();
    let mut continuing = false;

    for raw_line in contents.lines() {
        if !continuing {
            let trimmed_start = raw_line.trim_start();
            if trimmed_start.is_empty()
                || trimmed_start.starts_with('#')
                || trimmed_start.starts_with('!')
            {
                continue;
            }
            logical.clear();
        }

        let mut line = raw_line.to_string();
        // Handle line continuation: odd number of trailing backslashes means continue
        let mut trailing_backslashes = 0usize;
        for ch in line.chars().rev() {
            if ch == '\\' {
                trailing_backslashes += 1;
            } else {
                break;
            }
        }
        if trailing_backslashes % 2 == 1 {
            continuing = true;
            line.pop(); // remove continuation backslash
            if logical.is_empty() {
                logical.push_str(line.trim_start());
            } else {
                logical.push_str(line.trim_start());
            }
            continue;
        } else {
            if logical.is_empty() {
                logical.push_str(raw_line.trim_start());
            } else {
                logical.push_str(raw_line.trim_start());
            }
            continuing = false;
        }

        let current = std::mem::take(&mut logical);
        if current.is_empty() || current.starts_with('#') || current.starts_with('!') {
            continue;
        }

        // Find separator (= or : or whitespace) not escaped
        let chars: Vec<char> = current.chars().collect();
        let mut sep_idx = None;
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '\\' {
                i += 2;
                continue;
            }
            if chars[i] == '=' || chars[i] == ':' {
                sep_idx = Some(i);
                break;
            }
            if chars[i].is_whitespace() {
                sep_idx = Some(i);
                break;
            }
            i += 1;
        }

        if let Some(idx) = sep_idx {
            let key: String = chars[..idx].iter().collect();
            let key = unescape_java_properties_value(key.trim());
            let mut val_start = idx;
            if val_start < chars.len() && (chars[val_start] == '=' || chars[val_start] == ':') {
                val_start += 1;
            }
            while val_start < chars.len() && chars[val_start].is_whitespace() {
                val_start += 1;
            }
            let value: String = chars[val_start..].iter().collect();
            let value = unescape_java_properties_value(value.trim_end());
            if !key.is_empty() {
                map.insert(key, value);
            }
        } else {
            let key = unescape_java_properties_value(current.trim());
            if !key.is_empty() {
                map.insert(key, String::new());
            }
        }
    }

    map
}

fn parse_hex_color(s: &str) -> Option<Argb> {
    let s = s.trim().trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Argb::new(255, r, g, b))
    } else if s.len() == 8 {
        let a = u8::from_str_radix(&s[0..2], 16).ok()?;
        let r = u8::from_str_radix(&s[2..4], 16).ok()?;
        let g = u8::from_str_radix(&s[4..6], 16).ok()?;
        let b = u8::from_str_radix(&s[6..8], 16).ok()?;
        Some(Argb::new(a, r, g, b))
    } else {
        None
    }
}

pub fn generate_palette(source: Argb, style: PaletteStyle, is_dark: bool) -> Option<M3Theme> {
    let hct = Hct::new(source);
    let contrast = None;
    let dynamic_scheme = match style {
        PaletteStyle::TonalSpot => SchemeTonalSpot::new(hct, is_dark, contrast).scheme,
        PaletteStyle::Expressive => SchemeExpressive::new(hct, is_dark, contrast).scheme,
        PaletteStyle::Vibrant => SchemeVibrant::new(hct, is_dark, contrast).scheme,
        PaletteStyle::Neutral => SchemeNeutral::new(hct, is_dark, contrast).scheme,
        PaletteStyle::Rainbow => SchemeRainbow::new(hct, is_dark, contrast).scheme,
        PaletteStyle::FruitSalad => SchemeFruitSalad::new(hct, is_dark, contrast).scheme,
        PaletteStyle::Content => SchemeContent::new(hct, is_dark, contrast).scheme,
        PaletteStyle::Fidelity => SchemeFidelity::new(hct, is_dark, contrast).scheme,
        PaletteStyle::Monochrome => SchemeMonochrome::new(hct, is_dark, contrast).scheme,
    };
    let scheme = Scheme::from(dynamic_scheme);
    Some(M3Palette::from(&scheme))
}

pub fn resolve_lampcord_theme_mode(mode: &str) -> ThemeMode {
    match mode.trim().to_ascii_uppercase().as_str() {
        "LIGHT" => ThemeMode::Light,
        "DARK" => ThemeMode::Dark,
        "SYSTEM" => {
            // Best-effort: prefer dark if unknown; can be improved with desktop portal checks later.
            ThemeMode::Dark
        }
        _ => ThemeMode::Dark,
    }
}

#[must_use]
pub fn load_shell_synced_theme() -> M3Theme {
    // Prefer Kiite's active theme palette (current shell colors).
    if let Some(theme) = load_kiite_synced_theme() {
        return theme;
    }

    // Fallback: derive from Lampcord's saved accent/mode/style if Kiite theme is unavailable.
    let path = lampcord_settings_path();
    if let Ok(contents) = fs::read_to_string(path) {
        let props = parse_properties(&contents);

        let accent = props
            .get("accent_color")
            .and_then(|v| parse_hex_color(v))
            .unwrap_or_else(|| Argb::new(255, 103, 80, 164)); // M3 default-ish purple

        let mode = props
            .get("theme_mode")
            .map(|s| resolve_lampcord_theme_mode(s))
            .unwrap_or(ThemeMode::Dark);

        let style = props
            .get("theme_palette_style")
            .map(|s| PaletteStyle::from_setting(s))
            .unwrap_or(PaletteStyle::Expressive);

        let is_dark = matches!(mode, ThemeMode::Dark);
        if let Some(theme) = generate_palette(accent, style, is_dark) {
            return theme;
        }
        return if is_dark { dark_theme() } else { light_theme() };
    }

    dark_theme()
}

// Compatibility alias for earlier call sites.
#[must_use]
pub fn load_lampcord_synced_theme() -> M3Theme {
    load_shell_synced_theme()
}

// Keep for compatibility with older call sites.
#[must_use]
pub fn rgba(r: u8, g: u8, b: u8, a: f32) -> Rgba {
    Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a,
    }
}
