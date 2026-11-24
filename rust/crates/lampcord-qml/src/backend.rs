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
        fn logout(self: Pin<&mut LampcordBackend>);
    }
}

use std::collections::HashMap;
use std::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use lampcord_ffi::{
    AuthChallenge, ChannelSnapshot, ClientConfig, ConnectionState, GuildFolderSnapshot,
    GuildSnapshot, LampcordClient, PrivateChannelSnapshot, RemoteAuthState,
    MemberSnapshot, MessageSnapshot, PresenceSnapshot, ReadStateSnapshot, SnapshotVersions,
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
    presences: Vec<PresenceSnapshot>,
    read_states: Vec<ReadStateSnapshot>,
    me: Option<Me>,
    selected_channel: Option<u64>,
    mfa_ticket: Option<String>,
    mfa_method: String,
    acked: Option<u64>,

    connection: QString,
    has_token: bool,
    current_user: QString,
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
            presences: Vec::new(),
            read_states: Vec::new(),
            me: None,
            selected_channel: None,
            mfa_ticket: None,
            mfa_method: "totp".to_owned(),
            acked: None,
            connection: QString::from("disconnected"),
            has_token: false,
            current_user: QString::default(),
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

/// The right sidebar for the open channel: everyone in a DM (you first),
/// or the server's cached members, online before offline.
fn members_json(
    selected_channel: Option<u64>,
    private_channels: &[PrivateChannelSnapshot],
    members: &[MemberSnapshot],
    presences: &[PresenceSnapshot],
    me: Option<&Me>,
) -> String {
    let presence: HashMap<u64, &PresenceSnapshot> =
        presences.iter().map(|presence| (presence.user_id, presence)).collect();
    let row = |id: u64,
               name: String,
               avatar: Option<&str>,
               status: Option<&str>,
               decoration: Option<&str>,
               nameplate: Option<&str>| {
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
            "id": id.to_string(),
            "name": name,
            "avatar": if url.is_empty() { default_avatar_url(id) } else { url },
            "status": status,
            "activity": activity,
            "decoration": decoration_url(decoration),
            "nameplate": nameplate_url(nameplate),
        })
    };

    let dm = selected_channel.and_then(|id| private_channels.iter().find(|channel| channel.id == id));
    let mut rows: Vec<Value> = if let Some(dm) = dm {
        me.map(|me| {
            row(me.id, me.name.clone(), me.avatar.as_deref(), None, me.decoration.as_deref(), me.nameplate.as_deref())
        })
            .into_iter()
            .chain(dm.recipients.iter().map(|user| {
                let name = user.global_name.clone().unwrap_or_else(|| user.username.clone());
                row(user.id, name, user.avatar.as_deref(), None, user.decoration.as_deref(), user.nameplate.as_deref())
            }))
            .collect()
    } else if selected_channel.is_some() {
        members
            .iter()
            .map(|member| {
                let name = member
                    .nickname
                    .clone()
                    .filter(|nick| !nick.is_empty())
                    .or_else(|| member.global_name.clone())
                    .unwrap_or_else(|| member.username.clone());
                row(
                    member.user_id,
                    name,
                    member.avatar.as_deref(),
                    member.status.as_deref(),
                    member.decoration.as_deref(),
                    member.nameplate.as_deref(),
                )
            })
            .collect()
    } else {
        Vec::new()
    };
    if dm.is_none() {
        let offline = |row: &Value| matches!(row["status"].as_str(), Some("offline" | "invisible"));
        rows.sort_by(|a, b| {
            offline(a)
                .cmp(&offline(b))
                .then_with(|| a["name"].as_str().unwrap_or("").to_lowercase().cmp(&b["name"].as_str().unwrap_or("").to_lowercase()))
        });
    }
    Value::Array(rows).to_string()
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
        items.push(json!({
            "kind": "folder",
            "id": key,
            "name": folder.name.clone().unwrap_or_default(),
            "color": folder.color.map(|color| format!("#{color:06x}")).unwrap_or_default(),
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
            let status = recipient
                .and_then(|user| presence.get(&user.id))
                .map(|presence| {
                    if presence.mobile && presence.status == "online" { "mobile" } else { presence.status.as_str() }
                })
                .unwrap_or("offline");
            json!({
                "id": channel.id.to_string(),
                "name": name,
                "group": group,
                "avatar": avatar,
                "status": if group { "" } else { status },
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
fn channels_json(channels: &[ChannelSnapshot], read_states: &[ReadStateSnapshot]) -> String {
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
            "name": channel.name,
            "kind": channel.kind,
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
            "name": category.name.to_uppercase(),
            "kind": CHANNEL_CATEGORY,
            "category": true,
            "unread": false,
            "mentions": 0,
        }));
        items.extend(children.into_iter().map(row));
    }
    Value::Array(items).to_string()
}

/// What Discord shows instead of content for system messages, after the
/// author's name. `None` for ordinary messages.
fn system_text(kind: u8) -> Option<&'static str> {
    Some(match kind {
        1 => "added a recipient.",
        2 => "removed a recipient.",
        3 => "started a call.",
        4 => "changed the channel name.",
        5 => "changed the channel icon.",
        6 => "pinned a message to this channel.",
        7 => "joined the server.",
        8 => "just boosted the server!",
        9..=11 => "just boosted the server to a new level!",
        12 => "added a followed channel to this channel.",
        18 => "started a thread.",
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

/// Message rows for the chat list. Kept flat (attachments as a JSON
/// string, reply fields inlined) because they feed a QML ListModel.
fn messages_json(messages: &[MessageSnapshot], members: &[MemberSnapshot]) -> String {
    let members: HashMap<u64, &MemberSnapshot> =
        members.iter().map(|member| (member.user_id, member)).collect();
    let user_avatar = |id: u64, hash: Option<&str>| {
        let url = avatar_url(id, hash);
        if url.is_empty() { default_avatar_url(id) } else { url }
    };
    let mut previous: Option<&MessageSnapshot> = None;
    let items: Vec<Value> = messages
        .iter()
        .map(|message| {
            let time = snowflake_ms(message.id);
            let system = system_text(message.kind);
            // Like Compose: replies and system rows always start a new group.
            let grouped = message.reply.is_none()
                && system.is_none()
                && previous.is_some_and(|prev| {
                    prev.author_id == message.author_id
                        && system_text(prev.kind).is_none()
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
                        "width": attachment.width.unwrap_or(0),
                        "height": attachment.height.unwrap_or(0),
                    })
                })
                .collect();
            let reply = message.reply.as_ref();
            json!({
                "id": message.id.to_string(),
                "channelId": message.channel_id.to_string(),
                "authorId": message.author_id.map(|id| id.to_string()).unwrap_or_default(),
                "author": name,
                "avatar": avatar,
                "decoration": decoration_url(message.author_decoration.as_deref()),
                "content": message.content,
                "time": time,
                "edited": message.edited_timestamp.is_some(),
                "grouped": grouped,
                "system": system.unwrap_or(""),
                "kind": message.kind,
                "attachments": Value::Array(attachments).to_string(),
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
        let since = self.versions.clone();
        let patch = self.client.snapshot_patch(since);
        if patch.versions == self.versions {
            return;
        }
        self.as_mut().rust_mut().versions = patch.versions.clone();

        let mut members_dirty = false;
        if let Some(meta) = patch.meta {
            let connection = QString::from(connection_name(&meta.connection));
            self.as_mut().set_connection(connection);
            self.as_mut().set_has_token(meta.has_token);
            self.as_mut()
                .set_current_user(QString::from(meta.current_user.as_deref().unwrap_or("")));
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
            let profile = serde_json::to_string(&meta.profile).unwrap_or_else(|_| "{}".to_owned());
            self.as_mut().set_profile_json(QString::from(profile.as_str()));
            let me = meta.current_user_id.map(|id| Me {
                id,
                name: meta.current_user.clone().unwrap_or_default(),
                avatar: meta.current_user_avatar.clone(),
                decoration: meta.current_user_decoration.clone(),
                nameplate: meta.current_user_nameplate.clone(),
                status,
                custom_status: meta.current_user_custom_status.clone(),
            });
            if me != self.me || meta.selected_channel_id != self.selected_channel {
                let mut rust = self.as_mut().rust_mut();
                rust.me = me;
                rust.selected_channel = meta.selected_channel_id;
                members_dirty = true;
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

        let mut rail_dirty = false;
        if let Some(guilds) = patch.guilds {
            self.as_mut().set_guilds_json(QString::from(guilds_json(&guilds).as_str()));
            self.as_mut().rust_mut().guilds = guilds;
            rail_dirty = true;
        }
        if let Some(folders) = patch.guild_folders {
            self.as_mut().rust_mut().guild_folders = folders;
            rail_dirty = true;
        }
        if rail_dirty {
            let json = rail_json(&self.guilds, &self.guild_folders);
            self.as_mut().set_rail_json(QString::from(json.as_str()));
        }

        let mut channels_dirty = false;
        let mut messages_dirty = false;
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
        if let Some(messages) = patch.messages {
            self.as_mut().rust_mut().messages = messages;
            messages_dirty = true;
        }
        if let Some(pins) = patch.pinned_messages {
            // Newest pin first; each one stands alone, so nothing groups.
            let mut rows: Vec<Value> = serde_json::from_str(&messages_json(&pins, &self.members)).unwrap_or_default();
            rows.reverse();
            for row in &mut rows {
                row["grouped"] = Value::Bool(false);
            }
            self.as_mut().set_pins_json(QString::from(Value::Array(rows).to_string().as_str()));
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
            let json = members_json(
                self.selected_channel,
                &self.private_channels,
                &self.members,
                &self.presences,
                self.me.as_ref(),
            );
            self.as_mut().set_members_json(QString::from(json.as_str()));
        }
        if mentions_dirty {
            let json = messages_json(&self.mentions, &self.members);
            self.as_mut().set_mentions_json(QString::from(json.as_str()));
        }
        if dms_dirty {
            let json = dms_json(&self.private_channels, &self.read_states, &self.presences);
            self.as_mut().set_dms_json(QString::from(json.as_str()));
        }
        if messages_dirty {
            let json = messages_json(&self.messages, &self.members);
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
        let guild = |id: u64| GuildSnapshot { id, name: format!("g{id}"), icon_url: None };
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
            }],
            last_message_id: None,
        };
        let rows: Vec<Value> =
            serde_json::from_str(&members_json(Some(50), &[dm], &[], &[], Some(&me))).unwrap();
        let ids: Vec<&str> = rows.iter().map(|row| row["id"].as_str().unwrap()).collect();
        assert_eq!(ids, ["1", "2"]);

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
        };
        let members = [member(3, "zed", Some("online")), member(4, "amy", None), member(5, "bob", Some("idle"))];
        let rows: Vec<Value> =
            serde_json::from_str(&members_json(Some(77), &[], &members, &[], Some(&me))).unwrap();
        let names: Vec<&str> = rows.iter().map(|row| row["name"].as_str().unwrap()).collect();
        assert_eq!(names, ["bob", "zed", "amy"]);
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
        };
        let minute = 60_000u64 << 22;
        let base = 1u64 << 40;
        let messages = vec![
            message(base, 1),
            message(base + minute, 1),
            message(base + 2 * minute, 2),
            message(base + 20 * minute, 2),
        ];
        let rows: Vec<Value> = serde_json::from_str(&messages_json(&messages, &[])).unwrap();
        let grouped: Vec<bool> = rows.iter().map(|row| row["grouped"].as_bool().unwrap()).collect();
        assert_eq!(grouped, [false, true, false, false]);
    }
}
