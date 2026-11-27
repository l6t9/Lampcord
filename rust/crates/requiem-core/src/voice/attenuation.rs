use crate::utils::voice_settings;
use std::sync::Once;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::Builder;

static SELF_TALKING: AtomicBool = AtomicBool::new(false);
static OTHERS_TALKING: AtomicBool = AtomicBool::new(false);
static START: Once = Once::new();

pub fn set_talking(self_talking: bool, others_talking: bool) {
    SELF_TALKING.store(self_talking, Ordering::Relaxed);
    OTHERS_TALKING.store(others_talking, Ordering::Relaxed);
    if self_talking || others_talking {
        START.call_once(|| {
            let _ = Builder::new()
                .name("voice-attenuation".into())
                .spawn(platform::run);
        });
    }
}

fn wanted() -> Option<f32> {
    let settings = voice_settings::get();
    if settings.attenuation == 0 {
        return None;
    }
    let duck = (settings.attenuate_while_speaking_self && SELF_TALKING.load(Ordering::Relaxed))
        || (settings.attenuate_while_speaking_others && OTHERS_TALKING.load(Ordering::Relaxed));
    duck.then_some(f32::from(settings.attenuation) / 100.0)
}

#[cfg(windows)]
mod platform {
    use std::collections::HashMap;
    use std::ptr;
    use std::thread;
    use std::time::Duration;
    use windows::core;

    use windows::Win32::Media::Audio::{
        IAudioSessionControl2, IAudioSessionManager2, IMMDeviceEnumerator, ISimpleAudioVolume,
        MMDeviceEnumerator, eConsole, eRender,
    };
    use windows::Win32::System::Com::{
        CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
    };
    use windows::Win32::System::Threading::GetCurrentProcessId;
    use windows::core::Interface;

    type Ducked = HashMap<u32, f32>;

    pub fn run() {
        // SAFETY: CoInitializeEx on this dedicated thread, which runs for the life of the process,
        // so there is no matching uninitialise.
        let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        let mut ducked: Ducked = HashMap::new();
        loop {
            match super::wanted() {
                Some(amount) => {
                    if let Err(err) = duck(&mut ducked, 1.0 - amount) {
                        log::debug!("voice attenuation: {err}");
                    }
                }
                None if !ducked.is_empty() => restore(&mut ducked),
                None => {}
            }
            thread::sleep(Duration::from_millis(60));
        }
    }

    fn sessions() -> core::Result<Vec<(u32, ISimpleAudioVolume)>> {
        // SAFETY: COM was initialised on the calling thread by `run`; every interface is created
        // and owned within this call.
        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
            let manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)?;
            let list = manager.GetSessionEnumerator()?;
            let own = GetCurrentProcessId();
            let mut out = Vec::new();
            for i in 0..list.GetCount()? {
                let control = list.GetSession(i)?;
                let Ok(control2) = control.cast::<IAudioSessionControl2>() else {
                    continue;
                };
                let Ok(pid) = control2.GetProcessId() else {
                    continue;
                };

                if pid == 0 || pid == own {
                    continue;
                }
                if let Ok(volume) = control.cast::<ISimpleAudioVolume>() {
                    out.push((pid, volume));
                }
            }
            Ok(out)
        }
    }

    fn duck(ducked: &mut Ducked, factor: f32) -> core::Result<()> {
        for (pid, volume) in sessions()? {
            // SAFETY: `volume` is a live ISimpleAudioVolume from the session list; a null event
            // context is allowed.
            unsafe {
                let original = match ducked.get(&pid) {
                    Some(original) => *original,
                    None => {
                        let current = volume.GetMasterVolume()?;
                        ducked.insert(pid, current);
                        current
                    }
                };
                let target = original * factor;
                if (volume.GetMasterVolume()? - target).abs() > 0.005 {
                    volume.SetMasterVolume(target, ptr::null())?;
                }
            }
        }
        Ok(())
    }

    fn restore(ducked: &mut Ducked) {
        if let Ok(list) = sessions() {
            for (pid, volume) in list {
                if let Some(original) = ducked.get(&pid) {
                    // SAFETY: `volume` is a live ISimpleAudioVolume from the session list; a null
                    // event context is allowed.
                    let _ = unsafe { volume.SetMasterVolume(*original, ptr::null()) };
                }
            }
        }
        ducked.clear();
    }
}

/// PulseAudio, and PipeWire through its PulseAudio server, scale each app's stream volume. They
/// are driven with `pactl`, so no audio library is linked.
#[cfg(all(unix, not(target_os = "macos")))]
mod platform {
    use std::collections::HashMap;
    use std::process::{Command, Stdio};
    use std::time::Duration;

    /// Sink-input index to the volume it had before it was ducked.
    type Ducked = HashMap<u64, u64>;

    pub fn run() {
        let mut ducked: Ducked = HashMap::new();
        loop {
            match super::wanted() {
                Some(amount) => duck(&mut ducked, 1.0 - amount),
                None if !ducked.is_empty() => restore(&mut ducked),
                None => {}
            }
            // Each pass starts `pactl`, so this is slower than the Windows loop.
            std::thread::sleep(Duration::from_millis(250));
        }
    }

    struct Input {
        index: u64,
        volume: u64,
    }

    fn inputs() -> Vec<Input> {
        let Ok(output) = Command::new("pactl")
            .args(["--format=json", "list", "sink-inputs"])
            .stderr(Stdio::null())
            .output()
        else {
            return Vec::new();
        };
        parse_inputs(&output.stdout, std::process::id())
    }

    /// The streams other apps are playing, each with its loudest channel's volume.
    fn parse_inputs(json: &[u8], own_pid: u32) -> Vec<Input> {
        let Ok(serde_json::Value::Array(list)) = serde_json::from_slice(json) else {
            return Vec::new();
        };
        list.iter()
            .filter_map(|input| {
                let index = input["index"].as_u64()?;
                let pid = input["properties"]["application.process.id"]
                    .as_str()
                    .and_then(|pid| pid.parse::<u32>().ok());
                if pid == Some(own_pid) {
                    return None;
                }
                let volume = input["volume"]
                    .as_object()?
                    .values()
                    .filter_map(|channel| channel["value"].as_u64())
                    .max()?;
                Some(Input { index, volume })
            })
            .collect()
    }

    fn set_volume(index: u64, volume: u64) {
        let _ = Command::new("pactl")
            .args(["set-sink-input-volume", &index.to_string(), &volume.to_string()])
            .stderr(Stdio::null())
            .status();
    }

    fn duck(ducked: &mut Ducked, factor: f32) {
        for input in inputs() {
            let original = *ducked.entry(input.index).or_insert(input.volume);
            let target = (original as f32 * factor) as u64;
            if input.volume.abs_diff(target) > 300 {
                set_volume(input.index, target);
            }
        }
    }

    fn restore(ducked: &mut Ducked) {
        let present: Vec<u64> = inputs().into_iter().map(|input| input.index).collect();
        for (index, original) in ducked.drain() {
            if present.contains(&index) {
                set_volume(index, original);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::parse_inputs;

        #[test]
        fn streams_come_from_pactls_json_without_our_own() {
            let json = br#"[
                {"index": 3, "volume": {"front-left": {"value": 65536}, "front-right": {"value": 49152}},
                 "properties": {"application.process.id": "100"}},
                {"index": 4, "volume": {"mono": {"value": 32768}},
                 "properties": {"application.process.id": "200"}}
            ]"#;
            let inputs = parse_inputs(json, 200);
            assert_eq!(inputs.len(), 1);
            assert_eq!((inputs[0].index, inputs[0].volume), (3, 65536));
            assert!(parse_inputs(b"not json", 1).is_empty());
        }
    }
}

/// macOS has no per-app volume to scale, so ducking other apps is not available there.
#[cfg(target_os = "macos")]
mod platform {
    pub fn run() {}
}
