use crate::query;
use crate::ui::classes::inbox as cls;
use crate::ui::context_menu;
use crate::ui::home;
use crate::ui::search;
use crate::ui;
use crate::utils::guild;
use requiem_ui::app;
use requiem_ui::colors;
use requiem_ui::icons::Icon;
use requiem_ui::scroll_animation;
use requiem_ui::timer;
use std::cell::{Cell, RefCell};
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::{Duration, Instant};

use requiem_ui::components::{
    ButtonProps, ButtonSize, ButtonVariant, ConfirmModalProps, ContextItem, NumberBadgeProps,
    ScrollViewProps, TabBarProps, TabItem, TextProps, TextVariant, TooltipPosition, TooltipProps,
    confirm_modal, div, icon, number_badge, popover_backdrop, scroll_view_with, tab_bar, tooltip,
};
use requiem_ui::icons;
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::AbsoluteRect;
use requiem_ui::widget::anim::{ToggleState, ease_out, lerp};
use requiem_ui::widget::primitives::{ImageWidget, PulsingEllipsisWidget};

use crate::models::id::marker::{ChannelMarker, MessageMarker};
use crate::models::{Channel, Guild, Id, Message};
use crate::rest::endpoints::messages::get_channel_messages_after;
use crate::rest::endpoints::read_states::{MENTIONS_PAGE, MentionsFilter, recent_mentions};
use crate::ui::search::ResultsEnv;
use crate::ui::{TEXT_DEFAULT, TEXT_STRONG, TEXT_SUBTLE};

const EAGER_CARDS: usize = 4;
const CARD_HEIGHT: f32 = 96.0;
const MENTION_HEIGHT: f32 = 148.0;
const WINDOW_OVERSCAN: f32 = 480.0;
const WINDOW_EDGE: f32 = 240.0;
const CARD_MESSAGES: u8 = 8;
const STALE: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tab {
    Unreads,
    Mentions,
}

impl Tab {
    fn id(self) -> &'static str {
        match self {
            Tab::Unreads => "unreads",
            Tab::Mentions => "mentions",
        }
    }

    fn from_id(id: &str) -> Self {
        if id == "mentions" {
            Tab::Mentions
        } else {
            Tab::Unreads
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Filter {
    all_servers: bool,
    roles: bool,
    everyone: bool,
}

thread_local! {
    static OPEN: Signal<bool> = signal(false);
    static TAB: Signal<Tab> = signal(Tab::Unreads);

    static ANCHOR: Cell<(f32, f32)> = const { Cell::new((0.0, 28.0)) };
    static FILTER: Signal<Filter> = signal(Filter { all_servers: true, roles: true, everyone: true });

    static PAGES: Signal<Vec<Option<Id<MessageMarker>>>> = signal(vec![None]);

    static DISMISSED: RefCell<HashSet<u64>> = RefCell::new(HashSet::new());
    static CONFIRM: Signal<ToggleState> = signal(ToggleState::new(false));

    static COLLAPSED: Signal<Vec<u64>> = signal(Vec::new());

    static FOLDS: RefCell<HashMap<u64, Fold>> = RefCell::new(HashMap::new());

    static SCROLL: Cell<(f32, f32)> = const { Cell::new((0.0, 0.0)) };
    static WINDOW: Signal<u64> = signal(0);
    static HEIGHTS: RefCell<HashMap<u64, f32>> = RefCell::new(HashMap::new());
    static ANCHORED: Cell<Option<(u64, f32)>> = const { Cell::new(None) };
}

const FOLD_DURATION: Duration = Duration::from_millis(200);

struct Fold {
    style: Signal<String>,
    size: Signal<(f32, f32)>,
    anim: Option<FoldAnim>,
}

struct FoldAnim {
    from: f32,

    expanding: bool,

    started: Option<Instant>,
}

fn fold_signals(channel: u64) -> (Signal<String>, Signal<(f32, f32)>) {
    FOLDS.with(|folds| {
        let mut folds = folds.borrow_mut();
        let fold = folds.entry(channel).or_insert_with(|| Fold {
            style: signal(String::new()),
            size: signal((0.0, 0.0)),
            anim: None,
        });
        (fold.style.clone(), fold.size.clone())
    })
}

fn is_folding(channel: u64) -> bool {
    FOLDS.with(|folds| {
        folds
            .borrow()
            .get(&channel)
            .is_some_and(|fold| fold.anim.is_some())
    })
}

fn toggle_fold(channel: u64) {
    let (style, size) = fold_signals(channel);
    let collapsing = !COLLAPSED.with(|c| c.peek().contains(&channel));
    let height = size.peek().1;
    FOLDS.with(|folds| {
        if let Some(fold) = folds.borrow_mut().get_mut(&channel) {
            fold.anim = Some(if collapsing {
                FoldAnim {
                    from: height,
                    expanding: false,
                    started: Some(Instant::now()),
                }
            } else {
                FoldAnim {
                    from: 0.0,
                    expanding: true,
                    started: None,
                }
            });
        }
    });
    style.set(format!("height: {height}px"));
    if !collapsing {
        style.set("height: 0px".to_string());
    }
    COLLAPSED.with(|c| {
        c.update(|ids| match ids.iter().position(|id| *id == channel) {
            Some(at) => {
                ids.remove(at);
            }
            None => ids.push(channel),
        })
    });
    timer::request_animation_frame(fold_tick);
}

fn fold_tick() {
    let now = Instant::now();
    let mut running = false;
    let mut finished = false;
    FOLDS.with(|folds| {
        for fold in folds.borrow_mut().values_mut() {
            let Some(anim) = &mut fold.anim else { continue };
            let target = if anim.expanding {
                fold.size.peek().1
            } else {
                0.0
            };
            if anim.expanding && anim.started.is_none() && target > 0.0 {
                anim.started = Some(now);
            }
            let Some(started) = anim.started else {
                running = true;
                continue;
            };
            let t = (started.elapsed().as_secs_f32() / FOLD_DURATION.as_secs_f32()).min(1.0);
            if t >= 1.0 {
                fold.style.set(if anim.expanding {
                    String::new()
                } else {
                    "height: 0px".to_string()
                });
                fold.anim = None;
                finished = true;
            } else {
                fold.style.set(format!(
                    "height: {}px",
                    lerp(anim.from, target, ease_out(t))
                ));
                running = true;
            }
        }
    });
    if finished {
        COLLAPSED.with(|c| c.update(|_| {}));
    }
    if running {
        timer::request_animation_frame(fold_tick);
    }
}

const LOAD_AHEAD: f32 = 160.0;

pub fn toggle(anchor: AbsoluteRect) {
    ANCHOR.with(|a| a.set((anchor.x + anchor.width, anchor.y + anchor.height)));
    SCROLL.with(|s| s.set((0.0, 0.0)));
    OPEN.with(|open| open.update(|open| *open = !*open));
}

pub fn close() {
    SCROLL.with(|s| s.set((0.0, 0.0)));
    OPEN.with(|open| open.set(false));
}

pub fn is_open() -> bool {
    OPEN.with(Signal::peek)
}

fn panel_width(window_width: f32) -> f32 {
    (window_width * 0.35).clamp(480.0, 600.0)
}

fn panel_max_height(window_height: f32) -> f32 {
    window_height
        * if window_height <= 700.0 {
            0.85
        } else if window_height <= 800.0 {
            0.8
        } else {
            0.7
        }
}

fn mentions_key(filter: Filter, before: Option<Id<MessageMarker>>, guild: Option<u64>) -> String {
    format!(
        "inbox:mentions:{}:{}:{}:{}:{}",
        filter.roles,
        filter.everyone,
        filter.all_servers,
        guild.unwrap_or(0),
        before.map_or(0, |id| id.get())
    )
}

pub fn build(env: &ResultsEnv, on_friend_requests: &Rc<dyn Fn()>) -> Node {
    let mut layer = div().class(cls::LAYER);
    let open = OPEN.with(Signal::get);
    let confirm = CONFIRM.with(Signal::get);
    let closing = !confirm.on && confirm.changed_at.elapsed() < Duration::from_millis(350);

    if open {
        layer = layer
            .class(cls::LAYER_OPEN)
            .child(popover_backdrop(close))
            .child(panel(env, on_friend_requests));
    }
    if confirm.on || closing {
        let state = env.state.clone();
        let props = ConfirmModalProps::new(
            requiem_i18n::t!("inbox-mark-all-as-read-title"),
            requiem_i18n::t!("inbox-mark-as-read"),
            || CONFIRM.with(|c| c.set(ToggleState::new(false).set(false))),
            confirm,
        )
        .message(requiem_i18n::t!("inbox-mark-all-as-read-body"));
        let mut props = props;
        props.on_confirm = Rc::new(move || state.read_states.mark_all_read());
        layer = layer.child(confirm_modal(props));
    }
    layer
}

fn panel(env: &ResultsEnv, on_friend_requests: &Rc<dyn Fn()>) -> Node {
    let (window_w, window_h) = app::current_window_size();
    let (right_edge, bottom) = ANCHOR.with(Cell::get);
    let tab = TAB.with(Signal::get);

    let header = header(env, tab, on_friend_requests);
    let body = match tab {
        Tab::Unreads => unreads(env),
        Tab::Mentions => mentions(env),
    };

    let style = format!(
        "width: {}px; max-height: {}px; right: {}px; top: {}px",
        panel_width(window_w),
        panel_max_height(window_h),
        (window_w - right_edge).max(0.0),
        bottom + 8.0
    );
    div()
        .class(cls::PANEL)
        .style(style)
        .child(header)
        .child(body)
}

fn header(env: &ResultsEnv, tab: Tab, on_friend_requests: &Rc<dyn Fn()>) -> Node {
    let title = div()
        .class(cls::TITLE_ROW)
        .child(icon(icons::INBOX, TEXT_DEFAULT).class(cls::TITLE_ICON))
        .child(
            TextProps::styled(
                requiem_i18n::t!("inbox-title"),
                TextVariant::TEXT_LG_SEMIBOLD,
            )
            .color(TEXT_STRONG)
            .line_clamp(1_u32)
            .build()
            .class(cls::TITLE),
        )
        .child(
            div()
                .class(cls::CONTROLS)
                .child(tab_action(env, tab))
                .child(friend_requests_button(on_friend_requests)),
        );

    let tabs = tab_bar(
        TabBarProps::default()
            .items(vec![
                TabItem::new(Tab::Unreads.id(), requiem_i18n::t!("inbox-unreads")),
                TabItem::new(Tab::Mentions.id(), requiem_i18n::t!("inbox-mentions")),
            ])
            .selected(tab.id().to_string())
            .on_select(Rc::new(|id: String| {
                SCROLL.with(|s| s.set((0.0, 0.0)));
                TAB.with(|tab| tab.set(Tab::from_id(&id)));
            }) as Rc<dyn Fn(String)>)
            .fill(true),
    )
    .class(cls::TABS);
    div()
        .class(cls::HEADER)
        .child(title)
        .child(div().class(cls::HEADER_TABS).child(tabs))
}

fn tab_action(env: &ResultsEnv, tab: Tab) -> Node {
    match tab {
        Tab::Unreads => {
            if !env.state.read_states.has_unread_channels() {
                return div();
            }
            let button = ButtonProps::default()
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Small)
                .icon(icons::CHECKMARK_LARGE)
                .on_click(|| CONFIRM.with(|c| c.set(ToggleState::new(false).set(true))))
                .build();
            tooltip(
                button,
                TooltipProps::default()
                    .text(requiem_i18n::t!("inbox-mark-all-as-read"))
                    .position(TooltipPosition::Top),
            )
        }
        Tab::Mentions => {
            let button = ButtonProps::default()
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Small)
                .icon(icons::FILTERS_HORIZONTAL)
                .on_click_at(|rect: AbsoluteRect| {
                    context_menu::open(
                        rect.x + rect.width - 220.0,
                        rect.y + rect.height + 4.0,
                        filter_items,
                    )
                })
                .build();
            tooltip(
                button,
                TooltipProps::default()
                    .text(requiem_i18n::t!("inbox-filter"))
                    .position(TooltipPosition::Top),
            )
        }
    }
}

fn filter_items() -> Vec<ContextItem> {
    let filter = FILTER.with(Signal::get);
    let toggle = |label: String, on: bool, set: fn(&mut Filter, bool)| {
        ContextItem::action(label, move || {
            FILTER.with(|f| f.update(|f| set(f, !on)));

            PAGES.with(|pages| pages.set(vec![None]));
        })
        .checkbox(on)
    };
    vec![
        toggle(
            requiem_i18n::t!("inbox-include-everyone"),
            filter.everyone,
            |f, v| f.everyone = v,
        ),
        toggle(
            requiem_i18n::t!("inbox-include-roles"),
            filter.roles,
            |f, v| f.roles = v,
        ),
        toggle(
            requiem_i18n::t!("inbox-include-all-servers"),
            filter.all_servers,
            |f, v| f.all_servers = v,
        ),
    ]
}

fn friend_requests_button(on_friend_requests: &Rc<dyn Fn()>) -> Node {
    let open = on_friend_requests.clone();
    let button = ButtonProps::default()
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Small)
        .icon(icons::FRIENDS)
        .on_click(move || {
            close();
            open();
        })
        .build();
    tooltip(
        button,
        TooltipProps::default()
            .text(requiem_i18n::t!("inbox-view-friend-requests"))
            .position(TooltipPosition::Top),
    )
}

fn scroller(keys: &[u64], fallback: f32, on_near_end: Option<Box<dyn Fn()>>) -> Node {
    let (mut offset, _) = SCROLL.with(Cell::get);
    if let Some((key, from_top)) = ANCHORED.with(Cell::get) {
        if let Some(top) = top_of(keys, key, fallback) {
            offset = (top - from_top).max(0.0);
            SCROLL.with(|scroll| scroll.set((offset, scroll.get().1)));
        }
    }
    let watched = keys.to_vec();
    let on_scroll = move |offset: f32, max: f32| {
        if scroll_animation::is_programmatic() {
            return;
        }
        let (seen, _) = SCROLL.with(Cell::get);
        SCROLL.with(|s| s.set((offset, max)));
        ANCHORED.with(|anchor| anchor.set(card_at(&watched, offset, fallback)));
        if (offset - seen).abs() > WINDOW_EDGE {
            WINDOW.with(|tick| tick.update(|n| *n += 1));
        }
        if let Some(load) = &on_near_end {
            if max - offset < LOAD_AHEAD {
                load();
            }
        }
    };
    scroll_view_with(ScrollViewProps {
        initial_offset_y: offset,
        on_scroll: on_scroll.into(),
        ..Default::default()
    })
    .dom_id("inbox-scroll")
    .class(cls::SCROLL)
}

fn top_of(keys: &[u64], key: u64, fallback: f32) -> Option<f32> {
    let mut y = 0.0f32;
    for current in keys {
        if *current == key {
            return Some(y);
        }
        y += HEIGHTS.with(|heights| heights.borrow().get(current).copied().unwrap_or(fallback));
    }
    None
}

fn card_at(keys: &[u64], offset: f32, fallback: f32) -> Option<(u64, f32)> {
    let mut y = 0.0f32;
    for key in keys {
        let row = HEIGHTS.with(|heights| heights.borrow().get(key).copied().unwrap_or(fallback));
        if y + row > offset {
            return Some((*key, offset - y));
        }
        y += row;
    }
    None
}

fn list_window(keys: &[u64], fallback: f32) -> (usize, usize, f32, f32) {
    let _ = WINDOW.with(Signal::get);
    let (offset, _) = SCROLL.with(Cell::get);
    let heights = HEIGHTS.with(|heights| heights.borrow().clone());
    let height = |key: u64| heights.get(&key).copied().unwrap_or(fallback);
    let start_at = (offset - WINDOW_OVERSCAN).max(0.0);
    let end_at =
        offset + panel_max_height(app::current_window_size().1) + WINDOW_OVERSCAN;
    let (mut y, mut start, mut end, mut lead) = (0.0f32, 0usize, keys.len(), 0.0f32);
    for (index, key) in keys.iter().copied().enumerate() {
        let row = height(key);
        let bottom = y + row;
        if bottom < start_at {
            start = index + 1;
            lead = bottom;
        }
        if y > end_at && end == keys.len() {
            end = index;
        }
        y = bottom;
    }
    start = start.min(keys.len());
    end = end.clamp(start, keys.len());
    let through_end = keys[..end].iter().copied().map(height).sum::<f32>();
    (start, end, lead, y - through_end)
}

fn spacer(height: f32) -> Node {
    div().style(format!("height: {height}px; flex-shrink: 0"))
}

fn remember_height(key: u64) -> (Signal<(f32, f32)>, Node) {
    let size = signal((0.0f32, 0.0f32));
    let watched = size.clone();
    let watcher = Node::reactive(move || {
        let height = watched.get().1;
        if height > 1.0 {
            let changed = HEIGHTS.with(|heights| {
                let mut heights = heights.borrow_mut();
                let previous = heights.insert(key, height);
                previous.is_none_or(|previous| (previous - height).abs() > 8.0)
            });
            if changed {
                WINDOW.with(|tick| tick.update(|n| *n += 1));
            }
        }
        div()
    });
    (size, watcher)
}

fn content_width() -> f32 {
    let (window_w, _) = app::current_window_size();
    panel_width(window_w) - 32.0 - 2.0 - 72.0 - 16.0
}

fn unreads(env: &ResultsEnv) -> Node {
    let _ = env.state.read_states.marker_version().get();
    let _ = env.state.read_states.total_mentions().get();
    let channels = env.state.read_states.unread_channels();
    if channels.is_empty() {
        return empty_state(
            icons::INBOX,
            &requiem_i18n::t!("inbox-all-caught-up"),
            &requiem_i18n::t!("inbox-tip-keys"),
        );
    }
    let mut list = div().class(cls::LIST);
    let keys: Vec<u64> = channels.iter().map(|id| id.get()).collect();
    let (start, end, lead, tail) = list_window(&keys, CARD_HEIGHT);
    if lead > 0.0 {
        list = list.child(spacer(lead));
    }
    for (index, channel_id) in channels
        .into_iter()
        .enumerate()
        .skip(start)
        .take(end - start)
    {
        if let Some(card) = unread_card(env, channel_id, index < start + EAGER_CARDS) {
            let (size, watcher) = remember_height(channel_id.get());
            list = list.child(card.observe_size(size).child(watcher));
        }
    }
    if tail > 0.0 {
        list = list.child(spacer(tail));
    }
    scroller(&keys, CARD_HEIGHT, None).child(list)
}

struct Place {
    guild: Option<Guild>,
    category: Option<String>,
    channel: Channel,
}

fn find(env: &ResultsEnv, channel_id: Id<ChannelMarker>) -> Option<Place> {
    if let Some((guild_id, channel)) = ui::find_channel(env.state, channel_id) {
        let category = channel
            .parent_id
            .and_then(|parent| ui::find_channel(env.state, parent))
            .and_then(|(_, category)| category.name);
        return Some(Place {
            guild: env.state.guilds.find(guild_id),
            category,
            channel,
        });
    }
    env.state.dms.find(channel_id).map(|dm| Place {
        guild: None,
        category: None,
        channel: dm.as_channel(),
    })
}

fn channel_label(env: &ResultsEnv, channel: &Channel) -> String {
    match env.state.dms.find(channel.id) {
        Some(dm) => home::dm_title(env.state, &dm),
        None => channel.name.clone().unwrap_or_default(),
    }
}

fn card_icon(env: &ResultsEnv, guild: Option<&Guild>) -> Node {
    let Some(guild) = guild else {
        return div()
            .class(cls::CARD_ICON)
            .child(icon(icons::AT, TEXT_DEFAULT).class(cls::CARD_ICON_GLYPH));
    };
    match guild.icon.as_deref().and_then(|hash| {
        env.images
            .get_or_decode_image(&env.backend.guild_icon_url(guild.id, hash))
    }) {
        Some(image) => Node::new("div", ImageWidget::new(Some(image))).class(cls::CARD_ICON),
        None => div().class(cls::CARD_ICON).child(
            TextProps::styled(
                guild::acronym(&guild.name),
                TextVariant::TEXT_XS_MEDIUM,
            )
            .color(TEXT_DEFAULT)
            .center(true)
            .line_clamp(1_u32)
            .build(),
        ),
    }
}

fn card_header(
    env: &ResultsEnv,
    place: &Place,
    pings: u32,
    folded: bool,
    on_jump: impl Fn() + 'static,
    actions: Vec<Node>,
) -> Node {
    let mut header = div().class(cls::CARD_HEADER);
    if folded {
        header = header.class(cls::CARD_HEADER_FOLDED);
    }

    let mut link = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::CARD_LINK.to_string()])
        .on_click(on_jump)
        .build();

    let server_icon = card_icon(env, place.guild.as_ref());
    link = link.child(match &place.guild {
        Some(guild) => tooltip(
            server_icon,
            TooltipProps::default()
                .text(guild.name.clone())
                .position(TooltipPosition::Top),
        ),
        None => server_icon,
    });
    if place.guild.is_some() {
        link = link.child(icon(icons::CHEVRON_SMALL_RIGHT, TEXT_SUBTLE).class(cls::CARD_CHEVRON));
    }

    let mut title = div().class(cls::CARD_TITLE);
    if place.guild.is_some() {
        title = title.child(
            icon(
                ui::channel_icon(env.state, &place.channel),
                TEXT_SUBTLE,
            )
            .class(cls::CARD_HASH),
        );
    }
    title = title.child(
        TextProps::styled(
            channel_label(env, &place.channel),
            TextVariant::TEXT_MD_SEMIBOLD,
        )
        .line_height(20.0_f32)
        .color(TEXT_DEFAULT)
        .line_clamp(1_u32)
        .build()
        .class(cls::CARD_CHANNEL_NAME),
    );
    if pings > 0 {
        title = title.child(
            div()
                .class(cls::CARD_BADGE)
                .child(number_badge(NumberBadgeProps::new(pings))),
        );
    }
    let mut name = div().class(cls::CARD_NAME).child(title);
    if let Some(guild) = &place.guild {
        let breadcrumb = match &place.category {
            Some(category) => format!("{} \u{203a} {category}", guild.name),
            None => guild.name.clone(),
        };
        name = name.child(
            TextProps::styled(breadcrumb, TextVariant::TEXT_XS_NORMAL)
                .line_height(16.0_f32)
                .color(TEXT_DEFAULT)
                .line_clamp(1_u32)
                .build()
                .class(cls::CARD_BREADCRUMB),
        );
    }
    header
        .child(link.child(name))
        .child(div().class(cls::CARD_ACTIONS).children(actions))
}

fn unread_card(env: &ResultsEnv, channel_id: Id<ChannelMarker>, fetch: bool) -> Option<Node> {
    let place = find(env, channel_id)?;
    let id = channel_id.get();
    let collapsed = COLLAPSED.with(Signal::get).contains(&id);

    let folding = is_folding(id);
    let show_messages = !collapsed || folding;
    let state = env.state.clone();

    let acked = env.state.read_states.acked_through(channel_id);
    let key = format!("inbox:unread:{channel_id}:{acked}");
    let (http, queries) = (env.http.cloned(), env.queries.clone());
    let result: Option<query::QueryResult<Vec<Message>>> = (fetch && show_messages)
        .then_some(http)
        .flatten()
        .map(|http| {
            queries.query(&key, STALE, move || async move {
                get_channel_messages_after(&http, channel_id, Id::new(acked), CARD_MESSAGES).await
            })
        });
    let messages = result.as_ref().and_then(|result| result.data.clone());
    let mut ordered: Vec<&Message> = messages
        .as_ref()
        .map(|messages| messages.iter().collect())
        .unwrap_or_default();
    ordered.sort_by_key(|m| m.id.get());

    let newest = env.state.messages.by_channel.with_untracked(|all| {
        all.get(&channel_id)
            .and_then(|list| list.last().map(|m| m.id))
    });
    let target = ordered.first().map(|m| m.id).or(newest);
    let jump = env.search.on_jump.clone();
    let jump_to = move || {
        if let Some(target) = target {
            close();
            (jump.borrow())(channel_id, target);
        }
    };
    let actions = vec![
        tooltip(
            ButtonProps::default()
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Small)
                .icon(icons::CHECKMARK_LARGE)
                .on_click(move || state.read_states.mark_channel_read(channel_id))
                .build(),
            TooltipProps::default()
                .text(requiem_i18n::t!("inbox-mark-as-read"))
                .position(TooltipPosition::Top),
        ),
        tooltip(
            ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![cls::COLLAPSE.to_string()])
                .on_click(move || toggle_fold(id))
                .build()
                .child(
                    icon(
                        if collapsed {
                            icons::CHEVRON_SMALL_UP
                        } else {
                            icons::CHEVRON_SMALL_DOWN
                        },
                        TEXT_SUBTLE,
                    )
                    .class(cls::COLLAPSE_ICON),
                ),
            TooltipProps::default()
                .text(if collapsed {
                    requiem_i18n::t!("inbox-expand")
                } else {
                    requiem_i18n::t!("inbox-collapse")
                })
                .position(TooltipPosition::Top),
        ),
    ];
    let header = card_header(
        env,
        &place,
        env.state.read_states.mention_count(channel_id),
        collapsed && !folding,
        jump_to,
        actions,
    );
    let mut card = div().class(cls::CARD).child(header);
    if !show_messages {
        return Some(card);
    }
    let Some(result) = result else {
        return Some(card);
    };
    if result.data.is_none() {
        let _ = env.tick.get();
        return Some(card.child(div().class(cls::LOADING).child(Node::new(
            "div",
            PulsingEllipsisWidget::new(TEXT_DEFAULT, 1.0),
        ))));
    }

    let width = content_width();
    let mut list = div().class(cls::MESSAGES);
    let mut previous: Option<&Message> = None;
    for message in ordered {
        env.state.users.observe_messages([message]);

        let compact = !message.is_reply()
            && previous.is_some_and(|before| !message.starts_group_after(before));
        previous = Some(message);

        let (jump, message_id) = (env.search.on_jump.clone(), message.id);
        let jump_button = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::MESSAGE_JUMP.to_string()])
            .on_click(move || {
                close();
                (jump.borrow())(channel_id, message_id);
            })
            .build()
            .child(
                TextProps::styled(requiem_i18n::t!("search-jump"), TextVariant::TEXT_XS_MEDIUM)
                    .line_height(16.0_f32)
                    .color(TEXT_DEFAULT)
                    .line_clamp(1_u32)
                    .build(),
            );
        let mut row = div().class(cls::MESSAGE);
        if !compact {
            row = row.class(cls::MESSAGE_START);
        }
        list = list.child(
            row.child(search::message_body(
                env,
                Some(&place.channel),
                message,
                &[],
                width,
                compact,
            ))
            .child(jump_button),
        );
    }
    let (style, size) = fold_signals(id);
    card = card.child(
        div()
            .class(cls::CLIP)
            .bind_style(style)
            .child(list.observe_size(size)),
    );
    Some(card)
}

fn mentions(env: &ResultsEnv) -> Node {
    let filter = FILTER.with(Signal::get);
    let pages = PAGES.with(Signal::get);

    let guild = (!filter.all_servers)
        .then(|| env.state.read_states.viewing_guild())
        .flatten()
        .map(|id| id.get());

    let mut all: Vec<Message> = Vec::new();
    let mut loading = false;
    let mut failed: Option<String> = None;
    let mut more = false;

    let mut cursor: Option<Id<MessageMarker>> = None;
    for (index, before) in pages.iter().enumerate() {
        let key = mentions_key(filter, *before, guild);
        let Some(http) = env.http.cloned() else { break };
        let queries = env.queries.clone();
        let fetch_filter = MentionsFilter {
            guild_id: guild.map(Id::new),
            roles: filter.roles,
            everyone: filter.everyone,
        };
        let before = *before;
        let page: query::QueryResult<Vec<Message>> =
            queries.query(&key, STALE, move || async move {
                recent_mentions(&http, fetch_filter, before).await
            });
        match page.data {
            Some(messages) => {
                all.extend(messages.iter().cloned());
                cursor = messages.last().map(|m| m.id).or(cursor);
                more = index + 1 == pages.len() && messages.len() >= MENTIONS_PAGE as usize;
            }
            None => {
                if page.error.is_some() {
                    failed = Some(key);
                } else {
                    loading = true;
                }
                break;
            }
        }
    }
    if loading {
        let _ = env.tick.get();
    }

    let _ = env.state.read_states.marker_version().get();
    let me = env
        .state
        .users
        .current
        .with_untracked(|user| user.as_ref().map(|user| user.id));
    for message in env.state.read_states.live_mentions() {
        let direct = me.is_some_and(|me| message.mentions.iter().any(|user| user.id == me));
        let wanted = direct
            || (message.mention_everyone && filter.everyone)
            || (!message.mention_roles.is_empty() && filter.roles);
        let in_scope = guild.is_none_or(|guild| {
            find(env, message.channel_id)
                .and_then(|place| place.guild)
                .is_some_and(|g| g.id.get() == guild)
        });
        if wanted && in_scope && !all.iter().any(|known| known.id == message.id) {
            all.push(message);
        }
    }
    all.sort_by_key(|message| cmp::Reverse(message.id.get()));
    let dismissed = DISMISSED.with(|d| d.borrow().clone());
    all.retain(|message| !dismissed.contains(&message.id.get()));
    if all.is_empty() && !loading && failed.is_none() {
        return empty_state(
            icons::AT,
            &requiem_i18n::t!("inbox-made-it-through"),
            &requiem_i18n::t!("inbox-mentions-tip"),
        );
    }

    let width = content_width();
    let mut list = div().class(cls::LIST);
    let keys: Vec<u64> = all.iter().map(|message| message.id.get()).collect();
    let (start, end, lead, tail) = list_window(&keys, MENTION_HEIGHT);
    if lead > 0.0 {
        list = list.child(spacer(lead));
    }
    for message in all.iter().skip(start).take(end - start) {
        env.state.users.observe_messages([message]);
        let Some(place) = find(env, message.channel_id) else {
            continue;
        };
        let (jump, channel_id, message_id) =
            (env.search.on_jump.clone(), message.channel_id, message.id);
        let jump_to: Rc<dyn Fn()> = Rc::new(move || {
            close();
            (jump.borrow())(channel_id, message_id);
        });
        let actions = vec![
            tooltip(
                ButtonProps::default()
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Small)
                    .icon(icons::SCREEN_ARROW)
                    .on_click({
                        let jump_to = jump_to.clone();
                        move || jump_to()
                    })
                    .build(),
                TooltipProps::default()
                    .text(requiem_i18n::t!("inbox-jump-to-message"))
                    .position(TooltipPosition::Top),
            ),
            tooltip(
                ButtonProps::default()
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Small)
                    .icon(icons::CHECKMARK_LARGE)
                    .on_click(move || {
                        DISMISSED.with(|d| d.borrow_mut().insert(message_id.get()));

                        PAGES.with(|pages| pages.update(|_| {}));
                    })
                    .build(),
                TooltipProps::default()
                    .text(requiem_i18n::t!("inbox-mark-as-read"))
                    .position(TooltipPosition::Top),
            ),
        ];
        let header = card_header(env, &place, 0, false, move || jump_to(), actions);
        let body = div()
            .class(cls::MENTION)
            .child(search::message_body(
                env,
                Some(&place.channel),
                message,
                &[],
                width,
                false,
            ));
        list = list.child({
            let (size, watcher) = remember_height(message.id.get());
            div()
                .class(cls::CARD)
                .child(header)
                .child(body)
                .observe_size(size)
                .child(watcher)
        });
    }
    if tail > 0.0 {
        list = list.child(spacer(tail));
    }
    if loading {
        list = list.child(div().class(cls::LOADING).child(Node::new(
            "div",
            PulsingEllipsisWidget::new(TEXT_DEFAULT, 1.0),
        )));
    } else if let Some(key) = failed {
        let queries = env.queries.clone();
        list = list.child(
            ButtonProps::default()
                .variant(ButtonVariant::Secondary)
                .label(requiem_i18n::t!("inbox-load-more"))
                .on_click(move || queries.invalidate(&key))
                .build()
                .class(cls::LOAD_MORE),
        );
    }
    let near_end: Option<Box<dyn Fn()>> = cursor.filter(|_| more).map(|cursor| {
        Box::new(move || {
            PAGES.with(|pages| {
                if pages.peek().last().copied() != Some(Some(cursor)) {
                    pages.update(|pages| pages.push(Some(cursor)));
                }
            })
        }) as Box<dyn Fn()>
    });
    scroller(&keys, MENTION_HEIGHT, near_end).child(list)
}

fn empty_state(glyph: Icon, heading: &str, tip: &str) -> Node {
    div()
        .class(cls::EMPTY)
        .child(
            div()
                .class(cls::EMPTY_ICON)
                .child(icon(glyph, TEXT_DEFAULT).class(cls::EMPTY_GLYPH)),
        )
        .child(
            TextProps::styled(heading.to_string(), TextVariant::HEADING_XL_SEMIBOLD)
                .color(TEXT_STRONG)
                .center(true)
                .build()
                .class(cls::EMPTY_HEADING),
        )
        .child(
            div()
                .class(cls::EMPTY_TIP)
                .child(
                    TextProps::styled(
                        format!("{}: ", requiem_i18n::t!("inbox-protip")),
                        TextVariant::TEXT_XS_BOLD,
                    )
                    .color(colors::TEXT_POSITIVE)
                    .build(),
                )
                .child(
                    TextProps::styled(tip.to_string(), TextVariant::TEXT_XS_NORMAL)
                        .color(TEXT_DEFAULT)
                        .build(),
                ),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_is_35vw_wide_between_480_and_600() {
        let widths = [800.0, 1604.0, 4000.0].map(panel_width);
        assert_eq!(widths[0], 480.0);
        assert!((widths[1] - 561.4).abs() < 0.01, "{}", widths[1]);
        assert_eq!(widths[2], 600.0);
    }

    #[test]
    fn the_panel_is_at_most_70_percent_tall_and_more_in_short_windows() {
        assert_eq!(panel_max_height(1000.0), 700.0);
        assert_eq!(panel_max_height(800.0), 640.0);
        assert_eq!(panel_max_height(700.0), 595.0);
    }

    #[test]
    fn a_different_filter_or_page_is_a_different_fetch() {
        let on = Filter {
            all_servers: true,
            roles: true,
            everyone: true,
        };
        assert_ne!(
            mentions_key(on, None, None),
            mentions_key(
                Filter {
                    everyone: false,
                    ..on
                },
                None,
                None
            )
        );
        assert_ne!(
            mentions_key(on, None, None),
            mentions_key(on, Some(Id::new(5)), None)
        );
        assert_ne!(
            mentions_key(on, None, None),
            mentions_key(on, None, Some(9))
        );
    }

    #[test]
    fn tabs_round_trip_through_their_ids() {
        for tab in [Tab::Unreads, Tab::Mentions] {
            assert_eq!(Tab::from_id(tab.id()), tab);
        }
    }
}
