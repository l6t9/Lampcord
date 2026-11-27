use crate::ui::context_menu;
use requiem_ui::components::ButtonVariant;
use requiem_ui::components::{
    BadgeProps, ButtonProps, CheckboxProps, FormItemProps, RadioGroupProps, RadioOption,
    SelectOption, SelectProps, SwitchProps, TextInputProps, TextLinkProps, badge, div,
    input_accessory, search_bar, text_area, text_input,
};
use requiem_ui::components::{
    BadgeVariant, ButtonSize, CheckboxLabelType, InlineNoticeProps, ProgressBarProps, TabBarProps,
    TabItem, TextLinkVariant, toast,
};
use requiem_ui::icons::{PLUS_SMALL, STAR};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::anim::ToggleMemo;
use requiem_ui::widget::anim::ToggleState;
use std::rc::Rc;

pub struct StoryView {
    pub canvas: Node,
    pub controls: Vec<Node>,
}

pub struct Story {
    pub name: &'static str,
    pub build: Option<fn() -> StoryView>,
}

pub struct Section {
    pub title: &'static str,
    pub stories: &'static [Story],
}

pub fn find(name: &str) -> Option<&'static Story> {
    SECTIONS
        .iter()
        .flat_map(|s| s.stories)
        .find(|s| s.name == name)
}

const fn story(name: &'static str, build: fn() -> StoryView) -> Story {
    Story {
        name,
        build: Some(build),
    }
}

const fn missing(name: &'static str) -> Story {
    Story { name, build: None }
}

pub const SECTIONS: &[Section] = &[
    Section {
        title: "Text",
        stories: &[missing("Text"), missing("Typography Scales")],
    },
    Section {
        title: "Badge",
        stories: &[story("Badge", badge_story)],
    },
    Section {
        title: "Buttons",
        stories: &[
            story("Button", button_story),
            missing("IconButton"),
            missing("ToggleButton"),
            missing("ToggleIconButton"),
            story("TextButton", text_button_story),
            missing("ButtonGroup"),
        ],
    },
    Section {
        title: "Toolbar",
        stories: &[missing("Toolbar")],
    },
    Section {
        title: "Calendar",
        stories: &[missing("Calendar")],
    },
    Section {
        title: "Checkbox",
        stories: &[story("Checkbox", checkbox_story)],
    },
    Section {
        title: "CheckboxGroup",
        stories: &[missing("CheckboxGroup")],
    },
    Section {
        title: "DatePicker",
        stories: &[missing("DatePicker")],
    },
    Section {
        title: "DateRangePicker",
        stories: &[missing("DateRangePicker")],
    },
    Section {
        title: "RadioGroup",
        stories: &[story("RadioGroup", radio_group_story)],
    },
    Section {
        title: "RangeCalendar",
        stories: &[missing("RangeCalendar")],
    },
    Section {
        title: "Switch",
        stories: &[story("Switch", switch_story)],
    },
    Section {
        title: "Modal",
        stories: &[
            missing("Modal"),
            missing("ConfirmModal [Alpha]"),
            missing("ExpressiveModal"),
            missing("Dynamic Graphic Modal"),
            missing("LayerModal"),
            missing("MultiStepModal [Alpha]"),
            missing("BaseModal"),
        ],
    },
    Section {
        title: "InlineNotice",
        stories: &[
            story("InlineNotice", inline_notice_story),
            missing("All Types"),
            missing("Old vs New"),
        ],
    },
    Section {
        title: "Input Components",
        stories: &[
            story("TextInput", text_input_story),
            story("SearchBar", search_bar_story),
            story("TextArea", text_area_story),
            missing("EditableText"),
            missing("NumberInput"),
            missing("Input Collection"),
        ],
    },
    Section {
        title: "Select",
        stories: &[
            story("Select", select_story),
            missing("SearchableSelect"),
            missing("ComboBox"),
            missing("ListBox"),
        ],
    },
    Section {
        title: "Popover",
        stories: &[
            missing("Popover"),
            missing("VideoPopover"),
            missing("MultiStepPopover"),
        ],
    },
    Section {
        title: "ProgressBar",
        stories: &[story("ProgressBar", progress_bar_story)],
    },
    Section {
        title: "Toast",
        stories: &[story("Toast", toast_story)],
    },
    Section {
        title: "Tooltip",
        stories: &[missing("Tooltip"), missing("RichTooltip")],
    },
    Section {
        title: "Tabs",
        stories: &[story("Tabs", tabs_story)],
    },
    Section {
        title: "TagGroup",
        stories: &[
            missing("TagGroup"),
            missing("TagGroup links"),
            missing("TagGroup filter tags"),
        ],
    },
    Section {
        title: "Forms",
        stories: &[
            missing("FieldSet"),
            story("Field", field_story),
            missing("Field (Disabled, under-control)"),
            missing("Field (Disabled, under-label)"),
        ],
    },
    Section {
        title: "FloatingLayer",
        stories: &[missing("FloatingLayer")],
    },
    Section {
        title: "Rive Animations",
        stories: &[missing("All Animations")],
    },
];

fn text_control(title: &str, value: &Signal<String>) -> Node {
    let target = value.clone();
    let input = TextInputProps::new()
        .text(value.peek())
        .on_change(move |text: &str| target.set(text.to_string()));
    FormItemProps::new(title, text_input(input)).build()
}

fn bool_control(label: &'static str, value: &Signal<bool>) -> Node {
    let value = value.clone();

    let toggle = ToggleMemo::new(value.peek());
    Node::reactive(move || {
        let target = value.clone();
        CheckboxProps::new(toggle.track(value.get()), move |on: bool| target.set(on))
            .label(label.to_string())
            .build()
    })
}

fn select_control(
    title: &'static str,
    options: &'static [(&'static str, &'static str)],
    value: &Signal<String>,
) -> Node {
    let value = value.clone();
    let open = signal(false);
    Node::reactive(move || {
        let (toggle, pick, close) = (open.clone(), value.clone(), open.clone());
        let select = SelectProps::new(
            move || toggle.update(|o| *o = !*o),
            move |chosen: String| {
                pick.set(chosen);
                close.set(false);
            },
        )
        .options(
            options
                .iter()
                .map(|(value, label)| SelectOption::new(*value, *label))
                .collect::<Vec<_>>(),
        )
        .selected(value.get())
        .open(open.get())
        .build();
        FormItemProps::new(title, select).build()
    })
}

fn input_props(text: &str, placeholder: &str) -> TextInputProps {
    TextInputProps::new()
        .text(text.to_string())
        .placeholder(placeholder.to_string())
}

fn non_empty(text: String) -> Option<String> {
    Some(text).filter(|t| !t.is_empty())
}

fn canvas(render: impl Fn() -> Node + 'static) -> Node {
    Node::reactive(render)
}

const BUTTON_VARIANTS: &[(&str, &str)] = &[
    ("primary", "Primary"),
    ("secondary", "Secondary"),
    ("active", "Active"),
    ("critical-primary", "Critical Primary"),
    ("critical-secondary", "Critical Secondary"),
    ("overlay-primary", "Overlay Primary"),
    ("overlay-secondary", "Overlay Secondary"),
];

fn button_story() -> StoryView {
    let (text, variant, size) = (
        signal("Click me".to_string()),
        signal("primary".to_string()),
        signal("medium".to_string()),
    );
    let (disabled, loading, full_width, min_width) = (
        signal(false),
        signal(false),
        signal(false),
        signal(String::new()),
    );
    let controls = vec![
        text_control("Text", &text),
        select_control("Variant", BUTTON_VARIANTS, &variant),
        select_control("Size", &[("small", "Small"), ("medium", "Medium")], &size),
        bool_control("Disabled", &disabled),
        bool_control("Loading", &loading),
        bool_control("Full Width", &full_width),
        text_control("Min width", &min_width),
    ];
    let canvas = canvas(move || {
        let mut props = ButtonProps::default()
            .variant(ButtonVariant::parse(&variant.get()).unwrap_or_default())
            .size(if size.get() == "small" {
                ButtonSize::Small
            } else {
                ButtonSize::Medium
            })
            .label(text.get())
            .disabled(disabled.get())
            .loading(loading.get())
            .full_width(full_width.get());
        if let Ok(width) = min_width.get().trim().parse::<f32>() {
            props = props.min_width(width);
        }
        props.build()
    });
    StoryView { canvas, controls }
}

const BADGE_VARIANTS: &[(&str, &str)] = &[
    ("default", "Default"),
    ("brand", "Brand"),
    ("expressive", "Expressive"),
    ("common", "Common"),
    ("rare", "Rare"),
    ("epic", "Epic"),
    ("mythic", "Mythic"),
    ("critical", "Critical"),
];

fn badge_story() -> StoryView {
    let (text, variant, with_icon) = (
        signal("new".to_string()),
        signal("default".to_string()),
        signal(false),
    );
    let controls = vec![
        text_control("Text", &text),
        select_control("Variant", BADGE_VARIANTS, &variant),
        bool_control("Icon", &with_icon),
    ];
    let canvas = canvas(move || {
        let mut props = BadgeProps::default()
            .text(text.get())
            .variant(BadgeVariant::parse(&variant.get()).unwrap_or_default());
        if with_icon.get() {
            props = props.icon(STAR);
        }
        badge(props)
    });
    StoryView { canvas, controls }
}

fn inline_notice_story() -> StoryView {
    let text = signal("You need the Manage Roles permission to see this page.".to_string());
    let controls = vec![text_control("Text", &text)];
    let canvas = canvas(move || {
        InlineNoticeProps::new(text.get())
            .build()
            .style("width: 420px")
    });
    StoryView { canvas, controls }
}

fn tabs_story() -> StoryView {
    let (selected, roomy) = (signal("one".to_string()), signal(false));
    let controls = vec![bool_control("Roomy", &roomy)];
    let canvas = canvas(move || {
        let target = selected.clone();
        let items = vec![
            TabItem::new("one", "Overview"),
            TabItem::new("two", "Activity"),
            TabItem::new("three", "Settings"),
        ];
        div().style("width: 360px").child(
            TabBarProps::default()
                .items(items)
                .selected(selected.get())
                .roomy(roomy.get())
                .on_select(Rc::new(move |id: String| target.set(id))
                    as Rc<dyn Fn(String)>)
                .build(),
        )
    });
    StoryView { canvas, controls }
}

fn toast_story() -> StoryView {
    let message = signal("Link Copied!".to_string());
    let controls = vec![text_control("Message", &message)];
    let canvas = canvas(move || {
        let (ok, bad) = (message.clone(), message.clone());
        div()
            .style("display: flex; flex-direction: row; gap: 8px")
            .children([
                ButtonProps::new(ButtonVariant::Primary, "Show success", move || {
                    context_menu::actions()
                        .toasts
                        .show(&ok.peek(), toast::ToastKind::Success)
                })
                .build(),
                ButtonProps::new(ButtonVariant::CriticalPrimary, "Show failure", move || {
                    context_menu::actions()
                        .toasts
                        .show(&bad.peek(), toast::ToastKind::Failure)
                })
                .build(),
            ])
    });
    StoryView { canvas, controls }
}

fn progress_bar_story() -> StoryView {
    let percent = signal("60".to_string());
    let controls = vec![text_control("Percent", &percent)];
    let canvas = canvas(move || {
        let fraction = percent.get().trim().parse::<f32>().unwrap_or(0.0) / 100.0;
        div()
            .style("width: 320px")
            .child(ProgressBarProps::new(0x706c_6179, fraction).build())
    });
    StoryView { canvas, controls }
}

const TEXT_BUTTON_VARIANTS: &[(&str, &str)] = &[
    ("primary", "Primary"),
    ("secondary", "Secondary"),
    ("always-white", "Always White"),
    ("critical", "Critical"),
    ("anchor", "Anchor"),
];

fn text_button_story() -> StoryView {
    let (text, variant, small, disabled) = (
        signal("Click me".to_string()),
        signal("primary".to_string()),
        signal(false),
        signal(false),
    );
    let controls = vec![
        text_control("Text", &text),
        select_control("Variant", TEXT_BUTTON_VARIANTS, &variant),
        bool_control("Small", &small),
        bool_control("Disabled", &disabled),
    ];
    let canvas = canvas(move || {
        TextLinkProps::default()
            .label(text.get())
            .variant(TextLinkVariant::parse(&variant.get()).unwrap_or_default())
            .small(small.get())
            .disabled(disabled.get())
            .on_click(|| {})
            .build()
    });
    StoryView { canvas, controls }
}

fn checkbox_story() -> StoryView {
    let (label, description, label_type) = (
        signal("Accept terms and conditions".to_string()),
        signal(String::new()),
        signal("primary".to_string()),
    );
    let (checked, disabled) = (signal(false), signal(false));
    let controls = vec![
        text_control("Label", &label),
        text_control("Description", &description),
        select_control(
            "Label type",
            &[("primary", "Primary"), ("secondary", "Secondary")],
            &label_type,
        ),
        bool_control("Checked", &checked),
        bool_control("Disabled", &disabled),
    ];
    let toggle = ToggleMemo::new(checked.peek());
    let canvas = canvas(move || {
        let target = checked.clone();
        let mut props =
            CheckboxProps::new(toggle.track(checked.get()), move |on: bool| target.set(on))
                .label(label.get())
                .label_type(if label_type.get() == "secondary" {
                    CheckboxLabelType::Secondary
                } else {
                    CheckboxLabelType::Primary
                })
                .disabled(disabled.get());
        if let Some(description) = non_empty(description.get()) {
            props = props.description(description);
        }
        props.build()
    });
    StoryView { canvas, controls }
}

fn radio_group_story() -> StoryView {
    let (label, selected, disabled) = (
        signal("Mana Radio Group".to_string()),
        signal("1".to_string()),
        signal(false),
    );
    let controls = vec![
        text_control("Label", &label),
        bool_control("Disabled", &disabled),
    ];

    let toggles: Vec<(&'static str, ToggleMemo)> = ["1", "2", "3"]
        .into_iter()
        .map(|v| {
            (
                v,
                ToggleMemo::new(selected.peek() == v),
            )
        })
        .collect();
    let canvas = canvas(move || {
        let chosen = selected.get();
        let target = selected.clone();
        let option = |value: &'static str, text: &str| {
            RadioOption::new(
                value,
                text,
                toggles.iter().find(|(v, _)| *v == value).map_or_else(
                    || ToggleState::new(chosen == value),
                    |(_, memo)| memo.track(chosen == value),
                ),
            )
        };
        RadioGroupProps::new(move |value: String| target.set(value))
            .label(label.get())
            .options(vec![
                option("1", "Option 1").with_description("Helpful description text"),
                option("2", "Option 2"),
                option("3", "Option 3 (disabled)").disabled(true),
            ])
            .disabled(disabled.get())
            .build()
    });
    StoryView { canvas, controls }
}

fn switch_story() -> StoryView {
    let (label, description, error) = (
        signal("Always underline links".to_string()),
        signal(String::new()),
        signal(String::new()),
    );
    let (checked, icon, required, disabled) =
        (signal(false), signal(false), signal(false), signal(false));
    let controls = vec![
        text_control("Label", &label),
        text_control("Description", &description),
        text_control("Error", &error),
        bool_control("Checked", &checked),
        bool_control("Icon", &icon),
        bool_control("Required", &required),
        bool_control("Disabled", &disabled),
    ];
    let toggle = ToggleMemo::new(checked.peek());
    let canvas = canvas(move || {
        let target = checked.clone();
        let mut props =
            SwitchProps::new(toggle.track(checked.get()), move |on: bool| target.set(on))
                .label(label.get())
                .icon(icon.get())
                .required(required.get())
                .disabled(disabled.get());
        if let Some(description) = non_empty(description.get()) {
            props = props.description(description);
        }
        if let Some(error) = non_empty(error.get()) {
            props = props.error(error);
        }
        props.build()
    });
    StoryView { canvas, controls }
}

fn text_input_story() -> StoryView {
    let (placeholder, helper, error) = (
        signal("Enter some text...".to_string()),
        signal(String::new()),
        signal(String::new()),
    );
    let (max_length, character_count, clearable, read_only, disabled) = (
        signal(String::new()),
        signal(false),
        signal(false),
        signal(false),
        signal(false),
    );
    let controls = vec![
        text_control("Placeholder", &placeholder),
        text_control("Helper text", &helper),
        text_control("Error", &error),
        text_control("Max length", &max_length),
        bool_control("Show character count", &character_count),
        bool_control("Clearable", &clearable),
        bool_control("Read only", &read_only),
        bool_control("Disabled", &disabled),
    ];
    let canvas = canvas(move || {
        let mut props = input_props("", &placeholder.get())
            .show_character_count(character_count.get())
            .clearable(clearable.get())
            .read_only(read_only.get())
            .disabled(disabled.get());
        if let Some(helper) = non_empty(helper.get()) {
            props = props.helper_text(helper);
        }
        if let Some(error) = non_empty(error.get()) {
            props = props.error(error);
        }
        if let Ok(max) = max_length.get().trim().parse::<usize>() {
            props = props.max_length(max);
        }
        text_input(props)
    });
    StoryView { canvas, controls }
}

fn search_bar_story() -> StoryView {
    let placeholder = signal("Search...".to_string());
    let controls = vec![text_control("Placeholder", &placeholder)];
    let canvas = canvas(move || {
        div()
            .style("width: 300px")
            .child(search_bar(&placeholder.get(), |_| {}))
    });
    StoryView { canvas, controls }
}

fn text_area_story() -> StoryView {
    let (placeholder, rows, read_only, disabled) = (
        signal("Enter some text...".to_string()),
        signal("3".to_string()),
        signal(false),
        signal(false),
    );
    let controls = vec![
        text_control("Placeholder", &placeholder),
        text_control("Rows", &rows),
        bool_control("Read only", &read_only),
        bool_control("Disabled", &disabled),
    ];
    let canvas = canvas(move || {
        let props = input_props("", &placeholder.get())
            .rows(rows.get().trim().parse::<u32>().unwrap_or(3))
            .read_only(read_only.get())
            .disabled(disabled.get());
        text_area(props)
    });
    StoryView { canvas, controls }
}

fn animal_options() -> Vec<SelectOption> {
    vec![
        SelectOption::new("aardvark", "Aardvark")
            .with_description("This is an aardvark")
            .with_group("Wild"),
        SelectOption::new("kangaroo", "Kangaroo")
            .with_description("This is a kangaroo")
            .with_group("Wild"),
        SelectOption::new("cat", "Cat").with_group("Domesticated"),
        SelectOption::new("dog", "Dog").with_group("Domesticated"),
        SelectOption::new("snake", "Snake").disabled(true),
    ]
}

fn select_story() -> StoryView {
    let (placeholder, selected, open) = (
        signal("Select...".to_string()),
        signal(Vec::<String>::new()),
        signal(false),
    );
    let (multiple, clearable, read_only, disabled) =
        (signal(false), signal(false), signal(false), signal(false));
    let controls = vec![
        text_control("Placeholder", &placeholder),
        bool_control("Multiple", &multiple),
        bool_control("Clearable", &clearable),
        bool_control("Read only", &read_only),
        bool_control("Disabled", &disabled),
    ];
    let canvas = canvas(move || {
        let (toggle, pick, close, clear) = (
            open.clone(),
            selected.clone(),
            open.clone(),
            selected.clone(),
        );
        let is_multiple = multiple.get();
        let chosen = selected.get();
        let mut props = SelectProps::new(
            move || toggle.update(|o| *o = !*o),
            move |value: String| {
                pick.update(|values| {
                    if is_multiple {
                        match values.iter().position(|v| *v == value) {
                            Some(at) => {
                                values.remove(at);
                            }
                            None => values.push(value.clone()),
                        }
                    } else {
                        *values = vec![value.clone()];
                    }
                });
                if !is_multiple {
                    close.set(false);
                }
            },
        )
        .options(animal_options())
        .multiple(is_multiple)
        .placeholder(placeholder.get())
        .open(open.get())
        .read_only(read_only.get())
        .disabled(disabled.get());
        props = match (is_multiple, chosen.first().cloned()) {
            (true, _) => props.selected_values(chosen.clone()),
            (false, Some(value)) => props.selected(value),
            (false, None) => props,
        };
        if clearable.get() && !chosen.is_empty() {
            props = props.on_clear(move || clear.set(Vec::new()));
        }

        div().style("width: 300px").child(props.build())
    });
    StoryView { canvas, controls }
}

fn field_story() -> StoryView {
    let (title, description, helper, error) = (
        signal("Label".to_string()),
        signal(String::new()),
        signal(String::new()),
        signal(String::new()),
    );
    let (required, disabled, horizontal) = (signal(false), signal(false), signal(false));
    let controls = vec![
        text_control("Title", &title),
        text_control("Description", &description),
        text_control("Helper text", &helper),
        text_control("Error", &error),
        bool_control("Required", &required),
        bool_control("Disabled", &disabled),
        bool_control("Horizontal", &horizontal),
    ];
    let canvas = canvas(move || {
        let control = if horizontal.get() {
            SwitchProps::new(ToggleState::new(false), |_: bool| {}).build()
        } else {
            text_input(
                input_props("", "Enter some text...").trailing(input_accessory(PLUS_SMALL, || {})),
            )
        };
        let mut props = FormItemProps::default()
            .title(title.get())
            .required(required.get())
            .disabled(disabled.get())
            .horizontal(horizontal.get())
            .children(control);
        if let Some(description) = non_empty(description.get()) {
            props = props.description(description);
        }
        if let Some(helper) = non_empty(helper.get()) {
            props = props.helper_text(helper);
        }
        if let Some(error) = non_empty(error.get()) {
            props = props.error(error);
        }
        div().style("width: 100%").child(props.build())
    });
    StoryView { canvas, controls }
}
