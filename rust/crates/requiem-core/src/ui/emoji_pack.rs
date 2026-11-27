use futures_util::StreamExt;
use requiem_ui::control;
use requiem_ui::timer;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::fs;
use std::io::BufReader;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::fs as tokio_fs;

use requiem_ui::signal::{Signal, signal};
use skia_safe::Image;

use crate::images::ImageCache;
use crate::utils::prefs;

static VENDORS_JSON: &str = include_str!("../../assets/emoji-packs/vendors.json");

const DEFAULT_PACK_SOURCE: &str = "https://raw.githubusercontent.com/NurMarvin/discord-skia/mistress/crates/requiem-core/assets/requiem-emojis";

const POLL: Duration = Duration::from_millis(250);

const MEMO_LIMIT: usize = 1200;

const PACK_FILE: &str = "pack.bin";

const LEGACY_KEYS: [(&str, &str); 1] = [("fluent", "microsoft-3D-fluent")];

struct Vendor {
    slug: String,
    title: String,

    preview: Option<String>,
}

fn vendors() -> &'static [Vendor] {
    static VENDORS: OnceLock<Vec<Vendor>> = OnceLock::new();
    VENDORS.get_or_init(|| {
        let list: Vec<serde_json::Value> = serde_json::from_str(VENDORS_JSON).unwrap_or_default();
        list.iter()
            .filter_map(|v| {
                Some(Vendor {
                    slug: v.get("slug")?.as_str()?.to_string(),
                    title: v.get("title")?.as_str()?.to_string(),
                    preview: v
                        .get("preview")
                        .and_then(|p| p.as_str())
                        .map(str::to_string),
                })
            })
            .collect()
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct EmojiPack(usize);

const BUILT_IN: usize = 2;

impl EmojiPack {
    pub const TWEMOJI: EmojiPack = EmojiPack(0);
    pub const NATIVE: EmojiPack = EmojiPack(1);

    fn count() -> usize {
        BUILT_IN + vendors().len()
    }

    pub fn index(self) -> usize {
        self.0
    }

    fn vendor(self) -> Option<&'static Vendor> {
        vendors().get(self.0.checked_sub(BUILT_IN)?)
    }

    pub fn available() -> Vec<EmojiPack> {
        (0..Self::count())
            .map(EmojiPack)
            .filter(|pack| {
                !(cfg!(target_os = "macos") && pack.vendor().is_some_and(|v| v.slug == "apple"))
            })
            .collect()
    }

    pub fn key(self) -> &'static str {
        match self.0 {
            0 => "twemoji",
            1 => "native",
            _ => self.vendor().map_or("twemoji", |v| v.slug.as_str()),
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        let key = LEGACY_KEYS
            .iter()
            .find(|(old, _)| *old == key)
            .map_or(key, |(_, new)| *new);
        (0..Self::count())
            .map(EmojiPack)
            .find(|pack| pack.key() == key)
    }

    pub fn is_remote(self) -> bool {
        self.0 >= BUILT_IN
    }

    pub fn label(self) -> String {
        match self.0 {
            0 => requiem_i18n::t!("requiem-emoji-twemoji"),
            1 if cfg!(target_os = "macos") => requiem_i18n::t!("requiem-emoji-native-apple"),
            1 if cfg!(windows) => requiem_i18n::t!("requiem-emoji-native-windows"),
            1 => requiem_i18n::t!("requiem-emoji-native-other"),
            _ => self.vendor().map(|v| v.title.clone()).unwrap_or_default(),
        }
    }

    fn folder(self) -> Option<PathBuf> {
        Some(prefs::config_dir()?.join("emoji").join(self.key()))
    }

    fn pack_path(self) -> Option<PathBuf> {
        Some(self.folder()?.join(PACK_FILE))
    }

    pub fn is_downloaded(self) -> bool {
        !self.is_remote() || self.pack_path().is_some_and(|path| path.is_file())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackStatus {
    NotDownloaded,

    Downloading { percent: usize },

    Ready,

    Failed,
}

static CURRENT: AtomicUsize = AtomicUsize::new(0);

struct PackFile {
    header: requiem_emoji::Header,
    file: RefCell<File>,
}

impl PackFile {
    fn open(pack: EmojiPack) -> Option<PackFile> {
        let mut file = File::open(pack.pack_path()?).ok()?;
        let header = requiem_emoji::Header::read(&mut BufReader::new(&mut file)).ok()?;
        Some(PackFile {
            header,
            file: RefCell::new(file),
        })
    }

    fn tile_bytes(&self, emoji: &str) -> Option<Vec<u8>> {
        let (at, len) = self.header.tile(self.header.find(emoji)?)?;
        let mut file = self.file.borrow_mut();
        file.seek(SeekFrom::Start(at)).ok()?;
        let mut bytes = vec![0u8; len];
        file.read_exact(&mut bytes).ok()?;
        Some(bytes)
    }
}

thread_local! {

    static IMAGES: RefCell<Option<ImageCache>> = const { RefCell::new(None) };

    static FILES: RefCell<HashMap<EmojiPack, Option<Rc<PackFile>>>> = RefCell::new(HashMap::new());

    static MEMO: RefCell<(HashMap<(EmojiPack, String), Option<Image>>, Vec<(EmojiPack, String)>)> = RefCell::new((HashMap::new(), Vec::new()));

    static STATUS: Signal<Vec<PackStatus>> = signal(vec![PackStatus::Ready; EmojiPack::count()]);

    static PENDING: Signal<Option<EmojiPack>> = signal(None);

    static RUNNING: RefCell<Option<(EmojiPack, Arc<Progress>)>> = const { RefCell::new(None) };

    static PREVIEW_TICK: Signal<u64> = signal(0);

    static PREVIEWS: RefCell<HashMap<EmojiPack, Option<Image>>> = RefCell::new(HashMap::new());
}

struct Progress {
    percent: AtomicUsize,
    finished: AtomicBool,
    cancelled: AtomicBool,
}

pub fn current() -> EmojiPack {
    EmojiPack(CURRENT.load(Ordering::Relaxed))
}

pub fn statuses() -> Signal<Vec<PackStatus>> {
    STATUS.with(Clone::clone)
}

pub fn picked() -> EmojiPack {
    PENDING.with(|p| p.get()).unwrap_or_else(current)
}

pub fn preview_tick() -> Signal<u64> {
    PREVIEW_TICK.with(Clone::clone)
}

pub fn init(image_cache: Option<ImageCache>) {
    if let Some(cache) = image_cache {
        IMAGES.with(|images| *images.borrow_mut() = Some(cache));
    }
    let saved = prefs::load()
        .requiem_emoji
        .as_deref()
        .and_then(EmojiPack::from_key)
        .unwrap_or_default();

    let pack = if saved.is_downloaded() {
        saved
    } else {
        EmojiPack::default()
    };
    CURRENT.store(pack.index(), Ordering::Relaxed);
    STATUS.with(|status| {
        status.set(
            (0..EmojiPack::count())
                .map(|n| {
                    if EmojiPack(n).is_downloaded() {
                        PackStatus::Ready
                    } else {
                        PackStatus::NotDownloaded
                    }
                })
                .collect(),
        )
    });
}

fn set_status(pack: EmojiPack, status: PackStatus) {
    STATUS.with(|all| {
        let mut next = all.peek();
        if let Some(slot) = next.get_mut(pack.index()) {
            *slot = status;
            all.set(next);
        }
    });
}

fn activate(pack: EmojiPack) {
    CURRENT.store(pack.index(), Ordering::Relaxed);
    let mut saved = prefs::load();
    saved.requiem_emoji = Some(pack.key().to_string());
    prefs::save(&saved);
    control::locale_changed();
}

pub fn select(pack: EmojiPack) {
    cancel_running();
    if pack.is_downloaded() {
        PENDING.with(|p| p.set(None));
        set_status(pack, PackStatus::Ready);
        activate(pack);
    } else {
        PENDING.with(|p| p.set(Some(pack)));
        download(pack);
    }
}

fn cancel_running() {
    if let Some((pack, progress)) = RUNNING.with(|r| r.borrow_mut().take()) {
        progress.cancelled.store(true, Ordering::Relaxed);
        set_status(pack, PackStatus::NotDownloaded);
    }
}

async fn get(
    http: &wreq::Client,
    source: &str,
    progress: &Progress,
    from: usize,
    span: usize,
) -> Option<Vec<u8>> {
    if !source.starts_with("http") {
        let bytes = tokio_fs::read(source).await.ok()?;
        progress.percent.store(from + span, Ordering::Relaxed);
        return Some(bytes);
    }
    let response = http
        .get(source)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;
    let total = response.content_length().unwrap_or(0) as usize;
    let mut stream = response.bytes_stream();
    let mut bytes = Vec::with_capacity(total);
    while let Some(chunk) = stream.next().await {
        if progress.cancelled.load(Ordering::Relaxed) {
            return None;
        }
        bytes.extend_from_slice(&chunk.ok()?);
        if let Some(done) = (bytes.len() * span).checked_div(total) {
            progress
                .percent
                .store(from + done.min(span), Ordering::Relaxed);
        }
    }
    progress.percent.store(from + span, Ordering::Relaxed);
    Some(bytes)
}

fn download(pack: EmojiPack) {
    let Some((runtime, http, wake)) = IMAGES.with(|images| {
        images
            .borrow()
            .as_ref()
            .map(|cache| cache.download_context())
    }) else {
        set_status(pack, PackStatus::Failed);
        return;
    };
    let (Some(folder), Some(path)) = (pack.folder(), pack.pack_path()) else {
        set_status(pack, PackStatus::Failed);
        return;
    };
    let progress = Arc::new(Progress {
        percent: AtomicUsize::new(0),
        finished: AtomicBool::new(false),
        cancelled: AtomicBool::new(false),
    });
    set_status(pack, PackStatus::Downloading { percent: 0 });
    RUNNING.with(|r| *r.borrow_mut() = Some((pack, progress.clone())));

    let task = progress.clone();
    runtime.spawn(async move {
        let base = env::var("REQUIEM_EMOJI_PACK_SOURCE")
            .unwrap_or_else(|_| DEFAULT_PACK_SOURCE.to_string());
        let file = format!("{}.pack", pack.key());
        let source = if base.starts_with("http") {
            format!("{}/{file}", base.trim_end_matches('/'))
        } else {
            PathBuf::from(&base)
                .join(&file)
                .to_string_lossy()
                .into_owned()
        };
        let bytes = get(&http, &source, &task, 0, 100).await;

        let usable = bytes.filter(|bytes| {
            requiem_emoji::Header::read(&mut &bytes[..]).is_ok_and(|header| !header.is_empty())
        });
        if let (Some(bytes), false) = (usable, task.cancelled.load(Ordering::Relaxed)) {
            if tokio_fs::create_dir_all(&folder).await.is_ok() {
                let part = path.with_extension("part");
                if tokio_fs::write(&part, &bytes).await.is_ok() {
                    let _ = tokio_fs::rename(&part, &path).await;
                }
            }
        }
        task.finished.store(true, Ordering::Relaxed);
        wake();
    });
    poll(pack, progress);
}

fn poll(pack: EmojiPack, progress: Arc<Progress>) {
    timer::set_timeout(POLL, move || {
        if progress.cancelled.load(Ordering::Relaxed) {
            return;
        }
        if !progress.finished.load(Ordering::Relaxed) {
            set_status(
                pack,
                PackStatus::Downloading {
                    percent: progress.percent.load(Ordering::Relaxed).min(99),
                },
            );
            return poll(pack, progress);
        }
        RUNNING.with(|r| *r.borrow_mut() = None);
        PENDING.with(|p| p.set(None));
        if pack.is_downloaded() {
            FILES.with(|files| files.borrow_mut().remove(&pack));
            MEMO.with(|memo| *memo.borrow_mut() = (HashMap::new(), Vec::new()));
            set_status(pack, PackStatus::Ready);
            activate(pack);
        } else {
            set_status(pack, PackStatus::Failed);
        }
    });
}

pub fn tile(emoji: &str) -> Option<(Image, (f32, f32, f32, f32))> {
    tile_for(current(), &super::emoji_skin::shown(emoji))
}

pub fn tile_exact(emoji: &str) -> Option<(Image, (f32, f32, f32, f32))> {
    tile_for(current(), emoji)
}

fn tile_for(pack: EmojiPack, emoji: &str) -> Option<(Image, (f32, f32, f32, f32))> {
    match pack {
        EmojiPack::TWEMOJI => super::twemoji::bundled_tile(emoji),
        EmojiPack::NATIVE => None,
        _ => {
            let fallback = super::twemoji::bundled_tile(emoji)?;
            match pack_image(pack, emoji) {
                Some(image) => {
                    let (w, h) = (image.width() as f32, image.height() as f32);
                    Some((image, (0.0, 0.0, w, h)))
                }
                None => Some(fallback),
            }
        }
    }
}

fn open_pack(pack: EmojiPack) -> Option<Rc<PackFile>> {
    FILES.with(|files| {
        files
            .borrow_mut()
            .entry(pack)
            .or_insert_with(|| PackFile::open(pack).map(Rc::new))
            .clone()
    })
}

fn pack_image(pack: EmojiPack, emoji: &str) -> Option<Image> {
    let key = (pack, emoji.to_string());
    if let Some(known) = MEMO.with(|memo| memo.borrow().0.get(&key).cloned()) {
        return known;
    }
    let image = open_pack(pack)
        .and_then(|file| file.tile_bytes(emoji))
        .and_then(|bytes| super::decode_image(&bytes));
    MEMO.with(|memo| {
        let (map, order) = &mut *memo.borrow_mut();
        if order.len() >= MEMO_LIMIT {
            for old in order.drain(..MEMO_LIMIT / 4) {
                map.remove(&old);
            }
        }
        order.push(key.clone());
        map.insert(key, image.clone());
    });
    image
}

fn preview_path(pack: EmojiPack) -> Option<PathBuf> {
    Some(
        prefs::config_dir()?
            .join("emoji")
            .join("previews")
            .join(pack.key()),
    )
}

pub fn preview(pack: EmojiPack) -> Option<Image> {
    const SMILE: &str = "\u{1F600}";
    match pack {
        EmojiPack::TWEMOJI => super::twemoji::bundled_tile(SMILE).map(|(image, _)| image),
        EmojiPack::NATIVE => None,
        _ if pack.is_downloaded() => pack_image(pack, SMILE),
        _ => {
            if let Some(known) = PREVIEWS.with(|previews| previews.borrow().get(&pack).cloned()) {
                if known.is_some() {
                    return known;
                }
            }
            let path = preview_path(pack)?;
            if let Some(image) = fs::read(&path)
                .ok()
                .and_then(|bytes| super::decode_image(&bytes))
            {
                PREVIEWS.with(|previews| previews.borrow_mut().insert(pack, Some(image.clone())));
                return Some(image);
            }

            if PREVIEWS.with(|previews| previews.borrow_mut().insert(pack, None).is_some()) {
                return None;
            }
            let url = pack.vendor()?.preview.clone()?;
            let (runtime, http, wake) = IMAGES.with(|images| {
                images
                    .borrow()
                    .as_ref()
                    .map(|cache| cache.download_context())
            })?;
            watch_preview(pack, path.clone(), 25);
            runtime.spawn(async move {
                if let Ok(bytes) = async {
                    http.get(&url)
                        .send()
                        .await?
                        .error_for_status()?
                        .bytes()
                        .await
                }
                .await
                {
                    if let Some(dir) = path.parent() {
                        let _ = tokio_fs::create_dir_all(dir).await;
                    }
                    let _ = tokio_fs::write(&path, &bytes).await;
                    wake();
                }
            });
            None
        }
    }
}

fn watch_preview(pack: EmojiPack, path: PathBuf, tries: u32) {
    timer::set_timeout(Duration::from_millis(400), move || {
        if path.exists() {
            PREVIEWS.with(|previews| previews.borrow_mut().remove(&pack));
            PREVIEW_TICK.with(|t| t.update(|n| *n += 1));
        } else if tries > 0 {
            watch_preview(pack, path, tries - 1);
        }
    });
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use super::*;

    #[test]
    fn the_manifest_lists_the_vendors_after_twemoji_and_the_system_s_emoji() {
        assert!(vendors().len() >= 3, "{}", vendors().len());
        let keys: Vec<&str> = (0..EmojiPack::count())
            .map(|n| EmojiPack(n).key())
            .collect();
        assert_eq!(&keys[..2], ["twemoji", "native"]);
        for wanted in ["apple", "microsoft-3D-fluent", "openmoji"] {
            assert!(keys.contains(&wanted), "{wanted} in {keys:?}");
        }

        let mut sorted = keys.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), keys.len());
    }

    #[test]
    fn packs_round_trip_through_their_keys() {
        for n in 0..EmojiPack::count() {
            let pack = EmojiPack(n);
            assert_eq!(EmojiPack::from_key(pack.key()), Some(pack));
            assert_eq!(pack.is_remote(), n >= 2);
        }
        assert_eq!(EmojiPack::from_key("nonsense"), None);

        assert_eq!(
            EmojiPack::from_key("fluent").map(EmojiPack::key),
            Some("microsoft-3D-fluent")
        );
    }

    #[test]
    fn a_vendor_that_is_not_downloaded_draws_twemoji() {
        let vendor = EmojiPack::available()
            .into_iter()
            .find(|p| p.is_remote())
            .expect("a vendor");
        assert!(tile_for(vendor, "\u{1F600}").is_some());
        assert!(tile_for(vendor, "not an emoji").is_none());
        assert!(tile_for(EmojiPack::NATIVE, "\u{1F600}").is_none());
        assert!(tile_for(EmojiPack::TWEMOJI, "\u{1F600}").is_some());
    }

    #[test]
    fn the_built_packs_read_back_through_the_apps_reader() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("emoji-packs");
        let mut checked = 0;
        for vendor in vendors() {
            let Ok(mut file) = File::open(dir.join(format!("{}.pack", vendor.slug)))
            else {
                continue;
            };
            let header = requiem_emoji::Header::read(&mut BufReader::new(&mut file))
                .unwrap_or_else(|e| panic!("{}: {e}", vendor.slug));
            assert!(header.len() > 50, "{}: {}", vendor.slug, header.len());
            if let Some((at, len)) = header.find("\u{1F600}").and_then(|n| header.tile(n)) {
                let mut bytes = vec![0u8; len];
                file.seek(SeekFrom::Start(at)).unwrap();
                file.read_exact(&mut bytes).unwrap();
                assert!(bytes.starts_with(b"RIFF"), "{}: a WebP tile", vendor.slug);
            }
            checked += 1;
        }
        eprintln!("{checked} built packs checked");
    }

    #[test]
    fn every_vendor_has_a_name_to_list_it_by() {
        for pack in EmojiPack::available() {
            assert!(!pack.label().is_empty(), "{}", pack.key());
        }
    }
}
