use crate::paint as crate_paint;
use skia_safe::surfaces;
use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

use skia_safe::{Canvas, Rect, skottie};

use crate::style::ComputedStyle;
use crate::widget::{AbsoluteRect, Widget};

pub struct AnimatedIconWidget {
    animation: skottie::Animation,
    fps: f64,

    scene: Cell<(f64, f64, bool)>,

    scene_started_at: Cell<Instant>,

    last_render: RefCell<Option<(u64, i32, i32, skia_safe::Image)>>,
}

impl AnimatedIconWidget {
    pub fn new(json: &str, start_frame: f64, end_frame: f64, looping: bool) -> Self {
        let animation = skottie::Animation::from_str(json)
            .expect("animated icon JSON failed to parse as a Lottie animation");
        let fps = animation.fps() as f64;
        Self {
            animation,
            fps,
            scene: Cell::new((start_frame, end_frame, looping)),
            scene_started_at: Cell::new(Instant::now()),
            last_render: RefCell::new(None),
        }
    }

    pub fn set_scene(&self, start_frame: f64, end_frame: f64, looping: bool) {
        let next = (start_frame, end_frame, looping);
        if self.scene.get() != next {
            self.scene.set(next);
            self.scene_started_at.set(Instant::now());
        }
    }
}

impl Widget for AnimatedIconWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        let (start_frame, end_frame, looping) = self.scene.get();
        let span = (end_frame - start_frame).max(1.0 / self.fps);
        let started = self.scene_started_at.get();
        let elapsed = started.elapsed().as_secs_f64() * self.fps;
        let progress = if looping {
            elapsed % span
        } else {
            elapsed.min(span)
        };

        let frame = start_frame + progress;
        let dst = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
        if looping || elapsed < span {
            let next_at = started + Duration::from_secs_f64((elapsed.floor() + 1.0) / self.fps);
            crate_paint::animate_region_at(canvas, dst, next_at);
        }
        let matrix = canvas.local_to_device_as_3x3();
        let axis_aligned =
            matrix.skew_x() == 0.0 && matrix.skew_y() == 0.0 && !matrix.has_perspective();
        let (px_w, px_h) = (
            (rect.width * matrix.scale_x().abs()).round() as i32,
            (rect.height * matrix.scale_y().abs()).round() as i32,
        );
        if !axis_aligned || px_w <= 0 || px_h <= 0 {
            self.animation.seek_frame(frame);
            self.animation.render(canvas, dst);
            return;
        }
        let cached = self
            .last_render
            .borrow()
            .as_ref()
            .filter(|(f, w, h, _)| *f == frame.to_bits() && *w == px_w && *h == px_h)
            .map(|(_, _, _, image)| image.clone());
        let image = match cached {
            Some(image) => image,
            None => {
                self.animation.seek_frame(frame);
                let Some(mut surface) = surfaces::raster_n32_premul((px_w, px_h)) else {
                    self.animation.render(canvas, dst);
                    return;
                };
                surface.canvas().clear(skia_safe::Color::TRANSPARENT);
                self.animation
                    .render(surface.canvas(), Rect::from_iwh(px_w, px_h));
                let image = surface.image_snapshot();
                *self.last_render.borrow_mut() = Some((frame.to_bits(), px_w, px_h, image.clone()));
                image
            }
        };
        let mut paint = skia_safe::Paint::default();
        paint.set_anti_alias(true);
        canvas.draw_image_rect_with_sampling_options(
            &image,
            None,
            dst,
            skia_safe::SamplingOptions::new(
                skia_safe::FilterMode::Linear,
                skia_safe::MipmapMode::None,
            ),
            &paint,
        );
    }
}
