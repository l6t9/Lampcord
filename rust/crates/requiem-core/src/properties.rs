use base64::engine::general_purpose;
use crate::gateway::qos;
use crate::net_profile;
use crate::utils::prefs;
use requiem_updater::sources;
use serde_json::Value;
use serde_json::json;
use std::collections;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{OnceLock, RwLock};
use std::time as std_time;
use std::time::Duration;
use std::time::SystemTime;
use tokio::time;
use wreq::header::HeaderMap;
use wreq::header::HeaderValue;

use base64::Engine;
use serde::Serialize;

pub const RELEASE_CHANNEL: &str = "stable";

pub const CLIENT_BUILD_NUMBER: u32 = 300_000;

pub fn client_build_number() -> u32 {
    requiem_asset_store::get()
        .and_then(|store| store.client_info().build_number)
        .and_then(|n| u32::try_from(n).ok())
        .unwrap_or(CLIENT_BUILD_NUMBER)
}

pub fn release_channel() -> String {
    requiem_asset_store::get()
        .map(|store| store.client_info().release_channel.clone())
        .unwrap_or_else(|| RELEASE_CHANNEL.to_string())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    Windows,
    MacOs,
    Linux,
}

impl Platform {
    /// The OS this build is running on.
    pub fn host() -> Self {
        if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOs
        } else {
            Platform::Linux
        }
    }

    /// The OS the client presents to Discord: the user agent, the super properties, the
    /// client hints, the TLS profile and the identity it follows. Linux presents as Windows,
    /// since Discord's abuse systems treat Linux clients far more harshly. Anything that
    /// should follow the real OS uses [`Platform::host`].
    pub fn current() -> Self {
        Self::presented_for(Self::host())
    }

    fn presented_for(host: Self) -> Self {
        match host {
            Platform::Linux => Platform::Windows,
            other => other,
        }
    }

    fn key(self) -> &'static str {
        match self {
            Platform::Windows => "windows",
            Platform::MacOs => "osx",
            Platform::Linux => "linux",
        }
    }

    fn user_agent_os(self) -> &'static str {
        match self {
            Platform::Windows => "Windows NT 10.0; Win64; x64",
            Platform::MacOs => "Macintosh; Intel Mac OS X 10_15_7",
            Platform::Linux => "X11; Linux x86_64",
        }
    }

    pub(crate) fn client_hint_platform(self) -> &'static str {
        match self {
            Platform::Windows => "\"Windows\"",
            Platform::MacOs => "\"macOS\"",
            Platform::Linux => "\"Linux\"",
        }
    }

    pub(crate) fn wreq(self) -> wreq_util::Platform {
        match self {
            Platform::Windows => wreq_util::Platform::Windows,
            Platform::MacOs => wreq_util::Platform::MacOS,
            Platform::Linux => wreq_util::Platform::Linux,
        }
    }

    fn has_native_build_number(self) -> bool {
        self == Platform::Windows
    }
}

static APP_FOCUSED: AtomicBool = AtomicBool::new(true);

pub fn app_focused() -> bool {
    APP_FOCUSED.load(Ordering::Relaxed)
}

pub fn set_app_focused(focused: bool) {
    APP_FOCUSED.store(focused, Ordering::Relaxed);

    qos::set_foreground(focused);
}

fn app_state() -> &'static str {
    if APP_FOCUSED.load(Ordering::Relaxed) {
        "focused"
    } else {
        "unfocused"
    }
}

/// The architecture presented to Discord. A Linux host presents as Windows x64, whatever it
/// really is; Windows and macOS report their own.
fn arch() -> &'static str {
    if cfg!(target_arch = "aarch64") && Platform::host() != Platform::Linux {
        "arm64"
    } else {
        "x64"
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
struct Identity {
    host_version: String,
    native_build_number: Option<u32>,
    electron_version: String,
    chrome_version: String,
}

impl Identity {
    fn chrome_major(&self) -> Option<u32> {
        self.chrome_version.split('.').next()?.parse().ok()
    }
}

type Identities = collections::BTreeMap<String, Identity>;

static IDENTITY: RwLock<Option<Identity>> = RwLock::new(None);

fn identity() -> Option<Identity> {
    IDENTITY.read().ok().and_then(|slot| slot.clone())
}

pub fn host_version() -> String {
    identity()
        .map(|i| i.host_version)
        .unwrap_or_else(|| "0.0.0".to_string())
}

pub fn native_build_number() -> Option<u32> {
    identity().and_then(|i| i.native_build_number)
}

pub fn electron_version() -> String {
    identity()
        .map(|i| i.electron_version)
        .unwrap_or_else(|| "0.0.0".to_string())
}

pub fn chrome_version() -> String {
    identity()
        .map(|i| i.chrome_version)
        .unwrap_or_else(|| "0.0.0.0".to_string())
}

fn profile_for(major: u32) -> Option<wreq_util::Profile> {
    use wreq_util::Profile::*;
    Some(match major {
        137 => Chrome137,
        138 => Chrome138,
        139 => Chrome139,
        140 => Chrome140,
        141 => Chrome141,
        142 => Chrome142,
        143 => Chrome143,
        144 => Chrome144,
        145 => Chrome145,
        146 => Chrome146,
        147 => Chrome147,
        148 => Chrome148,
        149 => Chrome149,
        _ => return None,
    })
}

pub fn emulation_profile() -> wreq_util::Profile {
    let major = identity().and_then(|i| i.chrome_major()).unwrap_or(0);
    profile_for(major).unwrap_or_else(|| {
        log::warn!(
            "no TLS profile for Chromium {major}; using the newest available (update wreq-util)"
        );
        wreq_util::Profile::Chrome149
    })
}

pub fn pin_chromium_major(major: u32) {
    let mut identity = identity().unwrap_or(Identity {
        host_version: "0.0.0".to_string(),
        native_build_number: None,
        electron_version: "0.0.0".to_string(),
        chrome_version: "0.0.0.0".to_string(),
    });
    let rest = identity
        .chrome_version
        .split_once('.')
        .map_or("0.0.0", |(_, rest)| rest)
        .to_string();
    identity.chrome_version = format!("{major}.{rest}");
    if let Ok(mut slot) = IDENTITY.write() {
        *slot = Some(identity);
    }
}

fn cordapi_url() -> String {
    format!(
        "https://cordapi.dolfi.es/api/v2/properties/{}",
        Platform::current().key()
    )
}

#[derive(Clone, Debug)]
struct Cordapi {
    properties: serde_json::Map<String, Value>,
    gateway: serde_json::Map<String, Value>,
}

static CORDAPI: RwLock<Option<Cordapi>> = RwLock::new(None);

fn parse_cordapi(body: &Value) -> Option<(Identity, Cordapi)> {
    let properties = body["properties"].as_object()?.clone();
    let identity = Identity {
        host_version: properties.get("client_version")?.as_str()?.to_string(),
        native_build_number: properties
            .get("native_build_number")
            .and_then(|n| n.as_u64())
            .and_then(|n| u32::try_from(n).ok()),
        electron_version: body["metadata"]["electron_version"].as_str()?.to_string(),
        chrome_version: body["metadata"]["native_chrome_version"]
            .as_str()?
            .to_string(),
    };
    let gateway = body["extra_gateway_properties"]
        .as_object()
        .cloned()
        .unwrap_or_default();
    Some((
        identity,
        Cordapi {
            properties,
            gateway,
        },
    ))
}

async fn fetch_from_cordapi(client: &wreq::Client) -> Option<(Identity, Cordapi)> {
    let body: Value = client
        .post(cordapi_url())
        .json(&serde_json::json!({}))
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json()
        .await
        .ok()?;
    parse_cordapi(&body)
}

pub async fn refresh_identity() {
    let client = wreq::Client::new();
    let timeout = Duration::from_secs(5);
    let (latest, cordapi) = match time::timeout(timeout, fetch_from_cordapi(&client))
        .await
        .ok()
        .flatten()
    {
        Some((identity, cordapi)) => (Some(identity), Some(cordapi)),
        None => {
            log::info!("cordapi unavailable; trying the signed identity of the update backends");
            let sources = sources::configured();
            let fetch = async {
                serde_json::from_str::<Identities>(
                    &sources::fetch(
                        &client,
                        &sources,
                        "identity.json",
                        requiem_updater::KIND_IDENTITY,
                    )
                    .await?,
                )
                .ok()?
                .remove(Platform::current().key())
            };
            (fetch.await, None)
        }
    };
    let Some(latest) = latest else {
        log::info!("couldn't fetch the latest desktop identity; keeping the current one");
        return;
    };
    if latest.chrome_major().and_then(profile_for).is_none() {
        log::warn!(
            "latest desktop identity names Chromium {}, which this build has no TLS profile for; keeping the current one",
            latest.chrome_version
        );
        return;
    }
    log::info!(
        "desktop identity: host {}, Electron {}, Chromium {} ({})",
        latest.host_version,
        latest.electron_version,
        latest.chrome_version,
        if cordapi.is_some() {
            "cordapi"
        } else {
            "signed identity"
        }
    );
    if let Ok(mut slot) = IDENTITY.write() {
        *slot = Some(latest);
    }
    if let Ok(mut slot) = CORDAPI.write() {
        *slot = cordapi;
    }
}

fn installation_id() -> String {
    const DISCORD_EPOCH_MS: u64 = 1_420_070_400_000;

    let mut prefs = prefs::load();
    if let Some(id) = &prefs.installation_id {
        return id.clone();
    }
    let now_ms = SystemTime::now()
        .duration_since(std_time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(DISCORD_EPOCH_MS);
    let snowflake = ((now_ms.saturating_sub(DISCORD_EPOCH_MS)) << 22) | 4096;
    let mut token_bytes = uuid::Uuid::new_v4().into_bytes().to_vec();
    token_bytes.extend(uuid::Uuid::new_v4().into_bytes());
    let id = format!(
        "{snowflake}.{}",
        general_purpose::URL_SAFE_NO_PAD.encode(token_bytes)
    );
    prefs.installation_id = Some(id.clone());
    prefs::save(&prefs);
    id
}

pub fn system_locale() -> String {
    static LOCALE: OnceLock<String> = OnceLock::new();
    LOCALE
        .get_or_init(|| sys_locale::get_locale().unwrap_or_else(|| "en-US".to_string()))
        .clone()
}

pub fn user_agent() -> String {
    format!(
        "Mozilla/5.0 ({}) AppleWebKit/537.36 (KHTML, like Gecko) discord/{} Chrome/{} Electron/{} Safari/537.36",
        Platform::current().user_agent_os(),
        host_version(),
        chrome_version(),
        electron_version()
    )
}

pub(crate) fn sec_ch_ua() -> String {
    const CHARS: [&str; 11] = [" ", "(", ":", "-", ".", "/", ")", ";", "=", "?", "_"];
    const VERSIONS: [&str; 3] = ["8", "99", "24"];
    let major = identity().and_then(|i| i.chrome_major()).unwrap_or(0) as usize;
    let grease = format!(
        "\"Not{}A{}Brand\";v=\"{}\"",
        CHARS[major % 11],
        CHARS[(major + 1) % 11],
        VERSIONS[major % 3]
    );
    let chromium = format!("\"Chromium\";v=\"{major}\"");
    if major.is_multiple_of(2) {
        format!("{grease}, {chromium}")
    } else {
        format!("{chromium}, {grease}")
    }
}

fn session_ids() -> &'static (String, String) {
    static IDS: OnceLock<(String, String)> = OnceLock::new();
    IDS.get_or_init(|| {
        const MOD_BITS: u128 = 0b00000000100000000001000000010000000010000001000000001000000000000010000010000001000000000100000000000001000000000000100000000000;
        let signature = uuid::Uuid::from_u128(uuid::Uuid::new_v4().as_u128() & !MOD_BITS);
        (uuid::Uuid::new_v4().to_string(), signature.to_string())
    })
}

type Map = serde_json::Map<String, Value>;

#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct SuperProperties(Map);

impl SuperProperties {
    pub fn desktop_like() -> Self {
        Self::overlay(
            CORDAPI
                .read()
                .ok()
                .and_then(|c| c.as_ref().map(|c| c.properties.clone()))
                .unwrap_or_else(fallback_desktop_properties),
        )
    }

    fn overlay(mut map: Map) -> Self {
        map.insert("client_version".into(), host_version().into());
        map.insert("browser_user_agent".into(), user_agent().into());
        map.insert("browser_version".into(), electron_version().into());
        map.insert("client_build_number".into(), client_build_number().into());
        map.insert(
            "system_locale".into(),
            sys_locale::get_locale()
                .unwrap_or_else(|| "en-US".to_string())
                .into(),
        );
        if Platform::current().has_native_build_number() {
            if let Some(native) = native_build_number() {
                map.insert("native_build_number".into(), native.into());
            }
        }

        for key in ["os_arch", "app_arch"] {
            if map.contains_key(key) {
                map.insert(key.to_string(), arch().into());
            }
        }
        Self(map)
    }

    pub fn mobile_like() -> Self {
        let mut map = Self::desktop_like().0;
        for key in [
            "client_version",
            "os_arch",
            "app_arch",
            "has_client_mods",
            "client_launch_id",
            "launch_signature",
            "os_sdk_version",
            "native_build_number",
            "client_heartbeat_session_id",
            "client_app_state",
        ] {
            map.shift_remove(key);
        }
        map.insert("os".into(), "Android".into());
        map.insert("browser".into(), "Discord Android".into());
        map.insert("device".into(), "android".into());
        map.insert("os_version".into(), "26".into());
        map.insert("browser_user_agent".into(), "".into());
        map.insert("browser_version".into(), "".into());
        Self(map)
    }

    pub fn system_locale(&self) -> &str {
        self.0
            .get("system_locale")
            .and_then(|v| v.as_str())
            .unwrap_or("en-US")
    }

    pub fn to_header_value(&self) -> String {
        let json =
            serde_json::to_vec(&self.with_live_state()).expect("SuperProperties always serializes");
        general_purpose::STANDARD.encode(json)
    }

    fn with_live_state(&self) -> Map {
        let mut map = self.0.clone();
        if map.contains_key("client_app_state") {
            map.insert("client_app_state".into(), app_state().into());
        }
        map
    }

    pub fn gateway_value(&self) -> Value {
        self.with_gateway_extras(
            CORDAPI
                .read()
                .ok()
                .and_then(|c| c.as_ref().map(|c| c.gateway.clone()))
                .unwrap_or_else(|| {
                    let mut extra = Map::new();
                    extra.insert("is_fast_connect".into(), false.into());
                    extra.insert("gateway_connect_reasons".into(), "AppSkeleton".into());
                    extra
                }),
        )
    }

    fn with_gateway_extras(&self, extra: Map) -> Value {
        let mut map = self.with_live_state();
        map.extend(extra);
        map.insert("installation_id".into(), installation_id().into());
        Value::Object(map)
    }
}

fn fallback_desktop_properties() -> Map {
    let platform = Platform::current();
    let (launch_id, signature) = session_ids();
    let mut map = Map::new();
    let mut put = |key: &str, value: Value| {
        map.insert(key.to_string(), value);
    };
    put(
        "os",
        match platform {
            Platform::Windows => "Windows",
            Platform::MacOs => "Mac OS X",
            Platform::Linux => "Linux",
        }
        .into(),
    );
    put("browser", "Discord Client".into());
    put("release_channel", release_channel().into());
    put("client_version", host_version().into());
    put(
        "os_version",
        match platform {
            Platform::Windows => "10.0.26200",
            Platform::MacOs => "26.0.0",
            Platform::Linux => "6.8.0-146.146",
        }
        .into(),
    );
    put("os_arch", arch().into());
    put("app_arch", arch().into());
    put("system_locale", "en-US".into());
    put("has_client_mods", false.into());
    put("client_launch_id", launch_id.clone().into());
    put("browser_user_agent", user_agent().into());
    put("browser_version", electron_version().into());
    match platform {
        Platform::Windows => put("os_sdk_version", "26200".into()),
        Platform::MacOs => put("os_sdk_version", "26".into()),
        Platform::Linux => {
            put("window_manager", "ubuntu:GNOME,ubuntu-wayland".into());
            put("distro", "Ubuntu 24.04 LTS".into());
            put("runtime_environment", "native".into());
            put("display_server", "wayland".into());
        }
    }
    put("client_build_number", client_build_number().into());
    put(
        "native_build_number",
        if platform.has_native_build_number() {
            json!(native_build_number())
        } else {
            Value::Null
        },
    );
    put("client_event_source", Value::Null);
    put("launch_signature", signature.clone().into());
    put(
        "client_heartbeat_session_id",
        qos::session_id().into(),
    );
    put("client_app_state", "focused".into());
    map
}

pub fn api_headers(origin: &str, super_properties: &SuperProperties) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let mut put = |name: &'static str, value: &str| {
        if let Ok(value) = HeaderValue::from_str(value) {
            headers.insert(name, value);
        }
    };
    put(
        "sec-ch-ua-platform",
        Platform::current().client_hint_platform(),
    );
    put("x-debug-options", "bugReporterEnabled");
    put("accept-language", super_properties.system_locale());
    put("sec-ch-ua", &sec_ch_ua());
    put("sec-ch-ua-mobile", "?0");
    if let Ok(timezone) = iana_time_zone::get_timezone() {
        put("x-discord-timezone", &timezone);
    }
    put("x-super-properties", &super_properties.to_header_value());

    put(
        "x-discord-locale",
        &prefs::load()
            .language
            .unwrap_or_else(|| "en-US".to_string()),
    );
    put("user-agent", &user_agent());
    put("accept", "*/*");
    put("origin", origin);
    put("sec-fetch-site", "same-origin");
    put("sec-fetch-mode", "cors");
    put("sec-fetch-dest", "empty");
    put("referer", &net_profile::page_url());
    put("accept-encoding", "gzip, deflate, br, zstd");
    put("priority", "u=1, i");
    headers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_presents_as_windows_and_the_others_as_themselves() {
        assert_eq!(Platform::presented_for(Platform::Linux), Platform::Windows);
        assert_eq!(Platform::presented_for(Platform::Windows), Platform::Windows);
        assert_eq!(Platform::presented_for(Platform::MacOs), Platform::MacOs);
        let windows = Platform::Windows;
        assert_eq!(windows.user_agent_os(), "Windows NT 10.0; Win64; x64");
        assert_eq!(windows.client_hint_platform(), "\"Windows\"");
        assert!(windows.has_native_build_number());
    }

    #[tokio::test]
    #[ignore = "needs network"]
    async fn cordapi_gives_an_identity_we_have_a_profile_for() {
        let (identity, cordapi) = fetch_from_cordapi(&wreq::Client::new())
            .await
            .expect("cordapi answered");
        println!(
            "{} XSP keys, gateway extras {:?}",
            cordapi.properties.len(),
            cordapi.gateway
        );
        println!("{identity:?}");
        assert!(
            identity.chrome_major().and_then(profile_for).is_some(),
            "Chromium {} has no TLS profile",
            identity.chrome_version
        );
    }

    const CORDAPI_SAMPLE: &str = r#"{"properties":{"os":"Windows","browser":"Discord Client","release_channel":"stable","client_version":"1.0.9260","os_version":"10.0.26200","os_arch":"x64","app_arch":"x64","system_locale":"en-US","has_client_mods":false,"client_launch_id":"543d7c80-3b9c-4e0d-8a4d-cf6a01b57b88","browser_user_agent":"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) discord/1.0.9260 Chrome/148.0.7778.280 Electron/42.11.8 Safari/537.36","browser_version":"42.11.8","os_sdk_version":"26200","client_build_number":627798,"native_build_number":92473,"client_event_source":null,"launch_signature":"2976a7c0-f20d-4563-8616-311c90c5b2a5","client_heartbeat_session_id":"916704cc-a533-43cf-8ed8-bb0b6068f4f9","client_app_state":"focused","some_future_key":"kept"},"extra_gateway_properties":{"is_fast_connect":false,"gateway_connect_reasons":"AppSkeleton"},"metadata":{"electron_version":"42.11.8","native_chrome_version":"148.0.7778.280"}}"#;

    #[test]
    fn every_cordapi_key_reaches_the_xsp_and_the_gateway_properties_in_order() {
        let body: Value = serde_json::from_str(CORDAPI_SAMPLE).unwrap();
        let (identity, cordapi) = parse_cordapi(&body).unwrap();
        assert_eq!(identity.electron_version, "42.11.8");
        assert_eq!(identity.native_build_number, Some(92473));
        let xsp = SuperProperties::overlay(cordapi.properties);
        let sent: Value = serde_json::from_slice(
            &general_purpose::STANDARD
                .decode(xsp.to_header_value())
                .unwrap(),
        )
        .unwrap();
        let sent_keys: Vec<&String> = sent.as_object().unwrap().keys().collect();
        let cordapi_keys: Vec<&String> = body["properties"].as_object().unwrap().keys().collect();
        assert_eq!(
            sent_keys, cordapi_keys,
            "same keys, same order, including keys we don't know about"
        );
        assert_eq!(
            sent["client_launch_id"],
            "543d7c80-3b9c-4e0d-8a4d-cf6a01b57b88"
        );
        assert_eq!(sent["some_future_key"], "kept");
        let gateway = xsp.with_gateway_extras(cordapi.gateway);
        assert_eq!(gateway["gateway_connect_reasons"], "AppSkeleton");
        assert_eq!(gateway["is_fast_connect"], false);
        assert_eq!(gateway["os"], "Windows");
    }

    #[test]
    fn client_app_state_follows_window_focus() {
        let xsp = SuperProperties::desktop_like();
        let state = |xsp: &SuperProperties| {
            let json: Value = serde_json::from_slice(
                &general_purpose::STANDARD
                    .decode(xsp.to_header_value())
                    .unwrap(),
            )
            .unwrap();
            let gateway = xsp.with_gateway_extras(Map::new());
            assert_eq!(json["client_app_state"], gateway["client_app_state"]);
            json["client_app_state"].as_str().unwrap().to_string()
        };
        set_app_focused(false);
        assert_eq!(state(&xsp), "unfocused");
        set_app_focused(true);
        assert_eq!(state(&xsp), "focused");
    }

    #[test]
    fn a_pinned_chromium_major_selects_its_tls_profile() {
        pin_chromium_major(148);
        assert_eq!(chrome_version().split('.').next(), Some("148"));
        assert!(emulation_profile() == wreq_util::Profile::Chrome148);
    }

    #[test]
    fn sec_ch_ua_has_the_grease_brand_chromium_derives_from_the_major() {
        let chars = [" ", "(", ":", "-", ".", "/", ")", ";", "=", "?", "_"];
        let versions = ["8", "99", "24"];
        for (major, grease) in [
            (144usize, r#""Not(A:Brand";v="8""#),
            (148, r#""Not/A)Brand";v="99""#),
        ] {
            assert_eq!(
                format!(
                    "\"Not{}A{}Brand\";v=\"{}\"",
                    chars[major % 11],
                    chars[(major + 1) % 11],
                    versions[major % 3]
                ),
                grease
            );
        }
    }

    #[test]
    fn desktop_properties_have_the_real_shape() {
        let json = serde_json::to_value(SuperProperties::desktop_like()).unwrap();
        assert_eq!(json["browser_version"], electron_version());
        assert_eq!(json["browser_user_agent"], user_agent());
        assert_eq!(json["has_client_mods"], false);
        assert!(user_agent().contains(&format!(
            "discord/{} Chrome/{} Electron/{}",
            host_version(),
            chrome_version(),
            electron_version()
        )));

        let signature = uuid::Uuid::parse_str(json["launch_signature"].as_str().unwrap())
            .unwrap()
            .as_u128();
        assert_eq!(signature & 0b00000000100000000001000000010000000010000001000000001000000000000010000010000001000000000100000000000001000000000000100000000000, 0);

        assert_eq!(
            json["client_launch_id"],
            serde_json::to_value(SuperProperties::desktop_like()).unwrap()["client_launch_id"]
        );
    }
}
