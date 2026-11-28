use serde::{Deserialize, Deserializer};

use super::{
    User,
    id::{Id, marker::RoleMarker},
    user::{AvatarDecorationData, Collectibles, DisplayNameStyles},
};

#[derive(Clone, Debug, Deserialize)]
pub struct GuildMember {
    pub user: User,
    #[serde(default)]
    pub nick: Option<String>,
    #[serde(default)]
    pub roles: Vec<Id<RoleMarker>>,
    #[serde(default, rename = "avatar_decoration_data")]
    pub avatar_decoration_data: Option<AvatarDecorationData>,
    #[serde(default)]
    pub collectibles: Option<Collectibles>,
    #[serde(default, rename = "display_name_styles")]
    pub display_name_styles: Option<DisplayNameStyles>,
}

pub fn deserialize_timeout_until<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<i64>, D::Error> {
    Ok(Option::<String>::deserialize(deserializer)?
        .as_deref()
        .and_then(parse_timeout_until))
}

pub fn parse_timeout_until(time: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(time)
        .ok()
        .map(|time| time.timestamp_millis())
}

pub fn is_timed_out(until_ms: Option<i64>, now_ms: i64) -> bool {
    until_ms.is_some_and(|until| until > now_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeouts_parse_and_expire() {
        let until = parse_timeout_until("2026-10-13T12:00:00.000000+00:00").unwrap();
        assert!(is_timed_out(Some(until), until - 1));
        assert!(!is_timed_out(Some(until), until));
        assert!(!is_timed_out(None, 0));
        assert_eq!(parse_timeout_until("soon"), None);
    }
}
