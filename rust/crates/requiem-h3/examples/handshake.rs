#![allow(clippy::print_stdout)]

use std::env;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::net;
use tokio::time;

use wreq::IntoEmulation;

use requiem_h3::{chrome_transport_config, crypto, driver::Driver};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let insecure = args.iter().any(|a| a == "--insecure");
    let positional: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();
    let host = positional
        .first()
        .map_or("discord.com", |s| s.as_str())
        .to_string();
    let remote = match positional.get(1) {
        Some(addr) => addr.parse().expect("ip:port"),
        None => net::lookup_host((host.as_str(), 443))
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
    let ssl_factory = Arc::new(move |name: &str| requiem_h3::quic_ssl(&tls, name, !insecure, None));
    let crypto = Arc::new(crypto::ClientConfig {
        ssl: ssl_factory,
        params: Arc::new(requiem_h3::chrome_transport_parameters),
    });
    println!("connecting to {host} at {remote}");
    let mut driver = Driver::connect(remote, &host, crypto, chrome_transport_config(), 0)
        .await
        .expect("connect");
    let started = Instant::now();
    loop {
        match time::timeout(Duration::from_secs(10), driver.step()).await {
            Ok(Ok(events)) => {
                for event in &events {
                    println!("{:?} event: {event:?}", started.elapsed());
                }
                if events
                    .iter()
                    .any(|e| matches!(e, quinn_proto::Event::Connected))
                {
                    break;
                }
            }
            Ok(Err(e)) => panic!("{e}"),
            Err(_) => panic!("handshake timed out: {:?}", driver.conn.stats().udp_tx),
        }
    }
    println!("connected; handshaking={}", driver.conn.is_handshaking());
    if let Some(data) = driver.conn.crypto_session().handshake_data() {
        if let Ok(data) = data.downcast::<crypto::HandshakeData>() {
            println!(
                "ALPN: {:?}",
                data.protocol.as_deref().map(String::from_utf8_lossy)
            );
        }
    }
}
