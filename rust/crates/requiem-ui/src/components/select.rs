use crate::classes::select as classes_select;
use crate::classes::select as cls;
use crate::colors::{INPUT_PLACEHOLDER, TEXT_DEFAULT, TEXT_MUTED, TEXT_SECONDARY, TEXT_STRONG};
use crate::components::TextInputProps;
use crate::components;
use crate::event::NamedKey;
use crate::font_system;
use crate::icon_handle;
use crate::icons;
use crate::widget::anim::ToggleState;
use crate::widget::primitives::CheckboxWidget;
use skia_safe::font_style::Weight;
use std::mem;
use std::rc::Rc;

use crate::font_system::TextFonts;

use crate::node::Node;
use crate::widget::primitives::{ButtonWidget, ChevronWidget, PopoverPanelWidget};

use super::container::div;
use super::text::{TextProps, text};

pub struct SelectOption {
    pub value: String,
    pub label: String,

    pub description: Option<String>,

    /// Heading drawn above the first option that carries this name.
    pub group: Option<String>,

    pub leading: Option<Node>,

    pub trailing: Option<Node>,

    pub disabled: bool,

    /// Trigger text when this option is selected. `None` uses `label`.
    pub selected_label: Option<String>,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: None,
            group: None,
            leading: None,
            trailing: None,
            disabled: false,
            selected_label: None,
        }
    }

    pub fn with_selected_label(mut self, label: impl Into<String>) -> Self {
        self.selected_label = Some(label.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    pub fn with_leading(mut self, node: Node) -> Self {
        self.leading = Some(node);
        self
    }

    pub fn with_trailing(mut self, node: Node) -> Self {
        self.trailing = Some(node);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

pub fn select_icon(path: impl icon_handle::IconPath) -> Node {
    super::icon::icon(path, TEXT_SECONDARY).class(classes_select::LEADING_ICON)
}

const MAX_UNSCROLLED_OPTIONS: usize = 5;

/// Open or close the panel. The caller owns `open`.
pub struct OnToggle(pub Rc<dyn Fn()>);

impl<F: Fn() + 'static> From<F> for OnToggle {
    fn from(f: F) -> Self {
        OnToggle(Rc::new(f))
    }
}

/// Chosen option `value`.
pub struct OnSelect(pub Rc<dyn Fn(String)>);

impl<F: Fn(String) + 'static> From<F> for OnSelect {
    fn from(f: F) -> Self {
        OnSelect(Rc::new(f))
    }
}

pub struct SelectSearch {
    /// `None` paints the field empty. The caller owns the string.
    pub query: Option<String>,

    /// Highlighted row index in the filtered list.
    pub active: usize,
    pub on_query: Rc<dyn Fn(String)>,
    pub on_active: Rc<dyn Fn(usize)>,
}

pub struct SelectProps {
    pub options: Vec<SelectOption>,

    pub selected: Option<String>,

    /// `on_select` toggles membership in `selected_values` instead of replacing `selected`.
    pub multiple: bool,
    pub selected_values: Vec<String>,
    pub placeholder: String,
    pub open: bool,

    pub on_toggle: OnToggle,

    pub on_select: OnSelect,

    /// `Some` paints a clear control. The callback should drop the selection.
    pub on_clear: Option<OnToggle>,

    /// Open the list upward. Default opens downward.
    pub panel_above: bool,

    pub footer: Vec<Node>,

    /// Error border. Does not block selection.
    pub invalid: bool,

    /// Shows the value and ignores open and select.
    pub read_only: bool,

    pub disabled: bool,

    /// No box around the trigger. The label uses medium weight.
    pub text_only: bool,

    /// Shorter trigger, for a row that is already dense.
    pub compact: bool,

    pub trigger_leading: Option<Node>,
    pub trigger_trailing: Option<Node>,

    pub search: Option<SelectSearch>,

    pub fonts: Option<TextFonts>,
}

crate::builder_fields!(SelectProps {
    opt search: SelectSearch,
    opt trigger_leading: Node,
    opt trigger_trailing: Node,
    options: Vec<SelectOption>,
    opt selected: String,
    multiple: bool,
    selected_values: Vec<String>,
    placeholder: String,
    open: bool,
    on_toggle: OnToggle,
    on_select: OnSelect,
    opt on_clear: OnToggle,
    panel_above: bool,
    footer: Vec<Node>,
    invalid: bool,
    read_only: bool,
    disabled: bool,
    text_only: bool,
    compact: bool,
    opt fonts: TextFonts,
});

impl SelectProps {
    pub fn new(on_toggle: impl Into<OnToggle>, on_select: impl Into<OnSelect>) -> Self {
        Self {
            options: Vec::new(),
            trigger_leading: None,
            trigger_trailing: None,
            selected: None,
            multiple: false,
            selected_values: Vec::new(),
            placeholder: String::new(),
            open: false,
            on_toggle: on_toggle.into(),
            on_select: on_select.into(),
            on_clear: None,
            panel_above: false,
            footer: Vec::new(),
            invalid: false,
            read_only: false,
            disabled: false,
            text_only: false,
            compact: false,
            search: None,
            fonts: None,
        }
    }

    pub fn build(self) -> Node {
        select(self)
    }
}

fn filter_options(options: Vec<SelectOption>, query: &str) -> Vec<SelectOption> {
    if query.is_empty() {
        return options;
    }
    let query = query.to_lowercase();
    let (mut prefix, mut inside) = (Vec::new(), Vec::new());
    for option in options {
        let label = option.label.to_lowercase();
        if label.starts_with(&query) {
            prefix.push(option);
        } else if label.contains(&query) {
            inside.push(option);
        }
    }
    prefix.extend(inside);
    prefix
}

fn step_active(rows: &[(String, bool)], from: usize, step: isize) -> usize {
    let len = rows.len() as isize;
    let mut at = from as isize;
    for _ in 0..rows.len() {
        at = (at + step).rem_euclid(len);
        if !rows[at as usize].1 {
            return at as usize;
        }
    }
    from
}

fn tag(label: String, on_remove: impl FnMut() + 'static, fonts: &TextFonts) -> Node {
    div()
        .class(classes_select::TAG)
        .child(text(TextProps {
            content: label,
            size: 14.0,
            color: TEXT_STRONG,
            fonts: Some(fonts.clone()),
            ..Default::default()
        }))
        .child(
            Node::new("button", ButtonWidget::new().with_on_click(on_remove))
                .class(classes_select::TAG_REMOVE)
                .child(
                    super::icon::icon(icons::X_SMALL, TEXT_SECONDARY)
                        .class(classes_select::TAG_REMOVE_ICON),
                ),
        )
}

pub fn select(mut props: SelectProps) -> Node {
    let fonts = font_system::resolve(props.fonts);
    let inert = props.read_only || props.disabled;
    let open = props.open && !inert;

    let search = props.search.take().filter(|_| !inert);
    let searchable_label = props
        .selected
        .as_deref()
        .and_then(|value| props.options.iter().find(|o| o.value == value))
        .map(|o| o.selected_label.clone().unwrap_or_else(|| o.label.clone()));
    if let Some(search) = &search {
        props.options = filter_options(
            mem::take(&mut props.options),
            search.query.as_deref().unwrap_or(""),
        );
    }
    let rows_for_keys: Vec<(String, bool)> = props
        .options
        .iter()
        .map(|o| (o.value.clone(), o.disabled))
        .collect();

    let toggle = props.on_toggle.0.clone();
    let mut trigger_btn = ButtonWidget::new();
    if !inert {
        trigger_btn = trigger_btn.with_on_click(move || toggle());
    }
    let chevron = ChevronWidget::new(icons::CHEVRON_LARGE_DOWN, TEXT_STRONG, open);
    let mut trigger_box = div().class(cls::TRIGGER_BOX);
    if props.text_only {
        trigger_box = trigger_box.class(cls::TRIGGER_BOX_TEXT_ONLY);
    }
    if props.invalid {
        trigger_box = trigger_box.class(cls::TRIGGER_BOX_INVALID);
    } else if props.read_only {
        trigger_box = trigger_box.class(cls::TRIGGER_BOX_READ_ONLY);
    } else if open {
        trigger_box = trigger_box.class(cls::TRIGGER_BOX_OPEN);
    }
    if props.disabled {
        trigger_box = trigger_box.class(cls::DISABLED);
    }

    let has_value = if props.multiple {
        !props.selected_values.is_empty()
    } else {
        props.selected.is_some()
    };
    if let Some(search) = &search {
        if let Some(leading) = props
            .trigger_leading
            .take()
            .filter(|_| has_value && search.query.is_none())
        {
            trigger_box = trigger_box.child(div().class(cls::TRIGGER_LEADING).child(leading));
        }
        let (on_query, on_active, on_select_key, toggle_key, toggle_focus, toggle_change) = (
            search.on_query.clone(),
            search.on_active.clone(),
            props.on_select.0.clone(),
            props.on_toggle.0.clone(),
            props.on_toggle.0.clone(),
            props.on_toggle.0.clone(),
        );
        let (active, rows) = (search.active, rows_for_keys.clone());
        let input = TextInputProps::new()
            .text(
                search
                    .query
                    .clone()
                    .or(searchable_label.clone())
                    .unwrap_or_default(),
            )
            .placeholder(props.placeholder.clone())
            .bare(true)
            .on_change(move |typed: &str| {
                on_query(typed.to_string());
                if !open {
                    toggle_change();
                }
            })
            .on_focus_change(move |focused: bool| {
                if focused && !open {
                    toggle_focus();
                }
            })
            .key_hook(
                move |key: NamedKey, _shift: bool, _text: &str| {
                    match key {
                        NamedKey::ArrowDown | NamedKey::ArrowUp => {
                            if !open {
                                toggle_key();
                            } else if !rows.is_empty() {
                                on_active(step_active(
                                    &rows,
                                    active,
                                    if key == NamedKey::ArrowDown { 1 } else { -1 },
                                ));
                            }
                            true
                        }
                        NamedKey::Enter if open => {
                            if let Some((value, false)) = rows.get(active) {
                                on_select_key(value.clone());
                            }
                            true
                        }
                        NamedKey::Escape if open => {
                            toggle_key();
                            true
                        }
                        _ => false,
                    }
                },
            )
            .build()
            .class(cls::SEARCH_INPUT);
        trigger_box = trigger_box.child(input);
        if let Some(trailing) = props
            .trigger_trailing
            .take()
            .filter(|_| has_value && search.query.is_none())
        {
            trigger_box = trigger_box.child(div().class(cls::TRIGGER_TRAILING).child(trailing));
        }
    } else if props.multiple && has_value {
        let mut tags = div().class(cls::TAGS);
        for value in &props.selected_values {
            let label = props
                .options
                .iter()
                .find(|o| &o.value == value)
                .map_or_else(|| value.clone(), |o| o.label.clone());
            let on_select = props.on_select.0.clone();
            let value = value.clone();
            tags = tags.child(tag(label, move || on_select(value.clone()), &fonts));
        }
        trigger_box = trigger_box.class(cls::TRIGGER_BOX_TAGS).child(tags);
    } else {
        let selected_label = props
            .selected
            .as_deref()
            .and_then(|value| props.options.iter().find(|o| o.value == value))
            .map(|o| o.selected_label.clone().unwrap_or_else(|| o.label.clone()));
        let (label_text, label_color) = match selected_label {
            Some(label) => (label, TEXT_DEFAULT),
            None => (props.placeholder.clone(), INPUT_PLACEHOLDER),
        };
        let weight = if props.text_only {
            Weight::MEDIUM
        } else {
            Weight::NORMAL
        };
        if let Some(leading) = props.trigger_leading.filter(|_| has_value) {
            trigger_box = trigger_box.child(div().class(cls::TRIGGER_LEADING).child(leading));
        }
        trigger_box = trigger_box.child(
            text(TextProps {
                content: label_text,
                size: 16.0,
                color: label_color,
                weight: weight.into(),
                fonts: Some(fonts.clone()),
                line_clamp: Some(1),
                ..Default::default()
            })
            .class(cls::TRIGGER_LABEL),
        );
        if let Some(trailing) = props.trigger_trailing.filter(|_| has_value) {
            trigger_box = trigger_box.child(div().class(cls::TRIGGER_TRAILING).child(trailing));
        }
    }
    if let Some(on_clear) = props.on_clear.filter(|_| has_value && !inert) {
        let clear = on_clear.0.clone();
        trigger_box = trigger_box.child(
            Node::new("button", ButtonWidget::new().with_on_click(move || clear()))
                .class(cls::CLEAR)
                .child(
                    super::icon::icon(icons::X_SMALL, TEXT_SECONDARY).class(cls::CLEAR_ICON),
                ),
        );
    }
    let trigger_box = trigger_box.child(Node::new("span", chevron).class(cls::CHEVRON));
    let mut trigger = Node::new("label", trigger_btn)
        .class(cls::TRIGGER_ROW)
        .child(trigger_box);
    let mut wrapper = div().class(cls::WRAPPER);
    if props.compact {
        wrapper = wrapper.class(cls::COMPACT);
    }
    if props.text_only {
        trigger = trigger.class(cls::TEXT_ONLY_FIT);
        wrapper = wrapper.class(cls::TEXT_ONLY_FIT);
    }
    let mut wrapper = wrapper.child(trigger);

    if open {
        let side = if props.panel_above {
            cls::PANEL_ABOVE
        } else {
            cls::PANEL_BELOW
        };
        let mut panel = Node::new("div", PopoverPanelWidget::new())
            .class(cls::PANEL)
            .class(side);
        let scrolls = props.options.len() > MAX_UNSCROLLED_OPTIONS;
        let mut rows = Vec::with_capacity(props.options.len());
        let mut current_group: Option<String> = None;
        for (index, option) in props.options.into_iter().enumerate() {
            if option.group.is_some() && option.group != current_group {
                current_group = option.group.clone();
                rows.push(div().class(cls::GROUP_HEADER).child(text(TextProps {
                    content: option.group.clone().unwrap_or_default(),
                    size: 14.0,
                    color: TEXT_MUTED,
                    weight: Weight::SEMI_BOLD.into(),
                    fonts: Some(fonts.clone()),
                    ..Default::default()
                })));
            }
            let is_selected = if props.multiple {
                props.selected_values.contains(&option.value)
            } else {
                props.selected.as_deref() == Some(option.value.as_str())
            };
            let value = option.value.clone();
            let on_select = props.on_select.0.clone();
            let mut option_btn = ButtonWidget::new();
            if !option.disabled {
                option_btn = option_btn.with_on_click(move || on_select(value.clone()));
            }
            let mut row = Node::new("button", option_btn).class(cls::OPTION);
            if is_selected && !props.multiple {
                row = row.class(cls::OPTION_SELECTED);
            }
            if search.as_ref().is_some_and(|s| s.active == index) {
                row = row.class(cls::OPTION_ACTIVE);
            }
            if option.disabled {
                row = row.class(cls::DISABLED);
            }
            if let Some(leading) = option.leading {
                row = row.child(div().class(cls::LEADING).child(leading));
            }
            let mut labels = div().class(cls::OPTION_LABELS).child(text(TextProps {
                content: option.label,
                size: 16.0,
                color: TEXT_STRONG,
                fonts: Some(fonts.clone()),
                ..Default::default()
            }));
            if let Some(description) = option.description {
                labels = labels.child(text(TextProps {
                    content: description,
                    size: 14.0,
                    color: TEXT_SECONDARY,
                    fonts: Some(fonts.clone()),
                    ..Default::default()
                }));
            }
            row = row.child(labels);
            if let Some(trailing) = option.trailing {
                row = row.child(trailing);
            }
            if props.multiple {
                row = row.child(
                    Node::new(
                        "div",
                        CheckboxWidget::new(
                            ToggleState::new(is_selected),
                        ),
                    )
                    .class(cls::CHECK),
                );
            } else if is_selected {
                row = row.child(
                    super::icon::icon(icons::CHECKMARK_SMALL_BOLD, TEXT_STRONG)
                        .class(cls::CHECK),
                );
            }
            rows.push(row);
        }
        if search.is_some() && rows.is_empty() {
            rows.push(div().class(cls::EMPTY).child(text(TextProps {
                content: requiem_i18n::t!("ui-select-no-results"),
                size: 14.0,
                color: TEXT_MUTED,
                fonts: Some(fonts.clone()),
                ..Default::default()
            })));
        }
        panel = if scrolls {
            panel.child(
                components::scroll_view()
                    .class(cls::PANEL_SCROLL)
                    .children(rows),
            )
        } else {
            panel.children(rows)
        };
        if !props.footer.is_empty() {
            panel = panel
                .child(div().class(cls::SEPARATOR))
                .children(props.footer);
        }
        wrapper = wrapper.child(panel);
    }

    wrapper
}

#[cfg(test)]
mod search_tests {
    use super::*;

    fn labels(options: &[SelectOption]) -> Vec<&str> {
        options.iter().map(|o| o.label.as_str()).collect()
    }

    #[test]
    fn typing_keeps_labels_that_start_with_it_first_then_those_that_contain_it() {
        let options = [
            "Samsung",
            "Microsoft 3D Fluent",
            "Noto Color Emoji",
            "Facebook",
            "Messenger",
        ]
        .iter()
        .map(|l| SelectOption::new(*l, *l))
        .collect();
        assert_eq!(
            labels(&filter_options(options, "fl")),
            ["Microsoft 3D Fluent"]
        );
        let options = [
            "Samsung",
            "Microsoft 3D Fluent",
            "Noto Color Emoji",
            "Facebook",
            "Messenger",
        ]
        .iter()
        .map(|l| SelectOption::new(*l, *l))
        .collect();
        assert_eq!(
            labels(&filter_options(options, "M")),
            [
                "Microsoft 3D Fluent",
                "Messenger",
                "Samsung",
                "Noto Color Emoji"
            ],
            "starts with M, then contains an m"
        );
        let options = ["Apple"]
            .iter()
            .map(|l| SelectOption::new(*l, *l))
            .collect();
        assert!(filter_options(options, "zzz").is_empty());
        let options = ["Apple", "Google"]
            .iter()
            .map(|l| SelectOption::new(*l, *l))
            .collect();
        assert_eq!(
            filter_options(options, "").len(),
            2,
            "nothing typed: everything"
        );
    }

    #[test]
    fn the_arrow_keys_skip_disabled_rows_and_wrap() {
        let rows = vec![
            ("a".to_string(), false),
            ("b".to_string(), true),
            ("c".to_string(), false),
        ];
        assert_eq!(step_active(&rows, 0, 1), 2);
        assert_eq!(step_active(&rows, 2, 1), 0);
        assert_eq!(step_active(&rows, 0, -1), 2);
        assert_eq!(step_active(&[("only".to_string(), true)], 0, 1), 0);
    }
}
