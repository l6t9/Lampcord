use serde_json::Value;

use crate::{
    rest::{Body, HttpMethod, RestConfig},
    rest_client::RestClient,
    traits::{CaptchaSolver, RestTransport},
};

use super::{ApiError, send_empty, send_json};

pub async fn get_guild_channels<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
) -> Result<Vec<lampcord_model::channel::Channel>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/guilds/{guild_id}/channels"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_channel<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
) -> Result<lampcord_model::channel::Channel, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/channels/{channel_id}"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_private_channels<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
) -> Result<Vec<lampcord_model::channel::Channel>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        "/users/@me/channels",
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn create_dm<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    recipient_id: u64,
) -> Result<lampcord_model::channel::Channel, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "recipient_id": recipient_id.to_string() });
    let req = config.request(
        HttpMethod::Post,
        "/users/@me/channels",
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn trigger_typing<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Post,
        &format!("/channels/{channel_id}/typing"),
        config.standard_headers(),
        Body::Json("{}".to_owned()),
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

pub async fn delete_channel<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Delete,
        &format!("/channels/{channel_id}"),
        config.standard_headers(),
        Body::None,
    );
    let response = client.send(req).await?;
    if !(200..300).contains(&response.status) {
        return Err(ApiError::Http {
            status: response.status,
            body: response.body,
        });
    }
    if response.body.trim().is_empty() {
        Ok(Value::Null)
    } else {
        Ok(serde_json::from_str(&response.body)?)
    }
}

pub async fn update_channel<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    name: Option<&str>,
    topic: Option<&str>,
    nsfw: Option<bool>,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let mut body = serde_json::Map::new();
    if let Some(name) = name {
        body.insert("name".to_owned(), Value::String(name.to_owned()));
    }
    if let Some(topic) = topic {
        body.insert("topic".to_owned(), Value::String(topic.to_owned()));
    }
    if let Some(nsfw) = nsfw {
        body.insert("nsfw".to_owned(), Value::Bool(nsfw));
    }
    let req = config.request(
        HttpMethod::Patch,
        &format!("/channels/{channel_id}"),
        config.standard_headers(),
        Body::Json(Value::Object(body).to_string()),
    );
    send_json(client, req).await
}

pub async fn create_invite<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    max_age: u32,
    max_uses: u32,
    unique: bool,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({
        "max_age": max_age,
        "max_uses": max_uses,
        "unique": unique,
    });
    let req = config.request(
        HttpMethod::Post,
        &format!("/channels/{channel_id}/invites"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn ack_bulk<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    read_states: &[(u64, u64)], // (channel_id, message_id)
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    if read_states.is_empty() {
        return Ok(());
    }
    let states: Vec<Value> = read_states
        .iter()
        .map(|(channel_id, message_id)| {
            serde_json::json!({
                "channel_id": channel_id.to_string(),
                "message_id": message_id.to_string(),
            })
        })
        .collect();
    let body = serde_json::json!({ "read_states": states });
    let req = config.request(
        HttpMethod::Post,
        "/read-states/ack-bulk",
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

pub async fn get_active_threads<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/channels/{channel_id}/threads/active"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn search_threads<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    limit: u16,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let path = format!(
        "/channels/{channel_id}/threads/search?limit={limit}&sort_by=last_message_at&sort_order=desc"
    );
    let req = config.request(
        HttpMethod::Get,
        &path,
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_archived_public_threads<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    limit: u16,
    before: Option<&str>,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let mut path = format!("/channels/{channel_id}/threads/archived/public?limit={limit}");
    if let Some(before) = before {
        path.push_str("&before=");
        path.push_str(before);
    }
    let req = config.request(
        HttpMethod::Get,
        &path,
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn join_thread<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    thread_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Put,
        &format!("/channels/{thread_id}/thread-members/@me"),
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

pub async fn leave_thread<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    thread_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Delete,
        &format!("/channels/{thread_id}/thread-members/@me"),
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

pub async fn create_thread<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    name: &str,
    content: &str,
    applied_tags: &[String],
) -> Result<lampcord_model::channel::Channel, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({
        "name": name,
        "auto_archive_duration": 4320,
        "message": { "content": content },
        "applied_tags": applied_tags,
    });
    let req = config.request(
        HttpMethod::Post,
        &format!("/channels/{channel_id}/threads"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn create_thread_from_message<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    message_id: u64,
    name: &str,
) -> Result<lampcord_model::channel::Channel, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({
        "name": name,
        "auto_archive_duration": 4320,
    });
    let req = config.request(
        HttpMethod::Post,
        &format!("/channels/{channel_id}/messages/{message_id}/threads"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

async fn call_action<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    action: &str,
    all_recipients: bool,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = if all_recipients {
        serde_json::json!({ "recipients": Value::Null })
    } else {
        serde_json::json!({})
    };
    let req = config.request(
        HttpMethod::Post,
        &format!("/channels/{channel_id}/call/{action}"),
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

pub async fn ring_call<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    call_action(client, config, channel_id, "ring", true).await
}

pub async fn stop_ringing<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    all_recipients: bool,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    call_action(client, config, channel_id, "stop-ringing", all_recipients).await
}

pub async fn update_thread<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    thread_id: u64,
    name: Option<&str>,
    archived: Option<bool>,
    locked: Option<bool>,
    applied_tags: Option<&[String]>,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let mut body = serde_json::Map::new();
    if let Some(name) = name {
        body.insert("name".to_owned(), Value::String(name.to_owned()));
    }
    if let Some(archived) = archived {
        body.insert("archived".to_owned(), Value::Bool(archived));
    }
    if let Some(locked) = locked {
        body.insert("locked".to_owned(), Value::Bool(locked));
    }
    if let Some(tags) = applied_tags {
        body.insert(
            "applied_tags".to_owned(),
            Value::Array(tags.iter().cloned().map(Value::String).collect()),
        );
    }
    let req = config.request(
        HttpMethod::Patch,
        &format!("/channels/{thread_id}"),
        config.standard_headers(),
        Body::Json(Value::Object(body).to_string()),
    );
    send_json(client, req).await
}

pub async fn get_archived_private_threads<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    limit: u16,
    before: Option<&str>,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let mut path = format!("/channels/{channel_id}/threads/archived/private?limit={limit}");
    if let Some(before) = before {
        path.push_str("&before=");
        path.push_str(before);
    }
    let req = config.request(
        HttpMethod::Get,
        &path,
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn create_group_dm<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    recipient_ids: &[u64],
) -> Result<lampcord_model::channel::Channel, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let recipients: Vec<String> = recipient_ids.iter().map(|id| id.to_string()).collect();
    let body = serde_json::json!({ "recipients": recipients });
    let req = config.request(
        HttpMethod::Post,
        "/users/@me/channels",
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn add_group_dm_recipient<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    user_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Put,
        &format!("/channels/{channel_id}/recipients/{user_id}"),
        config.standard_headers(),
        Body::Json("{}".to_owned()),
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

pub async fn remove_group_dm_recipient<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    user_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Delete,
        &format!("/channels/{channel_id}/recipients/{user_id}"),
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

pub async fn get_thread_members<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    thread_id: u64,
    with_member: bool,
) -> Result<Vec<Value>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let path = if with_member {
        format!("/channels/{thread_id}/thread-members?with_member=true")
    } else {
        format!("/channels/{thread_id}/thread-members")
    };
    let req = config.request(
        HttpMethod::Get,
        &path,
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_channel_invites<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/channels/{channel_id}/invites"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn create_guild_channel<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    name: &str,
    kind: u8,
    parent_id: Option<u64>,
) -> Result<lampcord_model::channel::Channel, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let mut payload = serde_json::json!({
        "name": name,
        "type": kind,
    });
    if let Some(parent_id) = parent_id {
        payload["parent_id"] = Value::String(parent_id.to_string());
    }
    let req = config.request(
        HttpMethod::Post,
        &format!("/guilds/{guild_id}/channels"),
        config.standard_headers(),
        Body::Json(payload.to_string()),
    );
    send_json(client, req).await
}

pub async fn create_webhook<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    name: &str,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "name": name });
    let req = config.request(
        HttpMethod::Post,
        &format!("/channels/{channel_id}/webhooks"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn update_webhook<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    webhook_id: u64,
    body: &Value,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Patch,
        &format!("/webhooks/{webhook_id}"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn delete_webhook<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    webhook_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Delete,
        &format!("/webhooks/{webhook_id}"),
        config.standard_headers(),
        Body::None,
    );
    send_empty(client, req).await
}

pub async fn set_permission_overwrite<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    overwrite_id: u64,
    body: &Value,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Put,
        &format!("/channels/{channel_id}/permissions/{overwrite_id}"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_empty(client, req).await
}

pub async fn create_stage_instance<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    body: &Value,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Post,
        &format!("/stage-instances"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn update_stage_instance<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    body: &Value,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Patch,
        &format!("/stage-instances/{channel_id}"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn delete_stage_instance<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Delete,
        &format!("/stage-instances/{channel_id}"),
        config.standard_headers(),
        Body::None,
    );
    send_empty(client, req).await
}

pub async fn send_soundboard_sound<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    body: &Value,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Post,
        &format!("/channels/{channel_id}/send-soundboard-sound"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_empty(client, req).await
}
