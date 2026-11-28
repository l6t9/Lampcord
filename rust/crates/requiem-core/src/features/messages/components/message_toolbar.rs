use crate::models::Permissions;
use crate::rest::endpoints::reactions as endpoints_reactions;
use crate::ui::classes::message_list;
use crate::ui::classes::message_list as cls;
use crate::ui::context_menus;
use crate::ui::forward;
use crate::ui::message_edit;
use crate::ui::pins;
use crate::ui::reactions;
use crate::ui;
use crate::utils::prefs;
use requiem_ui::app;
use requiem_ui::components::ButtonVariant;
use requiem_ui::icons::Icon;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use requiem_ui::components::{ButtonProps, TooltipPosition, TooltipProps, div, icon, tooltip};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::AbsoluteRect;

use crate::ui::context_menu as menu;
use crate::ui::context_menus::{MenuEnv, MessageTarget, message_items};
use requiem_ui::icons::{
    ARROW_ANGLE_LEFT_UP, ARROW_ANGLE_RIGHT_UP, CHAT_MARK_UNREAD, EMOJI_SLIGHTLY_SMILING_FACE, ID,
    LINK, MORE_HORIZONTAL, PENCIL, PIN, TRASH,
};

thread_local! {

    static HOVERED: Cell<Option<u64>> = const { Cell::new(None) };

    static ROW_HOVER: RefCell<HashMap<u64, Signal<bool>>> = RefCell::new(HashMap::new());
}

const ROW_FLAGS_KEPT: usize = 2000;

fn row_flag(message_id: u64) -> Signal<bool> {
    ROW_HOVER.with(|flags| {
        let mut flags = flags.borrow_mut();
        if flags.len() > ROW_FLAGS_KEPT {
            flags.retain(|_, flag| flag.holders() > 1);
        }
        flags
            .entry(message_id)
            .or_insert_with(|| signal(false))
            .clone()
    })
}

pub fn is_hovered(message_id: u64) -> bool {
    row_flag(message_id).get()
}

pub fn on_row_hover(message_id: u64) -> impl FnMut(bool) + 'static {
    move |inside| {
        let previous = HOVERED.with(Cell::get);
        if inside {
            HOVERED.with(|h| h.set(Some(message_id)));
            if let Some(previous) = previous.filter(|p| *p != message_id) {
                row_flag(previous).set(false);
            }
            row_flag(message_id).set(true);
        } else if previous == Some(message_id) {
            HOVERED.with(|h| h.set(None));
            row_flag(message_id).set(false);
        }
    }
}

pub fn slot(env: &MenuEnv, target: &Rc<MessageTarget>) -> Node {
    let (env, target) = (env.clone(), target.clone());
    let id = target.id.get();
    let hovered = row_flag(id);
    Node::reactive(move || {
        if hovered.get() {
            build(&env, &target)
        } else {
            div()
        }
    })
    .class(message_list::MESSAGE_TOOLBAR_SLOT)
}

fn build(env: &MenuEnv, target: &Rc<MessageTarget>) -> Node {
    let own = env.own_user_id == Some(target.author_id);
    let permissions = context_menus::message_permissions(&env.state, target.channel_id);

    let expanded = app::shift_down() && !reactions::picker_open();
    let can_reply = permissions.contains(Permissions::SEND_MESSAGES);
    let can_pin = permissions.contains(Permissions::MANAGE_MESSAGES)
        || permissions.contains(Permissions::PIN_MESSAGES);
    let can_delete = own || permissions.contains(Permissions::MANAGE_MESSAGES);

    let button = |label: &str, content: Node, on_click: Option<Rc<dyn Fn()>>| {
        let mut props = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::MESSAGE_TOOLBAR_BTN.to_string()]);
        if let Some(on_click) = on_click {
            props = props.on_click(move || on_click());
        }
        let node = props.build().child(content);
        tooltip(
            node,
            TooltipProps::default()
                .text(label.to_string())
                .position(TooltipPosition::Top),
        )
    };
    let glyph = |path: Icon| icon(path, ui::TEXT_SECONDARY).class(cls::MESSAGE_TOOLBAR_ICON);

    let mut bar = div().class(cls::MESSAGE_TOOLBAR_BOX);
    if expanded {
        if prefs::developer_mode() {
            let id = target.id.to_string();
            bar = bar.child(button(
                &requiem_i18n::t!("message-toolbar-copy-message-id"),
                glyph(ID),
                Some(Rc::new(move || {
                    menu::copy_with_toast(id.clone(), &requiem_i18n::t!("context-menus-id-copied"))
                })),
            ));
        }
        let link = context_menus::message_link(env, target);
        bar = bar.child(button(
            &requiem_i18n::t!("message-toolbar-copy-link"),
            glyph(LINK),
            Some(Rc::new(move || {
                menu::copy_with_toast(link.clone(), &requiem_i18n::t!("context-menus-link-copied"))
            })),
        ));

        bar = bar.child(button(
            &requiem_i18n::t!("message-toolbar-mark-unread"),
            glyph(CHAT_MARK_UNREAD),
            None,
        ));
        if can_pin {
            let (channel_id, message_id) = (target.channel_id, target.id);
            let pinned = env
                .state
                .messages
                .is_pinned(channel_id, message_id)
                .unwrap_or(false);
            let label = if pinned {
                requiem_i18n::t!("message-toolbar-unpin-message")
            } else {
                requiem_i18n::t!("message-toolbar-pin-message")
            };
            bar = bar.child(button(
                &label,
                glyph(PIN),
                Some(Rc::new(move || {
                    pins::pin_action(
                        channel_id,
                        message_id,
                        pinned,
                        app::shift_held(),
                    )
                })),
            ));
        }
        if own && can_reply {
            let target = target.clone();
            bar = bar.child(button(
                &requiem_i18n::t!("message-toolbar-reply"),
                glyph(ARROW_ANGLE_LEFT_UP),
                Some(Rc::new(move || {
                    context_menus::start_reply(&target)
                })),
            ));
        }
    }

    let can_react = permissions.contains(Permissions::ADD_REACTIONS);
    let quick = if can_react && !expanded {
        reactions::quick_emoji(env, target.guild_id, 3)
    } else {
        Vec::new()
    };
    for emoji in quick {
        let mine = target.my_reactions.iter().any(|e| e.same_as(&emoji));
        let name = reactions::emoji_label(&emoji);
        let label = if mine {
            requiem_i18n::t!("message-toolbar-remove-reaction", name = name.as_str())
        } else {
            requiem_i18n::t!("message-toolbar-add-reaction-named", name = name.as_str())
        };
        let content = reactions::emoji_node(
            &emoji,
            cls::MESSAGE_TOOLBAR_EMOJI,
            &env.images,
            &env.backend,
        );
        let (react_env, channel_id, message_id) = (env.clone(), target.channel_id, target.id);
        let on_click: Rc<dyn Fn()> = Rc::new(move || {
            reactions::react(
                &react_env,
                channel_id,
                message_id,
                &emoji,
                mine,
                endpoints_reactions::LOCATION_HOVER_BAR,
            )
        });
        bar = bar.child(button(&label, content, Some(on_click)));
    }
    if can_react && !expanded {
        bar = bar.child(div().class(cls::MESSAGE_TOOLBAR_DIVIDER));
    }

    let (channel_id, message_id, guild_id) = (target.channel_id, target.id, target.guild_id);
    let add = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::MESSAGE_TOOLBAR_BTN.to_string()])
        .on_click_at(move |rect: AbsoluteRect| {
            reactions::open_picker(channel_id, guild_id, message_id, rect)
        })
        .build()
        .child(glyph(EMOJI_SLIGHTLY_SMILING_FACE));
    if can_react {
        bar = bar.child(tooltip(
            add,
            TooltipProps::default()
                .text(requiem_i18n::t!("message-toolbar-add-reaction").to_string())
                .position(TooltipPosition::Top),
        ));
    }

    if own {
        let (channel_id, message_id, content) =
            (target.channel_id, target.id, target.content.clone());
        bar = bar.child(button(
            &requiem_i18n::t!("message-toolbar-edit"),
            glyph(PENCIL),
            Some(Rc::new(move || {
                message_edit::start(channel_id, message_id, &content)
            })),
        ));
    } else if can_reply {
        let target = target.clone();
        bar = bar.child(button(
            &requiem_i18n::t!("message-toolbar-reply"),
            glyph(ARROW_ANGLE_LEFT_UP),
            Some(Rc::new(move || {
                context_menus::start_reply(&target)
            })),
        ));
    }
    let forward_message = target.message.clone();
    bar = bar.child(button(
        &requiem_i18n::t!("message-toolbar-forward"),
        glyph(ARROW_ANGLE_RIGHT_UP),
        Some(Rc::new(move || forward::open(forward_message.clone()))),
    ));

    if expanded && can_delete {
        let (delete_env, channel_id, message_id) = (env.clone(), target.channel_id, target.id);
        let danger = icon(TRASH, ui::TEXT_SECONDARY)
            .class(cls::MESSAGE_TOOLBAR_ICON)
            .class(cls::MESSAGE_TOOLBAR_ICON_DANGER);
        bar = bar.child(button(
            &requiem_i18n::t!("message-toolbar-delete"),
            danger,
            Some(Rc::new(move || {
                context_menus::delete(
                    &delete_env,
                    channel_id,
                    message_id,
                    app::shift_held(),
                    false,
                )
            })),
        ));
        return div().class(cls::MESSAGE_TOOLBAR).child(bar);
    }

    let (env, target) = (env.clone(), target.clone());
    let more_icon = glyph(MORE_HORIZONTAL);
    let more = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::MESSAGE_TOOLBAR_BTN.to_string()])
        .on_click_at(move |rect: AbsoluteRect| {
            let (env, target) = (env.clone(), target.clone());
            menu::open(rect.x, rect.y + rect.height, move || {
                message_items(&env, &target)
            });
        })
        .build()
        .child(more_icon);
    bar = bar.child(tooltip(
        more,
        TooltipProps::default()
            .text(requiem_i18n::t!("message-toolbar-more").to_string())
            .position(TooltipPosition::Top),
    ));

    div().class(cls::MESSAGE_TOOLBAR).child(bar)
}
