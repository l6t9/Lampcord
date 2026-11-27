use crate::decode_pool;
use crate::images as crate_images;
use requiem_ui::paint as requiem_ui_paint;
use skia_safe::codec::Result;
use skia_safe::image::CachingHint;
use skia_safe::images;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::primitives::EmojiFrame;
use requiem_ui::widget::{AbsoluteRect, Widget};
use skia_safe::codec::{Codec, NO_FRAME, Options};
use skia_safe::{AlphaType, Canvas, ColorType, Data, Image, ImageInfo, Paint, QuickReject, Rect};

use crate::config::BackendConfig;
use crate::images::ImageCache;

const MIN_FRAME_MS: u32 = 20;

const POLL: Duration = Duration::from_millis(100);

const MAX_POLLS: u32 = 200;

struct Frames {
    frames: Vec<(Image, u32)>,
    total_ms: u32,
}

impl Frames {
    fn bytes(&self) -> usize {
        self.frames
            .iter()
            .map(|(image, _)| image.width() as usize * image.height() as usize * 4)
            .sum()
    }
}

const FRAMES_BUDGET: usize = 64 * 1024 * 1024;

const FRAME_EDGE: i32 = 64;

pub const PICKER_FRAME_EDGE: i32 = 48;

const DECODING_RECHECK_MS: u32 = 100;

struct RawFrames {
    frames: Vec<RawFrame>,
}

struct RawFrame {
    pixels: Vec<u8>,
    width: i32,
    height: i32,
    duration: u32,
}

type Key = (u64, i32);

enum Decoded {
    Frames(Rc<Frames>),

    Still,

    Working,

    Next(u8),
}

thread_local! {

    static DECODED: RefCell<HashMap<Key, (Decoded, u64)>> = RefCell::new(HashMap::new());
    static TICK: Cell<u64> = const { Cell::new(0) };
    static CLOCK: Instant = Instant::now();
}

static FINISHED: Mutex<Vec<(Key, u8, Option<RawFrames>)>> = Mutex::new(Vec::new());

pub fn frame(
    images: &ImageCache,
    backend: &BackendConfig,
    id: u64,
    animated: bool,
) -> Option<EmojiFrame> {
    frame_sized(images, backend, id, animated, FRAME_EDGE)
}

pub fn frame_sized(
    images: &ImageCache,
    backend: &BackendConfig,
    id: u64,
    animated: bool,
    edge: i32,
) -> Option<EmojiFrame> {
    enum Next {
        Play(Rc<Frames>),
        Still,
        Wait,
        Fetch(u8),
    }

    let still = || {
        images
            .get_or_decode_image(&backend.emoji_picture_url(id))
            .map(|image| EmojiFrame {
                image,
                next_frame_ms: None,
            })
    };
    if !animated {
        return still();
    }
    absorb_finished();
    let key = (id, edge);
    let tick = TICK.with(|t| {
        t.set(t.get() + 1);
        t.get()
    });
    let next = DECODED.with(|d| match d.borrow_mut().get_mut(&key) {
        Some((decoded, used)) => {
            *used = tick;
            match decoded {
                Decoded::Frames(frames) => Next::Play(frames.clone()),
                Decoded::Still => Next::Still,
                Decoded::Working => Next::Wait,
                Decoded::Next(stage) => Next::Fetch(*stage),
            }
        }
        None => Next::Fetch(0),
    });
    match next {
        Next::Play(frames) => Some(at(&frames, CLOCK.with(|c| c.elapsed().as_millis() as u64))),
        Next::Still => still(),
        Next::Wait => still().map(|f| EmojiFrame {
            next_frame_ms: Some(DECODING_RECHECK_MS),
            ..f
        }),
        Next::Fetch(stage) => {
            let url = if stage == 0 {
                backend.animated_emoji_url(id)
            } else {
                backend.gif_emoji_url(id)
            };
            let Some(bytes) = images.get_or_request(&url) else {
                return still();
            };
            DECODED.with(|d| {
                d.borrow_mut().insert(key, (Decoded::Working, tick));
            });
            let (_runtime, _http, wake) = images.download_context();
            decode_pool::spawn(move || {
                let raw = decode_raw(&bytes, edge).filter(|raw| raw.frames.len() > 1);
                FINISHED
                    .lock()
                    .expect("finished decodes poisoned")
                    .push((key, stage, raw));
                wake();
            });
            still().map(|f| EmojiFrame {
                next_frame_ms: Some(DECODING_RECHECK_MS),
                ..f
            })
        }
    }
}

fn absorb_finished() {
    let finished: Vec<_> =
        mem::take(&mut *FINISHED.lock().expect("finished decodes poisoned"));
    if finished.is_empty() {
        return;
    }
    DECODED.with(|d| {
        let mut d = d.borrow_mut();
        let tick = TICK.with(Cell::get);
        for (key, stage, raw) in finished {
            let decoded = match raw.and_then(frames_from_raw) {
                Some(frames) => Decoded::Frames(Rc::new(frames)),
                None if stage == 0 => Decoded::Next(1),
                None => Decoded::Still,
            };
            d.insert(key, (decoded, tick));
            evict_over_budget(&mut d, key);
        }
    });
}

fn evict_over_budget(decoded: &mut HashMap<Key, (Decoded, u64)>, keep: Key) {
    let size = |d: &Decoded| {
        if let Decoded::Frames(f) = d {
            f.bytes()
        } else {
            0
        }
    };
    let mut total: usize = decoded.values().map(|(d, _)| size(d)).sum();
    while total > FRAMES_BUDGET {
        let Some((&oldest, _)) = decoded
            .iter()
            .filter(|(key, (d, _))| **key != keep && size(d) > 0)
            .min_by_key(|(_, (_, used))| *used)
        else {
            break;
        };
        if let Some((evicted, _)) = decoded.remove(&oldest) {
            total -= size(&evicted);
        }
    }
}

pub fn stats() -> (usize, usize, usize) {
    DECODED.with(|d| {
        d.borrow().values().fold((0, 0, 0), |(n, f, b), (d, _)| {
            if let Decoded::Frames(x) = d {
                (n + 1, f + x.frames.len(), b + x.bytes())
            } else {
                (n, f, b)
            }
        })
    })
}

fn frames_from_raw(raw: RawFrames) -> Option<Frames> {
    let mut frames = Vec::with_capacity(raw.frames.len());
    let mut total_ms = 0;
    for frame in raw.frames {
        let info = ImageInfo::new(
            (frame.width, frame.height),
            ColorType::N32,
            AlphaType::Premul,
            None,
        );
        let image = images::raster_from_data(
            &info,
            Data::new_copy(&frame.pixels),
            frame.width as usize * 4,
        )?;
        total_ms += frame.duration;
        frames.push((image, frame.duration));
    }
    Some(Frames { frames, total_ms })
}

fn at(frames: &Frames, elapsed_ms: u64) -> EmojiFrame {
    let mut t = (elapsed_ms % u64::from(frames.total_ms.max(1))) as u32;
    for (image, duration) in &frames.frames {
        if t < *duration {
            return EmojiFrame {
                image: image.clone(),
                next_frame_ms: Some(*duration - t),
            };
        }
        t -= duration;
    }
    let (image, _) = &frames.frames[0];
    EmojiFrame {
        image: image.clone(),
        next_frame_ms: Some(MIN_FRAME_MS),
    }
}

fn decode_raw(bytes: &[u8], edge: i32) -> Option<RawFrames> {
    let mut codec = Codec::from_data(Data::new_copy(bytes))?;
    let count = codec.get_frame_count();
    if count < 2 {
        return None;
    }
    let size = codec.info().dimensions();
    let info = ImageInfo::new(size, ColorType::N32, AlphaType::Premul, None);
    let row_bytes = size.width as usize * 4;
    let mut canvases: Vec<Vec<u8>> = Vec::with_capacity(count);
    let mut frames = Vec::with_capacity(count);
    for index in 0..count {
        let frame_info = codec.get_frame_info(index)?;

        let prior =
            (frame_info.required_frame != NO_FRAME).then_some(frame_info.required_frame as usize);
        let mut pixels = match prior {
            Some(prior) => canvases.get(prior)?.clone(),
            None => vec![0u8; row_bytes * size.height as usize],
        };
        let options = Options {
            frame_index: index,
            prior_frame: prior,
            ..Options::default()
        };
        if codec.get_pixels_with_options(&info, &mut pixels, row_bytes, Some(&options))
            != Result::Success
        {
            return None;
        }
        let image = crate_images::cap_edge(
            images::raster_from_data(&info, Data::new_copy(&pixels), row_bytes)?,
            edge,
        );
        let out_info = ImageInfo::new(image.dimensions(), ColorType::N32, AlphaType::Premul, None);
        let out_row_bytes = out_info.min_row_bytes();
        let mut out = vec![0u8; out_row_bytes * out_info.height() as usize];
        if !image.read_pixels(
            &out_info,
            &mut out,
            out_row_bytes,
            (0, 0),
            CachingHint::Allow,
        ) {
            return None;
        }
        frames.push(RawFrame {
            pixels: out,
            width: out_info.width(),
            height: out_info.height(),
            duration: (frame_info.duration.max(0) as u32).max(MIN_FRAME_MS),
        });
        canvases.push(pixels);
    }
    Some(RawFrames { frames })
}

pub struct EmojiImageWidget {
    id: u64,
    animated: bool,

    edge: i32,
    images: ImageCache,
    backend: BackendConfig,
    polls: Cell<u32>,
}

impl EmojiImageWidget {
    pub fn new(id: u64, animated: bool, images: &ImageCache, backend: &BackendConfig) -> Self {
        Self {
            id,
            animated,
            edge: FRAME_EDGE,
            images: images.clone(),
            backend: backend.clone(),
            polls: Cell::new(0),
        }
    }

    pub fn frame_edge(mut self, edge: i32) -> Self {
        self.edge = edge;
        self
    }
}

impl Widget for EmojiImageWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        let dst = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);

        if canvas.quick_reject(&dst) {
            return;
        }
        match frame_sized(
            &self.images,
            &self.backend,
            self.id,
            self.animated,
            self.edge,
        ) {
            Some(frame) => {
                let mut paint = Paint::default();
                paint.set_anti_alias(true);
                canvas.draw_image_rect_with_sampling_options(
                    &frame.image,
                    None,
                    dst,
                    skia_safe::SamplingOptions::new(
                        skia_safe::FilterMode::Linear,
                        skia_safe::MipmapMode::Linear,
                    ),
                    &paint,
                );
                if let Some(next) = frame.next_frame_ms {
                    requiem_ui_paint::animate_region_at(
                        canvas,
                        dst,
                        Instant::now() + Duration::from_millis(u64::from(next)),
                    );
                }
            }
            None if self.polls.get() < MAX_POLLS => {
                self.polls.set(self.polls.get() + 1);
                requiem_ui_paint::animate_region_at(canvas, dst, Instant::now() + POLL);
            }
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid() -> Image {
        let info = ImageInfo::new((2, 2), ColorType::N32, AlphaType::Premul, None);
        images::raster_from_data(&info, Data::new_copy(&[0u8; 16]), 8).unwrap()
    }

    #[test]
    fn the_due_frame_loops_through_the_animation() {
        let frames = Frames {
            frames: vec![(solid(), 100), (solid(), 50), (solid(), 150)],
            total_ms: 300,
        };
        assert_eq!(at(&frames, 0).next_frame_ms, Some(100));
        assert_eq!(at(&frames, 99).next_frame_ms, Some(1));
        assert_eq!(at(&frames, 100).next_frame_ms, Some(50));
        assert_eq!(at(&frames, 160).next_frame_ms, Some(140));
        assert_eq!(at(&frames, 300).next_frame_ms, Some(100), "loops");
        assert_eq!(at(&frames, 3_000_100).next_frame_ms, Some(50));
    }

    const TINY_GIF: [u8; 159] = [
        0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x04, 0x00, 0x04, 0x00, 0x81, 0x00, 0x00, 0xff, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0xff, 0x0b, 0x4e, 0x45,
        0x54, 0x53, 0x43, 0x41, 0x50, 0x45, 0x32, 0x2e, 0x30, 0x03, 0x01, 0x00, 0x00, 0x00, 0x21,
        0xf9, 0x04, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x04,
        0x00, 0x00, 0x08, 0x09, 0x00, 0x01, 0x08, 0x1c, 0x48, 0xb0, 0x20, 0x80, 0x80, 0x00, 0x21,
        0xf9, 0x04, 0x01, 0x0a, 0x00, 0x01, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x04,
        0x00, 0x81, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08,
        0x09, 0x00, 0x01, 0x08, 0x1c, 0x48, 0xb0, 0x20, 0x80, 0x80, 0x00, 0x21, 0xf9, 0x04, 0x01,
        0x0a, 0x00, 0x01, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x04, 0x00, 0x81, 0x00,
        0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x09, 0x00, 0x01,
        0x08, 0x1c, 0x48, 0xb0, 0x20, 0x80, 0x80, 0x00, 0x3b,
    ];

    #[test]
    fn an_animated_image_decodes_into_its_frames() {
        let frames = decode_raw(&TINY_GIF, FRAME_EDGE)
            .and_then(frames_from_raw)
            .expect("animated gif decodes");
        assert_eq!(frames.frames.len(), 3);
        assert_eq!(frames.total_ms, 300);
        assert_eq!(
            (frames.frames[0].0.width(), frames.frames[0].0.height()),
            (4, 4)
        );

        assert!(decode_raw(&TINY_GIF[..6], FRAME_EDGE).is_none());
    }
}
