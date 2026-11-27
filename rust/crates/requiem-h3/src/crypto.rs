use btls::error::ErrorStack;
use btls::ssl;
use btls::x509;
use std::any::Any;
use std::collections::VecDeque;
use std::env;
use std::ffi::c_int;
use std::ffi;
use std::ptr;
use std::slice;
use std::sync::{Arc, OnceLock};

use bytes::BytesMut;
use foreign_types::ForeignType;
use quinn_proto::crypto::{
    self, CryptoError, ExportKeyingMaterialError, HeaderKey, KeyPair, Keys, PacketKey,
};
use quinn_proto::transport_parameters::TransportParameters;
use quinn_proto::{ConnectError, ConnectionId, Side, TransportError, TransportErrorCode};
use ring::{aead, hkdf};

pub type SslFactory =
    dyn Fn(&str) -> Result<(ssl::Ssl, Option<Vec<u8>>), String> + Send + Sync;

pub type ParamsEncoder = dyn Fn(&TransportParameters) -> Vec<u8> + Send + Sync;

pub struct ClientConfig {
    pub ssl: Arc<SslFactory>,
    pub params: Arc<ParamsEncoder>,
}

pub struct HandshakeData {
    pub protocol: Option<Vec<u8>>,
}

impl crypto::ClientConfig for ClientConfig {
    fn start_session(
        self: Arc<Self>,
        version: u32,
        server_name: &str,
        params: &TransportParameters,
    ) -> Result<Box<dyn crypto::Session>, ConnectError> {
        if version != 1 {
            return Err(ConnectError::UnsupportedVersion);
        }
        let (ssl, remembered) = (self.ssl)(server_name)
            .map_err(|_| ConnectError::InvalidServerName(server_name.into()))?;
        let remembered = remembered
            .and_then(|bytes| TransportParameters::read(Side::Client, &mut &bytes[..]).ok());
        Ok(Box::new(Session::new(
            ssl,
            (self.params)(params),
            remembered,
        )))
    }
}

const LEVEL_EARLY: usize = 1;
const LEVEL_APPLICATION: u32 = 3;

enum Event {
    Data(Vec<u8>),
    Keys(Box<Keys>),
}

#[derive(Default)]
struct Shared {
    events: VecDeque<Event>,
    read: [Option<Secret>; 4],
    write: [Option<Secret>; 4],
    alert: Option<u8>,
}

#[derive(Clone)]
struct Secret {
    cipher: u16,
    secret: Vec<u8>,
}

fn ex_index() -> c_int {
    static INDEX: OnceLock<c_int> = OnceLock::new();
    // SAFETY: SSL_get_ex_new_index only reserves an ex-data slot; the null pointers and absent
    // callbacks are its documented "no argument, no free function" values.
    *INDEX.get_or_init(|| unsafe {
        btls_sys::SSL_get_ex_new_index(0, ptr::null_mut(), ptr::null_mut(), None, None)
    })
}

unsafe fn shared<'a>(ssl: *mut btls_sys::SSL) -> &'a mut Shared {
    // SAFETY: the caller guarantees `ssl` is a live SSL whose ex-data slot `Session::new` set to a
    // boxed `Shared` that outlives the returned reference and is not aliased.
    unsafe { &mut *(btls_sys::SSL_get_ex_data(ssl, ex_index()) as *mut Shared) }
}

unsafe extern "C" fn set_read_secret(
    ssl: *mut btls_sys::SSL,
    level: btls_sys::ssl_encryption_level_t,
    cipher: *const btls_sys::SSL_CIPHER,
    secret: *const u8,
    len: usize,
) -> c_int {
    // SAFETY: BoringSSL calls this callback with the SSL registered by `Session::new`; it runs
    // synchronously inside an SSL call made while the session is borrowed mutably, so the `Shared`
    // reference is unique.
    let state = unsafe { shared(ssl) };
    state.read[level.0 as usize] = Some(Secret {
        // SAFETY: BoringSSL passes a valid, non-null cipher pointer to the secret callbacks.
        cipher: unsafe { btls_sys::SSL_CIPHER_get_protocol_id(cipher) },
        // SAFETY: BoringSSL guarantees `secret` points to `len` readable bytes for the duration of
        // the callback; they are copied out immediately.
        secret: unsafe { slice::from_raw_parts(secret, len) }.to_vec(),
    });
    state.maybe_keys(level.0 as usize);
    1
}

unsafe extern "C" fn set_write_secret(
    ssl: *mut btls_sys::SSL,
    level: btls_sys::ssl_encryption_level_t,
    cipher: *const btls_sys::SSL_CIPHER,
    secret: *const u8,
    len: usize,
) -> c_int {
    // SAFETY: same contract as `set_read_secret`: the SSL is the one registered by `Session::new`
    // and the reference is unique for the duration of the callback.
    let state = unsafe { shared(ssl) };
    state.write[level.0 as usize] = Some(Secret {
        // SAFETY: BoringSSL passes a valid, non-null cipher pointer to the secret callbacks.
        cipher: unsafe { btls_sys::SSL_CIPHER_get_protocol_id(cipher) },
        // SAFETY: BoringSSL guarantees `secret` points to `len` readable bytes for the duration of
        // the callback; they are copied out immediately.
        secret: unsafe { slice::from_raw_parts(secret, len) }.to_vec(),
    });
    state.maybe_keys(level.0 as usize);
    1
}

unsafe extern "C" fn add_handshake_data(
    ssl: *mut btls_sys::SSL,
    _level: btls_sys::ssl_encryption_level_t,
    data: *const u8,
    len: usize,
) -> c_int {
    // SAFETY: BoringSSL guarantees `data` points to `len` readable bytes for the duration of the
    // callback.
    let bytes = unsafe { slice::from_raw_parts(data, len) };
    // SAFETY: the SSL is the one registered by `Session::new` and the reference is unique for the
    // duration of the callback.
    let state = unsafe { shared(ssl) };
    match state.events.back_mut() {
        Some(Event::Data(existing)) => existing.extend_from_slice(bytes),
        _ => state.events.push_back(Event::Data(bytes.to_vec())),
    }
    1
}

unsafe extern "C" fn flush_flight(_ssl: *mut btls_sys::SSL) -> c_int {
    1
}

unsafe extern "C" fn send_alert(
    ssl: *mut btls_sys::SSL,
    _level: btls_sys::ssl_encryption_level_t,
    alert: u8,
) -> c_int {
    // SAFETY: the SSL is the one registered by `Session::new` and the reference is unique for the
    // duration of the callback.
    unsafe { shared(ssl) }.alert = Some(alert);
    1
}

static METHOD: btls_sys::SSL_QUIC_METHOD = btls_sys::SSL_QUIC_METHOD {
    set_read_secret: Some(set_read_secret),
    set_write_secret: Some(set_write_secret),
    add_handshake_data: Some(add_handshake_data),
    flush_flight: Some(flush_flight),
    send_alert: Some(send_alert),
};

impl Shared {
    fn maybe_keys(&mut self, level: usize) {
        if let (Some(read), Some(write)) = (&self.read[level], &self.write[level]) {
            if let Some(keys) = keys_from_secrets(write, read) {
                self.events.push_back(Event::Keys(Box::new(keys)));
            }
        }
    }
}

pub struct Session {
    ssl: ssl::Ssl,
    shared: Box<Shared>,
    delivered_handshake_data: bool,

    remembered: Option<TransportParameters>,

    started: bool,

    one_rtt: Option<(Secret, Secret)>,
}

// SAFETY: quinn requires `Sync`. Every `&self` method only reads connection state through BoringSSL
// accessors that do not mutate it, and all mutation takes `&mut self`, so a shared reference never
// races with a writer.
unsafe impl Sync for Session {}

impl Session {
    fn new(ssl: ssl::Ssl, params: Vec<u8>, remembered: Option<TransportParameters>) -> Self {
        let mut shared = Box::new(Shared::default());
        // SAFETY: `raw` comes from the live `ssl` this session owns. `shared` is boxed, so the
        // ex-data pointer stays valid for as long as the session, which owns both. `METHOD` is a
        // static and `params` is a valid buffer of the given length that BoringSSL copies.
        unsafe {
            let raw = ssl.as_ptr();
            btls_sys::SSL_set_connect_state(raw);
            btls_sys::SSL_set_ex_data(
                raw,
                ex_index(),
                &mut *shared as *mut Shared as *mut ffi::c_void,
            );
            btls_sys::SSL_set_quic_use_legacy_codepoint(raw, 0);
            assert_eq!(btls_sys::SSL_set_quic_method(raw, &METHOD), 1);
            assert_eq!(
                btls_sys::SSL_set_quic_transport_params(raw, params.as_ptr(), params.len()),
                1
            );
        }
        Self {
            ssl,
            shared,
            delivered_handshake_data: false,
            remembered,
            started: false,
            one_rtt: None,
        }
    }

    fn raw(&self) -> *mut btls_sys::SSL {
        self.ssl.as_ptr()
    }

    fn alpn(&self) -> Option<Vec<u8>> {
        let mut data = ptr::null();
        let mut len = 0;
        // SAFETY: `self.raw()` is the live SSL owned by this session and the out-pointers are valid
        // locals.
        unsafe { btls_sys::SSL_get0_alpn_selected(self.raw(), &mut data, &mut len) };
        (!data.is_null() && len > 0)
            // SAFETY: when `data` is non-null BoringSSL returns `len` bytes owned by the SSL, valid
            // until the next SSL call; they are copied out before then.
            .then(|| unsafe { slice::from_raw_parts(data, len as usize) }.to_vec())
    }

    fn fail(&self) -> TransportError {
        if env::var_os("H3_DEBUG").is_some() {
            if let Some(chain) = self.ssl.peer_cert_chain() {
                for cert in chain {
                    let name = |n: &x509::X509NameRef| {
                        n.entries()
                            .map(|e| {
                                e.data()
                                    .as_utf8()
                                    .map(|s| s.to_string())
                                    .unwrap_or_default()
                            })
                            .collect::<Vec<_>>()
                            .join("/")
                    };
                    tracing::debug!(
                        "chain: subject {} / issuer {}",
                        name(cert.subject_name()),
                        name(cert.issuer_name())
                    );
                }
            }
        }
        TransportError {
            code: TransportErrorCode::crypto(self.shared.alert.unwrap_or(80)),
            frame: None,
            reason: format!("TLS failure: {}", ErrorStack::get()),
        }
    }

    fn advance(&mut self) -> Result<(), TransportError> {
        // SAFETY: `self.raw()` is the live SSL owned by this session, and `&mut self` means nothing
        // else is using it during the handshake calls.
        unsafe {
            if btls_sys::SSL_in_init(self.raw()) != 0 {
                let mut ret = btls_sys::SSL_do_handshake(self.raw());
                if ret <= 0 {
                    let mut err = btls_sys::SSL_get_error(self.raw(), ret);
                    if err == btls_sys::SSL_ERROR_EARLY_DATA_REJECTED as c_int {
                        btls_sys::SSL_reset_early_data_reject(self.raw());
                        ret = btls_sys::SSL_do_handshake(self.raw());
                        err = if ret <= 0 {
                            btls_sys::SSL_get_error(self.raw(), ret)
                        } else {
                            btls_sys::SSL_ERROR_WANT_READ as c_int
                        };
                    }
                    if err != btls_sys::SSL_ERROR_WANT_READ as c_int
                        && err != btls_sys::SSL_ERROR_WANT_WRITE as c_int
                    {
                        return Err(self.fail());
                    }
                }
            } else if btls_sys::SSL_process_quic_post_handshake(self.raw()) != 1 {
                return Err(self.fail());
            }
        }
        Ok(())
    }
}

impl crypto::Session for Session {
    fn initial_keys(&self, dst_cid: &ConnectionId, side: Side) -> Keys {
        initial_keys(dst_cid, side)
    }

    fn handshake_data(&self) -> Option<Box<dyn Any>> {
        self.delivered_handshake_data.then(|| {
            Box::new(HandshakeData {
                protocol: self.alpn(),
            }) as Box<dyn Any>
        })
    }

    fn peer_identity(&self) -> Option<Box<dyn Any>> {
        None
    }

    fn early_crypto(&self) -> Option<(Box<dyn HeaderKey>, Box<dyn PacketKey>)> {
        let secret = self.shared.write[LEVEL_EARLY].as_ref()?;
        Some((
            Box::new(HeaderKeys::new(secret)?),
            Box::new(PacketKeys::new(secret)?),
        ))
    }

    fn early_data_accepted(&self) -> Option<bool> {
        if self.shared.write[LEVEL_EARLY].is_none() || self.is_handshaking() {
            return None;
        }
        // SAFETY: `self.raw()` is the live SSL owned by this session.
        Some(unsafe { btls_sys::SSL_early_data_accepted(self.raw()) } == 1)
    }

    fn is_handshaking(&self) -> bool {
        // SAFETY: `self.raw()` is the live SSL owned by this session.
        unsafe { btls_sys::SSL_in_init(self.raw()) != 0 }
    }

    fn read_handshake(&mut self, buf: &[u8]) -> Result<bool, TransportError> {
        // SAFETY: `self.raw()` is the live SSL owned by this session. `buf` is valid for
        // `buf.len()` bytes, which BoringSSL copies into its own buffer.
        unsafe {
            let level = btls_sys::SSL_quic_read_level(self.raw());
            if btls_sys::SSL_provide_quic_data(self.raw(), level, buf.as_ptr(), buf.len()) != 1 {
                return Err(self.fail());
            }
        }
        self.advance()?;
        if !self.delivered_handshake_data && (self.alpn().is_some() || !self.is_handshaking()) {
            self.delivered_handshake_data = true;
            return Ok(true);
        }
        Ok(false)
    }

    fn transport_parameters(&self) -> Result<Option<TransportParameters>, TransportError> {
        let mut data = ptr::null();
        let mut len = 0;
        // SAFETY: `self.raw()` is the live SSL owned by this session and the out-pointers are valid
        // locals.
        unsafe { btls_sys::SSL_get_peer_quic_transport_params(self.raw(), &mut data, &mut len) };
        if data.is_null() || len == 0 {
            return Ok(self.remembered);
        }
        // SAFETY: when `data` is non-null BoringSSL returns `len` bytes owned by the SSL, valid
        // until the next SSL call; they are parsed before then.
        let mut bytes = unsafe { slice::from_raw_parts(data, len) };
        TransportParameters::read(Side::Client, &mut bytes)
            .map(Some)
            .map_err(Into::into)
    }

    fn write_handshake(&mut self, buf: &mut Vec<u8>) -> Option<Keys> {
        if !self.started {
            self.started = true;
            let _ = self.advance();
        }
        loop {
            match self.shared.events.front() {
                Some(Event::Data(_)) => {
                    if let Some(Event::Data(data)) = self.shared.events.pop_front() {
                        buf.extend_from_slice(&data);
                    }
                }

                Some(Event::Keys(_)) if !buf.is_empty() => return None,
                Some(Event::Keys(_)) => {
                    let Some(Event::Keys(keys)) = self.shared.events.pop_front() else {
                        unreachable!()
                    };
                    if self.one_rtt.is_none()
                        && self.shared.write[LEVEL_APPLICATION as usize].is_some()
                        && self.shared.read[LEVEL_APPLICATION as usize].is_some()
                    {
                        self.one_rtt = Some((
                            self.shared.write[LEVEL_APPLICATION as usize]
                                .clone()
                                .unwrap(),
                            self.shared.read[LEVEL_APPLICATION as usize]
                                .clone()
                                .unwrap(),
                        ));
                    }
                    return Some(*keys);
                }
                None => return None,
            }
        }
    }

    fn next_1rtt_keys(&mut self) -> Option<KeyPair<Box<dyn PacketKey>>> {
        let (local, remote) = self.one_rtt.as_mut()?;
        let next_local = next_secret(local);
        let next_remote = next_secret(remote);
        *local = next_local.clone();
        *remote = next_remote.clone();
        Some(KeyPair {
            local: Box::new(PacketKeys::new(&next_local)?),
            remote: Box::new(PacketKeys::new(&next_remote)?),
        })
    }

    fn is_valid_retry(&self, orig_dst_cid: &ConnectionId, header: &[u8], payload: &[u8]) -> bool {
        const KEY: [u8; 16] = [
            0xbe, 0x0c, 0x69, 0x0b, 0x9f, 0x66, 0x57, 0x5a, 0x1d, 0x76, 0x6b, 0x54, 0xe3, 0x68,
            0xc8, 0x4e,
        ];
        const NONCE: [u8; 12] = [
            0x46, 0x15, 0x99, 0xd3, 0x5d, 0x63, 0x2b, 0xf2, 0x23, 0x98, 0x25, 0xbb,
        ];

        if payload.len() < 16 {
            return false;
        }
        let (body, tag) = payload.split_at(payload.len() - 16);
        let mut aad = Vec::with_capacity(1 + orig_dst_cid.len() + header.len() + body.len());
        aad.push(orig_dst_cid.len() as u8);
        aad.extend_from_slice(orig_dst_cid);
        aad.extend_from_slice(header);
        aad.extend_from_slice(body);
        let key = aead::LessSafeKey::new(aead::UnboundKey::new(&aead::AES_128_GCM, &KEY).unwrap());
        let Ok(expected) = key.seal_in_place_separate_tag(
            aead::Nonce::assume_unique_for_key(NONCE),
            aead::Aad::from(&aad[..]),
            &mut [],
        ) else {
            return false;
        };
        expected.as_ref() == tag
    }

    fn export_keying_material(
        &self,
        output: &mut [u8],
        label: &[u8],
        context: &[u8],
    ) -> Result<(), ExportKeyingMaterialError> {
        // SAFETY: `self.raw()` is the live SSL owned by this session. Each pointer and length pair
        // comes from a slice valid for that length, and the output buffer is written in place.
        let ok = unsafe {
            btls_sys::SSL_export_keying_material(
                self.raw(),
                output.as_mut_ptr(),
                output.len(),
                label.as_ptr().cast(),
                label.len(),
                context.as_ptr(),
                context.len(),
                1,
            )
        };
        if ok == 1 {
            Ok(())
        } else {
            Err(ExportKeyingMaterialError)
        }
    }
}

struct Len(usize);
impl hkdf::KeyType for Len {
    fn len(&self) -> usize {
        self.0
    }
}

fn expand_label(prk: &hkdf::Prk, label: &str, out: &mut [u8]) {
    let full = [b"tls13 ".as_slice(), label.as_bytes()].concat();
    let length = (out.len() as u16).to_be_bytes();
    let label_len = [full.len() as u8];
    let context_len = [0u8];
    prk.expand(&[&length, &label_len, &full, &context_len], Len(out.len()))
        .expect("expand")
        .fill(out)
        .expect("fill");
}

pub fn initial_keys(dst_cid: &ConnectionId, side: Side) -> Keys {
    const SALT: [u8; 20] = [
        0x38, 0x76, 0x2c, 0xf7, 0xf5, 0x59, 0x34, 0xb3, 0x4d, 0x17, 0x9a, 0xe6, 0xa4, 0xc8, 0x0c,
        0xad, 0xcc, 0xbb, 0x7f, 0x0a,
    ];
    let initial = hkdf::Salt::new(hkdf::HKDF_SHA256, &SALT).extract(dst_cid);
    let derive = |label: &str| {
        let mut secret = [0u8; 32];
        expand_label(&initial, label, &mut secret);
        Secret {
            cipher: 0x1301,
            secret: secret.to_vec(),
        }
    };
    let (client, server) = (derive("client in"), derive("server in"));
    let (local, remote) = if side.is_client() {
        (client, server)
    } else {
        (server, client)
    };
    keys_from_secrets(&local, &remote).expect("initial keys")
}

pub fn keys_for_secret(secret: &[u8]) -> Option<(Box<dyn HeaderKey>, Box<dyn PacketKey>)> {
    let secret = Secret {
        cipher: if secret.len() == 48 { 0x1302 } else { 0x1301 },
        secret: secret.to_vec(),
    };
    Some((
        Box::new(HeaderKeys::new(&secret)?),
        Box::new(PacketKeys::new(&secret)?),
    ))
}

fn keys_from_secrets(local: &Secret, remote: &Secret) -> Option<Keys> {
    Some(Keys {
        header: KeyPair {
            local: Box::new(HeaderKeys::new(local)?),
            remote: Box::new(HeaderKeys::new(remote)?),
        },
        packet: KeyPair {
            local: Box::new(PacketKeys::new(local)?),
            remote: Box::new(PacketKeys::new(remote)?),
        },
    })
}

fn algorithms(
    cipher: u16,
) -> Option<(
    &'static aead::Algorithm,
    &'static aead::quic::Algorithm,
    hkdf::Algorithm,
)> {
    Some(match cipher {
        0x1301 => (&aead::AES_128_GCM, &aead::quic::AES_128, hkdf::HKDF_SHA256),
        0x1302 => (&aead::AES_256_GCM, &aead::quic::AES_256, hkdf::HKDF_SHA384),
        0x1303 => (
            &aead::CHACHA20_POLY1305,
            &aead::quic::CHACHA20,
            hkdf::HKDF_SHA256,
        ),
        _ => return None,
    })
}

fn next_secret(current: &Secret) -> Secret {
    let (_, _, hash) = algorithms(current.cipher).expect("known cipher");
    let prk = hkdf::Prk::new_less_safe(hash, &current.secret);
    let mut next = vec![0u8; current.secret.len()];
    expand_label(&prk, "quic ku", &mut next);
    Secret {
        cipher: current.cipher,
        secret: next,
    }
}

struct PacketKeys {
    key: aead::LessSafeKey,
    iv: [u8; 12],
    chacha: bool,
}

impl PacketKeys {
    fn new(secret: &Secret) -> Option<Self> {
        let (aead_alg, _, hash) = algorithms(secret.cipher)?;
        let prk = hkdf::Prk::new_less_safe(hash, &secret.secret);
        let mut key = vec![0u8; aead_alg.key_len()];
        expand_label(&prk, "quic key", &mut key);
        let mut iv = [0u8; 12];
        expand_label(&prk, "quic iv", &mut iv);
        Some(Self {
            key: aead::LessSafeKey::new(aead::UnboundKey::new(aead_alg, &key).ok()?),
            iv,
            chacha: secret.cipher == 0x1303,
        })
    }

    fn nonce(&self, packet: u64) -> aead::Nonce {
        let mut nonce = self.iv;
        for (byte, pn) in nonce[4..].iter_mut().zip(packet.to_be_bytes()) {
            *byte ^= pn;
        }
        aead::Nonce::assume_unique_for_key(nonce)
    }
}

impl PacketKey for PacketKeys {
    fn encrypt(&self, packet: u64, buf: &mut [u8], header_len: usize) {
        let (header, payload_and_tag) = buf.split_at_mut(header_len);
        let (payload, tag_out) =
            payload_and_tag.split_at_mut(payload_and_tag.len() - self.tag_len());
        let tag = self
            .key
            .seal_in_place_separate_tag(self.nonce(packet), aead::Aad::from(&*header), payload)
            .expect("seal");
        tag_out.copy_from_slice(tag.as_ref());
    }

    fn decrypt(
        &self,
        packet: u64,
        header: &[u8],
        payload: &mut BytesMut,
    ) -> Result<(), CryptoError> {
        let plain_len = self
            .key
            .open_in_place(
                self.nonce(packet),
                aead::Aad::from(header),
                payload.as_mut(),
            )
            .map_err(|_| CryptoError)?
            .len();
        payload.truncate(plain_len);
        Ok(())
    }

    fn tag_len(&self) -> usize {
        16
    }

    fn confidentiality_limit(&self) -> u64 {
        if self.chacha { u64::MAX } else { 1 << 23 }
    }

    fn integrity_limit(&self) -> u64 {
        if self.chacha { 1 << 36 } else { 1 << 52 }
    }
}

struct HeaderKeys {
    key: aead::quic::HeaderProtectionKey,
}

impl HeaderKeys {
    fn new(secret: &Secret) -> Option<Self> {
        let (_, quic_alg, hash) = algorithms(secret.cipher)?;
        let prk = hkdf::Prk::new_less_safe(hash, &secret.secret);
        let mut key = vec![0u8; quic_alg.key_len()];
        expand_label(&prk, "quic hp", &mut key);
        Some(Self {
            key: aead::quic::HeaderProtectionKey::new(quic_alg, &key).ok()?,
        })
    }

    fn apply(&self, pn_offset: usize, packet: &mut [u8], decrypting: bool) {
        let sample_len = self.key.algorithm().sample_len();
        let mask = self
            .key
            .new_mask(&packet[pn_offset + 4..pn_offset + 4 + sample_len])
            .expect("sample length");
        let low_bits = if packet[0] & 0x80 != 0 { 0x0f } else { 0x1f };
        if decrypting {
            packet[0] ^= mask[0] & low_bits;
        }
        let pn_len = (packet[0] & 0x03) as usize + 1;
        if !decrypting {
            packet[0] ^= mask[0] & low_bits;
        }
        for i in 0..pn_len {
            packet[pn_offset + i] ^= mask[1 + i];
        }
    }
}

impl HeaderKey for HeaderKeys {
    fn decrypt(&self, pn_offset: usize, packet: &mut [u8]) {
        self.apply(pn_offset, packet, true);
    }

    fn encrypt(&self, pn_offset: usize, packet: &mut [u8]) {
        self.apply(pn_offset, packet, false);
    }

    fn sample_size(&self) -> usize {
        self.key.algorithm().sample_len()
    }
}
