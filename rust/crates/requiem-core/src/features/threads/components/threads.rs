use crate::ui::classes::pins as pins_cls;
use crate::ui::classes::threads as cls;
use crate::models::id;
use crate::models::message::message_flags;
use crate::models;
use crate::rest::endpoints::messages;
use crate::rest::endpoints::threads;
use crate::rest::endpoints;
use crate::state;
use crate::ui::author_style;
use crate::ui::classes::pins;
use crate::ui::context_menu;
use crate::ui::context_menus;
use crate::ui::guild_settings;
use crate::ui::profile_popout;
use crate::ui::thread_browser;
use crate::ui::twemoji;
use crate::ui;
use crate::utils::time;
use requiem_ui::colors;
use requiem_ui::components::MessageTextProps;
use requiem_ui::components::message_text;
use requiem_ui::components;
use requiem_ui::widget::AbsoluteRect;
use std::cell::RefCell;
use std::rc::Rc;

use requiem_ui::components::{
    ButtonProps, ButtonSize, ButtonVariant, ContextItem, FormItemProps, SelectOption, SelectProps,
    StackProps, SwitchProps, TextInputProps, TextProps, TextVariant, div, icon,
};
use requiem_ui::control::section;
use requiem_ui::node::Node;
use requiem_ui::signal::{Effect, Signal, signal};
use requiem_ui::widget::anim::ToggleMemo;
use requiem_ui::widget::{WidgetId, WidgetTree};
use serde_json::json;

use crate::models::id::marker::{ChannelMarker, GuildMarker, MessageMarker};
use crate::models::{Channel, Id, Permissions};
use crate::ui::context_menu as menu;
use crate::ui::home::HomeEnv;
use requiem_ui::icons::{THREAD, X_SMALL};

const ARCHIVE_OPTIONS: [(u32, &str); 4] = [
    (60, "threads-1-hour"),
    (1440, "threads-24-hours"),
    (4320, "threads-3-days"),
    (10080, "threads-7-days"),
];

thread_local! {
    static OPEN: Signal<bool> = signal(false);
    static ENV: RefCell<Option<Rc<dyn Fn() -> HomeEnv>>> = const { RefCell::new(None) };
    static TARGET: RefCell<Option<ThreadTarget>> = const { RefCell::new(None) };
    static NAME: Signal<String> = signal(String::new());
    static MESSAGE: Signal<String> = signal(String::new());
    static PRIVATE: Signal<bool> = signal(false);
    static ARCHIVE: Signal<u32> = signal(1440);
    static SELECT_OPEN: Signal<bool> = signal(false);
    static SWITCH: RefCell<ToggleMemo> = RefCell::new(ToggleMemo::new(false));
}

#[derive(Clone, Copy)]
struct ThreadTarget {
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
    message_id: Option<Id<MessageMarker>>,
}

pub fn mount(tree: &mut WidgetTree, parent: WidgetId, env: Rc<dyn Fn() -> HomeEnv>) -> Effect {
    ENV.with(|slot| *slot.borrow_mut() = Some(env.clone()));
    section(tree, parent, move || {
        thread_browser::layer(&env())
    })
}

pub fn env() -> Option<HomeEnv> {
    ENV.with(|factory| factory.borrow().clone())
        .map(|factory| factory())
}

pub fn sidebar() -> Option<Node> {
    OPEN.with(Signal::get).then(create_sidebar)
}

pub fn open_create(
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
    message_id: Option<Id<MessageMarker>>,
) {
    TARGET.with(|target| {
        *target.borrow_mut() = Some(ThreadTarget {
            guild_id,
            channel_id,
            message_id,
        })
    });
    NAME.with(|name| name.set(String::new()));
    MESSAGE.with(|message| message.set(String::new()));
    PRIVATE.with(|private| private.set(false));
    ARCHIVE.with(|archive| archive.set(1440));
    SELECT_OPEN.with(|open| open.set(false));
    SWITCH.with(|switch| switch.borrow().reset(false));
    OPEN.with(|open| open.set(true));
}

fn close_create() {
    SELECT_OPEN.with(|open| open.set(false));
    OPEN.with(|open| open.set(false));
}

fn create() {
    let Some(target) = TARGET.with(|target| *target.borrow()) else {
        return;
    };
    let Some(env) = env() else { return };
    let name = NAME.with(|name| name.peek()).trim().to_string();
    if name.is_empty() {
        return;
    }
    let private = PRIVATE.with(|private| private.peek());
    let archive = ARCHIVE.with(|archive| archive.peek());
    let first_message = MESSAGE.with(|message| message.peek()).trim().to_string();
    let (http, state, select) = (
        env.http.clone(),
        env.state.clone(),
        env.on_select_channel.clone(),
    );
    guild_settings::run(
        &env.queries,
        move || async move {
            let mut thread = if let Some(message_id) = target.message_id {
                threads::create_from_message(
                    &http,
                    target.channel_id,
                    message_id,
                    &name,
                    archive,
                )
                .await?
            } else {
                threads::create(
                    &http,
                    target.channel_id,
                    &name,
                    if private { 12 } else { 11 },
                    archive,
                    true,
                )
                .await?
            };
            if !first_message.is_empty() {
                messages::send_message(&http, thread.id, &first_message)
                    .await
                    .map_err(|error| endpoints::error_message(&error))?;
            }
            thread.guild_id.get_or_insert(target.guild_id);
            Ok(thread)
        },
        move |result| {
            close_create();
            match result {
                Ok(mut thread) => {
                    thread.guild_id.get_or_insert(target.guild_id);
                    let id = thread.id;
                    state.channels.upsert(thread, false);
                    select(id);
                }
                Err(error) => menu::notify_failure(&error),
            }
        },
    );
}

fn create_sidebar() -> Node {
    let Some(target) = TARGET.with(|target| *target.borrow()) else {
        return div();
    };
    let env = env().unwrap();
    let private_allowed = env.state.guilds.find(target.guild_id).is_some_and(|guild| {
        env.state
            .permissions
            .guild_permissions(&guild)
            .contains(Permissions::CREATE_PRIVATE_THREADS)
    });
    let name = TextInputProps::new()
        .text(NAME.with(|name| name.peek()))
        .placeholder(requiem_i18n::t!("threads-name-placeholder"))
        .max_length(100usize)
        .autofocus(true)
        .on_change(|text: &str| NAME.with(|name| name.set(text.to_string())))
        .build();
    let archive = SelectProps::new(
        || SELECT_OPEN.with(|open| open.set(!open.peek())),
        |value: String| {
            SELECT_OPEN.with(|open| open.set(false));
            if let Ok(minutes) = value.parse() {
                ARCHIVE.with(|archive| archive.set(minutes));
            }
        },
    )
    .options(
        ARCHIVE_OPTIONS
            .iter()
            .map(|&(minutes, key)| SelectOption::new(minutes.to_string(), requiem_i18n::t!(key)))
            .collect::<Vec<_>>(),
    )
    .selected(ARCHIVE.with(|archive| archive.peek()).to_string())
    .open(SELECT_OPEN.with(|open| open.peek()))
    .build();
    let mut settings = StackProps::vertical()
        .gap(16)
        .full_width(true)
        .build()
        .children([
            FormItemProps::new(requiem_i18n::t!("threads-name"), name).build(),
            FormItemProps::new(requiem_i18n::t!("threads-auto-archive"), archive)
                .description(requiem_i18n::t!("threads-auto-archive-description"))
                .build(),
        ]);
    if private_allowed && target.message_id.is_none() {
        let switch = SwitchProps::new(
            SWITCH.with(|switch| switch.borrow().track(PRIVATE.with(|private| private.get()))),
            |on: bool| PRIVATE.with(|private| private.set(on)),
        )
        .build();
        settings = settings.child(
            FormItemProps::new(requiem_i18n::t!("threads-private"), switch)
                .description(requiem_i18n::t!("threads-private-description"))
                .horizontal(true)
                .build(),
        );
    }
    let message = TextInputProps::new()
        .text(MESSAGE.with(|message| message.peek()))
        .placeholder(requiem_i18n::t!("threads-first-message-placeholder"))
        .multiline(true)
        .max_length(2000usize)
        .on_change(|text: &str| MESSAGE.with(|message| message.set(text.to_string())))
        .build()
        .class(cls::FIRST_MESSAGE);
    let composer = div().class(cls::COMPOSER).children([
        message,
        ButtonProps::new(
            ButtonVariant::Primary,
            requiem_i18n::t!("threads-create-action"),
            create,
        )
        .size(ButtonSize::Medium)
        .full_width(true)
        .build(),
    ]);

    div().class(cls::SIDEBAR).children([
        div().class(cls::HEADER).children([
            icon(THREAD, colors::TEXT_MUTED).style("width: 20px; height: 20px;"),
            TextProps::styled(
                requiem_i18n::t!("threads-create-title"),
                TextVariant::TEXT_MD_SEMIBOLD,
            )
            .color(colors::TEXT_DEFAULT)
            .line_clamp(1_u32)
            .build()
            .class(cls::HEADER_TITLE),
            ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .on_click(close_create)
                .build()
                .child(
                    icon(X_SMALL, colors::TEXT_MUTED)
                        .style("width: 20px; height: 20px;"),
                ),
        ]),
        div().class(cls::BODY).children([settings, composer]),
    ])
}

pub fn join(thread_id: Id<ChannelMarker>) {
    let Some(env) = env() else { return };
    let http = env.http.clone();
    guild_settings::run(
        &env.queries,
        move || async move { threads::join(&http, thread_id).await },
        |result| {
            if result.is_err() {
                menu::notify_failure(&requiem_i18n::t!("threads-membership-failed"));
            }
        },
    );
}

pub fn leave(thread_id: Id<ChannelMarker>) {
    let Some(env) = env() else { return };
    let http = env.http.clone();
    guild_settings::run(
        &env.queries,
        move || async move { threads::leave(&http, thread_id).await },
        |result| {
            if result.is_err() {
                menu::notify_failure(&requiem_i18n::t!("threads-membership-failed"));
            }
        },
    );
}

fn patch(thread_id: Id<ChannelMarker>, fields: serde_json::Value) {
    let Some(env) = env() else { return };
    let (http, state) = (env.http.clone(), env.state.clone());
    guild_settings::run(
        &env.queries,
        move || async move { threads::update(&http, thread_id, &fields).await },
        move |result| match result {
            Ok(thread) => state.channels.upsert(thread, false),
            Err(_) => menu::notify_failure(&requiem_i18n::t!("threads-update-failed")),
        },
    );
}

pub fn set_archived(thread_id: Id<ChannelMarker>, archived: bool, locked: bool) {
    patch(thread_id, json!({ "archived": archived, "locked": locked }));
}

pub fn menu_items(state: &state::AppState, channel: &Channel) -> Vec<ContextItem> {
    let id = channel.id;
    let parent = channel.parent_id;
    let joined = channel.member.is_some();
    let archived = channel.is_archived_thread();
    let locked = channel.is_locked_thread();
    let can_manage = channel
        .guild_id
        .and_then(|guild_id| state.guilds.find(guild_id))
        .is_some_and(|guild| {
            state
                .permissions
                .guild_permissions(&guild)
                .contains(Permissions::MANAGE_THREADS)
        });
    let mut items = Vec::new();
    if joined {
        items.push(ContextItem::action(
            requiem_i18n::t!("threads-leave"),
            move || leave(id),
        ));
    } else if !archived {
        items.push(ContextItem::action(
            requiem_i18n::t!("threads-join"),
            move || join(id),
        ));
    }
    if let Some(parent) = parent {
        items.push(ContextItem::action(
            requiem_i18n::t!("threads-open-parent"),
            move || {
                if let Some(env) = env() {
                    (env.on_select_channel)(parent);
                }
            },
        ));
    }
    if can_manage {
        if archived {
            items.push(ContextItem::action(
                requiem_i18n::t!("threads-unarchive"),
                move || set_archived(id, false, false),
            ));
        } else {
            items.push(ContextItem::action(
                requiem_i18n::t!("threads-close"),
                move || set_archived(id, true, false),
            ));
            items.push(ContextItem::action(
                requiem_i18n::t!("threads-close-lock"),
                move || set_archived(id, true, true),
            ));
        }
    }
    let copied = id.to_string();
    items.push(ContextItem::action(
        requiem_i18n::t!("threads-copy-id"),
        move || menu::copy_with_toast(copied.clone(), &requiem_i18n::t!("context-menus-id-copied")),
    ));
    let _ = locked;
    items
}

pub fn thread_banner(channel: &Channel) -> Option<Node> {
    let metadata = channel.thread_metadata.as_ref()?;
    if !metadata.archived && !metadata.locked {
        return None;
    }
    let text = match (metadata.archived, metadata.locked) {
        (true, true) => requiem_i18n::t!("threads-archived-locked-banner"),
        (true, false) => requiem_i18n::t!("threads-archived-banner"),
        (false, true) => requiem_i18n::t!("threads-locked-banner"),
        (false, false) => return None,
    };
    Some(div().style("padding: 12px 16px; background-color: var(--background-surface-higher); border-radius: 8px;").child(TextProps::styled(text, TextVariant::TEXT_SM_MEDIUM).color(colors::TEXT_DEFAULT).build()))
}

pub fn parent_breadcrumb(state: &state::AppState, channel: &Channel) -> Option<Node> {
    let parent = channel
        .parent_id
        .and_then(|parent| ui::find_channel(state, parent).map(|(_, parent)| parent))?;
    let parent_id = parent.id;
    Some(
        ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .on_click(move || {
                if let Some(env) = env() {
                    (env.on_select_channel)(parent_id);
                }
            })
            .build()
            .children([
                icon(
                    ui::channel_icon(state, &parent),
                    colors::TEXT_MUTED,
                )
                .style("width: 16px; height: 16px;"),
                TextProps::styled(
                    parent.name.clone().unwrap_or_default(),
                    TextVariant::TEXT_XS_MEDIUM,
                )
                .color(colors::TEXT_MUTED)
                .line_clamp(1_u32)
                .build(),
            ]),
    )
}

pub fn system_icon(color: skia_safe::Color) -> Node {
    div()
        .class(pins_cls::SYSTEM_ICON_SLOT)
        .child(icon(THREAD, color).class(pins_cls::SYSTEM_ICON))
}

pub fn system_body(
    message: &models::Message,
    name: &str,
    fill: Option<&author_style::NameFill>,
    muted: skia_safe::Color,
    profile_popout: &profile_popout::ProfilePopoutState,
    card: Option<Node>,
) -> Node {
    const USER: &str = "https://threads.invalid/user";
    const THREAD: &str = "https://threads.invalid/thread";
    const ALL: &str = "https://threads.invalid/all";
    let escaped = name.replace(['[', ']', '*', '_', '`', '~', '|', '\\'], "");
    let thread_name = message
        .thread
        .as_ref()
        .and_then(|thread| thread.name.clone())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| message.content.clone());
    let thread_label = thread_name.replace(['[', ']', '*', '_', '`', '~', '|', '\\'], "");
    let content = requiem_i18n::t!(
        "threads-system-message",
        user = format!("[{escaped}]({USER})"),
        thread = format!("[{thread_label}]({THREAD})"),
        threads = format!("[{}]({ALL})", requiem_i18n::t!("threads-system-threads"))
    );
    let (row_id, author, channel_id, thread_id) = (
        message.id,
        message.author.id,
        message.channel_id,
        message.thread.as_ref().map(|thread| thread.id),
    );
    let popout = profile_popout.clone();
    let on_link: Rc<dyn Fn(&str, AbsoluteRect)> =
        Rc::new(move |url: &str, link: AbsoluteRect| match url {
            USER => profile_popout::toggle_for(row_id, Some(author), link, &popout),
            THREAD => {
                if let (Some(thread_id), Some(env)) = (thread_id, env()) {
                    (env.on_select_channel)(thread_id);
                }
            }
            ALL => thread_browser::open_modal(channel_id),
            _ => {}
        });
    let text = message_text(MessageTextProps {
        content,
        size: 16.0,
        color: muted,
        line_height: Some(22.0),
        link_style: Some((colors::TEXT_STRONG, 500)),
        link_colors: vec![(
            USER.to_string(),
            author_style::inline_name_color(fill, colors::TEXT_STRONG),
        )],
        on_link: Some(on_link),
        ..Default::default()
    });
    let mut body = div()
        .class(pins::SYSTEM_BODY)
        .child(text);
    if let Some(time) = message.format_header_timestamp() {
        body = body.child(
            TextProps::styled(time, TextVariant::TEXT_XS_MEDIUM)
                .line_height(22.0_f32)
                .color(muted)
                .build()
                .class(pins_cls::SYSTEM_TIME),
        );
    }
    let mut column = div()
        .class(cls::SYSTEM_COLUMN)
        .child(body);
    if let Some(card) = card {
        column = column.child(
            div()
                .class(cls::SYSTEM_CARD_SLOT)
                .child(card),
        );
    }
    column
}

/// The curved line from the avatar column down to the thread card (Discord's
/// `.cozy.hasThread::after`). Its row needs the `HAS_THREAD` class.
pub fn spine(message: &models::Message, system: bool) -> Node {
    let mut spine = div().class(cls::SPINE);
    if system {
        spine = spine.class(cls::SPINE_SYSTEM);
    } else if message.is_reply() {
        spine = spine.class(cls::SPINE_REPLY);
    }
    spine
}

/// The card under a message that started a thread, or under the
/// thread-created system message. The thread shares the message's id.
pub fn thread_card(message: &models::Message, system: bool) -> Option<Node> {
    if !message.has_flag(message_flags::HAS_THREAD) {
        return None;
    }
    let env = env()?;
    let thread = ui::find_channel(&env.state, message.id.cast())
        .map(|(_, thread)| thread)
        .or_else(|| message.thread.as_deref().cloned())
        .filter(|thread| env.state.can(thread, Permissions::VIEW_CHANNEL))?;
    let thread_id = thread.id;

    let top = div()
        .class(cls::CARD_TOP)
        .child(
            TextProps::styled(
                thread.name.clone().unwrap_or_default(),
                TextVariant::TEXT_SM_SEMIBOLD,
            )
            .color(colors::TEXT_STRONG)
            .line_clamp(1_u32)
            .build()
            .class(cls::CARD_NAME),
        )
        .child(
            TextProps::styled(message_count_label(&thread), TextVariant::TEXT_SM_SEMIBOLD)
                .color(colors::TEXT_ANCHOR)
                .build()
                .class(cls::CARD_CTA),
        );

    let (state, guild_id) = (env.state.clone(), thread.guild_id);
    let mut card = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::CARD.to_string()])
        .on_click(move || {
            if let Some(env) = self::env() {
                (env.on_select_channel)(thread_id);
            }
        })
        .build()
        .on_context_menu(move |x, y| {
            let state = state.clone();
            context_menu::open(x, y, move || {
                context_menus::channel_items(&state, guild_id, thread_id)
            });
        })
        .child(top)
        .child(card_bottom_line(&env, &thread));
    if system {
        card = card.class(cls::CARD_SYSTEM);
    }
    Some(card)
}

// Discord's label: "See Thread ›" while empty, counts capped at 50+ for
// threads from before message counts were tracked.
fn message_count_label(thread: &Channel) -> String {
    const COUNT_TRACKED_SINCE: u64 = 992_549_565_104_128_000;
    let count = thread.message_count;
    if count == 0 {
        return requiem_i18n::t!("threads-see-thread").to_string();
    }
    let shown = if count >= 50 && thread.id.get() <= COUNT_TRACKED_SINCE {
        "50+".to_string()
    } else if count >= 100_000 {
        "100k+".to_string()
    } else {
        count.to_string()
    };
    if shown == "1" {
        requiem_i18n::t!("threads-one-message").to_string()
    } else {
        requiem_i18n::t!("threads-messages", count = shown).to_string()
    }
}

/// When the thread was last active: its newest message, else its creation.
pub fn last_activity_ms(state: &state::AppState, thread: &Channel) -> i64 {
    let last_message = state
        .read_states
        .last_message_id(thread.id)
        .or(thread.last_message_id.map(|id| id.get()));
    let created = thread
        .thread_metadata
        .as_ref()
        .and_then(|metadata| metadata.create_timestamp.as_deref())
        .and_then(|time| chrono::DateTime::parse_from_rfc3339(time).ok())
        .map(|time| time.timestamp_millis());
    last_message
        .map(|id| ((id >> 22) as i64) + id::DISCORD_EPOCH_MS)
        .or(created)
        .unwrap_or_else(|| thread.id.created_at_millis())
}

pub fn archived_activity_ms(state: &state::AppState, thread: &Channel) -> i64 {
    state
        .read_states
        .last_message_id(thread.id)
        .or(thread.last_message_id.map(|id| id.get()))
        .map(|id| ((id >> 22) as i64) + id::DISCORD_EPOCH_MS)
        .or_else(|| {
            thread
                .thread_metadata
                .as_ref()
                .and_then(|metadata| metadata.archive_timestamp.as_deref())
                .and_then(|time| chrono::DateTime::parse_from_rfc3339(time).ok())
                .map(|time| time.timestamp_millis())
        })
        .unwrap_or_else(|| time::now_ms() as i64)
}

/// Discord's short relative time ("5m ago", "3d ago").
pub fn ago(unix_ms: i64) -> String {
    let seconds = (time::now_ms() as i64 - unix_ms).max(0) / 1000;
    match seconds {
        s if s < 3_600 => {
            requiem_i18n::t!("threads-ago-minutes", count = (s / 60).max(1)).to_string()
        }
        s if s < 86_400 => requiem_i18n::t!("threads-ago-hours", count = s / 3_600).to_string(),
        s if s < 2_592_000 => requiem_i18n::t!("threads-ago-days", count = s / 86_400).to_string(),
        s if s < 31_104_000 => requiem_i18n::t!("threads-ago-months").to_string(),
        _ => chrono::DateTime::from_timestamp_millis(unix_ms)
            .map(|time| time::date_long(&time.with_timezone(&chrono::Local)))
            .unwrap_or_default(),
    }
}

fn card_bottom_line(env: &HomeEnv, thread: &Channel) -> Node {
    let line = div().class(cls::CARD_BOTTOM);
    let text = |content: String| {
        TextProps::styled(content, TextVariant::TEXT_SM_NORMAL)
            .color(colors::TEXT_DEFAULT)
            .line_clamp(1_u32)
            .build()
            .class(cls::CARD_PREVIEW)
    };
    let time = || {
        TextProps::styled(
            ago(last_activity_ms(&env.state, thread)),
            TextVariant::TEXT_SM_NORMAL,
        )
        .color(colors::TEXT_MUTED)
        .build()
        .class(cls::CARD_TIME)
    };
    if thread.is_archived_thread() {
        return line.child(text(requiem_i18n::t!("threads-card-no-recent").to_string()));
    }
    if thread.message_count == 0 {
        return line.child(text(requiem_i18n::t!("threads-card-empty").to_string()));
    }
    match env.state.messages.in_channel(thread.id).last() {
        None => line
            .child(text(requiem_i18n::t!("threads-card-no-recent").to_string()))
            .child(time()),
        Some(latest) => line
            .child(recent_message(env, thread, latest))
            .child(time()),
    }
}

/// Avatar, name and one line of a thread's newest message (Discord's
/// `threadMessageAccessory`).
pub fn recent_message(env: &HomeEnv, thread: &Channel, message: &models::Message) -> Node {
    let author = env.state.users.resolve(&message.author);
    let mut avatar = ui::user_avatar_props(author.id, &author.discriminator, &env.fonts);
    if let Some(image) = author.avatar.as_deref().and_then(|hash| {
        env.images
            .get_or_decode_image(&env.backend.avatar_url(author.id, hash))
    }) {
        avatar = avatar.image(image);
    }
    let roles = thread
        .guild_id
        .map(|guild_id| author_style::guild_roles(&env.state, guild_id))
        .unwrap_or_default();
    let style = author_style::resolve(&env.state, &roles, thread.guild_id, &author);
    let name = author_style::name_node(
        env.state.users.display_name(&author),
        TextVariant::TEXT_SM_MEDIUM,
        18.0,
        style.fill.as_ref(),
        colors::TEXT_STRONG,
    )
    .class(cls::RECENT_NAME);
    let content = message
        .content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let preview = message_text(MessageTextProps {
        content,
        size: 14.0,
        line_height: Some(18.0),
        color: colors::TEXT_DEFAULT,
        emoji_provider: Some(twemoji::tile),
        ..Default::default()
    })
    .class(components::line_clamp_class(1))
    .class(cls::RECENT_PREVIEW);
    div()
        .class(cls::RECENT)
        .child(avatar.build().class(cls::RECENT_AVATAR))
        .child(name)
        .child(preview)
}
