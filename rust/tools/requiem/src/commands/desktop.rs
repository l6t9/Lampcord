use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use requiem_asset_build::desktop::{self, DesktopInfo};

use crate::util::{bad, ok, step, warn};
use reqwest::Client;
use std::fs;

const PLATFORMS: [(&str, &str, &str); 3] = [
    ("windows", "win", "x64"),
    ("osx", "osx", "arm64"),
    ("linux", "linux", "x64"),
];

type Identities = BTreeMap<String, DesktopInfo>;

pub async fn run(
    root: &Path,
    channel: &str,
    force: bool,
    check: bool,
    out: Option<&Path>,
) -> Result<i32> {
    let path = out
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.join(".cache").join("desktop-identity.json"));
    let current: Identities = fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    let client = Client::builder().user_agent("Mozilla/5.0").build()?;

    step(&format!("Reading the {channel} update manifests"));
    let mut manifests = BTreeMap::new();
    for (key, platform, arch) in PLATFORMS {
        let manifest = desktop::manifest(&client, channel, platform, arch)
            .await
            .with_context(|| format!("fetching the {key} update manifest"))?;
        ok(&format!("{key}: host {}", manifest.host_version));
        manifests.insert(key, manifest);
    }

    let windows = &manifests["windows"];
    let (electron_version, chrome_version) = match current.get("windows") {
        Some(known) if !force && known.host_version == windows.host_version => {
            ok(
                "Windows host version unchanged: Electron / Chromium can't have moved, skipping the download",
            );
            (known.electron_version.clone(), known.chrome_version.clone())
        }
        _ => {
            step("Scanning the Windows host package (~100 MB)");
            match desktop::scan(&client, windows).await {
                Ok(info) => (info.electron_version, info.chrome_version),
                Err(err) => {
                    warn(&format!(
                        "scanning the host package failed ({err}); trying cordapi"
                    ));
                    match cordapi(&client, "windows").await {
                        Some(api) if api.host_version == windows.host_version => {
                            (api.electron_version, api.chrome_version)
                        }
                        Some(api) => bail!(
                            "cordapi is behind (host {} vs {}) and the package scan failed: {err}",
                            api.host_version,
                            windows.host_version
                        ),
                        None => bail!("package scan failed and cordapi is unavailable: {err}"),
                    }
                }
            }
        }
    };
    ok(&format!(
        "Electron {electron_version} · Chromium {chrome_version}"
    ));

    let mut latest = Identities::new();
    for (key, _, _) in PLATFORMS {
        let manifest = &manifests[key];

        let native_build_number = if key == "windows" {
            manifest.native_build_number
        } else {
            None
        };
        latest.insert(
            key.to_string(),
            DesktopInfo {
                host_version: manifest.host_version.clone(),
                native_build_number,
                electron_version: electron_version.clone(),
                chrome_version: chrome_version.clone(),
            },
        );
        match cordapi(&client, key).await {
            Some(api) if api.host_version != manifest.host_version => warn(&format!(
                "{key}: cordapi is on host {} (manifest {}); not comparing",
                api.host_version, manifest.host_version
            )),
            Some(api)
                if api.electron_version != electron_version
                    || api.chrome_version != chrome_version =>
            {
                warn(&format!(
                    "{key}: cordapi disagrees: Electron {} / Chromium {}",
                    api.electron_version, api.chrome_version
                ))
            }
            Some(_) => ok(&format!("{key}: cordapi agrees")),
            None => warn(&format!("{key}: cordapi unreachable (cross-check skipped)")),
        }
    }

    if current == latest {
        ok("desktop identity is up to date");
        return Ok(0);
    }
    if let Some(known) = current.get("windows") {
        if known.electron_version != electron_version || known.chrome_version != chrome_version {
            warn(&format!(
                "Electron/Chromium moved: {} / {} -> {electron_version} / {chrome_version}",
                known.electron_version, known.chrome_version
            ));
        }
    }
    if check {
        bad("desktop identity is stale");
        return Ok(1);
    }
    fs::write(&path, serde_json::to_string_pretty(&latest)? + "\n")?;
    ok(&format!("wrote {}", path.display()));
    Ok(0)
}

async fn cordapi(client: &Client, slug: &str) -> Option<DesktopInfo> {
    let body: serde_json::Value = client
        .post(format!("https://cordapi.dolfi.es/api/v2/properties/{slug}"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json()
        .await
        .ok()?;
    Some(DesktopInfo {
        host_version: body["properties"]["client_version"].as_str()?.to_string(),
        native_build_number: body["properties"]["native_build_number"].as_u64(),
        electron_version: body["metadata"]["electron_version"].as_str()?.to_string(),
        chrome_version: body["metadata"]["native_chrome_version"]
            .as_str()?
            .to_string(),
    })
}
