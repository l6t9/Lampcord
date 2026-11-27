use crate::app;
use crate::paint as crate_paint;
use std::cell::Cell;
use std::rc::Rc;

use skia_safe::Canvas;

use crate::event::{EventResult, MouseButton, UiEvent};
use crate::style::ComputedStyle;
use crate::widget::primitives::PopoverPanelWidget;
use crate::widget::{AbsoluteRect, Widget};

const EDGE_MARGIN: f32 = 8.0;

pub fn place_menu(anchor: (f32, f32), size: (f32, f32), window: (f32, f32)) -> (f32, f32) {
    let x = if anchor.0 + size.0 > window.0 - EDGE_MARGIN {
        (anchor.0 - size.0).max(EDGE_MARGIN)
    } else {
        anchor.0
    };
    let y = if anchor.1 + size.1 > window.1 - EDGE_MARGIN {
        if anchor.1 - size.1 >= EDGE_MARGIN {
            anchor.1 - size.1
        } else {
            (window.1 - size.1 - EDGE_MARGIN).max(EDGE_MARGIN)
        }
    } else {
        anchor.1
    };
    (x, y)
}

pub struct ContextBackdropWidget {
    on_dismiss: Box<dyn FnMut()>,
}

impl ContextBackdropWidget {
    pub fn new(on_dismiss: impl FnMut() + 'static) -> Self {
        Self {
            on_dismiss: Box::new(on_dismiss),
        }
    }
}

impl Widget for ContextBackdropWidget {
    fn is_top_layer(&self) -> bool {
        true
    }

    fn handle_event(&mut self, event: &UiEvent, _rect: AbsoluteRect) -> EventResult {
        match *event {
            UiEvent::MouseDown {
                button: MouseButton::Left,
                ..
            } => {
                (self.on_dismiss)();
                EventResult::Handled
            }
            UiEvent::MouseUp {
                button: MouseButton::Left,
                ..
            } => EventResult::Handled,
            UiEvent::MouseDown {
                button: MouseButton::Right,
                ..
            }
            | UiEvent::Scroll { .. } => {
                (self.on_dismiss)();
                EventResult::Ignored
            }
            _ => EventResult::Ignored,
        }
    }
}

pub struct ContextPanelWidget {
    anchor: (f32, f32),

    above: bool,
    size: Cell<Option<(f32, f32)>>,
    popover: PopoverPanelWidget,
}

impl ContextPanelWidget {
    pub fn new(anchor: (f32, f32)) -> Self {
        Self {
            anchor,
            above: false,
            size: Cell::new(None),
            popover: PopoverPanelWidget::new(),
        }
    }

    pub fn above(mut self, above: bool) -> Self {
        self.above = above;
        self
    }
}

impl Widget for ContextPanelWidget {
    fn is_top_layer(&self) -> bool {
        true
    }

    fn absolute_position_override(&self) -> Option<(f32, f32)> {
        let size = self.size.get().unwrap_or((0.0, 0.0));
        let window = app::current_window_size();
        let (x, y) = if self.above {
            let x = self
                .anchor
                .0
                .min(window.0 - size.0 - EDGE_MARGIN)
                .max(EDGE_MARGIN);
            (x, (self.anchor.1 - size.1).max(EDGE_MARGIN))
        } else {
            place_menu(self.anchor, size, window)
        };

        Some((x, y - app::current_top_inset()))
    }

    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        if self.size.get() != Some((rect.width, rect.height)) {
            self.size.set(Some((rect.width, rect.height)));
            crate_paint::request_repaint();
        }
        self.popover.paint(canvas, rect, style);
    }

    fn paint_group_transform(&self) -> Option<(f32, f32, f32, f32)> {
        if self.size.get().is_none() {
            return Some((0.0, 1.0, 1.0, 0.0));
        }
        self.popover.paint_group_transform()
    }

    fn handle_event(&mut self, event: &UiEvent, _rect: AbsoluteRect) -> EventResult {
        match event {
            UiEvent::MouseDown { .. } | UiEvent::MouseUp { .. } | UiEvent::Scroll { .. } => {
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }
}

pub fn place_submenu(host: AbsoluteRect, size: (f32, f32), window: (f32, f32)) -> (f32, f32) {
    let right = host.x + host.width + 12.0;
    let x = if right + size.0 > window.0 - EDGE_MARGIN {
        (host.x - 12.0 - size.0).max(EDGE_MARGIN)
    } else {
        right
    };
    let top = host.y - 9.0;
    let y = if top + size.1 > window.1 - EDGE_MARGIN {
        (window.1 - size.1 - EDGE_MARGIN).max(EDGE_MARGIN)
    } else {
        top
    };
    (x, y)
}

pub struct SubmenuHostWidget {
    rect: Rc<Cell<Option<AbsoluteRect>>>,
    on_hover: Box<dyn FnMut(bool)>,
}

impl SubmenuHostWidget {
    pub fn new(rect: Rc<Cell<Option<AbsoluteRect>>>, on_hover: impl FnMut(bool) + 'static) -> Self {
        Self {
            rect,
            on_hover: Box::new(on_hover),
        }
    }
}

impl Widget for SubmenuHostWidget {
    fn paint(&self, _canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        self.rect.set(Some(rect));
    }

    fn hover_path_changed(&mut self, inside: bool) {
        (self.on_hover)(inside);
    }
}

pub const SUBMENU_BRIDGE: f32 = 12.0;

pub struct SubmenuPanelWidget {
    host: Rc<Cell<Option<AbsoluteRect>>>,
    size: Cell<Option<(f32, f32)>>,
    popover: PopoverPanelWidget,
}

impl SubmenuPanelWidget {
    pub fn new(host: Rc<Cell<Option<AbsoluteRect>>>) -> Self {
        Self {
            host,
            size: Cell::new(None),
            popover: PopoverPanelWidget::new(),
        }
    }
}

impl Widget for SubmenuPanelWidget {
    fn is_top_layer(&self) -> bool {
        true
    }

    fn absolute_position_override(&self) -> Option<(f32, f32)> {
        let host = self.host.get()?;
        let (width, height) = self.size.get().unwrap_or((0.0, 0.0));
        let (x, y) = place_submenu(
            host,
            ((width - 2.0 * SUBMENU_BRIDGE).max(0.0), height),
            app::current_window_size(),
        );
        Some((x - SUBMENU_BRIDGE - host.x, y - host.y))
    }

    fn paint(&self, _canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        if self.size.get() != Some((rect.width, rect.height)) {
            self.size.set(Some((rect.width, rect.height)));
            crate_paint::request_repaint();
        }
    }

    fn paint_group_transform(&self) -> Option<(f32, f32, f32, f32)> {
        if self.size.get().is_none() {
            return Some((0.0, 1.0, 1.0, 0.0));
        }
        self.popover.paint_group_transform()
    }

    fn handle_event(&mut self, event: &UiEvent, _rect: AbsoluteRect) -> EventResult {
        match event {
            UiEvent::MouseDown { .. } | UiEvent::MouseUp { .. } | UiEvent::Scroll { .. } => {
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::place_menu;

    const WINDOW: (f32, f32) = (1568.0, 675.0);

    #[test]
    fn opens_below_right_when_it_fits() {
        assert_eq!(
            place_menu((300.0, 200.0), (222.0, 306.0), WINDOW),
            (300.0, 200.0)
        );
    }

    #[test]
    fn flips_left_past_the_right_edge() {
        assert_eq!(
            place_menu((1560.0, 100.0), (222.0, 306.0), WINDOW),
            (1338.0, 100.0)
        );
    }

    #[test]
    fn flips_above_when_there_is_room() {
        assert_eq!(
            place_menu((300.0, 670.0), (222.0, 306.0), WINDOW),
            (300.0, 364.0)
        );
    }

    #[test]
    fn slides_up_when_neither_side_fits() {
        assert_eq!(
            place_menu((1000.0, 200.0), (222.0, 597.0), WINDOW),
            (1000.0, 70.0)
        );
    }
}
