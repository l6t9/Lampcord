use serde::{Deserialize, Serialize};

use super::id::{
    Id,
    marker::{EmojiMarker, GuildMarker, RoleMarker, UserMarker},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Guild {
    pub id: Id<GuildMarker>,
    pub name: String,
    pub icon: Option<String>,

    #[serde(default)]
    pub owner_id: Option<Id<UserMarker>>,

    #[serde(default)]
    pub banner: Option<String>,

    #[serde(default)]
    pub features: Vec<String>,

    #[serde(default)]
    pub premium_tier: u8,
    #[serde(default)]
    pub premium_subscription_count: u32,
    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub splash: Option<String>,
    #[serde(default)]
    pub discovery_splash: Option<String>,

    #[serde(default)]
    pub default_message_notifications: u8,

    /// The server's custom invite link (`discord.gg/<code>`), if it has one.
    #[serde(default)]
    pub vanity_url_code: Option<String>,
}

impl Guild {
    pub fn new(
        id: Id<GuildMarker>,
        name: impl Into<String>,
        icon: Option<String>,
        owner_id: Option<Id<UserMarker>>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            icon,
            owner_id,
            banner: None,
            features: Vec::new(),
            premium_tier: 0,
            premium_subscription_count: 0,
            description: None,
            splash: None,
            discovery_splash: None,
            default_message_notifications: 0,
            vanity_url_code: None,
        }
    }

    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.iter().any(|f| f == feature)
    }

    pub fn is_verified(&self) -> bool {
        self.has_feature("VERIFIED")
    }

    pub fn is_partnered(&self) -> bool {
        self.has_feature("PARTNERED")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomEmoji {
    pub id: Id<EmojiMarker>,
    pub name: String,
    #[serde(default)]
    pub animated: bool,

    #[serde(default = "available_by_default")]
    pub available: bool,

    #[serde(default)]
    pub roles: Vec<Id<RoleMarker>>,
}

fn available_by_default() -> bool {
    true
}

impl CustomEmoji {
    pub fn markup(&self) -> String {
        format!(
            "<{}:{}:{}>",
            if self.animated { "a" } else { "" },
            self.name,
            self.id
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleColors {
    #[serde(default)]
    pub primary_color: i64,
    #[serde(default)]
    pub secondary_color: Option<i64>,
    #[serde(default)]
    pub tertiary_color: Option<i64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoleStyle {
    Solid,
    Gradient,
    Holographic,
}

impl RoleStyle {
    pub fn defaults(self, solid: i64) -> (i64, Option<i64>, Option<i64>) {
        match self {
            RoleStyle::Solid => (solid, None, None),
            RoleStyle::Gradient => (0x9e6bff, Some(0x9fc1ff), None),
            RoleStyle::Holographic => (0xa9c9ff, Some(0xffbbec), Some(0xffc3a0)),
        }
    }

    pub fn of(secondary: Option<i64>, tertiary: Option<i64>) -> Self {
        match (secondary, tertiary) {
            (_, Some(_)) => RoleStyle::Holographic,
            (Some(_), None) => RoleStyle::Gradient,
            _ => RoleStyle::Solid,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Role {
    pub id: Id<RoleMarker>,

    #[serde(default)]
    pub colors: RoleColors,
    pub name: String,

    #[serde(default)]
    pub color: i64,
    #[serde(default)]
    pub position: i32,

    #[serde(default)]
    pub icon: Option<String>,

    #[serde(default)]
    pub unicode_emoji: Option<String>,

    #[serde(default)]
    pub managed: bool,

    #[serde(default)]
    pub hoist: bool,

    #[serde(default)]
    pub mentionable: bool,

    #[serde(default, deserialize_with = "super::permissions::deserialize_bits")]
    pub permissions: super::Permissions,
}
