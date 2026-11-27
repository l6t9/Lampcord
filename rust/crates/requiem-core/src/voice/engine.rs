use sonora::config::AdaptiveDigital;
use sonora::config::EchoCanceller;
use sonora::config::GainController2;
use sonora::config::HighPassFilter;
use sonora::config::NoiseSuppression;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::thread::Builder;
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::error::TryRecvError;

use super::audio::{AudioInput, AudioOutput};
use super::opus::FRAME_SAMPLES;
use crate::utils::voice_settings::{self, InputMode, VoiceSettings};

const HALF: usize = FRAME_SAMPLES / 2;

const HANGOVER_FRAMES: u32 = 13;

pub const SILENCE_DB: f32 = -100.0;

pub mod meter {
    use super::*;

    static LEVEL: AtomicU32 = AtomicU32::new(0);
    static SPEAKING: AtomicBool = AtomicBool::new(false);
    static AT: AtomicU64 = AtomicU64::new(0);

    pub(super) fn publish(level_db: f32, speaking: bool) {
        LEVEL.store(level_db.to_bits(), Ordering::Relaxed);
        SPEAKING.store(speaking, Ordering::Relaxed);
        AT.store(now_ms(), Ordering::Relaxed);
    }

    pub fn read() -> (f32, bool) {
        if now_ms().saturating_sub(AT.load(Ordering::Relaxed)) > 500 {
            return (SILENCE_DB, false);
        }
        (
            f32::from_bits(LEVEL.load(Ordering::Relaxed)),
            SPEAKING.load(Ordering::Relaxed),
        )
    }
}

fn epoch() -> Instant {
    static EPOCH: OnceLock<Instant> = OnceLock::new();
    *EPOCH.get_or_init(Instant::now)
}

fn now_ms() -> u64 {
    epoch().elapsed().as_millis() as u64
}

pub fn level_db(frame: &[f32]) -> f32 {
    let energy = frame.iter().map(|s| s * s).sum::<f32>() / frame.len().max(1) as f32;
    if energy <= 1e-10 {
        return SILENCE_DB;
    }
    (10.0 * energy.log10()).clamp(SILENCE_DB, 0.0)
}

fn apm_config(settings: &VoiceSettings) -> Option<sonora::Config> {
    if !settings.echo_cancellation
        && !settings.noise_suppression
        && !settings.automatic_gain_control
    {
        return None;
    }
    Some(sonora::Config {
        high_pass_filter: Some(HighPassFilter::default()),
        echo_canceller: settings.echo_cancellation.then(EchoCanceller::default),
        noise_suppression: settings.noise_suppression.then(NoiseSuppression::default),
        gain_controller2: settings.automatic_gain_control.then(|| GainController2 {
            adaptive_digital: Some(AdaptiveDigital::default()),
            ..GainController2::default()
        }),
        ..sonora::Config::default()
    })
}

pub struct CapturePipeline {
    settings: VoiceSettings,
    version: u64,
    apm: Option<sonora::AudioProcessing>,

    apm_switches: (bool, bool, bool),
    denoise: Box<nnnoiseless::DenoiseState<'static>>,
    denoise_out: Vec<f32>,

    floor_db: f32,
    hangover: u32,

    render_carry: Vec<f32>,
}

pub struct Processed {
    pub level_db: f32,

    pub speaking: bool,
}

impl Default for CapturePipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl CapturePipeline {
    pub fn new() -> Self {
        let mut pipeline = Self {
            settings: VoiceSettings::default(),
            version: 0,
            apm: None,
            apm_switches: (false, false, false),
            denoise: nnnoiseless::DenoiseState::new(),
            denoise_out: vec![0.0; nnnoiseless::DenoiseState::FRAME_SIZE],
            floor_db: -60.0,
            hangover: 0,
            render_carry: Vec::new(),
        };
        pipeline.refresh();
        pipeline
    }

    fn refresh(&mut self) {
        let version = voice_settings::version();
        if version == self.version {
            return;
        }
        self.version = version;
        let settings = voice_settings::effective();
        let config = apm_config(&settings);
        let switches = (
            settings.echo_cancellation,
            settings.noise_suppression,
            settings.automatic_gain_control,
        );
        match (&mut self.apm, config) {
            (_, None) => self.apm = None,
            (Some(apm), Some(config)) => {
                if self.apm_switches != switches {
                    apm.apply_config(config);
                }
            }
            (None, Some(config)) => {
                let apm = sonora::AudioProcessing::builder()
                    .config(config)
                    .capture_config(sonora::StreamConfig::new(48_000, 1))
                    .render_config(sonora::StreamConfig::new(48_000, 1))
                    .build();
                self.apm = Some(apm);
            }
        }
        self.apm_switches = switches;
        self.settings = settings;
    }

    pub fn settings(&self) -> &VoiceSettings {
        &self.settings
    }

    pub fn render(&mut self, stereo: &[f32]) {
        let Some(apm) = self
            .apm
            .as_mut()
            .filter(|_| self.settings.echo_cancellation)
        else {
            self.render_carry.clear();
            return;
        };
        self.render_carry.extend(
            stereo
                .chunks_exact(2)
                .map(|frame| (frame[0] + frame[1]) * 0.5),
        );
        while self.render_carry.len() >= HALF {
            let chunk: Vec<f32> = self.render_carry.drain(..HALF).collect();
            let mut out = vec![0.0f32; HALF];
            let _ = apm.process_render_f32(&[&chunk], &mut [&mut out]);
        }
    }

    pub fn process(&mut self, frame: &mut [f32]) -> Processed {
        self.refresh();
        let gain = self.settings.input_volume / 100.0;
        if (gain - 1.0).abs() > f32::EPSILON {
            for sample in frame.iter_mut() {
                *sample *= gain;
            }
        }
        if let Some(apm) = self.apm.as_mut() {
            for half in frame.chunks_mut(HALF) {
                let input = half.to_vec();
                let _ = apm.process_capture_f32(&[&input], &mut [half]);
            }
        }
        let level = level_db(frame);
        let speaking = match self.settings.mode {
            InputMode::PushToTalk => super::ptt::active(),
            InputMode::VoiceActivity => self.detect(frame, level),
        };
        meter::publish(level, speaking);
        Processed {
            level_db: level,
            speaking,
        }
    }

    fn detect(&mut self, frame: &[f32], level: f32) -> bool {
        let options = &self.settings.mode_options;
        let voiced = if options.auto_threshold {
            if level < self.floor_db {
                self.floor_db = self.floor_db * 0.8 + level * 0.2;
            } else {
                self.floor_db = (self.floor_db + 0.03).min(level);
            }
            if options.vad_use_krisp {
                let threshold = if self.settings.automatic_gain_control {
                    0.8
                } else {
                    0.5
                };
                let mut probability = 0.0f32;
                for half in frame.chunks(HALF) {
                    let scaled: Vec<f32> = half.iter().map(|s| s * 32767.0).collect();
                    probability =
                        probability.max(self.denoise.process_frame(&mut self.denoise_out, &scaled));
                }
                probability > threshold && level > SILENCE_DB + 30.0
            } else {
                level > self.floor_db + 9.0 && level > -70.0
            }
        } else {
            level > options.threshold
        };
        if voiced {
            self.hangover = HANGOVER_FRAMES;
            true
        } else if self.hangover > 0 {
            self.hangover -= 1;
            true
        } else {
            false
        }
    }
}

static KEEPALIVE: AtomicU64 = AtomicU64::new(0);
static MONITOR_RUNNING: AtomicBool = AtomicBool::new(false);
static LOOPBACK: AtomicBool = AtomicBool::new(false);

static CONNECTION_CAPTURING: AtomicBool = AtomicBool::new(false);

pub fn keepalive() {
    KEEPALIVE.store(now_ms(), Ordering::Relaxed);
    if CONNECTION_CAPTURING.load(Ordering::Relaxed) {
        return;
    }
    if MONITOR_RUNNING.swap(true, Ordering::AcqRel) {
        return;
    }
    let spawned = Builder::new()
        .name("voice-monitor".into())
        .spawn(|| {
            run_monitor();
            MONITOR_RUNNING.store(false, Ordering::Release);
            LOOPBACK.store(false, Ordering::Release);
        });
    if spawned.is_err() {
        MONITOR_RUNNING.store(false, Ordering::Release);
    }
}

pub fn set_loopback(on: bool) {
    LOOPBACK.store(on, Ordering::Release);
    if on {
        keepalive();
    }
}

pub fn loopback() -> bool {
    LOOPBACK.load(Ordering::Acquire)
}

pub fn measuring() -> bool {
    MONITOR_RUNNING.load(Ordering::Acquire) || CONNECTION_CAPTURING.load(Ordering::Acquire)
}

pub(super) fn set_connection_capturing(on: bool) {
    CONNECTION_CAPTURING.store(on, Ordering::Release);
}

fn device_key(
    settings: &VoiceSettings,
) -> (
    Option<String>,
    Option<String>,
    voice_settings::AudioSubsystem,
) {
    (
        settings.input_device.clone(),
        settings.output_device.clone(),
        settings.audio_subsystem,
    )
}

fn wants_standing_output(
    key: &(
        Option<String>,
        Option<String>,
        voice_settings::AudioSubsystem,
    ),
) -> bool {
    key.2 == voice_settings::AudioSubsystem::Asio
}

fn open_output(
    key: &(
        Option<String>,
        Option<String>,
        voice_settings::AudioSubsystem,
    ),
) -> Option<AudioOutput> {
    AudioOutput::start(key.1.as_deref())
        .map_err(|err| log::warn!("voice monitor: no output: {err}"))
        .ok()
}

fn run_monitor() {
    let key = device_key(&voice_settings::get());
    let mut input = match AudioInput::start(key.0.as_deref()) {
        Ok(input) => input,
        Err(err) => {
            log::warn!("voice monitor: no input: {err}");

            thread::sleep(Duration::from_secs(2));
            return;
        }
    };
    let mut output: Option<AudioOutput> = if wants_standing_output(&key) {
        open_output(&key)
    } else {
        None
    };
    let mut pipeline = CapturePipeline::new();
    loop {
        if now_ms().saturating_sub(KEEPALIVE.load(Ordering::Relaxed)) > 1500
            || CONNECTION_CAPTURING.load(Ordering::Relaxed)
        {
            break;
        }
        let settings = voice_settings::get();
        let wanted = device_key(&settings);
        if wanted != key {
            break;
        }
        let mut frame = match input.frames.try_recv() {
            Ok(frame) => frame,
            Err(TryRecvError::Empty) => {
                thread::sleep(Duration::from_millis(5));
                continue;
            }
            Err(TryRecvError::Disconnected) => break,
        };
        let processed = pipeline.process(&mut frame);
        if LOOPBACK.load(Ordering::Acquire) {
            if output.is_none() {
                output = open_output(&key);
            }
            if let Some(output) = &output {
                output.set_gain(settings.output_volume / 100.0);

                let audible = processed.speaking || matches!(settings.mode, InputMode::PushToTalk);
                let stereo: Vec<f32> = if audible {
                    frame.iter().flat_map(|s| [*s, *s]).collect()
                } else {
                    vec![0.0; frame.len() * 2]
                };
                output.push(&stereo);
            }
        } else if !wants_standing_output(&key) {
            output = None;
        }
    }

    drop(input);
    drop(output);
    meter::publish(SILENCE_DB, false);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_are_dbfs() {
        assert_eq!(level_db(&[0.0; 960]), SILENCE_DB);
        let full: Vec<f32> = (0..960)
            .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        assert!(level_db(&full).abs() < 0.01);
        let quiet = vec![0.01f32; 960];
        assert!((level_db(&quiet) + 40.0).abs() < 0.01);
    }

    #[test]
    fn manual_threshold_and_hangover() {
        let mut pipeline = CapturePipeline::new();
        pipeline.apm = None;
        pipeline.settings = VoiceSettings {
            mode_options: voice_settings::ModeOptions {
                auto_threshold: false,
                threshold: -50.0,
                ..Default::default()
            },
            ..VoiceSettings::default()
        };
        let loud = vec![0.1f32; 960];
        let quiet = vec![0.0001f32; 960];
        assert!(pipeline.detect(&loud, level_db(&loud)));

        for _ in 0..HANGOVER_FRAMES {
            assert!(pipeline.detect(&quiet, level_db(&quiet)));
        }
        assert!(!pipeline.detect(&quiet, level_db(&quiet)));
    }
}
