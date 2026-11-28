use serde::Deserialize;
use std::collections;

use super::id::Id;
use super::id::marker::{GuildMarker, RoleMarker, UserMarker};
use super::user::{
    AvatarDecorationData, CollectibleRef, Collectibles, DisplayNameStyles, ProfileEffectRef,
};

#[derive(Clone, Debug, Deserialize)]
pub struct UserProfile {
    pub user: ProfileUser,
    #[serde(default)]
    pub user_profile: Option<ProfileMeta>,
    #[serde(default)]
    pub badges: Vec<ProfileBadge>,
    #[serde(default)]
    pub premium_type: Option<i32>,
    #[serde(default)]
    pub mutual_guilds: Option<Vec<MutualGuild>>,
    #[serde(default)]
    pub mutual_friends: Option<Vec<MutualFriend>>,
    #[serde(default)]
    pub mutual_friends_count: Option<u32>,

    #[serde(default)]
    pub guild_member: Option<ProfileGuildMember>,

    #[serde(default)]
    pub guild_member_profile: Option<ProfileMeta>,

    #[serde(default)]
    pub connected_accounts: Vec<ConnectedAccount>,

    /// Made from what the client already knew because the profile request failed.
    #[serde(skip)]
    pub partial: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConnectedAccount {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub verified: bool,

    #[serde(default)]
    pub metadata: collections::HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProfileUser {
    pub id: Id<UserMarker>,
    pub username: String,

    #[serde(default = "default_discriminator")]
    pub discriminator: String,
    #[serde(default)]
    pub global_name: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub bot: bool,
    #[serde(default)]
    pub primary_guild: Option<super::user::PrimaryGuild>,
    #[serde(default, rename = "avatar_decoration_data")]
    pub avatar_decoration_data: Option<AvatarDecorationData>,
    #[serde(default)]
    pub collectibles: Option<Collectibles>,
    #[serde(default, rename = "display_name_styles")]
    pub display_name_styles: Option<DisplayNameStyles>,

    #[serde(default)]
    pub banner: Option<String>,

    #[serde(default)]
    pub accent_color: Option<i64>,

    #[serde(default)]
    pub bio: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ProfileMeta {
    #[serde(default)]
    pub pronouns: String,

    #[serde(default)]
    pub theme_colors: Option<Vec<i64>>,

    #[serde(default, rename = "display_name_styles")]
    pub display_name_styles: Option<DisplayNameStyles>,
    #[serde(default, rename = "profile_effect")]
    pub profile_effect: Option<ProfileEffectRef>,
    #[serde(default, rename = "profile_frame")]
    pub profile_frame: Option<ProfileEffectRef>,
    #[serde(default)]
    pub collectibles: Option<Vec<CollectibleRef>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProfileBadge {
    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MutualGuild {
    pub id: Id<GuildMarker>,
    #[serde(default)]
    pub nick: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MutualFriend {
    pub id: Id<UserMarker>,
    pub username: String,

    #[serde(default)]
    pub discriminator: Option<String>,
    #[serde(default)]
    pub global_name: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProfileGuildMember {
    #[serde(default)]
    pub roles: Vec<Id<RoleMarker>>,

    #[serde(default)]
    pub joined_at: Option<String>,
    #[serde(default)]
    pub nick: Option<String>,
    #[serde(default, rename = "avatar_decoration_data")]
    pub avatar_decoration_data: Option<AvatarDecorationData>,
    #[serde(default)]
    pub collectibles: Option<Collectibles>,
    #[serde(default, rename = "display_name_styles")]
    pub display_name_styles: Option<DisplayNameStyles>,
}

impl ProfileUser {
    pub fn nameplate_asset(&self) -> Option<&str> {
        let asset = self.collectibles.as_ref()?.nameplate.as_ref()?.asset.as_str();
        (!asset.is_empty()).then_some(asset)
    }

    pub fn avatar_decoration_asset(&self) -> Option<&str> {
        self.avatar_decoration_data
            .as_ref()
            .map(|data| data.asset.as_str())
            .filter(|asset| !asset.is_empty())
            .or_else(|| {
                let asset = self
                    .collectibles
                    .as_ref()?
                    .avatar_decoration
                    .as_ref()?
                    .asset
                    .as_str();
                (!asset.is_empty()).then_some(asset)
            })
    }
}

fn default_discriminator() -> String {
    "0".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_member_join_date_and_connections() {
        let json = serde_json::json!({
            "user": { "id": "1", "username": "a" },
            "guild_member": { "roles": [], "joined_at": "2026-07-17T10:00:00+00:00", "nick": null },
            "connected_accounts": [{ "type": "steam", "id": "7", "name": "Gabe", "verified": true }],
        });
        let profile: UserProfile = serde_json::from_value(json).unwrap();
        assert_eq!(
            profile.guild_member.unwrap().joined_at.as_deref(),
            Some("2026-07-17T10:00:00+00:00")
        );
        assert_eq!(profile.connected_accounts[0].kind, "steam");
        assert!(profile.connected_accounts[0].verified);
    }
}
