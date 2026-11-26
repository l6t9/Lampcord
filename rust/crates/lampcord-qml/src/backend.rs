//! The single QObject QML talks to. It owns a `LampcordClient`, polls its
//! snapshot patches from a QML timer and republishes them as properties.
//!
//! Snowflakes cross the bridge as strings: they do not fit a JS number.
//! Lists cross as JSON strings, already shaped for the views, so QML never
//! sorts or groups anything itself.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, connection)]
        #[qproperty(bool, has_token)]
        #[qproperty(QString, current_user)]
        #[qproperty(QString, current_user_username)]
        #[qproperty(QString, current_user_email)]
        #[qproperty(bool, current_user_mfa_enabled)]
        #[qproperty(QString, current_user_id)]
        #[qproperty(QString, current_user_avatar)]
        #[qproperty(QString, current_user_status)]
        #[qproperty(QString, current_user_custom_status)]
        #[qproperty(QString, current_user_decoration)]
        #[qproperty(QString, current_user_nameplate)]
        #[qproperty(QString, selected_guild_id)]
        #[qproperty(QString, selected_channel_id)]
        #[qproperty(QString, guilds_json)]
        #[qproperty(QString, rail_json)]
        #[qproperty(QString, dms_json)]
        #[qproperty(QString, channels_json)]
        #[qproperty(QString, messages_json)]
        #[qproperty(QString, members_json)]
        #[qproperty(QString, pins_json)]
        #[qproperty(QString, mentions_json)]
        #[qproperty(QString, profile_json)]
        #[qproperty(QString, relationships_json)]
        #[qproperty(QString, search_results_json)]
        #[qproperty(QString, guild_emojis_json)]
        #[qproperty(QString, guild_roles_json)]
        #[qproperty(QString, threads_json)]
        #[qproperty(QString, guild_bans_json)]
        #[qproperty(QString, saved_accounts_json)]
        #[qproperty(QString, config_json)]
        #[qproperty(QString, toast_title)]
        #[qproperty(QString, toast_body)]
        #[qproperty(QString, toast_channel_id)]
        #[qproperty(i32, toast_seq)]
        #[qproperty(bool, log_deleted_messages)]
        #[qproperty(bool, desktop_notifications)]
        #[qproperty(QString, auth_state)]
        #[qproperty(QString, auth_message)]
        #[qproperty(QString, error_text)]
        #[qproperty(QString, qr_state)]
        #[qproperty(QString, qr_modules)]
        #[qproperty(i32, qr_size)]
        #[qproperty(QString, qr_user)]
        type LampcordBackend = super::BackendRust;

        #[qinvokable]
        fn poll(self: Pin<&mut LampcordBackend>);

        #[qinvokable]
        fn login_token(self: Pin<&mut LampcordBackend>, token: QString);

        #[qinvokable]
        fn login_password(self: Pin<&mut LampcordBackend>, login: QString, password: QString);

        #[qinvokable]
        fn submit_mfa(self: Pin<&mut LampcordBackend>, code: QString);

        #[qinvokable]
        fn send_mfa_sms(self: Pin<&mut LampcordBackend>);

        #[qinvokable]
        fn forgot_password(self: Pin<&mut LampcordBackend>, login: QString);

        #[qinvokable]
        fn fetch_profile(self: Pin<&mut LampcordBackend>, user_id: QString);

        #[qinvokable]
        fn reset_auth(self: Pin<&mut LampcordBackend>);

        #[qinvokable]
        fn start_qr_login(self: Pin<&mut LampcordBackend>);

        #[qinvokable]
        fn cancel_qr_login(self: Pin<&mut LampcordBackend>);

        #[qinvokable]
        fn select_guild(self: Pin<&mut LampcordBackend>, guild_id: QString);

        #[qinvokable]
        fn select_channel(self: Pin<&mut LampcordBackend>, channel_id: QString);

        #[qinvokable]
        fn fetch_pins(self: Pin<&mut LampcordBackend>);

        #[qinvokable]
        fn fetch_mentions(self: Pin<&mut LampcordBackend>);

        #[qinvokable]
        fn load_older(self: Pin<&mut LampcordBackend>);

        #[qinvokable]
        fn send_message(self: Pin<&mut LampcordBackend>, content: QString);

        #[qinvokable]
        fn send_reply(self: Pin<&mut LampcordBackend>, content: QString, message_id: QString);

        /// `files` is a JSON array of file:// URLs from a file dialog.
        #[qinvokable]
        fn send_files(self: Pin<&mut LampcordBackend>, content: QString, files: QString);

        /// online, idle, dnd or invisible.
        #[qinvokable]
        fn set_status(self: Pin<&mut LampcordBackend>, status: QString);

        #[qinvokable]
        fn set_custom_status(self: Pin<&mut LampcordBackend>, text: QString);

        #[qinvokable]
        fn edit_message(self: Pin<&mut LampcordBackend>, message_id: QString, content: QString);

        #[qinvokable]
        fn delete_message(self: Pin<&mut LampcordBackend>, message_id: QString);

        #[qinvokable]
        fn send_typing(self: Pin<&mut LampcordBackend>);

        #[qinvokable]
        fn ack_latest(self: Pin<&mut LampcordBackend>);

        #[qinvokable]
        fn mark_guild_read(self: Pin<&mut LampcordBackend>, guild_id: QString);

        #[qinvokable]
        fn add_reaction(self: Pin<&mut LampcordBackend>, message_id: QString, emoji: QString);

        #[qinvokable]
        fn remove_reaction(self: Pin<&mut LampcordBackend>, message_id: QString, emoji: QString);

        #[qinvokable]
        fn pin_message(self: Pin<&mut LampcordBackend>, message_id: QString);

        #[qinvokable]
        fn unpin_message(self: Pin<&mut LampcordBackend>, message_id: QString);

        #[qinvokable]
        fn join_guild(self: Pin<&mut LampcordBackend>, invite_code: QString);

        #[qinvokable]
        fn leave_guild(self: Pin<&mut LampcordBackend>, guild_id: QString);

        #[qinvokable]
        fn send_friend_request(self: Pin<&mut LampcordBackend>, username: QString);

        #[qinvokable]
        fn accept_friend_request(self: Pin<&mut LampcordBackend>, user_id: QString);

        #[qinvokable]
        fn remove_relationship(self: Pin<&mut LampcordBackend>, user_id: QString);

        #[qinvokable]
        fn block_user(self: Pin<&mut LampcordBackend>, user_id: QString);

        #[qinvokable]
        fn open_dm(self: Pin<&mut LampcordBackend>, user_id: QString);

        #[qinvokable]
        fn search_messages(self: Pin<&mut LampcordBackend>, query: QString, scope: QString);

        #[qinvokable]
        fn fetch_threads(self: Pin<&mut LampcordBackend>);

        #[qinvokable]
        fn create_thread(self: Pin<&mut LampcordBackend>, name: QString, message_id: QString);

        #[qinvokable]
        fn fetch_guild_settings(self: Pin<&mut LampcordBackend>, guild_id: QString);

        #[qinvokable]
        fn create_guild_role(self: Pin<&mut LampcordBackend>, guild_id: QString, name: QString, color: i32);

        #[qinvokable]
        fn delete_guild_role(self: Pin<&mut LampcordBackend>, guild_id: QString, role_id: QString);

        #[qinvokable]
        fn delete_guild_emoji(self: Pin<&mut LampcordBackend>, guild_id: QString, emoji_id: QString);

        #[qinvokable]
        fn unban_member(self: Pin<&mut LampcordBackend>, guild_id: QString, user_id: QString);

        #[qinvokable]
        fn create_channel(self: Pin<&mut LampcordBackend>, guild_id: QString, name: QString, kind: i32);

        #[qinvokable]
        fn delete_channel(self: Pin<&mut LampcordBackend>, channel_id: QString);

        #[qinvokable]
        fn switch_account(self: Pin<&mut LampcordBackend>, user_id: QString);

        #[qinvokable]
        fn remove_saved_account(self: Pin<&mut LampcordBackend>, user_id: QString);

        #[qinvokable]
        fn clear_deleted_log(self: Pin<&mut LampcordBackend>);

        #[qinvokable]
        fn render_markdown(
            self: &LampcordBackend,
            content: QString,
            code_bg: QString,
            mention_bg: QString,
            mention_fg: QString,
            muted_fg: QString,
            mono_font: QString,
        ) -> QString;

        #[qinvokable]
        fn set_config(self: Pin<&mut LampcordBackend>, key: QString, value: QString);

        #[qinvokable]
        fn logout(self: Pin<&mut LampcordBackend>);
    }
}

use std::collections::HashMap;
use std::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use lampcord_ffi::{
    AuthChallenge, ChannelSnapshot, ClientConfig, ClientEvent, ConnectionState,
    GuildFolderSnapshot, GuildSnapshot, LampcordClient, MEMBER_LIST_ROW_GROUP,
    MEMBER_LIST_ROW_MEMBER, MemberListSnapshot, MemberSnapshot, MessageSnapshot, PresenceSnapshot,
    PrivateChannelSnapshot, ProfileSnapshot, ReadStateSnapshot, RelationshipSnapshot,
    RemoteAuthState, RoleSnapshot, SnapshotVersions,
};
use serde_json::{Value, json};

const DISCORD_EPOCH_MS: u64 = 1_420_070_400_000;
/// Same window the Compose client uses to fold consecutive messages.
const GROUP_WINDOW_MS: u64 = 7 * 60 * 1000;
const CDN: &str = "https://cdn.discordapp.com";

const CHANNEL_TEXT: u8 = 0;
const CHANNEL_VOICE: u8 = 2;
const CHANNEL_CATEGORY: u8 = 4;
const CHANNEL_ANNOUNCEMENT: u8 = 5;
const CHANNEL_STAGE: u8 = 13;
const CHANNEL_FORUM: u8 = 15;

/// The signed-in user, as the member list and message rows need it.
#[derive(Clone, PartialEq)]
struct Me {
    id: u64,
    name: String,
    avatar: Option<String>,
    decoration: Option<String>,
    nameplate: Option<String>,
    status: String,
    custom_status: Option<String>,
}

pub struct BackendRust {
    client: LampcordClient,
    versions: SnapshotVersions,
    guilds: Vec<GuildSnapshot>,
    guild_folders: Vec<GuildFolderSnapshot>,
    channels: Vec<ChannelSnapshot>,
    private_channels: Vec<PrivateChannelSnapshot>,
    messages: Vec<MessageSnapshot>,
    mentions: Vec<MessageSnapshot>,
    members: Vec<MemberSnapshot>,
    member_list: MemberListSnapshot,
    guild_roles: Vec<RoleSnapshot>,
    presences: Vec<PresenceSnapshot>,
    read_states: Vec<ReadStateSnapshot>,
    relationships: Vec<RelationshipSnapshot>,
    deleted_messages: HashMap<u64, Vec<MessageSnapshot>>,
    loaded_db_channels: std::collections::HashSet<u64>,
    me: Option<Me>,
    selected_guild: Option<u64>,
    selected_channel: Option<u64>,
    latest_profile: Option<ProfileSnapshot>,
    mfa_ticket: Option<String>,
    mfa_method: String,
    acked: Option<u64>,

    connection: QString,
    has_token: bool,
    current_user: QString,
    current_user_username: QString,
    current_user_email: QString,
    current_user_mfa_enabled: bool,
    current_user_id: QString,
    current_user_avatar: QString,
    current_user_status: QString,
    current_user_custom_status: QString,
    current_user_decoration: QString,
    current_user_nameplate: QString,
    selected_guild_id: QString,
    selected_channel_id: QString,
    guilds_json: QString,
    rail_json: QString,
    dms_json: QString,
    channels_json: QString,
    messages_json: QString,
    members_json: QString,
    pins_json: QString,
    mentions_json: QString,
    profile_json: QString,
    relationships_json: QString,
    search_results_json: QString,
    guild_emojis_json: QString,
    guild_roles_json: QString,
    threads_json: QString,
    guild_bans_json: QString,
    saved_accounts_json: QString,
    config_json: QString,
    toast_title: QString,
    toast_body: QString,
    toast_channel_id: QString,
    toast_seq: i32,
    log_deleted_messages: bool,
    desktop_notifications: bool,
    auth_state: QString,
    auth_message: QString,
    error_text: QString,
    qr_state: QString,
    qr_modules: QString,
    qr_size: i32,
    qr_user: QString,
}

impl Default for BackendRust {
    fn default() -> Self {
        let config = ClientConfig {
            storage_path: storage_path(),
            ..ClientConfig::default()
        };
        let client = LampcordClient::start(config);
        let _ = client.connect_stored_token();
        let props = load_lampcord_properties();
        let initial_log_deleted = props.get("message_logger_enabled").is_some_and(|v| v == "true");
        let initial_desktop_notif = props.get("notifications_enabled").map_or(true, |v| v == "true");
        let initial_config_json = properties_to_json(&props);
        Self {
            client,
            versions: SnapshotVersions::default(),
            guilds: Vec::new(),
            guild_folders: Vec::new(),
            channels: Vec::new(),
            private_channels: Vec::new(),
            messages: Vec::new(),
            mentions: Vec::new(),
            members: Vec::new(),
            member_list: MemberListSnapshot::default(),
            guild_roles: Vec::new(),
            presences: Vec::new(),
            read_states: Vec::new(),
            relationships: Vec::new(),
            deleted_messages: HashMap::new(),
            loaded_db_channels: std::collections::HashSet::new(),
            me: None,
            selected_guild: None,
            selected_channel: None,
            latest_profile: None,
            mfa_ticket: None,
            mfa_method: "totp".to_owned(),
            acked: None,
            connection: QString::from("disconnected"),
            has_token: false,
            current_user: QString::default(),
            current_user_username: QString::default(),
            current_user_email: QString::default(),
            current_user_mfa_enabled: false,
            current_user_id: QString::default(),
            current_user_avatar: QString::default(),
            current_user_status: QString::from("online"),
            current_user_custom_status: QString::default(),
            current_user_decoration: QString::default(),
            current_user_nameplate: QString::default(),
            selected_guild_id: QString::default(),
            selected_channel_id: QString::default(),
            guilds_json: QString::from("[]"),
            rail_json: QString::from("[]"),
            dms_json: QString::from("[]"),
            channels_json: QString::from("[]"),
            messages_json: QString::from("[]"),
            members_json: QString::from("[]"),
            pins_json: QString::from("[]"),
            mentions_json: QString::from("[]"),
            profile_json: QString::from("{}"),
            relationships_json: QString::from("[]"),
            search_results_json: QString::from("[]"),
            guild_emojis_json: QString::from("[]"),
            guild_roles_json: QString::from("[]"),
            threads_json: QString::from("[]"),
            guild_bans_json: QString::from("[]"),
            saved_accounts_json: QString::from("[]"),
            config_json: QString::from(initial_config_json.as_str()),
            toast_title: QString::default(),
            toast_body: QString::default(),
            toast_channel_id: QString::default(),
            toast_seq: 0,
            log_deleted_messages: initial_log_deleted,
            desktop_notifications: initial_desktop_notif,
            auth_state: QString::from("idle"),
            auth_message: QString::default(),
            error_text: QString::default(),
            qr_state: QString::from("idle"),
            qr_modules: QString::default(),
            qr_size: 0,
            qr_user: QString::default(),
        }
    }
}

impl Drop for BackendRust {
    fn drop(&mut self) {
        let _ = self.client.shutdown();
    }
}

/// `$XDG_DATA_HOME/lampcord-qml/accounts.json`, kept apart from the Compose
/// client's store so both can run side by side while the rewrite matures.
fn lampcord_properties_path() -> std::path::PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("lampcord").join("settings.properties")
}

fn unescape_java_prop(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some(other) => out.push(other),
                None => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn load_lampcord_properties() -> std::collections::BTreeMap<String, String> {
    let mut map = std::collections::BTreeMap::new();
    let Ok(text) = std::fs::read_to_string(lampcord_properties_path()) else {
        return map;
    };
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            let key = unescape_java_prop(k.trim());
            if matches!(key.as_str(), "discord_token" | "saved_accounts" | "rust_core_secrets") {
                continue;
            }
            map.insert(key, unescape_java_prop(v.trim()));
        }
    }
    map
}

fn properties_to_json(props: &std::collections::BTreeMap<String, String>) -> String {
    let mut obj = serde_json::Map::new();
    for (k, v) in props {
        obj.insert(k.clone(), Value::String(v.clone()));
    }
    Value::Object(obj).to_string()
}

fn save_lampcord_property(key: &str, value: &str) {
    let path = lampcord_properties_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let escaped_val = value.replace('\\', "\\\\").replace('\n', "\\n").replace('\r', "\\r");
    let new_line = format!("{key}={escaped_val}");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<String> = existing.lines().map(str::to_owned).collect();
    let mut replaced = false;
    for line in &mut lines {
        let trimmed = line.trim();
        if !trimmed.starts_with('#') && !trimmed.starts_with('!') {
            if let Some((k, _)) = trimmed.split_once('=') {
                if unescape_java_prop(k.trim()) == key {
                    *line = new_line.clone();
                    replaced = true;
                    break;
                }
            }
        }
    }
    if !replaced {
        lines.push(new_line);
    }
    let mut out = lines.join("\n");
    out.push('\n');
    let tmp = path.with_extension("properties.tmp");
    if std::fs::write(&tmp, out).is_ok() {
        let _ = std::fs::rename(&tmp, &path);
    }
}

fn sha256_hex(input: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let bit_len = (input.len() as u64) * 8;
    let mut data = input.to_vec();
    data.push(0x80);
    while (data.len() % 64) != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in data.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[i * 4], chunk[i * 4 + 1], chunk[i * 4 + 2], chunk[i * 4 + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g; g = f; f = e; e = d.wrapping_add(t1);
            d = c; c = b; b = a; a = t1.wrapping_add(t2);
        }
        for (slot, delta) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *slot = slot.wrapping_add(delta);
        }
    }
    let mut out = String::with_capacity(64);
    for word in h {
        use std::fmt::Write;
        let _ = write!(&mut out, "{word:08x}");
    }
    out
}

/// Looks up an image in Compose's Coil3 disk cache (`~/.config/lampcord/image_cache/<sha256>.1`)
/// so expired attachment URLs from deleted messages still render locally.
fn cached_image_file_url(url: &str) -> Option<String> {
    if url.is_empty() {
        return None;
    }
    let base = std::env::var_os("HOME")
        .map(|h| std::path::PathBuf::from(h).join(".config"))
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(std::path::PathBuf::from))?;
    let path = base
        .join("lampcord")
        .join("image_cache")
        .join(format!("{}.1", sha256_hex(url.as_bytes())));
    if path.exists() {
        Some(format!("file://{}", path.to_string_lossy()))
    } else {
        None
    }
}

fn lampcord_db_path() -> std::path::PathBuf {
    let base = std::env::var_os("HOME")
        .map(|h| std::path::PathBuf::from(h).join(".config"))
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(std::path::PathBuf::from))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("lampcord").join("lampcord.db")
}

/// Loads cached deleted messages for a channel from the Compose client's
/// Room SQLite database (`~/.config/lampcord/lampcord.db`, table `logged_messages`).
fn load_logged_deleted_messages(channel_id: u64) -> Vec<MessageSnapshot> {
    let db = lampcord_db_path();
    if !db.exists() {
        return Vec::new();
    }
    let query = format!(
        "SELECT id, channelId, authorId, authorName, content, isDeleted, jsonPayload \
         FROM logged_messages WHERE channelId = '{channel_id}' AND isDeleted = 1 \
         ORDER BY timestamp DESC LIMIT 200;"
    );
    let Ok(output) = std::process::Command::new("sqlite3")
        .arg("-json")
        .arg("-readonly")
        .arg(&db)
        .arg(&query)
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let Ok(rows) = serde_json::from_slice::<Vec<Value>>(&output.stdout) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for row in rows {
        let Some(id) = row["id"].as_str().and_then(|s| s.parse::<u64>().ok()) else {
            continue;
        };
        let payload: Value = row["jsonPayload"]
            .as_str()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(Value::Null);
        let author_obj = &payload["author"];
        let author_id = author_obj["id"]
            .as_str()
            .and_then(|s| s.parse::<u64>().ok())
            .or_else(|| row["authorId"].as_str().and_then(|s| s.parse::<u64>().ok()));
        let author_name = author_obj["global_name"]
            .as_str()
            .filter(|s| !s.is_empty())
            .or_else(|| author_obj["username"].as_str().filter(|s| !s.is_empty()))
            .or_else(|| row["authorName"].as_str().filter(|s| !s.is_empty()))
            .unwrap_or("Unknown")
            .to_owned();
        let author_avatar = author_obj["avatar"].as_str().map(str::to_owned);
        let author_decoration = author_obj["avatar_decoration_data"]["asset"]
            .as_str()
            .map(str::to_owned);
        let content = row["content"]
            .as_str()
            .or_else(|| payload["content"].as_str())
            .unwrap_or("")
            .to_owned();
        let kind = payload["type"].as_u64().unwrap_or(0) as u8;
        let attachments = payload["attachments"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|a| {
                        let aid = a["id"].as_str()?.parse::<u64>().ok()?;
                        let raw_url = a["url"].as_str().unwrap_or("");
                        let raw_proxy = a["proxy_url"].as_str();
                        let cached = raw_proxy
                            .and_then(cached_image_file_url)
                            .or_else(|| cached_image_file_url(raw_url));
                        Some(lampcord_ffi::AttachmentSnapshot {
                            id: aid,
                            filename: a["filename"].as_str().unwrap_or("file").to_owned(),
                            url: cached.clone().unwrap_or_else(|| raw_url.to_owned()),
                            proxy_url: cached.or_else(|| raw_proxy.map(str::to_owned)),
                            content_type: a["content_type"].as_str().map(str::to_owned),
                            size: a["size"].as_u64().unwrap_or(0),
                            width: a["width"].as_u64().map(|n| n as u32),
                            height: a["height"].as_u64().map(|n| n as u32),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        out.push(MessageSnapshot {
            id,
            channel_id,
            author_id,
            author_name,
            author_avatar,
            author_decoration,
            content,
            timestamp: payload["timestamp"].as_str().unwrap_or("").to_owned(),
            edited_timestamp: payload["edited_timestamp"].as_str().map(str::to_owned),
            kind,
            pinned: payload["pinned"].as_bool().unwrap_or(false),
            attachments,
            reply: None,
            reactions: Vec::new(),
            stickers: Vec::new(),
            embeds_json: "[]".to_owned(),
            forwarded_json: "[]".to_owned(),
            author_clan_tag: None,
            author_clan_badge: None,
        });
    }
    out
}

fn storage_path() -> Option<String> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| std::path::Path::new(&home).join(".local/share")))?;
    let dir = base.join("lampcord-qml");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join("accounts.json").to_string_lossy().into_owned())
}

/// A local path from a file:// URL, percent-decoding as QML hands them out.
fn local_path(url: &str) -> Option<String> {
    let encoded = url.strip_prefix("file://").unwrap_or(url);
    let bytes = encoded.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&encoded[i + 1..i + 3], 16) {
                decoded.push(byte);
                i += 3;
                continue;
            }
        }
        decoded.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(decoded).ok().filter(|path| !path.is_empty())
}

fn parse_id(id: &QString) -> Option<u64> {
    id.to_string().trim().parse().ok()
}

fn id_string(id: Option<u64>) -> QString {
    id.map(|id| QString::from(id.to_string().as_str()))
        .unwrap_or_default()
}

fn connection_name(state: &ConnectionState) -> &'static str {
    match state {
        ConnectionState::Disconnected => "disconnected",
        ConnectionState::Connecting => "connecting",
        ConnectionState::Connected => "connected",
        ConnectionState::Ready => "ready",
        ConnectionState::Resuming => "resuming",
        ConnectionState::Error { .. } => "error",
    }
}

fn snowflake_ms(id: u64) -> u64 {
    (id >> 22) + DISCORD_EPOCH_MS
}

/// Avatar hashes may arrive bare or already expanded to a URL.
fn avatar_url(user_id: u64, avatar: Option<&str>) -> String {
    match avatar {
        Some(url) if url.starts_with("http") => url.to_owned(),
        Some(hash) if !hash.is_empty() => {
            let ext = if hash.starts_with("a_") { "gif" } else { "png" };
            format!("{CDN}/avatars/{user_id}/{hash}.{ext}?size=80")
        }
        _ => String::new(),
    }
}

/// Avatar frame image, like CdnUrls.getAvatarDecorationUrl.
fn decoration_url(asset: Option<&str>) -> String {
    asset.map_or_else(String::new, |asset| {
        format!("{CDN}/avatar-decoration-presets/{asset}.png?size=96&passthrough=true")
    })
}

/// Nameplate background, as the Compose rows load it.
fn nameplate_url(asset: Option<&str>) -> String {
    asset.map_or_else(String::new, |asset| {
        format!("{CDN}/assets/collectibles/{asset}img.png?passthrough=true")
    })
}

/// The avatar Discord shows for users without one (new username system).
fn default_avatar_url(user_id: u64) -> String {
    format!("{CDN}/embed/avatars/{}.png", (user_id >> 22) % 6)
}

/// The remote-auth URL as a row-major string of '1' (dark) and '0' modules,
/// plus its width, for QML to paint. Medium error correction, like Compose.
fn qr_modules(url: &str) -> (String, i32) {
    let Ok(code) = qrcode::QrCode::with_error_correction_level(url, qrcode::EcLevel::M) else {
        return (String::new(), 0);
    };
    let modules = code
        .to_colors()
        .into_iter()
        .map(|color| if color == qrcode::Color::Dark { '1' } else { '0' })
        .collect();
    (modules, code.width() as i32)
}

/// The right sidebar for the open channel: everyone in a DM (you first) under
/// "Members — N", or the server's grouped member list (Online / hoisted roles /
/// Offline) with the server owner crown.
#[allow(clippy::too_many_arguments)]
fn members_json(
    selected_guild: Option<&GuildSnapshot>,
    selected_channel: Option<u64>,
    private_channels: &[PrivateChannelSnapshot],
    members: &[MemberSnapshot],
    member_list: &MemberListSnapshot,
    roles: &[RoleSnapshot],
    presences: &[PresenceSnapshot],
    me: Option<&Me>,
) -> String {
    let presence: HashMap<u64, &PresenceSnapshot> =
        presences.iter().map(|presence| (presence.user_id, presence)).collect();
    let role_map: HashMap<u64, &RoleSnapshot> =
        roles.iter().map(|role| (role.id, role)).collect();
    let owner_id = selected_guild.and_then(|g| g.owner_id);

    let role_color = |member_roles: &[u64]| -> String {
        member_roles
            .iter()
            .filter_map(|id| role_map.get(id))
            .filter(|r| r.color != 0)
            .max_by_key(|r| r.position)
            .map(|r| format!("#{:06x}", r.color))
            .unwrap_or_default()
    };

    let row = |id: u64,
               name: String,
               avatar: Option<&str>,
               status: Option<&str>,
               decoration: Option<&str>,
               nameplate: Option<&str>,
               is_owner: bool,
               color: String,
               clan_tag: Option<&str>,
               clan_badge: Option<&str>| {
        let url = avatar_url(id, avatar);
        let own = me.filter(|me| me.id == id);
        let status = own
            .map(|me| me.status.as_str())
            .or_else(|| presence.get(&id).map(|presence| presence.status.as_str()))
            .or(status)
            .unwrap_or("offline")
            .to_owned();
        // Compose shows a phone for people online only on mobile.
        let mobile = own.is_none() && presence.get(&id).is_some_and(|presence| presence.mobile);
        let status = if mobile && status == "online" { "mobile".to_owned() } else { status };
        // Compose shows the custom status, else the first activity.
        let activity = own
            .and_then(|me| me.custom_status.clone())
            .or_else(|| {
                presence.get(&id).and_then(|presence| {
                    presence.custom_status.clone().or_else(|| presence.activities.first().cloned())
                })
            })
            .unwrap_or_default();
        json!({
            "isGroup": false,
            "id": id.to_string(),
            "name": name,
            "avatar": if url.is_empty() { default_avatar_url(id) } else { url },
            "status": status,
            "activity": activity,
            "decoration": decoration_url(decoration),
            "nameplate": nameplate_url(nameplate),
            "owner": is_owner,
            "roleColor": color,
            "clanTag": clan_tag.unwrap_or(""),
            "clanBadge": clan_badge.unwrap_or(""),
        })
    };

    let dm = selected_channel.and_then(|id| private_channels.iter().find(|channel| channel.id == id));
    if let Some(dm) = dm {
        let member_rows: Vec<Value> = me
            .map(|me| {
                row(
                    me.id,
                    me.name.clone(),
                    me.avatar.as_deref(),
                    None,
                    me.decoration.as_deref(),
                    me.nameplate.as_deref(),
                    false,
                    String::new(),
                    None,
                    None,
                )
            })
            .into_iter()
            .chain(dm.recipients.iter().map(|user| {
                let name = user.global_name.clone().unwrap_or_else(|| user.username.clone());
                row(
                    user.id,
                    name,
                    user.avatar.as_deref(),
                    None,
                    user.decoration.as_deref(),
                    user.nameplate.as_deref(),
                    false,
                    String::new(),
                    None,
                    None,
                )
            }))
            .collect();
        let mut out = Vec::with_capacity(member_rows.len() + 1);
        out.push(json!({
            "isGroup": true,
            "label": format!("Members — {}", member_rows.len()),
        }));
        out.extend(member_rows);
        return Value::Array(out).to_string();
    }

    if selected_channel.is_none() {
        return "[]".to_owned();
    }

    // Prefer Discord's authoritative GUILD_MEMBER_LIST_UPDATE rows when available.
    let has_synced_rows = member_list
        .rows
        .iter()
        .any(|r| r.kind == MEMBER_LIST_ROW_GROUP || r.kind == MEMBER_LIST_ROW_MEMBER);
    if has_synced_rows {
        let mut out = Vec::new();
        for (idx, item) in member_list.rows.iter().enumerate() {
            if item.kind == MEMBER_LIST_ROW_GROUP {
                let raw_id = item.group_id.as_deref().unwrap_or("online");
                let group_name = match raw_id {
                    "online" => "Online".to_owned(),
                    "offline" => "Offline".to_owned(),
                    other => other
                        .parse::<u64>()
                        .ok()
                        .and_then(|rid| role_map.get(&rid))
                        .map(|r| r.name.clone())
                        .unwrap_or_else(|| "Role".to_owned()),
                };
                let count = item.group_count.unwrap_or_else(|| {
                    member_list.rows[idx + 1..]
                        .iter()
                        .take_while(|r| r.kind != MEMBER_LIST_ROW_GROUP)
                        .filter(|r| r.kind == MEMBER_LIST_ROW_MEMBER)
                        .count() as u32
                });
                out.push(json!({
                    "isGroup": true,
                    "label": format!("{group_name} — {count}"),
                }));
            } else if item.kind == MEMBER_LIST_ROW_MEMBER {
                if let Some(member) = &item.member {
                    let name = member
                        .nickname
                        .clone()
                        .filter(|nick| !nick.is_empty())
                        .or_else(|| member.global_name.clone())
                        .unwrap_or_else(|| member.username.clone());
                    out.push(row(
                        member.user_id,
                        name,
                        member.avatar.as_deref(),
                        member.status.as_deref(),
                        member.decoration.as_deref(),
                        member.nameplate.as_deref(),
                        owner_id == Some(member.user_id),
                        role_color(&member.roles),
                        member.clan_tag.as_deref(),
                        member.clan_badge.as_deref(),
                    ));
                }
            }
        }
        return Value::Array(out).to_string();
    }

    // Fallback: group cached guild members into Online — N and Offline — M.
    let mut online_rows = Vec::new();
    let mut offline_rows = Vec::new();
    for member in members {
        let name = member
            .nickname
            .clone()
            .filter(|nick| !nick.is_empty())
            .or_else(|| member.global_name.clone())
            .unwrap_or_else(|| member.username.clone());
        let item = row(
            member.user_id,
            name,
            member.avatar.as_deref(),
            member.status.as_deref(),
            member.decoration.as_deref(),
            member.nameplate.as_deref(),
            owner_id == Some(member.user_id),
            role_color(&member.roles),
            member.clan_tag.as_deref(),
            member.clan_badge.as_deref(),
        );
        if matches!(item["status"].as_str(), Some("offline" | "invisible")) {
            offline_rows.push(item);
        } else {
            online_rows.push(item);
        }
    }
    let by_name = |a: &Value, b: &Value| {
        a["name"]
            .as_str()
            .unwrap_or("")
            .to_lowercase()
            .cmp(&b["name"].as_str().unwrap_or("").to_lowercase())
    };
    online_rows.sort_by(by_name);
    offline_rows.sort_by(by_name);

    let mut out = Vec::new();
    if !online_rows.is_empty() {
        out.push(json!({
            "isGroup": true,
            "label": format!("Online — {}", online_rows.len()),
        }));
        out.extend(online_rows);
    }
    if !offline_rows.is_empty() {
        out.push(json!({
            "isGroup": true,
            "label": format!("Offline — {}", offline_rows.len()),
        }));
        out.extend(offline_rows);
    }
    Value::Array(out).to_string()
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 { TABLE[((n >> 6) & 63) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { TABLE[(n & 63) as usize] as char } else { '=' });
    }
    out
}

/// Decodes hidden Profile3y3 JSON (`U+E0033 U+E0079 U+E0033` + ASCII in `U+E0000..U+E007F`)
/// from a user bio, matching Compose's `Profile3y3.decode`.
fn decode_3y3(bio: &str) -> Option<Value> {
    const HEADER: &str = "\u{E0033}\u{E0079}\u{E0033}";
    let start = bio.find(HEADER)? + HEADER.len();
    let mut decoded = String::new();
    for ch in bio[start..].chars() {
        let cp = ch as u32;
        if (0xE0000..=0xE007F).contains(&cp) {
            decoded.push((cp - 0xE0000) as u8 as char);
        } else {
            break;
        }
    }
    if decoded.is_empty() {
        return None;
    }
    serde_json::from_str::<Value>(&decoded).ok()
}

fn strip_3y3(bio: &str) -> String {
    bio.chars()
        .filter(|ch| !(0xE0000..=0xE007F).contains(&(*ch as u32)))
        .collect::<String>()
        .trim_end()
        .to_owned()
}

fn lampcord_badge_data_uri(icon_path: &str) -> Option<String> {
    let bytes: &[u8] = match icon_path {
        "badges/blobcatcozystars.png" => {
            include_bytes!("../../../../shared/src/commonMain/resources/badges/blobcatcozystars.png")
        }
        "badges/lamp.png" => {
            include_bytes!("../../../../shared/src/commonMain/resources/badges/lamp.png")
        }
        "badges/friend.png" => {
            include_bytes!("../../../../shared/src/commonMain/resources/badges/friend.png")
        }
        _ => return None,
    };
    Some(format!("data:image/png;base64,{}", base64_encode(bytes)))
}

fn lampcord_badges_for(user_id: u64, member_roles: &[u64]) -> Vec<Value> {
    static RAW: &str = include_str!("../../../../shared/src/commonMain/resources/badges/badges.json");
    let Ok(mapping) = serde_json::from_str::<Value>(RAW) else {
        return Vec::new();
    };
    let uid = user_id.to_string();
    let mut ids: Vec<String> = Vec::new();
    if let Some(arr) = mapping["users"][&uid].as_array() {
        for v in arr {
            if let Some(s) = v.as_str() {
                if !ids.iter().any(|x| x == s) {
                    ids.push(s.to_owned());
                }
            }
        }
    }
    for rid in member_roles {
        let rstr = rid.to_string();
        if let Some(arr) = mapping["roles"][&rstr].as_array() {
            for v in arr {
                if let Some(s) = v.as_str() {
                    if !ids.iter().any(|x| x == s) {
                        ids.push(s.to_owned());
                    }
                }
            }
        }
    }
    let mut out = Vec::new();
    for id in ids {
        let Some(def) = mapping["badges"].get(&id) else { continue };
        let name = def["name"].as_str().unwrap_or(&id);
        let icon = def["icon"].as_str().unwrap_or("");
        let icon_url = if icon.starts_with("http") {
            icon.to_owned()
        } else if let Some(uri) = lampcord_badge_data_uri(icon) {
            uri
        } else {
            continue;
        };
        out.push(json!({
            "id": id,
            "description": name,
            "icon": icon,
            "iconUrl": icon_url,
            "link": Value::Null,
        }));
    }
    out
}

fn profile_json_str(
    profile: Option<&ProfileSnapshot>,
    members: &[MemberSnapshot],
    presences: &[PresenceSnapshot],
    me: Option<&Me>,
) -> String {
    let Some(p) = profile else {
        return "{}".to_owned();
    };
    let decoded_3y3 = p.bio.as_deref().and_then(decode_3y3);
    let clean_bio = p.bio.as_deref().map(strip_3y3);

    // 3y3 bio tags supply profile colors, banner, or avatar when absent.
    let mut theme_colors = p.theme_colors.clone();
    if let Some(arr) = decoded_3y3.as_ref().and_then(|v| v["theme_colors"].as_array()) {
        let parsed: Vec<u32> = arr.iter().filter_map(|x| x.as_u64().map(|n| (n & 0xFFFFFF) as u32)).collect();
        if !parsed.is_empty() {
            theme_colors = parsed;
        }
    }
    let accent_color = decoded_3y3
        .as_ref()
        .and_then(|v| v["accent_color"].as_u64().map(|n| (n & 0xFFFFFF) as u32))
        .or(p.accent_color);
    let banner_url = decoded_3y3
        .as_ref()
        .and_then(|v| v["banner"].as_str().filter(|s| !s.is_empty()).map(str::to_owned))
        .or_else(|| p.banner_url.clone());
    let avatar = decoded_3y3
        .as_ref()
        .and_then(|v| v["avatar"].as_str().filter(|s| !s.is_empty()).map(str::to_owned))
        .or_else(|| p.avatar.clone());

    let mut badges: Vec<Value> = p
        .badges
        .iter()
        .map(|b| {
            let icon_url = if b.icon.starts_with("http") {
                b.icon.clone()
            } else if b.icon.starts_with("assets/content/") || b.icon.starts_with("badge-icons/") {
                format!("{CDN}/{}", b.icon)
            } else {
                format!("{CDN}/badge-icons/{}.png?size=64", b.icon)
            };
            json!({
                "id": b.id,
                "description": b.description,
                "icon": b.icon,
                "iconUrl": icon_url,
                "link": b.link,
            })
        })
        .collect();
    let member_roles: &[u64] = members
        .iter()
        .find(|m| m.user_id == p.user_id)
        .map(|m| m.roles.as_slice())
        .unwrap_or(&[]);
    badges.extend(lampcord_badges_for(p.user_id, member_roles));

    let own = me.filter(|m| m.id == p.user_id);
    let pres = presences.iter().find(|pr| pr.user_id == p.user_id);
    let status = own
        .map(|m| m.status.as_str())
        .or_else(|| {
            pres.map(|pr| {
                if pr.mobile && pr.status == "online" { "mobile" } else { pr.status.as_str() }
            })
        })
        .unwrap_or("offline");
    let activity = own
        .and_then(|m| m.custom_status.clone())
        .or_else(|| {
            pres.and_then(|pr| pr.custom_status.clone().or_else(|| pr.activities.first().cloned()))
        })
        .unwrap_or_default();

    json!({
        "user_id": p.user_id.to_string(),
        "guild_id": p.guild_id.map(|id| id.to_string()),
        "username": p.username,
        "global_name": p.global_name,
        "nickname": p.nickname,
        "avatar": avatar,
        "banner_url": banner_url,
        "accent_color": accent_color,
        "theme_colors": theme_colors,
        "bio": clean_bio,
        "pronouns": p.pronouns,
        "decoration": p.decoration,
        "badges": badges,
        "status": status,
        "activity": activity,
        "mutual_guild_ids": p.mutual_guild_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
        "mutual_friends_count": p.mutual_friends_count,
        "joined_at": p.joined_at,
    })
    .to_string()
}

fn guild_json(guild: &GuildSnapshot) -> Value {
    let icon = match guild.icon_url.as_deref() {
        Some(url) if url.starts_with("http") => url.to_owned(),
        Some(hash) if !hash.is_empty() => format!("{CDN}/icons/{}/{hash}.png?size=96", guild.id),
        _ => String::new(),
    };
    json!({
        "kind": "guild",
        "id": guild.id.to_string(),
        "name": guild.name,
        "icon": icon,
        "initials": initials(&guild.name),
        "unread": guild.unread,
        "mentions": guild.mention_count,
    })
}

fn guilds_json(guilds: &[GuildSnapshot]) -> String {
    Value::Array(guilds.iter().map(guild_json).collect()).to_string()
}

/// The rail in the Compose order: one entry per settings folder (a lone
/// server in a nameless folder shows as a plain server), with servers that
/// no folder mentions yet (e.g. just joined) on top.
fn rail_json(guilds: &[GuildSnapshot], folders: &[GuildFolderSnapshot]) -> String {
    let by_id: HashMap<u64, &GuildSnapshot> = guilds.iter().map(|guild| (guild.id, guild)).collect();
    let foldered: std::collections::HashSet<u64> =
        folders.iter().flat_map(|folder| folder.guild_ids.iter().copied()).collect();

    let mut items: Vec<Value> = guilds
        .iter()
        .filter(|guild| !foldered.contains(&guild.id))
        .map(guild_json)
        .collect();
    for (index, folder) in folders.iter().enumerate() {
        let members: Vec<Value> = folder
            .guild_ids
            .iter()
            .filter_map(|id| by_id.get(id))
            .map(|guild| guild_json(guild))
            .collect();
        if members.is_empty() {
            continue;
        }
        if folder.id.is_none() && members.len() == 1 {
            items.extend(members);
            continue;
        }
        let key = folder.id.map_or_else(|| format!("index-{index}"), |id| id.to_string());
        let folder_unread = members.iter().any(|g| g["unread"].as_bool().unwrap_or(false));
        let folder_mentions: u64 = members.iter().map(|g| g["mentions"].as_u64().unwrap_or(0)).sum();
        items.push(json!({
            "kind": "folder",
            "id": key,
            "name": folder.name.clone().unwrap_or_default(),
            "color": folder.color.map(|color| format!("#{color:06x}")).unwrap_or_default(),
            "unread": folder_unread,
            "mentions": folder_mentions,
            "guilds": members,
        }));
    }
    Value::Array(items).to_string()
}

/// DM rows, most recent first: name, avatar and unread state.
fn dms_json(
    channels: &[PrivateChannelSnapshot],
    read_states: &[ReadStateSnapshot],
    presences: &[PresenceSnapshot],
) -> String {
    let presence: HashMap<u64, &PresenceSnapshot> =
        presences.iter().map(|presence| (presence.user_id, presence)).collect();
    let reads: HashMap<u64, &ReadStateSnapshot> =
        read_states.iter().map(|state| (state.channel_id, state)).collect();
    let items: Vec<Value> = channels
        .iter()
        .map(|channel| {
            let names: Vec<String> = channel
                .recipients
                .iter()
                .map(|user| user.global_name.clone().unwrap_or_else(|| user.username.clone()))
                .collect();
            let name = channel.name.clone().unwrap_or_else(|| {
                if names.is_empty() { "Unnamed".to_owned() } else { names.join(", ") }
            });
            let group = channel.kind == 3;
            let avatar = match (group, channel.recipients.first()) {
                (false, Some(user)) => {
                    let url = avatar_url(user.id, user.avatar.as_deref());
                    if url.is_empty() { default_avatar_url(user.id) } else { url }
                }
                _ => String::new(),
            };
            let read = reads.get(&channel.id);
            let recipient = channel.recipients.first().filter(|_| !group);
            let is_system = recipient.is_some_and(|u| u.system);
            let is_bot = recipient.is_some_and(|u| u.bot);
            let status = recipient
                .and_then(|user| presence.get(&user.id))
                .map(|presence| {
                    if presence.mobile && presence.status == "online" { "mobile" } else { presence.status.as_str() }
                })
                .unwrap_or("offline");
            json!({
                "id": channel.id.to_string(),
                "recipientId": recipient.map(|user| user.id.to_string()).unwrap_or_default(),
                "name": name,
                "group": group,
                "system": is_system,
                "bot": is_bot,
                "avatar": avatar,
                "status": if group || is_system { "" } else { status },
                "decoration": decoration_url(recipient.and_then(|user| user.decoration.as_deref())),
                "nameplate": nameplate_url(recipient.and_then(|user| user.nameplate.as_deref())),
                "members": channel.recipients.len() + 1,
                "unread": read.is_some_and(|state| state.unread),
                "mentions": read.map_or(0, |state| state.mention_count),
            })
        })
        .collect();
    Value::Array(items).to_string()
}

/// Matches the Compose monogram: first letter of each word, at most three.
fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(3)
        .collect()
}

fn is_listed_kind(kind: u8) -> bool {
    matches!(
        kind,
        CHANNEL_TEXT
            | CHANNEL_VOICE
            | CHANNEL_ANNOUNCEMENT
            | CHANNEL_STAGE
            | CHANNEL_FORUM
    )
}

/// Flattens the guild's channels into the order the sidebar draws them:
/// uncategorised channels first, then each category followed by its
/// children, everything by position with voice channels after text ones.
fn is_emoji_char(ch: char) -> bool {
    let cp = ch as u32;
    matches!(
        cp,
        0x231A..=0x231B
            | 0x23E9..=0x23FA
            | 0x25AA..=0x25AB
            | 0x25B6
            | 0x25C0
            | 0x25FB..=0x25FE
            | 0x2600..=0x27BF
            | 0x2934..=0x2935
            | 0x2B05..=0x2B07
            | 0x2B1B..=0x2B1C
            | 0x2B50
            | 0x2B55
            | 0x3030
            | 0x303D
            | 0x3297
            | 0x3299
            | 0xFE0E..=0xFE0F
            | 0x200D
            | 0x20E3
            | 0x1F000..=0x1FAFF
    )
}

fn clean_channel_name(
    raw: &str,
    is_category: bool,
    props: &std::collections::BTreeMap<String, String>,
) -> String {
    let remove_emojis = props.get("clean_channels_remove_emojis").map_or(true, |v| v == "true");
    let hide_symbols = props.get("clean_channels_hide_symbols").map_or(true, |v| v == "true");
    let normalize_letters = props.get("clean_channels_normalize_letters").map_or(true, |v| v == "true");
    let cap_categories = props.get("clean_channels_capitalize_categories").map_or(true, |v| v == "true");

    let mut s = raw.to_owned();
    if remove_emojis {
        s = s.chars().filter(|&c| !is_emoji_char(c)).collect();
    }
    if normalize_letters {
        // Normalize common fancy mathematical/fullwidth/accented letters to ASCII.
        s = s
            .chars()
            .map(|c| {
                let cp = c as u32;
                match cp {
                    0xFF21..=0xFF3A => char::from_u32(cp - 0xFF21 + b'A' as u32).unwrap_or(c),
                    0xFF41..=0xFF5A => char::from_u32(cp - 0xFF41 + b'a' as u32).unwrap_or(c),
                    0xFF10..=0xFF19 => char::from_u32(cp - 0xFF10 + b'0' as u32).unwrap_or(c),
                    0x1D400..=0x1D419 => char::from_u32(cp - 0x1D400 + b'A' as u32).unwrap_or(c),
                    0x1D41A..=0x1D433 => char::from_u32(cp - 0x1D41A + b'a' as u32).unwrap_or(c),
                    0x1D5A0..=0x1D5B9 => char::from_u32(cp - 0x1D5A0 + b'A' as u32).unwrap_or(c),
                    0x1D5BA..=0x1D5D3 => char::from_u32(cp - 0x1D5BA + b'a' as u32).unwrap_or(c),
                    _ => c,
                }
            })
            .collect();
    }
    if hide_symbols {
        s = s
            .chars()
            .filter(|&c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '#'))
            .collect();
    }
    let cleaned = s.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out = if cleaned.is_empty() { raw.to_owned() } else { cleaned };
    if is_category && cap_categories {
        let mut chars = out.chars();
        if let Some(first) = chars.next() {
            out = first.to_uppercase().to_string() + &chars.as_str().to_lowercase();
        }
    }
    out
}

fn channels_json(channels: &[ChannelSnapshot], read_states: &[ReadStateSnapshot]) -> String {
    let props = load_lampcord_properties();
    let reads: HashMap<u64, &ReadStateSnapshot> =
        read_states.iter().map(|state| (state.channel_id, state)).collect();
    let sort_key = |channel: &&ChannelSnapshot| {
        let voice = matches!(channel.kind, CHANNEL_VOICE | CHANNEL_STAGE);
        (voice, channel.position.unwrap_or(i32::MAX), channel.id)
    };
    let row = |channel: &ChannelSnapshot| {
        let read = reads.get(&channel.id);
        json!({
            "id": channel.id.to_string(),
            "guildId": channel.guild_id.map(|id| id.to_string()).unwrap_or_default(),
            "name": clean_channel_name(&channel.name, false, &props),
            "kind": channel.kind,
            "topic": channel.topic.clone().unwrap_or_default(),
            "nsfw": channel.nsfw,
            "locked": !channel.can_view,
            "muted": channel.muted,
            "unobtrusive": channel.unobtrusive_unread,
            "category": false,
            "unread": read.is_some_and(|state| state.unread),
            "mentions": read.map_or(0, |state| state.mention_count),
        })
    };

    let mut categories: Vec<&ChannelSnapshot> = channels
        .iter()
        .filter(|channel| channel.kind == CHANNEL_CATEGORY)
        .collect();
    categories.sort_by_key(|channel| (channel.position.unwrap_or(i32::MAX), channel.id));
    let category_ids: Vec<u64> = categories.iter().map(|channel| channel.id).collect();

    let mut items = Vec::new();
    let mut loose: Vec<&ChannelSnapshot> = channels
        .iter()
        .filter(|channel| is_listed_kind(channel.kind))
        .filter(|channel| channel.parent_id.is_none_or(|parent| !category_ids.contains(&parent)))
        .collect();
    loose.sort_by_key(sort_key);
    items.extend(loose.into_iter().map(row));

    for category in categories {
        let mut children: Vec<&ChannelSnapshot> = channels
            .iter()
            .filter(|channel| is_listed_kind(channel.kind) && channel.parent_id == Some(category.id))
            .collect();
        if children.is_empty() {
            continue;
        }
        children.sort_by_key(sort_key);
        items.push(json!({
            "id": category.id.to_string(),
            "name": clean_channel_name(&category.name, true, &props),
            "kind": CHANNEL_CATEGORY,
            "category": true,
            "unread": false,
            "mentions": 0,
        }));
        items.extend(children.into_iter().map(row));
    }
    Value::Array(items).to_string()
}

/// What Discord shows for system messages as `(prefix, suffix)` around the
/// author's bold name. `None` for ordinary messages.
fn system_parts(kind: u8, id: u64) -> Option<(&'static str, &'static str)> {
    Some(match kind {
        1 => ("", " added a recipient."),
        2 => ("", " removed a recipient."),
        3 => ("", " started a call."),
        4 => ("", " changed the channel name."),
        5 => ("", " changed the channel icon."),
        6 => ("", " pinned a message to this channel."),
        7 => {
            const TEMPLATES: [(&str, &str); 13] = [
                ("", " joined the party."),
                ("", " is here."),
                ("Welcome, ", ". We hope you brought pizza."),
                ("A wild ", " appeared."),
                ("", " just landed."),
                ("", " just slid into the server."),
                ("", " just showed up!"),
                ("Welcome ", ". Say hi!"),
                ("", " hopped into the server."),
                ("Everyone welcome ", "!"),
                ("Glad you're here, ", "."),
                ("Good to see you, ", "."),
                ("Yay you made it, ", "!"),
            ];
            let idx = (snowflake_ms(id) % TEMPLATES.len() as u64) as usize;
            TEMPLATES[idx]
        }
        8 => ("", " just boosted the server!"),
        9..=11 => ("", " just boosted the server to a new level!"),
        12 => ("", " added a followed channel to this channel."),
        18 => ("", " started a thread."),
        _ => return None,
    })
}

fn is_image(attachment: &lampcord_ffi::AttachmentSnapshot) -> bool {
    attachment
        .content_type
        .as_deref()
        .map_or_else(
            || {
                let name = attachment.filename.to_lowercase();
                [".png", ".jpg", ".jpeg", ".gif", ".webp"].iter().any(|ext| name.ends_with(ext))
            },
            |kind| kind.starts_with("image/"),
        )
}

fn is_video(attachment: &lampcord_ffi::AttachmentSnapshot) -> bool {
    attachment
        .content_type
        .as_deref()
        .map_or_else(
            || {
                let name = attachment.filename.to_lowercase();
                [".mp4", ".webm", ".mov", ".mkv"].iter().any(|ext| name.ends_with(ext))
            },
            |kind| kind.starts_with("video/"),
        )
}

fn is_audio(attachment: &lampcord_ffi::AttachmentSnapshot) -> bool {
    attachment
        .content_type
        .as_deref()
        .map_or_else(
            || {
                let name = attachment.filename.to_lowercase();
                [".mp3", ".ogg", ".wav", ".flac", ".m4a", ".opus"].iter().any(|ext| name.ends_with(ext))
            },
            |kind| kind.starts_with("audio/"),
        )
}

fn date_label_for_ms(ms: u64) -> String {
    let days = (ms / 86_400_000) as i64;
    let (year, month, day) = crate::markdown::civil_from_days(days);
    let month_name = match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        _ => "December",
    };
    format!("{month_name} {day}, {year}")
}

fn relationships_json(
    relationships: &[RelationshipSnapshot],
    presences: &[PresenceSnapshot],
) -> String {
    let presence: HashMap<u64, &PresenceSnapshot> =
        presences.iter().map(|p| (p.user_id, p)).collect();
    let rows: Vec<Value> = relationships
        .iter()
        .map(|rel| {
            let url = avatar_url(rel.user_id, rel.avatar.as_deref());
            let status = presence
                .get(&rel.user_id)
                .map(|p| if p.mobile && p.status == "online" { "mobile" } else { p.status.as_str() })
                .unwrap_or("offline");
            let activity = presence
                .get(&rel.user_id)
                .and_then(|p| p.custom_status.clone().or_else(|| p.activities.first().cloned()))
                .unwrap_or_default();
            let display = rel
                .global_name
                .clone()
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| rel.username.clone());
            json!({
                "id": rel.user_id.to_string(),
                "kind": rel.kind,
                "username": rel.username,
                "name": display,
                "avatar": if url.is_empty() { default_avatar_url(rel.user_id) } else { url },
                "decoration": decoration_url(rel.avatar_decoration.as_deref()),
                "status": status,
                "activity": activity,
            })
        })
        .collect();
    Value::Array(rows).to_string()
}

/// Message rows for the chat list. Kept flat (attachments/reactions/stickers as
/// JSON strings, reply fields inlined) because they feed a QML ListModel.
fn messages_json(
    messages: &[MessageSnapshot],
    members: &[MemberSnapshot],
    deleted_ids: &std::collections::HashSet<u64>,
    me: Option<&Me>,
) -> String {
    let members: HashMap<u64, &MemberSnapshot> =
        members.iter().map(|member| (member.user_id, member)).collect();
    let my_roles: &[u64] = me
        .and_then(|m| members.get(&m.id))
        .map(|m| m.roles.as_slice())
        .unwrap_or(&[]);
    let user_avatar = |id: u64, hash: Option<&str>| {
        let url = avatar_url(id, hash);
        if url.is_empty() { default_avatar_url(id) } else { url }
    };
    let mut previous: Option<&MessageSnapshot> = None;
    let mut previous_day: Option<u64> = None;
    let items: Vec<Value> = messages
        .iter()
        .map(|message| {
            let time = snowflake_ms(message.id);
            let day = time / 86_400_000;
            let show_date = previous_day != Some(day);
            previous_day = Some(day);
            let system = system_parts(message.kind, message.id);
            // Replies, system rows, and new days always start a new group.
            let grouped = !show_date
                && message.reply.is_none()
                && system.is_none()
                && previous.is_some_and(|prev| {
                    prev.author_id == message.author_id
                        && system_parts(prev.kind, prev.id).is_none()
                        && time.saturating_sub(snowflake_ms(prev.id)) < GROUP_WINDOW_MS
                });
            previous = Some(message);
            let member = message.author_id.and_then(|id| members.get(&id));
            let name = member
                .and_then(|member| member.nickname.clone())
                .filter(|nick| !nick.is_empty())
                .unwrap_or_else(|| message.author_name.clone());
            let avatar = message
                .author_id
                .map(|id| user_avatar(id, message.author_avatar.as_deref()))
                .unwrap_or_default();
            let attachments: Vec<Value> = message
                .attachments
                .iter()
                .map(|attachment| {
                    json!({
                        "url": attachment.url,
                        "preview": attachment.proxy_url.clone().unwrap_or_else(|| attachment.url.clone()),
                        "filename": attachment.filename,
                        "size": attachment.size,
                        "image": is_image(attachment),
                        "video": is_video(attachment),
                        "audio": is_audio(attachment),
                        "width": attachment.width.unwrap_or(0),
                        "height": attachment.height.unwrap_or(0),
                    })
                })
                .collect();
            let reactions: Vec<Value> = message
                .reactions
                .iter()
                .map(|r| {
                    let api_key = match &r.emoji_id {
                        Some(id) if !id.is_empty() => format!("{}:{id}", r.emoji_name),
                        _ => r.emoji_name.clone(),
                    };
                    let icon = r
                        .emoji_id
                        .as_deref()
                        .filter(|id| !id.is_empty())
                        .map(|id| {
                            let ext = if r.animated { "gif" } else { "webp" };
                            format!("{CDN}/emojis/{id}.{ext}?size=44")
                        })
                        .unwrap_or_else(|| crate::emoji::twemoji_url(&r.emoji_name));
                    json!({
                        "key": api_key,
                        "name": r.emoji_name,
                        "icon": icon,
                        "count": r.count,
                        "me": r.me,
                    })
                })
                .collect();
            let stickers: Vec<Value> = message
                .stickers
                .iter()
                .map(|s| {
                    let ext = if s.format_type == 4 { "gif" } else { "png" };
                    json!({
                        "id": s.id.to_string(),
                        "name": s.name,
                        "url": format!("https://media.discordapp.net/stickers/{}.{ext}?size=160", s.id),
                    })
                })
                .collect();
            let reply = message.reply.as_ref();
            let mentioned = me.is_some_and(|m| {
                let c = &message.content;
                c.contains("@everyone")
                    || c.contains("@here")
                    || c.contains(&format!("<@{}>", m.id))
                    || c.contains(&format!("<@!{}>", m.id))
                    || my_roles.iter().any(|rid| c.contains(&format!("<@&{rid}>")))
            });
            let clan_tag = message
                .author_clan_tag
                .as_deref()
                .or_else(|| member.and_then(|m| m.clan_tag.as_deref()))
                .unwrap_or("");
            let clan_badge = message
                .author_clan_badge
                .as_deref()
                .or_else(|| member.and_then(|m| m.clan_badge.as_deref()))
                .unwrap_or("");
            json!({
                "id": message.id.to_string(),
                "channelId": message.channel_id.to_string(),
                "authorId": message.author_id.map(|id| id.to_string()).unwrap_or_default(),
                "author": name,
                "avatar": avatar,
                "decoration": decoration_url(message.author_decoration.as_deref()),
                "clanTag": clan_tag,
                "clanBadge": clan_badge,
                "content": message.content,
                "time": time,
                "edited": message.edited_timestamp.is_some(),
                "pinned": message.pinned,
                "deleted": deleted_ids.contains(&message.id),
                "mentioned": mentioned,
                "grouped": grouped,
                "showDate": show_date,
                "dateLabel": date_label_for_ms(time),
                "systemPrefix": system.map(|(pre, _)| pre).unwrap_or(""),
                "system": system.map(|(_, suf)| suf).unwrap_or(""),
                "kind": message.kind,
                "attachments": Value::Array(attachments).to_string(),
                "embeds": if message.embeds_json.is_empty() { "[]".to_owned() } else { message.embeds_json.clone() },
                "forwarded": if message.forwarded_json.is_empty() { "[]".to_owned() } else { message.forwarded_json.clone() },
                "reactions": Value::Array(reactions).to_string(),
                "stickers": Value::Array(stickers).to_string(),
                "replyAuthor": reply.map(|reply| reply.author_name.clone()).unwrap_or_default(),
                "replyAvatar": reply.map(|reply| user_avatar(reply.author_id, reply.author_avatar.as_deref())).unwrap_or_default(),
                "replyContent": reply.map(|reply| reply.content.clone()).unwrap_or_default(),
            })
        })
        .collect();
    Value::Array(items).to_string()
}

impl qobject::LampcordBackend {
    pub fn poll(mut self: Pin<&mut Self>) {
        // Drain typed events for local message deletion logging and desktop/toast notifications.
        let events = self.client.drain_events();
        let mut deleted_dirty = false;
        for env in events {
            match env.event {
                ClientEvent::MessageDeleted { channel_id, message_id } => {
                    if *self.log_deleted_messages() {
                        if let Some(msg) = self.messages.iter().find(|m| m.id == message_id && m.channel_id == channel_id).cloned() {
                            let mut rust = self.as_mut().rust_mut();
                            let entry = rust.deleted_messages.entry(channel_id).or_default();
                            if !entry.iter().any(|m| m.id == message_id) {
                                entry.push(msg);
                                deleted_dirty = true;
                            }
                        }
                    }
                }
                ClientEvent::NotificationReceived(msg) => {
                    let title = msg.author_name.clone();
                    let body = if msg.content.is_empty() && !msg.attachments.is_empty() {
                        format!("Sent {} attachment(s)", msg.attachments.len())
                    } else {
                        msg.content.clone()
                    };
                    let channel_id = msg.channel_id.to_string();
                    self.as_mut().set_toast_title(QString::from(title.as_str()));
                    self.as_mut().set_toast_body(QString::from(body.as_str()));
                    self.as_mut().set_toast_channel_id(QString::from(channel_id.as_str()));
                    let seq = *self.toast_seq() + 1;
                    self.as_mut().set_toast_seq(seq);
                    if *self.desktop_notifications() {
                        #[cfg(target_os = "linux")]
                        {
                            let _ = std::process::Command::new("notify-send")
                                .args(["--app-name=Lampcord", &title, &body])
                                .spawn();
                        }
                    }
                }
                _ => {}
            }
        }

        let since = self.versions.clone();
        let patch = self.client.snapshot_patch(since);
        if patch.versions == self.versions && !deleted_dirty {
            return;
        }
        self.as_mut().rust_mut().versions = patch.versions.clone();

        let mut members_dirty = false;
        let mut relationships_dirty = false;
        if let Some(meta) = patch.meta {
            let connection = QString::from(connection_name(&meta.connection));
            self.as_mut().set_connection(connection);
            self.as_mut().set_has_token(meta.has_token);
            self.as_mut()
                .set_current_user(QString::from(meta.current_user.as_deref().unwrap_or("")));
            self.as_mut().set_current_user_username(QString::from(
                meta.current_user_username.as_deref().unwrap_or(""),
            ));
            self.as_mut().set_current_user_email(QString::from(
                meta.current_user_email.as_deref().unwrap_or(""),
            ));
            self.as_mut().set_current_user_mfa_enabled(meta.current_user_mfa_enabled);
            self.as_mut().set_current_user_id(id_string(meta.current_user_id));
            let avatar = meta.current_user_id.map_or_else(String::new, |id| {
                let url = avatar_url(id, meta.current_user_avatar.as_deref());
                if url.is_empty() { default_avatar_url(id) } else { url }
            });
            self.as_mut().set_current_user_avatar(QString::from(avatar.as_str()));
            let status = meta.current_user_status.clone().unwrap_or_else(|| "online".to_owned());
            self.as_mut().set_current_user_status(QString::from(status.as_str()));
            self.as_mut().set_current_user_custom_status(QString::from(
                meta.current_user_custom_status.as_deref().unwrap_or(""),
            ));
            self.as_mut().set_current_user_decoration(QString::from(
                decoration_url(meta.current_user_decoration.as_deref()).as_str(),
            ));
            self.as_mut().set_current_user_nameplate(QString::from(
                nameplate_url(meta.current_user_nameplate.as_deref()).as_str(),
            ));
            let mut profile_dirty = false;
            if meta.profile != self.latest_profile {
                self.as_mut().rust_mut().latest_profile = meta.profile;
                profile_dirty = true;
            }
            let accounts = serde_json::to_string(&meta.saved_accounts).unwrap_or_else(|_| "[]".to_owned());
            self.as_mut().set_saved_accounts_json(QString::from(accounts.as_str()));
            let me = meta.current_user_id.map(|id| Me {
                id,
                name: meta.current_user.clone().unwrap_or_default(),
                avatar: meta.current_user_avatar.clone(),
                decoration: meta.current_user_decoration.clone(),
                nameplate: meta.current_user_nameplate.clone(),
                status,
                custom_status: meta.current_user_custom_status.clone(),
            });
            if me != self.me
                || meta.selected_channel_id != self.selected_channel
                || meta.selected_guild_id != self.selected_guild
            {
                let mut rust = self.as_mut().rust_mut();
                rust.me = me;
                rust.selected_guild = meta.selected_guild_id;
                rust.selected_channel = meta.selected_channel_id;
                members_dirty = true;
                profile_dirty = true;
            }
            if profile_dirty {
                let profile = profile_json_str(
                    self.latest_profile.as_ref(),
                    &self.members,
                    &self.presences,
                    self.me.as_ref(),
                );
                self.as_mut().set_profile_json(QString::from(profile.as_str()));
            }
            self.as_mut().set_selected_guild_id(id_string(meta.selected_guild_id));
            self.as_mut().set_selected_channel_id(id_string(meta.selected_channel_id));
            let error = match (&meta.connection, &meta.error) {
                (ConnectionState::Error { message }, _) => message.clone(),
                (_, Some(error)) => error.clone(),
                _ => String::new(),
            };
            self.as_mut().set_error_text(QString::from(error.as_str()));
            self.as_mut().apply_auth(meta.auth.challenge);
            self.as_mut().apply_remote_auth(meta.auth.remote);
        }

        if patch.relationships != self.relationships {
            self.as_mut().rust_mut().relationships = patch.relationships;
            relationships_dirty = true;
        }

        let mut rail_dirty = false;
        if let Some(guilds) = patch.guilds {
            self.as_mut().set_guilds_json(QString::from(guilds_json(&guilds).as_str()));
            self.as_mut().rust_mut().guilds = guilds;
            rail_dirty = true;
            members_dirty = true;
        }
        if let Some(folders) = patch.guild_folders {
            self.as_mut().rust_mut().guild_folders = folders;
            rail_dirty = true;
        }
        if let Some(emojis) = patch.guild_emojis {
            let rows: Vec<Value> = emojis
                .iter()
                .map(|e| {
                    let ext = if e.animated { "gif" } else { "webp" };
                    json!({
                        "id": e.id.to_string(),
                        "name": e.name,
                        "animated": e.animated,
                        "url": format!("{CDN}/emojis/{}.{ext}?size=48", e.id),
                        "token": if e.animated { format!("<a:{}:{}>", e.name, e.id) } else { format!("<:{}:{}>", e.name, e.id) },
                        "key": format!("{}:{}", e.name, e.id),
                    })
                })
                .collect();
            self.as_mut().set_guild_emojis_json(QString::from(Value::Array(rows).to_string().as_str()));
        }
        if let Some(roles) = patch.guild_roles {
            let rows: Vec<Value> = roles
                .iter()
                .map(|r| {
                    json!({
                        "id": r.id.to_string(),
                        "name": r.name,
                        "color": if r.color == 0 { String::new() } else { format!("#{:06x}", r.color) },
                        "position": r.position,
                    })
                })
                .collect();
            self.as_mut().rust_mut().guild_roles = roles;
            members_dirty = true;
            self.as_mut().set_guild_roles_json(QString::from(Value::Array(rows).to_string().as_str()));
        }
        if let Some(bans) = patch.guild_bans {
            let rows: Vec<Value> = bans
                .iter()
                .map(|b| {
                    let url = avatar_url(b.user_id, b.avatar.as_deref());
                    json!({
                        "id": b.user_id.to_string(),
                        "username": b.username,
                        "avatar": if url.is_empty() { default_avatar_url(b.user_id) } else { url },
                        "reason": b.reason.clone().unwrap_or_default(),
                    })
                })
                .collect();
            self.as_mut().set_guild_bans_json(QString::from(Value::Array(rows).to_string().as_str()));
        }
        if let Some(threads) = patch.threads {
            let rows: Vec<Value> = threads
                .iter()
                .map(|t| {
                    json!({
                        "id": t.id.to_string(),
                        "parentId": t.parent_id.map(|id| id.to_string()).unwrap_or_default(),
                        "name": t.name,
                        "messageCount": t.message_count,
                        "memberCount": t.member_count,
                        "archived": t.archived,
                    })
                })
                .collect();
            self.as_mut().set_threads_json(QString::from(Value::Array(rows).to_string().as_str()));
        }
        if rail_dirty {
            let json = rail_json(&self.guilds, &self.guild_folders);
            self.as_mut().set_rail_json(QString::from(json.as_str()));
        }

        let mut channels_dirty = false;
        let mut messages_dirty = deleted_dirty;
        let mut mentions_dirty = false;
        let mut dms_dirty = false;
        if let Some(private_channels) = patch.private_channels {
            self.as_mut().rust_mut().private_channels = private_channels;
            dms_dirty = true;
            messages_dirty = true;
            members_dirty = true;
        }
        if let Some(presences) = patch.presences {
            self.as_mut().rust_mut().presences = presences;
            members_dirty = true;
            dms_dirty = true;
            relationships_dirty = true;
        }
        if let Some(read_states) = patch.read_states {
            self.as_mut().rust_mut().read_states = read_states;
            channels_dirty = true;
            dms_dirty = true;
        }
        if let Some(channels) = patch.channels {
            self.as_mut().rust_mut().channels = channels;
            channels_dirty = true;
        }
        if let Some(members) = patch.members {
            self.as_mut().rust_mut().members = members;
            messages_dirty = true;
            mentions_dirty = true;
            members_dirty = true;
        }
        if let Some(member_list) = patch.member_list {
            self.as_mut().rust_mut().member_list = member_list;
            members_dirty = true;
        }
        if let Some(messages) = patch.messages {
            // If deletion logger is enabled, detect messages that disappeared from the same channel.
            if *self.log_deleted_messages() && !self.messages.is_empty() {
                if let (Some(old_cid), Some(new_cid)) = (
                    self.messages.first().map(|m| m.channel_id),
                    messages.first().map(|m| m.channel_id),
                ) {
                    if old_cid == new_cid {
                        let new_ids: std::collections::HashSet<u64> = messages.iter().map(|m| m.id).collect();
                        let gone: Vec<MessageSnapshot> = self
                            .messages
                            .iter()
                            .filter(|m| !new_ids.contains(&m.id))
                            .cloned()
                            .collect();
                        if !gone.is_empty() {
                            let mut rust = self.as_mut().rust_mut();
                            let entry = rust.deleted_messages.entry(old_cid).or_default();
                            for msg in gone {
                                if !entry.iter().any(|e| e.id == msg.id) {
                                    entry.push(msg);
                                }
                            }
                        }
                    }
                }
            }
            self.as_mut().rust_mut().messages = messages;
            messages_dirty = true;
        }
        let empty_deleted = std::collections::HashSet::new();
        if let Some(pins) = patch.pinned_messages {
            // Newest pin first; each one stands alone, so nothing groups.
            let mut rows: Vec<Value> = serde_json::from_str(&messages_json(&pins, &self.members, &empty_deleted, self.me.as_ref())).unwrap_or_default();
            rows.reverse();
            for row in &mut rows {
                row["grouped"] = Value::Bool(false);
                row["showDate"] = Value::Bool(false);
            }
            self.as_mut().set_pins_json(QString::from(Value::Array(rows).to_string().as_str()));
        }
        if let Some(results) = patch.search_results {
            let mut rows: Vec<Value> = serde_json::from_str(&messages_json(&results, &self.members, &empty_deleted, self.me.as_ref())).unwrap_or_default();
            for row in &mut rows {
                row["grouped"] = Value::Bool(false);
                row["showDate"] = Value::Bool(false);
            }
            self.as_mut().set_search_results_json(QString::from(Value::Array(rows).to_string().as_str()));
        }
        if let Some(mentions) = patch.mentions {
            self.as_mut().rust_mut().mentions = mentions;
            mentions_dirty = true;
        }

        if channels_dirty {
            let json = channels_json(&self.channels, &self.read_states);
            self.as_mut().set_channels_json(QString::from(json.as_str()));
        }
        if members_dirty {
            let selected_guild = self
                .selected_guild
                .and_then(|gid| self.guilds.iter().find(|g| g.id == gid));
            let json = members_json(
                selected_guild,
                self.selected_channel,
                &self.private_channels,
                &self.members,
                &self.member_list,
                &self.guild_roles,
                &self.presences,
                self.me.as_ref(),
            );
            self.as_mut().set_members_json(QString::from(json.as_str()));
            if self.latest_profile.is_some() {
                let profile = profile_json_str(
                    self.latest_profile.as_ref(),
                    &self.members,
                    &self.presences,
                    self.me.as_ref(),
                );
                self.as_mut().set_profile_json(QString::from(profile.as_str()));
            }
        }
        if relationships_dirty {
            let json = relationships_json(&self.relationships, &self.presences);
            self.as_mut().set_relationships_json(QString::from(json.as_str()));
        }
        if mentions_dirty {
            let json = messages_json(&self.mentions, &self.members, &empty_deleted, self.me.as_ref());
            self.as_mut().set_mentions_json(QString::from(json.as_str()));
        }
        if dms_dirty {
            let json = dms_json(&self.private_channels, &self.read_states, &self.presences);
            self.as_mut().set_dms_json(QString::from(json.as_str()));
        }
        if messages_dirty {
            let mut combined = self.messages.clone();
            let mut deleted_ids = std::collections::HashSet::new();
            if let Some(cid) = self.selected_channel {
                if !self.loaded_db_channels.contains(&cid) {
                    let cached = load_logged_deleted_messages(cid);
                    let mut rust = self.as_mut().rust_mut();
                    rust.loaded_db_channels.insert(cid);
                    if !cached.is_empty() {
                        let entry = rust.deleted_messages.entry(cid).or_default();
                        for msg in cached {
                            if !entry.iter().any(|e| e.id == msg.id) {
                                entry.push(msg);
                            }
                        }
                    }
                }
                if let Some(logged) = self.deleted_messages.get(&cid) {
                    for msg in logged {
                        deleted_ids.insert(msg.id);
                        if !combined.iter().any(|m| m.id == msg.id) {
                            combined.push(msg.clone());
                        }
                    }
                    combined.sort_by_key(|m| m.id);
                }
            }
            let json = messages_json(&combined, &self.members, &deleted_ids, self.me.as_ref());
            self.as_mut().set_messages_json(QString::from(json.as_str()));
        }
    }

    fn apply_auth(mut self: Pin<&mut Self>, challenge: Option<AuthChallenge>) {
        let (state, message) = match challenge {
            None => return,
            Some(AuthChallenge::Success { .. }) => ("success", String::new()),
            Some(AuthChallenge::MfaRequired { ticket, methods }) => {
                let method = if methods.iter().any(|method| method == "totp") {
                    "totp".to_owned()
                } else {
                    methods.first().cloned().unwrap_or_else(|| "totp".to_owned())
                };
                let mut rust = self.as_mut().rust_mut();
                rust.mfa_ticket = Some(ticket);
                rust.mfa_method = method;
                ("mfa", String::new())
            }
            Some(AuthChallenge::CaptchaRequired { .. }) => (
                "failed",
                "Discord wants a captcha for this login, which this build can't show yet. Log in with a token instead.".to_owned(),
            ),
            Some(AuthChallenge::PasswordUpdateRequired) => (
                "failed",
                "Discord requires a password change. Update it on discord.com, then log in again.".to_owned(),
            ),
            Some(AuthChallenge::SmsSent { phone }) => ("mfa", format!("Code sent to {phone}")),
            Some(AuthChallenge::ResetSent { .. }) => {
                ("idle", "Check your email for a reset link.".to_owned())
            }
            Some(AuthChallenge::Failed { message, .. }) => ("failed", message),
        };
        self.as_mut().set_auth_state(QString::from(state));
        self.as_mut().set_auth_message(QString::from(message.as_str()));
    }

    fn apply_remote_auth(mut self: Pin<&mut Self>, remote: RemoteAuthState) {
        let state = match remote {
            RemoteAuthState::Idle => "idle",
            RemoteAuthState::Connecting => "connecting",
            RemoteAuthState::QrReady { url } => {
                let (modules, size) = qr_modules(&url);
                self.as_mut().set_qr_modules(QString::from(modules.as_str()));
                self.as_mut().set_qr_size(size);
                "ready"
            }
            RemoteAuthState::UserScanned { username, .. } => {
                self.as_mut().set_qr_user(QString::from(username.as_str()));
                "scanned"
            }
            RemoteAuthState::Finished { .. } => "finished",
            RemoteAuthState::Error { .. } => "error",
        };
        self.as_mut().set_qr_state(QString::from(state));
    }

    fn begin_auth(mut self: Pin<&mut Self>) {
        self.as_mut().set_auth_state(QString::from("pending"));
        self.as_mut().set_auth_message(QString::default());
    }

    fn fail_auth(mut self: Pin<&mut Self>, message: String) {
        self.as_mut().set_auth_state(QString::from("failed"));
        self.as_mut().set_auth_message(QString::from(message.as_str()));
    }

    pub fn login_token(mut self: Pin<&mut Self>, token: QString) {
        let token = token.to_string().trim().trim_matches('"').to_owned();
        self.as_mut().begin_auth();
        if let Err(error) = self.client.connect_token(token) {
            self.fail_auth(error.to_string());
        }
    }

    pub fn login_password(mut self: Pin<&mut Self>, login: QString, password: QString) {
        self.as_mut().begin_auth();
        let result = self
            .client
            .login_password(login.to_string().trim().to_owned(), password.to_string(), None);
        if let Err(error) = result {
            self.fail_auth(error.to_string());
        }
    }

    pub fn submit_mfa(mut self: Pin<&mut Self>, code: QString) {
        let Some(ticket) = self.mfa_ticket.clone() else {
            self.fail_auth("The login session expired. Start again.".to_owned());
            return;
        };
        let method = self.mfa_method.clone();
        self.as_mut().begin_auth();
        let code = code.to_string().trim().to_owned();
        if let Err(error) = self.client.submit_mfa(code, ticket, method) {
            self.fail_auth(error.to_string());
        }
    }

    pub fn send_mfa_sms(mut self: Pin<&mut Self>) {
        let Some(ticket) = self.mfa_ticket.clone() else {
            self.fail_auth("The login session expired. Start again.".to_owned());
            return;
        };
        // Choosing SMS changes the verification endpoint, not just the code
        // delivery. Keep the submit method in sync so the next code is checked
        // against /auth/mfa/sms.
        self.as_mut().rust_mut().mfa_method = "sms".to_owned();
        self.as_mut().begin_auth();
        if let Err(error) = self.client.send_mfa_sms(ticket) {
            self.fail_auth(error.to_string());
        }
    }

    pub fn forgot_password(mut self: Pin<&mut Self>, login: QString) {
        let login = login.to_string().trim().to_owned();
        if login.is_empty() {
            self.fail_auth("Enter your email or phone number first.".to_owned());
            return;
        }
        self.as_mut().begin_auth();
        if let Err(error) = self.client.forgot_password(login) {
            self.fail_auth(error.to_string());
        }
    }

    pub fn fetch_profile(self: Pin<&mut Self>, user_id: QString) {
        if let Some(user_id) = parse_id(&user_id) {
            let _ = self.client.fetch_profile(user_id, parse_id(self.selected_guild_id()));
        }
    }

    pub fn reset_auth(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().mfa_ticket = None;
        self.as_mut().set_auth_state(QString::from("idle"));
        self.as_mut().set_auth_message(QString::default());
    }

    pub fn start_qr_login(self: Pin<&mut Self>) {
        let _ = self.client.start_remote_auth();
    }

    pub fn cancel_qr_login(self: Pin<&mut Self>) {
        let _ = self.client.cancel_remote_auth();
    }

    pub fn select_guild(self: Pin<&mut Self>, guild_id: QString) {
        let _ = self.client.select_guild(parse_id(&guild_id));
    }

    pub fn select_channel(self: Pin<&mut Self>, channel_id: QString) {
        let _ = self.client.select_channel(parse_id(&channel_id));
    }

    pub fn fetch_pins(self: Pin<&mut Self>) {
        if let Some(channel_id) = parse_id(self.selected_channel_id()) {
            let _ = self.client.fetch_pins(channel_id);
        }
    }

    pub fn fetch_mentions(self: Pin<&mut Self>) {
        let _ = self.client.fetch_mentions(Some(50), None);
    }

    pub fn load_older(self: Pin<&mut Self>) {
        let Some(channel_id) = parse_id(self.selected_channel_id()) else { return };
        let oldest = self.messages.first().map(|message| message.id);
        let _ = self.client.fetch_messages(channel_id, oldest, Some(50));
    }

    pub fn send_message(self: Pin<&mut Self>, content: QString) {
        let content = content.to_string();
        if content.trim().is_empty() {
            return;
        }
        let Some(channel_id) = parse_id(self.selected_channel_id()) else { return };
        let _ = self.client.send_message(channel_id, content, None, None);
    }

    pub fn send_reply(self: Pin<&mut Self>, content: QString, message_id: QString) {
        let content = content.to_string();
        if content.trim().is_empty() {
            return;
        }
        let Some(channel_id) = parse_id(self.selected_channel_id()) else { return };
        let Some(reply_id) = parse_id(&message_id) else { return };
        let guild_id = parse_id(self.selected_guild_id());
        let _ = self.client.send_message(channel_id, content, Some(reply_id), guild_id);
    }

    pub fn send_files(self: Pin<&mut Self>, content: QString, files: QString) {
        let Some(channel_id) = parse_id(self.selected_channel_id()) else { return };
        let urls: Vec<String> = serde_json::from_str(&files.to_string()).unwrap_or_default();
        let paths: Vec<String> = urls.iter().filter_map(|url| local_path(url)).collect();
        if paths.is_empty() {
            return;
        }
        let _ = self.client.send_attachments(channel_id, content.to_string(), paths, None, None);
    }

    pub fn set_status(mut self: Pin<&mut Self>, status: QString) {
        // Show it right away; the snapshot confirms once Discord accepts it.
        self.as_mut().set_current_user_status(status.clone());
        let _ = self.client.set_status(status.to_string());
    }

    pub fn set_custom_status(mut self: Pin<&mut Self>, text: QString) {
        self.as_mut().set_current_user_custom_status(text.clone());
        let text = text.to_string();
        let _ = self.client.set_custom_status(if text.trim().is_empty() { None } else { Some(text) });
    }

    pub fn edit_message(self: Pin<&mut Self>, message_id: QString, content: QString) {
        let content = content.to_string();
        if content.trim().is_empty() {
            return;
        }
        if let (Some(channel_id), Some(message_id)) =
            (parse_id(self.selected_channel_id()), parse_id(&message_id))
        {
            let _ = self.client.edit_message(channel_id, message_id, content);
        }
    }

    pub fn delete_message(self: Pin<&mut Self>, message_id: QString) {
        if let (Some(channel_id), Some(message_id)) =
            (parse_id(self.selected_channel_id()), parse_id(&message_id))
        {
            let _ = self.client.delete_message(channel_id, message_id);
        }
    }

    pub fn send_typing(self: Pin<&mut Self>) {
        if let Some(channel_id) = parse_id(self.selected_channel_id()) {
            let _ = self.client.send_typing(channel_id);
        }
    }

    pub fn ack_latest(mut self: Pin<&mut Self>) {
        let Some(channel_id) = parse_id(self.selected_channel_id()) else { return };
        let Some(latest) = self.messages.last().map(|message| message.id) else { return };
        if self.acked == Some(latest) {
            return;
        }
        self.as_mut().rust_mut().acked = Some(latest);
        let _ = self.client.ack_message(channel_id, latest);
    }

    pub fn mark_guild_read(self: Pin<&mut Self>, guild_id: QString) {
        if let Some(guild_id) = parse_id(&guild_id) {
            let _ = self.client.mark_guild_read(guild_id);
        }
    }

    pub fn add_reaction(self: Pin<&mut Self>, message_id: QString, emoji: QString) {
        if let (Some(channel_id), Some(message_id)) =
            (parse_id(self.selected_channel_id()), parse_id(&message_id))
        {
            let _ = self.client.add_reaction(channel_id, message_id, emoji.to_string());
        }
    }

    pub fn remove_reaction(self: Pin<&mut Self>, message_id: QString, emoji: QString) {
        if let (Some(channel_id), Some(message_id)) =
            (parse_id(self.selected_channel_id()), parse_id(&message_id))
        {
            let _ = self.client.remove_reaction(channel_id, message_id, emoji.to_string());
        }
    }

    pub fn pin_message(self: Pin<&mut Self>, message_id: QString) {
        if let (Some(channel_id), Some(message_id)) =
            (parse_id(self.selected_channel_id()), parse_id(&message_id))
        {
            let _ = self.client.pin_message(channel_id, message_id);
        }
    }

    pub fn unpin_message(self: Pin<&mut Self>, message_id: QString) {
        if let (Some(channel_id), Some(message_id)) =
            (parse_id(self.selected_channel_id()), parse_id(&message_id))
        {
            let _ = self.client.unpin_message(channel_id, message_id);
        }
    }

    pub fn join_guild(self: Pin<&mut Self>, invite_code: QString) {
        let code = invite_code.to_string();
        if !code.trim().is_empty() {
            let _ = self.client.join_guild(code);
        }
    }

    pub fn leave_guild(self: Pin<&mut Self>, guild_id: QString) {
        if let Some(guild_id) = parse_id(&guild_id) {
            let _ = self.client.leave_guild(guild_id);
        }
    }

    pub fn send_friend_request(self: Pin<&mut Self>, username: QString) {
        let name = username.to_string();
        if !name.trim().is_empty() {
            let _ = self.client.send_friend_request(name);
        }
    }

    pub fn accept_friend_request(self: Pin<&mut Self>, user_id: QString) {
        if let Some(user_id) = parse_id(&user_id) {
            let _ = self.client.accept_friend_request(user_id);
        }
    }

    pub fn remove_relationship(self: Pin<&mut Self>, user_id: QString) {
        if let Some(user_id) = parse_id(&user_id) {
            let _ = self.client.remove_relationship(user_id);
        }
    }

    pub fn block_user(self: Pin<&mut Self>, user_id: QString) {
        if let Some(user_id) = parse_id(&user_id) {
            let _ = self.client.block_user(user_id);
        }
    }

    pub fn open_dm(self: Pin<&mut Self>, user_id: QString) {
        if let Some(user_id) = parse_id(&user_id) {
            let _ = self.client.open_dm(user_id);
        }
    }

    pub fn search_messages(self: Pin<&mut Self>, query: QString, scope: QString) {
        let q = query.to_string();
        if q.trim().is_empty() {
            return;
        }
        let guild_id = if scope.to_string() == "channel" { None } else { parse_id(self.selected_guild_id()) };
        let channel_id = parse_id(self.selected_channel_id());
        let _ = self.client.search_messages(guild_id, channel_id, q);
    }

    pub fn fetch_threads(self: Pin<&mut Self>) {
        if let (Some(guild_id), Some(channel_id)) =
            (parse_id(self.selected_guild_id()), parse_id(self.selected_channel_id()))
        {
            let _ = self.client.fetch_threads(guild_id, channel_id);
        }
    }

    pub fn create_thread(self: Pin<&mut Self>, name: QString, message_id: QString) {
        let name = name.to_string();
        if name.trim().is_empty() {
            return;
        }
        if let Some(channel_id) = parse_id(self.selected_channel_id()) {
            let _ = self.client.create_thread(channel_id, name, parse_id(&message_id));
        }
    }

    pub fn fetch_guild_settings(self: Pin<&mut Self>, guild_id: QString) {
        if let Some(guild_id) = parse_id(&guild_id) {
            let _ = self.client.fetch_guild_settings(guild_id);
        }
    }

    pub fn create_guild_role(self: Pin<&mut Self>, guild_id: QString, name: QString, color: i32) {
        if let Some(guild_id) = parse_id(&guild_id) {
            let _ = self.client.create_guild_role(guild_id, name.to_string(), color.max(0) as u32);
        }
    }

    pub fn delete_guild_role(self: Pin<&mut Self>, guild_id: QString, role_id: QString) {
        if let (Some(guild_id), Some(role_id)) = (parse_id(&guild_id), parse_id(&role_id)) {
            let _ = self.client.delete_guild_role(guild_id, role_id);
        }
    }

    pub fn delete_guild_emoji(self: Pin<&mut Self>, guild_id: QString, emoji_id: QString) {
        if let (Some(guild_id), Some(emoji_id)) = (parse_id(&guild_id), parse_id(&emoji_id)) {
            let _ = self.client.delete_guild_emoji(guild_id, emoji_id);
        }
    }

    pub fn unban_member(self: Pin<&mut Self>, guild_id: QString, user_id: QString) {
        if let (Some(guild_id), Some(user_id)) = (parse_id(&guild_id), parse_id(&user_id)) {
            let _ = self.client.unban_member(guild_id, user_id);
        }
    }

    pub fn create_channel(self: Pin<&mut Self>, guild_id: QString, name: QString, kind: i32) {
        let name = name.to_string();
        if name.trim().is_empty() {
            return;
        }
        if let Some(guild_id) = parse_id(&guild_id) {
            let _ = self.client.create_channel(guild_id, name, kind.clamp(0, 255) as u8, None);
        }
    }

    pub fn delete_channel(self: Pin<&mut Self>, channel_id: QString) {
        if let Some(channel_id) = parse_id(&channel_id) {
            let _ = self.client.delete_channel(channel_id);
        }
    }

    pub fn switch_account(self: Pin<&mut Self>, user_id: QString) {
        let _ = self.client.switch_account(user_id.to_string());
    }

    pub fn remove_saved_account(self: Pin<&mut Self>, user_id: QString) {
        let _ = self.client.remove_saved_account(user_id.to_string());
    }

    pub fn clear_deleted_log(mut self: Pin<&mut Self>) {
        {
            let mut rust = self.as_mut().rust_mut();
            rust.deleted_messages.clear();
            rust.loaded_db_channels.clear();
        }
        let db = lampcord_db_path();
        if db.exists() {
            let _ = std::process::Command::new("sqlite3")
                .arg(&db)
                .arg("DELETE FROM logged_messages;")
                .status();
        }
        let empty = std::collections::HashSet::new();
        let json = messages_json(&self.messages, &self.members, &empty, self.me.as_ref());
        self.as_mut().set_messages_json(QString::from(json.as_str()));
    }

    pub fn render_markdown(
        &self,
        content: QString,
        code_bg: QString,
        mention_bg: QString,
        mention_fg: QString,
        muted_fg: QString,
        mono_font: QString,
    ) -> QString {
        let member_names: HashMap<u64, String> = self
            .members
            .iter()
            .map(|m| {
                let name = m
                    .nickname
                    .clone()
                    .filter(|n| !n.is_empty())
                    .or_else(|| m.global_name.clone())
                    .unwrap_or_else(|| m.username.clone());
                (m.user_id, name)
            })
            .collect();
        let channel_names: HashMap<u64, String> = self
            .channels
            .iter()
            .map(|c| (c.id, c.name.clone()))
            .collect();
        let role_info: HashMap<u64, (String, u32)> = self
            .guild_roles
            .iter()
            .map(|r| (r.id, (r.name.clone(), r.color)))
            .collect();
        let html = crate::markdown::render_markdown(
            &content.to_string(),
            &code_bg.to_string(),
            &mention_bg.to_string(),
            &mention_fg.to_string(),
            &muted_fg.to_string(),
            &mono_font.to_string(),
            &member_names,
            &channel_names,
            &role_info,
        );
        QString::from(html.as_str())
    }

    pub fn set_config(mut self: Pin<&mut Self>, key: QString, value: QString) {
        let key = key.to_string();
        let value = value.to_string();
        if matches!(key.as_str(), "discord_token" | "saved_accounts" | "rust_core_secrets") {
            return;
        }
        save_lampcord_property(&key, &value);
        let props = load_lampcord_properties();
        if key == "message_logger_enabled" {
            self.as_mut().set_log_deleted_messages(value == "true");
        } else if key == "notifications_enabled" {
            self.as_mut().set_desktop_notifications(value == "true");
        } else if key.starts_with("clean_channels_") {
            let ch_json = channels_json(&self.channels, &self.read_states);
            self.as_mut().set_channels_json(QString::from(ch_json.as_str()));
        }
        let json = properties_to_json(&props);
        self.as_mut().set_config_json(QString::from(json.as_str()));
    }

    pub fn logout(mut self: Pin<&mut Self>) {
        let _ = self.client.logout();
        self.as_mut().reset_auth();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn channel(id: u64, kind: u8, parent: Option<u64>, position: i32) -> ChannelSnapshot {
        ChannelSnapshot {
            id,
            guild_id: Some(1),
            name: format!("c{id}"),
            kind,
            parent_id: parent,
            position: Some(position),
            can_view: true,
            can_send: true,
            icon: None,
            recipients: Vec::new(),
            topic: None,
            nsfw: false,
            muted: false,
            unobtrusive_unread: false,
        }
    }

    #[test]
    fn channels_are_grouped_under_their_categories() {
        let channels = vec![
            channel(10, CHANNEL_CATEGORY, None, 1),
            channel(11, CHANNEL_VOICE, Some(10), 0),
            channel(12, CHANNEL_TEXT, Some(10), 1),
            channel(20, CHANNEL_TEXT, None, 0),
            channel(30, CHANNEL_CATEGORY, None, 0),
        ];
        let rows: Vec<Value> = serde_json::from_str(&channels_json(&channels, &[])).unwrap();
        let ids: Vec<&str> = rows.iter().map(|row| row["id"].as_str().unwrap()).collect();
        // Empty category 30 is dropped; text sorts above voice.
        assert_eq!(ids, ["20", "10", "12", "11"]);
        assert_eq!(rows[1]["category"], true);
    }

    #[test]
    fn rail_follows_folders_with_unfiled_servers_on_top() {
        let guild = |id: u64| GuildSnapshot { id, name: format!("g{id}"), icon_url: None, owner_id: None, unread: false, mention_count: 0 };
        let folder = |id: Option<i64>, guild_ids: Vec<u64>| GuildFolderSnapshot {
            id,
            name: None,
            color: Some(0x5865f2),
            guild_ids,
        };
        let guilds = vec![guild(1), guild(2), guild(3), guild(4)];
        let folders = vec![folder(None, vec![2]), folder(Some(9), vec![3, 1, 404])];
        let rows: Vec<Value> = serde_json::from_str(&rail_json(&guilds, &folders)).unwrap();
        let shape: Vec<String> = rows
            .iter()
            .map(|row| format!("{}:{}", row["kind"].as_str().unwrap(), row["id"].as_str().unwrap()))
            .collect();
        assert_eq!(shape, ["guild:4", "guild:2", "folder:9"]);
        // Unknown ids are skipped; folder order is kept.
        let inner: Vec<&str> = rows[2]["guilds"].as_array().unwrap().iter().map(|g| g["id"].as_str().unwrap()).collect();
        assert_eq!(inner, ["3", "1"]);
        assert_eq!(rows[2]["color"], "#5865f2");
    }

    #[test]
    fn member_list_puts_you_first_in_dms_and_offline_last_in_servers() {
        let me = Me {
            id: 1,
            name: "me".into(),
            avatar: None,
            decoration: None,
            nameplate: None,
            status: "online".into(),
            custom_status: None,
        };
        let dm = PrivateChannelSnapshot {
            id: 50,
            kind: 1,
            name: None,
            recipients: vec![lampcord_ffi::RecipientSnapshot {
                id: 2,
                username: "friend".into(),
                global_name: None,
                avatar: None,
                decoration: None,
                nameplate: None,
                system: false,
                bot: false,
            }],
            last_message_id: None,
        };
        let empty_list = MemberListSnapshot::default();
        let rows: Vec<Value> = serde_json::from_str(&members_json(
            None,
            Some(50),
            &[dm],
            &[],
            &empty_list,
            &[],
            &[],
            Some(&me),
        ))
        .unwrap();
        let ids: Vec<&str> = rows
            .iter()
            .filter(|row| !row["isGroup"].as_bool().unwrap_or(false))
            .map(|row| row["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, ["1", "2"]);
        assert_eq!(rows[0]["isGroup"], true);
        assert_eq!(rows[0]["label"], "Members — 2");

        let member = |user_id: u64, name: &str, status: Option<&str>| MemberSnapshot {
            guild_id: 9,
            user_id,
            username: name.into(),
            avatar: None,
            nickname: None,
            roles: vec![],
            status: status.map(str::to_owned),
            global_name: None,
            decoration: None,
            nameplate: None,
            clan_tag: None,
            clan_badge: None,
        };
        let members = [member(3, "zed", Some("online")), member(4, "amy", None), member(5, "bob", Some("idle"))];
        let rows: Vec<Value> = serde_json::from_str(&members_json(
            None,
            Some(77),
            &[],
            &members,
            &empty_list,
            &[],
            &[],
            Some(&me),
        ))
        .unwrap();
        let names: Vec<&str> = rows
            .iter()
            .filter(|row| !row["isGroup"].as_bool().unwrap_or(false))
            .map(|row| row["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["bob", "zed", "amy"]);
        let groups: Vec<&str> = rows
            .iter()
            .filter(|row| row["isGroup"].as_bool().unwrap_or(false))
            .map(|row| row["label"].as_str().unwrap())
            .collect();
        assert_eq!(groups, ["Online — 2", "Offline — 1"]);
    }

    #[test]
    fn file_urls_become_local_paths() {
        assert_eq!(local_path("file:///home/me/My%20Cat.png").as_deref(), Some("/home/me/My Cat.png"));
        assert_eq!(local_path("/tmp/a.txt").as_deref(), Some("/tmp/a.txt"));
    }

    #[test]
    fn consecutive_messages_from_one_author_are_grouped() {
        let message = |id: u64, author: u64| MessageSnapshot {
            id,
            channel_id: 1,
            author_id: Some(author),
            author_name: format!("u{author}"),
            content: String::new(),
            timestamp: String::new(),
            edited_timestamp: None,
            kind: 0,
            author_avatar: None,
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
        };
        let minute = 60_000u64 << 22;
        let base = 1u64 << 40;
        let messages = vec![
            message(base, 1),
            message(base + minute, 1),
            message(base + 2 * minute, 2),
            message(base + 20 * minute, 2),
        ];
        let rows: Vec<Value> = serde_json::from_str(&messages_json(&messages, &[], &std::collections::HashSet::new(), None)).unwrap();
        let grouped: Vec<bool> = rows.iter().map(|row| row["grouped"].as_bool().unwrap()).collect();
        assert_eq!(grouped, [false, true, false, false]);
    }
}
