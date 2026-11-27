use requiem_core::models;
use requiem_core::sounds;
use requiem_core::utils::link;
use requiem_ui::colors;
use requiem_ui::components;
use requiem_ui::signal as requiem_ui_signal;
use requiem_ui::timer;
use std::cell::Cell;
use std::fs;
use std::rc::Rc;
use std::time::{Duration, Instant};

use requiem_ui::colors::{TEXT_SECONDARY, TEXT_STRONG};
use requiem_ui::components::{
    AccordionProps, ButtonProps, ButtonSize, ButtonVariant, ConfirmModalProps, ControlItemProps,
    FormItemProps, MessageTextProps, RadioGroupProps, RadioOption, SelectOption, SelectProps,
    SliderProps, SwitchProps, TextProps, TextVariant, TooltipProps, div,
};
use requiem_ui::control::section;
use requiem_ui::node::Node;
use requiem_ui::signal::{Effect, Signal, signal};
use requiem_ui::widget::anim::ToggleState;
use requiem_ui::widget::{WidgetId, WidgetTree};

use requiem_core::sounds::Sound;
use requiem_core::state::AppState;
use requiem_core::ui::SettingsUiState;
use requiem_core::ui::dialog::Dialog;
use requiem_core::utils::voice_settings::{
    self, AudioSubsystem, InputMode, InputProfile, MAX_PTT_RELEASE_DELAY, VoiceSettings,
    slider_to_volume, volume_to_slider,
};
use requiem_core::voice;

use crate::features::user_settings::settings_page::{self, Anchors, Switches, category, plain, rich, setting};
use super::voice_meter;
use crate::shell::classes::settings_page as settings_page_cls;
use crate::shell::classes::voice_settings as cls;
use requiem_ui::icons::BELL;

const TROUBLESHOOTING_URL: &str = "https://support.discord.com/hc/en-us/articles/360045138471?utm_source=discord&utm_medium=blog&utm_campaign=2020-06_help-voice-video&utm_content=--t%3Apm";
const NO_INPUT_URL: &str = "https://support.discord.com/hc/en-us/articles/214925018";
const SOUNDBOARD_URL: &str = "https://support.discord.com/hc/en-us/articles/12612888127767";

const KEYBINDS_LINK: &str = "https://app.invalid/keybinds";
const PREVIEW_LINK: &str = "https://app.invalid/preview";
const MIC_PERMISSION_LINK: &str = "https://app.invalid/microphone-permission";

pub const SUBNAV: [&str; 6] = [
    requiem_i18n::key!("voice-settings-voice"),
    requiem_i18n::key!("voice-settings-camera"),
    requiem_i18n::key!("voice-settings-streaming"),
    requiem_i18n::key!("voice-settings-sounds"),
    requiem_i18n::key!("voice-settings-soundboard"),
    requiem_i18n::key!("voice-settings-advanced"),
];

const SOUNDS: [(Sound, &str); 22] = [
    (
        Sound::Deafen,
        requiem_i18n::key!("voice-settings-sound-deafen"),
    ),
    (
        Sound::Undeafen,
        requiem_i18n::key!("voice-settings-sound-undeafen"),
    ),
    (Sound::Mute, requiem_i18n::key!("voice-settings-sound-mute")),
    (
        Sound::Unmute,
        requiem_i18n::key!("voice-settings-sound-unmute"),
    ),
    (
        Sound::CameraOn,
        requiem_i18n::key!("voice-settings-sound-camera-on"),
    ),
    (
        Sound::CameraOff,
        requiem_i18n::key!("voice-settings-sound-camera-off"),
    ),
    (
        Sound::Disconnect,
        requiem_i18n::key!("voice-settings-sound-disconnect"),
    ),
    (
        Sound::PttStart,
        requiem_i18n::key!("voice-settings-sound-ptt-start"),
    ),
    (
        Sound::PttStop,
        requiem_i18n::key!("voice-settings-sound-ptt-stop"),
    ),
    (
        Sound::UserJoin,
        requiem_i18n::key!("voice-settings-sound-user-join"),
    ),
    (
        Sound::UserLeave,
        requiem_i18n::key!("voice-settings-sound-user-leave"),
    ),
    (
        Sound::UserMoved,
        requiem_i18n::key!("voice-settings-sound-user-moved"),
    ),
    (
        Sound::CallCalling,
        requiem_i18n::key!("voice-settings-sound-call-calling"),
    ),
    (
        Sound::StreamStarted,
        requiem_i18n::key!("voice-settings-sound-stream-started"),
    ),
    (
        Sound::StreamEnded,
        requiem_i18n::key!("voice-settings-sound-stream-ended"),
    ),
    (
        Sound::StreamUserJoined,
        requiem_i18n::key!("voice-settings-sound-stream-user-joined"),
    ),
    (
        Sound::StreamUserLeft,
        requiem_i18n::key!("voice-settings-sound-stream-user-left"),
    ),
    (
        Sound::ActivityLaunch,
        requiem_i18n::key!("voice-settings-sound-activity-launch"),
    ),
    (
        Sound::ActivityEnd,
        requiem_i18n::key!("voice-settings-sound-activity-end"),
    ),
    (
        Sound::ActivityUserJoin,
        requiem_i18n::key!("voice-settings-sound-activity-user-join"),
    ),
    (
        Sound::ActivityUserLeft,
        requiem_i18n::key!("voice-settings-sound-activity-user-left"),
    ),
    (
        Sound::Reconnect,
        requiem_i18n::key!("voice-settings-sound-reconnect"),
    ),
];
const SOUNDS_ALWAYS_SHOWN: usize = 4;
const DEFAULT: &str = "default";

#[derive(Clone)]
enum Confirm {
    Reset,
    Subsystem(AudioSubsystem),

    SwitchChannel { channel: String, join: Rc<dyn Fn()> },
}

struct Page {
    revision: Signal<u64>,
    open_select: Signal<Option<&'static str>>,
    advanced_open: Signal<bool>,
    stream_advanced_open: Signal<bool>,
    diagnostics_open: Signal<bool>,
    sounds_open: Signal<bool>,

    testing: Signal<bool>,

    detecting: Signal<bool>,
    last_voice: Cell<Instant>,

    deafened_for_test: Cell<bool>,

    preview: Signal<bool>,

    recording: Signal<bool>,
    switches: Switches,
    dialog: Dialog,
    confirm: Signal<Option<Confirm>>,
}

fn settings_page() -> &'static Page {
    thread_local! {
        static PAGE: &'static Page = Box::leak(Box::new(Page {
            revision: signal(0),
            open_select: signal(None),
            advanced_open: signal(false),
            stream_advanced_open: signal(false),
            diagnostics_open: signal(false),
            sounds_open: signal(false),
            testing: signal(false),
            detecting: signal(true),
            last_voice: Cell::new(Instant::now()),
            deafened_for_test: Cell::new(false),
            preview: signal(false),
            recording: signal(false),
            switches: Switches::default(),
            dialog: Dialog::new(),
            confirm: signal(None),
        }));
    }
    PAGE.with(|page| *page)
}

fn change(f: impl FnOnce(&mut VoiceSettings)) {
    voice_settings::update(f);
    settings_page().revision.update(|r| *r += 1);
}

fn switch_state(key: &'static str, on: bool) -> ToggleState {
    settings_page().switches.state(key, on)
}

fn ensure_ticker(settings_ui: &Signal<SettingsUiState>) {
    fn schedule(settings_ui: Signal<SettingsUiState>) {
        timer::set_timeout(Duration::from_millis(50), move || {
            if settings_ui.peek().category != requiem_i18n::key!("settings-modal-voice-video") {
                TICKING.with(|t| t.set(false));
                on_leave();
                return;
            }
            tick();
            schedule(settings_ui);
        });
    }

    thread_local! {
        static TICKING: Cell<bool> = const { Cell::new(false) };
    }
    if TICKING.with(|t| t.replace(true)) {
        return;
    }
    schedule(settings_ui.clone());
}

fn tick() {
    let page = settings_page();
    if page.testing.peek() {
        voice::engine::keepalive();
        if !voice::engine::loopback() {
            stop_test();
        } else {
            let (level, _) = voice::engine::meter::read();
            if level > voice::engine::SILENCE_DB {
                page.last_voice.set(Instant::now());
            }
            let detecting = page.last_voice.get().elapsed() < Duration::from_secs(2);
            if page.detecting.peek() != detecting {
                page.detecting.set(detecting);
            }
        }
    }
    if page.recording.peek() {
        if let Some(keys) = voice::ptt::take_recorded() {
            page.recording.set(false);
            change(|s| s.mode_options.shortcut = keys);
        } else if !voice::ptt::is_recording() {
            page.recording.set(false);
        }
    }
}

pub fn on_leave() {
    let page = settings_page();
    if page.preview.peek() {
        page.preview.set(false);
    }
    if page.testing.peek() {
        stop_test();
    }
    if page.recording.peek() {
        voice::ptt::stop_recording();
        page.recording.set(false);
    }
}

fn toggle_test() {
    if settings_page().testing.peek() {
        stop_test();
    } else {
        start_test();
    }
}

fn start_test() {
    let page = settings_page();
    if voice::current_channel().is_some() && !voice::is_self_deaf() {
        voice::set_deaf(true);
        page.deafened_for_test.set(true);
    }
    voice::engine::set_loopback(true);
    page.last_voice.set(Instant::now());
    page.detecting.set(true);
    page.testing.set(true);
}

fn stop_test() {
    let page = settings_page();
    voice::engine::set_loopback(false);
    if page.deafened_for_test.replace(false) && voice::current_channel().is_some() {
        voice::set_deaf(false);
    }
    page.testing.set(false);
}

fn toggle(
    key: &'static str,
    label: String,
    description: Option<Node>,
    on: bool,
    disabled: bool,
    set: fn(&mut VoiceSettings, bool),
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
    disabled: bool,
    on_pick: impl Fn(String) + 'static,
) -> Node {
    let open = settings_page().open_select.get() == Some(id);
    SelectProps::new(
        move || {
            settings_page()
                .open_select
                .update(|o| *o = if *o == Some(id) { None } else { Some(id) })
        },
        move |value: String| {
            settings_page().open_select.set(None);
            on_pick(value);
        },
    )
    .options(options)
    .selected(selected)
    .open(open && !disabled)
    .disabled(disabled)
    .build()
}

fn list_subtitle(titles: &[String], limit: usize) -> String {
    let shown: Vec<&String> = titles.iter().take(limit).collect();
    let joined = |items: &[&String]| {
        items
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    if titles.len() > limit {
        return requiem_i18n::t!("voice-settings-list-more", items = joined(&shown));
    }
    match shown.as_slice() {
        [] => String::new(),
        [one] => (*one).clone(),
        [first, second] => requiem_i18n::t!(
            "voice-settings-list-two",
            first = first.as_str(),
            second = second.as_str()
        ),
        [rest @ .., last] => requiem_i18n::t!(
            "voice-settings-list-tail",
            rest = joined(rest),
            last = last.as_str()
        ),
    }
}

fn device_options(names: Vec<String>) -> Vec<SelectOption> {
    let mut options = vec![SelectOption::new(
        DEFAULT,
        requiem_i18n::t!("voice-settings-default"),
    )];
    options.extend(
        names
            .into_iter()
            .map(|name| SelectOption::new(name.clone(), name)),
    );
    options
}

fn voice_category(settings: &VoiceSettings) -> Node {
    let page = settings_page();
    let input = FormItemProps::new(
        requiem_i18n::t!("voice-settings-microphone"),
        select(
            "input",
            device_options(voice::audio::device_names(true)),
            settings
                .input_device
                .clone()
                .unwrap_or_else(|| DEFAULT.into()),
            false,
            |value| change(|s| s.input_device = (value != DEFAULT).then_some(value)),
        ),
    )
    .build();
    let output = FormItemProps::new(
        requiem_i18n::t!("voice-settings-speaker"),
        select(
            "output",
            device_options(voice::audio::device_names(false)),
            settings
                .output_device
                .clone()
                .unwrap_or_else(|| DEFAULT.into()),
            false,
            |value| change(|s| s.output_device = (value != DEFAULT).then_some(value)),
        ),
    )
    .build();
    let devices = div().class(cls::SPLIT).children([
        setting(input).class(cls::SPLIT_COLUMN),
        setting(output).class(cls::SPLIT_COLUMN),
    ]);

    let mic_volume = SliderProps::default()
        .value(volume_to_slider(settings.input_volume))
        .max(100.0_f32)
        .on_change(|value: f32| {
            voice_settings::update(|s| s.input_volume = slider_to_volume(value, 100.0))
        });
    let speaker_volume = SliderProps::default()
        .value(volume_to_slider(settings.output_volume))
        .max(200.0_f32)
        .on_change(|value: f32| {
            voice_settings::update(|s| s.output_volume = slider_to_volume(value, 200.0))
        });
    let volumes = div().class(cls::SPLIT).children([
        setting(
            FormItemProps::new(
                requiem_i18n::t!("voice-settings-microphone-volume"),
                mic_volume.build(),
            )
            .build(),
        )
        .class(cls::SPLIT_COLUMN),
        setting(
            FormItemProps::new(
                requiem_i18n::t!("voice-settings-speaker-volume"),
                speaker_volume.build(),
            )
            .build(),
        )
        .class(cls::SPLIT_COLUMN),
    ]);

    let testing = page.testing.get();
    let label = if testing {
        requiem_i18n::t!("voice-settings-mic-test-stop")
    } else {
        requiem_i18n::t!("voice-settings-mic-test")
    };
    let mut button = ButtonProps::new(ButtonVariant::Primary, label, toggle_test).build();
    if voice::current_channel().is_some() && !testing {
        button = TooltipProps {
            text: requiem_i18n::t!("voice-settings-mic-test-in-call"),
            ..Default::default()
        }
        .build(button);
    }
    let caption = if !testing {
        div()
    } else if page.detecting.get() {
        TextProps::styled(
            requiem_i18n::t!("voice-settings-mic-test-voice"),
            TextVariant::TEXT_XS_NORMAL,
        )
        .color(TEXT_SECONDARY)
        .build()
    } else {
        let help = format!(
            "[{}]({NO_INPUT_URL})",
            requiem_i18n::t!("voice-settings-mic-test-no-voice-link")
        );
        div().class(cls::MIC_TEST_NO_VOICE).children([
            TextProps::styled(
                requiem_i18n::t!("voice-settings-mic-test-no-voice-title"),
                TextVariant::TEXT_XS_SEMIBOLD,
            )
            .color(TEXT_STRONG)
            .build(),
            components::message_text(MessageTextProps {
                content: format!(
                    "{} {help}",
                    requiem_i18n::t!("voice-settings-mic-test-no-voice")
                ),
                size: 12.0,
                line_height: Some(16.0),
                color: TEXT_SECONDARY,
                ..Default::default()
            }),
        ])
    };
    let meter_column = div().class(cls::MIC_TEST_METER_COLUMN).children([
        voice_meter::mic_test_meter(testing),
        div().class(cls::MIC_TEST_CAPTION).child(caption),
    ]);
    let mut help = rich(
        requiem_i18n::t!(
            "voice-settings-mic-test-help",
            guideURL = TROUBLESHOOTING_URL
        ),
        None,
    );
    if testing && !page.detecting.get() {
        help = help.class(cls::INVISIBLE);
    }
    let mic_test = div().class(cls::MIC_TEST).children([
        div()
            .class(cls::MIC_TEST_ROW)
            .children([button, meter_column]),
        help,
    ]);

    category(
        Some(requiem_i18n::t!("voice-settings-voice")),
        vec![devices, volumes, setting(mic_test)],
    )
}

fn input_profile_category(settings: &VoiceSettings, settings_ui: &Signal<SettingsUiState>) -> Node {
    let page = settings_page();
    let effective = settings.effective();
    let custom = settings.is_input_profile_custom();
    let active = settings
        .active_input_profile
        .unwrap_or(InputProfile::Custom);
    let mut rows = Vec::new();

    let options: Vec<RadioOption> = [
        (
            InputProfile::VoiceIsolation,
            "voice-settings-profile-isolation",
            "voice-settings-profile-isolation-description",
        ),
        (
            InputProfile::Studio,
            "voice-settings-profile-studio",
            "voice-settings-profile-studio-description",
        ),
        (
            InputProfile::Custom,
            "voice-settings-profile-custom",
            "voice-settings-profile-custom-description",
        ),
    ]
    .into_iter()
    .map(|(profile, label, description)| {
        RadioOption::new(
            profile.as_str(),
            requiem_i18n::text(label),
            switch_state(profile.as_str(), profile == active),
        )
        .with_description(requiem_i18n::text(description))
    })
    .collect();
    let radios = RadioGroupProps::new(|value: String| {
        if let Some(profile) = InputProfile::parse(&value) {
            change(|s| s.active_input_profile = Some(profile));
        }
    })
    .options(options)
    .build();
    rows.push(setting(
        FormItemProps::new(requiem_i18n::t!("voice-settings-input-profile"), radios).build(),
    ));

    if custom {
        let disabled = effective.mode != InputMode::VoiceActivity;
        let auto = effective.mode_options.auto_threshold;
        let description = if auto {
            requiem_i18n::t!("voice-settings-input-sensitivity-auto-description")
        } else {
            requiem_i18n::t!("voice-settings-input-sensitivity-manual-description")
        };
        let mut auto_switch = SwitchProps::new(switch_state("auto-threshold", auto), |on: bool| {
            change(|s| s.mode_options.auto_threshold = on)
        })
        .label(requiem_i18n::t!("voice-settings-input-sensitivity-auto"))
        .disabled(disabled);
        auto_switch.description_node = plain(description);
        let meter = if auto {
            voice_meter::speaking_bar(disabled)
        } else {
            SliderProps::default()
                .value(effective.mode_options.threshold + 100.0)
                .max(100.0_f32)
                .bar_class(cls::SENSITIVITY_SLIDER)
                .overlay(voice_meter::level_fraction)
                .disabled(disabled)
                .on_change(|value: f32| {
                    voice_settings::update(|s| s.mode_options.threshold = value - 100.0)
                })
                .build()
        };
        let mut sensitivity = vec![auto_switch.build(), meter];
        if voice::audio::device_names(true).is_empty() {
            let on_link: Rc<dyn Fn(&str)> = Rc::new(|_url: &str| {
                link::open_url("ms-settings:privacy-microphone")
            });
            let message = rich(
                requiem_i18n::t!(
                    "voice-settings-no-input-devices",
                    onEnableClick = MIC_PERMISSION_LINK
                ),
                Some(on_link),
            );
            sensitivity.push(
                components::HelpMessageProps {
                    content: Some(message),
                    ..Default::default()
                }
                .build(),
            );
        }
        rows.push(setting(div().class(cls::STACK_8).children(sensitivity)));

        let value = if effective.noise_suppression {
            "STANDARD"
        } else {
            "NONE"
        };
        let options = vec![
            SelectOption::new(
                "STANDARD",
                requiem_i18n::t!("voice-settings-noise-suppression-standard"),
            ),
            SelectOption::new(
                "NONE",
                requiem_i18n::t!("voice-settings-noise-suppression-none"),
            ),
        ];
        let control = div().class(cls::NOISE_SELECT).child(select(
            "noise",
            options,
            value.into(),
            false,
            |value| {
                change(|s| {
                    s.noise_cancellation = false;
                    s.noise_suppression = value == "STANDARD";
                })
            },
        ));
        let mut field = FormItemProps::new(
            requiem_i18n::t!("voice-settings-noise-suppression"),
            control,
        )
        .horizontal(true);
        field.description_node = plain(requiem_i18n::t!(
            "voice-settings-noise-suppression-description"
        ));
        rows.push(setting(field.build()));

        rows.push(toggle(
            "echo",
            requiem_i18n::t!("voice-settings-echo-cancellation"),
            None,
            effective.echo_cancellation,
            false,
            |s, on| s.echo_cancellation = on,
        ));
    }

    if active != InputProfile::Studio {
        rows.push(toggle(
            "ptt",
            requiem_i18n::t!("voice-settings-push-to-talk"),
            None,
            effective.mode == InputMode::PushToTalk,
            false,
            |s, on| {
                s.mode = if on {
                    InputMode::PushToTalk
                } else {
                    InputMode::VoiceActivity
                }
            },
        ));
    }
    if effective.mode == InputMode::PushToTalk {
        rows.push(setting(keybind_row(&effective, settings_ui)));
        let delay = SliderProps::default()
            .value(effective.mode_options.delay as f32)
            .max(MAX_PTT_RELEASE_DELAY as f32)
            .on_change(|value: f32| {
                voice_settings::update(|s| s.mode_options.delay = value.round() as u32)
            });
        rows.push(setting(
            FormItemProps::new(
                requiem_i18n::t!("voice-settings-push-to-talk-release-delay"),
                delay.build(),
            )
            .build(),
        ));
    }

    let mut advanced: Vec<(String, Node)> = Vec::new();
    if custom {
        let title = requiem_i18n::t!("voice-settings-automatic-gain-control");
        advanced.push((
            title.clone(),
            toggle(
                "agc",
                title,
                plain(requiem_i18n::t!(
                    "voice-settings-automatic-gain-control-description"
                )),
                effective.automatic_gain_control,
                false,
                |s, on| s.automatic_gain_control = on,
            ),
        ));
        let title = requiem_i18n::t!("voice-settings-advanced-voice-activity");
        let ava_disabled =
            effective.mode != InputMode::VoiceActivity || !effective.mode_options.auto_threshold;
        advanced.push((
            title.clone(),
            toggle(
                "ava",
                title,
                plain(requiem_i18n::t!(
                    "voice-settings-advanced-voice-activity-description"
                )),
                effective.mode_options.vad_use_krisp,
                ava_disabled,
                |s, on| s.mode_options.vad_use_krisp = on,
            ),
        ));
    }
    let title = requiem_i18n::t!("voice-settings-device-suggestions");
    advanced.push((
        title.clone(),
        toggle(
            "suggestions",
            title,
            plain(requiem_i18n::t!(
                "voice-settings-device-suggestions-description"
            )),
            settings.device_suggestions,
            false,
            |s, on| s.device_suggestions = on,
        ),
    ));
    let title = requiem_i18n::t!("voice-settings-silence-warning");
    advanced.push((
        title.clone(),
        toggle(
            "silence",
            title,
            plain(requiem_i18n::t!(
                "voice-settings-silence-warning-description"
            )),
            settings.silence_warning,
            false,
            |s, on| s.silence_warning = on,
        ),
    ));
    let title = requiem_i18n::t!("voice-settings-switch-channel-warning");
    advanced.push((
        title.clone(),
        toggle(
            "switch-warning",
            title,
            plain(requiem_i18n::t!(
                "voice-settings-switch-channel-warning-description"
            )),
            !settings.disable_voice_channel_change_alert,
            false,
            |s, on| s.disable_voice_channel_change_alert = !on,
        ),
    ));
    let title = requiem_i18n::t!("voice-settings-attenuation");
    advanced.push((title, setting(attenuation_fieldset(settings))));
    let title = requiem_i18n::t!("voice-settings-audio-subsystem");
    let mut subsystems = vec![SelectOption::new(
        AudioSubsystem::Standard.as_str(),
        requiem_i18n::t!("voice-settings-audio-subsystem-standard"),
    )];
    if voice::audio::asio_available() || settings.audio_subsystem == AudioSubsystem::Asio {
        subsystems.push(SelectOption::new(
            AudioSubsystem::Asio.as_str(),
            requiem_i18n::t!("voice-settings-audio-subsystem-asio"),
        ));
    }
    let subsystem = select(
        "subsystem",
        subsystems,
        settings.audio_subsystem.as_str().into(),
        false,
        |value| {
            if let Some(subsystem) = AudioSubsystem::parse(&value) {
                if subsystem != voice_settings::get().audio_subsystem {
                    open_confirm(Confirm::Subsystem(subsystem));
                }
            }
        },
    );
    advanced.push((
        title.clone(),
        setting(FormItemProps::new(title, subsystem).build()),
    ));
    let title = requiem_i18n::t!("voice-settings-qos");
    advanced.push((
        title.clone(),
        toggle(
            "qos",
            title,
            plain(requiem_i18n::t!("voice-settings-qos-description")),
            settings.qos,
            false,
            |s, on| s.qos = on,
        ),
    ));

    let titles: Vec<String> = advanced.iter().map(|(title, _)| title.clone()).collect();
    let open = page.advanced_open.get();
    rows.push(
        AccordionProps {
            title_closed: requiem_i18n::t!("voice-settings-advanced-show"),
            title_open: requiem_i18n::t!("voice-settings-advanced-hide"),
            subtitle_closed: Some(list_subtitle(&titles, if custom { 3 } else { 2 })),
            open,
            on_toggle: Rc::new(|| settings_page().advanced_open.update(|o| *o = !*o)),
            content: div()
                .class(settings_page_cls::ROWS)
                .children(advanced.into_iter().map(|(_, node)| node)),
        }
        .build(),
    );
    category(None, rows)
}

fn keybind_row(settings: &VoiceSettings, settings_ui: &Signal<SettingsUiState>) -> Node {
    let page = settings_page();
    let recording = page.recording.get();
    let keys = if recording {
        voice::ptt::recording_keys()
    } else {
        settings.mode_options.shortcut.clone()
    };
    let shown = if keys.is_empty() && !recording {
        TextProps::styled(
            requiem_i18n::t!("voice-settings-no-keybind"),
            TextVariant::TEXT_SM_SEMIBOLD,
        )
        .color(colors::TEXT_SUBTLE)
        .build()
    } else {
        TextProps::styled(
            voice::ptt::shortcut_label(&keys),
            TextVariant::TEXT_SM_SEMIBOLD,
        )
        .color(TEXT_STRONG)
        .build()
    };
    let label = if recording {
        requiem_i18n::t!("voice-settings-stop-recording")
    } else {
        requiem_i18n::t!("voice-settings-record-keybind")
    };
    let button = ButtonProps::new(ButtonVariant::Secondary, label, || {
        let page = settings_page();
        if page.recording.peek() {
            voice::ptt::stop_recording();
            page.recording.set(false);
        } else {
            voice::ptt::start_recording();
            page.recording.set(true);
        }
    })
    .size(ButtonSize::Small)
    .build();
    let mut recorder = div().class(cls::RECORDER).children([
        div().class(cls::RECORDER_KEYS).child(shown),
        div().class(cls::RECORDER_BUTTON).child(button),
    ]);
    if recording {
        recorder = recorder.class(cls::RECORDER_ACTIVE);
    }

    let settings_ui = settings_ui.clone();
    let on_link: Rc<dyn Fn(&str)> = Rc::new(move |_url: &str| {
        settings_ui
            .update(|ui| ui.category = requiem_i18n::key!("settings-modal-system").to_string())
    });
    let mut field = FormItemProps::new(
        requiem_i18n::t!("voice-settings-push-to-talk-keybind"),
        recorder,
    )
    .horizontal(true);
    field.description_node = Some(rich(
        requiem_i18n::t!(
            "voice-settings-push-to-talk-keybind-description",
            onClick = KEYBINDS_LINK
        ),
        Some(on_link),
    ));
    field.build()
}

fn attenuation_fieldset(settings: &VoiceSettings) -> Node {
    let strength = SliderProps::default()
        .value(f32::from(settings.attenuation))
        .max(100.0_f32)
        .on_change(|value: f32| voice_settings::update(|s| s.attenuation = value.round() as u8));
    let switch = |key: &'static str, label: String, on: bool, set: fn(&mut VoiceSettings, bool)| {
        SwitchProps::new(switch_state(key, on), move |next: bool| {
            change(|s| set(s, next))
        })
        .label(label)
        .build()
    };
    let legend = vec![
        TextProps::styled(
            requiem_i18n::t!("voice-settings-attenuation"),
            TextVariant::TEXT_MD_MEDIUM,
        )
        .color(TEXT_STRONG)
        .build(),
        TextProps::styled(
            requiem_i18n::t!("voice-settings-attenuation-description"),
            TextVariant::TEXT_SM_NORMAL,
        )
        .color(TEXT_SECONDARY)
        .build(),
    ];
    settings_page::fieldset(
        legend,
        vec![
            FormItemProps::new(
                requiem_i18n::t!("voice-settings-attenuation-amount"),
                strength.build(),
            )
            .build(),
            switch(
                "attenuate-self",
                requiem_i18n::t!("voice-settings-attenuation-self"),
                settings.attenuate_while_speaking_self,
                |s, on| s.attenuate_while_speaking_self = on,
            ),
            switch(
                "attenuate-others",
                requiem_i18n::t!("voice-settings-attenuation-others"),
                settings.attenuate_while_speaking_others,
                |s, on| s.attenuate_while_speaking_others = on,
            ),
        ],
    )
}

fn camera_category(settings: &VoiceSettings) -> Node {
    let page = settings_page();
    let cameras = voice::capture::camera_names();
    let available = !cameras.is_empty();

    let content = if page.preview.get() && available {
        voice_meter::camera_preview()
    } else {
        let button = ButtonProps::new(
            ButtonVariant::Primary,
            requiem_i18n::t!("voice-settings-test-video"),
            || settings_page().preview.set(true),
        )
        .disabled(!available)
        .build();
        if available {
            button
        } else {
            TooltipProps {
                text: requiem_i18n::t!("voice-settings-no-video"),
                ..Default::default()
            }
            .build(button)
        }
    };
    let rows = vec![
        setting(div().class(cls::CAMERA_BOX).child(content)),
        toggle(
            "always-preview",
            requiem_i18n::t!("voice-settings-always-preview-video"),
            plain(requiem_i18n::t!(
                "voice-settings-always-preview-video-description"
            )),
            settings.always_preview_video,
            false,
            |s, on| s.always_preview_video = on,
        ),
        setting(
            FormItemProps::new(
                requiem_i18n::t!("voice-settings-camera-device"),
                select(
                    "camera",
                    device_options(cameras),
                    settings
                        .video_device
                        .clone()
                        .unwrap_or_else(|| DEFAULT.into()),
                    !available,
                    |value| change(|s| s.video_device = (value != DEFAULT).then_some(value)),
                ),
            )
            .build(),
        ),
    ];
    category(Some(requiem_i18n::t!("voice-settings-camera")), rows)
}

fn streaming_category(settings: &VoiceSettings) -> Node {
    let page = settings_page();
    let mut advanced = vec![toggle(
        "stream-attenuation",
        requiem_i18n::t!("voice-settings-stream-attenuation"),
        plain(requiem_i18n::t!(
            "voice-settings-stream-attenuation-description"
        )),
        settings.sidechain_compression,
        false,
        |s, on| s.sidechain_compression = on,
    )];
    let mut titles = vec![requiem_i18n::t!("voice-settings-stream-attenuation")];
    if settings.sidechain_compression {
        let strength = SliderProps::default()
            .value(f32::from(settings.sidechain_compression_strength))
            .min(1.0_f32)
            .max(100.0_f32)
            .on_change(|value: f32| {
                voice_settings::update(|s| s.sidechain_compression_strength = value.round() as u8)
            });
        advanced.push(setting(
            FormItemProps::new(
                requiem_i18n::t!("voice-settings-stream-attenuation-strength"),
                strength.build(),
            )
            .build(),
        ));
        titles.push(requiem_i18n::t!(
            "voice-settings-stream-attenuation-strength"
        ));
    }
    let rows = vec![
        toggle(
            "stream-previews",
            requiem_i18n::t!("voice-settings-stream-previews"),
            plain(requiem_i18n::t!(
                "voice-settings-stream-previews-description"
            )),
            !settings.stream_previews_disabled,
            false,
            |s, on| s.stream_previews_disabled = !on,
        ),
        AccordionProps {
            title_closed: requiem_i18n::t!("voice-settings-streaming-advanced-show"),
            title_open: requiem_i18n::t!("voice-settings-streaming-advanced-hide"),
            subtitle_closed: Some(list_subtitle(&titles, 3)),
            open: page.stream_advanced_open.get(),
            on_toggle: Rc::new(|| settings_page().stream_advanced_open.update(|o| *o = !*o)),
            content: div().class(settings_page_cls::ROWS).children(advanced),
        }
        .build(),
    ];
    category(Some(requiem_i18n::t!("voice-settings-streaming")), rows)
}

fn sound_row(sound: Sound, title: &'static str, enabled: bool) -> Node {
    let preview: Rc<dyn Fn(&str)> = Rc::new(move |_url: &str| sounds::preview(sound));
    let description = rich(
        requiem_i18n::t!("voice-settings-preview-sound", onClick = PREVIEW_LINK),
        Some(preview),
    );
    let mut props = SwitchProps::new(switch_state(sound.id(), enabled), move |on: bool| {
        change(|s| {
            s.disabled_sounds.retain(|id| id != sound.id());
            if !on {
                s.disabled_sounds.push(sound.id().to_string());
            }
        })
    })
    .label(requiem_i18n::text(title));
    props.description_node = Some(description);
    setting(props.build())
}

fn sounds_category(settings: &VoiceSettings, settings_ui: &Signal<SettingsUiState>) -> Node {
    let page = settings_page();
    let mut rows = Vec::new();
    let divider = settings_page::divider;
    for (i, (sound, title)) in SOUNDS.iter().take(SOUNDS_ALWAYS_SHOWN).enumerate() {
        if i > 0 {
            rows.push(divider());
        }
        rows.push(sound_row(
            *sound,
            title,
            settings.is_sound_enabled(sound.id()),
        ));
    }
    let hidden = &SOUNDS[SOUNDS_ALWAYS_SHOWN..];
    let open = page.sounds_open.get();
    let mut more = Vec::new();
    for (i, (sound, title)) in hidden.iter().enumerate() {
        if i > 0 {
            more.push(divider());
        }
        more.push(sound_row(
            *sound,
            title,
            settings.is_sound_enabled(sound.id()),
        ));
    }
    let titles: Vec<String> = hidden
        .iter()
        .map(|(_, title)| requiem_i18n::text(title))
        .collect();
    let count = hidden.len() as i64;
    rows.push(divider());
    rows.push(
        AccordionProps {
            title_closed: requiem_i18n::t!("voice-settings-sounds-show", count = count),
            title_open: requiem_i18n::t!("voice-settings-sounds-hide", count = count),
            subtitle_closed: Some(list_subtitle(&titles, 3)),
            open,
            on_toggle: Rc::new(|| settings_page().sounds_open.update(|o| *o = !*o)),
            content: div().class(settings_page_cls::LIST).children(more),
        }
        .build(),
    );

    let settings_ui = settings_ui.clone();
    rows.push(settings_page::related(
        requiem_i18n::t!("voice-settings-notifications"),
        requiem_i18n::t!("voice-settings-notification-sounds-description"),
        BELL,
        move || {
            settings_ui.update(|ui| {
                ui.category = requiem_i18n::key!("settings-modal-notifications").to_string()
            })
        },
    ));
    category(Some(requiem_i18n::t!("voice-settings-sounds")), rows)
}

fn soundboard_category(settings: &VoiceSettings) -> Node {
    let volume = SliderProps::default()
        .value(volume_to_slider(settings.soundboard_volume))
        .max(100.0_f32)
        .on_change(|value: f32| {
            voice_settings::update(|s| s.soundboard_volume = slider_to_volume(value, 100.0))
        });
    let mut volume_field = FormItemProps::new(
        requiem_i18n::t!("voice-settings-soundboard-volume"),
        volume.build(),
    );
    volume_field.description_node = plain(requiem_i18n::t!(
        "voice-settings-soundboard-volume-description",
        helpCenterArticle = SOUNDBOARD_URL
    ));
    category(
        Some(requiem_i18n::t!("voice-settings-soundboard")),
        vec![setting(volume_field.build())],
    )
}

fn advanced_category(settings: &VoiceSettings) -> Node {
    let page = settings_page();
    let open_logs: Rc<dyn Fn()> = Rc::new(|| {
        if let Some(dir) = voice::diag::log_dir() {
            let _ = fs::create_dir_all(&dir);
            link::open_url(&dir.to_string_lossy());
        }
    });
    let debug = div().class(cls::STACK_4).children([
        {
            let mut props = SwitchProps::new(
                switch_state("debug-logging", settings.debug_logging),
                |on: bool| change(|s| s.debug_logging = on),
            )
            .label(requiem_i18n::t!("voice-settings-debug-logging"));
            props.description_node =
                plain(requiem_i18n::t!("voice-settings-debug-logging-description"));
            props.build()
        },
        div().class(settings_page_cls::BUTTON_ROW).child(
            ButtonProps::new(
                ButtonVariant::Secondary,
                requiem_i18n::t!("voice-settings-show-logs-folder"),
                move || open_logs(),
            )
            .build(),
        ),
    ]);
    let diagnostics = vec![
        toggle(
            "stream-info",
            requiem_i18n::t!("voice-settings-stream-info-overlay"),
            plain(requiem_i18n::t!(
                "voice-settings-stream-info-overlay-description"
            )),
            settings.stream_info_overlay,
            false,
            |s, on| s.stream_info_overlay = on,
        ),
        toggle(
            "aec-dump",
            requiem_i18n::t!("voice-settings-audio-recording"),
            plain(requiem_i18n::t!(
                "voice-settings-audio-recording-description"
            )),
            settings.aec_dump,
            false,
            |s, on| s.aec_dump = on,
        ),
        setting(debug),
    ];
    let reset = ControlItemProps::new(requiem_i18n::t!("voice-settings-reset"))
        .subtitle(requiem_i18n::t!("voice-settings-reset-description"))
        .trailing(
            ButtonProps::new(
                ButtonVariant::CriticalSecondary,
                requiem_i18n::t!("voice-settings-reset-button"),
                || open_confirm(Confirm::Reset),
            )
            .build(),
        )
        .build();
    let rows = vec![
        AccordionProps {
            title_closed: requiem_i18n::t!("voice-settings-diagnostics-show"),
            title_open: requiem_i18n::t!("voice-settings-diagnostics-hide"),
            subtitle_closed: Some(requiem_i18n::t!("voice-settings-diagnostics-description")),
            open: page.diagnostics_open.get(),
            on_toggle: Rc::new(|| settings_page().diagnostics_open.update(|o| *o = !*o)),
            content: div().class(settings_page_cls::ROWS).children(diagnostics),
        }
        .build(),
        reset,
    ];
    category(Some(requiem_i18n::t!("voice-settings-advanced")), rows)
}

pub fn build_page(settings_ui: &Signal<SettingsUiState>) -> Node {
    ensure_ticker(settings_ui);
    let settings_ui = settings_ui.clone();
    let content = Node::reactive(move || {
        let _ = settings_page().revision.get();
        let settings = voice_settings::get();
        let categories = [
            anchored(0, voice_category(&settings)),
            input_profile_category(&settings, &settings_ui),
            anchored(1, camera_category(&settings)),
            anchored(2, streaming_category(&settings)),
            anchored(3, sounds_category(&settings, &settings_ui)),
            anchored(4, soundboard_category(&settings)),
            anchored(5, advanced_category(&settings)),
        ];
        settings_page::panel(categories.into())
    });
    settings_page::page_container(content)
}

fn open_confirm(confirm: Confirm) {
    let page = settings_page();
    page.confirm.set(Some(confirm));
    page.dialog.open();
}

fn close_confirm() {
    settings_page().dialog.close();
}

pub fn join_voice(state: &AppState, guild_id: u64, channel: &models::Channel) {
    let in_other = voice::current_channel().is_some_and(|(_, current)| current != channel.id.get());
    if in_other && !voice_settings::get().disable_voice_channel_change_alert {
        let (state, channel_owned) = (state.clone(), channel.clone());
        let join: Rc<dyn Fn()> = Rc::new(move || {
            voice::join_channel(&state, guild_id, &channel_owned);
        });
        open_confirm(Confirm::SwitchChannel {
            channel: channel.name.clone().unwrap_or_default(),
            join,
        });
        return;
    }
    voice::join_channel(state, guild_id, channel);
}

fn confirm_dialog() -> Node {
    let page = settings_page();
    let Some(confirm) = page.confirm.get() else {
        return div();
    };
    let open = page.dialog.toggle();
    match confirm {
        Confirm::Reset => ConfirmModalProps::new(
            requiem_i18n::t!("voice-settings-reset-confirm-title"),
            requiem_i18n::t!("voice-settings-okay"),
            close_confirm,
            open,
        )
        .message(requiem_i18n::t!("voice-settings-reset-confirm"))
        .on_confirm(|| {
            close_confirm();
            settings_page().switches.clear();
            voice_settings::reset();
            settings_page().revision.update(|r| *r += 1);
        })
        .build(),
        Confirm::Subsystem(subsystem) => ConfirmModalProps::new(
            requiem_i18n::t!("voice-settings-audio-subsystem-confirm-title"),
            requiem_i18n::t!("voice-settings-okay"),
            close_confirm,
            open,
        )
        .message(requiem_i18n::t!("voice-settings-audio-subsystem-confirm"))
        .on_confirm(move || {
            close_confirm();
            change(|s| {
                s.audio_subsystem = subsystem;

                s.input_device = None;
                s.output_device = None;
            });
        })
        .build(),
        Confirm::SwitchChannel { channel, join } => {
            let body = components::message_text(MessageTextProps {
                content: requiem_i18n::md!("voice-settings-switch-channel-body", channel = channel),
                size: 16.0,
                line_height: Some(20.0),
                color: colors::TEXT_DEFAULT,
                ..Default::default()
            });
            ConfirmModalProps::new(
                requiem_i18n::t!("voice-settings-switch-channel-title"),
                requiem_i18n::t!("voice-settings-switch-channel-confirm"),
                close_confirm,
                open,
            )
            .content(body)
            .on_confirm(move || {
                close_confirm();
                join();
            })
            .build()
        }
    }
}

pub fn mount(tree: &mut WidgetTree, parent: WidgetId) -> Effect {
    section(tree, parent, move || {
        if settings_page().dialog.is_mounted() {
            confirm_dialog()
        } else {
            div()
        }
    })
}

pub fn end_on_close(settings_modal_open: Signal<ToggleState>, tree: &mut WidgetTree) -> Effect {
    requiem_ui_signal::create_effect(
        move |_| {
            if !settings_modal_open.get().on {
                on_leave()
            }
        },
        tree,
    )
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
