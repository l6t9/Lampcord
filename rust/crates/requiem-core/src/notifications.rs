use crate::rest::endpoints::relationships;
use crate::ui::home;
use crate::ui;
use crate::utils::voice_settings;
use requiem_ui::widget::primitives;
use serde::Deserialize;

use crate::gateway::events::GatewayEvent;
use crate::models::id::marker::{
    ChannelMarker, GuildMarker, MessageMarker, RoleMarker, UserMarker,
};
use crate::models::{Channel, Id, Message};
use crate::presence::OwnStatus;
use crate::sounds::Sound;
use crate::state::AppState;
use crate::utils::notification_settings;

const SUPPRESS_NOTIFICATIONS: u64 = 1 << 12;

const THREAD_STARTER_MESSAGE: u8 = 21;

const ALL_MESSAGES: u8 = 0;
const NO_MESSAGES: u8 = 2;

#[derive(Clone, Copy, Debug)]
pub struct Context {
    pub window_focused: bool,
    pub selected_channel: Option<Id<ChannelMarker>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Notification {
    pub channel_id: Id<ChannelMarker>,
    pub guild_id: Option<Id<GuildMarker>>,
    pub message_id: Id<MessageMarker>,
    pub author_id: Id<UserMarker>,
    pub title: String,
    pub body: String,

    pub icon_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    Nothing,

    Sound(Sound),

    Show {
        notification: Notification,
        sound: Option<Sound>,
    },
}

#[derive(Deserialize, Default)]
struct Extras {
    #[serde(default)]
    sticker_items: Vec<StickerItem>,
    #[serde(default)]
    poll: Option<Poll>,
    #[serde(default)]
    member: Option<MemberNick>,
    #[serde(default)]
    message_snapshots: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct StickerItem {
    name: String,
}

#[derive(Deserialize)]
struct Poll {
    question: PollQuestion,
}

#[derive(Deserialize)]
struct PollQuestion {
    #[serde(default)]
    text: String,
}

#[derive(Deserialize)]
struct MemberNick {
    #[serde(default)]
    nick: Option<String>,
}

struct Place {
    guild_id: Option<Id<GuildMarker>>,
    channel: Channel,
    private: bool,
}

fn place(state: &AppState, channel_id: Id<ChannelMarker>) -> Option<Place> {
    if let Some(dm) = state.dms.find(channel_id) {
        return Some(Place {
            guild_id: None,
            channel: dm.as_channel(),
            private: true,
        });
    }
    let (guild_id, channel) = ui::find_channel(state, channel_id)?;
    Some(Place {
        guild_id: Some(guild_id),
        channel,
        private: false,
    })
}

fn guild_default_level(state: &AppState, guild_id: Id<GuildMarker>) -> u8 {
    state.guilds.list.with_untracked(|guilds| {
        guilds
            .iter()
            .find(|g| g.id == guild_id)
            .map_or(ALL_MESSAGES, |g| g.default_message_notifications)
    })
}

fn eligible(state: &AppState, message: &Message, place: &Place) -> bool {
    let Some(me) = state
        .users
        .current
        .with_untracked(|u| u.as_ref().map(|u| u.id))
    else {
        return false;
    };
    if message.author.id == me {
        return false;
    }
    if state.relationships.kind_of(message.author.id)
        == Some(relationships::BLOCKED)
    {
        return false;
    }
    if state.presence.status.peek() == OwnStatus::Dnd {
        return false;
    }

    if let Some(guild_id) = place.guild_id {
        let level = state.guild_settings.resolved_message_notifications(
            Some(guild_id),
            place.channel.id,
            place.channel.parent_id,
            guild_default_level(state, guild_id),
        );
        if level == NO_MESSAGES {
            return false;
        }
    }
    true
}

fn should_notify(
    state: &AppState,
    message: &Message,
    place: &Place,
    context: &Context,
    include_selected: bool,
) -> bool {
    if message.flags & SUPPRESS_NOTIFICATIONS != 0 || !eligible(state, message, place) {
        return false;
    }
    if !include_selected && context.selected_channel == Some(place.channel.id) {
        return false;
    }
    let channel = &place.channel;
    if place.private {
        if state
            .guild_settings
            .is_channel_muted(None, channel.id, None)
        {
            return false;
        }
        let level = state.guild_settings.resolved_message_notifications(
            None,
            channel.id,
            None,
            ALL_MESSAGES,
        );
        return level != NO_MESSAGES
            && (level == ALL_MESSAGES || state.read_states.pings_me(message, None));
    }
    let guild_id = place.guild_id;
    let level = state.guild_settings.resolved_message_notifications(
        guild_id,
        channel.id,
        channel.parent_id,
        guild_id.map_or(ALL_MESSAGES, |g| guild_default_level(state, g)),
    );
    let muted = state
        .guild_settings
        .is_channel_muted(guild_id, channel.id, channel.parent_id);

    let all_applies = !channel.is_thread() || context.selected_channel == Some(channel.id);
    if level == ALL_MESSAGES && !muted && all_applies {
        return true;
    }
    state.read_states.pings_me(message, guild_id)
}

fn in_selected_channel(
    state: &AppState,
    message: &Message,
    place: &Place,
    context: &Context,
) -> bool {
    context.selected_channel == Some(place.channel.id) && eligible(state, message, place)
}

pub fn on_message_create(
    state: &AppState,
    event: &GatewayEvent,
    context: &Context,
    avatar_url: impl Fn(&Message) -> Option<String>,
) -> Outcome {
    let Ok(message) = event.parse::<Message>() else {
        return Outcome::Nothing;
    };

    let place_id = if message.kind == THREAD_STARTER_MESSAGE {
        place(state, message.channel_id)
            .and_then(|p| p.channel.parent_id)
            .unwrap_or(message.channel_id)
    } else {
        message.channel_id
    };
    let Some(place) = place(state, place_id) else {
        return Outcome::Nothing;
    };
    let settings = notification_settings::get();

    let notify = should_notify(state, &message, &place, context, !context.window_focused);
    let selected =
        settings.notify_selected_channel && in_selected_channel(state, &message, &place, context);
    if !notify && !selected {
        return Outcome::Nothing;
    }
    let sound_on = voice_settings::get().is_sound_enabled(Sound::Message.id());
    if selected && (context.window_focused || !notify) {
        return if sound_on {
            Outcome::Sound(Sound::Message3)
        } else {
            Outcome::Nothing
        };
    }
    if !settings.desktop_notifications {
        return if sound_on {
            Outcome::Sound(Sound::Message)
        } else {
            Outcome::Nothing
        };
    }
    let extras: Extras = event.parse().unwrap_or_default();
    let (title, body) = text(state, &message, &place, &extras);
    let notification = Notification {
        channel_id: message.channel_id,
        guild_id: place.guild_id,
        message_id: message.id,
        author_id: message.author.id,
        title,
        body,
        icon_url: avatar_url(&message),
    };
    Outcome::Show {
        notification,
        sound: sound_on.then_some(Sound::Message),
    }
}

fn channel_label(channel: &Channel, with_prefix: bool) -> String {
    let name = channel.name.clone().unwrap_or_default();
    if with_prefix && !matches!(channel.kind, 2 | 4 | 13) {
        format!("#{name}")
    } else {
        name
    }
}

fn text(state: &AppState, message: &Message, place: &Place, extras: &Extras) -> (String, String) {
    let author_name = message
        .author
        .display_name(extras.member.as_ref().and_then(|m| m.nick.as_deref()));
    let channel = &place.channel;
    let title = if place.private {
        match state.dms.find(channel.id).filter(|dm| dm.is_group()) {
            Some(dm) => format!("{author_name} ({})", home::dm_title(state, &dm)),
            None => author_name,
        }
    } else {
        let parent = channel
            .parent_id
            .and_then(|id| ui::find_channel(state, id))
            .map(|(_, c)| c);
        match parent {
            Some(parent) => format!(
                "{author_name} ({}, {})",
                channel_label(channel, true),
                channel_label(&parent, false)
            ),
            None => format!("{author_name} ({})", channel_label(channel, true)),
        }
    };

    let body = if !extras.message_snapshots.is_empty() {
        requiem_i18n::t!("notifications-forwarded").to_string()
    } else if let Some(sticker) = extras.sticker_items.first() {
        requiem_i18n::t!("notifications-sticker", stickerName = sticker.name.clone()).to_string()
    } else if let Some(poll) = &extras.poll {
        requiem_i18n::t!("notifications-poll", question = poll.question.text.clone()).to_string()
    } else {
        let content = unparse(state, &message.content, place.guild_id);
        if content.is_empty() {
            fallback(message)
        } else {
            content
        }
    };
    (title, body)
}

fn fallback(message: &Message) -> String {
    if let Some(embed) = message.embeds.first() {
        match (&embed.title, &embed.description) {
            (Some(title), Some(description)) => return format!("{title} {description}"),
            (None, Some(description)) => return description.clone(),
            (Some(title), None) => return title.clone(),
            (None, None) => {}
        }
        if let Some(field) = embed.fields.as_ref().and_then(|f| f.first()) {
            return format!("{} {}", field.name, field.value);
        }
    }
    if let Some(attachment) = message.attachments.first() {
        return requiem_i18n::t!(
            "notifications-uploaded",
            filename = attachment.filename.clone()
        )
        .to_string();
    }
    String::new()
}

/// The message as plain text for a notification: markdown removed, and mentions, channels
/// and custom emoji written out as `@Name`, `#channel` and `:name:`.
pub fn unparse(state: &AppState, content: &str, guild_id: Option<Id<GuildMarker>>) -> String {
    // Tokens are swapped for placeholders (private-use marks around an index) so the
    // markdown pass can't read a `_` in `@some_name` as italics, then written back.
    let mut shielded = String::with_capacity(content.len());
    let mut tokens: Vec<String> = Vec::new();
    let mut rest = content;
    while let Some(start) = rest.find('<') {
        shielded.push_str(&rest[..start]);
        let tail = &rest[start..];
        let Some(end) = tail.find('>') else {
            shielded.push_str(tail);
            rest = "";
            break;
        };
        let inner = &tail[1..end];
        let text = match replacement(state, inner, guild_id) {
            Some(text) => text,
            // `<https://...>` only hides the embed; the address is what reads.
            None if inner.starts_with("http://") || inner.starts_with("https://") => {
                inner.to_string()
            }
            None => tail[..=end].to_string(),
        };
        shielded.push_str(&format!("{PLACEHOLDER_OPEN}{}{PLACEHOLDER_CLOSE}", tokens.len()));
        tokens.push(text);
        rest = &tail[end + 1..];
    }
    shielded.push_str(rest);

    let mut out = primitives::markdown_plain_text(&shielded);
    for (index, text) in tokens.iter().enumerate() {
        out = out.replace(&format!("{PLACEHOLDER_OPEN}{index}{PLACEHOLDER_CLOSE}"), text);
    }
    out
}

const PLACEHOLDER_OPEN: char = '\u{E000}';
const PLACEHOLDER_CLOSE: char = '\u{E001}';

fn replacement(state: &AppState, inner: &str, guild_id: Option<Id<GuildMarker>>) -> Option<String> {
    let id = |s: &str| s.parse::<u64>().ok().filter(|id| *id != 0);
    if let Some(user) = inner
        .strip_prefix("@!")
        .or_else(|| inner.strip_prefix('@'))
        .filter(|s| !s.starts_with('&'))
    {
        let user_id: Id<UserMarker> = Id::new(id(user)?);
        let nick = guild_id.and_then(|g| {
            state.members.by_guild.with_untracked(|all| {
                all.get(&g)
                    .and_then(|m| m.get(&user_id))
                    .and_then(|m| m.nick.clone())
            })
        });
        let name = nick.or_else(|| {
            state
                .users
                .get(user_id)
                .map(|u| state.users.display_name(&u))
        })?;
        return Some(format!("@{name}"));
    }
    if let Some(role) = inner.strip_prefix("@&") {
        let role_id: Id<RoleMarker> = Id::new(id(role)?);
        let name = state
            .permissions
            .guild_roles(guild_id?)
            .into_iter()
            .find(|r| r.id == role_id)?
            .name;
        return Some(format!("@{name}"));
    }
    if let Some(channel) = inner.strip_prefix('#') {
        let (_, channel) = ui::find_channel(state, Id::new(id(channel)?))?;
        return Some(format!("#{}", channel.name.unwrap_or_default()));
    }
    let emoji = inner
        .strip_prefix("a:")
        .or_else(|| inner.strip_prefix(':'))?;
    let (name, emoji_id) = emoji.split_once(':')?;
    id(emoji_id)?;
    Some(format!(":{name}:"))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn user(id: u64, name: &str) -> serde_json::Value {
        json!({ "id": id.to_string(), "username": name, "discriminator": "0", "global_name": null, "avatar": null })
    }

    fn state() -> AppState {
        let state = AppState::new();
        state.apply_event(&GatewayEvent::new(
            "READY",
            json!({
                "user": user(1, "me"),
                "guilds": [{ "id": "40", "properties": { "name": "G", "owner_id": "1", "default_message_notifications": 1 },
                    "roles": [{ "id": "40", "name": "@everyone", "permissions": "1024" }],
                    "channels": [{ "id": "41", "type": 4, "name": "Chat" }, { "id": "42", "type": 0, "name": "general", "parent_id": "41" }] }],
                "users": [user(2, "pal")],
                "private_channels": [{ "id": "50", "type": 1, "recipient_ids": ["2"] }],
            }),
        ));
        state
    }

    fn message(channel: u64, author: u64, content: &str) -> GatewayEvent {
        GatewayEvent::new(
            "MESSAGE_CREATE",
            json!({ "id": "900", "channel_id": channel.to_string(), "author": user(author, if author == 2 { "pal" } else { "someone" }), "content": content, "timestamp": "2026-10-16T10:00:00+00:00", "mentions": [], "mention_roles": [], "attachments": [], "embeds": [] }),
        )
    }

    const AWAY: Context = Context {
        window_focused: false,
        selected_channel: None,
    };

    fn shown(outcome: Outcome) -> Notification {
        match outcome {
            Outcome::Show { notification, .. } => notification,
            other => panic!("expected a notification, got {other:?}"),
        }
    }

    #[test]
    fn a_dm_notifies_with_the_authors_name() {
        let state = state();
        let n = shown(on_message_create(
            &state,
            &message(50, 2, "hi there"),
            &AWAY,
            |_| None,
        ));
        assert_eq!(n.title, "pal");
        assert_eq!(n.body, "hi there");
    }

    #[test]
    fn a_server_set_to_mentions_only_needs_a_mention_and_titles_with_channel_and_category() {
        let state = state();
        assert_eq!(
            on_message_create(&state, &message(42, 3, "hello"), &AWAY, |_| None),
            Outcome::Nothing
        );
        let mut mention = message(42, 3, "hey <@1>").value().clone();
        mention["mentions"] = json!([user(1, "me")]);
        let n = shown(on_message_create(
            &state,
            &GatewayEvent::new("MESSAGE_CREATE", mention),
            &AWAY,
            |_| None,
        ));
        assert_eq!(n.title, "someone (#general, Chat)");
        assert_eq!(n.body, "hey @me");
    }

    #[test]
    fn your_own_and_the_open_channel_while_focused_dont_notify() {
        let state = state();
        assert_eq!(
            on_message_create(&state, &message(50, 1, "mine"), &AWAY, |_| None),
            Outcome::Nothing
        );
        let reading = Context {
            window_focused: true,
            selected_channel: Some(Id::new(50)),
        };
        assert_eq!(
            on_message_create(&state, &message(50, 2, "hi"), &reading, |_| None),
            Outcome::Nothing
        );

        let unfocused = Context {
            window_focused: false,
            selected_channel: Some(Id::new(50)),
        };
        assert!(matches!(
            on_message_create(&state, &message(50, 2, "hi"), &unfocused, |_| None),
            Outcome::Show { .. }
        ));
    }

    #[test]
    fn silent_messages_and_dnd_dont_notify() {
        let state = state();
        let mut silent = message(50, 2, "shh").value().clone();
        silent["flags"] = json!(SUPPRESS_NOTIFICATIONS);
        assert_eq!(
            on_message_create(
                &state,
                &GatewayEvent::new("MESSAGE_CREATE", silent),
                &AWAY,
                |_| None
            ),
            Outcome::Nothing
        );
        state.presence.status.set(OwnStatus::Dnd);
        assert_eq!(
            on_message_create(&state, &message(50, 2, "hi"), &AWAY, |_| None),
            Outcome::Nothing
        );
    }

    #[test]
    fn empty_messages_fall_back_to_the_attachment() {
        let state = state();
        let mut upload = message(50, 2, "").value().clone();
        upload["attachments"] = json!([{ "id": "1", "filename": "cat.png", "size": 1, "url": "https://x/cat.png", "proxy_url": "https://x/cat.png" }]);
        let n = shown(on_message_create(
            &state,
            &GatewayEvent::new("MESSAGE_CREATE", upload),
            &AWAY,
            |_| None,
        ));
        assert_eq!(n.body, "Uploaded cat.png");
    }

    #[test]
    fn markup_reads_as_text() {
        let state = state();
        assert_eq!(
            unparse(&state, "look <#42> <:wave:123> <@2>", Some(Id::new(40))),
            "look #general :wave: @pal"
        );
        assert_eq!(unparse(&state, "a < b > c", None), "a < b > c");
    }

    #[test]
    fn markdown_is_stripped_but_mentions_still_resolve() {
        let state = state();
        let plain = |text: &str| unparse(&state, text, Some(Id::new(40)));
        assert_eq!(
            plain("**bold** *it* __under__ ~~gone~~ `code` ||secret||"),
            "bold it under gone code secret"
        );
        assert_eq!(
            plain("see [the docs](https://example.com/a_b_c) now"),
            "see the docs now"
        );
        assert_eq!(plain("hey <@2>, **read** <#42>"), "hey @pal, read #general");
        assert_eq!(plain("# Title\n-# small\n> quoted"), "Title\nsmall\nquoted");
        assert_eq!(plain("a \\*star\\* b"), "a *star* b");
        assert_eq!(plain("<https://example.com/x>"), "https://example.com/x");
        assert_eq!(plain("plain https://example.com/a_b_c link"), "plain https://example.com/a_b_c link");
    }
}
