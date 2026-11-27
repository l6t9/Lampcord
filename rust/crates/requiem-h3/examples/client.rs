#![allow(clippy::print_stdout)]

use std::sync::Arc;

use futures_util::StreamExt;
use requiem_h3::transport::Transport;
use wreq::IntoEmulation;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let emulation: wreq::Emulation = wreq_util::Emulation::builder()
        .profile(wreq_util::Profile::Chrome149)
        .platform(wreq_util::Platform::Windows)
        .headers(false)
        .build()
        .into_emulation();
    let transport = Arc::new(Transport::new(emulation.tls_options.clone().unwrap(), true));
    let client = wreq::Client::builder()
        .emulation(emulation)
        .cookie_store(true)
        .http3_transport(transport)
        .build()
        .unwrap();
    for round in 1..=3 {
        let response = client
            .get("https://discord.com/api/v9/experiments")
            .header("accept-encoding", "gzip, deflate, br, zstd")
            .send()
            .await
            .unwrap();
        let version = response.version();
        let status = response.status();
        let cookies = response.headers().get_all("set-cookie").iter().count();
        let body = response.text().await.unwrap();
        println!(
            "round {round}: {version:?} {status} set-cookie={cookies} body={} bytes (decompressed): {}",
            body.len(),
            &body[..body.len().min(60)]
        );
    }

    let response = client
        .get("https://discord.com/assets/web.b70721f9bc10ca0b.js")
        .send()
        .await
        .unwrap();
    println!("asset: {:?} {}", response.version(), response.status());
    let mut stream = response.bytes_stream();
    let mut total = 0;
    while let Some(chunk) = stream.next().await {
        total += chunk.unwrap().len();
    }
    println!("asset bytes: {total}");

    let response = client
        .post("https://discord.com/api/v9/auth/location-metadata")
        .json(&serde_json::json!({"x": 1}))
        .send()
        .await
        .unwrap();
    println!("post: {:?} {}", response.version(), response.status());
}
