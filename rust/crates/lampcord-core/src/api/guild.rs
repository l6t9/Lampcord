use serde_json::Value;

use crate::{
    rest::{Body, HttpMethod, RestConfig},
    rest_client::RestClient,
    traits::{CaptchaSolver, RestTransport},
};

use super::message::url_encode_component;
use super::{ApiError, send_empty, send_json};

pub async fn get_guild<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
) -> Result<lampcord_model::guild::Guild, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/guilds/{guild_id}"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_guild_preview<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/guilds/{guild_id}/preview"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn leave_guild<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Delete,
        &format!("/users/@me/guilds/{guild_id}"),
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

pub async fn get_guild_roles<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
) -> Result<Vec<Value>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/guilds/{guild_id}/roles"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_guild_member<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    user_id: u64,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/guilds/{guild_id}/members/{user_id}"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_guild_onboarding<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/guilds/{guild_id}/onboarding"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinFailure {
    CaptchaCancelled,
    CaptchaRejected,
    NoServerReturned,
    Unreachable,
    ServerLimit,
    ServerFull,
    JoinRestricted,
    InvalidInvite,
    Banned,
    VerificationLevel,
    NoPermission,
    NoAccess,
    RegionBlocked,
    VerifyAccount,
    RateLimited,
    DiscordUnavailable,
    Unauthorized,
    Forbidden,
    NotFound,
    Rejected,
    ServerMessage,
}

#[derive(Debug, Clone)]
pub enum JoinGuildResult {
    Joined(lampcord_model::guild::Guild),
    Failed {
        failure: JoinFailure,
        detail: Option<String>,
    },
}

pub async fn resolve_invite<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    code: &str,
) -> Result<Option<Value>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/invites/{code}?with_counts=true&with_expiration=true"),
        config.standard_headers(),
        Body::None,
    );
    let response = client.send(req).await?;
    if !(200..300).contains(&response.status) {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(&response.body)?))
}

pub async fn delete_invite<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    code: &str,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Delete,
        &format!("/invites/{code}"),
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

fn join_failure_for(
    code: Option<i32>,
    message: Option<&str>,
    status: u16,
) -> (JoinFailure, Option<String>) {
    if let Some(known) = match code {
        Some(30001) => Some(JoinFailure::ServerLimit),
        Some(30019) => Some(JoinFailure::ServerFull),
        Some(340015) => Some(JoinFailure::JoinRestricted),
        Some(10006 | 50020 | 50035) => Some(JoinFailure::InvalidInvite),
        Some(40007) => Some(JoinFailure::Banned),
        Some(50009) => Some(JoinFailure::VerificationLevel),
        Some(50013) => Some(JoinFailure::NoPermission),
        Some(50001) => Some(JoinFailure::NoAccess),
        Some(50095) => Some(JoinFailure::RegionBlocked),
        Some(40002) => Some(JoinFailure::VerifyAccount),
        _ => None,
    } {
        return (known, None);
    }
    if let Some(msg) = message.filter(|m| !m.is_empty()) {
        return (JoinFailure::ServerMessage, Some(msg.to_owned()));
    }
    match status {
        429 => (JoinFailure::RateLimited, None),
        500..=599 => (JoinFailure::DiscordUnavailable, Some(status.to_string())),
        401 => (JoinFailure::Unauthorized, None),
        403 => (JoinFailure::Forbidden, None),
        404 => (JoinFailure::NotFound, None),
        _ => (JoinFailure::Rejected, Some(status.to_string())),
    }
}

pub async fn join_guild<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    invite_code: &str,
    session_id: Option<&str>,
) -> Result<JoinGuildResult, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let mut headers = config.standard_headers();
    headers.push((
        "X-Context-Properties".to_owned(),
        "eyJsb2NhdGlvbiI6IkFjY2VwdCBJbnZpdGUgUGFnZSJ9".to_owned(),
    ));
    headers.push(("Content-Type".to_owned(), "application/json".to_owned()));

    let mut body = serde_json::Map::new();
    if let Some(session_id) = session_id {
        body.insert(
            "session_id".to_owned(),
            Value::String(session_id.to_owned()),
        );
    }
    let req = config.request(
        HttpMethod::Post,
        &format!("/invites/{invite_code}"),
        headers,
        Body::Json(Value::Object(body).to_string()),
    );

    let response = match client.send(req).await {
        Ok(r) => r,
        Err(e) => {
            return Ok(JoinGuildResult::Failed {
                failure: JoinFailure::Unreachable,
                detail: Some(e.to_string()),
            });
        }
    };

    if !(200..300).contains(&response.status) {
        if crate::captcha::is_captcha_response_body(&response.body) {
            return Ok(JoinGuildResult::Failed {
                failure: JoinFailure::CaptchaRejected,
                detail: None,
            });
        }
        let err = crate::auth::discord_error(&response.body);
        let (failure, detail) = join_failure_for(
            err.as_ref().and_then(|e| e.code),
            err.as_ref().and_then(|e| e.message.as_deref()),
            response.status,
        );
        return Ok(JoinGuildResult::Failed { failure, detail });
    }

    // Discord may return invite payload with embedded guild on success.
    let value: Value = serde_json::from_str(&response.body)?;
    let guild_value = value.get("guild").cloned().or_else(|| {
        if value.get("id").is_some() {
            Some(value.clone())
        } else {
            None
        }
    });
    if let Some(guild_value) = guild_value {
        if let Ok(guild) = serde_json::from_value::<lampcord_model::guild::Guild>(guild_value) {
            return Ok(JoinGuildResult::Joined(guild));
        }
    }
    Ok(JoinGuildResult::Failed {
        failure: JoinFailure::NoServerReturned,
        detail: None,
    })
}

/// Fetch the current user's guilds (includes id, name, icon, permissions, etc.).
/// Useful for populating guild metadata when gateway READY/GUILD_CREATE data is incomplete.
pub async fn get_user_guilds<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
) -> Result<Vec<lampcord_model::guild::Guild>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        "/users/@me/guilds",
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_guild_invites<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
) -> Result<Vec<Value>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/guilds/{guild_id}/invites"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_guild_members<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    limit: u16,
    after: Option<u64>,
) -> Result<Vec<Value>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let mut path = format!("/guilds/{guild_id}/members?limit={limit}");
    if let Some(after) = after {
        path.push_str(&format!("&after={after}"));
    }
    let req = config.request(
        HttpMethod::Get,
        &path,
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_guild_audit_log<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/guilds/{guild_id}/audit-logs"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_guild_bans<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
) -> Result<Vec<Value>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/guilds/{guild_id}/bans"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_guild_emojis<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
) -> Result<Vec<Value>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/guilds/{guild_id}/emojis"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn create_role<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "name": "new role" });
    let req = config.request(
        HttpMethod::Post,
        &format!("/guilds/{guild_id}/roles"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn update_role<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    role_id: u64,
    partial: &Value,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Patch,
        &format!("/guilds/{guild_id}/roles/{role_id}"),
        config.standard_headers(),
        Body::Json(partial.to_string()),
    );
    send_json(client, req).await
}

pub async fn delete_role<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    role_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Delete,
        &format!("/guilds/{guild_id}/roles/{role_id}"),
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

pub async fn ban_member<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    user_id: u64,
    delete_message_seconds: Option<u32>,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = match delete_message_seconds {
        Some(secs) => serde_json::json!({ "delete_message_seconds": secs }).to_string(),
        None => "{}".to_owned(),
    };
    let req = config.request(
        HttpMethod::Put,
        &format!("/guilds/{guild_id}/bans/{user_id}"),
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

pub async fn unban_member<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    user_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Delete,
        &format!("/guilds/{guild_id}/bans/{user_id}"),
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

pub async fn kick_member<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    user_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Delete,
        &format!("/guilds/{guild_id}/members/{user_id}"),
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

pub async fn create_emoji<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    name: &str,
    image_data_uri: &str,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "name": name, "image": image_data_uri });
    let req = config.request(
        HttpMethod::Post,
        &format!("/guilds/{guild_id}/emojis"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn update_emoji<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    emoji_id: u64,
    name: &str,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "name": name });
    let req = config.request(
        HttpMethod::Patch,
        &format!("/guilds/{guild_id}/emojis/{emoji_id}"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn delete_emoji<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    emoji_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Delete,
        &format!("/guilds/{guild_id}/emojis/{emoji_id}"),
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

/// Save guild onboarding responses with Discord's POST/PUT fallback behavior.
/// `option_ids` are selected option IDs; `prompts_in_onboarding_ids` are prompt IDs that should be marked seen on initial submit.
pub async fn save_onboarding_responses<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    option_ids: &[u64],
    prompts_in_onboarding_ids: &[u64],
    initial: bool,
) -> Result<bool, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    use std::time::{SystemTime, UNIX_EPOCH};

    let mut body = serde_json::Map::new();
    body.insert(
        "onboarding_responses".to_owned(),
        Value::Array(
            option_ids
                .iter()
                .map(|id| Value::String(id.to_string()))
                .collect(),
        ),
    );
    if initial {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let prompts_seen: serde_json::Map<String, Value> = prompts_in_onboarding_ids
            .iter()
            .map(|id| (id.to_string(), Value::Number(now.into())))
            .collect();
        let responses_seen: serde_json::Map<String, Value> = option_ids
            .iter()
            .map(|id| (id.to_string(), Value::Number(now.into())))
            .collect();
        body.insert(
            "onboarding_prompts_seen".to_owned(),
            Value::Object(prompts_seen),
        );
        body.insert(
            "onboarding_responses_seen".to_owned(),
            Value::Object(responses_seen),
        );
    }

    let body_json = Value::Object(body).to_string();
    let verbs: &[HttpMethod] = if initial {
        &[HttpMethod::Post, HttpMethod::Put]
    } else {
        &[HttpMethod::Put, HttpMethod::Post]
    };

    for method in verbs {
        let req = config.request(
            *method,
            &format!("/guilds/{guild_id}/onboarding-responses"),
            config.standard_headers(),
            Body::Json(body_json.clone()),
        );
        match client.send(req).await {
            Ok(response) => {
                if (200..300).contains(&response.status) || response.status == 204 {
                    return Ok(true);
                }
                if !(403..=409).contains(&response.status) {
                    return Ok(false);
                }
            }
            Err(_) => {
                // Try next verb on transport/request error (non-cancellation context not available here).
            }
        }
    }
    Ok(false)
}

pub async fn update_guild<T, S>(
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
        &format!("/guilds/{guild_id}"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn delete_guild<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Post,
        &format!("/guilds/{guild_id}/delete"),
        config.standard_headers(),
        Body::None,
    );
    send_empty(client, req).await
}

pub async fn get_guild_prune_count<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    days: u32,
    include_roles: &[u64],
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let mut query = format!("days={days}");
    for role_id in include_roles {
        query.push_str(&format!("&include_roles={role_id}"));
    }
    let req = config.request(
        HttpMethod::Get,
        &format!("/guilds/{guild_id}/prune?{query}"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn begin_guild_prune<T, S>(
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
        HttpMethod::Post,
        &format!("/guilds/{guild_id}/prune"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn set_guild_vanity_url<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    code: Option<&str>,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "code": code });
    let req = config.request(
        HttpMethod::Patch,
        &format!("/guilds/{guild_id}/vanity-url"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn search_guild_bans<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    limit: u16,
    user_ids: &[u64],
    query_text: Option<&str>,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let mut query = format!("limit={limit}");
    for user_id in user_ids {
        query.push_str(&format!("&user_ids={user_id}"));
    }
    if let Some(text) = query_text.filter(|text| !text.trim().is_empty()) {
        query.push_str(&format!("&query={}", url_encode_component(text)));
    }
    let req = config.request(
        HttpMethod::Get,
        &format!("/guilds/{guild_id}/bans/search?{query}"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn get_guild_templates<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/guilds/{guild_id}/templates"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn create_guild_template<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    name: &str,
    description: Option<&str>,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "name": name, "description": description });
    let req = config.request(
        HttpMethod::Post,
        &format!("/guilds/{guild_id}/templates"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn sync_guild_template<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    code: &str,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Put,
        &format!("/guilds/{guild_id}/templates/{code}"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn update_guild_template<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    code: &str,
    name: &str,
    description: Option<&str>,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "name": name, "description": description });
    let req = config.request(
        HttpMethod::Patch,
        &format!("/guilds/{guild_id}/templates/{code}"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn resolve_guild_template<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    code: &str,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/guilds/templates/{code}"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn create_guild_from_template<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    code: &str,
    name: &str,
    icon: Option<&str>,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "name": name, "icon": icon });
    let req = config.request(
        HttpMethod::Post,
        &format!("/guilds/templates/{code}"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn create_guild_sound<T, S>(
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
        HttpMethod::Post,
        &format!("/guilds/{guild_id}/soundboard-sounds"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn update_guild_sound<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    sound_id: u64,
    body: &Value,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Patch,
        &format!("/guilds/{guild_id}/soundboard-sounds/{sound_id}"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn delete_guild_sound<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    sound_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Delete,
        &format!("/guilds/{guild_id}/soundboard-sounds/{sound_id}"),
        config.standard_headers(),
        Body::None,
    );
    send_empty(client, req).await
}

pub async fn add_role_members<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    role_id: u64,
    member_ids: &[String],
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let body = serde_json::json!({ "member_ids": member_ids });
    let req = config.request(
        HttpMethod::Patch,
        &format!("/guilds/{guild_id}/roles/{role_id}/members"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn list_automod_rules<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/guilds/{guild_id}/auto-moderation/rules"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn create_automod_rule<T, S>(
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
        HttpMethod::Post,
        &format!("/guilds/{guild_id}/auto-moderation/rules"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn update_automod_rule<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    rule_id: u64,
    body: &Value,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Patch,
        &format!("/guilds/{guild_id}/auto-moderation/rules/{rule_id}"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn delete_automod_rule<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    rule_id: u64,
) -> Result<(), ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Delete,
        &format!("/guilds/{guild_id}/auto-moderation/rules/{rule_id}"),
        config.standard_headers(),
        Body::None,
    );
    send_empty(client, req).await
}

pub async fn list_guild_scheduled_events<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Get,
        &format!("/guilds/{guild_id}/scheduled-events"),
        config.standard_headers(),
        Body::None,
    );
    send_json(client, req).await
}

pub async fn create_guild_scheduled_event<T, S>(
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
        HttpMethod::Post,
        &format!("/guilds/{guild_id}/scheduled-events"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}

pub async fn update_guild_scheduled_event<T, S>(
    client: &RestClient<T, S>,
    config: &RestConfig,
    guild_id: u64,
    event_id: u64,
    body: &Value,
) -> Result<Value, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    let req = config.request(
        HttpMethod::Patch,
        &format!("/guilds/{guild_id}/scheduled-events/{event_id}"),
        config.standard_headers(),
        Body::Json(body.to_string()),
    );
    send_json(client, req).await
}
