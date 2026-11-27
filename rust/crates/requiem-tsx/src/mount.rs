use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use requiem_ui::classes;
use requiem_ui::components::{
    ButtonProps, Padding, ScrollViewProps, Space, StackProps, TextProps, icon,
};
use requiem_ui::node::Node;
use requiem_ui::signal::{self, Signal, create_effect};
use requiem_ui::icons::CHEVRON_LARGE_DOWN;
use requiem_ui::widget::primitives::{
    BoxWidget, ChevronWidget, ClickAreaWidget, ScrollAnchor, ScrollPosition, ScrollViewWidget,
    TextWidget,
};
use requiem_ui::widget::{WidgetId, WidgetTree};

use crate::actions::Actions;
use crate::eval::{Frame, eval, install_queries};
use crate::state::{QueryFn, StateQueries};
use crate::ir::{
    Attr, ButtonEl, ChevronEl, Child, Component, Decl, Div, Element, Expr, Handler, HostEl, IconEl,
    ChatSpec, ItemHeight, Part, ScrollEl, StackEl, Stmt, TextEl, UseEl, Window,
};
use crate::host::{BuildFn, HostComponents, HostProps};
use crate::lower::Module;
use crate::value::Value;

pub struct Host {
    dispatch: Box<dyn Fn(&str, &[Value])>,
    queries: RefCell<HashMap<String, QueryFn>>,
    components: RefCell<HashMap<String, BuildFn>>,
}

impl Host {
    pub fn new<A: Actions + 'static>(handler: impl Fn(A) + 'static) -> Rc<Host> {
        Rc::new(Host {
            dispatch: Box::new(move |name, args| {
                if let Some(action) = A::from_call(name, args) {
                    handler(action);
                }
            }),
            queries: RefCell::default(),
            components: RefCell::default(),
        })
    }

    pub fn inert() -> Rc<Host> {
        Rc::new(Host {
            dispatch: Box::new(|_, _| {}),
            queries: RefCell::default(),
            components: RefCell::default(),
        })
    }

    pub fn serve(&self, queries: &StateQueries) {
        *self.queries.borrow_mut() = queries.runtime();
    }

    pub fn serve_components(&self, components: &HostComponents) {
        *self.components.borrow_mut() = components.runtime();
    }
}

pub struct Cx {
    host: Rc<Host>,
    module: Module,
}

pub fn mount(
    tree: &mut WidgetTree,
    parent: WidgetId,
    module: &Module,
    name: &str,
    props: &[(&str, Signal<Value>)],
    host: &Rc<Host>,
) -> Option<WidgetId> {
    let component = module.component(name)?;
    install_queries(host.queries.borrow().clone());
    let cx = Rc::new(Cx {
        host: host.clone(),
        module: module.clone(),
    });
    let at = tree.nodes[parent].children.len();
    let given: Vec<(Rc<str>, Signal<Value>)> = props
        .iter()
        .map(|(name, signal)| (Rc::from(*name), signal.clone()))
        .collect();
    Some(instantiate(tree, parent, at, component, &given, &cx))
}

fn instantiate(
    tree: &mut WidgetTree,
    parent: WidgetId,
    index: usize,
    component: &Component,
    props: &[(Rc<str>, Signal<Value>)],
    cx: &Rc<Cx>,
) -> WidgetId {
    signal::untrack(|| {
        let mut frame: Frame = component
            .props
            .iter()
            .map(|name| {
                props
                    .iter()
                    .find(|(given, _)| given == name)
                    .map_or_else(|| Signal::new(Value::Null), |(_, signal)| signal.clone())
            })
            .collect();
        let mut derived = Vec::new();
        for decl in &component.decls {
            match decl {
                Decl::State { init } => frame.push(Signal::new(eval(init, &frame))),
                Decl::Derived { expr } => {
                    let sig = Signal::new(eval(expr, &frame));
                    derived.push(keep_in_sync(tree, sig.clone(), expr.clone(), frame.clone()));
                    frame.push(sig);
                }
            }
        }
        let root = mount_element(tree, parent, index, &component.root, &frame, cx);
        for effect in derived {
            tree.attach_effect(root, effect);
        }
        root
    })
}

fn mount_host(
    tree: &mut WidgetTree,
    parent: WidgetId,
    index: usize,
    element: &HostEl,
    frame: &Frame,
    cx: &Rc<Cx>,
) -> WidgetId {
    let Some(build) = cx.host.components.borrow().get(&*element.name).cloned() else {
        return tree.mount_at(parent, index, Node::new("div", BoxWidget));
    };
    let mut synced = Vec::new();
    let values: HashMap<String, Signal<Value>> = element
        .props
        .iter()
        .map(|(name, expr)| match expr {
            Expr::Var(slot) => (name.to_string(), frame[*slot].clone()),
            other => {
                let sig = Signal::new(eval(other, frame));
                synced.push(keep_in_sync(tree, sig.clone(), other.clone(), frame.clone()));
                (name.to_string(), sig)
            }
        })
        .collect();
    let invoke_cx = cx.clone();
    let props = HostProps::new(
        values,
        Rc::new(move |callee, args| signal::untrack(|| call_func(callee, args, &invoke_cx))),
    );
    let id = tree.mount_at(parent, index, build(&props));
    for effect in synced {
        tree.attach_effect(id, effect);
    }
    id
}

fn mount_use(
    tree: &mut WidgetTree,
    parent: WidgetId,
    index: usize,
    element: &UseEl,
    frame: &Frame,
    cx: &Rc<Cx>,
) -> WidgetId {
    let Some(component) = cx.module.component(&element.name) else {
        return tree.mount_at(parent, index, Node::new("div", BoxWidget));
    };
    let mut synced = Vec::new();
    let props: Vec<(Rc<str>, Signal<Value>)> = element
        .props
        .iter()
        .map(|(name, expr)| match expr {
            Expr::Var(slot) => (name.clone(), frame[*slot].clone()),
            other => {
                let sig = Signal::new(eval(other, frame));
                synced.push(keep_in_sync(tree, sig.clone(), other.clone(), frame.clone()));
                (name.clone(), sig)
            }
        })
        .collect();
    let id = instantiate(tree, parent, index, component, &props, cx);
    for effect in synced {
        tree.attach_effect(id, effect);
    }
    id
}

fn keep_in_sync(
    tree: &mut WidgetTree,
    sig: Signal<Value>,
    expr: Expr,
    frame: Frame,
) -> signal::Effect {
    create_effect(
        move |_| {
            let value = eval(&expr, &frame);
            if sig.peek() != value {
                sig.set(value);
            }
        },
        tree,
    )
}

fn words(value: &Value) -> Vec<String> {
    value
        .to_child_text()
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

fn mount_element(
    tree: &mut WidgetTree,
    parent: WidgetId,
    index: usize,
    element: &Element,
    frame: &Frame,
    cx: &Rc<Cx>,
) -> WidgetId {
    match element {
        Element::Div(div) => mount_div(tree, parent, index, div, frame, cx),
        Element::Text(text) => mount_text(tree, parent, index, text, frame),
        Element::Button(button) => mount_button(tree, parent, index, button, frame, cx),
        Element::Icon(icon) => mount_icon(tree, parent, index, icon),
        Element::Chevron(chevron) => mount_chevron(tree, parent, index, chevron, frame),
        Element::Host(host) => mount_host(tree, parent, index, host, frame, cx),
        Element::Scroll(scroll) => mount_scroll(tree, parent, index, scroll, frame, cx),
        Element::Stack(stack) => mount_stack(tree, parent, index, stack, frame, cx),
        Element::Use(element) => mount_use(tree, parent, index, element, frame, cx),
    }
}

fn context_handler(handler: &Handler, frame: &Frame, cx: &Rc<Cx>) -> impl Fn(f32, f32) + 'static {
    let (body, event) = (handler.body.clone(), handler.event);
    let (frame, cx) = (frame.clone(), cx.clone());
    move |x, y| {
        let mut inner = frame.clone();
        if event {
            inner.push(Signal::new(Value::obj([
                ("x", Value::Num(f64::from(x))),
                ("y", Value::Num(f64::from(y))),
            ])));
        }
        signal::untrack(|| run(&body, &inner, &cx));
    }
}

fn click_event_handler(
    body: &Rc<Vec<Stmt>>,
    frame: &Frame,
    cx: &Rc<Cx>,
) -> impl FnMut(bool) + 'static {
    let (body, frame, cx) = (body.clone(), frame.clone(), cx.clone());
    move |shift| {
        let mut inner = frame.clone();
        inner.push(Signal::new(Value::obj([("shiftKey", Value::Bool(shift))])));
        signal::untrack(|| run(&body, &inner, &cx));
    }
}

fn click_handler(body: &Rc<Vec<Stmt>>, frame: &Frame, cx: &Rc<Cx>) -> impl FnMut() + 'static {
    let (body, frame, cx) = (body.clone(), frame.clone(), cx.clone());
    move || signal::untrack(|| run(&body, &frame, &cx))
}

fn mount_text(
    tree: &mut WidgetTree,
    parent: WidgetId,
    index: usize,
    element: &TextEl,
    frame: &Frame,
) -> WidgetId {
    let content = |frame: &Frame| -> String {
        element
            .parts
            .iter()
            .map(|part| match part {
                Part::Lit(text) => text.to_string(),
                Part::Dyn(expr) => eval(expr, frame).to_child_text(),
            })
            .collect()
    };
    let mut props = TextProps::styled(content(frame), element.variant).color(element.color);
    if let Some(lines) = element.line_clamp {
        props = props.line_clamp(lines);
    }
    if element.center {
        props = props.center(true);
    }
    let mut node = props.build();
    if let Some(class) = &element.class {
        node = node.classes(class.split_whitespace());
    }
    let dynamic = element.parts.iter().any(|part| matches!(part, Part::Dyn(_)));
    let sig = Signal::new(content(frame));
    if dynamic {
        node = node.bind_text(sig.clone());
    }
    let id = tree.mount_at(parent, index, node);
    if dynamic {
        let (element, frame) = (element.clone(), frame.clone());
        let effect = create_effect(
            move |_| {
                let mut text: String = element
                    .parts
                    .iter()
                    .map(|part| match part {
                        Part::Lit(text) => text.to_string(),
                        Part::Dyn(expr) => eval(expr, &frame).to_child_text(),
                    })
                    .collect();
                if element.variant.uppercase {
                    text = text.to_uppercase();
                }
                if sig.peek() != text {
                    sig.set(text);
                }
            },
            tree,
        );
        tree.attach_effect(id, effect);
    }
    id
}

fn mount_icon(tree: &mut WidgetTree, parent: WidgetId, index: usize, element: &IconEl) -> WidgetId {
    let mut node = icon(element.icon, element.color).class(classes::icon::ROOT);
    let size: Vec<String> = [("width", element.width), ("height", element.height)]
        .into_iter()
        .filter_map(|(name, px)| px.map(|px| format!("{name}: {px}px")))
        .collect();
    if !size.is_empty() {
        node = node.style(size.join("; "));
    }
    if let Some(class) = &element.class {
        node = node.classes(class.split_whitespace());
    }
    tree.mount_at(parent, index, node)
}

fn mount_chevron(
    tree: &mut WidgetTree,
    parent: WidgetId,
    index: usize,
    element: &ChevronEl,
    frame: &Frame,
) -> WidgetId {
    let open = eval(&element.open, frame).truthy();
    let mut node = Node::new(
        "span",
        ChevronWidget::new(CHEVRON_LARGE_DOWN, element.color, open),
    )
    .class(classes::icon::ROOT);
    let size: Vec<String> = [("width", element.width), ("height", element.height)]
        .into_iter()
        .filter_map(|(name, px)| px.map(|px| format!("{name}: {px}px")))
        .collect();
    if !size.is_empty() {
        node = node.style(size.join("; "));
    }
    if let Some(class) = &element.class {
        node = node.classes(class.split_whitespace());
    }
    let id = tree.mount_at(parent, index, node);
    if !matches!(element.open, Expr::Lit(_)) {
        let sig = Signal::new(open);
        let (expr, frame) = (element.open.clone(), frame.clone());
        let follow = sig.clone();
        let effect = create_effect(
            move |_| {
                let value = eval(&expr, &frame).truthy();
                if follow.peek() != value {
                    follow.set(value);
                }
            },
            tree,
        );
        tree.attach_effect(id, effect);
        let bound = signal::bind(tree, id, sig, |widget: &mut ChevronWidget, open: bool| {
            widget.set_open(open);
        });
        tree.attach_effect(id, bound);
    }
    id
}

fn mount_stack(
    tree: &mut WidgetTree,
    parent: WidgetId,
    index: usize,
    element: &StackEl,
    frame: &Frame,
    cx: &Rc<Cx>,
) -> WidgetId {
    let mut props = StackProps::default()
        .direction(element.direction)
        .align(element.align)
        .justify(element.justify)
        .wrap(element.wrap)
        .gap(Space(element.gap))
        .full_width(element.full_width);
    if element.padding > 0.0 {
        props = props.padding(Padding::all(Space(element.padding)));
    }
    let mut node = props.build();
    if let Some(class) = &element.class {
        node = node.classes(class.split_whitespace());
    }
    let id = tree.mount_at(parent, index, node);
    mount_children(tree, id, &element.children, frame, cx);
    id
}

fn mount_scroll(
    tree: &mut WidgetTree,
    parent: WidgetId,
    index: usize,
    element: &ScrollEl,
    frame: &Frame,
    cx: &Rc<Cx>,
) -> WidgetId {
    let offset = Signal::new(0.0_f32);
    let viewport = Signal::new((0.0_f32, 0.0_f32));
    let mut emitted = None;
    let mut node = match &element.chat {
        Some(chat) => {
            let (node, recent) = chat_scroll_node(chat, frame, cx);
            emitted = Some(recent);
            node
        }
        None => plain_scroll_node(element, &offset, frame, cx),
    };
    if let Some(class) = &element.class {
        node = node.classes(class.split_whitespace());
    }
    if element.window.is_some() {
        node = node
            .style("flex-direction: column")
            .observe_size(viewport.clone());
    }
    let id = tree.mount_at(parent, index, node);
    if let (Some(chat), Some(recent)) = (&element.chat, emitted) {
        let pushed = Signal::new(scroll_position(&eval(&chat.position, frame)));
        let (expr, position_frame, sink) = (chat.position.clone(), frame.clone(), pushed.clone());
        let follow = create_effect(
            move |_| {
                let next = scroll_position(&eval(&expr, &position_frame));
                if !recent.borrow().contains(&next) {
                    sink.set(next);
                }
            },
            tree,
        );
        tree.attach_effect(id, follow);
        let applied = signal::bind(tree, id, pushed, |widget: &mut ScrollViewWidget, next| {
            widget.set_chat_position(next);
        });
        tree.attach_effect(id, applied);
    }
    match (&element.window, element.children.as_slice()) {
        (Some(window), [Child::Each { list, key, item, body }]) => {
            let chat = element.chat.as_ref();
            let container = match chat {
                Some(_) => id,
                None => tree.mount_at(
                    id,
                    0,
                    StackProps::vertical()
                        .gap(Space(window.gap))
                        .build()
                        .style("flex-shrink: 0"),
                ),
            };
            mount_window(
                tree,
                container,
                Windowed {
                    window: window.clone(),
                    list: list.clone(),
                    key: key.clone(),
                    item: *item,
                    body: (**body).clone(),
                    offset,
                    viewport,
                    position: chat.map(|chat| chat.position.clone()),
                },
                frame,
                cx,
            );
        }
        _ => mount_children(tree, id, &element.children, frame, cx),
    }
    id
}

fn plain_scroll_node(
    element: &ScrollEl,
    offset: &Signal<f32>,
    frame: &Frame,
    cx: &Rc<Cx>,
) -> Node {
    let initial = element
        .initial_offset
        .as_ref()
        .map_or(0.0, |expr| eval(expr, frame).as_f32());
    let (on_scroll, on_near_end) = (element.on_scroll.clone(), element.on_near_end.clone());
    let (scroll_frame, scroll_cx, tracked) = (frame.clone(), cx.clone(), offset.clone());
    ScrollViewProps::default()
        .initial_offset_y(initial)
        .on_scroll(move |position: f32, max: f32| {
            if tracked.peek() != position {
                tracked.set(position);
            }
            if let Some(handler) = &on_scroll {
                let mut inner = scroll_frame.clone();
                if handler.event {
                    inner.push(Signal::new(Value::obj([
                        ("offset", Value::Num(f64::from(position))),
                        ("max", Value::Num(f64::from(max))),
                    ])));
                }
                signal::untrack(|| run(&handler.body, &inner, &scroll_cx));
            }
            if let Some(body) = &on_near_end {
                if max - position < NEAR_END {
                    signal::untrack(|| run(body, &scroll_frame, &scroll_cx));
                }
            }
        })
        .build()
}

const RECENT_POSITIONS: usize = 16;

fn chat_scroll_node(
    chat: &ChatSpec,
    frame: &Frame,
    cx: &Rc<Cx>,
) -> (Node, Rc<RefCell<Vec<ScrollPosition>>>) {
    let initial = scroll_position(&eval(&chat.position, frame));
    let (on_position, on_measure) = (chat.on_position.clone(), chat.on_measure.clone());
    let (position_frame, position_cx) = (frame.clone(), cx.clone());
    let (measure_frame, measure_cx) = (frame.clone(), cx.clone());
    let recent: Rc<RefCell<Vec<ScrollPosition>>> = Rc::default();
    let emitted = recent.clone();
    let widget = ScrollViewWidget::new()
        .with_chat_position(initial, chat.pin_threshold)
        .with_on_position(move |position: &ScrollPosition| {
            {
                let mut recent = emitted.borrow_mut();
                if recent.len() == RECENT_POSITIONS {
                    recent.remove(0);
                }
                recent.push(position.clone());
            }
            let Some(handler) = &on_position else { return };
            let mut inner = position_frame.clone();
            if handler.event {
                inner.push(Signal::new(position_value(position)));
            }
            signal::untrack(|| run(&handler.body, &inner, &position_cx));
        })
        .with_on_measure(move |rows: &[(String, f32)]| {
            let Some(handler) = &on_measure else { return };
            let mut inner = measure_frame.clone();
            if handler.event {
                inner.push(Signal::new(Value::obj([
                    (
                        "keys",
                        Value::list(rows.iter().map(|(key, _)| Value::str(key))),
                    ),
                    (
                        "heights",
                        Value::list(rows.iter().map(|(_, height)| Value::Num(f64::from(*height)))),
                    ),
                ])));
            }
            signal::untrack(|| run(&handler.body, &inner, &measure_cx));
        });
    (Node::new("div", widget), recent)
}

fn scroll_position(value: &Value) -> ScrollPosition {
    match value.member("key") {
        Value::Str(key) if !value.member("pinned").truthy() => ScrollPosition::Anchored(ScrollAnchor {
            key: key.to_string(),
            offset_from_top: value.member("offset").as_f32(),
            height: value.member("height").as_f32(),
        }),
        _ => ScrollPosition::Pinned,
    }
}

fn position_value(position: &ScrollPosition) -> Value {
    match position {
        ScrollPosition::Pinned => Value::obj([
            ("pinned", Value::Bool(true)),
            ("key", Value::Null),
            ("offset", Value::Num(0.0)),
            ("height", Value::Num(0.0)),
        ]),
        ScrollPosition::Anchored(anchor) => Value::obj([
            ("pinned", Value::Bool(false)),
            ("key", Value::str(&anchor.key)),
            ("offset", Value::Num(f64::from(anchor.offset_from_top))),
            ("height", Value::Num(f64::from(anchor.height))),
        ]),
    }
}

const NEAR_END: f32 = 200.0;
const OVERSCAN: f32 = 400.0;
const UNMEASURED_VIEWPORT: f32 = 900.0;

struct Windowed {
    window: Window,
    list: Expr,
    key: Expr,
    item: usize,
    body: Element,
    offset: Signal<f32>,
    viewport: Signal<(f32, f32)>,
    position: Option<Expr>,
}

fn item_height(window: &Window, item: &Value) -> f32 {
    match &window.height {
        ItemHeight::Fixed(px) => *px,
        ItemHeight::Field(name) => item.member(name).as_f32(),
    }
}

fn spacer(tree: &mut WidgetTree, column: WidgetId, height: f32) -> (WidgetId, Signal<String>) {
    let style = Signal::new(spacer_style(height));
    let node = Node::new("div", BoxWidget).bind_style(style.clone());
    let at = tree.nodes[column].children.len();
    (tree.mount_at(column, at, node), style)
}

fn spacer_style(height: f32) -> String {
    format!("height: {height}px; flex-shrink: 0")
}

fn mount_window(
    tree: &mut WidgetTree,
    column: WidgetId,
    windowed: Windowed,
    frame: &Frame,
    cx: &Rc<Cx>,
) {
    debug_assert_eq!(windowed.item, frame.len(), "lowering and mounting agree on frame depth");
    let (frame, cx) = (frame.clone(), cx.clone());
    let rows: RefCell<HashMap<String, (WidgetId, Signal<Value>)>> = RefCell::default();
    let top: RefCell<Option<(WidgetId, Signal<String>)>> = RefCell::default();
    let bottom: RefCell<Option<(WidgetId, Signal<String>)>> = RefCell::default();
    let effect = create_effect(
        move |tree| {
            let items = match eval(&windowed.list, &frame) {
                Value::List(items) => items,
                _ => Rc::default(),
            };
            let viewport = match windowed.viewport.get().1 {
                height if height > 0.0 => height,
                _ => UNMEASURED_VIEWPORT,
            };
            let gap = windowed.window.gap;
            let heights: Vec<f32> = items
                .iter()
                .map(|item| item_height(&windowed.window, item))
                .collect();
            let mut tops = Vec::with_capacity(items.len());
            let mut cursor = 0.0_f32;
            for height in &heights {
                tops.push(cursor);
                cursor += height + gap;
            }
            let offset = match &windowed.position {
                None => windowed.offset.get(),
                Some(position) => {
                    let pinned_top = (cursor - viewport).max(0.0);
                    match scroll_position(&eval(position, &frame)) {
                        ScrollPosition::Pinned => pinned_top,
                        ScrollPosition::Anchored(anchor) => signal::untrack(|| {
                            items
                                .iter()
                                .position(|item| {
                                    let mut scratch = frame.clone();
                                    scratch.push(Signal::new(item.clone()));
                                    eval(&windowed.key, &scratch).to_js_string() == anchor.key
                                })
                                .map_or(pinned_top, |at| (tops[at] - anchor.offset_from_top).max(0.0))
                        }),
                    }
                }
            };
            let (from, to) = (offset - OVERSCAN, offset + viewport + OVERSCAN);
            let start = (0..items.len())
                .find(|&at| tops[at] + heights[at] >= from)
                .unwrap_or(items.len());
            let end = (start..items.len())
                .find(|&at| tops[at] > to)
                .unwrap_or(items.len());
            let above = if start == 0 { 0.0 } else { tops[start] - gap };
            let below = if end >= items.len() {
                0.0
            } else {
                cursor - tops[end] - gap
            };

            let keyed: Vec<(String, &Value)> = signal::untrack(|| {
                let mut seen = HashSet::new();
                items[start..end]
                    .iter()
                    .filter_map(|item| {
                        let mut scratch = frame.clone();
                        scratch.push(Signal::new(item.clone()));
                        let key = eval(&windowed.key, &scratch).to_js_string();
                        seen.insert(key.clone()).then_some((key, item))
                    })
                    .collect()
            });
            let mut rows = rows.borrow_mut();
            let live: HashSet<&String> = keyed.iter().map(|(key, _)| key).collect();
            let stale: Vec<String> = rows
                .keys()
                .filter(|key| !live.contains(key))
                .cloned()
                .collect();
            for key in stale {
                if let Some((id, _)) = rows.remove(&key) {
                    tree.unmount(id);
                }
            }
            let chat = windowed.position.is_some();
            for (key, item) in &keyed {
                if let Some((_, sig)) = rows.get(key) {
                    if sig.peek() != **item {
                        sig.set((*item).clone());
                    }
                    continue;
                }
                let sig = Signal::new((*item).clone());
                let mut row_frame = frame.clone();
                row_frame.push(sig.clone());
                let at = tree.nodes[column].children.len();
                let id = signal::untrack(|| {
                    if chat {
                        let wrapper = Node::new("div", BoxWidget)
                            .style("flex-shrink: 0")
                            .dom_id(key.clone());
                        let wrapper = tree.mount_at(column, at, wrapper);
                        mount_element(tree, wrapper, 0, &windowed.body, &row_frame, &cx);
                        wrapper
                    } else {
                        mount_element(tree, column, at, &windowed.body, &row_frame, &cx)
                    }
                });
                rows.insert(key.clone(), (id, sig));
            }

            let mut top = top.borrow_mut();
            let mut bottom = bottom.borrow_mut();
            for (slot, size) in [(&mut *top, above), (&mut *bottom, below)] {
                match (slot.as_ref(), size > 0.0) {
                    (Some((_, style)), true) => {
                        let css = spacer_style(size);
                        if style.peek() != css {
                            style.set(css);
                        }
                    }
                    (Some((id, _)), false) => {
                        tree.unmount(*id);
                        *slot = None;
                    }
                    (None, true) => *slot = Some(spacer(tree, column, size)),
                    (None, false) => {}
                }
            }
            let mut order: Vec<WidgetId> = Vec::new();
            order.extend(top.as_ref().map(|(id, _)| *id));
            order.extend(keyed.iter().map(|(key, _)| rows[key].0));
            order.extend(bottom.as_ref().map(|(id, _)| *id));
            tree.reorder_children(column, &order);
            tree.invalidate_styles();
        },
        tree,
    );
    tree.attach_effect(column, effect);
}

fn mount_button(
    tree: &mut WidgetTree,
    parent: WidgetId,
    index: usize,
    element: &ButtonEl,
    frame: &Frame,
    cx: &Rc<Cx>,
) -> WidgetId {
    let text = |frame: &Frame| -> String { parts_text(&element.label, frame) };
    let mut props = ButtonProps::default()
        .variant(element.variant)
        .size(element.size)
        .disabled(element.disabled)
        .full_width(element.full_width);
    let has_label = !element.label.is_empty();
    if has_label {
        props = props.label(text(frame));
    }
    if let Some(class) = &element.class {
        props = props.extra_classes(class.split_whitespace().map(str::to_string).collect::<Vec<String>>());
    }
    if let Some(body) = &element.on_click {
        if element.on_click_event {
            props = props.on_click_shift(click_event_handler(body, frame, cx));
        } else {
            props = props.on_click(click_handler(body, frame, cx));
        }
    }
    let mut node = props.build();
    if let Some(handler) = &element.on_context_menu {
        node = node.on_context_menu(context_handler(handler, frame, cx));
    }
    let id = tree.mount_at(parent, index, node);
    if has_label && element.label.iter().any(|part| matches!(part, Part::Dyn(_))) {
        if let Some(&label) = tree.nodes[id].children.last() {
            let sig = Signal::new(text(frame));
            let (parts, frame) = (element.label.clone(), frame.clone());
            let follow = sig.clone();
            let effect = create_effect(
                move |_| {
                    let value = parts_text(&parts, &frame);
                    if follow.peek() != value {
                        follow.set(value);
                    }
                },
                tree,
            );
            tree.attach_effect(id, effect);
            let bound = signal::bind(tree, label, sig, |widget: &mut TextWidget, value: String| {
                widget.set_text(value);
            });
            tree.attach_effect(id, bound);
        }
    }
    mount_children(tree, id, &element.children, frame, cx);
    id
}

fn parts_text(parts: &[Part], frame: &Frame) -> String {
    parts
        .iter()
        .map(|part| match part {
            Part::Lit(text) => text.to_string(),
            Part::Dyn(expr) => eval(expr, frame).to_child_text(),
        })
        .collect()
}

fn mount_div(
    tree: &mut WidgetTree,
    parent: WidgetId,
    index: usize,
    element: &Div,
    frame: &Frame,
    cx: &Rc<Cx>,
) -> WidgetId {
    let mut node = match &element.on_click {
        Some(body) => Node::new(
            "div",
            ClickAreaWidget::new().with_on_click(click_handler(body, frame, cx)),
        ),
        None => Node::new("div", BoxWidget),
    };
    if let Some(handler) = &element.on_context_menu {
        node = node.on_context_menu(context_handler(handler, frame, cx));
    }
    match &element.class {
        Attr::None => {}
        Attr::Static(class) => node = node.classes(class.split_whitespace()),
        Attr::Dyn(expr) => node = node.classes(words(&eval(expr, frame))),
    }
    let mut style_sync = None;
    match &element.style {
        Attr::None => {}
        Attr::Static(style) => node = node.style(style),
        Attr::Dyn(expr) => {
            let sig = Signal::new(eval(expr, frame).to_child_text());
            node = node.bind_style(sig.clone());
            style_sync = Some((sig, expr.clone()));
        }
    }
    let id = tree.mount_at(parent, index, node);

    if let Attr::Dyn(expr) = &element.class {
        let (expr, frame) = (expr.clone(), frame.clone());
        let effect = create_effect(
            move |tree| {
                let classes = words(&eval(&expr, &frame));
                if let Some(node) = tree.nodes.get_mut(id) {
                    if node.classes != classes {
                        node.classes = classes;
                        tree.invalidate_styles();
                    }
                }
            },
            tree,
        );
        tree.attach_effect(id, effect);
    }
    if let Some((sig, expr)) = style_sync {
        let frame = frame.clone();
        let effect = create_effect(
            move |_| {
                let css = eval(&expr, &frame).to_child_text();
                if sig.peek() != css {
                    sig.set(css);
                }
            },
            tree,
        );
        tree.attach_effect(id, effect);
    }
    mount_children(tree, id, &element.children, frame, cx);
    id
}

#[derive(Default)]
struct Slot {
    ids: RefCell<Vec<WidgetId>>,
}

#[derive(Default)]
struct Siblings {
    parts: RefCell<Vec<Rc<Slot>>>,
}

impl Siblings {
    fn push(&self) -> Rc<Slot> {
        let part = Rc::new(Slot::default());
        self.parts.borrow_mut().push(part.clone());
        part
    }

    fn offset(&self, part: &Rc<Slot>) -> usize {
        self.parts
            .borrow()
            .iter()
            .take_while(|other| !Rc::ptr_eq(other, part))
            .map(|other| other.ids.borrow().len())
            .sum()
    }

    fn flat(&self) -> Vec<WidgetId> {
        self.parts
            .borrow()
            .iter()
            .flat_map(|part| part.ids.borrow().clone())
            .collect()
    }
}

fn mount_children(
    tree: &mut WidgetTree,
    parent: WidgetId,
    children: &[Child],
    frame: &Frame,
    cx: &Rc<Cx>,
) {
    let siblings = Rc::new(Siblings::default());
    for child in children {
        let part = siblings.push();
        let at = siblings.offset(&part);
        match child {
            Child::Element(element) => {
                let id = mount_element(tree, parent, at, element, frame, cx);
                part.ids.borrow_mut().push(id);
            }
            Child::If {
                cond,
                then,
                otherwise,
            } => {
                let (cond, then, otherwise) = (cond.clone(), then.clone(), otherwise.clone());
                let (frame, cx) = (frame.clone(), cx.clone());
                let (siblings, shown): (_, Cell<Option<bool>>) = (siblings.clone(), Cell::new(None));
                let effect = create_effect(
                    move |tree| {
                        let branch = eval(&cond, &frame).truthy();
                        if shown.get() == Some(branch) {
                            return;
                        }
                        shown.set(Some(branch));
                        for id in part.ids.take() {
                            tree.unmount(id);
                        }
                        let elements = if branch { &then } else { &otherwise };
                        let at = siblings.offset(&part);
                        for (offset, element) in elements.iter().enumerate() {
                            let id = signal::untrack(|| {
                                mount_element(tree, parent, at + offset, element, &frame, &cx)
                            });
                            part.ids.borrow_mut().push(id);
                        }
                        tree.invalidate_styles();
                    },
                    tree,
                );
                tree.attach_effect(parent, effect);
            }
            Child::Each {
                list,
                key,
                item,
                body,
            } => {
                debug_assert_eq!(*item, frame.len(), "lowering and mounting agree on frame depth");
                let (list, key, body) = (list.clone(), key.clone(), body.clone());
                let (frame, cx, siblings) = (frame.clone(), cx.clone(), siblings.clone());
                let rows: RefCell<HashMap<String, (WidgetId, Signal<Value>)>> = RefCell::default();
                let effect = create_effect(
                    move |tree| {
                        let items = match eval(&list, &frame) {
                            Value::List(items) => items,
                            _ => Rc::default(),
                        };
                        let keyed: Vec<(String, &Value)> = signal::untrack(|| {
                            let mut seen = HashSet::new();
                            items
                                .iter()
                                .filter_map(|item| {
                                    let mut scratch = frame.clone();
                                    scratch.push(Signal::new(item.clone()));
                                    let key = eval(&key, &scratch).to_js_string();
                                    seen.insert(key.clone()).then_some((key, item))
                                })
                                .collect()
                        });
                        let mut rows = rows.borrow_mut();

                        let live: HashSet<&String> = keyed.iter().map(|(key, _)| key).collect();
                        let stale: Vec<String> = rows
                            .keys()
                            .filter(|key| !live.contains(key))
                            .cloned()
                            .collect();
                        for key in stale {
                            if let Some((id, _)) = rows.remove(&key) {
                                tree.unmount(id);
                                part.ids.borrow_mut().retain(|other| *other != id);
                            }
                        }
                        for (key, item) in &keyed {
                            if let Some((_, sig)) = rows.get(key) {
                                if sig.peek() != **item {
                                    sig.set((*item).clone());
                                }
                                continue;
                            }
                            let sig = Signal::new((*item).clone());
                            let mut row_frame = frame.clone();
                            row_frame.push(sig.clone());
                            let at = siblings.offset(&part) + part.ids.borrow().len();
                            let id = signal::untrack(|| {
                                mount_element(tree, parent, at, &body, &row_frame, &cx)
                            });
                            part.ids.borrow_mut().push(id);
                            rows.insert(key.clone(), (id, sig));
                        }
                        *part.ids.borrow_mut() = keyed.iter().map(|(key, _)| rows[key].0).collect();
                        tree.reorder_children(parent, &siblings.flat());
                        tree.invalidate_styles();
                    },
                    tree,
                );
                tree.attach_effect(parent, effect);
            }
        }
    }
}

fn call_func(callee: &Value, args: &[Value], cx: &Cx) {
    let Value::Func(func) = callee else {
        return;
    };
    let mut inner = func.0.frame.clone();
    for index in 0..func.0.params {
        inner.push(Signal::new(args.get(index).cloned().unwrap_or_default()));
    }
    run(&func.0.body, &inner, cx);
}

fn run(stmts: &[Stmt], frame: &Frame, cx: &Cx) {
    for stmt in stmts {
        match stmt {
            Stmt::Set { slot, value } => write(&frame[*slot], eval(value, frame)),
            Stmt::Update { slot, param, value } => {
                let mut scratch = frame.clone();
                debug_assert_eq!(*param, scratch.len(), "lowering and handlers agree on frame depth");
                scratch.push(Signal::new(frame[*slot].peek()));
                write(&frame[*slot], eval(value, &scratch));
            }
            Stmt::Action { name, args } => {
                let args: Vec<Value> = args.iter().map(|arg| eval(arg, frame)).collect();
                (cx.host.dispatch)(name, &args);
            }
            Stmt::Call { callee, args } => {
                let args: Vec<Value> = args.iter().map(|arg| eval(arg, frame)).collect();
                call_func(&eval(callee, frame), &args, cx);
            }
            Stmt::If {
                cond,
                then,
                otherwise,
            } => run(if eval(cond, frame).truthy() { then } else { otherwise }, frame, cx),
        }
    }
}

fn write(sig: &Signal<Value>, value: Value) {
    if sig.peek() != value {
        sig.set(value);
    }
}
