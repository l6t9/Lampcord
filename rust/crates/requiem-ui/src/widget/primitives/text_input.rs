use crate::app;
use crate::colors::{
    BACKGROUND_MOD_SUBTLE, BRAND, SURFACE_HIGH, TEXT_ANCHOR, TEXT_MUTED, TEXT_SECONDARY,
    TEXT_STRONG,
};
use crate::colors;
use crate::icon_handle::IconPath;
use crate::icons;
use crate::paint as crate_paint;
use crate::signal;
use crate::timestamp;
use skia_safe::surfaces;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::mem;
use std::ops::Range;
use std::rc::Rc;
use std::time::{Duration, Instant};

use cosmic_text::fontdb::Weight as FdWeight;
use cosmic_text::{
    Attrs, Buffer, Color as CosmicColor, Cursor, Family, FontSystem, Metrics, Shaping,
    UnderlineStyle, Wrap,
};
use skia_safe::{Canvas, Color, Image, Paint, PaintStyle, RRect, Rect};
use taffy::{AvailableSpace, Size};
use unicode_segmentation::UnicodeSegmentation;

use super::markdown::{self, MarkdownFeatures, SpanAttrs};
use super::message_text::{
    CustomEmojiResolver, EMOJI_PLACEHOLDER, EMOJI_SCALE, EmojiProvider, custom_emoji_tokens,
};
use crate::clipboard;
use crate::event::{CursorIcon, EventResult, MouseButton, NamedKey, UiEvent};
use crate::font_system::{GG_SANS_MONO_FAMILY, gg_sans_family};
use crate::glyph_cache::{GlyphCache, SkiaDecorationRenderer, paint_glyph};
use crate::style::ComputedStyle;
use crate::widget::{AbsoluteRect, Widget};

const MARKDOWN_CODE_FONT_SCALE: f32 = 0.85;

const SUBTEXT_METRICS: Metrics = Metrics::new(13.0, 17.875);

const QUOTE_BAR_COLOR: Color = Color::from_argb(255, 0x47, 0x48, 0x51);
const QUOTE_BAR_WIDTH: f32 = 4.0;

const META_NONE: usize = 0;
const META_CODE: usize = 1;
const META_SPOILER: usize = 2;
const META_CHIP_BASE: usize = 3;

const META_EMOJI_BASE: usize = 1_000_000;

pub struct AutocompleteTrigger {
    pub trigger_char: char,
    pub candidates: Vec<AutocompleteCandidate>,

    /// The trigger only opens a popup at the start of the text.
    pub only_at_message_start: bool,

    pub chip_background: Option<Color>,
    pub chip_text_color: Color,

    pub popup_label: String,

    /// The query may contain spaces. A trigger that is not multi-word stops at the first space.
    pub multi_word: bool,

    /// The query is the whole field, not the token after the trigger.
    pub whole_text: bool,
}

#[derive(Default)]
pub struct AutocompleteCandidate {
    pub display: String,

    pub id: Option<u64>,

    pub subtitle: Option<String>,

    pub avatar_color: Option<Color>,

    pub avatar: Option<Rc<dyn Fn() -> Option<Image>>>,

    pub status: Option<super::PresenceStatus>,

    pub fallback_icon: Option<super::FallbackIcon>,

    pub section: u8,

    pub name_color: Option<Color>,

    /// Text inserted on select. `None` inserts `display`.
    pub token: Option<String>,

    /// The inserted token keeps the trigger character.
    pub show_trigger: bool,

    pub section_label: Option<String>,

    /// Runs instead of inserting `token` when set.
    pub on_select: Option<Rc<dyn Fn()>>,
}

#[derive(Clone)]
pub struct DynamicAutocomplete {
    pub trigger_char: char,

    pub popup_label: String,

    /// Search waits until the query is at least this long.
    pub min_query_len: usize,

    pub search: Rc<dyn Fn(&str) -> Vec<DynamicMatch>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicMatch {
    pub label: String,

    pub glyph: String,
}

struct PopupState {
    dynamic_matches: Option<Vec<DynamicMatch>>,
    trigger_index: usize,

    query_start: usize,

    filtered: Vec<usize>,

    selected: usize,
}

#[derive(Clone, Debug)]
pub struct DecorationStyle {
    pub weight: u16,
    pub color: Color,
    pub background: Option<Color>,

    pub radii: [f32; 4],

    pub pad_left: f32,
    pub pad_right: f32,
}

#[derive(Clone, Debug)]
pub struct TextDecoration {
    pub range: Range<usize>,
    pub style: usize,
}

pub type Decorator = Rc<dyn Fn(&str) -> Vec<TextDecoration>>;

pub type CodeHighlighter = Rc<dyn Fn(&str, &str) -> Vec<(Range<usize>, Color)>>;

const META_DECORATION_BASE: usize = 500_000;

enum RunKind {
    Markdown(SpanAttrs),

    Chip {
        trigger_index: usize,
        display: String,
    },

    Emoji,

    Plain,

    Decorated(usize),

    Fence,

    Code(Option<Color>),
}

struct Run {
    range: Range<usize>,
    kind: RunKind,
}

struct ChipMatch {
    range: Range<usize>,
    trigger_index: usize,
    display: String,

    is_emoji: bool,
}

const PASSWORD_MASK: &str = "•";

const PASSWORD_MASK_TRIGGER: usize = usize::MAX;

pub const TIMESTAMP_TRIGGER_CHAR: char = '\u{2}';

const BLINK_INTERVAL: Duration = Duration::from_millis(500);

const DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(400);

const INSET_X: f32 = 12.0;

const CLEAR_ZONE: f32 = 36.0;

const INSET_Y_MULTILINE: f32 = 8.0;

const WRAP_PAD_Y: f32 = 8.0;

const SELECTION_COLOR: Color = Color::from_argb(90, 0x58, 0x65, 0xf2);

pub struct TextInputWidget {
    text: String,
    cursor: usize,
    placeholder: String,
    font_size: f32,

    placeholder_font_size: Option<f32>,

    placeholder_live: Option<signal::Signal<String>>,

    live_placeholder_seen: RefCell<Option<String>>,
    color: Color,
    placeholder_color: Color,
    font_system: Rc<RefCell<FontSystem>>,
    glyph_cache: Rc<GlyphCache>,
    focused: bool,
    focused_since: Instant,
    on_change: Option<Box<dyn FnMut(&str)>>,
    on_submit: Option<Box<dyn FnMut(String)>>,

    code_highlighter: Option<CodeHighlighter>,

    size_changed: Cell<bool>,

    highlight_cache: RefCell<Vec<(String, String, Rc<Vec<(Range<usize>, Color)>>)>>,

    submit_when_empty: Option<Rc<dyn Fn() -> bool>>,

    paste_override: Option<Rc<dyn Fn() -> bool>>,
    on_focus_change: Option<Box<dyn FnMut(bool)>>,

    key_hook: Option<Box<dyn FnMut(NamedKey, bool, &str) -> bool>>,

    edge_hook: Option<Box<dyn FnMut(NamedKey, bool, &str) -> bool>>,

    auto_width: bool,

    popup_frame: Option<signal::Signal<AbsoluteRect>>,

    decorator: Option<Decorator>,
    decoration_styles: Vec<DecorationStyle>,
    multiline: bool,

    wrap: bool,

    wrap_width: Cell<Option<f32>>,

    scroll_y: Cell<f32>,
    max_scroll: Cell<f32>,

    scroll_x: Cell<f32>,

    scroll_follow: Cell<Option<(usize, usize)>>,

    focus_border_color: Color,

    cache: RefCell<Option<BufferCache>>,
    triggers: Vec<AutocompleteTrigger>,

    markdown: MarkdownFeatures,
    popup: Option<PopupState>,

    dynamic_sources: HashMap<usize, DynamicAutocomplete>,

    popup_outset: (f32, f32),

    selection_anchor: Option<usize>,

    ime_preedit: String,

    dragging: bool,

    last_click: Option<(Instant, usize, u8)>,

    goal_x: Option<f32>,

    history: History,

    skip_history: bool,

    line_height: Option<f32>,

    pad_y_override: Option<f32>,

    emoji_provider: Option<EmojiProvider>,

    custom_emoji: Option<CustomEmojiResolver>,

    pending_custom_emoji: RefCell<Vec<(u64, bool)>>,

    password: bool,

    read_only: bool,

    disabled: bool,

    max_length: Option<usize>,

    clearable: bool,

    inset_left: f32,
    inset_right: f32,
}

const POPUP_ROW_HEIGHT: f32 = 40.0;
const POPUP_SMALL_ROW_HEIGHT: f32 = 28.0;
const POPUP_HEADER_HEIGHT: f32 = 40.0;
const POPUP_GAP: f32 = 8.0;

const POPUP_BOTTOM_PADDING: f32 = 8.0;

struct PopupGeometry {
    panel: Rect,
    rows_top: f32,
    #[cfg_attr(not(test), allow(dead_code))]
    row_height: f32,
}

struct RowSlot {
    top: f32,
    height: f32,

    divider: bool,

    heading: Option<String>,
}

const POPUP_SECTION_ROW_HEIGHT: f32 = 36.0;
const POPUP_DIVIDER_HEIGHT: f32 = 17.0;

enum PopupHit {
    Row(usize),

    Chrome,
}

#[derive(Clone, Debug, PartialEq)]
struct HistorySnapshot {
    text: String,
    cursor: usize,
    anchor: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EditKind {
    Insert,
    DeleteBack,
    DeleteForward,
    Other,
}

const HISTORY_LIMIT: usize = 200;

#[derive(Default)]
struct History {
    undo: Vec<HistorySnapshot>,
    redo: Vec<HistorySnapshot>,

    last: Option<(EditKind, usize)>,

    last_ended_in_space: bool,
}

type BufferCache = (String, Option<usize>, String, Option<u32>, Layout);

struct Layout {
    full: Buffer,

    map: Option<LayoutMap>,

    preedit_caret: Option<Buffer>,

    emoji_images: Vec<(Image, Rect)>,
}

#[derive(Default)]
struct LayoutMap {
    text: String,
    segs: Vec<LayoutSeg>,
    source_len: usize,
}

struct LayoutSeg {
    src: Range<usize>,
    lay: Range<usize>,

    atomic: bool,
}

impl LayoutMap {
    fn push(&mut self, src: Range<usize>, shown: &str, atomic: bool) {
        self.hide_up_to(src.start);
        let start = self.text.len();
        self.text.push_str(shown);
        self.segs.push(LayoutSeg {
            src,
            lay: start..self.text.len(),
            atomic,
        });
    }

    fn hide_up_to(&mut self, src_end: usize) {
        let covered = self.segs.last().map_or(0, |s| s.src.end);
        if covered < src_end {
            let at = self.text.len();
            self.segs.push(LayoutSeg {
                src: covered..src_end,
                lay: at..at,
                atomic: false,
            });
        }
    }

    fn finish(&mut self, source_len: usize) {
        self.hide_up_to(source_len);
        self.source_len = source_len;
    }

    fn to_layout(&self, byte: usize, snap_end: bool) -> usize {
        for seg in &self.segs {
            if seg.src.start <= byte && byte < seg.src.end {
                return if seg.lay.is_empty() || byte == seg.src.start {
                    seg.lay.start
                } else if seg.atomic {
                    if snap_end { seg.lay.end } else { seg.lay.start }
                } else {
                    seg.lay.start + (byte - seg.src.start)
                };
            }
        }
        self.text.len()
    }

    fn to_source(&self, lay: usize) -> usize {
        for seg in self.segs.iter().filter(|s| !s.lay.is_empty()) {
            if lay < seg.lay.end {
                if lay <= seg.lay.start {
                    return seg.src.start;
                }
                let into = lay - seg.lay.start;
                return if !seg.atomic {
                    seg.src.start + into
                } else if into * 2 < seg.lay.len() {
                    seg.src.start
                } else {
                    seg.src.end
                };
            }
        }
        self.source_len
    }

    fn cursor(&self, byte: usize, snap_end: bool) -> Cursor {
        byte_to_cursor(&self.text, self.to_layout(byte, snap_end))
    }
}

impl TextInputWidget {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        initial_text: impl Into<String>,
        placeholder: impl Into<String>,
        font_size: f32,
        color: Color,
        placeholder_color: Color,
        font_system: Rc<RefCell<FontSystem>>,
        glyph_cache: Rc<GlyphCache>,
        initial_focused: bool,
    ) -> Self {
        let text = normalize_line_breaks(&initial_text.into());
        let cursor = text.len();
        Self {
            text,
            cursor,
            placeholder: placeholder.into(),
            placeholder_font_size: None,
            placeholder_live: None,
            live_placeholder_seen: RefCell::new(None),
            font_size,
            color,
            placeholder_color,
            font_system,
            glyph_cache,
            focused: initial_focused,
            focused_since: Instant::now(),
            on_change: None,
            on_submit: None,
            code_highlighter: None,
            size_changed: Cell::new(false),
            highlight_cache: RefCell::new(Vec::new()),
            submit_when_empty: None,
            paste_override: None,
            on_focus_change: None,
            key_hook: None,
            edge_hook: None,
            auto_width: false,
            popup_frame: None,
            decorator: None,
            decoration_styles: Vec::new(),
            multiline: false,
            wrap: false,
            wrap_width: Cell::new(None),
            scroll_y: Cell::new(0.0),
            max_scroll: Cell::new(0.0),
            scroll_x: Cell::new(0.0),
            scroll_follow: Cell::new(None),
            focus_border_color: BRAND,
            cache: RefCell::new(None),
            triggers: Vec::new(),
            markdown: MarkdownFeatures::NONE,
            popup: None,
            dynamic_sources: HashMap::new(),
            popup_outset: (0.0, 0.0),
            selection_anchor: None,
            ime_preedit: String::new(),
            dragging: false,
            last_click: None,
            goal_x: None,
            history: History::default(),
            skip_history: false,
            line_height: None,
            pad_y_override: None,
            emoji_provider: None,
            custom_emoji: None,
            pending_custom_emoji: RefCell::new(Vec::new()),
            password: false,
            read_only: false,
            disabled: false,
            max_length: None,
            clearable: false,
            inset_left: INSET_X,
            inset_right: INSET_X,
        }
    }

    pub fn with_custom_emoji(mut self, resolver: CustomEmojiResolver) -> Self {
        self.custom_emoji = Some(resolver);
        self
    }

    pub fn refresh_custom_emoji(&mut self) -> bool {
        let Some(resolver) = self.custom_emoji.clone() else {
            return false;
        };
        if self
            .pending_custom_emoji
            .borrow()
            .iter()
            .any(|&(id, animated)| resolver(id, animated).is_some())
        {
            *self.cache.borrow_mut() = None;
            return true;
        }
        false
    }

    fn emoji_picture(&self, source: &str) -> Option<(Image, (f32, f32, f32, f32))> {
        if source.starts_with('<') {
            let &(_, id, animated) = custom_emoji_tokens(source).first()?;
            let image = self.custom_emoji.as_ref()?(id, animated)?.image;
            let size = (image.width() as f32, image.height() as f32);
            return Some((image, (0.0, 0.0, size.0, size.1)));
        }
        self.emoji_provider.and_then(|provider| provider(source))
    }

    pub fn with_emoji_provider(mut self, provider: EmojiProvider) -> Self {
        self.emoji_provider = Some(provider);
        self
    }

    pub fn with_autocomplete(mut self, triggers: Vec<AutocompleteTrigger>) -> Self {
        self.triggers = triggers;
        self.dynamic_sources.clear();
        self.refresh_popup();
        self
    }

    pub fn with_dynamic_autocomplete(mut self, source: DynamicAutocomplete) -> Self {
        let index = self.triggers.len();
        self.triggers.push(AutocompleteTrigger {
            trigger_char: source.trigger_char,
            candidates: Vec::new(),
            only_at_message_start: false,
            chip_background: None,
            chip_text_color: Color::WHITE,
            popup_label: source.popup_label.clone(),
            multi_word: false,
            whole_text: false,
        });
        self.dynamic_sources.insert(index, source);
        self.refresh_popup();
        self
    }

    pub fn with_markdown(mut self, markdown: MarkdownFeatures) -> Self {
        self.markdown = markdown;
        self
    }

    pub fn with_on_change(mut self, on_change: impl FnMut(&str) + 'static) -> Self {
        self.on_change = Some(Box::new(on_change));
        self
    }

    pub fn with_submit_when_empty(mut self, allowed: Rc<dyn Fn() -> bool>) -> Self {
        self.submit_when_empty = Some(allowed);
        self
    }

    pub fn with_paste_override(mut self, hook: Rc<dyn Fn() -> bool>) -> Self {
        self.paste_override = Some(hook);
        self
    }

    pub fn with_on_submit(mut self, on_submit: impl FnMut(String) + 'static) -> Self {
        self.on_submit = Some(Box::new(on_submit));
        self
    }

    pub fn with_code_highlighter(mut self, highlighter: CodeHighlighter) -> Self {
        self.code_highlighter = Some(highlighter);
        self
    }

    pub fn with_edge_hook(
        mut self,
        hook: impl FnMut(NamedKey, bool, &str) -> bool + 'static,
    ) -> Self {
        self.edge_hook = Some(Box::new(hook));
        self
    }

    pub fn with_auto_width(mut self, on: bool) -> Self {
        self.auto_width = on;
        self
    }

    pub fn with_popup_frame(mut self, frame: signal::Signal<AbsoluteRect>) -> Self {
        self.popup_frame = Some(frame);
        self
    }

    pub fn with_key_hook(
        mut self,
        hook: impl FnMut(NamedKey, bool, &str) -> bool + 'static,
    ) -> Self {
        self.key_hook = Some(Box::new(hook));
        self
    }

    pub fn with_decorator(mut self, styles: Vec<DecorationStyle>, decorator: Decorator) -> Self {
        self.decoration_styles = styles;
        self.decorator = Some(decorator);
        self
    }

    pub fn with_on_focus_change(mut self, on_focus_change: impl FnMut(bool) + 'static) -> Self {
        self.on_focus_change = Some(Box::new(on_focus_change));
        self
    }

    pub fn with_password(mut self) -> Self {
        self.password = true;
        self
    }

    pub fn with_read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    pub fn with_disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    pub fn with_clear_button(mut self) -> Self {
        self.clearable = true;
        self.inset_right = CLEAR_ZONE;
        self
    }

    pub fn with_live_placeholder(mut self, live: signal::Signal<String>) -> Self {
        self.placeholder_live = Some(live);
        self
    }

    pub fn with_placeholder_font_size(mut self, size: f32) -> Self {
        self.placeholder_font_size = Some(size);
        self
    }

    pub fn with_insets(mut self, left: f32, right: f32) -> Self {
        self.inset_left = left;
        self.inset_right = if self.clearable {
            right.max(CLEAR_ZONE)
        } else {
            right
        };
        self
    }

    pub fn with_wrap(mut self) -> Self {
        self.wrap = true;
        self
    }

    fn wraps(&self) -> bool {
        self.wrap || self.multiline
    }

    fn text_wrap_width(&self) -> Option<f32> {
        if self.wraps() {
            self.wrap_width.get()
        } else {
            None
        }
    }

    fn set_field_width(&self, width: f32) {
        if self.wraps() {
            self.wrap_width.set(Some(
                (width.round() - self.inset_left - self.inset_right).max(1.0),
            ));
        }
    }

    fn text_left(&self, rect: AbsoluteRect) -> f32 {
        rect.x + self.inset_left - self.scroll_x.get()
    }

    fn pad_y(&self) -> f32 {
        if self.multiline {
            INSET_Y_MULTILINE
        } else {
            self.pad_y_override.unwrap_or(WRAP_PAD_Y)
        }
    }

    pub fn with_line_height(mut self, px: f32) -> Self {
        self.line_height = Some(px);
        self
    }

    pub fn with_vertical_padding(mut self, px: f32) -> Self {
        self.pad_y_override = Some(px);
        self
    }

    fn line_height(&self) -> f32 {
        self.line_height.unwrap_or(self.font_size * 1.2)
    }

    fn emoji_line_height(&self, emoji_font_size: f32) -> f32 {
        self.line_height.unwrap_or(emoji_font_size * 1.2)
    }

    fn text_top(&self, rect: AbsoluteRect, content_h: f32) -> f32 {
        if !self.wraps() {
            return rect.y + (rect.height - content_h) / 2.0;
        }
        let pad = if self.multiline {
            INSET_Y_MULTILINE
        } else {
            ((rect.height - content_h) / 2.0).max(WRAP_PAD_Y)
        };
        rect.y + pad - self.scroll_y.get()
    }

    pub fn with_multiline(mut self) -> Self {
        self.multiline = true;
        self
    }

    pub fn with_focus_border_color(mut self, color: Color) -> Self {
        self.focus_border_color = color;
        self
    }

    pub fn with_popup_outset(mut self, left: f32, right: f32) -> Self {
        self.popup_outset = (left, right);
        self
    }

    fn content_key(&self) -> (u64, usize) {
        let mut hasher = DefaultHasher::new();
        self.text.hash(&mut hasher);
        self.ime_preedit.hash(&mut hasher);
        (hasher.finish(), self.text.len())
    }

    fn notify_change(&mut self) {
        if let Some(on_change) = &mut self.on_change {
            on_change(&self.text);
        }
    }

    fn record_history(&mut self, event: &UiEvent, before: HistorySnapshot) {
        if mem::take(&mut self.skip_history) || self.text == before.text {
            return;
        }
        let kind = match event {
            UiEvent::TextInput { .. }
            | UiEvent::KeyDown {
                key: NamedKey::Space,
                ..
            } => EditKind::Insert,
            UiEvent::KeyDown {
                key: NamedKey::Backspace,
                ..
            } => EditKind::DeleteBack,
            UiEvent::KeyDown {
                key: NamedKey::Delete,
                ..
            } => EditKind::DeleteForward,
            _ => EditKind::Other,
        };
        let ends_in_space = self.text[..self.cursor.min(self.text.len())]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace);
        let continues_run = matches!(
            kind,
            EditKind::Insert | EditKind::DeleteBack | EditKind::DeleteForward
        ) && before.anchor.is_none_or(|a| a == before.cursor)
            && self.history.last == Some((kind, before.cursor))
            && !(kind == EditKind::Insert && self.history.last_ended_in_space && !ends_in_space);
        if !continues_run {
            self.history.undo.push(before);
            if self.history.undo.len() > HISTORY_LIMIT {
                self.history.undo.remove(0);
            }
        }
        self.history.redo.clear();
        self.history.last = Some((kind, self.cursor));
        self.history.last_ended_in_space = ends_in_space;
    }

    pub fn caret_state(&self) -> (usize, Option<usize>) {
        (self.cursor, self.selection_anchor)
    }

    pub fn set_caret_state(&mut self, (cursor, anchor): (usize, Option<usize>)) {
        let ok = |i: usize| i <= self.text.len() && self.text.is_char_boundary(i);
        if ok(cursor) {
            self.cursor = cursor;
            self.selection_anchor = anchor.filter(|a| ok(*a));
        }
    }

    fn restore_snapshot(&mut self, snapshot: HistorySnapshot) {
        self.text = snapshot.text;
        let mut cursor = snapshot.cursor.min(self.text.len());
        while !self.text.is_char_boundary(cursor) {
            cursor -= 1;
        }
        self.cursor = cursor;
        self.selection_anchor = snapshot
            .anchor
            .filter(|a| *a <= self.text.len() && self.text.is_char_boundary(*a));
        self.ime_preedit.clear();
        self.history.last = None;
        self.skip_history = true;
        self.notify_change();
        self.refresh_popup();
    }

    fn current_snapshot(&self) -> HistorySnapshot {
        HistorySnapshot {
            text: self.text.clone(),
            cursor: self.cursor,
            anchor: self.selection_anchor,
        }
    }

    fn undo(&mut self) {
        if self.read_only || self.disabled {
            return;
        }
        if let Some(previous) = self.history.undo.pop() {
            let current = self.current_snapshot();
            self.history.redo.push(current);
            self.restore_snapshot(previous);
        }
    }

    fn redo(&mut self) {
        if self.read_only || self.disabled {
            return;
        }
        if let Some(next) = self.history.redo.pop() {
            let current = self.current_snapshot();
            self.history.undo.push(current);
            self.restore_snapshot(next);
        }
    }

    fn selection_range(&self) -> Option<Range<usize>> {
        let mut anchor = self.selection_anchor?.min(self.text.len());
        while !self.text.is_char_boundary(anchor) {
            anchor -= 1;
        }
        if anchor == self.cursor {
            return None;
        }
        Some(anchor.min(self.cursor)..anchor.max(self.cursor))
    }

    fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    fn extend_selection(&mut self) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }
    }

    fn select_all(&mut self) {
        self.selection_anchor = Some(0);
        self.cursor = self.text.len();
        self.popup = None;
    }

    fn begin_click(&mut self, pos: usize, shift: bool) {
        if shift {
            self.extend_selection();
        } else {
            self.selection_anchor = Some(pos);
        }
        self.cursor = pos;
        self.dragging = true;
        self.refresh_popup();
    }

    fn select_word_at(&mut self, pos: usize) {
        let range = match self
            .atom_ranges()
            .into_iter()
            .find(|r| r.start <= pos && pos < r.end)
        {
            Some(atom) => atom,
            None => word_range_at(&self.text, pos),
        };
        if range.is_empty() {
            self.selection_anchor = None;
            self.cursor = pos;
        } else {
            self.selection_anchor = Some(range.start);
            self.cursor = range.end;
        }
        self.refresh_popup();
    }

    fn select_line_at(&mut self, pos: usize) {
        let range = line_range_at(&self.text, pos);
        if range.is_empty() {
            self.selection_anchor = None;
            self.cursor = pos;
        } else {
            self.selection_anchor = Some(range.start);
            self.cursor = range.end;
        }
        self.refresh_popup();
    }

    fn delete_selection(&mut self) -> bool {
        let Some(range) = self.selection_range() else {
            return false;
        };
        self.text.replace_range(range.clone(), "");
        self.cursor = range.start;
        self.selection_anchor = None;
        true
    }

    fn copy(&self) {
        if self.password {
            return;
        }
        if let Some(range) = self.selection_range() {
            clipboard::set_text(self.text[range].to_string());
        }
    }

    fn cut(&mut self) {
        if self.read_only || self.disabled {
            return;
        }
        if self.password || self.selection_range().is_none() {
            return;
        }
        self.copy();
        self.delete_selection();
        self.notify_change();
        self.refresh_popup();
    }

    fn paste(&mut self) {
        if self.read_only || self.disabled {
            return;
        }
        if self.paste_override.as_ref().is_some_and(|hook| hook()) {
            return;
        }
        let Some(pasted) = clipboard::get_text() else {
            return;
        };
        let pasted = if self.wraps() {
            pasted
        } else {
            pasted.replace(['\n', '\r'], " ")
        };
        self.insert(&pasted);
    }

    fn insert(&mut self, s: &str) {
        if self.read_only || self.disabled {
            return;
        }
        self.ime_preedit.clear();
        self.delete_selection();
        let normalized = normalize_line_breaks(s);
        let s = normalized.as_str();
        let s = match self.max_length {
            Some(max) => {
                let room = max.saturating_sub(self.text.chars().count());
                &s[..s.char_indices().nth(room).map_or(s.len(), |(i, _)| i)]
            }
            None => s,
        };
        self.text.insert_str(self.cursor, s);
        self.cursor += s.len();
        if s == ":" {
            self.complete_typed_shortcode();
        }
        self.notify_change();
        self.refresh_popup();
    }

    fn complete_typed_shortcode(&mut self) {
        if self.caret_in_code_block() {
            return;
        }
        let before = &self.text[..self.cursor];
        let Some(open) = before[..before.len() - 1].rfind(':') else {
            return;
        };
        if !before[..open]
            .chars()
            .next_back()
            .is_none_or(char::is_whitespace)
        {
            return;
        }
        let name = &before[open + 1..before.len() - 1];
        if name.is_empty() || name.contains(char::is_whitespace) {
            return;
        }
        let name = name.to_lowercase();
        let glyph = self
            .dynamic_sources
            .values()
            .filter(|source| {
                source.trigger_char == ':' && name.chars().count() >= source.min_query_len
            })
            .find_map(|source| {
                (source.search)(&name)
                    .into_iter()
                    .find(|m| m.label == name)
                    .map(|m| m.glyph)
            });
        if let Some(glyph) = glyph {
            self.text.replace_range(open..self.cursor, &glyph);
            self.cursor = open + glyph.len();
        }
    }

    fn backspace(&mut self) {
        if self.read_only || self.disabled {
            return;
        }
        if self.delete_selection() {
            self.notify_change();
            self.refresh_popup();
            return;
        }
        let prev = self.prev_cursor_position(self.cursor);
        if prev == self.cursor {
            return;
        }
        self.text.replace_range(prev..self.cursor, "");
        self.cursor = prev;
        self.notify_change();
        self.refresh_popup();
    }

    fn delete_forward(&mut self) {
        if self.read_only || self.disabled {
            return;
        }
        if self.delete_selection() {
            self.notify_change();
            self.refresh_popup();
            return;
        }
        let next = self.next_cursor_position(self.cursor);
        if next == self.cursor {
            return;
        }
        self.text.replace_range(self.cursor..next, "");
        self.notify_change();
        self.refresh_popup();
    }

    fn refresh_popup(&mut self) {
        self.popup = self.detect_trigger();
    }

    fn detect_trigger(&self) -> Option<PopupState> {
        if self.caret_in_code_block() {
            return None;
        }
        if let Some((trigger_index, trigger)) =
            self.triggers.iter().enumerate().find(|(_, t)| t.whole_text)
        {
            if !self.focused {
                return None;
            }
            let query = self.text.to_lowercase();
            let filtered: Vec<usize> = trigger
                .candidates
                .iter()
                .enumerate()
                .filter(|(_, c)| c.display.to_lowercase().contains(&query))
                .map(|(i, _)| i)
                .collect();
            if filtered.is_empty() {
                return None;
            }
            return Some(PopupState {
                dynamic_matches: None,
                trigger_index,
                query_start: 0,
                filtered,
                selected: 0,
            });
        }
        let before_cursor = &self.text[..self.cursor];
        let mut best: Option<(usize, usize)> = None;

        for (trigger_index, trigger) in self.triggers.iter().enumerate() {
            let candidate_pos = if trigger.only_at_message_start {
                before_cursor.starts_with(trigger.trigger_char).then_some(0)
            } else {
                before_cursor.rfind(trigger.trigger_char)
            };
            let Some(pos) = candidate_pos else { continue };

            let query_start = pos + trigger.trigger_char.len_utf8();
            if !trigger.multi_word && before_cursor[query_start..].contains(char::is_whitespace) {
                continue;
            }
            if let Some(source) = self.dynamic_sources.get(&trigger_index) {
                let starts_word = before_cursor[..pos]
                    .chars()
                    .next_back()
                    .is_none_or(char::is_whitespace);
                if !starts_word
                    || before_cursor[query_start..].chars().count() < source.min_query_len
                {
                    continue;
                }
            }
            if best.is_none_or(|(_, best_pos)| pos > best_pos) {
                best = Some((trigger_index, pos));
            }
        }

        let (trigger_index, trigger_pos) = best?;
        let trigger = &self.triggers[trigger_index];
        let query_start = trigger_pos + trigger.trigger_char.len_utf8();
        let query = before_cursor[query_start..].to_lowercase();

        if let Some(source) = self.dynamic_sources.get(&trigger_index) {
            let matches = (source.search)(&query);
            if matches.is_empty() {
                return None;
            }
            return Some(PopupState {
                dynamic_matches: Some(matches.clone()),
                trigger_index,
                query_start,
                filtered: (0..matches.len()).collect(),
                selected: 0,
            });
        }

        let filtered: Vec<usize> = trigger
            .candidates
            .iter()
            .enumerate()
            .filter(|(_, c)| c.display.to_lowercase().starts_with(&query))
            .map(|(i, _)| i)
            .collect();
        if filtered.is_empty() {
            return None;
        }

        Some(PopupState {
            dynamic_matches: None,
            trigger_index,
            query_start,
            filtered,
            selected: 0,
        })
    }

    fn move_popup_selection(&mut self, delta: isize) {
        let Some(popup) = &mut self.popup else { return };
        let len = popup.filtered.len() as isize;
        if len == 0 {
            return;
        }
        popup.selected = (popup.selected as isize + delta).rem_euclid(len) as usize;
    }

    fn complete_popup_selection(&mut self) {
        let Some(popup) = self.popup.take() else {
            return;
        };
        let trigger = &self.triggers[popup.trigger_index];
        if let Some(matches) = &popup.dynamic_matches {
            let insertion = format!("{} ", matches[popup.filtered[popup.selected]].glyph);
            let replace_start = popup.query_start - trigger.trigger_char.len_utf8();
            self.text
                .replace_range(replace_start..self.cursor, &insertion);
            self.cursor = replace_start + insertion.len();
            self.notify_change();
            return;
        }
        let candidate = &trigger.candidates[popup.filtered[popup.selected]];
        if let Some(on_select) = candidate.on_select.clone() {
            if trigger.whole_text {
                self.text.clear();
                self.cursor = 0;
            } else {
                let start = popup.query_start - trigger.trigger_char.len_utf8();
                self.text.replace_range(start..self.cursor, "");
                self.cursor = start;
            }
            self.selection_anchor = None;
            self.notify_change();
            on_select();
            return;
        }
        let (replace_start, insertion) = match (&candidate.token, candidate.id) {
            (Some(token), _) => (
                popup.query_start - trigger.trigger_char.len_utf8(),
                format!("{token} "),
            ),
            (None, Some(id)) => (
                popup.query_start - trigger.trigger_char.len_utf8(),
                format!("<{}{}> ", trigger.trigger_char, id),
            ),
            (None, None) => (popup.query_start, format!("{} ", candidate.display)),
        };
        self.text
            .replace_range(replace_start..self.cursor, &insertion);
        self.cursor = replace_start + insertion.len();
        self.notify_change();
    }

    fn submit(&mut self) {
        if self.text.is_empty()
            && !self
                .submit_when_empty
                .as_ref()
                .is_some_and(|allowed| allowed())
        {
            return;
        }
        let content = mem::take(&mut self.text);
        self.cursor = 0;
        self.selection_anchor = None;
        self.popup = None;

        self.history = History::default();
        self.skip_history = true;
        if let Some(on_submit) = &mut self.on_submit {
            on_submit(content);
        }
    }

    fn build_layout(&self, text: &str, cursor: usize) -> Layout {
        if text.is_empty() && self.ime_preedit.is_empty() {
            let live = self
                .placeholder_live
                .as_ref()
                .map(|s| s.peek())
                .filter(|p| !p.is_empty());
            let full = self.build_flat_buffer_sized(
                live.as_deref().unwrap_or(&self.placeholder),
                colors::live(self.placeholder_color),
                self.placeholder_font_size.unwrap_or(self.font_size),
            );
            return Layout {
                full,
                map: None,
                preedit_caret: None,
                emoji_images: Vec::new(),
            };
        }

        if !self.ime_preedit.is_empty() {
            let full = self.build_preedit_buffer(text, cursor);
            let mut caret_text = text[..cursor.min(text.len())].to_string();
            caret_text.push_str(&self.ime_preedit);
            let caret = self.build_flat_buffer(&caret_text, colors::live(self.color));
            return Layout {
                full,
                map: None,
                preedit_caret: Some(caret),
                emoji_images: Vec::new(),
            };
        }

        let (full, map, emoji_images) = self.build_rich_buffer(text);
        Layout {
            full,
            map: Some(map),
            preedit_caret: None,
            emoji_images,
        }
    }

    fn markdown_run_attrs(&self, attrs: &SpanAttrs) -> Attrs<'static> {
        if attrs.delimiter {
            let a = Attrs::new()
                .weight(FdWeight::SEMIBOLD)
                .color(to_cosmic_color(colors::live(TEXT_MUTED)));
            if attrs.subtext {
                return Attrs::new()
                    .weight(FdWeight::BOLD)
                    .color(to_cosmic_color(colors::live(TEXT_MUTED)))
                    .family(Family::Name(gg_sans_family(FdWeight::BOLD, false)))
                    .metrics(SUBTEXT_METRICS);
            }
            return if attrs.code {
                a.family(Family::Name(GG_SANS_MONO_FAMILY))
                    .metrics(self.code_metrics())
                    .metadata(META_CODE)
            } else {
                a.family(Family::Name(gg_sans_family(FdWeight::SEMIBOLD, false)))
            };
        }

        let weight = if attrs.bold {
            FdWeight::BOLD
        } else {
            FdWeight::NORMAL
        };
        let style = if attrs.italic {
            cosmic_text::Style::Italic
        } else {
            cosmic_text::Style::Normal
        };
        let mut a = Attrs::new().weight(weight).style(style);

        if attrs.code {
            a = a
                .family(Family::Name(GG_SANS_MONO_FAMILY))
                .metrics(self.code_metrics())
                .metadata(META_CODE);
        } else {
            a = a.family(Family::Name(gg_sans_family(weight, attrs.italic)));
            if attrs.spoiler {
                a = a.metadata(META_SPOILER);
            }
        }

        a = a.color(if attrs.link {
            to_cosmic_color(colors::live(TEXT_ANCHOR))
        } else if attrs.subtext {
            to_cosmic_color(colors::live(TEXT_MUTED))
        } else {
            to_cosmic_color(colors::live(self.color))
        });
        if attrs.subtext && !attrs.code {
            a = a.metrics(SUBTEXT_METRICS);
        }
        if attrs.strikethrough {
            a = a.strikethrough();
        }
        if attrs.underline {
            a = a.underline(UnderlineStyle::Single);
        }
        a
    }

    fn code_metrics(&self) -> Metrics {
        Metrics::new(
            self.font_size * MARKDOWN_CODE_FONT_SCALE,
            self.line_height
                .unwrap_or(self.font_size * MARKDOWN_CODE_FONT_SCALE * 1.2),
        )
    }

    fn chip_attrs(&self, trigger_index: usize) -> Attrs<'static> {
        let trigger = &self.triggers[trigger_index];

        let weight = if trigger.trigger_char == TIMESTAMP_TRIGGER_CHAR
            || trigger.chip_background.is_none()
        {
            FdWeight::NORMAL
        } else {
            FdWeight::MEDIUM
        };
        let mut a = Attrs::new()
            .family(Family::Name(gg_sans_family(weight, false)))
            .weight(weight);
        if trigger.chip_background.is_some() {
            a = a
                .color(to_cosmic_color(trigger.chip_text_color))
                .metadata(META_CHIP_BASE + trigger_index);
        } else {
            a = a.color(to_cosmic_color(colors::live(self.color)));
        }
        a
    }

    fn background_color_for_metadata(&self, meta: usize) -> Option<Color> {
        match meta {
            META_CODE => Some(colors::background_code()),
            META_SPOILER => Some(BACKGROUND_MOD_SUBTLE),

            m if m >= META_EMOJI_BASE => None,
            m if m >= META_DECORATION_BASE => self
                .decoration_styles
                .get(m - META_DECORATION_BASE)
                .and_then(|d| d.background),
            m if m >= META_CHIP_BASE => self
                .triggers
                .get(m - META_CHIP_BASE)
                .and_then(|t| t.chip_background),
            _ => None,
        }
    }

    fn background_radius_for_metadata(meta: usize) -> f32 {
        match meta {
            META_SPOILER => 4.0,
            _ => 3.0,
        }
    }

    fn background_padding_for_metadata(meta: usize) -> f32 {
        if meta >= META_CHIP_BASE { 2.0 } else { 0.0 }
    }

    fn build_rich_buffer(&self, text: &str) -> (Buffer, LayoutMap, Vec<(Image, Rect)>) {
        self.pending_custom_emoji.borrow_mut().clear();
        let mut font_system = self.font_system.borrow_mut();
        let metrics = Metrics::new(self.font_size, self.line_height());
        let mut buffer = Buffer::new(&mut font_system, metrics);
        let wrap_width = self.text_wrap_width();
        buffer.set_wrap(if wrap_width.is_some() {
            Wrap::WordOrGlyph
        } else {
            Wrap::None
        });
        buffer.set_size(wrap_width, None);

        let hidden = if self.markdown.quotes {
            markdown::quote_prefix_ranges_excluding(text, &self.code_contents(text))
        } else {
            Vec::new()
        };
        let runs = self.tokenize_runs(text);

        let default_attrs = Attrs::new()
            .family(Family::Name(gg_sans_family(FdWeight::NORMAL, false)))
            .color(to_cosmic_color(colors::live(self.color)));
        let mut spans: Vec<(&str, Attrs)> = Vec::new();
        let mut map = LayoutMap::default();
        let mut emoji_images: Vec<(Image, Rect)> = Vec::new();
        for run in &runs {
            match &run.kind {
                RunKind::Chip {
                    trigger_index: PASSWORD_MASK_TRIGGER,
                    display,
                } => {
                    map.push(run.range.clone(), display, true);
                    spans.push((
                        display.as_str(),
                        Attrs::new()
                            .family(Family::Name(gg_sans_family(FdWeight::NORMAL, false)))
                            .color(to_cosmic_color(self.color)),
                    ));
                }
                RunKind::Chip {
                    trigger_index,
                    display,
                } => {
                    if hidden
                        .iter()
                        .any(|h| h.start <= run.range.start && run.range.end <= h.end)
                    {
                        continue;
                    }
                    map.push(run.range.clone(), display, true);
                    spans.push((display.as_str(), self.chip_attrs(*trigger_index)));
                }

                RunKind::Emoji => {
                    let source = &text[run.range.clone()];
                    let picture = self.emoji_picture(source).or_else(|| {
                        let &(_, id, animated) = custom_emoji_tokens(source)
                            .first()
                            .filter(|_| source.starts_with('<'))?;
                        self.pending_custom_emoji.borrow_mut().push((id, animated));
                        let blank =
                            surfaces::raster_n32_premul((1, 1))?.image_snapshot();
                        Some((blank, (0.0, 0.0, 1.0, 1.0)))
                    });
                    let Some((image, src)) = picture else {
                        map.push(run.range.clone(), source, false);
                        spans.push((
                            source,
                            Attrs::new()
                                .family(Family::Name(gg_sans_family(FdWeight::NORMAL, false)))
                                .color(to_cosmic_color(self.color)),
                        ));
                        continue;
                    };
                    let index = emoji_images.len();
                    emoji_images.push((image.clone(), Rect::from_xywh(src.0, src.1, src.2, src.3)));
                    let emoji_font_size = self.font_size * EMOJI_SCALE;
                    let attrs = Attrs::new()
                        .family(Family::Name(gg_sans_family(FdWeight::NORMAL, false)))
                        .color(to_cosmic_color(self.color))
                        .metrics(Metrics::new(
                            emoji_font_size,
                            self.emoji_line_height(emoji_font_size),
                        ))
                        .metadata(META_EMOJI_BASE + index);
                    map.push(run.range.clone(), EMOJI_PLACEHOLDER, true);
                    spans.push((EMOJI_PLACEHOLDER, attrs));
                }
                RunKind::Plain => {
                    map.push(run.range.clone(), &text[run.range.clone()], false);
                    spans.push((
                        &text[run.range.clone()],
                        Attrs::new()
                            .family(Family::Name(gg_sans_family(FdWeight::MEDIUM, false)))
                            .weight(FdWeight::MEDIUM)
                            .color(to_cosmic_color(self.color)),
                    ));
                }
                RunKind::Decorated(index) => {
                    let style = &self.decoration_styles
                        [(*index).min(self.decoration_styles.len().saturating_sub(1))];
                    map.push(run.range.clone(), &text[run.range.clone()], false);
                    spans.push((
                        &text[run.range.clone()],
                        Attrs::new()
                            .family(Family::Name(gg_sans_family(FdWeight(style.weight), false)))
                            .weight(FdWeight(style.weight))
                            .color(to_cosmic_color(style.color))
                            .metadata(META_DECORATION_BASE + *index),
                    ));
                }
                RunKind::Markdown(_) | RunKind::Fence | RunKind::Code(_) => {
                    let run_attrs = match &run.kind {
                        RunKind::Markdown(attrs) => self.markdown_run_attrs(attrs),

                        RunKind::Fence => Attrs::new()
                            .family(Family::Name(gg_sans_family(FdWeight::NORMAL, false)))
                            .color(to_cosmic_color(TEXT_MUTED)),
                        RunKind::Code(color) => Attrs::new()
                            .family(Family::Name(GG_SANS_MONO_FAMILY))
                            .metrics(self.code_metrics())
                            .color(to_cosmic_color(color.unwrap_or(self.color))),
                        _ => unreachable!(),
                    };
                    for visible in markdown::visible_subranges(run.range.clone(), &hidden) {
                        if visible.is_empty() {
                            continue;
                        }
                        map.push(visible.clone(), &text[visible.clone()], false);
                        spans.push((&text[visible], run_attrs.clone()));
                    }
                }
            }
        }

        if text.ends_with('\n') {
            spans.push(("\u{200B}", default_attrs.clone()));
        }
        buffer.set_rich_text(spans, &default_attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut font_system, false);
        map.finish(text.len());
        (buffer, map, emoji_images)
    }

    fn build_preedit_buffer(&self, text: &str, cursor: usize) -> Buffer {
        let cursor = cursor.min(text.len());
        let mut font_system = self.font_system.borrow_mut();
        let metrics = Metrics::new(self.font_size, self.line_height());
        let mut buffer = Buffer::new(&mut font_system, metrics);
        let wrap_width = self.text_wrap_width();
        buffer.set_wrap(if wrap_width.is_some() {
            Wrap::WordOrGlyph
        } else {
            Wrap::None
        });
        buffer.set_size(wrap_width, None);

        let base = Attrs::new()
            .family(Family::Name(gg_sans_family(FdWeight::NORMAL, false)))
            .color(to_cosmic_color(colors::live(self.color)));
        let preedit_attrs = base.clone().underline(UnderlineStyle::Single);
        let spans = [
            (&text[..cursor], base.clone()),
            (self.ime_preedit.as_str(), preedit_attrs),
            (&text[cursor..], base.clone()),
        ];
        buffer.set_rich_text(spans, &base, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut font_system, false);
        buffer
    }

    fn build_flat_buffer(&self, text: &str, color: Color) -> Buffer {
        self.build_flat_buffer_sized(text, color, self.font_size)
    }

    fn build_flat_buffer_sized(&self, text: &str, color: Color, font_size: f32) -> Buffer {
        let mut font_system = self.font_system.borrow_mut();
        let metrics = Metrics::new(font_size, self.line_height.unwrap_or(font_size * 1.2));
        let mut buffer = Buffer::new(&mut font_system, metrics);
        let wrap_width = self.text_wrap_width();
        buffer.set_wrap(if wrap_width.is_some() {
            Wrap::WordOrGlyph
        } else {
            Wrap::None
        });
        buffer.set_size(wrap_width, None);
        let attrs = Attrs::new()
            .family(Family::Name(gg_sans_family(FdWeight::NORMAL, false)))
            .color(to_cosmic_color(color));
        buffer.set_text(text, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut font_system, false);
        buffer
    }

    fn cursor_index_for_point(&self, rect: AbsoluteRect, x: f32, y: f32) -> usize {
        if self.text.is_empty() {
            return 0;
        }
        let measure_h = self.with_layout(|layout| buffer_extent(&layout.full).1);
        let text_y = self.text_top(rect, measure_h);
        self.cursor_index_for_local_point(x - self.text_left(rect), y - text_y)
    }

    fn code_contents(&self, text: &str) -> Vec<Range<usize>> {
        if !self.markdown.code {
            return Vec::new();
        }
        markdown::composer_code_blocks(text)
            .into_iter()
            .map(|b| b.content)
            .collect()
    }

    fn caret_in_code_block(&self) -> bool {
        self.markdown.code
            && markdown::composer_code_blocks(&self.text).iter().any(|b| {
                b.open.end <= self.cursor && b.close.as_ref().is_none_or(|c| self.cursor <= c.start)
            })
    }

    fn code_runs(&self, text: &str, block: &markdown::ComposerCodeBlock) -> Vec<Run> {
        let content = block.content.clone();
        if content.is_empty() {
            return Vec::new();
        }
        let colors = match (&block.lang, &self.code_highlighter) {
            (Some(lang), Some(highlighter)) if !block.lines.is_empty() => {
                self.highlight(highlighter, lang, &text[block.lines.clone()])
            }
            _ => Rc::new(Vec::new()),
        };
        let mut runs = Vec::new();
        let mut pos = content.start;
        for (range, color) in colors.iter() {
            let range = block.lines.start + range.start..block.lines.start + range.end;
            if range.start < pos
                || range.end > content.end
                || !text.is_char_boundary(range.start)
                || !text.is_char_boundary(range.end)
            {
                continue;
            }
            if pos < range.start {
                runs.push(Run {
                    range: pos..range.start,
                    kind: RunKind::Code(None),
                });
            }
            pos = range.end;
            runs.push(Run {
                range,
                kind: RunKind::Code(Some(*color)),
            });
        }
        if pos < content.end {
            runs.push(Run {
                range: pos..content.end,
                kind: RunKind::Code(None),
            });
        }
        runs
    }

    fn highlight(
        &self,
        highlighter: &CodeHighlighter,
        lang: &str,
        code: &str,
    ) -> Rc<Vec<(Range<usize>, Color)>> {
        const SLOTS: usize = 8;

        let mut cache = self.highlight_cache.borrow_mut();
        if let Some(index) = cache.iter().position(|(l, c, _)| l == lang && c == code) {
            let entry = cache.remove(index);
            let colors = entry.2.clone();
            cache.push(entry);
            return colors;
        }
        let colors = Rc::new(highlighter(lang, code));

        if cache.len() >= SLOTS {
            cache.remove(0);
        }
        cache.push((lang.to_string(), code.to_string(), colors.clone()));
        colors
    }

    fn indent_code_lines(&mut self, dedent: bool) -> bool {
        if !self.markdown.code || self.read_only || self.disabled {
            return false;
        }
        let selection = self.selection_range().unwrap_or(self.cursor..self.cursor);
        let first_line = self.text[..selection.start]
            .rfind('\n')
            .map_or(0, |i| i + 1);
        let last_line_end = self.text[selection.end..]
            .find('\n')
            .map_or(self.text.len(), |i| selection.end + i);
        let in_code = markdown::composer_code_blocks(&self.text).iter().any(|b| {
            b.content.start <= first_line
                && last_line_end <= b.content.end
                && b.close.as_ref().is_none_or(|c| last_line_end < c.start)
        });
        if !in_code {
            return false;
        }
        if !dedent && selection.is_empty() {
            self.insert("  ");
            return true;
        }
        let mut line_starts: Vec<usize> = vec![first_line];
        line_starts.extend(
            self.text[first_line..last_line_end]
                .match_indices('\n')
                .map(|(i, _)| first_line + i + 1),
        );

        let mut edits: Vec<(usize, isize)> = Vec::new();
        for &start in line_starts.iter().rev() {
            if dedent {
                if self.text[start..].starts_with("  ") {
                    self.text.replace_range(start..start + 2, "");
                    edits.push((start, -2));
                }
            } else {
                self.text.insert_str(start, "  ");
                edits.push((start, 2));
            }
        }
        if edits.is_empty() {
            return true;
        }

        let shift = |offset: usize| -> usize {
            let delta: isize = edits
                .iter()
                .map(|&(at, d)| match d {
                    2 if offset >= at => 2,
                    -2 if offset >= at + 2 => -2,
                    -2 if offset > at => at as isize - offset as isize,
                    _ => 0,
                })
                .sum();
            (offset as isize + delta).max(0) as usize
        };
        let anchor = self.selection_anchor.unwrap_or(self.cursor);
        self.selection_anchor = Some(shift(anchor).min(self.text.len()));
        self.cursor = shift(self.cursor).min(self.text.len());
        self.notify_change();
        true
    }

    fn tokenize_runs(&self, text: &str) -> Vec<Run> {
        if let Some(decorator) = &self.decorator {
            let mut decorations = decorator(text);
            decorations.sort_by_key(|d| d.range.start);
            let mut runs = Vec::new();
            let mut pos = 0;
            for decoration in decorations {
                let range = decoration.range;
                if range.start < pos
                    || range.end > text.len()
                    || range.is_empty()
                    || !text.is_char_boundary(range.start)
                    || !text.is_char_boundary(range.end)
                {
                    continue;
                }
                if pos < range.start {
                    runs.push(Run {
                        range: pos..range.start,
                        kind: RunKind::Plain,
                    });
                }
                pos = range.end;
                runs.push(Run {
                    range,
                    kind: RunKind::Decorated(decoration.style),
                });
            }
            if pos < text.len() {
                runs.push(Run {
                    range: pos..text.len(),
                    kind: RunKind::Plain,
                });
            }
            return runs;
        }

        let mut atoms: Vec<Run> = self
            .find_chip_ranges(text)
            .into_iter()
            .map(|chip| Run {
                range: chip.range,
                kind: if chip.is_emoji {
                    RunKind::Emoji
                } else {
                    RunKind::Chip {
                        trigger_index: chip.trigger_index,
                        display: chip.display,
                    }
                },
            })
            .collect();
        if self.markdown.code {
            for block in markdown::composer_code_blocks(text) {
                atoms.push(Run {
                    range: block.open.clone(),
                    kind: RunKind::Fence,
                });
                atoms.extend(self.code_runs(text, &block));
                if let Some(close) = block.close {
                    atoms.push(Run {
                        range: close,
                        kind: RunKind::Fence,
                    });
                }
            }
            atoms.sort_by_key(|run| run.range.start);
        }
        let mut runs = Vec::new();
        let mut pos = 0;

        for atom in atoms {
            if pos < atom.range.start {
                for span in markdown::tokenize(&text[pos..atom.range.start], self.markdown) {
                    runs.push(Run {
                        range: pos + span.range.start..pos + span.range.end,
                        kind: RunKind::Markdown(span.attrs),
                    });
                }
            }
            pos = atom.range.end;
            runs.push(atom);
        }
        if pos < text.len() {
            for span in markdown::tokenize(&text[pos..], self.markdown) {
                runs.push(Run {
                    range: pos + span.range.start..pos + span.range.end,
                    kind: RunKind::Markdown(span.attrs),
                });
            }
        }
        runs
    }

    fn find_chip_ranges(&self, text: &str) -> Vec<ChipMatch> {
        if self.password {
            return text
                .char_indices()
                .map(|(i, c)| ChipMatch {
                    range: i..i + c.len_utf8(),
                    trigger_index: PASSWORD_MASK_TRIGGER,
                    display: PASSWORD_MASK.to_string(),
                    is_emoji: false,
                })
                .collect();
        }
        let mut matches = self.scan_mention_tokens(text);
        matches.extend(self.scan_plaintext_matches(text));
        matches.extend(self.scan_emoji_graphemes(text));
        matches.extend(self.scan_custom_emoji_tokens(text));
        matches.extend(self.scan_timestamp_tokens(text));
        if self.markdown.code {
            let blocks: Vec<Range<usize>> = markdown::composer_code_blocks(text)
                .iter()
                .map(|b| b.full(text.len()))
                .collect();
            matches.retain(|m| {
                !blocks
                    .iter()
                    .any(|range| range.start < m.range.end && m.range.start < range.end)
            });
        }
        matches.sort_by_key(|m| m.range.start);

        let mut merged: Vec<ChipMatch> = Vec::new();
        for m in matches {
            if merged
                .last()
                .is_none_or(|prev: &ChipMatch| prev.range.end <= m.range.start)
            {
                merged.push(m);
            }
        }
        merged
    }

    fn prev_cursor_position(&self, pos: usize) -> usize {
        let chips = self.find_chip_ranges(&self.text);
        if let Some(chip) = chips
            .iter()
            .find(|c| pos > c.range.start && pos <= c.range.end)
        {
            return chip.range.start;
        }
        prev_grapheme_boundary(&self.text, pos).unwrap_or(pos)
    }

    fn next_cursor_position(&self, pos: usize) -> usize {
        let chips = self.find_chip_ranges(&self.text);
        if let Some(chip) = chips
            .iter()
            .find(|c| pos >= c.range.start && pos < c.range.end)
        {
            return chip.range.end;
        }
        next_grapheme_boundary(&self.text, pos).unwrap_or(pos)
    }

    fn atom_ranges(&self) -> Vec<Range<usize>> {
        self.find_chip_ranges(&self.text)
            .into_iter()
            .map(|c| c.range)
            .collect()
    }

    fn prev_word_position(&self, pos: usize) -> usize {
        prev_word_boundary(&self.text, &self.atom_ranges(), pos)
    }

    fn next_word_position(&self, pos: usize) -> usize {
        next_word_boundary(&self.text, &self.atom_ranges(), pos, cfg!(windows))
    }

    fn move_caret(&mut self, target: usize, shift: bool) {
        if shift {
            self.extend_selection();
        } else {
            self.clear_selection();
        }
        self.cursor = target;
        self.refresh_popup();
    }

    fn delete_word(&mut self, forward: bool) {
        if self.read_only || self.disabled {
            return;
        }
        if !self.delete_selection() {
            let (start, end) = if forward {
                (self.cursor, self.next_word_position(self.cursor))
            } else {
                (self.prev_word_position(self.cursor), self.cursor)
            };
            if start == end {
                return;
            }
            self.text.replace_range(start..end, "");
            self.cursor = start;
        }
        self.notify_change();
        self.refresh_popup();
    }

    fn caret_geometry(&self) -> (f32, f32, f32) {
        self.with_layout(|layout| self.caret_in(layout))
    }

    fn caret_in(&self, layout: &Layout) -> (f32, f32, f32) {
        let cursor = self.cursor.min(self.text.len());
        if let Some(caret) = &layout.preedit_caret {
            let prefix = format!("{}{}", &self.text[..cursor], self.ime_preedit);
            let (top, height) = caret_line_from_buffer(caret, self.line_height());
            return (caret_x_from_buffer(caret, &prefix), top, height);
        }
        match &layout.map {
            Some(map) => {
                caret_in_buffer(&layout.full, map.cursor(cursor, false), self.line_height())
            }

            None => (0.0, 0.0, self.line_height()),
        }
    }

    fn cursor_index_for_local_point(&self, local_x: f32, local_y: f32) -> usize {
        if self.text.is_empty() {
            return 0;
        }
        let pos = self.with_layout(|layout| {
            let Some(map) = &layout.map else {
                return self.text.len();
            };
            match layout.full.hit(local_x, local_y) {
                Some(cursor) => map.to_source(cursor_to_byte(&map.text, cursor)),
                None => self.text.len(),
            }
        });
        snap_to_grapheme_boundary(&self.text, pos)
    }

    fn move_vertically(&mut self, down: bool, shift: bool) {
        let (caret_x, top, height) = self.caret_geometry();
        let goal = *self.goal_x.get_or_insert(caret_x);
        let text_h = self.with_layout(|layout| buffer_extent(&layout.full).1);
        let target = if down {
            let y = top + height + self.line_height() / 2.0;
            if y >= text_h {
                self.text.len()
            } else {
                self.cursor_index_for_local_point(goal, y)
            }
        } else {
            let y = top - self.line_height() / 2.0;
            if y < 0.0 {
                0
            } else {
                self.cursor_index_for_local_point(goal, y)
            }
        };
        self.move_caret(target, shift);
    }

    fn move_to_line_edge(&mut self, end: bool, shift: bool) {
        let target = if !self.wraps() {
            if end { self.text.len() } else { 0 }
        } else {
            let (_, top, height) = self.caret_geometry();
            let y = top + height / 2.0;
            let pos = self.cursor_index_for_local_point(if end { 1.0e6 } else { -1.0e6 }, y);

            let before = self.text[..pos].chars().next_back();
            if end
                && pos < self.text.len()
                && before.is_some_and(|c| c.is_whitespace() && c != '\n')
            {
                prev_grapheme_boundary(&self.text, pos).unwrap_or(pos)
            } else {
                pos
            }
        };
        self.move_caret(target, shift);
    }

    fn scan_mention_tokens(&self, text: &str) -> Vec<ChipMatch> {
        let mut matches = Vec::new();
        for (trigger_index, trigger) in self.triggers.iter().enumerate() {
            let open = format!("<{}", trigger.trigger_char);
            let mut search_from = 0;
            while let Some(rel) = text[search_from..].find(open.as_str()) {
                let start = search_from + rel;

                let role =
                    trigger.trigger_char == '@' && text[start + open.len()..].starts_with('&');
                let digits_start = start + open.len() + usize::from(role);
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
                let found = if role {
                    trigger
                        .candidates
                        .iter()
                        .find(|c| c.token.as_deref() == Some(&text[start..end]))
                } else {
                    trigger
                        .candidates
                        .iter()
                        .find(|c| c.id == Some(id) && c.token.is_none())
                };
                let display = match found {
                    Some(candidate) => format!("{}{}", trigger.trigger_char, candidate.display),
                    None => format!("{}{}", trigger.trigger_char, id),
                };
                matches.push(ChipMatch {
                    range: start..end,
                    trigger_index,
                    display,
                    is_emoji: false,
                });
            }
        }
        matches
    }

    fn scan_plaintext_matches(&self, text: &str) -> Vec<ChipMatch> {
        let mut matches = Vec::new();
        for (trigger_index, trigger) in self.triggers.iter().enumerate() {
            let mut search_from = 0;
            while let Some(rel) = text[search_from..].find(trigger.trigger_char) {
                let pos = search_from + rel;
                let after = pos + trigger.trigger_char.len_utf8();
                search_from = after;

                let matched = trigger.candidates.iter().find(|c| {
                    text[after..].starts_with(c.display.as_str())
                        && text[after + c.display.len()..]
                            .chars()
                            .next()
                            .is_none_or(|ch| !ch.is_alphanumeric())
                });
                if let Some(candidate) = matched {
                    let end = after + candidate.display.len();
                    matches.push(ChipMatch {
                        range: pos..end,
                        trigger_index,
                        display: text[pos..end].to_string(),
                        is_emoji: false,
                    });
                }
            }
        }
        matches
    }

    fn scan_custom_emoji_tokens(&self, text: &str) -> Vec<ChipMatch> {
        if self.custom_emoji.is_none() {
            return Vec::new();
        }
        custom_emoji_tokens(text)
            .into_iter()
            .map(|(range, _, _)| ChipMatch {
                range,
                trigger_index: 0,
                display: EMOJI_PLACEHOLDER.to_string(),
                is_emoji: true,
            })
            .collect()
    }

    fn scan_timestamp_tokens(&self, text: &str) -> Vec<ChipMatch> {
        let Some(trigger_index) = self
            .triggers
            .iter()
            .position(|t| t.trigger_char == TIMESTAMP_TRIGGER_CHAR)
        else {
            return Vec::new();
        };
        timestamp::tokens(text)
            .into_iter()
            .filter_map(|(range, unix, letter)| {
                Some(ChipMatch {
                    range,
                    trigger_index,
                    display: timestamp::format(unix, letter)?,
                    is_emoji: false,
                })
            })
            .collect()
    }

    fn scan_emoji_graphemes(&self, text: &str) -> Vec<ChipMatch> {
        let Some(provider) = self.emoji_provider else {
            return Vec::new();
        };
        text.grapheme_indices(true)
            .filter(|(_, grapheme)| provider(grapheme).is_some())
            .map(|(offset, grapheme)| ChipMatch {
                range: offset..offset + grapheme.len(),
                trigger_index: 0,
                display: EMOJI_PLACEHOLDER.to_string(),
                is_emoji: true,
            })
            .collect()
    }

    fn with_layout<R>(&self, f: impl FnOnce(&Layout) -> R) -> R {
        let mut cache = self.cache.borrow_mut();
        let width_key = self.text_wrap_width().map(f32::to_bits);
        let live = self.placeholder_live.as_ref().map(|s| s.peek());
        let live_changed = *self.live_placeholder_seen.borrow() != live;

        let cursor_key = (!self.ime_preedit.is_empty()).then_some(self.cursor);
        let stale = live_changed
            || match &*cache {
                Some((cached_text, cached_cursor, cached_preedit, cached_width, _)) => {
                    cached_text != &self.text
                        || *cached_cursor != cursor_key
                        || cached_preedit != &self.ime_preedit
                        || *cached_width != width_key
                }
                None => true,
            };
        if stale {
            let layout = self.build_layout(&self.text, self.cursor);
            *cache = Some((
                self.text.clone(),
                cursor_key,
                self.ime_preedit.clone(),
                width_key,
                layout,
            ));
            *self.live_placeholder_seen.borrow_mut() = live;
        }
        let (.., layout) = cache.as_ref().expect("just populated above");
        f(layout)
    }

    fn popup_slots(&self, popup: &PopupState) -> Vec<RowSlot> {
        let mut slots = Vec::with_capacity(popup.filtered.len());
        let mut top = 0.0;
        let mut previous_section = None;
        for &index in &popup.filtered {
            if popup.dynamic_matches.is_some() {
                slots.push(RowSlot {
                    top,
                    height: POPUP_SMALL_ROW_HEIGHT,
                    divider: false,
                    heading: None,
                });
                top += POPUP_SMALL_ROW_HEIGHT;
                continue;
            }
            let section = self.triggers[popup.trigger_index].candidates[index].section;
            let starts_section = previous_section.is_some_and(|previous| previous != section);
            let heading = starts_section
                .then(|| {
                    self.triggers[popup.trigger_index].candidates[index]
                        .section_label
                        .clone()
                })
                .flatten();
            let divider = starts_section && heading.is_none();
            if divider {
                top += POPUP_DIVIDER_HEIGHT;
            } else if heading.is_some() {
                top += POPUP_HEADER_HEIGHT;
            }
            let height = if section > 0 {
                POPUP_SECTION_ROW_HEIGHT
            } else {
                POPUP_ROW_HEIGHT
            };
            slots.push(RowSlot {
                top,
                height,
                divider,
                heading,
            });
            top += height;
            previous_section = Some(section);
        }
        slots
    }

    fn popup_geometry(&self, rect: AbsoluteRect, popup: &PopupState) -> PopupGeometry {
        let (outset_left, outset_right) = self.popup_outset;
        let rect = self.popup_frame.as_ref().map_or(rect, |frame| frame.peek());
        let row_height = if popup.dynamic_matches.is_some() {
            POPUP_SMALL_ROW_HEIGHT
        } else {
            POPUP_ROW_HEIGHT
        };
        let width = rect.width + outset_left + outset_right;
        let rows_height = self
            .popup_slots(popup)
            .last()
            .map_or(0.0, |slot| slot.top + slot.height);
        let height = POPUP_HEADER_HEIGHT + rows_height + POPUP_BOTTOM_PADDING;
        let top = rect.y - height - POPUP_GAP;
        PopupGeometry {
            panel: Rect::from_xywh(rect.x - outset_left, top, width, height),
            rows_top: top + POPUP_HEADER_HEIGHT,
            row_height,
        }
    }

    fn popup_hit(&self, rect: AbsoluteRect, x: f32, y: f32) -> Option<PopupHit> {
        let popup = self.popup.as_ref()?;
        let geometry = self.popup_geometry(rect, popup);
        let panel = geometry.panel;
        if x < panel.left || x >= panel.right || y < panel.top || y >= panel.bottom {
            return None;
        }
        let inside = y - geometry.rows_top;
        let row = self
            .popup_slots(popup)
            .iter()
            .position(|slot| inside >= slot.top && inside < slot.top + slot.height);
        Some(row.map_or(PopupHit::Chrome, PopupHit::Row))
    }

    fn paint_popup(&self, canvas: &Canvas, rect: AbsoluteRect, popup: &PopupState) {
        const ROW_INSET: f32 = 8.0;

        const ROW_PAD_X: f32 = 8.0;
        const FILL_RADIUS: f32 = 3.0;
        const AVATAR_SIZE: f32 = 24.0;
        const RING: Color = Color::from_argb(0x14, 0xff, 0xff, 0xff);
        const DROP_SHADOW: Color = Color::from_argb(0x3d, 0x00, 0x00, 0x00);

        let trigger = &self.triggers[popup.trigger_index];
        let query = &self.text[popup.query_start..self.cursor.max(popup.query_start)];
        let geometry = self.popup_geometry(rect, popup);
        let panel_bounds = geometry.panel;
        let (panel_x, panel_y, panel_width) =
            (panel_bounds.left, panel_bounds.top, panel_bounds.width());

        let mut shadow_paint = Paint::default();
        shadow_paint.set_color(DROP_SHADOW);
        shadow_paint.set_anti_alias(true);
        shadow_paint.set_mask_filter(skia_safe::MaskFilter::blur(
            skia_safe::BlurStyle::Normal,
            12.0,
            false,
        ));
        canvas.draw_rrect(
            RRect::new_rect_xy(panel_bounds.with_offset((0.0, 12.0)), 5.0, 5.0),
            &shadow_paint,
        );

        let mut ring_paint = Paint::default();
        ring_paint.set_color(RING);
        ring_paint.set_anti_alias(true);
        canvas.draw_rrect(
            RRect::new_rect_xy(panel_bounds.with_outset((1.0, 1.0)), 6.0, 6.0),
            &ring_paint,
        );

        let mut bg_paint = Paint::default();
        bg_paint.set_color(SURFACE_HIGH);
        bg_paint.set_anti_alias(true);
        canvas.draw_rrect(RRect::new_rect_xy(panel_bounds, 5.0, 5.0), &bg_paint);

        let mut font_system = self.font_system.borrow_mut();

        let header_text = if trigger.whole_text {
            trigger.popup_label.to_uppercase()
        } else {
            format!(
                "{}{}{}",
                format!("{} matching ", trigger.popup_label).to_uppercase(),
                trigger.trigger_char,
                query
            )
        };
        let header_buffer = Self::shape_flat(
            &mut font_system,
            &header_text,
            12.0,
            FdWeight::SEMIBOLD,
            TEXT_SECONDARY,
            Some(panel_width - ROW_PAD_X * 2.0),
        );
        let (_, header_h) = buffer_extent(&header_buffer);

        Self::paint_flat_buffer(
            canvas,
            &self.glyph_cache,
            &mut font_system,
            &header_buffer,
            (
                panel_x + ROW_PAD_X,
                panel_y + 8.0 + 4.0 + (16.0 - header_h) / 2.0,
            ),
            TEXT_SECONDARY,
        );

        let rows_top = geometry.rows_top;

        let slots = self.popup_slots(popup);
        for (row_index, &candidate_index) in popup.filtered.iter().enumerate() {
            let slot = &slots[row_index];
            let row_y = rows_top + slot.top;
            let row_height = slot.height;
            if let Some(label) = &slot.heading {
                let heading = Self::shape_flat(
                    &mut font_system,
                    &label.to_uppercase(),
                    12.0,
                    FdWeight::SEMIBOLD,
                    TEXT_SECONDARY,
                    Some(panel_width - ROW_PAD_X * 2.0),
                );
                let (_, heading_h) = buffer_extent(&heading);
                Self::paint_flat_buffer(
                    canvas,
                    &self.glyph_cache,
                    &mut font_system,
                    &heading,
                    (
                        panel_x + ROW_PAD_X,
                        row_y - POPUP_HEADER_HEIGHT + 8.0 + 4.0 + (16.0 - heading_h) / 2.0,
                    ),
                    TEXT_SECONDARY,
                );
            }
            if slot.divider {
                let mut line = Paint::default();
                line.set_color(BACKGROUND_MOD_SUBTLE);
                canvas.draw_rect(
                    Rect::from_xywh(
                        panel_x + ROW_INSET,
                        row_y - POPUP_DIVIDER_HEIGHT / 2.0 - 0.5,
                        panel_width - ROW_INSET * 2.0,
                        1.0,
                    ),
                    &line,
                );
            }

            if row_index == popup.selected {
                let mut hover_paint = Paint::default();
                hover_paint.set_color(BACKGROUND_MOD_SUBTLE);
                hover_paint.set_anti_alias(true);
                let fill = Rect::from_xywh(
                    panel_x + ROW_INSET,
                    row_y,
                    panel_width - ROW_INSET * 2.0,
                    row_height,
                );
                canvas.draw_rrect(
                    RRect::new_rect_xy(fill, FILL_RADIUS, FILL_RADIUS),
                    &hover_paint,
                );
            }

            let content_x = panel_x + ROW_INSET + ROW_PAD_X;
            let content_right = panel_x + panel_width - ROW_INSET - ROW_PAD_X;

            if let Some(matches) = &popup.dynamic_matches {
                const GLYPH_SIZE: f32 = 20.0;
                const GLYPH_GAP: f32 = 8.0;
                let found = &matches[candidate_index];
                let mut text_x = content_x;
                if let Some((atlas, (sx, sy, sw, sh))) = self.emoji_picture(&found.glyph) {
                    let mut glyph_paint = Paint::default();
                    glyph_paint.set_anti_alias(true);
                    let dst = Rect::from_xywh(
                        text_x,
                        row_y + (row_height - GLYPH_SIZE) / 2.0,
                        GLYPH_SIZE,
                        GLYPH_SIZE,
                    );
                    super::draw_atlas_tile(
                        canvas,
                        &atlas,
                        Rect::from_xywh(sx, sy, sw, sh),
                        dst,
                        &glyph_paint,
                    );
                }
                text_x += GLYPH_SIZE + GLYPH_GAP;
                let label = format!(":{}:", found.label);
                let label_buffer = Self::shape_flat(
                    &mut font_system,
                    &label,
                    16.0,
                    FdWeight::NORMAL,
                    TEXT_STRONG,
                    Some((content_right - text_x).max(0.0)),
                );
                let (_, lh) = buffer_extent(&label_buffer);
                Self::paint_flat_buffer(
                    canvas,
                    &self.glyph_cache,
                    &mut font_system,
                    &label_buffer,
                    (text_x, row_y + (row_height - lh) / 2.0),
                    TEXT_STRONG,
                );
                continue;
            }
            let candidate = &trigger.candidates[candidate_index];

            let mut text_x = content_x;
            if candidate.avatar_color.is_some() || candidate.avatar.is_some() {
                let avatar_y = row_y + (row_height - AVATAR_SIZE) / 2.0;
                let initial = candidate
                    .display
                    .chars()
                    .next()
                    .map_or(String::new(), |c| c.to_uppercase().to_string());
                let mut avatar = super::AvatarWidget::new(
                    candidate.avatar.as_ref().and_then(|resolve| resolve()),
                    initial,
                    candidate.avatar_color.unwrap_or(Color::TRANSPARENT),
                    skia_safe::FontMgr::default()
                        .legacy_make_typeface(None, skia_safe::FontStyle::normal())
                        .expect("a system typeface"),
                    super::AvatarShape::Circle,
                )
                .with_fallback_icon(candidate.fallback_icon);
                if candidate.status.is_some() {
                    avatar = avatar.with_status_cutout(Some(super::StatusCutout::ROUND_24));
                }
                let style = ComputedStyle::default();
                avatar.paint(
                    canvas,
                    AbsoluteRect {
                        x: text_x,
                        y: avatar_y,
                        width: AVATAR_SIZE,
                        height: AVATAR_SIZE,
                    },
                    &style,
                );
                if let Some(status) = candidate.status {
                    let dot = super::StatusIndicatorWidget::new(
                        status,
                        super::status::status_dot_color(status),
                    );
                    dot.paint(
                        canvas,
                        AbsoluteRect {
                            x: text_x + 16.0,
                            y: avatar_y + 16.0,
                            width: 8.0,
                            height: 8.0,
                        },
                        &style,
                    );
                }
                text_x += AVATAR_SIZE + 8.0;
            }

            if let Some(subtitle) = &candidate.subtitle {
                let sub_buffer = Self::shape_flat(
                    &mut font_system,
                    subtitle,
                    12.0,
                    FdWeight::NORMAL,
                    TEXT_SECONDARY,
                    None,
                );
                let (sw, sh) = buffer_extent(&sub_buffer);
                let sub_x = content_right - sw;
                Self::paint_flat_buffer(
                    canvas,
                    &self.glyph_cache,
                    &mut font_system,
                    &sub_buffer,
                    (sub_x, row_y + (row_height - sh) / 2.0),
                    TEXT_SECONDARY,
                );
            }

            let has_avatar = candidate.avatar_color.is_some() || candidate.avatar.is_some();
            let primary_color = candidate.name_color.unwrap_or(TEXT_STRONG);
            let primary_text = if (has_avatar && !candidate.show_trigger) || trigger.whole_text {
                candidate.display.clone()
            } else {
                format!("{}{}", trigger.trigger_char, candidate.display)
            };
            let primary_buffer = Self::shape_flat(
                &mut font_system,
                &primary_text,
                16.0,
                FdWeight::NORMAL,
                primary_color,
                Some((content_right - text_x).max(0.0)),
            );
            let (_, ph) = buffer_extent(&primary_buffer);
            Self::paint_flat_buffer(
                canvas,
                &self.glyph_cache,
                &mut font_system,
                &primary_buffer,
                (text_x, row_y + (row_height - ph) / 2.0),
                primary_color,
            );
        }
    }

    fn shape_flat(
        font_system: &mut FontSystem,
        text: &str,
        font_size: f32,
        weight: FdWeight,
        color: Color,
        max_width: Option<f32>,
    ) -> Buffer {
        let metrics = Metrics::new(font_size, font_size * 1.2);
        let mut buffer = Buffer::new(font_system, metrics);
        buffer.set_wrap(Wrap::WordOrGlyph);
        buffer.set_size(max_width, None);
        let attrs = Attrs::new()
            .family(Family::Name(gg_sans_family(weight, false)))
            .weight(weight)
            .color(to_cosmic_color(color));
        buffer.set_text(text, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(font_system, false);
        buffer
    }

    fn paint_flat_buffer(
        canvas: &Canvas,
        glyph_cache: &GlyphCache,
        font_system: &mut FontSystem,
        buffer: &Buffer,
        pos: (f32, f32),
        default_color: Color,
    ) {
        let default_color = to_cosmic_color(default_color);
        let scale_factor = app::current_scale_factor();
        let pos_px = (pos.0 * scale_factor, pos.1 * scale_factor);
        canvas.save();
        canvas.scale((1.0 / scale_factor, 1.0 / scale_factor));
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let physical_glyph = glyph.physical((0., run.line_y * scale_factor), scale_factor);
                let glyph_color = glyph.color_opt.map_or(default_color, |c| c);
                paint_glyph(
                    canvas,
                    glyph_cache,
                    font_system,
                    physical_glyph,
                    glyph_color,
                    pos_px,
                );
            }
        }
        canvas.restore();
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
enum TextDirection {
    LTR,
    RTL,
}

#[allow(dead_code)]
fn detect_text_direction(text: &str) -> TextDirection {
    for ch in text.chars() {
        let cp = ch as u32;
        let is_rtl = matches!(cp,
            0x0590..=0x05FF
            | 0x0600..=0x06FF
            | 0x0750..=0x077F
            | 0x08A0..=0x08FF
            | 0xFB1D..=0xFDFF
            | 0xFE70..=0xFEFF
        );
        if is_rtl {
            return TextDirection::RTL;
        }
        if ch.is_alphabetic() {
            return TextDirection::LTR;
        }
    }
    TextDirection::LTR
}

#[allow(dead_code)]
fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

#[allow(dead_code)]
fn utf16_offset_to_byte(text: &str, utf16_offset: usize) -> usize {
    let mut seen = 0;
    for (byte_idx, ch) in text.char_indices() {
        if seen >= utf16_offset {
            return byte_idx;
        }
        seen += ch.len_utf16();
    }
    text.len()
}

fn word_range_at(text: &str, pos: usize) -> Range<usize> {
    let pos = pos.min(text.len());
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    let Some(class_char) = text[pos..]
        .chars()
        .next()
        .or_else(|| text[..pos].chars().next_back())
    else {
        return pos..pos;
    };
    let target_is_word = is_word(class_char);

    if !target_is_word && !class_char.is_whitespace() && !class_char.is_ascii() {
        let at = snap_to_grapheme_boundary(text, pos);
        let end = next_grapheme_boundary(text, at).unwrap_or(text.len());
        return if pos == text.len() {
            prev_grapheme_boundary(text, pos).unwrap_or(0)..pos
        } else {
            at..end
        };
    }

    let mut start = pos;
    for (idx, ch) in text[..pos].char_indices().rev() {
        if is_word(ch) != target_is_word {
            break;
        }
        start = idx;
    }
    let mut end = pos;
    for (idx, ch) in text[pos..].char_indices() {
        if is_word(ch) != target_is_word {
            break;
        }
        end = pos + idx + ch.len_utf8();
    }
    start..end
}

fn caret_x_from_buffer(caret_buffer: &Buffer, prefix: &str) -> f32 {
    if prefix.is_empty() {
        return 0.0;
    }
    let Some(run) = caret_buffer.layout_runs().last() else {
        return 0.0;
    };
    match run.glyphs.last() {
        Some(g) if g.level.is_rtl() => g.x,
        Some(g) => g.x + g.w,
        None => 0.0,
    }
}

fn caret_line_from_buffer(caret_buffer: &Buffer, fallback_height: f32) -> (f32, f32) {
    caret_buffer
        .layout_runs()
        .last()
        .map_or((0.0, fallback_height), |run| {
            (run.line_top, run.line_height)
        })
}

fn caret_in_buffer(buffer: &Buffer, cursor: Cursor, fallback_height: f32) -> (f32, f32, f32) {
    let runs: Vec<_> = buffer
        .layout_runs()
        .filter(|run| run.line_i == cursor.line)
        .collect();
    let run = runs
        .iter()
        .find(|run| run.glyphs.iter().any(|g| g.start == cursor.index))
        .or_else(|| runs.iter().find(|run| run.cursor_glyph(&cursor).is_some()))
        .or(runs.last());
    match run {
        Some(run) => (
            run.cursor_position(&cursor).unwrap_or(0.0),
            run.line_top,
            run.line_height,
        ),
        None => (0.0, 0.0, fallback_height),
    }
}

const NEWLINE_SELECTION_EM: f32 = 0.25;

fn selection_rects(
    buffer: &Buffer,
    start: Cursor,
    end: Cursor,
    newline_w: f32,
) -> Vec<(f32, f32, f32, f32)> {
    let runs: Vec<_> = buffer.layout_runs().collect();
    let mut out = Vec::new();
    for (i, run) in runs.iter().enumerate() {
        if run.line_i < start.line || run.line_i > end.line {
            continue;
        }
        let mut spans: Vec<(f32, f32)> = run.highlight(start, end).collect();

        let ends_source_line = runs.get(i + 1).is_none_or(|next| next.line_i != run.line_i);
        if ends_source_line && run.line_i < end.line {
            let x = if run.rtl {
                run.glyphs
                    .iter()
                    .map(|g| g.x)
                    .fold(f32::INFINITY, f32::min)
                    .min(run.line_w)
                    - newline_w
            } else {
                run.line_w
            };
            spans.push((x.max(0.0), newline_w));
        }
        out.extend(
            spans
                .into_iter()
                .map(|(x, w)| (x, run.line_top, w, run.line_height)),
        );
    }
    out
}

fn normalize_line_breaks(s: &str) -> String {
    const BREAKS: [char; 6] = ['\r', '\u{85}', '\u{2029}', '\u{1c}', '\u{1d}', '\u{1e}'];
    if !s.contains(BREAKS) {
        return s.to_string();
    }
    s.replace("\r\n", "\n").replace(BREAKS, "\n")
}

fn to_cosmic_color(color: Color) -> CosmicColor {
    CosmicColor::rgba(color.r(), color.g(), color.b(), color.a())
}

fn buffer_extent(buffer: &Buffer) -> (f32, f32) {
    buffer.layout_runs().fold((0.0f32, 0.0f32), |(w, h), run| {
        (w.max(run.line_w), h.max(run.line_top + run.line_height))
    })
}

fn cursor_to_byte(text: &str, cursor: Cursor) -> usize {
    let mut offset = 0;
    for (i, line) in text.split('\n').enumerate() {
        if i == cursor.line {
            return offset + cursor.index.min(line.len());
        }
        offset += line.len() + 1;
    }
    text.len()
}

fn byte_to_cursor(text: &str, byte: usize) -> Cursor {
    let mut offset = 0;
    for (i, line) in text.split('\n').enumerate() {
        let line_end = offset + line.len();
        if byte <= line_end {
            return Cursor::new(i, byte - offset);
        }
        offset = line_end + 1;
    }
    match text.split('\n').enumerate().last() {
        Some((i, line)) => Cursor::new(i, line.len()),
        None => Cursor::new(0, 0),
    }
}

fn prev_grapheme_boundary(s: &str, from: usize) -> Option<usize> {
    let from = from.min(s.len());
    s[..from]
        .graphemes(true)
        .next_back()
        .map(|g| from - g.len())
}

fn next_grapheme_boundary(s: &str, from: usize) -> Option<usize> {
    if from >= s.len() {
        return None;
    }
    s[from..].graphemes(true).next().map(|g| from + g.len())
}

fn snap_to_grapheme_boundary(s: &str, pos: usize) -> usize {
    let mut pos = pos.min(s.len());
    while !s.is_char_boundary(pos) {
        pos -= 1;
    }
    let mut cursor = unicode_segmentation::GraphemeCursor::new(pos, s.len(), true);
    if cursor.is_boundary(s, 0).unwrap_or(true) {
        pos
    } else {
        cursor.prev_boundary(s, 0).ok().flatten().unwrap_or(0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WordClass {
    Space,
    Word,

    Punct,

    Symbol,
}

fn classify_segment(seg: &str) -> WordClass {
    if seg.chars().all(char::is_whitespace) {
        WordClass::Space
    } else if seg.chars().any(|c| c.is_alphanumeric()) {
        WordClass::Word
    } else if seg.chars().all(|c| c.is_ascii_punctuation()) {
        WordClass::Punct
    } else {
        WordClass::Symbol
    }
}

fn word_segments(text: &str, atoms: &[Range<usize>]) -> Vec<(Range<usize>, WordClass)> {
    let mut segments: Vec<(Range<usize>, WordClass)> = Vec::new();
    let push_text = |segments: &mut Vec<(Range<usize>, WordClass)>, base: usize, chunk: &str| {
        for (offset, seg) in chunk.split_word_bound_indices() {
            let range = base + offset..base + offset + seg.len();
            let class = classify_segment(seg);
            match segments.last_mut() {
                Some((last, last_class))
                    if *last_class == class
                        && matches!(class, WordClass::Space | WordClass::Punct)
                        && last.end == range.start =>
                {
                    last.end = range.end
                }
                _ => segments.push((range, class)),
            }
        }
    };
    let mut pos = 0;
    for atom in atoms {
        if atom.start < pos {
            continue;
        }
        push_text(&mut segments, pos, &text[pos..atom.start]);
        segments.push((atom.clone(), WordClass::Symbol));
        pos = atom.end;
    }
    push_text(&mut segments, pos, &text[pos..]);
    segments
}

fn prev_word_boundary(text: &str, atoms: &[Range<usize>], pos: usize) -> usize {
    let segments = word_segments(text, atoms);
    let Some(mut i) = segments.iter().rposition(|(r, _)| r.start < pos) else {
        return 0;
    };
    while segments[i].1 == WordClass::Space {
        if i == 0 {
            return 0;
        }
        i -= 1;
    }
    segments[i].0.start
}

fn next_word_boundary(
    text: &str,
    atoms: &[Range<usize>],
    pos: usize,
    windows_style: bool,
) -> usize {
    let segments = word_segments(text, atoms);
    let Some(mut i) = segments.iter().position(|(r, _)| r.end > pos) else {
        return text.len();
    };
    if windows_style {
        if segments[i].1 != WordClass::Space {
            i += 1;
        }
        while i < segments.len() && segments[i].1 == WordClass::Space {
            i += 1;
        }
        segments.get(i).map_or(text.len(), |(r, _)| r.start)
    } else {
        while segments[i].1 == WordClass::Space {
            i += 1;
            if i == segments.len() {
                return text.len();
            }
        }
        segments[i].0.end
    }
}

fn line_range_at(text: &str, pos: usize) -> Range<usize> {
    let pos = pos.min(text.len());
    let start = text[..pos].rfind('\n').map_or(0, |i| i + 1);
    let end = text[pos..].find('\n').map_or(text.len(), |i| pos + i);
    start..end
}

impl Widget for TextInputWidget {
    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        let width = known_dimensions
            .width
            .unwrap_or(match available_space.width {
                AvailableSpace::Definite(w) => w,

                AvailableSpace::MinContent => 0.0,
                AvailableSpace::MaxContent => 10_000.0,
            });

        if known_dimensions.width.is_some()
            || matches!(available_space.width, AvailableSpace::Definite(_))
        {
            self.set_field_width(width);
        }
        self.with_layout(|layout| {
            let (natural, h) = buffer_extent(&layout.full);
            let width = if self.auto_width {
                natural + self.inset_left + self.inset_right + 2.0
            } else {
                width
            };
            Size {
                width,
                height: h.max(self.line_height.unwrap_or(self.font_size * 1.4))
                    + 2.0 * self.pad_y(),
            }
        })
    }

    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        if let Some(bg) = style.background {
            let mut paint = Paint::default();
            paint.set_color(bg);
            paint.set_anti_alias(true);
            let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
            if style.corner_radius > 0.0 {
                canvas.draw_rrect(
                    RRect::new_rect_xy(bounds, style.corner_radius, style.corner_radius),
                    &paint,
                );
            } else {
                canvas.draw_rect(bounds, &paint);
            }
        }

        if let (Some(color), width) = (style.border_color, style.border_width) {
            if width > 0.0 {
                let ringed = self.focused && !self.read_only;
                let color = if ringed {
                    self.focus_border_color
                } else {
                    color
                };
                let width = if ringed { width.max(2.0) } else { width };
                let mut paint = Paint::default();
                paint.set_color(color);
                paint.set_anti_alias(true);
                paint.set_style(PaintStyle::Stroke);
                paint.set_stroke_width(width);
                let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
                let inset = bounds.with_inset((width / 2.0, width / 2.0));
                if style.corner_radius > 0.0 {
                    let radius = (style.corner_radius - width / 2.0).max(0.0);
                    canvas.draw_rrect(RRect::new_rect_xy(inset, radius, radius), &paint);
                } else {
                    canvas.draw_rect(inset, &paint);
                }
            }
        }

        self.set_field_width(rect.width);
        self.with_layout(|layout| {
            let full = &layout.full;
            let emoji_images = &layout.emoji_images;
            let (_, full_h) = buffer_extent(full);
            let (caret_x, caret_top, caret_h) = self.caret_in(layout);
            if !self.wraps() && !self.auto_width {
                let visible = (rect.width - self.inset_left - self.inset_right).max(1.0);
                let (full_w, _) = buffer_extent(full);
                let max_scroll = (full_w + 2.0 - visible).max(0.0);
                let follow = (self.cursor, self.text.len());
                if self.scroll_follow.replace(Some(follow)) != Some(follow) {
                    let mut scroll = self.scroll_x.get();
                    if caret_x < scroll {
                        scroll = caret_x;
                    } else if caret_x + 2.0 > scroll + visible {
                        scroll = caret_x + 2.0 - visible;
                    }
                    self.scroll_x.set(scroll);
                }
                self.scroll_x
                    .set(self.scroll_x.get().clamp(0.0, max_scroll));

                let clip_left = if self.scroll_x.get() > 0.0 {
                    rect.x + self.inset_left
                } else {
                    rect.x + self.inset_left - QUOTE_BAR_WIDTH - 6.0
                };
                canvas.save();
                canvas.clip_rect(
                    Rect::from_ltrb(
                        clip_left,
                        rect.y,
                        rect.x + rect.width - self.inset_right + 2.0,
                        rect.y + rect.height,
                    ),
                    None,
                    true,
                );
            }
            if self.wraps() {
                let max_scroll = (full_h + 2.0 * self.pad_y() - rect.height).max(0.0);
                self.max_scroll.set(max_scroll);
                let follow = (self.cursor, self.text.len());
                if self.scroll_follow.replace(Some(follow)) != Some(follow) {
                    let visible = rect.height - 2.0 * self.pad_y();
                    let mut scroll = self.scroll_y.get();
                    if caret_top < scroll {
                        scroll = caret_top;
                    } else if caret_top + caret_h > scroll + visible {
                        scroll = caret_top + caret_h - visible;
                    }
                    self.scroll_y.set(scroll);
                }
                self.scroll_y
                    .set(self.scroll_y.get().clamp(0.0, max_scroll));
                canvas.save();
                canvas.clip_rect(
                    Rect::from_xywh(rect.x, rect.y, rect.width, rect.height),
                    None,
                    true,
                );
            }
            let text_y = self.text_top(rect, full_h);

            if let (Some(range), Some(map)) = (self.selection_range(), &layout.map) {
                let mut sel_paint = Paint::default();
                sel_paint.set_color(SELECTION_COLOR);
                sel_paint.set_anti_alias(true);
                for (x, top, w, h) in selection_rects(
                    full,
                    map.cursor(range.start, false),
                    map.cursor(range.end, true),
                    self.font_size * NEWLINE_SELECTION_EM,
                ) {
                    canvas.draw_rect(
                        Rect::from_xywh(self.text_left(rect) + x, text_y + top, w, h),
                        &sel_paint,
                    );
                }
            }

            let mut font_system = self.font_system.borrow_mut();
            let origin = (self.text_left(rect), text_y);
            let quoted = if self.text.is_empty() || !self.markdown.quotes {
                Vec::new()
            } else {
                markdown::quoted_lines_excluding(&self.text, &self.code_contents(&self.text))
            };

            let scale_factor = app::current_scale_factor();
            let origin_px = (origin.0 * scale_factor, origin.1 * scale_factor);
            canvas.save();
            canvas.scale((1.0 / scale_factor, 1.0 / scale_factor));

            for run in full.layout_runs() {
                let glyphs = run.glyphs;
                let mut i = 0;
                while i < glyphs.len() {
                    let meta = glyphs[i].metadata;
                    if meta == META_NONE {
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
                    if let Some(color) = self.background_color_for_metadata(meta) {
                        let mut paint = Paint::default();
                        paint.set_color(color);
                        paint.set_anti_alias(true);
                        let decoration = meta
                            .checked_sub(META_DECORATION_BASE)
                            .filter(|_| meta < META_EMOJI_BASE)
                            .and_then(|i| self.decoration_styles.get(i));
                        let (pad_left, pad_right) = match decoration {
                            Some(d) => (d.pad_left, d.pad_right),
                            None => (
                                Self::background_padding_for_metadata(meta),
                                Self::background_padding_for_metadata(meta),
                            ),
                        };
                        let bg_rect = Rect::from_xywh(
                            origin_px.0 + (x_min - pad_left) * scale_factor,
                            origin_px.1 + run.line_top * scale_factor,
                            (x_max - x_min + pad_left + pad_right) * scale_factor,
                            run.line_height * scale_factor,
                        );
                        match decoration {
                            Some(d) => {
                                let radii: [skia_safe::Point; 4] = d.radii.map(|r| {
                                    skia_safe::Point::new(r * scale_factor, r * scale_factor)
                                });
                                canvas.draw_rrect(RRect::new_rect_radii(bg_rect, &radii), &paint);
                            }
                            None => {
                                let radius =
                                    Self::background_radius_for_metadata(meta) * scale_factor;
                                canvas.draw_rrect(
                                    RRect::new_rect_xy(bg_rect, radius, radius),
                                    &paint,
                                );
                            }
                        }
                    }
                    i = j;
                }

                for glyph in run.glyphs {
                    if glyph.metadata >= META_EMOJI_BASE {
                        if let Some((image, src)) =
                            emoji_images.get(glyph.metadata - META_EMOJI_BASE)
                        {
                            let target = self.font_size * EMOJI_SCALE * scale_factor;
                            let box_x = origin_px.0 + glyph.x * scale_factor;
                            let box_y = origin_px.1 + run.line_top * scale_factor;
                            let box_w = glyph.w * scale_factor;
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
                        }
                        continue;
                    }
                    let physical_glyph =
                        glyph.physical((0., run.line_y * scale_factor), scale_factor);
                    let glyph_color = glyph
                        .color_opt
                        .map_or(to_cosmic_color(colors::live(self.color)), |c| c);
                    paint_glyph(
                        canvas,
                        &self.glyph_cache,
                        &mut font_system,
                        physical_glyph,
                        glyph_color,
                        origin_px,
                    );
                }
                let mut decoration_renderer = SkiaDecorationRenderer {
                    canvas,
                    origin: origin_px,
                    scale: scale_factor,
                };
                cosmic_text::render_decoration(
                    &mut decoration_renderer,
                    &run,
                    to_cosmic_color(colors::live(self.color)),
                );

                if quoted.get(run.line_i).copied().unwrap_or(false) {
                    let mut bar_paint = Paint::default();
                    bar_paint.set_color(QUOTE_BAR_COLOR);
                    bar_paint.set_anti_alias(true);
                    let bar_width = QUOTE_BAR_WIDTH * scale_factor;
                    let bar_rect = Rect::from_xywh(
                        origin_px.0 - bar_width - 4.0 * scale_factor,
                        origin_px.1 + run.line_top * scale_factor,
                        bar_width,
                        run.line_height * scale_factor,
                    );
                    canvas.draw_rrect(
                        RRect::new_rect_xy(bar_rect, bar_width, bar_width),
                        &bar_paint,
                    );
                }
            }
            canvas.restore();

            if self.focused {
                let phase = self.focused_since.elapsed().as_millis() / BLINK_INTERVAL.as_millis();
                let caret_x_px = self.text_left(rect) + caret_x;
                let caret_rect = Rect::from_xywh(caret_x_px, text_y + caret_top, 1.5, caret_h);
                if phase.is_multiple_of(2) {
                    let mut paint = Paint::default();
                    paint.set_color(colors::live(self.color));
                    canvas.draw_rect(caret_rect, &paint);
                }

                let next_flip = self.focused_since + BLINK_INTERVAL * (phase as u32 + 1);
                crate_paint::animate_region_at(canvas, caret_rect, next_flip);
            }

            canvas.restore();
        });

        if self.clearable && !self.text.is_empty() && !self.disabled && !self.read_only {
            if let Some(art) = IconPath::art(&icons::X_SMALL) {
                let size = 16.0;
                let x = rect.x + rect.width - CLEAR_ZONE + (CLEAR_ZONE - 4.0 - size) / 2.0;
                let y = rect.y + (rect.height - size) / 2.0;
                art.draw_fitted(canvas, (x, y, size, size), TEXT_SECONDARY, 1.0);
            }
        }

        if let Some(popup) = &self.popup {
            self.paint_popup(canvas, rect, popup);
        }
    }

    fn relayout_after(&self, _event: &UiEvent) -> bool {
        self.size_changed.get()
    }

    fn handle_event(&mut self, event: &UiEvent, rect: AbsoluteRect) -> EventResult {
        let content_before = self.content_key();

        self.skip_history = false;
        let before = matches!(
            event,
            UiEvent::KeyDown { .. }
                | UiEvent::TextInput { .. }
                | UiEvent::MouseDown { .. }
                | UiEvent::ImeSyncState { .. }
        )
        .then(|| HistorySnapshot {
            text: self.text.clone(),
            cursor: self.cursor,
            anchor: self.selection_anchor,
        });
        let result = (|| -> EventResult {
            if self.disabled {
                return EventResult::Ignored;
            }

            if let UiEvent::Scroll { delta_y, .. } = event {
                let max = self.max_scroll.get();
                let current = self.scroll_y.get();
                let next = (current - delta_y).clamp(0.0, max);
                if !self.wraps() || next == current {
                    return EventResult::Ignored;
                }
                self.scroll_y.set(next);
                crate_paint::request_repaint();
                return EventResult::Handled;
            }

            match event {
                UiEvent::MouseMoved { x, y } => {
                    if let Some(hit) = self.popup_hit(rect, *x, *y) {
                        if let (PopupHit::Row(row), Some(popup)) = (hit, self.popup.as_mut()) {
                            popup.selected = row;
                        }
                        return EventResult::Handled;
                    }
                }
                UiEvent::MouseDown {
                    x,
                    y,
                    button: MouseButton::Left,
                    ..
                } => {
                    if let Some(hit) = self.popup_hit(rect, *x, *y) {
                        if let PopupHit::Row(row) = hit {
                            if let Some(popup) = self.popup.as_mut() {
                                popup.selected = row;
                            }
                            self.complete_popup_selection();
                        }
                        return EventResult::Handled;
                    }
                }
                _ => {}
            }
            if let UiEvent::MouseDown {
                x,
                button: MouseButton::Left,
                ..
            } = event
            {
                if self.clearable
                    && !self.read_only
                    && !self.text.is_empty()
                    && *x >= rect.x + rect.width - CLEAR_ZONE
                {
                    self.text.clear();
                    self.cursor = 0;
                    self.selection_anchor = None;
                    self.notify_change();
                    self.refresh_popup();
                    return EventResult::Handled;
                }
            }
            if !matches!(
                event,
                UiEvent::KeyDown {
                    key: NamedKey::ArrowUp | NamedKey::ArrowDown,
                    ..
                } | UiEvent::MouseMoved { .. }
            ) {
                self.goal_x = None;
            }
            match event {
                UiEvent::MouseDown {
                    x,
                    y,
                    button: MouseButton::Left,
                    shift,
                } => {
                    if self.text.is_empty() {
                        self.clear_selection();
                        self.cursor = 0;
                        self.last_click = None;
                        return EventResult::Handled;
                    }

                    let pos = self.cursor_index_for_point(rect, *x, *y);
                    let repeat = self
                        .last_click
                        .filter(|(when, at, _)| {
                            *at == pos && when.elapsed() < DOUBLE_CLICK_INTERVAL
                        })
                        .map(|(_, _, count)| count);

                    match repeat {
                        Some(1) if !*shift => {
                            self.select_word_at(pos);
                            self.last_click = Some((Instant::now(), pos, 2));
                        }
                        Some(2) if !*shift => {
                            self.select_line_at(pos);
                            self.last_click = None;
                        }
                        _ => {
                            self.begin_click(pos, *shift);
                            self.last_click = Some((Instant::now(), pos, 1));
                        }
                    }
                    EventResult::Handled
                }
                UiEvent::MouseDown { .. } => {
                    self.clear_selection();
                    EventResult::Handled
                }
                UiEvent::MouseMoved { x, y } => {
                    if !self.dragging {
                        return EventResult::Ignored;
                    }

                    let pos = self.cursor_index_for_point(rect, *x, *y);
                    if pos == self.cursor {
                        return EventResult::Ignored;
                    }
                    self.cursor = pos;
                    if self.selection_range().is_some() {
                        self.popup = None;
                    } else {
                        self.refresh_popup();
                    }
                    EventResult::Handled
                }
                UiEvent::MouseUp {
                    button: MouseButton::Left,
                    ..
                } => {
                    self.dragging = false;
                    EventResult::Handled
                }
                UiEvent::TextInput { text } => {
                    self.insert(text);
                    EventResult::Handled
                }
                UiEvent::ImePreedit { text } => {
                    self.ime_preedit = text.clone();
                    EventResult::Handled
                }

                UiEvent::ImeSyncState { text, cursor } => {
                    if &self.text == text {
                        return EventResult::Ignored;
                    }
                    self.ime_preedit.clear();

                    self.skip_history = true;
                    self.text = text.clone();

                    let mut clamped = (*cursor).min(self.text.len());
                    while clamped > 0 && !self.text.is_char_boundary(clamped) {
                        clamped -= 1;
                    }
                    self.cursor = clamped;
                    self.selection_anchor = None;
                    self.notify_change();
                    self.refresh_popup();
                    EventResult::Handled
                }
                UiEvent::KeyDown { key, shift } => {
                    if let Some(hook) = &mut self.key_hook {
                        if hook(*key, *shift, &self.text) {
                            return EventResult::Handled;
                        }
                    }

                    if self.popup.is_some() {
                        match key {
                            NamedKey::ArrowUp => {
                                self.move_popup_selection(-1);
                                return EventResult::Handled;
                            }
                            NamedKey::ArrowDown => {
                                self.move_popup_selection(1);
                                return EventResult::Handled;
                            }
                            NamedKey::Enter | NamedKey::Tab if !*shift => {
                                self.complete_popup_selection();
                                return EventResult::Handled;
                            }
                            NamedKey::Escape => {
                                self.popup = None;
                                return EventResult::Handled;
                            }
                            _ => {}
                        }
                    }

                    match key {
                        NamedKey::Space => {
                            self.insert(" ");
                            EventResult::Handled
                        }
                        NamedKey::Backspace => {
                            if self.text.is_empty() {
                                if let Some(hook) = &mut self.edge_hook {
                                    if hook(*key, *shift, &self.text) {
                                        return EventResult::Handled;
                                    }
                                }
                            }
                            self.backspace();
                            EventResult::Handled
                        }
                        NamedKey::Delete => {
                            self.delete_forward();
                            EventResult::Handled
                        }

                        NamedKey::ArrowLeft => {
                            if self.cursor == 0 && self.selection_range().is_none() {
                                if let Some(hook) = &mut self.edge_hook {
                                    if hook(*key, *shift, &self.text) {
                                        return EventResult::Handled;
                                    }
                                }
                            }
                            let target = match self.selection_range() {
                                Some(sel) if !*shift => sel.start,
                                _ => self.prev_cursor_position(self.cursor),
                            };
                            self.move_caret(target, *shift);
                            EventResult::Handled
                        }
                        NamedKey::ArrowRight => {
                            if self.cursor == self.text.len() && self.selection_range().is_none() {
                                if let Some(hook) = &mut self.edge_hook {
                                    if hook(*key, *shift, &self.text) {
                                        return EventResult::Handled;
                                    }
                                }
                            }
                            let target = match self.selection_range() {
                                Some(sel) if !*shift => sel.end,
                                _ => self.next_cursor_position(self.cursor),
                            };
                            self.move_caret(target, *shift);
                            EventResult::Handled
                        }
                        NamedKey::WordLeft => {
                            let target = self.prev_word_position(self.cursor);
                            self.move_caret(target, *shift);
                            EventResult::Handled
                        }
                        NamedKey::WordRight => {
                            let target = self.next_word_position(self.cursor);
                            self.move_caret(target, *shift);
                            EventResult::Handled
                        }
                        NamedKey::DeleteWordBack => {
                            self.delete_word(false);
                            EventResult::Handled
                        }
                        NamedKey::DeleteWordForward => {
                            self.delete_word(true);
                            EventResult::Handled
                        }

                        NamedKey::ArrowUp | NamedKey::ArrowDown => {
                            if !self.wraps() {
                                return EventResult::Ignored;
                            }
                            self.move_vertically(*key == NamedKey::ArrowDown, *shift);
                            EventResult::Handled
                        }
                        NamedKey::Home => {
                            self.move_to_line_edge(false, *shift);
                            EventResult::Handled
                        }
                        NamedKey::End => {
                            self.move_to_line_edge(true, *shift);
                            EventResult::Handled
                        }
                        NamedKey::DocStart => {
                            self.move_caret(0, *shift);
                            EventResult::Handled
                        }
                        NamedKey::DocEnd => {
                            self.move_caret(self.text.len(), *shift);
                            EventResult::Handled
                        }
                        NamedKey::Undo => {
                            self.undo();
                            EventResult::Handled
                        }
                        NamedKey::Redo => {
                            self.redo();
                            EventResult::Handled
                        }
                        NamedKey::SelectAll => {
                            self.select_all();
                            EventResult::Handled
                        }
                        NamedKey::Copy => {
                            self.copy();
                            EventResult::Handled
                        }
                        NamedKey::Cut => {
                            self.cut();
                            EventResult::Handled
                        }
                        NamedKey::Paste => {
                            self.paste();
                            EventResult::Handled
                        }
                        NamedKey::Enter => {
                            let in_open_block = self.wrap
                                && self.markdown.code
                                && markdown::has_open_code_block_before(&self.text, self.cursor);
                            if self.multiline || (*shift && self.wrap) || in_open_block {
                                self.insert("\n");
                            } else if *shift {
                            } else {
                                self.submit();
                            }
                            EventResult::Handled
                        }
                        NamedKey::Escape => {
                            let consumed = match &mut self.edge_hook {
                                Some(hook) => hook(*key, *shift, &self.text),
                                None => false,
                            };
                            if consumed {
                                EventResult::Handled
                            } else {
                                EventResult::Ignored
                            }
                        }

                        NamedKey::Tab => {
                            if let Some(hook) = &mut self.edge_hook {
                                if hook(*key, *shift, &self.text) {
                                    return EventResult::Handled;
                                }
                            }
                            if self.indent_code_lines(*shift) {
                                EventResult::Handled
                            } else {
                                EventResult::Ignored
                            }
                        }
                    }
                }
                _ => EventResult::Ignored,
            }
        })();
        if let Some(before) = before {
            self.record_history(event, before);
        }

        if matches!(
            event,
            UiEvent::KeyDown { .. }
                | UiEvent::TextInput { .. }
                | UiEvent::MouseDown { .. }
                | UiEvent::ImeSyncState { .. }
        ) && !matches!(result, EventResult::Ignored)
        {
            self.focused_since = Instant::now();
        }
        self.size_changed.set(self.content_key() != content_before);
        result
    }

    fn extra_hit_rect(&self, rect: AbsoluteRect) -> Option<AbsoluteRect> {
        let popup = self.popup.as_ref()?;
        let panel = self.popup_geometry(rect, popup).panel;
        Some(AbsoluteRect {
            x: panel.left,
            y: panel.top,
            width: panel.width(),
            height: panel.height(),
        })
    }

    fn wants_focus(&self) -> bool {
        !self.disabled
    }

    fn wants_text_input(&self) -> bool {
        !self.disabled && !self.read_only
    }

    fn wants_pointer_capture(&self) -> bool {
        self.dragging
    }

    fn on_focus_change(&mut self, focused: bool) {
        self.focused = focused;
        self.focused_since = Instant::now();
        if self.triggers.iter().any(|t| t.whole_text) {
            self.refresh_popup();
        }
        if let Some(callback) = &mut self.on_focus_change {
            callback(focused);
        }
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        Some(CursorIcon::Text)
    }

    fn cursor_icon_at(&self, rect: AbsoluteRect, x: f32, y: f32) -> Option<CursorIcon> {
        if matches!(self.popup_hit(rect, x, y), Some(PopupHit::Row(_))) {
            return Some(CursorIcon::Pointer);
        }
        self.cursor_icon()
    }
}

#[cfg(test)]
mod mention_tests {
    use crate::font_system;
    use super::*;

    fn test_fonts() -> (Rc<RefCell<FontSystem>>, Rc<GlyphCache>) {
        (
            Rc::new(RefCell::new(font_system::make_font_system())),
            Rc::new(GlyphCache::new(1024 * 1024)),
        )
    }

    fn widget_with_mention_trigger() -> TextInputWidget {
        let trigger = AutocompleteTrigger {
            trigger_char: '@',
            candidates: vec![AutocompleteCandidate {
                display: "example".to_string(),
                id: Some(42),
                ..Default::default()
            }],
            only_at_message_start: false,
            chip_background: Some(Color::from_argb(255, 88, 101, 242)),
            chip_text_color: Color::WHITE,
            popup_label: "Members".to_string(),
            multi_word: false,
            whole_text: false,
        };
        let (font_system, glyph_cache) = test_fonts();
        TextInputWidget::new(
            "",
            "",
            14.0,
            Color::WHITE,
            Color::WHITE,
            font_system,
            glyph_cache,
            false,
        )
        .with_autocomplete(vec![trigger])
    }

    #[test]
    fn scan_mention_tokens_does_not_panic_on_truncated_token() {
        let widget = widget_with_mention_trigger();
        for text in ["<", "<@", "<@4", "<@42", "<@42>", "x<@", "hey <@ more"] {
            let _ = widget.scan_mention_tokens(text);
        }
    }

    #[test]
    fn complete_popup_selection_does_not_duplicate_trigger_char() {
        let mut widget = widget_with_mention_trigger();
        widget.insert("hey @exa");
        assert!(
            widget.popup.is_some(),
            "popup should be open after typing a matching query"
        );
        widget.complete_popup_selection();

        assert_eq!(widget.text, "hey <@42> ");
        assert_eq!(widget.text.matches('@').count(), 1);
    }

    #[test]
    fn completed_token_resolves_to_a_single_at_sign_chip() {
        let mut widget = widget_with_mention_trigger();
        widget.insert("hey @exa");
        widget.complete_popup_selection();
        let chips = widget.find_chip_ranges(&widget.text);
        assert_eq!(chips.len(), 1);
        assert_eq!(chips[0].display, "@example");
    }

    #[test]
    fn mention_tokens_inside_fenced_code_stay_literal() {
        let widget = widget_with_mention_trigger().with_markdown(MarkdownFeatures::ALL);
        let text = "```\n<@42>\n```";
        assert!(widget.find_chip_ranges(text).is_empty());
        let runs = widget.tokenize_runs(text);
        assert!(
            runs.iter()
                .any(|run| matches!(&run.kind, RunKind::Code(None))
                    && text[run.range.clone()].contains("<@42>"))
        );
    }

    #[test]
    fn code_block_runs_are_fences_language_and_highlighted_code() {
        let highlighter: CodeHighlighter = Rc::new(|lang: &str, code: &str| {
            assert_eq!(lang, "rust");
            code.find("fn")
                .map(|i| vec![(i..i + 2, Color::RED)])
                .unwrap_or_default()
        });
        let widget = widget_with_mention_trigger()
            .with_markdown(MarkdownFeatures::ALL)
            .with_code_highlighter(highlighter);
        let text = "```rust
fn main() **x**
```";
        let runs: Vec<(&str, String)> = widget
            .tokenize_runs(text)
            .iter()
            .map(|run| {
                let kind = match &run.kind {
                    RunKind::Fence => "fence".to_string(),
                    RunKind::Code(Some(_)) => "hl".to_string(),
                    RunKind::Code(None) => "code".to_string(),
                    RunKind::Markdown(attrs) => {
                        format!("md{}", if attrs.delimiter { "-delim" } else { "" })
                    }
                    _ => "other".to_string(),
                };
                (&text[run.range.clone()], kind)
            })
            .collect();
        assert_eq!(
            runs,
            vec![
                ("```", "fence".to_string()),
                (
                    "rust
",
                    "md".to_string()
                ),
                ("fn", "hl".to_string()),
                (
                    " main() **x**
",
                    "code".to_string()
                ),
                ("```", "fence".to_string()),
            ]
        );
    }

    #[test]
    fn enter_in_open_code_block_inserts_newline() {
        let mut widget = widget_with_mention_trigger()
            .with_wrap()
            .with_markdown(MarkdownFeatures::ALL);
        widget.text = "```js".to_string();
        widget.cursor = widget.text.len();
        let rect = AbsoluteRect {
            x: 0.0,
            y: 0.0,
            width: 300.0,
            height: 40.0,
        };
        widget.handle_event(
            &UiEvent::KeyDown {
                key: NamedKey::Enter,
                shift: false,
            },
            rect,
        );
        assert_eq!(
            widget.text,
            "```js
"
        );
    }

    #[test]
    fn tab_indents_and_dedents_code_lines_only() {
        let mut widget = widget_with_mention_trigger().with_markdown(MarkdownFeatures::ALL);
        widget.text = "```
a
b
```"
        .to_string();
        widget.cursor = 4;
        assert!(widget.indent_code_lines(false));
        assert_eq!(
            widget.text,
            "```
  a
b
```"
        );
        assert_eq!(widget.cursor, 6);

        widget.selection_anchor = Some(4);
        widget.cursor = 9;
        assert!(widget.indent_code_lines(false));
        assert_eq!(
            widget.text,
            "```
    a
  b
```"
        );
        assert!(widget.indent_code_lines(true));
        assert!(widget.indent_code_lines(true));
        assert_eq!(
            widget.text,
            "```
a
b
```"
        );

        widget.text = "plain".to_string();
        widget.selection_anchor = None;
        widget.cursor = 2;
        assert!(!widget.indent_code_lines(false));
    }

    #[test]
    fn autocomplete_is_closed_inside_fenced_code() {
        let mut widget = widget_with_mention_trigger().with_markdown(MarkdownFeatures::ALL);
        widget.text = "```\n@exa".to_string();
        widget.cursor = widget.text.len();
        assert!(widget.detect_trigger().is_none());

        widget.text = "```\n@exa\n```\n@exa".to_string();
        widget.cursor = widget.text.len();
        assert!(
            widget.detect_trigger().is_some(),
            "autocomplete should resume after the closing fence"
        );
    }

    #[test]
    fn layout_map_accounts_for_chip_display_length() {
        let mut widget = widget_with_mention_trigger();
        widget.text = "hey <@42> bye".to_string();
        let (_, map, _) = widget.build_rich_buffer(&widget.text);
        assert_eq!(map.text, "hey @example bye");

        assert_eq!(map.to_layout(4, false)..map.to_layout(9, true), 4..12);

        assert_eq!(map.to_layout(6, false), 4);
        assert_eq!(map.to_layout(6, true), 12);

        assert_eq!(map.to_layout(widget.text.len(), true), map.text.len());

        assert_eq!(map.to_source(5), 4);
        assert_eq!(map.to_source(11), 9);
        assert_eq!(map.to_source(map.text.len()), widget.text.len());
    }

    #[test]
    fn layout_map_skips_hidden_quote_markers() {
        let widget = widget_with_mention_trigger().with_markdown(MarkdownFeatures::ALL);
        let text = "a
> quoted
b";
        let (_, map, _) = widget.build_rich_buffer(text);
        assert_eq!(
            map.text,
            "a
quoted
b"
        );

        assert_eq!(map.to_layout(2, false), 2);
        assert_eq!(map.to_layout(4, false), 2);

        assert_eq!(map.to_source(2), 4);
        assert_eq!(map.to_source(map.text.len()), text.len());
    }

    #[test]
    fn arrow_keys_skip_over_a_mention_chip_atomically() {
        let mut widget = widget_with_mention_trigger();
        widget.text = "hey <@42> bye".to_string();
        widget.cursor = 4;

        widget.handle_event(
            &UiEvent::KeyDown {
                key: NamedKey::ArrowRight,
                shift: false,
            },
            AbsoluteRect::default(),
        );
        assert_eq!(
            widget.cursor, 9,
            "ArrowRight should jump clean over the whole chip in one step"
        );

        widget.handle_event(
            &UiEvent::KeyDown {
                key: NamedKey::ArrowLeft,
                shift: false,
            },
            AbsoluteRect::default(),
        );
        assert_eq!(
            widget.cursor, 4,
            "ArrowLeft should jump clean back over the whole chip in one step"
        );

        widget.handle_event(
            &UiEvent::KeyDown {
                key: NamedKey::ArrowRight,
                shift: true,
            },
            AbsoluteRect::default(),
        );
        assert_eq!(widget.selection_anchor, Some(4));
        assert_eq!(widget.cursor, 9);

        widget.cursor = 9;
        widget.clear_selection();
        widget.backspace();
        assert_eq!(widget.text, "hey  bye");
        assert_eq!(widget.cursor, 4);
    }
}

#[cfg(test)]
mod selection_tests {
    use crate::control;
    use crate::font_system;
    use crate::node::Node;
    use crate::paint as crate_paint;
    use crate::signal::flush;
    use crate::signal::signal;
    use crate::style::ComputedStyle;
    use crate::widget::WidgetTree;
    use crate::widget::primitives::BoxWidget;
    use std::any;
    use std::slice;
    use super::*;

    fn plain_widget() -> TextInputWidget {
        let font_system = Rc::new(RefCell::new(font_system::make_font_system()));
        let glyph_cache = Rc::new(GlyphCache::new(1024 * 1024));
        TextInputWidget::new(
            "",
            "",
            14.0,
            Color::WHITE,
            Color::WHITE,
            font_system,
            glyph_cache,
            true,
        )
    }

    #[test]
    fn caret_advances_past_trailing_spaces_and_decorated_text() {
        let mut widget = plain_widget();
        let caret_x = |w: &mut TextInputWidget, text: &str| {
            w.text = text.to_string();
            w.cursor = text.len();
            w.caret_geometry().0
        };
        let word = caret_x(&mut widget, "hello");
        let spaced = caret_x(&mut widget, "hello ");
        let spaced_twice = caret_x(&mut widget, "hello  ");
        assert!(
            spaced > word + 1.0,
            "caret should move past a trailing space: {word} vs {spaced}"
        );
        assert!(
            spaced_twice > spaced + 1.0,
            "and past a second one: {spaced} vs {spaced_twice}"
        );

        let style = |weight| DecorationStyle {
            weight,
            color: Color::WHITE,
            background: None,
            radii: [0.0; 4],
            pad_left: 0.0,
            pad_right: 0.0,
        };
        let mut decorated = plain_widget().with_decorator(
            vec![style(600), style(500)],
            Rc::new(|text: &str| {
                text.find("from:")
                    .map(|i| TextDecoration {
                        range: i..i + 5,
                        style: 0,
                    })
                    .into_iter()
                    .collect()
            }),
        );
        let before = caret_x(&mut decorated, "from:");
        let after = caret_x(&mut decorated, "from: testing ");
        assert!(
            before > 10.0 && after > before + 20.0,
            "decorated caret: {before} then {after}"
        );
    }

    #[test]
    fn typing_into_a_field_whose_section_rebuilds_keeps_focus_and_caret() {
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let root = tree.root;
        let text = signal(String::new());
        let (read, write) = (text.clone(), text.clone());
        let _section = control::section(&mut tree, root, move || {
            let value = read.get();
            let write = write.clone();
            let mut widget = plain_widget().with_on_change(move |t| write.set(t.to_string()));
            widget.text = value.clone();
            widget.cursor = value.len();
            Node::new("div", widget)
        });
        let input = tree.nodes[root].children[0];
        tree.focused = Some(input);

        let mut ids = vec![input];
        for c in "hello".chars() {
            crate_paint::dispatch_key_event(
                &mut tree,
                &UiEvent::TextInput {
                    text: c.to_string(),
                },
            );
            flush(&mut tree);
            ids.push(tree.nodes[root].children[0]);
        }
        assert_ne!(ids[0], ids[1], "the section really did rebuild");
        assert_eq!(
            tree.focused,
            Some(*ids.last().unwrap()),
            "focus followed the rebuild"
        );
        assert_eq!(text.peek(), "hello", "every character landed");

        let id = tree.nodes[root].children[0];
        (&mut *tree.nodes[id].widget as &mut dyn any::Any)
            .downcast_mut::<TextInputWidget>()
            .unwrap()
            .cursor = 2;
        crate_paint::dispatch_key_event(
            &mut tree,
            &UiEvent::TextInput {
                text: "X".to_string(),
            },
        );
        flush(&mut tree);
        let id = tree.nodes[root].children[0];
        let widget = (&*tree.nodes[id].widget as &dyn any::Any)
            .downcast_ref::<TextInputWidget>()
            .unwrap();
        assert_eq!((widget.text.as_str(), widget.cursor), ("heXllo", 3));
    }

    #[test]
    fn shift_arrow_extends_then_plain_arrow_collapses_selection() {
        let mut widget = plain_widget();
        widget.insert("hello");
        widget.cursor = 5;

        widget.handle_event(
            &UiEvent::KeyDown {
                key: NamedKey::ArrowLeft,
                shift: true,
            },
            AbsoluteRect::default(),
        );
        widget.handle_event(
            &UiEvent::KeyDown {
                key: NamedKey::ArrowLeft,
                shift: true,
            },
            AbsoluteRect::default(),
        );
        assert_eq!(widget.selection_range(), Some(3..5));

        widget.handle_event(
            &UiEvent::KeyDown {
                key: NamedKey::ArrowLeft,
                shift: false,
            },
            AbsoluteRect::default(),
        );
        assert_eq!(widget.selection_range(), None);
    }

    #[test]
    fn shift_enter_adds_a_line_only_in_a_wrapping_field() {
        let mut single = plain_widget();
        single.insert("hi");
        single.handle_event(
            &UiEvent::KeyDown {
                key: NamedKey::Enter,
                shift: true,
            },
            AbsoluteRect::default(),
        );
        assert_eq!(single.text, "hi", "a single-line field ignores Shift+Enter");

        let mut composer = plain_widget().with_wrap();
        composer.insert("hi");
        composer.handle_event(
            &UiEvent::KeyDown {
                key: NamedKey::Enter,
                shift: true,
            },
            AbsoluteRect::default(),
        );
        assert_eq!(
            composer.text,
            "hi
"
        );

        let submitted = Rc::new(RefCell::new(None));
        let sink = submitted.clone();
        let mut composer = plain_widget()
            .with_wrap()
            .with_on_submit(move |text: String| *sink.borrow_mut() = Some(text));
        composer.insert("send me");
        composer.handle_event(
            &UiEvent::KeyDown {
                key: NamedKey::Enter,
                shift: false,
            },
            AbsoluteRect::default(),
        );
        assert_eq!(
            submitted.borrow().as_deref(),
            Some("send me"),
            "plain Enter still submits in a wrapping field"
        );
    }

    #[test]
    fn wrapping_field_grows_with_wrapped_lines() {
        let mut widget = plain_widget().with_wrap();
        let one_line = widget
            .measure(
                Size {
                    width: Some(200.0),
                    height: None,
                },
                Size {
                    width: AvailableSpace::Definite(200.0),
                    height: AvailableSpace::MaxContent,
                },
            )
            .height;
        widget.insert("a sentence long enough that it cannot possibly fit on one line of a two hundred pixel wide field");
        let wrapped = widget
            .measure(
                Size {
                    width: Some(200.0),
                    height: None,
                },
                Size {
                    width: AvailableSpace::Definite(200.0),
                    height: AvailableSpace::MaxContent,
                },
            )
            .height;
        assert!(
            wrapped > one_line * 2.0,
            "wrapped text should take several lines: {one_line} -> {wrapped}"
        );

        let mut single = plain_widget();
        single.insert("a sentence long enough that it cannot possibly fit on one line of a two hundred pixel wide field");
        let unwrapped = single
            .measure(
                Size {
                    width: Some(200.0),
                    height: None,
                },
                Size {
                    width: AvailableSpace::Definite(200.0),
                    height: AvailableSpace::MaxContent,
                },
            )
            .height;
        assert!(
            (unwrapped - one_line).abs() < 0.5,
            "a single-line field never wraps"
        );
    }

    #[test]
    fn select_all_spans_the_whole_buffer() {
        let mut widget = plain_widget();
        widget.insert("hello world");
        widget.handle_event(
            &UiEvent::KeyDown {
                key: NamedKey::SelectAll,
                shift: false,
            },
            AbsoluteRect::default(),
        );
        assert_eq!(widget.selection_range(), Some(0..11));
    }

    #[test]
    fn typing_over_a_selection_replaces_it() {
        let mut widget = plain_widget();
        widget.insert("hello world");
        widget.selection_anchor = Some(0);
        widget.cursor = 5;
        widget.insert("goodbye");
        assert_eq!(widget.text, "goodbye world");
        assert_eq!(widget.selection_range(), None);
    }

    #[test]
    fn backspace_deletes_selection_instead_of_one_char() {
        let mut widget = plain_widget();
        widget.insert("hello world");
        widget.selection_anchor = Some(6);
        widget.cursor = 11;
        widget.backspace();
        assert_eq!(widget.text, "hello ");
        assert_eq!(widget.cursor, 6);
        assert_eq!(widget.selection_range(), None);
    }

    #[test]
    fn ime_preedit_is_not_spliced_into_the_real_buffer() {
        let mut widget = plain_widget();
        widget.insert("hi ");
        widget.handle_event(
            &UiEvent::ImePreedit {
                text: "ni".to_string(),
            },
            AbsoluteRect::default(),
        );
        assert_eq!(
            widget.text, "hi ",
            "preedit must stay out of the committed buffer"
        );
        assert_eq!(widget.ime_preedit, "ni");

        widget.handle_event(
            &UiEvent::TextInput {
                text: "\u{4f60}\u{597d}".to_string(),
            },
            AbsoluteRect::default(),
        );
        assert_eq!(widget.text, "hi \u{4f60}\u{597d}");
        assert_eq!(widget.ime_preedit, "");
    }

    #[test]
    fn detects_rtl_for_hebrew_and_arabic_but_not_latin() {
        assert_eq!(detect_text_direction("hello"), TextDirection::LTR);
        assert_eq!(
            detect_text_direction("\u{5e9}\u{5dc}\u{5d5}\u{5dd}"),
            TextDirection::RTL
        );
        assert_eq!(
            detect_text_direction("\u{645}\u{631}\u{62d}\u{628}\u{627}"),
            TextDirection::RTL
        );
        assert_eq!(detect_text_direction("123"), TextDirection::LTR);
        assert_eq!(detect_text_direction(""), TextDirection::LTR);
    }

    #[test]
    fn word_range_at_selects_the_word_touching_the_click() {
        let text = "hello world";
        assert_eq!(word_range_at(text, 2), 0..5);
        assert_eq!(word_range_at(text, 4), 0..5);
        assert_eq!(word_range_at(text, 6), 6..11);
        assert_eq!(word_range_at(text, 11), 6..11);
    }

    #[test]
    fn word_range_at_selects_the_whole_whitespace_run_for_a_click_on_whitespace() {
        assert_eq!(word_range_at("a    b", 3), 1..5);
    }

    #[test]
    fn utf16_offset_to_byte_roundtrips_through_multibyte_chars() {
        let text = "h\u{e9}llo";
        assert_eq!(utf16_len(text), 5);
        assert_eq!(utf16_offset_to_byte(text, 0), 0);
        assert_eq!(utf16_offset_to_byte(text, 1), 1);
        assert_eq!(utf16_offset_to_byte(text, 2), 1 + 'é'.len_utf8());
        assert_eq!(utf16_offset_to_byte(text, 5), text.len());
        assert_eq!(
            utf16_offset_to_byte(text, 999),
            text.len(),
            "past-the-end clamps to text.len()"
        );
    }

    #[test]
    fn double_click_same_spot_selects_a_word_instead_of_just_moving_caret() {
        let mut widget = plain_widget();
        widget.insert("hello world");
        let rect = AbsoluteRect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 30.0,
        };
        let click = |w: &mut TextInputWidget| {
            w.handle_event(
                &UiEvent::MouseDown {
                    x: 10.0,
                    y: 15.0,
                    button: MouseButton::Left,
                    shift: false,
                },
                rect,
            )
        };

        click(&mut widget);
        assert_eq!(
            widget.selection_range(),
            None,
            "a first, lone click should just position the caret"
        );

        click(&mut widget);
        assert!(
            widget.selection_range().is_some(),
            "a second click at the same resolved position within the double-click window should select a word"
        );
    }

    #[test]
    fn mouse_down_starts_a_drag_and_mouse_up_ends_it() {
        let mut widget = plain_widget();
        widget.insert("hello world");
        let rect = AbsoluteRect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 30.0,
        };

        widget.handle_event(
            &UiEvent::MouseDown {
                x: 5.0,
                y: 15.0,
                button: MouseButton::Left,
                shift: false,
            },
            rect,
        );
        assert!(
            widget.wants_pointer_capture(),
            "a click should start a drag gesture"
        );

        widget.handle_event(
            &UiEvent::MouseUp {
                x: 5.0,
                y: 15.0,
                button: MouseButton::Left,
            },
            rect,
        );
        assert!(
            !widget.wants_pointer_capture(),
            "releasing should end the drag gesture"
        );
    }

    #[test]
    fn mouse_moved_is_ignored_when_not_dragging() {
        let mut widget = plain_widget();
        widget.insert("hello world");
        widget.cursor = 0;
        let rect = AbsoluteRect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 30.0,
        };

        let result = widget.handle_event(&UiEvent::MouseMoved { x: 100.0, y: 15.0 }, rect);
        assert_eq!(result, EventResult::Ignored);
        assert_eq!(
            widget.cursor, 0,
            "no drag in progress, so the caret shouldn't move"
        );
    }

    #[test]
    fn shift_click_extends_from_the_existing_cursor() {
        let mut widget = plain_widget();
        widget.insert("hello world");
        widget.cursor = 5;
        widget.begin_click(11, true);
        assert_eq!(widget.selection_range(), Some(5..11));
    }

    #[test]
    fn word_boundaries_skip_whitespace_and_punctuation_runs() {
        let text = "hello, big  world";
        assert_eq!(prev_word_boundary(text, &[], text.len()), 12);
        assert_eq!(prev_word_boundary(text, &[], 12), 7);
        assert_eq!(
            prev_word_boundary(text, &[], 7),
            5,
            "punctuation is its own stop"
        );
        assert_eq!(prev_word_boundary(text, &[], 5), 0);
        assert_eq!(prev_word_boundary(text, &[], 0), 0);

        assert_eq!(next_word_boundary(text, &[], 0, false), 5);
        assert_eq!(next_word_boundary(text, &[], 5, false), 6);
        assert_eq!(next_word_boundary(text, &[], 6, false), 10);
        assert_eq!(next_word_boundary(text, &[], 17, false), 17);

        assert_eq!(next_word_boundary(text, &[], 0, true), 5);
        assert_eq!(next_word_boundary(text, &[], 7, true), 12);
    }

    #[test]
    fn emoji_are_single_grapheme_and_single_word_stops() {
        let text = "hi \u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467} \u{1F44D}\u{1F3FD}\u{1F44D}";
        let family = 3..3 + "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}".len();
        assert_eq!(next_grapheme_boundary(text, family.start), Some(family.end));
        assert_eq!(prev_grapheme_boundary(text, family.end), Some(family.start));
        assert_eq!(
            snap_to_grapheme_boundary(text, family.start + 4),
            family.start
        );

        let thumbs = family.end + 1;
        let second = thumbs + "\u{1F44D}\u{1F3FD}".len();
        assert_eq!(prev_word_boundary(text, &[], text.len()), second);
        assert_eq!(prev_word_boundary(text, &[], second), thumbs);
    }

    #[test]
    fn atoms_are_one_word_stop() {
        let text = "a <@123456789> b";
        let atom = 2..14;
        assert_eq!(prev_word_boundary(text, slice::from_ref(&atom), 14), 2);
        assert_eq!(next_word_boundary(text, &[atom], 2, false), 14);
    }

    #[test]
    fn line_range_at_stops_at_newlines() {
        let text = "one\ntwo words\nthree";
        assert_eq!(line_range_at(text, 6), 4..13);
        assert_eq!(line_range_at(text, 0), 0..3);
        assert_eq!(line_range_at(text, text.len()), 14..19);
    }

    #[test]
    fn double_click_on_an_emoji_selects_only_that_emoji() {
        let text = "a\u{1F44D}\u{1F44D}b";
        assert_eq!(word_range_at(text, 1), 1..5);
        assert_eq!(word_range_at(text, 5), 5..9);
    }

    #[test]
    fn ctrl_arrows_select_and_delete_by_word() {
        let mut widget = plain_widget();
        widget.insert("one two three");
        let key = |widget: &mut TextInputWidget, key, shift| {
            widget.handle_event(&UiEvent::KeyDown { key, shift }, AbsoluteRect::default());
        };
        key(&mut widget, NamedKey::WordLeft, false);
        assert_eq!(widget.cursor, 8);
        key(&mut widget, NamedKey::WordLeft, true);
        assert_eq!(widget.selection_range(), Some(4..8));
        key(&mut widget, NamedKey::DeleteWordBack, false);
        assert_eq!(
            widget.text, "one three",
            "deleting with a selection removes just the selection"
        );
        key(&mut widget, NamedKey::DeleteWordBack, false);
        assert_eq!(widget.text, "three");
        key(&mut widget, NamedKey::DocEnd, false);
        assert_eq!(widget.cursor, 5);
        key(&mut widget, NamedKey::DocStart, true);
        assert_eq!(widget.selection_range(), Some(0..5));
        key(&mut widget, NamedKey::ArrowRight, false);
        assert_eq!(
            (widget.cursor, widget.selection_range()),
            (5, None),
            "plain arrow collapses to the selection edge"
        );
    }

    #[test]
    fn arrows_and_backspace_step_over_a_whole_emoji() {
        let mut widget = plain_widget();
        widget.insert("a\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}b");
        let key = |widget: &mut TextInputWidget, key| {
            widget.handle_event(
                &UiEvent::KeyDown { key, shift: false },
                AbsoluteRect::default(),
            );
        };
        key(&mut widget, NamedKey::ArrowLeft);
        key(&mut widget, NamedKey::ArrowLeft);
        assert_eq!(widget.cursor, 1);
        key(&mut widget, NamedKey::ArrowRight);
        key(&mut widget, NamedKey::Backspace);
        assert_eq!(widget.text, "ab");
    }

    #[test]
    fn up_down_home_end_walk_visual_lines_of_a_wrapping_field() {
        let mut widget = plain_widget().with_wrap();
        widget.set_field_width(120.0);
        widget.insert("alpha beta gamma delta epsilon zeta eta theta");
        let key = |widget: &mut TextInputWidget, key, shift| {
            widget.handle_event(&UiEvent::KeyDown { key, shift }, AbsoluteRect::default());
        };
        let end = widget.text.len();
        assert_eq!(widget.cursor, end);
        key(&mut widget, NamedKey::ArrowUp, false);
        assert!(
            widget.cursor < end && widget.cursor > 0,
            "Up moves to an earlier visual line, got {}",
            widget.cursor
        );
        let after_up = widget.cursor;
        key(&mut widget, NamedKey::Home, false);
        assert!(widget.cursor <= after_up);
        let line_start = widget.cursor;
        key(&mut widget, NamedKey::End, true);
        assert!(
            widget.cursor > line_start && widget.cursor < end,
            "End stops at the end of this visual line: start {line_start} cursor {} end {end} after_up {after_up}",
            widget.cursor
        );
        assert!(
            !widget.text[..widget.cursor].ends_with(' '),
            "End stays before the wrap-point space"
        );
        assert_eq!(widget.selection_range(), Some(line_start..widget.cursor));

        key(&mut widget, NamedKey::DocEnd, false);
        key(&mut widget, NamedKey::ArrowDown, false);
        assert_eq!(widget.cursor, end);
        for _ in 0..10 {
            key(&mut widget, NamedKey::ArrowUp, false);
        }
        assert_eq!(widget.cursor, 0);
    }

    fn type_text(widget: &mut TextInputWidget, text: &str) {
        for c in text.chars() {
            widget.handle_event(
                &UiEvent::TextInput {
                    text: c.to_string(),
                },
                AbsoluteRect::default(),
            );
        }
    }

    fn press(widget: &mut TextInputWidget, key: NamedKey) {
        widget.handle_event(
            &UiEvent::KeyDown { key, shift: false },
            AbsoluteRect::default(),
        );
    }

    #[test]
    fn undo_and_redo_walk_typing_word_by_word() {
        let mut widget = plain_widget();
        type_text(&mut widget, "hello world");
        press(&mut widget, NamedKey::Undo);
        assert_eq!(widget.text, "hello ");
        press(&mut widget, NamedKey::Undo);
        assert_eq!(widget.text, "");
        press(&mut widget, NamedKey::Undo);
        assert_eq!(widget.text, "", "nothing left to undo");
        press(&mut widget, NamedKey::Redo);
        assert_eq!(widget.text, "hello ");
        press(&mut widget, NamedKey::Redo);
        assert_eq!((widget.text.as_str(), widget.cursor), ("hello world", 11));
    }

    #[test]
    fn a_new_edit_drops_the_redo_stack() {
        let mut widget = plain_widget();
        type_text(&mut widget, "abc");
        press(&mut widget, NamedKey::Undo);
        type_text(&mut widget, "x");
        press(&mut widget, NamedKey::Redo);
        assert_eq!(widget.text, "x", "redo does nothing after a fresh edit");
    }

    #[test]
    fn backspace_runs_and_deletions_undo_as_steps() {
        let mut widget = plain_widget();
        type_text(&mut widget, "abcdef");
        press(&mut widget, NamedKey::Backspace);
        press(&mut widget, NamedKey::Backspace);
        press(&mut widget, NamedKey::Backspace);
        assert_eq!(widget.text, "abc");
        press(&mut widget, NamedKey::Undo);
        assert_eq!(widget.text, "abcdef", "a run of backspaces is one step");
        press(&mut widget, NamedKey::Redo);
        assert_eq!(widget.text, "abc");
        press(&mut widget, NamedKey::DeleteWordBack);
        assert_eq!(widget.text, "");
        press(&mut widget, NamedKey::Undo);
        assert_eq!((widget.text.as_str(), widget.cursor), ("abc", 3));
    }

    #[test]
    fn undo_restores_a_replaced_selection_and_ignores_submitted_text() {
        let mut widget = plain_widget().with_wrap();
        type_text(&mut widget, "replace me");
        press(&mut widget, NamedKey::SelectAll);
        type_text(&mut widget, "new");
        assert_eq!(widget.text, "new");
        press(&mut widget, NamedKey::Undo);
        assert_eq!(widget.text, "replace me");
        press(&mut widget, NamedKey::Enter);
        assert_eq!(widget.text, "");
        press(&mut widget, NamedKey::Undo);
        assert_eq!(
            widget.text, "",
            "a submitted message can't be undone back into the box"
        );
    }

    fn fake_emoji_search(query: &str) -> Vec<DynamicMatch> {
        [
            ("smile", "\u{1F604}"),
            ("smiley", "\u{1F603}"),
            ("slight_smile", "\u{1F642}"),
            ("fire", "\u{1F525}"),
        ]
        .into_iter()
        .filter(|(label, _)| label.contains(query))
        .map(|(label, glyph)| DynamicMatch {
            label: label.to_string(),
            glyph: glyph.to_string(),
        })
        .collect()
    }

    fn emoji_widget() -> TextInputWidget {
        plain_widget().with_dynamic_autocomplete(DynamicAutocomplete {
            trigger_char: ':',
            popup_label: "Emoji".to_string(),
            min_query_len: 2,
            search: Rc::new(fake_emoji_search),
        })
    }

    #[test]
    fn emoji_lookup_needs_two_characters_and_a_word_start() {
        let mut widget = emoji_widget();
        type_text(&mut widget, "hi :s");
        assert!(
            widget.popup.is_none(),
            "one character after the colon is too short"
        );
        type_text(&mut widget, "m");
        let popup = widget.popup.as_ref().expect("popup after :sm");
        let labels: Vec<&str> = popup
            .dynamic_matches
            .as_ref()
            .unwrap()
            .iter()
            .map(|m| m.label.as_str())
            .collect();
        assert_eq!(
            labels,
            ["smile", "smiley", "slight_smile"],
            "substring matches, like the emoji picker's search"
        );

        let mut clock = emoji_widget();
        type_text(&mut clock, "meet at 12:30");
        assert!(clock.popup.is_none());
        let mut glued = emoji_widget();
        type_text(&mut glued, "a:sm");
        assert!(glued.popup.is_none(), "the colon has to start a word");
    }

    #[test]
    fn enter_completes_the_selected_emoji_and_arrows_pick_another() {
        let mut widget = emoji_widget();
        type_text(&mut widget, "hi :smi");
        press(&mut widget, NamedKey::ArrowDown);
        press(&mut widget, NamedKey::Enter);
        assert_eq!(
            widget.text, "hi \u{1F603} ",
            "the second match replaces the whole :smi, colon included"
        );
        assert!(widget.popup.is_none());
        assert_eq!(widget.cursor, widget.text.len());
    }

    fn composer_rect() -> AbsoluteRect {
        AbsoluteRect {
            x: 100.0,
            y: 500.0,
            width: 600.0,
            height: 58.0,
        }
    }

    #[test]
    fn popup_rows_are_hoverable_and_clickable() {
        let mut widget = emoji_widget();
        type_text(&mut widget, "hi :smi");
        let rect = composer_rect();
        let geometry = widget.popup_geometry(rect, widget.popup.as_ref().unwrap());
        let row_center = |row: usize| geometry.rows_top + geometry.row_height * (row as f32 + 0.5);
        let x = rect.x + 20.0;

        let extra = widget.extra_hit_rect(rect).expect("popup open");
        assert!(extra.contains(x, row_center(0)) && !rect.contains(x, row_center(0)));
        assert!(
            !extra.contains(x, rect.y + 10.0),
            "the input itself isn't part of the extra area"
        );

        widget.handle_event(
            &UiEvent::MouseMoved {
                x,
                y: row_center(2),
            },
            rect,
        );
        assert_eq!(
            widget.popup.as_ref().unwrap().selected,
            2,
            "hover selects the row"
        );

        let result = widget.handle_event(
            &UiEvent::MouseDown {
                x,
                y: geometry.rows_top - 10.0,
                button: MouseButton::Left,
                shift: false,
            },
            rect,
        );
        assert_eq!(result, EventResult::Handled);
        assert_eq!(widget.text, "hi :smi");

        widget.handle_event(
            &UiEvent::MouseDown {
                x,
                y: row_center(1),
                button: MouseButton::Left,
                shift: false,
            },
            rect,
        );
        assert_eq!(widget.text, "hi \u{1F603} ");
        assert!(widget.popup.is_none());
    }

    #[test]
    fn popup_keeps_8px_of_padding_below_its_last_row() {
        let mut widget = emoji_widget();
        type_text(&mut widget, ":smi");
        let rect = composer_rect();
        let popup = widget.popup.as_ref().unwrap();
        let geometry = widget.popup_geometry(rect, popup);
        let rows_bottom = geometry.rows_top + geometry.row_height * popup.filtered.len() as f32;
        assert_eq!(geometry.panel.bottom - rows_bottom, 8.0);
        assert_eq!(
            rect.y - geometry.panel.bottom,
            8.0,
            "8px gap to the composer"
        );
    }

    #[test]
    fn typing_a_complete_shortcode_converts_it() {
        let mut widget = emoji_widget();
        type_text(&mut widget, "go :fire:");
        assert_eq!(widget.text, "go \u{1F525}");
        let mut partial = emoji_widget();
        type_text(&mut partial, "go :fir:");
        assert_eq!(partial.text, "go :fir:", "only an exact shortcode converts");
    }
}

#[cfg(test)]
mod layout_invariant_tests {
    use crate::font_system;
    use crate::widget::Widget;
    use super::*;

    const LINE: f32 = 22.0;
    const WIDTHS: [f32; 4] = [180.0, 320.0, 560.0, 1000.0];

    const RUST: &str = "Here's the fix for the parser:\n```rust\nuse std::collections::HashMap;\n\n/// Counts words, case-insensitively.\npub fn word_counts(text: &str) -> HashMap<String, usize> {\n    let mut counts = HashMap::new();\n    for word in text.split_whitespace().map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase()) {\n        if word.is_empty() { continue; }\n        *counts.entry(word).or_insert(0) += 1;\n    }\n    counts\n}\n```\nLet me know if `cargo test` passes, **especially** the unicode one";
    const TS_TABS: &str = "```ts\nconst greet = (name: string): string => `Hello, ${name}!`;\nexport async function load(url: string) {\n\tconst res = await fetch(url, { headers: { \"Content-Type\": \"application/json\" } });\n\tif (!res.ok) throw new Error(`HTTP ${res.status}`);\n\treturn res.json() as Promise<Record<string, unknown>>;\n}\n```";
    const PYTHON_UNICODE: &str = "two blocks:\n```py\n# café ✓ — naïve façade, 日本語のコメント\ndef f(x):\n\n\n    return \"\"\"multi\n    line ``` not a fence? \"\"\"\n```\nand then\n```\n\n```\n> quoted after <@42> with `code`\n> second quote line";
    const UNCLOSED: &str = "wip:\n```py\ndef f(x):\n    return x * 2\n\n";
    const SAME_LINE_AND_NOT_LANG: &str = "a ```b **x**``` c\n```py extra\nx = 1\nfoo```\n\\```\nz\n```diff\n- old line\n+ new line that is quite a lot longer than the old one ever was\n```";
    const LONG_TOKEN: &str = "```\nAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\nhttps://example.com/a/very/long/path?with=query&and=more&stuff=1234567890\n```\nafter a long token";
    const EDGES: &str = "\n\n```\n```\n\nend with blank lines\n\n";
    const RTL: &str = "```js\n// تعليق باللغة العربية\nlet x = \"مرحبا\";\n```";

    const SAMPLES: [&str; 8] = [
        RUST,
        TS_TABS,
        PYTHON_UNICODE,
        UNCLOSED,
        SAME_LINE_AND_NOT_LANG,
        LONG_TOKEN,
        EDGES,
        RTL,
    ];

    fn composer(text: &str, width: f32) -> TextInputWidget {
        let font_system = Rc::new(RefCell::new(font_system::make_font_system()));
        let glyph_cache = Rc::new(GlyphCache::new(1024 * 1024));
        let trigger = AutocompleteTrigger {
            trigger_char: '@',
            candidates: vec![AutocompleteCandidate {
                display: "example".to_string(),
                id: Some(42),
                ..Default::default()
            }],
            only_at_message_start: false,
            chip_background: Some(Color::from_argb(255, 88, 101, 242)),
            chip_text_color: Color::WHITE,
            popup_label: "Members".to_string(),
            multi_word: false,
            whole_text: false,
        };

        let highlighter: CodeHighlighter = Rc::new(|_lang: &str, code: &str| {
            code.match_indices(|c: char| c.is_alphabetic())
                .step_by(3)
                .map(|(i, c)| (i..i + c.len(), Color::RED))
                .collect()
        });
        let widget = TextInputWidget::new(
            text,
            "",
            16.0,
            Color::WHITE,
            Color::WHITE,
            font_system,
            glyph_cache,
            false,
        )
        .with_autocomplete(vec![trigger])
        .with_markdown(MarkdownFeatures::ALL)
        .with_code_highlighter(highlighter)
        .with_line_height(LINE)
        .with_wrap();
        widget.set_field_width(width);
        widget
    }

    fn stops(widget: &TextInputWidget) -> Vec<usize> {
        let chips = widget.find_chip_ranges(&widget.text);
        let mut out = vec![0];
        let mut pos = 0;
        while let Some(next) = next_grapheme_boundary(&widget.text, pos) {
            pos = next;
            if !chips
                .iter()
                .any(|c| c.range.start < pos && pos < c.range.end)
            {
                out.push(pos);
            }
        }
        out
    }

    fn label(text: &str, width: f32) -> String {
        format!(
            "width {width}, text {:?}",
            text.chars().take(40).collect::<String>()
        )
    }

    #[test]
    fn every_visual_line_is_one_uniform_line_box() {
        for text in SAMPLES {
            for width in WIDTHS {
                let widget = composer(text, width);
                widget.with_layout(|layout| {
                    for (i, run) in layout.full.layout_runs().enumerate() {
                        assert_eq!(
                            run.line_height,
                            LINE,
                            "line {i} height, {}",
                            label(text, width)
                        );
                        assert!(
                            (run.line_top - i as f32 * LINE).abs() < 0.01,
                            "line {i} top {} ({})",
                            run.line_top,
                            label(text, width)
                        );
                    }
                });
            }
        }
    }

    #[test]
    fn baselines_are_consistent_per_line_kind() {
        for text in SAMPLES {
            for width in WIDTHS {
                let widget = composer(text, width);
                widget.with_layout(|layout| {
                    let mut by_kind: HashMap<u32, f32> =
                        HashMap::new();
                    for run in layout.full.layout_runs() {
                        let Some(first) = run.glyphs.first() else {
                            continue;
                        };
                        let baseline = run.line_y - run.line_top;
                        assert!(
                            baseline > 0.0 && baseline < LINE,
                            "baseline {baseline} outside its line ({})",
                            label(text, width)
                        );

                        if run
                            .glyphs
                            .iter()
                            .all(|g| g.font_size == first.font_size && g.font_id == first.font_id)
                        {
                            let kind = first.font_size.to_bits();
                            let seen = *by_kind.entry(kind).or_insert(baseline);
                            assert!(
                                (seen - baseline).abs() < 0.01,
                                "baseline {baseline} vs {seen} for {}px lines ({})",
                                first.font_size,
                                label(text, width)
                            );
                        }
                    }
                });
            }
        }
    }

    #[test]
    fn caret_round_trips_through_hit_testing() {
        for text in SAMPLES {
            for width in WIDTHS {
                let mut widget = composer(text, width);
                for pos in stops(&widget) {
                    widget.cursor = pos;
                    let (x, top, h) = widget.caret_geometry();
                    assert_eq!(h, LINE, "caret height at {pos} ({})", label(text, width));
                    let hit = widget.cursor_index_for_local_point(x, top + h / 2.0);
                    let same_spot = widget.with_layout(|layout| {
                        let map = layout.map.as_ref().unwrap();
                        let (a, b) = (map.cursor(hit, false), map.cursor(pos, false));

                        let bidi = layout.full.layout_runs().any(|r| {
                            r.line_i == b.line && r.glyphs.iter().any(|g| g.level.is_rtl())
                        });
                        if bidi {
                            a.line == b.line
                        } else {
                            map.to_layout(hit, false) == map.to_layout(pos, false)
                        }
                    });
                    assert!(
                        same_spot,
                        "caret at {pos} drawn at ({x}, {top}) hit-tests to {hit} ({})",
                        label(text, width)
                    );
                }
            }
        }
    }

    #[test]
    fn caret_moves_forward_along_each_visual_line() {
        for text in SAMPLES {
            if text == RTL {
                continue;
            }
            for width in WIDTHS {
                let mut widget = composer(text, width);
                let mut last: Option<(f32, f32, usize)> = None;
                for pos in stops(&widget) {
                    widget.cursor = pos;
                    let (x, top, _) = widget.caret_geometry();
                    if let Some((last_x, last_top, last_pos)) = last {
                        assert!(
                            top >= last_top,
                            "caret went up from {last_pos} to {pos} ({})",
                            label(text, width)
                        );
                        if top == last_top {
                            assert!(
                                x >= last_x - 0.01,
                                "caret went left from {last_pos} ({last_x}) to {pos} ({x}) ({})",
                                label(text, width)
                            );
                        }
                    }
                    last = Some((x, top, pos));
                }
            }
        }
    }

    #[test]
    fn caret_is_on_its_own_source_line() {
        for text in SAMPLES {
            for width in WIDTHS {
                let mut widget = composer(text, width);
                for pos in stops(&widget) {
                    widget.cursor = pos;
                    let (_, top, _) = widget.caret_geometry();
                    let ok = widget.with_layout(|layout| {
                        let map = layout.map.as_ref().unwrap();
                        let line = map.cursor(pos, false).line;
                        layout
                            .full
                            .layout_runs()
                            .any(|run| run.line_i == line && run.line_top == top)
                    });
                    assert!(
                        ok,
                        "caret at {pos} drawn on another line's row ({})",
                        label(text, width)
                    );
                }
            }
        }
    }

    #[test]
    fn select_all_covers_every_line() {
        for text in SAMPLES {
            for width in WIDTHS {
                let widget = composer(text, width);
                widget.with_layout(|layout| {
                    let map = layout.map.as_ref().unwrap();
                    let rects = selection_rects(
                        &layout.full,
                        map.cursor(0, false),
                        map.cursor(text.len(), true),
                        4.0,
                    );
                    let runs: Vec<_> = layout.full.layout_runs().collect();
                    let last_line = runs.last().map_or(0, |r| r.line_i);
                    for run in &runs {
                        let row: Vec<_> = rects.iter().filter(|r| r.1 == run.line_top).collect();
                        assert!(
                            row.iter().all(|r| r.3 == LINE),
                            "rect height ({})",
                            label(text, width)
                        );
                        let left = row.iter().map(|r| r.0).fold(f32::INFINITY, f32::min);
                        let right = row
                            .iter()
                            .map(|r| r.0 + r.2)
                            .fold(f32::NEG_INFINITY, f32::max);

                        if !run.glyphs.is_empty() && !run.rtl && run.line_w > 0.5 {
                            assert!(
                                left <= run.glyphs[0].x + 0.01,
                                "row at {} starts at {left} ({})",
                                run.line_top,
                                label(text, width)
                            );
                            assert!(
                                right >= run.line_w - 0.01,
                                "row at {} ends at {right} < {} ({})",
                                run.line_top,
                                run.line_w,
                                label(text, width)
                            );
                        }

                        if !run.rtl
                            && run.line_i < last_line
                            && runs
                                .iter().rfind(|r| r.line_i == run.line_i)
                                .is_some_and(|r| r.line_top == run.line_top)
                        {
                            assert!(
                                !row.is_empty(),
                                "line {} has no selection ({})",
                                run.line_i,
                                label(text, width)
                            );
                            assert!(
                                right <= run.line_w + 4.0 + 0.01,
                                "line {} selection runs past its text to {right} ({})",
                                run.line_i,
                                label(text, width)
                            );
                        }
                    }
                });
            }
        }
    }

    #[test]
    fn windows_line_breaks_are_normalized_on_insert() {
        let mut widget = composer("", 560.0);
        widget.insert("```rust\r\nfn main() {}\r\n```\rdone\u{2029}x");
        assert_eq!(widget.text, "```rust\nfn main() {}\n```\ndone\nx");
        let blocks = markdown::composer_code_blocks(&widget.text);
        assert_eq!(blocks[0].lang.as_deref(), Some("rust"));
        widget.with_layout(|layout| assert_eq!(layout.full.layout_runs().count(), 5));
    }

    #[test]
    fn trailing_line_break_gets_its_own_line() {
        let mut widget = composer("```rust\nfn x() {}", 560.0);
        let before = widget.with_layout(|layout| buffer_extent(&layout.full).1);
        widget.cursor = widget.text.len();
        widget.insert("\n");
        let after = widget.with_layout(|layout| buffer_extent(&layout.full).1);
        assert_eq!(after, before + LINE);
        let (x, top, _) = widget.caret_geometry();
        assert_eq!((x, top), (0.0, before));
    }

    #[test]
    fn drag_moves_never_relayout() {
        let mut widget = composer(RUST, 900.0);
        let rect = AbsoluteRect {
            x: 0.0,
            y: 0.0,
            width: 900.0,
            height: 420.0,
        };
        widget.handle_event(
            &UiEvent::MouseDown {
                x: 20.0,
                y: 20.0,
                button: MouseButton::Left,
                shift: false,
            },
            rect,
        );
        assert_eq!(
            widget.handle_event(&UiEvent::MouseMoved { x: 300.0, y: 120.0 }, rect),
            EventResult::Handled
        );
        assert!(!widget.relayout_after(&UiEvent::MouseMoved { x: 300.0, y: 120.0 }));
        assert!(widget.selection_range().is_some_and(|r| !r.is_empty()));
        assert_eq!(
            widget.handle_event(&UiEvent::MouseMoved { x: 300.4, y: 120.0 }, rect),
            EventResult::Ignored
        );
    }

    #[test]
    fn single_line_selection_stays_on_its_line() {
        let text = RUST;
        let start = text.find("let mut counts").unwrap();
        let end = start + "let mut counts".len();
        let widget = composer(text, 1000.0);
        widget.with_layout(|layout| {
            let map = layout.map.as_ref().unwrap();
            let rects = selection_rects(
                &layout.full,
                map.cursor(start, false),
                map.cursor(end, true),
                4.0,
            );
            assert_eq!(rects.len(), 1, "{rects:?}");
            let line = map.cursor(start, false).line;
            let run = layout
                .full
                .layout_runs()
                .find(|r| r.line_i == line)
                .unwrap();
            let first = run
                .glyphs
                .iter()
                .find(|g| g.start == map.cursor(start, false).index)
                .unwrap();
            assert!((rects[0].0 - first.x).abs() < 0.01);
            assert_eq!(rects[0].1, run.line_top);
        });
    }
}
