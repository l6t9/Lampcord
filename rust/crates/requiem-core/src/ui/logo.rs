#[cfg(feature = "video")]
use crate::logo_video;
use requiem_ui::paint as requiem_ui_paint;
use skia_safe::canvas::SrcRectConstraint;
#[cfg(feature = "video")]
use skia_safe::images;
use std::rc::Rc;
use std::time::{Duration, Instant};

use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::{AbsoluteRect, Widget};
#[cfg(feature = "video")]
use skia_safe::{AlphaType, ColorType, Data, ImageInfo};
use skia_safe::{Canvas, FilterMode, MipmapMode, Paint, Rect, SamplingOptions};

pub const ASSET: &str = "video/app_spinner";

pub struct Logo {
    atlas: skia_safe::Image,
    frame_width: i32,
    frame_height: i32,
    frame_count: usize,
    columns: usize,
    fps: f32,
    started: Instant,
}

impl Logo {
    #[cfg(feature = "video")]
    pub fn from_frames(frames: logo_video::LogoFrames) -> Option<Logo> {
        let (frame_width, frame_height) = (frames.width as usize, frames.height as usize);
        let frame_count = frames.frames.len();
        if frame_width == 0 || frame_height == 0 || frame_count == 0 {
            return None;
        }
        let columns = (frame_count as f64).sqrt().ceil() as usize;
        let rows = frame_count.div_ceil(columns);
        let (atlas_width, atlas_height) = (frame_width * columns, frame_height * rows);
        let frame_bytes = frame_width * frame_height * 4;
        let mut pixels = vec![0u8; atlas_width * atlas_height * 4];
        for (index, rgba) in frames.frames.iter().enumerate() {
            if rgba.len() != frame_bytes {
                return None;
            }
            let (column, row) = (index % columns, index / columns);
            for y in 0..frame_height {
                let src = y * frame_width * 4;
                let dst = ((row * frame_height + y) * atlas_width + column * frame_width) * 4;
                pixels[dst..dst + frame_width * 4]
                    .copy_from_slice(&rgba[src..src + frame_width * 4]);
            }
        }
        let info = ImageInfo::new(
            (atlas_width as i32, atlas_height as i32),
            ColorType::RGBA8888,
            AlphaType::Unpremul,
            None,
        );
        let atlas =
            images::raster_from_data(&info, Data::new_copy(&pixels), atlas_width * 4)?;
        Some(Logo {
            atlas,
            frame_width: frame_width as i32,
            frame_height: frame_height as i32,
            frame_count,
            columns,
            fps: frames.fps.max(1.0),
            started: Instant::now(),
        })
    }

    pub fn from_store(px: u32) -> Option<Logo> {
        #[cfg(feature = "video")]
        {
            let path = requiem_asset_store::get()?.cached_path(ASSET)?;
            Self::from_frames(logo_video::decode_logo_video(&path, px).ok()?)
        }
        #[cfg(not(feature = "video"))]
        {
            let _ = px;
            None
        }
    }
}

pub struct LogoWidget {
    pub logo: Option<Rc<Logo>>,
}

impl Widget for LogoWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        let Some(logo) = &self.logo else { return };
        let dst = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
        let elapsed_frames = logo.started.elapsed().as_secs_f64() * f64::from(logo.fps);
        let frame = elapsed_frames.floor() as usize % logo.frame_count;

        let next_frame = elapsed_frames.floor() + 1.0;
        let next_at = logo.started + Duration::from_secs_f64(next_frame / f64::from(logo.fps));
        requiem_ui_paint::animate_region_at(canvas, dst, next_at);

        let column = frame % logo.columns;
        let row = frame / logo.columns;
        let src = Rect::from_xywh(
            (column as i32 * logo.frame_width) as f32,
            (row as i32 * logo.frame_height) as f32,
            logo.frame_width as f32,
            logo.frame_height as f32,
        );
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        let constraint = Some((&src, SrcRectConstraint::Strict));
        canvas.draw_image_rect_with_sampling_options(
            &logo.atlas,
            constraint,
            dst,
            SamplingOptions::new(FilterMode::Linear, MipmapMode::None),
            &paint,
        );
    }
}
