use base64::Engine;
use base64::engine::general_purpose;
use crate::decode_pool;
use crate::net_profile::Resource;
use crate::net_profile;
use crate::ui;
use requiem_ui::perf;
use skia_safe::image::CachingHint;
use skia_safe::images;
use skia_safe::surfaces;
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::hash::Hash;
use std::hash::Hasher;
use std::mem;
use std::path::Path;
use std::path;
use std::process;
use std::rc::Rc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::runtime::Handle;

#[derive(Clone)]
enum CacheEntry {
    Loading,

    LoadingDisk,

    Loaded {
        bytes: Arc<[u8]>,
        last_used: u64,
        pinned: bool,
    },

    OnDisk(path::PathBuf),
    Failed,
}

const RAW_BYTES_BUDGET: usize = 4 * 1024 * 1024;

const DECODED_BUDGET: usize = 48 * 1024 * 1024;

const MAX_DECODED_EDGE: i32 = 2048;

static USE_CLOCK: AtomicU64 = AtomicU64::new(0);
fn tick() -> u64 {
    USE_CLOCK.fetch_add(1, Ordering::Relaxed)
}

fn spill_dir() -> path::PathBuf {
    env::temp_dir()
        .join("requiem_image_cache")
        .join(process::id().to_string())
}

fn spill_path(url: &str) -> path::PathBuf {
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    spill_dir().join(format!("{:016x}", hasher.finish()))
}

fn sweep_stale_spill_dirs() {
    let Ok(read) = fs::read_dir(spill_dir().parent().unwrap_or(Path::new(".")))
    else {
        return;
    };
    let ours = spill_dir();
    for entry in read.flatten() {
        let path = entry.path();
        let old = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age > Duration::from_secs(6 * 3600));
        if path != ours && old {
            let _ = fs::remove_dir_all(path);
        }
    }
}

fn evict_raw(
    entries: &mut HashMap<String, CacheEntry>,
) -> Vec<(String, path::PathBuf, Arc<[u8]>)> {
    let total: usize = entries
        .values()
        .map(|e| {
            if let CacheEntry::Loaded { bytes, .. } = e {
                bytes.len()
            } else {
                0
            }
        })
        .sum();
    if total <= RAW_BYTES_BUDGET {
        return Vec::new();
    }
    let mut lru: Vec<(u64, String)> = entries
        .iter()
        .filter_map(|(k, e)| match e {
            CacheEntry::Loaded {
                last_used,
                pinned: false,
                ..
            } => Some((*last_used, k.clone())),
            _ => None,
        })
        .collect();
    lru.sort_unstable_by_key(|(used, _)| *used);

    let mut spilled = Vec::new();
    let mut remaining = total;
    for (_, key) in lru {
        if remaining <= RAW_BYTES_BUDGET {
            break;
        }
        let Some(CacheEntry::Loaded { bytes, .. }) = entries.remove(&key) else {
            continue;
        };
        remaining -= bytes.len();
        let path = spill_path(&key);
        spilled.push((key, path, bytes));
    }
    spilled
}

struct RawImage {
    pixels: Vec<u8>,
    width: i32,
    height: i32,
}

static DECODED_INBOX: Mutex<Vec<(String, Option<RawImage>)>> = Mutex::new(Vec::new());

fn decode_raw(bytes: &[u8], max_edge: i32) -> Option<RawImage> {
    let image = cap_edge(ui::decode_image(bytes)?, max_edge);
    let info = skia_safe::ImageInfo::new(
        image.dimensions(),
        skia_safe::ColorType::N32,
        skia_safe::AlphaType::Premul,
        None,
    );
    let row_bytes = info.min_row_bytes();
    let mut pixels = vec![0u8; row_bytes * info.height() as usize];
    image
        .read_pixels(
            &info,
            &mut pixels,
            row_bytes,
            (0, 0),
            CachingHint::Disallow,
        )
        .then_some(RawImage {
            pixels,
            width: info.width(),
            height: info.height(),
        })
}

fn raw_to_image(raw: RawImage) -> Option<skia_safe::Image> {
    let info = skia_safe::ImageInfo::new(
        (raw.width, raw.height),
        skia_safe::ColorType::N32,
        skia_safe::AlphaType::Premul,
        None,
    );
    images::raster_from_data(
        &info,
        skia_safe::Data::new_copy(&raw.pixels),
        raw.width as usize * 4,
    )
}

fn pixel_bytes(image: &skia_safe::Image) -> usize {
    image.width() as usize * image.height() as usize * 4
}

fn requested_edge(url: &str) -> Option<i32> {
    let query = url.split_once('?')?.1;
    query
        .split('&')
        .find_map(|pair| pair.strip_prefix("size=")?.parse::<i32>().ok())
        .filter(|&n| n > 0)
}

pub(crate) fn cap_edge(image: skia_safe::Image, max_edge: i32) -> skia_safe::Image {
    let max_edge = max_edge.min(MAX_DECODED_EDGE);
    let (w, h) = (image.width(), image.height());
    if w.max(h) <= max_edge {
        return image;
    }
    let scale = max_edge as f32 / w.max(h) as f32;
    let (tw, th) = (
        ((w as f32 * scale).round() as i32).max(1),
        ((h as f32 * scale).round() as i32).max(1),
    );
    let Some(mut surface) = surfaces::raster_n32_premul((tw, th)) else {
        return image;
    };
    let sampling = skia_safe::SamplingOptions::new(
        skia_safe::FilterMode::Linear,
        skia_safe::MipmapMode::Linear,
    );
    surface.canvas().draw_image_rect_with_sampling_options(
        &image,
        None,
        skia_safe::Rect::from_iwh(tw, th),
        sampling,
        &skia_safe::Paint::default(),
    );
    surface.image_snapshot()
}

#[derive(Clone)]
pub struct ImageCache {
    entries: Arc<Mutex<HashMap<String, CacheEntry>>>,
    runtime: Handle,
    http: wreq::Client,
    on_ready: Arc<dyn Fn() + Send + Sync>,

    decoded: Rc<RefCell<HashMap<String, (Option<skia_safe::Image>, u64)>>>,

    placeholders: Rc<RefCell<HashMap<String, Option<skia_safe::Image>>>>,

    decoding: Rc<RefCell<HashSet<String>>>,
}

impl ImageCache {
    pub(crate) fn download_context(&self) -> (Handle, wreq::Client, Arc<dyn Fn() + Send + Sync>) {
        (
            self.runtime.clone(),
            self.http.clone(),
            self.on_ready.clone(),
        )
    }

    /// Wakes the UI the way a finished download does, from any thread. For async work that
    /// isn't an image but should redraw what showed a placeholder.
    pub fn ready_notifier(&self) -> Arc<dyn Fn() + Send + Sync> {
        self.on_ready.clone()
    }

    pub fn register_mem_probe(&self) {
        let entries = self.entries.clone();
        let decoded = self.decoded.clone();
        perf::register_mem_probe(move || {
            let (n_bytes, raw) = entries
                .lock()
                .map(|e| {
                    e.values().fold((0usize, 0usize), |(n, b), v| match v {
                        CacheEntry::Loaded { bytes, .. } => (n + 1, b + bytes.len()),
                        _ => (n, b),
                    })
                })
                .unwrap_or_default();
            let decoded = decoded.borrow();
            let (n_dec, pixels) = decoded
                .values()
                .filter_map(|(img, _)| img.as_ref())
                .fold((0usize, 0usize), |(n, b), img| {
                    (n + 1, b + pixel_bytes(img))
                });
            let biggest = decoded
                .iter()
                .filter_map(|(url, (img, _))| img.as_ref().map(|i| (pixel_bytes(i), url)))
                .max();
            let biggest = biggest
                .map(|(b, url)| {
                    format!("{} {}", perf::mb(b), &url[..url.len().min(90)])
                })
                .unwrap_or_default();
            format!(
                "image bytes in RAM: {n_bytes} urls {} (more on disk) | decoded: {n_dec} images {} (largest {biggest})",
                perf::mb(raw),
                perf::mb(pixels)
            )
        });
    }

    pub fn new(
        runtime: Handle,
        http: wreq::Client,
        on_ready: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        sweep_stale_spill_dirs();
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            runtime,
            http,
            on_ready: Arc::new(on_ready),
            decoded: Rc::new(RefCell::new(HashMap::new())),
            placeholders: Rc::new(RefCell::new(HashMap::new())),
            decoding: Rc::new(RefCell::new(HashSet::new())),
        }
    }

    pub fn is_loading(&self, url: &str) -> bool {
        matches!(
            self.entries
                .lock()
                .expect("image cache mutex poisoned")
                .get(url),
            Some(CacheEntry::Loading | CacheEntry::LoadingDisk)
        ) || self.decoding.borrow().iter().any(|key| {
            key == url
                || key
                    .strip_prefix(url)
                    .is_some_and(|rest| rest.starts_with("#edge="))
        })
    }

    fn absorb_decoded(&self) {
        let finished: Vec<_> =
            mem::take(&mut *DECODED_INBOX.lock().expect("decoded images inbox poisoned"));
        if finished.is_empty() {
            return;
        }
        for (key, raw) in finished {
            self.decoding.borrow_mut().remove(&key);
            let image = raw.and_then(raw_to_image);
            self.decoded
                .borrow_mut()
                .insert(key.clone(), (image, tick()));
            self.evict_decoded(&key);
        }
    }

    fn evict_decoded(&self, keep: &str) {
        let mut decoded = self.decoded.borrow_mut();
        let mut total: usize = decoded
            .values()
            .filter_map(|(i, _)| i.as_ref())
            .map(pixel_bytes)
            .sum();
        while total > DECODED_BUDGET && decoded.len() > 1 {
            let Some(oldest) = decoded
                .iter()
                .filter(|(k, _)| k.as_str() != keep)
                .min_by_key(|(_, (_, used))| *used)
                .map(|(k, _)| k.clone())
            else {
                break;
            };
            if let Some((Some(evicted), _)) = decoded.remove(&oldest) {
                total -= pixel_bytes(&evicted);
            }
        }
    }

    /// Reads the bytes of an already downloaded image from any thread, for work that
    /// can't touch the cache itself (e.g. a notification waiting for an avatar).
    pub fn loaded_bytes_probe(&self) -> impl Fn(&str) -> Option<Arc<[u8]>> + Send + Sync + 'static {
        let entries = self.entries.clone();
        move |url| match entries.lock().ok()?.get(url) {
            Some(CacheEntry::Loaded { bytes, .. }) => Some(bytes.clone()),
            _ => None,
        }
    }

    pub fn get_or_request(&self, url: &str) -> Option<Arc<[u8]>> {
        let mut entries = self.entries.lock().expect("image cache mutex poisoned");
        match entries.get(url) {
            Some(CacheEntry::Loaded { .. }) => {
                if let Some(CacheEntry::Loaded {
                    bytes, last_used, ..
                }) = entries.get_mut(url)
                {
                    *last_used = tick();
                    return Some(bytes.clone());
                }
            }
            Some(CacheEntry::OnDisk(path)) => {
                let path = path.clone();

                entries.insert(url.to_string(), CacheEntry::LoadingDisk);
                drop(entries);

                let cache = self.entries.clone();
                let on_ready = self.on_ready.clone();
                let runtime = self.runtime.clone();
                let url = url.to_string();
                runtime.spawn_blocking(move || {
                    let bytes = fs::read(&path).ok().map(Arc::from);
                    {
                        let mut entries = cache.lock().expect("image cache mutex poisoned");
                        match bytes {
                            Some(bytes) => {
                                entries.insert(
                                    url.clone(),
                                    CacheEntry::Loaded {
                                        bytes,
                                        last_used: tick(),
                                        pinned: false,
                                    },
                                );
                            }

                            None => {
                                entries.remove(&url);
                            }
                        }
                    }
                    on_ready();
                });
                return None;
            }
            Some(CacheEntry::Loading)
            | Some(CacheEntry::LoadingDisk)
            | Some(CacheEntry::Failed) => return None,
            None => {}
        }
        entries.insert(url.to_string(), CacheEntry::Loading);
        drop(entries);

        let entries = self.entries.clone();
        let http = self.http.clone();
        let url = url.to_string();
        let on_ready = self.on_ready.clone();

        self.runtime.spawn(async move {
            let outcome = async {
                let response =
                    net_profile::get(&http, &url, Resource::Image)
                        .send()
                        .await?
                        .error_for_status()?;
                response.bytes().await
            }
            .await;

            let entry = match outcome {
                Ok(bytes) => CacheEntry::Loaded {
                    bytes: Arc::from(bytes.as_ref()),
                    last_used: tick(),
                    pinned: false,
                },
                Err(err) => {
                    log::warn!("failed to fetch image {url}: {err}");
                    CacheEntry::Failed
                }
            };
            let spill = {
                let mut entries = entries.lock().expect("image cache mutex poisoned");
                entries.insert(url.clone(), entry);
                evict_raw(&mut entries)
            };
            for (key, path, bytes) in spill {
                let _ = fs::create_dir_all(spill_dir());
                if fs::write(&path, &bytes[..]).is_ok() {
                    entries
                        .lock()
                        .expect("image cache mutex poisoned")
                        .insert(key, CacheEntry::OnDisk(path));
                }
            }
            on_ready();
        });

        None
    }

    pub fn insert_local(&self, key: &str, bytes: Arc<[u8]>) {
        self.decoded.borrow_mut().remove(key);
        self.entries
            .lock()
            .expect("image cache mutex poisoned")
            .insert(
                key.to_string(),
                CacheEntry::Loaded {
                    bytes,
                    last_used: tick(),
                    pinned: true,
                },
            );
    }

    pub fn get_or_decode_image(&self, url: &str) -> Option<skia_safe::Image> {
        self.decode_capped(url, requested_edge(url).unwrap_or(MAX_DECODED_EDGE))
    }

    pub fn get_or_decode_image_capped(&self, url: &str, edge: i32) -> Option<skia_safe::Image> {
        self.decode_capped(url, edge.clamp(1, MAX_DECODED_EDGE))
    }

    fn decode_capped(&self, url: &str, max_edge: i32) -> Option<skia_safe::Image> {
        let key = if max_edge == MAX_DECODED_EDGE || requested_edge(url) == Some(max_edge) {
            url.to_string()
        } else {
            format!("{url}#edge={max_edge}")
        };
        let key = key.as_str();
        self.absorb_decoded();
        if let Some((cached, last_used)) = self.decoded.borrow_mut().get_mut(key) {
            *last_used = tick();
            return cached.clone();
        }
        if self.decoding.borrow().contains(key) {
            return None;
        }
        let bytes = self.get_or_request(url)?;

        self.decoding.borrow_mut().insert(key.to_string());
        let (inbox_key, wake) = (key.to_string(), self.on_ready.clone());
        decode_pool::spawn(move || {
            let raw = decode_raw(&bytes, max_edge);
            DECODED_INBOX
                .lock()
                .expect("decoded images inbox poisoned")
                .push((inbox_key, raw));
            wake();
        });
        None
    }

    pub fn get_or_decode_placeholder(&self, base64_hash: &str) -> Option<skia_safe::Image> {
        if let Some(cached) = self.placeholders.borrow().get(base64_hash) {
            return cached.clone();
        }
        let image = decode_thumbhash(base64_hash);
        self.placeholders
            .borrow_mut()
            .insert(base64_hash.to_string(), image.clone());
        image
    }
}

fn decode_thumbhash(base64_hash: &str) -> Option<skia_safe::Image> {
    let bytes = general_purpose::STANDARD
        .decode(base64_hash)
        .ok()?;
    let (width, height, rgba) = thumbhash::thumb_hash_to_rgba(&bytes).ok()?;
    let info = skia_safe::ImageInfo::new(
        skia_safe::ISize::new(width as i32, height as i32),
        skia_safe::ColorType::RGBA8888,
        skia_safe::AlphaType::Unpremul,
        None,
    );
    images::raster_from_data(&info, skia_safe::Data::new_copy(&rgba), width * 4)
}

#[cfg(test)]
mod tests {
    use crate::rest;
    use std::sync::atomic::AtomicUsize;
    use std::thread;
    use std::time::Instant;
    use super::*;
    use tokio::runtime::Builder;

    fn png() -> Vec<u8> {
        let info = skia_safe::ImageInfo::new(
            (600, 300),
            skia_safe::ColorType::N32,
            skia_safe::AlphaType::Premul,
            None,
        );
        let pixels = vec![200u8; 600 * 300 * 4];
        let image =
            images::raster_from_data(&info, skia_safe::Data::new_copy(&pixels), 600 * 4)
                .expect("raster image");
        image
            .encode(None, skia_safe::EncodedImageFormat::PNG, None)
            .expect("png")
            .as_bytes()
            .to_vec()
    }

    #[test]
    fn an_image_is_decoded_on_a_worker_and_arrives_shrunk_without_blocking_the_caller() {
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let woken = Arc::new(AtomicUsize::new(0));
        let woken_in = woken.clone();
        let cache = ImageCache::new(
            runtime.handle().clone(),
            rest::build_image_client().expect("client"),
            move || {
                woken_in.fetch_add(1, Ordering::Relaxed);
            },
        );
        cache.insert_local("local-test-image", Arc::from(png()));

        assert!(
            cache
                .get_or_decode_image_capped("local-test-image", 128)
                .is_none(),
            "decoding is not done inside the call"
        );
        assert!(
            cache.is_loading("local-test-image"),
            "and it counts as loading until it lands"
        );

        let started = Instant::now();
        let image = loop {
            if let Some(image) = cache.get_or_decode_image_capped("local-test-image", 128) {
                break image;
            }
            assert!(
                started.elapsed() < Duration::from_secs(10),
                "the worker never finished"
            );
            thread::sleep(Duration::from_millis(5));
        };
        assert_eq!(
            image.width().max(image.height()),
            128,
            "shrunk to the edge it was asked for"
        );
        assert!(
            woken.load(Ordering::Relaxed) >= 1,
            "the render loop is woken when it lands"
        );
        assert!(!cache.is_loading("local-test-image"));
    }
}
