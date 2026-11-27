use crate::utils::voice_settings;
use std::io::ErrorKind;
use std::net::{IpAddr, SocketAddr};
use std::str;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tokio::time::Instant;
use tokio::time;

use aes_gcm::Aes256Gcm;
use aes_gcm::aead::{AeadInPlace, KeyInit};
use chacha20poly1305::XChaCha20Poly1305;
use parking_lot::Mutex;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

use super::opcodes::EncryptionMode;

const TAG_LEN: usize = 16;
const NONCE_LEN: usize = 4;
const DISCOVERY_LEN: usize = 74;

pub const OPUS_PAYLOAD_TYPE: u8 = 120;

const KEEPALIVE: Duration = Duration::from_secs(5);
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(3);
const DISCOVERY_ATTEMPTS: usize = 3;

pub const SUPPORTED_MODES: &[EncryptionMode] = &[
    EncryptionMode::AeadAes256GcmRtpsize,
    EncryptionMode::AeadXChaCha20Poly1305Rtpsize,
];

pub fn choose_mode(offered: &[EncryptionMode]) -> Option<EncryptionMode> {
    SUPPORTED_MODES
        .iter()
        .copied()
        .find(|mode| offered.contains(mode))
}

enum Cipher {
    Aes(Box<Aes256Gcm>),
    XChaCha(Box<XChaCha20Poly1305>),
}

impl Cipher {
    fn new(mode: EncryptionMode, key: &[u8]) -> Result<Self, String> {
        match mode {
            EncryptionMode::AeadAes256GcmRtpsize => Aes256Gcm::new_from_slice(key)
                .map(|c| Self::Aes(Box::new(c)))
                .map_err(|_| "bad key length".into()),
            EncryptionMode::AeadXChaCha20Poly1305Rtpsize => XChaCha20Poly1305::new_from_slice(key)
                .map(|c| Self::XChaCha(Box::new(c)))
                .map_err(|_| "bad key length".into()),
            other => Err(format!("unsupported encryption mode {other:?}")),
        }
    }

    fn seal(&self, packet: &mut Vec<u8>, header_len: usize, counter: u32) -> Option<()> {
        let n4 = counter.to_be_bytes();
        let mut body = packet.split_off(header_len);
        let tag: [u8; TAG_LEN] = match self {
            Self::Aes(c) => {
                let mut nonce = [0u8; 12];
                nonce[..NONCE_LEN].copy_from_slice(&n4);
                c.encrypt_in_place_detached((&nonce).into(), packet, &mut body)
                    .ok()?
                    .into()
            }
            Self::XChaCha(c) => {
                let mut nonce = [0u8; 24];
                nonce[..NONCE_LEN].copy_from_slice(&n4);
                c.encrypt_in_place_detached((&nonce).into(), packet, &mut body)
                    .ok()?
                    .into()
            }
        };
        packet.extend_from_slice(&body);
        packet.extend_from_slice(&tag);
        packet.extend_from_slice(&n4);
        Some(())
    }

    fn open(&self, packet: &[u8], header_len: usize) -> Option<Vec<u8>> {
        let body_end = packet.len().checked_sub(TAG_LEN + NONCE_LEN)?;
        if body_end < header_len {
            return None;
        }
        let n4 = &packet[body_end + TAG_LEN..];
        let tag = &packet[body_end..body_end + TAG_LEN];
        let aad = &packet[..header_len];
        let mut body = packet[header_len..body_end].to_vec();
        match self {
            Self::Aes(c) => {
                let mut nonce = [0u8; 12];
                nonce[..NONCE_LEN].copy_from_slice(n4);
                c.decrypt_in_place_detached((&nonce).into(), aad, &mut body, tag.into())
                    .ok()?;
            }
            Self::XChaCha(c) => {
                let mut nonce = [0u8; 24];
                nonce[..NONCE_LEN].copy_from_slice(n4);
                c.decrypt_in_place_detached((&nonce).into(), aad, &mut body, tag.into())
                    .ok()?;
            }
        }
        Some(body)
    }
}

#[derive(Debug, Clone)]
pub struct RtpPacket {
    pub ssrc: u32,
    pub sequence: u16,
    pub timestamp: u32,
    pub payload_type: u8,
    pub marker: bool,

    pub payload: Vec<u8>,
}

fn clear_header_len(packet: &[u8]) -> Option<usize> {
    if packet.len() < 12 || packet[0] >> 6 != 2 {
        return None;
    }
    let csrc = usize::from(packet[0] & 0x0f);
    let mut len = 12 + 4 * csrc;
    if packet[0] & 0x10 != 0 {
        len += 4;
    }
    (packet.len() >= len).then_some(len)
}

fn is_rtcp(packet: &[u8]) -> bool {
    packet.len() >= 2 && (192..=223).contains(&packet[1])
}

fn parse_rtp(cipher: &Cipher, packet: &[u8]) -> Option<RtpPacket> {
    let header_len = clear_header_len(packet)?;
    let mut body = cipher.open(packet, header_len)?;
    if packet[0] & 0x10 != 0 {
        let words = usize::from(u16::from_be_bytes([
            packet[header_len - 2],
            packet[header_len - 1],
        ]));
        let ext = words * 4;
        if body.len() < ext {
            return None;
        }
        body.drain(..ext);
    }
    if packet[0] & 0x20 != 0 {
        let pad = usize::from(*body.last()?);
        if pad == 0 || pad > body.len() {
            return None;
        }
        body.truncate(body.len() - pad);
    }
    Some(RtpPacket {
        ssrc: u32::from_be_bytes(packet[8..12].try_into().ok()?),
        sequence: u16::from_be_bytes([packet[2], packet[3]]),
        timestamp: u32::from_be_bytes(packet[4..8].try_into().ok()?),
        payload_type: packet[1] & 0x7f,
        marker: packet[1] & 0x80 != 0,
        payload: body,
    })
}

fn discovery_request(ssrc: u32) -> [u8; DISCOVERY_LEN] {
    let mut packet = [0u8; DISCOVERY_LEN];
    packet[..4].copy_from_slice(&[0, 1, 0, 70]);
    packet[4..8].copy_from_slice(&ssrc.to_be_bytes());
    packet
}

fn parse_discovery(packet: &[u8], ssrc: u32) -> Option<(String, u16)> {
    if packet.len() != DISCOVERY_LEN || packet[..2] != [0, 2] || packet[4..8] != ssrc.to_be_bytes()
    {
        return None;
    }
    let ip = &packet[8..72];
    let end = ip.iter().position(|&b| b == 0).unwrap_or(ip.len());
    let ip = str::from_utf8(&ip[..end]).ok()?.to_string();
    Some((ip, u16::from_be_bytes([packet[72], packet[73]])))
}

pub struct UdpTransport {
    socket: Arc<UdpSocket>,
    server: SocketAddr,
    cipher: Mutex<Option<Arc<Cipher>>>,
    counter: AtomicU32,

    pub external: (String, u16),
}

impl UdpTransport {
    pub async fn connect(ip: &str, port: u16, ssrc: u32) -> Result<Self, String> {
        let ip: IpAddr = ip
            .parse()
            .map_err(|e| format!("bad media ip {ip:?}: {e}"))?;
        let server = SocketAddr::new(ip, port);
        let bind: SocketAddr = if ip.is_ipv6() {
            "[::]:0".parse().unwrap()
        } else {
            "0.0.0.0:0".parse().unwrap()
        };
        let socket = UdpSocket::bind(bind)
            .await
            .map_err(|e| format!("bind: {e}"))?;
        if voice_settings::get().qos && ip.is_ipv4() {
            if let Err(err) = socket2::SockRef::from(&socket).set_tos_v4(46 << 2) {
                log::debug!("voice: could not mark the socket for QoS: {err}");
            }
        }
        socket
            .connect(server)
            .await
            .map_err(|e| format!("connect {server}: {e}"))?;

        let request = discovery_request(ssrc);
        let mut buf = [0u8; 1500];
        for attempt in 1..=DISCOVERY_ATTEMPTS {
            socket
                .send(&request)
                .await
                .map_err(|e| format!("discovery send: {e}"))?;
            let deadline = Instant::now() + DISCOVERY_TIMEOUT;
            loop {
                match time::timeout_at(deadline, socket.recv(&mut buf)).await {
                    Ok(Ok(n)) => {
                        if let Some(external) = parse_discovery(&buf[..n], ssrc) {
                            log::info!(
                                "voice: ip discovery {}:{} (server {server})",
                                external.0,
                                external.1
                            );
                            return Ok(Self {
                                socket: Arc::new(socket),
                                server,
                                cipher: Mutex::new(None),
                                counter: AtomicU32::new(0),
                                external,
                            });
                        }
                    }
                    Ok(Err(e)) => return Err(format!("discovery recv: {e}")),
                    Err(_) => break,
                }
            }
            log::warn!("voice: ip discovery attempt {attempt} timed out");
        }
        Err(format!("ip discovery against {server} timed out"))
    }

    pub fn set_key(&self, mode: EncryptionMode, key: &[u8]) -> Result<(), String> {
        *self.cipher.lock() = Some(Arc::new(Cipher::new(mode, key)?));
        Ok(())
    }

    pub fn server(&self) -> SocketAddr {
        self.server
    }

    pub async fn send_rtp(
        &self,
        payload_type: u8,
        marker: bool,
        seq: u16,
        timestamp: u32,
        ssrc: u32,
        payload: &[u8],
    ) -> Result<(), String> {
        let cipher = self.cipher.lock().clone().ok_or("no key yet")?;
        let mut packet = Vec::with_capacity(12 + payload.len() + TAG_LEN + NONCE_LEN);
        packet.push(0x80);
        packet.push(payload_type | if marker { 0x80 } else { 0 });
        packet.extend_from_slice(&seq.to_be_bytes());
        packet.extend_from_slice(&timestamp.to_be_bytes());
        packet.extend_from_slice(&ssrc.to_be_bytes());
        packet.extend_from_slice(payload);
        let counter = self.counter.fetch_add(1, Ordering::Relaxed);
        cipher.seal(&mut packet, 12, counter).ok_or("seal failed")?;
        self.socket.send(&packet).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn spawn_receiver(&self) -> UnboundedReceiver<RtpPacket> {
        let (tx, rx) = unbounded_channel();
        let socket = self.socket.clone();
        let cipher = self.cipher.lock().clone();
        tokio::spawn(async move {
            let Some(cipher) = cipher else { return };
            let mut keepalive = time::interval(KEEPALIVE);
            let mut sent: u64 = 0;
            let mut buf = vec![0u8; 2048];
            let mut failures: u64 = 0;
            loop {
                tokio::select! {
                    _ = keepalive.tick() => {
                        sent += 1;
                        if socket.send(&sent.to_le_bytes()).await.is_err() {
                            break;
                        }
                    }
                    recv = socket.recv(&mut buf) => {
                        let n = match recv {
                            Ok(n) => n,

                            Err(e) if e.kind() == ErrorKind::ConnectionReset => continue,
                            Err(e) => {
                                log::warn!("voice: udp recv: {e}");
                                break;
                            }
                        };
                        let packet = &buf[..n];

                        if n <= 8 || is_rtcp(packet) {
                            continue;
                        }
                        match parse_rtp(&cipher, packet) {
                            Some(rtp) => {
                                if tx.send(rtp).is_err() {
                                    break;
                                }
                            }
                            None => {
                                failures += 1;
                                if failures == 1 || failures.is_multiple_of(500) {
                                    log::warn!("voice: dropped {failures} undecryptable packet(s) ({n}B)");
                                }
                            }
                        }
                    }
                }
            }
        });
        rx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: [u8; 32] = {
        let mut key = [0; 32];
        let mut i = 0;
        while i < 32 {
            key[i] = 0xA0 + i as u8;
            i += 1;
        }
        key
    };
    const PLAIN: [u8; 15] = [
        0x80, 0x78, 0xff, 0xff, 0x01, 0x02, 0x06, 0xc4, 0xde, 0xad, 0xbe, 0xef, 0x26, 0x31, 0x3c,
    ];

    fn hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    fn check(mode: EncryptionMode, expected: &str) {
        let cipher = Cipher::new(mode, &KEY).unwrap();
        let mut packet = PLAIN.to_vec();
        cipher.seal(&mut packet, 12, 0xFEFF_FFFF).unwrap();
        assert_eq!(packet, hex(expected));
        let rtp = parse_rtp(&cipher, &packet).unwrap();
        assert_eq!(rtp.payload, &PLAIN[12..]);
        assert_eq!(rtp.ssrc, 0xdeadbeef);
        assert_eq!(rtp.payload_type, 0x78);
        let mut tampered = packet.clone();
        tampered[12] ^= 1;
        assert!(parse_rtp(&cipher, &tampered).is_none());
    }

    #[test]
    fn aes256_gcm_rtpsize_matches_mmrs() {
        check(
            EncryptionMode::AeadAes256GcmRtpsize,
            "8078ffff010206c4deadbeef17ea434f41bae8c90f0f5dbd8ef56371e936d3feffffff",
        );
    }

    #[test]
    fn xchacha20_poly1305_rtpsize_matches_mmrs() {
        check(
            EncryptionMode::AeadXChaCha20Poly1305Rtpsize,
            "8078ffff010206c4deadbeefa06a33599ca91d7229c3d2508f739e57ee45ebfeffffff",
        );
    }

    #[test]
    fn extension_body_is_stripped_after_decrypt() {
        let cipher = Cipher::new(EncryptionMode::AeadAes256GcmRtpsize, &KEY).unwrap();

        let mut packet = vec![
            0x90, 0x78, 0, 1, 0, 0, 0, 0, 0, 0, 0, 7, 0xBE, 0xDE, 0, 1, 0x10, 0xff, 0, 0, 0xf8,
            0xff, 0xfe,
        ];
        cipher.seal(&mut packet, 16, 1).unwrap();
        let rtp = parse_rtp(&cipher, &packet).unwrap();
        assert_eq!(rtp.payload, vec![0xf8, 0xff, 0xfe]);
        assert_eq!(rtp.ssrc, 7);
    }

    #[test]
    fn padding_does_not_eat_the_aead_tag() {
        let cipher = Cipher::new(EncryptionMode::AeadAes256GcmRtpsize, &KEY).unwrap();
        let mut packet = vec![
            0xa0, 0x78, 0, 1, 0, 0, 0, 0, 0, 0, 0, 7, 0xf8, 0xff, 0xfe, 1,
        ];
        cipher.seal(&mut packet, 12, 1).unwrap();
        let rtp = parse_rtp(&cipher, &packet).unwrap();
        assert_eq!(rtp.payload, vec![0xf8, 0xff, 0xfe]);
    }

    #[test]
    fn discovery_round_trip() {
        let request = discovery_request(451);
        assert_eq!(request.len(), 74);
        assert_eq!(&request[..8], &[0, 1, 0, 70, 0, 0, 1, 0xc3]);
        let mut reply = [0u8; 74];
        reply[..4].copy_from_slice(&[0, 2, 0, 70]);
        reply[4..8].copy_from_slice(&451u32.to_be_bytes());
        reply[8..8 + 11].copy_from_slice(b"203.0.113.9");
        reply[72..].copy_from_slice(&50000u16.to_be_bytes());
        assert_eq!(
            parse_discovery(&reply, 451),
            Some(("203.0.113.9".into(), 50000))
        );
        assert_eq!(parse_discovery(&reply, 452), None);
    }

    #[test]
    fn prefers_aes_gcm() {
        assert_eq!(
            choose_mode(&[
                EncryptionMode::AeadXChaCha20Poly1305Rtpsize,
                EncryptionMode::AeadAes256GcmRtpsize
            ]),
            Some(EncryptionMode::AeadAes256GcmRtpsize)
        );
        assert_eq!(choose_mode(&[EncryptionMode::XSalsa20Poly1305]), None);
    }
}
