use crate::snowflake::Snowflake;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnavailableGuild {
    #[serde(with = "crate::snowflake::serde")]
    pub id: Snowflake,
    #[serde(default)]
    pub unavailable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Role {
    #[serde(with = "crate::snowflake::serde")]
    pub id: Snowflake,
    #[serde(default = "zero_permissions")]
    pub permissions: String,
}

fn zero_permissions() -> String {
    "0".to_owned()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Guild {
    #[serde(with = "crate::snowflake::serde")]
    pub id: Snowflake,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub splash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner: Option<String>,
    #[serde(default, with = "crate::snowflake::serde::option")]
    pub owner_id: Option<Snowflake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<Role>,
}
