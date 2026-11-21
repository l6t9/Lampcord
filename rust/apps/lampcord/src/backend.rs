use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

use lampcord_core::{
    gateway_client::{GatewayClient, GatewayEvent},
    gateway_plan::reconnect_delay_ms,
    remote_auth::{self, RemoteAuthState},
    rest::RestConfig,
    rest_client::RestClient,
    state_cache::StateCache,
    token_store::{SavedAccount, TokenStore},
    traits::CaptchaSolver,
};
use lampcord_model::{channel::Channel, guild::Guild, message::Message, snowflake::Snowflake};
use lampcord_transport::{http::ReqwestRestTransport, websocket::TungsteniteGatewayTransport};

use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum BackendCommand {
    Connect {
        token: String,
    },
    SelectGuild {
        guild_id: Option<Snowflake>,
    },
    SelectChannel {
        channel_id: Option<Snowflake>,
    },
    FetchMessages {
        channel_id: Snowflake,
        before: Option<Snowflake>,
    },
    SendMessage {
        channel_id: Snowflake,
        content: String,
        reply_to_message_id: Option<u64>,
        reply_to_guild_id: Option<u64>,
    },
    EditMessage {
        channel_id: Snowflake,
        message_id: u64,
        content: String,
    },
    PersistToken {
        token: String,
        user_id: String,
        username: Option<String>,
    },
    StartRemoteAuth,
    StopRemoteAuth,
    LoginPassword {
        login: String,
        password: String,
    },
    SubmitMfa {
        code: String,
    },
    SetMfaMethod {
        method: String,
    },
    ResendMfaSms,
    CancelLoginFlow,
    ForgotPassword {
        login: String,
    },
    ClearForgotPassword,
    LoadDeviceSessions,
    LogoutDeviceSessions {
        id_hashes: Vec<String>,
    },
    SubmitDeviceSessionPassword {
        password: String,
    },
    SubmitDeviceSessionCode {
        code: String,
    },
    UseDeviceSessionBackupCode,
    CancelDeviceSessionChallenge,
    Logout,
    Shutdown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GuildFolderView {
    /// Discord folder id. `None` means an ungrouped/virtual folder entry in some payloads;
    /// we treat `None` as non-collapsible inline entries like KMP.
    pub id: Option<i64>,
    pub name: Option<String>,
    /// ARGB/RGB integer from Discord settings (0 means default in practice).
    pub color: Option<u32>,
    pub guild_ids: Vec<Snowflake>,
}

#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub status: String,
    pub current_user: Option<String>,
    pub current_user_id: Option<Snowflake>,
    pub current_user_avatar_url: Option<String>,
    pub guilds: Vec<GuildView>,
    pub guild_folders: Vec<GuildFolderView>,
    pub channels: Vec<ChannelView>,
    pub messages: Vec<MessageView>,
    pub selected_guild_id: Option<Snowflake>,
    pub selected_channel_id: Option<Snowflake>,
    pub selected_channel_last_read_id: Option<u64>,
    pub saved_accounts: Vec<SavedAccount>,
    pub error: Option<String>,
    pub remote_auth: RemoteAuthView,
    pub login_flow: LoginFlowView,
    pub forgot_password: ForgotPasswordView,
    pub device_sessions: DeviceSessionsView,
}

#[derive(Debug, Clone, Default)]
pub struct RemoteAuthView {
    pub status: String,
    pub qr_url: Option<String>,
    pub scanned_user: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoginFlowView {
    pub stage: String, // "", "mfa", "busy"
    pub error: Option<String>,
    pub ticket: Option<String>,
    pub login_instance_id: Option<String>,
    pub mfa_methods: Vec<String>,
    pub selected_mfa_method: Option<String>,
    pub sms_phone: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ForgotPasswordView {
    pub stage: String, // "", "busy", "sent", "phone_required", "failed"
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceSessionView {
    pub id_hash: String,
    pub name: String,
    pub location: Option<String>,
    pub last_used: Option<String>,
    pub current: bool,
    pub icon: String, // "mobile" | "desktop" | "unknown"
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceSessionChallenge {
    Password {
        pending_id_hashes: Vec<String>,
        message: Option<String>,
    },
    Totp {
        pending_id_hashes: Vec<String>,
        ticket: String,
        mfa_type: String, // usually "totp"
        allow_backup: bool,
        password: Option<String>,
        message: Option<String>,
    },
    BackupCode {
        pending_id_hashes: Vec<String>,
        password: Option<String>,
        message: Option<String>,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeviceSessionsView {
    pub loading: bool,
    pub sessions: Vec<DeviceSessionView>,
    pub error: Option<String>,
    pub status: Option<String>,
    pub challenge: Option<DeviceSessionChallenge>,
}

#[derive(Debug, Clone)]
pub struct GuildView {
    pub id: Snowflake,
    pub name: String,
    pub icon_url: Option<String>,
    pub unread: bool,
    pub mention_count: u32,
    pub member_count: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ChannelView {
    pub id: Snowflake,
    pub name: String,
    pub kind: u8,
    pub icon_url: Option<String>,
    pub parent_id: Option<Snowflake>,
    pub position: u16,
    pub unread: bool,
    pub mention_count: u32,
}

#[derive(Debug, Clone)]
pub struct AttachmentView {
    pub filename: String,
    pub url: String,
    pub content_type: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub size: u64,
}

#[derive(Debug, Clone, Default)]
pub struct EmbedView {
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub author_name: Option<String>,
    pub thumbnail_url: Option<String>,
    pub image_url: Option<String>,
    pub footer_text: Option<String>,
    pub color: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ReplyPreviewView {
    pub author: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct MessageView {
    pub id: Snowflake,
    pub author_id: Snowflake,
    pub author: String,
    pub author_avatar_url: Option<String>,
    pub content: String,
    pub timestamp: String,
    pub edited: bool,
    pub attachment_count: usize,
    pub embed_count: usize,
    pub reply: bool,
    pub reply_preview: Option<ReplyPreviewView>,
    pub attachments: Vec<AttachmentView>,
    pub embeds: Vec<EmbedView>,
    pub sticker_names: Vec<String>,
    pub reaction_count: usize,
}

#[derive(Debug, Clone)]
pub enum BackendEvent {
    Snapshot(Snapshot),
}

struct NoopCaptchaSolver;

#[async_trait::async_trait]
impl CaptchaSolver for NoopCaptchaSolver {
    async fn solve(&self, _challenge: &lampcord_core::captcha::CaptchaChallenge) -> Option<String> {
        None
    }
}

struct StaticGatewayUrl {
    rest: Arc<RestClient<ReqwestRestTransport, NoopCaptchaSolver>>,
}

#[async_trait::async_trait]
impl lampcord_core::traits::GatewayUrlProvider for StaticGatewayUrl {
    async fn gateway_url(&self) -> Option<String> {
        self.rest.gateway_url().await
    }
}

#[derive(Clone)]
pub struct BackendHandle {
    pub tx: mpsc::UnboundedSender<BackendCommand>,
    pub rx: Arc<Mutex<mpsc::UnboundedReceiver<BackendEvent>>>,
    pub snapshot: Arc<Mutex<Snapshot>>,
}

impl BackendHandle {
    pub fn send(&self, cmd: BackendCommand) {
        let _ = self.tx.send(cmd);
    }

    pub fn drain_events(&self) -> bool {
        let mut changed = false;
        if let Ok(mut rx) = self.rx.lock() {
            while let Ok(event) = rx.try_recv() {
                match event {
                    BackendEvent::Snapshot(snapshot) => {
                        if let Ok(mut current) = self.snapshot.lock() {
                            *current = snapshot;
                            changed = true;
                        }
                    }
                }
            }
        }
        changed
    }

    pub fn snapshot(&self) -> Snapshot {
        self.snapshot.lock().map(|s| s.clone()).unwrap_or_default()
    }
}

pub fn start(token_store_path: PathBuf) -> BackendHandle {
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<BackendCommand>();
    let (evt_tx, evt_rx) = mpsc::unbounded_channel::<BackendEvent>();
    let snapshot = Arc::new(Mutex::new(Snapshot {
        status: "Idle".to_owned(),
        saved_accounts: load_token_store(&token_store_path).accounts().to_vec(),
        ..Snapshot::default()
    }));
    let snapshot_for_thread = snapshot.clone();

    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()
            .expect("tokio runtime");
        runtime.block_on(run(cmd_rx, evt_tx, snapshot_for_thread, token_store_path));
    });

    BackendHandle {
        tx: cmd_tx,
        rx: Arc::new(Mutex::new(evt_rx)),
        snapshot,
    }
}

fn load_token_store(path: &PathBuf) -> TokenStore {
    match std::fs::read_to_string(path) {
        Ok(body) => TokenStore::from_json(&body),
        Err(_) => TokenStore::new(),
    }
}

fn save_token_store(path: &PathBuf, store: &TokenStore) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, store.to_json());
}

fn config_dir_from_token_store(token_store_path: &PathBuf) -> PathBuf {
    token_store_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn installation_id_path(token_store_path: &PathBuf) -> PathBuf {
    config_dir_from_token_store(token_store_path).join("installation_id")
}

fn device_vendor_id_path(token_store_path: &PathBuf) -> PathBuf {
    config_dir_from_token_store(token_store_path).join("device_vendor_id")
}

fn load_or_create_stable_id(path: &PathBuf) -> String {
    if let Ok(existing) = std::fs::read_to_string(path) {
        let trimmed = existing.trim();
        if !trimmed.is_empty() {
            return trimmed.to_owned();
        }
    }
    let id = random_uuid();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, &id);
    id
}

fn load_or_create_installation_id(token_store_path: &PathBuf) -> String {
    load_or_create_stable_id(&installation_id_path(token_store_path))
}

fn load_or_create_device_vendor_id(token_store_path: &PathBuf) -> String {
    load_or_create_stable_id(&device_vendor_id_path(token_store_path))
}

fn random_uuid() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    )
}

fn detect_timezone() -> String {
    if let Ok(tz) = std::env::var("TZ") {
        let tz = tz.trim();
        if !tz.is_empty() {
            return tz.to_owned();
        }
    }
    if let Ok(contents) = std::fs::read_to_string("/etc/timezone") {
        let tz = contents.trim();
        if !tz.is_empty() {
            return tz.to_owned();
        }
    }
    if let Ok(link) = std::fs::read_link("/etc/localtime") {
        let s = link.to_string_lossy();
        if let Some(idx) = s.find("zoneinfo/") {
            let tz = &s[idx + "zoneinfo/".len()..];
            if !tz.is_empty() {
                return tz.to_owned();
            }
        }
    }
    "UTC".to_owned()
}

fn device_name() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::fs::read_to_string("/etc/hostname").map(|s| s.trim().to_owned()))
        .unwrap_or_else(|_| "Linux".to_owned())
}

fn os_version() -> String {
    std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|s| {
            s.lines()
                .find_map(|line| line.strip_prefix("PRETTY_NAME="))
                .map(|v| v.trim_matches('"').to_owned())
        })
        .or_else(|| {
            std::process::Command::new("uname")
                .arg("-r")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| "Linux".to_owned())
}

#[derive(Debug, Clone)]
struct ClientIdentity {
    installation_id: String,
    device_vendor_id: String,
    timezone: String,
    super_properties: String,
}

impl ClientIdentity {
    fn load(token_store_path: &PathBuf) -> Self {
        let installation_id = load_or_create_installation_id(token_store_path);
        let device_vendor_id = load_or_create_device_vendor_id(token_store_path);
        let timezone = detect_timezone();
        let super_properties = build_super_properties(&device_vendor_id);
        Self {
            installation_id,
            device_vendor_id,
            timezone,
            super_properties,
        }
    }
}

fn build_super_properties(device_vendor_id: &str) -> String {
    use base64::Engine;
    let props = serde_json::json!({
        "os": "Linux",
        "browser": "Discord Client",
        "device": device_name(),
        "system_locale": "en-US",
        "has_client_mods": false,
        "client_version": "0.0.398",
        "release_channel": "stable",
        "device_vendor_id": device_vendor_id,
        "design_id": 2,
        "browser_user_agent": "",
        "browser_version": "37.6.0",
        "os_version": os_version(),
        "client_build_number": 575562,
        "client_event_source": serde_json::Value::Null,
        "client_launch_id": serde_json::Value::Null,
        "launch_signature": "",
        "client_app_state": serde_json::Value::Null,
        "client_heartbeat_session_id": serde_json::Value::Null,
    });
    let json = props.to_string();
    base64::engine::general_purpose::STANDARD.encode(json.as_bytes())
}

fn rest_config(token: Option<String>, identity: &ClientIdentity) -> RestConfig {
    RestConfig::new(
        token,
        identity.super_properties.clone(),
        identity.installation_id.clone(),
        identity.timezone.clone(),
    )
}

fn extract_embed_view(v: &serde_json::Value) -> EmbedView {
    let get_str = |key: &str| v.get(key).and_then(|x| x.as_str()).map(|s| s.to_owned());
    let title = get_str("title");
    let description = get_str("description");
    let url = get_str("url");
    let author_name = v
        .get("author")
        .and_then(|a| a.get("name"))
        .and_then(|x| x.as_str())
        .map(|s| s.to_owned());
    let thumbnail_url = v
        .get("thumbnail")
        .and_then(|t| t.get("url").or_else(|| t.get("proxy_url")))
        .and_then(|x| x.as_str())
        .map(|s| s.to_owned());
    let image_url = v
        .get("image")
        .and_then(|t| t.get("url").or_else(|| t.get("proxy_url")))
        .and_then(|x| x.as_str())
        .map(|s| s.to_owned());
    let footer_text = v
        .get("footer")
        .and_then(|f| f.get("text"))
        .and_then(|x| x.as_str())
        .map(|s| s.to_owned());
    let color = v.get("color").and_then(|c| c.as_u64()).map(|c| c as u32);
    EmbedView {
        title,
        description,
        url,
        author_name,
        thumbnail_url,
        image_url,
        footer_text,
        color,
    }
}

fn reply_preview_from_message(m: &lampcord_model::message::Message) -> ReplyPreviewView {
    let author = m
        .author
        .global_name
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| m.author.username.clone());
    let mut content = m.content.trim().replace('\n', " ");
    if content.is_empty() {
        if !m.attachments.is_empty() {
            content = "Click to see attachment".to_owned();
        } else if !m.embeds.is_empty() {
            content = "Click to see embed".to_owned();
        } else {
            content = "Original message".to_owned();
        }
    }
    if content.chars().count() > 80 {
        content = content.chars().take(80).collect::<String>() + "…";
    }
    ReplyPreviewView { author, content }
}

fn parse_guild_folders_from_settings(settings: &serde_json::Value) -> Vec<GuildFolderView> {
    let Some(folders) = settings.get("guild_folders").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    folders
        .iter()
        .filter_map(|f| {
            let guild_ids = f
                .get("guild_ids")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| match x {
                            serde_json::Value::String(s) => s.parse::<u64>().ok(),
                            serde_json::Value::Number(n) => n.as_u64(),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if guild_ids.is_empty() && f.get("id").and_then(|v| v.as_i64()).is_none() {}
            let id = f.get("id").and_then(|v| v.as_i64());
            let name = f
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
                .filter(|s| !s.is_empty());
            let color = f.get("color").and_then(|v| v.as_u64()).map(|c| c as u32);
            Some(GuildFolderView {
                id,
                name,
                color,
                guild_ids,
            })
        })
        .collect()
}

async fn refresh_guild_metadata_and_folders(
    rest_client: &Arc<RestClient<ReqwestRestTransport, NoopCaptchaSolver>>,
    config: &RestConfig,
    cache: &Option<Arc<Mutex<StateCache>>>,
) -> (Option<Vec<GuildFolderView>>, Option<String>) {
    let mut folders_out: Option<Vec<GuildFolderView>> = None;
    let mut err: Option<String> = None;

    match lampcord_core::api::guild::get_user_guilds(&**rest_client, config).await {
        Ok(guilds) => {
            if let Some(cache_arc) = cache {
                if let Ok(mut c) = cache_arc.lock() {
                    for guild in guilds {
                        if let Ok(value) = serde_json::to_value(&guild) {
                            let _ = c.handle_event("GUILD_UPDATE", Some(&value));
                        }
                    }
                }
            }
        }
        Err(e) => {
            err = Some(format!("Guild metadata refresh failed: {e}"));
        }
    }

    match lampcord_core::api::user::get_user_settings(&**rest_client, config).await {
        Ok(settings) => {
            folders_out = Some(parse_guild_folders_from_settings(&settings));
        }
        Err(e) => {
            if err.is_none() {
                err = Some(format!("User settings refresh failed: {e}"));
            }
        }
    }

    (folders_out, err)
}

fn render_snapshot(
    status: impl Into<String>,
    cache: &Arc<Mutex<StateCache>>,
    selected_guild_id: Option<Snowflake>,
    selected_channel_id: Option<Snowflake>,
    fetched_messages: &HashMap<Snowflake, Vec<Message>>,
    saved_accounts: Vec<SavedAccount>,
    guild_folders: &[GuildFolderView],
    error: Option<String>,
    remote_auth: RemoteAuthView,
    login_flow: LoginFlowView,
    forgot_password: ForgotPasswordView,
    device_sessions: DeviceSessionsView,
) -> Snapshot {
    use crate::ui::cdn;
    let cache = cache.lock().expect("cache lock");
    let current_user_id = cache.current_user().map(|u| u.id);
    let current_user = cache.current_user().map(|u| {
        u.global_name
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| u.username.clone())
    });
    let current_user_avatar_url = cache
        .current_user()
        .map(|u| cdn::user_avatar_url(u.id, u.avatar.as_deref(), 64));

    let read_state = cache.read_state();
    let mut guild_unread: HashMap<Snowflake, bool> = HashMap::new();
    let mut guild_mentions: HashMap<Snowflake, u32> = HashMap::new();
    for c in cache.channels() {
        let Some(gid) = c.guild_id else {
            continue;
        };
        if read_state.is_unread(c.id, c.last_message_id, None) {
            guild_unread.insert(gid, true);
        }
        let mentions = read_state.mention_count(c.id);
        if mentions > 0 {
            let entry = guild_mentions.entry(gid).or_insert(0);
            *entry = entry.saturating_add(mentions);
        }
    }

    let guilds = cache
        .ordered_guilds()
        .into_iter()
        .map(|g: &Guild| GuildView {
            id: g.id,
            name: g.name.clone().unwrap_or_else(|| format!("Guild {}", g.id)),
            icon_url: cdn::guild_icon_url(g.id, g.icon.as_deref(), 96),
            unread: guild_unread.get(&g.id).copied().unwrap_or(false),
            mention_count: guild_mentions.get(&g.id).copied().unwrap_or(0),
            member_count: g.member_count,
        })
        .collect::<Vec<_>>();

    let channels = if let Some(guild_id) = selected_guild_id {
        let mut chs: Vec<ChannelView> = cache
            .channels()
            .filter(|c| c.guild_id == Some(guild_id))
            .map(|c: &Channel| {
                let unread = read_state.is_unread(c.id, c.last_message_id, None);
                ChannelView {
                    id: c.id,
                    name: c
                        .name
                        .clone()
                        .unwrap_or_else(|| format!("channel-{}", c.id)),
                    kind: c.kind.0,
                    icon_url: None,
                    parent_id: c.parent_id,
                    position: c.position.unwrap_or(u16::MAX),
                    unread,
                    mention_count: read_state.mention_count(c.id),
                }
            })
            .collect();
        chs.sort_by(|a, b| {
            a.position
                .cmp(&b.position)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                .then_with(|| a.id.cmp(&b.id))
        });
        chs
    } else {
        cache
            .private_channels()
            .into_iter()
            .map(|c: &Channel| {
                let dm_name = c
                    .name
                    .clone()
                    .filter(|s| !s.is_empty())
                    .or_else(|| {
                        if c.kind.0 == 1 {
                            c.recipients.first().map(|u| {
                                u.global_name
                                    .clone()
                                    .filter(|s| !s.is_empty())
                                    .unwrap_or_else(|| u.username.clone())
                            })
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| {
                        if c.kind.0 == 3 {
                            let names: Vec<String> = c
                                .recipients
                                .iter()
                                .take(3)
                                .map(|u| {
                                    u.global_name
                                        .clone()
                                        .filter(|s| !s.is_empty())
                                        .unwrap_or_else(|| u.username.clone())
                                })
                                .collect();
                            if names.is_empty() {
                                format!("Group DM {}", c.id)
                            } else {
                                names.join(", ")
                            }
                        } else {
                            format!("DM {}", c.id)
                        }
                    });
                let icon_url = if c.kind.0 == 1 {
                    c.recipients
                        .first()
                        .map(|u| cdn::user_avatar_url(u.id, u.avatar.as_deref(), 64))
                } else {
                    c.recipients
                        .first()
                        .map(|u| cdn::user_avatar_url(u.id, u.avatar.as_deref(), 64))
                };
                let unread = read_state.is_unread(c.id, c.last_message_id, None);
                ChannelView {
                    id: c.id,
                    name: dm_name,
                    kind: c.kind.0,
                    icon_url,
                    parent_id: c.parent_id,
                    position: c.position.unwrap_or(u16::MAX),
                    unread,
                    mention_count: read_state.mention_count(c.id),
                }
            })
            .collect::<Vec<_>>()
    };

    let selected_channel_last_read_id =
        selected_channel_id.and_then(|cid| read_state.last_read_message_id(u64::from(cid)));

    let messages = if let Some(channel_id) = selected_channel_id {
        let mut msgs = fetched_messages
            .get(&channel_id)
            .cloned()
            .unwrap_or_default();
        let cached = cache.messages(channel_id);
        for msg in cached {
            if !msgs.iter().any(|m| m.id == msg.id) {
                msgs.push(msg.clone());
            }
        }
        if msgs.is_empty() {
            msgs = cached.to_vec();
        }
        msgs.sort_by(|a, b| a.id.cmp(&b.id));
        if msgs.len() > 100 {
            let start = msgs.len() - 100;
            msgs.drain(0..start);
        }
        msgs.into_iter()
            .map(|m| {
                let reply_preview = m
                    .referenced_message
                    .as_deref()
                    .map(reply_preview_from_message);
                let attachments = m
                    .attachments
                    .iter()
                    .map(|a| AttachmentView {
                        filename: a.filename.clone(),
                        url: a.proxy_url.clone().unwrap_or_else(|| a.url.clone()),
                        content_type: a.content_type.clone(),
                        width: a.width,
                        height: a.height,
                        size: a.size,
                    })
                    .collect::<Vec<_>>();
                let embeds = m.embeds.iter().map(extract_embed_view).collect::<Vec<_>>();
                let sticker_names = m
                    .sticker_items
                    .iter()
                    .filter_map(|s| s.get("name").and_then(|n| n.as_str()).map(|n| n.to_owned()))
                    .collect::<Vec<_>>();

                MessageView {
                    id: m.id,
                    author_id: m.author.id,
                    author: m
                        .author
                        .global_name
                        .clone()
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| m.author.username.clone()),
                    author_avatar_url: Some(cdn::user_avatar_url(
                        m.author.id,
                        m.author.avatar.as_deref(),
                        80,
                    )),
                    content: m.content.clone(),
                    timestamp: m.timestamp.clone(),
                    edited: m.edited_timestamp.is_some(),
                    attachment_count: m.attachments.len(),
                    embed_count: m.embeds.len(),
                    reply: matches!(m.kind, lampcord_model::message::MessageType::REPLY)
                        || m.message_reference.is_some(),
                    reply_preview,
                    attachments,
                    embeds,
                    sticker_names,
                    reaction_count: m.reactions.len(),
                }
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    Snapshot {
        status: status.into(),
        current_user,
        current_user_id,
        current_user_avatar_url,
        guilds,
        guild_folders: guild_folders.to_vec(),
        channels,
        messages,
        selected_guild_id,
        selected_channel_id,
        selected_channel_last_read_id,
        saved_accounts,
        error,
        remote_auth,
        login_flow,
        forgot_password,
        device_sessions,
    }
}

fn remote_auth_view_from_state(state: &RemoteAuthState) -> RemoteAuthView {
    match state {
        RemoteAuthState::Idle => RemoteAuthView {
            status: "Idle".to_owned(),
            ..RemoteAuthView::default()
        },
        RemoteAuthState::Connecting => RemoteAuthView {
            status: "Connecting to Discord...".to_owned(),
            ..RemoteAuthView::default()
        },
        RemoteAuthState::QrReady { url } => RemoteAuthView {
            status: "Scan this QR code with the Discord mobile app".to_owned(),
            qr_url: Some(url.clone()),
            ..RemoteAuthView::default()
        },
        RemoteAuthState::UserScanned { user } => {
            let name = user
                .global_name
                .clone()
                .unwrap_or_else(|| user.username.clone());
            RemoteAuthView {
                status: format!("Confirm login as {name} on your phone"),
                scanned_user: Some(name),
                ..RemoteAuthView::default()
            }
        }
        RemoteAuthState::Finished { .. } => RemoteAuthView {
            status: "Login approved, connecting...".to_owned(),
            ..RemoteAuthView::default()
        },
        RemoteAuthState::Error { message } => RemoteAuthView {
            status: "QR login failed".to_owned(),
            error: Some(message.clone()),
            ..RemoteAuthView::default()
        },
    }
}

fn device_session_challenge_from_failure(
    pending_id_hashes: &[String],
    failure: &lampcord_core::api::sessions::SessionFailure,
    password: Option<String>,
) -> Option<DeviceSessionChallenge> {
    if let Some(mfa) = failure.mfa() {
        let ticket = mfa
            .get("ticket")
            .and_then(|v| v.as_str())
            .map(str::to_owned);
        if let Some(ticket) = ticket {
            let mut methods: Vec<String> = Vec::new();
            if let Some(arr) = mfa.get("methods").and_then(|v| v.as_array()) {
                for m in arr {
                    if let Some(s) = m.as_str() {
                        methods.push(s.to_owned());
                    } else if let Some(obj) = m.as_object() {
                        if let Some(t) = obj.get("type").and_then(|v| v.as_str()) {
                            methods.push(t.to_owned());
                        }
                    }
                }
            }
            let has_totp = methods.is_empty() || methods.iter().any(|m| m == "totp");
            let has_backup = methods.iter().any(|m| m == "backup");
            if has_totp {
                return Some(DeviceSessionChallenge::Totp {
                    pending_id_hashes: pending_id_hashes.to_vec(),
                    ticket,
                    mfa_type: "totp".to_owned(),
                    allow_backup: has_backup,
                    password,
                    message: failure.message().map(str::to_owned),
                });
            } else if has_backup {
                return Some(DeviceSessionChallenge::BackupCode {
                    pending_id_hashes: pending_id_hashes.to_vec(),
                    password,
                    message: failure.message().map(str::to_owned),
                });
            }
        }
    }

    if failure.requires_password() && password.is_none() {
        return Some(DeviceSessionChallenge::Password {
            pending_id_hashes: pending_id_hashes.to_vec(),
            message: failure.message().map(str::to_owned),
        });
    }

    if failure.requires_code() {
        return Some(DeviceSessionChallenge::BackupCode {
            pending_id_hashes: pending_id_hashes.to_vec(),
            password,
            message: failure.message().map(str::to_owned),
        });
    }

    None
}

async fn run(
    mut cmd_rx: mpsc::UnboundedReceiver<BackendCommand>,
    evt_tx: mpsc::UnboundedSender<BackendEvent>,
    snapshot: Arc<Mutex<Snapshot>>,
    token_store_path: PathBuf,
) {
    let mut token_store = load_token_store(&token_store_path);
    let mut selected_guild_id: Option<Snowflake> = None;
    let mut selected_channel_id: Option<Snowflake> = None;
    let mut fetched_messages: HashMap<Snowflake, Vec<Message>> = HashMap::new();
    let mut fetched_message_order: Vec<Snowflake> = Vec::new();
    const MAX_FETCHED_MESSAGE_CHANNELS: usize = 6;
    const MAX_FETCHED_MESSAGES_PER_CHANNEL: usize = 80;

    macro_rules! clear_fetched_messages {
        () => {{
            fetched_messages.clear();
            fetched_message_order.clear();
        }};
    }
    macro_rules! touch_fetched_channel {
        ($id:expr) => {{
            let id = $id;
            if let Some(pos) = fetched_message_order.iter().position(|x| *x == id) {
                fetched_message_order.remove(pos);
            }
            fetched_message_order.push(id);
            while fetched_message_order.len() > MAX_FETCHED_MESSAGE_CHANNELS {
                let oldest = fetched_message_order.remove(0);
                fetched_messages.remove(&oldest);
            }
        }};
    }
    macro_rules! cap_fetched_channel {
        ($id:expr) => {{
            if let Some(msgs) = fetched_messages.get_mut(&$id) {
                if msgs.len() > MAX_FETCHED_MESSAGES_PER_CHANNEL {
                    let drain = msgs.len() - MAX_FETCHED_MESSAGES_PER_CHANNEL;
                    msgs.drain(0..drain);
                }
            }
        }};
    }
    let mut active_token: Option<String> = None;
    let mut cache: Option<Arc<Mutex<StateCache>>> = None;
    let mut rest: Option<Arc<RestClient<ReqwestRestTransport, NoopCaptchaSolver>>> = None;
    let mut gateway_task: Option<tokio::task::JoinHandle<()>> = None;
    let mut gateway_events: Option<mpsc::UnboundedReceiver<GatewayEvent>> = None;
    let mut status = "Idle".to_owned();
    let mut error: Option<String> = None;
    let mut _should_reconnect = false;
    let mut _reconnect_attempt: u32 = 0;
    let mut remote_auth_view = RemoteAuthView::default();
    let mut remote_auth_cancel: Option<tokio::sync::oneshot::Sender<()>> = None;
    let mut remote_auth_task: Option<tokio::task::JoinHandle<()>> = None;
    let mut remote_auth_rx: Option<tokio::sync::watch::Receiver<RemoteAuthState>> = None;
    let mut login_flow = LoginFlowView::default();
    let mut forgot_password = ForgotPasswordView::default();
    let mut device_sessions = DeviceSessionsView::default();
    let mut guild_folders: Vec<GuildFolderView> = Vec::new();
    let mut login_fingerprint: Option<String> = None;
    let mut auth_transport: Option<ReqwestRestTransport> = None;
    let identity = ClientIdentity::load(&token_store_path);

    let push = |status: &str,
                cache: &Option<Arc<Mutex<StateCache>>>,
                selected_guild_id: Option<Snowflake>,
                selected_channel_id: Option<Snowflake>,
                fetched_messages: &HashMap<Snowflake, Vec<Message>>,
                token_store: &TokenStore,
                error: Option<String>,
                remote_auth: &RemoteAuthView,
                login_flow: &LoginFlowView,
                forgot_password: &ForgotPasswordView,
                device_sessions: &DeviceSessionsView,
                guild_folders: &[GuildFolderView],
                evt_tx: &mpsc::UnboundedSender<BackendEvent>,
                snapshot: &Arc<Mutex<Snapshot>>| {
        let snap = if let Some(cache) = cache {
            render_snapshot(
                status.to_owned(),
                cache,
                selected_guild_id,
                selected_channel_id,
                fetched_messages,
                token_store.accounts().to_vec(),
                guild_folders,
                error,
                remote_auth.clone(),
                login_flow.clone(),
                forgot_password.clone(),
                device_sessions.clone(),
            )
        } else {
            Snapshot {
                status: status.to_owned(),
                selected_guild_id,
                selected_channel_id,
                saved_accounts: token_store.accounts().to_vec(),
                guild_folders: guild_folders.to_vec(),
                error,
                remote_auth: remote_auth.clone(),
                login_flow: login_flow.clone(),
                forgot_password: forgot_password.clone(),
                device_sessions: device_sessions.clone(),
                ..Snapshot::default()
            }
        };
        if let Ok(mut current) = snapshot.lock() {
            *current = snap.clone();
        }
        crate::ui::debug_stats::record_backend_push();
        let _ = evt_tx.send(BackendEvent::Snapshot(snap));
    };

    push(
        &status,
        &cache,
        selected_guild_id,
        selected_channel_id,
        &fetched_messages,
        &token_store,
        error.clone(),
        &remote_auth_view,
        &login_flow,
        &forgot_password,
        &device_sessions,
        &guild_folders,
        &evt_tx,
        &snapshot,
    );

    loop {
        tokio::select! {
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    BackendCommand::Connect { token } => {
                        login_flow = LoginFlowView::default();
                        if let Some(task) = gateway_task.take() { task.abort(); }
                        gateway_events = None;
                        status = "Connecting...".to_owned();
                        error = None;
                        active_token = Some(token.clone());
                        _should_reconnect = true;
                        _reconnect_attempt = 0;
                        clear_fetched_messages!();
                        guild_folders.clear();

                        let transport = match ReqwestRestTransport::new() {
                            Ok(t) => t,
                            Err(e) => {
                                status = "Error".to_owned();
                                error = Some(format!("REST transport init failed: {e}"));
                                _should_reconnect = false;
                                push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                                continue;
                            }
                        };
                        let rest_client = Arc::new(RestClient::new(rest_config(Some(token.clone()), &identity), transport, NoopCaptchaSolver));
                        let gw_transport = TungsteniteGatewayTransport::default();
                        let url_provider = StaticGatewayUrl { rest: rest_client.clone() };
                        let gw_identity = lampcord_core::gateway_client::GatewayIdentity::desktop(Some(identity.device_vendor_id.clone()));
                        let mut client = GatewayClient::new(gw_transport, url_provider).with_identity(gw_identity);
                        let client_cache = client.cache().clone();
                        cache = Some(client_cache.clone());
                        rest = Some(rest_client.clone());
                        let (gw_tx, gw_rx) = mpsc::unbounded_channel();
                        gateway_events = Some(gw_rx);
                        let token_clone = token.clone();
                        gateway_task = Some(tokio::spawn(async move {
                            let mut attempt: u32 = 0;
                            loop {
                                let result = client.run_stream(&token_clone, gw_tx.clone()).await;
                                let terminal = match result {
                                    Ok(t) => t,
                                    Err(e) => {
                                        eprintln!("[lampcord] gateway error: {e}");
                                        None
                                    }
                                };
                                let reconnect = match terminal {
                                    Some(GatewayEvent::AuthFailed { .. }) => false,
                                    Some(GatewayEvent::Closed { code }) if matches!(code, Some(4004) | Some(4010) | Some(4011) | Some(4012) | Some(4013) | Some(4014)) => false,
                                    Some(GatewayEvent::ReconnectRequested)
                                    | Some(GatewayEvent::InvalidSession { .. })
                                    | Some(GatewayEvent::Closed { .. })
                                    | None => true,
                                    _ => true,
                                };
                                if !reconnect {
                                    break;
                                }
                                let delay_ms = reconnect_delay_ms(attempt);
                                attempt = attempt.saturating_add(1);
                                eprintln!("[lampcord] reconnecting in {delay_ms}ms (attempt {attempt})");
                                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                            }
                        }));
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::SelectGuild { guild_id } => {
                        selected_guild_id = guild_id;
                        selected_channel_id = None;
                        clear_fetched_messages!();

                        if let Some(rest_client) = rest.clone() {
                            let config = rest_config(active_token.clone(), &identity);
                            if let Some(gid) = guild_id {
                                match lampcord_core::api::channel::get_guild_channels(
                                    &*rest_client,
                                    &config,
                                    u64::from(gid),
                                )
                                .await
                                {
                                    Ok(channels) => {
                                        if let Some(cache_arc) = cache.clone() {
                                            if let Ok(mut c) = cache_arc.lock() {
                                                for channel in channels {
                                                    if let Ok(value) = serde_json::to_value(&channel)
                                                    {
                                                        let _ = c.handle_event(
                                                            "CHANNEL_CREATE",
                                                            Some(&value),
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error = Some(format!("Fetch channels failed: {e}"));
                                    }
                                }
                            } else {
                                match lampcord_core::api::channel::get_private_channels(
                                    &*rest_client,
                                    &config,
                                )
                                .await
                                {
                                    Ok(channels) => {
                                        if let Some(cache_arc) = cache.clone() {
                                            if let Ok(mut c) = cache_arc.lock() {
                                                for channel in channels {
                                                    if let Ok(value) = serde_json::to_value(&channel)
                                                    {
                                                        let _ = c.handle_event(
                                                            "CHANNEL_CREATE",
                                                            Some(&value),
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        if cache
                                            .as_ref()
                                            .and_then(|ca| ca.lock().ok())
                                            .map(|c| c.private_channels().is_empty())
                                            .unwrap_or(false)
                                        {
                                            error = Some(format!("Fetch DMs failed: {e}"));
                                        }
                                    }
                                }
                            }
                        }

                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::SelectChannel { channel_id } => {
                        selected_channel_id = channel_id;
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                        if let Some(cid) = channel_id {
                            if let Some(rest_client) = rest.clone() {
                                let config = rest_config(active_token.clone(), &identity);
                                match lampcord_core::api::fetch_channel_messages(&*rest_client, &config, cid, 50, None).await {
                                    Ok(mut msgs) => {
                                        msgs.sort_by(|a, b| a.id.cmp(&b.id));
                                        if let Some(latest) = msgs.last().map(|m| m.id) {
                                            let _ = lampcord_core::api::ack_channel_message(&*rest_client, &config, cid, latest).await;
                                            if let Some(cache_arc) = cache.clone() {
                                                if let Ok(mut c) = cache_arc.lock() {
                                                    c.read_state_mut().ack(cid, latest);
                                                }
                                            }
                                        }
                                        fetched_messages.insert(cid, msgs);
                                        touch_fetched_channel!(cid);
                                        cap_fetched_channel!(cid);
                                        error = None;
                                    }
                                    Err(e) => {
                                        error = Some(format!("Fetch messages failed: {e}"));
                                    }
                                }
                            }
                        }
                    }
                    BackendCommand::FetchMessages { channel_id, before } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = rest_config(active_token.clone(), &identity);
                            match lampcord_core::api::fetch_channel_messages(&*rest_client, &config, channel_id, 50, before).await {
                                Ok(mut msgs) => {
                                    msgs.sort_by(|a, b| a.id.cmp(&b.id));
                                    let entry = fetched_messages.entry(channel_id).or_default();
                                    if before.is_some() {
                                        for msg in msgs.into_iter().rev() {
                                            if !entry.iter().any(|m| m.id == msg.id) {
                                                entry.insert(0, msg);
                                            }
                                        }
                                    } else {
                                        for msg in msgs {
                                            if !entry.iter().any(|m| m.id == msg.id) {
                                                entry.push(msg);
                                            }
                                        }
                                    }
                                    entry.sort_by(|a, b| a.id.cmp(&b.id));
                                    if entry.len() > MAX_FETCHED_MESSAGES_PER_CHANNEL {
                                        let drain = entry.len() - MAX_FETCHED_MESSAGES_PER_CHANNEL;
                                        entry.drain(0..drain);
                                    }
                                    touch_fetched_channel!(channel_id);
                                    error = None;
                                }
                                Err(e) => {
                                    error = Some(format!("Fetch messages failed: {e}"));
                                }
                            }
                        }
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::SendMessage {
                        channel_id,
                        content,
                        reply_to_message_id,
                        reply_to_guild_id,
                    } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = rest_config(active_token.clone(), &identity);
                            match lampcord_core::api::send_channel_message_with_reply(
                                &*rest_client,
                                &config,
                                channel_id,
                                &content,
                                reply_to_message_id,
                                reply_to_guild_id,
                            )
                            .await
                            {
                                Ok(msg) => {
                                    let entry = fetched_messages.entry(channel_id).or_default();
                                    if !entry.iter().any(|m| m.id == msg.id) {
                                        entry.push(msg);
                                    }
                                    entry.sort_by(|a, b| a.id.cmp(&b.id));
                                    if entry.len() > MAX_FETCHED_MESSAGES_PER_CHANNEL {
                                        let drain = entry.len() - MAX_FETCHED_MESSAGES_PER_CHANNEL;
                                        entry.drain(0..drain);
                                    }
                                    touch_fetched_channel!(channel_id);
                                    error = None;
                                }
                                Err(e) => {
                                    error = Some(format!("Send failed: {e}"));
                                }
                            }
                        }
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::EditMessage {
                        channel_id,
                        message_id,
                        content,
                    } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = rest_config(active_token.clone(), &identity);
                            match lampcord_core::api::edit_channel_message(
                                &*rest_client,
                                &config,
                                channel_id,
                                message_id,
                                &content,
                            )
                            .await
                            {
                                Ok(msg) => {
                                    let entry = fetched_messages.entry(channel_id).or_default();
                                    if let Some(existing) = entry.iter_mut().find(|m| m.id == msg.id) {
                                        *existing = msg;
                                    } else {
                                        entry.push(msg);
                                    }
                                    entry.sort_by(|a, b| a.id.cmp(&b.id));
                                    if entry.len() > MAX_FETCHED_MESSAGES_PER_CHANNEL {
                                        let drain = entry.len() - MAX_FETCHED_MESSAGES_PER_CHANNEL;
                                        entry.drain(0..drain);
                                    }
                                    touch_fetched_channel!(channel_id);
                                    error = None;
                                }
                                Err(e) => {
                                    error = Some(format!("Edit failed: {e}"));
                                }
                            }
                        }
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::PersistToken { token, user_id, username } => {
                        token_store.add(SavedAccount { token, user_id, username });
                        save_token_store(&token_store_path, &token_store);
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }

                    BackendCommand::LoginPassword { login, password } => {
                        login_flow = LoginFlowView { stage: "busy".to_owned(), ..LoginFlowView::default() };
                        error = None;
                        login_fingerprint = None;
                        auth_transport = match ReqwestRestTransport::new_with_cookie_store() {
                            Ok(t) => Some(t),
                            Err(e) => {
                                login_flow = LoginFlowView { error: Some(format!("REST transport init failed: {e}")), ..LoginFlowView::default() };
                                push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                                continue;
                            }
                        };
                        let transport = auth_transport.clone().expect("auth transport");
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);

                        let client = RestClient::new(rest_config(None, &identity), transport, NoopCaptchaSolver);
                        let config = rest_config(None, &identity);

                        if login_fingerprint.is_none() {
                            login_fingerprint = lampcord_core::api::auth::get_fingerprint(&client, &config).await.ok().flatten();
                        }

                        let req = lampcord_core::auth::LoginRequest::new(login, password);
                        let result = lampcord_core::api::auth::login(&client, &config, &req, login_fingerprint.as_deref()).await;
                        match result {
                            Ok(resp) => {
                                if let Some(token) = resp.token.clone() {
                                    auth_transport = None;
                                    login_fingerprint = None;
                                    login_flow = LoginFlowView::default();
                                    if let Some(task) = gateway_task.take() { task.abort(); }
                                    gateway_events = None;
                                    status = "Connecting...".to_owned();
                                    error = None;
                                    active_token = Some(token.clone());
                                    _should_reconnect = true;
                                    _reconnect_attempt = 0;
                                    clear_fetched_messages!();
                                    let gw_rest_transport = match ReqwestRestTransport::new() {
                                        Ok(t) => t,
                                        Err(e) => {
                                            status = "Error".to_owned();
                                            error = Some(format!("REST transport init failed: {e}"));
                                            _should_reconnect = false;
                                            push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                                            continue;
                                        }
                                    };
                                    let rest_client = Arc::new(RestClient::new(rest_config(Some(token.clone()), &identity), gw_rest_transport, NoopCaptchaSolver));
                                    let gw_transport = TungsteniteGatewayTransport::default();
                                    let url_provider = StaticGatewayUrl { rest: rest_client.clone() };
                                    let gw_identity = lampcord_core::gateway_client::GatewayIdentity::desktop(Some(identity.device_vendor_id.clone()));
                                    let mut gw_client = GatewayClient::new(gw_transport, url_provider).with_identity(gw_identity);
                                    let client_cache = gw_client.cache().clone();
                                    cache = Some(client_cache.clone());
                                    rest = Some(rest_client.clone());
                                    let (gw_tx, gw_rx) = mpsc::unbounded_channel();
                                    gateway_events = Some(gw_rx);
                                    let token_clone = token.clone();
                                    gateway_task = Some(tokio::spawn(async move {
                                        let mut attempt: u32 = 0;
                                        loop {
                                            let result = gw_client.run_stream(&token_clone, gw_tx.clone()).await;
                                            let terminal = match result {
                                                Ok(t) => t,
                                                Err(e) => { eprintln!("[lampcord] gateway error: {e}"); None }
                                            };
                                            let reconnect = match terminal {
                                                Some(GatewayEvent::AuthFailed { .. }) => false,
                                                Some(GatewayEvent::Closed { code }) if matches!(code, Some(4004) | Some(4010) | Some(4011) | Some(4012) | Some(4013) | Some(4014)) => false,
                                                Some(GatewayEvent::ReconnectRequested)
                                                | Some(GatewayEvent::InvalidSession { .. })
                                                | Some(GatewayEvent::Closed { .. })
                                                | None => true,
                                                _ => true,
                                            };
                                            if !reconnect { break; }
                                            let delay_ms = reconnect_delay_ms(attempt);
                                            attempt = attempt.saturating_add(1);
                                            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                                        }
                                    }));
                                } else if resp.ticket.is_some() && (resp.mfa.is_some() || resp.totp == Some(true) || resp.sms == Some(true) || resp.backup == Some(true)) {
                                    let mut methods = Vec::new();
                                    if resp.totp == Some(true) { methods.push("totp".to_owned()); }
                                    if resp.sms == Some(true) { methods.push("sms".to_owned()); }
                                    if resp.backup == Some(true) { methods.push("backup".to_owned()); }
                                    if methods.is_empty() { methods.push("totp".to_owned()); }
                                    let selected = methods.first().cloned();
                                    login_flow = LoginFlowView {
                                        stage: "mfa".to_owned(),
                                        error: None,
                                        ticket: resp.ticket.clone(),
                                        login_instance_id: resp.login_instance_id.clone(),
                                        mfa_methods: methods,
                                        selected_mfa_method: selected,
                                        sms_phone: None,
                                    };
                                } else {
                                    auth_transport = None;
                                    let mut msg = resp
                                        .message
                                        .clone()
                                        .or_else(|| {
                                            resp.failure
                                                .map(|f| lampcord_core::auth::failure_message(f).to_owned())
                                        })
                                        .unwrap_or_else(|| "Login failed.".to_owned());
                                    if matches!(
                                        resp.failure,
                                        Some(lampcord_core::auth::LoginFailure::CaptchaRejected)
                                            | Some(lampcord_core::auth::LoginFailure::CaptchaCancelled)
                                    ) {
                                        msg.push_str(" Captcha solving is not yet available in the Rust rewrite UI.");
                                    }
                                    if matches!(
                                        resp.failure,
                                        Some(lampcord_core::auth::LoginFailure::VerificationRequired)
                                    ) {
                                        msg.push_str(" Complete verification (email/phone/IP authorization) and try again.");
                                    }
                                    if !resp.required_actions.is_empty() {
                                        let actions = resp
                                            .required_actions
                                            .iter()
                                            .map(|v| match v {
                                                serde_json::Value::String(s) => s.clone(),
                                                other => other.to_string(),
                                            })
                                            .collect::<Vec<_>>()
                                            .join(", ");
                                        msg.push_str(&format!(" Required actions: {actions}."));
                                    }
                                    login_flow = LoginFlowView {
                                        error: Some(msg),
                                        ..LoginFlowView::default()
                                    };
                                }
                            }
                            Err(e) => {
                                auth_transport = None;
                                login_flow = LoginFlowView { error: Some(format!("Login request failed: {e}")), ..LoginFlowView::default() };
                            }
                        }
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::SubmitMfa { code } => {
                        let code: String = code
                            .chars()
                            .filter(|c| !c.is_whitespace() && *c != '-')
                            .collect();
                        let (ticket, instance_id, method) = (
                            login_flow.ticket.clone(),
                            login_flow.login_instance_id.clone(),
                            login_flow.selected_mfa_method.clone().unwrap_or_else(|| "totp".to_owned()),
                        );
                        let Some(ticket) = ticket else {
                            login_flow.error = Some("Missing MFA ticket. Please try logging in again.".to_owned());
                            push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                            continue;
                        };
                        let Some(transport) = auth_transport.clone() else {
                            login_flow.stage = "mfa".to_owned();
                            login_flow.error = Some("Login session expired. Please start over.".to_owned());
                            push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                            continue;
                        };
                        login_flow.stage = "busy".to_owned();
                        login_flow.error = None;
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);

                        let auth_client = RestClient::new(rest_config(None, &identity), transport, NoopCaptchaSolver);
                        let config = rest_config(None, &identity);
                        let mut req = lampcord_core::auth::MfaLoginRequest::new(code, ticket);
                        req.login_instance_id = instance_id;
                        let result = lampcord_core::api::auth::login_mfa(&auth_client, &config, &req, login_fingerprint.as_deref(), &method).await;
                        match result {
                            Ok(resp) => {
                                if let Some(token) = resp.token.clone() {
                                    auth_transport = None;
                                    login_fingerprint = None;
                                    login_flow = LoginFlowView::default();
                                    if let Some(task) = gateway_task.take() { task.abort(); }
                                    gateway_events = None;
                                    status = "Connecting...".to_owned();
                                    error = None;
                                    active_token = Some(token.clone());
                                    _should_reconnect = true;
                                    _reconnect_attempt = 0;
                                    clear_fetched_messages!();
                                    let gw_rest_transport = match ReqwestRestTransport::new() {
                                        Ok(t) => t,
                                        Err(e) => {
                                            status = "Error".to_owned();
                                            error = Some(format!("REST transport init failed: {e}"));
                                            _should_reconnect = false;
                                            push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                                            continue;
                                        }
                                    };
                                    let rest_client = Arc::new(RestClient::new(rest_config(Some(token.clone()), &identity), gw_rest_transport, NoopCaptchaSolver));
                                    let gw_transport = TungsteniteGatewayTransport::default();
                                    let url_provider = StaticGatewayUrl { rest: rest_client.clone() };
                                    let gw_identity = lampcord_core::gateway_client::GatewayIdentity::desktop(Some(identity.device_vendor_id.clone()));
                                    let mut gw_client = GatewayClient::new(gw_transport, url_provider).with_identity(gw_identity);
                                    let client_cache = gw_client.cache().clone();
                                    cache = Some(client_cache.clone());
                                    rest = Some(rest_client.clone());
                                    let (gw_tx, gw_rx) = mpsc::unbounded_channel();
                                    gateway_events = Some(gw_rx);
                                    let token_clone = token.clone();
                                    gateway_task = Some(tokio::spawn(async move {
                                        let mut attempt: u32 = 0;
                                        loop {
                                            let result = gw_client.run_stream(&token_clone, gw_tx.clone()).await;
                                            let terminal = match result {
                                                Ok(t) => t,
                                                Err(e) => { eprintln!("[lampcord] gateway error: {e}"); None }
                                            };
                                            let reconnect = match terminal {
                                                Some(GatewayEvent::AuthFailed { .. }) => false,
                                                Some(GatewayEvent::Closed { code }) if matches!(code, Some(4004) | Some(4010) | Some(4011) | Some(4012) | Some(4013) | Some(4014)) => false,
                                                Some(GatewayEvent::ReconnectRequested)
                                                | Some(GatewayEvent::InvalidSession { .. })
                                                | Some(GatewayEvent::Closed { .. })
                                                | None => true,
                                                _ => true,
                                            };
                                            if !reconnect { break; }
                                            let delay_ms = reconnect_delay_ms(attempt);
                                            attempt = attempt.saturating_add(1);
                                            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                                        }
                                    }));
                                } else {
                                    login_flow.stage = "mfa".to_owned();
                                    login_flow.error = resp.message.clone().or_else(|| resp.failure.map(|f| lampcord_core::auth::failure_message(f).to_owned())).or_else(|| Some("Invalid or expired code.".to_owned()));
                                }
                            }
                            Err(e) => {
                                login_flow.stage = "mfa".to_owned();
                                login_flow.error = Some(format!("MFA request failed: {e}"));
                            }
                        }
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::SetMfaMethod { method } => {
                        login_flow.selected_mfa_method = Some(method);
                        login_flow.error = None;
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::ResendMfaSms => {
                        let Some(ticket) = login_flow.ticket.clone() else {
                            login_flow.error = Some("Missing MFA ticket.".to_owned());
                            push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                            continue;
                        };
                        let Some(transport) = auth_transport.clone() else {
                            login_flow.stage = "mfa".to_owned();
                            login_flow.error = Some("Login session expired. Please start over.".to_owned());
                            push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                            continue;
                        };
                        login_flow.stage = "busy".to_owned();
                        login_flow.error = None;
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                        let client = RestClient::new(rest_config(None, &identity), transport, NoopCaptchaSolver);
                        let config = rest_config(None, &identity);
                        match lampcord_core::api::auth::send_mfa_sms(&client, &config, &ticket, login_fingerprint.as_deref()).await {
                            Ok(phone) => {
                                login_flow.stage = "mfa".to_owned();
                                login_flow.sms_phone = Some(phone);
                            }
                            Err(e) => {
                                login_flow.stage = "mfa".to_owned();
                                login_flow.error = Some(format!("Failed to send SMS code: {e}"));
                            }
                        }
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::CancelLoginFlow => {
                        login_flow = LoginFlowView::default();
                        auth_transport = None;
                        login_fingerprint = None;
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::ForgotPassword { login } => {
                        forgot_password = ForgotPasswordView {
                            stage: "busy".to_owned(),
                            message: None,
                        };
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);

                        let transport = match ReqwestRestTransport::new() {
                            Ok(t) => t,
                            Err(e) => {
                                forgot_password = ForgotPasswordView {
                                    stage: "failed".to_owned(),
                                    message: Some(format!("REST transport init failed: {e}")),
                                };
                                push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                                continue;
                            }
                        };
                        let client = RestClient::new(rest_config(None, &identity), transport, NoopCaptchaSolver);
                        let config = rest_config(None, &identity);
                        if login_fingerprint.is_none() {
                            login_fingerprint = lampcord_core::api::auth::get_fingerprint(&client, &config).await.ok().flatten();
                        }
                        match lampcord_core::api::auth::forgot_password(&client, &config, &login, login_fingerprint.as_deref()).await {
                            Ok(lampcord_core::api::auth::ForgotPasswordResult::Sent) => {
                                forgot_password = ForgotPasswordView {
                                    stage: "sent".to_owned(),
                                    message: Some("If an account exists for that email or phone number, we sent password reset instructions.".to_owned()),
                                };
                            }
                            Ok(lampcord_core::api::auth::ForgotPasswordResult::PhoneVerificationRequired) => {
                                forgot_password = ForgotPasswordView {
                                    stage: "phone_required".to_owned(),
                                    message: Some("Phone verification is required to reset this password. Please complete verification on discord.com or try using your email.".to_owned()),
                                };
                            }
                            Ok(lampcord_core::api::auth::ForgotPasswordResult::Failed(msg)) => {
                                forgot_password = ForgotPasswordView {
                                    stage: "failed".to_owned(),
                                    message: Some(msg),
                                };
                            }
                            Err(e) => {
                                forgot_password = ForgotPasswordView {
                                    stage: "failed".to_owned(),
                                    message: Some(format!("Password reset request failed: {e}")),
                                };
                            }
                        }
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::ClearForgotPassword => {
                        forgot_password = ForgotPasswordView::default();
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::LoadDeviceSessions => {
                        if let Some(rest_client) = rest.clone() {
                            device_sessions.loading = true;
                            device_sessions.error = None;
                            device_sessions.status = None;
                            push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                            let config = rest_config(active_token.clone(), &identity);
                            match lampcord_core::api::sessions::list_sessions(&*rest_client, &config).await {
                                Ok(sessions) => {
                                    device_sessions.sessions = sessions
                                        .into_iter()
                                        .map(|s| DeviceSessionView {
                                            id_hash: s.id_hash,
                                            name: s.name,
                                            location: s.location,
                                            last_used: s.last_used,
                                            current: s.current,
                                            icon: match s.icon {
                                                lampcord_core::api::sessions::DeviceIcon::Mobile => "mobile".to_owned(),
                                                lampcord_core::api::sessions::DeviceIcon::Desktop => "desktop".to_owned(),
                                                lampcord_core::api::sessions::DeviceIcon::Unknown => "unknown".to_owned(),
                                            },
                                        })
                                        .collect();
                                    device_sessions.loading = false;
                                    device_sessions.error = None;
                                    device_sessions.status = Some(format!(
                                        "Loaded {} session(s)",
                                        device_sessions.sessions.len()
                                    ));
                                }
                                Err(e) => {
                                    device_sessions.loading = false;
                                    device_sessions.error = Some(format!("Failed to load sessions: {e}"));
                                }
                            }
                        } else {
                            device_sessions.loading = false;
                            device_sessions.error = Some("Not connected. Log in first to view device sessions.".to_owned());
                        }
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::LogoutDeviceSessions { id_hashes } => {
                        if id_hashes.is_empty() {
                            device_sessions.error = Some("No sessions selected.".to_owned());
                            push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                            continue;
                        }
                        if let Some(rest_client) = rest.clone() {
                            device_sessions.challenge = None;
                            device_sessions.loading = true;
                            device_sessions.error = None;
                            device_sessions.status = Some("Logging out selected session(s)...".to_owned());
                            push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                            let config = rest_config(active_token.clone(), &identity);
                            match lampcord_core::api::sessions::logout_sessions(&*rest_client, &config, &id_hashes, None, None, None).await {
                                Ok(()) => {
                                    device_sessions.status = Some("Session(s) logged out. Refreshing...".to_owned());
                                    match lampcord_core::api::sessions::list_sessions(&*rest_client, &config).await {
                                        Ok(sessions) => {
                                            device_sessions.sessions = sessions
                                                .into_iter()
                                                .map(|s| DeviceSessionView {
                                                    id_hash: s.id_hash,
                                                    name: s.name,
                                                    location: s.location,
                                                    last_used: s.last_used,
                                                    current: s.current,
                                                    icon: match s.icon {
                                                        lampcord_core::api::sessions::DeviceIcon::Mobile => "mobile".to_owned(),
                                                        lampcord_core::api::sessions::DeviceIcon::Desktop => "desktop".to_owned(),
                                                        lampcord_core::api::sessions::DeviceIcon::Unknown => "unknown".to_owned(),
                                                    },
                                                })
                                                .collect();
                                            device_sessions.loading = false;
                                            device_sessions.error = None;
                                            device_sessions.status = Some("Session(s) logged out.".to_owned());
                                            device_sessions.challenge = None;
                                        }
                                        Err(e) => {
                                            device_sessions.loading = false;
                                            device_sessions.error = Some(format!("Logged out, but failed to refresh sessions: {e}"));
                                        }
                                    }
                                }
                                Err(e) => {
                                    device_sessions.loading = false;
                                    match e {
                                        lampcord_core::api::sessions::SessionsError::Failure(f) => {
                                            let challenge = device_session_challenge_from_failure(&id_hashes, &f, None);
                                            if let Some(challenge) = challenge {
                                                device_sessions.error = None;
                                                device_sessions.status = None;
                                                device_sessions.challenge = Some(challenge);
                                            } else {
                                                device_sessions.error = Some(
                                                    f.message()
                                                        .unwrap_or("Failed to log out session(s).")
                                                        .to_owned(),
                                                );
                                                device_sessions.status = None;
                                                device_sessions.challenge = None;
                                            }
                                        }
                                        other => {
                                            device_sessions.error = Some(format!("Failed to log out session(s): {other}"));
                                            device_sessions.status = None;
                                            device_sessions.challenge = None;
                                        }
                                    }
                                }
                            }
                        } else {
                            device_sessions.error = Some("Not connected. Log in first to manage device sessions.".to_owned());
                        }
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::SubmitDeviceSessionPassword { password } => {
                        let (pending, _had_password_context) = match device_sessions.challenge.clone() {
                            Some(DeviceSessionChallenge::Password { pending_id_hashes, .. }) => (pending_id_hashes, false),
                            Some(DeviceSessionChallenge::Totp { pending_id_hashes, .. }) => (pending_id_hashes, true),
                            Some(DeviceSessionChallenge::BackupCode { pending_id_hashes, .. }) => (pending_id_hashes, true),
                            None => (Vec::new(), false),
                        };
                        if pending.is_empty() {
                            device_sessions.error = Some("No pending device session challenge.".to_owned());
                            push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                            continue;
                        }
                        if let Some(rest_client) = rest.clone() {
                            device_sessions.loading = true;
                            device_sessions.error = None;
                            device_sessions.status = Some("Verifying password...".to_owned());
                            push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                            let config = rest_config(active_token.clone(), &identity);
                            match lampcord_core::api::sessions::logout_sessions(&*rest_client, &config, &pending, Some(&password), None, None).await {
                                Ok(()) => {
                                    device_sessions.challenge = None;
                                    device_sessions.status = Some("Session(s) logged out. Refreshing...".to_owned());
                                    match lampcord_core::api::sessions::list_sessions(&*rest_client, &config).await {
                                        Ok(sessions) => {
                                            device_sessions.sessions = sessions.into_iter().map(|s| DeviceSessionView {
                                                id_hash: s.id_hash, name: s.name, location: s.location, last_used: s.last_used, current: s.current,
                                                icon: match s.icon { lampcord_core::api::sessions::DeviceIcon::Mobile => "mobile".to_owned(), lampcord_core::api::sessions::DeviceIcon::Desktop => "desktop".to_owned(), lampcord_core::api::sessions::DeviceIcon::Unknown => "unknown".to_owned() },
                                            }).collect();
                                            device_sessions.loading = false;
                                            device_sessions.error = None;
                                            device_sessions.status = Some("Session(s) logged out.".to_owned());
                                        }
                                        Err(e) => { device_sessions.loading = false; device_sessions.error = Some(format!("Logged out, but failed to refresh sessions: {e}")); }
                                    }
                                }
                                Err(e) => {
                                    device_sessions.loading = false;
                                    match e {
                                        lampcord_core::api::sessions::SessionsError::Failure(f) => {
                                            let challenge = device_session_challenge_from_failure(&pending, &f, Some(password.clone()));
                                            if let Some(challenge) = challenge {
                                                device_sessions.error = None;
                                                device_sessions.status = None;
                                                device_sessions.challenge = Some(challenge);
                                            } else {
                                                device_sessions.error = Some(f.message().unwrap_or("Failed to log out session(s).").to_owned());
                                                device_sessions.status = None;
                                                device_sessions.challenge = Some(DeviceSessionChallenge::Password { pending_id_hashes: pending, message: device_sessions.error.clone() });
                                            }
                                        }
                                        other => {
                                            device_sessions.error = Some(format!("Failed to log out session(s): {other}"));
                                            device_sessions.status = None;
                                        }
                                    }
                                }
                            }
                        }
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::SubmitDeviceSessionCode { code } => {
                        let challenge = device_sessions.challenge.clone();
                        if let Some(rest_client) = rest.clone() {
                            let config = rest_config(active_token.clone(), &identity);
                            match challenge {
                                Some(DeviceSessionChallenge::Totp { pending_id_hashes, ticket, mfa_type, allow_backup: _, password, .. }) => {
                                    device_sessions.loading = true;
                                    device_sessions.error = None;
                                    device_sessions.status = Some("Verifying code...".to_owned());
                                    push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                                    match lampcord_core::api::sessions::finish_mfa(&*rest_client, &config, &ticket, &mfa_type, &code).await {
                                        Ok(mfa_token) => {
                                            device_sessions.status = Some("Code verified. Logging out session(s)...".to_owned());
                                            push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                                            match lampcord_core::api::sessions::logout_sessions(&*rest_client, &config, &pending_id_hashes, password.as_deref(), Some(&mfa_token), None).await {
                                                Ok(()) => {
                                                    device_sessions.challenge = None;
                                                    match lampcord_core::api::sessions::list_sessions(&*rest_client, &config).await {
                                                        Ok(sessions) => {
                                                            device_sessions.sessions = sessions.into_iter().map(|s| DeviceSessionView {
                                                                id_hash: s.id_hash, name: s.name, location: s.location, last_used: s.last_used, current: s.current,
                                                                icon: match s.icon { lampcord_core::api::sessions::DeviceIcon::Mobile => "mobile".to_owned(), lampcord_core::api::sessions::DeviceIcon::Desktop => "desktop".to_owned(), lampcord_core::api::sessions::DeviceIcon::Unknown => "unknown".to_owned() },
                                                            }).collect();
                                                            device_sessions.loading = false;
                                                            device_sessions.error = None;
                                                            device_sessions.status = Some("Session(s) logged out.".to_owned());
                                                        }
                                                        Err(e) => { device_sessions.loading = false; device_sessions.error = Some(format!("Logged out, but failed to refresh sessions: {e}")); device_sessions.status = None; }
                                                    }
                                                }
                                                Err(e) => {
                                                    device_sessions.loading = false;
                                                    match e {
                                                        lampcord_core::api::sessions::SessionsError::Failure(f) => {
                                                            let next = device_session_challenge_from_failure(&pending_id_hashes, &f, password.clone());
                                                            if let Some(next) = next {
                                                                device_sessions.challenge = Some(next);
                                                                device_sessions.error = None;
                                                            } else {
                                                                device_sessions.challenge = Some(DeviceSessionChallenge::Totp { pending_id_hashes, ticket, mfa_type, allow_backup: true, password, message: Some(f.message().unwrap_or("Failed to log out session(s).").to_owned()) });
                                                                device_sessions.error = Some(f.message().unwrap_or("Failed to log out session(s).").to_owned());
                                                            }
                                                            device_sessions.status = None;
                                                        }
                                                        other => {
                                                            device_sessions.error = Some(format!("Failed to log out session(s): {other}"));
                                                            device_sessions.status = None;
                                                            device_sessions.challenge = Some(DeviceSessionChallenge::Totp { pending_id_hashes, ticket, mfa_type, allow_backup: true, password, message: device_sessions.error.clone() });
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            device_sessions.loading = false;
                                            device_sessions.error = Some(format!("Could not verify code: {e}"));
                                            device_sessions.status = None;
                                            device_sessions.challenge = Some(DeviceSessionChallenge::Totp { pending_id_hashes, ticket, mfa_type, allow_backup: true, password, message: device_sessions.error.clone() });
                                        }
                                    }
                                }
                                Some(DeviceSessionChallenge::BackupCode { pending_id_hashes, password, .. }) => {
                                    device_sessions.loading = true;
                                    device_sessions.error = None;
                                    device_sessions.status = Some("Verifying backup code...".to_owned());
                                    push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                                    match lampcord_core::api::sessions::logout_sessions(&*rest_client, &config, &pending_id_hashes, password.as_deref(), None, Some(&code)).await {
                                        Ok(()) => {
                                            device_sessions.challenge = None;
                                            match lampcord_core::api::sessions::list_sessions(&*rest_client, &config).await {
                                                Ok(sessions) => {
                                                    device_sessions.sessions = sessions.into_iter().map(|s| DeviceSessionView {
                                                        id_hash: s.id_hash, name: s.name, location: s.location, last_used: s.last_used, current: s.current,
                                                        icon: match s.icon { lampcord_core::api::sessions::DeviceIcon::Mobile => "mobile".to_owned(), lampcord_core::api::sessions::DeviceIcon::Desktop => "desktop".to_owned(), lampcord_core::api::sessions::DeviceIcon::Unknown => "unknown".to_owned() },
                                                    }).collect();
                                                    device_sessions.loading = false;
                                                    device_sessions.error = None;
                                                    device_sessions.status = Some("Session(s) logged out.".to_owned());
                                                }
                                                Err(e) => { device_sessions.loading = false; device_sessions.error = Some(format!("Logged out, but failed to refresh sessions: {e}")); device_sessions.status = None; }
                                            }
                                        }
                                        Err(e) => {
                                            device_sessions.loading = false;
                                            match e {
                                                lampcord_core::api::sessions::SessionsError::Failure(f) => {
                                                    let next = device_session_challenge_from_failure(&pending_id_hashes, &f, password.clone());
                                                    if let Some(next) = next {
                                                        device_sessions.challenge = Some(next);
                                                        device_sessions.error = None;
                                                    } else {
                                                        device_sessions.error = Some(f.message().unwrap_or("Failed to log out session(s).").to_owned());
                                                        device_sessions.challenge = Some(DeviceSessionChallenge::BackupCode { pending_id_hashes, password, message: device_sessions.error.clone() });
                                                    }
                                                    device_sessions.status = None;
                                                }
                                                other => {
                                                    device_sessions.error = Some(format!("Failed to log out session(s): {other}"));
                                                    device_sessions.status = None;
                                                    device_sessions.challenge = Some(DeviceSessionChallenge::BackupCode { pending_id_hashes, password, message: device_sessions.error.clone() });
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    device_sessions.error = Some("No pending code challenge.".to_owned());
                                }
                            }
                        }
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::UseDeviceSessionBackupCode => {
                        if let Some(DeviceSessionChallenge::Totp { pending_id_hashes, password, .. }) = device_sessions.challenge.clone() {
                            device_sessions.challenge = Some(DeviceSessionChallenge::BackupCode {
                                pending_id_hashes,
                                password,
                                message: Some("Enter a backup code.".to_owned()),
                            });
                            device_sessions.error = None;
                            device_sessions.status = None;
                        }
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::CancelDeviceSessionChallenge => {
                        device_sessions.challenge = None;
                        device_sessions.error = None;
                        device_sessions.status = None;
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::Logout => {
                        login_flow = LoginFlowView::default();
                        forgot_password = ForgotPasswordView::default();
                        device_sessions = DeviceSessionsView::default();
                        if let Some(task) = gateway_task.take() { task.abort(); }
                        gateway_events = None;
                        active_token = None;
                        _should_reconnect = false;
                        _reconnect_attempt = 0;
                        cache = None;
                        rest = None;
                        clear_fetched_messages!();
                        guild_folders.clear();
                        selected_guild_id = None;
                        selected_channel_id = None;
                        status = "Logged out".to_owned();
                        error = None;
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::StartRemoteAuth => {
                        if let Some(cancel) = remote_auth_cancel.take() { let _ = cancel.send(()); }
                        if let Some(task) = remote_auth_task.take() { task.abort(); }
                        remote_auth_rx = None;
                        let (state_tx, state_rx) = tokio::sync::watch::channel(RemoteAuthState::Connecting);
                        remote_auth_view = remote_auth_view_from_state(&RemoteAuthState::Connecting);
                        remote_auth_rx = Some(state_rx);
                        let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();
                        remote_auth_cancel = Some(cancel_tx);
                        let ra_rest = match ReqwestRestTransport::new() {
                            Ok(t) => Arc::new(RestClient::new(rest_config(None, &identity), t, NoopCaptchaSolver)),
                            Err(e) => {
                                remote_auth_view = RemoteAuthView {
                                    status: "QR login failed".to_owned(),
                                    error: Some(format!("REST transport init failed: {e}")),
                                    ..RemoteAuthView::default()
                                };
                                push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                                continue;
                            }
                        };
                        let ra_config = rest_config(None, &identity);
                        remote_auth_task = Some(tokio::spawn(async move {
                            remote_auth::run_remote_auth(ra_rest, ra_config, state_tx, cancel_rx).await;
                        }));
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::StopRemoteAuth => {
                        if let Some(cancel) = remote_auth_cancel.take() { let _ = cancel.send(()); }
                        if let Some(task) = remote_auth_task.take() { task.abort(); }
                        remote_auth_rx = None;
                        remote_auth_view = RemoteAuthView::default();
                        push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                    }
                    BackendCommand::Shutdown => {
                        if let Some(cancel) = remote_auth_cancel.take() { let _ = cancel.send(()); }
                        if let Some(task) = remote_auth_task.take() { task.abort(); }
                        if let Some(task) = gateway_task.take() { task.abort(); }
                        break;
                    }
                }
            }
            Some(event) = async {
                match gateway_events.as_mut() {
                    Some(rx) => rx.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                let mut events = vec![event];
                if let Some(rx) = gateway_events.as_mut() {
                    while let Ok(ev) = rx.try_recv() {
                        events.push(ev);
                        if events.len() >= 64 {
                            break;
                        }
                    }
                }
                crate::ui::debug_stats::record_gateway_batch(events.len());

                let mut should_push = false;
                for event in events {
                    match &event {
                        GatewayEvent::Hello { .. } => {
                            status = "Connected (handshake)".to_owned();
                            should_push = true;
                        }
                        GatewayEvent::Ready { user_id, .. } => {
                            status = "Ready".to_owned();
                            _reconnect_attempt = 0;
                            eprintln!("[lampcord] gateway READY (user_id={:?})", user_id);
                            if let (Some(token), Some(cache_arc)) =
                                (active_token.clone(), cache.clone())
                            {
                                let username = cache_arc
                                    .lock()
                                    .ok()
                                    .and_then(|c| c.current_user().map(|u| u.username.clone()));
                                if let Some(uid) = user_id.clone().or_else(|| {
                                    cache_arc
                                        .lock()
                                        .ok()
                                        .and_then(|c| c.current_user().map(|u| u.id.to_string()))
                                }) {
                                    token_store.add(SavedAccount {
                                        token,
                                        user_id: uid,
                                        username,
                                    });
                                    save_token_store(&token_store_path, &token_store);
                                }
                            }
                            if let (Some(rest_client), Some(_cache_arc)) =
                                (rest.clone(), cache.clone())
                            {
                                let config = rest_config(active_token.clone(), &identity);
                                let (folders, refresh_err) =
                                    refresh_guild_metadata_and_folders(
                                        &rest_client,
                                        &config,
                                        &cache,
                                    )
                                    .await;
                                if let Some(folders) = folders {
                                    guild_folders = folders;
                                }
                                if error.is_none() {
                                    error = refresh_err;
                                }
                            }
                            if let (Some(cid), Some(rest_client)) =
                                (selected_channel_id, rest.clone())
                            {
                                let config = rest_config(active_token.clone(), &identity);
                                if let Ok(mut msgs) =
                                    lampcord_core::api::fetch_channel_messages(
                                        &*rest_client,
                                        &config,
                                        cid,
                                        50,
                                        None,
                                    )
                                    .await
                                {
                                    msgs.sort_by(|a, b| a.id.cmp(&b.id));
                                    fetched_messages.insert(cid, msgs);
                                    touch_fetched_channel!(cid);
                                    cap_fetched_channel!(cid);
                                }
                            }
                            should_push = true;
                        }
                        GatewayEvent::Resumed => {
                            status = "Resumed".to_owned();
                            _reconnect_attempt = 0;
                            eprintln!("[lampcord] gateway resumed");
                            should_push = true;
                        }
                        GatewayEvent::AuthFailed { token } => {
                            status = "Authentication failed".to_owned();
                            error = Some("Gateway authentication failed (invalid token?)".to_owned());
                            eprintln!("[lampcord] gateway auth failed");
                            token_store.remove(&token);
                            save_token_store(&token_store_path, &token_store);
                            _should_reconnect = false;
                            should_push = true;
                        }
                        GatewayEvent::InvalidSession { resumable } => {
                            status = "Reconnecting (invalid session)...".to_owned();
                            eprintln!("[lampcord] invalid session resumable={resumable}");
                            should_push = true;
                        }
                        GatewayEvent::ReconnectRequested => {
                            status = "Reconnecting...".to_owned();
                            eprintln!("[lampcord] reconnect requested");
                            should_push = true;
                        }
                        GatewayEvent::Closed { code } => {
                            if matches!(
                                code,
                                Some(4004)
                                    | Some(4010)
                                    | Some(4011)
                                    | Some(4012)
                                    | Some(4013)
                                    | Some(4014)
                            ) {
                                status = format!("Disconnected ({:?})", code);
                            } else {
                                status = format!("Reconnecting (closed {:?})...", code);
                            }
                            eprintln!("[lampcord] gateway closed code={:?}", code);
                            should_push = true;
                        }
                        GatewayEvent::Dispatch { .. } | GatewayEvent::HandlerErrors { .. } => {
                            should_push = true;
                        }
                        GatewayEvent::HeartbeatSent | GatewayEvent::HeartbeatAck => {}
                    }
                }
                if should_push {
                    push(
                        &status,
                        &cache,
                        selected_guild_id,
                        selected_channel_id,
                        &fetched_messages,
                        &token_store,
                        error.clone(),
                        &remote_auth_view,
                        &login_flow,
                        &forgot_password,
                        &device_sessions,
                        &guild_folders,
                        &evt_tx,
                        &snapshot,
                    );
                }
            }
            changed = async {
                match remote_auth_rx.as_mut() {
                    Some(rx) => rx.changed().await.map(|_| ()).ok(),
                    None => std::future::pending::<Option<()>>().await,
                }
            } => {
                if changed.is_some() {
                    if let Some(rx) = remote_auth_rx.as_ref() {
                        let state = rx.borrow().clone();
                        remote_auth_view = remote_auth_view_from_state(&state);
                        if let RemoteAuthState::Finished { token } = state {
                            if let Some(cancel) = remote_auth_cancel.take() { let _ = cancel.send(()); }
                            remote_auth_rx = None;
                            if let Some(task) = remote_auth_task.take() { let _ = task; }
                            if let Some(task) = gateway_task.take() { task.abort(); }
                            gateway_events = None;
                            status = "Connecting...".to_owned();
                            error = None;
                            active_token = Some(token.clone());
                            _should_reconnect = true;
                            _reconnect_attempt = 0;
                            clear_fetched_messages!();
                            let transport = match ReqwestRestTransport::new() {
                                Ok(t) => t,
                                Err(e) => {
                                    status = "Error".to_owned();
                                    error = Some(format!("REST transport init failed: {e}"));
                                    _should_reconnect = false;
                                    push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                                    continue;
                                }
                            };
                            let rest_client = Arc::new(RestClient::new(rest_config(Some(token.clone()), &identity), transport, NoopCaptchaSolver));
                            let gw_transport = TungsteniteGatewayTransport::default();
                            let url_provider = StaticGatewayUrl { rest: rest_client.clone() };
                            let gw_identity = lampcord_core::gateway_client::GatewayIdentity::desktop(Some(identity.device_vendor_id.clone()));
                            let mut client = GatewayClient::new(gw_transport, url_provider).with_identity(gw_identity);
                            let client_cache = client.cache().clone();
                            cache = Some(client_cache.clone());
                            rest = Some(rest_client.clone());
                            let (gw_tx, gw_rx) = mpsc::unbounded_channel();
                            gateway_events = Some(gw_rx);
                            let token_clone = token.clone();
                            gateway_task = Some(tokio::spawn(async move {
                                let mut attempt: u32 = 0;
                                loop {
                                    let result = client.run_stream(&token_clone, gw_tx.clone()).await;
                                    let terminal = match result {
                                        Ok(t) => t,
                                        Err(e) => {
                                            eprintln!("[lampcord] gateway error: {e}");
                                            None
                                        }
                                    };
                                    let reconnect = match terminal {
                                        Some(GatewayEvent::AuthFailed { .. }) => false,
                                        Some(GatewayEvent::Closed { code }) if matches!(code, Some(4004) | Some(4010) | Some(4011) | Some(4012) | Some(4013) | Some(4014)) => false,
                                        Some(GatewayEvent::ReconnectRequested)
                                        | Some(GatewayEvent::InvalidSession { .. })
                                        | Some(GatewayEvent::Closed { .. })
                                        | None => true,
                                        _ => true,
                                    };
                                    if !reconnect { break; }
                                    let delay_ms = reconnect_delay_ms(attempt);
                                    attempt = attempt.saturating_add(1);
                                    eprintln!("[lampcord] reconnecting in {delay_ms}ms (attempt {attempt})");
                                    tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                                }
                            }));
                        }
                    }
                    push(&status, &cache, selected_guild_id, selected_channel_id, &fetched_messages, &token_store, error.clone(), &remote_auth_view, &login_flow, &forgot_password, &device_sessions, &guild_folders, &evt_tx, &snapshot);
                }
            }
            else => break,
        }
    }
}
