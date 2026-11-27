use crate::ui::classes::member_list as cls;
use std::cell::Cell;
use crate::gateway::guild_subscriptions;
use crate::ui::author_style::TagPlace;
use crate::ui::author_style;
use crate::ui::context_menu;
use crate::ui::context_menus;
use crate::ui::custom_status;
use crate::ui::profile_popout;
use crate::ui;
use requiem_ui::app;
use requiem_ui::signal;
use requiem_ui::timer;
use requiem_ui::widget;
use std::rc::Rc;
use std::time::Duration;

use requiem_ui::components::ButtonVariant;
use requiem_ui::components::TextVariant;
use requiem_ui::components::line_clamp_class;

use requiem_ui::components::{
    AvatarStatusSize, ButtonProps, ScrollViewProps, TextProps, avatar_with_status, div, icon,
};
use requiem_ui::node::Node;
use requiem_ui::signal::Signal;
use requiem_ui::widget::primitives::PresenceStatus;
use skia_safe::Color;
use skia_safe::font_style::Weight;

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::models::id::marker::GuildMarker;
use crate::models::{Id, Role};
use crate::query::QueryClient;
use crate::rest::HttpClient;
use crate::state::AppState;
use crate::state::member_list::{ListedMember, MemberListRow};
use crate::state::wire::GroupId;

use crate::ui::Fonts;
use requiem_ui::icons::CROWN;

#[derive(Clone)]
pub struct MemberListState {
    pub open: Signal<bool>,

    pub dm_profile_open: Signal<bool>,

    pub dm_mutuals_open: Signal<bool>,

    pub scroll_offset: Signal<f32>,

    pub popout_target: Signal<Option<profile_popout::MemberPopoutTarget>>,

    pub on_popout_open: Rc<dyn Fn()>,

    pub window_tick: Signal<u64>,
}

impl MemberListState {
    pub fn new() -> Self {
        Self {
            open: signal::signal(true),
            dm_profile_open: signal::signal(true),
            dm_mutuals_open: signal::signal(false),
            scroll_offset: signal::signal(0.0),
            popout_target: signal::signal(None),
            on_popout_open: Rc::new(|| {}),
            window_tick: signal::signal(0),
        }
    }
}

impl Default for MemberListState {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build(
    state: &AppState,
    guild_id: Id<GuildMarker>,
    ui: &MemberListState,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    queries: &QueryClient,
    http: Option<&HttpClient>,
) -> Node {
    let list = state.members.active_list(guild_id);
    let owner_id = state.guilds.list.with(|guilds| {
        guilds
            .iter()
            .find(|g| g.id == guild_id)
            .and_then(|g| g.owner_id)
    });

    let from_store = author_style::guild_roles(state, guild_id);
    let roles: &[Role] = &from_store;

    let menu_env = context_menus::MenuEnv {
        backend: backend.clone(),
        queries: queries.clone(),
        http: http.cloned(),
        images: images.clone(),
        on_save: None,
        own_user_id: state
            .users
            .current
            .with_untracked(|u| u.as_ref().map(|u| u.id)),
        state: state.clone(),
    };

    let rows: &[Option<MemberListRow>] = list.as_ref().map_or(&[], |l| l.rows.as_slice());
    let heights: Vec<f32> = rows
        .iter()
        .map(|row| row.as_ref().map_or(PLACEHOLDER_HEIGHT, row_height))
        .collect();
    let total: f32 = heights.iter().sum();
    let (_, window_height) = app::current_window_size();
    let viewport = if window_height > 200.0 {
        window_height
    } else {
        1200.0
    };
    let _ = ui.window_tick.get();
    let offset = ui
        .scroll_offset
        .peek()
        .clamp(0.0, (total - viewport).max(0.0));
    let (window_start, window_end) = (
        (offset - MEMBER_OVERSCAN_PX).max(0.0),
        offset + viewport + MEMBER_OVERSCAN_PX,
    );
    let mut column = div().class(cls::LIST);
    let mut y = 0.0f32;
    let (mut first_y, mut built_end) = (None::<f32>, 0.0f32);
    for (row, height) in rows.iter().zip(&heights) {
        let (top, bottom) = (y, y + height);
        y = bottom;
        if bottom < window_start || top > window_end {
            continue;
        }
        if first_y.is_none() {
            first_y = Some(top);
            if top > 0.0 {
                column = column.child(div().style(format!("height: {top}px; flex-shrink: 0")));
            }
        }
        built_end = bottom;
        column = column.child(match row {
            None => member_placeholder(),
            Some(MemberListRow::Group { id, count }) => {
                // The list's `groups` carry the counts; a group row inside an op may omit it.
                let count = list
                    .as_ref()
                    .and_then(|l| l.groups.iter().find(|g| g.id == *id))
                    .map_or(*count, |g| g.count);
                group_header(*id, count, roles)
            }
            Some(MemberListRow::Member(member)) => member_row(
                member,
                owner_id == Some(member.member.user.id),
                guild_id,
                ui,
                roles,
                fonts,
                images,
                backend,
                &menu_env,
            ),
        });
    }
    if built_end < total {
        column =
            column.child(div().style(format!("height: {}px; flex-shrink: 0", total - built_end)));
    }

    let built_start = first_y.unwrap_or(0.0);
    let (scroll_write, window_tick, rebuild_requested) = (
        ui.scroll_offset.clone(),
        ui.window_tick.clone(),
        Cell::new(false),
    );
    let scroll = ScrollViewProps::default()
        .initial_offset_y(ui.scroll_offset.peek())
        .on_scroll(move |offset_y, _max| {
            scroll_write.set(offset_y);
            ranges_changed(offset_y, viewport);
            let near_top = built_start > 0.0 && offset_y < built_start + MEMBER_EDGE_PX;
            let near_bottom = built_end < total && offset_y + viewport > built_end - MEMBER_EDGE_PX;
            if (near_top || near_bottom) && !rebuild_requested.replace(true) {
                window_tick.update(|n| *n += 1);
            }
        })
        .build()
        .class(cls::SCROLL)
        .child(column);
    div().class(cls::PANEL).child(scroll)
}

const PLACEHOLDER_HEIGHT: f32 = 42.0;

fn member_placeholder() -> Node {
    div().class(cls::PLACEHOLDER).children([
        div().class(cls::PLACEHOLDER_AVATAR),
        div().class(cls::PLACEHOLDER_NAME),
    ])
}

fn ranges_changed(offset_y: f32, height: f32) {
    thread_local! {
        static LATEST: Cell<(f32, f32)> = const { Cell::new((0.0, 0.0)) };
        static GENERATION: Cell<u64> = const { Cell::new(0) };
    }
    LATEST.with(|l| l.set((offset_y, height)));
    let generation = GENERATION.with(|g| {
        g.set(g.get() + 1);
        g.get()
    });
    timer::set_timeout(Duration::from_millis(50), move || {
        if GENERATION.with(Cell::get) == generation {
            let (y, h) = LATEST.with(Cell::get);
            guild_subscriptions::member_list_scrolled(y, h);
        }
    });
}

const MEMBER_OVERSCAN_PX: f32 = 600.0;
const MEMBER_EDGE_PX: f32 = 200.0;

fn row_height(row: &MemberListRow) -> f32 {
    match row {
        MemberListRow::Group { .. } => 42.0,
        MemberListRow::Member(member) => {
            let shows_status = member.custom_status.as_ref().is_some_and(|c| {
                !c.text.is_empty() || c.emoji.as_ref().is_some_and(|e| e.id.is_none())
            });
            if shows_status { 44.0 } else { 42.0 }
        }
    }
}

fn group_header(id: GroupId, count: usize, roles: &[Role]) -> Node {
    let name = match id {
        GroupId::Online => requiem_i18n::t!("member-list-online"),
        GroupId::Offline => requiem_i18n::t!("member-list-offline"),
        GroupId::Role(role_id) => roles
            .iter()
            .find(|r| r.id == role_id)
            .map_or_else(|| role_id.to_string(), |r| r.name.clone()),
    };
    let label = TextProps::styled(
        requiem_i18n::t!("member-list-group-header", name = name, count = count),
        TextVariant::TEXT_SM_MEDIUM,
    )
    .color(ui::TEXT_SUBTLE)
    .line_clamp(1_u32)
    .build()
    .class(cls::GROUP_LABEL);
    div().class(cls::GROUP).child(label)
}

fn role_fill(member: &ListedMember, roles: &[Role]) -> Option<author_style::NameFill> {
    let role = roles
        .iter()
        .filter(|r| {
            (r.color != 0 || r.colors.primary_color != 0) && member.member.roles.contains(&r.id)
        })
        .max_by_key(|r| r.position)?;
    author_style::role_fill(role)
}

#[allow(clippy::too_many_arguments)]
fn member_row(
    member: &ListedMember,
    is_owner: bool,
    guild_id: Id<GuildMarker>,
    ui: &MemberListState,
    roles: &[Role],
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    menu_env: &context_menus::MenuEnv,
) -> Node {
    let user = &member.member.user;
    let offline = member.status == PresenceStatus::Offline;

    let mut avatar = ui::user_avatar_props(user.id, &user.discriminator, fonts);
    if let Some(image) = user
        .avatar
        .as_deref()
        .and_then(|hash| images.get_or_decode_image(&backend.avatar_url(user.id, hash)))
    {
        avatar = avatar.image(image);
    }

    let avatar_wrap = avatar_with_status(
        avatar,
        AvatarStatusSize::Px32,
        (!offline).then_some(member.status),
    )
    .class(cls::AVATAR_WRAP);

    let display = user.display_name(member.member.nick.as_deref());
    let fill = role_fill(member, roles);
    let name = match &fill {
        Some(_) => author_style::name_node(
            display.clone(),
            TextVariant::TEXT_MD_MEDIUM,
            20.0,
            fill.as_ref(),
            ui::TEXT_SUBTLE,
        )
        .class(line_clamp_class(1)),
        None => TextProps::styled(display.clone(), TextVariant::TEXT_MD_MEDIUM)
            .line_clamp(1_u32)
            .color(ui::TEXT_SUBTLE)
            .build()
            .class(cls::NAME),
    };
    let mut name_row = div().class(cls::NAME_ROW).child(name);
    if let Some(tag) = author_style::guild_tag(user) {
        name_row = name_row.child(author_style::tag_chip(
            &tag,
            TagPlace::List,
            images,
            backend,
        ));
    }
    if user.bot {
        name_row = name_row.child(
            div().class(cls::APP_TAG).child(
                TextProps::styled(
                    requiem_i18n::t!("common-app-tag"),
                    TextVariant::TEXT_XS_SEMIBOLD,
                )
                .color(Color::WHITE)
                .build()
                .class(cls::APP_TAG_TEXT),
            ),
        );
    }
    if is_owner {
        name_row = name_row.child(icon(CROWN, Color::WHITE).class(cls::OWNER_ICON));
    }

    let status_line = member.custom_status.as_ref().and_then(|c| {
        custom_status::inline_status_widget(
            c,
            12.0,
            16.0,
            14.0,
            Weight::MEDIUM,
            ui::TEXT_SUBTLE,
            fonts,
        )
    });
    let mut names = div().class(cls::NAMES).child(name_row);
    let has_status = status_line.is_some();
    if let Some(line) = status_line {
        names = names.child(
            Node::new("span", line)
                .class(cls::SUBTEXT)
                .class(line_clamp_class(1)),
        );
    }
    let inner = div().class(cls::ROW_INNER).child(avatar_wrap).child(names);

    let user_id = user.id;
    let status = member.status;
    let (target, on_open) = (ui.popout_target.clone(), ui.on_popout_open.clone());
    let selected = ui.popout_target.get().is_some_and(|t| t.user_id == user_id);
    let mut row = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::ROW.to_string()])
        .on_click_at(move |rect: widget::AbsoluteRect| {
            if target.peek().is_some_and(|t| t.user_id == user_id) {
                target.set(None);
                return;
            }
            on_open();
            target.set(Some(profile_popout::MemberPopoutTarget {
                user_id,
                guild_id,
                status,
                row: rect,
            }));
        })
        .build()
        .child(inner);
    let (env, target) = (
        menu_env.clone(),
        Rc::new(context_menus::UserTarget {
            user_id,
            guild_id: Some(guild_id),
            display_name: display,
            webhook: false,
        }),
    );
    row = row.on_context_menu(move |x, y| {
        let (env, target) = (env.clone(), target.clone());
        context_menu::open(x, y, move || {
            context_menus::user_items(&env, &target)
        });
    });
    if selected {
        row = row.class(cls::ROW_SELECTED);
    }
    if has_status {
        row = row.class(cls::ROW_WITH_STATUS);
    }
    if offline {
        row = row.class(cls::ROW_OFFLINE);
    }
    row
}
