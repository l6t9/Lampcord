use skia_safe::{Canvas, Paint, PaintStyle, RRect, Rect};

use crate::style::ComputedStyle;
use crate::widget::AbsoluteRect;

pub(super) fn paint_background(canvas: &Canvas, bounds: Rect, style: &ComputedStyle) {
    let Some(color) = style.background else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.set_anti_alias(true);
    if style.corner_radius > 0.0 {
        canvas.draw_rrect(
            RRect::new_rect_xy(bounds, style.corner_radius, style.corner_radius),
            &paint,
        );
    } else {
        canvas.draw_rect(bounds, &paint);
    }
}

pub(super) fn paint_border(canvas: &Canvas, bounds: Rect, style: &ComputedStyle) {
    let (Some(color), width) = (style.border_color, style.border_width) else {
        return;
    };
    if width <= 0.0 {
        return;
    }
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.set_anti_alias(true);
    paint.set_style(PaintStyle::Stroke);
    paint.set_stroke_width(width);
    let inset = bounds.with_inset((width / 2.0, width / 2.0));
    if style.corner_radius > 0.0 {
        let radius = (style.corner_radius - width / 2.0).max(0.0);
        canvas.draw_rrect(RRect::new_rect_xy(inset, radius, radius), &paint);
    } else {
        canvas.draw_rect(inset, &paint);
    }
}

pub(super) fn paint_box(canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
    let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
    paint_background(canvas, bounds, style);
    paint_border(canvas, bounds, style);
}
