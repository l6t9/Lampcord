use crate::app;
use crate::colors::{
    BACKGROUND_MOD_NORMAL, BACKGROUND_MOD_SUBTLE, BRAND, MENTION_BACKGROUND, TEXT_ANCHOR,
    TEXT_MENTION,
};
use crate::colors;
use crate::components::hover_tooltip;
use crate::paint as crate_paint;
use crate::selection;
use crate::style;
use crate::timestamp;
use skia_safe::surfaces;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Range;
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::Duration;
use std::time::Instant;

use cosmic_text::fontdb::Weight as FdWeight;
use cosmic_text::{
    Attrs, Buffer, Color as CosmicColor, Family, FontSystem, Metrics, Shaping, Style as FdStyle,
    UnderlineStyle, Wrap,
};
use skia_safe::{Canvas, Color, Image, Paint, RRect, Rect};
use taffy::{AvailableSpace, Size};
use unicode_segmentation::UnicodeSegmentation;

use super::markdown::{self, MarkdownFeatures, SpanAttrs};
use crate::event::{CursorIcon, EventResult, MouseButton, UiEvent};
use crate::font_system::{GG_SANS_MONO_FAMILY, gg_sans_family};
use crate::glyph_cache::{GlyphCache, SkiaDecorationRenderer, paint_glyph};
use crate::style::ComputedStyle;
use crate::text_cache::{MeasureCache, TextCacheKey};
use crate::widget::{AbsoluteRect, Widget};
use skia_safe::font_style::Weight;

pub type EmojiProvider = fn(&str) -> Option<(Image, (f32, f32, f32, f32))>;

pub type CustomEmojiResolver = Rc<dyn Fn(u64, bool) -> Option<EmojiFrame>>;

#[derive(Clone)]
pub struct EmojiFrame {
    pub image: Image,

    pub next_frame_ms: Option<u32>,
}

pub(crate) fn custom_emoji_tokens(text: &str) -> Vec<(Range<usize>, u64, bool)> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = text[from..].find('<') {
        let start = from + rel;
        from = start + 1;
        if markdown::is_escaped(text, start) {
            continue;
        }
        let mut i = start + 1;
        let animated = bytes.get(i) == Some(&b'a');
        if animated {
            i += 1;
        }
        if bytes.get(i) != Some(&b':') {
            continue;
        }
        i += 1;
        let name_start = i;
        while bytes
            .get(i)
            .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
        {
            i += 1;
        }
        if !(2..=32).contains(&(i - name_start)) || bytes.get(i) != Some(&b':') {
            continue;
        }
        i += 1;
        let id_start = i;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if i == id_start || bytes.get(i) != Some(&b'>') {
            continue;
        }
        let Ok(id) = text[id_start..i].parse::<u64>() else {
            continue;
        };
        out.push((start..i + 1, id, animated));
        from = i + 1;
    }
    out
}

type EmojiTile = (Option<Image>, Rect, Option<(u64, bool)>, String);

pub(crate) const EMOJI_PLACEHOLDER: &str = "\u{FFFC}";

pub(crate) const EMOJI_SCALE: f32 = 1.375;

const MARKDOWN_CODE_FONT_SCALE: f32 = 0.85;
const QUOTE_BAR_COLOR: Color = Color::from_argb(255, 0x47, 0x48, 0x51);
const QUOTE_BAR_WIDTH: f32 = 4.0;

pub(crate) const SELECTION_BACKGROUND: Color = Color::from_argb(0x80, 0x58, 0x65, 0xf2);

pub(crate) fn encode_cursor(line: usize, index: usize) -> u64 {
    ((line as u64) << 32) | (index as u64 & 0xffff_ffff)
}

pub(crate) fn decode_cursor(offset: u64) -> (usize, usize) {
    ((offset >> 32) as usize, (offset & 0xffff_ffff) as usize)
}

pub(crate) fn selection_span(
    run: &cosmic_text::LayoutRun,
    range: (u64, u64),
) -> Option<(f32, f32)> {
    let ((start_line, start_index), (end_line, end_index)) =
        (decode_cursor(range.0), decode_cursor(range.1));
    if range.0 == range.1 || run.line_i < start_line || run.line_i > end_line {
        return None;
    }
    let from = if run.line_i == start_line {
        start_index
    } else {
        0
    };
    let to = if run.line_i == end_line {
        end_index
    } else {
        usize::MAX
    };
    let mut span: Option<(f32, f32)> = None;
    for glyph in run.glyphs {
        if glyph.start < to && glyph.end > from {
            let (left, right) = (glyph.x, glyph.x + glyph.w);
            span = Some(match span {
                Some((l, r)) => (l.min(left), r.max(right)),
                None => (left, right),
            });
        }
    }
    span.map(|(l, r)| (l, r - l))
}

const HIGHLIGHT_BACKGROUND: Color = Color::from_argb(0x4d, 0xfc, 0xb9, 0x32);

const META_NONE: usize = 0;
const META_CODE: usize = 1;
const META_SPOILER: usize = 2;
const META_MENTION: usize = 3;

const META_HIGHLIGHT: usize = 4;

const META_TIMESTAMP_BASE: usize = 1 << 28;

/// Tooltip key of the `(edited)` label. It shares the timestamp tooltip but not its pill.
const EDITED_KEY: usize = 1 << 27;
const META_EDITED: usize = META_TIMESTAMP_BASE + EDITED_KEY;

const EDITED_SIZE: f32 = 10.0;

const META_EMOJI_BASE: usize = 16;

const META_LINK_BASE: usize = 1 << 24;

const META_MENTION_BASE: usize = 1 << 26;
const META_ROLE_BASE: usize = 1 << 27;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MentionTarget {
    User(u64),
    Channel(u64),
    Role(u64),
}

fn is_emoji_meta(meta: usize) -> bool {
    (META_EMOJI_BASE..META_LINK_BASE).contains(&meta)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmojiKind {
    Unicode(String),

    Custom {
        id: u64,
        name: String,
        animated: bool,
    },
}

#[derive(Clone, Debug)]
pub struct EmojiClick {
    pub kind: EmojiKind,
    pub rect: AbsoluteRect,
}

pub type EmojiClickHandler = Rc<dyn Fn(EmojiClick)>;

thread_local! {
    static CHANNEL_NAME_RESOLVER: RefCell<Option<Rc<dyn Fn(u64) -> Option<String>>>> =
        const { RefCell::new(None) };
}

/// Names a channel mentioned as `<#id>` that the text's own map doesn't have. A channel it
/// can't name is shown as `#unknown`.
pub fn set_channel_name_resolver(resolver: Rc<dyn Fn(u64) -> Option<String>>) {
    CHANNEL_NAME_RESOLVER.with(|r| *r.borrow_mut() = Some(resolver));
}

fn resolve_channel_name(id: u64) -> Option<String> {
    let resolver = CHANNEL_NAME_RESOLVER.with(|r| r.borrow().clone())?;
    resolver(id)
}

/// Opens a clicked link. The flag is `true` when the link's label isn't its address.
static LINK_HANDLER: OnceLock<fn(&str, bool)> = OnceLock::new();

pub fn set_link_handler(handler: fn(&str, bool)) {
    let _ = LINK_HANDLER.set(handler);
}

enum RunKind {
    Markdown(SpanAttrs),

    Mention(String, Option<MentionTarget>),

    Role(String, Option<Color>, u64),

    CustomEmoji(u64, bool),

    Timestamp(String, String),
}

struct Run {
    range: Range<usize>,
    kind: RunKind,

    href: Option<Range<usize>>,
}

pub struct MessageTextWidget {
    text: String,
    font_size: f32,
    color: Color,
    font_system: Rc<RefCell<FontSystem>>,
    glyph_cache: Rc<GlyphCache>,
    mentions: Rc<HashMap<u64, String>>,
    channels: Rc<HashMap<u64, String>>,

    roles: Rc<HashMap<u64, (String, Option<Color>)>>,

    special_mentions: bool,

    on_mention: Option<Rc<dyn Fn(MentionTarget, AbsoluteRect)>>,

    mention_targets: RefCell<HashMap<usize, MentionTarget>>,

    timestamp_tooltips: RefCell<HashMap<usize, String>>,

    hovered_timestamp: Cell<Option<usize>>,

    hovered_mention: Cell<Option<usize>>,

    pressed: Cell<bool>,

    /// Set once a spoiler in this text was clicked.
    spoilers_shown: Cell<bool>,

    /// Link starts whose label differs from the address they open.
    masked_links: RefCell<HashSet<usize>>,

    /// Draws like normal but takes no clicks: links, mentions, emoji and spoilers stay as shown.
    inert: bool,

    /// Trailing `(edited)` label and the tooltip with the edit time.
    edited: Option<(String, String)>,
    emoji_provider: Option<EmojiProvider>,

    custom_emoji: Option<CustomEmojiResolver>,

    line_height: Option<f32>,

    emoji_scale: f32,

    line_clamp: Option<u32>,

    markdown: MarkdownFeatures,

    base_weight: FdWeight,

    link_style: Option<(Color, FdWeight)>,

    /// Colors for particular link targets, over `link_style`'s.
    link_colors: Vec<(String, Color)>,

    on_link: Option<Rc<dyn Fn(&str, AbsoluteRect)>>,

    cache: RefCell<Vec<(u32, Buffer, Vec<EmojiTile>)>>,
    theme_seen: Cell<u64>,

    layer: RefCell<Option<(super::text::LayerKey, Image)>>,

    last_measured_width: Cell<Option<f32>>,

    measure_cache: Option<Rc<MeasureCache>>,
    cache_key: Option<TextCacheKey>,

    quoted_lines: Vec<bool>,

    highlights: Vec<String>,

    links: RefCell<HashMap<usize, usize>>,

    hovered_link: Cell<Option<usize>>,

    on_emoji_click: Option<EmojiClickHandler>,

    emoji_in_box: bool,
}

impl MessageTextWidget {
    pub fn new(
        text: impl Into<String>,
        font_size: f32,
        color: Color,
        font_system: Rc<RefCell<FontSystem>>,
        glyph_cache: Rc<GlyphCache>,
        mentions: Rc<HashMap<u64, String>>,
        channels: Rc<HashMap<u64, String>>,
    ) -> Self {
        let text = text.into();
        let quoted_lines = if text.is_empty() {
            Vec::new()
        } else {
            markdown::quoted_lines(&text)
        };
        Self {
            text,
            font_size,
            color,
            font_system,
            glyph_cache,
            mentions,
            channels,
            roles: Rc::default(),
            special_mentions: false,
            on_mention: None,
            mention_targets: RefCell::new(HashMap::new()),
            timestamp_tooltips: RefCell::new(HashMap::new()),
            hovered_timestamp: Cell::new(None),
            hovered_mention: Cell::new(None),
            pressed: Cell::new(false),
            spoilers_shown: Cell::new(false),
            masked_links: RefCell::new(HashSet::new()),
            inert: false,
            edited: None,
            emoji_provider: None,
            custom_emoji: None,
            line_height: None,
            emoji_scale: EMOJI_SCALE,
            line_clamp: None,
            markdown: MarkdownFeatures::CHAT,
            base_weight: FdWeight::NORMAL,
            link_style: None,
            link_colors: Vec::new(),
            on_link: None,
            cache: RefCell::new(Vec::new()),
            theme_seen: Cell::new(crate::theme_generation()),
            layer: RefCell::new(None),
            last_measured_width: Cell::new(None),
            measure_cache: None,
            cache_key: None,
            quoted_lines,
            highlights: Vec::new(),
            links: RefCell::new(HashMap::new()),
            hovered_link: Cell::new(None),
            on_emoji_click: None,
            emoji_in_box: false,
        }
    }

    pub fn with_link_style(mut self, color: Color, weight: u16) -> Self {
        self.link_style = Some((color, FdWeight(weight)));
        self
    }

    pub fn with_link_colors(mut self, colors: Vec<(String, Color)>) -> Self {
        self.link_colors = colors;
        self
    }

    fn link_color(&self, url: &str) -> Color {
        self.link_colors
            .iter()
            .find(|(target, _)| target == url)
            .map(|(_, color)| *color)
            .or(self.link_style.map(|(color, _)| color))
            .unwrap_or(TEXT_ANCHOR)
    }

    pub fn with_inert(mut self) -> Self {
        self.inert = true;
        self
    }

    pub fn with_edited(mut self, label: String, tooltip: String) -> Self {
        self.edited = Some((label, tooltip));
        self
    }

    pub fn with_on_link(mut self, handler: Rc<dyn Fn(&str, AbsoluteRect)>) -> Self {
        self.on_link = Some(handler);
        self
    }

    pub fn with_emoji_in_box(mut self) -> Self {
        self.emoji_in_box = true;
        self
    }

    pub fn with_on_emoji_click(mut self, handler: EmojiClickHandler) -> Self {
        self.on_emoji_click = Some(handler);
        self
    }

    fn emoji_at(&self, rect: AbsoluteRect, x: f32, y: f32) -> Option<EmojiClick> {
        self.on_emoji_click.as_ref()?;
        let wrap_width = self.paint_width(rect.width, rect.height);
        self.with_buffer(wrap_width, |buffer, tiles| {
            buffer.layout_runs().find_map(|run| {
                let (top, bottom) = (
                    rect.y + run.line_top,
                    rect.y + run.line_top + run.line_height,
                );
                if y < top || y >= bottom {
                    return None;
                }
                let glyph = run.glyphs.iter().find(|g| {
                    is_emoji_meta(g.metadata) && x >= rect.x + g.x && x < rect.x + g.x + g.w
                })?;
                let (_, _, custom, label) = tiles.get(glyph.metadata - META_EMOJI_BASE)?;
                let kind = match custom {
                    Some((id, animated)) => EmojiKind::Custom {
                        id: *id,
                        name: label.trim_matches(':').to_string(),
                        animated: *animated,
                    },
                    None => EmojiKind::Unicode(label.clone()),
                };
                Some(EmojiClick {
                    kind,
                    rect: AbsoluteRect {
                        x: rect.x + glyph.x,
                        y: top,
                        width: glyph.w,
                        height: run.line_height,
                    },
                })
            })
        })
    }

    pub fn with_measure_cache(mut self, cache: Rc<MeasureCache>, key: Option<u64>) -> Self {
        self.cache_key = Some(
            key.map(TextCacheKey::Stable)
                .unwrap_or_else(|| TextCacheKey::from_text(&self.text)),
        );
        self.measure_cache = Some(cache);
        self
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.quoted_lines = if self.text.is_empty() {
            Vec::new()
        } else {
            markdown::quoted_lines(&self.text)
        };
        self.cache.borrow_mut().clear();
        self.layer.borrow_mut().take();
        self.cache_key = self.cache_key.map(|_| TextCacheKey::from_text(&self.text));
    }

    pub fn with_markdown(mut self, features: MarkdownFeatures) -> Self {
        self.markdown = features;
        self
    }

    pub fn with_weight(mut self, weight: Weight) -> Self {
        self.base_weight = FdWeight(*weight as u16);
        self
    }

    pub fn with_emoji_scale(mut self, scale: f32) -> Self {
        self.emoji_scale = scale;
        self
    }

    pub fn with_line_height(mut self, line_height: f32) -> Self {
        self.line_height = Some(line_height);
        self
    }

    pub fn with_highlights(mut self, terms: impl IntoIterator<Item = String>) -> Self {
        self.highlights = terms
            .into_iter()
            .filter(|t| !t.is_empty())
            .map(|t| t.to_ascii_lowercase())
            .collect();
        self
    }

    fn split_highlights(&self, text: &str) -> Vec<(Range<usize>, bool)> {
        let lower = text.to_ascii_lowercase();
        let mut hits: Vec<Range<usize>> = Vec::new();
        for term in &self.highlights {
            let mut from = 0;
            while let Some(rel) = lower[from..].find(term.as_str()) {
                let start = from + rel;
                hits.push(start..start + term.len());
                from = start + term.len();
            }
        }
        hits.sort_by_key(|r| r.start);
        let mut pieces = Vec::new();
        let mut at = 0;
        for hit in hits {
            if hit.start < at {
                continue;
            }
            if at < hit.start {
                pieces.push((at..hit.start, false));
            }
            pieces.push((hit.clone(), true));
            at = hit.end;
        }
        if at < text.len() {
            pieces.push((at..text.len(), false));
        }
        pieces
    }

    pub fn with_emoji_provider(mut self, provider: EmojiProvider) -> Self {
        self.emoji_provider = Some(provider);
        self
    }

    pub fn refresh_custom_emoji(&mut self) -> bool {
        let resolver = self.custom_emoji.clone();
        let provider = self.emoji_provider;
        let mut changed = false;
        for (_, _, tiles) in self.cache.get_mut().iter_mut() {
            for (image, rect, custom, label) in tiles.iter_mut() {
                if custom.is_none() {
                    if let Some((new, (x, y, w, h))) = provider.and_then(|provider| provider(label))
                    {
                        if image.as_ref().map(|i| i.unique_id()) != Some(new.unique_id()) {
                            *rect = Rect::from_xywh(x, y, w, h);
                            *image = Some(new);
                            changed = true;
                        }
                    }
                    continue;
                }
                let Some(resolver) = resolver.as_ref() else {
                    continue;
                };
                if image.is_some() {
                    continue;
                }
                let Some((id, animated)) = *custom else {
                    continue;
                };
                if let Some(loaded) = resolver(id, animated) {
                    *rect =
                        Rect::from_wh(loaded.image.width() as f32, loaded.image.height() as f32);
                    *image = Some(loaded.image);
                    changed = true;
                }
            }
        }
        changed
    }

    pub fn with_role_mentions(mut self, roles: Rc<HashMap<u64, (String, Option<Color>)>>) -> Self {
        self.roles = roles;
        self
    }

    pub fn with_on_mention(mut self, handler: Rc<dyn Fn(MentionTarget, AbsoluteRect)>) -> Self {
        self.on_mention = Some(handler);
        self
    }

    pub fn with_special_mentions(mut self, on: bool) -> Self {
        self.special_mentions = on;
        self
    }

    pub fn with_custom_emoji(mut self, resolver: CustomEmojiResolver) -> Self {
        self.custom_emoji = Some(resolver);
        self
    }

    fn scan_custom_emoji(text: &str, out: &mut Vec<Run>) {
        out.extend(
            custom_emoji_tokens(text)
                .into_iter()
                .map(|(range, id, animated)| Run {
                    range,
                    kind: RunKind::CustomEmoji(id, animated),
                    href: None,
                }),
        );
    }

    fn scan_mentions(&self, text: &str) -> Vec<Run> {
        let mut matches = Vec::new();
        Self::scan_kind(text, '@', &self.mentions, &mut matches);
        Self::scan_kind(text, '#', &self.channels, &mut matches);
        if self.custom_emoji.is_some() {
            Self::scan_custom_emoji(text, &mut matches);
        }
        self.scan_roles(text, &mut matches);
        for (range, unix, letter) in timestamp::tokens(text) {
            if let Some(shown) = timestamp::format(unix, letter) {
                let full =
                    timestamp::format(unix, Some('F')).unwrap_or_else(|| shown.clone());
                matches.push(Run {
                    range,
                    kind: RunKind::Timestamp(shown, full),
                    href: None,
                });
            }
        }
        if self.special_mentions {
            Self::scan_everyone(text, &mut matches);
        }
        matches.sort_by_key(|m: &Run| m.range.start);

        let mut merged: Vec<Run> = Vec::new();
        for m in matches {
            if merged
                .last()
                .is_none_or(|prev: &Run| prev.range.end <= m.range.start)
            {
                merged.push(m);
            }
        }
        merged
    }

    fn scan_roles(&self, text: &str, out: &mut Vec<Run>) {
        let mut search_from = 0;
        while let Some(rel) = text[search_from..].find("<@&") {
            let start = search_from + rel;
            let digits_start = start + 3;
            let digits_len = text[digits_start..]
                .bytes()
                .take_while(u8::is_ascii_digit)
                .count();
            search_from = digits_start + digits_len;
            if digits_len == 0
                || !text[search_from..].starts_with('>')
                || markdown::is_escaped(text, start)
            {
                continue;
            }
            let Ok(id) = text[digits_start..search_from].parse::<u64>() else {
                continue;
            };
            let (name, color) = match self.roles.get(&id) {
                Some((name, color)) => (format!("@{name}"), *color),
                None => ("@deleted-role".to_string(), None),
            };
            out.push(Run {
                range: start..search_from + 1,
                kind: RunKind::Role(name, color, id),
                href: None,
            });
        }
    }

    fn scan_everyone(text: &str, out: &mut Vec<Run>) {
        for word in ["@everyone", "@here"] {
            let mut search_from = 0;
            while let Some(rel) = text[search_from..].find(word) {
                let start = search_from + rel;
                let end = start + word.len();
                search_from = end;
                let before_ok = text[..start]
                    .chars()
                    .next_back()
                    .is_none_or(|c| !c.is_alphanumeric() && c != '<' && c != '\\');
                let after_ok = text[end..]
                    .chars()
                    .next()
                    .is_none_or(|c| !c.is_alphanumeric() && c != '_');
                if before_ok && after_ok {
                    out.push(Run {
                        range: start..end,
                        kind: RunKind::Mention(word.to_string(), None),
                        href: None,
                    });
                }
            }
        }
    }

    fn scan_kind(text: &str, trigger: char, names: &HashMap<u64, String>, out: &mut Vec<Run>) {
        let open = format!("<{trigger}");
        let mut search_from = 0;
        while let Some(rel) = text[search_from..].find(open.as_str()) {
            let start = search_from + rel;
            if markdown::is_escaped(text, start) {
                search_from = start + 1;
                continue;
            }

            let bang = if trigger == '@' && text[start + open.len()..].starts_with('!') {
                1
            } else {
                0
            };
            let digits_start = start + open.len() + bang;
            let digits_len = text[digits_start..]
                .bytes()
                .take_while(u8::is_ascii_digit)
                .count();
            search_from = digits_start + digits_len;
            if digits_len == 0 || !text[digits_start + digits_len..].starts_with('>') {
                continue;
            }
            let Ok(id) = text[digits_start..digits_start + digits_len].parse::<u64>() else {
                continue;
            };
            let end = digits_start + digits_len + 1;
            let display = match names.get(&id) {
                Some(name) => format!("{trigger}{name}"),
                None if trigger == '#' => {
                    format!("#{}", resolve_channel_name(id).unwrap_or_else(|| "unknown".into()))
                }
                None => format!("{trigger}{id}"),
            };
            out.push(Run {
                range: start..end,
                kind: RunKind::Mention(
                    display,
                    Some(if trigger == '#' {
                        MentionTarget::Channel(id)
                    } else {
                        MentionTarget::User(id)
                    }),
                ),
                href: None,
            });
        }
    }

    fn features(&self) -> MarkdownFeatures {
        MarkdownFeatures {
            masked_links: true,
            subtext: false,
            ..self.markdown
        }
    }

    fn build_runs(&self, text: &str) -> Vec<Run> {
        let mentions = self.scan_mentions(text);
        let mut runs = Vec::new();
        let mut pos = 0;

        for mention in mentions {
            if pos < mention.range.start {
                for span in markdown::tokenize(&text[pos..mention.range.start], self.features()) {
                    if span.attrs.delimiter {
                        continue;
                    }
                    runs.push(Run {
                        range: pos + span.range.start..pos + span.range.end,
                        kind: RunKind::Markdown(span.attrs),
                        href: span.href.map(|h| pos + h.start..pos + h.end),
                    });
                }
            }
            pos = mention.range.end;
            runs.push(mention);
        }
        if pos < text.len() {
            for span in markdown::tokenize(&text[pos..], self.features()) {
                if span.attrs.delimiter {
                    continue;
                }
                runs.push(Run {
                    range: pos + span.range.start..pos + span.range.end,
                    kind: RunKind::Markdown(span.attrs),
                    href: span.href.map(|h| pos + h.start..pos + h.end),
                });
            }
        }
        runs
    }

    fn markdown_attrs(&self, attrs: &SpanAttrs) -> Attrs<'static> {
        let weight = match (attrs.bold, attrs.link, self.link_style) {
            (true, _, _) => FdWeight::BOLD,
            (false, true, Some((_, weight))) => weight,
            _ => self.base_weight,
        };
        let style = if attrs.italic {
            FdStyle::Italic
        } else {
            FdStyle::Normal
        };
        let mut a = Attrs::new().weight(weight).style(style);

        if attrs.code {
            a = a
                .family(Family::Name(GG_SANS_MONO_FAMILY))
                .metrics(Metrics::new(
                    self.font_size * MARKDOWN_CODE_FONT_SCALE,
                    self.font_size * MARKDOWN_CODE_FONT_SCALE * 1.2,
                ))
                .metadata(META_CODE);
        } else {
            a = a.family(Family::Name(gg_sans_family(weight, attrs.italic)));
            if attrs.spoiler {
                a = a.metadata(META_SPOILER);
            }
        }

        a = a.color(if attrs.link {
            to_cosmic_color(colors::live(
                self.link_style.map_or(TEXT_ANCHOR, |(color, _)| color),
            ))
        } else {
            to_cosmic_color(colors::live(self.color))
        });
        if attrs.strikethrough {
            a = a.strikethrough();
        }
        if attrs.underline {
            a = a.underline(UnderlineStyle::Single);
        }
        a
    }

    fn mention_attrs(&self) -> Attrs<'static> {
        Attrs::new()
            .family(Family::Name(gg_sans_family(FdWeight::MEDIUM, false)))
            .weight(FdWeight::MEDIUM)
            .color(to_cosmic_color(TEXT_MENTION))
            .metadata(META_MENTION)
    }

    fn pad_span(&self, pill: &Attrs<'static>, size: f32) -> (&'static str, Attrs<'static>) {
        (
            "\u{00A0}",
            pill.clone().metrics(Metrics::new(
                size,
                self.line_height.unwrap_or(self.font_size * 1.2),
            )),
        )
    }

    fn emoji_span(
        &self,
        base_attrs: &Attrs<'static>,
        index: usize,
    ) -> (&'static str, Attrs<'static>) {
        let emoji_font_size = self.font_size * self.emoji_scale;

        let emoji_line_height = self
            .line_height
            .map_or(emoji_font_size * 1.2, |lh| lh.max(emoji_font_size));
        (
            EMOJI_PLACEHOLDER,
            base_attrs
                .clone()
                .metrics(Metrics::new(emoji_font_size, emoji_line_height))
                .metadata(META_EMOJI_BASE + index),
        )
    }

    fn push_text_with_emoji<'a>(
        &self,
        text: &'a str,
        base_attrs: &Attrs<'static>,
        spans: &mut Vec<(&'a str, Attrs<'static>)>,
        emoji_images: &mut Vec<EmojiTile>,
    ) {
        let Some(provider) = self.emoji_provider else {
            spans.push((text, base_attrs.clone()));
            return;
        };
        let mut plain_start = 0usize;
        for (offset, grapheme) in text.grapheme_indices(true) {
            let Some((image, (x, y, w, h))) = provider(grapheme) else {
                continue;
            };
            if plain_start < offset {
                spans.push((&text[plain_start..offset], base_attrs.clone()));
            }
            let index = emoji_images.len();
            emoji_images.push((
                Some(image.clone()),
                Rect::from_xywh(x, y, w, h),
                None,
                grapheme.to_string(),
            ));

            spans.push(self.emoji_span(base_attrs, index));
            plain_start = offset + grapheme.len();
        }
        if plain_start < text.len() {
            spans.push((&text[plain_start..], base_attrs.clone()));
        }
    }

    fn shape_buffer(&self, wrap: Wrap, width: Option<f32>) -> (Buffer, Vec<EmojiTile>) {
        let shaped = self.shape_buffer_of(&self.text, wrap, width);
        let Some(lines) = self.line_clamp.filter(|_| wrap != Wrap::None) else {
            return shaped;
        };
        if shaped.0.layout_runs().count() <= lines as usize {
            return shaped;
        }

        let boundaries: Vec<usize> = self.text.char_indices().map(|(i, _)| i).collect();
        let truncated = |k: usize| {
            format!(
                "{}\u{2026}",
                self.text[..boundaries.get(k).copied().unwrap_or(self.text.len())].trim_end()
            )
        };
        let (mut lo, mut hi) = (0usize, boundaries.len());
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            if self
                .shape_buffer_of(&truncated(mid), wrap, width)
                .0
                .layout_runs()
                .count()
                <= lines as usize
            {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        self.shape_buffer_of(&truncated(lo), wrap, width)
    }

    fn shape_buffer_of(
        &self,
        text: &str,
        wrap: Wrap,
        width: Option<f32>,
    ) -> (Buffer, Vec<EmojiTile>) {
        let mut font_system = self.font_system.borrow_mut();
        let metrics = Metrics::new(
            self.font_size,
            self.line_height.unwrap_or(self.font_size * 1.2),
        );
        let mut buffer = Buffer::new(&mut font_system, metrics);
        buffer.set_wrap(wrap);
        buffer.set_size(width, None);

        let hidden = markdown::quote_prefix_ranges(text);
        let runs = self.build_runs(text);

        let mut spans: Vec<(&str, Attrs)> = Vec::new();
        let mut emoji_images: Vec<EmojiTile> = Vec::new();
        let pad_size = pill_pad_font_size(&mut font_system);
        for run in &runs {
            match &run.kind {
                RunKind::Mention(display, target) => {
                    if hidden
                        .iter()
                        .any(|h| h.start <= run.range.start && run.range.end <= h.end)
                    {
                        continue;
                    }
                    let mut attrs = self.mention_attrs();
                    if let Some(target) = target {
                        attrs = attrs.metadata(META_MENTION_BASE + run.range.start);
                        self.mention_targets
                            .borrow_mut()
                            .insert(run.range.start, *target);
                    }
                    spans.push(self.pad_span(&attrs, pad_size));
                    spans.push((display.as_str(), attrs.clone()));
                    spans.push(self.pad_span(&attrs, pad_size));
                }
                RunKind::Role(name, color, id) => {
                    if hidden
                        .iter()
                        .any(|h| h.start <= run.range.start && run.range.end <= h.end)
                    {
                        continue;
                    }
                    self.mention_targets
                        .borrow_mut()
                        .insert(run.range.start, MentionTarget::Role(*id));
                    let attrs = self
                        .mention_attrs()
                        .metadata(META_ROLE_BASE + run.range.start)
                        .color(to_cosmic_color(color.unwrap_or(TEXT_MENTION)));
                    spans.push(self.pad_span(&attrs, pad_size));
                    spans.push((name.as_str(), attrs.clone()));
                    spans.push(self.pad_span(&attrs, pad_size));
                }
                RunKind::Timestamp(shown, full) => {
                    if hidden
                        .iter()
                        .any(|h| h.start <= run.range.start && run.range.end <= h.end)
                    {
                        continue;
                    }
                    self.timestamp_tooltips
                        .borrow_mut()
                        .insert(run.range.start, full.clone());
                    let attrs = Attrs::new()
                        .family(Family::Name(gg_sans_family(self.base_weight, false)))
                        .weight(self.base_weight)
                        .color(to_cosmic_color(colors::live(self.color)))
                        .metadata(META_TIMESTAMP_BASE + run.range.start);
                    spans.push(self.pad_span(&attrs, pad_size));
                    spans.push((shown.as_str(), attrs.clone()));
                    spans.push(self.pad_span(&attrs, pad_size));
                }
                RunKind::CustomEmoji(id, animated) => {
                    if hidden
                        .iter()
                        .any(|h| h.start <= run.range.start && run.range.end <= h.end)
                    {
                        continue;
                    }
                    let image = self
                        .custom_emoji
                        .as_ref()
                        .and_then(|resolve| resolve(*id, *animated))
                        .map(|frame| frame.image);
                    let tile = image.as_ref().map_or(Rect::default(), |i| {
                        Rect::from_wh(i.width() as f32, i.height() as f32)
                    });
                    let index = emoji_images.len();
                    let label = text[run.range.clone()]
                        .trim_matches(['<', '>'])
                        .split(':')
                        .nth(1)
                        .map(|name| format!(":{name}:"))
                        .unwrap_or_default();
                    emoji_images.push((image, tile, Some((*id, *animated)), label));
                    let base = Attrs::new()
                        .family(Family::Name(gg_sans_family(self.base_weight, false)))
                        .weight(self.base_weight)
                        .color(to_cosmic_color(colors::live(self.color)));
                    spans.push(self.emoji_span(&base, index));
                }
                RunKind::Markdown(attrs) => {
                    let mut run_attrs = self.markdown_attrs(attrs);
                    if let Some(href) = &run.href {
                        self.links.borrow_mut().insert(href.start, href.end);
                        if run.range != *href {
                            self.masked_links.borrow_mut().insert(href.start);
                        }
                        run_attrs = run_attrs.metadata(META_LINK_BASE + href.start);
                        if !self.link_colors.is_empty() {
                            let color = self.link_color(&text[href.clone()]);
                            run_attrs =
                                run_attrs.color(to_cosmic_color(colors::live(color)));
                        }
                    }
                    for visible in markdown::visible_subranges(run.range.clone(), &hidden) {
                        if visible.is_empty() {
                            continue;
                        }

                        if !self.highlights.is_empty() {
                            let base = visible.start;
                            for (piece, hit) in self.split_highlights(&text[visible.clone()]) {
                                let piece_attrs = if hit {
                                    run_attrs.clone().metadata(META_HIGHLIGHT)
                                } else {
                                    run_attrs.clone()
                                };
                                let piece_text = &text[base + piece.start..base + piece.end];
                                if attrs.code || self.emoji_provider.is_none() {
                                    spans.push((piece_text, piece_attrs));
                                } else {
                                    self.push_text_with_emoji(
                                        piece_text,
                                        &piece_attrs,
                                        &mut spans,
                                        &mut emoji_images,
                                    );
                                }
                            }
                            continue;
                        }
                        if attrs.code || self.emoji_provider.is_none() {
                            spans.push((&text[visible.clone()], run_attrs.clone()));
                            continue;
                        }
                        self.push_text_with_emoji(
                            &text[visible.clone()],
                            &run_attrs,
                            &mut spans,
                            &mut emoji_images,
                        );
                    }
                }
            }
        }

        let default_attrs = Attrs::new()
            .family(Family::Name(gg_sans_family(self.base_weight, false)))
            .weight(self.base_weight)
            .color(to_cosmic_color(colors::live(self.color)));
        if let Some((label, tooltip)) = &self.edited {
            self.timestamp_tooltips
                .borrow_mut()
                .insert(EDITED_KEY, tooltip.clone());
            let small = default_attrs
                .clone()
                .weight(FdWeight::NORMAL)
                .color(to_cosmic_color(colors::live(colors::TEXT_MUTED)))
                .metrics(Metrics::new(EDITED_SIZE, EDITED_SIZE))
                .metadata(META_EDITED);
            spans.push((" ", default_attrs.clone()));
            spans.push((label.as_str(), small));
        }
        buffer.set_rich_text(spans, &default_attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut font_system, false);
        (buffer, emoji_images)
    }

    fn link_start_at(&self, rect: AbsoluteRect, x: f32, y: f32) -> Option<usize> {
        if self.links.borrow().is_empty() {
            return None;
        }
        let wrap_width = self.paint_width(rect.width, rect.height);
        self.with_buffer(wrap_width, |buffer, _| {
            buffer.layout_runs().find_map(|run| {
                let (top, bottom) = (
                    rect.y + run.line_top,
                    rect.y + run.line_top + run.line_height,
                );
                if y < top || y >= bottom {
                    return None;
                }
                run.glyphs
                    .iter()
                    .find(|g| {
                        (META_LINK_BASE..META_MENTION_BASE).contains(&g.metadata)
                            && x >= rect.x + g.x
                            && x < rect.x + g.x + g.w
                    })
                    .map(|g| g.metadata - META_LINK_BASE)
            })
        })
    }

    fn link_rect_at(&self, rect: AbsoluteRect, x: f32, y: f32) -> Option<AbsoluteRect> {
        let start = self.link_start_at(rect, x, y)?;
        let wrap_width = self.paint_width(rect.width, rect.height);
        self.with_buffer(wrap_width, |buffer, _| {
            buffer.layout_runs().find_map(|run| {
                let top = rect.y + run.line_top;
                if y < top || y >= top + run.line_height {
                    return None;
                }
                let glyphs = run
                    .glyphs
                    .iter()
                    .filter(|g| g.metadata == META_LINK_BASE + start);
                let (left, right) = glyphs.fold((f32::MAX, f32::MIN), |(l, r), g| {
                    (l.min(g.x), r.max(g.x + g.w))
                });
                (left < right).then_some(AbsoluteRect {
                    x: rect.x + left,
                    y: top,
                    width: right - left,
                    height: run.line_height,
                })
            })
        })
    }

    fn mention_meta_at(&self, rect: AbsoluteRect, x: f32, y: f32) -> Option<usize> {
        if self.on_mention.is_none() || self.mention_targets.borrow().is_empty() {
            return None;
        }
        let wrap_width = self.paint_width(rect.width, rect.height);
        self.with_buffer(wrap_width, |buffer, _| {
            buffer.layout_runs().find_map(|run| {
                let top = rect.y + run.line_top;
                if y < top || y >= top + run.line_height {
                    return None;
                }
                run.glyphs
                    .iter()
                    .find(|g| {
                        (META_MENTION_BASE..META_TIMESTAMP_BASE).contains(&g.metadata)
                            && x >= rect.x + g.x
                            && x < rect.x + g.x + g.w
                    })
                    .map(|g| g.metadata)
            })
        })
    }

    fn timestamp_at(&self, rect: AbsoluteRect, x: f32, y: f32) -> Option<(usize, AbsoluteRect)> {
        if self.timestamp_tooltips.borrow().is_empty() {
            return None;
        }
        let wrap_width = self.paint_width(rect.width, rect.height);
        self.with_buffer(wrap_width, |buffer, _| {
            buffer.layout_runs().find_map(|run| {
                let top = rect.y + run.line_top;
                if y < top || y >= top + run.line_height {
                    return None;
                }
                let meta = run
                    .glyphs
                    .iter()
                    .find(|g| {
                        g.metadata >= META_TIMESTAMP_BASE
                            && x >= rect.x + g.x
                            && x < rect.x + g.x + g.w
                    })
                    .map(|g| g.metadata)?;
                let (left, right) = run
                    .glyphs
                    .iter()
                    .filter(|g| g.metadata == meta)
                    .fold((f32::MAX, f32::MIN), |(l, r), g| {
                        (l.min(g.x), r.max(g.x + g.w))
                    });
                Some((
                    meta - META_TIMESTAMP_BASE,
                    AbsoluteRect {
                        x: rect.x + left,
                        y: top,
                        width: right - left,
                        height: run.line_height,
                    },
                ))
            })
        })
    }

    fn mention_at(
        &self,
        rect: AbsoluteRect,
        x: f32,
        y: f32,
    ) -> Option<(MentionTarget, AbsoluteRect)> {
        if self.on_mention.is_none() || self.mention_targets.borrow().is_empty() {
            return None;
        }
        let wrap_width = self.paint_width(rect.width, rect.height);
        let (meta, found) = self.with_buffer(wrap_width, |buffer, _| {
            buffer.layout_runs().find_map(|run| {
                let (top, bottom) = (
                    rect.y + run.line_top,
                    rect.y + run.line_top + run.line_height,
                );
                if y < top || y >= bottom {
                    return None;
                }
                let meta = run
                    .glyphs
                    .iter()
                    .find(|g| {
                        (META_MENTION_BASE..META_TIMESTAMP_BASE).contains(&g.metadata)
                            && x >= rect.x + g.x
                            && x < rect.x + g.x + g.w
                    })
                    .map(|g| g.metadata)?;
                let (left, right) = run
                    .glyphs
                    .iter()
                    .filter(|g| g.metadata == meta)
                    .fold((f32::MAX, f32::MIN), |(l, r), g| {
                        (l.min(g.x), r.max(g.x + g.w))
                    });
                Some((
                    meta,
                    AbsoluteRect {
                        x: rect.x + left,
                        y: top,
                        width: right - left,
                        height: run.line_height,
                    },
                ))
            })
        })?;
        let start = if meta >= META_ROLE_BASE {
            meta - META_ROLE_BASE
        } else {
            meta - META_MENTION_BASE
        };
        let target = *self.mention_targets.borrow().get(&start)?;
        Some((target, found))
    }

    fn reveal_key(&self) -> Option<u64> {
        self.cache_key.map(|key| match key {
            TextCacheKey::Hash(id) | TextCacheKey::Stable(id) => id,
        })
    }

    fn spoilers_hidden(&self) -> bool {
        !self.spoilers_shown.get()
            && !self
                .reveal_key()
                .is_some_and(|key| REVEALED_SPOILERS.with(|set| set.borrow().contains(&key)))
    }

    fn reveal_spoilers(&self) {
        self.spoilers_shown.set(true);
        if let Some(key) = self.reveal_key() {
            REVEALED_SPOILERS.with(|set| set.borrow_mut().insert(key));
        }
    }

    fn spoiler_at(&self, rect: AbsoluteRect, x: f32, y: f32) -> bool {
        if !self.spoilers_hidden() || !self.text.contains("||") {
            return false;
        }
        let wrap_width = self.paint_width(rect.width, rect.height);
        self.with_buffer(wrap_width, |buffer, _| {
            buffer.layout_runs().any(|run| {
                y >= rect.y + run.line_top
                    && y < rect.y + run.line_top + run.line_height
                    && run.glyphs.iter().any(|g| {
                        g.metadata == META_SPOILER
                            && x >= rect.x + g.x
                            && x < rect.x + g.x + g.w
                    })
            })
        })
    }

    fn link_at(&self, rect: AbsoluteRect, x: f32, y: f32) -> Option<String> {
        if self.links.borrow().is_empty() {
            return None;
        }
        let wrap_width = self.paint_width(rect.width, rect.height);
        let start = self.with_buffer(wrap_width, |buffer, _| {
            buffer.layout_runs().find_map(|run| {
                let (top, bottom) = (
                    rect.y + run.line_top,
                    rect.y + run.line_top + run.line_height,
                );
                if y < top || y >= bottom {
                    return None;
                }
                run.glyphs
                    .iter()
                    .find(|g| {
                        (META_LINK_BASE..META_MENTION_BASE).contains(&g.metadata)
                            && x >= rect.x + g.x
                            && x < rect.x + g.x + g.w
                    })
                    .map(|g| g.metadata - META_LINK_BASE)
            })
        })?;
        let end = *self.links.borrow().get(&start)?;
        let url = self.text.get(start..end)?;
        Some(
            if url.starts_with("www.") || url.starts_with("discord.gg/") {
                format!("https://{url}")
            } else {
                url.to_string()
            },
        )
    }

    fn paint_width(&self, box_width: f32, box_height: f32) -> f32 {
        let measured = self
            .last_measured_width
            .get()
            .filter(|m| (m - box_width).abs() <= 2.0);
        let candidates = measured.into_iter().chain([
            box_width,
            box_width + 1.0,
            box_width - 1.0,
            box_width + 2.0,
            box_width - 2.0,
        ]);
        for width in candidates {
            if self.with_buffer(width, |buffer, _| {
                (buffer_extent(buffer).1 - box_height).abs() <= 1.5
            }) {
                return width;
            }
        }
        box_width
    }

    fn with_buffer<R>(&self, width: f32, f: impl FnOnce(&Buffer, &[EmojiTile]) -> R) -> R {
        let width = super::text::quantize_width(width);
        let theme = crate::theme_generation();
        let mut cache = self.cache.borrow_mut();
        if self.theme_seen.get() != theme {
            cache.clear();
            self.layer.borrow_mut().take();
            self.theme_seen.set(theme);
        }
        let index = match cache.iter().position(|(w, _, _)| *w == width.to_bits()) {
            Some(index) => index,
            None => {
                if cache.len() >= super::text::BUFFER_CACHE_SLOTS {
                    cache.remove(0);
                }
                let (buffer, emoji_images) =
                    self.shape_buffer(Wrap::WordOrGlyph, Some(width.max(0.0)));
                cache.push((width.to_bits(), buffer, emoji_images));
                cache.len() - 1
            }
        };
        let (_, buffer, emoji_images) = &cache[index];
        f(buffer, emoji_images)
    }
}

#[cfg(test)]
mod custom_emoji_token_tests {
    use super::*;

    #[test]
    fn finds_static_and_animated_tokens_only() {
        let text = "a <:happy:123> b <a:wig_gle:9>c <:x:1> <:ok:12x> <@5> <:no id:3> <:good:99999999999999999999999>";
        let found: Vec<(&str, u64, bool)> = custom_emoji_tokens(text)
            .into_iter()
            .map(|(range, id, animated)| (&text[range], id, animated))
            .collect();
        assert_eq!(
            found,
            [("<:happy:123>", 123, false), ("<a:wig_gle:9>", 9, true)]
        );
    }
}

fn to_cosmic_color(color: Color) -> CosmicColor {
    CosmicColor::rgba(color.r(), color.g(), color.b(), color.a())
}

fn background_for_metadata(meta: usize) -> Option<Color> {
    match meta {
        META_CODE => Some(colors::background_code()),
        META_SPOILER => Some(BACKGROUND_MOD_SUBTLE),
        META_MENTION => Some(MENTION_BACKGROUND),
        META_HIGHLIGHT => Some(HIGHLIGHT_BACKGROUND),
        META_EDITED => None,
        m if m >= META_TIMESTAMP_BASE => Some(BACKGROUND_MOD_NORMAL),
        _ => None,
    }
}

fn background_radius_for_metadata(meta: usize) -> f32 {
    match meta {
        META_SPOILER => 4.0,
        _ => 3.0,
    }
}

fn background_padding_for_metadata(_meta: usize) -> f32 {
    0.0
}

fn pill_pad_font_size(font_system: &mut FontSystem) -> f32 {
    const PROBE: f32 = 100.0;

    thread_local! {
        static SIZE: Cell<Option<f32>> = const { Cell::new(None) };
    }
    if let Some(size) = SIZE.with(Cell::get) {
        return size;
    }
    let mut buffer = Buffer::new(font_system, Metrics::new(PROBE, PROBE));
    buffer.set_size(None, None);
    let attrs = Attrs::new()
        .family(Family::Name(gg_sans_family(FdWeight::MEDIUM, false)))
        .weight(FdWeight::MEDIUM);
    buffer.set_rich_text(
        [("\u{00A0}", attrs.clone())],
        &attrs,
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(font_system, false);
    let advance = buffer
        .layout_runs()
        .next()
        .and_then(|run| run.glyphs.first().map(|g| g.w))
        .filter(|w| *w > 0.0);
    let size = advance.map_or(8.0, |w| 2.0 * PROBE / w);
    SIZE.with(|s| s.set(Some(size)));
    size
}

fn paint_metadata_backgrounds(
    canvas: &Canvas,
    run: &cosmic_text::LayoutRun,
    origin: (f32, f32),
    scale: f32,
) {
    let glyphs = run.glyphs;
    let mut i = 0;
    while i < glyphs.len() {
        let meta = glyphs[i].metadata;
        let is_mention = (META_MENTION_BASE..META_TIMESTAMP_BASE).contains(&meta);
        if meta == META_NONE
            || (meta >= META_EMOJI_BASE && !is_mention && meta < META_TIMESTAMP_BASE)
        {
            i += 1;
            continue;
        }
        let mut j = i;
        let mut x_min = f32::INFINITY;
        let mut x_max = f32::NEG_INFINITY;
        while j < glyphs.len() && glyphs[j].metadata == meta {
            x_min = x_min.min(glyphs[j].x);
            x_max = x_max.max(glyphs[j].x + glyphs[j].w);
            j += 1;
        }
        let tint = (is_mention && meta >= META_ROLE_BASE).then(|| {
            glyphs[i].color_opt.map_or(MENTION_BACKGROUND, |c| {
                Color::from_argb(26, c.r(), c.g(), c.b())
            })
        });
        let color = if is_mention {
            tint.or(Some(MENTION_BACKGROUND))
        } else {
            background_for_metadata(meta)
        };
        if let Some(color) = color {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.set_anti_alias(true);
            let pad = background_padding_for_metadata(meta) * scale;
            let bg_rect = Rect::from_xywh(
                origin.0 + (x_min - background_padding_for_metadata(meta)) * scale,
                origin.1 + run.line_top * scale,
                (x_max - x_min) * scale + pad * 2.0,
                run.line_height * scale,
            );
            let radius = background_radius_for_metadata(meta) * scale;
            canvas.draw_rrect(RRect::new_rect_xy(bg_rect, radius, radius), &paint);
        }
        i = j;
    }
}

thread_local! {
    /// Texts whose spoilers were clicked open, so a rebuild keeps them open.
    static REVEALED_SPOILERS: RefCell<HashSet<u64>> = RefCell::new(HashSet::new());
}

/// Hidden spoiler fill, measured on the web client.
const SPOILER_HIDDEN_BACKGROUND: Color = Color::from_argb(255, 102, 103, 112);

/// Paints an opaque fill over each spoiler's glyphs.
fn cover_spoilers(canvas: &Canvas, buffer: &Buffer, origin: (f32, f32), scale: f32) {
    let mut paint = Paint::default();
    paint.set_color(SPOILER_HIDDEN_BACKGROUND);
    paint.set_anti_alias(true);
    for run in buffer.layout_runs() {
        let glyphs = run.glyphs;
        let mut i = 0;
        while i < glyphs.len() {
            if glyphs[i].metadata != META_SPOILER {
                i += 1;
                continue;
            }
            let (mut x_min, mut x_max) = (f32::INFINITY, f32::NEG_INFINITY);
            while i < glyphs.len() && glyphs[i].metadata == META_SPOILER {
                x_min = x_min.min(glyphs[i].x);
                x_max = x_max.max(glyphs[i].x + glyphs[i].w);
                i += 1;
            }
            let bounds = Rect::from_xywh(
                origin.0 + x_min * scale,
                origin.1 + run.line_top * scale,
                (x_max - x_min) * scale,
                run.line_height * scale,
            );
            let radius = background_radius_for_metadata(META_SPOILER) * scale;
            canvas.draw_rrect(RRect::new_rect_xy(bounds, radius, radius), &paint);
        }
    }
}

const UNBOUNDED_WIDTH: f32 = 10_000.0;

fn buffer_extent(buffer: &Buffer) -> (f32, f32) {
    buffer.layout_runs().fold((0.0f32, 0.0f32), |(w, h), run| {
        (w.max(run.line_w), h.max(run.line_top + run.line_height))
    })
}

impl Widget for MessageTextWidget {
    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        let width = match known_dimensions.width {
            Some(w) => super::text::quantize_width(w),
            None => match available_space.width {
                AvailableSpace::Definite(w) => super::text::quantize_width(w),
                AvailableSpace::MinContent => {
                    let (buffer, _) = self.shape_buffer(Wrap::Word, Some(0.0));
                    (buffer
                        .layout_runs()
                        .fold(0.0f32, |max, run| max.max(run.line_w))
                        + 1.0)
                        .ceil()
                }
                AvailableSpace::MaxContent => UNBOUNDED_WIDTH,
            },
        };
        if known_dimensions.width.is_some()
            || matches!(available_space.width, AvailableSpace::Definite(_))
        {
            self.last_measured_width.set(Some(width));
        }

        let compute = || {
            self.with_buffer(width, |buffer, _emoji_images| {
                let (w, h) = buffer_extent(buffer);
                ((w + 1.0).min(width.max(0.0)), h)
            })
        };

        let (w, h) = match (&self.measure_cache, self.cache_key) {
            (Some(cache), Some(key)) => {
                cache.get_or_insert_with(key, self.font_size, Weight::NORMAL, Some(width), compute)
            }
            _ => compute(),
        };
        Size {
            width: w,
            height: h,
        }
    }

    fn hover_path_changed(&mut self, inside: bool) {
        if !inside && self.hovered_link.take().is_some() {
            crate_paint::request_repaint();
        }
        if !inside && self.hovered_timestamp.take().is_some() {
            hover_tooltip::hide();
        }
        if !inside && self.hovered_mention.take().is_some() {
            crate_paint::request_repaint();
        }
    }

    fn handle_event(&mut self, event: &UiEvent, rect: AbsoluteRect) -> EventResult {
        if self.inert {
            return EventResult::Ignored;
        }
        if let UiEvent::MouseMoved { x, y } = *event {
            let over = self.timestamp_at(rect, x, y);
            let start = over.as_ref().map(|(start, _)| *start);
            if start != self.hovered_timestamp.get() {
                self.hovered_timestamp.set(start);
                match over.and_then(|(start, at)| {
                    Some((self.timestamp_tooltips.borrow().get(&start)?.clone(), at))
                }) {
                    Some((label, at)) => hover_tooltip::show(label, at),
                    None => hover_tooltip::hide(),
                }
            }
            let mention = self.mention_meta_at(rect, x, y);
            if mention != self.hovered_mention.get() {
                self.hovered_mention.set(mention);
                crate_paint::request_repaint_rect(rect);
            }
            let hovered = self.link_start_at(rect, x, y);
            if hovered != self.hovered_link.get() {
                self.hovered_link.set(hovered);
                crate_paint::request_repaint_rect(rect);
            }
            return EventResult::Ignored;
        }
        let was_pressed = match event {
            UiEvent::MouseUp { .. } => self.pressed.replace(false),
            UiEvent::MouseDown { .. } => {
                self.pressed.set(false);
                false
            }
            _ => false,
        };
        match *event {
            UiEvent::MouseDown {
                x,
                y,
                button: MouseButton::Left,
                ..
            } if self.spoiler_at(rect, x, y) => {
                self.pressed.set(true);
                EventResult::Handled
            }
            UiEvent::MouseUp {
                x,
                y,
                button: MouseButton::Left,
            } if was_pressed && self.spoiler_at(rect, x, y) => {
                self.reveal_spoilers();
                crate_paint::request_repaint_rect(rect);
                EventResult::Handled
            }
            UiEvent::MouseDown {
                x,
                y,
                button: MouseButton::Left,
                ..
            } if self.emoji_at(rect, x, y).is_some() => {
                self.pressed.set(true);
                EventResult::Handled
            }
            UiEvent::MouseUp {
                x,
                y,
                button: MouseButton::Left,
            } if was_pressed && self.emoji_at(rect, x, y).is_some() => {
                if let (Some(click), Some(handler)) =
                    (self.emoji_at(rect, x, y), &self.on_emoji_click)
                {
                    handler(click);
                }
                EventResult::Handled
            }
            UiEvent::MouseDown {
                x,
                y,
                button: MouseButton::Left,
                ..
            } if self.mention_at(rect, x, y).is_some() => {
                self.pressed.set(true);
                EventResult::Handled
            }
            UiEvent::MouseUp {
                x,
                y,
                button: MouseButton::Left,
            } if was_pressed && self.mention_at(rect, x, y).is_some() => {
                if let (Some((target, at)), Some(handler)) =
                    (self.mention_at(rect, x, y), &self.on_mention)
                {
                    handler(target, at);
                }
                EventResult::Handled
            }
            UiEvent::MouseDown {
                x,
                y,
                button: MouseButton::Left,
                ..
            } if self.link_at(rect, x, y).is_some() => {
                self.pressed.set(true);
                EventResult::Handled
            }
            UiEvent::MouseUp {
                x,
                y,
                button: MouseButton::Left,
            } if was_pressed => match self.link_at(rect, x, y) {
                Some(url) => {
                    if let Some(handler) = &self.on_link {
                        handler(&url, self.link_rect_at(rect, x, y).unwrap_or(rect));
                    } else if let Some(open) = LINK_HANDLER.get() {
                        let masked = self
                            .link_start_at(rect, x, y)
                            .is_some_and(|start| self.masked_links.borrow().contains(&start));
                        open(&url, masked);
                    }
                    EventResult::Handled
                }
                None => EventResult::Ignored,
            },
            _ => EventResult::Ignored,
        }
    }

    fn relayout_after(&self, _event: &UiEvent) -> bool {
        false
    }

    fn cursor_icon_at(&self, rect: AbsoluteRect, x: f32, y: f32) -> Option<CursorIcon> {
        if self.inert {
            return None;
        }
        if self.spoiler_at(rect, x, y)
            || self.emoji_at(rect, x, y).is_some()
            || self.mention_at(rect, x, y).is_some()
        {
            return Some(CursorIcon::Pointer);
        }
        match self.link_at(rect, x, y) {
            Some(_) => Some(CursorIcon::Pointer),
            None => self.selection_key().map(|_| CursorIcon::Text),
        }
    }

    fn selection_key(&self) -> Option<u64> {
        match self.cache_key {
            Some(TextCacheKey::Stable(id)) if !self.text.is_empty() => Some(id),
            _ => None,
        }
    }

    fn selection_offset_at(&self, rect: AbsoluteRect, x: f32, y: f32) -> u64 {
        let wrap_width = self.paint_width(rect.width, rect.height);
        self.with_buffer(wrap_width, |buffer, _| {
            let (local_x, local_y) = (x - rect.x, y - rect.y);
            let runs: Vec<_> = buffer.layout_runs().collect();
            let Some(first) = runs.first() else { return 0 };

            let run = runs
                .iter()
                .find(|r| local_y >= r.line_top && local_y < r.line_top + r.line_height)
                .unwrap_or(if local_y < first.line_top {
                    first
                } else {
                    runs.last().unwrap_or(first)
                });
            let mut index = run.glyphs.first().map_or(0, |g| g.start);
            for glyph in run.glyphs {
                if local_x < glyph.x + glyph.w / 2.0 {
                    return encode_cursor(run.line_i, glyph.start);
                }
                index = glyph.end;
            }
            encode_cursor(run.line_i, index)
        })
    }

    fn selection_end(&self, rect: AbsoluteRect) -> u64 {
        let wrap_width = self.paint_width(rect.width, rect.height);
        self.with_buffer(wrap_width, |buffer, _| {
            buffer.layout_runs().last().map_or(0, |run| {
                encode_cursor(run.line_i, run.glyphs.last().map_or(0, |g| g.end))
            })
        })
    }

    fn selection_word(&self, rect: AbsoluteRect, offset: u64) -> (u64, u64) {
        let wrap_width = self.paint_width(rect.width, rect.height);
        self.with_buffer(wrap_width, |buffer, _| {
            let (line, index) = decode_cursor(offset);
            let Some(text) = buffer.lines.get(line).map(|l| l.text()) else {
                return (offset, offset);
            };
            for (start, word) in text.split_word_bound_indices() {
                if index >= start && index < start + word.len() {
                    return (
                        encode_cursor(line, start),
                        encode_cursor(line, start + word.len()),
                    );
                }
            }
            (offset, offset)
        })
    }

    fn selection_paragraph(&self, rect: AbsoluteRect, offset: u64) -> (u64, u64) {
        let wrap_width = self.paint_width(rect.width, rect.height);
        self.with_buffer(wrap_width, |buffer, _| {
            let (line, _) = decode_cursor(offset);
            match buffer.lines.get(line) {
                Some(l) => (encode_cursor(line, 0), encode_cursor(line, l.text().len())),
                None => (offset, offset),
            }
        })
    }

    fn selection_text(&self, rect: AbsoluteRect, range: (u64, u64)) -> String {
        let wrap_width = self.paint_width(rect.width, rect.height);
        self.with_buffer(wrap_width, |buffer, tiles| {
            let ((start_line, start_index), (end_line, end_index)) =
                (decode_cursor(range.0), decode_cursor(range.1));

            let mut placeholder = 0usize;
            let mut out = String::new();
            for (line, text) in buffer.lines.iter().map(|l| l.text()).enumerate() {
                let from = if line == start_line {
                    start_index.min(text.len())
                } else {
                    0
                };
                let to = if line == end_line {
                    end_index.min(text.len())
                } else {
                    text.len()
                };
                for (at, ch) in text.char_indices() {
                    let selected = line >= start_line && line <= end_line && at >= from && at < to;
                    if ch == '\u{FFFC}' {
                        if selected {
                            out.push_str(tiles.get(placeholder).map_or("", |t| t.3.as_str()));
                        }
                        placeholder += 1;
                    } else if selected {
                        out.push(ch);
                    }
                }
                if line >= start_line && line < end_line {
                    out.push('\n');
                }
            }
            out
        })
    }

    fn first_baseline(&self, content_width: f32) -> Option<f32> {
        self.with_buffer(content_width, |buffer, _emoji_images| {
            buffer.layout_runs().next().map(|run| run.line_y)
        })
    }

    fn apply_text_style(&mut self, text: &style::TextLayoutStyle) -> bool {
        let clamp = text.line_clamp.or(text.nowrap.then_some(1));
        if self.line_clamp == clamp {
            return false;
        }
        self.line_clamp = clamp;
        self.cache.borrow_mut().clear();
        self.layer.borrow_mut().take();
        true
    }

    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        let color = to_cosmic_color(colors::live(self.color));
        let scale_factor = app::current_scale_factor();
        let wrap_width = self.paint_width(rect.width, rect.height);
        self.with_buffer(wrap_width, |buffer, emoji_images| {
            let origin = (rect.x * scale_factor, rect.y * scale_factor);
            canvas.save();
            canvas.scale((1.0 / scale_factor, 1.0 / scale_factor));

            let selected = self.cache_key.and_then(|key| match key {
                TextCacheKey::Stable(id) => selection::range_for(id),
                TextCacheKey::Hash(_) => None,
            });
            for run in buffer.layout_runs() {
                if let Some(range) = selected {
                    if let Some((x, w)) = selection_span(&run, range) {
                        let mut paint = Paint::default();
                        paint.set_color(SELECTION_BACKGROUND);
                        canvas.draw_rect(
                            Rect::from_xywh(
                                origin.0 + x * scale_factor,
                                origin.1 + run.line_top * scale_factor,
                                w * scale_factor,
                                run.line_height * scale_factor,
                            ),
                            &paint,
                        );
                    }
                }
                paint_metadata_backgrounds(canvas, &run, origin, scale_factor);

                for glyph in run.glyphs {
                    if is_emoji_meta(glyph.metadata) {
                        if let Some((tile_image, tile_src, custom, _)) =
                            emoji_images.get(glyph.metadata - META_EMOJI_BASE)
                        {
                            let live = custom.and_then(|(id, animated)| {
                                self.custom_emoji.as_ref()?(id, animated)
                            });
                            let (image, src) = match &live {
                                Some(frame) => (
                                    Some(&frame.image),
                                    Rect::from_wh(
                                        frame.image.width() as f32,
                                        frame.image.height() as f32,
                                    ),
                                ),
                                None => (tile_image.as_ref(), *tile_src),
                            };
                            let (Some(image), src) = (image, &src) else {
                                continue;
                            };
                            let target = self.font_size * self.emoji_scale * scale_factor;
                            let (box_x, box_w) = if self.emoji_in_box {
                                (origin.0, rect.width * scale_factor)
                            } else {
                                (origin.0 + glyph.x * scale_factor, glyph.w * scale_factor)
                            };
                            let box_y = origin.1 + run.line_top * scale_factor;
                            let box_h = run.line_height * scale_factor;
                            let dst = Rect::from_xywh(
                                box_x + (box_w - target) / 2.0,
                                box_y + (box_h - target) / 2.0,
                                target,
                                target,
                            );
                            let mut paint = Paint::default();
                            paint.set_anti_alias(true);
                            super::draw_atlas_tile(canvas, image, *src, dst, &paint);
                            if let Some(next) = live.as_ref().and_then(|f| f.next_frame_ms) {
                                crate_paint::animate_region_at(
                                    canvas,
                                    dst,
                                    Instant::now()
                                        + Duration::from_millis(u64::from(next)),
                                );
                            }
                        }
                        continue;
                    }
                }

                if self.quoted_lines.get(run.line_i).copied().unwrap_or(false) {
                    let mut bar_paint = Paint::default();
                    bar_paint.set_color(QUOTE_BAR_COLOR);
                    bar_paint.set_anti_alias(true);
                    let bar_width = QUOTE_BAR_WIDTH * scale_factor;
                    let bar_rect = Rect::from_xywh(
                        origin.0 - bar_width - 4.0 * scale_factor,
                        origin.1 + run.line_top * scale_factor,
                        bar_width,
                        run.line_height * scale_factor,
                    );
                    canvas.draw_rrect(
                        RRect::new_rect_xy(bar_rect, bar_width, bar_width),
                        &bar_paint,
                    );
                }
            }

            let (px_w, px_h) = (
                (rect.width * scale_factor).ceil() as i32 + 2 * super::text::LAYER_PAD,
                (rect.height * scale_factor).ceil() as i32 + 2 * super::text::LAYER_PAD,
            );
            let key = super::text::LayerKey {
                wrap: wrap_width.to_bits(),
                underline: false,
                color: color.0,
                scale: scale_factor.to_bits(),
                w: px_w,
                h: px_h,
                gradient: 0,
            };
            let cached = self
                .layer
                .borrow()
                .as_ref()
                .filter(|(k, _)| *k == key)
                .map(|(_, image)| image.clone());
            let layer = match cached {
                Some(image) => Some(image),
                None if px_w <= super::text::MAX_LAYER_EDGE
                    && px_h <= super::text::MAX_LAYER_EDGE =>
                {
                    let pad = super::text::LAYER_PAD as f32;
                    let image =
                        surfaces::raster_n32_premul((px_w, px_h)).map(|mut surface| {
                            surface.canvas().clear(Color::TRANSPARENT);
                            let mut font_system = self.font_system.borrow_mut();
                            draw_glyphs(
                                surface.canvas(),
                                &self.glyph_cache,
                                &mut font_system,
                                buffer,
                                color,
                                scale_factor,
                                (pad, pad),
                            );
                            surface.image_snapshot()
                        });
                    if let Some(image) = &image {
                        *self.layer.borrow_mut() = Some((key, image.clone()));
                    }
                    image
                }
                None => None,
            };
            match layer {
                Some(image) => {
                    let pad = super::text::LAYER_PAD as f32;
                    canvas.draw_image(
                        &image,
                        (origin.0.round() - pad, origin.1.round() - pad),
                        None,
                    );
                }
                None => {
                    let mut font_system = self.font_system.borrow_mut();
                    draw_glyphs(
                        canvas,
                        &self.glyph_cache,
                        &mut font_system,
                        buffer,
                        color,
                        scale_factor,
                        origin,
                    );
                }
            }

            if self.spoilers_hidden() {
                cover_spoilers(canvas, buffer, origin, scale_factor);
            }

            if let Some(meta) = self.hovered_mention.get() {
                let mut wash = Paint::default();
                wash.set_color(BRAND);
                wash.set_anti_alias(true);
                let mut font_system = self.font_system.borrow_mut();
                for run in buffer.layout_runs() {
                    let glyphs: Vec<_> = run.glyphs.iter().filter(|g| g.metadata == meta).collect();
                    let (left, right) = glyphs.iter().fold((f32::MAX, f32::MIN), |(l, r), g| {
                        (l.min(g.x), r.max(g.x + g.w))
                    });
                    if left >= right {
                        continue;
                    }
                    let bounds = Rect::from_xywh(
                        origin.0 + left * scale_factor,
                        origin.1 + run.line_top * scale_factor,
                        (right - left) * scale_factor,
                        run.line_height * scale_factor,
                    );
                    canvas.draw_rrect(
                        RRect::new_rect_xy(bounds, 3.0 * scale_factor, 3.0 * scale_factor),
                        &wash,
                    );
                    for glyph in glyphs {
                        let physical_glyph =
                            glyph.physical((0., run.line_y * scale_factor), scale_factor);
                        paint_glyph(
                            canvas,
                            &self.glyph_cache,
                            &mut font_system,
                            physical_glyph,
                            CosmicColor::rgb(0xff, 0xff, 0xff),
                            origin,
                        );
                    }
                }
            }

            if let Some(start) = self.hovered_link.get() {
                let url = self
                    .links
                    .borrow()
                    .get(&start)
                    .and_then(|end| self.text.get(start..*end))
                    .unwrap_or_default()
                    .to_string();
                let mut paint = Paint::default();
                paint.set_color(self.link_color(&url));
                paint.set_anti_alias(true);
                for run in buffer.layout_runs() {
                    let glyphs = run
                        .glyphs
                        .iter()
                        .filter(|g| g.metadata == META_LINK_BASE + start);
                    let (left, right) = glyphs.fold((f32::MAX, f32::MIN), |(l, r), g| {
                        (l.min(g.x), r.max(g.x + g.w))
                    });
                    if left < right {
                        let y = origin.1 + (run.line_y + 2.0) * scale_factor;
                        canvas.draw_rect(
                            Rect::from_xywh(
                                origin.0 + left * scale_factor,
                                y,
                                (right - left) * scale_factor,
                                scale_factor.max(1.0),
                            ),
                            &paint,
                        );
                    }
                }
            }
            canvas.restore();
        });
    }
}

fn draw_glyphs(
    canvas: &Canvas,
    glyph_cache: &GlyphCache,
    font_system: &mut FontSystem,
    buffer: &Buffer,
    color: CosmicColor,
    scale_factor: f32,
    origin: (f32, f32),
) {
    for run in buffer.layout_runs() {
        for glyph in run.glyphs {
            if is_emoji_meta(glyph.metadata) {
                continue;
            }
            let physical_glyph = glyph.physical((0., run.line_y * scale_factor), scale_factor);
            let glyph_color = glyph.color_opt.map_or(color, |c| c);
            paint_glyph(
                canvas,
                glyph_cache,
                font_system,
                physical_glyph,
                glyph_color,
                origin,
            );
        }
        let mut decoration_renderer = SkiaDecorationRenderer {
            canvas,
            origin,
            scale: scale_factor,
        };
        cosmic_text::render_decoration(&mut decoration_renderer, &run, color);
    }
}
