use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::env::BuildEnv;
use crate::util::{hint, ok, step, warn};
use requiem_plugin_host::LogLevel;
use requiem_plugin_host::Manifest;
use requiem_plugin_host::PluginHost;
use std::env;
use std::fs;
use std::process;
use std::process::Stdio;
use tokio::process::Command;

const TARGET: &str = "wasm32-wasip2";

pub async fn build(root: &Path, dir: Option<PathBuf>, debug: bool) -> Result<i32> {
    let dir = fs::canonicalize(dir.unwrap_or_else(|| PathBuf::from(".")))
        .context("finding the plugin folder")?;
    let manifest_text = fs::read_to_string(dir.join("plugin.toml"))
        .with_context(|| format!("no plugin.toml in {}", dir.display()))?;
    let manifest = Manifest::parse(&manifest_text).context("plugin.toml")?;
    step(&format!(
        "Building {} {} ({})",
        manifest.name, manifest.version, manifest.id
    ));

    let mut command = Command::new("cargo");
    command
        .args([
            "build",
            "--target",
            TARGET,
            "--message-format=json-render-diagnostics",
        ])
        .current_dir(&dir)
        .stdout(Stdio::piped());
    if !debug {
        command.arg("--release");
    }
    for (name, value) in BuildEnv::discover().vars() {
        command.env(name, value);
    }
    let output = command.output().await.context("starting cargo")?;
    if !output.status.success() {
        bail!("cargo build failed");
    }

    let crate_manifest = dir.join("Cargo.toml");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let messages: Vec<Value> = stdout
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    let package_id = messages
        .iter()
        .find(|m| {
            m["reason"] == "compiler-artifact"
                && m["manifest_path"]
                    .as_str()
                    .is_some_and(|path| same_file(Path::new(path), &crate_manifest))
        })
        .and_then(|m| m["package_id"].as_str())
        .context("cargo reported nothing for the plugin crate")?;
    let ours: Vec<&Value> = messages
        .iter()
        .filter(|m| m["package_id"] == package_id)
        .collect();
    let wasm = ours
        .iter()
        .filter(|m| m["reason"] == "compiler-artifact")
        .flat_map(|m| {
            m["filenames"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
        })
        .find(|f| f.ends_with(".wasm"))
        .map(PathBuf::from);
    let out_dir = ours
        .iter()
        .find(|m| m["reason"] == "build-script-executed")
        .and_then(|m| m["out_dir"].as_str())
        .map(PathBuf::from);
    let wasm =
        wasm.context("the build made no .wasm: is the plugin crate `crate-type = [\"cdylib\"]`?")?;

    let target_dir = wasm
        .ancestors()
        .find(|p| p.file_name().is_some_and(|n| n == "target"))
        .map(Path::to_path_buf)
        .unwrap_or_else(|| dir.join("target"));
    let package = target_dir.join("drplugin").join(&manifest.id);
    let _ = fs::remove_dir_all(&package);
    fs::create_dir_all(&package)?;
    fs::copy(dir.join("plugin.toml"), package.join("plugin.toml"))?;
    fs::copy(&wasm, package.join("plugin.wasm"))?;
    match out_dir
        .map(|d| d.join("bundled.css"))
        .filter(|css| css.is_file())
    {
        Some(css) => {
            fs::copy(css, package.join("styles.css"))?;
        }
        None => hint(
            "no styles.css (add `requiem_plugin_build::styles()` to build.rs to ship CSS modules)",
        ),
    }
    if dir.join("README.md").is_file() {
        fs::copy(dir.join("README.md"), package.join("README.md"))?;
    }
    let size = fs::metadata(package.join("plugin.wasm"))?.len();
    ok(&format!("{} ({} KiB)", package.display(), size / 1024));

    check(&package, &manifest.id)?;
    hint("install it from Settings > Plugins > Install from Folder");
    let _ = root;
    Ok(0)
}

fn check(package: &Path, id: &str) -> Result<()> {
    struct Quiet;
    impl requiem_plugin_host::HostServices for Quiet {
        fn log(&self, _: &str, level: LogLevel, message: &str) {
            if matches!(level, LogLevel::Warn | LogLevel::Error) {
                warn(&format!("plugin: {message}"));
            }
        }
        fn overlay(&self, _: &str, _: requiem_plugin_host::OverlayUpdate) {}
        fn secret_get(&self, _: &str, _: &str) -> requiem_plugin_host::HostResult<Option<String>> {
            Ok(None)
        }
        fn secret_set(&self, _: &str, _: &str, _: &str) -> requiem_plugin_host::HostResult<()> {
            Ok(())
        }
        fn secret_delete(&self, _: &str, _: &str) -> requiem_plugin_host::HostResult<()> {
            Ok(())
        }
    }
    let scratch = env::temp_dir().join(format!("requiem-plugin-check-{}", process::id()));
    let _ = fs::remove_dir_all(&scratch);
    let result = (|| -> Result<()> {
        let host = PluginHost::new(&scratch, Arc::new(Quiet))?;
        host.install_dir(package)?;
        host.set_enabled(id, true)?;
        let info = host
            .list()
            .into_iter()
            .find(|p| p.id == id)
            .context("the plugin didn't load")?;
        if info.has_settings {
            let view = host.render_view(id, "settings", "{}")?;
            ok(&format!(
                "starts, and its settings page renders ({} nodes)",
                view.nodes.len()
            ));
        } else {
            host.render_view(id, "settings", "{}")?;
            ok("starts");
        }
        Ok(())
    })();
    let _ = fs::remove_dir_all(&scratch);
    result.context("the built plugin doesn't load")
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

pub fn new(root: &Path, name: &str, id: Option<String>, dir: Option<PathBuf>) -> Result<i32> {
    let slug: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if slug.is_empty() {
        bail!("the name needs at least one letter or digit");
    }
    let id = id.unwrap_or_else(|| format!("dev.example.{slug}"));
    let target = dir.unwrap_or_else(|| PathBuf::from(".")).join(&slug);
    if target.exists() {
        bail!("{} already exists", target.display());
    }
    let sdk = root.join("crates").join("plugin-sdk");
    fs::create_dir_all(target.join("src"))?;
    fs::create_dir_all(target.join(".cargo"))?;
    let target_abs = fs::canonicalize(&target)?;
    let sdk_path = relative(&target_abs, &sdk)
        .unwrap_or(sdk.clone())
        .to_string_lossy()
        .replace('\\', "/");
    let crate_name = slug.replace('-', "_");
    let in_sdk_workspace =
        target_abs.starts_with(fs::canonicalize(sdk.join("examples")).unwrap_or_default());

    let files: Vec<(&str, String)> = vec![
        (
            "Cargo.toml",
            format!(
                r#"[package]
name = "{slug}"
version = "0.1.0"
edition = "2021"
publish = false
{workspace}
[lib]
crate-type = ["cdylib"]

[dependencies]
requiem-plugin = {{ path = "{sdk_path}" }}
serde = {{ version = "1", features = ["derive"] }}

[build-dependencies]
requiem-plugin-build = {{ path = "{sdk_path}/build-support" }}
{profile}"#,
                workspace = if in_sdk_workspace {
                    ""
                } else {
                    "\n# Its own workspace (not part of the client's).\n[workspace]\n"
                },
                profile = if in_sdk_workspace {
                    ""
                } else {
                    "\n[profile.release]\nopt-level = \"s\"\nlto = true\ncodegen-units = 1\nstrip = true\n"
                },
            ),
        ),
        (
            "plugin.toml",
            format!(
                r#"id = "{id}"
name = "{name}"
version = "0.1.0"
api = "1.0"
description = ""
authors = []

# Each permission with why the plugin needs it (shown when it's installed):
# [permissions]
# "messages.intercept-incoming" = "..."
# "messages.intercept-outgoing" = "..."

[hooks]
settings-ui = {{}}
# incoming = {{}}   # needs messages.intercept-incoming
# outgoing = {{}}   # needs messages.intercept-outgoing
"#
            ),
        ),
        (
            "build.rs",
            "fn main() {\n    requiem_plugin_build::styles();\n}\n".to_string(),
        ),
        (
            ".cargo/config.toml",
            "[build]\ntarget = \"wasm32-wasip2\"\n".to_string(),
        ),
        (
            "src/lib.rs",
            format!(
                r#"use requiem_plugin::prelude::*;
use serde::{{Deserialize, Serialize}};

requiem_plugin::classes!();
use classes::settings as cls;

/// The plugin's settings, stored with `settings::save`; defaults live here.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct Settings {{
    enabled_thing: bool,
}}

struct {type_name};

impl Plugin for {type_name} {{
    fn init(_previous_version: Option<String>) -> Result<(), Error> {{
        log::info("{name} started");
        Ok(())
    }}

    fn settings_page() -> Node {{
        let current = settings::load::<Settings>();
        div().class(cls::PAGE).child(form::switch_row("Enable the thing", None, current.enabled_thing, |on| settings::update::<Settings>(|s| s.enabled_thing = on)))
    }}
}}

export_plugin!({type_name});
"#,
                type_name = crate_name
                    .split('_')
                    .map(|w| w[..1].to_uppercase() + &w[1..])
                    .collect::<String>(),
            ),
        ),
        (
            "src/settings.module.css",
            ".page {\n    display: flex;\n    flex-direction: column;\n    gap: 16px;\n}\n"
                .to_string(),
        ),
        (".gitignore", "/target\n".to_string()),
    ];
    for (file, content) in files {
        fs::write(target.join(file), content)?;
    }
    ok(&format!("created {}", target.display()));
    hint(&format!(
        "build it with `cargo requiem plugin build {}`",
        target.display()
    ));
    Ok(0)
}

fn relative(from: &Path, to: &Path) -> Option<PathBuf> {
    let to = fs::canonicalize(to).ok()?;
    let from: Vec<_> = from.components().collect();
    let to_parts: Vec<_> = to.components().collect();
    let common = from
        .iter()
        .zip(&to_parts)
        .take_while(|(a, b)| a == b)
        .count();
    if common == 0 {
        return None;
    }
    let mut path = PathBuf::new();
    for _ in common..from.len() {
        path.push("..");
    }
    for part in &to_parts[common..] {
        path.push(part);
    }
    Some(path)
}
