use std::collections::HashMap;
use crate::utils::voice_settings;
use ffmpeg_next::codec::Id;
use ffmpeg_next::error;
use std::collections::hash_map::Entry;
use std::mem;
use std::sync::mpsc;
use std::thread::Builder;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::mpsc as sync_mpsc;

use tokio::sync::mpsc::UnboundedReceiver;

use ffmpeg_next::Rational;
use ffmpeg_next::codec::context::Context;
use ffmpeg_next::codec::{decoder, encoder};
use ffmpeg_next::format::Pixel;
use ffmpeg_next::frame::Video as VideoFrame;
use ffmpeg_next::packet::Packet;
use ffmpeg_next::software::scaling::Context as ScaleCtx;
use ffmpeg_next::software::scaling::flag::Flags;

pub const VP8_PAYLOAD_TYPE: u8 = 103;

const VP8_MAX_PAYLOAD: usize = 1100;

pub fn packetize_vp8(frame: &[u8]) -> Vec<Vec<u8>> {
    frame
        .chunks(VP8_MAX_PAYLOAD)
        .enumerate()
        .map(|(i, chunk)| {
            let mut payload = Vec::with_capacity(1 + chunk.len());
            payload.push(if i == 0 { 0x10 } else { 0x00 });
            payload.extend_from_slice(chunk);
            payload
        })
        .collect()
}

fn strip_vp8_descriptor(payload: &[u8]) -> Option<(bool, &[u8])> {
    let first = *payload.first()?;
    let start = first & 0x10 != 0 && first & 0x07 == 0;
    let mut at = 1;
    if first & 0x80 != 0 {
        let ext = *payload.get(at)?;
        at += 1;
        if ext & 0x80 != 0 {
            at += if *payload.get(at)? & 0x80 != 0 { 2 } else { 1 };
        }
        if ext & 0x40 != 0 {
            at += 1;
        }
        if ext & 0x30 != 0 {
            at += 1;
        }
    }
    Some((start, payload.get(at..)?))
}

#[derive(Default)]
pub struct Vp8Depacketizer {
    frame: Vec<u8>,
    next_seq: Option<u16>,
    broken: bool,
}

impl Vp8Depacketizer {
    pub fn push(&mut self, seq: u16, marker: bool, payload: &[u8]) -> Option<Vec<u8>> {
        let (start, data) = strip_vp8_descriptor(payload)?;
        if start {
            self.frame.clear();
            self.broken = false;
        } else if self.next_seq != Some(seq) {
            self.broken = true;
        }
        self.next_seq = Some(seq.wrapping_add(1));
        self.frame.extend_from_slice(data);
        if !marker {
            return None;
        }
        let complete = !self.broken && !self.frame.is_empty();

        self.broken = true;
        complete.then(|| mem::take(&mut self.frame))
    }
}

pub struct Vp8Decoder {
    decoder: decoder::Video,
    scaler: Option<(ScaleCtx, u32, u32)>,
}

impl Vp8Decoder {
    pub fn new() -> Result<Self, String> {
        let codec = decoder::find(Id::VP8).ok_or("vp8 decoder not found")?;
        let decoder = Context::new_with_codec(codec)
            .decoder()
            .video()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            decoder,
            scaler: None,
        })
    }

    pub fn decode(
        &mut self,
        vp8: &[u8],
    ) -> Result<Option<super::capture::preview::PreviewFrame>, String> {
        self.decoder
            .send_packet(&Packet::copy(vp8))
            .map_err(|e| e.to_string())?;
        let mut yuv = VideoFrame::empty();
        if self.decoder.receive_frame(&mut yuv).is_err() {
            return Ok(None);
        }
        let (w, h) = (yuv.width(), yuv.height());
        if self
            .scaler
            .as_ref()
            .is_none_or(|(_, sw, sh)| (*sw, *sh) != (w, h))
        {
            let scaler = ScaleCtx::get(yuv.format(), w, h, Pixel::RGBA, w, h, Flags::BILINEAR)
                .map_err(|e| e.to_string())?;
            self.scaler = Some((scaler, w, h));
        }
        let mut rgba = VideoFrame::empty();
        self.scaler
            .as_mut()
            .expect("scaler set above")
            .0
            .run(&yuv, &mut rgba)
            .map_err(|e| e.to_string())?;
        let stride = rgba.stride(0);
        let row = w as usize * 4;
        let mut out = Vec::with_capacity(row * h as usize);
        for y in 0..h as usize {
            out.extend_from_slice(&rgba.data(0)[y * stride..y * stride + row]);
        }
        Ok(Some(super::capture::preview::PreviewFrame {
            width: w,
            height: h,
            rgba: out,
        }))
    }
}

pub fn spawn_decoder() -> mpsc::Sender<(u64, Vec<u8>)> {
    let (tx, rx) = mpsc::channel::<(u64, Vec<u8>)>();
    let spawned = Builder::new()
        .name("video-decode".into())
        .spawn(move || {
            let mut decoders: HashMap<u64, Vp8Decoder> =
                HashMap::new();
            while let Ok((user_id, vp8)) = rx.recv() {
                let decoder = match decoders.entry(user_id) {
                    Entry::Occupied(e) => e.into_mut(),
                    Entry::Vacant(e) => match Vp8Decoder::new() {
                        Ok(decoder) => e.insert(decoder),
                        Err(err) => {
                            log::warn!("voice: {err}");
                            continue;
                        }
                    },
                };
                match decoder.decode(&vp8) {
                    Ok(Some(rgba)) => frames::publish(user_id, rgba),
                    Ok(None) => {}
                    Err(err) => log::debug!("voice: vp8 decode: {err}"),
                }
            }
        });
    if let Err(err) = spawned {
        log::warn!("voice: video decode thread: {err}");
    }
    tx
}

pub mod frames {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    use parking_lot::Mutex;

    use super::super::capture::preview::PreviewFrame;

    static LATEST: Mutex<Option<HashMap<u64, (u64, Arc<PreviewFrame>)>>> = Mutex::new(None);
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

    pub fn publish(user_id: u64, frame: PreviewFrame) {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        LATEST
            .lock()
            .get_or_insert_with(HashMap::new)
            .insert(user_id, (id, Arc::new(frame)));
    }

    pub fn latest(user_id: u64) -> Option<(u64, Arc<PreviewFrame>)> {
        LATEST.lock().as_ref()?.get(&user_id).cloned()
    }

    pub fn clear(user_id: u64) {
        if let Some(all) = LATEST.lock().as_mut() {
            all.remove(&user_id);
        }
    }

    pub fn clear_all() {
        *LATEST.lock() = None;
    }
}

pub const FRAME_RATE: u32 = 30;

const RTP_TS_PER_FRAME: u32 = 90_000 / FRAME_RATE;

const FRAME_DURATION: Duration = Duration::from_millis(1000 / FRAME_RATE as u64);

pub struct Vp8Encoder {
    encoder: encoder::video::Encoder,
    scaler: ScaleCtx,
    rgb_frame: VideoFrame,
    yuv_frame: VideoFrame,
    width: u32,
    height: u32,

    row_bytes: usize,
}

impl Vp8Encoder {
    pub fn new(width: u32, height: u32) -> Result<Self, String> {
        let codec = encoder::find_by_name("libvpx").ok_or("libvpx encoder not found")?;
        let mut ctx = Context::new_with_codec(codec);
        ctx.set_time_base(Rational::new(1, FRAME_RATE as i32));

        let mut enc = ctx.encoder();
        enc.set_bit_rate(2_500_000);

        let mut video = enc.video().map_err(|e| e.to_string())?;
        video.set_width(width);
        video.set_height(height);
        video.set_format(Pixel::YUV420P);
        video.set_gop(FRAME_RATE * 4);
        video.set_max_b_frames(0);
        let encoder = video.open().map_err(|e| e.to_string())?;

        let scaler = ScaleCtx::get(
            Pixel::RGB24,
            width,
            height,
            Pixel::YUV420P,
            width,
            height,
            Flags::BILINEAR,
        )
        .map_err(|e| e.to_string())?;

        Ok(Self {
            encoder,
            scaler,
            rgb_frame: VideoFrame::new(Pixel::RGB24, width, height),
            yuv_frame: VideoFrame::new(Pixel::YUV420P, width, height),
            width,
            height,
            row_bytes: (width * 3) as usize,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn encode(&mut self, rgb: &[u8]) -> Result<Option<Vec<u8>>, String> {
        let expected = self.row_bytes * self.height as usize;
        if rgb.len() < expected {
            return Err(format!(
                "camera frame too small: {} < {expected}",
                rgb.len()
            ));
        }

        let stride = self.rgb_frame.stride(0);
        let data = self.rgb_frame.data_mut(0);
        for (row, src) in rgb[..expected].chunks_exact(self.row_bytes).enumerate() {
            let start = row * stride;
            data[start..start + self.row_bytes].copy_from_slice(src);
        }

        self.scaler
            .run(&self.rgb_frame, &mut self.yuv_frame)
            .map_err(|e| e.to_string())?;
        self.yuv_frame.set_pts(Some(0));

        self.encoder
            .send_frame(&self.yuv_frame)
            .map_err(|e| e.to_string())?;

        let mut packet = Packet::empty();
        match self.encoder.receive_packet(&mut packet) {
            Ok(()) => Ok(Some(packet.data().unwrap_or_default().to_vec())),
            Err(ffmpeg_next::Error::Other { errno }) if errno == error::EAGAIN => {
                Ok(None)
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

pub fn spawn_camera() -> Option<UnboundedReceiver<(Vec<u8>, u32)>> {
    spawn_source("camera-capture", super::capture::CameraSource::open, None)
}

pub fn spawn_camera_with_self_view(self_view: u64) -> Option<UnboundedReceiver<(Vec<u8>, u32)>> {
    let open = || {
        super::capture::CameraSource::open_named(voice_settings::get().video_device)
    };
    spawn_source("camera-capture", open, Some(self_view))
}

pub fn spawn_screen() -> Option<UnboundedReceiver<(Vec<u8>, u32)>> {
    spawn_source("screen-capture", super::capture::ScreenSource::open, None)
}

fn spawn_source<F: super::capture::FrameSource + 'static>(
    name: &'static str,
    open: impl FnOnce() -> Result<F, String> + Send + 'static,
    self_view: Option<u64>,
) -> Option<UnboundedReceiver<(Vec<u8>, u32)>> {
    let (tx, rx) = sync_mpsc::unbounded_channel();
    Builder::new()
        .name(name.into())
        .spawn(move || {
            let mut source = match open() {
                Ok(s) => s,
                Err(e) => {
                    log::warn!("voice: {name} unavailable: {e}");
                    return;
                }
            };

            let first = match source.next_frame() {
                Ok(f) => f,
                Err(e) => {
                    log::warn!("voice: {name}: {e}");
                    return;
                }
            };
            let mut encoder = match Vp8Encoder::new(first.width.max(2), first.height.max(2)) {
                Ok(e) => e,
                Err(e) => {
                    log::warn!("voice: {name} encoder: {e}");
                    return;
                }
            };

            let mut rtp_ts: u32 = 0;
            let mut pending = Some(first);
            loop {
                let start = Instant::now();
                let frame = match pending.take() {
                    Some(f) => f,
                    None => match source.next_frame() {
                        Ok(f) => f,
                        Err(e) => {
                            log::warn!("voice: {name}: {e}");
                            thread::sleep(FRAME_DURATION);
                            continue;
                        }
                    },
                };
                if let Some(user_id) = self_view {
                    let rgba = frame
                        .rgb
                        .chunks_exact(3)
                        .flat_map(|p| [p[0], p[1], p[2], 255])
                        .collect();
                    frames::publish(
                        user_id,
                        super::capture::preview::PreviewFrame {
                            width: frame.width,
                            height: frame.height,
                            rgba,
                        },
                    );
                }
                match encoder.encode(&frame.rgb) {
                    Ok(Some(vp8)) => {
                        if tx.send((vp8, rtp_ts)).is_err() {
                            return;
                        }
                    }
                    Ok(None) => {}
                    Err(e) => log::warn!("voice: {name} encode: {e}"),
                }
                rtp_ts = rtp_ts.wrapping_add(RTP_TS_PER_FRAME);
                let elapsed = start.elapsed();
                if elapsed < FRAME_DURATION {
                    thread::sleep(FRAME_DURATION - elapsed);
                }
            }
        })
        .ok()?;
    Some(rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_a_solid_frame() {
        let mut enc = Vp8Encoder::new(160, 120).expect("libvpx encoder");
        let rgb = vec![0x80u8; 160 * 120 * 3];
        let packet = enc.encode(&rgb).expect("encode");

        assert!(packet.is_some(), "first frame should produce a VP8 packet");
        assert!(!packet.unwrap().is_empty());
    }

    #[test]
    fn vp8_round_trips_through_rtp_packets() {
        let frame: Vec<u8> = (0..3000u32).map(|i| i as u8).collect();
        let packets = packetize_vp8(&frame);
        assert_eq!(packets.len(), 3);
        let mut depacketizer = Vp8Depacketizer::default();
        let mut out = None;
        for (i, payload) in packets.iter().enumerate() {
            out = depacketizer.push(10 + i as u16, i == packets.len() - 1, payload);
        }
        assert_eq!(out, Some(frame));

        let mut depacketizer = Vp8Depacketizer::default();
        assert_eq!(depacketizer.push(0, false, &packets[0]), None);
        assert_eq!(depacketizer.push(2, true, &packets[2]), None);
    }

    #[test]
    fn decodes_what_the_encoder_made() {
        let mut enc = Vp8Encoder::new(64, 48).expect("libvpx encoder");
        let vp8 = enc.encode(&vec![0x40u8; 64 * 48 * 3]).unwrap().unwrap();
        let mut dec = Vp8Decoder::new().expect("vp8 decoder");
        let frame = dec.decode(&vp8).unwrap().expect("a frame");
        assert_eq!((frame.width, frame.height), (64, 48));
        assert_eq!(frame.rgba.len(), 64 * 48 * 4);
    }
}
