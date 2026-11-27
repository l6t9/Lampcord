pub mod connection;
pub mod crypto;
pub mod driver;
pub mod qpack;
mod qpack_tables;
pub mod transport;

use btls::ssl;
use foreign_types::ForeignType;
use quinn_proto::coding::Codec;
use rand::RngExt;
use rand::seq::SliceRandom;
use std::collections;
use std::env;
use std::net;
use std::sync::Arc;
use std::sync;
use std::time::Duration;
use tokio::time;
use wreq::tls;

use quinn_proto::transport_parameters::TransportParameters;
use quinn_proto::{TransportConfig, VarInt};

const TRUST_ANCHORS: &[u8] = include_bytes!("trust_anchors.bin");

pub fn quic_tls_options(base: &tls::TlsOptions) -> tls::TlsOptions {
    let mut options = base.clone();
    options.grease_enabled = Some(false);
    options.enable_ocsp_stapling = false;
    options.enable_signed_cert_timestamps = false;
    options.pre_shared_key = false;
    if let Some(list) = &options.sigalgs_list {
        if !list.contains("rsa_pkcs1_sha1") {
            options.sigalgs_list = Some(format!("{list}:rsa_pkcs1_sha1").into());
        }
    }
    options
}

#[derive(Default)]
pub struct SessionStore {
    sessions: sync::Mutex<
        collections::HashMap<
            String,
            collections::VecDeque<(ssl::SslSession, Vec<u8>)>,
        >,
    >,

    rtts: sync::Mutex<collections::HashMap<String, Duration>>,
}

impl SessionStore {
    pub fn has_ticket(&self, host: &str) -> bool {
        self.sessions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(host)
            .is_some_and(|q| !q.is_empty())
    }

    pub fn rtt(&self, host: &str) -> Option<Duration> {
        self.rtts
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(host)
            .copied()
    }

    pub(crate) fn set_rtt(&self, host: &str, rtt: Duration) {
        self.rtts
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(host.to_string(), rtt);
    }
}

const TICKETS_PER_HOST: usize = 4;

impl tls::QuicSessionStore for SessionStore {
    fn put(&self, host: &str, session: ssl::SslSession, peer_params: Vec<u8>) {
        let mut sessions = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
        let queue = sessions.entry(host.to_string()).or_default();
        queue.push_back((session, peer_params));
        while queue.len() > TICKETS_PER_HOST {
            queue.pop_front();
        }
    }

    fn take(&self, host: &str) -> Option<(ssl::SslSession, Vec<u8>)> {
        self.sessions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get_mut(host)?
            .pop_back()
    }
}

pub fn quic_ssl(
    base: &tls::TlsOptions,
    host: &str,
    verify: bool,
    sessions: Option<Arc<SessionStore>>,
) -> Result<(ssl::Ssl, Option<Vec<u8>>), String> {
    let store = sessions.map(|s| s as Arc<dyn tls::QuicSessionStore>);
    let (ssl, remembered) = tls::quic_ssl(&quic_tls_options(base), host, verify, store)
        .map_err(|e| e.to_string())?;
    if env::var_os("H3_NO_TA").is_some() {
        return Ok((ssl, remembered));
    }
    // SAFETY: `ssl` is the live handle built just above, and TRUST_ANCHORS is a static byte slice
    // that BoringSSL copies.
    let ok = unsafe {
        btls_sys::SSL_set1_requested_trust_anchors(
            ssl.as_ptr(),
            TRUST_ANCHORS.as_ptr(),
            TRUST_ANCHORS.len(),
        )
    };
    if ok != 1 {
        return Err("SSL_set1_requested_trust_anchors failed".into());
    }
    Ok((ssl, remembered))
}

pub fn chrome_transport_parameters(ours: &TransportParameters) -> Vec<u8> {
    chrome_transport_parameters_resumed(ours, None)
}

pub fn chrome_transport_parameters_resumed(
    _ours: &TransportParameters,
    rtt: Option<Duration>,
) -> Vec<u8> {
    fn varint(out: &mut Vec<u8>, value: u64) {
        let v = VarInt::from_u64(value).expect("varint range");
        let mut buf = bytes::BytesMut::new();
        Codec::encode(&v, &mut buf);
        out.extend_from_slice(&buf);
    }

    let mut rng = rand::rng();
    let int = |value: u64| {
        let mut v = Vec::new();
        varint(&mut v, value);
        v
    };
    let grease_version: u32 = rng.random::<u32>() & 0xf0f0_f0f0 | 0x0a0a_0a0a;
    let mut version_information = 1u32.to_be_bytes().to_vec();
    let (first, second) = if rng.random::<bool>() {
        (grease_version, 1u32)
    } else {
        (1u32, grease_version)
    };
    version_information.extend_from_slice(&first.to_be_bytes());
    version_information.extend_from_slice(&second.to_be_bytes());
    let grease_len = rng.random_range(0..=16usize);
    let grease_payload: Vec<u8> = (0..grease_len).map(|_| rng.random::<u8>()).collect();
    let mut entries: Vec<(u64, Vec<u8>)> = vec![
        (0x03, int(1472)),
        (0x01, int(30_000)),
        (0x04, int(15 * 1024 * 1024)),
        (0x05, int(6 * 1024 * 1024)),
        (0x06, int(6 * 1024 * 1024)),
        (0x07, int(6 * 1024 * 1024)),
        (0x08, int(100)),
        (0x09, int(103)),
        (0x0f, Vec::new()),
        (0x20, int(65536)),
        (0x11, version_information),
        (0x3128, b"ORIG".to_vec()),
    ];
    if let Some(rtt) = rtt {
        entries.push((
            0x3127,
            int(rtt.as_micros().min(0x3fff_ffff_ffff_ffff) as u64),
        ));
    }
    entries.extend([(
        31 * rng.random_range(0..((1u64 << 62) - 27) / 31) + 27,
        grease_payload,
    )]);
    entries.shuffle(&mut rng);
    let mut out = Vec::new();
    for (id, value) in entries {
        varint(&mut out, id);
        varint(&mut out, value.len() as u64);
        out.extend_from_slice(&value);
    }
    out
}

pub fn chrome_transport_config() -> TransportConfig {
    let mut config = TransportConfig::default();
    config.max_idle_timeout(Some(Duration::from_secs(30).try_into().unwrap()));
    config.max_concurrent_bidi_streams(VarInt::from_u32(100));
    config.max_concurrent_uni_streams(VarInt::from_u32(103));
    config.stream_receive_window(VarInt::from_u32(6 * 1024 * 1024));
    config.receive_window(VarInt::from_u32(15 * 1024 * 1024));
    config.mtu_discovery_config(None);
    config
}

pub async fn connect(
    remote: net::SocketAddr,
    host: &str,
    tls: tls::TlsOptions,
    verify: bool,
    sessions: Option<Arc<SessionStore>>,
    timeout: Duration,
) -> Result<connection::Connection, driver::Error> {
    let sessions_for_rtt = sessions.clone();
    let rtt_sink = sessions.clone().map(|s| (s, host.to_string()));
    let ssl: Arc<crypto::SslFactory> =
        Arc::new(move |name: &str| quic_ssl(&tls, name, verify, sessions.clone()));
    let rtt = sessions_for_rtt
        .as_ref()
        .filter(|s| s.has_ticket(host))
        .and_then(|s| s.rtt(host));
    let params: Arc<crypto::ParamsEncoder> =
        Arc::new(move |ours: &TransportParameters| chrome_transport_parameters_resumed(ours, rtt));
    let crypto = Arc::new(crypto::ClientConfig { ssl, params });
    let mut driver =
        driver::Driver::connect(remote, host, crypto, chrome_transport_config(), 0).await?;

    if driver.conn.has_0rtt() {
        return connection::Connection::start(driver, rtt_sink).await;
    }
    let handshake = async {
        while driver.conn.is_handshaking() {
            for event in driver.step().await? {
                if let quinn_proto::Event::ConnectionLost { reason } = event {
                    return Err(driver::Error::Lost(reason));
                }
            }
        }
        Ok(())
    };
    time::timeout(timeout, handshake)
        .await
        .map_err(|_| driver::Error::Other("QUIC handshake timed out".into()))??;
    connection::Connection::start(driver, rtt_sink).await
}
