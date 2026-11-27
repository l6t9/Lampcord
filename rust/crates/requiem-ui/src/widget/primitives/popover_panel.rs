use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use skia_safe::Canvas;

use super::box_paint::paint_box;
use crate::event::{EventResult, MouseButton, UiEvent};
use crate::style::ComputedStyle;
use crate::widget::anim::ease_out;
use crate::widget::{AbsoluteRect, Widget};

struct Surface {
    opened_at: Instant,

    alive: u32,

    gone_at: Option<Instant>,
}

thread_local! {
    static SURFACES: RefCell<HashMap<String, Surface>> = RefCell::new(HashMap::new());
}

const REBUILD_GRACE: Duration = Duration::from_millis(150);

pub struct PopoverPanelWidget {
    opened_at: Instant,
    identity: Option<String>,
}

const DURATION: Duration = Duration::from_millis(100);
const INITIAL_OPACITY: f32 = 0.5;
const INITIAL_SCALE_Y: f32 = 0.96;

impl PopoverPanelWidget {
    pub fn new() -> Self {
        Self {
            opened_at: Instant::now(),
            identity: None,
        }
    }
}

impl Drop for PopoverPanelWidget {
    fn drop(&mut self) {
        let Some(identity) = &self.identity else {
            return;
        };
        let _ = SURFACES.try_with(|surfaces| {
            if let Some(surface) = surfaces.borrow_mut().get_mut(identity) {
                surface.alive = surface.alive.saturating_sub(1);
                if surface.alive == 0 {
                    surface.gone_at = Some(Instant::now());
                }
            }
        });
    }
}

impl Default for PopoverPanelWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for PopoverPanelWidget {
    fn is_top_layer(&self) -> bool {
        true
    }

    fn identify(&mut self, identity: &str) {
        if self.identity.is_some() {
            return;
        }
        self.identity = Some(identity.to_string());
        SURFACES.with(|surfaces| {
            let mut surfaces = surfaces.borrow_mut();
            let surface = surfaces.entry(identity.to_string()).or_insert(Surface {
                opened_at: self.opened_at,
                alive: 0,
                gone_at: None,
            });
            let continuing = surface.alive > 0
                || surface
                    .gone_at
                    .is_some_and(|gone| gone.elapsed() < REBUILD_GRACE);
            if continuing {
                self.opened_at = surface.opened_at;
            } else {
                surface.opened_at = self.opened_at;
            }
            surface.alive += 1;
            surface.gone_at = None;
        });
    }

    fn handle_event(&mut self, event: &UiEvent, _rect: AbsoluteRect) -> EventResult {
        match *event {
            UiEvent::MouseDown {
                button: MouseButton::Left,
                ..
            }
            | UiEvent::MouseUp {
                button: MouseButton::Left,
                ..
            }
            | UiEvent::Scroll { .. } => EventResult::Handled,
            _ => EventResult::Ignored,
        }
    }

    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        paint_box(canvas, rect, style);
    }

    fn paint_group_transform(&self) -> Option<(f32, f32, f32, f32)> {
        let raw_t = (self.opened_at.elapsed().as_secs_f32() / DURATION.as_secs_f32()).min(1.0);

        if raw_t >= 1.0 {
            return None;
        }
        let t = ease_out(raw_t);
        let opacity = INITIAL_OPACITY + (1.0 - INITIAL_OPACITY) * t;
        let scale_y = INITIAL_SCALE_Y + (1.0 - INITIAL_SCALE_Y) * t;
        Some((opacity, 1.0, scale_y, 0.0))
    }
}
