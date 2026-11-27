use crate::app;
use std::cell::Cell;
use std::rc::Rc;

use skia_safe::{Canvas, Color4f, Paint, RRect, Rect};

use crate::event::{EventResult, UiEvent};
use crate::style::ComputedStyle;
use crate::widget::{AbsoluteRect, ScrollChild, Widget};

const TRACK_HEIGHT: f32 = 8.0;
const THUMB_INSET: f32 = 2.0;
const THUMB_RADIUS: f32 = 2.0;
const THUMB_MIN_WIDTH: f32 = 40.0;

#[derive(Default)]
pub struct HScrollWidget {
    offset_x: f32,
    max_offset_x: f32,
    viewport_width: f32,
    content_width: f32,

    shared_offset: Option<Rc<Cell<f32>>>,
}

impl HScrollWidget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_shared_offset(mut self, offset: Rc<Cell<f32>>) -> Self {
        self.offset_x = offset.get();
        self.shared_offset = Some(offset);
        self
    }
}

impl Widget for HScrollWidget {
    fn paint(&self, _canvas: &Canvas, _rect: AbsoluteRect, _style: &ComputedStyle) {}

    fn paint_overlay(&self, canvas: &Canvas, rect: AbsoluteRect) {
        if self.max_offset_x <= 0.0 || self.content_width <= 0.0 {
            return;
        }
        let thumb_width = (self.viewport_width * self.viewport_width / self.content_width)
            .max(THUMB_MIN_WIDTH)
            .min(self.viewport_width);
        let travel = self.viewport_width - thumb_width;
        let thumb_x = rect.x + travel * (self.offset_x / self.max_offset_x);
        let thumb_y = rect.y + rect.height - TRACK_HEIGHT + THUMB_INSET;

        let mut paint = Paint::new(
            Color4f::new(
                0x5f as f32 / 255.0,
                0x60 as f32 / 255.0,
                0x6a as f32 / 255.0,
                1.0,
            ),
            None,
        );
        paint.set_anti_alias(true);
        let bounds = Rect::from_xywh(
            thumb_x,
            thumb_y,
            thumb_width,
            TRACK_HEIGHT - THUMB_INSET * 2.0,
        );
        canvas.draw_rrect(
            RRect::new_rect_xy(bounds, THUMB_RADIUS, THUMB_RADIUS),
            &paint,
        );
    }

    fn handle_event(&mut self, event: &UiEvent, _rect: AbsoluteRect) -> EventResult {
        let UiEvent::Scroll {
            delta_x, delta_y, ..
        } = event
        else {
            return EventResult::Ignored;
        };
        if self.max_offset_x <= 0.0 {
            return EventResult::Ignored;
        }
        let delta = if *delta_x != 0.0 {
            *delta_x
        } else if app::shift_held() {
            *delta_y
        } else {
            return EventResult::Ignored;
        };
        self.offset_x = (self.offset_x - delta).clamp(0.0, self.max_offset_x);
        if let Some(shared) = &self.shared_offset {
            shared.set(self.offset_x);
        }
        EventResult::Handled
    }

    fn scroll_offset(&self) -> Option<(f32, f32)> {
        Some((self.offset_x, 0.0))
    }

    fn update_scroll_bounds(
        &mut self,
        viewport: (f32, f32),
        content: (f32, f32),
        _children: &[ScrollChild<'_>],
    ) {
        self.viewport_width = viewport.0;
        self.content_width = content.0;
        self.max_offset_x = (content.0 - viewport.0).max(0.0);
        self.offset_x = self.offset_x.clamp(0.0, self.max_offset_x);
        if let Some(shared) = &self.shared_offset {
            shared.set(self.offset_x);
        }
    }
}
