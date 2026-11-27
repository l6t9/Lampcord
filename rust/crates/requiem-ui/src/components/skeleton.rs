use crate::classes::skeleton as cls;
use crate::node::Node;

use super::container::div;

pub fn skeleton() -> Node {
    div().class(cls::BAR)
}

#[cfg(test)]
mod tests {
    use crate::style::ComputedStyle;
    use crate::style::cascade;
    use crate::style::parser;
    use crate::widget::WidgetTree;
    use crate::widget::primitives::BoxWidget;
    use super::*;

    #[test]
    fn a_skeleton_pulses_through_the_real_cascade() {
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let bar = tree.mount(tree.root, skeleton().style("width: 40px; height: 12px"));
        let theme = parser::parse_stylesheet(crate::BUNDLED_CSS);
        cascade::apply_stylesheet(&mut tree, &theme);

        let cascaded = tree.nodes[bar]
            .style
            .background
            .expect("the bar has its subtle background");
        let node = &mut tree.nodes[bar];
        assert_eq!(
            node.style.animations.len(),
            1,
            "the animation shorthand cascaded"
        );
        let sampled = node
            .animations
            .sample(&node.style)
            .expect("the pulse is running");
        assert!(
            sampled.running && !sampled.spills,
            "a recolour only repaints the bar's own box"
        );

        assert_eq!(sampled.style.background, Some(cascaded));
    }
}
