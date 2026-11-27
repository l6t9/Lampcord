//! Discord's thread browser: a popout under the header's Threads button, or a
//! modal from "See all threads" in a thread-created message. Joined and other
//! active threads come from the store; older (archived) threads and name
//! searches come from `/threads/search`.

use crate::models;
use crate::rest::endpoints::threads as endpoints_threads;
use crate::state;
use crate::ui::author_style;
use crate::ui::context_menu;
use crate::ui::context_menus;
use crate::ui::guild_settings;
use crate::ui::message_list;
use crate::ui::threads;
use crate::ui;
use requiem_ui::app;
use requiem_ui::colors;
use requiem_ui::components;
use std::cell::Cell;
use std::env;
use std::rc::Rc;
use std::sync::OnceLock;
use std::cmp;
use std::collections::HashMap;

use requiem_ui::components::{
    Align, Anchor, ButtonProps, ButtonSize, ButtonVariant, ModalProps, ModalSize, PopoverAlign,
    PopoverProps, PopoverSide, ScrollViewProps, SearchBarProps, StackProps, TextProps, TextVariant,
    div, icon, popover_backdrop, scroll_view_with,
};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::AbsoluteRect;
use requiem_ui::widget::primitives::ImageWidget;

use crate::models::id::marker::{ChannelMarker, GuildMarker};
use crate::models::{Channel, Id, Message, Permissions};
use crate::rest::endpoints::threads::{ThreadSearch, ThreadSearchQuery};
use crate::ui::classes::thread_browser as cls;
use crate::ui::dialog::Dialog;
use crate::ui::home::HomeEnv;
use crate::ui_state::{ChannelView, id_arg};
use serde::Serialize;
use specta::Type;
use crate::ui_tsx::{self, Tsx, UiAction};
use requiem_tsx::{StateQueries, Value, to_value};
use requiem_ui::icons::{THREAD, X_SMALL};

/// The virtual list loads the next archived page this close to the end.
const LOAD_AHEAD: f32 = 200.0;

#[derive(Clone, Copy, PartialEq)]
enum Context {
    Popout,
    Modal,
}

#[derive(Clone, Copy, PartialEq)]
struct Open {
    channel_id: Id<ChannelMarker>,
    context: Context,
}

#[derive(Clone, Default)]
enum Search {
    #[default]
    Idle,
    Loading,
    Results(Vec<Id<ChannelMarker>>),
}

#[derive(Clone, Default)]
struct Archived {
    threads: Vec<Channel>,
    has_more: bool,
    loading: bool,
    loaded: bool,
}

thread_local! {
    static OPEN: Signal<Option<Open>> = signal(None);
    static ANCHOR: Signal<AbsoluteRect> = signal(AbsoluteRect::default());
    static MODAL: Dialog = Dialog::new();
    static QUERY: Signal<String> = signal(String::new());
    static SEARCH: Signal<Search> = signal(Search::Idle);
    static ARCHIVED: Signal<Archived> = signal(Archived::default());
    /// Newest messages that came with search and archive pages.
    static RECENT: Signal<HashMap<Id<ChannelMarker>, Message>> = signal(HashMap::new());
    static SCROLL: Cell<f32> = const { Cell::new(0.0) };
}

fn is_open_for(channel_id: Id<ChannelMarker>, context: Context) -> bool {
    OPEN.with(Signal::get)
        == Some(Open {
            channel_id,
            context,
        })
}

fn open(channel_id: Id<ChannelMarker>, context: Context) {
    QUERY.with(|query| query.set(String::new()));
    SEARCH.with(|search| search.set(Search::Idle));
    ARCHIVED.with(|archived| archived.set(Archived::default()));
    RECENT.with(|recent| recent.set(HashMap::new()));
    SCROLL.with(|scroll| scroll.set(0.0));
    OPEN.with(|open| {
        open.set(Some(Open {
            channel_id,
            context,
        }))
    });
    if context == Context::Modal {
        MODAL.with(Dialog::open);
    }
}

pub fn close() {
    match OPEN.with(Signal::peek) {
        Some(Open {
            context: Context::Modal,
            ..
        }) => {
            MODAL.with(|modal| modal.close_then(|| OPEN.with(|open| open.set(None))));
        }
        _ => OPEN.with(|open| open.set(None)),
    }
}

pub fn open_modal(channel_id: Id<ChannelMarker>) {
    open(channel_id, Context::Modal);
}

/// The channel header's Threads button. Discord's popout sits below it,
/// right-aligned, and does not flip.
pub fn header_button(channel: &Channel) -> Option<Node> {
    if channel.is_thread() || !matches!(channel.kind, 0 | 5) {
        return None;
    }
    let channel_id = channel.id;
    let shown = is_open_for(channel_id, Context::Popout);
    Some(
        message_list::header_popout_button(
            THREAD,
            requiem_i18n::t!("threads-browser-title").to_string(),
            shown,
            move || {
                if OPEN.with(Signal::peek).is_some() {
                    close();
                } else {
                    open(channel_id, Context::Popout);
                }
            },
        )
        .observe_rect(ANCHOR.with(Signal::clone)),
    )
}

pub fn layer(env: &HomeEnv) -> Node {
    let layer = div().class(cls::LAYER);
    let Some(open) = OPEN.with(Signal::get) else {
        return layer;
    };
    let Some(channel) = ui::find_channel(&env.state, open.channel_id).map(|(_, c)| c) else {
        return layer;
    };
    match open.context {
        Context::Popout => {
            let (window_w, window_h) = app::current_window_size();
            let anchor = Anchor::new(ANCHOR.with(Signal::get))
                .position(PopoverSide::Bottom)
                .align(PopoverAlign::End)
                .auto_invert(false);
            let panel = PopoverProps::default()
                .anchor(anchor)
                .build()
                .class(cls::POPOUT)
                .style(format!(
                    "width: {}px; max-height: {}px",
                    (window_w * 0.35).clamp(480.0, 600.0),
                    window_h * 0.8
                ))
                .child(browser(env, &channel, Context::Popout));
            layer
                .class(cls::LAYER_OPEN)
                .child(popover_backdrop(close))
                .child(panel)
        }
        Context::Modal => ModalProps::default()
            .size(ModalSize::Large)
            .header(false)
            .flush_body(true)
            .dismissable(true)
            .on_dismiss(close)
            .open(MODAL.with(Dialog::toggle))
            .body(browser(env, &channel, Context::Modal))
            .build(),
    }
}

fn browser(env: &HomeEnv, channel: &Channel, context: Context) -> Node {
    let can_read = env.state.can(channel, Permissions::READ_MESSAGE_HISTORY);
    let can_create = env.state.can(channel, Permissions::CREATE_PUBLIC_THREADS)
        || env.state.can(channel, Permissions::CREATE_PRIVATE_THREADS);

    let mut header = StackProps::horizontal()
        .gap(0)
        .align(Align::Center)
        .build()
        .class(cls::HEADER)
        .child(icon(THREAD, colors::TEXT_SECONDARY).class(cls::HEADER_ICON))
        .child(
            TextProps::styled(
                requiem_i18n::t!("threads-browser-title").to_string(),
                TextVariant::HEADING_MD_SEMIBOLD,
            )
            .color(colors::TEXT_STRONG)
            .build()
            .class(cls::TITLE),
        );
    if can_read {
        let channel_id = channel.id;
        header = header.child(div().class(cls::DIVIDER)).child(
            div().class(cls::SEARCH).child(
                SearchBarProps::new(&requiem_i18n::t!("threads-browser-search"), |query| {
                    QUERY.with(|q| q.set(query.to_string()));
                    if query.is_empty() {
                        SEARCH.with(|search| search.set(Search::Idle));
                    }
                })
                .text(QUERY.with(Signal::peek))
                .on_submit(move |query| submit_search(channel_id, query))
                .autofocus(true)
                .clearable(true)
                .build(),
            ),
        );
    }
    header = header.child(div().class(cls::SPACER));
    if can_create {
        let (guild_id, channel_id) = (channel.guild_id, channel.id);
        header = header.child(
            div().class(cls::CREATE).child(
                ButtonProps::new(
                    ButtonVariant::Primary,
                    requiem_i18n::t!("threads-browser-create"),
                    move || start_thread(guild_id, channel_id),
                )
                .size(ButtonSize::Small)
                .build(),
            ),
        );
    }
    if context == Context::Modal {
        header = header.child(
            ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![cls::CLOSE.to_string()])
                .on_click(close)
                .build()
                .child(icon(X_SMALL, colors::TEXT_SECONDARY).class(cls::CLOSE_ICON)),
        );
    }

    if tsx_rows_enabled() {
        if let Some(tsx) = tsx(env) {
            if can_read && !ARCHIVED.with(Signal::peek).loaded {
                load_archived(env, channel.id);
            }
            return tsx
                .node(
                    "ThreadBrowser",
                    vec![
                        ("channelId", Value::str(channel.id.to_string())),
                        ("modal", Value::Bool(context == Context::Modal)),
                    ],
                )
                .class(cls::BROWSER);
        }
    }

    let content = match SEARCH.with(Signal::get) {
        Search::Results(ids) => search_results(env, &ids, channel, can_create),
        Search::Loading | Search::Idle => sections(env, channel, can_read, can_create),
    };
    div().class(cls::BROWSER).child(header).child(content)
}

fn start_thread(guild_id: Option<Id<GuildMarker>>, channel_id: Id<ChannelMarker>) {
    close();
    if let Some(guild_id) = guild_id {
        threads::open_create(guild_id, channel_id, None);
    }
}

fn go_to(thread_id: Id<ChannelMarker>) {
    close();
    if let Some(env) = threads::env() {
        (env.on_select_channel)(thread_id);
    }
}

fn submit_search(channel_id: Id<ChannelMarker>, query: String) {
    if query.is_empty() || matches!(SEARCH.with(Signal::peek), Search::Loading) {
        return;
    }
    let Some(env) = threads::env() else {
        return;
    };
    SEARCH.with(|search| search.set(Search::Loading));
    let (http, state) = (env.http.clone(), env.state.clone());
    guild_settings::run(
        &env.queries,
        move || async move {
            endpoints_threads::search(
                &http,
                channel_id,
                ThreadSearchQuery::Name(&query),
            )
            .await
        },
        move |result| match result {
            Ok(page) => {
                let ids = absorb(&state, &page);
                SEARCH.with(|search| search.set(Search::Results(ids)));
            }
            Err(_) => SEARCH.with(|search| search.set(Search::Idle)),
        },
    );
}

fn load_archived(env: &HomeEnv, channel_id: Id<ChannelMarker>) {
    let archived = ARCHIVED.with(Signal::peek);
    if archived.loading || (archived.loaded && !archived.has_more) {
        return;
    }
    let offset = archived.threads.len();
    ARCHIVED.with(|a| a.update(|a| a.loading = true));
    let (http, state) = (env.http.clone(), env.state.clone());
    guild_settings::run(
        &env.queries,
        move || async move {
            endpoints_threads::search(
                &http,
                channel_id,
                ThreadSearchQuery::Archived { offset },
            )
            .await
        },
        move |result| {
            if OPEN.with(Signal::peek).map(|open| open.channel_id) != Some(channel_id) {
                return;
            }
            ARCHIVED.with(|a| {
                a.update(|a| {
                    a.loading = false;
                    a.loaded = true;
                    match result {
                        Ok(page) => {
                            absorb(&state, &page);
                            a.has_more = page.has_more;
                            a.threads.extend(page.threads);
                        }
                        Err(_) => a.has_more = false,
                    }
                })
            });
        },
    );
}

/// Takes a search page's membership and newest messages; returns its ids.
fn absorb(state: &state::AppState, page: &ThreadSearch) -> Vec<Id<ChannelMarker>> {
    let me = state
        .users
        .current
        .with_untracked(|user| user.as_ref().map(|user| user.id));
    for thread in &page.threads {
        let mut thread = thread.clone();
        if thread.member.is_none() {
            thread.member = page
                .members
                .iter()
                .find(|member| {
                    member.id.is_none_or(|id| id == thread.id) && (member.user_id.is_none() || member.user_id == me)
                })
                .cloned();
        }
        if !thread.is_archived_thread() {
            state.channels.upsert(thread, false);
        }
    }
    RECENT.with(|recent| {
        recent.update(|recent| {
            for message in &page.most_recent_messages {
                recent.insert(message.channel_id, message.clone());
            }
        })
    });
    page.threads.iter().map(|thread| thread.id).collect()
}

/// Discord's `useActiveThreads`: viewable active threads of the channel,
/// newest message first, split into joined and not.
fn active_threads(env: &HomeEnv, channel: &Channel) -> (Vec<Channel>, Vec<Channel>) {
    if let Some(guild_id) = channel.guild_id {
        env.state.channels.by_guild.watch(&guild_id);
    }
    let by_guild: HashMap<_, _> = channel
        .guild_id
        .and_then(|id| {
            env.state
                .channels
                .by_guild
                .with_untracked(|all| all.get(&id).cloned().map(|channels| (id, channels)))
        })
        .into_iter()
        .collect();
    let mut threads: Vec<Channel> = channel
        .guild_id
        .and_then(|guild_id| by_guild.get(&guild_id))
        .map(|channels| {
            channels
                .iter()
                .filter(|c| {
                    c.is_thread()
                        && c.parent_id == Some(channel.id)
                        && !c.is_archived_thread()
                        && env.state.can(c, Permissions::VIEW_CHANNEL)
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let last_message = |thread: &Channel| {
        env.state
            .read_states
            .last_message_id(thread.id)
            .or(thread.last_message_id.map(|id| id.get()))
            .unwrap_or(0)
    };
    threads.sort_by_key(|thread| cmp::Reverse(last_message(thread)));
    threads
        .into_iter()
        .partition(|thread| thread.member.is_some())
}

fn sections(env: &HomeEnv, channel: &Channel, can_read: bool, can_create: bool) -> Node {
    if can_read && !ARCHIVED.with(Signal::peek).loaded {
        load_archived(env, channel.id);
    }
    let (joined, other) = active_threads(env, channel);
    let archived = ARCHIVED.with(Signal::get);
    let older: Vec<Channel> = archived
        .threads
        .iter()
        .filter(|thread| env.state.can(thread, Permissions::VIEW_CHANNEL))
        .cloned()
        .collect();

    if joined.is_empty() && other.is_empty() && older.is_empty() {
        return if archived.loading || !archived.loaded && can_read {
            div().class(cls::LIST).child(div().class(cls::LOADING))
        } else {
            div()
                .class(cls::LIST)
                .child(empty_state(channel, can_create))
        };
    }

    let mut column = StackProps::vertical().gap(8).build().class(cls::ROWS);
    let mut shown_sections = 0;
    for (title, threads) in [
        (
            requiem_i18n::t!("threads-browser-joined", count = joined.len()),
            &joined,
        ),
        (
            requiem_i18n::t!("threads-browser-other", count = other.len()),
            &other,
        ),
        (requiem_i18n::t!("threads-browser-older"), &older),
    ] {
        if threads.is_empty() {
            continue;
        }
        shown_sections += 1;
        column = column.child(section_header(title.to_string()));
        for thread in threads {
            column = column.child(row(env, thread));
        }
    }
    // Discord's list reserves 64px for every section after the first but
    // draws its header in 32, so each one leaves 32px at the end.
    let column = column.style(format!(
        "padding-bottom: {}px",
        8 + 32 * (shown_sections - 1)
    ));

    let channel_id = channel.id;
    let (more, env_for_scroll) = (archived.has_more && !archived.loading, env.clone());
    scroll_view_with(ScrollViewProps {
        initial_offset_y: SCROLL.with(Cell::get),
        on_scroll: (move |offset: f32, max: f32| {
            SCROLL.with(|scroll| scroll.set(offset));
            if more && max - offset < LOAD_AHEAD {
                load_archived(&env_for_scroll, channel_id);
            }
        })
        .into(),
        ..Default::default()
    })
    .class(cls::LIST)
    .child(column)
}

fn search_results(
    env: &HomeEnv,
    ids: &[Id<ChannelMarker>],
    channel: &Channel,
    can_create: bool,
) -> Node {
    let archived = ARCHIVED.with(Signal::peek);
    let threads: Vec<Channel> = ids
        .iter()
        .filter_map(|id| {
            ui::find_channel(&env.state, *id)
                .map(|(_, thread)| thread)
                .or_else(|| archived.threads.iter().find(|t| t.id == *id).cloned())
        })
        .collect();
    if threads.is_empty() {
        return div()
            .class(cls::LIST)
            .child(empty_state(channel, can_create));
    }
    let mut column = StackProps::vertical()
        .gap(8)
        .build()
        .class(cls::ROWS)
        .class(cls::SEARCH_ROWS);
    for thread in &threads {
        column = column.child(row(env, thread));
    }
    scroll_view_with(ScrollViewProps::default())
        .class(cls::LIST)
        .child(column)
}

fn section_header(title: String) -> Node {
    TextProps::styled(title.to_uppercase(), TextVariant::TEXT_XS_BOLD)
        .color(colors::TEXT_DEFAULT)
        .build()
        .class(cls::SECTION_HEADER)
}

fn empty_state(channel: &Channel, can_create: bool) -> Node {
    thread_local! {
        static IMAGE: Option<skia_safe::Image> = ui::asset_svg("image/threads_empty", 256, 144);
    }
    let mut empty = StackProps::vertical()
        .gap(0)
        .align(Align::Center)
        .build()
        .class(cls::EMPTY)
        .child(Node::new("div", ImageWidget::new(IMAGE.with(Clone::clone))).class(cls::EMPTY_IMAGE))
        .child(
            TextProps::styled(
                requiem_i18n::t!("threads-browser-empty-title").to_string(),
                TextVariant::HEADING_XL_SEMIBOLD,
            )
            .color(colors::TEXT_STRONG)
            .center(true)
            .build()
            .class(cls::EMPTY_TITLE),
        )
        .child(
            TextProps::styled(
                requiem_i18n::t!("threads-browser-empty-body").to_string(),
                TextVariant::TEXT_MD_NORMAL,
            )
            .color(colors::TEXT_DEFAULT)
            .center(true)
            .build(),
        );
    if can_create {
        let (guild_id, channel_id) = (channel.guild_id, channel.id);
        empty = empty.child(
            div().class(cls::EMPTY_CTA).child(
                ButtonProps::new(
                    ButtonVariant::Primary,
                    requiem_i18n::t!("threads-create"),
                    move || start_thread(guild_id, channel_id),
                )
                .build(),
            ),
        );
    }
    empty
}

/// Discord's `ThreadBrowserRow`: name, then the newest message (active) or
/// who started it (archived).
fn tsx_rows_enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| env::var_os("REQUIEM_TSX_ROWS").is_none_or(|value| value != "0"))
}

fn tsx(env: &HomeEnv) -> Option<Rc<Tsx>> {
    ui_tsx::shared(&env.state, &env.fonts, &env.images, &env.backend, &env.tick)
}

pub fn handle(action: UiAction) {
    match action {
        UiAction::GoToThread { thread_id, .. } => go_to(thread_id),
        UiAction::OpenChannelMenu {
            channel_id,
            guild_id,
            x,
            y,
        } => {
            let Some(env) = threads::env() else { return };
            let state = env.state.clone();
            context_menu::open(x as f32, y as f32, move || {
                context_menus::channel_items(&state, guild_id, channel_id)
            });
        }
        UiAction::LoadMoreThreads { channel_id } => {
            if let Some(env) = threads::env() {
                load_archived(&env, channel_id);
            }
        }
        UiAction::RememberThreadScroll { offset } => SCROLL.with(|scroll| scroll.set(offset as f32)),
        UiAction::ThreadSearchChanged { query } => {
            if query.is_empty() {
                SEARCH.with(|search| search.set(Search::Idle));
            }
            QUERY.with(|q| q.set(query));
        }
        UiAction::SubmitThreadSearch { channel_id, query } => submit_search(channel_id, query),
        UiAction::CloseThreadBrowser {} => close(),
        UiAction::StartThread { channel_id } => {
            let Some(env) = threads::env() else { return };
            if let Some((guild_id, _)) = ui::find_channel(&env.state, channel_id) {
                start_thread(Some(guild_id), channel_id);
            }
        }
        _ => {}
    }
}

#[derive(Clone, Copy, Serialize, Type)]
#[serde(rename_all = "lowercase")]
enum EntryKind {
    Pad,
    Header,
    Thread,
}

#[derive(Clone, Copy, Serialize, Type)]
#[serde(rename_all = "lowercase")]
enum Section {
    Joined,
    Other,
    Older,
    Results,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
struct BrowserEntry {
    key: String,
    kind: EntryKind,
    section: Section,
    count: u32,
    height: f64,
    thread: Option<ChannelView>,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "lowercase")]
enum Phase {
    Loading,
    Empty,
    List,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
struct BrowserAccess {
    can_read: bool,
    can_create: bool,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
struct BrowserView {
    phase: Phase,
    can_create: bool,
    entries: Vec<BrowserEntry>,
}

const ROW_HEIGHT: f64 = 72.0;
const HEADER_HEIGHT: f64 = 24.0;

pub fn browser_queries(queries: StateQueries) -> StateQueries {
    queries
        .one(
            "threads.browserView",
            &[("channelId", "ChannelId")],
            |args| {
                let env = threads::env()?;
                let (_, channel) = ui::find_channel(&env.state, id_arg(args, 0)?)?;
                Some(browser_view(&env, &channel))
            },
        )
        .one(
            "threads.browserAccess",
            &[("channelId", "ChannelId")],
            |args| {
                let env = threads::env()?;
                let (_, channel) = ui::find_channel(&env.state, id_arg(args, 0)?)?;
                Some(BrowserAccess {
                    can_read: env.state.can(&channel, Permissions::READ_MESSAGE_HISTORY),
                    can_create: env.state.can(&channel, Permissions::CREATE_PUBLIC_THREADS)
                        || env.state.can(&channel, Permissions::CREATE_PRIVATE_THREADS),
                })
            },
        )
        .scalar("threads.searchQuery", &[], "string", |_| {
            QUERY.with(Signal::peek)
        })
        .scalar("threads.scrollOffset", &[], "number", |_| {
            f64::from(SCROLL.with(Cell::get))
        })
}

fn pad(key: &str, height: f64) -> BrowserEntry {
    BrowserEntry {
        key: key.to_string(),
        kind: EntryKind::Pad,
        section: Section::Results,
        count: 0,
        height,
        thread: None,
    }
}

fn thread_entry(thread: &Channel, section: Section) -> BrowserEntry {
    BrowserEntry {
        key: thread.id.to_string(),
        kind: EntryKind::Thread,
        section,
        count: 0,
        height: ROW_HEIGHT,
        thread: Some(ChannelView::from(thread)),
    }
}

fn browser_view(env: &HomeEnv, channel: &Channel) -> BrowserView {
    let can_read = env.state.can(channel, Permissions::READ_MESSAGE_HISTORY);
    let can_create = env.state.can(channel, Permissions::CREATE_PUBLIC_THREADS)
        || env.state.can(channel, Permissions::CREATE_PRIVATE_THREADS);
    let archived = ARCHIVED.with(Signal::get);

    if let Search::Results(ids) = SEARCH.with(Signal::get) {
        let found: Vec<Channel> = ids
            .iter()
            .filter_map(|id| {
                ui::find_channel(&env.state, *id)
                    .map(|(_, thread)| thread)
                    .or_else(|| archived.threads.iter().find(|t| t.id == *id).cloned())
            })
            .collect();
        if found.is_empty() {
            return BrowserView {
                phase: Phase::Empty,
                can_create,
                entries: Vec::new(),
            };
        }
        let mut entries = vec![pad("pad-top", 8.0)];
        entries.extend(found.iter().map(|thread| thread_entry(thread, Section::Results)));
        entries.push(pad("pad-bottom", 0.0));
        return BrowserView {
            phase: Phase::List,
            can_create,
            entries,
        };
    }

    let (joined, other) = active_threads(env, channel);
    let older: Vec<Channel> = archived
        .threads
        .iter()
        .filter(|thread| env.state.can(thread, Permissions::VIEW_CHANNEL))
        .cloned()
        .collect();
    if joined.is_empty() && other.is_empty() && older.is_empty() {
        let loading = archived.loading || !archived.loaded && can_read;
        return BrowserView {
            phase: if loading { Phase::Loading } else { Phase::Empty },
            can_create,
            entries: Vec::new(),
        };
    }

    let mut entries = vec![pad("pad-top", 0.0)];
    let mut shown_sections = 0_u32;
    for (section, key, threads) in [
        (Section::Joined, "joined", &joined),
        (Section::Other, "other", &other),
        (Section::Older, "older", &older),
    ] {
        if threads.is_empty() {
            continue;
        }
        shown_sections += 1;
        entries.push(BrowserEntry {
            key: format!("header-{key}"),
            kind: EntryKind::Header,
            section,
            count: threads.len() as u32,
            height: HEADER_HEIGHT,
            thread: None,
        });
        entries.extend(threads.iter().map(|thread| thread_entry(thread, section)));
    }
    entries.push(pad("pad-bottom", 32.0 * f64::from(shown_sections - 1)));
    BrowserView {
        phase: Phase::List,
        can_create,
        entries,
    }
}

fn tsx_row(env: &HomeEnv, thread: &Channel) -> Option<Node> {
    Some(tsx(env)?.node(
        "ThreadBrowserRow",
        vec![("thread", to_value(&ChannelView::from(thread)))],
    ))
}

fn row(env: &HomeEnv, thread: &Channel) -> Node {
    if tsx_rows_enabled() {
        if let Some(node) = tsx_row(env, thread) {
            return node;
        }
    }
    let thread_id = thread.id;
    let name = TextProps::styled(
        thread.name.clone().unwrap_or_default(),
        TextVariant::HEADING_MD_SEMIBOLD,
    )
    .color(colors::TEXT_DEFAULT)
    .line_clamp(1_u32)
    .build();
    let subtext = if thread.is_archived_thread() {
        started_by(env, thread)
    } else {
        latest(env, thread)
    };
    let (state, guild_id) = (env.state.clone(), thread.guild_id);
    ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::ROW.to_string()])
        .on_click(move || go_to(thread_id))
        .build()
        .on_context_menu(move |x, y| {
            let state = state.clone();
            context_menu::open(x, y, move || {
                context_menus::channel_items(&state, guild_id, thread_id)
            });
        })
        .child(
            StackProps::vertical()
                .gap(0)
                .build()
                .class(cls::ROW_LEFT)
                .child(name)
                .child(subtext),
        )
}

fn subtext_text(content: String) -> Node {
    TextProps::styled(content, TextVariant::TEXT_SM_NORMAL)
        .color(colors::TEXT_DEFAULT)
        .line_clamp(1_u32)
        .build()
}

fn avatar(env: &HomeEnv, user: &models::User) -> Node {
    let mut avatar = ui::user_avatar_props(user.id, &user.discriminator, &env.fonts);
    if let Some(image) = user.avatar.as_deref().and_then(|hash| {
        env.images
            .get_or_decode_image(&env.backend.avatar_url(user.id, hash))
    }) {
        avatar = avatar.image(image);
    }
    avatar.build().class(cls::AVATAR)
}

fn author_name(env: &HomeEnv, thread: &Channel, user: &models::User) -> Node {
    let roles = thread
        .guild_id
        .map(|guild_id| author_style::guild_roles(&env.state, guild_id))
        .unwrap_or_default();
    let style = author_style::resolve(&env.state, &roles, thread.guild_id, user);
    author_style::name_node(
        env.state.users.display_name(user),
        TextVariant::TEXT_SM_SEMIBOLD,
        18.0,
        style.fill.as_ref(),
        colors::TEXT_DEFAULT,
    )
    .class(components::line_clamp_class(1))
    .class(cls::AUTHOR)
}

fn latest(env: &HomeEnv, thread: &Channel) -> Node {
    let line = StackProps::horizontal()
        .gap(0)
        .align(Align::Center)
        .build()
        .class(cls::SUBTEXT);
    let time = format!(
        "\u{a0} • \u{a0}{}",
        threads::ago(threads::last_activity_ms(&env.state, thread))
    );
    let recent = RECENT
        .with(|recent| recent.get().get(&thread.id).cloned())
        .or_else(|| env.state.messages.in_channel(thread.id).last().cloned());
    let Some(message) = recent else {
        return line.child(subtext_text(format!(
            "{}{time}",
            requiem_i18n::t!("threads-browser-no-recent")
        )));
    };
    let author = env.state.users.resolve(&message.author);
    let content = message
        .content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    line.child(avatar(env, &author))
        .child(author_name(env, thread, &author))
        .child(subtext_text(format!(":\u{a0}{content}")).class(cls::MESSAGE))
        .child(subtext_text(time).class(cls::TIME))
}

fn started_by(env: &HomeEnv, thread: &Channel) -> Node {
    let line = StackProps::horizontal()
        .gap(0)
        .align(Align::Center)
        .build()
        .class(cls::SUBTEXT);
    let last_active = threads::archived_activity_ms(&env.state, thread);
    let last_active = subtext_text(
        requiem_i18n::t!(
            "threads-browser-last-active",
            time = threads::ago(last_active)
        )
        .to_string(),
    );
    let owner = thread.owner_id.and_then(|id| env.state.users.get(id));
    let Some(owner) = owner else {
        return line
            .child(
                div().class(cls::NO_AVATAR).child(
                    icon(THREAD, colors::TEXT_DEFAULT).class(cls::NO_AVATAR_ICON),
                ),
            )
            .child(last_active);
    };
    line.child(avatar(env, &owner))
        .child(subtext_text(
            requiem_i18n::t!("threads-browser-started-by").to_string(),
        ))
        .child(author_name(env, thread, &owner).class(cls::STARTED_BY_NAME))
        .child(
            TextProps::styled("•".to_string(), TextVariant::TEXT_SM_NORMAL)
                .color(colors::BACKGROUND_MOD_MUTED)
                .build()
                .class(cls::BULLET),
        )
        .child(last_active)
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::rc::Rc;

    use requiem_ui::font_system::{self, TextFonts};
    use requiem_ui::event::{NamedKey, UiEvent};
    use requiem_ui::signal::flush;
    use requiem_ui::style::ComputedStyle;
    use requiem_ui::theme;
    use requiem_ui::widget::primitives::{BoxWidget, TextInputWidget, TextWidget};
    use requiem_ui::widget::{AbsoluteRect, WidgetId, WidgetTree};
    use skia_safe::font_style::Weight;
    use tokio::runtime::Runtime;

    use super::*;
    use crate::auth::AuthCoordinator;
    use crate::config::BackendConfig;
    use crate::images::ImageCache;
    use crate::query::QueryClient;
    use crate::rest::HttpClient;
    use crate::state::AppState;
    use crate::ui::Fonts;

    struct Browser {
        tree: WidgetTree,
        env: HomeEnv,
        _runtime: Runtime,
    }

    fn browser() -> Browser {
        let runtime = Runtime::new().expect("runtime");
        let images = ImageCache::new(runtime.handle().clone(), wreq::Client::new(), || {});
        let fonts = Fonts {
            typeface: theme::make_typeface(Weight::MEDIUM),
            text_fonts: TextFonts::new(1024 * 1024),
        };
        font_system::install_ambient(fonts.text_fonts.clone());
        let backend = BackendConfig {
            rest_base_url: String::new(),
            gateway_url: String::new(),
            cdn_base_url: String::new(),
        };
        let env = HomeEnv {
            state: AppState::new(),
            fonts,
            images,
            backend: backend.clone(),
            queries: QueryClient::new(runtime.handle().clone(), || {}),
            http: HttpClient::new(backend.clone(), AuthCoordinator::new(backend, "token".into()))
                .expect("http"),
            tick: signal(0),
            on_select_channel: Rc::new(|_| {}),
            on_open_friends: Rc::new(|| {}),
        };
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let root = tree.root;
        let shared = env.clone();
        threads::mount(&mut tree, root, Rc::new(move || shared.clone()));
        ARCHIVED.with(|archived| archived.set(Archived::default()));
        SEARCH.with(|search| search.set(Search::Idle));
        Browser {
            tree,
            env,
            _runtime: runtime,
        }
    }

    fn channel(json: serde_json::Value) -> Channel {
        serde_json::from_value(json).expect("channel")
    }

    fn parent() -> Channel {
        channel(serde_json::json!({"id": "100", "guild_id": "9", "type": 0, "name": "general"}))
    }

    fn thread(id: u64, name: &str, joined: bool, archived: bool) -> Channel {
        let mut json = serde_json::json!({
            "id": id.to_string(), "guild_id": "9", "type": 11, "name": name, "parent_id": "100",
            "thread_metadata": {"archived": archived, "auto_archive_duration": 1440}
        });
        if joined {
            json["member"] = serde_json::json!({"flags": 0});
        }
        channel(json)
    }

    fn mount_list(browser: &mut Browser) -> WidgetId {
        let tsx = tsx(&browser.env).expect("tsx compiles");
        let root = browser.tree.root;
        let node = tsx.node(
            "ThreadBrowserList",
            vec![("channelId", Value::str("100"))],
        );
        browser.tree.mount(root, node)
    }

    fn texts(tree: &WidgetTree, id: WidgetId, out: &mut Vec<String>) {
        if let Some(text) = (&*tree.nodes[id].widget as &dyn Any).downcast_ref::<TextWidget>() {
            out.push(text.text().to_string());
        }
        for &child in &tree.nodes[id].children {
            texts(tree, child, out);
        }
    }

    fn shown(browser: &Browser, id: WidgetId) -> Vec<String> {
        let mut out = Vec::new();
        texts(&browser.tree, id, &mut out);
        out
    }

    #[test]
    fn active_threads_are_listed_under_joined_and_other_headers() {
        let mut b = browser();
        b.env.state.channels.upsert(parent(), false);
        b.env.state.channels.upsert(thread(201, "mine", true, false), false);
        b.env.state.channels.upsert(thread(202, "theirs", false, false), false);
        ARCHIVED.with(|a| a.set(Archived { loaded: true, ..Archived::default() }));
        let list = mount_list(&mut b);
        let text = shown(&b, list);
        let joined = requiem_i18n::t!("threads-browser-joined", count = 1).to_string().to_uppercase();
        let other = requiem_i18n::t!("threads-browser-other", count = 1).to_string().to_uppercase();
        assert_eq!(text[0], joined, "{text:?}");
        assert_eq!(text[1], "mine");
        assert!(text.contains(&other), "{text:?}");
        assert!(text.contains(&"theirs".to_string()), "{text:?}");
    }

    #[test]
    fn an_unloaded_archive_shows_the_loading_block_then_the_empty_state() {
        let mut b = browser();
        b.env.state.channels.upsert(parent(), false);
        let list = mount_list(&mut b);
        assert!(shown(&b, list).is_empty(), "loading draws no text");

        ARCHIVED.with(|a| a.set(Archived { loaded: true, ..Archived::default() }));
        flush(&mut b.tree);
        let text = shown(&b, list);
        assert!(
            text.contains(&requiem_i18n::t!("threads-browser-empty-title").to_string()),
            "{text:?}"
        );
        assert!(text.contains(&requiem_i18n::t!("threads-create").to_string()), "{text:?}");
    }

    #[test]
    fn a_long_archive_mounts_only_the_rows_near_the_viewport() {
        let mut b = browser();
        b.env.state.channels.upsert(parent(), false);
        let threads: Vec<Channel> = (0..500)
            .map(|n| thread(1000 + n, &format!("old {n}"), false, true))
            .collect();
        ARCHIVED.with(|a| {
            a.set(Archived {
                threads,
                has_more: true,
                loading: false,
                loaded: true,
            })
        });
        let list = mount_list(&mut b);
        let text = shown(&b, list);
        assert!(text.contains(&"old 0".to_string()), "{text:?}");
        assert!(!text.contains(&"old 499".to_string()));
        let rows = text.iter().filter(|t| t.starts_with("old ")).count();
        assert!(rows < 30, "{rows} rows mounted of 500");
    }

    #[test]
    fn search_results_replace_the_sections() {
        let mut b = browser();
        b.env.state.channels.upsert(parent(), false);
        b.env.state.channels.upsert(thread(201, "alpha", true, false), false);
        b.env.state.channels.upsert(thread(202, "beta", false, false), false);
        ARCHIVED.with(|a| a.set(Archived { loaded: true, ..Archived::default() }));
        let list = mount_list(&mut b);
        SEARCH.with(|s| s.set(Search::Results(vec![Id::new(202)])));
        flush(&mut b.tree);
        let text = shown(&b, list);
        assert_eq!(text[0], "beta", "{text:?}");
        assert!(!text.contains(&"alpha".to_string()), "{text:?}");
    }

    fn mount_browser(browser: &mut Browser, modal: bool) -> WidgetId {
        let tsx = tsx(&browser.env).expect("tsx compiles");
        let root = browser.tree.root;
        let node = tsx.node(
            "ThreadBrowser",
            vec![
                ("channelId", Value::str("100")),
                ("modal", Value::Bool(modal)),
            ],
        );
        browser.tree.mount(root, node)
    }

    fn find_input(tree: &WidgetTree, id: WidgetId) -> Option<WidgetId> {
        if (&*tree.nodes[id].widget as &dyn Any)
            .downcast_ref::<TextInputWidget>()
            .is_some()
        {
            return Some(id);
        }
        tree.nodes[id]
            .children
            .iter()
            .find_map(|&child| find_input(tree, child))
    }

    #[test]
    fn the_header_shows_the_title_search_and_create() {
        let mut b = browser();
        b.env.state.channels.upsert(parent(), false);
        ARCHIVED.with(|a| a.set(Archived { loaded: true, ..Archived::default() }));
        let root = mount_browser(&mut b, false);
        let text = shown(&b, root);
        assert_eq!(text[0], requiem_i18n::t!("threads-browser-title").to_string(), "{text:?}");
        assert!(text.contains(&requiem_i18n::t!("threads-browser-create").to_string()), "{text:?}");
        assert!(find_input(&b.tree, root).is_some(), "the search bar is mounted");
    }

    #[test]
    fn typing_in_the_search_bar_updates_the_query_and_clearing_it_resets_the_results() {
        let mut b = browser();
        b.env.state.channels.upsert(parent(), false);
        ARCHIVED.with(|a| a.set(Archived { loaded: true, ..Archived::default() }));
        let root = mount_browser(&mut b, false);
        let input = find_input(&b.tree, root).expect("search input");
        QUERY.with(|q| q.set(String::new()));
        b.tree.nodes[input].widget.handle_event(
            &UiEvent::TextInput { text: "rel".into() },
            AbsoluteRect::default(),
        );
        assert_eq!(QUERY.with(Signal::peek), "rel");

        SEARCH.with(|s| s.set(Search::Results(Vec::new())));
        for _ in 0..3 {
            b.tree.nodes[input].widget.handle_event(
                &UiEvent::KeyDown {
                    key: NamedKey::Backspace,
                    shift: false,
                },
                AbsoluteRect::default(),
            );
        }
        assert_eq!(QUERY.with(Signal::peek), "");
        assert!(matches!(SEARCH.with(Signal::peek), Search::Idle));
    }

    #[test]
    fn the_close_button_is_only_in_the_modal() {
        let mut b = browser();
        b.env.state.channels.upsert(parent(), false);
        ARCHIVED.with(|a| a.set(Archived { loaded: true, ..Archived::default() }));
        let popout = mount_browser(&mut b, false);
        let modal = mount_browser(&mut b, true);
        assert_eq!(count_with_class(&b.tree, popout, cls::CLOSE), 0);
        assert_eq!(count_with_class(&b.tree, modal, cls::CLOSE), 1);
    }

    fn count_with_class(tree: &WidgetTree, id: WidgetId, class: &str) -> usize {
        let own = usize::from(tree.nodes[id].classes.iter().any(|c| c == class));
        own + tree.nodes[id]
            .children
            .iter()
            .map(|&child| count_with_class(tree, child, class))
            .sum::<usize>()
    }
}
