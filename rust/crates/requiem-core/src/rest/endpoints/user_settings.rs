use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use crate::utils::time;
use serde::Deserialize;
use serde_json::json;
use std::iter;
use std::str;

use super::super::{HttpClient, RestError};
use super::protobuf::{put_bytes, put_fixed64, read_varint};
use crate::presence::{CustomStatus, OwnStatus, StatusEmoji};

const PRELOADED_SETTINGS_TYPE: u8 = 1;
const FIELD_STATUS_SETTINGS: u64 = 11;
const FIELD_STATUS: u64 = 1;
const FIELD_CUSTOM_STATUS: u64 = 2;

const FIELD_STATUS_EXPIRES_AT: u64 = 4;
const FIELD_GUILD_FOLDERS: u64 = 14;

/// One entry of the user's server list as stored in their settings: a folder, or (without
/// an `id`) a lone server that only carries its position.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FolderSetting {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub color: Option<u32>,
    pub guild_ids: Vec<u64>,
}

/// The server list as the settings blob stores it. `None` when the blob has no folder
/// settings at all, so an update that leaves them out doesn't clear what's shown.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GuildFolderSettings {
    pub folders: Vec<FolderSetting>,
    pub positions: Vec<u64>,
}

pub fn guild_folders_from_base64(blob: &str) -> Option<GuildFolderSettings> {
    let bytes = STANDARD.decode(blob.trim()).ok()?;
    guild_folders_from_settings(&bytes)
}

fn guild_folders_from_settings(settings: &[u8]) -> Option<GuildFolderSettings> {
    let message = fields(settings)
        .find(|f| f.number == FIELD_GUILD_FOLDERS)?
        .payload?;
    let mut out = GuildFolderSettings::default();
    for field in fields(message) {
        match field.number {
            1 => {
                if let Some(payload) = field.payload {
                    out.folders.push(folder_from_message(payload));
                }
            }
            2 => out.positions.extend(fixed64s(&field)),
            _ => {}
        }
    }
    Some(out)
}

/// The `value` of a protobuf wrapper message such as `Int64Value`.
fn wrapped<'a>(field: &Field<'a>) -> Option<Field<'a>> {
    field
        .payload
        .and_then(|p| fields(p).find(|f| f.number == 1))
}

fn folder_from_message(message: &[u8]) -> FolderSetting {
    let mut folder = FolderSetting::default();
    for field in fields(message) {
        match field.number {
            1 => folder.guild_ids.extend(fixed64s(&field)),
            2 => folder.id = wrapped(&field).and_then(|f| f.value).map(|v| v as i64),
            3 => {
                folder.name = wrapped(&field)
                    .and_then(|f| f.payload)
                    .and_then(|p| str::from_utf8(p).ok())
                    .map(str::to_string)
                    .filter(|name| !name.is_empty())
            }
            4 => folder.color = wrapped(&field).and_then(|f| f.value).map(|v| v as u32),
            _ => {}
        }
    }
    folder
}

/// A repeated `fixed64`, which arrives packed in one blob or as separate fields.
fn fixed64s(field: &Field<'_>) -> Vec<u64> {
    match field.payload {
        Some(packed) => packed
            .chunks_exact(8)
            .map(|c| u64::from_le_bytes(c.try_into().expect("chunk is 8 bytes")))
            .collect(),
        None => fixed64(field).into_iter().collect(),
    }
}

#[derive(Deserialize)]
struct SettingsResponse {
    settings: String,
}

pub fn status_from_base64(blob: &str) -> Option<OwnStatus> {
    let bytes = STANDARD.decode(blob.trim()).ok()?;
    status_from_settings(&bytes)
}

fn status_from_settings(settings: &[u8]) -> Option<OwnStatus> {
    let status_settings = fields(settings)
        .find(|f| f.number == FIELD_STATUS_SETTINGS)?
        .payload?;
    let string_value = fields(status_settings)
        .find(|f| f.number == FIELD_STATUS)?
        .payload?;
    let value = fields(string_value).find(|f| f.number == 1)?.payload?;
    OwnStatus::from_str(str::from_utf8(value).ok()?)
}

pub fn custom_status_from_base64(blob: &str) -> Option<CustomStatus> {
    let bytes = STANDARD.decode(blob.trim()).ok()?;
    custom_status_from_settings(&bytes)
}

fn custom_status_from_settings(settings: &[u8]) -> Option<CustomStatus> {
    let status_settings = fields(settings)
        .find(|f| f.number == FIELD_STATUS_SETTINGS)?
        .payload?;
    let custom = fields(status_settings)
        .find(|f| f.number == FIELD_CUSTOM_STATUS)?
        .payload?;
    let mut status = CustomStatus::default();
    let (mut emoji_id, mut emoji_name) = (0u64, String::new());
    for field in fields(custom) {
        match field.number {
            1 => {
                status.text = field
                    .payload
                    .and_then(|p| str::from_utf8(p).ok())
                    .unwrap_or_default()
                    .to_string()
            }
            2 => emoji_id = fixed64(&field).unwrap_or(0),
            3 => {
                emoji_name = field
                    .payload
                    .and_then(|p| str::from_utf8(p).ok())
                    .unwrap_or_default()
                    .to_string()
            }
            4 => status.expires_at_ms = fixed64(&field).filter(|&ms| ms != 0),
            _ => {}
        }
    }
    if !emoji_name.is_empty() {
        status.emoji = Some(StatusEmoji {
            id: (emoji_id != 0).then_some(emoji_id),
            name: emoji_name,
            animated: false,
        });
    }
    (!status.is_empty()).then_some(status)
}

async fn read_settings(http: &HttpClient, path: &str) -> Result<Vec<u8>, RestError> {
    let response = http
        .request(
            "GET /users/@me/settings-proto/{type}",
            wreq::Method::GET,
            path,
        )
        .await?;
    let current: SettingsResponse = response.json().await?;
    Ok(STANDARD.decode(current.settings.trim()).unwrap_or_default())
}

async fn write_settings(http: &HttpClient, path: &str, patch: Vec<u8>) -> Result<(), RestError> {
    let body = json!({ "settings": STANDARD.encode(patch) });
    http.request_with_json(
        "PATCH /users/@me/settings-proto/{type}",
        wreq::Method::PATCH,
        path,
        &body,
    )
    .await?;
    Ok(())
}

pub async fn set_status(http: &HttpClient, status: OwnStatus) -> Result<(), RestError> {
    let path = format!("/users/@me/settings-proto/{PRELOADED_SETTINGS_TYPE}");
    let current = read_settings(http, &path).await?;
    write_settings(http, &path, status_patch(&current, status)).await
}

pub async fn set_custom_status(
    http: &HttpClient,
    custom: Option<&CustomStatus>,
) -> Result<(), RestError> {
    let path = format!("/users/@me/settings-proto/{PRELOADED_SETTINGS_TYPE}");
    let current = read_settings(http, &path).await?;
    write_settings(
        http,
        &path,
        custom_status_patch(&current, custom, time::now_ms()),
    )
    .await
}

fn custom_status_patch(current: &[u8], custom: Option<&CustomStatus>, now_ms: u64) -> Vec<u8> {
    let existing = fields(current)
        .find(|f| f.number == FIELD_STATUS_SETTINGS)
        .and_then(|f| f.payload)
        .unwrap_or(&[]);
    let mut status_settings = Vec::new();
    for field in fields(existing).filter(|f| f.number != FIELD_CUSTOM_STATUS) {
        status_settings.extend_from_slice(field.raw);
    }
    if let Some(custom) = custom.filter(|c| !c.is_empty()) {
        let mut encoded = Vec::new();
        if !custom.text.is_empty() {
            put_bytes(&mut encoded, 1, custom.text.as_bytes());
        }
        if let Some(emoji) = &custom.emoji {
            if let Some(id) = emoji.id {
                put_fixed64(&mut encoded, 2, id);
            }
            put_bytes(&mut encoded, 3, emoji.name.as_bytes());
        }
        if let Some(expires) = custom.expires_at_ms {
            put_fixed64(&mut encoded, 4, expires);
        }
        put_fixed64(&mut encoded, 5, now_ms);
        put_bytes(&mut status_settings, FIELD_CUSTOM_STATUS, &encoded);
    }
    let mut out = Vec::new();
    put_bytes(&mut out, FIELD_STATUS_SETTINGS, &status_settings);
    out
}

fn status_patch(current: &[u8], status: OwnStatus) -> Vec<u8> {
    let existing = fields(current)
        .find(|f| f.number == FIELD_STATUS_SETTINGS)
        .and_then(|f| f.payload)
        .unwrap_or(&[]);

    let mut string_value = Vec::new();
    put_bytes(&mut string_value, 1, status.as_str().as_bytes());
    let mut status_settings = Vec::new();
    put_bytes(&mut status_settings, FIELD_STATUS, &string_value);
    for field in
        fields(existing).filter(|f| f.number != FIELD_STATUS && f.number != FIELD_STATUS_EXPIRES_AT)
    {
        status_settings.extend_from_slice(field.raw);
    }

    let mut out = Vec::new();
    put_bytes(&mut out, FIELD_STATUS_SETTINGS, &status_settings);
    out
}

struct Field<'a> {
    number: u64,

    payload: Option<&'a [u8]>,

    value: Option<u64>,

    raw: &'a [u8],
}

fn fields(buf: &[u8]) -> impl Iterator<Item = Field<'_>> {
    let mut pos = 0;
    iter::from_fn(move || {
        let start = pos;
        let key = read_varint(buf, &mut pos)?;
        let mut value = None;
        let payload = match key & 7 {
            0 => {
                value = Some(read_varint(buf, &mut pos)?);
                None
            }
            1 => {
                pos = pos.checked_add(8).filter(|&p| p <= buf.len())?;
                None
            }
            2 => {
                let len = usize::try_from(read_varint(buf, &mut pos)?).ok()?;
                let end = pos.checked_add(len).filter(|&e| e <= buf.len())?;
                let payload = &buf[pos..end];
                pos = end;
                Some(payload)
            }
            5 => {
                pos = pos.checked_add(4).filter(|&p| p <= buf.len())?;
                None
            }
            _ => return None,
        };
        Some(Field {
            number: key >> 3,
            payload,
            value,
            raw: &buf[start..pos],
        })
    })
}

fn fixed64(field: &Field<'_>) -> Option<u64> {
    let bytes: [u8; 8] = field
        .raw
        .get(field.raw.len().checked_sub(8)?..)?
        .try_into()
        .ok()?;
    Some(u64::from_le_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use crate::rest::endpoints::protobuf::put_key;
    use super::*;
    use crate::rest::endpoints::protobuf::put_varint;

    fn wrapper(value_field: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
        let mut out = Vec::new();
        value_field(&mut out);
        out
    }

    #[test]
    fn folders_decode_with_packed_and_unpacked_ids() {
        let mut folder = Vec::new();
        let packed: Vec<u8> = [11u64, 22].iter().flat_map(|v| v.to_le_bytes()).collect();
        put_bytes(&mut folder, 1, &packed);
        put_bytes(
            &mut folder,
            2,
            &wrapper(|o| {
                put_key(o, 1, 0);
                put_varint(o, 987_654_321_012);
            }),
        );
        put_bytes(&mut folder, 3, &wrapper(|o| put_bytes(o, 1, b"Games")));
        put_bytes(
            &mut folder,
            4,
            &wrapper(|o| {
                put_key(o, 1, 0);
                put_varint(o, 0x5865f2);
            }),
        );
        let mut lone = Vec::new();
        put_fixed64(&mut lone, 1, 33);

        let mut message = Vec::new();
        put_bytes(&mut message, 1, &folder);
        put_bytes(&mut message, 1, &lone);
        put_fixed64(&mut message, 2, 44);
        let mut settings = Vec::new();
        put_bytes(&mut settings, FIELD_GUILD_FOLDERS, &message);

        let decoded = guild_folders_from_base64(&STANDARD.encode(&settings)).expect("folders");
        assert_eq!(
            decoded,
            GuildFolderSettings {
                folders: vec![
                    FolderSetting {
                        id: Some(987_654_321_012),
                        name: Some("Games".into()),
                        color: Some(0x5865f2),
                        guild_ids: vec![11, 22],
                    },
                    FolderSetting {
                        guild_ids: vec![33],
                        ..Default::default()
                    },
                ],
                positions: vec![44],
            }
        );
        assert_eq!(
            guild_folders_from_base64(&STANDARD.encode(Vec::<u8>::new())),
            None,
            "settings without folders say nothing about them"
        );
    }

    #[test]
    fn patch_replaces_status_and_keeps_custom_status() {
        let mut sv = Vec::new();
        put_bytes(&mut sv, 1, b"dnd");
        let mut ss = Vec::new();
        put_bytes(&mut ss, 1, &sv);
        put_bytes(&mut ss, 2, b"cs");
        put_varint(&mut ss, (4 << 3) | 1);
        ss.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        let mut settings = Vec::new();
        put_bytes(&mut settings, 2, b"other-top-level");
        put_bytes(&mut settings, 11, &ss);

        assert_eq!(status_from_settings(&settings), Some(OwnStatus::Dnd));

        let patch = status_patch(&settings, OwnStatus::Idle);
        assert_eq!(status_from_settings(&patch), Some(OwnStatus::Idle));
        let inner = fields(&patch).next().unwrap().payload.unwrap();
        let numbers: Vec<u64> = fields(inner).map(|f| f.number).collect();
        assert_eq!(numbers, vec![1, 2]);
        assert_eq!(fields(inner).nth(1).unwrap().payload, Some(&b"cs"[..]));

        assert_eq!(fields(&patch).count(), 1);
    }

    #[test]
    fn custom_status_round_trips_and_keeps_status() {
        let mut sv = Vec::new();
        put_bytes(&mut sv, 1, b"idle");
        let mut ss = Vec::new();
        put_bytes(&mut ss, 1, &sv);
        let mut settings = Vec::new();
        put_bytes(&mut settings, 11, &ss);

        let custom = CustomStatus {
            text: "Working".into(),
            emoji: Some(StatusEmoji {
                id: None,
                name: "\u{1f349}".into(),
                animated: false,
            }),
            expires_at_ms: Some(1_800_000_000_000),
        };
        let patch = custom_status_patch(&settings, Some(&custom), 42);
        assert_eq!(custom_status_from_settings(&patch), Some(custom));
        assert_eq!(status_from_settings(&patch), Some(OwnStatus::Idle));

        let cleared = custom_status_patch(&patch, None, 42);
        assert_eq!(custom_status_from_settings(&cleared), None);
        assert_eq!(status_from_settings(&cleared), Some(OwnStatus::Idle));
    }

    #[test]
    fn missing_status_is_none() {
        assert_eq!(status_from_settings(&[]), None);
    }
}
