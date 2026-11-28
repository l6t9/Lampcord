use serde::{Deserialize, Serialize};

use super::id::{
    Id,
    marker::{GuildMarker, UserMarker},
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AvatarDecorationData {
    #[serde(default)]
    pub asset: String,
    #[serde(default, rename = "sku_id")]
    pub sku_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Nameplate {
    #[serde(default)]
    pub asset: String,
    #[serde(default)]
    pub palette: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Collectibles {
    #[serde(default)]
    pub nameplate: Option<Nameplate>,
    #[serde(default, rename = "avatar_decoration")]
    pub avatar_decoration: Option<AvatarDecorationData>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DisplayNameStyles {
    #[serde(default, rename = "font_id")]
    pub font_id: Option<u64>,
    #[serde(default, rename = "effect_id")]
    pub effect_id: Option<u64>,
    #[serde(default)]
    pub colors: Option<Vec<i64>>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProfileEffectRef {
    #[serde(default, rename = "sku_id")]
    pub sku_id: Option<String>,
    #[serde(default, rename = "expires_at")]
    pub expires_at: Option<i64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CollectibleRef {
    #[serde(rename = "type")]
    pub kind: Option<u8>,
    #[serde(default, rename = "sku_id")]
    pub sku_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Id<UserMarker>,
    pub username: String,
    pub discriminator: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,

    #[serde(default, rename = "avatar_decoration_data")]
    pub avatar_decoration_data: Option<AvatarDecorationData>,
    #[serde(default)]
    pub collectibles: Option<Collectibles>,
    #[serde(default, rename = "display_name_styles")]
    pub display_name_styles: Option<DisplayNameStyles>,

    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,

    #[serde(default)]
    pub verified: Option<bool>,
    #[serde(default)]
    pub mfa_enabled: Option<bool>,

    #[serde(default)]
    pub bot: bool,

    #[serde(default)]
    pub premium_type: Option<i32>,

    #[serde(default)]
    pub primary_guild: Option<PrimaryGuild>,
}

impl User {
    pub fn display_name(&self, nick: Option<&str>) -> String {
        nick.map(str::to_string)
            .or_else(|| self.global_name.clone())
            .unwrap_or_else(|| self.username.clone())
    }

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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimaryGuild {
    #[serde(default)]
    pub identity_enabled: Option<bool>,
    #[serde(default, rename = "guild_id")]
    pub guild_id: Option<Id<GuildMarker>>,
    #[serde(default, rename = "identity_guild_id")]
    pub identity_guild_id: Option<Id<GuildMarker>>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub badge: Option<String>,
}

impl PrimaryGuild {
    pub fn shown(&self) -> Option<(Id<GuildMarker>, &str, Option<&str>)> {
        if self.identity_enabled == Some(false) {
            return None;
        }
        Some((
            self.identity_guild_id.or(self.guild_id)?,
            self.tag.as_deref().filter(|t| !t.is_empty())?,
            self.badge.as_deref().filter(|b| !b.is_empty()),
        ))
    }
}
impl User {
    pub fn censor_email(email: &str) -> String {
        match email.split_once('@') {
            Some((name, domain)) => format!("{}@{}", "*".repeat(name.chars().count()), domain),
            None => "*".repeat(email.chars().count()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::User;

    #[test]
    fn censor_email_masks_local_part_to_its_exact_length() {
        assert_eq!(User::censor_email("testing@example.com"), "*******@example.com");
        assert_eq!(User::censor_email("a@b.co"), "*@b.co");
    }

    #[test]
    fn censor_email_masks_an_address_with_no_at_sign_entirely() {
        assert_eq!(User::censor_email("nonsense"), "********");
    }

    #[test]
    fn censor_email_counts_characters_not_bytes() {
        assert_eq!(User::censor_email("héé@x.com"), "***@x.com");
    }
}
