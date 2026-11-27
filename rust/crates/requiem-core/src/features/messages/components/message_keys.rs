use crate::ui::classes::message_list;
use crate::ui::context_menu;
use crate::ui::context_menus;
use crate::ui::message_edit;
use crate::ui::message_list as ui_message_list;
use crate::ui::reactions;
use requiem_ui::app;
use requiem_ui::paint;
use std::cell::RefCell;

use requiem_ui::widget::{WidgetId, WidgetTree};

use crate::models::id::marker::{ChannelMarker, MessageMarker};
use crate::models::message::message_type;
use crate::models::{Channel, Id, Message, Permissions};
use crate::state::AppState;
use crate::ui::context_menus::MenuEnv;

thread_local! {

    static ENV: RefCell<Option<MenuEnv>> = const { RefCell::new(None) };
}

pub fn set_env(env: MenuEnv) {
    ENV.with(|e| *e.borrow_mut() = Some(env));
}

fn env() -> Option<MenuEnv> {
    ENV.with(|e| e.borrow().clone())
}

pub fn list_id(tree: &WidgetTree) -> Option<WidgetId> {
    let class = message_list::MESSAGE_LIST;
    tree.nodes
        .iter()
        .find(|(_, node)| node.classes.iter().any(|c| c == class))
        .map(|(id, _)| id)
}

fn message_of(tree: &WidgetTree, id: WidgetId) -> Option<Id<MessageMarker>> {
    let node = tree.nodes.get(id)?;
    if !node
        .classes
        .iter()
        .any(|c| c == message_list::MESSAGE_ROW)
    {
        return None;
    }
    node.dom_id.as_deref()?.parse::<u64>().ok().map(Id::new)
}

fn rows(tree: &WidgetTree) -> Vec<(WidgetId, Id<MessageMarker>)> {
    fn walk(tree: &WidgetTree, id: WidgetId, out: &mut Vec<(WidgetId, Id<MessageMarker>)>) {
        if let Some(message) = message_of(tree, id) {
            out.push((id, message));
        }
        for &child in &tree.nodes[id].children {
            walk(tree, child, out);
        }
    }

    let mut out = Vec::new();
    if let Some(list) = list_id(tree) {
        walk(tree, list, &mut out);
    }
    out
}

fn focused_row(tree: &WidgetTree) -> Option<(WidgetId, Id<MessageMarker>)> {
    let mut current = tree.focused;
    while let Some(id) = current {
        if let Some(message) = message_of(tree, id) {
            return Some((id, message));
        }
        current = tree.nodes.get(id).and_then(|n| n.parent);
    }
    None
}

pub fn focused_message(tree: &WidgetTree) -> Option<Id<MessageMarker>> {
    focused_row(tree).map(|(_, message)| message)
}

pub fn armed(tree: &WidgetTree) -> bool {
    tree.focus_visible && focused_message(tree).is_some()
}

pub fn step(tree: &mut WidgetTree, step: isize) -> bool {
    let all = rows(tree);
    let Some((row, _)) = focused_row(tree) else {
        return false;
    };
    let Some(current) = all.iter().position(|(id, _)| *id == row) else {
        return false;
    };
    let target = (current as isize + step).clamp(0, all.len() as isize - 1) as usize;
    paint::focus_widget(tree, all[target].0);
    true
}

pub fn focus_last(tree: &mut WidgetTree) -> bool {
    let Some(&(id, _)) = rows(tree).last() else {
        return false;
    };
    paint::focus_widget(tree, id);
    true
}

pub fn focus_end(tree: &mut WidgetTree, first: bool) -> bool {
    let all = rows(tree);
    let Some(&(id, _)) = (if first { all.first() } else { all.last() }) else {
        return false;
    };
    paint::focus_widget(tree, id);
    true
}

fn message(state: &AppState, channel: Id<ChannelMarker>, id: Id<MessageMarker>) -> Option<Message> {
    state.messages.by_channel.with_untracked(|by_channel| {
        by_channel
            .get(&channel)?
            .iter()
            .find(|m| m.id == id)
            .cloned()
    })
}

fn editable(message: &Message) -> bool {
    matches!(message.kind, message_type::DEFAULT | message_type::REPLY)
}

pub fn text_key(tree: &WidgetTree, state: &AppState, channel: &Channel, text: &str) -> bool {
    if !armed(tree) {
        return false;
    }
    let Some(env) = env() else { return false };
    let Some(id) = focused_message(tree) else {
        return false;
    };
    let Some(message) = message(state, channel.id, id) else {
        return false;
    };
    let own = env.own_user_id == Some(message.author.id);
    let permissions = state.channel_permissions(channel);
    let target = ui_message_list::message_target(&message, channel);
    match text {
        "e" | "E" if own && editable(&message) => {
            message_edit::start(channel.id, id, &message.content)
        }
        "r" | "R" if permissions.contains(Permissions::SEND_MESSAGES) && editable(&message) => {
            context_menus::start_reply(&target)
        }
        "+" | "=" if permissions.contains(Permissions::ADD_REACTIONS) => {
            let Some((row, _)) = focused_row(tree) else {
                return false;
            };
            let rect = paint::rect_of(tree, row);
            reactions::open_picker(channel.id, channel.guild_id, id, rect);
        }
        _ => return false,
    }
    true
}

pub fn delete_focused(tree: &WidgetTree, state: &AppState, channel: &Channel) -> bool {
    if !armed(tree) {
        return false;
    }
    let (Some(env), Some(id)) = (env(), focused_message(tree)) else {
        return false;
    };
    let Some(message) = message(state, channel.id, id) else {
        return false;
    };
    let own = env.own_user_id == Some(message.author.id);
    if !(own
        || state
            .channel_permissions(channel)
            .contains(Permissions::MANAGE_MESSAGES))
    {
        return false;
    }

    context_menus::delete(&env, channel.id, id, app::shift_held(), false);
    true
}

pub fn copy_focused(tree: &WidgetTree, state: &AppState, channel: &Channel) -> bool {
    let Some(id) = focused_message(tree) else {
        return false;
    };
    let Some(message) = message(state, channel.id, id).filter(|m| !m.content.is_empty()) else {
        return false;
    };
    context_menu::copy_with_toast(
        message.content,
        &requiem_i18n::t!("context-menus-text-copied"),
    );
    true
}

pub fn edit_last_own(state: &AppState, channel: Id<ChannelMarker>) -> bool {
    let Some(me) = state
        .users
        .current
        .with_untracked(|u| u.as_ref().map(|u| u.id))
    else {
        return false;
    };
    let last = state.messages.by_channel.with_untracked(|by_channel| {
        by_channel
            .get(&channel)?
            .iter()
            .rev()
            .find(|m| m.author.id == me && editable(m))
            .cloned()
    });
    let Some(last) = last else { return false };
    message_edit::start(channel, last.id, &last.content);
    true
}
