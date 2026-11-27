//! How long the user has gone without keyboard or mouse input, if the OS says.

#[cfg(windows)]
use windows_sys::Win32::System::SystemInformation::GetTickCount;
#[cfg(windows)]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetLastInputInfo;
#[cfg(windows)]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::LASTINPUTINFO;

#[cfg(windows)]
pub fn idle_ms() -> Option<u64> {
    let mut info = LASTINPUTINFO {
        cbSize: size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };

    // SAFETY: `info` is a valid LASTINPUTINFO with `cbSize` set, as GetLastInputInfo requires;
    // GetTickCount has no preconditions.
    unsafe {
        if GetLastInputInfo(&mut info) == 0 {
            return None;
        }

        Some(u64::from(GetTickCount().wrapping_sub(info.dwTime)))
    }
}

#[cfg(target_os = "macos")]
pub fn idle_ms() -> Option<u64> {
    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGEventSourceSecondsSinceLastEventType(state: i32, event_type: u32) -> f64;
    }
    // kCGEventSourceStateCombinedSessionState and kCGAnyInputEventType (all bits set).
    const COMBINED_SESSION_STATE: i32 = 0;
    const ANY_INPUT_EVENT: u32 = u32::MAX;
    let seconds =
        // SAFETY: plain CoreGraphics query with no pointer arguments.
        unsafe { CGEventSourceSecondsSinceLastEventType(COMBINED_SESSION_STATE, ANY_INPUT_EVENT) };
    (seconds.is_finite() && seconds >= 0.0).then(|| (seconds * 1000.0) as u64)
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn idle_ms() -> Option<u64> {
    let on_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some_and(|v| !v.is_empty());
    if on_wayland {
        wayland::idle_ms().or_else(x11::idle_ms)
    } else {
        x11::idle_ms().or_else(wayland::idle_ms)
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod x11 {
    use std::cell::RefCell;
    use std::ptr;

    use x11_dl::xlib::{Display, Xlib};
    use x11_dl::xss::Xss;

    struct Connection {
        xlib: Xlib,
        xss: Xss,
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

    /// The MIT-SCREEN-SAVER extension keeps the time since the last input.
    pub fn idle_ms() -> Option<u64> {
        CONNECTION.with(|slot| {
            let mut slot = slot.borrow_mut();
            let connection = slot.get_or_insert_with(|| {
                let xlib = Xlib::open().ok()?;
                let xss = Xss::open().ok()?;
                // SAFETY: a null name makes Xlib use $DISPLAY; the result is null-checked before it
                // is used.
                let display = unsafe { (xlib.XOpenDisplay)(ptr::null()) };
                (!display.is_null()).then_some(Connection { xlib, xss, display })
            });
            let connection = connection.as_ref()?;
            // SAFETY: `connection.display` is the open connection owned by `Connection`. `info` is
            // checked for null, read while valid and freed once with XFree.
            unsafe {
                let info = (connection.xss.XScreenSaverAllocInfo)();
                if info.is_null() {
                    return None;
                }
                let root = (connection.xlib.XDefaultRootWindow)(connection.display);
                let queried = (connection.xss.XScreenSaverQueryInfo)(connection.display, root, info);
                let idle = (queried != 0).then(|| (*info).idle as u64);
                (connection.xlib.XFree)(info.cast());
                idle
            }
        })
    }
}

/// Wayland has no portable idle query; GNOME's compositor answers over D-Bus.
#[cfg(all(unix, not(target_os = "macos")))]
mod wayland {
    use std::process::{Command, Stdio};

    pub fn idle_ms() -> Option<u64> {
        let output = Command::new("gdbus")
            .args([
                "call",
                "--session",
                "--dest",
                "org.gnome.Mutter.IdleMonitor",
                "--object-path",
                "/org/gnome/Mutter/IdleMonitor/Core",
                "--method",
                "org.gnome.Mutter.IdleMonitor.GetIdletime",
            ])
            .stderr(Stdio::null())
            .output()
            .ok()?;
        output
            .status
            .success()
            .then(|| parse_idletime(&String::from_utf8_lossy(&output.stdout)))
            .flatten()
    }

    /// `(uint64 4200,)`
    fn parse_idletime(reply: &str) -> Option<u64> {
        reply
            .split("uint64")
            .nth(1)?
            .trim_start()
            .split(|c: char| !c.is_ascii_digit())
            .next()?
            .parse()
            .ok()
    }

    #[cfg(test)]
    mod tests {
        use super::parse_idletime;

        #[test]
        fn the_idle_time_is_read_from_gdbus_output() {
            assert_eq!(parse_idletime("(uint64 4200,)\n"), Some(4200));
            assert_eq!(parse_idletime("(uint64 0,)"), Some(0));
            assert_eq!(parse_idletime("Error: no such service"), None);
        }
    }
}
