use crate::images;
use crate::models::Channel;
use crate::models;
use crate::plugins;
use crate::rest::endpoints::guilds;
use crate::rest::endpoints::messages::AttachmentPatch;
use crate::rest::endpoints::messages::edit_message_attachments;
use crate::rest::endpoints::messages;
use crate::rest;
use crate::state;
use crate::ui::attachments;
use crate::ui::classes::voice_users as classes_voice_users;
use crate::ui::create_channel;
use crate::ui::emoji_skin;
use crate::ui::forward;
use crate::ui::guild_settings;
use crate::ui::invite_modal::location;
use crate::ui::invite_modal;
use crate::ui::media_viewer;
use crate::ui::message_confirm;
use crate::ui::message_edit;
use crate::ui::message_embed;
use crate::ui::notification_settings;
use crate::ui::pins;
use crate::ui::playground;
use crate::ui::plugin_view;
use crate::ui::reactions;
use crate::ui::reply;
use crate::ui::threads;
use crate::ui::voice_users;
use crate::ui;
use crate::utils::link;
use crate::utils::prefs;
use crate::utils::voice_settings::slider_to_volume;
use crate::utils::voice_settings::volume_to_slider;
use crate::voice_control;
use requiem_ui::app;
use requiem_ui::classes::context_menu as classes_context_menu;
use requiem_ui::components::IconPosition;
use requiem_ui::components::SliderProps;
use requiem_ui::components::TextProps;
use requiem_ui::components::TextVariant;
use requiem_ui::components::div;
use requiem_ui::icons::Icon;
use requiem_ui::node;
use requiem_ui::timer;
use requiem_ui::widget;
use std::fmt::Display;
use std::future::Future;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use requiem_ui::components::ContextItem;

use crate::config::BackendConfig;
use crate::models::id::marker::{ChannelMarker, GuildMarker, MessageMarker, UserMarker};
use crate::models::{Attachment, Id, Permissions};
use crate::query::QueryClient;
use crate::rest::HttpClient;
use crate::rest::endpoints::relationships::{self, BLOCKED, FRIEND, PENDING_OUTGOING};
use crate::ui::context_menu::{self as menu, ConfirmRequest, EditorCommand};
use crate::ui::profile_popout::relationship_kind;
use requiem_ui::icons;
use requiem_ui::icons::{
    APPS, BELL, BOOST_GEM, CALENDAR_PLUS, CIRCLE_PLUS, DOOR_EXIT, FOLDER_PLUS, GROUP_PLUS,
    SETTINGS, SHIELD,
};

#[derive(Clone)]
pub struct MenuEnv {
    pub backend: BackendConfig,
    pub queries: QueryClient,
    pub http: Option<HttpClient>,
    pub images: images::ImageCache,

    pub on_save: Option<Rc<dyn Fn(Vec<u8>, String)>>,
    pub own_user_id: Option<Id<UserMarker>>,

    pub state: state::AppState,
}

pub fn run_with_toast<E, Fut>(
    queries: &QueryClient,
    invalidates: Vec<String>,
    success: String,
    failure: String,
    work: impl FnOnce() -> Fut + Send + 'static,
) where
    E: Display,
    Fut: Future<Output = Result<(), E>> + Send + 'static,
{
    let outcome: Arc<Mutex<Option<bool>>> = Arc::new(Mutex::new(None));
    let slot = outcome.clone();
    queries.mutate(invalidates, move || async move {
        let result = work().await;
        *slot.lock().expect("outcome mutex poisoned") = Some(result.is_ok());
        result
    });
    poll_outcome(outcome, success, failure, 0);
}

fn poll_outcome(outcome: Arc<Mutex<Option<bool>>>, success: String, failure: String, attempt: u32) {
    timer::set_timeout(Duration::from_millis(100), move || {
        let done = outcome.lock().expect("outcome mutex poisoned").take();
        match done {
            Some(true) if success.is_empty() => {}
            Some(true) => menu::notify(&success),
            Some(false) => menu::notify_failure(&failure),
            None if attempt < 200 => poll_outcome(outcome, success, failure, attempt + 1),
            None => {}
        }
    });
}

pub struct MessageTarget {
    pub id: Id<MessageMarker>,
    pub channel_id: Id<ChannelMarker>,
    pub guild_id: Option<Id<GuildMarker>>,
    pub author_id: Id<UserMarker>,
    pub author_name: String,
    pub content: String,
    pub message: models::Message,

    pub my_reactions: Vec<models::ReactionEmoji>,
}

pub fn start_reply(target: &MessageTarget) {
    reply::start(reply::ReplyTarget {
        channel_id: target.channel_id,
        message_id: target.id,
        guild_id: target.guild_id,
        author_id: target.author_id,
        author_name: target.author_name.clone(),
        pings: true,
    });
    (menu::actions().editor_command)(EditorCommand::Focus);
}

pub fn message_link(env: &MenuEnv, target: &MessageTarget) -> String {
    let guild = target.guild_id.map_or("@me".to_string(), |g| g.to_string());
    format!(
        "{}/channels/{guild}/{}/{}",
        env.backend.web_base_url(),
        target.channel_id,
        target.id
    )
}

pub fn message_permissions(
    state: &state::AppState,
    channel_id: Id<ChannelMarker>,
) -> Permissions {
    ui::find_channel(state, channel_id).map_or(Permissions::ALL, |(_, channel)| {
        state.channel_permissions(&channel)
    })
}

pub fn message_items(env: &MenuEnv, target: &MessageTarget) -> Vec<ContextItem> {
    let own = env.own_user_id == Some(target.author_id);
    let permissions = message_permissions(&env.state, target.channel_id);
    let mut items = Vec::new();
    let (channel_id, message_id, guild_id) = (target.channel_id, target.id, target.guild_id);

    if permissions.contains(Permissions::ADD_REACTIONS) {
        items.push(ContextItem::Custom(reactions::menu_row(
            env,
            target.channel_id,
            target.guild_id,
            target.id,
        )));
        items.push(
            ContextItem::action(requiem_i18n::t!("context-menus-add-reaction"), move || {
                let (x, y) = menu::origin().unwrap_or((0.0, 0.0));
                reactions::open_picker(
                    channel_id,
                    guild_id,
                    message_id,
                    widget::AbsoluteRect {
                        x,
                        y,
                        width: 221.0,
                        height: 0.0,
                    },
                );
            })
            .icon_position(IconPosition::Leading)
            .icon(icons::EMOJI_SLIGHTLY_SMILING_FACE)
            .caret(),
        );
        items.push(ContextItem::Separator);
    }
    if own {
        let (channel_id, message_id, content) =
            (target.channel_id, target.id, target.content.clone());
        items.push(
            ContextItem::action(requiem_i18n::t!("context-menus-edit-message"), move || {
                message_edit::start(channel_id, message_id, &content)
            })
            .icon_position(IconPosition::Leading)
            .icon(icons::PENCIL),
        );
    }
    let reply_target = Rc::new(MessageTarget {
        id: target.id,
        channel_id: target.channel_id,
        guild_id: target.guild_id,
        author_id: target.author_id,
        author_name: target.author_name.clone(),
        content: String::new(),
        message: target.message.clone(),
        my_reactions: Vec::new(),
    });
    if permissions.contains(Permissions::SEND_MESSAGES) {
        items.push(
            ContextItem::action(requiem_i18n::t!("context-menus-reply"), move || {
                start_reply(&reply_target)
            })
            .icon_position(IconPosition::Leading)
            .icon(icons::ARROW_ANGLE_LEFT_UP),
        );
    }
    let forward_message = target.message.clone();
    items.push(
        ContextItem::action(requiem_i18n::t!("context-menus-forward"), move || {
            forward::open(forward_message.clone())
        })
        .icon_position(IconPosition::Leading)
        .icon(icons::ARROW_ANGLE_RIGHT_UP),
    );
    if let Some(guild_id) = guild_id {
        items.push(
            ContextItem::action(requiem_i18n::t!("context-menus-create-thread"), move || {
                threads::open_create(guild_id, channel_id, Some(message_id))
            })
            .icon_position(IconPosition::Leading)
            .icon(icons::THREAD),
        );
    }
    items.push(ContextItem::Separator);

    if !target.content.is_empty() {
        let content = target.content.clone();
        items.push(
            ContextItem::action(requiem_i18n::t!("context-menus-copy-text"), move || {
                menu::copy_with_toast(
                    content.clone(),
                    &requiem_i18n::t!("context-menus-text-copied"),
                )
            })
            .icon_position(IconPosition::Leading)
            .icon(icons::COPY),
        );
    }

    if permissions.contains(Permissions::MANAGE_MESSAGES)
        || permissions.contains(Permissions::PIN_MESSAGES)
    {
        let (channel_id, message_id) = (target.channel_id, target.id);
        let item = if env
            .state
            .messages
            .is_pinned(channel_id, message_id)
            .unwrap_or(false)
        {
            ContextItem::action(requiem_i18n::t!("pins-unpin-message"), move || {
                pins::pin_action(
                    channel_id,
                    message_id,
                    true,
                    app::shift_held(),
                )
            })
        } else {
            ContextItem::action(requiem_i18n::t!("context-menus-pin-message"), move || {
                pins::pin_action(
                    channel_id,
                    message_id,
                    false,
                    app::shift_held(),
                )
            })
        };
        items.push(item.icon_position(IconPosition::Leading).icon(icons::PIN));
    }
    items.push(
        ContextItem::unavailable(requiem_i18n::t!("context-menus-mark-unread"))
            .icon_position(IconPosition::Leading)
            .icon(icons::CHAT_MARK_UNREAD),
    );
    let link = message_link(env, target);
    items.push(
        ContextItem::action(
            requiem_i18n::t!("context-menus-copy-message-link"),
            move || {
                menu::copy_with_toast(link.clone(), &requiem_i18n::t!("context-menus-link-copied"))
            },
        )
        .icon_position(IconPosition::Leading)
        .icon(icons::LINK),
    );
    items.push(
        ContextItem::unavailable(requiem_i18n::t!("context-menus-speak-message"))
            .icon_position(IconPosition::Leading)
            .icon(icons::CHAT_SPEAK),
    );
    if prefs::developer_mode() {
        items.push(copy_id_item(
            target.id.to_string(),
            requiem_i18n::t!("context-menus-copy-message-id"),
        ));
    }
    items.push(ContextItem::Separator);

    if !own {
        items.push(
            ContextItem::unavailable(requiem_i18n::t!("context-menus-report-message"))
                .icon_position(IconPosition::Leading)
                .icon(icons::FLAG)
                .danger(),
        );
    }
    if own || permissions.contains(Permissions::MANAGE_MESSAGES) {
        items.push(delete_item(env, target));
    }
    items
}

fn delete_item(env: &MenuEnv, target: &MessageTarget) -> ContextItem {
    let (env, channel_id, message_id) = (env.clone(), target.channel_id, target.id);

    ContextItem::action(
        requiem_i18n::t!("context-menus-delete-message"),
        move || {
            delete(
                &env,
                channel_id,
                message_id,
                app::shift_held(),
                true,
            )
        },
    )
    .icon_position(IconPosition::Leading)
    .icon(icons::TRASH)
    .danger()
}

pub fn delete(
    env: &MenuEnv,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    shift: bool,
    hint: bool,
) {
    if shift {
        delete_now(env, channel_id, message_id);
    } else {
        confirm_delete(env, channel_id, message_id, hint);
    }
}

pub fn delete_now(env: &MenuEnv, channel_id: Id<ChannelMarker>, message_id: Id<MessageMarker>) {
    let Some(http) = env.http.clone() else { return };
    run_with_toast(
        &env.queries,
        Vec::new(),
        requiem_i18n::t!("context-menus-message-deleted"),
        requiem_i18n::t!("context-menus-unable-to-delete-message"),
        move || async move {
            messages::delete_message(&http, channel_id, message_id).await
        },
    );
}

pub fn confirm_delete(
    env: &MenuEnv,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    hint: bool,
) {
    let env = env.clone();
    menu::confirm(ConfirmRequest {
        title: requiem_i18n::t!("context-menus-delete-message").to_string(),
        body: requiem_i18n::t!("context-menus-are-you-sure-you-want-to").to_string(),
        confirm_label: requiem_i18n::t!("context-menus-delete").to_string(),
        on_confirm: Rc::new(move || delete_now(&env, channel_id, message_id)),
        critical: true,
        content: Some(Rc::new(move || {
            message_confirm::preview(
                channel_id,
                message_id,
                hint.then(|| requiem_i18n::t!("context-menus-delete-protip")),
            )
        })),
    });
}

pub fn save_edit(
    env: &MenuEnv,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    original: &str,
    content: String,
) {
    message_edit::cancel();
    if content.trim().is_empty() {
        confirm_delete(env, channel_id, message_id, false);
        return;
    }
    if content == original {
        return;
    }

    let content = emoji_skin::strip_tones(&content).into_owned();
    let Some(http) = env.http.clone() else { return };
    let original = original.to_string();
    run_with_toast(
        &env.queries,
        Vec::new(),
        String::new(),
        requiem_i18n::t!("context-menus-unable-to-edit-message"),
        move || async move {
            messages::edit_message_content(
                &http, channel_id, message_id, &original, &content,
            )
            .await
        },
    );
}

pub fn attachment_items(
    env: &MenuEnv,
    attachment: &Attachment,
    target: &MessageTarget,
) -> Vec<ContextItem> {
    let mut items = Vec::new();
    let url = attachment
        .proxy_url
        .clone()
        .unwrap_or_else(|| attachment.url.clone());
    if attachment.is_image() {
        let (images, image_url) = (env.images.clone(), url.clone());
        items.push(ContextItem::action(
            requiem_i18n::t!("context-menus-copy-image"),
            move || {
                if images
                    .get_or_decode_image(&image_url)
                    .is_some_and(|image| media_viewer::copy_image(&image))
                {
                    menu::notify(&requiem_i18n::t!("context-menus-image-copied"));
                } else {
                    menu::notify_failure(&requiem_i18n::t!("context-menus-unable-to-copy-image"));
                }
            },
        ));
        if let Some(on_save) = env.on_save.clone() {
            let (images, image_url, filename) =
                (env.images.clone(), url.clone(), attachment.filename.clone());
            items.push(ContextItem::action(
                requiem_i18n::t!("context-menus-save-image"),
                move || match images.get_or_request(&image_url) {
                    Some(bytes) => on_save(bytes.to_vec(), filename.clone()),
                    None => menu::notify_failure(&requiem_i18n::t!(
                        "context-menus-unable-to-save-image"
                    )),
                },
            ));
        }
    }
    let link = attachment.url.clone();
    items.push(ContextItem::action(
        requiem_i18n::t!("context-menus-copy-link"),
        move || menu::copy_with_toast(link.clone(), &requiem_i18n::t!("context-menus-link-copied")),
    ));
    let link = attachment.url.clone();
    items.push(ContextItem::action(
        requiem_i18n::t!("context-menus-open-link"),
        move || link::open_url(&link),
    ));
    items.push(ContextItem::Separator);
    items.extend(message_items(env, target));
    items
}

pub fn file_actions(
    env: &MenuEnv,
    message: &models::Message,
    attachments: Option<&attachments::AttachmentsState>,
) -> message_embed::FileActions {
    let (images, on_save) = (env.images.clone(), env.on_save.clone());
    let download: Rc<dyn Fn(&Attachment)> = Rc::new(move |a| {
        message_embed::download_attachment(&images, on_save.clone(), a)
    });
    let Some(state) = attachments else {
        return message_embed::FileActions {
            download,
            edit: None,
            delete: None,
        };
    };

    let (channel_id, message_id) = (message.channel_id, message.id);

    let edit: Rc<dyn Fn(&Attachment)> = {
        let (state, queries, http, message) = (
            state.clone(),
            env.queries.clone(),
            env.http.clone(),
            message.clone(),
        );
        Rc::new(move |attachment| {
            let (queries, http, message, id) = (
                queries.clone(),
                http.clone(),
                message.clone(),
                attachment.id,
            );
            state.open_modify_sent(
                attachment,
                Rc::new(move |filename, description| {
                    let Some(http) = http.clone() else { return };
                    let patches: Vec<AttachmentPatch> = message
                        .attachments
                        .iter()
                        .map(|a| {
                            if a.id == id {
                                AttachmentPatch {
                                    id: a.id.to_string(),
                                    filename: filename.clone(),
                                    description: Some(description.clone()),
                                }
                            } else {
                                AttachmentPatch {
                                    id: a.id.to_string(),
                                    filename: a.filename.clone(),
                                    description: a.description.clone(),
                                }
                            }
                        })
                        .collect();
                    run_with_toast(
                        &queries,
                        Vec::new(),
                        requiem_i18n::t!("context-menus-attachment-updated"),
                        requiem_i18n::t!("context-menus-unable-to-edit-attachment"),
                        move || async move {
                            edit_message_attachments(&http, channel_id, message_id, &patches).await
                        },
                    );
                }),
            );
        })
    };

    let delete: Rc<dyn Fn(&Attachment)> = {
        let (queries, http, message) = (env.queries.clone(), env.http.clone(), message.clone());
        Rc::new(move |attachment| {
            let (queries, http, message, id) = (
                queries.clone(),
                http.clone(),
                message.clone(),
                attachment.id,
            );
            menu::confirm(ConfirmRequest {
                title: requiem_i18n::t!("context-menus-delete-attachment").to_string(),
                body: requiem_i18n::t!("context-menus-are-you-sure-you-want-to-2").to_string(),
                confirm_label: requiem_i18n::t!("context-menus-delete").to_string(),
                on_confirm: Rc::new(move || {
                    let Some(http) = http.clone() else { return };
                    let remaining: Vec<AttachmentPatch> = message
                        .attachments
                        .iter()
                        .filter(|a| a.id != id)
                        .map(patch_of_static)
                        .collect();
                    run_with_toast(
                        &queries,
                        Vec::new(),
                        requiem_i18n::t!("context-menus-attachment-deleted"),
                        requiem_i18n::t!("context-menus-unable-to-delete-attachment"),
                        move || async move {
                            edit_message_attachments(&http, channel_id, message_id, &remaining)
                                .await
                        },
                    );
                }),
                critical: true,
                content: None,
            });
        })
    };
    let deletable = !message.content.trim().is_empty();
    message_embed::FileActions {
        download,
        edit: Some(edit),
        delete: deletable.then_some(delete),
    }
}

fn patch_of_static(a: &Attachment) -> AttachmentPatch {
    AttachmentPatch {
        id: a.id.to_string(),
        filename: a.filename.clone(),
        description: a.description.clone(),
    }
}

pub struct UserTarget {
    pub user_id: Id<UserMarker>,
    pub guild_id: Option<Id<GuildMarker>>,
    pub display_name: String,

    pub webhook: bool,
}

pub fn voice_user_items(env: &MenuEnv, target: &UserTarget) -> Vec<ContextItem> {
    let mut items = user_items(env, target);
    let user_id = target.user_id;
    if env.own_user_id == Some(user_id) || !voice_users::in_my_call(&env.state, user_id)
    {
        return items;
    }
    let volume = SliderProps::default()
        .value(volume_to_slider(voice_control::user_volume(user_id)))
        .max(200.0_f32)
        .on_change(move |value: f32| {
            voice_control::set_user_volume(user_id, slider_to_volume(value, 200.0))
        })
        .build();
    let volume_row = div()
        .class(classes_voice_users::VOICE_MENU_VOLUME)
        .child(
            TextProps::styled(
                requiem_i18n::t!("voice-user-volume"),
                TextVariant::TEXT_SM_MEDIUM,
            )
            .color(ui::TEXT_SECONDARY)
            .build(),
        )
        .child(volume);
    let muted = voice_control::is_locally_muted(user_id);
    let mute = ContextItem::action(requiem_i18n::t!("voice-mute"), move || {
        voice_control::set_locally_muted(user_id, !muted)
    })
    .checkbox(muted);
    let at = items
        .iter()
        .position(|item| matches!(item, ContextItem::Separator))
        .unwrap_or(items.len());
    items.splice(
        at..at,
        [
            ContextItem::Separator,
            ContextItem::Custom(volume_row),
            mute,
        ],
    );
    items
}

pub fn user_items(env: &MenuEnv, target: &UserTarget) -> Vec<ContextItem> {
    if target.webhook {
        return if prefs::developer_mode() {
            vec![copy_id_item(
                target.user_id.to_string(),
                requiem_i18n::t!("context-menus-copy-user-id"),
            )]
        } else {
            Vec::new()
        };
    }
    let is_self = env.own_user_id == Some(target.user_id);
    let (user_id, guild_id) = (target.user_id, target.guild_id);
    let mut items = vec![
        ContextItem::action(requiem_i18n::t!("context-menus-profile"), move || {
            (menu::actions().open_profile)(user_id, guild_id, false)
        }),
        ContextItem::unavailable(requiem_i18n::t!("context-menus-mention")),
    ];

    if is_self {
        items.push(ContextItem::Separator);
        items.push(ContextItem::unavailable(requiem_i18n::t!(
            "context-menus-change-nickname"
        )));
        items.push(ContextItem::unavailable(requiem_i18n::t!("context-menus-apps")).caret());
    } else {
        items.push(ContextItem::unavailable(requiem_i18n::t!(
            "context-menus-message"
        )));
        items.push(ContextItem::unavailable(requiem_i18n::t!(
            "context-menus-start-a-call"
        )));
        items.push(
            ContextItem::action(requiem_i18n::t!("context-menus-add-note"), move || {
                (menu::actions().open_profile)(user_id, guild_id, true)
            })
            .subtitle(requiem_i18n::t!("context-menus-only-visible-to-you")),
        );
        items.push(ContextItem::unavailable(requiem_i18n::t!(
            "context-menus-add-friend-nickname"
        )));
        items.push(ContextItem::Separator);
        items.push(ContextItem::unavailable(requiem_i18n::t!("context-menus-apps")).caret());
        items.extend(relationship_items(env, target));
    }
    if target.guild_id.is_some() {
        items.push(ContextItem::Separator);
        items.push(ContextItem::unavailable(requiem_i18n::t!("context-menus-roles")).caret());
    }
    if prefs::developer_mode() {
        items.push(ContextItem::Separator);
        items.push(copy_id_item(
            user_id.to_string(),
            requiem_i18n::t!("context-menus-copy-user-id"),
        ));
    }
    items
}

fn copy_id_item(id: String, label: String) -> ContextItem {
    ContextItem::action(label, move || {
        menu::copy_with_toast(id.clone(), &requiem_i18n::t!("context-menus-id-copied"))
    })
    .icon_position(IconPosition::Leading)
    .icon(icons::ID)
}

#[derive(Clone, Copy)]
enum Relation {
    Add,
    Remove,
    Ignore,
}

async fn apply_relation(
    relation: Relation,
    http: HttpClient,
    user_id: Id<UserMarker>,
    menu: &'static str,
) -> Result<(), rest::RestError> {
    match relation {
        Relation::Add => {
            relationships::send_friend_request(
                &http,
                user_id,
                relationships::location::CONTEXT_MENU,
            )
            .await
        }
        Relation::Remove => {
            relationships::remove(&http, user_id, relationships::location::CONTEXT_MENU).await
        }
        Relation::Ignore => relationships::ignore(&http, user_id, menu).await,
    }
}

fn relation_item(
    env: &MenuEnv,
    user_id: Id<UserMarker>,
    menu: &'static str,
    label: &str,
    success: String,
    relation: Relation,
) -> ContextItem {
    let (queries, http) = (env.queries.clone(), env.http.clone());
    ContextItem::action(label, move || {
        let Some(http) = http.clone() else { return };
        run_with_toast(
            &queries,
            Vec::new(),
            success.clone(),
            requiem_i18n::t!("context-menus-something-went-wrong"),
            move || apply_relation(relation, http, user_id, menu),
        );
    })
}

fn relationship_items(env: &MenuEnv, target: &UserTarget) -> Vec<ContextItem> {
    let user_id = target.user_id;
    let kind = relationship_kind(&env.state.relationships, user_id);
    let menu_name = if target.guild_id.is_some() {
        relationships::location::GUILD_CHANNEL_USER_CONTEXT_MENU
    } else {
        relationships::location::DM_USER_CONTEXT_MENU
    };
    let mut items = Vec::new();
    match kind {
        Some(FRIEND) => items.push(relation_item(
            env,
            user_id,
            menu_name,
            &requiem_i18n::t!("context-menus-remove-friend"),
            requiem_i18n::t!("context-menus-friend-removed"),
            Relation::Remove,
        )),
        Some(PENDING_OUTGOING) => items.push(relation_item(
            env,
            user_id,
            menu_name,
            &requiem_i18n::t!("context-menus-cancel-friend-request"),
            requiem_i18n::t!("context-menus-friend-request-cancelled"),
            Relation::Remove,
        )),
        Some(BLOCKED) => {}
        _ => items.push(relation_item(
            env,
            user_id,
            menu_name,
            &requiem_i18n::t!("context-menus-add-friend"),
            requiem_i18n::t!("context-menus-friend-request-sent"),
            Relation::Add,
        )),
    }
    if kind == Some(BLOCKED) {
        items.push(
            relation_item(
                env,
                user_id,
                menu_name,
                &requiem_i18n::t!("context-menus-unblock"),
                requiem_i18n::t!("context-menus-user-unblocked"),
                Relation::Remove,
            )
            .danger(),
        );
    } else {
        items.push(relation_item(
            env,
            user_id,
            menu_name,
            &requiem_i18n::t!("context-menus-ignore"),
            requiem_i18n::t!("context-menus-user-ignored"),
            Relation::Ignore,
        ));
        let name = target.display_name.clone();
        items.push(
            ContextItem::action(requiem_i18n::t!("context-menus-block"), move || {
                (menu::actions().confirm_block)(user_id, name.clone(), menu_name)
            })
            .danger(),
        );
    }
    items
}

pub fn composer_items() -> Vec<ContextItem> {
    let command = |label: &str, command: EditorCommand| {
        ContextItem::action(label, move || (menu::actions().editor_command)(command))
    };
    vec![
        command(&requiem_i18n::t!("context-menus-undo"), EditorCommand::Undo),
        command(&requiem_i18n::t!("context-menus-redo"), EditorCommand::Redo),
        ContextItem::Separator,
        command(&requiem_i18n::t!("context-menus-cut"), EditorCommand::Cut),
        command(&requiem_i18n::t!("context-menus-copy"), EditorCommand::Copy),
        command(
            &requiem_i18n::t!("context-menus-paste"),
            EditorCommand::Paste,
        ),
        ContextItem::Separator,
        command(
            &requiem_i18n::t!("context-menus-select-all"),
            EditorCommand::SelectAll,
        ),
    ]
}

pub fn settings_items(avatar: node::Node) -> Vec<ContextItem> {
    let page = |key: &'static str, icon: Icon| {
        ContextItem::action(requiem_i18n::text(key), move || {
            (menu::actions().open_settings)(key)
        })
        .icon(icon)
        .icon_position(IconPosition::Leading)
    };
    vec![
        {
            ContextItem::action(requiem_i18n::t!("settings-modal-profiles"), || {
                (menu::actions().open_settings)(requiem_i18n::key!("settings-modal-profiles"))
            })
            .leading(avatar.class(classes_context_menu::LEADING_AVATAR))
        },
        ContextItem::Separator,
        page(requiem_i18n::key!("settings-modal-account"), icons::USER),
        page(requiem_i18n::key!("settings-modal-data-privacy"), SHIELD),
        page(
            requiem_i18n::key!("context-menus-messaging-permissions"),
            icons::CHAT_SHIELD,
        ),
        page(requiem_i18n::key!("settings-modal-notifications"), BELL),
        ContextItem::Separator,
        ContextItem::header(requiem_i18n::t!("context-menus-billing")),
        page(
            requiem_i18n::key!("settings-modal-nitro"),
            icons::NITRO_WHEEL,
        ),
        page(requiem_i18n::key!("settings-modal-server-boost"), BOOST_GEM),
        page(
            requiem_i18n::key!("settings-modal-subscriptions"),
            icons::SUBSCRIPTION,
        ),
        page(
            requiem_i18n::key!("settings-modal-gift-inventory"),
            icons::GIFT,
        ),
        page(
            requiem_i18n::key!("settings-modal-billing"),
            icons::CREDIT_CARD,
        ),
        ContextItem::Separator,
        ContextItem::header(requiem_i18n::t!("context-menus-experience")),
        page(
            requiem_i18n::key!("settings-modal-voice-video"),
            icons::VOICE_NORMAL,
        ),
        page(
            requiem_i18n::key!("settings-modal-appearance"),
            icons::PAINT_PALETTE,
        ),
        page(
            requiem_i18n::key!("settings-modal-accessibility"),
            icons::ACCESSIBILITY,
        ),
        page(requiem_i18n::key!("settings-modal-system"), icons::SCREEN),
        page(
            requiem_i18n::key!("settings-modal-language-time"),
            icons::LANGUAGE,
        ),
        ContextItem::Separator,
        ContextItem::header(requiem_i18n::t!("context-menus-games-apps")),
        page(
            requiem_i18n::key!("settings-modal-activity-privacy"),
            icons::GAME_CONTROLLER,
        ),
        page(requiem_i18n::key!("context-menus-connected-apps"), APPS),
        ContextItem::Separator,
        ContextItem::action(requiem_i18n::t!("context-menus-log-out"), || {
            (menu::actions().log_out)()
        })
        .icon(DOOR_EXIT)
        .icon_position(IconPosition::Leading)
        .danger(),
        ContextItem::Separator,
        ContextItem::action(requiem_i18n::t!("context-menus-playgrounds"), || {}).submenu(|| {
            playground::PLAYGROUNDS
                .iter()
                .map(|p| ContextItem::action(p.name, move || playground::open(p.name)))
                .collect()
        }),
    ]
}

pub fn channel_link(guild_id: Option<Id<GuildMarker>>, channel_id: Id<ChannelMarker>) -> String {
    let guild = guild_id.map_or("@me".to_string(), |g| g.to_string());
    format!("{}/channels/{guild}/{channel_id}", menu::web_base_url())
}

pub fn channel_items(
    state: &state::AppState,
    guild_id: Option<Id<GuildMarker>>,
    channel_id: Id<ChannelMarker>,
) -> Vec<ContextItem> {
    let link = channel_link(guild_id, channel_id);
    let thread_items = guild_id
        .and_then(|_| {
            ui::find_channel(state, channel_id)
                .map(|(_, channel)| channel)
                .filter(Channel::is_thread)
        })
        .map(|channel| threads::menu_items(state, &channel))
        .unwrap_or_default();
    let notifications = guild_id
        .map(|guild_id| {
            notification_settings::channel_items(state, guild_id, channel_id)
        })
        .unwrap_or_default();
    let mut items = vec![
        ContextItem::unavailable(requiem_i18n::t!("context-menus-mark-as-read")),
        ContextItem::Separator,
        match guild_id {
            Some(guild_id) => ContextItem::action(
                requiem_i18n::t!("context-menus-invite-to-channel"),
                move || invite_modal::open_channel(guild_id, channel_id),
            ),
            None => ContextItem::unavailable(requiem_i18n::t!("context-menus-invite-to-channel")),
        },
        ContextItem::unavailable(requiem_i18n::t!("context-menus-pin-channel-to-top")),
        ContextItem::action(requiem_i18n::t!("context-menus-copy-link"), move || {
            menu::copy_with_toast(link.clone(), &requiem_i18n::t!("context-menus-link-copied"))
        }),
        ContextItem::Separator,
    ];
    items.extend(notifications);
    items.extend(thread_items);
    items.extend([
        ContextItem::Separator,
        ContextItem::unavailable(requiem_i18n::t!("context-menus-edit-channel")),
        ContextItem::unavailable(requiem_i18n::t!("context-menus-duplicate-channel")),
        ContextItem::unavailable(requiem_i18n::t!("context-menus-create-text-channel")),
        ContextItem::unavailable(requiem_i18n::t!("context-menus-delete-channel")).danger(),
    ]);
    items
}

pub fn category_items(
    state: &state::AppState,
    guild_id: Id<GuildMarker>,
    category_id: Id<ChannelMarker>,
) -> Vec<ContextItem> {
    let mut items = vec![
        ContextItem::unavailable(requiem_i18n::t!("context-menus-mark-as-read")),
        ContextItem::Separator,
        ContextItem::unavailable(requiem_i18n::t!("context-menus-collapse-category"))
            .checkbox(false),
        ContextItem::unavailable(requiem_i18n::t!("context-menus-collapse-all-categories")),
        ContextItem::Separator,
    ];
    items.extend(notification_settings::channel_items(
        state,
        guild_id,
        category_id,
    ));
    items.extend([
        ContextItem::Separator,
        ContextItem::unavailable(requiem_i18n::t!("context-menus-edit-category")),
        ContextItem::unavailable(requiem_i18n::t!("context-menus-delete-category")).danger(),
    ]);
    items
}

fn plugin_guild_items(
    state: &state::AppState,
    guild_id: Id<GuildMarker>,
    header: bool,
) -> Vec<ContextItem> {
    let registrations = state.plugins.registrations.with_untracked(|r| r.clone());

    let context = serde_json::json!({ "guild-id": guild_id.get().to_string() }).to_string();
    let guild_filter = format!("guild:{guild_id}");
    let mut items = Vec::new();
    for (plugin, registered) in registrations.iter() {
        let pick = |run: Box<dyn Fn(&str)>| {
            let (state, context) = (state.clone(), context.clone());
            move || {
                state::refresh_plugin_guild(&state, guild_id);
                run(&context);
            }
        };
        if header {
            for item in registered.slot_items.values().filter(|i| {
                i.slot == "guild-header.menu"
                    && !i.hidden
                    && (i.visible_in.is_empty() || i.visible_in.contains(&guild_filter))
            }) {
                let (plugin, slot, id) = (plugin.clone(), item.slot.clone(), item.id.clone());
                let mut entry = ContextItem::action(
                    item.label.clone(),
                    pick(Box::new(move |context| {
                        plugins::slot_click(&plugin, &slot, &id, context.to_string())
                    })),
                )
                .icon_position(IconPosition::Trailing);
                if let Some(icon) = &item.icon {
                    entry = entry.icon(plugin_view::named_icon(icon));
                }
                if item.danger {
                    entry = entry.danger();
                }
                items.push(if item.disabled {
                    entry.disabled()
                } else {
                    entry
                });
            }
        } else {
            for item in registered
                .menu_items
                .iter()
                .filter(|i| i.menu == "guild" && i.parent.is_none() && !i.dynamic)
            {
                let (plugin, id) = (plugin.clone(), item.id.clone());
                let mut entry = ContextItem::action(
                    item.label.clone(),
                    pick(Box::new(move |context| {
                        plugins::menu_item_click(&plugin, "guild", &id, context.to_string())
                    })),
                );
                if let Some(icon) = &item.icon {
                    entry = entry
                        .icon(plugin_view::named_icon(icon))
                        .icon_position(IconPosition::Trailing);
                }
                if item.danger {
                    entry = entry.danger();
                }
                items.push(entry);
            }
        }
    }
    items
}

/// The menu on a server folder in the rail.
pub fn folder_items(
    state: &state::AppState,
    guilds: &[Id<GuildMarker>],
) -> Vec<ContextItem> {
    let unread = guilds
        .iter()
        .any(|id| state.read_states.guild_unread(*id).with_untracked(|u| *u));
    let mark_read = requiem_i18n::t!("context-menus-mark-folder-as-read");
    let mark_read = if unread {
        let (state, guilds) = (state.clone(), guilds.to_vec());
        ContextItem::action(mark_read, move || {
            guilds
                .iter()
                .for_each(|id| state.read_states.mark_guild_read(*id))
        })
    } else {
        ContextItem::unavailable(mark_read)
    };
    let folders = state.folders.clone();
    vec![
        mark_read,
        ContextItem::Separator,
        ContextItem::unavailable(requiem_i18n::t!("context-menus-folder-settings")),
        ContextItem::action(
            requiem_i18n::t!("context-menus-close-all-folders"),
            move || folders.collapse_all(),
        ),
    ]
}

pub fn guild_items(state: &state::AppState, guild_id: Id<GuildMarker>) -> Vec<ContextItem> {
    let can_manage = state.guilds.find(guild_id).is_some_and(|guild| {
        state
            .permissions
            .guild_permissions(&guild)
            .contains(Permissions::MANAGE_CHANNELS)
    });
    let mut items = vec![
        ContextItem::unavailable(requiem_i18n::t!("context-menus-mark-as-read")),
        ContextItem::Separator,
        ContextItem::action(
            requiem_i18n::t!("context-menus-invite-to-server"),
            move || {
                invite_modal::open(
                    guild_id,
                    location::CONTEXT_MENU,
                )
            },
        ),
        ContextItem::Separator,
    ];
    items.extend(notification_settings::guild_items(
        state, guild_id,
    ));
    items.extend([
        ContextItem::unavailable(requiem_i18n::t!("context-menus-hide-muted-channels"))
            .checkbox(false),
        ContextItem::unavailable(requiem_i18n::t!("context-menus-show-all-channels"))
            .checkbox(true),
        ContextItem::Separator,
        ContextItem::unavailable(requiem_i18n::t!("context-menus-server-settings")).caret(),
        ContextItem::unavailable(requiem_i18n::t!("context-menus-privacy-settings")),
        ContextItem::unavailable(requiem_i18n::t!("context-menus-edit-per-server-profile")),
        ContextItem::Separator,
        ContextItem::unavailable(requiem_i18n::t!("context-menus-create-event")),
        ContextItem::Separator,
        ContextItem::unavailable(requiem_i18n::t!("context-menus-leave-server")).danger(),
    ]);

    if can_manage {
        let at = items.iter().position(|item| matches!(item, ContextItem::Action(a) if a.label == requiem_i18n::t!("context-menus-create-event"))).unwrap_or(items.len());
        items.splice(
            at..at,
            [
                ContextItem::action(
                    requiem_i18n::t!("context-menus-create-channel"),
                    move || create_channel::open(guild_id, None),
                ),
                ContextItem::action(
                    requiem_i18n::t!("context-menus-create-category"),
                    move || create_channel::open_category(guild_id),
                ),
            ],
        );
    }

    let plugin = plugin_guild_items(state, guild_id, false);
    if !plugin.is_empty() {
        let at = items.len() - 1;
        let mut group = plugin;
        group.push(ContextItem::Separator);
        items.splice(at..at, group);
    }
    items
}

pub fn channel_list_items(
    state: &state::AppState,
    guild_id: Id<GuildMarker>,
) -> Vec<ContextItem> {
    let Some(guild) = state.guilds.find(guild_id) else {
        return Vec::new();
    };
    let mut items = vec![
        ContextItem::unavailable(requiem_i18n::t!("context-menus-hide-muted-channels"))
            .checkbox(false),
        ContextItem::Separator,
    ];
    if state
        .permissions
        .guild_permissions(&guild)
        .contains(Permissions::MANAGE_CHANNELS)
    {
        items.push(ContextItem::action(
            requiem_i18n::t!("context-menus-create-channel"),
            move || create_channel::open(guild_id, None),
        ));
        items.push(ContextItem::action(
            requiem_i18n::t!("context-menus-create-category"),
            move || create_channel::open_category(guild_id),
        ));
    }
    items.push(ContextItem::action(
        requiem_i18n::t!("context-menus-invite-to-server"),
        move || {
            invite_modal::open(guild_id, location::CONTEXT_MENU)
        },
    ));
    items
}

pub fn guild_header_items(
    state: &state::AppState,
    queries: &QueryClient,
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
) -> Vec<ContextItem> {
    let Some(guild) = state.guilds.find(guild_id) else {
        return Vec::new();
    };
    let bits = state.permissions.guild_permissions(&guild);
    let has = |flag: Permissions| bits.contains(flag);
    let lead =
        |item: ContextItem, icon: Icon| item.icon_position(IconPosition::Trailing).icon(icon);

    let mut groups: Vec<Vec<ContextItem>> = Vec::new();
    groups.push(vec![lead(
        ContextItem::unavailable(requiem_i18n::t!("context-menus-server-boost")),
        BOOST_GEM,
    )]);

    let mut manage = Vec::new();
    manage.push(lead(
        ContextItem::action(
            requiem_i18n::t!("context-menus-invite-to-server"),
            move || {
                invite_modal::open(guild_id, location::HEADER)
            },
        ),
        GROUP_PLUS,
    ));
    if guild_settings::can_access(state, &guild) {
        let state = state.clone();
        manage.push(lead(
            ContextItem::action(
                requiem_i18n::t!("context-menus-server-settings"),
                move || {
                    guild_settings::open(
                        &state,
                        guild_id,
                        guild_settings::SERVER_PROFILE_PAGE,
                    )
                },
            ),
            SETTINGS,
        ));
    }
    if has(Permissions::MANAGE_CHANNELS) {
        manage.push(lead(
            ContextItem::action(
                requiem_i18n::t!("context-menus-create-channel"),
                move || create_channel::open(guild_id, None),
            ),
            CIRCLE_PLUS,
        ));
        manage.push(lead(
            ContextItem::action(
                requiem_i18n::t!("context-menus-create-category"),
                move || create_channel::open_category(guild_id),
            ),
            FOLDER_PLUS,
        ));
    }
    if has(Permissions::CREATE_EVENTS) || has(Permissions::MANAGE_EVENTS) {
        manage.push(lead(
            ContextItem::unavailable(requiem_i18n::t!("context-menus-create-event")),
            CALENDAR_PLUS,
        ));
    }
    if has(Permissions::MANAGE_GUILD) {
        manage.push(lead(
            ContextItem::unavailable(requiem_i18n::t!("context-menus-app-directory")),
            APPS,
        ));
    }
    groups.push(manage);

    groups.push(vec![
        ContextItem::unavailable(requiem_i18n::t!("context-menus-show-all-channels"))
            .checkbox(false),
        lead(
            ContextItem::action(
                requiem_i18n::t!("context-menus-notification-settings"),
                move || notification_settings::open(guild_id),
            ),
            BELL,
        ),
        lead(
            ContextItem::unavailable(requiem_i18n::t!("context-menus-privacy-settings")),
            SHIELD,
        ),
    ]);
    groups.push(vec![
        ContextItem::unavailable(requiem_i18n::t!("context-menus-hide-muted-channels"))
            .checkbox(false),
    ]);
    groups.push(plugin_guild_items(state, guild_id, true));

    if state
        .users
        .current
        .with_untracked(|u| u.as_ref().map(|u| u.id))
        != guild.owner_id
    {
        let (queries, http, name) = (queries.clone(), http.clone(), guild.name.clone());
        groups.push(vec![lead(
            ContextItem::action(requiem_i18n::t!("context-menus-leave-server"), move || {
                confirm_leave(&queries, &http, guild_id, &name)
            })
            .danger(),
            DOOR_EXIT,
        )]);
    }

    let mut items = Vec::new();
    for group in groups.into_iter().filter(|g| !g.is_empty()) {
        if !items.is_empty() {
            items.push(ContextItem::Separator);
        }
        items.extend(group);
    }
    items
}

fn confirm_leave(queries: &QueryClient, http: &HttpClient, guild_id: Id<GuildMarker>, name: &str) {
    let (queries, http) = (queries.clone(), http.clone());
    menu::confirm(ConfirmRequest {
        title: requiem_i18n::t!("context-menus-leave-server-title", name = name),
        body: requiem_i18n::t!("context-menus-leave-server-body", name = name),
        confirm_label: requiem_i18n::t!("context-menus-leave-server").to_string(),
        on_confirm: Rc::new(move || {
            let http = http.clone();
            run_with_toast(
                &queries,
                Vec::new(),
                String::new(),
                requiem_i18n::t!("context-menus-unable-to-leave-the-server"),
                move || async move {
                    guilds::leave_guild(&http, guild_id).await
                },
            );
        }),
        critical: true,
        content: None,
    });
}
