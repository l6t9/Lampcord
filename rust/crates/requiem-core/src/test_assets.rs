use requiem_ui::icon_handle;
use requiem_ui::icons;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::sync::OnceLock;

use requiem_asset_store::{AssetStore, Manifest, ManifestSource};

const MANIFEST: &str = include_str!("../../../assets/manifest.json");

pub fn install() -> bool {
    static INSTALLED: OnceLock<bool> = OnceLock::new();
    *INSTALLED.get_or_init(|| {
        build()
            .map(|store| requiem_asset_store::install(store).is_ok())
            .unwrap_or(false)
            && {
                icons::install_resolver(|name| {
                    requiem_asset_store::get().and_then(|store| store.text(name))
                });
                icon_handle::install_svg_resolver(|name| {
                    requiem_asset_store::get().and_then(|store| store.svg(name))
                });
                true
            }
    })
}

fn build() -> Option<AssetStore> {
    let cache = env::var_os("REQUIEM_BUILD_CACHE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.cache/discord-builds")
        });
    let files = cache.join("files");
    if !files.is_dir() {
        return None;
    }
    let manifest: Manifest = serde_json::from_str(MANIFEST).ok()?;
    let chunks: Vec<Vec<u8>> = manifest
        .chunks
        .iter()
        .map(|c| fs::read(files.join(c.url.rsplit('/').next()?)).ok())
        .collect::<Option<_>>()?;

    let dir = env::temp_dir().join(format!("requiem-test-assets-{}", process::id()));
    fs::create_dir_all(dir.join("files")).ok()?;
    for entry in &manifest.entries {
        if let requiem_asset_store::EntryKind::File {
            url,
            sha256: Some(sha),
        } = &entry.kind
        {
            let _ = fs::copy(
                files.join(url.rsplit('/').next()?),
                dir.join("files").join(sha),
            );
        }
    }
    AssetStore::from_chunks(manifest, ManifestSource::Embedded, &chunks, dir).ok()
}
