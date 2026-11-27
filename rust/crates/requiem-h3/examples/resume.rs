#![allow(clippy::print_stdout)]

use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net;
use tokio::time;

use futures_util::StreamExt;
use requiem_h3::SessionStore;
use requiem_h3::connection::{RequestBody, RequestHead};
use wreq::IntoEmulation;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let insecure = args.iter().any(|a| a == "--insecure");
    let positional: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();
    let host = positional.first().map_or("discord.com", |s| s.as_str());
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
    let tls = emulation.tls_options.clone().expect("tls options");
    let sessions = Arc::new(SessionStore::default());
    for round in 1..=3 {
        let started = Instant::now();
        let connection = requiem_h3::connect(
            remote,
            host,
            tls.clone(),
            !insecure,
            Some(sessions.clone()),
            Duration::from_secs(10),
        )
        .await
        .expect("connect");
        let connected_in = started.elapsed();
        let head = RequestHead {
            method: "GET".into(),
            authority: host.into(),
            path: format!("/q?case=resume_{round}"),
            headers: vec![("accept-encoding".into(), "identity".into())],
        };
        let response = connection
            .send(head, RequestBody::Empty)
            .await
            .expect("request");
        let status = response.status;
        let mut body = response.body;
        let mut total = 0;
        while let Some(chunk) = body.next().await {
            total += chunk.expect("body").len();
        }
        println!(
            "round {round}: connect returned after {connected_in:?}, response {status} ({total} bytes) after {:?}, 0-RTT: {:?}",
            started.elapsed(),
            connection.zero_rtt()
        );

        time::sleep(Duration::from_millis(800)).await;
    }
}
