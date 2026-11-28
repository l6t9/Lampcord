use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use requiem_ui::components::{
    AvatarStatusSize, ButtonProps, ButtonVariant, ModalProps, ModalSize, TextInputProps,
    TextProps, TextVariant, avatar_with_status, checkbox_indicator, div, icon,
};
use requiem_ui::control::section;
use requiem_ui::node::Node;
use requiem_ui::signal::{Effect, Signal, signal};
use requiem_ui::timer;
use requiem_ui::widget::anim::ToggleState;
use requiem_ui::widget::{WidgetId, WidgetTree};
use skia_safe::Color;

use crate::models::Permissions;
use crate::models::id::marker::{ChannelMarker, GuildMarker};
use crate::models::{self, Id};
use crate::rest::endpoints::messages::{self, ForwardSource};
use crate::rest::RestError;
use crate::ui::classes::forward as cls;
use crate::ui::context_menu::{notify, notify_failure};
use crate::ui::dialog::Dialog;
use crate::ui::home::{HomeEnv, dm_title};
use crate::ui::{self, TEXT_SECONDARY, TEXT_STRONG, TEXT_SUBTLE};

const MAX_TARGETS: usize = 10;
const MAX_RESULTS: usize = 120;

thread_local! {
    static DIALOG: Dialog = Dialog::new();
    static SOURCE: RefCell<Option<models::Message>> = const { RefCell::new(None) };
    static QUERY: Signal<String> = signal(String::new());
    static PICKED: Signal<Vec<Id<ChannelMarker>>> = signal(Vec::new());
    static COMMENT: Signal<String> = signal(String::new());
    static ERROR: Signal<Option<String>> = signal(None);
    static SENDING: Signal<bool> = signal(false);
}

#[derive(Clone)]
struct Destination {
    id: Id<ChannelMarker>,
    name: String,
    detail: String,
    user: Option<models::User>,
    channel_guild: Option<Id<GuildMarker>>,
}

pub fn is_open() -> bool {
    DIALOG.with(Dialog::is_open)
}

pub fn toggle() -> ToggleState {
    DIALOG.with(Dialog::toggle)
}

pub fn open(message: models::Message) {
    SOURCE.with(|source| *source.borrow_mut() = Some(message));
    QUERY.with(|query| query.set(String::new()));
    PICKED.with(|picked| picked.set(Vec::new()));
    COMMENT.with(|comment| comment.set(String::new()));
    ERROR.with(|error| error.set(None));
    SENDING.with(|sending| sending.set(false));
    DIALOG.with(Dialog::open);
}

pub fn close() {
    DIALOG.with(Dialog::close);
}

fn toggle_picked(channel_id: Id<ChannelMarker>) {
    PICKED.with(|picked| {
        picked.update(|list| {
            if let Some(index) = list.iter().position(|id| *id == channel_id) {
                list.remove(index);
            } else if list.len() < MAX_TARGETS {
                list.push(channel_id);
            }
        })
    });
}

fn source_preview(message: &models::Message) -> String {
    message
        .forward_snapshot()
        .map(|snapshot| snapshot.preview_text())
        .filter(|preview| !preview.trim().is_empty())
        .unwrap_or_else(|| message.content.clone())
}

fn destinations(env: &HomeEnv, query: &str) -> Vec<Destination> {
    let _ = env.tick.get();
    let needle = query.trim().to_lowercase();
    let matches = |name: &str| needle.is_empty() || name.to_lowercase().contains(&needle);
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    for dm in env.state.dms.list.with_untracked(|list| list.clone()) {
        if !seen.insert(dm.id) {
            continue;
        }
        let name = dm_title(&env.state, &dm);
        if !matches(&name) {
            continue;
        }
        let user = dm
            .recipients
            .first()
            .copied()
            .and_then(|id| env.state.users.get(id));
        out.push(Destination {
            id: dm.id,
            name,
            detail: requiem_i18n::t!("forward-dialog-direct-message").to_string(),
            user,
            channel_guild: None,
        });
    }

    let guilds = env.state.guilds.list.with_untracked(|g| g.clone());
    env.state.channels.by_guild.with_untracked(|by_guild| {
        for guild in &guilds {
            let Some(channels) = by_guild.get(&guild.id) else {
                continue;
            };
            let mut channels: Vec<_> = channels
                .iter()
                .filter(|channel| {
                    channel.kind == 0
                        && env.state.can(channel, Permissions::VIEW_CHANNEL)
                        && env.state.can(channel, Permissions::SEND_MESSAGES)
                })
                .collect();
            channels.sort_by_key(|channel| channel.position.unwrap_or(0));
            for channel in channels {
                if !seen.insert(channel.id) {
                    continue;
                }
                let name = channel.name.clone().unwrap_or_default();
                if !matches(&name) && !matches(&guild.name) {
                    continue;
                }
                out.push(Destination {
                    id: channel.id,
                    name,
                    detail: guild.name.clone(),
                    user: None,
                    channel_guild: Some(guild.id),
                });
            }
        }
    });

    out.truncate(MAX_RESULTS);
    out
}

fn send(env: &HomeEnv) {
    let picked = PICKED.with(|picked| picked.peek());
    let comment = COMMENT.with(|comment| comment.peek());
    let Some(source_message) = SOURCE.with(|source| source.borrow().clone()) else {
        return;
    };
    if picked.is_empty() || SENDING.with(Signal::get) {
        return;
    }

    ERROR.with(|error| error.set(None));
    SENDING.with(|sending| sending.set(true));

    let source = ForwardSource {
        message_id: source_message.id,
        channel_id: source_message.channel_id,
        guild_id: source_message.guild_id,
    };
    let http = env.http.clone();
    let queries = env.queries.clone();
    let outcome: Arc<Mutex<Option<Option<String>>>> = Arc::new(Mutex::new(None));
    let outcome_for_task = outcome.clone();
    queries.mutate(Vec::new(), move || {
        let http = http.clone();
        let picked = picked.clone();
        let comment = comment.clone();
        let outcome = outcome_for_task.clone();
        async move {
            let result: Result<(), RestError> = async {
                for channel_id in picked {
                    messages::send_forward(&http, channel_id, source).await?;
                    if !comment.trim().is_empty() {
                        messages::send_message(&http, channel_id, comment.trim()).await?;
                    }
                }
                Ok(())
            }
            .await;
            *outcome.lock().expect("forward outcome poisoned") = Some(result.as_ref().err().map(ToString::to_string));
            result
        }
    });
    poll_outcome(outcome, 0);
}

fn poll_outcome(outcome: Arc<Mutex<Option<Option<String>>>>, attempt: u32) {
    timer::set_timeout(Duration::from_millis(50), move || {
        let done = outcome.lock().expect("forward outcome poisoned").take();
        match done {
            Some(None) => {
                SENDING.with(|sending| sending.set(false));
                close();
                notify(&requiem_i18n::t!("forward-dialog-sent"));
            }
            Some(Some(error)) => {
                SENDING.with(|sending| sending.set(false));
                ERROR.with(|state| state.set(Some(error)));
                notify_failure(&requiem_i18n::t!("forward-dialog-failed"));
            }
            None if attempt < 400 => poll_outcome(outcome, attempt + 1),
            None => SENDING.with(|sending| sending.set(false)),
        }
    });
}

fn list(env: &HomeEnv) -> Node {
    let query = QUERY.with(Signal::get);
    let picked = PICKED.with(Signal::get);
    let entries = destinations(env, &query);
    let mut scroll = requiem_ui::components::scroll_view().class(cls::LIST);
    if entries.is_empty() {
        return scroll.child(
            TextProps::styled(
                requiem_i18n::t!("forward-dialog-no-results").to_string(),
                TextVariant::TEXT_SM_NORMAL,
            )
            .color(TEXT_SUBTLE)
            .build()
            .class(cls::NO_RESULTS),
        );
    }
    for entry in entries {
        let selected = picked.contains(&entry.id);
        scroll = scroll.child(destination_row(env, &entry, selected));
    }
    scroll
}

fn destination_row(env: &HomeEnv, entry: &Destination, selected: bool) -> Node {
    let lead = if let Some(user) = &entry.user {
        let mut props = ui::user_avatar_props(user.id, &user.discriminator, &env.fonts);
        if let Some(image) = user.avatar.as_deref().and_then(|hash| {
            env.images
                .get_or_decode_image(&env.backend.avatar_url(user.id, hash))
        }) {
            props = props.image(image);
        }
        avatar_with_status(props, AvatarStatusSize::Px32, None).class(cls::AVATAR)
    } else if let Some(guild_id) = entry.channel_guild {
        let channel = env
            .state
            .channels
            .by_guild
            .with_untracked(|by_guild| {
                by_guild
                    .get(&guild_id)
                    .and_then(|channels| channels.iter().find(|channel| channel.id == entry.id))
                    .cloned()
            });
        icon(
            channel.map_or(requiem_ui::icons::TEXT, |channel| {
                ui::channel_icon(&env.state, &channel)
            }),
            TEXT_SECONDARY,
        )
        .class(cls::CHANNEL_ICON)
    } else {
        icon(requiem_ui::icons::TEXT, TEXT_SECONDARY).class(cls::CHANNEL_ICON)
    };

    let texts = div().class(cls::ROW_TEXTS).children([
        TextProps::styled(entry.name.clone(), TextVariant::TEXT_MD_SEMIBOLD)
            .color(TEXT_STRONG)
            .line_clamp(1_u32)
            .build(),
        TextProps::styled(entry.detail.clone(), TextVariant::TEXT_XS_NORMAL)
            .color(TEXT_SUBTLE)
            .line_clamp(1_u32)
            .build(),
    ]);

    let channel_id = entry.id;
    let mut row = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::ROW.to_string()])
        .on_click(move || toggle_picked(channel_id))
        .build()
        .children([
            lead,
            texts,
            div().class(cls::SPACER),
            checkbox_indicator(ToggleState::new(selected)),
        ]);
    if selected {
        row = row.class(cls::ROW_SELECTED);
    }
    row
}

fn dialog(env: &HomeEnv) -> Node {
    let list_env = env.clone();
    let list = Node::reactive(move || list(&list_env)).class(cls::LIST_SLOT);

    let search = TextInputProps::new()
        .placeholder(requiem_i18n::t!("forward-dialog-search"))
        .autofocus(true)
        .on_change(|text: &str| QUERY.with(|query| query.set(text.to_string())))
        .build();

    let comment = TextInputProps::new()
        .placeholder(requiem_i18n::t!("forward-dialog-comment"))
        .multiline(true)
        .rows(2_u32)
        .on_change(|text: &str| COMMENT.with(|value| value.set(text.to_string())))
        .build();

    let source = SOURCE.with(|source| source.borrow().clone());
    let (author, preview) = source
        .as_ref()
        .map(|message| {
            (
                message.author.display_name(None),
                source_preview(message),
            )
        })
        .unwrap_or_default();
    let preview_node = div()
        .class(cls::PREVIEW)
        .child(
            TextProps::styled(author, TextVariant::TEXT_XS_SEMIBOLD)
                .color(TEXT_STRONG)
                .build(),
        )
        .child(
            TextProps::styled(preview, TextVariant::TEXT_XS_NORMAL)
                .color(TEXT_SUBTLE)
                .line_clamp(2_u32)
                .build()
                .class(cls::PREVIEW_TEXT),
        );

    let error = Node::reactive(move || {
        ERROR.with(Signal::get).map_or_else(div, |message| {
            TextProps::styled(message, TextVariant::TEXT_XS_NORMAL)
                .color(Color::from_rgb(0xf2, 0x3f, 0x42))
                .build()
                .class(cls::ERROR)
        })
    });

    let body = div().class(cls::BODY).children([
        search,
        preview_node,
        list,
        comment,
        error,
    ]);

    let send_env = env.clone();
    let send_button = Node::reactive(move || {
        let count = PICKED.with(Signal::get).len();
        let sending = SENDING.with(Signal::get);
        let env = send_env.clone();
        ButtonProps::default()
            .variant(ButtonVariant::Primary)
            .label(
                if count == 0 {
                    requiem_i18n::t!("forward-dialog-send").to_string()
                } else {
                    requiem_i18n::t!("forward-dialog-send-count", count = count).to_string()
                },
            )
            .disabled(count == 0 || sending)
            .on_click(move || send(&env))
            .build()
    });
    let cancel = ButtonProps::default()
        .variant(ButtonVariant::Secondary)
        .label(requiem_i18n::t!("forward-dialog-cancel").to_string())
        .on_click(close)
        .build();

    ModalProps::default()
        .size(ModalSize::Medium)
        .heading(requiem_i18n::t!("forward-dialog-title"))
        .subtitle(requiem_i18n::t!("forward-dialog-subtitle").to_string())
        .dismissable(true)
        .on_dismiss(close)
        .open(DIALOG.with(Dialog::toggle))
        .body(body)
        .footer(vec![cancel, send_button])
        .panel_class(cls::PANEL.to_string())
        .build()
}

pub fn mount(tree: &mut WidgetTree, parent: WidgetId, env: Rc<dyn Fn() -> HomeEnv>) -> Effect {
    section(tree, parent, move || {
        if DIALOG.with(Dialog::is_mounted) {
            dialog(&env())
        } else {
            div()
        }
    })
}
