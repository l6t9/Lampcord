use crate::models::id::marker;
use crate::state::RtcState;
use crate::ui::context_menu;
use crate::ui::context_menus;
use crate::ui::invite_modal;
use crate::ui::lottie_icon;
use crate::ui::voice_users;
use crate::ui;
use requiem_ui::colors;
use requiem_ui::icons::Icon;
use requiem_ui::paint as requiem_ui_paint;
#[cfg(feature = "voice")]
use skia_safe::images;
use std::cell::RefCell;
use std::rc::Rc;

use requiem_ui::colors::{TEXT_SECONDARY, TEXT_STRONG};
use requiem_ui::components::{
    ButtonProps, ButtonVariant, TextProps, TextVariant, TooltipProps, div, icon,
};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::{AbsoluteRect, Widget};
use skia_safe::{Canvas, Color, Paint, RRect, Rect};

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::models::id::marker::{GuildMarker, UserMarker};
use crate::models::{Channel, Id, User};
use crate::state::{AppState, VoiceState};
use crate::ui::classes::call_view as cls;
use crate::voice_control;
use requiem_ui::icons::{
    APPS, CHEVRON_SMALL_DOWN, FULLSCREEN_ENTER, GROUP_PLUS, MICROPHONE_SLASH, MORE_HORIZONTAL,
    WINDOW_LAUNCH,
};

#[derive(Clone)]
pub struct CallEnv {
    pub images: ImageCache,
    pub backend: BackendConfig,
    pub fonts: ui::Fonts,

    pub tick: Signal<u64>,

    pub muted: bool,
    pub on_toggle_mute: Rc<dyn Fn()>,

    pub menu: context_menus::MenuEnv,
}

const GAP: f32 = 8.0;

thread_local! {

    static STAGE: Signal<(f32, f32)> = signal((0.0, 0.0));
}

pub fn build(
    state: &AppState,
    guild_id: Id<GuildMarker>,
    channel: &Channel,
    env: &CallEnv,
) -> Node {
    let (state, channel, env) = (state.clone(), channel.clone(), env.clone());
    Node::reactive(move || {
        let rtc = state.voice.rtc.get();
        let in_call =
            rtc.channel_id == Some(channel.id) && rtc.state != RtcState::Disconnected;
        if in_call {
            call(&state, guild_id, &channel, &env)
        } else {
            prejoin(&state, guild_id, &channel, &env)
        }
    })
    .class(cls::CALL_VIEW)
}

fn avatar(
    user_id: Id<UserMarker>,
    user: Option<&User>,
    env: &CallEnv,
    class: &'static str,
) -> Node {
    let discriminator = user
        .map(|u| u.discriminator.clone())
        .unwrap_or_else(|| "0".into());
    let mut props = ui::user_avatar_props(user_id, &discriminator, &env.fonts);
    if let Some(hash) = user.and_then(|u| u.avatar.clone()) {
        let (images, url) = (env.images.clone(), env.backend.avatar_url(user_id, &hash));
        if let Some(image) = images.get_or_decode_image(&url) {
            props = props.image(image);
        }
        props = props.bind_image(env.tick.clone(), move || images.get_or_decode_image(&url));
    }
    props.build().class(class)
}

fn tile_color(user_id: Id<UserMarker>, user: Option<&User>) -> Option<Color> {
    let user = user?;
    user.avatar
        .is_none()
        .then(|| ui::default_avatar_color(user_id, &user.discriminator))
}

fn prejoin(state: &AppState, guild_id: Id<GuildMarker>, channel: &Channel, env: &CallEnv) -> Node {
    let members = state.voice.in_channel(channel.id);
    let names: Vec<String> = members
        .iter()
        .map(|m| voice_users::display_of(state, guild_id, m).0)
        .collect();
    let subtitle = match names.as_slice() {
        [] => requiem_i18n::t!("voice-nobody-in-voice").to_string(),
        [a] => requiem_i18n::t!("voice-one-in-voice", a = a.clone()).to_string(),
        [a, b] => requiem_i18n::t!("voice-two-in-voice", a = a.clone(), b = b.clone()).to_string(),
        [a, rest @ ..] => requiem_i18n::t!(
            "voice-two-in-voice",
            a = a.clone(),
            b = format!("{}", rest.len())
        )
        .to_string(),
    };
    let mut page = div().class(cls::CALL_PREJOIN);
    if !members.is_empty() {
        let tiles = members.iter().take(5).map(|m| {
            let (_, user) = voice_users::display_of(state, guild_id, m);
            let mut tile = div().class(cls::CALL_PREJOIN_TILE).child(avatar(
                m.user_id,
                user.as_ref(),
                env,
                cls::CALL_PREJOIN_AVATAR,
            ));
            if let Some(color) = tile_color(m.user_id, user.as_ref()) {
                tile = tile.style(format!("background-color: {}", css_color(color)));
            }
            tile
        });
        page = page.child(div().class(cls::CALL_PREJOIN_TILES).children(tiles));
    }
    let name = channel.name.clone().unwrap_or_default();
    page = page.child(
        TextProps::styled(name, TextVariant::HEADING_XL_SEMIBOLD)
            .color(TEXT_STRONG)
            .build(),
    );
    page = page.child(
        TextProps::styled(subtitle, TextVariant::TEXT_SM_NORMAL)
            .color(TEXT_SECONDARY)
            .build(),
    );
    let (join_state, join_channel) = (state.clone(), channel.clone());
    let join = ButtonProps::new(
        ButtonVariant::OverlayPrimary,
        requiem_i18n::t!("voice-join"),
        move || {
            voice_control::join(&join_state, guild_id, &join_channel);
        },
    )
    .build()
    .class(cls::CALL_PREJOIN_JOIN);
    page.child(join)
}

fn css_color(color: Color) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b())
}

fn tile_size(count: usize, (w, h): (f32, f32)) -> (usize, f32, f32) {
    let count = count.max(1);
    let cols = (count as f32).sqrt().ceil() as usize;
    let rows = count.div_ceil(cols);
    let width = ((w - GAP * (cols - 1) as f32) / cols as f32)
        .min((h - GAP * (rows - 1) as f32) / rows as f32 * 16.0 / 9.0);
    (cols, width, width * 9.0 / 16.0)
}

fn call(state: &AppState, guild_id: Id<GuildMarker>, channel: &Channel, env: &CallEnv) -> Node {
    let stage_size = STAGE.with(Clone::clone);
    let tiles = {
        let (state, env, channel_id) = (state.clone(), env.clone(), channel.id);
        Node::reactive(move || {
            let members = state.voice.in_channel(channel_id);
            let speaking = state.voice.speaking.get();
            let video = state.voice.video.get();
            let me = state.users.current.with(|u| u.as_ref().map(|u| u.id));
            let size = stage_size.get();
            let alone = members.len() <= 1;
            let count = members.len() + usize::from(alone);
            let (cols, w, h) = tile_size(count, (size.0 - 16.0, size.1 - 16.0));
            if w <= 0.0 {
                return div();
            }

            let mut ordered = voice_users::sorted(&state, guild_id, &members);
            ordered.sort_by_key(|m| Some(m.user_id) == me);
            let mut nodes: Vec<Node> = ordered
                .into_iter()
                .map(|m| {
                    let (name, user) = voice_users::display_of(&state, guild_id, m);

                    let camera =
                        m.self_video && (Some(m.user_id) == me || video.contains(&m.user_id));
                    tile(
                        guild_id,
                        m,
                        &name,
                        user.as_ref(),
                        speaking.contains(&m.user_id),
                        camera,
                        (w, h),
                        &env,
                    )
                })
                .collect();
            if alone {
                nodes.push(invite_tile(guild_id, channel_id, (w, h)));
            }
            let mut rows = Vec::new();
            let mut nodes = nodes.into_iter().peekable();
            while nodes.peek().is_some() {
                rows.push(
                    div()
                        .class(cls::CALL_TILE_ROW)
                        .children(nodes.by_ref().take(cols).collect::<Vec<_>>()),
                );
            }
            div().class(cls::CALL_TILES).children(rows)
        })
    };
    let stage = div()
        .class(cls::CALL_STAGE)
        .observe_size(STAGE.with(Clone::clone))
        .child(tiles);
    div()
        .style("flex: 1 1 0px; display: flex; flex-direction: column; min-height: 0px")
        .child(stage)
        .child(controls(state, guild_id, channel, env))
}

#[allow(clippy::too_many_arguments)]
fn tile(
    guild_id: Id<GuildMarker>,
    voice: &VoiceState,
    name: &str,
    user: Option<&User>,
    speaking: bool,
    camera: bool,
    (w, h): (f32, f32),
    env: &CallEnv,
) -> Node {
    let mut tile = div()
        .class(cls::CALL_TILE)
        .style(format!("width: {w}px; height: {h}px"));
    if let Some(color) = tile_color(voice.user_id, user).filter(|_| !camera) {
        tile = tile.style(format!(
            "width: {w}px; height: {h}px; background-color: {}",
            css_color(color)
        ));
    }
    tile = if camera {
        tile.child(
            Node::new(
                "div",
                VideoTile {
                    user_id: voice.user_id,
                    image: RefCell::new(None),
                },
            )
            .class(cls::CALL_TILE_VIDEO)
            .style(format!("width: {w}px; height: {h}px")),
        )
    } else {
        tile.child(avatar(voice.user_id, user, env, cls::CALL_TILE_AVATAR))
    };

    let mut pill = div().class(cls::CALL_TILE_NAME);
    if voice.is_muted() {
        pill = pill.child(icon(MICROPHONE_SLASH, Color::WHITE).class(cls::CALL_TILE_NAME_ICON));
    }
    pill = pill.child(
        TextProps::styled(name.to_string(), TextVariant::TEXT_SM_MEDIUM)
            .color(Color::WHITE)
            .line_clamp(1_u32)
            .build(),
    );
    tile = tile.child(pill);

    let target = Rc::new(context_menus::UserTarget {
        user_id: voice.user_id,
        guild_id: Some(guild_id),
        display_name: name.to_string(),
        webhook: false,
    });
    let open_menu = {
        let (menu, target) = (env.menu.clone(), target.clone());
        Rc::new(move |x: f32, y: f32| {
            let (menu, target) = (menu.clone(), target.clone());
            context_menu::open(x, y, move || {
                context_menus::voice_user_items(&menu, &target)
            });
        })
    };
    let options = {
        let open_menu = open_menu.clone();
        let button = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::CALL_TILE_OPTIONS.to_string()])
            .on_click_at(move |rect: AbsoluteRect| open_menu(rect.x, rect.y + rect.height + 8.0))
            .build()
            .child(icon(MORE_HORIZONTAL, Color::WHITE).class(cls::CALL_TILE_OPTIONS_ICON));
        TooltipProps::default()
            .text(requiem_i18n::t!("voice-options").to_string())
            .build(button)
            .class(cls::CALL_TILE_OPTIONS_ANCHOR)
    };
    tile = tile
        .child(options)
        .on_context_menu(move |x, y| open_menu(x, y));
    if speaking {
        tile = tile.class(cls::CALL_TILE_SPEAKING);
    }
    tile
}

fn invite_tile(
    guild_id: Id<GuildMarker>,
    channel_id: Id<marker::ChannelMarker>,
    (w, h): (f32, f32),
) -> Node {
    let invite = ButtonProps::new(
        ButtonVariant::Secondary,
        requiem_i18n::t!("voice-invite"),
        move || invite_modal::open_channel(guild_id, channel_id),
    )
    .icon(GROUP_PLUS)
    .build();
    let activity = ButtonProps::new(
        ButtonVariant::Secondary,
        requiem_i18n::t!("voice-choose-activity"),
        || {},
    )
    .icon(APPS)
    .build();
    div()
        .class(cls::CALL_INVITE_TILE)
        .style(format!("width: {w}px; height: {h}px"))
        .child(
            div()
                .class(cls::CALL_INVITE_BUTTONS)
                .children([invite, activity]),
        )
}

fn controls(state: &AppState, guild_id: Id<GuildMarker>, channel: &Channel, env: &CallEnv) -> Node {
    let channel_id = channel.id;
    let mine = state.voice.mine();
    let camera_on = mine.as_ref().is_some_and(|s| s.self_video);
    let streaming = mine.as_ref().is_some_and(|s| s.self_stream);

    let tip = |node: Node, label: String| TooltipProps::default().text(label).build(node);
    let edge_icon = |glyph, label: String, on_click: Rc<dyn Fn()>| {
        tip(
            ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .on_click(move || on_click())
                .build()
                .child(icon(glyph, TEXT_SECONDARY).class(cls::CALL_EDGE_ICON)),
            label,
        )
    };
    let left = div().class(cls::CALL_EDGE).child(edge_icon(
        GROUP_PLUS,
        requiem_i18n::t!("voice-invite").to_string(),
        Rc::new(move || invite_modal::open_channel(guild_id, channel_id)),
    ));
    let right = div()
        .class(cls::CALL_EDGE)
        .class(cls::CALL_EDGE_END)
        .children([
            edge_icon(WINDOW_LAUNCH, String::new(), Rc::new(|| {})),
            edge_icon(FULLSCREEN_ENTER, String::new(), Rc::new(|| {})),
        ]);

    let button =
        |glyph: Node, label: String, extra: Option<&'static str>, on_click: Rc<dyn Fn()>| {
            let mut classes = vec![cls::CALL_BUTTON.to_string()];
            if let Some(extra) = extra {
                classes.push(extra.to_string());
            }
            tip(
                ButtonProps::default()
                    .variant(ButtonVariant::Bare)
                    .extra_classes(classes)
                    .on_click(move || on_click())
                    .build()
                    .child(glyph.class(cls::CALL_GLYPH)),
                label,
            )
        };
    let still = |glyph: Icon| {
        div()
            .class(cls::CALL_GLYPH_CENTER)
            .child(icon(glyph, Color::WHITE).class(cls::CALL_BUTTON_ICON))
    };
    let caret = || {
        div()
            .class(cls::CALL_CARET)
            .child(icon(CHEVRON_SMALL_DOWN, Color::WHITE).class(cls::CALL_CARET_ICON))
    };

    let mic_color = if env.muted {
        colors::RED_400
    } else {
        Color::WHITE
    };
    let on_mute = env.on_toggle_mute.clone();
    let mic = div()
        .class(cls::CALL_ATTACHED)
        .child(button(
            lottie_icon::toggle(
                "call-mic",
                lottie_icon::MIC,
                env.muted,
                mic_color,
                mic_color,
                20.0,
            ),
            if env.muted {
                requiem_i18n::t!("voice-unmute")
            } else {
                requiem_i18n::t!("voice-mute")
            }
            .to_string(),
            Some(cls::CALL_ATTACHED_BUTTON),
            Rc::new(move || on_mute()),
        ))
        .child(caret());
    let camera = div()
        .class(cls::CALL_ATTACHED)
        .child(button(
            lottie_icon::toggle(
                "call-camera",
                lottie_icon::CAMERA,
                camera_on,
                Color::WHITE,
                Color::WHITE,
                20.0,
            ),
            if camera_on {
                requiem_i18n::t!("voice-turn-off-camera")
            } else {
                requiem_i18n::t!("voice-turn-on-camera")
            }
            .to_string(),
            Some(cls::CALL_ATTACHED_BUTTON),
            Rc::new(move || voice_control::set_camera(!camera_on)),
        ))
        .child(caret());
    let first = div().class(cls::CALL_SECTION).children([mic, camera]);
    let second = div().class(cls::CALL_SECTION).children([
        button(
            lottie_icon::toggle(
                "call-screen",
                lottie_icon::SCREENSHARE,
                streaming,
                Color::WHITE,
                Color::WHITE,
                20.0,
            ),
            requiem_i18n::t!("voice-share-screen").to_string(),
            None,
            Rc::new(move || voice_control::toggle_stream(guild_id, channel_id)),
        ),
        button(
            still(APPS),
            requiem_i18n::t!("voice-start-activity").to_string(),
            None,
            Rc::new(|| {}),
        ),
        button(
            lottie_icon::hover(
                lottie_icon::SOUNDBOARD,
                Color::WHITE,
                Color::WHITE,
                20.0,
            ),
            requiem_i18n::t!("voice-open-soundboard").to_string(),
            None,
            Rc::new(|| {}),
        ),
        button(still(MORE_HORIZONTAL), String::new(), None, Rc::new(|| {})),
    ]);
    let hangup = tip(
        ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::CALL_HANGUP.to_string()])
            .on_click(voice_control::disconnect)
            .build()
            .child(
                lottie_icon::hover(
                    lottie_icon::END_CALL,
                    Color::WHITE,
                    Color::WHITE,
                    20.0,
                )
                .class(cls::CALL_GLYPH),
            ),
        requiem_i18n::t!("voice-disconnect").to_string(),
    );
    let center = div()
        .class(cls::CALL_CENTER)
        .children([first, second, hangup]);
    div()
        .class(cls::CALL_CONTROLS)
        .children([left, center, right])
}

struct VideoTile {
    user_id: Id<UserMarker>,
    image: RefCell<Option<(u64, skia_safe::Image)>>,
}

impl Widget for VideoTile {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        #[cfg(not(feature = "voice"))]
        let _ = self.user_id;
        #[cfg(feature = "voice")]
        if let Some((id, frame)) = voice_control::video_frame(self.user_id) {
            let mut cached = self.image.borrow_mut();
            if cached.as_ref().map(|(seen, _)| *seen) != Some(id) {
                let info = skia_safe::ImageInfo::new(
                    (frame.width as i32, frame.height as i32),
                    skia_safe::ColorType::RGBA8888,
                    skia_safe::AlphaType::Opaque,
                    None,
                );
                let data = skia_safe::Data::new_copy(&frame.rgba);
                *cached =
                    images::raster_from_data(&info, data, frame.width as usize * 4)
                        .map(|image| (id, image));
            }
        }
        let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
        if let Some((_, image)) = self.image.borrow().as_ref() {
            let scale =
                (rect.width / image.width() as f32).max(rect.height / image.height() as f32);
            let (w, h) = (image.width() as f32 * scale, image.height() as f32 * scale);
            let dst = Rect::from_xywh(
                rect.x + (rect.width - w) / 2.0,
                rect.y + (rect.height - h) / 2.0,
                w,
                h,
            );
            canvas.save();
            canvas.clip_rrect(RRect::new_rect_xy(bounds, 8.0, 8.0), None, true);
            canvas.draw_image_rect(image, None, dst, &Paint::default());
            canvas.restore();
        }
        requiem_ui_paint::animate_region(canvas, bounds);
    }
}

#[cfg(test)]
mod tests {
    use super::tile_size;

    #[test]
    fn tiles_fill_the_stage_at_16_by_9() {
        let (cols, w, h) = tile_size(1, (1600.0, 450.0));
        assert_eq!(cols, 1);
        assert!((w - 800.0).abs() < 0.01 && (h - 450.0).abs() < 0.01);

        assert_eq!(tile_size(2, (1400.0, 800.0)).0, 2);

        assert_eq!(tile_size(4, (1000.0, 600.0)).0, 2);
    }
}
