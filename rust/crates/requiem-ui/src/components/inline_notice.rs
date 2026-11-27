use crate::classes::inline_notice as classes_inline_notice;
use crate::node::Node;

use super::container::div;
use super::text::TextProps;

pub struct InlineNoticeProps {
    pub text: String,
}

impl InlineNoticeProps {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn build(self) -> Node {
        inline_notice(self)
    }
}

pub fn inline_notice(props: InlineNoticeProps) -> Node {
    div()
        .class(classes_inline_notice::ROOT)
        .child(TextProps::new(props.text, 14.0).build())
}
