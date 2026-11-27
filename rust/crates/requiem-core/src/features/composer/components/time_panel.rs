use crate::ui::classes::time_panel as time_panel_cls;
use crate::ui::classes::command_composer as cls;
use crate::ui::command_composer;
use requiem_ui::timestamp;
use requiem_ui::widget;
use std::cell::RefCell;
use std::collections::HashMap;

use requiem_ui::colors::{TEXT_CRITICAL, TEXT_DEFAULT, TEXT_MUTED, TEXT_STRONG, TEXT_SUBTLE};
use requiem_ui::components::{
    Anchor, ButtonProps, ButtonVariant, PopoverAlign, PopoverProps, PopoverSide, TextInputProps,
    TextProps, TextVariant, div,
};
use requiem_ui::event::{CursorIcon, NamedKey};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::primitives::ClickAreaWidget;

use crate::models::id::marker::ChannelMarker;
use crate::models::{Channel, Id};
use crate::ui::context_menus::MenuEnv;
use crate::ui::timestamps::{self, Row};

#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    pub prefix: String,
    pub suffix: String,
}

thread_local! {
    static MODE: Signal<HashMap<Id<ChannelMarker>, Input>> = signal(HashMap::new());

    static QUERY: Signal<String> = signal(String::new());

    static SELECTED: Signal<usize> = signal(0);

    static FAILED: Signal<bool> = signal(false);

    static SHOWN: RefCell<Vec<Row>> = const { RefCell::new(Vec::new()) };
}

pub fn active(channel: Id<ChannelMarker>) -> Option<Input> {
    MODE.with(|m| m.with(|m| m.get(&channel).cloned()))
}

fn peek(channel: Id<ChannelMarker>) -> Option<Input> {
    MODE.with(|m| m.with_untracked(|m| m.get(&channel).cloned()))
}

pub fn begin(env: &MenuEnv, channel: Id<ChannelMarker>) {
    let text = env.state.drafts.text(channel);
    let prefix = text
        .rfind('@')
        .map_or(text.as_str(), |at| &text[..at])
        .to_string();
    QUERY.with(|q| q.set(String::new()));
    SELECTED.with(|s| s.set(0));
    FAILED.with(|f| f.set(false));
    MODE.with(|m| {
        m.update(|m| {
            m.insert(
                channel,
                Input {
                    prefix,
                    suffix: String::new(),
                },
            );
        })
    });
    command_composer::reseed();
}

fn finish(env: &MenuEnv, channel: Id<ChannelMarker>, text: &str, focus_composer: &Signal<bool>) {
    MODE.with(|m| {
        m.update(|m| {
            m.remove(&channel);
        })
    });
    QUERY.with(|q| q.set(String::new()));
    env.state.drafts.set(channel, text);

    focus_composer.set(true);
    command_composer::reseed();
}

fn without_pill(input: &Input) -> String {
    format!("{}{}", input.prefix, input.suffix)
}

fn parsed_now() -> Option<timestamps::Parsed> {
    let query = QUERY.with(|q| q.peek());
    timestamps::parse(&query, chrono::Local::now())
}

fn selectable(rows: &[Row]) -> Vec<usize> {
    rows.iter()
        .enumerate()
        .filter(|(_, r)| matches!(r, Row::Format { .. }))
        .map(|(i, _)| i)
        .collect()
}

fn choose(env: &MenuEnv, channel: Id<ChannelMarker>, row: &Row, focus_composer: &Signal<bool>) {
    let (Row::Format { unix, letter, .. }, Some(input)) = (row, peek(channel)) else {
        return;
    };
    let text = format!(
        "{}{}{}",
        input.prefix,
        timestamp::token(*unix, Some(*letter)),
        input.suffix
    );
    finish(env, channel, &text, focus_composer);
}

#[derive(Clone)]
pub struct Ctx {
    pub env: MenuEnv,
    pub channel: Channel,

    pub composer_focused: Signal<bool>,
}

fn key(ctx: &Ctx, key: NamedKey, shift: bool, text: &str) -> bool {
    let channel = ctx.channel.id;
    let Some(input) = peek(channel) else {
        return false;
    };
    let rows = SHOWN.with(|s| s.borrow().clone());
    let picks = selectable(&rows);
    let selected = SELECTED
        .with(|s| s.peek())
        .min(picks.len().saturating_sub(1));
    match key {
        NamedKey::ArrowDown if !picks.is_empty() => {
            SELECTED.with(|s| s.set((selected + 1) % picks.len()));
            true
        }
        NamedKey::ArrowUp if !picks.is_empty() => {
            SELECTED.with(|s| s.set((selected + picks.len() - 1) % picks.len()));
            true
        }
        NamedKey::Enter | NamedKey::Tab if !shift => {
            match picks.get(selected).and_then(|i| rows.get(*i)) {
                Some(row) if parsed_now().is_some() => {
                    choose(&ctx.env, channel, row, &ctx.composer_focused)
                }
                _ => FAILED.with(|f| f.set(true)),
            }
            true
        }
        NamedKey::Escape => {
            finish(
                &ctx.env,
                channel,
                &without_pill(&input),
                &ctx.composer_focused,
            );
            true
        }
        NamedKey::Backspace if text.is_empty() => {
            finish(
                &ctx.env,
                channel,
                &without_pill(&input),
                &ctx.composer_focused,
            );
            true
        }
        _ => false,
    }
}

fn around(text: &str) -> Node {
    let mut shown = String::new();
    let mut from = 0;
    for (range, unix, letter) in timestamp::tokens(text) {
        shown.push_str(&text[from..range.start]);
        shown.push_str(&timestamps::format(unix, letter));
        from = range.end;
    }
    shown.push_str(&text[from..]);
    TextProps::styled(shown, TextVariant::TEXT_MD_NORMAL)
        .line_height(22.0_f32)
        .color(TEXT_DEFAULT)
        .build()
}

pub fn content(ctx: &Ctx, input: &Input) -> Node {
    let failed = FAILED.with(Signal::get);
    let hook_ctx = ctx.clone();
    let field = TextInputProps::default()
        .text(QUERY.with(|q| q.peek()))
        .font_size(16.0_f32)
        .line_height(22.0_f32)
        .vertical_padding(1.0_f32)
        .color(TEXT_DEFAULT)
        .insets((8.0_f32, 8.0_f32))
        .bare(true)
        .auto_width(true)
        .focused(true)
        .on_change(|text: &str| {
            QUERY.with(|q| q.set(text.to_string()));
            SELECTED.with(|s| s.set(0));
            if FAILED.with(|f| f.peek()) {
                FAILED.with(|f| f.set(false));
            }
        })
        .key_hook(move |named: NamedKey, shift: bool, text: &str| {
            key(&hook_ctx, named, shift, text)
        })
        .build()
        .wants_focus();
    let name = div().class(cls::CMD_PILL_KEY).child(
        TextProps::styled("@time".to_string(), TextVariant::TEXT_MD_NORMAL)
            .line_height(22.0_f32)
            .color(TEXT_DEFAULT)
            .build(),
    );
    let mut pill = div()
        .class(cls::CMD_PILL)
        .class(cls::CMD_PILL_FOCUSED)
        .children([name, field]);
    if failed {
        pill = pill.class(cls::CMD_PILL_ERRORED);
    }

    let pill = Node::new("div", ClickAreaWidget::new().with_cursor(CursorIcon::Text)).child(pill);
    let mut line = div().class(cls::CMD_CONTENT_TIME);
    if !input.prefix.is_empty() {
        line = line.child(around(&input.prefix));
    }
    line = line.child(pill);
    if !input.suffix.is_empty() {
        line = line.child(around(&input.suffix));
    }
    line
}

pub fn bar() -> Node {
    let failed = FAILED.with(Signal::get);
    let note = if failed {
        TextProps::styled(
            requiem_i18n::t!("time-panel-pick-a-time").to_string(),
            TextVariant::TEXT_XS_MEDIUM,
        )
        .line_height(16.0_f32)
        .color(TEXT_CRITICAL)
        .build()
    } else {
        TextProps::styled(
            requiem_i18n::t!("time-panel-mention-description").to_string(),
            TextVariant::TEXT_XS_MEDIUM,
        )
        .line_height(16.0_f32)
        .color(TEXT_MUTED)
        .build()
    };
    div().class(time_panel_cls::TIME_BAR).children([
        TextProps::styled("@time".to_string(), TextVariant::TEXT_XS_BOLD)
            .line_height(16.0_f32)
            .color(TEXT_STRONG)
            .build(),
        note,
    ])
}

pub fn panel(ctx: Ctx, frame: Signal<widget::AbsoluteRect>) -> Node {
    Node::reactive(move || {
        if active(ctx.channel.id).is_none() {
            return div();
        }
        let query = QUERY.with(Signal::get);
        let selected = SELECTED.with(Signal::get);
        let now = chrono::Local::now();
        let Some(parsed) = timestamps::parse(&query, now) else {
            SHOWN.with(|s| s.borrow_mut().clear());
            return div();
        };
        let rows = timestamps::rows(&parsed, now);
        SHOWN.with(|s| *s.borrow_mut() = rows.clone());
        let picks = selectable(&rows);
        let selected = selected.min(picks.len().saturating_sub(1));
        let heading = if query.trim().is_empty() {
            requiem_i18n::t!("time-panel-formats").to_string()
        } else {
            requiem_i18n::t!("time-panel-formats-for", query = query.trim()).to_string()
        };
        let mut list = div().class(time_panel_cls::TIME_LIST).child(
            TextProps::styled(heading.to_uppercase(), TextVariant::TEXT_XS_SEMIBOLD)
                .line_height(16.0_f32)
                .color(TEXT_SUBTLE)
                .line_clamp(1_u32)
                .build()
                .class(time_panel_cls::TIME_HEADING),
        );
        for (index, row) in rows.iter().enumerate() {
            match row {
                Row::Divider => list = list.child(div().class(time_panel_cls::TIME_DIVIDER)),
                Row::Format {
                    unix,
                    letter,
                    description,
                } => {
                    let is_selected = picks.get(selected) == Some(&index);
                    let (pick_ctx, pick_row) = (ctx.clone(), row.clone());
                    let text = TextProps::styled(
                        timestamps::format(*unix, Some(*letter)),
                        TextVariant::TEXT_MD_NORMAL,
                    )
                    .line_height(22.0_f32)
                    .color(TEXT_DEFAULT)
                    .build();
                    let mut children = vec![div().class(time_panel_cls::TIME_TEXT_BOX).child(text)];
                    if let Some(key) = description {
                        children.push(
                            TextProps::styled(requiem_i18n::text(key), TextVariant::TEXT_XS_NORMAL)
                                .line_height(16.0_f32)
                                .color(TEXT_SUBTLE)
                                .build()
                                .class(time_panel_cls::TIME_DESCRIPTION),
                        );
                    }
                    let mut button = ButtonProps::default()
                        .variant(ButtonVariant::Bare)
                        .extra_classes(vec![time_panel_cls::TIME_ROW.to_string()])
                        .on_click(move || {
                            choose(
                                &pick_ctx.env,
                                pick_ctx.channel.id,
                                &pick_row,
                                &pick_ctx.composer_focused,
                            )
                        })
                        .build()
                        .children(children);
                    if is_selected {
                        button = button.class(time_panel_cls::TIME_ROW_SELECTED);
                    }
                    list = list.child(button);
                }
            }
        }

        let box_rect = frame.get();
        let target = widget::AbsoluteRect {
            x: box_rect.x + 1.0,
            width: (box_rect.width - 10.0).max(0.0),
            ..box_rect
        };
        let anchor = Anchor::new(target)
            .position(PopoverSide::Top)
            .align(PopoverAlign::Start)
            .spacing(6.0)
            .auto_invert(false)
            .nudge(false);
        PopoverProps::default()
            .anchor(anchor)
            .build()
            .class(time_panel_cls::TIME_PANEL)
            .style(format!("width: {}px", target.width))
            .child(list)
    })
}
