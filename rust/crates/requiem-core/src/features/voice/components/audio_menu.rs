use crate::ui::classes::voice_users;
use crate::ui;
use crate::voice_control;
use requiem_ui::components::{ContextItem, SliderProps, TextProps, TextVariant, div};
use requiem_ui::node::Node;

use crate::ui::context_menu as menu;
use crate::utils::voice_settings::{
    self, InputMode, InputProfile, slider_to_volume, volume_to_slider,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Input,
    Output,
}

pub fn open(kind: Kind, x: f32, y: f32) {
    menu::open_above(x, y - 4.0, move || items(kind));
}

fn items(kind: Kind) -> Vec<ContextItem> {
    let settings = voice_settings::get();
    let mut items = Vec::new();
    match kind {
        Kind::Input => {
            items.push(device_submenu(true, settings.input_device.clone()));
            let profile = settings
                .active_input_profile
                .unwrap_or(InputProfile::Custom);
            items.push(
                ContextItem::action(requiem_i18n::t!("voice-settings-input-profile"), || {})
                    .subtitle(profile_label(profile))
                    .submenu(|| {
                        let active = voice_settings::get()
                            .active_input_profile
                            .unwrap_or(InputProfile::Custom);
                        InputProfile::ALL
                            .into_iter()
                            .map(|profile| {
                                ContextItem::action(profile_label(profile), move || {
                                    voice_settings::update(|s| {
                                        s.active_input_profile = Some(profile)
                                    })
                                })
                                .radio(profile == active)
                            })
                            .collect()
                    }),
            );
            items.push(ContextItem::Separator);
            items.push(ContextItem::Custom(volume_row(
                requiem_i18n::t!("voice-input-volume"),
                settings.input_volume,
                100.0,
                |s, v| s.input_volume = v,
            )));
            items.push(ContextItem::Separator);

            if !settings.mode_options.shortcut.is_empty() {
                let ptt = settings.mode == InputMode::PushToTalk;
                let studio = settings.active_input_profile == Some(InputProfile::Studio);
                let mut item =
                    ContextItem::action(requiem_i18n::t!("voice-push-to-talk"), move || {
                        voice_settings::update(|s| {
                            s.mode = if ptt {
                                InputMode::VoiceActivity
                            } else {
                                InputMode::PushToTalk
                            }
                        })
                    })
                    .checkbox(ptt);
                if studio {
                    item = item.disabled();
                }
                items.push(item);
            }
        }
        Kind::Output => {
            items.push(device_submenu(false, settings.output_device.clone()));
            items.push(ContextItem::Separator);
            items.push(ContextItem::Custom(volume_row(
                requiem_i18n::t!("voice-output-volume"),
                settings.output_volume,
                200.0,
                |s, v| s.output_volume = v,
            )));
            items.push(ContextItem::Separator);
        }
    }
    items.push(ContextItem::action(
        requiem_i18n::t!("voice-voice-settings"),
        || (menu::actions().open_settings)(requiem_i18n::key!("settings-modal-voice-video")),
    ));
    items
}

fn profile_label(profile: InputProfile) -> String {
    match profile {
        InputProfile::VoiceIsolation => requiem_i18n::t!("voice-settings-profile-isolation"),
        InputProfile::Studio => requiem_i18n::t!("voice-settings-profile-studio"),
        InputProfile::Custom => requiem_i18n::t!("voice-settings-profile-custom"),
    }
    .to_string()
}

fn device_submenu(input: bool, chosen: Option<String>) -> ContextItem {
    let label = if input {
        requiem_i18n::t!("voice-input-device")
    } else {
        requiem_i18n::t!("voice-output-device")
    };
    let default_label = requiem_i18n::t!("voice-default-device").to_string();
    ContextItem::action(label, || {})
        .subtitle(chosen.clone().unwrap_or_else(|| default_label.clone()))
        .submenu(move || {
            let chosen = if input {
                voice_settings::get().input_device
            } else {
                voice_settings::get().output_device
            };
            let set = move |device: Option<String>| {
                voice_settings::update(|s| {
                    if input {
                        s.input_device = device
                    } else {
                        s.output_device = device
                    }
                });
            };
            let mut items = vec![
                ContextItem::action(requiem_i18n::t!("voice-default-device"), move || set(None))
                    .radio(chosen.is_none()),
            ];
            for name in voice_control::devices(input) {
                let selected = chosen.as_deref() == Some(name.as_str());
                let value = name.clone();
                items.push(
                    ContextItem::action(name, move || set(Some(value.clone()))).radio(selected),
                );
            }
            items
        })
}

fn volume_row(
    label: String,
    volume: f32,
    max: f32,
    set: fn(&mut voice_settings::VoiceSettings, f32),
) -> Node {
    let slider = SliderProps::default()
        .value(volume_to_slider(volume))
        .max(max)
        .on_change(move |value: f32| {
            voice_settings::update(|s| set(s, slider_to_volume(value, max)))
        })
        .build();
    div()
        .class(voice_users::VOICE_MENU_VOLUME)
        .child(
            TextProps::styled(label, TextVariant::TEXT_SM_MEDIUM)
                .color(ui::TEXT_SECONDARY)
                .build(),
        )
        .child(slider)
}
