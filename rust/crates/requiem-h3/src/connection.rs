use quinn_proto::coding::Codec;
use std::collections::{HashMap, VecDeque};
use std::env;
use std::error;
use std::future;
use std::mem;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use std::task::{Context, Poll};
use std::time::Duration;
use std::time::Instant;
use tokio::time;

use bytes::Bytes;
use futures_core::Stream;
use quinn_proto::{Dir, Event, ReadError, StreamId, VarInt, WriteError};
use rand::{Rng, RngExt};
use tokio::sync::{mpsc, oneshot};

use crate::driver::{Driver, Error};
use crate::qpack::{self, Decoder, Section};

pub const QPACK_MAX_TABLE_CAPACITY: u64 = 65536;
pub const MAX_FIELD_SECTION_SIZE: u64 = 262144;
pub const QPACK_BLOCKED_STREAMS: u64 = 100;

const FRAME_DATA: u64 = 0x00;
const FRAME_HEADERS: u64 = 0x01;
const FRAME_GOAWAY: u64 = 0x07;

const STREAM_CONTROL: u64 = 0x00;
const STREAM_QPACK_ENCODER: u64 = 0x02;
const STREAM_QPACK_DECODER: u64 = 0x03;

const H3_NO_ERROR: u32 = 0x100;
const H3_REQUEST_CANCELLED: u32 = 0x10c;

const READ_BUDGET: usize = 256 * 1024;
const BODY_QUEUE: usize = 8;

pub type BoxError = Box<dyn error::Error + Send + Sync>;

#[derive(Debug, Clone)]
pub struct RequestHead {
    pub method: String,
    pub authority: String,
    pub path: String,

    pub headers: Vec<(String, String)>,
}

pub enum RequestBody {
    Empty,
    Full(Bytes),
    Stream(Pin<Box<dyn Stream<Item = Result<Bytes, BoxError>> + Send>>),
}

pub struct ResponseHead {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: ResponseBody,
}

pub struct ResponseBody {
    rx: mpsc::Receiver<Result<Bytes, Error>>,
    wake: mpsc::UnboundedSender<Command>,
    key: u64,
}

impl Stream for ResponseBody {
    type Item = Result<Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let polled = self.rx.poll_recv(cx);
        if polled.is_ready() {
            let _ = self.wake.send(Command::Drained);
        }
        polled
    }
}

impl Drop for ResponseBody {
    fn drop(&mut self) {
        let _ = self.wake.send(Command::Cancel(self.key));
    }
}

enum Command {
    Request {
        key: u64,
        head: RequestHead,
        body: RequestBody,
        respond: oneshot::Sender<Result<ResponseHead, Error>>,
    },
    BodyChunk {
        key: u64,
        data: Option<Result<Bytes, BoxError>>,
        ack: oneshot::Sender<()>,
    },

    Drained,

    Cancel(u64),
}

#[derive(Clone)]
pub struct Connection {
    commands: mpsc::UnboundedSender<Command>,
    alive: Arc<AtomicBool>,

    confirmed: Arc<AtomicBool>,

    early_state: Arc<AtomicU8>,
    settings: Arc<Mutex<Option<Vec<(u64, u64)>>>>,
    keys: Arc<AtomicU64>,
}

impl Connection {
    pub async fn start(
        mut driver: Driver,
        rtt_sink: Option<(Arc<crate::SessionStore>, String)>,
    ) -> Result<Self, Error> {
        let settings = settings_frame(&mut rand::rng());
        let mut open_uni = |kind: u64, rest: &[u8]| -> Result<StreamId, Error> {
            let id = driver
                .conn
                .streams()
                .open(Dir::Uni)
                .ok_or_else(|| Error::Other("no stream credit".into()))?;
            let mut data = Vec::new();
            put_varint(&mut data, kind);
            data.extend_from_slice(rest);
            let _ = driver.conn.send_stream(id).write(&data);
            Ok(id)
        };
        open_uni(STREAM_CONTROL, &settings)?;
        let encoder_out = open_uni(STREAM_QPACK_ENCODER, &[])?;
        let decoder_stream = open_uni(STREAM_QPACK_DECODER, &[])?;
        driver.flush().await?;
        let (commands, receiver) = mpsc::unbounded_channel();
        let alive = Arc::new(AtomicBool::new(true));
        let confirmed = Arc::new(AtomicBool::new(!driver.conn.is_handshaking()));
        let early_state = Arc::new(AtomicU8::new(u8::from(driver.conn.has_0rtt())));
        let settings = Arc::new(Mutex::new(None));
        let task = Task {
            encoder: None,
            encoder_out,
            peer_settings: settings.clone(),
            rtt_sink,
            last_rtt: Duration::ZERO,
            early_state: early_state.clone(),
            connected: !driver.conn.is_handshaking(),
            early: Vec::new(),
            confirmed: confirmed.clone(),
            driver,
            commands: receiver,
            command_tx: commands.clone(),
            alive: alive.clone(),
            decoder: Decoder::new(QPACK_MAX_TABLE_CAPACITY),
            decoder_stream,
            sent_inserts: 0,
            out: HashMap::new(),
            streams: HashMap::new(),
            by_key: HashMap::new(),
            peer_uni: HashMap::new(),
            encoder_stream: Vec::new(),
            waiting: VecDeque::new(),
            goaway: false,
        };
        tokio::spawn(task.run());
        Ok(Self {
            commands,
            alive,
            confirmed,
            early_state,
            settings,
            keys: Arc::new(AtomicU64::new(1)),
        })
    }

    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Acquire) && !self.commands.is_closed()
    }

    pub fn zero_rtt(&self) -> Option<Option<bool>> {
        match self.early_state.load(Ordering::Acquire) {
            0 => None,
            1 => Some(None),
            2 => Some(Some(true)),
            _ => Some(Some(false)),
        }
    }

    pub fn peer_settings(&self) -> Option<Vec<(u64, u64)>> {
        self.settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn is_confirmed(&self) -> bool {
        self.confirmed.load(Ordering::Acquire)
    }

    pub async fn send(&self, head: RequestHead, body: RequestBody) -> Result<ResponseHead, Error> {
        let (respond, response) = oneshot::channel();
        let key = self.keys.fetch_add(1, Ordering::Relaxed);
        self.commands
            .send(Command::Request {
                key,
                head,
                body,
                respond,
            })
            .map_err(|_| Error::Other("connection closed".into()))?;
        response
            .await
            .map_err(|_| Error::Other("connection closed".into()))?
    }
}

fn put_varint(out: &mut Vec<u8>, value: u64) {
    let v = VarInt::from_u64(value).expect("varint range");
    let mut buf = bytes::BytesMut::new();
    Codec::encode(&v, &mut buf);
    out.extend_from_slice(&buf);
}

fn get_varint(data: &[u8]) -> Option<(u64, usize)> {
    let first = *data.first()?;
    let len = 1usize << (first >> 6);
    if data.len() < len {
        return None;
    }
    let mut value = (first & 0x3f) as u64;
    for byte in &data[1..len] {
        value = (value << 8) | *byte as u64;
    }
    Some((value, len))
}

pub fn settings_frame(rng: &mut impl Rng) -> Vec<u8> {
    let mut payload = Vec::new();
    for (id, value) in [
        (0x01, QPACK_MAX_TABLE_CAPACITY),
        (0x06, MAX_FIELD_SECTION_SIZE),
        (0x07, QPACK_BLOCKED_STREAMS),
        (0x33, 1),
    ] {
        put_varint(&mut payload, id);
        put_varint(&mut payload, value);
    }

    put_varint(
        &mut payload,
        0x1f * rng.random_range(0..(1u64 << 30)) + 0x21,
    );
    put_varint(&mut payload, rng.random_range(0..u32::MAX) as u64);
    let mut frame = Vec::new();
    put_varint(&mut frame, 0x04);
    put_varint(&mut frame, payload.len() as u64);
    frame.extend_from_slice(&payload);
    frame
}

#[derive(Default)]
struct Outgoing {
    buf: Vec<u8>,
    pos: usize,
    fin: bool,
    finished: bool,

    ack: Option<oneshot::Sender<()>>,
}

#[derive(Default)]
struct Incoming {
    buf: Vec<u8>,
    fin: bool,
}

struct Exchange {
    key: u64,
    incoming: Incoming,

    data_remaining: u64,

    blocked: Option<Vec<u8>>,
    respond: Option<oneshot::Sender<Result<ResponseHead, Error>>>,
    body_tx: Option<mpsc::Sender<Result<Bytes, Error>>>,

    queued: VecDeque<Bytes>,
    got_final_headers: bool,
}

type Waiting = (
    u64,
    RequestHead,
    RequestBody,
    oneshot::Sender<Result<ResponseHead, Error>>,
);

struct Task {
    encoder: Option<qpack::Encoder>,

    encoder_out: StreamId,
    peer_settings: Arc<Mutex<Option<Vec<(u64, u64)>>>>,

    rtt_sink: Option<(Arc<crate::SessionStore>, String)>,
    last_rtt: Duration,

    connected: bool,

    early: Vec<(u64, RequestHead)>,
    confirmed: Arc<AtomicBool>,
    early_state: Arc<AtomicU8>,
    driver: Driver,
    commands: mpsc::UnboundedReceiver<Command>,
    command_tx: mpsc::UnboundedSender<Command>,
    alive: Arc<AtomicBool>,
    decoder: Decoder,
    decoder_stream: StreamId,
    sent_inserts: u64,
    out: HashMap<StreamId, Outgoing>,
    streams: HashMap<StreamId, Exchange>,
    by_key: HashMap<u64, StreamId>,
    peer_uni: HashMap<StreamId, (Option<u64>, Incoming)>,
    encoder_stream: Vec<u8>,

    waiting: VecDeque<Waiting>,
    goaway: bool,
}

impl Task {
    async fn run(mut self) {
        let result = self.main_loop().await;
        self.alive.store(false, Ordering::Release);
        let error = match result {
            Err(e) => e,
            Ok(()) => Error::Other("connection closed".into()),
        };
        tracing::debug!("HTTP/3 connection ended: {error}");
        let message = error.to_string();
        for (_, mut exchange) in self.streams.drain() {
            if let Some(respond) = exchange.respond.take() {
                let _ = respond.send(Err(Error::Other(message.clone())));
            }
            if let Some(tx) = exchange.body_tx.take() {
                let _ = tx.try_send(Err(Error::Other(message.clone())));
            }
        }
        for (_, _, _, respond) in self.waiting.drain(..) {
            let _ = respond.send(Err(Error::Other(message.clone())));
        }
    }

    async fn main_loop(&mut self) -> Result<(), Error> {
        loop {
            self.pump()?;
            self.driver.flush().await?;
            if self.driver.conn.is_drained() {
                return Ok(());
            }
            tokio::select! {
                biased;
                command = self.commands.recv() => match command {
                    Some(command) => self.handle(command),
                    None => {

                        if self.streams.is_empty() {
                            self.driver.conn.close(Instant::now(), VarInt::from_u32(H3_NO_ERROR), Bytes::new());
                            self.driver.flush().await?;
                            return Ok(());
                        }
                        time::sleep(Duration::from_millis(50)).await;
                    }
                },
                ready = self.driver.ready() => ready?,
            }
            while let Ok(command) = self.commands.try_recv() {
                self.handle(command);
            }
            if self.connected {
                if let Some((store, host)) = &self.rtt_sink {
                    let rtt = self.driver.conn.rtt();
                    if rtt.as_micros().abs_diff(self.last_rtt.as_micros()) * 10
                        > self.last_rtt.as_micros()
                    {
                        self.last_rtt = rtt;
                        store.set_rtt(host, rtt);
                    }
                }
            }
            for event in self.driver.process()? {
                match event {
                    Event::ConnectionLost { reason } => return Err(Error::Lost(reason)),
                    Event::Connected => {
                        self.connected = true;
                        self.confirmed.store(true, Ordering::Release);
                        if self.driver.conn.has_0rtt() {
                            let accepted = self.driver.conn.accepted_0rtt();
                            self.early_state
                                .store(if accepted { 2 } else { 3 }, Ordering::Release);
                            if !accepted {
                                self.replay_after_rejection()?;
                            }
                        }
                        self.early.clear();
                    }
                    _ => {}
                }
            }
        }
    }

    fn handle(&mut self, command: Command) {
        match command {
            Command::Request {
                key,
                head,
                body,
                respond,
            } => self.waiting.push_back((key, head, body, respond)),
            Command::BodyChunk { key, data, ack } => {
                let Some(&id) = self.by_key.get(&key) else {
                    return;
                };
                match data {
                    Some(Ok(chunk)) if !chunk.is_empty() => {
                        let mut frame = Vec::new();
                        put_varint(&mut frame, FRAME_DATA);
                        put_varint(&mut frame, chunk.len() as u64);
                        frame.extend_from_slice(&chunk);
                        let out = self.out.entry(id).or_default();
                        out.buf.extend_from_slice(&frame);
                        out.ack = Some(ack);
                    }
                    Some(Ok(_)) => {
                        let _ = ack.send(());
                    }
                    Some(Err(_)) => {
                        let _ = self
                            .driver
                            .conn
                            .send_stream(id)
                            .reset(VarInt::from_u32(H3_REQUEST_CANCELLED));
                        self.out.remove(&id);
                    }
                    None => {
                        self.out.entry(id).or_default().fin = true;
                        let _ = ack.send(());
                    }
                }
            }
            Command::Drained => {}
            Command::Cancel(key) => {
                if let Some(id) = self.by_key.remove(&key) {
                    if self.streams.remove(&id).is_some() {
                        let _ = self
                            .driver
                            .conn
                            .recv_stream(id)
                            .stop(VarInt::from_u32(H3_REQUEST_CANCELLED));
                        let _ = self
                            .driver
                            .conn
                            .send_stream(id)
                            .reset(VarInt::from_u32(H3_REQUEST_CANCELLED));
                    }
                    self.out.remove(&id);
                }
            }
        }
    }

    fn queue(&mut self, id: StreamId, data: &[u8], fin: bool) {
        let out = self.out.entry(id).or_default();
        out.buf.extend_from_slice(data);
        out.fin |= fin;
    }

    fn pump(&mut self) -> Result<(), Error> {
        let mut held = VecDeque::new();
        while let Some((key, head, body, respond)) = self.waiting.pop_front() {
            if self.goaway {
                let _ = respond.send(Err(Error::Other("the server is going away".into())));
                continue;
            }
            let safe = matches!(head.method.as_str(), "GET" | "HEAD" | "OPTIONS")
                && matches!(body, RequestBody::Empty);
            if !self.connected && !safe {
                held.push_back((key, head, body, respond));
                continue;
            }
            let Some(id) = self.driver.conn.streams().open(Dir::Bi) else {
                self.waiting.push_front((key, head, body, respond));
                break;
            };
            self.start_request(id, key, head, body, respond);
        }
        while let Some(item) = held.pop_back() {
            self.waiting.push_front(item);
        }

        let ids: Vec<StreamId> = self.out.keys().copied().collect();
        for id in ids {
            let out = self.out.get_mut(&id).unwrap();
            while out.pos < out.buf.len() {
                match self.driver.conn.send_stream(id).write(&out.buf[out.pos..]) {
                    Ok(n) => out.pos += n,
                    Err(WriteError::Blocked) => break,
                    Err(_) => {
                        out.buf.clear();
                        out.pos = 0;
                        break;
                    }
                }
            }
            if out.pos == out.buf.len() {
                out.buf.clear();
                out.pos = 0;
                if let Some(ack) = out.ack.take() {
                    let _ = ack.send(());
                }
                if out.fin && !out.finished {
                    out.finished = true;
                    let _ = self.driver.conn.send_stream(id).finish();
                }
            }
        }

        while let Some(id) = self.driver.conn.streams().accept(Dir::Uni) {
            self.peer_uni.insert(id, (None, Incoming::default()));
        }
        let uni_ids: Vec<StreamId> = self.peer_uni.keys().copied().collect();
        for id in uni_ids {
            let mut incoming = mem::take(&mut self.peer_uni.get_mut(&id).unwrap().1);
            read_stream(&mut self.driver, id, &mut incoming, READ_BUDGET)?;
            self.peer_uni.get_mut(&id).unwrap().1 = incoming;
            self.process_uni(id)?;
        }

        let ids: Vec<StreamId> = self.streams.keys().copied().collect();
        for id in ids {
            let mut exchange = self.streams.remove(&id).unwrap();
            self.progress(id, &mut exchange)?;
            let finished = exchange.incoming.fin
                && exchange.incoming.buf.is_empty()
                && exchange.data_remaining == 0
                && exchange.blocked.is_none()
                && exchange.queued.is_empty();
            if finished {
                self.by_key.remove(&exchange.key);
            } else {
                self.streams.insert(id, exchange);
            }
        }

        if self.decoder.inserted() > self.sent_inserts {
            let mut instruction = Vec::new();
            qpack::put_int(
                &mut instruction,
                0,
                6,
                self.decoder.inserted() - self.sent_inserts,
            );
            self.sent_inserts = self.decoder.inserted();
            let stream = self.decoder_stream;
            self.queue(stream, &instruction, false);
        }
        Ok(())
    }

    fn start_request(
        &mut self,
        id: StreamId,
        key: u64,
        head: RequestHead,
        body: RequestBody,
        respond: oneshot::Sender<Result<ResponseHead, Error>>,
    ) {
        if !self.connected {
            self.early.push((key, head.clone()));
        }
        let mut fields: Vec<(&[u8], &[u8])> = vec![
            (b":method", head.method.as_bytes()),
            (b":authority", head.authority.as_bytes()),
            (b":scheme", b"https"),
            (b":path", head.path.as_bytes()),
        ];
        for (name, value) in &head.headers {
            fields.push((name.as_bytes(), value.as_bytes()));
        }
        let block = match (self.connected, self.encoder.as_mut()) {
            (true, Some(encoder)) => {
                let block = encoder.encode_section(u64::from(id), &fields);
                let instructions = mem::take(&mut encoder.instructions);
                if env::var_os("H3_DEBUG").is_some() {
                    tracing::debug!(
                        "QPACK: section {} bytes, {} bytes of table inserts",
                        block.len(),
                        instructions.len()
                    );
                }
                if !instructions.is_empty() {
                    let stream = self.encoder_out;
                    self.queue(stream, &instructions, false);
                }
                block
            }
            _ => qpack::encode_section(&fields),
        };
        let mut frames = Vec::new();
        put_varint(&mut frames, FRAME_HEADERS);
        put_varint(&mut frames, block.len() as u64);
        frames.extend_from_slice(&block);
        match body {
            RequestBody::Empty => self.queue(id, &frames, true),
            RequestBody::Full(bytes) => {
                if !bytes.is_empty() {
                    put_varint(&mut frames, FRAME_DATA);
                    put_varint(&mut frames, bytes.len() as u64);
                    frames.extend_from_slice(&bytes);
                }
                self.queue(id, &frames, true);
            }
            RequestBody::Stream(mut stream) => {
                self.queue(id, &frames, false);
                let commands = self.command_tx.clone();
                tokio::spawn(async move {
                    loop {
                        let next = future::poll_fn(|cx| stream.as_mut().poll_next(cx)).await;
                        let (ack, acked) = oneshot::channel();
                        let last = !matches!(next, Some(Ok(_)));
                        if commands
                            .send(Command::BodyChunk {
                                key,
                                data: next,
                                ack,
                            })
                            .is_err()
                            || last
                        {
                            break;
                        }
                        if acked.await.is_err() {
                            break;
                        }
                    }
                });
            }
        }
        self.by_key.insert(key, id);
        self.streams.insert(
            id,
            Exchange {
                key,
                incoming: Incoming::default(),
                data_remaining: 0,
                blocked: None,
                respond: Some(respond),
                body_tx: None,
                queued: VecDeque::new(),
                got_final_headers: false,
            },
        );
    }

    fn replay_after_rejection(&mut self) -> Result<(), Error> {
        tracing::debug!(
            "0-RTT rejected, sending {} early request(s) again",
            self.early.len()
        );
        self.out.clear();
        self.by_key.clear();
        let settings = settings_frame(&mut rand::rng());
        for (kind, rest) in [
            (STREAM_CONTROL, settings.as_slice()),
            (STREAM_QPACK_ENCODER, &[][..]),
            (STREAM_QPACK_DECODER, &[][..]),
        ] {
            let id = self
                .driver
                .conn
                .streams()
                .open(Dir::Uni)
                .ok_or_else(|| Error::Other("no stream credit".into()))?;
            let mut data = Vec::new();
            put_varint(&mut data, kind);
            data.extend_from_slice(rest);
            self.queue(id, &data, false);
            if kind == STREAM_QPACK_DECODER {
                self.decoder_stream = id;
            }
            if kind == STREAM_QPACK_ENCODER {
                self.encoder_out = id;
            }
        }
        let mut old: HashMap<u64, Exchange> = self
            .streams
            .drain()
            .map(|(_, exchange)| (exchange.key, exchange))
            .collect();
        for (key, head) in mem::take(&mut self.early) {
            let Some(mut exchange) = old.remove(&key) else {
                continue;
            };
            let Some(respond) = exchange.respond.take() else {
                continue;
            };
            let id = self
                .driver
                .conn
                .streams()
                .open(Dir::Bi)
                .ok_or_else(|| Error::Other("no stream credit".into()))?;
            self.start_request(id, key, head, RequestBody::Empty, respond);
        }
        Ok(())
    }

    fn process_uni(&mut self, id: StreamId) -> Result<(), Error> {
        let (kind, incoming) = self.peer_uni.get_mut(&id).unwrap();
        if kind.is_none() {
            match get_varint(&incoming.buf) {
                Some((value, used)) => {
                    *kind = Some(value);
                    incoming.buf.drain(..used);
                }
                None => return Ok(()),
            }
        }
        match kind.unwrap() {
            STREAM_CONTROL => {
                while let Some((ty, used)) = get_varint(&incoming.buf) {
                    let Some((len, used_len)) = get_varint(&incoming.buf[used..]) else {
                        break;
                    };
                    let total = used + used_len + len as usize;
                    if incoming.buf.len() < total {
                        break;
                    }
                    if ty == FRAME_GOAWAY {
                        self.goaway = true;
                    }
                    if ty == 0x04 {
                        let mut payload = &incoming.buf[used + used_len..total];
                        let mut pairs = Vec::new();
                        while let (Some((id, a)), true) = (get_varint(payload), !payload.is_empty())
                        {
                            let Some((value, b)) = get_varint(&payload[a..]) else {
                                break;
                            };
                            pairs.push((id, value));
                            payload = &payload[a + b..];
                        }
                        let capacity = pairs
                            .iter()
                            .find(|(id, _)| *id == 0x01)
                            .map_or(0, |(_, value)| *value);
                        if capacity > 0 && self.encoder.is_none() {
                            self.encoder = Some(qpack::Encoder::new(capacity));
                        }
                        *self.peer_settings.lock().unwrap_or_else(|e| e.into_inner()) = Some(pairs);
                    }
                    incoming.buf.drain(..total);
                }
            }
            STREAM_QPACK_ENCODER => {
                self.encoder_stream.append(&mut incoming.buf);
                self.decoder
                    .encoder_stream(&mut self.encoder_stream)
                    .map_err(|e| Error::Other(format!("QPACK encoder stream: {e}")))?;
            }
            STREAM_QPACK_DECODER => match self.encoder.as_mut() {
                Some(encoder) => encoder
                    .decoder_stream(&mut incoming.buf)
                    .map_err(|e| Error::Other(format!("QPACK decoder stream: {e}")))?,
                None => incoming.buf.clear(),
            },
            _ => incoming.buf.clear(),
        }
        Ok(())
    }

    fn deliver(exchange: &mut Exchange) -> bool {
        let Some(tx) = &exchange.body_tx else {
            return true;
        };
        while let Some(chunk) = exchange.queued.pop_front() {
            match tx.try_send(Ok(chunk)) {
                Ok(()) => {}
                Err(mpsc::error::TrySendError::Full(Ok(chunk))) => {
                    exchange.queued.push_front(chunk);
                    break;
                }
                Err(_) => return false,
            }
        }
        true
    }

    fn progress(&mut self, id: StreamId, exchange: &mut Exchange) -> Result<(), Error> {
        if !Self::deliver(exchange) {
            exchange.queued.clear();
            let _ = self
                .driver
                .conn
                .recv_stream(id)
                .stop(VarInt::from_u32(H3_REQUEST_CANCELLED));
            exchange.incoming.fin = true;
            exchange.incoming.buf.clear();
            exchange.data_remaining = 0;
            exchange.blocked = None;
            return Ok(());
        }
        if exchange.queued.len() >= BODY_QUEUE
            || (exchange.body_tx.is_some() && !exchange.queued.is_empty())
        {
            return Ok(());
        }
        read_stream(&mut self.driver, id, &mut exchange.incoming, READ_BUDGET)?;

        if let Some(block) = exchange.blocked.take() {
            self.decode_headers(id, exchange, block)?;
        }
        loop {
            if exchange.blocked.is_some() {
                break;
            }
            if exchange.data_remaining > 0 {
                let take = (exchange.data_remaining as usize).min(exchange.incoming.buf.len());
                if take == 0 {
                    break;
                }
                let chunk: Vec<u8> = exchange.incoming.buf.drain(..take).collect();
                exchange.data_remaining -= take as u64;
                exchange.queued.push_back(Bytes::from(chunk));
                continue;
            }
            let Some((ty, used)) = get_varint(&exchange.incoming.buf) else {
                break;
            };
            let Some((len, used_len)) = get_varint(&exchange.incoming.buf[used..]) else {
                break;
            };
            let header = used + used_len;
            if ty == FRAME_DATA {
                exchange.incoming.buf.drain(..header);
                exchange.data_remaining = len;
                continue;
            }
            if exchange.incoming.buf.len() < header + len as usize {
                break;
            }
            let payload: Vec<u8> = exchange.incoming.buf[header..header + len as usize].to_vec();
            exchange.incoming.buf.drain(..header + len as usize);
            if ty == FRAME_HEADERS {
                self.decode_headers(id, exchange, payload)?;
            }
        }
        if !Self::deliver(exchange) {
            exchange.queued.clear();
        }

        if exchange.incoming.fin
            && exchange.incoming.buf.is_empty()
            && exchange.data_remaining == 0
            && exchange.blocked.is_none()
            && exchange.queued.is_empty()
        {
            exchange.body_tx = None;
            if let Some(respond) = exchange.respond.take() {
                let _ = respond.send(Err(Error::Other(
                    "the response ended without headers".into(),
                )));
            }
        }
        Ok(())
    }

    fn decode_headers(
        &mut self,
        id: StreamId,
        exchange: &mut Exchange,
        block: Vec<u8>,
    ) -> Result<(), Error> {
        let section = self
            .decoder
            .decode_section(&block)
            .map_err(|e| Error::Other(format!("QPACK: {e}")))?;
        let fields = match section {
            Section::Blocked(_) => {
                exchange.blocked = Some(block);
                return Ok(());
            }
            Section::Fields(fields) => fields,
        };

        let required = self.decoder.required_insert_count_of(&block).unwrap_or(0);
        if required > 0 {
            self.sent_inserts = self.sent_inserts.max(required);
            let mut instruction = Vec::new();
            qpack::put_int(&mut instruction, 0b1000_0000, 7, u64::from(id));
            let stream = self.decoder_stream;
            self.queue(stream, &instruction, false);
        }
        let mut status = None;
        let mut headers = Vec::new();
        for (name, value) in fields {
            let (name, value) = (
                String::from_utf8_lossy(&name).into_owned(),
                String::from_utf8_lossy(&value).into_owned(),
            );
            if name == ":status" {
                status = value.parse::<u16>().ok();
            } else if !name.starts_with(':') {
                headers.push((name, value));
            }
        }

        let Some(status) = status else { return Ok(()) };
        if status >= 200 && !exchange.got_final_headers {
            exchange.got_final_headers = true;
            let (tx, rx) = mpsc::channel(BODY_QUEUE);
            exchange.body_tx = Some(tx);
            if let Some(respond) = exchange.respond.take() {
                let body = ResponseBody {
                    rx,
                    wake: self.command_tx.clone(),
                    key: exchange.key,
                };
                let _ = respond.send(Ok(ResponseHead {
                    status,
                    headers,
                    body,
                }));
            }
        }
        Ok(())
    }
}

fn read_stream(
    driver: &mut Driver,
    id: StreamId,
    incoming: &mut Incoming,
    budget: usize,
) -> Result<(), Error> {
    if incoming.fin {
        return Ok(());
    }
    let mut stream = driver.conn.recv_stream(id);
    let mut chunks = match stream.read(true) {
        Ok(chunks) => chunks,
        Err(_) => return Ok(()),
    };
    let mut taken = 0;
    while taken < budget {
        match chunks.next(budget - taken) {
            Ok(Some(chunk)) => {
                taken += chunk.bytes.len();
                incoming.buf.extend_from_slice(&chunk.bytes);
            }
            Ok(None) => {
                incoming.fin = true;
                break;
            }
            Err(ReadError::Blocked) => break,
            Err(ReadError::Reset(code)) => {
                let _ = chunks.finalize();
                return Err(Error::Other(format!("stream reset by peer: {code}")));
            }
        }
    }
    let _ = chunks.finalize();
    Ok(())
}
