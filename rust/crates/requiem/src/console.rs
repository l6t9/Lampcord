#[cfg(all(windows, not(debug_assertions)))]
use std::env;
#[cfg(all(windows, not(debug_assertions)))]
use windows_sys::Win32::System::Console;

#[cfg(all(windows, not(debug_assertions)))]
pub fn maybe_attach_console() {
    if env::var_os("REQUIEM_CONSOLE").is_none_or(|v| v.is_empty()) {
        return;
    }

    // SAFETY: AllocConsole has no preconditions; a failure only means the process stays console-less.
    unsafe {
        Console::AllocConsole();
    }
}

#[cfg(not(all(windows, not(debug_assertions))))]
pub fn maybe_attach_console() {}
