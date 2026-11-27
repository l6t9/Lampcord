use requiem_core::config;
use requiem_core::models;
use skia_safe::surfaces;
use std::collections::{HashMap, VecDeque};
use std::env;
use std::fs;
use std::path;
use std::sync::{Arc, Mutex, OnceLock};
#[cfg(windows)]
use windows::UI::Notifications;

use requiem_core::models::Id;
use requiem_core::models::id::marker::ChannelMarker;
use requiem_core::notifications::Notification;

const MAX_PER_CHANNEL: usize = 20;

#[cfg(windows)]
const AUMID: &str = "Requiem.Client";

/// The name notifications are shown under.
const APP_NAME: &str = "Requiem";

type OnClick = Arc<dyn Fn(Id<ChannelMarker>) + Send + Sync>;

static ON_CLICK: OnceLock<OnClick> = OnceLock::new();

#[cfg(windows)]
type Shown = Notifications::ToastNotification;
#[cfg(not(windows))]
type Shown = ();

static SHOWN: Mutex<Option<HashMap<Id<ChannelMarker>, VecDeque<(String, Shown)>>>> =
    Mutex::new(None);

static BACKEND: Mutex<Option<config::BackendConfig>> = Mutex::new(None);

pub fn set_backend(backend: config::BackendConfig) {
    *BACKEND.lock().expect("notifications poisoned") = Some(backend);
}

pub fn avatar_url(message: &models::Message) -> Option<String> {
    let hash = message.author.avatar.as_deref()?;
    Some(
        BACKEND
            .lock()
            .expect("notifications poisoned")
            .as_ref()?
            .avatar_url(message.author.id, hash),
    )
}

pub fn init(on_click: impl Fn(Id<ChannelMarker>) + Send + Sync + 'static) {
    let _ = ON_CLICK.set(Arc::new(on_click));
    #[cfg(windows)]
    platform::register();
}

pub fn os_accepts() -> bool {
    #[cfg(windows)]
    {
        platform::accepts()
    }
    #[cfg(not(windows))]
    {
        true
    }
}

pub fn show(notification: &Notification, avatar: Option<&skia_safe::Image>) {
    #[cfg(windows)]
    {
        let logo =
            avatar.and_then(|image| avatar_file(image, notification.author_id.get()));
        match platform::show(notification, logo.as_deref()) {
            Ok(toast) => {
                let mut shown = SHOWN.lock().expect("notifications poisoned");
                let list = shown
                    .get_or_insert_with(HashMap::new)
                    .entry(notification.channel_id)
                    .or_default();
                list.push_back((notification.message_id.get().to_string(), toast));
                while list.len() > MAX_PER_CHANNEL {
                    if let Some((tag, _)) = list.pop_front() {
                        platform::remove(&tag, notification.channel_id);
                    }
                }
            }
            Err(err) => log::warn!("could not show a notification: {err}"),
        }
    }
    #[cfg(not(windows))]
    {
        let logo = avatar.and_then(|image| avatar_file(image, notification.author_id.get()));
        if let Err(err) = desktop::show(notification, logo.as_deref()) {
            log::warn!("could not show a notification: {err}");
        }
    }
}

pub fn avatar_file(image: &skia_safe::Image, user_id: u64) -> Option<path::PathBuf> {
    let size = 128;
    let mut surface = surfaces::raster_n32_premul((size, size))?;
    let canvas = surface.canvas();
    canvas.clear(skia_safe::Color::TRANSPARENT);
    let rect = skia_safe::Rect::from_wh(size as f32, size as f32);
    canvas.clip_rrect(
        skia_safe::RRect::new_oval(rect),
        skia_safe::ClipOp::Intersect,
        true,
    );
    let paint = skia_safe::Paint::default();
    canvas.draw_image_rect_with_sampling_options(
        image,
        None,
        rect,
        skia_safe::SamplingOptions::new(
            skia_safe::FilterMode::Linear,
            skia_safe::MipmapMode::None,
        ),
        &paint,
    );
    let png =
        surface
            .image_snapshot()
            .encode(None, skia_safe::EncodedImageFormat::PNG, None)?;
    let dir = env::temp_dir().join("requiem-notifications");
    fs::create_dir_all(&dir).ok()?;
    let path = dir.join(format!("{user_id}.png"));
    fs::write(&path, png.as_bytes()).ok()?;
    Some(path)
}

pub fn clear_channel(channel_id: Id<ChannelMarker>) {
    let had = SHOWN
        .lock()
        .expect("notifications poisoned")
        .as_mut()
        .and_then(|shown| shown.remove(&channel_id))
        .is_some();
    #[cfg(windows)]
    if had {
        platform::remove_group(channel_id);
    }
    #[cfg(not(windows))]
    let _ = had;
}

/// Notifications where there is no Windows toast API: the desktop's own notifier, run as a
/// child process. They carry no click handler, so they don't jump to the channel.
#[cfg(not(windows))]
mod desktop {
    use std::path::Path;
    use std::process::{Command, Stdio};

    use requiem_core::notifications::Notification;

    #[cfg(target_os = "macos")]
    fn applescript_string(text: &str) -> String {
        format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
    }

    #[cfg(target_os = "macos")]
    pub fn show(notification: &Notification, _logo: Option<&Path>) -> std::io::Result<()> {
        let script = format!(
            "display notification {} with title {}",
            applescript_string(&notification.body),
            applescript_string(&notification.title)
        );
        Command::new("osascript")
            .arg("-e")
            .arg(script)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
    }

    #[cfg(not(target_os = "macos"))]
    pub fn show(notification: &Notification, logo: Option<&Path>) -> std::io::Result<()> {
        let mut command = Command::new("notify-send");
        command
            .arg(format!("--app-name={}", super::APP_NAME))
            .arg("--category=im.received")
            // A newer message from the same channel replaces the one on screen.
            .arg(format!(
                "--hint=string:x-canonical-private-synchronous:requiem-{}",
                notification.channel_id.get()
            ));
        if let Some(logo) = logo {
            command.arg(format!("--icon={}", logo.display()));
        }
        command
            .arg("--")
            .arg(&notification.title)
            .arg(&notification.body)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
    }
}

#[cfg(windows)]
mod platform {
    use requiem_core::models::Id;
    use requiem_core::models::id::marker::ChannelMarker;
    use requiem_core::notifications::Notification;
    use std::path;
    use std::ptr;
    use windows::Data::Xml::Dom::XmlDocument;
    use windows::Foundation::TypedEventHandler;
    use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};
    use windows::core::HSTRING;
    use windows::core;
    use windows_sys::Win32::System::Registry::HKEY;
    use windows_sys::Win32::System::Registry::HKEY_CURRENT_USER;
    use windows_sys::Win32::System::Registry::REG_SZ;
    use windows_sys::Win32::System::Registry::RegCloseKey;
    use windows_sys::Win32::System::Registry::RegCreateKeyW;
    use windows_sys::Win32::System::Registry::RegSetValueExW;
    use windows_sys::Win32::UI::Shell::QUNS_BUSY;
    use windows_sys::Win32::UI::Shell::QUNS_PRESENTATION_MODE;
    use windows_sys::Win32::UI::Shell::QUNS_QUIET_TIME;
    use windows_sys::Win32::UI::Shell::QUNS_RUNNING_D3D_FULL_SCREEN;
    use windows_sys::Win32::UI::Shell::SHQueryUserNotificationState;
    use windows_sys::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;

    use super::{APP_NAME, AUMID, ON_CLICK};

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }

    pub fn register() {
        let path = wide(&format!("Software\\Classes\\AppUserModelId\\{AUMID}"));
        let mut key: HKEY = ptr::null_mut();

        // SAFETY: every pointer is a NUL-terminated UTF-16 buffer (or the key handle just returned
        // by RegCreateKeyW) that outlives its call, and `name`'s byte length matches the buffer.
        unsafe {
            if RegCreateKeyW(HKEY_CURRENT_USER, path.as_ptr(), &mut key) == 0 {
                let name = wide(APP_NAME);
                RegSetValueExW(
                    key,
                    wide("DisplayName").as_ptr(),
                    0,
                    REG_SZ,
                    name.as_ptr().cast(),
                    (name.len() * 2) as u32,
                );
                RegCloseKey(key);
            } else {
                log::warn!("could not register the notification identity");
            }
            SetCurrentProcessExplicitAppUserModelID(wide(AUMID).as_ptr());
        }
    }

    pub fn accepts() -> bool {
        let mut state = 0;

        // SAFETY: SHQueryUserNotificationState writes into a valid local.
        if unsafe { SHQueryUserNotificationState(&mut state) } != 0 {
            return true;
        }
        !matches!(
            state,
            QUNS_BUSY | QUNS_RUNNING_D3D_FULL_SCREEN | QUNS_PRESENTATION_MODE | QUNS_QUIET_TIME
        )
    }

    fn escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }

    pub fn show(
        notification: &Notification,
        logo: Option<&path::Path>,
    ) -> core::Result<ToastNotification> {
        let image = logo
            .map(|path| {
                format!(
                    r#"<image placement="appLogoOverride" src="file:///{}"/>"#,
                    escape(&path.to_string_lossy().replace('\\', "/"))
                )
            })
            .unwrap_or_default();
        let xml = format!(
            r#"<toast><visual><binding template="ToastGeneric"><text>{}</text><text>{}</text>{image}</binding></visual><audio silent="true"/></toast>"#,
            escape(&notification.title),
            escape(&notification.body),
        );
        let document = XmlDocument::new()?;
        document.LoadXml(&HSTRING::from(xml))?;
        let toast = ToastNotification::CreateToastNotification(&document)?;
        toast.SetTag(&HSTRING::from(notification.message_id.get().to_string()))?;
        toast.SetGroup(&HSTRING::from(notification.channel_id.get().to_string()))?;
        let channel_id = notification.channel_id;
        toast.Activated(&TypedEventHandler::new(move |_, _| {
            if let Some(on_click) = ON_CLICK.get() {
                on_click(channel_id);
            }
            Ok(())
        }))?;
        ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(AUMID))?.Show(&toast)?;
        Ok(toast)
    }

    pub fn remove(tag: &str, channel_id: Id<ChannelMarker>) {
        if let Ok(history) = ToastNotificationManager::History() {
            let _ = history.RemoveGroupedTagWithId(
                &HSTRING::from(tag),
                &HSTRING::from(channel_id.get().to_string()),
                &HSTRING::from(AUMID),
            );
        }
    }

    pub fn remove_group(channel_id: Id<ChannelMarker>) {
        if let Ok(history) = ToastNotificationManager::History() {
            let _ = history.RemoveGroupWithId(
                &HSTRING::from(channel_id.get().to_string()),
                &HSTRING::from(AUMID),
            );
        }
    }
}
