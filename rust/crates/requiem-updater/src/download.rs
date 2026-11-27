use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use requiem_update::{Artifact, ArtifactCheck, Error};
use std::fmt;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;

#[derive(Debug, PartialEq, Eq)]
pub enum DownloadError {
    AllFailed(Vec<String>),
    Io(String),
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AllFailed(reasons) => {
                write!(f, "could not download the update: {}", reasons.join("; "))
            }
            Self::Io(e) => write!(f, "i/o error: {e}"),
        }
    }
}

fn io(e: io::Error) -> DownloadError {
    DownloadError::Io(e.to_string())
}

pub async fn download(
    client: &wreq::Client,
    artifact: &Artifact,
    dir: &Path,
    file_name: &str,
    on_progress: &(dyn Fn(u64, u64) + Sync),
) -> Result<PathBuf, DownloadError> {
    fs::create_dir_all(dir).map_err(io)?;
    let finished = dir.join(file_name);
    let part = dir.join(format!("{file_name}.part"));
    let mut reasons = Vec::new();
    for url in &artifact.urls {
        match attempt(client, artifact, url, &part, on_progress).await {
            Ok(()) => {
                fs::rename(&part, &finished).map_err(io)?;
                return Ok(finished);
            }
            Err(Attempt::Retryable(reason)) => reasons.push(format!("{url}: {reason}")),

            Err(Attempt::Corrupt(reason)) => {
                let _ = fs::remove_file(&part);
                reasons.push(format!("{url}: {reason}"));
            }
            Err(Attempt::Local(e)) => return Err(e),
        }
    }
    Err(DownloadError::AllFailed(reasons))
}

enum Attempt {
    Retryable(String),

    Corrupt(String),
    Local(DownloadError),
}

async fn attempt(
    client: &wreq::Client,
    artifact: &Artifact,
    url: &str,
    part: &Path,
    on_progress: &(dyn Fn(u64, u64) + Sync),
) -> Result<(), Attempt> {
    let local = |e: io::Error| Attempt::Local(io(e));
    let mut have = fs::metadata(part).map(|m| m.len()).unwrap_or(0);
    if have > artifact.size {
        let _ = fs::remove_file(part);
        have = 0;
    }

    let mut request = client.get(url);
    if have > 0 && have < artifact.size {
        request = request.header("Range", format!("bytes={have}-"));
    }
    let response = if have == artifact.size {
        None
    } else {
        Some(
            request
                .send()
                .await
                .map_err(|e| Attempt::Retryable(e.to_string()))?,
        )
    };

    let mut check = ArtifactCheck::new(artifact);
    let mut file;
    match &response {
        Some(r) if r.status().as_u16() == 206 => {
            feed_existing(&mut check, part, have).map_err(local)?;
            file = OpenOptions::new().append(true).open(part).map_err(local)?;
        }
        Some(r) if r.status().is_success() => {
            file = File::create(part).map_err(local)?;
        }
        Some(r) => return Err(Attempt::Retryable(format!("HTTP {}", r.status().as_u16()))),
        None => {
            feed_existing(&mut check, part, have).map_err(local)?;
            file = OpenOptions::new().append(true).open(part).map_err(local)?;
        }
    }

    if let Some(response) = response {
        on_progress(check.seen(), artifact.size);
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| Attempt::Retryable(e.to_string()))?;
            check
                .update(&chunk)
                .map_err(|e| Attempt::Corrupt(e.to_string()))?;
            file.write_all(&chunk).map_err(local)?;
            on_progress(check.seen(), artifact.size);
        }
    }
    file.flush().map_err(local)?;
    drop(file);
    match check.finish() {
        Ok(()) => Ok(()),

        Err(Error::WrongSize { expected, found }) if found < expected => Err(Attempt::Retryable(
            format!("ended after {found} of {expected} bytes"),
        )),
        Err(e) => Err(Attempt::Corrupt(e.to_string())),
    }
}

fn feed_existing(check: &mut ArtifactCheck, part: &Path, len: u64) -> io::Result<()> {
    let mut file = File::open(part)?.take(len);
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            return Ok(());
        }
        check
            .update(&buf[..n])
            .map_err(|e| io::Error::other(e.to_string()))?;
    }
}
