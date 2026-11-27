use crate::utils::notification_settings;
use crate::utils::voice_settings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sound {
    Message,

    Message3,
    Mention,
    Mute,
    Unmute,
    Deafen,
    Undeafen,
    Disconnect,
    Reconnect,
    PttStart,
    PttStop,
    UserJoin,
    UserLeave,
    UserMoved,
    CallCalling,
    CallRinging,
    StreamStarted,
    StreamEnded,
    StreamUserJoined,
    StreamUserLeft,
    CameraOn,
    CameraOff,
    ActivityLaunch,
    ActivityEnd,
    ActivityUserJoin,
    ActivityUserLeft,
}

impl Sound {
    pub const ALL: [Sound; 26] = [
        Sound::Message,
        Sound::Message3,
        Sound::Mention,
        Sound::Mute,
        Sound::Unmute,
        Sound::Deafen,
        Sound::Undeafen,
        Sound::Disconnect,
        Sound::Reconnect,
        Sound::PttStart,
        Sound::PttStop,
        Sound::UserJoin,
        Sound::UserLeave,
        Sound::UserMoved,
        Sound::CallCalling,
        Sound::CallRinging,
        Sound::StreamStarted,
        Sound::StreamEnded,
        Sound::StreamUserJoined,
        Sound::StreamUserLeft,
        Sound::CameraOn,
        Sound::CameraOff,
        Sound::ActivityLaunch,
        Sound::ActivityEnd,
        Sound::ActivityUserJoin,
        Sound::ActivityUserLeft,
    ];

    pub const fn asset(self) -> &'static str {
        match self {
            Sound::Message => "sound/message1",
            Sound::Message3 => "sound/message3",
            Sound::Mention => "sound/mention1",
            Sound::Mute => "sound/mute",
            Sound::Unmute => "sound/unmute",
            Sound::Deafen => "sound/deafen",
            Sound::Undeafen => "sound/undeafen",
            Sound::Disconnect => "sound/disconnect",
            Sound::Reconnect => "sound/reconnect",
            Sound::PttStart => "sound/ptt_start",
            Sound::PttStop => "sound/ptt_stop",
            Sound::UserJoin => "sound/user_join",
            Sound::UserLeave => "sound/user_leave",
            Sound::UserMoved => "sound/user_moved",
            Sound::CallCalling => "sound/call_calling",
            Sound::CallRinging => "sound/call_ringing",
            Sound::StreamStarted => "sound/stream_started",
            Sound::StreamEnded => "sound/stream_ended",
            Sound::StreamUserJoined => "sound/stream_user_joined",
            Sound::StreamUserLeft => "sound/stream_user_left",
            Sound::CameraOn => "sound/camera_on",
            Sound::CameraOff => "sound/camera_off",
            Sound::ActivityLaunch => "sound/activity_launch",
            Sound::ActivityEnd => "sound/activity_end",
            Sound::ActivityUserJoin => "sound/activity_user_join",
            Sound::ActivityUserLeft => "sound/activity_user_left",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Sound::Message => "message1",
            Sound::Message3 => "message3",
            Sound::Mention => "mention1",
            other => other.asset().trim_start_matches("sound/"),
        }
    }

    pub fn from_id(id: &str) -> Option<Sound> {
        Sound::ALL.into_iter().find(|s| s.id() == id)
    }
}

pub fn play(sound: Sound) {
    if notification_settings::get().disable_all_sounds
        || !voice_settings::get().is_sound_enabled(sound.id())
    {
        return;
    }
    preview(sound);
}

pub fn preview(sound: Sound) {
    #[cfg(feature = "sound")]
    player::play(sound);
    #[cfg(not(feature = "sound"))]
    let _ = sound;
}

#[cfg_attr(not(feature = "sound"), allow(dead_code))]
struct Pcm {
    rate: u32,
    channels: usize,
    samples: Vec<f32>,
}

#[cfg_attr(not(feature = "sound"), allow(dead_code))]
fn convert(pcm: &Pcm, rate: u32, channels: usize) -> Vec<f32> {
    if pcm.channels == 0 || channels == 0 || pcm.samples.is_empty() {
        return Vec::new();
    }
    let frames_in = pcm.samples.len() / pcm.channels;
    let frames_out = (frames_in as u64 * rate as u64 / pcm.rate.max(1) as u64) as usize;

    let sample = |frame: usize, out_channel: usize| -> f32 {
        let frame = frame.min(frames_in - 1);
        let at = |c: usize| pcm.samples[frame * pcm.channels + c];
        match (pcm.channels, channels) {
            (1, _) => at(0),
            (_, 1) => (0..pcm.channels).map(at).sum::<f32>() / pcm.channels as f32,
            (_, _) if out_channel < pcm.channels.min(2) => at(out_channel),
            _ => 0.0,
        }
    };
    let mut out = Vec::with_capacity(frames_out * channels);
    for frame in 0..frames_out {
        let position = frame as f64 * pcm.rate as f64 / rate as f64;
        let (i, t) = (position as usize, (position - position.floor()) as f32);
        for c in 0..channels {
            let (a, b) = (sample(i, c), sample(i + 1, c));
            out.push(a + (b - a) * t);
        }
    }
    out
}

#[cfg(feature = "sound")]
mod player {
    use crate::utils::voice_settings;
    use std::io::Cursor;
    use std::io::ErrorKind;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::thread::Builder;
    use std::thread;
    use std::time::Duration;
    use std::time::Instant;
    use symphonia::default;

    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::errors::Error as SymphoniaError;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    use super::{Pcm, Sound, convert};

    const GAIN: f32 = 0.6;

    pub fn play(sound: Sound) {
        let Some(bytes) =
            requiem_asset_store::get().and_then(|store| store.file_bytes(sound.asset()))
        else {
            log::debug!("sound {}: asset not available", sound.asset());
            return;
        };
        let spawned = Builder::new()
            .name("ui-sound".into())
            .spawn(move || {
                if let Err(why) = play_bytes(&bytes) {
                    log::warn!("sound {}: {why}", sound.asset());
                }
            });
        if let Err(err) = spawned {
            log::warn!("sound {}: could not start a thread: {err}", sound.asset());
        }
    }

    pub(super) fn decode(bytes: &[u8]) -> Result<Pcm, String> {
        let source =
            MediaSourceStream::new(Box::new(Cursor::new(bytes.to_vec())), Default::default());
        let mut hint = Hint::new();
        hint.with_extension("mp3");
        let probed = default::get_probe()
            .format(
                &hint,
                source,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|e| format!("not an mp3: {e}"))?;
        let mut format = probed.format;
        let track = format.default_track().ok_or("no audio track")?;
        let track_id = track.id;
        let mut decoder = default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| format!("no decoder: {e}"))?;
        let mut pcm = Pcm {
            rate: 0,
            channels: 0,
            samples: Vec::new(),
        };
        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(e))
                    if e.kind() == ErrorKind::UnexpectedEof =>
                {
                    break;
                }
                Err(SymphoniaError::ResetRequired) => break,
                Err(e) => return Err(format!("read failed: {e}")),
            };
            if packet.track_id() != track_id {
                continue;
            }
            match decoder.decode(&packet) {
                Ok(buffer) => {
                    let spec = *buffer.spec();
                    pcm.rate = spec.rate;
                    pcm.channels = spec.channels.count();
                    let mut samples = SampleBuffer::<f32>::new(buffer.capacity() as u64, spec);
                    samples.copy_interleaved_ref(buffer);
                    pcm.samples.extend_from_slice(samples.samples());
                }

                Err(SymphoniaError::DecodeError(_)) => continue,
                Err(e) => return Err(format!("decode failed: {e}")),
            }
        }
        if pcm.samples.is_empty() {
            return Err("decoded no audio".into());
        }
        Ok(pcm)
    }

    pub(super) fn play_bytes(bytes: &[u8]) -> Result<(), String> {
        let pcm = decode(bytes)?;
        let host = cpal::default_host();
        let wanted = voice_settings::get().output_device;
        let named = wanted.and_then(|name| {
            host.output_devices()
                .ok()?
                .find(|d| d.name().ok().as_deref() == Some(name.as_str()))
        });
        let device = named
            .or_else(|| host.default_output_device())
            .ok_or("no audio output device")?;
        let supported = device
            .default_output_config()
            .map_err(|e| format!("no output config: {e}"))?;
        let format = supported.sample_format();
        let config: cpal::StreamConfig = supported.into();
        let samples: Arc<[f32]> = convert(&pcm, config.sample_rate.0, config.channels as usize)
            .into_iter()
            .map(|s| s * GAIN)
            .collect();
        if samples.is_empty() {
            return Err("nothing to play".into());
        }
        let (position, done) = (
            Arc::new(AtomicUsize::new(0)),
            Arc::new(AtomicBool::new(false)),
        );
        let err = |e| log::warn!("sound output error: {e}");

        macro_rules! output {
            ($sample:ty, $convert:expr) => {{
                let (samples, position, done) = (samples.clone(), position.clone(), done.clone());
                device.build_output_stream(
                    &config,
                    move |data: &mut [$sample], _| {
                        let start = position.load(Ordering::Relaxed);
                        for (i, out) in data.iter_mut().enumerate() {
                            let value = samples.get(start + i).copied().unwrap_or(0.0);
                            *out = $convert(value);
                        }
                        let next = start + data.len();
                        position.store(next, Ordering::Relaxed);
                        if next >= samples.len() {
                            done.store(true, Ordering::Release);
                        }
                    },
                    err,
                    None,
                )
            }};
        }
        let stream = match format {
            cpal::SampleFormat::F32 => output!(f32, |v: f32| v),
            cpal::SampleFormat::I16 => {
                output!(i16, |v: f32| (v.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
            }
            other => return Err(format!("unsupported output sample format {other:?}")),
        }
        .map_err(|e| format!("could not open the output: {e}"))?;
        stream
            .play()
            .map_err(|e| format!("could not start the output: {e}"))?;

        let length = Duration::from_secs_f64(
            samples.len() as f64 / (config.sample_rate.0 as f64 * config.channels as f64),
        );
        let deadline = Instant::now() + length + Duration::from_secs(2);
        while !done.load(Ordering::Acquire) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }

        thread::sleep(Duration::from_millis(250));
        drop(stream);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use super::*;

    #[test]
    fn mono_becomes_stereo_and_rates_scale() {
        let pcm = Pcm {
            rate: 1000,
            channels: 1,
            samples: vec![0.0, 1.0, 0.0, -1.0],
        };
        let out = convert(&pcm, 2000, 2);
        assert_eq!(out.len(), 8 * 2);

        assert!(out.chunks(2).all(|f| f[0] == f[1]));
        assert_eq!(out[2 * 2], 1.0);
    }

    #[test]
    fn stereo_to_mono_averages_and_extra_channels_are_silent() {
        let pcm = Pcm {
            rate: 100,
            channels: 2,
            samples: vec![1.0, 0.0, 1.0, 0.0],
        };
        assert_eq!(convert(&pcm, 100, 1), vec![0.5, 0.5]);
        let six = convert(&pcm, 100, 6);
        assert_eq!(&six[..6], &[1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    }

    #[cfg(feature = "sound")]
    #[test]
    fn a_real_sound_decodes() {
        let Some(path) = env::var_os("REQUIEM_TEST_MP3") else {
            return;
        };
        let pcm = player::decode(&fs::read(path).expect("readable")).expect("decodes");
        assert!(pcm.rate >= 8000 && pcm.channels >= 1);
        let seconds = pcm.samples.len() as f32 / (pcm.rate as f32 * pcm.channels as f32);
        eprintln!("{} Hz, {} ch, {seconds:.2}s", pcm.rate, pcm.channels);
        assert!(seconds > 0.05 && seconds < 10.0, "{seconds}s");

        match player::play_bytes(
            &fs::read(env::var_os("REQUIEM_TEST_MP3").expect("set")).expect("readable"),
        ) {
            Ok(()) => eprintln!("played"),
            Err(why) => eprintln!("could not play: {why}"),
        }
    }
}
