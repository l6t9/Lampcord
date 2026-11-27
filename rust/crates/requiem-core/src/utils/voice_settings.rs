use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

pub const MAX_PTT_RELEASE_DELAY: u32 = 2000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InputMode {
    #[default]
    VoiceActivity,
    PushToTalk,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InputProfile {
    VoiceIsolation,
    Studio,
    Custom,
}

impl InputProfile {
    pub const ALL: [InputProfile; 3] = [
        InputProfile::VoiceIsolation,
        InputProfile::Studio,
        InputProfile::Custom,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            InputProfile::VoiceIsolation => "VOICE_ISOLATION",
            InputProfile::Studio => "STUDIO",
            InputProfile::Custom => "CUSTOM",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.as_str() == value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AudioSubsystem {
    #[default]
    Standard,
    Asio,
}

impl AudioSubsystem {
    pub fn as_str(self) -> &'static str {
        match self {
            AudioSubsystem::Standard => "STANDARD",
            AudioSubsystem::Asio => "ASIO",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        [AudioSubsystem::Standard, AudioSubsystem::Asio]
            .into_iter()
            .find(|s| s.as_str() == value)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ModeOptions {
    pub threshold: f32,
    pub auto_threshold: bool,

    pub vad_use_krisp: bool,

    pub delay: u32,

    pub shortcut: Vec<u32>,
}

impl Default for ModeOptions {
    fn default() -> Self {
        Self {
            threshold: -60.0,
            auto_threshold: true,
            vad_use_krisp: true,
            delay: 20,
            shortcut: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VoiceSettings {
    pub mode: InputMode,
    pub mode_options: ModeOptions,
    pub echo_cancellation: bool,

    pub noise_suppression: bool,

    pub noise_cancellation: bool,
    pub automatic_gain_control: bool,
    pub silence_warning: bool,

    pub attenuation: u8,
    pub attenuate_while_speaking_self: bool,
    pub attenuate_while_speaking_others: bool,

    pub input_volume: f32,

    pub output_volume: f32,

    pub input_device: Option<String>,
    pub output_device: Option<String>,
    pub video_device: Option<String>,
    pub qos: bool,
    pub audio_subsystem: AudioSubsystem,

    pub active_input_profile: Option<InputProfile>,

    pub sidechain_compression: bool,
    pub sidechain_compression_strength: u8,

    pub debug_logging: bool,
    pub aec_dump: bool,

    pub always_preview_video: bool,

    pub stream_previews_disabled: bool,

    pub disable_voice_channel_change_alert: bool,

    pub disabled_sounds: Vec<String>,

    pub soundboard_volume: f32,

    pub join_sounds: Vec<(String, String)>,

    pub device_suggestions: bool,
    pub stream_info_overlay: bool,

    pub local_mutes: Vec<u64>,

    pub local_volumes: Vec<(u64, f32)>,
}

impl Default for VoiceSettings {
    fn default() -> Self {
        Self {
            mode: InputMode::VoiceActivity,
            mode_options: ModeOptions::default(),
            echo_cancellation: true,
            noise_suppression: false,
            noise_cancellation: true,
            automatic_gain_control: true,
            silence_warning: true,
            attenuation: 0,
            attenuate_while_speaking_self: false,
            attenuate_while_speaking_others: true,
            input_volume: 100.0,
            output_volume: 100.0,
            input_device: None,
            output_device: None,
            video_device: None,
            qos: false,
            audio_subsystem: AudioSubsystem::Standard,
            active_input_profile: None,
            sidechain_compression: true,
            sidechain_compression_strength: 50,
            debug_logging: false,
            aec_dump: false,
            always_preview_video: false,
            stream_previews_disabled: false,
            disable_voice_channel_change_alert: false,
            disabled_sounds: Vec::new(),
            soundboard_volume: 100.0,
            join_sounds: Vec::new(),
            device_suggestions: true,
            stream_info_overlay: false,
            local_mutes: Vec::new(),
            local_volumes: Vec::new(),
        }
    }
}

impl VoiceSettings {
    pub fn is_input_profile_custom(&self) -> bool {
        matches!(self.active_input_profile, None | Some(InputProfile::Custom))
    }

    pub fn is_sound_enabled(&self, sound: &str) -> bool {
        !self.disabled_sounds.iter().any(|s| s == sound)
    }

    pub fn effective(&self) -> VoiceSettings {
        let mut out = self.clone();
        match self.active_input_profile {
            None | Some(InputProfile::Custom) => {}
            Some(InputProfile::VoiceIsolation) => {
                out.mode_options.auto_threshold = true;
                out.mode_options.vad_use_krisp = true;
                out.echo_cancellation = true;
                out.noise_suppression = false;
                out.automatic_gain_control = true;
                out.noise_cancellation = true;
            }
            Some(InputProfile::Studio) => {
                out.mode = InputMode::VoiceActivity;
                out.mode_options.threshold = -84.0;
                out.mode_options.auto_threshold = false;
                out.mode_options.vad_use_krisp = false;
                out.echo_cancellation = false;
                out.noise_suppression = false;
                out.automatic_gain_control = false;
                out.noise_cancellation = false;
            }
        }
        if out.noise_cancellation {
            out.noise_cancellation = false;
            out.noise_suppression = true;
        }
        out
    }
}

pub fn slider_to_volume(value: f32, max: f32) -> f32 {
    if value <= 0.0 {
        return 0.0;
    }
    let n = value / 100.0;
    let v = if n < 1.0 {
        n.powf(2.8)
    } else {
        10f32.powf((n - 1.0) * 6.0 / 20.0)
    };
    (v * 100.0).min(max)
}

pub fn volume_to_slider(volume: f32) -> f32 {
    if volume <= 0.0 {
        return 0.0;
    }
    let n = volume / 100.0;
    let v = if n < 1.0 {
        n.powf(1.0 / 2.8)
    } else {
        20.0 * n.log10() / 6.0 + 1.0
    };
    v * 100.0
}

static SETTINGS: RwLock<Option<VoiceSettings>> = RwLock::new(None);
static VERSION: AtomicU64 = AtomicU64::new(1);

fn load() -> VoiceSettings {
    let prefs = super::prefs::load();
    if let Some(settings) = prefs.voice_settings {
        return settings;
    }

    VoiceSettings {
        input_device: prefs.voice_input_device,
        output_device: prefs.voice_output_device,
        ..VoiceSettings::default()
    }
}

pub fn get() -> VoiceSettings {
    if let Some(settings) = SETTINGS.read().expect("voice settings poisoned").as_ref() {
        return settings.clone();
    }
    let mut slot = SETTINGS.write().expect("voice settings poisoned");
    slot.get_or_insert_with(load).clone()
}

pub fn effective() -> VoiceSettings {
    get().effective()
}

pub fn version() -> u64 {
    VERSION.load(Ordering::Acquire)
}

pub fn update(change: impl FnOnce(&mut VoiceSettings)) {
    let saved = {
        let mut slot = SETTINGS.write().expect("voice settings poisoned");
        let settings = slot.get_or_insert_with(load);
        change(settings);
        settings.clone()
    };
    VERSION.fetch_add(1, Ordering::AcqRel);
    super::prefs::save(&super::prefs::Prefs {
        voice_settings: Some(saved),
        voice_input_device: None,
        voice_output_device: None,
        ..super::prefs::load()
    });
}

pub fn reset() {
    update(|settings| *settings = VoiceSettings::default());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_volume_curve_round_trips_like_the_live_one() {
        assert!((slider_to_volume(50.0, 100.0) - 14.358).abs() < 0.01);
        assert!((slider_to_volume(200.0, 200.0) - 199.526).abs() < 0.01);
        for v in [0.0, 1.0, 37.0, 100.0, 150.0, 200.0] {
            assert!(
                (volume_to_slider(slider_to_volume(v, 200.0)) - v).abs() < 0.01,
                "{v}"
            );
        }
    }

    #[test]
    fn profiles_override_without_touching_the_custom_values() {
        let custom = VoiceSettings {
            echo_cancellation: false,
            automatic_gain_control: false,
            ..VoiceSettings::default()
        };
        let isolation = VoiceSettings {
            active_input_profile: Some(InputProfile::VoiceIsolation),
            ..custom.clone()
        };
        assert!(isolation.effective().echo_cancellation);
        assert!(
            isolation.effective().noise_suppression,
            "Krisp falls back to standard suppression"
        );
        let studio = VoiceSettings {
            active_input_profile: Some(InputProfile::Studio),
            ..custom.clone()
        };
        let studio = studio.effective();
        assert_eq!(studio.mode_options.threshold, -84.0);
        assert!(!studio.mode_options.auto_threshold && !studio.noise_suppression);
        assert!(!custom.effective().echo_cancellation);
    }
}
