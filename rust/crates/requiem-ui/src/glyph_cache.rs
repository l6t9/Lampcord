use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::OnceLock;

use cosmic_text::{
    CacheKey, Color as CosmicColor, FontSystem, PhysicalGlyph, Renderer, SwashCache,
};
use moka::sync::Cache;
use skia_safe::{AlphaType, Canvas, Color, ColorType, Data, ISize, ImageInfo, Paint, Rect, images};
use swash::scale::image::Content;

#[derive(Clone)]
struct RasterizedGlyphData {
    data: Vec<u8>,
    width: u32,
    height: u32,

    offset_x: i32,
    offset_y: i32,
    is_color: bool,
}

pub struct GlyphCache {
    cache: Cache<CacheKey, Option<RasterizedGlyphData>>,
    swash: RefCell<SwashCache>,

    images: RefCell<HashMap<CacheKey, Option<Rc<CachedGlyphImage>>>>,
}

struct CachedGlyphImage {
    image: skia_safe::Image,
    offset_x: i32,
    offset_y: i32,
    is_color: bool,
}

impl GlyphCache {
    pub fn new(max_capacity_bytes: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity_bytes)
            .weigher(|_key, value: &Option<RasterizedGlyphData>| {
                value.as_ref().map(|g| g.data.len() as u32).unwrap_or(1)
            })
            .build();
        Self {
            cache,
            swash: RefCell::new(SwashCache::new()),
            images: RefCell::new(HashMap::new()),
        }
    }

    pub fn stats(&self) -> (u64, u64, usize, usize) {
        let images = self.images.borrow();
        let pixels = images
            .values()
            .flatten()
            .map(|g| g.image.width() as usize * g.image.height() as usize * 4)
            .sum();
        (
            self.cache.entry_count(),
            self.cache.weighted_size(),
            images.len(),
            pixels,
        )
    }

    fn get_or_build_image(
        &self,
        font_system: &mut FontSystem,
        key: CacheKey,
    ) -> Option<Rc<CachedGlyphImage>> {
        if let Some(cached) = self.images.borrow().get(&key) {
            return cached.clone();
        }
        let image = self.get_or_rasterize(font_system, key).and_then(|glyph| {
            let image = build_skia_image(&glyph)?;
            Some(Rc::new(CachedGlyphImage {
                image,
                offset_x: glyph.offset_x,
                offset_y: glyph.offset_y,
                is_color: glyph.is_color,
            }))
        });
        self.images.borrow_mut().insert(key, image.clone());
        image
    }

    fn get_or_rasterize(
        &self,
        font_system: &mut FontSystem,
        key: CacheKey,
    ) -> Option<RasterizedGlyphData> {
        if let Some(cached) = self.cache.get(&key) {
            return cached;
        }
        let rasterized = self
            .swash
            .borrow_mut()
            .get_image_uncached(font_system, key)
            .and_then(|image| {
                if image.placement.width == 0 || image.placement.height == 0 {
                    return None;
                }
                let is_color = match image.content {
                    Content::Mask => false,
                    Content::Color => true,
                    Content::SubpixelMask => return None,
                };
                let mut data = image.data;
                if !is_color {
                    for byte in &mut data {
                        *byte = gamma_correct_coverage(*byte);
                    }
                }
                Some(RasterizedGlyphData {
                    data,
                    width: image.placement.width,
                    height: image.placement.height,
                    offset_x: image.placement.left,
                    offset_y: -image.placement.top,
                    is_color,
                })
            });
        self.cache.insert(key, rasterized.clone());
        rasterized
    }
}

fn gamma_correct_coverage(coverage: u8) -> u8 {
    const GAMMA: f32 = 0.72;
    static LUT: OnceLock<[u8; 256]> = OnceLock::new();
    let lut = LUT.get_or_init(|| {
        let mut table = [0u8; 256];
        for (i, entry) in table.iter_mut().enumerate() {
            *entry = (255.0 * (i as f32 / 255.0).powf(GAMMA)).round() as u8;
        }
        table
    });
    lut[coverage as usize]
}

fn build_skia_image(glyph: &RasterizedGlyphData) -> Option<skia_safe::Image> {
    if glyph.is_color {
        let info = ImageInfo::new(
            ISize::new(glyph.width as i32, glyph.height as i32),
            ColorType::RGBA8888,
            AlphaType::Premul,
            None,
        );
        images::raster_from_data(&info, Data::new_copy(&glyph.data), glyph.width as usize * 4)
    } else {
        let info = ImageInfo::new(
            ISize::new(glyph.width as i32, glyph.height as i32),
            ColorType::Alpha8,
            AlphaType::Premul,
            None,
        );
        images::raster_from_data(&info, Data::new_copy(&glyph.data), glyph.width as usize)
    }
}

fn to_skia_color(color: CosmicColor) -> Color {
    let (r, g, b, a) = color.as_rgba_tuple();
    Color::from_argb(a, r, g, b)
}

pub fn paint_glyph(
    canvas: &Canvas,
    glyph_cache: &GlyphCache,
    font_system: &mut FontSystem,
    physical_glyph: PhysicalGlyph,
    color: CosmicColor,
    origin: (f32, f32),
) {
    let Some(glyph) = glyph_cache.get_or_build_image(font_system, physical_glyph.cache_key) else {
        return;
    };

    let pos = (
        (origin.0 + (physical_glyph.x + glyph.offset_x) as f32).round(),
        (origin.1 + (physical_glyph.y + glyph.offset_y) as f32).round(),
    );
    if glyph.is_color {
        canvas.draw_image(&glyph.image, pos, None);
    } else {
        let mut paint = Paint::default();
        paint.set_color(to_skia_color(color));
        paint.set_anti_alias(true);
        canvas.draw_image(&glyph.image, pos, Some(&paint));
    }
}

pub struct SkiaDecorationRenderer<'a> {
    pub canvas: &'a Canvas,

    pub origin: (f32, f32),

    pub scale: f32,
}

impl Renderer for SkiaDecorationRenderer<'_> {
    fn rectangle(&mut self, x: i32, y: i32, w: u32, h: u32, color: CosmicColor) {
        let mut paint = Paint::default();
        paint.set_color(to_skia_color(color));
        let rect = Rect::from_xywh(
            self.origin.0 + x as f32 * self.scale,
            self.origin.1 + y as f32 * self.scale,
            w as f32 * self.scale,
            h as f32 * self.scale,
        );
        self.canvas.draw_rect(rect, &paint);
    }

    fn glyph(&mut self, _physical_glyph: PhysicalGlyph, _color: CosmicColor) {
        unreachable!(
            "render_decoration only calls Renderer::rectangle, never Renderer::glyph — see this struct's own doc comment"
        );
    }
}
