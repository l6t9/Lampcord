use moka::sync::Cache;
use skia_safe::font_style::Weight;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextCacheKey {
    Hash(u64),
    Stable(u64),
}

impl TextCacheKey {
    pub fn from_text(text: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        Self::Hash(hasher.finish())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct MeasureKey {
    key: TextCacheKey,
    font_size_bits: u32,

    weight: i32,
    width_bucket: Option<u32>,
}

pub struct MeasureCache {
    cache: Cache<MeasureKey, (u32, u32)>,
}

impl MeasureCache {
    pub fn new(max_capacity: u64) -> Self {
        Self {
            cache: Cache::builder().max_capacity(max_capacity).build(),
        }
    }

    pub fn get_or_insert_with(
        &self,
        key: TextCacheKey,
        font_size: f32,
        weight: Weight,
        width: Option<f32>,
        compute: impl FnOnce() -> (f32, f32),
    ) -> (f32, f32) {
        let measure_key = MeasureKey {
            key,
            font_size_bits: font_size.to_bits(),
            weight: *weight,
            width_bucket: width.map(|w| w.round() as u32),
        };
        if let Some((w, h)) = self.cache.get(&measure_key) {
            return (f32::from_bits(w), f32::from_bits(h));
        }
        let (w, h) = compute();
        self.cache.insert(measure_key, (w.to_bits(), h.to_bits()));
        (w, h)
    }
}

impl Default for MeasureCache {
    fn default() -> Self {
        Self::new(4096)
    }
}
