use crate::utils::backoff::Backoff;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time;
use wreq::header;

const PROBE_POLL: Duration = Duration::from_millis(20);

#[derive(Clone, Debug)]
struct Bucket {
    remaining: u32,
    reset_at: Instant,
}

#[derive(Clone, Default)]
pub struct RateLimiter {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Default)]
struct Inner {
    route_to_bucket_hash: HashMap<String, String>,
    buckets: HashMap<String, Bucket>,
    global_retry_after: Option<Instant>,

    probing: HashSet<String>,
}

enum Gate {
    Go { probe: bool },
    WaitUntil(Instant),
    WaitForProbe,
}

impl Inner {
    fn gate(&mut self, route: &str, now: Instant) -> Gate {
        if let Some(at) = self.global_retry_after.filter(|&at| at > now) {
            return Gate::WaitUntil(at);
        }
        let Some(hash) = self.route_to_bucket_hash.get(route) else {
            return if self.probing.insert(route.to_string()) {
                Gate::Go { probe: true }
            } else {
                Gate::WaitForProbe
            };
        };
        match self.buckets.get_mut(hash) {
            Some(bucket) if bucket.reset_at > now => {
                if bucket.remaining == 0 {
                    return Gate::WaitUntil(bucket.reset_at);
                }
                bucket.remaining -= 1;
                Gate::Go { probe: false }
            }

            _ => Gate::Go { probe: false },
        }
    }
}

pub struct Permit {
    limiter: RateLimiter,
    route: String,
    probe: bool,
}

impl Drop for Permit {
    fn drop(&mut self) {
        if self.probe {
            self.limiter
                .inner
                .lock()
                .unwrap()
                .probing
                .remove(&self.route);
        }
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn acquire(&self, route: &str) -> Permit {
        loop {
            let gate = self.inner.lock().unwrap().gate(route, Instant::now());
            match gate {
                Gate::Go { probe } => {
                    return Permit {
                        limiter: self.clone(),
                        route: route.to_string(),
                        probe,
                    };
                }
                Gate::WaitUntil(at) => {
                    time::sleep(at.saturating_duration_since(Instant::now())).await
                }
                Gate::WaitForProbe => time::sleep(PROBE_POLL).await,
            }
        }
    }

    pub fn observe(&self, route: &str, headers: &header::HeaderMap) {
        let get = |name: &str| headers.get(name).and_then(|v| v.to_str().ok());
        self.observe_values(
            route,
            get("x-ratelimit-global").is_some(),
            get("retry-after").and_then(|v| v.parse::<f64>().ok()),
            get("x-ratelimit-bucket"),
            get("x-ratelimit-remaining").and_then(|v| v.parse::<u32>().ok()),
            get("x-ratelimit-reset-after").and_then(|v| v.parse::<f64>().ok()),
        );
    }

    fn observe_values(
        &self,
        route: &str,
        global: bool,
        retry_after: Option<f64>,
        bucket: Option<&str>,
        remaining: Option<u32>,
        reset_after: Option<f64>,
    ) {
        let mut inner = self.inner.lock().unwrap();
        let now = Instant::now();

        if global {
            if let Some(retry_after) = retry_after {
                inner.global_retry_after = Some(now + Duration::from_secs_f64(retry_after));
            }
        }

        let Some(bucket_hash) = bucket else { return };
        inner
            .route_to_bucket_hash
            .insert(route.to_string(), bucket_hash.to_string());
        if let (Some(remaining), Some(reset_after)) = (remaining, reset_after) {
            inner.buckets.insert(
                bucket_hash.to_string(),
                Bucket {
                    remaining,
                    reset_at: now + Duration::from_secs_f64(reset_after),
                },
            );
        }
    }
}

static URL_BLOCKS: Mutex<Option<HashMap<String, (Instant, Backoff)>>> =
    Mutex::new(None);

pub fn block_url(url: &str, retry_after_secs: Option<f64>) -> Duration {
    let mut guard = URL_BLOCKS.lock().unwrap_or_else(|e| e.into_inner());
    let blocks = guard.get_or_insert_with(Default::default);
    let entry = blocks.entry(url.to_string()).or_insert_with(|| {
        (
            Instant::now(),
            Backoff::with_range(1_000, Some(60_000)),
        )
    });
    let minimum = Duration::from_secs_f64(retry_after_secs.unwrap_or(5.0).clamp(0.0, 3600.0));
    let wait = entry.1.fail(Some(minimum));
    let until = Instant::now() + wait;
    if until > entry.0 {
        entry.0 = until;
    }
    wait
}

pub async fn wait_for_url(url: &str) {
    let until = {
        let mut guard = URL_BLOCKS.lock().unwrap_or_else(|e| e.into_inner());
        let Some(blocks) = guard.as_mut() else { return };
        match blocks.get(url) {
            Some((until, _)) if *until > Instant::now() => *until,
            Some(_) => {
                blocks.remove(url);
                return;
            }
            None => return,
        }
    };
    time::sleep_until(until.into()).await;
}

pub fn retry_delay(header: Option<f64>, body: Option<f64>) -> Duration {
    Duration::from_secs_f64(header.or(body).unwrap_or(1.0).clamp(0.05, 30.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limiter_with_bucket(remaining: u32) -> RateLimiter {
        let limiter = RateLimiter::new();
        limiter.observe_values(
            "GET /x",
            false,
            None,
            Some("b"),
            Some(remaining),
            Some(60.0),
        );
        limiter
    }

    #[test]
    fn an_unknown_route_lets_one_probe_through_and_holds_the_rest() {
        let limiter = RateLimiter::new();
        let mut inner = limiter.inner.lock().unwrap();
        let now = Instant::now();
        assert!(matches!(
            inner.gate("GET /x", now),
            Gate::Go { probe: true }
        ));
        assert!(matches!(inner.gate("GET /x", now), Gate::WaitForProbe));
        assert!(
            matches!(inner.gate("GET /y", now), Gate::Go { probe: true }),
            "other routes are independent"
        );
    }

    #[tokio::test]
    async fn dropping_the_probe_permit_releases_the_route() {
        let limiter = RateLimiter::new();
        let permit = limiter.acquire("GET /x").await;
        assert!(limiter.inner.lock().unwrap().probing.contains("GET /x"));
        drop(permit);
        assert!(!limiter.inner.lock().unwrap().probing.contains("GET /x"));
    }

    #[test]
    fn a_known_bucket_hands_out_exactly_its_remaining_slots() {
        let limiter = limiter_with_bucket(2);
        let mut inner = limiter.inner.lock().unwrap();
        let now = Instant::now();
        assert!(matches!(
            inner.gate("GET /x", now),
            Gate::Go { probe: false }
        ));
        assert!(matches!(
            inner.gate("GET /x", now),
            Gate::Go { probe: false }
        ));
        assert!(
            matches!(inner.gate("GET /x", now), Gate::WaitUntil(_)),
            "the third concurrent request must wait for the reset"
        );
    }

    #[test]
    fn an_elapsed_window_lets_requests_through_again() {
        let limiter = limiter_with_bucket(0);
        let mut inner = limiter.inner.lock().unwrap();
        let later = Instant::now() + Duration::from_secs(61);
        assert!(matches!(
            inner.gate("GET /x", later),
            Gate::Go { probe: false }
        ));
    }

    #[test]
    fn a_global_limit_blocks_every_route() {
        let limiter = limiter_with_bucket(5);
        limiter.observe_values("GET /y", true, Some(3.0), None, None, None);
        let mut inner = limiter.inner.lock().unwrap();
        assert!(matches!(
            inner.gate("GET /x", Instant::now()),
            Gate::WaitUntil(_)
        ));
    }

    #[test]
    fn retry_delay_prefers_the_header_and_is_clamped() {
        assert_eq!(retry_delay(Some(2.0), Some(9.0)), Duration::from_secs(2));
        assert_eq!(retry_delay(None, Some(9.0)), Duration::from_secs(9));
        assert_eq!(retry_delay(None, None), Duration::from_secs(1));
        assert_eq!(retry_delay(Some(9999.0), None), Duration::from_secs(30));
    }

    #[tokio::test]
    async fn a_429_blocks_later_requests_to_the_same_url_for_the_retry_after() {
        let url = "https://discord.test/api/v9/channels/1/messages/block-test";
        let wait = block_url(url, Some(0.2));
        assert!(wait >= Duration::from_millis(200));
        let started = Instant::now();
        wait_for_url(url).await;
        assert!(
            started.elapsed() >= Duration::from_millis(150),
            "waited out the block"
        );
        let started = Instant::now();
        wait_for_url("https://discord.test/api/v9/other").await;
        assert!(
            started.elapsed() < Duration::from_millis(50),
            "another URL is not held"
        );
    }
}
