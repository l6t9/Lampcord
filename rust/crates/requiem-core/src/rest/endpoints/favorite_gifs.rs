use std::cmp::Reverse;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::Deserialize;
use serde_json::json;

use super::super::{HttpClient, RestError};
use super::protobuf::{Field, Reader, put_bytes, put_uint};

pub(super) const FRECENCY_SETTINGS_TYPE: u8 = 2;
const FIELD_FAVORITE_GIFS: u64 = 2;

pub const GIF_TYPE_IMAGE: u64 = 1;
pub const GIF_TYPE_VIDEO: u64 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FavoriteGif {
    pub url: String,
    pub format: u64,
    pub src: String,
    pub width: u32,
    pub height: u32,

    pub order: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FavoriteGifs {
    pub gifs: Vec<FavoriteGif>,
    pub hide_tooltip: bool,
}

impl FavoriteGifs {
    pub fn sorted(&self) -> Vec<FavoriteGif> {
        let mut gifs = self.gifs.clone();
        gifs.sort_by_key(|a| Reverse(a.order));
        gifs
    }

    pub fn contains(&self, url: &str) -> bool {
        self.gifs.iter().any(|g| g.url == url)
    }

    pub fn toggle(&mut self, mut gif: FavoriteGif) {
        if let Some(i) = self.gifs.iter().position(|g| g.url == gif.url) {
            self.gifs.remove(i);
        } else {
            gif.order = self.gifs.iter().map(|g| g.order).max().unwrap_or(0) + 1;
            self.gifs.push(gif);
        }
    }
}

#[derive(Deserialize)]
pub(super) struct SettingsResponse {
    pub(super) settings: String,
}

pub async fn get_favorite_gifs(http: &HttpClient) -> Result<FavoriteGifs, RestError> {
    let path = format!("/users/@me/settings-proto/{FRECENCY_SETTINGS_TYPE}");
    let response = http
        .request(
            "GET /users/@me/settings-proto/{type}",
            wreq::Method::GET,
            &path,
        )
        .await?;
    let parsed: SettingsResponse = response.json().await?;
    let bytes = STANDARD.decode(parsed.settings.trim()).unwrap_or_default();
    Ok(decode_frecency_favorites(&bytes))
}

pub async fn set_favorite_gifs(
    http: &HttpClient,
    favorites: &FavoriteGifs,
) -> Result<(), RestError> {
    let path = format!("/users/@me/settings-proto/{FRECENCY_SETTINGS_TYPE}");
    let body = json!({ "settings": STANDARD.encode(encode_frecency_favorites(favorites)) });
    http.request_with_json(
        "PATCH /users/@me/settings-proto/{type}",
        wreq::Method::PATCH,
        &path,
        &body,
    )
    .await?;
    Ok(())
}

pub fn encode_frecency_favorites(favorites: &FavoriteGifs) -> Vec<u8> {
    let mut inner = Vec::new();
    for gif in &favorites.gifs {
        let mut value = Vec::new();
        put_uint(&mut value, 1, gif.format);
        put_bytes(&mut value, 2, gif.src.as_bytes());
        put_uint(&mut value, 3, u64::from(gif.width));
        put_uint(&mut value, 4, u64::from(gif.height));
        put_uint(&mut value, 5, u64::from(gif.order));

        let mut entry = Vec::new();
        put_bytes(&mut entry, 1, gif.url.as_bytes());
        put_bytes(&mut entry, 2, &value);
        put_bytes(&mut inner, 1, &entry);
    }
    put_uint(&mut inner, 2, u64::from(favorites.hide_tooltip));

    let mut out = Vec::new();

    put_bytes(&mut out, FIELD_FAVORITE_GIFS, &inner);
    out
}

pub fn decode_frecency_favorites(buf: &[u8]) -> FavoriteGifs {
    let mut out = FavoriteGifs::default();
    let mut top = Reader::new(buf);
    while let Some((field, value)) = top.next() {
        let (FIELD_FAVORITE_GIFS, Field::Bytes(inner)) = (field, value) else {
            continue;
        };
        let mut favs = Reader::new(inner);
        while let Some((field, value)) = favs.next() {
            match (field, value) {
                (1, Field::Bytes(entry)) => {
                    if let Some(gif) = decode_entry(entry) {
                        out.gifs.push(gif);
                    }
                }
                (2, Field::Varint(v)) => out.hide_tooltip = v != 0,
                _ => {}
            }
        }
    }
    out
}

fn decode_entry(entry: &[u8]) -> Option<FavoriteGif> {
    let mut url = None;
    let mut gif = FavoriteGif {
        url: String::new(),
        format: 0,
        src: String::new(),
        width: 0,
        height: 0,
        order: 0,
    };
    let mut r = Reader::new(entry);
    while let Some((field, value)) = r.next() {
        match (field, value) {
            (1, Field::Bytes(b)) => url = Some(String::from_utf8_lossy(b).into_owned()),
            (2, Field::Bytes(value)) => {
                let mut v = Reader::new(value);
                while let Some((field, value)) = v.next() {
                    match (field, value) {
                        (1, Field::Varint(x)) => gif.format = x,
                        (2, Field::Bytes(b)) => gif.src = String::from_utf8_lossy(b).into_owned(),
                        (3, Field::Varint(x)) => gif.width = x as u32,
                        (4, Field::Varint(x)) => gif.height = x as u32,
                        (5, Field::Varint(x)) => gif.order = x as u32,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    gif.url = url?;
    Some(gif)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gif(url: &str, order: u32) -> FavoriteGif {
        FavoriteGif {
            url: url.into(),
            format: GIF_TYPE_IMAGE,
            src: format!("{url}.gif"),
            width: 220,
            height: 124,
            order,
        }
    }

    #[test]
    fn round_trip() {
        let favs = FavoriteGifs {
            gifs: vec![
                gif("https://klipy.com/gifs/a", 1),
                gif("https://klipy.com/gifs/b", 2),
            ],
            hide_tooltip: true,
        };
        assert_eq!(
            decode_frecency_favorites(&encode_frecency_favorites(&favs)),
            favs
        );
    }

    #[test]
    fn empty_patch_still_carries_field_2() {
        let bytes = encode_frecency_favorites(&FavoriteGifs::default());
        assert_eq!(bytes, vec![0x12, 0x00]);
    }

    #[test]
    fn skips_other_top_level_fields() {
        let mut buf = vec![0x0a, 0x02, 0x08, 0x01, 0x60, 0x05];
        let favs = FavoriteGifs {
            gifs: vec![gif("u", 3)],
            hide_tooltip: false,
        };
        buf.extend(encode_frecency_favorites(&favs));
        assert_eq!(decode_frecency_favorites(&buf), favs);
    }

    #[test]
    fn toggle_adds_newest_then_removes() {
        let mut favs = FavoriteGifs {
            gifs: vec![gif("a", 5)],
            hide_tooltip: false,
        };
        favs.toggle(gif("b", 0));
        assert_eq!(
            favs.sorted().first().map(|g| (g.url.as_str(), g.order)),
            Some(("b", 6))
        );
        favs.toggle(gif("a", 0));
        assert!(!favs.contains("a"));
    }

    #[test]
    fn malformed_input_does_not_panic() {
        assert_eq!(
            decode_frecency_favorites(&[0x12, 0x7f, 0x01]),
            FavoriteGifs::default()
        );
    }
}
