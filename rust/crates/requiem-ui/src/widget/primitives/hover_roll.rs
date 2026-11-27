use crate::paint as crate_paint;
use crate::style;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use skia_safe::{Canvas, Rect};
use taffy::{AvailableSpace, Size};

use crate::style::ComputedStyle;
use crate::widget::anim::FloatTransition;
use crate::widget::{AbsoluteRect, Widget};

#[derive(Clone, Default)]
pub struct HoverFlag(Rc<Cell<bool>>);

impl HoverFlag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&self, hovered: bool) {
        self.0.set(hovered);
        crate_paint::request_repaint();
    }

    pub fn get(&self) -> bool {
        self.0.get()
    }
}

pub struct HoverRollWidget {
    default: Box<dyn Widget>,
    hovered: Box<dyn Widget>,
    flag: HoverFlag,
    force: bool,
    progress: FloatTransition,
}

const TRAVEL: f32 = 1.07;

impl HoverRollWidget {
    pub fn new(default: impl Widget, hovered: impl Widget, flag: HoverFlag, force: bool) -> Self {
        Self {
            default: Box::new(default),
            hovered: Box::new(hovered),
            flag,
            force,
            progress: FloatTransition::new(Duration::from_millis(220)),
        }
    }
}

impl Widget for HoverRollWidget {
    fn measure(&self, known: Size<Option<f32>>, available: Size<AvailableSpace>) -> Size<f32> {
        self.default.measure(known, available)
    }

    fn first_baseline(&self, content_width: f32) -> Option<f32> {
        self.default.first_baseline(content_width)
    }

    fn apply_text_style(&mut self, text: &style::TextLayoutStyle) -> bool {
        let default = self.default.apply_text_style(text);
        let hovered = self.hovered.apply_text_style(text);
        default || hovered
    }

    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let wanted = if self.force || self.flag.get() {
            1.0
        } else {
            0.0
        };
        let t = self.progress.animate(wanted);
        if (t - wanted).abs() > 0.001 {
            crate_paint::animate_region(
                canvas,
                Rect::from_xywh(rect.x, rect.y, rect.width, rect.height),
            );
        }

        canvas.save();
        canvas.clip_rect(
            Rect::from_xywh(rect.x, rect.y, rect.width, rect.height),
            None,
            true,
        );
        let travel = rect.height * TRAVEL;
        let layers = [
            (&self.default, -t * travel, 1.0 - t),
            (&self.hovered, (1.0 - t) * travel, t),
        ];
        for (text, offset, alpha) in layers {
            if alpha <= 0.001 {
                continue;
            }
            let layer_rect = AbsoluteRect {
                y: rect.y + offset,
                ..rect
            };
            if alpha >= 0.999 {
                text.paint(canvas, layer_rect, style);
                continue;
            }
            canvas.save_layer_alpha_f(None, alpha);
            text.paint(canvas, layer_rect, style);
            canvas.restore();
        }
        canvas.restore();
    }
}
