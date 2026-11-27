use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::json;

use super::favorite_gifs::{FRECENCY_SETTINGS_TYPE, SettingsResponse};
use super::protobuf::{Field, Reader, put_bytes, put_uint, put_varint};
use crate::frecency::{Frecency, Item};
use crate::rest::{HttpClient, RestError};

const FIELD_EMOJI_REACTION_FRECENCY: u64 = 13;
const FIELD_EMOJI_FRECENCY: u64 = 6;

const SYNCED_KEYS: usize = 100;

pub async fn get_reaction_frecency(http: &HttpClient) -> Result<Frecency, RestError> {
    get_frecency(http, FIELD_EMOJI_REACTION_FRECENCY).await
}

pub async fn get_used_frecency(http: &HttpClient) -> Result<Frecency, RestError> {
    get_frecency(http, FIELD_EMOJI_FRECENCY).await
}

async fn get_frecency(http: &HttpClient, field: u64) -> Result<Frecency, RestError> {
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
    Ok(decode_frecency(field, &bytes))
}

pub async fn set_reaction_frecency(
    http: &HttpClient,
    frecency: &Frecency,
    now_ms: u64,
) -> Result<(), RestError> {
    set_frecency(http, FIELD_EMOJI_REACTION_FRECENCY, frecency, now_ms).await
}

pub async fn set_used_frecency(
    http: &HttpClient,
    frecency: &Frecency,
    now_ms: u64,
) -> Result<(), RestError> {
    set_frecency(http, FIELD_EMOJI_FRECENCY, frecency, now_ms).await
}

async fn set_frecency(
    http: &HttpClient,
    field: u64,
    frecency: &Frecency,
    now_ms: u64,
) -> Result<(), RestError> {
    let path = format!("/users/@me/settings-proto/{FRECENCY_SETTINGS_TYPE}");
    let body = json!({ "settings": STANDARD.encode(encode_frecency(field, frecency, now_ms)) });
    http.request_with_json(
        "PATCH /users/@me/settings-proto/{type}",
        wreq::Method::PATCH,
        &path,
        &body,
    )
    .await?;
    Ok(())
}

pub fn encode_reaction_frecency(frecency: &Frecency, now_ms: u64) -> Vec<u8> {
    encode_frecency(FIELD_EMOJI_REACTION_FRECENCY, frecency, now_ms)
}

fn encode_frecency(field: u64, frecency: &Frecency, now_ms: u64) -> Vec<u8> {
    let mut inner = Vec::new();
    for (key, item) in frecency.top(now_ms, SYNCED_KEYS) {
        let mut value = Vec::new();
        put_uint(&mut value, 1, u64::from(item.total_uses));

        let mut packed = Vec::new();
        for &at in &item.recent_uses {
            put_varint(&mut packed, at);
        }
        if !packed.is_empty() {
            put_bytes(&mut value, 2, &packed);
        }
        put_uint(&mut value, 3, item.frecency(now_ms));
        put_uint(&mut value, 4, item.score(now_ms));

        let mut entry = Vec::new();
        put_bytes(&mut entry, 1, key.as_bytes());
        put_bytes(&mut entry, 2, &value);
        put_bytes(&mut inner, 1, &entry);
    }
    let mut out = Vec::new();
    put_bytes(&mut out, field, &inner);
    out
}

pub fn decode_reaction_frecency(buf: &[u8]) -> Frecency {
    decode_frecency(FIELD_EMOJI_REACTION_FRECENCY, buf)
}

fn decode_frecency(field_wanted: u64, buf: &[u8]) -> Frecency {
    let mut items = Vec::new();
    let mut top = Reader::new(buf);
    while let Some((field, value)) = top.next() {
        let (true, Field::Bytes(inner)) = (field == field_wanted, value) else {
            continue;
        };
        let mut map = Reader::new(inner);
        while let Some((field, value)) = map.next() {
            if let (1, Field::Bytes(entry)) = (field, value) {
                items.extend(decode_entry(entry));
            }
        }
    }
    Frecency::from_items(items)
}

fn decode_entry(entry: &[u8]) -> Option<(String, Item)> {
    let mut key = None;
    let mut item = Item {
        total_uses: 0,
        recent_uses: Vec::new(),
    };
    let mut r = Reader::new(entry);
    while let Some((field, value)) = r.next() {
        match (field, value) {
            (1, Field::Bytes(b)) => key = Some(String::from_utf8_lossy(b).into_owned()),
            (2, Field::Bytes(value)) => {
                let mut v = Reader::new(value);
                while let Some((field, value)) = v.next() {
                    match (field, value) {
                        (1, Field::Varint(x)) => item.total_uses = x as u32,

                        (2, Field::Bytes(packed)) => {
                            let mut p = Reader::new(packed);
                            while let Some(at) = p.varint() {
                                item.recent_uses.push(at);
                            }
                        }

                        (2, Field::Varint(at)) => item.recent_uses.push(at),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    item.recent_uses.sort_unstable();
    Some((key?, item))
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: u64 = 1_800_000_000_000;

    #[test]
    fn round_trips_through_the_proto() {
        let mut frecency = Frecency::default();
        frecency.track("thumbsup", NOW - 5);
        frecency.track("thumbsup", NOW);
        frecency.track("1234567890", NOW);
        let decoded = decode_reaction_frecency(&encode_reaction_frecency(&frecency, NOW));
        assert_eq!(decoded, frecency);
    }

    #[test]
    fn reads_unpacked_recent_uses_and_skips_other_fields() {
        let mut value = Vec::new();
        put_uint(&mut value, 1, 2);
        put_uint(&mut value, 2, 10);
        put_uint(&mut value, 2, 20);
        let mut entry = Vec::new();
        put_bytes(&mut entry, 1, b"eyes");
        put_bytes(&mut entry, 2, &value);
        let mut inner = Vec::new();
        put_bytes(&mut inner, 1, &entry);
        let mut buf = vec![0x28, 0x07];
        put_bytes(&mut buf, 13, &inner);
        let decoded = decode_reaction_frecency(&buf);
        assert_eq!(
            decoded.items(),
            [(
                "eyes".to_string(),
                Item {
                    total_uses: 2,
                    recent_uses: vec![10, 20]
                }
            )]
        );
    }

    #[test]
    fn the_two_histories_do_not_read_each_others_field() {
        let mut frecency = Frecency::default();
        frecency.track("eyes", NOW);
        let used = encode_frecency(FIELD_EMOJI_FRECENCY, &frecency, NOW);
        assert_eq!(decode_frecency(FIELD_EMOJI_FRECENCY, &used), frecency);
        assert!(decode_reaction_frecency(&used).is_empty());
    }

    #[test]
    fn malformed_input_does_not_panic() {
        assert!(decode_reaction_frecency(&[0x6a, 0x7f, 0x01]).is_empty());
    }
}
