use crate::rest::endpoints::relationships::BLOCKED;
use crate::ui::classes::profile_popout as cls;
use requiem_ui::components::MenuItem;
use requiem_ui::signal::signal;
use std::time::Duration;
use chrono::{TimeZone, Utc};
use crate::models::user_profile;
use crate::models;
use crate::rest::endpoints::channels;
use crate::rest::endpoints::messages::send_location;
use crate::rest::endpoints::messages;
use crate::rest::endpoints::relationships::location;
use crate::rest::endpoints::relationships;
use crate::rest::endpoints::users;
use crate::state;
use crate::ui::author_style;
use crate::ui::avatar_color;
use crate::ui::classes::author_style as classes_author_style;
use crate::ui::classes::profile_popout;
use crate::ui::context_menu;
use crate::ui::custom_status;
use crate::ui::emoji_button_icon::EmojiButtonIconWidget;
use crate::ui::full_profile::FullProfileState;
use crate::ui::message_list;
use crate::ui::profile_card::CardContainer;
use crate::ui::profile_card;
use crate::ui::profile_decorations;
use crate::ui::profile_theme;
use crate::ui;
use crate::utils::prefs;
use crate::utils::time;
use requiem_ui::colors::{DEFAULT_ROLE, TEXT_MUTED};
use requiem_ui::components::Anchor;
use requiem_ui::components::HelpMessageProps;
use requiem_ui::components::PopoverSide;
use requiem_ui::components::TextVariant;
use requiem_ui::components::menu;
use requiem_ui::components::{Align, Padding, StackProps, skeleton};
use requiem_ui::components::{ButtonSize, ButtonVariant};
use requiem_ui::timer;
use requiem_ui::widget::primitives::PopoverPanelWidget;
use skia_safe::{Canvas, Color, Paint, Rect};
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::Arc;

use skia_safe::RRect;
use std::rc::Rc;

use requiem_ui::components::{
    ButtonProps, PopoverProps, TextInputProps, TextProps, TooltipProps, div, icon, popover_backdrop,
};
use requiem_ui::widget::anim::ToggleState;

use crate::presence::CustomStatus;
use requiem_ui::node::Node;
use requiem_ui::signal::Signal;
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::primitives::{
    ImageWidget, PresenceStatus, corner_radii, has_any_corner_radius,
};
use requiem_ui::widget::{AbsoluteRect, Widget};

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::models::id::marker::{GuildMarker, MessageMarker, RoleMarker, UserMarker};
use crate::models::{Id, Role, UserProfile};
use crate::query::QueryClient;
use crate::rest::{HttpClient, RestError};

use crate::ui::emoji_picker::{self, EmojiPickerSignals, EmojiPickerState, PickerTab};
use crate::ui::gif_picker::GifPickerState;
use crate::ui::{Fonts, TEXT_DEFAULT, TEXT_SECONDARY, TEXT_STRONG};

#[derive(Clone)]
pub struct ProfilePopoutState {
    pub open: Signal<Option<Id<MessageMarker>>>,

    pub draft: Signal<String>,
    pub emoji_open: Signal<bool>,
    pub emoji_tab: Signal<PickerTab>,
    pub emoji_search: Signal<String>,
    pub emoji_scroll_offset: Signal<f32>,
    pub emoji_category_jump: Signal<Option<emoji_picker::PickerCategory>>,
    pub emoji_hovered: Signal<Option<&'static emojis::Emoji>>,
    pub emoji_collapsed: Signal<Vec<emoji_picker::PickerCategory>>,

    pub role_picker_open: Signal<bool>,
    pub overlays: ProfileOverlays,

    pub window_resize_tick: Signal<u64>,
}

#[derive(Clone)]
pub struct ProfileOverlays {
    pub more_menu_open: Signal<bool>,

    pub friend_menu_open: Signal<bool>,

    pub popout_user: Signal<Option<Id<UserMarker>>>,

    pub popout_anchor: Signal<AbsoluteRect>,

    pub block_confirm: Signal<Option<(Id<UserMarker>, String, &'static str)>>,
    pub full_profile: Signal<Option<FullProfileTarget>>,

    pub member_target: Signal<Option<MemberPopoutTarget>>,

    pub role_search: Signal<String>,

    pub full_profile_ui: FullProfileState,

    pub block_open: Signal<ToggleState>,

    pub on_open_custom_status: Rc<dyn Fn(Option<CustomStatus>)>,

    pub on_clear_custom_status: Rc<dyn Fn()>,

    pub on_edit_profile: Rc<dyn Fn()>,

    pub on_open_guild: Rc<dyn Fn(Id<GuildMarker>)>,

    pub popout_hovered: Signal<bool>,

    pub prompt: Signal<&'static str>,
    popout_hover_timer: Rc<Cell<Option<timer::TimeoutHandle>>>,
}

impl ProfileOverlays {
    pub fn new() -> Self {
        Self {
            more_menu_open: signal(false),
            friend_menu_open: signal(false),
            popout_user: signal(None),
            popout_anchor: signal(AbsoluteRect::default()),
            block_confirm: signal(None),
            full_profile: signal(None),
            member_target: signal(None),
            role_search: signal(String::new()),
            full_profile_ui: FullProfileState::new(),
            block_open: signal(ToggleState::new(false)),
            on_open_custom_status: Rc::new(|_| {}),
            on_clear_custom_status: Rc::new(|| {}),
            on_edit_profile: Rc::new(|| {}),
            on_open_guild: Rc::new(|_| {}),
            popout_hovered: signal(false),
            prompt: signal(custom_status::PROMPTS[0]),
            popout_hover_timer: Rc::new(Cell::new(None)),
        }
    }

    fn set_popout_hovered(&self, inside: bool) {
        if let Some(handle) = self.popout_hover_timer.take() {
            timer::clear_timeout(handle);
        }
        if inside {
            if !self.popout_hovered.peek() {
                self.popout_hovered.set(true);
            }
        } else {
            let hovered = self.popout_hovered.clone();
            self.popout_hover_timer
                .set(Some(timer::set_timeout(
                    Duration::from_millis(500),
                    move || hovered.set(false),
                )));
        }
    }

    pub fn open_full_profile(&self, target: FullProfileTarget) {
        self.full_profile.set(Some(target));
        self.full_profile_ui.open_on(target);
    }

    pub fn confirm_block(&self, user_id: Id<UserMarker>, name: String, location: &'static str) {
        self.block_confirm.set(Some((user_id, name, location)));
        self.block_open.update(|t| *t = t.set(true));
    }

    pub fn close_block_confirm(&self) {
        if self.block_open.peek().on {
            self.block_open.update(|t| *t = t.set(false));
        }
    }
}

impl Default for ProfileOverlays {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FullProfileTarget {
    pub user_id: Id<UserMarker>,
    pub guild_id: Option<Id<GuildMarker>>,
    pub focus_note: bool,

    pub tab: FullProfileTab,

    pub server_guild_id: Option<Id<GuildMarker>>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FullProfileTab {
    #[default]
    First,
    MutualFriends,
    MutualServers,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MemberPopoutTarget {
    pub user_id: Id<UserMarker>,
    pub guild_id: Id<GuildMarker>,
    pub status: PresenceStatus,
    pub row: AbsoluteRect,
}

pub fn close_popouts(state: &ProfilePopoutState) {
    if state.open.peek().is_some() {
        state.open.set(None);
    }
    if state.overlays.popout_user.peek().is_some() {
        state.overlays.popout_user.set(None);
    }
    if state.overlays.member_target.peek().is_some() {
        state.overlays.member_target.set(None);
    }
    if state.overlays.more_menu_open.peek() {
        state.overlays.more_menu_open.set(false);
    }
    if state.overlays.friend_menu_open.peek() {
        state.overlays.friend_menu_open.set(false);
    }

    if state.role_picker_open.peek() {
        state.role_picker_open.set(false);
    }
}

pub fn relationship_kind(
    relationships: &state::RelationshipStore,
    user_id: Id<UserMarker>,
) -> Option<u8> {
    relationships
        .list
        .with(|list| list.iter().find(|r| r.id == user_id).map(|r| r.kind))
}

pub const PANEL_DOM_ID: &str = "profile-popout-panel";

pub const EMOJI_PANEL_DOM_ID: &str = "profile-popout-emoji-picker-panel";

pub fn load_profile(
    queries: &QueryClient,
    http: Option<&HttpClient>,
    users: &state::UserStore,
    user_id: Id<UserMarker>,
    guild_id: Option<Id<GuildMarker>>,
    surface: ProfileSurface,
) -> Option<Arc<UserProfile>> {
    let key = profile_query_key_for(surface, user_id, guild_id);
    let profile = match http {
        Some(http) => {
            let http = http.clone();

            let is_self = users
                .current
                .with_untracked(|me| me.as_ref().map(|me| me.id))
                == Some(user_id);
            let is_bot = users.get(user_id).is_some_and(|user| user.bot);
            let view = match surface {
                ProfileSurface::Popout => ProfileView::Popout { is_self, is_bot },
                ProfileSurface::Modal => ProfileView::Modal { is_bot },
                ProfileSurface::Sidebar => ProfileView::Sidebar,
                ProfileSurface::Plain => ProfileView::Plain,
            };
            let result = queries.query_default::<UserProfile, _, RestError>(
                &key,
                move || async move {
                    users::get_user_profile(&http, user_id, guild_id, view)
                        .await
                },
            );
            if result.is_hard_error() {
                // The request failed: show what the client already knows. The full profile
                // says so with a toast; the popout just shows the smaller card.
                if surface == ProfileSurface::Modal {
                    warn_partial(&key);
                }
                return Some(Arc::new(known_profile(users, user_id)));
            }
            result.data
        }
        None => queries.peek::<UserProfile>(&key),
    };
    if let Some(profile) = &profile {
        users.observe_profile(&profile.user);
        if let Some(friends) = &profile.mutual_friends {
            users.set_mutual_friends(user_id, friends);
        }
    }
    profile
}

/// The warning callout shown when a profile couldn't be fully loaded.
fn partial_notice() -> Node {
    HelpMessageProps::default()
        .content(
            TextProps::styled(
                requiem_i18n::t!("full-profile-load-partial").to_string(),
                TextVariant::TEXT_SM_MEDIUM,
            )
            .color(TEXT_DEFAULT)
            .build(),
        )
        .build()
}

/// A profile made only of what the client has already seen of the user.
fn known_profile(users: &state::UserStore, user_id: Id<UserMarker>) -> UserProfile {
    let user = users.get(user_id);
    UserProfile {
        user: user_profile::ProfileUser {
            id: user_id,
            username: user
                .as_ref()
                .map_or_else(|| requiem_i18n::t!("home-unknown").to_string(), |u| u.username.clone()),
            discriminator: user.as_ref().map_or_else(|| "0".to_string(), |u| u.discriminator.clone()),
            global_name: user.as_ref().and_then(|u| u.global_name.clone()),
            avatar: user.as_ref().and_then(|u| u.avatar.clone()),
            bot: user.as_ref().is_some_and(|u| u.bot),
            primary_guild: user.as_ref().and_then(|u| u.primary_guild.clone()),
            avatar_decoration_data: user
                .as_ref()
                .and_then(|u| u.avatar_decoration_data.clone()),
            collectibles: user.as_ref().and_then(|u| u.collectibles.clone()),
            display_name_styles: user
                .as_ref()
                .and_then(|u| u.display_name_styles.clone()),
            banner: None,
            accent_color: None,
            bio: String::new(),
        },
        user_profile: None,
        guild_member_profile: None,
        badges: Vec::new(),
        premium_type: None,
        mutual_guilds: None,
        mutual_friends: None,
        mutual_friends_count: None,
        guild_member: None,
        connected_accounts: Vec::new(),
        partial: true,
    }
}

thread_local! {
    static WARNED: RefCell<HashSet<String>> =
        RefCell::new(HashSet::new());
}

/// One "Unable to load parts of profile" toast per failed profile, not one per redraw.
fn warn_partial(key: &str) {
    if !WARNED.with(|warned| warned.borrow_mut().insert(key.to_string())) {
        return;
    }
    // Shown after this draw: a toast changes state the draw is reading.
    timer::set_timeout(Duration::ZERO, || {
        context_menu::notify_failure(&requiem_i18n::t!("full-profile-load-partial"));
    });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileSurface {
    Popout,
    Modal,
    Sidebar,
    Plain,
}

use crate::rest::endpoints::users::ProfileView;
use requiem_ui::icons::{
    ID, MORE_HORIZONTAL, PAPER_PLUS, PENCIL, PLUS_MEDIUM, USER_CHECK, USER_PLUS, X_SMALL,
};

pub fn profile_query_key_for(
    surface: ProfileSurface,
    user_id: Id<UserMarker>,
    guild_id: Option<Id<GuildMarker>>,
) -> String {
    match surface {
        ProfileSurface::Popout => profile_query_key(user_id, guild_id),
        ProfileSurface::Modal => format!("{}:modal", profile_query_key(user_id, guild_id)),
        ProfileSurface::Sidebar => format!("{}:sidebar", profile_query_key(user_id, guild_id)),
        ProfileSurface::Plain => format!("{}:plain", profile_query_key(user_id, guild_id)),
    }
}

pub fn profile_query_key(user_id: Id<UserMarker>, guild_id: Option<Id<GuildMarker>>) -> String {
    match guild_id {
        Some(guild_id) => format!("user-profile:{user_id}:{guild_id}"),
        None => format!("user-profile:{user_id}"),
    }
}

pub const DEFAULT_ROLE_COLOR: Color = Color::from_argb(255, 0xc4, 0xc9, 0xce);

pub struct ColorBoxWidget {
    pub color: Color,
}

impl Widget for ColorBoxWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
        let mut paint = Paint::default();
        paint.set_color(self.color);
        paint.set_anti_alias(true);
        if has_any_corner_radius(style) {
            canvas.draw_rrect(RRect::new_rect_radii(bounds, &corner_radii(style)), &paint);
        } else {
            canvas.draw_rect(bounds, &paint);
        }
    }
}

const AVATAR_COLOR_PENDING: Color = Color::from_rgb(116, 124, 140);

pub fn banner_color(
    user_id: Id<UserMarker>,
    discriminator: &str,
    accent: Option<i64>,
    avatar_hash: Option<&str>,
    images: &ImageCache,
    backend: &BackendConfig,
) -> Color {
    if let Some(accent) = accent {
        return accent_to_color(accent);
    }
    let Some(hash) = avatar_hash else {
        return avatar_color::default_avatar_primary_color(
            ui::default_avatar_color(user_id, discriminator),
        );
    };
    let url = backend.avatar_url_sized(user_id, hash, 80);
    match images.get_or_decode_image(&url) {
        Some(image) => avatar_color::primary_color(&url, &image),
        None => AVATAR_COLOR_PENDING,
    }
}

pub fn badges_row(
    profile: &UserProfile,
    images: &ImageCache,
    backend: &BackendConfig,
) -> Option<Node> {
    let icons: Vec<&str> = profile
        .badges
        .iter()
        .filter_map(|b| b.icon.as_deref())
        .collect();
    if icons.is_empty() {
        return None;
    }
    let mut row = div().class(profile_popout::BADGES_ROW);
    for hash in icons {
        if let Some(image) = images.get_or_decode_image(&backend.badge_icon_url(hash)) {
            row = row.child(
                Node::new("div", ImageWidget::new(Some(image)))
                    .class(profile_popout::BADGE_ICON),
            );
        }
    }
    Some(row)
}

pub fn accent_to_color(accent: i64) -> Color {
    let accent = accent.clamp(0, 0xff_ffff) as u32;
    Color::from_argb(
        255,
        ((accent >> 16) & 0xff) as u8,
        ((accent >> 8) & 0xff) as u8,
        (accent & 0xff) as u8,
    )
}

pub fn separator_dot(color: Color) -> Node {
    Node::new("div", ColorBoxWidget { color })
        .class(profile_popout::SEPARATOR_DOT)
}

fn loading_skeleton() -> Node {
    let bar =
        |width: &str, height: u32| skeleton().style(format!("width: {width}; height: {height}px"));
    let header = StackProps::vertical()
        .gap(8)
        .padding(Padding::x(16).with_top(12).with_bottom(16))
        .build()
        .children([bar("55%", 18), bar("35%", 12)]);
    let tabs = StackProps::horizontal()
        .gap(16)
        .padding(Padding::x(16).with_bottom(12))
        .build()
        .class(cls::SKELETON_TABS)
        .children([bar("52px", 16), bar("72px", 16), bar("48px", 16)]);
    let content = StackProps::vertical()
        .gap(10)
        .padding(Padding::all(16))
        .build()
        .children([
            bar("90%", 12),
            bar("75%", 12),
            bar("85%", 12),
            bar("60%", 12),
        ]);
    StackProps::vertical().gap(0).build().children([
        skeleton().class(cls::SKELETON_BANNER),
        div()
            .class(cls::SKELETON_AVATAR_RING)
            .child(skeleton().class(cls::SKELETON_AVATAR)),
        header,
        tabs,
        content,
    ])
}

#[allow(clippy::too_many_arguments)]
pub fn build(
    user_id: Id<UserMarker>,
    guild_id: Option<Id<GuildMarker>>,
    viewer: Option<Id<UserMarker>>,
    status: PresenceStatus,
    custom_status: Option<&CustomStatus>,
    anchor: Anchor,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    queries: &QueryClient,
    http: Option<&HttpClient>,
    app: &state::AppState,
    state: &ProfilePopoutState,
) -> Node {
    let users = &app.users;
    let profile_key = profile_query_key(user_id, guild_id);
    let profile = load_profile(
        queries,
        http,
        users,
        user_id,
        guild_id,
        ProfileSurface::Popout,
    );

    let hover_overlays = state.overlays.clone();
    let panel = PopoverProps::default()
        .anchor(anchor)
        .on_hover_change(move |inside| hover_overlays.set_popout_hovered(inside))
        .dom_id(PANEL_DOM_ID)
        .build()
        .class(profile_popout::PANEL);

    let Some(profile) = profile else {
        return panel.child(loading_skeleton());
    };

    let all_roles = guild_id
        .map(|guild_id| Arc::new(author_style::guild_roles(app, guild_id)));

    let banner_image =
        profile.user.banner.as_deref().and_then(|hash| {
            images.get_or_decode_image(&backend.banner_url(profile.user.id, hash))
        });
    let banner_fallback = banner_color(
        profile.user.id,
        &profile.user.discriminator,
        profile.user.accent_color,
        profile.user.avatar.as_deref(),
        images,
        backend,
    );
    let avatar_image =
        profile.user.avatar.as_deref().and_then(|hash| {
            images.get_or_decode_image(&backend.avatar_url(profile.user.id, hash))
        });
    let avatar_decoration = profile.user.avatar_decoration_asset().and_then(|asset| {
        images.get_or_decode_image(&backend.avatar_decoration_url(asset, 80))
    });
    let colors = profile_theme::resolve(&profile);

    let open_full_tab = {
        let state = state.clone();
        move |focus_note: bool, tab: FullProfileTab| {
            close_popouts(&state);
            state.overlays.open_full_profile(FullProfileTarget {
                user_id,
                guild_id,
                focus_note,
                tab,
                server_guild_id: None,
            });
        }
    };
    let open_full = {
        let open = open_full_tab.clone();
        move |focus_note: bool| open(focus_note, FullProfileTab::First)
    };

    let open_full_for_note = open_full.clone();
    let note_button = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![
            profile_popout::NOTE_BTN.to_string(),
        ])
        .on_click(move || open_full_for_note(true))
        .build()
        .child(
            icon(PAPER_PLUS, TEXT_SECONDARY).class(profile_popout::NOTE_ICON),
        );
    let note_slot = StackProps::horizontal()
        .gap(0)
        .full_width(false)
        .build()
        .bind_class(
            state.overlays.popout_hovered.clone(),
            profile_popout::NOTE_SHOWN,
            profile_popout::NOTE_HIDDEN,
        )
        .child(note_button);

    let mut after_username: Vec<Node> = Vec::new();
    let mut after_bio: Vec<Node> = Vec::new();

    let mutual_guild_count = profile.mutual_guilds.as_ref().map(|g| g.len()).unwrap_or(0);
    let mutual_friends = profile.mutual_friends.as_deref().unwrap_or(&[]);
    let mutual_friend_count = profile
        .mutual_friends_count
        .unwrap_or(mutual_friends.len() as u32);

    if viewer != Some(user_id) && (mutual_guild_count > 0 || mutual_friend_count > 0) {
        let mut mutuals_row = StackProps::horizontal()
            .align(Align::Center)
            .gap(6)
            .full_width(false)
            .build();
        if mutual_friend_count > 0 {
            let mut friend_section = StackProps::horizontal()
                .align(Align::Center)
                .gap(4)
                .full_width(false)
                .build();
            if !mutual_friends.is_empty() {
                let mut avatars = StackProps::horizontal().gap(2).full_width(false).build();
                for friend in mutual_friends.iter().take(3) {
                    let image = friend.avatar.as_deref().and_then(|hash| {
                        images.get_or_decode_image(&backend.avatar_url(friend.id, hash))
                    });
                    let mut props = ui::user_avatar_props(
                        friend.id,
                        friend.discriminator.as_deref().unwrap_or("0"),
                        fonts,
                    );
                    if let Some(image) = image {
                        props = props.image(image);
                    }
                    avatars = avatars.child(
                        props
                            .build()
                            .class(profile_popout::MUTUAL_AVATAR),
                    );
                }
                friend_section = friend_section.child(avatars);
            }
            let label = requiem_i18n::t!("mutual-friends", count = mutual_friend_count);
            friend_section = friend_section.child(
                TextProps::styled(label, TextVariant::TEXT_XS_NORMAL)
                    .color(TEXT_SECONDARY)
                    .build()
                    .class(profile_popout::MUTUAL_TEXT),
            );
            let open = open_full_tab.clone();
            mutuals_row = mutuals_row.child(
                ButtonProps::default()
                    .variant(ButtonVariant::Bare)
                    .extra_classes(vec![
                        profile_popout::MUTUAL_LINK.to_string(),
                    ])
                    .on_click(move || open(false, FullProfileTab::MutualFriends))
                    .build()
                    .child(friend_section),
            );
        }
        if mutual_guild_count > 0 {
            if mutual_friend_count > 0 {
                mutuals_row = mutuals_row.child(separator_dot(TEXT_SECONDARY));
            }
            let label = requiem_i18n::t!("mutual-servers", count = mutual_guild_count);
            let open = open_full_tab.clone();
            let section = StackProps::horizontal()
                .align(Align::Center)
                .gap(4)
                .full_width(false)
                .build()
                .child(
                    TextProps::styled(label, TextVariant::TEXT_XS_NORMAL)
                        .color(TEXT_SECONDARY)
                        .build()
                        .class(profile_popout::MUTUAL_TEXT),
                );
            mutuals_row = mutuals_row.child(
                ButtonProps::default()
                    .variant(ButtonVariant::Bare)
                    .extra_classes(vec![
                        profile_popout::MUTUAL_LINK.to_string(),
                    ])
                    .on_click(move || open(false, FullProfileTab::MutualServers))
                    .build()
                    .child(section),
            );
        }
        after_username.push(mutuals_row);
    }

    if let (Some(guild_id), Some(guild_member), Some(all_roles)) =
        (guild_id, profile.guild_member.as_ref(), all_roles.as_ref())
    {
        after_bio.push(build_role_section(
            user_id,
            guild_id,
            &guild_member.roles,
            all_roles,
            queries,
            http,
            &state.role_picker_open,
            &state.overlays.role_search,
            &profile_key,
            images,
            backend,
            app.guilds
                .find(guild_id)
                .as_ref()
                .map(|guild| {
                    app.permissions
                        .can_assign_any_role(guild, all_roles, &guild_member.roles)
                })
                .unwrap_or(false),
        ));
    }

    let footer_display_name = profile
        .user
        .global_name
        .clone()
        .unwrap_or_else(|| profile.user.username.clone());

    let is_self = viewer == Some(user_id);

    let footer = if is_self {
        let on_edit = state.overlays.on_edit_profile.clone();
        let close_state = state.clone();
        let button = ButtonProps::default()
            .size(ButtonSize::Small)
            .full_width(true)
            .label(requiem_i18n::t!("profile-popout-edit-profile").to_string())
            .icon(PENCIL)
            .on_click(move || {
                close_popouts(&close_state);
                on_edit();
            })
            .build();
        div()
            .class(profile_popout::FOOTER)
            .child(button)
    } else {
        build_footer(user_id, &footer_display_name, images, queries, http, state)
    };

    let is_self_card = is_self;
    let self_actions = is_self_card.then(|| custom_status::BubbleActions {
        on_open: {
            let (open, current, close_state) = (
                state.overlays.on_open_custom_status.clone(),
                custom_status.cloned(),
                state.clone(),
            );
            Rc::new(move || {
                close_popouts(&close_state);
                open(current.clone());
            })
        },
        on_clear: state.overlays.on_clear_custom_status.clone(),
    });
    let prompt = state.overlays.prompt.peek();

    let relationship = relationship_kind(&app.relationships, user_id);
    let display_for_menu = profile
        .user
        .global_name
        .clone()
        .unwrap_or_else(|| profile.user.username.clone());
    let header_extra = if is_self {
        Vec::new()
    } else {
        vec![banner_buttons(
            user_id,
            !profile.user.bot && relationship.is_none(),
            relationship == Some(relationships::FRIEND),
            queries,
            http,
            state,
            location::PROFILE_POPOUT,
        )]
    };
    let display_name = profile
        .user
        .global_name
        .clone()
        .unwrap_or_else(|| profile.user.username.clone());
    let pronouns = profile
        .user_profile
        .as_ref()
        .map(|meta| meta.pronouns.as_str())
        .unwrap_or("");
    let view = profile_card::ProfileCardView {
        user_id: profile.user.id,
        discriminator: &profile.user.discriminator,
        username: &profile.user.username,
        display_name: &display_name,
        pronouns,
        bio: &profile.user.bio,
        avatar: avatar_image,
        avatar_decoration,
        banner_image,
        banner_color: banner_fallback,
        theme: colors.theme,
        name_color: colors.name_color,
        status: Some(status),
        bot: profile.user.bot,
        badges_from: Some(&profile),
    };
    let header_layers = profile_decorations::layers(queries, http, &profile, images, backend);
    let open_for_card = open_full.clone();
    let parts = profile_card::CardParts {
        open_profile: Some(Rc::new(move || open_for_card(false))),
        header_layers,
        background_layers: Vec::new(),
        name_suffix: vec![note_slot],
        header_extra,
        after_username,
        after_bio,
        roomy_body: false,
        notice: profile.partial.then(partial_notice),
    };
    let card_status = profile_card::CardStatus {
        status: custom_status,
        actions: self_actions.as_ref(),
        prompt,
    };
    let mut panel = profile_card::profile_card(
        view,
        card_status,
        parts,
        Some(footer),
        CardContainer::Panel(panel),
        fonts,
        images,
        backend,
    );

    if !is_self && state.overlays.friend_menu_open.get() {
        panel = panel.children(friend_menu(
            user_id,
            queries,
            http,
            state,
            location::PROFILE_POPOUT,
        ));
    }
    if !is_self && state.overlays.more_menu_open.get() {
        let close = state.overlays.more_menu_open.clone();
        panel = panel.child(popover_backdrop(move || close.set(false)));
        panel = panel.child(
            more_menu(
                user_id,
                &display_for_menu,
                relationship,
                open_full,
                queries,
                http,
                state,
            )
            .class(profile_popout::MORE_MENU_POS),
        );
    }
    panel
}

pub fn banner_buttons(
    user_id: Id<UserMarker>,
    show_add_friend: bool,
    is_friend: bool,
    queries: &QueryClient,
    http: Option<&HttpClient>,
    state: &ProfilePopoutState,
    location: &'static str,
) -> Node {
    let tip = |label: &str, anchor: Node| {
        TooltipProps::default()
            .text(label.to_string())
            .build(anchor)
    };
    let mut row = div().class(cls::BANNER_BUTTONS);
    if show_add_friend {
        let (queries, http) = (queries.clone(), http.cloned());
        let add = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::BANNER_BTN.to_string()])
            .on_click(move || {
                let Some(http) = http.clone() else { return };
                queries.mutate(Vec::new(), move || async move {
                    relationships::send_friend_request(
                        &http, user_id, location,
                    )
                    .await
                });
            })
            .build()
            .child(icon(USER_PLUS, Color::WHITE).class(cls::BANNER_BTN_ICON));
        row = row.child(tip(&requiem_i18n::t!("profile-popout-add-friend"), add));
    }
    if is_friend {
        let friend_open = state.overlays.friend_menu_open.clone();
        let friend = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::BANNER_BTN.to_string()])
            .on_click(move || friend_open.set(!friend_open.peek()))
            .build()
            .child(icon(USER_CHECK, Color::WHITE).class(cls::BANNER_BTN_ICON));
        row = row.child(tip(&requiem_i18n::t!("profile-popout-friend"), friend));
    }
    let menu_open = state.overlays.more_menu_open.clone();
    let more = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::BANNER_BTN.to_string()])
        .on_click(move || menu_open.set(!menu_open.peek()))
        .build()
        .child(icon(MORE_HORIZONTAL, Color::WHITE).class(cls::BANNER_BTN_ICON));
    row.child(tip(&requiem_i18n::t!("profile-popout-more"), more))
}

pub fn friend_menu(
    user_id: Id<UserMarker>,
    queries: &QueryClient,
    http: Option<&HttpClient>,
    state: &ProfilePopoutState,
    location: &'static str,
) -> [Node; 2] {
    let close = state.overlays.friend_menu_open.clone();
    let backdrop_close = close.clone();
    let (queries, http) = (queries.clone(), http.cloned());
    let remove = MenuItem::action(requiem_i18n::t!("home-remove-friend"), move || {
        close.set(false);
        let Some(http) = http.clone() else { return };
        queries.mutate(Vec::new(), move || async move {
            relationships::remove(&http, user_id, location).await
        });
    });
    [
        popover_backdrop(move || backdrop_close.set(false)),
        menu(vec![remove]).class(profile_popout::FRIEND_MENU_POS),
    ]
}

#[allow(clippy::too_many_arguments)]
pub fn more_menu(
    user_id: Id<UserMarker>,
    display_name: &str,
    relationship: Option<u8>,
    open_full: impl Fn(bool) + 'static,
    queries: &QueryClient,
    http: Option<&HttpClient>,
    state: &ProfilePopoutState,
) -> Node {
    let close = state.overlays.more_menu_open.clone();
    let mut items = vec![
        MenuItem::action(
            requiem_i18n::t!("profile-popout-view-full-profile"),
            move || open_full(false),
        ),
        MenuItem::Separator,
    ];

    let (q, h, c) = (queries.clone(), http.cloned(), close.clone());
    items.push(MenuItem::action(
        requiem_i18n::t!("profile-popout-ignore"),
        move || {
            c.set(false);
            let Some(http) = h.clone() else { return };
            q.mutate(Vec::new(), move || async move {
                relationships::ignore(
                    &http,
                    user_id,
                    location::PROFILE_OVERFLOW_MENU,
                )
                .await
            });
        },
    ));

    if relationship == Some(BLOCKED) {
        let (q, h, c) = (queries.clone(), http.cloned(), close.clone());
        items.push(MenuItem::danger(
            requiem_i18n::t!("profile-popout-unblock"),
            move || {
                c.set(false);
                let Some(http) = h.clone() else { return };
                q.mutate(Vec::new(), move || async move {
                    relationships::remove(
                        &http,
                        user_id,
                        location::PROFILE_OVERFLOW_MENU,
                    )
                    .await
                });
            },
        ));
    } else {
        let (overlays, name) = (state.overlays.clone(), display_name.to_string());
        items.push(MenuItem::danger(
            requiem_i18n::t!("profile-popout-block"),
            move || {
                close.set(false);
                overlays.confirm_block(
                    user_id,
                    name.clone(),
                    location::PROFILE_OVERFLOW_MENU,
                );
            },
        ));
    }
    menu(items)
}

#[allow(clippy::too_many_arguments)]
pub fn build_webhook(
    author: &models::User,
    anchor: Anchor,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    state: &ProfilePopoutState,
) -> Node {
    let hover_overlays = state.overlays.clone();
    let panel = PopoverProps::default()
        .anchor(anchor)
        .on_hover_change(move |inside| hover_overlays.set_popout_hovered(inside))
        .dom_id(PANEL_DOM_ID)
        .build()
        .class(cls::PANEL);

    let avatar = author
        .avatar
        .as_deref()
        .and_then(|hash| images.get_or_decode_image(&backend.avatar_url(author.id, hash)));
    let avatar_decoration = author.avatar_decoration_asset().and_then(|asset| {
        images.get_or_decode_image(&backend.avatar_decoration_url(asset, 80))
    });
    let banner = banner_color(
        author.id,
        &author.discriminator,
        None,
        author.avatar.as_deref(),
        images,
        backend,
    );
    let display_name = author
        .global_name
        .clone()
        .unwrap_or_else(|| author.username.clone());
    let view = profile_card::ProfileCardView {
        user_id: author.id,
        discriminator: &author.discriminator,
        username: &author.username,
        display_name: &display_name,
        pronouns: "",
        bio: "",
        avatar,
        avatar_decoration,
        banner_image: None,
        banner_color: banner,
        theme: None,
        name_color: None,
        status: None,
        bot: true,
        badges_from: None,
    };

    let menu_open = state.overlays.more_menu_open.clone();
    let more = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::BANNER_BTN.to_string()])
        .on_click(move || menu_open.set(!menu_open.peek()))
        .build()
        .child(icon(MORE_HORIZONTAL, Color::WHITE).class(cls::BANNER_BTN_ICON));
    let buttons = div().class(cls::BANNER_BUTTONS).child(
        TooltipProps::default()
            .text(requiem_i18n::t!("profile-popout-more").to_string())
            .build(more),
    );

    let developer = prefs::developer_mode();
    let parts = profile_card::CardParts {
        header_extra: if developer { vec![buttons] } else { Vec::new() },
        ..Default::default()
    };
    let card_status = profile_card::CardStatus {
        status: None,
        actions: None,
        prompt: "",
    };
    let mut panel = profile_card::profile_card(
        view,
        card_status,
        parts,
        None,
        CardContainer::Panel(panel),
        fonts,
        images,
        backend,
    );

    if developer && state.overlays.more_menu_open.get() {
        let close = state.overlays.more_menu_open.clone();
        let (backdrop_close, id) = (close.clone(), author.id.to_string());
        let copy = MenuItem::action(requiem_i18n::t!("context-menus-copy-user-id"), move || {
            close.set(false);
            context_menu::copy_with_toast(
                id.clone(),
                &requiem_i18n::t!("context-menus-id-copied"),
            );
        });
        panel = panel.child(popover_backdrop(move || backdrop_close.set(false)));
        panel = panel.child(
            menu(vec![copy.with_leading(
                icon(ID, TEXT_SECONDARY).style("width: 20px; height: 20px"),
            )])
            .class(cls::MORE_MENU_POS),
        );
    }
    panel
}

#[allow(clippy::too_many_arguments)]
pub fn build_role_section(
    user_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>,
    member_role_ids: &[Id<RoleMarker>],
    all_roles: &[Role],
    queries: &QueryClient,
    http: Option<&HttpClient>,
    role_picker_open: &Signal<bool>,
    role_search: &Signal<String>,
    profile_key: &str,
    images: &ImageCache,
    backend: &BackendConfig,
    can_assign: bool,
) -> Node {
    let mut container = div().class(profile_popout::ROLE_LIST);

    let assigned: Vec<&Role> = all_roles
        .iter()
        .filter(|r| member_role_ids.contains(&r.id))
        .collect();

    let assignable: Vec<&Role> = if can_assign {
        all_roles
            .iter()
            .filter(|r| {
                !member_role_ids.contains(&r.id) && r.id.get() != guild_id.get() && !r.managed
            })
            .collect()
    } else {
        Vec::new()
    };

    for role in &assigned {
        let circle = if role.color == 0 {
            DEFAULT_ROLE_COLOR
        } else {
            accent_to_color(role.color)
        };
        let dot = Node::new("div", ColorBoxWidget { color: circle })
            .class(profile_popout::ROLE_CHIP_DOT);
        let name = TextProps::styled(role.name.clone(), TextVariant::TEXT_XS_NORMAL)
            .color(TEXT_DEFAULT)
            .build()
            .class(profile_popout::ROLE_CHIP_NAME);
        let (role_id, http_owned, queries_owned, profile_key_owned) = (
            role.id,
            http.cloned(),
            queries.clone(),
            profile_key.to_string(),
        );
        let remove = if can_assign {
            Some(
                ButtonProps::default()
                    .variant(ButtonVariant::Bare)
                    .extra_classes(vec![
                        profile_popout::ROLE_CHIP_REMOVE.to_string(),
                    ])
                    .on_click(move || {
                        let Some(http) = http_owned.clone() else {
                            return;
                        };
                        queries_owned.mutate(vec![profile_key_owned.clone()], move || async move {
                            users::remove_member_role(
                                &http, guild_id, user_id, role_id,
                            )
                            .await
                        });
                    })
                    .build()
                    .child(
                        icon(X_SMALL, TEXT_SECONDARY)
                            .class(profile_popout::ROLE_CHIP_REMOVE_ICON),
                    ),
            )
        } else {
            None
        };
        let mut chip = div()
            .class(profile_popout::ROLE_CHIP)
            .child(dot)
            .child(name);
        if remove.is_none() {
            // The remove button is the chip's right edge; without it the name needs its own.
            chip = chip.class(profile_popout::ROLE_CHIP_PLAIN);
        }
        if let Some(icon) = author_style::role_icon_of(role).and_then(|icon| {
            author_style::role_icon_node(
                &icon,
                classes_author_style::ROLE_ICON_CHIP,
                images,
                backend,
            )
        }) {
            chip = chip.child(icon);
        }
        container = container.child(match remove {
            Some(remove) => chip.child(remove),
            None => chip,
        });
    }

    if assignable.is_empty() {
        return container;
    }

    let picker_open = role_picker_open.get();
    let toggle_signal = role_picker_open.clone();
    let search_reset = role_search.clone();
    let add_button = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![
            profile_popout::ADD_ROLE_BTN.to_string(),
        ])
        .on_click(move || {
            if !toggle_signal.peek() && !search_reset.peek().is_empty() {
                search_reset.set(String::new());
            }
            toggle_signal.set(!toggle_signal.peek());
        })
        .build()
        .child(
            icon(PLUS_MEDIUM, TEXT_SECONDARY)
                .class(profile_popout::ADD_ROLE_ICON)
                .class(profile_popout::ADD_ROLE_COLOR),
        );

    let add_button = if assigned.is_empty() {
        add_button.child(
            TextProps::styled(
                requiem_i18n::t!("profile-popout-add-role").to_string(),
                TextVariant::TEXT_XS_NORMAL,
            )
            .color(TEXT_SECONDARY)
            .build()
            .class(profile_popout::ADD_ROLE_COLOR),
        )
    } else {
        add_button.class(profile_popout::ADD_ROLE_BTN_ICON_ONLY)
    };

    let mut add_wrap = div()
        .class(profile_popout::ADD_ROLE_WRAP)
        .child(add_button);
    if picker_open {
        add_wrap = add_wrap.child(role_picker(
            user_id,
            guild_id,
            &assignable,
            queries,
            http,
            role_picker_open,
            role_search,
            profile_key,
        ));
    }
    container.child(add_wrap)
}

#[allow(clippy::too_many_arguments)]
fn role_picker(
    user_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>,
    assignable: &[&Role],
    queries: &QueryClient,
    http: Option<&HttpClient>,
    role_picker_open: &Signal<bool>,
    role_search: &Signal<String>,
    profile_key: &str,
) -> Node {
    let query = role_search.peek().to_lowercase();
    let matches = |name: &str, query: &str| name.to_lowercase().contains(query);

    let names: Vec<String> = assignable.iter().map(|r| r.name.clone()).collect();
    let visible: Vec<Signal<bool>> = names
        .iter()
        .map(|n| signal(matches(n, &query)))
        .collect();
    let any_visible = signal(visible.iter().any(|v| v.peek()));

    let field = {
        let (search, visible, any_visible, names) = (
            role_search.clone(),
            visible.clone(),
            any_visible.clone(),
            names.clone(),
        );
        TextInputProps::new()
            .text(role_search.peek())
            .placeholder(requiem_i18n::t!("profile-popout-role").to_string())
            .autofocus(true)
            .on_change(move |text: &str| {
                let query = text.to_lowercase();
                let mut any = false;
                for (name, shown) in names.iter().zip(&visible) {
                    let show = matches(name, &query);
                    any |= show;
                    if shown.peek() != show {
                        shown.set(show);
                    }
                }
                if any_visible.peek() != any {
                    any_visible.set(any);
                }
                search.set(text.to_string());
            })
            .build()
            .class(cls::ROLE_PICKER_FIELD)
    };

    let mut list = StackProps::vertical()
        .gap(0)
        .full_width(false)
        .padding(Padding::all(4))
        .build();
    for (role, shown) in assignable.iter().zip(visible) {
        let circle = if role.color == 0 {
            DEFAULT_ROLE
        } else {
            accent_to_color(role.color)
        };
        let dot = Node::new("div", ColorBoxWidget { color: circle }).class(cls::ROLE_PICKER_CIRCLE);
        let name = TextProps::styled(role.name.clone(), TextVariant::TEXT_MD_NORMAL)
            .color(TEXT_STRONG)
            .build()
            .class(cls::ROLE_PICKER_NAME);
        let (role_id, http_owned, queries_owned, profile_key_owned, close_signal) = (
            role.id,
            http.cloned(),
            queries.clone(),
            profile_key.to_string(),
            role_picker_open.clone(),
        );
        let row = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::ROLE_PICKER_ITEM.to_string()])
            .on_click(move || {
                close_signal.set(false);
                let Some(http) = http_owned.clone() else {
                    return;
                };
                queries_owned.mutate(vec![profile_key_owned.clone()], move || async move {
                    users::add_member_role(
                        &http, guild_id, user_id, role_id,
                    )
                    .await
                });
            })
            .build()
            .children([dot, name]);
        list = list.child(
            StackProps::vertical()
                .gap(0)
                .full_width(false)
                .build()
                .bind_class(shown, cls::ROLE_PICKER_SHOWN, cls::ROLE_PICKER_HIDDEN)
                .child(row),
        );
    }
    let empty = div()
        .class(cls::ROLE_PICKER_EMPTY)
        .bind_class(any_visible, cls::ROLE_PICKER_HIDDEN, cls::ROLE_PICKER_SHOWN)
        .child(
            TextProps::styled(
                requiem_i18n::t!("profile-popout-no-items-to-show").to_string(),
                TextVariant::TEXT_MD_MEDIUM,
            )
            .color(TEXT_DEFAULT)
            .build(),
        );
    list = list.child(empty);

    Node::new(
        "div",
        PopoverPanelWidget::new(),
    )
    .class(cls::ROLE_PICKER)
    .child(field)
    .child(list)
}

fn build_footer(
    user_id: Id<UserMarker>,
    display_name: &str,
    images: &ImageCache,
    queries: &QueryClient,
    http: Option<&HttpClient>,
    state: &ProfilePopoutState,
) -> Node {
    let placeholder = requiem_i18n::t!("profile-popout-message-placeholder", name = display_name);

    let draft_read = state.draft.peek();
    let draft_for_change = state.draft.clone();
    let draft_for_submit = state.draft.clone();
    let queries_for_submit = queries.clone();
    let http_for_submit = http.cloned();

    let input = message_list::chat_input(TextInputProps::default())
        .text(draft_read)
        .placeholder(placeholder)
        .font_size(16.0_f32)
        .color(TEXT_DEFAULT)
        .placeholder_font_size(14.0_f32)
        .placeholder_color(TEXT_MUTED)
        .insets((15.0_f32, 0.0_f32))
        .on_change(move |text: &str| draft_for_change.set(text.to_string()))
        .on_submit(move |content: String| {
            draft_for_submit.set(String::new());
            let Some(http) = http_for_submit.clone() else {
                return;
            };
            if content.trim().is_empty() {
                return;
            }

            let known_dm = (context_menu::actions().dm_channel)(user_id);
            queries_for_submit.mutate(Vec::new(), move || async move {
                let channel_id = match known_dm {
                    Some(id) => id,
                    None => {
                        channels::create_dm(
                            &http,
                            user_id,
                            Some(location::PROFILE_POPOUT),
                        )
                        .await?
                        .id
                    }
                };
                messages::send_reply_from(
                    &http,
                    channel_id,
                    &content,
                    None,
                    send_location::USER_PROFILE,
                )
                .await?;
                Ok::<(), RestError>(())
            });
        })
        .build()
        .class(profile_popout::MESSAGE_INPUT);

    let emoji_open_toggle = state.emoji_open.clone();
    let emoji_hovered_reset = state.emoji_hovered.clone();
    let emoji_btn = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![
            profile_popout::EMOJI_BTN.to_string(),
        ])
        .on_click(move || {
            let opening = !emoji_open_toggle.peek();

            if opening && emoji_hovered_reset.peek().is_some() {
                emoji_hovered_reset.set(None);
            }
            emoji_open_toggle.set(opening);
        })
        .build()
        .child(
            Node::new(
                "span",
                EmojiButtonIconWidget::new(),
            )
            .class(profile_popout::EMOJI_BTN_ICON),
        );

    let input_row = div()
        .class(profile_popout::MESSAGE_INPUT_ROW)
        .children([input, emoji_btn]);
    let mut footer = div()
        .class(profile_popout::FOOTER)
        .child(input_row);

    if state.emoji_open.get() {
        let picker_state = EmojiPickerState {
            open: state.emoji_open.clone(),
            tab: state.emoji_tab.clone(),
            signals: EmojiPickerSignals {
                search: state.emoji_search.clone(),
                scroll_offset: state.emoji_scroll_offset.clone(),
                category_jump: state.emoji_category_jump.clone(),
                hovered: state.emoji_hovered.clone(),
                collapsed: state.emoji_collapsed.clone(),
            },

            gif: GifPickerState::inert(),
            on_insert: Rc::new(|_| {}),
            on_insert_at: Rc::new(|_, _| {}),
        };
        let insert_draft = state.draft.clone();
        let close_after_pick = state.emoji_open.clone();
        let on_pick_emoji = Rc::new(move |glyph: String, keep_open: bool| {
            insert_draft.update(|d| d.push_str(&glyph));
            if !keep_open {
                close_after_pick.set(false);
            }
        });
        let on_pick_gif = Rc::new(|_url: String| {});
        let picker_panel =
            emoji_picker::build(&picker_state, images, on_pick_emoji, on_pick_gif, None)
                .dom_id(EMOJI_PANEL_DOM_ID)
                .class(profile_popout::EMOJI_PICKER_POS);
        footer = footer.child(picker_panel);
    }

    footer
}

#[allow(dead_code)]
fn member_since(user_id: Id<UserMarker>) -> String {
    let millis = user_id.created_at_millis();
    let dt = Utc
        .timestamp_millis_opt(millis)
        .single()
        .unwrap_or_else(Utc::now);
    requiem_i18n::t!(
        "profile-popout-member-since",
        date = time::date_medium(&dt)
    )
}

#[allow(clippy::too_many_arguments)]
pub fn build_member_layer(
    state: &state::AppState,
    popout: &ProfilePopoutState,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    queries: &QueryClient,
    http: Option<&HttpClient>,
) -> Node {
    let layer = div().class(cls::MEMBER_LAYER);
    let Some(target) = popout.overlays.member_target.get() else {
        return layer;
    };

    let anchor = Anchor::new(target.row)
        .position(PopoverSide::Left)
        .spacing(16.0);
    let viewer = state
        .users
        .current
        .with_untracked(|u| u.as_ref().map(|u| u.id));
    let (status, custom) = if viewer == Some(target.user_id) {
        (
            state.presence.status.get().indicator(),
            state.presence.custom_status.get(),
        )
    } else {
        (
            target.status,
            state
                .members
                .custom_status_of(target.guild_id, target.user_id),
        )
    };
    let close = {
        let popout = popout.clone();
        Rc::new(move || close_popouts(&popout)) as Rc<dyn Fn()>
    };
    let panel = build(
        target.user_id,
        Some(target.guild_id),
        viewer,
        status,
        custom.as_ref(),
        anchor,
        fonts,
        images,
        backend,
        queries,
        http,
        state,
        popout,
    )
    .focus_scope();
    layer
        .class(cls::MEMBER_LAYER_OPEN)
        .child(popover_backdrop(move || close()))
        .child(panel)
}

pub fn clickable(
    child: Node,
    message_id: Id<MessageMarker>,
    state: &ProfilePopoutState,
    extra_class: Option<&'static str>,
) -> Node {
    popout_button(child, message_id, None, state, extra_class)
}

pub fn clickable_user(
    child: Node,
    message_id: Id<MessageMarker>,
    user_id: Id<UserMarker>,
    state: &ProfilePopoutState,
    extra_class: Option<&'static str>,
) -> Node {
    popout_button(child, message_id, Some(user_id), state, extra_class)
}

fn popout_button(
    child: Node,
    message_id: Id<MessageMarker>,
    user: Option<Id<UserMarker>>,
    state: &ProfilePopoutState,
    extra_class: Option<&'static str>,
) -> Node {
    let state = state.clone();
    let mut props = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .on_click_at(move |rect| toggle_for(message_id, user, rect, &state));
    if let Some(extra_class) = extra_class {
        props = props.extra_classes(vec![extra_class.to_string()]);
    }
    props.build().child(child)
}

pub fn toggle_for(
    message_id: Id<MessageMarker>,
    user: Option<Id<UserMarker>>,
    anchor: AbsoluteRect,
    state: &ProfilePopoutState,
) {
    let (open, overlays) = (&state.open, &state.overlays);
    let same = open.peek() == Some(message_id) && overlays.popout_user.peek() == user;
    let next = if same { None } else { Some(message_id) };
    if state.role_picker_open.peek() {
        state.role_picker_open.set(false);
    }

    if overlays.member_target.peek().is_some() {
        overlays.member_target.set(None);
    }
    overlays.more_menu_open.set(false);
    if next.is_some() {
        overlays
            .prompt
            .set(custom_status::random_prompt());
    }
    overlays
        .popout_user
        .set(if next.is_some() { user } else { None });
    if next.is_some() {
        overlays.popout_anchor.set(anchor);
    }
    open.set(next);
}

#[cfg(test)]
mod tests {
    use requiem_ui::layout;
    use requiem_ui::paint as requiem_ui_paint;
    use requiem_ui::style::cascade;
    use requiem_ui::style::parser;
    use requiem_ui::widget;
    use skia_safe::surfaces;
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::thread;
    use std::time::Duration;

    use requiem_ui::style::ComputedStyle;
    use requiem_ui::widget::WidgetTree;
    use requiem_ui::widget::primitives::BoxWidget;
    use taffy::{AvailableSpace, Size};

    use super::*;

    fn mounted() -> (WidgetTree, widget::WidgetId) {
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let panel = tree.mount(tree.root, div().style("width: 300px; background-color: var(--background-surface-high); border-radius: var(--radius-sm)"));
        let skeleton = tree.mount(panel, loading_skeleton());
        let theme = parser::parse_stylesheet(&format!(
            "{}
{}",
            requiem_ui::BUNDLED_CSS,
            ui::THEME_CSS
        ));
        cascade::apply_stylesheet(&mut tree, &theme);
        layout::compute_layout(
            &mut tree,
            Size {
                width: AvailableSpace::Definite(300.0),
                height: AvailableSpace::Definite(600.0),
            },
        );
        (tree, skeleton)
    }

    #[test]
    fn the_loading_skeleton_has_the_clients_proportions() {
        let (tree, skeleton) = mounted();
        let rects = requiem_ui_paint::all_absolute_rects(&tree);
        let [banner, ring, header, tabs, content] = tree.nodes[skeleton].children[..] else {
            panic!("five sections")
        };
        let bars = |id| {
            tree.nodes[id]
                .children
                .iter()
                .map(|c| (rects[c].width, rects[c].height))
                .collect::<Vec<_>>()
        };
        let near = |a: f32, b: f32| (a - b).abs() < 0.6;

        assert_eq!(
            (rects[&banner].width, rects[&banner].height),
            (300.0, 105.0)
        );
        assert_eq!((rects[&ring].width, rects[&ring].height), (80.0, 80.0));
        assert_eq!(
            (
                rects[&ring].x - rects[&banner].x,
                rects[&ring].y - rects[&banner].y
            ),
            (16.0, 105.0 - 40.0)
        );

        let header_bars = bars(header);
        assert!(
            near(header_bars[0].0, 0.55 * 268.0) && header_bars[0].1 == 18.0,
            "{header_bars:?}"
        );
        assert!(
            near(header_bars[1].0, 0.35 * 268.0) && header_bars[1].1 == 12.0,
            "{header_bars:?}"
        );

        assert_eq!(bars(tabs), [(52.0, 16.0), (72.0, 16.0), (48.0, 16.0)]);
        let tab_bars: Vec<_> = tree.nodes[tabs]
            .children
            .iter()
            .map(|c| rects[c].x)
            .collect();
        assert_eq!(tab_bars[1] - tab_bars[0], 52.0 + 16.0);

        let lines = bars(content);
        assert_eq!(lines.len(), 4);
        assert!(lines.iter().all(|l| l.1 == 12.0));
        let ys: Vec<_> = tree.nodes[content]
            .children
            .iter()
            .map(|c| rects[c].y)
            .collect();
        assert_eq!(ys[1] - ys[0], 12.0 + 10.0);
    }

    #[test]
    fn renders_the_loading_skeleton() {
        let Ok(out) = env::var("PROFILE_PREVIEW_DIR") else {
            return;
        };
        let (mut tree, _) = mounted();
        for (name, wait) in [("skeleton_start.png", 0), ("skeleton_mid.png", 800)] {
            thread::sleep(Duration::from_millis(wait));
            let mut surface = surfaces::raster_n32_premul((300, 360)).unwrap();
            surface.canvas().clear(Color::from_rgb(0x1a, 0x1a, 0x1e));
            requiem_ui_paint::paint(&mut tree, surface.canvas());
            let png = surface
                .image_snapshot()
                .encode(None, skia_safe::EncodedImageFormat::PNG, 100)
                .unwrap();
            fs::write(Path::new(&out).join(name), png.as_bytes()).unwrap();
        }
    }
}
