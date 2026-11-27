use crate::classes::color_picker as cls;
use crate::font_system;
use crate::icons;
use crate::paint as crate_paint;
use std::cell::Cell;
use std::rc::Rc;

use skia_safe::gradient::shaders::linear_gradient;
use skia_safe::gradient::{Colors, Gradient, Interpolation};
use skia_safe::{Canvas, Color, Color4f, Paint, PaintStyle, Point, RRect, Rect, TileMode};

use crate::event::{EventResult, MouseButton, UiEvent};
use crate::font_system::TextFonts;
use crate::node::Node;
use crate::signal::{Signal, signal};
use crate::style::ComputedStyle;
use crate::widget::primitives::{ButtonWidget, corner_radii, has_any_corner_radius};
use crate::widget::{AbsoluteRect, Widget};

use super::container::div;
use super::text::{TextProps, text};
use super::text_input::{TextInputProps, input_accessory};
use crate::colors::INPUT_PLACEHOLDER;

pub fn parse_hex_color(text: &str) -> Option<u32> {
    let hex = text.trim().trim_start_matches('#');
    match hex.len() {
        6 => u32::from_str_radix(hex, 16).ok(),
        3 => {
            let v = u32::from_str_radix(hex, 16).ok()?;
            let (r, g, b) = ((v >> 8) & 0xf, (v >> 4) & 0xf, v & 0xf);
            Some(((r * 17) << 16) | ((g * 17) << 8) | (b * 17))
        }
        _ => None,
    }
}

pub fn format_hex_color(rgb: u32) -> String {
    format!("#{:06x}", rgb & 0xffffff)
}

pub fn rgb_color(rgb: u32) -> Color {
    Color::from_rgb(
        ((rgb >> 16) & 0xff) as u8,
        ((rgb >> 8) & 0xff) as u8,
        (rgb & 0xff) as u8,
    )
}

pub type Hsv = (f32, f32, f32);

pub fn rgb_to_hsv(rgb: u32) -> Hsv {
    let (r, g, b) = (
        ((rgb >> 16) & 0xff) as f32 / 255.0,
        ((rgb >> 8) & 0xff) as f32 / 255.0,
        (rgb & 0xff) as f32 / 255.0,
    );
    let max = r.max(g).max(b);
    let delta = max - r.min(g).min(b);
    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    (
        (hue + 360.0) % 360.0,
        if max == 0.0 { 0.0 } else { delta / max },
        max,
    )
}

pub fn hsv_to_rgb((h, s, v): Hsv) -> u32 {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h / 60.0) as u32 % 6 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let channel = |value: f32| ((value + m) * 255.0).round().clamp(0.0, 255.0) as u32;
    (channel(r) << 16) | (channel(g) << 8) | channel(b)
}

struct Shared {
    hsv: Cell<Hsv>,

    hex: Signal<String>,
    on_change: Rc<dyn Fn(u32)>,
}

impl Shared {
    fn rgb(&self) -> u32 {
        hsv_to_rgb(self.hsv.get())
    }

    fn commit(&self) {
        let rgb = self.rgb();
        self.hex.set(format!("{:06x}", rgb));
        (self.on_change)(rgb);
        crate_paint::request_repaint();
    }

    fn set_rgb(&self, rgb: u32, update_text: bool) {
        let (mut h, s, v) = rgb_to_hsv(rgb);
        if s == 0.0 {
            h = self.hsv.get().0;
        }
        self.hsv.set((h, s, v));
        if update_text {
            self.hex.set(format!("{:06x}", rgb));
        }
        (self.on_change)(rgb);
        crate_paint::request_repaint();
    }
}

fn fill_rounded(canvas: &Canvas, bounds: Rect, style: &ComputedStyle, paint: &Paint) {
    if has_any_corner_radius(style) {
        canvas.draw_rrect(RRect::new_rect_radii(bounds, &corner_radii(style)), paint);
    } else {
        canvas.draw_rect(bounds, paint);
    }
}

struct SaturationFieldWidget {
    shared: Rc<Shared>,
    dragging: bool,
}

impl SaturationFieldWidget {
    fn pick(&self, x: f32, y: f32, rect: AbsoluteRect) {
        let (h, _, _) = self.shared.hsv.get();
        let s = ((x - rect.x) / rect.width.max(1.0)).clamp(0.0, 1.0);
        let v = 1.0 - ((y - rect.y) / rect.height.max(1.0)).clamp(0.0, 1.0);
        self.shared.hsv.set((h, s, v));
        self.shared.commit();
    }
}

impl Widget for SaturationFieldWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
        let (h, s, v) = self.shared.hsv.get();
        canvas.save();
        if has_any_corner_radius(style) {
            canvas.clip_rrect(
                RRect::new_rect_radii(bounds, &corner_radii(style)),
                None,
                true,
            );
        } else {
            canvas.clip_rect(bounds, None, true);
        }
        let mut paint = Paint::default();
        paint.set_color(rgb_color(hsv_to_rgb((h, 1.0, 1.0))));
        canvas.draw_rect(bounds, &paint);

        for (from, to, start, end) in [
            (
                Point::new(rect.x, rect.y),
                Point::new(rect.x + rect.width, rect.y),
                Color4f::new(1.0, 1.0, 1.0, 1.0),
                Color4f::new(1.0, 1.0, 1.0, 0.0),
            ),
            (
                Point::new(rect.x, rect.y + rect.height),
                Point::new(rect.x, rect.y),
                Color4f::new(0.0, 0.0, 0.0, 1.0),
                Color4f::new(0.0, 0.0, 0.0, 0.0),
            ),
        ] {
            let colors = [start, end];
            let gradient = Gradient::new(
                Colors::new(&colors, None, TileMode::Clamp, None),
                Interpolation::default(),
            );
            if let Some(shader) = linear_gradient((from, to), &gradient, None) {
                let mut layer = Paint::default();
                layer.set_shader(shader);
                canvas.draw_rect(bounds, &layer);
            }
        }
        canvas.restore();

        let center = Point::new(rect.x + s * rect.width, rect.y + (1.0 - v) * rect.height);
        let mut ring = Paint::default();
        ring.set_anti_alias(true);
        ring.set_style(PaintStyle::Stroke);
        ring.set_color(Color::from_argb(102, 0, 0, 0));
        ring.set_stroke_width(1.0);
        canvas.draw_circle(center, 3.6, &ring);
        ring.set_color(Color::WHITE);
        ring.set_stroke_width(1.5);
        canvas.draw_circle(center, 2.75, &ring);
        ring.set_color(Color::from_argb(77, 0, 0, 0));
        ring.set_stroke_width(1.0);
        canvas.draw_circle(center, 2.0, &ring);
    }

    fn handle_event(&mut self, event: &UiEvent, rect: AbsoluteRect) -> EventResult {
        match *event {
            UiEvent::MouseDown {
                x,
                y,
                button: MouseButton::Left,
                ..
            } => {
                self.dragging = true;
                self.pick(x, y, rect);
                EventResult::Handled
            }
            UiEvent::MouseMoved { x, y } if self.dragging => {
                self.pick(x, y, rect);
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

struct HueSliderWidget {
    shared: Rc<Shared>,
    dragging: bool,
}

const HUE_INSET: f32 = 2.0;

impl HueSliderWidget {
    fn pick(&self, x: f32, rect: AbsoluteRect) {
        let (_, s, v) = self.shared.hsv.get();
        let fraction =
            ((x - rect.x - HUE_INSET) / (rect.width - 2.0 * HUE_INSET).max(1.0)).clamp(0.0, 1.0);

        self.shared.hsv.set(((fraction * 360.0).min(359.99), s, v));
        self.shared.commit();
    }
}

impl Widget for HueSliderWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
        let stops = [
            0xff0000u32,
            0xffff00,
            0x00ff00,
            0x00ffff,
            0x0000ff,
            0xff00ff,
            0xff0000,
        ];
        let colors: Vec<Color4f> = stops.iter().map(|&c| Color4f::from(rgb_color(c))).collect();
        let gradient = Gradient::new(
            Colors::new(&colors, None, TileMode::Clamp, None),
            Interpolation::default(),
        );
        if let Some(shader) = linear_gradient(
            (
                Point::new(rect.x, rect.y),
                Point::new(rect.x + rect.width, rect.y),
            ),
            &gradient,
            None,
        ) {
            let mut paint = Paint::default();
            paint.set_shader(shader);
            paint.set_anti_alias(true);
            fill_rounded(canvas, bounds, style, &paint);
        }

        let (h, _, _) = self.shared.hsv.get();
        let center_x = rect.x + HUE_INSET + h / 360.0 * (rect.width - 2.0 * HUE_INSET) + 2.0;
        let knob = Rect::from_xywh(center_x - 4.0, rect.y - 4.0, 8.0, 16.0);
        let mut halo = Paint::default();
        halo.set_anti_alias(true);
        halo.set_color(Color::from_argb(153, 0, 0, 0));
        canvas.draw_rrect(
            RRect::new_rect_xy(knob.with_outset((1.0, 1.0)), 4.0, 4.0),
            &halo,
        );
        let mut fill = Paint::default();
        fill.set_anti_alias(true);
        fill.set_color(Color::WHITE);
        canvas.draw_rrect(RRect::new_rect_xy(knob, 3.0, 3.0), &fill);
    }

    fn handle_event(&mut self, event: &UiEvent, rect: AbsoluteRect) -> EventResult {
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

pub struct ColorPickerProps {
    /// Packed `0xRRGGBB`.
    pub rgb: u32,

    /// Swatches under the field, each packed `0xRRGGBB`.
    pub suggested: Vec<u32>,

    pub on_change: Rc<dyn Fn(u32)>,

    pub fonts: Option<TextFonts>,

    /// Field and hex input only. No swatches, no outer chrome.
    pub plain: bool,

    pub autofocus: bool,
}

impl Default for ColorPickerProps {
    fn default() -> Self {
        Self {
            rgb: 0,
            suggested: Vec::new(),
            on_change: Rc::new(|_| {}),
            fonts: None,
            plain: false,
            autofocus: false,
        }
    }
}

crate::builder_fields!(ColorPickerProps {
    rgb: u32,
    suggested: Vec<u32>,
    on_change: Rc<dyn Fn(u32)>,
    opt fonts: TextFonts,
    plain: bool,
    autofocus: bool,
});

impl ColorPickerProps {
    pub fn build(self) -> Node {
        color_picker(self)
    }
}

pub fn color_picker(props: ColorPickerProps) -> Node {
    let fonts = font_system::resolve(props.fonts);
    let shared = Rc::new(Shared {
        hsv: Cell::new(rgb_to_hsv(props.rgb)),
        hex: signal(format!("{:06x}", props.rgb & 0xffffff)),
        on_change: props.on_change,
    });

    let field = Node::new(
        "div",
        SaturationFieldWidget {
            shared: shared.clone(),
            dragging: false,
        },
    )
    .class(cls::FIELD);
    let hue = Node::new(
        "div",
        HueSliderWidget {
            shared: shared.clone(),
            dragging: false,
        },
    )
    .class(cls::HUE);

    let hex_row = Node::reactive({
        let (shared, fonts, plain, autofocus) =
            (shared.clone(), fonts.clone(), props.plain, props.autofocus);
        move || {
            let leading = text(TextProps {
                content: "#".to_string(),
                size: 16.0,
                color: INPUT_PLACEHOLDER,
                fonts: Some(fonts.clone()),
                ..Default::default()
            })
            .class(cls::HASH);
            let typed = shared.clone();
            let mut input = TextInputProps::default()
                .text(shared.hex.get())
                .placeholder("747c8c".to_string())
                .font_size(16.0_f32)
                .color(Color::from_rgb(0xef, 0xef, 0xf1))
                .placeholder_color(INPUT_PLACEHOLDER)
                .fonts(fonts.clone())
                .max_length(6_usize)
                .leading(leading)
                .autofocus(autofocus)
                .on_change(move |value: &str| {
                    if let Some(rgb) =
                        parse_hex_color(value).filter(|_| value.trim_start_matches('#').len() == 6)
                    {
                        typed.set_rgb(rgb, false);
                    }
                });
            if !plain {
                input = input.trailing(input_accessory(icons::EYE_DROPPER, || {}));
            }
            input.build()
        }
    });

    let mut suggested = div().class(cls::SUGGESTED);
    for rgb in props.suggested.iter().take(5).copied() {
        let shared = shared.clone();
        suggested = suggested.child(
            Node::new(
                "button",
                ButtonWidget::new().with_on_click(move || shared.set_rgb(rgb, true)),
            )
            .class(cls::SUGGESTION)
            .style(format!("background-color: #{:06x}", rgb & 0xffffff)),
        );
    }

    let mut root = div().class(cls::ROOT).children([field, hue, hex_row]);
    if !props.plain {
        root = root.child(suggested);
    }
    root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parsing_accepts_the_usual_spellings() {
        assert_eq!(parse_hex_color("#5865f2"), Some(0x5865f2));
        assert_eq!(parse_hex_color("5865F2"), Some(0x5865f2));
        assert_eq!(parse_hex_color("#fa0"), Some(0xffaa00));
        assert_eq!(parse_hex_color("#12345"), None);
        assert_eq!(parse_hex_color("zzzzzz"), None);
        assert_eq!(format_hex_color(0x0a0b0c), "#0a0b0c");
    }

    #[test]
    fn hsv_round_trips_the_live_pickers_colors() {
        for rgb in [
            0x747c8c, 0x4eb25f, 0xb2914e, 0xb24ea1, 0x000000, 0xffffff, 0x5865f2,
        ] {
            let back = hsv_to_rgb(rgb_to_hsv(rgb));
            for shift in [16, 8, 0] {
                let (a, b) = (
                    ((rgb >> shift) & 0xff) as i32,
                    ((back >> shift) & 0xff) as i32,
                );
                assert!((a - b).abs() <= 1, "{rgb:06x} -> {back:06x}");
            }
        }

        assert_eq!(hsv_to_rgb((220.0, 1.0, 1.0)), 0x0055ff);
    }
}
