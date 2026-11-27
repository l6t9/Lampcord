use crate::models::id::marker::{ChannelMarker, GuildMarker, UserMarker};
use crate::models::permissions::OverwriteKind;
use crate::models::{Channel, Id, Permissions};
use crate::models;

use super::super::{HttpClient, RequestExtras, RestError};

pub async fn get_guild_channels(
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
) -> Result<Vec<Channel>, RestError> {
    let response = http
        .request(
            "GET /guilds/:id/channels",
            wreq::Method::GET,
            &format!("/guilds/{guild_id}/channels"),
        )
        .await?;
    Ok(response.json().await?)
}

pub async fn open_private_channel(
    http: &HttpClient,
    user_ids: &[Id<UserMarker>],
    location: Option<&str>,
) -> Result<Channel, RestError> {
    let body = serde_json::json!({ "recipients": user_ids.iter().map(ToString::to_string).collect::<Vec<_>>() });
    let response = http
        .request_with(
            "POST /users/@me/channels",
            wreq::Method::POST,
            "/users/@me/channels",
            Some(&body),
            &RequestExtras {
                retries: 3,
                ..RequestExtras::context_opt(location)
            },
        )
        .await?;
    Ok(response.json().await?)
}

pub async fn create_dm(
    http: &HttpClient,
    user_id: Id<UserMarker>,
    location: Option<&str>,
) -> Result<Channel, RestError> {
    open_private_channel(http, &[user_id], location).await
}

pub async fn create_group_dm(
    http: &HttpClient,
    user_ids: &[Id<UserMarker>],
    location: Option<&str>,
) -> Result<Channel, RestError> {
    open_private_channel(http, user_ids, location).await
}

pub async fn send_typing(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
) -> Result<(), RestError> {
    http.request(
        "POST /channels/:id/typing",
        wreq::Method::POST,
        &format!("/channels/{channel_id}/typing"),
    )
    .await?;
    Ok(())
}

pub async fn create_guild_channel(
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
    name: &str,
    kind: u8,
    parent: Option<Id<ChannelMarker>>,
    overwrites: &[models::PermissionOverwrite],
) -> Result<Channel, String> {
    let mut body =
        serde_json::json!({ "type": kind, "name": name, "permission_overwrites": overwrites });
    if let Some(parent) = parent {
        body["parent_id"] = serde_json::json!(parent.to_string());
    }
    let response = http
        .request_with_json(
            "POST /guilds/:id/channels",
            wreq::Method::POST,
            &format!("/guilds/{guild_id}/channels"),
            &body,
        )
        .await
        .map_err(|e| super::error_message(&e))?;
    response.json().await.map_err(|e| e.to_string())
}

pub async fn edit_permission_overwrite(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    overwrite_id: u64,
    kind: OverwriteKind,
    allow: Permissions,
    deny: Permissions,
) -> Result<(), RestError> {
    let body = overwrite_body(kind, allow, deny);
    http.request_with_json(
        "PUT /channels/:id/permissions/:id",
        wreq::Method::PUT,
        &format!("/channels/{channel_id}/permissions/{overwrite_id}"),
        &body,
    )
    .await?;
    Ok(())
}

pub async fn delete_permission_overwrite(
    http: &HttpClient,
    channel_id: Id<ChannelMarker>,
    overwrite_id: u64,
) -> Result<(), RestError> {
    http.request(
        "DELETE /channels/:id/permissions/:id",
        wreq::Method::DELETE,
        &format!("/channels/{channel_id}/permissions/{overwrite_id}"),
    )
    .await?;
    Ok(())
}

fn overwrite_body(kind: OverwriteKind, allow: Permissions, deny: Permissions) -> serde_json::Value {
    serde_json::json!({ "type": kind, "allow": allow.0.to_string(), "deny": deny.0.to_string() })
}

#[cfg(test)]
mod overwrite_tests {
    use super::*;

    #[test]
    fn the_overwrite_body_has_string_bits_and_a_numeric_type() {
        let body = overwrite_body(
            OverwriteKind::Member,
            Permissions::VIEW_CHANNEL,
            Permissions::SEND_MESSAGES,
        );
        assert_eq!(
            body,
            serde_json::json!({ "type": 1, "allow": "1024", "deny": "2048" })
        );
    }
}
