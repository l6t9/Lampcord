use crate::classes::menu as cls;
use crate::font_system;
use std::rc::Rc;

use skia_safe::font_style::Weight;

use super::{TextProps, text};
use crate::node::Node;
use crate::widget::anim::ToggleState;
use crate::widget::primitives::{ButtonWidget, PopoverPanelWidget, RadioWidget};

pub enum MenuItem {
    Action {
        label: String,
        danger: bool,
        on_click: Rc<dyn Fn()>,

        leading: Option<Node>,

        subtitle: Option<String>,
    },

    Radio {
        label: String,
        selected: bool,
        on_click: Rc<dyn Fn()>,
    },
    Separator,
}

impl MenuItem {
    pub fn radio(label: impl Into<String>, selected: bool, on_click: impl Fn() + 'static) -> Self {
        MenuItem::Radio {
            label: label.into(),
            selected,
            on_click: Rc::new(on_click),
        }
    }

    pub fn action(label: impl Into<String>, on_click: impl Fn() + 'static) -> Self {
        MenuItem::Action {
            label: label.into(),
            danger: false,
            on_click: Rc::new(on_click),
            leading: None,
            subtitle: None,
        }
    }

    pub fn danger(label: impl Into<String>, on_click: impl Fn() + 'static) -> Self {
        MenuItem::Action {
            label: label.into(),
            danger: true,
            on_click: Rc::new(on_click),
            leading: None,
            subtitle: None,
        }
    }

    pub fn with_leading(mut self, node: Node) -> Self {
        if let MenuItem::Action { leading, .. } = &mut self {
            *leading = Some(node);
        }
        self
    }

    pub fn with_subtitle(mut self, text: impl Into<String>) -> Self {
        if let MenuItem::Action { subtitle, .. } = &mut self {
            *subtitle = Some(text.into());
        }
        self
    }
}

pub fn menu(items: Vec<MenuItem>) -> Node {
    let fonts = &font_system::resolve(None);
    let mut panel = Node::new("div", PopoverPanelWidget::new()).class(cls::PANEL);
    for item in items {
        panel = panel.child(match item {
            MenuItem::Separator => super::div().class(cls::SEPARATOR),
            MenuItem::Radio {
                label,
                selected,
                on_click,
            } => {
                let label = text(TextProps {
                    content: label,
                    size: 14.0,
                    weight: Weight::MEDIUM.into(),
                    color: skia_safe::Color::from_rgb(0xab, 0xac, 0xb2),
                    fonts: Some(fonts.clone()),
                    ..Default::default()
                })
                .class(cls::LABEL)
                .class(cls::LABEL_FILL);
                let indicator = Node::new("div", RadioWidget::new(ToggleState::new(selected)))
                    .class(cls::RADIO_INDICATOR);
                Node::new(
                    "button",
                    ButtonWidget::new().with_on_click(move || on_click()),
                )
                .class(cls::ITEM)
                .child(label)
                .child(indicator)
            }
            MenuItem::Action {
                label,
                danger,
                on_click,
                leading,
                subtitle,
            } => {
                let label = text(TextProps {
                    content: label,
                    size: 14.0,
                    weight: Weight::MEDIUM.into(),
                    color: skia_safe::Color::from_rgb(0xab, 0xac, 0xb2),
                    fonts: Some(fonts.clone()),
                    ..Default::default()
                })
                .class(cls::LABEL);
                let label = match subtitle {
                    Some(subtitle) => {
                        let subtitle = text(TextProps {
                            content: subtitle,
                            size: 12.0,
                            color: skia_safe::Color::from_rgb(0x96, 0x97, 0x9e),
                            fonts: Some(fonts.clone()),
                            ..Default::default()
                        })
                        .class(cls::SUBTITLE);
                        super::div()
                            .class(cls::TEXT_COLUMN)
                            .child(label)
                            .child(subtitle)
                    }
                    None => label,
                };
                let mut row = Node::new(
                    "button",
                    ButtonWidget::new().with_on_click(move || on_click()),
                )
                .class(cls::ITEM);
                if let Some(leading) = leading {
                    row = row.child(super::div().class(cls::LEADING).child(leading));
                }
                let mut row = row.child(label);
                if danger {
                    row = row.class(cls::ITEM_DANGER);
                }
                row
            }
        });
    }
    panel
}
