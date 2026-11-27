use requiem_core::ui::context_menu;
use requiem_core::ui::context_menus;
use requiem_core::ui::create_guild::Step;
use requiem_core::ui::create_guild;
use requiem_core::ui::devtools;
use requiem_core::ui::inbox;
use requiem_core::ui::message_keys;
use requiem_core::ui::message_list;
use requiem_core::ui::new_message;
use requiem_core::ui::popouts;
use requiem_core::ui::quick_switcher;
use requiem_core::ui::shortcuts_dialog;
use requiem_core::ui;
use requiem_core::utils::link;
use requiem_ui::event::EventResult;
use requiem_ui::paint;
use std::rc::Rc;

use requiem_core::keybinds::{self, Action};
use requiem_core::models::Id;
use requiem_core::models::id::marker::{ChannelMarker, GuildMarker};
use requiem_core::ui::emoji_picker::PickerTab;
use requiem_core::ui::navigation::{self, Place};
use requiem_core::state::AppState;
use requiem_core::ui::RootSignals;
use requiem_ui::event::{MouseButton, UiEvent};
use requiem_ui::keymap::Chord;
use requiem_ui::widget::WidgetTree;

use crate::features::title_bar;

#[derive(Clone)]
pub struct ShortcutEnv {
    pub state: AppState,
    pub signals: RootSignals,

    pub open_home: Rc<dyn Fn()>,
    pub devtools: devtools::DevtoolsContext,

    pub modal_open: Rc<dyn Fn() -> bool>,
}

pub fn hook(env: ShortcutEnv) -> impl FnMut(&mut WidgetTree, &Chord) -> bool {
    move |tree, chord| {
        let Some(action) = keybinds::action_for(chord) else {
            return false;
        };
        let handled = run(&env, tree, action);
        popouts::reconcile();
        handled
    }
}

fn selected(env: &ShortcutEnv) -> Option<Id<ChannelMarker>> {
    env.signals.selected_channel.peek()
}

fn current_channel(env: &ShortcutEnv) -> Option<Id<ChannelMarker>> {
    ui::displayed_channel(&env.state, selected(env)).map(|c| c.id)
}

fn go_to_guild(env: &ShortcutEnv, guild: Option<Id<GuildMarker>>) {
    match guild {
        None => {
            navigation::note_home();
            (env.open_home)();
        }
        Some(guild_id) => {
            if let Some(channel) = navigation::channel_for_guild(&env.state, guild_id) {
                (env.signals.on_select_channel)(channel);
            }
        }
    }
}

fn select(env: &ShortcutEnv, channel: Option<Id<ChannelMarker>>) {
    if let Some(channel) = channel {
        (env.signals.on_select_channel)(channel);
    }
}

pub fn run(env: &ShortcutEnv, tree: &mut WidgetTree, action: Action) -> bool {
    let anywhere = matches!(
        action,
        Action::ToggleHotkeys
            | Action::QuickSwitcher
            | Action::ToggleMute
            | Action::ToggleDeafen
            | Action::ToggleHelp
            | Action::DevTools
            | Action::SearchEmojis
            | Action::SearchGifs
    );
    if !anywhere && (env.modal_open)() {
        return false;
    }
    let sel = selected(env);
    match action {
        Action::ServerNext | Action::ServerPrev => {
            let step = if action == Action::ServerNext { 1 } else { -1 };
            go_to_guild(env, navigation::step_guild(&env.state, sel, step));
        }
        Action::ChannelNext | Action::ChannelPrev => {
            let step = if action == Action::ChannelNext { 1 } else { -1 };
            select(env, navigation::step_channel(&env.state, sel, step));
        }
        Action::NavigateBack => title_bar::go_back(&env.signals.on_select_channel),
        Action::NavigateForward => title_bar::go_forward(&env.signals.on_select_channel),
        Action::UnreadNext | Action::UnreadPrev => {
            let step = if action == Action::UnreadNext { 1 } else { -1 };
            select(env, navigation::next_unread(&env.state, sel, step));
        }
        Action::MentionNext | Action::MentionPrev => {
            let step = if action == Action::MentionNext { 1 } else { -1 };
            select(env, navigation::next_mention(&env.state, sel, step));
        }
        Action::TogglePreviousGuild => match navigation::place(&env.state, sel) {
            Place::Guild(_) => go_to_guild(env, None),
            Place::Home => {
                if let Some(previous) = navigation::previous_guild() {
                    go_to_guild(env, Some(previous));
                }
            }
        },
        Action::JumpToGuild(n) => match navigation::guild_slot(&env.state, usize::from(n)) {
            Ok(guild) => go_to_guild(env, guild),
            Err(()) => return false,
        },
        Action::QuickSwitcher => {
            if quick_switcher::is_open() {
                quick_switcher::close();
            } else {
                quick_switcher::open();
            }
        }
        Action::CreateGuild => {
            create_guild::open(if create_guild::is_open() {
                Step::Join
            } else {
                Step::Templates
            })
        }
        Action::CreateDmGroup => {
            if !matches!(navigation::place(&env.state, sel), Place::Home) {
                go_to_guild(env, None);
            }
            new_message::open();
        }
        Action::MarkChannelRead => {
            if let Some(channel) = current_channel(env) {
                env.state.read_states.mark_channel_read(channel);
            }
        }
        Action::MarkServerRead => match navigation::place(&env.state, sel) {
            Place::Guild(guild) => env.state.read_states.mark_guild_read(guild),
            Place::Home => {
                for channel in navigation::dm_channels(&env.state) {
                    env.state.read_states.mark_channel_read(channel);
                }
            }
        },
        Action::ToggleInbox => {
            let Some(anchor) = paint::rect_of_dom_id(
                tree,
                title_bar::INBOX_BUTTON_DOM_ID,
            ) else {
                return false;
            };
            inbox::toggle(anchor);
        }
        Action::ToggleUsers => env.signals.member_list.open.update(|open| *open = !*open),
        Action::SearchEmojis | Action::SearchGifs => {
            if current_channel(env).is_none() {
                return false;
            }
            let tab = if action == Action::SearchEmojis {
                PickerTab::Emoji
            } else {
                PickerTab::Gif
            };
            let picker = &env.signals.emoji_picker;
            if picker.open.peek() && picker.tab.peek() == tab {
                picker.open.set(false);
            } else {
                picker.tab.set(tab);
                picker.open.set(true);
            }
        }
        Action::ScrollUp | Action::ScrollDown => {
            if !scroll_messages(tree, action == Action::ScrollUp) {
                return false;
            }
        }
        Action::JumpToFirstUnread => {
            let Some(channel) = current_channel(env) else {
                return false;
            };
            let Some(first) = env
                .state
                .read_states
                .new_marker(channel)
                .and_then(|m| m.first_unread)
            else {
                return false;
            };
            message_list::request_jump(channel, Id::new(first));
        }
        Action::UploadFile => {
            let Some(channel) = current_channel(env) else {
                return false;
            };
            if !ui::can_attach_files(&env.state, sel) {
                return false;
            }
            (env.signals.attachments.backend.pick_files)(channel);
        }
        Action::ToggleMute => (env.signals.on_toggle_mute)(),
        Action::ToggleDeafen => (env.signals.on_toggle_deafen)(),
        Action::FocusSearch { prefill_channel } => {
            let Some(channel) = ui::displayed_channel(&env.state, sel) else {
                return false;
            };

            if prefill_channel && channel.guild_id.is_some() && !channel.is_obfuscated() {
                if let Some(name) = &channel.name {
                    env.signals.search.set_text(format!("in: {name} "));
                }
            }
            env.signals.search.popout_open.set(true);
        }
        Action::ToggleHotkeys => shortcuts_dialog::toggle(),
        Action::ToggleHelp => link::open_url("https://support.discord.com"),
        Action::CopyChannelLink => {
            let Some((guild, channel)) = current_channel(env).map(|id| {
                (
                    ui::find_channel(&env.state, id).map(|(g, _)| g),
                    id,
                )
            }) else {
                return false;
            };
            context_menu::copy_with_toast(
                context_menus::channel_link(guild, channel),
                &requiem_i18n::t!("context-menus-link-copied"),
            );
        }
        Action::OpenContextMenu => return open_context_menu(tree),
        Action::DevTools => devtools::toggle(&env.devtools),
    }
    true
}

fn scroll_messages(tree: &mut WidgetTree, up: bool) -> bool {
    let Some(list) = message_keys::list_id(tree) else {
        return false;
    };
    let rects = paint::all_absolute_rects(tree);
    let Some(rect) = rects.get(&list).copied() else {
        return false;
    };
    let page = (rect.height * 0.9).max(40.0);
    paint::dispatch_event(
        tree,
        &UiEvent::Scroll {
            x: rect.x + rect.width / 2.0,
            y: rect.y + rect.height / 2.0,
            delta_x: 0.0,
            delta_y: if up { page } else { -page },
            precise: true,
        },
    );
    true
}

fn open_context_menu(tree: &mut WidgetTree) -> bool {
    let Some(focused) = tree.focused else {
        return false;
    };
    let Some(rect) = paint::all_absolute_rects(tree)
        .get(&focused)
        .copied()
    else {
        return false;
    };
    let (x, y) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    let result = paint::dispatch_event(
        tree,
        &UiEvent::MouseDown {
            x,
            y,
            button: MouseButton::Right,
            shift: false,
        },
    );
    paint::dispatch_event(
        tree,
        &UiEvent::MouseUp {
            x,
            y,
            button: MouseButton::Right,
        },
    );
    matches!(
        result,
        EventResult::Handled
    )
}
