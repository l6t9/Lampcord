#![allow(clippy::print_stdout)]

use std::env;
use std::sync::Arc;

use requiem_h3::transport::Transport;
use wreq::IntoEmulation;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let url = env::args()
        .nth(1)
        .unwrap_or("https://storage.googleapis.com/".into());
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
    for round in 1..=8 {
        let response = client
            .get(&url)
            .header("accept-language", "en-GB,en-US;q=0.9,en;q=0.8")
            .header("x-goog-api-client", "gl-rust/1.0 gccl/2.0")
            .header("x-discord-test", format!("value-{}", round % 2))
            .send()
            .await;
        match response {
            Ok(r) => println!("round {round}: {:?} {}", r.version(), r.status()),
            Err(e) => println!("round {round}: error {e:?}"),
        }
    }
}
