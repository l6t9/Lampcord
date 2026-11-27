use crate::ui::classes::reactions as cls;
use crate::ui::context_menu;
use crate::ui::emoji_anim::EmojiImageWidget;
use crate::ui::emoji_names;
use crate::ui::emoji_picker::CustomEmojiSource;
use crate::ui::emoji_picker;
use crate::ui::emoji_skin;
use crate::ui::message_toolbar;
use crate::utils::time;
use requiem_ui::app;
use requiem_ui::colors::{TEXT_SECONDARY, TEXT_STRONG};
use requiem_ui::components::ButtonVariant;
use requiem_ui::components::TextVariant;
use requiem_ui::components::{Padding, StackProps};
use requiem_ui::components;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use requiem_ui::components::{ButtonProps, PopoverProps, TextProps, div, icon};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::primitives::ImageWidget;
use requiem_ui::widget::{AbsoluteRect, Widget};
use skia_safe::{Canvas, Color};

use crate::config::BackendConfig;
use crate::frecency::Frecency;
use crate::images::ImageCache;
use crate::models::id::marker::{ChannelMarker, GuildMarker, MessageMarker};
use crate::models::{Id, Message, Reaction, ReactionEmoji};
use crate::rest::RestError;
use crate::rest::endpoints::{emoji_frecency, reactions};
use crate::ui::context_menus::{MenuEnv, run_with_toast};
use crate::ui::twemoji;
use requiem_ui::icons::EMOJI_SLIGHTLY_SMILING_FACE;

const FRECENCY_KEY: &str = "reaction-frecency";

thread_local! {

    static WORKING: RefCell<Option<Frecency>> = const { RefCell::new(None) };
}

fn frecency(env: &MenuEnv) -> Frecency {
    if let Some(known) = WORKING.with(|w| w.borrow().clone()) {
        return known;
    }
    if let Some(http) = env.http.clone() {
        let loaded = env
            .queries
            .query_default::<Frecency, _, RestError>(FRECENCY_KEY, move || async move {
                emoji_frecency::get_reaction_frecency(&http).await
            });
        if let Some(loaded) = loaded.data {
            let list = if loaded.is_empty() {
                Frecency::default_reactions(time::now_ms())
            } else {
                (*loaded).clone()
            };
            WORKING.with(|w| *w.borrow_mut() = Some(list.clone()));
            return list;
        }
    }
    Frecency::default_reactions(time::now_ms())
}

fn emoji_for_key(key: &str) -> Option<ReactionEmoji> {
    emoji_names::emoji_for_name(key)
        .map(|emoji| ReactionEmoji::unicode(&emoji_skin::plain(emoji)))
}

pub fn quick_emoji(
    env: &MenuEnv,
    guild_id: Option<Id<GuildMarker>>,
    count: usize,
) -> Vec<ReactionEmoji> {
    let usable = |key: &str| -> Option<ReactionEmoji> {
        match key.parse::<u64>() {
            Ok(id) => {
                let (guild, emoji) = env.state.emojis.find(Id::new(id))?;
                (Some(guild) == guild_id && emoji.available).then_some(ReactionEmoji {
                    id: Some(emoji.id),
                    name: Some(emoji.name),
                    animated: emoji.animated,
                })
            }
            Err(_) => emoji_for_key(key),
        }
    };

    let mut seen = HashSet::new();
    frecency(env)
        .frequently(time::now_ms(), 42)
        .iter()
        .filter_map(|key| usable(key))
        .filter(|emoji| seen.insert((emoji.id, emoji.name.clone())))
        .take(count)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_emoji_markup_becomes_a_reaction_emoji() {
        let emoji = parse_custom_markup("<:happy:123>").unwrap();
        assert_eq!(
            (
                emoji.id.map(|id| id.get()),
                emoji.name.as_deref(),
                emoji.animated
            ),
            (Some(123), Some("happy"), false)
        );
        assert!(parse_custom_markup("<a:wiggle:9>").unwrap().animated);
        assert_eq!(parse_custom_markup("\u{1F44D}"), None);
        assert_eq!(parse_custom_markup("<:no_id:>"), None);
    }
}

pub fn parse_custom_markup(text: &str) -> Option<ReactionEmoji> {
    let inner = text.strip_prefix('<')?.strip_suffix('>')?;
    let (animated, inner) = match inner.strip_prefix("a:") {
        Some(rest) => (true, rest),
        None => (false, inner.strip_prefix(':')?),
    };
    let (name, id) = inner.rsplit_once(':')?;
    let id = id.parse::<u64>().ok()?;
    Some(ReactionEmoji {
        id: Some(Id::new(id)),
        name: Some(name.to_string()),
        animated,
    })
}

pub fn emoji_label(emoji: &ReactionEmoji) -> String {
    emoji
        .name
        .as_deref()
        .and_then(emoji_names::primary_name)
        .map_or_else(|| emoji.name.clone().unwrap_or_default(), str::to_string)
}

fn track(env: &MenuEnv, emoji: &ReactionEmoji) {
    let Some(key) = (match emoji.id {
        Some(id) => Some(id.to_string()),
        None => emoji
            .name
            .as_deref()
            .and_then(|name| {
                emoji_names::primary_name(&emoji_skin::plain(name))
            })
            .map(str::to_string),
    }) else {
        return;
    };
    let mut list = frecency(env);
    list.track(&key, time::now_ms());
    WORKING.with(|w| *w.borrow_mut() = Some(list.clone()));
    env.queries.set_data(FRECENCY_KEY, list.clone());
    if let Some(http) = env.http.clone() {
        env.queries.mutate(Vec::new(), move || async move {
            emoji_frecency::set_reaction_frecency(&http, &list, time::now_ms()).await
        });
    }
}

pub fn react(
    env: &MenuEnv,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    emoji: &ReactionEmoji,
    remove: bool,
    location: &'static str,
) {
    let Some(http) = env.http.clone() else { return };
    let emoji = emoji.clone();
    if remove {
        run_with_toast(
            &env.queries,
            Vec::new(),
            String::new(),
            requiem_i18n::t!("reactions-unable-to-remove-reaction"),
            move || async move {
                reactions::remove_reaction(&http, channel_id, message_id, &emoji, location).await
            },
        );
    } else {
        track(env, &emoji);
        run_with_toast(
            &env.queries,
            Vec::new(),
            String::new(),
            requiem_i18n::t!("reactions-unable-to-add-reaction"),
            move || async move {
                reactions::add_reaction(&http, channel_id, message_id, &emoji, location).await
            },
        );
    }
}

pub fn emoji_node(
    emoji: &ReactionEmoji,
    class: &'static str,
    images: &ImageCache,
    backend: &BackendConfig,
) -> Node {
    match emoji.id {
        Some(id) => Node::new(
            "div",
            EmojiImageWidget::new(id.get(), emoji.animated, images, backend),
        )
        .class(class),
        None => match emoji.name.as_deref() {
            Some(glyph) => match twemoji::tile(glyph) {
                Some((image, (x, y, w, h))) => Node::new(
                    "div",
                    ImageWidget::new(Some(image)).with_src_rect(x, y, w, h),
                )
                .class(class),
                None => div().class(class),
            },
            None => div().class(class),
        },
    }
}

pub fn menu_row(
    env: &MenuEnv,
    channel_id: Id<ChannelMarker>,
    guild_id: Option<Id<GuildMarker>>,
    message_id: Id<MessageMarker>,
) -> Node {
    let mut row = StackProps::horizontal()
        .gap(4)
        .full_width(false)
        .padding(Padding::all(8))
        .build();
    for emoji in quick_emoji(env, guild_id, 4) {
        let (react_env, picked) = (env.clone(), emoji.clone());
        let button = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::MENU_QUICK_BUTTON.to_string()])
            .on_click(move || {
                react(
                    &react_env,
                    channel_id,
                    message_id,
                    &picked,
                    false,
                    reactions::LOCATION_CONTEXT_MENU,
                );
                context_menu::close();
            })
            .build()
            .child(emoji_node(
                &emoji,
                cls::MENU_QUICK_EMOJI,
                &env.images,
                &env.backend,
            ));
        row = row.child(button);
    }
    row
}

pub const PICKER_DOM_ID: &str = "reaction-picker-panel";

const PICKER_SIZE: (f32, f32) = (498.0, 509.0);

#[derive(Clone)]
struct PickerTarget {
    channel_id: Id<ChannelMarker>,
    guild_id: Option<Id<GuildMarker>>,
    message_id: Id<MessageMarker>,

    anchor: AbsoluteRect,
}

struct OriginProbe;

impl Widget for OriginProbe {
    fn paint(&self, _canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        LAYER_ORIGIN.with(|o| o.set((rect.x, rect.y)));
    }
}

thread_local! {
    static LAYER_ORIGIN: Cell<(f32, f32)> = const { Cell::new((0.0, 0.0)) };
    static PICKER: Signal<Option<PickerTarget>> = signal(None);
    static PICKER_SIGNALS: emoji_picker::EmojiPickerSignals = emoji_picker::EmojiPickerSignals {
        search: signal(String::new()),
        scroll_offset: signal(0.0),
        category_jump: signal(None),
        hovered: signal(None),
        collapsed: signal(Vec::new()),
    };
}

pub fn open_picker(
    channel_id: Id<ChannelMarker>,
    guild_id: Option<Id<GuildMarker>>,
    message_id: Id<MessageMarker>,
    anchor: AbsoluteRect,
) {
    PICKER_SIGNALS.with(|s| {
        s.search.set(String::new());
        s.reset_for_open();
    });
    PICKER.with(|p| {
        p.set(Some(PickerTarget {
            channel_id,
            guild_id,
            message_id,
            anchor,
        }))
    });
}

fn picker_position(anchor: AbsoluteRect) -> (f32, f32) {
    let (window_width, window_height) = app::current_window_size();
    let x = (anchor.x + anchor.width - PICKER_SIZE.0)
        .clamp(8.0, (window_width - PICKER_SIZE.0 - 8.0).max(8.0));
    let below = anchor.y + anchor.height + 8.0;
    let y = if below + PICKER_SIZE.1 <= window_height {
        below
    } else {
        (anchor.y - 8.0 - PICKER_SIZE.1).max(8.0)
    };
    (x, y)
}

pub fn close_picker() {
    if picker_open() {
        PICKER.with(|p| p.set(None));
    }
}

pub fn picker_open() -> bool {
    PICKER.with(|p| p.peek()).is_some()
}

pub fn picker_overlay(env: &MenuEnv, image_tick: &Signal<u64>) -> Node {
    let (env, image_tick) = (env.clone(), image_tick.clone());
    Node::reactive(move || {
        let probe = Node::new("div", OriginProbe);
        let Some(target) = PICKER.with(Signal::get) else {
            return probe;
        };
        let pick_env = env.clone();
        let on_pick: Rc<dyn Fn(String, bool)> = Rc::new(move |picked, keep_open| {
            let emoji =
                parse_custom_markup(&picked).unwrap_or_else(|| ReactionEmoji::unicode(&picked));
            react(
                &pick_env,
                target.channel_id,
                target.message_id,
                &emoji,
                false,
                reactions::LOCATION_REACTION_PICKER,
            );
            if !keep_open {
                close_picker();
            }
        });

        let custom = CustomEmojiSource::new(
            &env,
            target.guild_id,
            false,
            &image_tick,
        );
        let panel = PICKER_SIGNALS.with(|signals| {
            emoji_picker::build_emoji_only(signals, on_pick, Some(custom))
        });

        let (window_x, window_y) = picker_position(target.anchor);
        let (origin_x, origin_y) = LAYER_ORIGIN.with(Cell::get);

        probe
            .child(components::anchored_popover_backdrop(
                close_picker,
            ))
            .child(
                PopoverProps::default()
                    .position(window_x - origin_x, window_y - origin_y)
                    .dom_id(PICKER_DOM_ID)
                    .build()
                    .child(panel),
            )
    })
    .class(cls::REACTION_PICKER_LAYER)
}

#[derive(Clone)]
pub struct ReactionActions {
    pub react: Rc<dyn Fn(&ReactionEmoji, bool)>,

    pub add: Rc<dyn Fn(AbsoluteRect)>,

    pub can_add: bool,
}

const COUNT_COLOR_ME: Color = Color::from_argb(255, 0xb9, 0xc8, 0xff);

pub fn row(
    message: &Message,
    images: &ImageCache,
    backend: &BackendConfig,
    actions: &ReactionActions,
) -> Option<Node> {
    if message.reactions.is_empty() {
        return None;
    }
    let mut row = div().class(cls::REACTIONS);
    for reaction in &message.reactions {
        row = row.child(pill(reaction, images, backend, actions));
    }
    let add = actions.add.clone();
    let message_id = message.id.get();
    let can_add = actions.can_add;
    Some(row.child(Node::reactive(move || {
        if !can_add || !message_toolbar::is_hovered(message_id) {
            return div();
        }
        let add = add.clone();
        ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::REACTION_ADD.to_string()])
            .on_click_at(move |rect: AbsoluteRect| add(rect))
            .build()
            .child(icon(EMOJI_SLIGHTLY_SMILING_FACE, TEXT_SECONDARY).class(cls::REACTION_ADD_ICON))
    })))
}

fn pill(
    reaction: &Reaction,
    images: &ImageCache,
    backend: &BackendConfig,
    actions: &ReactionActions,
) -> Node {
    let me = reaction.me;
    let (emoji, react, can_add) = (
        reaction.emoji.clone(),
        actions.react.clone(),
        actions.can_add,
    );
    let mut classes = vec![cls::REACTION.to_string()];
    if me {
        classes.push(cls::REACTION_ME.to_string());
    }
    let count = TextProps::styled(reaction.count.to_string(), TextVariant::TEXT_MD_SEMIBOLD)
        .line_height(16.0_f32)
        .color(if me { COUNT_COLOR_ME } else { TEXT_SECONDARY });
    let count = if me {
        count
    } else {
        count.hover_color(TEXT_STRONG)
    }
    .build()
    .class(cls::REACTION_COUNT);
    ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(classes)
        .on_click(move || {
            if me || can_add {
                react(&emoji, me);
            }
        })
        .build()
        .child(emoji_node(
            &reaction.emoji,
            cls::REACTION_EMOJI,
            images,
            backend,
        ))
        .child(count)
}
