use std::path::Path;

use anyhow::{Context, Result};

use crate::commands::asio;
use crate::env::BuildEnv;
use crate::util::{DIM, RESET, step, warn};
use tokio::process::Command;

const DESKTOP: &str = "requiem";

const TEST_PACKAGES: &[&str] = &[
    "requiem-icons",
    "requiem-assets",
    "requiem-asset-store",
    "requiem-asset-build",
    "requiem-ui",
    "requiem",
    "requiem-core",
    "requiem-plugin-host",
    "requiem-cli",
];

pub async fn cargo(root: &Path, args: Vec<String>) -> Result<i32> {
    asio::ensure(root).await?;
    let env = BuildEnv::discover();
    if cfg!(windows) {
        match (&env.msvc, &env.sdk) {
            (Some(msvc), Some(sdk)) => println!(
                "{DIM}MSVC {} · Windows SDK {}{}{RESET}",
                msvc.version,
                sdk.version,
                env.ffmpeg
                    .as_ref()
                    .map_or(String::new(), |f| format!(" · FFmpeg {}", f.display()))
            ),
            _ => warn(
                "MSVC or the Windows SDK was not found; building may fail (run `cargo requiem doctor`)",
            ),
        }
    }
    let mut command = Command::new("cargo");
    command.args(&args).current_dir(root);
    for (name, value) in env.vars() {
        command.env(name, value);
    }
    let status = command.status().await.context("starting cargo")?;
    Ok(status.code().unwrap_or(1))
}

pub async fn build(root: &Path, package: String, release: bool, extra: Vec<String>) -> Result<i32> {
    step(&format!(
        "Building {package}{}",
        if release { " (release)" } else { "" }
    ));
    let mut args = vec!["build".to_string(), "-p".into(), package.clone()];
    if release {
        args.push("--release".into());
    }
    args.extend(extra);
    let built = cargo(root, args).await?;
    if built != 0 || release {
        return Ok(built);
    }
    step(&format!("Linting {package}"));
    cargo(
        root,
        vec![
            "clippy".into(),
            "-p".into(),
            package,
            "--".into(),
            "-D".into(),
            "warnings".into(),
        ],
    )
    .await
}

pub async fn run(root: &Path, release: bool, app_args: Vec<String>) -> Result<i32> {
    let mut args = vec!["run".to_string(), "-p".into(), DESKTOP.into()];
    if release {
        args.push("--release".into());
    }
    if !app_args.is_empty() {
        args.push("--".into());
        args.extend(app_args);
    }
    cargo(root, args).await
}

pub async fn test(root: &Path, package: Option<String>, extra: Vec<String>) -> Result<i32> {
    let mut args = vec!["test".to_string()];
    match package {
        Some(package) => args.extend(["-p".to_string(), package]),
        None => {
            for package in TEST_PACKAGES {
                args.extend(["-p".to_string(), (*package).to_string()]);
            }
        }
    }
    args.extend(extra);
    cargo(root, args).await
}
