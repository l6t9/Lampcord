use crate::icon_handle::Icon;
use skia_safe::{Canvas, Matrix, Paint, PaintStyle, Path, PathBuilder, Point, RRect, Rect};

use crate::style::ComputedStyle;
use crate::widget::{AbsoluteRect, Widget};

pub struct BoxWidget;

pub fn squircle_path(bounds: Rect) -> Path {
    let path = Path::from_svg(Icon::Named("shape/squircle_mask").path())
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| {
            Path::rect(Rect::from_wh(40.0, 40.0), None).with_transform(&Matrix::default())
        });
    let mut matrix = Matrix::scale((bounds.width() / 40.0, bounds.height() / 40.0));
    matrix.post_translate((bounds.left, bounds.top));
    path.with_transform(&matrix)
}

pub fn corner_radii(style: &ComputedStyle) -> [Point; 4] {
    let tl = style.corner_radius_top_left.unwrap_or(style.corner_radius);
    let tr = style.corner_radius_top_right.unwrap_or(style.corner_radius);
    let br = style
        .corner_radius_bottom_right
        .unwrap_or(style.corner_radius);
    let bl = style
        .corner_radius_bottom_left
        .unwrap_or(style.corner_radius);
    [
        Point::new(tl, tl),
        Point::new(tr, tr),
        Point::new(br, br),
        Point::new(bl, bl),
    ]
}

pub fn has_any_corner_radius(style: &ComputedStyle) -> bool {
    style.corner_radius > 0.0
        || style.corner_radius_top_left.is_some()
        || style.corner_radius_top_right.is_some()
        || style.corner_radius_bottom_right.is_some()
        || style.corner_radius_bottom_left.is_some()
}

pub struct HoverBoxWidget {
    on_change: Box<dyn FnMut(bool)>,
    focus: Option<bool>,
}

impl HoverBoxWidget {
    pub fn new(on_change: impl FnMut(bool) + 'static) -> Self {
        Self {
            on_change: Box::new(on_change),
            focus: None,
        }
    }

    pub fn focusable(mut self, tab_stop: bool) -> Self {
        self.focus = Some(tab_stop);
        self
    }
}

impl Widget for HoverBoxWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        BoxWidget.paint(canvas, rect, style);
    }

    fn hover_path_changed(&mut self, inside: bool) {
        (self.on_change)(inside);
    }

    fn wants_focus(&self) -> bool {
        self.focus.is_some()
    }

    fn in_tab_order(&self) -> bool {
        self.focus == Some(true)
    }
}

impl Widget for BoxWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);

        if let Some(color) = style.background {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.set_anti_alias(true);
            if style.squircle {
                canvas.draw_path(&squircle_path(bounds), &paint);
            } else if style.corner_ellipse {
                canvas.draw_oval(bounds, &paint);
            } else if has_any_corner_radius(style) {
                let radii = corner_radii(style);
                canvas.draw_rrect(RRect::new_rect_radii(bounds, &radii), &paint);
            } else {
                canvas.draw_rect(bounds, &paint);
            }
        }

        paint_borders(canvas, bounds, style);
    }
}

pub(crate) fn paint_borders(canvas: &Canvas, bounds: Rect, style: &ComputedStyle) {
    if let (Some(color), width) = (style.border_color, style.border_width) {
        if width > 0.0 {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.set_anti_alias(true);
            paint.set_style(PaintStyle::Stroke);
            paint.set_stroke_width(width);
            if style.border_dashed {
                paint.set_path_effect(skia_safe::PathEffect::dash(
                    &[width * 2.0, width * 2.0],
                    0.0,
                ));
            }
            let inset = bounds.with_inset((width / 2.0, width / 2.0));
            if style.squircle {
                canvas.draw_path(&squircle_path(inset), &paint);
            } else if style.corner_radius > 0.0 {
                let radius = (style.corner_radius - width / 2.0).max(0.0);
                canvas.draw_rrect(RRect::new_rect_xy(inset, radius, radius), &paint);
            } else {
                canvas.draw_rect(inset, &paint);
            }
        }
    }

    if let (Some(color), width) = (style.border_bottom_color, style.border_bottom_width) {
        if width > 0.0 {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.set_anti_alias(true);
            paint.set_style(PaintStyle::Stroke);
            paint.set_stroke_width(width);
            let y = bounds.bottom - width / 2.0;
            // A rounded bottom-left corner is drawn by the left border.
            let bl = style.corner_radius_bottom_left.unwrap_or(0.0);
            let has_left = style.border_left_color.is_some() && style.border_left_width > 0.0;
            let x_start = if has_left && bl > width / 2.0 {
                bounds.left + bl
            } else {
                bounds.left
            };
            canvas.draw_line((x_start, y), (bounds.right, y), &paint);
        }
    }

    if let (Some(color), width) = (style.border_top_color, style.border_top_width) {
        if width > 0.0 {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.set_anti_alias(true);
            paint.set_style(PaintStyle::Stroke);
            paint.set_stroke_width(width);
            let y = bounds.top + width / 2.0;
            let tl = style.corner_radius_top_left.unwrap_or(0.0);
            let x_start = if tl > width / 2.0 {
                bounds.left + tl
            } else {
                bounds.left
            };
            canvas.draw_line((x_start, y), (bounds.right, y), &paint);
        }
    }

    if let (Some(color), width) = (style.border_left_color, style.border_left_width) {
        if width > 0.0 {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.set_anti_alias(true);
            paint.set_style(PaintStyle::Stroke);
            paint.set_stroke_width(width);
            let inset = width / 2.0;
            let tl = style.corner_radius_top_left.unwrap_or(0.0);
            let bl = style.corner_radius_bottom_left.unwrap_or(0.0);
            let has_bottom = style.border_bottom_color.is_some() && style.border_bottom_width > 0.0;
            let mut builder = PathBuilder::new();
            if tl > inset {
                let r = tl - inset;
                builder.move_to((bounds.left + tl, bounds.top + inset));
                builder.arc_to(
                    Rect::from_xywh(bounds.left + inset, bounds.top + inset, r * 2.0, r * 2.0),
                    270.0,
                    -90.0,
                    false,
                );
            } else {
                builder.move_to((bounds.left + inset, bounds.top));
            }
            if has_bottom && bl > inset {
                let r = bl - inset;
                builder.line_to((bounds.left + inset, bounds.bottom - bl));
                builder.arc_to(
                    Rect::from_xywh(
                        bounds.left + inset,
                        bounds.bottom - inset - r * 2.0,
                        r * 2.0,
                        r * 2.0,
                    ),
                    180.0,
                    -90.0,
                    false,
                );
            } else {
                builder.line_to((bounds.left + inset, bounds.bottom));
            }
            canvas.draw_path(&builder.detach(), &paint);
        }
    }

    if let (Some(color), width) = (style.border_right_color, style.border_right_width) {
        if width > 0.0 {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.set_anti_alias(true);
            paint.set_style(PaintStyle::Stroke);
            paint.set_stroke_width(width);
            let x = bounds.right - width / 2.0;
            canvas.draw_line((x, bounds.top), (x, bounds.bottom), &paint);
        }
    }
}
