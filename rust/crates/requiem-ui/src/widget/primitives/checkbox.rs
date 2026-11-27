use crate::colors::{BACKGROUND_MOD_SUBTLE, BRAND};
use crate::icon_handle::IconPath;
use crate::icons;
use crate::svg_icon;
use skia_safe::{Canvas, Color, Paint, PaintStyle, RRect, Rect};
use std::rc;

use crate::style::ComputedStyle;
use crate::widget::anim::{ToggleState, keyframe_eased, lerp_color};
use crate::widget::{AbsoluteRect, Widget};

pub struct CheckboxWidget {
    state: ToggleState,
    check: Option<rc::Rc<svg_icon::ParsedSvg>>,
}

const UNCHECKED_BG: Color = Color::from_argb(0x14, 0x00, 0x00, 0x00);
const UNCHECKED_BORDER: Color = Color::from_argb(0xa3, 0x97, 0x97, 0x9f);
const CHECK_COLOR: Color = Color::WHITE;
const RADIUS: f32 = 4.0;

const CHECK_DURATION_SECS: f32 = 0.3;
const UNCHECK_DURATION_SECS: f32 = 0.12;
const CHECK_BEZIER: (f32, f32, f32, f32) = (0.65, 0.0, 0.83, 0.83);
const UNCHECK_BEZIER: (f32, f32, f32, f32) = (0.65, 0.0, 0.83, 0.84);

const CHECK_OPACITY_BREAKS: [f32; 4] = [0.0, 0.25, 0.26, 1.0];
const CHECK_IN_OPACITY: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const CHECK_OUT_OPACITY: [f32; 4] = [1.0, 1.0, 0.0, 0.0];
const CHECK_IN_SCALE_BREAKS: [f32; 5] = [0.0, 0.25, 0.26, 0.75, 1.0];
const CHECK_IN_SCALE: [f32; 5] = [0.7, 0.7, 0.7, 1.1, 1.0];
const CHECK_OUT_SCALE_BREAKS: [f32; 4] = [0.0, 0.25, 0.26, 1.0];
const CHECK_OUT_SCALE: [f32; 4] = [1.0, 0.7, 0.7, 0.7];

impl CheckboxWidget {
    pub fn new(state: ToggleState) -> Self {
        Self {
            state,
            check: IconPath::art(&icons::CHECKMARK_SMALL_BOLD),
        }
    }
}

impl Widget for CheckboxWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        let elapsed = self.state.changed_at.elapsed().as_secs_f32();
        let duration = if self.state.on {
            CHECK_DURATION_SECS
        } else {
            UNCHECK_DURATION_SECS
        };
        let (x1, y1, x2, y2) = if self.state.on {
            CHECK_BEZIER
        } else {
            UNCHECK_BEZIER
        };
        let t = (elapsed / duration).min(1.0);

        let (bg_from, bg_to, border_from, border_to) = if self.state.on {
            (UNCHECKED_BG, BRAND, UNCHECKED_BORDER, BACKGROUND_MOD_SUBTLE)
        } else {
            (BRAND, UNCHECKED_BG, BACKGROUND_MOD_SUBTLE, UNCHECKED_BORDER)
        };

        let color_t = t;

        let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
        let rrect = RRect::new_rect_xy(bounds, RADIUS, RADIUS);

        let mut bg_paint = Paint::default();
        bg_paint.set_color(lerp_color(bg_from, bg_to, color_t));
        bg_paint.set_anti_alias(true);
        canvas.draw_rrect(rrect, &bg_paint);

        let mut border_paint = Paint::default();
        border_paint.set_color(lerp_color(border_from, border_to, color_t));
        border_paint.set_anti_alias(true);
        border_paint.set_style(PaintStyle::Stroke);
        border_paint.set_stroke_width(1.0);
        canvas.draw_rrect(
            RRect::new_rect_xy(bounds.with_inset((0.5, 0.5)), RADIUS - 0.5, RADIUS - 0.5),
            &border_paint,
        );

        let (opacity, scale) = if self.state.on {
            (
                keyframe_eased(t, &CHECK_OPACITY_BREAKS, &CHECK_IN_OPACITY, x1, y1, x2, y2),
                keyframe_eased(t, &CHECK_IN_SCALE_BREAKS, &CHECK_IN_SCALE, x1, y1, x2, y2),
            )
        } else {
            (
                keyframe_eased(t, &CHECK_OPACITY_BREAKS, &CHECK_OUT_OPACITY, x1, y1, x2, y2),
                keyframe_eased(t, &CHECK_OUT_SCALE_BREAKS, &CHECK_OUT_SCALE, x1, y1, x2, y2),
            )
        };

        if opacity > 0.0 {
            let inset = rect.width * 0.15;
            let Some(check) = &self.check else { return };
            let base_scale = (rect.width - inset * 2.0) / check.width;
            let cx = rect.x + rect.width / 2.0;
            let cy = rect.y + rect.height / 2.0;
            canvas.save();
            canvas.translate((cx, cy));
            canvas.scale((base_scale * scale, base_scale * scale));
            canvas.translate((-check.width / 2.0, -check.height / 2.0));
            check.draw(canvas, CHECK_COLOR, opacity);
            canvas.restore();
        }
    }
}
