#![allow(clippy::print_stdout, clippy::print_stderr)]

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use requiem::console;
use requiem::features::auth as features_auth;
use requiem::features::auth::AuthOutcome;
use requiem::features::auth::AuthState;
use requiem::features::captcha;
use requiem::features::splash;
use requiem::notifications as requiem_notifications;
use requiem::notifications as toasts;
use requiem::shift_state;
use requiem::shortcuts;
use requiem::system_idle;
use requiem::updater;
use requiem_core::notifications::Context;
use requiem_core::notifications::Outcome;
use requiem_core::notifications::self as decide;
use requiem_core::ui::context_menu::EditorCommand;
use requiem_core::assets;
use requiem_core::net_profile;
use requiem_core::plugins;
use requiem_core::presence_local;
use requiem_core::properties;
use requiem_core::typing_local;
use requiem_core::ui::avatar_color;
use requiem_core::ui::devtools;
use requiem_core::ui::emoji_skin;
use requiem_core::ui::language_settings;
use requiem_core::ui::link_warning;
use requiem_core::ui::reply;
use requiem_core::ui::requiem_emoji;
use requiem_core::ui::timestamps;
use requiem_core::ui;
use requiem_core::utils::backoff::Backoff;
use requiem_core::utils::notification_settings;
use requiem_core::utils::prefs;
use requiem_core::utils::time;
use requiem_core::voice::diag;
use requiem_core::voice::status;
use requiem_core::voice;
use requiem_ui::app::WindowAction;
use requiem_ui::app;
use requiem_ui::emoji_display;
use requiem_ui::font_system::TextFonts;
use requiem_ui::font_system;
use requiem_ui::paint;
use requiem_ui::perf;
use requiem_ui::signal as requiem_ui_signal;
use requiem_ui::theme;
use requiem_ui::timer;
use requiem_ui::timestamp;
use requiem_ui::widget::primitives;
use serde_json::value::RawValue;
use skia_safe::font_style::Weight;
use std::backtrace::Backtrace;
use std::env;
use std::fmt as std_fmt;
use std::fs::OpenOptions;
use std::fs;
use std::future;
use std::io;
use std::panic;
use std::path::PathBuf;
use std::process::Command;
use std::process;
use std::thread;
use tokio::runtime::Runtime;
use tokio::runtime;
use tokio::sync::mpsc;
use tokio::sync;
use tokio::task;
use tokio::time as tokio_time;
use tracing::field;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::layer;
use tracing_subscriber::util::SubscriberInitExt;
use winit::error;

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
#[global_allocator]
static ALLOCATOR: perf::CountingAlloc = perf::CountingAlloc;

use requiem::shell;
use boot::{AppEnv, EventHandler, LogoutEnv, Services, SessionEnv};
use requiem_core::config::BackendConfig;
use requiem_core::gateway::events::GatewayEvent;
use requiem_core::images::ImageCache;
use requiem_core::instances::{Instance, InstanceStore};
use requiem_core::models::id::marker::{ChannelMarker, GuildMarker, UserMarker};
use requiem_core::models::{Channel, Id, Message};
use requiem_core::presence::{PresenceField, PresenceUpdate};
use requiem_core::properties::SuperProperties;
use requiem_core::rest::HttpClient;
use requiem_core::rest::endpoints::favorite_gifs::FavoriteGifs;
use requiem_core::rest::endpoints::gifs::GifResult;
use requiem_core::sounds::{self};
use requiem_core::state::AppState;
use requiem_core::ui::attachments::AttachmentsState;
use requiem_core::ui::Fonts;
use requiem_core::uploads::{self, UploadFile, UploadProgress};
use requiem_core::{auth, gateway, rest};
use requiem_ui::app::App;
use requiem_ui::event::UiEvent;
use requiem_ui::signal::Signal;
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::WidgetTree;
use requiem_ui::widget::primitives::BoxWidget;
use winit::event_loop::EventLoopProxy;

mod boot;

fn main() {
    console::maybe_attach_console();

    let argv: Vec<String> = env::args().collect();
    if let Some(code) = updater::run_helper_if_asked(&argv) {
        process::exit(code);
    }

    dotenvy::dotenv().ok();

    let forced = env::var("REQUIEM_LOCALE").is_ok();
    let requested: Vec<String> = match env::var("REQUIEM_LOCALE") {
        Ok(list) => list.split(',').map(|tag| tag.trim().to_string()).collect(),
        Err(_) => sys_locale::get_locales().collect(),
    };
    requiem_emoji::init(None);
    emoji_skin::init();

    emoji_display::install(emoji_skin::restyle);
    timestamp::install(timestamps::format);
    language_settings::init(
        &requested.iter().map(String::as_str).collect::<Vec<_>>(),
        forced,
    );

    primitives::set_link_handler(link_warning::open_link);

    init_log();

    updater::on_start(&argv);

    if env::args().any(|arg| arg == splash::FLAG) {
        process::exit(splash::run(requiem::assets_config()));
    }

    if env::args().any(|arg| arg == captcha::FLAG) {
        process::exit(captcha::run());
    }

    if let Err(err) = run() {
        eprintln!("requiem: {err}");
        process::exit(1);
    }
}

#[derive(thiserror::Error, Debug)]
enum RunError {
    #[error("failed to run the UI event loop: {0}")]
    EventLoop(#[from] error::EventLoopError),
    #[error("could not load Discord's assets: {0}")]
    Assets(String),
}

enum GifQuery {
    Trending,
    Search(String),

    LoadFavorites,

    SaveFavorites(FavoriteGifs),
}

enum AppEvent {
    Gateway(GatewayEvent),

    NotificationClicked(Id<ChannelMarker>),

    GatewayDown,

    Toast {
        message: String,
        success: bool,
    },

    Update(updater::UpdateEvent),
    ChannelHistory {
        channel_id: Id<ChannelMarker>,
        messages: Vec<Message>,
    },

    Wake,

    InsertComposerText(String),

    InsertText {
        dom_id: &'static str,
        text: String,
    },

    EditorCommand(EditorCommand),

    GifTrending(rest::endpoints::gifs::Trending),

    GifFavorites(FavoriteGifs),

    GifSearchResults(Vec<GifResult>),

    Auth(AuthOutcome),

    FilesPicked {
        channel_id: Id<ChannelMarker>,
        paths: Vec<PathBuf>,
    },

    UploadProgress {
        nonce: u64,
        progress: UploadProgress,
    },

    UploadSent {
        nonce: u64,
        message: Box<Message>,
    },

    UploadFailed {
        nonce: u64,
        uploaded: Vec<(u64, String)>,
    },

    UploadCancelled {
        nonce: u64,
    },

    LogOut,
}

enum UploadCommand {
    Send {
        channel_id: Id<ChannelMarker>,
        nonce: u64,
        content: String,
        files: Vec<UploadFile>,
    },

    Cancel(u64),

    Delete(String),
}

type SharedRx<T> = Arc<sync::Mutex<mpsc::UnboundedReceiver<T>>>;

#[derive(Clone)]
struct BackgroundReceivers {
    compose_rx: SharedRx<(Id<ChannelMarker>, String)>,
    upload_rx: SharedRx<UploadCommand>,
    select_rx: SharedRx<Id<ChannelMarker>>,
    gif_query_rx: SharedRx<GifQuery>,
    status_rx: SharedRx<PresenceUpdate>,
}

impl BackgroundReceivers {
    fn drain_stale(&self) {
        fn drain<T>(rx: &SharedRx<T>) {
            if let Ok(mut rx) = rx.try_lock() {
                while rx.try_recv().is_ok() {}
            }
        }
        drain(&self.compose_rx);
        drain(&self.upload_rx);
        drain(&self.select_rx);
        drain(&self.gif_query_rx);
        drain(&self.status_rx);
    }
}

type SessionTasks = Arc<Mutex<Vec<task::AbortHandle>>>;

fn spawn_tracked(
    tasks: &SessionTasks,
    future: impl future::Future<Output = ()> + Send + 'static,
) {
    let handle = tokio::spawn(future);
    tasks
        .lock()
        .expect("session task list poisoned")
        .push(handle.abort_handle());
}

struct ActiveSession {
    instance: Instance,
    auth: auth::AuthCoordinator,
    http: HttpClient,
    tasks: SessionTasks,
}

fn stored_session(instances: &InstanceStore) -> Option<(Instance, String)> {
    let instance = instances.selected_instance();
    instance.backend()?;
    match auth::TokenStore::new(instance.token_account()).load() {
        Ok(Some(token)) => Some((instance, token)),
        Ok(None) => None,
        Err(err) => {
            log::warn!("failed to read stored token: {err}");
            None
        }
    }
}

enum SplashOutcome {
    Ready,

    Quit,

    Unavailable(String),
}

fn run_splash_helper() -> SplashOutcome {
    let exe = match env::current_exe() {
        Ok(exe) => exe,
        Err(e) => return SplashOutcome::Unavailable(e.to_string()),
    };
    match Command::new(exe)
        .arg(splash::FLAG)
        .status()
    {
        Ok(status) => match status.code() {
            Some(splash::EXIT_READY) => SplashOutcome::Ready,
            Some(splash::EXIT_QUIT) => SplashOutcome::Quit,
            other => SplashOutcome::Unavailable(format!("exit status {other:?}")),
        },
        Err(e) => SplashOutcome::Unavailable(e.to_string()),
    }
}

/// Fetches Discord's assets, showing the splash window first when they need downloading.
/// Returns `Ok(false)` when the user quit from the splash screen.
fn load_assets(runtime: &Runtime) -> Result<bool, RunError> {
    let config = requiem::assets_config();

    runtime.block_on(properties::refresh_identity());
    let http = rest::build_client().map_err(|e| RunError::Assets(e.to_string()))?;
    if runtime.block_on(assets::needs_splash(&http, &config)) {
        match run_splash_helper() {
            SplashOutcome::Ready => {}
            SplashOutcome::Quit => return Ok(false),
            SplashOutcome::Unavailable(why) => {
                log::warn!("the splash screen could not run ({why}); loading assets without it")
            }
        }
    }
    loop {
        log::info!("loading Discord's assets (release {})", config.app_version);
        match runtime.block_on(assets::init(&http, &config)) {
            Ok(()) => break,
            Err(err) => {
                let detail = requiem_i18n::t!(
                    "main-assets-error-detail",
                    error = err.to_string(),
                    release = config.app_version.as_str()
                );
                log::error!("could not load assets: {detail}");
                let retry_label = requiem_i18n::t!("main-assets-error-retry");
                let retry = rfd::MessageDialog::new()
                    .set_level(rfd::MessageLevel::Error)
                    .set_title(requiem_i18n::t!("main-assets-error-title"))
                    .set_description(detail.as_str())
                    .set_buttons(rfd::MessageButtons::OkCancelCustom(
                        retry_label.clone(),
                        requiem_i18n::t!("main-assets-error-quit"),
                    ))
                    .show();
                if !matches!(retry, rfd::MessageDialogResult::Custom(ref label) if *label == retry_label)
                {
                    return Err(RunError::Assets(err.to_string()));
                }
            }
        }
    }
    Ok(true)
}

fn run() -> Result<(), RunError> {
    let runtime = Runtime::new().expect("failed to start tokio runtime");

    if !load_assets(&runtime)? {
        return Ok(());
    }

    let instances = InstanceStore::load(InstanceStore::default_path());
    if instances.selected_instance().is_official() {
        runtime.spawn(net_profile::warm_up_dns());
    }
    let initial_session = stored_session(&instances);

    let plugin_wake: Arc<Mutex<Option<EventLoopProxy<AppEvent>>>> = Arc::new(Mutex::new(None));
    if let Some(dir) = plugins::default_dir() {
        let slot = plugin_wake.clone();
        plugins::init(dir, move || {
            if let Some(proxy) = slot.lock().expect("plugin wake slot poisoned").as_ref() {
                let _ = proxy.send_event(AppEvent::Wake);
            }
        });
    }

    let state = AppState::new();
    let fonts = Fonts {
        typeface: theme::make_typeface(Weight::MEDIUM),
        text_fonts: TextFonts::new(16 * 1024 * 1024),
    };
    font_system::install_ambient(fonts.text_fonts.clone());

    let (senders, receivers) = boot::channels();
    let services = Services::new(&runtime);
    let signals = boot::build_root_signals(&services, &senders);
    let (signals, arm_custom_status_expiry) =
        boot::wire_callbacks(signals, &state, &services, &senders);

    let ui_effects: Rc<RefCell<Vec<requiem_ui_signal::Effect>>> =
        Rc::new(RefCell::new(Vec::new()));
    let current_session: Rc<RefCell<Option<ActiveSession>>> = Rc::new(RefCell::new(None));
    let runtime = Arc::new(runtime);

    let start_session = boot::make_start_session(SessionEnv {
        state: state.clone(),
        signals: signals.clone(),
        fonts: fonts.clone(),
        services: services.clone(),
        ui_effects: ui_effects.clone(),
        receivers,
        runtime: runtime.clone(),
        current_session: current_session.clone(),
        compose_tx: senders.compose_tx.clone(),
    });

    let auth_state = AuthState::new(instances);
    let auth_ctx = features_auth::AuthContext {
        fonts: fonts.clone(),
        runtime: runtime.handle().clone(),
        deliver: {
            let proxy_slot = services.proxy_slot.clone();
            Arc::new(move |outcome| {
                if let Some(proxy) = proxy_slot.lock().expect("proxy slot poisoned").as_ref() {
                    let _ = proxy.send_event(AppEvent::Auth(outcome));
                }
            })
        },
    };

    let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
    if initial_session.is_none() {
        *ui_effects.borrow_mut() = features_auth::mount(&mut tree, &auth_state, &auth_ctx);
    }

    let end_session = boot::make_end_session(LogoutEnv {
        current_session: current_session.clone(),
        runtime: runtime.clone(),
        state: state.clone(),
        queries: services.queries.clone(),
        videos: services.videos.clone(),
        signals: signals.clone(),
        auth_state: auth_state.clone(),
        ui_effects: ui_effects.clone(),
        auth_ctx: auth_ctx.clone(),
    });

    boot::wire_search_jump(&signals);
    boot::register_popouts(&signals);
    let modal_open = boot::make_modal_open(&signals);

    let handler = EventHandler {
        state: state.clone(),
        signals: signals.clone(),
        auth_state: auth_state.clone(),
        current_session,
        start_session,
        end_session,
        images: services.images.clone(),
        arm_custom_status_expiry,
        plugin_styles_applied: Cell::new(state.plugins.styles_revision.peek()),
    };
    let shortcut_env = shortcuts::ShortcutEnv {
        state: state.clone(),
        signals: signals.clone(),
        open_home: shell::open_home(&state, &signals),
        devtools: devtools::DevtoolsContext {
            fonts: fonts.clone(),
            state: state.clone(),
        },
        modal_open: modal_open.clone(),
    };
    let app = boot::build_app(
        tree,
        handler,
        shortcut_env,
        &AppEnv {
            state: &state,
            signals: &signals,
            auth_state: &auth_state,
            modal_open: &modal_open,
            frame_waker: &services.frame_waker,
        },
    );

    connect_proxy(&app, &services, &plugin_wake, runtime.handle());

    if let Some((instance, token)) = initial_session {
        (auth_ctx.deliver)(AuthOutcome::LoggedIn { instance, token });
    }

    app.run().map_err(RunError::EventLoop)
}

/// Hands the event-loop proxy to everything that posts events to the UI thread.
fn connect_proxy(
    app: &App<AppEvent>,
    services: &Services,
    plugin_wake: &Mutex<Option<EventLoopProxy<AppEvent>>>,
    runtime: &runtime::Handle,
) {
    let proxy = app.proxy();

    *services.proxy_slot.lock().expect("proxy slot poisoned") = Some(proxy.clone());
    {
        let proxy = Mutex::new(proxy.clone());
        requiem_notifications::init(move |channel_id| {
            let _ = proxy
                .lock()
                .expect("proxy poisoned")
                .send_event(AppEvent::NotificationClicked(channel_id));
        });
    }
    {
        let proxy = Mutex::new(proxy.clone());
        updater::start(runtime, move |event| {
            let _ = proxy
                .lock()
                .expect("update proxy poisoned")
                .send_event(AppEvent::Update(event));
        });
    }
    *plugin_wake.lock().expect("plugin wake slot poisoned") = Some(proxy.clone());
    *services.emoji_insert_proxy.borrow_mut() = Some(proxy);
}

/// Debug: replays `{"t": "MESSAGE_CREATE", "d": {..}}` from `path` as a gateway
/// event each time the file changes, standing in for the web client's dispatcher.
fn spawn_gateway_spoof(path: PathBuf, proxy: EventLoopProxy<AppEvent>) {
    tokio::spawn(async move {
        let mut seen = None;
        loop {
            tokio_time::sleep(Duration::from_millis(500)).await;
            let Ok(modified) = fs::metadata(&path).and_then(|m| m.modified()) else {
                continue;
            };
            if seen.replace(modified) == Some(modified) {
                continue;
            }
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
                log::warn!("spoof gateway file is not valid JSON");
                continue;
            };
            let (Some(name), Some(data)) = (value["t"].as_str(), value.get("d")) else {
                continue;
            };
            let Ok(raw) = RawValue::from_string(data.to_string()) else {
                continue;
            };
            let _ = proxy.send_event(AppEvent::Gateway(GatewayEvent::from_raw(
                name.to_string(),
                raw,
            )));
        }
    });
}

async fn background_main(
    backend: BackendConfig,

    auth: auth::AuthCoordinator,
    http: HttpClient,
    proxy: EventLoopProxy<AppEvent>,
    receivers: BackgroundReceivers,

    tasks: SessionTasks,
) {
    let (tx, mut rx) = mpsc::unbounded_channel();

    if let Some(path) = env::var_os("REQUIEM_SPOOF_GATEWAY_FILE") {
        spawn_gateway_spoof(path.into(), proxy.clone());
    }

    let (gateway_tx, mut gateway_rx) = mpsc::unbounded_channel::<serde_json::Value>();
    gateway::guild_subscriptions::set_outgoing(gateway_tx.clone());

    voice::init(gateway_tx.clone(), http.client().clone());

    {
        let proxy = Mutex::new(proxy.clone());
        status::set_sink(move |event| {
            let _ = proxy
                .lock()
                .expect("voice status proxy poisoned")
                .send_event(AppEvent::Gateway(event));
        });
    }

    {
        let (typing_tx, mut typing_rx) = mpsc::unbounded_channel::<u64>();
        typing_local::set_poster(typing_tx);
        let http = http.clone();
        spawn_tracked(&tasks, async move {
            while let Some(channel) = typing_rx.recv().await {
                let _ = rest::endpoints::channels::send_typing(&http, Id::new(channel)).await;
            }
        });
    }

    spawn_tracked(&tasks, async {
        let mut every = tokio_time::interval(Duration::from_secs(10));
        loop {
            every.tick().await;
            match system_idle::idle_ms() {
                Some(ms) => presence_local::note_system_idle(ms),
                None => presence_local::tick(),
            }
        }
    });

    gateway::guild_subscriptions::set_members_visible(true);

    let gw_backend = backend.clone();
    let gw_auth = auth.clone();
    let gw_proxy = proxy.clone();
    let gw_client = http.client().clone();
    spawn_tracked(&tasks, async move {
        let mut session = gateway::Session::default();

        let mut reconnect_backoff =
            Backoff::with_range(1_000, Some(60_000));
        loop {
            let token_used = gw_auth.current_token();
            let result = gateway::run(
                &gw_client,
                &gw_backend,
                &token_used,
                &SuperProperties::desktop_like(),
                &mut session,
                tx.clone(),
                &mut gateway_rx,
            )
            .await;
            let _ = gw_proxy.send_event(AppEvent::GatewayDown);
            if session.take_established() {
                reconnect_backoff.succeed();
            }

            let wait = match result {
                Err(gateway::GatewayError::ReconnectRequested) => {
                    log::info!("gateway asked us to reconnect");
                    Duration::ZERO
                }
                Err(gateway::GatewayError::AuthenticationFailed) => {
                    session.clear();
                    log::warn!(
                        "gateway rejected our token (likely revoked — e.g. every session was logged out); recovering"
                    );
                    match gw_auth.recover_from(&token_used).await {
                        Ok(_fresh_token) => {
                            log::info!("re-authenticated, reconnecting");
                            Duration::from_secs(3)
                        }
                        Err(err) => {
                            log::error!("re-authentication failed ({err})");
                            Duration::from_secs(30)
                        }
                    }
                }
                other => {
                    if let Err(err) = &other {
                        log::warn!("gateway connection ended: {err}; reconnecting");
                    }

                    let wait = reconnect_backoff.fail(None);
                    if reconnect_backoff.fails() > 4 {
                        session.clear();
                    }
                    wait
                }
            };
            tokio_time::sleep(wait).await;
        }
    });

    spawn_tracked(
        &tasks,
        send_composed_messages(http.clone(), receivers.compose_rx.clone()),
    );
    spawn_tracked(
        &tasks,
        send_uploads(http.clone(), proxy.clone(), receivers.upload_rx.clone()),
    );

    let channel_guild_index: Arc<Mutex<HashMap<Id<ChannelMarker>, Id<GuildMarker>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let fetched_history: Arc<Mutex<HashSet<Id<ChannelMarker>>>> =
        Arc::new(Mutex::new(HashSet::new()));

    spawn_tracked(
        &tasks,
        handle_channel_selection(
            http.clone(),
            proxy.clone(),
            receivers.select_rx.clone(),
            gateway_tx.clone(),
            channel_guild_index.clone(),
            fetched_history.clone(),
        ),
    );
    spawn_tracked(
        &tasks,
        handle_gif_queries(http.clone(), proxy.clone(), receivers.gif_query_rx.clone()),
    );
    spawn_tracked(
        &tasks,
        handle_status_changes(
            http.clone(),
            gateway_tx.clone(),
            receivers.status_rx.clone(),
        ),
    );

    while let Some(event) = rx.recv().await {
        log::debug!("gateway event: {}", event.name);

        index_channels_from_dispatch(&event, &channel_guild_index);

        voice::note_event(&event);

        if event.name == "READY" {
            #[derive(serde::Deserialize)]
            struct ProtoOnly {
                user_settings_proto: Option<String>,
            }

            if let Some(username) = ready_username(&event) {
                log::info!("logged in as {username}");
            }

            if let Some(user_id) = ready_user_id(&event) {
                voice::set_user_id(user_id);
            }

            let proto = event
                .parse::<ProtoOnly>()
                .ok()
                .and_then(|ready| ready.user_settings_proto);
            let proto = proto.as_deref();
            let stored = proto
                .and_then(rest::endpoints::user_settings::status_from_base64)
                .unwrap_or_default();
            let custom = proto
                .and_then(rest::endpoints::user_settings::custom_status_from_base64)
                .filter(|c| !c.is_expired(time::now_ms()));

            presence_local::connection_open(stored, custom);
        }

        plugins::transform_gateway_event(&event).await;
        let _ = proxy.send_event(AppEvent::Gateway(event));
    }
}

fn notify_incoming_message(
    state: &AppState,
    selected: Option<Id<ChannelMarker>>,
    event: &GatewayEvent,
    images: Option<&ImageCache>,
) {
    let context = Context {
        window_focused: properties::app_focused(),
        selected_channel: selected,
    };
    let outcome = decide::on_message_create(state, event, &context, |message| {
        toasts::avatar_url(message)
    });
    if outcome != Outcome::Nothing
        && !context.window_focused
        && notification_settings::get().taskbar_flash
    {
        app::request_window_action(WindowAction::Attention);
    }
    match outcome {
        Outcome::Nothing => {}
        Outcome::Sound(sound) => sounds::play(sound),
        Outcome::Show {
            notification,
            sound,
        } => {
            if let Some(sound) = sound {
                sounds::play(sound);
            }
            if toasts::os_accepts() {
                let avatar = notification
                    .icon_url
                    .as_deref()
                    .and_then(|url| images?.get_or_decode_image(url))
                    .or_else(|| {
                        // No avatar of their own: the default one, in their color.
                        notification.icon_url.is_none().then(|| {
                            let discriminator = state
                                .users
                                .get(notification.author_id)
                                .map_or_else(|| "0".to_string(), |user| user.discriminator);
                            let color = ui::default_avatar_color(
                                notification.author_id,
                                &discriminator,
                            );
                            avatar_color::default_avatar_image(color)
                        })?
                    });
                match (avatar, notification.icon_url.clone(), images) {
                    (None, Some(url), Some(images)) => {
                        // First sight of this avatar: it is still downloading. Wait for it
                        // off the UI thread rather than showing the toast without one.
                        let probe = images.loaded_bytes_probe();
                        thread::spawn(move || {
                            let bytes = (0..30).find_map(|_| {
                                probe(&url).or_else(|| {
                                    thread::sleep(Duration::from_millis(100));
                                    None
                                })
                            });
                            let image = bytes.and_then(|bytes| {
                                skia_safe::Image::from_encoded(skia_safe::Data::new_copy(&bytes))
                            });
                            toasts::show(&notification, image.as_ref());
                        });
                    }
                    (avatar, _, _) => toasts::show(&notification, avatar.as_ref()),
                }
            }
        }
    }
}

#[derive(serde::Deserialize)]
struct GuildChannelsPeek {
    id: Id<GuildMarker>,
    #[serde(default)]
    channels: Vec<Channel>,
    #[serde(default)]
    unavailable: bool,
}

fn index_channels_from_dispatch(
    event: &GatewayEvent,
    index: &Arc<Mutex<HashMap<Id<ChannelMarker>, Id<GuildMarker>>>>,
) {
    let guilds: Vec<GuildChannelsPeek> = match event.name.as_str() {
        "READY" => {
            #[derive(serde::Deserialize)]
            struct ReadyPeek {
                #[serde(default)]
                guilds: Vec<Box<RawValue>>,
            }
            let Ok(ready) = event.parse::<ReadyPeek>() else {
                return;
            };
            ready
                .guilds
                .iter()
                .filter_map(|guild| serde_json::from_str(guild.get()).ok())
                .collect()
        }
        "GUILD_CREATE" => event.parse().ok().into_iter().collect(),
        _ => return,
    };
    for guild in guilds {
        if guild.unavailable {
            continue;
        }
        {
            let mut index = index.lock().expect("channel/guild index poisoned");
            for channel in &guild.channels {
                index.insert(channel.id, guild.id);
            }
        }
    }
}

fn ready_username(event: &GatewayEvent) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct UserPeek {
        username: String,
    }
    #[derive(serde::Deserialize)]
    struct ReadyPeek {
        user: UserPeek,
    }
    event
        .parse::<ReadyPeek>()
        .ok()
        .map(|ready| ready.user.username)
}

fn ready_user_id(event: &GatewayEvent) -> Option<u64> {
    #[derive(serde::Deserialize)]
    struct UserPeek {
        id: Id<UserMarker>,
    }
    #[derive(serde::Deserialize)]
    struct ReadyPeek {
        user: UserPeek,
    }
    event.parse::<ReadyPeek>().ok().map(|ready| ready.user.id.get())
}

async fn handle_status_changes(
    http: HttpClient,
    _gateway_tx: mpsc::UnboundedSender<serde_json::Value>,
    status_rx: SharedRx<PresenceUpdate>,
) {
    let mut status_rx = status_rx.lock().await;
    while let Some(update) = status_rx.recv().await {
        presence_local::set(update.status, update.custom.clone());
        let saved = match update.changed {
            PresenceField::Status => {
                rest::endpoints::user_settings::set_status(&http, update.status).await
            }
            PresenceField::CustomStatus => {
                rest::endpoints::user_settings::set_custom_status(&http, update.custom.as_ref())
                    .await
            }
        };
        if let Err(err) = saved {
            log::warn!("failed to save {:?}: {err}", update.changed);
        }
    }
}

fn os_shift_down() -> bool {
    shift_state::shift_down(app::shift_held())
}

fn restore_composer_text(
    tree: &mut WidgetTree,
    state: &AppState,
    selected: &Signal<Option<Id<ChannelMarker>>>,
    channel_id: Id<ChannelMarker>,
    text: String,
) {
    if text.is_empty() {
        return;
    }
    if ui::displayed_channel(state, selected.peek()).map(|c| c.id) != Some(channel_id)
    {
        if state.drafts.text(channel_id).is_empty() {
            state.drafts.set(channel_id, &text);
        }
        return;
    }
    paint::focus_by_dom_id(tree, "composer-input");
    paint::dispatch_key_event(tree, &UiEvent::TextInput { text });
}

fn poll_shift_while_dragging(attachments: AttachmentsState) {
    timer::set_timeout(Duration::from_millis(50), move || {
        if !attachments.drag_hover.peek() {
            return;
        }
        let shift = os_shift_down();
        if attachments.shift_held.peek() != shift {
            attachments.shift_held.set(shift);
        }
        poll_shift_while_dragging(attachments);
    });
}

async fn send_composed_messages(
    http: HttpClient,
    compose_rx: SharedRx<(Id<ChannelMarker>, String)>,
) {
    let mut compose_rx = compose_rx.lock().await;
    while let Some((channel_id, content)) = compose_rx.recv().await {
        let reply = reply::take_outgoing(channel_id);
        match rest::endpoints::messages::send_reply(&http, channel_id, &content, reply.as_ref())
            .await
        {
            Err(err @ rest::RestError::CancelledByPlugin { .. }) => log::info!("{err}"),
            Err(err) => log::warn!("failed to send message to channel {channel_id}: {err}"),
            Ok(_) => {}
        }
    }
}

async fn send_uploads(
    http: HttpClient,
    proxy: EventLoopProxy<AppEvent>,
    upload_rx: SharedRx<UploadCommand>,
) {
    let mut upload_rx = upload_rx.lock().await;
    let running: Arc<Mutex<HashMap<u64, task::AbortHandle>>> = Arc::default();
    while let Some(command) = upload_rx.recv().await {
        match command {
            UploadCommand::Send {
                channel_id,
                nonce,
                content,
                files,
            } => {
                let (http, proxy, done) = (http.clone(), proxy.clone(), running.clone());
                let task = tokio::spawn(async move {
                    let last_sent = Arc::new(Mutex::new(0u64));
                    let on_progress: Arc<dyn Fn(UploadProgress) + Send + Sync> = {
                        let proxy = proxy.clone();
                        Arc::new(move |progress: UploadProgress| {
                            let step = (progress.total / 100).max(1);
                            let mut last = last_sent.lock().expect("progress poisoned");
                            if progress.sent.saturating_sub(*last) < step
                                && progress.sent != progress.total
                                && *last != 0
                            {
                                return;
                            }
                            *last = progress.sent.max(1);
                            let _ = proxy.send_event(AppEvent::UploadProgress { nonce, progress });
                        })
                    };
                    let event = match uploads::upload_and_send(
                        &http,
                        channel_id,
                        &content,
                        &nonce.to_string(),
                        &files,
                        on_progress,
                    )
                    .await
                    {
                        Ok(message) => AppEvent::UploadSent {
                            nonce,
                            message: Box::new(message),
                        },
                        Err(err) => {
                            log::warn!(
                                "failed to send message with attachments to channel {channel_id}: {}",
                                err.reason
                            );
                            AppEvent::UploadFailed {
                                nonce,
                                uploaded: err.uploaded,
                            }
                        }
                    };
                    done.lock().expect("upload tasks poisoned").remove(&nonce);
                    let _ = proxy.send_event(event);
                });
                running
                    .lock()
                    .expect("upload tasks poisoned")
                    .insert(nonce, task.abort_handle());
            }
            UploadCommand::Cancel(nonce) => {
                if let Some(task) = running
                    .lock()
                    .expect("upload tasks poisoned")
                    .remove(&nonce)
                {
                    task.abort();
                }
                let _ = proxy.send_event(AppEvent::UploadCancelled { nonce });
            }
            UploadCommand::Delete(name) => {
                let http = http.clone();
                tokio::spawn(async move {
                    if let Err(err) =
                        rest::endpoints::attachments::delete_upload(&http, &name).await
                    {
                        log::warn!("failed to delete upload {name}: {err}");
                    }
                });
            }
        }
    }
}

async fn handle_channel_selection(
    http: HttpClient,
    proxy: EventLoopProxy<AppEvent>,
    select_rx: SharedRx<Id<ChannelMarker>>,
    gateway_tx: mpsc::UnboundedSender<serde_json::Value>,
    channel_guild_index: Arc<Mutex<HashMap<Id<ChannelMarker>, Id<GuildMarker>>>>,
    fetched_history: Arc<Mutex<HashSet<Id<ChannelMarker>>>>,
) {
    let mut select_rx = select_rx.lock().await;
    while let Some(channel_id) = select_rx.recv().await {
        let guild_id = channel_guild_index
            .lock()
            .expect("channel/guild index poisoned")
            .get(&channel_id)
            .copied();

        presence_local::note_activity();

        if let Some(guild_id) = guild_id {
            subscribe_to_member_list(&gateway_tx, guild_id, channel_id);
        }

        let already_have = fetched_history
            .lock()
            .expect("fetched-history set poisoned")
            .contains(&channel_id);
        if already_have {
            continue;
        }

        http.retry_denied(&format!("/channels/{channel_id}/"));

        match rest::endpoints::messages::get_channel_messages(
            &http,
            channel_id,
            rest::endpoints::messages::INITIAL_PAGE_SIZE,
        )
        .await
        {
            Ok(history) => {
                log::info!(
                    "fetched {} message(s) for channel {channel_id}",
                    history.len()
                );
                fetched_history
                    .lock()
                    .expect("fetched-history set poisoned")
                    .insert(channel_id);
                let _ = proxy.send_event(AppEvent::ChannelHistory {
                    channel_id,
                    messages: history,
                });
            }
            Err(err) => {
                log::warn!("failed to fetch channel history for channel {channel_id}: {err}")
            }
        }
    }
}

async fn handle_gif_queries(
    http: HttpClient,
    proxy: EventLoopProxy<AppEvent>,
    gif_query_rx: SharedRx<GifQuery>,
) {
    let mut gif_query_rx = gif_query_rx.lock().await;
    while let Some(query) = gif_query_rx.recv().await {
        match query {
            GifQuery::Trending => match rest::endpoints::gifs::get_trending(&http).await {
                Ok(trending) => {
                    let _ = proxy.send_event(AppEvent::GifTrending(trending));
                }
                Err(err) => log::warn!("failed to fetch trending GIFs: {err}"),
            },
            GifQuery::LoadFavorites => {
                match rest::endpoints::favorite_gifs::get_favorite_gifs(&http).await {
                    Ok(favorites) => {
                        let _ = proxy.send_event(AppEvent::GifFavorites(favorites));
                    }
                    Err(err) => log::warn!("failed to load favorite GIFs: {err}"),
                }
            }
            GifQuery::SaveFavorites(favorites) => {
                if let Err(err) =
                    rest::endpoints::favorite_gifs::set_favorite_gifs(&http, &favorites).await
                {
                    log::warn!("failed to save favorite GIFs: {err}");
                }
            }
            GifQuery::Search(query) => {
                if query.trim().is_empty() {
                    continue;
                }
                match rest::endpoints::gifs::search_gifs(&http, &query).await {
                    Ok(results) => {
                        let _ = proxy.send_event(AppEvent::GifSearchResults(results));
                    }
                    Err(err) => log::warn!("failed to search GIFs for {query:?}: {err}"),
                }
            }
        }
    }
}

fn subscribe_to_member_list(
    _gateway_tx: &mpsc::UnboundedSender<serde_json::Value>,
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
) {
    gateway::guild_subscriptions::select_channel(guild_id.get(), channel_id.get());
}

fn init_log() {
    let _ = tracing_log::LogTracer::init();
    let mut filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    for directive in [
        "requiem_core::gateway=debug",
        "requiem_core::state=debug",
        "requiem_core::voice=debug",
        "requiem_core::voice::dave=trace",
        "requiem_ui::rebuild=debug",
    ] {
        if let Ok(directive) = directive.parse() {
            filter = filter.add_directive(directive);
        }
    }
    let (path, file) = app_log_file();
    let fmt = fmt::layer()
        .with_writer(Mutex::new(LogTee { file }))
        .with_ansi(false)
        .boxed();
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt)
        .with(VoiceLog)
        .init();
    log::info!(
        "logging to {}",
        path.unwrap_or_else(|| "<stderr>".into()).display()
    );

    // A panic would otherwise reach only stderr, which a windowed build doesn't show, and
    // the log would just end.
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        log::error!(
            "panic: {info}
{}",
            Backtrace::force_capture()
        );
        previous(info);
    }));
}

struct LogTee {
    file: Option<fs::File>,
}

impl io::Write for LogTee {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let _ = io::stderr().write_all(buf);
        if let Some(file) = &mut self.file {
            let _ = file.write_all(buf);
            let _ = file.flush();
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let _ = io::stderr().flush();
        if let Some(file) = &mut self.file {
            file.flush()?;
        }
        Ok(())
    }
}

fn app_log_file() -> (Option<PathBuf>, Option<fs::File>) {
    let path = env::var_os("REQUIEM_LOG_FILE")
        .map(PathBuf::from)
        .or_else(|| {
            prefs::config_dir().map(|dir| dir.join("logs").join("requiem.log"))
        });
    let Some(path) = path else {
        return (None, None);
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok();
    (Some(path), file)
}

struct VoiceLog;

impl<S> Layer<S> for VoiceLog
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: layer::Context<'_, S>,
    ) {
        let target = event.metadata().target();
        if !target.starts_with("requiem_core::voice") {
            return;
        }
        let mut message = String::new();
        event.record(&mut VoiceMessage(&mut message));
        diag::write_log(
            log_level(event.metadata().level()),
            target,
            &format_args!("{message}"),
        );
    }
}

struct VoiceMessage<'a>(&'a mut String);

impl field::Visit for VoiceMessage<'_> {
    fn record_debug(&mut self, field: &field::Field, value: &dyn std_fmt::Debug) {
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        if field.name() == "message" {
            let _ = std_fmt::write(self.0, format_args!("{value:?}"));
        } else {
            let _ = write!(self.0, "{}={value:?}", field.name());
        }
    }
}

fn log_level(level: &tracing::Level) -> log::Level {
    match *level {
        tracing::Level::ERROR => log::Level::Error,
        tracing::Level::WARN => log::Level::Warn,
        tracing::Level::INFO => log::Level::Info,
        tracing::Level::DEBUG => log::Level::Debug,
        tracing::Level::TRACE => log::Level::Trace,
    }
}
