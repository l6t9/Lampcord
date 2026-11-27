use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;
use std::io::Error as IoError;
use std::io;
use std::path::{Path, PathBuf};
use std::result;
use std::sync::{Arc, OnceLock};
use std::sync;
use std::time::Duration;
use tokio::fs as tokio_fs;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use requiem_assets::extract::{
    chunk_of, extract_svg_verified, extract_text_verified, verify_chunk,
};
use requiem_assets::sha256_hex;

pub use requiem_assets::manifest::EntryKind;
pub use requiem_assets::manifest::Manifest;
use requiem_assets::manifest::{ChunkRef, ClientInfo};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("network: {0}")]
    Http(#[from] wreq::Error),
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("bad manifest: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Recipes(#[from] requiem_assets::Error),
    #[error("GET {url} returned HTTP {status}")]
    Status { url: String, status: u16 },
    #[error(
        "{url}: content does not match the manifest's SHA-256 (expected {expected}, got {actual})"
    )]
    HashMismatch {
        url: String,
        expected: String,
        actual: String,
    },
    #[error("no manifest: {0}")]
    NoManifest(String),
    #[error("manifest has no asset named `{0}`")]
    UnknownAsset(String),
    #[error("font conversion failed: {0}")]
    Font(String),
}

pub type Result<T, E = Error> = result::Result<T, E>;

#[derive(Debug, Clone)]
pub struct Config {
    pub embedded_manifest: Option<&'static str>,

    pub manifest_url: Option<String>,

    pub app_version: String,
    pub cache_dir: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestSource {
    Embedded,

    Remote,

    Cache,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Manifest,
    Chunks,
    Fonts,

    Files,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    pub stage: Stage,

    pub done: usize,
    pub total: usize,
}

#[derive(Debug)]
struct FileRef {
    url: String,
    sha256: Option<String>,
}

pub struct AssetStore {
    file_memory: sync::Mutex<HashMap<String, Arc<[u8]>>>,
    manifest: Manifest,
    source: ManifestSource,
    dir: PathBuf,

    svgs: HashMap<String, Arc<str>>,
    texts: HashMap<String, Arc<str>>,
    files: HashMap<String, FileRef>,

    problems: Vec<(String, String)>,
}

impl AssetStore {
    pub async fn load(client: &wreq::Client, config: &Config) -> Result<Self> {
        Self::load_with_progress(client, config, |_| {}).await
    }

    pub async fn load_with_progress(
        client: &wreq::Client,
        config: &Config,
        on_progress: impl Fn(Progress),
    ) -> Result<Self> {
        on_progress(Progress {
            stage: Stage::Manifest,
            done: 0,
            total: 1,
        });
        let (manifest, source) = fetch_manifest(client, config).await?;
        on_progress(Progress {
            stage: Stage::Manifest,
            done: 1,
            total: 1,
        });
        Self::load_manifest(client, &config.cache_dir, manifest, source, on_progress).await
    }

    pub async fn load_manifest(
        client: &wreq::Client,
        cache_dir: &Path,
        manifest: Manifest,
        source: ManifestSource,
        on_progress: impl Fn(Progress),
    ) -> Result<Self> {
        let chunks = fetch_chunks(client, cache_dir, &manifest.chunks, &on_progress).await?;
        Self::from_chunks(manifest, source, &chunks, cache_dir.to_path_buf())
    }

    pub async fn warm_fonts(
        &self,
        client: &wreq::Client,
        on_progress: impl Fn(Progress),
    ) -> Result<()> {
        let names: Vec<&String> = self
            .files
            .keys()
            .filter(|n| n.starts_with("font/"))
            .collect();
        on_progress(Progress {
            stage: Stage::Fonts,
            done: 0,
            total: names.len(),
        });
        for (i, name) in names.iter().enumerate() {
            self.font_ttf(client, name).await?;
            on_progress(Progress {
                stage: Stage::Fonts,
                done: i + 1,
                total: names.len(),
            });
        }
        Ok(())
    }

    pub fn from_chunks(
        manifest: Manifest,
        source: ManifestSource,
        chunks: &[Vec<u8>],
        dir: PathBuf,
    ) -> Result<Self> {
        let mut svgs = HashMap::new();
        let mut texts = HashMap::new();
        let mut files = HashMap::new();
        let mut problems = Vec::new();

        let mut verified = HashMap::new();
        for entry in &manifest.entries {
            if let Some(index) = chunk_of(entry) {
                if let Entry::Vacant(e) = verified.entry(index) {
                    let bytes = chunks.get(index as usize).ok_or_else(|| {
                        requiem_assets::Error::Unresolved(format!("no bytes for chunk {index}"))
                    })?;
                    e.insert(verify_chunk(&manifest, index, bytes)?);
                }
            }
        }

        for entry in &manifest.entries {
            match &entry.kind {
                EntryKind::SvgComponent { chunk, .. } => {
                    match extract_svg_verified(entry, &verified[chunk]) {
                        Ok(icon) => match icon.to_svg() {
                            Ok(svg) => {
                                svgs.insert(entry.name.clone(), Arc::from(svg));
                            }
                            Err(e) => problems.push((entry.name.clone(), e.to_string())),
                        },
                        Err(e) => problems.push((entry.name.clone(), e.to_string())),
                    }
                }
                EntryKind::Text { chunk, .. } => {
                    match extract_text_verified(entry, &verified[chunk]) {
                        Ok(text) => {
                            texts.insert(entry.name.clone(), Arc::from(text));
                        }
                        Err(e) => problems.push((entry.name.clone(), e.to_string())),
                    }
                }
                EntryKind::File { url, sha256 } => {
                    files.insert(
                        entry.name.clone(),
                        FileRef {
                            url: url.clone(),
                            sha256: sha256.clone(),
                        },
                    );
                }
            }
        }
        Ok(Self {
            file_memory: Default::default(),
            manifest,
            source,
            dir,
            svgs,
            texts,
            files,
            problems,
        })
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn client_info(&self) -> &ClientInfo {
        &self.manifest.client
    }

    pub fn manifest_source(&self) -> ManifestSource {
        self.source
    }

    pub fn problems(&self) -> &[(String, String)] {
        &self.problems
    }

    pub fn svg(&self, name: &str) -> Option<&str> {
        self.svgs.get(name).map(|s| &**s)
    }

    pub fn text(&self, name: &str) -> Option<&str> {
        self.texts.get(name).map(|t| &**t)
    }

    pub fn lottie(&self, name: &str) -> Option<&str> {
        self.text(name)
    }

    pub fn has_file(&self, name: &str) -> bool {
        self.files.contains_key(name)
    }

    fn file_ref(&self, name: &str) -> Result<&FileRef> {
        self.files
            .get(name)
            .ok_or_else(|| Error::UnknownAsset(name.to_string()))
    }

    pub async fn file(&self, client: &wreq::Client, name: &str) -> Result<Vec<u8>> {
        let file = self.file_ref(name)?;
        fetch_verified(
            client,
            &self.dir.join("files"),
            &file.url,
            file.sha256.as_deref(),
        )
        .await
    }

    pub async fn warm_files(
        &self,
        client: &wreq::Client,
        on_progress: impl Fn(Progress),
    ) -> Result<()> {
        let wanted: Vec<(String, Option<String>)> = self
            .files
            .iter()
            .filter(|(name, _)| !name.starts_with("font/"))
            .map(|(_, f)| (f.url.clone(), f.sha256.clone()))
            .collect();
        let total = wanted.len();
        on_progress(Progress {
            stage: Stage::Files,
            done: 0,
            total,
        });
        let permits = Arc::new(Semaphore::new(6));
        let mut set = JoinSet::new();
        for (url, sha256) in wanted {
            let (client, dir, permits) = (client.clone(), self.dir.join("files"), permits.clone());
            set.spawn(async move {
                let _permit = permits.acquire_owned().await.expect("semaphore open");
                fetch_verified(&client, &dir, &url, sha256.as_deref())
                    .await
                    .map(|_| ())
            });
        }
        let mut done = 0;
        while let Some(joined) = set.join_next().await {
            joined.map_err(|e| IoError::other(e.to_string()))??;
            done += 1;
            on_progress(Progress {
                stage: Stage::Files,
                done,
                total,
            });
        }
        Ok(())
    }

    pub fn file_bytes(&self, name: &str) -> Option<Arc<[u8]>> {
        if let Some(bytes) = self
            .file_memory
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(name)
        {
            return Some(bytes.clone());
        }
        let bytes: Arc<[u8]> = self.file_if_cached(name)?.into();
        self.file_memory
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(name.to_string(), bytes.clone());
        Some(bytes)
    }

    pub fn cached_path(&self, name: &str) -> Option<PathBuf> {
        cached_file(&self.dir, &self.manifest, name)
    }

    pub fn file_if_cached(&self, name: &str) -> Option<Vec<u8>> {
        let file = self.files.get(name)?;
        let path = self.dir.join("files").join(file.sha256.as_deref()?);
        let bytes = fs::read(path).ok()?;
        (sha256_hex(&bytes) == file.sha256.as_deref()?).then_some(bytes)
    }

    pub async fn font_ttf(&self, client: &wreq::Client, name: &str) -> Result<Vec<u8>> {
        let file = self.file_ref(name)?;
        let woff2 = fetch_verified(
            client,
            &self.dir.join("files"),
            &file.url,
            file.sha256.as_deref(),
        )
        .await?;
        let key = sha256_hex(&woff2);
        let cached = self.dir.join("fonts").join(format!("{key}.ttf"));
        if let Ok(ttf) = tokio_fs::read(&cached).await {
            return Ok(ttf);
        }
        let ttf = woff2_to_ttf(&woff2)?;
        write_atomic(&cached, &ttf).await?;
        Ok(ttf)
    }
}

pub fn woff2_to_ttf(woff2: &[u8]) -> Result<Vec<u8>> {
    wuff::decompress_woff2(woff2).map_err(|e| Error::Font(format!("{e:?}")))
}

static GLOBAL: OnceLock<AssetStore> = OnceLock::new();

pub type RequestHook = fn(wreq::RequestBuilder, &str) -> wreq::RequestBuilder;
static REQUEST_HOOK: OnceLock<RequestHook> = OnceLock::new();

pub fn set_request_hook(hook: RequestHook) {
    let _ = REQUEST_HOOK.set(hook);
}

fn shaped(builder: wreq::RequestBuilder, url: &str) -> wreq::RequestBuilder {
    match REQUEST_HOOK.get() {
        Some(hook) => hook(builder, url),
        None => builder,
    }
}

pub fn install(store: AssetStore) -> Result<&'static AssetStore, AssetStore> {
    GLOBAL.set(store)?;
    Ok(GLOBAL.get().expect("just installed"))
}

pub fn get() -> Option<&'static AssetStore> {
    GLOBAL.get()
}

pub async fn fetch_manifest(
    client: &wreq::Client,
    config: &Config,
) -> Result<(Manifest, ManifestSource)> {
    let mut override_failure = None;
    if let Some(url) = &config.manifest_url {
        let cache_path = config
            .cache_dir
            .join(format!("manifest-{}.json", sanitize(&config.app_version)));
        let fetched = async {
            let response = client
                .get(url)
                .timeout(Duration::from_secs(8))
                .send()
                .await?;
            if !response.status().is_success() {
                return Err(Error::Status {
                    url: url.clone(),
                    status: response.status().as_u16(),
                });
            }
            Ok::<_, Error>(response.bytes().await?)
        }
        .await;
        match fetched {
            Ok(bytes) => {
                let manifest: Manifest = serde_json::from_slice(&bytes)?;
                write_atomic(&cache_path, &bytes).await?;
                return Ok((manifest, ManifestSource::Remote));
            }
            Err(network) => {
                if let Ok(bytes) = tokio_fs::read(&cache_path).await {
                    return Ok((serde_json::from_slice(&bytes)?, ManifestSource::Cache));
                }
                override_failure = Some(network.to_string());
            }
        }
    }
    match config.embedded_manifest {
        Some(json) => Ok((serde_json::from_str(json)?, ManifestSource::Embedded)),
        None => Err(Error::NoManifest(override_failure.unwrap_or_else(|| {
            "this build embeds no manifest and no override URL is set".into()
        }))),
    }
}

async fn fetch_chunks(
    client: &wreq::Client,
    dir: &Path,
    chunks: &[ChunkRef],
    on_progress: &impl Fn(Progress),
) -> Result<Vec<Vec<u8>>> {
    let total = chunks.len();
    on_progress(Progress {
        stage: Stage::Chunks,
        done: 0,
        total,
    });
    let permits = Arc::new(Semaphore::new(6));
    let mut set = JoinSet::new();
    for (index, chunk) in chunks.iter().cloned().enumerate() {
        let (client, dir, permits) = (client.clone(), dir.to_path_buf(), permits.clone());
        set.spawn(async move {
            let _permit = permits.acquire_owned().await.expect("semaphore open");
            (
                index,
                fetch_verified(
                    &client,
                    &dir.join("chunks"),
                    &chunk.url,
                    Some(&chunk.sha256),
                )
                .await,
            )
        });
    }
    let mut bytes: Vec<Option<Vec<u8>>> = vec![None; total];
    let mut done = 0;
    while let Some(joined) = set.join_next().await {
        let (index, result) = joined.map_err(|e| IoError::other(e.to_string()))?;
        bytes[index] = Some(result?);
        done += 1;
        on_progress(Progress {
            stage: Stage::Chunks,
            done,
            total,
        });
    }
    Ok(bytes
        .into_iter()
        .map(|b| b.expect("every chunk finished"))
        .collect())
}

pub async fn fetch_file(
    client: &wreq::Client,
    cache_dir: &Path,
    manifest: &Manifest,
    name: &str,
) -> Result<PathBuf> {
    let (url, sha256) = file_entry(manifest, name)?;
    let bytes = fetch_verified(client, &cache_dir.join("files"), url, sha256).await?;
    Ok(cache_dir.join("files").join(sanitize(&sha256_hex(&bytes))))
}

pub fn cached_file(cache_dir: &Path, manifest: &Manifest, name: &str) -> Option<PathBuf> {
    let (_, sha256) = file_entry(manifest, name).ok()?;
    let path = cache_dir.join("files").join(sanitize(sha256?));
    let bytes = fs::read(&path).ok()?;
    (sha256_hex(&bytes) == sha256?).then_some(path)
}

fn file_entry<'m>(manifest: &'m Manifest, name: &str) -> Result<(&'m str, Option<&'m str>)> {
    match manifest.entry(name).map(|e| &e.kind) {
        Some(EntryKind::File { url, sha256 }) => Ok((url, sha256.as_deref())),
        _ => Err(Error::UnknownAsset(name.to_string())),
    }
}

pub fn cache_ready(manifest: &Manifest, cache_dir: &Path) -> bool {
    let chunks_present = manifest.chunks.iter().all(|c| {
        fs::metadata(cache_dir.join("chunks").join(sanitize(&c.sha256)))
            .is_ok_and(|m| m.len() == c.size)
    });
    let fonts_present = manifest
        .entries
        .iter()
        .filter(|e| e.name.starts_with("font/"))
        .all(|e| match &e.kind {
            EntryKind::File {
                sha256: Some(sha), ..
            } => cache_dir
                .join("fonts")
                .join(format!("{}.ttf", sanitize(sha)))
                .is_file(),
            _ => false,
        });
    let files_present = manifest
        .entries
        .iter()
        .filter(|e| !e.name.starts_with("font/"))
        .all(|e| match &e.kind {
            EntryKind::File {
                sha256: Some(sha), ..
            } => cache_dir.join("files").join(sanitize(sha)).is_file(),
            EntryKind::File { sha256: None, .. } => false,
            _ => true,
        });
    chunks_present && fonts_present && files_present
}

async fn fetch_verified(
    client: &wreq::Client,
    dir: &Path,
    url: &str,
    sha256: Option<&str>,
) -> Result<Vec<u8>> {
    if let Some(expected) = sha256 {
        let path = dir.join(sanitize(expected));
        if let Ok(bytes) = tokio_fs::read(&path).await {
            if sha256_hex(&bytes) == expected {
                return Ok(bytes);
            }

            let _ = tokio_fs::remove_file(&path).await;
        }
    }
    let response = shaped(client.get(url), url).send().await?;
    if !response.status().is_success() {
        return Err(Error::Status {
            url: url.to_string(),
            status: response.status().as_u16(),
        });
    }
    let bytes = response.bytes().await?.to_vec();
    let actual = sha256_hex(&bytes);
    if let Some(expected) = sha256 {
        if actual != expected {
            return Err(Error::HashMismatch {
                url: url.to_string(),
                expected: expected.to_string(),
                actual,
            });
        }
    }
    write_atomic(&dir.join(&actual), &bytes).await?;
    Ok(bytes)
}

async fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        tokio_fs::create_dir_all(parent).await?;
    }
    let partial = path.with_extension("partial");
    tokio_fs::write(&partial, bytes).await?;
    tokio_fs::rename(&partial, path).await
}

fn sanitize(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs as std_fs;
    use std::process;
    use super::*;
    use requiem_assets::manifest::{BuildRef, Entry, SCHEMA};

    fn manifest_with(entries: Vec<Entry>, chunk_sha: &str) -> Manifest {
        Manifest {
            schema: SCHEMA,
            client: ClientInfo {
                build_number: Some(9),
                build_id: "b".into(),
                release_channel: "stable".into(),
            },
            asset_base: "https://d/assets/".into(),
            builds: vec![BuildRef::new("b", Some(9))],
            chunks: vec![ChunkRef {
                url: "https://d/assets/a.js".into(),
                sha256: chunk_sha.into(),
                size: 1,
            }],
            entries,
        }
    }

    #[test]
    fn a_chunk_that_does_not_match_its_hash_is_rejected() {
        let entry = Entry {
            name: "lottie/x".into(),
            build: 0,
            kind: EntryKind::Text {
                chunk: 0,
                start: 0,
                end: 2,
            },
        };
        let manifest = manifest_with(vec![entry], "0000");
        let err = AssetStore::from_chunks(
            manifest,
            ManifestSource::Cache,
            &[b"''".to_vec()],
            PathBuf::new(),
        )
        .err()
        .expect("must fail");
        assert!(
            matches!(
                err,
                Error::Recipes(requiem_assets::Error::HashMismatch { .. })
            ),
            "{err}"
        );
    }

    #[test]
    fn text_entries_decode_js_string_literals() {
        let chunk = br#"x='{"a":"it\'s"}'"#.to_vec();
        let sha = sha256_hex(&chunk);
        let start = 2u32;
        let entry = Entry {
            name: "lottie/x".into(),
            build: 0,
            kind: EntryKind::Text {
                chunk: 0,
                start,
                end: chunk.len() as u32,
            },
        };
        let store = AssetStore::from_chunks(
            manifest_with(vec![entry], &sha),
            ManifestSource::Cache,
            &[chunk],
            PathBuf::new(),
        )
        .unwrap();
        assert_eq!(store.lottie("lottie/x"), Some(r#"{"a":"it's"}"#));
    }

    #[tokio::test]
    async fn file_cache_is_checked_before_any_network_use() {
        let dir =
            env::temp_dir().join(format!("requiem-asset-store-test-{}", process::id()));
        let _ = std_fs::remove_dir_all(&dir);
        let bytes = b"hello".to_vec();
        let sha = sha256_hex(&bytes);
        std_fs::create_dir_all(dir.join("files")).unwrap();
        std_fs::write(dir.join("files").join(&sha), &bytes).unwrap();

        let entry = Entry {
            name: "image/x".into(),
            build: 0,
            kind: EntryKind::File {
                url: "http://127.0.0.1:1/unreachable".into(),
                sha256: Some(sha),
            },
        };
        let mut manifest = manifest_with(vec![entry], "unused");
        manifest.chunks.clear();
        let store =
            AssetStore::from_chunks(manifest, ManifestSource::Cache, &[], dir.clone()).unwrap();
        assert_eq!(store.file_if_cached("image/x"), Some(bytes.clone()));
        let client = wreq::Client::new();
        assert_eq!(store.file(&client, "image/x").await.unwrap(), bytes);
        assert!(matches!(
            store.file(&client, "image/missing").await,
            Err(Error::UnknownAsset(_))
        ));

        std_fs::write(dir.join("files").join(sha256_hex(b"hello")), b"corrupt").unwrap();
        assert!(store.file_if_cached("image/x").is_none());
        assert!(store.file(&client, "image/x").await.is_err());
        let _ = std_fs::remove_dir_all(&dir);
    }
}
