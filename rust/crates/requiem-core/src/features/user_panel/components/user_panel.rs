use crate::ui::audio_menu::Kind;
use crate::models::id::marker;
use crate::models;
use crate::ui::account_popout;
use crate::ui::audio_menu;
use crate::ui::classes::user_panel;
use crate::ui::context_menu;
use crate::ui::context_menus;
use crate::ui::custom_status;
use crate::ui::voice_panel;
use crate::ui;
use requiem_ui::components::ButtonVariant;
use requiem_ui::components::TextVariant;
use requiem_ui::components::line_clamp_class;
use requiem_ui::signal;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::Instant;

use requiem_ui::components::{
    AvatarStatusSize, ButtonProps, TextProps, TooltipProps, avatar_with_status, div,
};
use requiem_ui::node::Node;
use requiem_ui::signal::Signal;
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::anim::ToggleState;
use requiem_ui::widget::primitives::{
    AnimatedIconWidget, ChevronWidget, HoverRollWidget, TextWidget,
};
use requiem_ui::widget::{AbsoluteRect, Widget, WidgetId, WidgetTree};
use skia_safe::Canvas;
use skia_safe::font_style::Weight;

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::lottie::{asset_json, fix_mask_gaps, recolor_fills_and_strokes};
use crate::state::AppState;
use crate::ui::{Fonts, TEXT_SECONDARY, TEXT_STRONG};
use requiem_ui::icons::CHEVRON_LARGE_DOWN;

const MIC_LOTTIE: &str = "lottie/mic";

const MIC_FPS: f64 = 60.0;

const MIC_TRANSITION_FRAMES: f64 = 70.0;

const MIC_MUTE_START: f64 = 0.0;
const MIC_UNMUTE_START: f64 = 100.0;

const MIC_MUTED_RESTING_FRAME: f64 = 90.0;

const MIC_UNMUTED_RESTING_FRAME: f64 = 200.0;
const MIC_HOVER_UNMUTED_START: f64 = 180.0;
const MIC_HOVER_UNMUTED_END: f64 = 220.0;
const MIC_HOVER_MUTED_START: f64 = 240.0;
const MIC_HOVER_MUTED_END: f64 = 280.0;

const MIC_MUTED_RED: skia_safe::Color = skia_safe::Color::from_argb(255, 0xda, 0x3e, 0x44);

#[derive(Clone, Copy)]
enum IconColorState {
    Resting,

    Hovered,

    Danger,
}

fn sanitized_mic_json(state: IconColorState) -> &'static str {
    static GRAY: OnceLock<String> = OnceLock::new();
    static WHITE: OnceLock<String> = OnceLock::new();
    static RED: OnceLock<String> = OnceLock::new();

    fn base_pipeline() -> String {
        fix_mask_gaps(asset_json(MIC_LOTTIE)).expect("Discord's mic animation should be valid JSON")
    }

    fn recolored(cache: &'static OnceLock<String>, color: skia_safe::Color) -> &'static str {
        cache.get_or_init(|| {
            let json = base_pipeline();
            let (r, g, b) = (
                f64::from(color.r()) / 255.0,
                f64::from(color.g()) / 255.0,
                f64::from(color.b()) / 255.0,
            );
            recolor_fills_and_strokes(&json, r, g, b).expect("still valid JSON after recoloring")
        })
    }

    match state {
        IconColorState::Resting => recolored(&GRAY, TEXT_SECONDARY),
        IconColorState::Hovered => recolored(&WHITE, TEXT_STRONG),
        IconColorState::Danger => recolored(&RED, MIC_MUTED_RED),
    }
}

fn inset_icon_rect(rect: AbsoluteRect) -> AbsoluteRect {
    const ICON_SIZE: f32 = 20.0;
    AbsoluteRect {
        x: rect.x + (rect.width - ICON_SIZE) / 2.0,
        y: rect.y + (rect.height - ICON_SIZE) / 2.0,
        width: ICON_SIZE,
        height: ICON_SIZE,
    }
}

struct MicIconWidget {
    icon_gray: AnimatedIconWidget,
    icon_white: AnimatedIconWidget,
    icon_red: AnimatedIconWidget,
    mute_toggle: ToggleState,
    hover_started_at: Cell<Option<Instant>>,
}

impl MicIconWidget {
    fn new(mute_toggle: ToggleState) -> Self {
        let initial = if mute_toggle.on {
            MIC_MUTED_RESTING_FRAME
        } else {
            MIC_UNMUTED_RESTING_FRAME
        };
        Self {
            icon_gray: AnimatedIconWidget::new(
                sanitized_mic_json(IconColorState::Resting),
                initial,
                initial,
                false,
            ),
            icon_white: AnimatedIconWidget::new(
                sanitized_mic_json(IconColorState::Hovered),
                initial,
                initial,
                false,
            ),
            icon_red: AnimatedIconWidget::new(
                sanitized_mic_json(IconColorState::Danger),
                initial,
                initial,
                false,
            ),
            mute_toggle,
            hover_started_at: Cell::new(None),
        }
    }
}

impl Widget for MicIconWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let elapsed_secs = self.mute_toggle.changed_at.elapsed().as_secs_f64();
        let transition_span = MIC_TRANSITION_FRAMES / MIC_FPS;

        if style.is_hovered && self.hover_started_at.get().is_none() {
            self.hover_started_at.set(Some(Instant::now()));
        } else if !style.is_hovered {
            self.hover_started_at.set(None);
        }

        let (start, end, looping) = if elapsed_secs < transition_span {
            let start_frame = if self.mute_toggle.on {
                MIC_MUTE_START
            } else {
                MIC_UNMUTE_START
            };
            let frame = start_frame + elapsed_secs * MIC_FPS;
            (frame, frame, false)
        } else if let Some(hover_started_at) = self.hover_started_at.get() {
            let (hover_start, hover_end) = if self.mute_toggle.on {
                (MIC_HOVER_MUTED_START, MIC_HOVER_MUTED_END)
            } else {
                (MIC_HOVER_UNMUTED_START, MIC_HOVER_UNMUTED_END)
            };
            let hover_span = (hover_end - hover_start) / MIC_FPS;
            let hover_elapsed = hover_started_at.elapsed().as_secs_f64();
            let frame = hover_start + hover_elapsed.min(hover_span) * MIC_FPS;
            (frame, frame, false)
        } else {
            let resting = if self.mute_toggle.on {
                MIC_MUTED_RESTING_FRAME
            } else {
                MIC_UNMUTED_RESTING_FRAME
            };
            (resting, resting, false)
        };

        let icon = if self.mute_toggle.on {
            &self.icon_red
        } else if self.hover_started_at.get().is_some() {
            &self.icon_white
        } else {
            &self.icon_gray
        };
        icon.set_scene(start, end, looping);
        icon.paint(canvas, inset_icon_rect(rect), style);
    }
}

const DEAFEN_LOTTIE: &str = "lottie/deafen";

const DEAFEN_FPS: f64 = 60.0;

const DEAFEN_TRANSITION_FRAMES: f64 = 70.0;

const DEAFEN_DEAFEN_START: f64 = 0.0;
const DEAFEN_UNDEAFEN_START: f64 = 110.0;

const DEAFEN_DEAFENED_RESTING_FRAME: f64 = 90.0;

const DEAFEN_UNDEAFENED_RESTING_FRAME: f64 = 220.0;
const DEAFEN_HOVER_UNDEAFENED_START: f64 = 200.0;
const DEAFEN_HOVER_UNDEAFENED_END: f64 = 270.0;
const DEAFEN_HOVER_DEAFENED_START: f64 = 300.0;
const DEAFEN_HOVER_DEAFENED_END: f64 = 370.0;

fn sanitized_deafen_json(state: IconColorState) -> &'static str {
    static GRAY: OnceLock<String> = OnceLock::new();
    static WHITE: OnceLock<String> = OnceLock::new();
    static RED: OnceLock<String> = OnceLock::new();

    fn base_pipeline() -> String {
        fix_mask_gaps(asset_json(DEAFEN_LOTTIE))
            .expect("Discord's deafen animation should be valid JSON")
    }

    fn recolored(cache: &'static OnceLock<String>, color: skia_safe::Color) -> &'static str {
        cache.get_or_init(|| {
            let json = base_pipeline();
            let (r, g, b) = (
                f64::from(color.r()) / 255.0,
                f64::from(color.g()) / 255.0,
                f64::from(color.b()) / 255.0,
            );
            recolor_fills_and_strokes(&json, r, g, b).expect("still valid JSON after recoloring")
        })
    }

    match state {
        IconColorState::Resting => recolored(&GRAY, TEXT_SECONDARY),
        IconColorState::Hovered => recolored(&WHITE, TEXT_STRONG),
        IconColorState::Danger => recolored(&RED, MIC_MUTED_RED),
    }
}

struct DeafenIconWidget {
    icon_gray: AnimatedIconWidget,
    icon_white: AnimatedIconWidget,
    icon_red: AnimatedIconWidget,
    deafen_toggle: ToggleState,
    hover_started_at: Cell<Option<Instant>>,
}

impl DeafenIconWidget {
    fn new(deafen_toggle: ToggleState) -> Self {
        let initial = if deafen_toggle.on {
            DEAFEN_DEAFENED_RESTING_FRAME
        } else {
            DEAFEN_UNDEAFENED_RESTING_FRAME
        };
        Self {
            icon_gray: AnimatedIconWidget::new(
                sanitized_deafen_json(IconColorState::Resting),
                initial,
                initial,
                false,
            ),
            icon_white: AnimatedIconWidget::new(
                sanitized_deafen_json(IconColorState::Hovered),
                initial,
                initial,
                false,
            ),
            icon_red: AnimatedIconWidget::new(
                sanitized_deafen_json(IconColorState::Danger),
                initial,
                initial,
                false,
            ),
            deafen_toggle,
            hover_started_at: Cell::new(None),
        }
    }
}

impl Widget for DeafenIconWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let elapsed_secs = self.deafen_toggle.changed_at.elapsed().as_secs_f64();
        let transition_span = DEAFEN_TRANSITION_FRAMES / DEAFEN_FPS;

        if style.is_hovered && self.hover_started_at.get().is_none() {
            self.hover_started_at.set(Some(Instant::now()));
        } else if !style.is_hovered {
            self.hover_started_at.set(None);
        }

        let (start, end, looping) = if elapsed_secs < transition_span {
            let start_frame = if self.deafen_toggle.on {
                DEAFEN_DEAFEN_START
            } else {
                DEAFEN_UNDEAFEN_START
            };
            let frame = start_frame + elapsed_secs * DEAFEN_FPS;
            (frame, frame, false)
        } else if let Some(hover_started_at) = self.hover_started_at.get() {
            let (hover_start, hover_end) = if self.deafen_toggle.on {
                (DEAFEN_HOVER_DEAFENED_START, DEAFEN_HOVER_DEAFENED_END)
            } else {
                (DEAFEN_HOVER_UNDEAFENED_START, DEAFEN_HOVER_UNDEAFENED_END)
            };
            let hover_span = (hover_end - hover_start) / DEAFEN_FPS;
            let hover_elapsed = hover_started_at.elapsed().as_secs_f64();
            let frame = hover_start + hover_elapsed.min(hover_span) * DEAFEN_FPS;
            (frame, frame, false)
        } else {
            let resting = if self.deafen_toggle.on {
                DEAFEN_DEAFENED_RESTING_FRAME
            } else {
                DEAFEN_UNDEAFENED_RESTING_FRAME
            };
            (resting, resting, false)
        };

        let icon = if self.deafen_toggle.on {
            &self.icon_red
        } else if self.hover_started_at.get().is_some() {
            &self.icon_white
        } else {
            &self.icon_gray
        };
        icon.set_scene(start, end, looping);
        icon.paint(canvas, inset_icon_rect(rect), style);
    }
}

const SETTINGS_LOTTIE: &str = "lottie/settings";

const SETTINGS_HOVER_START: f64 = 0.0;
const SETTINGS_HOVER_END: f64 = 66.0;
const SETTINGS_FPS: f64 = 60.0;

fn sanitized_settings_json() -> &'static str {
    static CLEANED: OnceLock<String> = OnceLock::new();
    CLEANED.get_or_init(|| {
        let json = fix_mask_gaps(asset_json(SETTINGS_LOTTIE))
            .expect("Discord's settings animation should be valid JSON");
        let (r, g, b) = (
            f64::from(TEXT_SECONDARY.r()) / 255.0,
            f64::from(TEXT_SECONDARY.g()) / 255.0,
            f64::from(TEXT_SECONDARY.b()) / 255.0,
        );
        recolor_fills_and_strokes(&json, r, g, b).expect("still valid JSON after recoloring")
    })
}

struct SettingsIconWidget {
    icon: AnimatedIconWidget,
    hover_started_at: Cell<Option<Instant>>,
}

impl SettingsIconWidget {
    fn new() -> Self {
        Self {
            icon: AnimatedIconWidget::new(
                sanitized_settings_json(),
                SETTINGS_HOVER_START,
                SETTINGS_HOVER_START,
                false,
            ),
            hover_started_at: Cell::new(None),
        }
    }
}

impl Widget for SettingsIconWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        if style.is_hovered && self.hover_started_at.get().is_none() {
            self.hover_started_at.set(Some(Instant::now()));
        } else if !style.is_hovered {
            self.hover_started_at.set(None);
        }

        let frame = match self.hover_started_at.get() {
            Some(hover_started_at) => {
                let hover_span = (SETTINGS_HOVER_END - SETTINGS_HOVER_START) / SETTINGS_FPS;
                let hover_elapsed = hover_started_at.elapsed().as_secs_f64();
                SETTINGS_HOVER_START + hover_elapsed.min(hover_span) * SETTINGS_FPS
            }
            None => SETTINGS_HOVER_START,
        };

        self.icon.set_scene(frame, frame, false);
        self.icon.paint(canvas, inset_icon_rect(rect), style);
    }
}

thread_local! {

    static MENU_AT: [Cell<Option<(f32, f32)>>; 2] = const { [Cell::new(None), Cell::new(None)] };
}

fn split_button(
    icon_glyph: Node,
    on_click: Option<Rc<dyn Fn()>>,
    danger: bool,
    kind: Kind,
    tooltip: String,
) -> Node {
    let slot = if kind == Kind::Input { 0 } else { 1 };

    let group_rect: Signal<AbsoluteRect> = signal::signal(AbsoluteRect::default());
    let open_menu = {
        let group_rect = group_rect.clone();
        Rc::new(move || {
            let rect = group_rect.peek();
            MENU_AT.with(|at| at[slot].set(Some((rect.x, rect.y - 4.0))));
            audio_menu::open(kind, rect.x, rect.y);
        })
    };
    let icon_glyph = icon_glyph.class(user_panel::USER_PANEL_ICON);
    let mut main_classes = vec![user_panel::USER_PANEL_SPLIT_MAIN.to_string()];
    if danger {
        main_classes.push(user_panel::USER_PANEL_SPLIT_DANGER.to_string());
    }
    let mut main = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(main_classes);
    if let Some(on_click) = on_click {
        main = main.on_click(move || on_click());
    }
    let main = {
        let open_menu = open_menu.clone();
        main.build()
            .child(icon_glyph)
            .on_context_menu(move |_, _| open_menu())
    };
    let main = TooltipProps::default().text(tooltip).build(main);

    let chevron_color = if danger {
        MIC_MUTED_RED
    } else {
        TEXT_SECONDARY
    };
    let chevron_glyph = Node::reactive(move || {
        let open = MENU_AT
            .with(|at| at[slot].get())
            .is_some_and(|at| context_menu::origin_tracked() == Some(at));
        Node::new(
            "span",
            ChevronWidget::new(CHEVRON_LARGE_DOWN, chevron_color, open),
        )
        .class(user_panel::USER_PANEL_CHEVRON_ICON)
    });
    let mut chevron_classes =
        vec![user_panel::USER_PANEL_SPLIT_CHEVRON.to_string()];
    if danger {
        chevron_classes.push(user_panel::USER_PANEL_SPLIT_DANGER.to_string());
    }
    let chevron = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(chevron_classes)
        .on_click(move || open_menu())
        .build()
        .child(chevron_glyph);
    let label = if kind == Kind::Input {
        requiem_i18n::t!("voice-input-options")
    } else {
        requiem_i18n::t!("voice-output-options")
    };
    let chevron = TooltipProps::default()
        .text(label.to_string())
        .build(chevron);

    div()
        .class(user_panel::USER_PANEL_SPLIT_GROUP)
        .observe_rect(group_rect)
        .children([main, chevron])
}

#[allow(clippy::too_many_arguments)]
pub fn build(
    tree: &mut WidgetTree,
    parent: WidgetId,
    state: &AppState,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    mute_toggle: ToggleState,
    on_toggle_mute: &Rc<dyn Fn()>,
    deafen_toggle: ToggleState,
    on_toggle_deafen: &Rc<dyn Fn()>,
    on_open_settings: &Rc<dyn Fn()>,
    account: &account_popout::AccountPopoutState,
    on_select_channel: &Rc<dyn Fn(models::Id<marker::ChannelMarker>)>,
) {
    let Some(user) = state.users.current.get() else {
        return;
    };
    let user = &user;

    let image = user.avatar.as_deref().and_then(|hash| {
        let url = backend.avatar_url(user.id, hash);
        images.get_or_decode_image(&url)
    });

    let avatar_image = image.clone();
    let mut avatar_props = ui::user_avatar_props(user.id, &user.discriminator, fonts);
    if let Some(image) = image {
        avatar_props = avatar_props.image(image);
    }
    let own_status = state.presence.status.get();
    let popout_open = account.open.get();
    let avatar = avatar_with_status(
        avatar_props,
        AvatarStatusSize::Px32,
        Some(own_status.indicator()),
    );

    let display_name = user
        .global_name
        .clone()
        .unwrap_or_else(|| user.username.clone());

    let subtext = |content: &str| {
        TextWidget::new_weighted(
            content,
            12.0,
            TEXT_SECONDARY,
            fonts.text_fonts.font_system.clone(),
            fonts.text_fonts.glyph_cache.clone(),
            Weight::MEDIUM,
        )
        .with_line_height(13.0)
    };

    let custom = state.presence.custom_status.get().and_then(|c| {
        custom_status::inline_status_widget(
            &c,
            12.0,
            13.0,
            14.0,
            Weight::MEDIUM,
            TEXT_SECONDARY,
            fonts,
        )
    });
    let roll = match custom {
        Some(line) => HoverRollWidget::new(
            line,
            subtext(&user.username),
            account.panel_hover.clone(),
            popout_open,
        ),
        None => HoverRollWidget::new(
            subtext(&own_status.label()),
            subtext(&user.username),
            account.panel_hover.clone(),
            popout_open,
        ),
    };

    let roll = if state.voice.mine().is_some() {
        HoverRollWidget::new(
            subtext(&requiem_i18n::t!("voice-in-voice")),
            subtext(&user.username),
            account.panel_hover.clone(),
            popout_open,
        )
    } else {
        roll
    };
    let name_col = div()
        .class(user_panel::USER_PANEL_NAME_COL)
        .children([
            TextProps::styled(display_name, TextVariant::TEXT_MD_MEDIUM)
                .line_height(18.0_f32)
                .color(TEXT_STRONG)
                .line_clamp(1_u32)
                .build()
                .class(user_panel::USER_PANEL_TITLE),
            Node::new("div", roll)
                .class(user_panel::USER_PANEL_SUBTEXT)
                .class(line_clamp_class(1)),
        ]);

    let account_btn = {
        let (toggle, hover) = (account.clone(), account.panel_hover.clone());
        let mut classes = vec![user_panel::USER_PANEL_ACCOUNT_BTN.to_string()];
        if popout_open {
            classes.push(user_panel::USER_PANEL_ACCOUNT_BTN_ACTIVE.to_string());
        }
        ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(classes)
            .on_click(move || toggle.toggle())
            .on_hover_change(move |hovered| hover.set(hovered))
            .build()
            .children([avatar, name_col])
    };

    let mine = state.voice.mine();
    let deafened = deafen_toggle.on || mine.as_ref().is_some_and(|s| s.deaf);
    let muted = mute_toggle.on || deafened || mine.as_ref().is_some_and(|s| s.mute || s.suppress);
    let deafen_toggle = if deafened && !deafen_toggle.on {
        ToggleState::new(true)
    } else {
        deafen_toggle
    };
    let mute_toggle = if muted && !mute_toggle.on {
        ToggleState::new(true)
    } else {
        mute_toggle
    };
    let mic_icon = MicIconWidget::new(mute_toggle);
    let on_toggle_mute = on_toggle_mute.clone();

    let mute_tip = if mine.as_ref().is_some_and(|s| s.suppress) {
        requiem_i18n::t!("voice-suppressed")
    } else if mine.as_ref().is_some_and(|s| s.mute) {
        requiem_i18n::t!("voice-server-muted")
    } else if mute_toggle.on {
        requiem_i18n::t!("voice-unmute")
    } else {
        requiem_i18n::t!("voice-mute")
    };
    let mute_btn = split_button(
        Node::new("span", mic_icon),
        Some(on_toggle_mute),
        mute_toggle.on,
        Kind::Input,
        mute_tip.to_string(),
    );

    let deafen_icon = DeafenIconWidget::new(deafen_toggle);
    let on_toggle_deafen = on_toggle_deafen.clone();
    let deafen_tip = if mine.as_ref().is_some_and(|s| s.deaf) {
        requiem_i18n::t!("voice-server-deafened")
    } else if deafen_toggle.on {
        requiem_i18n::t!("voice-undeafen")
    } else {
        requiem_i18n::t!("voice-deafen")
    };
    let deafen_btn = split_button(
        Node::new("span", deafen_icon),
        Some(on_toggle_deafen),
        deafen_toggle.on,
        Kind::Output,
        deafen_tip.to_string(),
    );

    let settings_glyph = Node::new("span", SettingsIconWidget::new())
        .class(user_panel::USER_PANEL_ICON);
    let on_open_settings = on_open_settings.clone();
    let settings_btn = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![
            user_panel::USER_PANEL_ICON_BTN.to_string(),
        ])
        .on_click(move || on_open_settings())
        .build()
        .on_context_menu({
            let (user, avatar, fonts) = (user.clone(), avatar_image.clone(), fonts.clone());
            move |x: f32, y: f32| {
                let (user, avatar, fonts) = (user.clone(), avatar.clone(), fonts.clone());
                context_menu::open(x, y, move || {
                    let mut props =
                        ui::user_avatar_props(user.id, &user.discriminator, &fonts);
                    if let Some(image) = &avatar {
                        props = props.image(image.clone());
                    }
                    context_menus::settings_items(props.build())
                })
            }
        })
        .child(settings_glyph);
    let settings_btn = TooltipProps::default()
        .text(requiem_i18n::t!("voice-user-settings").to_string())
        .build(settings_btn);

    let panel = div()
        .class(user_panel::USER_PANEL)
        .children([account_btn, mute_btn, deafen_btn, settings_btn]);

    let wrap = div()
        .class(user_panel::USER_PANEL_WRAP)
        .child(voice_panel::build(state, on_select_channel))
        .child(panel);
    tree.mount(parent, wrap);
}
