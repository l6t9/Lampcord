use crate::colors;
use std::rc::Rc;

use skia_safe::{Canvas, Color};

use crate::icon_handle::IconPath;
use crate::style::ComputedStyle;
use crate::svg_icon::ParsedSvg;
use crate::widget::{AbsoluteRect, Widget};

pub struct IconWidget {
    art: Rc<ParsedSvg>,
    color: Color,

    hover_color: Option<Color>,

    rotation_deg: f32,
}

impl IconWidget {
    pub fn new(icon: impl IconPath, color: Color) -> Option<Self> {
        Some(Self {
            art: icon.art()?,
            color,
            hover_color: None,
            rotation_deg: 0.0,
        })
    }

    pub fn with_rotation(mut self, degrees: f32) -> Self {
        self.rotation_deg = degrees;
        self
    }

    pub fn with_hover_color(mut self, hover_color: Color) -> Self {
        self.hover_color = Some(hover_color);
        self
    }
}

impl Widget for IconWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let active_color = colors::live(if let Some(color) = style.text_color {
            color
        } else if style.is_hovered {
            self.hover_color.unwrap_or(self.color)
        } else {
            self.color
        });
        if self.rotation_deg != 0.0 {
            canvas.save();
            canvas.rotate(
                self.rotation_deg,
                Some(skia_safe::Point::new(
                    rect.x + rect.width / 2.0,
                    rect.y + rect.height / 2.0,
                )),
            );
            self.art.draw_fitted(
                canvas,
                (rect.x, rect.y, rect.width, rect.height),
                active_color,
                1.0,
            );
            canvas.restore();
        } else {
            self.art.draw_fitted(
                canvas,
                (rect.x, rect.y, rect.width, rect.height),
                active_color,
                1.0,
            );
        }
    }
}
