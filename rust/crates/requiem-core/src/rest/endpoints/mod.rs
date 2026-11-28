pub mod attachments;
pub mod auth;
pub mod channels;
pub mod commands;
pub mod emoji_frecency;
pub mod favorite_gifs;
pub mod gifs;
pub mod guild_templates;
pub mod guilds;
pub mod invites;
pub mod messages;
pub mod pins;
pub mod profile_board;
mod protobuf;
pub mod reactions;
pub mod read_states;
pub mod relationships;
pub mod search;
pub mod threads;
pub mod top_emojis;
pub mod user_guild_settings;
pub mod user_settings;
pub mod users;

pub fn error_message(err: &super::RestError) -> String {
    match err {
        super::RestError::Status { body, .. } => serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| v.get("message")?.as_str().map(str::to_owned))
            .unwrap_or_else(|| "Something went wrong.".to_string()),
        other => other.to_string(),
    }
}
