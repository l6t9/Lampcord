use std::str::FromStr;

use requiem_tsx::{FromValue, StateQueries, Value};
use serde::Serialize;
use specta::datatype::{DataType, Reference};
use specta::{Type, Types};
use specta_typescript::Branded;

use crate::models::id::marker::{
    ApplicationMarker, AttachmentMarker, ChannelMarker, CommandMarker, EmojiMarker, GuildMarker,
    MessageMarker, RoleMarker, UserMarker, WebhookMarker,
};
use crate::models::{Channel, Id, Message, Permissions, User};
use crate::state::{AppState, ChannelStore, MessageStore, UserStore};
use crate::ui::threads;
use crate::utils::time;

pub trait Brand {
    const NAME: &'static str;
}

macro_rules! brands {
    ($($marker:ident => $name:literal),+ $(,)?) => {
        $(impl Brand for $marker {
            const NAME: &'static str = $name;
        })+

        pub const ID_BRANDS: &[&str] = &[$($name),+];
    };
}

brands! {
    ApplicationMarker => "ApplicationId",
    AttachmentMarker => "AttachmentId",
    ChannelMarker => "ChannelId",
    CommandMarker => "CommandId",
    EmojiMarker => "EmojiId",
    GuildMarker => "GuildId",
    MessageMarker => "MessageId",
    RoleMarker => "RoleId",
    UserMarker => "UserId",
    WebhookMarker => "WebhookId",
}

impl<T: Brand> Type for Id<T> {
    fn definition(types: &mut Types) -> DataType {
        let text = <String as Type>::definition(types);
        DataType::Reference(Reference::opaque(Branded::new(T::NAME, text)))
    }
}

impl<T: Brand> FromValue for Id<T> {
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Str(text) => Id::from_str(text).ok(),
            _ => None,
        }
    }

    fn ts() -> String {
        T::NAME.to_string()
    }
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UserView {
    pub id: Id<UserMarker>,
    pub username: String,
    pub display_name: String,
    pub avatar: Option<String>,
    pub bot: bool,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChannelView {
    pub id: Id<ChannelMarker>,
    pub guild_id: Option<Id<GuildMarker>>,
    pub parent_id: Option<Id<ChannelMarker>>,
    pub name: Option<String>,
    pub kind: u8,
    pub is_thread: bool,
    pub archived: bool,
    pub owner_id: Option<Id<UserMarker>>,
    pub last_message_id: Option<Id<MessageMarker>>,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MessageView {
    pub id: Id<MessageMarker>,
    pub channel_id: Id<ChannelMarker>,
    pub author_id: Id<UserMarker>,
    pub content: String,
    pub content_preview: String,
    pub timestamp: String,
    pub edited: bool,
}

impl From<&User> for UserView {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            display_name: user.display_name(None),
            avatar: user.avatar.clone(),
            bot: user.bot,
        }
    }
}

impl From<&Channel> for ChannelView {
    fn from(channel: &Channel) -> Self {
        Self {
            id: channel.id,
            guild_id: channel.guild_id,
            parent_id: channel.parent_id,
            name: channel.name.clone(),
            kind: channel.kind,
            is_thread: channel.is_thread(),
            archived: channel.is_archived_thread(),
            owner_id: channel.owner_id,
            last_message_id: channel.last_message_id,
        }
    }
}

impl From<&Message> for MessageView {
    fn from(message: &Message) -> Self {
        Self {
            id: message.id,
            channel_id: message.channel_id,
            author_id: message.author.id,
            content: message.content.clone(),
            content_preview: message.content.split_whitespace().collect::<Vec<_>>().join(" "),
            timestamp: message.timestamp.clone(),
            edited: message.edited_timestamp.is_some(),
        }
    }
}

pub fn id_arg<T>(args: &[Value], index: usize) -> Option<Id<T>> {
    match args.get(index) {
        Some(Value::Str(text)) => Id::from_str(text).ok(),
        _ => None,
    }
}

fn channel_watched(channels: &ChannelStore, id: Id<ChannelMarker>) -> Option<Channel> {
    let found = channels.by_guild.with_untracked(|all| {
        all.iter().find_map(|(guild_id, list)| {
            list.iter()
                .find(|channel| channel.id == id)
                .map(|channel| (*guild_id, channel.clone()))
        })
    });
    match found {
        Some((guild_id, channel)) => {
            channels.by_guild.watch(&guild_id);
            Some(channel)
        }
        None => {
            channels.by_guild.with(|_| ());
            None
        }
    }
}

pub fn queries(state: &AppState) -> StateQueries {
    let permission_names: Vec<(String, Permissions)> = Permissions::named();
    let permission_union = permission_names
        .iter()
        .map(|(name, _)| format!("\"{name}\""))
        .collect::<Vec<_>>()
        .join(" | ");
    let (permissions_state, channels) = (state.clone(), state.channels.clone());
    let read_states = state.read_states.clone();
    queries_for(&state.users, &state.channels, &state.messages)
        .scalar(
            "permissions.can",
            &[("channelId", "ChannelId"), ("permission", &permission_union)],
            "boolean",
            move |args| {
                let (Some(id), Some(Value::Str(name))) =
                    (id_arg::<ChannelMarker>(args, 0), args.get(1))
                else {
                    return false;
                };
                let Some((_, permission)) = permission_names.iter().find(|(known, _)| known == &**name)
                else {
                    return false;
                };
                channel_watched(&channels, id)
                    .is_some_and(|channel| permissions_state.can(&channel, *permission))
            },
        )
        .scalar(
            "readStates.lastMessageId",
            &[("channelId", "ChannelId")],
            "MessageId | null",
            move |args| {
                id_arg::<ChannelMarker>(args, 0)
                    .and_then(|id| read_states.last_message_id_watched(id))
                    .map(Id::<MessageMarker>::new)
            },
        )
        .one("messages.latestIn", &[("channelId", "ChannelId")], {
            let messages = state.messages.clone();
            move |args| {
                let id = id_arg::<ChannelMarker>(args, 0)?;
                messages.in_channel(id).last().map(MessageView::from)
            }
        })
        .scalar("threads.lastActivity", &[("channelId", "ChannelId")], "number", {
            let (state, channels) = (state.clone(), state.channels.clone());
            move |args| {
                let Some(id) = id_arg::<ChannelMarker>(args, 0) else {
                    return time::now_ms() as i64;
                };
                state.read_states.last_message_id_watched(id);
                match channel_watched(&channels, id) {
                    Some(thread) => threads::last_activity_ms(&state, &thread),
                    None => time::now_ms() as i64,
                }
            }
        })
        .scalar("threads.archivedActivity", &[("channelId", "ChannelId")], "number", {
            let (state, channels) = (state.clone(), state.channels.clone());
            move |args| {
                let Some(id) = id_arg::<ChannelMarker>(args, 0) else {
                    return time::now_ms() as i64;
                };
                state.read_states.last_message_id_watched(id);
                match channel_watched(&channels, id) {
                    Some(thread) => threads::archived_activity_ms(&state, &thread),
                    None => time::now_ms() as i64,
                }
            }
        })
        .scalar("format.ago", &[("unixMs", "number")], "string", |args| match args.first() {
            Some(Value::Num(ms)) => threads::ago(*ms as i64),
            _ => String::new(),
        })
}

pub fn queries_for(users: &UserStore, channels: &ChannelStore, messages: &MessageStore) -> StateQueries {
    let users = users.clone();
    let channels = channels.clone();
    let messages = messages.clone();
    ID_BRANDS
        .iter()
        .fold(StateQueries::new(), |queries, brand| queries.brand(brand))
        .one("users.get", &[("id", "UserId")], move |args| {
            let id = id_arg::<UserMarker>(args, 0)?;
            users.get_watched(id).as_ref().map(UserView::from)
        })
        .one("channels.get", &[("id", "ChannelId")], {
            let channels = channels.clone();
            move |args| {
                let id = id_arg::<ChannelMarker>(args, 0)?;
                channel_watched(&channels, id).as_ref().map(ChannelView::from)
            }
        })
        .list("channels.inGuild", &[("guildId", "GuildId")], {
            move |args| {
                let Some(id) = id_arg::<GuildMarker>(args, 0) else {
                    return Vec::new();
                };
                channels.by_guild.watch(&id);
                channels.by_guild.with_untracked(|all| {
                    all.get(&id)
                        .map(|list| list.iter().map(ChannelView::from).collect())
                        .unwrap_or_default()
                })
            }
        })
        .list("messages.inChannel", &[("channelId", "ChannelId")], move |args| {
            let Some(id) = id_arg::<ChannelMarker>(args, 0) else {
                return Vec::new();
            };
            messages.in_channel(id).iter().map(MessageView::from).collect()
        })
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::env;
    use std::fs;
    use std::rc::Rc;

    use crate::state::Action;
    use requiem_tsx::ui_actions;
    use requiem_ui::signal::{create_effect, flush};
    use requiem_ui::style::ComputedStyle;
    use requiem_ui::widget::WidgetTree;
    use requiem_ui::widget::primitives::BoxWidget;

    use super::*;

    fn user(id: u64, username: &str) -> User {
        User {
            id: Id::new(id),
            username: username.into(),
            discriminator: "0".into(),
            global_name: None,
            avatar: Some("hash".into()),
            email: None,
            phone: None,
            verified: None,
            mfa_enabled: None,
            bot: false,
            premium_type: None,
            primary_guild: None,
        }
    }

    #[test]
    fn a_user_change_reruns_only_the_readers_of_that_user() {
        let users = UserStore::new();
        let channels = ChannelStore::new(users.clone());
        let messages = MessageStore::new();
        users.observe(&user(1, "ana"));
        users.observe(&user(2, "bo"));
        let queries = queries_for(&users, &channels, &messages);

        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let reads = [Rc::new(Cell::new(0)), Rc::new(Cell::new(0))];
        let mut effects = Vec::new();
        for (index, id) in ["1", "2"].into_iter().enumerate() {
            let (queries, counter) = (queries.clone(), reads[index].clone());
            effects.push(create_effect(
                move |_| {
                    counter.set(counter.get() + 1);
                    queries.call("users.get", &[Value::str(id)]);
                },
                &mut tree,
            ));
        }
        assert_eq!([reads[0].get(), reads[1].get()], [1, 1]);

        users.observe(&user(1, "ana renamed"));
        flush(&mut tree);
        assert_eq!([reads[0].get(), reads[1].get()], [2, 1]);
    }

    #[test]
    fn a_channel_that_loads_later_wakes_the_reader_that_found_nothing() {
        let users = UserStore::new();
        let channels = ChannelStore::new(users.clone());
        let queries = queries_for(&users, &channels, &MessageStore::new());
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let seen: Rc<Cell<Option<bool>>> = Rc::new(Cell::new(None));
        let (reader, found) = (queries.clone(), seen.clone());
        let _effect = create_effect(
            move |_| {
                let value = reader.call("channels.get", &[Value::str("5")]);
                found.set(Some(!matches!(value, Some(Value::Null))));
            },
            &mut tree,
        );
        assert_eq!(seen.get(), Some(false));

        let channel: Channel = serde_json::from_value(serde_json::json!({
            "id": "5", "guild_id": "9", "type": 0, "name": "general", "position": 0
        }))
        .expect("channel");
        channels.upsert(channel, false);
        flush(&mut tree);
        assert_eq!(seen.get(), Some(true));
    }

    #[test]
    fn a_new_message_wakes_only_the_last_message_readers_of_its_channel() {
        let state = AppState::new();
        let queries = queries(&state);
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let reads = [Rc::new(Cell::new(0)), Rc::new(Cell::new(0))];
        let mut effects = Vec::new();
        for (index, channel) in ["10", "11"].into_iter().enumerate() {
            let (queries, counter) = (queries.clone(), reads[index].clone());
            effects.push(create_effect(
                move |_| {
                    counter.set(counter.get() + 1);
                    queries.call("readStates.lastMessageId", &[Value::str(channel)]);
                },
                &mut tree,
            ));
        }
        assert_eq!(queries.call("readStates.lastMessageId", &[Value::str("10")]), Some(Value::Null));

        let message: Message = serde_json::from_value(serde_json::json!({
            "id": "500",
            "channel_id": "10",
            "author": {"id": "1", "username": "ana", "discriminator": "0"},
            "content": "hi",
            "timestamp": "2026-10-27T00:00:00.000000+00:00"
        }))
        .expect("message");
        state.dispatch(&Action::MessageCreate(message));
        flush(&mut tree);

        assert_eq!([reads[0].get(), reads[1].get()], [2, 1]);
        assert_eq!(
            queries.call("readStates.lastMessageId", &[Value::str("10")]),
            Some(Value::str("500"))
        );
    }

    #[test]
    fn permissions_answer_false_for_an_unknown_channel_or_name() {
        let state = AppState::new();
        let queries = queries(&state);
        let ask = |channel: &str, permission: &str| {
            queries.call("permissions.can", &[Value::str(channel), Value::str(permission)])
        };
        assert_eq!(ask("77", "viewChannel"), Some(Value::Bool(false)));
        assert_eq!(ask("77", "notAPermission"), Some(Value::Bool(false)));
        let text = queries.declarations();
        assert!(text.contains("\"viewChannel\" |"), "{text}");
        assert!(text.contains("\"createPublicThreads\""), "{text}");
        assert!(
            text.contains("lastMessageId(channelId: Types.ChannelId): Types.MessageId | null;"),
            "{text}"
        );
    }

    #[test]
    fn the_declarations_can_be_written_out_for_tsc() {
        let Some(path) = env::var_os("REQUIEM_WRITE_STATE_TYPES") else {
            return;
        };
        fs::write(path, queries(&AppState::new()).declarations()).expect("write declarations");
    }

    #[test]
    fn views_serialize_with_camel_case_names() {
        let queries = queries_for(&UserStore::new(), &ChannelStore::new(UserStore::new()), &MessageStore::new());
        let text = queries.declarations();
        assert!(text.contains("displayName: string"), "{text}");
        assert!(text.contains("get(id: Types.UserId): Types.UserView | null;"), "{text}");
        assert!(text.contains("inChannel(channelId: Types.ChannelId): Types.MessageView[];"), "{text}");
        assert!(text.contains("inGuild(guildId: Types.GuildId): Types.ChannelView[];"), "{text}");
        assert!(!text.contains("import(\""), "{text}");
    }

    #[test]
    fn ids_are_branded_types_in_the_views_and_declared_once() {
        let text = queries_for(&UserStore::new(), &ChannelStore::new(UserStore::new()), &MessageStore::new())
            .declarations();
        assert!(
            text.contains("export type UserId = string & { readonly __brand: \"UserId\" };"),
            "{text}"
        );
        assert!(
            text.contains("export type GuildId = string & { readonly __brand: \"GuildId\" };"),
            "{text}"
        );
        assert!(text.contains("guildId: GuildId | null"), "{text}");
        assert!(text.contains("authorId: UserId"), "{text}");
        assert!(text.contains("lastMessageId: MessageId | null"), "{text}");
        assert!(text.contains("declare module \"@requiem/types\""), "{text}");
        assert!(text.contains("export type UserView = {"), "{text}");
        assert!(text.contains("export type UserView = Types.UserView;"), "{text}");
        assert!(text.contains("export type UserId = Types.UserId;"), "{text}");
    }

    ui_actions! {
        #[derive(Debug, PartialEq)]
        enum Sample {
            openThread(id: Id<ChannelMarker>, shift_key: bool) => OpenThread,
        }
    }

    #[test]
    fn actions_take_typed_ids() {
        use requiem_tsx::Actions;

        assert_eq!(
            Sample::specs()[0].signature(),
            "openThread(id: ChannelId, shiftKey: boolean): void"
        );
        assert_eq!(
            Sample::from_call("openThread", &[Value::str("42"), Value::Bool(true)]),
            Some(Sample::OpenThread {
                id: Id::new(42),
                shift_key: true
            })
        );
        assert_eq!(Sample::from_call("openThread", &[Value::str("abc"), Value::Bool(true)]), None);
    }
}
