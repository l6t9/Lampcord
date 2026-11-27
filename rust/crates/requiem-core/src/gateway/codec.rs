use serde_json::Value;
use std::io;
use std::mem;
use wreq::ws::message::Message as WsMessage;
use zstd::stream::raw::InBuffer;
use zstd::stream::raw::Operation;
use zstd::stream::raw::OutBuffer;
use zstd::stream::raw;

use super::etf;

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("zstd: {0}")]
    Zstd(io::Error),
    #[error("zlib: {0}")]
    Zlib(flate2::DecompressError),
    #[error("etf: {0}")]
    Etf(#[from] etf::EtfError),
    #[error("frame is not UTF-8 JSON")]
    NotText,
}

pub struct Codec {
    etf: bool,
    compression: Compression,
}

enum Compression {
    None,
    Zstd(raw::Decoder<'static>),

    Zlib(Box<flate2::Decompress>, Vec<u8>),
}

const ZLIB_SUFFIX: [u8; 4] = [0x00, 0x00, 0xff, 0xff];

impl Codec {
    pub fn for_url(url: &str) -> Self {
        let query = url.split_once('?').map_or("", |(_, q)| q);
        let param = |name: &str| {
            query
                .split('&')
                .find_map(|pair| pair.strip_prefix(name)?.strip_prefix('='))
        };
        let compression = match param("compress") {
            Some("zstd-stream") => {
                Compression::Zstd(raw::Decoder::new().expect("zstd decoder"))
            }
            Some("zlib-stream") => {
                Compression::Zlib(Box::new(flate2::Decompress::new(true)), Vec::new())
            }
            _ => Compression::None,
        };
        Self {
            etf: param("encoding") == Some("etf"),
            compression,
        }
    }

    pub fn encode(&self, payload: &Value) -> WsMessage {
        if self.etf {
            WsMessage::binary(etf::encode(payload))
        } else {
            WsMessage::text(payload.to_string())
        }
    }

    pub fn decode(&mut self, message: WsMessage) -> Result<Option<String>, CodecError> {
        match message {
            WsMessage::Text(text) => Ok(Some(text.as_str().to_string())),
            WsMessage::Binary(bytes) => {
                let data = match &mut self.compression {
                    Compression::Zstd(decoder) => {
                        decompress(decoder, &bytes).map_err(CodecError::Zstd)?
                    }
                    Compression::Zlib(inflater, pending) => {
                        pending.extend_from_slice(&bytes);
                        if !pending.ends_with(&ZLIB_SUFFIX) {
                            return Ok(None);
                        }
                        let input = mem::take(pending);
                        inflate(inflater, &input).map_err(CodecError::Zlib)?
                    }
                    Compression::None => bytes.to_vec(),
                };
                if self.etf {
                    Ok(Some(etf::to_json(&data)?))
                } else {
                    String::from_utf8(data)
                        .map(Some)
                        .map_err(|_| CodecError::NotText)
                }
            }
            _ => Ok(None),
        }
    }
}

fn inflate(
    inflater: &mut flate2::Decompress,
    input: &[u8],
) -> Result<Vec<u8>, flate2::DecompressError> {
    let mut out = Vec::with_capacity(input.len() * 4);
    let start = inflater.total_in();
    loop {
        let consumed = (inflater.total_in() - start) as usize;
        if consumed >= input.len() && out.capacity() > out.len() {
            return Ok(out);
        }
        out.reserve(16 * 1024);
        inflater.decompress_vec(&input[consumed..], &mut out, flate2::FlushDecompress::Sync)?;
        let consumed = (inflater.total_in() - start) as usize;
        if consumed >= input.len() && out.capacity() > out.len() {
            return Ok(out);
        }
    }
}

fn decompress(
    decoder: &mut raw::Decoder<'static>,
    input: &[u8],
) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(input.len() * 4);
    let mut source = InBuffer::around(input);
    let mut chunk = vec![0u8; 64 * 1024];
    loop {
        let mut sink = OutBuffer::around(&mut chunk[..]);
        decoder.run(&mut source, &mut sink)?;
        let produced = sink.pos();
        out.extend_from_slice(&chunk[..produced]);

        if source.pos() >= input.len() && produced < chunk.len() {
            return Ok(out);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use super::*;
    use serde_json::json;

    #[test]
    fn the_url_chooses_the_wire_format() {
        assert!(
            Codec::for_url("wss://gateway.discord.gg/?encoding=etf&v=9&compress=zstd-stream").etf
        );
        assert!(matches!(
            Codec::for_url("wss://gateway.discord.gg/?encoding=etf&v=9&compress=zstd-stream")
                .compression,
            Compression::Zstd(_)
        ));
        let plain = Codec::for_url("wss://example.org/?v=9&encoding=json");
        assert!(!plain.etf && matches!(plain.compression, Compression::None));
        assert!(matches!(
            Codec::for_url("wss://example.org/?encoding=json&v=9&compress=zlib-stream").compression,
            Compression::Zlib(..)
        ));
        assert!(!Codec::for_url("wss://example.org").etf);
    }

    #[test]
    fn json_frames_are_text_and_etf_frames_are_binary() {
        let payload = json!({ "op": 1, "d": 5 });
        assert!(matches!(
            Codec::for_url("wss://x/?encoding=json").encode(&payload),
            WsMessage::Text(_)
        ));
        assert!(matches!(
            Codec::for_url("wss://x/?encoding=etf").encode(&payload),
            WsMessage::Binary(_)
        ));
    }

    #[test]
    fn a_zstd_stream_of_etf_frames_decodes_frame_by_frame() {
        let mut encoder = zstd::stream::write::Encoder::new(Vec::new(), 3).unwrap();
        let mut frames = Vec::new();
        for payload in [
            json!({ "op": 10, "d": { "heartbeat_interval": 41250 } }),
            json!({ "op": 0, "t": "READY", "s": 1, "d": { "session_id": "abc", "users": [] } }),
        ] {
            encoder.write_all(&etf::encode(&payload)).unwrap();
            encoder.flush().unwrap();
            frames.push(mem::take(encoder.get_mut()));
        }
        let mut codec = Codec::for_url("wss://x/?encoding=etf&compress=zstd-stream");
        for (frame, expected) in frames.into_iter().zip([
            r#"{"op":10,"d":{"heartbeat_interval":41250}}"#,
            r#"{"op":0,"t":"READY","s":1,"d":{"session_id":"abc","users":[]}}"#,
        ]) {
            let text = codec.decode(WsMessage::binary(frame)).unwrap().unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(&text).unwrap(),
                serde_json::from_str::<Value>(expected).unwrap()
            );
        }
    }

    #[test]
    fn a_zlib_stream_decodes_message_by_message_even_when_a_message_arrives_in_pieces() {
        let mut compressor = flate2::Compress::new(flate2::Compression::default(), true);
        let mut messages = Vec::new();
        for payload in [
            json!({ "op": 10, "d": { "heartbeat_interval": 41250 } }),
            json!({ "op": 0, "t": "READY", "s": 1, "d": { "session_id": "abc", "padding": "x".repeat(5000) } }),
        ] {
            let input = payload.to_string();
            let mut out = Vec::with_capacity(input.len() + 64);
            compressor
                .compress_vec(input.as_bytes(), &mut out, flate2::FlushCompress::Sync)
                .unwrap();
            assert!(out.ends_with(&ZLIB_SUFFIX));
            messages.push((out, payload));
        }
        let mut codec = Codec::for_url("wss://x/?encoding=json&compress=zlib-stream");
        let (first, first_payload) = &messages[0];
        let text = codec
            .decode(WsMessage::binary(first.clone()))
            .unwrap()
            .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&text).unwrap(),
            *first_payload
        );

        let (second, second_payload) = &messages[1];
        let cut = second.len() / 2;
        assert!(
            codec
                .decode(WsMessage::binary(second[..cut].to_vec()))
                .unwrap()
                .is_none()
        );
        let text = codec
            .decode(WsMessage::binary(second[cut..].to_vec()))
            .unwrap()
            .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&text).unwrap(),
            *second_payload
        );
    }

    #[test]
    fn json_text_frames_pass_through() {
        let mut codec = Codec::for_url("wss://x/?encoding=json");
        assert_eq!(
            codec
                .decode(WsMessage::text(r#"{"op":11}"#))
                .unwrap()
                .as_deref(),
            Some(r#"{"op":11}"#)
        );
    }
}
