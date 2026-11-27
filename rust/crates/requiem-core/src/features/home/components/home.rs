use crate::rest::endpoints::channels;
use crate::ui::classes::home as cls;
use crate::ui::classes::message_list;
use crate::ui::context_menu;
use crate::ui::context_menu as menu;
use crate::ui::custom_status;
use crate::ui::new_message;
use crate::ui::quick_switcher;
use crate::ui::sidebar_resize;
use crate::ui;
use requiem_ui::app;
use requiem_ui::components::NumberBadgeProps;
use requiem_ui::components::line_clamp_class;
use requiem_ui::components::{Align, StackProps};
use requiem_ui::components::{ButtonSize, ButtonVariant};
use requiem_ui::components::{HeadingLevel, TextVariant};
use requiem_ui::components;
use requiem_ui::icons::IconPath;
use requiem_ui::widget;
use std::rc::Rc;

use requiem_ui::components::{
    AvatarStatusSize, ButtonProps, ContextItem, HeadingProps, TextInputProps, TextProps,
    ScrollViewProps, avatar_with_status, div, icon, scroll_view,
};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::primitives::PresenceStatus;
use skia_safe::Color;
use skia_safe::font_style::Weight;

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::models::id::marker::{ChannelMarker, UserMarker};
use crate::models::{Id, User};
use crate::query::QueryClient;
use crate::rest::endpoints::relationships as rels;
use crate::rest::{HttpClient, RestError};
use crate::state::{AppState, DmChannel, FriendsTab, NavEvent, Relationship};
use crate::ui::{Fonts, TEXT_DEFAULT, TEXT_SECONDARY, TEXT_STRONG, TEXT_SUBTLE};
use requiem_ui::icons::{
    CHAT, CHECKMARK_SMALL_BOLD, FRIENDS, MAGNIFYING_GLASS, MORE_HORIZONTAL, NITRO_WHEEL,
    PLUS_SMALL, QUESTS, SHOP, X_SMALL,
};

#[derive(Clone)]
pub struct HomeEnv {
    pub state: AppState,
    pub fonts: Fonts,
    pub images: ImageCache,
    pub backend: BackendConfig,
    pub queries: QueryClient,
    pub http: HttpClient,

    pub tick: Signal<u64>,
    pub on_select_channel: Rc<dyn Fn(Id<ChannelMarker>)>,

    pub on_open_friends: Rc<dyn Fn()>,
}

pub fn user_name(state: &AppState, user: &User) -> String {
    state.users.display_name(user)
}

pub fn dm_title(state: &AppState, dm: &DmChannel) -> String {
    if let Some(name) = dm.name.as_deref().filter(|n| !n.is_empty()) {
        return name.to_string();
    }
    let names: Vec<String> = dm
        .recipients
        .iter()
        .filter_map(|id| state.users.get(*id))
        .map(|u| user_name(state, &u))
        .collect();
    if names.is_empty() {
        requiem_i18n::t!("home-unknown").to_string()
    } else {
        names.join(", ")
    }
}

fn status_label(status: PresenceStatus) -> String {
    match status {
        PresenceStatus::Online => requiem_i18n::t!("home-online"),
        PresenceStatus::Idle => requiem_i18n::t!("home-idle"),
        PresenceStatus::Dnd => requiem_i18n::t!("home-do-not-disturb"),
        PresenceStatus::Offline => requiem_i18n::t!("home-offline"),
    }
}

pub fn user_avatar(
    env: &HomeEnv,
    user: &User,
    size: Option<AvatarStatusSize>,
    status: Option<PresenceStatus>,
) -> Node {
    let mut props = ui::user_avatar_props(user.id, &user.discriminator, &env.fonts);
    if let Some(hash) = user.avatar.as_deref() {
        let url = env.backend.avatar_url(user.id, hash);
        let images = env.images.clone();
        let tick = env.tick.clone();
        props = props.bind_image(tick, move || images.get_or_decode_image(&url));
    }
    match size {
        Some(size) => avatar_with_status(props, size, status),
        None => props.build(),
    }
}

fn open_dm(env: &HomeEnv, user_id: Id<UserMarker>) {
    if let Some(dm) = env.state.dms.with_user(user_id) {
        (env.on_select_channel)(dm.id);
        return;
    }
    let (http, inbox) = (env.http.clone(), env.state.nav.inbox());
    env.queries.mutate(Vec::new(), move || async move {
        let channel = channels::create_dm(&http, user_id, None).await?;
        inbox
            .lock()
            .expect("nav inbox poisoned")
            .push(NavEvent::DmOpened {
                channel_id: channel.id,
                recipients: vec![user_id],
            });
        Ok::<(), RestError>(())
    });
}

#[derive(Clone, Copy)]
enum RelOp {
    Add,
    Remove,
    Ignore,
}

fn rel_action(
    env: &HomeEnv,
    user_id: Id<UserMarker>,
    op: RelOp,
    ignore_location: &'static str,
) -> impl Fn() + 'static {
    let (http, queries) = (env.http.clone(), env.queries.clone());
    move || {
        let http = http.clone();
        queries.mutate(Vec::new(), move || async move {
            match op {
                RelOp::Add => {
                    rels::send_friend_request(&http, user_id, rels::location::CONTEXT_MENU).await
                }
                RelOp::Remove => rels::remove(&http, user_id, rels::location::CONTEXT_MENU).await,
                RelOp::Ignore => rels::ignore(&http, user_id, ignore_location).await,
            }
        });
    }
}

fn person_menu(env: &HomeEnv, user_id: Id<UserMarker>, name: &str, dm: bool) -> Vec<ContextItem> {
    let kind = env.state.relationships.kind_of(user_id);

    let menu_location = if dm {
        rels::location::DM_USER_CONTEXT_MENU
    } else {
        rels::location::CONTEXT_MENU
    };
    let mutate = |op: RelOp| rel_action(env, user_id, op, menu_location);
    let open_env = env.clone();
    let mut items = Vec::new();
    if dm {
        items.extend([
            ContextItem::unavailable(requiem_i18n::t!("home-mark-as-read")),
            ContextItem::Separator,
            ContextItem::unavailable(requiem_i18n::t!("home-pin")),
            ContextItem::Separator,
        ]);
    }
    items.push(ContextItem::action(
        requiem_i18n::t!("home-profile"),
        move || (menu::actions().open_profile)(user_id, None, false),
    ));
    if !dm {
        items.push(ContextItem::action(
            requiem_i18n::t!("home-message"),
            move || open_dm(&open_env, user_id),
        ));
    }
    items.extend([
        ContextItem::unavailable(requiem_i18n::t!("home-start-a-call")),
        ContextItem::action(requiem_i18n::t!("home-edit-note"), move || {
            (menu::actions().open_profile)(user_id, None, true)
        })
        .subtitle(requiem_i18n::t!("home-only-visible-to-you")),
        ContextItem::unavailable(requiem_i18n::t!("home-add-friend-nickname")),
    ]);
    if dm {
        items.push(ContextItem::unavailable(requiem_i18n::t!("home-close-dm")));
    }
    items.extend([
        ContextItem::Separator,
        ContextItem::unavailable(requiem_i18n::t!("home-invite-to-server")),
    ]);
    match kind {
        Some(rels::FRIEND) => items.push(
            ContextItem::action(
                requiem_i18n::t!("home-remove-friend"),
                mutate(RelOp::Remove),
            )
            .danger(),
        ),
        Some(rels::BLOCKED) => {}
        Some(rels::PENDING_INCOMING) | None => items.push(ContextItem::action(
            requiem_i18n::t!("home-add-friend"),
            mutate(RelOp::Add),
        )),
        Some(_) => {}
    }
    items
        .push(ContextItem::action(requiem_i18n::t!("home-ignore"), mutate(RelOp::Ignore)).danger());
    let block_name = name.to_string();
    items.push(
        ContextItem::action(requiem_i18n::t!("home-block"), move || {
            (menu::actions().confirm_block)(user_id, block_name.clone(), menu_location)
        })
        .danger(),
    );
    if dm {
        items.extend([
            ContextItem::Separator,
            ContextItem::unavailable(requiem_i18n::t!("home-mute-user", name = name)),
        ]);
    }
    items
}

fn friend_more_menu(env: &HomeEnv, user_id: Id<UserMarker>) -> Vec<ContextItem> {
    let (http, queries) = (env.http.clone(), env.queries.clone());
    vec![
        ContextItem::unavailable(requiem_i18n::t!("home-start-video-call")),
        ContextItem::unavailable(requiem_i18n::t!("home-start-voice-call")),
        ContextItem::action(requiem_i18n::t!("home-remove-friend"), move || {
            let http = http.clone();
            queries.mutate(Vec::new(), move || async move {
                rels::remove(&http, user_id, rels::location::FRIENDS).await
            });
        })
        .danger(),
    ]
}

pub fn sidebar(env: &HomeEnv, selected: Option<Id<ChannelMarker>>) -> Node {
    let search_button = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::SEARCH_BUTTON.to_string()])
        .on_click(quick_switcher::open)
        .build()
        .child(
            TextProps::styled(
                requiem_i18n::t!("home-find-or-start-a-conversation"),
                TextVariant::TEXT_SM_MEDIUM,
            )
            .color(TEXT_STRONG)
            .line_clamp(1_u32)
            .build(),
        );
    let search = div().class(cls::SEARCH_AREA).child(search_button);

    let on_open_friends = env.on_open_friends.clone();
    let rows = [
        nav_row(
            FRIENDS,
            &requiem_i18n::t!("home-friends"),
            selected.is_none(),
            Some(on_open_friends),
        ),
        nav_row(NITRO_WHEEL, &requiem_i18n::t!("home-nitro"), false, None),
        nav_row(SHOP, &requiem_i18n::t!("home-shop"), false, None),
        nav_row(QUESTS, &requiem_i18n::t!("home-quests"), false, None),
    ];

    let header = div().class(cls::DM_HEADER).children([
        TextProps::styled(
            requiem_i18n::t!("home-direct-messages"),
            TextVariant::TEXT_SM_MEDIUM,
        )
        .color(TEXT_SUBTLE)
        .line_clamp(1_u32)
        .build()
        .class(cls::DM_HEADER_TEXT),
        ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .on_click(new_message::open)
            .build()
            .child(icon(PLUS_SMALL, TEXT_SECONDARY).class(cls::DM_HEADER_PLUS)),
    ]);

    let dm_env = env.clone();
    let dm_list = Node::reactive(move || dm_list(&dm_env, selected)).class(cls::SLOT);

    let list = scroll_view()
        .class(cls::NAV_LIST)
        .children(rows)
        .child(div().class(cls::SECTION_DIVIDER))
        .child(header)
        .child(dm_list);
    sidebar_resize::container()
        .child(search)
        .child(list)
        .child(sidebar_resize::handle())
}

fn nav_row(
    icon_path: impl IconPath,
    text: &str,
    selected: bool,
    on_click: Option<Rc<dyn Fn()>>,
) -> Node {
    let mut classes = vec![cls::NAV_ROW.to_string()];
    if selected {
        classes.push(cls::NAV_ROW_SELECTED.to_string());
    }
    let mut button = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(classes);
    if let Some(on_click) = on_click {
        button = button.on_click(move || on_click());
    }
    button.build().children([
        icon(icon_path, TEXT_SUBTLE).class(cls::NAV_ICON),
        TextProps::styled(text, TextVariant::TEXT_MD_MEDIUM)
            .color(TEXT_SUBTLE)
            .line_clamp(1_u32)
            .build()
            .class(cls::NAV_LABEL),
    ])
}

fn dm_list(env: &HomeEnv, selected: Option<Id<ChannelMarker>>) -> Node {
    let dms = env.state.dms.list.get();
    let mut list = StackProps::vertical().gap(2).build();
    for dm in &dms {
        list = list.child(dm_row(env, dm, selected == Some(dm.id)));
    }
    list
}

fn dm_row(env: &HomeEnv, dm: &DmChannel, selected: bool) -> Node {
    let content = {
        let (env, dm) = (env.clone(), dm.clone());
        let badge = env.state.read_states.badge(dm.id);
        Node::reactive(move || {
            let read = badge.get();
            let muted = env.state.guild_settings.is_channel_muted(None, dm.id, None);
            let first = dm
                .recipients
                .first()
                .and_then(|id| env.state.users.get(*id));
            let avatar = match &first {
                Some(user) => {
                    let status = (!dm.is_group())
                        .then(|| env.state.user_presence.of(user.id).status)
                        .filter(|s| *s != PresenceStatus::Offline);
                    user_avatar(&env, user, Some(AvatarStatusSize::Px32), status)
                }
                None => div(),
            }
            .class(cls::DM_AVATAR);
            let avatar = if muted {
                avatar.class(cls::DM_AVATAR_MUTED)
            } else {
                avatar
            };

            let name_color = if read.unread && !selected {
                TEXT_STRONG
            } else {
                TEXT_SUBTLE
            };
            let mut text = div().class(cls::DM_TEXT).child(
                TextProps::styled(dm_title(&env.state, &dm), TextVariant::TEXT_MD_MEDIUM)
                    .color(name_color)
                    .line_clamp(1_u32)
                    .build()
                    .class(cls::DM_NAME),
            );
            if dm.is_group() {
                text = text.child(
                    TextProps::styled(
                        requiem_i18n::t!("home-group-members", count = dm.recipients.len() + 1),
                        TextVariant::TEXT_XS_NORMAL,
                    )
                    .color(TEXT_SUBTLE)
                    .line_clamp(1_u32)
                    .build()
                    .class(cls::DM_SUBTEXT),
                );
            }
            let mut inner = div().class(cls::DM_ROW_INNER).child(avatar).child(text);
            if read.mentions > 0 {
                inner = inner.child(div().class(cls::DM_BADGE).child(
                    components::number_badge(
                        NumberBadgeProps::new(read.mentions),
                    ),
                ));
            }
            inner
        })
        .class(cls::DM_ROW_REACTIVE)
    };

    let mut classes = vec![cls::DM_ROW.to_string()];
    if selected {
        classes.push(cls::DM_ROW_SELECTED.to_string());
    }
    let (on_select, id) = (env.on_select_channel.clone(), dm.id);
    let mut row = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(classes)
        .on_click(move || on_select(id))
        .build()
        .child(content);

    if let (false, Some(user_id)) = (dm.is_group(), dm.recipients.first().copied()) {
        let (env, name) = (env.clone(), dm_title(&env.state, dm));
        row = row.on_context_menu(move |x, y| {
            let (env, name) = (env.clone(), name.clone());
            context_menu::open(x, y, move || person_menu(&env, user_id, &name, true));
        });
    }
    row
}

pub fn friends_page(env: &HomeEnv) -> Node {
    let header_env = env.clone();
    let header = Node::reactive(move || friends_header(&header_env)).class(cls::SLOT);

    let people_env = env.clone();
    let people = Node::reactive(move || match people_env.state.nav.friends_tab.get() {
        FriendsTab::AddFriend => add_friend_form(&people_env),
        tab => people_column(&people_env, tab),
    })
    .class(cls::FILL_COL);

    div().class(cls::FRIENDS_PAGE).child(header).child(
        div()
            .class(cls::FRIENDS_BODY)
            .child(people)
            .child(active_now()),
    )
}

fn tab_label(tab: FriendsTab) -> String {
    match tab {
        FriendsTab::Online => requiem_i18n::t!("home-online"),
        FriendsTab::All => requiem_i18n::t!("home-all"),
        FriendsTab::Pending => requiem_i18n::t!("home-pending"),
        FriendsTab::Blocked => requiem_i18n::t!("home-blocked"),
        FriendsTab::AddFriend => requiem_i18n::t!("home-add-friend"),
    }
}

fn friends_header(env: &HomeEnv) -> Node {
    let current = env.state.nav.friends_tab.get();
    let (pending, blocked) = env.state.relationships.list.with(|list| {
        (
            list.iter()
                .filter(|r| r.is_incoming() || r.is_outgoing())
                .count(),
            list.iter().filter(|r| r.is_blocked()).count(),
        )
    });

    let mut tabs = vec![FriendsTab::Online, FriendsTab::All];
    if pending > 0 || current == FriendsTab::Pending {
        tabs.push(FriendsTab::Pending);
    }
    if blocked > 0 || current == FriendsTab::Blocked {
        tabs.push(FriendsTab::Blocked);
    }
    tabs.push(FriendsTab::AddFriend);

    let mut header = div().class(cls::FRIENDS_HEADER).children([
        div()
            .class(cls::FRIENDS_HEADER_ICON_BOX)
            .child(icon(FRIENDS, TEXT_SUBTLE).class(cls::FRIENDS_HEADER_ICON)),
        TextProps::styled(
            requiem_i18n::t!("home-friends"),
            TextVariant::TEXT_MD_MEDIUM,
        )
        .color(TEXT_STRONG)
        .line_clamp(1_u32)
        .build()
        .class(cls::FRIENDS_TITLE),
        div().class(cls::FRIENDS_HEADER_DOT),
    ]);
    for tab in tabs {
        let mut classes = vec![cls::TAB.to_string()];
        let selected = tab == current;
        match (tab, selected) {
            (FriendsTab::AddFriend, true) => classes.push(cls::TAB_ADD_SELECTED.to_string()),
            (FriendsTab::AddFriend, false) => classes.push(cls::TAB_ADD.to_string()),
            (_, true) => classes.push(cls::TAB_SELECTED.to_string()),
            _ => {}
        }
        let tab_signal = env.state.nav.friends_tab.clone();
        let mut node = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(classes)
            .on_click(move || tab_signal.set(tab))
            .build()
            .child(
                TextProps::styled(tab_label(tab), TextVariant::TEXT_MD_MEDIUM)
                    .color(TEXT_SECONDARY)
                    .line_clamp(1_u32)
                    .build()
                    .class(cls::TAB_LABEL),
            );
        if tab == FriendsTab::Pending && pending > 0 {
            node = node.child(
                div().class(cls::TAB_BADGE).child(
                    TextProps::styled(pending.to_string(), TextVariant::TEXT_XS_BOLD)
                        .color(Color::WHITE)
                        .line_clamp(1_u32)
                        .build()
                        .class(cls::TAB_BADGE_TEXT),
                ),
            );
        }
        header = header.child(node);
    }
    header
}

fn people_column(env: &HomeEnv, tab: FriendsTab) -> Node {
    let search_signal = env.state.nav.friends_search.clone();
    let initial = search_signal.peek();
    let field = TextInputProps::new()
        .bare(true)
        .text(initial)
        .placeholder(requiem_i18n::t!("home-search"))
        .insets((0.0_f32, 12.0_f32))
        .on_change(move |text: &str| search_signal.set(text.to_string()))
        .build()
        .class(cls::PEOPLE_SEARCH_INPUT);
    let search = div().class(cls::PEOPLE_SEARCH).children([
        div()
            .class(cls::PEOPLE_SEARCH_GLYPH)
            .child(icon(MAGNIFYING_GLASS, TEXT_SECONDARY).class(cls::PEOPLE_SEARCH_GLYPH_ICON)),
        field,
    ]);

    let list_env = env.clone();
    let list = Node::reactive(move || people_list(&list_env, tab)).class(cls::LIST_SLOT);
    div().class(cls::PEOPLE_COLUMN).child(search).child(list)
}

fn empty_text(tab: FriendsTab, searching: bool) -> String {
    if searching {
        return requiem_i18n::t!("home-no-one-matches-that-search");
    }
    match tab {
        FriendsTab::Online => requiem_i18n::t!("home-no-one-s-around-to-play"),
        FriendsTab::All => requiem_i18n::t!("home-wumpus-is-waiting-on-friends-you"),
        FriendsTab::Pending => requiem_i18n::t!("home-there-are-no-pending-friend-requests"),
        FriendsTab::Blocked => requiem_i18n::t!("home-you-can-t-unblock-the-wumpus"),
        FriendsTab::AddFriend => String::new(),
    }
}

fn people_list(env: &HomeEnv, tab: FriendsTab) -> Node {
    let query = env.state.nav.friends_search.get().trim().to_lowercase();
    let relationships = env.state.relationships.list.get();

    let mut people: Vec<(&Relationship, String, String, PresenceStatus)> =
        Vec::with_capacity(relationships.len());
    for r in relationships.iter() {
        let presence = env.state.user_presence.of(r.id);
        if !match tab {
            FriendsTab::Online => r.is_friend() && presence.status != PresenceStatus::Offline,
            FriendsTab::All => r.is_friend(),
            FriendsTab::Pending => r.is_incoming() || r.is_outgoing(),
            FriendsTab::Blocked => r.is_blocked(),
            FriendsTab::AddFriend => false,
        } {
            continue;
        }
        let name = r
            .nickname
            .clone()
            .unwrap_or_else(|| user_name(&env.state, &r.user));
        let key = name.to_lowercase();
        if !query.is_empty()
            && !key.contains(&query)
            && !r.user.username.to_lowercase().contains(&query)
        {
            continue;
        }
        people.push((r, name, key, presence.status));
    }

    people.sort_by(|a, b| {
        (!a.0.is_incoming())
            .cmp(&!b.0.is_incoming())
            .then_with(|| a.2.cmp(&b.2))
    });

    if people.is_empty() {
        return div().class(cls::EMPTY_STATE).child(
            TextProps::styled(
                empty_text(tab, !query.is_empty() && !relationships.is_empty()),
                TextVariant::TEXT_MD_NORMAL,
            )
            .color(TEXT_SUBTLE)
            .line_clamp(1_u32)
            .build()
            .class(cls::EMPTY_TEXT),
        );
    }

    let title = match tab {
        FriendsTab::Online => requiem_i18n::t!("home-online"),
        FriendsTab::All => requiem_i18n::t!("home-all-friends"),
        FriendsTab::Pending => requiem_i18n::t!("home-pending"),
        _ => requiem_i18n::t!("home-blocked"),
    };
    let heading = div().class(cls::SECTION_TITLE).child(
        TextProps::styled(
            requiem_i18n::t!("home-section-title", title = title, count = people.len()),
            TextVariant::TEXT_SM_MEDIUM,
        )
        .color(TEXT_DEFAULT)
        .line_clamp(1_u32)
        .build(),
    );

    // Only the rows near the viewport are built; the rest are spacers of the same height.
    let people: Rc<Vec<(Relationship, String, PresenceStatus)>> = Rc::new(
        people
            .into_iter()
            .map(|(rel, name, _, status)| (rel.clone(), name, status))
            .collect(),
    );
    let first_band = signal(0usize);
    let (window_env, window_people, window_band) = (env.clone(), people.clone(), first_band.clone());
    let window = Node::reactive(move || {
        let (_, window_height) = app::current_window_size();
        let visible = (window_height.max(FRIEND_FALLBACK_VIEWPORT) / FRIEND_PITCH).ceil() as usize;
        let first = window_band.get();
        let start = first.saturating_sub(FRIEND_OVERSCAN);
        let end = (first + visible + FRIEND_OVERSCAN).min(window_people.len());
        let mut rows = div()
            .class(cls::FRIENDS_WINDOW)
            .child(div().style(format!("height: {}px; flex-shrink: 0", start as f32 * FRIEND_PITCH)));
        for (rel, name, status) in &window_people[start.min(end)..end] {
            rows = rows
                .child(div().class(cls::ROW_DIVIDER))
                .child(friend_row(&window_env, rel, name, *status, tab));
        }
        rows.child(div().style(format!(
            "height: {}px; flex-shrink: 0",
            window_people.len().saturating_sub(end) as f32 * FRIEND_PITCH
        )))
    })
    .class(cls::FRIENDS_WINDOW);
    ScrollViewProps::default()
        .on_scroll(move |offset: f32, _max: f32| {
            let row = ((offset - FRIEND_TITLE_HEIGHT).max(0.0) / FRIEND_PITCH) as usize;
            let band = row / FRIEND_BAND * FRIEND_BAND;
            if first_band.peek() != band {
                first_band.set(band);
            }
        })
        .build()
        .class(cls::PEOPLE_LIST)
        .child(heading)
        .child(window)
}

/// A divider (1px) and a row (61px).
const FRIEND_PITCH: f32 = 62.0;
const FRIEND_TITLE_HEIGHT: f32 = 52.0;
const FRIEND_OVERSCAN: usize = 8;
/// The window is rebuilt every this many rows scrolled, not on every pixel.
const FRIEND_BAND: usize = 4;
const FRIEND_FALLBACK_VIEWPORT: f32 = 600.0;

fn action_button(
    icon_path: impl IconPath,
    extra: Option<&str>,
    on_click: impl FnMut() + 'static,
) -> Node {
    let mut classes = vec![cls::ACTION_BUTTON.to_string()];
    classes.extend(extra.map(str::to_string));
    ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(classes)
        .on_click(on_click)
        .build()
        .child(icon(icon_path, TEXT_SECONDARY).class(cls::ACTION_ICON))
}

fn friend_row(
    env: &HomeEnv,
    rel: &Relationship,
    name: &str,
    status: PresenceStatus,
    tab: FriendsTab,
) -> Node {
    let fonts = &env.fonts;
    let user = env.state.users.resolve(&rel.user);
    let user_id = rel.id;

    let shown_status = match tab {
        FriendsTab::Pending | FriendsTab::Blocked => None,
        _ => Some(status).filter(|s| *s != PresenceStatus::Offline),
    };
    let avatar = user_avatar(env, &user, Some(AvatarStatusSize::Px32), shown_status)
        .class(cls::FRIEND_AVATAR);

    let name_row = StackProps::horizontal()
        .align(Align::Center)
        .gap(0)
        .full_width(false)
        .build()
        .children([
            TextProps::styled(name.to_string(), TextVariant::TEXT_MD_SEMIBOLD)
                .color(TEXT_STRONG)
                .line_clamp(1_u32)
                .build(),
            TextProps::styled(user.username.clone(), TextVariant::TEXT_SM_MEDIUM)
                .color(TEXT_DEFAULT)
                .line_clamp(1_u32)
                .build()
                .class(cls::FRIEND_HANDLE),
        ]);

    let subtext = if rel.is_incoming() {
        TextProps::styled(
            requiem_i18n::t!("home-incoming-friend-request"),
            TextVariant::TEXT_SM_MEDIUM,
        )
        .color(TEXT_SUBTLE)
        .line_clamp(1_u32)
        .build()
    } else if rel.is_outgoing() {
        TextProps::styled(
            requiem_i18n::t!("home-outgoing-friend-request"),
            TextVariant::TEXT_SM_MEDIUM,
        )
        .color(TEXT_SUBTLE)
        .line_clamp(1_u32)
        .build()
    } else if rel.is_blocked() {
        TextProps::styled(
            requiem_i18n::t!("home-blocked"),
            TextVariant::TEXT_SM_MEDIUM,
        )
        .color(TEXT_SUBTLE)
        .line_clamp(1_u32)
        .build()
    } else {
        let custom = env
            .state
            .user_presence
            .of(user_id)
            .custom_status
            .and_then(|c| {
                custom_status::inline_status_widget(
                    &c,
                    14.0,
                    18.0,
                    16.0,
                    Weight::MEDIUM,
                    TEXT_SUBTLE,
                    fonts,
                )
            });
        match custom {
            Some(widget) => Node::new("span", widget).class(line_clamp_class(1)),
            None => TextProps::styled(status_label(status), TextVariant::TEXT_SM_MEDIUM)
                .color(TEXT_SUBTLE)
                .line_clamp(1_u32)
                .build(),
        }
    };
    let text = div().class(cls::FRIEND_TEXT).children([name_row, subtext]);

    let mut main = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::FRIEND_MAIN.to_string()]);
    if rel.is_friend() {
        let open_env = env.clone();
        main = main.on_click(move || open_dm(&open_env, user_id));
    }
    let main = main.build().children([avatar, text]);

    let (http, queries) = (env.http.clone(), env.queries.clone());
    let remove = {
        let (http, queries) = (http.clone(), queries.clone());
        move || {
            let http = http.clone();
            queries.mutate(Vec::new(), move || async move {
                rels::remove(&http, user_id, rels::location::FRIENDS).await
            });
        }
    };
    let actions = if rel.is_incoming() {
        let accept = move || {
            let http = http.clone();
            queries.mutate(Vec::new(), move || async move {
                rels::send_friend_request(&http, user_id, rels::location::FRIENDS).await
            });
        };
        vec![
            action_button(CHECKMARK_SMALL_BOLD, Some(cls::ACTION_ACCEPT), accept),
            action_button(X_SMALL, Some(cls::ACTION_REJECT), remove),
        ]
    } else if rel.is_outgoing() || rel.is_blocked() {
        vec![action_button(X_SMALL, Some(cls::ACTION_REJECT), remove)]
    } else {
        let open_env = env.clone();
        {
            let more_env = env.clone();
            let more = ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![cls::ACTION_BUTTON.to_string()])
                .on_click_at(move |rect: widget::AbsoluteRect| {
                    let env = more_env.clone();
                    context_menu::open(
                        rect.x + rect.width - 188.0,
                        rect.y + rect.height + 4.0,
                        move || friend_more_menu(&env, user_id),
                    );
                })
                .build()
                .child(icon(MORE_HORIZONTAL, TEXT_SECONDARY).class(cls::ACTION_ICON));
            vec![
                action_button(CHAT, None, move || open_dm(&open_env, user_id)),
                more,
            ]
        }
    };

    let (menu_env, menu_name) = (env.clone(), name.to_string());
    div()
        .class(cls::FRIEND_ROW)
        .child(main)
        .child(div().class(cls::FRIEND_ACTIONS).children(actions))
        .on_context_menu(move |x, y| {
            let (env, name) = (menu_env.clone(), menu_name.clone());
            context_menu::open(x, y, move || person_menu(&env, user_id, &name, false));
        })
}

fn active_now() -> Node {
    let heading = HeadingProps::default()
        .level(HeadingLevel::H2)
        .variant(TextVariant::HEADING_MD_SEMIBOLD)
        .content(requiem_i18n::t!("home-active-now").to_string())
        .color(TEXT_STRONG)
        .build()
        .class(cls::ACTIVE_NOW_TITLE);
    let card = div().class(cls::ACTIVE_NOW_CARD).children([
        TextProps::styled(
            requiem_i18n::t!("home-it-s-quiet-for-now").to_string(),
            TextVariant::TEXT_MD_SEMIBOLD,
        )
        .color(TEXT_STRONG)
        .center(true)
        .build()
        .class(cls::ACTIVE_NOW_CARD_TITLE),
        TextProps::styled(
            requiem_i18n::t!("home-when-a-friend-starts-an-activity").to_string(),
            TextVariant::TEXT_SM_NORMAL,
        )
        .color(TEXT_SUBTLE)
        .center(true)
        .build()
        .class(cls::ACTIVE_NOW_CARD_TEXT),
    ]);
    div().class(cls::ACTIVE_NOW).child(heading).child(card)
}

fn add_friend_form(env: &HomeEnv) -> Node {
    let username: Signal<String> = signal(String::new());

    let submit: Rc<dyn Fn()> = {
        let (http, queries, inbox, username, result) = (
            env.http.clone(),
            env.queries.clone(),
            env.state.nav.inbox(),
            username.clone(),
            env.state.nav.add_friend_result.clone(),
        );
        Rc::new(move || {
            let name = username.peek().trim().to_string();
            if name.is_empty() {
                return;
            }
            result.set(None);

            username.set(String::new());
            let (http, inbox) = (http.clone(), inbox.clone());
            queries.mutate(Vec::new(), move || async move {
                let error = rels::send_friend_request_by_username(&http, &name)
                    .await
                    .err();
                inbox
                    .lock()
                    .expect("nav inbox poisoned")
                    .push(NavEvent::FriendRequestSent {
                        username: name,
                        error,
                    });
                Ok::<(), RestError>(())
            });
        })
    };

    let (change_username, submit_on_enter) = (username.clone(), submit.clone());
    let input = TextInputProps::new()
        .bare(true)
        .placeholder(requiem_i18n::t!("home-enter-a-username"))
        .insets((0.0_f32, 12.0_f32))
        .on_change(move |text: &str| change_username.set(text.to_string()))
        .on_submit(move |_: String| submit_on_enter())
        .build()
        .class(cls::ADD_FRIEND_INPUT)
        .dom_id("add-friend-input");

    let button_username = username.clone();
    let button_submit = submit.clone();
    let button = Node::reactive(move || {
        let empty = button_username.get().trim().is_empty();
        let submit = button_submit.clone();
        ButtonProps::default()
            .variant(ButtonVariant::Primary)
            .size(ButtonSize::Small)
            .label(requiem_i18n::t!("home-send-friend-request").to_string())
            .disabled(empty)
            .on_click(move || submit())
            .build()
    });

    let result_env = env.clone();
    let result = Node::reactive(move || match result_env.state.nav.add_friend_result.get() {
        Some((message, ok)) => TextProps::styled(message, TextVariant::TEXT_SM_NORMAL)
            .color(if ok {
                Color::from_rgb(0x5e, 0xb4, 0x79)
            } else {
                Color::from_rgb(0xf8, 0x7e, 0x7a)
            })
            .line_clamp(1_u32)
            .build()
            .class(cls::ADD_FRIEND_RESULT),
        None => div(),
    });

    let form = div().class(cls::ADD_FRIEND).children([
        HeadingProps::default()
            .level(HeadingLevel::H2)
            .variant(TextVariant::HEADING_MD_SEMIBOLD)
            .content(requiem_i18n::t!("home-add-friend").to_string())
            .color(TEXT_STRONG)
            .build()
            .class(cls::ADD_FRIEND_TITLE),
        TextProps::styled(
            requiem_i18n::t!("home-you-can-add-friends-with-their"),
            TextVariant::TEXT_MD_NORMAL,
        )
        .color(TEXT_DEFAULT)
        .line_clamp(1_u32)
        .build()
        .class(cls::ADD_FRIEND_HINT),
        div().class(cls::ADD_FRIEND_FIELD).children([input, button]),
        result,
    ]);
    div().class(cls::PEOPLE_COLUMN).child(form)
}

pub fn friend_request_message(username: &str, error: Option<&str>) -> (String, bool) {
    match error {
        None => (
            requiem_i18n::t!("home-friend-request-sent", username = username),
            true,
        ),
        Some(message) => (message.to_string(), false),
    }
}

pub fn dm_header(env: &HomeEnv, dm: &DmChannel, toolbar: Option<Node>) -> Node {
    let first = dm
        .recipients
        .first()
        .and_then(|id| env.state.users.get(*id));
    let mut header = div().class(message_list::CHANNEL_HEADER);
    if let Some(user) = first.filter(|_| !dm.is_group()) {
        let status = env.state.user_presence.of(user.id).status;
        header = header.child(
            user_avatar(env, &user, Some(AvatarStatusSize::Px20), Some(status))
                .class(cls::DM_HEADER_AVATAR),
        );
    }
    header = header.child(
        HeadingProps::default()
            .level(HeadingLevel::H3)
            .variant(TextVariant::HEADING_MD_MEDIUM)
            .content(dm_title(&env.state, dm))
            .color(TEXT_STRONG)
            .build(),
    );
    if let Some(toolbar) = toolbar {
        header = header
            .child(div().class(message_list::CHANNEL_HEADER_SPACER))
            .child(toolbar);
    }
    header
}
