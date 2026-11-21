//! UI-agnostic facade for embedding Lampcord core in non-Rust frontends.
//!
//! This crate is the foundation for Option A (Kotlin/Compose UI + Rust core).
//! It exposes simple, ownership-friendly types that can be adapted to
//! UniFFI/cbindgen bindings.

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();

pub mod error;
pub mod runtime;
pub mod snapshot;
pub mod types;

pub use error::{FfiClientError, FfiError, FfiErrorKind, FfiResult};
pub use runtime::RuntimeHandle;
pub use snapshot::{ChannelSnapshot, ClientSnapshot, GuildSnapshot, MessageSnapshot};
pub use types::{ClientCommand, ClientConfig, ClientEvent, ConnectionState};

/// High-level client handle for Kotlin/Swift/desktop frontends.
///
/// Internally this owns a background Tokio runtime and a command-driven worker.
/// The API is intentionally poll/snapshot based so it can be wrapped as
/// Kotlin Flow / Swift AsyncSequence later.
#[derive(Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct LampcordClient {
    runtime: RuntimeHandle,
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
impl LampcordClient {
    /// Starts the background runtime worker.
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    pub fn start(config: ClientConfig) -> Self {
        Self {
            runtime: runtime::start(config),
        }
    }

    /// Sends a command to the background worker.
    pub fn send(&self, command: ClientCommand) -> Result<(), FfiClientError> {
        if self.runtime.send(command) {
            Ok(())
        } else {
            Err(FfiClientError::not_connected())
        }
    }

    /// Returns the latest snapshot.
    pub fn snapshot(&self) -> ClientSnapshot {
        self.runtime.snapshot()
    }

    /// Drains pending events since the last call.
    pub fn drain_events(&self) -> Vec<ClientEvent> {
        self.runtime.drain_events()
    }

    /// Convenience: connect with a token.
    pub fn connect_token(&self, token: String) -> Result<(), FfiClientError> {
        self.send(ClientCommand::ConnectToken { token })
    }

    /// Connect using the first stored token from storage_path, if available.
    pub fn connect_stored_token(&self) -> Result<(), FfiClientError> {
        self.send(ClientCommand::ConnectStoredToken)
    }

    pub fn select_guild(&self, guild_id: Option<u64>) -> Result<(), FfiClientError> {
        self.send(ClientCommand::SelectGuild { guild_id })
    }

    pub fn select_channel(&self, channel_id: Option<u64>) -> Result<(), FfiClientError> {
        self.send(ClientCommand::SelectChannel { channel_id })
    }

    pub fn fetch_messages(
        &self,
        channel_id: u64,
        before: Option<u64>,
        limit: Option<u8>,
    ) -> Result<(), FfiClientError> {
        self.send(ClientCommand::FetchMessages {
            channel_id,
            before,
            limit,
        })
    }

    pub fn send_message(
        &self,
        channel_id: u64,
        content: String,
        reply_to_message_id: Option<u64>,
        reply_to_guild_id: Option<u64>,
    ) -> Result<(), FfiClientError> {
        self.send(ClientCommand::SendMessage {
            channel_id,
            content,
            reply_to_message_id,
            reply_to_guild_id,
        })
    }

    pub fn send_typing(&self, channel_id: u64) -> Result<(), FfiClientError> {
        self.send(ClientCommand::SendTyping { channel_id })
    }

    pub fn logout(&self) -> Result<(), FfiClientError> {
        self.send(ClientCommand::Logout)
    }

    /// Convenience: request shutdown.
    pub fn shutdown(&self) -> Result<(), FfiClientError> {
        self.send(ClientCommand::Shutdown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_starts_and_shuts_down() {
        let client = LampcordClient::start(ClientConfig::default());
        let snap = client.snapshot();
        assert_eq!(snap.connection, ConnectionState::Disconnected);
        assert!(client.shutdown().is_ok());
    }

    #[test]
    fn select_channel_updates_snapshot() {
        let client = LampcordClient::start(ClientConfig::default());
        assert!(
            client
                .send(ClientCommand::SelectGuild { guild_id: Some(1) })
                .is_ok()
        );
        assert!(
            client
                .send(ClientCommand::SelectChannel {
                    channel_id: Some(2)
                })
                .is_ok()
        );
        // Give the worker a moment to process commands.
        std::thread::sleep(std::time::Duration::from_millis(50));
        let snap = client.snapshot();
        assert_eq!(snap.selected_guild_id, Some(1));
        assert_eq!(snap.selected_channel_id, Some(2));
        let _ = client.shutdown();
    }

    #[test]
    fn stored_token_sets_has_token() {
        let path = std::env::temp_dir().join(format!(
            "lampcord-ffi-test-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, r#"[{"token":"abc","user_id":"1"}]"#).unwrap();

        let config = ClientConfig {
            storage_path: Some(path.to_string_lossy().into_owned()),
            ..ClientConfig::default()
        };
        let client = LampcordClient::start(config);
        std::thread::sleep(std::time::Duration::from_millis(50));
        let snap = client.snapshot();
        assert!(snap.has_token);
        let _ = client.shutdown();
        let _ = std::fs::remove_file(path);
    }
}
