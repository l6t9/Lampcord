use crate::css;
use crate::events;
use crate::id::Id;
use crate::manifest;
use crate::timers;
use crate::view;
use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, RwLock, Weak};
use std::sync;
use std::thread::Builder;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::time;
use wasmtime_wasi::p2;

use wasmtime::component::{HasSelf, Linker, Resource, ResourceTable};
use wasmtime::{Config, Engine, StoreLimits};
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

use crate::data::*;
use crate::services::{HostError, HostServices};
use crate::storage::KvStore;
use crate::view::{KNOWN_COMPONENTS, ViewBuilderState};

pub(crate) mod bindings {
    wasmtime::component::bindgen!({
        path: "../requiem-plugin-api/wit",
        world: "plugin",
        with: {
            "requiem:plugin/model.user": crate::runtime::UserRes,
            "requiem:plugin/model.member": crate::runtime::MemberRes,
            "requiem:plugin/model.role": crate::runtime::RoleRes,
            "requiem:plugin/model.guild": crate::runtime::GuildRes,
            "requiem:plugin/model.channel": crate::runtime::ChannelRes,
            "requiem:plugin/model.attachment": crate::runtime::AttachmentRes,
            "requiem:plugin/model.embed": crate::runtime::EmbedRes,
            "requiem:plugin/model.reaction": crate::runtime::ReactionRes,
            "requiem:plugin/model.message": crate::runtime::MessageRes,
            "requiem:plugin/model.activity": crate::runtime::ActivityRes,
            "requiem:plugin/model.presence": crate::runtime::PresenceRes,
            "requiem:plugin/model.voice-state": crate::runtime::VoiceStateRes,
            "requiem:plugin/model.relationship": crate::runtime::RelationshipRes,
            "requiem:plugin/model.read-state": crate::runtime::ReadStateRes,
            "requiem:plugin/model.profile": crate::runtime::ProfileRes,
            "requiem:plugin/drafts.outgoing-attachment": crate::runtime::OutgoingAttachmentRes,
            "requiem:plugin/drafts.outgoing-message": crate::runtime::OutgoingMessageRes,
            "requiem:plugin/drafts.outgoing-edit": crate::runtime::OutgoingEditRes,
            "requiem:plugin/drafts.incoming-message": crate::runtime::IncomingMessageRes,
            "requiem:plugin/drafts.notification": crate::runtime::NotificationRes,
            "requiem:plugin/drafts.event": crate::runtime::EventRes,
            "requiem:plugin/drafts.outgoing-presence": crate::runtime::OutgoingPresenceRes,
            "requiem:plugin/discord.response": crate::runtime::ResponseRes,
            "requiem:plugin/presence.activity-builder": crate::runtime::ActivityBuilderRes,
            "requiem:plugin/files.picked-file": crate::runtime::PickedFileRes,
            "requiem:plugin/net.http-response": crate::runtime::HttpResponseRes,
            "requiem:plugin/ui.view-builder": crate::runtime::ViewBuilderRes,
            "requiem:plugin/slots.slot-item": crate::runtime::SlotItemRes,
            "requiem:plugin/registry.command": crate::runtime::CommandRes,
            "requiem:plugin/registry.menu-item": crate::runtime::MenuItemRes,
            "requiem:plugin/registry.suggestion": crate::runtime::SuggestionRes,
            "requiem:plugin/registry.menu-state": crate::runtime::MenuStateRes,
        },
    });
}

use bindings::requiem::plugin as wit;
use wit::types::{Error as WitError, Json, Snowflake, Timestamp as WitTimestamp};

pub(crate) const EPOCH_TICK: Duration = Duration::from_millis(1);

pub(crate) fn engine() -> wasmtime::Result<Engine> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.epoch_interruption(true);
    let engine = Engine::new(&config)?;
    let ticker = engine.weak();
    Builder::new()
        .name("plugin-epoch".into())
        .spawn(move || {
            loop {
                thread::sleep(EPOCH_TICK);
                match ticker.upgrade() {
                    Some(engine) => engine.increment_epoch(),
                    None => return,
                }
            }
        })
        .expect("spawning the plugin epoch thread");
    Ok(engine)
}

pub(crate) fn linker(engine: &Engine) -> wasmtime::Result<Linker<State>> {
    let mut linker = Linker::new(engine);

    p2::add_to_linker_sync(&mut linker)?;
    bindings::Plugin::add_to_linker::<State, HasSelf<State>>(&mut linker, |state| state)?;
    Ok(linker)
}

pub(crate) fn wasi_ctx() -> WasiCtx {
    WasiCtx::builder()
        .allow_tcp(false)
        .allow_udp(false)
        .allow_ip_name_lookup(false)
        .build()
}

pub struct UserRes(pub(crate) UserData);
pub struct MemberRes(pub(crate) MemberData);
pub struct RoleRes(pub(crate) RoleData);
pub struct GuildRes(pub(crate) GuildData);
pub struct ChannelRes(pub(crate) ChannelData);
pub struct AttachmentRes(pub(crate) AttachmentData);
pub struct EmbedRes(pub(crate) EmbedData);
pub struct ReactionRes(pub(crate) ReactionData);
pub struct MessageRes(pub(crate) Arc<MessageData>);
pub struct ActivityRes(pub(crate) ActivityData);
pub struct PresenceRes(pub(crate) PresenceData);
pub struct VoiceStateRes(pub(crate) VoiceStateData);
pub struct RelationshipRes(pub(crate) RelationshipData);
pub struct ReadStateRes(pub(crate) ReadStateData);
pub struct ProfileRes(pub(crate) ProfileData);

pub struct OutgoingAttachmentRes {
    pub(crate) draft: Arc<Mutex<OutgoingDraft>>,
    pub(crate) index: usize,
}
pub struct OutgoingMessageRes(pub(crate) Arc<Mutex<OutgoingDraft>>);
pub struct OutgoingEditRes(pub(crate) EditDraft);
pub struct NotificationRes(pub(crate) NotificationDraft);
pub struct EventRes(pub(crate) Arc<EventData>);
pub struct OutgoingPresenceRes(pub(crate) PresenceDraft);
pub struct ResponseRes(pub(crate) u16, pub(crate) String);
pub struct ActivityBuilderRes(pub(crate) ActivityData);
pub struct PickedFileRes(pub(crate) String, pub(crate) Vec<u8>);
pub struct HttpResponseRes(pub(crate) HttpResponse);
pub struct ViewBuilderRes(pub(crate) ViewBuilderState);
pub struct SlotItemRes(pub(crate) SlotItemData);
pub struct CommandRes(pub(crate) CommandData);
pub struct MenuItemRes(pub(crate) MenuItemData);
pub struct SuggestionRes(pub(crate) SuggestionData);
pub struct MenuStateRes(pub(crate) MenuStateData);

pub struct IncomingMessageRes {
    pub(crate) message: Arc<MessageData>,
    pub(crate) previous: Option<Arc<MessageData>>,
    pub(crate) source: &'static str,
    pub(crate) changes: Arc<Mutex<IncomingChanges>>,
}

#[derive(Default)]
pub(crate) struct IncomingChanges {
    pub display_content: Option<String>,
    pub annotation: Option<String>,
    pub spans: Vec<Span>,
    pub decorations: Vec<(String, Option<String>)>,
    pub embed_states: Vec<(u32, String)>,
    pub attachment_states: Vec<(u32, String)>,
    pub author_display: Vec<(String, Option<String>)>,
    pub deleted_label: Option<String>,
}

#[derive(Default)]
pub(crate) struct PluginShared {
    pub registrations: Mutex<Registrations>,
    pub images: Mutex<HashMap<String, Vec<u8>>>,
    next_image: AtomicU32,
    next_socket: AtomicU32,

    pub gesture: AtomicBool,
}

pub(crate) struct State {
    pub plugin_id: String,
    pub package_dir: PathBuf,
    pub wasi: WasiCtx,
    pub table: ResourceTable,
    pub limits: StoreLimits,
    pub kv: KvStore,
    pub granted: Arc<RwLock<BTreeSet<String>>>,
    pub services: Arc<dyn HostServices>,
    pub shared: Arc<PluginShared>,
    pub host: Weak<crate::PluginHost>,
}

impl WasiView for State {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

fn wit_error(err: HostError) -> WitError {
    WitError {
        code: err.code,
        message: err.message,
    }
}

fn invalid(message: impl Into<String>) -> WitError {
    wit_error(HostError::invalid(message))
}

impl State {
    fn has(&self, permission: &str) -> bool {
        self.granted
            .read()
            .expect("grants lock poisoned")
            .contains(permission)
    }

    fn need(&self, permission: &str) -> Result<(), WitError> {
        if self.has(permission) {
            Ok(())
        } else {
            Err(wit_error(HostError::denied(permission)))
        }
    }

    fn gesture(&self) -> bool {
        self.shared.gesture.load(Ordering::Acquire)
    }

    fn get<T: 'static>(&self, resource: &Resource<T>) -> &T {
        self.table
            .get(resource)
            .expect("plugin passed a stale resource handle")
    }

    fn get_mut<T: 'static>(&mut self, resource: &Resource<T>) -> &mut T {
        self.table
            .get_mut(resource)
            .expect("plugin passed a stale resource handle")
    }

    fn push<T: Send + 'static>(&mut self, value: T) -> Resource<T> {
        self.table
            .push(value)
            .expect("plugin resource table is full")
    }

    fn take<T: 'static>(&mut self, resource: Resource<T>) -> T {
        self.table
            .delete(resource)
            .expect("plugin passed a stale resource handle")
    }

    fn drop_res<T: 'static>(&mut self, resource: Resource<T>) -> wasmtime::Result<()> {
        self.table.delete(resource)?;
        Ok(())
    }

    fn id(&self) -> String {
        self.plugin_id.clone()
    }

    fn registrations(&self) -> sync::MutexGuard<'_, Registrations> {
        self.shared
            .registrations
            .lock()
            .expect("registrations lock poisoned")
    }

    fn registrations_changed(&self) {
        self.services.status_changed(&self.plugin_id);
    }

    fn check_url(&self, url: &str) -> Result<(), WitError> {
        let Some(host) = url_host(url) else {
            return Err(invalid(format!("not an http(s)/ws(s) URL: {url}")));
        };
        if self.services.is_discord_host(&host) {
            return Err(wit_error(HostError::denied(
                "network (Discord's own API is off limits; use `discord.request`)",
            )));
        }
        if matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1" | "[::1]") {
            return self.need("network.localhost");
        }
        let granted = self.granted.read().expect("grants lock poisoned");
        let allowed = granted
            .iter()
            .filter_map(|p| p.strip_prefix("network:"))
            .any(|pattern| host_matches(pattern, &host));
        if allowed {
            Ok(())
        } else {
            Err(wit_error(HostError::denied(&format!("network:{host}"))))
        }
    }

    fn resolve_source(&self, source: &str) -> Result<String, WitError> {
        if let Some(path) = source.strip_prefix("plugin:") {
            if path
                .split(['/', '\\'])
                .any(|part| part == ".." || part.is_empty())
            {
                return Err(invalid(format!("bad asset path: {path}")));
            }
            let full = self.package_dir.join("assets").join(path);
            return if full.is_file() {
                Ok(format!("plugin:{}", full.display()))
            } else {
                Err(wit_error(HostError::not_found(format!("no asset {path}"))))
            };
        }
        if source.starts_with("https://") {
            self.check_url(source)?;
        } else if !source.starts_with("builtin:") {
            return Err(invalid(format!(
                "sound sources are builtin:, plugin: or https: ({source})"
            )));
        }
        Ok(source.to_string())
    }

    fn overlay_message(&self, update: OverlayUpdate) -> Result<(), WitError> {
        self.need("messages.intercept-incoming")?;
        self.services.overlay(&self.plugin_id, update);
        Ok(())
    }
}

pub(crate) fn url_host(url: &str) -> Option<String> {
    let rest = ["https://", "http://", "wss://", "ws://"]
        .iter()
        .find_map(|scheme| url.strip_prefix(scheme))?;
    let authority = rest.split(['/', '?', '#']).next()?;
    let authority = authority.rsplit_once('@').map_or(authority, |(_, a)| a);
    let host = if authority.starts_with('[') {
        authority.split_once(']').map(|(h, _)| format!("{h}]"))?
    } else {
        authority.split(':').next()?.to_string()
    };
    (!host.is_empty()).then(|| host.to_ascii_lowercase())
}

pub(crate) fn host_matches(pattern: &str, host: &str) -> bool {
    let pattern = pattern
        .split(':')
        .next()
        .unwrap_or(pattern)
        .to_ascii_lowercase();
    match pattern.strip_prefix("*.") {
        _ if pattern == "*" => true,
        Some(base) => host.ends_with(&format!(".{base}")),
        None => host == pattern,
    }
}

pub(crate) fn route_matches(permission: &str, method: &str, path: &str) -> bool {
    let Some(route) = permission.strip_prefix("discord-api:") else {
        return false;
    };
    let Some((allowed_method, pattern)) = route.split_once(' ') else {
        return false;
    };
    if !allowed_method.eq_ignore_ascii_case(method) {
        return false;
    }
    let path = path.split('?').next().unwrap_or(path);
    let (pattern, path): (Vec<_>, Vec<_>) = (
        pattern.trim_matches('/').split('/').collect(),
        path.trim_matches('/').split('/').collect(),
    );
    pattern.len() == path.len()
        && pattern
            .iter()
            .zip(&path)
            .all(|(p, s)| *p == "*" && !s.is_empty() || p == s)
}

fn size_url(url: &Option<String>, size: u32) -> Option<String> {
    let size = size.clamp(16, 4096).next_power_of_two();
    url.as_ref().map(|url| {
        format!(
            "{url}{}size={size}",
            if url.contains('?') { '&' } else { '?' }
        )
    })
}

fn snowflake(json: &serde_json::Value, key: &str) -> Option<u64> {
    let value = json.get(key)?;
    value.as_u64().or_else(|| value.as_str()?.parse().ok())
}

fn sid_opt<T>(id: Option<Id<T>>) -> Option<Snowflake> {
    id.map(|id| id.get())
}

fn hid<T>(id: Snowflake) -> Id<T> {
    Id::new(id)
}

fn hid_opt<T>(id: Option<Snowflake>) -> Option<Id<T>> {
    id.map(Id::new)
}

fn sids<T: Copy>(ids: &[Id<T>]) -> Vec<Snowflake> {
    ids.iter().copied().map(|id| id.get()).collect()
}

impl wit::types::Host for State {}

impl wit::log::Host for State {
    fn log(&mut self, level: wit::log::Level, message: String) {
        let level = match level {
            wit::log::Level::Trace => LogLevel::Trace,
            wit::log::Level::Debug => LogLevel::Debug,
            wit::log::Level::Info => LogLevel::Info,
            wit::log::Level::Warn => LogLevel::Warn,
            wit::log::Level::Error => LogLevel::Error,
        };
        self.services.log(&self.plugin_id, level, &message);
    }
}

impl wit::storage::Host for State {
    fn get(&mut self, key: String) -> Option<Vec<u8>> {
        self.kv.get(&key)
    }
    fn set(&mut self, key: String, value: Vec<u8>) -> Result<(), WitError> {
        self.kv.set(key, value).map_err(|(code, message)| WitError {
            code: code.into(),
            message,
        })
    }
    fn delete(&mut self, key: String) {
        self.kv.delete(&key);
    }
    fn keys(&mut self, prefix: String) -> Vec<String> {
        self.kv.keys(&prefix)
    }
    fn usage(&mut self) -> (u64, u64) {
        self.kv.usage()
    }
    fn get_secret(&mut self, key: String) -> Result<Option<String>, WitError> {
        self.services
            .secret_get(&self.plugin_id, &key)
            .map_err(wit_error)
    }
    fn set_secret(&mut self, key: String, value: String) -> Result<(), WitError> {
        self.services
            .secret_set(&self.plugin_id, &key, &value)
            .map_err(wit_error)
    }
    fn delete_secret(&mut self, key: String) -> Result<(), WitError> {
        self.services
            .secret_delete(&self.plugin_id, &key)
            .map_err(wit_error)
    }
}

impl wit::timers::Host for State {
    fn set(&mut self, id: String, delay_ms: u64, repeat: bool) {
        if let Some(host) = self.host.upgrade() {
            host.timers.set(
                &self.plugin_id,
                id,
                Duration::from_millis(delay_ms.max(if repeat { 50 } else { 0 })),
                repeat,
            );
        }
    }
    fn clear(&mut self, id: String) {
        if let Some(host) = self.host.upgrade() {
            host.timers.clear(&self.plugin_id, &id);
        }
    }
    fn now(&mut self) -> i64 {
        SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or_default()
    }
    fn monotonic(&mut self) -> u64 {
        timers::monotonic_ms()
    }
}

macro_rules! getters {
    ($trait:path, $res:ty { $($name:ident -> $ret:ty = |$d:ident| $body:expr;)* }) => {
        impl $trait for State {
            $(fn $name(&mut self, this: Resource<$res>) -> $ret {
                let $d = &self.get(&this).0;
                $body
            })*
            fn drop(&mut self, this: Resource<$res>) -> wasmtime::Result<()> {
                self.drop_res(this)
            }
        }
    };
}

impl wit::model::Host for State {}

impl wit::model::HostUser for State {
    fn id(&mut self, this: Resource<UserRes>) -> Snowflake {
        self.get(&this).0.id.get()
    }
    fn username(&mut self, this: Resource<UserRes>) -> String {
        self.get(&this).0.username.clone()
    }
    fn global_name(&mut self, this: Resource<UserRes>) -> Option<String> {
        self.get(&this).0.global_name.clone()
    }
    fn display_name(&mut self, this: Resource<UserRes>) -> String {
        self.get(&this).0.display_name()
    }
    fn avatar_url(&mut self, this: Resource<UserRes>, size: u32) -> Option<String> {
        size_url(&self.get(&this).0.avatar_url, size)
    }
    fn banner_url(&mut self, this: Resource<UserRes>, size: u32) -> Option<String> {
        size_url(&self.get(&this).0.banner_url, size)
    }
    fn accent_color(&mut self, this: Resource<UserRes>) -> Option<u32> {
        self.get(&this).0.accent_color
    }
    fn is_bot(&mut self, this: Resource<UserRes>) -> bool {
        self.get(&this).0.bot
    }
    fn is_system(&mut self, this: Resource<UserRes>) -> bool {
        self.get(&this).0.system
    }
    fn badges(&mut self, this: Resource<UserRes>) -> Vec<String> {
        self.get(&this).0.badges.clone()
    }
    fn drop(&mut self, this: Resource<UserRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

impl wit::model::HostMember for State {
    fn guild_id(&mut self, this: Resource<MemberRes>) -> Snowflake {
        self.get(&this).0.guild_id.get()
    }
    fn user(&mut self, this: Resource<MemberRes>) -> Resource<UserRes> {
        let user = self.get(&this).0.user.clone();
        self.push(UserRes(user))
    }
    fn nick(&mut self, this: Resource<MemberRes>) -> Option<String> {
        self.get(&this).0.nick.clone()
    }
    fn display_name(&mut self, this: Resource<MemberRes>) -> String {
        let member = &self.get(&this).0;
        member
            .nick
            .clone()
            .unwrap_or_else(|| member.user.display_name())
    }
    fn role_ids(&mut self, this: Resource<MemberRes>) -> Vec<Snowflake> {
        sids(&self.get(&this).0.role_ids)
    }
    fn color(&mut self, this: Resource<MemberRes>) -> Option<u32> {
        self.get(&this).0.color
    }
    fn joined_at(&mut self, this: Resource<MemberRes>) -> Option<WitTimestamp> {
        self.get(&this).0.joined_at
    }
    fn timeout_until(&mut self, this: Resource<MemberRes>) -> Option<WitTimestamp> {
        self.get(&this).0.timeout_until
    }
    fn guild_avatar_url(&mut self, this: Resource<MemberRes>, size: u32) -> Option<String> {
        size_url(&self.get(&this).0.guild_avatar_url, size)
    }
    fn drop(&mut self, this: Resource<MemberRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

getters!(wit::model::HostRole, RoleRes {
    id -> Snowflake = |d| d.id.get();
    guild_id -> Snowflake = |d| d.guild_id.get();
    name -> String = |d| d.name.clone();
    color -> Option<u32> = |d| d.color;
    position -> i32 = |d| d.position;
    hoist -> bool = |d| d.hoist;
    mentionable -> bool = |d| d.mentionable;
    icon_url -> Option<String> = |d| d.icon_url.clone();
    permissions -> Vec<String> = |d| d.permissions.clone();
});

impl wit::model::HostGuild for State {
    fn id(&mut self, this: Resource<GuildRes>) -> Snowflake {
        self.get(&this).0.id.get()
    }
    fn name(&mut self, this: Resource<GuildRes>) -> String {
        self.get(&this).0.name.clone()
    }
    fn icon_url(&mut self, this: Resource<GuildRes>, size: u32) -> Option<String> {
        size_url(&self.get(&this).0.icon_url, size)
    }
    fn banner_url(&mut self, this: Resource<GuildRes>, size: u32) -> Option<String> {
        size_url(&self.get(&this).0.banner_url, size)
    }
    fn owner_id(&mut self, this: Resource<GuildRes>) -> Snowflake {
        self.get(&this).0.owner_id.get()
    }
    fn member_count(&mut self, this: Resource<GuildRes>) -> Option<u32> {
        self.get(&this).0.member_count
    }
    fn online_count(&mut self, this: Resource<GuildRes>) -> Option<u32> {
        if !self.has("presence.read") {
            return None;
        }
        self.get(&this).0.online_count
    }
    fn features(&mut self, this: Resource<GuildRes>) -> Vec<String> {
        self.get(&this).0.features.clone()
    }
    fn description(&mut self, this: Resource<GuildRes>) -> Option<String> {
        self.get(&this).0.description.clone()
    }
    fn boost_tier(&mut self, this: Resource<GuildRes>) -> u32 {
        self.get(&this).0.boost_tier
    }
    fn boost_count(&mut self, this: Resource<GuildRes>) -> u32 {
        self.get(&this).0.boost_count
    }
    fn splash_url(&mut self, this: Resource<GuildRes>, size: u32) -> Option<String> {
        size_url(&self.get(&this).0.splash_url, size)
    }
    fn discovery_splash_url(&mut self, this: Resource<GuildRes>, size: u32) -> Option<String> {
        size_url(&self.get(&this).0.discovery_splash_url, size)
    }
    fn emojis(&mut self, this: Resource<GuildRes>, size: u32) -> Vec<(String, String)> {
        self.get(&this)
            .0
            .emojis
            .iter()
            .filter_map(|(name, url)| Some((name.clone(), size_url(&Some(url.clone()), size)?)))
            .collect()
    }
    fn drop(&mut self, this: Resource<GuildRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

impl wit::model::HostChannel for State {
    fn id(&mut self, this: Resource<ChannelRes>) -> Snowflake {
        self.get(&this).0.id.get()
    }
    fn guild_id(&mut self, this: Resource<ChannelRes>) -> Option<Snowflake> {
        sid_opt(self.get(&this).0.guild_id)
    }
    fn name(&mut self, this: Resource<ChannelRes>) -> Option<String> {
        self.get(&this).0.name.clone()
    }
    fn kind(&mut self, this: Resource<ChannelRes>) -> String {
        self.get(&this).0.kind.clone()
    }
    fn parent_id(&mut self, this: Resource<ChannelRes>) -> Option<Snowflake> {
        sid_opt(self.get(&this).0.parent_id)
    }
    fn position(&mut self, this: Resource<ChannelRes>) -> Option<i32> {
        self.get(&this).0.position
    }
    fn topic(&mut self, this: Resource<ChannelRes>) -> Option<String> {
        self.get(&this).0.topic.clone()
    }
    fn nsfw(&mut self, this: Resource<ChannelRes>) -> bool {
        self.get(&this).0.nsfw
    }
    fn recipient_ids(&mut self, this: Resource<ChannelRes>) -> Vec<Snowflake> {
        sids(&self.get(&this).0.recipient_ids)
    }
    fn last_message_id(&mut self, this: Resource<ChannelRes>) -> Option<Snowflake> {
        sid_opt(self.get(&this).0.last_message_id)
    }
    fn can(&mut self, this: Resource<ChannelRes>, permission: String) -> bool {
        self.get(&this)
            .0
            .my_permissions
            .iter()
            .any(|p| *p == permission || p == "administrator")
    }
    fn drop(&mut self, this: Resource<ChannelRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

getters!(wit::model::HostAttachment, AttachmentRes {
    id -> Snowflake = |d| d.id.get();
    filename -> String = |d| d.filename.clone();
    content_type -> Option<String> = |d| d.content_type.clone();
    size -> u64 = |d| d.size;
    url -> String = |d| d.url.clone();
    width -> Option<u32> = |d| d.width;
    height -> Option<u32> = |d| d.height;
    description -> Option<String> = |d| d.description.clone();
    spoiler -> bool = |d| d.spoiler;
    duration_secs -> Option<f32> = |d| d.duration_secs;
});

getters!(wit::model::HostEmbed, EmbedRes {
    kind -> String = |d| d.kind.clone();
    title -> Option<String> = |d| d.title.clone();
    description -> Option<String> = |d| d.description.clone();
    url -> Option<String> = |d| d.url.clone();
    color -> Option<u32> = |d| d.color;
    author_name -> Option<String> = |d| d.author_name.clone();
    provider_name -> Option<String> = |d| d.provider_name.clone();
    footer_text -> Option<String> = |d| d.footer_text.clone();
    image_url -> Option<String> = |d| d.image_url.clone();
    thumbnail_url -> Option<String> = |d| d.thumbnail_url.clone();
    video_url -> Option<String> = |d| d.video_url.clone();
    fields -> Vec<(String, String, bool)> = |d| d.fields.clone();
});

getters!(wit::model::HostReaction, ReactionRes {
    emoji -> String = |d| d.emoji.clone();
    count -> u32 = |d| d.count;
    me -> bool = |d| d.me;
    burst -> bool = |d| d.burst;
});

impl wit::model::HostMessage for State {
    fn id(&mut self, this: Resource<MessageRes>) -> Snowflake {
        self.get(&this).0.id.get()
    }
    fn channel_id(&mut self, this: Resource<MessageRes>) -> Snowflake {
        self.get(&this).0.channel_id.get()
    }
    fn guild_id(&mut self, this: Resource<MessageRes>) -> Option<Snowflake> {
        sid_opt(self.get(&this).0.guild_id)
    }
    fn author(&mut self, this: Resource<MessageRes>) -> Resource<UserRes> {
        let author = self.get(&this).0.author.clone();
        self.push(UserRes(author))
    }
    fn member(&mut self, this: Resource<MessageRes>) -> Option<Resource<MemberRes>> {
        let member = self.get(&this).0.member.clone();
        member.map(|m| self.push(MemberRes(m)))
    }
    fn content(&mut self, this: Resource<MessageRes>) -> String {
        self.get(&this).0.content.clone()
    }
    fn timestamp(&mut self, this: Resource<MessageRes>) -> WitTimestamp {
        self.get(&this).0.timestamp
    }
    fn edited_timestamp(&mut self, this: Resource<MessageRes>) -> Option<WitTimestamp> {
        self.get(&this).0.edited_timestamp
    }
    fn kind(&mut self, this: Resource<MessageRes>) -> String {
        self.get(&this).0.kind.clone()
    }
    fn flags(&mut self, this: Resource<MessageRes>) -> Vec<String> {
        self.get(&this).0.flags.clone()
    }
    fn attachments(&mut self, this: Resource<MessageRes>) -> Vec<Resource<AttachmentRes>> {
        let list = self.get(&this).0.attachments.clone();
        list.into_iter()
            .map(|a| self.push(AttachmentRes(a)))
            .collect()
    }
    fn embeds(&mut self, this: Resource<MessageRes>) -> Vec<Resource<EmbedRes>> {
        let list = self.get(&this).0.embeds.clone();
        list.into_iter().map(|e| self.push(EmbedRes(e))).collect()
    }
    fn reactions(&mut self, this: Resource<MessageRes>) -> Vec<Resource<ReactionRes>> {
        let list = self.get(&this).0.reactions.clone();
        list.into_iter()
            .map(|r| self.push(ReactionRes(r)))
            .collect()
    }
    fn stickers(&mut self, this: Resource<MessageRes>) -> Vec<String> {
        self.get(&this).0.stickers.clone()
    }
    fn mention_ids(&mut self, this: Resource<MessageRes>) -> Vec<Snowflake> {
        sids(&self.get(&this).0.mention_ids)
    }
    fn mention_role_ids(&mut self, this: Resource<MessageRes>) -> Vec<Snowflake> {
        sids(&self.get(&this).0.mention_role_ids)
    }
    fn mentions_everyone(&mut self, this: Resource<MessageRes>) -> bool {
        self.get(&this).0.mentions_everyone
    }
    fn mentions_me(&mut self, this: Resource<MessageRes>) -> bool {
        self.get(&this).0.mentions_me
    }
    fn reply_to(&mut self, this: Resource<MessageRes>) -> Option<Snowflake> {
        sid_opt(self.get(&this).0.reply_to)
    }
    fn referenced(&mut self, this: Resource<MessageRes>) -> Option<Resource<MessageRes>> {
        let referenced = self.get(&this).0.referenced.clone();
        referenced.map(|m| self.push(MessageRes(Arc::new(*m))))
    }
    fn pinned(&mut self, this: Resource<MessageRes>) -> bool {
        self.get(&this).0.pinned
    }
    fn webhook_id(&mut self, this: Resource<MessageRes>) -> Option<Snowflake> {
        sid_opt(self.get(&this).0.webhook_id)
    }
    fn raw_json(&mut self, this: Resource<MessageRes>) -> Option<Json> {
        if !self.has("data.raw") {
            return None;
        }
        self.get(&this).0.raw_json.clone()
    }
    fn drop(&mut self, this: Resource<MessageRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

getters!(wit::model::HostActivity, ActivityRes {
    kind -> String = |d| d.kind.clone();
    name -> String = |d| d.name.clone();
    details -> Option<String> = |d| d.details.clone();
    state -> Option<String> = |d| d.state.clone();
    application_id -> Option<Snowflake> = |d| sid_opt(d.application_id);
    url -> Option<String> = |d| d.url.clone();
    start -> Option<WitTimestamp> = |d| d.start;
    end -> Option<WitTimestamp> = |d| d.end;
    large_image_url -> Option<String> = |d| d.large_image.clone();
    small_image_url -> Option<String> = |d| d.small_image.clone();
    emoji -> Option<String> = |d| d.emoji.clone();
});

impl wit::model::HostPresence for State {
    fn user_id(&mut self, this: Resource<PresenceRes>) -> Snowflake {
        self.get(&this).0.user_id.get()
    }
    fn status(&mut self, this: Resource<PresenceRes>) -> String {
        self.get(&this).0.status.clone()
    }
    fn platforms(&mut self, this: Resource<PresenceRes>) -> Vec<String> {
        self.get(&this).0.platforms.clone()
    }
    fn activities(&mut self, this: Resource<PresenceRes>) -> Vec<Resource<ActivityRes>> {
        let list = self.get(&this).0.activities.clone();
        list.into_iter()
            .map(|a| self.push(ActivityRes(a)))
            .collect()
    }
    fn drop(&mut self, this: Resource<PresenceRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

getters!(wit::model::HostVoiceState, VoiceStateRes {
    user_id -> Snowflake = |d| d.user_id.get();
    channel_id -> Option<Snowflake> = |d| sid_opt(d.channel_id);
    guild_id -> Option<Snowflake> = |d| sid_opt(d.guild_id);
    self_mute -> bool = |d| d.self_mute;
    self_deaf -> bool = |d| d.self_deaf;
    server_mute -> bool = |d| d.server_mute;
    server_deaf -> bool = |d| d.server_deaf;
    streaming -> bool = |d| d.streaming;
    video -> bool = |d| d.video;
    suppressed -> bool = |d| d.suppressed;
    joined_at -> Option<WitTimestamp> = |d| d.joined_at;
});

impl wit::model::HostRelationship for State {
    fn user(&mut self, this: Resource<RelationshipRes>) -> Resource<UserRes> {
        let user = self.get(&this).0.user.clone();
        self.push(UserRes(user))
    }
    fn kind(&mut self, this: Resource<RelationshipRes>) -> String {
        self.get(&this).0.kind.clone()
    }
    fn nickname(&mut self, this: Resource<RelationshipRes>) -> Option<String> {
        self.get(&this).0.nickname.clone()
    }
    fn since(&mut self, this: Resource<RelationshipRes>) -> Option<WitTimestamp> {
        self.get(&this).0.since
    }
    fn drop(&mut self, this: Resource<RelationshipRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

getters!(wit::model::HostReadState, ReadStateRes {
    channel_id -> Snowflake = |d| d.channel_id.get();
    unread -> bool = |d| d.unread;
    mention_count -> u32 = |d| d.mention_count;
    last_read_message_id -> Option<Snowflake> = |d| sid_opt(d.last_read_message_id);
    notification_level -> String = |d| d.notification_level.clone();
    muted -> bool = |d| d.muted;
});

impl wit::model::HostProfile for State {
    fn user(&mut self, this: Resource<ProfileRes>) -> Resource<UserRes> {
        let user = self.get(&this).0.user.clone();
        self.push(UserRes(user))
    }
    fn bio(&mut self, this: Resource<ProfileRes>) -> Option<String> {
        self.get(&this).0.bio.clone()
    }
    fn pronouns(&mut self, this: Resource<ProfileRes>) -> Option<String> {
        self.get(&this).0.pronouns.clone()
    }
    fn connections(&mut self, this: Resource<ProfileRes>) -> Vec<(String, String, bool)> {
        self.get(&this).0.connections.clone()
    }
    fn member(&mut self, this: Resource<ProfileRes>) -> Option<Resource<MemberRes>> {
        let member = self.get(&this).0.member.clone();
        member.map(|m| self.push(MemberRes(m)))
    }
    fn mutual_guild_ids(&mut self, this: Resource<ProfileRes>) -> Vec<Snowflake> {
        sids(&self.get(&this).0.mutual_guild_ids)
    }
    fn mutual_friend_ids(&mut self, this: Resource<ProfileRes>) -> Vec<Snowflake> {
        sids(&self.get(&this).0.mutual_friend_ids)
    }
    fn premium_since(&mut self, this: Resource<ProfileRes>) -> Option<WitTimestamp> {
        self.get(&this).0.premium_since
    }
    fn drop(&mut self, this: Resource<ProfileRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

impl State {
    fn users(&mut self, list: Option<UserData>) -> Option<Resource<UserRes>> {
        list.map(|u| self.push(UserRes(u)))
    }
    fn message_list(&mut self, list: Vec<MessageData>) -> Vec<Resource<MessageRes>> {
        list.into_iter()
            .map(|m| self.push(MessageRes(Arc::new(m))))
            .collect()
    }
}

impl wit::client::Host for State {
    fn info(&mut self) -> Json {
        let mut info: serde_json::Value = serde_json::from_str(&self.services.client_info())
            .unwrap_or_else(|_| serde_json::json!({}));
        info["api"] = format!(
            "{}.{}",
            manifest::API_VERSION.0,
            manifest::API_VERSION.1
        )
        .into();
        info.to_string()
    }
    fn current_user(&mut self) -> Option<Resource<UserRes>> {
        let user = self.services.current_user();
        self.users(user)
    }
    fn selected_guild(&mut self) -> Option<Snowflake> {
        sid_opt(self.services.selected_guild())
    }
    fn selected_channel(&mut self) -> Option<Snowflake> {
        sid_opt(self.services.selected_channel())
    }
    fn get_user(&mut self, id: Snowflake) -> Option<Resource<UserRes>> {
        let user = self.services.user(hid(id));
        self.users(user)
    }
    fn get_guild(&mut self, id: Snowflake) -> Option<Resource<GuildRes>> {
        if !self.has("guilds.read") {
            return None;
        }
        self.services.guild(hid(id)).map(|g| self.push(GuildRes(g)))
    }
    fn guilds(&mut self) -> Vec<Resource<GuildRes>> {
        if !self.has("guilds.read") {
            return Vec::new();
        }
        self.services
            .guilds()
            .into_iter()
            .map(|g| self.push(GuildRes(g)))
            .collect()
    }
    fn guild_folders(&mut self) -> Json {
        if !self.has("guilds.read") {
            return "[]".to_string();
        }
        self.services.guild_folders()
    }
    fn get_channel(&mut self, id: Snowflake) -> Option<Resource<ChannelRes>> {
        if !self.has("guilds.read") {
            return None;
        }
        self.services.channel(hid(id)).map(|c| self.push(ChannelRes(c)))
    }
    fn channels(&mut self, guild_id: Snowflake) -> Vec<Resource<ChannelRes>> {
        if !self.has("guilds.read") {
            return Vec::new();
        }
        self.services
            .channels(hid(guild_id))
            .into_iter()
            .map(|c| self.push(ChannelRes(c)))
            .collect()
    }
    fn private_channels(&mut self) -> Vec<Resource<ChannelRes>> {
        if !self.has("guilds.read") {
            return Vec::new();
        }
        self.services
            .private_channels()
            .into_iter()
            .map(|c| self.push(ChannelRes(c)))
            .collect()
    }
    fn get_role(&mut self, guild_id: Snowflake, id: Snowflake) -> Option<Resource<RoleRes>> {
        if !self.has("guilds.read") {
            return None;
        }
        self.services
            .role(hid(guild_id), hid(id))
            .map(|r| self.push(RoleRes(r)))
    }
    fn roles(&mut self, guild_id: Snowflake) -> Vec<Resource<RoleRes>> {
        if !self.has("guilds.read") {
            return Vec::new();
        }
        self.services
            .roles(hid(guild_id))
            .into_iter()
            .map(|r| self.push(RoleRes(r)))
            .collect()
    }
    fn get_member(
        &mut self,
        guild_id: Snowflake,
        user_id: Snowflake,
    ) -> Option<Resource<MemberRes>> {
        if !self.has("members.read") {
            return None;
        }
        self.services
            .member(hid(guild_id), hid(user_id))
            .map(|m| self.push(MemberRes(m)))
    }
    fn members(&mut self, guild_id: Snowflake) -> Vec<Resource<MemberRes>> {
        if !self.has("members.read") {
            return Vec::new();
        }
        self.services
            .members(hid(guild_id))
            .into_iter()
            .map(|m| self.push(MemberRes(m)))
            .collect()
    }
    fn user_can(
        &mut self,
        channel_id: Snowflake,
        user_id: Snowflake,
        permission: String,
    ) -> Result<bool, WitError> {
        self.need("members.read")?;
        self.services
            .user_can(hid(channel_id), hid(user_id), &permission)
            .map_err(wit_error)
    }
    fn messages(&mut self, channel_id: Snowflake) -> Vec<Resource<MessageRes>> {
        if !self.has("messages.read") {
            return Vec::new();
        }
        let list = self.services.messages(hid(channel_id));
        self.message_list(list)
    }
    fn get_message(
        &mut self,
        channel_id: Snowflake,
        id: Snowflake,
    ) -> Option<Resource<MessageRes>> {
        if !self.has("messages.read") {
            return None;
        }
        self.services
            .message(hid(channel_id), hid(id))
            .map(|m| self.push(MessageRes(Arc::new(m))))
    }
    fn fetch_messages(
        &mut self,
        channel_id: Snowflake,
        anchor: Option<Snowflake>,
        direction: String,
        limit: u32,
    ) -> Result<Vec<Resource<MessageRes>>, WitError> {
        self.need("messages.read")?;
        if !matches!(direction.as_str(), "before" | "after" | "around") {
            return Err(invalid("direction is \"before\", \"after\" or \"around\""));
        }
        let list = self
            .services
            .fetch_messages(hid(channel_id), hid_opt(anchor), &direction, limit.clamp(1, 100))
            .map_err(wit_error)?;
        Ok(self.message_list(list))
    }
    fn fetch_message(
        &mut self,
        channel_id: Snowflake,
        id: Snowflake,
    ) -> Result<Resource<MessageRes>, WitError> {
        self.need("messages.read")?;
        let message = self
            .services
            .fetch_message(hid(channel_id), hid(id))
            .map_err(wit_error)?;
        Ok(self.push(MessageRes(Arc::new(message))))
    }
    fn search_messages(
        &mut self,
        scope_id: Snowflake,
        query: Json,
    ) -> Result<Vec<Resource<MessageRes>>, WitError> {
        self.need("messages.read")?;
        let list = self
            .services
            .search_messages(scope_id, &query)
            .map_err(wit_error)?;
        Ok(self.message_list(list))
    }
    fn get_presence(&mut self, user_id: Snowflake) -> Option<Resource<PresenceRes>> {
        if !self.has("presence.read") {
            return None;
        }
        self.services
            .presence(hid(user_id))
            .map(|p| self.push(PresenceRes(p)))
    }
    fn get_voice_state(&mut self, user_id: Snowflake) -> Option<Resource<VoiceStateRes>> {
        if !self.has("voice.read") {
            return None;
        }
        self.services
            .voice_state(hid(user_id))
            .map(|v| self.push(VoiceStateRes(v)))
    }
    fn voice_states(&mut self, channel_id: Snowflake) -> Vec<Resource<VoiceStateRes>> {
        if !self.has("voice.read") {
            return Vec::new();
        }
        self.services
            .voice_states(hid(channel_id))
            .into_iter()
            .map(|v| self.push(VoiceStateRes(v)))
            .collect()
    }
    fn relationships(&mut self) -> Vec<Resource<RelationshipRes>> {
        if !self.has("relationships.read") {
            return Vec::new();
        }
        self.services
            .relationships()
            .into_iter()
            .map(|r| self.push(RelationshipRes(r)))
            .collect()
    }
    fn typing(&mut self, channel_id: Snowflake) -> Vec<Snowflake> {
        if !self.has("presence.read") {
            return Vec::new();
        }
        sids(&self.services.typing(hid(channel_id)))
    }
    fn get_read_state(&mut self, channel_id: Snowflake) -> Option<Resource<ReadStateRes>> {
        if !self.has("read-states.read") {
            return None;
        }
        self.services
            .read_state(hid(channel_id))
            .map(|r| self.push(ReadStateRes(r)))
    }
    fn fetch_profile(
        &mut self,
        user_id: Snowflake,
        guild_id: Option<Snowflake>,
    ) -> Result<Resource<ProfileRes>, WitError> {
        self.need("members.read")?;
        let profile = self
            .services
            .fetch_profile(hid(user_id), hid_opt(guild_id))
            .map_err(wit_error)?;
        Ok(self.push(ProfileRes(profile)))
    }
    fn fetch_members(
        &mut self,
        guild_id: Snowflake,
        query: String,
        limit: u32,
    ) -> Result<Vec<Resource<MemberRes>>, WitError> {
        self.need("members.read")?;
        let list = self
            .services
            .fetch_members(hid(guild_id), &query, limit.clamp(1, 100))
            .map_err(wit_error)?;
        Ok(list.into_iter().map(|m| self.push(MemberRes(m))).collect())
    }
}

fn send_options(options: &str) -> Result<SendOptions, WitError> {
    if options.trim().is_empty() {
        return Ok(SendOptions::default());
    }
    let json: serde_json::Value = serde_json::from_str(options)
        .map_err(|err| invalid(format!("options aren't JSON: {err}")))?;
    let strings = |key: &str| {
        json.get(key)
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|s| s.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };
    Ok(SendOptions {
        reply_to: snowflake(&json, "reply-to"),
        reply_mention: json
            .get("reply-mention")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        flags: strings("flags"),
        sticker_ids: json
            .get("sticker-ids")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_u64().or_else(|| v.as_str()?.parse().ok()))
                    .collect()
            })
            .unwrap_or_default(),
    })
}

impl wit::actions::Host for State {
    fn send_message(
        &mut self,
        channel_id: Snowflake,
        content: String,
        options: Json,
    ) -> Result<Snowflake, WitError> {
        if !self.gesture() {
            self.need("messages.send")?;
        }
        let options = send_options(&options)?;
        self.services
            .send_message(&self.id(), hid(channel_id), &content, Vec::new(), options)
            .map(|id| id.get())
            .map_err(wit_error)
    }
    fn send_files(
        &mut self,
        channel_id: Snowflake,
        content: String,
        files: Vec<(String, Vec<u8>)>,
        options: Json,
    ) -> Result<Snowflake, WitError> {
        if !self.gesture() {
            self.need("messages.send")?;
        }
        let options = send_options(&options)?;
        self.services
            .send_message(&self.id(), hid(channel_id), &content, files, options)
            .map(|id| id.get())
            .map_err(wit_error)
    }
    fn edit_message(
        &mut self,
        channel_id: Snowflake,
        message_id: Snowflake,
        content: String,
    ) -> Result<(), WitError> {
        self.need("messages.manage")?;
        self.services
            .edit_message(&self.id(), hid(channel_id), hid(message_id), &content)
            .map_err(wit_error)
    }
    fn delete_message(
        &mut self,
        channel_id: Snowflake,
        message_id: Snowflake,
    ) -> Result<(), WitError> {
        self.need("messages.manage")?;
        self.services
            .delete_message(&self.id(), hid(channel_id), hid(message_id))
            .map_err(wit_error)
    }
    fn pin_message(
        &mut self,
        channel_id: Snowflake,
        message_id: Snowflake,
        pinned: bool,
    ) -> Result<(), WitError> {
        self.need("messages.manage")?;
        self.services
            .pin_message(&self.id(), hid(channel_id), hid(message_id), pinned)
            .map_err(wit_error)
    }
    fn add_reaction(
        &mut self,
        channel_id: Snowflake,
        message_id: Snowflake,
        emoji: String,
        burst: bool,
    ) -> Result<(), WitError> {
        if !self.gesture() {
            self.need("reactions")?;
        }
        self.services
            .add_reaction(&self.id(), hid(channel_id), hid(message_id), &emoji, burst)
            .map_err(wit_error)
    }
    fn remove_reaction(
        &mut self,
        channel_id: Snowflake,
        message_id: Snowflake,
        emoji: String,
    ) -> Result<(), WitError> {
        if !self.gesture() {
            self.need("reactions")?;
        }
        self.services
            .remove_reaction(&self.id(), hid(channel_id), hid(message_id), &emoji)
            .map_err(wit_error)
    }
    fn ack_channel(&mut self, channel_id: Snowflake) -> Result<(), WitError> {
        self.need("read-states.write")?;
        self.services
            .ack_channel(&self.id(), hid(channel_id))
            .map_err(wit_error)
    }
    fn ack_guild(&mut self, guild_id: Snowflake) -> Result<(), WitError> {
        self.need("read-states.write")?;
        self.services
            .ack_guild(&self.id(), hid(guild_id))
            .map_err(wit_error)
    }
    fn add_local_message(
        &mut self,
        channel_id: Snowflake,
        content: String,
    ) -> Result<Snowflake, WitError> {
        self.need("ui.local-messages")?;
        self.services
            .add_local_message(&self.id(), hid(channel_id), &content)
            .map(|id| id.get())
            .map_err(wit_error)
    }
    fn remove_local_message(&mut self, channel_id: Snowflake, id: Snowflake) {
        self.services
            .remove_local_message(&self.plugin_id, hid(channel_id), hid(id));
    }
    fn start_typing(&mut self, channel_id: Snowflake) -> Result<(), WitError> {
        self.need("messages.send")?;
        self.services
            .start_typing(&self.id(), hid(channel_id))
            .map_err(wit_error)
    }
    fn open_dm(&mut self, user_id: Snowflake) -> Result<Snowflake, WitError> {
        if !self.gesture() {
            self.need("messages.send")?;
        }
        self.services
            .open_dm(&self.id(), hid(user_id))
            .map(|id| id.get())
            .map_err(wit_error)
    }
    fn copy_to_clipboard(&mut self, text: String) -> Result<(), WitError> {
        if !self.gesture() {
            self.need("clipboard.write")?;
        }
        self.services
            .copy_to_clipboard(&self.id(), &text)
            .map_err(wit_error)
    }
    fn read_clipboard(&mut self) -> Result<String, WitError> {
        self.need("clipboard.read")?;
        self.services.read_clipboard(&self.id()).map_err(wit_error)
    }
    fn open_url(&mut self, url: String) -> Result<(), WitError> {
        if !(url.starts_with("https://") || url.starts_with("http://")) {
            return Err(invalid("only http(s) links can be opened"));
        }
        self.services.open_url(&self.id(), &url).map_err(wit_error)
    }
}

impl wit::navigation::Host for State {
    fn open_channel(
        &mut self,
        channel_id: Snowflake,
        message_id: Option<Snowflake>,
    ) -> Result<(), WitError> {
        self.services
            .open_channel(hid(channel_id), hid_opt(message_id))
            .map_err(wit_error)
    }
    fn open_guild(&mut self, guild_id: Snowflake) -> Result<(), WitError> {
        self.services.open_guild(hid(guild_id)).map_err(wit_error)
    }
    fn open_home(&mut self) {
        self.services.open_home();
    }
    fn open_profile(
        &mut self,
        user_id: Snowflake,
        guild_id: Option<Snowflake>,
    ) -> Result<(), WitError> {
        self.services
            .open_profile(hid(user_id), hid_opt(guild_id))
            .map_err(wit_error)
    }
    fn open_settings(&mut self, page: String) -> Result<(), WitError> {
        self.services.open_settings(&page).map_err(wit_error)
    }
    fn search(&mut self, query: String) -> Result<(), WitError> {
        self.services.open_search(&query).map_err(wit_error)
    }
    fn open_media(&mut self, url: String) -> Result<(), WitError> {
        self.services.open_media(&url).map_err(wit_error)
    }
}

impl wit::discord::Host for State {
    fn request(
        &mut self,
        method: String,
        path: String,
        body: Option<Json>,
    ) -> Result<Resource<ResponseRes>, WitError> {
        let method = method.to_ascii_uppercase();
        let allowed = self
            .granted
            .read()
            .expect("grants lock poisoned")
            .iter()
            .any(|p| route_matches(p, &method, &path));
        if !allowed {
            return Err(wit_error(HostError::denied(&format!(
                "discord-api:{method} {path}"
            ))));
        }
        let (status, body) = self
            .services
            .discord_request(&self.id(), &method, &path, body.as_deref())
            .map_err(wit_error)?;
        Ok(self.push(ResponseRes(status, body)))
    }
}

impl wit::discord::HostResponse for State {
    fn status(&mut self, this: Resource<ResponseRes>) -> u16 {
        self.get(&this).0
    }
    fn body(&mut self, this: Resource<ResponseRes>) -> String {
        self.get(&this).1.clone()
    }
    fn drop(&mut self, this: Resource<ResponseRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

impl wit::presence::Host for State {
    fn set_status(&mut self, status: String) -> Result<(), WitError> {
        self.need("presence.write")?;
        if !matches!(status.as_str(), "online" | "idle" | "dnd" | "invisible") {
            return Err(invalid("status is online, idle, dnd or invisible"));
        }
        self.services
            .set_status(&self.id(), &status)
            .map_err(wit_error)
    }
    fn set_custom_status(
        &mut self,
        text: Option<String>,
        emoji: Option<String>,
        expires: Option<WitTimestamp>,
    ) -> Result<(), WitError> {
        self.need("presence.write")?;
        self.services
            .set_custom_status(&self.id(), text.as_deref(), emoji.as_deref(), expires)
            .map_err(wit_error)
    }
    fn set_activity(
        &mut self,
        id: String,
        activity: Resource<ActivityBuilderRes>,
    ) -> Result<(), WitError> {
        let activity = self.take(activity).0;
        self.need("presence.write")?;
        self.services
            .set_activity(&self.id(), &id, Some(activity))
            .map_err(wit_error)
    }
    fn clear_activity(&mut self, id: String) {
        if self.has("presence.write") {
            let _ = self.services.set_activity(&self.id(), &id, None);
        }
    }
}

impl wit::presence::HostActivityBuilder for State {
    fn new(&mut self, kind: String, name: String) -> Resource<ActivityBuilderRes> {
        self.push(ActivityBuilderRes(ActivityData {
            kind,
            name,
            ..Default::default()
        }))
    }
    fn details(&mut self, this: Resource<ActivityBuilderRes>, text: String) {
        self.get_mut(&this).0.details = Some(text);
    }
    fn state(&mut self, this: Resource<ActivityBuilderRes>, text: String) {
        self.get_mut(&this).0.state = Some(text);
    }
    fn application_id(&mut self, this: Resource<ActivityBuilderRes>, id: Snowflake) {
        self.get_mut(&this).0.application_id = Some(hid(id));
    }
    fn url(&mut self, this: Resource<ActivityBuilderRes>, url: String) {
        self.get_mut(&this).0.url = Some(url);
    }
    fn timestamps(
        &mut self,
        this: Resource<ActivityBuilderRes>,
        start: Option<WitTimestamp>,
        end: Option<WitTimestamp>,
    ) {
        let activity = &mut self.get_mut(&this).0;
        activity.start = start;
        activity.end = end;
    }
    fn large_image(
        &mut self,
        this: Resource<ActivityBuilderRes>,
        image: String,
        text: Option<String>,
    ) {
        let activity = &mut self.get_mut(&this).0;
        activity.large_image = Some(image);
        activity.large_text = text;
    }
    fn small_image(
        &mut self,
        this: Resource<ActivityBuilderRes>,
        image: String,
        text: Option<String>,
    ) {
        let activity = &mut self.get_mut(&this).0;
        activity.small_image = Some(image);
        activity.small_text = text;
    }
    fn button(&mut self, this: Resource<ActivityBuilderRes>, label: String, url: String) {
        let activity = &mut self.get_mut(&this).0;
        if activity.buttons.len() < 2 {
            activity.buttons.push((label, url));
        }
    }
    fn party(
        &mut self,
        this: Resource<ActivityBuilderRes>,
        id: Option<String>,
        size: Option<(u32, u32)>,
    ) {
        let activity = &mut self.get_mut(&this).0;
        activity.party_id = id;
        activity.party_size = size;
    }
    fn drop(&mut self, this: Resource<ActivityBuilderRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

impl wit::voice::Host for State {
    fn join(&mut self, channel_id: Snowflake) -> Result<(), WitError> {
        self.need("voice.control")?;
        self.services
            .voice_join(&self.id(), hid(channel_id))
            .map_err(wit_error)
    }
    fn leave(&mut self) -> Result<(), WitError> {
        self.need("voice.control")?;
        self.services.voice_leave(&self.id()).map_err(wit_error)
    }
    fn current_channel(&mut self) -> Option<Snowflake> {
        if self.has("voice.read") || self.has("voice.control") {
            sid_opt(self.services.voice_channel())
        } else {
            None
        }
    }
    fn set_self_mute(&mut self, muted: bool) -> Result<(), WitError> {
        self.need("voice.control")?;
        self.services
            .voice_set_self(&self.id(), "mute", muted)
            .map_err(wit_error)
    }
    fn set_self_deaf(&mut self, deafened: bool) -> Result<(), WitError> {
        self.need("voice.control")?;
        self.services
            .voice_set_self(&self.id(), "deaf", deafened)
            .map_err(wit_error)
    }
    fn set_video(&mut self, on: bool) -> Result<(), WitError> {
        self.need("voice.control")?;
        self.services
            .voice_set_self(&self.id(), "video", on)
            .map_err(wit_error)
    }
    fn set_user_volume(&mut self, user_id: Snowflake, volume: f32) -> Result<(), WitError> {
        self.need("voice.control")?;
        if !(0.0..=4.0).contains(&volume) {
            return Err(invalid("volume is 0.0 to 4.0"));
        }
        self.services
            .voice_set_user_volume(&self.id(), hid(user_id), volume)
            .map_err(wit_error)
    }
    fn move_member(
        &mut self,
        guild_id: Snowflake,
        user_id: Snowflake,
        channel_id: Option<Snowflake>,
    ) -> Result<(), WitError> {
        self.need("voice.moderate")?;
        self.services
            .voice_move_member(&self.id(), hid(guild_id), hid(user_id), hid_opt(channel_id))
            .map_err(wit_error)
    }
    fn set_member_mute(
        &mut self,
        guild_id: Snowflake,
        user_id: Snowflake,
        muted: bool,
        deafened: bool,
    ) -> Result<(), WitError> {
        self.need("voice.moderate")?;
        self.services
            .voice_set_member_mute(&self.id(), hid(guild_id), hid(user_id), muted, deafened)
            .map_err(wit_error)
    }
    fn start_screen_share(&mut self) -> Result<(), WitError> {
        self.need("voice.control")?;
        self.services
            .voice_start_screen_share(&self.id())
            .map_err(wit_error)
    }
    fn record_voice_message(&mut self, channel_id: Snowflake) -> Result<(), WitError> {
        if !self.gesture() {
            self.need("messages.send")?;
        }
        self.services
            .voice_record_message(&self.id(), hid(channel_id))
            .map_err(wit_error)
    }
}

impl wit::notify::Host for State {
    fn notify(
        &mut self,
        title: String,
        body: String,
        channel_id: Option<Snowflake>,
        tag: Option<String>,
    ) -> Result<(), WitError> {
        self.need("notifications")?;
        self.services
            .notify(&self.id(), &title, &body, hid_opt(channel_id), tag.as_deref())
            .map_err(wit_error)
    }
    fn play_sound(&mut self, source: String, volume: f32) -> Result<u32, WitError> {
        self.need("audio.play")?;
        let source = self.resolve_source(&source)?;
        self.services
            .play_sound(&self.id(), &source, volume.clamp(0.0, 1.0))
            .map_err(wit_error)
    }
    fn stop_sound(&mut self, id: u32) {
        self.services.stop_sound(&self.plugin_id, id);
    }
    fn override_sound(&mut self, name: String, source: Option<String>) -> Result<(), WitError> {
        self.need("audio.replace")?;
        let source = source.map(|s| self.resolve_source(&s)).transpose()?;
        self.services
            .override_sound(&self.id(), &name, source.as_deref())
            .map_err(wit_error)
    }
    fn speak(&mut self, text: String, voice: Option<String>, rate: f32) -> Result<(), WitError> {
        self.need("audio.play")?;
        self.services
            .speak(&self.id(), &text, voice.as_deref(), rate.clamp(0.25, 4.0))
            .map_err(wit_error)
    }
    fn known_sounds(&mut self) -> Vec<String> {
        self.services.known_sounds()
    }
}

impl wit::tweaks::Host for State {
    fn known(&mut self) -> Vec<String> {
        self.services.known_tweaks()
    }
    fn get(&mut self, name: String) -> Option<Json> {
        self.services.tweak(&name)
    }
    fn set(&mut self, name: String, value: Json) -> Result<(), WitError> {
        self.need("tweaks")?;
        if !self.services.known_tweaks().contains(&name) {
            return Err(invalid(format!("no tweak `{name}` in this client")));
        }
        serde_json::from_str::<serde_json::Value>(&value)
            .map_err(|err| invalid(format!("value isn't JSON: {err}")))?;
        self.services
            .set_tweak(&self.id(), &name, Some(&value))
            .map_err(wit_error)
    }
    fn reset(&mut self, name: String) {
        if self.has("tweaks") {
            let _ = self.services.set_tweak(&self.id(), &name, None);
        }
    }
}

impl wit::files::Host for State {
    fn save(&mut self, suggested_name: String, data: Vec<u8>) -> Result<bool, WitError> {
        self.need("files")?;
        self.services
            .save_file(&self.id(), &suggested_name, &data)
            .map_err(wit_error)
    }
    fn pick(
        &mut self,
        multiple: bool,
        filters: Vec<String>,
    ) -> Result<Vec<Resource<PickedFileRes>>, WitError> {
        self.need("files")?;
        let files = self
            .services
            .pick_files(&self.id(), multiple, &filters)
            .map_err(wit_error)?;
        Ok(files
            .into_iter()
            .map(|(name, data)| self.push(PickedFileRes(name, data)))
            .collect())
    }
}

impl wit::files::HostPickedFile for State {
    fn name(&mut self, this: Resource<PickedFileRes>) -> String {
        self.get(&this).0.clone()
    }
    fn size(&mut self, this: Resource<PickedFileRes>) -> u64 {
        self.get(&this).1.len() as u64
    }
    fn read(&mut self, this: Resource<PickedFileRes>) -> Result<Vec<u8>, WitError> {
        Ok(self.get(&this).1.clone())
    }
    fn drop(&mut self, this: Resource<PickedFileRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

impl wit::net::Host for State {
    fn fetch(
        &mut self,
        method: String,
        url: String,
        headers: Vec<(String, String)>,
        body: Option<Vec<u8>>,
        timeout_ms: u32,
    ) -> Result<Resource<HttpResponseRes>, WitError> {
        self.check_url(&url)?;
        let request = HttpRequest {
            method: method.to_ascii_uppercase(),
            url,
            headers,
            body,
            timeout_ms: if timeout_ms == 0 {
                30_000
            } else {
                timeout_ms.min(120_000)
            },
        };
        let response = self
            .services
            .http_fetch(&self.id(), request)
            .map_err(wit_error)?;
        Ok(self.push(HttpResponseRes(response)))
    }
    fn connect_websocket(&mut self, url: String, protocols: Vec<String>) -> Result<u32, WitError> {
        self.check_url(&url)?;
        let id = self.shared.next_socket.fetch_add(1, Ordering::AcqRel) + 1;
        let (host, plugin) = (self.host.clone(), self.id());
        let on_event: Box<dyn Fn(&str, Vec<u8>) + Send + Sync> = Box::new(move |kind, data| {
            if let Some(host) = host.upgrade() {
                host.post_websocket_event(&plugin, id, kind, data);
            }
        });
        self.services
            .websocket_connect(&self.id(), id, &url, &protocols, on_event)
            .map_err(wit_error)?;
        Ok(id)
    }
    fn send_websocket(
        &mut self,
        id: u32,
        text: Option<String>,
        binary: Option<Vec<u8>>,
    ) -> Result<(), WitError> {
        self.services
            .websocket_send(&self.id(), id, text.as_deref(), binary.as_deref())
            .map_err(wit_error)
    }
    fn close_websocket(&mut self, id: u32) {
        self.services.websocket_close(&self.plugin_id, id);
    }
}

impl wit::net::HostHttpResponse for State {
    fn status(&mut self, this: Resource<HttpResponseRes>) -> u16 {
        self.get(&this).0.status
    }
    fn headers(&mut self, this: Resource<HttpResponseRes>) -> Vec<(String, String)> {
        self.get(&this).0.headers.clone()
    }
    fn body(&mut self, this: Resource<HttpResponseRes>) -> Vec<u8> {
        self.get(&this).0.body.clone()
    }
    fn text(&mut self, this: Resource<HttpResponseRes>) -> Result<String, WitError> {
        String::from_utf8(self.get(&this).0.body.clone())
            .map_err(|_| invalid("the body isn't UTF-8"))
    }
    fn drop(&mut self, this: Resource<HttpResponseRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

impl wit::overlay::Host for State {
    fn set_display_content(
        &mut self,
        channel_id: Snowflake,
        message_id: Snowflake,
        content: Option<String>,
    ) -> Result<(), WitError> {
        self.overlay_message(OverlayUpdate::DisplayContent {
            channel_id: hid(channel_id),
            message_id: hid(message_id),
            content,
        })
    }
    fn set_annotation(
        &mut self,
        channel_id: Snowflake,
        message_id: Snowflake,
        text: Option<String>,
    ) -> Result<(), WitError> {
        self.overlay_message(OverlayUpdate::Annotation {
            channel_id: hid(channel_id),
            message_id: hid(message_id),
            text,
        })
    }
    fn set_decoration(
        &mut self,
        channel_id: Snowflake,
        message_id: Snowflake,
        slot: String,
        view_id: Option<String>,
    ) -> Result<(), WitError> {
        self.overlay_message(OverlayUpdate::Decoration {
            channel_id: hid(channel_id),
            message_id: hid(message_id),
            slot,
            view_id,
        })
    }
    fn add_span(
        &mut self,
        channel_id: Snowflake,
        message_id: Snowflake,
        start: u32,
        end: u32,
        kind: String,
        options: Json,
    ) -> Result<(), WitError> {
        self.overlay_message(OverlayUpdate::Span {
            channel_id: hid(channel_id),
            message_id: hid(message_id),
            span: Span {
                start,
                end: end.max(start),
                kind,
                options,
            },
        })
    }
    fn set_hidden(
        &mut self,
        channel_id: Snowflake,
        message_id: Snowflake,
        hidden: bool,
    ) -> Result<(), WitError> {
        self.overlay_message(OverlayUpdate::Hidden {
            channel_id: hid(channel_id),
            message_id: hid(message_id),
            hidden,
        })
    }
    fn clear(&mut self, channel_id: Snowflake, message_id: Snowflake) -> Result<(), WitError> {
        self.overlay_message(OverlayUpdate::Clear {
            channel_id: hid(channel_id),
            message_id: hid(message_id),
        })
    }
    fn set_user_display(
        &mut self,
        user_id: Snowflake,
        guild_id: Option<Snowflake>,
        field: String,
        value: Option<Json>,
    ) -> Result<(), WitError> {
        self.need("display.users")?;
        self.services.overlay(
            &self.plugin_id,
            OverlayUpdate::Display {
                target: "user",
                id: user_id,
                scope: guild_id,
                field,
                value,
            },
        );
        Ok(())
    }
    fn set_user_decoration(
        &mut self,
        user_id: Snowflake,
        slot: String,
        view_id: Option<String>,
    ) -> Result<(), WitError> {
        self.need("display.users")?;
        self.services.overlay(
            &self.plugin_id,
            OverlayUpdate::Decorate {
                target: "user",
                id: user_id,
                slot,
                view_id,
            },
        );
        Ok(())
    }
    fn set_channel_display(
        &mut self,
        channel_id: Snowflake,
        field: String,
        value: Option<Json>,
    ) -> Result<(), WitError> {
        self.need("display.channels")?;
        self.services.overlay(
            &self.plugin_id,
            OverlayUpdate::Display {
                target: "channel",
                id: channel_id,
                scope: None,
                field,
                value,
            },
        );
        Ok(())
    }
    fn set_channel_decoration(
        &mut self,
        channel_id: Snowflake,
        slot: String,
        view_id: Option<String>,
    ) -> Result<(), WitError> {
        self.need("display.channels")?;
        self.services.overlay(
            &self.plugin_id,
            OverlayUpdate::Decorate {
                target: "channel",
                id: channel_id,
                slot,
                view_id,
            },
        );
        Ok(())
    }
    fn set_guild_display(
        &mut self,
        guild_id: Snowflake,
        field: String,
        value: Option<Json>,
    ) -> Result<(), WitError> {
        self.need("display.guilds")?;
        self.services.overlay(
            &self.plugin_id,
            OverlayUpdate::Display {
                target: "guild",
                id: guild_id,
                scope: None,
                field,
                value,
            },
        );
        Ok(())
    }
    fn set_guild_decoration(
        &mut self,
        guild_id: Snowflake,
        slot: String,
        view_id: Option<String>,
    ) -> Result<(), WitError> {
        self.need("display.guilds")?;
        self.services.overlay(
            &self.plugin_id,
            OverlayUpdate::Decorate {
                target: "guild",
                id: guild_id,
                slot,
                view_id,
            },
        );
        Ok(())
    }
}

impl wit::composer::Host for State {
    fn text(&mut self) -> Option<String> {
        if !self.has("composer") {
            return None;
        }
        self.services.composer_text()
    }
    fn set_text(&mut self, text: String) -> Result<(), WitError> {
        self.need("composer")?;
        self.services
            .composer_set_text(&self.id(), &text)
            .map_err(wit_error)
    }
    fn insert(&mut self, text: String) -> Result<(), WitError> {
        if !self.gesture() {
            self.need("composer")?;
        }
        self.services
            .composer_insert(&self.id(), &text)
            .map_err(wit_error)
    }
    fn focus(&mut self) {
        self.services.composer_focus();
    }
    fn channel_id(&mut self) -> Option<Snowflake> {
        sid_opt(self.services.composer_channel())
    }
    fn replying_to(&mut self) -> Option<Snowflake> {
        if !self.has("composer") {
            return None;
        }
        sid_opt(self.services.composer_replying_to())
    }
    fn reply_to(
        &mut self,
        channel_id: Snowflake,
        message_id: Snowflake,
        mention: bool,
    ) -> Result<(), WitError> {
        self.services
            .composer_reply_to(&self.id(), hid(channel_id), hid(message_id), mention)
            .map_err(wit_error)
    }
    fn edit(&mut self, channel_id: Snowflake, message_id: Snowflake) -> Result<(), WitError> {
        self.services
            .composer_edit(&self.id(), hid(channel_id), hid(message_id))
            .map_err(wit_error)
    }
    fn add_pending_file(&mut self, filename: String, data: Vec<u8>) -> Result<(), WitError> {
        if !self.gesture() {
            self.need("composer")?;
        }
        self.services
            .composer_add_file(&self.id(), &filename, data)
            .map_err(wit_error)
    }
}

impl wit::drafts::Host for State {}

impl wit::drafts::HostOutgoingAttachment for State {
    fn filename(&mut self, this: Resource<OutgoingAttachmentRes>) -> String {
        self.with_file(&this, |f| f.filename.clone())
            .unwrap_or_default()
    }
    fn set_filename(&mut self, this: Resource<OutgoingAttachmentRes>, name: String) {
        let name = name.replace(['/', '\\'], "_");
        self.with_file_mut(&this, |f| f.filename = name);
    }
    fn content_type(&mut self, this: Resource<OutgoingAttachmentRes>) -> Option<String> {
        self.with_file(&this, |f| f.content_type.clone()).flatten()
    }
    fn size(&mut self, this: Resource<OutgoingAttachmentRes>) -> u64 {
        self.with_file(&this, |f| {
            f.data.as_ref().map_or(f.size, |d| d.len() as u64)
        })
        .unwrap_or_default()
    }
    fn read(&mut self, this: Resource<OutgoingAttachmentRes>) -> Result<Vec<u8>, WitError> {
        self.need("attachments.read")?;
        self.with_file(&this, |f| f.data.clone())
            .flatten()
            .ok_or_else(|| wit_error(HostError::unavailable()))
    }
    fn replace(
        &mut self,
        this: Resource<OutgoingAttachmentRes>,
        data: Vec<u8>,
        content_type: Option<String>,
    ) -> Result<(), WitError> {
        self.need("attachments.read")?;
        self.with_file_mut(&this, |f| {
            f.size = data.len() as u64;
            f.data = Some(data);
            if content_type.is_some() {
                f.content_type = content_type;
            }
        });
        Ok(())
    }
    fn spoiler(&mut self, this: Resource<OutgoingAttachmentRes>) -> bool {
        self.with_file(&this, |f| f.spoiler).unwrap_or_default()
    }
    fn set_spoiler(&mut self, this: Resource<OutgoingAttachmentRes>, on: bool) {
        self.with_file_mut(&this, |f| f.spoiler = on);
    }
    fn description(&mut self, this: Resource<OutgoingAttachmentRes>) -> Option<String> {
        self.with_file(&this, |f| f.description.clone()).flatten()
    }
    fn set_description(&mut self, this: Resource<OutgoingAttachmentRes>, text: Option<String>) {
        self.with_file_mut(&this, |f| f.description = text);
    }
    fn drop(&mut self, this: Resource<OutgoingAttachmentRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

impl State {
    fn with_file<R>(
        &self,
        this: &Resource<OutgoingAttachmentRes>,
        f: impl FnOnce(&OutgoingFile) -> R,
    ) -> Option<R> {
        let res = self.get(this);
        let draft = res.draft.lock().expect("draft lock poisoned");
        draft.files.get(res.index).map(f)
    }

    fn with_file_mut(
        &self,
        this: &Resource<OutgoingAttachmentRes>,
        f: impl FnOnce(&mut OutgoingFile),
    ) {
        let res = self.get(this);
        let mut draft = res.draft.lock().expect("draft lock poisoned");
        if let Some(file) = draft.files.get_mut(res.index) {
            f(file);
        }
    }

    fn draft(
        &self,
        this: &Resource<OutgoingMessageRes>,
    ) -> sync::MutexGuard<'_, OutgoingDraft> {
        self.get(this).0.lock().expect("draft lock poisoned")
    }
}

const MAX_MESSAGE_LEN: usize = 4000;

impl wit::drafts::HostOutgoingMessage for State {
    fn channel_id(&mut self, this: Resource<OutgoingMessageRes>) -> Snowflake {
        self.draft(&this).channel_id.get()
    }
    fn guild_id(&mut self, this: Resource<OutgoingMessageRes>) -> Option<Snowflake> {
        sid_opt(self.draft(&this).guild_id)
    }
    fn content(&mut self, this: Resource<OutgoingMessageRes>) -> String {
        self.draft(&this).content.clone()
    }
    fn set_content(&mut self, this: Resource<OutgoingMessageRes>, content: String) {
        self.draft(&this).content = content;
    }
    fn reply_to(&mut self, this: Resource<OutgoingMessageRes>) -> Option<Snowflake> {
        sid_opt(self.draft(&this).reply_to)
    }
    fn reply_mention(&mut self, this: Resource<OutgoingMessageRes>) -> bool {
        self.draft(&this).reply_mention
    }
    fn set_reply_mention(&mut self, this: Resource<OutgoingMessageRes>, ping: bool) {
        self.draft(&this).reply_mention = ping;
    }
    fn flags(&mut self, this: Resource<OutgoingMessageRes>) -> Vec<String> {
        self.draft(&this).flags.clone()
    }
    fn set_flag(
        &mut self,
        this: Resource<OutgoingMessageRes>,
        name: String,
        on: bool,
    ) -> Result<(), WitError> {
        if !matches!(name.as_str(), "silent" | "suppress-embeds") {
            return Err(invalid(format!(
                "unknown flag `{name}` (silent, suppress-embeds)"
            )));
        }
        let mut draft = self.draft(&this);
        draft.flags.retain(|f| *f != name);
        if on {
            draft.flags.push(name);
        }
        Ok(())
    }
    fn stickers(&mut self, this: Resource<OutgoingMessageRes>) -> Vec<String> {
        self.draft(&this).stickers.clone()
    }
    fn attachments(
        &mut self,
        this: Resource<OutgoingMessageRes>,
    ) -> Vec<Resource<OutgoingAttachmentRes>> {
        let (draft, count) = {
            let res = self.get(&this);
            let count = res.0.lock().expect("draft lock poisoned").files.len();
            (res.0.clone(), count)
        };
        (0..count)
            .map(|index| {
                self.push(OutgoingAttachmentRes {
                    draft: draft.clone(),
                    index,
                })
            })
            .collect()
    }
    fn add_attachment(
        &mut self,
        this: Resource<OutgoingMessageRes>,
        filename: String,
        data: Vec<u8>,
        content_type: Option<String>,
    ) -> Result<(), WitError> {
        let mut draft = self.draft(&this);
        if draft.files.len() >= 10 {
            return Err(invalid("at most 10 files per message"));
        }
        draft.files.push(OutgoingFile {
            filename: filename.replace(['/', '\\'], "_"),
            content_type,
            size: data.len() as u64,
            data: Some(data),
            spoiler: false,
            description: None,
        });
        Ok(())
    }
    fn remove_attachment(&mut self, this: Resource<OutgoingMessageRes>, index: u32) {
        let mut draft = self.draft(&this);
        if (index as usize) < draft.files.len() {
            draft.files.remove(index as usize);
        }
    }
    fn sent_by_plugin(&mut self, this: Resource<OutgoingMessageRes>) -> Option<String> {
        self.draft(&this).sent_by_plugin.clone()
    }
    fn split_into(
        &mut self,
        this: Resource<OutgoingMessageRes>,
        contents: Vec<String>,
    ) -> Result<(), WitError> {
        if contents.is_empty()
            || contents.len() > 10
            || contents.iter().any(|c| c.len() > MAX_MESSAGE_LEN)
        {
            return Err(invalid(
                "1 to 10 parts, each within the message length limit",
            ));
        }
        let mut draft = self.draft(&this);
        draft.content = contents[0].clone();
        draft.split = Some(contents);
        Ok(())
    }
    fn edit_instead(
        &mut self,
        this: Resource<OutgoingMessageRes>,
        message_id: Snowflake,
    ) -> Result<(), WitError> {
        self.draft(&this).edit_instead = Some(hid(message_id));
        Ok(())
    }
    fn drop(&mut self, this: Resource<OutgoingMessageRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

impl wit::drafts::HostOutgoingEdit for State {
    fn channel_id(&mut self, this: Resource<OutgoingEditRes>) -> Snowflake {
        self.get(&this).0.channel_id.get()
    }
    fn guild_id(&mut self, this: Resource<OutgoingEditRes>) -> Option<Snowflake> {
        sid_opt(self.get(&this).0.guild_id)
    }
    fn message_id(&mut self, this: Resource<OutgoingEditRes>) -> Snowflake {
        self.get(&this).0.message_id.get()
    }
    fn original_content(&mut self, this: Resource<OutgoingEditRes>) -> String {
        self.get(&this).0.original_content.clone()
    }
    fn content(&mut self, this: Resource<OutgoingEditRes>) -> String {
        self.get(&this).0.content.clone()
    }
    fn set_content(&mut self, this: Resource<OutgoingEditRes>, content: String) {
        self.get_mut(&this).0.content = content;
    }
    fn drop(&mut self, this: Resource<OutgoingEditRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

impl State {
    fn changes(
        &self,
        this: &Resource<IncomingMessageRes>,
    ) -> sync::MutexGuard<'_, IncomingChanges> {
        self.get(this)
            .changes
            .lock()
            .expect("incoming changes lock poisoned")
    }
}

impl wit::drafts::HostIncomingMessage for State {
    fn message(&mut self, this: Resource<IncomingMessageRes>) -> Resource<MessageRes> {
        let message = self.get(&this).message.clone();
        self.push(MessageRes(message))
    }
    fn previous(&mut self, this: Resource<IncomingMessageRes>) -> Option<Resource<MessageRes>> {
        let previous = self.get(&this).previous.clone();
        previous.map(|m| self.push(MessageRes(m)))
    }
    fn source(&mut self, this: Resource<IncomingMessageRes>) -> String {
        self.get(&this).source.to_string()
    }
    fn set_display_content(&mut self, this: Resource<IncomingMessageRes>, content: String) {
        self.changes(&this).display_content = Some(content);
    }
    fn set_annotation(&mut self, this: Resource<IncomingMessageRes>, text: String) {
        self.changes(&this).annotation = Some(text);
    }
    fn add_span(
        &mut self,
        this: Resource<IncomingMessageRes>,
        start: u32,
        end: u32,
        kind: String,
        options: Json,
    ) {
        let mut changes = self.changes(&this);
        if changes.spans.len() < 256 {
            changes.spans.push(Span {
                start,
                end: end.max(start),
                kind,
                options,
            });
        }
    }
    fn set_decoration(
        &mut self,
        this: Resource<IncomingMessageRes>,
        slot: String,
        view_id: Option<String>,
    ) {
        let mut changes = self.changes(&this);
        changes.decorations.retain(|(s, _)| *s != slot);
        changes.decorations.push((slot, view_id));
    }
    fn set_embed_state(&mut self, this: Resource<IncomingMessageRes>, index: u32, state: String) {
        self.changes(&this).embed_states.push((index, state));
    }
    fn set_attachment_state(
        &mut self,
        this: Resource<IncomingMessageRes>,
        index: u32,
        state: String,
    ) {
        self.changes(&this).attachment_states.push((index, state));
    }
    fn set_author_display(
        &mut self,
        this: Resource<IncomingMessageRes>,
        field: String,
        value: Option<String>,
    ) {
        let mut changes = self.changes(&this);
        changes.author_display.retain(|(f, _)| *f != field);
        changes.author_display.push((field, value));
    }
    fn set_deleted_label(&mut self, this: Resource<IncomingMessageRes>, text: String) {
        self.changes(&this).deleted_label = Some(text);
    }
    fn drop(&mut self, this: Resource<IncomingMessageRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

impl wit::drafts::HostNotification for State {
    fn kind(&mut self, this: Resource<NotificationRes>) -> String {
        self.get(&this).0.kind.clone()
    }
    fn channel_id(&mut self, this: Resource<NotificationRes>) -> Option<Snowflake> {
        sid_opt(self.get(&this).0.channel_id)
    }
    fn guild_id(&mut self, this: Resource<NotificationRes>) -> Option<Snowflake> {
        sid_opt(self.get(&this).0.guild_id)
    }
    fn message(&mut self, this: Resource<NotificationRes>) -> Option<Resource<MessageRes>> {
        if !self.has("messages.read") {
            return None;
        }
        let message = self.get(&this).0.message.clone();
        message.map(|m| self.push(MessageRes(Arc::new(m))))
    }
    fn title(&mut self, this: Resource<NotificationRes>) -> String {
        self.get(&this).0.title.clone()
    }
    fn set_title(&mut self, this: Resource<NotificationRes>, title: String) {
        self.get_mut(&this).0.title = title;
    }
    fn body(&mut self, this: Resource<NotificationRes>) -> String {
        self.get(&this).0.body.clone()
    }
    fn set_body(&mut self, this: Resource<NotificationRes>, body: String) {
        self.get_mut(&this).0.body = body;
    }
    fn set_silent(&mut self, this: Resource<NotificationRes>, silent: bool) {
        self.get_mut(&this).0.silent = silent;
    }
    fn set_bypass_dnd(&mut self, this: Resource<NotificationRes>, bypass: bool) {
        self.get_mut(&this).0.bypass_dnd = bypass;
    }
    fn drop(&mut self, this: Resource<NotificationRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

impl wit::drafts::HostEvent for State {
    fn kind(&mut self, this: Resource<EventRes>) -> String {
        self.get(&this).0.kind.clone()
    }
    fn channel_id(&mut self, this: Resource<EventRes>) -> Option<Snowflake> {
        sid_opt(self.get(&this).0.channel_id)
    }
    fn guild_id(&mut self, this: Resource<EventRes>) -> Option<Snowflake> {
        sid_opt(self.get(&this).0.guild_id)
    }
    fn user_id(&mut self, this: Resource<EventRes>) -> Option<Snowflake> {
        sid_opt(self.get(&this).0.user_id)
    }
    fn message_id(&mut self, this: Resource<EventRes>) -> Option<Snowflake> {
        sid_opt(self.get(&this).0.message_id)
    }
    fn message(&mut self, this: Resource<EventRes>) -> Option<Resource<MessageRes>> {
        let message = self.get(&this).0.message.clone();
        message.map(|m| self.push(MessageRes(Arc::new(m))))
    }
    fn previous_message(&mut self, this: Resource<EventRes>) -> Option<Resource<MessageRes>> {
        let message = self.get(&this).0.previous_message.clone();
        message.map(|m| self.push(MessageRes(Arc::new(m))))
    }
    fn presence(&mut self, this: Resource<EventRes>) -> Option<Resource<PresenceRes>> {
        let presence = self.get(&this).0.presence.clone();
        presence.map(|p| self.push(PresenceRes(p)))
    }
    fn voice_state(&mut self, this: Resource<EventRes>) -> Option<Resource<VoiceStateRes>> {
        let voice = self.get(&this).0.voice_state.clone();
        voice.map(|v| self.push(VoiceStateRes(v)))
    }
    fn data(&mut self, this: Resource<EventRes>) -> Json {
        self.get(&this).0.data.clone()
    }
    fn drop(&mut self, this: Resource<EventRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

impl wit::drafts::HostOutgoingPresence for State {
    fn status(&mut self, this: Resource<OutgoingPresenceRes>) -> String {
        self.get(&this).0.status.clone()
    }
    fn activities(&mut self, this: Resource<OutgoingPresenceRes>) -> Vec<Resource<ActivityRes>> {
        let list = self.get(&this).0.activities.clone();
        list.into_iter()
            .map(|a| self.push(ActivityRes(a)))
            .collect()
    }
    fn remove_activity(&mut self, this: Resource<OutgoingPresenceRes>, index: u32) {
        let activities = &mut self.get_mut(&this).0.activities;
        if (index as usize) < activities.len() {
            activities.remove(index as usize);
        }
    }
    fn drop(&mut self, this: Resource<OutgoingPresenceRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

impl wit::ui::Host for State {
    fn known_components(&mut self) -> Vec<String> {
        KNOWN_COMPONENTS.iter().map(|s| s.to_string()).collect()
    }
    fn image_from_bytes(&mut self, data: Vec<u8>) -> Result<String, WitError> {
        const MAX_IMAGE: usize = 16 * 1024 * 1024;
        const MAX_IMAGES: usize = 256;
        if data.len() > MAX_IMAGE {
            return Err(invalid("images are at most 16 MiB"));
        }
        let mut images = self.shared.images.lock().expect("images lock poisoned");
        if images.len() >= MAX_IMAGES {
            return Err(wit_error(HostError::new(
                "quota-exceeded",
                "too many images; drop some with `drop-image`",
            )));
        }
        let src = format!(
            "plugin-image:{}",
            self.shared.next_image.fetch_add(1, Ordering::AcqRel) + 1
        );
        images.insert(src.clone(), data);
        Ok(src)
    }
    fn drop_image(&mut self, src: String) {
        self.shared
            .images
            .lock()
            .expect("images lock poisoned")
            .remove(&src);
    }
    fn refresh(&mut self, view_id: String) {
        self.services.refresh_view(&self.plugin_id, &view_id);
    }
    fn toast(&mut self, text: String, kind: String) {
        let kind = if matches!(kind.as_str(), "success" | "failure") {
            kind
        } else {
            "info".to_string()
        };
        self.services.toast(&self.plugin_id, &text, &kind);
    }
    fn open_modal(&mut self, view_id: String, title: String, size: String) -> Result<(), WitError> {
        self.need("ui.modal")?;
        self.services
            .open_surface(&self.id(), "modal", &view_id, &title, &size)
            .map_err(wit_error)
    }
    fn close_modal(&mut self, view_id: String) {
        self.services
            .close_surface(&self.plugin_id, "modal", &view_id);
    }
    fn open_popout(&mut self, view_id: String, anchor_item_id: String) -> Result<(), WitError> {
        self.need("ui.slots")?;
        self.services
            .open_surface(&self.id(), "popout", &view_id, "", &anchor_item_id)
            .map_err(wit_error)
    }
    fn close_popout(&mut self, view_id: String) {
        self.services
            .close_surface(&self.plugin_id, "popout", &view_id);
    }
    fn open_page(&mut self, view_id: String, title: String) -> Result<(), WitError> {
        self.need("ui.pages")?;
        self.services
            .open_surface(&self.id(), "page", &view_id, &title, "")
            .map_err(wit_error)
    }
    fn open_side_panel(&mut self, view_id: String, title: String) -> Result<(), WitError> {
        self.need("ui.pages")?;
        self.services
            .open_surface(&self.id(), "side-panel", &view_id, &title, "")
            .map_err(wit_error)
    }
    fn close_side_panel(&mut self, view_id: String) {
        self.services
            .close_surface(&self.plugin_id, "side-panel", &view_id);
    }
    fn show_notice(
        &mut self,
        id: String,
        text: String,
        button_label: Option<String>,
        action_id: Option<String>,
    ) -> Result<(), WitError> {
        self.need("ui.notices")?;
        self.services
            .show_notice(
                &self.id(),
                &id,
                &text,
                button_label.as_deref(),
                action_id.as_deref(),
            )
            .map_err(wit_error)
    }
    fn dismiss_notice(&mut self, id: String) {
        self.services.dismiss_notice(&self.plugin_id, &id);
    }
    fn set_window_overlay(&mut self, view_id: Option<String>) -> Result<(), WitError> {
        self.need("ui.overlay")?;
        match view_id {
            Some(view_id) => self
                .services
                .open_surface(&self.id(), "overlay", &view_id, "", "")
                .map_err(wit_error),
            None => {
                self.services.close_surface(&self.plugin_id, "overlay", "");
                Ok(())
            }
        }
    }
}

impl wit::ui::HostViewBuilder for State {
    fn new(&mut self) -> Resource<ViewBuilderRes> {
        self.push(ViewBuilderRes(ViewBuilderState::default()))
    }
    fn element(
        &mut self,
        this: Resource<ViewBuilderRes>,
        parent: Option<u32>,
        tag: String,
        key: String,
    ) -> u32 {
        self.get_mut(&this).0.element(parent, tag, key)
    }
    fn prop(&mut self, this: Resource<ViewBuilderRes>, node: u32, name: String, json_value: Json) {
        let plugin = self.plugin_id.clone();
        if let Some(node) = self.get_mut(&this).0.node_mut(node) {
            match serde_json::from_str(&json_value) {
                Ok(value) => {
                    node.props.insert(name, value);
                }
                Err(_) => {
                    log::debug!(target: "plugin", "[{plugin}] prop `{name}` isn't JSON: {json_value}")
                }
            }
        }
    }
    fn text(&mut self, this: Resource<ViewBuilderRes>, node: u32, content: String) {
        if let Some(node) = self.get_mut(&this).0.node_mut(node) {
            node.text = Some(content);
        }
    }
    fn class(&mut self, this: Resource<ViewBuilderRes>, node: u32, name: String) {
        let scoped = css::scope_class(&self.plugin_id, &name);
        if let Some(node) = self.get_mut(&this).0.node_mut(node) {
            node.classes.push(scoped);
        }
    }
    fn style(&mut self, this: Resource<ViewBuilderRes>, node: u32, declarations: String) {
        if let Some(node) = self.get_mut(&this).0.node_mut(node) {
            node.style = Some(css::scope_inline_style(&declarations));
        }
    }
    fn on(&mut self, this: Resource<ViewBuilderRes>, node: u32, event: String, action_id: String) {
        if let Some(node) = self.get_mut(&this).0.node_mut(node) {
            node.events.push((event, action_id));
        }
    }
    fn drop(&mut self, this: Resource<ViewBuilderRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

impl wit::slots::Host for State {
    fn set_content(
        &mut self,
        slot: String,
        id: String,
        filters: Vec<String>,
        view_id: Option<String>,
    ) -> Result<(), WitError> {
        self.need("ui.slots")?;
        if !view::KNOWN_CONTENT_SLOTS.contains(&slot.as_str()) {
            return Err(invalid(format!("no content slot `{slot}` in this client")));
        }
        {
            let mut registrations = self.registrations();
            match view_id {
                Some(view_id) => registrations
                    .slot_contents
                    .insert((slot, id), (filters, view_id)),
                None => registrations.slot_contents.remove(&(slot, id)),
            };
        }
        self.registrations_changed();
        Ok(())
    }
    fn known_slots(&mut self) -> Vec<String> {
        view::KNOWN_ITEM_SLOTS
            .iter()
            .chain(view::KNOWN_CONTENT_SLOTS)
            .map(|s| s.to_string())
            .collect()
    }
}

impl wit::slots::HostSlotItem for State {
    fn new(&mut self, slot: String, id: String) -> Resource<SlotItemRes> {
        self.push(SlotItemRes(SlotItemData {
            slot,
            id,
            ..Default::default()
        }))
    }
    fn set_icon(&mut self, this: Resource<SlotItemRes>, icon: String) {
        self.get_mut(&this).0.icon = Some(icon);
    }
    fn set_label(&mut self, this: Resource<SlotItemRes>, label: String) {
        self.get_mut(&this).0.label = label;
    }
    fn set_active(&mut self, this: Resource<SlotItemRes>, active: bool) {
        self.get_mut(&this).0.active = active;
    }
    fn set_disabled(&mut self, this: Resource<SlotItemRes>, disabled: bool) {
        self.get_mut(&this).0.disabled = disabled;
    }
    fn set_badge(&mut self, this: Resource<SlotItemRes>, count: u32) {
        self.get_mut(&this).0.badge = count;
    }
    fn set_danger(&mut self, this: Resource<SlotItemRes>, danger: bool) {
        self.get_mut(&this).0.danger = danger;
    }
    fn set_visible_in(&mut self, this: Resource<SlotItemRes>, filters: Vec<String>) {
        self.get_mut(&this).0.visible_in = filters;
    }
    fn set_hidden(&mut self, this: Resource<SlotItemRes>, hidden: bool) {
        self.get_mut(&this).0.hidden = hidden;
    }
    fn commit(&mut self, this: Resource<SlotItemRes>) -> Result<(), WitError> {
        self.need("ui.slots")?;
        let item = self.get(&this).0.clone();
        if !view::KNOWN_ITEM_SLOTS.contains(&item.slot.as_str()) {
            return Err(invalid(format!("no slot `{}` in this client", item.slot)));
        }
        if item.label.trim().is_empty() {
            return Err(invalid("a slot item needs a label"));
        }
        self.registrations()
            .slot_items
            .insert((item.slot.clone(), item.id.clone()), item);
        self.registrations_changed();
        Ok(())
    }
    fn remove(&mut self, this: Resource<SlotItemRes>) {
        let key = {
            let item = &self.get(&this).0;
            (item.slot.clone(), item.id.clone())
        };
        self.registrations().slot_items.remove(&key);
        self.registrations_changed();
    }
    fn drop(&mut self, this: Resource<SlotItemRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

impl wit::registry::Host for State {
    fn subscribe(&mut self, event_kind: String) -> Result<(), WitError> {
        let Some(permission) = events::permission_for(&event_kind) else {
            return Err(invalid(format!(
                "no event kind `{event_kind}` in this client"
            )));
        };
        if let Some(permission) = permission {
            self.need(permission)?;
        }
        let mut registrations = self.registrations();
        if !registrations.events.contains(&event_kind) {
            registrations.events.push(event_kind);
        }
        Ok(())
    }
    fn unsubscribe(&mut self, event_kind: String) {
        self.registrations().events.retain(|e| *e != event_kind);
    }
    fn register_command(&mut self, command: Resource<CommandRes>) -> Result<(), WitError> {
        let command = self.take(command).0;
        let valid_name = !command.name.is_empty()
            && command.name.len() <= 64
            && command.name.split(' ').count() <= 3
            && command.name.chars().all(|c| {
                c.is_lowercase() || c.is_ascii_digit() || c == '-' || c == '_' || c == ' '
            });
        if !valid_name {
            return Err(invalid(format!(
                "command names are lowercase, up to 64 characters: `{}`",
                command.name
            )));
        }
        {
            let mut registrations = self.registrations();
            registrations.commands.retain(|c| c.name != command.name);
            registrations.commands.push(command);
        }
        self.registrations_changed();
        Ok(())
    }
    fn unregister_command(&mut self, name: String) {
        self.registrations().commands.retain(|c| c.name != name);
        self.registrations_changed();
    }
    fn register_menu_item(&mut self, item: Resource<MenuItemRes>) -> Result<(), WitError> {
        let item = self.take(item).0;
        if item.label.trim().is_empty() {
            return Err(invalid("a menu item needs a label"));
        }
        {
            let mut registrations = self.registrations();
            registrations
                .menu_items
                .retain(|i| !(i.menu == item.menu && i.id == item.id));
            registrations.menu_items.push(item);
        }
        self.registrations_changed();
        Ok(())
    }
    fn unregister_menu_item(&mut self, menu: String, id: String) {
        self.registrations()
            .menu_items
            .retain(|i| !(i.menu == menu && i.id == id));
        self.registrations_changed();
    }
    fn register_shortcut(
        &mut self,
        id: String,
        label: String,
        default_chord: String,
    ) -> Result<(), WitError> {
        {
            let mut registrations = self.registrations();
            registrations.shortcuts.retain(|(i, _, _)| *i != id);
            registrations.shortcuts.push((id, label, default_chord));
        }
        self.registrations_changed();
        Ok(())
    }
    fn register_autocomplete(&mut self, trigger: String) -> Result<(), WitError> {
        if trigger.is_empty() || trigger.len() > 16 {
            return Err(invalid("a trigger is 1 to 16 characters"));
        }
        let mut registrations = self.registrations();
        if !registrations.autocomplete_triggers.contains(&trigger) {
            registrations.autocomplete_triggers.push(trigger);
        }
        drop(registrations);
        self.registrations_changed();
        Ok(())
    }
    fn register_search_provider(
        &mut self,
        id: String,
        prefix: Option<String>,
        label: String,
    ) -> Result<(), WitError> {
        {
            let mut registrations = self.registrations();
            registrations.search_providers.retain(|(i, _, _)| *i != id);
            registrations.search_providers.push((id, prefix, label));
        }
        self.registrations_changed();
        Ok(())
    }
    fn register_markdown_rule(
        &mut self,
        id: String,
        pattern: String,
        inline: bool,
    ) -> Result<(), WitError> {
        if pattern.is_empty() || pattern.len() > 512 {
            return Err(invalid("a pattern is 1 to 512 characters"));
        }
        {
            let mut registrations = self.registrations();
            registrations.markdown_rules.retain(|(i, _, _)| *i != id);
            registrations.markdown_rules.push((id, pattern, inline));
        }
        self.registrations_changed();
        Ok(())
    }
    fn register_audio_processor(&mut self, stream: String) -> Result<(), WitError> {
        self.need("audio.process")?;
        if !matches!(stream.as_str(), "incoming" | "outgoing" | "notification") {
            return Err(invalid("streams are incoming, outgoing and notification"));
        }
        let mut registrations = self.registrations();
        if !registrations.audio_streams.contains(&stream) {
            registrations.audio_streams.push(stream);
        }
        drop(registrations);
        self.registrations_changed();
        Ok(())
    }
}

impl wit::registry::HostCommand for State {
    fn new(&mut self, name: String, description: String) -> Resource<CommandRes> {
        self.push(CommandRes(CommandData {
            name,
            description,
            options: Vec::new(),
        }))
    }
    fn add_option(
        &mut self,
        this: Resource<CommandRes>,
        name: String,
        description: String,
        kind: String,
        required: bool,
    ) {
        let command = &mut self.get_mut(&this).0;
        command.options.retain(|o| o.name != name);
        command.options.push(CommandOption {
            name,
            description,
            kind,
            required,
            ..Default::default()
        });
    }
    fn add_choice(
        &mut self,
        this: Resource<CommandRes>,
        option_name: String,
        name: String,
        value: Json,
    ) {
        if let Some(option) = self
            .get_mut(&this)
            .0
            .options
            .iter_mut()
            .find(|o| o.name == option_name)
        {
            option.choices.push((name, value));
        }
    }
    fn autocomplete_option(&mut self, this: Resource<CommandRes>, option_name: String) {
        if let Some(option) = self
            .get_mut(&this)
            .0
            .options
            .iter_mut()
            .find(|o| o.name == option_name)
        {
            option.autocomplete = true;
        }
    }
    fn drop(&mut self, this: Resource<CommandRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

impl wit::registry::HostMenuItem for State {
    fn new(&mut self, menu: String, id: String, label: String) -> Resource<MenuItemRes> {
        self.push(MenuItemRes(MenuItemData {
            menu,
            id,
            label,
            ..Default::default()
        }))
    }
    fn set_icon(&mut self, this: Resource<MenuItemRes>, icon: String) {
        self.get_mut(&this).0.icon = Some(icon);
    }
    fn set_danger(&mut self, this: Resource<MenuItemRes>, danger: bool) {
        self.get_mut(&this).0.danger = danger;
    }
    fn set_checkable(&mut self, this: Resource<MenuItemRes>, checkable: bool) {
        self.get_mut(&this).0.checkable = checkable;
    }
    fn set_parent(&mut self, this: Resource<MenuItemRes>, id: String) {
        self.get_mut(&this).0.parent = Some(id);
    }
    fn set_dynamic(&mut self, this: Resource<MenuItemRes>, dynamic: bool) {
        self.get_mut(&this).0.dynamic = dynamic;
    }
    fn drop(&mut self, this: Resource<MenuItemRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

impl wit::registry::HostSuggestion for State {
    fn new(&mut self, id: String, label: String) -> Resource<SuggestionRes> {
        self.push(SuggestionRes(SuggestionData {
            id,
            label,
            ..Default::default()
        }))
    }
    fn set_description(&mut self, this: Resource<SuggestionRes>, text: String) {
        self.get_mut(&this).0.description = Some(text);
    }
    fn set_insert_text(&mut self, this: Resource<SuggestionRes>, text: String) {
        self.get_mut(&this).0.insert_text = Some(text);
    }
    fn set_icon(&mut self, this: Resource<SuggestionRes>, icon: String) {
        self.get_mut(&this).0.icon = Some(icon);
    }
    fn set_image_url(&mut self, this: Resource<SuggestionRes>, url: String) {
        self.get_mut(&this).0.image_url = Some(url);
    }
    fn drop(&mut self, this: Resource<SuggestionRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

impl wit::registry::HostMenuState for State {
    fn new(&mut self, id: String) -> Resource<MenuStateRes> {
        self.push(MenuStateRes(MenuStateData {
            id,
            ..Default::default()
        }))
    }
    fn set_label(&mut self, this: Resource<MenuStateRes>, label: String) {
        self.get_mut(&this).0.label = Some(label);
    }
    fn set_checked(&mut self, this: Resource<MenuStateRes>, checked: bool) {
        self.get_mut(&this).0.checked = Some(checked);
    }
    fn set_disabled(&mut self, this: Resource<MenuStateRes>, disabled: bool) {
        self.get_mut(&this).0.disabled = disabled;
    }
    fn drop(&mut self, this: Resource<MenuStateRes>) -> wasmtime::Result<()> {
        self.drop_res(this)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hosts_and_routes() {
        assert_eq!(
            url_host("https://API.example.org:8443/x?y").as_deref(),
            Some("api.example.org")
        );
        assert_eq!(url_host("wss://[::1]:9/").as_deref(), Some("[::1]"));
        assert_eq!(url_host("ftp://x"), None);
        assert!(host_matches("api.example.org", "api.example.org"));
        assert!(host_matches("*.example.org", "a.b.example.org"));
        assert!(!host_matches("*.example.org", "example.org"));
        assert!(!host_matches("example.org", "evil-example.org"));
        assert!(route_matches(
            "discord-api:PATCH /users/@me/guilds/*/settings",
            "patch",
            "/users/@me/guilds/123/settings?x=1"
        ));
        assert!(!route_matches(
            "discord-api:PATCH /users/@me/guilds/*/settings",
            "PATCH",
            "/users/@me/guilds//settings"
        ));
        assert!(!route_matches(
            "discord-api:GET /channels/*",
            "GET",
            "/channels/1/messages"
        ));
    }
}
