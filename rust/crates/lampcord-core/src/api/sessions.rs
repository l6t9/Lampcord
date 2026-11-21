use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    rest::{Body, HttpMethod, RestConfig},
    rest_client::RestClient,
    traits::{CaptchaSolver, RestTransport},
};

use super::ApiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceIcon {
    Mobile,
    Desktop,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceSession {
    pub id_hash: String,
    pub name: String,
    pub location: Option<String>,
    pub last_used: Option<String>,
    pub current: bool,
    pub icon: DeviceIcon,
}

#[derive(Debug, Clone)]
pub struct SessionFailure {
    pub status: u16,
    pub body_text: String,
    pub body: Option<Value>,
}

impl std::fmt::Display for SessionFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(msg) = self.message() {
            write!(f, "{msg}")
        } else {
            write!(f, "HTTP {}", self.status)
        }
    }
}

impl std::error::Error for SessionFailure {}

impl SessionFailure {
    pub fn message(&self) -> Option<&str> {
        let obj = self.body.as_ref()?.as_object()?;
        for key in ["message", "error"] {
            if let Some(v) = obj.get(key).and_then(Value::as_str) {
                if !v.is_empty() && v != "null" {
                    return Some(v);
                }
            }
        }
        None
    }

    pub fn mfa(&self) -> Option<&Value> {
        let obj = self.body.as_ref()?.as_object()?;
        obj.get("mfa").filter(|v| {
            v.is_object() || v.is_array() || v.is_string() || v.is_boolean() || v.is_number()
        })
    }

    pub fn requires_password(&self) -> bool {
        self.body
            .as_ref()
            .and_then(Value::as_object)
            .is_some_and(|o| o.contains_key("password"))
            || self
                .message()
                .is_some_and(|m| m.to_ascii_lowercase().contains("password"))
    }

    pub fn requires_code(&self) -> bool {
        if self
            .body
            .as_ref()
            .and_then(Value::as_object)
            .is_some_and(|o| o.contains_key("code"))
        {
            return true;
        }
        let msg = self.message().unwrap_or_default().to_ascii_lowercase();
        msg.contains("two-factor") || msg.contains("2fa")
    }
}

#[derive(Debug)]
pub enum SessionsError {
    Api(ApiError),
    Failure(SessionFailure),
    Decode(serde_json::Error),
    InvalidResponse(String),
}

impl std::fmt::Display for SessionsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api(e) => write!(f, "{e}"),
            Self::Failure(e) => write!(f, "{e}"),
            Self::Decode(e) => write!(f, "decode error: {e}"),
            Self::InvalidResponse(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for SessionsError {}

impl From<ApiError> for SessionsError {
    fn from(value: ApiError) -> Self {
        Self::Api(value)
    }
}

impl From<serde_json::Error> for SessionsError {
    fn from(value: serde_json::Error) -> Self {
        Self::Decode(value)
    }
}

fn text_opt(v: Option<&Value>) -> Option<String> {
    match v? {
        Value::String(s) if !s.is_empty() && s != "null" => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn bool_opt(v: Option<&Value>) -> Option<bool> {
    match v? {
        Value::Bool(b) => Some(*b),
        Value::String(s) => s.parse::<bool>().ok(),
        _ => None,
    }
}

fn device_icon(name: &str) -> DeviceIcon {
    let mobile = [
        "android", "ios", "iphone", "ipad", "mobile", "pixel", "samsung",
    ]
    .iter()
    .any(|k| name.to_ascii_lowercase().contains(k));
    if mobile {
        DeviceIcon::Mobile
    } else if !name.is_empty() {
        DeviceIcon::Desktop
    } else {
        DeviceIcon::Unknown
    }
}

async fn request_json<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    method: HttpMethod,
    route: &str,
    body: Option<Value>,
    mfa_token: Option<&str>,
) -> Result<Value, SessionsError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let mut headers = config.standard_headers();
    headers.push(("Content-Type".to_owned(), "application/json".to_owned()));
    if let Some(token) = mfa_token {
        headers.push(("X-Discord-MFA-Authorization".to_owned(), token.to_owned()));
    }
    let body = match body {
        Some(v) => Body::Json(v.to_string()),
        None => Body::None,
    };
    let req = config.request(method, route, headers, body);
    let response = client
        .send(req)
        .await
        .map_err(ApiError::from)
        .map_err(SessionsError::from)?;
    if !(200..300).contains(&response.status) {
        let parsed = serde_json::from_str::<Value>(&response.body).ok();
        return Err(SessionsError::Failure(SessionFailure {
            status: response.status,
            body_text: response.body,
            body: parsed,
        }));
    }
    if response.body.trim().is_empty() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    Ok(serde_json::from_str(&response.body)?)
}

pub async fn list_sessions<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
) -> Result<Vec<DeviceSession>, SessionsError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let response = request_json(
        client,
        config,
        HttpMethod::Get,
        "/auth/sessions",
        None,
        None,
    )
    .await?;
    let obj = response.as_object().ok_or_else(|| {
        SessionsError::InvalidResponse(
            "Discord returned an unrecognized Devices response".to_owned(),
        )
    })?;
    let items = obj
        .get("user_sessions")
        .or_else(|| obj.get("sessions"))
        .and_then(Value::as_array)
        .ok_or_else(|| {
            SessionsError::InvalidResponse(
                "Discord returned an unrecognized Devices response".to_owned(),
            )
        })?;

    let current_hash = text_opt(obj.get("current_session_id_hash"))
        .or_else(|| text_opt(obj.get("auth_session_id_hash")));

    let mut sessions = Vec::new();
    for element in items {
        let Some(item) = element.as_object() else {
            continue;
        };
        let hash = text_opt(item.get("id_hash"))
            .or_else(|| text_opt(item.get("session_id_hash")))
            .or_else(|| text_opt(item.get("id")));
        let Some(hash) = hash else {
            continue;
        };

        let info = item.get("client_info").and_then(Value::as_object);
        let os = info.and_then(|i| text_opt(i.get("os")));
        let platform = info.and_then(|i| text_opt(i.get("platform")));
        let browser = info.and_then(|i| text_opt(i.get("browser")));
        let device = info.and_then(|i| text_opt(i.get("device")));
        let mut parts: Vec<String> = [device, browser, os, platform]
            .into_iter()
            .flatten()
            .collect();
        parts.dedup();
        let name = if parts.is_empty() {
            "Unknown device".to_owned()
        } else {
            parts.into_iter().take(2).collect::<Vec<_>>().join(" · ")
        };

        let location = info
            .and_then(|i| text_opt(i.get("location")))
            .or_else(|| text_opt(item.get("location")));
        let last_used =
            text_opt(item.get("approx_last_used_time")).or_else(|| text_opt(item.get("last_used")));
        let current = bool_opt(item.get("is_current")).unwrap_or(false)
            || bool_opt(item.get("current")).unwrap_or(false)
            || current_hash.as_ref().is_some_and(|h| h == &hash);
        let icon = device_icon(&name);

        sessions.push(DeviceSession {
            id_hash: hash,
            name,
            location,
            last_used,
            current,
            icon,
        });
    }

    if !items.is_empty() && sessions.is_empty() {
        return Err(SessionsError::InvalidResponse(
            "Discord returned sessions in an unrecognized format".to_owned(),
        ));
    }
    Ok(sessions)
}

pub async fn logout_sessions<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    id_hashes: &[String],
    password: Option<&str>,
    mfa_token: Option<&str>,
    backup_code: Option<&str>,
) -> Result<(), SessionsError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let mut payload = serde_json::json!({
        "session_id_hashes": id_hashes,
    });
    if let Some(password) = password.filter(|s| !s.is_empty()) {
        payload["password"] = Value::String(password.to_owned());
    }
    if let Some(code) = backup_code.filter(|s| !s.is_empty()) {
        payload["code"] = Value::String(code.to_owned());
    }
    request_json(
        client,
        config,
        HttpMethod::Post,
        "/auth/sessions/logout",
        Some(payload),
        mfa_token,
    )
    .await?;
    Ok(())
}

pub async fn finish_mfa<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    ticket: &str,
    mfa_type: &str,
    code: &str,
) -> Result<String, SessionsError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let payload = serde_json::json!({
        "ticket": ticket,
        "mfa_type": mfa_type,
        "data": code,
    });
    let response = request_json(
        client,
        config,
        HttpMethod::Post,
        "/mfa/finish",
        Some(payload),
        None,
    )
    .await?;
    text_opt(response.get("token")).ok_or_else(|| {
        SessionsError::InvalidResponse(
            "Discord did not return an MFA authorization token".to_owned(),
        )
    })
}
