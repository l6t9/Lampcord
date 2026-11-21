use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

use async_trait::async_trait;
use base64::Engine;
use lampcord_core::{
    api,
    captcha::CaptchaChallenge,
    gateway_client::{GatewayClient, GatewayEvent},
    gateway_plan::reconnect_delay_ms,
    rest::RestConfig,
    rest_client::RestClient,
    state_cache::StateCache,
    token_store::{SavedAccount, TokenStore},
    traits::{CaptchaSolver, GatewayUrlProvider},
};
use lampcord_model::{message::Message, snowflake::Snowflake};
use lampcord_transport::{http::ReqwestRestTransport, websocket::TungsteniteGatewayTransport};
use tokio::sync::mpsc;

use crate::{
    snapshot::{ChannelSnapshot, ClientSnapshot, GuildSnapshot, MessageSnapshot},
    types::{ClientCommand, ClientConfig, ClientEvent, ConnectionState},
};

struct NoopCaptchaSolver;

#[async_trait]
impl CaptchaSolver for NoopCaptchaSolver {
    async fn solve(&self, _challenge: &CaptchaChallenge) -> Option<String> {
        None
    }
}

struct StaticGatewayUrl {
    rest: Arc<RestClient<ReqwestRestTransport, NoopCaptchaSolver>>,
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

fn default_rest_config(api_base: &str, token: Option<String>) -> RestConfig {
    let super_properties = serde_json::json!({
        "os": "Linux",
        "browser": "Discord Client",
        "device": "lampcord-ffi",
        "system_locale": "en-US",
        "has_client_mods": false,
        "release_channel": "stable",
        "client_version": "0.0.0",
        "os_version": "",
        "browser_user_agent": "",
        "browser_version": "",
        "device_vendor_id": "",
        "client_build_number": 0
    });
    let encoded =
        base64::engine::general_purpose::STANDARD.encode(super_properties.to_string().as_bytes());
    RestConfig::new(
        token,
        encoded,
        "lampcord-ffi-installation".to_owned(),
        "UTC".to_owned(),
    )
    .with_api_base(api_base.to_owned())
}

#[derive(Clone)]
pub struct RuntimeHandle {
    cmd_tx: mpsc::UnboundedSender<ClientCommand>,
    snapshot: Arc<Mutex<ClientSnapshot>>,
    events: Arc<Mutex<Vec<ClientEvent>>>,
}

impl RuntimeHandle {
    pub fn send(&self, cmd: ClientCommand) -> bool {
        self.cmd_tx.send(cmd).is_ok()
    }

    pub fn snapshot(&self) -> ClientSnapshot {
        self.snapshot.lock().expect("snapshot lock").clone()
    }

    pub fn drain_events(&self) -> Vec<ClientEvent> {
        let mut events = self.events.lock().expect("events lock");
        std::mem::take(&mut *events)
    }
}

pub fn start(config: ClientConfig) -> RuntimeHandle {
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    let snapshot = Arc::new(Mutex::new(ClientSnapshot {
        connection: ConnectionState::Disconnected,
        ..ClientSnapshot::default()
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
            rt.block_on(run(config, cmd_rx, snapshot, events));
        })
        .expect("spawn lampcord-ffi runtime thread");

    handle
}

fn push_event(events: &Arc<Mutex<Vec<ClientEvent>>>, event: ClientEvent) {
    events.lock().expect("events lock").push(event);
}

fn load_token_store(path: Option<&str>) -> TokenStore {
    let Some(path) = path else {
        return TokenStore::new();
    };
    match std::fs::read_to_string(path) {
        Ok(body) => TokenStore::from_json(&body),
        Err(_) => TokenStore::new(),
    }
}

fn save_token_store(path: Option<&str>, store: &TokenStore) {
    let Some(path) = path else {
        return;
    };
    if let Some(parent) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, store.to_json());
}

fn set_snapshot(
    snapshot: &Arc<Mutex<ClientSnapshot>>,
    events: &Arc<Mutex<Vec<ClientEvent>>>,
    seq: &Arc<std::sync::atomic::AtomicU64>,
    mut next: ClientSnapshot,
) {
    next.seq = seq.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    *snapshot.lock().expect("snapshot lock") = next;
    push_event(events, ClientEvent::SnapshotUpdated);
}

fn build_snapshot(
    connection: ConnectionState,
    has_token: bool,
    cache: &Option<Arc<Mutex<StateCache>>>,
    selected_guild_id: Option<Snowflake>,
    selected_channel_id: Option<Snowflake>,
    fetched_messages: &HashMap<Snowflake, Vec<Message>>,
    error: Option<String>,
) -> ClientSnapshot {
    let Some(cache) = cache else {
        return ClientSnapshot {
            connection,
            has_token,
            selected_guild_id,
            selected_channel_id,
            error,
            ..ClientSnapshot::default()
        };
    };
    let cache = cache.lock().expect("cache lock");

    let current_user_id = cache.current_user().map(|u| u.id);
    let current_user = cache.current_user().map(|u| {
        u.global_name
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| u.username.clone())
    });

    let guilds: Vec<GuildSnapshot> = cache
        .ordered_guilds()
        .into_iter()
        .map(|g| GuildSnapshot {
            id: g.id,
            name: g.name.clone().unwrap_or_else(|| "Unknown guild".to_owned()),
            icon_url: g.icon.clone(),
        })
        .collect();

    let channels: Vec<ChannelSnapshot> = selected_guild_id
        .into_iter()
        .flat_map(|gid| {
            cache
                .channels_for_guild(gid)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>()
        })
        .map(|c| ChannelSnapshot {
            id: c.id,
            guild_id: c.guild_id,
            name: c
                .name
                .clone()
                .unwrap_or_else(|| format!("channel-{}", c.id)),
            kind: c.kind.0,
            parent_id: c.parent_id,
            position: c.position.map(|p| p as i32),
        })
        .collect();

    let mut messages: Vec<MessageSnapshot> = if let Some(channel_id) = selected_channel_id {
        let mut merged: HashMap<Snowflake, &Message> = HashMap::new();
        if let Some(fetched) = fetched_messages.get(&channel_id) {
            for m in fetched {
                merged.insert(m.id, m);
            }
        }
        for m in cache.messages(channel_id) {
            merged.insert(m.id, m);
        }
        let mut list = merged.into_values().cloned().collect::<Vec<_>>();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list.into_iter()
            .map(|m| MessageSnapshot {
                id: m.id,
                channel_id: m.channel_id,
                author_id: Some(m.author.id),
                author_name: m
                    .author
                    .global_name
                    .clone()
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| m.author.username.clone()),
                content: m.content.clone(),
                timestamp: m.timestamp.clone(),
                edited_timestamp: m.edited_timestamp.clone(),
            })
            .collect()
    } else {
        Vec::new()
    };
    if messages.len() > 200 {
        let keep_from = messages.len() - 200;
        messages = messages.split_off(keep_from);
    }

    ClientSnapshot {
        seq: 0,
        connection,
        has_token,
        current_user_id,
        current_user,
        selected_guild_id,
        selected_channel_id,
        guilds,
        channels,
        messages,
        error,
    }
}

async fn run(
    config: ClientConfig,
    mut cmd_rx: mpsc::UnboundedReceiver<ClientCommand>,
    snapshot: Arc<Mutex<ClientSnapshot>>,
    events: Arc<Mutex<Vec<ClientEvent>>>,
) {
    let mut connection = ConnectionState::Disconnected;
    let mut error: Option<String> = None;
    let mut active_token: Option<String> = None;
    let mut cache: Option<Arc<Mutex<StateCache>>> = None;
    let mut rest: Option<Arc<RestClient<ReqwestRestTransport, NoopCaptchaSolver>>> = None;
    let mut selected_guild_id: Option<Snowflake> = None;
    let mut selected_channel_id: Option<Snowflake> = None;
    let mut fetched_messages: HashMap<Snowflake, Vec<Message>> = HashMap::new();
    let mut gateway_task: Option<tokio::task::JoinHandle<()>> = None;
    let mut gateway_events: Option<mpsc::UnboundedReceiver<GatewayEvent>> = None;
    let mut should_reconnect = false;
    let mut _reconnect_attempt: u32 = 0;
    let storage_path = config.storage_path.clone();
    let mut token_store = load_token_store(storage_path.as_deref());
    let api_base = config.api_base.clone();
    let gateway_url_override = config.gateway_url.clone();
    let seq = Arc::new(std::sync::atomic::AtomicU64::new(0));

    let publish = |connection: &ConnectionState,
                   has_token: bool,
                   cache: &Option<Arc<Mutex<StateCache>>>,
                   selected_guild_id: Option<Snowflake>,
                   selected_channel_id: Option<Snowflake>,
                   fetched_messages: &HashMap<Snowflake, Vec<Message>>,
                   error: &Option<String>,
                   snapshot: &Arc<Mutex<ClientSnapshot>>,
                   events: &Arc<Mutex<Vec<ClientEvent>>>,
                   seq: &Arc<std::sync::atomic::AtomicU64>| {
        let next = build_snapshot(
            connection.clone(),
            has_token,
            cache,
            selected_guild_id,
            selected_channel_id,
            fetched_messages,
            error.clone(),
        );
        set_snapshot(snapshot, events, seq, next);
    };

    let has_stored_token = !token_store.accounts().is_empty();
    publish(
        &connection,
        active_token.is_some() || has_stored_token,
        &cache,
        selected_guild_id,
        selected_channel_id,
        &fetched_messages,
        &error,
        &snapshot,
        &events,
        &seq,
    );

    loop {
        tokio::select! {
            maybe_cmd = cmd_rx.recv() => {
                let Some(cmd) = maybe_cmd else { break; };
                match cmd {
                    ClientCommand::Shutdown => {
                        if let Some(task) = gateway_task.take() { task.abort(); }
                        break;
                    }
                    ClientCommand::Logout => {
                        if let Some(task) = gateway_task.take() { task.abort(); }
                        gateway_events = None;
                        should_reconnect = false;
                        _reconnect_attempt = 0;
                        if let Some(token) = active_token.as_deref() {
                            token_store.remove(token);
                            save_token_store(storage_path.as_deref(), &token_store);
                        }
                        active_token = None;
                        cache = None;
                        rest = None;
                        selected_guild_id = None;
                        selected_channel_id = None;
                        fetched_messages.clear();
                        error = None;
                        connection = ConnectionState::Disconnected;
                        push_event(&events, ClientEvent::LoggedOut);
                        publish(&connection, active_token.is_some(), &cache, selected_guild_id, selected_channel_id, &fetched_messages, &error, &snapshot, &events, &seq);
                    }
                    ClientCommand::ConnectToken { token } => {
                        if let Some(task) = gateway_task.take() { task.abort(); }
                        gateway_events = None;
                        should_reconnect = true;
                        _reconnect_attempt = 0;
                        active_token = Some(token.clone());
                        token_store.remove(&token);
                        token_store.add(SavedAccount {
                            token: token.clone(),
                            user_id: format!("token:{:x}", token.len()),
                            username: None,
                        });
                        save_token_store(storage_path.as_deref(), &token_store);
                        connection = ConnectionState::Connecting;
                        error = None;
                        fetched_messages.clear();

                        let transport = match ReqwestRestTransport::new() {
                            Ok(t) => t,
                            Err(e) => {
                                connection = ConnectionState::Error { message: e.to_string() };
                                error = Some(format!("REST transport init failed: {e}"));
                                should_reconnect = false;
                                publish(&connection, active_token.is_some(), &cache, selected_guild_id, selected_channel_id, &fetched_messages, &error, &snapshot, &events, &seq);
                                continue;
                            }
                        };
                        let rest_client = Arc::new(RestClient::new(default_rest_config(&api_base, Some(token.clone())), transport, NoopCaptchaSolver));
                        let gw_transport = TungsteniteGatewayTransport::default();
                        let url_provider = StaticGatewayUrl {
                            rest: rest_client.clone(),
                            override_url: gateway_url_override.clone(),
                        };
                        let mut client = GatewayClient::new(gw_transport, url_provider);
                        let client_cache = client.cache().clone();
                        cache = Some(client_cache);
                        rest = Some(rest_client);
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
                                        eprintln!("[lampcord-ffi] gateway error: {e}");
                                        None
                                    }
                                };
                                let reconnect = match terminal {
                                    Some(GatewayEvent::AuthFailed { .. }) => false,
                                    Some(GatewayEvent::Closed { code }) if matches!(code, Some(4004)|Some(4010)|Some(4011)|Some(4012)|Some(4013)|Some(4014)) => false,
                                    _ => true,
                                };
                                if !reconnect { break; }
                                let delay_ms = reconnect_delay_ms(attempt);
                                attempt = attempt.saturating_add(1);
                                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                            }
                        }));
                        push_event(&events, ClientEvent::ConnectionChanged(connection.clone()));
                        publish(&connection, active_token.is_some(), &cache, selected_guild_id, selected_channel_id, &fetched_messages, &error, &snapshot, &events, &seq);
                    }
                    ClientCommand::ConnectStoredToken => {
                        if let Some(account) = token_store.accounts().first().cloned() {
                            // Re-dispatch by simulating connect token command path.
                            // Keep it simple: set token and fall through logic by reusing a local block is not possible;
                            // so we duplicate the minimal connect flow by sending to ourselves? No.
                            // Instead, just handle inline by calling the same steps.
                            let token = account.token;
                            if let Some(task) = gateway_task.take() { task.abort(); }
                            gateway_events = None;
                            should_reconnect = true;
                            _reconnect_attempt = 0;
                            active_token = Some(token.clone());
                            connection = ConnectionState::Connecting;
                            error = None;
                            fetched_messages.clear();
                            let transport = match ReqwestRestTransport::new() {
                                Ok(t) => t,
                                Err(e) => {
                                    connection = ConnectionState::Error { message: e.to_string() };
                                    error = Some(format!("REST transport init failed: {e}"));
                                    should_reconnect = false;
                                    publish(&connection, active_token.is_some(), &cache, selected_guild_id, selected_channel_id, &fetched_messages, &error, &snapshot, &events, &seq);
                                    continue;
                                }
                            };
                            let rest_client = Arc::new(RestClient::new(default_rest_config(&api_base, Some(token.clone())), transport, NoopCaptchaSolver));
                            let gw_transport = TungsteniteGatewayTransport::default();
                            let url_provider = StaticGatewayUrl {
                                rest: rest_client.clone(),
                                override_url: gateway_url_override.clone(),
                            };
                            let mut client = GatewayClient::new(gw_transport, url_provider);
                            let client_cache = client.cache().clone();
                            cache = Some(client_cache);
                            rest = Some(rest_client);
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
                                            eprintln!("[lampcord-ffi] gateway error: {e}");
                                            None
                                        }
                                    };
                                    let reconnect = match terminal {
                                        Some(GatewayEvent::AuthFailed { .. }) => false,
                                        Some(GatewayEvent::Closed { code }) if matches!(code, Some(4004)|Some(4010)|Some(4011)|Some(4012)|Some(4013)|Some(4014)) => false,
                                        _ => true,
                                    };
                                    if !reconnect { break; }
                                    let delay_ms = reconnect_delay_ms(attempt);
                                    attempt = attempt.saturating_add(1);
                                    tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                                }
                            }));
                            push_event(&events, ClientEvent::ConnectionChanged(connection.clone()));
                        } else {
                            error = Some("No stored token available".to_owned());
                            push_event(&events, ClientEvent::Error { message: error.clone().unwrap_or_default() });
                        }
                        publish(&connection, active_token.is_some(), &cache, selected_guild_id, selected_channel_id, &fetched_messages, &error, &snapshot, &events, &seq);
                    }
                    ClientCommand::SelectGuild { guild_id } => {
                        selected_guild_id = guild_id.map(Snowflake::from);
                        selected_channel_id = None;
                        publish(&connection, active_token.is_some(), &cache, selected_guild_id, selected_channel_id, &fetched_messages, &error, &snapshot, &events, &seq);
                    }
                    ClientCommand::SelectChannel { channel_id } => {
                        selected_channel_id = channel_id.map(Snowflake::from);
                        if let Some(cid) = selected_channel_id {
                            if let Some(rest_client) = rest.clone() {
                                let config = default_rest_config(&api_base, active_token.clone());
                                if let Ok(mut msgs) = api::fetch_channel_messages(&*rest_client, &config, cid, 50, None).await {
                                    msgs.sort_by(|a, b| a.id.cmp(&b.id));
                                    fetched_messages.insert(cid, msgs);
                                }
                            }
                        }
                        publish(&connection, active_token.is_some(), &cache, selected_guild_id, selected_channel_id, &fetched_messages, &error, &snapshot, &events, &seq);
                    }
                    ClientCommand::FetchMessages { channel_id, before, limit } => {
                        let cid = Snowflake::from(channel_id);
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, active_token.clone());
                            let limit = limit.unwrap_or(50).clamp(1, 100);
                            match api::fetch_channel_messages(&*rest_client, &config, cid, limit, before.map(Snowflake::from)).await {
                                Ok(mut msgs) => {
                                    msgs.sort_by(|a, b| a.id.cmp(&b.id));
                                    let entry = fetched_messages.entry(cid).or_default();
                                    for m in msgs {
                                        if !entry.iter().any(|e| e.id == m.id) {
                                            entry.push(m);
                                        }
                                    }
                                    entry.sort_by(|a, b| a.id.cmp(&b.id));
                                    if entry.len() > 300 {
                                        let keep_from = entry.len() - 300;
                                        *entry = entry.split_off(keep_from);
                                    }
                                }
                                Err(e) => {
                                    error = Some(format!("Failed to fetch messages: {e}"));
                                    push_event(&events, ClientEvent::Error { message: error.clone().unwrap_or_default() });
                                }
                            }
                        }
                        publish(&connection, active_token.is_some(), &cache, selected_guild_id, selected_channel_id, &fetched_messages, &error, &snapshot, &events, &seq);
                    }
                    ClientCommand::SendMessage { channel_id, content, reply_to_message_id, reply_to_guild_id } => {
                        let cid = Snowflake::from(channel_id);
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, active_token.clone());
                            match api::send_channel_message_with_reply(&*rest_client, &config, cid, &content, reply_to_message_id, reply_to_guild_id).await {
                                Ok(msg) => {
                                    let entry = fetched_messages.entry(cid).or_default();
                                    if !entry.iter().any(|m| m.id == msg.id) {
                                        entry.push(msg);
                                        entry.sort_by(|a, b| a.id.cmp(&b.id));
                                    }
                                    error = None;
                                }
                                Err(e) => {
                                    error = Some(format!("Failed to send message: {e}"));
                                    push_event(&events, ClientEvent::Error { message: error.clone().unwrap_or_default() });
                                }
                            }
                        }
                        publish(&connection, active_token.is_some(), &cache, selected_guild_id, selected_channel_id, &fetched_messages, &error, &snapshot, &events, &seq);
                    }
                    ClientCommand::EditMessage { channel_id, message_id, content } => {
                        let cid = Snowflake::from(channel_id);
                        let mid = Snowflake::from(message_id);
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, active_token.clone());
                            match api::message::edit_message(&*rest_client, &config, channel_id, message_id, &content).await {
                                Ok(msg) => {
                                    let entry = fetched_messages.entry(cid).or_default();
                                    if let Some(existing) = entry.iter_mut().find(|m| m.id == mid) {
                                        *existing = msg;
                                    } else {
                                        entry.push(msg);
                                        entry.sort_by(|a, b| a.id.cmp(&b.id));
                                    }
                                    error = None;
                                }
                                Err(e) => {
                                    error = Some(format!("Failed to edit message: {e}"));
                                    push_event(&events, ClientEvent::Error { message: error.clone().unwrap_or_default() });
                                }
                            }
                        }
                        publish(&connection, active_token.is_some(), &cache, selected_guild_id, selected_channel_id, &fetched_messages, &error, &snapshot, &events, &seq);
                    }
                    ClientCommand::DeleteMessage { channel_id, message_id } => {
                        let cid = Snowflake::from(channel_id);
                        let mid = Snowflake::from(message_id);
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, active_token.clone());
                            match api::message::delete_message(&*rest_client, &config, channel_id, message_id).await {
                                Ok(()) => {
                                    if let Some(entry) = fetched_messages.get_mut(&cid) {
                                        entry.retain(|m| m.id != mid);
                                    }
                                    error = None;
                                }
                                Err(e) => {
                                    error = Some(format!("Failed to delete message: {e}"));
                                    push_event(&events, ClientEvent::Error { message: error.clone().unwrap_or_default() });
                                }
                            }
                        }
                        publish(&connection, active_token.is_some(), &cache, selected_guild_id, selected_channel_id, &fetched_messages, &error, &snapshot, &events, &seq);
                    }
                    ClientCommand::SendTyping { channel_id } => {
                        if let Some(rest_client) = rest.clone() {
                            let config = default_rest_config(&api_base, active_token.clone());
                            if let Err(e) = api::channel::trigger_typing(&*rest_client, &config, channel_id).await {
                                error = Some(format!("Failed to send typing: {e}"));
                                push_event(&events, ClientEvent::Error { message: error.clone().unwrap_or_default() });
                            }
                        }
                        publish(&connection, active_token.is_some(), &cache, selected_guild_id, selected_channel_id, &fetched_messages, &error, &snapshot, &events, &seq);
                    }
                    ClientCommand::AckMessage { channel_id, message_id } => {
                        if let Some(cache) = cache.as_ref() {
                            let mut guard = cache.lock().expect("cache lock");
                            guard.read_state_mut().ack(channel_id, message_id);
                        }
                        publish(&connection, active_token.is_some(), &cache, selected_guild_id, selected_channel_id, &fetched_messages, &error, &snapshot, &events, &seq);
                    }
                }
            }
            maybe_gw = async {
                match gateway_events.as_mut() {
                    Some(rx) => rx.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                let Some(ev) = maybe_gw else { gateway_events = None; continue; };
                let prev_connection = connection.clone();
                match ev {
                    GatewayEvent::Hello { .. } => {
                        connection = ConnectionState::Connecting;
                    }
                    GatewayEvent::Ready { .. } => {
                        connection = ConnectionState::Ready;
                        _reconnect_attempt = 0;
                        error = None;
                    }
                    GatewayEvent::Resumed => {
                        connection = ConnectionState::Ready;
                        _reconnect_attempt = 0;
                    }
                    GatewayEvent::AuthFailed { .. } => {
                        connection = ConnectionState::Error { message: "Authentication failed".to_owned() };
                        error = Some("Authentication failed".to_owned());
                        should_reconnect = false;
                    }
                    GatewayEvent::Closed { .. } | GatewayEvent::ReconnectRequested | GatewayEvent::InvalidSession { .. } => {
                        if should_reconnect {
                            connection = ConnectionState::Resuming;
                        } else {
                            connection = ConnectionState::Disconnected;
                        }
                    }
                    GatewayEvent::Dispatch { .. } | GatewayEvent::HandlerErrors { .. } | GatewayEvent::HeartbeatSent | GatewayEvent::HeartbeatAck => {}
                }
                if connection != prev_connection {
                    push_event(&events, ClientEvent::ConnectionChanged(connection.clone()));
                }
                publish(&connection, active_token.is_some(), &cache, selected_guild_id, selected_channel_id, &fetched_messages, &error, &snapshot, &events, &seq);
            }
        }
    }
}
