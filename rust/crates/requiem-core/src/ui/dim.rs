use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::primitives::{StatusCutout, corner_radii, has_any_corner_radius};
use requiem_ui::widget::{AbsoluteRect, Widget};
use skia_safe::{Canvas, ClipOp, Color, Paint, Path, RRect, Rect};

pub struct DimWidget {
    pub cutout: Option<StatusCutout>,
}

impl Widget for DimWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(Color::from_argb(102, 0, 0, 0));
        canvas.save();
        if let Some(cutout) = self.cutout {
            let hole = Path::circle(
                (
                    rect.x + cutout.cx * rect.width,
                    rect.y + cutout.cy * rect.height,
                ),
                cutout.r * rect.width,
                None,
            );
            canvas.clip_path(&hole, ClipOp::Difference, true);
        }
        if has_any_corner_radius(style) {
            canvas.draw_rrect(RRect::new_rect_radii(bounds, &corner_radii(style)), &paint);
        } else {
            canvas.draw_rect(bounds, &paint);
        }
        canvas.restore();
    }
}
