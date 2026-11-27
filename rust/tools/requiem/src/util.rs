use std::path::PathBuf;

use anyhow::{Context, Result};
use std::env;
use tokio::process::Command;

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";

pub fn fg(code: u8) -> String {
    format!("\x1b[38;5;{code}m")
}

pub fn step(msg: &str) {
    println!("{BOLD}▸{RESET} {msg}");
}

pub fn ok(msg: &str) {
    println!("  {}✓{} {msg}", fg(42), RESET);
}

pub fn warn(msg: &str) {
    println!("  {}!{} {msg}", fg(214), RESET);
}

pub fn bad(msg: &str) {
    println!("  {}✗{} {msg}", fg(203), RESET);
}

pub fn hint(msg: &str) {
    println!("      {DIM}↳ {msg}{RESET}");
}

pub fn workspace_root() -> Result<PathBuf> {
    let start = env::current_dir().context("reading current directory")?;
    let mut dir = start.as_path();
    loop {
        if dir.join("rust-toolchain.toml").is_file() && dir.join("crates").is_dir() {
            return Ok(dir.to_path_buf());
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => anyhow::bail!(
                "could not find the repo root (no rust-toolchain.toml + crates/ in {} or any parent)",
                start.display()
            ),
        }
    }
}

pub async fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().await.ok()?;
    if !output.status.success() {
        return None;
    }
    let text = if output.stdout.is_empty() {
        output.stderr
    } else {
        output.stdout
    };
    Some(
        String::from_utf8_lossy(&text)
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string(),
    )
}

pub async fn command_output_all(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().await.ok()?;
    Some(format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}
