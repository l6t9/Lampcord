use std::time::Duration;

use skia_safe::{Canvas, Paint, Rect};

use super::box_paint::paint_box;
use crate::event::{CursorIcon, EventResult, MouseButton, UiEvent};
use crate::style::ComputedStyle;
use crate::widget::anim::{ToggleState, cubic_bezier, ease_out};
use crate::widget::{AbsoluteRect, Widget};

const DURATION: Duration = Duration::from_millis(300);
const INITIAL_SCALE: f32 = 0.7;
const EASE_OUT_BACK: (f32, f32, f32, f32) = (0.34, 1.56, 0.64, 1.0);

fn raw_progress(toggle: ToggleState, duration: Duration) -> f32 {
    (toggle.changed_at.elapsed().as_secs_f32() / duration.as_secs_f32()).min(1.0)
}

fn open_amount(toggle: ToggleState) -> f32 {
    let eased = ease_out(raw_progress(toggle, DURATION));
    if toggle.on { eased } else { 1.0 - eased }
}

fn scale_amount(toggle: ToggleState) -> f32 {
    let t = raw_progress(toggle, DURATION);
    let eased = cubic_bezier(
        EASE_OUT_BACK.0,
        EASE_OUT_BACK.1,
        EASE_OUT_BACK.2,
        EASE_OUT_BACK.3,
        t,
    );
    if toggle.on { eased } else { 1.0 - eased }
}

pub struct ModalPanelWidget {
    toggle: ToggleState,
}

impl ModalPanelWidget {
    pub fn new(toggle: ToggleState) -> Self {
        Self { toggle }
    }
}

impl Widget for ModalPanelWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        paint_box(canvas, rect, style);
    }

    fn handle_event(&mut self, event: &UiEvent, _rect: AbsoluteRect) -> EventResult {
        if !self.toggle.on {
            return EventResult::Ignored;
        }
        match *event {
            UiEvent::MouseDown {
                button: MouseButton::Left,
                ..
            } => EventResult::Handled,
            UiEvent::MouseUp {
                button: MouseButton::Left,
                ..
            } => EventResult::Handled,
            _ => EventResult::Ignored,
        }
    }

    fn traps_pointer(&self) -> bool {
        self.toggle.on
    }

    fn paint_group_transform(&self) -> Option<(f32, f32, f32, f32)> {
        if self.toggle.on && raw_progress(self.toggle, DURATION) >= 1.0 {
            return None;
        }
        let scale = INITIAL_SCALE + (1.0 - INITIAL_SCALE) * scale_amount(self.toggle);
        Some((open_amount(self.toggle), scale, scale, 0.5))
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        self.toggle.on.then_some(CursorIcon::Default)
    }

    fn blocks_descendant_hit_test(&self) -> bool {
        !self.toggle.on
    }

    fn is_open_modal(&self) -> bool {
        self.toggle.on
    }

    fn hides_descendant_focus(&self) -> bool {
        !self.toggle.on
    }
}

pub struct ModalBackdropWidget {
    toggle: ToggleState,
    on_click: Option<Box<dyn FnMut()>>,
}

impl ModalBackdropWidget {
    pub fn new(toggle: ToggleState) -> Self {
        Self {
            toggle,
            on_click: None,
        }
    }

    pub fn with_on_click(mut self, on_click: impl FnMut() + 'static) -> Self {
        self.on_click = Some(Box::new(on_click));
        self
    }
}

impl Widget for ModalBackdropWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let Some(color) = style.background else {
            return;
        };
        let mut paint = Paint::default();
        paint.set_color(color);
        paint.set_anti_alias(true);
        canvas.draw_rect(
            Rect::from_xywh(rect.x, rect.y, rect.width, rect.height),
            &paint,
        );
    }

    fn handle_event(&mut self, event: &UiEvent, _rect: AbsoluteRect) -> EventResult {
        if !self.toggle.on {
            return EventResult::Ignored;
        }
        match *event {
            UiEvent::MouseDown {
                button: MouseButton::Left,
                ..
            } => {
                if let Some(on_click) = &mut self.on_click {
                    on_click();
                }
                EventResult::Handled
            }
            UiEvent::MouseUp {
                button: MouseButton::Left,
                ..
            } => EventResult::Handled,
            _ => EventResult::Ignored,
        }
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        self.toggle.on.then_some(CursorIcon::Default)
    }

    fn paint_group_transform(&self) -> Option<(f32, f32, f32, f32)> {
        Some((open_amount(self.toggle), 1.0, 1.0, 0.5))
    }

    fn blocks_descendant_hit_test(&self) -> bool {
        !self.toggle.on
    }

    fn traps_pointer(&self) -> bool {
        self.toggle.on
    }
}
