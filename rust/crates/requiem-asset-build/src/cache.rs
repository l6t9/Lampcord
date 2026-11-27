use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::discord::{self, BuildInfo, ChunkFile};
use crate::{Error, Result};
use reqwest::Client;
use std::fs;
use std::io;
use tokio::fs as tokio_fs;

pub const DISCORD_ORIGIN: &str = "https://discord.com";

pub struct Cache {
    root: PathBuf,
    client: Client,
    origin: String,
}

#[derive(Debug, Default)]
pub struct DownloadReport {
    pub total: usize,
    pub already_cached: usize,

    pub missing: Vec<(String, String)>,
}

impl Cache {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        Self::with_origin(root, DISCORD_ORIGIN)
    }

    pub fn with_origin(root: impl Into<PathBuf>, origin: &str) -> Result<Self> {
        let root = root.into();
        fs::create_dir_all(root.join("files"))?;
        fs::create_dir_all(root.join("builds"))?;
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (asset-service)")
            .build()?;
        Ok(Self {
            root,
            client,
            origin: origin.trim_end_matches('/').to_string(),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn file_path(&self, name: &str) -> PathBuf {
        let safe: String = name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
            .collect();
        self.root.join("files").join(safe)
    }

    pub fn has(&self, name: &str) -> bool {
        self.file_path(name).is_file()
    }

    pub fn read(&self, name: &str) -> io::Result<Vec<u8>> {
        fs::read(self.file_path(name))
    }

    pub async fn ensure(&self, name: &str) -> Result<PathBuf> {
        self.ensure_from(name, &format!("{}/assets/{name}", self.origin))
            .await
    }

    pub async fn ensure_from(&self, name: &str, url: &str) -> Result<PathBuf> {
        let path = self.file_path(name);
        if path.is_file() {
            return Ok(path);
        }
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(Error::Status {
                name: name.to_string(),
                status: response.status().as_u16(),
            });
        }
        let bytes = response.bytes().await?;

        let partial = path.with_extension("partial");
        tokio_fs::write(&partial, &bytes).await?;
        tokio_fs::rename(&partial, &path).await?;
        Ok(path)
    }

    pub async fn ensure_all(
        self: &Arc<Self>,
        chunks: &[ChunkFile],
        on_progress: impl Fn(usize, usize) + Send + Sync,
    ) -> DownloadReport {
        let total = chunks.len();
        let todo: Vec<String> = chunks
            .iter()
            .filter(|c| !self.has(&c.name))
            .map(|c| c.name.clone())
            .collect();
        let already_cached = total - todo.len();
        let mut done = already_cached;
        on_progress(done, total);

        let permits = Arc::new(Semaphore::new(16));
        let mut set = JoinSet::new();
        for name in todo {
            let (cache, permits) = (self.clone(), permits.clone());
            set.spawn(async move {
                let _permit = permits.acquire_owned().await.expect("semaphore open");
                let result = cache
                    .ensure(&name)
                    .await
                    .map(|_| ())
                    .map_err(|e| e.to_string());
                (name, result)
            });
        }
        let mut missing = Vec::new();
        while let Some(joined) = set.join_next().await {
            done += 1;
            on_progress(done, total);
            match joined {
                Ok((name, Err(reason))) => missing.push((name, reason)),
                Ok(_) => {}
                Err(join_error) => missing.push(("(task)".to_string(), join_error.to_string())),
            }
        }
        DownloadReport {
            total,
            already_cached,
            missing,
        }
    }

    pub async fn latest_page(&self) -> Result<discord::PageInfo> {
        let html = self
            .client
            .get(format!("{}/app", self.origin))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(discord::parse_page(&html)?)
    }

    pub async fn discover_latest(&self) -> Result<BuildInfo> {
        let page = self.latest_page().await?;
        let entry = self.ensure(&page.entry_script).await?;
        let script = tokio_fs::read_to_string(entry).await?;
        let info = discord::build_info(page, &script);
        self.save_build(&info)?;
        Ok(info)
    }

    pub fn save_build(&self, info: &BuildInfo) -> Result<()> {
        let path = self
            .root
            .join("builds")
            .join(format!("{}.json", sanitize(&info.build_id)));
        fs::write(path, serde_json::to_vec(info)?)?;
        Ok(())
    }

    pub fn load_build(&self, build_id: &str) -> Result<BuildInfo> {
        let path = self
            .root
            .join("builds")
            .join(format!("{}.json", sanitize(build_id)));
        let bytes = fs::read(&path)
            .map_err(|_| Error::NotFound(format!("build `{build_id}` has not been fetched")))?;
        Ok(serde_json::from_slice(&bytes)?)
    }
}

fn sanitize(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
        .collect()
}
