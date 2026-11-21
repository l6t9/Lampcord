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

pub use error::{FfiError, FfiResult};
pub use runtime::RuntimeHandle;
pub use snapshot::{ChannelSnapshot, ClientSnapshot, GuildSnapshot, MessageSnapshot};
pub use types::{ClientCommand, ClientConfig, ClientEvent, ConnectionState};

/// High-level client handle for Kotlin/Swift/desktop frontends.
///
/// Internally this owns a background Tokio runtime and a command-driven worker.
/// The API is intentionally poll/snapshot based so it can be wrapped as
/// Kotlin Flow / Swift AsyncSequence later.
#[derive(Clone)]
pub struct LampcordClient {
    runtime: RuntimeHandle,
}

impl LampcordClient {
    /// Starts the background runtime worker.
    pub fn start(config: ClientConfig) -> Self {
        Self {
            runtime: runtime::start(config),
        }
    }

    /// Sends a command to the background worker.
    /// Returns `false` if the worker is no longer running.
    pub fn send(&self, command: ClientCommand) -> bool {
        self.runtime.send(command)
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
    pub fn connect_token(&self, token: impl Into<String>) -> bool {
        self.send(ClientCommand::ConnectToken {
            token: token.into(),
        })
    }

    /// Convenience: request shutdown.
    pub fn shutdown(&self) -> bool {
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
        assert!(client.shutdown());
    }

    #[test]
    fn select_channel_updates_snapshot() {
        let client = LampcordClient::start(ClientConfig::default());
        assert!(client.send(ClientCommand::SelectGuild { guild_id: Some(1) }));
        assert!(client.send(ClientCommand::SelectChannel {
            channel_id: Some(2)
        }));
        // Give the worker a moment to process commands.
        std::thread::sleep(std::time::Duration::from_millis(50));
        let snap = client.snapshot();
        assert_eq!(snap.selected_guild_id, Some(1));
        assert_eq!(snap.selected_channel_id, Some(2));
        let _ = client.shutdown();
    }
}
