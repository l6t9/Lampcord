use crate::classes::stack as cls;
use crate::node::Node;

use super::container::div;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Space(pub f32);

impl Space {
    pub const ZERO: Space = Space(0.0);
    pub const XXS: Space = Space(4.0);
    pub const XS: Space = Space(8.0);
    pub const SM: Space = Space(12.0);
    pub const MD: Space = Space(16.0);
    pub const LG: Space = Space(20.0);
    pub const XL: Space = Space(24.0);
    pub const XXL: Space = Space(32.0);
}

impl From<i32> for Space {
    fn from(px: i32) -> Self {
        Space(px as f32)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Padding {
    pub top: Option<Space>,
    pub right: Option<Space>,
    pub bottom: Option<Space>,
    pub left: Option<Space>,
}

impl Padding {
    pub fn all(space: impl Into<Space>) -> Self {
        let s = Some(space.into());
        Self {
            top: s,
            right: s,
            bottom: s,
            left: s,
        }
    }

    pub fn x(space: impl Into<Space>) -> Self {
        let s = Some(space.into());
        Self {
            right: s,
            left: s,
            ..Self::default()
        }
    }

    pub fn y(space: impl Into<Space>) -> Self {
        let s = Some(space.into());
        Self {
            top: s,
            bottom: s,
            ..Self::default()
        }
    }

    pub fn top(space: impl Into<Space>) -> Self {
        Self {
            top: Some(space.into()),
            ..Self::default()
        }
    }

    pub fn right(space: impl Into<Space>) -> Self {
        Self {
            right: Some(space.into()),
            ..Self::default()
        }
    }

    pub fn bottom(space: impl Into<Space>) -> Self {
        Self {
            bottom: Some(space.into()),
            ..Self::default()
        }
    }

    pub fn left(space: impl Into<Space>) -> Self {
        Self {
            left: Some(space.into()),
            ..Self::default()
        }
    }

    pub fn with_top(mut self, space: impl Into<Space>) -> Self {
        self.top = Some(space.into());
        self
    }

    pub fn with_right(mut self, space: impl Into<Space>) -> Self {
        self.right = Some(space.into());
        self
    }

    pub fn with_bottom(mut self, space: impl Into<Space>) -> Self {
        self.bottom = Some(space.into());
        self
    }

    pub fn with_left(mut self, space: impl Into<Space>) -> Self {
        self.left = Some(space.into());
        self
    }

    fn css(&self) -> String {
        let side = |name: &str, v: Option<Space>| {
            v.filter(|s| s.0 != 0.0)
                .map(|s| format!("padding-{name}: {}px; ", s.0))
                .unwrap_or_default()
        };
        format!(
            "{}{}{}{}",
            side("top", self.top),
            side("right", self.right),
            side("bottom", self.bottom),
            side("left", self.left)
        )
    }
}

impl From<Space> for Padding {
    fn from(space: Space) -> Self {
        Padding::all(space)
    }
}

impl From<i32> for Padding {
    fn from(px: i32) -> Self {
        Padding::all(px)
    }
}

macro_rules! enum_with_default {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => $text:literal),+ $(,)? } default $default:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $name {
            $($variant),+
        }

        impl Default for $name {
            fn default() -> Self {
                $name::$default
            }
        }
    };
}

enum_with_default!(

    Direction { Vertical => "vertical", Horizontal => "horizontal", VerticalReverse => "vertical-reverse", HorizontalReverse => "horizontal-reverse" } default Vertical
);

enum_with_default!(

    Align { Start => "start", Center => "center", End => "end", Stretch => "stretch", Baseline => "baseline" } default Stretch
);

enum_with_default!(

    Justify { Start => "start", Center => "center", End => "end", SpaceBetween => "space-between", SpaceAround => "space-around" } default Start
);

pub struct StackProps {
    pub direction: Direction,
    pub align: Align,
    pub justify: Justify,
    /// Children wrap onto the next line instead of overflowing.
    pub wrap: bool,

    pub gap: Space,
    pub padding: Padding,

    /// Stretch to the container width.
    pub full_width: bool,
}

impl Default for StackProps {
    fn default() -> Self {
        Self {
            direction: Direction::Vertical,
            align: Align::Stretch,
            justify: Justify::Start,
            wrap: false,
            gap: Space(8.0),
            padding: Padding::default(),
            full_width: true,
        }
    }
}

crate::builder_fields!(StackProps {
    direction: Direction,
    align: Align,
    justify: Justify,
    wrap: bool,
    gap: Space,
    padding: Padding,
    full_width: bool,
});

impl StackProps {
    pub fn vertical() -> Self {
        Self::default()
    }

    pub fn horizontal() -> Self {
        Self {
            direction: Direction::Horizontal,
            ..Self::default()
        }
    }

    pub fn build(self) -> Node {
        stack(self)
    }
}

pub fn stack(props: StackProps) -> Node {
    let StackProps {
        direction,
        align,
        justify,
        wrap,
        gap,
        padding,
        full_width,
    } = props;
    let direction_class = match direction {
        Direction::Vertical => cls::VERTICAL,
        Direction::Horizontal => cls::HORIZONTAL,
        Direction::VerticalReverse => cls::VERTICAL_REVERSE,
        Direction::HorizontalReverse => cls::HORIZONTAL_REVERSE,
    };
    let align_class = match align {
        Align::Start => cls::ALIGN_START,
        Align::Center => cls::ALIGN_CENTER,
        Align::End => cls::ALIGN_END,
        Align::Stretch => cls::ALIGN_STRETCH,
        Align::Baseline => cls::ALIGN_BASELINE,
    };
    let justify_class = match justify {
        Justify::Start => cls::JUSTIFY_START,
        Justify::Center => cls::JUSTIFY_CENTER,
        Justify::End => cls::JUSTIFY_END,
        Justify::SpaceBetween => cls::JUSTIFY_SPACE_BETWEEN,
        Justify::SpaceAround => cls::JUSTIFY_SPACE_AROUND,
    };
    let mut node = div()
        .class(cls::ROOT)
        .class(direction_class)
        .class(align_class)
        .class(justify_class);
    if wrap {
        node = node.class(cls::WRAP);
    }
    if full_width {
        node = node.class(cls::FULL_WIDTH);
    }
    node.style(format!("gap: {}px; {}", gap.0, padding.css()))
}

#[cfg(test)]
mod tests {
    use crate::layout;
    use crate::paint;
    use crate::style::ComputedStyle;
    use crate::style::cascade;
    use crate::style::parser;
    use crate::widget::WidgetTree;
    use crate::widget::primitives::BoxWidget;
    use super::*;
    use taffy::AvailableSpace;
    use taffy::Size;

    #[test]
    fn padding_css_skips_unset_sides() {
        assert_eq!(
            Padding::top(8).with_left(16).css(),
            "padding-top: 8px; padding-left: 16px; "
        );
        assert_eq!(
            Padding::all(4).css(),
            "padding-top: 4px; padding-right: 4px; padding-bottom: 4px; padding-left: 4px; "
        );
        assert_eq!(Padding::default().css(), "");
    }

    #[test]
    fn lays_out_with_gap_direction_and_padding() {
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let root = tree.root;
        tree.nodes[root].classes.push("root".to_string());
        let child = || div().style("width: 20px; height: 10px; flex-shrink: 0");
        let row = tree.mount(
            root,
            StackProps::horizontal()
                .gap(16)
                .padding(Padding::top(8).with_left(4))
                .align(Align::Start)
                .build()
                .children([child(), child()]),
        );
        let column = tree.mount(
            root,
            StackProps::vertical()
                .gap(Space::XS)
                .align(Align::Start)
                .build()
                .children([child(), child()]),
        );
        let theme = parser::parse_stylesheet(&format!(
            "{}
.root {{ display: flex; flex-direction: column; width: 400px; height: 400px; }}",
            crate::BUNDLED_CSS
        ));
        cascade::apply_stylesheet(&mut tree, &theme);
        layout::compute_layout(
            &mut tree,
            Size {
                width: AvailableSpace::Definite(400.0),
                height: AvailableSpace::Definite(400.0),
            },
        );
        let rects = paint::all_absolute_rects(&tree);
        let first = |id| rects[&tree.nodes[id].children[0]];
        let second = |id| rects[&tree.nodes[id].children[1]];

        assert_eq!(
            (first(row).x - rects[&row].x, first(row).y - rects[&row].y),
            (4.0, 8.0)
        );
        assert_eq!(second(row).x - first(row).x, 20.0 + 16.0);
        assert_eq!(second(row).y, first(row).y);

        assert_eq!(second(column).y - first(column).y, 10.0 + 8.0);
        assert_eq!(second(column).x, first(column).x);
    }

    #[test]
    fn enum_defaults() {
        assert_eq!(Align::default(), Align::Stretch);
    }
}
