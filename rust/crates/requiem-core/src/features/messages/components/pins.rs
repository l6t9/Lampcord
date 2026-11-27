use crate::models::Permissions;
use crate::query;
use crate::rest::endpoints::pins;
use crate::rest::endpoints;
use crate::rest;
use crate::state;
use crate::ui::author_style;
use crate::ui::classes::pins as cls;
use crate::ui::guild_settings;
use crate::ui::message_confirm;
use crate::ui::message_list;
use crate::ui::profile_popout;
use crate::ui::search;
use crate::ui;
use requiem_ui::app;
use requiem_ui::colors;
use requiem_ui::components::PopoverProps;
use requiem_ui::components::TooltipPosition;
use requiem_ui::components::TooltipProps;
use requiem_ui::components;
use requiem_ui::widget::primitives::HoverBoxWidget;
use std::cell::Cell;
use std::rc::Rc;

use requiem_ui::components::{
    ButtonProps, ButtonSize, ButtonVariant, MessageTextProps, ScrollViewProps, TextProps,
    TextVariant, div, icon, popover_backdrop, scroll_view_with,
};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::AbsoluteRect;
use requiem_ui::widget::primitives::{ImageWidget, PulsingEllipsisWidget};

use crate::models::id::marker::{ChannelMarker, MessageMarker};
use crate::models::{Channel, Id, Message};
use crate::ui::context_menu::{self as menu, ConfirmRequest};
use crate::ui::search::ResultsEnv;
use crate::ui::{TEXT_DEFAULT, TEXT_STRONG};
use requiem_ui::icons::{PIN, X_SMALL};

const WIDTH: f32 = 420.0;

const LOAD_AHEAD: f32 = 400.0;

thread_local! {

    static OPEN: Signal<Option<Id<ChannelMarker>>> = signal(None);

    static ANCHOR: Signal<AbsoluteRect> = signal(AbsoluteRect::default());

    static HOVERED: Signal<Option<Id<MessageMarker>>> = signal(None);
    static SCROLL: Cell<f32> = const { Cell::new(0.0) };
}

pub fn is_open() -> bool {
    OPEN.with(Signal::peek).is_some()
}

pub fn is_open_for(channel_id: Id<ChannelMarker>) -> bool {
    OPEN.with(Signal::get) == Some(channel_id)
}

pub fn close() {
    HOVERED.with(|h| h.set(None));
    OPEN.with(|o| o.set(None));
}

pub fn anchor(button: Node) -> Node {
    button.observe_rect(ANCHOR.with(Signal::clone))
}

pub fn toggle(channel_id: Id<ChannelMarker>) {
    if OPEN.with(Signal::peek) == Some(channel_id) {
        close();
        return;
    }
    open(channel_id);
}

fn open(channel_id: Id<ChannelMarker>) {
    SCROLL.with(|s| s.set(0.0));
    HOVERED.with(|h| h.set(None));
    OPEN.with(|o| o.set(Some(channel_id)));
    if let Some(env) = message_confirm::env() {
        let http = env.http.clone();
        env.queries.mutate(Vec::new(), move || async move {
            pins::ack(&http, channel_id).await
        });
    }
}

fn fetch(env: &ResultsEnv, channel_id: Id<ChannelMarker>) {
    let Some(http) = env.http.cloned() else {
        return;
    };
    env.state.pins.start_loading(channel_id);
    let state = env.state.clone();
    guild_settings::run(
        env.queries,
        move || async move {
            pins::list(&http, channel_id, None)
                .await
                .map_err(|e| e.to_string())
        },
        move |result| match result {
            Ok((page, has_more)) => {
                let ids: Vec<Id<MessageMarker>> = page.iter().map(|pin| pin.message.id).collect();
                state
                    .users
                    .observe_messages(page.iter().map(|pin| &pin.message));

                state.messages.set_pinned(channel_id, &ids, &[], !has_more);
                state.pins.loaded(channel_id, page, has_more, true);
            }
            Err(_) => state.pins.failed(channel_id),
        },
    );
}

pub fn build(env: &ResultsEnv) -> Node {
    let layer = div().class(cls::LAYER);
    let Some(channel_id) = OPEN.with(Signal::get) else {
        return layer;
    };
    let channel = ui::find_channel(env.state, channel_id)
        .map(|(_, c)| c)
        .or_else(|| env.state.dms.find(channel_id).map(|dm| dm.as_channel()));
    let Some(channel) = channel else { return layer };
    if env.state.pins.needs_fetch(channel_id) {
        fetch(env, channel_id);
    }
    let pins = env.state.pins.of(channel_id);

    let (window_w, window_h) = app::current_window_size();
    let anchor = ANCHOR.with(Signal::get);
    let (right_edge, bottom) = (anchor.x + anchor.width, anchor.y + anchor.height);
    let style = format!(
        "width: {WIDTH}px; max-height: {}px; right: {}px; top: {}px",
        window_h * 0.8,
        (window_w - right_edge).max(0.0),
        bottom + 8.0
    );

    let header = div()
        .class(cls::HEADER)
        .child(icon(PIN, colors::TEXT_SECONDARY).class(cls::HEADER_ICON))
        .child(
            TextProps::styled(
                requiem_i18n::t!("pins-title"),
                TextVariant::TEXT_LG_SEMIBOLD,
            )
            .color(TEXT_STRONG)
            .line_clamp(1_u32)
            .build(),
        );

    let is_dm = channel.guild_id.is_none();

    let mut panel = PopoverProps::default()
        .build()
        .class(cls::PANEL)
        .style(style)
        .child(header);
    if pins.items.is_empty() {
        if pins.loading || !pins.known {
            panel = panel.child(div().class(cls::LOADING).child(Node::new(
                "div",
                PulsingEllipsisWidget::new(TEXT_DEFAULT, 1.0),
            )));
        } else {
            panel = panel.child(empty_state(is_dm)).child(footer(is_dm));
        }
    } else {
        panel = panel.child(list(env, &channel, &pins));
    }
    layer
        .class(cls::LAYER_OPEN)
        .child(popover_backdrop(close))
        .child(panel)
}

fn list(env: &ResultsEnv, channel: &Channel, pins: &state::ChannelPins) -> Node {
    let channel_id = channel.id;
    let hovered = HOVERED.with(Signal::get);
    let mut column = div().class(cls::LIST);
    for pin in &pins.items {
        column = column.child(card(
            env,
            channel,
            &pin.message,
            hovered == Some(pin.message.id),
        ));
    }
    if pins.loading {
        column = column.child(div().class(cls::LOADING_MORE).child(Node::new(
            "div",
            PulsingEllipsisWidget::new(TEXT_DEFAULT, 1.0),
        )));
    }
    let more = pins.has_more && !pins.loading;
    let (state, queries, http) = (env.state.clone(), env.queries.clone(), env.http.cloned());
    let on_scroll = move |offset: f32, max: f32| {
        SCROLL.with(|s| s.set(offset));
        if more && max - offset < LOAD_AHEAD {
            let Some(http) = http.clone() else { return };
            load_more(&state, &queries, http, channel_id);
        }
    };
    scroll_view_with(ScrollViewProps {
        initial_offset_y: SCROLL.with(Cell::get),
        on_scroll: on_scroll.into(),
        ..Default::default()
    })
    .class(cls::SCROLL)
    .child(column)
}

fn load_more(
    state: &state::AppState,
    queries: &query::QueryClient,
    http: rest::HttpClient,
    channel_id: Id<ChannelMarker>,
) {
    let pins = state.pins.clone();
    if pins.by_channel.with_untracked(|all| {
        all.get(&channel_id)
            .is_none_or(|p| p.loading || !p.has_more)
    }) {
        return;
    }
    let Some(before) = pins.by_channel.with_untracked(|all| {
        all.get(&channel_id)
            .and_then(|p| p.items.last().map(|pin| pin.pinned_at.clone()))
    }) else {
        return;
    };
    pins.start_loading(channel_id);
    let state = state.clone();
    guild_settings::run(
        queries,
        move || async move {
            pins::list(&http, channel_id, Some(&before))
                .await
                .map_err(|e| e.to_string())
        },
        move |result| match result {
            Ok((page, has_more)) => {
                let ids: Vec<Id<MessageMarker>> = page.iter().map(|pin| pin.message.id).collect();
                state
                    .users
                    .observe_messages(page.iter().map(|pin| &pin.message));
                state.messages.set_pinned(channel_id, &ids, &[], false);
                state.pins.loaded(channel_id, page, has_more, false);
            }
            Err(_) => state.pins.failed(channel_id),
        },
    );
}

fn card(env: &ResultsEnv, channel: &Channel, message: &Message, hovered: bool) -> Node {
    let (channel_id, message_id) = (channel.id, message.id);
    let body = div()
        .class(cls::CARD_MESSAGE)
        .child(search::message_body(
            env,
            Some(channel),
            message,
            &[],
            content_width(),
            false,
        ));
    let mut card = Node::new(
        "div",
        HoverBoxWidget::new(move |inside| {
            HOVERED.with(|h| {
                if inside {
                    h.set(Some(message_id));
                } else if h.peek() == Some(message_id) {
                    h.set(None);
                }
            })
        }),
    )
    .class(cls::CARD)
    .child(body);
    if hovered {
        let jump_to = env.search.on_jump.clone();
        let jump = ButtonProps::new(
            ButtonVariant::Secondary,
            requiem_i18n::t!("pins-jump"),
            move || {
                close();
                (jump_to.borrow())(channel_id, message_id);
            },
        )
        .size(ButtonSize::Small)
        .build();
        let can_unpin = env
            .state
            .can(channel, Permissions::MANAGE_MESSAGES)
            || env
                .state
                .can(channel, Permissions::PIN_MESSAGES);
        let mut actions = div()
            .class(cls::ACTIONS)
            .child(div().class(cls::ACTION).child(jump));
        if can_unpin {
            let unpin = ButtonProps::default()
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Small)
                .icon(X_SMALL)
                .on_click_shift(move |shift: bool| {
                    if shift {
                        unpin_now(channel_id, message_id)
                    } else {
                        confirm_unpin(channel_id, message_id)
                    }
                })
                .build();
            actions = actions.child(
                div().class(cls::ACTION).child(
                    TooltipProps::default()
                        .text(requiem_i18n::t!("pins-unpin-message"))
                        .position(TooltipPosition::Top)
                        .build(unpin),
                ),
            );
        }
        card = card.child(actions);
    }
    card
}

fn content_width() -> f32 {
    WIDTH - 8.0 - 4.0 - 2.0 - 72.0 - 16.0
}

fn empty_state(is_dm: bool) -> Node {
    thread_local! {
        static IMAGE: Option<skia_safe::Image> = ui::asset_svg("image/pins_empty", 188, 240);
    }
    let image = IMAGE.with(Clone::clone);
    let text = if is_dm {
        requiem_i18n::t!("pins-empty-dm")
    } else {
        requiem_i18n::t!("pins-empty")
    };
    div()
        .class(cls::EMPTY)
        .child(Node::new("div", ImageWidget::new(image)).class(cls::EMPTY_IMAGE))
        .child(
            TextProps::styled(text, TextVariant::TEXT_MD_MEDIUM)
                .color(TEXT_DEFAULT)
                .center(true)
                .build()
                .class(cls::EMPTY_TEXT),
        )
}

fn footer(is_dm: bool) -> Node {
    let tip = if is_dm {
        requiem_i18n::t!("pins-protip-dm")
    } else {
        requiem_i18n::t!("pins-protip")
    };
    div().class(cls::FOOTER).child(
        div()
            .class(cls::PROTIP)
            .child(
                TextProps::styled(
                    format!("{}:", requiem_i18n::t!("inbox-protip")),
                    TextVariant::TEXT_SM_BOLD,
                )
                .color(colors::TEXT_POSITIVE)
                .center(true)
                .build(),
            )
            .child(
                TextProps::styled(tip, TextVariant::TEXT_SM_NORMAL)
                    .line_height(16.0_f32)
                    .color(TEXT_DEFAULT)
                    .center(true)
                    .build(),
            ),
    )
}

pub fn pin_action(
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    pinned: bool,
    shift: bool,
) {
    match (pinned, shift) {
        (false, true) => pin_now(channel_id, message_id),
        (false, false) => confirm_pin(channel_id, message_id),
        (true, true) => unpin_now(channel_id, message_id),
        (true, false) => confirm_unpin(channel_id, message_id),
    }
}

pub fn confirm_pin(channel_id: Id<ChannelMarker>, message_id: Id<MessageMarker>) {
    let Some(env) = message_confirm::env() else {
        return;
    };
    let channel_name = ui::find_channel(&env.state, channel_id).and_then(|(_, c)| c.name);
    let body = match channel_name {
        Some(name) => requiem_i18n::t!("pins-confirm-pin-body", channel = name),
        None => requiem_i18n::t!("pins-confirm-pin-body-dm"),
    };
    menu::confirm(ConfirmRequest {
        title: requiem_i18n::t!("pins-confirm-pin-title"),
        body,
        confirm_label: requiem_i18n::t!("pins-confirm-pin"),
        on_confirm: Rc::new(move || pin_now(channel_id, message_id)),
        critical: false,
        content: Some(Rc::new(move || {
            message_confirm::preview(channel_id, message_id, None)
        })),
    });
}

pub fn confirm_unpin(channel_id: Id<ChannelMarker>, message_id: Id<MessageMarker>) {
    menu::confirm(ConfirmRequest {
        title: requiem_i18n::t!("pins-unpin-message"),
        body: requiem_i18n::t!("pins-confirm-unpin-body"),
        confirm_label: requiem_i18n::t!("pins-confirm-unpin"),
        on_confirm: Rc::new(move || unpin_now(channel_id, message_id)),
        critical: true,
        content: Some(Rc::new(move || {
            message_confirm::preview(
                channel_id,
                message_id,
                Some(requiem_i18n::t!("pins-unpin-protip")),
            )
        })),
    });
}

pub fn pin_now(channel_id: Id<ChannelMarker>, message_id: Id<MessageMarker>) {
    let Some(env) = message_confirm::env() else {
        return;
    };
    let (http, state) = (env.http.clone(), env.state.clone());
    guild_settings::run(
        &env.queries,
        move || async move {
            pins::pin(&http, channel_id, message_id)
                .await
                .map_err(|e| endpoints::error_message(&e))
        },
        move |result| match result {
            Ok(()) => state
                .messages
                .set_pinned(channel_id, &[message_id], &[], false),
            Err(message) => menu::notify_failure(&format!(
                "{} {message}",
                requiem_i18n::t!("pins-pin-failed")
            )),
        },
    );
}

pub fn unpin_now(channel_id: Id<ChannelMarker>, message_id: Id<MessageMarker>) {
    let Some(env) = message_confirm::env() else {
        return;
    };
    let (http, state) = (env.http.clone(), env.state.clone());
    guild_settings::run(
        &env.queries,
        move || async move {
            pins::unpin(&http, channel_id, message_id)
                .await
                .map_err(|e| e.to_string())
        },
        move |result| match result {
            Ok(()) => {
                state
                    .messages
                    .set_pinned(channel_id, &[], &[message_id], false);
                state.pins.remove(channel_id, message_id);
            }
            Err(_) => menu::notify_failure(&requiem_i18n::t!("pins-unpin-failed")),
        },
    );
}

pub fn system_icon(color: skia_safe::Color) -> Node {
    div()
        .class(cls::SYSTEM_ICON_SLOT)
        .child(icon(PIN, color).class(cls::SYSTEM_ICON))
}

pub fn system_body(
    message: &Message,
    name: &str,
    fill: Option<&author_style::NameFill>,
    muted: skia_safe::Color,
    profile_popout: &profile_popout::ProfilePopoutState,
) -> Node {
    const USER: &str = "https://pins.invalid/user";
    const MESSAGE: &str = "https://pins.invalid/message";
    const PINS: &str = "https://pins.invalid/pins";
    let escaped = name.replace(['[', ']', '*', '_', '`', '~', '|', '\\'], "");
    let content = requiem_i18n::t!(
        "pins-system-message",
        user = format!("[{escaped}]({USER})"),
        message = format!("[{}]({MESSAGE})", requiem_i18n::t!("pins-system-a-message")),
        pins = format!(
            "[{}]({PINS})",
            requiem_i18n::t!("pins-system-pinned-messages")
        )
    );
    let (row_id, author, channel_id) = (message.id, message.author.id, message.channel_id);
    let pinned = message
        .message_reference
        .as_ref()
        .and_then(|r| r.message_id);
    let popout = profile_popout.clone();
    let on_link: Rc<dyn Fn(&str, AbsoluteRect)> =
        Rc::new(move |url: &str, link: AbsoluteRect| match url {
            USER => profile_popout::toggle_for(row_id, Some(author), link, &popout),
            MESSAGE => {
                if let Some(pinned) = pinned {
                    message_list::request_jump(channel_id, pinned);
                }
            }
            PINS => open(channel_id),
            _ => {}
        });
    let text = components::message_text(MessageTextProps {
        content,
        size: 16.0,
        color: muted,
        line_height: Some(22.0),
        link_style: Some((TEXT_STRONG, 500)),
        link_colors: vec![(
            USER.to_string(),
            author_style::inline_name_color(fill, TEXT_STRONG),
        )],
        on_link: Some(on_link),
        ..Default::default()
    });
    let mut body = div().class(cls::SYSTEM_BODY).child(text);
    if let Some(time) = message.format_header_timestamp() {
        body = body.child(
            TextProps::styled(time, TextVariant::TEXT_XS_MEDIUM)
                .line_height(22.0_f32)
                .color(muted)
                .build()
                .class(cls::SYSTEM_TIME),
        );
    }
    body
}
