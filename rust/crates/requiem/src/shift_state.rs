pub fn shift_down(fallback: bool) -> bool {
    imp::shift_down().unwrap_or(fallback)
}

#[cfg(windows)]
mod imp {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_SHIFT;

    pub fn shift_down() -> Option<bool> {
        // SAFETY: GetAsyncKeyState has no preconditions.
        Some(unsafe { GetAsyncKeyState(i32::from(VK_SHIFT)) } < 0)
    }
}

#[cfg(target_os = "macos")]
mod imp {
    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGEventSourceFlagsState(state_id: i32) -> u64;
    }

    const COMBINED_SESSION_STATE: i32 = 0;

    const FLAG_MASK_SHIFT: u64 = 0x0002_0000;

    pub fn shift_down() -> Option<bool> {
        // SAFETY: plain CoreGraphics query with no pointer arguments.
        Some(unsafe { CGEventSourceFlagsState(COMBINED_SESSION_STATE) } & FLAG_MASK_SHIFT != 0)
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod imp {
    use std::cell::RefCell;
    use std::os::raw::{c_int, c_uint};
    use std::ptr;

    use x11_dl::xlib::{Display, ShiftMask, Xlib};

    struct X11 {
        xlib: Xlib,
        display: *mut Display,
    }

    impl Drop for X11 {
        fn drop(&mut self) {
            // SAFETY: `display` was returned non-null by XOpenDisplay, is owned by this struct, and
            // is closed exactly once here.
            unsafe { (self.xlib.XCloseDisplay)(self.display) };
        }
    }

    thread_local! {

        static CONNECTION: RefCell<Option<Option<X11>>> = const { RefCell::new(None) };
    }

    fn on_native_wayland() -> bool {
        std::env::var_os("WAYLAND_DISPLAY").is_some_and(|v| !v.is_empty())
            && std::env::var("WINIT_UNIX_BACKEND").map_or(true, |b| b != "x11")
    }

    pub fn shift_down() -> Option<bool> {
        if on_native_wayland() {
            return None;
        }
        CONNECTION.with(|slot| {
            let mut slot = slot.borrow_mut();
            let connection = slot.get_or_insert_with(|| {
                let xlib = Xlib::open().ok()?;
                // SAFETY: a null name makes Xlib use $DISPLAY; the result is null-checked before it
                // is used.
                let display = unsafe { (xlib.XOpenDisplay)(ptr::null()) };
                (!display.is_null()).then_some(X11 { xlib, display })
            });
            let x11 = connection.as_ref()?;
            // SAFETY: `x11.display` is the open connection owned by `X11`, `root` comes from that
            // display, and every out-parameter is a valid local.
            unsafe {
                let root = (x11.xlib.XDefaultRootWindow)(x11.display);
                let (mut root_return, mut child_return) = (0, 0);
                let (mut root_x, mut root_y, mut win_x, mut win_y): (c_int, c_int, c_int, c_int) =
                    (0, 0, 0, 0);
                let mut mask: c_uint = 0;
                let ok = (x11.xlib.XQueryPointer)(
                    x11.display,
                    root,
                    &mut root_return,
                    &mut child_return,
                    &mut root_x,
                    &mut root_y,
                    &mut win_x,
                    &mut win_y,
                    &mut mask,
                );
                (ok != 0).then_some(mask & ShiftMask != 0)
            }
        })
    }
}

#[cfg(not(any(windows, unix)))]
mod imp {
    pub fn shift_down() -> Option<bool> {
        None
    }
}

#[cfg(all(test, unix, not(target_os = "macos")))]
mod tests {
    use std::os::raw::c_ulong;
    use std::ptr;

    use x11_dl::keysym::XK_Shift_L;
    use x11_dl::xlib::Xlib;

    #[test]
    fn x11_reports_a_faked_shift_press() {
        if std::env::var_os("DISPLAY").is_none()
            || std::env::var_os("WAYLAND_DISPLAY").is_some_and(|v| !v.is_empty())
        {
            return;
        }
        let xlib = Xlib::open().expect("libX11");
        let xtest = x11_dl::xtest::Xf86vmode::open().expect("libXtst");
        // SAFETY: the test opens its own display, checks it for null and closes it at the end; the
        // XTest calls use that same display and a keycode obtained from it.
        unsafe {
            let display = (xlib.XOpenDisplay)(ptr::null());
            assert!(!display.is_null());
            let keycode = (xlib.XKeysymToKeycode)(display, XK_Shift_L as c_ulong);
            assert_eq!(super::imp::shift_down(), Some(false));
            (xtest.XTestFakeKeyEvent)(display, keycode as u32, 1, 0);
            (xlib.XSync)(display, 0);
            assert_eq!(super::imp::shift_down(), Some(true));
            (xtest.XTestFakeKeyEvent)(display, keycode as u32, 0, 0);
            (xlib.XSync)(display, 0);
            assert_eq!(super::imp::shift_down(), Some(false));
            (xlib.XCloseDisplay)(display);
        }
    }
}
