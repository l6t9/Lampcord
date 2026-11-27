use requiem_ui::app;
use requiem_ui::components::div;
use requiem_ui::event::{CursorIcon, EventResult, MouseButton, UiEvent};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::anim::ColorTransition;
use requiem_ui::widget::{AbsoluteRect, Widget};
use skia_safe::{Canvas, Color, Paint, Rect};
use std::time::Duration;

pub const MIN_WIDTH: f32 = 240.0;

pub const MAX_WIDTH: f32 = 480.0;

const PANEL_EXTRA: f32 = 72.0 - 16.0;

thread_local! {
    static WIDTH: Signal<f32> = signal(MIN_WIDTH);
    static STYLE: Signal<String> = signal(format!("width: {MIN_WIDTH}px"));
    static PANEL_STYLE: Signal<String> = signal(format!("width: {}px", MIN_WIDTH + PANEL_EXTRA));
}

pub fn style() -> Signal<String> {
    STYLE.with(Clone::clone)
}

pub fn panel_style() -> Signal<String> {
    PANEL_STYLE.with(Clone::clone)
}

pub fn width() -> f32 {
    WIDTH.with(|w| w.peek())
}

fn set_width(width: f32) {
    WIDTH.with(|w| w.set(width));
    STYLE.with(|s| s.set(format!("width: {width}px")));
    PANEL_STYLE.with(|s| s.set(format!("width: {}px", width + PANEL_EXTRA)));
}

pub fn reset() {
    set_width(MIN_WIDTH);
}

struct ResizeHandle {
    fade: ColorTransition,
    dragging: bool,

    start: (f32, f32),
}

impl Widget for ResizeHandle {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let color = self
            .fade
            .animate(style.background.unwrap_or(Color::TRANSPARENT));
        if color.a() > 0 {
            let mut paint = Paint::default();
            paint.set_color(color);
            canvas.draw_rect(
                Rect::from_xywh(rect.x, rect.y, rect.width, rect.height),
                &paint,
            );
        }
    }

    fn handle_event(&mut self, event: &UiEvent, _rect: AbsoluteRect) -> EventResult {
        match *event {
            UiEvent::MouseDown {
                x,
                button: MouseButton::Left,
                ..
            } => {
                self.dragging = true;
                self.start = (x, width());
                EventResult::Handled
            }
            UiEvent::MouseMoved { x, .. } if self.dragging => {
                let max = (app::current_window_size().0 - 72.0 - 400.0)
                    .clamp(MIN_WIDTH, MAX_WIDTH);
                set_width((self.start.1 + (x - self.start.0)).clamp(MIN_WIDTH, max));
                EventResult::Handled
            }
            UiEvent::MouseUp { .. } if self.dragging => {
                self.dragging = false;
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }

    fn wants_pointer_capture(&self) -> bool {
        self.dragging
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        Some(CursorIcon::ColResize)
    }
}

pub fn handle() -> Node {
    Node::new(
        "div",
        ResizeHandle {
            fade: ColorTransition::new(Duration::from_millis(200)),
            dragging: false,
            start: (0.0, 0.0),
        },
    )
    .class(super::classes::sidebar_resize::HANDLE)
}

pub fn container() -> Node {
    div()
        .class(super::classes::channel_sidebar::CHANNEL_SIDEBAR)
        .bind_style(style())
}
