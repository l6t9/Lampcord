use crate::classes::popover as classes_popover;
use std::rc::Rc;

use crate::node::Node;
pub use crate::widget::anchor::Anchor;
use crate::widget::primitives::{ClickAreaWidget, DismissBackdropWidget};

#[derive(Default)]
pub struct PopoverProps {
    /// Window coordinates. Ignored when `anchor` is set.
    pub position: Option<(f32, f32)>,

    /// Places the popover against a rect. Wins over `position`.
    pub anchor: Option<Anchor>,

    pub on_hover_change: Option<Box<dyn FnMut(bool)>>,
    pub dom_id: Option<String>,
}

impl PopoverProps {
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.position = Some((x, y));
        self
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = Some(anchor);
        self
    }

    pub fn on_hover_change(mut self, f: impl FnMut(bool) + 'static) -> Self {
        self.on_hover_change = Some(Box::new(f));
        self
    }

    pub fn dom_id(mut self, id: impl Into<String>) -> Self {
        self.dom_id = Some(id.into());
        self
    }

    pub fn build(self) -> Node {
        popover(self)
    }
}

pub fn popover(props: PopoverProps) -> Node {
    let mut widget = ClickAreaWidget::new().with_swallowed_scroll();
    if let Some(anchor) = props.anchor {
        widget = widget.with_anchor(anchor);
    } else if let Some((x, y)) = props.position {
        widget = widget.with_absolute_position(x, y);
    }
    if let Some(f) = props.on_hover_change {
        widget = widget.with_on_hover_path(f);
    }
    let node = Node::new("div", widget);
    match props.dom_id {
        Some(id) => node.dom_id(id),
        None => node,
    }
}

pub fn popover_backdrop(on_dismiss: impl Fn() + 'static) -> Node {
    Node::new("div", DismissBackdropWidget::new(Rc::new(on_dismiss)))
        .class(classes_popover::BACKDROP)
}

pub fn anchored_popover_backdrop(on_dismiss: impl Fn() + 'static) -> Node {
    Node::new(
        "div",
        DismissBackdropWidget::covering_window(Rc::new(on_dismiss)),
    )
}
