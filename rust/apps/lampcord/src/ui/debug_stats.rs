use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[derive(Debug, Default, Clone)]
pub struct MemorySample {
    pub vm_rss_kb: u64,
    pub rss_anon_kb: u64,
    pub rss_file_kb: u64,
    pub rss_shmem_kb: u64,
    pub vm_hwm_kb: u64,
}

#[derive(Debug, Default, Clone)]
pub struct RenderSnapshotStats {
    pub render_frame: u64,
    pub image_elements: usize,
    pub unique_image_urls: usize,
    pub guild_icons: usize,
    pub avatars: usize,
    pub attachment_images: usize,
    pub embed_images: usize,
}

#[derive(Debug)]
struct Inner {
    http_requests: AtomicU64,
    http_bytes: AtomicU64,
    http_errors: AtomicU64,
    http_unique_urls: Mutex<HashSet<String>>,
    last_http_url: Mutex<Option<String>>,
    last_http_bytes: AtomicU64,
    render: Mutex<RenderSnapshotStats>,
    render_unique_urls: Mutex<HashSet<String>>,
    frame_counter: AtomicU64,
    backend_pushes: AtomicU64,
    backend_gateway_batches: AtomicU64,
    backend_gateway_events: AtomicU64,
    started_at: Instant,
    last_log_at: Mutex<Instant>,
    enabled_overlay: AtomicBool,
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            http_requests: AtomicU64::new(0),
            http_bytes: AtomicU64::new(0),
            http_errors: AtomicU64::new(0),
            http_unique_urls: Mutex::new(HashSet::new()),
            last_http_url: Mutex::new(None),
            last_http_bytes: AtomicU64::new(0),
            render: Mutex::new(RenderSnapshotStats::default()),
            render_unique_urls: Mutex::new(HashSet::new()),
            frame_counter: AtomicU64::new(0),
            backend_pushes: AtomicU64::new(0),
            backend_gateway_batches: AtomicU64::new(0),
            backend_gateway_events: AtomicU64::new(0),
            started_at: Instant::now(),
            last_log_at: Mutex::new(Instant::now()),
            enabled_overlay: AtomicBool::new(false),
        }
    }
}

static STATS: OnceLock<Inner> = OnceLock::new();

fn stats() -> &'static Inner {
    STATS.get_or_init(Inner::default)
}

pub fn toggle_overlay() -> bool {
    let s = stats();
    let prev = s.enabled_overlay.load(Ordering::Relaxed);
    s.enabled_overlay.store(!prev, Ordering::Relaxed);
    !prev
}

pub fn set_overlay(enabled: bool) {
    stats().enabled_overlay.store(enabled, Ordering::Relaxed);
}

pub fn overlay_enabled() -> bool {
    stats().enabled_overlay.load(Ordering::Relaxed)
}

pub fn record_http_response(url: &str, bytes: usize, ok: bool) {
    let s = stats();
    s.http_requests.fetch_add(1, Ordering::Relaxed);
    s.http_bytes.fetch_add(bytes as u64, Ordering::Relaxed);
    s.last_http_bytes.store(bytes as u64, Ordering::Relaxed);
    if !ok {
        s.http_errors.fetch_add(1, Ordering::Relaxed);
    }
    if let Ok(mut set) = s.http_unique_urls.lock() {
        if set.len() < 20_000 {
            set.insert(url.to_owned());
        }
    }
    if let Ok(mut last) = s.last_http_url.lock() {
        *last = Some(url.to_owned());
    }
}

pub fn record_backend_push() {
    stats().backend_pushes.fetch_add(1, Ordering::Relaxed);
}

pub fn record_gateway_batch(events: usize) {
    let s = stats();
    s.backend_gateway_batches.fetch_add(1, Ordering::Relaxed);
    s.backend_gateway_events
        .fetch_add(events as u64, Ordering::Relaxed);
}

pub fn begin_render_frame() -> u64 {
    let s = stats();
    let frame = s.frame_counter.fetch_add(1, Ordering::Relaxed) + 1;
    if let Ok(mut set) = s.render_unique_urls.lock() {
        set.clear();
    }
    if let Ok(mut render) = s.render.lock() {
        *render = RenderSnapshotStats {
            render_frame: frame,
            ..RenderSnapshotStats::default()
        };
    }
    frame
}

pub fn record_render_image(url: Option<&str>) {
    record_render_image_with_kind(url, ImageKind::Avatar);
}

pub fn record_render_image_with_kind(url: Option<&str>, kind: ImageKind) {
    let s = stats();
    if let Some(url) = url.filter(|u| !u.is_empty()) {
        if let Ok(mut set) = s.render_unique_urls.lock() {
            set.insert(url.to_owned());
        }
    }
    if let Ok(mut render) = s.render.lock() {
        render.image_elements += 1;
        render.unique_image_urls = s
            .render_unique_urls
            .lock()
            .map(|set| set.len())
            .unwrap_or(0);
        match kind {
            ImageKind::GuildIcon => render.guild_icons += 1,
            ImageKind::Avatar => render.avatars += 1,
            ImageKind::Attachment => render.attachment_images += 1,
            ImageKind::Embed => render.embed_images += 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ImageKind {
    GuildIcon,
    Avatar,
    Attachment,
    Embed,
}

pub fn set_render_unique_image_urls(count: usize) {
    if let Ok(mut render) = stats().render.lock() {
        render.unique_image_urls = count;
    }
}

pub fn reset_http_stats() {
    let s = stats();
    s.http_requests.store(0, Ordering::Relaxed);
    s.http_bytes.store(0, Ordering::Relaxed);
    s.http_errors.store(0, Ordering::Relaxed);
    s.last_http_bytes.store(0, Ordering::Relaxed);
    if let Ok(mut set) = s.http_unique_urls.lock() {
        set.clear();
    }
    if let Ok(mut last) = s.last_http_url.lock() {
        *last = None;
    }
}

pub fn reset_backend_stats() {
    let s = stats();
    s.backend_pushes.store(0, Ordering::Relaxed);
    s.backend_gateway_batches.store(0, Ordering::Relaxed);
    s.backend_gateway_events.store(0, Ordering::Relaxed);
}

pub fn snapshot_render_stats() -> RenderSnapshotStats {
    stats().render.lock().map(|r| r.clone()).unwrap_or_default()
}

#[derive(Debug, Clone)]
pub struct DebugSnapshot {
    pub uptime_secs: u64,
    pub mem: MemorySample,
    pub http_requests: u64,
    pub http_bytes: u64,
    pub http_errors: u64,
    pub http_unique_urls: usize,
    pub last_http_url: Option<String>,
    pub last_http_bytes: u64,
    pub render: RenderSnapshotStats,
    pub backend_pushes: u64,
    pub backend_gateway_batches: u64,
    pub backend_gateway_events: u64,
}

pub fn collect_snapshot() -> DebugSnapshot {
    let s = stats();
    DebugSnapshot {
        uptime_secs: s.started_at.elapsed().as_secs(),
        mem: read_memory_sample(),
        http_requests: s.http_requests.load(Ordering::Relaxed),
        http_bytes: s.http_bytes.load(Ordering::Relaxed),
        http_errors: s.http_errors.load(Ordering::Relaxed),
        http_unique_urls: s.http_unique_urls.lock().map(|set| set.len()).unwrap_or(0),
        last_http_url: s.last_http_url.lock().ok().and_then(|v| v.clone()),
        last_http_bytes: s.last_http_bytes.load(Ordering::Relaxed),
        render: snapshot_render_stats(),
        backend_pushes: s.backend_pushes.load(Ordering::Relaxed),
        backend_gateway_batches: s.backend_gateway_batches.load(Ordering::Relaxed),
        backend_gateway_events: s.backend_gateway_events.load(Ordering::Relaxed),
    }
}

pub fn maybe_log_periodic() {
    let s = stats();
    let should_log = s
        .last_log_at
        .lock()
        .map(|mut last| {
            if last.elapsed() >= Duration::from_secs(5) {
                *last = Instant::now();
                true
            } else {
                false
            }
        })
        .unwrap_or(false);
    if !should_log {
        return;
    }
    let snap = collect_snapshot();
    eprintln!(
        "[debug-stats] up={}s rss={}MB anon={}MB file={}MB shmem={}MB http(req={} bytes={}MB uniq={} err={}) render(frame={} imgs={} uniq={} guild={} avatars={} att={} embed={}) backend(pushes={} gw_batches={} gw_events={})",
        snap.uptime_secs,
        snap.mem.vm_rss_kb / 1024,
        snap.mem.rss_anon_kb / 1024,
        snap.mem.rss_file_kb / 1024,
        snap.mem.rss_shmem_kb / 1024,
        snap.http_requests,
        snap.http_bytes / (1024 * 1024),
        snap.http_unique_urls,
        snap.http_errors,
        snap.render.render_frame,
        snap.render.image_elements,
        snap.render.unique_image_urls,
        snap.render.guild_icons,
        snap.render.avatars,
        snap.render.attachment_images,
        snap.render.embed_images,
        snap.backend_pushes,
        snap.backend_gateway_batches,
        snap.backend_gateway_events,
    );
}

pub fn read_memory_sample() -> MemorySample {
    let mut out = MemorySample::default();
    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
        return out;
    };
    for line in status.lines() {
        let mut parts = line.split_whitespace();
        let Some(key) = parts.next() else { continue };
        let Some(val) = parts.next() else { continue };
        let kb = val.parse::<u64>().unwrap_or(0);
        match key {
            "VmRSS:" => out.vm_rss_kb = kb,
            "VmHWM:" => out.vm_hwm_kb = kb,
            "RssAnon:" => out.rss_anon_kb = kb,
            "RssFile:" => out.rss_file_kb = kb,
            "RssShmem:" => out.rss_shmem_kb = kb,
            _ => {}
        }
    }
    out
}

pub fn format_overlay_lines() -> Vec<String> {
    let snap = collect_snapshot();
    let mut lines = vec![
        format!(
            "MEM rss={}MB (anon={} file={} shmem={}) hwm={}MB",
            snap.mem.vm_rss_kb / 1024,
            snap.mem.rss_anon_kb / 1024,
            snap.mem.rss_file_kb / 1024,
            snap.mem.rss_shmem_kb / 1024,
            snap.mem.vm_hwm_kb / 1024
        ),
        format!(
            "HTTP req={} bytes={}MB uniq={} err={} last={}B",
            snap.http_requests,
            snap.http_bytes / (1024 * 1024),
            snap.http_unique_urls,
            snap.http_errors,
            snap.last_http_bytes
        ),
        format!(
            "RENDER frame={} imgs={} uniq={} (guild={} av={} att={} emb={})",
            snap.render.render_frame,
            snap.render.image_elements,
            snap.render.unique_image_urls,
            snap.render.guild_icons,
            snap.render.avatars,
            snap.render.attachment_images,
            snap.render.embed_images
        ),
        format!(
            "BACKEND pushes={} gw_batches={} gw_events={}",
            snap.backend_pushes, snap.backend_gateway_batches, snap.backend_gateway_events
        ),
    ];
    if let Some(url) = snap.last_http_url {
        let short = if url.len() > 110 {
            format!("{}…", &url[..110])
        } else {
            url
        };
        lines.push(format!("LAST {}", short));
    }
    lines.push("Toggle: F3  (logs every 5s while open)".to_owned());
    lines
}
