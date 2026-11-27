use std::path::PathBuf;

use axum::body;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use requiem_update::{
    Artifact, KIND_IDENTITY, KIND_UPDATE, KeyEntry, KeySet, Signed, SigningKey, UpdateManifest,
    sha256_hex, sign_key_from_seed,
};
use requiem_update_server::auth::StaticToken;
use requiem_update_server::{AppState, Config, router};
use std::env;
use std::fs;
use std::process;
use tower::ServiceExt;

const BASE: &str = "https://updates.example.test/api";
const TOKEN: &str = "test-token";

fn key() -> SigningKey {
    sign_key_from_seed(&[5; 32])
}

fn root(name: &str) -> PathBuf {
    let d = env::temp_dir().join(format!("requiem-update-server-{name}-{}", process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn app(name: &str, max_release: u64) -> (axum::Router, PathBuf) {
    let root = root(name);
    let keys = KeySet::from_entries([KeyEntry::new("k", &key().verifying_key())]).unwrap();
    let state = AppState::new(
        Config {
            root: root.clone(),
            public_base: BASE.into(),
            max_release_bytes: max_release,
            max_part_bytes: max_release,
        },
        keys,
        Box::new(StaticToken(TOKEN.into())),
    );
    (router(state), root)
}

async fn send(
    app: &axum::Router,
    method: Method,
    path: &str,
    token: Option<&str>,
    body: Vec<u8>,
) -> (StatusCode, Vec<u8>) {
    let mut request = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(body)).unwrap())
        .await
        .unwrap();
    let status = response.status();
    (
        status,
        body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
}

async fn put(app: &axum::Router, path: &str, body: Vec<u8>) -> (StatusCode, String) {
    let (status, bytes) = send(app, Method::PUT, path, Some(TOKEN), body).await;
    (status, String::from_utf8_lossy(&bytes).to_string())
}

fn zip_bytes() -> Vec<u8> {
    (0..5000u32).map(|i| (i % 251) as u8).collect()
}

fn manifest(zip: &[u8], serial: u64, expires_at: u64) -> UpdateManifest {
    UpdateManifest {
        schema: 1,
        channel: "stable".into(),
        serial,
        version: "1.2.0".into(),
        released_at: 1,
        expires_at,
        revoked: vec![],
        notes_url: None,
        sources: vec![BASE.into()],
        artifacts: vec![Artifact {
            target: "windows-x64".into(),
            sha256: sha256_hex(zip),
            size: zip.len() as u64,
            urls: vec![
                format!("{BASE}/v1/releases/1.2.0/app.zip"),
                "https://github.com/o/r/releases/download/v1.2.0/app.zip".into(),
            ],
        }],
    }
}

fn signed_feed(m: &UpdateManifest) -> Vec<u8> {
    Signed::sign(KIND_UPDATE, m, "k", &key())
        .unwrap()
        .to_json()
        .into_bytes()
}

const FAR: u64 = u64::MAX / 2;
const FEED: &str = "/api/v1/channels/stable/windows-x64.json";
const ZIP: &str = "/api/v1/releases/1.2.0/app.zip";

#[tokio::test]
async fn publishing_needs_a_valid_token_and_reading_does_not() {
    let (app, _) = app("auth", 1 << 20);
    assert_eq!(
        send(&app, Method::PUT, ZIP, None, zip_bytes()).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        send(&app, Method::PUT, ZIP, Some("wrong"), zip_bytes())
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        send(&app, Method::GET, "/api/health", None, vec![]).await.0,
        StatusCode::OK
    );

    assert_eq!(
        send(&app, Method::GET, ZIP, None, vec![]).await.0,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn a_release_is_stored_served_and_never_changes() {
    let (app, _) = app("release", 1 << 20);
    assert_eq!(put(&app, ZIP, zip_bytes()).await.0, StatusCode::CREATED);
    let (status, body) = send(&app, Method::GET, ZIP, None, vec![]).await;
    assert_eq!((status, body), (StatusCode::OK, zip_bytes()));

    assert_eq!(put(&app, ZIP, zip_bytes()).await.0, StatusCode::OK);
    let mut other = zip_bytes();
    other[0] ^= 1;
    assert_eq!(put(&app, ZIP, other).await.0, StatusCode::CONFLICT);
    assert_eq!(
        send(&app, Method::GET, ZIP, None, vec![]).await.1,
        zip_bytes(),
        "the published file is untouched"
    );
}

#[tokio::test]
async fn release_uploads_are_checked_before_anything_is_written() {
    let (app, root) = app("release-bad", 4000);
    assert_eq!(
        put(&app, ZIP, zip_bytes()).await.0,
        StatusCode::PAYLOAD_TOO_LARGE,
        "larger than the limit"
    );
    for path in [
        "/api/v1/releases/not-a-version/app.zip",
        "/api/v1/releases/1.2.0/app.txt",
        "/api/v1/releases/1.2.0/.hidden.zip",
        "/api/v1/releases/1.2.0/a..b.zip",
        "/api/v1/releases/1.2.0/a%2Fb.zip",
    ] {
        assert_eq!(
            put(&app, path, vec![1, 2, 3]).await.0,
            StatusCode::BAD_REQUEST,
            "{path}"
        );
    }
    assert_eq!(
        put(&app, "/api/v1/releases/1.2.0/empty.zip", vec![])
            .await
            .0,
        StatusCode::BAD_REQUEST,
        "empty file"
    );
    assert!(!root.join("v1").exists(), "nothing was created");
    let leftovers = fs::read_dir(root.join(".tmp"))
        .map(|d| d.count())
        .unwrap_or(0);
    assert_eq!(leftovers, 0, "no temporary files are left behind");
}

#[tokio::test]
async fn a_feed_goes_live_only_after_the_zip_it_names_is_there_and_matches() {
    let (app, _) = app("feed", 1 << 20);
    let zip = zip_bytes();

    let (status, message) = put(&app, FEED, signed_feed(&manifest(&zip, 1, FAR))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{message}");
    assert!(message.contains("has not been uploaded"), "{message}");

    assert_eq!(put(&app, ZIP, zip.clone()).await.0, StatusCode::CREATED);

    let mut wrong = manifest(&zip, 1, FAR);
    wrong.artifacts[0].sha256 = sha256_hex(b"something else");
    let (status, message) = put(&app, FEED, signed_feed(&wrong)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{message}");
    assert!(message.contains("does not match"), "{message}");

    let (status, message) = put(&app, FEED, signed_feed(&manifest(&zip, 1, FAR))).await;
    assert_eq!(status, StatusCode::OK, "{message}");
    let (status, served) = send(&app, Method::GET, FEED, None, vec![]).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        served,
        signed_feed(&manifest(&zip, 1, FAR)),
        "served exactly as signed"
    );

    assert_eq!(
        put(&app, FEED, signed_feed(&manifest(&zip, 1, FAR)))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        put(&app, FEED, signed_feed(&manifest(&zip, 2, FAR)))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        put(&app, FEED, signed_feed(&manifest(&zip, 1, FAR)))
            .await
            .0,
        StatusCode::CONFLICT
    );
}

#[tokio::test]
async fn feeds_that_are_not_acceptable_are_refused_and_the_live_one_stays() {
    let (app, _) = app("feed-bad", 1 << 20);
    let zip = zip_bytes();
    assert_eq!(put(&app, ZIP, zip.clone()).await.0, StatusCode::CREATED);
    assert_eq!(
        put(&app, FEED, signed_feed(&manifest(&zip, 5, FAR)))
            .await
            .0,
        StatusCode::OK
    );
    let live = send(&app, Method::GET, FEED, None, vec![]).await.1;

    let forged = Signed::sign(
        KIND_UPDATE,
        &manifest(&zip, 9, FAR),
        "k",
        &sign_key_from_seed(&[6; 32]),
    )
    .unwrap()
    .to_json()
    .into_bytes();
    let tampered = String::from_utf8(signed_feed(&manifest(&zip, 9, FAR)))
        .unwrap()
        .replace("1.2.0", "9.9.9")
        .into_bytes();
    let expired = signed_feed(&manifest(&zip, 9, 1));
    let other_channel = signed_feed(&UpdateManifest {
        channel: "beta".into(),
        ..manifest(&zip, 9, FAR)
    });
    for (name, body, path) in [
        ("forged", forged, FEED),
        ("tampered", tampered, FEED),
        ("expired", expired, FEED),
        ("channel does not match the path", other_channel, FEED),
        (
            "target does not match the path",
            signed_feed(&manifest(&zip, 9, FAR)),
            "/api/v1/channels/stable/linux-x64.json",
        ),
        ("not json", b"hello".to_vec(), FEED),
        (
            "a bad file name",
            signed_feed(&manifest(&zip, 9, FAR)),
            "/api/v1/channels/stable/..json",
        ),
    ] {
        let (status, message) = put(&app, path, body).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{name}: {message}");
    }
    assert_eq!(
        send(&app, Method::GET, FEED, None, vec![]).await.1,
        live,
        "the live feed was not touched"
    );
}

#[tokio::test]
async fn the_identity_is_checked_and_served() {
    let (app, _) = app("identity", 1 << 20);
    let platform = r#"{"host_version":"1.0.1","native_build_number":1,"electron_version":"42.0.0","chrome_version":"148.0.0.0"}"#;
    let good = format!(r#"{{"windows":{platform},"linux":{platform},"osx":{platform}}}"#);
    let signed = |payload: &str, kind: &str| {
        Signed::sign_text(kind, payload.to_string(), "k", &key())
            .to_json()
            .into_bytes()
    };

    assert_eq!(
        put(&app, "/api/v1/identity.json", signed(&good, KIND_IDENTITY))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        send(&app, Method::GET, "/api/v1/identity.json", None, vec![])
            .await
            .1,
        signed(&good, KIND_IDENTITY)
    );

    assert_eq!(
        put(&app, "/api/v1/identity.json", signed(&good, KIND_UPDATE))
            .await
            .0,
        StatusCode::BAD_REQUEST,
        "wrong kind"
    );
    assert_eq!(
        put(
            &app,
            "/api/v1/identity.json",
            signed(&format!(r#"{{"windows":{platform}}}"#), KIND_IDENTITY)
        )
        .await
        .0,
        StatusCode::BAD_REQUEST,
        "missing platforms"
    );
    let forged = Signed::sign_text(
        KIND_IDENTITY,
        good.clone(),
        "k",
        &sign_key_from_seed(&[8; 32]),
    )
    .to_json()
    .into_bytes();
    assert_eq!(
        put(&app, "/api/v1/identity.json", forged).await.0,
        StatusCode::BAD_REQUEST,
        "wrong key"
    );
}

#[tokio::test]
async fn keys_json_lists_the_trusted_keys() {
    let (app, _) = app("keys", 1 << 20);
    let (status, body) = send(&app, Method::GET, "/api/v1/keys.json", None, vec![]).await;
    assert_eq!(status, StatusCode::OK);
    let doc: requiem_update::KeysDocument = serde_json::from_slice(&body).unwrap();
    assert_eq!(doc.keys, vec![KeyEntry::new("k", &key().verifying_key())]);
}

#[tokio::test]
async fn paths_cannot_escape_or_reach_other_things() {
    let (app, root) = app("paths", 1 << 20);
    fs::write(root.join("secret.txt"), b"private").unwrap();
    for path in [
        "/api/v1/../secret.txt",
        "/api/v1/%2e%2e/secret.txt",
        "/api/v1/..%2fsecret.txt",
        "/api/secret.txt",
        "/secret.txt",
        "/api/.tmp/x",
    ] {
        let (status, body) = send(&app, Method::GET, path, None, vec![]).await;
        assert_ne!(body, b"private".to_vec(), "{path} leaked a file outside v1");
        assert!(
            status == StatusCode::NOT_FOUND || status == StatusCode::BAD_REQUEST,
            "{path} -> {status}"
        );
    }

    assert_eq!(
        put(&app, "/api/v1/anything/else", vec![1]).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        put(&app, "/api/v1/keys.json", vec![1]).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        send(&app, Method::DELETE, ZIP, Some(TOKEN), vec![]).await.0,
        StatusCode::METHOD_NOT_ALLOWED
    );
}

#[tokio::test]
async fn release_files_are_cached_for_good_and_feeds_briefly() {
    let (app, _) = app("cache", 1 << 20);
    let zip = zip_bytes();
    put(&app, ZIP, zip.clone()).await;
    put(&app, FEED, signed_feed(&manifest(&zip, 1, FAR))).await;
    let cache = |path: &'static str| {
        let app = app.clone();
        async move {
            app.oneshot(Request::get(path).body(Body::empty()).unwrap())
                .await
                .unwrap()
                .headers()
                .get("cache-control")
                .map(|v| v.to_str().unwrap().to_string())
        }
    };
    assert_eq!(
        cache(ZIP).await.as_deref(),
        Some("public, max-age=31536000, immutable")
    );
    assert_eq!(cache(FEED).await.as_deref(), Some("public, max-age=60"));
}

fn big() -> Vec<u8> {
    (0..5000u32).map(|i| (i % 253) as u8).collect()
}

fn chunked_app(name: &str) -> (axum::Router, PathBuf) {
    let root = root(name);
    let keys = KeySet::from_entries([KeyEntry::new("k", &key().verifying_key())]).unwrap();

    let state = AppState::new(
        Config {
            root: root.clone(),
            public_base: BASE.into(),
            max_release_bytes: 1 << 20,
            max_part_bytes: 2000,
        },
        keys,
        Box::new(StaticToken(TOKEN.into())),
    );
    (router(state), root)
}

const PARTS: &str = "/api/v1/releases/1.2.0/big.zip/parts";
const COMPLETE: &str = "/api/v1/releases/1.2.0/big.zip/complete";

fn complete_body(data: &[u8], parts: u32) -> Vec<u8> {
    serde_json::to_vec(
        &serde_json::json!({ "parts": parts, "size": data.len(), "sha256": sha256_hex(data) }),
    )
    .unwrap()
}

#[tokio::test]
async fn a_release_too_big_for_one_request_is_sent_in_parts_and_joined_and_checked() {
    let (app, root) = chunked_app("chunks");
    let data = big();

    assert_eq!(
        put(&app, "/api/v1/releases/1.2.0/big.zip", data.clone())
            .await
            .0,
        StatusCode::PAYLOAD_TOO_LARGE
    );

    let chunks: Vec<&[u8]> = data.chunks(2000).collect();
    assert_eq!(chunks.len(), 3);
    for (i, chunk) in chunks.iter().enumerate() {
        assert_eq!(
            put(&app, &format!("{PARTS}/{i}"), chunk.to_vec()).await.0,
            StatusCode::OK
        );
    }
    assert_eq!(
        put(&app, &format!("{PARTS}/1"), chunks[1].to_vec()).await.0,
        StatusCode::OK
    );

    let (status, message) = put(&app, COMPLETE, complete_body(&data, 3)).await;
    assert_eq!(status, StatusCode::CREATED, "{message}");
    assert_eq!(
        send(
            &app,
            Method::GET,
            "/api/v1/releases/1.2.0/big.zip",
            None,
            vec![]
        )
        .await
        .1,
        data,
        "joined in order"
    );
    assert!(
        !root
            .join(".tmp")
            .join("parts")
            .join("1.2.0")
            .join("big.zip")
            .exists(),
        "the parts are cleaned up"
    );

    assert_eq!(
        put(&app, COMPLETE, complete_body(&data, 3)).await.0,
        StatusCode::OK
    );
    let mut other = data.clone();
    other[10] ^= 1;
    assert_eq!(
        put(&app, COMPLETE, complete_body(&other, 3)).await.0,
        StatusCode::CONFLICT
    );
}

#[tokio::test]
async fn parts_are_checked_and_a_wrong_join_publishes_nothing() {
    let (app, root) = chunked_app("chunks-bad");
    let data = big();
    let chunks: Vec<&[u8]> = data.chunks(2000).collect();

    assert_eq!(
        put(&app, &format!("{PARTS}/0"), vec![0u8; 2001]).await.0,
        StatusCode::PAYLOAD_TOO_LARGE,
        "a part over the limit"
    );
    assert_eq!(
        put(&app, &format!("{PARTS}/0"), vec![]).await.0,
        StatusCode::BAD_REQUEST,
        "an empty part"
    );
    for bad in ["x", "-1", "10000", "1.5"] {
        assert_eq!(
            put(&app, &format!("{PARTS}/{bad}"), vec![1]).await.0,
            StatusCode::BAD_REQUEST,
            "part number {bad}"
        );
    }

    put(&app, &format!("{PARTS}/0"), chunks[0].to_vec()).await;
    let (status, message) = put(&app, COMPLETE, complete_body(&data, 3)).await;
    assert_eq!(
        (status, message.contains("part 1 has not been uploaded")),
        (StatusCode::BAD_REQUEST, true),
        "{message}"
    );

    put(&app, &format!("{PARTS}/1"), chunks[1].to_vec()).await;

    let mut altered = chunks[2].to_vec();
    altered[0] ^= 1;
    put(&app, &format!("{PARTS}/2"), altered).await;
    let (status, message) = put(&app, COMPLETE, complete_body(&data, 3)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{message}");
    assert!(message.contains("does not match"), "{message}");
    assert!(!root.join("v1").exists(), "nothing was published");

    let mut wrong_size =
        serde_json::from_slice::<serde_json::Value>(&complete_body(&data, 3)).unwrap();
    wrong_size["size"] = serde_json::json!(4999);
    assert_eq!(
        put(&app, COMPLETE, serde_json::to_vec(&wrong_size).unwrap())
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    for body in [
        r#"{"parts":0,"size":5000,"sha256":"00"}"#,
        r#"{"parts":3,"size":0,"sha256":"00"}"#,
        r#"{"parts":3,"size":5000,"sha256":"XYZ"}"#,
        "nope",
    ] {
        assert_eq!(
            put(&app, COMPLETE, body.as_bytes().to_vec()).await.0,
            StatusCode::BAD_REQUEST,
            "{body}"
        );
    }

    assert_eq!(
        send(&app, Method::PUT, &format!("{PARTS}/0"), None, vec![1])
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        send(&app, Method::PUT, COMPLETE, None, complete_body(&data, 3))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn a_feed_can_name_a_release_that_was_sent_in_parts() {
    let (app, _) = chunked_app("chunks-feed");
    let data = big();
    for (i, chunk) in data.chunks(2000).enumerate() {
        put(&app, &format!("{PARTS}/{i}"), chunk.to_vec()).await;
    }
    assert_eq!(
        put(&app, COMPLETE, complete_body(&data, 3)).await.0,
        StatusCode::CREATED
    );
    let mut m = manifest(&data, 1, FAR);
    m.artifacts[0].urls[0] = format!("{BASE}/v1/releases/1.2.0/big.zip");
    assert_eq!(put(&app, FEED, signed_feed(&m)).await.0, StatusCode::OK);
}

#[tokio::test]
async fn routes_exist_only_under_the_public_base_path() {
    let (app, _) = app("prefix", 1 << 20);
    assert_eq!(
        send(&app, Method::GET, "/api/health", None, vec![]).await.0,
        StatusCode::OK
    );
    for path in ["/health", "/v1/keys.json", "/other/health"] {
        assert_eq!(
            send(&app, Method::GET, path, None, vec![]).await.0,
            StatusCode::NOT_FOUND,
            "{path}"
        );
    }
    assert_eq!(
        put(&app, "/v1/releases/1.2.0/app.zip", vec![1]).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        requiem_update_server::prefix_of("https://example.test/api/"),
        "/api"
    );
    assert_eq!(requiem_update_server::prefix_of("https://example.test"), "");
    assert_eq!(
        requiem_update_server::prefix_of("http://127.0.0.1:8080/a/b"),
        "/a/b"
    );
}
