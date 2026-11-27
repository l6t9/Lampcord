use crate::classes::help_message as cls;
use crate::colors::WARNING;
use crate::font_system;
use crate::icons;
use skia_safe::font_style::Weight;

use super::container::div;
use super::icon::icon;
use super::text::{TextProps, text};
use crate::font_system::TextFonts;
use crate::node::Node;

#[derive(Default)]
pub struct HelpMessageProps {
    pub text: String,

    /// Replaces `text` when set.
    pub content: Option<Node>,

    pub fonts: Option<TextFonts>,
}


crate::builder_fields!(HelpMessageProps {
    text: String,
    opt content: Node,
    opt fonts: TextFonts,
});

impl HelpMessageProps {
    pub fn build(self) -> Node {
        help_message_warning(self)
    }
}

pub fn help_message_warning(props: HelpMessageProps) -> Node {
    let fonts = font_system::resolve(props.fonts);
    let label = match props.content {
        Some(content) => content.class(cls::TEXT),
        None => text(TextProps {
            content: props.text,
            size: 14.0,
            weight: Weight::MEDIUM.into(),
            color: WARNING,
            fonts: Some(fonts),
            ..Default::default()
        })
        .class(cls::TEXT),
    };
    div()
        .class(cls::ROOT)
        .children([icon(icons::WARNING, WARNING).class(cls::ICON), label])
}
