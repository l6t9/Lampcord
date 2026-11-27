#![allow(clippy::print_stdout)]

use std::env;
use std::time::Duration;
use tokio::net;

use futures_util::StreamExt;
use requiem_h3::connection::{RequestBody, RequestHead};
use wreq::IntoEmulation;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let insecure = args.iter().any(|a| a == "--insecure");
    let positional: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();
    let url = positional.first().expect("a URL");
    let rest = url.strip_prefix("https://").expect("https URL");
    let (host, path) = rest
        .split_once('/')
        .map_or((rest, "/".to_string()), |(h, p)| (h, format!("/{p}")));
    let remote = match positional.get(1) {
        Some(addr) => addr.parse().expect("ip:port"),
        None => net::lookup_host((host, 443))
            .await
            .unwrap()
            .find(|a| a.is_ipv4())
            .expect("an address"),
    };
    let emulation: wreq::Emulation = wreq_util::Emulation::builder()
        .profile(wreq_util::Profile::Chrome149)
        .platform(wreq_util::Platform::Windows)
        .headers(false)
        .build()
        .into_emulation();
    let connection = requiem_h3::connect(
        remote,
        host,
        emulation.tls_options.clone().expect("tls options"),
        !insecure,
        None,
        Duration::from_secs(10),
    )
    .await
    .expect("connect");
    let head = RequestHead {
        method: "GET".into(),
        authority: host.into(),
        path,
        headers: vec![
            ("accept-encoding".into(), "identity".into()),
            ("user-agent".into(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0.0.0 Safari/537.36".into()),
        ],
    };
    let response = connection
        .send(head, RequestBody::Empty)
        .await
        .expect("request");
    println!("status {}", response.status);
    println!("server settings: {:?}", connection.peer_settings());
    for (name, value) in &response.headers {
        println!("{name}: {value}");
    }
    let mut body = response.body;
    let mut total = 0;
    let mut first = Vec::new();
    while let Some(chunk) = body.next().await {
        let chunk = chunk.expect("body");
        if first.len() < 600 {
            first.extend_from_slice(&chunk[..chunk.len().min(600 - first.len())]);
        }
        total += chunk.len();
    }
    println!("body: {total} bytes\n{}", String::from_utf8_lossy(&first));
}
