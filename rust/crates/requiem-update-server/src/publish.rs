use axum::body::Body;
use axum::http::StatusCode;
use requiem_update::{Decision, KIND_IDENTITY, KIND_UPDATE, Policy, Signed, UpdateManifest};
use semver::Version;

use crate::error::ApiError;
use crate::{AppState, store};
use requiem_update::Error;
use std::str;
use std::time;
use std::time::SystemTime;
use tokio::fs;

const MAX_DOCUMENT: usize = 256 * 1024;

pub struct Outcome {
    pub status: StatusCode,
    pub message: String,
}

fn ok(status: StatusCode, message: impl Into<String>) -> Result<Outcome, ApiError> {
    Ok(Outcome {
        status,
        message: message.into(),
    })
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub async fn put_release(
    state: &AppState,
    version: &str,
    name: &str,
    content_length: Option<u64>,
    body: Body,
) -> Result<Outcome, ApiError> {
    store::release_version(version)
        .ok_or_else(|| ApiError::bad("the version must be a semantic version"))?;
    if !store::release_name_ok(name) {
        return Err(ApiError::bad(
            "the file name must be letters, digits, '.', '_' and '-', ending in .zip or .exe",
        ));
    }

    let max = state
        .config()
        .max_release_bytes
        .min(state.config().max_part_bytes);
    if content_length.is_some_and(|len| len > max) {
        return Err(ApiError::TooLarge(format!(
            "more than {max} bytes in one request; send it in parts"
        )));
    }
    let dest = state
        .config()
        .root
        .join("v1")
        .join("releases")
        .join(version)
        .join(name);
    let received = store::receive(&state.config().root, body, max).await?;
    if received.size == 0 {
        received.discard().await;
        return Err(ApiError::bad("the file is empty"));
    }
    if dest.exists() {
        let (hash, _) = store::file_hash(&dest).await.map_err(ApiError::internal)?;
        let same = hash == received.sha256;
        received.discard().await;
        return if same {
            ok(StatusCode::OK, "already published, unchanged")
        } else {
            Err(ApiError::Conflict(format!(
                "{name} is already published with different content"
            )))
        };
    }
    let (sha, size) = (received.sha256.clone(), received.size);
    store::commit(&received.tmp, &dest).await?;
    log::info!("published release file {version}/{name} ({size} bytes, sha256 {sha})");
    ok(
        StatusCode::CREATED,
        format!("published {size} bytes, sha256 {sha}"),
    )
}

const MAX_PARTS: u32 = 10_000;

pub async fn put_part(
    state: &AppState,
    version: &str,
    name: &str,
    index: &str,
    content_length: Option<u64>,
    body: Body,
) -> Result<Outcome, ApiError> {
    store::release_version(version)
        .ok_or_else(|| ApiError::bad("the version must be a semantic version"))?;
    if !store::release_name_ok(name) {
        return Err(ApiError::bad(
            "the file name must be letters, digits, '.', '_' and '-', ending in .zip or .exe",
        ));
    }
    let index: u32 = index
        .parse()
        .ok()
        .filter(|i| *i < MAX_PARTS)
        .ok_or_else(|| ApiError::bad("the part number must be a number below 10000"))?;
    let max = state.config().max_part_bytes;
    if content_length.is_some_and(|len| len > max) {
        return Err(ApiError::TooLarge(format!("a part is at most {max} bytes")));
    }
    let received = store::receive(&state.config().root, body, max).await?;
    if received.size == 0 {
        received.discard().await;
        return Err(ApiError::bad("the part is empty"));
    }
    let size = received.size;
    store::commit(
        &received.tmp,
        &store::parts_dir(&state.config().root, version, name).join(format!("{index:05}")),
    )
    .await?;
    ok(
        StatusCode::OK,
        format!("stored part {index} ({size} bytes)"),
    )
}

#[derive(serde::Deserialize)]
struct Complete {
    parts: u32,
    size: u64,
    sha256: String,
}

pub async fn complete_parts(
    state: &AppState,
    version: &str,
    name: &str,
    body: Body,
) -> Result<Outcome, ApiError> {
    store::release_version(version)
        .ok_or_else(|| ApiError::bad("the version must be a semantic version"))?;
    if !store::release_name_ok(name) {
        return Err(ApiError::bad(
            "the file name must be letters, digits, '.', '_' and '-', ending in .zip or .exe",
        ));
    }
    let request: Complete = serde_json::from_slice(&store::read_capped(body, 4096).await?)
        .map_err(|e| ApiError::bad(format!("unreadable request: {e}")))?;
    if request.parts == 0
        || request.parts > MAX_PARTS
        || request.size == 0
        || request.size > state.config().max_release_bytes
    {
        return Err(ApiError::bad(
            "the number of parts or the size is out of range",
        ));
    }
    if request.sha256.len() != 64
        || !request
            .sha256
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err(ApiError::bad("sha256 must be 64 lowercase hex digits"));
    }
    let root = &state.config().root;
    let parts_dir = store::parts_dir(root, version, name);
    let dest = root.join("v1").join("releases").join(version).join(name);

    if dest.exists() {
        let (hash, size) = store::file_hash(&dest).await.map_err(ApiError::internal)?;
        let _ = fs::remove_dir_all(&parts_dir).await;
        return if hash == request.sha256 && size == request.size {
            ok(StatusCode::OK, "already published, unchanged")
        } else {
            Err(ApiError::Conflict(format!(
                "{name} is already published with different content"
            )))
        };
    }

    let mut parts = Vec::new();
    let mut total = 0u64;
    for index in 0..request.parts {
        let path = parts_dir.join(format!("{index:05}"));
        let len = fs::metadata(&path)
            .await
            .map_err(|_| ApiError::bad(format!("part {index} has not been uploaded")))?
            .len();
        total += len;
        parts.push(path);
    }
    if total != request.size {
        return Err(ApiError::bad(format!(
            "the parts add up to {total} bytes, not {}",
            request.size
        )));
    }
    let joined = store::assemble(root, &parts).await?;
    if joined.sha256 != request.sha256 || joined.size != request.size {
        joined.discard().await;
        return Err(ApiError::bad(
            "the joined file does not match the sha256 you gave",
        ));
    }
    let sha = joined.sha256.clone();
    store::commit(&joined.tmp, &dest).await?;
    let _ = fs::remove_dir_all(&parts_dir).await;

    if let Some(version_dir) = parts_dir.parent() {
        let _ = fs::remove_dir(version_dir).await;
    }
    log::info!(
        "published release file {version}/{name} from {} parts ({total} bytes, sha256 {sha})",
        request.parts
    );
    ok(
        StatusCode::CREATED,
        format!("published {total} bytes, sha256 {sha}"),
    )
}

pub async fn put_feed(
    state: &AppState,
    channel: &str,
    file: &str,
    body: Body,
) -> Result<Outcome, ApiError> {
    if !store::slug_ok(channel) {
        return Err(ApiError::bad("bad channel name"));
    }
    let target = store::feed_target(file)
        .ok_or_else(|| ApiError::bad("the file must be named <target>.json"))?;
    let bytes = store::read_capped(body, MAX_DOCUMENT).await?;
    let text = str::from_utf8(&bytes).map_err(|_| ApiError::bad("not UTF-8"))?;
    let signed = Signed::parse(text).map_err(|e| ApiError::bad(e.to_string()))?;
    let payload = signed
        .verify(state.keys(), KIND_UPDATE)
        .map_err(|e| ApiError::bad(format!("the feed is not signed by a trusted key: {e}")))?;
    let manifest: UpdateManifest = serde_json::from_str(payload)
        .map_err(|e| ApiError::bad(format!("unreadable feed: {e}")))?;

    let _guard = state.publish_lock().lock().await;
    let path = state
        .config()
        .root
        .join("v1")
        .join("channels")
        .join(channel)
        .join(file);
    let last_serial = match fs::read_to_string(&path).await {
        Ok(existing) => Signed::parse(&existing)
            .ok()
            .and_then(|s| serde_json::from_str::<UpdateManifest>(&s.payload).ok())
            .map_or(0, |m| m.serial),
        Err(_) => 0,
    };

    let nothing_installed = Version::new(0, 0, 0);
    match manifest.check(&Policy {
        current: &nothing_installed,
        channel,
        target,
        now: now(),
        last_serial,
    }) {
        Ok(Decision::Update { .. }) => {}
        Ok(Decision::UpToDate) => return Err(ApiError::bad("the version must be above 0.0.0")),
        Err(Error::OlderThanSeen { found, seen }) => {
            return Err(ApiError::Conflict(format!(
                "serial {found} is older than the published {seen}"
            )));
        }
        Err(error) => return Err(ApiError::bad(error.to_string())),
    }

    let own = format!("{}/v1/releases/", state.config().public_base);
    for artifact in &manifest.artifacts {
        for url in artifact.urls.iter().filter(|u| u.starts_with(&own)) {
            let (version, name) = url[own.len()..]
                .split_once('/')
                .ok_or_else(|| ApiError::bad(format!("{url} is not a release file")))?;
            if store::release_version(version).is_none() || !store::release_name_ok(name) {
                return Err(ApiError::bad(format!("{url} is not a release file")));
            }
            let file = state
                .config()
                .root
                .join("v1")
                .join("releases")
                .join(version)
                .join(name);
            let (hash, size) = store::file_hash(&file)
                .await
                .map_err(|_| ApiError::bad(format!("{url} has not been uploaded")))?;
            if hash != artifact.sha256 || size != artifact.size {
                return Err(ApiError::bad(format!(
                    "{url} does not match the hash and size in the feed"
                )));
            }
        }
    }

    store::write_atomic(&state.config().root, &path, &bytes).await?;
    log::info!(
        "published feed {channel}/{file}: version {}, serial {}",
        manifest.version,
        manifest.serial
    );
    ok(
        StatusCode::OK,
        format!(
            "published {channel}/{file}: version {}, serial {}",
            manifest.version, manifest.serial
        ),
    )
}

pub async fn put_identity(state: &AppState, body: Body) -> Result<Outcome, ApiError> {
    let bytes = store::read_capped(body, MAX_DOCUMENT).await?;
    let text = str::from_utf8(&bytes).map_err(|_| ApiError::bad("not UTF-8"))?;
    let signed = Signed::parse(text).map_err(|e| ApiError::bad(e.to_string()))?;
    let payload = signed
        .verify(state.keys(), KIND_IDENTITY)
        .map_err(|e| ApiError::bad(format!("the identity is not signed by a trusted key: {e}")))?;
    let identity: serde_json::Value = serde_json::from_str(payload)
        .map_err(|e| ApiError::bad(format!("unreadable identity: {e}")))?;
    for platform in ["windows", "linux", "osx"] {
        let entry = identity
            .get(platform)
            .ok_or_else(|| ApiError::bad(format!("no `{platform}` entry")))?;
        for field in ["host_version", "electron_version", "chrome_version"] {
            if !entry
                .get(field)
                .is_some_and(|v| v.as_str().is_some_and(|s| !s.is_empty()))
            {
                return Err(ApiError::bad(format!("`{platform}` has no `{field}`")));
            }
        }
    }
    let _guard = state.publish_lock().lock().await;
    store::write_atomic(
        &state.config().root,
        &state.config().root.join("v1").join("identity.json"),
        &bytes,
    )
    .await?;
    log::info!("published identity");
    ok(StatusCode::OK, "published identity")
}
