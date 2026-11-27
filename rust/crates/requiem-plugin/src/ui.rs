use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use serde_json::Value;

use crate::bindings::requiem::plugin::ui::ViewBuilder;

type Callback = Rc<dyn Fn(Option<Value>)>;

pub struct Node {
    tag: String,
    key: Option<String>,
    props: Vec<(String, Value)>,
    text: Option<String>,
    classes: Vec<String>,
    style: Option<String>,
    events: Vec<(String, Callback)>,
    children: Vec<Node>,
}

impl Node {
    pub fn new(tag: impl Into<String>) -> Self {
        Node {
            tag: tag.into(),
            key: None,
            props: Vec::new(),
            text: None,
            classes: Vec::new(),
            style: None,
            events: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: Node) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = Node>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.classes.push(class.into());
        self
    }

    pub fn style(mut self, declarations: impl Into<String>) -> Self {
        self.style = Some(declarations.into());
        self
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn prop(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.props.push((name.into(), value.into()));
        self
    }

    pub fn text(mut self, content: impl Into<String>) -> Self {
        self.text = Some(content.into());
        self
    }

    pub fn on(
        mut self,
        event: impl Into<String>,
        handler: impl Fn(Option<Value>) + 'static,
    ) -> Self {
        self.events.push((event.into(), Rc::new(handler)));
        self
    }

    pub fn tooltip(self, text: impl Into<String>) -> Self {
        self.prop("tooltip", text.into())
    }

    pub fn on_click(self, handler: impl Fn() + 'static) -> Self {
        self.on("click", move |_| handler())
    }
}

pub fn div() -> Node {
    Node::new("div")
}

pub fn span() -> Node {
    Node::new("span")
}

#[derive(Clone, Debug)]
pub struct Color(pub String);

impl From<&str> for Color {
    fn from(css: &str) -> Self {
        Color(css.to_string())
    }
}

impl From<String> for Color {
    fn from(css: String) -> Self {
        Color(css)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextVariant(pub &'static str);

macro_rules! text_variants {
    ($($name:ident = $css:literal),+ $(,)?) => {
        impl TextVariant {
            $(pub const $name: TextVariant = TextVariant($css);)+
        }
    };
}

text_variants! {
    TEXT_XXS_NORMAL = "text-xxs/normal", TEXT_XXS_MEDIUM = "text-xxs/medium", TEXT_XXS_SEMIBOLD = "text-xxs/semibold", TEXT_XXS_BOLD = "text-xxs/bold",
    TEXT_XS_NORMAL = "text-xs/normal", TEXT_XS_MEDIUM = "text-xs/medium", TEXT_XS_SEMIBOLD = "text-xs/semibold", TEXT_XS_BOLD = "text-xs/bold",
    TEXT_SM_NORMAL = "text-sm/normal", TEXT_SM_MEDIUM = "text-sm/medium", TEXT_SM_SEMIBOLD = "text-sm/semibold", TEXT_SM_BOLD = "text-sm/bold",
    TEXT_MD_NORMAL = "text-md/normal", TEXT_MD_MEDIUM = "text-md/medium", TEXT_MD_SEMIBOLD = "text-md/semibold", TEXT_MD_BOLD = "text-md/bold",
    TEXT_LG_NORMAL = "text-lg/normal", TEXT_LG_MEDIUM = "text-lg/medium", TEXT_LG_SEMIBOLD = "text-lg/semibold", TEXT_LG_BOLD = "text-lg/bold",
    HEADING_SM_NORMAL = "heading-sm/normal", HEADING_SM_MEDIUM = "heading-sm/medium", HEADING_SM_SEMIBOLD = "heading-sm/semibold", HEADING_SM_BOLD = "heading-sm/bold", HEADING_SM_EXTRABOLD = "heading-sm/extrabold",
    HEADING_MD_NORMAL = "heading-md/normal", HEADING_MD_MEDIUM = "heading-md/medium", HEADING_MD_SEMIBOLD = "heading-md/semibold", HEADING_MD_BOLD = "heading-md/bold", HEADING_MD_EXTRABOLD = "heading-md/extrabold",
    HEADING_LG_NORMAL = "heading-lg/normal", HEADING_LG_MEDIUM = "heading-lg/medium", HEADING_LG_SEMIBOLD = "heading-lg/semibold", HEADING_LG_BOLD = "heading-lg/bold", HEADING_LG_EXTRABOLD = "heading-lg/extrabold",
    HEADING_XL_NORMAL = "heading-xl/normal", HEADING_XL_MEDIUM = "heading-xl/medium", HEADING_XL_SEMIBOLD = "heading-xl/semibold", HEADING_XL_BOLD = "heading-xl/bold", HEADING_XL_EXTRABOLD = "heading-xl/extrabold",
    HEADING_XXL_NORMAL = "heading-xxl/normal", HEADING_XXL_MEDIUM = "heading-xxl/medium", HEADING_XXL_SEMIBOLD = "heading-xxl/semibold", HEADING_XXL_BOLD = "heading-xxl/bold", HEADING_XXL_EXTRABOLD = "heading-xxl/extrabold",
    DISPLAY_SM = "display-sm", DISPLAY_MD = "display-md", DISPLAY_LG = "display-lg",
    EYEBROW = "eyebrow", CODE = "code",
}

pub struct TextProps {
    content: String,
    variant: TextVariant,
    color: Option<Color>,
    line_clamp: Option<u32>,
}

impl TextProps {
    pub fn styled(content: impl Into<String>, variant: TextVariant) -> Self {
        TextProps {
            content: content.into(),
            variant,
            color: None,
            line_clamp: None,
        }
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn line_clamp(mut self, lines: u32) -> Self {
        self.line_clamp = Some(lines);
        self
    }

    pub fn build(self) -> Node {
        let mut node = Node::new("mana:text")
            .text(self.content)
            .prop("variant", self.variant.0);
        if let Some(color) = self.color {
            node = node.prop("color", color.0);
        }
        if let Some(lines) = self.line_clamp {
            node = node.prop("line-clamp", lines);
        }
        node
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HeadingLevel {
    #[default]
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

pub struct HeadingProps {
    level: HeadingLevel,
    content: String,
    variant: Option<TextVariant>,
    color: Option<Color>,
}

impl HeadingProps {
    pub fn new(level: HeadingLevel, content: impl Into<String>) -> Self {
        HeadingProps {
            level,
            content: content.into(),
            variant: None,
            color: None,
        }
    }

    pub fn variant(mut self, variant: TextVariant) -> Self {
        self.variant = Some(variant);
        self
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn build(self) -> Node {
        let level = self.level as u32 + 1;
        let mut node = Node::new("mana:heading")
            .text(self.content)
            .prop("level", level);
        if let Some(variant) = self.variant {
            node = node.prop("variant", variant.0);
        }
        if let Some(color) = self.color {
            node = node.prop("color", color.0);
        }
        node
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    CriticalPrimary,
    CriticalSecondary,
    Active,
    OverlayPrimary,
    OverlaySecondary,
    Expressive,
    Bare,
}

impl ButtonVariant {
    fn css(self) -> &'static str {
        match self {
            ButtonVariant::Primary => "primary",
            ButtonVariant::Secondary => "secondary",
            ButtonVariant::CriticalPrimary => "critical-primary",
            ButtonVariant::CriticalSecondary => "critical-secondary",
            ButtonVariant::Active => "active",
            ButtonVariant::OverlayPrimary => "overlay-primary",
            ButtonVariant::OverlaySecondary => "overlay-secondary",
            ButtonVariant::Expressive => "expressive",
            ButtonVariant::Bare => "bare",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    #[default]
    Medium,
    Small,
}

pub struct ButtonProps {
    variant: ButtonVariant,
    size: ButtonSize,
    label: String,
    on_click: Rc<dyn Fn()>,
    full_width: bool,
    disabled: bool,
    loading: bool,
    icon: Option<String>,
    tooltip: Option<String>,
}

impl ButtonProps {
    pub fn new(
        variant: ButtonVariant,
        label: impl Into<String>,
        on_click: impl Fn() + 'static,
    ) -> Self {
        ButtonProps {
            variant,
            size: ButtonSize::Medium,
            label: label.into(),
            on_click: Rc::new(on_click),
            full_width: false,
            disabled: false,
            loading: false,
            icon: None,
            tooltip: None,
        }
    }

    pub fn icon(mut self, name: impl Into<String>) -> Self {
        self.icon = Some(name.into());
        self
    }

    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip = Some(text.into());
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn full_width(mut self, on: bool) -> Self {
        self.full_width = on;
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }

    pub fn loading(mut self, on: bool) -> Self {
        self.loading = on;
        self
    }

    pub fn build(self) -> Node {
        let on_click = self.on_click;
        let mut node = Node::new("mana:button")
            .text(self.label)
            .prop("variant", self.variant.css())
            .prop(
                "size",
                if self.size == ButtonSize::Small {
                    "small"
                } else {
                    "medium"
                },
            )
            .prop("full-width", self.full_width)
            .prop("disabled", self.disabled)
            .prop("loading", self.loading)
            .on("click", move |_| on_click());
        if let Some(icon) = self.icon {
            node = node.prop("icon", icon);
        }
        if let Some(tooltip) = self.tooltip {
            node = node.prop("tooltip", tooltip);
        }
        node
    }
}

pub struct SwitchProps {
    checked: bool,
    on_change: Rc<dyn Fn(bool)>,
    label: Option<String>,
    description: Option<String>,
    disabled: bool,
}

impl SwitchProps {
    pub fn new(checked: bool, on_change: impl Fn(bool) + 'static) -> Self {
        SwitchProps {
            checked,
            on_change: Rc::new(on_change),
            label: None,
            description: None,
            disabled: false,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }

    pub fn build(self) -> Node {
        let on_change = self.on_change;
        let mut node = Node::new("mana:switch")
            .prop("checked", self.checked)
            .prop("disabled", self.disabled)
            .on("change", move |value| {
                on_change(value.and_then(|v| v.as_bool()).unwrap_or(false))
            });
        if let Some(label) = self.label {
            node = node.prop("label", label);
        }
        if let Some(description) = self.description {
            node = node.prop("description", description);
        }
        node
    }
}

pub struct TextInputProps {
    text: String,
    placeholder: String,
    password: bool,
    disabled: bool,
    max_length: Option<u32>,
    on_change: Option<Rc<dyn Fn(String)>>,
    on_submit: Option<Rc<dyn Fn(String)>>,
}

impl Default for TextInputProps {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInputProps {
    pub fn new() -> Self {
        TextInputProps {
            text: String::new(),
            placeholder: String::new(),
            password: false,
            disabled: false,
            max_length: None,
            on_change: None,
            on_submit: None,
        }
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn password(mut self, on: bool) -> Self {
        self.password = on;
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }

    pub fn max_length(mut self, max: u32) -> Self {
        self.max_length = Some(max);
        self
    }

    pub fn on_change(mut self, handler: impl Fn(String) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn on_submit(mut self, handler: impl Fn(String) + 'static) -> Self {
        self.on_submit = Some(Rc::new(handler));
        self
    }

    fn into_node(self, tag: &str) -> Node {
        let mut node = Node::new(tag)
            .prop("text", self.text)
            .prop("placeholder", self.placeholder)
            .prop("password", self.password)
            .prop("disabled", self.disabled);
        if let Some(max) = self.max_length {
            node = node.prop("max-length", max);
        }
        if let Some(handler) = self.on_change {
            node = node.on("change", move |value| {
                handler(
                    value
                        .and_then(|v| v.as_str().map(str::to_string))
                        .unwrap_or_default(),
                )
            });
        }
        if let Some(handler) = self.on_submit {
            node = node.on("submit", move |value| {
                handler(
                    value
                        .and_then(|v| v.as_str().map(str::to_string))
                        .unwrap_or_default(),
                )
            });
        }
        node
    }

    pub fn build(self) -> Node {
        self.into_node("mana:text-input")
    }
}

pub struct TextFieldProps {
    title: String,
    description: Option<String>,
    error: Option<String>,
    input: TextInputProps,
}

impl TextFieldProps {
    pub fn new(title: impl Into<String>, input: TextInputProps) -> Self {
        TextFieldProps {
            title: title.into(),
            description: None,
            error: None,
            input,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    pub fn build(self) -> Node {
        let mut node = self
            .input
            .into_node("mana:text-field")
            .prop("title", self.title);
        if let Some(description) = self.description {
            node = node.prop("description", description);
        }
        if let Some(error) = self.error {
            node = node.prop("error", error);
        }
        node
    }
}

pub struct FormItemProps {
    title: String,
    description: Option<String>,
    children: Node,
}

impl FormItemProps {
    pub fn new(title: impl Into<String>, children: Node) -> Self {
        FormItemProps {
            title: title.into(),
            description: None,
            children,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn build(self) -> Node {
        let mut node = Node::new("mana:form-item").prop("title", self.title);
        if let Some(description) = self.description {
            node = node.prop("description", description);
        }
        node.child(self.children)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Space(pub f32);

impl Space {
    pub const ZERO: Space = Space(0.0);
    pub const XXS: Space = Space(4.0);
    pub const XS: Space = Space(8.0);
    pub const SM: Space = Space(12.0);
    pub const MD: Space = Space(16.0);
    pub const LG: Space = Space(20.0);
    pub const XL: Space = Space(24.0);
    pub const XXL: Space = Space(32.0);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
    #[default]
    Stretch,
    Baseline,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Justify {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
}

pub struct StackProps {
    horizontal: bool,
    gap: Space,
    align: Align,
    justify: Justify,
    wrap: bool,
}

impl StackProps {
    pub fn vertical() -> Self {
        StackProps {
            horizontal: false,
            gap: Space::ZERO,
            align: Align::Stretch,
            justify: Justify::Start,
            wrap: false,
        }
    }

    pub fn horizontal() -> Self {
        StackProps {
            horizontal: true,
            ..Self::vertical()
        }
    }

    pub fn gap(mut self, gap: Space) -> Self {
        self.gap = gap;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    pub fn wrap(mut self, on: bool) -> Self {
        self.wrap = on;
        self
    }

    pub fn build(self) -> Node {
        let align = match self.align {
            Align::Start => "start",
            Align::Center => "center",
            Align::End => "end",
            Align::Stretch => "stretch",
            Align::Baseline => "baseline",
        };
        let justify = match self.justify {
            Justify::Start => "start",
            Justify::Center => "center",
            Justify::End => "end",
            Justify::SpaceBetween => "space-between",
            Justify::SpaceAround => "space-around",
        };
        Node::new("mana:stack")
            .prop(
                "direction",
                if self.horizontal {
                    "horizontal"
                } else {
                    "vertical"
                },
            )
            .prop("gap", self.gap.0)
            .prop("align", align)
            .prop("justify", justify)
            .prop("wrap", self.wrap)
    }
}

thread_local! {

    static CALLBACKS: RefCell<HashMap<String, Callback>> = RefCell::new(HashMap::new());
}

pub(crate) fn record(view: &str, root: Node) -> ViewBuilder {
    let builder = ViewBuilder::new();
    let prefix = format!("{view}:");
    CALLBACKS.with(|callbacks| {
        let mut callbacks = callbacks.borrow_mut();
        callbacks.retain(|id, _| !id.starts_with(&prefix));
        let mut next = 0u32;
        record_node(
            &builder,
            None,
            root,
            "0".to_string(),
            &prefix,
            &mut next,
            &mut callbacks,
        );
    });
    builder
}

fn record_node(
    builder: &ViewBuilder,
    parent: Option<u32>,
    node: Node,
    default_key: String,
    prefix: &str,
    next: &mut u32,
    callbacks: &mut HashMap<String, Callback>,
) {
    let key = node.key.unwrap_or(default_key);
    let id = builder.element(parent, &node.tag, &key);
    for (name, value) in &node.props {
        builder.prop(id, name, &value.to_string());
    }
    if let Some(text) = &node.text {
        builder.text(id, text);
    }
    for class in &node.classes {
        builder.class(id, class);
    }
    if let Some(style) = &node.style {
        builder.style(id, style);
    }
    for (event, callback) in node.events {
        let action = format!("{prefix}{next}");
        *next += 1;
        builder.on(id, &event, &action);
        callbacks.insert(action, callback);
    }
    for (index, child) in node.children.into_iter().enumerate() {
        let child_key = format!("{}{index}", child.tag);
        record_node(builder, Some(id), child, child_key, prefix, next, callbacks);
    }
}

pub(crate) fn dispatch(action_id: &str, value: Option<String>) {
    let callback = CALLBACKS.with(|callbacks| callbacks.borrow().get(action_id).cloned());
    match callback {
        Some(callback) => callback(value.and_then(|json| serde_json::from_str(&json).ok())),
        None => crate::log::debug(&format!(
            "UI action {action_id} has no handler (the view was re-rendered)"
        )),
    }
}

pub fn known_components() -> Vec<String> {
    crate::bindings::requiem::plugin::ui::known_components()
}

pub fn image(src: impl Into<String>, width: u32, height: u32) -> Node {
    Node::new("mana:image")
        .prop("src", src.into())
        .prop("width", width)
        .prop("height", height)
}

pub fn avatar(src: Option<&str>, name: &str, size: u32, status: Option<&str>) -> Node {
    let mut node = Node::new("mana:avatar")
        .prop("name", name.to_string())
        .prop("size", size);
    if let Some(src) = src {
        node = node.prop("src", src.to_string());
    }
    if let Some(status) = status {
        node = node.prop("status", status.to_string());
    }
    node
}

pub fn icon(name: &str, color: Option<&str>) -> Node {
    let node = Node::new("mana:icon").prop("name", name.to_string());
    match color {
        Some(color) => node.prop("color", color.to_string()),
        None => node,
    }
}

pub fn scroll() -> Node {
    Node::new("mana:scroll")
}

pub fn empty_state(title: &str, text: &str) -> Node {
    Node::new("mana:empty-state")
        .prop("title", title.to_string())
        .prop("text", text.to_string())
}

pub fn profile(name: impl Into<String>) -> Node {
    Node::new("mana:profile").prop("name", name.into())
}

pub fn section(title: &str, content: Node) -> Node {
    Node::new("mana:section")
        .prop("title", title.to_string())
        .child(content)
}

pub fn tab(id: &str, label: &str, content: Node) -> Node {
    Node::new("mana:tab")
        .prop("id", id.to_string())
        .prop("label", label.to_string())
        .child(content)
}

pub fn toast(text: &str, kind: &str) {
    crate::bindings::requiem::plugin::ui::toast(text, kind);
}

pub fn open_modal(view_id: &str, title: &str, size: &str) -> Result<(), crate::Error> {
    crate::bindings::requiem::plugin::ui::open_modal(view_id, title, size)
}

pub fn close_modal(view_id: &str) {
    crate::bindings::requiem::plugin::ui::close_modal(view_id);
}

pub fn refresh(view_id: &str) {
    crate::bindings::requiem::plugin::ui::refresh(view_id);
}
