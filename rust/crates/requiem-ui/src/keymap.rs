use winit::keyboard::{Key, ModifiersState, NamedKey as WinitNamedKey};

use crate::event::NamedKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChordKey {
    Char(char),
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    PageUp,
    PageDown,
    Home,
    End,
    Enter,
    Escape,
    Tab,
    Space,
    Backspace,
    Delete,
    F(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Chord {
    pub key: ChordKey,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

impl Chord {
    pub fn parse(bind: &str) -> Option<Chord> {
        let bind = bind.to_lowercase();
        let (mods, key) = bind.rsplit_once('+').unwrap_or(("", bind.as_str()));
        let mut chord = Chord {
            key: Self::parse_key(key)?,
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
        };
        for modifier in mods.split('+').filter(|m| !m.is_empty()) {
            match modifier {
                "mod" if cfg!(target_os = "macos") => chord.meta = true,
                "mod" | "ctrl" => chord.ctrl = true,
                "shift" => chord.shift = true,
                "alt" => chord.alt = true,
                "meta" | "cmd" | "super" => chord.meta = true,
                _ => return None,
            }
        }
        Some(chord)
    }

    fn parse_key(key: &str) -> Option<ChordKey> {
        Some(match key {
            "up" => ChordKey::ArrowUp,
            "down" => ChordKey::ArrowDown,
            "left" => ChordKey::ArrowLeft,
            "right" => ChordKey::ArrowRight,
            "pageup" => ChordKey::PageUp,
            "pagedown" => ChordKey::PageDown,
            "home" => ChordKey::Home,
            "end" => ChordKey::End,
            "return" | "enter" => ChordKey::Enter,
            "esc" | "escape" => ChordKey::Escape,
            "tab" => ChordKey::Tab,
            "space" | "spacebar" => ChordKey::Space,
            "backspace" => ChordKey::Backspace,
            "delete" => ChordKey::Delete,
            "plus" => ChordKey::Char('='),
            "minus" => ChordKey::Char('-'),
            _ => {
                if let Some(n) = key
                    .strip_prefix('f')
                    .and_then(|n| n.parse::<u8>().ok())
                    .filter(|n| (1..=24).contains(n))
                {
                    return Some(ChordKey::F(n));
                }
                let mut chars = key.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => ChordKey::Char(c),
                    _ => return None,
                }
            }
        })
    }

    pub fn from_key(
        unmodified: &Key,
        text: Option<&str>,
        modifiers: ModifiersState,
    ) -> Option<Chord> {
        let key = match unmodified {
            Key::Character(s) => {
                let mut chars = s.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => ChordKey::Char(c.to_lowercase().next().unwrap_or(c)),
                    _ => return None,
                }
            }
            Key::Named(named) => match named {
                WinitNamedKey::ArrowUp => ChordKey::ArrowUp,
                WinitNamedKey::ArrowDown => ChordKey::ArrowDown,
                WinitNamedKey::ArrowLeft => ChordKey::ArrowLeft,
                WinitNamedKey::ArrowRight => ChordKey::ArrowRight,
                WinitNamedKey::PageUp => ChordKey::PageUp,
                WinitNamedKey::PageDown => ChordKey::PageDown,
                WinitNamedKey::Home => ChordKey::Home,
                WinitNamedKey::End => ChordKey::End,
                WinitNamedKey::Enter => ChordKey::Enter,
                WinitNamedKey::Escape => ChordKey::Escape,
                WinitNamedKey::Tab => ChordKey::Tab,
                WinitNamedKey::Space => ChordKey::Space,
                WinitNamedKey::Backspace => ChordKey::Backspace,
                WinitNamedKey::Delete => ChordKey::Delete,
                WinitNamedKey::F1 => ChordKey::F(1),
                WinitNamedKey::F2 => ChordKey::F(2),
                WinitNamedKey::F3 => ChordKey::F(3),
                WinitNamedKey::F4 => ChordKey::F(4),
                WinitNamedKey::F5 => ChordKey::F(5),
                WinitNamedKey::F6 => ChordKey::F(6),
                WinitNamedKey::F7 => ChordKey::F(7),
                WinitNamedKey::F8 => ChordKey::F(8),
                WinitNamedKey::F9 => ChordKey::F(9),
                WinitNamedKey::F10 => ChordKey::F(10),
                WinitNamedKey::F11 => ChordKey::F(11),
                WinitNamedKey::F12 => ChordKey::F(12),
                _ => return None,
            },
            _ => return None,
        };
        let chord = Chord {
            key,
            ctrl: modifiers.control_key(),
            shift: modifiers.shift_key(),
            alt: modifiers.alt_key(),
            meta: modifiers.super_key(),
        };

        if chord.ctrl
            && chord.alt
            && matches!(key, ChordKey::Char(_))
            && text.is_some_and(|t| !t.is_empty() && !t.chars().any(char::is_control))
        {
            return None;
        }
        Some(chord)
    }

    pub fn is_global(&self) -> bool {
        self.ctrl
            || self.alt
            || self.meta
            || matches!(
                self.key,
                ChordKey::F(_) | ChordKey::PageUp | ChordKey::PageDown
            )
            || (self.shift && self.key == ChordKey::Escape)
    }
}

pub fn ctrl_shortcut(ch: &str, shift: bool) -> Option<NamedKey> {
    match ch.to_lowercase().as_str() {
        "a" if !shift => Some(NamedKey::SelectAll),
        "c" => Some(NamedKey::Copy),
        "x" => Some(NamedKey::Cut),
        "v" => Some(NamedKey::Paste),
        "z" if shift => Some(NamedKey::Redo),
        "z" => Some(NamedKey::Undo),
        "y" => Some(NamedKey::Redo),
        _ => None,
    }
}

pub fn named_key(key: &WinitNamedKey) -> Option<NamedKey> {
    match key {
        WinitNamedKey::Backspace => Some(NamedKey::Backspace),
        WinitNamedKey::Delete => Some(NamedKey::Delete),
        WinitNamedKey::Enter => Some(NamedKey::Enter),
        WinitNamedKey::ArrowLeft => Some(NamedKey::ArrowLeft),
        WinitNamedKey::ArrowRight => Some(NamedKey::ArrowRight),
        WinitNamedKey::ArrowUp => Some(NamedKey::ArrowUp),
        WinitNamedKey::ArrowDown => Some(NamedKey::ArrowDown),
        WinitNamedKey::Home => Some(NamedKey::Home),
        WinitNamedKey::End => Some(NamedKey::End),
        WinitNamedKey::Escape => Some(NamedKey::Escape),
        WinitNamedKey::Tab => Some(NamedKey::Tab),
        WinitNamedKey::Space => Some(NamedKey::Space),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctrl_chords() {
        assert_eq!(ctrl_shortcut("C", false), Some(NamedKey::Copy));
        assert_eq!(ctrl_shortcut("z", false), Some(NamedKey::Undo));
        assert_eq!(ctrl_shortcut("z", true), Some(NamedKey::Redo));
        assert_eq!(ctrl_shortcut("q", false), None);
    }

    #[test]
    fn parses_discord_binds() {
        let chord = |bind| Chord::parse(bind).unwrap_or_else(|| panic!("{bind}"));
        assert_eq!(
            chord("alt+down"),
            Chord {
                key: ChordKey::ArrowDown,
                ctrl: false,
                shift: false,
                alt: true,
                meta: false
            }
        );
        assert!(chord("mod+shift+alt+up").shift);
        assert_eq!(chord("shift+esc").key, ChordKey::Escape);
        assert_eq!(chord("f1").key, ChordKey::F(1));
        assert_eq!(chord("mod+plus").key, ChordKey::Char('='));
        assert_eq!(chord("ctrl+shift+'").key, ChordKey::Char('\''));
        assert_eq!(chord("mod+/").key, ChordKey::Char('/'));
        assert!(chord("mod+k").ctrl || chord("mod+k").meta);
        assert_eq!(Chord::parse("any-character"), None);
        assert_eq!(Chord::parse("h+h+right+n+k"), None);
    }

    #[test]
    fn global_chords() {
        assert!(Chord::parse("mod+k").unwrap().is_global());
        assert!(Chord::parse("pageup").unwrap().is_global());
        assert!(Chord::parse("shift+esc").unwrap().is_global());
        assert!(!Chord::parse("esc").unwrap().is_global());
        assert!(!Chord::parse("up").unwrap().is_global());
        assert!(!Chord::parse("shift+k").unwrap().is_global());
    }

    #[test]
    fn named_keys() {
        assert_eq!(named_key(&WinitNamedKey::Tab), Some(NamedKey::Tab));
        assert_eq!(named_key(&WinitNamedKey::F1), None);
    }
}
