use crate::models::id::marker::{ChannelMarker, GuildMarker, UserMarker};
use crate::models::{Channel, Id};
use crate::state::AppState;
use crate::utils::voice_settings;
#[cfg(feature = "voice")]
use crate::voice::audio;
#[cfg(feature = "voice")]
use crate::voice::capture::preview;
#[cfg(feature = "voice")]
use crate::voice::local_mute;
#[cfg(feature = "voice")]
use crate::voice::video::frames;
#[cfg(feature = "voice")]
use crate::voice;
#[cfg(feature = "voice")]
use std::sync;

pub fn join(state: &AppState, guild_id: Id<GuildMarker>, channel: &Channel) -> bool {
    #[cfg(feature = "voice")]
    {
        if voice::current_channel() == Some((guild_id.get(), channel.id.get())) {
            return false;
        }
        voice::join_channel(state, guild_id.get(), channel)
    }
    #[cfg(not(feature = "voice"))]
    {
        let _ = (state, guild_id, channel);
        false
    }
}

pub fn disconnect() {
    #[cfg(feature = "voice")]
    voice::leave();
}

pub fn set_camera(on: bool) {
    #[cfg(feature = "voice")]
    voice::set_camera(on);
    #[cfg(not(feature = "voice"))]
    let _ = on;
}

pub fn camera_on() -> bool {
    #[cfg(feature = "voice")]
    return voice::is_camera_on();
    #[cfg(not(feature = "voice"))]
    false
}

pub const AVAILABLE: bool = cfg!(feature = "voice");

pub fn request_channel_info(guild_id: Id<GuildMarker>) {
    #[cfg(feature = "voice")]
    voice::request_channel_info(guild_id.get());
    #[cfg(not(feature = "voice"))]
    let _ = guild_id;
}

pub fn current_channel() -> Option<Id<ChannelMarker>> {
    #[cfg(feature = "voice")]
    return voice::current_call().map(|call| Id::new(call.channel));
    #[cfg(not(feature = "voice"))]
    None
}

pub fn is_locally_muted(user_id: Id<UserMarker>) -> bool {
    #[cfg(feature = "voice")]
    return local_mute::is_muted(user_id.get());
    #[cfg(not(feature = "voice"))]
    {
        let _ = user_id;
        false
    }
}

pub fn set_locally_muted(user_id: Id<UserMarker>, muted: bool) {
    #[cfg(feature = "voice")]
    local_mute::set_muted(user_id.get(), muted);
    #[cfg(not(feature = "voice"))]
    let _ = (user_id, muted);
}

pub fn user_volume(user_id: Id<UserMarker>) -> f32 {
    #[cfg(feature = "voice")]
    return local_mute::volume_percent(user_id.get());
    #[cfg(not(feature = "voice"))]
    {
        let _ = user_id;
        100.0
    }
}

pub fn set_user_volume(user_id: Id<UserMarker>, percent: f32) {
    #[cfg(feature = "voice")]
    local_mute::set_volume_percent(user_id.get(), percent);
    #[cfg(not(feature = "voice"))]
    let _ = (user_id, percent);
}

#[cfg(feature = "voice")]
pub fn video_frame(
    user_id: Id<UserMarker>,
) -> Option<(
    u64,
    sync::Arc<preview::PreviewFrame>,
)> {
    frames::latest(user_id.get())
}

pub fn is_streaming() -> bool {
    #[cfg(feature = "voice")]
    return voice::own_stream_key().is_some();
    #[cfg(not(feature = "voice"))]
    false
}

pub fn toggle_stream(guild_id: Id<GuildMarker>, channel_id: Id<ChannelMarker>) {
    #[cfg(feature = "voice")]
    match voice::own_stream_key() {
        Some(key) => voice::stop_stream(&key),
        None => voice::start_stream(guild_id.get(), channel_id.get(), ""),
    }
    #[cfg(not(feature = "voice"))]
    let _ = (guild_id, channel_id);
}

pub fn noise_suppression() -> bool {
    voice_settings::get().noise_suppression
}

pub fn set_noise_suppression(on: bool) {
    voice_settings::update(|s| s.noise_suppression = on);
}

pub fn devices(input: bool) -> Vec<String> {
    #[cfg(feature = "voice")]
    return audio::device_names(input);
    #[cfg(not(feature = "voice"))]
    {
        let _ = input;
        Vec::new()
    }
}
