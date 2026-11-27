use serde::{Deserialize, Serialize};

use crate::models::Id;
use crate::models::id::marker::UserMarker;

pub mod op {
    pub const IDENTIFY: u8 = 0;
    pub const SELECT_PROTOCOL: u8 = 1;
    pub const HEARTBEAT: u8 = 3;
    pub const SPEAKING: u8 = 5;
    pub const RESUME: u8 = 7;
    pub const VIDEO: u8 = 12;
    pub const SESSION_UPDATE: u8 = 14;
    pub const MEDIA_SINK_WANTS: u8 = 15;
    pub const VOICE_BACKEND_VERSION: u8 = 16;
    pub const DAVE_TRANSITION_READY: u8 = 23;
    pub const DAVE_MLS_KEY_PACKAGE: u8 = 26;
    pub const DAVE_MLS_COMMIT_WELCOME: u8 = 28;
    pub const DAVE_MLS_INVALID_COMMIT_WELCOME: u8 = 31;
    pub const NO_ROUTE: u8 = 32;
}

pub mod recv {
    pub const READY: u8 = 2;
    pub const HEARTBEAT: u8 = 3;
    pub const SESSION_DESCRIPTION: u8 = 4;
    pub const SPEAKING: u8 = 5;
    pub const HEARTBEAT_ACK: u8 = 6;
    pub const HELLO: u8 = 8;
    pub const RESUMED: u8 = 9;
    pub const CLIENTS_CONNECT: u8 = 11;
    pub const VIDEO: u8 = 12;
    pub const CLIENT_DISCONNECT: u8 = 13;
    pub const SESSION_UPDATE: u8 = 14;
    pub const MEDIA_SINK_WANTS: u8 = 15;
    pub const VOICE_BACKEND_VERSION: u8 = 16;
    pub const FLAGS: u8 = 18;
    pub const PLATFORM: u8 = 20;
    pub const DAVE_PREPARE_TRANSITION: u8 = 21;
    pub const DAVE_EXECUTE_TRANSITION: u8 = 22;
    pub const DAVE_PREPARE_EPOCH: u8 = 24;
    pub const DAVE_MLS_EXTERNAL_SENDER: u8 = 25;
    pub const DAVE_MLS_PROPOSALS: u8 = 27;
    pub const DAVE_MLS_ANNOUNCE_COMMIT_TRANSITION: u8 = 29;
    pub const DAVE_MLS_WELCOME: u8 = 30;
    pub const SFU_UPDATE: u8 = 33;
}

pub mod close {

    pub const AUTHENTICATION_FAILED: u16 = 4004;

    pub const INVALID_SESSION: u16 = 4006;

    pub const SERVER_NOT_FOUND: u16 = 4011;

    pub const SERVER_CRASH: u16 = 4015;

    pub const CANCELED: u16 = 4016;
    pub const HEARTBEAT_TIMEOUT: u16 = 4800;
    pub const UNRESUMABLE: u16 = 4801;
    pub const RESET_BACKOFF: u16 = 4802;
    pub const REPEATED_MLS_INVALID_MESSAGES: u16 = 4803;

    pub fn is_fatal(code: u16) -> bool {
        matches!(
            code,
            AUTHENTICATION_FAILED | INVALID_SESSION | SERVER_NOT_FOUND | SERVER_CRASH | CANCELED
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum EncryptionMode {
    #[serde(rename = "aead_aes256_gcm_rtpsize")]
    AeadAes256GcmRtpsize,
    #[serde(rename = "aead_xchacha20_poly1305_rtpsize")]
    AeadXChaCha20Poly1305Rtpsize,
    #[serde(rename = "xsalsa20_poly1305")]
    XSalsa20Poly1305,
    #[serde(rename = "xsalsa20_poly1305_suffix")]
    XSalsa20Poly1305Suffix,
    #[serde(rename = "xsalsa20_poly1305_lite")]
    XSalsa20Poly1305Lite,
    #[serde(rename = "xsalsa20_poly1305_lite_rtpsize")]
    XSalsa20Poly1305LiteRtpsize,
    #[serde(rename = "aead_aes256_gcm")]
    AeadAes256Gcm,
}

#[derive(Serialize)]
pub struct Identify<'a> {
    pub server_id: u64,
    pub channel_id: u64,
    pub user_id: u64,
    pub session_id: &'a str,
    pub token: &'a str,

    pub max_dave_protocol_version: u16,

    pub video: bool,
    pub streams: Vec<Stream>,
}

#[derive(Serialize)]
pub struct SelectProtocol<'a> {
    pub protocol: &'a str,
    pub data: SelectProtocolData<'a>,
    pub rtc_connection_id: &'a str,
    pub codecs: Vec<Codec>,
}

#[derive(Serialize)]
pub struct SelectProtocolData<'a> {
    pub address: &'a str,
    pub port: u16,
    pub mode: EncryptionMode,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Codec {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub priority: i32,
    pub payload_type: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rtx_payload_type: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decode: Option<bool>,
}

pub fn opus_codec() -> Codec {
    Codec {
        name: "opus".into(),
        kind: "audio".into(),
        priority: 1000,
        payload_type: 120,
        rtx_payload_type: None,
        encode: None,
        decode: None,
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Stream {
    #[serde(rename = "type")]
    pub kind: String,
    pub rid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssrc: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rtx_ssrc: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bitrate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_framerate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_resolution: Option<StreamResolution>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StreamResolution {
    #[serde(rename = "type")]
    pub kind: String,
    pub width: i32,
    pub height: i32,
}

#[derive(Deserialize)]
pub struct Hello {
    pub heartbeat_interval: u64,
    #[serde(default)]
    pub v: u8,
}

#[derive(Deserialize)]
pub struct Ready {
    pub ssrc: u32,
    pub ip: String,
    pub port: u16,
    pub modes: Vec<EncryptionMode>,
    #[serde(default)]
    pub experiments: Vec<String>,
    #[serde(default)]
    pub streams: Vec<Stream>,
}

#[derive(Deserialize)]
pub struct SessionDescription {
    pub audio_codec: String,
    pub video_codec: String,
    pub media_session_id: String,
    pub mode: Option<EncryptionMode>,
    pub secret_key: Vec<u8>,
    #[serde(default)]
    pub dave_protocol_version: u16,
    #[serde(default)]
    pub sdp: Option<String>,
    #[serde(default)]
    pub keyframe_interval: Option<i32>,
}

#[derive(Serialize)]
pub struct Heartbeat {
    pub t: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq_ack: Option<u64>,
}

pub mod speaking {
    pub const NONE: u8 = 0;
    pub const VOICE: u8 = 1;
    pub const SOUNDSHARE: u8 = 2;
    pub const PRIORITY: u8 = 4;
    pub const HIDDEN: u8 = 8;
}

#[derive(Serialize, Deserialize)]
pub struct Speaking {
    pub speaking: u8,
    pub ssrc: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Id<UserMarker>>,
}

#[derive(Serialize)]
pub struct Resume<'a> {
    pub server_id: u64,
    pub channel_id: u64,
    pub session_id: &'a str,
    pub token: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq_ack: Option<u64>,
}

#[derive(Deserialize)]
pub struct ClientsConnect {
    pub user_ids: Vec<Id<UserMarker>>,
}

#[derive(Deserialize)]
pub struct ClientDisconnect {
    pub user_id: Id<UserMarker>,
}

#[derive(Serialize, Deserialize)]
pub struct Video {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Id<UserMarker>>,
    pub audio_ssrc: u32,
    pub video_ssrc: u32,
    pub rtx_ssrc: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streams: Option<Vec<Stream>>,
}

#[derive(Deserialize)]
pub struct SessionUpdate {
    pub audio_codec: Option<String>,
    pub video_codec: Option<String>,
    pub media_session_id: Option<String>,
    pub keyframe_interval: Option<i32>,
}

#[derive(Deserialize)]
pub struct VoiceBackendVersion {
    pub voice: Option<String>,
    pub rtc_worker: Option<String>,
}

#[derive(Deserialize)]
pub struct Flags {
    pub user_id: Id<UserMarker>,
    pub flags: u64,
}

#[derive(Deserialize)]
pub struct Platform {
    pub user_id: Id<UserMarker>,
    pub platform: String,
}

#[derive(Deserialize)]
pub struct DavePrepareTransition {
    pub transition_id: u16,
    pub protocol_version: u16,
}

#[derive(Deserialize)]
pub struct DaveExecuteTransition {
    pub transition_id: u16,
}

#[derive(Deserialize)]
pub struct DavePrepareEpoch {
    pub epoch: u32,
    pub protocol_version: u16,
}

#[derive(Serialize)]
pub struct DaveTransitionReady {
    pub transition_id: u16,
}

#[derive(Serialize)]
pub struct DaveMlsInvalidCommitWelcome {
    pub transition_id: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ProposalsOperationType {
    Append = 0,
    Revoke = 1,
}

impl ProposalsOperationType {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self::Append),
            1 => Some(Self::Revoke),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct BinaryFrame<'a> {
    pub seq: u16,
    pub opcode: u8,
    pub payload: &'a [u8],
}

impl<'a> BinaryFrame<'a> {
    pub fn parse(buf: &'a [u8]) -> Option<Self> {
        if buf.len() < 3 {
            return None;
        }
        let seq = u16::from_be_bytes([buf[0], buf[1]]);
        let opcode = buf[2];
        Some(Self {
            seq,
            opcode,
            payload: &buf[3..],
        })
    }
}

pub const MAX_DAVE_PROTOCOL_VERSION: u16 = 1;
