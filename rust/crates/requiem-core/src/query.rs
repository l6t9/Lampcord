use std::any::Any;
use std::any;
use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::runtime::Handle;

pub const DEFAULT_STALE_TIME: Duration = Duration::from_secs(60);

struct Entry {
    data: Option<Arc<dyn Any + Send + Sync>>,

    error: Option<Arc<str>>,

    updated_at: Option<Instant>,

    fetching: bool,

    type_name: &'static str,
}

pub struct QueryResult<T> {
    pub data: Option<Arc<T>>,
    pub error: Option<Arc<str>>,
    pub is_loading: bool,
    pub is_fetching: bool,
}

impl<T> QueryResult<T> {
    pub fn is_hard_error(&self) -> bool {
        self.data.is_none() && self.error.is_some()
    }
}

#[derive(Clone)]
pub struct QueryClient {
    entries: Arc<Mutex<HashMap<String, Entry>>>,
    runtime: Handle,
    on_change: Arc<dyn Fn() + Send + Sync>,
}

impl QueryClient {
    pub fn new(runtime: Handle, on_change: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            runtime,
            on_change: Arc::new(on_change),
        }
    }

    pub fn query<T, Fut, E>(
        &self,
        key: &str,
        stale_time: Duration,
        fetch: impl FnOnce() -> Fut + Send + 'static,
    ) -> QueryResult<T>
    where
        T: Send + Sync + 'static,
        E: fmt::Display,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
    {
        let mut entries = self.entries.lock().expect("query cache mutex poisoned");
        let entry = entries.entry(key.to_string()).or_insert_with(|| Entry {
            data: None,
            error: None,
            updated_at: None,
            fetching: false,
            type_name: any::type_name::<T>(),
        });

        assert!(
            entry.type_name == any::type_name::<T>(),
            "query key {key:?} was previously used with result type {} but is now being read as {} \
             — one key must always map to one type",
            entry.type_name,
            any::type_name::<T>(),
        );

        let stale = match entry.updated_at {
            Some(at) => at.elapsed() >= stale_time,
            None => true,
        };
        let needs_fetch = stale && !entry.fetching;
        if needs_fetch {
            entry.fetching = true;
        }

        let data = entry.data.clone();
        let error = entry.error.clone();
        let had_data = data.is_some();
        let fetching = entry.fetching;
        drop(entries);

        if needs_fetch {
            self.spawn_fetch(key.to_string(), fetch);
        }

        let data = data.map(|any| any.downcast::<T>().expect("query entry type checked above"));

        QueryResult {
            data,
            error,
            is_loading: fetching && !had_data,
            is_fetching: fetching,
        }
    }

    pub fn query_default<T, Fut, E>(
        &self,
        key: &str,
        fetch: impl FnOnce() -> Fut + Send + 'static,
    ) -> QueryResult<T>
    where
        T: Send + Sync + 'static,
        E: fmt::Display,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
    {
        self.query(key, DEFAULT_STALE_TIME, fetch)
    }

    pub fn clear(&self) {
        self.entries
            .lock()
            .expect("query cache mutex poisoned")
            .clear();
    }

    pub fn invalidate(&self, key: &str) {
        let mut entries = self.entries.lock().expect("query cache mutex poisoned");
        if let Some(entry) = entries.get_mut(key) {
            entry.updated_at = None;
        }
        drop(entries);
        (self.on_change)();
    }

    pub fn remove(&self, key: &str) {
        self.entries
            .lock()
            .expect("query cache mutex poisoned")
            .remove(key);
        (self.on_change)();
    }

    pub fn peek<T: Send + Sync + 'static>(&self, key: &str) -> Option<Arc<T>> {
        let entries = self.entries.lock().expect("query cache mutex poisoned");
        let entry = entries.get(key)?;

        entry.data.clone()?.downcast::<T>().ok()
    }

    pub fn set_data<T: Send + Sync + 'static>(&self, key: &str, value: T) {
        let mut entries = self.entries.lock().expect("query cache mutex poisoned");
        let entry = entries.entry(key.to_string()).or_insert_with(|| Entry {
            data: None,
            error: None,
            updated_at: None,
            fetching: false,
            type_name: any::type_name::<T>(),
        });
        assert!(
            entry.type_name == any::type_name::<T>(),
            "query key {key:?} holds {} but is being set as {} — one key must always map to one type",
            entry.type_name,
            any::type_name::<T>(),
        );
        entry.data = Some(Arc::new(value));
        entry.error = None;
        entry.updated_at = Some(Instant::now());
        drop(entries);
        (self.on_change)();
    }

    pub fn mutate<Fut, E>(
        &self,
        invalidates: Vec<String>,
        mutation: impl FnOnce() -> Fut + Send + 'static,
    ) where
        E: fmt::Display,
        Fut: Future<Output = Result<(), E>> + Send + 'static,
    {
        let entries = self.entries.clone();
        let on_change = self.on_change.clone();

        self.runtime.spawn(async move {
            if let Err(err) = mutation().await {
                log::warn!("mutation failed: {err}");
            }
            {
                let mut entries = entries.lock().expect("query cache mutex poisoned");
                for key in &invalidates {
                    if let Some(entry) = entries.get_mut(key) {
                        entry.updated_at = None;
                    }
                }
            }
            on_change();
        });
    }

    fn spawn_fetch<T, Fut, E>(&self, key: String, fetch: impl FnOnce() -> Fut + Send + 'static)
    where
        T: Send + Sync + 'static,
        E: fmt::Display,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
    {
        let entries = self.entries.clone();
        let on_change = self.on_change.clone();

        self.runtime.spawn(async move {
            let outcome = fetch().await;

            {
                let mut entries = entries.lock().expect("query cache mutex poisoned");
                let entry = entries.entry(key.clone()).or_insert_with(|| Entry {
                    data: None,
                    error: None,
                    updated_at: None,
                    fetching: false,
                    type_name: any::type_name::<T>(),
                });
                entry.fetching = false;

                entry.updated_at = Some(Instant::now());
                match outcome {
                    Ok(value) => {
                        entry.data = Some(Arc::new(value));
                        entry.error = None;
                    }
                    Err(err) => {
                        log::warn!("query {key} failed: {err}");
                        entry.error = Some(Arc::from(err.to_string().as_str()));
                    }
                }
            }

            on_change();
        });
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use super::*;
    use tokio::runtime::Builder;
    use tokio::runtime;
    use tokio::time;

    fn client() -> (runtime::Runtime, QueryClient) {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("runtime");
        let client = QueryClient::new(runtime.handle().clone(), || {});
        (runtime, client)
    }

    fn settled(client: &QueryClient, key: &str) -> bool {
        let entries = client.entries.lock().expect("mutex");
        entries.get(key).is_some_and(|e| !e.fetching)
    }

    fn wait_settled(client: &QueryClient, key: &str) {
        for _ in 0..200 {
            if settled(client, key) {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("query {key} never settled");
    }

    #[test]
    fn first_read_is_loading_then_resolves_and_is_served_from_cache() {
        let (_rt, client) = client();
        let calls = Arc::new(Mutex::new(0_usize));

        let spawn = |calls: Arc<Mutex<usize>>| {
            move || async move {
                *calls.lock().expect("mutex") += 1;
                Ok::<_, String>(7_u32)
            }
        };

        let first = client.query::<u32, _, String>("k", DEFAULT_STALE_TIME, spawn(calls.clone()));
        assert!(first.is_loading, "first read has no data yet");
        assert!(first.data.is_none());

        wait_settled(&client, "k");

        let second = client.query::<u32, _, String>("k", DEFAULT_STALE_TIME, spawn(calls.clone()));
        assert_eq!(second.data.as_deref(), Some(&7));
        assert!(!second.is_loading);
        assert!(!second.is_fetching, "fresh data must not trigger a refetch");
        assert_eq!(
            *calls.lock().expect("mutex"),
            1,
            "cache hit must not refetch"
        );
    }

    #[test]
    fn concurrent_reads_dedupe_to_one_fetch() {
        let (_rt, client) = client();
        let calls = Arc::new(Mutex::new(0_usize));

        for _ in 0..25 {
            let calls = calls.clone();
            let _ = client.query::<u32, _, String>("k", DEFAULT_STALE_TIME, move || async move {
                time::sleep(Duration::from_millis(60)).await;
                *calls.lock().expect("mutex") += 1;
                Ok::<_, String>(1_u32)
            });
        }
        wait_settled(&client, "k");
        assert_eq!(
            *calls.lock().expect("mutex"),
            1,
            "25 reads while in flight must spawn exactly 1 fetch"
        );
    }

    #[test]
    fn zero_stale_time_refetches_but_keeps_serving_cached_data() {
        let (_rt, client) = client();
        let calls = Arc::new(Mutex::new(0_usize));
        let spawn = |calls: Arc<Mutex<usize>>| {
            move || async move {
                *calls.lock().expect("mutex") += 1;
                Ok::<_, String>(9_u32)
            }
        };

        let _ = client.query::<u32, _, String>("k", Duration::ZERO, spawn(calls.clone()));
        wait_settled(&client, "k");

        let again = client.query::<u32, _, String>("k", Duration::ZERO, spawn(calls.clone()));
        assert_eq!(
            again.data.as_deref(),
            Some(&9),
            "stale-while-revalidate keeps data readable"
        );
        assert!(again.is_fetching, "a revalidation is in flight");
        assert!(
            !again.is_loading,
            "is_loading is first-load only, not revalidation"
        );
        wait_settled(&client, "k");
        assert_eq!(*calls.lock().expect("mutex"), 2);
    }

    #[test]
    fn failure_records_error_and_does_not_refetch_until_stale() {
        let (_rt, client) = client();
        let calls = Arc::new(Mutex::new(0_usize));
        let spawn = |calls: Arc<Mutex<usize>>| {
            move || async move {
                *calls.lock().expect("mutex") += 1;
                Err::<u32, String>("boom".to_string())
            }
        };

        let _ = client.query::<u32, _, String>("k", DEFAULT_STALE_TIME, spawn(calls.clone()));
        wait_settled(&client, "k");

        let result = client.query::<u32, _, String>("k", DEFAULT_STALE_TIME, spawn(calls.clone()));
        assert_eq!(result.error.as_deref(), Some("boom"));
        assert!(result.is_hard_error(), "no cached value to fall back on");
        assert!(
            !result.is_fetching,
            "a failed query must not retry every read"
        );
        assert_eq!(*calls.lock().expect("mutex"), 1);
    }

    #[test]
    fn invalidate_refetches_while_keeping_the_old_value_visible() {
        let (_rt, client) = client();
        let next = Arc::new(Mutex::new(1_u32));
        let spawn = |next: Arc<Mutex<u32>>| {
            move || async move {
                let mut n = next.lock().expect("mutex");
                let value = *n;
                *n += 1;
                Ok::<_, String>(value)
            }
        };

        let _ = client.query::<u32, _, String>("k", DEFAULT_STALE_TIME, spawn(next.clone()));
        wait_settled(&client, "k");
        client.invalidate("k");

        let during = client.query::<u32, _, String>("k", DEFAULT_STALE_TIME, spawn(next.clone()));
        assert_eq!(
            during.data.as_deref(),
            Some(&1),
            "old value stays visible during revalidation"
        );
        wait_settled(&client, "k");

        let after = client.query::<u32, _, String>("k", DEFAULT_STALE_TIME, spawn(next.clone()));
        assert_eq!(
            after.data.as_deref(),
            Some(&2),
            "refetched value replaces it"
        );
    }

    #[test]
    fn remove_clears_cached_data_entirely() {
        let (_rt, client) = client();
        let _ = client
            .query::<u32, _, String>("k", DEFAULT_STALE_TIME, || async { Ok::<_, String>(5_u32) });
        wait_settled(&client, "k");
        client.remove("k");

        let after = client
            .query::<u32, _, String>("k", DEFAULT_STALE_TIME, || async { Ok::<_, String>(5_u32) });
        assert!(
            after.data.is_none(),
            "remove drops the value, unlike invalidate"
        );
        assert!(after.is_loading);
    }
}
