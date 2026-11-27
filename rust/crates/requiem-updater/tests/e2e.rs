use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Cursor;
use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::sync::{Arc, Mutex};

use requiem_db::Db;
use requiem_update::{
    Artifact, KIND_UPDATE, KeyEntry, KeySet, Signed, SigningKey, UpdateManifest, sha256_hex,
    sign_key_from_seed,
};
use requiem_updater::check::{self, Checked, Config};
use requiem_updater::run::{State, Updater, auto_update, set_auto_update};
use requiem_updater::sources::Source;
use semver::Version;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use wreq::Client;
use zip::CompressionMethod;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn key(n: u8) -> SigningKey {
    sign_key_from_seed(&[n; 32])
}

fn keys(k: &SigningKey) -> KeySet {
    KeySet::from_entries([KeyEntry::new("test", &k.verifying_key())]).unwrap()
}

fn temp(name: &str) -> PathBuf {
    let d = env::temp_dir().join(format!("requiem-updater-{name}-{}", process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn release_zip(extra: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut w = ZipWriter::new(&mut buf);

        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        let base: [(&str, &[u8]); 2] = [
            ("release-1.2.0/app.exe", &[7u8; 20_000]),
            ("release-1.2.0/codec.dll", &[9u8; 30_000]),
        ];
        for (name, bytes) in base.iter().chain(extra.iter()) {
            w.start_file(*name, options).unwrap();
            w.write_all(bytes).unwrap();
        }
        w.finish().unwrap();
    }
    buf.into_inner()
}

#[derive(Default)]
struct Log {
    requests: Mutex<Vec<(String, Option<String>)>>,
}

struct Backend {
    base: String,
    log: Arc<Log>,

    cut: Arc<Mutex<HashMap<String, usize>>>,
}

impl Backend {
    async fn start(make_files: impl FnOnce(&str) -> HashMap<String, Vec<u8>>) -> Backend {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let files = make_files(&base);
        let log = Arc::new(Log::default());
        let cut: Arc<Mutex<HashMap<String, usize>>> = Arc::default();
        let files = Arc::new(files);
        let (l, c) = (log.clone(), cut.clone());
        tokio::spawn(async move {
            loop {
                let (mut stream, _) = listener.accept().await.unwrap();
                let (files, log, cut) = (files.clone(), l.clone(), c.clone());
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 8192];
                    let n = stream.read(&mut buf).await.unwrap_or(0);
                    let request = String::from_utf8_lossy(&buf[..n]).to_string();
                    let path = request.split_whitespace().nth(1).unwrap_or("/").to_string();
                    let range = request.lines().find_map(|l| {
                        l.to_ascii_lowercase()
                            .strip_prefix("range: bytes=")
                            .map(|r| r.trim().to_string())
                    });
                    log.requests
                        .lock()
                        .unwrap()
                        .push((path.clone(), range.clone()));
                    let Some(body) = files.get(&path) else {
                        let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\ncontent-length: 0\r\nconnection: close\r\n\r\n").await;
                        return;
                    };
                    let (status, slice, extra) = match range
                        .as_deref()
                        .and_then(|r| r.strip_suffix('-'))
                        .and_then(|r| r.parse::<usize>().ok())
                    {
                        Some(from) if from < body.len() => (
                            "206 Partial Content",
                            &body[from..],
                            format!(
                                "content-range: bytes {from}-{}/{}\r\n",
                                body.len() - 1,
                                body.len()
                            ),
                        ),
                        _ => ("200 OK", &body[..], String::new()),
                    };
                    let head = format!(
                        "HTTP/1.1 {status}\r\ncontent-length: {}\r\n{extra}connection: close\r\n\r\n",
                        slice.len()
                    );
                    let _ = stream.write_all(head.as_bytes()).await;
                    let send = cut
                        .lock()
                        .unwrap()
                        .remove(&path)
                        .map_or(slice.len(), |n| n.min(slice.len()));
                    let _ = stream.write_all(&slice[..send]).await;
                    let _ = stream.shutdown().await;
                });
            }
        });
        Backend { base, log, cut }
    }

    fn zip_requests(&self) -> Vec<(String, Option<String>)> {
        self.log
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|(p, _)| p.ends_with(".zip"))
            .cloned()
            .collect()
    }
}

struct Release {
    zip: Vec<u8>,
    version: &'static str,
    serial: u64,
}

fn feed(release: &Release, urls: Vec<String>, signer: &SigningKey) -> String {
    let manifest = UpdateManifest {
        schema: 1,
        channel: "stable".into(),
        serial: release.serial,
        version: release.version.into(),
        released_at: 1,
        expires_at: u64::MAX / 2,
        revoked: vec![],
        notes_url: Some("https://example.test/notes".into()),
        sources: vec![],
        artifacts: vec![Artifact {
            target: "windows-x64".into(),
            sha256: sha256_hex(&release.zip),
            size: release.zip.len() as u64,
            urls,
        }],
    };
    Signed::sign(KIND_UPDATE, &manifest, "test", signer)
        .unwrap()
        .to_json()
}

const ZIP_PATH: &str = "/v1/releases/1.2.0/r.zip";
const FEED_PATH: &str = "/v1/channels/stable/windows-x64.json";

struct Setup {
    backend: Backend,
    source: Source,
    updater: Updater,
    root: PathBuf,
}

async fn setup(
    name: &str,
    release: &Release,
    current: &str,
    mutate_files: impl FnOnce(&mut HashMap<String, Vec<u8>>, &str),
) -> Setup {
    let signer = key(1);
    let backend = Backend::start(|base| {
        let mut files = HashMap::new();
        files.insert(
            FEED_PATH.to_string(),
            feed(release, vec![format!("{base}{ZIP_PATH}")], &signer).into_bytes(),
        );
        files.insert(ZIP_PATH.to_string(), release.zip.clone());
        mutate_files(&mut files, base);
        files
    })
    .await;
    let root = temp(name).join(".update");
    let updater = Updater {
        client: Client::new(),
        config: Config {
            current: Version::parse(current).unwrap(),
            channel: "stable".into(),
            target: "windows-x64".into(),
        },
        root: root.clone(),
        exe_name: "app.exe".into(),
        db: Db::open_in_memory().unwrap(),
    };
    Setup {
        source: Source {
            base: backend.base.clone(),
            keys: keys(&signer),
            official: true,
        },
        backend,
        updater,
        root,
    }
}

fn states() -> (Arc<Mutex<Vec<State>>>, impl Fn(State) + Sync) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let s = seen.clone();
    (seen, move |state| s.lock().unwrap().push(state))
}

fn sources(s: &Setup) -> Vec<Source> {
    vec![s.source.clone()]
}

fn release() -> Release {
    Release {
        zip: release_zip(&[]),
        version: "1.2.0",
        serial: 5,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_newer_release_is_downloaded_verified_unpacked_and_staged() {
    let s = setup("happy", &release(), "1.1.0", |_, _| {}).await;
    let (seen, on_state) = states();
    let result = s.updater.run_once(&sources(&s), &on_state).await;
    let State::Ready(staged) = result else {
        panic!("{result:?}")
    };
    assert_eq!(staged.version, "1.2.0");
    assert_eq!(
        staged.notes_url.as_deref(),
        Some("https://example.test/notes")
    );
    assert_eq!(fs::read(staged.dir.join("app.exe")).unwrap().len(), 20_000);
    assert!(staged.dir.join("codec.dll").is_file());

    assert!(!s.root.join("1.2.0.zip").exists());
    assert_eq!(s.updater.staged().unwrap().version, "1.2.0");
    let seen = seen.lock().unwrap();
    assert_eq!(seen.first(), Some(&State::Checking));
    assert!(
        seen.iter()
            .any(|st| matches!(st, State::Downloading { done, total, .. } if done == total)),
        "progress reached 100%: {seen:?}"
    );

    let before = s.backend.zip_requests().len();
    assert!(matches!(
        s.updater.run_once(&sources(&s), &|_| {}).await,
        State::Ready(_)
    ));
    assert_eq!(s.backend.zip_requests().len(), before);
}

#[tokio::test(flavor = "multi_thread")]
async fn an_up_to_date_client_downloads_nothing() {
    for current in ["1.2.0", "2.0.0"] {
        let s = setup("uptodate", &release(), current, |_, _| {}).await;
        assert_eq!(
            s.updater.run_once(&sources(&s), &|_| {}).await,
            State::UpToDate
        );
        assert!(s.backend.zip_requests().is_empty());
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn an_interrupted_download_resumes_with_a_range_request() {
    let s = setup("resume", &release(), "1.1.0", |_, _| {}).await;
    s.backend
        .cut
        .lock()
        .unwrap()
        .insert(ZIP_PATH.to_string(), 10_000);
    let first = s.updater.run_once(&sources(&s), &|_| {}).await;
    assert!(matches!(first, State::Failed(_)), "{first:?}");
    let part = s.root.join("1.2.0.zip.part");
    assert_eq!(
        fs::metadata(&part).unwrap().len(),
        10_000,
        "the bytes that arrived are kept"
    );

    assert!(matches!(
        s.updater.run_once(&sources(&s), &|_| {}).await,
        State::Ready(_)
    ));
    let requests = s.backend.zip_requests();
    assert_eq!(
        requests.last().unwrap().1.as_deref(),
        Some("10000-"),
        "{requests:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_tampered_zip_is_rejected_and_nothing_is_staged() {
    let r = release();
    let mut tampered = r.zip.clone();
    let last = tampered.len() / 2;
    tampered[last] ^= 0xff;
    let s = setup("tamper", &r, "1.1.0", |files, _| {
        files.insert(ZIP_PATH.to_string(), tampered);
    })
    .await;
    let result = s.updater.run_once(&sources(&s), &|_| {}).await;
    assert!(matches!(result, State::Failed(_)), "{result:?}");
    assert!(s.updater.staged().is_none());
    assert!(
        !s.root.join("1.2.0").exists()
            && !s.root.join("1.2.0.zip").exists()
            && !s.root.join("1.2.0.zip.part").exists()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_zip_that_hashes_right_but_is_hostile_is_rejected() {
    let hostile = Release {
        zip: release_zip(&[("release-1.2.0/data/app.db", b"clobber")]),
        version: "1.2.0",
        serial: 5,
    };
    let s = setup("hostile", &hostile, "1.1.0", |_, _| {}).await;
    let result = s.updater.run_once(&sources(&s), &|_| {}).await;
    assert!(
        matches!(&result, State::Failed(m) if m.contains("rejected")),
        "{result:?}"
    );
    assert!(s.updater.staged().is_none());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_dead_first_mirror_falls_through_to_the_next() {
    let r = release();
    let signer = key(1);
    let s = setup("mirror", &r, "1.1.0", |files, base| {
        let urls = vec![
            format!("{base}/v1/releases/1.2.0/missing.zip"),
            format!("{base}{ZIP_PATH}"),
        ];
        files.insert(FEED_PATH.to_string(), feed(&r, urls, &signer).into_bytes());
    })
    .await;
    assert!(matches!(
        s.updater.run_once(&sources(&s), &|_| {}).await,
        State::Ready(_)
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_feed_signed_by_another_key_is_ignored() {
    let r = release();
    let s = setup("forged", &r, "1.1.0", |files, base| {
        files.insert(
            FEED_PATH.to_string(),
            feed(&r, vec![format!("{base}{ZIP_PATH}")], &key(9)).into_bytes(),
        );
    })
    .await;
    let result = s.updater.run_once(&sources(&s), &|_| {}).await;
    assert!(matches!(result, State::Failed(_)), "{result:?}");
    assert!(
        s.backend.zip_requests().is_empty(),
        "nothing is downloaded from an untrusted feed"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn an_older_copy_of_the_feed_cannot_be_replayed() {
    let newer = Release {
        serial: 10,
        ..release()
    };
    let s = setup("replay", &newer, "1.1.0", |_, _| {}).await;
    let config = s.updater.config.clone();
    assert!(matches!(
        check::check(&s.updater.client, &sources(&s), &config, &s.updater.db).await,
        Checked::Available(_)
    ));

    let older = Release {
        serial: 4,
        ..release()
    };
    let signer = key(1);
    let s2 = setup("replay2", &older, "1.1.0", |_, _| {}).await;
    let s2_source = Source {
        base: s2.backend.base.clone(),
        keys: keys(&signer),
        official: true,
    };

    s.updater
        .db
        .set_meta(&format!("update.serial.{}", s2_source.base), "10")
        .unwrap();
    assert_eq!(
        check::check(&s.updater.client, &[s2_source], &config, &s.updater.db).await,
        Checked::Unavailable
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_newest_release_among_several_sources_wins() {
    let old = Release {
        zip: release_zip(&[]),
        version: "1.2.0",
        serial: 5,
    };
    let s_old = setup("multi-old", &old, "1.1.0", |_, _| {}).await;
    let new_zip = release_zip(&[("release-1.3.0/extra.txt", b"x")]);
    let new = Release {
        zip: new_zip,
        version: "1.3.0",
        serial: 6,
    };
    let s_new = setup("multi-new", &new, "1.1.0", |_, _| {}).await;
    let both = vec![s_old.source.clone(), s_new.source.clone()];
    let config = s_old.updater.config.clone();
    match check::check(&s_old.updater.client, &both, &config, &s_old.updater.db).await {
        Checked::Available(offer) => assert_eq!(offer.version.to_string(), "1.3.0"),
        other => panic!("{other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn an_unreachable_backend_is_reported_not_fatal() {
    let s = setup("down", &release(), "1.1.0", |_, _| {}).await;
    let dead = vec![Source {
        base: "http://127.0.0.1:9".into(),
        keys: s.source.keys.clone(),
        official: true,
    }];
    let result = s.updater.run_once(&dead, &|_| {}).await;
    assert!(matches!(result, State::Failed(_)));
}

#[test]
fn auto_update_defaults_on_and_remembers_being_turned_off() {
    let db = Db::open_in_memory().unwrap();
    assert!(auto_update(&db));
    set_auto_update(&db, false);
    assert!(!auto_update(&db));
    set_auto_update(&db, true);
    assert!(auto_update(&db));
}
