use std::fmt::Write as _;

use serde_json::Value;

const VERSION: u8 = 131;
const NEW_FLOAT_EXT: u8 = 70;
const SMALL_INTEGER_EXT: u8 = 97;
const INTEGER_EXT: u8 = 98;
const FLOAT_EXT: u8 = 99;
const ATOM_EXT: u8 = 100;
const SMALL_TUPLE_EXT: u8 = 104;
const LARGE_TUPLE_EXT: u8 = 105;
const NIL_EXT: u8 = 106;
const STRING_EXT: u8 = 107;
const LIST_EXT: u8 = 108;
const BINARY_EXT: u8 = 109;
const SMALL_BIG_EXT: u8 = 110;
const LARGE_BIG_EXT: u8 = 111;
const SMALL_ATOM_EXT: u8 = 115;
const MAP_EXT: u8 = 116;
const ATOM_UTF8_EXT: u8 = 118;
const SMALL_ATOM_UTF8_EXT: u8 = 119;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EtfError {
    #[error("not ETF (missing version byte)")]
    NotEtf,
    #[error("ETF data ends early")]
    Truncated,
    #[error("unsupported ETF tag {0}")]
    Unsupported(u8),
    #[error("ETF nests too deeply")]
    TooDeep,
}

const MAX_DEPTH: usize = 512;

pub fn to_json(data: &[u8]) -> Result<String, EtfError> {
    if data.first() != Some(&VERSION) {
        return Err(EtfError::NotEtf);
    }
    let mut reader = Reader { data, at: 1 };
    let mut out = String::with_capacity(data.len() + data.len() / 4);
    reader.term(&mut out, 0)?;
    Ok(out)
}

struct Reader<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], EtfError> {
        let end = self
            .at
            .checked_add(n)
            .filter(|end| *end <= self.data.len())
            .ok_or(EtfError::Truncated)?;
        let slice = &self.data[self.at..end];
        self.at = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, EtfError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<usize, EtfError> {
        let b = self.take(2)?;
        Ok(u16::from_be_bytes([b[0], b[1]]) as usize)
    }

    fn u32(&mut self) -> Result<usize, EtfError> {
        let b = self.take(4)?;
        Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize)
    }

    fn term(&mut self, out: &mut String, depth: usize) -> Result<(), EtfError> {
        if depth > MAX_DEPTH {
            return Err(EtfError::TooDeep);
        }
        let tag = self.u8()?;
        match tag {
            SMALL_INTEGER_EXT => {
                let _ = write!(out, "{}", self.u8()?);
            }
            INTEGER_EXT => {
                let b = self.take(4)?;
                let _ = write!(out, "{}", i32::from_be_bytes([b[0], b[1], b[2], b[3]]));
            }
            NEW_FLOAT_EXT => {
                let b = self.take(8)?;
                float(
                    out,
                    f64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]),
                );
            }
            FLOAT_EXT => {
                let text = self.take(31)?;
                let text = String::from_utf8_lossy(text);
                float(
                    out,
                    text.trim_end_matches('\0')
                        .trim()
                        .parse::<f64>()
                        .unwrap_or(0.0),
                );
            }
            ATOM_EXT | ATOM_UTF8_EXT => {
                let n = self.u16()?;
                let bytes = self.take(n)?;
                atom(out, bytes);
            }
            SMALL_ATOM_EXT | SMALL_ATOM_UTF8_EXT => {
                let n = self.u8()? as usize;
                let bytes = self.take(n)?;
                atom(out, bytes);
            }
            NIL_EXT => out.push_str("[]"),
            STRING_EXT => {
                let n = self.u16()?;
                out.push('[');
                for (i, byte) in self.take(n)?.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    let _ = write!(out, "{byte}");
                }
                out.push(']');
            }
            LIST_EXT => {
                let n = self.u32()?;
                out.push('[');
                for i in 0..n {
                    if i > 0 {
                        out.push(',');
                    }
                    self.term(out, depth + 1)?;
                }

                let mut sink = String::new();
                self.term(&mut sink, depth + 1)?;
                out.push(']');
            }
            SMALL_TUPLE_EXT | LARGE_TUPLE_EXT => {
                let n = if tag == SMALL_TUPLE_EXT {
                    self.u8()? as usize
                } else {
                    self.u32()?
                };
                out.push('[');
                for i in 0..n {
                    if i > 0 {
                        out.push(',');
                    }
                    self.term(out, depth + 1)?;
                }
                out.push(']');
            }
            BINARY_EXT => {
                let n = self.u32()?;
                string(out, &String::from_utf8_lossy(self.take(n)?));
            }
            SMALL_BIG_EXT | LARGE_BIG_EXT => {
                let n = if tag == SMALL_BIG_EXT {
                    self.u8()? as usize
                } else {
                    self.u32()?
                };
                let negative = self.u8()? == 1;
                let digits = self.take(n)?;
                big(out, negative, digits);
            }
            MAP_EXT => {
                let n = self.u32()?;
                out.push('{');
                for i in 0..n {
                    if i > 0 {
                        out.push(',');
                    }
                    let mut key = String::new();
                    self.term(&mut key, depth + 1)?;

                    if key.starts_with('"') {
                        out.push_str(&key);
                    } else {
                        string(out, &key);
                    }
                    out.push(':');
                    self.term(out, depth + 1)?;
                }
                out.push('}');
            }
            other => return Err(EtfError::Unsupported(other)),
        }
        Ok(())
    }
}

fn atom(out: &mut String, bytes: &[u8]) {
    match bytes {
        b"nil" | b"null" | b"undefined" => out.push_str("null"),
        b"true" => out.push_str("true"),
        b"false" => out.push_str("false"),
        other => string(out, &String::from_utf8_lossy(other)),
    }
}

fn string(out: &mut String, text: &str) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn float(out: &mut String, value: f64) {
    if value.is_finite() {
        let _ = write!(out, "{value}");
    } else {
        out.push_str("null");
    }
}

fn big(out: &mut String, negative: bool, digits: &[u8]) {
    if digits.len() <= 16 {
        let mut value: u128 = 0;
        for (i, byte) in digits.iter().enumerate() {
            value |= u128::from(*byte) << (8 * i);
        }
        if negative {
            let _ = write!(out, "-{value}");
        } else {
            let _ = write!(out, "{value}");
        }
    } else {
        out.push('"');
        for byte in digits.iter().rev() {
            let _ = write!(out, "{byte:02x}");
        }
        out.push('"');
    }
}

pub fn encode(value: &Value) -> Vec<u8> {
    let mut out = vec![VERSION];
    write_term(&mut out, value);
    out
}

fn write_atom(out: &mut Vec<u8>, name: &str) {
    if name.len() < 256 {
        out.push(SMALL_ATOM_EXT);
        out.push(name.len() as u8);
    } else {
        out.push(ATOM_EXT);
        out.extend_from_slice(&(name.len() as u16).to_be_bytes());
    }
    out.extend_from_slice(name.as_bytes());
}

fn write_binary(out: &mut Vec<u8>, text: &str) {
    out.push(BINARY_EXT);
    out.extend_from_slice(&(text.len() as u32).to_be_bytes());
    out.extend_from_slice(text.as_bytes());
}

fn write_term(out: &mut Vec<u8>, value: &Value) {
    match value {
        Value::Null => write_atom(out, "nil"),
        Value::Bool(true) => write_atom(out, "true"),
        Value::Bool(false) => write_atom(out, "false"),
        Value::String(text) => write_binary(out, text),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                write_integer(out, i);
            } else if let Some(u) = n.as_u64() {
                write_big(out, false, u);
            } else {
                out.push(NEW_FLOAT_EXT);
                out.extend_from_slice(&n.as_f64().unwrap_or(0.0).to_be_bytes());
            }
        }
        Value::Array(items) => {
            if items.is_empty() {
                out.push(NIL_EXT);
            } else {
                out.push(LIST_EXT);
                out.extend_from_slice(&(items.len() as u32).to_be_bytes());
                for item in items {
                    write_term(out, item);
                }
                out.push(NIL_EXT);
            }
        }
        Value::Object(map) => {
            out.push(MAP_EXT);
            out.extend_from_slice(&(map.len() as u32).to_be_bytes());
            for (key, item) in map {
                write_binary(out, key);
                write_term(out, item);
            }
        }
    }
}

fn write_integer(out: &mut Vec<u8>, value: i64) {
    if (0..256).contains(&value) {
        out.push(SMALL_INTEGER_EXT);
        out.push(value as u8);
    } else if let Ok(v) = i32::try_from(value) {
        out.push(INTEGER_EXT);
        out.extend_from_slice(&v.to_be_bytes());
    } else {
        write_big(out, value < 0, value.unsigned_abs());
    }
}

fn write_big(out: &mut Vec<u8>, negative: bool, magnitude: u64) {
    let bytes = magnitude.to_le_bytes();
    let used = 8 - bytes.iter().rev().take_while(|b| **b == 0).count();
    out.push(SMALL_BIG_EXT);
    out.push(used as u8);
    out.push(u8::from(negative));
    out.extend_from_slice(&bytes[..used]);
}

#[cfg(test)]
mod tests {
    use std::iter;
    use super::*;
    use serde_json::json;

    fn round_trip(value: Value) {
        let json = to_json(&encode(&value)).unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&json).unwrap(),
            value,
            "{json}"
        );
    }

    #[test]
    fn a_gateway_payload_survives_a_round_trip() {
        round_trip(
            json!({ "op": 40, "d": { "seq": 23, "qos": { "active": true, "ver": 31, "reasons": ["foregrounded"] } } }),
        );
        round_trip(json!({ "op": 40, "d": { "seq": null, "qos": null } }));
        round_trip(
            json!({ "op": 2, "d": { "capabilities": 1734653, "compress": false, "presence": { "status": "unknown", "since": 0, "activities": [], "afk": false } } }),
        );
    }

    #[test]
    fn numbers_use_the_smallest_tag_and_big_ones_round_trip() {
        assert_eq!(encode(&json!(7)), [131, 97, 7]);
        assert_eq!(encode(&json!(256)), [131, 98, 0, 0, 1, 0]);
        assert_eq!(encode(&json!(-1)), [131, 98, 255, 255, 255, 255]);
        round_trip(json!(1527668330054295552u64));
        round_trip(json!(-9_000_000_000i64));
        round_trip(json!(1.5));
        round_trip(json!(u64::MAX));
    }

    #[test]
    fn strings_keys_and_unicode_are_binaries() {
        assert_eq!(encode(&json!("hi")), [131, 109, 0, 0, 0, 2, b'h', b'i']);
        round_trip(json!({ "name": "Ünï \"quoted\" \\ \n \u{1f349}", "empty": "" }));
    }

    #[test]
    fn nil_true_false_and_empty_lists_have_erlpacks_shapes() {
        assert_eq!(encode(&json!(null)), [131, 115, 3, b'n', b'i', b'l']);
        assert_eq!(encode(&json!(true)), [131, 115, 4, b't', b'r', b'u', b'e']);
        assert_eq!(
            encode(&json!(false)),
            [131, 115, 5, b'f', b'a', b'l', b's', b'e']
        );
        assert_eq!(encode(&json!([])), [131, 106]);
        assert_eq!(
            to_json(&encode(&json!([true, false, null, []]))).unwrap(),
            "[true,false,null,[]]"
        );
    }

    #[test]
    fn a_byte_list_decodes_to_an_array_of_numbers() {
        assert_eq!(to_json(&[131, 107, 0, 3, 1, 2, 3]).unwrap(), "[1,2,3]");
    }

    #[test]
    fn utf8_atoms_and_binary_keys_decode() {
        assert_eq!(
            to_json(&[131, 116, 0, 0, 0, 1, 119, 2, b'o', b'p', 97, 10]).unwrap(),
            r#"{"op":10}"#
        );
        assert_eq!(
            to_json(&[131, 116, 0, 0, 0, 1, 109, 0, 0, 0, 2, b'o', b'p', 97, 10]).unwrap(),
            r#"{"op":10}"#
        );
    }

    #[test]
    fn malformed_input_is_an_error_not_a_panic() {
        assert_eq!(to_json(b"{}"), Err(EtfError::NotEtf));
        assert_eq!(
            to_json(&[131, 109, 0, 0, 0, 9, 1]),
            Err(EtfError::Truncated)
        );
        assert_eq!(to_json(&[131, 200]), Err(EtfError::Unsupported(200)));
        let deep: Vec<u8> = iter::once(131)
            .chain(iter::repeat_n([108, 0, 0, 0, 1], 2000).flatten())
            .collect();
        assert_eq!(to_json(&deep), Err(EtfError::TooDeep));
    }
}
