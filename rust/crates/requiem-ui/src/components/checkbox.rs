use crate::classes::checkbox as classes_checkbox;
use crate::colors::{TEXT_SECONDARY, TEXT_STRONG};
use crate::font_system;
use crate::icon_handle;
use std::rc::Rc;

use crate::font_system::TextFonts;
use skia_safe::Color;

use crate::node::Node;
use crate::widget::anim::ToggleState;
use crate::widget::primitives::{ButtonWidget, CheckboxWidget};

use super::container::div;
use super::text::{TextProps, text};

/// New checked state. Wrapper so a builder setter accepts a closure.
pub struct CheckboxOnChange(pub Rc<dyn Fn(bool)>);

impl Default for CheckboxOnChange {
    fn default() -> Self {
        CheckboxOnChange(Rc::new(|_| {}))
    }
}

impl<F: Fn(bool) + 'static> From<F> for CheckboxOnChange {
    fn from(f: F) -> Self {
        CheckboxOnChange(Rc::new(f))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CheckboxLabelType {
    #[default]
    Primary,
    Secondary,
}

pub struct CheckboxProps {
    pub state: ToggleState,
    pub label: Option<String>,
    pub label_type: CheckboxLabelType,

    pub description: Option<String>,

    pub icon: Option<icon_handle::IconSpec>,

    pub disabled: bool,
    pub on_change: CheckboxOnChange,

    /// Replaces the label row when set.
    pub content: Option<Node>,

    pub fonts: Option<TextFonts>,
}

crate::builder_fields!(CheckboxProps {
    state: ToggleState,
    opt label: String,
    label_type: CheckboxLabelType,
    opt description: String,
    opt icon: icon_handle::IconSpec,
    disabled: bool,
    on_change: CheckboxOnChange,
    opt content: Node,
    opt fonts: TextFonts,
});

impl CheckboxProps {
    pub fn new(state: ToggleState, on_change: impl Into<CheckboxOnChange>) -> Self {
        Self {
            state,
            label: None,
            label_type: CheckboxLabelType::Primary,
            description: None,
            icon: None,
            disabled: false,
            on_change: on_change.into(),
            content: None,
            fonts: None,
        }
    }

    pub fn build(self) -> Node {
        checkbox(self)
    }
}

pub(crate) fn option_label(
    label: String,
    color: Color,
    description: Option<String>,
    icon: Option<icon_handle::IconSpec>,
    fonts: &TextFonts,
) -> Node {
    let mut row = div().class(classes_checkbox::LABEL_ROW);
    if let Some(path) = icon {
        row =
            row.child(super::icon::icon(&path, TEXT_STRONG).class(classes_checkbox::ICON));
    }
    row = row.child(text(TextProps {
        content: label,
        size: 16.0,
        color,
        fonts: Some(fonts.clone()),
        ..Default::default()
    }));
    let mut stack = div()
        .class(classes_checkbox::LABEL_STACK)
        .child(row);
    if let Some(description) = description {
        stack = stack.child(text(TextProps {
            content: description,
            size: 14.0,
            color: TEXT_SECONDARY,
            fonts: Some(fonts.clone()),
            ..Default::default()
        }));
    }
    stack
}

pub fn checkbox_indicator(state: ToggleState) -> Node {
    Node::new("div", CheckboxWidget::new(state)).class(classes_checkbox::BOX)
}

pub fn checkbox(props: CheckboxProps) -> Node {
    let on = props.state.on;
    let on_change = props.on_change.0;
    let btn = ButtonWidget::new().with_on_click(move || on_change(!on));

    let box_node = checkbox_indicator(props.state);

    let mut row = Node::new("label", btn)
        .class(classes_checkbox::ROW)
        .child(box_node);
    if props.disabled {
        row = row.class(classes_checkbox::DISABLED);
    }
    if let Some(label) = props.label {
        let fonts = font_system::resolve(props.fonts);
        let color = match props.label_type {
            CheckboxLabelType::Primary => TEXT_STRONG,
            CheckboxLabelType::Secondary => TEXT_SECONDARY,
        };
        row = row.child(option_label(
            label,
            color,
            props.description,
            props.icon,
            &fonts,
        ));
    }
    if let Some(content) = props.content {
        row = row.child(content.class(classes_checkbox::CONTENT));
    }
    row
}
