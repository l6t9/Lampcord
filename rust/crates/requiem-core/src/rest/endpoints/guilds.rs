use crate::models::id::marker::GuildMarker;
use crate::models::id::marker;
use crate::models::{Guild, Id, Role};
use crate::models;

use super::super::{HttpClient, RestError};

pub async fn get_my_guilds(http: &HttpClient) -> Result<Vec<Guild>, RestError> {
    let response = http
        .request(
            "GET /users/@me/guilds",
            wreq::Method::GET,
            "/users/@me/guilds",
        )
        .await?;
    Ok(response.json().await?)
}

pub async fn list_roles(
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
) -> Result<Vec<Role>, RestError> {
    let response = http
        .request(
            "GET /guilds/:id/roles",
            wreq::Method::GET,
            &format!("/guilds/{guild_id}/roles"),
        )
        .await?;
    Ok(response.json().await?)
}

pub async fn create_guild(
    http: &HttpClient,
    name: &str,
    icon: Option<&str>,
    template: &super::guild_templates::GuildTemplate,
) -> Result<Id<GuildMarker>, String> {
    let body = serde_json::json!({ "name": name, "icon": icon, "channels": template.channels, "system_channel_id": template.system_channel_id, "guild_template_code": template.code });
    let response = http
        .request_with_json("POST /guilds", wreq::Method::POST, "/guilds", &body)
        .await
        .map_err(|e| super::error_message(&e))?;
    let created: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    created
        .get("id")
        .and_then(|id| id.as_str()?.parse().ok())
        .map(Id::new)
        .ok_or_else(|| "The server was created but its id was missing.".to_string())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleEdit {
    pub name: String,
    pub color: i64,

    pub secondary: Option<i64>,
    pub tertiary: Option<i64>,
    pub hoist: bool,
    pub mentionable: bool,
    pub permissions: models::Permissions,
}

impl RoleEdit {
    pub fn of(role: &Role) -> Self {
        Self {
            name: role.name.clone(),
            color: role.color,
            secondary: role.colors.secondary_color,
            tertiary: role.colors.tertiary_color,
            hoist: role.hoist,
            mentionable: role.mentionable,
            permissions: role.permissions,
        }
    }
}

pub async fn create_role(http: &HttpClient, guild_id: Id<GuildMarker>) -> Result<Role, String> {
    let body = serde_json::json!({ "name": "new role", "color": 0, "colors": { "primary_color": 0, "secondary_color": null, "tertiary_color": null }, "permissions": "0" });
    let response = http
        .request_with_json(
            "POST /guilds/:id/roles",
            wreq::Method::POST,
            &format!("/guilds/{guild_id}/roles"),
            &body,
        )
        .await
        .map_err(|e| super::error_message(&e))?;
    response.json().await.map_err(|e| e.to_string())
}

pub async fn update_role(
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
    role_id: Id<marker::RoleMarker>,
    edit: &RoleEdit,
) -> Result<Role, String> {
    let body = serde_json::json!({ "name": edit.name, "color": edit.color, "colors": { "primary_color": edit.color, "secondary_color": edit.secondary, "tertiary_color": edit.tertiary }, "hoist": edit.hoist, "mentionable": edit.mentionable, "permissions": edit.permissions.0.to_string() });
    let response = http
        .request_with_json(
            "PATCH /guilds/:id/roles/:id",
            wreq::Method::PATCH,
            &format!("/guilds/{guild_id}/roles/{role_id}"),
            &body,
        )
        .await
        .map_err(|e| super::error_message(&e))?;
    response.json().await.map_err(|e| e.to_string())
}

pub async fn delete_role(
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
    role_id: Id<marker::RoleMarker>,
) -> Result<(), String> {
    http.request(
        "DELETE /guilds/:id/roles/:id",
        wreq::Method::DELETE,
        &format!("/guilds/{guild_id}/roles/{role_id}"),
    )
    .await
    .map_err(|e| super::error_message(&e))?;
    Ok(())
}

pub async fn role_member_ids(
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
    role_id: Id<marker::RoleMarker>,
) -> Result<Vec<Id<marker::UserMarker>>, RestError> {
    let response = http
        .request(
            "GET /guilds/:id/roles/:id/member-ids",
            wreq::Method::GET,
            &format!("/guilds/{guild_id}/roles/{role_id}/member-ids"),
        )
        .await?;
    Ok(response.json().await?)
}

pub async fn add_role_members(
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
    role_id: Id<marker::RoleMarker>,
    member_ids: &[Id<marker::UserMarker>],
) -> Result<(), String> {
    let body = serde_json::json!({ "member_ids": member_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>() });
    http.request_with_json(
        "PATCH /guilds/:id/roles/:id/members",
        wreq::Method::PATCH,
        &format!("/guilds/{guild_id}/roles/{role_id}/members"),
        &body,
    )
    .await
    .map_err(|e| super::error_message(&e))?;
    Ok(())
}

#[derive(Clone, Debug, Default)]
pub struct GuildEdit {
    pub name: Option<String>,
    pub icon: Option<Option<String>>,
    pub banner: Option<Option<String>>,
    pub description: Option<Option<String>>,
}

pub async fn update_guild(
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
    edit: &GuildEdit,
) -> Result<(), String> {
    let mut body = serde_json::Map::new();
    if let Some(name) = &edit.name {
        body.insert("name".into(), name.clone().into());
    }
    for (key, value) in [
        ("icon", &edit.icon),
        ("banner", &edit.banner),
        ("description", &edit.description),
    ] {
        if let Some(value) = value {
            body.insert(
                key.into(),
                value.clone().map_or(serde_json::Value::Null, Into::into),
            );
        }
    }
    http.request_with_json(
        "PATCH /guilds/:id",
        wreq::Method::PATCH,
        &format!("/guilds/{guild_id}"),
        &serde_json::Value::Object(body),
    )
    .await
    .map_err(|e| super::error_message(&e))?;
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct GuildTrait {
    #[serde(default)]
    pub emoji_id: Option<Id<marker::EmojiMarker>>,
    #[serde(default)]
    pub emoji_name: Option<String>,
    #[serde(default)]
    pub emoji_animated: bool,
    pub label: String,
    #[serde(default)]
    pub position: i32,
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct GuildProfile {
    #[serde(default)]
    pub member_count: u32,
    #[serde(default)]
    pub online_count: u32,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub traits: Vec<GuildTrait>,
    #[serde(default)]
    pub game_application_ids: Vec<String>,

    #[serde(default)]
    pub visibility: u32,
}

pub async fn get_guild_profile(
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
) -> Result<GuildProfile, RestError> {
    let response = http
        .request(
            "GET /guilds/:id/profile",
            wreq::Method::GET,
            &format!("/guilds/{guild_id}/profile"),
        )
        .await?;
    Ok(response.json().await?)
}

#[derive(Clone, Debug)]
pub struct GuildProfileEdit {
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub visibility: u32,

    pub brand_color_primary: Option<String>,
    pub traits: Vec<GuildTrait>,
    pub game_application_ids: Vec<String>,
}

pub async fn update_guild_profile(
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
    edit: &GuildProfileEdit,
) -> Result<(), String> {
    let body = serde_json::json!({
        "name": edit.name,
        "description": edit.description,
        "icon": edit.icon,
        "custom_banner": serde_json::Value::Null,
        "visibility": edit.visibility,
        "brand_color_primary": edit.brand_color_primary,
        "traits": edit.traits,
        "game_application_ids": edit.game_application_ids,
    });
    http.request_with_json(
        "PATCH /guilds/:id/profile",
        wreq::Method::PATCH,
        &format!("/guilds/{guild_id}/profile"),
        &body,
    )
    .await
    .map_err(|e| super::error_message(&e))?;
    Ok(())
}

pub async fn leave_guild(http: &HttpClient, guild_id: Id<GuildMarker>) -> Result<(), String> {
    http.request_with_json(
        "DELETE /users/@me/guilds/:id",
        wreq::Method::DELETE,
        &format!("/users/@me/guilds/{guild_id}"),
        &serde_json::json!({ "lurking": false }),
    )
    .await
    .map_err(|e| super::error_message(&e))?;
    Ok(())
}

pub async fn delete_guild(http: &HttpClient, guild_id: Id<GuildMarker>) -> Result<(), String> {
    http.request(
        "POST /guilds/:id/delete",
        wreq::Method::POST,
        &format!("/guilds/{guild_id}/delete"),
    )
    .await
    .map_err(|e| super::error_message(&e))?;
    Ok(())
}
