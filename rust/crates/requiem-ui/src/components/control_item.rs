use crate::classes::control_item as cls;
use crate::font_system;
use std::rc::Rc;

use skia_safe::font_style::Weight;

use crate::colors::TEXT_STRONG;
use crate::node::Node;
use crate::widget::primitives::{ButtonWidget, ChevronWidget};

use super::container::div;
use super::text::{TextProps, text};
use crate::icons::CHEVRON_LARGE_DOWN;

#[derive(Default)]
pub struct ControlItemProps {
    pub title: String,

    pub subtitle: Option<String>,

    /// Replaces `subtitle` when set.
    pub subtitle_node: Option<Node>,
    pub leading: Option<Node>,
    pub trailing: Option<Node>,

    /// `None` leaves the row inert.
    pub on_click: Option<Rc<dyn Fn()>>,
}

crate::builder_fields!(ControlItemProps {
    title: String,
    opt subtitle: String,
    opt subtitle_node: Node,
    opt leading: Node,
    opt trailing: Node,
});

impl ControlItemProps {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    pub fn on_click(mut self, on_click: impl Fn() + 'static) -> Self {
        self.on_click = Some(Rc::new(on_click));
        self
    }

    pub fn build(self) -> Node {
        control_item(self)
    }
}

pub fn control_item(props: ControlItemProps) -> Node {
    let fonts = font_system::resolve(None);
    let mut content = div()
        .class(cls::CONTENT)
        .child(div().class(cls::TITLE).child(text(TextProps {
            content: props.title,
            size: 16.0,
            line_height: Some(20.0),
            weight: Weight::MEDIUM.into(),
            fonts: Some(fonts.clone()),
            ..Default::default()
        })));
    if let Some(subtitle) = props.subtitle_node {
        content = content.child(subtitle);
    } else if let Some(subtitle) = props.subtitle {
        content = content.child(
            text(TextProps {
                content: subtitle,
                size: 14.0,
                line_height: Some(18.0),
                fonts: Some(fonts),
                ..Default::default()
            })
            .class(cls::SUBTITLE),
        );
    }
    let mut children = Vec::with_capacity(3);
    if let Some(leading) = props.leading {
        children.push(div().class(cls::LEADING).child(leading));
    }
    children.push(content);
    if let Some(trailing) = props.trailing {
        children.push(div().class(cls::TRAILING).child(trailing));
    }
    match props.on_click {
        Some(on_click) => Node::new("div", ButtonWidget::new().with_on_click(move || on_click()))
            .classes([cls::ROOT, cls::CLICKABLE])
            .children(children),
        None => div().class(cls::ROOT).children(children),
    }
}

pub struct AccordionProps {
    /// Label while collapsed.
    pub title_closed: String,
    /// Label while expanded.
    pub title_open: String,
    pub subtitle_closed: Option<String>,
    pub open: bool,
    pub on_toggle: Rc<dyn Fn()>,
    pub content: Node,
}

impl AccordionProps {
    pub fn build(self) -> Node {
        accordion(self)
    }
}

pub fn accordion(props: AccordionProps) -> Node {
    let toggle = props.on_toggle;
    let chevron = Node::new(
        "span",
        ChevronWidget::new(CHEVRON_LARGE_DOWN, TEXT_STRONG, props.open),
    )
    .class(cls::TRIGGER_ICON);
    let trigger = div().class(cls::TRIGGER).child(chevron);
    let mut item = ControlItemProps::new(if props.open {
        props.title_open
    } else {
        props.title_closed
    })
    .trailing(trigger)
    .on_click(move || toggle());
    if !props.open {
        item.subtitle = props.subtitle_closed;
    }
    let mut root = div().class(cls::ACCORDION).child(item.build());
    if props.open {
        root = root.child(div().class(cls::PANEL).child(props.content));
    }
    root
}
