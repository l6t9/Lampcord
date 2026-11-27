use crate::style;
use std::cell::{Cell, RefCell};
use std::collections::hash_map::DefaultHasher;
use std::collections;
use std::hash::Hash;
use std::hash::Hasher;
use std::mem;
use std::rc::Rc;

use skia_safe::{Canvas, Color, Font, Paint, PaintStyle, Rect};
use slotmap::{Key, KeyData};

use crate::paint;
use crate::signal::{Signal, signal};
use crate::style::cascade::{self, MatchedRule};
use crate::widget::{AbsoluteRect, WidgetId, WidgetTree};

pub const EXCLUDED_DOM_ID: &str = "devtools-panel";

#[derive(Clone, PartialEq)]
pub struct NodeEntry {
    pub id: u64,
    pub parent: Option<u64>,
    pub depth: u16,
    pub tag: &'static str,
    pub dom_id: Option<String>,
    pub classes: Vec<String>,
    pub widget: &'static str,

    pub top_layer: bool,
    pub child_count: usize,

    pub text: Option<String>,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, PartialEq, Default)]
pub struct Snapshot {
    pub nodes: Vec<NodeEntry>,

    pub total: usize,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Clone, PartialEq, Default)]
pub struct BoxModel {
    pub width: f32,
    pub height: f32,
    pub margin: Edges,
    pub border: Edges,
    pub padding: Edges,
}

#[derive(Clone)]
pub struct Details {
    pub id: u64,
    pub entry: NodeEntry,

    pub x: f32,
    pub y: f32,
    pub box_model: BoxModel,
    pub rules: Rc<Vec<MatchedRule>>,

    pub computed: Vec<(String, String)>,

    pub props: Vec<(String, String)>,

    fingerprint: u64,
}

impl PartialEq for Details {
    fn eq(&self, other: &Self) -> bool {
        self.fingerprint == other.fingerprint
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub struct Stats {
    pub fps: f64,
    pub frame_ms: f64,
    pub gpu_resources: usize,
    pub gpu_bytes: usize,
    pub gpu_limit: usize,
    pub window: (f32, f32),
    pub scale: f32,
}

#[derive(Default)]
struct Requests {
    snapshot: bool,
    details: Option<u64>,
    tokens: bool,
}

thread_local! {
    static REQUESTS: RefCell<Requests> = RefCell::new(Requests::default());
    static SNAPSHOT: Signal<Option<Rc<Snapshot>>> = signal(None);
    static DETAILS: Signal<Option<Rc<Details>>> = signal(None);
    static TOKENS: Signal<Rc<Vec<(String, String)>>> = signal(Rc::new(Vec::new()));
    static STATS: Signal<Stats> = signal(Stats::default());
    static STATS_WANTED: Cell<bool> = const { Cell::new(false) };

    static HIGHLIGHT: Cell<Option<u64>> = const { Cell::new(None) };
    static HIGHLIGHT_DIRTY: Cell<bool> = const { Cell::new(false) };
    static OUTLINES: Cell<bool> = const { Cell::new(false) };
    static PICKING: Signal<bool> = signal(false);
    static SELECTED: Signal<Option<u64>> = signal(None);
}

pub fn snapshot() -> Signal<Option<Rc<Snapshot>>> {
    SNAPSHOT.with(Clone::clone)
}

pub fn details() -> Signal<Option<Rc<Details>>> {
    DETAILS.with(Clone::clone)
}

pub fn selected() -> Signal<Option<u64>> {
    SELECTED.with(Clone::clone)
}

pub fn picking() -> Signal<bool> {
    PICKING.with(Clone::clone)
}

pub fn tokens() -> Signal<Rc<Vec<(String, String)>>> {
    TOKENS.with(Clone::clone)
}

pub fn request_tokens() {
    REQUESTS.with(|r| r.borrow_mut().tokens = true);
    paint::request_repaint();
}

pub fn stats() -> Signal<Stats> {
    STATS.with(Clone::clone)
}

pub fn request_stats() {
    STATS_WANTED.with(|w| w.set(true));
    paint::request_repaint();
}

pub fn take_stats_wanted() -> bool {
    STATS_WANTED.with(|w| w.replace(false))
}

pub fn publish_stats(stats: Stats) {
    if STATS.with(|s| s.with_untracked(|prev| *prev != stats)) {
        STATS.with(|s| s.set(stats));
    }
}

pub fn request_snapshot() {
    REQUESTS.with(|r| r.borrow_mut().snapshot = true);
    paint::request_repaint();
}

pub fn request_details(id: u64) {
    REQUESTS.with(|r| r.borrow_mut().details = Some(id));
    paint::request_repaint();
}

pub fn highlighted() -> Option<u64> {
    HIGHLIGHT.with(Cell::get)
}

pub fn set_highlight(id: Option<u64>) {
    if HIGHLIGHT.with(|h| h.replace(id)) != id {
        HIGHLIGHT_DIRTY.with(|d| d.set(true));
        paint::request_repaint();
    }
}

pub fn take_highlight_dirty() -> bool {
    HIGHLIGHT_DIRTY.with(|d| d.replace(false))
}

pub fn has_highlight() -> bool {
    HIGHLIGHT.with(|h| h.get().is_some()) || OUTLINES.with(Cell::get)
}

pub fn set_outlines(on: bool) {
    if OUTLINES.with(|o| o.replace(on)) != on {
        HIGHLIGHT_DIRTY.with(|d| d.set(true));
        paint::request_repaint();
    }
}

pub fn outlines() -> bool {
    OUTLINES.with(Cell::get)
}

pub fn has_requests() -> bool {
    REQUESTS.with(|r| {
        let r = r.borrow();
        r.snapshot || r.details.is_some() || r.tokens
    }) || STATS_WANTED.with(Cell::get)
}

fn to_id(raw: u64) -> WidgetId {
    KeyData::from_ffi(raw).into()
}

fn raw(id: WidgetId) -> u64 {
    id.data().as_ffi()
}

pub fn service(tree: &WidgetTree) -> bool {
    let (want_snapshot, want_details, want_tokens) = REQUESTS.with(|r| {
        let mut r = r.borrow_mut();
        (
            mem::take(&mut r.snapshot),
            r.details.take(),
            mem::take(&mut r.tokens),
        )
    });
    let mut published = false;
    if want_tokens {
        let next = cascade::custom_properties(tree);
        if TOKENS.with(|t| t.with_untracked(|prev| **prev != next)) {
            TOKENS.with(|t| t.set(Rc::new(next)));
            published = true;
        }
    }
    if want_snapshot {
        let next = take_snapshot(tree);

        if SNAPSHOT.with(|s| s.with_untracked(|prev| prev.as_deref() != Some(&next))) {
            SNAPSHOT.with(|s| s.set(Some(Rc::new(next))));
            published = true;
        }
    }
    if let Some(id) = want_details {
        let next = node_details(tree, to_id(id));
        let changed = DETAILS.with(|d| {
            d.with_untracked(|prev| match (&next, prev) {
                (Some(next), Some(prev)) => {
                    next.fingerprint != prev.fingerprint || next.id != prev.id
                }
                (None, None) => false,
                _ => true,
            })
        });
        if changed {
            DETAILS.with(|d| d.set(next.map(Rc::new)));
            published = true;
        }
    }
    published
}

fn short(text: &str) -> String {
    let flat: String = text
        .chars()
        .map(|c| if c == '\n' { ' ' } else { c })
        .collect();
    if flat.chars().count() > 60 {
        format!("{}…", flat.chars().take(60).collect::<String>())
    } else {
        flat
    }
}

fn entry_of(tree: &WidgetTree, id: WidgetId, depth: u16) -> NodeEntry {
    let node = &tree.nodes[id];
    let text = node
        .widget
        .debug_props()
        .into_iter()
        .find(|(k, _)| *k == "text")
        .map(|(_, v)| short(v.trim_matches('"')));
    NodeEntry {
        id: raw(id),
        parent: node.parent.map(raw),
        depth,
        tag: node.tag,
        dom_id: node.dom_id.clone(),
        classes: node.classes.clone(),
        widget: node.widget.debug_name(),
        top_layer: node.widget.is_top_layer(),
        child_count: node.children.len(),
        text,
        width: node.layout.size.width,
        height: node.layout.size.height,
    }
}

fn take_snapshot(tree: &WidgetTree) -> Snapshot {
    fn walk(tree: &WidgetTree, id: WidgetId, depth: u16, out: &mut Snapshot) {
        if tree.nodes[id].dom_id.as_deref() == Some(EXCLUDED_DOM_ID) {
            return;
        }
        out.nodes.push(entry_of(tree, id, depth));
        out.total += 1;
        for &child in &tree.nodes[id].children {
            walk(tree, child, depth + 1, out);
        }
    }

    let mut out = Snapshot::default();
    walk(tree, tree.root, 0, &mut out);
    out
}

fn color(c: Color) -> String {
    if c.a() == 255 {
        format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
    } else {
        format!(
            "rgba({}, {}, {}, {:.2})",
            c.r(),
            c.g(),
            c.b(),
            c.a() as f32 / 255.0
        )
    }
}

fn dim(d: taffy::Dimension) -> String {
    format!("{d:?}").replace("Dimension::", "")
}

fn push(out: &mut Vec<(String, String)>, name: &str, value: impl Into<String>) {
    out.push((name.to_string(), value.into()));
}

fn computed_values(style: &style::ComputedStyle) -> Vec<(String, String)> {
    let l = &style.layout;
    let mut v = Vec::new();
    push(&mut v, "display", format!("{:?}", l.display).to_lowercase());
    push(
        &mut v,
        "position",
        format!("{:?}", l.position).to_lowercase(),
    );
    push(&mut v, "flex-direction", format!("{:?}", l.flex_direction));
    push(&mut v, "flex-grow", l.flex_grow.to_string());
    push(&mut v, "flex-shrink", l.flex_shrink.to_string());
    if let Some(a) = l.align_items {
        push(&mut v, "align-items", format!("{a:?}"));
    }
    if let Some(j) = l.justify_content {
        push(&mut v, "justify-content", format!("{j:?}"));
    }
    push(&mut v, "width", dim(l.size.width));
    push(&mut v, "height", dim(l.size.height));
    push(&mut v, "min-width", dim(l.min_size.width));
    push(&mut v, "min-height", dim(l.min_size.height));
    push(&mut v, "max-width", dim(l.max_size.width));
    push(&mut v, "max-height", dim(l.max_size.height));
    push(
        &mut v,
        "overflow",
        format!("{:?} {:?}", l.overflow.x, l.overflow.y),
    );
    push(
        &mut v,
        "gap",
        format!("{:?} {:?}", l.gap.width, l.gap.height),
    );
    if let Some(c) = style.background {
        push(&mut v, "background-color", color(c));
    }
    if let Some(c) = style.text_color {
        push(&mut v, "color", color(c));
    }
    if style.corner_radius != 0.0 {
        push(
            &mut v,
            "border-radius",
            format!("{}px", style.corner_radius),
        );
    }
    if let Some(c) = style.border_color {
        push(
            &mut v,
            "border",
            format!(
                "{}px {} {}",
                style.border_width,
                if style.border_dashed {
                    "dashed"
                } else {
                    "solid"
                },
                color(c)
            ),
        );
    }
    if let Some(o) = style.opacity {
        push(&mut v, "opacity", o.to_string());
    }
    if style.pointer_events_none {
        push(&mut v, "pointer-events", "none");
    }
    if let Some(c) = style.cursor {
        push(&mut v, "cursor", format!("{c:?}").to_lowercase());
    }
    for shadow in &style.box_shadows {
        push(
            &mut v,
            "box-shadow",
            format!(
                "{}px {}px {}px {}px {}",
                shadow.x,
                shadow.y,
                shadow.blur,
                shadow.spread,
                color(shadow.color)
            ),
        );
    }
    if style.text_underline {
        push(&mut v, "text-decoration", "underline");
    }
    if style.text_layout.nowrap {
        push(&mut v, "white-space", "nowrap");
    }
    if style.text_layout.ellipsis {
        push(&mut v, "text-overflow", "ellipsis");
    }
    if let Some(n) = style.text_layout.line_clamp {
        push(&mut v, "-webkit-line-clamp", n.to_string());
    }
    if style.transform.is_some() {
        push(&mut v, "transform", "(set)");
    }
    v
}

fn node_details(tree: &WidgetTree, id: WidgetId) -> Option<Details> {
    let node = tree.nodes.get(id)?;
    let rects = paint::all_absolute_rects(tree);
    let rect = rects.get(&id).copied().unwrap_or_default();
    let layout = &node.layout;
    let box_model = BoxModel {
        width: layout.size.width,
        height: layout.size.height,
        margin: Edges {
            top: layout.margin.top,
            right: layout.margin.right,
            bottom: layout.margin.bottom,
            left: layout.margin.left,
        },
        border: Edges {
            top: layout.border.top,
            right: layout.border.right,
            bottom: layout.border.bottom,
            left: layout.border.left,
        },
        padding: Edges {
            top: layout.padding.top,
            right: layout.padding.right,
            bottom: layout.padding.bottom,
            left: layout.padding.left,
        },
    };
    let depth = {
        let (mut d, mut cur) = (0u16, node.parent);
        while let Some(p) = cur {
            d += 1;
            cur = tree.nodes[p].parent;
        }
        d
    };

    let mut props: Vec<(String, String)> = Vec::new();
    push(&mut props, "widget", node.widget.debug_name());
    push(&mut props, "tag", node.tag);
    if let Some(dom_id) = &node.dom_id {
        push(&mut props, "dom_id", dom_id.clone());
    }
    push(&mut props, "classes", node.classes.join(" "));
    push(&mut props, "children", node.children.len().to_string());
    push(
        &mut props,
        "focused",
        (tree.focused == Some(id)).to_string(),
    );
    push(
        &mut props,
        "hovered",
        tree.hovered_path.contains(&id).to_string(),
    );
    push(
        &mut props,
        "active",
        tree.active_path.contains(&id).to_string(),
    );
    push(
        &mut props,
        "focusable",
        node.widget.wants_focus().to_string(),
    );
    push(
        &mut props,
        "text input",
        node.widget.wants_text_input().to_string(),
    );
    push(
        &mut props,
        "top layer",
        node.widget.is_top_layer().to_string(),
    );
    push(
        &mut props,
        "reactive section",
        node.reactive.is_some().to_string(),
    );
    push(
        &mut props,
        "context menu handler",
        node.context_menu.is_some().to_string(),
    );
    push(
        &mut props,
        "class binding",
        node.class_binding.is_some().to_string(),
    );
    push(&mut props, "bound effects", node.effects.len().to_string());
    if let Some((x, y)) = node.widget.scroll_offset() {
        push(&mut props, "scroll offset", format!("{x:.0}, {y:.0}"));
    }
    for (k, v) in node.widget.debug_props() {
        push(&mut props, k, v);
    }

    let rules = Rc::new(cascade::matched_rules(tree, id));
    let computed = computed_values(&node.style);
    let entry = entry_of(tree, id, depth);

    let mut hasher = DefaultHasher::new();
    format!(
        "{:?}{:?}{:?}{}{}",
        props,
        computed,
        (box_model.width, box_model.height),
        rect.x,
        rect.y
    )
    .hash(&mut hasher);
    for rule in rules.iter() {
        rule.selector.hash(&mut hasher);
        for d in &rule.declarations {
            (&d.name, &d.value, d.overridden).hash(&mut hasher);
        }
    }
    Some(Details {
        id: raw(id),
        entry,
        x: rect.x,
        y: rect.y,
        box_model,
        rules,
        computed,
        props,
        fingerprint: hasher.finish(),
    })
}

pub fn pick_hover(tree: &WidgetTree, x: f32, y: f32) {
    let hit = paint::hit_test_all(tree, x, y).filter(|&id| !inside_panel(tree, id));
    set_highlight(hit.map(raw));
}

pub fn pick_click(tree: &WidgetTree, x: f32, y: f32) -> bool {
    if !PICKING.with(|p| p.peek()) {
        return false;
    }
    let hit = paint::hit_test_all(tree, x, y);

    if hit.is_some_and(|id| inside_panel(tree, id)) {
        return false;
    }
    if let Some(hit) = hit {
        SELECTED.with(|s| s.set(Some(raw(hit))));
        request_snapshot();
        request_details(raw(hit));
    }
    PICKING.with(|p| p.set(false));
    set_highlight(None);
    true
}

fn inside_panel(tree: &WidgetTree, mut id: WidgetId) -> bool {
    loop {
        if tree.nodes[id].dom_id.as_deref() == Some(EXCLUDED_DOM_ID) {
            return true;
        }
        match tree.nodes[id].parent {
            Some(p) => id = p,
            None => return false,
        }
    }
}

pub fn paint_highlight(tree: &WidgetTree, canvas: &Canvas, font: &Font, scale: f32) {
    if OUTLINES.with(Cell::get) {
        paint_outlines(tree, canvas, scale);
    }
    let Some(raw_id) = HIGHLIGHT.with(Cell::get) else {
        return;
    };
    let id = to_id(raw_id);
    let Some(node) = tree.nodes.get(id) else {
        return;
    };
    let rects = paint::all_absolute_rects(tree);
    let Some(&AbsoluteRect {
        x,
        y,
        width,
        height,
    }) = rects.get(&id)
    else {
        return;
    };
    let l = &node.layout;

    canvas.save();
    canvas.scale((scale, scale));
    let fill = |canvas: &Canvas, rect: Rect, c: Color| {
        let mut paint = Paint::default();
        paint.set_color(c);
        canvas.draw_rect(rect, &paint);
    };
    let border_box = Rect::from_xywh(x, y, width, height);
    let margin_box = Rect::new(
        border_box.left - l.margin.left,
        border_box.top - l.margin.top,
        border_box.right + l.margin.right,
        border_box.bottom + l.margin.bottom,
    );
    let padding_box = Rect::new(
        border_box.left + l.border.left,
        border_box.top + l.border.top,
        border_box.right - l.border.right,
        border_box.bottom - l.border.bottom,
    );
    let content_box = Rect::new(
        padding_box.left + l.padding.left,
        padding_box.top + l.padding.top,
        padding_box.right - l.padding.right,
        padding_box.bottom - l.padding.bottom,
    );

    fill(canvas, margin_box, Color::from_argb(80, 246, 178, 107));
    fill(canvas, border_box, Color::from_argb(80, 255, 229, 153));
    fill(canvas, padding_box, Color::from_argb(85, 147, 196, 125));
    fill(canvas, content_box, Color::from_argb(105, 111, 168, 220));

    let mut outline = Paint::default();
    outline.set_style(PaintStyle::Stroke);
    outline.set_stroke_width(1.0);
    outline.set_color(Color::from_argb(255, 111, 168, 220));
    canvas.draw_rect(border_box, &outline);

    let mut label = node.tag.to_string();
    if let Some(dom_id) = &node.dom_id {
        label.push('#');
        label.push_str(dom_id);
    }
    if let Some(class) = node.classes.first() {
        label.push('.');
        label.push_str(class);
    }
    label.push_str(&format!("  {} × {}", width.round(), height.round()));
    let (text_w, _) = font.measure_str(&label, None);
    let pad = 6.0;
    let h = 22.0;
    let label_y = if border_box.top - h - 4.0 >= 0.0 {
        border_box.top - h - 4.0
    } else {
        border_box.bottom + 4.0
    };
    let label_x = border_box.left.max(0.0);
    let label_rect = Rect::from_xywh(label_x, label_y, text_w + pad * 2.0, h);
    fill(canvas, label_rect, Color::from_argb(240, 36, 36, 41));
    let mut text_paint = Paint::default();
    text_paint.set_color(Color::from_argb(255, 255, 255, 255));
    text_paint.set_anti_alias(true);
    canvas.draw_str(&label, (label_x + pad, label_y + 15.0), font, &text_paint);
    canvas.restore();
}

fn paint_outlines(tree: &WidgetTree, canvas: &Canvas, scale: f32) {
    fn walk(
        tree: &WidgetTree,
        id: WidgetId,
        depth: u32,
        rects: &collections::HashMap<WidgetId, AbsoluteRect>,
        canvas: &Canvas,
        paint: &mut Paint,
    ) {
        if tree.nodes[id].dom_id.as_deref() == Some(EXCLUDED_DOM_ID) {
            return;
        }
        if let Some(rect) = rects.get(&id) {
            if rect.width > 0.0 && rect.height > 0.0 {
                let hsv = skia_safe::HSV::from(((depth * 47 % 360) as f32, 0.65, 1.0));
                let c = hsv.to_color(150);
                paint.set_color(c);
                canvas.draw_rect(
                    Rect::from_xywh(
                        rect.x + 0.5,
                        rect.y + 0.5,
                        (rect.width - 1.0).max(0.0),
                        (rect.height - 1.0).max(0.0),
                    ),
                    paint,
                );
            }
        }
        for &child in &tree.nodes[id].children {
            walk(tree, child, depth + 1, rects, canvas, paint);
        }
    }

    let rects = paint::all_absolute_rects(tree);
    canvas.save();
    canvas.scale((scale, scale));
    let mut paint = Paint::default();
    paint.set_style(PaintStyle::Stroke);
    paint.set_stroke_width(1.0 / scale.max(1.0));
    walk(tree, tree.root, 0, &rects, canvas, &mut paint);
    canvas.restore();
}
