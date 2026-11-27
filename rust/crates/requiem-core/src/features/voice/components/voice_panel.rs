use crate::ui;
use requiem_ui::signal;
use std::rc::Rc;

use requiem_ui::colors::{TEXT_CRITICAL, TEXT_POSITIVE, TEXT_STRONG, TEXT_SUBTLE, WARNING};
use requiem_ui::components::{
    ButtonProps, ButtonVariant, TextProps, TextVariant, TooltipProps, div, icon,
};
use requiem_ui::node::Node;
use skia_safe::Color;

use crate::models::Id;
use crate::models::id::marker::ChannelMarker;
use crate::state::{AppState, RtcState};
use crate::ui::classes::voice_panel as cls;
use crate::ui::lottie_icon;
use crate::voice_control;
use requiem_ui::icons::{APPS, WAVEFORM};

const ICON_ON_WHITE: Color = Color::from_argb(255, 0x1a, 0x1a, 0x1e);

pub fn build(state: &AppState, on_select_channel: &Rc<dyn Fn(Id<ChannelMarker>)>) -> Node {
    let (state, on_select_channel) = (state.clone(), on_select_channel.clone());
    Node::reactive(move || panel(&state, &on_select_channel))
}

thread_local! {

    static REFRESH: signal::Signal<u64> = signal::signal(0);
}

fn refresh() {
    REFRESH.with(|r| r.update(|n| *n += 1));
}

fn panel(state: &AppState, on_select_channel: &Rc<dyn Fn(Id<ChannelMarker>)>) -> Node {
    let _ = REFRESH.with(|r| r.get());
    let rtc = state.voice.rtc.get();
    let Some(channel_id) = rtc
        .channel_id
        .filter(|_| rtc.state != RtcState::Disconnected)
    else {
        return div();
    };
    let channel = ui::find_channel(state, channel_id).map(|(_, channel)| channel);
    let channel_name = channel
        .as_ref()
        .and_then(|c| c.name.clone())
        .unwrap_or_default();
    let guild_name = rtc
        .guild_id
        .and_then(|g| {
            state
                .guilds
                .list
                .with(|list| list.iter().find(|x| x.id == g).map(|x| x.name.clone()))
        })
        .unwrap_or_default();
    let mine = state.voice.mine();
    let camera_on = mine.as_ref().is_some_and(|s| s.self_video);
    let streaming = mine.as_ref().is_some_and(|s| s.self_stream);

    let (label, color) = match rtc.state {
        RtcState::RtcConnected if camera_on => {
            (requiem_i18n::t!("voice-video-connected"), TEXT_POSITIVE)
        }
        RtcState::RtcConnected => (requiem_i18n::t!("voice-connected"), TEXT_POSITIVE),
        RtcState::AwaitingEndpoint => (requiem_i18n::t!("voice-awaiting-endpoint"), WARNING),
        RtcState::Authenticating => (requiem_i18n::t!("voice-authenticating"), WARNING),
        RtcState::RtcConnecting => (requiem_i18n::t!("voice-rtc-connecting"), WARNING),
        RtcState::NoRoute => (requiem_i18n::t!("voice-no-route"), TEXT_CRITICAL),
        RtcState::RtcDisconnected => (requiem_i18n::t!("voice-rtc-disconnected"), TEXT_CRITICAL),
        RtcState::Disconnected => (requiem_i18n::t!("voice-connecting"), WARNING),
    };
    let (quality_class, glyph_color) = match (rtc.state, rtc.ping) {
        (RtcState::RtcConnected, Some(ms)) if ms < 250 => {
            (Some(cls::VOICE_PANEL_PING_FINE), TEXT_POSITIVE)
        }
        (RtcState::RtcConnected, Some(ms)) if ms < 500 => {
            (Some(cls::VOICE_PANEL_PING_AVERAGE), WARNING)
        }
        (RtcState::RtcConnected, Some(_)) => (Some(cls::VOICE_PANEL_PING_BAD), TEXT_CRITICAL),
        (RtcState::RtcConnected, None) => (Some(cls::VOICE_PANEL_PING_FINE), TEXT_POSITIVE),
        (RtcState::NoRoute | RtcState::RtcDisconnected, _) => {
            (Some(cls::VOICE_PANEL_PING_BAD), TEXT_CRITICAL)
        }
        _ => (Some(cls::VOICE_PANEL_PING_AVERAGE), WARNING),
    };
    let mut ping = div()
        .class(cls::VOICE_PANEL_PING)
        .child(icon(WAVEFORM, glyph_color).class(cls::VOICE_PANEL_PING_ICON));
    if let Some(quality) = quality_class {
        ping = ping.class(quality);
    }
    let ping = match rtc.ping {
        Some(ms) => TooltipProps::default()
            .text(requiem_i18n::t!("voice-ping", ms = ms))
            .build(ping),
        None => ping,
    };

    let label = TextProps::styled(label, TextVariant::TEXT_MD_MEDIUM)
        .color(color)
        .line_clamp(1_u32)
        .build()
        .class(cls::VOICE_PANEL_LABEL);
    let subtitle_text = if guild_name.is_empty() {
        channel_name
    } else {
        requiem_i18n::t!(
            "voice-panel-subtitle",
            channel = channel_name,
            guild = guild_name
        )
        .to_string()
    };
    let subtitle = {
        let on_select = on_select_channel.clone();
        ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::VOICE_PANEL_SUBTITLE.to_string()])
            .on_click(move || on_select(channel_id))
            .build()
            .child(
                TextProps::styled(subtitle_text, TextVariant::TEXT_XS_MEDIUM)
                    .color(TEXT_SUBTLE)
                    .line_clamp(1_u32)
                    .build(),
            )
    };
    let status = div().class(cls::VOICE_PANEL_STATUS).child(ping).child(
        div()
            .class(cls::VOICE_PANEL_LABELS)
            .child(label)
            .child(subtitle),
    );

    let noise = voice_control::noise_suppression();
    let noise_glyph = lottie_icon::toggle(
        "panel-noise",
        lottie_icon::NOISE_SUPPRESSION,
        noise,
        ui::TEXT_SECONDARY,
        TEXT_STRONG,
        20.0,
    );
    let noise_btn = icon_button(
        noise_glyph,
        requiem_i18n::t!("voice-noise-suppression").to_string(),
        move || {
            voice_control::set_noise_suppression(!noise);
            refresh();
        },
    );
    let disconnect_btn = icon_button(
        lottie_icon::hover(
            lottie_icon::END_CALL,
            ui::TEXT_SECONDARY,
            TEXT_STRONG,
            20.0,
        ),
        requiem_i18n::t!("voice-disconnect").to_string(),
        voice_control::disconnect,
    );
    let connection = div()
        .class(cls::VOICE_PANEL_CONNECTION)
        .child(status)
        .child(
            div()
                .class(cls::VOICE_PANEL_BUTTONS)
                .child(noise_btn)
                .child(disconnect_btn),
        );

    let guild_id = rtc.guild_id;
    let glyph_color = |active: bool| if active { ICON_ON_WHITE } else { TEXT_STRONG };
    let camera = action(
        lottie_icon::toggle(
            "panel-camera",
            lottie_icon::CAMERA,
            camera_on,
            glyph_color(camera_on),
            glyph_color(camera_on),
            20.0,
        ),
        if camera_on {
            requiem_i18n::t!("voice-turn-off-camera")
        } else {
            requiem_i18n::t!("voice-turn-on-camera")
        }
        .to_string(),
        camera_on,
        Some(Rc::new(move || voice_control::set_camera(!camera_on))),
    );
    let screen = action(
        lottie_icon::toggle(
            "panel-screen",
            lottie_icon::SCREENSHARE,
            streaming,
            glyph_color(streaming),
            glyph_color(streaming),
            20.0,
        ),
        requiem_i18n::t!("voice-share-screen").to_string(),
        streaming,
        guild_id
            .map(|g| Rc::new(move || voice_control::toggle_stream(g, channel_id)) as Rc<dyn Fn()>),
    );
    let activity = action(
        div()
            .class(cls::VOICE_PANEL_CENTER)
            .child(icon(APPS, TEXT_STRONG).class(cls::VOICE_PANEL_ICON)),
        requiem_i18n::t!("voice-start-activity").to_string(),
        false,
        None,
    );
    let soundboard = action(
        lottie_icon::hover(lottie_icon::SOUNDBOARD, TEXT_STRONG, TEXT_STRONG, 20.0),
        requiem_i18n::t!("voice-open-soundboard").to_string(),
        false,
        None,
    );
    let actions = div()
        .class(cls::VOICE_PANEL_ACTIONS)
        .children([camera, screen, activity, soundboard]);

    div()
        .class(cls::VOICE_PANEL)
        .child(connection)
        .child(actions)
}

fn icon_button(glyph: Node, label: String, on_click: impl Fn() + 'static) -> Node {
    let button = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::VOICE_PANEL_ICON_BTN.to_string()])
        .on_click(on_click)
        .build()
        .child(glyph.class(cls::VOICE_PANEL_GLYPH));
    TooltipProps::default().text(label).build(button)
}

fn action(glyph: Node, label: String, active: bool, on_click: Option<Rc<dyn Fn()>>) -> Node {
    let mut classes = vec![cls::VOICE_PANEL_ACTION.to_string()];
    if active {
        classes.push(cls::VOICE_PANEL_ACTION_ACTIVE.to_string());
    }
    let button = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(classes)
        .on_click(move || {
            if let Some(on_click) = &on_click {
                on_click();
            }
        })
        .build()
        .child(glyph.class(cls::VOICE_PANEL_GLYPH));
    TooltipProps::default()
        .text(label)
        .anchor_classes(vec![cls::VOICE_PANEL_ACTION_CELL.to_string()])
        .build(button)
}
