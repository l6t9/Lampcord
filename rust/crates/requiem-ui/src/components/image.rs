use skia_safe::Image;

use crate::node::Node;
use crate::widget::primitives::ImageWidget;

pub fn image(image: Option<Image>, width: f32, height: f32) -> Node {
    Node::new(
        "div",
        ImageWidget::new(image).with_intrinsic_size(width, height),
    )
}
