use crate::classes::form_section as cls;
use crate::font_system;
use crate::icons;
use skia_safe::Color;
use skia_safe::font_style::Weight;

use crate::font_system::TextFonts;
use crate::node::Node;

use super::badge::{BadgeProps, BadgeVariant, badge};
use super::container::div;
use super::icon::icon;
use super::text::{TextProps, text};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeatureTag {
    New,

    Premium,
}

pub fn premium_section_tags() -> Vec<FeatureTag> {
    vec![FeatureTag::Premium]
}

#[derive(Default)]
pub struct FormSectionProps {
    pub title: String,
    pub tags: Vec<FeatureTag>,
    pub description: Option<String>,

    /// Rule under the section.
    pub divider: bool,

    /// Tinted background, for a section that should stand out from the form.
    pub highlighted: bool,
    pub children: Vec<Node>,

    pub fonts: Option<TextFonts>,
}


crate::builder_fields!(FormSectionProps {
    title: String,
    tags: Vec<FeatureTag>,
    opt description: String,
    divider: bool,
    highlighted: bool,
    children: Vec<Node>,
    opt fonts: TextFonts,
});

impl FormSectionProps {
    pub fn build(self) -> Node {
        form_section(self)
    }
}

pub fn feature_tag(tag: FeatureTag) -> Node {
    let fonts = &font_system::resolve(None);
    match tag {
        FeatureTag::New => badge(
            BadgeProps::default()
                .text(requiem_i18n::t!("ui-new-badge"))
                .variant(BadgeVariant::Critical)
                .fonts(fonts.clone()),
        ),
        FeatureTag::Premium => {
            icon(icons::NITRO_WHEEL, Color::from_rgb(0xc9, 0xcd, 0xfb)).class(cls::TAG_ICON)
        }
    }
}

pub fn form_section(props: FormSectionProps) -> Node {
    let fonts = font_system::resolve(props.fonts);
    let mut title_row = div().class(cls::TITLE_ROW).child(text(TextProps {
        content: props.title,
        size: 16.0,
        color: Color::from_rgb(0xfb, 0xfb, 0xfb),
        weight: Weight::MEDIUM.into(),
        fonts: Some(fonts.clone()),
        ..Default::default()
    }));
    for tag in props.tags {
        title_row = title_row.child(feature_tag(tag));
    }
    let mut node = div().class(cls::SECTION).child(title_row);
    if let Some(description) = props.description {
        node = node.child(text(TextProps {
            content: description,
            size: 14.0,
            color: Color::from_rgb(0xef, 0xef, 0xf1),
            fonts: Some(fonts.clone()),
            ..Default::default()
        }));
    }
    node = node.children(props.children);
    if props.highlighted {
        node = node.class(cls::HIGHLIGHTED);
    }
    node.class(if props.divider {
        cls::DIVIDER
    } else {
        cls::LAST
    })
}
