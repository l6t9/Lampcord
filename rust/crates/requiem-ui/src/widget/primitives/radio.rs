use crate::colors::{BACKGROUND_MOD_SUBTLE, BRAND};
use skia_safe::{Canvas, Color, Paint, PaintStyle};

use crate::style::ComputedStyle;
use crate::widget::anim::{ToggleState, keyframe_eased, lerp_color};
use crate::widget::{AbsoluteRect, Widget};

pub struct RadioWidget {
    state: ToggleState,
}

const UNSELECTED_BG: Color = Color::from_argb(0x14, 0x00, 0x00, 0x00);
const UNSELECTED_BORDER: Color = Color::from_argb(0xa3, 0x97, 0x97, 0x9f);
const DOT_COLOR: Color = Color::WHITE;
const BORDER_WIDTH: f32 = 2.0;

const DOT_RADIUS_FRACTION: f32 = 0.4;

const FILL_IN_DURATION_SECS: f32 = 0.225;
const FILL_OUT_DURATION_SECS: f32 = 0.16;
const FILL_BEZIER: (f32, f32, f32, f32) = (0.33, 0.0, 0.67, 1.0);
const FILL_IN_OPACITY_BREAKS: [f32; 3] = [0.0, 0.01, 1.0];
const FILL_IN_OPACITY: [f32; 3] = [0.0, 1.0, 1.0];
const FILL_IN_SCALE_BREAKS: [f32; 4] = [0.0, 0.35, 0.40, 1.0];
const FILL_IN_SCALE: [f32; 4] = [0.0, 0.33, 1.15, 1.0];
const FILL_OUT_BREAKS: [f32; 3] = [0.0, 0.99, 1.0];
const FILL_OUT_OPACITY: [f32; 3] = [1.0, 1.0, 0.0];
const FILL_OUT_SCALE: [f32; 3] = [1.0, 1.0, 0.0];

const DOT_IN_DURATION_SECS: f32 = 0.333;
const DOT_OUT_DURATION_SECS: f32 = 0.25;
const DOT_IN_BEZIER: (f32, f32, f32, f32) = (0.26, 0.21, 0.67, 1.0);
const DOT_OUT_BEZIER: (f32, f32, f32, f32) = (0.33, 0.0, 0.25, 1.0);
const DOT_IN_SCALE_BREAKS: [f32; 4] = [0.0, 0.40, 0.75, 1.0];
const DOT_IN_SCALE: [f32; 4] = [0.0, 1.2, 1.0, 1.0];
const DOT_OUT_BREAKS: [f32; 3] = [0.0, 0.99, 1.0];
const DOT_OUT_SCALE: [f32; 3] = [1.0, 1.0, 0.0];
const DOT_OUT_OPACITY: [f32; 3] = [1.0, 1.0, 0.0];

impl RadioWidget {
    pub fn new(state: ToggleState) -> Self {
        Self { state }
    }
}

impl Widget for RadioWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        let elapsed = self.state.changed_at.elapsed().as_secs_f32();

        let cx = rect.x + rect.width / 2.0;
        let cy = rect.y + rect.height / 2.0;
        let radius = rect.width.min(rect.height) / 2.0;

        let border_t = (elapsed
            / if self.state.on {
                FILL_IN_DURATION_SECS
            } else {
                FILL_OUT_DURATION_SECS
            })
        .min(1.0);
        let (border_from, border_to) = if self.state.on {
            (UNSELECTED_BORDER, BACKGROUND_MOD_SUBTLE)
        } else {
            (BACKGROUND_MOD_SUBTLE, UNSELECTED_BORDER)
        };
        let mut border_paint = Paint::default();
        border_paint.set_color(lerp_color(border_from, border_to, border_t));
        border_paint.set_anti_alias(true);
        border_paint.set_style(PaintStyle::Stroke);
        border_paint.set_stroke_width(BORDER_WIDTH);
        canvas.draw_circle((cx, cy), radius - BORDER_WIDTH / 2.0, &border_paint);

        let mut base_paint = Paint::default();
        base_paint.set_color(UNSELECTED_BG);
        base_paint.set_anti_alias(true);
        canvas.draw_circle((cx, cy), radius, &base_paint);

        let (fill_opacity, fill_scale) = if self.state.on {
            let t = (elapsed / FILL_IN_DURATION_SECS).min(1.0);
            let (x1, y1, x2, y2) = FILL_BEZIER;
            (
                keyframe_eased(t, &FILL_IN_OPACITY_BREAKS, &FILL_IN_OPACITY, x1, y1, x2, y2),
                keyframe_eased(t, &FILL_IN_SCALE_BREAKS, &FILL_IN_SCALE, x1, y1, x2, y2),
            )
        } else {
            let t = (elapsed / FILL_OUT_DURATION_SECS).min(1.0);
            let (x1, y1, x2, y2) = FILL_BEZIER;
            (
                keyframe_eased(t, &FILL_OUT_BREAKS, &FILL_OUT_OPACITY, x1, y1, x2, y2),
                keyframe_eased(t, &FILL_OUT_BREAKS, &FILL_OUT_SCALE, x1, y1, x2, y2),
            )
        };
        if fill_opacity > 0.0 && fill_scale > 0.0 {
            let mut fill_paint = Paint::default();
            fill_paint.set_color(BRAND);
            fill_paint.set_alpha_f(fill_opacity);
            fill_paint.set_anti_alias(true);
            canvas.draw_circle((cx, cy), radius * fill_scale, &fill_paint);
        }

        let (dot_opacity, dot_scale) = if self.state.on {
            let t = (elapsed / DOT_IN_DURATION_SECS).min(1.0);
            let (x1, y1, x2, y2) = DOT_IN_BEZIER;
            (
                1.0,
                keyframe_eased(t, &DOT_IN_SCALE_BREAKS, &DOT_IN_SCALE, x1, y1, x2, y2),
            )
        } else {
            let t = (elapsed / DOT_OUT_DURATION_SECS).min(1.0);
            let (x1, y1, x2, y2) = DOT_OUT_BEZIER;
            (
                keyframe_eased(t, &DOT_OUT_BREAKS, &DOT_OUT_OPACITY, x1, y1, x2, y2),
                keyframe_eased(t, &DOT_OUT_BREAKS, &DOT_OUT_SCALE, x1, y1, x2, y2),
            )
        };
        if dot_opacity > 0.0 && dot_scale > 0.0 {
            let mut dot_paint = Paint::default();
            dot_paint.set_color(DOT_COLOR);
            dot_paint.set_alpha_f(dot_opacity);
            dot_paint.set_anti_alias(true);
            canvas.draw_circle(
                (cx, cy),
                radius * DOT_RADIUS_FRACTION * dot_scale,
                &dot_paint,
            );
        }
    }
}
