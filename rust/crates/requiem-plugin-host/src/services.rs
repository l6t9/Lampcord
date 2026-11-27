use crate::data::*;
use crate::id::{ChannelId, GuildId, MessageId, RoleId, UserId};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostError {
    pub code: String,
    pub message: String,
}

impl HostError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        HostError {
            code: code.to_string(),
            message: message.into(),
        }
    }

    pub fn unavailable() -> Self {
        Self::new("unavailable", "this client doesn't support that (yet)")
    }

    pub fn denied(permission: &str) -> Self {
        Self::new(
            "permission-denied",
            format!("needs the `{permission}` permission"),
        )
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new("invalid", message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("not-found", message)
    }
}

pub type HostResult<T> = Result<T, HostError>;

fn na<T>() -> HostResult<T> {
    Err(HostError::unavailable())
}

#[allow(unused_variables)]
pub trait HostServices: Send + Sync + 'static {
    fn log(&self, plugin: &str, level: LogLevel, message: &str) {
        let level = match level {
            LogLevel::Trace => log::Level::Trace,
            LogLevel::Debug => log::Level::Debug,
            LogLevel::Info => log::Level::Info,
            LogLevel::Warn => log::Level::Warn,
            LogLevel::Error => log::Level::Error,
        };
        log::log!(target: "plugin", level, "[{plugin}] {message}");
    }

    fn is_discord_host(&self, host: &str) -> bool {
        matches!(
            host,
            "discord.com"
                | "discordapp.com"
                | "ptb.discord.com"
                | "canary.discord.com"
                | "gateway.discord.gg"
                | "discord.gg"
                | "latency.discord.media"
        ) || host.ends_with(".discord.gg")
    }

    fn status_changed(&self, plugin: &str) {}

    fn secret_get(&self, plugin: &str, key: &str) -> HostResult<Option<String>> {
        na()
    }
    fn secret_set(&self, plugin: &str, key: &str, value: &str) -> HostResult<()> {
        na()
    }
    fn secret_delete(&self, plugin: &str, key: &str) -> HostResult<()> {
        na()
    }

    fn client_info(&self) -> String {
        "{}".to_string()
    }
    fn current_user(&self) -> Option<UserData> {
        None
    }
    fn selected_guild(&self) -> Option<GuildId> {
        None
    }
    fn selected_channel(&self) -> Option<ChannelId> {
        None
    }
    fn user(&self, id: UserId) -> Option<UserData> {
        None
    }
    fn guild(&self, id: GuildId) -> Option<GuildData> {
        None
    }
    fn guilds(&self) -> Vec<GuildData> {
        Vec::new()
    }
    fn guild_folders(&self) -> String {
        "[]".to_string()
    }
    fn channel(&self, id: ChannelId) -> Option<ChannelData> {
        None
    }
    fn channels(&self, guild_id: GuildId) -> Vec<ChannelData> {
        Vec::new()
    }
    fn private_channels(&self) -> Vec<ChannelData> {
        Vec::new()
    }
    fn role(&self, guild_id: GuildId, id: RoleId) -> Option<RoleData> {
        self.roles(guild_id).into_iter().find(|r| r.id == id)
    }
    fn roles(&self, guild_id: GuildId) -> Vec<RoleData> {
        Vec::new()
    }
    fn member(&self, guild_id: GuildId, user_id: UserId) -> Option<MemberData> {
        None
    }
    fn members(&self, guild_id: GuildId) -> Vec<MemberData> {
        Vec::new()
    }
    fn user_can(&self, channel_id: ChannelId, user_id: UserId, permission: &str) -> HostResult<bool> {
        na()
    }
    fn messages(&self, channel_id: ChannelId) -> Vec<MessageData> {
        Vec::new()
    }
    fn message(&self, channel_id: ChannelId, id: MessageId) -> Option<MessageData> {
        None
    }
    fn fetch_messages(
        &self,
        channel_id: ChannelId,
        anchor: Option<MessageId>,
        direction: &str,
        limit: u32,
    ) -> HostResult<Vec<MessageData>> {
        na()
    }
    fn fetch_message(&self, channel_id: ChannelId, id: MessageId) -> HostResult<MessageData> {
        na()
    }
    fn search_messages(&self, scope_id: u64, query: &str) -> HostResult<Vec<MessageData>> {
        na()
    }
    fn presence(&self, user_id: UserId) -> Option<PresenceData> {
        None
    }
    fn voice_state(&self, user_id: UserId) -> Option<VoiceStateData> {
        None
    }
    fn voice_states(&self, channel_id: ChannelId) -> Vec<VoiceStateData> {
        Vec::new()
    }
    fn relationships(&self) -> Vec<RelationshipData> {
        Vec::new()
    }
    fn typing(&self, channel_id: ChannelId) -> Vec<UserId> {
        Vec::new()
    }
    fn read_state(&self, channel_id: ChannelId) -> Option<ReadStateData> {
        None
    }
    fn fetch_profile(&self, user_id: UserId, guild_id: Option<GuildId>) -> HostResult<ProfileData> {
        na()
    }
    fn fetch_members(&self, guild_id: GuildId, query: &str, limit: u32) -> HostResult<Vec<MemberData>> {
        na()
    }

    fn send_message(
        &self,
        plugin: &str,
        channel_id: ChannelId,
        content: &str,
        files: Vec<(String, Vec<u8>)>,
        options: SendOptions,
    ) -> HostResult<MessageId> {
        na()
    }
    fn edit_message(
        &self,
        plugin: &str,
        channel_id: ChannelId,
        message_id: MessageId,
        content: &str,
    ) -> HostResult<()> {
        na()
    }
    fn delete_message(&self, plugin: &str, channel_id: ChannelId, message_id: MessageId) -> HostResult<()> {
        na()
    }
    fn pin_message(
        &self,
        plugin: &str,
        channel_id: ChannelId,
        message_id: MessageId,
        pinned: bool,
    ) -> HostResult<()> {
        na()
    }
    fn add_reaction(
        &self,
        plugin: &str,
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: &str,
        burst: bool,
    ) -> HostResult<()> {
        na()
    }
    fn remove_reaction(
        &self,
        plugin: &str,
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: &str,
    ) -> HostResult<()> {
        na()
    }
    fn ack_channel(&self, plugin: &str, channel_id: ChannelId) -> HostResult<()> {
        na()
    }
    fn ack_guild(&self, plugin: &str, guild_id: GuildId) -> HostResult<()> {
        na()
    }
    fn add_local_message(&self, plugin: &str, channel_id: ChannelId, content: &str) -> HostResult<MessageId> {
        na()
    }
    fn remove_local_message(&self, plugin: &str, channel_id: ChannelId, id: MessageId) {}
    fn start_typing(&self, plugin: &str, channel_id: ChannelId) -> HostResult<()> {
        na()
    }
    fn open_dm(&self, plugin: &str, user_id: UserId) -> HostResult<ChannelId> {
        na()
    }
    fn copy_to_clipboard(&self, plugin: &str, text: &str) -> HostResult<()> {
        na()
    }
    fn read_clipboard(&self, plugin: &str) -> HostResult<String> {
        na()
    }

    fn open_url(&self, plugin: &str, url: &str) -> HostResult<()> {
        na()
    }

    fn open_channel(&self, channel_id: ChannelId, message_id: Option<MessageId>) -> HostResult<()> {
        na()
    }
    fn open_guild(&self, guild_id: GuildId) -> HostResult<()> {
        na()
    }
    fn open_home(&self) {}
    fn open_profile(&self, user_id: UserId, guild_id: Option<GuildId>) -> HostResult<()> {
        na()
    }
    fn open_settings(&self, page: &str) -> HostResult<()> {
        na()
    }
    fn open_search(&self, query: &str) -> HostResult<()> {
        na()
    }
    fn open_media(&self, url: &str) -> HostResult<()> {
        na()
    }

    fn discord_request(
        &self,
        plugin: &str,
        method: &str,
        path: &str,
        body: Option<&str>,
    ) -> HostResult<(u16, String)> {
        na()
    }

    fn set_status(&self, plugin: &str, status: &str) -> HostResult<()> {
        na()
    }
    fn set_custom_status(
        &self,
        plugin: &str,
        text: Option<&str>,
        emoji: Option<&str>,
        expires: Option<Timestamp>,
    ) -> HostResult<()> {
        na()
    }

    fn set_activity(
        &self,
        plugin: &str,
        id: &str,
        activity: Option<ActivityData>,
    ) -> HostResult<()> {
        na()
    }

    fn voice_join(&self, plugin: &str, channel_id: ChannelId) -> HostResult<()> {
        na()
    }
    fn voice_leave(&self, plugin: &str) -> HostResult<()> {
        na()
    }
    fn voice_channel(&self) -> Option<ChannelId> {
        None
    }
    fn voice_set_self(&self, plugin: &str, what: &str, on: bool) -> HostResult<()> {
        na()
    }
    fn voice_set_user_volume(&self, plugin: &str, user_id: UserId, volume: f32) -> HostResult<()> {
        na()
    }
    fn voice_move_member(
        &self,
        plugin: &str,
        guild_id: GuildId,
        user_id: UserId,
        channel_id: Option<ChannelId>,
    ) -> HostResult<()> {
        na()
    }
    fn voice_set_member_mute(
        &self,
        plugin: &str,
        guild_id: GuildId,
        user_id: UserId,
        muted: bool,
        deafened: bool,
    ) -> HostResult<()> {
        na()
    }
    fn voice_start_screen_share(&self, plugin: &str) -> HostResult<()> {
        na()
    }
    fn voice_record_message(&self, plugin: &str, channel_id: ChannelId) -> HostResult<()> {
        na()
    }

    fn notify(
        &self,
        plugin: &str,
        title: &str,
        body: &str,
        channel_id: Option<ChannelId>,
        tag: Option<&str>,
    ) -> HostResult<()> {
        na()
    }

    fn play_sound(&self, plugin: &str, source: &str, volume: f32) -> HostResult<u32> {
        na()
    }
    fn stop_sound(&self, plugin: &str, id: u32) {}
    fn override_sound(&self, plugin: &str, name: &str, source: Option<&str>) -> HostResult<()> {
        na()
    }
    fn speak(&self, plugin: &str, text: &str, voice: Option<&str>, rate: f32) -> HostResult<()> {
        na()
    }
    fn known_sounds(&self) -> Vec<String> {
        Vec::new()
    }

    fn known_tweaks(&self) -> Vec<String> {
        Vec::new()
    }
    fn tweak(&self, name: &str) -> Option<String> {
        None
    }

    fn set_tweak(&self, plugin: &str, name: &str, value: Option<&str>) -> HostResult<()> {
        na()
    }

    fn save_file(&self, plugin: &str, suggested_name: &str, data: &[u8]) -> HostResult<bool> {
        na()
    }

    fn pick_files(
        &self,
        plugin: &str,
        multiple: bool,
        filters: &[String],
    ) -> HostResult<Vec<(String, Vec<u8>)>> {
        na()
    }

    fn http_fetch(&self, plugin: &str, request: HttpRequest) -> HostResult<HttpResponse> {
        na()
    }

    fn websocket_connect(
        &self,
        plugin: &str,
        id: u32,
        url: &str,
        protocols: &[String],
        on_event: Box<dyn Fn(&str, Vec<u8>) + Send + Sync>,
    ) -> HostResult<()> {
        na()
    }
    fn websocket_send(
        &self,
        plugin: &str,
        id: u32,
        text: Option<&str>,
        binary: Option<&[u8]>,
    ) -> HostResult<()> {
        na()
    }
    fn websocket_close(&self, plugin: &str, id: u32) {}

    fn overlay(&self, plugin: &str, update: OverlayUpdate) {}

    fn composer_text(&self) -> Option<String> {
        None
    }
    fn composer_set_text(&self, plugin: &str, text: &str) -> HostResult<()> {
        na()
    }
    fn composer_insert(&self, plugin: &str, text: &str) -> HostResult<()> {
        na()
    }
    fn composer_focus(&self) {}
    fn composer_channel(&self) -> Option<ChannelId> {
        None
    }
    fn composer_replying_to(&self) -> Option<MessageId> {
        None
    }
    fn composer_reply_to(
        &self,
        plugin: &str,
        channel_id: ChannelId,
        message_id: MessageId,
        mention: bool,
    ) -> HostResult<()> {
        na()
    }
    fn composer_edit(&self, plugin: &str, channel_id: ChannelId, message_id: MessageId) -> HostResult<()> {
        na()
    }
    fn composer_add_file(&self, plugin: &str, filename: &str, data: Vec<u8>) -> HostResult<()> {
        na()
    }

    fn refresh_view(&self, plugin: &str, view_id: &str) {}
    fn toast(&self, plugin: &str, text: &str, kind: &str) {}

    fn open_surface(
        &self,
        plugin: &str,
        surface: &str,
        view_id: &str,
        title: &str,
        extra: &str,
    ) -> HostResult<()> {
        na()
    }
    fn close_surface(&self, plugin: &str, surface: &str, view_id: &str) {}
    fn show_notice(
        &self,
        plugin: &str,
        id: &str,
        text: &str,
        button: Option<&str>,
        action_id: Option<&str>,
    ) -> HostResult<()> {
        na()
    }
    fn dismiss_notice(&self, plugin: &str, id: &str) {}
}
