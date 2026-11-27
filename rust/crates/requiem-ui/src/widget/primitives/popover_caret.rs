use skia_safe::{Canvas, Paint, PaintStyle, PathBuilder, Point};

use crate::style::ComputedStyle;
use crate::widget::{AbsoluteRect, Widget};

use super::tooltip::PointerEdge;

pub struct PopoverCaretWidget {
    edge: PointerEdge,
}

impl PopoverCaretWidget {
    pub fn new(edge: PointerEdge) -> Self {
        Self { edge }
    }
}

fn p(x: f32, y: f32) -> Point {
    Point::new(x, y)
}

fn fill_path() -> skia_safe::Path {
    let mut b = PathBuilder::new();
    b.move_to(p(14.0535, 9.39127));
    b.cubic_to(
        p(12.4557, 11.2796),
        p(9.54425, 11.2796),
        p(7.94646, 9.39127),
    );
    b.line_to(p(1.0, 1.0));
    b.quad_to(p(0.0, 0.0), p(1.0, 0.0));
    b.line_to(p(21.0, 0.0));
    b.quad_to(p(22.0, 0.0), p(21.0, 1.0));
    b.line_to(p(14.0535, 9.39127));
    b.close();
    b.detach()
}

fn stroke_path() -> skia_safe::Path {
    let mut b = PathBuilder::new();
    b.move_to(p(13.6572, 9.13184));
    b.cubic_to(p(12.2604, 10.761), p(9.73957, 10.761), p(8.34277, 9.13184));
    b.line_to(p(1.0869141, 0.5));
    b.quad_to(p(0.0869141, -0.5), p(1.0869141, -0.5));
    b.line_to(p(20.9131, -0.5));
    b.quad_to(p(21.9131, -0.5), p(20.9131, 0.5));
    b.line_to(p(13.6572, 9.13184));
    b.close();
    b.detach()
}

impl Widget for PopoverCaretWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let degrees = match self.edge {
            PointerEdge::Bottom => 0.0,
            PointerEdge::Top => 180.0,
            PointerEdge::Left => 90.0,
            PointerEdge::Right => -90.0,
        };
        let (cx, cy) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
        canvas.save();
        canvas.translate((cx, cy));
        canvas.rotate(degrees, None);

        canvas.translate((-11.0, -7.0));

        canvas.clip_rect(skia_safe::Rect::from_xywh(0.0, 0.0, 22.0, 14.0), None, true);
        if let Some(color) = style.background {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.set_anti_alias(true);
            canvas.draw_path(&fill_path(), &paint);
        }
        if let Some(color) = style.border_color.filter(|_| style.border_width > 0.0) {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.set_anti_alias(true);
            paint.set_style(PaintStyle::Stroke);
            paint.set_stroke_width(1.0);
            canvas.draw_path(&stroke_path(), &paint);
        }
        canvas.restore();
    }
}
