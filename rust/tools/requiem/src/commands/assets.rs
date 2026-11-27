use requiem_asset_build::discord::BuildInfo;
use requiem_assets::resolve::Build;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use requiem_asset_build::cache::Cache;
use requiem_asset_build::compose::{BuildSelector, ClientSelector, Composition, compose_manifest};
use requiem_asset_build::health::{check_urls, manifest_urls};
use requiem_assets::manifest::Manifest;

use crate::util::{DIM, RESET, bad, hint, ok, step, warn};
use requiem_asset_build::Error;
use std::fs;

fn manifest_path(root: &Path, given: Option<PathBuf>) -> PathBuf {
    given.unwrap_or_else(|| root.join("assets").join("manifest.json"))
}

fn load(path: &Path) -> Result<Manifest> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&text)
        .with_context(|| format!("{} is not a valid manifest", path.display()))
}

#[allow(clippy::too_many_arguments)]
pub async fn resolve(
    root: &Path,
    base: Option<PathBuf>,
    fresh: bool,
    out: Option<PathBuf>,
    build: String,
    client: String,
    refresh: Vec<String>,
    cache_dir: Option<PathBuf>,
) -> Result<i32> {
    let out = manifest_path(root, out);
    let base_path = manifest_path(root, base);
    let base = if fresh || !base_path.is_file() {
        None
    } else {
        Some(load(&base_path)?)
    };
    match &base {
        Some(base) => step(&format!(
            "Building on {} (client build {})",
            base_path.display(),
            base.client
                .build_number
                .map_or("?".into(), |n| n.to_string())
        )),
        None => step("Resolving every recipe from scratch"),
    }

    let cache = Arc::new(Cache::new(
        cache_dir.unwrap_or_else(|| root.join(".cache").join("discord-builds")),
    )?);
    let composition = Composition {
        assets: BuildSelector::parse(&build),
        client: ClientSelector::parse(&client),
        refresh,
    };
    let progress = |message: &str| println!("  {DIM}{message}{RESET}");

    let (manifest, summary) = match compose_manifest(&cache, base.as_ref(), &composition, &progress)
        .await
    {
        Ok(done) => done,
        Err(Error::Unresolved(failures)) => {
            bad(&format!(
                "{} recipe(s) do not resolve against this build:",
                failures.len()
            ));
            for (name, why) in failures {
                println!("      {name}: {why}");
            }
            hint(
                "fix the recipe in crates/requiem-assets/src/recipes.rs, or pick another build with --build",
            );
            return Ok(1);
        }
        Err(e) => return Err(e.into()),
    };
    if let Some(downloads) = &summary.downloads {
        if !downloads.missing.is_empty() {
            warn(&format!(
                "{} chunk(s) could not be downloaded (recipes that needed them would have failed)",
                downloads.missing.len()
            ));
        }
    }

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out, serde_json::to_string_pretty(&manifest)? + "\n")
        .with_context(|| format!("writing {}", out.display()))?;

    ok(&format!("wrote {}", out.display()));
    if summary.resolved.is_empty() {
        println!(
            "    nothing new to resolve; {} entries unchanged",
            summary.carried_over
        );
    } else {
        println!(
            "    resolved {} recipe(s), carried over {} entries",
            summary.resolved.len(),
            summary.carried_over
        );
        for name in summary.resolved.iter().take(12) {
            println!("      + {name}");
        }
        if summary.resolved.len() > 12 {
            println!("      … and {} more", summary.resolved.len() - 12);
        }
    }
    println!(
        "    client build {} ({})",
        manifest
            .client
            .build_number
            .map_or("?".into(), |n| n.to_string()),
        manifest.client.release_channel
    );
    hint(
        "rebuild the app to embed it; run `cargo requiem assets check` to verify the CDN files are still there",
    );
    Ok(0)
}

pub fn show(root: &Path, path: Option<PathBuf>) -> Result<i32> {
    let path = manifest_path(root, path);
    let manifest = load(&path)?;
    step(&format!("{}", path.display()));
    println!(
        "  client   build {} · {} · {}",
        manifest
            .client
            .build_number
            .map_or("?".into(), |n| n.to_string()),
        manifest.client.release_channel,
        &manifest.client.build_id[..manifest.client.build_id.len().min(12)]
    );
    println!(
        "  builds   {}",
        manifest
            .builds
            .iter()
            .map(|b| b.build_number.map_or("?".into(), |n| format!("#{n}")))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "  chunks   {} ({} MB)",
        manifest.chunks.len(),
        manifest.chunks.iter().map(|c| c.size).sum::<u64>() / 1_000_000
    );

    let mut groups: BTreeMap<&str, BTreeMap<u32, usize>> = BTreeMap::new();
    for entry in &manifest.entries {
        *groups
            .entry(entry.name.split('/').next().unwrap_or(""))
            .or_default()
            .entry(entry.build)
            .or_default() += 1;
    }
    println!("  entries  {}", manifest.entries.len());
    for (group, by_build) in groups {
        let total: usize = by_build.values().sum();
        let origin = by_build
            .iter()
            .map(|(b, n)| {
                format!(
                    "#{} x{n}",
                    manifest
                        .builds
                        .get(*b as usize)
                        .and_then(|b| b.build_number)
                        .map_or("?".into(), |n| n.to_string())
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        println!("    {group:<14} {total:>4}  {DIM}{origin}{RESET}");
    }
    Ok(0)
}

pub async fn check(root: &Path, path: Option<PathBuf>) -> Result<i32> {
    let path = manifest_path(root, path);
    let manifest = load(&path)?;
    let urls = manifest_urls(&manifest);
    step(&format!(
        "Checking {} URLs from {}",
        urls.len(),
        path.display()
    ));
    let results = check_urls(urls).await?;
    let problems: Vec<_> = results.iter().filter(|r| !r.ok).collect();
    if problems.is_empty() {
        ok(&format!("all {} URLs are reachable", results.len()));
        return Ok(0);
    }
    bad(&format!(
        "{} of {} URLs have a problem:",
        problems.len(),
        results.len()
    ));
    for p in &problems {
        println!("      {} — {}", p.url, p.detail);
    }
    if problems.len() == results.len() {
        bail!("every URL failed; is this machine online?");
    }
    Ok(1)
}

fn load_cached_build(root: &Path, cache_dir: Option<PathBuf>) -> Result<Build> {
    let cache_dir = cache_dir.unwrap_or_else(|| root.join(".cache").join("discord-builds"));
    let newest = fs::read_dir(cache_dir.join("builds"))
        .with_context(|| {
            format!(
                "no cached build in {} (run `cargo requiem assets resolve` first)",
                cache_dir.display()
            )
        })?
        .flatten()
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())
        .context("no cached build")?;
    let info: BuildInfo = serde_json::from_slice(&fs::read(newest.path())?)?;
    step(&format!(
        "Indexing build #{} ({} chunks)",
        info.build_number.map_or("?".into(), |n| n.to_string()),
        info.chunks.len()
    ));
    let mut build = Build::default();
    for chunk in &info.chunks {
        let Ok(bytes) = fs::read(cache_dir.join("files").join(&chunk.name)) else {
            continue;
        };
        let url = format!("https://discord.com/assets/{}", chunk.name);
        let _ = if chunk.is_css() {
            build.add_css(&url, &bytes)
        } else {
            build.add_chunk(&url, &bytes)
        };
    }
    Ok(build)
}

pub fn svg(root: &Path, export: String, cache_dir: Option<PathBuf>) -> Result<i32> {
    let build = load_cached_build(root, cache_dir)?;
    match build.icon_svg(&export) {
        Ok(svg) => {
            println!("{svg}");
            Ok(0)
        }
        Err(why) => {
            bad(&why);
            Ok(1)
        }
    }
}

pub fn inventory(root: &Path, cache_dir: Option<PathBuf>, section: Option<String>) -> Result<i32> {
    let build = load_cached_build(root, cache_dir)?;
    let inv = build.inventory();
    let want = |s: &str| section.as_deref().is_none_or(|w| w == s);
    let ext_known = |e: &str| {
        matches!(
            e,
            "svg"
                | "png"
                | "mp3"
                | "jpg"
                | "jpeg"
                | "gif"
                | "webp"
                | "webm"
                | "mp4"
                | "wasm"
                | "scm"
                | "json"
        )
    };
    println!(
        "icons: {} whole SVGs, {} unreadable  (cargo requiem assets inventory icons)",
        inv.icons.len(),
        inv.icon_failures.len()
    );
    if want("icons") && section.is_some() {
        for n in &inv.icons {
            println!("  {n}");
        }
    }
    if section.as_deref() == Some("icons") {
        for (name, why) in &inv.icon_failures {
            println!("  ✗ {name}: {why}");
        }
    }
    println!("lottie: {} embedded animations", inv.lottie.len());
    if want("lottie") && section.is_some() {
        for n in &inv.lottie {
            println!("  {n}");
        }
    }
    println!("fonts: {} families", inv.fonts.len());
    if want("fonts") && section.is_some() {
        for (family, weights) in &inv.fonts {
            println!("  {family}: {}", weights.join(", "));
        }
    }
    for (ext, files) in &inv.files {
        let named = files.iter().filter(|(n, _)| n.is_some()).count();
        println!("{ext}: {} asset modules ({named} named)", files.len());
        if section.as_deref() == Some(ext.as_str())
            || (section.as_deref() == Some("files") && ext_known(ext))
        {
            for (name, file) in files {
                println!("  {file}  {}", name.as_deref().unwrap_or(""));
            }
        }
    }
    if section.is_none() {
        println!(
            "
List one section: cargo requiem assets inventory <icons|lottie|fonts|svg|png|mp3|...>"
        );
    }
    Ok(0)
}

pub fn find(root: &Path, d_prefix: String, cache_dir: Option<PathBuf>) -> Result<i32> {
    let build = load_cached_build(root, cache_dir)?;
    let found = build.find_svg(&d_prefix);
    if found.is_empty() {
        bad("no component has a path starting with that data");
        return Ok(1);
    }
    for f in found {
        println!(
            "  module {} function {}  viewBox {}",
            f.module_id, f.function, f.icon.view_box
        );
        for (i, p) in f.icon.paths.iter().enumerate() {
            println!(
                "    path {i}: {}{}",
                &p.d[..p.d.len().min(60)],
                if p.d.len() > 60 { "…" } else { "" }
            );
        }
    }
    Ok(0)
}
