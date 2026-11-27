use crate::net_profile::Resource;
use crate::net_profile;
use futures_util::future;
use requiem_db::paths;
use requiem_ui::fonts::FontFiles;
use requiem_ui::fonts;
use requiem_ui::icon_handle;
use requiem_ui::icons;
use std::env;
use std::path::PathBuf;
use std::time::Instant;

pub use requiem_asset_store::{Progress, Stage};

#[derive(Debug, Clone)]
pub struct AssetsConfig {
    pub embedded_manifest: Option<&'static str>,

    pub manifest_url: Option<String>,

    pub app_version: String,

    pub cache_dir: PathBuf,
}

impl AssetsConfig {
    pub fn from_env(app_version: &str, embedded_manifest: &'static str) -> Self {
        let cache_dir = env::var_os("REQUIEM_ASSET_CACHE_DIR")
            .map(PathBuf::from)
            .or_else(Self::default_cache_dir)
            .unwrap_or_else(|| env::temp_dir().join("requiem-assets"));
        Self {
            embedded_manifest: Some(embedded_manifest),
            manifest_url: env::var("REQUIEM_ASSET_MANIFEST_URL")
                .ok()
                .filter(|u| !u.is_empty()),
            app_version: app_version.to_string(),
            cache_dir,
        }
    }

    pub fn default_cache_dir() -> Option<PathBuf> {
        paths::cache_root().map(|dir| dir.join("assets"))
    }
}

impl AssetsConfig {
    pub fn store_config(&self) -> requiem_asset_store::Config {
        requiem_asset_store::Config {
            embedded_manifest: self.embedded_manifest,
            manifest_url: self.manifest_url.clone(),
            app_version: self.app_version.clone(),
            cache_dir: self.cache_dir.clone(),
        }
    }
}

pub const LOGO_ASSET: &str = "video/app_spinner";

pub async fn needs_splash(client: &wreq::Client, config: &AssetsConfig) -> bool {
    match requiem_asset_store::fetch_manifest(client, &config.store_config()).await {
        Ok((manifest, _)) => !requiem_asset_store::cache_ready(&manifest, &config.cache_dir),
        Err(_) => true,
    }
}

fn install_request_hook() {
    requiem_asset_store::set_request_hook(
        |builder, url| match Resource::for_url(url) {
            Some(resource) => net_profile::decorate(builder, url, resource),
            None => builder,
        },
    );
}

pub async fn prefetch(
    client: &wreq::Client,
    config: &AssetsConfig,
    manifest: requiem_asset_store::Manifest,
    source: requiem_asset_store::ManifestSource,
    on_progress: impl Fn(Progress),
) -> Result<(), requiem_asset_store::Error> {
    install_request_hook();
    let store = requiem_asset_store::AssetStore::load_manifest(
        client,
        &config.cache_dir,
        manifest,
        source,
        &on_progress,
    )
    .await?;
    store.warm_files(client, &on_progress).await?;
    store.warm_fonts(client, &on_progress).await
}

#[tracing::instrument(skip(client, config), fields(release = %config.app_version))]
pub async fn init(
    client: &wreq::Client,
    config: &AssetsConfig,
) -> Result<(), requiem_asset_store::Error> {
    let started = Instant::now();
    install_request_hook();
    let store = requiem_asset_store::AssetStore::load(client, &config.store_config()).await?;
    let info = store.client_info();
    log::info!(
        "assets: release {} -> client build {:?} ({}), manifest from {:?}, {} entries, loaded in {:?}",
        config.app_version,
        info.build_number,
        info.release_channel,
        store.manifest_source(),
        store.manifest().entries.len(),
        started.elapsed()
    );
    for (name, why) in store.problems() {
        log::warn!("assets: `{name}` could not be extracted: {why}");
    }
    if requiem_asset_store::install(store).is_err() {
        log::warn!("assets: a store was already installed; keeping it");
    }

    icons::install_resolver(|name| {
        requiem_asset_store::get().and_then(|store| store.text(name))
    });
    icon_handle::install_svg_resolver(|name| {
        requiem_asset_store::get().and_then(|store| store.svg(name))
    });

    if let Some(store) = requiem_asset_store::get() {
        store.warm_files(client, |_| {}).await?;
    }
    install_fonts(client).await
}

async fn install_fonts(client: &wreq::Client) -> Result<(), requiem_asset_store::Error> {
    let Some(store) = requiem_asset_store::get() else {
        return Ok(());
    };
    let names: Vec<&str> = store
        .manifest()
        .entries
        .iter()
        .map(|e| e.name.as_str())
        .filter(|n| n.starts_with("font/"))
        .collect();
    let loaded = future::try_join_all(
        names
            .iter()
            .map(|name| async move { store.font_ttf(client, name).await.map(|ttf| (*name, ttf)) }),
    )
    .await?;
    let mut files = FontFiles::default();
    for (name, ttf) in loaded {
        if name.starts_with("font/ggsansmono") {
            files.gg_sans_mono.push(ttf);
        } else if name.starts_with("font/ggsans") {
            files.gg_sans.push(ttf);
        }
    }
    log::info!(
        "assets: {} gg sans + {} gg sans mono font files",
        files.gg_sans.len(),
        files.gg_sans_mono.len()
    );
    if !fonts::install(files) {
        log::warn!("assets: fonts were already installed; keeping them");
    }
    Ok(())
}

#[cfg(test)]
mod icon_tests {
    use crate::test_assets;
    use requiem_ui::svg_icon;
    use skia_safe::{Color, surfaces};

    #[test]
    fn every_icon_in_the_manifest_draws_something() {
        if !test_assets::install() {
            return;
        }
        let store = requiem_asset_store::get().expect("installed");
        let mut drawn = 0;
        for entry in &store.manifest().entries {
            let Some(name) = entry.name.strip_prefix("icon/") else {
                continue;
            };
            let svg = store
                .svg(&entry.name)
                .unwrap_or_else(|| panic!("{name} has no SVG"));
            let art = svg_icon::parse(svg)
                .unwrap_or_else(|| panic!("{name} does not parse: {svg}"));
            let mut surface = surfaces::raster_n32_premul((48, 48)).expect("surface");
            surface.canvas().clear(Color::TRANSPARENT);
            art.draw_fitted(surface.canvas(), (0.0, 0.0, 48.0, 48.0), Color::WHITE, 1.0);
            let info = skia_safe::ImageInfo::new_n32_premul((48, 48), None);
            let mut pixels = vec![0u8; 48 * 48 * 4];
            assert!(surface.read_pixels(&info, &mut pixels, 48 * 4, (0, 0)));
            let painted = pixels.chunks(4).filter(|p| p[3] > 0).count();
            assert!(painted > 20, "{name} painted only {painted} pixels: {svg}");
            drawn += 1;
        }
        assert!(drawn > 100, "only {drawn} icons");
    }
}
