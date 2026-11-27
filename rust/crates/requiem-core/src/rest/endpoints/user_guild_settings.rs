use serde_json::Value;

use crate::models::Id;
use crate::models::id::marker::GuildMarker;
use crate::state::wire::UserGuildSettingsEntry;

use super::super::HttpClient;

pub async fn update(
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
    changes: &Value,
) -> Result<UserGuildSettingsEntry, String> {
    let response = http
        .request_with_json(
            "PATCH /users/@me/guilds/:id/settings",
            wreq::Method::PATCH,
            &format!("/users/@me/guilds/{guild_id}/settings"),
            changes,
        )
        .await
        .map_err(|e| super::error_message(&e))?;
    response.json().await.map_err(|e| e.to_string())
}
