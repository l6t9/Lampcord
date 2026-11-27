use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

pub const WHEEL_NOTCH_PX: f32 = 100.0;

static SMOOTH_SCROLLING: AtomicBool = AtomicBool::new(true);

pub fn smooth_scrolling() -> bool {
    SMOOTH_SCROLLING.load(Ordering::Relaxed)
}

pub fn set_smooth_scrolling(on: bool) {
    SMOOTH_SCROLLING.store(on, Ordering::Relaxed);
}

const DURATION_DIVISOR: f64 = 60.0;
const INVERSE_DELTA_RAMP_START_PX: f64 = 120.0;
const INVERSE_DELTA_RAMP_END_PX: f64 = 480.0;
const INVERSE_DELTA_MIN_DURATION: f64 = 6.0;
const INVERSE_DELTA_MAX_DURATION: f64 = 12.0;
const INVERSE_DELTA_SLOPE: f64 = (INVERSE_DELTA_MIN_DURATION - INVERSE_DELTA_MAX_DURATION)
    / (INVERSE_DELTA_RAMP_END_PX - INVERSE_DELTA_RAMP_START_PX);
const INVERSE_DELTA_OFFSET: f64 =
    INVERSE_DELTA_MAX_DURATION - INVERSE_DELTA_RAMP_START_PX * INVERSE_DELTA_SLOPE;
const EPSILON: f64 = 0.01;

const EASE_IN_OUT: (f64, f64, f64, f64) = (0.42, 0.0, 0.58, 1.0);

fn segment_duration(delta: f64) -> f64 {
    let frames = (INVERSE_DELTA_OFFSET + delta.abs() * INVERSE_DELTA_SLOPE)
        .clamp(INVERSE_DELTA_MIN_DURATION, INVERSE_DELTA_MAX_DURATION);
    frames / DURATION_DIVISOR
}

fn velocity_bound(velocity: f64, new_delta: f64) -> f64 {
    if new_delta.abs() < EPSILON {
        return 0.0;
    }
    if velocity.abs() < EPSILON {
        return f64::INFINITY;
    }
    let bound = new_delta / velocity * 2.5;
    if bound < 0.0 { f64::INFINITY } else { bound }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Bezier {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

impl Bezier {
    fn sample(a: f64, b: f64, t: f64) -> f64 {
        let c = 3.0 * a;
        let bb = 3.0 * (b - a) - c;
        let aa = 1.0 - c - bb;
        ((aa * t + bb) * t + c) * t
    }

    fn derivative(a: f64, b: f64, t: f64) -> f64 {
        let c = 3.0 * a;
        let bb = 3.0 * (b - a) - c;
        let aa = 1.0 - c - bb;
        (3.0 * aa * t + 2.0 * bb) * t + c
    }

    fn solve_x(&self, x: f64) -> f64 {
        let mut t = x;
        for _ in 0..8 {
            let err = Self::sample(self.x1, self.x2, t) - x;
            if err.abs() < 1e-7 {
                return t;
            }
            let d = Self::derivative(self.x1, self.x2, t);
            if d.abs() < 1e-6 {
                break;
            }
            t -= err / d;
        }
        let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
        t = x.clamp(0.0, 1.0);
        while lo < hi {
            let v = Self::sample(self.x1, self.x2, t);
            if (v - x).abs() < 1e-7 {
                return t;
            }
            if x > v {
                lo = t;
            } else {
                hi = t;
            }
            t = (hi - lo) / 2.0 + lo;
            if hi - lo < 1e-9 {
                break;
            }
        }
        t
    }

    fn value(&self, x: f64) -> f64 {
        Self::sample(self.y1, self.y2, self.solve_x(x.clamp(0.0, 1.0)))
    }

    fn slope(&self, x: f64) -> f64 {
        let t = self.solve_x(x.clamp(0.0, 1.0));
        let dx = Self::derivative(self.x1, self.x2, t);
        let dy = Self::derivative(self.y1, self.y2, t);
        if dx == 0.0 && dy == 0.0 { 0.0 } else { dy / dx }
    }
}

#[derive(Clone, Debug)]
pub struct WheelAnimation {
    initial: f64,
    target: f64,
    start: Instant,

    total: f64,

    last_retarget: f64,
    curve: Bezier,
}

impl WheelAnimation {
    pub fn new(current: f32, target: f32, now: Instant) -> Self {
        let (initial, target) = (f64::from(current), f64::from(target));
        let (x1, y1, x2, y2) = EASE_IN_OUT;
        Self {
            initial,
            target,
            start: now,
            total: segment_duration(target - initial),
            last_retarget: 0.0,
            curve: Bezier { x1, y1, x2, y2 },
        }
    }

    fn t(&self, now: Instant) -> f64 {
        now.saturating_duration_since(self.start).as_secs_f64()
    }

    pub fn target(&self) -> f32 {
        self.target as f32
    }

    pub fn finished(&self, now: Instant) -> bool {
        self.t(now) >= self.total
    }

    pub fn value(&self, now: Instant) -> f32 {
        self.value_at(self.t(now)) as f32
    }

    fn value_at(&self, t: f64) -> f64 {
        let duration = self.total - self.last_retarget;
        let t = t - self.last_retarget;
        if duration <= 0.0 || t >= duration {
            return self.target;
        }
        if t <= 0.0 {
            return self.initial;
        }
        let progress = self.curve.value(t / duration);
        self.initial + (self.target - self.initial) * progress
    }

    fn velocity(&self, t: f64) -> f64 {
        let duration = self.total - self.last_retarget;
        let slope = self.curve.slope((t - self.last_retarget) / duration);
        slope * ((self.target - self.initial) / duration)
    }

    pub fn retarget(&mut self, now: Instant, new_target: f32) {
        let new_target = f64::from(new_target);
        let t = self.t(now).max(self.last_retarget);

        if (self.target - new_target).abs() < EPSILON {
            self.target = new_target;
            return;
        }

        let current = self.value_at(t);
        let new_delta = new_target - current;
        if new_delta.abs() < EPSILON {
            self.last_retarget = t;
            self.total = t;
            self.target = new_target;
            return;
        }

        let old_duration = self.total - self.last_retarget;
        if old_duration <= 0.0 {
            self.total = segment_duration(new_delta);
            self.target = new_target;
            return;
        }

        let velocity = self.velocity(t);
        let new_duration = segment_duration(new_delta).min(velocity_bound(velocity, new_delta));
        if new_duration < EPSILON {
            self.target = new_target;
            self.total = t;
            return;
        }

        let slope = (velocity * (new_duration / new_delta)).clamp(-1000.0, 1000.0);
        let (x1, _, x2, y2) = EASE_IN_OUT;
        self.curve = Bezier {
            x1,
            y1: x1 * slope,
            x2,
            y2,
        };
        self.initial = current;
        self.target = new_target;
        self.total = t + new_duration;
        self.last_retarget = t;
    }
}

const SPRING_TENSION: f32 = 200.0;
const SPRING_FRICTION: f32 = 35.0;
const SPRING_MASS: f32 = 2.0;
const SPRING_THRESHOLD: f32 = 0.001;
const SPRING_STEP: f32 = 1.0 / 240.0;

#[derive(Clone, Copy, Debug)]
pub struct SpringScroll {
    from: f32,
    target: f32,
    vel: f32,
    accumulator: f32,
    last: Option<Instant>,
}

impl SpringScroll {
    pub fn new(from: f32, target: f32) -> Self {
        Self {
            from,
            target,
            vel: 0.0,
            accumulator: 0.0,
            last: None,
        }
    }

    pub fn target(&self) -> f32 {
        self.target
    }

    pub fn clamp_target(&mut self, max: f32) {
        self.target = self.target.clamp(0.0, max.max(0.0));
    }

    fn updates(&self, vel: f32, from: f32) -> (f32, f32, f32) {
        let accel = (-SPRING_TENSION * (from - self.target) + -SPRING_FRICTION * vel) / SPRING_MASS;
        let vel = vel + accel * SPRING_STEP;
        (from + vel * SPRING_STEP, vel, accel)
    }

    pub fn advance(&mut self, now: Instant) -> (f32, bool) {
        let Some(last) = self.last.replace(now) else {
            return (self.from, false);
        };
        self.accumulator =
            ((now.saturating_duration_since(last)).as_secs_f32() + self.accumulator).min(2.0);
        while self.accumulator > SPRING_STEP {
            self.accumulator -= SPRING_STEP;
            let (from, vel, accel) = self.updates(self.vel, self.from);
            self.vel = vel;
            let crossed = from == self.target
                || (from < self.target && self.from > self.target)
                || (from > self.target && self.from < self.target);
            if crossed || (accel * SPRING_STEP).abs() < SPRING_THRESHOLD {
                self.from = self.target;
                return (self.target, true);
            }
            self.from = from;
        }
        let mut shown = self.from;
        if self.accumulator > 0.0 {
            let (next, _, _) = self.updates(self.vel, shown);
            shown += (next - shown) * (self.accumulator / SPRING_STEP);
        }
        (shown, false)
    }
}

thread_local! {
    static PENDING: RefCell<HashMap<String, f32>> = RefCell::new(HashMap::new());
    static PROGRAMMATIC: Cell<bool> = const { Cell::new(false) };
}

pub fn scroll_to(identity: &str, offset: f32) {
    PENDING.with(|pending| pending.borrow_mut().insert(identity.to_string(), offset));
}

pub(crate) fn take_scroll_to(identity: &str) -> Option<f32> {
    PENDING.with(|pending| pending.borrow_mut().remove(identity))
}

pub fn is_programmatic() -> bool {
    PROGRAMMATIC.with(|flag| flag.get())
}

pub(crate) fn set_programmatic(on: bool) {
    PROGRAMMATIC.with(|flag| flag.set(on));
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn at(start: Instant, ms: u64) -> Instant {
        start + Duration::from_millis(ms)
    }

    #[test]
    fn the_spring_settles_on_its_target_without_overshooting() {
        let start = Instant::now();
        let mut spring = SpringScroll::new(0.0, 1000.0);
        assert_eq!(spring.advance(start), (0.0, false));
        let mut peak = 0.0_f32;
        let mut done_at = None;
        for ms in (8..4000).step_by(8) {
            let (value, done) = spring.advance(at(start, ms));
            peak = peak.max(value);
            if done {
                assert_eq!(value, 1000.0);
                done_at = Some(ms);
                break;
            }
        }
        assert!(peak <= 1000.0, "clamped: never past the target ({peak})");
        let done_at = done_at.expect("it arrives");
        assert!(
            (200..2500).contains(&done_at),
            "a smooth glide, not a jump or a crawl ({done_at}ms)"
        );
    }

    #[test]
    fn one_notch_takes_200ms_and_eases_in_and_out() {
        let start = Instant::now();
        let a = WheelAnimation::new(0.0, 100.0, start);
        assert_eq!(a.value(start), 0.0);
        assert!(
            (a.value(at(start, 100)) - 50.0).abs() < 0.01,
            "the symmetric curve is halfway at half the time"
        );

        assert!(a.value(at(start, 50)) < 25.0 && a.value(at(start, 150)) > 75.0);
        assert!(!a.finished(at(start, 199)));
        assert!(a.finished(at(start, 200)));
        assert_eq!(a.value(at(start, 200)), 100.0);
        assert_eq!(a.value(at(start, 5000)), 100.0);
    }

    #[test]
    fn distance_shortens_the_duration_down_to_100ms() {
        let ms = |delta: f64| (segment_duration(delta) * 1000.0).round() as i64;
        assert_eq!(ms(40.0), 200);
        assert_eq!(ms(120.0), 200);
        assert_eq!(ms(300.0), 150);
        assert_eq!(ms(480.0), 100);
        assert_eq!(ms(5000.0), 100);
        assert_eq!(ms(-300.0), 150, "direction does not matter");
    }

    #[test]
    fn scrolling_up_mirrors_scrolling_down() {
        let start = Instant::now();
        let down = WheelAnimation::new(0.0, 100.0, start);
        let up = WheelAnimation::new(500.0, 400.0, start);
        for ms in [20, 60, 100, 140, 180] {
            assert!((down.value(at(start, ms)) - (500.0 - up.value(at(start, ms)))).abs() < 0.001);
        }
    }

    #[test]
    fn a_second_notch_continues_from_where_the_first_is_without_a_jump() {
        let start = Instant::now();
        let mut a = WheelAnimation::new(0.0, 100.0, start);
        let before = a.value(at(start, 80));
        a.retarget(at(start, 80), 200.0);
        let after = a.value(at(start, 80));
        assert!(
            (before - after).abs() < 0.01,
            "position is continuous across the retarget ({before} -> {after})"
        );
        assert_eq!(a.target(), 200.0);
        assert_eq!(a.value(at(start, 5000)), 200.0, "it ends on the new target");
        assert!(a.value(at(start, 400)) <= 200.0 && a.value(at(start, 400)) > 100.0);
    }

    #[test]
    fn a_retarget_keeps_the_velocity() {
        let start = Instant::now();
        let mut a = WheelAnimation::new(0.0, 100.0, start);

        let speed = |a: &WheelAnimation, ms: u64| {
            (a.value(at(start, ms + 1)) - a.value(at(start, ms))) as f64
        };
        let before = speed(&a, 79);
        a.retarget(at(start, 80), 200.0);
        let after = speed(&a, 80);
        assert!(before > 0.3, "mid-animation speed is real ({before})");
        assert!(
            (after - before).abs() / before < 0.25,
            "velocity carries over: {before} px/ms before, {after} after"
        );
    }

    #[test]
    fn a_notch_against_the_motion_turns_around_instead_of_jumping() {
        let start = Instant::now();
        let mut a = WheelAnimation::new(0.0, 100.0, start);
        let before = a.value(at(start, 100));
        a.retarget(at(start, 100), 0.0);
        assert!((a.value(at(start, 100)) - before).abs() < 0.01);
        assert_eq!(a.value(at(start, 5000)), 0.0);

        for ms in (100..1000).step_by(10) {
            let v = a.value(at(start, ms));
            assert!((-0.5..=100.5).contains(&v), "{v} at {ms}ms");
        }
    }

    #[test]
    fn retargeting_to_the_same_place_changes_nothing() {
        let start = Instant::now();
        let mut a = WheelAnimation::new(0.0, 100.0, start);
        let v = a.value(at(start, 60));
        a.retarget(at(start, 60), 100.0);
        assert_eq!(a.value(at(start, 60)), v);
        assert!(a.finished(at(start, 200)));
    }
}
