use parking_lot::Mutex;
use serde_json::json;

use crate::gateway::events::GatewayEvent;

pub use crate::state::RtcState;

type Sink = Box<dyn Fn(GatewayEvent) + Send + Sync>;

static SINK: Mutex<Option<Sink>> = Mutex::new(None);

pub fn set_sink(sink: impl Fn(GatewayEvent) + Send + Sync + 'static) {
    *SINK.lock() = Some(Box::new(sink));
}

fn emit(name: &str, d: serde_json::Value) {
    if let Some(sink) = SINK.lock().as_ref() {
        sink(GatewayEvent::new(name, d));
    }
}

pub fn rtc_state(state: RtcState, guild_id: Option<u64>, channel_id: Option<u64>) {
    log::info!("voice: {}", state.as_str());
    emit(
        "RTC_CONNECTION_STATE",
        json!({ "state": state.as_str(), "guild_id": guild_id.map(|g| g.to_string()), "channel_id": channel_id.map(|c| c.to_string()) }),
    );
}

pub fn ping(ms: u64) {
    emit("RTC_CONNECTION_PING", json!({ "ping": ms }));
}

pub fn speaking(user_id: u64, flags: u8) {
    emit(
        "SPEAKING",
        json!({ "user_id": user_id.to_string(), "speaking": flags }),
    );
}

pub fn video(user_id: u64, active: bool) {
    emit(
        "VOICE_VIDEO_STATE",
        json!({ "user_id": user_id.to_string(), "active": active }),
    );
}
