#![allow(clippy::print_stdout)]

use quinn_proto::crypto::HeaderKey;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::panic;
use std::time::Instant;

use quinn_proto::{ConnectionId, Side};
use requiem_h3::crypto::{initial_keys, keys_for_secret};
use tokio::net::UdpSocket;

fn varint(data: &[u8]) -> Option<(u64, usize)> {
    let first = *data.first()?;
    let len = 1usize << (first >> 6);
    if data.len() < len {
        return None;
    }
    let mut v = (first & 0x3f) as u64;
    for b in &data[1..len] {
        v = (v << 8) | *b as u64;
    }
    Some((v, len))
}

fn frames(mut data: &[u8]) -> String {
    let mut out = Vec::new();
    let mut padding = 0;
    while let Some(&ty) = data.first() {
        if ty == 0 {
            padding += 1;
            data = &data[1..];
            continue;
        }
        if padding > 0 {
            out.push(format!("PADDING x{padding}"));
            padding = 0;
        }
        match ty {
            0x01 => {
                out.push("PING".into());
                data = &data[1..];
            }
            0x02 | 0x03 => {
                out.push("ACK".into());
                break;
            }
            0x06 => {
                let Some((offset, a)) = varint(&data[1..]) else {
                    break;
                };
                let Some((len, b)) = varint(&data[1 + a..]) else {
                    break;
                };
                out.push(format!("CRYPTO off={offset} len={len}"));
                data = &data[(1 + a + b + len as usize).min(data.len())..];
            }
            other => {
                out.push(format!("frame 0x{other:02x}"));
                break;
            }
        }
    }
    if padding > 0 {
        out.push(format!("PADDING x{padding}"));
    }
    out.join(", ")
}

#[derive(Default)]
struct State {
    original_dcid: Option<Vec<u8>>,
    server_cid: Option<Vec<u8>>,
    keylog: Option<String>,
}

impl State {
    fn secrets(&self, label: &str) -> Vec<Vec<u8>> {
        let Some(text) = self
            .keylog
            .as_ref()
            .and_then(|path| fs::read_to_string(path).ok())
        else {
            return Vec::new();
        };
        text.lines()
            .rev()
            .filter(|l| l.starts_with(label))
            .filter_map(|line| {
                let hex = line.split_whitespace().nth(2)?;
                (0..hex.len() / 2)
                    .map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok())
                    .collect()
            })
            .collect()
    }

    fn unprotect(
        &self,
        label: &str,
        pn_offset: usize,
        packet: &[u8],
    ) -> Option<(usize, u64, Vec<u8>)> {
        for secret in self.secrets(label) {
            let Some((hp, key)) = keys_for_secret(&secret) else {
                continue;
            };
            let mut copy = packet.to_vec();
            HeaderKey::decrypt(&*hp, pn_offset, &mut copy);
            let pn_len = (copy[0] & 3) as usize + 1;
            let pn = copy[pn_offset..pn_offset + pn_len]
                .iter()
                .fold(0u64, |a, b| (a << 8) | *b as u64);
            let header = copy[..pn_offset + pn_len].to_vec();
            let mut payload = bytes::BytesMut::from(&copy[pn_offset + pn_len..]);
            if key.decrypt(pn, &header, &mut payload).is_ok() {
                return Some((pn_len, pn, frames(&payload).into_bytes()));
            }
        }
        None
    }

    fn describe(&mut self, datagram: &[u8]) -> String {
        let mut rest = datagram;
        let mut packets = Vec::new();
        while !rest.is_empty() {
            let first = rest[0];
            if first & 0x80 == 0 {
                let dcid_len = self.server_cid.as_ref().map_or(0, |c| c.len());
                let packet = rest.to_vec();
                if packet.len() < 1 + dcid_len + 20 {
                    packets.push(format!("1-RTT {}B (too short)", rest.len()));
                    break;
                }
                match self.unprotect("CLIENT_TRAFFIC_SECRET_0", 1 + dcid_len, &packet) {
                    Some((pn_len, pn, f)) => packets.push(format!(
                        "1-RTT {}B pn_len={pn_len} pn(truncated)={pn} [{}]",
                        rest.len(),
                        String::from_utf8_lossy(&f)
                    )),
                    None => packets.push(format!(
                        "1-RTT {}B first=0x{first:02x} (not unprotected)",
                        rest.len()
                    )),
                }
                break;
            }
            let version = u32::from_be_bytes(rest[1..5].try_into().unwrap());
            let dcid_len = rest[5] as usize;
            let dcid = rest[6..6 + dcid_len].to_vec();
            let scid_len = rest[6 + dcid_len] as usize;
            let mut pos = 7 + dcid_len + scid_len;
            let ty = (first >> 4) & 3;
            let token_len = if ty == 0 {
                let (token_len, n) = varint(&rest[pos..]).unwrap();
                pos += n + token_len as usize;
                token_len
            } else {
                0
            };
            let (length, n) = varint(&rest[pos..]).unwrap();
            pos += n;
            let end = pos + length as usize;
            let mut packet = rest[..end].to_vec();
            let name = ["Initial", "0-RTT", "Handshake", "Retry"][ty as usize];
            let described = match ty {
                0 => {
                    if self.original_dcid.is_none() {
                        self.original_dcid = Some(dcid.clone());
                    }
                    let keys = initial_keys(
                        &ConnectionId::new(self.original_dcid.as_ref().unwrap()),
                        Side::Server,
                    );
                    keys.header.remote.decrypt(pos, &mut packet);
                    let pn_len = (packet[0] & 3) as usize + 1;
                    let pn = packet[pos..pos + pn_len]
                        .iter()
                        .fold(0u64, |a, b| (a << 8) | *b as u64);
                    let header = packet[..pos + pn_len].to_vec();
                    let mut payload = bytes::BytesMut::from(&packet[pos + pn_len..]);
                    let f = if keys
                        .packet
                        .remote
                        .decrypt(pn, &header, &mut payload)
                        .is_ok()
                    {
                        frames(&payload)
                    } else {
                        "undecryptable".into()
                    };
                    format!(
                        "Initial dcid={dcid_len}B scid={scid_len}B token={token_len}B len={end} pn_len={pn_len} pn(truncated)={pn} [{f}]"
                    )
                }
                1 => match self.unprotect("CLIENT_EARLY_TRAFFIC_SECRET", pos, &packet) {
                    Some((pn_len, pn, f)) => format!(
                        "0-RTT len={end} pn_len={pn_len} pn(truncated)={pn} [{}]",
                        String::from_utf8_lossy(&f)
                    ),
                    None => format!("0-RTT len={end} (not unprotected)"),
                },
                2 => match self.unprotect("CLIENT_HANDSHAKE_TRAFFIC_SECRET", pos, &packet) {
                    Some((pn_len, pn, f)) => format!(
                        "Handshake len={end} pn_len={pn_len} pn(truncated)={pn} [{}]",
                        String::from_utf8_lossy(&f)
                    ),
                    None => format!("Handshake len={end} (not unprotected)"),
                },
                _ => format!("{name} len={end}"),
            };
            let _ = version;
            packets.push(described);
            rest = &rest[end..];
        }
        packets.join(" + ")
    }

    fn server_datagram(&mut self, datagram: &[u8]) {
        if datagram.len() > 7 && datagram[0] & 0x80 != 0 && (datagram[0] >> 4) & 3 == 0 {
            let dcid_len = datagram[5] as usize;
            let scid_len = datagram[6 + dcid_len] as usize;
            self.server_cid
                .get_or_insert_with(|| datagram[7 + dcid_len..7 + dcid_len + scid_len].to_vec());
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let port = env::args().nth(1).unwrap_or("4443".into());
    let target: SocketAddr = env::args()
        .nth(2)
        .unwrap_or("127.0.0.1:443".into())
        .parse()
        .unwrap();
    let mut state = State {
        keylog: env::args().nth(3),
        ..State::default()
    };
    let listen = UdpSocket::bind(format!("127.0.0.1:{port}")).await.unwrap();
    let upstream = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let started = Instant::now();
    let mut client: Option<SocketAddr> = None;
    let (mut a, mut b) = (vec![0u8; 65536], vec![0u8; 65536]);
    println!("relaying 127.0.0.1:{port} -> {target}");
    loop {
        tokio::select! {
            r = listen.recv_from(&mut a) => {
                let (n, from) = r.unwrap();
                client = Some(from);
                let datagram = a[..n].to_vec();
                let text = panic::catch_unwind(panic::AssertUnwindSafe(|| state.describe(&datagram))).unwrap_or_else(|_| "?".into());
                println!("{:>6}ms C->S {n:>5}B  {text}", started.elapsed().as_millis());
                upstream.send_to(&a[..n], target).await.unwrap();
            }
            r = upstream.recv_from(&mut b) => {
                let (n, _) = r.unwrap();
                state.server_datagram(&b[..n]);
                println!("{:>6}ms S->C {n:>5}B", started.elapsed().as_millis());
                if let Some(c) = client { listen.send_to(&b[..n], c).await.unwrap(); }
            }
        }
    }
}
