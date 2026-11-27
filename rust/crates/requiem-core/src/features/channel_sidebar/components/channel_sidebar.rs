use crate::models::id;
use crate::state::RtcState;
use crate::ui::classes::channel_sidebar;
use crate::ui::classes::channel_sidebar as cls;
use crate::ui::context_menu;
use crate::ui::context_menus;
use crate::ui::create_channel;
use crate::ui::invite_modal::location;
use crate::ui::invite_modal;
use crate::ui::sidebar_resize;
use crate::ui::voice_users;
use crate::ui;
use crate::utils::time;
use crate::voice_control;
use requiem_ui::colors;
use requiem_ui::components::ButtonVariant;
use requiem_ui::components::TooltipProps;
use requiem_ui::components::line_clamp_class;
use requiem_ui::components::{HeadingLevel, TextVariant};
use requiem_ui::icon_handle::Icon;
use requiem_ui::icon_handle::IconSpec;
use requiem_ui::widget::primitives::corner_radii;
use requiem_ui::widget::primitives::has_any_corner_radius;
use requiem_ui::widget::primitives;
use skia_safe::RRect;
use skia_safe::Rect;
use skia_safe::image_filters;
use std::cell::Cell;
use std::cell;
use std::cmp;
use std::collections::HashMap;
use std::collections;
use std::env;
use std::fs;
use std::rc::Rc;
use std::time::Duration;

use requiem_ui::colors::SPINE_DEFAULT;
use requiem_ui::components::{
    ButtonProps, HeadingProps, NumberBadgeProps, Padding, StackProps, TextProps, div, icon,
    icon_with_hover, number_badge, ScrollViewProps,
};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::style::ComputedStyle;
use requiem_ui::timer::{TimeoutHandle, clear_timeout, set_timeout};
use requiem_ui::widget::primitives::ImageWidget;
use requiem_ui::widget::{AbsoluteRect, Widget, WidgetId, WidgetTree};
use skia_safe::{Canvas, Color, Paint, Path, PathFillType};

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::models::id::marker::{ChannelMarker, GuildMarker};
use crate::models::{Channel, Guild, Id, Permissions};
use crate::query::QueryClient;
use crate::rest::HttpClient;
use crate::state::AppState;
use crate::ui::{TEXT_MUTED, TEXT_SECONDARY, TEXT_STRONG, TEXT_SUBTLE};
use requiem_ui::icons::{
    CHEVRON_SMALL_DOWN, CHEVRON_SMALL_UP, GROUP_PLUS, PLUS_SMALL, THREAD_SPINE,
};

#[derive(Clone)]
pub struct SidebarEnv {
    pub images: ImageCache,
    pub backend: BackendConfig,
    pub queries: QueryClient,
    pub http: HttpClient,

    pub tick: Signal<u64>,

    pub fonts: ui::Fonts,
}

const VERIFIED_GREEN: Color = Color::from_rgb(0x3d, 0x9e, 0x60);

struct FlowerBadge {
    color: Color,
}

impl Widget for FlowerBadge {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        let Some(mut flower) =
            Path::from_svg(Icon::Named("shape/flower_star").path())
        else {
            return;
        };
        flower.set_fill_type(PathFillType::EvenOdd);
        let scale = rect.width / 16.0;
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(self.color);
        canvas.save();
        canvas.translate((rect.x, rect.y));
        canvas.scale((scale, scale));
        canvas.draw_path(&flower, &paint);
        canvas.restore();
    }
}

pub fn verified_badge() -> Node {
    let check = Path::from_svg(Icon::Named("shape/badge_check").path())
        .map(|_| {
            icon(
                IconSpec::Path(
                    Icon::Named("shape/badge_check")
                        .path()
                        .to_string(),
                ),
                Color::WHITE,
            )
            .class(cls::BADGE_CHECK)
        });
    let mut badge = div().class(cls::BADGE).child(
        Node::new(
            "span",
            FlowerBadge {
                color: VERIFIED_GREEN,
            },
        )
        .class(cls::BADGE_FLOWER),
    );
    if let Some(check) = check {
        badge = badge.child(check);
    }
    badge
}

struct PartnerMark;

impl Widget for PartnerMark {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        let scale = rect.width / 16.0;
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(Color::WHITE);
        canvas.save();
        canvas.translate((rect.x, rect.y));
        canvas.scale((scale, scale));
        for name in ["shape/partner_mark_a", "shape/partner_mark_b"] {
            if let Some(path) = Path::from_svg(Icon::Named(name).path()) {
                canvas.draw_path(&path, &paint);
            }
        }
        canvas.restore();
    }
}

pub fn partnered_badge() -> Node {
    div()
        .class(cls::BADGE)
        .child(
            Node::new(
                "span",
                FlowerBadge {
                    color: Color::from_rgb(0x58, 0x65, 0xf2),
                },
            )
            .class(cls::BADGE_FLOWER),
        )
        .child(Node::new("span", PartnerMark).class(cls::BADGE_CHECK))
}

#[allow(clippy::too_many_arguments)]
pub fn build(
    tree: &mut WidgetTree,
    parent: WidgetId,
    state: &AppState,
    selected_guild: Option<Id<GuildMarker>>,
    selected_channel: Option<Id<ChannelMarker>>,
    on_select: &Rc<dyn Fn(Id<ChannelMarker>)>,
    env: &SidebarEnv,
) {
    let sidebar = sidebar_resize::container();
    let sidebar_id = tree.mount(parent, sidebar);

    let guilds = state.guilds.list.get();
    if let Some(guild_id) = selected_guild {
        state.channels.by_guild.watch(&guild_id);
    }
    let channels_by_guild: HashMap<_, _> = selected_guild
        .and_then(|id| {
            state
                .channels
                .by_guild
                .with_untracked(|all| all.get(&id).cloned().map(|channels| (id, channels)))
        })
        .into_iter()
        .collect();

    let guild = selected_guild.and_then(|id| guilds.iter().find(|g| g.id == id));

    // A list with an `on_scroll` handler isn't given its offset back after a rebuild, so it is
    // seeded from the saved one; another server's list starts at the top.
    if LIST_GUILD.with(|last| last.replace(guild.map(|g| g.id))) != guild.map(|g| g.id) {
        BANNER_SCROLL.with(|offset| offset.set(0.0));
        BANNER_VISIBLE.with(|visible| visible.set(true));
    }

    tree.mount(sidebar_id, guild_header(state, guild, env));

    if let Some(guild) = guild {
        thread_local! { static ASKED: cell::RefCell<collections::HashSet<Id<GuildMarker>>> = Default::default(); }
        if ASKED.with(|asked| asked.borrow_mut().insert(guild.id)) {
            voice_control::request_channel_info(guild.id);
        }
    }

    let has_banner =
        guild.is_some_and(|guild| guild.banner.is_some()) || spoofed_banner().is_some();
    let mut list = ScrollViewProps::default()
        .initial_offset_y(BANNER_SCROLL.with(Signal::peek))
        .on_scroll(|offset: f32, _max: f32| {
            BANNER_SCROLL.with(|s| s.set(offset));
            let visible = offset < BANNER_COLLAPSE_PX;
            BANNER_VISIBLE.with(|v| {
                if v.peek() != visible {
                    v.set(visible);
                }
            });
        })
        .build()
        .class(channel_sidebar::CHANNEL_LIST);

    if let Some(guild_id) = guild.map(|g| g.id) {
        let state = state.clone();
        list = list.on_context_menu(move |x, y| {
            let state = state.clone();
            context_menu::open(x, y, move || {
                context_menus::channel_list_items(&state, guild_id)
            });
        });
    }
    let scroller_id = tree.mount(sidebar_id, list);
    if has_banner {
        // Transparent, so the banner shows through; the rows below it have a background and
        // slide over the banner as the list scrolls.
        tree.mount(
            scroller_id,
            div().class(channel_sidebar::CHANNEL_LIST_BANNER_GAP),
        );
    }
    let list_id = tree.mount(
        scroller_id,
        div().class(channel_sidebar::CHANNEL_LIST_CONTENT),
    );

    let _ = THREAD_RELEVANCE.with(Signal::get);
    let mut relevance_deadlines = Vec::new();
    if let Some(guild) = guild {
        if let Some(channels) = channels_by_guild.get(&guild.id) {
            let viewable = |c: &&Channel| state.can(c, Permissions::VIEW_CHANNEL);
            let can_manage = state
                .permissions
                .guild_permissions(guild)
                .contains(Permissions::MANAGE_CHANNELS);
            let mut uncategorized: Vec<&Channel> = channels
                .iter()
                .filter(|c| c.kind != 4 && !c.is_thread() && c.parent_id.is_none())
                .filter(viewable)
                .collect();
            uncategorized.sort_by_key(|c| c.position.unwrap_or(0));
            for channel in uncategorized {
                add_channel_row(
                    tree,
                    list_id,
                    state,
                    guild.id,
                    channel,
                    selected_channel,
                    on_select,
                    env,
                );
                add_thread_rows(
                    tree,
                    list_id,
                    state,
                    guild.id,
                    channel,
                    channels,
                    selected_channel,
                    on_select,
                    &mut relevance_deadlines,
                );
            }

            let mut categories: Vec<&Channel> = channels
                .iter()
                .filter(|c| c.kind == 4)
                .filter(viewable)
                .collect();
            categories.sort_by_key(|c| c.position.unwrap_or(0));

            let mut by_parent: HashMap<Id<ChannelMarker>, Vec<&Channel>> =
                HashMap::new();
            for channel in channels.iter().filter(viewable).filter(|c| !c.is_thread()) {
                if let Some(parent) = channel.parent_id {
                    by_parent.entry(parent).or_default().push(channel);
                }
            }

            for category in categories {
                let mut children: Vec<&Channel> =
                    by_parent.remove(&category.id).unwrap_or_default();

                if children.is_empty() && !can_manage {
                    continue;
                }
                let label = category.name.clone().unwrap_or_default().to_uppercase();

                let name_node = HeadingProps::default()
                    .level(HeadingLevel::H5)
                    .variant(TextVariant::TEXT_XS_BOLD)
                    .content(label)
                    .color(TEXT_SUBTLE)
                    .build()
                    .class(line_clamp_class(1))
                    .class(channel_sidebar::CATEGORY_NAME);
                let mut label_node = div()
                    .class(channel_sidebar::CATEGORY_LABEL)
                    .child(name_node);

                if can_manage {
                    let (guild_id, category_id) = (guild.id, category.id);
                    let add = ButtonProps::default()
                        .variant(ButtonVariant::Bare)
                        .extra_classes(vec![
                            channel_sidebar::CATEGORY_ADD.to_string(),
                        ])
                        .on_click(move || {
                            create_channel::open(guild_id, Some(category_id))
                        })
                        .build()
                        .child(
                            icon_with_hover(PLUS_SMALL, TEXT_SUBTLE, TEXT_STRONG)
                                .class(channel_sidebar::CATEGORY_ADD_ICON),
                        );
                    label_node = label_node.child(add);
                }
                let (menu_state, guild_id, category_id) = (state.clone(), guild.id, category.id);
                label_node = label_node.on_context_menu(move |x, y| {
                    let state = menu_state.clone();
                    context_menu::open(x, y, move || {
                        context_menus::category_items(&state, guild_id, category_id)
                    });
                });
                tree.mount(list_id, label_node);

                children.sort_by_key(|c| c.position.unwrap_or(0));
                for channel in children {
                    add_channel_row(
                        tree,
                        list_id,
                        state,
                        guild.id,
                        channel,
                        selected_channel,
                        on_select,
                        env,
                    );
                    add_thread_rows(
                        tree,
                        list_id,
                        state,
                        guild.id,
                        channel,
                        channels,
                        selected_channel,
                        on_select,
                        &mut relevance_deadlines,
                    );
                }
            }
        }
    }

    schedule_relevance_check(&relevance_deadlines);
    tree.mount(sidebar_id, sidebar_resize::handle());
}

fn spoofed_banner() -> Option<skia_safe::Image> {
    thread_local! {
        static BANNER: Option<skia_safe::Image> = env::var_os("REQUIEM_SPOOF_BANNER").and_then(|path| fs::read(path).ok()).and_then(|bytes| ui::decode_image(&bytes));
    }
    BANNER.with(Clone::clone)
}

fn guild_header(state: &AppState, guild: Option<&Guild>, env: &SidebarEnv) -> Node {
    let Some(guild) = guild else {
        return div().class(cls::GUILD_HEADER_WRAP).child(
            div().class(cls::GUILD_HEADER).child(
                TextProps::styled(
                    requiem_i18n::t!("channel-sidebar-no-server-selected").to_string(),
                    TextVariant::TEXT_MD_SEMIBOLD,
                )
                .color(TEXT_STRONG)
                .build(),
            ),
        );
    };

    let banner = guild.banner.as_deref().and_then(|hash| {
        let image = env
            .images
            .get_or_decode_image(&env.backend.guild_banner_url(guild.id, hash));
        if image.is_none() {
            let _ = env.tick.get();
        }
        image
    });
    let banner = banner.or_else(spoofed_banner);
    let has_banner = banner.is_some();

    let mut name_row = div().class(cls::GUILD_BADGE_AND_NAME);

    let badge = if guild.is_verified() {
        Some((
            verified_badge(),
            if guild.is_partnered() {
                "Verified & Partnered"
            } else {
                "Verified"
            },
        ))
    } else if guild.is_partnered() {
        Some((partnered_badge(), "Partnered"))
    } else {
        None
    };
    if let Some((badge, label)) = badge {
        name_row = name_row.child(
            div().class(cls::GUILD_BADGE_BOX).child(
                TooltipProps::default()
                    .text(label)
                    .build(badge),
            ),
        );
    }
    name_row = name_row.child(
        TextProps::styled(guild.name.clone(), TextVariant::TEXT_MD_SEMIBOLD)
            .color(TEXT_STRONG)
            .line_clamp(1_u32)
            .build()
            .class(cls::GUILD_NAME),
    );

    let chevron = Node::reactive(|| {
        let open = HEADER_MENU
            .with(Cell::get)
            .is_some_and(|at| context_menu::origin_tracked() == Some(at));
        icon(
            if open {
                CHEVRON_SMALL_UP
            } else {
                CHEVRON_SMALL_DOWN
            },
            TEXT_STRONG,
        )
        .class(cls::GUILD_CHEVRON)
    })
    .class(cls::GUILD_CHEVRON_BOX);

    let (menu_state, env_for_menu, guild_id) = (state.clone(), env.clone(), guild.id);
    let dropdown = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::GUILD_DROPDOWN.to_string()])
        .on_click_at(move |rect: AbsoluteRect| {
            let (state, env) = (menu_state.clone(), env_for_menu.clone());
            let at = (
                rect.x - 8.0 + (rect.width + 16.0 + 32.0 + 4.0 - 220.0) / 2.0,
                rect.y + rect.height + 8.0 + 4.0,
            );
            HEADER_MENU.with(|m| m.set(Some(at)));
            context_menu::open(at.0, at.1, move || {
                context_menus::guild_header_items(
                    &state,
                    &env.queries,
                    &env.http,
                    guild_id,
                )
            });
        })
        .build()
        .children([name_row, chevron]);

    let mut row = div().class(cls::GUILD_HEADER_CONTENT).child(dropdown);
    // Always there, as in the web client: what it opens depends on whether you can invite, the
    // server has a vanity link, or neither.
    {
        let invite_guild = guild.id;
        row = row.child(
            ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![cls::GUILD_INVITE.to_string()])
                .on_click(move || {
                    invite_modal::open(
                        invite_guild,
                        location::HEADER_BUTTON,
                    )
                })
                .build()
                .child(icon(GROUP_PLUS, TEXT_STRONG).class(cls::GUILD_INVITE_ICON)),
        );
    }
    let mut header = Node::new(
        "div",
        HoverProbeWidget(Box::new(|inside| {
            HEADER_HOVER.with(|hover| {
                if hover.peek() != inside {
                    hover.set(inside);
                }
            })
        })),
    )
    .class(cls::GUILD_HEADER);
    if has_banner {
        // The banner's look (no rule, a soft shadow) only holds while the banner is showing.
        header = header.bind_class(
            BANNER_VISIBLE.with(Clone::clone),
            cls::GUILD_HEADER_ON_BANNER,
            "",
        );
    }
    header = header.child(row);

    let mut wrap = div().class(cls::GUILD_HEADER_WRAP);
    if let Some(banner) = banner {
        // The banner stays where it is. The channel list, whose content has a background,
        // slides over it as it scrolls, and the header layers fade in as it does.
        let anchor = Node::reactive(move || {
            let progress = (BANNER_SCROLL.with(Signal::get) / BANNER_COLLAPSE_PX).clamp(0.0, 1.0);
            let hovered = HEADER_HOVER.with(Signal::get);
            // Past the collapse the banner and the header layers are gone altogether.
            if progress >= 1.0 {
                return div();
            }
            let value = 1.0 - progress;
            let layer = |layer| {
                Node::new("div", HeaderLayerWidget { layer, value }).class(cls::GUILD_HEADER_LAYER)
            };
            div()
                .child(
                    div()
                        .class(cls::GUILD_BANNER)
                        .child(
                            Node::new(
                                "div",
                                ImageWidget::new(Some(banner.clone())).with_cover_centered(),
                            )
                            .class(cls::GUILD_BANNER_PICTURE),
                        )
                        .style(format!("opacity: {value}")),
                )
                .child(layer(HeaderLayer::Backdrop))
                // Black at 52%, an ellipse twice the header's height that reaches a quarter of the
                // sidebar past each side, blurred 10px and multiplied onto the banner.
                .child(
                    div()
                        .class(cls::GUILD_HEADER_ELLIPSE_CLIP)
                        .child(
                            div()
                                .class(cls::GUILD_HEADER_ELLIPSE_BLUR)
                                .child(div().class(cls::GUILD_HEADER_ELLIPSE)),
                        )
                        .style(format!("opacity: {value}")),
                )
                .child(layer(HeaderLayer::Glass { hovered }))
        })
        .class(cls::GUILD_BANNER_ANCHOR);
        wrap = wrap.child(anchor);
    }
    wrap.child(header)
}

thread_local! {
    static LIST_GUILD: Cell<Option<Id<GuildMarker>>> = const { Cell::new(None) };
    /// Whether the pointer is over the header; while the banner shows, that brings the glass in.
    static HEADER_HOVER: Signal<bool> = signal(false);
    /// How far the channel list is scrolled; drives the banner's collapse.
    static BANNER_SCROLL: Signal<f32> = signal(0.0);
    /// True until the banner has scrolled away.
    static BANNER_VISIBLE: Signal<bool> = signal(true);
}

/// The header's layers above the banner, as Discord stacks them: a 2px blur behind the header,
/// a blurred black ellipse that darkens the top, and the 30px "glass" that frosts the header
/// once the list scrolls under it (or while it is hovered). `value` is 1 at the top of the
/// list and 0 once it has scrolled 88px.
#[derive(Clone, Copy)]
enum HeaderLayer {
    Backdrop,
    Glass { hovered: bool },
}

struct HeaderLayerWidget {
    layer: HeaderLayer,
    value: f32,
}

/// Reports whether the pointer is over the header or anything in it.
struct HoverProbeWidget(Box<dyn FnMut(bool)>);

impl Widget for HoverProbeWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        primitives::BoxWidget.paint(canvas, rect, style);
    }

    fn hover_path_changed(&mut self, inside: bool) {
        (self.0)(inside);
    }
}

const HEADER_HEIGHT: f32 = 49.0;
/// How far the list scrolls before the banner is gone.
const BANNER_COLLAPSE_PX: f32 = 88.0;

fn backdrop_blur(canvas: &Canvas, bounds: Rect, sigma: f32, alpha: f32) {
    if alpha <= 0.0 {
        return;
    }
    let Some(blur) =
        image_filters::blur((sigma, sigma), skia_safe::TileMode::Clamp, None, None)
    else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_alpha_f(alpha.min(1.0));
    let layer = skia_safe::canvas::SaveLayerRec::default()
        .bounds(&bounds)
        .backdrop(&blur)
        .paint(&paint);
    canvas.save_layer(&layer);
    canvas.restore();
}

impl Widget for HeaderLayerWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let value = self.value.clamp(0.0, 1.0);
        let area = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
        canvas.save();
        // Round-clipped like the banner, so the sidebar's rounded corner holds.
        if has_any_corner_radius(style) {
            canvas.clip_rrect(RRect::new_rect_radii(area, &corner_radii(style)), None, true);
        } else {
            canvas.clip_rect(area, None, true);
        }
        match self.layer {
            HeaderLayer::Backdrop => backdrop_blur(
                canvas,
                Rect::from_xywh(rect.x + 1.0, rect.y, rect.width - 2.0, HEADER_HEIGHT),
                2.0,
                0.5 * value,
            ),
            HeaderLayer::Glass { hovered } => {
                // A 30px blur that comes in over the first 15px scrolled, a little darker the
                // further the banner has gone, with a hairline under it.
                let scrolled = ((1.0 - value) * 6.0).clamp(0.0, 1.0);
                let glass = if hovered { 1.0 } else { scrolled };
                let glass_area =
                    Rect::from_xywh(rect.x + 1.0, rect.y, rect.width - 1.0, HEADER_HEIGHT + 1.0);
                backdrop_blur(canvas, glass_area, 30.0, glass);
                let dim = 1.0 - (0.75 + 0.25 * value);
                if dim > 0.0 && glass > 0.0 {
                    let mut shade = Paint::default();
                    shade.set_color(Color::from_argb((dim * glass * 255.0) as u8, 0, 0, 0));
                    canvas.draw_rect(glass_area, &shade);
                }
                if glass > 0.0 {
                    let mut line = Paint::default();
                    line.set_color(Color::from_argb((glass * 0.12 * 255.0) as u8, 171, 172, 178));
                    canvas.draw_rect(
                        Rect::from_xywh(
                            glass_area.left,
                            glass_area.bottom - 1.0,
                            glass_area.width(),
                            1.0,
                        ),
                        &line,
                    );
                }
            }
        }
        canvas.restore();
    }
}

thread_local! {

    static HEADER_MENU: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
}

thread_local! {
    static THREAD_RELEVANCE: Signal<u64> = signal(0);
    static THREAD_RELEVANCE_TIMER: Cell<Option<TimeoutHandle>> = const { Cell::new(None) };
}

const CHANNEL_FLAG_PINNED: u64 = 1 << 1;

fn rfc3339_millis(value: Option<&str>) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(value?)
        .ok()
        .map(|time| time.timestamp_millis())
}

fn thread_muted(thread: &Channel) -> bool {
    thread.member.as_ref().is_some_and(|member| member.muted)
}

// Discord's thread relevance deadline: the latest activity plus the
// auto-archive duration.
fn relevance_deadline(state: &AppState, thread: &Channel) -> Option<i64> {
    let metadata = thread.thread_metadata.as_ref()?;
    let last_message = state
        .read_states
        .last_message_id(thread.id)
        .or(thread.last_message_id.map(|id| id.get()))
        .unwrap_or(thread.id.get());
    let last_activity = ((last_message >> 22) as i64 + id::DISCORD_EPOCH_MS)
        .max(rfc3339_millis(metadata.archive_timestamp.as_deref()).unwrap_or(0));
    Some(last_activity + i64::from(metadata.auto_archive_duration) * 60_000)
}

struct ThreadStatus {
    unread: bool,
    relevant: bool,
    relevant_until: Option<i64>,
}

fn thread_status(state: &AppState, thread: &Channel, now: i64) -> ThreadStatus {
    let badge = state.read_states.badge(thread.id).get();
    let mentioned = badge.mentions > 0;
    let unread = badge.unread && !thread_muted(thread);
    let pinned = thread.flags & CHANNEL_FLAG_PINNED != 0;
    let active = !thread.is_archived_thread();
    let relevant_until = active
        .then(|| relevance_deadline(state, thread))
        .flatten()
        .filter(|deadline| *deadline > now);
    let in_voice = state
        .voice
        .rtc
        .with_untracked(|rtc| rtc.channel_id == Some(thread.id) && rtc.state.is_connected());
    ThreadStatus {
        unread: (active || pinned) && unread || mentioned,
        relevant: relevant_until.is_some() || pinned || unread || mentioned || in_voice,
        relevant_until,
    }
}

// Which threads Discord lists under a channel, newest join first: the joined
// active threads that are relevant (or only the unread ones while the channel
// is muted), all of them while the channel is selected, and the selected
// thread even when it isn't joined.
fn sidebar_threads<'a>(
    state: &AppState,
    guild_id: Id<GuildMarker>,
    parent: &Channel,
    channels: &'a [Channel],
    selected_channel: Option<Id<ChannelMarker>>,
    relevance_deadlines: &mut Vec<i64>,
) -> Vec<&'a Channel> {
    let selected = selected_channel.and_then(|id| channels.iter().find(|c| c.id == id));
    let voice_channel = state
        .voice
        .rtc
        .with_untracked(|rtc| rtc.channel_id.filter(|_| rtc.state.is_connected()));
    let parent_selected = selected.is_some()
        && (selected_channel == Some(parent.id) || voice_channel == Some(parent.id));
    let selected_thread =
        selected.filter(|channel| channel.is_thread() && channel.parent_id == Some(parent.id));
    let use_relevant = parent_selected
        || selected_thread.is_some()
        || !state
            .guild_settings
            .is_channel_muted(Some(guild_id), parent.id, parent.parent_id);

    let now = time::now_ms() as i64;
    let mut joined: Vec<(&Channel, i64)> = channels
        .iter()
        .filter(|channel| {
            channel.is_thread()
                && channel.parent_id == Some(parent.id)
                && !channel.is_archived_thread()
        })
        .filter_map(|thread| {
            let joined_at = rfc3339_millis(thread.member.as_ref()?.join_timestamp.as_deref())?;
            let status = thread_status(state, thread, now);
            relevance_deadlines.extend(status.relevant_until);
            let shown = if use_relevant {
                status.relevant
            } else {
                status.unread
            };
            shown.then_some((thread, joined_at))
        })
        .collect();
    joined.sort_by_key(|(_, joined_at)| cmp::Reverse(*joined_at));
    let mut threads: Vec<&Channel> = joined.into_iter().map(|(thread, _)| thread).collect();

    if parent_selected {
        return threads;
    }
    if let Some(selected) = selected_thread {
        if !threads.iter().any(|thread| thread.id == selected.id) {
            threads.insert(0, selected);
        }
        return threads;
    }
    if state.guild_settings.hides_muted_channels(guild_id) {
        threads.retain(|thread| {
            !thread_muted(thread) || state.read_states.badge(thread.id).get().mentions > 0
        });
    }
    threads
}

// Rebuilds the sidebar when the earliest shown thread stops being recently
// active, like Discord's THREAD_UPDATE timeout.
fn schedule_relevance_check(deadlines: &[i64]) {
    if let Some(handle) = THREAD_RELEVANCE_TIMER.with(Cell::take) {
        clear_timeout(handle);
    }
    let Some(&deadline) = deadlines.iter().min() else {
        return;
    };
    let delay = (deadline - time::now_ms() as i64 + 1).max(0) as u64;
    let handle = set_timeout(Duration::from_millis(delay), || {
        THREAD_RELEVANCE_TIMER.with(|timer| timer.set(None));
        THREAD_RELEVANCE.with(|tick| tick.update(|tick| *tick += 1));
    });
    THREAD_RELEVANCE_TIMER.with(|timer| timer.set(Some(handle)));
}

#[allow(clippy::too_many_arguments)]
fn add_thread_rows(
    tree: &mut WidgetTree,
    list_id: WidgetId,
    state: &AppState,
    guild_id: Id<GuildMarker>,
    parent: &Channel,
    channels: &[Channel],
    selected_channel: Option<Id<ChannelMarker>>,
    on_select: &Rc<dyn Fn(Id<ChannelMarker>)>,
    relevance_deadlines: &mut Vec<i64>,
) {
    let threads = sidebar_threads(
        state,
        guild_id,
        parent,
        channels,
        selected_channel,
        relevance_deadlines,
    );
    let Some(last) = threads.len().checked_sub(1) else {
        return;
    };

    let spine = || icon(THREAD_SPINE, SPINE_DEFAULT).class(cls::THREAD_SPINE);
    let group_id = tree.mount(
        list_id,
        StackProps::vertical()
            .gap(0)
            .build()
            .class(cls::THREAD_GROUP),
    );
    tree.mount(group_id, div().class(cls::THREAD_SPINE_BORDER));
    for (index, thread) in threads.into_iter().enumerate() {
        let mut item = StackProps::vertical()
            .gap(0)
            .build()
            .class(cls::THREAD_ITEM)
            .child(spine());
        if index != last {
            item = item.child(spine().class(cls::THREAD_SPINE_INVERTED));
        }
        tree.mount(
            group_id,
            item.child(thread_row(
                state,
                guild_id,
                thread,
                selected_channel,
                on_select,
            )),
        );
    }
}

fn thread_row(
    state: &AppState,
    guild_id: Id<GuildMarker>,
    thread: &Channel,
    selected_channel: Option<Id<ChannelMarker>>,
    on_select: &Rc<dyn Fn(Id<ChannelMarker>)>,
) -> Node {
    let thread_id = thread.id;
    let is_selected = selected_channel == Some(thread_id);
    let muted = thread_muted(thread);
    let badge = state.read_states.badge(thread_id);
    let label = thread.name.clone().unwrap_or_default();

    let content = Node::reactive(move || {
        let read = badge.get();
        let unread = read.unread && !muted && !is_selected;
        let mut inner = div().class(cls::CHANNEL_ROW_INNER);
        if unread {
            inner = inner.child(
                div()
                    .class(cls::CHANNEL_ROW_UNREAD)
                    .class(cls::CHANNEL_ROW_UNREAD_IMPORTANT)
                    .class(cls::THREAD_UNREAD),
            );
        }
        let color = if is_selected || unread {
            TEXT_STRONG
        } else if muted {
            TEXT_MUTED
        } else {
            TEXT_SUBTLE
        };
        let name = TextProps::styled(label.clone(), TextVariant::TEXT_SM_MEDIUM)
            .color(color)
            .line_clamp(1_u32)
            .build()
            .class(cls::CHANNEL_ROW_NAME);
        inner = inner.child(if muted && !is_selected {
            name.class(cls::CHANNEL_ROW_DIM)
        } else {
            name
        });
        if read.mentions > 0 {
            inner = inner.child(number_badge(NumberBadgeProps::new(read.mentions)));
        }
        inner
    })
    .class(cls::CHANNEL_ROW_REACTIVE);

    let mut classes = vec![cls::CHANNEL_ROW.to_string(), cls::THREAD_ROW.to_string()];
    if is_selected {
        classes.push(cls::CHANNEL_ROW_SELECTED.to_string());
    }
    let on_select = on_select.clone();
    let menu_state = state.clone();
    let link = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(classes)
        .on_click(move || on_select(thread_id))
        .build()
        .child(content)
        .on_context_menu(move |x, y| {
            let state = menu_state.clone();
            context_menu::open(x, y, move || {
                context_menus::channel_items(&state, Some(guild_id), thread_id)
            });
        });
    StackProps::vertical()
        .gap(0)
        .padding(Padding::y(1))
        .full_width(false)
        .build()
        .class(cls::THREAD_WRAPPER)
        .child(link)
}

#[allow(clippy::too_many_arguments)]
fn add_channel_row(
    tree: &mut WidgetTree,
    list_id: WidgetId,
    state: &AppState,
    guild_id: Id<GuildMarker>,
    channel: &Channel,
    selected_channel: Option<Id<ChannelMarker>>,
    on_select: &Rc<dyn Fn(Id<ChannelMarker>)>,
    env: &SidebarEnv,
) {
    let is_selected = selected_channel == Some(channel.id);
    let channel_id = channel.id;
    let on_select = on_select.clone();

    let mut classes = vec![channel_sidebar::CHANNEL_ROW.to_string()];
    if is_selected {
        classes.push(channel_sidebar::CHANNEL_ROW_SELECTED.to_string());
    }

    let badge = state.read_states.badge(channel_id);
    let settings = state.guild_settings.clone();
    let parent_id = channel.parent_id;

    let label = channel.name.as_deref().unwrap_or("unknown").to_string();
    let glyph = ui::channel_icon(state, channel);
    let is_voice = matches!(channel.kind, 2 | 13);
    let voice_state = state.clone();
    // The web client's per-row invite icon: shown when you can invite from here or the server
    // has a vanity link to hand out.
    let can_invite = state.can(channel, Permissions::CREATE_INSTANT_INVITE)
        || state
            .guilds
            .find(guild_id)
            .and_then(|guild| guild.vanity_url_code)
            .is_some_and(|code| !code.is_empty());

    let content = Node::reactive(move || {
        let read = badge.get();
        let muted = settings.is_channel_muted(Some(guild_id), channel_id, parent_id);
        let bright = read.unread && read.important;

        let (live, connected) = if is_voice {
            let rtc = voice_state.voice.rtc.get();
            (
                !voice_state.voice.in_channel(channel_id).is_empty(),
                rtc.channel_id == Some(channel_id)
                    && rtc.state != RtcState::Disconnected,
            )
        } else {
            (false, false)
        };
        let (name_color, icon_color) = if is_selected || connected {
            (TEXT_STRONG, TEXT_STRONG)
        } else if muted {
            (TEXT_MUTED, TEXT_MUTED)
        } else if bright {
            (TEXT_STRONG, TEXT_SECONDARY)
        } else {
            (TEXT_SUBTLE, TEXT_SUBTLE)
        };
        let fade = muted && !is_selected;
        let mut inner = div().class(cls::CHANNEL_ROW_INNER);
        if read.unread {
            let dot = div().class(cls::CHANNEL_ROW_UNREAD);
            inner = inner.child(if bright {
                dot.class(cls::CHANNEL_ROW_UNREAD_IMPORTANT)
            } else {
                dot
            });
        }
        let glyph_node = icon(
            glyph,
            if live {
                colors::TEXT_POSITIVE
            } else {
                icon_color
            },
        )
        .class(cls::CHANNEL_ROW_ICON);
        inner = inner.child(if fade {
            glyph_node.class(cls::CHANNEL_ROW_DIM)
        } else {
            glyph_node
        });
        let name = TextProps::styled(label.clone(), TextVariant::TEXT_MD_MEDIUM)
            .color(name_color)
            .line_clamp(1_u32)
            .build()
            .class(cls::CHANNEL_ROW_NAME);
        inner = inner.child(if fade {
            name.class(cls::CHANNEL_ROW_DIM)
        } else {
            name
        });
        if read.mentions > 0 {
            inner = inner.child(number_badge(NumberBadgeProps::new(read.mentions)));
        }
        if let Some(timer) = is_voice
            .then(|| voice_users::channel_timer(&voice_state, channel_id))
            .flatten()
        {
            inner = inner.child(timer);
        }
        if can_invite {
            inner = inner.child(
                TooltipProps::default()
                    .text(requiem_i18n::t!("permission-create-invite"))
                    .build(
                        ButtonProps::default()
                            .variant(ButtonVariant::Bare)
                            .extra_classes(vec![cls::CHANNEL_ROW_INVITE.to_string()])
                            .on_click(move || {
                                invite_modal::open_channel(guild_id, channel_id)
                            })
                            .build()
                            .child(
                                icon(GROUP_PLUS, TEXT_SECONDARY)
                                    .class(cls::CHANNEL_ROW_INVITE_ICON),
                            ),
                    ),
            );
        }
        inner
    })
    .class(channel_sidebar::CHANNEL_ROW_REACTIVE);

    let row = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(classes)
        .on_click(move || on_select(channel_id))
        .build()
        .child(content)
        .on_context_menu({
            let state = state.clone();
            move |x, y| {
                let state = state.clone();
                context_menu::open(x, y, move || {
                    context_menus::channel_items(&state, Some(guild_id), channel_id)
                });
            }
        });

    tree.mount(list_id, row);
    if is_voice {
        tree.mount(
            list_id,
            voice_users::list(state, guild_id, channel_id, env),
        );
    }
}
