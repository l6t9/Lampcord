use crate::gateway::events::GatewayEvent;
use crate::gateway::session;
use crate::models;
use crate::utils::time;
use std::env;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};

use super::context_menus::MenuEnv;
use crate::models::application_command::{CommandEntry, CommandIndex, CommandOption, option_type};
use crate::models::id::marker::{ChannelMarker, GuildMarker};
use crate::models::{Channel, Id};
use crate::rest::endpoints::commands;
use requiem_ui::widget::primitives::AutocompleteCandidate;

pub(super) fn spoofed() -> bool {
    cfg!(debug_assertions) && env::var_os("REQUIEM_SPOOF_COMMANDS").is_some()
}

fn query_key(guild_id: Option<Id<GuildMarker>>, channel_id: Id<ChannelMarker>) -> String {
    match guild_id {
        Some(guild_id) => format!("command-index:guild:{guild_id}"),
        None => format!("command-index:channel:{channel_id}"),
    }
}

fn spoof_index() -> CommandIndex {
    let value = json!({
        "applications": [
            { "id": "900000000000000001", "name": "Moderator", "icon": null },
            { "id": "900000000000000002", "name": "Fun Bot", "icon": null }
        ],
        "application_commands": [
            { "id": "910000000000000001", "application_id": "900000000000000001", "name": "ban", "description": "Ban a member from the server", "type": 1, "version": "920000000000000001",
              "options": [
                { "type": 6, "name": "user", "description": "Who to ban", "required": true },
                { "type": 3, "name": "reason", "description": "Why they are banned", "required": false },
                { "type": 4, "name": "delete_days", "description": "Days of messages to delete", "required": false,
                  "choices": [ { "name": "Don't delete any", "value": 0 }, { "name": "Previous 24 hours", "value": 1 }, { "name": "Previous 7 days", "value": 7 } ] }
              ] },
            { "id": "910000000000000002", "application_id": "900000000000000001", "name": "permissions", "description": "Get or edit permissions", "type": 1, "version": "920000000000000002",
              "options": [
                { "type": 2, "name": "user", "description": "Permissions of a user", "options": [
                    { "type": 1, "name": "get", "description": "Get a user's permissions", "options": [
                        { "type": 6, "name": "user", "description": "The user", "required": true },
                        { "type": 7, "name": "channel", "description": "A channel to look at", "required": false } ] } ] },
                { "type": 1, "name": "role", "description": "Get a role's permissions", "options": [
                    { "type": 8, "name": "role", "description": "The role", "required": true } ] }
              ] },
            { "id": "910000000000000003", "application_id": "900000000000000002", "name": "roll", "description": "Roll a die", "type": 1, "version": "920000000000000003",
              "options": [
                { "type": 4, "name": "sides", "description": "How many sides it has", "required": false },
                { "type": 5, "name": "secret", "description": "Only you see the result", "required": false } ] },
            { "id": "910000000000000004", "application_id": "900000000000000002", "name": "echo", "description": "Repeat what you say", "type": 1, "version": "920000000000000004",
              "options": [ { "type": 3, "name": "text", "description": "What to repeat", "required": true } ] },
            { "id": "910000000000000005", "application_id": "900000000000000002", "name": "remind", "description": "Remind you of something", "type": 1, "version": "920000000000000005",
              "options": [
                { "type": 10, "name": "hours", "description": "In how many hours", "required": true },
                { "type": 9, "name": "who", "description": "Whom to remind", "required": false },
                { "type": 3, "name": "what", "description": "About what", "required": false } ] }
        ]
    });
    serde_json::from_value(value).expect("the spoofed command index is valid")
}

pub fn index_for(env: &MenuEnv, channel: &Channel) -> Option<Arc<CommandIndex>> {
    let key = query_key(channel.guild_id, channel.id);
    if spoofed() {
        env.queries.set_data(&key, spoof_index());
        return env.queries.peek::<CommandIndex>(&key);
    }
    let http = env.http.clone()?;
    let (guild_id, channel_id) = (channel.guild_id, channel.id);
    env.queries
        .query::<CommandIndex, _, String>(&key, Duration::from_secs(300), move || async move {
            commands::fetch_index(&http, guild_id, channel_id).await
        })
        .data
}

pub fn candidates(
    env: &MenuEnv,
    index: &CommandIndex,
    channel_id: Id<ChannelMarker>,
) -> Vec<AutocompleteCandidate> {
    index
        .entries()
        .into_iter()
        .map(|entry| {
            let app = index.app(entry.command.application_id);
            let icon = app.and_then(|app| Some((app.id, app.icon.clone()?)));
            let avatar = icon.map(|(id, hash)| {
                let (url, images) = (
                    env.backend.app_icon_url(&id.to_string(), &hash),
                    env.images.clone(),
                );
                Rc::new(move || images.get_or_decode_image(&url))
                    as Rc<dyn Fn() -> Option<skia_safe::Image>>
            });
            let app_id = entry.command.application_id.get();
            let display = entry.display();
            let (begin_entry, begin_app) = (entry.clone(), app.cloned());
            AutocompleteCandidate {
                on_select: Some(Rc::new(move || {
                    super::command_composer::begin(
                        channel_id,
                        begin_entry.clone(),
                        begin_app.clone(),
                    )
                })),
                subtitle: Some(entry.description.clone()),
                avatar_color: Some(super::default_avatar_color(Id::new(app_id.max(1)), "0")),
                avatar,
                show_trigger: true,
                display,
                ..Default::default()
            }
        })
        .collect()
}

pub fn is_structural(option: &CommandOption) -> bool {
    matches!(
        option.kind,
        option_type::SUB_COMMAND | option_type::SUB_COMMAND_GROUP
    )
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Unknown,
    Missing(String),
    Invalid {
        option: String,
        expected: &'static str,
    },
}

#[derive(Debug, PartialEq)]
pub struct Invocation {
    pub entry: CommandEntry,
    pub values: Vec<(u8, String, Value)>,
}

pub fn parse(text: &str, entries: &[CommandEntry]) -> Result<Invocation, ParseError> {
    let rest = text
        .trim_start()
        .strip_prefix('/')
        .ok_or(ParseError::Unknown)?;
    let lowered = rest.to_lowercase();
    let entry = entries
        .iter()
        .filter(|entry| {
            let name = entry.display().to_lowercase();
            lowered.starts_with(&name)
                && lowered[name.len()..]
                    .chars()
                    .next()
                    .is_none_or(char::is_whitespace)
        })
        .max_by_key(|entry| entry.display().len())
        .ok_or(ParseError::Unknown)?;
    let typed = &rest[entry.display().len()..];

    let mut marks: Vec<(usize, usize, &CommandOption)> = Vec::new();
    for option in &entry.options {
        let marker = format!("{}:", option.name);
        let mut from = 0;
        while let Some(found) = typed[from..].find(&marker) {
            let at = from + found;
            if at == 0 || typed[..at].ends_with(char::is_whitespace) {
                marks.push((at, at + marker.len(), option));
                break;
            }
            from = at + marker.len();
        }
    }
    marks.sort_by_key(|mark| mark.0);

    let mut values = Vec::new();

    let rest_text = typed.trim();
    if marks.is_empty()
        && !rest_text.is_empty()
        && super::built_in_commands::is_built_in(entry)
        && entry.options.len() == 1
    {
        let option = &entry.options[0];
        values.push((option.kind, option.name.clone(), coerce(option, rest_text)?));
    }
    for (i, (_, value_start, option)) in marks.iter().enumerate() {
        let end = marks.get(i + 1).map_or(typed.len(), |next| next.0);
        let raw = typed[*value_start..end].trim();
        if raw.is_empty() {
            continue;
        }
        values.push((option.kind, option.name.clone(), coerce(option, raw)?));
    }
    for option in entry.options.iter().filter(|o| o.required) {
        if !values.iter().any(|(_, name, _)| *name == option.name) {
            return Err(ParseError::Missing(option.name.clone()));
        }
    }
    Ok(Invocation {
        entry: entry.clone(),
        values,
    })
}

fn mention_id(raw: &str) -> Option<String> {
    let inner = raw
        .strip_prefix('<')
        .and_then(|r| r.strip_suffix('>'))
        .unwrap_or(raw);
    let digits = inner.trim_start_matches(['@', '#', '&', '!']);
    (!digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())).then(|| digits.to_string())
}

pub fn coerce(option: &CommandOption, raw: &str) -> Result<Value, ParseError> {
    let invalid = |expected| ParseError::Invalid {
        option: option.name.clone(),
        expected,
    };

    if let Some(choice) = option
        .choices
        .iter()
        .find(|c| c.name.eq_ignore_ascii_case(raw) || c.value.to_string().trim_matches('"') == raw)
    {
        return Ok(choice.value.clone());
    }
    if !option.choices.is_empty() {
        return Err(invalid("one of the choices"));
    }
    match option.kind {
        option_type::STRING => Ok(Value::String(raw.to_string())),
        option_type::INTEGER => raw
            .parse::<i64>()
            .map(Value::from)
            .map_err(|_| invalid("a whole number")),
        option_type::NUMBER => raw
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(Value::Number)
            .ok_or_else(|| invalid("a number")),
        option_type::BOOLEAN => match raw.to_lowercase().as_str() {
            "true" | "yes" | "on" => Ok(Value::Bool(true)),
            "false" | "no" | "off" => Ok(Value::Bool(false)),
            _ => Err(invalid("true or false")),
        },
        option_type::USER | option_type::CHANNEL | option_type::ROLE | option_type::MENTIONABLE => {
            mention_id(raw)
                .map(Value::String)
                .ok_or_else(|| invalid("a mention"))
        }
        _ => Err(invalid("a value this client can send")),
    }
}

pub fn interaction_body(
    invocation: &Invocation,
    guild_id: Option<Id<GuildMarker>>,
    channel_id: Id<ChannelMarker>,
    session_id: Option<String>,
    nonce: String,
) -> Value {
    let mut options: Vec<Value> = invocation
        .values
        .iter()
        .map(|(kind, name, value)| json!({ "type": kind, "name": name, "value": value }))
        .collect();
    for (kind, name) in invocation.entry.path.iter().rev() {
        options = vec![json!({ "type": kind, "name": name, "options": options })];
    }
    let command = &invocation.entry.command;
    let mut body = json!({
        "type": 2,
        "application_id": command.application_id.to_string(),
        "channel_id": channel_id.to_string(),
        "data": {
            "version": command.version.unwrap_or(command.id).to_string(),
            "id": command.id.to_string(),
            "name": command.name,
            "type": 1,
            "options": options,
        },
        "nonce": nonce,
        "analytics_location": "slash_ui",
    });
    if let Some(guild_id) = guild_id {
        body["guild_id"] = json!(guild_id.to_string());
    }
    if let Some(session_id) = session_id {
        body["session_id"] = json!(session_id);
    }
    body
}

pub fn try_send(env: &MenuEnv, channel: &Channel, text: &str) -> bool {
    if !text.trim_start().starts_with('/') {
        return false;
    }
    let mut entries = env
        .queries
        .peek::<CommandIndex>(&query_key(channel.guild_id, channel.id))
        .map(|index| index.entries())
        .unwrap_or_default();
    entries.extend(super::built_in_commands::entries(env, channel));
    let invocation = match parse(text, &entries) {
        Ok(invocation) => invocation,
        Err(ParseError::Unknown) => return false,
        Err(ParseError::Missing(name)) => {
            super::context_menu::notify_failure(&requiem_i18n::t!(
                "slash-commands-missing-option",
                option = name
            ));
            return true;
        }
        Err(ParseError::Invalid { option, expected }) => {
            super::context_menu::notify_failure(&requiem_i18n::t!(
                "slash-commands-invalid-option",
                option = option,
                expected = expected
            ));
            return true;
        }
    };
    dispatch(env, channel, &invocation)
}

pub fn dispatch(env: &MenuEnv, channel: &Channel, invocation: &Invocation) -> bool {
    if super::built_in_commands::is_built_in(&invocation.entry) {
        super::built_in_commands::run(env, channel, invocation);
        return true;
    }
    let nonce = time::snowflake_at(chrono::Utc::now().timestamp_millis()).to_string();
    let body = interaction_body(
        invocation,
        channel.guild_id,
        channel.id,
        session::current_id(),
        nonce,
    );
    if spoofed() {
        log::info!("spoofed interaction: {body}");
        super::context_menu::notify(&format!(
            "/{} {}",
            invocation.entry.display(),
            body["data"]["options"]
        ));

        if let Some(me) = env.state.users.current.with_untracked(|u| u.clone()) {
            let id = time::snowflake_at(chrono::Utc::now().timestamp_millis());
            let app_id = invocation.entry.command.application_id.get().max(1);
            let event = serde_json::json!({
                "id": id.to_string(), "channel_id": channel.id.to_string(), "guild_id": channel.guild_id.map(|g| g.to_string()),
                "type": 20, "flags": 64, "timestamp": chrono::Utc::now().to_rfc3339(), "content": format!("Ran /{} with {}", invocation.entry.display(), body["data"]["options"]),
                "author": { "id": app_id.to_string(), "username": "Moderator", "discriminator": "0", "bot": true },
                "interaction": { "id": id.to_string(), "type": 2, "name": invocation.entry.display(), "user": { "id": me.id.to_string(), "username": me.username, "discriminator": me.discriminator, "global_name": me.global_name, "avatar": me.avatar } },
                "attachments": [], "embeds": [], "mentions": [], "mention_roles": [], "mention_everyone": false, "pinned": false,
            });
            env.state
                .apply_event(&GatewayEvent::new(
                    "MESSAGE_CREATE",
                    event,
                ));
            spoof_messages(env, channel, &me);
        }
        return true;
    }
    let Some(http) = env.http.clone() else {
        return false;
    };
    super::context_menus::run_with_toast(
        &env.queries,
        Vec::new(),
        String::new(),
        requiem_i18n::t!("slash-commands-did-not-respond").to_string(),
        move || async move { commands::send_interaction(&http, &body).await },
    );
    true
}

fn spoof_messages(env: &MenuEnv, channel: &Channel, me: &models::User) {
    let base = time::snowflake_at(chrono::Utc::now().timestamp_millis());
    let author = serde_json::json!({ "id": me.id.to_string(), "username": me.username, "discriminator": me.discriminator, "global_name": me.global_name, "avatar": me.avatar });
    let message = |offset: u64, kind: u8, content: &str, extra: Value| {
        let mut event = serde_json::json!({
            "id": (base + offset).to_string(), "channel_id": channel.id.to_string(), "guild_id": channel.guild_id.map(|g| g.to_string()),
            "type": kind, "flags": 0, "timestamp": chrono::Utc::now().to_rfc3339(), "content": content, "author": author.clone(),
            "attachments": [], "embeds": [], "mentions": [], "mention_roles": [], "mention_everyone": false, "pinned": false,
        });
        if let (Some(target), Some(extra)) = (event.as_object_mut(), extra.as_object()) {
            target.extend(extra.clone());
        }
        env.state
            .apply_event(&GatewayEvent::new(
                "MESSAGE_CREATE",
                event,
            ));
    };
    message(10, 7, "", json!({}));
    let first = (base + 20).to_string();
    message(20, 0, "a message to reply to, with some text", json!({}));
    message(
        30,
        0,
        "replying",
        json!({ "type": 19, "message_reference": { "message_id": first, "channel_id": channel.id.to_string() }, "referenced_message": { "id": first, "channel_id": channel.id.to_string(), "type": 0, "content": "a message to reply to, with some text that goes on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on and on", "author": author.clone(), "timestamp": chrono::Utc::now().to_rfc3339(), "attachments": [], "embeds": [], "mentions": [], "mention_roles": [], "mention_everyone": false, "pinned": false, "flags": 0 } }),
    );
    message(
        40,
        0,
        "hey @everyone and @here, <@&1> and <#1> and <@1>",
        json!({ "mention_everyone": true }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries() -> Vec<CommandEntry> {
        spoof_index().entries()
    }

    #[test]
    fn subcommands_are_flattened() {
        let names: Vec<String> = entries().iter().map(CommandEntry::display).collect();
        assert!(names.contains(&"permissions user get".to_string()));
        assert!(names.contains(&"permissions role".to_string()));
        assert!(
            !names.contains(&"permissions".to_string()),
            "a command with subcommands can't be run itself"
        );
        assert!(names.contains(&"ban".to_string()));
    }

    #[test]
    fn options_are_read_by_name_and_coerced() {
        let found = parse(
            "/ban user:<@123> reason: spamming the chat delete_days:Previous 7 days",
            &entries(),
        )
        .unwrap();
        assert_eq!(
            found.values,
            vec![
                (6, "user".to_string(), json!("123")),
                (3, "reason".to_string(), json!("spamming the chat")),
                (4, "delete_days".to_string(), json!(7))
            ]
        );
    }

    #[test]
    fn a_missing_required_option_is_reported() {
        assert_eq!(
            parse("/echo", &entries()),
            Err(ParseError::Missing("text".to_string()))
        );
        assert_eq!(
            parse("/roll sides:abc", &entries()),
            Err(ParseError::Invalid {
                option: "sides".to_string(),
                expected: "a whole number"
            })
        );
    }

    #[test]
    fn text_that_is_not_a_command_is_left_alone() {
        assert_eq!(parse("/shrug hello", &entries()), Err(ParseError::Unknown));
        assert_eq!(parse("hello /ban", &entries()), Err(ParseError::Unknown));

        assert_eq!(parse("/bank", &entries()), Err(ParseError::Unknown));
    }

    #[test]
    fn the_longest_command_wins_and_nests_its_path() {
        let found = parse("/permissions user get user:<@5> channel:<#9>", &entries()).unwrap();
        assert_eq!(found.entry.display(), "permissions user get");
        let body = interaction_body(
            &found,
            Some(Id::new(7)),
            Id::new(8),
            Some("s".to_string()),
            "n".to_string(),
        );
        assert_eq!(
            body["data"]["options"],
            json!([{ "type": 2, "name": "user", "options": [{ "type": 1, "name": "get", "options": [{ "type": 6, "name": "user", "value": "5" }, { "type": 7, "name": "channel", "value": "9" }] }] }])
        );
        assert_eq!(body["guild_id"], json!("7"));
        assert_eq!(body["channel_id"], json!("8"));
        assert_eq!(body["data"]["name"], json!("permissions"));
    }
}
