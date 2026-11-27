use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use requiem_assets::manifest::{EntryKind, Manifest};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::{Error, Result};
use reqwest::Client;
use reqwest::header;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlHealth {
    pub url: String,
    pub ok: bool,

    pub status: u16,

    pub detail: String,
}

pub fn manifest_urls(manifest: &Manifest) -> BTreeMap<String, Option<u64>> {
    let mut urls = BTreeMap::new();
    for chunk in &manifest.chunks {
        urls.insert(chunk.url.clone(), Some(chunk.size));
    }
    for entry in &manifest.entries {
        if let EntryKind::File { url, .. } = &entry.kind {
            urls.entry(url.clone()).or_insert(None);
        }
    }
    urls
}

async fn check_one(client: &Client, url: String, expected_size: Option<u64>) -> UrlHealth {
    let response = match client.head(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            return UrlHealth {
                url,
                ok: false,
                status: 0,
                detail: format!("request failed: {e}"),
            };
        }
    };
    let status = response.status().as_u16();
    if !response.status().is_success() {
        return UrlHealth {
            url,
            ok: false,
            status,
            detail: format!("HTTP {status}"),
        };
    }

    let declared = response
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());
    if let (Some(expected), Some(actual)) = (expected_size, declared) {
        if expected != actual {
            return UrlHealth {
                url,
                ok: false,
                status,
                detail: format!("size changed: expected {expected} bytes, now {actual}"),
            };
        }
    }
    UrlHealth {
        url,
        ok: true,
        status,
        detail: String::new(),
    }
}

pub async fn check_urls(urls: BTreeMap<String, Option<u64>>) -> Result<Vec<UrlHealth>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (requiem-asset-build health)")
        .timeout(Duration::from_secs(20))
        .build()?;
    let permits = Arc::new(Semaphore::new(16));
    let mut set = JoinSet::new();
    for (url, size) in urls {
        let (client, permits) = (client.clone(), permits.clone());
        set.spawn(async move {
            let _permit = permits.acquire_owned().await.expect("semaphore open");
            check_one(&client, url, size).await
        });
    }
    let mut out = Vec::new();
    while let Some(joined) = set.join_next().await {
        out.push(joined.map_err(|e| Error::Invalid(format!("health check task failed: {e}")))?);
    }
    out.sort_by(|a, b| a.url.cmp(&b.url));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use requiem_assets::manifest::{BuildRef, ChunkRef, ClientInfo, Entry};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn serve() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    return;
                };
                tokio::spawn(async move {
                    let mut buf = [0u8; 1024];
                    let n = socket.read(&mut buf).await.unwrap_or(0);
                    let request = String::from_utf8_lossy(&buf[..n]).to_string();
                    let path = request.split_whitespace().nth(1).unwrap_or("/").to_string();
                    let (status, length) = match path.as_str() {
                        "/ok" => ("200 OK", 12),
                        "/gone" => ("404 Not Found", 0),
                        _ => ("200 OK", 5),
                    };
                    let _ = socket.write_all(format!("HTTP/1.1 {status}\r\nContent-Length: {length}\r\nConnection: close\r\n\r\n").as_bytes()).await;
                });
            }
        });
        format!("http://{addr}")
    }

    fn manifest(base: &str) -> Manifest {
        Manifest {
            schema: 2,
            client: ClientInfo {
                build_number: Some(1),
                build_id: "b".into(),
                release_channel: "stable".into(),
            },
            asset_base: format!("{base}/"),
            builds: vec![BuildRef::new("b", Some(1))],
            chunks: vec![
                ChunkRef {
                    url: format!("{base}/ok"),
                    sha256: "x".into(),
                    size: 12,
                },
                ChunkRef {
                    url: format!("{base}/resized"),
                    sha256: "x".into(),
                    size: 99,
                },
            ],
            entries: vec![
                Entry {
                    name: "f".into(),
                    build: 0,
                    kind: EntryKind::File {
                        url: format!("{base}/gone"),
                        sha256: None,
                    },
                },
                Entry {
                    name: "g".into(),
                    build: 0,
                    kind: EntryKind::File {
                        url: format!("{base}/ok"),
                        sha256: None,
                    },
                },
            ],
        }
    }

    #[test]
    fn urls_are_deduplicated_and_keep_chunk_sizes() {
        let urls = manifest_urls(&manifest("http://x"));
        assert_eq!(urls.len(), 3);
        assert_eq!(urls["http://x/ok"], Some(12));
        assert_eq!(urls["http://x/gone"], None);
    }

    #[tokio::test]
    async fn detects_gone_and_changed_files() {
        let base = serve().await;
        let results = check_urls(manifest_urls(&manifest(&base))).await.unwrap();
        let bad: Vec<_> = results
            .iter()
            .filter(|r| !r.ok)
            .map(|r| (r.url.rsplit('/').next().unwrap().to_string(), r.status))
            .collect();
        assert_eq!(
            bad,
            [("gone".to_string(), 404), ("resized".to_string(), 200)]
        );
        assert!(results.iter().any(|r| r.detail.contains("size changed")));
    }
}
