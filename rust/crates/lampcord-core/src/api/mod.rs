pub mod application;
pub mod auth;
pub mod channel;
pub mod guild;
pub mod media;
pub mod message;
pub mod oauth;
pub mod profile_board;
pub mod sessions;
pub mod user;

use serde::de::DeserializeOwned;

use crate::{
    rest::{Body, HttpMethod, RestConfig, RestRequest},
    rest_client::{RestClient, RestError},
    traits::{CaptchaSolver, RestTransport},
};

#[derive(Debug)]
pub enum ApiError {
    Rest(RestError),
    Http { status: u16, body: String },
    Decode(serde_json::Error),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rest(e) => write!(f, "rest error: {e}"),
            Self::Http { status, body } => write!(f, "http {status}: {body}"),
            Self::Decode(e) => write!(f, "decode error: {e}"),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<RestError> for ApiError {
    fn from(value: RestError) -> Self {
        Self::Rest(value)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(value: serde_json::Error) -> Self {
        Self::Decode(value)
    }
}

pub fn get_channel_messages_request(
    config: &RestConfig,
    channel_id: u64,
    limit: u8,
    before: Option<u64>,
) -> RestRequest {
    let mut path = format!("/channels/{channel_id}/messages?limit={limit}");
    if let Some(before) = before {
        path.push_str(&format!("&before={before}"));
    }
    config.request(
        HttpMethod::Get,
        &path,
        config.standard_headers(),
        Body::None,
    )
}

pub fn create_message_request(config: &RestConfig, channel_id: u64, content: &str) -> RestRequest {
    create_message_request_with_reply(config, channel_id, content, None, None)
}

pub fn create_message_request_with_reply(
    config: &RestConfig,
    channel_id: u64,
    content: &str,
    reply_to_message_id: Option<u64>,
    reply_to_guild_id: Option<u64>,
) -> RestRequest {
    let mut body = serde_json::json!({ "content": content });
    if let Some(mid) = reply_to_message_id {
        let mut reference = serde_json::json!({
            "message_id": mid.to_string(),
            "channel_id": channel_id.to_string(),
            "fail_if_not_exists": false,
        });
        if let Some(gid) = reply_to_guild_id {
            reference["guild_id"] = serde_json::Value::String(gid.to_string());
        }
        body["message_reference"] = reference;
    }
    config.request(
        HttpMethod::Post,
        &format!("/channels/{channel_id}/messages"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    )
}

pub async fn send_json<T, S, Resp>(
    client: &RestClient<T, S>,
    request: RestRequest,
) -> Result<Resp, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
    Resp: DeserializeOwned,
{
    let response = client.send(request).await?;
    if !(200..300).contains(&response.status) {
        return Err(ApiError::Http {
            status: response.status,
            body: response.body,
        });
    }
    Ok(serde_json::from_str::<Resp>(&response.body)?)
}

pub async fn fetch_channel_messages<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    limit: u8,
    before: Option<u64>,
) -> Result<Vec<lampcord_model::message::Message>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = get_channel_messages_request(config, channel_id, limit, before);
    send_json(client, req).await
}

pub async fn send_channel_message<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    content: &str,
) -> Result<lampcord_model::message::Message, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    send_channel_message_with_reply(client, config, channel_id, content, None, None).await
}

pub async fn send_channel_message_with_reply<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    content: &str,
    reply_to_message_id: Option<u64>,
    reply_to_guild_id: Option<u64>,
) -> Result<lampcord_model::message::Message, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = create_message_request_with_reply(
        config,
        channel_id,
        content,
        reply_to_message_id,
        reply_to_guild_id,
    );
    send_json(client, req).await
}

pub async fn edit_channel_message<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    message_id: u64,
    content: &str,
) -> Result<lampcord_model::message::Message, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    message::edit_message(client, config, channel_id, message_id, content).await
}

pub async fn ack_channel_message<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    message_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "token": null });
    let req = config.request(
        HttpMethod::Post,
        &format!("/channels/{channel_id}/messages/{message_id}/ack"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    let response = client.send(req).await?;
    if !(200..300).contains(&response.status) {
        // Best-effort: many clients tolerate ack failures; treat non-2xx as soft failure.
        return Err(ApiError::Http {
            status: response.status,
            body: response.body,
        });
    }
    Ok(())
}
