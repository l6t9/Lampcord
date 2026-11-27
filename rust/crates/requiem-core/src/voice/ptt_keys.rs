//! Global key state for push-to-talk off Windows.
//!
//! A shortcut is stored as Windows virtual-key codes whatever the OS, so a saved setting means
//! the same thing everywhere. Each code maps to the platform's own key: a CoreGraphics key code
//! on macOS, a keysym on X11. Wayland offers no global key state, so there nothing is ever down
//! and the shortcut can't be recorded.

struct Key {
    vk: u32,
    name: &'static str,
    /// The macOS virtual key code (`kVK_*`).
    mac: u16,
    /// The X11 keysym.
    keysym: u32,
}

const fn key(vk: u32, name: &'static str, mac: u16, keysym: u32) -> Key {
    Key {
        vk,
        name,
        mac,
        keysym,
    }
}

const KEYS: &[Key] = &[
    key(0x08, "BACKSPACE", 51, 0xff08),
    key(0x09, "TAB", 48, 0xff09),
    key(0x0D, "ENTER", 36, 0xff0d),
    key(0x14, "CAPS LOCK", 57, 0xffe5),
    key(0x1B, "ESCAPE", 53, 0xff1b),
    key(0x20, "SPACE", 49, 0x20),
    key(0x21, "PAGE UP", 116, 0xff55),
    key(0x22, "PAGE DOWN", 121, 0xff56),
    key(0x23, "END", 119, 0xff57),
    key(0x24, "HOME", 115, 0xff50),
    key(0x25, "LEFT", 123, 0xff51),
    key(0x26, "UP", 126, 0xff52),
    key(0x27, "RIGHT", 124, 0xff53),
    key(0x28, "DOWN", 125, 0xff54),
    key(0x2D, "INSERT", 114, 0xff63),
    key(0x2E, "DELETE", 117, 0xffff),
    key(0x30, "0", 29, 0x30),
    key(0x31, "1", 18, 0x31),
    key(0x32, "2", 19, 0x32),
    key(0x33, "3", 20, 0x33),
    key(0x34, "4", 21, 0x34),
    key(0x35, "5", 23, 0x35),
    key(0x36, "6", 22, 0x36),
    key(0x37, "7", 26, 0x37),
    key(0x38, "8", 28, 0x38),
    key(0x39, "9", 25, 0x39),
    key(0x41, "A", 0, 0x61),
    key(0x42, "B", 11, 0x62),
    key(0x43, "C", 8, 0x63),
    key(0x44, "D", 2, 0x64),
    key(0x45, "E", 14, 0x65),
    key(0x46, "F", 3, 0x66),
    key(0x47, "G", 5, 0x67),
    key(0x48, "H", 4, 0x68),
    key(0x49, "I", 34, 0x69),
    key(0x4A, "J", 38, 0x6a),
    key(0x4B, "K", 40, 0x6b),
    key(0x4C, "L", 37, 0x6c),
    key(0x4D, "M", 46, 0x6d),
    key(0x4E, "N", 45, 0x6e),
    key(0x4F, "O", 31, 0x6f),
    key(0x50, "P", 35, 0x70),
    key(0x51, "Q", 12, 0x71),
    key(0x52, "R", 15, 0x72),
    key(0x53, "S", 1, 0x73),
    key(0x54, "T", 17, 0x74),
    key(0x55, "U", 32, 0x75),
    key(0x56, "V", 9, 0x76),
    key(0x57, "W", 13, 0x77),
    key(0x58, "X", 7, 0x78),
    key(0x59, "Y", 16, 0x79),
    key(0x5A, "Z", 6, 0x7a),
    key(0x5B, "META", 55, 0xffeb),
    key(0x5C, "META", 54, 0xffec),
    key(0x60, "NUM 0", 82, 0xffb0),
    key(0x61, "NUM 1", 83, 0xffb1),
    key(0x62, "NUM 2", 84, 0xffb2),
    key(0x63, "NUM 3", 85, 0xffb3),
    key(0x64, "NUM 4", 86, 0xffb4),
    key(0x65, "NUM 5", 87, 0xffb5),
    key(0x66, "NUM 6", 88, 0xffb6),
    key(0x67, "NUM 7", 89, 0xffb7),
    key(0x68, "NUM 8", 91, 0xffb8),
    key(0x69, "NUM 9", 92, 0xffb9),
    key(0x70, "F1", 122, 0xffbe),
    key(0x71, "F2", 120, 0xffbf),
    key(0x72, "F3", 99, 0xffc0),
    key(0x73, "F4", 118, 0xffc1),
    key(0x74, "F5", 96, 0xffc2),
    key(0x75, "F6", 97, 0xffc3),
    key(0x76, "F7", 98, 0xffc4),
    key(0x77, "F8", 100, 0xffc5),
    key(0x78, "F9", 101, 0xffc6),
    key(0x79, "F10", 109, 0xffc7),
    key(0x7A, "F11", 103, 0xffc8),
    key(0x7B, "F12", 111, 0xffc9),
    key(0xA0, "SHIFT", 56, 0xffe1),
    key(0xA1, "SHIFT", 60, 0xffe2),
    key(0xA2, "CTRL", 59, 0xffe3),
    key(0xA3, "CTRL", 62, 0xffe4),
    key(0xA4, "ALT", 58, 0xffe9),
    key(0xA5, "ALT", 61, 0xffea),
    key(0xBA, ";", 41, 0x3b),
    key(0xBB, "=", 24, 0x3d),
    key(0xBC, ",", 43, 0x2c),
    key(0xBD, "-", 27, 0x2d),
    key(0xBE, ".", 47, 0x2e),
    key(0xBF, "/", 44, 0x2f),
    key(0xC0, "`", 50, 0x60),
    key(0xDB, "[", 33, 0x5b),
    key(0xDC, "\\", 42, 0x5c),
    key(0xDD, "]", 30, 0x5d),
    key(0xDE, "'", 39, 0x27),
];

/// The mouse buttons a shortcut may include, as Windows names them: the middle button and
/// the two side buttons.
const MOUSE: &[(u32, &str)] = &[(0x04, "MOUSE 3"), (0x05, "MOUSE 4"), (0x06, "MOUSE 5")];

fn find(vk: u32) -> Option<&'static Key> {
    KEYS.iter().find(|key| key.vk == vk)
}

pub fn recordable(vk: u32) -> bool {
    find(vk).is_some() || MOUSE.iter().any(|(code, _)| *code == vk)
}

pub fn name(vk: u32) -> String {
    if let Some((_, name)) = MOUSE.iter().find(|(code, _)| *code == vk) {
        return (*name).to_string();
    }
    find(vk).map_or_else(|| format!("KEY {vk}"), |key| key.name.to_string())
}

/// Every key and button of a shortcut that is down right now.
pub fn pressed() -> Vec<u32> {
    let state = platform::State::capture();
    KEYS.iter()
        .map(|key| key.vk)
        .chain(MOUSE.iter().map(|(vk, _)| *vk))
        .filter(|&vk| state.is_down(vk))
        .collect()
}

pub fn is_down(vk: u32) -> bool {
    platform::State::capture().is_down(vk)
}

#[cfg(target_os = "macos")]
mod platform {
    use super::{find, MOUSE};

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGEventSourceKeyState(state: i32, key: u16) -> bool;
        fn CGEventSourceButtonState(state: i32, button: u32) -> bool;
    }

    /// kCGEventSourceStateCombinedSessionState: what the user is doing now, whatever app has focus.
    const COMBINED_SESSION_STATE: i32 = 0;

    pub struct State;

    impl State {
        pub fn capture() -> Self {
            State
        }

        pub fn is_down(&self, vk: u32) -> bool {
            if let Some(index) = MOUSE.iter().position(|(code, _)| *code == vk) {
                // Buttons are numbered from 0; the middle one is 2.
                let button = 2 + index as u32;
                // SAFETY: plain CoreGraphics query with no pointer arguments.
                return unsafe { CGEventSourceButtonState(COMBINED_SESSION_STATE, button) };
            }
            // SAFETY: plain CoreGraphics query with no pointer arguments.
            find(vk).is_some_and(|key| unsafe {
                CGEventSourceKeyState(COMBINED_SESSION_STATE, key.mac)
            })
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod platform {
    use std::cell::RefCell;
    use std::os::raw::{c_char, c_int, c_uint};
    use std::ptr;

    use x11_dl::xlib::{Button2Mask, Display, Xlib};

    use super::find;

    struct Connection {
        xlib: Xlib,
        display: *mut Display,
    }

    impl Drop for Connection {
        fn drop(&mut self) {
            // SAFETY: `display` was returned non-null by XOpenDisplay, is owned by this struct, and
            // is closed exactly once here.
            unsafe { (self.xlib.XCloseDisplay)(self.display) };
        }
    }

    thread_local! {
        static CONNECTION: RefCell<Option<Option<Connection>>> = const { RefCell::new(None) };
    }

    fn on_native_wayland() -> bool {
        std::env::var_os("WAYLAND_DISPLAY").is_some_and(|v| !v.is_empty())
            && std::env::var("WINIT_UNIX_BACKEND").map_or(true, |b| b != "x11")
    }

    pub struct State {
        keymap: Option<[c_char; 32]>,
        middle_button: bool,
    }

    impl State {
        pub fn capture() -> Self {
            let none = State {
                keymap: None,
                middle_button: false,
            };
            if on_native_wayland() {
                return none;
            }
            CONNECTION.with(|slot| {
                let mut slot = slot.borrow_mut();
                let connection = slot.get_or_insert_with(|| {
                    let xlib = Xlib::open().ok()?;
                    // SAFETY: a null name makes Xlib use $DISPLAY; the result is null-checked
                    // before it is used.
                    let display = unsafe { (xlib.XOpenDisplay)(ptr::null()) };
                    (!display.is_null()).then_some(Connection { xlib, display })
                });
                let Some(connection) = connection.as_ref() else {
                    return none;
                };
                // SAFETY: `connection.display` is the open connection owned by `Connection`;
                // `keymap` is the 32-byte buffer XQueryKeymap requires and the other out-parameters
                // are valid locals.
                unsafe {
                    let mut keymap = [0 as c_char; 32];
                    (connection.xlib.XQueryKeymap)(connection.display, keymap.as_mut_ptr());
                    let root = (connection.xlib.XDefaultRootWindow)(connection.display);
                    let (mut root_return, mut child_return) = (0, 0);
                    let (mut root_x, mut root_y, mut win_x, mut win_y): (c_int, c_int, c_int, c_int) =
                        (0, 0, 0, 0);
                    let mut mask: c_uint = 0;
                    (connection.xlib.XQueryPointer)(
                        connection.display,
                        root,
                        &mut root_return,
                        &mut child_return,
                        &mut root_x,
                        &mut root_y,
                        &mut win_x,
                        &mut win_y,
                        &mut mask,
                    );
                    State {
                        keymap: Some(keymap),
                        middle_button: mask & Button2Mask != 0,
                    }
                }
            })
        }

        pub fn is_down(&self, vk: u32) -> bool {
            // The side buttons aren't in X's pointer mask; only the middle one is.
            if vk == 0x04 {
                return self.middle_button;
            }
            let (Some(keymap), Some(key)) = (&self.keymap, find(vk)) else {
                return false;
            };
            let keycode = CONNECTION.with(|slot| {
                let slot = slot.borrow();
                let connection = slot.as_ref()?.as_ref()?;
                // SAFETY: `connection.display` is the open connection owned by `Connection`, and
                // XKeysymToKeycode only reads it.
                let code = unsafe {
                    (connection.xlib.XKeysymToKeycode)(connection.display, key.keysym.into())
                };
                (code != 0).then_some(code as usize)
            });
            keycode.is_some_and(|code| keymap[code / 8] as u8 & (1 << (code % 8)) != 0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_is_listed_once_and_named() {
        for (index, key) in KEYS.iter().enumerate() {
            assert!(
                KEYS[index + 1..].iter().all(|other| other.vk != key.vk),
                "{} is listed twice",
                key.vk
            );
            assert!(!key.name.is_empty());
        }
    }

    #[test]
    fn names_follow_the_windows_ones() {
        assert_eq!(name(0x41), "A");
        assert_eq!(name(0xA0), "SHIFT");
        assert_eq!(name(0x05), "MOUSE 4");
        assert_eq!(name(0xFE), "KEY 254");
        assert!(recordable(0x70));
        assert!(!recordable(0x01));
    }
}
