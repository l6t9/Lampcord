use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    rest::{Body, HttpMethod, RestConfig},
    rest_client::RestClient,
    traits::{CaptchaSolver, RestTransport},
};

use super::{ApiError, send_empty, send_json};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SettingsProtoResponse {
    #[serde(default)]
    pub settings: Option<String>,
}

pub async fn get_user_settings_proto<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    proto_type: u8,
) -> Result<Option<String>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/users/@me/settings-proto/{proto_type}"),
        config.standard_headers(),
        Body::None,
    );
    let response = client.send(req).await?;
    if !(200..300).contains(&response.status) {
        return Ok(None);
    }
    let parsed: SettingsProtoResponse = serde_json::from_str(&response.body)?;
    Ok(parsed.settings)
}

pub async fn get_relationships<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
) -> Result<Vec<Value>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        "/users/@me/relationships",
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_current_user<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
) -> Result<lampcord_model::user::User, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        "/users/@me",
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_user<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    user_id: u64,
) -> Result<lampcord_model::user::User, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/users/{user_id}"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_user_profile<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    user_id: u64,
    guild_id: Option<u64>,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let path = if let Some(guild_id) = guild_id {
        format!(
            "/users/{user_id}/profile?guild_id={guild_id}&with_mutual_guilds=true&with_mutual_friends=true&with_mutual_friends_count=true"
        )
    } else {
        format!(
            "/users/{user_id}/profile?with_mutual_guilds=true&with_mutual_friends=true&with_mutual_friends_count=true"
        )
    };
    let req = config.request(
        HttpMethod::Get,
        &path,
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn send_friend_request_by_username<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    username: &str,
    discriminator: Option<&str>,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let mut body = serde_json::json!({ "username": username });
    if let Some(discriminator) = discriminator {
        body["discriminator"] = Value::String(discriminator.to_owned());
    }
    let req = config.request(
        HttpMethod::Post,
        "/users/@me/relationships",
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn delete_relationship<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    user_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Delete,
        &format!("/users/@me/relationships/{user_id}"),
        config.standard_headers(),
        Body::None,
    );
    let response = client.send(req).await?;
    if (200..300).contains(&response.status) || response.status == 204 {
        Ok(())
    } else {
        Err(ApiError::Http {
            status: response.status,
            body: response.body,
        })
    }
}

/// Adds or accepts a relationship (PUT /users/@me/relationships/{user_id}).
/// `relationship_type` is optional (e.g. friend/block depending on context); for accepting incoming requests
/// Discord typically expects an empty JSON body or a type field depending on client behavior.
pub async fn put_relationship<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    user_id: u64,
    relationship_type: Option<u8>,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = match relationship_type {
        Some(t) => serde_json::json!({ "type": t }).to_string(),
        None => "{}".to_owned(),
    };
    let req = config.request(
        HttpMethod::Put,
        &format!("/users/@me/relationships/{user_id}"),
        config.standard_headers(),
        Body::Json(body),
    );
    let response = client.send(req).await?;
    if (200..300).contains(&response.status) || response.status == 204 {
        Ok(())
    } else {
        Err(ApiError::Http {
            status: response.status,
            body: response.body,
        })
    }
}

pub async fn get_mutual_friends<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    user_id: u64,
) -> Result<Vec<Value>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/users/{user_id}/relationships"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_user_settings<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        "/users/@me/settings",
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn update_user_settings<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    partial: &Value,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Patch,
        "/users/@me/settings",
        config.standard_headers(),
        Body::Json(partial.to_string()),
    );
    send_json(client, req).await
}

pub async fn update_status<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    status: &str,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "status": status });
    update_user_settings(client, config, &body).await
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FriendRequestResult {
    Sent,
    Failed(String),
}

pub async fn send_friend_request<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    username: &str,
    discriminator: Option<&str>,
    note: Option<&str>,
) -> Result<FriendRequestResult, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let mut body = serde_json::json!({ "username": username });
    body["discriminator"] = match discriminator {
        Some(d) => Value::String(d.to_owned()),
        None => Value::Null,
    };
    if let Some(note) = note {
        let trimmed: String = note.chars().take(120).collect();
        body["note"] = Value::String(trimmed);
    }
    let req = config.request(
        HttpMethod::Post,
        "/users/@me/relationships",
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    let response = client.send(req).await?;
    if (200..300).contains(&response.status) || response.status == 204 {
        return Ok(FriendRequestResult::Sent);
    }
    let discord = crate::auth::discord_error(&response.body);
    let discord_message = discord.and_then(|e| e.message).filter(|m| !m.is_empty());
    let msg = match response.status {
        429 => "Discord is rate limiting friend requests. Wait a moment and try again.".to_owned(),
        500..=599 => format!(
            "Discord is having trouble right now (HTTP {}). Try again in a moment.",
            response.status
        ),
        _ => discord_message.unwrap_or_else(|| "Couldn't find that user.".to_owned()),
    };
    Ok(FriendRequestResult::Failed(msg))
}

pub async fn update_user_settings_proto<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    base64_payload: &str,
    proto_type: u8,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "settings": base64_payload });
    let req = config.request(
        HttpMethod::Patch,
        &format!("/users/@me/settings-proto/{proto_type}"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    let response = client.send(req).await?;
    if (200..300).contains(&response.status) || response.status == 204 {
        Ok(())
    } else {
        Err(ApiError::Http {
            status: response.status,
            body: response.body,
        })
    }
}

pub async fn get_connections<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
) -> Result<Vec<Value>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        "/users/@me/connections",
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_devices<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
) -> Result<Vec<Value>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        "/users/@me/devices",
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn update_relationship<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    user_id: u64,
    nickname: Option<&str>,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let mut body = serde_json::Map::new();
    if let Some(nickname) = nickname {
        body.insert("nickname".to_owned(), Value::String(nickname.to_owned()));
    }
    let req = config.request(
        HttpMethod::Patch,
        &format!("/users/@me/relationships/{user_id}"),
        config.standard_headers(),
        Body::Json(Value::Object(body).to_string()),
    );
    let response = client.send(req).await?;
    if (200..300).contains(&response.status) || response.status == 204 {
        Ok(())
    } else {
        Err(ApiError::Http {
            status: response.status,
            body: response.body,
        })
    }
}

pub async fn update_custom_status<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    text: Option<&str>,
    emoji_name: Option<&str>,
    emoji_id: Option<&str>,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({
        "custom_status": {
            "text": text,
            "emoji_name": emoji_name,
            "emoji_id": emoji_id,
        }
    });
    update_user_settings(client, config, &body).await
}

pub async fn get_note<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    user_id: u64,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/users/@me/notes/{user_id}"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn set_note<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    user_id: u64,
    note: &str,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "note": note });
    let req = config.request(
        HttpMethod::Put,
        &format!("/users/@me/notes/{user_id}"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_empty(client, req).await
}

pub async fn update_user_guild_settings<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    body: &Value,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Patch,
        &format!("/users/@me/guilds/{guild_id}/settings"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}
