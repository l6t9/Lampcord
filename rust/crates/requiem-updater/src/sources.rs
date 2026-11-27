use std::time::Duration;

use crate::custom;
use requiem_db::Db;
use requiem_update::{KeySet, Signed};
use std::env;
use tokio::time;
use wreq::Client;

const OFFICIAL_BASES: &[&str] = &["https://requiem.nurmarv.in/api"];

pub const BASE_ENV: &str = "REQUIEM_UPDATE_BASE";

#[cfg(debug_assertions)]
pub const TEST_KEYS_ENV: &str = "REQUIEM_UPDATE_TEST_KEYS";

const TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct Source {
    pub base: String,
    pub keys: KeySet,

    pub official: bool,
}

pub fn configured() -> Vec<Source> {
    configured_from(Db::shared())
}

pub fn configured_from(db: &Db) -> Vec<Source> {
    if let Some(custom) = custom::get(db) {
        return match KeySet::from_entries(custom.keys.iter().cloned()) {
            Ok(keys) => vec![Source {
                base: custom.base,
                keys,
                official: false,
            }],
            Err(err) => {
                log::warn!(
                    "the custom update server's pinned keys are unusable ({err}); no updates will be looked for"
                );
                Vec::new()
            }
        };
    }
    official()
}

pub fn official() -> Vec<Source> {
    let bases: Vec<String> = match env::var(BASE_ENV) {
        Ok(list) => list
            .split(',')
            .map(|b| b.trim().trim_end_matches('/').to_string())
            .filter(|b| !b.is_empty())
            .collect(),
        Err(_) => OFFICIAL_BASES.iter().map(|b| b.to_string()).collect(),
    };
    let keys = trusted_keys();
    bases
        .into_iter()
        .map(|base| Source {
            base,
            keys: keys.clone(),
            official: true,
        })
        .collect()
}

fn trusted_keys() -> KeySet {
    #[allow(unused_mut)]
    let mut keys = requiem_update::official_keys();
    #[cfg(debug_assertions)]
    if let Ok(list) = env::var(TEST_KEYS_ENV) {
        for item in list.split(',').filter_map(|i| i.split_once('=')) {
            if let Err(err) = keys.insert(&requiem_update::KeyEntry {
                id: item.0.trim().to_string(),
                public: item.1.trim().to_string(),
            }) {
                log::warn!("{TEST_KEYS_ENV}: {err}");
            }
        }
    }
    keys
}

pub fn open(body: &str, keys: &KeySet, kind: &str) -> Option<String> {
    let signed = Signed::parse(body).ok()?;
    match signed.verify(keys, kind) {
        Ok(payload) => Some(payload.to_string()),
        Err(err) => {
            log::warn!("rejected a signed {kind}: {err}");
            None
        }
    }
}

pub async fn fetch_one(
    client: &Client,
    source: &Source,
    path: &str,
    kind: &str,
) -> Option<String> {
    let url = format!("{}/v1/{path}", source.base);
    let body = async {
        client
            .get(&url)
            .send()
            .await
            .ok()?
            .error_for_status()
            .ok()?
            .text()
            .await
            .ok()
    };
    let Some(body) = time::timeout(TIMEOUT, body).await.ok().flatten() else {
        log::info!("{url}: unreachable");
        return None;
    };
    open(&body, &source.keys, kind)
}

pub async fn fetch(
    client: &Client,
    sources: &[Source],
    path: &str,
    kind: &str,
) -> Option<String> {
    for source in sources {
        if let Some(payload) = fetch_one(client, source, path, kind).await {
            return Some(payload);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use requiem_update::{KIND_IDENTITY, KIND_UPDATE, KeyEntry, sign_key_from_seed};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use super::*;

    fn key(n: u8) -> requiem_update::SigningKey {
        sign_key_from_seed(&[n; 32])
    }

    fn source_trusting(base: String, k: &requiem_update::SigningKey) -> Source {
        Source {
            base,
            keys: KeySet::from_entries([KeyEntry::new("t", &k.verifying_key())]).unwrap(),
            official: true,
        }
    }

    async fn serve(bodies: Vec<String>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move {
            for body in bodies {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut buf = [0u8; 2048];
                let _ = stream.read(&mut buf).await;
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes()).await;
            }
        });
        base
    }

    #[test]
    fn a_custom_server_replaces_the_official_ones_and_is_believed_only_for_its_pinned_keys() {
        let db = Db::open_in_memory().unwrap();
        let pinned = key(7);
        custom::set(
            &db,
            &custom::CustomSource {
                base: "https://custom.test".into(),
                keys: vec![KeyEntry::new("c", &pinned.verifying_key())],
                confirmed_at: 1,
            },
        );
        let sources = configured_from(&db);
        assert_eq!(sources.len(), 1);
        assert_eq!(
            (sources[0].base.as_str(), sources[0].official),
            ("https://custom.test", false)
        );
        let signed_by_pinned =
            Signed::sign_text(KIND_IDENTITY, "{}".into(), "c", &pinned).to_json();
        assert!(open(&signed_by_pinned, &sources[0].keys, KIND_IDENTITY).is_some());

        let by_other = Signed::sign_text(KIND_IDENTITY, "{}".into(), "c", &key(8)).to_json();
        assert!(open(&by_other, &sources[0].keys, KIND_IDENTITY).is_none());

        custom::clear(&db);
        assert!(
            configured_from(&db).iter().all(|s| s.official),
            "back to the official servers"
        );
    }

    #[test]
    fn open_accepts_only_the_right_signer_and_kind() {
        let k = key(1);
        let keys = KeySet::from_entries([KeyEntry::new("t", &k.verifying_key())]).unwrap();
        let body = Signed::sign_text(KIND_IDENTITY, "{\"x\":1}".into(), "t", &k).to_json();
        assert_eq!(
            open(&body, &keys, KIND_IDENTITY).as_deref(),
            Some("{\"x\":1}")
        );
        assert_eq!(open(&body, &keys, KIND_UPDATE), None, "wrong kind");
        let other = Signed::sign_text(KIND_IDENTITY, "{}".into(), "t", &key(2)).to_json();
        assert_eq!(open(&other, &keys, KIND_IDENTITY), None, "wrong key");
        assert_eq!(open("not json", &keys, KIND_IDENTITY), None);
    }

    #[tokio::test]
    async fn the_first_source_with_a_valid_document_wins() {
        let (trusted, attacker) = (key(1), key(2));
        let forged = Signed::sign_text(
            KIND_IDENTITY,
            "{\"from\":\"attacker\"}".into(),
            "t",
            &attacker,
        )
        .to_json();
        let genuine =
            Signed::sign_text(KIND_IDENTITY, "{\"from\":\"mirror\"}".into(), "t", &trusted)
                .to_json();
        let sources = vec![
            source_trusting(serve(vec![forged]).await, &trusted),
            source_trusting("http://127.0.0.1:9".into(), &trusted),
            source_trusting(serve(vec![genuine]).await, &trusted),
        ];
        let payload = fetch(&Client::new(), &sources, "identity.json", KIND_IDENTITY).await;
        assert_eq!(payload.as_deref(), Some("{\"from\":\"mirror\"}"));
    }

    #[tokio::test]
    async fn no_valid_source_gives_nothing() {
        let trusted = key(1);
        let forged = Signed::sign_text(KIND_IDENTITY, "{}".into(), "t", &key(9)).to_json();
        let sources = vec![source_trusting(serve(vec![forged]).await, &trusted)];
        assert_eq!(
            fetch(&Client::new(), &sources, "identity.json", KIND_IDENTITY).await,
            None
        );
        assert_eq!(
            fetch(&Client::new(), &[], "identity.json", KIND_IDENTITY).await,
            None
        );
    }
}
