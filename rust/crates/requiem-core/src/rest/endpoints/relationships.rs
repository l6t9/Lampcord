use crate::models::Id;
use crate::models::id::marker::UserMarker;

use super::super::{HttpClient, RequestExtras, RestError};

pub const FRIEND: u8 = 1;
pub const BLOCKED: u8 = 2;
pub const PENDING_INCOMING: u8 = 3;
pub const PENDING_OUTGOING: u8 = 4;

pub mod location {

    pub const DM_CHANNEL: &str = "DM Channel";

    pub const FRIENDS: &str = "Friends";

    pub const ADD_FRIEND: &str = "Add Friend";

    pub const CONTEXT_MENU: &str = "ContextMenu";

    pub const DM_USER_CONTEXT_MENU: &str = "DMUserContextMenu";

    pub const GROUP_DM_USER_CONTEXT_MENU: &str = "GroupDMUserContextMenu";

    pub const GUILD_CHANNEL_USER_CONTEXT_MENU: &str = "GuildChannelUserContextMenu";

    pub const PROFILE_POPOUT: &str = "user profile popout";

    pub const PROFILE_SIDEBAR: &str = "user profile sidebar";

    pub const PROFILE_MODAL: &str = "user profile modal v2";

    pub const PROFILE_OVERFLOW_MENU: &str = "user profile overflow menu";
}

pub async fn send_friend_request(
    http: &HttpClient,
    user_id: Id<UserMarker>,
    location: &str,
) -> Result<(), RestError> {
    put(http, user_id, &serde_json::json!({}), location).await
}

pub async fn send_friend_request_by_username(
    http: &HttpClient,
    username: &str,
) -> Result<(), String> {
    let body =
        serde_json::json!({ "username": username, "discriminator": serde_json::Value::Null });
    let extras = RequestExtras::context(location::ADD_FRIEND);
    match http
        .request_with(
            "POST /users/@me/relationships",
            wreq::Method::POST,
            "/users/@me/relationships",
            Some(&body),
            &extras,
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(err) => Err(super::error_message(&err)),
    }
}

pub async fn block(
    http: &HttpClient,
    user_id: Id<UserMarker>,
    location: &str,
) -> Result<(), RestError> {
    put(
        http,
        user_id,
        &serde_json::json!({ "type": BLOCKED }),
        location,
    )
    .await
}

pub async fn ignore(
    http: &HttpClient,
    user_id: Id<UserMarker>,
    location: &str,
) -> Result<(), RestError> {
    let path = format!("/users/@me/relationships/{user_id}/ignore");
    http.request_with::<()>(
        "PUT /users/@me/relationships/:id/ignore",
        wreq::Method::PUT,
        &path,
        None,
        &RequestExtras::context(location),
    )
    .await?;
    Ok(())
}

pub async fn remove(
    http: &HttpClient,
    user_id: Id<UserMarker>,
    location: &str,
) -> Result<(), RestError> {
    let path = format!("/users/@me/relationships/{user_id}");
    http.request_with::<()>(
        "DELETE /users/@me/relationships/:id",
        wreq::Method::DELETE,
        &path,
        None,
        &RequestExtras::context(location),
    )
    .await?;
    Ok(())
}

async fn put(
    http: &HttpClient,
    user_id: Id<UserMarker>,
    body: &serde_json::Value,
    location: &str,
) -> Result<(), RestError> {
    let path = format!("/users/@me/relationships/{user_id}");
    http.request_with(
        "PUT /users/@me/relationships/:id",
        wreq::Method::PUT,
        &path,
        Some(body),
        &RequestExtras::context(location),
    )
    .await?;
    Ok(())
}
