use crate::models::Id;
use crate::plugins;
use crate::ui::classes::full_profile as classes_full_profile;
use crate::ui::dim;
use crate::ui::full_profile;
use crate::ui::media_viewer;
use crate::ui;
use crate::utils::guild;
use requiem_plugin_host::css;
use requiem_ui::colors;
use requiem_ui::components::TooltipProps;
use requiem_ui::components;
use requiem_ui::icons::Icon;
use requiem_ui::style::values;
use requiem_ui::timer;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use requiem_ui::components::{
    Align, AvatarStatusSize, ButtonProps, ButtonSize, ButtonVariant, FormItemProps, HeadingLevel,
    HeadingProps, Justify, Space, StackProps, SwitchProps, TabItem, TextFieldProps, TextInputProps,
    TextProps, TextVariant, avatar_with_status, div, icon,
};
use requiem_ui::node::Node;
use requiem_ui::signal::Signal;
use requiem_ui::widget::anim::ToggleMemo;
use requiem_ui::widget::primitives::{ClickAreaWidget, ImageWidget, PresenceStatus};
use serde_json::Value;

use crate::images::ImageCache;
use crate::plugins::{ViewNode, ViewTree};

use crate::ui::Fonts;

#[derive(Clone)]
pub struct ViewEnv {
    pub images: ImageCache,
    pub tick: Signal<u64>,
    pub fonts: Fonts,

    pub viewer: Option<media_viewer::MediaViewerState>,
}

pub type OnAction = Rc<dyn Fn(String, Option<Value>, bool)>;

const TYPING_PAUSE: Duration = Duration::from_millis(400);

thread_local! {

    static TOGGLES: RefCell<HashMap<String, ToggleMemo>> = RefCell::new(HashMap::new());
}

pub fn render(
    plugin_id: &str,
    tree: &ViewTree,
    on_action: OnAction,
    env: Option<&ViewEnv>,
) -> Node {
    let ctx = Ctx {
        plugin_id,
        tree,
        on_action,
        env,
    };
    div()
        .class(css::view_class(plugin_id))
        .children(tree.roots().map(|id| ctx.node(id, "")))
}

struct Ctx<'a> {
    plugin_id: &'a str,
    tree: &'a ViewTree,
    on_action: OnAction,
    env: Option<&'a ViewEnv>,
}

impl Ctx<'_> {
    fn node(&self, id: u32, parent_path: &str) -> Node {
        let view = self.tree.node(id);
        let path = format!("{parent_path}/{}", view.key);
        let children = || {
            self.tree
                .children(id)
                .map(|child| self.node(child, &path))
                .collect::<Vec<_>>()
        };
        let node = match view.tag.as_str() {
            "div" | "span" => {
                let mut node = match self.tree.action(id, "click") {
                    Some(action) => {
                        let on_click = self.action(action, true);
                        Node::new(
                            "div",
                            ClickAreaWidget::new().with_on_click(move || on_click(None)),
                        )
                        .children(children())
                    }
                    None => div().children(children()),
                };
                if let Some(text) = &view.text {
                    node = node.child(
                        TextProps::styled(text.clone(), TextVariant::TEXT_MD_NORMAL).build(),
                    );
                }
                node
            }
            "mana:text" => {
                let mut text = TextProps::styled(
                    view.text.clone().unwrap_or_default(),
                    text_variant(str_prop(view, "variant")).unwrap_or(TextVariant::TEXT_MD_NORMAL),
                );
                if let Some(lines) = u32_prop(view, "line-clamp") {
                    text = text.line_clamp(lines);
                }
                with_color(text.build(), view)
            }
            "mana:heading" => {
                let level = match u32_prop(view, "level").unwrap_or(1) {
                    1 => HeadingLevel::H1,
                    2 => HeadingLevel::H2,
                    3 => HeadingLevel::H3,
                    4 => HeadingLevel::H4,
                    5 => HeadingLevel::H5,
                    _ => HeadingLevel::H6,
                };
                let mut heading = HeadingProps::new(level, view.text.clone().unwrap_or_default());
                if let Some(variant) = text_variant(str_prop(view, "variant")) {
                    heading = heading.variant(variant);
                }
                with_color(heading.build(), view)
            }
            "mana:button" => {
                let on_click = self.tree.action(id, "click").map(|a| self.action(a, true));
                let variant = str_prop(view, "variant")
                    .and_then(ButtonVariant::parse)
                    .unwrap_or_default();
                let size = if str_prop(view, "size") == Some("small") {
                    ButtonSize::Small
                } else {
                    ButtonSize::Medium
                };
                let mut button =
                    ButtonProps::new(variant, view.text.clone().unwrap_or_default(), move || {
                        if let Some(on_click) = &on_click {
                            on_click(None);
                        }
                    })
                    .size(size)
                    .full_width(bool_prop(view, "full-width"))
                    .disabled(bool_prop(view, "disabled"))
                    .loading(bool_prop(view, "loading"));
                if let Some(name) = str_prop(view, "icon") {
                    button = button.icon(named_icon(name));

                    if view.text.as_deref().unwrap_or_default().is_empty() {
                        button.label = None;
                    }
                }
                match str_prop(view, "tooltip") {
                    Some(text) => TooltipProps::default()
                        .text(text.to_string())
                        .build(button.build()),
                    None => button.build(),
                }
            }
            "mana:switch" => {
                let checked = bool_prop(view, "checked");
                let state = TOGGLES.with(|toggles| {
                    toggles
                        .borrow_mut()
                        .entry(format!("{}{path}", self.plugin_id))
                        .or_insert_with(|| ToggleMemo::new(checked))
                        .track(checked)
                });
                let on_change = self.tree.action(id, "change").map(|a| self.action(a, true));
                let mut switch = SwitchProps::new(state, move |on: bool| {
                    if let Some(on_change) = &on_change {
                        on_change(Some(Value::Bool(on)));
                    }
                })
                .disabled(bool_prop(view, "disabled"));
                if let Some(label) = str_prop(view, "label") {
                    switch = switch.label(label.to_string());
                }
                if let Some(description) = str_prop(view, "description") {
                    switch = switch.description(description.to_string());
                }
                switch.build()
            }

            "mana:empty-state" => full_profile::empty_state(
                full_profile::empty_title(str_prop(view, "title").unwrap_or_default()),
                str_prop(view, "text").unwrap_or_default(),
                children(),
            ),
            "mana:scroll" => components::scroll_view().children(children()),
            "mana:section" => full_profile::section(
                str_prop(view, "title").unwrap_or_default(),
                div().children(children()),
            ),
            "mana:profile" => return self.profile(id, view, &path),
            "mana:image" => self.image(view),
            "mana:avatar" => self.avatar(view),
            "mana:icon" => match (str_prop(view, "name"), icon_color(view)) {
                (Some(name), Some(color)) => icon(named_icon(name), color),
                _ => div(),
            },
            "mana:text-input" => self.text_input(id, view).build(),
            "mana:text-field" => {
                let mut field = TextFieldProps::new(
                    str_prop(view, "title").unwrap_or_default(),
                    self.text_input(id, view),
                );
                if let Some(description) = str_prop(view, "description") {
                    field = field.description(description.to_string());
                }
                if let Some(error) = str_prop(view, "error") {
                    field = field.error(Some(error.to_string()));
                }
                field.build()
            }
            "mana:form-item" => {
                let body = div().children(children());
                let mut item =
                    FormItemProps::new(str_prop(view, "title").unwrap_or_default(), body);
                if let Some(description) = str_prop(view, "description") {
                    item = item.description(description.to_string());
                }
                return finish(item.build(), view);
            }
            "mana:stack" => {
                let stack = if str_prop(view, "direction") == Some("horizontal") {
                    StackProps::horizontal()
                } else {
                    StackProps::vertical()
                };
                let align = match str_prop(view, "align") {
                    Some("start") => Align::Start,
                    Some("center") => Align::Center,
                    Some("end") => Align::End,
                    Some("baseline") => Align::Baseline,
                    _ => Align::Stretch,
                };
                let justify = match str_prop(view, "justify") {
                    Some("center") => Justify::Center,
                    Some("end") => Justify::End,
                    Some("space-between") => Justify::SpaceBetween,
                    Some("space-around") => Justify::SpaceAround,
                    _ => Justify::Start,
                };
                let gap = view.props.get("gap").and_then(Value::as_f64).unwrap_or(0.0) as f32;
                stack
                    .gap(Space(gap))
                    .align(align)
                    .justify(justify)
                    .wrap(bool_prop(view, "wrap"))
                    .build()
                    .children(children())
            }
            other => {
                log::debug!(target: "plugin", "[{}] can't render `{other}`; showing an empty div", self.plugin_id);
                div().children(children())
            }
        };
        let node = finish(node, view);
        match str_prop(view, "tooltip") {
            Some(text) if view.tag != "mana:button" => {
                TooltipProps::default()
                    .text(text.to_string())
                    .build(node)
            }
            _ => node,
        }
    }

    fn image(&self, view: &ViewNode) -> Node {
        let Some(env) = self.env else { return div() };
        let Some(src) = str_prop(view, "src") else {
            return div();
        };
        let node = self.image_node(env, src);
        let radius = u32_prop(view, "radius").unwrap_or(0);
        let loaded = env
            .images
            .get_or_decode_image(&self.resolve_image(env, src))
            .is_some();
        let open = match (&env.viewer, str_prop(view, "open")) {
            (Some(viewer), Some(url)) if loaded => Some((
                viewer.clone(),
                media_viewer::MediaItem {
                    url: url.to_string(),
                    video_url: None,
                    width: None,
                    height: None,
                    placeholder: None,
                    filename: None,
                    size: None,
                    alt: None,
                    author: None,
                    is_attachment: false,
                    zoom_only: true,
                },
            )),
            _ => None,
        };
        match (u32_prop(view, "width"), u32_prop(view, "height"), open) {
            (Some(width), Some(height), Some((viewer, item))) => {
                let fill = node.style(format!("width: {width}px; height: {height}px; border-radius: {radius}px"));
                let dim = Node::new("div", dim::DimWidget { cutout: None }).class(classes_full_profile::DIM).style(format!("border-radius: {radius}px"));
                Node::new("div", ClickAreaWidget::new().with_on_click(move || viewer.open(vec![item.clone()], 0))).class(classes_full_profile::DIMMABLE).style(format!("width: {width}px; height: {height}px")).child(fill).child(dim)
            }

            (Some(width), Some(height), None) if !loaded => div()
                .style(format!("width: {width}px; height: {height}px; flex-shrink: 0; border-radius: {radius}px; background-color: var(--background-mod-subtle)"))
                .child(node.style(format!("width: {width}px; height: {height}px; border-radius: {radius}px"))),
            (Some(width), Some(height), None) => node.style(format!("width: {width}px; height: {height}px; flex-shrink: 0; border-radius: {radius}px")),
            _ => node,
        }
    }

    fn image_node(&self, env: &ViewEnv, src: &str) -> Node {
        let source = self.resolve_image(env, src);
        let initial = env.images.get_or_decode_image(&source);
        Node::new("div", ImageWidget::new(initial).with_cover_centered()).bind_image(
            env.tick.clone(),
            {
                let images = env.images.clone();
                move || images.get_or_decode_image(&source)
            },
        )
    }

    fn profile(&self, id: u32, view: &ViewNode, path: &str) -> Node {
        let Some(env) = self.env else { return div() };
        let (mut body, mut tabs) = (Vec::new(), Vec::new());
        for child in self.tree.children(id) {
            let node = self.tree.node(child);
            if node.tag == "mana:tab" {
                let child_path = format!("{path}/{}", node.key);
                let panel = finish(
                    div()
                        .style(
                            "flex-grow: 1; min-height: 0px; display: flex; flex-direction: column",
                        )
                        .children(
                            self.tree
                                .children(child)
                                .map(|c| self.node(c, &child_path))
                                .collect::<Vec<_>>(),
                        ),
                    node,
                );
                tabs.push((
                    TabItem::new(
                        str_prop(node, "id").unwrap_or_default(),
                        str_prop(node, "label").unwrap_or_default(),
                    ),
                    panel,
                ));
            } else {
                body.push(self.node(child, path));
            }
        }
        let on_tab: Rc<dyn Fn(String)> = match self.tree.action(id, "tab") {
            Some(action) => {
                let send = self.action(action, true);
                Rc::new(move |tab| send(Some(Value::String(tab))))
            }
            None => Rc::new(|_| {}),
        };
        let name = str_prop(view, "name").unwrap_or_default().to_string();
        let spec = full_profile::PluginProfile {
            banner: str_prop(view, "banner").map(|src| {
                let picture = self.image_node(env, src);

                let loaded = env.images.get_or_decode_image(&self.resolve_image(env, src)).is_some();
                match (&env.viewer, str_prop(view, "banner-full")) {
                    (Some(viewer), Some(full)) if loaded => {
                        let item = media_viewer::MediaItem { url: full.to_string(), video_url: None, width: None, height: None, placeholder: None, filename: None, size: None, alt: None, author: None, is_attachment: false, zoom_only: true };
                        let viewer = viewer.clone();
                        let dim = Node::new("div", dim::DimWidget { cutout: None }).class(classes_full_profile::BANNER_DIM);
                        Node::new("div", ClickAreaWidget::new().with_on_click(move || viewer.open(vec![item.clone()], 0))).style("position: relative").child(picture.style("width: 100%; height: 140px; border-top-left-radius: var(--radius-lg); border-top-right-radius: var(--radius-lg)")).child(dim)
                    }
                    _ => picture,
                }
            }),
            icon: str_prop(view, "icon").map(|src| {
                let picture = self.image_node(env, src);

                let loaded = env.images.get_or_decode_image(&self.resolve_image(env, src)).is_some();
                match (&env.viewer, str_prop(view, "icon-full")) {
                    (Some(viewer), Some(full)) if loaded => {
                        let item = media_viewer::MediaItem { url: full.to_string(), video_url: None, width: None, height: None, placeholder: None, filename: None, size: None, alt: None, author: None, is_attachment: false, zoom_only: true };
                        let viewer = viewer.clone();
                        let dim = Node::new("div", dim::DimWidget { cutout: None }).class(classes_full_profile::ICON_DIM);
                        Node::new("div", ClickAreaWidget::new().with_on_click(move || viewer.open(vec![item.clone()], 0))).child(picture.style("width: 120px; height: 120px; border-radius: var(--radius-xl)")).child(dim)
                    }
                    _ => picture,
                }
            }),
            acronym: guild::acronym(&name),
            name,
            subtitle: str_prop(view, "subtitle").unwrap_or_default().to_string(),
            stats: view
                .props
                .get("stats")
                .and_then(Value::as_array)
                .map(|stats| stats.iter().filter_map(|s| Some((s.get("text")?.as_str()?.to_string(), s.get("dot").and_then(Value::as_str) == Some("online")))).collect())
                .unwrap_or_default(),
            badges: view.props.get("badges").and_then(Value::as_array).map(|badges| badges.iter().filter_map(|b| b.as_str().map(str::to_string)).collect()).unwrap_or_default(),
            body,
            tabs,
            selected: str_prop(view, "tab").unwrap_or_default().to_string(),
            on_tab,
        };
        finish(full_profile::plugin_profile(spec), view)
    }

    fn avatar(&self, view: &ViewNode) -> Node {
        let Some(env) = self.env else { return div() };
        let name = str_prop(view, "name").unwrap_or("?");
        let user_id = view
            .props
            .get("user-id")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        let mut props =
            ui::user_avatar_props(Id::new(user_id.max(1)), "0", &env.fonts)
                .fallback_initial(
                    name.chars()
                        .next()
                        .map(|c| c.to_uppercase().to_string())
                        .unwrap_or_default(),
                );
        if let Some(src) = str_prop(view, "src") {
            let source = self.resolve_image(env, src);
            if let Some(image) = env.images.get_or_decode_image(&source) {
                props = props.image(image);
            }
        }
        let size = match u32_prop(view, "size").unwrap_or(40) {
            0..=20 => AvatarStatusSize::Px20,
            21..=32 => AvatarStatusSize::Px32,
            33..=40 => AvatarStatusSize::Px40,
            41..=80 => AvatarStatusSize::Px80,
            _ => AvatarStatusSize::Px120,
        };
        let status = match str_prop(view, "status") {
            Some("online") => Some(PresenceStatus::Online),
            Some("idle") => Some(PresenceStatus::Idle),
            Some("dnd") => Some(PresenceStatus::Dnd),
            Some("offline") => Some(PresenceStatus::Offline),
            _ => None,
        };
        avatar_with_status(props, size, status)
    }

    fn resolve_image(&self, env: &ViewEnv, src: &str) -> String {
        if src.starts_with("plugin-image:") {
            let key = format!("{}/{src}", self.plugin_id);
            if env.images.get_or_decode_image(&key).is_none() {
                if let Some(bytes) = plugins::plugin_image(self.plugin_id, src) {
                    env.images.insert_local(&key, bytes.into());
                }
            }
            return key;
        }
        src.to_string()
    }

    fn text_input(&self, id: u32, view: &ViewNode) -> TextInputProps {
        let mut input = TextInputProps::new()
            .text(str_prop(view, "text").unwrap_or_default().to_string())
            .placeholder(
                str_prop(view, "placeholder")
                    .unwrap_or_default()
                    .to_string(),
            )
            .password(bool_prop(view, "password"))
            .disabled(bool_prop(view, "disabled"));
        if let Some(max) = u32_prop(view, "max-length") {
            input = input.max_length(max as usize);
        }
        if let Some(action) = self.tree.action(id, "change") {
            let send = self.action(action, false);

            let generation = Rc::new(Cell::new(0u64));
            input = input.on_change(move |text: &str| {
                let mine = generation.get() + 1;
                generation.set(mine);
                let (generation, send, text) = (generation.clone(), send.clone(), text.to_string());
                timer::set_timeout(TYPING_PAUSE, move || {
                    if generation.get() == mine {
                        send(Some(Value::String(text)));
                    }
                });
            });
        }
        if let Some(action) = self.tree.action(id, "submit") {
            let send = self.action(action, true);
            input = input.on_submit(move |text: String| send(Some(Value::String(text))));
        }
        input
    }

    fn action(&self, action_id: &str, rerender: bool) -> Rc<dyn Fn(Option<Value>)> {
        let (on_action, action_id) = (self.on_action.clone(), action_id.to_string());
        Rc::new(move |value| on_action(action_id.clone(), value, rerender))
    }
}

fn finish(node: Node, view: &ViewNode) -> Node {
    let node = node.classes(view.classes.iter().cloned());
    match &view.style {
        Some(style) => node.style(style),
        None => node,
    }
}

fn with_color(node: Node, view: &ViewNode) -> Node {
    match str_prop(view, "color") {
        Some(color) if !color.contains(';') && !color.contains('}') => {
            node.style(format!("color: {color}"))
        }
        _ => node,
    }
}

pub fn named_icon(name: &str) -> Icon {
    thread_local! {
        static NAMES: RefCell<HashMap<String, &'static str>> = RefCell::new(HashMap::new());
    }
    let key = if name.starts_with("icon/") {
        name.to_string()
    } else {
        format!("icon/{name}")
    };
    let interned = NAMES.with(|names| {
        *names
            .borrow_mut()
            .entry(key.clone())
            .or_insert_with(|| Box::leak(key.into_boxed_str()))
    });
    Icon::Named(interned)
}

fn icon_color(view: &ViewNode) -> Option<skia_safe::Color> {
    str_prop(view, "color")
        .and_then(values::parse_color)
        .or(Some(colors::TEXT_SECONDARY))
}

fn str_prop<'a>(view: &'a ViewNode, name: &str) -> Option<&'a str> {
    view.props.get(name).and_then(Value::as_str)
}

fn bool_prop(view: &ViewNode, name: &str) -> bool {
    view.props
        .get(name)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn u32_prop(view: &ViewNode, name: &str) -> Option<u32> {
    view.props
        .get(name)
        .and_then(Value::as_u64)
        .map(|n| n.min(u32::MAX as u64) as u32)
}

fn text_variant(name: Option<&str>) -> Option<TextVariant> {
    let name = name?;
    let (family, weight) = name.split_once('/').unwrap_or((name, ""));
    Some(match (family, weight) {
        ("text-xxs", "normal") => TextVariant::TEXT_XXS_NORMAL,
        ("text-xxs", "medium") => TextVariant::TEXT_XXS_MEDIUM,
        ("text-xxs", "semibold") => TextVariant::TEXT_XXS_SEMIBOLD,
        ("text-xxs", "bold") => TextVariant::TEXT_XXS_BOLD,
        ("text-xs", "normal") => TextVariant::TEXT_XS_NORMAL,
        ("text-xs", "medium") => TextVariant::TEXT_XS_MEDIUM,
        ("text-xs", "semibold") => TextVariant::TEXT_XS_SEMIBOLD,
        ("text-xs", "bold") => TextVariant::TEXT_XS_BOLD,
        ("text-sm", "normal") => TextVariant::TEXT_SM_NORMAL,
        ("text-sm", "medium") => TextVariant::TEXT_SM_MEDIUM,
        ("text-sm", "semibold") => TextVariant::TEXT_SM_SEMIBOLD,
        ("text-sm", "bold") => TextVariant::TEXT_SM_BOLD,
        ("text-md", "normal") => TextVariant::TEXT_MD_NORMAL,
        ("text-md", "medium") => TextVariant::TEXT_MD_MEDIUM,
        ("text-md", "semibold") => TextVariant::TEXT_MD_SEMIBOLD,
        ("text-md", "bold") => TextVariant::TEXT_MD_BOLD,
        ("text-lg", "normal") => TextVariant::TEXT_LG_NORMAL,
        ("text-lg", "medium") => TextVariant::TEXT_LG_MEDIUM,
        ("text-lg", "semibold") => TextVariant::TEXT_LG_SEMIBOLD,
        ("text-lg", "bold") => TextVariant::TEXT_LG_BOLD,
        ("heading-sm", "normal") => TextVariant::HEADING_SM_NORMAL,
        ("heading-sm", "medium") => TextVariant::HEADING_SM_MEDIUM,
        ("heading-sm", "semibold") => TextVariant::HEADING_SM_SEMIBOLD,
        ("heading-sm", "bold") => TextVariant::HEADING_SM_BOLD,
        ("heading-sm", "extrabold") => TextVariant::HEADING_SM_EXTRABOLD,
        ("heading-md", "normal") => TextVariant::HEADING_MD_NORMAL,
        ("heading-md", "medium") => TextVariant::HEADING_MD_MEDIUM,
        ("heading-md", "semibold") => TextVariant::HEADING_MD_SEMIBOLD,
        ("heading-md", "bold") => TextVariant::HEADING_MD_BOLD,
        ("heading-md", "extrabold") => TextVariant::HEADING_MD_EXTRABOLD,
        ("heading-lg", "normal") => TextVariant::HEADING_LG_NORMAL,
        ("heading-lg", "medium") => TextVariant::HEADING_LG_MEDIUM,
        ("heading-lg", "semibold") => TextVariant::HEADING_LG_SEMIBOLD,
        ("heading-lg", "bold") => TextVariant::HEADING_LG_BOLD,
        ("heading-lg", "extrabold") => TextVariant::HEADING_LG_EXTRABOLD,
        ("heading-xl", "normal") => TextVariant::HEADING_XL_NORMAL,
        ("heading-xl", "medium") => TextVariant::HEADING_XL_MEDIUM,
        ("heading-xl", "semibold") => TextVariant::HEADING_XL_SEMIBOLD,
        ("heading-xl", "bold") => TextVariant::HEADING_XL_BOLD,
        ("heading-xl", "extrabold") => TextVariant::HEADING_XL_EXTRABOLD,
        ("heading-xxl", "normal") => TextVariant::HEADING_XXL_NORMAL,
        ("heading-xxl", "medium") => TextVariant::HEADING_XXL_MEDIUM,
        ("heading-xxl", "semibold") => TextVariant::HEADING_XXL_SEMIBOLD,
        ("heading-xxl", "bold") => TextVariant::HEADING_XXL_BOLD,
        ("heading-xxl", "extrabold") => TextVariant::HEADING_XXL_EXTRABOLD,
        ("display-sm", _) => TextVariant::DISPLAY_SM,
        ("display-md", _) => TextVariant::DISPLAY_MD,
        ("display-lg", _) => TextVariant::DISPLAY_LG,
        ("eyebrow", _) => TextVariant::EYEBROW,
        ("code", _) => TextVariant::CODE,
        _ => return None,
    })
}
