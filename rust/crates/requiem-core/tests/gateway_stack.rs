use requiem_core::rest;
use requiem_db::paths;
use std::env;
use std::fs;
use std::io::Write;
use std::mem;
use std::process;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;
use tokio_tungstenite::tungstenite::protocol;

use futures_util::{SinkExt, StreamExt};
use requiem_core::config::BackendConfig;
use requiem_core::gateway::{self, stack};
use requiem_core::properties::SuperProperties;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};

#[derive(Clone, Copy)]
enum Speaks {
    Plain,

    Zlib,
}

async fn serve(listener: TcpListener, speaks: Speaks, queries: Arc<Mutex<Vec<String>>>) {
    loop {
        let Ok((stream, _)) = listener.accept().await else {
            return;
        };
        let queries = queries.clone();
        tokio::spawn(async move {
            let query = Arc::new(Mutex::new(String::new()));
            let seen = query.clone();
            let callback = move |request: &Request, response: Response| {
                *seen.lock().unwrap() = request.uri().query().unwrap_or_default().to_string();
                Ok(response)
            };
            let Ok(mut socket) = tokio_tungstenite::accept_hdr_async(stream, callback).await else {
                return;
            };
            let query = query.lock().unwrap().clone();
            queries.lock().unwrap().push(query.clone());
            let understood = match speaks {
                Speaks::Plain => !query.contains("compress=") && query.contains("encoding=json"),
                Speaks::Zlib => {
                    (query.contains("compress=zlib-stream") || !query.contains("compress="))
                        && query.contains("encoding=json")
                }
            };
            if !understood {
                let _ = socket.close(None).await;
                return;
            }
            let hello =
                serde_json::json!({ "op": 10, "d": { "heartbeat_interval": 41250 } }).to_string();
            if query.contains("compress=zlib-stream") {
                let mut compressor = flate2::Compress::new(flate2::Compression::default(), true);
                let mut out = Vec::with_capacity(hello.len() + 64);
                compressor
                    .compress_vec(hello.as_bytes(), &mut out, flate2::FlushCompress::Sync)
                    .unwrap();
                let _ = socket.send(Message::binary(out)).await;
            } else {
                let _ = socket.send(Message::text(hello)).await;
            }

            let _ = socket.next().await;
            let _ = socket
                .close(Some(protocol::CloseFrame {
                    code: 4004.into(),
                    reason: "Authentication failed".into(),
                }))
                .await;
        });
    }
}

async fn connect_to(
    speaks: Speaks,
) -> (
    Vec<String>,
    Option<stack::Stack>,
    Result<(), gateway::GatewayError>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("ws://{}", listener.local_addr().unwrap());
    let queries = Arc::new(Mutex::new(Vec::new()));
    tokio::spawn(serve(listener, speaks, queries.clone()));
    let client = rest::build_client().unwrap();
    let config = BackendConfig {
        rest_base_url: String::new(),
        gateway_url: base.clone(),
        cdn_base_url: String::new(),
    };
    let (tx, _rx) = mpsc::unbounded_channel();
    let (_out_tx, mut out_rx) = mpsc::unbounded_channel();
    let mut session = gateway::Session::default();
    let result = time::timeout(
        Duration::from_secs(30),
        gateway::run(
            &client,
            &config,
            "token",
            &SuperProperties::desktop_like(),
            &mut session,
            tx,
            &mut out_rx,
        ),
    )
    .await
    .expect("no hang");
    let queries = queries.lock().unwrap().clone();
    (queries, stack::remembered(&base), result)
}

#[tokio::test(flavor = "multi_thread")]
async fn the_wire_format_falls_back_and_the_working_one_is_remembered() {
    let dir = env::temp_dir().join(format!("gateway-stack-test-{}", process::id()));
    fs::create_dir_all(&dir).unwrap();
    env::set_var(paths::DATA_DIR_ENV, &dir);
    env::set_var("APPDATA", &dir);
    env::set_var("XDG_CONFIG_HOME", &dir);

    let (queries, remembered, result) = connect_to(Speaks::Zlib).await;
    assert!(
        queries[0].contains("encoding=etf") && queries[0].contains("zstd-stream"),
        "{queries:?}"
    );
    assert!(
        queries[1].contains("encoding=json") && queries[1].contains("compress=zlib-stream"),
        "{queries:?}"
    );
    assert_eq!(queries.len(), 2, "{queries:?}");
    assert_eq!(remembered, Some(stack::Stack::Web));
    assert!(
        matches!(result, Err(gateway::GatewayError::AuthenticationFailed)),
        "{result:?}"
    );

    let (queries, remembered, result) = connect_to(Speaks::Plain).await;
    assert_eq!(queries.len(), 3, "{queries:?}");
    assert!(!queries[2].contains("compress="), "{queries:?}");
    assert_eq!(remembered, Some(stack::Stack::Plain));
    assert!(
        matches!(result, Err(gateway::GatewayError::AuthenticationFailed)),
        "{result:?}"
    );

    let _ = fs::remove_dir_all(dir);
}

#[tokio::test(flavor = "multi_thread")]
async fn an_address_that_names_its_encoding_is_not_negotiated() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!(
        "ws://{}/?encoding=etf&v=9&compress=zstd-stream",
        listener.local_addr().unwrap()
    );
    let queries = Arc::new(Mutex::new(Vec::new()));
    tokio::spawn(serve(listener, Speaks::Plain, queries.clone()));
    let client = rest::build_client().unwrap();
    let config = BackendConfig {
        rest_base_url: String::new(),
        gateway_url: base,
        cdn_base_url: String::new(),
    };
    let (tx, _rx) = mpsc::unbounded_channel();
    let (_out_tx, mut out_rx) = mpsc::unbounded_channel();
    let mut session = gateway::Session::default();
    let result = time::timeout(
        Duration::from_secs(30),
        gateway::run(
            &client,
            &config,
            "token",
            &SuperProperties::desktop_like(),
            &mut session,
            tx,
            &mut out_rx,
        ),
    )
    .await
    .expect("no hang");

    assert_eq!(queries.lock().unwrap().len(), 1);
    assert!(
        matches!(
            result,
            Err(gateway::GatewayError::Closed) | Err(gateway::GatewayError::ClosedBeforeHello)
        ),
        "{result:?}"
    );
}

async fn serve_etf_hello_then_decode_error(
    listener: TcpListener,
    queries: Arc<Mutex<Vec<String>>>,
) {
    loop {
        let Ok((stream, _)) = listener.accept().await else {
            return;
        };
        let queries = queries.clone();
        tokio::spawn(async move {
            let query = Arc::new(Mutex::new(String::new()));
            let seen = query.clone();
            let callback = move |request: &Request, response: Response| {
                *seen.lock().unwrap() = request.uri().query().unwrap_or_default().to_string();
                Ok(response)
            };
            let Ok(mut socket) = tokio_tungstenite::accept_hdr_async(stream, callback).await else {
                return;
            };
            let query = query.lock().unwrap().clone();
            queries.lock().unwrap().push(query.clone());
            if query.contains("encoding=etf") {
                let hello = serde_json::json!({ "op": 10, "d": { "heartbeat_interval": 41250 } });
                let mut encoder = zstd::stream::write::Encoder::new(Vec::new(), 3).unwrap();
                encoder
                    .write_all(&gateway::etf::encode(&hello))
                    .unwrap();
                encoder.flush().unwrap();
                let frame = mem::take(encoder.get_mut());
                let _ = socket.send(Message::binary(frame)).await;
                let _ = socket.next().await;
                let _ = socket
                    .close(Some(protocol::CloseFrame {
                        code: 4002.into(),
                        reason: "Decode error".into(),
                    }))
                    .await;
            } else {
                let hello = serde_json::json!({ "op": 10, "d": { "heartbeat_interval": 41250 } })
                    .to_string();
                if query.contains("compress=zlib-stream") {
                    let mut compressor =
                        flate2::Compress::new(flate2::Compression::default(), true);
                    let mut out = Vec::with_capacity(hello.len() + 64);
                    compressor
                        .compress_vec(hello.as_bytes(), &mut out, flate2::FlushCompress::Sync)
                        .unwrap();
                    let _ = socket.send(Message::binary(out)).await;
                } else {
                    let _ = socket.send(Message::text(hello)).await;
                }
                let _ = socket.next().await;
                let _ = socket
                    .close(Some(protocol::CloseFrame {
                        code: 4004.into(),
                        reason: "Authentication failed".into(),
                    }))
                    .await;
            }
        });
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn an_etf_identify_rejected_with_4002_falls_back_to_json() {
    let dir = env::temp_dir().join(format!("gateway-stack-test-4002-{}", process::id()));
    fs::create_dir_all(&dir).unwrap();
    env::set_var(paths::DATA_DIR_ENV, &dir);
    env::set_var("APPDATA", &dir);
    env::set_var("XDG_CONFIG_HOME", &dir);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("ws://{}", listener.local_addr().unwrap());
    let queries = Arc::new(Mutex::new(Vec::new()));
    tokio::spawn(serve_etf_hello_then_decode_error(listener, queries.clone()));
    let client = rest::build_client().unwrap();
    let config = BackendConfig {
        rest_base_url: String::new(),
        gateway_url: base.clone(),
        cdn_base_url: String::new(),
    };
    let (tx, _rx) = mpsc::unbounded_channel();
    let (_out_tx, mut out_rx) = mpsc::unbounded_channel();
    let mut session = gateway::Session::default();
    let result = time::timeout(
        Duration::from_secs(30),
        gateway::run(
            &client,
            &config,
            "token",
            &SuperProperties::desktop_like(),
            &mut session,
            tx,
            &mut out_rx,
        ),
    )
    .await
    .expect("no hang");
    let queries = queries.lock().unwrap().clone();

    assert!(queries[0].contains("encoding=etf"), "{queries:?}");
    assert_eq!(queries.len(), 2, "{queries:?}");
    assert!(queries[1].contains("encoding=json"), "{queries:?}");
    assert!(
        matches!(result, Err(gateway::GatewayError::AuthenticationFailed)),
        "{result:?}"
    );

    assert_eq!(
        stack::remembered(&base),
        Some(stack::Stack::Web),
        "the fallback format must be remembered after a 4002"
    );

    let _ = fs::remove_dir_all(dir);
}
