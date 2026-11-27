use crate::paint as crate_paint;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use skia_safe::{Canvas, Color, Paint, RRect, Rect};

use crate::style::ComputedStyle;
use crate::widget::{AbsoluteRect, Widget};

const FILL_TRANSITION: Duration = Duration::from_millis(200);

#[derive(Clone, Copy)]
struct FillAnim {
    from: f32,
    to: f32,
    started: Instant,
}

impl FillAnim {
    fn current(&self) -> f32 {
        let t = (self.started.elapsed().as_secs_f32() / FILL_TRANSITION.as_secs_f32()).min(1.0);

        let eased = 1.0 - (1.0 - t) * (1.0 - t);
        self.from + (self.to - self.from) * eased
    }
}

thread_local! {

    static FILLS: RefCell<HashMap<u64, FillAnim>> = RefCell::new(HashMap::new());
}

pub fn forget_progress(key: u64) {
    FILLS.with(|fills| fills.borrow_mut().remove(&key));
}

pub struct ProgressFillWidget {
    key: u64,
}

impl ProgressFillWidget {
    pub fn new(key: u64, fraction: f32) -> Self {
        FILLS.with(|fills| {
            let mut fills = fills.borrow_mut();
            match fills.get_mut(&key) {
                Some(anim) if (anim.to - fraction).abs() > f32::EPSILON => {
                    *anim = FillAnim {
                        from: anim.current(),
                        to: fraction,
                        started: Instant::now(),
                    }
                }
                Some(_) => {}
                None => {
                    fills.insert(
                        key,
                        FillAnim {
                            from: 0.0,
                            to: fraction,
                            started: Instant::now(),
                        },
                    );
                }
            }
        });
        Self { key }
    }
}

impl Widget for ProgressFillWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let Some(anim) = FILLS.with(|fills| fills.borrow().get(&self.key).copied()) else {
            return;
        };
        let width = rect.width * anim.current().clamp(0.0, 1.0);
        if width > 0.0 {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_color(
                style
                    .text_color
                    .unwrap_or(Color::from_rgb(0x58, 0x65, 0xf2)),
            );
            let radius = rect.height / 2.0;
            canvas.draw_rrect(
                RRect::new_rect_xy(
                    Rect::from_xywh(rect.x, rect.y, width, rect.height),
                    radius,
                    radius,
                ),
                &paint,
            );
        }
        if anim.started.elapsed() < FILL_TRANSITION {
            crate_paint::animate_region(
                canvas,
                Rect::from_xywh(rect.x, rect.y, rect.width, rect.height),
            );
        }
    }
}
