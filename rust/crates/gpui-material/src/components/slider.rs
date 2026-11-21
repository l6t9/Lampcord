//! Material 3 Expressive slider specification, metrics, and colors.
//!
//! Provides metrics and color configurations for continuous and discrete sliders,
//! including track dimensions, thumb dimensions, and tick marks.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gpui::{
    App, Context, IntoElement, MouseButton, ParentElement, Render, Rgba, SharedString, Stateful,
    Styled, Window, div, prelude::*, px,
};

use crate::motion::{SpringParams, SpringState, spring_step};
use crate::theme::{M3Palette, light_scheme};

/// Available slider configurations matching Material 3 Expressive and Quickshell tokens.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SliderConfiguration {
    /// 12dp track height, 6dp corner radius.
    ExtraSmall,
    /// 18dp track height, 6dp corner radius.
    Small,
    /// 30dp track height, 9dp corner radius (Standard Control Center configuration).
    #[default]
    Medium,
    /// 42dp track height, 12dp corner radius.
    Large,
    /// 72dp track height, 21dp corner radius.
    ExtraLarge,
}

impl SliderConfiguration {
    /// Height of the slider track.
    #[must_use]
    pub fn track_height(&self) -> f32 {
        match self {
            Self::ExtraSmall => 12.0,
            Self::Small => 18.0,
            Self::Medium => 30.0,
            Self::Large => 42.0,
            Self::ExtraLarge => 72.0,
        }
    }

    /// Outer corner radius of the slider track.
    #[must_use]
    pub fn track_radius(&self) -> f32 {
        match self {
            Self::ExtraSmall | Self::Small => 6.0,
            Self::Medium => 9.0,
            Self::Large => 12.0,
            Self::ExtraLarge => 21.0,
        }
    }

    /// Height of the vertical draggable handle bar.
    #[must_use]
    pub fn handle_height(&self) -> f32 {
        (self.track_height() + 9.0).max(33.0)
    }

    /// Width of the vertical handle when idle.
    #[must_use]
    pub fn handle_width_idle(&self) -> f32 {
        4.0
    }

    /// Width of the vertical handle when pressed.
    #[must_use]
    pub fn handle_width_pressed(&self) -> f32 {
        1.5
    }

    /// Gap margin between the handle and active/inactive track portions.
    #[must_use]
    pub fn handle_gap(&self) -> f32 {
        4.0
    }

    /// Unsharpened corner radius near the handle cutout.
    #[must_use]
    pub fn unsharpen_radius(&self) -> f32 {
        2.0
    }

    /// Diameter of track indicator dots.
    #[must_use]
    pub fn dot_size(&self) -> f32 {
        3.0
    }
}

/// Layout metrics and dimensions for sliders.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderMetrics {
    /// Inactive and active track height, 16dp.
    pub track_height: f32,
    /// Handle width, 4dp idle, 2dp pressed.
    pub thumb_w: f32,
    /// Handle height, 44dp.
    pub thumb_h: f32,
    /// Gap between thumb and track, 4dp (12dp cutout).
    pub thumb_gap: f32,
    /// Stop indicator size, 4dp.
    pub tick_size: f32,
}

impl Default for SliderMetrics {
    fn default() -> Self {
        Self {
            track_height: 16.0,
            thumb_w: 4.0,
            thumb_h: 44.0,
            thumb_gap: 4.0,
            tick_size: 4.0,
        }
    }
}

/// Colors resolved from `SliderDefaults.colors()`.
/// Color mapping for Material 3 Slider components.
///
/// Active maps to `Primary`, inactive to `SecondaryContainer`,
/// disabled uses on-surface with 0.38 and 0.12 opacity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderColors {
    /// Pill-shaped draggable handle fill color.
    pub thumb: Rgba,
    /// Filled track portion between leading edge and current thumb position.
    pub active_track: Rgba,
    /// Unfilled track portion between thumb position and trailing edge.
    pub inactive_track: Rgba,
    /// Discrete indicator dots on the active track section.
    pub active_tick: Rgba,
    /// Discrete indicator dots on the inactive track section.
    pub inactive_tick: Rgba,
    /// Thumb fill color when the slider is disabled.
    pub disabled_thumb: Rgba,
    /// Active track fill color when the slider is disabled.
    pub disabled_active_track: Rgba,
    /// Inactive track fill color when the slider is disabled.
    pub disabled_inactive_track: Rgba,
}

impl SliderColors {
    /// Light defaults from the current palette.
    #[must_use]
    pub fn light(p: &M3Palette) -> Self {
        Self::from_palette(p)
    }

    /// Dark defaults from the current palette.
    #[must_use]
    pub fn dark(p: &M3Palette) -> Self {
        Self::from_palette(p)
    }

    /// Resolve colors from any M3 palette (light or dark).
    #[must_use]
    pub fn from_palette(p: &M3Palette) -> Self {
        Self {
            thumb: p.primary,
            active_track: p.primary,
            inactive_track: p.secondary_container,
            active_tick: p.secondary_container,
            inactive_tick: p.on_secondary_container,
            disabled_thumb: p.on_surface,
            disabled_active_track: p.on_surface,
            disabled_inactive_track: p.on_surface,
        }
    }
}

impl Default for SliderColors {
    fn default() -> Self {
        Self::light(&light_scheme())
    }
}

/// Value range and stepping configuration for continuous or discrete sliders.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderSpec {
    /// Lower bound of the interactive range.
    pub min: f32,
    /// Upper bound of the interactive range.
    pub max: f32,
    /// Number of discrete intervals between minimum and maximum, or zero for continuous.
    pub steps: u32,
}

impl SliderSpec {
    /// Create a spec, coerces inverted ranges.
    #[must_use]
    pub fn new(min: f32, max: f32, steps: u32) -> Self {
        if max >= min {
            Self { min, max, steps }
        } else {
            Self {
                min: max,
                max: min,
                steps,
            }
        }
    }

    /// Fraction 0..=1 for a value, coerced to range.
    #[inline]
    #[must_use]
    pub fn fraction(&self, value: f32) -> f32 {
        let clamped = value.clamp(self.min, self.max);
        if self.max <= self.min {
            return 0.0;
        }
        (clamped - self.min) / (self.max - self.min)
    }

    /// Tick fractions including ends, empty when continuous.
    #[inline]
    #[must_use]
    pub fn tick_fractions(&self) -> impl ExactSizeIterator<Item = f32> + DoubleEndedIterator + '_ {
        let denom = if self.steps == 0 { 1 } else { self.steps + 1 };
        let count = if self.steps == 0 { 0 } else { denom + 1 };
        (0..count).map(move |i| i as f32 / denom as f32)
    }

    /// Snap a fraction to the nearest tick in O(1) arithmetic.
    #[inline]
    #[must_use]
    pub fn snap_fraction(&self, frac: f32) -> f32 {
        if self.steps == 0 {
            return frac.clamp(0.0, 1.0);
        }
        let denom = (self.steps + 1) as f32;
        let clamped = frac.clamp(0.0, 1.0);
        let nearest_idx = (clamped * denom).round();
        nearest_idx / denom
    }

    /// Value from a 0..=1 fraction, with snap when stepped.
    #[inline]
    #[must_use]
    pub fn value_from_fraction(&self, frac: f32) -> f32 {
        let f = if self.steps == 0 {
            frac.clamp(0.0, 1.0)
        } else {
            self.snap_fraction(frac)
        };
        self.min + f * (self.max - self.min)
    }

    /// Apply a drag delta in px over a track width in px.
    #[inline]
    #[must_use]
    pub fn drag_by_px(&self, current: f32, delta_px: f32, track_w_px: f32) -> f32 {
        if track_w_px <= 0.0 {
            return current;
        }
        let cur_frac = self.fraction(current);
        let next_frac = cur_frac + delta_px / track_w_px;
        self.value_from_fraction(next_frac)
    }
}

/// Drag gesture payload carrying slider instance ID.
#[derive(Clone)]
pub struct SliderDragPayload(pub usize);

/// Empty drag renderable to eliminate ghost visuals during slider drag.
pub struct EmptySliderDrag;

impl Render for EmptySliderDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_0()
    }
}

/// Material 3 Expressive Slider model.
#[derive(Clone, Debug)]
pub struct SliderModel {
    /// Instance identifier.
    pub id: usize,
    /// Value range specification.
    pub spec: SliderSpec,
    /// Current slider value.
    pub value: f32,
    /// Size configuration token.
    pub configuration: SliderConfiguration,
    /// Whether the slider accepts input.
    pub enabled: bool,
    /// Optional embedded icon displayed inside the slider track.
    pub icon: Option<SharedString>,
    /// Stop indicator dot positions in normalized 0.0..=1.0 values.
    pub stop_indicators: Vec<f32>,
    /// Custom color overrides.
    pub custom_colors: Option<SliderColors>,
    /// Optional tooltip override string.
    pub tooltip: Option<SharedString>,
    /// Spring state for animated fractional position.
    pub display: SpringState,
    /// Spring state for pressed tactile response.
    pub press_state: SpringState,
    /// Spring state for hover state layer.
    pub hover_state: SpringState,
}

/// Persistent state across render passes for a slider.
#[derive(Clone, Debug)]
pub struct SliderPersistentState {
    /// Measured bounds (origin X and width in px).
    pub bounds: Option<(f32, f32)>,
    /// Whether the slider is currently being dragged or pressed.
    pub is_dragging: bool,
    /// Temporary live value during active user drag.
    pub live_value: Option<f32>,
    /// Animated fractional position (0.0 ..= 1.0).
    pub display_pos: SpringState,
    /// Animated handle width in pixels.
    pub handle_width: SpringState,
    /// Whether the initial position has been established.
    pub initialized: bool,
    /// Timestamp of last animation tick.
    pub last_tick: Option<std::time::Instant>,
    pub moving: bool,
    pub drawn: Option<(gpui::WindowId, std::time::Instant)>,
}

impl Default for SliderPersistentState {
    fn default() -> Self {
        Self {
            bounds: None,
            is_dragging: false,
            live_value: None,
            display_pos: SpringState { pos: 0.0, vel: 0.0 },
            handle_width: SpringState { pos: 4.0, vel: 0.0 },
            initialized: false,
            last_tick: None,
            moving: false,
            drawn: None,
        }
    }
}

thread_local! {
    static SLIDER_STATES: RefCell<HashMap<usize, SliderPersistentState>> =
        RefCell::new(HashMap::new());
}

impl SliderModel {
    /// Create a new slider model with defaults.
    #[must_use]
    pub fn new(id: usize) -> Self {
        Self {
            id,
            spec: SliderSpec::new(0.0, 1.0, 0),
            value: 0.0,
            configuration: SliderConfiguration::Medium,
            enabled: true,
            icon: None,
            stop_indicators: vec![1.0],
            custom_colors: None,
            tooltip: None,
            display: SpringState::default(),
            press_state: SpringState::default(),
            hover_state: SpringState::default(),
        }
    }

    /// Set value specification.
    #[must_use]
    pub fn spec(mut self, spec: SliderSpec) -> Self {
        self.spec = spec;
        self
    }

    /// Set the current value.
    #[must_use]
    pub fn value(mut self, val: f32) -> Self {
        self.value = val;
        let frac = self.spec.fraction(val);
        if (self.display.pos - frac).abs() > 0.5 {
            self.display.pos = frac;
        }
        self
    }

    /// Set slider size configuration.
    #[must_use]
    pub fn configuration(mut self, config: SliderConfiguration) -> Self {
        self.configuration = config;
        self
    }

    /// Set whether the slider is enabled.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set the embedded Material icon.
    #[must_use]
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set stop indicator positions.
    #[must_use]
    pub fn stop_indicators(mut self, stops: Vec<f32>) -> Self {
        self.stop_indicators = stops;
        self
    }

    /// Set custom color overrides.
    #[must_use]
    pub fn colors(mut self, colors: SliderColors) -> Self {
        self.custom_colors = Some(colors);
        self
    }

    /// Set custom tooltip text.
    #[must_use]
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Resolves colors for the slider using the provided palette.
    #[must_use]
    pub fn resolve_colors(&self, pal: &M3Palette) -> SliderColors {
        self.custom_colors
            .unwrap_or_else(|| SliderColors::from_palette(pal))
    }

    /// Step spring animations by `dt` seconds. Returns `true` if still animating.
    pub fn step(&mut self, dt: f32) -> bool {
        let frac = self.spec.fraction(self.value);
        let spring = SpringParams::expressive();

        self.display = spring_step(spring, self.display, frac, dt);
        let settled = self.display.is_settled(frac, 0.001, 0.002);
        !settled
    }

    /// Advance spring animations by `dt` seconds, requesting an animation frame if unsettled.
    pub fn advance_springs(&mut self, window: &mut Window, dt: f32) {
        if self.step(dt) {
            window.request_animation_frame();
        }
    }

    /// Render the slider with an interactive change handler.
    pub fn render<F>(
        &self,
        window: &mut Window,
        pal: &M3Palette,
        on_change: F,
    ) -> Stateful<gpui::Div>
    where
        F: Fn(f32, &mut Window, &mut App) + 'static,
    {
        self.render_with_icon_click(window, pal, on_change, None::<fn(&mut Window, &mut App)>)
    }

    /// Render the slider with interactive change handler and optional clickable icon handler.
    pub fn render_with_icon_click<F, I>(
        &self,
        window: &mut Window,
        pal: &M3Palette,
        on_change: F,
        on_icon_click: Option<I>,
    ) -> Stateful<gpui::Div>
    where
        F: Fn(f32, &mut Window, &mut App) + 'static,
        I: Fn(&mut Window, &mut App) + 'static,
    {
        let config = self.configuration;
        let track_h = config.track_height();
        let track_rad = config.track_radius();
        let handle_h = config.handle_height();
        let handle_w_idle = config.handle_width_idle();
        let handle_w_pressed = config.handle_width_pressed();
        let handle_gap = config.handle_gap();
        let unsharpen_rad = config.unsharpen_radius();
        let dot_size = config.dot_size();

        let colors = self.resolve_colors(pal);
        let slider_id = self.id;
        let value_frac = self.spec.fraction(self.value).clamp(0.0, 1.0);

        let (saved_bounds, is_dragging, frac, current_handle_w) = SLIDER_STATES.with(|states| {
            let mut map = states.borrow_mut();
            let st = map.entry(slider_id).or_default();

            let window_id = window.window_handle().window_id();
            let now_draw = std::time::Instant::now();
            let shown = st.drawn.is_some_and(|(w, t)| {
                w == window_id && now_draw.duration_since(t).as_millis() < 150
            });
            st.drawn = Some((window_id, now_draw));
            if !shown {
                st.is_dragging = false;
                st.live_value = None;
            }

            let target_frac = if let Some(live) = st.live_value {
                self.spec.fraction(live).clamp(0.0, 1.0)
            } else {
                value_frac
            };

            let now = std::time::Instant::now();
            if !st.initialized || !shown {
                st.display_pos = SpringState {
                    pos: target_frac,
                    vel: 0.0,
                };
                st.handle_width = SpringState {
                    pos: handle_w_idle,
                    vel: 0.0,
                };
                st.initialized = true;
                st.moving = false;
            }
            if !st.moving {
                st.last_tick = Some(now);
            }

            let elapsed = st
                .last_tick
                .map(|t| (now - t).as_secs_f32())
                .unwrap_or(1.0 / 60.0)
                .min(0.1)
                .max(0.0);
            st.last_tick = Some(now);

            let target_w = if st.is_dragging {
                handle_w_pressed
            } else {
                handle_w_idle
            };

            let max_step = 1.0 / 120.0;
            let steps = ((elapsed / max_step).ceil() as usize).max(1);
            let dt = elapsed / steps as f32;

            let pos_spring = SpringParams::standard();
            let width_spring = SpringParams::standard();

            for _ in 0..steps {
                st.display_pos = spring_step(pos_spring, st.display_pos, target_frac, dt);
                st.handle_width = spring_step(width_spring, st.handle_width, target_w, dt);
            }

            let pos_settled = st.display_pos.is_settled(target_frac, 0.0005, 0.001);
            let width_settled = st.handle_width.is_settled(target_w, 0.02, 0.05);

            if pos_settled {
                st.display_pos = SpringState {
                    pos: target_frac,
                    vel: 0.0,
                };
            }
            if width_settled {
                st.handle_width = SpringState {
                    pos: target_w,
                    vel: 0.0,
                };
            }

            st.moving = !pos_settled || !width_settled;
            if st.moving {
                window.request_animation_frame();
            }

            (
                st.bounds,
                st.is_dragging,
                st.display_pos.pos.clamp(0.0, 1.0),
                st.handle_width.pos.clamp(1.0, 10.0),
            )
        });

        let total_h = handle_h.max(track_h + 8.0);
        let track_top = (total_h - track_h) / 2.0;
        let handle_top = (total_h - handle_h) / 2.0;

        let (_, track_w) = saved_bounds.unwrap_or((0.0, 370.0));
        let effective_w = (track_w - handle_gap * 2.0).max(1.0);
        let handle_cx = handle_gap + frac * effective_w;
        let handle_left = handle_cx - current_handle_w / 2.0;

        let active_w = (handle_cx - handle_gap - current_handle_w / 2.0).max(0.0);
        let inactive_left = handle_cx + handle_gap + current_handle_w / 2.0;
        let inactive_w = (track_w - inactive_left).max(0.0);

        let stop_x = track_w - 8.0;
        let stop_color = if frac >= 0.98 {
            colors.active_tick
        } else {
            colors.inactive_tick
        };

        let tip_str = self
            .tooltip
            .clone()
            .unwrap_or_else(|| SharedString::from(format!("{:.0}%", frac * 100.0)));

        let current_val = self.value;
        let spec = self.spec;
        let on_change_rc = Rc::new(on_change);
        let on_change_down = on_change_rc.clone();
        let on_change_move = on_change_rc.clone();
        let on_change_up = on_change_rc.clone();
        let on_change_out = on_change_rc.clone();
        let on_change_drag = on_change_rc.clone();

        let on_icon_click_rc = on_icon_click.map(Rc::new);

        div()
            .id(("m3-slider", self.id))
            .relative()
            .w_full()
            .h(px(total_h))
            .cursor_pointer()
            // Canvas for accurate layout measurements
            .child(
                gpui::canvas(
                    move |bounds, _, _| {
                        SLIDER_STATES.with(|states| {
                            let mut map = states.borrow_mut();
                            let st = map.entry(slider_id).or_default();
                            st.bounds =
                                Some((f32::from(bounds.origin.x), f32::from(bounds.size.width)));
                        });
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
            // Left active track
            .when(active_w > 0.5, |d| {
                d.child(
                    div()
                        .absolute()
                        .top(px(track_top))
                        .left(px(0.0))
                        .w(px(active_w))
                        .h(px(track_h))
                        .rounded_l(px(track_rad))
                        .rounded_r(px(unsharpen_rad))
                        .bg(colors.active_track),
                )
            })
            // Right inactive track
            .when(inactive_w > 0.5, |d| {
                d.child(
                    div()
                        .absolute()
                        .top(px(track_top))
                        .left(px(inactive_left))
                        .w(px(inactive_w))
                        .h(px(track_h))
                        .rounded_l(px(unsharpen_rad))
                        .rounded_r(px(track_rad))
                        .bg(colors.inactive_track),
                )
            })
            // Stop indicator dot: omitted if embedded icon is present
            .when(self.icon.is_none() && inactive_w > 12.0, |d| {
                d.child(
                    div()
                        .absolute()
                        .top(px(track_top + (track_h - dot_size) / 2.0))
                        .left(px(stop_x - dot_size / 2.0))
                        .size(px(dot_size))
                        .rounded_full()
                        .bg(stop_color),
                )
            })
            // Draggable handle bar: width smoothly narrows when grabbed
            .child(
                div()
                    .absolute()
                    .top(px(handle_top))
                    .left(px(handle_left))
                    .w(px(current_handle_w))
                    .h(px(handle_h))
                    .rounded_full()
                    .bg(colors.thumb),
            )
            // Embedded Material Icon
            .children(self.icon.clone().map(|icon_glyph| {
                let (icon_color, is_left, icon_x) = if frac >= 0.9 {
                    (pal.on_primary, true, (handle_left - 14.0 - 20.0).max(8.0))
                } else {
                    (pal.on_secondary_container, false, 8.0)
                };

                let mut icon_div = div()
                    .absolute()
                    .top(px((total_h - 20.0) / 2.0))
                    .when(is_left, |d| d.left(px(icon_x)))
                    .when(!is_left, |d| d.right(px(icon_x)))
                    .size(px(20.0))
                    .flex()
                    .items_center()
                    .justify_center();

                if let Some(ref on_click) = on_icon_click_rc {
                    let on_click = on_click.clone();
                    icon_div = icon_div.cursor_pointer().on_mouse_down(
                        MouseButton::Left,
                        move |_, window, cx| {
                            cx.stop_propagation();
                            on_click(window, cx);
                        },
                    );
                }

                icon_div.child(
                    div()
                        .font_family("Material Symbols Rounded")
                        .text_size(px(20.0))
                        .text_color(icon_color)
                        .child(icon_glyph),
                )
            }))
            // Tooltip when dragging
            .when(is_dragging, |d| {
                d.child(
                    div()
                        .absolute()
                        .top(px(-24.0))
                        .left(px((handle_cx - 24.0).max(0.0)))
                        .w(px(48.0))
                        .h(px(22.0))
                        .flex()
                        .justify_center()
                        .items_center()
                        .rounded_full()
                        .bg(pal.inverse_surface)
                        .text_color(pal.inverse_on_surface)
                        .font_family("Google Sans Flex Body")
                        .text_size(px(11.0))
                        .child(tip_str),
                )
            })
            // Interactive mouse down & seek (glide smoothly on click)
            .on_mouse_down(MouseButton::Left, {
                let on_change = on_change_down.clone();
                move |e, window, cx| {
                    cx.stop_propagation();
                    let saved_b = SLIDER_STATES.with(|states| {
                        let mut map = states.borrow_mut();
                        let st = map.entry(slider_id).or_default();
                        st.is_dragging = true;
                        st.bounds
                    });
                    let (left, w) = saved_b.unwrap_or((0.0, 370.0));
                    let avail_w = (w - handle_gap * 2.0).max(1.0);
                    let frac =
                        ((f32::from(e.position.x) - left - handle_gap) / avail_w).clamp(0.0, 1.0);
                    let val = spec.value_from_fraction(frac);
                    SLIDER_STATES.with(|states| {
                        if let Some(st) = states.borrow_mut().get_mut(&slider_id) {
                            st.live_value = Some(val);
                            // Do NOT snap st.display_pos.pos = frac here;
                            // smoothly glide to target fraction with spring physics!
                        }
                    });
                    on_change(val, window, cx);
                }
            })
            // Scrub move while mouse is pressed
            .on_mouse_move({
                let on_change = on_change_move.clone();
                move |e, window, cx| {
                    let is_drag = SLIDER_STATES.with(|states| {
                        states
                            .borrow()
                            .get(&slider_id)
                            .map(|st| st.is_dragging)
                            .unwrap_or(false)
                    });
                    if e.pressed_button == Some(MouseButton::Left) && is_drag {
                        let saved_b = SLIDER_STATES.with(|states| {
                            states.borrow().get(&slider_id).and_then(|st| st.bounds)
                        });
                        let (left, w) = saved_b.unwrap_or((0.0, 370.0));
                        let avail_w = (w - handle_gap * 2.0).max(1.0);
                        let frac = ((f32::from(e.position.x) - left - handle_gap) / avail_w)
                            .clamp(0.0, 1.0);
                        let val = spec.value_from_fraction(frac);
                        SLIDER_STATES.with(|states| {
                            if let Some(st) = states.borrow_mut().get_mut(&slider_id) {
                                st.live_value = Some(val);
                                st.display_pos.pos = frac;
                                st.display_pos.vel = 0.0;
                            }
                        });
                        on_change(val, window, cx);
                    } else if e.pressed_button != Some(MouseButton::Left) && is_drag {
                        let cur_val = SLIDER_STATES.with(|states| {
                            if let Some(st) = states.borrow_mut().get_mut(&slider_id) {
                                st.is_dragging = false;
                                st.live_value.take()
                            } else {
                                None
                            }
                        });
                        on_change(cur_val.unwrap_or(current_val), window, cx);
                    }
                }
            })
            // Release mouse
            .on_mouse_up(MouseButton::Left, {
                let on_change = on_change_up.clone();
                move |_, window, cx| {
                    let cur_val = SLIDER_STATES.with(|states| {
                        if let Some(st) = states.borrow_mut().get_mut(&slider_id) {
                            st.is_dragging = false;
                            st.live_value.take()
                        } else {
                            None
                        }
                    });
                    on_change(cur_val.unwrap_or(current_val), window, cx);
                }
            })
            .on_mouse_up_out(MouseButton::Left, {
                let on_change = on_change_out.clone();
                move |_, window, cx| {
                    let cur_val = SLIDER_STATES.with(|states| {
                        if let Some(st) = states.borrow_mut().get_mut(&slider_id) {
                            st.is_dragging = false;
                            st.live_value.take()
                        } else {
                            None
                        }
                    });
                    on_change(cur_val.unwrap_or(current_val), window, cx);
                }
            })
            // Full drag support
            .on_drag(SliderDragPayload(self.id), |_, _, _, cx| {
                cx.new(|_| EmptySliderDrag)
            })
            .on_drag_move({
                let on_change = on_change_drag.clone();
                move |event: &gpui::DragMoveEvent<SliderDragPayload>, window, cx| {
                    if event.drag(cx).0 != slider_id {
                        return;
                    }
                    let left = f32::from(event.bounds.origin.x);
                    let w = f32::from(event.bounds.size.width);
                    if w > 0.0 {
                        let avail_w = (w - handle_gap * 2.0).max(1.0);
                        let frac = ((f32::from(event.event.position.x) - left - handle_gap)
                            / avail_w)
                            .clamp(0.0, 1.0);
                        let val = spec.value_from_fraction(frac);
                        SLIDER_STATES.with(|states| {
                            let mut map = states.borrow_mut();
                            let st = map.entry(slider_id).or_default();
                            st.is_dragging = true;
                            st.bounds = Some((left, w));
                            st.live_value = Some(val);
                            st.display_pos.pos = frac;
                            st.display_pos.vel = 0.0;
                        });
                        on_change(val, window, cx);
                    }
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continuous_fraction_roundtrip() {
        let s = SliderSpec::new(0.0, 100.0, 0);
        assert_eq!(s.fraction(25.0), 0.25);
        assert!((s.value_from_fraction(0.25) - 25.0).abs() < 1e-5);
    }

    #[test]
    fn stepped_snaps_to_nearest_tick() {
        let s = SliderSpec::new(0.0, 10.0, 4);
        assert!((s.value_from_fraction(0.21) - 2.0).abs() < 1e-4);
        assert!((s.value_from_fraction(0.62) - 6.0).abs() < 1e-4);
    }

    #[test]
    fn drag_moves_proportionally() {
        let s = SliderSpec::new(0.0, 1.0, 0);
        let next = s.drag_by_px(0.5, 48.0, 480.0);
        assert!((next - 0.6).abs() < 1e-5);
    }

    #[test]
    fn metrics_match_slider_tokens() {
        let m = SliderMetrics::default();
        assert_eq!(m.track_height, 16.0);
        assert_eq!(m.thumb_w, 4.0);
        assert_eq!(m.thumb_h, 44.0);
        assert_eq!(m.thumb_gap, 4.0);
    }

    #[test]
    fn slider_model_defaults() {
        let model = SliderModel::new(1);
        assert_eq!(model.configuration, SliderConfiguration::Medium);
        assert_eq!(model.configuration.track_height(), 30.0);
        assert_eq!(model.configuration.handle_height(), 39.0);
        assert_eq!(model.configuration.track_radius(), 9.0);
        assert_eq!(model.configuration.handle_gap(), 4.0);
    }
}
