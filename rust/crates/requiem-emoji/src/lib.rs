use std::collections::HashMap;
use std::io::{self, Read};

pub const MAGIC: &[u8; 4] = b"EMP1";

pub fn write(entries: &[(String, Vec<u8>)]) -> Vec<u8> {
    let index: String = entries
        .iter()
        .enumerate()
        .map(|(n, (emoji, _))| format!("{emoji}\t{n}\n"))
        .collect();
    let tiles: usize = entries.iter().map(|(_, tile)| tile.len()).sum();
    let mut out = Vec::with_capacity(12 + index.len() + (entries.len() + 1) * 4 + tiles);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    out.extend_from_slice(&(index.len() as u32).to_le_bytes());
    out.extend_from_slice(index.as_bytes());
    let mut at = 0u32;
    for (_, tile) in entries {
        out.extend_from_slice(&at.to_le_bytes());
        at += tile.len() as u32;
    }
    out.extend_from_slice(&at.to_le_bytes());
    for (_, tile) in entries {
        out.extend_from_slice(tile);
    }
    out
}

pub struct Header {
    index: HashMap<String, u32>,
    offsets: Vec<u32>,

    tiles_start: u64,
}

impl Header {
    pub fn read(reader: &mut impl Read) -> io::Result<Header> {
        let bad = |what: &str| io::Error::new(io::ErrorKind::InvalidData, what.to_string());
        let mut word = [0u8; 4];
        reader.read_exact(&mut word)?;
        if &word != MAGIC {
            return Err(bad("not an emoji pack"));
        }
        let mut next = |reader: &mut dyn Read| -> io::Result<u32> {
            reader.read_exact(&mut word)?;
            Ok(u32::from_le_bytes(word))
        };
        let count = next(reader)? as usize;
        let index_len = next(reader)? as usize;
        if count > 1 << 20 || index_len > 1 << 28 {
            return Err(bad("implausible emoji pack"));
        }
        let mut raw = vec![0u8; index_len];
        reader.read_exact(&mut raw)?;
        let text = String::from_utf8(raw).map_err(|_| bad("index is not UTF-8"))?;
        let index = text
            .lines()
            .filter_map(|line| line.rsplit_once('\t'))
            .filter_map(|(emoji, n)| Some((emoji.to_string(), n.parse().ok()?)))
            .collect();
        let offsets = (0..=count)
            .map(|_| next(reader))
            .collect::<io::Result<Vec<u32>>>()?;
        Ok(Header {
            index,
            offsets,
            tiles_start: 12 + index_len as u64 + (count as u64 + 1) * 4,
        })
    }

    pub fn len(&self) -> usize {
        self.offsets.len().saturating_sub(1)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn find(&self, emoji: &str) -> Option<u32> {
        if let Some(n) = self.index.get(emoji) {
            return Some(*n);
        }
        let bare: String = emoji.chars().filter(|c| *c != '\u{FE0F}').collect();
        if bare != emoji {
            return self.index.get(&bare).copied();
        }
        self.index.get(&format!("{emoji}\u{FE0F}")).copied()
    }

    pub fn tile(&self, n: u32) -> Option<(u64, usize)> {
        let (start, end) = (
            *self.offsets.get(n as usize)?,
            *self.offsets.get(n as usize + 1)?,
        );
        (end > start).then_some((self.tiles_start + start as u64, (end - start) as usize))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack() -> Vec<u8> {
        write(&[
            ("\u{1F600}".to_string(), vec![1, 2, 3]),
            ("\u{2764}\u{FE0F}".to_string(), vec![4, 5]),
            ("\u{1F44D}\u{1F3FD}".to_string(), vec![6]),
        ])
    }

    #[test]
    fn a_written_pack_reads_back() {
        let bytes = pack();
        let header = Header::read(&mut &bytes[..]).unwrap();
        assert_eq!(header.len(), 3);
        let tile = |emoji: &str| {
            let (at, len) = header.tile(header.find(emoji)?)?;
            Some(bytes[at as usize..at as usize + len].to_vec())
        };
        assert_eq!(tile("\u{1F600}"), Some(vec![1, 2, 3]));
        assert_eq!(tile("\u{2764}\u{FE0F}"), Some(vec![4, 5]));
        assert_eq!(tile("\u{1F44D}\u{1F3FD}"), Some(vec![6]));
        assert_eq!(tile("\u{1F680}"), None);
    }

    #[test]
    fn the_variation_selector_does_not_matter_when_looking_up() {
        let bytes = pack();
        let header = Header::read(&mut &bytes[..]).unwrap();
        assert_eq!(header.find("\u{2764}"), header.find("\u{2764}\u{FE0F}"));
        assert!(header.find("\u{2764}").is_some());
        assert_eq!(header.find("\u{1F600}\u{FE0F}"), header.find("\u{1F600}"));
    }

    #[test]
    fn something_else_is_refused() {
        assert!(Header::read(&mut &b"RIFFxxxxxxxxxxxx"[..]).is_err());
        assert!(Header::read(&mut &b"EMP1"[..]).is_err());
    }
}
