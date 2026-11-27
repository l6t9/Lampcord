use crate::state::Action;
#[cfg(feature = "voice")]
use crate::voice;
use serde_json::value as serde_json_value;
use std::cell::OnceCell;
use std::collections;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::de::DeserializeOwned;
use serde_json::Value;
use serde_json::value::RawValue;

#[derive(Clone, Debug)]
pub struct GatewayEvent {
    pub name: String,
    raw: Box<RawValue>,
    value: OnceCell<Value>,
}

impl GatewayEvent {
    pub fn from_raw(name: String, raw: Box<RawValue>) -> Self {
        Self {
            name,
            raw,
            value: OnceCell::new(),
        }
    }

    pub fn new(name: impl Into<String>, data: Value) -> Self {
        let raw = serde_json_value::to_raw_value(&data).expect("a Value always serializes");
        Self {
            name: name.into(),
            raw,
            value: OnceCell::from(data),
        }
    }

    pub fn json(&self) -> &str {
        self.raw.get()
    }

    pub fn parse<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_str(self.raw.get())
    }

    pub fn value(&self) -> &Value {
        self.value
            .get_or_init(|| serde_json::from_str(self.raw.get()).unwrap_or_default())
    }
}

static CAPTURE_ALL: AtomicBool = AtomicBool::new(false);

pub fn set_capture_all(on: bool) {
    CAPTURE_ALL.store(on, Ordering::Relaxed);
}

pub fn is_wanted(name: &str) -> bool {
    CAPTURE_ALL.load(Ordering::Relaxed)
        || Action::HANDLED_EVENTS.contains(&name)
        || voice_wants(name)
}

#[cfg(feature = "voice")]
fn voice_wants(name: &str) -> bool {
    voice::HANDLED_EVENTS.contains(&name)
}
#[cfg(not(feature = "voice"))]
fn voice_wants(_name: &str) -> bool {
    false
}

pub fn summarize(name: &str, raw: &str) -> String {
    let value: Value = serde_json::from_str(raw).unwrap_or(Value::Null);
    match name {
        "READY" => ready_summary(&value),
        "GUILD_CREATE" | "GUILD_UPDATE" => guild_summary(&value),
        "CHANNEL_CREATE" | "CHANNEL_UPDATE" | "CHANNEL_DELETE" | "THREAD_CREATE"
        | "THREAD_UPDATE" | "THREAD_DELETE" => channel_summary(&value),
        "VOICE_STATE_UPDATE" => format!(
            "user={} guild={} channel={} session={}",
            id_of(&value, "user_id"),
            id_of(&value, "guild_id"),
            id_of(&value, "channel_id"),
            present(&value, "session_id")
        ),
        "VOICE_SERVER_UPDATE" => format!(
            "guild={} channel={} endpoint={} token={}",
            id_of(&value, "guild_id"),
            id_of(&value, "channel_id"),
            value
                .get("endpoint")
                .and_then(Value::as_str)
                .unwrap_or("<none>"),
            present(&value, "token")
        ),
        "STREAM_CREATE" | "STREAM_SERVER_UPDATE" | "STREAM_CLOSE" | "STREAM_DELETE" => {
            format!(
                "key={} endpoint={} token={}",
                value
                    .get("stream_key")
                    .and_then(Value::as_str)
                    .unwrap_or("<none>"),
                value
                    .get("endpoint")
                    .and_then(Value::as_str)
                    .unwrap_or("<none>"),
                present(&value, "token")
            )
        }
        _ => format!("{}B", raw.len()),
    }
}

fn ready_summary(value: &Value) -> String {
    let guilds = value.get("guilds").and_then(Value::as_array);
    let (available, unavailable, channels, missing_guild) = guilds.map_or((0, 0, 0, 0), |guilds| {
        let mut available = 0;
        let mut unavailable = 0;
        let mut channels = 0;
        let mut missing_guild = 0;
        for guild in guilds {
            if guild.get("unavailable").and_then(Value::as_bool) == Some(true)
                || guild.get("properties").is_none()
            {
                unavailable += 1;
                continue;
            }
            available += 1;
            let list = guild.get("channels").and_then(Value::as_array);
            channels += list.map_or(0, Vec::len);
            missing_guild += list.map_or(0, |list| {
                list.iter()
                    .filter(|channel| id_of(channel, "guild_id") == "<none>")
                    .count()
            });
        }
        (available, unavailable, channels, missing_guild)
    });
    let private = value
        .get("private_channels")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    format!(
        "user={} guilds={available} unavailable={unavailable} guild_channels={channels} channels_without_guild_id={missing_guild} private_channels={private} session={}",
        id_of(value.get("user").unwrap_or(&Value::Null), "id"),
        present(value, "session_id")
    )
}

fn guild_summary(value: &Value) -> String {
    let channels = value.get("channels").and_then(Value::as_array);
    let missing = channels.map_or(0, |list| {
        list.iter()
            .filter(|channel| id_of(channel, "guild_id") == "<none>")
            .count()
    });
    let kinds = channels.map_or_else(String::new, |list| {
        let mut counts = collections::BTreeMap::<u64, usize>::new();
        for channel in list {
            if let Some(kind) = channel.get("type").and_then(Value::as_u64) {
                *counts.entry(kind).or_default() += 1;
            }
        }
        counts
            .into_iter()
            .map(|(kind, n)| format!("{kind}:{n}"))
            .collect::<Vec<_>>()
            .join(",")
    });
    format!(
        "guild={} unavailable={} properties={} channels={} threads={} without_guild_id={} kinds=[{kinds}] voice_states={}",
        id_of(value, "id"),
        value
            .get("unavailable")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        value.get("properties").is_some(),
        channels.map_or(0, Vec::len),
        value
            .get("threads")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        missing,
        value
            .get("voice_states")
            .and_then(Value::as_array)
            .map_or(0, Vec::len)
    )
}

fn channel_summary(value: &Value) -> String {
    format!(
        "id={} type={} guild={} parent={} name={}",
        id_of(value, "id"),
        value
            .get("type")
            .map(|kind| kind.to_string())
            .unwrap_or_else(|| "<none>".into()),
        id_of(value, "guild_id"),
        id_of(value, "parent_id"),
        value
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("<none>")
    )
}

fn id_of(value: &Value, key: &str) -> String {
    match value.get(key) {
        Some(Value::String(id)) if !id.is_empty() => id.clone(),
        Some(Value::Number(id)) => id.to_string(),
        Some(Value::Null) | None => "<none>".into(),
        Some(other) => other.to_string(),
    }
}

fn present(value: &Value, key: &str) -> &'static str {
    match value.get(key) {
        Some(Value::String(text)) if !text.is_empty() => "present",
        Some(Value::Null) | None => "absent",
        Some(_) => "present",
    }
}
