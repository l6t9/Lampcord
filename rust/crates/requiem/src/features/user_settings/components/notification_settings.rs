use requiem_core::sounds;
use std::rc::Rc;

use requiem_ui::colors::TEXT_STRONG;
use requiem_ui::components::{
    AccordionProps, FormItemProps, RadioGroupProps, RadioOption, SelectOption, SelectProps,
    SwitchProps, TextProps, TextVariant, div,
};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::anim::ToggleState;

use requiem_core::sounds::Sound;
use requiem_core::ui::SettingsUiState;
use requiem_core::utils::notification_settings::{
    self, NotificationSettings, Reactions, TtsMessages,
};
use requiem_core::utils::voice_settings;

use super::settings_page::{self, Anchors, Switches, category, plain, rich, setting};
use crate::shell::classes::settings_page as settings_page_cls;
use requiem_ui::icons::MICROPHONE;

pub const SUBNAV: [&str; 4] = [
    requiem_i18n::key!("notifications-settings-overview"),
    requiem_i18n::key!("notifications-settings-sounds"),
    requiem_i18n::key!("notifications-settings-badges"),
    requiem_i18n::key!("notifications-settings-advanced"),
];

const PREVIEW_LINK: &str = "https://app.invalid/preview";
const ACCESSIBILITY_LINK: &str = "https://app.invalid/accessibility";

struct Page {
    revision: Signal<u64>,
    open_select: Signal<Option<&'static str>>,
    advanced_open: Signal<bool>,
    switches: Switches,
}

fn page() -> &'static Page {
    thread_local! {
        static PAGE: &'static Page = Box::leak(Box::new(Page { revision: signal(0), open_select: signal(None), advanced_open: signal(false), switches: Switches::default() }));
    }
    PAGE.with(|page| *page)
}

fn bump() {
    page().revision.update(|r| *r += 1);
}

fn change(f: impl FnOnce(&mut NotificationSettings)) {
    notification_settings::update(f);
    bump();
}

fn switch_state(key: &'static str, on: bool) -> ToggleState {
    page().switches.state(key, on)
}

fn toggle(
    key: &'static str,
    label: String,
    description: Option<Node>,
    on: bool,
    disabled: bool,
    set: fn(&mut NotificationSettings, bool),
) -> Node {
    let mut props = SwitchProps::new(switch_state(key, on), move |next: bool| {
        change(|s| set(s, next))
    })
    .label(label)
    .disabled(disabled);
    props.description_node = description;
    setting(props.build())
}

fn select(
    id: &'static str,
    options: Vec<SelectOption>,
    selected: String,
    on_pick: impl Fn(String) + 'static,
) -> Node {
    let open = page().open_select.get() == Some(id);
    SelectProps::new(
        move || {
            page()
                .open_select
                .update(|o| *o = if *o == Some(id) { None } else { Some(id) })
        },
        move |value: String| {
            page().open_select.set(None);
            on_pick(value);
        },
    )
    .options(options)
    .selected(selected)
    .open(open)
    .build()
}

fn overview_category(settings: &NotificationSettings) -> Node {
    let switch =
        |key: &'static str, label: String, on: bool, set: fn(&mut NotificationSettings, bool)| {
            SwitchProps::new(switch_state(key, on), move |next: bool| {
                change(|s| set(s, next))
            })
            .label(label)
            .build()
        };
    let reaction = match settings.reactions {
        Reactions::All => "enabled",
        Reactions::OnlyDms => "only_dms",
        Reactions::Never => "disabled",
    };
    let reactions = select(
        "reactions",
        vec![
            SelectOption::new(
                "enabled",
                requiem_i18n::t!("notifications-settings-reactions-all"),
            ),
            SelectOption::new(
                "only_dms",
                requiem_i18n::t!("notifications-settings-reactions-dms"),
            ),
            SelectOption::new("disabled", requiem_i18n::t!("notifications-settings-never")),
        ],
        reaction.into(),
        |value| {
            change(|s| {
                s.reactions = match value.as_str() {
                    "only_dms" => Reactions::OnlyDms,
                    "disabled" => Reactions::Never,
                    _ => Reactions::All,
                }
            })
        },
    );
    let notify_me = settings_page::fieldset(
        vec![
            TextProps::styled(
                requiem_i18n::t!("notifications-settings-notify-me"),
                TextVariant::TEXT_MD_MEDIUM,
            )
            .color(TEXT_STRONG)
            .build(),
        ],
        vec![
            switch(
                "go-live",
                requiem_i18n::t!("notifications-settings-go-live"),
                settings.go_live,
                |s, on| s.go_live = on,
            ),
            switch(
                "anniversary",
                requiem_i18n::t!("notifications-settings-anniversary"),
                settings.friend_anniversary,
                |s, on| s.friend_anniversary = on,
            ),
            switch(
                "friends-online",
                requiem_i18n::t!("notifications-settings-friends-online"),
                settings.friends_online,
                |s, on| s.friends_online = on,
            ),
            switch(
                "profile-updates",
                requiem_i18n::t!("notifications-settings-profile-updates"),
                settings.profile_updates,
                |s, on| s.profile_updates = on,
            ),
            FormItemProps::new(
                requiem_i18n::t!("notifications-settings-reactions"),
                reactions,
            )
            .build(),
        ],
    );
    let rows = vec![
        toggle(
            "desktop",
            requiem_i18n::t!("notifications-settings-desktop"),
            plain(requiem_i18n::t!(
                "notifications-settings-desktop-description"
            )),
            settings.desktop_notifications,
            false,
            |s, on| s.desktop_notifications = on,
        ),
        toggle(
            "taskbar",
            requiem_i18n::t!("notifications-settings-taskbar"),
            plain(requiem_i18n::t!(
                "notifications-settings-taskbar-description"
            )),
            settings.taskbar_flash,
            false,
            |s, on| s.taskbar_flash = on,
        ),
        setting(notify_me),
    ];
    category(
        Some(requiem_i18n::t!("notifications-settings-overview")),
        rows,
    )
}

fn sound_row(sound: Sound, title: String, disable_all: bool) -> Node {
    let enabled = voice_settings::get().is_sound_enabled(sound.id()) && !disable_all;
    let preview: Rc<dyn Fn(&str)> = Rc::new(move |_url: &str| sounds::preview(sound));
    let description = rich(
        requiem_i18n::t!("voice-settings-preview-sound", onClick = PREVIEW_LINK),
        Some(preview),
    );
    let mut props = SwitchProps::new(switch_state(sound.id(), enabled), move |on: bool| {
        voice_settings::update(|s| {
            s.disabled_sounds.retain(|id| id != sound.id());
            if !on {
                s.disabled_sounds.push(sound.id().to_string());
            }
        });
        bump();
    })
    .label(title)
    .disabled(disable_all);
    props.description_node = Some(description);
    setting(props.build())
}

fn sounds_category(settings: &NotificationSettings, settings_ui: &Signal<SettingsUiState>) -> Node {
    let disable_all = settings.disable_all_sounds;
    let divider = settings_page::divider;

    let preview: Rc<dyn Fn(&str)> =
        Rc::new(|_url: &str| sounds::preview(Sound::Message3));
    let selected_description = Some(rich(
        requiem_i18n::t!("voice-settings-preview-sound", onClick = PREVIEW_LINK),
        Some(preview),
    ));
    let mut rows = vec![
        sound_row(
            Sound::Message,
            requiem_i18n::t!("notifications-settings-new-message"),
            disable_all,
        ),
        divider(),
        toggle(
            "selected-channel",
            requiem_i18n::t!("notifications-settings-selected-channel"),
            selected_description,
            settings.notify_selected_channel && !disable_all,
            disable_all,
            |s, on| s.notify_selected_channel = on,
        ),
        divider(),
        sound_row(
            Sound::CallRinging,
            requiem_i18n::t!("notifications-settings-incoming-ring"),
            disable_all,
        ),
        divider(),
        toggle(
            "disable-all",
            requiem_i18n::t!("notifications-settings-disable-all"),
            plain(requiem_i18n::t!(
                "notifications-settings-disable-all-description"
            )),
            disable_all,
            false,
            |s, on| s.disable_all_sounds = on,
        ),
    ];
    let settings_ui = settings_ui.clone();
    rows.push(settings_page::related(
        requiem_i18n::t!("settings-modal-voice-video"),
        requiem_i18n::t!("notifications-settings-voice-sounds-description"),
        MICROPHONE,
        move || {
            settings_ui.update(|ui| {
                ui.category = requiem_i18n::key!("settings-modal-voice-video").to_string()
            })
        },
    ));
    category(
        Some(requiem_i18n::t!("notifications-settings-sounds")),
        rows,
    )
}

fn badges_category(settings: &NotificationSettings) -> Node {
    let rows = vec![toggle(
        "badge",
        requiem_i18n::t!("notifications-settings-badge"),
        plain(requiem_i18n::t!("notifications-settings-badge-description")),
        settings.unread_badge,
        false,
        |s, on| s.unread_badge = on,
    )];
    category(
        Some(requiem_i18n::t!("notifications-settings-badges")),
        rows,
    )
}

fn advanced_category(
    settings: &NotificationSettings,
    settings_ui: &Signal<SettingsUiState>,
) -> Node {
    let delay_options = (1..=10u8)
        .map(|minutes| {
            SelectOption::new(
                minutes.to_string(),
                requiem_i18n::t!(
                    "notifications-settings-minutes",
                    minutes = i64::from(minutes)
                ),
            )
        })
        .collect();
    let delay = select(
        "mobile-delay",
        delay_options,
        settings.mobile_delay_minutes.to_string(),
        |value| {
            if let Ok(minutes) = value.parse() {
                change(|s| s.mobile_delay_minutes = minutes);
            }
        },
    );
    let mut delay_field = FormItemProps::new(
        requiem_i18n::t!("notifications-settings-mobile-delay"),
        delay,
    );
    delay_field.description_node = plain(requiem_i18n::t!(
        "notifications-settings-mobile-delay-description"
    ));

    let settings_ui = settings_ui.clone();
    let to_accessibility: Rc<dyn Fn(&str)> = Rc::new(move |_url: &str| {
        settings_ui.update(|ui| {
            ui.category = requiem_i18n::key!("settings-modal-accessibility").to_string()
        })
    });
    let tts_description = Some(rich(
        requiem_i18n::t!(
            "notifications-settings-tts-command-description",
            onClick = ACCESSIBILITY_LINK
        ),
        Some(to_accessibility),
    ));

    let active = settings.tts_messages;
    let tts_options = [
        (
            TtsMessages::AllChannels,
            "ALL_CHANNELS",
            requiem_i18n::t!("notifications-settings-tts-all"),
        ),
        (
            TtsMessages::SelectedChannel,
            "SELECTED_CHANNEL",
            requiem_i18n::t!("notifications-settings-tts-selected"),
        ),
        (
            TtsMessages::Never,
            "NEVER",
            requiem_i18n::t!("notifications-settings-never"),
        ),
    ];
    let options: Vec<RadioOption> = tts_options
        .into_iter()
        .map(|(value, id, label)| RadioOption::new(id, label, switch_state(id, value == active)))
        .collect();
    let radios = RadioGroupProps::new(|value: String| {
        change(|s| {
            s.tts_messages = match value.as_str() {
                "ALL_CHANNELS" => TtsMessages::AllChannels,
                "SELECTED_CHANNEL" => TtsMessages::SelectedChannel,
                _ => TtsMessages::Never,
            }
        })
    })
    .options(options)
    .build();
    let mut tts_field = FormItemProps::new(
        requiem_i18n::t!("notifications-settings-tts-messages"),
        radios,
    );
    tts_field.description_node = plain(requiem_i18n::t!(
        "notifications-settings-tts-messages-description"
    ));

    let content = div().class(settings_page_cls::ROWS).children([
        setting(delay_field.build()),
        toggle(
            "tts-command",
            requiem_i18n::t!("notifications-settings-tts-command"),
            tts_description,
            settings.tts_command,
            false,
            |s, on| s.tts_command = on,
        ),
        setting(tts_field.build()),
    ]);
    let accordion = AccordionProps {
        title_closed: requiem_i18n::t!("notifications-settings-advanced-show"),
        title_open: requiem_i18n::t!("notifications-settings-advanced-hide"),
        subtitle_closed: None,
        open: page().advanced_open.get(),
        on_toggle: Rc::new(|| page().advanced_open.update(|o| *o = !*o)),
        content,
    }
    .build();
    category(
        Some(requiem_i18n::t!("notifications-settings-advanced")),
        vec![accordion],
    )
}

pub fn build_page(settings_ui: &Signal<SettingsUiState>) -> Node {
    let settings_ui = settings_ui.clone();
    let content = Node::reactive(move || {
        let _ = page().revision.get();
        let settings = notification_settings::get();
        let categories = vec![
            anchored(0, overview_category(&settings)),
            anchored(1, sounds_category(&settings, &settings_ui)),
            anchored(2, badges_category(&settings)),
            anchored(3, advanced_category(&settings, &settings_ui)),
        ];
        settings_page::panel(categories)
    });
    settings_page::page_container(content)
}

fn anchors() -> &'static Anchors {
    thread_local! {
        static ANCHORS: &'static Anchors = Anchors::leak(SUBNAV.len());
    }
    ANCHORS.with(|anchors| *anchors)
}

pub fn reset_scroll() {
    anchors().reset();
}

pub fn active_subnav() -> usize {
    anchors().active()
}

pub fn jump_to(index: usize, rebuild: impl FnOnce()) {
    anchors().jump_to(index, rebuild);
}

pub fn scroll_view(class: &'static str, page: Node) -> Node {
    anchors().scroll_view(class, page)
}

fn anchored(index: usize, node: Node) -> Node {
    anchors().anchored(index, node)
}
