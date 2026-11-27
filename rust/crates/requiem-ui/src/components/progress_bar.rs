use crate::classes::progress_bar as cls;
use crate::node::Node;
use crate::widget::primitives::ProgressFillWidget;

use super::container::div;

pub use crate::widget::primitives::forget_progress;

pub struct ProgressBarProps {
    /// Identity for the fill animation. A new key restarts it.
    pub key: u64,

    /// `0.0` to `1.0`. Values outside that clamp.
    pub fraction: f32,
}

impl ProgressBarProps {
    pub fn new(key: u64, fraction: f32) -> Self {
        Self { key, fraction }
    }

    pub fn build(self) -> Node {
        progress_bar(self)
    }
}

pub fn progress_bar(props: ProgressBarProps) -> Node {
    div().class(cls::TRACK).child(
        Node::new(
            "div",
            ProgressFillWidget::new(props.key, props.fraction.clamp(0.0, 1.0)),
        )
        .class(cls::FILL),
    )
}
