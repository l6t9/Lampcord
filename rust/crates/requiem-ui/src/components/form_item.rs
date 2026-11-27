use crate::classes::form_item as classes_form_item;
use crate::classes::form_item as cls;
use crate::colors::{TEXT_CRITICAL, TEXT_STRONG};
use crate::font_system::TextFonts;
use crate::font_system;
use crate::icon_handle;
use crate::icons;
use skia_safe::font_style::Weight;

use crate::node::Node;

use super::container::div;
use super::icon::icon;
use super::text::{TextProps, text};

pub fn field_error(message: String) -> Node {
    let fonts = font_system::resolve(None);
    div()
        .class(classes_form_item::ERROR_ROW)
        .child(
            icon(icons::CIRCLE_ERROR, TEXT_CRITICAL)
                .class(classes_form_item::ERROR_ICON),
        )
        .child(text(TextProps {
            content: message,
            size: 12.0,
            color: TEXT_CRITICAL,
            fonts: Some(fonts),
            ..Default::default()
        }))
}

#[derive(Default)]
pub struct FormItemProps {
    pub title: Option<String>,
    pub description: Option<String>,

    /// Replaces `description` when set.
    pub description_node: Option<Node>,
    /// Draws the required mark. Does not validate.
    pub required: bool,

    /// Replaces `helper_text` and marks the field invalid.
    pub error: Option<String>,

    pub helper_text: Option<String>,

    pub icon: Option<icon_handle::IconSpec>,

    pub badge: Option<Node>,

    /// Label and control on one row.
    pub horizontal: bool,

    /// Dims the field. The child control still needs its own disabled state.
    pub disabled: bool,

    /// `title` is not painted.
    pub hide_label: bool,

    /// `**bold**` in `title` renders as bold.
    pub markup: bool,
    pub children: Node,

    pub fonts: Option<TextFonts>,
}

crate::builder_fields!(FormItemProps {
    opt title: String,
    opt description: String,
    opt description_node: Node,
    required: bool,
    opt error: String,
    opt helper_text: String,
    opt icon: icon_handle::IconSpec,
    opt badge: Node,
    horizontal: bool,
    disabled: bool,
    hide_label: bool,
    markup: bool,
    children: Node,
    opt fonts: TextFonts,
});

impl FormItemProps {
    pub fn new(title: impl Into<String>, children: Node) -> Self {
        Self {
            title: Some(title.into()),
            children,
            ..Default::default()
        }
    }

    pub fn build(self) -> Node {
        form_item(self)
    }
}

pub fn form_item(props: FormItemProps) -> Node {
    let FormItemProps {
        title,
        description,
        description_node,
        required,
        error,
        helper_text,
        icon: title_icon,
        badge,
        horizontal,
        disabled,
        hide_label,
        markup,
        children,
        fonts,
    } = props;
    let mut field = div().class(if horizontal {
        cls::FIELD_HORIZONTAL
    } else {
        cls::FIELD
    });

    let mut label_group = div().class(cls::LABEL_GROUP);
    let mut has_label = false;
    if let Some(title) = title.filter(|_| !hide_label) {
        let fonts = font_system::resolve(fonts.clone());
        let mut title_row = div().class(cls::TITLE_ROW);
        if let Some(path) = title_icon {
            title_row = title_row.child(icon(&path, TEXT_STRONG).class(cls::TITLE_ICON));
        }
        let mut title = TextProps {
            content: title,
            size: 16.0,
            weight: Weight::MEDIUM.into(),
            fonts: Some(fonts.clone()),
            ..Default::default()
        };
        if markup {
            title = title.bold_markup();
        }
        title_row = title_row.child(text(title).class(cls::TITLE));
        if required {
            title_row = title_row.child(
                text(TextProps {
                    content: "*".to_string(),
                    size: 16.0,
                    fonts: Some(fonts),
                    ..Default::default()
                })
                .class(cls::REQUIRED),
            );
        }
        if let Some(badge) = badge {
            title_row = title_row.child(badge);
        }
        label_group = label_group.child(title_row);
        has_label = true;
    }
    let description = description_node.or_else(|| {
        let fonts = font_system::resolve(fonts.clone());
        description.map(|content| {
            text(TextProps {
                content,
                size: 14.0,
                fonts: Some(fonts),
                ..Default::default()
            })
            .class(cls::DESCRIPTION)
        })
    });
    if let Some(mut node) = description {
        if horizontal {
            node = node.class(cls::DESCRIPTION_CAPPED);
        }
        label_group = label_group.child(node);
        has_label = true;
    }
    if disabled {
        label_group = label_group.class(cls::DIMMED);
    }
    if has_label {
        field = field.child(label_group);
    }

    let mut control = div()
        .class(if horizontal {
            cls::CONTROL_END
        } else {
            cls::CONTROL
        })
        .child(children);
    if let Some(error) = error.filter(|e| !e.is_empty()) {
        control = control.child(field_error(error));
    } else if let Some(helper) = helper_text {
        let fonts = font_system::resolve(fonts);
        let mut node = text(TextProps {
            content: helper,
            size: 12.0,
            fonts: Some(fonts),
            ..Default::default()
        })
        .class(cls::HELPER);
        if disabled {
            node = node.class(cls::DIMMED);
        }
        control = control.child(node);
    }
    field.child(control)
}
