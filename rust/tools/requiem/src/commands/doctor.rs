use std::path::Path;

use anyhow::Result;

use crate::commands::asio;
use crate::env::BuildEnv;
use crate::util::{self, DIM, RESET, bad, hint, ok, step, warn};
use requiem_assets::manifest;
use requiem_assets::recipes;
use std::fs;

pub async fn run(root: &Path) -> Result<i32> {
    let mut problems = 0;

    step("Rust");
    let pinned = fs::read_to_string(root.join("rust-toolchain.toml"))
        .ok()
        .and_then(|t| {
            t.lines().find_map(|l| {
                l.trim().strip_prefix("channel").map(|r| {
                    r.trim_start_matches([' ', '='])
                        .trim()
                        .trim_matches('"')
                        .to_string()
                })
            })
        });
    match util::command_output("rustc", &["--version"]).await {
        Some(v) => ok(&format!(
            "{v}{}",
            pinned
                .as_ref()
                .map_or(String::new(), |p| format!("  {DIM}(pinned {p}){RESET}"))
        )),
        None => {
            bad("rustc not found");
            problems += 1;
        }
    }
    if util::command_output("cargo", &["--version"])
        .await
        .is_none()
    {
        bad("cargo not found");
        problems += 1;
    }

    if cfg!(windows) {
        step("Windows build environment");
        let env = BuildEnv::discover();
        match &env.msvc {
            Some(msvc) => ok(&format!(
                "MSVC {}  {DIM}{}{RESET}",
                msvc.version,
                msvc.root.display()
            )),
            None => {
                bad("MSVC (Visual Studio C++ build tools) not found");
                hint("install the \"Desktop development with C++\" workload");
                problems += 1;
            }
        }
        match &env.sdk {
            Some(sdk) => ok(&format!("Windows SDK {}", sdk.version)),
            None => {
                bad("Windows 10/11 SDK not found");
                problems += 1;
            }
        }
        match &env.ffmpeg {
            Some(dir) => {
                ok(&format!("FFmpeg  {DIM}{}{RESET}", dir.display()));
                let ffmpeg = dir.join("bin").join("ffmpeg.exe");
                let decoders = util::command_output_all(
                    ffmpeg.to_string_lossy().as_ref(),
                    &["-hide_banner", "-decoders"],
                )
                .await
                .unwrap_or_default();
                if decoders.contains("libvpx-vp9") {
                    ok(
                        "FFmpeg has the libvpx-vp9 decoder (needed for the logo animation's transparency)",
                    );
                } else {
                    warn(
                        "this FFmpeg has no libvpx-vp9 decoder: the splash screen would have no logo",
                    );
                    hint("use the winget package Gyan.FFmpeg.Shared");
                }
            }
            None => {
                bad("FFmpeg (shared build) not found: the `video` feature cannot build");
                hint("winget install Gyan.FFmpeg.Shared, or set FFMPEG_DIR");
                problems += 1;
            }
        }
        if asio::present(root) {
            ok(&format!("Steinberg ASIO SDK  {DIM}vendor/ASIOSDK{RESET}"));
        } else {
            warn("Steinberg ASIO SDK is missing: an `asio` build cannot compile");
            hint("run `cargo requiem asio`, or `cargo requiem build` (it fetches the SDK)");
        }
    }

    step("Project");
    if root.join(".env").is_file() {
        ok(".env present");
    } else {
        warn(".env missing");
    }
    let manifest_path = root.join("assets").join("manifest.json");
    match fs::read_to_string(&manifest_path)
        .ok()
        .and_then(|t| serde_json::from_str::<manifest::Manifest>(&t).ok())
    {
        Some(manifest) => {
            ok(&format!(
                "assets/manifest.json: {} entries, client build {}",
                manifest.entries.len(),
                manifest
                    .client
                    .build_number
                    .map_or("?".into(), |n| n.to_string())
            ));
            let missing: Vec<String> = recipes::all()
                .iter()
                .map(|r| r.name().to_string())
                .filter(|n| manifest.entry(n).is_none())
                .collect();
            if !missing.is_empty() {
                warn(&format!(
                    "{} recipe(s) are not in the manifest yet (e.g. {})",
                    missing.len(),
                    missing[0]
                ));
                hint("run `cargo requiem assets resolve`");
            }
        }
        None => {
            bad("assets/manifest.json is missing or unreadable: the app cannot build without it");
            hint("run `cargo requiem assets resolve`");
            problems += 1;
        }
    }

    Ok(if problems == 0 { 0 } else { 1 })
}
