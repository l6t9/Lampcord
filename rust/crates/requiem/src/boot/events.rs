
use requiem::features::auth as features_auth;
use requiem::features::auth::AuthOutcome;
use requiem::features::auth::AuthState;
use requiem::notifications as requiem_notifications;
use requiem::updater;
use requiem_core::ui::context_menu::EditorCommand;
use requiem_ui::event::NamedKey;
use requiem_core::plugins;
use requiem_core::ui::create_channel;
use requiem_core::ui::create_guild;
use requiem_core::ui::home;
use requiem_ui::app::WindowAction;
use requiem_ui::app;
use requiem_ui::paint;
use requiem_ui::style::cascade;
use std::path::PathBuf;

use requiem_ui::components::toast;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};
use requiem_ui::signal::Signal;
use requiem_ui::timer;
use requiem::shell;
use requiem_core::gateway::events::GatewayEvent;
use requiem_core::images::ImageCache;
use requiem_core::instances::Instance;
use requiem_core::models::id::marker::ChannelMarker;
use requiem_core::models::Id;
use requiem_core::state::AppState;
use requiem_core::state::NavEvent;
use requiem_core::ui::RootSignals;
use requiem_core::auth;
use requiem_ui::event::UiEvent;
use requiem_ui::widget::WidgetTree;

use crate::{notify_incoming_message, restore_composer_text, ActiveSession, AppEvent};

const TICK_WINDOW: Duration = Duration::from_millis(60);

thread_local! {
    static LAST_TICK: Cell<Option<Instant>> = const { Cell::new(None) };
    static TICK_TRAILING: Cell<bool> = const { Cell::new(false) };
}

fn bump_tick(tick: &Signal<u64>) {
    let now = Instant::now();
    let since = LAST_TICK.with(|last| last.get()).map(|at| now.duration_since(at));
    match since {
        Some(elapsed) if elapsed < TICK_WINDOW => {
            if !TICK_TRAILING.with(|pending| pending.replace(true)) {
                let tick = tick.clone();
                timer::set_timeout(TICK_WINDOW - elapsed, move || {
                    TICK_TRAILING.with(|pending| pending.set(false));
                    LAST_TICK.with(|last| last.set(Some(Instant::now())));
                    tick.update(|n| *n += 1);
                });
            }
        }
        _ => {
            LAST_TICK.with(|last| last.set(Some(now)));
            tick.update(|n| *n += 1);
        }
    }
}

/// Handles the events other threads post to the UI thread through the event-loop proxy.
pub(crate) struct EventHandler {
    pub state: AppState,
    pub signals: RootSignals,
    pub auth_state: AuthState,
    pub current_session: Rc<RefCell<Option<ActiveSession>>>,
    pub start_session: Rc<dyn Fn(&mut WidgetTree, Instance, String)>,
    pub end_session: Rc<dyn Fn(&mut WidgetTree)>,
    pub images: Rc<RefCell<Option<ImageCache>>>,
    /// Re-arms the timer that clears an expiring custom status.
    pub arm_custom_status_expiry: Rc<dyn Fn()>,
    pub plugin_styles_applied: Cell<u64>,
}

impl EventHandler {
    fn session_is_active(&self) -> bool {
        self.current_session.borrow().is_some()
    }

    pub(crate) fn handle(&self, event: AppEvent, tree: &mut WidgetTree) {
        match event {
            AppEvent::Gateway(_)
            | AppEvent::GatewayDown
            | AppEvent::ChannelHistory { .. }
            | AppEvent::GifTrending(_)
            | AppEvent::GifFavorites(_)
            | AppEvent::GifSearchResults(_)
                if !self.session_is_active() => {}
            AppEvent::LogOut => (self.end_session)(tree),
            AppEvent::Auth(outcome) => self.on_auth(outcome, tree),
            AppEvent::Gateway(event) => self.on_gateway(event),
            AppEvent::GatewayDown => self.state.connection.ready.set(false),
            AppEvent::ChannelHistory {
                channel_id,
                messages,
            } => self.state.set_initial_history(channel_id, messages),
            AppEvent::Wake => self.on_wake(tree),
            AppEvent::InsertComposerText(text) => {
                paint::focus_by_dom_id(tree, "composer-input");
                paint::dispatch_key_event(tree, &UiEvent::TextInput { text });
            }
            AppEvent::InsertText { dom_id, text } => {
                paint::focus_by_dom_id(tree, dom_id);
                paint::dispatch_key_event(tree, &UiEvent::TextInput { text });
            }
            AppEvent::EditorCommand(command) => run_editor_command(tree, command),
            AppEvent::NotificationClicked(channel_id) => {
                app::request_window_action(WindowAction::Focus);
                (self.signals.on_select_channel)(channel_id);
            }
            AppEvent::Update(event) => updater::handle_event(event),
            AppEvent::Toast { message, success } => {
                let kind = if success {
                    toast::ToastKind::Success
                } else {
                    toast::ToastKind::Failure
                };
                self.signals.toasts.show(&message, kind);
            }
            AppEvent::FilesPicked { channel_id, paths } => {
                self.on_files_picked(channel_id, paths, tree)
            }
            AppEvent::UploadProgress { nonce, progress } => {
                self.signals.attachments.progress(nonce, progress)
            }
            AppEvent::UploadSent { nonce, message } => {
                self.state.insert_message(*message);
                self.signals.attachments.sent(nonce);
            }
            AppEvent::UploadFailed { nonce, uploaded } => {
                self.on_upload_failed(nonce, &uploaded, tree)
            }
            AppEvent::UploadCancelled { nonce } => {
                if let Some((channel_id, text)) = self.signals.attachments.cancelled(nonce) {
                    self.restore_composer(tree, channel_id, text);
                }
            }
            AppEvent::GifTrending(trending) => {
                let gif = &self.signals.emoji_picker.gif.signals;
                gif.categories.set(trending.categories);
                gif.trending.set(trending.gifs);
            }
            AppEvent::GifFavorites(favorites) => {
                self.signals.emoji_picker.gif.signals.favorites.set(favorites)
            }
            AppEvent::GifSearchResults(results) => {
                self.signals.emoji_picker.gif.signals.results.set(results)
            }
        }
    }

    fn restore_composer(&self, tree: &mut WidgetTree, channel_id: Id<ChannelMarker>, text: String) {
        restore_composer_text(
            tree,
            &self.state,
            &self.signals.selected_channel,
            channel_id,
            text,
        );
    }

    /// A login finished: remember its token and start the session.
    fn on_auth(&self, outcome: AuthOutcome, tree: &mut WidgetTree) {
        if let Some((instance, token)) = features_auth::apply_outcome(&self.auth_state, outcome) {
            if let Err(err) = auth::TokenStore::new(instance.token_account()).save(&token) {
                log::error!("logged in, but failed to store the token: {err}");
            }
            (self.start_session)(tree, instance, token);
        }
    }

    fn on_gateway(&self, event: GatewayEvent) {
        let app_state = &self.state;
        let nav_select = &*self.signals.on_select_channel;
        let selected_channel_for_sounds = &self.signals.selected_channel;
        let images_for_notifications = &self.images;
        let attachments = &self.signals.attachments;
        app_state.apply_event(&event);

        if event.name == "READY" {
            updater::resume_voice();
        }

        if event.name == "GUILD_CREATE" {
            create_guild::select_pending(app_state, nav_select);

        }

        if event.name == "MESSAGE_ACK" {
            #[derive(serde::Deserialize)]
            struct Ack {
                channel_id: Id<ChannelMarker>,
            }
            if let Ok(ack) = event.parse::<Ack>() {
                requiem_notifications::clear_channel(ack.channel_id);
            }
        }
        if event.name == "MESSAGE_CREATE" {
            #[derive(serde::Deserialize)]
            struct NonceOnly {
                nonce: Option<serde_json::Value>,
            }

            notify_incoming_message(app_state, selected_channel_for_sounds.peek(), &event, images_for_notifications.borrow().as_ref());
            let nonce = event.parse::<NonceOnly>().ok().and_then(|m| m.nonce).and_then(|n| n.as_str().map(str::to_string).or_else(|| n.as_u64().map(|n| n.to_string())));
            if let Some(nonce) = nonce {
                attachments.message_created(&nonce);
            }
        }
        if event.name == "READY" {
            (self.arm_custom_status_expiry)();
        }
    }

    /// The background tasks poked the UI thread: advance the tick and drain what they queued.
    fn on_wake(&self, tree: &mut WidgetTree) {
        let app_state = &self.state;
        let nav_select = &*self.signals.on_select_channel;
        let global_tick = &self.signals.global_tick;
        let selected_channel_for_restore = &self.signals.selected_channel;
        let plugin_styles_applied = &self.plugin_styles_applied;
        bump_tick(global_tick);

        for event in app_state.nav.take_inbox() {
            match event {
                NavEvent::FriendRequestSent { username, error } => {
                    let (message, ok) = home::friend_request_message(&username, error.as_deref());
                    app_state.nav.add_friend_result.set(Some((message, ok)));
                }
                NavEvent::GuildFlow { guild_id, error, .. } => {
                    create_guild::finish(app_state, guild_id, error);
                    create_guild::select_pending(app_state, nav_select);
                }
                NavEvent::DmOpened { channel_id, recipients } => {

                    let kind = if recipients.len() > 1 { 3 } else { 1 };
                    let data = serde_json::json!({ "id": channel_id.to_string(), "type": kind, "recipient_ids": recipients.iter().map(ToString::to_string).collect::<Vec<_>>() });
                    app_state.apply_event(&GatewayEvent::new("CHANNEL_CREATE", data));
                    nav_select(channel_id);
                }
                NavEvent::OpenChannel(channel_id) => nav_select(channel_id),
                NavEvent::ChannelCreated { channel_id, error } => {
                    create_channel::finish(error);
                    if let Some(id) = channel_id {
                        nav_select(id);
                    }
                }
            }
        }

        for restore in app_state.plugins.apply_events(plugins::take_events()) {
            restore_composer_text(tree, app_state, selected_channel_for_restore, restore.channel_id, restore.content);
        }
        let styles = app_state.plugins.styles_revision.peek();
        if plugin_styles_applied.replace(styles) != styles {
            cascade::apply_stylesheet(tree, &shell::load_theme());
        }
    }

    fn on_files_picked(&self, channel_id: Id<ChannelMarker>, paths: Vec<PathBuf>, tree: &mut WidgetTree) {
        let attachments = &self.signals.attachments;
        let result = attachments.add_paths(channel_id, paths);
        if !result.unreadable.is_empty() || !result.empty.is_empty() {
            log::warn!("attachments: unreadable: {:?}, empty: {:?}", result.unreadable, result.empty);
        }
        paint::focus_by_dom_id(tree, "composer-input");
    }

    fn on_upload_failed(&self, nonce: u64, uploaded: &[(u64, String)], tree: &mut WidgetTree) {
        let attachments = &self.signals.attachments;
        let app_state = &self.state;
        let selected_channel_for_restore = &self.signals.selected_channel;
        let channel = attachments.uploads.with_untracked(|list| list.iter().find(|u| u.nonce == nonce).map(|u| u.channel));
        let after = channel.and_then(|channel| app_state.messages.by_channel.with_untracked(|by| by.get(&channel).and_then(|list| list.last().map(|m| m.id))));
        if let Some((channel_id, text)) = attachments.failed(nonce, uploaded, after) {
            restore_composer_text(tree, app_state, selected_channel_for_restore, channel_id, text);
        }
    }
}

fn run_editor_command(tree: &mut WidgetTree, command: EditorCommand) {
    paint::focus_by_dom_id(tree, "composer-input");
    let key = match command {
        EditorCommand::Cut => NamedKey::Cut,
        EditorCommand::Copy => NamedKey::Copy,
        EditorCommand::Paste => NamedKey::Paste,
        EditorCommand::SelectAll => NamedKey::SelectAll,
        EditorCommand::Undo => NamedKey::Undo,
        EditorCommand::Redo => NamedKey::Redo,
        EditorCommand::Focus => return,
    };
    paint::dispatch_key_event(tree, &UiEvent::KeyDown { key, shift: false });
}
