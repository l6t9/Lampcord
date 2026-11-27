use crate::ui::classes::typing_bar as cls;
use crate::ui;
use requiem_ui::colors;
use requiem_ui::components::{TextProps, TextVariant, div};
use requiem_ui::node::Node;
use requiem_ui::paint as requiem_ui_paint;
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::{AbsoluteRect, Widget};
use skia_safe::font_style::Weight;
use skia_safe::gradient::shaders::linear_gradient;
use skia_safe::gradient::{Colors, Gradient, Interpolation};
use skia_safe::{Canvas, Color, Color4f, Paint, Point, TileMode};
use std::f32::consts;
use std::time::SystemTime;
use std::time;

use crate::models::Id;
use crate::models::id::marker::{ChannelMarker, GuildMarker, UserMarker};
use crate::rest::endpoints::relationships::BLOCKED;
use crate::state::AppState;
use crate::ui::{TEXT_SECONDARY, TEXT_STRONG};

/// Discord names up to three people, then says "Several people".
const NAMED: usize = 3;

/// Three dots that pulse one after another, each between 2.8px and 3.5px across.
struct TypingDotsWidget;

impl Widget for TypingDotsWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        const PERIOD: f32 = 1.2;
        const STAGGER: f32 = 0.2;
        let seconds = SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .map_or(0.0, |elapsed| elapsed.as_secs_f32());
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(TEXT_STRONG);
        for dot in 0..3 {
            let phase = (seconds / PERIOD - dot as f32 * STAGGER) * consts::TAU;
            let radius = 2.8 + 0.7 * (0.5 + 0.5 * phase.sin());
            let cx = rect.x + 3.5 + dot as f32 * 8.75;
            canvas.draw_circle((cx, rect.y + rect.height / 2.0), radius, &paint);
        }
        // The dots never settle; ask for the next frame.
        requiem_ui_paint::keep_animating();
    }
}

/// Height of the composer's top fade (0.5rem).
const FADE_HEIGHT: f32 = 8.0;

/// Transparent to a color, top to bottom.
struct FadeWidget {
    token: &'static str,
    fallback: Color,
}

impl Widget for FadeWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        let base = colors::token(self.token, self.fallback);
        // The same color at zero alpha, not transparent black: blending from black
        // would pass through a grey band.
        let colors = [
            Color4f::from(Color::from_argb(0, base.r(), base.g(), base.b())),
            Color4f::from(base),
        ];
        let gradient = Gradient::new(
            Colors::new(&colors, None, TileMode::Clamp, None),
            Interpolation::default(),
        );
        let Some(shader) = linear_gradient(
            (
                Point::new(rect.x, rect.y),
                Point::new(rect.x, rect.y + rect.height),
            ),
            &gradient,
            None,
        ) else {
            return;
        };
        let mut paint = Paint::default();
        paint.set_shader(shader);
        canvas.draw_rect(
            skia_safe::Rect::from_xywh(rect.x, rect.y, rect.width, rect.height),
            &paint,
        );
    }
}

/// Discord's `form::before`: an 8px strip over the top of the composer, from
/// transparent to the chat background, painted behind the composer box. Messages
/// scroll beneath it, so they dissolve instead of ending on a hard edge.
pub fn fade() -> Node {
    let widget = FadeWidget {
        token: "background-base-low",
        fallback: Color::from_rgb(0x20, 0x20, 0x24),
    };
    Node::new("div", widget).style(format!(
        "position: absolute; left: 0px; right: 0px; top: 0px; height: {FADE_HEIGHT}px; pointer-events: none"
    ))
}

/// Width of the message list's scrollbar (`scrollbar-width: auto`), which the backdrop leaves clear.
const LIST_SCROLLBAR_PX: f32 = 16.0;

/// How far the backdrop reaches above the strip, so the text sits on mostly solid color.
const BACKDROP_RISE: f32 = 16.0;

/// What the typing text sits on: messages under the strip fade into the composer's
/// background, so the text stays readable and joins the composer without an edge.
fn backdrop() -> Node {
    let widget = FadeWidget {
        token: "background-base-lower",
        fallback: Color::from_rgb(0x1a, 0x1a, 0x1e),
    };
    Node::new("div", widget).style(format!(
        "position: absolute; left: 0px; right: {LIST_SCROLLBAR_PX}px; top: -{BACKDROP_RISE}px; bottom: 0px; pointer-events: none"
    ))
}

/// The people to name: not blocked, with a name we can show.
fn typers(state: &AppState, channel_id: Id<ChannelMarker>, guild: Option<Id<GuildMarker>>) -> Vec<String> {
    state
        .typing
        .typing_in(channel_id)
        .into_iter()
        .filter(|id| state.relationships.kind_of(*id) != Some(BLOCKED))
        .filter_map(|id| display_name(state, guild, id))
        .collect()
}

fn display_name(state: &AppState, guild: Option<Id<GuildMarker>>, user_id: Id<UserMarker>) -> Option<String> {
    if let Some(guild) = guild {
        let named = state.members.by_guild.with_untracked(|all| {
            all.get(&guild)?
                .get(&user_id)
                .map(|member| member.user.display_name(member.nick.as_deref()))
        });
        if named.is_some() {
            return named;
        }
    }
    state
        .users
        .get(user_id)
        .map(|user| state.users.display_name(&user))
}

/// The sentence as pieces: `(text, is_name)`.
fn sentence(names: &[String]) -> Vec<(String, bool)> {
    let name = |index: usize| (names[index].clone(), true);
    let plain = |text: String| (text, false);
    match names.len() {
        0 => Vec::new(),
        1 => vec![name(0), plain(requiem_i18n::t!("typing-bar-one"))],
        2 => vec![
            name(0),
            plain(requiem_i18n::t!("typing-bar-and")),
            name(1),
            plain(requiem_i18n::t!("typing-bar-many")),
        ],
        n if n <= NAMED => vec![
            name(0),
            plain(requiem_i18n::t!("typing-bar-comma")),
            name(1),
            plain(requiem_i18n::t!("typing-bar-comma-and")),
            name(2),
            plain(requiem_i18n::t!("typing-bar-many")),
        ],
        _ => vec![plain(requiem_i18n::t!("typing-bar-several"))],
    }
}

/// The strip above the composer. It keeps its height when nobody is typing so the composer
/// doesn't move.
pub fn build(state: &AppState, channel_id: Id<ChannelMarker>) -> Node {
    let guild = ui::find_channel(state, channel_id).map(|(guild, _)| guild);
    let state = state.clone();
    Node::reactive(move || {
        let names = typers(&state, channel_id, guild);
        if names.is_empty() {
            return div().class(cls::TYPING_BAR);
        }
        let pieces = sentence(&names).into_iter().map(|(text, is_name)| {
            let (weight, color) = if is_name {
                (Weight::SEMI_BOLD, TEXT_STRONG)
            } else {
                (Weight::MEDIUM, TEXT_SECONDARY)
            };
            let mut props = TextProps::styled(text, TextVariant::TEXT_XS_MEDIUM).color(color);
            props.weight = weight.into();
            props.build()
        });
        div().class(cls::TYPING_BAR).children([
            backdrop(),
            Node::new("div", TypingDotsWidget).class(cls::TYPING_DOTS),
            div().class(cls::TYPING_TEXT).children(pieces),
        ])
    })
    .style("width: 100%; flex-shrink: 0; margin-top: -24px")
}
