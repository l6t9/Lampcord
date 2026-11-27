use crate::colors;
use skia_safe::Color;

use crate::font_system::TextFonts;
use crate::node::Node;
use crate::typography::TextVariant;

use super::text::{TextProps, text};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum HeadingLevel {
    #[default]
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl HeadingLevel {
    pub fn default_variant(self) -> TextVariant {
        match self {
            HeadingLevel::H1 => TextVariant::HEADING_XXL_BOLD,
            HeadingLevel::H2 => TextVariant::HEADING_XL_BOLD,
            HeadingLevel::H3 => TextVariant::HEADING_LG_BOLD,
            HeadingLevel::H4 => TextVariant::HEADING_MD_BOLD,
            HeadingLevel::H5 => TextVariant::HEADING_SM_BOLD,
            HeadingLevel::H6 => TextVariant::HEADING_SM_SEMIBOLD,
        }
    }

    fn tag(self) -> &'static str {
        match self {
            HeadingLevel::H1 => "h1",
            HeadingLevel::H2 => "h2",
            HeadingLevel::H3 => "h3",
            HeadingLevel::H4 => "h4",
            HeadingLevel::H5 => "h5",
            HeadingLevel::H6 => "h6",
        }
    }
}

pub struct HeadingProps {
    pub level: HeadingLevel,
    pub content: String,

    /// Type size. Overrides the size implied by `level`.
    pub variant: TextVariant,
    pub color: Color,

    pub fonts: Option<TextFonts>,
}

impl Default for HeadingProps {
    fn default() -> Self {
        Self::new(HeadingLevel::default(), "")
    }
}

crate::builder_fields!(HeadingProps {
    level: HeadingLevel,
    content: String,
    variant: TextVariant,
    color: Color,
    opt fonts: TextFonts,
});

impl HeadingProps {
    pub fn new(level: HeadingLevel, content: impl Into<String>) -> Self {
        Self {
            level,
            content: content.into(),
            variant: level.default_variant(),
            color: colors::TEXT_STRONG,
            fonts: None,
        }
    }

    pub fn build(self) -> Node {
        heading(self)
    }
}

pub fn heading(props: HeadingProps) -> Node {
    let mut text_props = TextProps::styled(props.content, props.variant).color(props.color);
    text_props.fonts = props.fonts;
    let node = text(text_props);
    Node {
        tag: props.level.tag(),
        ..node
    }
}
