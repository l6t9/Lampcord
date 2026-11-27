use skia_safe::gradient::shaders::linear_gradient;
use skia_safe::gradient::{Colors, Gradient, Interpolation};
use skia_safe::{Canvas, Color, Color4f, Paint, PaintStyle, Point, RRect, Rect, TileMode};

use crate::style::ComputedStyle;
use crate::widget::{AbsoluteRect, Widget};

pub struct GradientRingWidget {
    start: Color,
    end: Color,

    angle: f32,
}

impl GradientRingWidget {
    pub fn new(start: Color, end: Color, angle: f32) -> Self {
        Self { start, end, angle }
    }
}

impl Widget for GradientRingWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
        let radius = style.corner_radius.min(rect.height / 2.0);
        if let Some(bg) = style.background {
            let mut paint = Paint::default();
            paint.set_color(bg);
            paint.set_anti_alias(true);
            canvas.draw_rrect(RRect::new_rect_xy(bounds, radius, radius), &paint);
        }

        let theta = self.angle.to_radians();
        let (dx, dy) = (theta.sin(), -theta.cos());
        let half = (rect.width * dx.abs() + rect.height * dy.abs()) / 2.0;
        let (cx, cy) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
        let from = Point::new(cx - dx * half, cy - dy * half);
        let to = Point::new(cx + dx * half, cy + dy * half);
        let colors = [Color4f::from(self.start), Color4f::from(self.end)];
        let gradient = Gradient::new(
            Colors::new(&colors, None, TileMode::Clamp, None),
            Interpolation::default(),
        );
        let Some(shader) = linear_gradient((from, to), &gradient, None) else {
            return;
        };
        let mut paint = Paint::default();
        paint.set_shader(shader);
        paint.set_anti_alias(true);
        paint.set_style(PaintStyle::Stroke);
        paint.set_stroke_width(1.0);
        let inset = bounds.with_inset((0.5, 0.5));
        canvas.draw_rrect(
            RRect::new_rect_xy(inset, (radius - 0.5).max(0.0), (radius - 0.5).max(0.0)),
            &paint,
        );
    }
}
