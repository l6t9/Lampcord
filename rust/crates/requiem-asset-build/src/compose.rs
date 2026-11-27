use std::sync::Arc;

use requiem_assets::manifest::{BuildRef, ClientInfo, EntryKind, Manifest};
use requiem_assets::recipes;
use requiem_assets::resolve::{Build, Recipe, resolve, sha256_hex};

use crate::cache::{Cache, DISCORD_ORIGIN, DownloadReport};
use crate::discord::BuildInfo;
use crate::{Error, Result};
use tokio::fs;
use tokio::task;

#[derive(Debug, Clone)]
pub enum BuildSelector {
    Latest,

    Existing(String),
}

impl BuildSelector {
    pub fn parse(value: &str) -> Self {
        if value == "latest" {
            BuildSelector::Latest
        } else {
            BuildSelector::Existing(value.to_string())
        }
    }
}

#[derive(Debug, Clone)]
pub enum ClientSelector {
    SameAsAssets,

    KeepBase,

    Build(BuildSelector),
}

impl ClientSelector {
    pub fn parse(value: &str) -> Self {
        match value {
            "same" => ClientSelector::SameAsAssets,
            "keep" => ClientSelector::KeepBase,
            other => ClientSelector::Build(BuildSelector::parse(other)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Composition {
    pub assets: BuildSelector,
    pub client: ClientSelector,

    pub refresh: Vec<String>,
}

pub trait Progress: Send + Sync {
    fn step(&self, message: &str);
}

impl<F: Fn(&str) + Send + Sync> Progress for F {
    fn step(&self, message: &str) {
        self(message)
    }
}

#[derive(Debug)]
pub struct Summary {
    pub resolved: Vec<String>,

    pub carried_over: usize,
    pub downloads: Option<DownloadReport>,
}

pub fn recipes_to_resolve(base: Option<&Manifest>, refresh: &[String]) -> Vec<Recipe> {
    recipes::all()
        .into_iter()
        .filter(|r| match base {
            None => true,
            Some(base) => {
                base.entry(r.name()).is_none()
                    || refresh.iter().any(|p| r.name().starts_with(p.as_str()))
            }
        })
        .collect()
}

async fn prepare_build(
    cache: &Arc<Cache>,
    selector: &BuildSelector,
    progress: &dyn Progress,
) -> Result<(BuildInfo, DownloadReport)> {
    let info = match selector {
        BuildSelector::Latest => {
            progress.step("Looking up the current Discord build");
            cache.discover_latest().await?
        }
        BuildSelector::Existing(id) => cache.load_build(id)?,
    };
    progress.step(&format!(
        "Build {} (#{}): {} chunks, downloading what is not cached",
        &info.build_id[..info.build_id.len().min(8)],
        info.build_number
            .map(|n| n.to_string())
            .unwrap_or_else(|| "?".into()),
        info.chunks.len()
    ));
    let report = cache
        .ensure_all(&info.chunks, |done, total| {
            if done % 500 == 0 || done == total {
                progress.step(&format!("Downloaded {done}/{total} chunks"));
            }
        })
        .await;
    Ok((info, report))
}

async fn resolve_manifest(
    cache: &Arc<Cache>,
    info: &BuildInfo,
    recipes: Vec<Recipe>,
    progress: &dyn Progress,
) -> Result<Manifest> {
    progress.step(&format!(
        "Indexing chunks and resolving {} recipes",
        recipes.len()
    ));
    let source = BuildRef::new(&info.build_id, info.build_number);
    let (channel, chunks, cache_for_index) =
        (info.channel.clone(), info.chunks.clone(), cache.clone());
    let resolution = task::spawn_blocking(move || {
        let mut build = Build::default();
        for chunk in &chunks {
            let Ok(bytes) = cache_for_index.read(&chunk.name) else {
                continue;
            };
            let url = format!("{DISCORD_ORIGIN}/assets/{}", chunk.name);

            let _ = if chunk.is_css() {
                build.add_css(&url, &bytes)
            } else {
                build.add_chunk(&url, &bytes)
            };
        }
        resolve(
            &build,
            &recipes,
            &source,
            &channel,
            &format!("{DISCORD_ORIGIN}/assets/"),
        )
    })
    .await
    .map_err(|e| Error::Invalid(format!("indexing task failed: {e}")))?;

    if !resolution.failures.is_empty() {
        return Err(Error::Unresolved(resolution.failures));
    }
    let mut manifest = resolution.manifest;

    progress.step("Hashing pinned files");
    for entry in &mut manifest.entries {
        if let EntryKind::File { url, sha256 } = &mut entry.kind {
            let name = url.rsplit('/').next().unwrap_or_default().to_string();

            let path = if url.contains("/assets/") && !url.contains("/assets/content/") {
                cache.ensure(&name).await?
            } else {
                cache.ensure_from(&name, url).await?
            };
            let bytes = fs::read(path).await?;
            *sha256 = Some(sha256_hex(&bytes));
        }
    }
    Ok(manifest)
}

async fn client_for_selector(cache: &Cache, selector: &BuildSelector) -> Result<ClientInfo> {
    let (build_id, build_number, release_channel) = match selector {
        BuildSelector::Latest => {
            let page = cache.latest_page().await?;
            (page.build_id, page.build_number, page.channel)
        }
        BuildSelector::Existing(id) => {
            let info = cache.load_build(id)?;
            (info.build_id, info.build_number, info.channel)
        }
    };
    Ok(ClientInfo {
        build_number,
        build_id,
        release_channel,
    })
}

pub async fn compose_manifest(
    cache: &Arc<Cache>,
    base: Option<&Manifest>,
    composition: &Composition,
    progress: &dyn Progress,
) -> Result<(Manifest, Summary)> {
    let wanted = recipes_to_resolve(base, &composition.refresh);
    let resolved: Vec<String> = wanted.iter().map(|r| r.name().to_string()).collect();
    let carried_over = base.map_or(0, |b| {
        b.entries
            .iter()
            .filter(|e| !resolved.contains(&e.name))
            .count()
    });

    let mut overlay = None;
    let mut downloads = None;
    let mut assets_client = None;
    if !wanted.is_empty() {
        let (info, report) = prepare_build(cache, &composition.assets, progress).await?;
        assets_client = Some(ClientInfo {
            build_number: info.build_number,
            build_id: info.build_id.clone(),
            release_channel: info.channel.clone(),
        });
        overlay = Some(resolve_manifest(cache, &info, wanted, progress).await?);
        downloads = Some(report);
    } else {
        progress.step("Every recipe is already in the base manifest; nothing to resolve");
    }

    let client = match &composition.client {
        ClientSelector::SameAsAssets => assets_client
            .or_else(|| base.map(|b| b.client.clone()))
            .ok_or_else(|| {
                Error::Invalid(
                    "nothing was resolved, so there is no build to take the client identity from"
                        .into(),
                )
            })?,
        ClientSelector::KeepBase => base.map(|b| b.client.clone()).ok_or_else(|| {
            Error::Invalid("there is no base manifest to keep the client identity of".into())
        })?,
        ClientSelector::Build(selector) => client_for_selector(cache, selector).await?,
    };

    let manifest = match (base, overlay) {
        (Some(base), Some(mut overlay)) => {
            overlay.client = client;
            Manifest::compose(base, &overlay)
        }
        (Some(base), None) => Manifest {
            client,
            ..base.clone()
        },
        (None, Some(mut overlay)) => {
            overlay.client = client;
            overlay
        }
        (None, None) => return Err(Error::Invalid("nothing to build".into())),
    };
    Ok((
        manifest,
        Summary {
            resolved,
            carried_over,
            downloads,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use requiem_assets::manifest::{Entry, SCHEMA};

    fn base_with(names: &[&str]) -> Manifest {
        Manifest {
            schema: SCHEMA,
            client: ClientInfo {
                build_number: Some(1),
                build_id: "old".into(),
                release_channel: "stable".into(),
            },
            asset_base: "https://d/assets/".into(),
            builds: vec![BuildRef::new("old", Some(1))],
            chunks: vec![],
            entries: names
                .iter()
                .map(|n| Entry {
                    name: (*n).into(),
                    build: 0,
                    kind: EntryKind::File {
                        url: "u".into(),
                        sha256: None,
                    },
                })
                .collect(),
        }
    }

    #[test]
    fn only_new_recipes_are_resolved_against_a_base() {
        let all = recipes::all().len();
        assert_eq!(recipes_to_resolve(None, &[]).len(), all);

        let names: Vec<String> = recipes::all()
            .iter()
            .map(|r| r.name().to_string())
            .collect();
        let full = base_with(&names.iter().map(String::as_str).collect::<Vec<_>>());
        assert!(recipes_to_resolve(Some(&full), &[]).is_empty());
        let refreshed = recipes_to_resolve(Some(&full), &["lottie/".into()]);
        let lottie = names.iter().filter(|n| n.starts_with("lottie/")).count();
        assert!(lottie > 0);
        assert_eq!(refreshed.len(), lottie);
        assert!(refreshed.iter().all(|r| r.name().starts_with("lottie/")));

        let partial = base_with(&names.iter().skip(2).map(String::as_str).collect::<Vec<_>>());
        assert_eq!(recipes_to_resolve(Some(&partial), &[]).len(), 2);
    }

    #[test]
    fn selectors_parse() {
        assert!(matches!(
            ClientSelector::parse("keep"),
            ClientSelector::KeepBase
        ));
        assert!(matches!(
            ClientSelector::parse("same"),
            ClientSelector::SameAsAssets
        ));
        assert!(matches!(
            ClientSelector::parse("latest"),
            ClientSelector::Build(BuildSelector::Latest)
        ));
        assert!(
            matches!(ClientSelector::parse("abc123"), ClientSelector::Build(BuildSelector::Existing(id)) if id == "abc123")
        );
        assert!(matches!(
            BuildSelector::parse("latest"),
            BuildSelector::Latest
        ));
    }
}
