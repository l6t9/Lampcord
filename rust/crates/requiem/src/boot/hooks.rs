
use requiem::features::auth as features_auth;
use requiem::features::auth::AuthState;
use requiem::features::guild_settings as features_guild_settings;
use requiem::notifications as requiem_notifications;
use requiem_core::ui::inbox;
use requiem_core::ui::message_keys as message_keys;
use requiem_core::ui::pins;
use requiem_core::ui::reactions;
use requiem_core::ui::popouts;
use requiem_core::ui::context_menu;
use requiem_ui::event::NamedKey;
use requiem_core::presence_local;
use requiem_core::properties;
use requiem_core::ui::attachments;
use requiem_core::ui::commands_panel;
use requiem_core::ui::create_channel;
use requiem_core::ui::create_guild;
use requiem_core::ui::custom_status;
use requiem_core::ui::emoji_style_menu;
use requiem_core::ui::forward;
use requiem_core::ui::guild_settings;
use requiem_core::ui::invite_modal;
use requiem_core::ui::media_viewer::MediaViewerState;
use requiem_core::ui::message_edit;
use requiem_core::ui::message_keys as ui_message_keys;
use requiem_core::ui::message_list;
use requiem_core::ui::new_message;
use requiem_core::ui::plugin_install;
use requiem_core::ui::profile_popout::ProfileOverlays;
use requiem_core::ui::profile_popout;
use requiem_core::ui::quick_switcher;
use requiem_core::ui::reply;
use requiem_core::ui::search;
use requiem_core::ui::search_state::SearchState;
use requiem_core::ui::shortcuts_dialog;
use requiem_core::ui::video_player;
use requiem_core::ui;
use requiem_ui::app::FileDrop;
use requiem_ui::components;
use requiem_ui::event::NamedKey as Key;
use requiem_ui::paint;
use requiem_ui::timer;
use std::path::PathBuf;
use std::time::Instant;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;
use requiem::shell;
use requiem_core::models::id::marker::{ChannelMarker, MessageMarker};
use requiem_core::models::Id;
use requiem_core::state::AppState;
use requiem_core::ui::attachments::AttachmentsState;
use requiem_core::ui::{MessageListScroll, RootSignals, SettingsUiState};
use requiem_ui::event::UiEvent;
use requiem_ui::signal::Signal;
use requiem_ui::widget::WidgetTree;
use requiem_ui::widget::anim::ToggleState;

use crate::{os_shift_down, poll_shift_while_dragging};
use requiem_ui::widget::WidgetId;
use requiem::shortcuts;
use requiem_ui::app::App;
use requiem_ui::waker::FrameWaker;
use crate::AppEvent;
use super::EventHandler;

/// Closes the popouts a click outside of them should dismiss, and handles the attach button's
/// double click.
pub(crate) fn mouse_down_hook(
    signals: &RootSignals,
    auth_state: &AuthState,
) -> impl FnMut(&mut WidgetTree, Option<WidgetId>) + 'static {
    let search = signals.search.clone();
    let attachments = signals.attachments.clone();
    let attach_selected_channel = signals.selected_channel.clone();
    let last_attach_press: Cell<Option<Instant>> = Cell::new(None);
    let emoji_picker_open = signals.emoji_picker.open.clone();
    let profile_popout_open = signals.profile_popout.open.clone();
    let profile_popout_emoji_open = signals.profile_popout.emoji_open.clone();
    let auth_state = auth_state.clone();
    let click_settings_ui = signals.settings_ui.clone();
    move |tree, handled_id| {
        features_auth::handle_mouse_down(&auth_state, tree, handled_id);
        dismiss_search_popouts(tree, handled_id, &search);
        dismiss_color_popouts(tree, handled_id, &click_settings_ui);
        handle_attach_press(
            tree,
            handled_id,
            &attachments,
            &attach_selected_channel,
            &last_attach_press,
        );
        dismiss_floating_panels(
            tree,
            handled_id,
            &emoji_picker_open,
            &profile_popout_open,
            &profile_popout_emoji_open,
        );
        popouts::reconcile();
    }
}

fn dismiss_search_popouts(tree: &WidgetTree, handled_id: Option<WidgetId>, search: &SearchState) {
    {
        let inside = handled_id.is_some_and(|id| paint::is_descendant_or_self(tree, id, search::BAR_DOM_ID));
        if inside != search.popout_open.peek() {
            search.popout_open.set(inside);
        }
    }
    if search.sort_menu_open.peek() {
        let inside = handled_id.is_some_and(|id| {
            paint::is_descendant_or_self(tree, id, search::SORT_MENU_DOM_ID) || paint::is_descendant_or_self(tree, id, search::SORT_BUTTON_DOM_ID)
        });
        if !inside {
            search.sort_menu_open.set(false);
        }
    }
}

fn dismiss_color_popouts(
    tree: &WidgetTree,
    handled_id: Option<WidgetId>,
    click_settings_ui: &Signal<SettingsUiState>,
) {
    let color_popout = click_settings_ui.peek().color_popout;
    if color_popout.peek().is_some() {

        let inside = handled_id.is_some_and(|id| paint::is_descendant_or_self(tree, id, "color-popout"));
        if !inside {
            color_popout.set(None);
        }
    }

    {
        let inside = handled_id.is_some_and(|id| paint::is_descendant_or_self(tree, id, "role-color-popout"));
        if !inside {
            features_guild_settings::close_color_popout();
        }
    }
}

fn handle_attach_press(
    tree: &WidgetTree,
    handled_id: Option<WidgetId>,
    attachments: &AttachmentsState,
    attach_selected_channel: &Signal<Option<Id<ChannelMarker>>>,
    last_attach_press: &Cell<Option<Instant>>,
) {
    // The menu's backdrop takes the second click of a double click, so spot
    // it here by position: a double click on the + button goes straight to upload.
    let (press_x, press_y) = paint::last_press();
    let on_attach_button = paint::rect_of_dom_id(tree, attachments::MENU_BUTTON_DOM_ID)
        .is_some_and(|rect| rect.contains(press_x, press_y));
    if on_attach_button {
        let now = Instant::now();
        let doubled = last_attach_press.get().is_some_and(|t| now.duration_since(t) < Duration::from_millis(400));
        last_attach_press.set((!doubled).then_some(now));
        if doubled {
            attachments.menu_open.set(false);
            if let Some(channel) = attach_selected_channel.peek() {
                (attachments.backend.pick_files)(channel);
            }
        }
    } else {
        last_attach_press.set(None);
    }
    if attachments.menu_open.peek() {
        let inside_menu = handled_id.is_some_and(|id| {
            paint::is_descendant_or_self(tree, id, attachments::MENU_DOM_ID) || paint::is_descendant_or_self(tree, id, attachments::MENU_BUTTON_DOM_ID)
        });
        if !inside_menu {
            attachments.menu_open.set(false);
        }
    }
}

fn dismiss_floating_panels(
    tree: &WidgetTree,
    handled_id: Option<WidgetId>,
    emoji_picker_open: &Signal<bool>,
    profile_popout_open: &Signal<Option<Id<MessageMarker>>>,
    profile_popout_emoji_open: &Signal<bool>,
) {
    if reactions::picker_open() {
        let inside_picker = handled_id.is_some_and(|id| paint::is_descendant_or_self(tree, id, reactions::PICKER_DOM_ID));
        if !inside_picker {
            reactions::close_picker();
        }
    }

    if reactions::list_open() {
        let inside_list = handled_id.is_some_and(|id| paint::is_descendant_or_self(tree, id, reactions::REACTOR_LIST_DOM_ID));
        if !inside_list {
            reactions::close_list();
        }
    }

    if emoji_style_menu::is_open() {
        let inside_menu = handled_id.is_some_and(|id| {
            paint::is_descendant_or_self(tree, id, emoji_style_menu::PANEL_DOM_ID) || paint::is_descendant_or_self(tree, id, emoji_style_menu::BUTTON_DOM_ID)
        });
        if !inside_menu {
            emoji_style_menu::close();
        }
    }
    if emoji_picker_open.peek() {
        let inside_picker = handled_id.is_some_and(|id| paint::is_descendant_or_self(tree, id, "emoji-picker-panel"));
        if !inside_picker {
            emoji_picker_open.set(false);
        }
    }

    if emoji_picker_open.peek() {
        commands_panel::dismiss();
    }

    if profile_popout_open.peek().is_some() {
        let inside_popout = handled_id.is_some_and(|id| paint::is_descendant_or_self(tree, id, profile_popout::PANEL_DOM_ID));
        if !inside_popout {
            profile_popout_open.set(None);
        }
    }

    if profile_popout_emoji_open.peek() {
        let inside_emoji_picker = handled_id.is_some_and(|id| paint::is_descendant_or_self(tree, id, profile_popout::EMOJI_PANEL_DOM_ID));
        if !inside_emoji_picker {
            profile_popout_emoji_open.set(false);
        }
    }
}

/// Typing with nothing focused goes to the message under the cursor, or else to the composer.
pub(crate) fn text_fallback_hook(
    modal_open: &Rc<dyn Fn() -> bool>,
    state: &AppState,
    selected_channel: &Signal<Option<Id<ChannelMarker>>>,
) -> impl FnMut(&mut WidgetTree, &UiEvent) -> bool + 'static {
    let modal_open = modal_open.clone();
    let (text_state, text_selected) = (state.clone(), selected_channel.clone());
    move |tree, event| {

    if let (UiEvent::TextInput { text }, Some(channel)) = (event, ui::displayed_channel(&text_state, text_selected.peek())) {
        if !modal_open() && ui_message_keys::text_key(tree, &text_state, &channel, text) {
            return true;
        }
    }
    if modal_open() || !paint::focus_dom_id(tree, "composer-input") {
        return false;
    }
    paint::dispatch_key_event(tree, event);
    true
    }
}

pub(crate) fn focus_hook(
    state: &AppState,
    selected_channel: &Signal<Option<Id<ChannelMarker>>>,
) -> impl FnMut(bool) + 'static {
    let state = state.clone();
    let selected_channel = selected_channel.clone();
    move |focused| {

        if let (true, Some(channel_id)) = (focused, selected_channel.peek()) {
            requiem_notifications::clear_channel(channel_id);
        }
        properties::set_app_focused(focused);

        presence_local::note_activity();
        state.read_states.set_window_focused(focused);
    }
}

pub(crate) fn resize_hook(signals: &RootSignals) -> impl FnMut(f32, f32) + 'static {
    let profile_popout_window_resize_tick = signals.profile_popout.window_resize_tick.clone();
    move |_width, _height| profile_popout_window_resize_tick.update(|n| *n += 1)
}

/// Registers every popout with the escape-key stack.
pub(crate) fn register_popouts(signals: &RootSignals) {
    let picker = signals.emoji_picker.open.clone();
    popouts::register(
        "emoji picker",
        {
            let p = picker.clone();
            move || p.peek()
        },
        move || picker.set(false),
    );
    let account = signals.account_popout.clone();
    popouts::register(
        "account",
        {
            let a = account.clone();
            move || a.open.peek()
        },
        move || account.close(),
    );
    let (profile, member, overlays) = (
        signals.profile_popout.clone(),
        signals.profile_popout.overlays.clone(),
        signals.profile_popout.overlays.clone(),
    );
    popouts::register(
        "profile",
        {
            let p = profile.clone();
            move || p.open.peek().is_some()
        },
        move || profile_popout::close_popouts(&profile),
    );
    popouts::register(
        "member profile",
        {
            let o = overlays.clone();
            move || o.member_target.peek().is_some()
        },
        move || member.member_target.set(None),
    );
    popouts::register("pins", pins::is_open, pins::close);
    popouts::register("inbox", inbox::is_open, inbox::close);
    popouts::register(
        "reaction picker",
        reactions::picker_open,
        reactions::close_picker,
    );
    popouts::register(
        "reaction users",
        reactions::list_open,
        reactions::close_list,
    );
    popouts::register("context menu", context_menu::is_open, context_menu::close);
    let attach = signals.attachments.menu_open.clone();
    popouts::register(
        "attach menu",
        {
            let a = attach.clone();
            move || a.peek()
        },
        move || attach.set(false),
    );
    let search = signals.search.popout_open.clone();
    popouts::register(
        "search",
        {
            let s = search.clone();
            move || s.peek()
        },
        move || search.set(false),
    );
}

/// Whether a modal dialog is up, which disables the composer shortcuts behind it.
pub(crate) fn make_modal_open(signals: &RootSignals) -> Rc<dyn Fn() -> bool> {
    let overlays = signals.profile_popout.overlays.clone();
    let custom_status = signals.account_popout.custom_status.clone();
    let (settings_modal_open, settings_ui) = (
        signals.settings_modal_open.clone(),
        signals.settings_ui.clone(),
    );
    let (emoji_picker_open, fullscreen_video) = (
        signals.emoji_picker.open.clone(),
        signals.fullscreen_video.clone(),
    );
    let media_viewer = signals.media_viewer.clone();
    let attachment_dialogs = signals.attachments.clone();
    Rc::new(move || {
        media_viewer.is_open()
            || quick_switcher::is_open()
            || shortcuts_dialog::is_open()
            || new_message::is_open()
            || forward::is_open()
            || create_guild::is_open()
            || create_channel::is_open()
            || guild_settings::is_open()
            || attachment_dialogs.modify.open.peek().on
            || attachment_dialogs.too_many_open.peek().on
            || settings_modal_open.peek().on
            || settings_ui.peek().logout_confirm.on
            || overlays.block_open.peek().on
            || overlays.full_profile_ui.open.peek().on
            || custom_status.open.peek().on
            || emoji_picker_open.peek()
            || fullscreen_video.peek().is_some()
    })
}

/// The state the named-key hook (arrows, escape, edit shortcuts) reads.
#[derive(Clone)]
struct KeyEnv {
    modal_open: Rc<dyn Fn() -> bool>,
    fullscreen_video: Signal<Option<video_player::FullscreenVideoState>>,
    overlays: ProfileOverlays,
    custom_status: custom_status::CustomStatusModalState,
    settings_ui: Signal<SettingsUiState>,
    attachments: AttachmentsState,
    viewer: MediaViewerState,
    settings_modal_open: Signal<ToggleState>,
    close_settings: Rc<dyn Fn()>,
    state: AppState,
    selected_channel: Signal<Option<Id<ChannelMarker>>>,
    message_scroll: Signal<HashMap<Id<ChannelMarker>, MessageListScroll>>,
}

pub(crate) fn named_key_hook(
    signals: &RootSignals,
    state: &AppState,
    modal_open: &Rc<dyn Fn() -> bool>,
) -> impl FnMut(&mut WidgetTree, NamedKey) -> bool + 'static {
    let env = KeyEnv {
        modal_open: modal_open.clone(),
        fullscreen_video: signals.fullscreen_video.clone(),
        overlays: signals.profile_popout.overlays.clone(),
        custom_status: signals.account_popout.custom_status.clone(),
        settings_ui: signals.settings_ui.clone(),
        attachments: signals.attachments.clone(),
        viewer: signals.media_viewer.clone(),
        settings_modal_open: signals.settings_modal_open.clone(),
        close_settings: signals.on_close_settings.clone(),
        state: state.clone(),
        selected_channel: signals.selected_channel.clone(),
        message_scroll: signals.message_scroll.clone(),
    };
    move |tree, key| env.on_key(tree, key)
}

impl KeyEnv {
    /// Returns whether the key was handled, which stops the normal key dispatch.
    fn on_key(&self, tree: &mut WidgetTree, key: NamedKey) -> bool {
        if self.message_keys(tree, key) {
            return true;
        }
        if let Some(handled) = self.viewer_key(key) {
            return handled;
        }
        if key != NamedKey::Escape {
            return false;
        }
        self.dismiss_global_dialogs()
            || self.dismiss_settings_surfaces()
            || self.escape_to_composer(tree)
    }

    /// The context menu's own keys, and the message list's keyboard navigation.
    fn message_keys(&self, tree: &mut WidgetTree, key: NamedKey) -> bool {
        let viewer = &self.viewer;
        let (key_state, key_selected) = (&self.state, &self.selected_channel);
        let modal_open_for_keys = &self.modal_open;
        if context_menu::is_open() && components::context_menu_handle_key(key) {
            return true;
        }
        if !viewer.is_open() {
            if let Some(channel) = ui::displayed_channel(key_state, key_selected.peek()) {
                let on_message = message_keys::focused_message(tree).is_some();
                let handled = match key {
                    Key::ArrowUp | Key::ArrowDown if on_message => message_keys::step(tree, if key == Key::ArrowUp { -1 } else { 1 }),
                    Key::Home | Key::End if on_message => message_keys::focus_end(tree, key == Key::Home),
                    Key::Backspace => message_keys::delete_focused(tree, key_state, &channel),
                    Key::Copy => message_keys::armed(tree) && message_keys::copy_focused(tree, key_state, &channel),
                    Key::ArrowUp => {
                        let composer_focused = paint::find_by_dom_id(tree, "composer-input").is_some_and(|id| tree.focused == Some(id));
                        composer_focused && key_state.drafts.text(channel.id).is_empty() && !message_edit::is_active() && !(modal_open_for_keys)() && message_keys::edit_last_own(key_state, channel.id)
                    }
                    _ => false,
                };
                if handled {
                    return true;
                }
            }
        }
        false
    }

    /// While the media viewer is open it owns the arrow keys and escape.
    fn viewer_key(&self, key: NamedKey) -> Option<bool> {
        let viewer = &self.viewer;
        if !viewer.is_open() {
            return None;
        }
        match key {
            NamedKey::ArrowLeft => viewer.step(-1),
            NamedKey::ArrowRight => viewer.step(1),
            NamedKey::Escape => {
                viewer.escape();
            }
            _ => return Some(false),
        }
        Some(true)
    }

    /// Escape closes the topmost of the app-level dialogs.
    fn dismiss_global_dialogs(&self) -> bool {
        let (key_state, key_selected) = (&self.state, &self.selected_channel);
        if emoji_style_menu::is_open() {
            emoji_style_menu::close();
            return true;
        }
        if quick_switcher::is_open() {
            quick_switcher::close();
            return true;
        }
        if shortcuts_dialog::is_open() {
            shortcuts_dialog::close();
            return true;
        }
        if reactions::picker_open() {
            reactions::close_picker();
            return true;
        }
        if message_edit::is_active() {
            message_edit::cancel();
            return true;
        }
        if let Some(channel) = ui::displayed_channel(key_state, key_selected.peek()).map(|c| c.id).filter(|&c| reply::is_active(c)) {
            reply::cancel(channel);
            return true;
        }
        if new_message::is_open() {
            new_message::close();
            return true;
        }
        if forward::is_open() {
            forward::close();
            return true;
        }
        if create_guild::is_open() {
            create_guild::close();
            return true;
        }
        if create_channel::is_open() {
            create_channel::close();
            return true;
        }
        if invite_modal::is_open() {
            invite_modal::escape();
            return true;
        }

        if shell::native_dialog_just_closed() {
            return true;
        }
        false
    }

    /// Escape closes the topmost settings-related surface: color popouts, popouts, guild settings,
    /// attachment dialogs, profile dialogs and the settings modal itself.
    fn dismiss_settings_surfaces(&self) -> bool {
        let KeyEnv {
            fullscreen_video,
            overlays,
            custom_status,
            settings_ui,
            settings_modal_open: escape_settings_open,
            close_settings: escape_close_settings,
            state: key_state,
            ..
        } = self;
        let escape_attachments = &self.attachments;
        let color_popout = settings_ui.peek().color_popout;
        if color_popout.peek().is_some() {
            color_popout.set(None);
            return true;
        }

        if context_menu::is_open() {
            context_menu::close();
            return true;
        }

        if popouts::close_open() {
            return true;
        }

        {
            if features_guild_settings::close_color_popout() {
                return true;
            }
            let server_settings = guild_settings::state();
            if server_settings.dialog.peek().is_some() {
                server_settings.dialog.set(None);
                return true;
            }
            if guild_settings::is_open() {
                guild_settings::close();
                return true;
            }
        }
        if fullscreen_video.peek().is_some() {
            video_player::exit_fullscreen(fullscreen_video);
            return true;
        }
        if escape_attachments.menu_open.peek() {
            escape_attachments.menu_open.set(false);
            return true;
        }

        if escape_attachments.too_many_open.peek().on {
            escape_attachments.close_too_many();
            return true;
        }
        if escape_attachments.modify.open.peek().on {
            escape_attachments.close_modify();
            return true;
        }

        let profile_dialog = settings_ui.peek().profile_dialog;
        if profile_dialog.peek().open.on {
            profile_dialog.update(|d| d.open = d.open.set(false));
            return true;
        }

        let plugin_install = settings_ui.peek().plugin_install;
        if plugin_install.peek().open.on {
            plugin_install::close(&plugin_install);
            return true;
        }

        if settings_ui.peek().logout_confirm.on {
            settings_ui.update(|ui| ui.logout_confirm = ui.logout_confirm.set(false));
            return true;
        }

        if overlays.block_open.peek().on {
            overlays.close_block_confirm();
            return true;
        }
        if overlays.full_profile_ui.open.peek().on {
            overlays.full_profile_ui.close();
            return true;
        }
        if custom_status.open.peek().on {
            custom_status.close();
            return true;
        }

        if key_state.plugins.close_top_surface() {
            return true;
        }

        if escape_settings_open.peek().on {
            escape_close_settings();
            return true;
        }
        false
    }

    /// With nothing left to close, escape jumps the message list to the present and focuses the
    /// composer.
    fn escape_to_composer(&self, tree: &mut WidgetTree) -> bool {
        let escape_modal_open = &self.modal_open;
        let escape_state = &self.state;
        let escape_selected_channel = &self.selected_channel;
        let escape_message_scroll = &self.message_scroll;
        if escape_modal_open() {
            return false;
        }
        let Some(composer) = paint::find_by_dom_id(tree, "composer-input") else {
            return false;
        };
        let other_text_focused = tree.focused.is_some_and(|id| id != composer && tree.nodes.get(id).is_some_and(|n| n.widget.wants_text_input()));
        if other_text_focused {
            return false;
        }
        let channel_id = ui::displayed_channel(escape_state, escape_selected_channel.peek()).map(|c| c.id);
        if let Some(channel_id) = channel_id {
            message_list::jump_to_present(escape_message_scroll, channel_id);
            escape_state.read_states.mark_channel_read(channel_id);
        }
        let composer_was_focused = tree.focused == Some(composer);
        paint::focus_dom_id(tree, "composer-input");

        !composer_was_focused
    }
}

/// Dropping files on the window attaches them to the open channel (shift sends right away).
pub(crate) fn file_drop_hook(
    signals: &RootSignals,
    state: &AppState,
    modal_open: &Rc<dyn Fn() -> bool>,
) -> impl FnMut(&mut WidgetTree, FileDrop) + 'static {
    let attachments = signals.attachments.clone();
    let selected_channel = signals.selected_channel.clone();
    let drop_state = state.clone();
    let drop_scroll = signals.message_scroll.clone();
    let modal_open = modal_open.clone();

    let batch: Rc<RefCell<Option<(Id<ChannelMarker>, Vec<PathBuf>, bool)>>> = Rc::default();
    move |_tree, drop| match drop {
        FileDrop::Hovered => {
            if !modal_open() && !attachments.drag_hover.peek() && ui::can_attach_files(&drop_state, selected_channel.peek()) {
                attachments.drag_hover.set(true);
                poll_shift_while_dragging(attachments.clone());
            }
        }
        FileDrop::Cancelled => {
            attachments.drag_hover.set(false);
            attachments.shift_held.set(false);
        }
        FileDrop::Dropped { path, shift } => {
            let shift = shift || os_shift_down();
            attachments.drag_hover.set(false);
            attachments.shift_held.set(false);
            if modal_open() || !ui::can_attach_files(&drop_state, selected_channel.peek()) {
                return;
            }

            let Some(channel_id) = ui::displayed_channel(&drop_state, selected_channel.peek()).map(|c| c.id) else { return };
            let first = {
                let mut batch = batch.borrow_mut();
                let entry = batch.get_or_insert_with(|| (channel_id, Vec::new(), false));
                entry.1.push(path);
                entry.2 |= shift;
                entry.1.len() == 1
            };
            if first {
                let (attachments, batch, scroll) = (attachments.clone(), batch.clone(), drop_scroll.clone());
                timer::set_timeout(Duration::from_millis(100), move || {
                    let Some((channel_id, paths, shift)) = batch.borrow_mut().take() else { return };
                    let result = attachments.add_paths(channel_id, paths);
                    if !result.unreadable.is_empty() || !result.empty.is_empty() {
                        log::warn!("attachments: dropped files skipped (unreadable: {:?}, empty: {:?})", result.unreadable, result.empty);
                    }
                    if shift && result.added > 0 {
                        if let Some((nonce, files)) = attachments.begin_send(channel_id, String::new()) {
                            (attachments.backend.send)(channel_id, nonce, String::new(), files);
                            message_list::jump_to_present(&scroll, channel_id);
                        }
                    }
                });
            }
        }
    }
}

/// What `build_app` hangs the hooks on.
pub(crate) struct AppEnv<'a> {
    pub state: &'a AppState,
    pub signals: &'a RootSignals,
    pub auth_state: &'a AuthState,
    pub modal_open: &'a Rc<dyn Fn() -> bool>,
    pub frame_waker: &'a FrameWaker,
}

/// Creates the application with the user-event handler and every input hook attached.
pub(crate) fn build_app(
    tree: WidgetTree,
    handler: EventHandler,
    shortcut_env: shortcuts::ShortcutEnv,
    env: &AppEnv<'_>,
) -> App<AppEvent> {
    let AppEnv {
        state,
        signals,
        auth_state,
        modal_open,
        frame_waker,
    } = env;
    App::<AppEvent>::new("Requiem", tree, move |event, tree| {
        handler.handle(event, tree)
    })
    .with_animation_settle(Duration::from_millis(1250))
    .with_frame_waker((*frame_waker).clone())
    .with_mouse_down_hook(mouse_down_hook(signals, auth_state))
    .with_shortcut_hook(shortcuts::hook(shortcut_env))
    .with_text_fallback_hook(text_fallback_hook(modal_open, state, &signals.selected_channel))
    .with_custom_chrome()
    .with_focus_hook(focus_hook(state, &signals.selected_channel))
    .with_resize_hook(resize_hook(signals))
    .with_named_key_hook(named_key_hook(signals, state, modal_open))
    .with_file_drop_hook(file_drop_hook(signals, state, modal_open))
}
