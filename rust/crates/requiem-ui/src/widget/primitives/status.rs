use skia_safe::{Color, Paint, Path, PathOp, RRect, Rect};

use crate::style::ComputedStyle;
use crate::widget::{AbsoluteRect, Widget};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PresenceStatus {
    Online,
    Idle,
    Dnd,
    #[default]
    Offline,
}

impl PresenceStatus {
    pub fn from_gateway(status: &str) -> Self {
        match status {
            "online" => Self::Online,
            "idle" => Self::Idle,
            "dnd" => Self::Dnd,
            _ => Self::Offline,
        }
    }
}

pub struct StatusIndicatorWidget {
    status: PresenceStatus,
    fallback: Color,
}

impl StatusIndicatorWidget {
    pub fn new(status: PresenceStatus, fallback: Color) -> Self {
        Self { status, fallback }
    }
}

pub fn status_dot_color(status: PresenceStatus) -> Color {
    match status {
        PresenceStatus::Online => Color::from_rgb(0x45, 0xa3, 0x66),
        PresenceStatus::Idle => Color::from_rgb(0xff, 0xc0, 0x4e),
        PresenceStatus::Dnd => Color::from_rgb(0xda, 0x3e, 0x44),
        PresenceStatus::Offline => Color::from_rgb(0x84, 0x85, 0x8d),
    }
}

impl Widget for StatusIndicatorWidget {
    fn paint(&self, canvas: &skia_safe::Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let size = rect.width.min(rect.height);
        let (x, y) = (
            rect.x + (rect.width - size) / 2.0,
            rect.y + (rect.height - size) / 2.0,
        );
        let oval = |cx: f32, cy: f32, r: f32| {
            Path::oval(
                Rect::from_xywh(
                    x + size * (cx - r),
                    y + size * (cy - r),
                    size * r * 2.0,
                    size * r * 2.0,
                ),
                None,
            )
        };

        let dot = oval(0.5, 0.5, 0.5);
        let cut = match self.status {
            PresenceStatus::Online => None,
            PresenceStatus::Idle => Some(oval(0.25, 0.25, 0.375)),
            PresenceStatus::Dnd => {
                let bar =
                    Rect::from_xywh(x + size * 0.125, y + size * 0.375, size * 0.75, size * 0.25);
                Some(Path::rrect(
                    RRect::new_rect_xy(bar, size * 0.125, size * 0.125),
                    None,
                ))
            }
            PresenceStatus::Offline => Some(oval(0.5, 0.5, 0.25)),
        };
        let shape = match cut {
            Some(cut) => dot.op(&cut, PathOp::Difference).unwrap_or(dot),
            None => dot,
        };

        let mut paint = Paint::default();
        paint.set_color(style.text_color.unwrap_or(self.fallback));
        paint.set_anti_alias(true);
        canvas.draw_path(&shape, &paint);
    }
}
