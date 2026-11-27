use crate::ui::classes::reply as cls;
use crate::models;
use crate::ui::author_style;
use crate::ui::twemoji;
use crate::ui;
use requiem_ui::colors;
use requiem_ui::components::ButtonVariant;
use requiem_ui::components::TextVariant;
use requiem_ui::components::{MessageTextProps, line_clamp_class, message_text};
use requiem_ui::widget::primitives::ImageWidget;
use requiem_ui::widget::primitives::MarkdownFeatures;
use requiem_ui::widget::primitives;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;

use requiem_ui::colors::TEXT_ANCHOR;
use requiem_ui::components::{ButtonProps, TextProps, div, icon};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::primitives::ClickAreaWidget;
use requiem_ui::widget::{AbsoluteRect, Widget};
use skia_safe::font_style::Weight;
use skia_safe::{Canvas, Color, Paint, PaintStyle, PathBuilder, Rect};

use crate::models::Id;
use crate::models::id::marker::{ChannelMarker, GuildMarker, MessageMarker, UserMarker};
use crate::rest::endpoints::messages::ReplyTo;
use requiem_ui::icons::{AT, CIRCLE_X};

#[derive(Clone)]
pub struct ReplyTarget {
    pub channel_id: Id<ChannelMarker>,
    pub message_id: Id<MessageMarker>,

    pub guild_id: Option<Id<GuildMarker>>,
    pub author_id: Id<UserMarker>,
    pub author_name: String,

    pub pings: bool,
}

thread_local! {
    static REPLYING: Signal<HashMap<Id<ChannelMarker>, ReplyTarget>> = signal(HashMap::new());
}

static OUTGOING: Mutex<Option<HashMap<u64, ReplyTo>>> = Mutex::new(None);

pub fn start(target: ReplyTarget) {
    REPLYING.with(|r| {
        r.update(|replies| {
            replies.insert(target.channel_id, target);
        })
    });
}

pub fn cancel(channel: Id<ChannelMarker>) {
    if is_active(channel) {
        REPLYING.with(|r| {
            r.update(|replies| {
                replies.remove(&channel);
            })
        });
    }
}

pub fn is_active(channel: Id<ChannelMarker>) -> bool {
    REPLYING.with(|r| r.with_untracked(|replies| replies.contains_key(&channel)))
}

pub fn target_in(channel: Id<ChannelMarker>) -> Option<ReplyTarget> {
    REPLYING.with(|r| r.with(|replies| replies.get(&channel).cloned()))
}

pub fn take_for_send(channel: Id<ChannelMarker>) {
    let Some(target) =
        REPLYING.with(|r| r.with_untracked(|replies| replies.get(&channel).cloned()))
    else {
        return;
    };
    OUTGOING
        .lock()
        .expect("outgoing replies poisoned")
        .get_or_insert_with(HashMap::new)
        .insert(
            channel.get(),
            ReplyTo {
                message_id: target.message_id,
                channel_id: channel,
                guild_id: target.guild_id,
                pings: target.pings,
            },
        );
    REPLYING.with(|r| {
        r.update(|replies| {
            replies.remove(&channel);
        })
    });
}

pub fn take_outgoing(channel: Id<ChannelMarker>) -> Option<ReplyTo> {
    OUTGOING
        .lock()
        .expect("outgoing replies poisoned")
        .as_mut()?
        .remove(&channel.get())
}

pub fn bar(channel: Id<ChannelMarker>, own_id: Option<Id<UserMarker>>) -> Node {
    Node::reactive(move || {
        let Some(target) = target_in(channel) else {
            return div();
        };
        let piece = |content: &str, weight: Weight, color: Color| {
            TextProps::new(content.to_string(), 14.0_f32)
                .weight(weight)
                .color(color)
                .build()
        };
        let mut bar = div().class(cls::REPLY_BAR).child(
            div()
                .class(cls::REPLY_BAR_TEXT)
                .child(piece(
                    &requiem_i18n::t!("reply-replying-to"),
                    Weight::NORMAL,
                    ui::TEXT_DEFAULT,
                ))
                .child(piece(
                    &target.author_name,
                    Weight::SEMI_BOLD,
                    ui::TEXT_DEFAULT,
                )),
        );
        if own_id != Some(target.author_id) {
            let color = if target.pings {
                TEXT_ANCHOR
            } else {
                Color::from_rgb(0x96, 0x97, 0x9e)
            };
            let toggle = ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![cls::REPLY_TOGGLE.to_string()])
                .on_click(move || {
                    REPLYING.with(|r| {
                        r.update(|replies| {
                            if let Some(t) = replies.get_mut(&channel) {
                                t.pings = !t.pings
                            }
                        })
                    })
                })
                .build()
                .child(icon(AT, color).class(cls::REPLY_TOGGLE_ICON))
                .child(piece(
                    if target.pings { "ON" } else { "OFF" },
                    Weight::BOLD,
                    color,
                ));
            bar = bar.child(toggle).child(div().class(cls::REPLY_DIVIDER));
        }
        let close = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::REPLY_CLOSE.to_string()])
            .on_click(move || cancel(channel))
            .build()
            .child(icon(CIRCLE_X, ui::TEXT_SECONDARY).class(cls::REPLY_CLOSE_ICON));
        bar.child(close)
    })
}

pub enum Snippet {
    Text(String),

    Attachment,
    Deleted,
}

impl Snippet {
    pub fn of(target: Option<&models::Message>) -> Self {
        let Some(target) = target else {
            return Self::Deleted;
        };
        let text = target
            .content
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if text.is_empty() && !target.attachments.is_empty() {
            Self::Attachment
        } else {
            Self::Text(text)
        }
    }
}

#[derive(Clone, Copy)]
pub enum AuthorPart {
    Avatar,
    Name,
}

#[allow(clippy::too_many_arguments)]
pub fn preview(
    avatar: Option<Node>,
    author_name: Option<(&str, Option<&author_style::NameFill>)>,
    author_tag: Option<Node>,
    snippet: Snippet,
    jump: Option<Rc<dyn Fn()>>,
    emoji: primitives::CustomEmojiResolver,
    mentions: Rc<HashMap<u64, String>>,
    channels: Rc<HashMap<u64, String>>,
    roles: Rc<HashMap<u64, (String, Option<Color>)>>,
    wrap_author: &dyn Fn(Node, AuthorPart) -> Node,
) -> Node {
    let text = |content: String, color: Color, italic: bool| {
        TextProps::styled(content, TextVariant::TEXT_SM_NORMAL)
            .color(color)
            .italic(italic)
            .line_clamp(1_u32)
            .build()
    };
    let name = author_name
        .map(|(name, fill)| {
            author_style::name_node(
                name.to_string(),
                TextVariant::TEXT_SM_MEDIUM,
                16.0,
                fill,
                Color::from_argb(163, 0xfb, 0xfb, 0xfb),
            )
            .class(cls::REPLY_AUTHOR)
        })
        .map(|name| wrap_author(name, AuthorPart::Name));
    let muted = Color::from_rgb(0x96, 0x97, 0x9e);
    let snippet = match snippet {
        Snippet::Text(content) => message_text(MessageTextProps {
            content,
            size: 14.0,
            color: ui::TEXT_DEFAULT,
            mentions,
            channels,
            roles,
            emoji_provider: Some(twemoji::tile),
            custom_emoji: Some(emoji),
            markdown: Some(MarkdownFeatures::INLINE),
            inert: true,
            ..Default::default()
        })
        .class(line_clamp_class(1)),
        Snippet::Attachment => text(
            requiem_i18n::t!("reply-click-to-see-attachment").to_string(),
            ui::TEXT_DEFAULT,
            true,
        ),
        Snippet::Deleted => text(
            requiem_i18n::t!("reply-original-message-was-deleted").to_string(),
            muted,
            true,
        ),
    };
    let mut line = div()
        .class(cls::REPLY_CONTEXT)
        .child(Node::new("div", ConnectorWidget).class(cls::REPLY_CONNECTOR));
    if let Some(avatar) = avatar {
        line = line.child(wrap_author(
            avatar.class(cls::REPLY_AVATAR),
            AuthorPart::Avatar,
        ));
    }
    if let Some(name) = name {
        line = line.child(name);
    }
    if let Some(tag) = author_tag {
        line = line.child(tag.class(cls::REPLY_TAG));
    }
    let line = line.child(div().class(cls::REPLY_SNIPPET).child(snippet));
    match jump {
        Some(jump) => Node::new("div", ClickAreaWidget::new().with_on_click(move || jump()))
            .class(cls::REPLY_JUMP)
            .child(line),
        None => line,
    }
}

pub fn used_line(avatar: Option<Node>, name: Node, tag: Option<Node>, command: Node) -> Node {
    let mut line = div()
        .class(cls::REPLY_CONTEXT)
        .child(Node::new("div", ConnectorWidget).class(cls::REPLY_CONNECTOR));
    if let Some(avatar) = avatar {
        line = line.child(avatar.class(cls::REPLY_AVATAR));
    }
    line = line.child(name.class(cls::REPLY_AUTHOR));
    if let Some(tag) = tag {
        line = line.child(tag.class(cls::REPLY_TAG));
    }
    line.child(
        TextProps::styled(
            requiem_i18n::t!("reply-used").to_string(),
            TextVariant::TEXT_SM_NORMAL,
        )
        .color(Color::from_argb(163, 0xfb, 0xfb, 0xfb))
        .build()
        .class(cls::USED_TEXT),
    )
    .child(command)
}

pub fn command_chip(name: &str) -> Node {
    let mut chip = div().class(cls::USED_CHIP);
    if let Some(image) = ui::asset_svg("image/application_command", 28, 28) {
        chip = chip.child(
            Node::new(
                "div",
                ImageWidget::new(Some(image)),
            )
            .class(cls::USED_CHIP_ICON),
        );
    }
    chip.child(
        TextProps::styled(name.to_string(), TextVariant::TEXT_SM_NORMAL)
            .color(colors::TEXT_MENTION)
            .build(),
    )
}

struct ConnectorWidget;

impl Widget for ConnectorWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        const STROKE: f32 = 2.0;
        const RADIUS: f32 = 6.0;
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_style(PaintStyle::Stroke);
        paint.set_stroke_width(STROKE);
        paint.set_color(Color::from_rgb(0x47, 0x48, 0x51));
        let (left, top) = (rect.x + STROKE / 2.0, rect.y + STROKE / 2.0);
        let mut path = PathBuilder::new();
        path.move_to((left, rect.y + rect.height));
        path.line_to((left, top + RADIUS));
        path.arc_to(
            Rect::from_xywh(left, top, RADIUS * 2.0, RADIUS * 2.0),
            180.0,
            90.0,
            false,
        );
        path.line_to((rect.x + rect.width, top));
        canvas.draw_path(&path.detach(), &paint);
    }
}

#[cfg(test)]
mod tests {
    use requiem_ui::font_system::TextFonts;
    use requiem_ui::layout;
    use requiem_ui::paint as requiem_ui_paint;
    use requiem_ui::style::ComputedStyle;
    use requiem_ui::style::cascade;
    use requiem_ui::style::parser;
    use requiem_ui::widget::WidgetTree;
    use requiem_ui::widget::primitives::BoxWidget;
    use taffy::{AvailableSpace, Size};

    use super::*;
    use crate::ui::classes::message_list as message_list_cls;

    fn chat_frame_width(snippet: Snippet) -> (f32, f32) {
        TextFonts::install_new(1024 * 1024);
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let root = tree.root;
        tree.nodes[root].classes.push("root".to_string());
        let resolver: primitives::CustomEmojiResolver = Rc::new(|_, _| None);
        let wrap = |node: Node, _part: AuthorPart| node;
        let body = div().class(message_list_cls::MESSAGE_BODY).child(preview(
            None,
            Some(("someone", None)),
            None,
            snippet,
            Some(Rc::new(|| {})),
            resolver,
            Rc::default(),
            Rc::default(),
            Rc::default(),
            &wrap,
        ));
        let row = div().class(message_list_cls::MESSAGE_ROW).children([
            div().style("width: 40px; height: 40px; flex-shrink: 0"),
            body,
        ]);
        let frame = tree.mount(
            root,
            div()
                .class(message_list_cls::CHAT_FRAME)
                .child(div().class(message_list_cls::MESSAGE_LIST).child(row)),
        );
        tree.mount(
            root,
            div().style("width: 240px; flex-shrink: 0; height: 100px"),
        );
        let theme = parser::parse_stylesheet(&format!(
            "{}
{}
.root {{ position: relative; width: 1000px; height: 300px; display: flex; flex-direction: row; }}",
            requiem_ui::BUNDLED_CSS,
            ui::THEME_CSS
        ));
        cascade::apply_stylesheet(&mut tree, &theme);
        layout::compute_layout(
            &mut tree,
            Size {
                width: AvailableSpace::Definite(1000.0),
                height: AvailableSpace::Definite(300.0),
            },
        );
        let rects = requiem_ui_paint::all_absolute_rects(&tree);

        let bottom = rects
            .values()
            .map(|r| r.y + r.height)
            .fold(0.0_f32, f32::max);
        (rects[&frame].width, bottom)
    }

    #[test]
    fn a_long_reply_snippet_does_not_widen_the_chat() {
        let (short, short_bottom) = chat_frame_width(Snippet::Text("hello".to_string()));
        for (name, text) in [
            ("plain text", "word ".repeat(400)),
            (
                "text with custom emoji",
                format!("<a:wave:1> {}", "word ".repeat(400)),
            ),
            ("one unbroken word", "x".repeat(2000)),
            ("a mention", format!("<@1> {}", "word ".repeat(400))),
        ] {
            let (width, bottom) = chat_frame_width(Snippet::Text(text));
            assert_eq!(width, short, "{name} widens the chat");
            assert_eq!(bottom, short_bottom, "{name} wraps to more than one line");
        }
    }
}
