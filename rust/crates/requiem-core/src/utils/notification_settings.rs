use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Reactions {
    #[default]
    All,
    OnlyDms,
    Never,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TtsMessages {
    AllChannels,
    SelectedChannel,
    #[default]
    Never,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationSettings {
    pub desktop_notifications: bool,

    pub taskbar_flash: bool,

    pub notify_selected_channel: bool,

    pub unread_badge: bool,

    pub disable_all_sounds: bool,
    pub go_live: bool,
    pub friend_anniversary: bool,
    pub friends_online: bool,
    pub profile_updates: bool,
    pub reactions: Reactions,

    pub mobile_delay_minutes: u8,

    pub tts_command: bool,
    pub tts_messages: TtsMessages,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            desktop_notifications: true,
            taskbar_flash: true,
            notify_selected_channel: false,
            unread_badge: true,
            disable_all_sounds: false,
            go_live: true,
            friend_anniversary: true,
            friends_online: true,
            profile_updates: true,
            reactions: Reactions::All,
            mobile_delay_minutes: 1,
            tts_command: true,
            tts_messages: TtsMessages::Never,
        }
    }
}

static SETTINGS: RwLock<Option<NotificationSettings>> = RwLock::new(None);
static VERSION: AtomicU64 = AtomicU64::new(1);

pub fn get() -> NotificationSettings {
    if let Some(settings) = SETTINGS
        .read()
        .expect("notification settings poisoned")
        .as_ref()
    {
        return settings.clone();
    }
    let mut slot = SETTINGS.write().expect("notification settings poisoned");
    slot.get_or_insert_with(|| {
        super::prefs::load()
            .notification_settings
            .unwrap_or_default()
    })
    .clone()
}

pub fn version() -> u64 {
    VERSION.load(Ordering::Acquire)
}

pub fn update(change: impl FnOnce(&mut NotificationSettings)) {
    let saved = {
        let mut slot = SETTINGS.write().expect("notification settings poisoned");
        let settings = slot.get_or_insert_with(|| {
            super::prefs::load()
                .notification_settings
                .unwrap_or_default()
        });
        change(settings);
        settings.clone()
    };
    VERSION.fetch_add(1, Ordering::AcqRel);
    super::prefs::save(&super::prefs::Prefs {
        notification_settings: Some(saved),
        ..super::prefs::load()
    });
}
