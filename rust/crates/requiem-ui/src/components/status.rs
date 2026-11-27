use crate::classes::status;
use skia_safe::Color;

use crate::node::Node;
use crate::widget::primitives::{PresenceStatus, StatusIndicatorWidget};

pub fn status_indicator(status: PresenceStatus) -> Node {
    let class = match status {
        PresenceStatus::Online => status::ONLINE,
        PresenceStatus::Idle => status::IDLE,
        PresenceStatus::Dnd => status::DND,
        PresenceStatus::Offline => status::OFFLINE,
    };
    Node::new(
        "div",
        StatusIndicatorWidget::new(status, Color::from_rgb(0x84, 0x85, 0x8d)),
    )
    .class(class)
}
