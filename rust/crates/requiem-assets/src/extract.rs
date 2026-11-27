use crate::Error;
use crate::manifest::{Entry, EntryKind, Manifest};
use crate::resolve::sha256_hex;
use crate::svg::{self, SvgIcon};
use std::str;

pub struct VerifiedChunk<'a> {
    index: u32,
    bytes: &'a [u8],
}

pub fn verify_chunk<'a>(
    manifest: &Manifest,
    index: u32,
    bytes: &'a [u8],
) -> Result<VerifiedChunk<'a>, Error> {
    let expected = &manifest
        .chunks
        .get(index as usize)
        .ok_or_else(|| Error::Unresolved(format!("no chunk {index}")))?
        .sha256;
    let actual = sha256_hex(bytes);
    if &actual != expected {
        return Err(Error::HashMismatch {
            expected: expected.clone(),
            actual,
        });
    }
    Ok(VerifiedChunk { index, bytes })
}

fn slice<'a>(
    chunk: &VerifiedChunk<'a>,
    wanted: u32,
    start: u32,
    end: u32,
) -> Result<&'a str, Error> {
    if chunk.index != wanted {
        return Err(Error::Unresolved(format!(
            "entry is in chunk {wanted}, not chunk {}",
            chunk.index
        )));
    }
    let slice = chunk
        .bytes
        .get(start as usize..end as usize)
        .ok_or(Error::BadSpan {
            start,
            end,
            len: chunk.bytes.len(),
        })?;
    str::from_utf8(slice).map_err(|e| Error::Parse(e.to_string()))
}

fn checked_slice<'a>(
    manifest: &Manifest,
    chunk: u32,
    bytes: &'a [u8],
    start: u32,
    end: u32,
) -> Result<&'a str, Error> {
    slice(&verify_chunk(manifest, chunk, bytes)?, chunk, start, end)
}

pub fn chunk_of(entry: &Entry) -> Option<u32> {
    match &entry.kind {
        EntryKind::SvgComponent { chunk, .. } | EntryKind::Text { chunk, .. } => Some(*chunk),
        EntryKind::File { .. } => None,
    }
}

pub fn extract_svg_verified(entry: &Entry, chunk: &VerifiedChunk<'_>) -> Result<SvgIcon, Error> {
    let EntryKind::SvgComponent {
        chunk: wanted,
        start,
        end,
        ..
    } = &entry.kind
    else {
        return Err(Error::Unresolved(format!(
            "`{}` is not an svg_component entry",
            entry.name
        )));
    };
    svg::summarize(slice(chunk, *wanted, *start, *end)?)
}

pub fn extract_text_verified(entry: &Entry, chunk: &VerifiedChunk<'_>) -> Result<String, Error> {
    let EntryKind::Text {
        chunk: wanted,
        start,
        end,
    } = &entry.kind
    else {
        return Err(Error::Unresolved(format!(
            "`{}` is not a text entry",
            entry.name
        )));
    };
    svg::decode_string_literal(slice(chunk, *wanted, *start, *end)?)
}

pub fn extract_svg(
    manifest: &Manifest,
    entry: &Entry,
    chunk_bytes: &[u8],
) -> Result<SvgIcon, Error> {
    let EntryKind::SvgComponent { chunk, .. } = &entry.kind else {
        return Err(Error::Unresolved(format!(
            "`{}` is not an svg_component entry",
            entry.name
        )));
    };
    extract_svg_verified(entry, &verify_chunk(manifest, *chunk, chunk_bytes)?)
}

pub fn extract_text(
    manifest: &Manifest,
    entry: &Entry,
    chunk_bytes: &[u8],
) -> Result<String, Error> {
    let EntryKind::Text { chunk, start, end } = &entry.kind else {
        return Err(Error::Unresolved(format!(
            "`{}` is not a text entry",
            entry.name
        )));
    };
    svg::decode_string_literal(checked_slice(manifest, *chunk, chunk_bytes, *start, *end)?)
}
