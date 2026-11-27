use crate::paint as crate_paint;
use crate::style::transition::NodeTransitions;
use skia_safe::{Canvas, Color, Paint, RRect, Rect};

use crate::event::{CursorIcon, EventResult, MouseButton, NamedKey, UiEvent};
use crate::style::ComputedStyle;
use crate::widget::anim::ColorTransition;
use crate::widget::primitives::container::{corner_radii, has_any_corner_radius};
use crate::widget::{AbsoluteRect, Widget};

pub struct ButtonWidget {
    pressed: bool,

    pressed_shift: bool,
    on_click: Option<Box<dyn FnMut()>>,

    on_click_shift: Option<Box<dyn FnMut(bool)>>,
    transition: ColorTransition,

    hovered: bool,
    on_hover_change: Option<Box<dyn FnMut(bool)>>,
    on_click_at: Option<Box<dyn FnMut(AbsoluteRect)>>,

    focusable: bool,
}

impl ButtonWidget {
    pub fn new() -> Self {
        Self {
            pressed: false,
            pressed_shift: false,
            on_click: None,
            on_click_shift: None,
            transition: ColorTransition::default(),
            hovered: false,
            on_hover_change: None,
            on_click_at: None,
            focusable: true,
        }
    }

    pub fn with_on_click(mut self, on_click: impl FnMut() + 'static) -> Self {
        self.on_click = Some(Box::new(on_click));
        self
    }

    pub fn with_on_click_shift(mut self, on_click_shift: impl FnMut(bool) + 'static) -> Self {
        self.on_click_shift = Some(Box::new(on_click_shift));
        self
    }

    pub fn with_on_click_at(mut self, on_click_at: impl FnMut(AbsoluteRect) + 'static) -> Self {
        self.on_click_at = Some(Box::new(on_click_at));
        self
    }

    pub fn with_focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn with_on_hover_change(mut self, on_hover_change: impl FnMut(bool) + 'static) -> Self {
        self.on_hover_change = Some(Box::new(on_hover_change));
        self
    }
}

impl Default for ButtonWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ButtonWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let wanted = style.background.unwrap_or(Color::TRANSPARENT);

        let color = if NodeTransitions::animates_background(style) {
            wanted
        } else {
            self.transition.animate(wanted)
        };
        let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
        if color != wanted {
            crate_paint::animate_region(canvas, bounds);
        }
        let mut paint = Paint::default();
        paint.set_color(color);
        paint.set_anti_alias(true);

        if has_any_corner_radius(style) {
            let radii = corner_radii(style);
            canvas.draw_rrect(RRect::new_rect_radii(bounds, &radii), &paint);
        } else {
            canvas.draw_rect(bounds, &paint);
        }
        super::container::paint_borders(canvas, bounds, style);
    }

    fn handle_event(&mut self, event: &UiEvent, rect: AbsoluteRect) -> EventResult {
        match *event {
            UiEvent::MouseMoved { .. } => {
                if !self.hovered {
                    self.hovered = true;
                    if let Some(on_hover_change) = &mut self.on_hover_change {
                        on_hover_change(true);
                    }
                }
                EventResult::Ignored
            }
            UiEvent::MouseDown {
                button: MouseButton::Left,
                shift,
                ..
            } => {
                self.pressed = true;
                self.pressed_shift = shift;
                EventResult::Handled
            }
            UiEvent::MouseUp {
                button: MouseButton::Left,
                ..
            } => {
                let was_pressed = self.pressed;
                self.pressed = false;
                if was_pressed {
                    if let Some(on_click) = &mut self.on_click {
                        on_click();
                    }
                    if let Some(on_click_shift) = &mut self.on_click_shift {
                        on_click_shift(self.pressed_shift);
                    }
                    if let Some(on_click_at) = &mut self.on_click_at {
                        on_click_at(rect);
                    }
                }
                EventResult::Handled
            }

            UiEvent::KeyDown {
                key: NamedKey::Enter | NamedKey::Space,
                shift,
            } => {
                if self.on_click.is_none()
                    && self.on_click_shift.is_none()
                    && self.on_click_at.is_none()
                {
                    return EventResult::Ignored;
                }
                if let Some(on_click) = &mut self.on_click {
                    on_click();
                }
                if let Some(on_click_shift) = &mut self.on_click_shift {
                    on_click_shift(shift);
                }
                if let Some(on_click_at) = &mut self.on_click_at {
                    on_click_at(rect);
                }
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }

    fn clear_press(&mut self) {
        self.pressed = false;
    }

    fn clear_hover(&mut self) {
        if self.hovered {
            self.hovered = false;
            if let Some(on_hover_change) = &mut self.on_hover_change {
                on_hover_change(false);
            }
        }
    }

    fn wants_focus(&self) -> bool {
        self.focusable
            && (self.on_click.is_some()
                || self.on_click_shift.is_some()
                || self.on_click_at.is_some())
    }

    fn on_focus_change(&mut self, focused: bool) {
        if focused && !self.hovered {
            if let Some(on_hover_change) = &mut self.on_hover_change {
                on_hover_change(true);
            }
        }
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        (self.on_click.is_some() || self.on_click_shift.is_some() || self.on_click_at.is_some())
            .then_some(CursorIcon::Pointer)
    }
}
