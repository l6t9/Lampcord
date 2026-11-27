use requiem_core::voice::Call;
use requiem_core::voice;
use crate::features::user_settings::update_settings;
use crate::uninstall_entry;
use requiem_updater::custom;
use requiem_updater::resume;
use requiem_updater::run;
use requiem_updater::sources;
use requiem_updater::staged;
use std::env;
use std::io::Error;
use std::io;
use std::path::PathBuf;
use std::process;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::Duration;
use tokio::runtime;
use tokio::time;

use requiem_core::ui::context_menu::{ConfirmRequest, confirm, notify_failure};
use requiem_ui::signal::{Signal, signal};
use requiem_updater::apply;
use requiem_updater::check::Config;
use requiem_updater::run::{Handle, State, Updater};
use requiem_updater::staged::Staged;

const HEALTHY_AFTER: Duration = Duration::from_secs(15);

pub const POST_UPDATE_FLAG: &str = "--post-update";

pub const APPLY_FLAG: &str = "--apply-update";

pub fn install_dir() -> Option<PathBuf> {
    env::current_exe().ok()?.parent().map(PathBuf::from)
}

pub fn update_root() -> Option<PathBuf> {
    install_dir().map(|dir| dir.join(".update"))
}

pub fn exe_name() -> String {
    if cfg!(windows) {
        "requiem.exe"
    } else {
        "requiem"
    }
    .to_string()
}

pub fn current_version() -> semver::Version {
    run::parse_version(
        option_env!("REQUIEM_RELEASE_VERSION")
            .filter(|v| !v.is_empty())
            .unwrap_or(env!("CARGO_PKG_VERSION")),
    )
}

pub fn run_helper_if_asked(args: &[String]) -> Option<i32> {
    let at = args.iter().position(|a| a == APPLY_FLAG)?;
    Some(match apply::helper_args_from(&args[at + 1..]) {
        Some(helper) => apply::run_helper(&helper),
        None => 64,
    })
}

pub fn on_start(args: &[String]) {
    let (Some(install), Some(root)) = (install_dir(), update_root()) else {
        return;
    };
    apply::sweep_helpers(&install);

    uninstall_entry::sync_version(&install, &current_version().to_string());
    if let Some(report) = apply::take_report(&root) {
        log::warn!(
            "update {} was rolled back: {}",
            report.version,
            report.reason
        );
        run::skip_version(requiem_db::Db::shared(), &report.version);
    }
    if args.iter().any(|a| a == POST_UPDATE_FLAG) {
        log::info!(
            "started after an update; vouching for it in {}s",
            HEALTHY_AFTER.as_secs()
        );
        thread::spawn(move || {
            thread::sleep(HEALTHY_AFTER);
            apply::mark_healthy(&root);
            apply::commit_pending(&install, &root);
        });
    } else {
        apply::commit_pending(&install, &root);
    }
}

pub fn restart_into(staged: &Staged) -> io::Result<()> {
    let install = install_dir().ok_or_else(|| Error::other("no install folder"))?;
    apply::launch_helper(&install, &staged.dir, &staged.version, &exe_name())
}

#[derive(Debug, Clone)]
pub enum UpdateEvent {
    State(State),

    Keys {
        base: String,
        result: Result<Vec<requiem_updater::KeyEntry>, String>,
    },

    Armed,
}

thread_local! {
    static STATE: Signal<State> = signal(State::Idle);
}

pub fn handle_event(event: UpdateEvent) {
    match event {
        UpdateEvent::State(state) => set_state(state),
        other => update_settings::on_event(other),
    }
}

pub fn state() -> State {
    STATE.with(Signal::get)
}

pub fn set_state(state: State) {
    STATE.with(|s| s.set(state));
}

static HANDLE: OnceLock<Arc<Handle>> = OnceLock::new();
static RUNTIME: OnceLock<runtime::Handle> = OnceLock::new();
static EVENTS: OnceLock<Arc<dyn Fn(UpdateEvent) + Send + Sync>> = OnceLock::new();

fn send(event: UpdateEvent) {
    if let Some(events) = EVENTS.get() {
        events(event);
    }
}

pub fn fetch_custom_keys(base: String) {
    let Some(runtime) = RUNTIME.get() else { return };
    runtime.spawn(async move {
        let result = custom::fetch_keys(&wreq::Client::new(), &base).await;
        send(UpdateEvent::Keys { base, result });
    });
}

pub fn arm_after(delay: Duration) {
    let Some(runtime) = RUNTIME.get() else { return };
    runtime.spawn(async move {
        time::sleep(delay).await;
        send(UpdateEvent::Armed);
    });
}

pub fn source_changed() {
    if let Some(root) = update_root() {
        staged::discard(&root);
    }
    set_state(State::Idle);
    if let Some(handle) = handle() {
        handle.check_now();
    }
}

pub fn handle() -> Option<&'static Arc<Handle>> {
    HANDLE.get()
}

pub fn start(
    runtime: &runtime::Handle,
    on_event: impl Fn(UpdateEvent) + Send + Sync + 'static,
) {
    let _ = RUNTIME.set(runtime.clone());
    let _ = EVENTS.set(Arc::new(on_event));
    let on_state = |state: State| send(UpdateEvent::State(state));

    if let Ok(spoof) = env::var("REQUIEM_SPOOF_UPDATE") {
        let state = match spoof.as_str() {
            "downloading" => Some(State::Downloading {
                version: "9.9.9".into(),
                done: 0,
                total: 1,
            }),
            "ready" => Some(State::Ready(Staged {
                version: "9.9.9".into(),
                dir: PathBuf::new(),
                notes_url: None,
                source: SPOOF_SOURCE.into(),
            })),
            _ => None,
        };
        if let Some(state) = state {
            on_state(state);
            return;
        }
    }
    if !cfg!(windows) {
        return;
    }
    let Some(root) = update_root() else { return };
    let updater = Arc::new(Updater {
        client: wreq::Client::new(),
        config: Config {
            current: current_version(),
            channel: "stable".into(),
            target: requiem_updater::current_target().into(),
        },
        root,
        exe_name: exe_name(),
        db: requiem_db::Db::shared().clone(),
    });

    let download_reported = AtomicBool::new(false);
    let handle = run::spawn(
        runtime,
        updater,
        sources::configured,
        move |state| {
            if matches!(state, State::Downloading { .. }) {
                if download_reported.swap(true, Ordering::Relaxed) {
                    return;
                }
            } else {
                download_reported.store(false, Ordering::Relaxed);
            }
            on_state(state);
        },
    );
    let _ = HANDLE.set(handle);
}

pub fn confirm_restart(staged: Staged) {
    let in_call = voice::current_call().is_some();
    let version = staged.version.clone();

    let custom = custom::get(requiem_db::Db::shared())
        .filter(|c| c.base == staged.source)
        .map(|c| custom::host_of(&c.base));
    let body = match (custom, in_call) {
        (Some(host), true) => requiem_i18n::t!(
            "updater-confirm-body-custom-voice",
            version = version,
            host = host
        ),
        (Some(host), false) => requiem_i18n::t!(
            "updater-confirm-body-custom",
            version = version,
            host = host
        ),
        (None, true) => requiem_i18n::t!("updater-confirm-body-voice", version = version),
        (None, false) => requiem_i18n::t!("updater-confirm-body", version = version),
    };
    confirm(ConfirmRequest {
        title: requiem_i18n::t!("updater-confirm-title"),
        body,
        confirm_label: requiem_i18n::t!("updater-confirm-update"),
        on_confirm: Rc::new(move || restart_now(&staged)),
        critical: false,
        content: None,
    });
}

const SPOOF_SOURCE: &str = "spoof";

fn restart_now(staged: &Staged) {
    if staged.source == SPOOF_SOURCE {
        log::info!("spoofed update: not restarting");
        return;
    }
    let db = requiem_db::Db::shared();
    if let Some(call) = voice::current_call() {
        resume::save(
            db,
            call.guild,
            call.channel,
            voice::is_self_mute(),
            voice::is_self_deaf(),
        );
    }
    match restart_into(staged) {
        Ok(()) => process::exit(0),
        Err(err) => {
            log::error!("could not start the update helper: {err}");
            let _ = resume::take(db);
            notify_failure(&requiem_i18n::t!("updater-failed"));
        }
    }
}

pub fn resume_voice() {
    let Some(resume) = resume::take(requiem_db::Db::shared()) else {
        return;
    };
    log::info!("rejoining the voice channel from before the update");
    voice::join_call(Call {
        guild: resume.guild_id,
        channel: resume.channel_id,
    });
    voice::set_mute(resume.muted);
    voice::set_deaf(resume.deafened);
}
