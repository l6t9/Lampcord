use std::collections::VecDeque;
use std::collections;
use std::sync::OnceLock;

use crate::qpack_tables::{HUFFMAN, STATIC_TABLE};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("truncated QPACK data")]
    Truncated,
    #[error("malformed QPACK data: {0}")]
    Malformed(&'static str),
    #[error("reference to a missing table entry")]
    BadReference,
}

pub type Field = (Vec<u8>, Vec<u8>);

pub fn put_int(out: &mut Vec<u8>, first: u8, n: u8, mut value: u64) {
    let max = (1u64 << n) - 1;
    if value < max {
        out.push(first | value as u8);
        return;
    }
    out.push(first | max as u8);
    value -= max;
    while value >= 128 {
        out.push((value & 0x7f) as u8 | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

pub fn get_int(data: &[u8], n: u8) -> Result<(u64, usize), Error> {
    let max = (1u64 << n) - 1;
    let first = *data.first().ok_or(Error::Truncated)? as u64 & max;
    if first < max {
        return Ok((first, 1));
    }
    let mut value = max;
    let mut shift = 0;
    for (i, byte) in data[1..].iter().enumerate() {
        if shift > 56 {
            return Err(Error::Malformed("integer too large"));
        }
        value += ((byte & 0x7f) as u64) << shift;
        shift += 7;
        if byte & 0x80 == 0 {
            return Ok((value, i + 2));
        }
    }
    Err(Error::Truncated)
}

pub fn huffman_encode(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut acc: u64 = 0;
    let mut bits = 0u32;
    for &byte in input {
        let (code, len) = HUFFMAN[byte as usize];
        acc = (acc << len) | code as u64;
        bits += len as u32;
        while bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    if bits > 0 {
        out.push(((acc << (8 - bits)) | ((1 << (8 - bits)) - 1)) as u8);
    }
    out
}

pub fn huffman_decode(input: &[u8]) -> Result<Vec<u8>, Error> {
    struct Tree {
        children: Vec<[u32; 2]>,
        leaf: Vec<i32>,
    }
    static TREE: OnceLock<Tree> = OnceLock::new();
    let tree = TREE.get_or_init(|| {
        let mut tree = Tree {
            children: vec![[0, 0]],
            leaf: vec![-1],
        };
        for (symbol, &(code, len)) in HUFFMAN.iter().enumerate() {
            let mut node = 0usize;
            for i in (0..len).rev() {
                let bit = ((code >> i) & 1) as usize;
                if tree.children[node][bit] == 0 {
                    tree.children.push([0, 0]);
                    tree.leaf.push(-1);
                    let id = tree.children.len() as u32 - 1;
                    tree.children[node][bit] = id;
                }
                node = tree.children[node][bit] as usize;
            }
            tree.leaf[node] = symbol as i32;
        }
        tree
    });
    let mut out = Vec::with_capacity(input.len() * 2);
    let mut node = 0usize;
    let mut padding_bits = 0;
    let mut padding_all_ones = true;
    for &byte in input {
        for i in (0..8).rev() {
            let bit = ((byte >> i) & 1) as usize;
            node = tree.children[node][bit] as usize;
            if node == 0 {
                return Err(Error::Malformed("invalid Huffman code"));
            }
            padding_bits += 1;
            padding_all_ones &= bit == 1;
            match tree.leaf[node] {
                -1 => {}
                256 => return Err(Error::Malformed("EOS in a string")),
                symbol => {
                    out.push(symbol as u8);
                    node = 0;
                    padding_bits = 0;
                    padding_all_ones = true;
                }
            }
        }
    }
    if node != 0 && (padding_bits > 7 || !padding_all_ones) {
        return Err(Error::Malformed("invalid Huffman padding"));
    }
    Ok(out)
}

fn put_string(out: &mut Vec<u8>, first: u8, n: u8, value: &[u8]) {
    let huffman = huffman_encode(value);
    let (flag, bytes) = if huffman.len() < value.len() {
        (1u8 << (n - 1), &huffman[..])
    } else {
        (0, value)
    };
    put_int(out, first | flag, n - 1, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

fn get_string(data: &[u8], n: u8) -> Result<(Vec<u8>, usize), Error> {
    let first = *data.first().ok_or(Error::Truncated)?;
    let huffman = first & (1 << (n - 1)) != 0;
    let (len, used) = get_int(data, n - 1)?;
    let end = used
        .checked_add(len as usize)
        .ok_or(Error::Malformed("length"))?;
    let bytes = data.get(used..end).ok_or(Error::Truncated)?;
    Ok((
        if huffman {
            huffman_decode(bytes)?
        } else {
            bytes.to_vec()
        },
        end,
    ))
}

pub fn encode_section(fields: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut out = vec![0, 0];
    for &(name, value) in fields {
        let exact = STATIC_TABLE
            .iter()
            .position(|(n, v)| *n == name && *v == value);
        if let Some(index) = exact {
            put_int(&mut out, 0b1100_0000, 6, index as u64);
            continue;
        }
        if let Some(index) = STATIC_TABLE.iter().position(|(n, _)| *n == name) {
            put_int(&mut out, 0b0101_0000, 4, index as u64);
            put_string(&mut out, 0, 8, value);
        } else {
            put_string(&mut out, 0b0010_0000, 4, name);
            put_string(&mut out, 0, 8, value);
        }
    }
    out
}

fn never_index(name: &[u8], value: &[u8]) -> bool {
    matches!(
        name,
        b"cookie"
            | b"set-cookie"
            | b"authorization"
            | b"proxy-authorization"
            | b"content-length"
            | b"content-range"
            | b"range"
            | b"if-range"
            | b"if-none-match"
            | b"if-modified-since"
            | b"date"
            | b":path"
    ) || value.len() > 200
        || (!value.is_empty() && value.iter().all(|b| b.is_ascii_digit()))
}

pub struct Encoder {
    peer_max_capacity: u64,

    capacity: u64,
    size: u64,
    table: Vec<Field>,

    known_received: u64,

    in_flight: collections::HashMap<u64, u64>,

    pub instructions: Vec<u8>,
}

const ENCODER_TABLE_LIMIT: u64 = 16 * 1024;

impl Encoder {
    pub fn new(peer_max_capacity: u64) -> Self {
        Self {
            peer_max_capacity,
            capacity: 0,
            size: 0,
            table: Vec::new(),
            known_received: 0,
            in_flight: Default::default(),
            instructions: Vec::new(),
        }
    }

    pub fn encode_section(&mut self, stream: u64, fields: &[(&[u8], &[u8])]) -> Vec<u8> {
        enum Line {
            Raw(Vec<u8>),

            Dynamic(u64),
        }
        let mut lines = Vec::new();
        let mut required = 0u64;
        let limit = self.peer_max_capacity.min(ENCODER_TABLE_LIMIT);
        for &(name, value) in fields {
            if let Some(index) = STATIC_TABLE
                .iter()
                .position(|(n, v)| *n == name && *v == value)
            {
                let mut raw = Vec::new();
                put_int(&mut raw, 0b1100_0000, 6, index as u64);
                lines.push(Line::Raw(raw));
                continue;
            }

            let exact = self
                .table
                .iter()
                .position(|(n, v)| n == name && v == value)
                .filter(|&abs| (abs as u64) < self.known_received);
            if let Some(abs) = exact {
                required = required.max(abs as u64 + 1);
                lines.push(Line::Dynamic(abs as u64));
                continue;
            }

            let entry_size = (name.len() + value.len() + 32) as u64;
            if limit > 0
                && !never_index(name, value)
                && self.size + entry_size <= limit
                && !self.table.iter().any(|(n, v)| n == name && v == value)
            {
                if self.capacity < limit {
                    self.capacity = limit;
                    put_int(&mut self.instructions, 0b0010_0000, 5, limit);
                }
                if let Some(index) = STATIC_TABLE.iter().position(|(n, _)| *n == name) {
                    put_int(&mut self.instructions, 0b1100_0000, 6, index as u64);
                    put_string(&mut self.instructions, 0, 8, value);
                } else if let Some(abs) = self
                    .table
                    .iter()
                    .position(|(n, _)| n == name)
                    .filter(|&abs| (abs as u64) < self.known_received)
                {
                    put_int(
                        &mut self.instructions,
                        0b1000_0000,
                        6,
                        (self.table.len() - 1 - abs) as u64,
                    );
                    put_string(&mut self.instructions, 0, 8, value);
                } else {
                    put_string(&mut self.instructions, 0b0100_0000, 6, name);
                    put_string(&mut self.instructions, 0, 8, value);
                }
                self.table.push((name.to_vec(), value.to_vec()));
                self.size += entry_size;
            }

            let mut raw = Vec::new();
            if let Some(index) = STATIC_TABLE.iter().position(|(n, _)| *n == name) {
                put_int(&mut raw, 0b0101_0000, 4, index as u64);
                put_string(&mut raw, 0, 8, value);
            } else {
                put_string(&mut raw, 0b0010_0000, 4, name);
                put_string(&mut raw, 0, 8, value);
            }
            lines.push(Line::Raw(raw));
        }

        let mut body = Vec::new();
        for line in lines {
            match line {
                Line::Raw(raw) => body.extend_from_slice(&raw),
                Line::Dynamic(abs) => put_int(&mut body, 0b1000_0000, 6, required - 1 - abs),
            }
        }
        let mut out = Vec::with_capacity(body.len() + 4);
        if required == 0 {
            out.extend_from_slice(&[0, 0]);
        } else {
            let max_entries = self.peer_max_capacity / 32;
            put_int(&mut out, 0, 8, required % (2 * max_entries) + 1);
            out.push(0);
            self.in_flight.insert(stream, required);
        }
        out.extend_from_slice(&body);
        out
    }

    pub fn decoder_stream(&mut self, buffer: &mut Vec<u8>) -> Result<(), Error> {
        let mut pos = 0;
        while pos < buffer.len() {
            let first = buffer[pos];
            let (value, used) = match get_int(&buffer[pos..], if first & 0x80 != 0 { 7 } else { 6 })
            {
                Ok(x) => x,
                Err(Error::Truncated) => break,
                Err(e) => return Err(e),
            };
            if first & 0x80 != 0 {
                if let Some(required) = self.in_flight.remove(&value) {
                    self.known_received = self.known_received.max(required);
                }
            } else if first & 0x40 != 0 {
                self.in_flight.remove(&value);
            } else {
                self.known_received = self
                    .known_received
                    .saturating_add(value)
                    .min(self.table.len() as u64);
            }
            pos += used;
        }
        buffer.drain(..pos);
        Ok(())
    }
}

pub struct Decoder {
    max_capacity: u64,
    capacity: u64,
    size: u64,
    table: VecDeque<Field>,

    inserted: u64,
}

pub enum Section {
    Fields(Vec<Field>),

    Blocked(u64),
}

impl Decoder {
    pub fn new(max_capacity: u64) -> Self {
        Self {
            max_capacity,
            capacity: 0,
            size: 0,
            table: VecDeque::new(),
            inserted: 0,
        }
    }

    pub fn inserted(&self) -> u64 {
        self.inserted
    }

    fn evict_to(&mut self, limit: u64) {
        while self.size > limit {
            match self.table.pop_front() {
                Some((n, v)) => self.size -= (n.len() + v.len() + 32) as u64,
                None => break,
            }
        }
    }

    fn insert(&mut self, name: Vec<u8>, value: Vec<u8>) -> Result<(), Error> {
        let entry_size = (name.len() + value.len() + 32) as u64;
        if entry_size > self.capacity {
            return Err(Error::Malformed("entry larger than the table"));
        }
        self.evict_to(self.capacity - entry_size);
        self.size += entry_size;
        self.table.push_back((name, value));
        self.inserted += 1;
        Ok(())
    }

    fn dynamic(&self, absolute: u64) -> Result<&Field, Error> {
        let oldest = self.inserted - self.table.len() as u64;
        if absolute < oldest || absolute >= self.inserted {
            return Err(Error::BadReference);
        }
        Ok(&self.table[(absolute - oldest) as usize])
    }

    pub fn encoder_stream(&mut self, buffer: &mut Vec<u8>) -> Result<u64, Error> {
        let before = self.inserted;
        let mut pos = 0;
        loop {
            let data = &buffer[pos..];
            let Some(&first) = data.first() else { break };
            let parsed: Result<usize, Error> = (|| {
                if first & 0b1000_0000 != 0 {
                    let (index, used) = get_int(data, 6)?;
                    let (value, used_value) = get_string(&data[used..], 8)?;
                    let name = if first & 0b0100_0000 != 0 {
                        STATIC_TABLE
                            .get(index as usize)
                            .ok_or(Error::BadReference)?
                            .0
                            .to_vec()
                    } else {
                        let absolute = self
                            .inserted
                            .checked_sub(1 + index)
                            .ok_or(Error::BadReference)?;
                        self.dynamic(absolute)?.0.clone()
                    };
                    self.insert(name, value)?;
                    Ok(used + used_value)
                } else if first & 0b0100_0000 != 0 {
                    let (name, used) = get_string(data, 6)?;
                    let (value, used_value) = get_string(&data[used..], 8)?;
                    self.insert(name, value)?;
                    Ok(used + used_value)
                } else if first & 0b0010_0000 != 0 {
                    let (capacity, used) = get_int(data, 5)?;
                    if capacity > self.max_capacity {
                        return Err(Error::Malformed("capacity above the advertised maximum"));
                    }
                    self.capacity = capacity;
                    self.evict_to(capacity);
                    Ok(used)
                } else {
                    let (index, used) = get_int(data, 5)?;
                    let absolute = self
                        .inserted
                        .checked_sub(1 + index)
                        .ok_or(Error::BadReference)?;
                    let (name, value) = self.dynamic(absolute)?.clone();
                    self.insert(name, value)?;
                    Ok(used)
                }
            })();
            match parsed {
                Ok(used) => pos += used,
                Err(Error::Truncated) => break,
                Err(e) => return Err(e),
            }
        }
        buffer.drain(..pos);
        Ok(self.inserted - before)
    }

    pub fn decode_section(&self, data: &[u8]) -> Result<Section, Error> {
        let (encoded_ric, mut pos) = get_int(data, 8)?;
        let ric = self.required_insert_count(encoded_ric)?;
        if ric > self.inserted {
            return Ok(Section::Blocked(ric));
        }
        let sign = *data.get(pos).ok_or(Error::Truncated)? & 0x80 != 0;
        let (delta, used) = get_int(&data[pos..], 7)?;
        pos += used;
        let base = if sign {
            ric.checked_sub(delta + 1).ok_or(Error::BadReference)?
        } else {
            ric + delta
        };
        let mut fields = Vec::new();
        while pos < data.len() {
            let first = data[pos];
            let rest = &data[pos..];
            let used;
            if first & 0b1000_0000 != 0 {
                let (index, n) = get_int(rest, 6)?;
                used = n;
                if first & 0b0100_0000 != 0 {
                    let (name, value) = STATIC_TABLE
                        .get(index as usize)
                        .ok_or(Error::BadReference)?;
                    fields.push((name.to_vec(), value.to_vec()));
                } else {
                    let absolute = base.checked_sub(1 + index).ok_or(Error::BadReference)?;
                    fields.push(self.dynamic(absolute)?.clone());
                }
            } else if first & 0b0100_0000 != 0 {
                let (index, n) = get_int(rest, 4)?;
                let (value, m) = get_string(&rest[n..], 8)?;
                used = n + m;
                let name = if first & 0b0001_0000 != 0 {
                    STATIC_TABLE
                        .get(index as usize)
                        .ok_or(Error::BadReference)?
                        .0
                        .to_vec()
                } else {
                    let absolute = base.checked_sub(1 + index).ok_or(Error::BadReference)?;
                    self.dynamic(absolute)?.0.clone()
                };
                fields.push((name, value));
            } else if first & 0b0010_0000 != 0 {
                let (name, n) = get_string(rest, 4)?;
                let (value, m) = get_string(&rest[n..], 8)?;
                used = n + m;
                fields.push((name, value));
            } else if first & 0b0001_0000 != 0 {
                let (index, n) = get_int(rest, 4)?;
                used = n;
                fields.push(self.dynamic(base + index)?.clone());
            } else {
                let (index, n) = get_int(rest, 3)?;
                let (value, m) = get_string(&rest[n..], 8)?;
                used = n + m;
                fields.push((self.dynamic(base + index)?.0.clone(), value));
            }
            pos += used;
        }
        Ok(Section::Fields(fields))
    }

    pub fn required_insert_count_of(&self, data: &[u8]) -> Result<u64, Error> {
        let (encoded, _) = get_int(data, 8)?;
        self.required_insert_count(encoded)
    }

    fn required_insert_count(&self, encoded: u64) -> Result<u64, Error> {
        if encoded == 0 {
            return Ok(0);
        }
        let max_entries = self.max_capacity / 32;
        let full_range = 2 * max_entries;
        if encoded > full_range {
            return Err(Error::Malformed("encoded insert count out of range"));
        }
        let max_value = self.inserted + max_entries;
        let max_wrapped = (max_value / full_range) * full_range;
        let mut ric = max_wrapped + encoded - 1;
        if ric > max_value {
            if ric <= full_range {
                return Err(Error::Malformed("insert count out of range"));
            }
            ric -= full_range;
        }
        if ric == 0 {
            return Err(Error::Malformed("insert count of zero"));
        }
        Ok(ric)
    }
}

#[cfg(test)]
mod tests {
    use std::mem;
    use super::*;

    #[test]
    fn integers_follow_the_rfc_examples() {
        let mut out = Vec::new();
        put_int(&mut out, 0b1110_0000, 5, 10);
        assert_eq!(out, [0b1110_1010]);
        out.clear();
        put_int(&mut out, 0, 5, 1337);
        assert_eq!(out, [0x1f, 0x9a, 0x0a]);
        assert_eq!(get_int(&out, 5), Ok((1337, 3)));
    }

    #[test]
    fn huffman_matches_the_rfc_example() {
        let encoded = huffman_encode(b"www.example.com");
        assert_eq!(
            encoded,
            [
                0xf1, 0xe3, 0xc2, 0xe5, 0xf2, 0x3a, 0x6b, 0xa0, 0xab, 0x90, 0xf4, 0xff
            ]
        );
        assert_eq!(huffman_decode(&encoded).unwrap(), b"www.example.com");
        let all: Vec<u8> = (0..=255).collect();
        assert_eq!(huffman_decode(&huffman_encode(&all)).unwrap(), all);
    }

    #[test]
    fn a_static_only_section_round_trips() {
        let fields: Vec<(&[u8], &[u8])> = vec![
            (b":method", b"GET"),
            (b":authority", b"discord.com"),
            (b":scheme", b"https"),
            (b":path", b"/api/v9/users/@me"),
            (b"accept-encoding", b"gzip, deflate, br, zstd"),
            (b"x-super-properties", b"eyJvcyI6IldpbmRvd3MifQ=="),
        ];
        let encoded = encode_section(&fields);
        let decoder = Decoder::new(65536);
        let Section::Fields(decoded) = decoder.decode_section(&encoded).unwrap() else {
            panic!("blocked")
        };
        let expected: Vec<Field> = fields
            .iter()
            .map(|(n, v)| (n.to_vec(), v.to_vec()))
            .collect();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn the_encoder_inserts_then_references_only_what_the_peer_acknowledged() {
        let fields: Vec<(&[u8], &[u8])> = vec![
            (b":method", b"POST"),
            (b"x-goog-api-client", b"gl-rust/1.0"),
            (b"content-length", b"1234"),
        ];
        let mut encoder = Encoder::new(4096);
        let mut decoder = Decoder::new(4096);

        let first = encoder.encode_section(0, &fields);
        assert_eq!(&first[..2], &[0, 0], "no dynamic references yet");
        let mut instructions = mem::take(&mut encoder.instructions);
        assert!(!instructions.is_empty());
        assert_eq!(
            decoder.encoder_stream(&mut instructions).unwrap(),
            1,
            "only the indexable field was inserted"
        );
        let Section::Fields(decoded) = decoder.decode_section(&first).unwrap() else {
            panic!("blocked")
        };
        assert_eq!(decoded.len(), 3);

        let mut ack = Vec::new();
        put_int(&mut ack, 0, 6, 1);
        encoder.decoder_stream(&mut ack).unwrap();
        let second = encoder.encode_section(4, &fields);
        assert_ne!(&second[..2], &[0, 0], "now it uses the table");
        assert!(second.len() < first.len());
        let Section::Fields(decoded) = decoder.decode_section(&second).unwrap() else {
            panic!("blocked")
        };
        let expected: Vec<Field> = fields
            .iter()
            .map(|(n, v)| (n.to_vec(), v.to_vec()))
            .collect();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn dynamic_entries_inserted_by_the_peer_are_usable_and_block_until_then() {
        let mut decoder = Decoder::new(4096);

        let section = [0x02, 0x00, 0b1000_0000];
        assert!(matches!(
            decoder.decode_section(&section).unwrap(),
            Section::Blocked(1)
        ));

        let mut stream = Vec::new();
        put_int(&mut stream, 0b0010_0000, 5, 220);
        put_string(&mut stream, 0b0100_0000, 6, b"custom-key");
        put_string(&mut stream, 0, 8, b"custom-value");
        assert_eq!(decoder.encoder_stream(&mut stream).unwrap(), 1);
        assert!(stream.is_empty());
        let Section::Fields(fields) = decoder.decode_section(&section).unwrap() else {
            panic!("still blocked")
        };
        assert_eq!(
            fields,
            vec![(b"custom-key".to_vec(), b"custom-value".to_vec())]
        );
    }
}
