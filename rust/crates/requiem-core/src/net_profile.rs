use crate::properties::Platform;
use crate::properties;
use crate::rest;
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;
use std::time;
use tokio::net;

use wreq::header::{HeaderMap, HeaderName, HeaderValue, OrigHeaderMap};

use crate::rest::chromium_headers;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resource {
    Image,

    Video,

    Audio,

    Font,

    Script,

    Upload,
}

impl Resource {
    pub fn for_url(url: &str) -> Option<Resource> {
        let path = url.split(['?', '#']).next().unwrap_or(url);
        let extension = path
            .rsplit('/')
            .next()?
            .rsplit_once('.')?
            .1
            .to_ascii_lowercase();
        Some(match extension.as_str() {
            "js" => Resource::Script,
            "svg" | "png" | "jpg" | "jpeg" | "gif" | "webp" | "avif" | "apng" | "ico" => {
                Resource::Image
            }
            "mp3" | "ogg" | "wav" | "m4a" | "flac" | "opus" => Resource::Audio,
            "webm" | "mp4" | "mov" => Resource::Video,
            "woff2" | "woff" | "ttf" | "otf" => Resource::Font,
            _ => return None,
        })
    }

    pub fn http2_weight(self) -> u16 {
        match self {
            Resource::Font => 256,
            Resource::Upload => 220,
            Resource::Image | Resource::Video | Resource::Audio | Resource::Script => 147,
        }
    }
}

static ORIGIN: Mutex<Option<String>> = Mutex::new(None);
static ROUTE: Mutex<Option<String>> = Mutex::new(None);
static LOADED_AT_MS: OnceLock<u128> = OnceLock::new();

pub fn set_origin(origin: &str) {
    if let Ok(mut slot) = ORIGIN.lock() {
        *slot = Some(origin.trim_end_matches('/').to_string());
    }
}

pub fn origin() -> String {
    ORIGIN
        .lock()
        .ok()
        .and_then(|slot| slot.clone())
        .filter(|o| !o.is_empty())
        .unwrap_or_else(|| "https://discord.com".to_string())
}

pub fn set_route(path: &str) {
    if let Ok(mut slot) = ROUTE.lock() {
        *slot = Some(if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        });
    }
}

pub fn page_url() -> String {
    let route = ROUTE.lock().ok().and_then(|slot| slot.clone());
    match route {
        Some(route) => format!("{}{route}", origin()),
        None => {
            let loaded = *LOADED_AT_MS.get_or_init(|| {
                SystemTime::now()
                    .duration_since(time::UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0)
            });
            format!("{}/app?_={loaded}", origin())
        }
    }
}

fn host_of(url: &str) -> &str {
    url.split("://")
        .nth(1)
        .unwrap_or(url)
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UploadShape {
    Full,

    Resume { from: u64, size: u64 },

    Probe,
}

pub fn headers_for(url: &str, resource: Resource) -> (HeaderMap, OrigHeaderMap) {
    headers_for_upload(url, resource, UploadShape::Full)
}

pub fn headers_for_upload(
    url: &str,
    resource: Resource,
    shape: UploadShape,
) -> (HeaderMap, OrigHeaderMap) {
    let origin = origin();
    let same_origin = host_of(url) == host_of(&origin);
    let mut headers = HeaderMap::new();
    let mut put = |name: &'static str, value: &str| {
        if let Ok(value) = HeaderValue::from_str(value) {
            headers.insert(HeaderName::from_static(name), value);
        }
    };

    put(
        "sec-ch-ua-platform",
        Platform::current().client_hint_platform(),
    );
    put("user-agent", &properties::user_agent());
    put("sec-ch-ua", &properties::sec_ch_ua());
    put("sec-ch-ua-mobile", "?0");
    let (accept, dest, mode, priority): (&str, &str, &str, Option<&str>) = match resource {
        Resource::Image => (
            "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8",
            "image",
            "no-cors",
            Some("i"),
        ),
        Resource::Video => ("*/*", "video", "no-cors", Some("i")),
        Resource::Audio => ("*/*", "audio", "no-cors", Some("i")),
        Resource::Font => ("*/*", "font", "cors", Some("u=0")),
        Resource::Script => ("*/*", "script", "no-cors", None),
        Resource::Upload => ("*/*", "empty", "cors", Some("u=1, i")),
    };
    put("accept", accept);
    put(
        "sec-fetch-site",
        if same_origin {
            "same-origin"
        } else {
            "cross-site"
        },
    );
    put("sec-fetch-mode", mode);
    put("sec-fetch-dest", dest);

    if !same_origin && mode == "no-cors" {
        put("sec-fetch-storage-access", "active");
    }
    put(
        "referer",
        &if same_origin {
            page_url()
        } else {
            format!("{origin}/")
        },
    );
    let media = matches!(resource, Resource::Video | Resource::Audio);
    put(
        "accept-encoding",
        if media {
            "identity;q=1, *;q=0"
        } else {
            "gzip, deflate, br, zstd"
        },
    );
    put("accept-language", &properties::system_locale());
    if media {
        put("range", "bytes=0-");
    }
    if let Some(priority) = priority {
        put("priority", priority);
    }

    let origin_in_map = resource == Resource::Font;
    if origin_in_map || resource == Resource::Upload {
        put("origin", &origin);
    }
    if resource == Resource::Upload {
        if shape != UploadShape::Probe {
            put("content-type", "application/octet-stream");
        }
        match shape {
            UploadShape::Full => {}
            UploadShape::Resume { from, size } => put(
                "content-range",
                &format!("bytes {from}-{}/{size}", size.saturating_sub(1)),
            ),
            UploadShape::Probe => put("content-range", "bytes */*"),
        }
    }

    let mut map: Vec<&str> = Vec::new();
    if origin_in_map {
        map.push("origin");
    }
    if media {
        map.push("accept-encoding");
    }
    if resource == Resource::Upload {
        if shape != UploadShape::Probe {
            map.push("content-type");
        }
        if shape != UploadShape::Full {
            map.push("content-range");
        }
    }
    let mut tail: Vec<&str> = vec!["accept"];
    if resource == Resource::Upload {
        tail.push("origin");
    }
    tail.extend([
        "sec-fetch-site",
        "sec-fetch-mode",
        "sec-fetch-dest",
        "sec-fetch-storage-access",
        "referer",
    ]);
    if !media {
        tail.push("accept-encoding");
    }
    tail.extend(["accept-language", "cookie", "range", "priority"]);
    let blink_added: Vec<&str> = [
        "sec-ch-ua",
        "sec-ch-ua-mobile",
        "sec-ch-ua-platform",
        "user-agent",
    ]
    .to_vec();
    let order =
        chromium_headers::order_parts(&map, &blink_added, &tail, resource == Resource::Upload);
    let mut orig = OrigHeaderMap::new();
    for name in order {
        orig.insert(name);
    }
    (headers, orig)
}

pub fn websocket_handshake() -> (HeaderMap, OrigHeaderMap) {
    let mut headers = HeaderMap::new();
    let mut orig = OrigHeaderMap::new();
    let mut put = |name: &'static str, value: &str| {
        if let Ok(value) = HeaderValue::from_str(value) {
            headers.insert(HeaderName::from_static(name), value);
        }
    };
    put("connection", "Upgrade");
    put("pragma", "no-cache");
    put("cache-control", "no-cache");
    put("user-agent", &properties::user_agent());
    put("upgrade", "websocket");
    put("origin", &origin());
    put("sec-websocket-version", "13");
    put("accept-encoding", "gzip, deflate, br, zstd");
    put("accept-language", &properties::system_locale());
    put(
        "sec-websocket-extensions",
        "permessage-deflate; client_max_window_bits",
    );
    for name in [
        "Host",
        "Connection",
        "Pragma",
        "Cache-Control",
        "User-Agent",
        "Upgrade",
        "Origin",
        "Sec-WebSocket-Version",
        "Accept-Encoding",
        "Accept-Language",
        "Sec-WebSocket-Key",
        "Sec-WebSocket-Extensions",
    ] {
        orig.insert(name);
    }
    (headers, orig)
}

pub fn get(client: &wreq::Client, url: &str, resource: Resource) -> wreq::RequestBuilder {
    let (headers, order) = headers_for(url, resource);
    client.get(url).headers(headers).orig_headers(order)
}

pub fn get_from(
    client: &wreq::Client,
    url: &str,
    resource: Resource,
    start: u64,
) -> wreq::RequestBuilder {
    let (mut headers, order) = headers_for(url, resource);
    if let Ok(value) = HeaderValue::from_str(&format!("bytes={start}-")) {
        headers.insert(HeaderName::from_static("range"), value);
    }
    client.get(url).headers(headers).orig_headers(order)
}

const WARMUP_HOSTS: [&str; 4] = [
    "discord.com",
    "cdn.discordapp.com",
    "gateway.discord.gg",
    "updates.discord.com",
];

pub async fn warm_up_dns() {
    for host in WARMUP_HOSTS {
        let _ = net::lookup_host((host, 443)).await;
    }
}

pub fn decorate(
    builder: wreq::RequestBuilder,
    url: &str,
    resource: Resource,
) -> wreq::RequestBuilder {
    let (headers, order) = headers_for(url, resource);
    let builder = builder.emulation(rest::emulation_with_weight(resource.http2_weight()));
    builder.headers(headers).orig_headers(order)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(order: &OrigHeaderMap) -> Vec<String> {
        order
            .iter()
            .map(|(name, _)| name.as_str().to_string())
            .collect()
    }

    #[test]
    fn orders_match_the_real_clients_loads() {
        set_origin("https://discord.com");
        let want = |kind: &str| -> Vec<&str> {
            match kind {
                "img_cross" => "sec-ch-ua-platform,user-agent,sec-ch-ua,sec-ch-ua-mobile,accept,sec-fetch-site,sec-fetch-mode,sec-fetch-dest,sec-fetch-storage-access,referer,accept-encoding,accept-language,priority",
                "img_same" => "sec-ch-ua-platform,user-agent,sec-ch-ua,sec-ch-ua-mobile,accept,sec-fetch-site,sec-fetch-mode,sec-fetch-dest,referer,accept-encoding,accept-language,priority",
                "video_cross" => "sec-ch-ua-platform,accept-encoding,user-agent,sec-ch-ua,sec-ch-ua-mobile,accept,sec-fetch-site,sec-fetch-mode,sec-fetch-dest,sec-fetch-storage-access,referer,accept-language,range,priority",
                "audio_same" => "sec-ch-ua-platform,accept-encoding,user-agent,sec-ch-ua,sec-ch-ua-mobile,accept,sec-fetch-site,sec-fetch-mode,sec-fetch-dest,referer,accept-language,range,priority",
                "font_same" => "origin,sec-ch-ua-platform,user-agent,sec-ch-ua,sec-ch-ua-mobile,accept,sec-fetch-site,sec-fetch-mode,sec-fetch-dest,referer,accept-encoding,accept-language,priority",
                "script_same" => "sec-ch-ua-platform,user-agent,sec-ch-ua,sec-ch-ua-mobile,accept,sec-fetch-site,sec-fetch-mode,sec-fetch-dest,referer,accept-encoding,accept-language",
                "upload" => "content-length,sec-ch-ua-platform,user-agent,sec-ch-ua,content-type,sec-ch-ua-mobile,accept,origin,sec-fetch-site,sec-fetch-mode,sec-fetch-dest,referer,accept-encoding,accept-language,priority",
                _ => unreachable!(),
            }
            .split(',')
            .collect()
        };
        let cases = [
            (
                "img_cross",
                "https://cdn.discordapp.com/avatars/1/a.png?size=128",
                Resource::Image,
            ),
            (
                "img_same",
                "https://discord.com/assets/x.png",
                Resource::Image,
            ),
            (
                "video_cross",
                "https://cdn.discordapp.com/attachments/1/2/x.mp4",
                Resource::Video,
            ),
            (
                "audio_same",
                "https://discord.com/assets/x.mp3",
                Resource::Audio,
            ),
            (
                "font_same",
                "https://discord.com/assets/f.woff2",
                Resource::Font,
            ),
            (
                "script_same",
                "https://discord.com/assets/x.js",
                Resource::Script,
            ),
            (
                "upload",
                "https://discord-attachments-uploads-prd.storage.googleapis.com/attachments/1/2/x.png",
                Resource::Upload,
            ),
        ];
        for (kind, url, resource) in cases {
            let (headers, order) = headers_for(url, resource);
            let present: Vec<String> = names(&order)
                .into_iter()
                .filter(|n| headers.contains_key(n.as_str()) || n == "content-length")
                .collect();
            assert_eq!(present, want(kind), "{kind}");
        }
    }

    #[test]
    fn extensions_pick_the_load_kind() {
        assert_eq!(
            Resource::for_url("https://discord.com/assets/a.woff2?x=1"),
            Some(Resource::Font)
        );
        assert_eq!(
            Resource::for_url("https://discord.com/assets/50dd.js"),
            Some(Resource::Script)
        );
        assert_eq!(Resource::for_url("https://discord.com/assets/"), None);
    }

    #[test]
    fn upload_shapes_differ_in_content_type_and_content_range() {
        set_origin("https://discord.com");
        let url =
            "https://discord-attachments-uploads-prd.storage.googleapis.com/attachments/1/2/x.png";
        let (full, _) = headers_for_upload(url, Resource::Upload, UploadShape::Full);
        assert!(full.contains_key("content-type") && !full.contains_key("content-range"));
        let (resume, order) = headers_for_upload(
            url,
            Resource::Upload,
            UploadShape::Resume {
                from: 400,
                size: 1000,
            },
        );
        assert_eq!(resume["content-range"], "bytes 400-999/1000");
        assert!(resume.contains_key("content-type"));
        assert!(
            names(&order).contains(&"content-range".to_string())
                && names(&order)[0] == "content-length"
        );
        let (probe, _) = headers_for_upload(url, Resource::Upload, UploadShape::Probe);
        assert_eq!(probe["content-range"], "bytes */*");
        assert!(
            !probe.contains_key("content-type"),
            "no body, no content type"
        );
    }
}
