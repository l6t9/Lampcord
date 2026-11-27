use crate::classes::slider as cls;
use crate::paint as crate_paint;
use std::cell::Cell;
use std::rc::Rc;

use skia_safe::{Canvas, Color, MaskFilter, Paint, PaintStyle, RRect, Rect};

use crate::event::{EventResult, MouseButton, UiEvent};
use crate::node::Node;
use crate::style::ComputedStyle;
use crate::widget::{AbsoluteRect, Widget};

const INSET: f32 = 5.0;
const BAR: f32 = 4.0;
const GRABBER: f32 = 16.0;

struct SliderWidget {
    value: Cell<f32>,
    min: f32,
    max: f32,
    on_change: Rc<dyn Fn(f32)>,
    overlay: Option<Rc<dyn Fn() -> f32>>,
    disabled: bool,
    dragging: bool,
}

impl SliderWidget {
    fn fraction(&self) -> f32 {
        let span = (self.max - self.min).max(f32::EPSILON);
        ((self.value.get() - self.min) / span).clamp(0.0, 1.0)
    }

    fn pick(&self, x: f32, rect: AbsoluteRect) {
        let fraction = ((x - rect.x - INSET) / (rect.width - 2.0 * INSET).max(1.0)).clamp(0.0, 1.0);
        let next = self.min + fraction * (self.max - self.min);
        self.value.set(next);
        (self.on_change)(next);
        crate_paint::request_repaint();
    }
}

impl Widget for SliderWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let alpha = if self.disabled { 0.5 } else { 1.0 };
        let with_alpha = |color: Color| color.with_a((color.a() as f32 * alpha) as u8);
        let fraction = self.fraction();
        let bar = Rect::from_xywh(rect.x, rect.y + (rect.height - BAR) / 2.0, rect.width, BAR);
        let bar_rrect = RRect::new_rect_xy(bar, BAR, BAR);

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(with_alpha(style.background.unwrap_or(Color::DARK_GRAY)));
        canvas.draw_rrect(bar_rrect, &paint);

        canvas.save();
        canvas.clip_rrect(bar_rrect, None, true);
        let fill = Rect::from_xywh(bar.left, bar.top, fraction * bar.width(), BAR);
        if fill.width() > 0.0 {
            paint.set_color(with_alpha(style.text_color.unwrap_or(Color::WHITE)));
            canvas.draw_rect(fill, &paint);
            let mut edge = Paint::default();
            edge.set_anti_alias(true);
            edge.set_style(PaintStyle::Stroke);
            edge.set_stroke_width(1.0);
            edge.set_color(with_alpha(Color::from_argb(26, 255, 255, 255)));
            canvas.draw_rect(fill.with_inset((0.5, 0.5)), &edge);
        }
        if let Some(overlay) = &self.overlay {
            let shade = overlay().clamp(0.0, 1.0);
            if shade > 0.0 {
                paint.set_color(
                    style
                        .border_left_color
                        .unwrap_or(Color::from_argb(61, 0, 0, 0)),
                );
                canvas.draw_rect(
                    Rect::from_xywh(bar.left, bar.top, shade * bar.width(), BAR),
                    &paint,
                );
            }
            crate_paint::animate_region(
                canvas,
                Rect::from_xywh(rect.x, rect.y, rect.width, rect.height),
            );
        }
        canvas.restore();

        let center_x = rect.x + INSET + fraction * (rect.width - 2.0 * INSET);
        let center_y = rect.y + rect.height / 2.0;
        let grabber = Rect::from_xywh(
            center_x - GRABBER / 2.0,
            center_y - GRABBER / 2.0,
            GRABBER,
            GRABBER,
        );
        let mut shadow = Paint::default();
        shadow.set_anti_alias(true);
        shadow.set_color(with_alpha(Color::from_argb(36, 0, 0, 0)));
        shadow.set_mask_filter(MaskFilter::blur(skia_safe::BlurStyle::Normal, 2.0, false));
        canvas.draw_oval(grabber.with_offset((0.0, 1.0)), &shadow);
        paint.set_color(with_alpha(Color::WHITE));
        canvas.draw_oval(grabber, &paint);
        let mut ring = Paint::default();
        ring.set_anti_alias(true);
        ring.set_style(PaintStyle::Stroke);
        ring.set_stroke_width(1.0);
        ring.set_color(with_alpha(
            style
                .outline_color
                .unwrap_or(Color::from_argb(51, 0x97, 0x97, 0x9f)),
        ));
        canvas.draw_oval(grabber.with_inset((0.5, 0.5)), &ring);
    }

    fn handle_event(&mut self, event: &UiEvent, rect: AbsoluteRect) -> EventResult {
        if self.disabled {
            return EventResult::Ignored;
        }
        match *event {
            UiEvent::MouseDown {
                x,
                button: MouseButton::Left,
                ..
            } => {
                self.dragging = true;
                self.pick(x, rect);
                EventResult::Handled
            }
            UiEvent::MouseMoved { x, .. } if self.dragging => {
                self.pick(x, rect);
                EventResult::Handled
            }
            UiEvent::MouseUp { .. } if self.dragging => {
                self.dragging = false;
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }

    fn wants_pointer_capture(&self) -> bool {
        self.dragging
    }
}

pub struct SliderProps {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub on_change: Rc<dyn Fn(f32)>,

    /// Second fill, read each frame, drawn under the value. Used for a live level.
    pub overlay: Option<Rc<dyn Fn() -> f32>>,

    /// Extra class on the track.
    pub bar_class: Option<&'static str>,
    pub disabled: bool,
}

impl Default for SliderProps {
    fn default() -> Self {
        Self {
            value: 0.0,
            min: 0.0,
            max: 100.0,
            on_change: Rc::new(|_| {}),
            overlay: None,
            bar_class: None,
            disabled: false,
        }
    }
}

crate::builder_fields!(SliderProps {
    value: f32,
    min: f32,
    max: f32,
    opt bar_class: &'static str,
    disabled: bool,
});

impl SliderProps {
    pub fn on_change(mut self, on_change: impl Fn(f32) + 'static) -> Self {
        self.on_change = Rc::new(on_change);
        self
    }

    pub fn overlay(mut self, overlay: impl Fn() -> f32 + 'static) -> Self {
        self.overlay = Some(Rc::new(overlay));
        self
    }

    pub fn build(self) -> Node {
        slider(self)
    }
}

pub fn slider(props: SliderProps) -> Node {
    let span = props.max - props.min;
    let value = if span == 0.0 {
        props.min
    } else {
        props.value.clamp(props.min, props.max)
    };
    let mut node = Node::new(
        "div",
        SliderWidget {
            value: Cell::new(value),
            min: props.min,
            max: props.max,
            on_change: props.on_change,
            overlay: props.overlay,
            disabled: props.disabled,
            dragging: false,
        },
    )
    .class(cls::SLIDER);
    if let Some(class) = props.bar_class {
        node = node.class(class);
    }
    node
}
