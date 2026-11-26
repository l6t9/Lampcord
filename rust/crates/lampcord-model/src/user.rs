use crate::snowflake::Snowflake;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    #[serde(with = "crate::snowflake::serde")]
    pub id: Snowflake,
    pub username: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bot: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mfa_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub premium_type: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_flags: Option<u32>,
    /// The avatar frame (an avatar decoration preset).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_decoration_data: Option<AvatarDecoration>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collectibles: Option<Collectibles>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_guild: Option<PrimaryGuild>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PrimaryGuild {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_guild_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badge: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_enabled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AvatarDecoration {
    pub asset: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sku_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nameplate {
    pub asset: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub palette: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Collectibles {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nameplate: Option<Nameplate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_decoration: Option<AvatarDecoration>,
}

impl User {
    /// The avatar frame asset, from either place Discord puts it.
    pub fn decoration_asset(&self) -> Option<&str> {
        self.avatar_decoration_data
            .as_ref()
            .or_else(|| self.collectibles.as_ref()?.avatar_decoration.as_ref())
            .map(|decoration| decoration.asset.as_str())
    }

    pub fn nameplate_asset(&self) -> Option<&str> {
        self.collectibles
            .as_ref()?
            .nameplate
            .as_ref()
            .map(|nameplate| nameplate.asset.as_str())
    }

    pub fn clan_tag(&self) -> Option<&str> {
        let pg = self.primary_guild.as_ref()?;
        if pg.identity_enabled == Some(false) {
            return None;
        }
        pg.tag.as_deref().filter(|t| !t.is_empty())
    }

    pub fn clan_badge_url(&self) -> Option<String> {
        let pg = self.primary_guild.as_ref()?;
        if pg.identity_enabled == Some(false) {
            return None;
        }
        let gid = pg.identity_guild_id.as_deref().filter(|s| !s.is_empty())?;
        let hash = pg.badge.as_deref().filter(|s| !s.is_empty())?;
        Some(format!("https://cdn.discordapp.com/clan-badges/{gid}/{hash}.png?size=32"))
    }
}
