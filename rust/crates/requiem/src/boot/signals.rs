use std::collections::HashMap;
use std::fs;
use std::rc::Rc;
use std::thread;

use requiem::shell;
use requiem_core::models::Id;
use requiem_core::models::id::marker::ChannelMarker;
use requiem_core::rest::endpoints::favorite_gifs::FavoriteGifs;
use requiem_core::ui::account_popout::AccountPopoutState;
use requiem_core::ui::attachments::{AttachmentsBackend, AttachmentsState};
use requiem_core::ui::emoji_picker::{
    EmojiPickerSignals, EmojiPickerState, PickerCategory, PickerTab,
};
use requiem_core::ui::gif_picker::{GifPickerSignals, GifPickerState, GifView};
use requiem_core::ui::media_viewer;
use requiem_core::ui::media_viewer::MediaViewerState;
use requiem_core::ui::member_list::MemberListState;
use requiem_core::ui::profile_popout::{ProfileOverlays, ProfilePopoutState};
use requiem_core::ui::search_state::SearchState;
use requiem_core::ui::{RootSignals, SettingsUiState};
use requiem_core::uploads::UploadFile;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::anim::ToggleState;

use super::{Senders, Services};
use crate::{AppEvent, GifQuery, UploadCommand};

/// Builds the root signals with their state. Every callback starts as a no-op; `wire_callbacks`
/// fills them in.
pub(crate) fn build_root_signals(services: &Services, senders: &Senders) -> RootSignals {
    let global_tick: Signal<u64> = signal(0);
    RootSignals {
        selected_channel: signal(None),
        member_list: MemberListState::new(),
        search: SearchState::new(),
        account_popout: AccountPopoutState::default(),
        on_select_channel: Rc::new(|_| {}),
        composer_focused: signal(true),
        composer_has_text: signal(false),
        composer_send_toggle: signal(ToggleState::new(false)),
        message_scroll: signal(HashMap::new()),
        mute_toggle: signal(ToggleState::new(false)),
        on_toggle_mute: Rc::new(|| {}),
        deafen_toggle: signal(ToggleState::new(false)),
        on_toggle_deafen: Rc::new(|| {}),
        settings_modal_open: signal(ToggleState::new(false)),
        on_open_settings: Rc::new(|| {}),
        on_close_settings: Rc::new(|| {}),
        on_log_out: Rc::new(|| {}),
        settings_ui: signal(SettingsUiState::default()),
        global_tick: global_tick.clone(),
        fullscreen_video: signal(None),
        attachments: attachments_state(services, senders),
        toasts: services.toasts.clone(),
        media_viewer: media_viewer_state(services),
        emoji_picker: emoji_picker_state(services, senders, &global_tick),
        profile_popout: profile_popout_state(),
    }
}

/// Picking files opens the native dialog on its own thread; sending, cancelling and deleting go
/// to the upload task.
fn attachments_state(services: &Services, senders: &Senders) -> AttachmentsState {
    AttachmentsState::new(AttachmentsBackend {
        pick_files: {
            let proxy_slot = services.proxy_slot.clone();
            Rc::new(move |channel_id: Id<ChannelMarker>| {
                let Some(proxy) = proxy_slot.lock().expect("proxy slot poisoned").clone() else {
                    return;
                };
                thread::spawn(move || {
                    let picked_files = rfd::FileDialog::new()
                        .set_title(requiem_i18n::t!("main-upload-a-file"))
                        .pick_files();
                    shell::note_native_dialog_closed();
                    if let Some(paths) = picked_files {
                        let _ = proxy.send_event(AppEvent::FilesPicked { channel_id, paths });
                    }
                });
            })
        },
        send: {
            let upload_tx = senders.upload_tx.clone();
            Rc::new(
                move |channel_id: Id<ChannelMarker>,
                      nonce: u64,
                      content: String,
                      files: Vec<UploadFile>| {
                    let _ = upload_tx.send(UploadCommand::Send {
                        channel_id,
                        nonce,
                        content,
                        files,
                    });
                },
            )
        },
        cancel: {
            let upload_tx = senders.upload_tx.clone();
            Rc::new(move |nonce: u64| {
                let _ = upload_tx.send(UploadCommand::Cancel(nonce));
            })
        },
        delete_upload: {
            let upload_tx = senders.upload_tx.clone();
            Rc::new(move |name: String| {
                let _ = upload_tx.send(UploadCommand::Delete(name));
            })
        },
    })
}

fn media_viewer_state(services: &Services) -> MediaViewerState {
    let proxy_slot = services.proxy_slot.clone();
    let save_to_disk = Rc::new(move |bytes: Vec<u8>, filename: String| {
        let Some(proxy) = proxy_slot.lock().expect("proxy slot poisoned").clone() else {
            return;
        };

        thread::spawn(move || {
            let chosen = rfd::FileDialog::new()
                .set_title(requiem_i18n::t!("main-save-as"))
                .set_file_name(filename)
                .save_file();
            shell::note_native_dialog_closed();
            let Some(path) = chosen else { return };
            let saved = match fs::write(&path, bytes) {
                Ok(()) => true,
                Err(err) => {
                    log::warn!("media viewer: saving {} failed: {err}", path.display());
                    false
                }
            };
            let _ = proxy.send_event(AppEvent::Toast {
                message: if saved {
                    requiem_i18n::t!("main-image-saved")
                } else {
                    requiem_i18n::t!("main-unable-to-save-image")
                },
                success: saved,
            });
        });
    });
    MediaViewerState::new(Some(save_to_disk))
        .with_toasts(services.toasts.clone())
        .with_video_frames(media_viewer::video_frames(services.videos.clone()))
}

/// The emoji picker and its GIF tab. Inserting text goes through the event loop so it lands in
/// whichever input has focus.
fn emoji_picker_state(
    services: &Services,
    senders: &Senders,
    global_tick: &Signal<u64>,
) -> EmojiPickerState {
    let gif_picker_view: Signal<GifView> = signal(GifView::Browse);
    EmojiPickerState {
        open: signal(false),
        tab: signal(PickerTab::Emoji),
        signals: EmojiPickerSignals {
            search: signal(String::new()),
            scroll_offset: signal(0.0),
            category_jump: signal(None::<PickerCategory>),
            hovered: signal(None),
            collapsed: signal(Vec::new()),
        },
        gif: GifPickerState {
            signals: GifPickerSignals {
                search: signal(String::new()),
                categories: signal(Vec::new()),
                trending: signal(Vec::new()),
                results: signal(Vec::new()),
                favorites: signal(FavoriteGifs::default()),
                view: gif_picker_view.clone(),
                image_tick: global_tick.clone(),
            },
            on_search: {
                let gif_query_tx = senders.gif_query_tx.clone();
                Rc::new(move |query: String| {
                    let _ = gif_query_tx.send(GifQuery::Search(query));
                })
            },
            on_open: {
                let gif_query_tx = senders.gif_query_tx.clone();
                Rc::new(move || {
                    gif_picker_view.set(GifView::Browse);
                    let _ = gif_query_tx.send(GifQuery::Trending);
                    let _ = gif_query_tx.send(GifQuery::LoadFavorites);
                })
            },
            on_save_favorites: {
                let gif_query_tx = senders.gif_query_tx.clone();
                Rc::new(move |favorites: FavoriteGifs| {
                    let _ = gif_query_tx.send(GifQuery::SaveFavorites(favorites));
                })
            },
        },
        on_insert: {
            let proxy_cell = services.emoji_insert_proxy.clone();
            Rc::new(move |text: String| {
                if let Some(proxy) = proxy_cell.borrow().as_ref() {
                    let _ = proxy.send_event(AppEvent::InsertComposerText(text));
                }
            })
        },
        on_insert_at: {
            let proxy_cell = services.emoji_insert_proxy.clone();
            Rc::new(move |dom_id: &'static str, text: String| {
                if let Some(proxy) = proxy_cell.borrow().as_ref() {
                    let _ = proxy.send_event(AppEvent::InsertText { dom_id, text });
                }
            })
        },
    }
}

fn profile_popout_state() -> ProfilePopoutState {
    ProfilePopoutState {
        open: signal(None),
        draft: signal(String::new()),
        emoji_open: signal(false),
        emoji_tab: signal(PickerTab::Emoji),
        emoji_search: signal(String::new()),
        emoji_scroll_offset: signal(0.0),
        emoji_category_jump: signal(None),
        emoji_hovered: signal(None),
        emoji_collapsed: signal(Vec::new()),
        role_picker_open: signal(false),
        window_resize_tick: signal(0),
        overlays: ProfileOverlays::new(),
    }
}
