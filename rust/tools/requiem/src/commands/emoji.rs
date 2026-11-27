use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use image::{RgbaImage, imageops};
use serde_json::{Value, json};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::util::{DIM, RESET, hint, ok, step, warn};
use imageops::FilterType;
use reqwest::Client;
use reqwest::StatusCode;
use std::fs;
use std::time::Duration;
use tokio::task;
use webp::Encoder;

const DATA: &str = "https://bignutty.gitlab.io/emojipedia-data";
const TILE: u32 = 64;
const PARALLEL: usize = 32;
const WEBP_QUALITY: f32 = 85.0;

fn default_out(root: &Path) -> PathBuf {
    root.join("crates")
        .join("requiem-core")
        .join("assets")
        .join("emoji-packs")
}

fn client() -> Result<Client> {
    Ok(Client::builder()
        .user_agent("requiem-emoji-builder")
        .timeout(Duration::from_secs(60))
        .build()?)
}

async fn fetch(client: &Client, url: &str) -> Option<Vec<u8>> {
    for _ in 0..3 {
        match client.get(url).send().await {
            Ok(response) if response.status() == StatusCode::NOT_FOUND => return None,
            Ok(response) if response.status().is_success() => {
                if let Ok(bytes) = response.bytes().await {
                    return Some(bytes.to_vec());
                }
            }
            _ => {}
        }
    }
    None
}

async fn fetch_json(client: &Client, path: &str) -> Result<Value> {
    let url = format!("{DATA}/{path}");
    let bytes = fetch(client, &url)
        .await
        .with_context(|| format!("fetching {url}"))?;
    serde_json::from_slice(&bytes).with_context(|| format!("{url} is not JSON"))
}

async fn vendor_slugs(client: &Client) -> Result<Vec<String>> {
    let list = fetch_json(client, "vendors.json").await?;
    Ok(list
        .as_array()
        .context("vendors.json is not a list")?
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect())
}

fn emoji_of(key: &str) -> Option<String> {
    key.split('-')
        .map(|hex| u32::from_str_radix(hex, 16).ok().and_then(char::from_u32))
        .collect()
}

fn make_tile(fetched: &[u8]) -> Option<Vec<u8>> {
    let image: RgbaImage = image::load_from_memory(fetched).ok()?.to_rgba8();
    if image.width() == 0 || image.height() == 0 {
        return None;
    }
    let side = image.width().max(image.height());
    let mut square = RgbaImage::new(side, side);
    imageops::overlay(
        &mut square,
        &image,
        ((side - image.width()) / 2) as i64,
        ((side - image.height()) / 2) as i64,
    );
    let tile = imageops::resize(&square, TILE, TILE, FilterType::Lanczos3);
    Some(
        Encoder::from_rgba(tile.as_raw(), TILE, TILE)
            .encode(WEBP_QUALITY)
            .to_vec(),
    )
}

pub async fn vendors() -> Result<i32> {
    let client = client()?;
    for slug in vendor_slugs(&client).await? {
        let latest = fetch_json(&client, &format!("vendors/{slug}/latest.json")).await?;
        println!(
            "  {slug:28} {:5} emoji",
            latest.as_object().map_or(0, |m| m.len())
        );
    }
    Ok(0)
}

const OFFERED: [&str; 3] = ["apple", "microsoft-3D-fluent", "openmoji"];

pub async fn manifest(root: &Path, out: Option<PathBuf>, all: bool) -> Result<i32> {
    let out = out.unwrap_or_else(|| default_out(root));
    let client = client()?;
    let mut slugs = vendor_slugs(&client).await?;
    if !all {
        slugs.retain(|slug| OFFERED.contains(&slug.as_str()));
    }
    step(&format!("Reading {} vendors", slugs.len()));
    let mut jobs = JoinSet::new();
    let permits = Arc::new(Semaphore::new(8));
    for (n, slug) in slugs.into_iter().enumerate() {
        let (client, permits) = (client.clone(), permits.clone());
        jobs.spawn(async move {
            let _permit = permits.acquire().await.ok()?;
            let info = fetch_json(&client, &format!("vendors/{slug}.json"))
                .await
                .ok()?;
            let latest = fetch_json(&client, &format!("vendors/{slug}/latest.json"))
                .await
                .ok()?;
            let title = info
                .pointer("/vendor/title")
                .and_then(Value::as_str)
                .unwrap_or(&slug)
                .to_string();
            let count = latest.as_object().map_or(0, |m| m.len());
            let preview = latest
                .get("1f600")
                .and_then(Value::as_str)
                .map(str::to_string);
            Some((
                n,
                json!({ "slug": slug, "title": title, "emoji": count, "preview": preview }),
            ))
        });
    }
    let mut entries = Vec::new();
    while let Some(result) = jobs.join_next().await {
        if let Ok(Some(entry)) = result {
            entries.push(entry);
        }
    }
    entries.sort_by_key(|(n, _)| *n);
    let list: Vec<Value> = entries.into_iter().map(|(_, v)| v).collect();
    fs::create_dir_all(&out)?;
    let path = out.join("vendors.json");
    fs::write(&path, serde_json::to_string_pretty(&list)? + "\n")?;
    ok(&format!("{} ({} vendors)", path.display(), list.len()));
    Ok(0)
}

async fn build_vendor(out_dir: &Path, client: &Client, slug: &str) -> Result<()> {
    let latest = fetch_json(client, &format!("vendors/{slug}/latest.json")).await?;
    let mut images: Vec<(String, String)> = latest
        .as_object()
        .with_context(|| format!("{slug}: latest.json is not a map"))?
        .iter()
        .filter_map(|(key, url)| Some((emoji_of(key)?, url.as_str()?.to_string())))
        .collect();

    images.sort();
    step(&format!("{slug}: fetching {} images", images.len()));
    let permits = Arc::new(Semaphore::new(PARALLEL));
    let mut jobs = JoinSet::new();
    let total = images.len();
    for (n, (emoji, url)) in images.into_iter().enumerate() {
        let (client, permits) = (client.clone(), permits.clone());
        jobs.spawn(async move {
            let _permit = permits.acquire().await.ok()?;
            let bytes = fetch(&client, &url).await?;

            let tile = task::spawn_blocking(move || make_tile(&bytes))
                .await
                .ok()
                .flatten()?;
            Some((n, emoji, tile))
        });
    }
    let mut found = Vec::new();
    let mut done = 0usize;
    while let Some(result) = jobs.join_next().await {
        done += 1;
        if done.is_multiple_of(500) {
            println!("  {DIM}{done} / {total}{RESET}");
        }
        if let Ok(Some(entry)) = result {
            found.push(entry);
        }
    }
    found.sort_by_key(|(n, ..)| *n);
    if found.len() * 2 < total {
        bail!(
            "{slug}: only {} of {total} images came down: is the network or the CDN down?",
            found.len()
        );
    }
    let missing = total - found.len();
    let entries: Vec<(String, Vec<u8>)> = found
        .into_iter()
        .map(|(_, emoji, tile)| (emoji, tile))
        .collect();
    let bytes = requiem_emoji::write(&entries);
    fs::create_dir_all(out_dir)?;
    let path = out_dir.join(format!("{slug}.pack"));
    fs::write(&path, &bytes).with_context(|| format!("writing {}", path.display()))?;
    ok(&format!(
        "{} ({:.1} MB, {} emoji)",
        path.display(),
        bytes.len() as f64 / 1e6,
        entries.len()
    ));
    if missing > 0 {
        warn(&format!(
            "{missing} images could not be fetched or decoded (the app draws Twemoji for those emoji)"
        ));
    }
    Ok(())
}

pub async fn build(root: &Path, which: &str, out: Option<PathBuf>, force: bool) -> Result<i32> {
    let out = out.unwrap_or_else(|| default_out(root));
    let client = client()?;
    let all = vendor_slugs(&client).await?;
    let everything = which == "all";
    let slugs: Vec<String> = if everything {
        all
    } else if all.iter().any(|s| s == which) {
        vec![which.to_string()]
    } else {
        bail!("unknown vendor `{which}`; `cargo requiem emoji vendors` lists them");
    };
    let mut failed = Vec::new();
    for slug in &slugs {
        if everything && !force && out.join(format!("{slug}.pack")).is_file() {
            println!("  {DIM}{slug}: already built{RESET}");
            continue;
        }
        match build_vendor(&out, &client, slug).await {
            Ok(()) => {}
            Err(err) if everything => {
                warn(&format!("{slug}: {err:#}"));
                failed.push(slug.clone());
            }
            Err(err) => return Err(err),
        }
    }
    if !failed.is_empty() {
        warn(&format!(
            "{} vendor(s) failed: {}",
            failed.len(),
            failed.join(", ")
        ));
        return Ok(1);
    }
    hint("`cargo requiem emoji manifest` refreshes the list of vendors the app offers");
    Ok(0)
}
