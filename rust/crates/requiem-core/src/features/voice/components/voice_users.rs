use crate::gateway::member_requests;
use crate::ui::context_menu;
use crate::ui::context_menus;
use crate::ui;
use crate::utils::time;
use crate::voice_control;
use requiem_ui::timer;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use requiem_ui::colors::{TEXT_CRITICAL, TEXT_POSITIVE, TEXT_STRONG, TEXT_SUBTLE};
use requiem_ui::components::{
    ButtonProps, ButtonVariant, TextProps, TextVariant, TooltipProps, div, icon,
};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use skia_safe::Color;

use crate::models::id::marker::{ChannelMarker, GuildMarker, UserMarker};
use crate::models::{Id, User};
use crate::state::{AppState, VoiceState};
use crate::ui::channel_sidebar::SidebarEnv;
use crate::ui::classes::voice_users as cls;
use requiem_ui::icons::{HEADPHONES_SLASH, MICROPHONE_SLASH, VIDEO};

const CHANNELS_DEFAULT: Color = TEXT_SUBTLE;

thread_local! {

    static SECOND: Signal<u64> = signal(0);
    static TICKING: Cell<bool> = const { Cell::new(false) };
}

fn second_tick() -> Signal<u64> {
    if !TICKING.with(Cell::get) {
        TICKING.with(|t| t.set(true));
        schedule_tick();
    }
    SECOND.with(Clone::clone)
}

fn schedule_tick() {
    timer::set_timeout(Duration::from_secs(1), || {
        SECOND.with(|s| s.update(|n| *n += 1));

        schedule_tick();
    });
}

pub fn format_elapsed(seconds: u64) -> String {
    let (h, m, s) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

pub fn channel_timer(state: &AppState, channel_id: Id<ChannelMarker>) -> Option<Node> {
    let started = state.voice.info(channel_id).voice_start_time?;
    if state.voice.in_channel(channel_id).is_empty() {
        return None;
    }
    Some(
        Node::reactive(move || {
            let _ = second_tick().get();
            let now = (time::now_ms() / 1000) as i64;
            let elapsed = (now - started).max(0) as u64;
            TextProps::styled(format_elapsed(elapsed), TextVariant::TEXT_XS_NORMAL)
                .color(TEXT_POSITIVE)
                .build()
        })
        .class(cls::VOICE_CHANNEL_TIMER),
    )
}

pub fn list(
    state: &AppState,
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
    env: &SidebarEnv,
) -> Node {
    let (state, env) = (state.clone(), env.clone());
    Node::reactive(move || {
        let members = state.voice.in_channel(channel_id);
        if members.is_empty() {
            return div();
        }
        let speaking = state.voice.speaking.get();

        let unknown = members
            .iter()
            .filter(|m| m.member.is_none() && state.users.get(m.user_id).is_none())
            .map(|m| m.user_id.get());
        member_requests::request(guild_id.get(), unknown, |user| {
            state.members.is_member(guild_id, Id::new(user))
        });
        let rows = sorted(&state, guild_id, &members)
            .into_iter()
            .map(|member| {
                row(
                    &state,
                    guild_id,
                    member,
                    speaking.contains(&member.user_id),
                    &env,
                )
            });
        div().class(cls::VOICE_USERS).children(rows)
    })
    .class(cls::VOICE_USERS_WRAP)
}

pub(crate) fn sorted<'a>(
    state: &AppState,
    guild_id: Id<GuildMarker>,
    members: &'a [VoiceState],
) -> Vec<&'a VoiceState> {
    let mut members: Vec<(&VoiceState, String)> = members
        .iter()
        .map(|m| (m, display_of(state, guild_id, m).0.to_lowercase()))
        .collect();
    members.sort_by(|a, b| a.1.cmp(&b.1));
    members.into_iter().map(|(m, _)| m).collect()
}

fn user_of(
    state: &AppState,
    guild_id: Id<GuildMarker>,
    voice: &VoiceState,
) -> (Option<User>, Option<String>) {
    if let Some(member) = &voice.member {
        return (Some(member.user.clone()), member.nick.clone());
    }
    let member = state.members.by_guild.with(|all| {
        all.get(&guild_id)
            .and_then(|m| m.get(&voice.user_id).cloned())
    });
    if let Some(member) = member {
        return (Some(member.user), member.nick);
    }
    (state.users.get(voice.user_id), None)
}

pub(crate) fn display_of(
    state: &AppState,
    guild_id: Id<GuildMarker>,
    voice: &VoiceState,
) -> (String, Option<User>) {
    let (user, nick) = user_of(state, guild_id, voice);
    let name = nick
        .or_else(|| user.as_ref().and_then(|u| u.global_name.clone()))
        .or_else(|| user.as_ref().map(|u| u.username.clone()))
        .unwrap_or_else(|| voice.user_id.to_string());
    (name, user)
}

fn row(
    state: &AppState,
    guild_id: Id<GuildMarker>,
    voice: &VoiceState,
    speaking: bool,
    env: &SidebarEnv,
) -> Node {
    let user_id = voice.user_id;
    let (name, user) = display_of(state, guild_id, voice);
    let discriminator = user
        .as_ref()
        .map(|u| u.discriminator.clone())
        .unwrap_or_else(|| "0".into());

    let mut avatar = ui::user_avatar_props(user_id, &discriminator, &env.fonts);
    if let Some(hash) = user.as_ref().and_then(|u| u.avatar.clone()) {
        let (images, url) = (env.images.clone(), env.backend.avatar_url(user_id, &hash));
        if let Some(image) = images.get_or_decode_image(&url) {
            avatar = avatar.image(image);
        }
        avatar = avatar.bind_image(env.tick.clone(), move || images.get_or_decode_image(&url));
    }
    let mut avatar = avatar.build().class(cls::VOICE_USER_AVATAR);
    if speaking {
        avatar = avatar.class(cls::VOICE_USER_AVATAR_SPEAKING);
    }

    let name_node = TextProps::styled(name.clone(), TextVariant::TEXT_SM_MEDIUM)
        .color(if speaking {
            TEXT_STRONG
        } else {
            CHANNELS_DEFAULT
        })
        .hover_color(TEXT_STRONG)
        .line_clamp(1_u32)
        .build()
        .class(cls::VOICE_USER_NAME);

    let content = div()
        .class(cls::VOICE_USER_CONTENT)
        .child(avatar)
        .child(name_node)
        .child(state_icons(voice));
    let target = Rc::new(context_menus::UserTarget {
        user_id,
        guild_id: Some(guild_id),
        display_name: name,
        webhook: false,
    });
    let menu_env = context_menus::MenuEnv {
        backend: env.backend.clone(),
        queries: env.queries.clone(),
        http: Some(env.http.clone()),
        images: env.images.clone(),
        on_save: None,
        own_user_id: state
            .users
            .current
            .with_untracked(|u| u.as_ref().map(|u| u.id)),
        state: state.clone(),
    };
    ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::VOICE_USER.to_string()])
        .build()
        .child(content)
        .on_context_menu(move |x, y| {
            let (env, target) = (menu_env.clone(), target.clone());
            context_menu::open(x, y, move || {
                context_menus::voice_user_items(&env, &target)
            });
        })
}

fn state_icons(voice: &VoiceState) -> Node {
    let mut icons = div().class(cls::VOICE_USER_ICONS);
    if voice.self_stream {
        icons = icons.child(
            div().class(cls::VOICE_USER_LIVE).child(
                TextProps::styled(
                    requiem_i18n::t!("voice-live").to_uppercase(),
                    TextVariant::TEXT_XS_BOLD,
                )
                .color(Color::WHITE)
                .build(),
            ),
        );
    }
    if voice.self_video {
        icons = icons.child(icon(VIDEO, CHANNELS_DEFAULT).class(cls::VOICE_USER_ICON));
    }
    let tip = |node: Node, label: String| TooltipProps::default().text(label).build(node);

    let local = voice_control::is_locally_muted(voice.user_id);
    let server_mute = voice.mute || voice.suppress;
    if server_mute || local || voice.self_mute || voice.self_deaf {
        let (glyph, color) = if server_mute || local {
            (
                MICROPHONE_SLASH,
                if server_mute {
                    TEXT_CRITICAL
                } else {
                    CHANNELS_DEFAULT
                },
            )
        } else {
            (MICROPHONE_SLASH, CHANNELS_DEFAULT)
        };
        let label = if local {
            requiem_i18n::t!("voice-muted-by-you")
        } else if server_mute {
            requiem_i18n::t!("voice-server-muted")
        } else {
            requiem_i18n::t!("voice-muted")
        };
        icons = icons.child(tip(
            icon(glyph, color).class(cls::VOICE_USER_ICON),
            label.to_string(),
        ));
    }
    if voice.deaf || voice.self_deaf {
        let (glyph, color, label) = if voice.deaf {
            (
                HEADPHONES_SLASH,
                TEXT_CRITICAL,
                requiem_i18n::t!("voice-server-deafened"),
            )
        } else {
            (
                HEADPHONES_SLASH,
                CHANNELS_DEFAULT,
                requiem_i18n::t!("voice-deafened"),
            )
        };
        icons = icons.child(tip(
            icon(glyph, color).class(cls::VOICE_USER_ICON),
            label.to_string(),
        ));
    }
    icons
}

pub fn in_my_call(state: &AppState, user_id: Id<UserMarker>) -> bool {
    let Some(channel) = state.voice.rtc.with_untracked(|r| r.channel_id) else {
        return false;
    };
    state.voice.by_channel.with_untracked(|all| {
        all.get(&channel)
            .is_some_and(|states| states.iter().any(|s| s.user_id == user_id))
    })
}

#[cfg(test)]
mod tests {
    use super::format_elapsed;

    #[test]
    fn the_clock_reads_like_the_live_one() {
        assert_eq!(format_elapsed(0), "0:00");
        assert_eq!(format_elapsed(1275), "21:15");
        assert_eq!(format_elapsed(3600 + 62), "1:01:02");
    }
}
