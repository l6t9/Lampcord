use crate::classes::switch as classes_switch;
use crate::colors::{TEXT_CRITICAL, TEXT_SECONDARY, TEXT_STRONG};
use crate::font_system;
use skia_safe::font_style::Weight;
use std::rc::Rc;

use crate::font_system::TextFonts;

use crate::node::Node;
use crate::widget::anim::ToggleState;
use crate::widget::primitives::{ButtonWidget, SwitchWidget};

use super::container::div;
use super::text::{TextProps, text};

/// New on state.
pub struct SwitchOnChange(pub Rc<dyn Fn(bool)>);

impl<F: Fn(bool) + 'static> From<F> for SwitchOnChange {
    fn from(f: F) -> Self {
        SwitchOnChange(Rc::new(f))
    }
}

pub struct SwitchProps {
    pub state: ToggleState,

    pub label: Option<String>,
    pub description: Option<String>,

    /// Replaces `description` when set.
    pub description_node: Option<Node>,

    /// Draws the required mark. Does not validate.
    pub required: bool,

    pub error: Option<String>,

    /// Check and cross inside the thumb.
    pub icon: bool,

    pub disabled: bool,
    pub on_change: SwitchOnChange,

    pub fonts: Option<TextFonts>,
}

crate::builder_fields!(SwitchProps {
    state: ToggleState,
    opt label: String,
    opt description: String,
    opt description_node: Node,
    required: bool,
    opt error: String,
    icon: bool,
    disabled: bool,
    on_change: SwitchOnChange,
    opt fonts: TextFonts,
});

impl SwitchProps {
    pub fn new(state: ToggleState, on_change: impl Into<SwitchOnChange>) -> Self {
        Self {
            state,
            label: None,
            description: None,
            description_node: None,
            required: false,
            error: None,
            icon: false,
            disabled: false,
            on_change: on_change.into(),
            fonts: None,
        }
    }

    pub fn build(self) -> Node {
        switch(self)
    }
}

pub fn switch(props: SwitchProps) -> Node {
    let on = props.state.on;
    let on_change = props.on_change.0;
    let btn = ButtonWidget::new().with_on_click(move || on_change(!on));

    let mut widget = SwitchWidget::new(props.state);
    if props.icon {
        widget = widget.with_icons();
    }
    let mut track = Node::new("div", widget).class(classes_switch::TRACK);
    if props.disabled {
        track = track.class(classes_switch::DISABLED);
    }
    let toggle = Node::new("label", btn)
        .class(classes_switch::TOGGLE)
        .child(track);

    if props.label.is_none()
        && props.description.is_none()
        && props.description_node.is_none()
        && props.error.is_none()
    {
        return toggle;
    }
    let fonts = font_system::resolve(props.fonts);

    let mut labels = div().class(classes_switch::LABELS);
    if let Some(label) = props.label {
        let mut row = div()
            .class(classes_switch::LABEL_ROW)
            .child(text(TextProps {
                content: label,
                size: 16.0,
                color: TEXT_STRONG,
                weight: Weight::MEDIUM.into(),
                fonts: Some(fonts.clone()),
                ..Default::default()
            }));
        if props.required {
            row = row.child(text(TextProps {
                content: "*".to_string(),
                size: 16.0,
                color: TEXT_CRITICAL,
                fonts: Some(fonts.clone()),
                ..Default::default()
            }));
        }
        labels = labels.child(row);
    }
    if let Some(description) = props.description_node {
        labels = labels.child(description);
    } else if let Some(description) = props.description {
        labels = labels.child(text(TextProps {
            content: description,
            size: 14.0,
            color: TEXT_SECONDARY,
            fonts: Some(fonts.clone()),
            ..Default::default()
        }));
    }
    if let Some(error) = props.error.filter(|e| !e.is_empty()) {
        labels = labels.child(super::form_item::field_error(error));
    }
    div()
        .class(classes_switch::FIELD)
        .child(labels)
        .child(toggle)
}
