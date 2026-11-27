pub type FillFn = Box<dyn FnMut(&mut [f32], f64) + Send>;

#[cfg(windows)]
pub use wasapi::AudioOut;

#[cfg(not(windows))]
pub use fallback::AudioOut;

#[cfg(windows)]
mod wasapi {
    use cpal::traits::DeviceTrait;
    use cpal::traits::HostTrait;
    use std::ptr;
    use std::slice;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, mpsc};
    use std::thread::Builder;
    use std::thread::JoinHandle;
    use windows::core::Error;
    use windows::core;

    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::Media::Audio::{
        AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_EVENTCALLBACK, IAudioClient, IAudioClock,
        IAudioRenderClient, IMMDeviceEnumerator, MMDeviceEnumerator, WAVEFORMATEX,
        WAVEFORMATEXTENSIBLE, eConsole, eRender,
    };
    use windows::Win32::Media::Multimedia::KSDATAFORMAT_SUBTYPE_IEEE_FLOAT;
    use windows::Win32::System::Com::{
        CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree,
        CoUninitialize,
    };
    use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
    use windows::Win32::System::Threading::{CreateEventA, WaitForSingleObject};
    use windows::core::PCSTR;

    use super::FillFn;

    const WAVE_FORMAT_IEEE_FLOAT: u16 = 3;
    const WAVE_FORMAT_EXTENSIBLE: u16 = 0xFFFE;

    const BUFFER_DURATION_HNS: i64 = 200_000;

    #[derive(Default)]
    struct Control {
        playing: AtomicBool,
        flush: AtomicBool,
        quit: AtomicBool,
    }

    pub struct AudioOut {
        control: Arc<Control>,
        rate: u32,
        channels: u16,
        thread: Option<JoinHandle<()>>,
    }

    impl AudioOut {
        pub fn open(fill: FillFn) -> Option<Self> {
            let control = Arc::new(Control::default());
            let (ready_tx, ready_rx) = mpsc::sync_channel(1);
            let thread_control = control.clone();
            let thread = Builder::new()
                .name("wasapi-render".into())
                .spawn(move || {
                    // SAFETY: CoInitializeEx on this freshly spawned thread; the matching
                    // CoUninitialize runs below when it succeeded.
                    let com = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
                    match Renderer::new() {
                        Ok(renderer) => {
                            let _ = ready_tx.send(Some((renderer.rate, renderer.channels)));
                            renderer.run(&thread_control, fill);
                        }
                        Err(err) => {
                            log::warn!(
                                "WASAPI output unavailable ({err}) — video will play silently"
                            );
                            let _ = ready_tx.send(None);
                        }
                    }
                    if com.is_ok() {
                        // SAFETY: balances the successful CoInitializeEx made at the start of this
                        // thread.
                        unsafe { CoUninitialize() };
                    }
                })
                .ok()?;
            let (rate, channels) = ready_rx.recv().ok().flatten()?;
            Some(Self {
                control,
                rate,
                channels,
                thread: Some(thread),
            })
        }

        pub fn rate(&self) -> u32 {
            self.rate
        }

        pub fn channels(&self) -> u16 {
            self.channels
        }

        pub fn set_playing(&self, playing: bool) {
            self.control.playing.store(playing, Ordering::Release);
        }

        pub fn flush(&self) {
            self.control.flush.store(true, Ordering::Release);
        }
    }

    impl Drop for AudioOut {
        fn drop(&mut self) {
            self.control.quit.store(true, Ordering::Release);
            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }
        }
    }

    struct Renderer {
        client: IAudioClient,
        render: IAudioRenderClient,
        clock: IAudioClock,
        event: HANDLE,
        buffer_frames: u32,
        rate: u32,
        channels: u16,
        clock_freq: u64,
        qpc_freq: i64,
    }

    impl Renderer {
        fn new() -> core::Result<Self> {
            // SAFETY: COM was initialised on this thread by `AudioOut::open`. Every interface used
            // here is created and owned here, and the mix format from GetMixFormat is freed once
            // with CoTaskMemFree.
            unsafe {
                let enumerator: IMMDeviceEnumerator =
                    CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
                let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
                let client: IAudioClient = device.Activate(CLSCTX_ALL, None)?;
                let format = client.GetMixFormat()?;
                let result = Self::init_with_format(client, format);
                CoTaskMemFree(Some(format as *const _));
                result
            }
        }

        unsafe fn init_with_format(
            client: IAudioClient,
            format: *mut WAVEFORMATEX,
        ) -> core::Result<Self> {
            // SAFETY: `format` comes from GetMixFormat, so it points to a WAVEFORMATEX; the read
            // does not require alignment.
            let base = unsafe { ptr::read_unaligned(format) };
            let is_float = match base.wFormatTag {
                WAVE_FORMAT_IEEE_FLOAT => true,
                WAVE_FORMAT_EXTENSIBLE => {
                    let ext =
                        // SAFETY: when wFormatTag is WAVE_FORMAT_EXTENSIBLE, `format` points to a
                        // full WAVEFORMATEXTENSIBLE.
                        unsafe { ptr::read_unaligned(format as *const WAVEFORMATEXTENSIBLE) };
                    let sub = ext.SubFormat;
                    sub == KSDATAFORMAT_SUBTYPE_IEEE_FLOAT
                }
                _ => false,
            };
            if !is_float || base.wBitsPerSample != 32 {
                return Err(Error::new(
                    core::HRESULT(-1),
                    "device mix format isn't 32-bit float",
                ));
            }
            // SAFETY: `client` is a live IAudioClient and `format` is the valid mix format from
            // GetMixFormat.
            unsafe {
                client.Initialize(
                    AUDCLNT_SHAREMODE_SHARED,
                    AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
                    BUFFER_DURATION_HNS,
                    0,
                    format,
                    None,
                )?;
            }
            // SAFETY: COM call on the live WASAPI interfaces this renderer owns, with valid locals
            // as arguments.
            let event = unsafe { CreateEventA(None, false, false, PCSTR::null())? };
            unsafe { client.SetEventHandle(event)? };
            let buffer_frames = unsafe { client.GetBufferSize()? };
            let render: IAudioRenderClient = unsafe { client.GetService()? };
            let clock: IAudioClock = unsafe { client.GetService()? };
            let clock_freq = unsafe { clock.GetFrequency()? };
            let mut qpc_freq = 0i64;
            unsafe { QueryPerformanceFrequency(&mut qpc_freq)? };
            let stream_latency = unsafe { client.GetStreamLatency() }.unwrap_or(0);
            let (rate, channels) = (base.nSamplesPerSec, base.nChannels);
            let device_name = {
                cpal::default_host()
                    .default_output_device()
                    .and_then(|d| d.name().ok())
                    .unwrap_or_default()
            };
            log::info!(
                "WASAPI output ({device_name}): {} Hz, {} ch, buffer {} frames ({:.1}ms), stream latency {:.1}ms",
                rate,
                channels,
                buffer_frames,
                buffer_frames as f64 * 1000.0 / rate as f64,
                stream_latency as f64 / 10_000.0
            );
            Ok(Self {
                client,
                render,
                clock,
                event,
                buffer_frames,
                rate,
                channels,
                clock_freq,
                qpc_freq,
            })
        }

        fn device_delay(&self, written_frames: u64) -> f64 {
            let (mut position, mut qpc_position) = (0u64, 0u64);
            // SAFETY: `self.clock` is a live IAudioClock and the out-pointers are valid locals.
            if unsafe {
                self.clock
                    .GetPosition(&mut position, Some(&mut qpc_position))
            }
            .is_err()
            {
                return 0.0;
            }
            let played_frames =
                (position as u128 * self.rate as u128 / self.clock_freq.max(1) as u128) as u64;
            let mut delay = written_frames.saturating_sub(played_frames) as f64 / self.rate as f64;
            let mut now = 0i64;
            // SAFETY: QueryPerformanceCounter writes into a valid local.
            if unsafe { QueryPerformanceCounter(&mut now) }.is_ok() {
                let now_hns = (now as i128 * 10_000_000 / self.qpc_freq.max(1) as i128) as i64;
                let age_hns = now_hns - qpc_position as i64;

                if (0..10 * 10_000_000).contains(&age_hns) {
                    delay -= age_hns as f64 / 10_000_000.0;
                }
            }
            delay.max(0.0)
        }

        fn fill_available(
            &self,
            fill: &mut FillFn,
            written_frames: &mut u64,
            started: bool,
        ) -> core::Result<()> {
            // SAFETY: the WASAPI interfaces are owned by this renderer. GetBuffer returns
            // `available` frames of `channels` f32 samples (the mix format is 32-bit float, checked
            // in `init_with_format`), valid until ReleaseBuffer.
            unsafe {
                let padding = self.client.GetCurrentPadding()?;
                let available = self.buffer_frames.saturating_sub(padding);
                if available == 0 {
                    return Ok(());
                }

                let delay = if started {
                    self.device_delay(*written_frames)
                } else {
                    f64::NAN
                };
                let data = self.render.GetBuffer(available)?;
                let samples = slice::from_raw_parts_mut(
                    data as *mut f32,
                    available as usize * self.channels as usize,
                );
                fill(samples, delay);
                self.render.ReleaseBuffer(available, 0)?;
                *written_frames += available as u64;
            }
            Ok(())
        }

        fn run(self, control: &Control, mut fill: FillFn) {
            let mut started = false;
            let mut written_frames: u64 = 0;
            let result = (|| -> core::Result<()> {
                loop {
                    // SAFETY: `self.event` is the live event handle created in `init_with_format`.
                    unsafe { WaitForSingleObject(self.event, 5) };
                    if control.quit.load(Ordering::Acquire) {
                        return Ok(());
                    }
                    if control.flush.swap(false, Ordering::AcqRel) {
                        // SAFETY: COM call on the live WASAPI interfaces this renderer owns, with
                        // valid locals as arguments.
                        unsafe {
                            if started {
                                self.client.Stop()?;
                                started = false;
                            }

                            self.client.Reset()?;
                        }
                        written_frames = 0;
                    }
                    let want = control.playing.load(Ordering::Acquire);
                    if want && !started {
                        self.fill_available(&mut fill, &mut written_frames, false)?;
                        // SAFETY: COM call on the live WASAPI interfaces this renderer owns, with
                        // valid locals as arguments.
                        unsafe { self.client.Start()? };
                        started = true;
                        continue;
                    }
                    if !want && started {
                        // SAFETY: COM call on the live WASAPI interfaces this renderer owns, with
                        // valid locals as arguments.
                        unsafe { self.client.Stop()? };
                        started = false;
                    }
                    if started {
                        self.fill_available(&mut fill, &mut written_frames, true)?;
                    }
                }
            })();
            if let Err(err) = result {
                log::warn!("WASAPI output stopped: {err}");
            }
            // SAFETY: last use of the client and event handle: the renderer is consumed here, so
            // nothing else touches them.
            unsafe {
                let _ = self.client.Stop();
                let _ = CloseHandle(self.event);
            }
        }
    }
}

#[cfg(not(windows))]
mod fallback {
    use std::cell::{Cell, RefCell};
    use std::sync::{Arc, Mutex};

    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    use super::FillFn;

    pub struct AudioOut {
        device: cpal::Device,
        config: cpal::StreamConfig,
        fill: Arc<Mutex<FillFn>>,
        stream: RefCell<cpal::Stream>,
        playing: Cell<bool>,
    }

    impl AudioOut {
        pub fn open(fill: FillFn) -> Option<Self> {
            let device = cpal::default_host().default_output_device()?;
            let config = device.default_output_config().ok()?;
            if config.sample_format() != cpal::SampleFormat::F32 {
                log::warn!("default audio output isn't f32 — video will play silently");
                return None;
            }
            let config: cpal::StreamConfig = config.into();
            let fill = Arc::new(Mutex::new(fill));
            let stream = build_stream(&device, &config, &fill)?;
            Some(Self {
                device,
                config,
                fill,
                stream: RefCell::new(stream),
                playing: Cell::new(false),
            })
        }

        pub fn rate(&self) -> u32 {
            self.config.sample_rate.0
        }

        pub fn channels(&self) -> u16 {
            self.config.channels
        }

        pub fn set_playing(&self, playing: bool) {
            self.playing.set(playing);
            let stream = self.stream.borrow();
            if playing {
                let _ = stream.play();
            } else {
                let _ = stream.pause();
            }
        }

        pub fn flush(&self) {
            if let Some(fresh) = build_stream(&self.device, &self.config, &self.fill) {
                *self.stream.borrow_mut() = fresh;
                self.set_playing(self.playing.get());
            }
        }
    }

    fn build_stream(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        fill: &Arc<Mutex<FillFn>>,
    ) -> Option<cpal::Stream> {
        let fill = fill.clone();
        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {
                    let ts = info.timestamp();
                    let delay = ts
                        .playback
                        .duration_since(&ts.callback)
                        .map_or(0.0, |d| d.as_secs_f64());
                    if let Ok(mut fill) = fill.lock() {
                        fill(data, delay);
                    }
                },
                |err| log::warn!("audio output stream error: {err}"),
                None,
            )
            .ok()?;
        let _ = stream.pause();
        Some(stream)
    }
}
