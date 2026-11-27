use crate::node::Node;
use crate::widget::primitives::BoxWidget;

pub fn div() -> Node {
    Node::new("div", BoxWidget)
}
