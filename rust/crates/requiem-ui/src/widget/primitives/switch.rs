use crate::colors::{BACKGROUND_MOD_SUBTLE, BRAND};
use crate::icon_handle::Icon;
use skia_safe::{Canvas, Color, Paint, PaintStyle, RRect, Rect};

use crate::style::ComputedStyle;
use crate::widget::anim::{ToggleState, keyframe, lerp_color};
use crate::widget::{AbsoluteRect, Widget};

pub struct SwitchWidget {
    state: ToggleState,

    icons: bool,
}

const THUMB_X: [Icon; 2] = [
    Icon::Named("shape/switch_thumb_x_a"),
    Icon::Named("shape/switch_thumb_x_b"),
];
const THUMB_CHECK: [Icon; 2] = [
    Icon::Named("shape/switch_thumb_check_a"),
    Icon::Named("shape/switch_thumb_check_b"),
];

const THUMB_ICON_OFF: Color = Color::from_argb(255, 0x2f, 0x30, 0x35);

const OFF_BG: Color = Color::from_argb(0x1f, 0x00, 0x00, 0x00);
const OFF_BORDER: Color = Color::from_argb(0x33, 0x97, 0x97, 0x9f);
const THUMB: Color = Color::WHITE;

const DURATION_SECS: f32 = 0.3;

const S_BREAKS: [f32; 6] = [0.0, 0.1, 0.4, 0.6, 0.9, 1.0];

const N_BREAKS: [f32; 4] = [0.0, 0.1, 0.9, 1.0];

const WIDTH_KEYFRAMES: [f32; 6] = [16.0, 16.0, 19.0, 22.0, 19.0, 16.0];
const HEIGHT_KEYFRAMES: [f32; 6] = [16.0, 10.0, 19.0, 10.0, 19.0, 16.0];

const REFERENCE_BOX: f32 = 24.0;

const COLOR_RAMP: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

impl SwitchWidget {
    pub fn new(state: ToggleState) -> Self {
        Self {
            state,
            icons: false,
        }
    }

    pub fn with_icons(mut self) -> Self {
        self.icons = true;
        self
    }
}

impl Widget for SwitchWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        let t = (self.state.changed_at.elapsed().as_secs_f32() / DURATION_SECS).min(1.0);

        let (bg_from, bg_to, border_from, border_to) = if self.state.on {
            (OFF_BG, BRAND, OFF_BORDER, BACKGROUND_MOD_SUBTLE)
        } else {
            (BRAND, OFF_BG, BACKGROUND_MOD_SUBTLE, OFF_BORDER)
        };
        let color_t = keyframe(t, &N_BREAKS, &COLOR_RAMP);

        let track_bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
        let track_radius = rect.height / 2.0;

        let mut bg_paint = Paint::default();
        bg_paint.set_color(lerp_color(bg_from, bg_to, color_t));
        bg_paint.set_anti_alias(true);
        canvas.draw_rrect(
            RRect::new_rect_xy(track_bounds, track_radius, track_radius),
            &bg_paint,
        );

        let mut border_paint = Paint::default();
        border_paint.set_color(lerp_color(border_from, border_to, color_t));
        border_paint.set_anti_alias(true);
        border_paint.set_style(PaintStyle::Stroke);
        border_paint.set_stroke_width(1.0);
        canvas.draw_rrect(
            RRect::new_rect_xy(
                track_bounds.with_inset((0.5, 0.5)),
                track_radius - 0.5,
                track_radius - 0.5,
            ),
            &border_paint,
        );

        let box_size = rect.height;
        let scale = box_size / REFERENCE_BOX;
        let box_off_left = 0.0;
        let box_on_left = rect.width - box_size;
        let position_keyframes = if self.state.on {
            [
                box_off_left,
                box_off_left,
                box_off_left,
                box_on_left,
                box_on_left,
                box_on_left,
            ]
        } else {
            [
                box_on_left,
                box_on_left,
                box_on_left,
                box_off_left,
                box_off_left,
                box_off_left,
            ]
        };
        let box_left = keyframe(t, &S_BREAKS, &position_keyframes);

        let width = keyframe(t, &S_BREAKS, &WIDTH_KEYFRAMES) * scale;
        let height = keyframe(t, &S_BREAKS, &HEIGHT_KEYFRAMES) * scale;

        let corner_radius = height / 2.0;
        let local_x = (box_size - width) / 2.0;
        let local_y = (box_size - height) / 2.0;

        let mut thumb_paint = Paint::default();
        thumb_paint.set_color(THUMB);
        thumb_paint.set_anti_alias(true);
        let thumb_bounds =
            Rect::from_xywh(rect.x + box_left + local_x, rect.y + local_y, width, height);
        canvas.draw_rrect(
            RRect::new_rect_xy(thumb_bounds, corner_radius, corner_radius),
            &thumb_paint,
        );

        if self.icons {
            let showing_on = if t < 0.5 {
                !self.state.on
            } else {
                self.state.on
            };
            let (paths, color) = if showing_on {
                (THUMB_CHECK, BRAND)
            } else {
                (THUMB_X, THUMB_ICON_OFF)
            };

            let unit = box_size / 24.0;
            canvas.save();
            canvas.translate((rect.x + box_left + 4.0 * unit, rect.y + 4.0 * unit));
            canvas.scale((unit, unit));
            let mut icon_paint = Paint::default();
            icon_paint.set_color(color);
            icon_paint.set_anti_alias(true);
            for glyph in paths {
                if let Some(path) = skia_safe::Path::from_svg(glyph.path()) {
                    canvas.draw_path(&path, &icon_paint);
                }
            }
            canvas.restore();
        }
    }
}
