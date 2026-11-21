use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    Context, IntoElement, ParentElement, Pixels, Render, SharedString, Styled, Window, div,
    prelude::*, px,
};
use gpui_material::{M3Palette, SliderSpec, SpringParams, SpringState, spring_step};

use crate::Showcase;

pub(crate) const TRACK_W: f32 = 480.0;

/// Identifier identifying which slider is being manipulated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SliderId {
    /// One of the main showcase sliders (by index).
    Showcase(usize),
    /// The spring stiffness slider in the 35 shapes playground.
    ShapeStiffness,
    /// The spring damping ratio slider in the 35 shapes playground.
    ShapeDamping,
}

/// Drag marker for the main showcase sliders, carrying the slider index.
///
/// Each slider group uses a distinct drag value type so the framework-level
/// `TypeId` check in `on_drag_move` filters out cross-fire: capture-phase
/// drag listeners of *every* slider fire on *every* mouse move during any
/// drag. Handlers must route via the drag payload ([`DragSliderId`]), never
/// the element's own id — otherwise every same-typed slider would seek
/// itself to the cursor. The main sliders safely share one type because
/// identical track geometry maps to the same value either way.
#[derive(Clone)]
pub(crate) struct ShowcaseDrag(pub(crate) usize);

/// Drag marker isolating the stiffness slider's drag routing.
#[derive(Clone)]
pub(crate) struct ShapeStiffnessDrag;

/// Drag marker isolating the damping ratio slider's drag routing.
#[derive(Clone)]
pub(crate) struct ShapeDampingDrag;

/// Resolves the dragged slider from a drag payload.
///
/// Handlers route via this (the drag source), never via the element id:
/// same-typed listeners all fire during a drag, so element-id routing would
/// seek every one of them to the cursor.
pub(crate) trait DragSliderId {
    /// Slider this drag gesture belongs to.
    fn slider_id(&self) -> SliderId;
}

impl DragSliderId for ShowcaseDrag {
    fn slider_id(&self) -> SliderId {
        SliderId::Showcase(self.0)
    }
}

impl DragSliderId for ShapeStiffnessDrag {
    fn slider_id(&self) -> SliderId {
        SliderId::ShapeStiffness
    }
}

impl DragSliderId for ShapeDampingDrag {
    fn slider_id(&self) -> SliderId {
        SliderId::ShapeDamping
    }
}

/// Attach press/drag seeking, isolated by drag type `D`.
///
/// The `TypeId` check drops listeners of other groups; same-typed listeners
/// all fire but route to the dragged slider via the payload.
fn with_slider_drag<D>(
    el: gpui::Stateful<gpui::Div>,
    tag: D,
    cx: &Context<Showcase>,
) -> gpui::Stateful<gpui::Div>
where
    D: Clone + DragSliderId + 'static,
{
    el.on_drag(tag, |_drag, _, _, cx| cx.new(|_| EmptyDrag))
        .on_drag_move(cx.listener(move |this, e: &gpui::DragMoveEvent<D>, _, cx| {
            let drag_id = e.drag(cx).slider_id();
            let mx = f32::from(e.event.position.x);
            let left = f32::from(e.bounds.origin.x);
            let w = f32::from(e.bounds.size.width);
            this.slider_model_mut(drag_id).seek_absolute(mx, left, w);
            cx.notify();
        }))
}

/// Attach the drag handlers matching slider `id`, each group isolated by its
/// own drag value type (see [`ShowcaseDrag`]).
fn attach_slider_drag(
    el: gpui::Stateful<gpui::Div>,
    id: SliderId,
    cx: &Context<Showcase>,
) -> gpui::Stateful<gpui::Div> {
    match id {
        SliderId::Showcase(ix) => with_slider_drag(el, ShowcaseDrag(ix), cx),
        SliderId::ShapeStiffness => with_slider_drag(el, ShapeStiffnessDrag, cx),
        SliderId::ShapeDamping => with_slider_drag(el, ShapeDampingDrag, cx),
    }
}

pub(crate) struct EmptyDrag;

impl Render for EmptyDrag {
    fn render(&mut self, _w: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_0()
    }
}

/// Showcase state model for interactive slider controls.
#[derive(Clone)]
pub(crate) struct SliderModel {
    pub(crate) label: SharedString,
    pub(crate) spec: SliderSpec,
    pub(crate) value: f32,
    pub(crate) display: SpringState,
    pub(crate) press_state: SpringState,
    pub(crate) hover_state: SpringState,
    pub(crate) is_pressed: bool,
    pub(crate) is_hovered: bool,
    pub(crate) drag: Option<(Pixels, f32)>,
    /// Last measured element bounds `(left, width)` from layout/drag events.
    /// Hit-testing only; render width stays fixed so measuring can never
    /// feed back into layout and stretch the track.
    pub(crate) meas: Rc<Cell<Option<(f32, f32)>>>,
    /// Fixed render width of the track; never mutated after construction.
    pub(crate) track_w: f32,
    pub(crate) enabled: bool,
    pub(crate) expressive: bool,
}

impl SliderModel {
    pub(crate) fn new(label: impl Into<SharedString>, spec: SliderSpec, value: f32) -> Self {
        let frac = spec.fraction(value);
        Self {
            label: label.into(),
            spec,
            value,
            display: SpringState {
                pos: frac,
                vel: 0.0,
            },
            press_state: SpringState::default(),
            hover_state: SpringState::default(),
            is_pressed: false,
            is_hovered: false,
            drag: None,
            meas: Rc::new(Cell::new(None)),
            track_w: TRACK_W,
            enabled: true,
            expressive: true,
        }
    }

    pub(crate) fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub(crate) fn expressive(mut self, expressive: bool) -> Self {
        self.expressive = expressive;
        self
    }

    pub(crate) fn track_w(mut self, w: f32) -> Self {
        self.track_w = w;
        self
    }

    pub(crate) fn target_frac(&self) -> f32 {
        self.spec.fraction(self.value)
    }

    /// Advance spring positions for value display, press feedback, and hover state.
    pub(crate) fn advance_springs(&mut self, window: &mut Window, dt: f32) {
        let target = self.target_frac();
        let params = if self.expressive {
            SpringParams::expressive()
        } else {
            SpringParams::standard()
        };
        let next = spring_step(params, self.display, target, dt);
        let settled = (next.pos - target).abs() < 0.0005 && next.vel.abs() < 0.0005;
        self.display = if settled {
            SpringState {
                pos: target,
                vel: 0.0,
            }
        } else {
            window.request_animation_frame();
            next
        };

        // Press spring: targets 1.0 when pressed or dragging, 0.0 otherwise
        let press_target = if self.enabled && (self.is_pressed || self.drag.is_some()) {
            1.0
        } else {
            0.0
        };
        let press_params = SpringParams {
            stiffness: 450.0,
            damping: 30.0,
            mass: 1.0,
        };
        let next_press = spring_step(press_params, self.press_state, press_target, dt);
        let press_settled =
            (next_press.pos - press_target).abs() < 0.001 && next_press.vel.abs() < 0.001;
        self.press_state = if press_settled {
            SpringState {
                pos: press_target,
                vel: 0.0,
            }
        } else {
            window.request_animation_frame();
            next_press
        };

        // Hover spring: targets 1.0 when hovered, 0.0 otherwise
        let hover_target = if self.enabled && self.is_hovered {
            1.0
        } else {
            0.0
        };
        let hover_params = SpringParams {
            stiffness: 400.0,
            damping: 32.0,
            mass: 1.0,
        };
        let next_hover = spring_step(hover_params, self.hover_state, hover_target, dt);
        let hover_settled =
            (next_hover.pos - hover_target).abs() < 0.001 && next_hover.vel.abs() < 0.001;
        self.hover_state = if hover_settled {
            SpringState {
                pos: hover_target,
                vel: 0.0,
            }
        } else {
            window.request_animation_frame();
            next_hover
        };
    }

    /// Bounds used for hit-testing: measured live on layout/prepaint, fallback otherwise.
    fn hit_bounds(&self) -> (f32, f32) {
        self.meas.get().unwrap_or((0.0, self.track_w))
    }

    /// Click or drag seek to an absolute coordinate along the track.
    pub(crate) fn seek_absolute(&mut self, mouse_x: f32, left: f32, w: f32) {
        if !self.enabled {
            return;
        }
        // Cache bounds for hit-testing only; `track_w` (render width) is fixed.
        self.meas.set(Some((left, w.max(1.0))));
        let avail_w = (w - 12.0).max(1.0);
        let frac = ((mouse_x - left - 6.0) / avail_w).clamp(0.0, 1.0);
        self.value = self.spec.value_from_fraction(frac);
    }

    /// Handle mouse down event to initiate click-to-seek and drag interaction.
    pub(crate) fn on_down(&mut self, x: Pixels) {
        if !self.enabled {
            return;
        }
        self.is_pressed = true;
        // Click-to-seek: jump straight to click point.
        // Spring animation smooths the thumb to the click point.
        let (left, w) = self.hit_bounds();
        let avail_w = (w - 12.0).max(1.0);
        let frac = ((f32::from(x) - left - 6.0) / avail_w).clamp(0.0, 1.0);
        self.value = self.spec.value_from_fraction(frac);
        let v = self.value;
        self.drag = Some((x, v));
    }

    /// Handle mouse move during active drag interaction.
    pub(crate) fn on_move(&mut self, x: Pixels) {
        if self.drag.is_none() || !self.enabled {
            return;
        }
        let (left, w) = self.hit_bounds();
        let avail_w = (w - 12.0).max(1.0);
        let frac = ((f32::from(x) - left - 6.0) / avail_w).clamp(0.0, 1.0);
        self.value = self.spec.value_from_fraction(frac);
    }

    /// Release drag or press state.
    pub(crate) fn on_up(&mut self) {
        self.drag = None;
        self.is_pressed = false;
    }

    /// Update hover state.
    pub(crate) fn hover(&mut self, hovered: bool) {
        if !self.enabled {
            return;
        }
        self.is_hovered = hovered;
    }

    /// Step value incrementally by +/- direction.
    pub(crate) fn step_by(&mut self, dir: f32) {
        if !self.enabled {
            return;
        }
        let s = self.spec;
        let span = (s.max - s.min).max(1e-6);
        let step = if s.steps == 0 {
            span / 20.0
        } else {
            span / (s.steps + 1) as f32
        };
        let cur = self.value;
        self.value = s.value_from_fraction(s.fraction(cur + dir * step));
    }
}

pub(crate) fn init_sliders() -> Vec<SliderModel> {
    vec![
        SliderModel::new("Media Volume", SliderSpec::new(0.0, 100.0, 0), 42.0),
        SliderModel::new("Display Brightness", SliderSpec::new(0.0, 10.0, 4), 6.0),
        SliderModel::new("Balance", SliderSpec::new(0.0, 1.0, 0), 0.5)
            .enabled(false)
            .expressive(false),
    ]
}

impl Showcase {
    /// Advance animation springs for showcase sliders.
    pub(crate) fn advance_sliders_springs(&mut self, window: &mut Window, dt: f32) {
        for m in &mut self.sliders {
            m.advance_springs(window, dt);
        }
    }

    /// Access immutable reference to any slider model by its ID.
    #[must_use]
    pub(crate) fn slider_model(&self, id: SliderId) -> &SliderModel {
        match id {
            SliderId::Showcase(ix) => &self.sliders[ix],
            SliderId::ShapeStiffness => &self.shapes.stiffness_slider,
            SliderId::ShapeDamping => &self.shapes.damping_slider,
        }
    }

    /// Access mutable reference to any slider model by its ID.
    pub(crate) fn slider_model_mut(&mut self, id: SliderId) -> &mut SliderModel {
        match id {
            SliderId::Showcase(ix) => &mut self.sliders[ix],
            SliderId::ShapeStiffness => &mut self.shapes.stiffness_slider,
            SliderId::ShapeDamping => &mut self.shapes.damping_slider,
        }
    }

    /// Render any slider control by its ID using unified M3 Expressive slider specs.
    pub(crate) fn render_slider(
        &self,
        id: SliderId,
        pal: &M3Palette,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let m = self.slider_model(id);
        let frac = m.display.pos.clamp(0.0, 1.0);
        let press = m.press_state.pos.clamp(0.0, 1.0);
        let hover = m.hover_state.pos.clamp(0.0, 1.0);
        let track_w = m.track_w;

        let avail_w = (track_w - 12.0).max(1.0);
        let p = frac * avail_w;
        let thumb_cx = p + 6.0;

        // Handle "|" animation: shrinks in width from 4px down to 2px when pressed.
        let thumb_w = 4.0 - 2.0 * press;
        // Height expands: 40px idle -> 44px hover -> 48px pressed.
        let thumb_h = 40.0 + 4.0 * hover * (1.0 - press) + 8.0 * press;
        let thumb_top = 36.0 - thumb_h / 2.0;
        let thumb_left = thumb_cx - thumb_w / 2.0;

        let active_end = p;
        let inactive_start = p + 12.0;
        let show_active = active_end >= 8.0;
        let show_inactive = (track_w - inactive_start) >= 8.0;
        let stop_x = track_w - 8.0;
        let show_stop = show_inactive && (stop_x - thumb_cx).abs() >= 10.0;
        let ticks = m.spec.tick_fractions();
        let truth_frac = m.target_frac();

        let val_str = if !m.enabled {
            "Disabled".to_string()
        } else if m.spec.steps > 0 {
            format!("{:.0} / {:.0}", m.value, m.spec.max)
        } else if (m.spec.max - 100.0).abs() < 1e-4 {
            format!("{:.0}%", m.value)
        } else if (m.spec.max - 1.0).abs() < 1e-4 || (m.spec.max - 1.5).abs() < 1e-4 {
            format!("{:.2}", m.value)
        } else {
            format!("{:.0}", m.value)
        };

        let meas_cell = m.meas.clone();

        div()
            .flex_col()
            .gap_2()
            .py_2()
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child(m.label.clone()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(pal.on_surface_variant)
                            .child(val_str),
                    ),
            )
            .child(
                div()
                    .relative()
                    .w(px(track_w))
                    .h(px(72.0))
                    .id((
                        "m3-slider",
                        match id {
                            SliderId::Showcase(ix) => ix,
                            SliderId::ShapeStiffness => 1001,
                            SliderId::ShapeDamping => 1002,
                        },
                    ))
                    // Continuously cache live layout bounds for click-to-seek without
                    // hardcoded coordinates or layout feedback loops.
                    .child(
                        gpui::canvas(
                            move |bounds, _, _| {
                                meas_cell.set(Some((
                                    f32::from(bounds.origin.x),
                                    f32::from(bounds.size.width),
                                )));
                            },
                            |_, _, _, _| {},
                        )
                        .absolute()
                        .size_full(),
                    )
                    .cursor_pointer()
                    .when(!m.enabled, |d| d.opacity(0.38))
                    .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                        this.slider_model_mut(id).hover(*hovered);
                        cx.notify();
                    }))
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, e: &gpui::MouseDownEvent, _, cx| {
                            this.slider_model_mut(id).on_down(e.position.x);
                            cx.notify();
                        }),
                    )
                    .on_mouse_move(cx.listener(move |this, e: &gpui::MouseMoveEvent, _, cx| {
                        this.slider_model_mut(id).on_move(e.position.x);
                        cx.notify();
                    }))
                    .on_mouse_up(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.slider_model_mut(id).on_up();
                            cx.notify();
                        }),
                    )
                    .on_mouse_up_out(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.slider_model_mut(id).on_up();
                            cx.notify();
                        }),
                    )
                    // Isolated per-group drag routing: without distinct drag
                    // types, every slider's capture-phase listener would
                    // overwrite the dragged value against its own bounds.
                    .when(true, |el| attach_slider_drag(el, id, cx))
                    .when(m.drag.is_some(), |d| {
                        let cur_val = self.slider_model(id).value;
                        let tip_str =
                            if (m.spec.max - 1.0).abs() < 1e-4 || (m.spec.max - 1.5).abs() < 1e-4 {
                                format!("{:.2}", cur_val)
                            } else {
                                format!("{:.1}", cur_val)
                            };
                        d.child(
                            div()
                                .absolute()
                                .top(px(0.0))
                                .left(px(thumb_cx - 28.0))
                                .w(px(56.0))
                                .h(px(24.0))
                                .flex()
                                .justify_center()
                                .items_center()
                                .rounded_full()
                                .bg(pal.inverse_surface)
                                .text_color(pal.inverse_on_surface)
                                .text_xs()
                                .child(tip_str),
                        )
                    })
                    .when(show_active, |d| {
                        d.child(
                            div()
                                .absolute()
                                .top(px(28.0))
                                .left(px(0.0))
                                .w(px(active_end))
                                .h(px(16.0))
                                .rounded_l_full()
                                .rounded_tr_xs()
                                .rounded_br_xs()
                                .bg(pal.primary),
                        )
                    })
                    .when(show_inactive, |d| {
                        d.child(
                            div()
                                .absolute()
                                .top(px(28.0))
                                .left(px(inactive_start))
                                .w(px(track_w - inactive_start))
                                .h(px(16.0))
                                .rounded_r_full()
                                .rounded_tl_xs()
                                .rounded_bl_xs()
                                .bg(pal.secondary_container),
                        )
                    })
                    .when(show_stop, |d| {
                        d.child(
                            div()
                                .absolute()
                                .top(px(34.0))
                                .left(px(stop_x - 2.0))
                                .size(px(4.0))
                                .rounded_full()
                                .bg(pal.primary),
                        )
                    })
                    .children(ticks.into_iter().filter_map(|t| {
                        if (t - 1.0).abs() < 1e-4 {
                            return None;
                        }
                        let x = t * (track_w - 12.0) + 6.0;
                        // Hide ticks inside the thumb cutout
                        if (x - thumb_cx).abs() < 8.0 {
                            return None;
                        }
                        let c = if t <= truth_frac + 1e-5 {
                            pal.secondary_container
                        } else {
                            pal.primary
                        };
                        Some(
                            div()
                                .absolute()
                                .top(px(34.0))
                                .left(px(x - 2.0))
                                .size(px(4.0))
                                .rounded_full()
                                .bg(c),
                        )
                    }))
                    .child(
                        div()
                            .absolute()
                            .top(px(thumb_top))
                            .left(px(thumb_left))
                            .w(px(thumb_w))
                            .h(px(thumb_h))
                            .rounded_full()
                            .bg(pal.primary)
                            .when(!m.enabled, |d| {
                                d.bg(gpui::Rgba {
                                    a: 0.38,
                                    ..pal.on_surface
                                })
                            })
                            .when(m.enabled && (m.drag.is_some() || m.is_pressed), |d| {
                                d.shadow_md()
                            })
                            .when(
                                m.enabled && m.drag.is_none() && !m.is_pressed && m.is_hovered,
                                |d| d.shadow_sm(),
                            ),
                    ),
            )
            .when(matches!(id, SliderId::Showcase(_)), |d| {
                d.child(
                    div()
                        .flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded_full()
                                .bg(pal.secondary_container)
                                .text_color(pal.on_secondary_container)
                                .cursor_pointer()
                                .child("-")
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        this.slider_model_mut(id).step_by(-1.0);
                                        cx.notify();
                                    }),
                                ),
                        )
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded_full()
                                .bg(pal.secondary_container)
                                .text_color(pal.on_secondary_container)
                                .cursor_pointer()
                                .child("+")
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        this.slider_model_mut(id).step_by(1.0);
                                        cx.notify();
                                    }),
                                ),
                        ),
                )
            })
    }

    /// Render the sliders showcase section inside a single unified card.
    pub(crate) fn render_sliders_section(
        &self,
        pal: &M3Palette,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex_col()
            .gap_4()
            .p_6()
            .rounded_2xl()
            .bg(gpui::Rgba {
                a: if self.dark_theme { 0.08 } else { 0.04 },
                ..pal.on_surface
            })
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_baseline()
                    .child(
                        div()
                            .text_base()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Sliders"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(pal.on_surface_variant)
                            .child("Continuous & Discrete Stepped Sliders • Spring Snap & Seek"),
                    ),
            )
            .children(
                (0..self.sliders.len())
                    .map(|ix| self.render_slider(SliderId::Showcase(ix), pal, cx)),
            )
    }
}
