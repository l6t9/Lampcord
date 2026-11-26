use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use base64::Engine;
use lampcord_core::{
    api,
    auth::{self, LoginFailure},
    captcha::{self, CaptchaChallenge},
    gateway_client::{GatewayClient, GatewayEvent, lazy_request_payload},
    gateway_plan::reconnect_delay_ms,
    remote_auth::{self, RemoteAuthState as CoreRemoteAuthState},
    rest::{DESKTOP_USER_AGENT, RestConfig},
    rest_client::RestClient,
    secret_store::{FileSecretStore, SecretStore},
    state_cache::{CachedPresence, MemberListRow, PersistedCache, StateCache},
    token_store::{SavedAccount, TokenStore},
    traits::{CaptchaSolver, GatewayUrlProvider},
};
use lampcord_model::{channel::Channel, message::Message, snowflake::Snowflake};
use lampcord_transport::{http::ReqwestRestTransport, websocket::TungsteniteGatewayTransport};
use serde_json::Value;
use tokio::sync::{mpsc, oneshot, watch};

use crate::{
    error::FfiErrorKind,
    snapshot::{
        AttachmentSnapshot, AuthStateSnapshot, BadgeSnapshot, BanSnapshot, ChannelSnapshot,
        ClientSnapshot, EmojiSnapshot, GuildFolderSnapshot, GuildSnapshot, MAX_SNAPSHOT_MEMBERS,
        MAX_SNAPSHOT_PRESENCES, MAX_SNAPSHOT_PRIVATE_CHANNELS, MAX_SNAPSHOT_READ_STATES,
        MEMBER_LIST_ROW_GROUP, MEMBER_LIST_ROW_MEMBER, MEMBER_LIST_ROW_PENDING,
        MemberListRowSnapshot, MemberListSnapshot, MemberSnapshot, MessageSnapshot,
        PresenceSnapshot, PrivateChannelSnapshot, ProfileSnapshot, ReactionSnapshot,
        ReadStateSnapshot, RecipientSnapshot, RelationshipSnapshot, ReplySnapshot, RoleSnapshot,
        SNAPSHOT_SCHEMA_VERSION, SavedAccountSnapshot, SnapshotMetaPatch, SnapshotPatch,
        SnapshotVersions, StickerSnapshot, ThreadSnapshot,
    },
    types::{
        AuthChallenge, AuthFailureKind, ClientCommand, ClientConfig, ClientEvent,
        ClientEventEnvelope, ConnectionState, RemoteAuthState,
    },
};

const MAX_EVENT_QUEUE: usize = 512;
const EVENT_DROP_BATCH: usize = 64;
const SNAPSHOT_COALESCE: Duration = Duration::from_millis(32);
const SNAPSHOT_DISPATCH_EVENTS: &[&str] = &[
    "READY_SUPPLEMENTAL",
    "RELATIONSHIP_ADD",
    "RELATIONSHIP_UPDATE",
    "RELATIONSHIP_REMOVE",
    "USER_SETTINGS_UPDATE",
    "GUILD_MEMBER_LIST_UPDATE",
    "MESSAGE_CREATE",
    "MESSAGE_UPDATE",
    "MESSAGE_DELETE",
    "GUILD_CREATE",
    "GUILD_UPDATE",
    "GUILD_DELETE",
    "CHANNEL_CREATE",
    "CHANNEL_UPDATE",
    "CHANNEL_DELETE",
    "USER_UPDATE",
    "GUILD_MEMBER_ADD",
    "GUILD_MEMBER_UPDATE",
    "GUILD_MEMBER_REMOVE",
    "GUILD_MEMBERS_CHUNK",
    "GUILD_MEMBER_LIST_UPDATE",
    "PRESENCE_UPDATE",
    "USER_SETTINGS_UPDATE",
];
static EVENT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

type EventQueue = Arc<Mutex<Vec<ClientEventEnvelope>>>;
type FfiRestClient = RestClient<ReqwestRestTransport, ConfigurableCaptchaSolver>;

#[derive(Clone, Default)]
struct SnapshotState {
    snapshot: ClientSnapshot,
    versions: SnapshotVersions,
}

#[derive(Clone, Default)]
struct ConfigurableCaptchaSolver {
    token: Arc<Mutex<Option<String>>>,
}

impl ConfigurableCaptchaSolver {
    fn set(&self, token: Option<String>) {
        *self.token.lock().expect("captcha lock") = token;
    }
}

#[async_trait]
impl CaptchaSolver for ConfigurableCaptchaSolver {
    async fn solve(&self, _challenge: &CaptchaChallenge) -> Option<String> {
        self.token.lock().expect("captcha lock").clone()
    }
}

struct StaticGatewayUrl {
    rest: Arc<FfiRestClient>,
    override_url: Option<String>,
}

#[async_trait]
impl GatewayUrlProvider for StaticGatewayUrl {
    async fn gateway_url(&self) -> Option<String> {
        if let Some(url) = &self.override_url {
            return Some(url.clone());
        }
        self.rest.gateway_url().await
    }
}

struct Session {
    task: tokio::task::JoinHandle<()>,
    cache: Arc<Mutex<StateCache>>,
    events: mpsc::UnboundedReceiver<GatewayEvent>,
    outbound: mpsc::UnboundedSender<Value>,
}

enum InternalEvent {
    Messages(Snowflake, Vec<Message>),
    PrivateChannels(Vec<Channel>),
}

static INSTALLATION_ID: OnceLock<String> = OnceLock::new();
static SUPER_PROPERTIES: OnceLock<String> = OnceLock::new();

fn unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

fn random_uuid() -> String {
    let mixed = unix_nanos() ^ ((std::process::id() as u128) << 64);
    let b = mixed.to_le_bytes();
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-4{:02x}{:01x}-a{:02x}{:01x}-\
         {:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0],
        b[1],
        b[2],
        b[3],
        b[4],
        b[5],
        b[6],
        b[7] >> 4,
        b[8],
        b[9] >> 4,
        b[10],
        b[11],
        b[12],
        b[13],
        b[14],
        b[15]
    )
}

fn installation_id() -> &'static str {
    INSTALLATION_ID.get_or_init(random_uuid)
}

#[cfg(target_os = "android")]
fn android_property(name: &str) -> Option<String> {
    let output = std::process::Command::new("getprop")
        .arg(name)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (!value.is_empty()).then_some(value)
}

fn os_version() -> String {
    #[cfg(target_os = "android")]
    {
        android_property("ro.build.version.release").unwrap_or_else(|| "unknown".to_owned())
    }
    #[cfg(not(target_os = "android"))]
    {
        std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| std::env::consts::OS.to_owned())
    }
}

fn device_name() -> String {
    #[cfg(target_os = "android")]
    {
        let model = android_property("ro.product.model").unwrap_or_default();
        let product = android_property("ro.product.build.product")
            .or_else(|| android_property("ro.product.device"))
            .unwrap_or_default();
        format!("{model}, {product}")
    }
    #[cfg(not(target_os = "android"))]
    {
        "Desktop".to_owned()
    }
}

fn desktop_os_name() -> String {
    match std::env::consts::OS {
        "macos" => "Mac OS X".to_owned(),
        "windows" => "Windows".to_owned(),
        os => {
            let mut chars = os.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => os.to_owned(),
            }
        }
    }
}

fn system_timezone() -> String {
    if let Ok(tz) = std::env::var("TZ") {
        if !tz.is_empty() {
            return tz;
        }
    }
    if let Ok(tz) = std::fs::read_to_string("/etc/timezone") {
        let tz = tz.trim();
        if !tz.is_empty() {
            return tz.to_owned();
        }
    }
    "UTC".to_owned()
}

// Discord checks X-Super-Properties during MFA. The old placeholder build
// number and device strings are rejected as an invalid code even when the TOTP
// value is correct, so mirror the properties the Kotlin client sends.
fn super_properties() -> &'static str {
    SUPER_PROPERTIES.get_or_init(|| {
        let mobile = cfg!(target_os = "android");
        let vendor_id = random_uuid();
        let value = if mobile {
            serde_json::json!({
                "os": "Android",
                "browser": "Discord Android",
                "device": device_name(),
                "system_locale": "en-US",
                "has_client_mods": false,
                "client_version": "341.0 - rn",
                "release_channel": "canaryRelease",
                "device_vendor_id": vendor_id,
                "design_id": 2,
                "browser_user_agent": "Discord-Android/341200;RNA",
                "browser_version": "",
                "os_version": os_version(),
                "client_build_number": 6081,
                "client_event_source": null,
                "client_launch_id": random_uuid(),
                "launch_signature": unix_nanos().to_string(),
                "client_app_state": "active",
                "client_heartbeat_session_id": random_uuid()
            })
        } else {
            serde_json::json!({
                "os": desktop_os_name(),
                "browser": "Discord Client",
                "device": "Desktop",
                "system_locale": "en-US",
                "has_client_mods": false,
                "client_version": "0.0.398",
                "release_channel": "stable",
                "device_vendor_id": vendor_id,
                "design_id": 2,
                "browser_user_agent": "",
                "browser_version": "37.6.0",
                "os_version": os_version(),
                "client_build_number": 575562,
                "client_event_source": null,
                "client_launch_id": null,
                "launch_signature": "",
                "client_app_state": null,
                "client_heartbeat_session_id": null
            })
        };
        base64::engine::general_purpose::STANDARD.encode(value.to_string().as_bytes())
    })
}

fn default_user_agent() -> &'static str {
    if cfg!(target_os = "android") {
        "Discord-Android/341200;RNA"
    } else {
        DESKTOP_USER_AGENT
    }
}

fn default_rest_config(
    api_base: &str,
    user_agent: Option<&str>,
    token: Option<String>,
) -> RestConfig {
    let mut config = RestConfig::new(
        token,
        super_properties(),
        installation_id(),
        system_timezone(),
    )
    .with_api_base(api_base.to_owned());
    if let Some(user_agent) = user_agent.filter(|agent| !agent.is_empty()) {
        config = config.with_user_agent(user_agent.to_owned());
    } else {
        config = config.with_user_agent(default_user_agent());
    }
    config
}

fn establish_session(
    token: &str,
    api_base: &str,
    user_agent: Option<&str>,
    gateway_url_override: Option<String>,
    captcha: ConfigurableCaptchaSolver,
) -> Result<(Arc<FfiRestClient>, Session), String> {
    let transport = ReqwestRestTransport::new().map_err(|error| error.to_string())?;
    let rest = Arc::new(RestClient::new(
        default_rest_config(api_base, user_agent, Some(token.to_owned())),
        transport,
        captcha,
    ));
    let url_provider = StaticGatewayUrl {
        rest: rest.clone(),
        override_url: gateway_url_override,
    };
    let (out_tx, out_rx) = mpsc::unbounded_channel();
    let mut client = GatewayClient::new(TungsteniteGatewayTransport::default(), url_provider)
        .with_outbound(out_rx);
    let cache = client.cache().clone();
    let (gw_tx, gw_rx) = mpsc::unbounded_channel();
    let token = token.to_owned();
    let task = tokio::spawn(async move {
        let mut attempt: u32 = 0;
        loop {
            let result = client.run_stream(&token, gw_tx.clone()).await;
            let terminal = match result {
                Ok(terminal) => terminal,
                Err(error) => {
                    eprintln!("[lampcord-ffi] gateway error: {error}");
                    let _ = gw_tx.send(GatewayEvent::Failed {
                        message: error.to_string(),
                    });
                    None
                }
            };
            let reconnect = match terminal {
                Some(GatewayEvent::AuthFailed { .. }) => false,
                Some(GatewayEvent::Closed { code })
                    if matches!(
                        code,
                        Some(4004) | Some(4010) | Some(4011) | Some(4012) | Some(4013) | Some(4014)
                    ) =>
                {
                    false
                }
                _ => true,
            };
            if !reconnect {
                break;
            }
            let delay_ms = reconnect_delay_ms(attempt);
            attempt = attempt.saturating_add(1);
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
        }
    });
    Ok((
        rest,
        Session {
            task,
            cache,
            events: gw_rx,
            outbound: out_tx,
        },
    ))
}

#[derive(Clone)]
pub struct RuntimeHandle {
    cmd_tx: mpsc::UnboundedSender<ClientCommand>,
    snapshot: Arc<Mutex<SnapshotState>>,
    events: EventQueue,
}

impl RuntimeHandle {
    pub fn send(&self, cmd: ClientCommand) -> bool {
        self.cmd_tx.send(cmd).is_ok()
    }

    pub fn snapshot(&self) -> ClientSnapshot {
        self.snapshot
            .lock()
            .expect("snapshot lock")
            .snapshot
            .clone()
    }

    pub fn snapshot_if_newer(&self, since: u64) -> Option<ClientSnapshot> {
        let state = self.snapshot.lock().expect("snapshot lock");
        if state.snapshot.seq > since {
            Some(state.snapshot.clone())
        } else {
            None
        }
    }

    pub fn snapshot_versions(&self) -> SnapshotVersions {
        self.snapshot
            .lock()
            .expect("snapshot lock")
            .versions
            .clone()
    }

    pub fn snapshot_patch(&self, since: SnapshotVersions) -> SnapshotPatch {
        let state = self.snapshot.lock().expect("snapshot lock");
        let snapshot = &state.snapshot;
        let versions = &state.versions;
        SnapshotPatch {
            seq: snapshot.seq,
            schema_version: snapshot.schema_version,
            versions: versions.clone(),
            meta: (versions.meta > since.meta).then(|| SnapshotMetaPatch {
                connection: snapshot.connection.clone(),
                has_token: snapshot.has_token,
                current_user_id: snapshot.current_user_id,
                current_user: snapshot.current_user.clone(),
                current_user_username: snapshot.current_user_username.clone(),
                current_user_email: snapshot.current_user_email.clone(),
                current_user_mfa_enabled: snapshot.current_user_mfa_enabled,
                current_user_avatar: snapshot.current_user_avatar.clone(),
                current_user_status: snapshot.current_user_status.clone(),
                current_user_custom_status: snapshot.current_user_custom_status.clone(),
                current_user_decoration: snapshot.current_user_decoration.clone(),
                current_user_nameplate: snapshot.current_user_nameplate.clone(),
                profile: snapshot.profile.clone(),
                selected_guild_id: snapshot.selected_guild_id,
                selected_channel_id: snapshot.selected_channel_id,
                auth: snapshot.auth.clone(),
                error: snapshot.error.clone(),
                saved_accounts: snapshot.saved_accounts.clone(),
            }),
            guilds: (versions.guilds > since.guilds).then(|| snapshot.guilds.clone()),
            channels: (versions.channels > since.channels).then(|| snapshot.channels.clone()),
            messages: (versions.messages > since.messages).then(|| snapshot.messages.clone()),
            pinned_messages: (versions.messages > since.messages)
                .then(|| snapshot.pinned_messages.clone()),
            mentions: (versions.messages > since.messages).then(|| snapshot.mentions.clone()),
            search_results: (versions.messages > since.messages)
                .then(|| snapshot.search_results.clone()),
            guild_emojis: (versions.guilds > since.guilds).then(|| snapshot.guild_emojis.clone()),
            guild_roles: (versions.guilds > since.guilds).then(|| snapshot.guild_roles.clone()),
            threads: (versions.channels > since.channels).then(|| snapshot.threads.clone()),
            guild_bans: (versions.guilds > since.guilds).then(|| snapshot.guild_bans.clone()),
            members: (versions.members > since.members).then(|| snapshot.members.clone()),
            presences: (versions.presences > since.presences).then(|| snapshot.presences.clone()),
            read_states: (versions.read_states > since.read_states)
                .then(|| snapshot.read_states.clone()),
            private_channels: (versions.channels > since.channels)
                .then(|| snapshot.private_channels.clone()),
            guild_folders: (versions.guilds > since.guilds).then(|| snapshot.guild_folders.clone()),
            member_list: (versions.members > since.members).then(|| snapshot.member_list.clone()),
            relationships: snapshot.relationships.clone(),
        }
    }

    pub fn drain_events(&self) -> Vec<ClientEventEnvelope> {
        let mut events = self.events.lock().expect("events lock");
        std::mem::take(&mut *events)
    }
}

#[cfg(feature = "bench")]
pub fn benchmark_handle() -> RuntimeHandle {
    let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel();
    let versions = SnapshotVersions {
        seq: 1,
        meta: 1,
        guilds: 1,
        channels: 1,
        messages: 1,
        members: 1,
        presences: 1,
        read_states: 1,
    };
    RuntimeHandle {
        cmd_tx,
        snapshot: Arc::new(Mutex::new(SnapshotState {
            snapshot: benchmark_snapshot(),
            versions,
        })),
        events: Arc::new(Mutex::new(Vec::new())),
    }
}

#[cfg(feature = "bench")]
fn benchmark_snapshot() -> ClientSnapshot {
    ClientSnapshot {
        schema_version: SNAPSHOT_SCHEMA_VERSION,
        seq: 1,
        connection: ConnectionState::Ready,
        has_token: true,
        current_user_id: Some(1),
        current_user: Some("bench".to_owned()),
        current_user_username: Some("bench".to_owned()),
        current_user_email: None,
        current_user_mfa_enabled: false,
        current_user_avatar: None,
        selected_guild_id: Some(1),
        selected_channel_id: Some(1),
        guilds: (1..=50)
            .map(|id| GuildSnapshot {
                id,
                name: format!("guild-{id}"),
                icon_url: None,
                owner_id: None,
                unread: false,
                mention_count: 0,
            })
            .collect(),
        channels: (1..=200)
            .map(|id| ChannelSnapshot {
                id,
                guild_id: Some(id % 50 + 1),
                name: format!("channel-{id}"),
                kind: 0,
                parent_id: None,
                position: Some((id % 20) as i32),
                can_view: true,
                can_send: true,
                icon: None,
                recipients: Vec::new(),
                topic: None,
                nsfw: false,
                muted: false,
                unobtrusive_unread: false,
            })
            .collect(),
        messages: (1..=200)
            .map(|id| MessageSnapshot {
                id,
                channel_id: 1,
                author_id: Some(id % 100 + 1),
                author_name: format!("user-{}", id % 100 + 1),
                author_avatar: None,
                content: format!("benchmark message {id}"),
                timestamp: "2026-01-01T00:00:00Z".to_owned(),
                edited_timestamp: None,
                kind: 0,
                author_decoration: None,
                attachments: Vec::new(),
                reply: None,
                reactions: Vec::new(),
                stickers: Vec::new(),
                pinned: false,
                embeds_json: "[]".to_owned(),
                forwarded_json: "[]".to_owned(),
                author_clan_tag: None,
                author_clan_badge: None,
            })
            .collect(),
        members: (1..=200)
            .map(|id| MemberSnapshot {
                guild_id: id % 50 + 1,
                user_id: id,
                username: format!("user-{id}"),
                avatar: None,
                nickname: None,
                roles: vec![1, 2, 3],
                status: Some("online".to_owned()),
                global_name: None,
                decoration: None,
                nameplate: None,
                clan_tag: None,
                clan_badge: None,
            })
            .collect(),
        presences: (1..=200)
            .map(|id| PresenceSnapshot {
                user_id: id,
                status: "online".to_owned(),
                activities: vec!["Compose".to_owned()],
                custom_status: None,
                mobile: false,
            })
            .collect(),
        read_states: (1..=200)
            .map(|id| ReadStateSnapshot {
                channel_id: id,
                last_message_id: Some(id),
                last_read_message_id: Some(id),
                mention_count: 0,
                unread: false,
            })
            .collect(),
        auth: AuthStateSnapshot::default(),
        error: None,
        private_channels: Vec::new(),
        guild_folders: Vec::new(),
        member_list: MemberListSnapshot::default(),
        relationships: Vec::new(),
        current_user_status: None,
        current_user_custom_status: None,
        current_user_decoration: None,
        current_user_nameplate: None,
        profile: None,
        pinned_messages: Vec::new(),
        mentions: Vec::new(),
        search_results: Vec::new(),
        guild_emojis: Vec::new(),
        guild_roles: Vec::new(),
        threads: Vec::new(),
        guild_bans: Vec::new(),
        saved_accounts: Vec::new(),
    }
}

pub fn start(config: ClientConfig, secrets: Option<Arc<dyn SecretStore>>) -> RuntimeHandle {
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    let snapshot = Arc::new(Mutex::new(SnapshotState {
        snapshot: ClientSnapshot {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            connection: ConnectionState::Disconnected,
            ..ClientSnapshot::default()
        },
        versions: SnapshotVersions::default(),
    }));
    let events = Arc::new(Mutex::new(Vec::new()));
    let handle = RuntimeHandle {
        cmd_tx,
        snapshot: snapshot.clone(),
        events: events.clone(),
    };

    thread::Builder::new()
        .name("lampcord-ffi-runtime".to_owned())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(2)
                .build()
                .expect("tokio runtime");
            rt.block_on(run(config, secrets, cmd_rx, snapshot, events));
        })
        .expect("spawn lampcord-ffi runtime thread");

    handle
}

fn is_low_priority_event(event: &ClientEvent) -> bool {
    matches!(
        event,
        ClientEvent::TypingStarted { .. }
            | ClientEvent::PresenceUpdated(_)
            | ClientEvent::ReactionChanged { .. }
            | ClientEvent::NotificationReceived(_)
    )
}

fn push_event(events: &EventQueue, event: ClientEvent) {
    let mut queue = events.lock().expect("events lock");
    match &event {
        ClientEvent::SnapshotUpdated => {
            if queue
                .iter()
                .any(|envelope| matches!(envelope.event, ClientEvent::SnapshotUpdated))
            {
                return;
            }
        }
        ClientEvent::TypingStarted {
            channel_id,
            user_id,
        } => {
            queue.retain(|envelope| {
                !matches!(
                    envelope.event,
                    ClientEvent::TypingStarted {
                        channel_id: channel,
                        user_id: user,
                    } if channel == *channel_id && user == *user_id
                )
            });
        }
        ClientEvent::PresenceUpdated(presence) => {
            queue.retain(|envelope| {
                !matches!(
                    envelope.event,
                    ClientEvent::PresenceUpdated(ref queued) if queued.user_id == presence.user_id
                )
            });
        }
        _ => {}
    }
    if queue.len() >= MAX_EVENT_QUEUE {
        let mut remaining = EVENT_DROP_BATCH.min(queue.len());
        queue.retain(|envelope| {
            if remaining > 0 && is_low_priority_event(&envelope.event) {
                remaining -= 1;
                false
            } else {
                true
            }
        });
        if remaining > 0 {
            queue.drain(..remaining);
        }
    }
    let emitted_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64;
    queue.push(ClientEventEnvelope {
        sequence: EVENT_SEQUENCE.fetch_add(1, Ordering::Relaxed) + 1,
        emitted_at_ms,
        event,
    });
}

fn persist_token(store: &mut TokenStore, secrets: &Arc<dyn SecretStore>, token: &str) {
    store.remove(token);
    store.add(SavedAccount {
        token: token.to_owned(),
        user_id: format!("token:{:x}", token.len()),
        username: None,
    });
    secrets.save(&store.to_json());
}

fn forget_token(store: &mut TokenStore, secrets: &Arc<dyn SecretStore>, token: &str) {
    store.remove(token);
    if store.accounts().is_empty() {
        secrets.clear();
    } else {
        secrets.save(&store.to_json());
    }
}

fn set_snapshot(
    state: &Arc<Mutex<SnapshotState>>,
    events: &EventQueue,
    seq: &Arc<AtomicU64>,
    mut next: ClientSnapshot,
) {
    next.schema_version = SNAPSHOT_SCHEMA_VERSION;
    next.seq = seq.fetch_add(1, Ordering::Relaxed) + 1;
    let mut state = state.lock().expect("snapshot lock");
    let previous = &state.snapshot;
    let meta_changed = previous.connection != next.connection
        || previous.has_token != next.has_token
        || previous.current_user_id != next.current_user_id
        || previous.current_user != next.current_user
        || previous.current_user_username != next.current_user_username
        || previous.current_user_email != next.current_user_email
        || previous.current_user_mfa_enabled != next.current_user_mfa_enabled
        || previous.current_user_avatar != next.current_user_avatar
        || previous.current_user_status != next.current_user_status
        || previous.current_user_custom_status != next.current_user_custom_status
        || previous.current_user_decoration != next.current_user_decoration
        || previous.current_user_nameplate != next.current_user_nameplate
        || previous.profile != next.profile
        || previous.selected_guild_id != next.selected_guild_id
        || previous.selected_channel_id != next.selected_channel_id
        || previous.auth != next.auth
        || previous.error != next.error
        || previous.saved_accounts != next.saved_accounts;
    let guilds_changed = previous.guilds != next.guilds
        || previous.guild_folders != next.guild_folders
        || previous.guild_emojis != next.guild_emojis
        || previous.guild_roles != next.guild_roles
        || previous.guild_bans != next.guild_bans;
    let channels_changed = previous.channels != next.channels
        || previous.private_channels != next.private_channels
        || previous.threads != next.threads;
    let messages_changed = previous.messages != next.messages
        || previous.pinned_messages != next.pinned_messages
        || previous.mentions != next.mentions
        || previous.search_results != next.search_results;
    let members_changed =
        previous.members != next.members || previous.member_list != next.member_list;
    let presences_changed = previous.presences != next.presences;
    let read_states_changed = previous.read_states != next.read_states;
    if meta_changed {
        state.versions.meta += 1;
    }
    if guilds_changed {
        state.versions.guilds += 1;
    }
    if channels_changed {
        state.versions.channels += 1;
    }
    if messages_changed {
        state.versions.messages += 1;
    }
    if members_changed {
        state.versions.members += 1;
    }
    if presences_changed {
        state.versions.presences += 1;
    }
    if read_states_changed {
        state.versions.read_states += 1;
    }
    state.versions.seq = next.seq;
    state.snapshot = next;
    drop(state);
    push_event(events, ClientEvent::SnapshotUpdated);
}

fn message_snapshot(message: &Message) -> MessageSnapshot {
    MessageSnapshot {
        id: message.id,
        channel_id: message.channel_id,
        author_id: Some(message.author.id),
        author_name: message
            .author
            .global_name
            .clone()
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| message.author.username.clone()),
        author_avatar: message.author.avatar.clone(),
        content: message.content.clone(),
        timestamp: message.timestamp.clone(),
        edited_timestamp: message.edited_timestamp.clone(),
        kind: message.kind.0,
        author_decoration: message.author.decoration_asset().map(str::to_owned),
        attachments: message
            .attachments
            .iter()
            .map(|attachment| AttachmentSnapshot {
                id: attachment.id,
                filename: attachment.filename.clone(),
                url: attachment.url.clone(),
                proxy_url: attachment.proxy_url.clone(),
                content_type: attachment.content_type.clone(),
                size: attachment.size,
                width: attachment.width,
                height: attachment.height,
            })
            .collect(),
        reply: message
            .referenced_message
            .as_deref()
            .map(|replied| ReplySnapshot {
                message_id: replied.id,
                author_id: replied.author.id,
                author_name: replied
                    .author
                    .global_name
                    .clone()
                    .filter(|name| !name.is_empty())
                    .unwrap_or_else(|| replied.author.username.clone()),
                author_avatar: replied.author.avatar.clone(),
                content: replied.content.clone(),
            }),
        reactions: message
            .reactions
            .iter()
            .filter_map(|r| {
                let emoji = r.get("emoji")?;
                let emoji_name = emoji
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned();
                if emoji_name.is_empty() {
                    return None;
                }
                Some(ReactionSnapshot {
                    emoji_id: emoji
                        .get("id")
                        .and_then(|v| v.as_str().map(str::to_owned).or_else(|| v.as_u64().map(|n| n.to_string()))),
                    emoji_name,
                    animated: emoji.get("animated").and_then(Value::as_bool).unwrap_or(false),
                    count: r.get("count").and_then(Value::as_u64).unwrap_or(1) as u32,
                    me: r.get("me").and_then(Value::as_bool).unwrap_or(false),
                })
            })
            .collect(),
        stickers: message
            .sticker_items
            .iter()
            .filter_map(|s| {
                let id = json_id(s.get("id"))?;
                let name = s.get("name").and_then(Value::as_str).unwrap_or_default().to_owned();
                let format_type = s.get("format_type").and_then(Value::as_u64).unwrap_or(1) as u8;
                Some(StickerSnapshot { id, name, format_type })
            })
            .collect(),
        pinned: message.pinned,
        embeds_json: if message.embeds.is_empty() {
            "[]".to_owned()
        } else {
            serde_json::to_string(&message.embeds).unwrap_or_else(|_| "[]".to_owned())
        },
        forwarded_json: if message.message_snapshots.is_empty() {
            "[]".to_owned()
        } else {
            let items: Vec<Value> = message
                .message_snapshots
                .iter()
                .filter_map(|snap| {
                    let inner = snap.get("message")?;
                    Some(serde_json::json!({
                        "content": inner.get("content").and_then(Value::as_str).unwrap_or(""),
                        "timestamp": inner.get("timestamp").and_then(Value::as_str).unwrap_or(""),
                        "attachments": inner.get("attachments").cloned().unwrap_or_else(|| Value::Array(Vec::new())),
                        "embeds": inner.get("embeds").cloned().unwrap_or_else(|| Value::Array(Vec::new())),
                    }))
                })
                .collect();
            serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_owned())
        },
        author_clan_tag: message.author.clan_tag().map(str::to_owned),
        author_clan_badge: message.author.clan_badge_url(),
    }
}

fn message_snapshot_from_value(value: &Value) -> MessageSnapshot {
    if let Ok(message) = serde_json::from_value::<Message>(value.clone()) {
        return message_snapshot(&message);
    }
    let author = value.get("author");
    let author_id = json_id(author.and_then(|author| author.get("id")))
        .or_else(|| json_id(value.get("author_id")));
    let author_name = author
        .and_then(|author| author.get("global_name"))
        .and_then(Value::as_str)
        .filter(|name| !name.is_empty())
        .or_else(|| {
            author
                .and_then(|author| author.get("username"))
                .and_then(Value::as_str)
        })
        .unwrap_or("unknown")
        .to_owned();
    MessageSnapshot {
        author_avatar: author
            .and_then(|author| author.get("avatar"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        id: json_id(value.get("id")).unwrap_or_default(),
        channel_id: json_id(value.get("channel_id")).unwrap_or_default(),
        author_id,
        author_name,
        content: value
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        timestamp: value
            .get("timestamp")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        edited_timestamp: value
            .get("edited_timestamp")
            .and_then(Value::as_str)
            .map(str::to_owned),
        kind: value
            .get("type")
            .and_then(Value::as_u64)
            .and_then(|kind| u8::try_from(kind).ok())
            .unwrap_or_default(),
        author_decoration: None,
        attachments: Vec::new(),
        reply: None,
        reactions: Vec::new(),
        stickers: Vec::new(),
        pinned: value.get("pinned").and_then(Value::as_bool).unwrap_or(false),
        embeds_json: value
            .get("embeds")
            .and_then(|e| serde_json::to_string(e).ok())
            .unwrap_or_else(|| "[]".to_owned()),
        forwarded_json: "[]".to_owned(),
        author_clan_tag: None,
        author_clan_badge: None,
    }
}

fn message_snapshot_from_cached_value(
    value: &Value,
    cache: Option<&Arc<Mutex<StateCache>>>,
) -> MessageSnapshot {
    let id = json_id(value.get("id")).unwrap_or_default();
    let channel_id = json_id(value.get("channel_id"));
    let cached = cache.and_then(|cache| {
        let guard = cache.lock().ok()?;
        channel_id.and_then(|channel_id| {
            guard
                .messages(channel_id)
                .iter()
                .find(|message| message.id == id)
                .cloned()
        })
    });
    let mut snapshot = cached
        .as_ref()
        .map(message_snapshot)
        .unwrap_or_else(|| message_snapshot_from_value(value));
    snapshot.id = id;
    if let Some(channel_id) = channel_id {
        snapshot.channel_id = channel_id;
    }
    if let Some(author) = value.get("author") {
        if let Some(author_id) = json_id(author.get("id")) {
            snapshot.author_id = Some(author_id);
        }
        let name = author
            .get("global_name")
            .and_then(Value::as_str)
            .filter(|name| !name.is_empty())
            .or_else(|| author.get("username").and_then(Value::as_str));
        if let Some(name) = name {
            snapshot.author_name = name.to_owned();
        }
    }
    if let Some(content) = value.get("content").and_then(Value::as_str) {
        snapshot.content = content.to_owned();
    }
    if let Some(timestamp) = value.get("timestamp").and_then(Value::as_str) {
        snapshot.timestamp = timestamp.to_owned();
    }
    if value.get("edited_timestamp").is_some() {
        snapshot.edited_timestamp = value
            .get("edited_timestamp")
            .and_then(Value::as_str)
            .map(str::to_owned);
    }
    snapshot
}

/// Maps GET /users/{id}/profile into a ProfileSnapshot, letting the
/// server member profile override the account's bio, pronouns, banner and
/// theme like the official client.
fn profile_snapshot(value: &Value, user_id: u64, guild_id: Option<u64>) -> ProfileSnapshot {
    let user = &value["user"];
    let account = &value["user_profile"];
    let member_profile = &value["guild_member_profile"];
    let member = &value["guild_member"];
    let text = |value: &Value| {
        value
            .as_str()
            .filter(|text| !text.is_empty())
            .map(str::to_owned)
    };
    let pick = |key: &str| {
        text(&member_profile[key])
            .or_else(|| text(&account[key]))
            .or_else(|| text(&user[key]))
    };
    let colors = |value: &Value| -> Vec<u32> {
        value
            .as_array()
            .map(|colors| {
                colors
                    .iter()
                    .filter_map(|c| c.as_u64().and_then(|c| u32::try_from(c).ok()))
                    .collect()
            })
            .unwrap_or_default()
    };

    let banner_url = match (guild_id, text(&member_profile["banner"])) {
        (Some(guild), Some(hash)) => Some(format!(
            "https://cdn.discordapp.com/guilds/{guild}/users/{user_id}/banners/{hash}.png?size=600"
        )),
        _ => text(&account["banner"])
            .or_else(|| text(&user["banner"]))
            .map(|hash| {
                format!("https://cdn.discordapp.com/banners/{user_id}/{hash}.png?size=600")
            }),
    };
    let theme_colors = Some(colors(&member_profile["theme_colors"]))
        .filter(|colors| !colors.is_empty())
        .unwrap_or_else(|| colors(&account["theme_colors"]));
    let accent_color = [
        &member_profile["accent_color"],
        &account["accent_color"],
        &user["accent_color"],
    ]
    .iter()
    .find_map(|value| value.as_u64().and_then(|c| u32::try_from(c).ok()));
    let decoration = text(&user["avatar_decoration_data"]["asset"])
        .or_else(|| text(&user["collectibles"]["avatar_decoration"]["asset"]));

    ProfileSnapshot {
        user_id,
        guild_id,
        username: text(&user["username"]).unwrap_or_default(),
        global_name: text(&user["global_name"]),
        nickname: text(&member["nick"]),
        avatar: text(&user["avatar"]),
        banner_url,
        accent_color,
        theme_colors,
        bio: pick("bio"),
        pronouns: pick("pronouns"),
        decoration,
        badges: value["badges"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|badge| BadgeSnapshot {
                id: text(&badge["id"]).unwrap_or_default(),
                description: text(&badge["description"]).unwrap_or_default(),
                icon: text(&badge["icon"]).unwrap_or_default(),
                link: text(&badge["link"]),
            })
            .collect(),
        mutual_guild_ids: value["mutual_guilds"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|guild| json_id(guild.get("id")))
            .collect(),
        mutual_friends_count: value["mutual_friends_count"]
            .as_u64()
            .and_then(|count| u32::try_from(count).ok()),
        joined_at: text(&member["joined_at"]),
    }
}

fn presence_snapshot(presence: &CachedPresence) -> PresenceSnapshot {
    PresenceSnapshot {
        user_id: presence.user_id,
        status: presence.status.clone(),
        activities: presence.activities.clone(),
        custom_status: presence.custom_status.clone(),
        mobile: presence.mobile,
    }
}

fn json_id(value: Option<&Value>) -> Option<u64> {
    match value? {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn subscribe_guild_members(
    guild_id: Snowflake,
    cache: &Option<Arc<Mutex<StateCache>>>,
    outbound: &Option<mpsc::UnboundedSender<Value>>,
) {
    request_member_list_ranges(guild_id, &[[0, 99]], cache, outbound);
}

fn request_member_list_ranges(
    guild_id: Snowflake,
    ranges: &[[u32; 2]],
    cache: &Option<Arc<Mutex<StateCache>>>,
    outbound: &Option<mpsc::UnboundedSender<Value>>,
) {
    let (Some(cache), Some(outbound)) = (cache, outbound) else {
        return;
    };
    let channel_id = {
        let cache = cache.lock().expect("cache lock");
        cache
            .channels_for_guild(guild_id)
            .into_iter()
            .find(|channel| matches!(channel.kind.0, 0 | 2 | 5 | 13))
            .map(|channel| channel.id)
    };
    if let Some(channel_id) = channel_id {
        let _ = outbound.send(lazy_request_payload(guild_id, channel_id, ranges));
    }
}

fn member_list_snapshot(cache: &StateCache, guild_id: Option<Snowflake>) -> MemberListSnapshot {
    let Some(guild_id) = guild_id else {
        return MemberListSnapshot::default();
    };
    let Some(state) = cache.member_list(guild_id) else {
        return MemberListSnapshot::default();
    };
    let rows = state
        .rows
        .iter()
        .map(|row| match row {
            MemberListRow::Pending => pending_row(),
            MemberListRow::Group { id, count } => MemberListRowSnapshot {
                kind: MEMBER_LIST_ROW_GROUP,
                group_id: Some(id.clone()),
                group_count: *count,
                member: None,
            },
            MemberListRow::Member(user_id) => match cache.member(guild_id, *user_id) {
                Some(member) => MemberListRowSnapshot {
                    kind: MEMBER_LIST_ROW_MEMBER,
                    member: Some(MemberSnapshot {
                        guild_id,
                        user_id: *user_id,
                        username: member.username.clone(),
                        avatar: member.avatar.clone(),
                        nickname: member.nickname.clone(),
                        roles: member.roles.clone(),
                        status: cache
                            .presence(*user_id)
                            .map(|presence| presence.status.clone()),
                        global_name: member.global_name.clone(),
                        decoration: member.decoration.clone(),
                        nameplate: member.nameplate.clone(),
                        clan_tag: cache.user(*user_id).and_then(|u| u.clan_tag()).map(str::to_owned),
                        clan_badge: cache.user(*user_id).and_then(|u| u.clan_badge_url()),
                    }),
                    ..MemberListRowSnapshot::default()
                },
                None => pending_row(),
            },
        })
        .collect();
    MemberListSnapshot {
        rows,
        online_count: state.online_count,
        member_count: state.member_count,
    }
}

fn relationships_snapshot(cache: &StateCache) -> Vec<RelationshipSnapshot> {
    cache
        .relationship_entries()
        .into_iter()
        .map(|relationship| {
            let user = cache.user(relationship.user_id);
            RelationshipSnapshot {
                user_id: relationship.user_id,
                kind: relationship.kind,
                username: user.map(|user| user.username.clone()).unwrap_or_default(),
                global_name: user.and_then(|user| user.global_name.clone()),
                avatar: user.and_then(|user| user.avatar.clone()),
                avatar_decoration: user
                    .and_then(|user| user.decoration_asset())
                    .map(str::to_owned),
            }
        })
        .collect()
}

fn pending_row() -> MemberListRowSnapshot {
    MemberListRowSnapshot {
        kind: MEMBER_LIST_ROW_PENDING,
        ..MemberListRowSnapshot::default()
    }
}

fn channel_can_view(cache: &StateCache, channel: &Channel) -> bool {
    let Some(guild_id) = channel.guild_id else {
        return true;
    };
    let (Some(guild), Some(user)) = (cache.guild(guild_id), cache.current_user()) else {
        return false;
    };
    let Some(member) = cache.member(guild_id, user.id) else {
        return true;
    };
    let member_roles = member.roles.clone();
    lampcord_core::permissions::can_view_channel(guild, channel, &member_roles, user.id)
}

fn channel_can_send(cache: &StateCache, channel: &Channel) -> bool {
    let Some(guild_id) = channel.guild_id else {
        return true;
    };
    let (Some(guild), Some(user)) = (cache.guild(guild_id), cache.current_user()) else {
        return false;
    };
    let Some(member) = cache.member(guild_id, user.id) else {
        return true;
    };
    let member_roles = member.roles.clone();
    lampcord_core::permissions::can_send_messages(guild, channel, &member_roles, user.id)
}

fn channel_snapshot(
    channel: &Channel,
    can_view: bool,
    can_send: bool,
    muted: bool,
    unobtrusive_unread: bool,
) -> ChannelSnapshot {
    ChannelSnapshot {
        can_send,
        muted,
        unobtrusive_unread,
        icon: channel.icon.clone(),
        recipients: channel
            .recipients
            .iter()
            .map(|user| RecipientSnapshot {
                id: user.id,
                username: user.username.clone(),
                global_name: user.global_name.clone(),
                avatar: user.avatar.clone(),
                decoration: user.decoration_asset().map(str::to_owned),
                nameplate: user.nameplate_asset().map(str::to_owned),
                system: user.system.unwrap_or(false),
                bot: user.bot.unwrap_or(false),
            })
            .collect(),
        can_view,
        id: channel.id,
        guild_id: channel.guild_id,
        name: channel_display_name(channel),
        kind: channel.kind.0,
        parent_id: channel.parent_id,
        position: channel.position.map(|position| position as i32),
        topic: channel.topic.clone(),
        nsfw: channel.nsfw.unwrap_or(false),
    }
}

fn channel_display_name(channel: &Channel) -> String {
    if let Some(name) = channel.name.as_ref().filter(|name| !name.is_empty()) {
        return name.clone();
    }
    let names: Vec<String> = channel
        .recipients
        .iter()
        .map(|user| {
            user.global_name
                .clone()
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| user.username.clone())
        })
        .collect();
    if names.is_empty() {
        format!("channel-{}", channel.id)
    } else {
        names.join(", ")
    }
}

fn build_snapshot(
    connection: ConnectionState,
    has_token: bool,
    cache: &Option<Arc<Mutex<StateCache>>>,
    selected_guild_id: Option<Snowflake>,
    selected_channel_id: Option<Snowflake>,
    fetched_messages: &HashMap<Snowflake, Vec<Message>>,
    pinned_messages: &HashMap<Snowflake, Vec<Message>>,
    fetched_mentions: &[Message],
    search_results: &[Message],
    guild_emojis: &HashMap<Snowflake, Vec<EmojiSnapshot>>,
    guild_roles: &HashMap<Snowflake, Vec<RoleSnapshot>>,
    threads: &[ThreadSnapshot],
    guild_bans: &HashMap<Snowflake, Vec<BanSnapshot>>,
    saved_accounts: &[SavedAccountSnapshot],
    profile: &Option<ProfileSnapshot>,
    auth: &AuthStateSnapshot,
    error: Option<String>,
) -> ClientSnapshot {
    let Some(cache) = cache else {
        return ClientSnapshot {
            connection,
            has_token,
            selected_guild_id,
            selected_channel_id,
            auth: auth.clone(),
            error,
            saved_accounts: saved_accounts.to_vec(),
            ..ClientSnapshot::default()
        };
    };
    let cache = cache.lock().expect("cache lock");

    let current_user_id = cache.current_user().map(|user| user.id);
    let current_user = cache.current_user().map(|user| {
        user.global_name
            .clone()
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| user.username.clone())
    });

    let read_state_for_guilds = cache.read_state();
    let guilds: Vec<GuildSnapshot> = cache
        .ordered_guilds()
        .into_iter()
        .map(|guild| {
            let mut unread = false;
            let mut mention_count = 0u32;
            for ch in cache.channels_for_guild(guild.id) {
                if matches!(ch.kind.0, 0 | 5 | 10 | 11 | 12 | 15)
                    && read_state_for_guilds.is_unread(ch.id, ch.last_message_id, None)
                {
                    unread = true;
                }
                mention_count = mention_count.saturating_add(read_state_for_guilds.mention_count(ch.id));
            }
            GuildSnapshot {
                id: guild.id,
                name: guild
                    .name
                    .clone()
                    .unwrap_or_else(|| "Unknown guild".to_owned()),
                icon_url: guild.icon.clone(),
                owner_id: guild.owner_id,
                unread,
                mention_count,
            }
        })
        .collect();

    let guild_channels: Vec<Channel> = selected_guild_id
        .into_iter()
        .flat_map(|guild_id| {
            cache
                .channels_for_guild(guild_id)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>()
        })
        .collect();
    let channels: Vec<ChannelSnapshot> = guild_channels
        .iter()
        .chain(cache.private_channels())
        .map(|channel| {
            channel_snapshot(
                channel,
                channel_can_view(&cache, channel),
                channel_can_send(&cache, channel),
                cache.channel_muted(channel.guild_id, channel.id),
                cache.channel_unobtrusive_unread(channel.guild_id, channel.id),
            )
        })
        .collect();

    let mut messages: Vec<MessageSnapshot> = if let Some(channel_id) = selected_channel_id {
        let mut merged: HashMap<Snowflake, &Message> = HashMap::new();
        if let Some(fetched) = fetched_messages.get(&channel_id) {
            for message in fetched {
                merged.insert(message.id, message);
            }
        }
        for message in cache.messages(channel_id) {
            merged.insert(message.id, message);
        }
        let mut list = merged.into_values().cloned().collect::<Vec<_>>();
        list.sort_by(|left, right| left.id.cmp(&right.id));
        list.into_iter()
            .map(|message| message_snapshot(&message))
            .collect()
    } else {
        Vec::new()
    };
    if messages.len() > 200 {
        let keep_from = messages.len() - 200;
        messages = messages.split_off(keep_from);
    }

    let members: Vec<MemberSnapshot> = selected_guild_id
        .into_iter()
        .flat_map(|guild_id| cache.members_for_guild(guild_id).into_iter().cloned())
        .take(MAX_SNAPSHOT_MEMBERS)
        .map(|member| MemberSnapshot {
            guild_id: member.guild_id,
            user_id: member.user_id,
            username: member.username,
            avatar: member.avatar,
            nickname: member.nickname,
            roles: member.roles,
            status: cache.presence(member.user_id).map(|p| p.status.clone()),
            global_name: member.global_name,
            decoration: member.decoration,
            nameplate: member.nameplate,
            clan_tag: cache.user(member.user_id).and_then(|u| u.clan_tag()).map(str::to_owned),
            clan_badge: cache.user(member.user_id).and_then(|u| u.clan_badge_url()),
        })
        .collect();

    let presences: Vec<PresenceSnapshot> = cache
        .presences()
        .into_iter()
        .take(MAX_SNAPSHOT_PRESENCES)
        .map(presence_snapshot)
        .collect();

    let private_channels: Vec<PrivateChannelSnapshot> = cache
        .private_channels()
        .into_iter()
        .take(MAX_SNAPSHOT_PRIVATE_CHANNELS)
        .map(|channel| PrivateChannelSnapshot {
            id: channel.id,
            kind: channel.kind.0,
            name: channel.name.clone().filter(|name| !name.is_empty()),
            recipients: channel
                .recipients
                .iter()
                .map(|user| RecipientSnapshot {
                    id: user.id,
                    username: user.username.clone(),
                    global_name: user.global_name.clone(),
                    avatar: user.avatar.clone(),
                    decoration: user.decoration_asset().map(str::to_owned),
                    nameplate: user.nameplate_asset().map(str::to_owned),
                    system: user.system.unwrap_or(false),
                    bot: user.bot.unwrap_or(false),
                })
                .collect(),
            last_message_id: channel.last_message_id,
        })
        .collect();

    let guild_folders: Vec<GuildFolderSnapshot> = cache
        .guild_folders()
        .iter()
        .map(|folder| GuildFolderSnapshot {
            id: folder.id,
            name: folder.name.clone(),
            color: folder.color,
            guild_ids: folder.guild_ids.clone(),
        })
        .collect();

    let mut channel_ids: Vec<Snowflake> = cache.channels().map(|channel| channel.id).collect();
    channel_ids.extend(fetched_messages.keys().copied());
    channel_ids.sort_unstable();
    channel_ids.dedup();
    let read_states: Vec<ReadStateSnapshot> = channel_ids
        .into_iter()
        .take(MAX_SNAPSHOT_READ_STATES)
        .map(|channel_id| {
            let read_state = cache.read_state();
            let channel_last = cache
                .channel(channel_id)
                .and_then(|channel| channel.last_message_id);
            ReadStateSnapshot {
                channel_id,
                last_message_id: read_state.most_recent_message_id(channel_id),
                last_read_message_id: read_state.last_read_message_id(channel_id),
                mention_count: read_state.mention_count(channel_id),
                unread: read_state.is_unread(channel_id, channel_last, None),
            }
        })
        .collect();

    ClientSnapshot {
        schema_version: SNAPSHOT_SCHEMA_VERSION,
        seq: 0,
        connection,
        has_token,
        current_user_id,
        current_user,
        current_user_username: cache.current_user().map(|user| user.username.clone()),
        current_user_email: cache.current_user().and_then(|user| user.email.clone()),
        current_user_mfa_enabled: cache.current_user().and_then(|user| user.mfa_enabled).unwrap_or(false),
        current_user_avatar: cache.current_user().and_then(|user| user.avatar.clone()),
        selected_guild_id,
        selected_channel_id,
        guilds,
        channels,
        messages,
        members,
        presences,
        read_states,
        member_list: member_list_snapshot(&cache, selected_guild_id),
        relationships: relationships_snapshot(&cache),
        auth: auth.clone(),
        error,
        private_channels,
        guild_folders,
        current_user_status: cache.own_status().map(str::to_owned),
        current_user_custom_status: cache.own_custom_status().map(str::to_owned),
        current_user_decoration: cache
            .current_user()
            .and_then(|user| user.decoration_asset().map(str::to_owned)),
        current_user_nameplate: cache
            .current_user()
            .and_then(|user| user.nameplate_asset().map(str::to_owned)),
        profile: profile.clone(),
        pinned_messages: selected_channel_id
            .and_then(|channel_id| pinned_messages.get(&channel_id))
            .map(|pins| pins.iter().map(message_snapshot).collect())
            .unwrap_or_default(),
        mentions: fetched_mentions.iter().map(message_snapshot).collect(),
        search_results: search_results.iter().map(message_snapshot).collect(),
        guild_emojis: selected_guild_id
            .and_then(|gid| guild_emojis.get(&gid))
            .cloned()
            .unwrap_or_default(),
        guild_roles: selected_guild_id
            .and_then(|gid| guild_roles.get(&gid))
            .cloned()
            .unwrap_or_default(),
        threads: threads.to_vec(),
        guild_bans: selected_guild_id
            .and_then(|gid| guild_bans.get(&gid))
            .cloned()
            .unwrap_or_default(),
        saved_accounts: saved_accounts.to_vec(),
    }
}

/// Backfills a guild's channels from REST. Discord no longer sends
/// `GUILD_CREATE` for every guild, so the cache needs this to populate a
/// selected guild's sidebar.
fn cache_guild_channels(
    cache: &Arc<Mutex<StateCache>>,
    guild_id: Snowflake,
    channels: Vec<Channel>,
) {
    let mut cache = cache.lock().expect("cache lock");
    for mut channel in channels {
        channel.guild_id = Some(guild_id);
        cache.upsert_channel(channel);
    }
}

fn auth_failure_kind(failure: Option<LoginFailure>) -> AuthFailureKind {
    match failure {
        Some(LoginFailure::InvalidCredentials) => AuthFailureKind::InvalidCredentials,
        Some(LoginFailure::Suspended) => AuthFailureKind::Suspended,
        Some(LoginFailure::RateLimited) => AuthFailureKind::RateLimited,
        Some(LoginFailure::ServerError) => AuthFailureKind::ServerError,
        Some(LoginFailure::VerificationRequired) => AuthFailureKind::VerificationRequired,
        Some(LoginFailure::AccountDisabled) => AuthFailureKind::AccountDisabled,
        Some(LoginFailure::AccountScheduledForDeletion) => AuthFailureKind::ScheduledForDeletion,
        Some(LoginFailure::CaptchaRejected) => AuthFailureKind::CaptchaRejected,
        Some(LoginFailure::CaptchaCancelled) => AuthFailureKind::CaptchaCancelled,
        Some(LoginFailure::PasswordUpdateRequired) => AuthFailureKind::PasswordUpdateRequired,
        Some(LoginFailure::Network) => AuthFailureKind::Network,
        _ => AuthFailureKind::Unknown,
    }
}

#[derive(Clone, Debug, Default)]
struct MfaFlow {
    login_instance_id: Option<String>,
    fingerprint: Option<String>,
}

fn remember_mfa_flow(
    flows: &mut HashMap<String, MfaFlow>,
    challenge: &AuthChallenge,
    login_instance_id: Option<String>,
    fingerprint: Option<String>,
) {
    if let AuthChallenge::MfaRequired { ticket, .. } = challenge {
        flows.insert(
            ticket.clone(),
            MfaFlow {
                login_instance_id,
                fingerprint,
            },
        );
    } else if matches!(challenge, AuthChallenge::Success { .. }) {
        flows.clear();
    }
}

fn auth_challenge_from_response(response: auth::LoginResponse) -> AuthChallenge {
    if let Some(token) = response.token.filter(|token| !token.is_empty()) {
        return AuthChallenge::Success { token };
    }
    if let Some(body) = response.body.as_deref() {
        if let Some(challenge) = captcha::parse_captcha_challenge(body) {
            let invisible = challenge.is_invisible();
            return AuthChallenge::CaptchaRequired {
                sitekey: challenge.sitekey,
                service: challenge.service.wire_name().to_owned(),
                rqdata: challenge.rqdata,
                session_id: challenge.session_id,
                invisible,
            };
        }
    }
    if let Some(ticket) = response.ticket.filter(|ticket| !ticket.is_empty()) {
        let mut methods = Vec::new();
        if response.totp == Some(true) {
            methods.push("totp".to_owned());
        }
        if response.sms == Some(true) {
            methods.push("sms".to_owned());
        }
        if response.backup == Some(true) {
            methods.push("backup".to_owned());
        }
        if response
            .webauthn
            .as_deref()
            .is_some_and(|value| !value.is_empty())
        {
            methods.push("webauthn".to_owned());
        }
        return AuthChallenge::MfaRequired { ticket, methods };
    }
    if response.failure == Some(LoginFailure::PasswordUpdateRequired)
        || response
            .required_actions
            .iter()
            .any(|action| action.as_str() == Some("update_password"))
    {
        return AuthChallenge::PasswordUpdateRequired;
    }
    AuthChallenge::Failed {
        kind: auth_failure_kind(response.failure),
        message: response
            .message
            .unwrap_or_else(|| "Authentication failed".to_owned()),
    }
}

fn handle_dispatch(
    event_type: &str,
    data: Option<&Value>,
    events: &EventQueue,
    cache: Option<&Arc<Mutex<StateCache>>>,
) {
    let Some(data) = data else {
        return;
    };
    let id = |key: &str| json_id(data.get(key));
    match event_type {
        "MESSAGE_UPDATE" => {
            let snapshot = message_snapshot_from_cached_value(data, cache);
            if snapshot.id != 0 {
                push_event(events, ClientEvent::MessageUpdated(snapshot));
            }
        }
        "MESSAGE_DELETE" => {
            if let (Some(channel_id), Some(message_id)) = (id("channel_id"), id("id")) {
                push_event(
                    events,
                    ClientEvent::MessageDeleted {
                        channel_id,
                        message_id,
                    },
                );
            }
        }
        "MESSAGE_REACTION_ADD" | "MESSAGE_REACTION_REMOVE" => {
            let emoji = data
                .get("emoji")
                .and_then(|emoji| emoji.get("name"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            if let (Some(channel_id), Some(message_id)) = (id("channel_id"), id("message_id")) {
                push_event(
                    events,
                    ClientEvent::ReactionChanged {
                        channel_id,
                        message_id,
                        emoji,
                        added: event_type == "MESSAGE_REACTION_ADD",
                    },
                );
            }
        }
        "TYPING_START" => {
            if let (Some(channel_id), Some(user_id)) = (
                id("channel_id"),
                json_id(
                    data.get("user_id")
                        .or_else(|| data.get("user").and_then(|u| u.get("id"))),
                ),
            ) {
                push_event(
                    events,
                    ClientEvent::TypingStarted {
                        channel_id,
                        user_id,
                    },
                );
            }
        }
        "PRESENCE_UPDATE" => {
            if let Some(presence) = lampcord_core::state_cache::presence_from_value(data) {
                push_event(
                    events,
                    ClientEvent::PresenceUpdated(presence_snapshot(&presence)),
                );
            }
        }
        "NOTIFICATION" => {
            let payload = data.get("message").unwrap_or(data);
            let snapshot = message_snapshot_from_value(payload);
            if snapshot.id != 0 {
                push_event(events, ClientEvent::NotificationReceived(snapshot));
            }
        }
        _ => {}
    }
}

async fn run(
    config: ClientConfig,
    secrets: Option<Arc<dyn SecretStore>>,
    mut cmd_rx: mpsc::UnboundedReceiver<ClientCommand>,
    snapshot: Arc<Mutex<SnapshotState>>,
    events: EventQueue,
) {
    let secret_store: Arc<dyn SecretStore> = secrets.unwrap_or_else(|| {
        Arc::new(FileSecretStore::new(
            config
                .storage_path
                .clone()
                .unwrap_or_else(|| "lampcord-accounts.json".to_owned()),
        ))
    });
    let mut token_store = TokenStore::from_json(&secret_store.load().unwrap_or_default());

    let mut connection = ConnectionState::Disconnected;
    let mut error: Option<String> = None;
    let mut active_token: Option<String> = None;
    let cache_file = config
        .storage_path
        .as_deref()
        .map(|path| std::path::Path::new(path).with_file_name("lampcord-cache.bin"));
    let mut persisted_initial: Option<PersistedCache> = cache_file
        .as_ref()
        .and_then(|file| std::fs::read(file).ok())
        .and_then(|bytes| PersistedCache::from_bytes(&bytes));
    let preloaded = Arc::new(Mutex::new(StateCache::new()));
    if let Some(persisted) = &persisted_initial {
        preloaded.lock().expect("cache lock").restore(persisted);
    }
    let mut cache: Option<Arc<Mutex<StateCache>>> = Some(preloaded);
    let mut persist_ticker = tokio::time::interval(std::time::Duration::from_secs(5));
    persist_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    persist_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut rest: Option<Arc<FfiRestClient>> = None;
    let mut selected_guild_id: Option<Snowflake> = None;
    let mut selected_channel_id: Option<Snowflake> = None;
    let mut fetched_messages: HashMap<Snowflake, Vec<Message>> = HashMap::new();
    let mut fetched_mentions: Vec<Message> = Vec::new();
    let mut pinned_messages: HashMap<Snowflake, Vec<Message>> = HashMap::new();
    let mut search_results: Vec<Message> = Vec::new();
    let mut guild_emojis: HashMap<Snowflake, Vec<EmojiSnapshot>> = HashMap::new();
    let mut guild_roles: HashMap<Snowflake, Vec<RoleSnapshot>> = HashMap::new();
    let mut threads: Vec<ThreadSnapshot> = Vec::new();
    let mut guild_bans: HashMap<Snowflake, Vec<BanSnapshot>> = HashMap::new();
    let mut profile: Option<ProfileSnapshot> = None;
    let mut auth_state = AuthStateSnapshot::default();
    // Discord ties an MFA ticket to the login instance and fingerprint from the
    // password response. Keep those alongside the ticket; the public challenge
    // only exposes what the UI needs to display.
    let mut mfa_flows: HashMap<String, MfaFlow> = HashMap::new();
    let mut gateway_task: Option<tokio::task::JoinHandle<()>> = None;
    let mut gateway_events: Option<mpsc::UnboundedReceiver<GatewayEvent>> = None;
    let mut gateway_outbound: Option<mpsc::UnboundedSender<Value>> = None;
    let mut remote_task: Option<tokio::task::JoinHandle<()>> = None;
    let mut remote_cancel: Option<oneshot::Sender<()>> = None;
    let mut remote_events: Option<mpsc::UnboundedReceiver<CoreRemoteAuthState>> = None;
    let mut should_reconnect = false;
    let api_base = config.api_base.clone();
    let user_agent = config.user_agent.clone();
    let gateway_url_override = config.gateway_url.clone();
    let captcha_solver = ConfigurableCaptchaSolver::default();
    // Kotlin uses one Ktor client for /experiments, /auth/login and /auth/mfa,
    // so the cookies set by the first two requests reach the MFA request. A
    // fresh transport per command silently drops them and Discord answers with
    // "invalid two-factor code" even for a correct code.
    let auth_transport = ReqwestRestTransport::new();
    let seq = Arc::new(AtomicU64::new(0));
    let mut snapshot_dirty = true;
    let mut last_publish = Instant::now();
    let mut flush_ticker = tokio::time::interval(SNAPSHOT_COALESCE);
    flush_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    let publish = |connection: &ConnectionState,
                   has_token: bool,
                   cache: &Option<Arc<Mutex<StateCache>>>,
                   selected_guild_id: Option<Snowflake>,
                   selected_channel_id: Option<Snowflake>,
                   fetched_messages: &HashMap<Snowflake, Vec<Message>>,
                   pinned_messages: &HashMap<Snowflake, Vec<Message>>,
                   fetched_mentions: &[Message],
                   search_results: &[Message],
                   guild_emojis: &HashMap<Snowflake, Vec<EmojiSnapshot>>,
                   guild_roles: &HashMap<Snowflake, Vec<RoleSnapshot>>,
                   threads: &[ThreadSnapshot],
                   guild_bans: &HashMap<Snowflake, Vec<BanSnapshot>>,
                   saved_accounts: &[SavedAccountSnapshot],
                   profile: &Option<ProfileSnapshot>,
                   auth: &AuthStateSnapshot,
                   error: &Option<String>,
                   snapshot: &Arc<Mutex<SnapshotState>>,
                   events: &EventQueue,
                   seq: &Arc<AtomicU64>| {
        let next = build_snapshot(
            connection.clone(),
            has_token,
            cache,
            selected_guild_id,
            selected_channel_id,
            fetched_messages,
            pinned_messages,
            fetched_mentions,
            search_results,
            guild_emojis,
            guild_roles,
            threads,
            guild_bans,
            saved_accounts,
            profile,
            auth,
            error.clone(),
        );
        set_snapshot(snapshot, events, seq, next);
    };

    macro_rules! publish_snapshot {
        () => {{
            let saved_accounts: Vec<SavedAccountSnapshot> = token_store
                .accounts()
                .iter()
                .map(|acc| SavedAccountSnapshot {
                    user_id: acc.user_id.clone(),
                    username: acc.username.clone(),
                    active: active_token.as_deref() == Some(acc.token.as_str()),
                })
                .collect();
            publish(
                &connection,
                active_token.is_some() || !token_store.accounts().is_empty(),
                &cache,
                selected_guild_id,
                selected_channel_id,
                &fetched_messages,
                &pinned_messages,
                &fetched_mentions,
                &search_results,
                &guild_emojis,
                &guild_roles,
                &threads,
                &guild_bans,
                &saved_accounts,
                &profile,
                &auth_state,
                &error,
                &snapshot,
                &events,
                &seq,
            )
        }};
    }

    macro_rules! publish_now {
        () => {{
            publish_snapshot!();
            snapshot_dirty = false;
            last_publish = Instant::now();
        }};
    }

    publish_snapshot!();

    macro_rules! mark_snapshot_dirty {
        () => {
            snapshot_dirty = true;
        };
    }

    macro_rules! flush_snapshot {
        ($immediate:expr) => {
            if snapshot_dirty && ($immediate || last_publish.elapsed() >= SNAPSHOT_COALESCE) {
                publish_now!();
            }
        };
    }

    macro_rules! connect_session {
        ($token:expr) => {{
            let token: String = $token;
            match establish_session(
                &token,
                &api_base,
                user_agent.as_deref(),
                gateway_url_override.clone(),
                captcha_solver.clone(),
            ) {
                Ok((rest_client, session)) => {
                    if let Some(task) = gateway_task.take() {
                        task.abort();
                    }
                    gateway_outbound = Some(session.outbound.clone());
                    gateway_task = Some(session.task);
                    gateway_events = Some(session.events);
                    if let Some(persisted) = &persisted_initial {
                        session.cache.lock().expect("cache lock").restore(persisted);
                    }
                    persisted_initial = None;
                    cache = Some(session.cache);
                    rest = Some(rest_client);
                    active_token = Some(token.clone());
                    should_reconnect = true;
                    connection = ConnectionState::Connecting;
                    error = None;
                    fetched_messages.clear();
                    fetched_mentions.clear();
                    persist_token(&mut token_store, &secret_store, &token);
                    push_event(&events, ClientEvent::ConnectionChanged(connection.clone()));
                    Ok::<(), String>(())
                }
                Err(message) => Err(message),
            }
        }};
    }

    macro_rules! update_auth {
        ($challenge:expr) => {{
            let challenge = $challenge;
            auth_state.challenge = Some(challenge.clone());
            push_event(&events, ClientEvent::AuthChanged(challenge));
        }};
    }

    let (fetch_tx, mut fetch_rx) = mpsc::unbounded_channel::<InternalEvent>();

    loop {
        tokio::select! {
            Some(event) = fetch_rx.recv() => {
                match event {
                    InternalEvent::Messages(channel_id, messages) => {
                        let selected = selected_channel_id == Some(channel_id);
                        fetched_messages.insert(channel_id, messages);
                        if selected {
                            mark_snapshot_dirty!();
                        }
                    }
                    InternalEvent::PrivateChannels(channels) => {
                        if let Some(cache) = &cache {
                            cache.lock().expect("cache lock").upsert_channels(channels);
                        }
                        mark_snapshot_dirty!();
                    }
                }
            }
            maybe_cmd = cmd_rx.recv() => {
                let Some(cmd) = maybe_cmd else { break; };
                let is_add_reaction = matches!(&cmd, ClientCommand::AddReaction { .. });
                match cmd {
                    ClientCommand::Shutdown => {
                        if let Some(task) = gateway_task.take() { task.abort(); }
                        if let Some(cancel) = remote_cancel.take() { let _ = cancel.send(()); }
                        if let Some(task) = remote_task.take() { task.abort(); }
                        break;
                    }
                    ClientCommand::Logout => {
                        if let Some(file) = &cache_file {
                            let _ = std::fs::remove_file(file);
                        }
                        if let Some(task) = gateway_task.take() { task.abort(); }
                        if let Some(cancel) = remote_cancel.take() { let _ = cancel.send(()); }
                        if let Some(task) = remote_task.take() { task.abort(); }
                        gateway_events = None;
                        remote_cancel = None;
                        remote_events = None;
                        should_reconnect = false;
                        if let Some(token) = active_token.as_deref() {
                            forget_token(&mut token_store, &secret_store, token);
                        }
                        active_token = None;
                        cache = None;
                        rest = None;
                        selected_guild_id = None;
                        selected_channel_id = None;
                        fetched_messages.clear();
                        fetched_mentions.clear();
                        auth_state = AuthStateSnapshot::default();
                        error = None;
                        connection = ConnectionState::Disconnected;
                        push_event(&events, ClientEvent::LoggedOut);
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::ConnectToken { token } => {
                        if let Err(message) = connect_session!(token) {
                            connection = ConnectionState::Error { message: message.clone() };
                            error = Some(format!("Session init failed: {message}"));
                            should_reconnect = false;
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::ConnectStoredToken => {
                        if let Some(account) = token_store.accounts().first().cloned() {
                            if let Err(message) = connect_session!(account.token) {
                                connection = ConnectionState::Error { message: message.clone() };
                                error = Some(format!("Session init failed: {message}"));
                                should_reconnect = false;
                            }
                        } else {
                            error = Some("No stored token available".to_owned());
                            push_event(&events, ClientEvent::CommandFailed {
                                kind: FfiErrorKind::NotFound,
                                message: error.clone().unwrap_or_default(),
                            });
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::LoginPassword { login, password, captcha_key } => {
                        captcha_solver.set(captcha_key);
                        let config = default_rest_config(&api_base, user_agent.as_deref(), None);
                        match auth_transport.clone() {
                            Err(transport_error) => {
                                update_auth!(AuthChallenge::Failed {
                                    kind: AuthFailureKind::Network,
                                    message: transport_error.to_string(),
                                });
                            }
                            Ok(transport) => {
                                let client = RestClient::new(config.clone(), transport, captcha_solver.clone());
                                let fingerprint = api::auth::get_fingerprint(&client, &config)
                                    .await
                                    .ok()
                                    .flatten()
                                    .unwrap_or_default();
                                let request = auth::LoginRequest::new(login, password);
                                mfa_flows.clear();
                                match api::auth::login(&client, &config, &request, Some(&fingerprint)).await {
                                    Ok(response) => {
                                        let login_instance_id = response.login_instance_id.clone();
                                        let challenge = auth_challenge_from_response(response);
                                        remember_mfa_flow(
                                            &mut mfa_flows,
                                            &challenge,
                                            login_instance_id,
                                            Some(fingerprint),
                                        );
                                        if let AuthChallenge::Success { token } = &challenge {
                                            if let Err(message) = connect_session!(token.clone()) {
                                                update_auth!(AuthChallenge::Failed {
                                                    kind: AuthFailureKind::Network,
                                                    message,
                                                });
                                                mark_snapshot_dirty!();
                                                continue;
                                            }
                                        }
                                        update_auth!(challenge);
                                        error = None;
                                    }
                                    Err(api_error) => {
                                        update_auth!(AuthChallenge::Failed {
                                            kind: AuthFailureKind::Network,
                                            message: api_error.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::SubmitMfa { code, ticket, method } => {
                        captcha_solver.set(None);
                        let flow = mfa_flows.get(&ticket).cloned().unwrap_or_default();
                        let config = default_rest_config(&api_base, user_agent.as_deref(), None);
                        match auth_transport.clone() {
                            Err(transport_error) => {
                                update_auth!(AuthChallenge::Failed {
                                    kind: AuthFailureKind::Network,
                                    message: transport_error.to_string(),
                                });
                            }
                            Ok(transport) => {
                                let client = RestClient::new(config.clone(), transport, captcha_solver.clone());
                                let fingerprint = match flow.fingerprint {
                                    Some(fingerprint) => fingerprint,
                                    None => api::auth::get_fingerprint(&client, &config)
                                        .await
                                        .ok()
                                        .flatten()
                                        .unwrap_or_default(),
                                };
                                let request = auth::MfaLoginRequest::new(code, ticket)
                                    .with_login_instance_id(flow.login_instance_id.as_deref());
                                match api::auth::login_mfa(&client, &config, &request, Some(&fingerprint), &method).await {
                                    Ok(response) => {
                                        let login_instance_id = response.login_instance_id.clone();
                                        let challenge = auth_challenge_from_response(response);
                                        remember_mfa_flow(
                                            &mut mfa_flows,
                                            &challenge,
                                            login_instance_id,
                                            Some(fingerprint),
                                        );
                                        if let AuthChallenge::Success { token } = &challenge {
                                            if let Err(message) = connect_session!(token.clone()) {
                                                update_auth!(AuthChallenge::Failed {
                                                    kind: AuthFailureKind::Network,
                                                    message,
                                                });
                                                mark_snapshot_dirty!();
                                                continue;
                                            }
                                        }
                                        update_auth!(challenge);
                                        error = None;
                                    }
                                    Err(api_error) => {
                                        update_auth!(AuthChallenge::Failed {
                                            kind: AuthFailureKind::Network,
                                            message: api_error.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::SendMfaSms { ticket } => {
                        let flow = mfa_flows.get(&ticket).cloned().unwrap_or_default();
                        let config = default_rest_config(&api_base, user_agent.as_deref(), None);
                        if let Ok(transport) = auth_transport.clone() {
                            let client = RestClient::new(config.clone(), transport, captcha_solver.clone());
                            let fingerprint = match flow.fingerprint {
                                Some(fingerprint) => fingerprint,
                                None => api::auth::get_fingerprint(&client, &config)
                                    .await
                                    .ok()
                                    .flatten()
                                    .unwrap_or_default(),
                            };
                            match api::auth::send_mfa_sms(&client, &config, &ticket, Some(&fingerprint)).await {
                                Ok(phone) => update_auth!(AuthChallenge::SmsSent { phone }),
                                Err(api_error) => update_auth!(AuthChallenge::Failed {
                                    kind: AuthFailureKind::Network,
                                    message: api_error.to_string(),
                                }),
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::ForgotPassword { login } => {
                        let config = default_rest_config(&api_base, user_agent.as_deref(), None);
                        if let Ok(transport) = auth_transport.clone() {
                            let client = RestClient::new(config.clone(), transport, captcha_solver.clone());
                            match api::auth::forgot_password(&client, &config, &login, None).await {
                                Ok(api::auth::ForgotPasswordResult::Sent) => {
                                    update_auth!(AuthChallenge::ResetSent { phone_verification_required: false });
                                }
                                Ok(api::auth::ForgotPasswordResult::PhoneVerificationRequired) => {
                                    update_auth!(AuthChallenge::ResetSent { phone_verification_required: true });
                                }
                                Ok(api::auth::ForgotPasswordResult::Failed(message)) => {
                                    update_auth!(AuthChallenge::Failed {
                                        kind: AuthFailureKind::Unknown,
                                        message,
                                    });
                                }
                                Err(api_error) => update_auth!(AuthChallenge::Failed {
                                    kind: AuthFailureKind::Network,
                                    message: api_error.to_string(),
                                }),
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::StartRemoteAuth => {
                        let config = default_rest_config(&api_base, user_agent.as_deref(), None);
                        match ReqwestRestTransport::new() {
                            Ok(transport) => {
                                let client = Arc::new(RestClient::new(
                                    config.clone(),
                                    transport,
                                    captcha_solver.clone(),
                                ));
                                let (state_tx, mut state_rx) = watch::channel(CoreRemoteAuthState::Idle);
                                let (cancel_tx, cancel_rx) = oneshot::channel();
                                let (fwd_tx, fwd_rx) = mpsc::unbounded_channel();
                                remote_cancel = Some(cancel_tx);
                                remote_events = Some(fwd_rx);
                                auth_state.remote = RemoteAuthState::Connecting;
                                push_event(&events, ClientEvent::RemoteAuthChanged(RemoteAuthState::Connecting));
                                remote_task = Some(tokio::spawn(async move {
                                    let forward = async {
                                        let _ = state_rx.borrow_and_update();
                                        while state_rx.changed().await.is_ok() {
                                            let state = state_rx.borrow().clone();
                                            if fwd_tx.send(state).is_err() {
                                                break;
                                            }
                                        }
                                    };
                                    tokio::join!(
                                        remote_auth::run_remote_auth(client, config, state_tx, cancel_rx),
                                        forward,
                                    );
                                }));
                            }
                            Err(transport_error) => {
                                auth_state.remote = RemoteAuthState::Error { message: transport_error.to_string() };
                                push_event(&events, ClientEvent::RemoteAuthChanged(auth_state.remote.clone()));
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::CancelRemoteAuth => {
                        if let Some(cancel) = remote_cancel.take() { let _ = cancel.send(()); }
                        if let Some(task) = remote_task.take() { task.abort(); }
                        remote_events = None;
                        auth_state.remote = RemoteAuthState::Idle;
                        push_event(&events, ClientEvent::RemoteAuthChanged(RemoteAuthState::Idle));
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::SelectGuild { guild_id } => {
                        let next = guild_id.map(Snowflake::from);
                        if next != selected_guild_id {
                            selected_guild_id = next;
                            selected_channel_id = None;
                            if let Some(guild_id) = selected_guild_id {
                                subscribe_guild_members(guild_id, &cache, &gateway_outbound);
                                // Backfill channels for servers the cache only knows by id.
                                if let (Some(rest_client), Some(cache)) = (rest.clone(), cache.clone()) {
                                    let config = default_rest_config(
                                        &api_base,
                                        user_agent.as_deref(),
                                        active_token.clone(),
                                    );
                                    match api::channel::get_guild_channels(&*rest_client, &config, guild_id).await {
                                        Ok(channels) => cache_guild_channels(&cache, guild_id, channels),
                                        Err(api_error) => {
                                            error = Some(format!("Failed to fetch channels: {api_error}"));
                                            push_event(&events, ClientEvent::CommandFailed {
                                                kind: FfiErrorKind::Transport,
                                                message: error.clone().unwrap_or_default(),
                                            });
                                        }
                                    }
                                }
                            }
                            mark_snapshot_dirty!();
                        }
                    }
                    ClientCommand::SetMemberListRanges { ranges } => {
                        if let Some(guild_id) = selected_guild_id {
                            let pairs: Vec<[u32; 2]> = ranges
                                .iter()
                                .filter_map(|range| match range.as_slice() {
                                    [start, end] => Some([*start, *end]),
                                    _ => None,
                                })
                                .collect();
                            request_member_list_ranges(guild_id, &pairs, &cache, &gateway_outbound);
                        }
                    }
                    ClientCommand::SelectChannel { channel_id } => {
                        let next = channel_id.map(Snowflake::from);
                        if next == selected_channel_id {
                            continue;
                        }
                        selected_channel_id = next;
                        if let (Some(channel_id), Some(rest_client)) = (selected_channel_id, rest.clone()) {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            let fetch_tx = fetch_tx.clone();
                            tokio::spawn(async move {
                                if let Ok(mut messages) =
                                    api::fetch_channel_messages(&*rest_client, &config, channel_id, 50, None).await
                                {
                                    messages.sort_by(|left, right| left.id.cmp(&right.id));
                                    let _ = fetch_tx.send(InternalEvent::Messages(channel_id, messages));
                                }
                            });
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::FetchMessages { channel_id, before, limit } => {
                        let channel_id = Snowflake::from(channel_id);
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            let limit = limit.unwrap_or(50).clamp(1, 100);
                            match api::fetch_channel_messages(&*rest_client, &config, channel_id, limit, before.map(Snowflake::from)).await {
                                Ok(mut messages) => {
                                    messages.sort_by(|left, right| left.id.cmp(&right.id));
                                    let entry = fetched_messages.entry(channel_id).or_default();
                                    for message in messages {
                                        if !entry.iter().any(|existing| existing.id == message.id) {
                                            entry.push(message);
                                        }
                                    }
                                    entry.sort_by(|left, right| left.id.cmp(&right.id));
                                    if entry.len() > 300 {
                                        let keep_from = entry.len() - 300;
                                        *entry = entry.split_off(keep_from);
                                    }
                                }
                                Err(api_error) => {
                                    error = Some(format!("Failed to fetch messages: {api_error}"));
                                    push_event(&events, ClientEvent::CommandFailed {
                                        kind: FfiErrorKind::Transport,
                                        message: error.clone().unwrap_or_default(),
                                    });
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::FetchMentions { limit, before } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            match api::message::get_mentions(&*rest_client, &config, limit.unwrap_or(50).clamp(1, 100), before).await {
                                Ok(messages) => {
                                    let count = messages.len() as u32;
                                    fetched_mentions = messages;
                                    push_event(&events, ClientEvent::MentionsLoaded { count });
                                }
                                Err(api_error) => {
                                    error = Some(format!("Failed to fetch mentions: {api_error}"));
                                    push_event(&events, ClientEvent::CommandFailed {
                                        kind: FfiErrorKind::Transport,
                                        message: error.clone().unwrap_or_default(),
                                    });
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::SendMessage { channel_id, content, reply_to_message_id, reply_to_guild_id } => {
                        let channel_id = Snowflake::from(channel_id);
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            match api::send_channel_message_with_reply(&*rest_client, &config, channel_id, &content, reply_to_message_id, reply_to_guild_id).await {
                                Ok(message) => {
                                    let entry = fetched_messages.entry(channel_id).or_default();
                                    if !entry.iter().any(|existing| existing.id == message.id) {
                                        entry.push(message);
                                        entry.sort_by(|left, right| left.id.cmp(&right.id));
                                    }
                                    error = None;
                                }
                                Err(api_error) => {
                                    error = Some(format!("Failed to send message: {api_error}"));
                                    push_event(&events, ClientEvent::CommandFailed {
                                        kind: FfiErrorKind::Transport,
                                        message: error.clone().unwrap_or_default(),
                                    });
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::SendAttachments { channel_id, content, file_paths, reply_to_message_id, reply_to_guild_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            let mut uploaded = Vec::new();
                            let mut failure = None;
                            for path in &file_paths {
                                let path = std::path::Path::new(path);
                                let filename = path
                                    .file_name()
                                    .map(|name| name.to_string_lossy().into_owned())
                                    .unwrap_or_else(|| "file".to_owned());
                                let data = match std::fs::read(path) {
                                    Ok(data) => data,
                                    Err(io_error) => {
                                        failure = Some(format!("Couldn't read {filename}: {io_error}"));
                                        break;
                                    }
                                };
                                match api::upload_attachment(&*rest_client, &config, channel_id, &filename, data).await {
                                    Ok(file) => uploaded.push(file),
                                    Err(api_error) => {
                                        failure = Some(format!("Failed to upload {filename}: {api_error}"));
                                        break;
                                    }
                                }
                            }
                            if failure.is_none() {
                                match api::send_channel_message_with_attachments(&*rest_client, &config, channel_id, &content, &uploaded, reply_to_message_id, reply_to_guild_id).await {
                                    Ok(message) => {
                                        let entry = fetched_messages.entry(Snowflake::from(channel_id)).or_default();
                                        if !entry.iter().any(|existing| existing.id == message.id) {
                                            entry.push(message);
                                            entry.sort_by(|left, right| left.id.cmp(&right.id));
                                        }
                                        error = None;
                                    }
                                    Err(api_error) => failure = Some(format!("Failed to send message: {api_error}")),
                                }
                            }
                            if let Some(message) = failure {
                                error = Some(message.clone());
                                push_event(&events, ClientEvent::CommandFailed { kind: FfiErrorKind::Transport, message });
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::FetchProfile { user_id, guild_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            match api::user::get_user_profile(&*rest_client, &config, user_id, guild_id).await {
                                Ok(value) => profile = Some(profile_snapshot(&value, user_id, guild_id)),
                                Err(api_error) => {
                                    error = Some(format!("Failed to load profile: {api_error}"));
                                    push_event(&events, ClientEvent::CommandFailed {
                                        kind: FfiErrorKind::Transport,
                                        message: error.clone().unwrap_or_default(),
                                    });
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::SetCustomStatus { text } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            match api::user::update_custom_status(&*rest_client, &config, text.as_deref(), None, None).await {
                                Ok(_) => {
                                    if let Some(cache) = cache.as_ref() {
                                        cache.lock().expect("cache lock").set_own_custom_status(text.as_deref());
                                    }
                                }
                                Err(api_error) => {
                                    error = Some(format!("Failed to set custom status: {api_error}"));
                                    push_event(&events, ClientEvent::CommandFailed {
                                        kind: FfiErrorKind::Transport,
                                        message: error.clone().unwrap_or_default(),
                                    });
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::FetchPins { channel_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            match api::message::get_pinned_messages(&*rest_client, &config, channel_id).await {
                                Ok(pins) => {
                                    pinned_messages.insert(Snowflake::from(channel_id), pins);
                                }
                                Err(api_error) => {
                                    error = Some(format!("Failed to load pins: {api_error}"));
                                    push_event(&events, ClientEvent::CommandFailed {
                                        kind: FfiErrorKind::Transport,
                                        message: error.clone().unwrap_or_default(),
                                    });
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::EditMessage { channel_id, message_id, content } => {
                        let channel_id = Snowflake::from(channel_id);
                        let message_id = Snowflake::from(message_id);
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            match api::message::edit_message(&*rest_client, &config, channel_id, message_id, &content).await {
                                Ok(message) => {
                                    let entry = fetched_messages.entry(channel_id).or_default();
                                    if let Some(existing) = entry.iter_mut().find(|existing| existing.id == message_id) {
                                        *existing = message;
                                    } else {
                                        entry.push(message);
                                        entry.sort_by(|left, right| left.id.cmp(&right.id));
                                    }
                                    error = None;
                                }
                                Err(api_error) => {
                                    error = Some(format!("Failed to edit message: {api_error}"));
                                    push_event(&events, ClientEvent::CommandFailed {
                                        kind: FfiErrorKind::Transport,
                                        message: error.clone().unwrap_or_default(),
                                    });
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::DeleteMessage { channel_id, message_id } => {
                        let channel_id = Snowflake::from(channel_id);
                        let message_id = Snowflake::from(message_id);
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            match api::message::delete_message(&*rest_client, &config, channel_id, message_id).await {
                                Ok(()) => {
                                    if let Some(entry) = fetched_messages.get_mut(&channel_id) {
                                        entry.retain(|message| message.id != message_id);
                                    }
                                    error = None;
                                }
                                Err(api_error) => {
                                    error = Some(format!("Failed to delete message: {api_error}"));
                                    push_event(&events, ClientEvent::CommandFailed {
                                        kind: FfiErrorKind::Transport,
                                        message: error.clone().unwrap_or_default(),
                                    });
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::AddReaction { channel_id, message_id, emoji }
                    | ClientCommand::RemoveReaction { channel_id, message_id, emoji } => {
                        let add = is_add_reaction;
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            let encoded = api::message::encode_reaction_emoji(&emoji);
                            let result = if add {
                                api::message::add_reaction(&*rest_client, &config, channel_id, message_id, &encoded).await
                            } else {
                                api::message::remove_own_reaction(&*rest_client, &config, channel_id, message_id, &encoded).await
                            };
                            match result {
                                Ok(()) => {
                                    push_event(&events, ClientEvent::ReactionChanged {
                                        channel_id, message_id, emoji, added: add,
                                    });
                                    error = None;
                                }
                                Err(api_error) => {
                                    error = Some(format!("Failed to update reaction: {api_error}"));
                                    push_event(&events, ClientEvent::CommandFailed {
                                        kind: FfiErrorKind::Transport,
                                        message: error.clone().unwrap_or_default(),
                                    });
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::SendTyping { channel_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            if let Err(api_error) = api::channel::trigger_typing(&*rest_client, &config, channel_id).await {
                                error = Some(format!("Failed to send typing: {api_error}"));
                                push_event(&events, ClientEvent::CommandFailed {
                                    kind: FfiErrorKind::Transport,
                                    message: error.clone().unwrap_or_default(),
                                });
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::SetStatus { status } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            let result = api::user::update_status(&*rest_client, &config, &status).await;
                            if result.is_ok() {
                                if let Some(cache) = cache.as_ref() {
                                    cache.lock().expect("cache lock").set_own_status(&status);
                                }
                            }
                            if let Err(api_error) = result {
                                error = Some(format!("Failed to set status: {api_error}"));
                                push_event(&events, ClientEvent::CommandFailed {
                                    kind: FfiErrorKind::Transport,
                                    message: error.clone().unwrap_or_default(),
                                });
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::AckMessage { channel_id, message_id } => {
                        if let Some(cache) = cache.as_ref() {
                            cache.lock().expect("cache lock").read_state_mut().ack(channel_id, message_id);
                        }
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            if let Err(api_error) = api::ack_channel_message(&*rest_client, &config, channel_id, message_id).await {
                                push_event(&events, ClientEvent::CommandFailed {
                                    kind: FfiErrorKind::Transport,
                                    message: format!("Failed to ack message: {api_error}"),
                                });
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::MarkGuildRead { guild_id } => {
                        let pairs: Vec<(u64, u64)> = if let Some(cache) = cache.as_ref() {
                            let mut guard = cache.lock().expect("cache lock");
                            let channels: Vec<(u64, Option<u64>)> = guard
                                .channels_for_guild(guild_id)
                                .into_iter()
                                .map(|ch| (ch.id, ch.last_message_id))
                                .collect();
                            let mut to_ack = Vec::new();
                            for (cid, last_msg) in channels {
                                let newest = last_msg.or_else(|| guard.read_state().most_recent_message_id(cid));
                                if let Some(mid) = newest {
                                    if guard.read_state().is_unread(cid, last_msg, None)
                                        || guard.read_state().mention_count(cid) > 0
                                    {
                                        guard.read_state_mut().ack(cid, mid);
                                        to_ack.push((cid, mid));
                                    }
                                }
                            }
                            to_ack
                        } else {
                            Vec::new()
                        };
                        if !pairs.is_empty() {
                            if let Some(rest_client) = rest.clone() {
                                let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                                let _ = api::channel::ack_bulk(&*rest_client, &config, &pairs).await;
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::PinMessage { channel_id, message_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            if let Err(api_error) = api::message::pin_message(&*rest_client, &config, channel_id, message_id).await {
                                error = Some(format!("Failed to pin message: {api_error}"));
                            } else if let Ok(pins) = api::message::get_pinned_messages(&*rest_client, &config, channel_id).await {
                                pinned_messages.insert(Snowflake::from(channel_id), pins);
                                error = None;
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::UnpinMessage { channel_id, message_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            if let Err(api_error) = api::message::unpin_message(&*rest_client, &config, channel_id, message_id).await {
                                error = Some(format!("Failed to unpin message: {api_error}"));
                            } else if let Ok(pins) = api::message::get_pinned_messages(&*rest_client, &config, channel_id).await {
                                pinned_messages.insert(Snowflake::from(channel_id), pins);
                                error = None;
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::JoinGuild { invite_code } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            let code = invite_code
                                .trim()
                                .rsplit('/')
                                .next()
                                .unwrap_or(invite_code.trim());
                            match api::guild::join_guild(&*rest_client, &config, code, None).await {
                                Ok(_) => error = None,
                                Err(api_error) => {
                                    error = Some(format!("Failed to join server: {api_error}"));
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::LeaveGuild { guild_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            match api::guild::leave_guild(&*rest_client, &config, guild_id).await {
                                Ok(()) => {
                                    if selected_guild_id == Some(guild_id) {
                                        selected_guild_id = None;
                                        selected_channel_id = None;
                                    }
                                    error = None;
                                }
                                Err(api_error) => {
                                    error = Some(format!("Failed to leave server: {api_error}"));
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::SendFriendRequest { username } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            let (name, disc) = match username.trim().split_once('#') {
                                Some((n, d)) => (n.trim(), Some(d.trim())),
                                None => (username.trim(), None),
                            };
                            match api::user::send_friend_request_by_username(&*rest_client, &config, name, disc).await {
                                Ok(_) => error = None,
                                Err(api_error) => {
                                    error = Some(format!("Failed to send friend request: {api_error}"));
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::AcceptFriendRequest { user_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            match api::user::put_relationship(&*rest_client, &config, user_id, None).await {
                                Ok(()) => error = None,
                                Err(api_error) => {
                                    error = Some(format!("Failed to accept friend request: {api_error}"));
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::RemoveRelationship { user_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            match api::user::delete_relationship(&*rest_client, &config, user_id).await {
                                Ok(()) => error = None,
                                Err(api_error) => {
                                    error = Some(format!("Failed to remove relationship: {api_error}"));
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::BlockUser { user_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            match api::user::put_relationship(&*rest_client, &config, user_id, Some(2)).await {
                                Ok(()) => error = None,
                                Err(api_error) => {
                                    error = Some(format!("Failed to block user: {api_error}"));
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::OpenDm { user_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            match api::channel::create_dm(&*rest_client, &config, user_id).await {
                                Ok(channel) => {
                                    let cid = channel.id;
                                    if let Some(cache) = &cache {
                                        cache.lock().expect("cache lock").upsert_channels(vec![channel]);
                                    }
                                    selected_guild_id = None;
                                    selected_channel_id = Some(cid);
                                    error = None;
                                }
                                Err(api_error) => {
                                    error = Some(format!("Failed to open DM: {api_error}"));
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::SearchMessages { guild_id, channel_id, query } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            let q = query.trim();
                            let result = if let Some(gid) = guild_id {
                                api::message::search_guild_messages(
                                    &*rest_client, &config, gid, Some(q),
                                    &[], &[], &[], None, None, Some("desc"), &[], 0, &[],
                                ).await
                            } else if let Some(cid) = channel_id {
                                api::message::search_channel_messages(
                                    &*rest_client, &config, cid, Some(q),
                                    &[], &[], &[], None, None, Some("desc"), &[], 0,
                                ).await
                            } else {
                                Ok(None)
                            };
                            match result {
                                Ok(Some(body)) => {
                                    let mut found = Vec::new();
                                    if let Some(groups) = body.get("messages").and_then(Value::as_array) {
                                        for group in groups {
                                            if let Some(arr) = group.as_array() {
                                                for item in arr {
                                                    if item.get("hit").and_then(Value::as_bool).unwrap_or(false) {
                                                        if let Ok(msg) = serde_json::from_value::<Message>(item.clone()) {
                                                            found.push(msg);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    search_results = found;
                                    error = None;
                                }
                                Ok(None) => {
                                    search_results.clear();
                                    error = None;
                                }
                                Err(api_error) => {
                                    error = Some(format!("Search failed: {api_error}"));
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::FetchThreads { guild_id: _, channel_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            match api::channel::get_active_threads(&*rest_client, &config, channel_id).await {
                                Ok(body) => {
                                    threads = body
                                        .get("threads")
                                        .and_then(Value::as_array)
                                        .map(|arr| {
                                            arr.iter()
                                                .filter_map(|t| {
                                                    Some(ThreadSnapshot {
                                                        id: json_id(t.get("id"))?,
                                                        parent_id: json_id(t.get("parent_id")),
                                                        name: t.get("name").and_then(Value::as_str).unwrap_or("Thread").to_owned(),
                                                        message_count: t.get("message_count").and_then(Value::as_u64).unwrap_or(0) as u32,
                                                        member_count: t.get("member_count").and_then(Value::as_u64).unwrap_or(0) as u32,
                                                        archived: t.get("thread_metadata").and_then(|m| m.get("archived")).and_then(Value::as_bool).unwrap_or(false),
                                                    })
                                                })
                                                .collect()
                                        })
                                        .unwrap_or_default();
                                    error = None;
                                }
                                Err(api_error) => {
                                    error = Some(format!("Failed to load threads: {api_error}"));
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::CreateThread { channel_id, name, message_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            let res = if let Some(mid) = message_id {
                                api::channel::create_thread_from_message(&*rest_client, &config, channel_id, mid, &name).await
                            } else {
                                api::channel::create_thread(&*rest_client, &config, channel_id, &name, &name, &[]).await
                            };
                            match res {
                                Ok(thread) => {
                                    threads.push(ThreadSnapshot {
                                        id: thread.id,
                                        parent_id: thread.parent_id,
                                        name: thread.name.clone().unwrap_or(name),
                                        message_count: 0,
                                        member_count: 1,
                                        archived: false,
                                    });
                                    if let Some(cache) = &cache {
                                        cache.lock().expect("cache lock").upsert_channels(vec![thread]);
                                    }
                                    error = None;
                                }
                                Err(api_error) => {
                                    error = Some(format!("Failed to create thread: {api_error}"));
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::FetchGuildSettings { guild_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            if let Ok(roles_val) = api::guild::get_guild_roles(&*rest_client, &config, guild_id).await {
                                let roles: Vec<RoleSnapshot> = roles_val
                                    .iter()
                                    .filter_map(|r| {
                                        Some(RoleSnapshot {
                                            id: json_id(r.get("id"))?,
                                            name: r.get("name").and_then(Value::as_str).unwrap_or("").to_owned(),
                                            color: r.get("color").and_then(Value::as_u64).unwrap_or(0) as u32,
                                            position: r.get("position").and_then(Value::as_i64).unwrap_or(0) as i32,
                                            permissions: r.get("permissions").and_then(Value::as_str).unwrap_or("0").to_owned(),
                                        })
                                    })
                                    .collect();
                                guild_roles.insert(guild_id, roles);
                            }
                            if let Ok(emojis_val) = api::guild::get_guild_emojis(&*rest_client, &config, guild_id).await {
                                let emojis: Vec<EmojiSnapshot> = emojis_val
                                    .iter()
                                    .filter_map(|e| {
                                        Some(EmojiSnapshot {
                                            id: json_id(e.get("id"))?,
                                            name: e.get("name").and_then(Value::as_str).unwrap_or("").to_owned(),
                                            animated: e.get("animated").and_then(Value::as_bool).unwrap_or(false),
                                            guild_id,
                                        })
                                    })
                                    .collect();
                                guild_emojis.insert(guild_id, emojis);
                            }
                            if let Ok(bans_val) = api::guild::get_guild_bans(&*rest_client, &config, guild_id).await {
                                let bans: Vec<BanSnapshot> = bans_val
                                    .iter()
                                    .filter_map(|b| {
                                        let user = b.get("user")?;
                                        Some(BanSnapshot {
                                            user_id: json_id(user.get("id"))?,
                                            username: user.get("username").and_then(Value::as_str).unwrap_or("unknown").to_owned(),
                                            avatar: user.get("avatar").and_then(Value::as_str).map(str::to_owned),
                                            reason: b.get("reason").and_then(Value::as_str).map(str::to_owned),
                                        })
                                    })
                                    .collect();
                                guild_bans.insert(guild_id, bans);
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::CreateGuildRole { guild_id, name, color } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            match api::guild::create_role(&*rest_client, &config, guild_id).await {
                                Ok(created) => {
                                    if let Some(role_id) = json_id(created.get("id")) {
                                        let body = serde_json::json!({ "name": name, "color": color });
                                        let _ = api::guild::update_role(&*rest_client, &config, guild_id, role_id, &body).await;
                                        guild_roles.entry(guild_id).or_default().push(RoleSnapshot {
                                            id: role_id,
                                            name,
                                            color,
                                            position: 0,
                                            permissions: "0".to_owned(),
                                        });
                                    }
                                    error = None;
                                }
                                Err(api_error) => {
                                    error = Some(format!("Failed to create role: {api_error}"));
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::DeleteGuildRole { guild_id, role_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            match api::guild::delete_role(&*rest_client, &config, guild_id, role_id).await {
                                Ok(()) => {
                                    if let Some(roles) = guild_roles.get_mut(&guild_id) {
                                        roles.retain(|r| r.id != role_id);
                                    }
                                    error = None;
                                }
                                Err(api_error) => {
                                    error = Some(format!("Failed to delete role: {api_error}"));
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::DeleteGuildEmoji { guild_id, emoji_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            match api::guild::delete_emoji(&*rest_client, &config, guild_id, emoji_id).await {
                                Ok(()) => {
                                    if let Some(emojis) = guild_emojis.get_mut(&guild_id) {
                                        emojis.retain(|e| e.id != emoji_id);
                                    }
                                    error = None;
                                }
                                Err(api_error) => {
                                    error = Some(format!("Failed to delete emoji: {api_error}"));
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::UnbanMember { guild_id, user_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            match api::guild::unban_member(&*rest_client, &config, guild_id, user_id).await {
                                Ok(()) => {
                                    if let Some(bans) = guild_bans.get_mut(&guild_id) {
                                        bans.retain(|b| b.user_id != user_id);
                                    }
                                    error = None;
                                }
                                Err(api_error) => {
                                    error = Some(format!("Failed to unban member: {api_error}"));
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::CreateChannel { guild_id, name, kind, parent_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            match api::channel::create_guild_channel(&*rest_client, &config, guild_id, &name, kind, parent_id).await {
                                Ok(channel) => {
                                    if let Some(cache) = &cache {
                                        cache.lock().expect("cache lock").upsert_channels(vec![channel]);
                                    }
                                    error = None;
                                }
                                Err(api_error) => {
                                    error = Some(format!("Failed to create channel: {api_error}"));
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::DeleteChannel { channel_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            match api::channel::delete_channel(&*rest_client, &config, channel_id).await {
                                Ok(_) => {
                                    if selected_channel_id == Some(channel_id) {
                                        selected_channel_id = None;
                                    }
                                    error = None;
                                }
                                Err(api_error) => {
                                    error = Some(format!("Failed to delete channel: {api_error}"));
                                }
                            }
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::SwitchAccount { user_id } => {
                        if let Some(account) = token_store.accounts().iter().find(|a| a.user_id == user_id).cloned() {
                            if let Err(message) = connect_session!(account.token) {
                                connection = ConnectionState::Error { message: message.clone() };
                                error = Some(format!("Account switch failed: {message}"));
                                should_reconnect = false;
                            }
                        } else {
                            error = Some("Account not found".to_owned());
                        }
                        mark_snapshot_dirty!();
                    }
                    ClientCommand::RemoveSavedAccount { user_id } => {
                        if let Some(token) = token_store.accounts().iter().find(|a| a.user_id == user_id).map(|a| a.token.clone()) {
                            forget_token(&mut token_store, &secret_store, &token);
                        }
                        mark_snapshot_dirty!();
                    }
                }
                flush_snapshot!(true);
            }
            maybe_remote = async {
                match remote_events.as_mut() {
                    Some(rx) => rx.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                let Some(state) = maybe_remote else { remote_events = None; continue; };
                let dto = match state {
                    CoreRemoteAuthState::Idle => RemoteAuthState::Idle,
                    CoreRemoteAuthState::Connecting => RemoteAuthState::Connecting,
                    CoreRemoteAuthState::QrReady { url } => RemoteAuthState::QrReady { url },
                    CoreRemoteAuthState::UserScanned { user } => RemoteAuthState::UserScanned {
                        user_id: user.id,
                        username: user.global_name.unwrap_or(user.username),
                    },
                    CoreRemoteAuthState::Finished { token } => {
                        if let Err(message) = connect_session!(token.clone()) {
                            RemoteAuthState::Error { message }
                        } else {
                            update_auth!(AuthChallenge::Success { token: token.clone() });
                            RemoteAuthState::Finished { token }
                        }
                    }
                    CoreRemoteAuthState::Error { message } => RemoteAuthState::Error { message },
                };
                let terminal = matches!(dto, RemoteAuthState::Finished { .. } | RemoteAuthState::Error { .. });
                auth_state.remote = dto.clone();
                push_event(&events, ClientEvent::RemoteAuthChanged(dto));
                if terminal {
                    if let Some(cancel) = remote_cancel.take() { let _ = cancel.send(()); }
                    if let Some(task) = remote_task.take() { task.abort(); }
                    remote_events = None;
                }
                mark_snapshot_dirty!();
                flush_snapshot!(true);
            }
            maybe_gw = async {
                match gateway_events.as_mut() {
                    Some(rx) => rx.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                let Some(event) = maybe_gw else { gateway_events = None; continue; };
                let prev_connection = connection.clone();
                let snapshot_changed = match &event {
                    GatewayEvent::Hello { .. }
                    | GatewayEvent::HandlerErrors { .. }
                    | GatewayEvent::HeartbeatSent
                    | GatewayEvent::HeartbeatAck => false,
                    GatewayEvent::Dispatch { event_type, .. } => {
                        SNAPSHOT_DISPATCH_EVENTS.contains(&event_type.as_str())
                    }
                    _ => true,
                };
                match event {
                    GatewayEvent::Hello { .. } => {
                        connection = ConnectionState::Connecting;
                    }
                    GatewayEvent::Ready { .. } => {
                        connection = ConnectionState::Ready;
                        error = None;
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, user_agent.as_deref(), active_token.clone());
                            let fetch_tx = fetch_tx.clone();
                            tokio::spawn(async move {
                                if let Ok(channels) = api::channel::get_private_channels(&*rest_client, &config).await {
                                    let _ = fetch_tx.send(InternalEvent::PrivateChannels(channels));
                                }
                            });
                        }
                        if let Some(guild_id) = selected_guild_id {
                            subscribe_guild_members(guild_id, &cache, &gateway_outbound);
                        }
                    }
                    GatewayEvent::Resumed => {
                        connection = ConnectionState::Ready;
                    }
                    GatewayEvent::AuthFailed { .. } => {
                        connection = ConnectionState::Error { message: "Authentication failed".to_owned() };
                        error = Some("Authentication failed".to_owned());
                        should_reconnect = false;
                    }
                    GatewayEvent::Closed { .. } | GatewayEvent::ReconnectRequested | GatewayEvent::InvalidSession { .. } => {
                        connection = if should_reconnect {
                            ConnectionState::Resuming
                        } else {
                            ConnectionState::Disconnected
                        };
                    }
                    GatewayEvent::Dispatch { event_type, data } => {
                        handle_dispatch(&event_type, data.as_deref(), &events, cache.as_ref());
                    }
                    GatewayEvent::Failed { message } => {
                        push_event(&events, ClientEvent::Error { message });
                    }
                    GatewayEvent::HandlerErrors { errors } => {
                        for handler_error in errors {
                            push_event(&events, ClientEvent::CommandFailed {
                                kind: FfiErrorKind::Protocol,
                                message: format!("{} handler failed on {}: {}", handler_error.handler, handler_error.event, handler_error.message),
                            });
                        }
                    }
                    GatewayEvent::HeartbeatSent | GatewayEvent::HeartbeatAck => {}
                }
                let connection_changed = connection != prev_connection;
                if connection_changed {
                    push_event(&events, ClientEvent::ConnectionChanged(connection.clone()));
                }
                if snapshot_changed || connection_changed {
                    mark_snapshot_dirty!();
                }
                flush_snapshot!(false);
            }
            _ = persist_ticker.tick() => {
                if let (Some(live), Some(file)) = (cache.clone(), cache_file.clone()) {
                    let persisted = live.lock().expect("cache lock").take_persisted_if_dirty();
                    if let Some(persisted) = persisted {
                        tokio::task::spawn_blocking(move || {
                            let Ok(bytes) = persisted.to_bytes() else {
                                return;
                            };
                            let temp = file.with_extension("tmp");
                            if std::fs::write(&temp, bytes).is_ok() {
                                let _ = std::fs::rename(&temp, &file);
                            }
                        });
                    }
                }
            }
            _ = flush_ticker.tick() => {
                flush_snapshot!(false);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn partial_message_update_still_emits_a_typed_event() {
        let events: EventQueue = Arc::new(Mutex::new(Vec::new()));
        handle_dispatch(
            "MESSAGE_UPDATE",
            Some(&json!({
                "id": "42",
                "channel_id": "7",
                "content": "edited",
                "edited_timestamp": "2026-01-01T00:00:00Z"
            })),
            &events,
            None,
        );

        let queued = events.lock().expect("events lock");
        assert_eq!(queued.len(), 1);
        match &queued[0].event {
            ClientEvent::MessageUpdated(message) => {
                assert_eq!(message.id, 42);
                assert_eq!(message.channel_id, 7);
                assert_eq!(message.content, "edited");
                assert_eq!(message.author_name, "unknown");
                assert!(message.edited_timestamp.is_some());
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn partial_message_update_preserves_cached_fields() {
        let events: EventQueue = Arc::new(Mutex::new(Vec::new()));
        let cache = Arc::new(Mutex::new(StateCache::new()));
        cache
            .lock()
            .expect("cache lock")
            .handle_event(
                "MESSAGE_CREATE",
                Some(&json!({
                    "id": "42",
                    "channel_id": "7",
                    "author": { "id": "9", "username": "alice" },
                    "content": "original",
                    "timestamp": "2026-01-01T00:00:00Z"
                })),
            )
            .expect("seed cached message");

        handle_dispatch(
            "MESSAGE_UPDATE",
            Some(&json!({
                "id": "42",
                "channel_id": "7",
                "content": "edited",
                "edited_timestamp": "2026-01-02T00:00:00Z"
            })),
            &events,
            Some(&cache),
        );

        let queued = events.lock().expect("events lock");
        assert_eq!(queued.len(), 1);
        match &queued[0].event {
            ClientEvent::MessageUpdated(message) => {
                assert_eq!(message.content, "edited");
                assert_eq!(message.author_id, Some(9));
                assert_eq!(message.author_name, "alice");
                assert_eq!(message.timestamp, "2026-01-01T00:00:00Z");
                assert_eq!(
                    message.edited_timestamp.as_deref(),
                    Some("2026-01-02T00:00:00Z")
                );
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn profiles_prefer_the_server_member_profile() {
        let value = serde_json::json!({
            "user": {"id": "7", "username": "dim", "global_name": "DiM", "avatar": "abc", "banner": "acct",
                     "avatar_decoration_data": {"asset": "deco"}},
            "user_profile": {"bio": "account bio", "pronouns": "they/them", "theme_colors": [1, 2]},
            "guild_member_profile": {"bio": "server bio", "banner": "srv"},
            "guild_member": {"nick": "dimmy", "joined_at": "2024-01-01T00:00:00Z"},
            "badges": [{"id": "hypesquad", "description": "HypeSquad", "icon": "h"}],
            "mutual_guilds": [{"id": "9"}],
            "mutual_friends_count": 3
        });
        let profile = profile_snapshot(&value, 7, Some(42));
        assert_eq!(profile.bio.as_deref(), Some("server bio"));
        assert_eq!(profile.pronouns.as_deref(), Some("they/them"));
        assert_eq!(profile.theme_colors, vec![1, 2]);
        assert_eq!(profile.nickname.as_deref(), Some("dimmy"));
        assert_eq!(profile.decoration.as_deref(), Some("deco"));
        assert!(
            profile
                .banner_url
                .unwrap()
                .contains("/guilds/42/users/7/banners/srv.png")
        );
        assert_eq!(profile.mutual_guild_ids, vec![9]);
        assert_eq!(profile.badges[0].description, "HypeSquad");
    }

    #[test]
    fn guild_channel_backfill_populates_selected_guild_snapshot() {
        let cache = Arc::new(Mutex::new(StateCache::new()));
        let guild_id: Snowflake = 7;
        let channels: Vec<Channel> = vec![
            serde_json::from_value(json!({
                "id": "11",
                "type": 4,
                "name": "Info",
                "position": 0
            }))
            .expect("category"),
            serde_json::from_value(json!({
                "id": "10",
                "type": 0,
                "name": "general",
                "position": 1
            }))
            .expect("text channel"),
        ];

        cache_guild_channels(&cache, guild_id, channels);
        let snapshot = build_snapshot(
            ConnectionState::Ready,
            true,
            &Some(cache),
            Some(guild_id),
            None,
            &HashMap::new(),
            &HashMap::new(),
            &[],
            &[],
            &HashMap::new(),
            &HashMap::new(),
            &[],
            &HashMap::new(),
            &[],
            &None,
            &AuthStateSnapshot::default(),
            None,
        );

        let names: Vec<&str> = snapshot
            .channels
            .iter()
            .map(|channel| channel.name.as_str())
            .collect();
        assert_eq!(names, ["Info", "general"]);
        assert!(
            snapshot
                .channels
                .iter()
                .all(|channel| channel.guild_id == Some(guild_id))
        );
    }

    #[test]
    fn auth_response_maps_captcha_and_mfa_challenges() {
        let captcha = auth_challenge_from_response(auth::LoginResponse {
            body: Some(
                json!({
                    "captcha_key": ["invalid-input-response"],
                    "captcha_sitekey": "site-key",
                    "captcha_service": "hcaptcha"
                })
                .to_string(),
            ),
            ..auth::LoginResponse::default()
        });
        match captcha {
            AuthChallenge::CaptchaRequired {
                sitekey, service, ..
            } => {
                assert_eq!(sitekey, "site-key");
                assert_eq!(service, "hcaptcha");
            }
            other => panic!("unexpected challenge: {other:?}"),
        }

        let mfa = auth_challenge_from_response(auth::LoginResponse {
            ticket: Some("ticket".to_owned()),
            totp: Some(true),
            sms: Some(true),
            ..auth::LoginResponse::default()
        });
        match mfa {
            AuthChallenge::MfaRequired { ticket, methods } => {
                assert_eq!(ticket, "ticket");
                assert_eq!(methods, vec!["totp".to_owned(), "sms".to_owned()]);
            }
            other => panic!("unexpected challenge: {other:?}"),
        }
    }

    #[test]
    fn rest_config_uses_client_super_properties_for_mfa() {
        let config = default_rest_config("https://example.invalid/api/v9", None, None);
        let headers = config.login_headers(None);
        let encoded = headers
            .iter()
            .find(|(name, _)| name == "X-Super-Properties")
            .map(|(_, value)| value.as_str())
            .expect("super properties header");
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .expect("base64 super properties");
        let properties: Value = serde_json::from_slice(&decoded).expect("json super properties");

        if cfg!(target_os = "android") {
            assert_eq!(properties["client_build_number"], json!(6081));
            assert_eq!(properties["browser"], "Discord Android");
        } else {
            assert_eq!(properties["client_build_number"], json!(575562));
            assert_eq!(properties["browser"], "Discord Client");
            assert_eq!(properties["device"], "Desktop");
            assert_eq!(properties["browser_version"], "37.6.0");
        }
        assert_eq!(properties["design_id"], json!(2));
        assert!(properties["device_vendor_id"].as_str().is_some_and(|id| id.len() == 36));

        let other = default_rest_config("https://example.invalid/api/v9", None, None);
        let installation = |headers: &[(String, String)]| {
            headers
                .iter()
                .find(|(name, _)| name == "X-Installation-Id")
                .map(|(_, value)| value.clone())
        };
        assert_eq!(installation(&headers), installation(&other.login_headers(None)));
    }

    #[test]
    fn mfa_flow_keeps_login_instance_and_fingerprint_for_ticket() {
        let mut flows = HashMap::new();
        remember_mfa_flow(
            &mut flows,
            &AuthChallenge::MfaRequired {
                ticket: "ticket-1".to_owned(),
                methods: vec!["totp".to_owned()],
            },
            Some("instance-1".to_owned()),
            Some("fingerprint-1".to_owned()),
        );

        let flow = flows.get("ticket-1").expect("stored MFA flow");
        assert_eq!(flow.login_instance_id.as_deref(), Some("instance-1"));
        assert_eq!(flow.fingerprint.as_deref(), Some("fingerprint-1"));

        remember_mfa_flow(
            &mut flows,
            &AuthChallenge::Success {
                token: "token".to_owned(),
            },
            None,
            None,
        );
        assert!(flows.is_empty());
    }

    #[test]
    fn ephemeral_events_are_coalesced_while_queued() {
        let events: EventQueue = Arc::new(Mutex::new(Vec::new()));
        for _ in 0..3 {
            push_event(
                &events,
                ClientEvent::TypingStarted {
                    channel_id: 1,
                    user_id: 2,
                },
            );
        }
        push_event(
            &events,
            ClientEvent::TypingStarted {
                channel_id: 1,
                user_id: 3,
            },
        );
        let queued = events.lock().expect("events lock");
        assert_eq!(queued.len(), 2);
        assert!(
            queued
                .windows(2)
                .all(|pair| pair[0].sequence < pair[1].sequence)
        );
    }

    #[test]
    fn low_priority_events_are_dropped_before_high_priority_events() {
        let events: EventQueue = Arc::new(Mutex::new(Vec::new()));
        for index in 0..MAX_EVENT_QUEUE - 1 {
            push_event(
                &events,
                ClientEvent::CommandFailed {
                    kind: FfiErrorKind::Protocol,
                    message: index.to_string(),
                },
            );
        }
        push_event(
            &events,
            ClientEvent::TypingStarted {
                channel_id: 1,
                user_id: 999,
            },
        );
        push_event(
            &events,
            ClientEvent::TypingStarted {
                channel_id: 1,
                user_id: 1000,
            },
        );

        let queued = events.lock().expect("events lock");
        assert_eq!(
            queued
                .iter()
                .filter(|envelope| matches!(envelope.event, ClientEvent::TypingStarted { .. }))
                .count(),
            1
        );
        assert_eq!(
            queued
                .iter()
                .filter(|envelope| matches!(envelope.event, ClientEvent::CommandFailed { .. }))
                .count(),
            MAX_EVENT_QUEUE - EVENT_DROP_BATCH
        );
    }
}
