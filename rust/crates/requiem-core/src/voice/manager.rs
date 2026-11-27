use crate::utils::voice_settings;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::collections::btree_map::Entry;
use std::mem;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;
use tokio::time;

use tokio::sync::mpsc::UnboundedReceiver;

use super::audio::{AudioInput, AudioOutput};
use super::dave::{Dave, DaveAction};
use super::engine::CapturePipeline;
use super::gateway::{VoiceEvent, VoiceGateway, VoiceGatewayConfig};
use super::opcodes;
use super::opus::{FRAME_SAMPLES, MAX_PACKET, OpusDecoder, OpusEncoder, STEREO_FRAME_SAMPLES};
use super::status::{self, RtcState};
use super::udp::{self, OPUS_PAYLOAD_TYPE, UdpTransport};
use super::video;

const BROWSER_OPUS_PAYLOAD_TYPE: u8 = 111;

const SILENCE_FRAME: [u8; 3] = [0xF8, 0xFF, 0xFE];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VideoMode {
    None,

    Camera,

    Screen,
}

pub struct ConnectionConfig {
    pub client: wreq::Client,
    pub guild_id: u64,
    pub channel_id: u64,
    pub user_id: u64,
    pub session_id: String,
    pub token: String,
    pub endpoint: String,

    pub scope_guild: Option<u64>,

    pub video: VideoMode,
}

#[derive(Default)]
pub struct VoiceFlags {
    pub mute: AtomicBool,
    pub deaf: AtomicBool,

    pub camera: AtomicBool,
}

struct JitterBuffer {
    next: Option<u32>,

    pending: BTreeMap<u32, Vec<u8>>,

    pre_roll: usize,
}

impl JitterBuffer {
    fn new() -> Self {
        Self {
            next: None,
            pending: BTreeMap::new(),
            pre_roll: 5,
        }
    }

    fn push(&mut self, seq: u16, frame: Vec<u8>) -> (Vec<Vec<u8>>, usize) {
        let ext = self.extend(seq);
        self.pending.insert(ext, frame);
        if self.pending.len() > 100 {
            let first = *self.pending.keys().next().unwrap();
            self.pending.remove(&first);
        }

        if self.next.is_none() {
            if self.pending.len() < self.pre_roll {
                return (Vec::new(), 0);
            }
            self.next = self.pending.keys().next().copied();
        }

        let mut ready = Vec::new();
        let mut next = self.next.unwrap();
        while let Some(frame) = self.pending.remove(&next) {
            ready.push(frame);
            next = next.wrapping_add(1);
        }
        self.next = Some(next);

        let mut lost = 0;
        if !self.pending.contains_key(&next) {
            if let Some(&first) = self.pending.keys().next() {
                if first > next && self.pending.len() >= self.pre_roll {
                    lost = (first - next) as usize;
                    self.next = Some(first);
                }
            }
        }
        (ready, lost)
    }

    fn extend(&mut self, seq: u16) -> u32 {
        match self.next {
            Some(last_ext) => {
                let last_low = (last_ext & 0xFFFF) as u16;
                let delta = seq.wrapping_sub(last_low) as u32 & 0xFFFF;
                last_ext.wrapping_add(delta)
            }
            None => u32::from(seq),
        }
    }
}

const TRAILING_SILENCE_FRAMES: u8 = 5;

struct Session {
    transport: Arc<UdpTransport>,

    ssrc: u32,

    video_ssrc: Option<u32>,
    dave: Dave,

    backlog: Vec<VoiceEvent>,
}

struct RemoteVideo {
    user_id: u64,
    depacketizer: video::Vp8Depacketizer,
}

#[tracing::instrument(skip_all, fields(channel = config.channel_id))]
pub async fn run(
    config: ConnectionConfig,
    flags: Arc<VoiceFlags>,
    mut cancel: UnboundedReceiver<()>,
) {
    let is_call = config.video != VideoMode::Screen;
    if let Err(err) = run_inner(&config, flags, &mut cancel).await {
        log::warn!("voice: connection ended: {err}");
        if is_call {
            status::rtc_state(
                if err.contains("discovery") {
                    RtcState::NoRoute
                } else {
                    RtcState::RtcDisconnected
                },
                config.scope_guild,
                Some(config.channel_id),
            );
        }
    }
    if is_call {
        video::frames::clear_all();
    }
}

async fn run_inner(
    config: &ConnectionConfig,
    flags: Arc<VoiceFlags>,
    cancel: &mut UnboundedReceiver<()>,
) -> Result<(), String> {
    let is_call = config.video != VideoMode::Screen;
    let report = |state: RtcState| {
        if is_call {
            status::rtc_state(state, config.scope_guild, Some(config.channel_id));
        }
    };
    report(RtcState::Authenticating);
    let rtc_connection_id = uuid::Uuid::new_v4().simple().to_string();
    let (gw, mut events) = VoiceGateway::connect(
        &config.client,
        VoiceGatewayConfig {
            endpoint: config.endpoint.clone(),
            token: config.token.clone(),
            server_id: config.guild_id,
            channel_id: config.channel_id,
            user_id: config.user_id,
            session_id: config.session_id.clone(),
            rtc_connection_id,
            video: true,
        },
    )
    .map_err(|e| e.to_string())?;

    let session = tokio::select! {
        _ = cancel.recv() => {
            gw.close();
            return Ok(());
        }
        session = handshake(&gw, &mut events, config, &report) => session?,
    };
    let Session {
        transport,
        ssrc,
        video_ssrc,
        mut dave,
        backlog,
    } = session;

    if !backlog.is_empty() {
        let (tx, rx) = mpsc::unbounded_channel();
        for event in backlog {
            let _ = tx.send(event);
        }
        let mut rest = mem::replace(&mut events, rx);
        tokio::spawn(async move {
            while let Some(event) = rest.recv().await {
                if tx.send(event).is_err() {
                    break;
                }
            }
        });
    }
    let mut rtp_rx = transport.spawn_receiver();
    report(RtcState::RtcConnected);

    let _capturing = is_call.then(CaptureGuard::new);
    let mut devices = device_key();
    let mut audio_in = if is_call {
        AudioInput::start(devices.0.as_deref())
            .map_err(|e| log::warn!("voice: input: {e}"))
            .ok()
    } else {
        None
    };
    let mut audio_out = if is_call {
        AudioOutput::start(devices.1.as_deref())
            .map_err(|e| log::warn!("voice: output: {e}"))
            .ok()
    } else {
        None
    };
    let mut pipeline = CapturePipeline::new();
    let mut settings_version = 0;
    let mut others_spoke = Instant::now() - Duration::from_secs(10);

    let mut recording = super::diag::AudioRecording::new();
    let encoder = OpusEncoder::new().map_err(|e| format!("opus encoder: {e}"))?;
    let mut decoders: BTreeMap<u32, OpusDecoder> = BTreeMap::new();
    let mut jitter: BTreeMap<u32, JitterBuffer> = BTreeMap::new();
    let mut ssrc_to_user: BTreeMap<u32, u64> = BTreeMap::new();
    let mut videos: BTreeMap<u32, RemoteVideo> = BTreeMap::new();
    let decode = video::spawn_decoder();
    let mut recognized_users: Vec<u64> = Vec::new();
    let mut seen_ssrcs: HashSet<u32> = HashSet::new();
    let mut control = time::interval(Duration::from_millis(250));
    control.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let mut seq: u16 = rand_u16();
    let mut timestamp: u32 = rand_u32();
    let mut video_seq: u16 = rand_u16();
    let video_ts_base: u32 = rand_u32();
    let mut speaking = false;
    let mut trailing_silence: u8 = 0;
    let mut sent_frames: u64 = 0;
    let mut send_failures: u64 = 0;

    let mut camera_on = false;
    let mut camera_rx: Option<UnboundedReceiver<(Vec<u8>, u32)>> = None;
    if config.video == VideoMode::Screen {
        camera_rx = video::spawn_screen();
        camera_on = camera_rx.is_some();
        announce_video(&gw, ssrc, video_ssrc, camera_on);
    }

    loop {
        tokio::select! {
            _ = cancel.recv() => break,
            event = events.recv() => {
                match event {
                    None | Some(VoiceEvent::Closed { .. }) => {
                        report(RtcState::RtcDisconnected);
                        break;
                    }
                    Some(VoiceEvent::Speaking { user_id, ssrc: s, speaking: flags }) => {
                        ssrc_to_user.insert(s, user_id);
                        status::speaking(user_id, flags);
                    }
                    Some(VoiceEvent::ClientsConnect(ids)) => {
                        log::info!("voice: clients connect: {ids:?}");
                        for id in ids {
                            if !recognized_users.contains(&id) {
                                recognized_users.push(id);
                            }
                        }
                    }
                    Some(VoiceEvent::ClientDisconnect(user_id)) => {
                        log::debug!("voice: client disconnect user={user_id}");
                        ssrc_to_user.retain(|_, u| *u != user_id);
                        videos.retain(|_, v| v.user_id != user_id);
                        recognized_users.retain(|u| *u != user_id);
                        status::speaking(user_id, 0);
                        status::video(user_id, false);
                        video::frames::clear(user_id);
                    }
                    Some(VoiceEvent::Video { user_id, audio_ssrc, video_ssrc: v_ssrc, .. }) => {
                        log::debug!("voice: VIDEO user={user_id} audio_ssrc={audio_ssrc} video_ssrc={v_ssrc}");
                        if audio_ssrc != 0 {
                            ssrc_to_user.insert(audio_ssrc, user_id);
                        }
                        videos.retain(|_, v| v.user_id != user_id);
                        if v_ssrc != 0 {
                            videos.insert(v_ssrc, RemoteVideo { user_id, depacketizer: Default::default() });
                        } else {
                            video::frames::clear(user_id);
                        }
                        status::video(user_id, v_ssrc != 0);
                    }
                    Some(VoiceEvent::HeartbeatAck { rtt_ms: Some(rtt) }) => {
                        if is_call {
                            status::ping(rtt);
                        }
                    }
                    Some(VoiceEvent::DaveBinary { opcode, payload, .. }) => {
                        handle_dave_binary(&gw, &mut dave, config, opcode, &payload, &recognized_users);
                    }
                    Some(VoiceEvent::DavePrepareTransition { transition_id, protocol_version }) => {
                        if let DaveAction::TransitionReady(id) = dave.on_prepare_transition(transition_id, protocol_version) {
                            gw.dave_transition_ready(id);
                        }
                    }
                    Some(VoiceEvent::DaveExecuteTransition { transition_id }) => {
                        dave.on_execute_transition(transition_id);
                    }
                    Some(VoiceEvent::DavePrepareEpoch { epoch, protocol_version }) => {
                        if let Some(kp) = dave.on_prepare_epoch(epoch, protocol_version, config.user_id, config.channel_id) {
                            gw.dave_binary(opcodes::op::DAVE_MLS_KEY_PACKAGE, kp);
                        }
                    }
                    Some(_) => {}
                }
            }
            frame = recv_mic_frame(audio_in.as_mut()), if audio_in.is_some() => {
                let Some(mut frame) = frame else { continue };

                let version = voice_settings::version();
                if version != settings_version {
                    settings_version = version;
                    let wanted = device_key();
                    if wanted != devices {
                        devices = wanted;

                        drop(audio_in.take());
                        drop(audio_out.take());
                        audio_in = AudioInput::start(devices.0.as_deref()).map_err(|e| log::warn!("voice: input: {e}")).ok();
                        audio_out = AudioOutput::start(devices.1.as_deref()).map_err(|e| log::warn!("voice: output: {e}")).ok();
                        continue;
                    }
                }
                let loopback = super::engine::loopback();
                if let Some(out) = &audio_out {
                    let deaf = flags.deaf.load(Ordering::Relaxed);
                    out.set_gain(if deaf && !loopback { 0.0 } else { pipeline.settings().output_volume / 100.0 });
                }

                let muted = flags.mute.load(Ordering::Relaxed) || flags.deaf.load(Ordering::Relaxed);
                let processed = pipeline.process(&mut frame);
                let loud = !muted && processed.speaking;
                if pipeline.settings().aec_dump {
                    recording.push(&frame);
                }

                if loopback {
                    if let Some(out) = &audio_out {
                        let stereo: Vec<f32> = frame.iter().flat_map(|s| if processed.speaking { [*s, *s] } else { [0.0, 0.0] }).collect();
                        out.push(&stereo);
                    }
                }
                super::attenuation::set_talking(loud, others_spoke.elapsed() < Duration::from_millis(300));
                if loud != speaking {
                    speaking = loud;
                    let flag = if loud { opcodes::speaking::VOICE } else { opcodes::speaking::NONE };
                    gw.speaking(flag, ssrc, 0);
                    status::speaking(config.user_id, flag);
                    if !loud {
                        trailing_silence = TRAILING_SILENCE_FRAMES;
                    }
                }

                let payload: Vec<u8> = if loud {
                    let mut packet = [0u8; MAX_PACKET];
                    match encoder.encode(&frame, &mut packet) {
                        Ok(n) => packet[..n].to_vec(),
                        Err(_) => SILENCE_FRAME.to_vec(),
                    }
                } else if trailing_silence > 0 {
                    trailing_silence -= 1;
                    SILENCE_FRAME.to_vec()
                } else {

                    timestamp = timestamp.wrapping_add(FRAME_SAMPLES as u32);
                    continue;
                };
                let Some(e2ee) = dave.encrypt_opus(&payload) else {
                    continue;
                };
                match transport.send_rtp(OPUS_PAYLOAD_TYPE, false, seq, timestamp, ssrc, &e2ee).await {
                    Err(e) => {
                        if send_failures == 0 {
                            log::warn!("voice: audio send failed: {e}");
                        }
                        send_failures += 1;
                    }
                    Ok(()) if sent_frames == 0 => log::info!("voice: first audio send ok ({}B)", e2ee.len()),
                    Ok(()) => {}
                }
                sent_frames += 1;
                seq = seq.wrapping_add(1);
                timestamp = timestamp.wrapping_add(FRAME_SAMPLES as u32);
            }
            packet = rtp_rx.recv() => {
                let Some(packet) = packet else {
                    report(RtcState::RtcDisconnected);
                    break;
                };
                if seen_ssrcs.insert(packet.ssrc) {
                    log::info!("voice: first RTP from ssrc {} (pt {}, user={:?})", packet.ssrc, packet.payload_type, ssrc_to_user.get(&packet.ssrc));
                }
                if let Some(remote) = videos.get_mut(&packet.ssrc) {
                    let user_id = remote.user_id;
                    let payload = decrypt_dave(&mut dave, Some(user_id), &packet.payload).unwrap_or(packet.payload);
                    if let Some(frame) = remote.depacketizer.push(packet.sequence, packet.marker, &payload) {
                        let _ = decode.send((user_id, frame));
                    }
                    continue;
                }
                if !is_call || packet.payload_type != OPUS_PAYLOAD_TYPE && packet.payload_type != BROWSER_OPUS_PAYLOAD_TYPE {

                    continue;
                }
                let user_id = ssrc_to_user.get(&packet.ssrc).copied();
                let frame = decrypt_dave(&mut dave, user_id, &packet.payload).unwrap_or(packet.payload);

                if frame != SILENCE_FRAME && !super::engine::loopback() {
                    others_spoke = Instant::now();
                    let muted_locally = user_id.is_some_and(super::local_mute::is_muted);
                    if !muted_locally {
                        play_frame(&mut decoders, &mut jitter, audio_out.as_ref(), &mut pipeline, packet.ssrc, packet.sequence, frame, user_id.map(super::local_mute::volume).unwrap_or(1.0));
                    }
                }
            }
            _ = control.tick() => {

                if is_call {
                    let wanted = flags.camera.load(Ordering::Relaxed) && video_ssrc.is_some();
                    if wanted != camera_on {
                        camera_on = wanted;
                        camera_rx = if wanted { video::spawn_camera_with_self_view(config.user_id) } else { None };
                        if !wanted {
                            video::frames::clear(config.user_id);
                        }
                        announce_video(&gw, ssrc, video_ssrc, camera_on && camera_rx.is_some());
                    }
                }
            }
            camera = recv_camera(camera_rx.as_mut()), if camera_rx.is_some() => {
                let Some(video_ssrc) = video_ssrc else { continue };
                match camera {
                    Some((vp8, rtp_ts)) => {
                        let encrypted = dave.encrypt_video(&vp8).unwrap_or(vp8);
                        let packets = video::packetize_vp8(&encrypted);
                        let last = packets.len().saturating_sub(1);
                        for (i, payload) in packets.iter().enumerate() {
                            let _ = transport.send_rtp(video::VP8_PAYLOAD_TYPE, i == last, video_seq, video_ts_base.wrapping_add(rtp_ts), video_ssrc, payload).await;
                            video_seq = video_seq.wrapping_add(1);
                        }
                    }
                    None => {

                        camera_rx = None;
                        announce_video(&gw, ssrc, Some(video_ssrc), false);
                        if is_call {
                            video::frames::clear(config.user_id);
                        }
                    }
                }
            }
        }
    }

    if speaking {
        status::speaking(config.user_id, 0);
    }
    recording.save();
    gw.close();
    Ok(())
}

fn announce_video(gw: &VoiceGateway, audio_ssrc: u32, video_ssrc: Option<u32>, active: bool) {
    let Some(video_ssrc) = video_ssrc else { return };
    let stream = serde_json::json!({
        "type": "video",
        "rid": "100",
        "ssrc": video_ssrc,
        "rtx_ssrc": video_ssrc + 1,
        "active": active,
        "quality": 100,
        "max_bitrate": 2_500_000,
        "max_framerate": video::FRAME_RATE,
        "max_resolution": { "type": "source", "width": 0, "height": 0 },
    });
    let (v, rtx) = if active {
        (video_ssrc, video_ssrc + 1)
    } else {
        (0, 0)
    };
    gw.send_json(opcodes::op::VIDEO, serde_json::json!({ "audio_ssrc": audio_ssrc, "video_ssrc": v, "rtx_ssrc": rtx, "streams": [stream] }));
}

async fn handshake(
    gw: &VoiceGateway,
    events: &mut UnboundedReceiver<VoiceEvent>,
    config: &ConnectionConfig,
    report: &impl Fn(RtcState),
) -> Result<Session, String> {
    let mut backlog = Vec::new();
    let (ssrc, ip, port, modes, video_ssrc) = loop {
        match events.recv().await {
            Some(VoiceEvent::Ready {
                ssrc,
                ip,
                port,
                modes,
                video_ssrc,
            }) => {
                log::info!(
                    "voice: READY ssrc={ssrc} ip={ip} port={port} modes={modes:?} video_ssrc={video_ssrc:?}"
                );
                break (ssrc, ip, port, modes, video_ssrc);
            }
            Some(VoiceEvent::Closed { code }) => {
                log::warn!("voice: gateway closed during handshake ({code:?})");
                return Err(format!("voice gateway closed during handshake ({code:?})"));
            }
            None => {
                log::warn!("voice: gateway closed during handshake before READY");
                return Err("voice gateway closed during handshake".into());
            }
            Some(other) => {
                log::debug!("voice: handshake queued {other:?} before READY");
                backlog.push(other);
            }
        }
    };
    report(RtcState::RtcConnecting);
    let mode = udp::choose_mode(&modes)
        .ok_or_else(|| format!("no supported encryption mode in {modes:?}"))?;
    let transport = UdpTransport::connect(&ip, port, ssrc).await?;
    gw.select_protocol_udp(&transport.external.0, transport.external.1, mode);

    loop {
        match events.recv().await {
            Some(VoiceEvent::SessionDescription {
                mode: Some(mode),
                secret_key,
                dave_protocol_version,
                ..
            }) => {
                log::info!(
                    "voice: SESSION_DESCRIPTION mode={mode:?} dave_protocol_version={dave_protocol_version}"
                );
                transport.set_key(mode, &secret_key)?;
                let mut dave = Dave::new();
                if let Some(key_package) = dave.on_session_description(
                    dave_protocol_version,
                    config.user_id,
                    config.channel_id,
                ) {
                    gw.dave_binary(opcodes::op::DAVE_MLS_KEY_PACKAGE, key_package);
                }
                return Ok(Session {
                    transport: Arc::new(transport),
                    ssrc,
                    video_ssrc,
                    dave,
                    backlog,
                });
            }
            Some(VoiceEvent::SessionDescription { mode: None, .. }) => {
                log::warn!("voice: SESSION_DESCRIPTION arrived without an encryption mode");
                return Err("SESSION_DESCRIPTION without a mode".into());
            }
            Some(VoiceEvent::Closed { code }) => {
                log::warn!("voice: gateway closed during handshake after READY ({code:?})");
                return Err(format!("voice gateway closed during handshake ({code:?})"));
            }
            None => {
                log::warn!("voice: gateway closed during handshake before SESSION_DESCRIPTION");
                return Err("voice gateway closed during handshake".into());
            }
            Some(other) => {
                log::debug!("voice: handshake queued {other:?} before SESSION_DESCRIPTION");
                backlog.push(other);
            }
        }
    }
}

fn rand_u16() -> u16 {
    uuid::Uuid::new_v4().as_bytes()[0] as u16 | (uuid::Uuid::new_v4().as_bytes()[1] as u16) << 8
}

fn rand_u32() -> u32 {
    let bytes = *uuid::Uuid::new_v4().as_bytes();
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn handle_dave_binary(
    gw: &VoiceGateway,
    dave: &mut Dave,
    config: &ConnectionConfig,
    opcode: u8,
    payload: &[u8],
    recognized: &[u64],
) {
    match opcode {
        opcodes::recv::DAVE_MLS_EXTERNAL_SENDER => dave.set_external_sender(payload),
        opcodes::recv::DAVE_MLS_PROPOSALS => {
            if let Some(DaveAction::CommitWelcome(bytes)) = dave.on_proposals(payload, recognized) {
                gw.dave_binary(opcodes::op::DAVE_MLS_COMMIT_WELCOME, bytes);
            }
        }
        opcodes::recv::DAVE_MLS_ANNOUNCE_COMMIT_TRANSITION => {
            if payload.len() < 2 {
                return;
            }
            let transition_id = u16::from_be_bytes([payload[0], payload[1]]);
            match dave.on_commit(&payload[2..], transition_id) {
                Ok(Some(DaveAction::TransitionReady(id))) => gw.dave_transition_ready(id),
                Ok(_) => {}
                Err(()) => recover_dave(gw, dave, config, transition_id),
            }
        }
        opcodes::recv::DAVE_MLS_WELCOME => {
            if payload.len() < 2 {
                return;
            }
            let transition_id = u16::from_be_bytes([payload[0], payload[1]]);
            match dave.on_welcome(&payload[2..], transition_id) {
                Ok(Some(DaveAction::TransitionReady(id))) => gw.dave_transition_ready(id),
                Ok(_) => {}
                Err(()) => recover_dave(gw, dave, config, transition_id),
            }
        }
        other => log::debug!("voice: unhandled DAVE binary opcode {other}"),
    }
}

fn recover_dave(gw: &VoiceGateway, dave: &mut Dave, config: &ConnectionConfig, transition_id: u16) {
    gw.dave_invalid_commit_welcome(transition_id);
    let _ = dave.recover(config.user_id, config.channel_id);
}

fn decrypt_dave(dave: &mut Dave, user_id: Option<u64>, frame: &[u8]) -> Option<Vec<u8>> {
    let user_id = user_id?;
    if dave.protocol_version() != 0 {
        return dave.decrypt(user_id, frame);
    }
    None
}

async fn recv_mic_frame(audio_in: Option<&mut AudioInput>) -> Option<Vec<f32>> {
    audio_in?.frames.recv().await
}

async fn recv_camera(rx: Option<&mut UnboundedReceiver<(Vec<u8>, u32)>>) -> Option<(Vec<u8>, u32)> {
    rx?.recv().await
}

fn play_frame(
    decoders: &mut BTreeMap<u32, OpusDecoder>,
    jitter: &mut BTreeMap<u32, JitterBuffer>,
    audio_out: Option<&AudioOutput>,
    pipeline: &mut CapturePipeline,
    ssrc: u32,
    seq: u16,
    frame: Vec<u8>,
    volume: f32,
) {
    let buffer = jitter.entry(ssrc).or_insert_with(JitterBuffer::new);
    let (ready, lost) = buffer.push(seq, frame);

    if lost > 0 && decoders.contains_key(&ssrc) {
        let mut pcm = vec![0.0f32; STEREO_FRAME_SAMPLES];
        for _ in 0..lost {
            if let Ok(n) = decoders.get_mut(&ssrc).unwrap().decode_lost(&mut pcm) {
                let len = n * 2;
                if let Some(out) = audio_out {
                    out.mix(ssrc, &pcm[..len], volume);
                }
                pipeline.render(&pcm[..len]);
            }
        }
    }
    for opus in ready {
        if let Entry::Vacant(e) = decoders.entry(ssrc) {
            match OpusDecoder::new() {
                Ok(decoder) => {
                    e.insert(decoder);
                }
                Err(err) => {
                    log::warn!("voice: opus decoder for ssrc {ssrc}: {err}");
                    continue;
                }
            }
        }
        let mut pcm = vec![0.0f32; STEREO_FRAME_SAMPLES];
        if let Ok(n) = decoders.get_mut(&ssrc).unwrap().decode(&opus, &mut pcm) {
            let len = n * 2;
            if let Some(out) = audio_out {
                out.mix(ssrc, &pcm[..len], volume);
            }
            pipeline.render(&pcm[..len]);
        }
    }
}

fn device_key() -> (
    Option<String>,
    Option<String>,
    voice_settings::AudioSubsystem,
) {
    let settings = voice_settings::get();
    (
        settings.input_device,
        settings.output_device,
        settings.audio_subsystem,
    )
}

struct CaptureGuard;

impl CaptureGuard {
    fn new() -> Self {
        super::engine::set_connection_capturing(true);
        super::ptt::set_in_call(true);
        CaptureGuard
    }
}

impl Drop for CaptureGuard {
    fn drop(&mut self) {
        super::engine::set_connection_capturing(false);
        super::ptt::set_in_call(false);
        super::attenuation::set_talking(false, false);
    }
}

#[cfg(test)]
mod tests {
    use super::JitterBuffer;

    #[test]
    fn pre_roll_then_play_in_order() {
        let mut jb = JitterBuffer::new();
        let mut played = 0;
        for seq in 0..5 {
            played += jb.push(seq, vec![seq as u8]).0.len();
        }
        assert_eq!(played, 5);
    }

    #[test]
    fn a_lost_packet_is_skipped_instead_of_stalling() {
        let mut jb = JitterBuffer::new();
        let mut played = 0;
        let mut lost = 0;
        for seq in 0..5 {
            played += jb.push(seq, vec![seq as u8]).0.len();
        }
        assert_eq!(played, 5);

        for seq in 6..=11 {
            let (ready, l) = jb.push(seq, vec![seq as u8]);
            played += ready.len();
            lost += l;
        }
        assert_eq!(played, 11, "frames 6-11 play after skipping the lost 5");
        assert_eq!(lost, 1, "exactly one frame (seq 5) is reported lost");
    }
}
