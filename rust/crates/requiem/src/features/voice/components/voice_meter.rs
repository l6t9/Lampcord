use requiem_core::voice::capture::preview;
use requiem_ui::paint as requiem_ui_paint;
use skia_safe::images;
use std::cell::RefCell;

use requiem_ui::node::Node;
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::{AbsoluteRect, Widget};
use skia_safe::gradient::shaders::linear_gradient;
use skia_safe::gradient::{Colors, Gradient, Interpolation};
use skia_safe::{Canvas, Color, Color4f, Paint, Point, RRect, Rect, TileMode};

use requiem_core::voice::engine::{self, SILENCE_DB, meter};

use crate::shell::classes::voice_settings as cls;

struct MicTestMeter {
    active: bool,
}

impl Widget for MicTestMeter {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        engine::keepalive();
        let (level, _) = meter::read();
        let progress = if self.active {
            (level - SILENCE_DB).clamp(0.0, 100.0)
        } else {
            0.0
        };
        let width = 8.0 * (rect.width / 8.0).round();
        let height = rect.height;
        let filled = (8.0 * (width * progress / 100.0 / 8.0).round()).min(width);

        let mut frame = Paint::default();
        frame.set_anti_alias(true);
        frame.set_color(style.text_color.unwrap_or(Color::BLACK));
        canvas.draw_rect(Rect::from_xywh(rect.x, rect.y, width, height), &frame);

        let empty = style.background.unwrap_or(Color::DARK_GRAY);
        let start = style.border_left_color.unwrap_or(Color::YELLOW);
        let end = style.border_right_color.unwrap_or(Color::GREEN);
        let stops = [Color4f::from(start), Color4f::from(end)];
        let gradient = linear_gradient(
            (
                Point::new(rect.x, rect.y),
                Point::new(rect.x + width, rect.y),
            ),
            &Gradient::new(
                Colors::new(&stops, None, TileMode::Clamp, None),
                Interpolation::default(),
            ),
            None,
        );
        let mut pill = Paint::default();
        pill.set_anti_alias(true);
        let cells = (width / 8.0) as usize;
        for cell in 0..cells {
            let x = rect.x + cell as f32 * 8.0;
            let bounds = Rect::from_xywh(x + 2.0, rect.y + 2.0, 4.0, height - 4.0);
            if (cell as f32 * 8.0) < filled {
                pill.set_shader(gradient.clone());
                if gradient.is_none() {
                    pill.set_color(end);
                }
            } else {
                pill.set_shader(None);
                pill.set_color(empty);
            }
            canvas.draw_rrect(RRect::new_rect_xy(bounds, 2.0, 2.0), &pill);
        }
        requiem_ui_paint::animate_region(
            canvas,
            Rect::from_xywh(rect.x, rect.y, rect.width, rect.height),
        );
    }
}

pub fn mic_test_meter(active: bool) -> Node {
    Node::new("div", MicTestMeter { active }).class(cls::MIC_TEST_METER)
}

struct SpeakingBar {
    disabled: bool,
}

impl Widget for SpeakingBar {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        engine::keepalive();
        let (_, speaking) = meter::read();
        let color = if speaking && !self.disabled {
            style.text_color
        } else {
            style.background
        };
        let mut color = color.unwrap_or(Color::GRAY);
        if self.disabled {
            color = color.with_a((color.a() as f32 * 0.6) as u8);
        }
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(color);
        canvas.draw_rrect(
            RRect::new_rect_xy(
                Rect::from_xywh(rect.x, rect.y, rect.width, rect.height),
                4.0,
                4.0,
            ),
            &paint,
        );
        requiem_ui_paint::animate_region(
            canvas,
            Rect::from_xywh(rect.x, rect.y, rect.width, rect.height),
        );
    }
}

pub fn speaking_bar(disabled: bool) -> Node {
    Node::new("div", SpeakingBar { disabled }).class(cls::SPEAKING_BAR)
}

pub fn level_fraction() -> f32 {
    engine::keepalive();
    let (level, _) = meter::read();
    ((level - SILENCE_DB) / 100.0).clamp(0.0, 1.0)
}

struct CameraPreview {
    image: RefCell<Option<(u64, skia_safe::Image)>>,
}

impl Widget for CameraPreview {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        preview::keepalive();
        if let Some((id, frame)) = preview::latest() {
            let mut cached = self.image.borrow_mut();
            if cached.as_ref().map(|(seen, _)| *seen) != Some(id) {
                let info = skia_safe::ImageInfo::new(
                    (frame.width as i32, frame.height as i32),
                    skia_safe::ColorType::RGBA8888,
                    skia_safe::AlphaType::Opaque,
                    None,
                );
                let data = skia_safe::Data::new_copy(&frame.rgba);
                *cached =
                    images::raster_from_data(&info, data, frame.width as usize * 4)
                        .map(|image| (id, image));
            }
            if let Some((_, image)) = cached.as_ref() {
                let scale =
                    (rect.width / image.width() as f32).min(rect.height / image.height() as f32);
                let (w, h) = (image.width() as f32 * scale, image.height() as f32 * scale);
                let dst = Rect::from_xywh(
                    rect.x + (rect.width - w) / 2.0,
                    rect.y + (rect.height - h) / 2.0,
                    w,
                    h,
                );
                canvas.save();
                canvas.translate((dst.center_x(), 0.0));
                canvas.scale((-1.0, 1.0));
                canvas.translate((-dst.center_x(), 0.0));
                canvas.draw_image_rect(image, None, dst, &Paint::default());
                canvas.restore();
            }
        }
        requiem_ui_paint::animate_region(
            canvas,
            Rect::from_xywh(rect.x, rect.y, rect.width, rect.height),
        );
    }
}

pub fn camera_preview() -> Node {
    Node::new(
        "div",
        CameraPreview {
            image: RefCell::new(None),
        },
    )
    .class(cls::CAMERA_PREVIEW)
}
