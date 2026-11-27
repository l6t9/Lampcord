use crate::colors;
use crate::icon_handle;
use crate::svg_icon;
use std::cell::RefCell;
use std::collections::HashMap;
use std::f32::consts;
use std::rc;
use std::time::{Duration, Instant};

use skia_safe::{Canvas, Color};

use crate::style::ComputedStyle;
use crate::widget::anim::ease_out;
use crate::widget::{AbsoluteRect, Widget};

pub struct ChevronWidget {
    art: Option<rc::Rc<svg_icon::ParsedSvg>>,
    color: Color,
    open: bool,
    opened_at: Instant,
    identity: Option<String>,
}

struct Flip {
    opened_at: Instant,

    alive: u32,

    gone_at: Option<Instant>,
}

thread_local! {
    static FLIPS: RefCell<HashMap<String, Flip>> = RefCell::new(HashMap::new());
}

const REBUILD_GRACE: Duration = Duration::from_millis(150);

const DURATION_SECS: f32 = 0.2;

impl ChevronWidget {
    pub fn new(icon: impl icon_handle::IconPath, color: Color, open: bool) -> Self {
        Self {
            art: icon.art(),
            color,
            open,
            opened_at: Instant::now(),
            identity: None,
        }
    }

    pub fn set_open(&mut self, open: bool) {
        if open && !self.open {
            self.opened_at = Instant::now();
        }
        self.open = open;
    }
}

impl Drop for ChevronWidget {
    fn drop(&mut self) {
        let Some(identity) = &self.identity else {
            return;
        };
        let _ = FLIPS.try_with(|flips| {
            if let Some(flip) = flips.borrow_mut().get_mut(identity) {
                flip.alive = flip.alive.saturating_sub(1);
                if flip.alive == 0 {
                    flip.gone_at = Some(Instant::now());
                }
            }
        });
    }
}

impl Widget for ChevronWidget {
    fn identify(&mut self, identity: &str) {
        if !self.open || self.identity.is_some() {
            return;
        }
        self.identity = Some(identity.to_string());
        FLIPS.with(|flips| {
            let mut flips = flips.borrow_mut();
            let flip = flips.entry(identity.to_string()).or_insert(Flip {
                opened_at: self.opened_at,
                alive: 0,
                gone_at: None,
            });
            let continuing = flip.alive > 0
                || flip
                    .gone_at
                    .is_some_and(|gone| gone.elapsed() < REBUILD_GRACE);
            if continuing {
                self.opened_at = flip.opened_at;
            } else {
                flip.opened_at = self.opened_at;
            }
            flip.alive += 1;
            flip.gone_at = None;
        });
    }

    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let scale_y = if self.open {
            let t = ease_out((self.opened_at.elapsed().as_secs_f32() / DURATION_SECS).min(1.0));
            (consts::PI * t).cos()
        } else {
            1.0
        };

        let Some(art) = &self.art else { return };
        let scale = (rect.width / art.width).min(rect.height / art.height);
        let cx = rect.x + rect.width / 2.0;
        let cy = rect.y + rect.height / 2.0;
        canvas.save();
        canvas.translate((cx, cy));
        canvas.scale((scale, scale * scale_y));
        canvas.translate((-art.width / 2.0, -art.height / 2.0));
        art.draw(
            canvas,
            colors::live(style.text_color.unwrap_or(self.color)),
            1.0,
        );
        canvas.restore();
    }
}
