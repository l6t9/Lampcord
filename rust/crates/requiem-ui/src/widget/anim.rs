use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use skia_safe::{Color, Color4f};

pub const DEFAULT_TRANSITION: Duration = Duration::from_millis(150);

#[derive(Clone)]
pub struct ToggleMemo(Rc<Cell<ToggleState>>);

impl ToggleMemo {
    pub fn new(on: bool) -> Self {
        Self(Rc::new(Cell::new(ToggleState::new(on))))
    }

    pub fn track(&self, on: bool) -> ToggleState {
        let state = self.0.get().set(on);
        self.0.set(state);
        state
    }

    pub fn reset(&self, on: bool) {
        self.0.set(ToggleState::new(on));
    }
}

const SETTLED_AGE: Duration = Duration::from_secs(2);

pub fn ease_out(t: f32) -> f32 {
    let inv = 1.0 - t;
    1.0 - inv * inv * inv
}

pub fn lerp_color(from: Color, to: Color, t: f32) -> Color {
    let a = Color4f::from(from);
    let b = Color4f::from(to);
    Color4f::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
    .to_color()
}

pub fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

pub fn keyframe(t: f32, breakpoints: &[f32], values: &[f32]) -> f32 {
    debug_assert_eq!(breakpoints.len(), values.len());
    debug_assert!(breakpoints.len() >= 2);
    let t = t.clamp(breakpoints[0], breakpoints[breakpoints.len() - 1]);
    for i in 0..breakpoints.len() - 1 {
        if t <= breakpoints[i + 1] {
            let span = breakpoints[i + 1] - breakpoints[i];
            let local_t = if span > 0.0 {
                (t - breakpoints[i]) / span
            } else {
                0.0
            };
            return lerp(values[i], values[i + 1], local_t);
        }
    }
    values[values.len() - 1]
}

pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }

    let sample_x = |p: f32| {
        let inv = 1.0 - p;
        3.0 * inv * inv * p * x1 + 3.0 * inv * p * p * x2 + p * p * p
    };
    let sample_y = |p: f32| {
        let inv = 1.0 - p;
        3.0 * inv * inv * p * y1 + 3.0 * inv * p * p * y2 + p * p * p
    };
    let sample_dx = |p: f32| {
        let inv = 1.0 - p;
        3.0 * inv * inv * x1 + 6.0 * inv * p * (x2 - x1) + 3.0 * p * p * (1.0 - x2)
    };

    let mut p = t;
    for _ in 0..8 {
        let x_err = sample_x(p) - t;
        if x_err.abs() < 1e-5 {
            return sample_y(p);
        }
        let dx = sample_dx(p);
        if dx.abs() < 1e-6 {
            break;
        }
        p -= x_err / dx;
    }

    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    let mut p = t;
    for _ in 0..20 {
        let x_at_p = sample_x(p);
        if (x_at_p - t).abs() < 1e-5 {
            break;
        }
        if x_at_p < t {
            lo = p;
        } else {
            hi = p;
        }
        p = (lo + hi) / 2.0;
    }
    sample_y(p)
}

#[allow(clippy::too_many_arguments)]
pub fn keyframe_eased(
    t: f32,
    breakpoints: &[f32],
    values: &[f32],
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
) -> f32 {
    debug_assert_eq!(breakpoints.len(), values.len());
    debug_assert!(breakpoints.len() >= 2);
    let t = t.clamp(breakpoints[0], breakpoints[breakpoints.len() - 1]);
    for i in 0..breakpoints.len() - 1 {
        if t <= breakpoints[i + 1] {
            let span = breakpoints[i + 1] - breakpoints[i];
            let local_t = if span > 0.0 {
                (t - breakpoints[i]) / span
            } else {
                0.0
            };
            let eased = cubic_bezier(x1, y1, x2, y2, local_t);
            return lerp(values[i], values[i + 1], eased);
        }
    }
    values[values.len() - 1]
}

pub struct ColorTransition {
    target: Cell<Option<Color>>,
    from: Cell<Color>,
    start: Cell<Instant>,
    duration: Duration,
}

impl ColorTransition {
    pub fn new(duration: Duration) -> Self {
        Self {
            target: Cell::new(None),
            from: Cell::new(Color::TRANSPARENT),
            start: Cell::new(Instant::now()),
            duration,
        }
    }

    pub fn animate(&self, wanted: Color) -> Color {
        let now = Instant::now();
        if self.target.get() != Some(wanted) {
            let from = match self.target.get() {
                Some(prev_target) => lerp_color(self.from.get(), prev_target, self.progress(now)),
                None => wanted,
            };
            self.from.set(from);
            self.start.set(now);
            self.target.set(Some(wanted));
        }
        lerp_color(self.from.get(), wanted, self.progress(now))
    }

    fn progress(&self, now: Instant) -> f32 {
        let elapsed = now.duration_since(self.start.get());
        ease_out((elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0))
    }
}

impl Default for ColorTransition {
    fn default() -> Self {
        Self::new(DEFAULT_TRANSITION)
    }
}

pub struct FloatTransition {
    target: Cell<Option<f32>>,
    from: Cell<f32>,
    start: Cell<Instant>,
    duration: Duration,
}

impl FloatTransition {
    pub fn new(duration: Duration) -> Self {
        Self {
            target: Cell::new(None),
            from: Cell::new(0.0),
            start: Cell::new(Instant::now()),
            duration,
        }
    }

    pub fn animate(&self, wanted: f32) -> f32 {
        let now = Instant::now();
        if self.target.get() != Some(wanted) {
            let from = match self.target.get() {
                Some(prev_target) => lerp(self.from.get(), prev_target, self.progress(now)),
                None => wanted,
            };
            self.from.set(from);
            self.start.set(now);
            self.target.set(Some(wanted));
        }
        lerp(self.from.get(), wanted, self.progress(now))
    }

    fn progress(&self, now: Instant) -> f32 {
        let elapsed = now.duration_since(self.start.get());
        ease_out((elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ToggleState {
    pub on: bool,
    pub changed_at: Instant,
}

impl ToggleState {
    pub fn new(on: bool) -> Self {
        let now = Instant::now();
        Self {
            on,
            changed_at: now.checked_sub(SETTLED_AGE).unwrap_or(now),
        }
    }

    pub fn set(self, new_on: bool) -> Self {
        if self.on == new_on {
            self
        } else {
            Self {
                on: new_on,
                changed_at: Instant::now(),
            }
        }
    }

    pub fn progress(&self, duration: Duration) -> f32 {
        let elapsed = self.changed_at.elapsed();
        ease_out((elapsed.as_secs_f32() / duration.as_secs_f32()).min(1.0))
    }
}
