use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use rand::rngs::OsRng;
use rsa::pkcs8::EncodePublicKey;
use rsa::{Oaep, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::Arc;
use tokio::sync::{Mutex, watch};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;

use crate::{
    api::{self, ApiError},
    rest::RestConfig,
    rest_client::RestClient,
    traits::{CaptchaSolver, RestTransport},
};

pub const REMOTE_AUTH_GATEWAY_URL: &str = "wss://remote-auth-gateway.discord.gg/?v=2";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteAuthPayload {
    pub op: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heartbeat_interval: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoded_public_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encrypted_nonce: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encrypted_user_payload: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<RemoteUserPayload>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticket: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encrypted_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemoteUserPayload {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteAuthState {
    Idle,
    Connecting,
    QrReady { url: String },
    UserScanned { user: RemoteUserPayload },
    Finished { token: String },
    Error { message: String },
}

impl Default for RemoteAuthState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Clone)]
pub struct RemoteAuthKeyPair {
    public_spki_der: Vec<u8>,
    private_key: RsaPrivateKey,
}

impl RemoteAuthKeyPair {
    pub fn generate() -> Result<Self, String> {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048).map_err(|e| e.to_string())?;
        let public_key = RsaPublicKey::from(&private_key);
        let public_spki_der = public_key
            .to_public_key_der()
            .map_err(|e| e.to_string())?
            .as_bytes()
            .to_vec();
        Ok(Self {
            public_spki_der,
            private_key,
        })
    }

    pub fn public_key_base64(&self) -> String {
        base64::engine::general_purpose::STANDARD.encode(&self.public_spki_der)
    }

    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, String> {
        let padding = Oaep::new::<Sha256>();
        self.private_key
            .decrypt(padding, encrypted_data)
            .map_err(|e| e.to_string())
    }
}

fn decode_b64(input: &str) -> Result<Vec<u8>, String> {
    base64::engine::general_purpose::STANDARD
        .decode(input)
        .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(input))
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(input))
        .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(input))
        .map_err(|e| e.to_string())
}

fn encode_nonce_proof(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn decrypt_token(keypair: &RemoteAuthKeyPair, encrypted_b64: &str) -> Option<String> {
    let encrypted = decode_b64(encrypted_b64).ok()?;
    let decrypted = keypair.decrypt(&encrypted).ok()?;
    let token = String::from_utf8(decrypted).ok()?;
    let token = token.trim().to_owned();
    if token.is_empty() { None } else { Some(token) }
}

async fn exchange_ticket<T, S>(
    rest_client: &RestClient<T, S>,
    rest_config: &RestConfig,
    ticket: &str,
) -> Result<Option<String>, ApiError>
where
    T: RestTransport,
    S: CaptchaSolver,
{
    api::auth::exchange_remote_auth_ticket(rest_client, rest_config, ticket).await
}

/// Run the Discord remote auth (QR login) gateway flow.
///
/// Updates `state_tx` as the flow progresses. Returns when the connection ends or is cancelled.
/// The caller is responsible for spawning/cancelling this task and for applying the final token.
pub async fn run_remote_auth<T, S>(
    rest_client: Arc<RestClient<T, S>>,
    rest_config: RestConfig,
    state_tx: watch::Sender<RemoteAuthState>,
    mut cancel_rx: tokio::sync::oneshot::Receiver<()>,
) where
    T: RestTransport + Send + Sync + 'static,
    S: CaptchaSolver + Send + Sync + 'static,
{
    let _ = state_tx.send(RemoteAuthState::Connecting);
    let keypair = match RemoteAuthKeyPair::generate() {
        Ok(k) => Arc::new(k),
        Err(e) => {
            let _ = state_tx.send(RemoteAuthState::Error {
                message: format!("Failed to generate keypair: {e}"),
            });
            return;
        }
    };

    let mut request = match REMOTE_AUTH_GATEWAY_URL.into_client_request() {
        Ok(r) => r,
        Err(e) => {
            let _ = state_tx.send(RemoteAuthState::Error {
                message: format!("Failed to build request: {e}"),
            });
            return;
        }
    };
    let headers = request.headers_mut();
    headers.insert("Origin", HeaderValue::from_static("https://discord.com"));
    headers.insert(
        "Accept-Language",
        HeaderValue::from_static("en-US,en;q=0.9"),
    );
    headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));
    headers.insert(
        "User-Agent",
        HeaderValue::from_static(
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) discord/0.0.398 Chrome/138.0.7204.251 Electron/37.6.0 Safari/537.36",
        ),
    );

    let ws_stream = match tokio_tungstenite::connect_async(request).await {
        Ok((stream, _)) => stream,
        Err(e) => {
            let _ = state_tx.send(RemoteAuthState::Error {
                message: format!("Connection failed: {e}"),
            });
            return;
        }
    };

    let (write, mut read) = ws_stream.split();
    let heartbeat_interval_ms: Arc<Mutex<Option<u64>>> = Arc::new(Mutex::new(None));
    let mut heartbeat_task: Option<tokio::task::JoinHandle<()>> = None;

    // Helper: send payload
    let send_payload = |payload: &RemoteAuthPayload| -> Result<Message, String> {
        serde_json::to_string(payload)
            .map(|s| Message::Text(s.into()))
            .map_err(|e| e.to_string())
    };

    // Shared writer
    let write = Arc::new(Mutex::new(write));

    loop {
        tokio::select! {
            _ = &mut cancel_rx => {
                if let Some(task) = heartbeat_task.take() { task.abort(); }
                let _ = state_tx.send(RemoteAuthState::Idle);
                return;
            }
            msg = read.next() => {
                let Some(msg) = msg else {
                    break;
                };
                let msg = match msg {
                    Ok(m) => m,
                    Err(e) => {
                        if let Some(task) = heartbeat_task.take() { task.abort(); }
                        let current = state_tx.borrow().clone();
                        if !matches!(current, RemoteAuthState::Finished { .. }) {
                            let _ = state_tx.send(RemoteAuthState::Error {
                                message: format!("Gateway error: {e}"),
                            });
                        }
                        return;
                    }
                };
                let text = match msg {
                    Message::Text(t) => t.to_string(),
                    Message::Close(_) => break,
                    Message::Ping(_) | Message::Pong(_) | Message::Binary(_) | Message::Frame(_) => continue,
                };
                let payload: RemoteAuthPayload = match serde_json::from_str(&text) {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                match payload.op.as_str() {
                    "hello" => {
                        if let Some(interval) = payload.heartbeat_interval {
                            *heartbeat_interval_ms.lock().await = Some(interval);
                            // start heartbeat
                            if let Some(task) = heartbeat_task.take() { task.abort(); }
                            let write_hb = Arc::clone(&write);
                            let interval_dur = tokio::time::Duration::from_millis(interval);
                            heartbeat_task = Some(tokio::spawn(async move {
                                loop {
                                    tokio::time::sleep(interval_dur).await;
                                    let hb = RemoteAuthPayload { op: "heartbeat".to_owned(), ..Default::default() };
                                    let Ok(text) = serde_json::to_string(&hb) else { break; };
                                    let mut w = write_hb.lock().await;
                                    if w.send(Message::Text(text.into())).await.is_err() { break; }
                                }
                            }));
                        }
                        let init = RemoteAuthPayload {
                            op: "init".to_owned(),
                            encoded_public_key: Some(keypair.public_key_base64()),
                            ..Default::default()
                        };
                        if let Ok(msg) = send_payload(&init) {
                            let mut w = write.lock().await;
                            if w.send(msg).await.is_err() { break; }
                        }
                    }
                    "nonce_proof" => {
                        let Some(encrypted_nonce_b64) = payload.encrypted_nonce.as_deref() else { continue; };
                        let Ok(encrypted_nonce) = decode_b64(encrypted_nonce_b64) else { continue; };
                        let Ok(decrypted_nonce) = keypair.decrypt(&encrypted_nonce) else { continue; };
                        let nonce = encode_nonce_proof(&decrypted_nonce);
                        let proof = RemoteAuthPayload {
                            op: "nonce_proof".to_owned(),
                            nonce: Some(nonce),
                            ..Default::default()
                        };
                        if let Ok(msg) = send_payload(&proof) {
                            let mut w = write.lock().await;
                            if w.send(msg).await.is_err() { break; }
                        }
                    }
                    "pending_remote_init" => {
                        if let Some(fingerprint) = payload.fingerprint {
                            let _ = state_tx.send(RemoteAuthState::QrReady {
                                url: format!("https://discord.com/ra/{fingerprint}"),
                            });
                        }
                    }
                    "pending_ticket" => {
                        let Some(encrypted_user_b64) = payload.encrypted_user_payload.as_deref() else { continue; };
                        let Ok(encrypted_user) = decode_b64(encrypted_user_b64) else { continue; };
                        let Ok(decrypted_user) = keypair.decrypt(&encrypted_user) else { continue; };
                        let Ok(user_str) = String::from_utf8(decrypted_user) else { continue; };
                        let parts: Vec<&str> = user_str.split(':').collect();
                        if parts.len() >= 4 {
                            let user = RemoteUserPayload {
                                id: parts[0].to_owned(),
                                discriminator: parts[1].to_owned(),
                                avatar: {
                                    let a = parts[2];
                                    if a.is_empty() || a == "0" { None } else { Some(a.to_owned()) }
                                },
                                username: parts[3].to_owned(),
                                global_name: None,
                            };
                            let _ = state_tx.send(RemoteAuthState::UserScanned { user });
                        }
                    }
                    "pending_login" => {
                        let Some(ticket) = payload.ticket else { continue; };
                        // exchange ticket via REST and decrypt token
                        match exchange_ticket(&rest_client, &rest_config, &ticket).await {
                            Ok(Some(encrypted_token)) => {
                                if let Some(token) = decrypt_token(&keypair, &encrypted_token) {
                                    if let Some(task) = heartbeat_task.take() { task.abort(); }
                                    let _ = state_tx.send(RemoteAuthState::Finished { token });
                                    return;
                                } else {
                                    let _ = state_tx.send(RemoteAuthState::Error {
                                        message: "Failed to decrypt token".to_owned(),
                                    });
                                }
                            }
                            Ok(None) => {
                                let _ = state_tx.send(RemoteAuthState::Error {
                                    message: "Failed to exchange ticket".to_owned(),
                                });
                            }
                            Err(e) => {
                                let _ = state_tx.send(RemoteAuthState::Error {
                                    message: format!("Ticket exchange failed: {e}"),
                                });
                            }
                        }
                    }
                    "finish" => {
                        let Some(encrypted_token_b64) = payload.encrypted_token.as_deref() else { continue; };
                        if let Some(task) = heartbeat_task.take() { task.abort(); }
                        if let Some(token) = decrypt_token(&keypair, encrypted_token_b64) {
                            let _ = state_tx.send(RemoteAuthState::Finished { token });
                        } else {
                            let _ = state_tx.send(RemoteAuthState::Error {
                                message: "Failed to decrypt token".to_owned(),
                            });
                        }
                        return;
                    }
                    "cancel" => {
                        if let Some(task) = heartbeat_task.take() { task.abort(); }
                        let _ = state_tx.send(RemoteAuthState::Error {
                            message: "Canceled by user".to_owned(),
                        });
                        return;
                    }
                    _ => {}
                }
            }
        }
    }

    if let Some(task) = heartbeat_task.take() {
        task.abort();
    }
    let current = state_tx.borrow().clone();
    if !matches!(
        current,
        RemoteAuthState::Finished { .. } | RemoteAuthState::Error { .. } | RemoteAuthState::Idle
    ) {
        let _ = state_tx.send(RemoteAuthState::Error {
            message: "This QR code expired or could not connect. Try again.".to_owned(),
        });
    }
}
