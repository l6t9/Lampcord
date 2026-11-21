//! Material 3 Expressive Progress Bar component.
//!
//! Directly based on the Material 3 Expressive slider specification with the draggable
//! handle/thumb removed. Consists of a filled active track, a gap, an unfilled inactive track,
//! and a stop indicator point dot.

use gpui::prelude::FluentBuilder;
use gpui::{DefiniteLength, IntoElement, ParentElement, Rgba, Styled, div, px};

use crate::theme::{M3Palette, light_scheme};

/// Layout metrics and dimensions for progress bars.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressBarMetrics {
    /// Track height (default: 4.0dp).
    pub track_height: f32,
    /// Gap between active and inactive track portions, and stop indicator size (default: 4.0dp).
    pub gap: f32,
    /// Corner radius for track ends (default: 2.0dp).
    pub corner_radius: f32,
}

impl Default for ProgressBarMetrics {
    fn default() -> Self {
        Self {
            track_height: 4.0,
            gap: 4.0,
            corner_radius: 2.0,
        }
    }
}

/// Color mapping for Material 3 Progress Bar components.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressBarColors {
    /// Filled track portion and stop dot color.
    pub active_track: Rgba,
    /// Unfilled track portion color.
    pub inactive_track: Rgba,
}

impl ProgressBarColors {
    /// Resolve colors from any M3 palette (light or dark).
    #[must_use]
    pub fn from_palette(p: &M3Palette) -> Self {
        Self {
            active_track: p.primary,
            inactive_track: p.secondary_container,
        }
    }
}

impl Default for ProgressBarColors {
    fn default() -> Self {
        Self::from_palette(&light_scheme())
    }
}

/// Material 3 Progress Bar component (linear indicator without a draggable handle).
///
/// Directly derived from the Material 3 Expressive slider specification with the thumb/handle removed.
/// Features a leading active track, a 4dp gap, an unfilled trailing track, and a 4dp stop indicator dot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressBar {
    /// Progress fraction in the range 0.0..=1.0.
    pub fraction: f32,
    /// Layout metrics.
    pub metrics: ProgressBarMetrics,
    /// Colors.
    pub colors: Option<ProgressBarColors>,
}

impl ProgressBar {
    /// Create a new progress bar with a given progress fraction (0.0..=1.0).
    #[must_use]
    pub fn new(fraction: f32) -> Self {
        Self {
            fraction: fraction.clamp(0.0, 1.0),
            metrics: ProgressBarMetrics::default(),
            colors: None,
        }
    }

    /// Set custom metrics.
    #[must_use]
    pub fn metrics(mut self, metrics: ProgressBarMetrics) -> Self {
        self.metrics = metrics;
        self
    }

    /// Set custom track height.
    #[must_use]
    pub fn track_height(mut self, height: f32) -> Self {
        self.metrics.track_height = height;
        self.metrics.corner_radius = height / 2.0;
        self
    }

    /// Set custom gap size.
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

    /// Render the progress bar as a GPUI element given an M3 palette.
    pub fn render(&self, pal: &M3Palette) -> impl IntoElement {
        let colors = self
            .colors
            .unwrap_or_else(|| ProgressBarColors::from_palette(pal));
        let m = self.metrics;
        let fraction = self.fraction.clamp(0.0, 1.0);

        let active_color = colors.active_track;
        let inactive_color = colors.inactive_track;

        div()
            .relative()
            .w_full()
            .h(px(m.track_height))
            .flex()
            .flex_row()
            .items_center()
            // 1. Active track (fraction of width, fully rounded capsule ends)
            .when(fraction > 0.001, |el| {
                el.child(
                    div()
                        .h_full()
                        .w(DefiniteLength::Fraction(fraction))
                        .rounded_full()
                        .bg(active_color),
                )
            })
            // 2. Gap between active and inactive tracks (only when both exist)
            .when(fraction > 0.001 && fraction < 0.999, |el| {
                el.child(div().w(px(m.gap)).h_full())
            })
            // 3. Inactive track filling the remainder, containing the stop dot at far right
            .when(fraction < 0.999, |el| {
                el.child(
                    div()
                        .flex_1()
                        .h_full()
                        .relative()
                        .rounded_full()
                        .bg(inactive_color)
                        // Stop point dot at far right, inside the inactive track
                        .child(
                            div()
                                .absolute()
                                .right(px(0.0))
                                .top(px(0.0))
                                .size(px(m.gap))
                                .rounded_full()
                                .bg(active_color),
                        ),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_defaults() {
        let pb = ProgressBar::new(0.42);
        assert_eq!(pb.fraction, 0.42);
        assert_eq!(pb.metrics.track_height, 4.0);
        assert_eq!(pb.metrics.gap, 4.0);
    }

    #[test]
    fn test_progress_bar_clamp() {
        let pb_neg = ProgressBar::new(-0.5);
        assert_eq!(pb_neg.fraction, 0.0);
        let pb_over = ProgressBar::new(1.5);
        assert_eq!(pb_over.fraction, 1.0);
    }
}
