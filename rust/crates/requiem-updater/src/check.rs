use requiem_update::{Artifact, Decision, Policy, UpdateManifest};
use semver::Version;

use crate::sources::{self, Source};
use std::time;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct Config {
    pub current: Version,
    pub channel: String,

    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    pub version: Version,
    pub artifact: Artifact,
    pub notes_url: Option<String>,

    pub source: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Checked {
    UpToDate,
    Available(Offer),

    Unavailable,
}

fn serial_key(base: &str) -> String {
    format!("update.serial.{base}")
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub async fn check(
    client: &wreq::Client,
    sources: &[Source],
    config: &Config,
    db: &requiem_db::Db,
) -> Checked {
    check_at(client, sources, config, db, unix_now()).await
}

pub async fn check_at(
    client: &wreq::Client,
    sources: &[Source],
    config: &Config,
    db: &requiem_db::Db,
    now: u64,
) -> Checked {
    let path = format!("channels/{}/{}.json", config.channel, config.target);
    let mut best: Option<Offer> = None;
    let mut any_valid = false;
    for source in sources {
        let Some(payload) =
            sources::fetch_one(client, source, &path, requiem_update::KIND_UPDATE).await
        else {
            continue;
        };
        let manifest: UpdateManifest = match serde_json::from_str(&payload) {
            Ok(m) => m,
            Err(err) => {
                log::warn!("{}: unreadable update feed: {err}", source.base);
                continue;
            }
        };
        let last_serial = db
            .meta(&serial_key(&source.base))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let policy = Policy {
            current: &config.current,
            channel: &config.channel,
            target: &config.target,
            now,
            last_serial,
        };
        match manifest.check(&policy) {
            Ok(decision) => {
                any_valid = true;
                if manifest.serial > last_serial {
                    let _ = db.set_meta(&serial_key(&source.base), &manifest.serial.to_string());
                }
                if let Decision::Update { version, artifact } = decision {
                    if best.as_ref().is_none_or(|b| version > b.version) {
                        best = Some(Offer {
                            version,
                            artifact: artifact.clone(),
                            notes_url: manifest.notes_url.clone(),
                            source: source.base.clone(),
                        });
                    }
                }
            }
            Err(err) => log::warn!("{}: update feed rejected: {err}", source.base),
        }
    }
    match best {
        Some(offer) => Checked::Available(offer),
        None if any_valid => Checked::UpToDate,
        None => Checked::Unavailable,
    }
}
