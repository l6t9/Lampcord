use ffmpeg_next::codec::context::Context;
use ffmpeg_next::format::Pixel;
use ffmpeg_next::format;
use ffmpeg_next::frame::Video;
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::Context as ScalingContext;
use ffmpeg_next::software::scaling::flag::Flags;
use skia_safe::images;
use std::path::Path;

use skia_safe::Image;

/// Longest edge of the decoded frame; the tray shows it at 200px, so this leaves room for HiDPI.
const MAX_EDGE: u32 = 400;

/// Decodes the first frame of a video file as an upscaled-free RGBA image.
pub fn first_frame(path: &Path) -> Result<Image, String> {

    ffmpeg_next::init().map_err(|e| e.to_string())?;
    let mut input = format::input(path).map_err(|e| format!("open: {e}"))?;
    let stream = input
        .streams()
        .best(Type::Video)
        .ok_or("no video stream")?;
    let index = stream.index();
    let context = Context::from_parameters(stream.parameters())
        .map_err(|e| e.to_string())?;
    let mut decoder = context
        .decoder()
        .video()
        .map_err(|e| format!("decoder: {e}"))?;

    let mut decoded = Video::empty();
    let mut got = false;
    for (packet_stream, packet) in input.packets() {
        if packet_stream.index() != index || decoder.send_packet(&packet).is_err() {
            continue;
        }
        if decoder.receive_frame(&mut decoded).is_ok() {
            got = true;
            break;
        }
    }
    if !got {
        decoder.send_eof().ok();
        got = decoder.receive_frame(&mut decoded).is_ok();
    }
    if !got {
        return Err("the video decoded to no frames".into());
    }

    let (src_w, src_h) = (decoded.width(), decoded.height());
    if src_w == 0 || src_h == 0 {
        return Err("empty frame".into());
    }
    let scale = (MAX_EDGE as f32 / src_w.max(src_h) as f32).min(1.0);
    let width = ((src_w as f32 * scale).round() as u32).max(1);
    let height = ((src_h as f32 * scale).round() as u32).max(1);

    let mut scaler = ScalingContext::get(
        decoded.format(),
        src_w,
        src_h,
        Pixel::RGBA,
        width,
        height,
        Flags::BILINEAR,
    )
    .map_err(|e| format!("scaler: {e}"))?;
    let mut rgba = Video::empty();
    scaler
        .run(&decoded, &mut rgba)
        .map_err(|e| format!("scale: {e}"))?;

    let (stride, row) = (rgba.stride(0), width as usize * 4);
    let data = rgba.data(0);
    let mut packed = Vec::with_capacity(row * height as usize);
    for y in 0..height as usize {
        packed.extend_from_slice(&data[y * stride..y * stride + row]);
    }

    let info = skia_safe::ImageInfo::new(
        (width as i32, height as i32),
        skia_safe::ColorType::RGBA8888,
        skia_safe::AlphaType::Unpremul,
        None,
    );
    images::raster_from_data(&info, skia_safe::Data::new_copy(&packed), row)
        .ok_or_else(|| "couldn't build an image from the frame".to_string())
}
