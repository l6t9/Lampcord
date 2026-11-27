use crate::emoji_display;
use crate::font_system;
use crate::signal;
use crate::widget::primitives;
use crate::widget;
use skia_safe::font_style::Weight;
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use skia_safe::Color;

use super::text::TextProps;
use crate::colors::{TEXT_DEFAULT, TEXT_STRONG, TEXT_SUBTLE};
use crate::font_system::TextFonts;
use crate::node::Node;
use crate::text_cache::MeasureCache;
use crate::widget::primitives::markdown_blocks::{self, Block, List};
use crate::widget::primitives::{BoxWidget, EmojiProvider, MessageTextWidget};

#[derive(Default)]
pub struct MessageTextProps {
    pub content: String,

    pub size: f32,
    pub color: Color,

    pub fonts: Option<TextFonts>,
    /// User id to display name, for `<@id>` tokens.
    pub mentions: Rc<HashMap<u64, String>>,
    /// Channel id to name, for `<#id>` tokens.
    pub channels: Rc<HashMap<u64, String>>,

    /// Role id to name and color, for `<@&id>` tokens.
    pub roles: Rc<HashMap<u64, (String, Option<Color>)>>,

    /// `@everyone`, `@here`, and role-less mentions highlight when set.
    pub special_mentions: bool,

    pub on_mention:
        Option<Rc<dyn Fn(primitives::MentionTarget, widget::AbsoluteRect)>>,

    /// Reuses a previous wrap measurement for `cache_key`.
    pub measure_cache: Option<Rc<MeasureCache>>,
    pub cache_key: Option<u64>,

    /// Resolves a unicode emoji to an image. `None` leaves the glyph as text.
    pub emoji_provider: Option<EmojiProvider>,

    /// Resolves `<:name:id>` tokens. `None` leaves them as text.
    pub custom_emoji: Option<primitives::CustomEmojiResolver>,

    /// Which markdown spans to honor. `None` paints the string as plain text.
    pub markdown: Option<primitives::MarkdownFeatures>,

    /// Emoji size relative to the font size. `None` uses 1.
    pub emoji_scale: Option<f32>,

    pub line_height: Option<f32>,

    /// Substrings drawn with the highlight wash, case-insensitive.
    pub highlights: Vec<String>,

    pub on_emoji_click: Option<primitives::EmojiClickHandler>,

    /// Emoji occupy a square of the line height instead of the glyph's own width.
    pub emoji_in_box: bool,

    /// Link color and font weight. `None` uses the body style.
    pub link_style: Option<(Color, u16)>,

    /// Colors for particular link targets, over `link_style`'s, e.g. a
    /// system message's author in their role color.
    pub link_colors: Vec<(String, Color)>,

    pub on_link: Option<Rc<dyn Fn(&str, widget::AbsoluteRect)>>,

    /// Takes no clicks or hover: for previews that sit inside something clickable.
    pub inert: bool,

    /// `(edited)` label and its tooltip, drawn after the text.
    pub edited: Option<(String, String)>,

    /// Bumps a repaint when an emoji image arrives after first layout.
    pub image_tick: Option<signal::Signal<u64>>,
}

pub fn message_text(props: MessageTextProps) -> Node {
    let fonts = font_system::resolve(props.fonts.clone());

    let content = match emoji_display::apply(&props.content) {
        Cow::Borrowed(_) => props.content.clone(),
        Cow::Owned(shown) => shown,
    };
    let shared = Shared {
        fonts,
        props,
        counter: Cell::new(0),
    };
    if let Some(features) = shared.props.markdown.as_ref() {
        if let Some(blocks) = markdown_blocks::blocks(&content, features) {
            return shared.column(&blocks, 0);
        }
    }
    let mut leaf = Leaf::plain(&shared.props);
    leaf.edited = true;
    shared.leaf(content, leaf, shared.props.cache_key)
}

struct Leaf {
    size: f32,
    color: Color,
    weight: Option<Weight>,
    line_height: Option<f32>,
    edited: bool,
}

impl Leaf {
    fn plain(props: &MessageTextProps) -> Self {
        Self {
            size: props.size,
            color: props.color,
            weight: None,
            line_height: props.line_height,
            edited: false,
        }
    }
}

struct Shared {
    fonts: TextFonts,
    props: MessageTextProps,

    counter: Cell<u64>,
}

const HEADING_SIZES: [f32; 3] = [24.0, 20.0, 16.0];
const SUBTEXT_SIZE: f32 = 14.0;
const SUBTEXT_LINE_HEIGHT: f32 = 19.25;

const QUOTE_BAR_WIDTH: f32 = 4.0;
const QUOTE_TEXT_INSET: f32 = 12.0;
const QUOTE_BAR_COLOR: Color = Color::from_argb(255, 0x47, 0x48, 0x51);

const LIST_INDENT: f32 = 16.0;
const BULLET_SIZE: f32 = 5.0;
const BULLET_GAP: f32 = 7.0;

const NUMBER_GAP: f32 = 4.0;

fn hex(color: Color) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b())
}

fn boxed() -> Node {
    Node::new("div", BoxWidget)
}

impl Shared {
    fn leaf(&self, content: String, leaf: Leaf, cache_key: Option<u64>) -> Node {
        let props = &self.props;
        let mut widget = MessageTextWidget::new(
            content,
            leaf.size,
            leaf.color,
            self.fonts.font_system.clone(),
            self.fonts.glyph_cache.clone(),
            props.mentions.clone(),
            props.channels.clone(),
        );
        widget = widget
            .with_role_mentions(props.roles.clone())
            .with_special_mentions(props.special_mentions);
        if let Some(handler) = &props.on_mention {
            widget = widget.with_on_mention(handler.clone());
        }
        if let Some(cache) = &props.measure_cache {
            widget = widget.with_measure_cache(cache.clone(), cache_key);
        }
        if let Some(provider) = props.emoji_provider {
            widget = widget.with_emoji_provider(provider);
        }
        if let Some(resolver) = &props.custom_emoji {
            widget = widget.with_custom_emoji(resolver.clone());
        }
        if let Some(line_height) = leaf.line_height {
            widget = widget.with_line_height(line_height);
        }
        if let Some(weight) = leaf.weight {
            widget = widget.with_weight(weight);
        }
        if let Some(features) = props.markdown {
            widget = widget.with_markdown(features);
        }
        if let Some(scale) = props.emoji_scale {
            widget = widget.with_emoji_scale(scale);
        }
        if !props.highlights.is_empty() {
            widget = widget.with_highlights(props.highlights.clone());
        }
        if let Some(handler) = &props.on_emoji_click {
            widget = widget.with_on_emoji_click(handler.clone());
        }
        if props.emoji_in_box {
            widget = widget.with_emoji_in_box();
        }
        if let Some((color, weight)) = props.link_style {
            widget = widget.with_link_style(color, weight);
        }
        if !props.link_colors.is_empty() {
            widget = widget.with_link_colors(props.link_colors.clone());
        }
        if let Some(handler) = &props.on_link {
            widget = widget.with_on_link(handler.clone());
        }
        if props.inert {
            widget = widget.with_inert();
        }
        if let (true, Some((label, tooltip))) = (leaf.edited, &props.edited) {
            widget = widget.with_edited(label.clone(), tooltip.clone());
        }
        let node = Node::new("span", widget);
        match &props.image_tick {
            Some(tick) => node.bind_image(tick.clone(), || None),
            None => node,
        }
    }

    fn next_key(&self) -> Option<u64> {
        let n = self.counter.get();
        self.counter.set(n + 1);
        self.props.cache_key.map(|key| {
            if n == 0 {
                key
            } else {
                key ^ n.wrapping_mul(0x9E37_79B9_7F4A_7C15)
            }
        })
    }

    fn column(&self, blocks: &[Block], level: usize) -> Node {
        let mut column = boxed().style("display: flex; flex-direction: column");
        let mut previous_bottom = 0.0f32;
        for (index, block) in blocks.iter().enumerate() {
            let (top, bottom) = margins(block, index == 0 && level == 0);
            let gap = if index == 0 {
                top
            } else {
                top.max(previous_bottom)
            };
            let node = self.block(block, level, level == 0 && index + 1 == blocks.len());
            column = column.child(if gap > 0.0 {
                node.style(format!("margin-top: {gap}px"))
            } else {
                node
            });
            previous_bottom = bottom;
        }
        if previous_bottom > 0.0 {
            column = column.style(format!("padding-bottom: {previous_bottom}px"));
        }
        column
    }

    fn block(&self, block: &Block, level: usize, last: bool) -> Node {
        let props = &self.props;
        match block {
            Block::Text(text) => {
                let mut leaf = Leaf::plain(props);
                leaf.edited = last;
                self.leaf(text.clone(), leaf, self.next_key())
            }
            Block::Heading {
                level: heading,
                content,
            } => {
                let size = HEADING_SIZES[usize::from(*heading - 1)];

                let color = if props.color == TEXT_DEFAULT {
                    TEXT_STRONG
                } else {
                    props.color
                };
                self.leaf(
                    content.clone(),
                    Leaf {
                        size,
                        color,
                        weight: Some(Weight::BOLD),
                        line_height: Some(size * 1.375),
                        edited: false,
                    },
                    self.next_key(),
                )
            }
            Block::Subtext(content) => self.leaf(
                content.clone(),
                Leaf {
                    size: SUBTEXT_SIZE,
                    color: TEXT_SUBTLE,
                    weight: None,
                    line_height: Some(SUBTEXT_LINE_HEIGHT),
                    edited: false,
                },
                self.next_key(),
            ),
            Block::List(list) => self.list(list, level),
            Block::Quote(content) => self.quote(content),
        }
    }

    /// A bar flush with the text's left edge, the quoted text 16px in.
    fn quote(&self, content: &str) -> Node {
        let bar = boxed().style(format!(
            "width: {QUOTE_BAR_WIDTH}px; flex: none; border-radius: {QUOTE_BAR_WIDTH}px; background-color: {}",
            hex(QUOTE_BAR_COLOR)
        ));
        let text = self
            .leaf(content.to_string(), Leaf::plain(&self.props), self.next_key())
            .style("min-width: 0");
        boxed()
            .style("display: flex; flex-direction: row")
            .child(bar)
            .child(
                boxed()
                    .style(format!(
                        "flex: 1; min-width: 0; padding-left: {QUOTE_TEXT_INSET}px; padding-right: 8px"
                    ))
                    .child(text),
            )
    }

    fn list(&self, list: &List, level: usize) -> Node {
        let props = &self.props;
        let line_height = props.line_height.unwrap_or(props.size * 1.2);

        let digits = (list.start + list.items.len().saturating_sub(1) as u64)
            .to_string()
            .len() as f32;
        let indent = if list.ordered {
            0.4 * props.size + digits * 0.6 * props.size
        } else {
            LIST_INDENT
        };
        let mut rows = boxed().style("display: flex; flex-direction: column");
        for (index, item) in list.items.iter().enumerate() {
            let marker = if list.ordered {
                TextProps::new(format!("{}.", list.start + index as u64), props.size)
                    .color(props.color)
                    .line_height(line_height)
                    .build()
                    .style(format!("white-space: nowrap; margin-right: {NUMBER_GAP}px"))
            } else {
                let look = if level == 0 {
                    format!("background-color: {}", hex(props.color))
                } else {
                    format!("border: 1px solid {}", hex(props.color))
                };
                let top = (line_height - BULLET_SIZE) / 2.0 + 1.0;
                boxed().style(format!("width: {BULLET_SIZE}px; height: {BULLET_SIZE}px; flex: none; border-radius: {}px; margin-top: {top}px; margin-right: {BULLET_GAP}px; {look}", BULLET_SIZE / 2.0))
            };
            let marker_box = boxed().style(format!("display: flex; flex-direction: row; justify-content: flex-end; flex: none; width: {indent}px")).child(marker);
            let content = self.column(item, level + 1).style("flex: 1; min-width: 0");
            rows = rows.child(
                boxed()
                    .style("display: flex; flex-direction: row")
                    .child(marker_box)
                    .child(content),
            );
        }
        rows
    }
}

fn margins(block: &Block, first: bool) -> (f32, f32) {
    match block {
        Block::Text(_) | Block::Subtext(_) => (0.0, 0.0),
        Block::Heading { level, .. } => (
            if !first {
                16.0
            } else if *level <= 2 {
                8.0
            } else {
                4.0
            },
            8.0,
        ),
        Block::List(_) => (4.0, 0.0),
        Block::Quote(_) => (0.0, 0.0),
    }
}
