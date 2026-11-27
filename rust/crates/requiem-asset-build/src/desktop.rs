use std::hash::BuildHasher;
use std::hash::Hasher;
use std::io::Read;

use serde_json::Value;

use crate::Error;
use brotli::Decompressor;
use reqwest::Client;
use std::collections::hash_map::RandomState;
use tar::Archive;
use tokio::task;

const MANIFEST: &str = "https://updates.discord.com/distributions/app/manifests/latest";

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DesktopInfo {
    pub host_version: String,

    pub native_build_number: Option<u64>,
    pub electron_version: String,

    pub chrome_version: String,
}

impl DesktopInfo {
    pub fn chrome_major(&self) -> Option<u32> {
        self.chrome_version.split('.').next()?.parse().ok()
    }
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub host_version: String,
    pub native_build_number: Option<u64>,
    package_url: String,
}

pub async fn manifest(
    client: &Client,
    channel: &str,
    platform: &str,
    arch: &str,
) -> Result<Manifest, Error> {
    let id = format!("{:032x}", rand_u128());
    let url = format!(
        "{MANIFEST}?channel={channel}&platform={platform}&arch={arch}&install_id={}-{}-{}-{}-{}",
        &id[..8],
        &id[8..12],
        &id[12..16],
        &id[16..20],
        &id[20..]
    );
    let manifest: Value = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let host_version = manifest["full"]["host_version"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(Value::as_u64)
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(".")
        })
        .filter(|v| v.split('.').count() == 3)
        .ok_or_else(|| Error::Other("manifest has no full.host_version".into()))?;
    let package_url = manifest["full"]["url"]
        .as_str()
        .ok_or_else(|| Error::Other("manifest has no full.url".into()))?
        .to_string();
    Ok(Manifest {
        host_version,
        native_build_number: manifest["metadata_version"].as_u64(),
        package_url,
    })
}

pub async fn scan(client: &Client, manifest: &Manifest) -> Result<DesktopInfo, Error> {
    let package = client
        .get(&manifest.package_url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    let (electron_version, chrome_version) =
        task::spawn_blocking(move || versions_in_package(&package))
            .await
            .map_err(|e| Error::Other(e.to_string()))??;
    Ok(DesktopInfo {
        host_version: manifest.host_version.clone(),
        native_build_number: manifest.native_build_number,
        electron_version,
        chrome_version,
    })
}

pub async fn probe(client: &Client, channel: &str) -> Result<DesktopInfo, Error> {
    scan(client, &manifest(client, channel, "win", "x64").await?).await
}

pub fn versions_in_package(distro: &[u8]) -> Result<(String, String), Error> {
    let mut archive = Archive::new(Decompressor::new(distro, 64 * 1024));
    for entry in archive.entries()? {
        let mut entry = entry?;
        if entry.path()?.to_string_lossy() != "files/Discord.exe" {
            continue;
        }
        let mut exe = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut exe)?;
        let electron = version_after(&exe, b"Electron/")
            .ok_or_else(|| Error::Other("no Electron/ version in Discord.exe".into()))?;
        let chrome = version_after(&exe, b"Chrome/")
            .ok_or_else(|| Error::Other("no Chrome/ version in Discord.exe".into()))?;
        return Ok((electron, chrome));
    }
    Err(Error::Other("package has no files/Discord.exe".into()))
}

fn version_after(haystack: &[u8], marker: &[u8]) -> Option<String> {
    let mut from = 0;
    while let Some(pos) = haystack[from..]
        .windows(marker.len())
        .position(|w| w == marker)
    {
        let start = from + pos + marker.len();
        let version: String = haystack[start..]
            .iter()
            .take_while(|b| b.is_ascii_digit() || **b == b'.')
            .map(|b| *b as char)
            .collect();
        if version.split('.').count() >= 3 {
            return Some(version);
        }
        from = start;
    }
    None
}

fn rand_u128() -> u128 {
    let state = RandomState::new();
    let (mut a, mut b) = (state.build_hasher(), state.build_hasher());
    a.write_u8(1);
    b.write_u8(2);
    ((a.finish() as u128) << 64) | b.finish() as u128
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_versions_after_markers() {
        let blob = b"junk Electron/abc Electron/42.11.8\0 more Chrome/148.0.7778.280 Safari";
        assert_eq!(
            version_after(blob, b"Electron/").as_deref(),
            Some("42.11.8")
        );
        assert_eq!(
            version_after(blob, b"Chrome/").as_deref(),
            Some("148.0.7778.280")
        );
    }

    #[tokio::test]
    #[ignore = "downloads the desktop host package"]
    async fn probes_the_live_host_package() {
        let info = probe(&Client::new(), "stable").await.unwrap();
        println!("{info:?}");
        assert!(info.chrome_major().is_some());
    }
}
