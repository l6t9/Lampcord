use crate::models;
use crate::models::marker;

#[derive(Clone, Debug)]
pub struct BackendConfig {
    pub rest_base_url: String,
    pub gateway_url: String,
    pub cdn_base_url: String,
}

impl BackendConfig {
    pub fn web_base_url(&self) -> String {
        self.rest_base_url
            .parse::<wreq::Uri>()
            .ok()
            .and_then(|uri| Some(format!("{}://{}", uri.scheme_str()?, uri.authority()?)))
            .unwrap_or_default()
    }

    pub fn avatar_url(
        &self,
        user_id: models::Id<marker::UserMarker>,
        avatar_hash: &str,
    ) -> String {
        self.avatar_url_sized(user_id, avatar_hash, 128)
    }

    pub fn avatar_url_sized(
        &self,
        user_id: models::Id<marker::UserMarker>,
        avatar_hash: &str,
        size: u32,
    ) -> String {
        format!(
            "{}/avatars/{}/{}.png?size={size}",
            self.cdn_base_url.trim_end_matches('/'),
            user_id,
            avatar_hash
        )
    }

    pub fn guild_icon_url(
        &self,
        guild_id: models::Id<marker::GuildMarker>,
        icon_hash: &str,
    ) -> String {
        format!(
            "{}/icons/{}/{}.png?size=256",
            self.cdn_base_url.trim_end_matches('/'),
            guild_id,
            icon_hash
        )
    }

    pub fn guild_banner_url(
        &self,
        guild_id: models::Id<marker::GuildMarker>,
        banner_hash: &str,
    ) -> String {
        format!(
            "{}/banners/{}/{}.png?size=600",
            self.cdn_base_url.trim_end_matches('/'),
            guild_id,
            banner_hash
        )
    }

    pub fn banner_url(
        &self,
        user_id: models::Id<marker::UserMarker>,
        banner_hash: &str,
    ) -> String {
        format!(
            "{}/banners/{}/{}.png?size=600",
            self.cdn_base_url.trim_end_matches('/'),
            user_id,
            banner_hash
        )
    }

    pub fn badge_icon_url(&self, icon_hash: &str) -> String {
        format!(
            "{}/badge-icons/{}.png?size=64",
            self.cdn_base_url.trim_end_matches('/'),
            icon_hash
        )
    }

    pub fn emoji_picture_url(&self, emoji_id: u64) -> String {
        format!(
            "{}/emojis/{}.png?size=96",
            self.cdn_base_url.trim_end_matches('/'),
            emoji_id
        )
    }

    pub fn animated_emoji_url(&self, emoji_id: u64) -> String {
        format!(
            "{}/emojis/{}.webp?size=96&animated=true",
            self.cdn_base_url.trim_end_matches('/'),
            emoji_id
        )
    }

    pub fn gif_emoji_url(&self, emoji_id: u64) -> String {
        format!(
            "{}/emojis/{}.gif",
            self.cdn_base_url.trim_end_matches('/'),
            emoji_id
        )
    }

    pub fn role_icon_url(
        &self,
        role_id: models::Id<marker::RoleMarker>,
        icon_hash: &str,
    ) -> String {
        format!(
            "{}/role-icons/{}/{}.png?size=40",
            self.cdn_base_url.trim_end_matches('/'),
            role_id,
            icon_hash
        )
    }

    pub fn guild_tag_badge_url(
        &self,
        guild_id: models::Id<marker::GuildMarker>,
        badge_hash: &str,
    ) -> String {
        format!(
            "{}/clan-badges/{}/{}.png?size=16",
            self.cdn_base_url.trim_end_matches('/'),
            guild_id,
            badge_hash
        )
    }

    pub fn app_icon_url(&self, application_id: &str, icon_hash: &str) -> String {
        format!(
            "{}/app-icons/{}/{}.png?size=48",
            self.cdn_base_url.trim_end_matches('/'),
            application_id,
            icon_hash
        )
    }

    pub fn emoji_url(&self, emoji_id: u64) -> String {
        format!(
            "{}/emojis/{}.png?size=48",
            self.cdn_base_url.trim_end_matches('/'),
            emoji_id
        )
    }
}
