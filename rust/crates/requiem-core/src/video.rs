use crate::net_profile::Resource;
use crate::audio_out::AudioOut;
use crate::audio_out;
use crate::net_profile::get;
use crate::net_profile::get_from;
use ffmpeg_next::codec::context::Context;
use ffmpeg_next::decoder;
use ffmpeg_next::ffi;
use ffmpeg_next::format::Pixel;
use ffmpeg_next::format::Sample;
use ffmpeg_next::format::context;
use ffmpeg_next::format::sample::Type as SampleType;
use ffmpeg_next::format;
use ffmpeg_next::frame::Audio;
use ffmpeg_next::frame::Video;
use ffmpeg_next::media::Type;
use ffmpeg_next::software::resampling::Context as ResamplingContext;
use ffmpeg_next::software::resampling;
use ffmpeg_next::software::scaling::context::Context as ContextContext;
use ffmpeg_next::software::scaling::flag::Flags;
use ffmpeg_next::software::scaling;
use futures_util::StreamExt;
use requiem_ui::timer;
use requiem_ui::waker;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use std::process;
use std::rc::Rc;
#[cfg(windows)]
use std::sync::Once;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use tokio::task;
#[cfg(windows)]
use windows::Win32::Media;

use tokio::runtime::Handle;

const FRAME_SKIP_THRESHOLD: Duration = Duration::from_millis(80);

pub struct VideoFrame {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

struct SharedState {
    frame: Mutex<Option<VideoFrame>>,

    waker: Option<waker::FrameWaker>,

    frame_version: AtomicU64,
    position_micros: AtomicU64,
    duration_micros: AtomicU64,
    playing: AtomicBool,

    ended: AtomicBool,

    failed: AtomicBool,

    muted: AtomicBool,

    volume: AtomicU32,

    audio_queue: Mutex<Option<VecDeque<AudioChunk>>>,

    has_audio: AtomicBool,

    audio_started: AtomicBool,

    audio_rate: AtomicU32,
    audio_channels: AtomicU32,

    audio_newest_queued_pts_micros: AtomicU64,

    audio_output_latency_micros: AtomicU64,

    audio_clock_anchor: Mutex<(f64, Instant)>,

    audio_generation: AtomicU64,

    video_ready_to_play: AtomicBool,
    audio_ready_to_play: AtomicBool,
}

struct AudioChunk {
    generation: u64,
    pts_secs: f64,
    samples: Vec<f32>,
    consumed: usize,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            frame: Mutex::new(None),
            waker: None,
            frame_version: AtomicU64::new(0),
            position_micros: AtomicU64::new(0),
            duration_micros: AtomicU64::new(0),
            playing: AtomicBool::new(false),
            ended: AtomicBool::new(false),
            failed: AtomicBool::new(false),
            muted: AtomicBool::new(false),
            volume: AtomicU32::new(1.0f32.to_bits()),
            audio_queue: Mutex::new(None),
            has_audio: AtomicBool::new(false),
            audio_started: AtomicBool::new(false),
            audio_rate: AtomicU32::new(0),
            audio_channels: AtomicU32::new(0),
            audio_newest_queued_pts_micros: AtomicU64::new(0),
            audio_output_latency_micros: AtomicU64::new(0),
            audio_clock_anchor: Mutex::new((0.0, Instant::now())),
            audio_generation: AtomicU64::new(0),
            video_ready_to_play: AtomicBool::new(false),
            audio_ready_to_play: AtomicBool::new(false),
        }
    }
}

enum Command {
    Play,
    Pause,
    Seek(f64),
}

#[derive(Clone)]
pub struct VideoPlayer {
    shared: Arc<SharedState>,
    cmd_tx: mpsc::Sender<Command>,

    audio_cmd_tx: mpsc::Sender<Command>,
}

impl VideoPlayer {
    pub fn open(
        runtime: Handle,
        http: wreq::Client,
        url: String,
        loop_playback: bool,
        waker: Option<waker::FrameWaker>,
        preload: Option<task::JoinHandle<Option<Partial>>>,
    ) -> Self {
        #[cfg(windows)]
        {
            static TIMER_RESOLUTION: Once = Once::new();
            // SAFETY: timeBeginPeriod has no preconditions, and `Once` makes this the only call.
            TIMER_RESOLUTION.call_once(|| unsafe {
                Media::timeBeginPeriod(1);
            });
        }
        let shared = Arc::new(SharedState {
            waker,
            ..SharedState::default()
        });
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (audio_cmd_tx, audio_cmd_rx) = mpsc::channel();

        let thread_shared = shared.clone();
        thread::spawn(move || {
            let path = match runtime.block_on(async {
                let preloaded = match preload {
                    Some(handle) => handle.await.ok().flatten(),
                    None => None,
                };
                download_to_temp_file(&http, &url, preloaded).await
            }) {
                Ok(path) => path,
                Err(err) => {
                    log::warn!("failed to download video {url}: {err}");
                    thread_shared.failed.store(true, Ordering::Relaxed);
                    return;
                }
            };
            let audio_path = path.clone();
            let audio_shared = thread_shared.clone();
            thread::spawn(move || {
                audio_thread(audio_path, audio_shared, audio_cmd_rx, loop_playback)
            });
            decode_thread(path, thread_shared, cmd_rx, loop_playback);
        });

        Self {
            shared,
            cmd_tx,
            audio_cmd_tx,
        }
    }

    pub fn play(&self) {
        let _ = self.cmd_tx.send(Command::Play);
        let _ = self.audio_cmd_tx.send(Command::Play);
    }

    pub fn pause(&self) {
        let _ = self.cmd_tx.send(Command::Pause);
        let _ = self.audio_cmd_tx.send(Command::Pause);
    }

    pub fn seek(&self, position_secs: f64) {
        self.shared.ended.store(false, Ordering::Relaxed);
        let _ = self.cmd_tx.send(Command::Seek(position_secs));
        let _ = self.audio_cmd_tx.send(Command::Seek(position_secs));
    }

    pub fn is_playing(&self) -> bool {
        self.shared.playing.load(Ordering::Relaxed)
    }

    pub fn has_ended(&self) -> bool {
        self.shared.ended.load(Ordering::Relaxed)
    }

    pub fn set_muted(&self, muted: bool) {
        self.shared.muted.store(muted, Ordering::Relaxed);
    }

    pub fn set_volume(&self, amplitude: f32) {
        self.shared
            .volume
            .store(amplitude.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn is_muted(&self) -> bool {
        self.shared.muted.load(Ordering::Relaxed)
    }

    pub fn has_failed(&self) -> bool {
        self.shared.failed.load(Ordering::Relaxed)
    }

    pub fn position_secs(&self) -> f64 {
        self.shared.position_micros.load(Ordering::Relaxed) as f64 / 1_000_000.0
    }

    pub fn duration_secs(&self) -> f64 {
        self.shared.duration_micros.load(Ordering::Relaxed) as f64 / 1_000_000.0
    }

    pub fn frame_version(&self) -> u64 {
        self.shared.frame_version.load(Ordering::Relaxed)
    }

    pub fn current_frame(&self) -> Option<VideoFrame> {
        let guard = self
            .shared
            .frame
            .lock()
            .expect("video frame mutex poisoned");
        guard.as_ref().map(|f| VideoFrame {
            rgba: f.rgba.clone(),
            width: f.width,
            height: f.height,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Moov {
    Front,

    At(u64),

    NeedMore,
}

fn locate_moov(prefix: &[u8], total: u64) -> Moov {
    let mut off: u64 = 0;
    loop {
        let at = off as usize;
        if at + 8 > prefix.len() {
            return Moov::NeedMore;
        }
        let size32 =
            u32::from_be_bytes([prefix[at], prefix[at + 1], prefix[at + 2], prefix[at + 3]]) as u64;
        let kind = &prefix[at + 4..at + 8];
        if off == 0 && kind != b"ftyp" {
            return Moov::Front;
        }
        let size = match size32 {
            0 => total.saturating_sub(off),
            1 if at + 16 <= prefix.len() => {
                u64::from_be_bytes(prefix[at + 8..at + 16].try_into().unwrap())
            }
            1 => return Moov::NeedMore,
            n => n,
        };
        if kind == b"moov" || size < 8 {
            return Moov::Front;
        }
        let end = off + size;
        if end > prefix.len() as u64 {
            return if kind == b"mdat" && end < total {
                Moov::At(end)
            } else {
                Moov::Front
            };
        }
        off = end;
    }
}

fn align_to_block(offset: u64) -> u64 {
    offset & !0xFFFF
}

fn content_range_total(response: &wreq::Response) -> Option<u64> {
    response
        .headers()
        .get("content-range")?
        .to_str()
        .ok()?
        .rsplit('/')
        .next()?
        .parse()
        .ok()
}

const PRELOAD_BYTES: usize = 3 * 1024 * 1024;

pub struct Partial {
    head: Vec<u8>,

    complete: bool,
}

async fn fetch_head(
    http: &wreq::Client,
    url: &str,
    stop_after: Option<usize>,
) -> Result<Partial, wreq::Error> {
    let response = get(http, url, Resource::Video)
        .send()
        .await?
        .error_for_status()?;
    let ranged = response.status() == wreq::StatusCode::PARTIAL_CONTENT;
    let total = content_range_total(&response)
        .or(response.content_length())
        .unwrap_or(0);
    let mut stream = response.bytes_stream();
    let mut data: Vec<u8> = Vec::with_capacity(total.min(1 << 28) as usize);
    let mut checked = !ranged;
    let mut tail = None;
    let mut complete = true;
    while let Some(chunk) = stream.next().await {
        data.extend_from_slice(&chunk?);
        if !checked {
            match locate_moov(&data, total) {
                Moov::NeedMore => {}
                Moov::Front => checked = true,
                Moov::At(offset) => {
                    checked = true;
                    let (http, url) = (http.clone(), url.to_string());
                    tail = Some(tokio::spawn(async move {
                        get_from(&http, &url, Resource::Video, align_to_block(offset))
                            .send()
                            .await?
                            .error_for_status()?
                            .bytes()
                            .await
                    }));
                }
            }
        }
        if ranged
            && stop_after.is_some_and(|limit| data.len() >= limit)
            && (data.len() as u64) < total
        {
            complete = false;
            break;
        }
    }

    if let Some(tail) = tail {
        if let Ok(Err(err)) = tail.await {
            log::debug!("the index request for {url} failed: {err}");
        }
    }
    Ok(Partial {
        head: data,
        complete,
    })
}

async fn download_to_temp_file(
    http: &wreq::Client,
    url: &str,
    preloaded: Option<Partial>,
) -> Result<PathBuf, wreq::Error> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let mut partial = match preloaded {
        Some(partial) => partial,
        None => fetch_head(http, url, None).await?,
    };
    if !partial.complete {
        let resume = align_to_block(partial.head.len() as u64) as usize;
        partial.head.truncate(resume);
        let response = get_from(
            http,
            url,
            Resource::Video,
            resume as u64,
        )
        .send()
        .await?
        .error_for_status()?;
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            partial.head.extend_from_slice(&chunk?);
        }
    }
    let bytes = partial.head;

    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = env::temp_dir().join(format!("requiem_video_{}_{n}.tmp", process::id()));
    let mut file = File::create(&path)
        .unwrap_or_else(|err| panic!("failed to create temp video file at {path:?}: {err}"));
    file.write_all(&bytes)
        .expect("failed to write temp video file");
    Ok(path)
}

const AUDIO_LOOKAHEAD_CAP_SECS: f64 = 5.0;

fn audio_thread(
    path: PathBuf,
    shared: Arc<SharedState>,
    cmd_rx: mpsc::Receiver<Command>,
    loop_playback: bool,
) {
    let Ok(mut ictx) = format::input(&path) else {
        return;
    };
    let Some(stream) = ictx.streams().best(Type::Audio) else {
        return;
    };
    let stream_index = stream.index();
    let time_base = stream.time_base();
    let Ok(context_decoder) =
        Context::from_parameters(stream.parameters())
    else {
        return;
    };
    let Ok(mut decoder) = context_decoder.decoder().audio() else {
        return;
    };

    let callback_shared = shared.clone();
    let fill: audio_out::FillFn = Box::new(move |data: &mut [f32], delay_secs: f64| {
        let s = &callback_shared;
        if delay_secs.is_finite() {
            s.audio_output_latency_micros
                .store((delay_secs * 1_000_000.0) as u64, Ordering::Relaxed);
        }
        let channels = s.audio_channels.load(Ordering::Relaxed).max(1) as f64;
        let rate = s.audio_rate.load(Ordering::Relaxed).max(1) as f64;
        let muted = s.muted.load(Ordering::Relaxed);
        let gain = f32::from_bits(s.volume.load(Ordering::Relaxed));
        let mut guard = s
            .audio_queue
            .lock()
            .expect("video audio queue mutex poisoned");
        let Some(queue) = guard.as_mut() else {
            data.fill(0.0);
            return;
        };

        let generation = s.audio_generation.load(Ordering::Relaxed);
        while queue.front().is_some_and(|c| c.generation != generation) {
            queue.pop_front();
        }
        let first_pts = queue
            .front()
            .map(|c| c.pts_secs + c.consumed as f64 / channels / rate);
        for sample in data.iter_mut() {
            *sample = match queue.front_mut() {
                Some(front) => {
                    let value = front.samples[front.consumed];
                    front.consumed += 1;
                    if front.consumed == front.samples.len() {
                        queue.pop_front();
                    }
                    if muted { 0.0 } else { value * gain }
                }
                None => 0.0,
            };
        }
        drop(guard);

        if let Some(pts) = first_pts.filter(|_| delay_secs.is_finite()) {
            *s.audio_clock_anchor
                .lock()
                .expect("video audio clock anchor mutex poisoned") =
                (pts - delay_secs, Instant::now());
            s.audio_started.store(true, Ordering::Relaxed);
        }
    });
    let Some(out) = AudioOut::open(fill) else {
        return;
    };
    let out_channels = out.channels() as i32;
    let out_rate = out.rate();

    let (in_format, in_layout, in_rate) =
        (decoder.format(), decoder.channel_layout(), decoder.rate());
    let make_resampler = || {
        ResamplingContext::get(
            in_format,
            in_layout,
            in_rate,
            Sample::F32(SampleType::Packed),
            ffmpeg_next::ChannelLayout::default(out_channels),
            out_rate,
        )
        .inspect_err(|err| {
            log::warn!("failed to build audio resampler: {err} — video will play silently")
        })
        .ok()
    };
    let Some(mut resampler) = make_resampler() else {
        return;
    };

    *shared
        .audio_queue
        .lock()
        .expect("video audio queue mutex poisoned") = Some(VecDeque::new());
    shared
        .audio_channels
        .store(out_channels as u32, Ordering::Relaxed);
    shared.audio_rate.store(out_rate, Ordering::Relaxed);
    shared.has_audio.store(true, Ordering::Relaxed);

    let frame_bytes = out_channels as usize * out_rate as usize;

    let seek_floor: Cell<Option<f64>> = Cell::new(None);
    let push_resampled = |shared: &Arc<SharedState>, pts_secs: f64, samples: &[f32]| {
        let (mut pts_secs, mut samples) = (pts_secs, samples);
        if let Some(floor) = seek_floor.get() {
            let chunk_secs = samples.len() as f64 / out_channels as f64 / out_rate as f64;
            if pts_secs + chunk_secs <= floor {
                return;
            }
            if pts_secs < floor {
                let skip = (((floor - pts_secs) * out_rate as f64) as usize
                    * out_channels as usize)
                    .min(samples.len());
                samples = &samples[skip..];
                pts_secs = floor;
            }
            seek_floor.set(None);
        }
        let generation = shared.audio_generation.load(Ordering::Relaxed);
        if let Some(queue) = shared
            .audio_queue
            .lock()
            .expect("video audio queue mutex poisoned")
            .as_mut()
        {
            queue.push_back(AudioChunk {
                generation,
                pts_secs,
                samples: samples.to_vec(),
                consumed: 0,
            });
        }
        let chunk_secs = samples.len() as f64 / out_channels as f64 / out_rate as f64;
        shared.audio_newest_queued_pts_micros.store(
            ((pts_secs + chunk_secs).max(0.0) * 1_000_000.0) as u64,
            Ordering::Relaxed,
        );
    };

    let resample_and_push = |resampler: &mut resampling::Context,
                             decoded: &Audio,
                             shared: &Arc<SharedState>| {
        let pts_secs = decoded
            .pts()
            .map_or(0.0, |p| p as f64 * f64::from(time_base));
        if let Some((resampled, buffered_secs)) = resample(resampler, Some(decoded), out_channels) {
            push_resampled(shared, pts_secs - buffered_secs, resampled.plane(0));
        }
    };
    let flush_audio = |decoder: &mut decoder::Audio,
                       resampler: &mut resampling::Context,
                       shared: &Arc<SharedState>| {
        decoder.send_eof().ok();
        let mut decoded_audio = Audio::empty();
        while decoder.receive_frame(&mut decoded_audio).is_ok() {
            resample_and_push(resampler, &decoded_audio, shared);
        }

        while let Some((tail, _)) = resample(resampler, None, out_channels) {
            let pts_secs = shared
                .audio_newest_queued_pts_micros
                .load(Ordering::Relaxed) as f64
                / 1_000_000.0;
            push_resampled(shared, pts_secs, tail.plane(0));
        }
    };
    let seek = |ictx: &mut context::Input,
                decoder: &mut decoder::Audio,
                resampler: &mut resampling::Context,
                shared: &Arc<SharedState>,
                secs: f64| {
        if let Err(err) = ictx.seek((secs * 1_000_000.0) as i64, ..) {
            log::warn!("audio seek to {secs:.3}s failed: {err}");
            return;
        }
        decoder.flush();

        if let Some(fresh) = make_resampler() {
            *resampler = fresh;
        }
        shared.audio_generation.fetch_add(1, Ordering::Relaxed);
        seek_floor.set(Some(secs.max(0.0)));
        if let Some(queue) = shared
            .audio_queue
            .lock()
            .expect("video audio queue mutex poisoned")
            .as_mut()
        {
            queue.clear();
        }
        shared
            .audio_newest_queued_pts_micros
            .store((secs * 1_000_000.0).max(0.0) as u64, Ordering::Relaxed);

        shared.audio_started.store(false, Ordering::Relaxed);

        out.flush();
    };
    let pause = |shared: &Arc<SharedState>| {
        out.set_playing(false);
        shared.audio_started.store(false, Ordering::Relaxed);
        shared.audio_ready_to_play.store(false, Ordering::Relaxed);
    };

    let mut playing = false;
    loop {
        if !playing {
            match cmd_rx.recv() {
                Ok(Command::Play) => {
                    rendezvous_play_start(&shared.audio_ready_to_play, &shared.video_ready_to_play);
                    playing = true;
                    shared.audio_started.store(false, Ordering::Relaxed);
                    out.set_playing(true);
                }
                Ok(Command::Pause) => continue,
                Ok(Command::Seek(secs)) => {
                    seek(&mut ictx, &mut decoder, &mut resampler, &shared, secs)
                }
                Err(_) => break,
            }
            continue;
        }

        match cmd_rx.try_recv() {
            Ok(Command::Pause) => {
                playing = false;
                pause(&shared);
                continue;
            }
            Ok(Command::Seek(secs)) => {
                seek(&mut ictx, &mut decoder, &mut resampler, &shared, secs);
                continue;
            }
            Ok(Command::Play) | Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => break,
        }

        let buffered_secs = shared
            .audio_queue
            .lock()
            .expect("video audio queue mutex poisoned")
            .as_ref()
            .map_or(0.0, |q| {
                q.iter()
                    .map(|c| c.samples.len() - c.consumed)
                    .sum::<usize>() as f64
                    / frame_bytes as f64
            });
        if buffered_secs > AUDIO_LOOKAHEAD_CAP_SECS {
            match cmd_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(Command::Pause) => {
                    playing = false;
                    pause(&shared);
                }
                Ok(Command::Seek(secs)) => {
                    seek(&mut ictx, &mut decoder, &mut resampler, &shared, secs)
                }
                Ok(Command::Play) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
            continue;
        }

        let Some((packet_stream, packet)) = ictx.packets().next() else {
            if loop_playback {
                seek(&mut ictx, &mut decoder, &mut resampler, &shared, 0.0);
                continue;
            }
            flush_audio(&mut decoder, &mut resampler, &shared);

            let mut interrupted = false;
            loop {
                let remaining = shared
                    .audio_queue
                    .lock()
                    .expect("video audio queue mutex poisoned")
                    .as_ref()
                    .map_or(0, |q| {
                        q.iter()
                            .map(|c| c.samples.len() - c.consumed)
                            .sum::<usize>()
                    });
                if remaining == 0 {
                    break;
                }
                match cmd_rx.recv_timeout(Duration::from_millis(20)) {
                    Ok(Command::Pause) => {
                        interrupted = true;
                        break;
                    }
                    Ok(Command::Seek(secs)) => {
                        seek(&mut ictx, &mut decoder, &mut resampler, &shared, secs);
                        interrupted = true;
                        break;
                    }
                    Ok(Command::Play) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => return,
                }
            }
            if !interrupted {
                thread::sleep(
                    Duration::from_micros(
                        shared.audio_output_latency_micros.load(Ordering::Relaxed),
                    )
                    .min(Duration::from_millis(500)),
                );
            }

            playing = false;
            pause(&shared);
            continue;
        };
        if packet_stream.index() != stream_index {
            continue;
        }
        if let Err(err) = decoder.send_packet(&packet) {
            log::warn!("audio decoder rejected a packet, skipping: {err}");
            continue;
        }
        let mut decoded_audio = Audio::empty();
        while decoder.receive_frame(&mut decoded_audio).is_ok() {
            resample_and_push(&mut resampler, &decoded_audio, &shared);
        }
    }
}

fn resample(
    resampler: &mut resampling::Context,
    input: Option<&Audio>,
    out_channels: i32,
) -> Option<(Audio, f64)> {
    let out_rate = resampler.output().rate;

    // SAFETY: `ctx` is the live SwrContext owned by `resampler`; both calls only read its state.
    let (capacity, buffered_secs) = unsafe {
        let ctx = resampler.as_mut_ptr();
        let in_samples = input.map_or(0, |f| f.samples() as i32);
        (
            ffi::swr_get_out_samples(ctx, in_samples),
            ffi::swr_get_delay(ctx, i64::from(out_rate)) as f64 / f64::from(out_rate),
        )
    };
    if capacity <= 0 {
        return None;
    }
    let mut out = Audio::new(
        Sample::F32(SampleType::Packed),
        capacity as usize,
        ffmpeg_next::ChannelLayout::default(out_channels),
    );
    let converted = match input {
        Some(frame) => resampler.run(frame, &mut out),
        None => resampler.flush(&mut out),
    };
    (converted.is_ok() && out.samples() > 0).then_some((out, buffered_secs))
}

fn seek_to(
    ictx: &mut context::Input,
    decoder: &mut decoder::Video,
    target_secs: f64,
) -> Result<(), ffmpeg_next::Error> {
    let ts = (target_secs * 1_000_000.0) as i64;
    ictx.seek(ts, ..)?;
    decoder.flush();
    Ok(())
}

fn audio_clock_secs(shared: &SharedState) -> Option<f64> {
    if !shared.has_audio.load(Ordering::Relaxed) || !shared.audio_started.load(Ordering::Relaxed) {
        return None;
    }
    let (anchor_pts, anchor_wall) = *shared
        .audio_clock_anchor
        .lock()
        .expect("video audio clock anchor mutex poisoned");
    Some(
        (anchor_pts
            + Instant::now()
                .saturating_duration_since(anchor_wall)
                .as_secs_f64())
        .max(0.0),
    )
}

const RENDEZVOUS_TIMEOUT: Duration = Duration::from_millis(750);

fn rendezvous_play_start(own_ready: &AtomicBool, other_ready: &AtomicBool) {
    own_ready.store(true, Ordering::Relaxed);
    let deadline = Instant::now() + RENDEZVOUS_TIMEOUT;
    while Instant::now() < deadline && !other_ready.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(2));
    }
}

fn pace_remaining(
    shared: &SharedState,
    base_instant: Instant,
    base_position: f64,
    pts_secs: f64,
) -> (Duration, bool) {
    if let Some(audio_now) = audio_clock_secs(shared) {
        let diff = pts_secs - audio_now;
        if diff <= 0.0 {
            (Duration::ZERO, -diff > FRAME_SKIP_THRESHOLD.as_secs_f64())
        } else {
            (Duration::from_secs_f64(diff), false)
        }
    } else {
        let target = base_instant + Duration::from_secs_f64((pts_secs - base_position).max(0.0));
        let now = Instant::now();
        if target > now {
            (target - now, false)
        } else {
            (
                Duration::ZERO,
                now.duration_since(target) > FRAME_SKIP_THRESHOLD,
            )
        }
    }
}

fn coalesce_seeks(cmd_rx: &mpsc::Receiver<Command>, mut secs: f64) -> (f64, Option<Command>) {
    loop {
        match cmd_rx.try_recv() {
            Ok(Command::Seek(next)) => secs = next,
            Ok(other) => return (secs, Some(other)),
            Err(_) => return (secs, None),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn decode_preview_frame(
    ictx: &mut context::Input,
    decoder: &mut decoder::Video,
    scaler: &mut scaling::Context,
    stream_index: usize,
    time_base: ffmpeg_next::Rational,
    width: u32,
    height: u32,
    shared: &Arc<SharedState>,
    target_secs: f64,
) -> Result<Option<f64>, ffmpeg_next::Error> {
    let mut decoded = Video::empty();
    let mut last: Option<(Video, f64)> = None;
    while let Some((stream, packet)) = ictx.packets().next() {
        if stream.index() != stream_index || decoder.send_packet(&packet).is_err() {
            continue;
        }
        while decoder.receive_frame(&mut decoded).is_ok() {
            let pts = decoded
                .pts()
                .map_or(target_secs, |p| p as f64 * f64::from(time_base));
            if pts + 0.001 >= target_secs {
                publish_video_frame(&decoded, scaler, width, height, shared, pts)?;
                return Ok(Some(pts));
            }
            last = Some((decoded.clone(), pts));
        }
    }
    match last {
        Some((frame, pts)) => {
            publish_video_frame(&frame, scaler, width, height, shared, pts)?;
            Ok(Some(pts))
        }
        None => Ok(None),
    }
}

fn publish_video_frame(
    decoded: &Video,
    scaler: &mut scaling::Context,
    width: u32,
    height: u32,
    shared: &Arc<SharedState>,
    pts_secs: f64,
) -> Result<(), ffmpeg_next::Error> {
    let mut rgb = Video::empty();
    scaler.run(decoded, &mut rgb)?;
    let stride = rgb.stride(0);
    let data = rgb.data(0);
    let row_bytes = width as usize * 4;
    let mut buf = vec![0u8; row_bytes * height as usize];
    for row in 0..height as usize {
        let src_start = row * stride;
        buf[row * row_bytes..(row + 1) * row_bytes]
            .copy_from_slice(&data[src_start..src_start + row_bytes]);
    }
    *shared.frame.lock().expect("video frame mutex poisoned") = Some(VideoFrame {
        rgba: buf,
        width,
        height,
    });
    shared.frame_version.fetch_add(1, Ordering::Relaxed);
    shared
        .position_micros
        .store((pts_secs * 1_000_000.0).max(0.0) as u64, Ordering::Relaxed);
    if let Some(waker) = &shared.waker {
        waker.wake();
    }
    Ok(())
}

fn decode_thread(
    path: PathBuf,
    shared: Arc<SharedState>,
    cmd_rx: mpsc::Receiver<Command>,
    loop_playback: bool,
) {
    let result = decode_loop(&path, &shared, &cmd_rx, loop_playback);
    if result.is_err() {
        shared.failed.store(true, Ordering::Relaxed);
    }
    let _ = fs::remove_file(&path);
}

fn decode_loop(
    path: &PathBuf,
    shared: &Arc<SharedState>,
    cmd_rx: &mpsc::Receiver<Command>,
    loop_playback: bool,
) -> Result<(), ffmpeg_next::Error> {
    ffmpeg_next::init()?;
    let mut ictx = format::input(path)?;
    let stream_index = ictx
        .streams()
        .best(Type::Video)
        .ok_or(ffmpeg_next::Error::StreamNotFound)?
        .index();
    let time_base = ictx
        .stream(stream_index)
        .expect("stream_index came from this same ictx")
        .time_base();

    let context_decoder = Context::from_parameters(
        ictx.stream(stream_index).unwrap().parameters(),
    )?;
    let mut decoder = context_decoder.decoder().video()?;
    let width = decoder.width();
    let height = decoder.height();
    let mut scaler = ContextContext::get(
        decoder.format(),
        width,
        height,
        Pixel::RGBA,
        width,
        height,
        Flags::BILINEAR,
    )?;

    let video_stream_duration = ictx
        .stream(stream_index)
        .expect("stream_index came from this same ictx")
        .duration();
    let duration_micros = if video_stream_duration > 0 {
        (video_stream_duration as f64 * f64::from(time_base) * 1_000_000.0) as u64
    } else {
        ictx.duration().max(0) as u64
    };
    shared
        .duration_micros
        .store(duration_micros, Ordering::Relaxed);

    let mut playing = false;

    let mut seek_floor: Option<f64> = None;
    let mut base_instant = Instant::now();
    let mut base_position = 0.0f64;

    let mut pending: Option<Command> = None;

    'outer: loop {
        if !playing {
            let next = match pending.take() {
                Some(cmd) => Ok(cmd),
                None => cmd_rx.recv(),
            };
            match next {
                Ok(Command::Play) => {
                    rendezvous_play_start(&shared.video_ready_to_play, &shared.audio_ready_to_play);
                    playing = true;
                    shared.playing.store(true, Ordering::Relaxed);
                    shared.ended.store(false, Ordering::Relaxed);
                    base_instant = Instant::now();
                    base_position =
                        shared.position_micros.load(Ordering::Relaxed) as f64 / 1_000_000.0;
                }
                Ok(Command::Pause) => continue,
                Ok(Command::Seek(secs)) => {
                    let (secs, next) = coalesce_seeks(cmd_rx, secs);
                    pending = next;
                    if let Err(err) = seek_to(&mut ictx, &mut decoder, secs) {
                        log::warn!("video seek failed, ignoring: {err}");
                    } else {
                        shared.ended.store(false, Ordering::Relaxed);
                        shared
                            .position_micros
                            .store((secs * 1_000_000.0) as u64, Ordering::Relaxed);
                        base_position = secs;

                        seek_floor = None;
                        match decode_preview_frame(
                            &mut ictx,
                            &mut decoder,
                            &mut scaler,
                            stream_index,
                            time_base,
                            width,
                            height,
                            shared,
                            secs,
                        ) {
                            Ok(Some(pts)) => base_position = pts,
                            Ok(None) => {}
                            Err(err) => {
                                log::warn!("failed to decode preview frame after seek: {err}")
                            }
                        }
                    }
                }
                Err(_) => break 'outer,
            }
            continue;
        }

        match cmd_rx.try_recv() {
            Ok(Command::Pause) => {
                playing = false;
                shared.playing.store(false, Ordering::Relaxed);
                shared.video_ready_to_play.store(false, Ordering::Relaxed);
                continue;
            }
            Ok(Command::Seek(secs)) => {
                if let Err(err) = seek_to(&mut ictx, &mut decoder, secs) {
                    log::warn!("video seek failed, ignoring: {err}");
                } else {
                    shared.ended.store(false, Ordering::Relaxed);
                    shared
                        .position_micros
                        .store((secs * 1_000_000.0) as u64, Ordering::Relaxed);
                    base_instant = Instant::now();
                    base_position = secs;
                    seek_floor = Some(secs);
                }
                continue;
            }
            Ok(Command::Play) => {}
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => break 'outer,
        }

        let Some((stream, packet)) = ictx.packets().next() else {
            if loop_playback {
                if seek_to(&mut ictx, &mut decoder, 0.0).is_ok() {
                    shared.position_micros.store(0, Ordering::Relaxed);
                    base_instant = Instant::now();
                    base_position = 0.0;
                }
                continue;
            }

            if let Err(err) = decoder.send_eof() {
                log::warn!("video decoder send_eof failed, skipping end-of-stream flush: {err}");
            } else {
                let mut decoded = Video::empty();
                while decoder.receive_frame(&mut decoded).is_ok() {
                    let pts_secs = decoded
                        .pts()
                        .map_or(base_position, |p| p as f64 * f64::from(time_base));
                    let too_late = loop {
                        let (remaining, late) =
                            pace_remaining(shared, base_instant, base_position, pts_secs);
                        if remaining.is_zero() {
                            break late;
                        }
                        match cmd_rx.recv_timeout(remaining.min(Duration::from_millis(50))) {
                            Ok(Command::Pause) => {}
                            Ok(Command::Seek(secs)) => {
                                if seek_to(&mut ictx, &mut decoder, secs).is_ok() {
                                    shared
                                        .position_micros
                                        .store((secs * 1_000_000.0) as u64, Ordering::Relaxed);
                                    base_instant = Instant::now();
                                    base_position = secs;
                                }
                                continue 'outer;
                            }
                            Ok(Command::Play) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                            Err(mpsc::RecvTimeoutError::Disconnected) => break 'outer,
                        }
                    };
                    if too_late {
                        continue;
                    }

                    base_instant = Instant::now();
                    base_position = pts_secs;
                    if let Err(err) =
                        publish_video_frame(&decoded, &mut scaler, width, height, shared, pts_secs)
                    {
                        log::warn!("failed to scale a flushed video frame, skipping: {err}");
                    }
                }
            }
            shared.ended.store(true, Ordering::Relaxed);
            shared.playing.store(false, Ordering::Relaxed);
            shared.video_ready_to_play.store(false, Ordering::Relaxed);
            playing = false;
            continue;
        };
        if stream.index() != stream_index {
            continue;
        }

        if let Err(err) = decoder.send_packet(&packet) {
            log::warn!("video decoder rejected a packet, skipping: {err}");
            continue;
        }
        let mut decoded = Video::empty();
        while decoder.receive_frame(&mut decoded).is_ok() {
            let pts_secs = decoded
                .pts()
                .map_or(base_position, |p| p as f64 * f64::from(time_base));
            if let Some(floor) = seek_floor {
                if pts_secs + 0.001 < floor {
                    continue;
                }
                seek_floor = None;
            }

            let too_late = loop {
                let (remaining, late) =
                    pace_remaining(shared, base_instant, base_position, pts_secs);
                if remaining.is_zero() {
                    break late;
                }
                match cmd_rx.recv_timeout(remaining.min(Duration::from_millis(50))) {
                    Ok(Command::Pause) => {
                        playing = false;
                        shared.playing.store(false, Ordering::Relaxed);
                        shared.video_ready_to_play.store(false, Ordering::Relaxed);
                        continue 'outer;
                    }
                    Ok(Command::Seek(secs)) => {
                        if seek_to(&mut ictx, &mut decoder, secs).is_ok() {
                            shared.ended.store(false, Ordering::Relaxed);
                            shared
                                .position_micros
                                .store((secs * 1_000_000.0) as u64, Ordering::Relaxed);
                            base_instant = Instant::now();
                            base_position = secs;
                            seek_floor = Some(secs);
                        }
                        continue 'outer;
                    }
                    Ok(Command::Play) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break 'outer,
                }
            };
            if too_late {
                shared
                    .position_micros
                    .store((pts_secs * 1_000_000.0).max(0.0) as u64, Ordering::Relaxed);
                continue;
            }

            base_instant = Instant::now();
            base_position = pts_secs;

            if let Err(err) =
                publish_video_frame(&decoded, &mut scaler, width, height, shared, pts_secs)
            {
                log::warn!("failed to scale a decoded video frame, skipping: {err}");
            }
        }
    }

    Ok(())
}

const IDLE_PLAYER_GRACE: Duration = Duration::from_secs(4);

struct Slot {
    player: VideoPlayer,

    viewers: usize,

    idle_since: Option<Instant>,
}

#[derive(Clone)]
pub struct VideoCache {
    runtime: Handle,
    http: wreq::Client,
    players: Rc<RefCell<HashMap<String, Slot>>>,

    preloads: Rc<
        RefCell<
            HashMap<String, task::JoinHandle<Option<Partial>>>,
        >,
    >,
    waker: Option<waker::FrameWaker>,
}

impl VideoCache {
    pub fn new(runtime: Handle, http: wreq::Client) -> Self {
        Self {
            runtime,
            http,
            players: Rc::new(RefCell::new(HashMap::new())),
            preloads: Rc::new(RefCell::new(HashMap::new())),
            waker: None,
        }
    }

    pub fn with_frame_waker(mut self, waker: waker::FrameWaker) -> Self {
        self.waker = Some(waker);
        self
    }

    pub fn clear(&self) {
        self.players.borrow_mut().clear();
    }

    pub fn preload(&self, url: &str) {
        if self.players.borrow().contains_key(url) || self.preloads.borrow().contains_key(url) {
            return;
        }
        let (http, owned) = (self.http.clone(), url.to_string());
        let handle = self
            .runtime
            .spawn(async move { fetch_head(&http, &owned, Some(PRELOAD_BYTES)).await.ok() });
        self.preloads.borrow_mut().insert(url.to_string(), handle);
    }

    pub fn existing(&self, url: &str) -> Option<VideoPlayer> {
        self.players
            .borrow()
            .get(url)
            .map(|slot| slot.player.clone())
    }

    pub fn acquire(&self, url: &str) {
        if let Some(slot) = self.players.borrow_mut().get_mut(url) {
            slot.viewers += 1;
            slot.idle_since = None;
        }
    }

    pub fn release(&self, url: &str) {
        {
            let mut players = self.players.borrow_mut();
            let Some(slot) = players.get_mut(url) else {
                return;
            };
            slot.viewers = slot.viewers.saturating_sub(1);
            if slot.viewers > 0 {
                return;
            }
            slot.idle_since = Some(Instant::now());
        }
        let cache = self.clone();
        timer::set_timeout(IDLE_PLAYER_GRACE + Duration::from_millis(50), move || {
            cache.sweep_idle()
        });
    }

    fn sweep_idle(&self) {
        let now = Instant::now();
        let expired: Vec<VideoPlayer> = {
            let mut players = self.players.borrow_mut();
            let urls: Vec<String> = players
                .iter()
                .filter(|(_, s)| {
                    s.viewers == 0
                        && s.idle_since
                            .is_some_and(|t| now.duration_since(t) >= IDLE_PLAYER_GRACE)
                })
                .map(|(u, _)| u.clone())
                .collect();
            urls.iter()
                .filter_map(|u| players.remove(u))
                .map(|s| s.player)
                .collect()
        };
        drop(expired);
    }

    pub fn open_players(&self) -> usize {
        self.players.borrow().len()
    }

    pub fn get_or_open(&self, url: &str, loop_playback: bool) -> VideoPlayer {
        if let Some(slot) = self.players.borrow().get(url) {
            return slot.player.clone();
        }
        let player = VideoPlayer::open(
            self.runtime.clone(),
            self.http.clone(),
            url.to_string(),
            loop_playback,
            self.waker.clone(),
            self.preloads.borrow_mut().remove(url),
        );
        self.players.borrow_mut().insert(
            url.to_string(),
            Slot {
                player: player.clone(),
                viewers: 0,
                idle_since: None,
            },
        );
        player
    }
}

#[cfg(test)]
mod tests {
    use crate::rest;
    use super::*;

    fn boxed(kind: &[u8; 4], size: u32) -> Vec<u8> {
        let mut v = size.to_be_bytes().to_vec();
        v.extend_from_slice(kind);
        v.resize(size.min(64) as usize, 0);
        v
    }

    #[test]
    fn an_index_at_the_front_needs_no_second_request() {
        let mut file = boxed(b"ftyp", 24);
        file.extend(boxed(b"moov", 40));
        file.extend(boxed(b"mdat", 64));
        assert_eq!(locate_moov(&file, 10_000_000), Moov::Front);
    }

    #[test]
    fn an_index_after_a_big_mdat_is_asked_for_where_the_mdat_ends() {
        let mut file = boxed(b"ftyp", 32);
        file.extend(boxed(b"free", 8));
        let mdat_start = file.len() as u64;
        let mut mdat = (10_944_472u32).to_be_bytes().to_vec();
        mdat.extend_from_slice(b"mdat");
        file.extend(mdat);
        assert_eq!(
            locate_moov(&file, 11_005_300),
            Moov::At(mdat_start + 10_944_472)
        );
    }

    #[test]
    fn range_requests_start_on_a_64k_block() {
        assert_eq!(
            align_to_block(10_971_993),
            10_944_512,
            "the tail request of the measured file"
        );
        assert_eq!(align_to_block(65_536), 65_536);
        assert_eq!(align_to_block(65_535), 0);
    }

    #[test]
    fn too_few_bytes_and_non_mp4_files() {
        assert_eq!(locate_moov(&[0, 0, 0], 100), Moov::NeedMore);
        assert_eq!(
            locate_moov(b"\x1aE\xdf\xa3 webm header....", 100),
            Moov::Front
        );
    }

    #[tokio::test]
    #[ignore = "needs the local range-logging server"]
    async fn downloads_ask_for_the_index() {
        let client = rest::build_client().unwrap();
        let path = download_to_temp_file(&client, "http://127.0.0.1:8765/slow.mp4", None)
            .await
            .unwrap();
        println!("{} bytes", fs::metadata(&path).unwrap().len());
        let _ = fs::remove_file(path);
    }

    #[tokio::test]
    #[ignore = "needs the local range-logging server"]
    async fn a_hover_then_a_play_asks_for_the_rest_from_the_cut_block() {
        let client = rest::build_client().unwrap();
        let partial = fetch_head(
            &client,
            "http://127.0.0.1:8765/slow.mp4",
            Some(PRELOAD_BYTES),
        )
        .await
        .unwrap();
        assert!(!partial.complete && partial.head.len() >= PRELOAD_BYTES);
        let path = download_to_temp_file(&client, "http://127.0.0.1:8765/slow.mp4", Some(partial))
            .await
            .unwrap();
        assert_eq!(fs::metadata(&path).unwrap().len(), 11_005_300);
        let _ = fs::remove_file(path);
    }
}
