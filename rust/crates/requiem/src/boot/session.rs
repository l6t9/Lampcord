
use requiem::features::auth as features_auth;
use requiem::features::auth::AuthState;
use requiem::notifications as requiem_notifications;
use requiem_core::ui::context_menu;
use requiem_core::plugins;
use requiem_core::ui::navigation;
use requiem_core::video::VideoCache;
use requiem_ui::signal as requiem_ui_signal;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use requiem::shell;
use requiem_core::instances::Instance;
use requiem_core::models::id::marker::ChannelMarker;
use requiem_core::models::Id;
use requiem_core::query::QueryClient;
use requiem_core::rest::HttpClient;
use requiem_core::state::AppState;
use requiem_core::ui::{Fonts, RootContext, RootSignals};
use requiem_core::{auth, rest};
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::WidgetTree;
use requiem_ui::widget::anim::ToggleState;
use requiem_ui::widget::primitives::BoxWidget;

use crate::{background_main, ActiveSession, BackgroundReceivers, SessionTasks};
use super::Services;

/// Everything `make_start_session` needs to bring a session up.
pub(crate) struct SessionEnv {
    pub state: AppState,
    pub signals: RootSignals,
    pub fonts: Fonts,
    pub services: Services,
    pub ui_effects: Rc<RefCell<Vec<requiem_ui_signal::Effect>>>,
    pub receivers: BackgroundReceivers,
    pub runtime: Arc<Runtime>,
    pub current_session: Rc<RefCell<Option<ActiveSession>>>,
    pub compose_tx: mpsc::UnboundedSender<(Id<ChannelMarker>, String)>,
}

/// Returns the function that starts a session on an instance: it builds the REST client, mounts
/// the app UI and spawns the background tasks.
pub(crate) fn make_start_session(
    env: SessionEnv,
) -> Rc<dyn Fn(&mut WidgetTree, Instance, String)> {
    let SessionEnv {
        state,
        signals,
        fonts,
        services,
        ui_effects,
        receivers,
        runtime,
        current_session,
        compose_tx,
    } = env;
    let Services {
        images,
        queries,
        measure_cache,
        videos,
        proxy_slot,
        ..
    } = services;
    Rc::new(
        move |tree: &mut WidgetTree, instance: Instance, token: String| {
            let Some(backend) = instance.backend() else {
                log::error!("instance \"{}\" has no usable URLs", instance.name);
                return;
            };
            if current_session.borrow().is_some() {
                log::error!("a session is already running");
                return;
            }

            receivers.drain_stale();
            log::info!(
                "starting session on instance \"{}\" ({})",
                instance.name,
                backend.rest_base_url
            );

            context_menu::set_web_base_url(backend.web_base_url());
            plugins::set_cdn_base(backend.cdn_base_url.clone());
            requiem_notifications::set_backend(backend.clone());
            let auth_coordinator = auth::AuthCoordinator::new(backend.clone(), token)
                .with_token_account(instance.token_account());

            let http = match HttpClient::new(backend.clone(), auth_coordinator.clone()) {
                Ok(http) => http,
                Err(err) => {
                    log::error!("failed to build the REST client: {err}");
                    return;
                }
            };

            state
                .read_states
                .set_ack_sender(rest::endpoints::read_states::ack_sender(
                    runtime.handle().clone(),
                    http.clone(),
                ));
            plugins::set_http(runtime.handle().clone(), http.clone());
            let ctx = RootContext {
                fonts: fonts.clone(),
                backend: backend.clone(),
                images: images.clone(),
                queries: queries.clone(),
                http: http.clone(),
                compose_tx: compose_tx.clone(),
                measure_cache: measure_cache.clone(),
                videos: videos.clone(),
            };
            *tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
            *ui_effects.borrow_mut() = shell::mount_root(tree, &state, &ctx, &signals);

            let proxy = proxy_slot
                .lock()
                .expect("proxy slot poisoned")
                .clone()
                .expect("start_session before the proxy exists");
            let tasks: SessionTasks = Arc::new(Mutex::new(Vec::new()));
            let main_task = runtime.spawn(background_main(
                backend,
                auth_coordinator.clone(),
                http.clone(),
                proxy,
                receivers.clone(),
                tasks.clone(),
            ));
            tasks
                .lock()
                .expect("session task list poisoned")
                .push(main_task.abort_handle());
            *current_session.borrow_mut() = Some(ActiveSession {
                instance,
                auth: auth_coordinator,
                http,
                tasks,
            });
        },
    )
}

/// Everything `make_end_session` needs to tear a session down and show the login screen again.
pub(crate) struct LogoutEnv {
    pub current_session: Rc<RefCell<Option<ActiveSession>>>,
    pub runtime: Arc<Runtime>,
    pub state: AppState,
    pub queries: Rc<RefCell<Option<QueryClient>>>,
    pub videos: VideoCache,
    pub signals: RootSignals,
    pub auth_state: AuthState,
    pub ui_effects: Rc<RefCell<Vec<requiem_ui_signal::Effect>>>,
    pub auth_ctx: features_auth::AuthContext,
}

/// Returns the function that logs out of the running session.
pub(crate) fn make_end_session(env: LogoutEnv) -> Rc<dyn Fn(&mut WidgetTree)> {
    let LogoutEnv {
        current_session,
        runtime,
        state,
        queries,
        videos,
        signals,
        auth_state,
        ui_effects,
        auth_ctx,
    } = env;
    Rc::new(move |tree: &mut WidgetTree| {
        let Some(session) = current_session.borrow_mut().take() else {
            return;
        };
        stop_session(&runtime, &session);

        reset_ui_state(&state, &queries, &videos, &signals);

        auth_state.reset_for_logout();
        *tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        *ui_effects.borrow_mut() = features_auth::mount(tree, &auth_state, &auth_ctx);
    })
}

/// Stops the session's tasks, tells the server, and forgets the stored token.
fn stop_session(runtime: &Runtime, session: &ActiveSession) {
    log::info!("logging out of instance \"{}\"", session.instance.name);

    session.auth.mark_logged_out();
    for task in session
        .tasks
        .lock()
        .expect("session task list poisoned")
        .drain(..)
    {
        task.abort();
    }
    let http = session.http.clone();
    runtime.spawn(async move {
        match rest::endpoints::auth::logout(&http).await {
            Ok(()) => log::info!("server-side logout succeeded"),
            Err(err) => {
                log::warn!("POST /auth/logout failed (logged out locally anyway): {err}")
            }
        }
    });
    if let Err(err) = auth::TokenStore::new(session.instance.token_account()).clear() {
        log::error!("failed to remove the stored token: {err}");
    }
}

/// Clears everything the signed-in UI left behind, so the next login starts clean.
fn reset_ui_state(
    state: &AppState,
    queries: &Rc<RefCell<Option<QueryClient>>>,
    videos: &VideoCache,
    signals: &RootSignals,
) {
    state.reset();
    if let Some(queries) = queries.borrow().as_ref() {
        queries.clear();
    }
    videos.clear();
    navigation::reset();
    signals.selected_channel.set(None);
    signals.message_scroll.set(HashMap::new());
    signals.settings_modal_open.set(ToggleState::new(false));
    signals.settings_ui.set(Default::default());
    signals.emoji_picker.open.set(false);
    signals.search.clear();
    signals.search.history.set(Vec::new());
    signals.media_viewer.reset();
    signals.toasts.reset();
    signals.fullscreen_video.set(None);
    signals.profile_popout.open.set(None);
    signals.profile_popout.draft.set(String::new());
    signals.profile_popout.overlays.member_target.set(None);
    signals.profile_popout.overlays.full_profile.set(None);
    signals.profile_popout.overlays.block_confirm.set(None);
    signals.account_popout.close();

}
