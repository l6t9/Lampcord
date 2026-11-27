use skia_safe::Color;

use crate::node::Node;
use crate::widget::primitives::ScrollViewWidget;

pub fn scroll_view() -> Node {
    Node::new("div", ScrollViewWidget::new())
}

#[derive(Default)]
/// `(offset, max_offset)`, both in px.
pub struct OnScroll(pub Option<Box<dyn FnMut(f32, f32)>>);

impl<F: FnMut(f32, f32) + 'static> From<F> for OnScroll {
    fn from(f: F) -> Self {
        OnScroll(Some(Box::new(f)))
    }
}
#[derive(Clone)]
pub enum TopFade {
    Color(Color),
    Token(&'static str),
}

impl From<Color> for TopFade {
    fn from(color: Color) -> Self {
        TopFade::Color(color)
    }
}

impl From<&'static str> for TopFade {
    fn from(token: &'static str) -> Self {
        TopFade::Token(token)
    }
}

#[derive(Default)]
pub struct ScrollViewProps {
    /// Applied once, on the first layout.
    pub initial_offset_y: f32,

    pub on_scroll: OnScroll,

    /// Fade painted over the top edge while scrolled. A token name resolves
    /// each frame; a color is used as given.
    pub top_fade: Option<TopFade>,
}

crate::builder_fields!(ScrollViewProps {
    initial_offset_y: f32,
    on_scroll: OnScroll,
    opt top_fade: TopFade,
});

impl ScrollViewProps {
    pub fn build(self) -> Node {
        scroll_view_with(self)
    }
}

pub fn scroll_view_with(props: ScrollViewProps) -> Node {
    let mut widget = ScrollViewWidget::new().with_initial_offset(props.initial_offset_y);
    if let Some(on_scroll) = props.on_scroll.0 {
        widget = widget.with_on_scroll(on_scroll);
    }
    if let Some(color) = props.top_fade {
        widget = widget.with_top_fade(color);
    }
    Node::new("div", widget)
}
