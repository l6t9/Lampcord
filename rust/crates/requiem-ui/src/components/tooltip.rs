use crate::classes::tooltip as classes_tooltip;
use crate::classes::tooltip as cls;
use crate::clipboard;
use crate::font_system;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use skia_safe::Color;
use skia_safe::font_style::Weight;

use super::container::div;
use super::text::{TextProps, text};
use crate::font_system::TextFonts;
use crate::node::Node;
use crate::signal::{Signal, signal};
use crate::timer::{TimeoutHandle, clear_timeout, set_timeout};
use crate::widget::primitives::{
    PointerEdge, TooltipAnchorWidget, TooltipPointerWidget, TopLayerWidget,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipPosition {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipColor {
    #[default]
    Primary,
    Green,
    Brand,
}

impl TooltipColor {
    fn class(self) -> &'static str {
        match self {
            TooltipColor::Primary => classes_tooltip::PRIMARY,
            TooltipColor::Green => classes_tooltip::GREEN,
            TooltipColor::Brand => classes_tooltip::BRAND,
        }
    }
}

pub const COPY_MESSAGES: [&str; 11] = [
    requiem_i18n::key!("ui-copy-1"),
    requiem_i18n::key!("ui-copy-2"),
    requiem_i18n::key!("ui-copy-3"),
    requiem_i18n::key!("ui-copy-4"),
    requiem_i18n::key!("ui-copy-5"),
    requiem_i18n::key!("ui-copy-6"),
    requiem_i18n::key!("ui-copy-7"),
    requiem_i18n::key!("ui-copy-8"),
    requiem_i18n::key!("ui-copy-9"),
    requiem_i18n::key!("ui-copy-10"),
    requiem_i18n::key!("ui-copy-11"),
];

const COPY_FORCE_OPEN: Duration = Duration::from_secs(1);

const COPY_STREAK_RESET: Duration = Duration::from_secs(2);

const LEAVE_ANIMATION: Duration = Duration::from_millis(100);

struct Inner {
    hovered: Cell<bool>,

    dismissed: Cell<bool>,
    forced: Cell<bool>,
    hide_on_click: Cell<bool>,
    delay: Cell<Option<Duration>>,
    show_timer: Cell<Option<TimeoutHandle>>,
    delay_elapsed: Cell<bool>,

    label: RefCell<String>,
    copy_count: Cell<usize>,
    copied: Cell<bool>,
    timers: RefCell<Vec<TimeoutHandle>>,
    visible: Signal<bool>,
    feedback: Signal<bool>,
    text: Signal<String>,
}

#[derive(Clone)]
pub struct TooltipState(Rc<Inner>);

impl Default for TooltipState {
    fn default() -> Self {
        Self::new()
    }
}

impl TooltipState {
    pub fn new() -> Self {
        Self(Rc::new(Inner {
            hovered: Cell::new(false),
            dismissed: Cell::new(false),
            forced: Cell::new(false),
            hide_on_click: Cell::new(true),
            delay: Cell::new(None),
            show_timer: Cell::new(None),
            delay_elapsed: Cell::new(false),
            label: RefCell::new(String::new()),
            copy_count: Cell::new(0),
            copied: Cell::new(false),
            timers: RefCell::new(Vec::new()),
            visible: signal(false),
            feedback: signal(false),
            text: signal(String::new()),
        }))
    }

    fn refresh(&self) {
        let s = &self.0;
        let shown = s.forced.get()
            || (s.hovered.get() && !s.dismissed.get() && s.delay_elapsed.get() && !s.copied.get());
        if s.visible.peek() != shown {
            s.visible.set(shown);
        }
        let feedback = s.copied.get();
        if s.feedback.peek() != feedback {
            s.feedback.set(feedback);
        }
        let text = if feedback {
            requiem_i18n::text(
                COPY_MESSAGES[s
                    .copy_count
                    .get()
                    .saturating_sub(1)
                    .min(COPY_MESSAGES.len() - 1)],
            )
        } else {
            s.label.borrow().clone()
        };
        if s.text.peek() != text {
            s.text.set(text);
        }
    }

    pub(crate) fn set_hovered(&self, hovered: bool) {
        let s = &self.0;
        s.hovered.set(hovered);
        if let Some(handle) = s.show_timer.take() {
            clear_timeout(handle);
        }
        if hovered {
            if s.copied.get() {
                s.forced.set(false);
            }
            match s.delay.get() {
                Some(delay) if !delay.is_zero() => {
                    s.delay_elapsed.set(false);
                    let this = self.clone();
                    s.show_timer.set(Some(set_timeout(delay, move || {
                        this.0.delay_elapsed.set(true);
                        this.refresh();
                    })));
                }
                _ => s.delay_elapsed.set(true),
            }
        } else {
            s.dismissed.set(false);
            s.delay_elapsed.set(false);
        }
        self.refresh();
    }

    pub(crate) fn visible_signal(&self) -> Signal<bool> {
        self.0.visible.clone()
    }

    pub(crate) fn pressed(&self) {
        if self.0.hide_on_click.get() {
            self.0.dismissed.set(true);
            self.refresh();
        }
    }

    pub fn force_open(&self, open: bool) {
        self.0.forced.set(open);
        self.refresh();
    }

    pub fn set_label(&self, label: impl Into<String>) {
        *self.0.label.borrow_mut() = label.into();
        self.refresh();
    }

    fn schedule(&self, delay: Duration, f: impl FnOnce(&TooltipState) + 'static) {
        let this = self.clone();
        let handle = set_timeout(delay, move || f(&this));
        self.0.timers.borrow_mut().push(handle);
    }

    pub fn copy(&self, value: impl Into<String>) {
        clipboard::set_text(value.into());
        let s = &self.0;
        for handle in s.timers.borrow_mut().drain(..) {
            clear_timeout(handle);
        }
        if !s.forced.get() {
            s.copy_count
                .set((s.copy_count.get() + 1).min(COPY_MESSAGES.len()));
        }
        s.copied.set(true);
        s.forced.set(true);
        self.refresh();
        self.schedule(COPY_FORCE_OPEN, |this| {
            this.0.forced.set(false);
            this.refresh();
            this.schedule(LEAVE_ANIMATION, |this| {
                if !this.0.forced.get() {
                    this.0.copied.set(false);
                    this.refresh();
                }
            });
        });
        self.schedule(COPY_STREAK_RESET, |this| this.0.copy_count.set(0));
    }
}

pub struct TooltipProps {
    pub text: String,
    pub position: TooltipPosition,
    pub color: TooltipColor,

    pub fonts: Option<TextFonts>,

    /// Shared hover and dismiss state. `None` creates one for this tooltip.
    pub state: Option<TooltipState>,

    /// A click on the anchor dismisses until the pointer leaves and returns.
    pub hide_on_click: bool,

    /// Wait before showing. `None` shows immediately.
    pub delay_ms: Option<u64>,

    /// Classes on the hover target, not the tooltip bubble.
    pub anchor_classes: Vec<String>,

    /// Wider bubble, for a label that should not wrap.
    pub large: bool,

    /// Cuts the label off, with an ellipsis, after this many lines.
    pub max_lines: Option<u32>,
}

impl Default for TooltipProps {
    fn default() -> Self {
        Self {
            text: String::new(),
            position: TooltipPosition::Top,
            color: TooltipColor::Primary,
            fonts: None,
            state: None,
            hide_on_click: true,
            delay_ms: None,
            anchor_classes: Vec::new(),
            large: false,
            max_lines: None,
        }
    }
}

crate::builder_fields!(TooltipProps {
    text: String,
    position: TooltipPosition,
    color: TooltipColor,
    opt fonts: TextFonts,
    opt state: TooltipState,
    hide_on_click: bool,
    opt delay_ms: u64,
    anchor_classes: Vec<String>,
    large: bool,
    opt max_lines: u32,
});

impl TooltipProps {
    pub fn build(self, anchor: Node) -> Node {
        tooltip(anchor, self)
    }
}

pub(crate) fn bubble(
    state: &TooltipState,
    position: TooltipPosition,
    color: TooltipColor,
    large: bool,
    max_lines: Option<u32>,
    fonts: TextFonts,
) -> Node {
    let (bubble_class, pointer_class, edge) = match position {
        TooltipPosition::Top => (cls::BUBBLE_TOP, cls::POINTER_TOP, PointerEdge::Bottom),
        TooltipPosition::Bottom => (cls::BUBBLE_BOTTOM, cls::POINTER_BOTTOM, PointerEdge::Top),
        TooltipPosition::Left => (cls::BUBBLE_LEFT, cls::POINTER_LEFT, PointerEdge::Right),
        TooltipPosition::Right => (cls::BUBBLE_RIGHT, cls::POINTER_RIGHT, PointerEdge::Left),
    };
    let label = text(TextProps {
        content: state.0.text.peek(),
        size: if large { 16.0 } else { 14.0 },
        line_height: Some(if large { 20.0 } else { 16.0 }),
        weight: if large {
            Weight::SEMI_BOLD
        } else {
            Weight::MEDIUM
        }
        .into(),
        color: Color::from_rgb(0xef, 0xef, 0xf1),
        fonts: Some(fonts),
        center: true,
        line_clamp: max_lines,
        ..Default::default()
    })
    .class(cls::TEXT)
    .bind_text(state.0.text.clone());
    let pointer = Node::new("div", TooltipPointerWidget::new(edge))
        .class(cls::POINTER)
        .class(pointer_class);
    div()
        .class(cls::BUBBLE)
        .class(bubble_class)
        .bind_class(state.0.feedback.clone(), cls::GREEN, color.class())
        .child(pointer)
        .child(div().class(cls::CONTENT).child(label))
}

pub fn tooltip(anchor: Node, props: TooltipProps) -> Node {
    let fonts = font_system::resolve(props.fonts);
    let state = props.state.unwrap_or_default();
    state.0.hide_on_click.set(props.hide_on_click);
    state.0.delay.set(props.delay_ms.map(Duration::from_millis));
    state.set_label(props.text);

    let layer_class = match props.position {
        TooltipPosition::Top => cls::LAYER_TOP,
        TooltipPosition::Bottom => cls::LAYER_BOTTOM,
        TooltipPosition::Left => cls::LAYER_LEFT,
        TooltipPosition::Right => cls::LAYER_RIGHT,
    };
    let bubble = bubble(
        &state,
        props.position,
        props.color,
        props.large,
        props.max_lines,
        fonts,
    );
    let layer = Node::new("div", TopLayerWidget)
        .class(cls::LAYER)
        .class(layer_class)
        .bind_class(state.0.visible.clone(), cls::SHOWN, cls::HIDDEN)
        .child(bubble);

    Node::new("div", TooltipAnchorWidget::new(state))
        .class(cls::ANCHOR)
        .classes(props.anchor_classes)
        .child(anchor)
        .child(layer)
}
