pub(super) fn read_varint(buf: &[u8], pos: &mut usize) -> Option<u64> {
    let mut value = 0u64;
    for shift in (0..64).step_by(7) {
        let byte = *buf.get(*pos)?;
        *pos += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Some(value);
        }
    }
    None
}

pub(super) fn put_varint(out: &mut Vec<u8>, mut v: u64) {
    while v >= 0x80 {
        out.push((v as u8) | 0x80);
        v >>= 7;
    }
    out.push(v as u8);
}

pub(super) fn put_key(out: &mut Vec<u8>, field: u64, wire: u64) {
    put_varint(out, (field << 3) | wire);
}

pub(super) fn put_bytes(out: &mut Vec<u8>, field: u64, bytes: &[u8]) {
    put_key(out, field, 2);
    put_varint(out, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

pub(super) fn put_uint(out: &mut Vec<u8>, field: u64, v: u64) {
    if v != 0 {
        put_key(out, field, 0);
        put_varint(out, v);
    }
}

pub(super) fn put_fixed64(out: &mut Vec<u8>, field: u64, value: u64) {
    put_key(out, field, 1);
    out.extend_from_slice(&value.to_le_bytes());
}

pub(super) enum Field<'a> {
    Varint(u64),
    Bytes(&'a [u8]),
}

pub(super) struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub(super) fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub(super) fn varint(&mut self) -> Option<u64> {
        read_varint(self.buf, &mut self.pos)
    }

    pub(super) fn next(&mut self) -> Option<(u64, Field<'a>)> {
        loop {
            if self.pos >= self.buf.len() {
                return None;
            }
            let key = read_varint(self.buf, &mut self.pos)?;
            let (field, wire) = (key >> 3, key & 7);
            match wire {
                0 => return Some((field, Field::Varint(read_varint(self.buf, &mut self.pos)?))),
                2 => {
                    let len = usize::try_from(read_varint(self.buf, &mut self.pos)?).ok()?;
                    let end = self.pos.checked_add(len).filter(|&e| e <= self.buf.len())?;
                    let bytes = &self.buf[self.pos..end];
                    self.pos = end;
                    return Some((field, Field::Bytes(bytes)));
                }
                1 => self.pos += 8,
                5 => self.pos += 4,
                _ => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varints_round_trip() {
        for v in [0, 1, 127, 128, 300, u64::from(u32::MAX), u64::MAX] {
            let mut buf = Vec::new();
            put_varint(&mut buf, v);
            let mut pos = 0;
            assert_eq!(read_varint(&buf, &mut pos), Some(v));
            assert_eq!(pos, buf.len());
        }
    }

    #[test]
    fn reader_walks_fields() {
        let mut buf = Vec::new();
        put_uint(&mut buf, 1, 7);
        put_bytes(&mut buf, 2, b"hi");
        put_fixed64(&mut buf, 3, 9);
        put_uint(&mut buf, 4, 1);
        let mut r = Reader::new(&buf);
        assert!(matches!(r.next(), Some((1, Field::Varint(7)))));
        assert!(matches!(r.next(), Some((2, Field::Bytes(b"hi")))));
        assert!(matches!(r.next(), Some((4, Field::Varint(1)))));
        assert!(r.next().is_none());
    }
}
