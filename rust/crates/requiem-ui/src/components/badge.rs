use crate::classes::badge as cls;
use crate::font_system;
use crate::icon_handle;
use skia_safe::Color;
use skia_safe::font_style::Weight;

use crate::font_system::TextFonts;
use crate::node::Node;
use crate::widget::primitives::{BoxWidget, GradientRingWidget};

use super::icon::icon;
use super::text::{TextProps, text};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Brand,
    Expressive,
    Common,
    Rare,
    Epic,
    Mythic,

    /// Error red, not a rarity tier.
    Critical,
}

impl BadgeVariant {
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "default" => BadgeVariant::Default,
            "brand" => BadgeVariant::Brand,
            "expressive" => BadgeVariant::Expressive,
            "common" => BadgeVariant::Common,
            "rare" => BadgeVariant::Rare,
            "epic" => BadgeVariant::Epic,
            "mythic" => BadgeVariant::Mythic,
            "critical" => BadgeVariant::Critical,
            _ => return None,
        })
    }
}

#[derive(Default)]
pub struct BadgeProps {
    pub text: String,
    pub variant: BadgeVariant,

    pub icon: Option<icon_handle::IconSpec>,

    pub fonts: Option<TextFonts>,
}

crate::builder_fields!(BadgeProps {
    text: String,
    variant: BadgeVariant,
    opt icon: icon_handle::IconSpec,
    opt fonts: TextFonts,
});

impl BadgeProps {
    pub fn build(self) -> Node {
        badge(self)
    }
}

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_argb(255, r, g, b)
}

pub fn badge(props: BadgeProps) -> Node {
    let fonts = font_system::resolve(props.fonts);
    let (class, text_color) = match props.variant {
        BadgeVariant::Default => (cls::DEFAULT, rgb(0xfb, 0xfb, 0xfb)),
        BadgeVariant::Brand => (cls::BRAND, rgb(0xff, 0xff, 0xff)),
        BadgeVariant::Expressive => (cls::EXPRESSIVE, rgb(0x2f, 0x30, 0x35)),
        BadgeVariant::Common => (cls::COMMON, rgb(0xc9, 0xca, 0xce)),
        BadgeVariant::Rare => (cls::RARE, rgb(0xa0, 0xa9, 0xfa)),
        BadgeVariant::Epic => (cls::EPIC, rgb(0xca, 0x9e, 0xf9)),
        BadgeVariant::Mythic => (cls::MYTHIC, rgb(0xfe, 0x92, 0x42)),
        BadgeVariant::Critical => (cls::CRITICAL, rgb(0xff, 0xff, 0xff)),
    };

    let ring = match props.variant {
        BadgeVariant::Rare => Some((rgb(0x58, 0x65, 0xf2), rgb(0x2d, 0x35, 0xad))),
        BadgeVariant::Epic => Some((rgb(0xff, 0x4c, 0xd2), rgb(0xa0, 0x56, 0xf2))),
        BadgeVariant::Mythic => Some((rgb(0xff, 0xe0, 0x47), rgb(0xfe, 0x6e, 0x0d))),
        _ => None,
    };
    let mut node = match ring {
        Some((start, end)) => Node::new("div", GradientRingWidget::new(start, end, -78.0)),
        None => Node::new("div", BoxWidget),
    }
    .class(cls::BADGE)
    .class(class);
    if let Some(path) = props.icon {
        node = node.child(icon(&path, text_color).class(cls::ICON));
    }
    node.child(text(TextProps {
        content: props.text.to_uppercase(),
        size: 12.0,
        color: text_color,
        weight: Weight::BOLD.into(),
        fonts: Some(fonts),
        ..Default::default()
    }))
}
