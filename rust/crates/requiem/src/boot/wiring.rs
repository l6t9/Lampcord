
use requiem::features::title_bar;
use requiem::features::user_settings::profile_settings;
use requiem::features::voice::voice_settings;
use requiem::notifications as requiem_notifications;
use requiem_core::ui::context_menu;
use requiem_core::ui::create_guild;
use requiem_core::ui::custom_status;
use requiem_core::ui::media_viewer;
use requiem_core::ui::message_list;
use requiem_core::ui::navigation;
use requiem_core::ui::profile_popout;
use requiem_core::ui;
use requiem_core::utils::time;
use requiem_core::voice;
use requiem_ui::timer;
use std::env;
use std::path::PathBuf;
use tokio::sync::mpsc;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use requiem::shell;
use requiem_core::models::id::marker::ChannelMarker;
use requiem_core::models::Id;
use requiem_core::presence::{CustomStatus, OwnStatus, PresenceField, PresenceUpdate};
use requiem_core::sounds::{self, Sound};
use requiem_core::state::AppState;
use requiem_core::ui::RootSignals;
use winit::event_loop::EventLoopProxy;

use crate::AppEvent;
use super::{Senders, Services};

/// Fills in the callbacks that need more than the signals themselves: channel selection, the voice
/// toggles, settings, presence, profile popouts and the context menu. Returns the root signals and
/// the function that re-arms the custom-status expiry timer.
pub(crate) fn wire_callbacks(
    mut signals: RootSignals,
    state: &AppState,
    services: &Services,
    senders: &Senders,
) -> (RootSignals, Rc<dyn Fn()>) {
    wire_channel_selection(&mut signals, state, &senders.select_tx);
    wire_voice_toggles(&mut signals);
    install_image_picker();
    wire_settings(&mut signals, &services.proxy_slot);
    let arm_custom_status_expiry = wire_presence(&mut signals, state, &senders.status_tx);
    wire_profile_overlays(&mut signals, state);
    install_context_menu(&signals, state, services);
    signals.member_list.on_popout_open = {
        let prompt = signals.profile_popout.overlays.prompt.clone();
        Rc::new(move || prompt.set(custom_status::random_prompt()))
    };
    (signals, arm_custom_status_expiry)
}

/// Jumping to a search hit selects its channel first when it is not the open one.
pub(crate) fn wire_search_jump(signals: &RootSignals) {
    *signals.search.on_jump.borrow_mut() = {
        let (select, selected) = (
            signals.on_select_channel.clone(),
            signals.selected_channel.clone(),
        );
        Rc::new(move |channel_id, message_id| {
            if selected.peek() != Some(channel_id) {
                select(channel_id);
            }
            message_list::request_jump(channel_id, message_id);
        })
    };
}

fn wire_channel_selection(
    signals: &mut RootSignals,
    state: &AppState,
    select_tx: &mpsc::UnboundedSender<Id<ChannelMarker>>,
) {
    let selected_channel = signals.selected_channel.clone();
    signals.on_select_channel = {
        let selected_channel = selected_channel.clone();
        let select_tx = select_tx.clone();
        let state = state.clone();
        Rc::new(move |channel_id: Id<ChannelMarker>| {
            if let Some((guild_id, channel)) =
                ui::find_channel(&state, channel_id)
            {
                if channel.kind == 2 {
                    let (g, c) = (u64::from(guild_id), u64::from(channel_id));
                    if voice::current_channel() != Some((g, c)) {
                        voice_settings::join_voice(
                            &state, g, &channel,
                        );
                    }
                }
            }
            title_bar::record_navigation(
                selected_channel.peek(),
                channel_id,
            );
            navigation::note_selection(&state, channel_id);

            state.nav.home.set(state.dms.find(channel_id).is_some());
            selected_channel.set(Some(channel_id));
            requiem_notifications::clear_channel(channel_id);
            let _ = select_tx.send(channel_id);
        })
    };
}

fn wire_voice_toggles(signals: &mut RootSignals) {
    let mute_toggle = signals.mute_toggle.clone();
    let deafen_toggle = signals.deafen_toggle.clone();
    let mute_choice = Rc::new(Cell::new(false));
    signals.on_toggle_mute = {
        let (mute_toggle, deafen_toggle, mute_choice) = (
            mute_toggle.clone(),
            deafen_toggle.clone(),
            mute_choice.clone(),
        );
        Rc::new(move || {
            let next = !mute_toggle.peek().on;
            if !next && deafen_toggle.peek().on {
                deafen_toggle.update(|t| *t = t.set(false));
                voice::set_deaf(false);
            }
            mute_choice.set(next);
            mute_toggle.update(|t| *t = t.set(next));
            voice::set_mute(next);
            sounds::play(if next { Sound::Mute } else { Sound::Unmute });
        })
    };
    signals.on_toggle_deafen = {
        let (mute_toggle, deafen_toggle, mute_choice) = (
            mute_toggle.clone(),
            deafen_toggle.clone(),
            mute_choice.clone(),
        );
        Rc::new(move || {
            let next = !deafen_toggle.peek().on;
            deafen_toggle.update(|t| *t = t.set(next));
            mute_toggle.update(|t| *t = t.set(next || mute_choice.get()));
            voice::set_deaf(next);
            sounds::play(if next { Sound::Deafen } else { Sound::Undeafen });
        })
    };
}

fn install_image_picker() {
    create_guild::install_image_picker(Rc::new(|| {
        if let Some(path) = env::var_os("REQUIEM_PICK_IMAGE") {
            return Some(PathBuf::from(path));
        }
        let picked = rfd::FileDialog::new()
            .set_title(requiem_i18n::t!("guild-settings-choose-an-icon"))
            .add_filter(
                requiem_i18n::t!("dialogs-images"),
                &["png", "jpg", "jpeg", "gif", "webp"],
            )
            .pick_file();
        shell::note_native_dialog_closed();
        picked
    }));
}

/// The settings modal: opening, closing, jumping to the profile page, and logging out.
fn wire_settings(
    signals: &mut RootSignals,
    proxy_slot: &Arc<Mutex<Option<EventLoopProxy<AppEvent>>>>,
) {
    let settings_modal_open = signals.settings_modal_open.clone();
    let settings_ui = signals.settings_ui.clone();
    signals.on_open_settings = {
        let settings_modal_open = settings_modal_open.clone();
        Rc::new(move || settings_modal_open.update(|t| *t = t.set(true)))
    };
    signals.account_popout.on_edit_profile = {
        let settings_modal_open = settings_modal_open.clone();
        let settings_ui = settings_ui.clone();
        Rc::new(move || {
            settings_ui.update(|ui| {
                ui.category =
                    profile_settings::PROFILES_CATEGORY
                        .to_string();
                ui.sub_page = None;
            });
            settings_modal_open.update(|t| *t = t.set(true));
        })
    };
    signals.on_close_settings = {
        let settings_modal_open = settings_modal_open.clone();
        Rc::new(move || settings_modal_open.update(|t| *t = t.set(false)))
    };

    signals.on_log_out = {
        let proxy_slot = proxy_slot.clone();
        Rc::new(move || {
            if let Some(proxy) = proxy_slot.lock().expect("proxy slot poisoned").as_ref() {
                let _ = proxy.send_event(AppEvent::LogOut);
            }
        })
    };
}

/// Own status and custom status, with the timer that clears an expiring custom status. Returns
/// the function that (re)arms that timer.
fn wire_presence(
    signals: &mut RootSignals,
    state: &AppState,
    status_tx: &mpsc::UnboundedSender<PresenceUpdate>,
) -> Rc<dyn Fn()> {
    signals.account_popout.on_set_status = {
        let (own_status, own_custom) = (
            state.presence.status.clone(),
            state.presence.custom_status.clone(),
        );
        let status_tx = status_tx.clone();
        Rc::new(move |status: OwnStatus| {
            own_status.set(status);
            let _ = status_tx.send(PresenceUpdate {
                status,
                custom: own_custom.peek(),
                changed: PresenceField::Status,
            });
        })
    };

    let expiry_timer: Rc<Cell<Option<timer::TimeoutHandle>>> =
        Rc::new(Cell::new(None));
    let set_custom: Rc<RefCell<Option<Rc<dyn Fn(Option<CustomStatus>)>>>> =
        Rc::new(RefCell::new(None));
    let arm_expiry: Rc<dyn Fn()> = {
        let (own_custom, timer, set_custom) = (
            state.presence.custom_status.clone(),
            expiry_timer.clone(),
            set_custom.clone(),
        );
        Rc::new(move || {
            if let Some(handle) = timer.take() {
                timer::clear_timeout(handle);
            }
            let Some(expires) = own_custom.peek().and_then(|c| c.expires_at_ms) else {
                return;
            };
            let remaining = Duration::from_millis(
                expires.saturating_sub(time::now_ms()),
            );
            let (own_custom, set_custom) = (own_custom.clone(), set_custom.clone());
            timer.set(Some(timer::set_timeout(remaining, move || {
                if own_custom
                    .peek()
                    .is_some_and(|c| c.is_expired(time::now_ms()))
                {
                    if let Some(set) = set_custom.borrow().as_ref() {
                        set(None);
                    }
                }
            })));
        })
    };
    signals.account_popout.on_set_custom_status = {
        let (own_status, own_custom, arm_expiry) = (
            state.presence.status.clone(),
            state.presence.custom_status.clone(),
            arm_expiry.clone(),
        );
        let status_tx = status_tx.clone();
        Rc::new(move |custom: Option<CustomStatus>| {
            own_custom.set(custom.clone());
            arm_expiry();
            let _ = status_tx.send(PresenceUpdate {
                status: own_status.peek(),
                custom,
                changed: PresenceField::CustomStatus,
            });
        })
    };
    *set_custom.borrow_mut() = Some(signals.account_popout.on_set_custom_status.clone());
    arm_expiry
}

fn wire_profile_overlays(signals: &mut RootSignals, state: &AppState) {
    let selected_channel = signals.selected_channel.clone();
    let overlays = signals.profile_popout.overlays.clone();
    signals.account_popout.on_view_full_profile = {
        let (state, overlays, selected_channel) =
            (state.clone(), overlays.clone(), selected_channel.clone());
        Rc::new(move || {
            let Some(user_id) = state
                .users
                .current
                .with_untracked(|u| u.as_ref().map(|u| u.id))
            else {
                return;
            };
            let guild_id =
                ui::resolve_selected_guild(&state, selected_channel.peek());
            overlays.open_full_profile(profile_popout::FullProfileTarget {
                user_id,
                guild_id,
                focus_note: false,
                tab: Default::default(),
                server_guild_id: None,
            });
        })
    };
    signals.profile_popout.overlays.on_open_custom_status = {
        let account = signals.account_popout.clone();
        Rc::new(move |current: Option<CustomStatus>| {
            account.open_custom_status(current.as_ref())
        })
    };
    signals.profile_popout.overlays.on_clear_custom_status = {
        let set = signals.account_popout.on_set_custom_status.clone();
        Rc::new(move || set(None))
    };
    signals.profile_popout.overlays.on_edit_profile =
        signals.account_popout.on_edit_profile.clone();
    signals.profile_popout.overlays.on_open_guild = {
        let (state, on_select) = (state.clone(), signals.on_select_channel.clone());
        Rc::new(move |guild_id| {
            if let Some(channel) = ui::default_channel(&state, Some(guild_id)) {
                on_select(channel.id);
            }
        })
    };

    signals.member_list.popout_target = signals.profile_popout.overlays.member_target.clone();
}

fn install_context_menu(signals: &RootSignals, state: &AppState, services: &Services) {
    let Services {
        toasts, proxy_slot, ..
    } = services;
    let overlays = signals.profile_popout.overlays.clone();
    let selected_channel = signals.selected_channel.clone();
    let settings_ui = signals.settings_ui.clone();
    let settings_modal_open = signals.settings_modal_open.clone();
    context_menu::install(context_menu::Actions {
        toasts: toasts.clone(),
        open_profile: {
            let (overlays, state, selected_channel) =
                (overlays.clone(), state.clone(), selected_channel.clone());
            Rc::new(move |user_id, guild_id, focus_note| {
                let guild_id = guild_id.or_else(|| {
                    ui::resolve_selected_guild(&state, selected_channel.peek())
                });
                overlays.open_full_profile(
                    profile_popout::FullProfileTarget {
                        user_id,
                        guild_id,
                        focus_note,
                        tab: Default::default(),
                        server_guild_id: None,
                    },
                );
            })
        },
        confirm_block: {
            let overlays = overlays.clone();
            Rc::new(move |user_id, name, location| {
                overlays.confirm_block(user_id, name, location)
            })
        },
        dm_channel: {
            let state = state.clone();
            Rc::new(move |user_id| state.dms.with_user(user_id).map(|dm| dm.id))
        },
        open_settings: {
            let (settings_ui, open) = (settings_ui.clone(), settings_modal_open.clone());
            Rc::new(move |page: &str| {
                settings_ui.update(|ui| {
                    ui.category = page.to_string();
                    ui.sub_page = None;
                });
                open.update(|t| *t = t.set(true));
            })
        },
        open_guild: signals.profile_popout.overlays.on_open_guild.clone(),
        open_media: {
            let viewer = signals.media_viewer.clone();
            Rc::new(move |url| {
                viewer.open(
                    vec![media_viewer::MediaItem {
                        url,
                        video_url: None,
                        width: None,
                        height: None,
                        placeholder: None,
                        filename: None,
                        size: None,
                        alt: None,
                        author: None,
                        is_attachment: false,
                        zoom_only: true,
                    }],
                    0,
                )
            })
        },
        log_out: {
            let settings_ui = settings_ui.clone();
            Rc::new(move || {
                settings_ui.update(|ui| ui.logout_confirm = ui.logout_confirm.set(true))
            })
        },
        editor_command: {
            let proxy_slot = proxy_slot.clone();
            Rc::new(move |command| {
                if let Some(proxy) = proxy_slot.lock().expect("proxy slot poisoned").as_ref() {
                    let _ = proxy.send_event(AppEvent::EditorCommand(command));
                }
            })
        },
    });
}
