use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;

pub mod app;
#[cfg(debug_assertions)]
pub mod automation;
pub mod builder;
pub mod clipboard;
pub mod colors;
pub mod components;
pub mod control;
pub mod emoji_display;
pub mod event;
pub mod font_system;
pub mod form;
pub mod glyph_cache;
pub mod gpu;
pub mod inspect;
pub mod keymap;
pub mod layout;
pub mod node;
pub mod paint;
pub mod perf;
pub mod popout;
pub mod reveal;
pub mod scroll_animation;
pub mod selection;
pub mod signal;
pub mod style;
pub mod text_cache;
pub mod theme;
pub mod timer;
pub mod timestamp;
pub mod typography;
pub mod waker;
pub mod widget;

pub const BUNDLED_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/bundled.css"));

pub mod classes {
    include!(concat!(env!("OUT_DIR"), "/css_classes.rs"));
}

pub mod fonts;
pub mod icon_handle;
pub mod svg_icon;

pub mod icons {
    pub use crate::icon_handle::{Icon, IconPath, install_resolver};
    include!(concat!(env!("OUT_DIR"), "/requiem_icons.rs"));
}

thread_local! {
    static THEME_NAME: Cell<Option<&'static str>> = const { Cell::new(None) };
    static THEME_GENERATION: Cell<u64> = const { Cell::new(0) };
    static THEME_TOKENS: RefCell<HashMap<String, String>> =
        RefCell::new(HashMap::new());
}

pub fn set_theme_name(name: &'static str) {
    let changed = THEME_NAME.with(|slot| {
        let changed = slot.get() != Some(name);
        slot.set(Some(name));
        changed
    });
    if changed {
        THEME_GENERATION.with(|slot| slot.set(slot.get().wrapping_add(1)));
        paint::request_repaint();
    }
}

pub fn theme_name() -> String {
    THEME_NAME
        .with(|slot| slot.get())
        .unwrap_or("dark")
        .to_string()
}

pub fn theme_generation() -> u64 {
    THEME_GENERATION.with(|slot| slot.get())
}

pub(crate) fn install_theme_tokens(tokens: HashMap<String, String>) {
    THEME_TOKENS.with(|slot| *slot.borrow_mut() = tokens);
}

/// Active theme value for `name`, or `fallback` before a stylesheet is installed.
pub fn theme_color(name: &str, fallback: skia_safe::Color) -> skia_safe::Color {
    THEME_TOKENS.with(|slot| {
        slot.borrow()
            .get(name)
            .and_then(|value| style::values::parse_color(value))
            .unwrap_or(fallback)
    })
}
