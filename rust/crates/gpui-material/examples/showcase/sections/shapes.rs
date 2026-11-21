//! Section: 35 Expressive Shapes & Spring Morpher Playground.
//! Spring-driven morphing plus the looping loading indicator, sharing one card.

use gpui::{Context, IntoElement, ParentElement, Pixels, Styled, Window, div, prelude::*, px};
use gpui_material::{
    M3Palette, MaterialShape, SHAPE_SAMPLE_COUNT, SliderSpec, SpringParams, SpringState,
    damping_from_ratio, draw_morph_with_bounce, draw_points, draw_shape,
    draw_shape_morph_with_bounce, lerp_and_rotate_points, spring_step,
};

use crate::Showcase;
use crate::sections::sliders::{SliderId, SliderModel};

/// Stiffness matching the platform indeterminate indicator motion.
pub(crate) const ANDROID_STIFFNESS: f32 = 200.0;
/// Damping ratio matching the platform indeterminate indicator motion.
pub(crate) const ANDROID_DAMPING_RATIO: f32 = 0.6;
/// Damping ratio for the smoother preset (same speed, less overshoot).
pub(crate) const SMOOTHER_DAMPING_RATIO: f32 = 0.7;
/// Seconds per shape morph in the loading loop (official AndroidX spec: 650ms).
pub(crate) const LOADING_INTERVAL: f32 = 0.650;
/// Showcase loop interval: slightly calmer dwell.
pub(crate) const LOADING_INTERVAL_SMOOTHER: f32 = 0.850;
/// Seconds per full global rotation of the loading loop (official AndroidX spec: 4.666s).
pub(crate) const LOADING_ROTATION_PERIOD: f32 = 4.666;
/// Showcase global rotation period: slightly calmer spin.
pub(crate) const LOADING_ROTATION_PERIOD_SMOOTHER: f32 = 6.0;
/// Degrees added to the rotation target on every completed morph (official AndroidX spec: 90°).
pub(crate) const LOADING_QUARTER_TURN: f32 = 90.0;
/// Showcase turn per morph.
pub(crate) const LOADING_QUARTER_TURN_SMOOTHER: f32 = 60.0;
/// Bounce scale for the platform loop (matching AndroidX / Dim's m3-shapes).
pub(crate) const LOADING_BOUNCE: f32 = 0.14;
/// Bounce scale for the showcase loop: subtler pop.
pub(crate) const LOADING_BOUNCE_SMOOTHER: f32 = 0.08;

/// Seven-shape loop from the platform indeterminate indicator spec.
pub(crate) const LOADING_SEQUENCE: [MaterialShape; 7] = [
    MaterialShape::SoftBurst,
    MaterialShape::Cookie9Sided,
    MaterialShape::Pentagon,
    MaterialShape::Pill,
    MaterialShape::Sunny,
    MaterialShape::Cookie4Sided,
    MaterialShape::Oval,
];

/// State model for the 35 Expressive Shapes & Spring Morpher playground.
pub(crate) struct ShapePlaygroundModel {
    /// Currently selected and displayed target shape.
    pub(crate) current: MaterialShape,
    /// Previously settled shape before the current morph began.
    pub(crate) previous: MaterialShape,
    /// Captured vertex coordinates when morphing was interrupted mid-flight.
    pub(crate) morph_intermediate_from: Option<[[f32; 2]; SHAPE_SAMPLE_COUNT]>,
    /// Coerced morph geometry progress (0.0 -> 1.0); overshoot only scales.
    pub(crate) morph_t: f32,
    /// Harmonic oscillator spring tracking physical bounce and settlement.
    pub(crate) morph_spring: SpringState,
    /// Stiffness snapshot taken when the current morph started.
    pub(crate) morph_stiffness: f32,
    /// Damping ratio snapshot taken when the current morph started.
    pub(crate) morph_damping_ratio: f32,
    /// Stiffness slider (restoring force coefficient k, Android default: 200.0).
    pub(crate) stiffness_slider: SliderModel,
    /// Damping ratio slider (zeta, Android default: 0.60).
    pub(crate) damping_slider: SliderModel,
    /// False = exact platform motion, true = smoother (higher damping, longer dwell).
    pub(crate) loading_smoother: bool,
    /// Index into [`LOADING_SEQUENCE`] for the morph currently on screen.
    pub(crate) loading_index: usize,
    /// Spring driving the active loading morph geometry (0.0 -> 1.0).
    pub(crate) loading_progress: SpringState,
    /// Seconds elapsed since the active loading morph started.
    pub(crate) loading_elapsed: f32,
    /// Accumulated quarter-turn target, advanced per morph.
    pub(crate) loading_quarter: f32,
    /// Continuous global rotation in degrees, looping smoothly.
    pub(crate) loading_rotation: f32,
    /// Timestamp of previous loading frame to ensure real wall-clock cadence.
    pub(crate) last_loading_tick: Option<std::time::Instant>,
}

impl Default for ShapePlaygroundModel {
    fn default() -> Self {
        Self {
            current: MaterialShape::Arch,
            previous: MaterialShape::Arch,
            morph_intermediate_from: None,
            morph_t: 1.0,
            morph_spring: SpringState { pos: 1.0, vel: 0.0 },
            morph_stiffness: ANDROID_STIFFNESS,
            morph_damping_ratio: ANDROID_DAMPING_RATIO,
            stiffness_slider: SliderModel::new(
                "Stiffness (k)",
                SliderSpec::new(100.0, 1000.0, 0),
                ANDROID_STIFFNESS,
            )
            .track_w(340.0),
            damping_slider: SliderModel::new(
                "Damping Ratio (ζ)",
                SliderSpec::new(0.10, 1.50, 0),
                ANDROID_DAMPING_RATIO,
            )
            .track_w(340.0),
            loading_smoother: true,
            loading_index: 0,
            loading_progress: SpringState { pos: 0.0, vel: 0.0 },
            loading_elapsed: 0.0,
            loading_quarter: 90.0,
            loading_rotation: 0.0,
            last_loading_tick: None,
        }
    }
}

impl ShapePlaygroundModel {
    /// Spring parameters for the active loading morph.
    fn loading_params(&self) -> SpringParams {
        let zeta = if self.loading_smoother {
            SMOOTHER_DAMPING_RATIO
        } else {
            ANDROID_DAMPING_RATIO
        };
        SpringParams {
            stiffness: ANDROID_STIFFNESS,
            damping: damping_from_ratio(ANDROID_STIFFNESS, zeta),
            mass: 1.0,
        }
    }

    /// Interval per loading morph in seconds.
    fn loading_interval(&self) -> f32 {
        if self.loading_smoother {
            LOADING_INTERVAL_SMOOTHER
        } else {
            LOADING_INTERVAL
        }
    }

    /// Global rotation period in seconds.
    fn loading_rotation_period(&self) -> f32 {
        if self.loading_smoother {
            LOADING_ROTATION_PERIOD_SMOOTHER
        } else {
            LOADING_ROTATION_PERIOD
        }
    }

    /// Degrees added to the rotation target on every completed morph.
    fn loading_turn(&self) -> f32 {
        if self.loading_smoother {
            LOADING_QUARTER_TURN_SMOOTHER
        } else {
            LOADING_QUARTER_TURN
        }
    }

    /// Overshoot-to-scale factor for the loading loop.
    fn loading_bounce(&self) -> f32 {
        if self.loading_smoother {
            LOADING_BOUNCE_SMOOTHER
        } else {
            LOADING_BOUNCE
        }
    }
}

/// Draw an indeterminate loading indicator shape morph with rotation,
/// AndroidX/Dim breathing scale pulse, and spring overshoot bounce.
#[allow(clippy::too_many_arguments)]
fn draw_loading_shape(
    window: &mut Window,
    bounds: gpui::Bounds<Pixels>,
    from_s: MaterialShape,
    to_s: MaterialShape,
    t: f32,
    angle: f32,
    spring: f32,
    bounce: f32,
    color: gpui::Rgba,
) {
    let pts = lerp_and_rotate_points(from_s.points(), to_s.points(), t, angle);
    // Dim's & AndroidX breathing pulse: expands smoothly at the midpoint of morph
    let pulse = (t.clamp(0.0, 1.0) * std::f32::consts::PI).sin() * 0.14;
    // Spring overshoot bounce:
    let overshoot = (spring - t.clamp(0.0, 1.0)).clamp(-0.5, 0.5);
    // Base scale of ~0.78 matches AndroidX ActiveIndicatorScale (38dp / 48dp)
    // to keep rotating lobes (Sunny, SoftBurst) safely within the container.
    let base_scale = 0.78;
    let scale = (1.0 + pulse + overshoot * bounce) * base_scale;

    use gpui::{point, size};
    let cx = bounds.left() + bounds.size.width / 2.0;
    let cy = bounds.top() + bounds.size.height / 2.0;
    let sw = bounds.size.width * scale;
    let sh = bounds.size.height * scale;
    let target = gpui::Bounds {
        origin: point(cx - sw / 2.0, cy - sh / 2.0),
        size: size(sw, sh),
    };
    draw_points(window, target, &pts, color);
}

impl Showcase {
    /// Advance the manual morph and physics slider springs by one fixed step.
    /// Halts animation frame requests when settled, satisfying the zero-idle power budget.
    pub(crate) fn advance_shapes_springs(&mut self, window: &mut Window, dt: f32) {
        self.shapes.stiffness_slider.advance_springs(window, dt);
        self.shapes.damping_slider.advance_springs(window, dt);

        // Manual morph: geometry tracks the coerced spring position so overshoot
        // only affects draw scale (never morphs backwards), matching platform behavior.
        let settled = {
            let m = &mut self.shapes;
            if m.morph_spring.is_settled(1.0, 0.001, 0.02) {
                if (m.morph_spring.pos - 1.0).abs() > f32::EPSILON
                    || m.morph_spring.vel.abs() > f32::EPSILON
                    || (m.morph_t - 1.0).abs() > f32::EPSILON
                {
                    m.morph_spring = SpringState { pos: 1.0, vel: 0.0 };
                    m.morph_t = 1.0;
                    m.previous = m.current;
                    m.morph_intermediate_from = None;
                }
                true
            } else {
                let params = SpringParams {
                    stiffness: m.morph_stiffness,
                    damping: damping_from_ratio(m.morph_stiffness, m.morph_damping_ratio),
                    mass: 1.0,
                };
                let next = spring_step(params, m.morph_spring, 1.0, dt);
                m.morph_spring = next;
                // Coerced (not latched): tracks the spring each frame so the shape
                // eases out instead of popping to the target on first overshoot.
                m.morph_t = next.pos.clamp(0.0, 1.0);
                if next.is_settled(1.0, 0.001, 0.02) {
                    m.morph_spring = SpringState { pos: 1.0, vel: 0.0 };
                    m.morph_t = 1.0;
                    m.previous = m.current;
                    m.morph_intermediate_from = None;
                    true
                } else {
                    window.request_animation_frame();
                    false
                }
            }
        };
        let _ = settled;
    }

    /// Advance the loading indicator loop by wall-clock `dt` seconds.
    /// Runs on real time (not sim substeps) so its cadence stays calm on
    /// every display; independent of playground sliders. Always requests
    /// the next frame while the showcase lives.
    pub(crate) fn advance_loading(&mut self, window: &mut Window, _dt: f32) {
        let m = &mut self.shapes;
        let now = std::time::Instant::now();
        let dt = m
            .last_loading_tick
            .map(|t| (now - t).as_secs_f32())
            .unwrap_or(0.016)
            .clamp(0.001, 0.05);
        m.last_loading_tick = Some(now);

        let params = m.loading_params();
        let interval = m.loading_interval();
        let period = m.loading_rotation_period();
        let turn = m.loading_turn();
        m.loading_progress = spring_step(params, m.loading_progress, 1.0, dt);
        m.loading_elapsed += dt;
        m.loading_rotation = (m.loading_rotation + 360.0 * dt / period) % 360.0;
        let settled = m.loading_progress.is_settled(1.0, 0.02, 0.3);
        if (m.loading_elapsed >= interval && settled) || m.loading_elapsed >= interval * 1.6 {
            m.loading_index = (m.loading_index + 1) % LOADING_SEQUENCE.len();
            m.loading_progress = SpringState { pos: 0.0, vel: 0.0 };
            m.loading_elapsed = 0.0;
            m.loading_quarter = (m.loading_quarter + turn) % 360.0;
        }
        window.request_animation_frame();
    }

    /// Returns the active interpolated 180-polygon coordinates at this instant.
    #[must_use]
    pub(crate) fn current_morph_points(&self) -> [[f32; 2]; SHAPE_SAMPLE_COUNT] {
        let t = self.shapes.morph_t.clamp(0.0, 1.0);
        let mut res = [[0.0f32; 2]; SHAPE_SAMPLE_COUNT];
        let one_minus_t = 1.0 - t;
        if let Some(from_pts) = &self.shapes.morph_intermediate_from {
            let to_pts = self.shapes.current.points();
            for i in 0..SHAPE_SAMPLE_COUNT {
                res[i][0] = one_minus_t * from_pts[i][0] + t * to_pts[i][0];
                res[i][1] = one_minus_t * from_pts[i][1] + t * to_pts[i][1];
            }
        } else {
            let p0 = self.shapes.previous.points();
            let p1 = self.shapes.current.points();
            for i in 0..SHAPE_SAMPLE_COUNT {
                res[i][0] = one_minus_t * p0[i][0] + t * p1[i][0];
                res[i][1] = one_minus_t * p0[i][1] + t * p1[i][1];
            }
        }
        res
    }

    /// Select a new target shape and initiate spring-driven morphing.
    /// Seamlessly handles mid-animation interruptions without visual jumps.
    pub(crate) fn select_shape(&mut self, shape: MaterialShape, cx: &mut Context<Self>) {
        if shape == self.shapes.current {
            return;
        }
        if (self.shapes.morph_spring.pos - 1.0).abs() > 0.005 {
            // Mid-flight interruption: capture current vertex positions as start geometry
            let cur_pts = self.current_morph_points();
            self.shapes.morph_intermediate_from = Some(cur_pts);
        } else {
            self.shapes.morph_intermediate_from = None;
            self.shapes.previous = self.shapes.current;
        }
        self.shapes.current = shape;
        // Snapshot slider values so dragging stiffness/damping mid-flight cannot
        // destabilize the in-progress oscillation.
        self.shapes.morph_stiffness = self.shapes.stiffness_slider.value;
        self.shapes.morph_damping_ratio = self.shapes.damping_slider.value;
        self.shapes.morph_spring = SpringState { pos: 0.0, vel: 0.0 };
        self.shapes.morph_t = 0.0;
        cx.notify();
    }

    /// Set playground sliders to exact platform motion (k=200, zeta=0.6).
    pub(crate) fn apply_android_preset(&mut self, cx: &mut Context<Self>) {
        for (id, target) in [
            (SliderId::ShapeStiffness, ANDROID_STIFFNESS),
            (SliderId::ShapeDamping, ANDROID_DAMPING_RATIO),
        ] {
            let m = self.slider_model_mut(id);
            m.value = target;
            m.display = SpringState {
                pos: m.target_frac(),
                vel: 0.0,
            };
        }
        self.shapes.morph_stiffness = ANDROID_STIFFNESS;
        self.shapes.morph_damping_ratio = ANDROID_DAMPING_RATIO;
        self.shapes.loading_smoother = false;
        cx.notify();
    }

    /// Set playground sliders to the smoother preset (k=200, zeta=0.7).
    pub(crate) fn apply_smoother_preset(&mut self, cx: &mut Context<Self>) {
        for (id, target) in [
            (SliderId::ShapeStiffness, ANDROID_STIFFNESS),
            (SliderId::ShapeDamping, SMOOTHER_DAMPING_RATIO),
        ] {
            let m = self.slider_model_mut(id);
            m.value = target;
            m.display = SpringState {
                pos: m.target_frac(),
                vel: 0.0,
            };
        }
        self.shapes.morph_stiffness = ANDROID_STIFFNESS;
        self.shapes.morph_damping_ratio = SMOOTHER_DAMPING_RATIO;
        self.shapes.loading_smoother = true;
        cx.notify();
    }

    /// Render a shape grid row for a slice of [`MaterialShape::ALL`].
    fn render_shape_row(
        &self,
        shapes: &[MaterialShape],
        pal: &M3Palette,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex()
            .gap_2()
            .w_full()
            .items_stretch()
            .children(shapes.iter().copied().map(|shape| {
                let is_selected = shape == self.shapes.current;
                let icon_color = if is_selected {
                    pal.primary
                } else {
                    gpui::Rgba {
                        a: 0.70,
                        ..pal.on_surface
                    }
                };
                div()
                    .id(("shape-btn", shape as usize))
                    .flex_1()
                    .min_w_0()
                    .h(px(64.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .rounded_xl()
                    .border(px(2.0))
                    .cursor_pointer()
                    .when(is_selected, |d| {
                        d.border_color(pal.primary).bg(gpui::Rgba {
                            a: if self.dark_theme { 0.18 } else { 0.12 },
                            ..pal.primary
                        })
                    })
                    .when(!is_selected, |d| {
                        d.border_color(gpui::transparent_black())
                            .bg(gpui::Rgba {
                                a: if self.dark_theme { 0.05 } else { 0.03 },
                                ..pal.on_surface
                            })
                            .hover(|h| {
                                h.bg(gpui::Rgba {
                                    a: if self.dark_theme { 0.10 } else { 0.06 },
                                    ..pal.on_surface
                                })
                            })
                    })
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.select_shape(shape, cx);
                        }),
                    )
                    .child(
                        gpui::canvas(
                            move |bounds, _, _| (bounds, shape, icon_color),
                            move |_, (bounds, s, c), window, _| {
                                draw_shape(window, bounds, s, c);
                            },
                        )
                        .size(px(26.0)),
                    )
                    .child(
                        div()
                            .mt_1()
                            .text_size(px(9.5))
                            .font_weight(if is_selected {
                                gpui::FontWeight::SEMIBOLD
                            } else {
                                gpui::FontWeight::NORMAL
                            })
                            .text_color(if is_selected {
                                pal.primary
                            } else {
                                pal.on_surface_variant
                            })
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .child(shape.short_name()),
                    )
            }))
    }

    /// Render the complete 35 Expressive Shapes & Spring Morpher showcase section.
    pub(crate) fn render_shapes_section(
        &self,
        pal: &M3Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let inter = self.shapes.morph_intermediate_from;
        let p0 = self.shapes.previous;
        let p1 = self.shapes.current;
        let morph_t = self.shapes.morph_t;
        let spring_pos = self.shapes.morph_spring.pos;
        let shape_color = pal.primary;

        // Loading loop snapshot (Copy scalars only; points resolve inside paint).
        let from_loading = LOADING_SEQUENCE[self.shapes.loading_index];
        let to_loading = LOADING_SEQUENCE[(self.shapes.loading_index + 1) % LOADING_SEQUENCE.len()];
        let loading_t = self.shapes.loading_progress.pos.clamp(0.0, 1.0);
        let loading_spring = self.shapes.loading_progress.pos;
        let loading_turn = self.shapes.loading_turn();
        let loading_bounce = self.shapes.loading_bounce();
        let loading_angle =
            loading_t * loading_turn + self.shapes.loading_quarter + self.shapes.loading_rotation;
        let loading_smoother = self.shapes.loading_smoother;
        let primary_color = pal.primary;
        let contained_color = pal.primary;

        div()
            .flex_col()
            .gap_4()
            .p_6()
            .rounded_2xl()
            .bg(gpui::Rgba {
                a: if self.dark_theme { 0.08 } else { 0.04 },
                ..pal.on_surface
            })
            // Top Section Header
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_baseline()
                    .child(
                        div()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child("35 Expressive Shapes & Spring Morpher"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(pal.on_surface_variant)
                                    .child(
                                        "Platform spring defaults k=200 ζ=0.60 • Loading loop below",
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .text_base()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(pal.primary)
                            .child(self.shapes.current.name()),
                    ),
            )
            // Big Central Shape Canvas
            .child(
                div()
                    .rounded_2xl()
                    .bg(pal.surface)
                    .py_8()
                    .px_6()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .child(
                        gpui::canvas(
                            move |bounds, _, _| (bounds, inter, p0, p1, morph_t, spring_pos, shape_color),
                            move |_, (bounds, inter, p0, p1, morph_t, spring_pos, color), window, _| {
                                if let Some(pts) = inter {
                                    draw_morph_with_bounce(
                                        window,
                                        bounds,
                                        &pts,
                                        p1.points(),
                                        morph_t,
                                        spring_pos,
                                        color,
                                    );
                                } else {
                                    draw_shape_morph_with_bounce(
                                        window,
                                        bounds,
                                        p0,
                                        p1,
                                        morph_t,
                                        spring_pos,
                                        color,
                                    );
                                }
                            },
                        )
                        .size(px(160.0)),
                    )
                    .child(
                        div()
                            .mt_4()
                            .text_xs()
                            .text_color(pal.on_surface_variant)
                            .child("Click any shape below to morph with live spring physics"),
                    ),
            )
            // Physics Controls: side-by-side on one row. Safe with per-type
            // drag isolation and fixed track widths (no shared hit-test state).
            .child(
                div()
                    .flex()
                    .gap_6()
                    .items_start()
                    .child(
                        div()
                            .flex_1()
                            .child(self.render_slider(SliderId::ShapeStiffness, pal, cx)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .child(self.render_slider(SliderId::ShapeDamping, pal, cx)),
                    ),
            )
            // Preset row: one-tap platform / smoother motion.
            .child(
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .text_xs()
                            .text_color(pal.on_surface_variant)
                            .child("Presets:"),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .rounded_full()
                            .cursor_pointer()
                            .when(!loading_smoother, |d| {
                                d.bg(pal.primary).text_color(pal.on_primary)
                            })
                            .when(loading_smoother, |d| {
                                d.bg(pal.secondary_container)
                                    .text_color(pal.on_secondary_container)
                            })
                            .text_xs()
                            .child("Android k=200 ζ=0.60")
                            .on_mouse_down(
                                gpui::MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    this.apply_android_preset(cx);
                                }),
                            ),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .rounded_full()
                            .cursor_pointer()
                            .when(loading_smoother, |d| {
                                d.bg(pal.primary).text_color(pal.on_primary)
                            })
                            .when(!loading_smoother, |d| {
                                d.bg(pal.secondary_container)
                                    .text_color(pal.on_secondary_container)
                            })
                            .text_xs()
                            .child("Smoother ζ=0.70")
                            .on_mouse_down(
                                gpui::MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    this.apply_smoother_preset(cx);
                                }),
                            ),
                    ),
            )
            .child(self.render_shape_row(&MaterialShape::ALL[0..18], pal, cx))
            .child(self.render_shape_row(&MaterialShape::ALL[18..35], pal, cx))
            // Divider before the loading indicator subsection.
            .child(div().h(px(1.0)).w_full().bg(gpui::Rgba {
                a: 0.10,
                ..pal.on_surface
            }))
            // Loading Indicator subsection header with active morph badge
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child("Loading Indicator"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(pal.on_surface_variant)
                                    .child(
                                        "Material 3 Expressive • Indeterminate 7-shape continuous loop (650ms / 4.66s)",
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .px_3()
                            .py_1()
                            .rounded_full()
                            .bg(pal.secondary_container)
                            .text_color(pal.on_secondary_container)
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child(format!("{}  →  {}", from_loading.name(), to_loading.name())),
                    ),
            )
            // Side-by-side indicator presentation cards: Uncontained & Contained
            .child(
                div()
                    .flex()
                    .gap_6()
                    .items_stretch()
                    .child(
                        // Uncontained LoadingIndicator
                        div()
                            .flex_1()
                            .rounded_2xl()
                            .bg(pal.surface)
                            .py_6()
                            .px_6()
                            .flex_col()
                            .items_center()
                            .justify_between()
                            .gap_4()
                            .child(
                                div()
                                    .flex_col()
                                    .items_center()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .child("LoadingIndicator"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(pal.on_surface_variant)
                                            .child("Uncontained Variant"),
                                    ),
                            )
                            .child(
                                div()
                                    .size(px(88.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        gpui::canvas(
                                            move |bounds, _, _| {
                                                (
                                                    bounds,
                                                    from_loading,
                                                    to_loading,
                                                    loading_t,
                                                    loading_angle,
                                                    loading_spring,
                                                    loading_bounce,
                                                    primary_color,
                                                )
                                            },
                                            move |_,
                                                  (
                                                bounds,
                                                from_s,
                                                to_s,
                                                t,
                                                angle,
                                                spring,
                                                bounce,
                                                color,
                                            ),
                                                  window,
                                                  _| {
                                                draw_loading_shape(
                                                    window, bounds, from_s, to_s, t, angle, spring,
                                                    bounce, color,
                                                );
                                            },
                                        )
                                        .size(px(88.0)),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(pal.on_surface_variant)
                                    .text_center()
                                    .child("Indicator: Primary • Container: None"),
                            ),
                    )
                    .child(
                        // ContainedLoadingIndicator
                        div()
                            .flex_1()
                            .rounded_2xl()
                            .bg(pal.surface)
                            .py_6()
                            .px_6()
                            .flex_col()
                            .items_center()
                            .justify_between()
                            .gap_4()
                            .child(
                                div()
                                    .flex_col()
                                    .items_center()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .child("ContainedLoadingIndicator"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(pal.on_surface_variant)
                                            .child("Contained Variant"),
                                    ),
                            )
                            .child(
                                div()
                                    .size(px(88.0))
                                    .rounded_full()
                                    .bg(pal.primary_container)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        gpui::canvas(
                                            move |bounds, _, _| {
                                                (
                                                    bounds,
                                                    from_loading,
                                                    to_loading,
                                                    loading_t,
                                                    loading_angle,
                                                    loading_spring,
                                                    loading_bounce,
                                                    contained_color,
                                                )
                                            },
                                            move |_,
                                                  (
                                                bounds,
                                                from_s,
                                                to_s,
                                                t,
                                                angle,
                                                spring,
                                                bounce,
                                                color,
                                            ),
                                                  window,
                                                  _| {
                                                draw_loading_shape(
                                                    window, bounds, from_s, to_s, t, angle, spring,
                                                    bounce, color,
                                                );
                                            },
                                        )
                                        .size(px(88.0)),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(pal.on_surface_variant)
                                    .text_center()
                                    .child(
                                        "Indicator: Primary • Container: PrimaryContainer",
                                    ),
                            ),
                    ),
            )
    }
}
