use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use requiem_update::{ExtractRules, extract_zip};
use semver::Version;
use tokio::sync::Notify;

use crate::check::{self, Checked, Config};
use crate::download::{self, DownloadError};
use crate::sources::Source;
use crate::staged::{self, Staged};
use std::fs;
use std::time as std_time;
use std::time::SystemTime;
use tokio::runtime;
use tokio::task;
use tokio::time;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Idle,
    Checking,
    Downloading {
        version: String,
        done: u64,
        total: u64,
    },

    Ready(Staged),
    UpToDate,
    Failed(String),
}

pub struct Updater {
    pub client: wreq::Client,
    pub config: Config,

    pub root: PathBuf,

    pub exe_name: String,
    pub db: requiem_db::Db,
}

const AUTO_KEY: &str = "update.auto";
const SKIP_KEY: &str = "update.skip";

pub fn skip_version(db: &requiem_db::Db, version: &str) {
    let _ = db.set_meta(SKIP_KEY, version);
}

pub fn auto_update(db: &requiem_db::Db) -> bool {
    db.meta(AUTO_KEY).is_none_or(|v| v != "0")
}

pub fn set_auto_update(db: &requiem_db::Db, on: bool) {
    let _ = db.set_meta(AUTO_KEY, if on { "1" } else { "0" });
}

impl Updater {
    pub fn staged(&self) -> Option<Staged> {
        staged::load(&self.root, &self.config.current)
    }

    fn rules(&self) -> ExtractRules {
        ExtractRules {
            required_file: self.exe_name.clone(),
            forbidden_top: vec!["data".into(), ".update".into()],
            max_files: 4_000,
            max_total: 2 << 30,
        }
    }

    pub async fn run_once(&self, sources: &[Source], on_state: &(dyn Fn(State) + Sync)) -> State {
        let end = |state: State| {
            on_state(state.clone());
            state
        };
        if sources.is_empty() {
            return State::Idle;
        }
        on_state(State::Checking);
        let offer = match check::check(&self.client, sources, &self.config, &self.db).await {
            Checked::Unavailable => {
                return end(State::Failed("no update server could be reached".into()));
            }
            Checked::UpToDate => {
                return match self.staged() {
                    Some(staged) => end(State::Ready(staged)),
                    None => end(State::UpToDate),
                };
            }
            Checked::Available(offer) => offer,
        };
        let version = offer.version.to_string();
        if self.db.meta(SKIP_KEY).as_deref() == Some(version.as_str()) {
            return end(State::UpToDate);
        }
        if let Some(staged) = self.staged().filter(|s| s.version == version) {
            return end(State::Ready(staged));
        }

        staged::discard(&self.root);
        let zip_name = format!("{version}.zip");

        if let Ok(entries) = fs::read_dir(&self.root) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".part") && name != format!("{zip_name}.part") {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
        let progress = |done, total| {
            on_state(State::Downloading {
                version: version.clone(),
                done,
                total,
            })
        };
        progress(0, offer.artifact.size);
        let zip = match download::download(
            &self.client,
            &offer.artifact,
            &self.root,
            &zip_name,
            &progress,
        )
        .await
        {
            Ok(path) => path,
            Err(DownloadError::AllFailed(reasons)) => {
                return end(State::Failed(format!(
                    "download failed: {}",
                    reasons.join("; ")
                )));
            }
            Err(e) => return end(State::Failed(e.to_string())),
        };

        let dir = self.root.join(&version);
        let _ = fs::remove_dir_all(&dir);
        let rules = self.rules();
        let extracted = {
            let (zip, dir) = (zip.clone(), dir.clone());
            task::spawn_blocking(move || extract_zip(&zip, &dir, &rules)).await
        };
        let _ = fs::remove_file(&zip);
        match extracted {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                return end(State::Failed(format!(
                    "the downloaded update was rejected: {e}"
                )));
            }
            Err(e) => return end(State::Failed(e.to_string())),
        }
        let staged = Staged {
            version,
            dir,
            notes_url: offer.notes_url,
            source: offer.source,
        };
        if let Err(e) = staged::save(&self.root, &staged) {
            return end(State::Failed(format!("could not stage the update: {e}")));
        }
        end(State::Ready(staged))
    }
}

pub struct Handle {
    check_now: Notify,
    auto: AtomicBool,
}

impl Handle {
    pub fn check_now(&self) {
        self.check_now.notify_one();
    }

    pub fn set_auto(&self, on: bool) {
        self.auto.store(on, Ordering::Relaxed);
    }
}

const FIRST_DELAY: Duration = Duration::from_secs(20);
const INTERVAL: Duration = Duration::from_secs(60 * 60);

pub fn spawn(
    runtime: &runtime::Handle,
    updater: Arc<Updater>,
    sources: impl Fn() -> Vec<Source> + Send + Sync + 'static,
    on_state: impl Fn(State) + Send + Sync + 'static,
) -> Arc<Handle> {
    let handle = Arc::new(Handle {
        check_now: Notify::new(),
        auto: AtomicBool::new(auto_update(&updater.db)),
    });
    let task = handle.clone();
    runtime.spawn(async move {
        if let Some(staged) = updater.staged() {
            on_state(State::Ready(staged));
        }
        let mut first = true;
        loop {
            let wait = if first {
                FIRST_DELAY
            } else {
                INTERVAL + jitter()
            };
            first = false;
            let manual = tokio::select! {
                _ = time::sleep(wait) => false,
                _ = task.check_now.notified() => true,
            };
            if !manual && !task.auto.load(Ordering::Relaxed) {
                continue;
            }
            updater.run_once(&sources(), &on_state).await;
        }
    });
    handle
}

fn jitter() -> Duration {
    let nanos = SystemTime::now()
        .duration_since(std_time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    Duration::from_secs(u64::from(nanos) % 600)
}

pub fn parse_version(text: &str) -> Version {
    Version::parse(text.trim_start_matches('v')).unwrap_or_else(|_| Version::new(0, 0, 0))
}
