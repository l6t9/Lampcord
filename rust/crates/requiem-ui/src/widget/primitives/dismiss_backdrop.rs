use std::rc::Rc;

use crate::event::{EventResult, UiEvent};
use crate::widget::{AbsoluteRect, Widget};

pub struct DismissBackdropWidget {
    on_dismiss: Rc<dyn Fn()>,

    covers_window: bool,
}

impl DismissBackdropWidget {
    pub fn new(on_dismiss: Rc<dyn Fn()>) -> Self {
        Self {
            on_dismiss,
            covers_window: false,
        }
    }

    pub fn covering_window(on_dismiss: Rc<dyn Fn()>) -> Self {
        Self {
            on_dismiss,
            covers_window: true,
        }
    }
}

impl Widget for DismissBackdropWidget {
    fn is_dismiss_backdrop(&self) -> bool {
        true
    }

    fn traps_pointer(&self) -> bool {
        true
    }

    fn is_top_layer(&self) -> bool {
        self.covers_window
    }

    fn extra_hit_rect(&self, _rect: AbsoluteRect) -> Option<AbsoluteRect> {
        self.covers_window.then_some(AbsoluteRect {
            x: -1.0e6,
            y: -1.0e6,
            width: 2.0e6,
            height: 2.0e6,
        })
    }

    fn dismiss(&mut self) -> bool {
        (self.on_dismiss)();
        true
    }

    fn handle_event(&mut self, event: &UiEvent, _rect: AbsoluteRect) -> EventResult {
        match *event {
            UiEvent::MouseDown { .. } => {
                (self.on_dismiss)();
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }
}
