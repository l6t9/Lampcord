use std::collections::HashMap;
use std::error;
use std::pin::Pin;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex as StdMutex};
use std::sync;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::net;
use tokio::time::Instant as TimeInstant;
use tokio::time;
use wreq::tls;

use futures_core::Stream;
use futures_util::StreamExt;
use futures_util::future::BoxFuture;
use http::HeaderMap;
use tokio::sync::watch;
use wreq::tls::TlsOptions;
use wreq::{Http3Body, Http3Head, Http3Response, Http3Session, Http3Transport};

use crate::connection::{Connection, RequestBody, RequestHead};

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(3);

const RACE_DELAY: Duration = Duration::from_millis(300);

const BROKEN_BASE: Duration = Duration::from_secs(5 * 60);
const BROKEN_MAX: Duration = Duration::from_secs(24 * 60 * 60);

type BoxError = Box<dyn error::Error + Send + Sync>;

struct Origin {
    alt_svc_until: Option<SystemTime>,
    broken_until: Option<Instant>,
    failures: u32,
    connection: Option<Connection>,
    connecting: bool,

    changed: watch::Sender<u32>,
}

impl Default for Origin {
    fn default() -> Self {
        Self {
            alt_svc_until: None,
            broken_until: None,
            failures: 0,
            connection: None,
            connecting: false,
            changed: watch::channel(0).0,
        }
    }
}

struct Inner {
    tls: TlsOptions,
    verify: bool,
    origins: StdMutex<HashMap<String, Arc<StdMutex<Origin>>>>,
    sessions: Arc<crate::SessionStore>,

    store: Option<requiem_db::Db>,

    saved_intermediates: AtomicUsize,
}

#[derive(Clone)]
pub struct Transport {
    inner: Arc<Inner>,
    disabled: bool,
}

fn lock(origin: &StdMutex<Origin>) -> sync::MutexGuard<'_, Origin> {
    origin.lock().unwrap_or_else(|e| e.into_inner())
}

impl Transport {
    pub fn new(tls: TlsOptions, verify: bool) -> Self {
        Self::with_store(tls, verify, None)
    }

    pub fn with_store(tls: TlsOptions, verify: bool, store: Option<requiem_db::Db>) -> Self {
        let transport = Self {
            inner: Arc::new(Inner {
                tls,
                verify,
                origins: StdMutex::new(HashMap::new()),
                sessions: Arc::new(crate::SessionStore::default()),
                store,
                saved_intermediates: AtomicUsize::new(0),
            }),
            disabled: false,
        };
        transport.load();
        transport
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    fn origin(&self, key: &str) -> Arc<StdMutex<Origin>> {
        self.inner
            .origins
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .entry(key.to_string())
            .or_default()
            .clone()
    }

    fn load(&self) {
        let Some(db) = &self.inner.store else { return };
        let now = SystemTime::now();
        for (key, expires) in db.alt_svc() {
            let until = UNIX_EPOCH + Duration::from_secs(expires);
            if until > now {
                lock(&self.origin(&key)).alt_svc_until = Some(until);
            }
        }
        let certs = db.intermediates();
        self.inner
            .saved_intermediates
            .store(certs.len(), Ordering::Relaxed);
        tls::add_seen_intermediates(certs);
    }

    fn save(&self) {
        let Some(db) = &self.inner.store else { return };
        let now = SystemTime::now();
        let alt_svc: Vec<(String, u64)> = self
            .inner
            .origins
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .filter_map(|(key, origin)| {
                let until = lock(origin).alt_svc_until.filter(|until| *until > now)?;
                Some((
                    key.clone(),
                    until.duration_since(UNIX_EPOCH).ok()?.as_secs(),
                ))
            })
            .collect();
        let intermediates = tls::seen_intermediates();
        self.inner
            .saved_intermediates
            .store(intermediates.len(), Ordering::Relaxed);
        if let Err(err) = db
            .set_alt_svc(&alt_svc)
            .and_then(|_| db.add_intermediates(&intermediates))
        {
            tracing::debug!("could not save Alt-Svc knowledge: {err}");
        }
    }

    pub fn set_alt_svc(&self, authority: &str, max_age: Option<Duration>) {
        let (key, _, _) = split_authority(authority);
        lock(&self.origin(&key)).alt_svc_until = max_age.map(|age| SystemTime::now() + age);
        self.save();
    }

    fn mark_broken(&self, key: &str) {
        let origin = self.origin(key);
        let mut origin = lock(&origin);
        origin.connection = None;
        origin.failures += 1;
        let delay = BROKEN_BASE
            .saturating_mul(1 << (origin.failures - 1).min(8))
            .min(BROKEN_MAX);
        origin.broken_until = Some(Instant::now() + delay);
    }

    fn spawn_connect(&self, key: String, host: String, port: u16) {
        let this = self.clone();
        tokio::spawn(async move {
            let attempt = async {
                let remote = net::lookup_host((host.as_str(), port))
                    .await
                    .ok()?
                    .min_by_key(|a| a.is_ipv6())?;
                crate::connect(
                    remote,
                    &host,
                    this.inner.tls.clone(),
                    this.inner.verify,
                    Some(this.inner.sessions.clone()),
                    HANDSHAKE_TIMEOUT,
                )
                .await
                .map_err(|e| tracing::debug!("QUIC to {host}:{port} failed: {e}"))
                .ok()
            };
            let result = attempt.await;
            let origin = this.origin(&key);
            let mut origin = lock(&origin);
            origin.connecting = false;
            match result {
                Some(connection) => {
                    origin.failures = 0;
                    origin.connection = Some(connection);
                }
                None => {
                    origin.failures += 1;
                    let delay = BROKEN_BASE
                        .saturating_mul(1 << (origin.failures - 1).min(8))
                        .min(BROKEN_MAX);
                    origin.broken_until = Some(Instant::now() + delay);
                }
            }
            origin.changed.send_modify(|n| *n += 1);
        });
    }
}

fn split_authority(authority: &str) -> (String, String, u16) {
    let authority = authority.rsplit('@').next().unwrap_or(authority);
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port))
            if port.parse::<u16>().is_ok() && !host.ends_with(']')
                || (host.ends_with(']') && port.parse::<u16>().is_ok()) =>
        {
            (host, port.parse().unwrap_or(443))
        }
        _ => (authority, 443),
    };
    let host = host
        .trim_start_matches('[')
        .trim_end_matches(']')
        .to_ascii_lowercase();
    (format!("{host}:{port}"), host, port)
}

pub fn parse_alt_svc(value: &str, port: u16) -> Option<Option<Duration>> {
    if value.trim().eq_ignore_ascii_case("clear") {
        return Some(None);
    }
    for entry in value.split(',') {
        let mut parts = entry.split(';');
        let Some((protocol, target)) = parts.next().and_then(|p| p.trim().split_once('=')) else {
            continue;
        };
        if protocol.trim() != "h3" {
            continue;
        }
        let target = target.trim().trim_matches('"');

        let Some(alt_port) = target.strip_prefix(':').and_then(|p| p.parse::<u16>().ok()) else {
            continue;
        };
        if alt_port != port && alt_port != 443 {
            continue;
        }
        let mut max_age = 86_400u64;
        for parameter in parts {
            if let Some((name, value)) = parameter.trim().split_once('=') {
                if name.trim() == "ma" {
                    max_age = value.trim().trim_matches('"').parse().unwrap_or(86_400);
                }
            }
        }
        return Some(Some(Duration::from_secs(max_age)));
    }
    None
}

impl Http3Transport for Transport {
    fn route(&self, authority: &str) -> BoxFuture<'static, Option<Arc<dyn Http3Session>>> {
        let this = self.clone();
        let (key, host, port) = split_authority(authority);
        Box::pin(async move {
            if this.disabled {
                return None;
            }
            let origin = this.origin(&key);
            let deadline = TimeInstant::now() + RACE_DELAY;
            loop {
                let mut changed = {
                    let mut state = lock(&origin);
                    if !state
                        .alt_svc_until
                        .is_some_and(|until| until > SystemTime::now())
                        || state
                            .broken_until
                            .is_some_and(|until| until > Instant::now())
                    {
                        return None;
                    }
                    if let Some(connection) = state.connection.as_ref().filter(|c| c.is_alive()) {
                        return Some(Arc::new(Session {
                            connection: connection.clone(),
                            transport: this.clone(),
                            key: key.clone(),
                        }) as Arc<dyn Http3Session>);
                    }
                    state.connection = None;
                    if !state.connecting {
                        state.connecting = true;
                        this.spawn_connect(key.clone(), host.clone(), port);
                    }
                    state.changed.subscribe()
                };

                if time::timeout_at(deadline, changed.changed())
                    .await
                    .is_err()
                {
                    return None;
                }
            }
        })
    }

    fn observe(&self, authority: &str, headers: &HeaderMap) {
        if self.disabled {
            return;
        }
        let (key, _, port) = split_authority(authority);
        let mut verdict = None;
        for value in headers.get_all("alt-svc") {
            if let Ok(value) = value.to_str() {
                if let Some(parsed) = parse_alt_svc(value, port) {
                    verdict = Some(parsed);
                }
            }
        }

        let new_intermediates = tls::seen_intermediates().len()
            != self
                .inner
                .saved_intermediates
                .load(Ordering::Relaxed);
        let Some(verdict) = verdict else {
            if new_intermediates {
                self.save();
            }
            return;
        };
        let until = verdict.map(|age| SystemTime::now() + age);
        let changed = {
            let origin = self.origin(&key);
            let mut state = lock(&origin);

            let changed = match (state.alt_svc_until, until) {
                (Some(old), Some(new)) => new
                    .duration_since(old)
                    .map_or(true, |d| d > Duration::from_secs(3600)),
                (None, None) => false,
                _ => true,
            };
            if changed || (until.is_some() && state.alt_svc_until.is_none()) {
                state.alt_svc_until = until;
            }
            changed
        };
        if changed || new_intermediates {
            self.save();
        }
    }
}

struct Session {
    connection: Connection,
    transport: Transport,
    key: String,
}

impl Http3Session for Session {
    fn send(
        &self,
        head: Http3Head,
        body: Http3Body,
    ) -> BoxFuture<'static, Result<Http3Response, BoxError>> {
        let connection = self.connection.clone();
        let (transport, key) = (self.transport.clone(), self.key.clone());
        Box::pin(async move {
            let head = RequestHead {
                method: head.method,
                authority: head.authority,
                path: head.path,
                headers: head.headers,
            };
            let body = match body {
                Http3Body::Empty => RequestBody::Empty,
                Http3Body::Full(bytes) => RequestBody::Full(bytes),
                Http3Body::Stream(stream) => RequestBody::Stream(stream),
            };
            let response = match connection.send(head, body).await {
                Ok(response) => response,
                Err(e) => {
                    if !connection.is_alive() && !connection.is_confirmed() {
                        transport.mark_broken(&key);
                    }
                    return Err(Box::new(e) as BoxError);
                }
            };
            let stream: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, BoxError>> + Send + Sync>> =
                Box::pin(
                    response
                        .body
                        .map(|chunk| chunk.map_err(|e| Box::new(e) as BoxError)),
                );
            Ok(Http3Response {
                status: response.status,
                headers: response.headers,
                body: stream,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alt_svc_is_read_like_a_browser_reads_it() {
        assert_eq!(
            parse_alt_svc("h3=\":443\"; ma=86400", 443),
            Some(Some(Duration::from_secs(86_400)))
        );
        assert_eq!(
            parse_alt_svc("h3-29=\":443\"; ma=3600, h3=\":443\"; ma=60", 443),
            Some(Some(Duration::from_secs(60)))
        );
        assert_eq!(
            parse_alt_svc("h3=\":443\"", 443),
            Some(Some(Duration::from_secs(86_400)))
        );
        assert_eq!(parse_alt_svc("clear", 443), Some(None));
        assert_eq!(parse_alt_svc("h2=\":443\"", 443), None);
        assert_eq!(parse_alt_svc("h3=\"other.example:443\"", 443), None);
    }

    #[test]
    fn authorities_split_into_host_and_port() {
        assert_eq!(
            split_authority("discord.com"),
            ("discord.com:443".into(), "discord.com".into(), 443)
        );
        assert_eq!(
            split_authority("Example.com:8443"),
            ("example.com:8443".into(), "example.com".into(), 8443)
        );
        assert_eq!(
            split_authority("[::1]:4433"),
            ("::1:4433".into(), "::1".into(), 4433)
        );
    }
}
