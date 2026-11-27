use ffmpeg_next::codec::context::Context;
use ffmpeg_next::decoder;
use ffmpeg_next::format::Pixel;
use ffmpeg_next::format;
use ffmpeg_next::frame::Video;
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::Context as ScalingContext;
use ffmpeg_next::software::scaling::flag::Flags;
use ffmpeg_next::software::scaling;
use std::path::Path;

pub struct LogoFrames {
    pub width: u32,
    pub height: u32,
    pub fps: f32,

    pub frames: Vec<Vec<u8>>,
}

pub fn decode_logo_video(path: &Path, size: u32) -> Result<LogoFrames, String> {

    ffmpeg_next::init().map_err(|e| e.to_string())?;
    let mut input = format::input(path).map_err(|e| format!("open: {e}"))?;
    let stream = input
        .streams()
        .best(Type::Video)
        .ok_or("no video stream")?;
    let index = stream.index();
    let fps = {
        let rate = stream.avg_frame_rate();
        if rate.denominator() > 0 && rate.numerator() > 0 {
            rate.numerator() as f32 / rate.denominator() as f32
        } else {
            60.0
        }
    };

    let codec = decoder::find_by_name("libvpx-vp9").ok_or(
        "this FFmpeg build has no libvpx-vp9 decoder (needed for the logo's alpha channel)",
    )?;
    let mut context = Context::new_with_codec(codec);
    context
        .set_parameters(stream.parameters())
        .map_err(|e| e.to_string())?;
    let mut decoder = context
        .decoder()
        .video()
        .map_err(|e| format!("decoder: {e}"))?;

    let mut scaler: Option<scaling::Context> = None;
    let mut frames = Vec::new();
    let mut drain =
        |decoder: &mut decoder::Video, frames: &mut Vec<Vec<u8>>| -> Result<(), String> {
            let mut decoded = Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                if scaler.is_none() {
                    scaler = Some(
                        ScalingContext::get(
                            decoded.format(),
                            decoded.width(),
                            decoded.height(),
                            Pixel::RGBA,
                            size,
                            size,
                            Flags::BILINEAR,
                        )
                        .map_err(|e| format!("scaler: {e}"))?,
                    );
                }
                let mut rgba = Video::empty();
                scaler
                    .as_mut()
                    .expect("just built")
                    .run(&decoded, &mut rgba)
                    .map_err(|e| format!("scale: {e}"))?;

                let (stride, row) = (rgba.stride(0), size as usize * 4);
                let data = rgba.data(0);
                let mut packed = Vec::with_capacity(row * size as usize);
                for y in 0..size as usize {
                    packed.extend_from_slice(&data[y * stride..y * stride + row]);
                }
                frames.push(packed);
            }
            Ok(())
        };

    for (stream, packet) in input.packets() {
        if stream.index() == index && decoder.send_packet(&packet).is_ok() {
            drain(&mut decoder, &mut frames)?;
        }
    }
    decoder.send_eof().ok();
    drain(&mut decoder, &mut frames)?;

    if frames.is_empty() {
        return Err("the video decoded to no frames".into());
    }
    Ok(LogoFrames {
        width: size,
        height: size,
        fps,
        frames,
    })
}

#[cfg(test)]
mod tests {
    use std::env;
    use super::*;

    #[test]
    fn decodes_64_frames_with_real_transparency() {
        let Ok(path) = env::var("LOGO_VIDEO") else {
            return;
        };
        let logo = decode_logo_video(Path::new(&path), 256).unwrap();
        assert_eq!((logo.frames.len(), logo.width, logo.height), (64, 256, 256));
        assert!((logo.fps - 60.0).abs() < 0.01);
        let alpha: Vec<u8> = logo.frames[20].iter().skip(3).step_by(4).copied().collect();
        let transparent = alpha.iter().filter(|a| **a == 0).count();

        assert!(
            transparent > alpha.len() / 2,
            "{transparent} of {}",
            alpha.len()
        );
        assert!(alpha.contains(&255));
    }
}
