use crate::classes::base_popover as cls;
use crate::node::Node;
use crate::widget::AbsoluteRect;
use crate::widget::primitives::{PointerEdge, PopoverCaretWidget, PopoverPanelWidget};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopoverSide {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

impl PopoverSide {
    fn opposite(self) -> Self {
        match self {
            PopoverSide::Top => PopoverSide::Bottom,
            PopoverSide::Bottom => PopoverSide::Top,
            PopoverSide::Left => PopoverSide::Right,
            PopoverSide::Right => PopoverSide::Left,
        }
    }

    fn horizontal(self) -> bool {
        matches!(self, PopoverSide::Top | PopoverSide::Bottom)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopoverAlign {
    Start,
    #[default]
    Center,
    End,
}

pub const DEFAULT_WIDTH: f32 = 240.0;

const CARET_EDGE_OFFSET: f32 = 20.0;
const CARET_LONG: f32 = 22.0;

const GUTTER: f32 = 14.0;

const WINDOW_MARGIN: f32 = 8.0;

pub fn fit(
    anchor: AbsoluteRect,
    panel: (f32, f32),
    preferred: PopoverSide,
    align: PopoverAlign,
    window: (f32, f32),
) -> (PopoverSide, f32) {
    let (w, h) = panel;
    let fits = |side: PopoverSide| match side {
        PopoverSide::Top => anchor.y - h - GUTTER >= 0.0,
        PopoverSide::Bottom => anchor.y + anchor.height + GUTTER + h <= window.1,
        PopoverSide::Left => anchor.x - w - GUTTER >= 0.0,
        PopoverSide::Right => anchor.x + anchor.width + GUTTER + w <= window.0,
    };
    let side = if !fits(preferred) && fits(preferred.opposite()) {
        preferred.opposite()
    } else {
        preferred
    };
    let (start, length, size, limit) = if side.horizontal() {
        (anchor.x, anchor.width, w, window.0)
    } else {
        (anchor.y, anchor.height, h, window.1)
    };
    let cross = match align {
        PopoverAlign::Start => start,
        PopoverAlign::End => start + length - size,
        PopoverAlign::Center => start + length / 2.0 - size / 2.0,
    };
    let clamped = cross.clamp(
        WINDOW_MARGIN,
        (limit - size - WINDOW_MARGIN).max(WINDOW_MARGIN),
    );
    (side, clamped - cross)
}

pub struct BasePopoverProps {
    pub side: PopoverSide,
    pub align: PopoverAlign,
    /// Panel width in px. Height comes from the children.
    pub width: f32,
    pub hide_caret: bool,

    /// Caret position along the edge, in px. `None` uses the default inset.
    pub caret_offset: Option<f32>,

    /// Extra px shift along the anchor edge after placement.
    pub shift: f32,
    pub dom_id: Option<String>,
    pub children: Vec<Node>,
}

impl BasePopoverProps {
    pub fn new(children: Vec<Node>) -> Self {
        Self {
            side: PopoverSide::Top,
            align: PopoverAlign::Center,
            width: DEFAULT_WIDTH,
            hide_caret: false,
            caret_offset: None,
            shift: 0.0,
            dom_id: None,
            children,
        }
    }

    pub fn side(mut self, side: PopoverSide) -> Self {
        self.side = side;
        self
    }

    pub fn align(mut self, align: PopoverAlign) -> Self {
        self.align = align;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn hide_caret(mut self, hide: bool) -> Self {
        self.hide_caret = hide;
        self
    }

    pub fn caret_offset(mut self, offset: f32) -> Self {
        self.caret_offset = Some(offset);
        self
    }

    pub fn shift(mut self, shift: f32) -> Self {
        self.shift = shift;
        self
    }

    pub fn dom_id(mut self, id: impl Into<String>) -> Self {
        self.dom_id = Some(id.into());
        self
    }

    pub fn build(self) -> Node {
        base_popover(self)
    }
}

pub fn base_popover(props: BasePopoverProps) -> Node {
    let BasePopoverProps {
        side,
        align,
        width,
        hide_caret,
        caret_offset,
        shift,
        dom_id,
        children,
    } = props;
    let horizontal = side.horizontal();
    let (place_class, caret_class, caret_edge) = match side {
        PopoverSide::Top => (cls::PLACE_TOP, cls::CARET_BOTTOM, PointerEdge::Bottom),
        PopoverSide::Bottom => (cls::PLACE_BOTTOM, cls::CARET_TOP, PointerEdge::Top),
        PopoverSide::Left => (cls::PLACE_LEFT, cls::CARET_RIGHT, PointerEdge::Right),
        PopoverSide::Right => (cls::PLACE_RIGHT, cls::CARET_LEFT, PointerEdge::Left),
    };

    let align = if horizontal {
        align
    } else if align == PopoverAlign::End {
        PopoverAlign::End
    } else {
        PopoverAlign::Start
    };
    let align_class = match (horizontal, align) {
        (true, PopoverAlign::Start) => cls::ALIGN_START,
        (true, PopoverAlign::Center) => cls::ALIGN_CENTER,
        (true, PopoverAlign::End) => cls::ALIGN_END,
        (false, PopoverAlign::End) => cls::ALIGN_BOTTOM,
        (false, _) => cls::ALIGN_TOP,
    };
    let mut style = format!("width: {width}px");

    match (horizontal, align) {
        (true, PopoverAlign::Center) => {
            style.push_str(&format!("; margin-left: {}px", shift - width / 2.0))
        }
        (true, PopoverAlign::Start) => style.push_str(&format!("; margin-left: {shift}px")),
        (true, PopoverAlign::End) => style.push_str(&format!("; margin-right: {}px", -shift)),
        (false, PopoverAlign::End) => style.push_str(&format!("; margin-bottom: {}px", -shift)),
        (false, _) => style.push_str(&format!("; margin-top: {shift}px")),
    }
    let mut panel = Node::new("div", PopoverPanelWidget::new())
        .class(cls::PANEL)
        .class(place_class)
        .class(align_class)
        .style(style);
    if let Some(id) = dom_id {
        panel = panel.dom_id(id);
    }
    panel = panel.children(children);
    if !hide_caret {
        let offset = caret_offset.unwrap_or(CARET_EDGE_OFFSET);

        let caret_style = match (horizontal, align) {
            (true, PopoverAlign::Start) => format!("left: {}px", offset - shift),
            (true, PopoverAlign::End) => format!("right: {}px", offset + shift),
            (true, PopoverAlign::Center) => {
                format!("left: 50%; margin-left: {}px", -CARET_LONG / 2.0 - shift)
            }
            (false, PopoverAlign::End) => format!("bottom: {}px", offset + shift),
            (false, _) => format!("top: {}px", offset - shift),
        };

        panel = panel.child(
            Node::new("div", PopoverCaretWidget::new(caret_edge))
                .class(cls::CARET)
                .class(caret_class)
                .style(caret_style),
        );
    }
    panel
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOW: (f32, f32) = (1000.0, 800.0);

    fn anchor(x: f32, y: f32) -> AbsoluteRect {
        AbsoluteRect {
            x,
            y,
            width: 32.0,
            height: 32.0,
        }
    }

    #[test]
    fn a_panel_with_room_stays_where_it_was_asked_to_be() {
        let (side, shift) = fit(
            anchor(500.0, 400.0),
            (240.0, 200.0),
            PopoverSide::Top,
            PopoverAlign::Center,
            WINDOW,
        );
        assert_eq!((side, shift), (PopoverSide::Top, 0.0));
    }

    #[test]
    fn a_panel_that_would_leave_the_window_flips_to_the_side_that_fits() {
        let (side, _) = fit(
            anchor(500.0, 100.0),
            (240.0, 200.0),
            PopoverSide::Top,
            PopoverAlign::Center,
            WINDOW,
        );
        assert_eq!(side, PopoverSide::Bottom);
        let (side, _) = fit(
            anchor(900.0, 400.0),
            (240.0, 200.0),
            PopoverSide::Right,
            PopoverAlign::Start,
            WINDOW,
        );
        assert_eq!(side, PopoverSide::Left);
    }

    #[test]
    fn it_stays_put_when_neither_side_fits() {
        let (side, _) = fit(
            anchor(500.0, 300.0),
            (240.0, 700.0),
            PopoverSide::Top,
            PopoverAlign::Center,
            WINDOW,
        );
        assert_eq!(side, PopoverSide::Top);
    }

    #[test]
    fn a_panel_near_the_window_edge_is_pushed_back_in_and_the_caret_follows() {
        let (side, shift) = fit(
            anchor(960.0, 400.0),
            (240.0, 200.0),
            PopoverSide::Top,
            PopoverAlign::Center,
            WINDOW,
        );
        assert_eq!(side, PopoverSide::Top);
        assert_eq!(976.0 - 120.0 + shift + 240.0, 992.0);
        assert!(shift < 0.0);

        let (_, shift) = fit(
            anchor(4.0, 400.0),
            (240.0, 200.0),
            PopoverSide::Top,
            PopoverAlign::Center,
            WINDOW,
        );
        assert!(shift > 0.0);
    }
}
