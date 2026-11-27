use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

use crate::util::{hint, ok, step, warn};
use reqwest::Client;

const SDK_URL: &str = "https://download.steinberg.net/sdk_downloads/ASIO-SDK_2.3.4_2025-10-15.zip";
const SDK_SHA256: &str = "d5ebf0c20dd2c5f43771fd0c1418f4b361bf52434ee670097cfa6b3a335e2eca";
const SDK_LABEL: &str = "Steinberg ASIO SDK 2.3.4";

const REQUIRED: &[&str] = &[
    "common/asio.h",
    "host/asiodrivers.h",
    "host/pc/asiolist.cpp",
];

pub fn present(root: &Path) -> bool {
    REQUIRED
        .iter()
        .all(|rel| root.join("vendor/ASIOSDK").join(rel).is_file())
}

pub async fn ensure(root: &Path) -> Result<()> {
    if !cfg!(windows) {
        return Ok(());
    }
    if present(root) {
        return Ok(());
    }
    step("Fetching the Steinberg ASIO SDK (needed for the `asio` feature)");
    fetch(root).await?;
    Ok(())
}

pub async fn run(root: &Path, force: bool) -> Result<i32> {
    if !cfg!(windows) {
        warn("ASIO is Windows-only; nothing to fetch");
        return Ok(0);
    }
    if present(root) && !force {
        ok(&format!("{SDK_LABEL} is already at vendor/ASIOSDK"));
        hint("pass --force to download it again");
        return Ok(0);
    }
    step(&format!("Fetching {SDK_LABEL}"));
    fetch(root).await?;
    Ok(0)
}

async fn fetch(root: &Path) -> Result<()> {
    let dest = root.join("vendor/ASIOSDK");
    let cache = root.join(".cache");
    fs::create_dir_all(&cache).context("creating .cache")?;
    let zip_path = cache.join("asio-sdk.zip");

    let cached = file_sha256(&zip_path).is_ok_and(|hash| hash == SDK_SHA256);
    if cached {
        ok("using the cached SDK zip");
    } else {
        download(&zip_path).await?;
    }

    let staging = cache.join("asio-sdk-extract");
    let _ = fs::remove_dir_all(&staging);
    extract(&zip_path, &staging)?;
    let sdk = find_sdk_root(&staging)?;
    install(&sdk, &dest)?;
    let _ = fs::remove_dir_all(&staging);

    if !present(root) {
        bail!("vendor/ASIOSDK is missing host/ or common/ after extract");
    }
    ok(&format!("installed {SDK_LABEL} at vendor/ASIOSDK"));
    Ok(())
}

async fn download(zip_path: &Path) -> Result<()> {
    let client = Client::builder()
        .user_agent("requiem-dev")
        .timeout(Duration::from_secs(180))
        .build()?;
    let response = client
        .get(SDK_URL)
        .send()
        .await
        .with_context(|| format!("downloading {SDK_URL}"))?
        .error_for_status()
        .with_context(|| format!("downloading {SDK_URL}"))?;
    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("reading {SDK_URL}"))?;
    let hash = hex_sha256(&bytes);
    if hash != SDK_SHA256 {
        bail!("ASIO SDK zip hash is {hash}, expected {SDK_SHA256}");
    }
    let partial = zip_path.with_extension("partial");
    fs::write(&partial, &bytes).with_context(|| format!("writing {}", partial.display()))?;
    fs::rename(&partial, zip_path).with_context(|| format!("writing {}", zip_path.display()))?;
    ok(&format!(
        "downloaded {} ({})",
        zip_path.display(),
        bytes.len()
    ));
    Ok(())
}

fn file_sha256(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex_encode(&hasher.finalize()))
}

fn hex_sha256(bytes: &[u8]) -> String {
    hex_encode(&Sha256::digest(bytes))
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0xf) as usize] as char);
    }
    out
}

struct Entry {
    index: usize,
    parts: Vec<String>,
    is_dir: bool,
}

fn extract(zip_path: &Path, dest: &Path) -> Result<()> {
    let mut archive = zip::ZipArchive::new(File::open(zip_path).context("opening ASIO SDK zip")?)
        .context("ASIO SDK zip is not a zip")?;
    if archive.len() > 2_000 {
        bail!("ASIO SDK zip has {} entries", archive.len());
    }

    let mut entries = Vec::new();
    let mut total = 0u64;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .with_context(|| format!("reading zip entry {index}"))?;
        if entry.is_symlink() {
            bail!("ASIO SDK zip contains a symlink ({})", entry.name());
        }
        let name = entry
            .enclosed_name()
            .with_context(|| format!("zip entry `{}` escapes the archive", entry.name()))?;
        let parts = safe_components(&name)
            .with_context(|| format!("zip entry `{}` is not a safe path", entry.name()))?;
        total = total.saturating_add(entry.size());
        if total > 40 * 1024 * 1024 {
            bail!("ASIO SDK zip unpacks to more than 40 MiB");
        }
        entries.push(Entry {
            index,
            parts,
            is_dir: entry.is_dir(),
        });
    }

    fs::create_dir_all(dest).context("creating the extract directory")?;
    for entry in &entries {
        if entry.parts.is_empty() {
            continue;
        }
        let target = entry
            .parts
            .iter()
            .fold(dest.to_path_buf(), |p, part| p.join(part));
        if entry.is_dir {
            fs::create_dir_all(&target)?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = archive.by_index(entry.index)?;
        let mut out = File::create(&target)?;
        let declared = file.size();
        let copied = io::copy(&mut (&mut file).take(declared + 1), &mut out)?;
        if copied != declared {
            bail!(
                "zip entry `{}` is larger than it declares",
                entry.parts.join("/")
            );
        }
        out.flush()?;
    }
    Ok(())
}

fn safe_components(path: &Path) -> Option<Vec<String>> {
    let mut out = Vec::new();
    for component in path.components() {
        let Component::Normal(part) = component else {
            return None;
        };
        let part = part.to_str()?;
        if part.is_empty()
            || part.contains([':', '\\', '<', '>', '"', '|', '?', '*'])
            || part.ends_with(['.', ' '])
        {
            return None;
        }
        out.push(part.to_string());
    }
    Some(out)
}

fn find_sdk_root(extracted: &Path) -> Result<PathBuf> {
    let mut stack = vec![extracted.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if dir.join("common/asio.h").is_file() && dir.join("host/asiodrivers.h").is_file() {
            return Ok(dir);
        }
        let entries = fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))?;
        for entry in entries {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                stack.push(entry.path());
            }
        }
    }
    bail!("the ASIO SDK zip has no directory containing host/ and common/")
}

fn install(sdk: &Path, dest: &Path) -> Result<()> {
    let partial = dest.with_extension("partial");
    let _ = fs::remove_dir_all(&partial);
    copy_dir(sdk, &partial)?;
    if dest.exists() {
        let stale = dest.with_extension("old");
        let _ = fs::remove_dir_all(&stale);
        fs::rename(dest, &stale).with_context(|| format!("replacing {}", dest.display()))?;
        if let Err(err) = fs::rename(&partial, dest) {
            let _ = fs::rename(&stale, dest);
            return Err(err).with_context(|| format!("installing {}", dest.display()));
        }
        let _ = fs::remove_dir_all(&stale);
    } else if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
        fs::rename(&partial, dest).with_context(|| format!("installing {}", dest.display()))?;
    }
    Ok(())
}

fn copy_dir(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}
