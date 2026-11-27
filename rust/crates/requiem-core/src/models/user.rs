use serde::{Deserialize, Serialize};

use super::id::{
    Id,
    marker::{GuildMarker, UserMarker},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Id<UserMarker>,
    pub username: String,
    pub discriminator: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,

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
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimaryGuild {
    #[serde(default)]
    pub identity_enabled: Option<bool>,
    #[serde(default)]
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
            self.identity_guild_id?,
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
