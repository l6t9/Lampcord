use crate::widget::anchor;
use skia_safe::Canvas;

use super::box_paint::paint_box;
use crate::event::{CursorIcon, EventResult, MouseButton, UiEvent};
use crate::style::ComputedStyle;
use crate::widget::{AbsoluteRect, Widget};

pub struct ClickAreaWidget {
    on_click: Option<Box<dyn FnMut()>>,

    position_override: Option<(f32, f32)>,

    anchor: Option<anchor::Anchor>,

    on_hover_path: Option<Box<dyn FnMut(bool)>>,

    swallow_scroll: bool,

    cursor: Option<CursorIcon>,
}

impl ClickAreaWidget {
    pub fn new() -> Self {
        Self {
            on_click: None,
            position_override: None,
            anchor: None,
            on_hover_path: None,
            swallow_scroll: false,
            cursor: None,
        }
    }

    pub fn with_swallowed_scroll(mut self) -> Self {
        self.swallow_scroll = true;
        self
    }

    pub fn with_cursor(mut self, cursor: CursorIcon) -> Self {
        self.cursor = Some(cursor);
        self
    }

    pub fn with_on_click(mut self, on_click: impl FnMut() + 'static) -> Self {
        self.on_click = Some(Box::new(on_click));
        self
    }

    pub fn with_absolute_position(mut self, x: f32, y: f32) -> Self {
        self.position_override = Some((x, y));
        self
    }

    pub fn with_anchor(mut self, anchor: anchor::Anchor) -> Self {
        self.anchor = Some(anchor);
        self
    }

    pub fn with_on_hover_path(mut self, on_change: impl FnMut(bool) + 'static) -> Self {
        self.on_hover_path = Some(Box::new(on_change));
        self
    }
}

impl Default for ClickAreaWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ClickAreaWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        paint_box(canvas, rect, style);
    }

    fn handle_event(&mut self, event: &UiEvent, _rect: AbsoluteRect) -> EventResult {
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
            UiEvent::Scroll { .. } if self.swallow_scroll => EventResult::Handled,
            _ => EventResult::Ignored,
        }
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        if self.cursor.is_some() {
            return self.cursor;
        }

        self.on_click.is_some().then_some(CursorIcon::Pointer)
    }

    fn absolute_position_override(&self) -> Option<(f32, f32)> {
        self.position_override
    }

    fn anchored_position(&self, size: (f32, f32)) -> Option<(f32, f32)> {
        self.anchor.map(|anchor| {
            anchor::place(&anchor, size, anchor::viewport())
        })
    }

    fn hover_path_changed(&mut self, inside: bool) {
        if let Some(on_change) = &mut self.on_hover_path {
            on_change(inside);
        }
    }
}
