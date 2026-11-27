use crate::gateway::events::GatewayEvent;
use crate::gateway::events;
use crate::ui::context_menu;
use crate::ui::playground;
use crate::utils::format;
use crate::utils::time;
use requiem_ui::app;
use requiem_ui::clipboard;
use requiem_ui::colors::{TEXT_CRITICAL, TEXT_DEFAULT, TEXT_SECONDARY, TEXT_STRONG, TEXT_SUBTLE};
use requiem_ui::components::ButtonVariant;
use requiem_ui::components::TextVariant;
use requiem_ui::components::toast::ToastKind;
use requiem_ui::components::{Align, Justify, Padding, StackProps};
use requiem_ui::components::{MenuItem, menu, popover_backdrop};
use requiem_ui::icons::IconPath;
use requiem_ui::paint as requiem_ui_paint;
use requiem_ui::perf;
use requiem_ui::style;
use requiem_ui::timer;
use std::cell::Cell;
use std::collections::HashSet;
use std::collections;
use std::env;
use std::rc::Rc;
use std::time::Duration;

use requiem_ui::components::{
    ButtonProps, ScrollViewProps, SwitchProps, TextProps, div, icon, scroll_view_with, search_bar,
    switch, text,
};
use requiem_ui::control::section;
use requiem_ui::event::{CursorIcon, EventResult, MouseButton, UiEvent};
use requiem_ui::inspect::{self, Details, NodeEntry, Snapshot};
use requiem_ui::node::Node;
use requiem_ui::popout::{self, PopoutSpec};
use requiem_ui::signal::{Effect, Signal, signal};
use requiem_ui::widget::anim::ToggleState;
use requiem_ui::widget::{AbsoluteRect, Widget, WidgetId, WidgetTree};
use serde_json::Value;
use skia_safe::Color;
use skia_safe::font_style::Weight;

use crate::netlog::{self, NetEntry};
use crate::state::AppState;
use crate::ui::Fonts;
use crate::ui::classes::devtools as css;
use requiem_ui::icons::{
    CHEVRON_SMALL_DOWN, CHEVRON_SMALL_RIGHT, DOUBLE_CHEVRON_SMALL_RIGHT, EYE_DROPPER, REFRESH,
    STAFF_BADGE, WINDOW_LAUNCH, X_SMALL,
};

const TABS: [&str; 9] = [
    "Elements",
    "Layers",
    "Stores",
    "Dispatcher",
    "Network",
    "Performance",
    "Colors",
    "Triggers",
    "Overrides",
];
const DETAIL_TABS: [&str; 4] = ["Styles", "Computed", "Layout", "Props"];

const MIN_WIDTH: f32 = 320.0;
const DEFAULT_WIDTH: f32 = 460.0;

const LOG_LIMIT: usize = 400;

const ROW_LIMIT: usize = 1500;

const POLL: Duration = Duration::from_millis(500);

const JSON_LIMIT: usize = 12_000;

const REPLAY_LIMIT: usize = 256 * 1024;

const TAG: Color = Color::from_argb(255, 0x7a, 0xa2, 0xf7);
const CLASS: Color = Color::from_argb(255, 0xe0, 0xaf, 0x68);
const DOM_ID: Color = Color::from_argb(255, 0x9e, 0xce, 0x6a);

#[derive(Clone)]
struct Dispatch {
    seq: u64,

    at_ms: u64,
    name: String,

    handled: bool,
    bytes: usize,
    json: String,

    value: Option<Value>,
}

thread_local! {
    static OPEN: Signal<bool> = signal(false);

    static POPPED: Signal<Option<u64>> = signal(None);

    static DOCKING: Cell<bool> = const { Cell::new(false) };
    static WIDTH: Signal<f32> = signal(DEFAULT_WIDTH);
    static WIDTH_STYLE: Signal<String> = signal(format!("width: {DEFAULT_WIDTH}px"));
    static TAB: Signal<&'static str> = signal(TABS[0]);
    static DETAIL_TAB: Signal<&'static str> = signal(DETAIL_TABS[0]);
    static EXPANDED: Signal<HashSet<u64>> = signal(HashSet::new());
    static FILTER: Signal<String> = signal(String::new());
    static EXPANDED_SEEDED: Cell<bool> = const { Cell::new(false) };
    static TREE_SCROLL: Cell<f32> = const { Cell::new(0.0) };
    static POLL_GENERATION: Cell<u64> = const { Cell::new(0) };

    static LOG: Signal<Vec<Dispatch>> = signal(Vec::new());
    static LOG_SEQ: Cell<u64> = const { Cell::new(0) };
    static LOG_PAUSED: Signal<bool> = signal(false);
    static LOG_EXPANDED: Signal<HashSet<u64>> = signal(HashSet::new());
    static STORE: Signal<&'static str> = signal("UserStore");
    static STORE_FILTER: Signal<String> = signal(String::new());
    static NET: Signal<Vec<NetEntry>> = signal(Vec::new());
    static NET_SEEN: Cell<u64> = const { Cell::new(u64::MAX) };
    static NET_FILTER: Signal<String> = signal(String::new());
    static NET_OPEN: Signal<HashSet<u64>> = signal(HashSet::new());

    static PERF_TICK: Signal<u64> = signal(0);
    static COLOR_FILTER: Signal<String> = signal(String::new());

    static TABS_WIDTH: Signal<f32> = signal(DEFAULT_WIDTH);
    static OVERFLOW_OPEN: Signal<bool> = signal(false);

    static SPOOF_NITRO: Signal<ToggleState> = signal(ToggleState::new(false));
    static OUTLINES: Signal<ToggleState> = signal(ToggleState::new(false));
}

pub fn spoof_nitro() -> bool {
    SPOOF_NITRO.with(|s| s.get().on)
}

pub fn init_from_env() {
    if env::var_os("REQUIEM_SPOOF_NITRO").is_some_and(|v| v == "1") {
        SPOOF_NITRO.with(|s| s.set(ToggleState::new(true)));
    }
}

pub fn is_open() -> bool {
    OPEN.with(|o| o.peek())
}

pub fn toggle(ctx: &DevtoolsContext) {
    if is_open() {
        close();
    } else {
        open(ctx);
    }
}

pub fn open(_ctx: &DevtoolsContext) {
    OPEN.with(|o| o.set(true));
    events::set_capture_all(true);
    start_polling();
}

pub fn close() {
    if let Some(id) = POPPED.with(|p| p.peek()) {
        popout::close(id);
    }
    inspect::picking().set(false);
    inspect::set_highlight(None);
    OPEN.with(|o| o.set(false));
    events::set_capture_all(false);
}

#[derive(Clone)]
pub struct DevtoolsContext {
    pub fonts: Fonts,
    pub state: AppState,
}

pub fn record_dispatch(event: &GatewayEvent, handled: bool) {
    if !is_open() || LOG_PAUSED.with(|p| p.peek()) {
        return;
    }
    let (name, data) = (event.name.as_str(), event.value());
    let mut json = serde_json::to_string_pretty(data).unwrap_or_default();
    let bytes = json.len();
    if json.len() > JSON_LIMIT {
        json = truncate_json(json);
    }
    let seq = LOG_SEQ.with(|s| s.replace(s.get() + 1));
    let at_ms = time::time_of_day_ms();
    LOG.with(|log| {
        log.update(|log| {
            log.push(Dispatch {
                seq,
                at_ms,
                name: name.to_string(),
                handled,
                bytes,
                json,
                value: (bytes <= REPLAY_LIMIT).then(|| data.clone()),
            });
            if log.len() > LOG_LIMIT {
                log.remove(0);
            }
        })
    });
}

fn truncate_json(mut json: String) -> String {
    let mut cut = JSON_LIMIT;
    while !json.is_char_boundary(cut) {
        cut -= 1;
    }
    json.truncate(cut);
    json.push_str("\n… (truncated)");
    json
}

pub fn mount(tree: &mut WidgetTree, parent: WidgetId, ctx: &DevtoolsContext) -> Vec<Effect> {
    let ctx = ctx.clone();
    vec![section(tree, parent, move || {
        let docked = OPEN.with(Signal::get) && POPPED.with(Signal::get).is_none();
        if !docked {
            return div();
        }
        start_polling();
        let panel = panel(&ctx, false);
        let handle = Node::new(
            "div",
            ResizeHandle {
                dragging: false,
                start: (0.0, 0.0),
            },
        )
        .class(css::RESIZE_HANDLE);

        div()
            .class(css::HOST)
            .dom_id(inspect::EXCLUDED_DOM_ID)
            .bind_style(WIDTH_STYLE.with(Clone::clone))
            .child(panel)
            .child(handle)
    })]
}

fn pop_out(ctx: &DevtoolsContext) {
    if POPPED.with(|p| p.peek()).is_some() {
        return;
    }

    inspect::set_highlight(None);
    let build_ctx = ctx.clone();
    let id = popout::open(PopoutSpec {
        title: "DevTools".to_string(),
        width: 760.0,
        height: 720.0,
        build: Rc::new(move || panel(&build_ctx, true)),
        on_close: Some(Box::new(|| {
            POPPED.with(|p| p.set(None));
            if !DOCKING.with(Cell::get) {
                inspect::picking().set(false);
                inspect::set_highlight(None);
                OPEN.with(|o| o.set(false));
                events::set_capture_all(false);
            }
            DOCKING.with(|d| d.set(false));
        })),
    });
    POPPED.with(|p| p.set(Some(id)));
}

fn dock() {
    inspect::set_highlight(None);
    if let Some(id) = POPPED.with(|p| p.peek()) {
        DOCKING.with(|d| d.set(true));
        popout::close(id);
    }
}

struct ResizeHandle {
    dragging: bool,

    start: (f32, f32),
}

impl Widget for ResizeHandle {
    fn paint(
        &self,
        canvas: &skia_safe::Canvas,
        rect: AbsoluteRect,
        _style: &style::ComputedStyle,
    ) {
        let mut paint = skia_safe::Paint::default();
        paint.set_color(Color::from_rgb(0x12, 0x12, 0x14));
        canvas.draw_rect(
            skia_safe::Rect::from_xywh(rect.x, rect.y, 1.0, rect.height),
            &paint,
        );
    }

    fn handle_event(&mut self, event: &UiEvent, _rect: AbsoluteRect) -> EventResult {
        match *event {
            UiEvent::MouseDown {
                x,
                button: MouseButton::Left,
                ..
            } => {
                self.dragging = true;
                self.start = (x, WIDTH.with(|w| w.peek()));
                EventResult::Handled
            }
            UiEvent::MouseMoved { x, .. } if self.dragging => {
                let max = (app::current_window_size().0 - 280.0).max(MIN_WIDTH);
                let width = (self.start.1 + (self.start.0 - x)).clamp(MIN_WIDTH, max);
                WIDTH.with(|w| w.set(width));
                WIDTH_STYLE.with(|s| s.set(format!("width: {width}px")));
                EventResult::Handled
            }
            UiEvent::MouseUp { .. } if self.dragging => {
                self.dragging = false;
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }

    fn wants_pointer_capture(&self) -> bool {
        self.dragging
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        Some(CursorIcon::ColResize)
    }
}

fn icon_button(glyph: impl IconPath, color: Color, on_click: impl Fn() + 'static) -> Node {
    ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![css::ICON_BUTTON.to_string()])
        .on_click(on_click)
        .build()
        .child(icon(glyph, color).class(css::ICON))
}

fn panel(ctx: &DevtoolsContext, popped: bool) -> Node {
    let title = StackProps::horizontal()
        .align(Align::Center)
        .full_width(false)
        .build()
        .child(icon(STAFF_BADGE, TEXT_STRONG).class(css::TITLE_ICON))
        .child(
            TextProps::styled("DevTools", TextVariant::TEXT_MD_MEDIUM)
                .color(TEXT_STRONG)
                .line_height((16.0_f32) * 1.25_f32)
                .line_clamp(1_u32)
                .build(),
        );

    let pop_ctx = ctx.clone();
    let move_button = if popped {
        icon_button(WINDOW_LAUNCH, TEXT_SECONDARY, dock)
    } else {
        icon_button(WINDOW_LAUNCH, TEXT_SECONDARY, move || pop_out(&pop_ctx))
    };
    let toolbar = StackProps::horizontal()
        .align(Align::Center)
        .full_width(false)
        .build()
        .child(move_button)
        .child(icon_button(X_SMALL, TEXT_SECONDARY, close));
    let header = div().class(css::HEADER).child(title).child(toolbar);

    let tabs = Node::reactive(tab_row);

    let body_ctx = ctx.clone();
    let body = Node::reactive(move || match TAB.with(Signal::get) {
        "Elements" => elements_tab(),
        "Layers" => layers_tab(),
        "Stores" => stores_tab(&body_ctx),
        "Dispatcher" => dispatcher_tab(&body_ctx),
        "Network" => network_tab(),
        "Performance" => performance_tab(),
        "Colors" => colors_tab(),
        "Triggers" => triggers_tab(),
        _ => overrides_tab(),
    })
    .class(css::BODY_HOST);

    let mut root = div()
        .class(css::PANEL)
        .child(header)
        .child(Node::new("div", WidthProbe).class(css::TAB_BAR).child(tabs))
        .child(body)
        .child(overflow_menu());
    if popped {
        root = root.class(css::PANEL_POPPED);
    }
    root
}

struct WidthProbe;

impl Widget for WidthProbe {
    fn paint(
        &self,
        _canvas: &skia_safe::Canvas,
        rect: AbsoluteRect,
        _style: &style::ComputedStyle,
    ) {
        if (TABS_WIDTH.with(|w| w.peek()) - rect.width).abs() > 1.0 {
            TABS_WIDTH.with(|w| w.set(rect.width));
            requiem_ui_paint::request_repaint();
        }
    }
}

fn tab_width(name: &str) -> f32 {
    name.chars().count() as f32 * 7.6 + 24.0
}

const OVERFLOW_BUTTON_WIDTH: f32 = 23.0;

fn fit_tabs(width: f32, selected: &str) -> (Vec<&'static str>, Vec<&'static str>) {
    let budget = width - 8.0;
    let total: f32 = TABS.iter().map(|t| tab_width(t)).sum();
    if total <= budget {
        return (TABS.to_vec(), Vec::new());
    }
    let room = budget - OVERFLOW_BUTTON_WIDTH;
    let mut visible: Vec<&'static str> = Vec::new();
    let mut used = 0.0;
    for name in TABS {
        let w = tab_width(name);
        if used + w > room {
            break;
        }
        used += w;
        visible.push(name);
    }
    if !visible.contains(&selected) {
        let selected = *TABS.iter().find(|t| **t == selected).unwrap_or(&TABS[0]);
        while !visible.is_empty() && used + tab_width(selected) > room {
            used -= tab_width(visible.pop().unwrap_or(selected));
        }
        visible.push(selected);
    }
    let hidden = TABS
        .iter()
        .copied()
        .filter(|t| !visible.contains(t))
        .collect();
    (visible, hidden)
}

fn tab_row() -> Node {
    let current = TAB.with(Signal::get);
    let width = TABS_WIDTH.with(Signal::get);
    let (visible, hidden) = fit_tabs(width, current);

    let mut list = div().class(css::TAB_LIST);
    for name in &visible {
        let name = *name;
        let mut tab = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![css::TAB.to_string()])
            .on_click(move || {
                TAB.with(|t| t.set(name));
                start_polling();
            });
        if name == current {
            tab = tab.extra_classes(vec![css::TAB.to_string(), css::TAB_SELECTED.to_string()]);
        }
        list = list.child(
            tab.build().child(
                TextProps::styled(name, TextVariant::TEXT_SM_MEDIUM)
                    .color(if name == current {
                        TEXT_STRONG
                    } else {
                        TEXT_SECONDARY
                    })
                    .line_height((14.0_f32) * 1.25_f32)
                    .line_clamp(1_u32)
                    .build(),
            ),
        );
    }
    let bar = StackProps::horizontal()
        .gap(0)
        .padding(Padding::x(4))
        .build()
        .child(list);
    if hidden.is_empty() {
        return bar;
    }

    let toggle = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![css::OVERFLOW_BUTTON.to_string()])
        .on_click(|| OVERFLOW_OPEN.with(|o| o.set(!o.peek())))
        .build()
        .child(icon(DOUBLE_CHEVRON_SMALL_RIGHT, TEXT_SECONDARY).class(css::OVERFLOW_ICON));
    bar.child(div().class(css::OVERFLOW).child(toggle))
}

fn overflow_menu() -> Node {
    let open = OVERFLOW_OPEN.with(Clone::clone);
    Node::reactive(move || {
        if !OVERFLOW_OPEN.with(Signal::get) {
            return div();
        }
        let (_, hidden) = fit_tabs(TABS_WIDTH.with(Signal::get), TAB.with(Signal::get));
        if hidden.is_empty() {
            return div();
        }

        let items: Vec<MenuItem> = hidden
            .into_iter()
            .map(|name| {
                MenuItem::action(name, move || {
                    OVERFLOW_OPEN.with(|o| o.set(false));
                    TAB.with(|t| t.set(name));
                    start_polling();
                })
            })
            .collect();
        div().class(css::OVERFLOW_FILL).children([
            popover_backdrop(|| OVERFLOW_OPEN.with(|o| o.set(false))),
            div().class(css::OVERFLOW_PANEL).child(menu(items)),
        ])
    })
    .bind_class(open, css::OVERFLOW_LAYER_OPEN, css::OVERFLOW_LAYER_CLOSED)
}

fn start_polling() {
    let generation = POLL_GENERATION.with(|g| {
        g.set(g.get() + 1);
        g.get()
    });
    poll(generation);
}

fn poll(generation: u64) {
    if POLL_GENERATION.with(Cell::get) != generation || !is_open() {
        return;
    }
    refresh(TAB.with(|t| t.peek()));
    timer::set_timeout(POLL, move || poll(generation));
}

fn refresh(tab: &str) {
    match tab {
        "Elements" | "Layers" => {
            inspect::request_snapshot();
            if tab == "Elements" {
                if let Some(id) = inspect::selected().peek() {
                    inspect::request_details(id);
                }
            }
        }
        "Network" => {
            let version = netlog::version();
            if NET_SEEN.with(|s| s.replace(version)) != version {
                NET.with(|n| n.set(netlog::entries()));
            }
        }
        "Performance" => {
            inspect::request_stats();
            inspect::request_snapshot();
            PERF_TICK.with(|t| t.update(|t| *t += 1));
        }
        "Colors"
            if inspect::tokens().peek().is_empty() => {
                inspect::request_tokens();
            }
        _ => {}
    }
}

fn select(id: u64) {
    inspect::selected().set(Some(id));
    inspect::request_details(id);

    if let Some(snapshot) = inspect::snapshot().peek() {
        let by_id: collections::HashMap<u64, &NodeEntry> =
            snapshot.nodes.iter().map(|n| (n.id, n)).collect();
        let mut cursor = by_id.get(&id).and_then(|n| n.parent);
        EXPANDED.with(|e| {
            e.update(|set| {
                while let Some(parent) = cursor {
                    set.insert(parent);
                    cursor = by_id.get(&parent).and_then(|n| n.parent);
                }
            })
        });
    }
}

fn elements_tab() -> Node {
    let pick = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![css::ICON_BUTTON.to_string()])
        .on_click(|| {
            let next = !inspect::picking().peek();
            inspect::picking().set(next);
            if !next {
                inspect::set_highlight(None);
            }
        })
        .build()
        .bind_class(
            inspect::picking(),
            css::ICON_BUTTON_ON,
            css::ICON_BUTTON_OFF,
        )
        .child(icon(EYE_DROPPER, TEXT_SECONDARY).class(css::ICON));
    let refresh = icon_button(REFRESH, TEXT_SECONDARY, || {
        inspect::request_snapshot();
        if let Some(id) = inspect::selected().peek() {
            inspect::request_details(id);
        }
    });
    let filter = search_bar("Filter by tag, class, id or text", |query| {
        FILTER.with(|f| f.set(query.to_string()))
    })
    .class(css::FILTER);
    let count = Node::reactive(move || {
        TextProps::styled(
            format!("{} nodes", inspect::snapshot().get().map_or(0, |s| s.total)),
            TextVariant::TEXT_XS_NORMAL,
        )
        .color(TEXT_SUBTLE)
        .line_height((12.0_f32) * 1.25_f32)
        .line_clamp(1_u32)
        .build()
    });
    let toolbar = div()
        .class(css::ELEMENTS_TOOLBAR)
        .children([pick, refresh, filter, count]);

    let tree = Node::reactive(tree_view).class(css::TREE_HOST);
    let details = Node::reactive(details_pane).class(css::DETAILS_HOST);
    div()
        .class(css::ELEMENTS)
        .children([toolbar, tree, details])
}

fn visible_rows<'a>(
    snapshot: &'a Snapshot,
    expanded: &HashSet<u64>,
    filter: &str,
) -> Vec<&'a NodeEntry> {
    if filter.is_empty() {
        let mut out = Vec::new();
        let mut hidden_below: Option<u16> = None;
        for node in &snapshot.nodes {
            if let Some(depth) = hidden_below {
                if node.depth > depth {
                    continue;
                }
                hidden_below = None;
            }
            out.push(node);
            if node.child_count > 0 && !expanded.contains(&node.id) {
                hidden_below = Some(node.depth);
            }
        }
        return out;
    }
    let needle = filter.to_lowercase();
    let matches = |n: &NodeEntry| {
        n.tag.to_lowercase().contains(&needle)
            || n.widget.to_lowercase().contains(&needle)
            || n.dom_id
                .as_deref()
                .is_some_and(|d| d.to_lowercase().contains(&needle))
            || n.classes.iter().any(|c| c.to_lowercase().contains(&needle))
            || n.text
                .as_deref()
                .is_some_and(|t| t.to_lowercase().contains(&needle))
    };
    let mut keep: HashSet<u64> = HashSet::new();
    let by_id: collections::HashMap<u64, &NodeEntry> =
        snapshot.nodes.iter().map(|n| (n.id, n)).collect();
    for node in snapshot.nodes.iter().filter(|n| matches(n)) {
        let mut cursor = Some(node.id);
        while let Some(id) = cursor {
            if !keep.insert(id) {
                break;
            }
            cursor = by_id.get(&id).and_then(|n| n.parent);
        }
    }
    snapshot
        .nodes
        .iter()
        .filter(|n| keep.contains(&n.id))
        .collect()
}

fn tree_view() -> Node {
    let snapshot = inspect::snapshot().get();
    let expanded = EXPANDED.with(Signal::get);
    let selected = inspect::selected().get();
    let filter = FILTER.with(Signal::get);
    let Some(snapshot) = snapshot else {
        return div().class(css::EMPTY).child(
            TextProps::styled("Reading the widget tree…", TextVariant::TEXT_SM_NORMAL)
                .color(TEXT_SUBTLE)
                .line_height((14.0_f32) * 1.25_f32)
                .line_clamp(1_u32)
                .build(),
        );
    };

    let mut expanded = expanded;
    if !EXPANDED_SEEDED.with(|s| s.replace(true)) && !snapshot.nodes.is_empty() {
        let root = snapshot.nodes[0].id;
        expanded.insert(root);
        EXPANDED.with(|e| {
            e.update(|set| {
                set.insert(root);
            })
        });
    }
    let rows = visible_rows(&snapshot, &expanded, &filter);
    let mut list = StackProps::vertical()
        .gap(0)
        .full_width(false)
        .padding(Padding::bottom(8))
        .build();
    for node in rows.iter().take(ROW_LIMIT) {
        list = list.child(tree_row(
            node,
            &expanded,
            selected == Some(node.id),
            filter.is_empty(),
        ));
    }
    if rows.len() > ROW_LIMIT {
        list = list.child(
            div().class(css::EMPTY).child(
                TextProps::styled(
                    format!("{} more rows, narrow the filter", rows.len() - ROW_LIMIT),
                    TextVariant::TEXT_XS_NORMAL,
                )
                .color(TEXT_SUBTLE)
                .line_height((12.0_f32) * 1.25_f32)
                .line_clamp(1_u32)
                .build(),
            ),
        );
    }
    scroll_view_with(ScrollViewProps {
        initial_offset_y: TREE_SCROLL.with(Cell::get),
        on_scroll: (|_x: f32, y: f32| TREE_SCROLL.with(|s| s.set(y))).into(),
        ..Default::default()
    })
    .class(css::TREE_SCROLL)
    .child(list)
}

struct TreeRow {
    id: u64,
}

impl Widget for TreeRow {
    fn handle_event(&mut self, event: &UiEvent, _rect: AbsoluteRect) -> EventResult {
        if let UiEvent::MouseDown {
            button: MouseButton::Left,
            ..
        } = event
        {
            select(self.id);
            return EventResult::Handled;
        }
        EventResult::Ignored
    }

    fn hover_path_changed(&mut self, inside: bool) {
        if inside {
            inspect::set_highlight(Some(self.id));
        } else if inspect::highlighted() == Some(self.id) {
            inspect::set_highlight(None);
        }
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        Some(CursorIcon::Pointer)
    }
}

fn tree_row(node: &NodeEntry, expanded: &HashSet<u64>, selected: bool, indent: bool) -> Node {
    let depth = if indent { node.depth as f32 } else { 0.0 };
    let open = expanded.contains(&node.id);
    let id = node.id;

    let chevron: Node = if node.child_count > 0 {
        let glyph = if open {
            CHEVRON_SMALL_DOWN
        } else {
            CHEVRON_SMALL_RIGHT
        };

        ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![css::CHEVRON_BUTTON.to_string()])
            .on_click(move || {
                EXPANDED.with(|e| {
                    e.update(|set| {
                        if !set.remove(&id) {
                            set.insert(id);
                        }
                    })
                });
            })
            .build()
            .child(icon(glyph, TEXT_SECONDARY).class(css::CHEVRON))
    } else {
        div().class(css::CHEVRON_BUTTON)
    };

    let mut row = Node::new("div", TreeRow { id })
        .class(css::ROW)
        .style(format!("padding-left: {}px", 6.0 + depth * 12.0));
    if selected {
        row = row.class(css::ROW_SELECTED);
    }
    row = row.child(chevron).child(
        TextProps::new(node.tag, 13.0_f32)
            .color(TAG)
            .weight(Weight::MEDIUM)
            .line_height((13.0_f32) * 1.25_f32)
            .line_clamp(1_u32)
            .build(),
    );
    if let Some(dom_id) = &node.dom_id {
        row = row.child(
            TextProps::new(format!("#{dom_id}"), 13.0_f32)
                .color(DOM_ID)
                .weight(Weight::NORMAL)
                .line_height((13.0_f32) * 1.25_f32)
                .line_clamp(1_u32)
                .build(),
        );
    }
    for class in node.classes.iter().take(2) {
        row = row.child(
            TextProps::new(format!(".{class}"), 13.0_f32)
                .color(CLASS)
                .weight(Weight::NORMAL)
                .line_height((13.0_f32) * 1.25_f32)
                .line_clamp(1_u32)
                .build(),
        );
    }
    if node.classes.len() > 2 {
        row = row.child(
            TextProps::new(format!("+{}", node.classes.len() - 2), 13.0_f32)
                .color(TEXT_SUBTLE)
                .weight(Weight::NORMAL)
                .line_height((13.0_f32) * 1.25_f32)
                .line_clamp(1_u32)
                .build(),
        );
    }
    row = row.child(
        TextProps::styled(node.widget, TextVariant::TEXT_XS_NORMAL)
            .color(TEXT_SUBTLE)
            .line_height((12.0_f32) * 1.25_f32)
            .line_clamp(1_u32)
            .build(),
    );
    if let Some(content) = &node.text {
        row = row.child(
            TextProps::styled(format!("\"{content}\""), TextVariant::TEXT_XS_NORMAL)
                .color(TEXT_SECONDARY)
                .line_height((12.0_f32) * 1.25_f32)
                .line_clamp(1_u32)
                .build(),
        );
    }
    row
}

fn details_pane() -> Node {
    let current = DETAIL_TAB.with(Signal::get);
    let details = inspect::details().get();
    let mut pills = div().class(css::SUBTABS);
    for name in DETAIL_TABS {
        let mut pill = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![css::SUBTAB.to_string()])
            .on_click(move || DETAIL_TAB.with(|t| t.set(name)));
        if name == current {
            pill = pill.extra_classes(vec![
                css::SUBTAB.to_string(),
                css::SUBTAB_SELECTED.to_string(),
            ]);
        }
        pills = pills.child(
            pill.build().child(
                TextProps::new(name, 13.0_f32)
                    .color(if name == current {
                        TEXT_STRONG
                    } else {
                        TEXT_SECONDARY
                    })
                    .weight(Weight::MEDIUM)
                    .line_height((13.0_f32) * 1.25_f32)
                    .line_clamp(1_u32)
                    .build(),
            ),
        );
    }

    let content = match details {
        None => div().class(css::EMPTY).child(
            TextProps::styled(
                "Select an element, or use the picker, to inspect it.",
                TextVariant::TEXT_SM_NORMAL,
            )
            .color(TEXT_SUBTLE)
            .line_height((14.0_f32) * 1.25_f32)
            .line_clamp(1_u32)
            .build(),
        ),
        Some(details) => match current {
            "Styles" => styles_view(&details),
            "Computed" => key_values(&details.computed),
            "Layout" => layout_view(&details),
            _ => key_values(&details.props),
        },
    };
    let header = match inspect::details().get() {
        Some(d) => {
            let e = &d.entry;
            let mut name = e.tag.to_string();
            if let Some(id) = &e.dom_id {
                name.push('#');
                name.push_str(id);
            }
            if let Some(class) = e.classes.first() {
                name.push('.');
                name.push_str(class);
            }
            div().class(css::SELECTION).children([
                TextProps::new(name, 13.0_f32)
                    .color(TAG)
                    .weight(Weight::MEDIUM)
                    .line_height((13.0_f32) * 1.25_f32)
                    .line_clamp(1_u32)
                    .build(),
                TextProps::styled(
                    format!("{} × {}", e.width.round(), e.height.round()),
                    TextVariant::TEXT_XS_NORMAL,
                )
                .color(TEXT_SUBTLE)
                .line_height((12.0_f32) * 1.25_f32)
                .line_clamp(1_u32)
                .build(),
            ])
        }
        None => div(),
    };
    div().class(css::DETAILS).child(pills).child(header).child(
        scroll_view_with(ScrollViewProps::default())
            .class(css::DETAILS_SCROLL)
            .child(content),
    )
}

fn styles_view(details: &Rc<Details>) -> Node {
    let mut out = StackProps::vertical()
        .gap(0)
        .full_width(false)
        .padding(Padding::right(12).with_bottom(12).with_left(12))
        .build();
    if details.rules.is_empty() {
        return out.child(
            TextProps::new("No rule matches this node.", 13.0_f32)
                .color(TEXT_SUBTLE)
                .weight(Weight::NORMAL)
                .line_height((13.0_f32) * 1.25_f32)
                .line_clamp(1_u32)
                .build(),
        );
    }
    for rule in details.rules.iter() {
        let mut block = div().class(css::RULE);
        let spec = if rule.specificity == u32::MAX {
            String::new()
        } else {
            format!(
                "({}, {}, {})",
                (rule.specificity >> 20) & 0x3ff,
                (rule.specificity >> 10) & 0x3ff,
                rule.specificity & 0x3ff
            )
        };
        block = block.child(
            StackProps::horizontal()
                .align(Align::Center)
                .justify(Justify::SpaceBetween)
                .gap(0)
                .full_width(false)
                .build()
                .children([
                    TextProps::new(format!("{} {{", rule.selector), 13.0_f32)
                        .color(CLASS)
                        .weight(Weight::MEDIUM)
                        .line_height((13.0_f32) * 1.25_f32)
                        .line_clamp(1_u32)
                        .build(),
                    TextProps::new(spec, 11.0_f32)
                        .color(TEXT_SUBTLE)
                        .weight(Weight::NORMAL)
                        .line_height((11.0_f32) * 1.25_f32)
                        .line_clamp(1_u32)
                        .build(),
                ]),
        );
        for decl in &rule.declarations {
            let color = if decl.overridden {
                TEXT_SUBTLE
            } else {
                TEXT_DEFAULT
            };
            let mut line = div().class(css::DECL);
            if decl.resolved.starts_with('#') {
                line = line.child(
                    div()
                        .class(css::SWATCH)
                        .style(format!("background-color: {}", decl.resolved)),
                );
            }
            line = line.child(
                TextProps::new(format!("{}:", decl.name), 13.0_f32)
                    .color(if decl.overridden { TEXT_SUBTLE } else { TAG })
                    .weight(Weight::NORMAL)
                    .line_height((13.0_f32) * 1.25_f32)
                    .line_clamp(1_u32)
                    .build(),
            );
            line = line.child(
                TextProps::new(decl.value.clone(), 13.0_f32)
                    .color(color)
                    .weight(Weight::NORMAL)
                    .line_height((13.0_f32) * 1.25_f32)
                    .line_clamp(1_u32)
                    .build(),
            );
            if decl.resolved != decl.value {
                line = line.child(
                    TextProps::styled(format!("→ {}", decl.resolved), TextVariant::TEXT_XS_NORMAL)
                        .color(TEXT_SUBTLE)
                        .line_height((12.0_f32) * 1.25_f32)
                        .line_clamp(1_u32)
                        .build(),
                );
            }
            if decl.overridden {
                line = line.child(
                    TextProps::new("overridden", 11.0_f32)
                        .color(TEXT_SUBTLE)
                        .weight(Weight::NORMAL)
                        .line_height((11.0_f32) * 1.25_f32)
                        .line_clamp(1_u32)
                        .build(),
                );
            }
            block = block.child(line);
        }
        out = out.child(
            block.child(
                TextProps::new("}", 13.0_f32)
                    .color(CLASS)
                    .weight(Weight::MEDIUM)
                    .line_height((13.0_f32) * 1.25_f32)
                    .line_clamp(1_u32)
                    .build(),
            ),
        );
    }
    out
}

fn key_values(rows: &[(String, String)]) -> Node {
    let mut out = StackProps::vertical()
        .gap(0)
        .full_width(false)
        .padding(Padding::right(12).with_bottom(12).with_left(12))
        .build();
    for (key, value) in rows {
        let value_color = match value.as_str() {
            "true" => Color::from_argb(255, 0x5e, 0xb4, 0x79),
            "false" => TEXT_SUBTLE,
            _ => TEXT_DEFAULT,
        };
        out = out.child(
            div().class(css::KV).children([
                div().class(css::KV_KEY).child(
                    TextProps::new(key.clone(), 13.0_f32)
                        .color(TEXT_SECONDARY)
                        .weight(Weight::NORMAL)
                        .line_height((13.0_f32) * 1.25_f32)
                        .line_clamp(1_u32)
                        .build(),
                ),
                div().class(css::KV_VALUE).child(
                    TextProps::new(value.clone(), 13.0_f32)
                        .color(value_color)
                        .weight(Weight::NORMAL)
                        .line_height((13.0_f32) * 1.25_f32)
                        .line_clamp(1_u32)
                        .build(),
                ),
            ]),
        );
    }
    out
}

fn layout_view(details: &Rc<Details>) -> Node {
    let bm = &details.box_model;
    let edge = |v: f32| {
        TextProps::new(
            if v == 0.0 {
                "-".to_string()
            } else {
                format!("{}", (v * 10.0).round() / 10.0)
            },
            11.0_f32,
        )
        .color(TEXT_DEFAULT)
        .weight(Weight::NORMAL)
        .line_height((11.0_f32) * 1.25_f32)
        .line_clamp(1_u32)
        .build()
    };
    let tag = |name: &str| {
        TextProps::styled(name.to_string(), TextVariant::TEXT_XXS_MEDIUM)
            .color(TEXT_STRONG)
            .line_height((10.0_f32) * 1.25_f32)
            .line_clamp(1_u32)
            .build()
            .class(css::BM_TAG)
    };

    let layer = |class: &'static str, name: &str, edges: inspect::Edges, inner: Node| {
        div().class(css::BM_LAYER).class(class).children([
            tag(name),
            StackProps::horizontal()
                .justify(Justify::Center)
                .gap(0)
                .full_width(false)
                .build()
                .child(edge(edges.top)),
            StackProps::horizontal()
                .align(Align::Center)
                .justify(Justify::SpaceBetween)
                .gap(6)
                .full_width(false)
                .build()
                .children([
                    div().class(css::BM_SIDE).child(edge(edges.left)),
                    inner,
                    div().class(css::BM_SIDE).child(edge(edges.right)),
                ]),
            StackProps::horizontal()
                .justify(Justify::Center)
                .gap(0)
                .full_width(false)
                .build()
                .child(edge(edges.bottom)),
        ])
    };
    let content = div().class(css::BM_CONTENT).child(
        TextProps::styled(
            format!("{} × {}", bm.width.round(), bm.height.round()),
            TextVariant::TEXT_XS_MEDIUM,
        )
        .color(TEXT_STRONG)
        .line_height((12.0_f32) * 1.25_f32)
        .line_clamp(1_u32)
        .build(),
    );
    let padding = layer(css::BM_PADDING, "padding", bm.padding, content);
    let border = layer(css::BM_BORDER, "border", bm.border, padding);
    let margin = layer(css::BM_MARGIN, "margin", bm.margin, border);
    StackProps::vertical().full_width(false).build().children([
        StackProps::horizontal()
            .justify(Justify::Center)
            .gap(0)
            .full_width(false)
            .padding(Padding::y(8).with_left(12).with_right(12))
            .build()
            .child(margin),
        key_values(&[
            (
                "position".to_string(),
                format!("{:.1}, {:.1}", details.x, details.y),
            ),
            (
                "size".to_string(),
                format!("{:.1} × {:.1}", bm.width, bm.height),
            ),
        ]),
    ])
}

fn stores_tab(ctx: &DevtoolsContext) -> Node {
    let filter = search_bar("Filter stores", |query| {
        STORE_FILTER.with(|f| f.set(query.to_string()))
    })
    .class(css::FILTER);
    let toolbar = div().class(css::ELEMENTS_TOOLBAR).child(filter);

    let ctx = ctx.clone();
    let body = Node::reactive(move || {
        let state = &ctx.state;
        let selected = STORE.with(Signal::get);
        let needle = STORE_FILTER.with(Signal::get).to_lowercase();

        let user = state.users.current.get();
        let guilds = state.guilds.list.get();
        let channels = state.channels.by_guild.get();
        let messages = state.messages.by_channel.get();
        let members = state.members.by_guild.get();
        let status = state.presence.status.get();
        let custom_status = state.presence.custom_status.get();
        let ready = state.connection.ready.get();
        let session_hash = state.connection.auth_session_id_hash.get();

        let channel_count: usize = channels.values().map(Vec::len).sum();
        let message_count: usize = messages.values().map(Vec::len).sum();
        let member_count: usize = members.values().map(|m| m.len()).sum();
        let entries: [(&'static str, String); 7] = [
            ("UserStore", user.as_ref().map_or("no current user".to_string(), |u| format!("signed in as {}", u.username))),
            ("GuildStore", format!("{} guilds", guilds.len())),
            ("ChannelStore", format!("{channel_count} channels in {} guilds", channels.len())),
            ("MessageStore", format!("{message_count} messages in {} channels", messages.len())),
            ("MemberStore", format!("{member_count} members in {} guilds", members.len())),
            ("SelfPresenceStore", format!("{status:?}")),
            ("ConnectionStore", if ready { "ready".to_string() } else { "not ready".to_string() }),
        ];

        let mut list = div().class(css::STORE_LIST);
        for (name, summary) in entries.iter().filter(|(name, _)| needle.is_empty() || name.to_lowercase().contains(&needle)) {
            let name = *name;
            let mut row = ButtonProps::default().variant(ButtonVariant::Bare).extra_classes(vec![css::STORE_ROW.to_string()]).on_click(move || STORE.with(|s| s.set(name)));
            if name == selected {
                row = row.extra_classes(vec![css::STORE_ROW.to_string(), css::ROW_SELECTED.to_string()]);
            }
            list = list.child(row.build().children([TextProps::styled(name, TextVariant::TEXT_SM_MEDIUM).color(TEXT_STRONG).line_height((14.0_f32) * 1.25_f32).line_clamp(1_u32).build(), TextProps::styled(summary.clone(), TextVariant::TEXT_XS_NORMAL).color(TEXT_SUBTLE).line_height((12.0_f32) * 1.25_f32).line_clamp(1_u32).build()]));
        }

        let full = match selected {
            "UserStore" => json_pretty(&serde_json::json!({ "current": user })),
            "GuildStore" => json_pretty(&serde_json::json!({ "list": guilds })),
            "ChannelStore" => json_pretty(&serde_json::json!({ "by_guild": channels.iter().map(|(guild, list)| (guild.to_string(), list)).collect::<collections::BTreeMap<_, _>>() })),
            "MessageStore" => json_pretty(&serde_json::json!({
                "by_channel": messages.iter().map(|(channel, list)| (channel.to_string(), list)).collect::<collections::BTreeMap<_, _>>(),
                "exhausted": state.messages.exhausted.get().iter().map(|c| c.to_string()).collect::<Vec<_>>(),
            })),
            "MemberStore" => {
                let summary: collections::BTreeMap<String, String> = members.iter().map(|(guild, members)| (guild.to_string(), format!("{} members: {}", members.len(), members.values().take(5).map(|m| format!("{m:?}")).collect::<Vec<_>>().join(", ")))).collect();
                json_pretty(&serde_json::json!({ "by_guild": summary }))
            }
            "SelfPresenceStore" => json_pretty(&serde_json::json!({ "status": format!("{status:?}"), "custom_status": custom_status.map(|c| format!("{c:?}")) })),
            _ => json_pretty(&serde_json::json!({ "ready": ready, "auth_session_id_hash": session_hash })),
        };
        let shown = if full.len() > JSON_LIMIT { truncate_json(full.clone()) } else { full.clone() };
        let copy = ButtonProps::default().variant(ButtonVariant::Bare).extra_classes(vec![css::PILL.to_string()]).on_click({
            let full = full.clone();
            move || copy_text(&full)
        });
        let head = div().class(css::JSON_HEAD).children([TextProps::styled(format!("{selected} · {:.1} KB", full.len() as f32 / 1024.0), TextVariant::TEXT_XS_NORMAL).color(TEXT_SUBTLE).line_height((12.0_f32) * 1.25_f32).line_clamp(1_u32).build(), copy.build().child(TextProps::styled("Copy JSON", TextVariant::TEXT_XS_MEDIUM).color(TEXT_STRONG).line_height((12.0_f32) * 1.25_f32).line_clamp(1_u32).build())]);
        let json_view = scroll_view_with(ScrollViewProps::default()).class(css::JSON_SCROLL).child(code(&shown));
        div().class(css::STORES).children([list, head, json_view])
    })
    .class(css::BODY_HOST);
    div().class(css::ELEMENTS).children([toolbar, body])
}

fn json_pretty(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|e| format!("(not serializable: {e})"))
}

fn copy_text(text: &str) {
    let copied = clipboard::set_text(text.to_string());
    context_menu::actions().toasts.show(
        if copied { "Copied" } else { "Couldn't copy" },
        if copied {
            ToastKind::Success
        } else {
            ToastKind::Failure
        },
    );
}

fn code(content: &str) -> Node {
    text(TextProps {
        content: content.to_string(),
        size: 12.0,
        color: TEXT_DEFAULT,
        ..Default::default()
    })
    .class(css::CODE)
}

fn pill(name: &str, on_click: impl Fn() + 'static) -> Node {
    ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![css::PILL.to_string()])
        .on_click(on_click)
        .build()
        .child(
            TextProps::new(name, 13.0_f32)
                .color(TEXT_STRONG)
                .weight(Weight::MEDIUM)
                .line_height((13.0_f32) * 1.25_f32)
                .line_clamp(1_u32)
                .build(),
        )
}

fn empty(message: &str) -> Node {
    div().class(css::EMPTY).child(
        TextProps::styled(message, TextVariant::TEXT_SM_NORMAL)
            .color(TEXT_SUBTLE)
            .line_height((14.0_f32) * 1.25_f32)
            .line_clamp(1_u32)
            .build(),
    )
}

fn dispatcher_tab(ctx: &DevtoolsContext) -> Node {
    let toolbar = Node::reactive(move || {
        let paused = LOG_PAUSED.with(Signal::get);
        let count = LOG.with(Signal::get).len();
        div().class(css::ELEMENTS_TOOLBAR).children([
            pill(if paused { "Resume" } else { "Pause" }, || {
                LOG_PAUSED.with(|p| p.set(!p.peek()))
            }),
            pill("Clear", || {
                LOG.with(|l| l.set(Vec::new()));
                LOG_EXPANDED.with(|e| e.set(HashSet::new()));
            }),
            TextProps::styled(format!("{count} events"), TextVariant::TEXT_XS_NORMAL)
                .color(TEXT_SUBTLE)
                .line_height((12.0_f32) * 1.25_f32)
                .line_clamp(1_u32)
                .build(),
        ])
    });

    let list_ctx = ctx.clone();
    let list = Node::reactive(move || {
        let log = LOG.with(Signal::get);
        let expanded = LOG_EXPANDED.with(Signal::get);
        if log.is_empty() {
            return empty("Gateway events appear here while DevTools is open.");
        }
        let mut out = StackProps::vertical().gap(0).full_width(false).build();
        for entry in log.iter().rev() {
            let seq = entry.seq;
            let open = expanded.contains(&seq);
            let row = ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![css::LOG_ROW.to_string()])
                .on_click(move || {
                    LOG_EXPANDED.with(|e| {
                        e.update(|set| {
                            if !set.remove(&seq) {
                                set.insert(seq);
                            }
                        })
                    })
                });
            let name_color = if entry.handled {
                TEXT_STRONG
            } else {
                TEXT_SUBTLE
            };
            let mut item = div().class(css::LOG_ITEM).child(
                row.build().children([
                    icon(
                        if open {
                            CHEVRON_SMALL_DOWN
                        } else {
                            CHEVRON_SMALL_RIGHT
                        },
                        TEXT_SECONDARY,
                    )
                    .class(css::CHEVRON),
                    TextProps::styled(
                        format::clock_with_millis(entry.at_ms),
                        TextVariant::TEXT_XS_NORMAL,
                    )
                    .color(TEXT_SUBTLE)
                    .line_height((12.0_f32) * 1.25_f32)
                    .line_clamp(1_u32)
                    .build(),
                    TextProps::new(entry.name.clone(), 13.0_f32)
                        .color(name_color)
                        .weight(Weight::MEDIUM)
                        .line_height((13.0_f32) * 1.25_f32)
                        .line_clamp(1_u32)
                        .build(),
                    TextProps::new(
                        if entry.handled {
                            String::new()
                        } else {
                            "no store".to_string()
                        },
                        11.0_f32,
                    )
                    .color(TEXT_SUBTLE)
                    .weight(Weight::NORMAL)
                    .line_height((11.0_f32) * 1.25_f32)
                    .line_clamp(1_u32)
                    .build(),
                    TextProps::styled(
                        format!("{:.1} KB", entry.bytes as f32 / 1024.0),
                        TextVariant::TEXT_XS_NORMAL,
                    )
                    .color(TEXT_SUBTLE)
                    .line_height((12.0_f32) * 1.25_f32)
                    .line_clamp(1_u32)
                    .build()
                    .class(css::LOG_SIZE),
                ]),
            );
            if open {
                let mut actions = StackProps::horizontal()
                    .full_width(false)
                    .padding(Padding::top(4).with_right(12).with_bottom(4).with_left(28))
                    .build();
                let json = entry.json.clone();
                actions = actions.child(pill("Copy", move || copy_text(&json)));
                if let Some(value) = entry.value.clone() {
                    let (state, name) = (list_ctx.state.clone(), entry.name.clone());
                    actions = actions.child(pill("Replay", move || {
                        state.apply_event(&GatewayEvent::new(
                            name.clone(),
                            value.clone(),
                        ))
                    }));
                }
                item = item
                    .child(actions)
                    .child(code(&entry.json).class(css::LOG_JSON));
            }
            out = out.child(item);
        }
        out
    });
    div().class(css::ELEMENTS).children([
        toolbar,
        scroll_view_with(ScrollViewProps::default())
            .class(css::TREE_SCROLL)
            .child(list),
    ])
}

fn layers_tab() -> Node {
    let list = Node::reactive(move || {
        let Some(snapshot) = inspect::snapshot().get() else {
            return empty("Reading the widget tree…");
        };
        let selected = inspect::selected().get();
        let mut out = StackProps::vertical().gap(0).full_width(false).build();
        let mut shown = 0;

        for node in snapshot
            .nodes
            .iter()
            .filter(|n| n.depth == 1 || n.top_layer)
        {
            if node.depth == 1 && node.child_count == 0 && !node.top_layer {
                continue;
            }
            shown += 1;
            let id = node.id;
            let mut row = ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![css::LAYER_ROW.to_string()])
                .on_click(move || {
                    TAB.with(|t| t.set("Elements"));
                    select(id);
                    start_polling();
                });
            if selected == Some(id) {
                row = row.extra_classes(vec![
                    css::LAYER_ROW.to_string(),
                    css::ROW_SELECTED.to_string(),
                ]);
            }
            let name = node
                .classes
                .first()
                .cloned()
                .unwrap_or_else(|| node.tag.to_string());
            let badge = if node.top_layer { "top layer" } else { "" };
            out = out.child(
                row.build().children([
                    div().class(css::LAYER_MAIN).children([
                        TextProps::new(name, 13.0_f32)
                            .color(CLASS)
                            .weight(Weight::MEDIUM)
                            .line_height((13.0_f32) * 1.25_f32)
                            .line_clamp(1_u32)
                            .build(),
                        TextProps::styled(node.widget, TextVariant::TEXT_XS_NORMAL)
                            .color(TEXT_SUBTLE)
                            .line_height((12.0_f32) * 1.25_f32)
                            .line_clamp(1_u32)
                            .build(),
                    ]),
                    div().class(css::LAYER_META).children([
                        TextProps::new(badge, 11.0_f32)
                            .color(TEXT_SUBTLE)
                            .weight(Weight::NORMAL)
                            .line_height((11.0_f32) * 1.25_f32)
                            .line_clamp(1_u32)
                            .build(),
                        TextProps::styled(
                            format!("{} × {}", node.width.round(), node.height.round()),
                            TextVariant::TEXT_XS_NORMAL,
                        )
                        .color(TEXT_SECONDARY)
                        .line_height((12.0_f32) * 1.25_f32)
                        .line_clamp(1_u32)
                        .build(),
                        TextProps::styled(
                            format!("{} children", node.child_count),
                            TextVariant::TEXT_XS_NORMAL,
                        )
                        .color(TEXT_SUBTLE)
                        .line_height((12.0_f32) * 1.25_f32)
                        .line_clamp(1_u32)
                        .build(),
                    ]),
                ]),
            );
        }
        if shown == 0 {
            return empty("No layers.");
        }
        out
    });
    scroll_view_with(ScrollViewProps::default())
        .class(css::TREE_SCROLL)
        .child(list)
}

fn status_color(entry: &NetEntry) -> Color {
    match entry.status {
        Some(s) if s < 300 => Color::from_argb(255, 0x5e, 0xb4, 0x79),
        Some(s) if s < 400 => TEXT_SECONDARY,
        _ => TEXT_CRITICAL,
    }
}

fn network_tab() -> Node {
    let filter = search_bar("Filter by path or method", |query| {
        NET_FILTER.with(|f| f.set(query.to_string()))
    })
    .class(css::FILTER);
    let toolbar = div().class(css::ELEMENTS_TOOLBAR).children([
        filter,
        pill("Clear", || {
            netlog::clear();
            NET_OPEN.with(|o| o.set(HashSet::new()));
            NET.with(|n| n.set(Vec::new()));
        }),
    ]);
    let list = Node::reactive(move || {
        if NET.with(|n| n.with_untracked(Vec::is_empty)) {
            NET.with(|n| n.set(netlog::entries()));
        }
        let entries = NET.with(Signal::get);
        let needle = NET_FILTER.with(Signal::get).to_lowercase();
        let open = NET_OPEN.with(Signal::get);
        let mut out = StackProps::vertical().gap(0).full_width(false).build();
        let mut shown = 0;
        for entry in entries.iter().rev().filter(|e| {
            needle.is_empty()
                || e.path.to_lowercase().contains(&needle)
                || e.method.to_lowercase().contains(&needle)
        }) {
            shown += 1;
            let seq = entry.seq;
            let expanded = open.contains(&seq);
            let row = ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![css::LOG_ROW.to_string()])
                .on_click(move || {
                    NET_OPEN.with(|o| {
                        o.update(|set| {
                            if !set.remove(&seq) {
                                set.insert(seq);
                            }
                        })
                    })
                });
            let status = entry.status.map_or("failed".to_string(), |s| s.to_string());
            let mut item = div().class(css::LOG_ITEM).child(
                row.build().children([
                    TextProps::styled(entry.method.clone(), TextVariant::TEXT_XS_MEDIUM)
                        .color(TAG)
                        .line_height((12.0_f32) * 1.25_f32)
                        .line_clamp(1_u32)
                        .build()
                        .class(css::NET_METHOD),
                    TextProps::new(entry.path.clone(), 13.0_f32)
                        .color(TEXT_DEFAULT)
                        .weight(Weight::NORMAL)
                        .line_height((13.0_f32) * 1.25_f32)
                        .line_clamp(1_u32)
                        .build()
                        .class(css::NET_PATH),
                    TextProps::styled(status, TextVariant::TEXT_XS_MEDIUM)
                        .color(status_color(entry))
                        .line_height((12.0_f32) * 1.25_f32)
                        .line_clamp(1_u32)
                        .build(),
                    TextProps::styled(
                        format!("{} ms", entry.duration_ms),
                        TextVariant::TEXT_XS_NORMAL,
                    )
                    .color(TEXT_SUBTLE)
                    .line_height((12.0_f32) * 1.25_f32)
                    .line_clamp(1_u32)
                    .build(),
                ]),
            );
            if expanded {
                let mut detail = format!(
                    "{} {}\nfinished {}",
                    entry.method,
                    entry.path,
                    format::clock_with_millis(entry.at_ms)
                );
                if let Some(error) = &entry.error {
                    detail.push_str(&format!("\nerror: {error}"));
                }
                item = item.child(code(&detail).class(css::LOG_JSON));
            }
            out = out.child(item);
        }
        if shown == 0 {
            return empty("No requests yet.");
        }
        out
    });
    div().class(css::ELEMENTS).children([
        toolbar,
        scroll_view_with(ScrollViewProps::default())
            .class(css::TREE_SCROLL)
            .child(list),
    ])
}

fn performance_tab() -> Node {
    Node::reactive(move || {
        let stats = inspect::stats().get();
        let widgets = inspect::snapshot().get().map_or(0, |s| s.total);
        PERF_TICK.with(Signal::get);
        let mb = |bytes: usize| format!("{:.1} MB", bytes as f32 / (1024.0 * 1024.0));
        let mut rows: Vec<(String, String)> = vec![
            ("Frame rate".into(), format!("{:.0} fps", stats.fps)),
            ("Frame time".into(), format!("{:.2} ms", stats.frame_ms)),
            (
                "Window".into(),
                format!(
                    "{:.0} × {:.0} logical px at {:.2}x",
                    stats.window.0, stats.window.1, stats.scale
                ),
            ),
            ("Widgets".into(), widgets.to_string()),
        ];
        if let Some((working_set, private)) = perf::process_memory() {
            rows.push(("Working set".into(), mb(working_set)));
            rows.push(("Private bytes".into(), mb(private)));
        }
        rows.push((
            "GPU cache".into(),
            format!(
                "{} resources, {} of {}",
                stats.gpu_resources,
                mb(stats.gpu_bytes),
                mb(stats.gpu_limit)
            ),
        ));
        rows.push(("REST requests".into(), netlog::entries().len().to_string()));
        for report in perf::mem_probe_reports() {
            if let Some((name, value)) = report.split_once(':') {
                rows.push((name.trim().to_string(), value.trim().to_string()));
            }
        }
        scroll_view_with(ScrollViewProps::default())
            .class(css::TREE_SCROLL)
            .child(div().class(css::PERF).child(key_values(&rows)))
    })
    .class(css::BODY_HOST)
}

fn colors_tab() -> Node {
    let filter = search_bar("Filter tokens", |query| {
        COLOR_FILTER.with(|f| f.set(query.to_string()))
    })
    .class(css::FILTER);
    let toolbar = div().class(css::ELEMENTS_TOOLBAR).child(filter);
    let list = Node::reactive(move || {
        let tokens = inspect::tokens().get();
        let needle = COLOR_FILTER.with(Signal::get).to_lowercase();
        if tokens.is_empty() {
            inspect::request_tokens();
            return empty("Reading the stylesheet…");
        }
        let mut out = StackProps::vertical().gap(0).full_width(false).build();
        for (name, value) in tokens.iter().filter(|(n, v)| {
            needle.is_empty()
                || n.to_lowercase().contains(&needle)
                || v.to_lowercase().contains(&needle)
        }) {
            let token = format!("var({name})");
            let mut row = ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![css::TOKEN_ROW.to_string()])
                .on_click(move || copy_text(&token))
                .build();
            if value.starts_with('#') {
                row = row.child(
                    div()
                        .class(css::TOKEN_SWATCH)
                        .style(format!("background-color: {value}")),
                );
            } else {
                row = row.child(div().class(css::TOKEN_SWATCH_EMPTY));
            }
            out = out.child(
                row.children([
                    TextProps::new(name.clone(), 13.0_f32)
                        .color(TEXT_DEFAULT)
                        .weight(Weight::NORMAL)
                        .line_height((13.0_f32) * 1.25_f32)
                        .line_clamp(1_u32)
                        .build(),
                    TextProps::styled(value.clone(), TextVariant::TEXT_XS_NORMAL)
                        .color(TEXT_SUBTLE)
                        .line_height((12.0_f32) * 1.25_f32)
                        .line_clamp(1_u32)
                        .build()
                        .class(css::LOG_SIZE),
                ]),
            );
        }
        out
    });
    div().class(css::ELEMENTS).children([
        toolbar,
        scroll_view_with(ScrollViewProps::default())
            .class(css::TREE_SCROLL)
            .child(list),
    ])
}

fn trigger_group(title: &str, description: &str, buttons: Vec<Node>) -> Node {
    StackProps::vertical().full_width(false).build().children([
        TextProps::styled(title, TextVariant::TEXT_MD_MEDIUM)
            .color(TEXT_STRONG)
            .line_height((16.0_f32) * 1.25_f32)
            .line_clamp(1_u32)
            .build(),
        text(TextProps {
            content: description.to_string(),
            size: 14.0,
            color: TEXT_SECONDARY,
            ..Default::default()
        }),
        StackProps::horizontal()
            .wrap(true)
            .full_width(false)
            .build()
            .children(buttons),
    ])
}

fn triggers_tab() -> Node {
    let toasts = trigger_group(
        "Toasts",
        "Show the toast that confirms or reports an action.",
        vec![
            pill("Success toast", || {
                context_menu::actions()
                    .toasts
                    .show("This is a success toast.", ToastKind::Success)
            }),
            pill("Failure toast", || {
                context_menu::actions()
                    .toasts
                    .show("This is a failure toast.", ToastKind::Failure)
            }),
        ],
    );
    let playgrounds = trigger_group(
        "Playgrounds",
        "Open a design playground, as from the user settings menu.",
        vec![pill("Mana", || playground::open("Mana"))],
    );
    let tree = trigger_group(
        "Widget tree",
        "Read the tree again, or turn the picker on.",
        vec![
            pill("Refresh tree", inspect::request_snapshot),
            pill("Pick an element", || {
                TAB.with(|t| t.set("Elements"));
                inspect::picking().set(true);
                start_polling();
            }),
        ],
    );
    scroll_view_with(ScrollViewProps::default())
        .class(css::OVERRIDES)
        .child(
            StackProps::vertical()
                .gap(24)
                .full_width(false)
                .padding(Padding::all(16))
                .build()
                .children([toasts, playgrounds, tree]),
        )
}

fn overrides_tab() -> Node {
    Node::reactive(move || {
        let spoof = SPOOF_NITRO.with(Signal::get);
        let outlines = OUTLINES.with(Signal::get);
        let nitro = switch(
            SwitchProps::new(spoof, |on| SPOOF_NITRO.with(|s| s.update(|t| *t = t.set(on))))
                .label("Spoof Nitro".to_string())
                .description("Treats the signed-in account as a Nitro subscriber, only in this client. Premium-gated settings unlock; the server is never told.".to_string())
                ,
        );
        let boxes = switch(
            SwitchProps::new(outlines, |on| {
                OUTLINES.with(|s| s.update(|t| *t = t.set(on)));
                inspect::set_outlines(on);
            })
            .label("Outline every element".to_string())
            .description("Draws a thin box around each node of the app, coloured by nesting depth, to see how the layout is put together.".to_string())
            ,
        );
        let note = TextProps::styled(
            "These apply to this session and reset when the app restarts.",
            TextVariant::TEXT_XS_NORMAL,
        )
        .color(TEXT_SUBTLE)
        .line_height((12.0_f32) * 1.25_f32)
        .line_clamp(1_u32)
        .build();
        let critical_hint = TextProps::styled(
            "Overrides never change what is sent to the server.",
            TextVariant::TEXT_XS_NORMAL,
        )
        .color(TEXT_CRITICAL)
        .line_height((12.0_f32) * 1.25_f32)
        .line_clamp(1_u32)
        .build();
        scroll_view_with(ScrollViewProps::default())
            .class(css::OVERRIDES)
            .child(
                StackProps::vertical()
                    .gap(24)
                    .full_width(false)
                    .padding(Padding::all(16))
                    .build()
                    .children([nitro, boxes, note, critical_hint]),
            )
    })
}
