use crate::app;
use crate::perf;
use skia_safe::canvas::SrcRectConstraint;
use skia_safe::surfaces;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Instant;

use skia_safe::{
    Canvas, CubicResampler, FilterMode, IRect, Image, MipmapMode, Paint, RRect, Rect,
    SamplingOptions,
};

use super::container::{corner_radii, has_any_corner_radius};
use crate::style::ComputedStyle;
use crate::widget::{AbsoluteRect, Widget};

pub struct ImageWidget {
    image: Option<Image>,

    src_rect: Option<(f32, f32, f32, f32)>,

    intrinsic_size: Option<(f32, f32)>,

    cover: bool,

    cover_centered: bool,
}

impl ImageWidget {
    pub fn new(image: Option<Image>) -> Self {
        Self {
            image,
            src_rect: None,
            intrinsic_size: None,
            cover: false,
            cover_centered: false,
        }
    }

    pub fn with_src_rect(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        self.src_rect = Some((x, y, width, height));
        self
    }

    pub fn with_cover(mut self) -> Self {
        self.cover = true;
        self
    }

    pub fn with_cover_centered(mut self) -> Self {
        self.cover = true;
        self.cover_centered = true;
        self
    }

    pub fn with_intrinsic_size(mut self, width: f32, height: f32) -> Self {
        self.intrinsic_size = Some((width, height));
        self
    }

    pub fn set_image(&mut self, image: Image) {
        self.image = Some(image);
    }
}

impl Widget for ImageWidget {
    fn debug_props(&self) -> Vec<(&'static str, String)> {
        let loaded = self.image.as_ref().map_or("none".to_string(), |i| {
            format!("{}x{}", i.width(), i.height())
        });
        vec![
            ("image", loaded),
            ("intrinsic_size", format!("{:?}", self.intrinsic_size)),
        ]
    }

    fn measure(
        &self,
        _known_dimensions: taffy::Size<Option<f32>>,
        _available_space: taffy::Size<taffy::AvailableSpace>,
    ) -> taffy::Size<f32> {
        match self.intrinsic_size {
            Some((width, height)) => taffy::Size { width, height },
            None => taffy::Size::ZERO,
        }
    }

    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let Some(image) = &self.image else {
            return;
        };

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        let dst = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
        let src = self
            .src_rect
            .map(|(x, y, w, h)| Rect::from_xywh(x, y, w, h))
            .or_else(|| {
                if !self.cover || rect.width <= 0.0 || rect.height <= 0.0 {
                    return None;
                }
                let (iw, ih) = (image.width() as f32, image.height() as f32);
                let scale = (rect.width / iw).max(rect.height / ih);
                let (sw, sh) = ((rect.width / scale).min(iw), (rect.height / scale).min(ih));
                let (sx, sy) = if self.cover_centered {
                    ((iw - sw) / 2.0, (ih - sh) / 2.0)
                } else {
                    (0.0, 0.0)
                };
                Some(Rect::from_xywh(sx, sy, sw, sh))
            });

        if has_any_corner_radius(style) {
            let radii = corner_radii(style);
            canvas.save();
            canvas.clip_rrect(RRect::new_rect_radii(dst, &radii), None, true);
            draw_image_filtered(canvas, image, src, dst, &paint);
            canvas.restore();
        } else {
            draw_image_filtered(canvas, image, src, dst, &paint);
        }
    }
}

thread_local! {

    static SCALED_CACHE: RefCell<HashMap<(u32, [i32; 4], i32, i32), Option<Image>>> = RefCell::new(HashMap::new());

    static SCALED_BYTES: Cell<usize> = const { Cell::new(0) };
}

const SCALED_CACHE_BYTE_LIMIT: usize = 64 * 1024 * 1024;

const RESIZE_SIZE_STEP: i32 = 128;

pub fn scaled_cache_stats() -> (usize, usize) {
    SCALED_CACHE.with(|cache| {
        cache
            .borrow()
            .values()
            .flatten()
            .fold((0, 0), |(n, b), img| {
                (n + 1, b + img.width() as usize * img.height() as usize * 4)
            })
    })
}

const SCALED_CACHE_LIMIT: usize = 2048;

pub fn sampling_for(canvas: &Canvas, src_w: f32, dst_w: f32) -> SamplingOptions {
    let device_w = dst_w * canvas.local_to_device_as_3x3().scale_x().abs();
    if device_w < src_w * 0.5 {
        SamplingOptions::new(FilterMode::Linear, MipmapMode::Linear)
    } else {
        SamplingOptions::from(CubicResampler::catmull_rom())
    }
}

pub fn draw_image_filtered(
    canvas: &Canvas,
    image: &Image,
    src: Option<Rect>,
    dst: Rect,
    paint: &Paint,
) {
    let src = src.unwrap_or_else(|| Rect::from_iwh(image.width(), image.height()));
    let matrix = canvas.local_to_device_as_3x3();
    let axis_aligned =
        matrix.skew_x() == 0.0 && matrix.skew_y() == 0.0 && !matrix.has_perspective();
    let (mut target_w, mut target_h) = (
        (dst.width() * matrix.scale_x().abs()).round() as i32,
        (dst.height() * matrix.scale_y().abs()).round() as i32,
    );
    if target_w <= 0 || target_h <= 0 || src.is_empty() {
        return;
    }
    if app::live_resizing() && axis_aligned {
        let ratio = target_h as f32 / target_w as f32;
        let snapped = ((target_w + RESIZE_SIZE_STEP - 1) / RESIZE_SIZE_STEP * RESIZE_SIZE_STEP)
            .min(src.width().round() as i32)
            .max(1);
        target_w = snapped;
        target_h = ((snapped as f32 * ratio).round() as i32).max(1);
    }
    let shrinking =
        (target_w as f32) < src.width() * 0.75 || (target_h as f32) < src.height() * 0.75;
    if !axis_aligned || !shrinking {
        let sampling = SamplingOptions::from(CubicResampler::catmull_rom());
        canvas.draw_image_rect_with_sampling_options(
            image,
            Some((&src, SrcRectConstraint::Strict)),
            dst,
            sampling,
            paint,
        );
        return;
    }
    let src_px = src.round();
    let key = (
        image.unique_id(),
        [src_px.left, src_px.top, src_px.right, src_px.bottom],
        target_w,
        target_h,
    );
    let scaled = SCALED_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.len() > SCALED_CACHE_LIMIT
            || SCALED_BYTES.with(Cell::get) > SCALED_CACHE_BYTE_LIMIT
        {
            if perf::profile_paint() {
                perf::record_widget_paint(
                    format!(
                        "IMAGE-CACHE-CLEARED at {} entries {}MB",
                        cache.len(),
                        SCALED_BYTES.with(Cell::get) / 1_000_000
                    ),
                    0.0,
                );
            }
            cache.clear();
            SCALED_BYTES.with(|b| b.set(0));
        }
        cache
            .entry(key)
            .or_insert_with(|| {
                let started = Instant::now();
                let made = downsample(image, src_px, target_w, target_h);
                if perf::profile_paint() {
                    perf::record_widget_paint(
                        format!(
                            "IMAGE-DOWNSAMPLE {}x{}->{}x{}",
                            image.width(),
                            image.height(),
                            target_w,
                            target_h
                        ),
                        started.elapsed().as_secs_f64() * 1000.0,
                    );
                }
                if let Some(made) = &made {
                    SCALED_BYTES.with(|b| {
                        b.set(b.get() + made.width() as usize * made.height() as usize * 4)
                    });
                }
                made
            })
            .clone()
    });
    if let Some(scaled) = scaled {
        canvas.draw_image_rect_with_sampling_options(
            &scaled,
            None,
            dst,
            SamplingOptions::new(FilterMode::Linear, MipmapMode::None),
            paint,
        );
    }
}

pub fn draw_atlas_tile(canvas: &Canvas, atlas: &Image, src: Rect, dst: Rect, paint: &Paint) {
    draw_image_filtered(canvas, atlas, Some(src), dst, paint);
}

fn downsample(image: &Image, src: IRect, target_w: i32, target_h: i32) -> Option<Image> {
    let whole =
        src.left <= 0 && src.top <= 0 && src.right >= image.width() && src.bottom >= image.height();
    let mut scaled = if whole {
        image.clone()
    } else {
        resample(
            image,
            Some(Rect::from(src)),
            src.width(),
            src.height(),
            SamplingOptions::new(FilterMode::Nearest, MipmapMode::None),
        )?
    };
    while scaled.width() >= target_w * 2 && scaled.height() >= target_h * 2 {
        let (w, h) = (scaled.width() / 2, scaled.height() / 2);
        scaled = resample(
            &scaled,
            None,
            w,
            h,
            SamplingOptions::new(FilterMode::Linear, MipmapMode::None),
        )?;
    }
    resample(
        &scaled,
        None,
        target_w,
        target_h,
        SamplingOptions::from(CubicResampler::mitchell()),
    )
}

fn resample(
    image: &Image,
    src: Option<Rect>,
    width: i32,
    height: i32,
    sampling: SamplingOptions,
) -> Option<Image> {
    let mut surface = surfaces::raster_n32_premul((width, height))?;
    let canvas = surface.canvas();
    canvas.clear(skia_safe::Color::TRANSPARENT);
    let constraint = src
        .as_ref()
        .map(|r| (r, SrcRectConstraint::Strict));
    canvas.draw_image_rect_with_sampling_options(
        image,
        constraint,
        Rect::from_iwh(width, height),
        sampling,
        &Paint::default(),
    );
    Some(surface.image_snapshot())
}
