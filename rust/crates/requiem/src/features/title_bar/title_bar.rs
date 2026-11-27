use crate::updater;
use requiem_core::ui::devtools;
use requiem_core::ui::inbox;
use requiem_core::utils::guild;
use requiem_ui::colors::{GREEN_360, TEXT_MUTED, TEXT_SECONDARY};
use requiem_ui::components::ButtonVariant;
use requiem_ui::components::{Align, Justify, StackProps};
use requiem_ui::icons::Icon;
use requiem_ui::widget::primitives;
use requiem_updater::run::State;
use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use requiem_core::config::BackendConfig;
use requiem_core::images::ImageCache;
use requiem_core::models::Id;
use requiem_core::models::id::marker::{ChannelMarker, GuildMarker};
use requiem_core::state::AppState;
use requiem_core::ui::{Fonts, TEXT_DEFAULT};
use requiem_ui::app::{WindowAction, request_window_action};
use requiem_ui::components::{
    AvatarProps, ButtonProps, TextProps, TooltipPosition, TooltipProps, avatar, div, icon, text,
    tooltip,
};
use requiem_ui::event::{EventResult, MouseButton, UiEvent};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::primitives::AvatarShape;
use requiem_ui::widget::{AbsoluteRect, Widget, WidgetId, WidgetTree};
use skia_safe::{Canvas, Color};

use crate::shell::classes::title_bar as classes;
use requiem_ui::icons::{
    ARROW_SMALL_LEFT, ARROW_SMALL_RIGHT, CIRCLE_QUESTION, DOWNLOAD, INBOX, STAFF_BADGE, WIN_CLOSE,
    WIN_MAXIMIZE, WIN_MINIMIZE,
};

const FALLBACK_BACKGROUND: Color = Color::from_argb(255, 0x1a, 0x1a, 0x1e);

const INTERACTIVE_TEXT_DISABLED: Color = Color::from_argb(77, 0xab, 0xac, 0xb2);

const DOUBLE_CLICK: Duration = Duration::from_millis(500);

#[derive(Clone, Default)]
struct Navigation {
    entries: Vec<Id<ChannelMarker>>,
    index: usize,
}

thread_local! {
    static NAVIGATION: Signal<Navigation> = signal(Navigation::default());

    static REPLAYING: Cell<bool> = const { Cell::new(false) };
}

pub fn record_navigation(previous: Option<Id<ChannelMarker>>, next: Id<ChannelMarker>) {
    if REPLAYING.with(Cell::get) {
        return;
    }
    NAVIGATION.with(|nav| {
        nav.update(|nav| {
            if nav.entries.is_empty() {
                nav.entries.extend(previous.filter(|p| *p != next));
            } else if nav.entries.get(nav.index) == Some(&next) {
                return;
            } else {
                nav.entries.truncate(nav.index + 1);
            }
            nav.entries.push(next);
            nav.index = nav.entries.len() - 1;
        });
    });
}

pub fn go_back(on_select_channel: &Rc<dyn Fn(Id<ChannelMarker>)>) {
    navigate(-1, on_select_channel);
}

pub fn go_forward(on_select_channel: &Rc<dyn Fn(Id<ChannelMarker>)>) {
    navigate(1, on_select_channel);
}

pub const INBOX_BUTTON_DOM_ID: &str = "inbox-button";

fn navigate(step: isize, on_select_channel: &Rc<dyn Fn(Id<ChannelMarker>)>) {
    let target = NAVIGATION.with(|nav| {
        let nav = nav.peek();
        let index = nav.index.checked_add_signed(step)?;
        nav.entries.get(index).map(|id| (index, *id))
    });
    let Some((index, channel_id)) = target else {
        return;
    };
    REPLAYING.with(|r| r.set(true));
    on_select_channel(channel_id);
    REPLAYING.with(|r| r.set(false));
    NAVIGATION.with(|nav| nav.update(|nav| nav.index = index));
}

struct DragRegion;

impl Widget for DragRegion {
    fn paint(&self, _canvas: &Canvas, _rect: AbsoluteRect, _style: &ComputedStyle) {}

    fn hit_priority(&self) -> bool {
        true
    }

    fn handle_event(&mut self, event: &UiEvent, _rect: AbsoluteRect) -> EventResult {
        let UiEvent::MouseDown {
            button: MouseButton::Left,
            ..
        } = event
        else {
            return EventResult::Ignored;
        };
        let now = Instant::now();
        let double = LAST_PRESS
            .with(|last| last.replace(Some(now)))
            .is_some_and(|t| now.duration_since(t) < DOUBLE_CLICK);
        if double {
            LAST_PRESS.with(|last| last.set(None));
            request_window_action(WindowAction::ToggleMaximize);
        } else {
            request_window_action(WindowAction::Drag);
        }
        EventResult::Handled
    }
}

thread_local! {

    static LAST_PRESS: Cell<Option<Instant>> = const { Cell::new(None) };
}

fn icon_button(classes: &[&'static str], glyph: Node, on_click: impl Fn() + 'static) -> Node {
    ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(
            classes
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>(),
        )
        .on_click(on_click)
        .build()
        .child(glyph)
}

fn nav_button(
    side_class: &'static str,
    path: Icon,
    enabled: bool,
    on_click: impl Fn() + 'static,
) -> Node {
    let (color, state_class) = if enabled {
        (TEXT_SECONDARY, classes::NAV_BUTTON_ENABLED)
    } else {
        (INTERACTIVE_TEXT_DISABLED, classes::NAV_BUTTON_DISABLED)
    };
    icon_button(
        &[classes::NAV_BUTTON, side_class, state_class],
        icon(path, color).class(classes::NAV_ICON),
        move || {
            if enabled {
                on_click();
            }
        },
    )
}

#[allow(clippy::too_many_arguments)]
pub fn build(
    tree: &mut WidgetTree,
    parent: WidgetId,
    state: &AppState,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    selected_guild_id: Option<Id<GuildMarker>>,
    home_title: Option<(String, Icon)>,
    on_select_channel: &Rc<dyn Fn(Id<ChannelMarker>)>,
    devtools: &devtools::DevtoolsContext,
) {
    let guilds = state.guilds.list.get();
    let guild = selected_guild_id.and_then(|id| guilds.iter().find(|g| g.id == id));
    let nav = NAVIGATION.with(Signal::get);

    let bar = tree.mount(
        parent,
        Node::new("div", DragRegion).class(classes::APP_TITLE_BAR),
    );

    let title_id = tree.mount(bar, div().class(classes::APP_TITLE_BAR_TITLE));
    if let Some(guild) = guild {
        let image = guild.icon.as_deref().and_then(|hash| {
            let url = backend.guild_icon_url(guild.id, hash);
            images.get_or_decode_image(&url)
        });
        let guild_icon = avatar(AvatarProps {
            image,
            fallback_initial: guild::acronym(&guild.name),
            fallback_background: FALLBACK_BACKGROUND,
            typeface: Some(fonts.typeface.clone()),
            shape: AvatarShape::Squircle,
            status_cutout: None,
            fallback_icon: None,
            image_binding: None,
        })
        .class(classes::APP_TITLE_BAR_ICON);
        tree.mount(title_id, guild_icon);
        tree.mount(
            title_id,
            text(TextProps {
                content: guild.name.clone(),
                size: 14.0,
                color: TEXT_DEFAULT,
                line_clamp: Some(1),
                ..Default::default()
            }),
        );
    }

    if let Some((title, icon_path)) = home_title {
        tree.mount(
            title_id,
            icon(icon_path, TEXT_MUTED).class(classes::APP_TITLE_BAR_ICON),
        );
        tree.mount(
            title_id,
            text(TextProps {
                content: title.to_string(),
                size: 14.0,
                color: TEXT_DEFAULT,
                line_clamp: Some(1),
                ..Default::default()
            }),
        );
    }

    let (back_select, forward_select) = (on_select_channel.clone(), on_select_channel.clone());
    let leading = StackProps::horizontal()
        .align(Align::Center)
        .gap(12)
        .full_width(false)
        .build()
        .child(div().class(classes::BACK_FORWARD).children([
            nav_button(
                classes::NAV_BACK,
                ARROW_SMALL_LEFT,
                nav.index > 0,
                move || navigate(-1, &back_select),
            ),
            nav_button(
                classes::NAV_FORWARD,
                ARROW_SMALL_RIGHT,
                nav.index + 1 < nav.entries.len(),
                move || navigate(1, &forward_select),
            ),
        ]));
    tree.mount(bar, leading);

    let win_buttons = window_buttons();
    let devtools = devtools.clone();
    let mut icons = vec![
        inbox_button(state),
        icon_button(
            &[classes::TOP_BAR_ICON_BUTTON],
            icon(CIRCLE_QUESTION, TEXT_MUTED).class(classes::TOP_BAR_ICON),
            || {},
        ),
        icon_button(
            &[classes::TOP_BAR_ICON_BUTTON],
            icon(STAFF_BADGE, TEXT_MUTED).class(classes::TOP_BAR_ICON),
            move || devtools::toggle(&devtools),
        ),
    ];

    icons.extend(update_button());
    icons.push(win_buttons);
    let trailing = StackProps::horizontal()
        .align(Align::Center)
        .justify(Justify::End)
        .gap(12)
        .full_width(false)
        .build()
        .children(icons);
    tree.mount(bar, trailing);
}

fn update_button() -> Option<Node> {
    let (button, label) = match updater::state() {
        State::Downloading { .. } => (
            icon_button(
                &[classes::TOP_BAR_ICON_BUTTON, classes::UPDATE_DISABLED],
                icon(DOWNLOAD, GREEN_360).class(classes::UPDATE_BOUNCE),
                || {},
            ),
            requiem_i18n::t!("updater-downloading"),
        ),
        State::Ready(staged) => (
            icon_button(
                &[classes::TOP_BAR_ICON_BUTTON],
                icon(DOWNLOAD, GREEN_360).class(classes::TOP_BAR_ICON),
                move || updater::confirm_restart(staged.clone()),
            ),
            requiem_i18n::t!("updater-ready"),
        ),
        _ => return None,
    };
    Some(tooltip(
        button,
        TooltipProps::default()
            .text(label)
            .position(TooltipPosition::Bottom),
    ))
}

fn inbox_button(state: &AppState) -> Node {
    let waiting = {
        let any_unread = state.read_states.any_unread();
        Node::reactive(move || {
            if !any_unread.get() {
                return div();
            }
            div().class(classes::TOP_BAR_BADGE)
        })
        .class(classes::TOP_BAR_BADGE_SLOT)
    };
    ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![classes::TOP_BAR_ICON_BUTTON.to_string()])
        .on_click_at(|rect: AbsoluteRect| inbox::toggle(rect))
        .build()
        .dom_id(INBOX_BUTTON_DOM_ID)
        .child(icon(INBOX, TEXT_MUTED).class(classes::TOP_BAR_ICON))
        .child(waiting)
}

fn window_buttons() -> Node {
    Node::new("div", primitives::TopLayerWidget).child(window_buttons_row())
}

fn window_buttons_row() -> Node {
    StackProps::horizontal()
        .align(Align::Center)
        .justify(Justify::End)
        .gap(4)
        .full_width(false)
        .build()
        .children([
            div().class(classes::WIN_BUTTONS_DIVIDER),
            icon_button(
                &[classes::WIN_BUTTON, classes::WIN_BUTTON_MIN_MAX],
                icon(WIN_MINIMIZE, TEXT_SECONDARY).class(classes::WIN_ICON),
                || request_window_action(WindowAction::Minimize),
            ),
            icon_button(
                &[classes::WIN_BUTTON, classes::WIN_BUTTON_MIN_MAX],
                icon(WIN_MAXIMIZE, TEXT_SECONDARY).class(classes::WIN_ICON),
                || request_window_action(WindowAction::ToggleMaximize),
            ),
            icon_button(
                &[classes::WIN_BUTTON, classes::WIN_BUTTON_CLOSE],
                icon(WIN_CLOSE, TEXT_SECONDARY).class(classes::WIN_ICON),
                || request_window_action(WindowAction::Close),
            ),
        ])
}

pub fn bare() -> Node {
    Node::new("div", DragRegion)
        .class(classes::APP_TITLE_BAR)
        .child(
            StackProps::horizontal()
                .align(Align::Center)
                .gap(12)
                .full_width(false)
                .build(),
        )
        .child(
            StackProps::horizontal()
                .align(Align::Center)
                .justify(Justify::End)
                .gap(12)
                .full_width(false)
                .build()
                .child(window_buttons()),
        )
}
