use requiem_ui::paint as requiem_ui_paint;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::time::Instant;

use requiem_ui::node::Node;
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::primitives::AnimatedIconWidget;
use requiem_ui::widget::{AbsoluteRect, Widget};
use skia_safe::{Canvas, Color};

use crate::lottie::{
    asset_json, fix_leading_subtract_masks, fix_mask_gaps, recolor_fills_and_strokes,
};

#[derive(Clone, Copy)]
pub struct ToggleMarkers {
    pub asset: &'static str,

    pub to_on: f64,
    pub to_off: f64,
    pub span: f64,

    pub hover_on: (f64, f64),
    pub hover_off: (f64, f64),
}

pub const NOISE_SUPPRESSION: ToggleMarkers = ToggleMarkers {
    asset: "lottie/noise_suppression",
    to_on: 100.0,
    to_off: 0.0,
    span: 70.0,
    hover_on: (200.0, 270.0),
    hover_off: (300.0, 370.0),
};

pub const CAMERA: ToggleMarkers = ToggleMarkers {
    asset: "lottie/camera",
    to_on: 100.0,
    to_off: 0.0,
    span: 70.0,
    hover_on: (180.0, 250.0),
    hover_off: (280.0, 350.0),
};

pub const SCREENSHARE: ToggleMarkers = ToggleMarkers {
    asset: "lottie/screenshare",
    to_on: 0.0,
    to_off: 90.0,
    span: 70.0,
    hover_on: (180.0, 220.0),
    hover_off: (240.0, 280.0),
};

pub const MIC: ToggleMarkers = ToggleMarkers {
    asset: "lottie/mic",
    to_on: 0.0,
    to_off: 100.0,
    span: 70.0,
    hover_on: (240.0, 280.0),
    hover_off: (180.0, 220.0),
};

pub const END_CALL: (&str, f64) = ("lottie/end_call", 90.0);
pub const SOUNDBOARD: (&str, f64) = ("lottie/soundboard", 66.0);

const FPS: f64 = 60.0;

fn json(asset: &'static str, color: Color) -> &'static str {
    thread_local! {
        static CACHE: RefCell<HashMap<(&'static str, u32), &'static str>> = RefCell::new(HashMap::new());
    }
    CACHE.with(|cache| {
        *cache
            .borrow_mut()
            .entry((
                asset,
                u32::from_be_bytes([color.a(), color.r(), color.g(), color.b()]),
            ))
            .or_insert_with(|| {
                let raw = asset_json(asset);
                let cleaned = if asset == "lottie/mic" {
                    fix_mask_gaps(raw)
                } else {
                    fix_leading_subtract_masks(raw)
                }
                .unwrap_or_default();
                let (r, g, b) = (
                    f64::from(color.r()) / 255.0,
                    f64::from(color.g()) / 255.0,
                    f64::from(color.b()) / 255.0,
                );
                Box::leak(
                    recolor_fills_and_strokes(&cleaned, r, g, b)
                        .unwrap_or(cleaned)
                        .into_boxed_str(),
                )
            })
    })
}

thread_local! {

    static TOGGLES: RefCell<HashMap<&'static str, (bool, Option<Instant>)>> = RefCell::new(HashMap::new());
}

fn changed_at(key: &'static str, on: bool) -> Option<Instant> {
    TOGGLES.with(|toggles| {
        let mut toggles = toggles.borrow_mut();
        let entry = toggles.entry(key).or_insert((on, None));
        if entry.0 != on {
            *entry = (on, Some(Instant::now()));
        }
        entry.1
    })
}

fn hover_clock(cell: &Cell<Option<Instant>>, hovered: bool) -> Option<Instant> {
    if hovered && cell.get().is_none() {
        cell.set(Some(Instant::now()));
    } else if !hovered {
        cell.set(None);
    }
    cell.get()
}

fn centered(rect: AbsoluteRect, size: f32) -> AbsoluteRect {
    AbsoluteRect {
        x: rect.x + (rect.width - size) / 2.0,
        y: rect.y + (rect.height - size) / 2.0,
        width: size,
        height: size,
    }
}

struct ToggleLottie {
    markers: ToggleMarkers,
    on: bool,
    changed_at: Option<Instant>,
    rest: AnimatedIconWidget,
    hover: AnimatedIconWidget,
    hover_started_at: Cell<Option<Instant>>,
    size: f32,
}

impl Widget for ToggleLottie {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let m = self.markers;
        let hovered = hover_clock(&self.hover_started_at, style.is_hovered);
        let in_transition = self
            .changed_at
            .map(|at| at.elapsed().as_secs_f64())
            .filter(|secs| *secs < m.span / FPS);
        let (hover_start, hover_end) = if self.on { m.hover_on } else { m.hover_off };
        let frame = match (in_transition, hovered) {
            (Some(secs), _) => (if self.on { m.to_on } else { m.to_off }) + secs * FPS,
            (None, Some(at)) => {
                hover_start
                    + at.elapsed()
                        .as_secs_f64()
                        .min((hover_end - hover_start) / FPS)
                        * FPS
            }
            (None, None) => hover_start,
        };
        let icon = if hovered.is_some() {
            &self.hover
        } else {
            &self.rest
        };
        icon.set_scene(frame, frame, false);
        icon.paint(canvas, centered(rect, self.size), style);
        if in_transition.is_some()
            || hovered
                .is_some_and(|at| at.elapsed().as_secs_f64() < (hover_end - hover_start) / FPS)
        {
            requiem_ui_paint::request_repaint();
        }
    }
}

pub fn toggle(
    key: &'static str,
    markers: ToggleMarkers,
    on: bool,
    color: Color,
    hover_color: Color,
    size: f32,
) -> Node {
    let start = if on {
        markers.hover_on.0
    } else {
        markers.hover_off.0
    };
    Node::new(
        "span",
        ToggleLottie {
            markers,
            on,
            changed_at: changed_at(key, on),
            rest: AnimatedIconWidget::new(json(markers.asset, color), start, start, false),
            hover: AnimatedIconWidget::new(json(markers.asset, hover_color), start, start, false),
            hover_started_at: Cell::new(None),
            size,
        },
    )
}

struct HoverLottie {
    frames: f64,
    rest: AnimatedIconWidget,
    hover: AnimatedIconWidget,
    hover_started_at: Cell<Option<Instant>>,
    size: f32,
}

impl Widget for HoverLottie {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let hovered = hover_clock(&self.hover_started_at, style.is_hovered);
        let frame = hovered.map_or(0.0, |at| {
            at.elapsed().as_secs_f64().min(self.frames / FPS) * FPS
        });
        let icon = if hovered.is_some() {
            &self.hover
        } else {
            &self.rest
        };
        icon.set_scene(frame, frame, false);
        icon.paint(canvas, centered(rect, self.size), style);
        if hovered.is_some_and(|at| at.elapsed().as_secs_f64() < self.frames / FPS) {
            requiem_ui_paint::request_repaint();
        }
    }
}

pub fn hover(
    (asset, frames): (&'static str, f64),
    color: Color,
    hover_color: Color,
    size: f32,
) -> Node {
    Node::new(
        "span",
        HoverLottie {
            frames,
            rest: AnimatedIconWidget::new(json(asset, color), 0.0, 0.0, false),
            hover: AnimatedIconWidget::new(json(asset, hover_color), 0.0, 0.0, false),
            hover_started_at: Cell::new(None),
            size,
        },
    )
}
