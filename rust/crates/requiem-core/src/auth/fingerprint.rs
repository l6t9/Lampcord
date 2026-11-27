use crate::rest;
use std::sync::Mutex;
use tokio::sync::Mutex as SyncMutex;
use tokio::sync;

use crate::config::BackendConfig;
use crate::properties::SuperProperties;
use crate::rest::RequestExtras;

static FINGERPRINT: Mutex<Option<String>> = Mutex::new(None);
static SETUP: sync::Mutex<bool> = SyncMutex::const_new(false);

pub fn current() -> Option<String> {
    FINGERPRINT.lock().ok().and_then(|slot| slot.clone())
}

pub fn forget() {
    if let Ok(mut slot) = FINGERPRINT.lock() {
        *slot = None;
    }
}

pub async fn ensure(
    client: &wreq::Client,
    config: &BackendConfig,
    super_properties: &SuperProperties,
) {
    let mut done = SETUP.lock().await;
    if *done && current().is_some() {
        return;
    }
    let base = config.rest_base_url.trim_end_matches('/');
    let experiments = RequestExtras {
        context: Some(serde_json::json!({ "location": "LOGIN" })),
        ..RequestExtras::default()
    };
    let (headers, order) = rest::anonymous_request_headers(
        &config.web_base_url(),
        super_properties,
        &wreq::Method::GET,
        false,
        &experiments,
        current().as_deref(),
    );
    match client
        .get(format!("{base}/experiments?with_guild_experiments=true"))
        .headers(headers)
        .orig_headers(order)
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            match response.json::<serde_json::Value>().await {
                Ok(body) => {
                    if let Some(fingerprint) = body
                        .get("fingerprint")
                        .and_then(|f| f.as_str())
                        .filter(|f| !f.is_empty())
                    {
                        if let Ok(mut slot) = FINGERPRINT.lock() {
                            *slot = Some(fingerprint.to_string());
                        }
                    }
                }
                Err(err) => log::warn!("experiments answer unreadable: {err}"),
            }
        }
        Ok(response) => log::warn!("GET /experiments answered {}", response.status()),
        Err(err) => log::warn!("GET /experiments failed: {err}"),
    }
    let (headers, order) = rest::anonymous_request_headers(
        &config.web_base_url(),
        super_properties,
        &wreq::Method::GET,
        false,
        &RequestExtras::default(),
        current().as_deref(),
    );
    if let Err(err) = client
        .get(format!("{base}/auth/location-metadata"))
        .headers(headers)
        .orig_headers(order)
        .send()
        .await
    {
        log::warn!("GET /auth/location-metadata failed: {err}");
    }
    *done = true;
}
