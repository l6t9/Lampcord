//! Material 3 Expressive circular progress indicator.
//!
//! The ring version of [`ProgressBar`](crate::progress_bar::ProgressBar): a filled active
//! arc starting at 12 o'clock, a gap, and the inactive track arc, both with rounded ends.
//! As in the M3 circular indicator there is no stop dot, and the gap is measured along
//! the ring between the two rounded ends.

use gpui::{
    Bounds, IntoElement, PathBuilder, Pixels, Point, Rgba, Styled, Window, canvas, point, px,
};

use crate::progress_bar::ProgressBarColors;
use crate::theme::M3Palette;

/// Layout metrics for progress rings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressRingMetrics {
    /// Outer diameter (default: 40dp, the M3 circular indicator size).
    pub size: f32,
    /// Track thickness (default: 4dp).
    pub thickness: f32,
    /// Gap between the active arc and the track, along the ring (default: 4dp).
    pub gap: f32,
}

impl Default for ProgressRingMetrics {
    fn default() -> Self {
        Self {
            size: 40.0,
            thickness: 4.0,
            gap: 4.0,
        }
    }
}

/// Material 3 circular progress indicator (determinate).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressRing {
    /// Progress fraction in the range 0.0..=1.0.
    pub fraction: f32,
    /// Layout metrics.
    pub metrics: ProgressRingMetrics,
    /// Colors, shared with the linear progress bar.
    pub colors: Option<ProgressBarColors>,
    /// Extra rotation of the start point, in degrees clockwise from 12 o'clock.
    pub rotation: f32,
}

impl ProgressRing {
    /// Create a ring at `fraction` (0.0..=1.0).
    #[must_use]
    pub fn new(fraction: f32) -> Self {
        Self {
            fraction: fraction.clamp(0.0, 1.0),
            metrics: ProgressRingMetrics::default(),
            colors: None,
            rotation: 0.0,
        }
    }

    /// Set custom metrics.
    #[must_use]
    pub fn metrics(mut self, metrics: ProgressRingMetrics) -> Self {
        self.metrics = metrics;
        self
    }

    /// Set the outer diameter.
    #[must_use]
    pub fn size(mut self, size: f32) -> Self {
        self.metrics.size = size;
        self
    }

    /// Set the track thickness.
    #[must_use]
    pub fn thickness(mut self, thickness: f32) -> Self {
        self.metrics.thickness = thickness;
        self
    }

    /// Set the gap between the active arc and the track.
    #[must_use]
    pub fn gap(mut self, gap: f32) -> Self {
        self.metrics.gap = gap;
        self
    }

    /// Set explicit colors.
    #[must_use]
    pub fn colors(mut self, colors: ProgressBarColors) -> Self {
        self.colors = Some(colors);
        self
    }

    /// Rotate the start point (degrees, clockwise), for indeterminate style spinning.
    #[must_use]
    pub fn rotation(mut self, degrees: f32) -> Self {
        self.rotation = degrees;
        self
    }

    /// Render as a square element of `metrics.size`.
    pub fn render(&self, pal: &M3Palette) -> impl IntoElement {
        let ring = *self;
        let colors = self
            .colors
            .unwrap_or_else(|| ProgressBarColors::from_palette(pal));
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| ring.paint(window, bounds, colors),
        )
        .size(px(self.metrics.size))
    }

    /// Paint into `bounds` (centered, using the smaller side).
    pub fn paint(&self, window: &mut Window, bounds: Bounds<Pixels>, colors: ProgressBarColors) {
        let m = self.metrics;
        let side = f32::from(bounds.size.width)
            .min(f32::from(bounds.size.height))
            .min(m.size);
        let center = bounds.center();
        let t = m.thickness.min(side / 2.0);
        let r = (side - t) / 2.0;
        if r <= 0.0 {
            return;
        }
        let tau = std::f32::consts::TAU;
        let start = -tau / 4.0 + self.rotation.to_radians();
        let f = self.fraction;
        // gap along the ring between the rounded ends: cap radius on each side plus the gap
        let gap_angle = (m.gap + t) / r;
        if f <= 0.001 {
            paint_arc(
                window,
                center,
                r,
                t,
                start,
                start + tau,
                colors.inactive_track,
                true,
            );
            return;
        }
        if f >= 0.999 {
            paint_arc(
                window,
                center,
                r,
                t,
                start,
                start + tau,
                colors.active_track,
                true,
            );
            return;
        }
        let active_end = start + tau * f;
        // keep both arcs at least a dot long so tiny values still read
        let min_sweep = 0.0001;
        let active_sweep = (active_end - start).max(min_sweep);
        paint_arc(
            window,
            center,
            r,
            t,
            start,
            start + active_sweep,
            colors.active_track,
            false,
        );
        let track_start = start + active_sweep + gap_angle;
        let track_end = start + tau - gap_angle;
        if track_end > track_start {
            paint_arc(
                window,
                center,
                r,
                t,
                track_start,
                track_end,
                colors.inactive_track,
                false,
            );
        }
    }
}

/// Fills an arc of radius `r` and thickness `t` from `a0` to `a1` (radians, clockwise
/// with y down), with semicircular ends unless it is a closed ring.
fn paint_arc(
    window: &mut Window,
    c: Point<Pixels>,
    r: f32,
    t: f32,
    a0: f32,
    a1: f32,
    color: Rgba,
    closed: bool,
) {
    let (ro, ri) = (r + t / 2.0, r - t / 2.0);
    let at = |rad: f32, a: f32| point(c.x + px(rad * a.cos()), c.y + px(rad * a.sin()));
    let sweep = a1 - a0;
    let steps = ((sweep.abs() * ro / 2.0).ceil() as usize).clamp(8, 720);
    let mut b = PathBuilder::fill();
    if closed {
        // outer circle then inner circle reversed (even-odd leaves the hole)
        b.move_to(at(ro, 0.0));
        for i in 1..=steps {
            b.line_to(at(ro, std::f32::consts::TAU * i as f32 / steps as f32));
        }
        b.close();
        b.move_to(at(ri, 0.0));
        for i in 1..=steps {
            b.line_to(at(ri, -std::f32::consts::TAU * i as f32 / steps as f32));
        }
        b.close();
    } else {
        b.move_to(at(ro, a0));
        for i in 1..=steps {
            b.line_to(at(ro, a0 + sweep * i as f32 / steps as f32));
        }
        // end cap
        let cap_c = at(r, a1);
        for i in 1..=12 {
            let a = a1 + std::f32::consts::PI * i as f32 / 12.0;
            b.line_to(point(
                cap_c.x + px(t / 2.0 * a.cos()),
                cap_c.y + px(t / 2.0 * a.sin()),
            ));
        }
        for i in (0..steps).rev() {
            b.line_to(at(ri, a0 + sweep * i as f32 / steps as f32));
        }
        // start cap
        let cap_c = at(r, a0);
        for i in 1..12 {
            let a = a0 + std::f32::consts::PI + std::f32::consts::PI * i as f32 / 12.0;
            b.line_to(point(
                cap_c.x + px(t / 2.0 * a.cos()),
                cap_c.y + px(t / 2.0 * a.sin()),
            ));
        }
        b.close();
    }
    if let Ok(path) = b.build() {
        window.paint_path(path, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_fraction() {
        assert_eq!(ProgressRing::new(1.4).fraction, 1.0);
        assert_eq!(ProgressRing::new(-1.0).fraction, 0.0);
    }

    #[test]
    fn defaults_match_m3() {
        let r = ProgressRing::new(0.5);
        assert_eq!(r.metrics.size, 40.0);
        assert_eq!(r.metrics.thickness, 4.0);
        assert_eq!(r.metrics.gap, 4.0);
    }
}
