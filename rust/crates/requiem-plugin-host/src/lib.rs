pub mod id;
pub mod css;
pub mod data;
pub mod events;
pub mod manifest;
mod runtime;
pub mod services;
mod storage;
mod timers;
pub mod view;
mod worker;

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::io;
use std::mem;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock, Weak, mpsc};
use std::sync;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use wasmtime::Engine;
use wasmtime::component::{Linker, Resource};

pub use data::*;
pub use id::{
    ApplicationId, AttachmentId, ChannelId, EmojiId, GuildId, Id, MessageId, RoleId, UserId,
    WebhookId,
};
pub use manifest::Manifest;
pub use services::{HostError, HostResult, HostServices};
pub use view::{ViewNode, ViewTree};

use runtime::bindings::requiem::plugin::types::Verdict;
use runtime::{
    EventRes, IncomingChanges, IncomingMessageRes, MessageRes, NotificationRes, OutgoingEditRes,
    OutgoingMessageRes, OutgoingPresenceRes, PluginShared, State,
};
use worker::{Job, Worker, WorkerConfig};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid plugin.toml: {0}")]
    Manifest(String),
    #[error("needs a newer client ({0})")]
    NeedsNewerClient(String),
    #[error("{0}")]
    Io(String),
    #[error("not a valid plugin component: {0}")]
    Wasm(String),
    #[error("plugin crashed: {0}")]
    Trap(String),
    #[error("plugin took longer than {0:?}")]
    Timeout(Duration),
    #[error("{0}")]
    Plugin(String),
    #[error("no plugin `{0}`")]
    NotFound(String),
    #[error("plugin `{0}` is disabled")]
    Disabled(String),
}

pub mod budget {
    use std::time::Duration;

    pub const BEFORE_SEND: Duration = Duration::from_secs(3);
    pub const INCOMING_BATCH: Duration = Duration::from_millis(20);
    pub const NOTIFICATION: Duration = Duration::from_millis(50);
    pub const PRESENCE: Duration = Duration::from_millis(200);
    pub const MESSAGE_CLICK: Duration = Duration::from_millis(20);
    pub const RENDER: Duration = Duration::from_millis(250);
    pub const UI_ACTION: Duration = Duration::from_secs(2);
    pub const COMMAND: Duration = Duration::from_secs(3);
    pub const CONTEXT_MENU: Duration = Duration::from_millis(50);
    pub const AUTOCOMPLETE: Duration = Duration::from_millis(150);
    pub const EVENT: Duration = Duration::from_secs(1);
    pub const TIMER: Duration = Duration::from_secs(5);
    pub const AUDIO_FRAME: Duration = Duration::from_millis(2);

    pub const QUEUE_GRACE: Duration = Duration::from_secs(3);
}

pub const INCOMING_BATCH_SIZE: usize = 50;

const MAX_QUEUED: usize = 256;

const STRIKES: usize = 3;
const STRIKE_WINDOW: Duration = Duration::from_secs(60);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Idle,
    Running,

    Suspended(String),

    Failed(String),
    Disabled,
}

#[derive(Clone, Debug)]
pub struct PermissionInfo {
    pub name: String,
    pub reason: String,
    pub granted: bool,

    pub required: bool,

    pub tier: &'static str,
}

#[derive(Clone, Debug)]
pub struct PluginPreview {
    pub dir: PathBuf,
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,

    pub permissions: Vec<PermissionInfo>,

    pub installed_version: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
    pub enabled: bool,
    pub status: Status,
    pub permissions: Vec<PermissionInfo>,
    pub has_settings: bool,
    pub hooks: Vec<String>,
}

struct Entry {
    manifest: Arc<Manifest>,
    dir: PathBuf,
    sender: mpsc::Sender<Job>,
    enabled: AtomicBool,
    granted: Arc<RwLock<BTreeSet<String>>>,
    status: Mutex<Status>,
    strikes: Mutex<VecDeque<Instant>>,
    queued: Arc<AtomicUsize>,
    shared: Arc<PluginShared>,

    styles: String,
}

impl Entry {
    fn id(&self) -> &str {
        &self.manifest.id
    }

    fn status(&self) -> Status {
        self.status.lock().expect("status lock poisoned").clone()
    }

    fn set_status(&self, status: Status) -> bool {
        let mut current = self.status.lock().expect("status lock poisoned");
        let changed = *current != status;
        *current = status;
        changed
    }

    fn granted(&self, permission: &str) -> bool {
        self.granted
            .read()
            .expect("grants lock poisoned")
            .contains(permission)
    }

    fn active(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
            && !matches!(self.status(), Status::Suspended(_) | Status::Failed(_))
    }

    fn runs(&self, hook: &str) -> bool {
        if !self.active() || !self.manifest.has_hook(hook) {
            return false;
        }
        manifest::KNOWN_HOOKS
            .iter()
            .find(|(name, _)| *name == hook)
            .and_then(|(_, permission)| *permission)
            .is_none_or(|permission| self.granted(permission))
    }

    fn registrations(&self) -> sync::MutexGuard<'_, Registrations> {
        self.shared
            .registrations
            .lock()
            .expect("registrations lock poisoned")
    }
}

const STATE_DOCUMENT: &str = "plugin-state";

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct PersistedState {
    order: Vec<String>,
    plugins: BTreeMap<String, PersistedPlugin>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct PersistedPlugin {
    enabled: bool,
    granted: BTreeSet<String>,

    last_version: Option<String>,
}

pub struct PluginHost {
    root: PathBuf,
    engine: Engine,
    linker: Arc<Linker<State>>,
    services: Arc<dyn HostServices>,
    entries: RwLock<Vec<Arc<Entry>>>,
    db: requiem_db::Db,
    state: Mutex<PersistedState>,
    timers: timers::Timers,
    me: Weak<PluginHost>,
}

impl PluginHost {
    pub fn new(
        root: impl Into<PathBuf>,
        services: Arc<dyn HostServices>,
    ) -> Result<Arc<PluginHost>, Error> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(|err| Error::Io(format!("{}: {err}", root.display())))?;
        let db = requiem_db::Db::open(&root.join("plugins.db"))
            .map_err(|err| Error::Io(format!("{}: {err}", root.join("plugins.db").display())))?;
        Self::with_db(root, db, services)
    }

    pub fn with_db(
        root: impl Into<PathBuf>,
        db: requiem_db::Db,
        services: Arc<dyn HostServices>,
    ) -> Result<Arc<PluginHost>, Error> {
        let root = root.into();
        fs::create_dir_all(root.join("installed"))
            .map_err(|err| Error::Io(format!("{}: {err}", root.display())))?;
        let engine = runtime::engine().map_err(|err| Error::Wasm(format!("{err:#}")))?;
        let linker =
            Arc::new(runtime::linker(&engine).map_err(|err| Error::Wasm(format!("{err:#}")))?);
        db.import_once("plugin-state.json", |db| {
            let path = root.join("state.json");
            let Ok(text) = fs::read_to_string(&path) else {
                return Ok(());
            };
            serde_json::from_str::<serde_json::Value>(&text).map_err(|e| e.to_string())?;
            db.set_document(STATE_DOCUMENT, &text)
                .map_err(|e| e.to_string())?;
            let _ = fs::rename(&path, root.join("state.json.imported"));
            Ok(())
        });
        let state = db
            .document(STATE_DOCUMENT)
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        let host = Arc::new_cyclic(|me: &Weak<PluginHost>| PluginHost {
            root,
            db,
            engine,
            linker,
            services,
            entries: RwLock::new(Vec::new()),
            state: Mutex::new(state),
            timers: timers::Timers::start(me.clone()),
            me: me.clone(),
        });
        let installed = fs::read_dir(host.root.join("installed"))
            .map(|dir| {
                dir.flatten()
                    .map(|e| e.path())
                    .filter(|p| {
                        p.is_dir()
                            && !p
                                .file_name()
                                .is_some_and(|n| n.to_string_lossy().starts_with('.'))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for dir in installed {
            if let Err(err) = host.load(&dir) {
                log::error!(target: "plugin", "skipping {}: {err}", dir.display());
            }
        }
        Ok(host)
    }

    pub fn install_dir(&self, dir: &Path) -> Result<String, Error> {
        self.install_dir_with(dir, &[], false)
    }

    pub fn inspect_dir(&self, dir: &Path) -> Result<PluginPreview, Error> {
        let manifest_text = fs::read_to_string(dir.join("plugin.toml"))
            .map_err(|err| Error::Io(format!("{}: {err}", dir.join("plugin.toml").display())))?;
        let manifest = Manifest::parse(&manifest_text)?;
        if !dir.join("plugin.wasm").is_file() {
            return Err(Error::Io(format!(
                "{} is missing",
                dir.join("plugin.wasm").display()
            )));
        }
        let mut permissions = permission_infos(&manifest, |name| manifest.is_required(name));

        let installed = self.entry(&manifest.id).ok();
        if let Some(entry) = &installed {
            permissions
                .iter_mut()
                .for_each(|p| p.granted |= entry.granted(&p.name));
        }
        Ok(PluginPreview {
            dir: dir.to_path_buf(),
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            description: manifest.description.clone(),
            authors: manifest.authors.clone(),
            permissions,
            installed_version: installed.map(|e| e.manifest.version.clone()),
        })
    }

    pub fn install_dir_with(
        &self,
        dir: &Path,
        optional: &[String],
        enable: bool,
    ) -> Result<String, Error> {
        let manifest_text = fs::read_to_string(dir.join("plugin.toml"))
            .map_err(|err| Error::Io(format!("{}: {err}", dir.join("plugin.toml").display())))?;
        let manifest = Manifest::parse(&manifest_text)?;
        if !dir.join("plugin.wasm").is_file() {
            return Err(Error::Io(format!(
                "{} is missing",
                dir.join("plugin.wasm").display()
            )));
        }
        let target = self.root.join("installed").join(&manifest.id);
        let staging = self
            .root
            .join("installed")
            .join(format!(".{}.new", manifest.id));
        let _ = fs::remove_dir_all(&staging);
        fs::create_dir_all(&staging).map_err(|err| Error::Io(err.to_string()))?;
        for file in ["plugin.toml", "plugin.wasm", "styles.css", "README.md"] {
            let source = dir.join(file);
            if source.is_file() {
                fs::copy(&source, staging.join(file))
                    .map_err(|err| Error::Io(format!("{}: {err}", source.display())))?;
            }
        }
        if dir.join("assets").is_dir() {
            copy_dir(&dir.join("assets"), &staging.join("assets"))
                .map_err(|err| Error::Io(format!("assets: {err}")))?;
        }
        self.unload(&manifest.id);
        let _ = fs::remove_dir_all(&target);
        fs::rename(&staging, &target).map_err(|err| Error::Io(err.to_string()))?;
        {
            let mut state = self.state.lock().expect("state lock poisoned");
            let entry = state.plugins.entry(manifest.id.clone()).or_default();
            entry
                .granted
                .retain(|permission| manifest.asks_for(permission));
            entry.granted.extend(manifest.permissions.keys().cloned());
            entry.granted.extend(
                optional
                    .iter()
                    .filter(|p| manifest.optional_permissions.contains_key(*p))
                    .cloned(),
            );
            if enable {
                entry.enabled = true;
            }
        }
        self.save_state();
        self.load(&target)?;
        Ok(manifest.id)
    }

    pub fn remove(&self, id: &str) -> Result<(), Error> {
        if !self.unload(id) {
            return Err(Error::NotFound(id.to_string()));
        }
        let _ = fs::remove_dir_all(self.root.join("installed").join(id));
        let _ = fs::remove_dir_all(self.root.join("data").join(id));
        let _ = self.db.kv_clear(id);
        {
            let mut state = self.state.lock().expect("state lock poisoned");
            state.plugins.remove(id);
            state.order.retain(|o| o != id);
        }
        self.save_state();
        Ok(())
    }

    pub fn list(&self) -> Vec<PluginInfo> {
        self.ordered()
            .iter()
            .map(|entry| {
                let m = &entry.manifest;
                PluginInfo {
                    id: m.id.clone(),
                    name: m.name.clone(),
                    version: m.version.clone(),
                    description: m.description.clone(),
                    authors: m.authors.clone(),
                    enabled: entry.enabled.load(Ordering::Acquire),
                    status: entry.status(),
                    permissions: permission_infos(m, |name| entry.granted(name)),
                    has_settings: m.has_hook("settings-ui"),
                    hooks: m.hooks.keys().cloned().collect(),
                }
            })
            .collect()
    }

    pub fn set_enabled(&self, id: &str, enabled: bool) -> Result<(), Error> {
        let entry = self.entry(id)?;
        entry.enabled.store(enabled, Ordering::Release);
        self.state
            .lock()
            .expect("state lock poisoned")
            .plugins
            .entry(id.to_string())
            .or_default()
            .enabled = enabled;
        self.save_state();
        if enabled {
            entry.strikes.lock().expect("strikes lock poisoned").clear();
            entry.set_status(Status::Idle);
            self.warm_up(&entry);
        } else {
            entry.set_status(Status::Disabled);
            self.timers.clear_plugin(id);
            let _ = entry
                .sender
                .send(Box::new(|worker: &mut Worker| worker.stop()));
        }
        self.services.status_changed(id);
        Ok(())
    }

    pub fn set_permission(&self, id: &str, permission: &str, granted: bool) -> Result<(), Error> {
        let entry = self.entry(id)?;
        if !entry.manifest.asks_for(permission) {
            return Err(Error::Plugin(format!(
                "`{id}` doesn't ask for `{permission}`"
            )));
        }
        if entry.manifest.is_required(permission) {
            return if granted {
                Ok(())
            } else {
                Err(Error::Plugin(format!(
                    "`{permission}` is required by `{id}`"
                )))
            };
        }
        let mut grants = entry.granted.write().expect("grants lock poisoned");
        if granted {
            grants.insert(permission.to_string());
        } else {
            grants.remove(permission);
        }
        self.state
            .lock()
            .expect("state lock poisoned")
            .plugins
            .entry(id.to_string())
            .or_default()
            .granted = grants.clone();
        drop(grants);
        self.save_state();

        if entry.enabled.load(Ordering::Acquire) {
            self.timers.clear_plugin(id);
            let _ = entry
                .sender
                .send(Box::new(|worker: &mut Worker| worker.stop()));
            entry.set_status(Status::Idle);
            self.warm_up(&entry);
        }
        self.services.status_changed(id);
        Ok(())
    }

    pub fn set_order(&self, ids: Vec<String>) {
        self.state.lock().expect("state lock poisoned").order = ids;
        self.save_state();
    }

    pub fn runs_any(&self, hook: &str) -> bool {
        self.entries
            .read()
            .expect("entries lock poisoned")
            .iter()
            .any(|e| e.runs(hook))
    }

    pub fn stylesheet(&self) -> String {
        self.ordered()
            .iter()
            .filter(|e| e.enabled.load(Ordering::Acquire) && !e.styles.is_empty())
            .map(|e| format!("/* plugin {} */\n{}", e.id(), e.styles))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn registrations(&self) -> Vec<(String, Registrations)> {
        self.ordered()
            .into_iter()
            .filter(|e| e.active())
            .map(|e| (e.id().to_string(), e.registrations().clone()))
            .collect()
    }

    pub fn image(&self, plugin: &str, src: &str) -> Option<Vec<u8>> {
        let entry = self.entry(plugin).ok()?;
        let images = entry.shared.images.lock().expect("images lock poisoned");
        images.get(src).cloned()
    }

    pub fn asset_path(&self, plugin: &str, path: &str) -> Option<PathBuf> {
        let entry = self.entry(plugin).ok()?;
        if path
            .split(['/', '\\'])
            .any(|part| part == ".." || part.is_empty())
        {
            return None;
        }
        let full = entry.dir.join("assets").join(path);
        full.is_file().then_some(full)
    }

    pub fn run_outgoing(&self, draft: OutgoingDraft) -> OutgoingOutcome<OutgoingDraft> {
        let sender = draft.sent_by_plugin.clone();
        self.run_draft_pipeline(draft, sender.as_deref(), |worker, draft| {
            worker.call(budget::BEFORE_SEND, move |store, plugin| {
                let shared = Arc::new(Mutex::new(draft));
                let resource = store
                    .data_mut()
                    .table
                    .push(OutgoingMessageRes(shared.clone()))?;
                let verdict = plugin
                    .requiem_plugin_outgoing()
                    .call_before_send(&mut *store, Resource::new_borrow(resource.rep()))?;
                store.data_mut().table.delete(resource)?;
                let draft = shared.lock().expect("draft lock poisoned").clone();
                Ok((verdict, draft))
            })
        })
    }

    pub fn run_edit(&self, draft: EditDraft) -> OutgoingOutcome<EditDraft> {
        self.run_draft_pipeline(draft, None, |worker, draft| {
            worker.call(budget::BEFORE_SEND, move |store, plugin| {
                let resource = store.data_mut().table.push(OutgoingEditRes(draft))?;
                let verdict = plugin
                    .requiem_plugin_outgoing()
                    .call_before_edit(&mut *store, Resource::new_borrow(resource.rep()))?;
                let draft = store.data_mut().table.delete(resource)?.0;
                Ok((verdict, draft))
            })
        })
    }

    fn run_draft_pipeline<D: Clone + PartialEq + Send + 'static>(
        &self,
        mut draft: D,
        skip: Option<&str>,
        call: fn(&mut Worker, D) -> Result<(Verdict, D), Error>,
    ) -> OutgoingOutcome<D> {
        let mut modified_by = Vec::new();
        for entry in self
            .ordered()
            .into_iter()
            .filter(|e| e.runs("outgoing") && Some(e.id()) != skip)
        {
            let input = draft.clone();
            match self.call(&entry, budget::BEFORE_SEND, move |worker| {
                call(worker, input)
            }) {
                Ok((Verdict::Pass, changed)) => {
                    if changed != draft {
                        modified_by.push(entry.id().to_string());
                        draft = changed;
                    }
                }
                Ok((Verdict::Cancel(reason), _)) => {
                    return OutgoingOutcome {
                        draft,
                        cancelled: Some(ByPlugin {
                            plugin: entry.id().to_string(),
                            value: reason,
                        }),
                        modified_by,
                    };
                }
                Err(err) => {
                    log::warn!(target: "plugin", "[{}] outgoing hook skipped: {err}", entry.id())
                }
            }
        }
        OutgoingOutcome {
            draft,
            cancelled: None,
            modified_by,
        }
    }

    pub fn run_presence(&self, mut draft: PresenceDraft) -> OutgoingOutcome<PresenceDraft> {
        let mut modified_by = Vec::new();
        for entry in self.ordered().into_iter().filter(|e| e.runs("presence")) {
            let input = draft.clone();
            let result = self.call(&entry, budget::PRESENCE, move |worker| {
                worker.call(budget::PRESENCE, move |store, plugin| {
                    let resource = store.data_mut().table.push(OutgoingPresenceRes(input))?;
                    let verdict = plugin
                        .requiem_plugin_outgoing()
                        .call_before_presence_update(
                            &mut *store,
                            Resource::new_borrow(resource.rep()),
                        )?;
                    Ok((verdict, store.data_mut().table.delete(resource)?.0))
                })
            });
            match result {
                Ok((Verdict::Pass, changed)) => {
                    if changed != draft {
                        modified_by.push(entry.id().to_string());
                        draft = changed;
                    }
                }
                Ok((Verdict::Cancel(reason), _)) => {
                    return OutgoingOutcome {
                        draft,
                        cancelled: Some(ByPlugin {
                            plugin: entry.id().to_string(),
                            value: reason,
                        }),
                        modified_by,
                    };
                }
                Err(err) => {
                    log::warn!(target: "plugin", "[{}] presence hook skipped: {err}", entry.id())
                }
            }
        }
        OutgoingOutcome {
            draft,
            cancelled: None,
            modified_by,
        }
    }

    pub fn run_notification(&self, draft: NotificationDraft) -> OutgoingOutcome<NotificationDraft> {
        let mut draft = draft;
        let mut modified_by = Vec::new();
        for entry in self
            .ordered()
            .into_iter()
            .filter(|e| e.runs("notifications"))
        {
            let input = draft.clone();
            let result = self.call(&entry, budget::NOTIFICATION, move |worker| {
                worker.call(budget::NOTIFICATION, move |store, plugin| {
                    let resource = store.data_mut().table.push(NotificationRes(input))?;
                    let verdict = plugin
                        .requiem_plugin_incoming()
                        .call_before_notify(&mut *store, Resource::new_borrow(resource.rep()))?;
                    Ok((verdict, store.data_mut().table.delete(resource)?.0))
                })
            });
            match result {
                Ok((Verdict::Pass, changed)) => {
                    if changed != draft {
                        modified_by.push(entry.id().to_string());
                        draft = changed;
                    }
                }
                Ok((Verdict::Cancel(reason), _)) => {
                    return OutgoingOutcome {
                        draft,
                        cancelled: Some(ByPlugin {
                            plugin: entry.id().to_string(),
                            value: reason,
                        }),
                        modified_by,
                    };
                }
                Err(err) => {
                    log::warn!(target: "plugin", "[{}] notification hook skipped: {err}", entry.id())
                }
            }
        }
        OutgoingOutcome {
            draft,
            cancelled: None,
            modified_by,
        }
    }

    pub fn run_incoming(&self, items: Vec<IncomingItem>) -> Vec<IncomingOutcome> {
        let mut outcomes = vec![IncomingOutcome::default(); items.len()];
        let plugins: Vec<_> = self
            .ordered()
            .into_iter()
            .filter(|e| e.runs("incoming"))
            .collect();
        if plugins.is_empty() {
            return outcomes;
        }
        let items: Vec<(Arc<MessageData>, Option<Arc<MessageData>>, IncomingSource)> = items
            .into_iter()
            .map(|i| (Arc::new(i.message), i.previous.map(Arc::new), i.source))
            .collect();
        for entry in plugins {
            for (chunk_index, chunk) in items.chunks(INCOMING_BATCH_SIZE).enumerate() {
                let batch: Vec<_> = chunk.to_vec();
                let result = self.call(&entry, budget::INCOMING_BATCH, move |worker| {
                    worker.call(budget::INCOMING_BATCH, move |store, plugin| {
                        let mut changes = Vec::with_capacity(batch.len());
                        let mut handles = Vec::with_capacity(batch.len());
                        for (message, previous, source) in batch {
                            let shared = Arc::new(Mutex::new(IncomingChanges::default()));
                            handles.push(store.data_mut().table.push(IncomingMessageRes {
                                message,
                                previous,
                                source: source.as_str(),
                                changes: shared.clone(),
                            })?);
                            changes.push(shared);
                        }
                        let verdicts = plugin
                            .requiem_plugin_incoming()
                            .call_transform(&mut *store, &handles)?;
                        Ok((verdicts, changes))
                    })
                });
                let (verdicts, changes) = match result {
                    Ok(result) => result,
                    Err(err) => {
                        log::warn!(target: "plugin", "[{}] incoming hook skipped: {err}", entry.id());
                        continue;
                    }
                };
                let id = entry.id().to_string();
                let by = |value| ByPlugin {
                    plugin: id.clone(),
                    value,
                };
                for (i, change) in changes.into_iter().enumerate() {
                    let index = chunk_index * INCOMING_BATCH_SIZE + i;
                    let outcome = &mut outcomes[index];
                    let source = items[index].2;
                    let change = mem::take(
                        &mut *change.lock().expect("incoming changes lock poisoned"),
                    );
                    if let Some(Verdict::Cancel(reason)) = verdicts.get(i) {
                        if source == IncomingSource::GatewayDelete {
                            outcome.kept_deleted.get_or_insert(by(change
                                .deleted_label
                                .clone()
                                .unwrap_or_else(|| reason.clone())));
                        } else {
                            outcome.hidden.get_or_insert(by(reason.clone()));
                        }
                    }
                    if let Some(content) = change.display_content {
                        outcome.display_content = Some(by(content));
                    }
                    if let Some(text) = change.annotation {
                        outcome.annotations.push(by(text));
                    }
                    outcome
                        .spans
                        .extend(change.spans.into_iter().map(|s| ByPlugin {
                            plugin: id.clone(),
                            value: s,
                        }));
                    outcome
                        .decorations
                        .extend(change.decorations.into_iter().filter_map(|(slot, view)| {
                            view.map(|v| ByPlugin {
                                plugin: id.clone(),
                                value: (slot, v),
                            })
                        }));
                    outcome
                        .embed_states
                        .extend(change.embed_states.into_iter().map(|s| ByPlugin {
                            plugin: id.clone(),
                            value: s,
                        }));
                    outcome
                        .attachment_states
                        .extend(change.attachment_states.into_iter().map(|s| ByPlugin {
                            plugin: id.clone(),
                            value: s,
                        }));
                    outcome
                        .author_display
                        .extend(change.author_display.into_iter().map(|s| ByPlugin {
                            plugin: id.clone(),
                            value: s,
                        }));
                }
            }
        }
        outcomes
    }

    pub fn message_click(&self, message: MessageData, button: &str, modifiers: &str) -> bool {
        let message = Arc::new(message);
        for entry in self
            .ordered()
            .into_iter()
            .filter(|e| e.runs("message-click"))
        {
            let (message, button, modifiers) =
                (message.clone(), button.to_string(), modifiers.to_string());
            let handled = self.call(&entry, budget::MESSAGE_CLICK, move |worker| {
                worker.call_gesture(budget::MESSAGE_CLICK, move |store, plugin| {
                    let resource = store.data_mut().table.push(MessageRes(message))?;
                    plugin.requiem_plugin_interactions().call_on_message_click(
                        &mut *store,
                        resource,
                        &button,
                        &modifiers,
                    )
                })
            });
            if handled.unwrap_or(false) {
                return true;
            }
        }
        false
    }

    pub fn process_audio(
        &self,
        stream: &str,
        sample_rate: u32,
        channels: u8,
        frame: Vec<f32>,
    ) -> Vec<f32> {
        let mut frame = frame;
        for entry in self.ordered().into_iter().filter(|e| {
            e.active()
                && e.granted("audio.process")
                && e.registrations().audio_streams.iter().any(|s| s == stream)
        }) {
            let (input, stream) = (frame.clone(), stream.to_string());
            let len = input.len();
            let result = self.call(&entry, budget::AUDIO_FRAME, move |worker| {
                worker.call(budget::AUDIO_FRAME, move |store, plugin| {
                    plugin.requiem_plugin_providers().call_process_audio(
                        &mut *store,
                        &stream,
                        sample_rate,
                        channels,
                        &input,
                    )
                })
            });
            match result {
                Ok(out) if out.len() == len => frame = out,
                Ok(_) => {
                    log::debug!(target: "plugin", "[{}] process-audio returned the wrong length", entry.id())
                }
                Err(_) => {}
            }
        }
        frame
    }

    pub fn dispatch_event(&self, event: EventData) {
        let permission = match events::permission_for(&event.kind) {
            Some(permission) => permission,
            None => {
                log::warn!(target: "plugin", "dispatching unknown event kind `{}`", event.kind);
                return;
            }
        };
        let event = Arc::new(event);
        for entry in self.ordered() {
            if !entry.active()
                || permission.is_some_and(|p| !entry.granted(p))
                || !entry.registrations().events.contains(&event.kind)
            {
                continue;
            }
            let event = event.clone();
            self.post(&entry, move |worker| {
                worker.call(budget::EVENT, move |store, plugin| {
                    let resource = store.data_mut().table.push(EventRes(event))?;
                    plugin
                        .requiem_plugin_events()
                        .call_on_event(&mut *store, resource)
                })
            });
        }
    }

    pub(crate) fn post_timer(&self, plugin: &str, id: &str) {
        let Ok(entry) = self.entry(plugin) else {
            return;
        };
        if !entry.active() {
            return;
        }
        let id = id.to_string();
        self.post(&entry, move |worker| {
            worker.call(budget::TIMER, move |store, plugin| {
                plugin
                    .requiem_plugin_events()
                    .call_on_timer(&mut *store, &id)
            })
        });
    }

    pub(crate) fn post_websocket_event(&self, plugin: &str, id: u32, kind: &str, data: Vec<u8>) {
        let Ok(entry) = self.entry(plugin) else {
            return;
        };
        if !entry.active() {
            return;
        }
        let kind = kind.to_string();
        self.post(&entry, move |worker| {
            worker.call(budget::EVENT, move |store, plugin| {
                plugin
                    .requiem_plugin_events()
                    .call_on_websocket(&mut *store, id, &kind, &data)
            })
        });
    }

    pub fn render_view(
        &self,
        plugin: &str,
        view_id: &str,
        context: &str,
    ) -> Result<ViewTree, Error> {
        let entry = self.active_entry(plugin)?;
        let (view_id, context) = (view_id.to_string(), context.to_string());
        self.call(&entry, budget::RENDER, move |worker| {
            worker.call(budget::RENDER, move |store, plugin| {
                render(store, plugin, &view_id, &context)
            })
        })
    }

    pub fn ui_action(
        &self,
        plugin: &str,
        view_id: &str,
        action_id: &str,
        value: Option<serde_json::Value>,
        rerender: Option<&str>,
    ) -> Result<Option<ViewTree>, Error> {
        let entry = self.active_entry(plugin)?;
        let (view_id, action_id, value, rerender) = (
            view_id.to_string(),
            action_id.to_string(),
            value.map(|v| v.to_string()),
            rerender.map(str::to_string),
        );
        self.call(&entry, budget::UI_ACTION + budget::RENDER, move |worker| {
            worker.call_gesture(budget::UI_ACTION, |store, plugin| {
                plugin.requiem_plugin_interactions().call_on_ui_action(
                    &mut *store,
                    &view_id,
                    &action_id,
                    value.as_ref(),
                )
            })?;
            match rerender {
                Some(context) => worker
                    .call(budget::RENDER, |store, plugin| {
                        render(store, plugin, &view_id, &context)
                    })
                    .map(Some),
                None => Ok(None),
            }
        })
    }

    pub fn run_command(
        &self,
        plugin: &str,
        name: &str,
        options: &str,
        context: &str,
    ) -> Result<Result<(), String>, Error> {
        let entry = self.active_entry(plugin)?;
        let (name, options, context) = (name.to_string(), options.to_string(), context.to_string());
        self.call(&entry, budget::COMMAND, move |worker| {
            worker.call_gesture(budget::COMMAND, move |store, plugin| {
                plugin.requiem_plugin_interactions().call_on_command(
                    &mut *store,
                    &name,
                    &options,
                    &context,
                )
            })
        })
    }

    pub fn menu_item(
        &self,
        plugin: &str,
        menu: &str,
        item_id: &str,
        target: &str,
        checked: Option<bool>,
    ) -> Result<(), Error> {
        let entry = self.active_entry(plugin)?;
        let (menu, item_id, target) = (menu.to_string(), item_id.to_string(), target.to_string());
        self.call(&entry, budget::UI_ACTION, move |worker| {
            worker.call_gesture(budget::UI_ACTION, move |store, plugin| {
                plugin.requiem_plugin_interactions().call_on_menu_item(
                    &mut *store,
                    &menu,
                    &item_id,
                    &target,
                    checked,
                )
            })
        })
    }

    pub fn context_menu(&self, menu: &str, target: &str) -> Vec<(String, MenuStateData)> {
        let mut out = Vec::new();
        for entry in self.ordered() {
            if !entry.active()
                || !entry
                    .registrations()
                    .menu_items
                    .iter()
                    .any(|i| i.menu == menu && i.dynamic)
            {
                continue;
            }
            let (menu, target) = (menu.to_string(), target.to_string());
            let result = self.call(&entry, budget::CONTEXT_MENU, move |worker| {
                worker.call(budget::CONTEXT_MENU, move |store, plugin| {
                    let states = plugin.requiem_plugin_providers().call_context_menu(
                        &mut *store,
                        &menu,
                        &target,
                    )?;
                    states
                        .into_iter()
                        .map(|s| Ok(store.data_mut().table.delete(s)?.0))
                        .collect::<wasmtime::Result<Vec<_>>>()
                })
            });
            match result {
                Ok(states) => out.extend(states.into_iter().map(|s| (entry.id().to_string(), s))),
                Err(err) => {
                    log::warn!(target: "plugin", "[{}] context-menu skipped: {err}", entry.id())
                }
            }
        }
        out
    }

    pub fn slot_action(
        &self,
        plugin: &str,
        slot: &str,
        item_id: &str,
        context: &str,
    ) -> Result<(), Error> {
        let entry = self.active_entry(plugin)?;
        let (slot, item_id, context) = (slot.to_string(), item_id.to_string(), context.to_string());
        self.call(&entry, budget::UI_ACTION, move |worker| {
            worker.call_gesture(budget::UI_ACTION, move |store, plugin| {
                plugin.requiem_plugin_interactions().call_on_slot_action(
                    &mut *store,
                    &slot,
                    &item_id,
                    &context,
                )
            })
        })
    }

    pub fn shortcut(&self, plugin: &str, id: &str) -> Result<(), Error> {
        let entry = self.active_entry(plugin)?;
        let id = id.to_string();
        self.call(&entry, budget::UI_ACTION, move |worker| {
            worker.call_gesture(budget::UI_ACTION, move |store, plugin| {
                plugin
                    .requiem_plugin_interactions()
                    .call_on_shortcut(&mut *store, &id)
            })
        })
    }

    pub fn command_autocomplete(
        &self,
        plugin: &str,
        command: &str,
        option: &str,
        partial: &str,
        context: &str,
    ) -> Result<Vec<SuggestionData>, Error> {
        let entry = self.active_entry(plugin)?;
        let (command, option, partial, context) = (
            command.to_string(),
            option.to_string(),
            partial.to_string(),
            context.to_string(),
        );
        self.call(&entry, budget::AUTOCOMPLETE, move |worker| {
            worker.call(budget::AUTOCOMPLETE, move |store, plugin| {
                let list = plugin
                    .requiem_plugin_providers()
                    .call_command_autocomplete(
                        &mut *store,
                        &command,
                        &option,
                        &partial,
                        &context,
                    )?;
                take_suggestions(store, list)
            })
        })
    }

    pub fn autocomplete(
        &self,
        trigger: &str,
        query: &str,
        context: &str,
    ) -> Vec<(String, SuggestionData)> {
        let mut out = Vec::new();
        for entry in self.ordered() {
            if !entry.active()
                || !entry
                    .registrations()
                    .autocomplete_triggers
                    .iter()
                    .any(|t| t == trigger)
            {
                continue;
            }
            let (trigger, query, context) =
                (trigger.to_string(), query.to_string(), context.to_string());
            let result = self.call(&entry, budget::AUTOCOMPLETE, move |worker| {
                worker.call(budget::AUTOCOMPLETE, move |store, plugin| {
                    let list = plugin.requiem_plugin_providers().call_autocomplete(
                        &mut *store,
                        &trigger,
                        &query,
                        &context,
                    )?;
                    take_suggestions(store, list)
                })
            });
            if let Ok(list) = result {
                out.extend(
                    list.into_iter()
                        .take(25)
                        .map(|s| (entry.id().to_string(), s)),
                );
            }
        }
        out
    }

    pub fn search(
        &self,
        plugin: &str,
        provider: &str,
        query: &str,
    ) -> Result<Vec<SuggestionData>, Error> {
        let entry = self.active_entry(plugin)?;
        let (provider, query) = (provider.to_string(), query.to_string());
        self.call(&entry, budget::AUTOCOMPLETE, move |worker| {
            worker.call(budget::AUTOCOMPLETE, move |store, plugin| {
                let list = plugin.requiem_plugin_providers().call_search(
                    &mut *store,
                    &provider,
                    &query,
                )?;
                take_suggestions(store, list)
            })
        })
    }

    pub fn render_markdown(
        &self,
        plugin: &str,
        rule: &str,
        captures: Vec<String>,
        context: &str,
    ) -> Result<ViewTree, Error> {
        let entry = self.active_entry(plugin)?;
        let (rule, context) = (rule.to_string(), context.to_string());
        self.call(&entry, budget::RENDER, move |worker| {
            worker.call(budget::RENDER, move |store, plugin| {
                let builder = plugin.requiem_plugin_providers().call_render_markdown(
                    &mut *store,
                    &rule,
                    &captures,
                    &context,
                )?;
                finish_view(store, builder)
            })
        })
    }

    fn load(&self, dir: &Path) -> Result<(), Error> {
        let manifest = Arc::new(Manifest::parse(
            &fs::read_to_string(dir.join("plugin.toml"))
                .map_err(|err| Error::Io(err.to_string()))?,
        )?);
        let id = manifest.id.clone();
        if self
            .entries
            .read()
            .expect("entries lock poisoned")
            .iter()
            .any(|e| e.id() == id)
        {
            return Err(Error::Plugin(format!("`{id}` is installed twice")));
        }
        let (enabled, granted, previous_version) = {
            let state = self.state.lock().expect("state lock poisoned");
            let persisted = state.plugins.get(&id);
            let last = persisted.and_then(|p| p.last_version.clone());
            (
                persisted.is_some_and(|p| p.enabled),
                persisted.map(|p| p.granted.clone()).unwrap_or_default(),
                last.filter(|v| *v != manifest.version),
            )
        };

        self.state
            .lock()
            .expect("state lock poisoned")
            .plugins
            .entry(id.clone())
            .or_default()
            .last_version = Some(manifest.version.clone());
        self.save_state();
        let styles = match fs::read_to_string(dir.join("styles.css")) {
            Ok(css) => {
                let scoped = css::scope_stylesheet(&id, &css);
                for warning in &scoped.warnings {
                    self.services
                        .log(&id, LogLevel::Warn, &format!("styles.css: {warning}"));
                }
                scoped.css
            }
            Err(_) => String::new(),
        };
        let mut granted = granted;
        granted.extend(manifest.permissions.keys().cloned());
        let granted = Arc::new(RwLock::new(granted));
        let shared = Arc::new(PluginShared::default());
        let sender = worker::spawn(WorkerConfig {
            manifest: manifest.clone(),
            engine: self.engine.clone(),
            linker: self.linker.clone(),
            package_dir: dir.to_path_buf(),
            cache_dir: self.root.join("cache"),
            data_dir: self.root.join("data").join(&id),
            db: self.db.clone(),
            granted: granted.clone(),
            services: self.services.clone(),
            shared: shared.clone(),
            host: self.me.clone(),
            previous_version,
        });
        let entry = Arc::new(Entry {
            manifest,
            dir: dir.to_path_buf(),
            sender,
            enabled: AtomicBool::new(enabled),
            granted,
            status: Mutex::new(if enabled {
                Status::Idle
            } else {
                Status::Disabled
            }),
            strikes: Mutex::new(VecDeque::new()),
            queued: Arc::new(AtomicUsize::new(0)),
            shared,
            styles,
        });
        self.entries
            .write()
            .expect("entries lock poisoned")
            .push(entry.clone());
        if enabled {
            self.warm_up(&entry);
        }
        Ok(())
    }

    fn unload(&self, id: &str) -> bool {
        self.timers.clear_plugin(id);
        let mut entries = self.entries.write().expect("entries lock poisoned");
        let before = entries.len();
        entries.retain(|e| e.id() != id);
        before != entries.len()
    }

    fn warm_up(&self, entry: &Arc<Entry>) {
        let entry_for_job = entry.clone();
        let services = self.services.clone();
        let _ = entry.sender.send(Box::new(move |worker: &mut Worker| {
            let status = match worker.start() {
                Ok(()) => Status::Running,
                Err(err) => {
                    services.log(
                        entry_for_job.id(),
                        LogLevel::Error,
                        &format!("failed to start: {err}"),
                    );
                    Status::Failed(err.to_string())
                }
            };
            if entry_for_job.set_status(status) {
                services.status_changed(entry_for_job.id());
            }
        }));
    }

    fn entry(&self, id: &str) -> Result<Arc<Entry>, Error> {
        self.entries
            .read()
            .expect("entries lock poisoned")
            .iter()
            .find(|e| e.id() == id)
            .cloned()
            .ok_or_else(|| Error::NotFound(id.to_string()))
    }

    fn active_entry(&self, id: &str) -> Result<Arc<Entry>, Error> {
        let entry = self.entry(id)?;
        if !entry.active() {
            return Err(Error::Disabled(id.to_string()));
        }
        Ok(entry)
    }

    fn ordered(&self) -> Vec<Arc<Entry>> {
        let order = self
            .state
            .lock()
            .expect("state lock poisoned")
            .order
            .clone();
        let mut entries = self.entries.read().expect("entries lock poisoned").clone();
        entries.sort_by_key(|e| {
            let position = order
                .iter()
                .position(|id| *id == e.manifest.id)
                .unwrap_or(usize::MAX);
            let priority = e
                .manifest
                .hooks
                .values()
                .map(|h| h.priority)
                .min()
                .unwrap_or(0);
            (position, priority, e.manifest.id.clone())
        });
        entries
    }

    fn call<R: Send + 'static>(
        &self,
        entry: &Arc<Entry>,
        budget: Duration,
        job: impl FnOnce(&mut Worker) -> Result<R, Error> + Send + 'static,
    ) -> Result<R, Error> {
        let (tx, rx) = mpsc::sync_channel(1);
        let sent = entry.sender.send(Box::new(move |worker: &mut Worker| {
            let _ = tx.send(job(worker));
        }));
        if sent.is_err() {
            return Err(Error::Plugin("the plugin's thread is gone".to_string()));
        }
        let result = rx
            .recv_timeout(budget + budget::QUEUE_GRACE)
            .unwrap_or(Err(Error::Timeout(budget + budget::QUEUE_GRACE)));
        self.record(entry, &result);
        result
    }

    fn post(
        &self,
        entry: &Arc<Entry>,
        job: impl FnOnce(&mut Worker) -> Result<(), Error> + Send + 'static,
    ) {
        if entry.queued.fetch_add(1, Ordering::AcqRel) >= MAX_QUEUED {
            entry.queued.fetch_sub(1, Ordering::AcqRel);
            log::warn!(target: "plugin", "[{}] too many queued events; dropping one", entry.id());
            return;
        }
        let (host, entry_for_job) = (self.me.clone(), entry.clone());
        let _ = entry.sender.send(Box::new(move |worker: &mut Worker| {
            entry_for_job.queued.fetch_sub(1, Ordering::AcqRel);
            let result = job(worker);
            if let Some(host) = host.upgrade() {
                host.record(&entry_for_job, &result);
            }
        }));
    }

    fn record<R>(&self, entry: &Arc<Entry>, result: &Result<R, Error>) {
        match result {
            Ok(_) => {
                if entry.enabled.load(Ordering::Acquire)
                    && matches!(entry.status(), Status::Idle | Status::Failed(_))
                    && entry.set_status(Status::Running)
                {
                    self.services.status_changed(entry.id());
                }
            }
            Err(err @ (Error::Trap(_) | Error::Timeout(_) | Error::Plugin(_) | Error::Wasm(_))) => {
                self.strike(entry, err)
            }
            Err(_) => {}
        }
    }

    fn strike(&self, entry: &Arc<Entry>, err: &Error) {
        let id = entry.id();
        self.services.log(id, LogLevel::Error, &err.to_string());
        let mut strikes = entry.strikes.lock().expect("strikes lock poisoned");
        let now = Instant::now();
        strikes.push_back(now);
        while strikes
            .front()
            .is_some_and(|t| now.duration_since(*t) > STRIKE_WINDOW)
        {
            strikes.pop_front();
        }
        if strikes.len() >= STRIKES && entry.set_status(Status::Suspended(err.to_string())) {
            self.services.log(
                id,
                LogLevel::Error,
                "suspended after repeated failures; re-enable it in Settings > Plugins",
            );
            self.timers.clear_plugin(id);
            self.services.status_changed(id);
        }
    }

    fn save_state(&self) {
        let json = serde_json::to_string(&*self.state.lock().expect("state lock poisoned"))
            .expect("plugin state serializes");
        if let Err(err) = self.db.set_document(STATE_DOCUMENT, &json) {
            log::error!(target: "plugin", "saving the plugin state: {err}");
        }
    }
}

fn render(
    store: &mut wasmtime::Store<State>,
    plugin: &runtime::bindings::Plugin,
    view_id: &str,
    context: &str,
) -> wasmtime::Result<ViewTree> {
    let builder = plugin.requiem_plugin_views().call_render(
        &mut *store,
        view_id,
        &context.to_string(),
    )?;
    finish_view(store, builder)
}

fn finish_view(
    store: &mut wasmtime::Store<State>,
    builder: Resource<runtime::ViewBuilderRes>,
) -> wasmtime::Result<ViewTree> {
    let state = store.data_mut().table.delete(builder)?.0;
    if state.overflowed {
        log::warn!(target: "plugin", "[{}] a view has more than {} nodes; the rest was dropped", store.data().plugin_id, view::MAX_NODES);
    }
    Ok(state.tree)
}

fn take_suggestions(
    store: &mut wasmtime::Store<State>,
    list: Vec<Resource<runtime::SuggestionRes>>,
) -> wasmtime::Result<Vec<SuggestionData>> {
    list.into_iter()
        .map(|s| Ok(store.data_mut().table.delete(s)?.0))
        .collect()
}

fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn permission_infos(manifest: &Manifest, granted: impl Fn(&str) -> bool) -> Vec<PermissionInfo> {
    let info = |name: &String, reason: &String, required: bool| PermissionInfo {
        name: name.clone(),
        reason: reason.clone(),
        granted: required || granted(name),
        required,
        tier: manifest::permission_tier(name).unwrap_or("dangerous"),
    };
    let required = manifest
        .permissions
        .iter()
        .map(|(name, reason)| info(name, reason, true));
    required
        .chain(
            manifest
                .optional_permissions
                .iter()
                .map(|(name, reason)| info(name, reason, false)),
        )
        .collect()
}
