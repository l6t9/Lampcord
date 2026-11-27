use requiem_core::assets::AssetsConfig;

pub mod features;
pub mod console;
pub mod notifications;
pub mod shell;
pub mod shift_state;
pub mod shortcuts;
pub mod system_idle;
pub mod uninstall_entry;
pub mod updater;

const EMBEDDED_ASSET_MANIFEST: &str = include_str!("../../../assets/manifest.json");

pub fn assets_config() -> AssetsConfig {
    AssetsConfig::from_env(env!("CARGO_PKG_VERSION"), EMBEDDED_ASSET_MANIFEST)
}
