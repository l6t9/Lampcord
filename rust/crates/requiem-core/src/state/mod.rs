pub mod action;
mod channel_store;
mod connection_store;
mod dm_store;
mod draft_store;
mod emoji_store;
mod guild_folder_store;
mod guild_settings_store;
mod guild_store;
pub mod member_list;
mod member_store;
mod message_store;
mod nav_store;
mod permission_store;
mod pin_store;
mod plugin_store;
mod presence_store;
mod read_state_store;
mod relationship_store;
mod typing_store;
mod self_presence_store;
mod user_store;
pub mod voice_state_store;
pub mod wire;

pub use action::{Action, GuildData, MemberRoles};
pub use channel_store::ChannelStore;
pub use connection_store::ConnectionStore;
pub use dm_store::{DmChannel, DmStore};
pub use draft_store::DraftStore;
pub use emoji_store::EmojiStore;
pub use guild_folder_store::{GuildFolder, GuildFolderStore, RailEntry};
pub use guild_settings_store::{ChannelOverride, GuildSettings, GuildSettingsStore, Mute};
pub use guild_store::GuildStore;
pub use member_store::MemberStore;
pub use message_store::MessageStore;
pub use nav_store::{FriendsTab, NavEvent, NavStore};
pub use permission_store::PermissionStore;
pub use pin_store::{ChannelPins, Pin, PinStore};
pub use plugin_store::{
    ComposerRestore, MODAL_CONTEXT as PLUGIN_MODAL_CONTEXT, PluginStore, PluginSurface,
    refresh_guild as refresh_plugin_guild,
};
pub use presence_store::{PresenceStore, UserPresence};
pub use typing_store::TypingStore;
pub use read_state_store::{AckRequest, ChannelBadge, GuildBadge, NewMarker, ReadStateStore};
pub use relationship_store::{Relationship, RelationshipStore};
pub use self_presence_store::SelfPresenceStore;
pub use user_store::UserStore;
pub use voice_state_store::{ChannelVoiceInfo, Rtc, RtcState, VoiceState, VoiceStateStore};
use crate::gateway::member_requests;
use crate::models::Permissions;
use crate::models::permissions;
use crate::models;
use crate::ui::devtools;
use std::iter;

use crate::gateway::events::GatewayEvent;
use crate::models::id::marker::ChannelMarker;
use crate::models::{Id, Message};

pub trait Store {
    fn handle(&self, action: &Action);

    fn reset(&self);
}

#[derive(Clone)]
pub struct AppState {
    pub users: UserStore,
    pub guilds: GuildStore,
    pub folders: GuildFolderStore,
    pub emojis: EmojiStore,
    pub channels: ChannelStore,
    pub permissions: PermissionStore,
    pub messages: MessageStore,
    pub members: MemberStore,
    pub nav: NavStore,
    pub relationships: RelationshipStore,
    pub dms: DmStore,
    pub user_presence: PresenceStore,
    pub presence: SelfPresenceStore,
    pub connection: ConnectionStore,
    pub guild_settings: GuildSettingsStore,
    pub read_states: ReadStateStore,
    pub plugins: PluginStore,
    pub drafts: DraftStore,
    pub pins: PinStore,
    pub voice: VoiceStateStore,
    pub typing: TypingStore,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let users = UserStore::new();
        let guilds = GuildStore::new();
        let channels = ChannelStore::new(users.clone());
        let permissions = PermissionStore::new();
        let messages = MessageStore::new();
        let relationships = RelationshipStore::new();
        let dms = DmStore::new();
        let guild_settings = GuildSettingsStore::new();
        let read_states = ReadStateStore::new(
            users.clone(),
            permissions.clone(),
            guild_settings.clone(),
            channels.clone(),
            dms.clone(),
            guilds.clone(),
            messages.clone(),
            relationships.clone(),
        );
        Self {
            users,
            guilds,
            folders: GuildFolderStore::new(),
            emojis: EmojiStore::new(),
            channels,
            permissions,
            messages,
            members: MemberStore::new(),
            nav: NavStore::new(),
            relationships,
            dms,
            user_presence: PresenceStore::new(),
            presence: SelfPresenceStore::new(),
            connection: ConnectionStore::new(),
            guild_settings,
            read_states,
            plugins: PluginStore::new(),
            drafts: DraftStore::new(),
            pins: PinStore::new(),
            voice: VoiceStateStore::new(),
            typing: TypingStore::new(),
        }
    }

    fn stores(&self) -> [&dyn Store; 21] {
        [
            &self.users,
            &self.guilds,
            &self.folders,
            &self.emojis,
            &self.channels,
            &self.permissions,
            &self.messages,
            &self.members,
            &self.nav,
            &self.relationships,
            &self.dms,
            &self.user_presence,
            &self.presence,
            &self.guild_settings,
            &self.read_states,
            &self.plugins,
            &self.drafts,
            &self.pins,
            &self.connection,
            &self.voice,
            &self.typing,
        ]
    }

    pub fn reset(&self) {
        self.stores().iter().for_each(|store| store.reset());
    }

    fn request_unknown_members(&self, channel_id: Id<ChannelMarker>, messages: &[Message]) {
        let Some(guild_id) = self.channels.by_guild.with_untracked(|by_guild| {
            by_guild
                .iter()
                .find(|(_, channels)| channels.iter().any(|c| c.id == channel_id))
                .map(|(guild, _)| *guild)
        }) else {
            return;
        };
        let users = messages
            .iter()
            .flat_map(|m| iter::once(m.author.id).chain(m.mentions.iter().map(|u| u.id)));
        member_requests::request(guild_id.get(), users.map(|u| u.get()), |user| {
            self.members.is_member(guild_id, Id::new(user))
        });
    }

    pub fn set_history(&self, channel_id: Id<ChannelMarker>, messages: Vec<Message>) {
        self.users.observe_messages(&messages);
        self.request_unknown_members(channel_id, &messages);
        self.messages.set_history(channel_id, messages);
        self.read_states.on_history(channel_id);
    }

    /// The history fetched when a channel is first opened. A jump can land its own window
    /// first; overwriting that with the latest page would pull the jump target out from
    /// under the highlight, so the late page is dropped.
    pub fn set_initial_history(&self, channel_id: Id<ChannelMarker>, messages: Vec<Message>) {
        if self.messages.is_detached(channel_id) {
            return;
        }
        self.set_history(channel_id, messages);
    }

    pub fn replace_window(&self, channel_id: Id<ChannelMarker>, page: Vec<Message>) {
        self.users.observe_messages(&page);
        self.request_unknown_members(channel_id, &page);
        self.messages.replace_window(channel_id, page);
        self.read_states.on_history(channel_id);
    }

    pub fn prepend_history(&self, channel_id: Id<ChannelMarker>, page: Vec<Message>) {
        self.users.observe_messages(&page);
        self.request_unknown_members(channel_id, &page);
        self.messages.prepend_history(channel_id, page);
        self.read_states.on_history(channel_id);
    }

    pub fn insert_message(&self, message: Message) {
        self.users.observe_messages([&message]);
        self.messages.insert(message);
    }

    pub fn channel_permissions(
        &self,
        channel: &models::Channel,
    ) -> Permissions {
        let Some(guild_id) = channel.guild_id else {
            return Permissions::ALL;
        };
        let owner = self.guilds.owner_of(guild_id);
        if !permissions::is_thread_kind(channel.kind) {
            return self
                .permissions
                .channel_permissions_for(guild_id, owner, channel);
        }
        let parent = channel.parent_id.and_then(|parent_id| {
            self.channels.by_guild.with_untracked(|all| {
                all.get(&guild_id)
                    .and_then(|list| list.iter().find(|c| c.id == parent_id).cloned())
            })
        });
        self.permissions
            .channel_permissions_in(guild_id, owner, channel, parent.as_ref())
    }

    pub fn can(
        &self,
        channel: &models::Channel,
        permission: Permissions,
    ) -> bool {
        self.channel_permissions(channel).contains(permission)
    }

    pub fn dispatch(&self, action: &Action) {
        self.stores().iter().for_each(|store| store.handle(action));
    }

    pub fn apply_event(&self, event: &GatewayEvent) {
        let action = Action::from_gateway(event);
        devtools::record_dispatch(event, action.is_some());
        if let Some(action) = action {
            self.dispatch(&action);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::Permissions as P;
    use requiem_ui::widget::primitives::PresenceStatus;
    use serde_json::json;

    use super::*;
    use crate::models::Id;
    use crate::models::Message;
    use crate::models::id::marker::{ChannelMarker, GuildMarker};
    use crate::state::message_store::TRUNCATED_VIEW_SIZE;

    fn edited(event: GatewayEvent, edit: impl FnOnce(&mut serde_json::Value)) -> GatewayEvent {
        let mut value = event.value().clone();
        edit(&mut value);
        GatewayEvent::new(event.name.clone(), value)
    }

    fn event(name: &str, data: serde_json::Value) -> GatewayEvent {
        GatewayEvent::new(name, data)
    }

    fn user(id: u64, name: &str) -> serde_json::Value {
        json!({ "id": id.to_string(), "username": name, "discriminator": "0", "global_name": null, "avatar": null })
    }

    fn message(id: u64, channel: u64, content: &str) -> serde_json::Value {
        json!({ "id": id.to_string(), "channel_id": channel.to_string(), "author": user(1, "me"), "content": content, "timestamp": "2026-01-01T00:00:00.000Z" })
    }

    fn ready() -> GatewayEvent {
        event(
            "READY",
            json!({
                "user": user(1, "me"),
                "auth_session_id_hash": "abc",
                "guilds": [
                    { "id": "10", "properties": { "name": "Guild", "icon": null, "owner_id": "1" }, "channels": [{ "id": "11", "type": 0, "name": "general", "position": 0 }] },
                    { "id": "20", "unavailable": true },
                ],
            }),
        )
    }

    fn reactions_of(state: &AppState, channel: u64, message: u64) -> Vec<(String, u32, bool)> {
        state.messages.by_channel.with_untracked(|by_channel| {
            by_channel[&Id::<ChannelMarker>::new(channel)]
                .iter()
                .find(|m| m.id.get() == message)
                .unwrap()
                .reactions
                .iter()
                .map(|r| (r.emoji.name.clone().unwrap_or_default(), r.count, r.me))
                .collect()
        })
    }

    fn reaction_event(name: &str, user_id: u64, glyph: &str) -> GatewayEvent {
        event(
            name,
            json!({ "channel_id": "11", "message_id": "100", "user_id": user_id.to_string(), "emoji": { "id": null, "name": glyph } }),
        )
    }

    #[test]
    fn masked_channels_nested_without_guild_id_stay_hidden() {
        let state = AppState::new();
        state.apply_event(&ready());
        let hidden = |id: &str, kind: u8, parent: Option<&str>| json!({ "id": id, "type": kind, "name": "___hidden___", "flags": 131072, "parent_id": parent, "position": 0, "permission_overwrites": [{ "id": "30", "type": 0, "allow": "0", "deny": "1024" }] });
        state.apply_event(&event(
            "GUILD_CREATE",
            json!({
                "id": "30",
                "properties": { "name": "Real", "icon": null, "owner_id": "99" },
                "roles": [{ "id": "30", "name": "@everyone", "permissions": "1024" }],
                "channels": [
                    { "id": "31", "type": 0, "name": "general", "position": 0 },
                    hidden("1087502660502028391", 0, Some("1216097816833687632")),
                    hidden("1542824587950297139", 0, Some("1216097816833687632")),
                    hidden("1216097816833687632", 4, None),
                ],
            }),
        ));
        let channels = state
            .channels
            .by_guild
            .with_untracked(|all| all[&Id::<GuildMarker>::new(30)].clone());
        assert!(
            channels.iter().all(|c| c.guild_id == Some(Id::new(30))),
            "every nested channel knows its guild"
        );
        let visible: Vec<u64> = channels
            .iter()
            .filter(|c| state.can(c, Permissions::VIEW_CHANNEL))
            .map(|c| c.id.get())
            .collect();
        assert_eq!(
            visible,
            [31],
            "only #general; the masked channels and their category are hidden"
        );
    }

    fn stored_channel(state: &AppState, guild: u64, id: u64) -> models::Channel {
        state.channels.by_guild.with_untracked(|all| {
            all[&Id::<GuildMarker>::new(guild)]
                .iter()
                .find(|c| c.id.get() == id)
                .cloned()
                .expect("stored channel")
        })
    }

    fn permission_guild(everyone: u64, channels: serde_json::Value) -> GatewayEvent {
        event(
            "GUILD_CREATE",
            json!({ "id": "30", "properties": { "name": "Perms", "icon": null, "owner_id": "99" }, "roles": [{ "id": "30", "name": "@everyone", "permissions": everyone.to_string() }], "channels": channels }),
        )
    }

    #[test]
    fn threads_inherit_the_parents_overwrites_and_send_with_send_messages_in_threads() {
        let state = AppState::new();
        state.apply_event(&ready());
        let view_send = (P::VIEW_CHANNEL | P::SEND_MESSAGES | P::ATTACH_FILES).0;
        state.apply_event(&permission_guild(
            view_send,
            json!([
                { "id": "31", "type": 0, "name": "general", "position": 0 },
                { "id": "32", "type": 11, "name": "a thread", "parent_id": "31" },
            ]),
        ));
        let thread = stored_channel(&state, 30, 32);
        assert!(state.can(&thread, P::VIEW_CHANNEL));
        assert!(
            !state.can(&thread, P::SEND_MESSAGES),
            "SEND_MESSAGES alone doesn't send in a thread"
        );
        assert!(!state.can(&thread, P::ATTACH_FILES), "nor its extras");
        assert!(state.can(&stored_channel(&state, 30, 31), P::SEND_MESSAGES));

        let threads_only = json!([{ "id": "30", "type": 0, "allow": P::SEND_MESSAGES_IN_THREADS.0.to_string(), "deny": P::SEND_MESSAGES.0.to_string() }]);
        state.apply_event(&event("CHANNEL_UPDATE", json!({ "id": "31", "guild_id": "30", "type": 0, "name": "general", "permission_overwrites": threads_only })));
        assert!(!state.can(&stored_channel(&state, 30, 31), P::SEND_MESSAGES));
        assert!(state.can(&thread, P::SEND_MESSAGES | P::ATTACH_FILES));

        let hidden =
            json!([{ "id": "30", "type": 0, "allow": "0", "deny": P::VIEW_CHANNEL.0.to_string() }]);
        state.apply_event(&event("CHANNEL_UPDATE", json!({ "id": "31", "guild_id": "30", "type": 0, "name": "general", "permission_overwrites": hidden })));
        assert_eq!(state.channel_permissions(&thread), P::NONE);
    }

    #[test]
    fn a_timeout_from_guild_member_update_masks_permissions_until_cleared() {
        let state = AppState::new();
        state.apply_event(&ready());
        let everyone = (P::VIEW_CHANNEL
            | P::SEND_MESSAGES
            | P::READ_MESSAGE_HISTORY
            | P::ADD_REACTIONS
            | P::CONNECT)
            .0;
        state.apply_event(&permission_guild(everyone, json!([{ "id": "31", "type": 0, "name": "general" }, { "id": "33", "type": 2, "name": "voice" }])));
        let (text, voice) = (
            stored_channel(&state, 30, 31),
            stored_channel(&state, 30, 33),
        );
        assert!(state.can(&text, P::SEND_MESSAGES) && state.can(&voice, P::CONNECT));

        let until = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
        state.apply_event(&event("GUILD_MEMBER_UPDATE", json!({ "guild_id": "30", "user": user(1, "me"), "roles": [], "communication_disabled_until": until })));
        assert_eq!(
            state.channel_permissions(&text),
            P::VIEW_CHANNEL | P::READ_MESSAGE_HISTORY
        );
        assert_eq!(
            state.channel_permissions(&voice),
            P::VIEW_CHANNEL,
            "no CONNECT in voice: only listed"
        );
        let guild = state.guilds.find(Id::new(30)).unwrap();
        assert_eq!(
            state.permissions.guild_permissions(&guild),
            P::VIEW_CHANNEL | P::READ_MESSAGE_HISTORY
        );

        let past = (chrono::Utc::now() - chrono::Duration::minutes(1)).to_rfc3339();
        state.apply_event(&event("GUILD_MEMBER_UPDATE", json!({ "guild_id": "30", "user": user(1, "me"), "roles": [], "communication_disabled_until": past })));
        assert!(state.can(&text, P::SEND_MESSAGES));
        state.apply_event(&event("GUILD_MEMBER_UPDATE", json!({ "guild_id": "30", "user": user(1, "me"), "roles": [], "communication_disabled_until": until })));
        assert!(!state.can(&text, P::SEND_MESSAGES));
        state.apply_event(&event("GUILD_MEMBER_UPDATE", json!({ "guild_id": "30", "user": user(1, "me"), "roles": [], "communication_disabled_until": null })));
        assert!(state.can(&text, P::SEND_MESSAGES));

        state.apply_event(&event("GUILD_MEMBER_UPDATE", json!({ "guild_id": "30", "user": user(2, "ada"), "roles": [], "communication_disabled_until": until })));
        assert!(state.can(&text, P::SEND_MESSAGES));
    }

    #[test]
    fn a_timeout_in_readys_merged_members_is_read() {
        let state = AppState::new();
        let until = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
        state.apply_event(&edited(ready(), |v| v["merged_members"] = json!([[{ "guild_id": "10", "user_id": "1", "roles": [], "communication_disabled_until": until }]])));
        assert!(state.permissions.is_timed_out(Id::new(10)));
    }

    #[test]
    fn a_channel_update_without_overwrites_keeps_the_stored_ones() {
        let state = AppState::new();
        state.apply_event(&ready());
        let deny_send = json!([{ "id": "30", "type": 0, "allow": "0", "deny": P::SEND_MESSAGES.0.to_string() }]);
        state.apply_event(&permission_guild((P::VIEW_CHANNEL | P::SEND_MESSAGES).0, json!([{ "id": "31", "type": 0, "name": "general", "permission_overwrites": deny_send }])));
        state.apply_event(&event(
            "CHANNEL_UPDATE",
            json!({ "id": "31", "guild_id": "30", "type": 0, "name": "renamed" }),
        ));
        let channel = stored_channel(&state, 30, 31);
        assert_eq!(channel.name.as_deref(), Some("renamed"));
        assert_eq!(
            channel.permission_overwrites.len(),
            1,
            "an omitted list keeps the overwrites"
        );
        assert!(!state.can(&channel, P::SEND_MESSAGES));

        state.apply_event(&event("CHANNEL_UPDATE", json!({ "id": "31", "guild_id": "30", "type": 0, "name": "renamed", "permission_overwrites": [] })));
        let channel = stored_channel(&state, 30, 31);
        assert!(channel.permission_overwrites.is_empty());
        assert!(state.can(&channel, P::SEND_MESSAGES));
    }

    #[test]
    fn reaction_events_count_up_and_down_and_know_which_are_yours() {
        let state = AppState::new();
        state.apply_event(&ready());
        state.apply_event(&event("MESSAGE_CREATE", message(100, 11, "hi")));

        state.apply_event(&reaction_event("MESSAGE_REACTION_ADD", 1, "up"));
        state.apply_event(&reaction_event("MESSAGE_REACTION_ADD", 2, "up"));
        state.apply_event(&reaction_event("MESSAGE_REACTION_ADD", 2, "laugh"));
        assert_eq!(
            reactions_of(&state, 11, 100),
            [("up".to_string(), 2, true), ("laugh".to_string(), 1, false)]
        );

        state.apply_event(&reaction_event("MESSAGE_REACTION_REMOVE", 2, "up"));
        state.apply_event(&reaction_event("MESSAGE_REACTION_REMOVE", 2, "laugh"));
        assert_eq!(reactions_of(&state, 11, 100), [("up".to_string(), 1, true)]);

        state.apply_event(&reaction_event("MESSAGE_REACTION_REMOVE", 1, "up"));
        assert!(reactions_of(&state, 11, 100).is_empty());
    }

    #[test]
    fn remove_emoji_and_remove_all_clear_reactions() {
        let state = AppState::new();
        state.apply_event(&ready());
        state.apply_event(&event("MESSAGE_CREATE", message(100, 11, "hi")));
        state.apply_event(&reaction_event("MESSAGE_REACTION_ADD", 2, "a"));
        state.apply_event(&reaction_event("MESSAGE_REACTION_ADD", 2, "b"));

        state.apply_event(&event("MESSAGE_REACTION_REMOVE_EMOJI", json!({ "channel_id": "11", "message_id": "100", "emoji": { "id": null, "name": "a" } })));
        assert_eq!(reactions_of(&state, 11, 100), [("b".to_string(), 1, false)]);

        state.apply_event(&event(
            "MESSAGE_REACTION_REMOVE_ALL",
            json!({ "channel_id": "11", "message_id": "100" }),
        ));
        assert!(reactions_of(&state, 11, 100).is_empty());
    }

    fn guild_with_emojis(emojis: serde_json::Value) -> serde_json::Value {
        json!({ "id": "10", "properties": { "name": "Guild", "icon": null, "owner_id": "1" }, "channels": [], "emojis": emojis })
    }

    #[test]
    fn guild_emojis_arrive_with_the_guild_and_follow_updates() {
        let state = AppState::new();
        let open = edited(ready(), |v| {
            v["guilds"] = json!([guild_with_emojis(
                json!([{ "id": "500", "name": "happy", "animated": false }, { "id": "501", "name": "wiggle", "animated": true, "available": false }])
            )])
        });
        state.apply_event(&open);
        let listed = state
            .emojis
            .by_guild
            .with_untracked(|m| m[&Id::<GuildMarker>::new(10)].clone());
        assert_eq!(
            listed
                .iter()
                .map(|e| (e.name.as_str(), e.animated, e.available))
                .collect::<Vec<_>>(),
            [("happy", false, true), ("wiggle", true, false)]
        );
        assert_eq!(
            state
                .emojis
                .find(Id::new(500))
                .map(|(guild, e)| (guild.get(), e.markup())),
            Some((10, "<:happy:500>".to_string()))
        );
        assert_eq!(
            state.emojis.find(Id::new(501)).map(|(_, e)| e.markup()),
            Some("<a:wiggle:501>".to_string())
        );

        state.apply_event(&event(
            "GUILD_EMOJIS_UPDATE",
            json!({ "guild_id": "10", "emojis": [{ "id": "502", "name": "new" }] }),
        ));
        assert!(state.emojis.find(Id::new(500)).is_none());
        assert!(state.emojis.find(Id::new(502)).is_some());
    }

    #[test]
    fn a_message_with_reactions_deserializes_them() {
        let mut data = message(100, 11, "hi");
        data["reactions"] = json!([{ "count": 3, "me": true, "emoji": { "id": null, "name": "up", "animated": false }, "count_details": { "normal": 3, "burst": 0 }, "me_burst": false }]);
        let message: Message = serde_json::from_value(data).unwrap();
        assert_eq!(
            (message.reactions[0].count, message.reactions[0].me),
            (3, true)
        );
    }

    #[test]
    fn user_update_refreshes_the_signed_in_user_and_keeps_caller_only_fields() {
        let state = AppState::new();
        let me = edited(ready(), |v| v["user"]["email"] = json!("me@example.com"));
        state.apply_event(&me);
        state.apply_event(&event("USER_UPDATE", json!({ "id": "1", "username": "me", "discriminator": "0", "global_name": "Mia", "avatar": "abc" })));
        let current = state.users.current.peek().expect("signed in");
        assert_eq!(current.global_name.as_deref(), Some("Mia"));
        assert_eq!(current.avatar.as_deref(), Some("abc"));
        assert_eq!(
            current.email.as_deref(),
            Some("me@example.com"),
            "an update without caller-only fields keeps them"
        );
    }

    #[test]
    fn ready_fills_every_store_and_flips_ready_last() {
        let state = AppState::new();
        assert!(!state.connection.ready.peek());
        state.apply_event(&ready());
        assert_eq!(
            state.users.current.peek().map(|u| u.username),
            Some("me".to_string())
        );
        assert_eq!(
            state.guilds.list.peek().len(),
            1,
            "unavailable stub skipped"
        );
        assert_eq!(
            state
                .channels
                .by_guild
                .peek()
                .get(&Id::new(10))
                .map(Vec::len),
            Some(1)
        );
        assert_eq!(
            state.connection.auth_session_id_hash.peek().as_deref(),
            Some("abc")
        );
        assert!(state.connection.ready.peek());
    }

    #[test]
    fn reset_empties_everything() {
        let state = AppState::new();
        state.apply_event(&ready());
        state.apply_event(&event("MESSAGE_CREATE", message(100, 11, "hi")));
        state.reset();
        assert!(state.users.current.peek().is_none());
        assert!(state.guilds.list.peek().is_empty());
        assert!(state.channels.by_guild.peek().is_empty());
        assert!(state.messages.by_channel.peek().is_empty());
        assert!(!state.connection.ready.peek());
    }

    #[test]
    fn a_resumed_connection_is_ready_again_without_a_new_ready_and_keeps_what_it_had() {
        let state = AppState::new();
        state.apply_event(&ready());
        state.connection.ready.set(false);
        state.apply_event(&event("RESUMED", json!({})));
        assert!(state.connection.ready.peek());
        assert_eq!(state.guilds.list.peek().len(), 1);
    }

    #[test]
    fn a_channels_revision_moves_only_for_its_own_messages() {
        let state = AppState::new();
        let (open, other): (Id<ChannelMarker>, Id<ChannelMarker>) = (Id::new(11), Id::new(12));
        let revision = state.messages.channel_revision(open);
        state.apply_event(&event("MESSAGE_CREATE", message(100, 11, "hi")));
        assert_eq!(revision.peek(), 1);
        state.apply_event(&event("MESSAGE_CREATE", message(200, 12, "elsewhere")));
        state.apply_event(&event(
            "MESSAGE_UPDATE",
            message(200, 12, "edited elsewhere"),
        ));
        assert_eq!(
            revision.peek(),
            1,
            "a message in another channel must not rebuild this channel's list"
        );
        assert_eq!(state.messages.in_channel(other).len(), 1);
        state.apply_event(&event("MESSAGE_UPDATE", message(100, 11, "edited")));
        assert_eq!(revision.peek(), 2);
        assert_eq!(state.messages.in_channel(open)[0].content, "edited");
    }

    #[test]
    fn message_create_dedupes_update_replaces_delete_removes() {
        let state = AppState::new();
        let channel: Id<ChannelMarker> = Id::new(11);
        state.apply_event(&event("MESSAGE_CREATE", message(100, 11, "hi")));
        state.apply_event(&event("MESSAGE_CREATE", message(100, 11, "hi")));
        state.apply_event(&event("MESSAGE_CREATE", message(101, 11, "second")));
        assert_eq!(state.messages.by_channel.peek()[&channel].len(), 2);

        state.apply_event(&event("MESSAGE_UPDATE", message(100, 11, "edited")));
        assert_eq!(
            state.messages.by_channel.peek()[&channel][0].content,
            "edited"
        );

        state.apply_event(&event("MESSAGE_UPDATE", message(999, 11, "ghost")));
        assert_eq!(state.messages.by_channel.peek()[&channel].len(), 2);

        state.apply_event(&event(
            "MESSAGE_DELETE",
            json!({ "id": "100", "channel_id": "11" }),
        ));
        let remaining: Vec<_> = state.messages.by_channel.peek()[&channel]
            .iter()
            .map(|m| m.content.clone())
            .collect();
        assert_eq!(remaining, vec!["second".to_string()]);
    }

    #[test]
    fn history_is_flipped_oldest_first_and_short_pages_mark_exhausted() {
        let state = AppState::new();
        let channel: Id<ChannelMarker> = Id::new(11);
        let parse = |id, text| serde_json::from_value::<Message>(message(id, 11, text)).unwrap();
        state
            .messages
            .set_history(channel, vec![parse(103, "c"), parse(102, "b")]);
        let contents: Vec<_> = state.messages.by_channel.peek()[&channel]
            .iter()
            .map(|m| m.content.clone())
            .collect();
        assert_eq!(contents, vec!["b", "c"]);
        assert!(
            state.messages.exhausted.peek().contains(&channel),
            "a page shorter than the size asked for is the start of history"
        );

        state
            .messages
            .prepend_history(channel, vec![parse(102, "dup"), parse(101, "a")]);
        let contents: Vec<_> = state.messages.by_channel.peek()[&channel]
            .iter()
            .map(|m| m.content.clone())
            .collect();
        assert_eq!(
            contents,
            vec!["a", "b", "c"],
            "already-loaded messages aren't duplicated"
        );
    }

    #[test]
    fn loading_too_much_history_cuts_the_newest_end_back_and_detaches_the_channel() {
        let state = AppState::new();
        let channel: Id<ChannelMarker> = Id::new(11);
        let parse = |id: u64| serde_json::from_value::<Message>(message(id, 11, "m")).unwrap();
        let page = |from: u64| (from..from + 50).rev().map(parse).collect::<Vec<_>>();

        state.messages.set_history(channel, page(151));
        for from in [101, 51, 1] {
            state.messages.prepend_history(channel, page(from));
        }
        assert_eq!(state.messages.by_channel.peek()[&channel].len(), 200);
        assert!(!state.messages.detached.peek().contains(&channel));

        state.messages.prepend_history(
            channel,
            (0..50)
                .rev()
                .map(|i| parse(1000 + i))
                .map(|mut m| {
                    m.id = Id::new(m.id.get() - 1000);
                    m
                })
                .collect(),
        );
        let loaded = state.messages.by_channel.peek()[&channel].clone();
        assert_eq!(loaded.len(), TRUNCATED_VIEW_SIZE);
        assert!(state.messages.detached.peek().contains(&channel));
        assert_eq!(loaded.first().map(|m| m.id.get()), Some(0));
        state.messages.set_history(channel, page(151));
        assert!(
            !state.messages.detached.peek().contains(&channel),
            "the present is loaded again"
        );
    }

    #[test]
    fn ready_fills_friends_dms_and_presence_and_events_keep_them_current() {
        let state = AppState::new();
        let data = edited(ready(), |v| {
            v["users"] = json!([user(2, "ada")]);
            v["relationships"] = json!([{ "id": "2", "type": 1, "nickname": null, "user": user(2, "ada") }, { "id": "3", "type": 3, "nickname": null, "user": user(3, "bob") }]);
            v["private_channels"] = json!([{ "id": "50", "type": 1, "recipient_ids": ["2"], "last_message_id": "60" }, { "id": "51", "type": 1, "recipient_ids": ["3"], "last_message_id": "70" }]);
        });
        state.apply_event(&data);
        assert_eq!(state.relationships.list.peek().len(), 2);
        assert!(state.relationships.list.peek()[1].is_incoming());
        assert_eq!(
            state
                .dms
                .list
                .peek()
                .iter()
                .map(|d| d.id.get())
                .collect::<Vec<_>>(),
            vec![51, 50],
            "most recent first"
        );
        assert_eq!(
            state.users.get(Id::new(2)).map(|u| u.username),
            Some("ada".to_string())
        );

        state.apply_event(&event("READY_SUPPLEMENTAL", json!({ "merged_presences": { "friends": [{ "user": { "id": "2" }, "status": "dnd", "activities": [] }] } })));
        assert_eq!(
            state.user_presence.of(Id::new(2)).status,
            PresenceStatus::Dnd
        );
        state.apply_event(&event(
            "PRESENCE_UPDATE",
            json!({ "user": { "id": "2" }, "status": "offline", "activities": [] }),
        ));
        assert_eq!(
            state.user_presence.of(Id::new(2)).status,
            PresenceStatus::Offline
        );

        state.apply_event(&event(
            "RELATIONSHIP_UPDATE",
            json!({ "id": "3", "type": 1, "nickname": null }),
        ));
        assert!(state.relationships.list.peek()[1].is_friend());
        state.apply_event(&event(
            "RELATIONSHIP_REMOVE",
            json!({ "id": "3", "type": 1 }),
        ));
        assert_eq!(state.relationships.list.peek().len(), 1);

        state.apply_event(&event("MESSAGE_CREATE", message(80, 50, "hi")));
        assert_eq!(state.dms.list.peek()[0].id.get(), 50);

        state.apply_event(&event(
            "CHANNEL_CREATE",
            json!({ "id": "52", "type": 1, "recipients": [user(4, "cy")] }),
        ));
        assert_eq!(
            state.dms.with_user(Id::new(4)).map(|d| d.id.get()),
            Some(52)
        );
        state.reset();
        assert!(state.relationships.list.peek().is_empty() && state.dms.list.peek().is_empty());
    }

    #[test]
    fn unknown_events_are_ignored() {
        let state = AppState::new();
        state.apply_event(&event("SOME_FUTURE_EVENT", json!({})));
        assert!(state.messages.by_channel.peek().is_empty());
    }
}
