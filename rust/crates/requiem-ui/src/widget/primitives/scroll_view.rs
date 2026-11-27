use crate::colors;
use crate::components::TopFade;
use crate::paint as crate_paint;
use crate::scroll_animation;
use skia_safe::gradient::shaders::linear_gradient;
use skia_safe::gradient::{Colors, Gradient, Interpolation};
use skia_safe::{Canvas, Color, Color4f, Paint, PaintStyle, Point, RRect, Rect, TileMode};
use std::cell::Cell;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

use super::box_paint::paint_background;
use crate::colors::{SCROLLBAR_AUTO_THUMB, SCROLLBAR_THIN_THUMB};
use crate::event::{EventResult, MouseButton, UiEvent};
use crate::scroll_animation::{SpringScroll, WheelAnimation, smooth_scrolling};
use crate::style::{ComputedStyle, ScrollbarKind, ScrollbarStyle};
use crate::widget::{AbsoluteRect, ScrollChild, Widget};

const THUMB_MIN_HEIGHT: f32 = 40.0;

fn scrollbar_metrics(kind: ScrollbarKind) -> Option<(f32, f32, Color)> {
    match kind {
        ScrollbarKind::Legacy | ScrollbarKind::Thin => Some((8.0, 2.0, SCROLLBAR_THIN_THUMB)),
        ScrollbarKind::Auto => Some((16.0, 4.0, SCROLLBAR_AUTO_THUMB)),
        ScrollbarKind::None => None,
    }
}

const TOP_FADE_HEIGHT: f32 = 40.0;

const TOP_FADE_RIGHT_INSET: f32 = 16.0;

const AUTOSCROLL_DEAD_ZONE: f32 = 12.0;

const AUTOSCROLL_GAIN: f32 = 10.0;

const AUTOSCROLL_MARKER: f32 = 28.0;

#[derive(Clone, Copy, Debug)]
struct Autoscroll {
    anchor: (f32, f32),
    pointer_y: f32,
    last_tick: Instant,

    dragged: bool,
}

impl Autoscroll {
    fn velocity(&self) -> f32 {
        let distance = self.pointer_y - self.anchor.1;
        distance.signum() * (distance.abs() - AUTOSCROLL_DEAD_ZONE).max(0.0) * AUTOSCROLL_GAIN
    }
}

fn paint_autoscroll_marker(canvas: &Canvas, autoscroll: &Autoscroll) {
    let (cx, cy) = autoscroll.anchor;
    let radius = AUTOSCROLL_MARKER / 2.0;
    let mut fill = Paint::new(Color4f::new(0.07, 0.07, 0.08, 0.85), None);
    fill.set_anti_alias(true);
    canvas.draw_circle((cx, cy), radius, &fill);
    let mut ring = Paint::new(Color4f::new(1.0, 1.0, 1.0, 0.9), None);
    ring.set_anti_alias(true);
    ring.set_style(PaintStyle::Stroke);
    ring.set_stroke_width(1.5);
    canvas.draw_circle((cx, cy), radius - 0.75, &ring);
    let mut arrow = Paint::new(Color4f::new(1.0, 1.0, 1.0, 0.95), None);
    arrow.set_anti_alias(true);
    for sign in [-1.0_f32, 1.0] {
        let mut b = skia_safe::PathBuilder::new();
        b.move_to(Point::new(cx - 4.5, cy + sign * 3.0));
        b.line_to(Point::new(cx + 4.5, cy + sign * 3.0));
        b.line_to(Point::new(cx, cy + sign * 8.0));
        b.close();
        canvas.draw_path(&b.detach(), &arrow);
    }
}

#[derive(Clone, Debug)]
pub struct WheelGlide {
    anim: WheelAnimation,
    applied: f32,
}

pub type WheelCarry = Rc<RefCell<Option<WheelGlide>>>;

const PARKED_FOR: Duration = Duration::from_millis(250);

thread_local! {

    static PARKED: RefCell<HashMap<String, (WheelGlide, Instant)>> = RefCell::new(HashMap::new());

    static PARKED_SPRINGS: RefCell<HashMap<String, (SpringScroll, Instant)>> = RefCell::new(HashMap::new());

    static PARKED_OFFSETS: RefCell<HashMap<String, (f32, Instant)>> = RefCell::new(HashMap::new());
}

#[derive(Clone, Debug, PartialEq)]
pub enum ScrollPosition {
    Pinned,
    Anchored(ScrollAnchor),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollAnchor {
    pub key: String,
    pub offset_from_top: f32,

    pub height: f32,
}

type PositionCallback = Box<dyn FnMut(&ScrollPosition)>;
type MeasureCallback = Box<dyn FnMut(&[(String, f32)])>;

struct ChatMode {
    position: ScrollPosition,

    pin_threshold: f32,

    children: Vec<(String, f32, f32)>,
    last_max_offset: f32,
    on_position: Option<PositionCallback>,
    on_measure: Option<MeasureCallback>,
}

impl ChatMode {
    fn find_anchor(&self, offset_y: f32) -> Option<ScrollAnchor> {
        let (key, top, height) = self
            .children
            .iter()
            .find(|(_, top, _)| *top >= offset_y - 0.5)
            .or_else(|| {
                self.children
                    .iter()
                    .rev()
                    .find(|(_, top, _)| *top < offset_y)
            })?;
        Some(ScrollAnchor {
            key: key.clone(),
            offset_from_top: top - offset_y,
            height: *height,
        })
    }

    fn apply_anchor(
        anchor: &mut ScrollAnchor,
        top: f32,
        height: f32,
        children_changed: bool,
    ) -> (f32, bool) {
        let offset =
            if children_changed && anchor.height > 0.0 && (height - anchor.height).abs() > 0.5 {
                top + height - (anchor.offset_from_top + anchor.height)
            } else {
                top - anchor.offset_from_top
            };
        let updated = ScrollAnchor {
            key: anchor.key.clone(),
            offset_from_top: top - offset,
            height,
        };
        let changed = (updated.offset_from_top - anchor.offset_from_top).abs() > 0.5
            || (updated.height - anchor.height).abs() > 0.5;
        *anchor = updated;
        (offset, changed)
    }

    fn emit(&mut self) {
        if let Some(cb) = &mut self.on_position {
            cb(&self.position);
        }
    }

    fn update_children(&mut self, children: &[ScrollChild<'_>]) -> bool {
        let keyed: Vec<(&str, f32, f32)> = children
            .iter()
            .filter_map(|c| c.key.map(|k| (k, c.top, c.height)))
            .collect();
        let keys_changed = self.children.len() != keyed.len()
            || self
                .children
                .iter()
                .zip(&keyed)
                .any(|((k, _, _), (k2, _, _))| k != k2);
        let same = !keys_changed
            && self
                .children
                .iter()
                .zip(&keyed)
                .all(|((_, t, h), (_, t2, h2))| t == t2 && h == h2);
        if same {
            return false;
        }
        let heights_changed = self.children.len() != keyed.len()
            || self
                .children
                .iter()
                .zip(&keyed)
                .any(|((k, _, h), (k2, _, h2))| k != k2 || h != h2);
        self.children = keyed
            .into_iter()
            .map(|(k, t, h)| (k.to_string(), t, h))
            .collect();
        if heights_changed {
            if let Some(cb) = &mut self.on_measure {
                let measured: Vec<(String, f32)> = self
                    .children
                    .iter()
                    .map(|(k, _, h)| (k.clone(), *h))
                    .collect();
                cb(&measured);
            }
        }
        keys_changed
    }
}

pub struct ScrollViewWidget {
    offset_y: f32,
    max_offset_y: f32,
    viewport_height: f32,
    content_height: f32,
    on_scroll: Option<Box<dyn FnMut(f32, f32)>>,

    top_fade: Option<TopFade>,
    chat: Option<ChatMode>,

    wheel: Option<WheelGlide>,

    carry: Option<WheelCarry>,

    identity: Option<String>,

    seeded: bool,

    thumb_grab: Option<f32>,

    autoscroll: Option<Autoscroll>,

    spring: Option<SpringScroll>,

    scrollbar: Cell<(ScrollbarStyle, bool)>,
}

impl Drop for ScrollViewWidget {
    fn drop(&mut self) {
        let Some(identity) = self.identity.take() else {
            return;
        };

        if self.offset_y > 0.0 && self.owns_offset() {
            PARKED_OFFSETS.with(|parked| {
                parked
                    .borrow_mut()
                    .insert(identity.clone(), (self.offset_y, Instant::now()))
            });
        }
        if let Some(spring) = self.spring.take() {
            PARKED_SPRINGS.with(|parked| {
                parked
                    .borrow_mut()
                    .insert(identity.clone(), (spring, Instant::now()))
            });
        }

        if self.carry.is_none() {
            if let Some(glide) = self.wheel.take() {
                PARKED.with(|parked| {
                    parked
                        .borrow_mut()
                        .insert(identity, (glide, Instant::now()))
                });
            }
        }
    }
}

impl ScrollViewWidget {
    pub fn new() -> Self {
        Self {
            offset_y: 0.0,
            max_offset_y: 0.0,
            viewport_height: 0.0,
            content_height: 0.0,
            on_scroll: None,
            top_fade: None,
            chat: None,
            wheel: None,
            carry: None,
            identity: None,
            seeded: false,
            thumb_grab: None,
            autoscroll: None,
            spring: None,
            scrollbar: Cell::new((ScrollbarStyle::default(), false)),
        }
    }

    pub fn with_wheel_carry(mut self, carry: WheelCarry) -> Self {
        self.wheel = carry.borrow().clone();
        self.carry = Some(carry);
        self
    }

    fn sync_carry(&self) {
        if let Some(carry) = &self.carry {
            *carry.borrow_mut() = self.wheel.clone();
        }
    }

    fn note_user_scroll(&mut self) {
        if let Some(chat) = &mut self.chat {
            let position = if chat.children.is_empty() {
                chat.position.clone()
            } else if self.offset_y >= self.max_offset_y - chat.pin_threshold {
                ScrollPosition::Pinned
            } else {
                chat.find_anchor(self.offset_y)
                    .map_or(chat.position.clone(), ScrollPosition::Anchored)
            };
            if position != chat.position {
                chat.position = position;
                chat.emit();
            }
        }
        if let Some(on_scroll) = &mut self.on_scroll {
            on_scroll(self.offset_y, self.max_offset_y);
        }
    }

    fn tick_wheel(&mut self) {
        let Some(glide) = &mut self.wheel else { return };
        let now = Instant::now();
        let before = self.offset_y;
        let value = glide.anim.value(now);
        let step = value - glide.applied;
        glide.applied = value;
        self.offset_y = (self.offset_y + step).clamp(0.0, self.max_offset_y);
        if glide.anim.finished(now) {
            self.wheel = None;
        }
        self.sync_carry();
        if self.offset_y != before {
            self.note_user_scroll();
        }
    }

    pub fn with_chat_position(mut self, position: ScrollPosition, pin_threshold: f32) -> Self {
        self.chat = Some(ChatMode {
            position,
            pin_threshold,
            children: Vec::new(),
            last_max_offset: -1.0,
            on_position: None,
            on_measure: None,
        });
        self
    }

    pub fn set_chat_position(&mut self, position: ScrollPosition) {
        let Some(chat) = &mut self.chat else { return };
        if chat.position == position {
            return;
        }
        chat.position = position;
        chat.last_max_offset = -1.0;
        self.wheel = None;
        self.spring = None;
        self.sync_carry();
    }

    pub fn with_on_position(mut self, on_position: impl FnMut(&ScrollPosition) + 'static) -> Self {
        if let Some(chat) = &mut self.chat {
            chat.on_position = Some(Box::new(on_position));
        }
        self
    }

    pub fn with_on_measure(mut self, on_measure: impl FnMut(&[(String, f32)]) + 'static) -> Self {
        if let Some(chat) = &mut self.chat {
            chat.on_measure = Some(Box::new(on_measure));
        }
        self
    }

    pub fn with_top_fade(mut self, fade: impl Into<TopFade>) -> Self {
        self.top_fade = Some(fade.into());
        self
    }

    pub fn with_initial_offset(mut self, offset_y: f32) -> Self {
        self.offset_y = offset_y;

        self.seeded = offset_y != 0.0;
        self
    }

    fn scrollbar_geometry(&self, rect: AbsoluteRect) -> Option<((f32, f32), (f32, f32))> {
        if self.max_offset_y <= 0.0 || self.content_height <= 0.0 {
            return None;
        }
        let (scrollbar, _) = self.scrollbar.get();
        let (track, _, _) = scrollbar_metrics(scrollbar.kind)?;
        let thumb_height = (self.viewport_height * self.viewport_height / self.content_height)
            .max(THUMB_MIN_HEIGHT)
            .min(self.viewport_height);
        let travel = self.viewport_height - thumb_height;
        let thumb_top = rect.y + travel * (self.offset_y / self.max_offset_y);
        Some((
            (rect.x + rect.width - track, track),
            (thumb_top, thumb_height),
        ))
    }

    fn set_offset_by_user(&mut self, offset: f32) {
        self.wheel = None;
        self.spring = None;
        self.sync_carry();
        let offset = offset.clamp(0.0, self.max_offset_y);
        if offset != self.offset_y {
            self.offset_y = offset;
            self.note_user_scroll();
        }
    }

    fn owns_offset(&self) -> bool {
        !self.seeded && self.chat.is_none() && self.on_scroll.is_none()
    }

    pub fn with_on_scroll(mut self, on_scroll: impl FnMut(f32, f32) + 'static) -> Self {
        self.on_scroll = Some(Box::new(on_scroll));
        self
    }
}

impl Default for ScrollViewWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ScrollViewWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
        self.scrollbar.set((style.scrollbar, style.is_hovered));

        if self.wheel.is_some() || self.autoscroll.is_some() || self.spring.is_some() {
            crate_paint::animate_region(canvas, bounds);
        }

        paint_background(canvas, bounds, style);

        if let (Some(color), width) = (style.border_top_color, style.border_top_width) {
            if width > 0.0 {
                let mut paint = Paint::default();
                paint.set_color(color);
                paint.set_anti_alias(true);
                paint.set_style(PaintStyle::Stroke);
                paint.set_stroke_width(width);
                let y = bounds.top + width / 2.0;
                canvas.draw_line((bounds.left, y), (bounds.right, y), &paint);
            }
        }
    }

    fn paint_overlay(&self, canvas: &Canvas, rect: AbsoluteRect) {
        if let Some(fade) = &self.top_fade {
            if self.offset_y > 0.0 {
                let color = match fade {
                    TopFade::Color(color) => *color,
                    TopFade::Token(name) => {
                        colors::token(name, Color::TRANSPARENT)
                    }
                };
                let opaque = Color4f::from(color);
                let transparent = Color4f::new(opaque.r, opaque.g, opaque.b, 0.0);
                let colors = [opaque, transparent];
                let gradient_colors = Colors::new(&colors, None, TileMode::Clamp, None);
                let gradient = Gradient::new(gradient_colors, Interpolation::default());
                let top = (
                    Point::new(rect.x, rect.y),
                    Point::new(rect.x, rect.y + TOP_FADE_HEIGHT),
                );
                if let Some(shader) = linear_gradient(top, &gradient, None) {
                    let mut paint = Paint::default();
                    paint.set_shader(shader);
                    paint.set_anti_alias(true);
                    let width = (rect.width - TOP_FADE_RIGHT_INSET).max(0.0);
                    canvas.draw_rect(
                        Rect::from_xywh(rect.x, rect.y, width, TOP_FADE_HEIGHT),
                        &paint,
                    );
                }
            }
        }

        if let Some(autoscroll) = &self.autoscroll {
            paint_autoscroll_marker(canvas, autoscroll);
        }

        let (scrollbar, hovered) = self.scrollbar.get();
        let Some(((track_x, track), (thumb_y, thumb_height))) = self.scrollbar_geometry(rect)
        else {
            return;
        };
        let Some((_, inset, color)) = scrollbar_metrics(scrollbar.kind) else {
            return;
        };

        if scrollbar.fade && !hovered && self.thumb_grab.is_none() {
            return;
        }

        let thumb_width = track - inset * 2.0;
        let thumb_x = track_x + inset;

        let mut paint = Paint::new(Color4f::from(color), None);
        paint.set_anti_alias(true);

        let bounds = Rect::from_xywh(thumb_x, thumb_y, thumb_width, thumb_height);
        canvas.draw_rrect(
            RRect::new_rect_xy(bounds, thumb_width / 2.0, thumb_width / 2.0),
            &paint,
        );
    }

    fn relayout_after(&self, event: &UiEvent) -> bool {
        !matches!(event, UiEvent::Scroll { .. } | UiEvent::MouseMoved { .. })
    }

    fn wants_pointer_capture(&self) -> bool {
        self.thumb_grab.is_some() || self.autoscroll.is_some()
    }

    fn scroll_to_offset(&mut self, offset: f32) {
        self.wheel = None;
        self.sync_carry();
        self.spring = Some(SpringScroll::new(self.offset_y, offset.max(0.0)));
    }

    fn intercepts_pointer(&self, event: &UiEvent, rect: AbsoluteRect) -> bool {
        let UiEvent::MouseDown { x, y, button, .. } = event else {
            return false;
        };

        if self.autoscroll.is_some() {
            return true;
        }
        if *button != MouseButton::Left {
            return false;
        }
        self.scrollbar_geometry(rect)
            .is_some_and(|((track_x, track), _)| {
                *x >= track_x && *x < track_x + track && *y >= rect.y && *y < rect.y + rect.height
            })
    }

    fn identify(&mut self, identity: &str) {
        self.identity = Some(identity.to_string());
        if let Some((spring, _)) = PARKED_SPRINGS
            .with(|parked| parked.borrow_mut().remove(identity))
            .filter(|(_, at)| at.elapsed() < PARKED_FOR)
        {
            self.spring = Some(spring);
        }
        if self.owns_offset() {
            let parked = PARKED_OFFSETS.with(|parked| parked.borrow_mut().remove(identity));
            if let Some((offset, _)) = parked.filter(|(_, at)| at.elapsed() < PARKED_FOR) {
                self.offset_y = offset;
            }
        }
        if self.carry.is_none() && self.wheel.is_none() {
            self.wheel = PARKED
                .with(|parked| parked.borrow_mut().remove(identity))
                .filter(|(_, at)| at.elapsed() < PARKED_FOR)
                .map(|(glide, _)| glide);
        }
    }

    fn handle_event(&mut self, event: &UiEvent, rect: AbsoluteRect) -> EventResult {
        match *event {
            UiEvent::MouseDown { x, y, button, .. } => {
                if self.autoscroll.take().is_some() {
                    crate_paint::request_repaint_rect(rect);
                    return EventResult::Handled;
                }
                if button == MouseButton::Left {
                    if let Some(((track_x, track), (thumb_top, thumb_height))) =
                        self.scrollbar_geometry(rect)
                    {
                        if x >= track_x
                            && x < track_x + track
                            && y >= rect.y
                            && y < rect.y + rect.height
                        {
                            if y >= thumb_top && y < thumb_top + thumb_height {
                                self.thumb_grab = Some(y - thumb_top);
                            } else {
                                let page = self.viewport_height;
                                self.set_offset_by_user(if y < thumb_top {
                                    self.offset_y - page
                                } else {
                                    self.offset_y + page
                                });
                            }
                            return EventResult::Handled;
                        }
                    }
                }
                if button == MouseButton::Middle && self.max_offset_y > 0.0 {
                    self.wheel = None;
                    self.spring = None;
                    self.sync_carry();
                    self.autoscroll = Some(Autoscroll {
                        anchor: (x, y),
                        pointer_y: y,
                        last_tick: Instant::now(),
                        dragged: false,
                    });
                    return EventResult::Handled;
                }
                return EventResult::Ignored;
            }
            UiEvent::MouseMoved { y, .. } => {
                if let Some(grab) = self.thumb_grab {
                    if let Some((_, (_, thumb_height))) = self.scrollbar_geometry(rect) {
                        let travel = (self.viewport_height - thumb_height).max(1.0);
                        self.set_offset_by_user((y - rect.y - grab) / travel * self.max_offset_y);
                    }
                    return EventResult::Handled;
                }
                if let Some(autoscroll) = &mut self.autoscroll {
                    autoscroll.pointer_y = y;
                    autoscroll.dragged |= (y - autoscroll.anchor.1).abs() > AUTOSCROLL_DEAD_ZONE;
                    return EventResult::Handled;
                }
                return EventResult::Ignored;
            }
            UiEvent::MouseUp { button, .. } => {
                if self.thumb_grab.take().is_some() {
                    return EventResult::Handled;
                }

                if button == MouseButton::Middle && self.autoscroll.is_some_and(|a| a.dragged) {
                    self.autoscroll = None;
                    return EventResult::Handled;
                }
                return if self.autoscroll.is_some() {
                    EventResult::Handled
                } else {
                    EventResult::Ignored
                };
            }
            _ => {}
        }
        let UiEvent::Scroll {
            delta_y, precise, ..
        } = event
        else {
            return EventResult::Ignored;
        };
        self.spring = None;

        if !precise && smooth_scrolling() && self.max_offset_y > 0.0 {
            let now = Instant::now();

            let heading = self.offset_y
                + self
                    .wheel
                    .as_ref()
                    .map_or(0.0, |g| g.anim.target() - g.applied);
            let target = (heading - delta_y).clamp(0.0, self.max_offset_y);
            match &mut self.wheel {
                Some(glide) if !glide.anim.finished(now) => glide
                    .anim
                    .retarget(now, glide.applied + (target - self.offset_y)),
                _ => {
                    self.wheel = Some(WheelGlide {
                        anim: WheelAnimation::new(0.0, target - self.offset_y, now),
                        applied: 0.0,
                    })
                }
            }
            self.sync_carry();
            return EventResult::Handled;
        }
        self.wheel = None;
        self.sync_carry();
        self.offset_y = (self.offset_y - delta_y).clamp(0.0, self.max_offset_y);
        self.note_user_scroll();
        EventResult::Handled
    }

    fn scroll_offset(&self) -> Option<(f32, f32)> {
        Some((0.0, self.offset_y))
    }

    fn wants_scroll_children(&self) -> bool {
        self.chat.is_some()
    }

    fn update_scroll_bounds(
        &mut self,
        viewport: (f32, f32),
        content: (f32, f32),
        children: &[ScrollChild<'_>],
    ) {
        self.viewport_height = viewport.1;
        self.content_height = content.1;
        self.max_offset_y = (content.1 - viewport.1).max(0.0);
        if let Some(chat) = &mut self.chat {
            let children_changed = chat.update_children(children);
            let mut remeasured = false;
            match &mut chat.position {
                ScrollPosition::Pinned => {
                    if self.max_offset_y != chat.last_max_offset {
                        self.offset_y = self.max_offset_y;
                    }
                }
                ScrollPosition::Anchored(anchor) => {
                    let found = chat
                        .children
                        .iter()
                        .find(|(k, _, _)| *k == anchor.key)
                        .map(|(_, top, height)| (*top, *height));
                    if let Some((top, height)) = found {
                        let (offset, changed) =
                            ChatMode::apply_anchor(anchor, top, height, children_changed);
                        self.offset_y = offset;
                        remeasured = changed;
                    }
                }
            }

            if remeasured {
                chat.emit();
            }
            chat.last_max_offset = self.max_offset_y;
        }
        self.offset_y = self.offset_y.clamp(0.0, self.max_offset_y);

        self.tick_wheel();
        self.tick_spring();
        self.tick_autoscroll();
    }
}

impl ScrollViewWidget {
    fn tick_spring(&mut self) {
        if let Some(identity) = &self.identity {
            if let Some(target) = scroll_animation::take_scroll_to(identity) {
                self.wheel = None;
                self.sync_carry();
                self.spring = Some(SpringScroll::new(self.offset_y, target));
            }
        }
        let max = self.max_offset_y;
        let Some(spring) = &mut self.spring else {
            return;
        };
        spring.clamp_target(max);
        let (value, done) = spring.advance(Instant::now());
        let before = self.offset_y;
        self.offset_y = value.clamp(0.0, max);
        if done {
            self.spring = None;
        }
        if self.offset_y != before {
            scroll_animation::set_programmatic(true);
            self.note_user_scroll();
            scroll_animation::set_programmatic(false);
        }
    }

    fn tick_autoscroll(&mut self) {
        let Some(autoscroll) = &mut self.autoscroll else {
            return;
        };
        let now = Instant::now();

        let dt = now
            .duration_since(autoscroll.last_tick)
            .as_secs_f32()
            .min(0.05);
        autoscroll.last_tick = now;
        let step = autoscroll.velocity() * dt;
        if step != 0.0 {
            let offset = self.offset_y + step;
            self.set_offset_by_user(offset);
        }
    }
}

#[cfg(test)]
mod wheel_tests {
    use crate::paint as crate_paint;
    use skia_safe::surfaces;
    use std::sync::Mutex;
    use std::thread;
    use std::time::Duration;

    use super::*;

    static SWITCH: Mutex<()> = Mutex::new(());

    fn view() -> ScrollViewWidget {
        let mut view = ScrollViewWidget::new();
        view.update_scroll_bounds((300.0, 400.0), (300.0, 2000.0), &[]);
        view
    }

    fn notch(view: &mut ScrollViewWidget, delta_y: f32, precise: bool) {
        let rect = AbsoluteRect {
            x: 0.0,
            y: 0.0,
            width: 300.0,
            height: 400.0,
        };
        view.handle_event(
            &UiEvent::Scroll {
                x: 10.0,
                y: 10.0,
                delta_x: 0.0,
                delta_y,
                precise,
            },
            rect,
        );
    }

    #[test]
    fn a_wheel_notch_glides_to_100px_and_a_precise_delta_jumps() {
        let _turn = SWITCH.lock().unwrap_or_else(|e| e.into_inner());
        let mut v = view();
        notch(&mut v, -100.0, false);
        assert_eq!(
            v.offset_y, 0.0,
            "nothing moves until the frame that follows the notch"
        );
        v.update_scroll_bounds((300.0, 400.0), (300.0, 2000.0), &[]);
        assert!(
            v.offset_y < 100.0,
            "the first frame is the start of a glide ({})",
            v.offset_y
        );
        thread::sleep(Duration::from_millis(80));
        v.update_scroll_bounds((300.0, 400.0), (300.0, 2000.0), &[]);
        assert!(
            v.offset_y > 0.0 && v.offset_y < 100.0,
            "mid-glide ({})",
            v.offset_y
        );
        thread::sleep(Duration::from_millis(260));
        v.update_scroll_bounds((300.0, 400.0), (300.0, 2000.0), &[]);
        assert_eq!(v.offset_y, 100.0, "one notch is 100px");
        assert!(v.wheel.is_none(), "the glide is over");

        notch(&mut v, -30.0, true);
        assert_eq!(v.offset_y, 130.0);
    }

    #[test]
    fn a_wheel_glide_damages_only_its_viewport() {
        let _turn = SWITCH.lock().unwrap_or_else(|e| e.into_inner());
        let _ = crate_paint::take_animation();
        let mut v = view();
        notch(&mut v, -100.0, false);
        let mut surface = surfaces::raster_n32_premul((300, 400)).unwrap();
        let rect = AbsoluteRect {
            x: 0.0,
            y: 0.0,
            width: 300.0,
            height: 400.0,
        };
        v.paint(surface.canvas(), rect, &ComputedStyle::default());
        let animation = crate_paint::take_animation();
        assert!(
            !animation.full,
            "scrolling must not repaint the whole window"
        );
        assert_eq!(
            animation.regions.len(),
            1,
            "the scroll viewport is the one damaged region"
        );
    }

    #[test]
    fn notches_accumulate_while_gliding_and_stop_at_the_end() {
        let _turn = SWITCH.lock().unwrap_or_else(|e| e.into_inner());
        let mut v = view();
        notch(&mut v, -100.0, false);
        notch(&mut v, -100.0, false);
        notch(&mut v, -100.0, false);
        thread::sleep(Duration::from_millis(600));
        v.update_scroll_bounds((300.0, 400.0), (300.0, 2000.0), &[]);
        assert_eq!(v.offset_y, 300.0);

        for _ in 0..5 {
            notch(&mut v, 100.0, false);
        }
        thread::sleep(Duration::from_millis(700));
        v.update_scroll_bounds((300.0, 400.0), (300.0, 2000.0), &[]);
        assert_eq!(v.offset_y, 0.0);
    }

    #[test]
    fn a_glide_carries_on_in_a_rebuilt_view_even_around_a_different_origin() {
        let _turn = SWITCH.lock().unwrap_or_else(|e| e.into_inner());
        let carry: WheelCarry = Rc::new(RefCell::new(None));
        let mut first = view().with_wheel_carry(carry.clone());
        notch(&mut first, -100.0, false);
        thread::sleep(Duration::from_millis(60));
        first.update_scroll_bounds((300.0, 400.0), (300.0, 2000.0), &[]);
        let moved = first.offset_y;
        assert!(moved > 0.0 && moved < 100.0, "part of the way ({moved})");
        assert!(carry.borrow().is_some(), "the glide is parked in the carry");

        let mut rebuilt = ScrollViewWidget::new()
            .with_initial_offset(500.0 + moved)
            .with_wheel_carry(carry.clone());
        rebuilt.update_scroll_bounds((300.0, 400.0), (300.0, 2000.0), &[]);
        thread::sleep(Duration::from_millis(500));
        rebuilt.update_scroll_bounds((300.0, 400.0), (300.0, 2000.0), &[]);

        assert!(
            (rebuilt.offset_y - 600.0).abs() < 0.01,
            "it finished the full 100px from where it was ({})",
            rebuilt.offset_y
        );
        assert!(carry.borrow().is_none(), "and cleared the carry");
    }

    #[test]
    fn a_notch_during_a_carried_glide_extends_it() {
        let _turn = SWITCH.lock().unwrap_or_else(|e| e.into_inner());
        let carry: WheelCarry = Rc::new(RefCell::new(None));
        let mut first = view().with_wheel_carry(carry.clone());
        notch(&mut first, -100.0, false);
        let mut rebuilt = view().with_wheel_carry(carry.clone());
        notch(&mut rebuilt, -100.0, false);
        thread::sleep(Duration::from_millis(600));
        rebuilt.update_scroll_bounds((300.0, 400.0), (300.0, 2000.0), &[]);
        assert_eq!(rebuilt.offset_y, 200.0);
    }

    #[test]
    fn any_rebuilt_scroller_picks_up_its_glide_by_identity_without_a_carry() {
        let _turn = SWITCH.lock().unwrap_or_else(|e| e.into_inner());
        let mut first = view();
        first.identify("test-list-scroller");
        notch(&mut first, -100.0, false);
        drop(first);

        let mut other = view();
        other.identify("another-scroller");
        assert!(other.wheel.is_none());

        let mut rebuilt = view();
        rebuilt.identify("test-list-scroller");
        assert!(
            rebuilt.wheel.is_some(),
            "the replacement carries on the glide"
        );
        thread::sleep(Duration::from_millis(400));
        rebuilt.update_scroll_bounds((300.0, 400.0), (300.0, 2000.0), &[]);
        assert_eq!(rebuilt.offset_y, 100.0);
    }

    #[test]
    fn a_rebuilt_scroller_keeps_its_offset_unless_the_caller_tracks_it() {
        let mut first = view();
        first.identify("offset-scroller");
        notch_precise(&mut first, -250.0);
        assert_eq!(first.offset_y, 250.0);
        drop(first);

        let mut other = view();
        other.identify("another-offset-scroller");
        assert_eq!(other.offset_y, 0.0, "another scroller starts at the top");

        let mut mirrored = ScrollViewWidget::new().with_on_scroll(|_, _| {});
        mirrored.identify("offset-scroller");
        assert_eq!(
            mirrored.offset_y, 0.0,
            "a caller mirroring the position owns it"
        );

        let mut rebuilt = view();
        rebuilt.identify("offset-scroller");
        assert_eq!(rebuilt.offset_y, 250.0);
    }

    fn mouse(view: &mut ScrollViewWidget, event: UiEvent) -> EventResult {
        let rect = AbsoluteRect {
            x: 0.0,
            y: 0.0,
            width: 300.0,
            height: 400.0,
        };
        view.handle_event(&event, rect)
    }

    #[test]
    fn the_scrollbar_thumb_can_be_dragged() {
        let mut v = view();
        let rect = AbsoluteRect {
            x: 0.0,
            y: 0.0,
            width: 300.0,
            height: 400.0,
        };
        let press = UiEvent::MouseDown {
            x: 296.0,
            y: 10.0,
            button: MouseButton::Left,
            shift: false,
        };
        assert!(
            v.intercepts_pointer(&press, rect),
            "the thumb claims a press over the content"
        );
        assert!(!v.intercepts_pointer(
            &UiEvent::MouseDown {
                x: 100.0,
                y: 10.0,
                button: MouseButton::Left,
                shift: false
            },
            rect
        ));
        mouse(&mut v, press);
        assert!(v.wants_pointer_capture());

        mouse(&mut v, UiEvent::MouseMoved { x: 296.0, y: 170.0 });
        assert_eq!(v.offset_y, 800.0);
        mouse(&mut v, UiEvent::MouseMoved { x: 20.0, y: 900.0 });
        assert_eq!(v.offset_y, 1600.0, "dragging past the end clamps");
        mouse(
            &mut v,
            UiEvent::MouseUp {
                x: 20.0,
                y: 900.0,
                button: MouseButton::Left,
            },
        );
        assert!(!v.wants_pointer_capture());

        mouse(
            &mut v,
            UiEvent::MouseDown {
                x: 296.0,
                y: 5.0,
                button: MouseButton::Left,
                shift: false,
            },
        );
        assert_eq!(v.offset_y, 1200.0);
    }

    #[test]
    fn a_middle_click_autoscrolls_until_the_next_press() {
        let mut v = view();
        mouse(
            &mut v,
            UiEvent::MouseDown {
                x: 50.0,
                y: 100.0,
                button: MouseButton::Middle,
                shift: false,
            },
        );
        mouse(
            &mut v,
            UiEvent::MouseUp {
                x: 50.0,
                y: 100.0,
                button: MouseButton::Middle,
            },
        );
        assert!(v.wants_pointer_capture(), "a click leaves it on");
        mouse(&mut v, UiEvent::MouseMoved { x: 50.0, y: 105.0 });
        thread::sleep(Duration::from_millis(40));
        v.update_scroll_bounds((300.0, 400.0), (300.0, 2000.0), &[]);
        assert_eq!(v.offset_y, 0.0, "inside the dead zone nothing moves");
        mouse(&mut v, UiEvent::MouseMoved { x: 50.0, y: 200.0 });
        thread::sleep(Duration::from_millis(40));
        v.update_scroll_bounds((300.0, 400.0), (300.0, 2000.0), &[]);
        assert!(
            v.offset_y > 0.0,
            "below the anchor scrolls down ({})",
            v.offset_y
        );
        mouse(
            &mut v,
            UiEvent::MouseDown {
                x: 10.0,
                y: 10.0,
                button: MouseButton::Left,
                shift: false,
            },
        );
        assert!(!v.wants_pointer_capture(), "any press ends it");

        mouse(
            &mut v,
            UiEvent::MouseDown {
                x: 50.0,
                y: 100.0,
                button: MouseButton::Middle,
                shift: false,
            },
        );
        mouse(&mut v, UiEvent::MouseMoved { x: 50.0, y: 200.0 });
        mouse(
            &mut v,
            UiEvent::MouseUp {
                x: 50.0,
                y: 200.0,
                button: MouseButton::Middle,
            },
        );
        assert!(!v.wants_pointer_capture());
    }

    fn notch_precise(view: &mut ScrollViewWidget, delta_y: f32) {
        notch(view, delta_y, true);
    }

    #[test]
    fn a_stale_glide_is_not_adopted_by_a_much_later_view() {
        let _turn = SWITCH.lock().unwrap_or_else(|e| e.into_inner());
        let mut first = view();
        first.identify("stale-scroller");
        notch(&mut first, -100.0, false);
        drop(first);
        thread::sleep(PARKED_FOR + Duration::from_millis(50));
        let mut later = view();
        later.identify("stale-scroller");
        assert!(
            later.wheel.is_none(),
            "the scroller it belonged to went away for good"
        );
    }

    #[test]
    fn smooth_scrolling_can_be_switched_off() {
        let _turn = SWITCH.lock().unwrap_or_else(|e| e.into_inner());
        let mut v = view();
        scroll_animation::set_smooth_scrolling(false);
        notch(&mut v, -100.0, false);
        scroll_animation::set_smooth_scrolling(true);
        assert_eq!(v.offset_y, 100.0, "with smoothing off a notch jumps");
        assert!(v.wheel.is_none());
    }
}

#[cfg(test)]
mod scrollbar_tests {
    use crate::layout;
    use crate::node::Node;
    use crate::paint as crate_paint;
    use crate::style::cascade;
    use crate::style::parser;
    use taffy::{AvailableSpace, Size};

    use super::*;
    use crate::components::div;
    use crate::widget::WidgetTree;
    use crate::widget::primitives::BoxWidget;

    fn content_width(scroller_css: &str) -> f32 {
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let scroller = tree.mount(
            tree.root,
            Node::new("div", ScrollViewWidget::new())
                .class("scroller")
                .child(div().class("content")),
        );
        let theme = parser::parse_stylesheet(&format!(
            ".scroller {{ display: flex; flex-direction: column; width: 300px; height: 200px; overflow-y: scroll; {scroller_css} }} .content {{ width: 100%; height: 500px; flex-shrink: 0; }}"
        ));
        cascade::apply_stylesheet(&mut tree, &theme);
        layout::compute_layout(
            &mut tree,
            Size {
                width: AvailableSpace::Definite(300.0),
                height: AvailableSpace::Definite(200.0),
            },
        );
        let rects = crate_paint::all_absolute_rects(&tree);
        rects[&tree.nodes[scroller].children[0]].width
    }

    #[test]
    fn the_scrollbar_track_is_reserved_layout_space() {
        assert_eq!(
            content_width(""),
            300.0,
            "no declaration: the bar is drawn over the content, as before"
        );
        assert_eq!(content_width("scrollbar-width: thin;"), 292.0);
        assert_eq!(content_width("scrollbar-width: auto;"), 284.0);
        assert_eq!(content_width("scrollbar-width: none;"), 300.0);
    }

    #[test]
    fn scrollbar_properties_cascade() {
        let sheet = |css: &str| {
            let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
            let id = tree.mount(
                tree.root,
                Node::new("div", ScrollViewWidget::new()).class("s"),
            );
            cascade::apply_stylesheet(
                &mut tree,
                &parser::parse_stylesheet(css),
            );
            tree.nodes[id].style.scrollbar
        };
        assert_eq!(
            sheet(".s { }"),
            ScrollbarStyle {
                kind: ScrollbarKind::Legacy,
                fade: false
            }
        );
        assert_eq!(
            sheet(".s { scrollbar-width: auto; }"),
            ScrollbarStyle {
                kind: ScrollbarKind::Auto,
                fade: false
            }
        );
        assert_eq!(
            sheet(".s { scrollbar-width: thin; scrollbar-fade: hover; }"),
            ScrollbarStyle {
                kind: ScrollbarKind::Thin,
                fade: true
            }
        );
        assert_eq!(
            sheet(".s { scrollbar-width: none; }").kind,
            ScrollbarKind::None
        );
    }
}
