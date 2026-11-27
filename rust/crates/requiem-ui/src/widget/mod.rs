pub mod anchor;
pub mod anim;
pub mod primitives;

use crate::control;
use crate::node::Retain;
use crate::node;
use crate::perf;
use crate::signal;
use crate::style::animation;
use crate::style::cascade;
use crate::style::parser;
use crate::style::transition;
use crate::style;
use std::any::Any;
use std::any;
use std::collections::{HashMap, HashSet};
use std::path;
use std::rc::Rc;

use skia_safe::Canvas;
use slotmap::{SlotMap, new_key_type};
use taffy::prelude::*;

use crate::event::{CursorIcon, EventResult, UiEvent};
use crate::node::{ClassBinding, ImageBinding, Node};
use crate::signal::Effect;
use crate::signal::Signal;
use crate::style::ComputedStyle;

new_key_type! { pub struct WidgetId; }

pub struct ScrollChild<'a> {
    pub key: Option<&'a str>,
    pub top: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AbsoluteRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl AbsoluteRect {
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

pub trait Widget: Any {
    fn measure(
        &self,
        _known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        Size::ZERO
    }

    fn first_baseline(&self, _content_width: f32) -> Option<f32> {
        None
    }

    fn identify(&mut self, _identity: &str) {}

    fn apply_text_style(&mut self, _text: &style::TextLayoutStyle) -> bool {
        false
    }

    fn paint(&self, _canvas: &Canvas, _rect: AbsoluteRect, _style: &ComputedStyle) {}

    fn absolute_position_override(&self) -> Option<(f32, f32)> {
        None
    }

    fn anchored_position(&self, _size: (f32, f32)) -> Option<(f32, f32)> {
        None
    }

    fn handle_event(&mut self, _event: &UiEvent, _rect: AbsoluteRect) -> EventResult {
        EventResult::Ignored
    }

    fn relayout_after(&self, _event: &UiEvent) -> bool {
        true
    }

    fn clear_press(&mut self) {}

    fn clear_hover(&mut self) {}

    fn scroll_offset(&self) -> Option<(f32, f32)> {
        None
    }

    fn wants_scroll_children(&self) -> bool {
        false
    }

    fn update_scroll_bounds(
        &mut self,
        _viewport: (f32, f32),
        _content: (f32, f32),
        _children: &[ScrollChild<'_>],
    ) {
    }

    fn wants_focus(&self) -> bool {
        false
    }

    fn wants_text_input(&self) -> bool {
        false
    }

    fn in_tab_order(&self) -> bool {
        true
    }

    fn on_focus_change(&mut self, _focused: bool) {}

    fn wants_pointer_capture(&self) -> bool {
        false
    }

    fn intercepts_pointer(&self, _event: &UiEvent, _rect: AbsoluteRect) -> bool {
        false
    }

    fn scroll_to_offset(&mut self, _offset: f32) {}

    fn paint_group_transform(&self) -> Option<(f32, f32, f32, f32)> {
        None
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        None
    }

    fn cursor_icon_at(&self, _rect: AbsoluteRect, _x: f32, _y: f32) -> Option<CursorIcon> {
        self.cursor_icon()
    }

    fn paint_overlay(&self, _canvas: &Canvas, _rect: AbsoluteRect) {}

    fn blocks_descendant_hit_test(&self) -> bool {
        false
    }

    fn extra_hit_rect(&self, _rect: AbsoluteRect) -> Option<AbsoluteRect> {
        None
    }

    fn is_top_layer(&self) -> bool {
        false
    }

    fn hit_priority(&self) -> bool {
        false
    }

    fn is_dismiss_backdrop(&self) -> bool {
        false
    }

    fn traps_pointer(&self) -> bool {
        false
    }

    fn dismiss(&mut self) -> bool {
        false
    }

    fn is_open_modal(&self) -> bool {
        false
    }

    fn hides_descendant_focus(&self) -> bool {
        false
    }

    fn selection_key(&self) -> Option<u64> {
        None
    }

    fn selection_offset_at(&self, _rect: AbsoluteRect, _x: f32, _y: f32) -> u64 {
        0
    }

    fn selection_end(&self, _rect: AbsoluteRect) -> u64 {
        0
    }

    fn selection_word(&self, _rect: AbsoluteRect, offset: u64) -> (u64, u64) {
        (offset, offset)
    }

    fn selection_paragraph(&self, _rect: AbsoluteRect, offset: u64) -> (u64, u64) {
        (offset, offset)
    }

    fn selection_text(&self, _rect: AbsoluteRect, _range: (u64, u64)) -> String {
        String::new()
    }

    fn hover_path_changed(&mut self, _inside: bool) {}

    fn press_path_changed(&mut self, _inside: bool) {}

    fn debug_name(&self) -> &'static str {
        let full = any::type_name::<Self>();
        full.rsplit("::").next().unwrap_or(full)
    }

    fn debug_props(&self) -> Vec<(&'static str, String)> {
        Vec::new()
    }
}

pub struct WidgetNode {
    pub widget: Box<dyn Widget>,
    pub style: ComputedStyle,

    pub(crate) layout_style: Style,
    pub(crate) layout_cache: taffy::Cache,
    pub(crate) unrounded_layout: Layout,

    pub layout: Layout,
    pub tag: &'static str,
    pub dom_id: Option<String>,
    pub classes: Vec<String>,
    pub parent: Option<WidgetId>,
    pub children: Vec<WidgetId>,

    pub class_binding: Option<ClassBinding>,

    pub image_binding: Option<ImageBinding>,

    pub text_binding: Option<Signal<String>>,

    pub size_observer: Option<Signal<(f32, f32)>>,

    pub rect_observer: Option<Signal<AbsoluteRect>>,

    pub reveal: Option<(String, f32)>,

    pub inline_style: Vec<parser::Declaration>,

    pub inline_style_source: Option<String>,

    pub(crate) transitions: transition::NodeTransitions,

    pub(crate) animations: animation::NodeAnimations,

    pub(crate) effects: Vec<Effect>,

    pub reactive: Option<Rc<dyn Fn() -> Node>>,

    pub context_menu: Option<node::ContextMenuHandler>,

    pub retain: Option<Retain>,

    pub focus_ring_host: bool,
    pub mount_hook: Option<node::MountHook>,
}

impl WidgetNode {
    fn new(
        widget: Box<dyn Widget>,
        style: ComputedStyle,
        tag: &'static str,
        parent: Option<WidgetId>,
    ) -> Self {
        Self {
            widget,
            layout_style: style.to_taffy(),
            style,
            layout_cache: taffy::Cache::new(),
            unrounded_layout: Layout::new(),
            layout: Layout::new(),
            tag,
            dom_id: None,
            classes: Vec::new(),
            parent,
            children: Vec::new(),
            class_binding: None,
            image_binding: None,
            text_binding: None,
            size_observer: None,
            rect_observer: None,
            reveal: None,
            inline_style: Vec::new(),
            inline_style_source: None,
            transitions: Default::default(),
            animations: Default::default(),
            effects: Vec::new(),
            reactive: None,
            context_menu: None,
            retain: None,
            focus_ring_host: false,
            mount_hook: None,
        }
    }
}

pub struct WidgetTree {
    pub nodes: SlotMap<WidgetId, WidgetNode>,
    pub root: WidgetId,

    pub focused: Option<WidgetId>,

    pub defer_mount_hooks: bool,

    pub focus_scope: Option<WidgetId>,

    pub focus_visible: bool,

    pub hovered_path: HashSet<WidgetId>,

    pub active_path: HashSet<WidgetId>,

    pub(crate) rules: Option<Rc<cascade::StyleIndex>>,

    pub(crate) style_dirty: bool,

    pub(crate) interactive_changed: Vec<WidgetId>,

    retained_pool: HashMap<(u64, u64), WidgetId>,

    has_retained: bool,
    pub(crate) layout_dirty: bool,
    pub(crate) last_layout_space: Option<(u32, u32)>,

    pub pointer_capture: Option<WidgetId>,

    pub swallow_release: bool,

    pub owner: u64,
}

pub struct TextFocusMemo {
    path: Vec<usize>,
    caret: Option<(usize, Option<usize>)>,
}

impl WidgetTree {
    pub fn new(root_widget: Box<dyn Widget>, root_style: ComputedStyle) -> Self {
        let mut nodes = SlotMap::with_key();
        let root = nodes.insert(WidgetNode::new(root_widget, root_style, "root", None));

        Self {
            nodes,
            root,
            focused: None,
            focus_scope: None,
            focus_visible: false,
            hovered_path: HashSet::new(),
            active_path: HashSet::new(),
            rules: None,
            style_dirty: false,
            interactive_changed: Vec::new(),
            retained_pool: HashMap::new(),
            has_retained: false,
            defer_mount_hooks: false,
            layout_dirty: true,
            last_layout_space: None,
            pointer_capture: None,
            swallow_release: false,
            owner: 0,
        }
    }

    pub fn invalidate_styles(&mut self) {
        self.style_dirty = true;
    }

    #[track_caller]
    pub fn mark_dirty(&mut self, id: WidgetId) {
        if perf::profile_paint() {
            let at = std::panic::Location::caller();
            let node = self.nodes.get(id);
            let what = node
                .map(|n| {
                    n.classes
                        .first()
                        .cloned()
                        .unwrap_or_else(|| n.tag.to_string())
                })
                .unwrap_or_default();
            perf::record_widget_paint(
                format!(
                    "SIGNAL-SET LAYOUT-DIRTY {}:{} on {}",
                    at.file()
                        .rsplit(['/', path::MAIN_SEPARATOR])
                        .next()
                        .unwrap_or(""),
                    at.line(),
                    what
                ),
                0.0,
            );
        }
        self.layout_dirty = true;
        let mut current = Some(id);
        while let Some(node) = current {
            let Some(widget_node) = self.nodes.get_mut(node) else {
                break;
            };
            widget_node.layout_cache.clear();
            current = widget_node.parent;
        }
    }

    pub(crate) fn set_layout_style(&mut self, id: WidgetId, style: Style) {
        if self.nodes[id].layout_style != style {
            self.nodes[id].layout_style = style;
            self.mark_dirty(id);
        }
    }

    pub fn layout(&self, id: WidgetId) -> &Layout {
        &self.nodes[id].layout
    }

    pub fn add_child(
        &mut self,
        parent: WidgetId,
        tag: &'static str,
        widget: Box<dyn Widget>,
        style: ComputedStyle,
    ) -> WidgetId {
        let child = self
            .nodes
            .insert(WidgetNode::new(widget, style, tag, Some(parent)));
        self.nodes[parent].children.push(child);
        self.mark_dirty(parent);
        child
    }

    fn take_focus_on_mount(&mut self, id: WidgetId) {
        if let Some(prev) = self.focused.filter(|&prev| prev != id) {
            if let Some(node) = self.nodes.get_mut(prev) {
                node.widget.on_focus_change(false);
            }
        }
        self.focused = Some(id);
    }

    fn mount_child(&mut self, parent: WidgetId, child: Node) -> WidgetId {
        if let Some(Retain::Reuse(key, version)) = child.retain {
            if let Some(existing) = self.retained_pool.remove(&(key, version)) {
                self.nodes[existing].parent = Some(parent);
                self.nodes[parent].children.push(existing);

                self.mark_dirty(parent);
                return existing;
            }
            log::warn!("a reused row ({key}, {version}) was not in the pool: it is shown empty");
        }
        self.mount(parent, child)
    }

    pub fn collect_retained(&self, root: WidgetId) -> HashSet<(u64, u64)> {
        let mut found = HashSet::new();
        if !self.has_retained {
            return found;
        }
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let Some(node) = self.nodes.get(id) else {
                continue;
            };
            if let Some(Retain::Tag(key, version)) = node.retain {
                found.insert((key, version));
                continue;
            }
            if id != root && node.reactive.is_some() {
                continue;
            }
            stack.extend(node.children.iter().copied());
        }
        found
    }

    pub fn pool_retained(&mut self, root: WidgetId) {
        if !self.has_retained {
            return;
        }
        let mut lifted = Vec::new();
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let Some(node) = self.nodes.get(id) else {
                continue;
            };
            if let Some(Retain::Tag(key, version)) = node.retain {
                lifted.push((id, (key, version)));
                continue;
            }
            if id != root && node.reactive.is_some() {
                continue;
            }
            stack.extend(node.children.iter().copied());
        }
        for (id, tag) in lifted {
            if let Some(parent) = self.nodes[id].parent.take() {
                self.nodes[parent].children.retain(|&c| c != id);
            }
            self.retained_pool.insert(tag, id);
        }
    }

    pub fn drop_retained_pool(&mut self) {
        if self.retained_pool.is_empty() {
            return;
        }
        let leftovers: Vec<WidgetId> = self.retained_pool.drain().map(|(_, id)| id).collect();
        for id in leftovers {
            self.unmount(id);
        }
    }

    pub fn mount(&mut self, parent: WidgetId, node: Node) -> WidgetId {
        let mount_hook = node.mount_hook;
        let id = self.add_child(parent, node.tag, node.widget, ComputedStyle::default());
        self.nodes[id].classes = node.classes;
        self.nodes[id].dom_id = node.dom_id;
        self.identify_widget(id);
        self.nodes[id].context_menu = node.context_menu;
        if let Some(css) = node.inline_style {
            self.nodes[id].inline_style = parser::parse_inline_declarations(&css);
            self.nodes[id].inline_style_source = Some(css);
        }

        if node.wants_focus {
            self.take_focus_on_mount(id);
        }
        self.nodes[id].focus_ring_host = node.focus_ring_host;
        if node.wants_focus_scope {
            self.focus_scope = Some(id);
        }
        if let Some(binding) = node.class_binding {
            self.nodes[id].class_binding = Some(binding.clone());
            self.apply_class_binding(id, binding);
        }
        if let Some(style) = node.style_binding {
            self.apply_style_binding(id, style);
        }
        self.nodes[id].size_observer = node.size_observer;
        self.nodes[id].rect_observer = node.rect_observer;
        self.nodes[id].reveal = node.reveal;
        if let Some(binding) = node.image_binding {
            self.nodes[id].image_binding = Some(binding.clone());
            self.apply_image_binding(id, binding);
        }
        if let Some(binding) = node.text_binding {
            self.nodes[id].text_binding = Some(binding.clone());
            self.apply_text_binding(id, binding);
        }
        self.nodes[id].retain = node.retain;
        if matches!(node.retain, Some(Retain::Tag(..))) {
            self.has_retained = true;
        }
        if let Some(render) = node.reactive {
            self.apply_reactive(id, render);
        } else {
            for child in node.children {
                self.mount_child(id, child);
            }
        }
        if let Some(hook) = mount_hook {
            if self.defer_mount_hooks {
                self.nodes[id].mount_hook = Some(hook);
            } else {
                hook(self, id);
            }
        }
        id
    }

    fn identify_widget(&mut self, id: WidgetId) {
        let node = &mut self.nodes[id];
        let identity = node
            .dom_id
            .clone()
            .unwrap_or_else(|| node.classes.join(" "));
        if !identity.is_empty() {
            node.widget.identify(&identity);
        }
    }

    pub fn attach_effect(&mut self, id: WidgetId, effect: Effect) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.effects.push(effect);
        }
    }

    fn apply_reactive(&mut self, id: WidgetId, render: Rc<dyn Fn() -> Node>) {
        self.nodes[id].reactive = Some(render.clone());
        let effect = control::section(self, id, move || render());
        if let Some(node) = self.nodes.get_mut(id) {
            node.effects.push(effect);
        }
    }

    fn apply_class_binding(&mut self, id: WidgetId, binding: ClassBinding) {
        let ClassBinding {
            signal,
            class_if_true,
            class_if_false,
        } = binding;
        let effect = signal::create_effect(
            move |tree| {
                let value = signal.get();
                if let Some(node) = tree.nodes.get_mut(id) {
                    node.classes
                        .retain(|c| c != &class_if_true && c != &class_if_false);
                    node.classes.push(if value {
                        class_if_true.clone()
                    } else {
                        class_if_false.clone()
                    });
                } else {
                    return;
                }

                tree.invalidate_styles();
            },
            self,
        );
        if let Some(node) = self.nodes.get_mut(id) {
            node.effects.push(effect);
        }
    }

    fn apply_style_binding(&mut self, id: WidgetId, style: Signal<String>) {
        let effect = signal::create_effect(
            move |tree| {
                let css = style.get();
                if let Some(node) = tree.nodes.get_mut(id) {
                    node.inline_style = parser::parse_inline_declarations(&css);
                    node.inline_style_source = Some(css);
                } else {
                    return;
                }
                tree.invalidate_styles();
                tree.mark_dirty(id);
            },
            self,
        );
        if let Some(node) = self.nodes.get_mut(id) {
            node.effects.push(effect);
        }
    }

    fn apply_image_binding(&mut self, id: WidgetId, binding: ImageBinding) {
        let ImageBinding { tick, resolve } = binding;
        let effect = signal::create_effect(
            move |tree| {
                let _ = tick.get();

                if let Some(node) = tree.nodes.get_mut(id) {
                    if let Some(widget) = (&mut *node.widget as &mut dyn Any)
                        .downcast_mut::<primitives::MessageTextWidget>()
                    {
                        if widget.refresh_custom_emoji() {
                            tree.mark_dirty(id);
                        }
                        return;
                    }
                    if let Some(widget) = (&mut *node.widget as &mut dyn Any)
                        .downcast_mut::<primitives::TextInputWidget>()
                    {
                        if widget.refresh_custom_emoji() {
                            tree.mark_dirty(id);
                        }
                        return;
                    }
                }
                let Some(image) = resolve() else { return };
                let swapped = match tree.nodes.get_mut(id) {
                    Some(node) => {
                        let widget = &mut *node.widget;
                        if let Some(widget) =
                            (widget as &mut dyn Any).downcast_mut::<primitives::ImageWidget>()
                        {
                            widget.set_image(image);
                            true
                        } else if let Some(widget) =
                            (widget as &mut dyn Any).downcast_mut::<primitives::AvatarWidget>()
                        {
                            widget.set_image(image);
                            true
                        } else {
                            false
                        }
                    }
                    None => false,
                };
                if swapped {
                    tree.mark_dirty(id);
                }
            },
            self,
        );
        if let Some(node) = self.nodes.get_mut(id) {
            node.effects.push(effect);
        }
    }

    fn apply_text_binding(&mut self, id: WidgetId, text: Signal<String>) {
        let effect = signal::create_effect(
            move |tree| {
                let value = text.get();
                if let Some(node) = tree.nodes.get_mut(id) {
                    let any = &mut *node.widget as &mut dyn Any;
                    if let Some(widget) = any.downcast_mut::<primitives::TextWidget>() {
                        widget.set_text(value);
                        tree.mark_dirty(id);
                    } else if let Some(widget) = any.downcast_mut::<primitives::MessageTextWidget>()
                    {
                        widget.set_text(value);
                        tree.mark_dirty(id);
                    }
                }
            },
            self,
        );
        if let Some(node) = self.nodes.get_mut(id) {
            node.effects.push(effect);
        }
    }

    pub fn add_child_at(
        &mut self,
        parent: WidgetId,
        index: usize,
        tag: &'static str,
        widget: Box<dyn Widget>,
        style: ComputedStyle,
    ) -> WidgetId {
        let child = self
            .nodes
            .insert(WidgetNode::new(widget, style, tag, Some(parent)));
        let index = index.min(self.nodes[parent].children.len());
        self.nodes[parent].children.insert(index, child);
        self.mark_dirty(parent);
        child
    }

    pub fn mount_at(&mut self, parent: WidgetId, index: usize, node: Node) -> WidgetId {
        let mount_hook = node.mount_hook;
        let id = self.add_child_at(
            parent,
            index,
            node.tag,
            node.widget,
            ComputedStyle::default(),
        );
        self.nodes[id].classes = node.classes;
        self.nodes[id].dom_id = node.dom_id;
        self.identify_widget(id);
        self.nodes[id].context_menu = node.context_menu;
        if let Some(css) = node.inline_style {
            self.nodes[id].inline_style = parser::parse_inline_declarations(&css);
            self.nodes[id].inline_style_source = Some(css);
        }

        if node.wants_focus {
            self.take_focus_on_mount(id);
        }
        self.nodes[id].focus_ring_host = node.focus_ring_host;
        if node.wants_focus_scope {
            self.focus_scope = Some(id);
        }
        if let Some(binding) = node.class_binding {
            self.nodes[id].class_binding = Some(binding.clone());
            self.apply_class_binding(id, binding);
        }
        if let Some(style) = node.style_binding {
            self.apply_style_binding(id, style);
        }
        self.nodes[id].size_observer = node.size_observer;
        self.nodes[id].rect_observer = node.rect_observer;
        self.nodes[id].reveal = node.reveal;
        if let Some(binding) = node.image_binding {
            self.nodes[id].image_binding = Some(binding.clone());
            self.apply_image_binding(id, binding);
        }
        if let Some(binding) = node.text_binding {
            self.nodes[id].text_binding = Some(binding.clone());
            self.apply_text_binding(id, binding);
        }
        self.nodes[id].retain = node.retain;
        if matches!(node.retain, Some(Retain::Tag(..))) {
            self.has_retained = true;
        }
        if let Some(render) = node.reactive {
            self.apply_reactive(id, render);
        } else {
            for child in node.children {
                self.mount_child(id, child);
            }
        }
        if let Some(hook) = mount_hook {
            if self.defer_mount_hooks {
                self.nodes[id].mount_hook = Some(hook);
            } else {
                hook(self, id);
            }
        }
        id
    }

    pub fn reorder_children(&mut self, parent: WidgetId, order: &[WidgetId]) {
        debug_assert_eq!(
            order.len(),
            self.nodes[parent].children.len(),
            "reorder_children must be given a permutation of parent's existing children, not a different set"
        );
        self.nodes[parent].children = order.to_vec();
        self.mark_dirty(parent);
    }

    pub fn remember_text_focus(&self, root: WidgetId) -> Option<TextFocusMemo> {
        let focused = self.focused?;
        if !self.nodes.get(focused)?.widget.wants_text_input() {
            return None;
        }
        let mut path = Vec::new();
        let mut current = focused;
        while current != root {
            let parent = self.nodes.get(current)?.parent?;
            path.push(
                self.nodes
                    .get(parent)?
                    .children
                    .iter()
                    .position(|&c| c == current)?,
            );
            current = parent;
        }
        path.reverse();
        let caret = (&*self.nodes[focused].widget as &dyn Any)
            .downcast_ref::<primitives::TextInputWidget>()
            .map(|w| w.caret_state());
        Some(TextFocusMemo { path, caret })
    }

    pub fn restore_text_focus(&mut self, root: WidgetId, memo: TextFocusMemo) {
        let mut current = root;
        for index in &memo.path {
            let Some(&next) = self.nodes.get(current).and_then(|n| n.children.get(*index)) else {
                return;
            };
            current = next;
        }
        if self.focused.is_some_and(|f| self.nodes.contains_key(f)) {
            // The rebuilt input took focus as it mounted (a composer built focused), which
            // would otherwise leave its caret at the default end of the text.
            if self.focused == Some(current) {
                if let (Some(caret), Some(node)) = (memo.caret, self.nodes.get_mut(current)) {
                    if let Some(widget) =
                        (&mut *node.widget as &mut dyn Any).downcast_mut::<primitives::TextInputWidget>()
                    {
                        widget.set_caret_state(caret);
                        self.mark_dirty(current);
                    }
                }
            }
            return;
        }
        let Some(node) = self.nodes.get_mut(current) else {
            return;
        };
        if !node.widget.wants_text_input() {
            return;
        }
        node.widget.on_focus_change(true);
        if let (Some(caret), Some(widget)) = (
            memo.caret,
            (&mut *node.widget as &mut dyn Any).downcast_mut::<primitives::TextInputWidget>(),
        ) {
            widget.set_caret_state(caret);
        }
        self.focused = Some(current);
        self.mark_dirty(current);
    }

    pub fn unmount(&mut self, id: WidgetId) {
        let children = self.nodes[id].children.clone();
        for child in children {
            self.unmount(child);
        }

        if let Some(parent) = self.nodes[id].parent {
            self.nodes[parent].children.retain(|&c| c != id);
            self.mark_dirty(parent);
        }
        self.nodes.remove(id);

        if self.focused == Some(id) {
            self.focused = None;
        }
        if self.focus_scope == Some(id) {
            self.focus_scope = None;
        }
        if self.pointer_capture == Some(id) {
            self.pointer_capture = None;
        }
        self.hovered_path.remove(&id);
        self.active_path.remove(&id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::primitives::BoxWidget;

    fn test_tree() -> WidgetTree {
        WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default())
    }

    #[test]
    fn unmount_removes_node_and_descendants() {
        let mut tree = test_tree();
        let root = tree.root;
        let parent = tree.add_child(root, "div", Box::new(BoxWidget), ComputedStyle::default());
        let child = tree.add_child(parent, "div", Box::new(BoxWidget), ComputedStyle::default());
        tree.unmount(parent);

        assert!(!tree.nodes.contains_key(parent));
        assert!(!tree.nodes.contains_key(child));
        assert!(tree.nodes[root].children.is_empty());
    }

    #[test]
    fn unmount_clears_dangling_interaction_state() {
        let mut tree = test_tree();
        let root = tree.root;
        let id = tree.add_child(root, "div", Box::new(BoxWidget), ComputedStyle::default());
        tree.focused = Some(id);
        tree.focus_scope = Some(id);
        tree.pointer_capture = Some(id);
        tree.hovered_path.insert(id);
        tree.active_path.insert(id);

        tree.unmount(id);

        assert_eq!(tree.focused, None);
        assert_eq!(tree.focus_scope, None);
        assert_eq!(tree.pointer_capture, None);
        assert!(!tree.hovered_path.contains(&id));
        assert!(!tree.active_path.contains(&id));
    }

    #[test]
    fn add_child_at_inserts_in_order() {
        let mut tree = test_tree();
        let root = tree.root;
        let a = tree.add_child(root, "div", Box::new(BoxWidget), ComputedStyle::default());
        let c = tree.add_child(root, "div", Box::new(BoxWidget), ComputedStyle::default());
        let b = tree.add_child_at(
            root,
            1,
            "div",
            Box::new(BoxWidget),
            ComputedStyle::default(),
        );

        assert_eq!(tree.nodes[root].children, vec![a, b, c]);
    }

    #[test]
    fn widget_trait_object_upcasts_to_any_for_downcasting() {
        let mut boxed: Box<dyn Widget> = Box::new(BoxWidget);
        let any: &mut dyn Any = &mut *boxed;
        assert!(any.downcast_mut::<BoxWidget>().is_some());
    }
}
