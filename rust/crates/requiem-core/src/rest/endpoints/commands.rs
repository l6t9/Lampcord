use crate::models::Id;
use crate::models::application_command::CommandIndex;
use crate::models::id::marker::{ChannelMarker, GuildMarker};

use super::super::HttpClient;

pub async fn fetch_index(
    http: &HttpClient,
    guild_id: Option<Id<GuildMarker>>,
    channel_id: Id<ChannelMarker>,
) -> Result<CommandIndex, String> {
    let (route, path) = match guild_id {
        Some(guild_id) => (
            "GET /guilds/:id/application-command-index",
            format!("/guilds/{guild_id}/application-command-index"),
        ),
        None => (
            "GET /channels/:id/application-command-index",
            format!("/channels/{channel_id}/application-command-index"),
        ),
    };
    let response = http
        .request(route, wreq::Method::GET, &path)
        .await
        .map_err(|e| super::error_message(&e))?;
    response.json().await.map_err(|e| e.to_string())
}

pub async fn send_interaction(http: &HttpClient, body: &serde_json::Value) -> Result<(), String> {
    http.request_with_json(
        "POST /interactions",
        wreq::Method::POST,
        "/interactions",
        body,
    )
    .await
    .map_err(|e| super::error_message(&e))?;
    Ok(())
}
