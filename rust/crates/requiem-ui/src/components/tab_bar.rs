use crate::classes::tab_bar as cls;
use crate::font_system;
use super::ButtonVariant;
use std::rc::Rc;

use skia_safe::Color;
use skia_safe::font_style::Weight;

use super::ButtonProps;
use super::container::div;
use super::text::{TextProps, text};
use crate::font_system::TextFonts;
use crate::node::Node;

#[derive(Clone)]
pub struct TabItem {
    pub id: String,
    pub label: String,
}

impl TabItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

pub struct TabBarProps {
    pub items: Vec<TabItem>,
    pub selected: String,
    pub on_select: Rc<dyn Fn(String)>,

    pub fonts: Option<TextFonts>,

    /// Extra padding on each tab.
    pub roomy: bool,

    /// Tabs share the bar width instead of sizing to their labels.
    pub fill: bool,
}

impl Default for TabBarProps {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected: String::new(),
            on_select: Rc::new(|_| {}),
            fonts: None,
            roomy: false,
            fill: false,
        }
    }
}

crate::builder_fields!(TabBarProps {
    items: Vec<TabItem>,
    selected: String,
    on_select: Rc<dyn Fn(String)>,
    opt fonts: TextFonts,
    roomy: bool,
    fill: bool,
});

impl TabBarProps {
    pub fn build(self) -> Node {
        tab_bar(self)
    }
}

pub fn tab_bar(props: TabBarProps) -> Node {
    let fonts = font_system::resolve(props.fonts);
    let mut bar = div().class(cls::BAR);
    if props.roomy {
        bar = bar.class(cls::ROOMY_BAR);
    }
    if props.fill {
        bar = bar.class(cls::FILL_BAR);
    }
    for item in props.items {
        let selected = item.id == props.selected;
        let label = text(TextProps {
            content: item.label,
            size: 14.0,
            line_height: Some(14.0),
            weight: Weight::SEMI_BOLD.into(),
            color: Color::from_rgb(0xab, 0xac, 0xb2),
            fonts: Some(fonts.clone()),
            line_clamp: Some(1),
            ..Default::default()
        })
        .class(cls::LABEL);
        let mut classes = vec![cls::ITEM.to_string()];
        if props.roomy {
            classes.push(cls::ROOMY_ITEM.to_string());
        }
        if props.fill {
            classes.push(cls::FILL_ITEM.to_string());
        }
        let mut button = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(classes);
        if !selected {
            let (on_select, id) = (props.on_select.clone(), item.id);
            button = button.on_click(move || on_select(id.clone()));
        }
        let mut node = button
            .build()
            .child(label)
            .child(div().class(cls::INDICATOR));
        if selected {
            node = node.class(cls::SELECTED);
        }
        bar = bar.child(node);
    }
    bar
}
