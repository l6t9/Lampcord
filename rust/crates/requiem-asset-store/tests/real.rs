use std::env;
use std::fs;
use std::path::PathBuf;

use requiem_asset_store::{AssetStore, ManifestSource};
use requiem_assets::manifest::Manifest;

fn load() -> Option<AssetStore> {
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(env::var("ASSET_STORE_MANIFEST").ok()?).ok()?)
            .ok()?;
    let dir = PathBuf::from(env::var("ASSET_STORE_CHUNK_DIR").ok()?);
    let chunks: Vec<Vec<u8>> = manifest
        .chunks
        .iter()
        .map(|c| {
            fs::read(dir.join(c.url.rsplit('/').next().unwrap()))
                .unwrap_or_else(|e| panic!("{}: {e}", c.url))
        })
        .collect();
    Some(
        AssetStore::from_chunks(
            manifest,
            ManifestSource::Cache,
            &chunks,
            env::temp_dir().join("requiem-asset-store-real"),
        )
        .unwrap(),
    )
}

#[test]
fn extracts_every_icon_and_lottie_in_the_manifest() {
    let Some(store) = load() else { return };
    eprintln!("client: {:?}", store.client_info());
    for (name, why) in store.problems() {
        eprintln!("  problem {name}: {why}");
    }
    assert!(store.problems().is_empty());
    let check = store.svg("icon/CheckmarkSmallBoldIcon").unwrap();
    assert!(
        check.starts_with("<svg") && check.contains("M19.06 6.94"),
        "{check}"
    );

    let circle = store.svg("icon/CircleCheckIcon").unwrap();
    assert!(
        circle.contains(r#"<circle cx="12" cy="12" r="10" fill="transparent"/>"#),
        "{circle}"
    );
    for name in [
        "lottie/mic",
        "lottie/deafen",
        "lottie/gif_button",
        "lottie/settings",
    ] {
        let json: serde_json::Value = serde_json::from_str(store.lottie(name).unwrap()).unwrap();
        assert!(json["layers"].is_array(), "{name}");
    }
    assert!(store.has_file("font/ggsansmedium") && store.has_file("connection/steam"));
}

#[test]
fn real_woff2_converts_to_a_usable_truetype_font() {
    let Ok(path) = env::var("ASSET_STORE_WOFF2") else {
        return;
    };
    let ttf = requiem_asset_store::woff2_to_ttf(&fs::read(path).unwrap()).unwrap();

    assert!(
        ttf.starts_with(&[0, 1, 0, 0]) || ttf.starts_with(b"OTTO"),
        "{:?}",
        &ttf[..4]
    );

    if let Ok(reference) = env::var("ASSET_REFERENCE_TTF") {
        let bundled = fs::read(reference).unwrap();
        eprintln!(
            "converted {} bytes (bundled TTF: {} bytes), byte-identical: {}",
            ttf.len(),
            bundled.len(),
            ttf == bundled
        );
    }
}

#[tokio::test]
async fn all_fonts_match_the_formerly_bundled_ttfs() {
    let (Some(store), Ok(dir)) = (load(), env::var("ASSET_REFERENCE_FONTS")) else {
        return;
    };
    let client = wreq::Client::new();
    let names: Vec<String> = store
        .manifest()
        .entries
        .iter()
        .map(|e| e.name.clone())
        .filter(|n| n.starts_with("font/"))
        .collect();
    assert_eq!(names.len(), 12);
    let mut different = Vec::new();
    for name in &names {
        let converted = store.font_ttf(&client, name).await.unwrap();
        let reference = fs::read(
            PathBuf::from(&dir).join(format!("{}.ttf", name.trim_start_matches("font/"))),
        )
        .unwrap();
        if converted != reference {
            different.push(name.clone());
        }
    }
    eprintln!("fonts differing from the bundled TTFs: {different:?}");
    assert!(different.is_empty());
}

#[test]
fn dump_lottie() {
    let (Some(store), Ok(dir)) = (load(), env::var("ASSET_STORE_DUMP_LOTTIE")) else {
        return;
    };
    fs::create_dir_all(&dir).unwrap();
    for entry in &store.manifest().entries {
        if let Some(stem) = entry.name.strip_prefix("lottie/") {
            fs::write(
                PathBuf::from(&dir).join(format!("{stem}.json")),
                store.lottie(&entry.name).unwrap(),
            )
            .unwrap();
        }
    }
}
