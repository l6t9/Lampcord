pub(crate) mod chromium_headers;
mod denied;
pub mod endpoints;
pub mod ratelimit;
pub use chromium_headers::order_for as wire_order;
#[cfg(test)]
mod measured_wire_orders;
use base64::Engine as _;
use base64::engine::general_purpose;
use crate::net_profile::Resource;
use crate::net_profile::UploadShape;
use crate::net_profile;
use crate::netlog;
use crate::properties::Platform;
use crate::properties;
use crate::utils::backoff::Backoff;
use futures_util::stream;
use requiem_h3::transport::Transport;
use std::env;
use std::io::SeekFrom;
use std::io;
use std::net;
use std::path;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;
use std::time::Instant;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::time;
use wreq::header::HeaderName;
use wreq::header::HeaderValue;
use wreq::header;
use wreq::http2::StreamDependency;
use wreq::http2::StreamId;
use wreq::ws;

use thiserror::Error;
use wreq::IntoEmulation;

use crate::auth::AuthCoordinator;
use crate::config::BackendConfig;
use crate::properties::{SuperProperties, api_headers};
use ratelimit::RateLimiter;

pub fn build_client() -> Result<wreq::Client, wreq::Error> {
    client_builder().build()
}

pub fn build_image_client() -> Result<wreq::Client, wreq::Error> {
    client_builder_weighted(147)
        .timeout(Duration::from_secs(15))
        .build()
}

#[cfg(feature = "video")]
pub fn build_video_client() -> Result<wreq::Client, wreq::Error> {
    client_builder_weighted(147)
        .timeout(Duration::from_secs(60))
        .build()
}

#[derive(Clone, Debug, Default)]
pub struct RequestExtras {
    pub context: Option<serde_json::Value>,

    pub reason: Option<String>,

    pub headers: Vec<(&'static str, String)>,

    pub failed: u32,

    pub retries: u32,

    pub requeue_on_429: bool,
}

impl RequestExtras {
    pub fn context(location: &str) -> Self {
        Self {
            context: Some(serde_json::json!({ "location": location })),
            ..Self::default()
        }
    }

    pub fn context_opt(location: Option<&str>) -> Self {
        match location {
            Some(location) => Self::context(location),
            None => Self {
                context: Some(serde_json::json!({})),
                ..Self::default()
            },
        }
    }

    fn apply(&self, headers: &mut header::HeaderMap) {
        let mut put = |name: &'static str, value: &str| {
            if let Ok(value) = HeaderValue::from_str(value) {
                headers.insert(HeaderName::from_static(name), value);
            }
        };
        for (name, value) in &self.headers {
            put(name, value);
        }
        if let Some(reason) = &self.reason {
            put("x-audit-log-reason", &encode_uri_component(reason));
        }
        if let Some(context) = &self.context {
            put(
                "x-context-properties",
                &general_purpose::STANDARD.encode(context.to_string()),
            );
        }
        if self.failed > 0 {
            put("x-failed-requests", &self.failed.to_string());
        }
    }
}

fn encode_uri_component(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&byte) {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

pub fn request_headers(
    origin: &str,
    super_properties: &SuperProperties,
    method: &wreq::Method,
    token: &str,
    json_body: bool,
    extras: &RequestExtras,
) -> (header::HeaderMap, header::OrigHeaderMap) {
    assemble_headers(
        origin,
        super_properties,
        method,
        Some(token),
        None,
        json_body,
        extras,
    )
}

pub fn anonymous_request_headers(
    origin: &str,
    super_properties: &SuperProperties,
    method: &wreq::Method,
    json_body: bool,
    extras: &RequestExtras,
    fingerprint: Option<&str>,
) -> (header::HeaderMap, header::OrigHeaderMap) {
    assemble_headers(
        origin,
        super_properties,
        method,
        None,
        fingerprint,
        json_body,
        extras,
    )
}

fn assemble_headers(
    origin: &str,
    super_properties: &SuperProperties,
    method: &wreq::Method,
    token: Option<&str>,
    fingerprint: Option<&str>,
    json_body: bool,
    extras: &RequestExtras,
) -> (header::HeaderMap, header::OrigHeaderMap) {
    let mut headers = api_headers(origin, super_properties);
    if matches!(*method, wreq::Method::GET | wreq::Method::HEAD) {
        headers.remove(header::ORIGIN);
    }
    if let Some(token) = token {
        headers.insert(
            header::AUTHORIZATION,
            token
                .parse()
                .expect("an authorization token is a valid header value"),
        );
    }
    if let Some(fingerprint) = fingerprint.and_then(|f| HeaderValue::from_str(f).ok())
    {
        headers.insert(
            HeaderName::from_static("x-fingerprint"),
            fingerprint,
        );
    }
    if json_body {
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
    }
    extras.apply(&mut headers);
    let content_length = json_body
        || matches!(
            *method,
            wreq::Method::POST | wreq::Method::PUT | wreq::Method::PATCH
        );
    let order = wire_order(&headers, content_length);
    (headers, order)
}

pub fn emulation_with_weight(weight: u16) -> wreq::Emulation {
    let mut emulation: wreq::Emulation = wreq_util::Emulation::builder()
        .profile(properties::emulation_profile())
        .platform(Platform::current().wreq())
        .headers(false)
        .build()
        .into_emulation();
    if let Some(options) = emulation.http2_options.as_mut() {
        options.headers_stream_dependency = Some(StreamDependency::new(
            StreamId::zero(),
            (weight - 1) as u8,
            true,
        ));
    }
    emulation
}

#[doc(hidden)]
pub fn measuring_client(
    hosts: &[&'static str],
    addr: net::SocketAddr,
) -> Result<wreq::Client, wreq::Error> {
    let mut builder = plain_client_builder(220).tls_cert_verification(false);
    for host in hosts {
        builder = builder.resolve(*host, addr);
    }
    builder.build()
}

#[doc(hidden)]
pub fn measuring_client_unpooled(
    hosts: &[&'static str],
    addr: net::SocketAddr,
) -> Result<wreq::Client, wreq::Error> {
    let mut builder = plain_client_builder(220)
        .tls_cert_verification(false)
        .pool_max_idle_per_host(0);
    for host in hosts {
        builder = builder.resolve(*host, addr);
    }
    builder.build()
}

#[doc(hidden)]
pub fn measuring_http_client(
    client: wreq::Client,
    config: BackendConfig,
    auth: AuthCoordinator,
) -> HttpClient {
    net_profile::set_origin(&config.web_base_url());
    HttpClient {
        client,
        config,
        super_properties: SuperProperties::desktop_like(),
        rate_limiter: RateLimiter::new(),
        auth,
        denied: denied::Denied::default(),
    }
}

pub fn websocket(client: &wreq::Client, url: &str) -> ws::WebSocketRequestBuilder {
    let mut emulation = emulation_with_weight(220);
    if let Some(tls) = emulation.tls_options.as_mut() {
        tls.alps_protocols = None;
    }
    let (headers, order) = net_profile::websocket_handshake();
    client
        .websocket(url)
        .emulation(emulation)
        .headers(headers)
        .orig_headers(order)
}

fn client_builder() -> wreq::ClientBuilder {
    client_builder_weighted(220)
}

fn client_builder_weighted(weight: u16) -> wreq::ClientBuilder {
    plain_client_builder(weight).http3_transport(http3_transport())
}

fn plain_client_builder(weight: u16) -> wreq::ClientBuilder {
    wreq::Client::builder()
        .emulation(emulation_with_weight(weight))
        .cookie_store(true)
}

fn http3_transport() -> Arc<dyn wreq::Http3Transport> {
    static TRANSPORT: OnceLock<Arc<Transport>> =
        OnceLock::new();
    TRANSPORT
        .get_or_init(|| {
            let tls = emulation_with_weight(220)
                .tls_options
                .expect("the Chromium emulation has TLS options");
            let transport = Transport::with_store(
                tls,
                true,
                Some(requiem_db::Db::shared().clone()),
            );
            Arc::new(if env::var_os("REQUIEM_NO_H3").is_some() {
                transport.disabled()
            } else {
                transport
            })
        })
        .clone()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UploadStatus {
    Complete,

    ResumeFrom(u64),
}

#[derive(Debug, Error)]
pub enum RestError {
    #[error("request failed: {0}")]
    Request(#[from] wreq::Error),
    #[error("{method} {url} returned {status}: {body}")]
    Status {
        method: wreq::Method,
        url: String,
        status: wreq::StatusCode,
        body: String,
    },

    #[error("request returned 401 and re-authentication failed: {0}")]
    ReauthenticationFailed(String),

    #[error("{plugin} stopped this message: {reason}")]
    CancelledByPlugin { plugin: String, reason: String },

    #[error("couldn't read the file: {0}")]
    Io(#[from] io::Error),
}

#[derive(Clone)]
pub struct HttpClient {
    client: wreq::Client,
    config: BackendConfig,
    super_properties: SuperProperties,
    rate_limiter: RateLimiter,
    auth: AuthCoordinator,

    denied: denied::Denied,
}

impl HttpClient {
    pub fn new(config: BackendConfig, auth: AuthCoordinator) -> Result<Self, RestError> {
        net_profile::set_origin(&config.web_base_url());
        let client = build_client()?;

        Ok(Self {
            client,
            config,
            super_properties: SuperProperties::desktop_like(),
            rate_limiter: RateLimiter::new(),
            auth,
            denied: denied::Denied::default(),
        })
    }

    pub fn retry_denied(&self, needle: &str) {
        self.denied.forget(needle);
    }

    pub fn client(&self) -> &wreq::Client {
        &self.client
    }

    pub async fn request_with<B: serde::Serialize + ?Sized>(
        &self,
        route: &str,
        method: wreq::Method,
        path: &str,
        body: Option<&B>,
        extras: &RequestExtras,
    ) -> Result<wreq::Response, RestError> {
        self.send(route, method, path, body, extras).await
    }

    pub async fn request(
        &self,
        route: &str,
        method: wreq::Method,
        path: &str,
    ) -> Result<wreq::Response, RestError> {
        self.send(route, method, path, None::<&()>, &RequestExtras::default())
            .await
    }

    pub async fn request_with_json<B: serde::Serialize + ?Sized>(
        &self,
        route: &str,
        method: wreq::Method,
        path: &str,
        body: &B,
    ) -> Result<wreq::Response, RestError> {
        self.send(route, method, path, Some(body), &RequestExtras::default())
            .await
    }

    async fn execute<B: serde::Serialize + ?Sized>(
        &self,
        method: &wreq::Method,
        url: &str,
        token: &str,
        body: Option<&B>,
        extras: &RequestExtras,
    ) -> Result<wreq::Response, RestError> {
        let (headers, order) = request_headers(
            &self.config.web_base_url(),
            &self.super_properties,
            method,
            token,
            body.is_some(),
            extras,
        );
        let mut request = self
            .client
            .request(method.clone(), url)
            .headers(headers)
            .orig_headers(order);
        if let Some(body) = body {
            request = request.json(body);
        }
        Ok(request.send().await?)
    }

    pub async fn put_file(
        &self,
        url: &str,
        path: &path::Path,
        size: u64,
        from: u64,
        on_progress: impl Fn(u64) + Send + Sync + 'static,
    ) -> Result<(), RestError> {
        const CHUNK: usize = 64 * 1024;
        let mut file = File::open(path).await?;
        if from > 0 {
            file.seek(SeekFrom::Start(from)).await?;
        }
        let stream = stream::unfold(
            (file, from, on_progress),
            |(mut file, sent, on_progress)| async move {
                let mut buffer = vec![0u8; CHUNK];
                match file.read(&mut buffer).await {
                    Ok(0) => None,
                    Ok(n) => {
                        #[cfg(debug_assertions)]
                        if let Some(ms) = env::var("REQUIEM_UPLOAD_THROTTLE_MS")
                            .ok()
                            .and_then(|v| v.parse::<u64>().ok())
                        {
                            time::sleep(Duration::from_millis(ms)).await;
                        }
                        buffer.truncate(n);
                        let sent = sent + n as u64;
                        on_progress(sent);
                        Some((Ok::<_, io::Error>(buffer), (file, sent, on_progress)))
                    }
                    Err(err) => Some((Err(err), (file, sent, on_progress))),
                }
            },
        );

        let shape = if from > 0 {
            UploadShape::Resume { from, size }
        } else {
            UploadShape::Full
        };
        let (headers, order) = net_profile::headers_for_upload(
            url,
            Resource::Upload,
            shape,
        );
        let response = self
            .client
            .put(url)
            .headers(headers)
            .header(header::CONTENT_LENGTH, size.saturating_sub(from))
            .orig_headers(order)
            .body(wreq::Body::wrap_stream(stream))
            .send()
            .await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RestError::Status {
                method: wreq::Method::PUT,
                url: url.split('?').next().unwrap_or(url).to_string(),
                status,
                body,
            });
        }
        Ok(())
    }

    pub async fn upload_status(&self, url: &str) -> Result<UploadStatus, RestError> {
        let (headers, order) = net_profile::headers_for_upload(
            url,
            Resource::Upload,
            UploadShape::Probe,
        );
        let response = self
            .client
            .put(url)
            .headers(headers)
            .header(header::CONTENT_LENGTH, 0)
            .orig_headers(order)
            .send()
            .await?;
        let status = response.status();
        if status == wreq::StatusCode::OK || status == wreq::StatusCode::CREATED {
            return Ok(UploadStatus::Complete);
        }
        if status.as_u16() == 308 {
            let received = response
                .headers()
                .get("range")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.rsplit('-').next()?.parse::<u64>().ok())
                .map_or(0, |last| last + 1);
            return Ok(UploadStatus::ResumeFrom(received));
        }
        let body = response.text().await.unwrap_or_default();
        Err(RestError::Status {
            method: wreq::Method::PUT,
            url: url.split('?').next().unwrap_or(url).to_string(),
            status,
            body,
        })
    }

    #[tracing::instrument(skip_all, fields(route, method = %method, path))]
    async fn send<B: serde::Serialize + ?Sized>(
        &self,
        route: &str,
        method: wreq::Method,
        path: &str,
        body: Option<&B>,
        extras: &RequestExtras,
    ) -> Result<wreq::Response, RestError> {
        let started = Instant::now();
        let result = self
            .send_inner(route, method.clone(), path, body, extras)
            .await;
        let (status, error, bytes) = match &result {
            Ok(response) => (
                Some(response.status().as_u16()),
                None,
                response.content_length(),
            ),
            Err(RestError::Status { status, .. }) => (Some(status.as_u16()), None, None),
            Err(err) => (None, Some(err.to_string()), None),
        };
        netlog::record(netlog::NetEntry {
            seq: 0,
            at_ms: 0,
            method: method.to_string(),
            path: path.to_string(),
            status,
            error,
            duration_ms: started.elapsed().as_millis() as u64,
            bytes,
        });
        result
    }

    async fn send_inner<B: serde::Serialize + ?Sized>(
        &self,
        route: &str,
        method: wreq::Method,
        path: &str,
        body: Option<&B>,
        extras: &RequestExtras,
    ) -> Result<wreq::Response, RestError> {
        let url = format!(
            "{}{}",
            self.config.rest_base_url.trim_end_matches('/'),
            path
        );

        if let Some(refused) = self.denied.check(&method, path) {
            log::debug!(
                "REST {route} {path}: refused before ({}), not asking again",
                refused.status
            );
            return Err(RestError::Status {
                method,
                url,
                status: refused.status,
                body: refused.body,
            });
        }
        let url_key = url.split('?').next().unwrap_or(&url).to_string();
        let mut token_used = self.auth.current_token();
        let mut reauthenticated = false;

        let mut retried = 0;
        let mut retries_left = extras.retries;
        let mut requeues = 0;
        let mut backoff = Backoff::new();

        loop {
            ratelimit::wait_for_url(&url_key).await;

            let permit = self.rate_limiter.acquire(route).await;
            let attempt = RequestExtras {
                failed: retried,
                ..extras.clone()
            };
            let response = match self
                .execute(&method, &url, &token_used, body, &attempt)
                .await
            {
                Ok(response) => response,

                Err(err) if retries_left > 0 => {
                    drop(permit);
                    retries_left -= 1;
                    retried += 1;
                    log::warn!("REST {route} failed ({err}), retrying ({retried})");
                    time::sleep(backoff.fail(None)).await;
                    continue;
                }
                Err(err) => return Err(err),
            };
            self.rate_limiter.observe(route, response.headers());
            drop(permit);
            let status = response.status();

            if status == wreq::StatusCode::TOO_MANY_REQUESTS {
                let header = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<f64>().ok());
                let text = response.text().await.unwrap_or_default();
                let from_body = serde_json::from_str::<serde_json::Value>(&text)
                    .ok()
                    .and_then(|v| v.get("retry_after")?.as_f64());
                let wait = ratelimit::block_url(&url_key, header.or(from_body));
                if extras.requeue_on_429 && requeues < MAX_REQUEUES {
                    requeues += 1;
                    retried = 0;
                    retries_left = extras.retries;
                    log::warn!("REST {route} got 429, sending again in {wait:?}");
                    continue;
                }
                return Err(RestError::Status {
                    method,
                    url,
                    status,
                    body: text,
                });
            }

            if retries_left > 0 && RETRY_STATUSES.contains(&status.as_u16()) {
                retries_left -= 1;
                retried += 1;
                log::warn!("REST {route} got {status}, retrying ({retried})");
                time::sleep(backoff.fail(None)).await;
                continue;
            }

            if status == wreq::StatusCode::UNAUTHORIZED && !reauthenticated {
                log::warn!("REST request got 401 (token likely revoked) — recovering");
                token_used = match self.auth.recover_from(&token_used).await {
                    Ok(fresh_token) => {
                        log::info!("re-authenticated, retrying the request");
                        fresh_token
                    }
                    Err(err) => {
                        log::error!("re-authentication failed: {err}");
                        return Err(RestError::ReauthenticationFailed(err));
                    }
                };
                reauthenticated = true;
                continue;
            }

            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                self.denied.note(&method, path, status, &body);
                return Err(RestError::Status {
                    method,
                    url,
                    status,
                    body,
                });
            }
            return Ok(response);
        }
    }
}

const RETRY_STATUSES: [u16; 8] = [502, 504, 507, 598, 599, 522, 523, 524];

const MAX_REQUEUES: u32 = 30;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::properties::api_headers;

    fn names(order: &header::OrigHeaderMap) -> Vec<String> {
        order
            .iter()
            .map(|(name, _)| name.as_str().to_string())
            .collect()
    }

    #[test]
    fn context_reason_and_retry_headers_use_discords_encodings() {
        let xsp = SuperProperties::desktop_like();
        let extras = RequestExtras {
            context: Some(serde_json::json!({ "location": "chat_input" })),
            reason: Some("spam & ünï".into()),
            headers: vec![("x-captcha-key", "k".into())],
            failed: 2,
            ..Default::default()
        };
        let (headers, order) = request_headers(
            "https://discord.com",
            &xsp,
            &wreq::Method::POST,
            "t",
            true,
            &extras,
        );
        assert_eq!(
            headers["x-context-properties"], "eyJsb2NhdGlvbiI6ImNoYXRfaW5wdXQifQ==",
            "the real client's value for chat_input"
        );
        assert_eq!(headers["x-audit-log-reason"], "spam%20%26%20%C3%BCn%C3%AF");
        assert_eq!(headers["x-failed-requests"], "2");
        assert!(names(&order).contains(&"x-captcha-key".to_string()));
    }

    #[test]
    fn a_context_without_a_location_is_the_empty_object_not_no_header() {
        let xsp = SuperProperties::desktop_like();
        let value = |extras: RequestExtras| {
            request_headers(
                "https://discord.com",
                &xsp,
                &wreq::Method::POST,
                "t",
                true,
                &extras,
            )
            .0
            .get("x-context-properties")
            .map(|v| v.to_str().unwrap().to_string())
        };
        assert_eq!(
            value(RequestExtras::context_opt(None)).as_deref(),
            Some("e30=")
        );
        assert_eq!(
            value(RequestExtras::context_opt(Some("chat_input"))).as_deref(),
            Some("eyJsb2NhdGlvbiI6ImNoYXRfaW5wdXQifQ==")
        );
        assert_eq!(value(RequestExtras::default()), None);
    }

    #[test]
    fn request_headers_follow_the_real_clients_measured_order() {
        let xsp = SuperProperties::desktop_like();
        let (_, get) = request_headers(
            "https://discord.com",
            &xsp,
            &wreq::Method::GET,
            "t",
            false,
            &RequestExtras::default(),
        );
        assert_eq!(
            names(&get),
            "sec-ch-ua-platform,authorization,x-debug-options,accept-language,sec-ch-ua,sec-ch-ua-mobile,x-discord-timezone,x-super-properties,x-discord-locale,user-agent,accept,sec-fetch-site,sec-fetch-mode,sec-fetch-dest,referer,accept-encoding,cookie,priority"
                .split(',')
                .collect::<Vec<_>>()
        );
        let (mut headers, _) = request_headers(
            "https://discord.com",
            &xsp,
            &wreq::Method::POST,
            "t",
            true,
            &RequestExtras::default(),
        );
        headers.insert("x-audit-log-reason", "r".parse().unwrap());
        let post = wire_order(&headers, true);
        assert_eq!(
            names(&post),
            "content-length,sec-ch-ua-platform,authorization,x-debug-options,accept-language,sec-ch-ua,sec-ch-ua-mobile,x-discord-timezone,x-super-properties,x-audit-log-reason,x-discord-locale,user-agent,content-type,accept,origin,sec-fetch-site,sec-fetch-mode,sec-fetch-dest,referer,accept-encoding,cookie,priority"
                .split(',')
                .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    #[ignore = "needs network"]
    async fn fingerprint_looks_like_chromium() {
        let client = build_client().unwrap();
        let headers = api_headers("https://discord.com", &SuperProperties::desktop_like());
        let body = client
            .get("https://tls.peet.ws/api/all")
            .headers(headers)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        println!("{body}");
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(
            json["tls"]["ja4"]
                .as_str()
                .unwrap()
                .starts_with("t13d1516h2"),
            "ja4 = {}",
            json["tls"]["ja4"]
        );
        assert_eq!(json["http_version"], "h2");
    }
}
