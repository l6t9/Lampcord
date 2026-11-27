use crate::classes::text_link as classes_text_link;
use crate::colors::{TEXT_ANCHOR, TEXT_LINK};
use crate::font_system;
use crate::typography;
use skia_safe::Color;
use skia_safe::font_style::Weight;

use crate::font_system::TextFonts;
use crate::node::Node;
use crate::widget::primitives::ButtonWidget;

use super::button::OnClick;
use super::text::{TextProps, text};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TextLinkVariant {
    #[default]
    Primary,

    Secondary,

    /// Stays white on hover, for use over imagery.
    AlwaysWhite,

    Critical,

    /// Underlined, for a link inside a sentence.
    Anchor,
}

impl TextLinkVariant {
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "primary" => TextLinkVariant::Primary,
            "secondary" => TextLinkVariant::Secondary,
            "always-white" => TextLinkVariant::AlwaysWhite,
            "critical" => TextLinkVariant::Critical,
            "anchor" => TextLinkVariant::Anchor,
            _ => return None,
        })
    }
}

#[derive(Default)]
pub struct TextLinkProps {
    pub label: String,
    pub variant: TextLinkVariant,

    /// 14px instead of 16px.
    pub small: bool,

    pub disabled: bool,

    /// Type size and weight. Wins over `small` and the variant default.
    pub inline_variant: Option<typography::TextVariant>,
    pub on_click: OnClick,

    pub fonts: Option<TextFonts>,
}

crate::builder_fields!(TextLinkProps {
    label: String,
    variant: TextLinkVariant,
    small: bool,
    disabled: bool,
    opt inline_variant: typography::TextVariant,
    on_click: OnClick,
    opt fonts: TextFonts,
});

impl TextLinkProps {
    pub fn new(label: impl Into<String>, on_click: impl Into<OnClick>) -> Self {
        Self {
            label: label.into(),
            on_click: on_click.into(),
            ..Default::default()
        }
    }

    pub fn build(self) -> Node {
        text_link(self)
    }
}

pub fn text_link(props: TextLinkProps) -> Node {
    let fonts = font_system::resolve(props.fonts);
    let color = match props.variant {
        TextLinkVariant::Primary => TEXT_LINK,
        TextLinkVariant::Secondary => Color::from_argb(255, 0xfb, 0xfb, 0xfb),
        TextLinkVariant::AlwaysWhite => Color::WHITE,
        TextLinkVariant::Critical => Color::from_argb(255, 0xf8, 0x7e, 0x7a),
        TextLinkVariant::Anchor => TEXT_ANCHOR,
    };
    let (size, weight) = if let Some(variant) = props.inline_variant {
        (variant.size, variant.weight)
    } else if props.small || props.variant == TextLinkVariant::Anchor {
        (if props.small { 14.0 } else { 16.0 }, Weight::NORMAL)
    } else {
        (16.0, Weight::MEDIUM)
    };
    let line_height = props.inline_variant.map(|variant| variant.line_height);
    let mut widget = ButtonWidget::new();
    if let Some(on_click) = props.on_click.0.filter(|_| !props.disabled) {
        widget = widget.with_on_click(on_click);
    }
    let mut node = Node::new("button", widget)
        .class(classes_text_link::ROOT)
        .child(
            text(TextProps {
                content: props.label,
                size,
                color,
                weight: weight.into(),
                line_height,
                fonts: Some(fonts),
                ..Default::default()
            })
            .class(classes_text_link::LABEL),
        );
    if props.disabled {
        node = node.class(classes_text_link::DISABLED);
    }
    node
}
