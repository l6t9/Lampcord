use crate::models::Permissions;
use crate::ui::author_style::NameFill;
use crate::ui::author_style;
use crate::ui::built_in_commands;
use crate::ui::classes::command_composer;
use crate::ui::classes::command_composer as cls;
use crate::ui::classes::message_list as classes_message_list;
use crate::ui::commands_panel;
use crate::ui::context_menu;
use crate::ui::emoji_picker;
use crate::ui::message_list;
use crate::ui;
use requiem_ui::clipboard;
use requiem_ui::colors;
use requiem_ui::components::AvatarProps;
use requiem_ui::widget::primitives::AvatarShape;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use requiem_ui::colors::{TEXT_DEFAULT, TEXT_STRONG, TEXT_SUBTLE};
use requiem_ui::components::{
    ButtonProps, ButtonVariant, TextInputProps, TextProps, TextVariant, div, icon,
};
use requiem_ui::event::{CursorIcon, NamedKey};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::AbsoluteRect;
use requiem_ui::widget::primitives::{AutocompleteCandidate, AutocompleteTrigger, ClickAreaWidget};
use serde_json::{Value, json};

use crate::models::application_command::{CommandApp, CommandEntry, CommandOption, option_type};
use crate::models::id::marker::ChannelMarker;
use crate::models::{Channel, Id};
use crate::ui::Fonts;
use crate::ui::context_menus::MenuEnv;
use crate::ui::slash_commands::{self, Invocation, ParseError};
use requiem_ui::icons::{ANGLE_BRACKETS, CIRCLE_X};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MentionKind {
    User,
    Channel,
    Role,
}

#[derive(Clone, Debug)]
pub struct Filled {
    pub display: String,
    pub json: Value,
    pub mention: Option<MentionKind>,
}

#[derive(Clone, Debug)]
pub struct Slot {
    pub option: CommandOption,

    pub added: bool,
    pub value: Option<Filled>,

    pub draft: Option<String>,
}

impl Slot {
    fn is_chip(&self) -> bool {
        self.value.as_ref().is_some_and(|v| v.mention.is_some())
            || !self.option.choices.is_empty()
            || self.option.kind == option_type::BOOLEAN
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Slot(usize),

    Tail,
}

#[derive(Clone)]
pub struct Active {
    pub entry: CommandEntry,
    pub app: Option<CommandApp>,
    pub slots: Vec<Slot>,
    pub focus: Focus,

    pub all_selected: bool,
}

impl Active {
    fn first_unfilled_required(&self) -> Option<usize> {
        self.slots
            .iter()
            .position(|s| s.option.required && s.value.is_none())
    }

    fn advance(&mut self) {
        self.focus = self
            .first_unfilled_required()
            .map_or(Focus::Tail, Focus::Slot);
    }

    fn shown(&self) -> impl Iterator<Item = usize> + '_ {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.added)
            .map(|(i, _)| i)
    }

    fn hidden_count(&self) -> usize {
        self.slots.iter().filter(|s| !s.added).count()
    }
}

thread_local! {

    static RESEED: Signal<u64> = signal(0);
    static ACTIVE: Signal<HashMap<Id<ChannelMarker>, Active>> = signal(HashMap::new());

    static LIVE_TEXT: RefCell<String> = const { RefCell::new(String::new()) };
    static HISTORY: RefCell<HashMap<Id<ChannelMarker>, History>> = RefCell::new(HashMap::new());
}

#[derive(Default)]
struct History {
    undo: Vec<Option<Active>>,
    redo: Vec<Option<Active>>,

    seed: Option<String>,
}

type Structure = Vec<(String, bool, Option<(String, String)>)>;

fn structure(active: &Active) -> Structure {
    let mut parts: Structure = vec![(active.entry.display(), true, None)];
    parts.extend(active.slots.iter().map(|s| {
        (
            s.option.name.clone(),
            s.added,
            s.value
                .as_ref()
                .map(|v| (v.display.clone(), v.json.to_string())),
        )
    }));
    parts
}

fn remember(channel: Id<ChannelMarker>, before: Option<Active>) {
    HISTORY.with(|h| {
        let mut all = h.borrow_mut();
        let history = all.entry(channel).or_default();
        history.undo.push(before);
        history.redo.clear();
        history.undo.truncate(200);
    });
}

fn restore(env: &MenuEnv, channel: Id<ChannelMarker>, snapshot: Option<Active>) {
    ACTIVE.with(|all| {
        all.update(|all| match snapshot {
            Some(mut active) => {
                active.all_selected = false;
                all.insert(channel, active);
            }
            None => {
                all.remove(&channel);
            }
        })
    });
    env.state.drafts.clear(channel);
    commands_panel::set_text(channel, "");
    reseed();
}

pub fn undo(env: &MenuEnv, channel: Id<ChannelMarker>) -> bool {
    let Some(snapshot) = HISTORY.with(|h| {
        h.borrow_mut()
            .get_mut(&channel)
            .and_then(|history| history.undo.pop())
    }) else {
        return false;
    };
    let current = peek(channel);
    HISTORY.with(|h| {
        h.borrow_mut()
            .entry(channel)
            .or_default()
            .redo
            .push(current)
    });
    restore(env, channel, snapshot);
    true
}

pub fn redo(env: &MenuEnv, channel: Id<ChannelMarker>) -> bool {
    let Some(snapshot) = HISTORY.with(|h| {
        h.borrow_mut()
            .get_mut(&channel)
            .and_then(|history| history.redo.pop())
    }) else {
        return false;
    };
    let current = peek(channel);
    HISTORY.with(|h| {
        h.borrow_mut()
            .entry(channel)
            .or_default()
            .undo
            .push(current)
    });
    restore(env, channel, snapshot);
    true
}

pub fn plain_undo(env: &MenuEnv, channel: Id<ChannelMarker>, text: &str) -> bool {
    let brings_back = peek(channel).is_none()
        && HISTORY.with(|h| {
            h.borrow().get(&channel).is_some_and(|history| {
                matches!(history.undo.last(), Some(Some(_)))
                    && history.seed.as_deref().unwrap_or("") == text
            })
        });
    brings_back && undo(env, channel)
}

pub fn plain_redo(env: &MenuEnv, channel: Id<ChannelMarker>, text: &str) -> bool {
    let applies = peek(channel).is_none()
        && text.is_empty()
        && HISTORY.with(|h| {
            h.borrow()
                .get(&channel)
                .is_some_and(|history| matches!(history.redo.last(), Some(Some(_))))
        });
    applies && redo(env, channel)
}

pub fn reseed_tick() -> u64 {
    RESEED.with(Signal::get)
}

pub fn reseed() {
    RESEED.with(|r| r.update(|n| *n += 1));
}

pub fn active(channel: Id<ChannelMarker>) -> Option<Active> {
    ACTIVE.with(|all| all.with(|all| all.get(&channel).cloned()))
}

fn peek(channel: Id<ChannelMarker>) -> Option<Active> {
    ACTIVE.with(|all| all.with_untracked(|all| all.get(&channel).cloned()))
}

fn update(channel: Id<ChannelMarker>, change: impl FnOnce(&mut Active)) {
    let before = peek(channel);
    ACTIVE.with(|all| {
        all.update(|all| {
            if let Some(active) = all.get_mut(&channel) {
                change(active);
            }
        })
    });

    if let (Some(before), Some(after)) = (before, peek(channel)) {
        if structure(&before) != structure(&after) {
            remember(channel, Some(before));
        }
    }
}

pub fn begin(channel: Id<ChannelMarker>, entry: CommandEntry, app: Option<CommandApp>) {
    let slots = entry
        .options
        .iter()
        .filter(|o| !slash_commands::is_structural(o))
        .map(|o| Slot {
            option: o.clone(),
            added: o.required,
            value: None,
            draft: None,
        })
        .collect();
    let mut active = Active {
        entry,
        app,
        slots,
        focus: Focus::Tail,
        all_selected: false,
    };
    active.advance();
    remember(channel, peek(channel));
    ACTIVE.with(|all| {
        all.update(|all| {
            all.insert(channel, active);
        })
    });
}

pub fn cancel(channel: Id<ChannelMarker>) {
    if let Some(active) = peek(channel) {
        remember(channel, Some(active));
    }
    HISTORY.with(|h| h.borrow_mut().entry(channel).or_default().seed = None);
    ACTIVE.with(|all| {
        all.update(|all| {
            all.remove(&channel);
        })
    });
}

fn finish(channel: Id<ChannelMarker>) {
    HISTORY.with(|h| {
        h.borrow_mut().remove(&channel);
    });
    ACTIVE.with(|all| {
        all.update(|all| {
            all.remove(&channel);
        })
    });
}

fn seed_plain(channel: Id<ChannelMarker>, text: &str) {
    HISTORY.with(|h| h.borrow_mut().entry(channel).or_default().seed = Some(text.to_string()));
}

fn keep_live_text(channel: Id<ChannelMarker>) {
    let Some(active) = peek(channel) else { return };
    let Focus::Slot(index) = active.focus else {
        return;
    };
    let text = LIVE_TEXT.with(|t| mem::take(&mut *t.borrow_mut()));
    let slot = &active.slots[index];
    let typed = slot.option.choices.is_empty()
        && matches!(
            slot.option.kind,
            option_type::STRING | option_type::INTEGER | option_type::NUMBER
        );
    let text = text.trim();
    if !typed || slot.is_chip() || text.is_empty() {
        return;
    }
    match slash_commands::coerce(&slot.option, text) {
        Ok(json) => update(channel, |a| {
            a.slots[index].value = Some(Filled {
                display: text.to_string(),
                json,
                mention: None,
            });
            a.slots[index].draft = None;
        }),
        Err(_) => update(channel, |a| {
            a.slots[index].value = None;
            a.slots[index].draft = Some(text.to_string());
        }),
    }
}

fn focus(channel: Id<ChannelMarker>, focus: Focus) {
    keep_live_text(channel);
    update(channel, |a| {
        a.focus = focus;
        a.all_selected = false;
    });
}

const SELECT_ALL_WINDOW: Duration = Duration::from_millis(500);

thread_local! {
    static LAST_SELECT_ALL: Cell<Option<Instant>> = const { Cell::new(None) };
}

fn line_text(active: &Active) -> String {
    let mut text = format!("/{}", active.entry.display());
    for index in active.shown() {
        let slot = &active.slots[index];
        let Some(value) = &slot.value else { continue };
        let shown = match value.mention {
            Some(MentionKind::User) => format!("<@{}>", value.json.as_str().unwrap_or_default()),
            Some(MentionKind::Role) => format!("<@&{}>", value.json.as_str().unwrap_or_default()),
            Some(MentionKind::Channel) => format!("<#{}>", value.json.as_str().unwrap_or_default()),
            None => value.display.clone(),
        };
        text.push_str(&format!(" {}:{shown}", slot.option.name));
    }
    text
}

fn select_all_pressed(channel: Id<ChannelMarker>, field_text: &str) -> bool {
    let again = LAST_SELECT_ALL
        .with(|last| last.get())
        .is_some_and(|at| at.elapsed() < SELECT_ALL_WINDOW);
    if !again && !field_text.is_empty() {
        LAST_SELECT_ALL.with(|last| last.set(Some(Instant::now())));
        return false;
    }
    LAST_SELECT_ALL.with(|last| last.set(None));
    keep_live_text(channel);
    update(channel, |a| {
        a.all_selected = true;
        a.focus = Focus::Tail;
    });
    true
}

fn all_selected(channel: Id<ChannelMarker>) -> bool {
    peek(channel).is_some_and(|a| a.all_selected)
}

fn replace_line_with(env: &MenuEnv, channel: Id<ChannelMarker>, text: &str) {
    cancel(channel);
    seed_plain(channel, text);
    env.state.drafts.set(channel, text);
    commands_panel::set_text(channel, text);
    reseed();
}

fn key_in_selection(env: &MenuEnv, channel: Id<ChannelMarker>, key: NamedKey) -> bool {
    let Some(active) = peek(channel) else {
        return false;
    };
    match key {
        NamedKey::Copy => {
            clipboard::set_text(line_text(&active));
            true
        }
        NamedKey::Cut => {
            clipboard::set_text(line_text(&active));
            cancel(channel);
            env.state.drafts.clear(channel);
            commands_panel::set_text(channel, "");
            reseed();
            true
        }
        NamedKey::Backspace | NamedKey::Delete => {
            cancel(channel);
            env.state.drafts.clear(channel);
            commands_panel::set_text(channel, "");
            reseed();
            true
        }

        NamedKey::ArrowLeft | NamedKey::Home => {
            let first = active.shown().next();
            focus(channel, first.map_or(Focus::Tail, Focus::Slot));
            true
        }
        NamedKey::ArrowRight | NamedKey::End => {
            focus(channel, Focus::Tail);
            true
        }
        NamedKey::Escape => {
            focus(channel, Focus::Tail);
            true
        }
        NamedKey::SelectAll => true,
        NamedKey::Undo => undo(env, channel),
        NamedKey::Redo => redo(env, channel),
        _ => false,
    }
}

fn focus_end(channel: Id<ChannelMarker>) {
    let Some(active) = peek(channel) else { return };
    let target = active
        .shown()
        .last()
        .filter(|i| !active.slots[*i].is_chip())
        .map_or(Focus::Tail, Focus::Slot);
    focus(channel, target);
}

fn pick(channel: Id<ChannelMarker>, index: usize, filled: Filled) {
    update(channel, |a| {
        a.slots[index].value = Some(filled);
        a.slots[index].draft = None;
        a.advance();
    });
}

fn add_option(channel: Id<ChannelMarker>, index: usize) {
    keep_live_text(channel);
    update(channel, |a| {
        a.slots[index].added = true;
        a.focus = Focus::Slot(index);
    });
}

fn commit_text(channel: Id<ChannelMarker>, index: usize, text: &str) -> bool {
    let Some(active) = peek(channel) else {
        return false;
    };
    let slot = &active.slots[index];
    let text = text.trim();
    if text.is_empty() {
        return slot.value.is_some();
    }

    let typed = slot.option.choices.is_empty()
        && matches!(
            slot.option.kind,
            option_type::STRING | option_type::INTEGER | option_type::NUMBER
        );
    if !typed {
        return slot.value.is_some();
    }
    match slash_commands::coerce(&slot.option, text) {
        Ok(json) => {
            pick(
                channel,
                index,
                Filled {
                    display: text.to_string(),
                    json,
                    mention: None,
                },
            );
            true
        }
        Err(ParseError::Invalid { option, expected }) => {
            context_menu::notify_failure(&requiem_i18n::t!(
                "slash-commands-invalid-option",
                option = option,
                expected = expected
            ));
            false
        }
        Err(_) => false,
    }
}

fn send(env: &MenuEnv, channel: &Channel) {
    let Some(active) = peek(channel.id) else {
        return;
    };
    if let Some(missing) = active.first_unfilled_required() {
        context_menu::notify_failure(&requiem_i18n::t!(
            "slash-commands-option-required",
            option = active.slots[missing].option.name.clone()
        ));
        focus(channel.id, Focus::Slot(missing));
        return;
    }
    let values = active
        .slots
        .iter()
        .filter_map(|s| {
            s.value
                .as_ref()
                .map(|v| (s.option.kind, s.option.name.clone(), v.json.clone()))
        })
        .collect();
    let invocation = Invocation {
        entry: active.entry,
        values,
    };
    finish(channel.id);
    slash_commands::dispatch(env, channel, &invocation);
}

fn backspace_in_slot(channel: Id<ChannelMarker>, index: usize) {
    let Some(active) = peek(channel) else { return };
    if active.slots[index].is_chip() {
        update(channel, |a| a.slots[index].value = None);
    } else {
        update(channel, |a| {
            a.slots[index].added = false;
            a.slots[index].value = None;
            a.slots[index].draft = None;
            a.focus = Focus::Tail;
        });
    }
}

fn backspace_at_tail(env: &MenuEnv, channel: &Channel) {
    let Some(active) = peek(channel.id) else {
        return;
    };
    let Some(last) = active.shown().last() else {
        let text = format!("/{}", active.entry.display());
        cancel(channel.id);
        seed_plain(channel.id, &text);
        env.state.drafts.set(channel.id, &text);
        commands_panel::set_text(channel.id, &text);
        reseed();
        return;
    };
    let slot = &active.slots[last];
    match (&slot.value, slot.is_chip()) {
        (Some(_), true) => update(channel.id, |a| {
            a.slots[last].value = None;
            a.focus = Focus::Slot(last);
        }),
        (Some(value), false) => {
            let mut text = value.display.clone();
            text.pop();
            update(channel.id, |a| {
                a.slots[last].value = None;
                a.slots[last].draft = Some(text);
                a.focus = Focus::Slot(last);
            });
        }
        (None, _) => update(channel.id, |a| {
            a.slots[last].added = false;
            a.slots[last].draft = None;
            a.focus = Focus::Tail;
        }),
    }
}

fn step_sideways(channel: Id<ChannelMarker>, from: Option<usize>, left: bool) {
    let Some(active) = peek(channel) else { return };
    let shown: Vec<usize> = active.shown().collect();
    let target = match (from, left) {
        (Some(i), true) => shown
            .iter()
            .rev()
            .find(|s| **s < i)
            .map(|s| Focus::Slot(*s)),
        (Some(i), false) => Some(
            shown
                .iter()
                .find(|s| **s > i)
                .map_or(Focus::Tail, |s| Focus::Slot(*s)),
        ),
        (None, true) => shown.last().map(|s| Focus::Slot(*s)),
        (None, false) => None,
    };
    if let Some(target) = target {
        focus(channel, target);
    }
}

pub struct Ctx<'a> {
    pub env: &'a MenuEnv,
    pub channel: &'a Channel,
    pub fonts: &'a Fonts,
    pub colors: &'a message_list::ChatColors,

    pub custom_emoji: &'a emoji_picker::CustomEmojiSource,

    pub frame: Signal<AbsoluteRect>,
}

pub fn bar(ctx: &Ctx<'_>, active: &Active) -> Node {
    let (title, description) = match active.focus {
        Focus::Slot(i) => (
            active.slots[i].option.name.clone(),
            active.slots[i].option.description.clone(),
        ),
        Focus::Tail => (
            format!("/{}", active.entry.display()),
            active.entry.description.clone(),
        ),
    };
    let channel_id = ctx.channel.id;
    let close = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::CMD_BAR_CLOSE.to_string()])
        .on_click(move || cancel(channel_id))
        .build()
        .child(icon(CIRCLE_X, ui::TEXT_SECONDARY).class(cls::CMD_BAR_CLOSE_ICON));
    div().class(cls::CMD_BAR).children([
        div().class(cls::CMD_BAR_INFO).children([
            TextProps::styled(title, TextVariant::TEXT_MD_SEMIBOLD)
                .line_height(20.0_f32)
                .color(TEXT_STRONG)
                .line_clamp(1_u32)
                .build(),
            TextProps::styled(description, TextVariant::TEXT_SM_MEDIUM)
                .line_height(16.0_f32)
                .color(TEXT_SUBTLE)
                .line_clamp(1_u32)
                .build(),
        ]),
        close,
    ])
}

pub fn line(ctx: &Ctx<'_>, active: &Active) -> Node {
    let channel_id = ctx.channel.id;

    let app_icon = if built_in_commands::is_built_in(&active.entry) {
        icon(ANGLE_BRACKETS, ui::TEXT_SECONDARY).class(cls::CMD_APP_ICON)
    } else {
        let mut icon_props = AvatarProps::default()
            .fallback_background(colors::BRAND)
            .shape(AvatarShape::Circle)
            .typeface(ctx.fonts.typeface.clone());
        if let Some(image) = active
            .app
            .as_ref()
            .and_then(|app| {
                Some(
                    ctx.env
                        .backend
                        .app_icon_url(&app.id.to_string(), app.icon.as_deref()?),
                )
            })
            .and_then(|url| ctx.env.images.get_or_decode_image(&url))
        {
            icon_props = icon_props.image(image);
        }
        icon_props.build().class(cls::CMD_APP_ICON)
    };
    let app_slot = div().class(cls::CMD_APP_SLOT).child(app_icon);

    let selected = active.all_selected;
    let mut content = Node::new(
        "div",
        ClickAreaWidget::new()
            .with_cursor(CursorIcon::Text)
            .with_on_click(move || focus_end(channel_id)),
    )
    .class(cls::CMD_CONTENT_CLICK);
    if selected {
        content = content.class(cls::CMD_CONTENT_SELECTED);
    }
    let name = TextProps::styled(
        format!("/{}", active.entry.display()),
        TextVariant::TEXT_MD_SEMIBOLD,
    )
    .line_height(22.0_f32)
    .color(TEXT_STRONG)
    .build();
    content = content.child(if selected {
        div()
            .class(cls::CMD_SELECTED_PART)
            .class(cls::CMD_SELECTED_FIRST)
            .child(name)
    } else {
        name.class(cls::CMD_NAME)
    });
    for index in active.shown() {
        content = content.child(if active.focus == Focus::Slot(index) {
            live_pill(ctx, active, index)
        } else {
            pill(ctx, active, index)
        });
    }
    if active.focus == Focus::Tail {
        let tail = div().class(cls::CMD_TAIL).child(tail_input(ctx, active));
        content = content.child(if selected {
            tail.class(cls::CMD_SELECTED_PART)
                .class(cls::CMD_SELECTED_LAST)
        } else {
            tail
        });
    }
    let hidden = active.hidden_count();
    if hidden > 0 {
        let more = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::CMD_MORE.to_string()])
            .on_click(move || focus(channel_id, Focus::Tail))
            .build()
            .child(
                TextProps::styled(
                    if active.shown().next().is_none() {
                        requiem_i18n::t!("slash-commands-more-options", count = hidden).to_string()
                    } else {
                        requiem_i18n::t!("slash-commands-more", count = hidden).to_string()
                    },
                    TextVariant::TEXT_MD_NORMAL,
                )
                .line_height(22.0_f32)
                .color(ui::TEXT_SECONDARY)
                .build(),
            );
        content = content.child(more);
    }
    div()
        .class(classes_message_list::COMPOSER_LINE)
        .class(classes_message_list::COMPOSER_LINE_SANS_ATTACH)
        .children([app_slot, content])
}

fn key_box(name: &str) -> Node {
    div().class(cls::CMD_PILL_KEY).child(
        TextProps::styled(name.to_string(), TextVariant::TEXT_MD_NORMAL)
            .line_height(22.0_f32)
            .color(TEXT_DEFAULT)
            .build(),
    )
}

fn value_node(value: &Filled) -> Node {
    match value.mention {
        Some(kind) => {
            let sigil = if kind == MentionKind::Channel {
                '#'
            } else {
                '@'
            };
            div().class(cls::CMD_MENTION).child(
                TextProps::styled(
                    format!("{sigil}{}", value.display),
                    TextVariant::TEXT_MD_NORMAL,
                )
                .line_height(20.0_f32)
                .color(colors::TEXT_MENTION)
                .build(),
            )
        }
        None => TextProps::styled(value.display.clone(), TextVariant::TEXT_MD_NORMAL)
            .line_height(22.0_f32)
            .color(TEXT_DEFAULT)
            .build(),
    }
}

fn pill_frame(channel_id: Id<ChannelMarker>, index: usize, pill: Node) -> Node {
    Node::new(
        "div",
        ClickAreaWidget::new()
            .with_cursor(CursorIcon::Text)
            .with_on_click(move || focus(channel_id, Focus::Slot(index))),
    )
    .child(pill)
}

fn pill(ctx: &Ctx<'_>, active: &Active, index: usize) -> Node {
    let slot = &active.slots[index];
    let mut inner = div().class(cls::CMD_PILL).child(key_box(&slot.option.name));
    if slot.option.required && slot.value.is_none() {
        inner = inner.class(cls::CMD_PILL_ERRORED);
    }

    let value = match (&slot.value, &slot.draft) {
        (Some(value), _) => Some(value_node(value)),
        (None, Some(draft)) => Some(
            TextProps::styled(draft.clone(), TextVariant::TEXT_MD_NORMAL)
                .line_height(22.0_f32)
                .color(TEXT_DEFAULT)
                .build(),
        ),
        (None, None) => None,
    };
    inner = inner.child(div().class(cls::CMD_PILL_VALUE).children(value));
    let frame = pill_frame(ctx.channel.id, index, inner);
    if active.all_selected {
        frame.class(command_composer::CMD_SELECTED_PART)
    } else {
        frame
    }
}

fn live_pill(ctx: &Ctx<'_>, active: &Active, index: usize) -> Node {
    let slot = &active.slots[index];
    let (env, channel) = (ctx.env.clone(), ctx.channel.clone());
    let channel_id = channel.id;
    let chip = slot.is_chip();
    let initial = if chip {
        String::new()
    } else {
        slot.draft
            .clone()
            .or_else(|| slot.value.as_ref().map(|v| v.display.clone()))
            .unwrap_or_default()
    };
    LIVE_TEXT.with(|t| *t.borrow_mut() = initial.clone());
    let initial_text = initial.clone();

    let free_text = slot.option.kind == option_type::STRING && slot.option.choices.is_empty();
    let (base, triggers) = if free_text {
        (
            message_list::chat_input_with(field(ctx, initial), Some(ctx.custom_emoji)),
            message_list::autocomplete_triggers(
                &ctx.env.state,
                ctx.channel,
                ctx.colors,
                ctx.env,
            ),
        )
    } else {
        (field(ctx, initial), value_triggers(ctx, active, index))
    };
    let (submit_env, submit_channel) = (env.clone(), channel.clone());
    let input = base
        .autocomplete(triggers)
        .on_change(|text: &str| LIVE_TEXT.with(|t| *t.borrow_mut() = text.to_string()))
        .key_hook({
            let (history_env, unedited) = (env.clone(), initial_text);
            move |key: NamedKey, _shift: bool, text: &str| match key {
                NamedKey::SelectAll => select_all_pressed(channel_id, text),

                NamedKey::Undo => text == unedited && undo(&history_env, channel_id),
                NamedKey::Redo => text == unedited && redo(&history_env, channel_id),
                _ => false,
            }
        })
        .on_submit(move |text: String| {
            commit_text(channel_id, index, &text);
            send(&submit_env, &submit_channel);
        })
        .edge_hook(move |key: NamedKey, _shift: bool, text: &str| match key {
            NamedKey::Tab => {
                if commit_text(channel_id, index, text) || text.trim().is_empty() {
                    update(channel_id, |a| a.advance());
                }
                true
            }
            NamedKey::Escape => {
                cancel(channel_id);
                true
            }
            NamedKey::Backspace => {
                backspace_in_slot(channel_id, index);
                true
            }
            NamedKey::ArrowLeft | NamedKey::ArrowRight => {
                commit_text(channel_id, index, text);
                step_sideways(channel_id, Some(index), key == NamedKey::ArrowLeft);
                true
            }
            _ => false,
        })
        .build()
        .wants_focus();
    let mut pill = div()
        .class(cls::CMD_PILL)
        .class(cls::CMD_PILL_FOCUSED)
        .child(key_box(&slot.option.name));
    if chip {
        if let Some(value) = &slot.value {
            pill = pill.child(div().class(cls::CMD_PILL_VALUE).child(value_node(value)));
        }
    }

    pill_frame(channel_id, index, pill.child(input))
}

fn tail_input(ctx: &Ctx<'_>, active: &Active) -> Node {
    let (env, channel) = (ctx.env.clone(), ctx.channel.clone());
    let channel_id = channel.id;
    let missing_required = active.slots.iter().any(|s| !s.added && s.option.required);
    let mut offers: Vec<AutocompleteCandidate> = Vec::new();
    for required in [true, false] {
        for (i, s) in active
            .slots
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.added && s.option.required == required)
        {
            let starts_other =
                !required && missing_required && offers.iter().all(|o| o.section == 1);
            offers.push(AutocompleteCandidate {
                display: s.option.name.clone(),
                subtitle: Some(s.option.description.clone()),
                section: if required { 1 } else { 2 },
                section_label: starts_other
                    .then(|| requiem_i18n::t!("slash-commands-other-options").to_string()),
                on_select: Some(Rc::new(move || add_option(channel_id, i))),
                ..Default::default()
            });
        }
    }
    let label = if missing_required {
        requiem_i18n::t!("slash-commands-required-options").to_string()
    } else {
        requiem_i18n::t!("slash-commands-options").to_string()
    };
    let triggers = if offers.is_empty() {
        Vec::new()
    } else {
        vec![whole_text_trigger(label, offers)]
    };
    let (key_env, key_channel) = (env.clone(), channel.clone());
    let (selection_env, change_env) = (env.clone(), env.clone());
    field(ctx, String::new())
        .autocomplete(triggers)
        .key_hook(move |key: NamedKey, _shift: bool, text: &str| {
            if all_selected(channel_id) {
                return key_in_selection(&selection_env, channel_id, key);
            }
            match key {
                NamedKey::SelectAll => select_all_pressed(channel_id, text),
                NamedKey::Undo => undo(&selection_env, channel_id),
                NamedKey::Redo => redo(&selection_env, channel_id),
                _ => false,
            }
        })
        .on_change(move |text: &str| {
            if all_selected(channel_id) && !text.is_empty() {
                replace_line_with(&change_env, channel_id, text);
            }
        })
        .on_submit(move |_text: String| send(&env, &channel))
        .edge_hook(move |key: NamedKey, _shift: bool, _text: &str| match key {
            NamedKey::Escape => {
                cancel(channel_id);
                true
            }
            NamedKey::Backspace => {
                backspace_at_tail(&key_env, &key_channel);
                true
            }
            NamedKey::ArrowLeft => {
                step_sideways(channel_id, None, true);
                true
            }
            _ => false,
        })
        .build()
        .wants_focus()
}

fn field(ctx: &Ctx<'_>, text: String) -> TextInputProps {
    TextInputProps::default()
        .text(text)
        .font_size(16.0_f32)
        .line_height(22.0_f32)
        .vertical_padding(1.0_f32)
        .color(TEXT_DEFAULT)
        .insets((8.0_f32, 8.0_f32))
        .bare(true)
        .auto_width(true)
        .focused(true)
        .popup_frame(ctx.frame.clone())
        .popup_outset((-1.0_f32, -9.0_f32))
}

fn whole_text_trigger(
    label: String,
    candidates: Vec<AutocompleteCandidate>,
) -> AutocompleteTrigger {
    AutocompleteTrigger {
        trigger_char: '\0',
        candidates,
        only_at_message_start: false,
        chip_background: None,
        chip_text_color: TEXT_DEFAULT,
        popup_label: label,
        multi_word: false,
        whole_text: true,
    }
}

fn value_triggers(ctx: &Ctx<'_>, active: &Active, index: usize) -> Vec<AutocompleteTrigger> {
    let option = &active.slots[index].option;
    let channel_id = ctx.channel.id;
    let chosen =
        |display: String, json: Value, mention: Option<MentionKind>| -> Option<Rc<dyn Fn()>> {
            Some(Rc::new(move || {
                pick(
                    channel_id,
                    index,
                    Filled {
                        display: display.clone(),
                        json: json.clone(),
                        mention,
                    },
                )
            }))
        };
    let options_label = || requiem_i18n::t!("slash-commands-options").to_string();

    if !option.choices.is_empty() {
        let rows = option
            .choices
            .iter()
            .map(|c| AutocompleteCandidate {
                display: c.name.clone(),
                section: 1,
                on_select: chosen(c.name.clone(), c.value.clone(), None),
                ..Default::default()
            })
            .collect();
        return vec![whole_text_trigger(options_label(), rows)];
    }
    let members = |this: &Ctx<'_>| -> Vec<AutocompleteCandidate> {
        message_list::member_candidates(&this.env.state, this.channel, this.env)
            .into_iter()
            .filter(|c| c.section == 0)
            .map(|mut c| {
                let id = c.id.unwrap_or_default();
                c.on_select = chosen(
                    c.display.clone(),
                    json!(id.to_string()),
                    Some(MentionKind::User),
                );
                c.id = None;
                c
            })
            .collect()
    };
    let roles = |this: &Ctx<'_>| -> Vec<AutocompleteCandidate> {
        let Some(guild_id) = this.channel.guild_id else {
            return Vec::new();
        };
        let all = author_style::guild_roles(&this.env.state, guild_id);
        let mut found: Vec<AutocompleteCandidate> = all
            .iter()
            .filter(|r| r.id.get() != guild_id.get())
            .map(|r| {
                let color = match author_style::role_fill(r) {
                    Some(NameFill::Solid(color)) => Some(color),
                    Some(NameFill::Gradient(colors)) => {
                        colors.first().copied()
                    }
                    None => None,
                };
                AutocompleteCandidate {
                    display: r.name.clone(),
                    name_color: color,
                    section: 1,
                    on_select: chosen(
                        r.name.clone(),
                        json!(r.id.to_string()),
                        Some(MentionKind::Role),
                    ),
                    show_trigger: true,
                    ..Default::default()
                }
            })
            .collect();
        found.sort_by(|a, b| a.display.cmp(&b.display));
        found
    };
    match option.kind {
        option_type::BOOLEAN => {
            let rows = [("true", true), ("false", false)]
                .iter()
                .map(|(name, value)| AutocompleteCandidate {
                    display: name.to_string(),
                    section: 1,
                    on_select: chosen(name.to_string(), json!(value), None),
                    ..Default::default()
                })
                .collect();
            vec![whole_text_trigger(options_label(), rows)]
        }
        option_type::USER => vec![whole_text_trigger(
            requiem_i18n::t!("message-list-members").to_string(),
            members(ctx),
        )],
        option_type::ROLE => vec![whole_text_trigger(
            requiem_i18n::t!("command-composer-roles").to_string(),
            roles(ctx),
        )],
        option_type::MENTIONABLE => {
            let mut rows = members(ctx);
            rows.extend(roles(ctx));
            vec![whole_text_trigger(
                requiem_i18n::t!("message-list-members").to_string(),
                rows,
            )]
        }
        option_type::CHANNEL => {
            let rows: Vec<AutocompleteCandidate> = ctx
                .channel
                .guild_id
                .and_then(|guild_id| {
                    ctx.env
                        .state
                        .channels
                        .by_guild
                        .peek()
                        .get(&guild_id)
                        .cloned()
                })
                .into_iter()
                .flatten()
                .filter(|c| {
                    ctx.env
                        .state
                        .can(c, Permissions::VIEW_CHANNEL)
                })
                .filter_map(|c| Some((c.id, c.name.clone()?)))
                .map(|(id, name)| AutocompleteCandidate {
                    display: name.clone(),
                    on_select: chosen(name, json!(id.to_string()), Some(MentionKind::Channel)),
                    show_trigger: true,
                    ..Default::default()
                })
                .collect();
            vec![whole_text_trigger(
                requiem_i18n::t!("message-list-channels").to_string(),
                rows,
            )]
        }
        _ => Vec::new(),
    }
}
