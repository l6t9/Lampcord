use crate::app;
use crate::colors;
use crate::event::CursorIcon;
use crate::selection;
use crate::style::TextLayoutStyle;
use skia_safe::gradient::Colors;
use skia_safe::gradient::Gradient;
use skia_safe::gradient::Interpolation;
use skia_safe::gradient::shaders::linear_gradient;
use skia_safe::surfaces;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops;
use std::rc::Rc;
use unicode_segmentation::UnicodeSegmentation;

use cosmic_text::fontdb::Weight as FdWeight;
use cosmic_text::{
    Align, Attrs, Buffer, Color as CosmicColor, Family, FontSystem, Metrics, Shaping, Wrap,
};
use skia_safe::font_style::Weight;
use skia_safe::{Canvas, Color, Paint, Rect};
use taffy::{AvailableSpace, Size};

use crate::font_system::gg_sans_family;
use crate::glyph_cache::{GlyphCache, SkiaDecorationRenderer, paint_glyph};
use crate::style::ComputedStyle;
use crate::text_cache::{MeasureCache, TextCacheKey};
use crate::widget::{AbsoluteRect, Widget};

pub struct TextWidget {
    text: String,
    font_size: f32,
    color: Color,

    hover_color: Option<Color>,
    weight: Weight,
    font_system: Rc<RefCell<FontSystem>>,
    glyph_cache: Rc<GlyphCache>,

    cache: RefCell<Vec<(u32, bool, Buffer)>>,
    theme_seen: Cell<u64>,

    layer: RefCell<Option<TextLayer>>,

    last_measured_width: Cell<Option<f32>>,

    measure_cache: Option<Rc<MeasureCache>>,
    cache_key: Option<TextCacheKey>,

    centered: bool,

    strong: Vec<(ops::Range<usize>, Weight)>,

    line_height: Option<f32>,

    italic: bool,

    letter_spacing: f32,

    gradient: Option<TextGradient>,

    text_layout: TextLayoutStyle,
}

impl TextWidget {
    pub fn new(
        text: impl Into<String>,
        font_size: f32,
        color: Color,
        font_system: Rc<RefCell<FontSystem>>,
        glyph_cache: Rc<GlyphCache>,
    ) -> Self {
        Self::new_weighted(
            text,
            font_size,
            color,
            font_system,
            glyph_cache,
            Weight::NORMAL,
        )
    }

    pub fn new_weighted(
        text: impl Into<String>,
        font_size: f32,
        color: Color,
        font_system: Rc<RefCell<FontSystem>>,
        glyph_cache: Rc<GlyphCache>,
        weight: Weight,
    ) -> Self {
        Self {
            text: text.into(),
            font_size,
            color,
            hover_color: None,
            weight,
            font_system,
            glyph_cache,
            cache: RefCell::new(Vec::new()),
            theme_seen: Cell::new(crate::theme_generation()),
            layer: RefCell::new(None),
            last_measured_width: Cell::new(None),
            measure_cache: None,
            cache_key: None,
            centered: false,
            strong: Vec::new(),
            line_height: None,
            italic: false,
            letter_spacing: 0.0,
            gradient: None,
            text_layout: TextLayoutStyle::default(),
        }
    }

    pub fn with_letter_spacing(mut self, em: f32) -> Self {
        self.letter_spacing = em;
        self
    }

    pub fn with_italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn with_gradient(mut self, colors: Vec<Color>, tile: f32) -> Self {
        self.gradient = Some(TextGradient { colors, tile });
        self
    }

    pub fn with_line_height(mut self, line_height: f32) -> Self {
        self.line_height = Some(line_height);
        self
    }

    fn metrics(&self) -> Metrics {
        Metrics::new(
            self.font_size,
            self.line_height.unwrap_or(self.font_size * 1.2),
        )
    }

    pub fn with_center(mut self) -> Self {
        self.centered = true;
        self
    }

    pub fn with_strong_range(mut self, range: ops::Range<usize>, weight: Weight) -> Self {
        self.strong.push((range, weight));
        self
    }

    pub fn with_hover_color(mut self, hover_color: Color) -> Self {
        self.hover_color = Some(hover_color);
        self
    }

    pub fn with_measure_cache(mut self, cache: Rc<MeasureCache>, key: Option<u64>) -> Self {
        self.cache_key = Some(
            key.map(TextCacheKey::Stable)
                .unwrap_or_else(|| TextCacheKey::from_text(&self.text)),
        );
        self.measure_cache = Some(cache);
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.cache.borrow_mut().clear();
        self.layer.borrow_mut().take();
    }

    fn active_color(&self, style: &ComputedStyle) -> Color {
        if let Some(color) = style.text_color {
            return color;
        }
        let color = if style.is_hovered {
            self.hover_color.unwrap_or(self.color)
        } else {
            self.color
        };
        colors::live(color)
    }

    fn fontdb_weight(&self) -> FdWeight {
        FdWeight(*self.weight as u16)
    }

    fn attrs(&self, underline: bool) -> Attrs<'static> {
        self.attrs_weighted(underline, self.fontdb_weight())
    }

    fn attrs_weighted(&self, underline: bool, weight: FdWeight) -> Attrs<'static> {
        let style = if underline {
            cosmic_text::UnderlineStyle::Single
        } else {
            cosmic_text::UnderlineStyle::None
        };
        let font_style = if self.italic {
            cosmic_text::Style::Italic
        } else {
            cosmic_text::Style::Normal
        };
        let attrs = Attrs::new()
            .family(Family::Name(gg_sans_family(weight, self.italic)))
            .weight(weight)
            .style(font_style)
            .underline(style);
        if self.letter_spacing != 0.0 {
            attrs.letter_spacing(self.letter_spacing)
        } else {
            attrs
        }
    }

    fn build_buffer(&self, width: f32, underline: bool) -> Buffer {
        let mut font_system = self.font_system.borrow_mut();
        let layout = self.text_layout;
        let buffer = self.shape_text(&mut font_system, &self.text, width, underline);

        let fits: Option<Box<dyn Fn(&Buffer) -> bool>> = if layout.nowrap && layout.ellipsis {
            Some(Box::new(move |b: &Buffer| {
                b.layout_runs().fold(0.0f32, |w, run| w.max(run.line_w)) <= width + 0.5
            }))
        } else if let Some(lines) = layout.line_clamp.filter(|_| !layout.nowrap) {
            Some(Box::new(move |b: &Buffer| {
                b.layout_runs().count() <= lines as usize
            }))
        } else {
            None
        };
        let Some(fits) = fits else { return buffer };
        if fits(&buffer) {
            return buffer;
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
            if fits(&self.shape_text(&mut font_system, &truncated(mid), width, underline)) {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        self.shape_text(&mut font_system, &truncated(lo), width, underline)
    }

    fn shape_text(
        &self,
        font_system: &mut FontSystem,
        text: &str,
        width: f32,
        underline: bool,
    ) -> Buffer {
        let mut buffer = Buffer::new(font_system, self.metrics());
        buffer.set_wrap(if self.text_layout.nowrap {
            Wrap::None
        } else {
            Wrap::WordOrGlyph
        });
        buffer.set_size(Some(width.max(0.0)), None);
        self.set_buffer_text_of(&mut buffer, text, underline);
        buffer.shape_until_scroll(font_system, false);
        buffer
    }

    fn natural_width(&self) -> f32 {
        let mut font_system = self.font_system.borrow_mut();
        let mut buffer = Buffer::new(&mut font_system, self.metrics());
        buffer.set_wrap(Wrap::None);
        buffer.set_size(None, None);
        self.set_buffer_text_of(&mut buffer, &self.text, false);
        buffer.shape_until_scroll(&mut font_system, false);
        buffer
            .layout_runs()
            .fold(0.0f32, |w, run| w.max(run.line_w))
            + 1.0
    }

    fn set_buffer_text(&self, buffer: &mut Buffer, underline: bool) {
        self.set_buffer_text_of(buffer, &self.text, underline);
    }

    fn set_buffer_text_of(&self, buffer: &mut Buffer, text: &str, underline: bool) {
        let base = self.attrs(underline);
        if self.strong.is_empty() {
            buffer.set_text(text, &base, Shaping::Advanced, None);
        } else {
            let mut spans: Vec<(&str, Attrs)> = Vec::new();
            let mut pos = 0;
            let mut strong = self.strong.clone();
            strong.sort_by_key(|(r, _)| r.start);
            for (range, weight) in strong {
                if range.start < pos
                    || range.end > text.len()
                    || !text.is_char_boundary(range.start)
                    || !text.is_char_boundary(range.end)
                {
                    continue;
                }
                spans.push((&text[pos..range.start], base.clone()));
                spans.push((
                    &text[range.clone()],
                    self.attrs_weighted(underline, FdWeight(*weight as u16)),
                ));
                pos = range.end;
            }
            spans.push((&text[pos..], base.clone()));
            buffer.set_rich_text(spans, &base, Shaping::Advanced, None);
        }
        if self.centered {
            for line in buffer.lines.iter_mut() {
                line.set_align(Some(Align::Center));
            }
        }
    }

    fn min_content_width(&self) -> f32 {
        if self.text_layout.nowrap {
            return self.natural_width();
        }
        let mut font_system = self.font_system.borrow_mut();
        let metrics = self.metrics();
        let mut buffer = Buffer::new(&mut font_system, metrics);
        buffer.set_wrap(Wrap::Word);
        buffer.set_size(Some(0.0), None);
        self.set_buffer_text(&mut buffer, false);
        buffer.shape_until_scroll(&mut font_system, false);
        buffer
            .layout_runs()
            .fold(0.0f32, |max, run| max.max(run.line_w))
            + 1.0
    }

    fn paint_width(&self, box_width: f32, box_height: f32) -> f32 {
        if self.text_layout.nowrap {
            return box_width;
        }
        let reserved_lines = (box_height / self.metrics().line_height.max(1.0))
            .round()
            .max(1.0) as usize;
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
            if self.with_buffer(width, false, |buffer| buffer.layout_runs().count())
                == reserved_lines
            {
                return width;
            }
        }
        box_width
    }

    fn with_buffer<R>(&self, width: f32, underline: bool, f: impl FnOnce(&Buffer) -> R) -> R {
        let width = quantize_width(width);
        let theme = crate::theme_generation();
        let mut cache = self.cache.borrow_mut();
        if self.theme_seen.get() != theme {
            cache.clear();
            self.layer.borrow_mut().take();
            self.theme_seen.set(theme);
        }
        let index = match cache
            .iter()
            .position(|(w, u, _)| *w == width.to_bits() && *u == underline)
        {
            Some(index) => index,
            None => {
                if cache.len() >= BUFFER_CACHE_SLOTS {
                    cache.remove(0);
                }
                cache.push((
                    width.to_bits(),
                    underline,
                    self.build_buffer(width, underline),
                ));
                cache.len() - 1
            }
        };
        f(&cache[index].2)
    }
}

const UNBOUNDED_WIDTH: f32 = 10_000.0;

#[cfg(any(test, feature = "test-hooks"))]
thread_local! {
    static PAINTED_LINES: RefCell<HashMap<String, (usize, f32)>> = RefCell::new(HashMap::new());
}

#[cfg(any(test, feature = "test-hooks"))]
pub fn painted_lines_of(text_prefix: &str) -> Option<(usize, f32)> {
    PAINTED_LINES.with(|m| {
        m.borrow()
            .iter()
            .find(|(key, _)| key.starts_with(text_prefix) || text_prefix.starts_with(key.as_str()))
            .map(|(_, lines)| *lines)
    })
}

pub(super) const BUFFER_CACHE_SLOTS: usize = 4;

pub(super) fn quantize_width(width: f32) -> f32 {
    width.round()
}

fn to_cosmic_color(color: Color) -> CosmicColor {
    CosmicColor::rgba(color.r(), color.g(), color.b(), color.a())
}

fn buffer_extent(buffer: &Buffer) -> (f32, f32) {
    buffer.layout_runs().fold((0.0f32, 0.0f32), |(w, h), run| {
        (w.max(run.line_w), h.max(run.line_top + run.line_height))
    })
}

impl Widget for TextWidget {
    fn debug_props(&self) -> Vec<(&'static str, String)> {
        vec![
            ("text", format!("{:?}", self.text)),
            ("font_size", format!("{}px", self.font_size)),
            ("weight", format!("{}", *self.weight)),
        ]
    }

    fn measure(
        &self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        let width = match known_dimensions.width {
            Some(w) => quantize_width(w),
            None => match available_space.width {
                _ if self.text_layout.nowrap => UNBOUNDED_WIDTH,
                AvailableSpace::Definite(w) => quantize_width(w),
                AvailableSpace::MinContent => self.min_content_width().ceil(),
                AvailableSpace::MaxContent => UNBOUNDED_WIDTH,
            },
        };
        if known_dimensions.width.is_some()
            || (!self.text_layout.nowrap
                && matches!(available_space.width, AvailableSpace::Definite(_)))
        {
            self.last_measured_width.set(Some(width));
        }

        let compute = || {
            if width < UNBOUNDED_WIDTH {
                let (natural_w, natural_h) =
                    self.with_buffer(UNBOUNDED_WIDTH, false, buffer_extent);
                if natural_w + 1.0 <= width {
                    return ((natural_w + 1.0).min(width.max(0.0)), natural_h);
                }
            }

            if width < UNBOUNDED_WIDTH {
                let (natural_w, natural_h) =
                    self.with_buffer(UNBOUNDED_WIDTH, false, buffer_extent);
                if natural_w + 1.0 <= width {
                    return ((natural_w + 1.0).min(width.max(0.0)), natural_h);
                }
            }
            self.with_buffer(width, false, |buffer| {
                let (w, h) = buffer_extent(buffer);
                ((w + 1.0).min(width.max(0.0)), h)
            })
        };

        let (w, h) = match (&self.measure_cache, self.cache_key) {
            (Some(cache), Some(key)) => {
                cache.get_or_insert_with(key, self.font_size, self.weight, Some(width), compute)
            }
            _ => compute(),
        };
        Size {
            width: w,
            height: h,
        }
    }

    fn first_baseline(&self, content_width: f32) -> Option<f32> {
        self.with_buffer(content_width, false, |buffer| {
            buffer.layout_runs().next().map(|run| run.line_y)
        })
    }

    fn apply_text_style(&mut self, text: &TextLayoutStyle) -> bool {
        if self.text_layout == *text {
            return false;
        }
        self.text_layout = *text;
        self.cache.borrow_mut().clear();
        self.layer.borrow_mut().take();
        true
    }
    fn cursor_icon(&self) -> Option<CursorIcon> {
        self.selection_key().map(|_| CursorIcon::Text)
    }

    fn selection_key(&self) -> Option<u64> {
        match self.cache_key {
            Some(TextCacheKey::Stable(id)) if !self.text.is_empty() => Some(id),
            _ => None,
        }
    }

    fn selection_offset_at(&self, rect: AbsoluteRect, x: f32, y: f32) -> u64 {
        self.with_buffer(self.paint_width(rect.width, rect.height), false, |buffer| {
            offset_at(buffer, x - rect.x, y - rect.y)
        })
    }

    fn selection_end(&self, rect: AbsoluteRect) -> u64 {
        self.with_buffer(self.paint_width(rect.width, rect.height), false, |buffer| {
            buffer.layout_runs().last().map_or(0, |run| {
                super::message_text::encode_cursor(
                    run.line_i,
                    run.glyphs.last().map_or(0, |g| g.end),
                )
            })
        })
    }

    fn selection_word(&self, rect: AbsoluteRect, offset: u64) -> (u64, u64) {
        self.with_buffer(self.paint_width(rect.width, rect.height), false, |buffer| {
            word_range(buffer, offset)
        })
    }

    fn selection_paragraph(&self, rect: AbsoluteRect, offset: u64) -> (u64, u64) {
        self.with_buffer(self.paint_width(rect.width, rect.height), false, |buffer| {
            let (line, _) = super::message_text::decode_cursor(offset);
            match buffer.lines.get(line) {
                Some(l) => (
                    super::message_text::encode_cursor(line, 0),
                    super::message_text::encode_cursor(line, l.text().len()),
                ),
                None => (offset, offset),
            }
        })
    }

    fn selection_text(&self, rect: AbsoluteRect, range: (u64, u64)) -> String {
        self.with_buffer(self.paint_width(rect.width, rect.height), false, |buffer| {
            selected_text(buffer, range)
        })
    }

    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let active = self.active_color(style);

        if active.a() == 0 && self.gradient.is_none() {
            return;
        }
        let color = to_cosmic_color(active);
        let scale_factor = app::current_scale_factor();
        let wrap_width = self.paint_width(rect.width, rect.height);
        self.with_buffer(wrap_width, style.text_underline, |buffer| {
            #[cfg(any(test, feature = "test-hooks"))]
            PAINTED_LINES.with(|m| {
                m.borrow_mut().insert(
                    self.text.chars().take(40).collect(),
                    (buffer.layout_runs().count(), rect.height),
                )
            });

            let origin = (rect.x * scale_factor, rect.y * scale_factor);

            let (px_w, px_h) = (
                (rect.width * scale_factor).ceil() as i32 + 2 * LAYER_PAD,
                (rect.height * scale_factor).ceil() as i32 + 2 * LAYER_PAD,
            );
            let key = LayerKey {
                wrap: wrap_width.to_bits(),
                underline: style.text_underline,
                color: color.0,
                scale: scale_factor.to_bits(),
                w: px_w,
                h: px_h,
                gradient: self.gradient.as_ref().map_or(0, TextGradient::key),
            };
            let cached = self
                .layer
                .borrow()
                .as_ref()
                .filter(|l| l.key == key)
                .map(|l| l.image.clone());
            let layer = match cached {
                Some(image) => Some(image),
                None if px_w <= MAX_LAYER_EDGE && px_h <= MAX_LAYER_EDGE => {
                    let image = self.cached_layer(buffer, color, scale_factor, (px_w, px_h), key);
                    if let Some(image) = &image {
                        *self.layer.borrow_mut() = Some(TextLayer {
                            key,
                            image: image.clone(),
                        });
                    }
                    image
                }
                None => None,
            };
            canvas.save();
            canvas.scale((1.0 / scale_factor, 1.0 / scale_factor));
            paint_selection(canvas, buffer, self.cache_key, origin, scale_factor);
            match layer {
                Some(image) => {
                    canvas.draw_image(
                        &image,
                        (
                            origin.0.round() - LAYER_PAD as f32,
                            origin.1.round() - LAYER_PAD as f32,
                        ),
                        None,
                    );
                }
                None => {
                    let mut font_system = self.font_system.borrow_mut();
                    draw_buffer(
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
            canvas.restore();
        });
    }
}

impl TextWidget {
    fn cached_layer(
        &self,
        buffer: &Buffer,
        color: CosmicColor,
        scale_factor: f32,
        size: (i32, i32),
        per_widget: LayerKey,
    ) -> Option<skia_safe::Image> {
        let key = LayerCacheKey {
            content: content_hash(&self.text, &self.strong),
            font_size: self.font_size.to_bits(),
            line_height: self.line_height.map_or(u32::MAX, f32::to_bits),
            weight: *self.weight as u16,
            italic: self.italic,
            letter_spacing: self.letter_spacing.to_bits(),
            nowrap: self.text_layout.nowrap,
            ellipsis: self.text_layout.ellipsis,
            line_clamp: self.text_layout.line_clamp.unwrap_or(0),
            centered: self.centered,
            wrap: per_widget.wrap,
            underline: per_widget.underline,
            color: per_widget.color,
            scale: per_widget.scale,
            w: per_widget.w,
            h: per_widget.h,
            gradient: per_widget.gradient,
        };
        TEXT_LAYER_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            if TEXT_LAYER_BYTES.with(Cell::get) > TEXT_LAYER_BYTE_LIMIT {
                cache.clear();
                TEXT_LAYER_BYTES.with(|b| b.set(0));
            }
            if let Some(image) = cache.get(&key) {
                return Some(image.clone());
            }
            let image = self.render_layer(buffer, color, scale_factor, size);
            if let Some(image) = &image {
                cache.insert(key, image.clone());
                TEXT_LAYER_BYTES.with(|b| b.set(b.get() + size.0 as usize * size.1 as usize * 4));
            }
            image
        })
    }

    fn render_layer(
        &self,
        buffer: &Buffer,
        color: CosmicColor,
        scale_factor: f32,
        size: (i32, i32),
    ) -> Option<skia_safe::Image> {
        let mut surface = surfaces::raster_n32_premul(size)?;
        surface.canvas().clear(Color::TRANSPARENT);
        let mut font_system = self.font_system.borrow_mut();
        draw_buffer(
            surface.canvas(),
            &self.glyph_cache,
            &mut font_system,
            buffer,
            color,
            scale_factor,
            (LAYER_PAD as f32, LAYER_PAD as f32),
        );
        if let Some(gradient) = &self.gradient {
            gradient.fill_glyphs(surface.canvas(), scale_factor, size);
        }
        Some(surface.image_snapshot())
    }
}

fn offset_at(buffer: &Buffer, local_x: f32, local_y: f32) -> u64 {
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
            return super::message_text::encode_cursor(run.line_i, glyph.start);
        }
        index = glyph.end;
    }
    super::message_text::encode_cursor(run.line_i, index)
}

fn word_range(buffer: &Buffer, offset: u64) -> (u64, u64) {
    let (line, index) = super::message_text::decode_cursor(offset);
    let Some(text) = buffer.lines.get(line).map(|l| l.text()) else {
        return (offset, offset);
    };
    for (start, word) in text.split_word_bound_indices() {
        if index >= start && index < start + word.len() {
            return (
                super::message_text::encode_cursor(line, start),
                super::message_text::encode_cursor(line, start + word.len()),
            );
        }
    }
    (offset, offset)
}

fn selected_text(buffer: &Buffer, range: (u64, u64)) -> String {
    let ((start_line, start_index), (end_line, end_index)) = (
        super::message_text::decode_cursor(range.0),
        super::message_text::decode_cursor(range.1),
    );
    let mut out = String::new();
    for (line, text) in buffer.lines.iter().map(|l| l.text()).enumerate() {
        if line < start_line || line > end_line {
            continue;
        }
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
        out.push_str(&text[from..to]);
        if line < end_line {
            out.push('\n');
        }
    }
    out
}

fn paint_selection(
    canvas: &Canvas,
    buffer: &Buffer,
    key: Option<TextCacheKey>,
    origin: (f32, f32),
    scale: f32,
) {
    let Some(TextCacheKey::Stable(id)) = key else {
        return;
    };
    let Some(range) = selection::range_for(id) else {
        return;
    };
    for run in buffer.layout_runs() {
        let Some((x, w)) = super::message_text::selection_span(&run, range) else {
            continue;
        };
        let mut paint = Paint::default();
        paint.set_color(super::message_text::SELECTION_BACKGROUND);
        canvas.draw_rect(
            Rect::from_xywh(
                origin.0 + x * scale,
                origin.1 + run.line_top * scale,
                w * scale,
                run.line_height * scale,
            ),
            &paint,
        );
    }
}

pub(super) const LAYER_PAD: i32 = 4;

pub(super) const MAX_LAYER_EDGE: i32 = 2048;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct LayerKey {
    pub(super) wrap: u32,
    pub(super) underline: bool,
    pub(super) color: u32,
    pub(super) scale: u32,
    pub(super) w: i32,
    pub(super) h: i32,

    pub(super) gradient: u64,
}

struct TextGradient {
    colors: Vec<Color>,
    tile: f32,
}

impl TextGradient {
    fn key(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.tile.to_bits().hash(&mut hasher);
        for color in &self.colors {
            (u32::from(color.a()) << 24
                | u32::from(color.r()) << 16
                | u32::from(color.g()) << 8
                | u32::from(color.b()))
            .hash(&mut hasher);
        }
        hasher.finish() | 1
    }

    fn fill_glyphs(&self, canvas: &Canvas, scale_factor: f32, size: (i32, i32)) {
        if self.colors.len() < 2 {
            return;
        }
        let colors: Vec<skia_safe::Color4f> = self
            .colors
            .iter()
            .map(|c| skia_safe::Color4f::from(*c))
            .collect();
        let gradient = Gradient::new(
            Colors::new(&colors, None, skia_safe::TileMode::Repeat, None),
            Interpolation::default(),
        );
        let x0 = LAYER_PAD as f32;
        let Some(shader) = linear_gradient(
            ((x0, 0.0), (x0 + self.tile * scale_factor, 0.0)),
            &gradient,
            None,
        ) else {
            return;
        };
        let mut paint = Paint::default();
        paint.set_shader(shader);
        paint.set_blend_mode(skia_safe::BlendMode::SrcIn);
        canvas.draw_rect(Rect::from_wh(size.0 as f32, size.1 as f32), &paint);
    }
}

struct TextLayer {
    key: LayerKey,
    image: skia_safe::Image,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct LayerCacheKey {
    content: u64,
    font_size: u32,
    line_height: u32,
    weight: u16,
    italic: bool,
    letter_spacing: u32,
    nowrap: bool,
    ellipsis: bool,
    line_clamp: u32,
    centered: bool,
    wrap: u32,
    underline: bool,
    color: u32,
    scale: u32,
    w: i32,
    h: i32,
    gradient: u64,
}

fn content_hash(text: &str, strong: &[(ops::Range<usize>, Weight)]) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    for (range, weight) in strong {
        range.start.hash(&mut hasher);
        range.end.hash(&mut hasher);
        (**weight as u16).hash(&mut hasher);
    }
    hasher.finish()
}

thread_local! {

    static TEXT_LAYER_CACHE: RefCell<HashMap<LayerCacheKey, skia_safe::Image>> = RefCell::new(HashMap::new());
    static TEXT_LAYER_BYTES: Cell<usize> = const { Cell::new(0) };
}

const TEXT_LAYER_BYTE_LIMIT: usize = 32 * 1024 * 1024;

fn draw_buffer(
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

#[cfg(test)]
mod text_layout_tests {
    use crate::font_system;
    use super::*;
    use crate::style::TextLayoutStyle;

    const LONG: &str = "a very long role name that will never fit in a narrow chip";

    fn widget(layout: TextLayoutStyle) -> TextWidget {
        let font_system = Rc::new(RefCell::new(font_system::make_font_system()));
        let glyph_cache = Rc::new(GlyphCache::new(1024 * 1024));
        let mut widget = TextWidget::new(LONG, 12.0, Color::WHITE, font_system, glyph_cache);
        widget.apply_text_style(&layout);
        widget
    }

    #[test]
    fn letter_spacing_widens_the_line() {
        let plain = widget(TextLayoutStyle::default());
        let spaced = widget(TextLayoutStyle::default()).with_letter_spacing(0.2);
        let extra = spaced.natural_width() - plain.natural_width();
        let expected = 0.2 * 12.0 * LONG.chars().count() as f32;
        assert!(
            (extra - expected).abs() < expected * 0.1,
            "grew by {extra}px, expected about {expected}px"
        );
    }

    #[test]
    fn nowrap_ellipsis_truncates_to_one_line_that_fits() {
        let widget = widget(TextLayoutStyle {
            nowrap: true,
            ellipsis: true,
            line_clamp: None,
        });
        let (lines, width) = widget.with_buffer(100.0, false, |b| {
            (
                b.layout_runs().count(),
                b.layout_runs().fold(0.0f32, |w, r| w.max(r.line_w)),
            )
        });
        assert_eq!(lines, 1);
        assert!(width <= 100.5, "ellipsized line is {width}px wide");
        assert!(
            width > 60.0,
            "should keep as much text as fits, got {width}px"
        );
    }

    fn measured_lines(widget: &TextWidget, width: f32) -> usize {
        let size = widget.measure(
            Size {
                width: None,
                height: None,
            },
            Size {
                width: AvailableSpace::Definite(width),
                height: AvailableSpace::MaxContent,
            },
        );
        (size.height / (widget.font_size * 1.2)).round() as usize
    }

    #[test]
    fn wrapping_depends_only_on_the_width_not_on_measure_history() {
        let widget = widget(TextLayoutStyle::default());
        let natural = widget.natural_width();
        let widths: Vec<f32> = (-30..=30).map(|i| natural - 1.0 + i as f32 * 0.1).collect();
        let ascending: Vec<usize> = widths.iter().map(|w| measured_lines(&widget, *w)).collect();
        let descending: Vec<usize> = widths
            .iter()
            .rev()
            .map(|w| measured_lines(&widget, *w))
            .collect();
        let descending: Vec<usize> = descending.into_iter().rev().collect();
        assert_eq!(ascending, descending, "widths {widths:?}");

        let fresh: Vec<usize> = widths
            .iter()
            .map(|w| measured_lines(&self::widget(TextLayoutStyle::default()), *w))
            .collect();
        assert_eq!(ascending, fresh);
    }

    #[test]
    fn paint_breaks_lines_like_measure_does_in_the_rounded_box() {
        let widget = widget(TextLayoutStyle::default());
        let natural = widget.natural_width();
        for i in -30..=30 {
            let width = natural - 1.0 + i as f32 * 0.1;
            let lines = measured_lines(&widget, width);

            let reported = widget
                .measure(
                    Size {
                        width: None,
                        height: None,
                    },
                    Size {
                        width: AvailableSpace::Definite(width),
                        height: AvailableSpace::MaxContent,
                    },
                )
                .width;
            let box_width = reported.round();
            let painted = widget.with_buffer(box_width, false, |b| b.layout_runs().count());
            assert_eq!(
                lines, painted,
                "available {width} -> measured {reported} -> box {box_width}"
            );
        }
    }

    #[test]
    fn text_that_fits_measures_the_same_at_every_wide_enough_width_without_a_buffer_per_width() {
        let widget = widget(TextLayoutStyle::default());
        let natural = widget.natural_width();
        let at = |w: f32| {
            widget.measure(
                Size {
                    width: None,
                    height: None,
                },
                Size {
                    width: AvailableSpace::Definite(w),
                    height: AvailableSpace::MaxContent,
                },
            )
        };
        let reference = at(natural + 40.0);
        for extra in [2.0, 7.3, 15.0, 40.0, 90.0, 300.0, 1000.0] {
            let size = at(natural + extra);
            assert_eq!(
                (size.width, size.height),
                (reference.width, reference.height),
                "width {extra} past natural"
            );
        }
        assert!(
            widget.cache.borrow().len() <= 1,
            "one shaped buffer serves every wide enough width, got {}",
            widget.cache.borrow().len()
        );
    }

    #[test]
    fn nowrap_measures_its_full_width() {
        let widget = widget(TextLayoutStyle {
            nowrap: true,
            ellipsis: true,
            line_clamp: None,
        });
        let size = widget.measure(
            Size {
                width: None,
                height: None,
            },
            Size {
                width: AvailableSpace::Definite(100.0),
                height: AvailableSpace::MaxContent,
            },
        );
        assert!(
            size.width > 200.0,
            "content width should be the whole line, got {}",
            size.width
        );
    }

    #[test]
    fn line_clamp_caps_the_line_count() {
        let widget = widget(TextLayoutStyle {
            nowrap: false,
            ellipsis: false,
            line_clamp: Some(2),
        });
        let lines = widget.with_buffer(80.0, false, |b| b.layout_runs().count());
        assert_eq!(lines, 2);
    }
}
