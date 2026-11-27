use crate::auth::TokenStore;
use crate::instances::InstanceStore;
use crate::models::id::marker::GuildMarker;
use crate::models::id::marker::UserMarker;
use crate::models;
use crate::rest::endpoints::users::ProfileView;
use crate::rest::endpoints::users;
use crate::rest;
use requiem_ui::clipboard;
use std::collections::HashMap;
use std::future;
use std::mem;
use std::panic;
use std::path::PathBuf;
use std::path;
use std::slice;
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::thread::Builder;
use tokio::runtime::Handle;

use requiem_plugin_host::{
    AttachmentId, ChannelData, ChannelId, EditDraft, GuildData as PluginGuild, GuildId, HostError,
    HostResult, HostServices, IncomingItem, MemberData, MessageData, MessageId, OutgoingDraft,
    OverlayUpdate, PresenceData, ProfileData, RelationshipData, RoleData, RoleId, UserData, UserId,
};
pub use requiem_plugin_host::{
    ByPlugin, IncomingOutcome, IncomingSource, MenuItemData, PermissionInfo, PluginHost,
    PluginInfo, PluginPreview, Registrations, SlotItemData, Status as PluginStatus, ViewNode,
    ViewTree,
};

use crate::gateway::events::GatewayEvent;
use crate::models::id::marker::{ChannelMarker, MessageMarker};
use crate::models::{Id, Message};
use crate::rest::RestError;
use crate::rest::endpoints::messages::ReplyTo;

#[derive(Clone, Debug)]
pub enum PluginEvent {
    Overlays(Vec<(Id<MessageMarker>, IncomingOutcome)>),

    OverlayUpdate {
        plugin: String,
        update: OverlayUpdate,
    },

    StatusChanged,

    SendCancelled {
        channel_id: Id<ChannelMarker>,
        content: String,
        plugin: String,
        reason: String,
    },

    SendModified {
        channel_id: Id<ChannelMarker>,
        plugins: Vec<String>,
    },

    SettingsView {
        plugin: String,
        view: Result<ViewTree, String>,
    },
    InstallFailed(String),

    Toast {
        text: String,
        kind: String,
    },

    Surface(SurfaceRequest),

    ViewRendered {
        plugin: String,
        view_id: String,
        view: Result<ViewTree, String>,
    },

    OpenMedia {
        url: String,
    },

    OpenGuild {
        guild_id: u64,
    },

    OpenProfile {
        user_id: u64,
        guild_id: Option<u64>,
    },

    RefreshView {
        plugin: String,
        view_id: String,
    },
}

#[derive(Clone, Debug)]
pub enum SurfaceRequest {
    OpenModal {
        plugin: String,
        view_id: String,
        title: String,
        size: String,
    },
    CloseModal {
        plugin: String,
        view_id: String,
    },
}

struct Shared {
    inbox: Mutex<Vec<PluginEvent>>,
    wake: Box<dyn Fn() + Send + Sync>,
    context: RwLock<Context>,

    http: RwLock<Option<(Handle, rest::HttpClient)>>,
}

#[derive(Default)]
struct Context {
    me: Option<u64>,
    channel_guilds: HashMap<u64, u64>,
    cdn_base: String,

    guilds: HashMap<u64, GuildSnapshot>,
    users: HashMap<u64, UserData>,
    selected_guild: Option<u64>,
    relationships: Vec<RelationshipData>,
    presences: HashMap<u64, PresenceData>,

    my_joined: HashMap<u64, i64>,
}

#[derive(Clone, Default)]
pub struct GuildSnapshot {
    pub guild: PluginGuild,
    pub channels: Vec<ChannelData>,
    pub roles: Vec<RoleData>,
    pub members: Vec<MemberData>,
}

impl Shared {
    fn push(&self, event: PluginEvent) {
        self.inbox
            .lock()
            .expect("plugin inbox poisoned")
            .push(event);
        (self.wake)();
    }
}

struct Runtime {
    host: Arc<PluginHost>,
    shared: Arc<Shared>,
}

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

const SECRET_SERVICE: &str = "requiem.plugins";

struct Services(Arc<Shared>);

impl Services {
    fn secret(plugin_id: &str, key: &str) -> TokenStore {
        TokenStore::in_service(SECRET_SERVICE, format!("{plugin_id}/{key}"))
    }
}

impl HostServices for Services {
    fn overlay(&self, plugin_id: &str, update: OverlayUpdate) {
        self.0.push(PluginEvent::OverlayUpdate {
            plugin: plugin_id.to_string(),
            update,
        });
    }

    fn secret_get(&self, plugin_id: &str, key: &str) -> HostResult<Option<String>> {
        Self::secret(plugin_id, key)
            .load()
            .map_err(|err| HostError::new("unavailable", err.to_string()))
    }

    fn secret_set(&self, plugin_id: &str, key: &str, value: &str) -> HostResult<()> {
        Self::secret(plugin_id, key)
            .save(value)
            .map_err(|err| HostError::new("unavailable", err.to_string()))
    }

    fn secret_delete(&self, plugin_id: &str, key: &str) -> HostResult<()> {
        Self::secret(plugin_id, key)
            .clear()
            .map_err(|err| HostError::new("unavailable", err.to_string()))
    }

    fn status_changed(&self, _plugin_id: &str) {
        self.0.push(PluginEvent::StatusChanged);
    }

    fn client_info(&self) -> String {
        serde_json::json!({ "version": env!("CARGO_PKG_VERSION"), "platform": "desktop" })
            .to_string()
    }

    fn current_user(&self) -> Option<UserData> {
        let context = self.0.context.read().expect("plugin context poisoned");
        context.me.and_then(|me| context.users.get(&me).cloned())
    }

    fn selected_guild(&self) -> Option<GuildId> {
        self.0
            .context
            .read()
            .expect("plugin context poisoned")
            .selected_guild
            .map(GuildId::new)
    }

    fn user(&self, id: UserId) -> Option<UserData> {
        self.0
            .context
            .read()
            .expect("plugin context poisoned")
            .users
            .get(&id.get())
            .cloned()
    }

    fn guild(&self, id: GuildId) -> Option<PluginGuild> {
        self.0
            .context
            .read()
            .expect("plugin context poisoned")
            .guilds
            .get(&id.get())
            .map(|g| g.guild.clone())
    }

    fn guilds(&self) -> Vec<PluginGuild> {
        let context = self.0.context.read().expect("plugin context poisoned");
        let mut guilds: Vec<_> = context.guilds.values().map(|g| g.guild.clone()).collect();
        guilds.sort_by_key(|g| g.id);
        guilds
    }

    fn channels(&self, guild_id: GuildId) -> Vec<ChannelData> {
        self.0
            .context
            .read()
            .expect("plugin context poisoned")
            .guilds
            .get(&guild_id.get())
            .map(|g| g.channels.clone())
            .unwrap_or_default()
    }

    fn roles(&self, guild_id: GuildId) -> Vec<RoleData> {
        self.0
            .context
            .read()
            .expect("plugin context poisoned")
            .guilds
            .get(&guild_id.get())
            .map(|g| g.roles.clone())
            .unwrap_or_default()
    }

    fn members(&self, guild_id: GuildId) -> Vec<MemberData> {
        self.0
            .context
            .read()
            .expect("plugin context poisoned")
            .guilds
            .get(&guild_id.get())
            .map(|g| g.members.clone())
            .unwrap_or_default()
    }

    fn member(&self, guild_id: GuildId, user_id: UserId) -> Option<MemberData> {
        let context = self.0.context.read().expect("plugin context poisoned");
        let found = context
            .guilds
            .get(&guild_id.get())
            .and_then(|g| g.members.iter().find(|m| m.user.id == user_id).cloned());

        match (
            context.me == Some(user_id.get()),
            context.my_joined.get(&guild_id.get()),
        ) {
            (true, Some(joined)) => {
                let mut member = found.unwrap_or_else(|| MemberData {
                    guild_id,
                    user: context
                        .users
                        .get(&user_id.get())
                        .cloned()
                        .unwrap_or_default(),
                    ..Default::default()
                });
                member.joined_at = Some(*joined);
                Some(member)
            }
            _ => found,
        }
    }

    fn open_media(&self, url: &str) -> HostResult<()> {
        self.0.push(PluginEvent::OpenMedia {
            url: url.to_string(),
        });
        Ok(())
    }

    fn open_guild(&self, guild_id: GuildId) -> HostResult<()> {
        self.0.push(PluginEvent::OpenGuild {
            guild_id: guild_id.get(),
        });
        Ok(())
    }

    fn open_profile(&self, user_id: UserId, guild_id: Option<GuildId>) -> HostResult<()> {
        self.0.push(PluginEvent::OpenProfile {
            user_id: user_id.get(),
            guild_id: guild_id.map(|id| id.get()),
        });
        Ok(())
    }

    fn relationships(&self) -> Vec<RelationshipData> {
        self.0
            .context
            .read()
            .expect("plugin context poisoned")
            .relationships
            .clone()
    }

    fn presence(&self, user_id: UserId) -> Option<PresenceData> {
        self.0
            .context
            .read()
            .expect("plugin context poisoned")
            .presences
            .get(&user_id.get())
            .cloned()
    }

    fn fetch_profile(&self, user_id: UserId, guild_id: Option<GuildId>) -> HostResult<ProfileData> {
        let (handle, http) = self
            .0
            .http
            .read()
            .expect("plugin http poisoned")
            .clone()
            .ok_or_else(HostError::unavailable)?;
        let cdn = self
            .0
            .context
            .read()
            .expect("plugin context poisoned")
            .cdn_base
            .clone();
        let request = users::get_user_profile(
            &http,
            Id::<UserMarker>::new(user_id.get()),
            guild_id.map(|id| Id::<GuildMarker>::new(id.get())),
            ProfileView::Plain,
        );

        let profile = handle
            .block_on(request)
            .map_err(|err| HostError::new("unavailable", err.to_string()))?;
        Ok(profile_data(&profile, guild_id.map(|id| id.get()), &cdn))
    }

    fn copy_to_clipboard(&self, _plugin_id: &str, text: &str) -> HostResult<()> {
        if clipboard::set_text(text) {
            Ok(())
        } else {
            Err(HostError::new(
                "unavailable",
                "the clipboard isn't available",
            ))
        }
    }

    fn refresh_view(&self, plugin_id: &str, view_id: &str) {
        self.0.push(PluginEvent::RefreshView {
            plugin: plugin_id.to_string(),
            view_id: view_id.to_string(),
        });
    }

    fn toast(&self, _plugin_id: &str, text: &str, kind: &str) {
        self.0.push(PluginEvent::Toast {
            text: text.to_string(),
            kind: kind.to_string(),
        });
    }

    fn open_surface(
        &self,
        plugin_id: &str,
        surface: &str,
        view_id: &str,
        title: &str,
        extra: &str,
    ) -> HostResult<()> {
        match surface {
            "modal" => {
                self.0.push(PluginEvent::Surface(SurfaceRequest::OpenModal {
                    plugin: plugin_id.to_string(),
                    view_id: view_id.to_string(),
                    title: title.to_string(),
                    size: extra.to_string(),
                }));
                Ok(())
            }
            _ => Err(HostError::unavailable()),
        }
    }

    fn close_surface(&self, plugin_id: &str, surface: &str, view_id: &str) {
        if surface == "modal" {
            self.0
                .push(PluginEvent::Surface(SurfaceRequest::CloseModal {
                    plugin: plugin_id.to_string(),
                    view_id: view_id.to_string(),
                }));
        }
    }
}

pub fn default_dir() -> Option<PathBuf> {
    InstanceStore::default_path()
        .and_then(|path| path.parent().map(|dir| dir.join("plugins")))
}

pub fn init(dir: PathBuf, wake: impl Fn() + Send + Sync + 'static) {
    let shared = Arc::new(Shared {
        inbox: Mutex::new(Vec::new()),
        wake: Box::new(wake),
        context: RwLock::new(Context::default()),
        http: RwLock::new(None),
    });
    match PluginHost::with_db(
        &dir,
        requiem_db::Db::shared().clone(),
        Arc::new(Services(shared.clone())),
    ) {
        Ok(host) => {
            log::info!(
                "plugins: {} installed in {}",
                host.list().len(),
                dir.display()
            );
            let _ = RUNTIME.set(Runtime { host, shared });
        }
        Err(err) => log::error!("plugins are off: {err}"),
    }
}

pub fn set_cdn_base(cdn_base: String) {
    if let Some(runtime) = RUNTIME.get() {
        runtime
            .shared
            .context
            .write()
            .expect("plugin context poisoned")
            .cdn_base = cdn_base;
    }
}

pub fn host() -> Option<Arc<PluginHost>> {
    RUNTIME.get().map(|runtime| runtime.host.clone())
}

pub fn take_events() -> Vec<PluginEvent> {
    RUNTIME
        .get()
        .map(|runtime| {
            mem::take(&mut *runtime.shared.inbox.lock().expect("plugin inbox poisoned"))
        })
        .unwrap_or_default()
}

pub(crate) fn update_context(update: impl FnOnce(&mut Option<u64>, &mut HashMap<u64, u64>)) {
    if let Some(runtime) = RUNTIME.get() {
        let mut context = runtime
            .shared
            .context
            .write()
            .expect("plugin context poisoned");
        let Context {
            me, channel_guilds, ..
        } = &mut *context;
        update(me, channel_guilds);
    }
}

pub(crate) fn remember_joined(guild_id: u64, joined_ms: i64) {
    if let Some(runtime) = RUNTIME.get() {
        runtime
            .shared
            .context
            .write()
            .expect("plugin context poisoned")
            .my_joined
            .insert(guild_id, joined_ms);
    }
}

pub fn set_http(handle: Handle, http: rest::HttpClient) {
    if let Some(runtime) = RUNTIME.get() {
        *runtime.shared.http.write().expect("plugin http poisoned") = Some((handle, http));
    }
}

pub(crate) fn set_social(relationships: Vec<RelationshipData>, presences: Vec<PresenceData>) {
    if let Some(runtime) = RUNTIME.get() {
        let mut context = runtime
            .shared
            .context
            .write()
            .expect("plugin context poisoned");
        context.relationships = relationships;
        for presence in presences {
            context.presences.insert(presence.user_id.get(), presence);
        }
    }
}

fn profile_data(
    profile: &models::UserProfile,
    guild_id: Option<u64>,
    cdn: &str,
) -> ProfileData {
    let user = &profile.user;
    let user_data = UserData {
        id: UserId::new(user.id.get()),
        username: user.username.clone(),
        global_name: user.global_name.clone(),
        avatar_url: user.avatar.as_ref().map(|hash| {
            format!(
                "{}/avatars/{}/{hash}.png",
                cdn.trim_end_matches('/'),
                user.id
            )
        }),
        bot: user.bot,
        ..Default::default()
    };
    let member = profile
        .guild_member
        .as_ref()
        .zip(guild_id)
        .map(|(member, guild_id)| MemberData {
            guild_id: GuildId::new(guild_id),
            user: user_data.clone(),
            nick: member.nick.clone(),
            role_ids: member.roles.iter().map(|r| RoleId::new(r.get())).collect(),
            joined_at: member
                .joined_at
                .as_deref()
                .and_then(|at| chrono::DateTime::parse_from_rfc3339(at).ok())
                .map(|at| at.timestamp_millis()),
            ..Default::default()
        });
    ProfileData {
        user: user_data,
        bio: Some(user.bio.clone()).filter(|bio| !bio.trim().is_empty()),
        member,
        mutual_guild_ids: profile
            .mutual_guilds
            .iter()
            .flatten()
            .map(|g| GuildId::new(g.id.get()))
            .collect(),
        mutual_friend_ids: profile
            .mutual_friends
            .iter()
            .flatten()
            .map(|f| UserId::new(f.id.get()))
            .collect(),
        ..Default::default()
    }
}

pub(crate) fn set_guild_snapshot(guild_id: u64, build: impl FnOnce(&str) -> GuildSnapshot) {
    if let Some(runtime) = RUNTIME.get() {
        let mut context = runtime
            .shared
            .context
            .write()
            .expect("plugin context poisoned");
        let snapshot = build(&context.cdn_base);
        context.guilds.insert(guild_id, snapshot);
    }
}

pub(crate) fn remove_guild_snapshot(guild_id: u64) {
    if let Some(runtime) = RUNTIME.get() {
        runtime
            .shared
            .context
            .write()
            .expect("plugin context poisoned")
            .guilds
            .remove(&guild_id);
    }
}

pub(crate) fn remember_users(users: impl IntoIterator<Item = UserData>) {
    if let Some(runtime) = RUNTIME.get() {
        let mut context = runtime
            .shared
            .context
            .write()
            .expect("plugin context poisoned");
        for user in users {
            context.users.insert(user.id.get(), user);
        }
    }
}

pub(crate) fn set_selected_guild(guild_id: Option<u64>) {
    if let Some(runtime) = RUNTIME.get() {
        runtime
            .shared
            .context
            .write()
            .expect("plugin context poisoned")
            .selected_guild = guild_id;
    }
}

pub(crate) fn cdn_base() -> String {
    RUNTIME
        .get()
        .map(|runtime| {
            runtime
                .shared
                .context
                .read()
                .expect("plugin context poisoned")
                .cdn_base
                .clone()
        })
        .unwrap_or_default()
}

async fn blocking<R: Send + 'static>(job: impl FnOnce() -> R + Send + 'static) -> R {
    match Handle::try_current() {
        Ok(handle) => match handle.spawn_blocking(job).await {
            Ok(result) => result,
            Err(err) => panic::resume_unwind(err.into_panic()),
        },
        Err(_) => job(),
    }
}

fn message_data(message: &Message, context: &Context) -> MessageData {
    let author = &message.author;
    let me = context.me;
    MessageData {
        id: MessageId::new(message.id.get()),
        channel_id: ChannelId::new(message.channel_id.get()),
        guild_id: context
            .channel_guilds
            .get(&message.channel_id.get())
            .copied()
            .map(GuildId::new),
        author: UserData {
            id: UserId::new(author.id.get()),
            username: author.username.clone(),
            global_name: author.global_name.clone(),
            avatar_url: author.avatar.as_ref().map(|hash| {
                format!(
                    "{}/avatars/{}/{hash}.png",
                    context.cdn_base.trim_end_matches('/'),
                    author.id
                )
            }),
            bot: author.bot,
            ..Default::default()
        },
        content: message.content.clone(),
        timestamp: message.timestamp_millis().unwrap_or_default(),

        edited_timestamp: None,
        attachments: message
            .attachments
            .iter()
            .map(|a| requiem_plugin_host::AttachmentData {
                id: AttachmentId::new(a.id.get()),
                filename: a.filename.clone(),
                content_type: a.content_type.clone(),
                size: a.size,
                url: a.url.clone(),
                ..Default::default()
            })
            .collect(),
        reply_to: message
            .message_reference
            .as_ref()
            .and_then(|r| r.message_id)
            .map(|id| MessageId::new(id.get())),
        mentions_me: message.mention_everyone
            || me.is_some_and(|me| message.mentions.iter().any(|u| u.id.get() == me)),
        ..Default::default()
    }
}

pub async fn transform(messages: &[Message], source: IncomingSource) {
    let Some(runtime) = RUNTIME.get() else { return };
    if messages.is_empty() || !runtime.host.runs_any("incoming") {
        return;
    }
    let (host, shared) = (runtime.host.clone(), runtime.shared.clone());
    let batch: Vec<IncomingItem> = {
        let context = shared.context.read().expect("plugin context poisoned");
        messages
            .iter()
            .map(|m| IncomingItem {
                message: message_data(m, &context),
                previous: None,
                source,
            })
            .collect()
    };
    let ids: Vec<Id<MessageMarker>> = messages.iter().map(|m| m.id).collect();
    blocking(move || {
        let outcomes = host.run_incoming(batch);
        let changed: Vec<_> = ids
            .into_iter()
            .zip(outcomes)
            .filter(|(_, outcome)| !outcome.is_unchanged())
            .collect();
        if !changed.is_empty() {
            shared.push(PluginEvent::Overlays(changed));
        }
    })
    .await;
}

pub fn transform_gateway_event(
    event: &GatewayEvent,
) -> impl future::Future<Output = ()> + Send + 'static {
    let source = match event.name.as_str() {
        "MESSAGE_CREATE" => Some(IncomingSource::GatewayCreate),
        "MESSAGE_UPDATE" => Some(IncomingSource::GatewayUpdate),
        _ => None,
    };

    let message = source.and_then(|_| event.parse::<Message>().ok());
    async move {
        if let (Some(message), Some(source)) = (message, source) {
            transform(slice::from_ref(&message), source).await;
        }
    }
}

pub async fn before_send(
    channel_id: Id<ChannelMarker>,
    content: &str,
    reply: Option<&ReplyTo>,
) -> Result<(String, Option<ReplyTo>), RestError> {
    let Some(runtime) = RUNTIME.get() else {
        return Ok((content.to_string(), reply.cloned()));
    };
    if !runtime.host.runs_any("outgoing") {
        return Ok((content.to_string(), reply.cloned()));
    }
    let guild_id = runtime
        .shared
        .context
        .read()
        .expect("plugin context poisoned")
        .channel_guilds
        .get(&channel_id.get())
        .copied();
    let draft = OutgoingDraft {
        channel_id: ChannelId::new(channel_id.get()),
        guild_id: guild_id.map(GuildId::new),
        content: content.to_string(),
        reply_to: reply.map(|r| MessageId::new(r.message_id.get())),
        reply_mention: reply.is_some_and(|r| r.pings),
        ..Default::default()
    };
    let host = runtime.host.clone();
    let outcome = blocking(move || host.run_outgoing(draft)).await;
    let shared = &runtime.shared;
    if let Some(cancelled) = outcome.cancelled {
        shared.push(PluginEvent::SendCancelled {
            channel_id,
            content: content.to_string(),
            plugin: cancelled.plugin.clone(),
            reason: cancelled.value.clone(),
        });
        return Err(RestError::CancelledByPlugin {
            plugin: cancelled.plugin,
            reason: cancelled.value,
        });
    }
    if !outcome.modified_by.is_empty() {
        shared.push(PluginEvent::SendModified {
            channel_id,
            plugins: outcome.modified_by,
        });
    }
    let reply = reply.map(|r| ReplyTo {
        pings: outcome.draft.reply_mention,
        ..r.clone()
    });
    Ok((outcome.draft.content, reply))
}

pub async fn before_edit(
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    original: &str,
    content: &str,
) -> Result<String, RestError> {
    let Some(runtime) = RUNTIME.get() else {
        return Ok(content.to_string());
    };
    if !runtime.host.runs_any("outgoing") {
        return Ok(content.to_string());
    }
    let guild_id = runtime
        .shared
        .context
        .read()
        .expect("plugin context poisoned")
        .channel_guilds
        .get(&channel_id.get())
        .copied();
    let draft = EditDraft {
        channel_id: ChannelId::new(channel_id.get()),
        guild_id: guild_id.map(GuildId::new),
        message_id: MessageId::new(message_id.get()),
        original_content: original.to_string(),
        content: content.to_string(),
    };
    let host = runtime.host.clone();
    let outcome = blocking(move || host.run_edit(draft)).await;
    if let Some(cancelled) = outcome.cancelled {
        return Err(RestError::CancelledByPlugin {
            plugin: cancelled.plugin,
            reason: cancelled.value,
        });
    }
    if !outcome.modified_by.is_empty() {
        runtime.shared.push(PluginEvent::SendModified {
            channel_id,
            plugins: outcome.modified_by,
        });
    }
    Ok(outcome.draft.content)
}

fn in_background(job: impl FnOnce(&PluginHost, &Shared) + Send + 'static) {
    let Some(runtime) = RUNTIME.get() else { return };
    let (host, shared) = (runtime.host.clone(), runtime.shared.clone());
    Builder::new()
        .name("plugin-ui".into())
        .spawn(move || job(&host, &shared))
        .expect("spawning a plugin UI thread");
}

pub fn request_settings_view(plugin_id: &str) {
    let id = plugin_id.to_string();
    in_background(move |host, shared| {
        let view = host
            .render_view(&id, "settings", "{}")
            .map_err(|err| err.to_string());
        shared.push(PluginEvent::SettingsView { plugin: id, view });
    });
}

pub fn send_ui_action(
    plugin_id: &str,
    action_id: String,
    value: Option<serde_json::Value>,
    rerender: bool,
) {
    let id = plugin_id.to_string();
    in_background(move |host, shared| {
        match host.ui_action(&id, "settings", &action_id, value, rerender.then_some("{}")) {
            Ok(Some(view)) => shared.push(PluginEvent::SettingsView {
                plugin: id,
                view: Ok(view),
            }),
            Ok(None) => {}
            Err(err) => shared.push(PluginEvent::SettingsView {
                plugin: id,
                view: Err(err.to_string()),
            }),
        }
    });
}

pub fn set_enabled(plugin_id: &str, enabled: bool) {
    let id = plugin_id.to_string();
    in_background(move |host, shared| {
        if let Err(err) = host.set_enabled(&id, enabled) {
            log::error!("plugins: enabling {id}: {err}");
        }
        shared.push(PluginEvent::StatusChanged);
    });
}

pub fn set_permission(plugin_id: &str, permission: &str, granted: bool) {
    let (id, permission) = (plugin_id.to_string(), permission.to_string());
    in_background(move |host, shared| {
        if let Err(err) = host.set_permission(&id, &permission, granted) {
            log::error!("plugins: {permission} for {id}: {err}");
        }
        shared.push(PluginEvent::StatusChanged);
    });
}

pub fn remove(plugin_id: &str) {
    let id = plugin_id.to_string();
    in_background(move |host, shared| {
        if let Err(err) = host.remove(&id) {
            log::error!("plugins: removing {id}: {err}");
        }
        shared.push(PluginEvent::StatusChanged);
    });
}

pub fn preview_dir(dir: &path::Path) -> Result<PluginPreview, String> {
    let host = host().ok_or_else(|| "plugins are off".to_string())?;
    host.inspect_dir(dir).map_err(|err| err.to_string())
}

pub fn install_dir(dir: PathBuf, optional: Vec<String>) {
    in_background(move |host, shared| {
        match host.install_dir_with(&dir, &optional, true) {
            Ok(id) => log::info!("plugins: installed {id} from {}", dir.display()),
            Err(err) => shared.push(PluginEvent::InstallFailed(err.to_string())),
        }
        shared.push(PluginEvent::StatusChanged);
    });
}

pub fn menu_item_click(plugin_id: &str, menu: &str, item_id: &str, target: String) {
    let (id, menu, item) = (plugin_id.to_string(), menu.to_string(), item_id.to_string());
    in_background(move |host, _| {
        if let Err(err) = host.menu_item(&id, &menu, &item, &target, None) {
            log::error!("plugins: {id}'s menu item {item}: {err}");
        }
    });
}

pub fn slot_click(plugin_id: &str, slot: &str, item_id: &str, context: String) {
    let (id, slot, item) = (plugin_id.to_string(), slot.to_string(), item_id.to_string());
    in_background(move |host, _| {
        if let Err(err) = host.slot_action(&id, &slot, &item, &context) {
            log::error!("plugins: {id}'s slot item {item}: {err}");
        }
    });
}

pub fn render_view(plugin_id: &str, view_id: &str, context: &str) {
    let (id, view_id, context) = (
        plugin_id.to_string(),
        view_id.to_string(),
        context.to_string(),
    );
    in_background(move |host, shared| {
        let view = host
            .render_view(&id, &view_id, &context)
            .map_err(|err| err.to_string());
        shared.push(PluginEvent::ViewRendered {
            plugin: id,
            view_id,
            view,
        });
    });
}

pub fn view_action(
    plugin_id: &str,
    view_id: &str,
    action_id: String,
    value: Option<serde_json::Value>,
    context: &str,
) {
    let (id, view_id, context) = (
        plugin_id.to_string(),
        view_id.to_string(),
        context.to_string(),
    );
    in_background(move |host, shared| {
        match host.ui_action(&id, &view_id, &action_id, value, Some(&context)) {
            Ok(Some(view)) => shared.push(PluginEvent::ViewRendered {
                plugin: id,
                view_id,
                view: Ok(view),
            }),
            Ok(None) => {}
            Err(err) => shared.push(PluginEvent::ViewRendered {
                plugin: id,
                view_id,
                view: Err(err.to_string()),
            }),
        }
    });
}

pub fn plugin_image(plugin_id: &str, src: &str) -> Option<Vec<u8>> {
    host()?.image(plugin_id, src)
}
