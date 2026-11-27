use crate::classes::disclosure as classes_disclosure;
use crate::colors;
use crate::font_system;
use skia_safe::font_style::Weight;

use crate::font_system::TextFonts;
use crate::node::Node;
use crate::widget::primitives::{ButtonWidget, ChevronWidget};

use super::container::div;
use super::select::OnToggle;
use super::text::{TextProps, text};
use crate::icons::CHEVRON_LARGE_DOWN;

pub struct DisclosureProps {
    pub title: String,
    pub description: Option<String>,
    pub open: bool,
    pub on_toggle: OnToggle,
    pub content: Node,

    pub fonts: Option<TextFonts>,
}

crate::builder_fields!(DisclosureProps {
    title: String,
    opt description: String,
    open: bool,
    on_toggle: OnToggle,
    content: Node,
    opt fonts: TextFonts,
});

impl DisclosureProps {
    pub fn new(on_toggle: impl Into<OnToggle>) -> Self {
        Self {
            title: String::new(),
            description: None,
            open: false,
            on_toggle: on_toggle.into(),
            content: Node::default(),
            fonts: None,
        }
    }

    pub fn build(self) -> Node {
        disclosure(self)
    }
}

pub fn disclosure(props: DisclosureProps) -> Node {
    let fonts = font_system::resolve(props.fonts);
    let toggle = props.on_toggle.0.clone();

    let mut labels = div().class(classes_disclosure::LABELS).child(
        text(TextProps {
            content: props.title,
            size: 16.0,
            weight: Weight::MEDIUM.into(),
            fonts: Some(fonts.clone()),
            ..Default::default()
        })
        .class(classes_disclosure::TITLE),
    );
    if let Some(description) = props.description {
        labels = labels.child(
            text(TextProps {
                content: description,
                size: 14.0,
                fonts: Some(fonts),
                ..Default::default()
            })
            .class(classes_disclosure::DESCRIPTION),
        );
    }

    let header = Node::new(
        "button",
        ButtonWidget::new().with_on_click(move || toggle()),
    )
    .class(classes_disclosure::HEADER)
    .child(labels)
    .child(
        Node::new(
            "span",
            ChevronWidget::new(CHEVRON_LARGE_DOWN, colors::TEXT_SUBTLE, props.open),
        )
        .class(classes_disclosure::CHEVRON),
    );

    let mut root = div().class(classes_disclosure::ROOT).child(header);
    if props.open {
        root = root.child(
            div()
                .class(classes_disclosure::CONTENT)
                .child(props.content),
        );
    }
    root
}
