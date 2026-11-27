use crate::classes::radio;
use crate::colors::{TEXT_DEFAULT, TEXT_SECONDARY, TEXT_STRONG};
use crate::font_system;
use crate::icon_handle;
use skia_safe::font_style::Weight;
use std::rc::Rc;

use crate::font_system::TextFonts;

use crate::node::Node;
use crate::widget::anim::ToggleState;
use crate::widget::primitives::{ButtonWidget, RadioWidget};

use super::container::div;
use super::text::{TextProps, text};

pub struct RadioOption {
    pub value: String,
    pub label: String,

    /// Selected when `on`. The group does not enforce a single selection.
    pub state: ToggleState,

    pub description: Option<String>,

    pub icon: Option<icon_handle::IconSpec>,

    pub disabled: bool,

    /// `**bold**` in `label` renders as bold.
    pub markup: bool,
}

impl RadioOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>, state: ToggleState) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            state,
            description: None,
            icon: None,
            disabled: false,
            markup: false,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_icon(mut self, path: impl Into<icon_handle::IconSpec>) -> Self {
        self.icon = Some(path.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn markup(mut self) -> Self {
        self.markup = true;
        self
    }
}

/// Selected option `value`.
pub struct RadioOnChange(pub Rc<dyn Fn(String)>);

impl<F: Fn(String) + 'static> From<F> for RadioOnChange {
    fn from(f: F) -> Self {
        RadioOnChange(Rc::new(f))
    }
}

pub struct RadioGroupProps {
    pub options: Vec<RadioOption>,

    pub on_change: RadioOnChange,

    pub label: Option<String>,

    pub disabled: bool,

    pub fonts: Option<TextFonts>,
}

crate::builder_fields!(RadioGroupProps {
    options: Vec<RadioOption>,
    on_change: RadioOnChange,
    opt label: String,
    disabled: bool,
    opt fonts: TextFonts,
});

impl RadioGroupProps {
    pub fn new(on_change: impl Into<RadioOnChange>) -> Self {
        Self {
            options: Vec::new(),
            on_change: on_change.into(),
            label: None,
            disabled: false,
            fonts: None,
        }
    }

    pub fn build(self) -> Node {
        radio_group(self)
    }
}

pub fn radio_group(props: RadioGroupProps) -> Node {
    let fonts = font_system::resolve(props.fonts);
    let mut group = div().class(radio::GROUP);

    for option in props.options {
        let disabled = props.disabled || option.disabled;
        let value = option.value.clone();
        let on_change = props.on_change.0.clone();
        let btn = ButtonWidget::new().with_on_click(move || on_change(value.clone()));

        let ring =
            Node::new("div", RadioWidget::new(option.state)).class(radio::RING);
        let has_icon = option.icon.is_some();

        let mut row = Node::new("label", btn)
            .class(radio::ROW)
            .child(ring);
        if has_icon {
            row = row.class(radio::ROW_ICON);
        }
        if let Some(path) = option.icon {
            row = row.child(
                super::icon::icon(&path, TEXT_STRONG).class(radio::ITEM_ICON),
            );
        }
        let mut label = TextProps {
            content: option.label,
            size: 16.0,
            color: TEXT_DEFAULT,
            fonts: Some(fonts.clone()),
            ..Default::default()
        };
        if option.markup {
            label = label.bold_markup();
        }
        row = row.child(text(label).class(radio::LABEL));
        if let Some(description) = option.description {
            let mut description = text(TextProps {
                content: description,
                size: 14.0,
                color: TEXT_SECONDARY,
                fonts: Some(fonts.clone()),
                ..Default::default()
            })
            .class(radio::DESCRIPTION);
            if has_icon {
                description = description.class(radio::DESCRIPTION_ICON);
            }
            row = row.child(description);
        }
        if disabled {
            row = row.class(radio::DISABLED);
        }
        group = group.child(row);
    }

    match props.label {
        Some(label) => div()
            .class(radio::ROOT)
            .child(text(TextProps {
                content: label,
                size: 16.0,
                color: TEXT_STRONG,
                weight: Weight::MEDIUM.into(),
                fonts: Some(fonts),
                ..Default::default()
            }))
            .child(group),
        None => group,
    }
}
