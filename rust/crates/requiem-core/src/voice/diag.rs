use crate::utils::prefs;
use crate::utils::voice_settings;
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;
use std::time;

const RECORDING_SAMPLES: usize = 5 * 60 * 48_000;

pub fn log_dir() -> Option<PathBuf> {
    prefs::config_dir().map(|dir| dir.join("voice-logs"))
}

static LOG: Mutex<Option<fs::File>> = Mutex::new(None);

pub fn write_log(level: log::Level, target: &str, message: &std::fmt::Arguments) {
    if !voice_settings::get().debug_logging {
        *LOG.lock().expect("voice log poisoned") = None;
        return;
    }
    let mut slot = LOG.lock().expect("voice log poisoned");
    if slot.is_none() {
        let Some(dir) = log_dir() else { return };
        let _ = fs::create_dir_all(&dir);
        *slot = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("voice.log"))
            .ok();
    }
    if let Some(file) = slot.as_mut() {
        let at = SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let _ = writeln!(file, "{at} {level} {target}: {message}");
        let _ = file.flush();
    }
}

pub struct AudioRecording {
    samples: VecDeque<i16>,
}

impl Default for AudioRecording {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioRecording {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::new(),
        }
    }

    pub fn push(&mut self, frame: &[f32]) {
        self.samples
            .extend(frame.iter().map(|s| (s.clamp(-1.0, 1.0) * 32767.0) as i16));
        let excess = self.samples.len().saturating_sub(RECORDING_SAMPLES);
        self.samples.drain(..excess);
    }

    pub fn save(&self) {
        if self.samples.is_empty() {
            return;
        }
        let Some(dir) = log_dir() else { return };
        let _ = fs::create_dir_all(&dir);
        let at = SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let path = dir.join(format!("voice-recording-{at}.wav"));
        let data_len = (self.samples.len() * 2) as u32;
        let mut bytes = Vec::with_capacity(44 + data_len as usize);
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&48_000u32.to_le_bytes());
        bytes.extend_from_slice(&(48_000u32 * 2).to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());
        for sample in &self.samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        match fs::write(&path, bytes) {
            Ok(()) => log::info!("voice: diagnostic recording saved to {}", path.display()),
            Err(err) => log::warn!("voice: could not save the diagnostic recording: {err}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_recording_keeps_only_the_last_five_minutes() {
        let mut recording = AudioRecording::new();
        let frame = vec![0.5f32; 960];
        for _ in 0..(RECORDING_SAMPLES / 960 + 10) {
            recording.push(&frame);
        }
        assert_eq!(recording.samples.len(), RECORDING_SAMPLES);
    }
}
