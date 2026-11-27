use crate::sounds::Sound;
use crate::sounds;
use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, Once};
use std::thread::Builder;
use std::thread;
use std::time::{Duration, Instant};

use crate::utils::voice_settings::{self, InputMode};

static ACTIVE: AtomicBool = AtomicBool::new(false);
static IN_CALL: AtomicBool = AtomicBool::new(false);
static RECORDING: Mutex<Option<Recording>> = Mutex::new(None);
static START: Once = Once::new();

#[derive(Default)]
struct Recording {
    keys: Vec<u32>,
    done: Option<Vec<u32>>,
}

pub fn active() -> bool {
    ensure_thread();
    ACTIVE.load(Ordering::Relaxed)
}

pub(super) fn set_in_call(on: bool) {
    IN_CALL.store(on, Ordering::Relaxed);
    ensure_thread();
}

pub fn start_recording() {
    *RECORDING.lock().expect("ptt recording poisoned") = Some(Recording::default());
    ensure_thread();
}

pub fn stop_recording() {
    *RECORDING.lock().expect("ptt recording poisoned") = None;
}

pub fn is_recording() -> bool {
    RECORDING
        .lock()
        .expect("ptt recording poisoned")
        .as_ref()
        .is_some_and(|r| r.done.is_none())
}

pub fn recording_keys() -> Vec<u32> {
    RECORDING
        .lock()
        .expect("ptt recording poisoned")
        .as_ref()
        .map(|r| r.keys.clone())
        .unwrap_or_default()
}

pub fn take_recorded() -> Option<Vec<u32>> {
    let mut slot = RECORDING.lock().expect("ptt recording poisoned");
    let done = slot.as_mut()?.done.take()?;
    *slot = None;
    Some(done)
}

fn ensure_thread() {
    START.call_once(|| {
        let _ = Builder::new()
            .name("voice-ptt".into())
            .spawn(run);
    });
}

fn run() {
    let mut held = false;
    let mut released_at: Option<Instant> = None;
    loop {
        let recording = RECORDING
            .lock()
            .expect("ptt recording poisoned")
            .as_ref()
            .is_some_and(|r| r.done.is_none());
        if recording {
            record_step();
            thread::sleep(Duration::from_millis(5));
            continue;
        }
        let settings = voice_settings::effective();
        let wanted = settings.mode == InputMode::PushToTalk
            && !settings.mode_options.shortcut.is_empty()
            && super::engine::measuring();
        if !wanted {
            if ACTIVE.swap(false, Ordering::Relaxed) && IN_CALL.load(Ordering::Relaxed) {
                sounds::play(Sound::PttStop);
            }
            held = false;
            released_at = None;
            thread::sleep(Duration::from_millis(50));
            continue;
        }
        let down = settings
            .mode_options
            .shortcut
            .iter()
            .all(|&vk| keys::is_down(vk));
        if down && !held {
            held = true;
            released_at = None;
            if !ACTIVE.swap(true, Ordering::Relaxed) && IN_CALL.load(Ordering::Relaxed) {
                sounds::play(Sound::PttStart);
            }
        } else if !down && held {
            held = false;
            released_at = Some(Instant::now());
        }
        if let Some(at) = released_at {
            if at.elapsed() >= Duration::from_millis(u64::from(settings.mode_options.delay)) {
                released_at = None;
                if ACTIVE.swap(false, Ordering::Relaxed) && IN_CALL.load(Ordering::Relaxed) {
                    sounds::play(Sound::PttStop);
                }
            }
        }
        thread::sleep(Duration::from_millis(4));
    }
}

fn record_step() {
    let down = keys::pressed();
    let mut slot = RECORDING.lock().expect("ptt recording poisoned");
    let Some(recording) = slot.as_mut() else {
        return;
    };
    for vk in &down {
        if !recording.keys.contains(vk) {
            recording.keys.push(*vk);
        }
    }
    if down.is_empty() && !recording.keys.is_empty() {
        recording.done = Some(mem::take(&mut recording.keys));
    }
}

pub fn shortcut_label(shortcut: &[u32]) -> String {
    shortcut
        .iter()
        .map(|&vk| keys::name(vk))
        .collect::<Vec<_>>()
        .join(" + ")
}

#[cfg(windows)]
mod keys {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, GetKeyNameTextW, MAPVK_VK_TO_VSC, MapVirtualKeyW,
    };

    pub fn is_down(vk: u32) -> bool {
        // SAFETY: GetAsyncKeyState has no preconditions.
        unsafe { GetAsyncKeyState(vk as i32) as u16 & 0x8000 != 0 }
    }

    /// Every recordable key and button that is down right now.
    pub fn pressed() -> Vec<u32> {
        (3..=0xFE)
            .filter(|&vk| recordable(vk) && is_down(vk))
            .collect()
    }

    pub fn recordable(vk: u32) -> bool {
        !matches!(vk, 0x01 | 0x02 | 0x10 | 0x11 | 0x12)
    }

    pub fn name(vk: u32) -> String {
        match vk {
            0x04 => return "MOUSE 3".into(),
            0x05 => return "MOUSE 4".into(),
            0x06 => return "MOUSE 5".into(),
            0xA0 | 0xA1 => return "SHIFT".into(),
            0xA2 | 0xA3 => return "CTRL".into(),
            0xA4 | 0xA5 => return "ALT".into(),
            0x5B | 0x5C => return "META".into(),
            _ => {}
        }

        // SAFETY: plain Win32 calls, and GetKeyNameTextW writes into a local buffer of the length
        // it is given.
        unsafe {
            let scan = MapVirtualKeyW(vk, MAPVK_VK_TO_VSC);

            let extended = matches!(vk, 0x21..=0x2E | 0x6F | 0x90) as i32;
            let mut buffer = [0u16; 64];
            let len = GetKeyNameTextW(((scan as i32) << 16) | (extended << 24), &mut buffer);
            if len > 0 {
                return String::from_utf16_lossy(&buffer[..len as usize]).to_uppercase();
            }
        }
        format!("KEY {vk}")
    }
}

#[cfg(not(windows))]
#[path = "ptt_keys.rs"]
mod keys;
