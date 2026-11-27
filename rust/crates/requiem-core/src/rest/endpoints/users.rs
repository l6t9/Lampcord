use crate::models::id::marker::{GuildMarker, RoleMarker, UserMarker};
use crate::models::user_profile::MutualFriend;
use crate::models::{Id, UserProfile};

use super::super::{HttpClient, RestError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileView {
    Popout { is_self: bool, is_bot: bool },

    Modal { is_bot: bool },

    Sidebar,

    Plain,
}

impl ProfileView {
    pub fn query(self, guild_id: Option<Id<GuildMarker>>) -> String {
        let (kind, guilds, friends, friends_count) = match self {
            ProfileView::Popout { is_self, is_bot } => {
                (Some("popout"), !is_self, !is_bot && !is_self, false)
            }
            ProfileView::Modal { is_bot } => (Some("modal"), true, false, !is_bot),
            ProfileView::Sidebar => (Some("sidebar"), true, true, false),
            ProfileView::Plain => (None, false, false, false),
        };
        let mut parts = Vec::new();
        if let Some(kind) = kind {
            parts.push(format!("type={kind}"));
        }
        parts.push(format!("with_mutual_guilds={guilds}"));
        parts.push(format!("with_mutual_friends={friends}"));

        parts.push(format!(
            "with_mutual_friends_count={}",
            friends_count && !friends
        ));
        if let Some(guild_id) = guild_id {
            parts.push(format!("guild_id={guild_id}"));
        }
        parts.join("&")
    }
}

pub async fn get_user_profile(
    http: &HttpClient,
    user_id: Id<UserMarker>,
    guild_id: Option<Id<GuildMarker>>,
    view: ProfileView,
) -> Result<UserProfile, RestError> {
    let path = format!("/users/{user_id}/profile?{}", view.query(guild_id));
    let response = http
        .request("GET /users/:id/profile", wreq::Method::GET, &path)
        .await?;
    Ok(response.json().await?)
}

/// The friends both you and `user_id` have. The profile modal asks for the count only, and its
/// Mutual Friends tab loads this list when it opens without a complete one.
pub async fn get_mutual_friends(
    http: &HttpClient,
    user_id: Id<UserMarker>,
) -> Result<Vec<MutualFriend>, RestError> {
    let response = http
        .request(
            "GET /users/:id/relationships",
            wreq::Method::GET,
            &format!("/users/{user_id}/relationships"),
        )
        .await?;
    Ok(response.json().await?)
}

pub async fn add_member_role(
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    role_id: Id<RoleMarker>,
) -> Result<(), RestError> {
    http.request(
        "PUT /guilds/:id/members/:id/roles/:id",
        wreq::Method::PUT,
        &format!("/guilds/{guild_id}/members/{user_id}/roles/{role_id}"),
    )
    .await?;
    Ok(())
}

pub async fn remove_member_role(
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    role_id: Id<RoleMarker>,
) -> Result<(), RestError> {
    http.request(
        "DELETE /guilds/:id/members/:id/roles/:id",
        wreq::Method::DELETE,
        &format!("/guilds/{guild_id}/members/{user_id}/roles/{role_id}"),
    )
    .await?;
    Ok(())
}

pub async fn get_note(
    http: &HttpClient,
    user_id: Id<UserMarker>,
) -> Result<Option<String>, RestError> {
    #[derive(serde::Deserialize)]
    struct Note {
        note: String,
    }
    match http
        .request(
            "GET /users/@me/notes/:id",
            wreq::Method::GET,
            &format!("/users/@me/notes/{user_id}"),
        )
        .await
    {
        Ok(response) => Ok(Some(response.json::<Note>().await?.note)),
        Err(RestError::Status { status, .. }) if status == wreq::StatusCode::NOT_FOUND => Ok(None),
        Err(err) => Err(err),
    }
}

pub async fn set_note(
    http: &HttpClient,
    user_id: Id<UserMarker>,
    note: &str,
) -> Result<(), RestError> {
    http.request_with_json(
        "PUT /users/@me/notes/:id",
        wreq::Method::PUT,
        &format!("/users/@me/notes/{user_id}"),
        &serde_json::json!({ "note": note }),
    )
    .await?;
    Ok(())
}

#[derive(Clone, Debug, Default)]
pub struct ProfileUpdate {
    pub pronouns: Option<String>,
    pub bio: Option<String>,
    pub accent_color: Option<Option<i64>>,

    pub banner: Option<Option<String>>,

    pub theme_colors: Option<Option<[i64; 2]>>,
}

impl ProfileUpdate {
    pub fn is_empty(&self) -> bool {
        self.pronouns.is_none()
            && self.bio.is_none()
            && self.accent_color.is_none()
            && self.banner.is_none()
            && self.theme_colors.is_none()
    }
}

pub async fn update_profile(http: &HttpClient, update: &ProfileUpdate) -> Result<(), RestError> {
    let mut body = serde_json::Map::new();
    if let Some(pronouns) = &update.pronouns {
        body.insert("pronouns".into(), serde_json::json!(pronouns));
    }
    if let Some(bio) = &update.bio {
        body.insert("bio".into(), serde_json::json!(bio));
    }
    if let Some(accent) = update.accent_color {
        body.insert("accent_color".into(), serde_json::json!(accent));
    }
    if let Some(banner) = &update.banner {
        body.insert("banner".into(), serde_json::json!(banner));
    }
    if let Some(theme) = update.theme_colors {
        body.insert("theme_colors".into(), serde_json::json!(theme));
    }
    http.request_with_json(
        "PATCH /users/@me/profile",
        wreq::Method::PATCH,
        "/users/@me/profile",
        &body,
    )
    .await?;
    Ok(())
}

#[derive(Clone, Debug, Default)]
pub struct AccountUpdate {
    pub display_name: Option<String>,

    pub avatar: Option<Option<String>>,
}

impl AccountUpdate {
    pub fn is_empty(&self) -> bool {
        self.display_name.is_none() && self.avatar.is_none()
    }
}

pub async fn update_account(http: &HttpClient, update: &AccountUpdate) -> Result<(), RestError> {
    let mut body = serde_json::Map::new();
    if let Some(name) = &update.display_name {
        body.insert(
            "global_name".into(),
            if name.is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::json!(name)
            },
        );
    }
    if let Some(avatar) = &update.avatar {
        body.insert("avatar".into(), serde_json::json!(avatar));
    }
    http.request_with_json("PATCH /users/@me", wreq::Method::PATCH, "/users/@me", &body)
        .await?;
    Ok(())
}

pub async fn update_member_nick(
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
    nick: &str,
) -> Result<(), RestError> {
    let value = if nick.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::json!(nick)
    };
    http.request_with_json(
        "PATCH /guilds/:id/members/@me",
        wreq::Method::PATCH,
        &format!("/guilds/{guild_id}/members/@me"),
        &serde_json::json!({ "nick": value }),
    )
    .await?;
    Ok(())
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct CatalogItem {
    pub sku_id: String,
    #[serde(default)]
    pub name: String,

    #[serde(default, rename = "type")]
    pub kind: i32,
}

pub async fn list_catalog(http: &HttpClient) -> Result<Vec<CatalogItem>, RestError> {
    #[derive(serde::Deserialize)]
    struct Category {
        #[serde(default)]
        products: Vec<CatalogItem>,
    }
    match http
        .request(
            "GET /collectibles-categories",
            wreq::Method::GET,
            "/collectibles-categories",
        )
        .await
    {
        Ok(response) => Ok(response
            .json::<Vec<Category>>()
            .await?
            .into_iter()
            .flat_map(|c| c.products)
            .collect()),
        Err(RestError::Status { status, .. }) if status == wreq::StatusCode::NOT_FOUND => {
            Ok(Vec::new())
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod profile_view_tests {
    use super::*;

    #[test]
    fn each_surface_makes_the_query_discords_own_surface_makes() {
        assert_eq!(
            ProfileView::Sidebar.query(None),
            "type=sidebar&with_mutual_guilds=true&with_mutual_friends=true&with_mutual_friends_count=false"
        );
        assert_eq!(
            ProfileView::Plain.query(None),
            "with_mutual_guilds=false&with_mutual_friends=false&with_mutual_friends_count=false"
        );
        assert_eq!(
            ProfileView::Popout {
                is_self: false,
                is_bot: false
            }
            .query(Some(Id::new(5))),
            "type=popout&with_mutual_guilds=true&with_mutual_friends=true&with_mutual_friends_count=false&guild_id=5"
        );
        assert_eq!(
            ProfileView::Popout {
                is_self: true,
                is_bot: false
            }
            .query(None),
            "type=popout&with_mutual_guilds=false&with_mutual_friends=false&with_mutual_friends_count=false"
        );
        assert_eq!(
            ProfileView::Popout {
                is_self: false,
                is_bot: true
            }
            .query(None),
            "type=popout&with_mutual_guilds=true&with_mutual_friends=false&with_mutual_friends_count=false"
        );
        assert_eq!(
            ProfileView::Modal { is_bot: false }.query(None),
            "type=modal&with_mutual_guilds=true&with_mutual_friends=false&with_mutual_friends_count=true"
        );
        assert_eq!(
            ProfileView::Modal { is_bot: true }.query(None),
            "type=modal&with_mutual_guilds=true&with_mutual_friends=false&with_mutual_friends_count=false"
        );
    }
}
