//! Material 3 Expressive motion engine.
//!
//! Provides harmonic oscillator spring physics with expressive (zeta ≈ 0.56)
//! and standard (zeta ≈ 0.8) damping profiles.

/// Spring parameters for damped harmonic oscillation: stiffness `k`, damping `c`, and mass `m`.
///
/// Underdamped expressive profile has damping ratio `zeta` near 0.56 with visible overshoot.
/// Standard profile has `zeta` near 0.8 for smooth settling without significant bounce.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringParams {
    /// Restoring force coefficient `k` in Newtons per meter.
    pub stiffness: f32,
    /// Damping friction coefficient `c` dissipating kinetic energy over time.
    pub damping: f32,
    /// Oscillator point mass `m` in kilograms.
    pub mass: f32,
}

impl SpringParams {
    /// Constructs a spring parameter profile from natural frequency (rad/s) and damping ratio zeta.
    ///
    /// - `zeta < 1.0` produces underdamped harmonic oscillation with expressive overshoot.
    /// - `zeta == 1.0` produces critically damped motion without overshoot.
    /// - `zeta > 1.0` produces overdamped motion.
    #[must_use]
    pub fn from_frequency_and_damping(natural_frequency: f32, damping_ratio: f32) -> Self {
        Self {
            stiffness: natural_frequency * natural_frequency,
            damping: 2.0 * damping_ratio * natural_frequency,
            mass: 1.0,
        }
    }

    /// Quickshell `expressiveDefaultSpatial`: 500ms duration, 21% overshoot.
    /// Used for sidebar sliding, bottom widget height, modal dialogs, and drawer expands.
    #[must_use]
    pub fn expressive_default_spatial() -> Self {
        Self {
            stiffness: 428.0,
            damping: 18.4,
            mass: 1.0,
        }
    }

    /// Quickshell `expressiveFastSpatial`: 350ms duration, springy snap.
    /// Used for toggle button tap bounces and fast interactive pops.
    #[must_use]
    pub fn expressive_fast_spatial() -> Self {
        Self {
            stiffness: 700.0,
            damping: 26.5,
            mass: 1.0,
        }
    }

    /// Quickshell `emphasized`: 400ms duration, critically damped (zeta = 1.0, ZERO BOUNCE).
    /// Used for workspace active indicator leading edge, notification dismissals, and gliding transitions.
    #[must_use]
    pub fn emphasized() -> Self {
        Self {
            stiffness: 144.0,
            damping: 24.0,
            mass: 1.0,
        }
    }

    /// Quickshell `emphasized` trailing: 800ms duration (normal * 2), critically damped (zeta = 1.0, ZERO BOUNCE).
    /// Used for workspace active indicator trailing edge to create the signature Material 3 trailing stretch.
    #[must_use]
    pub fn emphasized_trailing() -> Self {
        Self {
            stiffness: 49.0,
            damping: 14.0,
            mass: 1.0,
        }
    }

    /// Quickshell `expressiveDefaultEffects`: 200ms duration, critically damped (zeta = 1.0, ZERO BOUNCE).
    /// Used for smooth opacity fades. Fading never overshoots or vibrates.
    #[must_use]
    pub fn default_effects() -> Self {
        Self {
            stiffness: 400.0,
            damping: 40.0,
            mass: 1.0,
        }
    }

    /// Quickshell `expressiveSlowEffects` (CAnim): 300ms duration, critically damped (zeta = 1.0, ZERO BOUNCE).
    /// Used for smooth color crossfades and toggle background morphing.
    #[must_use]
    pub fn slow_effects() -> Self {
        Self {
            stiffness: 180.0,
            damping: 27.0,
            mass: 1.0,
        }
    }

    /// Quickshell expressive default spatial profile alias.
    #[must_use]
    pub fn expressive() -> Self {
        Self::expressive_default_spatial()
    }

    /// Quickshell standard emphasized profile alias.
    #[must_use]
    pub fn standard() -> Self {
        Self::emphasized()
    }
}

/// Evaluator for exact cubic bezier curves matching Quickshell Caelestia animation tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezier {
    /// First control point X coordinate.
    pub x1: f32,
    /// First control point Y coordinate.
    pub y1: f32,
    /// Second control point X coordinate.
    pub x2: f32,
    /// Second control point Y coordinate.
    pub y2: f32,
}

impl CubicBezier {
    /// Creates a cubic bezier with control points (x1, y1) and (x2, y2).
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// Quickshell `Tokens.anim.expressiveDefaultSpatial`: cubic bezier (0.38, 1.21, 0.22, 1.0).
    pub const fn expressive_default_spatial() -> Self {
        Self::new(0.38, 1.21, 0.22, 1.0)
    }

    /// Quickshell `Tokens.anim.expressiveFastSpatial`: cubic bezier (0.42, 1.67, 0.21, 0.9).
    pub const fn expressive_fast_spatial() -> Self {
        Self::new(0.42, 1.67, 0.21, 0.9)
    }

    /// Quickshell `Tokens.anim.emphasizedDecel`: cubic bezier (0.05, 0.7, 0.1, 1.0).
    pub const fn emphasized_decel() -> Self {
        Self::new(0.05, 0.7, 0.1, 1.0)
    }

    /// Quickshell `Tokens.anim.emphasizedAccel`: cubic bezier (0.3, 0.0, 0.8, 0.15).
    pub const fn emphasized_accel() -> Self {
        Self::new(0.3, 0.0, 0.8, 0.15)
    }

    /// Quickshell `Tokens.anim.expressiveDefaultEffects`: cubic bezier (0.34, 0.8, 0.34, 1.0).
    pub const fn default_effects() -> Self {
        Self::new(0.34, 0.8, 0.34, 1.0)
    }

    /// Quickshell `Tokens.anim.expressiveSlowEffects`: cubic bezier (0.34, 0.88, 0.34, 1.0).
    pub const fn slow_effects() -> Self {
        Self::new(0.34, 0.88, 0.34, 1.0)
    }

    #[inline]
    fn sample_x(&self, t: f32) -> f32 {
        let inv = 1.0 - t;
        3.0 * inv * inv * t * self.x1 + 3.0 * inv * t * t * self.x2 + t * t * t
    }

    #[inline]
    fn sample_x_derivative(&self, t: f32) -> f32 {
        let inv = 1.0 - t;
        3.0 * inv * inv * self.x1
            + 6.0 * inv * t * (self.x2 - self.x1)
            + 3.0 * t * t * (1.0 - self.x2)
    }

    #[inline]
    fn sample_y(&self, t: f32) -> f32 {
        let inv = 1.0 - t;
        3.0 * inv * inv * t * self.y1 + 3.0 * inv * t * t * self.y2 + t * t * t
    }

    /// Evaluates y at normalized time progress t in [0.0, 1.0].
    #[must_use]
    pub fn evaluate(&self, t: f32) -> f32 {
        if t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return 1.0;
        }

        // Newton-Raphson iteration
        let mut s = t;
        for _ in 0..8 {
            let x_err = self.sample_x(s) - t;
            if x_err.abs() < 1e-5 {
                return self.sample_y(s);
            }
            let d = self.sample_x_derivative(s);
            if d.abs() < 1e-6 {
                break;
            }
            s = (s - x_err / d).clamp(0.0, 1.0);
        }

        // Binary search fallback
        let mut low = 0.0;
        let mut high = 1.0;
        for _ in 0..10 {
            let mid = (low + high) * 0.5;
            let x = self.sample_x(mid);
            if (x - t).abs() < 1e-5 {
                return self.sample_y(mid);
            }
            if x < t {
                low = mid;
            } else {
                high = mid;
            }
        }
        self.sample_y((low + high) * 0.5)
    }
}

/// Time-based animation driven by a cubic bezier easing curve matching Quickshell Caelestia.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BezierAnimation {
    /// Starting value of the animation.
    pub start_val: f32,
    /// Destination target value.
    pub target_val: f32,
    /// Time when the transition began.
    pub start_time: std::time::Instant,
    /// Duration of the transition.
    pub duration: std::time::Duration,
    /// Cubic bezier easing curve.
    pub curve: CubicBezier,
}

impl BezierAnimation {
    /// Creates a new bezier transition from start_val to target_val over duration.
    #[must_use]
    pub fn new(
        start_val: f32,
        target_val: f32,
        duration: std::time::Duration,
        curve: CubicBezier,
    ) -> Self {
        Self {
            start_val,
            target_val,
            start_time: std::time::Instant::now(),
            duration,
            curve,
        }
    }

    /// Evaluates the animated value at the current instant.
    #[must_use]
    pub fn value(&self) -> f32 {
        let total = self.duration.as_secs_f32();
        if total <= 0.0 {
            return self.target_val;
        }
        let elapsed = self.start_time.elapsed().as_secs_f32();
        if elapsed >= total {
            return self.target_val;
        }
        let p = (elapsed / total).clamp(0.0, 1.0);
        let y = self.curve.evaluate(p);
        self.start_val + (self.target_val - self.start_val) * y
    }

    /// Returns true if the animation duration has elapsed.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.start_time.elapsed() >= self.duration
    }
}

/// Spring state tracking position and velocity for one animated axis.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct SpringState {
    /// Current scalar displacement from the zero origin.
    pub pos: f32,
    /// Current instantaneous velocity along the displacement axis.
    pub vel: f32,
}

impl SpringState {
    /// Returns true if the spring position and velocity are settled within the given thresholds.
    #[inline]
    #[must_use]
    pub fn is_settled(&self, target: f32, pos_epsilon: f32, vel_epsilon: f32) -> bool {
        (self.pos - target).abs() <= pos_epsilon && self.vel.abs() <= vel_epsilon
    }
}

/// Advance a damped spring toward target by dt seconds.
/// Semi-implicit Euler, stable for our stiffness at 60Hz.
#[inline]
#[must_use]
pub fn spring_step(params: SpringParams, state: SpringState, target: f32, dt: f32) -> SpringState {
    let mass = params.mass.max(1e-4);
    let force = -params.stiffness * (state.pos - target) - params.damping * state.vel;
    let vel = state.vel + force / mass * dt;
    let pos = state.pos + vel * dt;
    SpringState { pos, vel }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_step_advances_toward_target() {
        let params = SpringParams::expressive();
        let state = SpringState { pos: 0.0, vel: 0.0 };
        let next = spring_step(params, state, 1.0, 1.0 / 60.0);
        assert!(next.pos > 0.0);
        assert!(next.vel > 0.0);
    }

    #[test]
    fn test_cubic_bezier_expressive_default_spatial() {
        let curve = CubicBezier::expressive_default_spatial();
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            println!("default t = {:.1}: y = {:.4}", t, curve.evaluate(t));
        }
        let fast = CubicBezier::expressive_fast_spatial();
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            println!("fast t = {:.1}: y = {:.4}", t, fast.evaluate(t));
        }
    }
}
