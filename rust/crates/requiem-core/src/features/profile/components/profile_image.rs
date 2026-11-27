use base64::engine::general_purpose;
use skia_safe::canvas::SrcRectConstraint;
use std::fmt as std_fmt;
use std::fs;
use std::io;
use std::path;
use std::rc::Rc;

use base64::Engine;
use skia_safe::{
    AlphaType, ColorType, EncodedImageFormat, Image, ImageInfo, Paint, Rect, SamplingOptions,
    surfaces,
};

use crate::ui::{ImageTarget, PendingUpload};

const AVATAR_SIZE: i32 = 512;
const BANNER_SIZE: (i32, i32) = (1200, 480);

const GUILD_BANNER_SIZE: (i32, i32) = (960, 540);

#[derive(Debug)]
pub enum PrepareError {
    Read(io::Error),
    NotAnImage,
    Encode,
}

impl std_fmt::Display for PrepareError {
    fn fmt(&self, f: &mut std_fmt::Formatter<'_>) -> std_fmt::Result {
        match self {
            PrepareError::Read(err) => write!(f, "couldn't read the file: {err}"),
            PrepareError::NotAnImage => f.write_str("that file isn't an image this app can read"),
            PrepareError::Encode => f.write_str("couldn't encode the cropped image"),
        }
    }
}

fn is_gif(bytes: &[u8]) -> bool {
    bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a")
}

pub fn prepare_upload(
    path: &path::Path,
    target: ImageTarget,
    keep_animation: bool,
) -> Result<PendingUpload, PrepareError> {
    let bytes = fs::read(path).map_err(PrepareError::Read)?;
    prepare_bytes(&bytes, target, keep_animation)
}

pub fn prepare_bytes(
    bytes: &[u8],
    target: ImageTarget,
    keep_animation: bool,
) -> Result<PendingUpload, PrepareError> {
    let source =
        Image::from_encoded(skia_safe::Data::new_copy(bytes)).ok_or(PrepareError::NotAnImage)?;
    let b64 = |mime: &str, data: &[u8]| -> Rc<str> {
        format!(
            "data:{mime};base64,{}",
            general_purpose::STANDARD.encode(data)
        )
        .into()
    };

    if keep_animation && is_gif(bytes) {
        return Ok(PendingUpload {
            data_uri: b64("image/gif", bytes),
            preview: source,
        });
    }

    let size = match target {
        ImageTarget::Avatar => (AVATAR_SIZE, AVATAR_SIZE),
        ImageTarget::Banner => BANNER_SIZE,
    };
    crop_to(source, size, b64)
}

pub fn prepare_guild_image(
    path: &path::Path,
    banner: bool,
) -> Result<PendingUpload, PrepareError> {
    let bytes = fs::read(path).map_err(PrepareError::Read)?;
    let source =
        Image::from_encoded(skia_safe::Data::new_copy(&bytes)).ok_or(PrepareError::NotAnImage)?;
    let b64 = |mime: &str, data: &[u8]| -> Rc<str> {
        format!(
            "data:{mime};base64,{}",
            general_purpose::STANDARD.encode(data)
        )
        .into()
    };
    crop_to(
        source,
        if banner {
            GUILD_BANNER_SIZE
        } else {
            (AVATAR_SIZE, AVATAR_SIZE)
        },
        b64,
    )
}

fn crop_to(
    source: Image,
    (out_w, out_h): (i32, i32),
    b64: impl Fn(&str, &[u8]) -> Rc<str>,
) -> Result<PendingUpload, PrepareError> {
    let (sw, sh) = (source.width() as f32, source.height() as f32);

    let aspect = out_w as f32 / out_h as f32;
    let (cw, ch) = if sw / sh > aspect {
        (sh * aspect, sh)
    } else {
        (sw, sw / aspect)
    };
    let src = Rect::from_xywh((sw - cw) / 2.0, (sh - ch) / 2.0, cw, ch);

    let mut surface = surfaces::raster(
        &ImageInfo::new((out_w, out_h), ColorType::RGBA8888, AlphaType::Premul, None),
        None,
        None,
    )
    .ok_or(PrepareError::Encode)?;
    let paint = Paint::default();
    surface.canvas().draw_image_rect_with_sampling_options(
        &source,
        Some((&src, SrcRectConstraint::Strict)),
        Rect::from_wh(out_w as f32, out_h as f32),
        SamplingOptions::new(skia_safe::FilterMode::Linear, skia_safe::MipmapMode::Linear),
        &paint,
    );
    let cropped = surface.image_snapshot();
    let png = cropped
        .encode(None, EncodedImageFormat::PNG, 100)
        .ok_or(PrepareError::Encode)?;
    Ok(PendingUpload {
        data_uri: b64("image/png", png.as_bytes()),
        preview: cropped,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(w: i32, h: i32) -> Vec<u8> {
        let mut surface = surfaces::raster_n32_premul((w, h)).unwrap();
        surface.canvas().clear(skia_safe::Color::RED);
        surface
            .image_snapshot()
            .encode(None, EncodedImageFormat::PNG, 100)
            .unwrap()
            .as_bytes()
            .to_vec()
    }

    #[test]
    fn avatar_is_cropped_square_and_encoded_as_png() {
        let upload = prepare_bytes(&png(800, 300), ImageTarget::Avatar, false).unwrap();
        assert_eq!(
            (upload.preview.width(), upload.preview.height()),
            (AVATAR_SIZE, AVATAR_SIZE)
        );
        assert!(upload.data_uri.starts_with("data:image/png;base64,"));
    }

    #[test]
    fn banner_gets_the_banner_aspect() {
        let upload = prepare_bytes(&png(300, 800), ImageTarget::Banner, false).unwrap();
        assert_eq!(
            (upload.preview.width(), upload.preview.height()),
            BANNER_SIZE
        );
    }

    #[test]
    fn non_images_are_rejected() {
        assert!(matches!(
            prepare_bytes(b"not an image", ImageTarget::Avatar, false),
            Err(PrepareError::NotAnImage)
        ));
    }
}
