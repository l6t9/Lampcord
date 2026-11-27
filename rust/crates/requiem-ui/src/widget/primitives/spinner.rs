use crate::paint as crate_paint;
use std::time::Instant;

use skia_safe::{Canvas, Color, Paint, RRect, Rect};

use crate::style::ComputedStyle;
use crate::widget::{AbsoluteRect, Widget};

const DOT: f32 = 6.0;
const GAP: f32 = 2.0;
const PERIOD_SECS: f32 = 1.4;
const STAGGER_SECS: f32 = 0.2;

pub struct PulsingEllipsisWidget {
    color: Color,
    scale: f32,
    started: Instant,
}

impl PulsingEllipsisWidget {
    pub fn new(color: Color, scale: f32) -> Self {
        Self {
            color,
            scale,
            started: Instant::now(),
        }
    }
}

fn ease_in_out(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

impl Widget for PulsingEllipsisWidget {
    fn measure(
        &self,
        _known: taffy::Size<Option<f32>>,
        _available: taffy::Size<taffy::AvailableSpace>,
    ) -> taffy::Size<f32> {
        taffy::Size {
            width: 28.0 * self.scale,
            height: DOT * self.scale,
        }
    }

    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        let elapsed = self.started.elapsed().as_secs_f32();
        let total = (3.0 * DOT + 2.0 * GAP) * self.scale;
        let mut x = rect.x + (rect.width - total) / 2.0;
        let cy = rect.y + rect.height / 2.0;
        for i in 0..3 {
            let phase = ((elapsed - i as f32 * STAGGER_SECS) / PERIOD_SECS).rem_euclid(1.0);

            let k = ease_in_out(if phase < 0.5 {
                phase * 2.0
            } else {
                (1.0 - phase) * 2.0
            });
            let opacity = 1.0 - 0.7 * k;
            let size = DOT * self.scale * (1.0 - 0.2 * k);
            let cx = x + DOT * self.scale / 2.0;
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_color(self.color.with_a((self.color.a() as f32 * opacity) as u8));
            let dot = Rect::from_xywh(cx - size / 2.0, cy - size / 2.0, size, size);
            canvas.draw_rrect(RRect::new_rect_xy(dot, size / 2.0, size / 2.0), &paint);
            x += (DOT + GAP) * self.scale;
        }
        crate_paint::animate_region(
            canvas,
            Rect::from_xywh(rect.x, rect.y, rect.width, rect.height),
        );
    }
}
