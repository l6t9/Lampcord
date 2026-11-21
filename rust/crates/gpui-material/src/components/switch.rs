//! Material 3 switch (`MaterialSwitch`), as Android's Settings draws it.
//!
//! Specification (material-components-android `Widget.Material3.CompoundButton.MaterialSwitch`):
//! - Track: 52x32dp pill; unselected tracks carry a 2dp `outline` border
//! - Thumb: a circle, 16dp when unselected without an icon, 24dp when selected or when
//!   it shows an icon, 28dp while pressed; it grows around its centre, which travels from
//!   16dp to 36dp
//! - Icon: 16dp check (selected) / close (unselected) drawn in the thumb
//! - State layer: a 40dp circle around the thumb, 0.08 hovered and 0.1 pressed
//! - Damped spring animation for travel and press growth

use gpui::{Bounds, Div, PathBuilder, Pixels, Rgba, Window, canvas, div, point, prelude::*, px};

use crate::motion::{SpringParams, SpringState, spring_step};
use crate::theme::{M3Palette, light_scheme};

/// Geometry matching the Material 3 switch tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitchMetrics {
    /// Track width: 52dp.
    pub track_width: f32,
    /// Track height: 32dp.
    pub track_height: f32,
    /// Outline of an unselected track: 2dp.
    pub track_outline_width: f32,
    /// Thumb diameter when unselected and iconless: 16dp.
    pub thumb_size_unchecked: f32,
    /// Thumb diameter when selected, or unselected with an icon: 24dp.
    pub thumb_size_checked: f32,
    /// Thumb diameter while pressed: 28dp.
    pub thumb_size_pressed: f32,
    /// State layer diameter around the thumb: 40dp.
    pub state_layer_size: f32,
    /// Icon canvas box: 16dp.
    pub icon_size: f32,
    /// Icon stroke thickness: 2dp.
    pub icon_stroke_width: f32,
}

impl Default for SwitchMetrics {
    fn default() -> Self {
        Self {
            track_width: 52.0,
            track_height: 32.0,
            track_outline_width: 2.0,
            thumb_size_unchecked: 16.0,
            thumb_size_checked: 24.0,
            thumb_size_pressed: 28.0,
            state_layer_size: 40.0,
            icon_size: 16.0,
            icon_stroke_width: 2.0,
        }
    }
}

/// Colors resolved from M3 palette tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitchColors {
    /// Track when selected.
    pub track_checked: Rgba,
    /// Track when unselected.
    pub track_unchecked: Rgba,
    /// Border of an unselected track.
    pub outline_unchecked: Rgba,
    /// Thumb when selected.
    pub thumb_checked: Rgba,
    /// Thumb when selected and hovered or pressed.
    pub thumb_checked_active: Rgba,
    /// Thumb when unselected.
    pub thumb_unchecked: Rgba,
    /// Thumb when unselected and hovered or pressed.
    pub thumb_unchecked_active: Rgba,
    /// State layer tint when selected.
    pub state_layer_checked: Rgba,
    /// State layer tint when unselected.
    pub state_layer_unchecked: Rgba,
    /// Check icon.
    pub icon_checked: Rgba,
    /// Close icon.
    pub icon_unchecked: Rgba,
    /// Track when selected and disabled (drawn at 12%).
    pub disabled_track_checked: Rgba,
    /// Track when unselected and disabled (drawn at 12%).
    pub disabled_track_unchecked: Rgba,
    /// Thumb when selected and disabled.
    pub disabled_thumb_checked: Rgba,
    /// Thumb when unselected and disabled (drawn at 38%).
    pub disabled_thumb_unchecked: Rgba,
    /// Check icon when disabled (drawn at 38%).
    pub disabled_icon_checked: Rgba,
    /// Close icon when disabled (drawn at 38%).
    pub disabled_icon_unchecked: Rgba,
}

impl SwitchColors {
    /// The M3 color roles; the same roles serve light and dark schemes.
    #[must_use]
    pub fn from_scheme(p: &M3Palette) -> Self {
        Self {
            track_checked: p.primary,
            track_unchecked: p.surface_container_highest,
            outline_unchecked: p.outline,
            thumb_checked: p.on_primary,
            thumb_checked_active: p.primary_container,
            thumb_unchecked: p.outline,
            thumb_unchecked_active: p.on_surface_variant,
            state_layer_checked: p.primary,
            state_layer_unchecked: p.on_surface,
            icon_checked: p.on_primary_container,
            icon_unchecked: p.surface_container_highest,
            disabled_track_checked: p.on_surface,
            disabled_track_unchecked: p.surface_container_highest,
            disabled_thumb_checked: p.surface,
            disabled_thumb_unchecked: p.on_surface,
            disabled_icon_checked: p.on_surface,
            disabled_icon_unchecked: p.surface_container_highest,
        }
    }

    /// Light color mapping.
    #[must_use]
    pub fn light(p: &M3Palette) -> Self {
        Self::from_scheme(p)
    }

    /// Dark color mapping.
    #[must_use]
    pub fn dark(p: &M3Palette) -> Self {
        Self::from_scheme(p)
    }

    /// Resolve colors from palette and dark mode flag.
    #[must_use]
    pub fn from_palette(p: &M3Palette, _is_dark: bool) -> Self {
        Self::from_scheme(p)
    }
}

impl Default for SwitchColors {
    fn default() -> Self {
        Self::light(&light_scheme())
    }
}

/// A single line segment in a 16x16 icon coordinate space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IconSegment {
    /// Beginning X coordinate in the local 16x16 icon viewport.
    pub start_x: f32,
    /// Beginning Y coordinate in the local 16x16 icon viewport.
    pub start_y: f32,
    /// Ending X coordinate in the local 16x16 icon viewport.
    pub end_x: f32,
    /// Ending Y coordinate in the local 16x16 icon viewport.
    pub end_y: f32,
}

impl IconSegment {
    /// Create a new segment.
    #[must_use]
    pub const fn new(sx: f32, sy: f32, ex: f32, ey: f32) -> Self {
        Self {
            start_x: sx,
            start_y: sy,
            end_x: ex,
            end_y: ey,
        }
    }

    /// Linear interpolation between two segments.
    #[inline]
    #[must_use]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            start_x: self.start_x + (other.start_x - self.start_x) * t,
            start_y: self.start_y + (other.start_y - self.start_y) * t,
            end_x: self.end_x + (other.end_x - self.end_x) * t,
            end_y: self.end_y + (other.end_y - self.end_y) * t,
        }
    }
}

/// The 16x16 icon strokes: Material's `check` when selected, `close` when not. Pressing
/// only grows the thumb; the icon keeps its shape.
#[inline]
#[must_use]
pub fn icon_segments(checked: bool, _pressed: bool) -> (IconSegment, IconSegment) {
    if checked {
        (
            IconSegment::new(2.7, 8.4, 6.0, 11.7),
            IconSegment::new(6.0, 11.7, 13.3, 4.4),
        )
    } else {
        (
            IconSegment::new(3.5, 3.5, 12.5, 12.5),
            IconSegment::new(3.5, 12.5, 12.5, 3.5),
        )
    }
}

/// Icon strokes part way between close (0) and check (1).
#[inline]
#[must_use]
pub fn animated_icon_segments(pos: f32, _press_pos: f32) -> (IconSegment, IconSegment) {
    let (u1, u2) = icon_segments(false, false);
    let (c1, c2) = icon_segments(true, false);
    (u1.lerp(c1, pos), u2.lerp(c2, pos))
}

/// Thumb `(left, top, diameter)` inside the track for travel `pos` (0 off, 1 on) and
/// press `press_pos`; the thumb grows around its centre, which moves 16dp -> 36dp.
#[inline]
#[must_use]
pub fn thumb_geometry(
    metrics: &SwitchMetrics,
    pos: f32,
    press_pos: f32,
    show_icons: bool,
) -> (f32, f32, f32) {
    let off = if show_icons {
        metrics.thumb_size_checked
    } else {
        metrics.thumb_size_unchecked
    };
    let idle = off + (metrics.thumb_size_checked - off) * pos;
    let size = idle + (metrics.thumb_size_pressed - idle) * press_pos;
    let half_h = metrics.track_height / 2.0;
    let cx = half_h + (metrics.track_width - metrics.track_height) * pos;
    (cx - size / 2.0, half_h - size / 2.0, size)
}

/// Draw the 16x16 icon strokes on a GPUI canvas.
pub fn draw_switch_icon(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    seg1: IconSegment,
    seg2: IconSegment,
    color: Rgba,
) {
    let scale = f32::from(bounds.size.width) / 16.0;
    let mut builder = PathBuilder::stroke(px(2.0 * scale));
    let ox = bounds.left();
    let oy = bounds.top();
    let p = |x: f32, y: f32| point(ox + px(x * scale), oy + px(y * scale));
    builder.move_to(p(seg1.start_x, seg1.start_y));
    builder.line_to(p(seg1.end_x, seg1.end_y));
    builder.move_to(p(seg2.start_x, seg2.start_y));
    builder.line_to(p(seg2.end_x, seg2.end_y));
    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

fn mix(a: Rgba, b: Rgba, t: f32) -> Rgba {
    Rgba {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

fn alpha(c: Rgba, a: f32) -> Rgba {
    Rgba { a: c.a * a, ..c }
}

/// State model for an animated switch.
///
/// # Examples
///
/// ```rust,no_run
/// use gpui_material::switch::SwitchModel;
///
/// let mut sw = SwitchModel::new("Dark mode", false);
///
/// // Toggle state on click
/// sw.toggle();
/// ```
#[derive(Clone, Debug)]
pub struct SwitchModel {
    /// Descriptive label text displayed alongside the switch.
    pub label: String,
    /// Whether the switch is currently toggled on.
    pub checked: bool,
    /// Whether user interaction is enabled.
    pub enabled: bool,
    /// Whether to render the morphing checkmark and cross icon lines.
    pub show_icons: bool,
    /// Spring state tracking horizontal thumb travel progress from 0.0 to 1.0.
    pub display_pos: SpringState,
    /// Spring state tracking thumb expansion progress on press from 0.0 to 1.0.
    pub press_pos: SpringState,
    /// Whether the pointer is currently holding down on the switch.
    pub is_pressed: bool,
    /// Whether the pointer is currently hovering over the switch.
    pub is_hovered: bool,
}

impl SwitchModel {
    /// Create a new switch model.
    #[must_use]
    pub fn new(label: impl Into<String>, checked: bool) -> Self {
        let pos = if checked { 1.0 } else { 0.0 };
        Self {
            label: label.into(),
            checked,
            enabled: true,
            show_icons: true,
            display_pos: SpringState { pos, vel: 0.0 },
            press_pos: SpringState { pos: 0.0, vel: 0.0 },
            is_pressed: false,
            is_hovered: false,
        }
    }

    /// Set whether the switch is enabled.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set whether to render the morphing icons inside the thumb.
    #[must_use]
    pub fn show_icons(mut self, show: bool) -> Self {
        self.show_icons = show;
        self
    }

    /// Toggle checked state if enabled.
    pub fn toggle(&mut self) {
        if !self.enabled {
            return;
        }
        self.checked = !self.checked;
    }

    /// Advance spring animations by dt seconds, requesting an animation frame if unsettled.
    pub fn advance_springs(&mut self, window: &mut Window, dt: f32) {
        let target_pos = if self.checked { 1.0 } else { 0.0 };
        let target_press = if self.is_pressed && self.enabled {
            1.0
        } else {
            0.0
        };

        let pos_params = SpringParams::standard();
        let press_params = SpringParams {
            stiffness: 450.0,
            damping: 32.0,
            mass: 1.0,
        };

        let next_pos = spring_step(pos_params, self.display_pos, target_pos, dt);
        let settled_pos = (next_pos.pos - target_pos).abs() < 0.0005 && next_pos.vel.abs() < 0.0005;
        self.display_pos = if settled_pos {
            SpringState {
                pos: target_pos,
                vel: 0.0,
            }
        } else {
            window.request_animation_frame();
            next_pos
        };

        let next_press = spring_step(press_params, self.press_pos, target_press, dt);
        let settled_press =
            (next_press.pos - target_press).abs() < 0.0005 && next_press.vel.abs() < 0.0005;
        self.press_pos = if settled_press {
            SpringState {
                pos: target_press,
                vel: 0.0,
            }
        } else {
            window.request_animation_frame();
            next_press
        };
    }

    /// Resolve display colors for the current state.
    #[must_use]
    pub fn resolve_colors(&self, colors: &SwitchColors) -> (Rgba, Rgba, Rgba, Rgba, f32) {
        let (track_color, thumb_color, icon_color, state_layer_color) = if !self.enabled {
            if self.checked {
                (
                    colors.disabled_track_checked,
                    colors.disabled_thumb_checked,
                    colors.disabled_icon_checked,
                    colors.state_layer_checked,
                )
            } else {
                (
                    colors.disabled_track_unchecked,
                    colors.disabled_thumb_unchecked,
                    colors.disabled_icon_unchecked,
                    colors.state_layer_unchecked,
                )
            }
        } else if self.checked {
            let active = self.is_hovered || self.is_pressed;
            (
                colors.track_checked,
                if active {
                    colors.thumb_checked_active
                } else {
                    colors.thumb_checked
                },
                colors.icon_checked,
                colors.state_layer_checked,
            )
        } else {
            let active = self.is_hovered || self.is_pressed;
            (
                colors.track_unchecked,
                if active {
                    colors.thumb_unchecked_active
                } else {
                    colors.thumb_unchecked
                },
                colors.icon_unchecked,
                colors.state_layer_unchecked,
            )
        };

        let state_layer_opacity = if self.is_pressed && self.enabled {
            0.1f32
        } else if self.is_hovered && self.enabled {
            0.08f32
        } else {
            0.0f32
        };

        (
            track_color,
            thumb_color,
            icon_color,
            state_layer_color,
            state_layer_opacity,
        )
    }

    /// Draws the switch at its current spring state: outlined track, round thumb with its
    /// icon, and the state layer around the thumb. Event wiring is left to the caller, who
    /// sets `is_hovered` / `is_pressed` and calls [`Self::toggle`].
    #[must_use]
    pub fn render(&self, pal: &M3Palette) -> Div {
        let m = SwitchMetrics::default();
        let colors = SwitchColors::from_scheme(pal);
        let pos = self.display_pos.pos.clamp(0.0, 1.0);
        let press = self.press_pos.pos.clamp(0.0, 1.0);
        let (left, top, size) = thumb_geometry(&m, pos, press, self.show_icons);
        let (track, thumb, icon, state, state_opacity) = self.resolve_colors(&colors);
        let enabled = self.enabled;
        // the track fades between its two fills and loses the outline as it turns on
        let track = if enabled {
            mix(colors.track_unchecked, colors.track_checked, pos)
        } else {
            alpha(track, 0.12)
        };
        let outline = alpha(
            if enabled {
                colors.outline_unchecked
            } else {
                alpha(colors.disabled_thumb_unchecked, 0.12)
            },
            1.0 - pos,
        );
        let thumb = if enabled {
            thumb
        } else if self.checked {
            thumb
        } else {
            alpha(thumb, 0.38)
        };
        let icon = if enabled { icon } else { alpha(icon, 0.38) };
        let (seg1, seg2) = animated_icon_segments(pos, press);
        let icon_opacity = if self.show_icons { 1.0 } else { pos };
        let layer = m.state_layer_size;
        let cx = left + size / 2.0;
        let cy = top + size / 2.0;
        let icon_box = m.icon_size * (size / m.thumb_size_checked).min(1.0);
        div()
            .flex_none()
            .relative()
            .w(px(m.track_width))
            .h(px(m.track_height))
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .rounded_full()
                    .bg(track)
                    .border(px(m.track_outline_width))
                    .border_color(outline),
            )
            .child(
                div()
                    .absolute()
                    .left(px(cx - layer / 2.0))
                    .top(px(cy - layer / 2.0))
                    .size(px(layer))
                    .rounded_full()
                    .bg(alpha(state, if enabled { state_opacity } else { 0.0 })),
            )
            .child(
                div()
                    .absolute()
                    .left(px(left))
                    .top(px(top))
                    .size(px(size))
                    .rounded_full()
                    .bg(thumb)
                    .child(
                        canvas(
                            move |b, _, _| b,
                            move |_, b, window, _| {
                                draw_switch_icon(window, b, seg1, seg2, alpha(icon, icon_opacity))
                            },
                        )
                        .absolute()
                        .left(px((size - icon_box) / 2.0))
                        .top(px((size - icon_box) / 2.0))
                        .size(px(icon_box)),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_match_m3_tokens() {
        let m = SwitchMetrics::default();
        assert_eq!(
            (m.track_width, m.track_height, m.track_outline_width),
            (52.0, 32.0, 2.0)
        );
        assert_eq!(
            (
                m.thumb_size_unchecked,
                m.thumb_size_checked,
                m.thumb_size_pressed
            ),
            (16.0, 24.0, 28.0)
        );
        assert_eq!(m.icon_size, 16.0);
    }

    #[test]
    fn thumb_is_a_circle_centred_on_its_travel() {
        let m = SwitchMetrics::default();
        // off without icon: 16dp circle centred at (16, 16)
        assert_eq!(thumb_geometry(&m, 0.0, 0.0, false), (8.0, 8.0, 16.0));
        // off with icon: 24dp
        assert_eq!(thumb_geometry(&m, 0.0, 0.0, true), (4.0, 4.0, 24.0));
        // on: 24dp centred at (36, 16)
        assert_eq!(thumb_geometry(&m, 1.0, 0.0, false), (24.0, 4.0, 24.0));
        // pressed: 28dp, 2dp from the track edge
        assert_eq!(thumb_geometry(&m, 1.0, 1.0, true), (22.0, 2.0, 28.0));
        assert_eq!(thumb_geometry(&m, 0.0, 1.0, false), (2.0, 2.0, 28.0));
    }

    #[test]
    fn icon_keeps_its_shape_when_pressed() {
        assert_eq!(icon_segments(true, true), icon_segments(true, false));
        assert_eq!(icon_segments(false, true), icon_segments(false, false));
    }
}
