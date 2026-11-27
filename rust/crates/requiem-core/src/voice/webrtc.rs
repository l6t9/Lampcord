use crate::utils::voice_settings;
use std::future;
use std::io;
use std::net;
use std::pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time;
use webrtc::runtime;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use rtc::interceptor::Registry;
use rtc::media::Sample;
use rtc::media_stream::MediaStreamTrack;
use rtc::peer_connection::configuration::RTCConfigurationBuilder;
use rtc::peer_connection::configuration::interceptor_registry::register_default_interceptors;
use rtc::peer_connection::configuration::media_engine::{
    MIME_TYPE_OPUS, MIME_TYPE_VP8, MediaEngine,
};
use rtc::peer_connection::sdp::RTCSessionDescription;
use rtc::rtp::{Header, Packet};
use rtc::rtp_transceiver::rtp_sender::{
    RTCRtpCodec, RTCRtpCodecParameters, RTCRtpCodingParameters, RTCRtpEncodingParameters,
    RtpCodecKind,
};
use webrtc::media_stream::track_local::TrackLocal;
use webrtc::media_stream::track_local::static_rtp::TrackLocalStaticRTP;
use webrtc::media_stream::track_local::static_sample::TrackLocalStaticSample;
use webrtc::media_stream::track_remote::{TrackRemote, TrackRemoteEvent};
use webrtc::peer_connection::{
    PeerConnection, PeerConnectionBuilder, PeerConnectionEventHandler, RTCIceGatheringState,
    RTCPeerConnectionState,
};
use webrtc::runtime::default_runtime;

use super::sdp::{build_fragment, repair_answer};

pub const OPUS_PAYLOAD_TYPE: u8 = 120;

pub const VP8_PAYLOAD_TYPE: u8 = 96;

const OPUS_CLOCK_RATE: u32 = 48_000;

const VP8_CLOCK_RATE: u32 = 90_000;

pub struct WebRtcTransport {
    pc: Arc<dyn PeerConnection>,
    audio_track: Arc<TrackLocalStaticRTP>,
    audio_ssrc: u32,
    video_track: Option<Arc<TrackLocalStaticSample>>,
    video_ssrc: Option<u32>,

    pub remote_tracks: UnboundedReceiver<Arc<dyn TrackRemote>>,

    pub connection_state: UnboundedReceiver<RTCPeerConnectionState>,

    pub gathering_state: UnboundedReceiver<RTCIceGatheringState>,
}

#[derive(Clone)]
struct Handler {
    track_tx: UnboundedSender<Arc<dyn TrackRemote>>,
    state_tx: UnboundedSender<RTCPeerConnectionState>,
    gather_tx: UnboundedSender<RTCIceGatheringState>,
}

#[async_trait::async_trait]
impl PeerConnectionEventHandler for Handler {
    async fn on_track(&self, track: Arc<dyn TrackRemote>) {
        log::info!(
            "voice: remote track kind={:?} ssrcs={:?}",
            track.kind().await,
            track.ssrcs().await
        );
        let _ = self.track_tx.send(track);
    }

    async fn on_connection_state_change(&self, state: RTCPeerConnectionState) {
        log::info!("voice: webrtc connection state -> {state:?}");
        let _ = self.state_tx.send(state);
    }

    async fn on_ice_gathering_state_change(&self, state: RTCIceGatheringState) {
        log::debug!("voice: ice gathering -> {state:?}");
        let _ = self.gather_tx.send(state);
    }
}

impl WebRtcTransport {
    pub async fn connect(video: bool) -> Result<(Self, String), String> {
        let audio_ssrc = random_ssrc();
        let video_ssrc = video.then(random_ssrc);

        let opus = RTCRtpCodecParameters {
            rtp_codec: RTCRtpCodec {
                mime_type: MIME_TYPE_OPUS.to_owned(),
                clock_rate: OPUS_CLOCK_RATE,
                channels: 2,
                sdp_fmtp_line: "minptime=10;useinbandfec=1".to_owned(),
                rtcp_feedback: vec![],
            },
            payload_type: OPUS_PAYLOAD_TYPE,
        };
        let vp8 = RTCRtpCodecParameters {
            rtp_codec: RTCRtpCodec {
                mime_type: MIME_TYPE_VP8.to_owned(),
                clock_rate: VP8_CLOCK_RATE,
                channels: 0,
                sdp_fmtp_line: String::new(),
                rtcp_feedback: vec![],
            },
            payload_type: VP8_PAYLOAD_TYPE,
        };

        let mut media_engine = MediaEngine::default();
        media_engine
            .register_codec(opus.clone(), RtpCodecKind::Audio)
            .map_err(|e| e.to_string())?;
        if video {
            media_engine
                .register_codec(vp8.clone(), RtpCodecKind::Video)
                .map_err(|e| e.to_string())?;
        }
        let registry = register_default_interceptors(Registry::new(), &mut media_engine)
            .map_err(|e| e.to_string())?;

        let config = RTCConfigurationBuilder::new()
            .with_ice_servers(vec![])
            .build();

        let (track_tx, remote_tracks) = unbounded_channel();
        let (state_tx, connection_state) = unbounded_channel();
        let (gather_tx, gathering_state) = unbounded_channel();
        let handler = Arc::new(Handler {
            track_tx,
            state_tx,
            gather_tx,
        });

        let runtime: Arc<dyn runtime::Runtime> = Arc::new(QosRuntime(
            default_runtime().ok_or("webrtc runtime not available".to_string())?,
        ));
        let pc: Arc<dyn PeerConnection> = Arc::new(
            PeerConnectionBuilder::new()
                .with_configuration(config)
                .with_media_engine(media_engine)
                .with_interceptor_registry(registry)
                .with_handler(handler)
                .with_runtime(runtime)
                .with_udp_addrs(vec!["0.0.0.0:0"])
                .build()
                .await
                .map_err(|e| e.to_string())?,
        );

        let audio_track = Arc::new(TrackLocalStaticRTP::new(MediaStreamTrack::new(
            "requiem-stream-audio".into(),
            "requiem-track-audio".into(),
            "audio".into(),
            RtpCodecKind::Audio,
            vec![RTCRtpEncodingParameters {
                rtp_coding_parameters: RTCRtpCodingParameters {
                    ssrc: Some(audio_ssrc),
                    ..Default::default()
                },
                codec: opus.rtp_codec.clone(),
                ..Default::default()
            }],
        )));
        pc.add_track(Arc::clone(&audio_track) as Arc<dyn TrackLocal>)
            .await
            .map_err(|e| e.to_string())?;

        let video_track = if let Some(ssrc) = video_ssrc {
            let track = Arc::new(
                TrackLocalStaticSample::new(
                    Instant::now(),
                    MediaStreamTrack::new(
                        "requiem-stream-video".into(),
                        "requiem-track-video".into(),
                        "video".into(),
                        RtpCodecKind::Video,
                        vec![RTCRtpEncodingParameters {
                            rtp_coding_parameters: RTCRtpCodingParameters {
                                ssrc: Some(ssrc),
                                ..Default::default()
                            },
                            codec: vp8.rtp_codec.clone(),
                            ..Default::default()
                        }],
                    ),
                )
                .map_err(|e| e.to_string())?,
            );
            pc.add_track(Arc::clone(&track) as Arc<dyn TrackLocal>)
                .await
                .map_err(|e| e.to_string())?;
            Some(track)
        } else {
            None
        };

        let offer = pc.create_offer(None).await.map_err(|e| e.to_string())?;
        pc.set_local_description(offer)
            .await
            .map_err(|e| e.to_string())?;

        let mut transport = Self {
            pc,
            audio_track,
            audio_ssrc,
            video_track,
            video_ssrc,
            remote_tracks,
            connection_state,
            gathering_state,
        };

        transport.wait_gathering_complete().await?;
        let fragment = transport.offer_fragment().await?;

        Ok((transport, fragment))
    }

    async fn wait_gathering_complete(&mut self) -> Result<(), String> {
        let timeout = Duration::from_secs(10);
        time::timeout(timeout, async {
            while let Some(state) = self.gathering_state.recv().await {
                if state == RTCIceGatheringState::Complete {
                    return;
                }
            }
        })
        .await
        .map_err(|_| "ICE gathering timed out".to_string())?;
        Ok(())
    }

    async fn offer_fragment(&self) -> Result<String, String> {
        let desc = self
            .pc
            .local_description()
            .await
            .ok_or("no local description")?;
        Ok(build_fragment(&desc.sdp))
    }

    pub async fn apply_answer(&self, answer_sdp: &str) -> Result<(), String> {
        let answer =
            RTCSessionDescription::answer(repair_answer(answer_sdp)).map_err(|e| e.to_string())?;
        self.pc
            .set_remote_description(answer)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn send(&self, seq: u16, timestamp: u32, payload: &[u8]) -> Result<(), String> {
        let packet = Packet {
            header: Header {
                version: 2,
                padding: false,
                extension: false,
                marker: false,
                payload_type: OPUS_PAYLOAD_TYPE,
                sequence_number: seq,
                timestamp,
                ssrc: self.audio_ssrc,
                csrc: vec![],
                extension_profile: 0,
                extensions: vec![],
                extensions_padding: 0,
            },
            payload: payload.to_vec().into(),
        };
        self.audio_track
            .write_rtp(packet)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn send_video_sample(
        &self,
        data: &[u8],
        packet_timestamp: u32,
        duration: Duration,
    ) -> Result<(), String> {
        let ssrc = self
            .video_ssrc
            .ok_or("video not enabled on this connection")?;
        let track = self
            .video_track
            .as_ref()
            .ok_or("video not enabled on this connection")?;
        let sample = Sample {
            data: data.to_vec().into(),
            timestamp: Instant::now(),
            duration,
            packet_timestamp,
            prev_dropped_packets: 0,
            prev_padding_packets: 0,
        };
        track
            .write_sample(ssrc, VP8_PAYLOAD_TYPE, &sample, &[])
            .await
            .map_err(|e| e.to_string())
    }

    pub fn audio_ssrc(&self) -> u32 {
        self.audio_ssrc
    }

    pub fn video_ssrc(&self) -> u32 {
        self.video_ssrc.unwrap_or(0)
    }

    pub async fn close(&self) {
        let _ = self.pc.close().await;
    }
}

pub async fn recv_rtp(track: &Arc<dyn TrackRemote>) -> Option<Packet> {
    loop {
        match track.poll().await? {
            TrackRemoteEvent::OnRtpPacket(packet) => return Some(packet),
            TrackRemoteEvent::OnEnded | TrackRemoteEvent::OnEnding => return None,
            _ => continue,
        }
    }
}

fn random_ssrc() -> u32 {
    (uuid::Uuid::new_v4().as_u128() & 0xFFFF_FFFF) as u32
}

#[derive(Debug)]
struct QosRuntime(Arc<dyn runtime::Runtime>);

const DSCP_EF_TOS: u32 = 46 << 2;

impl runtime::Runtime for QosRuntime {
    fn spawn(
        &self,
        future: pin::Pin<Box<dyn future::Future<Output = ()> + Send>>,
    ) -> Box<dyn runtime::JoinHandle> {
        self.0.spawn(future)
    }

    fn spawn_reactor(
        &self,
        pool: usize,
        future: pin::Pin<Box<dyn future::Future<Output = ()> + Send>>,
    ) -> Box<dyn runtime::JoinHandle> {
        self.0.spawn_reactor(pool, future)
    }

    fn wrap_udp_socket(
        &self,
        socket: net::UdpSocket,
    ) -> io::Result<Arc<dyn runtime::AsyncUdpSocket>> {
        if voice_settings::get().qos {
            if let Err(err) = socket2::SockRef::from(&socket).set_tos_v4(DSCP_EF_TOS) {
                log::debug!("voice: could not mark the socket for QoS: {err}");
            }
        }
        self.0.wrap_udp_socket(socket)
    }

    fn wrap_tcp_listener(
        &self,
        listener: net::TcpListener,
    ) -> io::Result<Arc<dyn runtime::AsyncTcpListener>> {
        self.0.wrap_tcp_listener(listener)
    }

    fn connect_tcp<'a>(
        &'a self,
        remote_addr: net::SocketAddr,
    ) -> pin::Pin<
        Box<
            dyn future::Future<
                    Output = io::Result<Arc<dyn runtime::AsyncTcpStream>>,
                > + Send
                + 'a,
        >,
    > {
        self.0.connect_tcp(remote_addr)
    }

    fn resolve_host<'a>(
        &'a self,
        host: &'a str,
    ) -> pin::Pin<
        Box<
            dyn future::Future<Output = io::Result<Vec<net::SocketAddr>>>
                + Send
                + 'a,
        >,
    > {
        self.0.resolve_host(host)
    }

    fn now(&self) -> Instant {
        self.0.now()
    }

    fn sleep(
        &self,
        duration: Duration,
    ) -> pin::Pin<Box<dyn future::Future<Output = ()> + Send + 'static>> {
        self.0.sleep(duration)
    }

    fn interval(&self, period: Duration) -> Box<dyn runtime::AsyncInterval> {
        self.0.interval(period)
    }

    fn block_on(&self, future: pin::Pin<Box<dyn future::Future<Output = ()> + '_>>) {
        self.0.block_on(future)
    }

    fn yield_now(
        &self,
    ) -> pin::Pin<Box<dyn future::Future<Output = ()> + Send + 'static>> {
        self.0.yield_now()
    }

    fn name(&self) -> &'static str {
        self.0.name()
    }
}
