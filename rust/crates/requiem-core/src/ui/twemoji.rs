use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;

use skia_safe::Image;

use super::twemoji_atlas_data::{TWEMOJI_ATLAS_ENTRIES, TWEMOJI_ATLAS_TILE_PX};

static PACK: &[u8] = include_bytes!("../../assets/twemoji/tiles.bin");
const GRID_COLS: usize = 63;

const TILE_CACHE_LIMIT: usize = 768;

static ATLAS_LOOKUP: OnceLock<HashMap<&'static str, (u16, u16)>> = OnceLock::new();

thread_local! {

    static TILES: RefCell<(HashMap<usize, (Option<Image>, u64)>, u64)> = RefCell::new((HashMap::new(), 0));
}

fn lookup() -> &'static HashMap<&'static str, (u16, u16)> {
    ATLAS_LOOKUP.get_or_init(|| {
        TWEMOJI_ATLAS_ENTRIES
            .iter()
            .map(|&(e, col, row)| (e, (col, row)))
            .collect()
    })
}

fn u32_at(offset: usize) -> usize {
    u32::from_le_bytes(
        PACK[offset..offset + 4]
            .try_into()
            .expect("tile pack truncated"),
    ) as usize
}

fn decode_tile(index: usize) -> Option<Image> {
    let count = u32_at(0);
    if index >= count {
        return None;
    }
    let table_end = 4 + (count + 1) * 4;
    let (start, end) = (u32_at(4 + index * 4), u32_at(4 + (index + 1) * 4));
    if start == end {
        return None;
    }
    super::decode_image(&PACK[table_end + start..table_end + end])
}

fn tile_image(index: usize) -> Option<Image> {
    TILES.with(|tiles| {
        let (map, clock) = &mut *tiles.borrow_mut();
        *clock += 1;
        if let Some((image, used)) = map.get_mut(&index) {
            *used = *clock;
            return image.clone();
        }
        if map.len() >= TILE_CACHE_LIMIT {
            let mut by_use: Vec<(u64, usize)> =
                map.iter().map(|(k, (_, used))| (*used, *k)).collect();
            by_use.sort_unstable();
            for (_, key) in by_use.into_iter().take(TILE_CACHE_LIMIT / 4) {
                map.remove(&key);
            }
        }
        let image = decode_tile(index);
        map.insert(index, (image.clone(), *clock));
        image
    })
}

pub fn tile(emoji: &str) -> Option<(Image, (f32, f32, f32, f32))> {
    super::requiem_emoji::tile(emoji)
}

pub fn exact_tile(emoji: &str) -> Option<(Image, (f32, f32, f32, f32))> {
    super::requiem_emoji::tile_exact(emoji)
}

pub fn bundled_tile(emoji: &str) -> Option<(Image, (f32, f32, f32, f32))> {
    let &(col, row) = lookup().get(emoji)?;
    let image = tile_image(row as usize * GRID_COLS + col as usize)?;
    let size = TWEMOJI_ATLAS_TILE_PX;
    Some((image, (0.0, 0.0, size, size)))
}
