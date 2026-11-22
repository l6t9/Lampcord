use serde_json::Value;

use crate::{
    rest::{Body, HttpMethod, RestConfig},
    rest_client::RestClient,
    traits::{CaptchaSolver, RestTransport},
};

use super::{ApiError, send_json};

pub async fn edit_message<T, S>(
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
    let body = serde_json::json!({ "content": content });
    let req = config.request(
        HttpMethod::Patch,
        &format!("/channels/{channel_id}/messages/{message_id}"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn delete_message<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    message_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Delete,
        &format!("/channels/{channel_id}/messages/{message_id}"),
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

pub async fn add_reaction<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    message_id: u64,
    emoji: &str,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    // emoji should be URL-encoded by caller if needed (e.g. unicode or name:id)
    let req = config.request(
        HttpMethod::Put,
        &format!("/channels/{channel_id}/messages/{message_id}/reactions/{emoji}/@me"),
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

pub async fn remove_own_reaction<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    message_id: u64,
    emoji: &str,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Delete,
        &format!("/channels/{channel_id}/messages/{message_id}/reactions/{emoji}/@me"),
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

pub async fn pin_message<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    message_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Put,
        &format!("/channels/{channel_id}/pins/{message_id}"),
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

pub async fn unpin_message<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    message_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Delete,
        &format!("/channels/{channel_id}/pins/{message_id}"),
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

pub async fn get_pinned_messages<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
) -> Result<Vec<lampcord_model::message::Message>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/channels/{channel_id}/pins"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_message<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    message_id: u64,
) -> Result<lampcord_model::message::Message, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/channels/{channel_id}/messages/{message_id}"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_channel_messages_around<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    message_id: u64,
    limit: u8,
) -> Result<Vec<lampcord_model::message::Message>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let path = format!("/channels/{channel_id}/messages?around={message_id}&limit={limit}");
    let req = config.request(
        HttpMethod::Get,
        &path,
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_channel_messages_after<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    message_id: u64,
    limit: u8,
) -> Result<Vec<lampcord_model::message::Message>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let path = format!("/channels/{channel_id}/messages?after={message_id}&limit={limit}");
    let req = config.request(
        HttpMethod::Get,
        &path,
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_reaction_users<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    message_id: u64,
    emoji: &str,
) -> Result<Vec<lampcord_model::user::User>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/channels/{channel_id}/messages/{message_id}/reactions/{emoji}"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn vote_poll_answer<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    message_id: u64,
    answer_id: u32,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Put,
        &format!("/channels/{channel_id}/polls/{message_id}/answers/{answer_id}"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn unvote_poll_answer<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    message_id: u64,
    answer_id: u32,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Delete,
        &format!("/channels/{channel_id}/polls/{message_id}/answers/{answer_id}"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn expire_poll<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    message_id: u64,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Post,
        &format!("/channels/{channel_id}/polls/{message_id}/expire"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_mentions<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    limit: u8,
    before: Option<u64>,
) -> Result<Vec<lampcord_model::message::Message>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let mut path = format!("/users/@me/mentions?limit={limit}");
    if let Some(before) = before {
        path.push_str(&format!("&before={before}"));
    }
    let req = config.request(
        HttpMethod::Get,
        &path,
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn crosspost_message<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    message_id: u64,
) -> Result<lampcord_model::message::Message, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Post,
        &format!("/channels/{channel_id}/messages/{message_id}/crosspost"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn bulk_delete_messages<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    message_ids: &[u64],
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    if message_ids.is_empty() {
        return Ok(());
    }
    let ids: Vec<String> = message_ids.iter().map(|id| id.to_string()).collect();
    let body = serde_json::json!({ "messages": ids });
    let req = config.request(
        HttpMethod::Post,
        &format!("/channels/{channel_id}/messages/bulk-delete"),
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

fn push_query_param(params: &mut Vec<(String, String)>, key: &str, value: String) {
    params.push((key.to_owned(), value));
}

fn url_encode_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn build_search_query(
    content: Option<&str>,
    author_ids: &[String],
    mentions: &[String],
    has: &[String],
    min_id: Option<&str>,
    max_id: Option<&str>,
    sort_order: Option<&str>,
    author_types: &[String],
    offset: u32,
    channel_ids: &[String],
) -> String {
    let mut params: Vec<(String, String)> = Vec::new();
    push_query_param(&mut params, "limit", "25".to_owned());
    if offset > 0 {
        push_query_param(&mut params, "offset", offset.to_string());
    }
    if let Some(content) = content {
        push_query_param(&mut params, "content", content.to_owned());
    }
    for id in author_ids {
        push_query_param(&mut params, "author_id", id.clone());
    }
    for id in mentions {
        push_query_param(&mut params, "mentions", id.clone());
    }
    for h in has {
        push_query_param(&mut params, "has", h.clone());
    }
    for t in author_types {
        push_query_param(&mut params, "author_type", t.clone());
    }
    for id in channel_ids {
        push_query_param(&mut params, "channel_id", id.clone());
    }
    if let Some(v) = min_id {
        push_query_param(&mut params, "min_id", v.to_owned());
    }
    if let Some(v) = max_id {
        push_query_param(&mut params, "max_id", v.to_owned());
    }
    push_query_param(&mut params, "sort_by", "timestamp".to_owned());
    push_query_param(
        &mut params,
        "sort_order",
        sort_order.unwrap_or("desc").to_owned(),
    );
    params
        .into_iter()
        .map(|(k, v)| format!("{}={}", url_encode_component(&k), url_encode_component(&v)))
        .collect::<Vec<_>>()
        .join("&")
}

async fn request_search<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    path_and_query: String,
) -> Result<Option<Value>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    for attempt in 0..8 {
        let req = config.request(
            HttpMethod::Get,
            &path_and_query,
            config.standard_headers(),
            Body::None,
        );
        let response = client.send(req).await?;
        if response.status == 202 {
            if attempt == 7 {
                return Ok(None);
            }
            let retry_after_secs = serde_json::from_str::<Value>(&response.body)
                .ok()
                .and_then(|v| v.get("retry_after").cloned())
                .and_then(|v| {
                    v.as_f64()
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                })
                .unwrap_or(1.0);
            let ms = (retry_after_secs * 1000.0).clamp(500.0, 3000.0) as u64;
            tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
            continue;
        }
        if !(200..300).contains(&response.status) {
            return Ok(None);
        }
        return Ok(Some(serde_json::from_str(&response.body)?));
    }
    Ok(None)
}

#[allow(clippy::too_many_arguments)]
pub async fn search_channel_messages<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    channel_id: u64,
    content: Option<&str>,
    author_ids: &[String],
    mentions: &[String],
    has: &[String],
    min_id: Option<&str>,
    max_id: Option<&str>,
    sort_order: Option<&str>,
    author_types: &[String],
    offset: u32,
) -> Result<Option<Value>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let query = build_search_query(
        content,
        author_ids,
        mentions,
        has,
        min_id,
        max_id,
        sort_order,
        author_types,
        offset,
        &[],
    );
    request_search(
        client,
        config,
        format!("/channels/{channel_id}/messages/search?{query}"),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn search_guild_messages<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    content: Option<&str>,
    author_ids: &[String],
    mentions: &[String],
    has: &[String],
    min_id: Option<&str>,
    max_id: Option<&str>,
    sort_order: Option<&str>,
    author_types: &[String],
    offset: u32,
    channel_ids: &[String],
) -> Result<Option<Value>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let query = build_search_query(
        content,
        author_ids,
        mentions,
        has,
        min_id,
        max_id,
        sort_order,
        author_types,
        offset,
        channel_ids,
    );
    request_search(
        client,
        config,
        format!("/guilds/{guild_id}/messages/search?{query}"),
    )
    .await
}

/// Percent-encode an emoji for use in reaction URL path segments (e.g. unicode emoji or `name:id`).
pub fn encode_reaction_emoji(emoji: &str) -> String {
    url_encode_component(emoji)
}
