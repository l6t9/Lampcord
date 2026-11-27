use std::time::Duration;
use crate::utils::voice_settings;
use ffmpeg_next::decoder;
use ffmpeg_next::ffi::AVMediaType;
use ffmpeg_next::ffi;
use ffmpeg_next::format::context::Input;
use ffmpeg_next::media::Type;
use std::ffi::CStr;
use std::slice;
use std::thread::Builder;
use xcap::Monitor;

use std::ffi::CString;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use ffmpeg_next::Dictionary;
use ffmpeg_next::codec::context::Context as CodecContext;
use ffmpeg_next::format::Pixel;
use ffmpeg_next::frame::Video as VideoFrame;
use ffmpeg_next::software::scaling::Context as ScaleCtx;
use ffmpeg_next::software::scaling::flag::Flags;

pub struct CameraFrame {
    pub width: u32,
    pub height: u32,

    pub rgb: Vec<u8>,
}

pub trait FrameSource {
    fn next_frame(&mut self) -> Result<CameraFrame, String>;
}

pub fn camera_names() -> Vec<String> {
    if ffmpeg_next::init().is_err() {
        return Vec::new();
    }
    let mut names = Vec::new();

    // SAFETY: `fmt_name` is a valid NUL-terminated string. The device list from avdevice is freed
    // once at the end, every device pointer is null-checked, and the media-type and description
    // pointers it holds stay valid while the list is alive.
    unsafe {
        let fmt_name = CString::new("dshow").expect("static name");
        let fmt = ffi::av_find_input_format(fmt_name.as_ptr());
        if fmt.is_null() {
            return names;
        }
        let mut list: *mut ffi::AVDeviceInfoList = ptr::null_mut();
        if ffi::avdevice_list_input_sources(
            fmt,
            ptr::null(),
            ptr::null_mut(),
            &mut list,
        ) < 0
            || list.is_null()
        {
            return names;
        }
        for i in 0..(*list).nb_devices as usize {
            let device = *(*list).devices.add(i);
            if device.is_null() {
                continue;
            }

            let types = slice::from_raw_parts(
                (*device).media_types,
                (*device).nb_media_types as usize,
            );
            if !types.contains(&AVMediaType::AVMEDIA_TYPE_VIDEO) {
                continue;
            }
            let name = CStr::from_ptr((*device).device_description)
                .to_string_lossy()
                .into_owned();
            if !names.contains(&name) {
                names.push(name);
            }
        }
        ffi::avdevice_free_list_devices(&mut list);
    }
    names
}

fn open_input_device(
    device_name: &str,
    url: &str,
    options: Dictionary,
) -> Result<Input, String> {
    // SAFETY: `fmt_name` and `url_c` are valid NUL-terminated strings that outlive the calls.
    // Ownership of `opts` goes to FFmpeg and is taken back with `Dictionary::own`; `ps` is closed
    // on the failure path or handed to `Input::wrap`, which owns it.
    unsafe {
        let fmt_name = CString::new(device_name).map_err(|e| e.to_string())?;
        let fmt = ffi::av_find_input_format(fmt_name.as_ptr());
        if fmt.is_null() {
            return Err(format!("input format '{device_name}' not found"));
        }
        let url_c = CString::new(url).map_err(|e| e.to_string())?;
        let mut ps = ptr::null_mut();
        let mut opts = options.disown();
        let res = ffi::avformat_open_input(&mut ps, url_c.as_ptr(), fmt, &mut opts);
        Dictionary::own(opts);
        if res < 0 {
            return Err(ffmpeg_next::Error::from(res).to_string());
        }
        let r = ffi::avformat_find_stream_info(ps, ptr::null_mut());
        if r < 0 {
            ffi::avformat_close_input(&mut ps);
            return Err(ffmpeg_next::Error::from(r).to_string());
        }
        Ok(Input::wrap(ps))
    }
}

#[derive(Default)]
struct LatestPacket {
    slot: Mutex<Option<ffmpeg_next::Packet>>,
    ready: Condvar,
    ended: AtomicBool,
    stop: AtomicBool,
}

pub struct CameraSource {
    latest: Arc<LatestPacket>,
    decoder: decoder::Video,
    scaler: ScaleCtx,
}

impl Drop for CameraSource {
    fn drop(&mut self) {
        self.latest.stop.store(true, Ordering::Release);
    }
}

impl CameraSource {
    pub fn open() -> Result<Self, String> {
        Self::open_named(voice_settings::get().video_device)
    }

    pub fn open_named(name: Option<String>) -> Result<Self, String> {
        ffmpeg_next::init().map_err(|e| e.to_string())?;
        let listed = camera_names();
        let name = name
            .filter(|name| listed.contains(name))
            .or_else(|| listed.into_iter().next())
            .ok_or("no camera found")?;
        let device = format!("video={name}");

        let mut opts = Dictionary::new();
        opts.set("rtbufsize", "64M");
        let ictx = open_input_device("dshow", &device, opts)?;

        let stream = ictx
            .streams()
            .best(Type::Video)
            .ok_or("no video stream on camera")?;
        let stream_index = stream.index();
        let decoder = CodecContext::from_parameters(stream.parameters())
            .map_err(|e| e.to_string())?
            .decoder()
            .video()
            .map_err(|e| e.to_string())?;
        let scaler = ScaleCtx::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGB24,
            decoder.width(),
            decoder.height(),
            Flags::BILINEAR,
        )
        .map_err(|e| e.to_string())?;

        let latest = Arc::new(LatestPacket::default());
        let reader = latest.clone();
        Builder::new()
            .name("camera-reader".into())
            .spawn(move || {
                let mut ictx = ictx;
                for (stream, packet) in ictx.packets() {
                    if reader.stop.load(Ordering::Acquire) {
                        break;
                    }
                    if stream.index() != stream_index {
                        continue;
                    }
                    *reader.slot.lock().expect("camera packet poisoned") = Some(packet);
                    reader.ready.notify_one();
                }
                reader.ended.store(true, Ordering::Release);
                reader.ready.notify_all();
            })
            .map_err(|e| e.to_string())?;

        log::info!("voice: camera opened ({device})");
        Ok(Self {
            latest,
            decoder,
            scaler,
        })
    }

    fn next_packet(&self) -> Option<ffmpeg_next::Packet> {
        let mut slot = self.latest.slot.lock().expect("camera packet poisoned");
        loop {
            if let Some(packet) = slot.take() {
                return Some(packet);
            }
            if self.latest.ended.load(Ordering::Acquire) {
                return None;
            }
            slot = self
                .latest
                .ready
                .wait_timeout(slot, Duration::from_millis(200))
                .expect("camera packet poisoned")
                .0;
        }
    }
}

impl FrameSource for CameraSource {
    fn next_frame(&mut self) -> Result<CameraFrame, String> {
        loop {
            let Some(packet) = self.next_packet() else {
                return Err("camera stream ended".into());
            };
            if self.decoder.send_packet(&packet).is_err() {
                continue;
            }
            let mut decoded = VideoFrame::empty();
            if self.decoder.receive_frame(&mut decoded).is_err() {
                continue;
            }
            let mut rgb = VideoFrame::empty();
            self.scaler
                .run(&decoded, &mut rgb)
                .map_err(|e| e.to_string())?;
            let (width, height) = (rgb.width(), rgb.height());
            let stride = rgb.stride(0);
            let data = rgb.data(0);
            let mut out = Vec::with_capacity((width * height * 3) as usize);
            for row in 0..height as usize {
                let start = row * stride;
                out.extend_from_slice(&data[start..start + (width * 3) as usize]);
            }
            return Ok(CameraFrame {
                width,
                height,
                rgb: out,
            });
        }
    }
}

pub struct ScreenSource {
    monitor: Monitor,
}

impl ScreenSource {
    pub fn open() -> Result<Self, String> {
        let monitors = Monitor::all().map_err(|e| e.to_string())?;
        let monitor = monitors.into_iter().next().ok_or("no display found")?;
        log::info!(
            "voice: screen opened ({}x{})",
            monitor.width().unwrap_or(0),
            monitor.height().unwrap_or(0)
        );
        Ok(Self { monitor })
    }
}

impl FrameSource for ScreenSource {
    fn next_frame(&mut self) -> Result<CameraFrame, String> {
        let img = self.monitor.capture_image().map_err(|e| e.to_string())?;
        let (width, height) = (img.width(), img.height());

        let rgba = img.into_raw();
        let mut rgb = Vec::with_capacity(rgba.len() / 4 * 3);
        for px in rgba.chunks_exact(4) {
            rgb.push(px[0]);
            rgb.push(px[1]);
            rgb.push(px[2]);
        }
        Ok(CameraFrame { width, height, rgb })
    }
}

pub mod preview {
    use crate::utils::voice_settings;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread::Builder;
    use std::time::{Duration, Instant};

    use super::{CameraSource, FrameSource};

    pub struct PreviewFrame {
        pub width: u32,
        pub height: u32,
        pub rgba: Vec<u8>,
    }

    static RUNNING: AtomicBool = AtomicBool::new(false);
    static KEEPALIVE: Mutex<Option<Instant>> = Mutex::new(None);
    static LATEST: Mutex<Option<(u64, Arc<PreviewFrame>)>> = Mutex::new(None);
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

    pub fn keepalive() {
        *KEEPALIVE.lock().expect("preview keepalive poisoned") = Some(Instant::now());
        if RUNNING.swap(true, Ordering::AcqRel) {
            return;
        }
        let spawned = Builder::new()
            .name("camera-preview".into())
            .spawn(|| {
                run();
                *LATEST.lock().expect("preview frame poisoned") = None;
                RUNNING.store(false, Ordering::Release);
            });
        if spawned.is_err() {
            RUNNING.store(false, Ordering::Release);
        }
    }

    pub fn latest() -> Option<(u64, Arc<PreviewFrame>)> {
        LATEST.lock().expect("preview frame poisoned").clone()
    }

    fn alive() -> bool {
        KEEPALIVE
            .lock()
            .expect("preview keepalive poisoned")
            .is_some_and(|at| at.elapsed() < Duration::from_secs(1))
    }

    fn run() {
        let mut device = voice_settings::get().video_device;
        let mut source = match CameraSource::open_named(device.clone()) {
            Ok(source) => source,
            Err(err) => {
                log::warn!("camera preview: {err}");
                return;
            }
        };
        while alive() {
            let wanted = voice_settings::get().video_device;
            if wanted != device {
                device = wanted;
                match CameraSource::open_named(device.clone()) {
                    Ok(fresh) => source = fresh,
                    Err(err) => {
                        log::warn!("camera preview: {err}");
                        return;
                    }
                }
            }
            match source.next_frame() {
                Ok(frame) => {
                    let mut rgba = Vec::with_capacity(frame.rgb.len() / 3 * 4);
                    for px in frame.rgb.chunks_exact(3) {
                        rgba.extend_from_slice(&[px[0], px[1], px[2], 255]);
                    }
                    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                    *LATEST.lock().expect("preview frame poisoned") = Some((
                        id,
                        Arc::new(PreviewFrame {
                            width: frame.width,
                            height: frame.height,
                            rgba,
                        }),
                    ));
                }
                Err(err) => {
                    log::warn!("camera preview: {err}");
                    return;
                }
            }
        }
    }
}
