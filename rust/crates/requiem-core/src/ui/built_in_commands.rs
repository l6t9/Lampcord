use crate::rest::endpoints::users;
use crate::rest;
use crate::utils::time;
use serde_json::{Value, json};

use super::context_menus::MenuEnv;
use super::slash_commands::Invocation;
use crate::models::application_command::{
    ApplicationCommand, CommandChoice, CommandEntry, CommandOption, option_type,
};
use crate::models::id::marker::ApplicationMarker;
use crate::models::{Channel, Id, Permissions};

pub const APPLICATION_ID: Id<ApplicationMarker> = Id::new(u64::MAX);

pub fn is_built_in(entry: &CommandEntry) -> bool {
    entry.command.application_id == APPLICATION_ID
}

fn text(key: &str) -> String {
    requiem_i18n::text(key)
}

fn option(command: &str, name: &str, kind: u8, required: bool) -> CommandOption {
    CommandOption {
        kind,
        name: name.to_string(),
        description: text(&format!("built-in-{command}-{name}")),
        required,
        ..Default::default()
    }
}

fn choices(list: &[(&str, i64)]) -> Vec<CommandChoice> {
    list.iter()
        .map(|(key, value)| CommandChoice {
            name: text(key),
            value: json!(value),
        })
        .collect()
}

fn entry(id: u64, name: &str, options: Vec<CommandOption>) -> CommandEntry {
    let description = text(&format!("built-in-{name}"));
    CommandEntry {
        command: ApplicationCommand {
            id: Id::new(id),
            application_id: APPLICATION_ID,
            name: name.to_string(),
            description: description.clone(),
            kind: 1,
            options: options.clone(),
            version: None,
        },
        path: Vec::new(),
        options,
        description,
    }
}

pub fn entries(env: &MenuEnv, channel: &Channel) -> Vec<CommandEntry> {
    let in_guild = channel.guild_id.is_some();
    let can = |permission: Permissions| in_guild && env.state.can(channel, permission);
    let text_command = |id: u64, name: &str, required: bool| {
        entry(
            id,
            name,
            vec![option(name, "message", option_type::STRING, required)],
        )
    };
    let mut all = vec![
        text_command(1, "shrug", false),
        text_command(2, "tableflip", false),
        text_command(3, "unflip", false),
    ];
    if can(Permissions::SEND_TTS_MESSAGES) {
        all.push(text_command(4, "tts", true));
    }
    all.push(text_command(5, "me", true));
    all.push(text_command(6, "spoiler", true));
    if can(Permissions::CHANGE_NICKNAME) || can(Permissions::MANAGE_NICKNAMES) {
        all.push(entry(
            7,
            "nick",
            vec![option("nick", "new_nick", option_type::STRING, false)],
        ));
    }
    if can(Permissions::KICK_MEMBERS) {
        all.push(entry(
            11,
            "kick",
            vec![
                option("kick", "user", option_type::USER, true),
                option("kick", "reason", option_type::STRING, false),
            ],
        ));
    }
    if can(Permissions::BAN_MEMBERS) {
        let mut delete = option("ban", "delete_messages", option_type::INTEGER, true);
        delete.choices = choices(&[
            ("built-in-delete-none", 0),
            ("built-in-delete-hour", 3600),
            ("built-in-delete-6-hours", 6 * 3600),
            ("built-in-delete-12-hours", 12 * 3600),
            ("built-in-delete-24-hours", 24 * 3600),
            ("built-in-delete-3-days", 3 * 86400),
            ("built-in-delete-7-days", 7 * 86400),
        ]);
        all.push(entry(
            12,
            "ban",
            vec![
                option("ban", "user", option_type::USER, true),
                delete,
                option("ban", "reason", option_type::STRING, false),
            ],
        ));
    }
    if can(Permissions::MODERATE_MEMBERS) {
        let mut duration = option("timeout", "duration", option_type::INTEGER, true);
        duration.choices = choices(&[
            ("built-in-timeout-60-secs", 60),
            ("built-in-timeout-5-mins", 300),
            ("built-in-timeout-10-mins", 600),
            ("built-in-timeout-1-hour", 3600),
            ("built-in-timeout-1-day", 86400),
            ("built-in-timeout-1-week", 7 * 86400),
        ]);
        all.push(entry(
            13,
            "timeout",
            vec![
                option("timeout", "user", option_type::USER, true),
                duration,
                option("timeout", "reason", option_type::STRING, false),
            ],
        ));
    }
    all
}

fn string(invocation: &Invocation, name: &str) -> String {
    invocation
        .values
        .iter()
        .find(|(_, n, _)| n == name)
        .and_then(|(_, _, v)| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

fn number(invocation: &Invocation, name: &str) -> i64 {
    invocation
        .values
        .iter()
        .find(|(_, n, _)| n == name)
        .and_then(|(_, _, v)| v.as_i64())
        .unwrap_or_default()
}

fn message_for(invocation: &Invocation) -> Option<String> {
    let message = string(invocation, "message");
    let content = match invocation.entry.command.name.as_str() {
        "shrug" => format!("{message} ¯\\_(ツ)_/¯"),
        "tableflip" => format!("{message} (╯°□°)╯︵ ┻━┻"),
        "unflip" => format!("{message} ┬─┬ ノ( º _ ºノ)"),
        "tts" => message,
        "me" => format!("_{message}_"),
        "spoiler" => format!("||{message}||"),
        _ => return None,
    };
    Some(content.trim().to_string())
}

pub fn run(env: &MenuEnv, channel: &Channel, invocation: &Invocation) {
    let Some(http) = env.http.clone() else { return };
    let channel_id = channel.id;
    let failure = |key: &str| text(key);
    if let Some(content) = message_for(invocation) {
        let tts = invocation.entry.command.name == "tts";
        super::context_menus::run_with_toast(
            &env.queries,
            Vec::new(),
            String::new(),
            failure("built-in-send-failed"),
            move || async move {
                let body = json!({ "content": content, "tts": tts, "nonce": time::snowflake_at(chrono::Utc::now().timestamp_millis()).to_string(), "flags": 0 });
                http.request_with_json(
                    "POST /channels/:id/messages",
                    wreq::Method::POST,
                    &format!("/channels/{channel_id}/messages"),
                    &body,
                )
                .await
                .map(|_| ())
            },
        );
        return;
    }
    let Some(guild_id) = channel.guild_id else {
        return;
    };
    let reason = Some(string(invocation, "reason")).filter(|r| !r.is_empty());
    let extras = rest::RequestExtras {
        reason,
        ..Default::default()
    };
    let user = string(invocation, "user");
    match invocation.entry.command.name.as_str() {
        "nick" => {
            let nick = string(invocation, "new_nick");
            super::context_menus::run_with_toast(
                &env.queries,
                Vec::new(),
                text("built-in-nick-changed"),
                failure("built-in-nick-failed"),
                move || async move {
                    users::update_member_nick(&http, guild_id, &nick).await
                },
            );
        }
        "kick" => super::context_menus::run_with_toast(
            &env.queries,
            Vec::new(),
            text("built-in-kicked"),
            failure("built-in-kick-failed"),
            move || async move {
                http.request_with(
                    "DELETE /guilds/:id/members/:id",
                    wreq::Method::DELETE,
                    &format!("/guilds/{guild_id}/members/{user}"),
                    None::<&Value>,
                    &extras,
                )
                .await
                .map(|_| ())
            },
        ),
        "ban" => {
            let seconds = number(invocation, "delete_messages");
            super::context_menus::run_with_toast(
                &env.queries,
                Vec::new(),
                text("built-in-banned"),
                failure("built-in-ban-failed"),
                move || async move {
                    let body = json!({ "delete_message_seconds": seconds });
                    http.request_with(
                        "PUT /guilds/:id/bans/:id",
                        wreq::Method::PUT,
                        &format!("/guilds/{guild_id}/bans/{user}"),
                        Some(&body),
                        &extras,
                    )
                    .await
                    .map(|_| ())
                },
            );
        }
        "timeout" => {
            let seconds = number(invocation, "duration");
            super::context_menus::run_with_toast(
                &env.queries,
                Vec::new(),
                text("built-in-timed-out"),
                failure("built-in-timeout-failed"),
                move || async move {
                    let until =
                        (chrono::Utc::now() + chrono::Duration::seconds(seconds)).to_rfc3339();
                    let body = json!({ "communication_disabled_until": until });
                    http.request_with(
                        "PATCH /guilds/:id/members/:id",
                        wreq::Method::PATCH,
                        &format!("/guilds/{guild_id}/members/{user}"),
                        Some(&body),
                        &extras,
                    )
                    .await
                    .map(|_| ())
                },
            );
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn invocation(name: &str, message: &str) -> Invocation {
        Invocation {
            entry: entry(u64::MAX, name, Vec::new()),
            values: vec![(option_type::STRING, "message".to_string(), json!(message))],
        }
    }

    #[test]
    fn text_commands_wrap_the_message() {
        assert_eq!(
            message_for(&invocation("me", "waves")).as_deref(),
            Some("_waves_")
        );
        assert_eq!(
            message_for(&invocation("spoiler", "boo")).as_deref(),
            Some("||boo||")
        );
        assert_eq!(
            message_for(&invocation("shrug", "")).as_deref(),
            Some("¯\\_(ツ)_/¯")
        );
        assert_eq!(message_for(&invocation("ban", "x")), None);
    }
}
