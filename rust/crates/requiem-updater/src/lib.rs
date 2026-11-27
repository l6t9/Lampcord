pub mod apply;
pub mod check;
pub mod custom;
pub mod download;
pub mod resume;
pub mod run;
pub mod sources;
pub mod staged;

pub use requiem_update::{KIND_IDENTITY, KIND_UPDATE, KeyEntry, fingerprint};

pub fn current_target() -> &'static str {
    if cfg!(all(windows, target_arch = "x86_64")) {
        "windows-x64"
    } else if cfg!(all(windows, target_arch = "aarch64")) {
        "windows-arm64"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux-x64"
    }
}
