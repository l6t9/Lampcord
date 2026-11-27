use std::path;
use std::borrow::Cow;

pub fn get_text() -> Option<String> {
    arboard::Clipboard::new().ok()?.get_text().ok()
}

pub fn set_text(text: impl Into<String>) -> bool {
    arboard::Clipboard::new()
        .and_then(|mut clipboard| clipboard.set_text(text.into()))
        .is_ok()
}

pub fn get_files() -> Vec<path::PathBuf> {
    arboard::Clipboard::new()
        .ok()
        .and_then(|mut c| c.get().file_list().ok())
        .unwrap_or_default()
}

pub fn get_image() -> Option<(usize, usize, Vec<u8>)> {
    let image = arboard::Clipboard::new().ok()?.get_image().ok()?;
    Some((image.width, image.height, image.bytes.into_owned()))
}

pub fn set_image(width: usize, height: usize, rgba: Vec<u8>) -> bool {
    arboard::Clipboard::new()
        .and_then(|mut clipboard| {
            clipboard.set_image(arboard::ImageData {
                width,
                height,
                bytes: Cow::Owned(rgba),
            })
        })
        .is_ok()
}
