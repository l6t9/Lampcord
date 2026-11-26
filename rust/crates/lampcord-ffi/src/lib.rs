#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();

pub mod c_api;
pub mod error;
pub mod runtime;
pub mod snapshot;
pub mod storage;
pub mod types;

use std::sync::Arc;

pub use error::{FfiClientError, FfiError, FfiErrorKind, FfiResult};
pub use runtime::RuntimeHandle;
pub use snapshot::{
    AttachmentSnapshot, AuthStateSnapshot, BadgeSnapshot, BanSnapshot, ChannelSnapshot,
    ClientSnapshot, EmojiSnapshot, GuildFolderSnapshot, GuildSnapshot, MEMBER_LIST_ROW_GROUP,
    MEMBER_LIST_ROW_MEMBER, MemberListSnapshot, MemberSnapshot, MessageSnapshot, PresenceSnapshot,
    PrivateChannelSnapshot, ProfileSnapshot, ReactionSnapshot, ReadStateSnapshot,
    RecipientSnapshot, RelationshipSnapshot, ReplySnapshot, RoleSnapshot, SNAPSHOT_SCHEMA_VERSION,
    SavedAccountSnapshot, SnapshotMetaPatch, SnapshotPatch, SnapshotVersions, StickerSnapshot,
    ThreadSnapshot,
};
pub use storage::{ForeignSecretStore, PlatformSecretStore};
pub use types::{
    AuthChallenge, AuthFailureKind, ClientCommand, ClientConfig, ClientEvent, ClientEventEnvelope,
    ConnectionState, RemoteAuthState,
};

#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn snapshot_schema_version() -> u32 {
    SNAPSHOT_SCHEMA_VERSION
}

#[derive(Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct LampcordClient {
    runtime: RuntimeHandle,
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
impl LampcordClient {
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    pub fn start(config: ClientConfig) -> Self {
        Self {
            runtime: runtime::start(config, None),
        }
    }

    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    pub fn start_with_store(config: ClientConfig, store: Arc<dyn PlatformSecretStore>) -> Self {
        Self {
            runtime: runtime::start(config, Some(Arc::new(ForeignSecretStore::new(store)))),
        }
    }

    pub fn send(&self, command: ClientCommand) -> Result<(), FfiClientError> {
        if self.runtime.send(command) {
            Ok(())
        } else {
            Err(FfiClientError::not_connected())
        }
    }

    pub fn snapshot(&self) -> ClientSnapshot {
        self.runtime.snapshot()
    }

    pub fn snapshot_if_newer(&self, since: u64) -> Option<ClientSnapshot> {
        self.runtime.snapshot_if_newer(since)
    }

    pub fn snapshot_versions(&self) -> SnapshotVersions {
        self.runtime.snapshot_versions()
    }

    pub fn snapshot_patch(&self, since: SnapshotVersions) -> SnapshotPatch {
        self.runtime.snapshot_patch(since)
    }

    pub fn drain_events(&self) -> Vec<ClientEventEnvelope> {
        self.runtime.drain_events()
    }

    pub fn connect_token(&self, token: String) -> Result<(), FfiClientError> {
        if token.trim().is_empty() {
            return Err(FfiClientError::invalid_input("token must not be empty"));
        }
        self.send(ClientCommand::ConnectToken { token })
    }

    pub fn connect_stored_token(&self) -> Result<(), FfiClientError> {
        self.send(ClientCommand::ConnectStoredToken)
    }

    pub fn login_password(
        &self,
        login: String,
        password: String,
        captcha_key: Option<String>,
    ) -> Result<(), FfiClientError> {
        if login.trim().is_empty() || password.is_empty() {
            return Err(FfiClientError::invalid_input(
                "login and password must not be empty",
            ));
        }
        self.send(ClientCommand::LoginPassword {
            login,
            password,
            captcha_key,
        })
    }

    pub fn submit_mfa(
        &self,
        code: String,
        ticket: String,
        method: String,
    ) -> Result<(), FfiClientError> {
        let code = code.trim().to_owned();
        let method = method.trim().to_owned();
        if code.is_empty() || ticket.trim().is_empty() || method.is_empty() {
            return Err(FfiClientError::invalid_input(
                "code, ticket and method must not be empty",
            ));
        }
        self.send(ClientCommand::SubmitMfa {
            code,
            ticket,
            method,
        })
    }

    pub fn send_mfa_sms(&self, ticket: String) -> Result<(), FfiClientError> {
        if ticket.trim().is_empty() {
            return Err(FfiClientError::invalid_input("ticket must not be empty"));
        }
        self.send(ClientCommand::SendMfaSms { ticket })
    }

    pub fn forgot_password(&self, login: String) -> Result<(), FfiClientError> {
        if login.trim().is_empty() {
            return Err(FfiClientError::invalid_input("login must not be empty"));
        }
        self.send(ClientCommand::ForgotPassword { login })
    }

    pub fn start_remote_auth(&self) -> Result<(), FfiClientError> {
        self.send(ClientCommand::StartRemoteAuth)
    }

    pub fn cancel_remote_auth(&self) -> Result<(), FfiClientError> {
        self.send(ClientCommand::CancelRemoteAuth)
    }

    pub fn select_guild(&self, guild_id: Option<u64>) -> Result<(), FfiClientError> {
        self.send(ClientCommand::SelectGuild { guild_id })
    }

    pub fn select_channel(&self, channel_id: Option<u64>) -> Result<(), FfiClientError> {
        self.send(ClientCommand::SelectChannel { channel_id })
    }

    pub fn set_member_list_ranges(&self, ranges: Vec<Vec<u32>>) -> Result<(), FfiClientError> {
        self.send(ClientCommand::SetMemberListRanges { ranges })
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

    pub fn fetch_mentions(
        &self,
        limit: Option<u8>,
        before: Option<u64>,
    ) -> Result<(), FfiClientError> {
        self.send(ClientCommand::FetchMentions { limit, before })
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

    pub fn fetch_profile(&self, user_id: u64, guild_id: Option<u64>) -> Result<(), FfiClientError> {
        self.send(ClientCommand::FetchProfile { user_id, guild_id })
    }

    pub fn set_custom_status(&self, text: Option<String>) -> Result<(), FfiClientError> {
        self.send(ClientCommand::SetCustomStatus { text })
    }

    pub fn fetch_pins(&self, channel_id: u64) -> Result<(), FfiClientError> {
        self.send(ClientCommand::FetchPins { channel_id })
    }

    pub fn send_attachments(
        &self,
        channel_id: u64,
        content: String,
        file_paths: Vec<String>,
        reply_to_message_id: Option<u64>,
        reply_to_guild_id: Option<u64>,
    ) -> Result<(), FfiClientError> {
        if file_paths.is_empty() {
            return Err(FfiClientError::invalid_input(
                "file_paths must not be empty",
            ));
        }
        self.send(ClientCommand::SendAttachments {
            channel_id,
            content,
            file_paths,
            reply_to_message_id,
            reply_to_guild_id,
        })
    }

    pub fn edit_message(
        &self,
        channel_id: u64,
        message_id: u64,
        content: String,
    ) -> Result<(), FfiClientError> {
        self.send(ClientCommand::EditMessage {
            channel_id,
            message_id,
            content,
        })
    }

    pub fn delete_message(&self, channel_id: u64, message_id: u64) -> Result<(), FfiClientError> {
        self.send(ClientCommand::DeleteMessage {
            channel_id,
            message_id,
        })
    }

    pub fn add_reaction(
        &self,
        channel_id: u64,
        message_id: u64,
        emoji: String,
    ) -> Result<(), FfiClientError> {
        self.send(ClientCommand::AddReaction {
            channel_id,
            message_id,
            emoji,
        })
    }

    pub fn remove_reaction(
        &self,
        channel_id: u64,
        message_id: u64,
        emoji: String,
    ) -> Result<(), FfiClientError> {
        self.send(ClientCommand::RemoveReaction {
            channel_id,
            message_id,
            emoji,
        })
    }

    pub fn send_typing(&self, channel_id: u64) -> Result<(), FfiClientError> {
        self.send(ClientCommand::SendTyping { channel_id })
    }

    pub fn set_status(&self, status: String) -> Result<(), FfiClientError> {
        self.send(ClientCommand::SetStatus { status })
    }

    pub fn ack_message(&self, channel_id: u64, message_id: u64) -> Result<(), FfiClientError> {
        self.send(ClientCommand::AckMessage {
            channel_id,
            message_id,
        })
    }

    pub fn mark_guild_read(&self, guild_id: u64) -> Result<(), FfiClientError> {
        self.send(ClientCommand::MarkGuildRead { guild_id })
    }

    pub fn pin_message(&self, channel_id: u64, message_id: u64) -> Result<(), FfiClientError> {
        self.send(ClientCommand::PinMessage {
            channel_id,
            message_id,
        })
    }

    pub fn unpin_message(&self, channel_id: u64, message_id: u64) -> Result<(), FfiClientError> {
        self.send(ClientCommand::UnpinMessage {
            channel_id,
            message_id,
        })
    }

    pub fn join_guild(&self, invite_code: String) -> Result<(), FfiClientError> {
        self.send(ClientCommand::JoinGuild { invite_code })
    }

    pub fn leave_guild(&self, guild_id: u64) -> Result<(), FfiClientError> {
        self.send(ClientCommand::LeaveGuild { guild_id })
    }

    pub fn send_friend_request(&self, username: String) -> Result<(), FfiClientError> {
        self.send(ClientCommand::SendFriendRequest { username })
    }

    pub fn accept_friend_request(&self, user_id: u64) -> Result<(), FfiClientError> {
        self.send(ClientCommand::AcceptFriendRequest { user_id })
    }

    pub fn remove_relationship(&self, user_id: u64) -> Result<(), FfiClientError> {
        self.send(ClientCommand::RemoveRelationship { user_id })
    }

    pub fn block_user(&self, user_id: u64) -> Result<(), FfiClientError> {
        self.send(ClientCommand::BlockUser { user_id })
    }

    pub fn open_dm(&self, user_id: u64) -> Result<(), FfiClientError> {
        self.send(ClientCommand::OpenDm { user_id })
    }

    pub fn search_messages(
        &self,
        guild_id: Option<u64>,
        channel_id: Option<u64>,
        query: String,
    ) -> Result<(), FfiClientError> {
        self.send(ClientCommand::SearchMessages {
            guild_id,
            channel_id,
            query,
        })
    }

    pub fn fetch_threads(&self, guild_id: u64, channel_id: u64) -> Result<(), FfiClientError> {
        self.send(ClientCommand::FetchThreads {
            guild_id,
            channel_id,
        })
    }

    pub fn create_thread(
        &self,
        channel_id: u64,
        name: String,
        message_id: Option<u64>,
    ) -> Result<(), FfiClientError> {
        self.send(ClientCommand::CreateThread {
            channel_id,
            name,
            message_id,
        })
    }

    pub fn fetch_guild_settings(&self, guild_id: u64) -> Result<(), FfiClientError> {
        self.send(ClientCommand::FetchGuildSettings { guild_id })
    }

    pub fn create_guild_role(
        &self,
        guild_id: u64,
        name: String,
        color: u32,
    ) -> Result<(), FfiClientError> {
        self.send(ClientCommand::CreateGuildRole {
            guild_id,
            name,
            color,
        })
    }

    pub fn delete_guild_role(&self, guild_id: u64, role_id: u64) -> Result<(), FfiClientError> {
        self.send(ClientCommand::DeleteGuildRole { guild_id, role_id })
    }

    pub fn delete_guild_emoji(&self, guild_id: u64, emoji_id: u64) -> Result<(), FfiClientError> {
        self.send(ClientCommand::DeleteGuildEmoji { guild_id, emoji_id })
    }

    pub fn unban_member(&self, guild_id: u64, user_id: u64) -> Result<(), FfiClientError> {
        self.send(ClientCommand::UnbanMember { guild_id, user_id })
    }

    pub fn create_channel(
        &self,
        guild_id: u64,
        name: String,
        kind: u8,
        parent_id: Option<u64>,
    ) -> Result<(), FfiClientError> {
        self.send(ClientCommand::CreateChannel {
            guild_id,
            name,
            kind,
            parent_id,
        })
    }

    pub fn delete_channel(&self, channel_id: u64) -> Result<(), FfiClientError> {
        self.send(ClientCommand::DeleteChannel { channel_id })
    }

    pub fn switch_account(&self, user_id: String) -> Result<(), FfiClientError> {
        self.send(ClientCommand::SwitchAccount { user_id })
    }

    pub fn remove_saved_account(&self, user_id: String) -> Result<(), FfiClientError> {
        self.send(ClientCommand::RemoveSavedAccount { user_id })
    }

    pub fn logout(&self) -> Result<(), FfiClientError> {
        self.send(ClientCommand::Logout)
    }

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
        let snapshot = client.snapshot();
        assert_eq!(snapshot.connection, ConnectionState::Disconnected);
        assert_eq!(snapshot.schema_version, SNAPSHOT_SCHEMA_VERSION);
        assert!(client.shutdown().is_ok());
    }

    #[test]
    fn snapshot_if_newer_returns_only_changed_snapshots() {
        let client = LampcordClient::start(ClientConfig::default());
        let initial = client.snapshot();
        assert!(client.snapshot_if_newer(initial.seq).is_none());

        assert!(client.select_guild(Some(7)).is_ok());
        std::thread::sleep(std::time::Duration::from_millis(50));
        let updated = client
            .snapshot_if_newer(initial.seq)
            .expect("snapshot after change");
        assert_eq!(updated.selected_guild_id, Some(7));
        assert!(client.snapshot_if_newer(updated.seq).is_none());
        let _ = client.shutdown();
    }

    #[test]
    fn snapshot_patch_contains_only_changed_sections() {
        let client = LampcordClient::start(ClientConfig::default());
        let before = client.snapshot_versions();
        let unchanged = client.snapshot_patch(before.clone());
        assert!(unchanged.meta.is_none());
        assert!(unchanged.guilds.is_none());
        assert!(unchanged.messages.is_none());

        assert!(client.select_guild(Some(11)).is_ok());
        std::thread::sleep(std::time::Duration::from_millis(50));
        let after = client.snapshot_versions();
        assert!(after.seq > before.seq);
        assert!(after.meta > before.meta);
        assert_eq!(after.guilds, before.guilds);

        let patch = client.snapshot_patch(before);
        assert_eq!(patch.seq, after.seq);
        assert!(patch.guilds.is_none());
        assert!(patch.channels.is_none());
        let meta = patch.meta.expect("meta changed");
        assert_eq!(meta.selected_guild_id, Some(11));
        let _ = client.shutdown();
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
        std::thread::sleep(std::time::Duration::from_millis(50));
        let snapshot = client.snapshot();
        assert_eq!(snapshot.selected_guild_id, Some(1));
        assert_eq!(snapshot.selected_channel_id, Some(2));
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
        assert!(client.snapshot().has_token);
        let _ = client.shutdown();
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn events_are_ordered_and_snapshots_coalesced() {
        let client = LampcordClient::start(ClientConfig::default());
        for guild_id in 1..=3 {
            assert!(client.select_guild(Some(guild_id)).is_ok());
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
        let events = client.drain_events();
        assert!(!events.is_empty());
        assert!(
            events
                .windows(2)
                .all(|pair| pair[0].sequence < pair[1].sequence)
        );
        assert!(events.iter().all(|envelope| envelope.emitted_at_ms > 0));
        assert!(
            events
                .iter()
                .filter(|envelope| matches!(envelope.event, ClientEvent::SnapshotUpdated))
                .count()
                <= 1
        );
        let _ = client.shutdown();
    }

    #[test]
    fn invalid_auth_input_is_rejected_before_dispatch() {
        let client = LampcordClient::start(ClientConfig::default());
        let error = client.connect_token(String::new()).unwrap_err();
        assert_eq!(error.kind(), FfiErrorKind::InvalidInput);
        let error = client
            .login_password(String::new(), "pw".to_owned(), None)
            .unwrap_err();
        assert_eq!(error.kind(), FfiErrorKind::InvalidInput);
        let _ = client.shutdown();
    }
}
