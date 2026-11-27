use requiem_assets::extract;
use requiem_assets::extract::extract_svg;
use requiem_assets::manifest::BuildRef;
use requiem_assets::manifest::EntryKind;
use requiem_assets::recipes;
use requiem_assets::resolve::{Build, Recipe, resolve};
use std::collections;
use std::collections::HashMap;
use std::env;
use std::fs;

fn load() -> Option<(Vec<u8>, Build)> {
    let path = env::var("ASSET_RECIPES_WEB_JS").ok()?;
    let bytes = fs::read(path).ok()?;
    let mut build = Build::default();
    build
        .add_chunk("https://discord.com/assets/web.js", &bytes)
        .unwrap();
    Some((bytes, build))
}

#[test]
fn resolves_and_extracts_check_icon() {
    let Some((bytes, build)) = load() else { return };
    eprintln!("modules indexed: {}", build.module_count());
    let recipes = [Recipe::SvgComponent {
        name: "icon/check".into(),
        export: "CheckmarkSmallBoldIcon".into(),
    }];
    let res = resolve(
        &build,
        &recipes,
        &BuildRef::new("test", None),
        "stable",
        "https://discord.com/assets/",
    );
    assert!(res.failures.is_empty(), "{:?}", res.failures);
    let icon = extract_svg(&res.manifest, &res.manifest.entries[0], &bytes).unwrap();
    eprintln!("{icon:?}");
    assert!(icon.paths[0].d.starts_with("M19.06 6.94"));
}

#[test]
fn survey_icon_exports() {
    let Some((_, build)) = load() else { return };
    let names: Vec<String> = build
        .export_names()
        .filter(|n| n.ends_with("Icon"))
        .map(String::from)
        .collect();
    let recipes: Vec<Recipe> = names
        .iter()
        .map(|n| Recipe::SvgComponent {
            name: n.clone(),
            export: n.clone(),
        })
        .collect();
    let res = resolve(
        &build,
        &recipes,
        &BuildRef::new("test", None),
        "stable",
        "https://discord.com/assets/",
    );
    eprintln!(
        "icon exports: {}, resolved: {}, failed: {}",
        names.len(),
        res.manifest.entries.len(),
        res.failures.len()
    );
    let mut reasons = collections::BTreeMap::<String, usize>::new();
    for (_, why) in &res.failures {
        let key = why
            .split('`').rfind(|_| true)
            .unwrap_or(why)
            .trim()
            .chars()
            .take(70)
            .collect::<String>();
        *reasons.entry(key).or_default() += 1;
    }
    for (why, n) in reasons {
        eprintln!("  {n:5}  {why}");
    }
}

#[test]
fn whole_svg_and_fingerprint_recipes() {
    let Some((bytes, build)) = load() else { return };
    let recipes = [
        Recipe::SvgComponent {
            name: "icon/CopyIcon".into(),
            export: "CopyIcon".into(),
        },
        Recipe::SvgComponent {
            name: "icon/CircleCheckIcon".into(),
            export: "CircleCheckIcon".into(),
        },
        Recipe::SvgByPath {
            name: "icon/replay".into(),
            d_prefix: "M12,5 L12,1 L7,6 L12,11".into(),
        },
        Recipe::SvgByPath {
            name: "icon/ambiguous".into(),
            d_prefix: "M12 23a11 11 0 1 0 0-22".into(),
        },
    ];
    let res = resolve(
        &build,
        &recipes,
        &BuildRef::new("test", None),
        "stable",
        "https://discord.com/assets/",
    );
    eprintln!("failures: {:?}", res.failures);
    let svg = |n: &str| {
        let entry = res.manifest.entries.iter().find(|e| e.name == n)?;
        Some(
            extract_svg(&res.manifest, entry, &bytes)
                .unwrap()
                .to_svg()
                .unwrap(),
        )
    };

    let copy = svg("icon/CopyIcon").unwrap();
    assert!(copy.starts_with("<svg"), "{copy}");
    assert_eq!(copy.matches("<path").count(), 3, "{copy}");
    let check = svg("icon/CircleCheckIcon").unwrap();
    assert!(
        check.contains("<circle")
            && check.contains("fill=\"transparent\"")
            && check.contains("fill=\"currentColor\""),
        "{check}"
    );
    assert!(svg("icon/replay").is_some());
    assert!(
        res.failures
            .iter()
            .any(|(n, why)| n == "icon/ambiguous" && why.contains("components match")),
        "{:?}",
        res.failures
    );
}

#[test]
fn font_face_recipes() {
    let (Ok(path), Some(_)) = (env::var("ASSET_RECIPES_FONT_CSS"), load()) else {
        return;
    };
    let mut build = Build::default();
    build
        .add_css(
            "https://discord.com/assets/fonts.css",
            &fs::read(path).unwrap(),
        )
        .unwrap();
    let recipes = [
        Recipe::FontFace {
            name: "font/ggsans-medium".into(),
            family: "gg sans".into(),
            weight: 500,
            italic: false,
        },
        Recipe::FontFace {
            name: "font/ggsans-bold-italic".into(),
            family: "gg sans".into(),
            weight: 700,
            italic: true,
        },
        Recipe::FontFace {
            name: "font/ggmono-bold".into(),
            family: "gg mono".into(),
            weight: 700,
            italic: false,
        },
        Recipe::FontFace {
            name: "font/nope".into(),
            family: "gg sans".into(),
            weight: 100,
            italic: false,
        },
    ];
    let res = resolve(
        &build,
        &recipes,
        &BuildRef::new("test", None),
        "stable",
        "https://discord.com/assets/",
    );
    let url = |n: &str| match &res
        .manifest
        .entries
        .iter()
        .find(|e| e.name == n)
        .unwrap()
        .kind
    {
        EntryKind::File { url, .. } => url.clone(),
        _ => panic!(),
    };
    assert_eq!(
        url("font/ggsans-medium"),
        "https://discord.com/assets/b272b33815319bae.woff2"
    );
    assert_eq!(
        url("font/ggsans-bold-italic"),
        "https://discord.com/assets/ce3b8055f5114434.woff2"
    );
    assert_eq!(
        url("font/ggmono-bold"),
        "https://discord.com/assets/45efa6936fdfb918.woff2"
    );
    assert_eq!(res.failures.len(), 1);
}

#[test]
fn lottie_and_asset_files_from_full_build() {
    let Ok(dir) = env::var("ASSET_RECIPES_CHUNK_DIR") else {
        return;
    };
    let mut build = Build::default();
    let mut bytes_of = HashMap::<String, Vec<u8>>::new();
    for entry in fs::read_dir(&dir).unwrap().flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "js") {
            continue;
        }
        let bytes = fs::read(&path).unwrap();
        let url = format!(
            "https://discord.com/assets/{}",
            path.file_name().unwrap().to_string_lossy()
        );
        if build.add_chunk(&url, &bytes).is_ok() {
            bytes_of.insert(url, bytes);
        }
    }
    eprintln!(
        "modules indexed across all chunks: {}",
        build.module_count()
    );
    let recipes = [
        Recipe::LottieJson {
            name: "lottie/mic".into(),
            nm: "Lottie_HoverOnOff_Microphone".into(),
        },
        Recipe::LottieJson {
            name: "lottie/deafen".into(),
            nm: "Lottie_HoverOnOff_Deafen".into(),
        },
        Recipe::LottieJson {
            name: "lottie/gif".into(),
            nm: "Lottie_HoverOn_GIF".into(),
        },
        Recipe::LottieJson {
            name: "lottie/settings".into(),
            nm: "L".into(),
        },
        Recipe::ContextFile {
            name: "file_icon/acrobat".into(),
            file: "icon-file-acrobat.svg".into(),
        },
    ];
    let res = resolve(
        &build,
        &recipes,
        &BuildRef::new("test", None),
        "stable",
        "https://discord.com/assets/",
    );
    eprintln!("failures: {:?}", res.failures);
    assert!(res.failures.is_empty());
    for entry in &res.manifest.entries {
        match &entry.kind {
            EntryKind::Text { chunk, .. } => {
                let url = &res.manifest.chunks[*chunk as usize].url;
                let text = extract::extract_text(&res.manifest, entry, &bytes_of[url]).unwrap();
                let json: serde_json::Value = serde_json::from_str(&text).unwrap();
                eprintln!(
                    "{} -> {} bytes, nm={}, layers={}",
                    entry.name,
                    text.len(),
                    json["nm"],
                    json["layers"].as_array().unwrap().len()
                );
            }
            other => eprintln!("{} -> {other:?}", entry.name),
        }
    }
}

fn load_full_build(dir: &str) -> (Build, HashMap<String, Vec<u8>>) {
    let mut build = Build::default();
    let mut bytes_of = HashMap::new();
    let mut files: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    files.sort();
    for path in files {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let bytes = fs::read(&path).unwrap();
        let url = format!(
            "https://discord.com/assets/{}",
            path.file_name().unwrap().to_string_lossy()
        );
        let ok = match ext {
            "js" => build.add_chunk(&url, &bytes).is_ok(),
            "css" => build.add_css(&url, &bytes).is_ok(),
            _ => false,
        };
        if ok {
            bytes_of.insert(url, bytes);
        }
    }
    (build, bytes_of)
}

#[test]
fn full_recipe_list_resolves() {
    let Ok(dir) = env::var("ASSET_RECIPES_CHUNK_DIR") else {
        return;
    };
    let (build, bytes_of) = load_full_build(&dir);
    let recipes = recipes::all();
    let res = resolve(
        &build,
        &recipes,
        &BuildRef::new("test", None),
        "stable",
        "https://discord.com/assets/",
    );
    eprintln!(
        "recipes: {}, entries: {}, failures: {}",
        recipes.len(),
        res.manifest.entries.len(),
        res.failures.len()
    );
    for (name, why) in &res.failures {
        eprintln!("  FAIL {name}: {why}");
    }
    assert!(res.failures.is_empty());

    for entry in &res.manifest.entries {
        let Some(name) = entry.name.strip_prefix("icon/") else {
            continue;
        };
        let EntryKind::SvgComponent { chunk, .. } = &entry.kind else {
            continue;
        };
        let svg = extract_svg(
            &res.manifest,
            entry,
            &bytes_of[&res.manifest.chunks[*chunk as usize].url],
        )
        .unwrap()
        .to_svg()
        .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(
            svg.starts_with("<svg") && svg.contains("viewBox"),
            "{name}: {svg}"
        );
        assert!(
            name == "replay" || name.ends_with("Icon"),
            "`{name}` is not Discord's component name"
        );
    }
}

#[test]
fn app_spinner_video_resolves_to_the_dark_webm() {
    let Ok(dir) = env::var("ASSET_RECIPES_CHUNK_DIR") else {
        return;
    };
    let (build, _) = load_full_build(&dir);
    let recipes = recipes::videos();
    let res = resolve(
        &build,
        &recipes,
        &BuildRef::new("test", None),
        "stable",
        "https://discord.com/assets/",
    );
    assert!(res.failures.is_empty(), "{:?}", res.failures);
    match &res.manifest.entries[0].kind {
        EntryKind::File { url, .. } => {
            assert_eq!(url, "https://discord.com/assets/7ba7fcf2c4710bb7.webm")
        }
        other => panic!("{other:?}"),
    }
}
