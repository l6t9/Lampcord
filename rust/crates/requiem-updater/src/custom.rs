use std::time::Duration;

use requiem_update::{KeyEntry, KeysDocument};
use serde::{Deserialize, Serialize};
use tokio::time;

use requiem_db::Db;
#[cfg(test)]
use std::slice;
use wreq::Client;

const KEY: &str = "update.custom";
const MAX_KEYS_BYTES: usize = 64 * 1024;
const MAX_KEYS: usize = 4;
const TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomSource {
    pub base: String,

    pub keys: Vec<KeyEntry>,

    pub confirmed_at: u64,
}

pub fn get(db: &Db) -> Option<CustomSource> {
    serde_json::from_str(&db.meta(KEY)?).ok()
}

pub fn set(db: &Db, source: &CustomSource) {
    let _ = db.set_meta(
        KEY,
        &serde_json::to_string(source).expect("custom source serializes"),
    );
}

pub fn clear(db: &Db) {
    let _ = db.set_meta(KEY, "");
}

pub fn normalize_base(input: &str) -> Result<String, String> {
    let text = input.trim();
    let (scheme, rest) = text
        .split_once("://")
        .ok_or("The address must start with https://")?;
    let loopback = |authority: &str| {
        let host = authority.rsplit_once(':').map_or(authority, |(h, _)| h);
        matches!(host, "localhost" | "127.0.0.1" | "[::1]")
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    match scheme {
        "https" => {}
        "http" if cfg!(debug_assertions) && loopback(authority) => {}
        _ => return Err("The address must start with https://".into()),
    }
    if authority.is_empty() || authority.contains(['@', ' ']) {
        return Err("That is not a server address.".into());
    }
    let path = rest[authority.len()..]
        .split(['?', '#'])
        .next()
        .unwrap_or("")
        .trim_end_matches('/');
    let path = path.strip_suffix("/v1").unwrap_or(path);
    Ok(format!("{scheme}://{authority}{path}"))
}

pub fn host_of(base: &str) -> String {
    base.split_once("://")
        .map_or(base, |(_, rest)| rest)
        .split('/')
        .next()
        .unwrap_or("")
        .to_string()
}

pub async fn fetch_keys(client: &Client, base: &str) -> Result<Vec<KeyEntry>, String> {
    let url = format!("{base}/v1/keys.json");
    let body = async {
        let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status().as_u16()));
        }
        let bytes = response.bytes().await.map_err(|e| e.to_string())?;
        if bytes.len() > MAX_KEYS_BYTES {
            return Err("the answer is too large".into());
        }
        Ok(bytes)
    };
    let bytes = time::timeout(TIMEOUT, body)
        .await
        .map_err(|_| "the server did not answer in time".to_string())??;
    parse_keys(&bytes)
}

fn parse_keys(bytes: &[u8]) -> Result<Vec<KeyEntry>, String> {
    let document: KeysDocument =
        serde_json::from_slice(bytes).map_err(|e| format!("not a keys document ({e})"))?;
    if document.schema != 1 {
        return Err(format!(
            "unsupported keys document version {}",
            document.schema
        ));
    }
    if document.keys.is_empty() || document.keys.len() > MAX_KEYS {
        return Err(format!(
            "it lists {} keys (expected 1 to {MAX_KEYS})",
            document.keys.len()
        ));
    }
    for key in &document.keys {
        key.verifying_key().map_err(|e| e.to_string())?;
    }
    Ok(document.keys)
}

#[cfg(test)]
mod tests {
    use requiem_update::sign_key_from_seed;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use super::*;

    #[test]
    fn addresses_are_tidied_or_refused() {
        assert_eq!(
            normalize_base("https://updates.example.test/").unwrap(),
            "https://updates.example.test"
        );
        assert_eq!(
            normalize_base("  https://updates.example.test/v1/ ").unwrap(),
            "https://updates.example.test"
        );
        assert_eq!(
            normalize_base("https://example.test/app/v1?x=1#y").unwrap(),
            "https://example.test/app"
        );
        assert_eq!(
            normalize_base("https://example.test:8443").unwrap(),
            "https://example.test:8443"
        );
        for bad in [
            "example.test",
            "ftp://example.test",
            "http://example.test",
            "https://",
            "https://user@example.test",
            "https://exa mple.test",
            "",
        ] {
            assert!(normalize_base(bad).is_err(), "{bad:?} should be refused");
        }
    }

    #[test]
    fn plain_http_is_only_for_this_machine_and_only_in_debug_builds() {
        assert_eq!(
            normalize_base("http://127.0.0.1:8080").is_ok(),
            cfg!(debug_assertions)
        );
        assert!(normalize_base("http://192.168.1.5").is_err());
    }

    #[test]
    fn the_host_is_what_a_person_would_type() {
        assert_eq!(
            host_of("https://updates.example.test"),
            "updates.example.test"
        );
        assert_eq!(
            host_of("https://example.test:8443/app"),
            "example.test:8443"
        );
    }

    #[test]
    fn a_confirmed_server_is_stored_and_cleared() {
        let db = Db::open_in_memory().unwrap();
        assert!(get(&db).is_none());
        let source = CustomSource {
            base: "https://x.test".into(),
            keys: vec![KeyEntry::new(
                "k",
                &sign_key_from_seed(&[1; 32]).verifying_key(),
            )],
            confirmed_at: 5,
        };
        set(&db, &source);
        assert_eq!(get(&db), Some(source));
        clear(&db);
        assert!(get(&db).is_none());
    }

    fn doc(keys: &[KeyEntry]) -> Vec<u8> {
        serde_json::to_vec(&KeysDocument {
            schema: 1,
            keys: keys.to_vec(),
        })
        .unwrap()
    }

    #[test]
    fn keys_documents_are_checked() {
        let good = KeyEntry::new("k", &sign_key_from_seed(&[1; 32]).verifying_key());
        assert_eq!(
            parse_keys(&doc(slice::from_ref(&good))).unwrap(),
            vec![good.clone()]
        );
        assert!(parse_keys(&doc(&[])).is_err(), "no keys");
        assert!(
            parse_keys(&doc(&vec![good.clone(); 5])).is_err(),
            "too many keys"
        );
        assert!(
            parse_keys(&doc(&[KeyEntry {
                id: "x".into(),
                public: "AAAA".into()
            }]))
            .is_err(),
            "not a key"
        );
        assert!(parse_keys(b"{\"schema\":2,\"keys\":[]}").is_err());
        assert!(parse_keys(b"nope").is_err());
    }

    async fn serve_once(status: &str, body: Vec<u8>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let status = status.to_string();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 2048];
            let _ = stream.read(&mut buf).await;
            let head = format!(
                "HTTP/1.1 {status}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(head.as_bytes()).await;
            let _ = stream.write_all(&body).await;
        });
        base
    }

    #[tokio::test]
    async fn keys_are_fetched_from_a_server_and_refused_when_it_misbehaves() {
        let good = KeyEntry::new("k", &sign_key_from_seed(&[2; 32]).verifying_key());
        let base = serve_once("200 OK", doc(slice::from_ref(&good))).await;
        assert_eq!(fetch_keys(&Client::new(), &base).await.unwrap(), vec![good]);

        let base = serve_once("404 Not Found", vec![]).await;
        assert!(
            fetch_keys(&Client::new(), &base)
                .await
                .unwrap_err()
                .contains("404")
        );

        let base = serve_once("200 OK", vec![b'x'; MAX_KEYS_BYTES + 1]).await;
        assert!(
            fetch_keys(&Client::new(), &base)
                .await
                .unwrap_err()
                .contains("too large")
        );
    }
}
