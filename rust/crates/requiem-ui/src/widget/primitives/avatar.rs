use crate::icon_handle::IconPath;
use crate::icon_handle;
use crate::svg_icon;
use skia_safe::surfaces;
use skia_safe::{Canvas, ClipOp, Color, Font, Image, Matrix, Paint, RRect, Rect, Typeface};
use std::rc;

use crate::style::ComputedStyle;
use crate::widget::{AbsoluteRect, Widget};

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum AvatarShape {
    #[default]
    Circle,

    RoundedSquare(f32),

    Squircle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FallbackIcon {
    pub path: icon_handle::Icon,
    pub width_fraction: f32,
}

/// Draws a fallback icon in white, centered in `rect` and `width_fraction` of its width
/// across. Shared by the avatar widget and the default-avatar images made for places
/// that can't use the widget (notifications).
pub fn draw_fallback_icon(
    canvas: &Canvas,
    art: &svg_icon::ParsedSvg,
    rect: AbsoluteRect,
    width_fraction: f32,
) {
    let glyph = art.glyph_bounds();
    if glyph.width() <= 0.0 {
        return;
    }
    let scale = rect.width * width_fraction / glyph.width();
    let (cx, cy) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    let mut matrix = Matrix::translate((
        cx - glyph.center_x() * scale,
        cy - glyph.center_y() * scale,
    ));
    matrix.pre_scale((scale, scale), None);
    canvas.save();
    canvas.concat(&matrix);
    art.draw(canvas, Color::WHITE, 1.0);
    canvas.restore();
}

/// A square avatar image: `icon` on `background`, with the same drawing the widget uses.
pub fn render_fallback_avatar(background: Color, icon: FallbackIcon, size: i32) -> Option<Image> {
    let mut surface = surfaces::raster_n32_premul((size, size))?;
    let canvas = surface.canvas();
    canvas.clear(background);
    let art = icon.path.art()?;
    let side = size as f32;
    draw_fallback_icon(
        canvas,
        &art,
        AbsoluteRect {
            x: 0.0,
            y: 0.0,
            width: side,
            height: side,
        },
        icon.width_fraction,
    );
    Some(surface.image_snapshot())
}

pub struct AvatarWidget {
    image: Option<Image>,
    fallback_initial: String,
    fallback_background: Color,
    typeface: Typeface,
    shape: AvatarShape,
    status_cutout: Option<StatusCutout>,
    fallback_icon: Option<(rc::Rc<svg_icon::ParsedSvg>, f32)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatusCutout {
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
}

impl StatusCutout {
    pub const ROUND_20: Self = Self {
        cx: 0.85,
        cy: 0.85,
        r: 0.25,
    };

    pub const ROUND_24: Self = Self {
        cx: 0.8333333,
        cy: 0.8333333,
        r: 0.25,
    };

    pub const ROUND_32: Self = Self {
        cx: 0.84375,
        cy: 0.84375,
        r: 0.25,
    };

    pub const ROUND_80: Self = Self {
        cx: 0.85,
        cy: 0.85,
        r: 0.175,
    };

    pub const ROUND_40: Self = Self {
        cx: 0.85,
        cy: 0.85,
        r: 0.25,
    };

    pub const ROUND_120: Self = Self {
        cx: 0.8333333,
        cy: 0.8333333,
        r: 0.16666667,
    };
}

impl AvatarWidget {
    pub fn new(
        image: Option<Image>,
        fallback_initial: impl Into<String>,
        fallback_background: Color,
        typeface: Typeface,
        shape: AvatarShape,
    ) -> Self {
        Self {
            image,
            fallback_initial: fallback_initial.into(),
            fallback_background,
            typeface,
            shape,
            status_cutout: None,
            fallback_icon: None,
        }
    }

    pub fn with_fallback_icon(mut self, icon: Option<FallbackIcon>) -> Self {
        self.fallback_icon = icon.and_then(|icon| {
            IconPath::art(&icon.path).map(|art| (art, icon.width_fraction))
        });
        self
    }

    pub fn with_status_cutout(mut self, cutout: Option<StatusCutout>) -> Self {
        self.status_cutout = cutout;
        self
    }

    pub fn set_image(&mut self, image: Image) {
        self.image = Some(image);
    }
}

impl Widget for AvatarWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
        let radius = match self.shape {
            AvatarShape::Circle => rect.width.min(rect.height) / 2.0,
            AvatarShape::RoundedSquare(fraction) => rect.width.min(rect.height) * fraction,
            AvatarShape::Squircle => 0.0,
        };

        canvas.save();
        if self.shape == AvatarShape::Squircle {
            canvas.clip_path(&super::container::squircle_path(bounds), None, true);
        } else {
            canvas.clip_rrect(RRect::new_rect_xy(bounds, radius, radius), None, true);
        }
        if let Some(cut) = self.status_cutout {
            let size = rect.width.min(rect.height);
            let (cx, cy, r) = (
                rect.x + rect.width * cut.cx,
                rect.y + rect.height * cut.cy,
                size * cut.r,
            );
            canvas.clip_rrect(
                RRect::new_oval(Rect::from_xywh(cx - r, cy - r, r * 2.0, r * 2.0)),
                ClipOp::Difference,
                true,
            );
        }

        match &self.image {
            Some(image) => {
                let mut paint = Paint::default();
                paint.set_anti_alias(true);

                super::draw_image_filtered(canvas, image, None, bounds, &paint);
            }
            None => {
                let mut bg_paint = Paint::default();
                bg_paint.set_color(self.fallback_background);
                bg_paint.set_anti_alias(true);
                canvas.draw_rect(bounds, &bg_paint);

                if let Some((art, width_fraction)) = &self.fallback_icon {
                    draw_fallback_icon(canvas, art, rect, *width_fraction);
                    canvas.restore();
                    return;
                }

                let font_size = rect.height * 0.45;
                let font = Font::new(self.typeface.clone(), font_size);
                let mut text_paint = Paint::default();
                text_paint.set_color(Color::WHITE);
                text_paint.set_anti_alias(true);

                let (text_width, _) = font.measure_str(&self.fallback_initial, Some(&text_paint));
                let x = rect.x + (rect.width - text_width) / 2.0;

                let y = rect.y + rect.height / 2.0 + font_size * 0.35;
                canvas.draw_str(&self.fallback_initial, (x, y), &font, &text_paint);
            }
        }

        canvas.restore();
    }
}
