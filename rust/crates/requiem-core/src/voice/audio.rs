use crate::utils::voice_settings;
use parking_lot::Mutex;
use std::collections;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::mpsc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use tokio::sync::mpsc::UnboundedReceiver;

use super::opus::{FRAME_SAMPLES, SAMPLE_RATE};
use crate::utils::voice_settings::AudioSubsystem;

const OUTPUT_BUFFER_SECS: f32 = 0.25;

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("no default {0} device")]
    NoDevice(&'static str),
    #[error("unsupported {0} stream config: {1}")]
    Unsupported(&'static str, String),
    #[error("cpal: {0}")]
    Cpal(String),
}

fn host(subsystem: AudioSubsystem) -> cpal::Host {
    #[cfg(all(windows, feature = "asio"))]
    if subsystem == AudioSubsystem::Asio {
        match cpal::host_from_id(cpal::HostId::Asio) {
            Ok(host) => return host,
            Err(err) => log::warn!("voice: ASIO unavailable ({err}), using WASAPI"),
        }
    }
    let _ = subsystem;
    cpal::default_host()
}

fn current_host() -> cpal::Host {
    host(voice_settings::get().audio_subsystem)
}

pub fn asio_available() -> bool {
    #[cfg(all(windows, feature = "asio"))]
    {
        !asio::names().is_empty()
    }
    #[cfg(not(all(windows, feature = "asio")))]
    false
}

#[cfg(all(windows, feature = "asio"))]
mod asio {
    use cpal::traits::{DeviceTrait, HostTrait};
    use parking_lot::Mutex;

    struct Drivers {
        names: Vec<String>,
        loaded: Vec<cpal::Device>,
    }

    static DRIVERS: Mutex<Option<Drivers>> = Mutex::new(None);

    fn with<R>(f: impl FnOnce(&mut Drivers) -> R) -> R {
        let mut slot = DRIVERS.lock();
        let drivers = slot.get_or_insert_with(|| {
            let mut drivers = Drivers {
                names: Vec::new(),
                loaded: Vec::new(),
            };
            if let Some(devices) = cpal::host_from_id(cpal::HostId::Asio)
                .ok()
                .and_then(|host| host.devices().ok())
            {
                for device in devices {
                    if let Ok(name) = device.name() {
                        drivers.names.push(name);
                        if drivers.loaded.is_empty() {
                            drivers.loaded.push(device);
                        }
                    }
                }
            }
            log::info!("voice: ASIO drivers: {:?}", drivers.names);
            drivers
        });
        f(drivers)
    }

    pub fn names() -> Vec<String> {
        with(|drivers| drivers.names.clone())
    }

    pub fn device(name: Option<&str>) -> Option<cpal::Device> {
        with(|drivers| {
            let wanted = name
                .map(str::to_string)
                .or_else(|| drivers.names.first().cloned())?;
            if let Some(device) = drivers
                .loaded
                .iter()
                .find(|d| d.name().ok().as_deref() == Some(wanted.as_str()))
            {
                return Some(device.clone());
            }
            let device = cpal::host_from_id(cpal::HostId::Asio)
                .ok()?
                .devices()
                .ok()?
                .find(|d| d.name().ok().as_deref() == Some(wanted.as_str()));
            match device {
                Some(device) => {
                    drivers.loaded.push(device.clone());
                    Some(device)
                }
                None => {
                    log::warn!(
                        "voice: the ASIO driver {wanted:?} could not be loaded while another one is (ASIO allows one at a time)"
                    );
                    None
                }
            }
        })
    }
}

fn resolve_device(
    host: &cpal::Host,
    name: Option<&str>,
    input: bool,
) -> Result<cpal::Device, AudioError> {
    let which = if input { "input" } else { "output" };
    #[cfg(all(windows, feature = "asio"))]
    if host.id() == cpal::HostId::Asio {
        let name = name.filter(|name| asio::names().iter().any(|n| n == name));
        return asio::device(name).ok_or(AudioError::NoDevice(which));
    }
    if let Some(wanted) = name.filter(|name| !name.is_empty()) {
        let listed = if input {
            host.input_devices()
        } else {
            host.output_devices()
        };
        if let Ok(listed) = listed {
            for device in listed {
                if device.name().ok().as_deref() == Some(wanted) {
                    return Ok(device);
                }
            }
            log::warn!("voice: {which} device {wanted:?} is gone, using the default");
        }
    }
    let fallback = if input {
        host.default_input_device()
    } else {
        host.default_output_device()
    };

    let fallback = fallback.or_else(|| {
        let listed = if input {
            host.input_devices()
        } else {
            host.output_devices()
        };
        listed.ok().and_then(|mut devices| devices.next())
    });
    fallback.ok_or(AudioError::NoDevice(which))
}

pub fn device_names(input: bool) -> Vec<String> {
    let host = current_host();
    #[cfg(all(windows, feature = "asio"))]
    if host.id() == cpal::HostId::Asio {
        let _ = input;
        return asio::names();
    }
    let listed = if input {
        host.input_devices()
    } else {
        host.output_devices()
    };
    listed
        .ok()
        .map(|listed| listed.filter_map(|device| device.name().ok()).collect())
        .unwrap_or_default()
}

pub fn default_device_name(input: bool) -> Option<String> {
    let host = current_host();
    #[cfg(all(windows, feature = "asio"))]
    if host.id() == cpal::HostId::Asio {
        return asio::names().into_iter().next();
    }
    let device = if input {
        host.default_input_device()
    } else {
        host.default_output_device()
    };
    device.and_then(|device| device.name().ok())
}

fn cpal_err<E: fmt::Display>(err: E) -> AudioError {
    AudioError::Cpal(err.to_string())
}

pub struct AudioInput {
    stream: cpal::Stream,
    pub frames: UnboundedReceiver<Vec<f32>>,
}

struct InputBuffer {
    mono: Vec<f32>,
    device_rate: u32,
    device_channels: u16,
}

impl AudioInput {
    pub fn start(device: Option<&str>) -> Result<Self, AudioError> {
        let host = current_host();
        let device = resolve_device(&host, device, true)?;
        let config = device.default_input_config().map_err(cpal_err)?;
        let (rate, channels, format) = (
            config.sample_rate().0,
            config.channels(),
            config.sample_format(),
        );
        log::info!(
            "voice: input {:?} at {rate} Hz, {channels} ch, {format:?} ({:?})",
            device.name().unwrap_or_default(),
            host.id()
        );

        let (tx, frames) = mpsc::unbounded_channel();
        let buffer = Arc::new(Mutex::new(InputBuffer {
            mono: Vec::with_capacity(4096),
            device_rate: rate,
            device_channels: channels,
        }));
        let on_samples = move |data: &[f32]| {
            let mut buf = buffer.lock();
            let (rate, channels) = (buf.device_rate, buf.device_channels);
            push_downmixed_resampled(&mut buf.mono, data, rate, channels, SAMPLE_RATE);

            while buf.mono.len() >= FRAME_SAMPLES {
                let frame: Vec<f32> = buf.mono.drain(..FRAME_SAMPLES).collect();
                let _ = tx.send(frame);
            }
        };
        let err = |err| log::warn!("voice: audio input error: {err}");
        let stream_config: cpal::StreamConfig = config.into();

        macro_rules! input {
            ($t:ty, $to_f32:expr) => {{
                let mut scratch: Vec<f32> = Vec::new();
                device.build_input_stream(
                    &stream_config,
                    move |data: &[$t], _| {
                        scratch.clear();
                        scratch.extend(data.iter().map(|&s| $to_f32(s)));
                        on_samples(&scratch);
                    },
                    err,
                    None,
                )
            }};
        }
        let stream = match format {
            cpal::SampleFormat::F32 => input!(f32, |s: f32| s),
            cpal::SampleFormat::I16 => input!(i16, |s: i16| s as f32 / 32768.0),
            cpal::SampleFormat::I32 => input!(i32, |s: i32| s as f32 / 2_147_483_648.0),
            cpal::SampleFormat::U16 => input!(u16, |s: u16| (s as f32 - 32768.0) / 32768.0),
            other => {
                return Err(AudioError::Unsupported(
                    "input",
                    format!("sample format {other:?}"),
                ));
            }
        }
        .map_err(cpal_err)?;
        stream.play().map_err(cpal_err)?;

        Ok(Self { stream, frames })
    }

    pub fn set_paused(&self, paused: bool) {
        if paused {
            let _ = self.stream.pause();
        } else {
            let _ = self.stream.play();
        }
    }
}

pub struct AudioOutput {
    _stream: cpal::Stream,
    buffer: Arc<Mutex<OutputBuffer>>,

    gain: Arc<AtomicU32>,
}

struct OutputBuffer {
    stereo: Vec<f32>,

    cursors: collections::HashMap<u32, usize>,
    device_rate: u32,
    device_channels: u16,
}

impl AudioOutput {
    pub fn start(device: Option<&str>) -> Result<Self, AudioError> {
        let host = current_host();
        let device = resolve_device(&host, device, false)?;
        let config = device.default_output_config().map_err(cpal_err)?;
        let (rate, channels, format) = (
            config.sample_rate().0,
            config.channels(),
            config.sample_format(),
        );
        log::info!(
            "voice: output {:?} at {rate} Hz, {channels} ch, {format:?} ({:?})",
            device.name().unwrap_or_default(),
            host.id()
        );

        let max_frames = (rate as f32 * OUTPUT_BUFFER_SECS) as usize;
        let buffer = Arc::new(Mutex::new(OutputBuffer {
            stereo: Vec::with_capacity(max_frames * 2),
            cursors: Default::default(),
            device_rate: rate,
            device_channels: channels,
        }));
        let gain = Arc::new(AtomicU32::new(1.0f32.to_bits()));
        let fill = {
            let buffer = Arc::clone(&buffer);
            let gain = Arc::clone(&gain);
            move |data: &mut [f32]| {
                let mut buf = buffer.lock();
                let to_channels = buf.device_channels.max(1) as usize;

                let device_frames = data.len() / to_channels;
                let source_frames = device_frames * SAMPLE_RATE as usize / buf.device_rate as usize;
                let wanted = (source_frames * 2).min(buf.stereo.len());
                let rendered = render_upmixed_resampled(
                    &buf.stereo[..wanted],
                    SAMPLE_RATE,
                    buf.device_rate,
                    buf.device_channels,
                );
                let len = rendered.len().min(data.len());
                let gain = f32::from_bits(gain.load(Ordering::Relaxed));
                for (out, sample) in data[..len].iter_mut().zip(&rendered[..len]) {
                    *out = (sample * gain).clamp(-1.0, 1.0);
                }
                data[len..].fill(0.0);
                buf.stereo.drain(..wanted);
                for cursor in buf.cursors.values_mut() {
                    *cursor = cursor.saturating_sub(wanted);
                }
            }
        };
        let err = |err| log::warn!("voice: audio output error: {err}");
        let stream_config: cpal::StreamConfig = config.into();
        macro_rules! output {
            ($t:ty, $from_f32:expr) => {{
                let mut scratch: Vec<f32> = Vec::new();
                device.build_output_stream(
                    &stream_config,
                    move |data: &mut [$t], _| {
                        scratch.resize(data.len(), 0.0);
                        fill(&mut scratch);
                        for (out, &s) in data.iter_mut().zip(scratch.iter()) {
                            *out = $from_f32(s);
                        }
                    },
                    err,
                    None,
                )
            }};
        }
        let stream = match format {
            cpal::SampleFormat::F32 => output!(f32, |s: f32| s),
            cpal::SampleFormat::I16 => output!(i16, |s: f32| (s * 32767.0) as i16),
            cpal::SampleFormat::I32 => output!(i32, |s: f32| (s as f64 * 2_147_483_647.0) as i32),
            cpal::SampleFormat::U16 => output!(u16, |s: f32| (s * 32767.0 + 32768.0) as u16),
            other => {
                return Err(AudioError::Unsupported(
                    "output",
                    format!("sample format {other:?}"),
                ));
            }
        }
        .map_err(cpal_err)?;
        stream.play().map_err(cpal_err)?;

        Ok(Self {
            _stream: stream,
            buffer,
            gain,
        })
    }

    pub fn set_gain(&self, gain: f32) {
        self.gain.store(gain.max(0.0).to_bits(), Ordering::Relaxed);
    }

    pub fn push(&self, samples: &[f32]) {
        self.mix(0, samples, 1.0);
    }

    pub fn mix(&self, source: u32, samples: &[f32], volume: f32) {
        let mut buf = self.buffer.lock();
        let at = buf
            .cursors
            .get(&source)
            .copied()
            .unwrap_or(0)
            .min(buf.stereo.len())
            & !1;
        let overlap = (buf.stereo.len() - at).min(samples.len());
        for (out, s) in buf.stereo[at..at + overlap].iter_mut().zip(samples) {
            *out += s * volume;
        }
        buf.stereo
            .extend(samples[overlap..].iter().map(|s| s * volume));
        buf.cursors.insert(source, at + samples.len());

        let max = (buf.device_rate as f32 * OUTPUT_BUFFER_SECS) as usize * 2;
        if buf.stereo.len() > max {
            let excess = buf.stereo.len() - max;
            buf.stereo.drain(..excess);
            for cursor in buf.cursors.values_mut() {
                *cursor = cursor.saturating_sub(excess);
            }
        }
    }
}

fn push_downmixed_resampled(
    out: &mut Vec<f32>,
    data: &[f32],
    from_rate: u32,
    channels: u16,
    to_rate: u32,
) {
    let channels = channels.max(1) as usize;
    let used = channels.min(2);

    let mono: Vec<f32> = data
        .chunks_exact(channels)
        .map(|frame| frame[..used].iter().sum::<f32>() / used as f32)
        .collect();
    resample_into(out, &mono, from_rate, to_rate);
}

fn resample_into(out: &mut Vec<f32>, input: &[f32], from_rate: u32, to_rate: u32) {
    if from_rate == to_rate {
        out.extend_from_slice(input);
        return;
    }
    let step = from_rate as f64 / to_rate as f64;
    let mut pos = 0.0f64;
    while (pos as usize) + 1 < input.len() {
        let i = pos as usize;
        let frac = (pos - i as f64) as f32;
        out.push(input[i] * (1.0 - frac) + input[i + 1] * frac);
        pos += step;
    }
}

fn render_upmixed_resampled(
    stereo: &[f32],
    from_rate: u32,
    to_rate: u32,
    to_channels: u16,
) -> Vec<f32> {
    let to_channels = to_channels.max(1) as usize;

    let left: Vec<f32> = stereo.iter().step_by(2).copied().collect();
    let right: Vec<f32> = stereo.iter().skip(1).step_by(2).copied().collect();
    let mut l = Vec::new();
    let mut r = Vec::new();
    resample_into(&mut l, &left, from_rate, to_rate);
    resample_into(&mut r, &right, from_rate, to_rate);

    let frames = l.len().min(r.len());
    let mut out = Vec::with_capacity(frames * to_channels);
    for i in 0..frames {
        let (l, r) = (l[i], r[i]);
        if to_channels >= 2 {
            out.push(l);
            out.push(r);
        } else {
            out.push((l + r) * 0.5);
        }
        for _ in 2..to_channels {
            out.push((l + r) * 0.5);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_identity_is_copy() {
        let mut out = Vec::new();
        resample_into(&mut out, &[1.0, 2.0, 3.0], 48000, 48000);
        assert_eq!(out, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn resample_2x_decimation() {
        let mut out = Vec::new();
        resample_into(&mut out, &[0.0, 1.0, 2.0, 3.0], 48000, 24000);

        assert_eq!(out, vec![0.0, 2.0]);
    }

    #[test]
    fn a_six_input_interface_uses_its_first_pair() {
        let mut out = Vec::new();
        push_downmixed_resampled(&mut out, &[1.0, 3.0, 9.0, 9.0, 9.0, 9.0], 48000, 6, 48000);
        assert_eq!(out, vec![2.0]);
    }

    #[test]
    fn downmix_stereo_to_mono() {
        let mut out = Vec::new();

        push_downmixed_resampled(
            &mut out,
            &[0.0, 2.0, 1.0, 3.0, 2.0, 4.0, 3.0, 5.0],
            48000,
            2,
            48000,
        );
        assert_eq!(out, vec![1.0, 2.0, 3.0, 4.0]);
    }
}
