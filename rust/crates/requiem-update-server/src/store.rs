use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use axum::body::Body;
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::ApiError;
use semver::Version;
use std::io;
use std::process;
use tokio::fs;
use tokio::fs::File;

pub fn release_version(text: &str) -> Option<Version> {
    if text.len() > 64
        || !text
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'+'))
    {
        return None;
    }
    Version::parse(text).ok()
}

pub fn release_name_ok(name: &str) -> bool {
    (name.ends_with(".zip") || name.ends_with(".exe"))
        && name.len() <= 128
        && name
            .bytes()
            .next()
            .is_some_and(|b| b.is_ascii_alphanumeric())
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        && !name.contains("..")
}

pub fn slug_ok(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 32
        && text
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

pub fn feed_target(file: &str) -> Option<&str> {
    file.strip_suffix(".json").filter(|t| slug_ok(t))
}

pub struct Received {
    pub tmp: PathBuf,
    pub sha256: String,
    pub size: u64,
}

impl Received {
    pub async fn discard(self) {
        let _ = fs::remove_file(&self.tmp).await;
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn tmp_path(root: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    root.join(".tmp").join(format!(
        "upload-{}-{}",
        process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ))
}

pub async fn receive(root: &Path, body: Body, max: u64) -> Result<Received, ApiError> {
    let tmp = tmp_path(root);
    fs::create_dir_all(tmp.parent().expect("has a parent"))
        .await
        .map_err(ApiError::internal)?;
    let mut file = File::create(&tmp).await.map_err(ApiError::internal)?;
    let (mut hasher, mut size) = (Sha256::new(), 0u64);
    let mut stream = body.into_data_stream();
    let result: Result<(), ApiError> = async {
        while let Some(chunk) = stream.next().await {
            let chunk =
                chunk.map_err(|e| ApiError::bad(format!("the upload was interrupted: {e}")))?;
            size += chunk.len() as u64;
            if size > max {
                return Err(ApiError::TooLarge(format!("more than {max} bytes")));
            }
            hasher.update(&chunk);
            file.write_all(&chunk).await.map_err(ApiError::internal)?;
        }
        file.flush().await.map_err(ApiError::internal)
    }
    .await;
    drop(file);
    match result {
        Ok(()) => Ok(Received {
            tmp,
            sha256: hex(&hasher.finalize()),
            size,
        }),
        Err(err) => {
            let _ = fs::remove_file(&tmp).await;
            Err(err)
        }
    }
}

pub async fn read_capped(body: Body, max: usize) -> Result<Vec<u8>, ApiError> {
    let mut out = Vec::new();
    let mut stream = body.into_data_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| ApiError::bad(format!("the upload was interrupted: {e}")))?;
        if out.len() + chunk.len() > max {
            return Err(ApiError::TooLarge(format!("more than {max} bytes")));
        }
        out.extend_from_slice(&chunk);
    }
    Ok(out)
}

pub async fn file_hash(path: &Path) -> io::Result<(String, u64)> {
    let mut file = File::open(path).await?;
    let (mut hasher, mut size) = (Sha256::new(), 0u64);
    let mut buf = vec![0u8; 256 * 1024];
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            return Ok((hex(&hasher.finalize()), size));
        }
        hasher.update(&buf[..n]);
        size += n as u64;
    }
}

pub async fn commit(tmp: &Path, dest: &Path) -> Result<(), ApiError> {
    if let Some(dir) = dest.parent() {
        fs::create_dir_all(dir).await.map_err(ApiError::internal)?;
    }
    fs::rename(tmp, dest).await.map_err(ApiError::internal)
}

pub async fn write_atomic(root: &Path, dest: &Path, bytes: &[u8]) -> Result<(), ApiError> {
    let tmp = tmp_path(root);
    fs::create_dir_all(tmp.parent().expect("has a parent"))
        .await
        .map_err(ApiError::internal)?;
    fs::write(&tmp, bytes).await.map_err(ApiError::internal)?;
    commit(&tmp, dest).await
}

pub fn parts_dir(root: &Path, version: &str, name: &str) -> PathBuf {
    root.join(".tmp").join("parts").join(version).join(name)
}

pub async fn assemble(root: &Path, parts: &[PathBuf]) -> Result<Received, ApiError> {
    let tmp = tmp_path(root);
    fs::create_dir_all(tmp.parent().expect("has a parent"))
        .await
        .map_err(ApiError::internal)?;
    let result: Result<(String, u64), ApiError> = async {
        let mut out = File::create(&tmp).await.map_err(ApiError::internal)?;
        let (mut hasher, mut size) = (Sha256::new(), 0u64);
        let mut buf = vec![0u8; 256 * 1024];
        for part in parts {
            let mut file = File::open(part).await.map_err(ApiError::internal)?;
            loop {
                let n = file.read(&mut buf).await.map_err(ApiError::internal)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
                out.write_all(&buf[..n]).await.map_err(ApiError::internal)?;
                size += n as u64;
            }
        }
        out.flush().await.map_err(ApiError::internal)?;
        Ok((hex(&hasher.finalize()), size))
    }
    .await;
    match result {
        Ok((sha256, size)) => Ok(Received { tmp, sha256, size }),
        Err(err) => {
            let _ = fs::remove_file(&tmp).await;
            Err(err)
        }
    }
}

pub async fn clear_tmp(root: &Path) {
    let _ = fs::remove_dir_all(root.join(".tmp")).await;
}
