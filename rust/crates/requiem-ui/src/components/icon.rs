use crate::widget::primitives;
use skia_safe::Color;

use crate::icon_handle::IconPath;
use crate::node::Node;
use crate::widget::primitives::IconWidget;

pub fn icon(icon: impl IconPath, color: Color) -> Node {
    match IconWidget::new(icon, color) {
        Some(widget) => Node::new("span", widget),
        None => Node::new("span", primitives::BoxWidget),
    }
}

pub fn icon_rotated(icon: impl IconPath, color: Color, degrees: f32) -> Node {
    match IconWidget::new(icon, color) {
        Some(widget) => Node::new("span", widget.with_rotation(degrees)),
        None => Node::new("span", primitives::BoxWidget),
    }
}

pub fn icon_with_hover(icon: impl IconPath, color: Color, hover_color: Color) -> Node {
    match IconWidget::new(icon, color) {
        Some(widget) => Node::new("span", widget.with_hover_color(hover_color)),
        None => Node::new("span", primitives::BoxWidget),
    }
}
