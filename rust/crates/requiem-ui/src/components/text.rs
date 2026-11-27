use crate::classes::text as classes_text;
use crate::colors;
use crate::emoji_display;
use crate::font_system;
use std::borrow::Cow;
use std::ops;
use std::rc::Rc;

use skia_safe::Color;
use skia_safe::font_style::Weight;

use crate::font_system::TextFonts;
use crate::node::Node;
use crate::text_cache::MeasureCache;
use crate::typography::TextVariant;
use crate::widget::primitives::TextWidget;

#[derive(Clone, Copy)]
pub struct TextWeight(pub Weight);

impl Default for TextWeight {
    fn default() -> Self {
        TextWeight(Weight::NORMAL)
    }
}

impl From<Weight> for TextWeight {
    fn from(weight: Weight) -> Self {
        TextWeight(weight)
    }
}

#[derive(Default)]
pub struct TextProps {
    pub content: String,

    pub size: f32,
    pub color: Color,

    /// Color while the pointer is over the text. `None` keeps `color`.
    pub hover_color: Option<Color>,
    pub weight: TextWeight,

    pub fonts: Option<TextFonts>,

    /// Reuses a previous wrap measurement for `cache_key`.
    pub measure_cache: Option<Rc<MeasureCache>>,

    pub cache_key: Option<u64>,

    /// Center each line in the box.
    pub center: bool,

    /// Byte ranges drawn at a different weight.
    pub strong: Vec<(ops::Range<usize>, Weight)>,

    pub line_height: Option<f32>,

    pub italic: bool,

    /// Paints uppercase. `content` is unchanged.
    pub uppercase: bool,

    /// Extra px between glyphs. `0` is the font default.
    pub letter_spacing: f32,

    /// Max lines before an ellipsis. `None` wraps freely.
    pub line_clamp: Option<u32>,

    /// Fill colors. Empty uses `color`.
    pub gradient: Vec<Color>,
    /// Repeat width of `gradient`, in px.
    pub gradient_tile: f32,
}

crate::builder_fields!(TextProps {
    content: String,
    size: f32,
    color: Color,
    opt hover_color: Color,
    weight: TextWeight,
    opt fonts: TextFonts,
    opt measure_cache: Rc<MeasureCache>,
    opt cache_key: u64,
    center: bool,
    strong: Vec<(ops::Range<usize>, Weight)>,
    opt line_height: f32,
    italic: bool,
    uppercase: bool,
    letter_spacing: f32,
    opt line_clamp: u32,
    gradient: Vec<Color>,
    gradient_tile: f32,
});

impl TextProps {
    pub fn new(content: impl Into<String>, size: f32) -> Self {
        Self {
            content: content.into(),
            size,
            color: colors::TEXT_DEFAULT,
            ..Default::default()
        }
    }

    pub fn styled(content: impl Into<String>, variant: impl Into<TextVariant>) -> Self {
        Self {
            content: content.into(),
            color: colors::TEXT_DEFAULT,
            ..Default::default()
        }
        .variant(variant)
    }

    pub fn variant(mut self, variant: impl Into<TextVariant>) -> Self {
        let v = variant.into();
        self.size = v.size;
        self.weight = TextWeight(v.weight);
        self.line_height = Some(v.line_height);
        self.uppercase = v.uppercase;
        self.letter_spacing = v.letter_spacing;
        self
    }

    pub fn build(self) -> Node {
        text(self)
    }
}

pub fn bold_markup(content: &str) -> (String, Vec<(ops::Range<usize>, Weight)>) {
    let mut out = String::with_capacity(content.len());
    let mut spans = Vec::new();
    let mut rest = content;
    while let Some(start) = rest.find("**") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("**") else { break };
        out.push_str(&rest[..start]);
        let from = out.len();
        out.push_str(&after[..end]);
        spans.push((from..out.len(), Weight::SEMI_BOLD));
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    (out, spans)
}

impl TextProps {
    pub fn bold_markup(mut self) -> Self {
        let (content, spans) = bold_markup(&self.content);
        self.content = content;
        self.strong.extend(spans);
        self
    }
}

pub fn line_clamp_class(lines: u32) -> &'static str {
    match lines {
        0 | 1 => classes_text::LINE_CLAMP_1,
        2 => classes_text::LINE_CLAMP_2,
        3 => classes_text::LINE_CLAMP_3,
        _ => classes_text::LINE_CLAMP_4,
    }
}

pub fn text(props: TextProps) -> Node {
    let fonts = font_system::resolve(props.fonts);
    let content = if props.uppercase {
        props.content.to_uppercase()
    } else {
        props.content
    };

    let content = match emoji_display::apply(&content) {
        Cow::Borrowed(_) => content,
        Cow::Owned(shown) => shown,
    };
    let mut widget = TextWidget::new_weighted(
        content,
        props.size,
        props.color,
        fonts.font_system,
        fonts.glyph_cache,
        props.weight.0,
    );
    if let Some(hover_color) = props.hover_color {
        widget = widget.with_hover_color(hover_color);
    }
    if props.center {
        widget = widget.with_center();
    }

    if let Some(line_height) = props
        .line_height
        .or_else(|| TextVariant::line_height_for(props.size))
    {
        widget = widget.with_line_height(line_height);
    }
    if props.italic {
        widget = widget.with_italic();
    }
    if props.letter_spacing != 0.0 {
        widget = widget.with_letter_spacing(props.letter_spacing);
    }
    if !props.gradient.is_empty() {
        widget = widget.with_gradient(
            props.gradient,
            if props.gradient_tile > 0.0 {
                props.gradient_tile
            } else {
                100.0
            },
        );
    }
    for (range, weight) in props.strong {
        widget = widget.with_strong_range(range, weight);
    }
    if let Some(cache) = props.measure_cache {
        widget = widget.with_measure_cache(cache, props.cache_key);
    }
    let node = Node::new("span", widget);
    match props.line_clamp {
        Some(lines) => node.class(line_clamp_class(lines)),
        None => node,
    }
}
