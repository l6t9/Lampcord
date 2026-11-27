use crate::runtime;
use std::collections::BTreeSet;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::{Arc, RwLock, Weak};
use std::thread::Builder;
use std::time::Duration;
use std::time::Instant;

use sha2::{Digest, Sha256};
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Engine, Store, StoreLimitsBuilder};

use crate::manifest::Manifest;
use crate::runtime::bindings::{Plugin, PluginPre};
use crate::runtime::{EPOCH_TICK, PluginShared, State};
use crate::storage::KvStore;
use crate::{Error, HostServices};

pub(crate) type Job = Box<dyn FnOnce(&mut Worker) + Send>;

const INIT_BUDGET: Duration = Duration::from_secs(2);

pub(crate) struct Worker {
    pub manifest: Arc<Manifest>,
    engine: Engine,
    linker: Arc<Linker<State>>,
    package_dir: PathBuf,
    cache_dir: PathBuf,
    data_dir: PathBuf,
    db: requiem_db::Db,
    granted: Arc<RwLock<BTreeSet<String>>>,
    services: Arc<dyn HostServices>,
    shared: Arc<PluginShared>,
    host: Weak<crate::PluginHost>,

    pub previous_version: Option<String>,
    pre: Option<PluginPre<State>>,
    live: Option<Live>,
}

struct Live {
    store: Store<State>,
    plugin: Plugin,
}

pub(crate) struct WorkerConfig {
    pub manifest: Arc<Manifest>,
    pub engine: Engine,
    pub linker: Arc<Linker<State>>,
    pub package_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub data_dir: PathBuf,
    pub db: requiem_db::Db,
    pub granted: Arc<RwLock<BTreeSet<String>>>,
    pub services: Arc<dyn HostServices>,
    pub shared: Arc<PluginShared>,
    pub host: Weak<crate::PluginHost>,
    pub previous_version: Option<String>,
}

pub(crate) fn spawn(config: WorkerConfig) -> mpsc::Sender<Job> {
    let (tx, rx) = mpsc::channel::<Job>();
    let name = format!("plugin:{}", config.manifest.id);
    let mut worker = Worker {
        manifest: config.manifest,
        engine: config.engine,
        linker: config.linker,
        package_dir: config.package_dir,
        cache_dir: config.cache_dir,
        data_dir: config.data_dir,
        db: config.db,
        granted: config.granted,
        services: config.services,
        shared: config.shared,
        host: config.host,
        previous_version: config.previous_version,
        pre: None,
        live: None,
    };
    Builder::new()
        .name(name)
        .spawn(move || {
            while let Ok(job) = rx.recv() {
                job(&mut worker);
            }
            worker.stop();
        })
        .expect("spawning a plugin thread");
    tx
}

impl Worker {
    pub fn call<R>(
        &mut self,
        budget: Duration,
        f: impl FnOnce(&mut Store<State>, &Plugin) -> wasmtime::Result<R>,
    ) -> Result<R, Error> {
        self.start()?;
        let live = self.live.as_mut().expect("started above");
        live.store.set_epoch_deadline(ticks(budget));
        match f(&mut live.store, &live.plugin) {
            Ok(value) => Ok(value),
            Err(err) => {
                self.live = None;
                Err(trap_error(err, budget))
            }
        }
    }

    pub fn call_gesture<R>(
        &mut self,
        budget: Duration,
        f: impl FnOnce(&mut Store<State>, &Plugin) -> wasmtime::Result<R>,
    ) -> Result<R, Error> {
        self.shared.gesture.store(true, Ordering::Release);
        let result = self.call(budget, f);
        self.shared.gesture.store(false, Ordering::Release);
        result
    }

    pub fn start(&mut self) -> Result<(), Error> {
        if self.live.is_some() {
            return Ok(());
        }
        if self.pre.is_none() {
            let component = self.load_component()?;
            let instance_pre = self
                .linker
                .instantiate_pre(&component)
                .map_err(|err| Error::Wasm(format!("{err:#}")))?;
            self.pre =
                Some(PluginPre::new(instance_pre).map_err(|err| Error::Wasm(format!("{err:#}")))?);
        }
        let pre = self.pre.as_ref().expect("set above");
        let limits = StoreLimitsBuilder::new()
            .memory_size(self.manifest.limits.memory_mib as usize * 1024 * 1024)
            .instances(4)
            .tables(16)
            .memories(4)
            .build();
        let state = State {
            plugin_id: self.manifest.id.clone(),
            package_dir: self.package_dir.clone(),
            wasi: runtime::wasi_ctx(),
            table: ResourceTable::new(),
            limits,
            kv: KvStore::open(
                self.db.clone(),
                &self.manifest.id,
                self.data_dir.join("storage.bin"),
                self.manifest.limits.storage_kib,
            ),
            granted: self.granted.clone(),
            services: self.services.clone(),
            shared: self.shared.clone(),
            host: self.host.clone(),
        };

        *self
            .shared
            .registrations
            .lock()
            .expect("registrations lock poisoned") = Default::default();
        let mut store = Store::new(&self.engine, state);
        store.limiter(|state| &mut state.limits);
        store.epoch_deadline_trap();
        store.set_epoch_deadline(ticks(INIT_BUDGET));
        let plugin = pre
            .instantiate(&mut store)
            .map_err(|err| trap_error(err, INIT_BUDGET))?;
        let previous = self.previous_version.clone();
        let result = plugin
            .requiem_plugin_lifecycle()
            .call_init(&mut store, previous.as_deref());
        self.services.status_changed(&self.manifest.id);
        match result {
            Ok(Ok(())) => {}
            Ok(Err(err)) => {
                return Err(Error::Plugin(format!(
                    "init failed ({}): {}",
                    err.code, err.message
                )));
            }
            Err(err) => return Err(trap_error(err, INIT_BUDGET)),
        }
        self.previous_version = None;
        self.live = Some(Live { store, plugin });
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(mut live) = self.live.take() {
            live.store
                .set_epoch_deadline(ticks(Duration::from_millis(500)));
            if let Err(err) = live
                .plugin
                .requiem_plugin_lifecycle()
                .call_shutdown(&mut live.store)
            {
                log::warn!(target: "plugin", "[{}] shutdown failed: {err:#}", self.manifest.id);
            }
        }
        *self
            .shared
            .registrations
            .lock()
            .expect("registrations lock poisoned") = Default::default();
        self.shared
            .images
            .lock()
            .expect("images lock poisoned")
            .clear();
    }

    fn load_component(&self) -> Result<Component, Error> {
        let wasm_path = self.package_dir.join("plugin.wasm");
        let wasm = fs::read(&wasm_path)
            .map_err(|err| Error::Io(format!("{}: {err}", wasm_path.display())))?;
        let mut engine_hash = DefaultHasher::new();
        self.engine
            .precompile_compatibility_hash()
            .hash(&mut engine_hash);
        let digest = Sha256::digest(&wasm);
        let cached = self.cache_dir.join(format!(
            "{}-{:016x}.cwasm",
            hex(&digest[..12]),
            engine_hash.finish()
        ));
        if cached.exists() {
            // SAFETY: the cache file is only written below, from `Component::serialize` for this
            // engine, and its name embeds the wasm hash and the engine compatibility hash, so the
            // bytes are a trusted precompiled artifact for this configuration.
            match unsafe { Component::deserialize_file(&self.engine, &cached) } {
                Ok(component) => return Ok(component),
                Err(err) => {
                    log::warn!(target: "plugin", "[{}] ignoring a bad compile cache: {err:#}", self.manifest.id)
                }
            }
        }
        let started = Instant::now();
        let component =
            Component::new(&self.engine, &wasm).map_err(|err| Error::Wasm(format!("{err:#}")))?;
        log::info!(target: "plugin", "[{}] compiled in {:?}", self.manifest.id, started.elapsed());
        match component.serialize() {
            Ok(bytes) => {
                let _ = fs::create_dir_all(&self.cache_dir);
                if let Err(err) = fs::write(&cached, bytes) {
                    log::warn!(target: "plugin", "[{}] couldn't write the compile cache: {err}", self.manifest.id);
                }
            }
            Err(err) => {
                log::warn!(target: "plugin", "[{}] couldn't serialize the compile: {err:#}", self.manifest.id)
            }
        }
        Ok(component)
    }
}

fn ticks(budget: Duration) -> u64 {
    (budget.as_millis() / EPOCH_TICK.as_millis()).max(1) as u64 + 1
}

fn trap_error(err: wasmtime::Error, budget: Duration) -> Error {
    match err.downcast_ref::<wasmtime::Trap>() {
        Some(wasmtime::Trap::Interrupt) => Error::Timeout(budget),
        _ => Error::Trap(format!("{err:#}")),
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
