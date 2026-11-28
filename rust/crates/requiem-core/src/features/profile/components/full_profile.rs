use crate::models::user_profile::ConnectedAccount;
use crate::models::user_profile::MutualFriend;
use crate::models;
use crate::presence;
use crate::query;
use crate::rest::endpoints::profile_board::{self, BoardApplication, BoardGameEntry, BoardGameInfo, ProfileBoard};
use crate::rest::endpoints::relationships::BLOCKED;
use crate::rest::endpoints::relationships::location;
use crate::rest::endpoints::relationships;
use crate::rest::endpoints::users;
use crate::ui::author_style;
use crate::ui::author_style::TagPlace;
use crate::ui::channel_sidebar;
use crate::ui::classes::full_profile;
use crate::ui::classes::full_profile as cls;
use crate::ui::custom_status::BubbleContent;
use crate::ui::custom_status;
use crate::ui::dim;
use crate::ui::media_viewer;
use crate::ui::profile_card as ui_profile_card;
use crate::ui::profile_decorations;
use crate::ui::profile_theme;
use crate::ui::twemoji;
use crate::ui;
use crate::utils::guild;
use crate::utils::link;
use crate::utils::time;
use requiem_ui::classes::modal;
use requiem_ui::components::TextVariant;
use requiem_ui::components::{Align, Justify, Padding, StackProps};
use requiem_ui::components::{ButtonSize, ButtonVariant};
use requiem_ui::components::{TextLinkProps, TextLinkVariant, menu};
use requiem_ui::icons::Icon;
use requiem_ui::widget::primitives::StatusCutout;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::OnceLock;

use requiem_ui::components::{
    AvatarStatusSize, ButtonProps, MenuItem, TabBarProps, TabItem,
    TextInputProps, TextProps, TooltipProps, avatar_with_status, div, icon,
    scroll_view,
};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::anim::ToggleState;
use requiem_ui::widget::primitives::{
    BoxWidget, ClickAreaWidget, ImageWidget, ModalBackdropWidget, ModalPanelWidget, PresenceStatus,
};
use skia_safe::Color;
use skia_safe::font_style::Weight;

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::models::id::marker::UserMarker;
use crate::models::{Id, Role, UserProfile};
use crate::query::QueryClient;
use crate::rest::{HttpClient, RestError};
use crate::state::AppState;
use crate::state::wire::Activity;
use crate::ui::profile_popout::{
    self, ColorBoxWidget, FullProfileTab, FullProfileTarget, ProfileOverlays,
};
use crate::ui::{Fonts, TEXT_DEFAULT, TEXT_MUTED, TEXT_SECONDARY, TEXT_STRONG};
use requiem_ui::icons::{
    CHAT, CIRCLE_CHECK, CLYDE, DENY, MORE_HORIZONTAL, PENCIL, SHOP, USER_PLUS,
    WARNING, WINDOW_LAUNCH, X_SMALL,
};

const BLOCK_GUIDE_URL: &str = "https://support.discord.com/hc/en-us/articles/217916488";

const TAB_ACTIVITY: &str = "activity";
const TAB_FRIENDS: &str = "friends";
const TAB_SERVERS: &str = "servers";
const TAB_BOARD: &str = "board";
const TAB_WISHLIST: &str = "wishlist";
const TAB_BIO: &str = "bio";

#[derive(Clone)]
pub struct FullProfileState {
    pub open: Signal<ToggleState>,

    pub tab: Signal<&'static str>,
    pub more_open: Signal<bool>,
    pub role_picker_open: Signal<bool>,
    pub role_search: Signal<String>,

    pub note_draft: Signal<Option<String>>,
    pub focus_note: Signal<bool>,
}

impl Default for FullProfileState {
    fn default() -> Self {
        Self::new()
    }
}

impl FullProfileState {
    pub fn new() -> Self {
        Self {
            open: signal(ToggleState::new(false)),
            tab: signal(""),
            more_open: signal(false),
            role_picker_open: signal(false),
            role_search: signal(String::new()),
            note_draft: signal(None),
            focus_note: signal(false),
        }
    }

    pub fn open_on(&self, target: FullProfileTarget) {
        self.tab.set(match target.tab {
            FullProfileTab::First => "",
            FullProfileTab::MutualFriends => TAB_FRIENDS,
            FullProfileTab::MutualServers => TAB_SERVERS,
        });
        self.more_open.set(false);
        self.role_picker_open.set(false);
        self.note_draft.set(None);
        self.focus_note.set(target.focus_note);
        self.open.update(|t| *t = t.set(true));
    }

    pub fn close(&self) {
        if self.more_open.peek() {
            self.more_open.set(false);
        }
        if self.role_picker_open.peek() {
            self.role_picker_open.set(false);
        }
        if self.open.peek().on {
            self.open.update(|t| *t = t.set(false));
        }
    }
}

pub fn section(heading: &str, content: Node) -> Node {
    StackProps::vertical()
        .full_width(false)
        .build()
        .child(
            TextProps::styled(heading, TextVariant::TEXT_XS_MEDIUM)
                .color(TEXT_SECONDARY)
                .build()
                .class(cls::SECTION_HEADING),
        )
        .child(content)
}

struct Ctx<'a> {
    target: FullProfileTarget,
    profile: &'a UserProfile,
    is_self: bool,
    status: PresenceStatus,
    custom_status: Option<presence::CustomStatus>,
    relationship: Option<u8>,
    all_roles: Option<Arc<Vec<Role>>>,
    state: &'a AppState,
    overlays: &'a ProfileOverlays,
    fonts: &'a Fonts,
    images: &'a ImageCache,
    backend: &'a BackendConfig,
    queries: &'a QueryClient,
    http: Option<&'a HttpClient>,

    viewer: Option<&'a media_viewer::MediaViewerState>,
}

#[allow(clippy::too_many_arguments)]
pub fn build(
    target: FullProfileTarget,
    open: ToggleState,
    state: &AppState,
    overlays: &ProfileOverlays,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    queries: &QueryClient,
    http: Option<&HttpClient>,
    media_viewer: Option<&media_viewer::MediaViewerState>,
) -> Node {
    let ui = &overlays.full_profile_ui;
    let user_id = target.user_id;
    let guild_id = target.guild_id;
    let profile = profile_popout::load_profile(
        queries,
        http,
        &state.users,
        user_id,
        guild_id,
        profile_popout::ProfileSurface::Modal,
    );
    let dismiss = {
        let ui = ui.clone();
        move || ui.close()
    };
    let backdrop = ModalBackdropWidget::new(open).with_on_click(dismiss);
    let is_bot = profile.as_ref().is_some_and(|p| p.user.bot);
    let panel_class = if is_bot { cls::PANEL_BOT } else { cls::PANEL };
    let theme = profile
        .as_deref()
        .and_then(|profile| profile_theme::resolve(profile).theme);
    let mut panel = Node::new("div", ModalPanelWidget::new(open))
        .class(cls::PANEL_BASE)
        .class(panel_class);
    let mut themed_content = div();
    if let Some((primary, secondary)) = theme {
        panel = panel
            .class(cls::PROFILE_THEMED)
            .child(
                Node::new(
                    "div",
                    ui_profile_card::GradientBoxWidget {
                        top: primary,
                        stop: 105.0_f32,
                        bottom: secondary,
                    },
                )
                .class(cls::PROFILE_THEME_BORDER),
            );
    }

    if let Some(profile) = profile.as_deref() {
        let viewer = state
            .users
            .current
            .with_untracked(|u| u.as_ref().map(|u| u.id));
        let is_self = viewer == Some(user_id);
        let status = if is_self {
            state.presence.status.get().indicator()
        } else {
            guild_id
                .and_then(|g| state.members.presence_of(g, user_id))
                .unwrap_or(PresenceStatus::Offline)
        };
        let custom_status = if is_self {
            state.presence.custom_status.get()
        } else {
            guild_id.and_then(|g| state.members.custom_status_of(g, user_id))
        };
        let all_roles = guild_id.map(|guild_id| {
            Arc::new(author_style::guild_roles(state, guild_id))
        });
        let relationship = profile_popout::relationship_kind(&state.relationships, user_id);
        let ctx = Ctx {
            target,
            profile,
            is_self,
            status,
            custom_status,
            relationship,
            all_roles,
            state,
            overlays,
            fonts,
            images,
            backend,
            queries,
            http,
            viewer: media_viewer,
        };
        let layout = if is_bot {
            bot_layout(&ctx)
        } else {
            user_layout(&ctx)
        };
        if theme.is_some() {
            themed_content = themed_content.child(layout);
        } else {
            panel = panel.child(layout);
        }
    }

    if !is_bot {
        let close = {
            let ui = ui.clone();
            ButtonProps::default()
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Small)
                .icon(X_SMALL)
                .on_click(move || ui.close())
                .build()
        };
        let close = div().class(cls::CLOSE).child(close);
        if theme.is_some() {
            themed_content = themed_content.child(close);
        } else {
            panel = panel.child(close);
        }
    }
    if theme.is_some() {
        panel = panel.child(themed_content.class(cls::PROFILE_THEME_CONTENT));
    }
    Node::new("div", backdrop)
        .class(modal::MODAL_BACKDROP)
        .child(panel)
}

fn user_layout(ctx: &Ctx<'_>) -> Node {
    let columns = div()
        .class(cls::COLUMNS)
        .child(profile_card(ctx))
        .child(tabs_column(ctx));
    div().class(cls::INNER).child(columns)
}

fn banner(ctx: &Ctx<'_>, class: &'static str) -> Node {
    let user = &ctx.profile.user;
    match user.banner.as_deref().and_then(|hash| {
        ctx.images
            .get_or_decode_image(&ctx.backend.banner_url(user.id, hash))
    }) {
        Some(image) => Node::new(
            "div",
            ImageWidget::new(Some(image)).with_cover_centered(),
        )
        .class(class),
        None => {
            let color = profile_theme::resolve(ctx.profile)
                .theme
                .map(|(primary, _)| primary)
                .unwrap_or_else(profile_theme::default_background);
            Node::new("div", ColorBoxWidget { color }).class(class)
        }
    }
}

fn avatar_120(ctx: &Ctx<'_>, class: &str) -> Node {
    let user = &ctx.profile.user;
    let mut props = ui::with_avatar_decoration(
        ui::user_avatar_props(user.id, &user.discriminator, ctx.fonts),
        user.avatar_decoration_asset(),
        ctx.images,
        ctx.backend,
        120,
    );
    if let Some(image) = user.avatar.as_deref().and_then(|hash| {
        ctx.images
            .get_or_decode_image(&ctx.backend.avatar_url(user.id, hash))
    }) {
        props = props.image(image);
    }
    let avatar = avatar_with_status(props, AvatarStatusSize::Px120, Some(ctx.status));
    let (Some(viewer), Some(hash)) = (ctx.viewer, user.avatar.as_deref()) else {
        return avatar.class(class);
    };
    let item = media_viewer::MediaItem {
        url: ctx.backend.avatar_url_sized(user.id, hash, 1024),
        video_url: None,
        width: None,
        height: None,
        placeholder: None,
        filename: None,
        size: None,
        alt: None,
        author: None,
        is_attachment: false,
        zoom_only: true,
    };
    let viewer = viewer.clone();
    let dim = Node::new(
        "div",
        dim::DimWidget {
            cutout: Some(StatusCutout::ROUND_120),
        },
    )
    .class(full_profile::AVATAR_OVERLAY);
    Node::new(
        "div",
        ClickAreaWidget::new().with_on_click(move || viewer.open(vec![item.clone()], 0)),
    )
    .class(class)
    .child(avatar)
    .child(dim)
}

fn profile_card(ctx: &Ctx<'_>) -> Node {
    let user = &ctx.profile.user;
    let colors = profile_theme::resolve(ctx.profile);

    let mut ring = div().class(cls::AVATAR_RING);
    if let Some((primary, _)) = colors.theme {
        let dim = |c: u8| (c as u16 * 40 / 100) as u8;
        ring = ring.style(format!(
            "background-color: #{:02x}{:02x}{:02x}",
            dim(primary.r()),
            dim(primary.g()),
            dim(primary.b())
        ));
    }
    let mut header = div()
        .class(cls::HEADER)
        .child(banner(ctx, cls::BANNER))
        .child(ring)
        .child(avatar_120(ctx, cls::AVATAR));
    header = header.children(profile_decorations::layers_with_frame_class(
        ctx.queries,
        ctx.http,
        ctx.profile,
        ctx.images,
        ctx.backend,
        full_profile::PROFILE_FRAME,
    ));
    if ctx.is_self {
        let overlays = ctx.overlays.clone();
        let current = ctx.custom_status.clone();
        let actions = custom_status::BubbleActions {
            on_open: Rc::new(move || {
                overlays.full_profile_ui.close();
                (overlays.on_open_custom_status)(current.clone());
            }),
            on_clear: ctx.overlays.on_clear_custom_status.clone(),
        };
        let content = match &ctx.custom_status {
            Some(status) => BubbleContent::Status(status),
            None => BubbleContent::Prompt(ctx.overlays.prompt.peek()),
        };
        header = header.child(
            custom_status::bubble(content, Some(&actions), ctx.images, ctx.backend)
                .class(cls::BUBBLE),
        );
    } else if let Some(status) = &ctx.custom_status {
        header = header.child(
            custom_status::bubble(
                BubbleContent::Status(status),
                None,
                ctx.images,
                ctx.backend,
            )
            .class(cls::BUBBLE),
        );
    }

    let display_name = ctx
        .profile
        .guild_member
        .as_ref()
        .and_then(|m| m.nick.clone())
        .or_else(|| user.global_name.clone())
        .unwrap_or_else(|| user.username.clone());
    let name_fill = profile_theme::name_fill(ctx.profile);
    let name_row = StackProps::horizontal()
        .align(Align::Center)
        .full_width(false)
        .build()
        .child(
            author_style::name_node(
                display_name,
                TextVariant::HEADING_XL_BOLD,
                28.0_f32,
                name_fill.as_ref(),
                colors.name_color.unwrap_or(TEXT_DEFAULT),
            ),
        );
    let mut username_row = div().class(cls::USERNAME_ROW).child(
        TextProps::styled(user.username.clone(), TextVariant::TEXT_SM_NORMAL)
            .color(TEXT_STRONG)
            .build(),
    );
    let pronouns = ctx
        .profile
        .user_profile
        .as_ref()
        .map(|m| m.pronouns.trim().to_string())
        .filter(|p| !p.is_empty());
    if let Some(pronouns) = pronouns {
        username_row = username_row
            .class(cls::USERNAME_ROW_PRONOUNS)
            .child(
                Node::new(
                    "div",
                    ColorBoxWidget {
                        color: TEXT_DEFAULT,
                    },
                )
                .class(cls::DOT),
            )
            .child(
                TextProps::styled(pronouns, TextVariant::TEXT_SM_NORMAL)
                    .color(TEXT_STRONG)
                    .build(),
            );
    }
    if let Some(tag) = ctx
        .profile
        .user
        .primary_guild
        .as_ref()
        .and_then(author_style::tag_of)
    {
        username_row = username_row.child(author_style::tag_chip(
            &tag,
            TagPlace::Pill,
            ctx.images,
            ctx.backend,
        ));
    }
    if let Some(badges) = profile_popout::badges_row(ctx.profile, ctx.images, ctx.backend) {
        username_row = username_row.child(badges);
    }
    let names = StackProps::vertical()
        .gap(0)
        .full_width(false)
        .build()
        .child(name_row)
        .child(username_row);

    let mut body = scroll_view()
        .class(cls::BODY)
        .child(names)
        .child(profile_buttons(ctx));

    if !user.bio.trim().is_empty() {
        let bio = ui_profile_card::bio_text(&user.bio, 14.0, 18.0, ctx.images, ctx.backend);
        body = body.child(StackProps::vertical().full_width(false).build().child(bio));
    }
    body = body.child(section(
        &requiem_i18n::t!("full-profile-member-since"),
        member_since(ctx),
    ));
    if let Some(roles) = roles_section(ctx) {
        body = body.child(
            section(&requiem_i18n::t!("full-profile-roles"), roles).class(cls::ROLES_SECTION),
        );
    }
    let accounts = connected_accounts(ctx);
    if !accounts.is_empty() {
        body = body.child(section(
            &requiem_i18n::t!("full-profile-connections"),
            connections(&accounts),
        ));
    }
    body = body.child(section(
        &requiem_i18n::t!("full-profile-note-only-visible-to-you"),
        note_field(ctx),
    ));

    let mut card = div().class(cls::CARD);
    if let Some(nameplate) = profile_decorations::nameplate_node(
        user.nameplate_asset(),
        ctx.images,
        ctx.backend,
        ui::classes::profile_popout::NAMEPLATE_BACKDROP,
    ) {
        card = card.child(nameplate);
    }
    card.child(header).child(body)
}

fn profile_buttons(ctx: &Ctx<'_>) -> Node {
    let mut row = StackProps::horizontal()
        .align(Align::Center)
        .full_width(false)
        .build();
    if ctx.is_self {
        let (on_edit, ui) = (
            ctx.overlays.on_edit_profile.clone(),
            ctx.overlays.full_profile_ui.clone(),
        );
        row = row.child(
            ButtonProps::default()
                .size(ButtonSize::Small)
                .label(requiem_i18n::t!("full-profile-edit-profile").to_string())
                .icon(PENCIL)
                .on_click(move || {
                    ui.close();
                    on_edit();
                })
                .build(),
        );
        row = row.child(
            TooltipProps::default()
                .text(requiem_i18n::t!("full-profile-shop"))
                .build(
                    ButtonProps::default()
                        .variant(ButtonVariant::Secondary)
                        .size(ButtonSize::Small)
                        .icon(SHOP)
                        .build(),
                ),
        );
    } else {
        if ctx.relationship.is_none() && !ctx.profile.user.bot {
            let (queries, http, user_id) =
                (ctx.queries.clone(), ctx.http.cloned(), ctx.target.user_id);
            row = row.child(
                ButtonProps::default()
                    .size(ButtonSize::Small)
                    .label(requiem_i18n::t!("full-profile-add-friend").to_string())
                    .icon(USER_PLUS)
                    .on_click(move || {
                        let Some(http) = http.clone() else { return };
                        queries.mutate(Vec::new(), move || async move {
                            relationships::send_friend_request(
                                &http,
                                user_id,
                                location::PROFILE_MODAL,
                            )
                            .await
                        });
                    })
                    .build(),
            );
        }
        row = row.child(
            TooltipProps::default()
                .text(requiem_i18n::t!("full-profile-message"))
                .build(
                    ButtonProps::default()
                        .variant(ButtonVariant::Secondary)
                        .size(ButtonSize::Small)
                        .icon(CHAT)
                        .build(),
                ),
        );
    }

    let more_open = ctx.overlays.full_profile_ui.more_open.clone();
    let toggle = more_open.clone();
    let more = ButtonProps::default()
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Small)
        .icon(MORE_HORIZONTAL)
        .on_click(move || toggle.set(!toggle.peek()))
        .build();
    let mut more_wrap = div().class(cls::MORE_WRAP).child(
        TooltipProps::default()
            .text(requiem_i18n::t!("full-profile-more"))
            .build(more),
    );
    if more_open.get() {
        if let Some(menu) = more_menu(ctx) {
            more_wrap = more_wrap.child(menu.class(cls::MORE_MENU));
        }
    }
    row.child(more_wrap)
}

fn more_menu(ctx: &Ctx<'_>) -> Option<Node> {
    let ui = ctx.overlays.full_profile_ui.clone();
    let mut items = Vec::new();
    if ctx.target.guild_id.is_some() {
        let (overlays, target) = (ctx.overlays.clone(), ctx.target);
        let aka = ctx
            .profile
            .user
            .global_name
            .clone()
            .unwrap_or_else(|| ctx.profile.user.username.clone());
        items.push(
            MenuItem::action(
                requiem_i18n::t!("full-profile-view-main-profile"),
                move || {
                    overlays.open_full_profile(FullProfileTarget {
                        guild_id: None,
                        focus_note: false,
                        tab: FullProfileTab::First,
                        server_guild_id: target.guild_id,
                        ..target
                    })
                },
            )
            .with_subtitle(requiem_i18n::t!("full-profile-aka", aka = aka)),
        );
    } else if let Some(server) = ctx.target.server_guild_id {
        let (overlays, target) = (ctx.overlays.clone(), ctx.target);
        let aka = ctx
            .state
            .members
            .nick_of(server, ctx.target.user_id)
            .or_else(|| ctx.profile.user.global_name.clone())
            .unwrap_or_else(|| ctx.profile.user.username.clone());
        items.push(
            MenuItem::action(
                requiem_i18n::t!("full-profile-view-per-server-profile"),
                move || {
                    overlays.open_full_profile(FullProfileTarget {
                        guild_id: Some(server),
                        server_guild_id: None,
                        focus_note: false,
                        tab: FullProfileTab::First,
                        ..target
                    })
                },
            )
            .with_subtitle(requiem_i18n::t!("full-profile-aka", aka = aka)),
        );
    }
    if !ctx.is_self {
        if !items.is_empty() {
            items.push(MenuItem::Separator);
        }
        let user_id = ctx.target.user_id;
        let (q, h, u) = (ctx.queries.clone(), ctx.http.cloned(), ui.clone());
        items.push(MenuItem::action(
            requiem_i18n::t!("full-profile-ignore"),
            move || {
                u.more_open.set(false);
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
        if ctx.relationship == Some(BLOCKED) {
            let (q, h, u) = (ctx.queries.clone(), ctx.http.cloned(), ui.clone());
            items.push(MenuItem::danger(
                requiem_i18n::t!("full-profile-unblock"),
                move || {
                    u.more_open.set(false);
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
            let (overlays, name) = (
                ctx.overlays.clone(),
                ctx.profile
                    .user
                    .global_name
                    .clone()
                    .unwrap_or_else(|| ctx.profile.user.username.clone()),
            );
            items.push(MenuItem::danger(
                requiem_i18n::t!("full-profile-block"),
                move || {
                    overlays.full_profile_ui.more_open.set(false);
                    overlays.confirm_block(
                        user_id,
                        name.clone(),
                        location::PROFILE_OVERFLOW_MENU,
                    );
                },
            ));
        }
    }
    if items.is_empty() {
        return None;
    }
    Some(menu(items))
}

fn member_since(ctx: &Ctx<'_>) -> Node {
    let discord = StackProps::horizontal()
        .align(Align::Center)
        .gap(4)
        .full_width(false)
        .build()
        .child(
            TooltipProps::default()
                .text(requiem_i18n::t!("full-profile-discord"))
                .build(icon(CLYDE, TEXT_SECONDARY).class(cls::MEMBER_SINCE_ICON)),
        )
        .child(
            TextProps::styled(
                time::date_label(ctx.target.user_id.created_at_millis()),
                TextVariant::TEXT_SM_NORMAL,
            )
            .color(TEXT_DEFAULT)
            .build(),
        );
    let mut row = StackProps::horizontal()
        .align(Align::Center)
        .full_width(false)
        .build()
        .child(discord);
    let joined = ctx
        .profile
        .guild_member
        .as_ref()
        .and_then(|m| m.joined_at.as_deref())
        .and_then(|at| chrono::DateTime::parse_from_rfc3339(at).ok())
        .map(|at| at.timestamp_millis());
    if let (Some(guild_id), Some(joined)) = (ctx.target.guild_id, joined) {
        let guild = ctx.state.guilds.find(guild_id);
        if let Some(guild) = guild {
            let icon = guild_icon(&guild, 16.0, ctx).class(cls::GUILD_ICON_16);
            let server = StackProps::horizontal()
                .align(Align::Center)
                .gap(4)
                .full_width(false)
                .build()
                .child(TooltipProps::default().text(&guild.name).build(icon))
                .child(
                    TextProps::styled(
                        time::date_label(joined),
                        TextVariant::TEXT_SM_NORMAL,
                    )
                    .color(TEXT_DEFAULT)
                    .build(),
                );
            row = row
                .child(
                    Node::new(
                        "div",
                        ColorBoxWidget {
                            color: TEXT_SECONDARY,
                        },
                    )
                    .class(cls::DOT),
                )
                .child(server);
        }
    }
    row
}

fn guild_icon(guild: &models::Guild, size: f32, ctx: &Ctx<'_>) -> Node {
    if let Some(image) = guild.icon.as_deref().and_then(|hash| {
        ctx.images
            .get_or_decode_image(&ctx.backend.guild_icon_url(guild.id, hash))
    }) {
        return Node::new("div", ImageWidget::new(Some(image)));
    }
    let acronym = guild::acronym(&guild.name);

    let (font, weight, color) = if size <= 16.0 {
        (6.0_f32, Weight::NORMAL, TEXT_DEFAULT)
    } else {
        (14.0_f32, Weight::MEDIUM, Color::WHITE)
    };
    let label = TextProps::new(acronym, font)
        .weight(weight)
        .color(color)
        .center(true)
        .line_clamp(1_u32)
        .build();
    div()
        .class(if size <= 16.0 {
            cls::ACRONYM_16
        } else {
            cls::ACRONYM_40
        })
        .child(label)
}

fn roles_section(ctx: &Ctx<'_>) -> Option<Node> {
    let guild_id = ctx.target.guild_id?;
    let member = ctx.profile.guild_member.as_ref()?;
    let roles = ctx.all_roles.as_ref()?;
    let ui = &ctx.overlays.full_profile_ui;
    let key = profile_popout::profile_query_key_for(
        profile_popout::ProfileSurface::Modal,
        ctx.target.user_id,
        Some(guild_id),
    );
    Some(profile_popout::build_role_section(
        ctx.target.user_id,
        guild_id,
        &member.roles,
        roles,
        ctx.queries,
        ctx.http,
        &ui.role_picker_open,
        &ui.role_search,
        &key,
        ctx.images,
        ctx.backend,
        ctx.state
            .guilds
            .find(guild_id)
            .as_ref()
            .is_some_and(|guild| {
                ctx.state
                    .permissions
                    .can_assign_any_role(guild, roles, &member.roles)
            }),
    ))
}

fn note_field(ctx: &Ctx<'_>) -> Node {
    let user_id = ctx.target.user_id;
    let key = format!("note:{user_id}");
    let saved = match ctx.http {
        Some(http) => {
            let http = http.clone();
            ctx.queries
                .query_default::<Option<String>, _, RestError>(&key, move || async move {
                    users::get_note(&http, user_id).await
                })
                .data
        }
        None => ctx.queries.peek::<Option<String>>(&key),
    };
    let ui = ctx.overlays.full_profile_ui.clone();
    let initial = ui
        .note_draft
        .peek()
        .or_else(|| saved.as_deref().cloned().flatten())
        .unwrap_or_default();
    let (draft, draft_for_blur, queries, http) = (
        ui.note_draft.clone(),
        ui.note_draft.clone(),
        ctx.queries.clone(),
        ctx.http.cloned(),
    );
    let saved_text = saved.as_deref().cloned().flatten().unwrap_or_default();
    let field = TextInputProps::default()
        .text(initial)
        .placeholder(requiem_i18n::t!("full-profile-click-to-add-a-note").to_string())
        .font_size(14.0_f32)
        .color(TEXT_DEFAULT)
        .placeholder_color(Color::from_rgb(0x96, 0x97, 0x9e))
        .multiline(true)
        .max_length(256_usize)
        .insets((4.0_f32, 4.0_f32))
        .autofocus(ui.focus_note.peek())
        .on_change(move |text: &str| draft.set(Some(text.to_string())))
        .on_focus_change(move |focused: bool| {
            if focused {
                return;
            }
            let Some(note) = draft_for_blur.peek() else {
                return;
            };
            if note == saved_text {
                return;
            }
            let Some(http) = http.clone() else { return };
            queries.mutate(vec![format!("note:{user_id}")], move || async move {
                users::set_note(&http, user_id, &note).await
            });
        })
        .build()
        .class(cls::NOTE);
    div().class(cls::NOTE_WRAP).child(field)
}

fn connected_accounts(ctx: &Ctx<'_>) -> Vec<ConnectedAccount> {
    if ctx.profile.connected_accounts.is_empty()
        && cfg!(debug_assertions)
        && env::var_os("REQUIEM_SPOOF_CONNECTIONS").is_some()
    {
        let account = |kind: &str, id: &str, name: &str, verified: bool| ConnectedAccount {
            kind: kind.to_string(),
            id: id.to_string(),
            name: name.to_string(),
            verified,
            metadata: Default::default(),
        };
        let with = |mut a: ConnectedAccount, pairs: &[(&str, &str)]| {
            a.metadata = pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            a
        };
        return vec![
            with(
                account("steam", "76561198000000000", "Gabe", true),
                &[
                    ("game_count", "123"),
                    ("item_count_dota2", "5"),
                    ("item_count_tf2", "1"),
                    ("created_at", "2010-05-03T00:00:00Z"),
                ],
            ),
            with(
                account("reddit", "2", "spez", false),
                &[
                    ("total_karma", "1234567"),
                    ("gold", "1"),
                    ("mod", "1"),
                    ("created_at", "2006-06-06T00:00:00Z"),
                ],
            ),
            with(
                account("tiktok", "3", "tt", false),
                &[
                    ("verified", "1"),
                    ("follower_count", "1500"),
                    ("following_count", "12"),
                    ("likes_count", "99999"),
                ],
            ),
            with(account("paypal", "4", "pp", false), &[("verified", "1")]),
            with(
                account("ebay", "5", "eb", false),
                &[
                    ("positive_feedback_percentage", "99"),
                    ("top_rated_seller", "1"),
                ],
            ),
            with(
                account("twitter", "6", "tw", false),
                &[("statuses_count", "42"), ("followers_count", "1500000")],
            ),
            account("twitch", "7", "a_very_long_streamer_name_here", true),
            with(
                account("github", "8", "octocat", false),
                &[("created_at", "2011-01-25T00:00:00Z")],
            ),
        ];
    }
    ctx.profile.connected_accounts.clone()
}

fn connection_url(account: &ConnectedAccount) -> Option<String> {
    let (id, name) = (&account.id, &account.name);
    Some(match account.kind.as_str() {
        "steam" => format!("https://steamcommunity.com/profiles/{id}"),
        "github" => format!("https://github.com/{name}"),
        "twitch" => format!("https://twitch.tv/{name}"),
        "twitter" => format!("https://twitter.com/{name}"),
        "reddit" => format!("https://reddit.com/u/{name}"),
        "spotify" => format!("https://open.spotify.com/user/{id}"),
        "youtube" => format!("https://www.youtube.com/channel/{id}"),
        "tiktok" => format!("https://tiktok.com/@{name}"),
        "instagram" => format!("https://instagram.com/{name}"),
        "facebook" => format!("https://facebook.com/{id}"),
        "xbox" => format!("https://account.xbox.com/en-us/Profile?gamertag={name}"),
        "bluesky" => format!("https://bsky.app/profile/{name}"),
        _ => return None,
    })
}

fn connections(accounts: &[ConnectedAccount]) -> Node {
    let mut list = StackProps::vertical()
        .gap(16)
        .padding(Padding::top(4))
        .build();
    for account in accounts {
        let mut row = StackProps::horizontal()
            .align(Align::Stretch)
            .gap(8)
            .full_width(false)
            .build();
        if let Some(image) = connection_list_icon(&account.kind) {
            row = row.child(
                TooltipProps::default()
                    .text(platform_name(&account.kind))
                    .build(
                        StackProps::horizontal()
                            .align(Align::Center)
                            .justify(Justify::Center)
                            .gap(0)
                            .full_width(false)
                            .build()
                            .class(cls::CONNECTION_TILE)
                            .child(
                                Node::new("div", ImageWidget::new(Some(image)))
                                    .class(cls::CONNECTION_ICON),
                            ),
                    ),
            );
        }
        let name = TextProps::styled(account.name.clone(), TextVariant::TEXT_SM_NORMAL)
            .color(TEXT_DEFAULT)
            .line_clamp(1_u32)
            .build()
            .class(cls::CONNECTION_NAME);
        let link = match connection_url(account) {
            Some(url) => {
                let content = StackProps::horizontal()
                    .align(Align::Center)
                    .gap(2)
                    .full_width(false)
                    .build()
                    .children([
                        name,
                        icon(WINDOW_LAUNCH, TEXT_DEFAULT).class(cls::CONNECTION_OPEN_ICON),
                    ]);
                ButtonProps::default()
                    .variant(ButtonVariant::Bare)
                    .on_click(move || link::open_url(&url))
                    .build()
                    .child(content)
            }
            None => name,
        };
        let mut info = StackProps::vertical()
            .justify(Justify::Center)
            .gap(4)
            .full_width(false)
            .build()
            .child(link);
        if let Some(date) = account
            .metadata
            .get("created_at")
            .and_then(|at| chrono::DateTime::parse_from_rfc3339(at).ok())
        {
            info = info.child(
                TextProps::styled(
                    requiem_i18n::t!(
                        "full-profile-connection-member-since",
                        date = time::date_label(date.timestamp_millis())
                    ),
                    TextVariant::TEXT_XS_NORMAL,
                )
                .color(TEXT_DEFAULT)
                .build(),
            );
        }
        let metadata = connection_metadata(account);
        if !metadata.is_empty() {
            info = info.child(
                StackProps::horizontal()
                    .wrap(true)
                    .align(Align::Center)
                    .gap(8)
                    .padding(Padding::top(4))
                    .full_width(false)
                    .build()
                    .children(metadata),
            );
        }
        list = list.child(row.child(info));
    }
    list
}

fn count_label(n: i64) -> String {
    if n < 1_000_000 {
        let digits = n.abs().to_string();
        let mut out = String::new();
        for (i, c) in digits.chars().enumerate() {
            if i > 0 && (digits.len() - i).is_multiple_of(3) {
                out.push(',');
            }
            out.push(c);
        }
        if n < 0 { format!("-{out}") } else { out }
    } else {
        format!("{:.1}M", n as f64 / 1e6)
    }
}

fn connection_metadata(account: &ConnectedAccount) -> Vec<Node> {
    let m = &account.metadata;
    let number = |key: &str| {
        m.get(key)
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|v| *v > -1.0)
            .map(|v| v as i64)
    };
    let flag = |key: &str| m.get(key).map(|v| v == "1").unwrap_or(false);
    let text = |label: String| {
        TextProps::styled(label, TextVariant::TEXT_XXS_NORMAL)
            .color(TEXT_DEFAULT)
            .build()
    };
    let item = |label: String| text(label);
    let tag = |label: String| div().class(cls::CONNECTION_META_TAG).child(text(label));
    let mut out = Vec::new();
    match account.kind.as_str() {
        "reddit" => {
            if let Some(n) = number("total_karma") {
                out.push(item(requiem_i18n::t!(
                    "full-profile-connection-karma",
                    value = count_label(n)
                )));
            }
            if flag("gold") {
                out.push(tag(requiem_i18n::t!("full-profile-connection-gold")));
            }
            if flag("mod") {
                out.push(tag(requiem_i18n::t!("full-profile-connection-mod")));
            }
        }
        "steam" => {
            if let Some(n) = number("game_count") {
                out.push(item(requiem_i18n::t!(
                    "full-profile-connection-games",
                    value = count_label(n)
                )));
            }
            for (key, asset, name) in [
                (
                    "item_count_dota2",
                    "connection_item/dota2",
                    requiem_i18n::t!("full-profile-connection-dota2"),
                ),
                (
                    "item_count_tf2",
                    "connection_item/tf2",
                    requiem_i18n::t!("full-profile-connection-tf2"),
                ),
            ] {
                if let Some(n) = number(key) {
                    let mut row = StackProps::horizontal()
                        .align(Align::Center)
                        .gap(4)
                        .full_width(false)
                        .build();
                    if let Some(image) = ui::asset_image(asset) {
                        row = row.child(
                            TooltipProps::default().text(name).build(
                                Node::new("div", ImageWidget::new(Some(image)))
                                    .class(cls::CONNECTION_META_ICON),
                            ),
                        );
                    }
                    out.push(row.child(text(requiem_i18n::t!(
                        "full-profile-connection-items",
                        count = n,
                        value = count_label(n)
                    ))));
                }
            }
        }
        "twitter" | "bluesky" | "mastodon" => {
            if let Some(n) = number("statuses_count") {
                out.push(item(requiem_i18n::t!(
                    "full-profile-connection-posts",
                    value = count_label(n)
                )));
            }
            if let Some(n) = number("followers_count") {
                out.push(item(requiem_i18n::t!(
                    "full-profile-connection-followers",
                    value = count_label(n)
                )));
            }
        }
        "paypal" => {
            if flag("verified") {
                out.push(
                    div()
                        .class(cls::CONNECTION_META_TAG)
                        .class(cls::CONNECTION_META_POSITIVE)
                        .child(text(requiem_i18n::t!(
                            "full-profile-connection-verified-bank"
                        ))),
                );
            }
        }
        "ebay" => {
            if let Some(n) = number("positive_feedback_percentage").filter(|n| *n > 0) {
                out.push(item(requiem_i18n::t!(
                    "full-profile-connection-ratings",
                    value = format!("{}%", count_label(n))
                )));
            }
            if flag("top_rated_seller") {
                out.push(tag(requiem_i18n::t!("full-profile-connection-top-rated")));
            }
        }
        "tiktok" => {
            if let Some(n) = number("follower_count") {
                out.push(item(requiem_i18n::t!(
                    "full-profile-connection-followers",
                    value = count_label(n)
                )));
            }
            if let Some(n) = number("following_count") {
                out.push(item(requiem_i18n::t!(
                    "full-profile-connection-following",
                    value = count_label(n)
                )));
            }
            if let Some(n) = number("likes_count") {
                out.push(item(requiem_i18n::t!(
                    "full-profile-connection-likes",
                    value = count_label(n)
                )));
            }
            if flag("verified") {
                out.push(tag(requiem_i18n::t!("full-profile-connection-verified")));
            }
        }
        _ => {}
    }
    out
}

fn connection_list_icon(kind: &str) -> Option<skia_safe::Image> {
    thread_local! {
        static ICONS: RefCell<HashMap<String, Option<skia_safe::Image>>> = RefCell::new(HashMap::new());
    }
    ICONS.with(|icons| {
        icons
            .borrow_mut()
            .entry(kind.to_string())
            .or_insert_with(|| ui::asset_image(&format!("connection_icon/{kind}")))
            .clone()
    })
}

fn platform_name(kind: &str) -> String {
    match kind {
        "battlenet" => "Battle.net",
        "bungie" => "Bungie.net",
        "leagueoflegends" => "League of Legends",
        "epicgames" => "Epic Games",
        "riotgames" => "Riot Games",
        "playstation" => "PlayStation Network",
        "twitter" => "X",
        "twitter_legacy" => "Twitter",
        "github" => "GitHub",
        "paypal" => "PayPal",
        "ebay" => "eBay",
        "tiktok" => "TikTok",
        "youtube" => "YouTube",
        "amazon-music" => "Amazon Music",
        other => {
            return other
                .chars()
                .next()
                .map(|c| c.to_uppercase().collect::<String>() + &other[c.len_utf8()..])
                .unwrap_or_default();
        }
    }
    .to_string()
}

fn connection_icon(kind: &str) -> Option<&'static skia_safe::Image> {
    macro_rules! icon {
        ($name:literal) => {{
            static IMAGE: OnceLock<Option<skia_safe::Image>> = OnceLock::new();
            IMAGE
                .get_or_init(|| crate::ui::asset_image(concat!("connection/", $name)))
                .as_ref()
        }};
    }
    match kind {
        "steam" => icon!("steam"),
        "playstation" => icon!("playstation"),
        "xbox" => icon!("xbox"),
        "twitch" => icon!("twitch"),
        "battlenet" => icon!("battlenet"),
        "leagueoflegends" => icon!("leagueoflegends"),
        "epicgames" => icon!("epicgames"),
        "riotgames" => icon!("riotgames"),
        "roblox" => icon!("roblox"),
        "spotify" => icon!("spotify"),
        "youtube" => icon!("youtube"),
        "crunchyroll" => icon!("crunchyroll"),
        "bungie" => icon!("bungie"),
        _ => None,
    }
}

const ACTIVITY_PLATFORMS: [&str; 13] = [
    "steam",
    "playstation",
    "xbox",
    "twitch",
    "battlenet",
    "leagueoflegends",
    "epicgames",
    "riotgames",
    "roblox",
    "spotify",
    "youtube",
    "crunchyroll",
    "bungie",
];

fn tabs_column(ctx: &Ctx<'_>) -> Node {
    let friends = ctx.profile.mutual_friends.as_deref().unwrap_or(&[]);
    let friend_count = ctx
        .profile
        .mutual_friends_count
        .unwrap_or(friends.len() as u32) as usize;
    let server_count = ctx.profile.mutual_guilds.as_ref().map_or(0, |g| g.len());
    let items: Vec<TabItem> = if ctx.is_self {
        vec![
            TabItem::new(TAB_BOARD, requiem_i18n::t!("full-profile-board")),
            TabItem::new(TAB_ACTIVITY, requiem_i18n::t!("full-profile-activity")),
            TabItem::new(TAB_WISHLIST, requiem_i18n::t!("full-profile-wishlist")),
        ]
    } else {
        let mut items = vec![TabItem::new(
            TAB_ACTIVITY,
            requiem_i18n::t!("full-profile-activity"),
        )];

        let friends_label = if friend_count > 0 {
            requiem_i18n::t!("mutual-friends", count = friend_count)
        } else {
            requiem_i18n::t!("full-profile-no-mutual-friends").to_string()
        };
        items.push(TabItem::new(TAB_FRIENDS, friends_label));
        let servers_label = if server_count > 0 {
            requiem_i18n::t!("mutual-servers", count = server_count)
        } else {
            requiem_i18n::t!("full-profile-no-mutual-servers").to_string()
        };
        items.push(TabItem::new(TAB_SERVERS, servers_label));
        items
    };
    let wanted = ctx.overlays.full_profile_ui.tab.get();
    let selected = items
        .iter()
        .find(|i| i.id == wanted)
        .map(|i| i.id.clone())
        .unwrap_or_else(|| items[0].id.clone());
    let tab_signal = ctx.overlays.full_profile_ui.tab.clone();
    let bar = TabBarProps::default()
        .items(items)
        .selected(selected.clone())
        .on_select(Rc::new(move |id: String| {
            let id: &'static str = [
                TAB_ACTIVITY,
                TAB_FRIENDS,
                TAB_SERVERS,
                TAB_BOARD,
                TAB_WISHLIST,
            ]
            .into_iter()
            .find(|t| *t == id)
            .unwrap_or("");
            tab_signal.set(id);
        }) as Rc<dyn Fn(String)>)
        .build();
    let panel = match selected.as_str() {
        TAB_FRIENDS => mutual_friends(ctx),
        TAB_SERVERS => mutual_servers(ctx),
        TAB_BOARD => board(ctx),
        TAB_WISHLIST => wishlist(),
        _ if ctx.is_self => own_activity(ctx),
        _ => activity(ctx),
    };
    div()
        .class(cls::TABS_COLUMN)
        .child(div().class(cls::TAB_BAR).child(bar))
        .child(div().class(cls::TAB_PANEL).child(panel))
}

pub fn empty_state(title: Node, line: &str, extra: Vec<Node>) -> Node {
    let text_block = div().class(cls::EMPTY_TEXT).child(title).child(
        TextProps::styled(line.to_string(), TextVariant::TEXT_SM_NORMAL)
            .color(TEXT_DEFAULT)
            .center(true)
            .build(),
    );
    div().class(cls::EMPTY).child(text_block).children(extra)
}

pub fn empty_title(content: &str) -> Node {
    TextProps::styled(content.to_string(), TextVariant::TEXT_MD_MEDIUM)
        .color(TEXT_STRONG)
        .center(true)
        .build()
        .class(full_profile::EMPTY_TITLE)
}

fn profile_activities(ctx: &Ctx<'_>) -> Vec<Activity> {
    let activities = if ctx.is_self {
        ctx.state.presence.activities.get()
    } else {
        ctx.state.user_presence.of(ctx.profile.user.id).activities
    };
    activities
        .into_iter()
        .filter(|activity| activity.kind != 4)
        .filter(|activity| {
            activity
                .name
                .as_deref()
                .is_some_and(|name| !name.trim().is_empty())
        })
        .collect()
}

fn activity_asset_url(
    backend: &BackendConfig,
    activity: &Activity,
    raw: &str,
) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    if raw.starts_with("http://") || raw.starts_with("https://") {
        return Some(raw.to_string());
    }
    if let Some(spotify) = raw.strip_prefix("spotify:") {
        let hash = spotify.strip_prefix("ab:").unwrap_or(spotify);
        return Some(format!("https://i.scdn.co/image/{hash}"));
    }
    let application_id = activity.application_id.as_deref()?;
    (!application_id.is_empty()).then(|| backend.app_asset_url(application_id, raw))
}

fn duration_label(ms: u64) -> String {
    let seconds = ms / 1000;
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

fn activity_timing(activity: &Activity) -> Option<String> {
    let timestamps = activity.timestamps.as_ref()?;
    let now = time::now_ms();
    if timestamps.end > 0 {
        return Some(format!(
            "{} left",
            duration_label(timestamps.end.saturating_sub(now))
        ));
    }
    if timestamps.start > 0 {
        return Some(format!(
            "{} elapsed",
            duration_label(now.saturating_sub(timestamps.start))
        ));
    }
    None
}

fn activity_line(text: &str, variant: TextVariant, color: Color, class: &'static str) -> Node {
    TextProps::styled(text.to_string(), variant)
        .color(color)
        .line_clamp(1_u32)
        .build()
        .class(class)
}

fn activity_card(ctx: &Ctx<'_>, activity: &Activity) -> Node {
    let verb = match activity.kind {
        1 => "Streaming",
        2 => "Listening to",
        3 => "Watching",
        5 => "Competing in",
        _ => "Playing",
    };
    let name = activity.name.as_deref().unwrap_or("Unknown activity");
    let mut main = div()
        .class(cls::ACTIVITY_MAIN)
        .child(activity_line(
            &format!("{verb} {name}"),
            TextVariant::TEXT_XS_SEMIBOLD,
            TEXT_SECONDARY,
            cls::ACTIVITY_HEADING,
        ));

    let assets = activity.assets.as_ref();
    if let Some(details) = activity
        .details
        .as_deref()
        .filter(|details| !details.trim().is_empty())
    {
        main = main.child(activity_line(
            details,
            TextVariant::TEXT_MD_NORMAL,
            TEXT_STRONG,
            cls::ACTIVITY_DETAILS,
        ));
    }
    if let Some(state) = activity
        .state
        .as_deref()
        .filter(|state| !state.trim().is_empty())
    {
        main = main.child(activity_line(
            state,
            TextVariant::TEXT_SM_NORMAL,
            TEXT_DEFAULT,
            cls::ACTIVITY_STATE,
        ));
    }
    if let Some(timing) = activity_timing(activity) {
        main = main.child(activity_line(
            &timing,
            TextVariant::TEXT_XS_NORMAL,
            TEXT_MUTED,
            cls::ACTIVITY_TIME,
        ));
    }

    let large = assets
        .and_then(|assets| assets.large_image.as_deref())
        .and_then(|raw| activity_asset_url(ctx.backend, activity, raw))
        .and_then(|url| ctx.images.get_or_decode_image(&url));
    let small = assets
        .and_then(|assets| assets.small_image.as_deref())
        .and_then(|raw| activity_asset_url(ctx.backend, activity, raw))
        .and_then(|url| ctx.images.get_or_decode_image(&url));

    let mut art = match large {
        Some(image) => Node::new("div", ImageWidget::new(Some(image)).with_cover_centered()),
        None => Node::new("div", BoxWidget),
    }
    .class(cls::ACTIVITY_ART);
    if let Some(small) = small {
        art = art.child(
            Node::new("div", ImageWidget::new(Some(small)).with_cover_centered())
                .class(cls::ACTIVITY_ART_SMALL),
        );
    }
    div()
        .class(cls::ACTIVITY_CARD)
        .child(main)
        .child(div().class(cls::ACTIVITY_ART_WRAP).child(art))
}

fn activity_cards(ctx: &Ctx<'_>, activities: &[Activity]) -> Node {
    let mut list = div().class(cls::ACTIVITY_LIST);
    for activity in activities {
        list = list.child(activity_card(ctx, activity));
    }
    list
}

fn activity(ctx: &Ctx<'_>) -> Node {
    let activities = profile_activities(ctx);
    if !activities.is_empty() {
        return activity_cards(ctx, &activities);
    }
    let name = ctx
        .profile
        .user
        .global_name
        .clone()
        .unwrap_or_else(|| ctx.profile.user.username.clone());
    let message = ButtonProps::default()
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Small)
        .label(requiem_i18n::t!("full-profile-message").to_string())
        .icon(CHAT)
        .build();
    let buttons = StackProps::horizontal()
        .full_width(false)
        .build()
        .child(message);
    empty_state(
        empty_title(&requiem_i18n::t!(
            "full-profile-no-activity-title",
            name = name
        )),
        &requiem_i18n::t!("full-profile-no-activity-body"),
        vec![buttons],
    )
}

fn own_activity(ctx: &Ctx<'_>) -> Node {
    let activities = profile_activities(ctx);
    if !activities.is_empty() {
        return activity_cards(ctx, &activities);
    }
    let mut platforms = div().class(cls::PLATFORMS);
    for kind in ACTIVITY_PLATFORMS {
        if let Some(image) = connection_icon(kind) {
            platforms = platforms.child(
                Node::new("div", ImageWidget::new(Some(image.clone()))).class(cls::PLATFORM_ICON),
            );
        }
    }
    let settings_button = |label: &str| {
        let (on_edit, ui) = (
            ctx.overlays.on_edit_profile.clone(),
            ctx.overlays.full_profile_ui.clone(),
        );
        ButtonProps::default()
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Small)
            .label(label.to_string())
            .on_click(move || {
                ui.close();
                on_edit();
            })
            .build()
    };
    let buttons = StackProps::horizontal()
        .full_width(false)
        .build()
        .child(settings_button(&requiem_i18n::t!(
            "full-profile-connect-accounts"
        )))
        .child(settings_button(&requiem_i18n::t!("full-profile-add-game")));
    empty_state(
        empty_title(&requiem_i18n::t!(
            "full-profile-you-don-t-have-any-activity"
        )),
        &requiem_i18n::t!("full-profile-connect-your-accounts-to-share-your"),
        vec![platforms, buttons],
    )
}

fn board(ctx: &Ctx<'_>) -> Node {
    let user_id = ctx.profile.user.id;
    let key = format!("profile-board:{}", user_id.get());
    let Some(http) = ctx.http.cloned() else {
        return board_message(&requiem_i18n::t!("full-profile-board-unavailable"), Vec::new());
    };
    let config = ctx.backend.clone();
    let result = ctx.queries.query_default::<ProfileBoard, _, RestError>(&key, move || async move {
        profile_board::load(&http, &config, user_id).await
    });

    if result.is_loading {
        return board_message(
            &requiem_i18n::t!("full-profile-board-loading"),
            Vec::new(),
        );
    }
    if result.data.is_none() {
        if let Some(error) = result.error.as_deref() {
            let key = key.clone();
            let queries = ctx.queries.clone();
            return empty_state(
                empty_title(&requiem_i18n::t!("full-profile-board-error")),
                error,
                vec![
                    ButtonProps::default()
                        .size(ButtonSize::Small)
                        .label(requiem_i18n::t!("full-profile-board-retry").to_string())
                        .on_click(move || queries.invalidate(&key))
                        .build(),
                ],
            );
        }
        return board_message(
            &requiem_i18n::t!("full-profile-board-empty"),
            Vec::new(),
        );
    }
    let Some(board) = result.data.as_deref() else {
        return board_message(
            &requiem_i18n::t!("full-profile-board-empty"),
            Vec::new(),
        );
    };
    if board.is_empty() {
        return board_message(
            &requiem_i18n::t!("full-profile-board-empty"),
            Vec::new(),
        );
    }

    let mut sections = Vec::new();
    for widget in &board.widgets {
        sections.push(board_widget_section(ctx, widget, board));
    }
    StackProps::vertical()
        .gap(24)
        .full_width(false)
        .build()
        .children(sections)
}

fn board_message(message: &str, extra: Vec<Node>) -> Node {
    empty_state(empty_title(message), "", extra)
}

fn board_widget_section(ctx: &Ctx<'_>, widget: &profile_board::BoardWidget, board: &ProfileBoard) -> Node {
    let mut cards = Vec::new();
    if let Some(application_id) = widget.application_id.as_deref() {
        match board.applications.get(application_id) {
            Some(application) => cards.push(board_application_card(ctx, application)),
            None => cards.push(board_message(
                &requiem_i18n::t!("full-profile-board-application-unavailable"),
                Vec::new(),
            )),
        }
    }
    for entry in &widget.games {
        cards.push(board_game_card(ctx, entry, board.games.get(&entry.game_id)));
    }
    div()
        .class(cls::BOARD_SECTION)
        .child(
            TextProps::styled(board_widget_title(&widget.widget_type), TextVariant::TEXT_MD_SEMIBOLD)
                .color(TEXT_STRONG)
                .build()
                .class(cls::BOARD_SECTION_TITLE),
        )
        .child(
            StackProps::horizontal()
                .justify(Justify::Center)
                .gap(12)
                .wrap(true)
                .full_width(false)
                .build()
                .children(cards),
        )
}

fn board_widget_title(widget_type: &str) -> String {
    match widget_type {
        "current_games" => requiem_i18n::t!("full-profile-board-playing"),
        "played_games" => requiem_i18n::t!("full-profile-board-recently-played"),
        "favorite_games" => requiem_i18n::t!("full-profile-board-favorites"),
        "want_to_play_games" => requiem_i18n::t!("full-profile-board-want-to-play"),
        "application" => requiem_i18n::t!("full-profile-board-application"),
        _ => requiem_i18n::t!("full-profile-board-title"),
    }
    .to_string()
}

fn board_image(ctx: &Ctx<'_>, url: Option<&str>, image_class: &'static str, fallback_class: &'static str) -> Node {
    url.and_then(|url| ctx.images.get_or_decode_image(url))
        .map(|image| {
            Node::new("div", ImageWidget::new(Some(image)).with_cover_centered()).class(image_class)
        })
        .unwrap_or_else(|| div().class(fallback_class))
}

fn board_game_card(ctx: &Ctx<'_>, entry: &BoardGameEntry, info: Option<&BoardGameInfo>) -> Node {
    let name = info
        .map(|info| info.name.clone())
        .unwrap_or_else(|| entry.game_id.clone());
    let cover = board_image(
        ctx,
        info.and_then(|info| info.image.as_deref()),
        cls::BOARD_COVER,
        cls::BOARD_COVER_PLACEHOLDER,
    );
    let mut copy = div().class(cls::BOARD_COPY).child(
        TextProps::styled(name, TextVariant::TEXT_MD_MEDIUM)
            .color(TEXT_STRONG)
            .build()
            .class(cls::BOARD_CARD_TITLE),
    );
    if let Some(comment) = entry.comment.as_deref().filter(|comment| !comment.trim().is_empty()) {
        copy = copy.child(
            TextProps::styled(comment.to_string(), TextVariant::TEXT_XS_NORMAL)
                .color(TEXT_DEFAULT)
                .build()
                .class(cls::BOARD_CARD_SUBTITLE),
        );
    }
    if !entry.tags.is_empty() {
        copy = copy.child(
            TextProps::styled(entry.tags.join(" • "), TextVariant::TEXT_XS_NORMAL)
                .color(TEXT_SECONDARY)
                .build()
                .class(cls::BOARD_CARD_TAGS),
        );
    }
    div()
        .class(cls::BOARD_CARD)
        .child(cover)
        .child(copy)
}

fn board_application_card(ctx: &Ctx<'_>, application: &BoardApplication) -> Node {
    let hero = board_image(
        ctx,
        application.image.as_deref(),
        cls::BOARD_APP_IMAGE,
        cls::BOARD_APP_IMAGE_PLACEHOLDER,
    );
    let icon = board_image(
        ctx,
        application.icon.as_deref(),
        cls::BOARD_APP_ICON,
        cls::BOARD_APP_ICON_PLACEHOLDER,
    );
    let mut overlay = div()
        .class(cls::BOARD_APP_OVERLAY)
        .child(
            StackProps::horizontal()
                .align(Align::Center)
                .gap(10)
                .full_width(false)
                .build()
                .child(icon)
                .child(
                    TextProps::styled(application.name.clone(), TextVariant::TEXT_SM_SEMIBOLD)
                        .color(TEXT_STRONG)
                        .build(),
                ),
        );
    if let Some(title) = application.title.as_deref().filter(|title| !title.is_empty()) {
        overlay = overlay.child(
            TextProps::styled(title.to_string(), TextVariant::TEXT_MD_SEMIBOLD)
                .color(TEXT_STRONG)
                .build(),
        );
    }
    for subtitle in application.subtitles.iter().filter(|s| !s.is_empty()) {
        overlay = overlay.child(
            TextProps::styled(subtitle.clone(), TextVariant::TEXT_XS_NORMAL)
                .color(TEXT_DEFAULT)
                .build(),
        );
    }
    if !application.stats.is_empty() {
        let mut stats = StackProps::horizontal()
            .gap(16)
            .wrap(true)
            .full_width(false)
            .build();
        for stat in &application.stats {
            stats = stats.child(
                div()
                    .class(cls::BOARD_STAT)
                    .child(
                        TextProps::styled(stat.value.clone(), TextVariant::TEXT_SM_SEMIBOLD)
                            .color(TEXT_STRONG)
                            .build(),
                    )
                    .child(
                        TextProps::styled(stat.label.clone(), TextVariant::TEXT_XS_NORMAL)
                            .color(TEXT_SECONDARY)
                            .build(),
                    ),
            );
        }
        overlay = overlay.child(stats);
    }
    div()
        .class(cls::BOARD_APP_CARD)
        .child(hero)
        .child(overlay)
}

fn wishlist() -> Node {
    let shop = ButtonProps::default()
        .size(ButtonSize::Small)
        .label(requiem_i18n::t!("full-profile-explore-the-shop").to_string())
        .icon(SHOP)
        .build();
    let buttons = StackProps::horizontal()
        .full_width(false)
        .build()
        .child(shop);

    let mut title = StackProps::horizontal()
        .align(Align::Center)
        .gap(4)
        .full_width(false)
        .build()
        .child(
            TextProps::styled(
                requiem_i18n::t!("full-profile-no-hearts-yet").to_string(),
                TextVariant::TEXT_MD_MEDIUM,
            )
            .color(TEXT_STRONG)
            .build(),
        );
    if let Some((image, (x, y, w, h))) = twemoji::tile("\u{1f494}") {
        title = title.child(
            Node::new(
                "div",
                ImageWidget::new(Some(image.clone())).with_src_rect(x, y, w, h),
            )
            .class(full_profile::EMPTY_TITLE_EMOJI),
        );
    }
    empty_state(
        title,
        &requiem_i18n::t!("full-profile-browse-the-shop-and-heart-the"),
        vec![buttons],
    )
}

/// The modal's profile request only asks for the count, as Discord's does. The list comes from
/// the user store, which keeps it from the popout and sidebar profiles that ask for it. Like
/// Discord's tab, when the count says there are friends and the stored list is missing or a
/// different length, this fetches it once.
fn mutual_friend_list(ctx: &Ctx<'_>) -> Vec<MutualFriend> {
    let user_id = ctx.target.user_id;
    if let Some(friends) = &ctx.profile.mutual_friends {
        return friends.clone();
    }
    let count = ctx
        .profile
        .mutual_friends_count
        .or_else(|| {
            ctx.state
                .users
                .mutual_friends_of(user_id)
                .map(|friends| friends.len() as u32)
        })
        .unwrap_or(0);
    if let Some(http) = ctx.http.cloned() {
        let key = format!("mutual-friends:{user_id}");
        let fetched = if ctx.state.users.should_fetch_mutual_friends(user_id, count) {
            Some(ctx.queries.query_default::<Vec<MutualFriend>, _, RestError>(
                &key,
                move || async move {
                    users::get_mutual_friends(&http, user_id).await
                },
            ))
        } else {
            ctx.queries.peek::<Vec<MutualFriend>>(&key).map(|data| {
                query::QueryResult {
                    data: Some(data),
                    error: None,
                    is_loading: false,
                    is_fetching: false,
                }
            })
        };
        if let Some(friends) = fetched.and_then(|result| result.data) {
            ctx.state.users.set_mutual_friends(user_id, &friends);
        }
    }
    ctx.state
        .users
        .mutual_friends_of(user_id)
        .unwrap_or_default()
}

fn mutual_friends(ctx: &Ctx<'_>) -> Node {
    let friends = mutual_friend_list(ctx);
    if friends.is_empty() {
        return empty_state(
            empty_title(&requiem_i18n::t!("full-profile-no-mutual-friends-yet")),
            &requiem_i18n::t!("full-profile-join-the-same-servers-to-meet"),
            Vec::new(),
        );
    }
    let mut list = scroll_view().class(cls::LIST);
    for friend in &friends {
        let mut props = ui::user_avatar_props(
            friend.id,
            friend.discriminator.as_deref().unwrap_or("0"),
            ctx.fonts,
        );
        if let Some(image) = friend.avatar.as_deref().and_then(|hash| {
            ctx.images
                .get_or_decode_image(&ctx.backend.avatar_url(friend.id, hash))
        }) {
            props = props.image(image);
        }
        let status = ctx
            .target
            .guild_id
            .and_then(|g| ctx.state.members.presence_of(g, friend.id))
            .unwrap_or(PresenceStatus::Offline);
        let avatar =
            avatar_with_status(props, AvatarStatusSize::Px40, Some(status)).class(cls::ROW_AVATAR);
        let name = friend
            .global_name
            .clone()
            .unwrap_or_else(|| friend.username.clone());
        let (overlays, friend_id) = (ctx.overlays.clone(), friend.id);
        let row = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::ROW.to_string()])
            .on_click(move || {
                overlays.open_full_profile(FullProfileTarget {
                    user_id: friend_id,
                    guild_id: None,
                    focus_note: false,
                    tab: FullProfileTab::First,
                    server_guild_id: None,
                })
            })
            .build()
            .child(avatar)
            .child(
                TextProps::styled(name, TextVariant::TEXT_MD_MEDIUM)
                    .color(TEXT_DEFAULT)
                    .build()
                    .class(cls::ROW_NAME),
            );
        list = list.child(row);
    }
    list
}

fn mutual_servers(ctx: &Ctx<'_>) -> Node {
    if ctx
        .profile
        .mutual_guilds
        .as_deref()
        .unwrap_or(&[])
        .is_empty()
    {
        return empty_state(
            empty_title(&requiem_i18n::t!("full-profile-no-servers-in-common")),
            "",
            Vec::new(),
        );
    }
    let mut list = scroll_view().class(cls::LIST);
    for mutual in ctx.profile.mutual_guilds.as_deref().unwrap_or(&[]) {
        let Some(guild) = ctx.state.guilds.find(mutual.id) else {
            continue;
        };
        let (on_open, ui, guild_id) = (
            ctx.overlays.on_open_guild.clone(),
            ctx.overlays.full_profile_ui.clone(),
            guild.id,
        );
        let row = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::ROW.to_string()])
            .on_click(move || {
                ui.close();
                on_open(guild_id);
            })
            .build()
            .child(guild_icon(&guild, 40.0, ctx).class(cls::GUILD_ICON_40))
            .child(
                TextProps::styled(guild.name.clone(), TextVariant::TEXT_MD_MEDIUM)
                    .color(TEXT_DEFAULT)
                    .build()
                    .class(cls::ROW_NAME),
            );
        list = list.child(row);
    }
    list
}

fn bot_layout(ctx: &Ctx<'_>) -> Node {
    let user = &ctx.profile.user;

    let more_open = ctx.overlays.full_profile_ui.more_open.clone();
    let toggle = more_open.clone();
    let more = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::BOT_BANNER_BTN.to_string()])
        .on_click(move || toggle.set(!toggle.peek()))
        .build()
        .child(icon(MORE_HORIZONTAL, Color::WHITE).class(cls::BOT_BANNER_BTN_ICON));
    let mut more_wrap = div().class(cls::BOT_BANNER_BUTTONS).child(
        TooltipProps::default()
            .text(requiem_i18n::t!("full-profile-more"))
            .build(more),
    );
    if more_open.get() {
        if let Some(menu) = more_menu(ctx) {
            more_wrap = more_wrap.child(menu.class(cls::BOT_MORE_MENU));
        }
    }
    let message = ButtonProps::default()
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Small)
        .label(requiem_i18n::t!("full-profile-message").to_string())
        .icon(CHAT)
        .build();
    let header = div()
        .class(cls::BOT_HEADER)
        .child(banner(ctx, cls::BOT_BANNER))
        .child(div().class(cls::BOT_AVATAR_RING))
        .child(avatar_120(ctx, cls::BOT_AVATAR))
        .child(more_wrap)
        .child(div().class(cls::BOT_HEADER_BUTTONS).child(message));

    let tag = div().class(cls::BOT_TAG).child(
        TextProps::styled(
            requiem_i18n::t!("common-app-tag"),
            TextVariant::TEXT_XS_SEMIBOLD,
        )
        .color(Color::WHITE)
        .build(),
    );
    let name_row = StackProps::horizontal()
        .align(Align::Center)
        .full_width(false)
        .build()
        .child(
            TextProps::new(
                user.global_name
                    .clone()
                    .unwrap_or_else(|| user.username.clone()),
                24.0_f32,
            )
            .weight(Weight::SEMI_BOLD)
            .color(TEXT_DEFAULT)
            .build(),
        )
        .child(tag);
    let info = div().class(cls::BOT_INFO).child(name_row).child(
        div().class(cls::BOT_USERNAME_ROW).child(
            TextProps::styled(user.username.clone(), TextVariant::TEXT_SM_NORMAL)
                .color(TEXT_STRONG)
                .build(),
        ),
    );

    let server_count = ctx.profile.mutual_guilds.as_ref().map_or(0, |g| g.len());
    let mut items = vec![TabItem::new(TAB_BIO, requiem_i18n::t!("full-profile-bio"))];
    if server_count > 0 {
        items.push(TabItem::new(
            TAB_SERVERS,
            requiem_i18n::t!("mutual-servers", count = server_count),
        ));
    }
    let wanted = ctx.overlays.full_profile_ui.tab.get();
    let selected = items
        .iter()
        .find(|i| i.id == wanted)
        .map(|i| i.id.clone())
        .unwrap_or_else(|| TAB_BIO.to_string());
    let tab_signal = ctx.overlays.full_profile_ui.tab.clone();
    let bar = TabBarProps::default()
        .items(items)
        .selected(selected.clone())
        .on_select(Rc::new(move |id: String| {
            tab_signal.set(if id == TAB_SERVERS {
                TAB_SERVERS
            } else {
                TAB_BIO
            })
        }) as Rc<dyn Fn(String)>)
        .build();
    let content = if selected == TAB_SERVERS {
        mutual_servers(ctx)
    } else {
        let mut bio = scroll_view().class(cls::BOT_BIO);
        if !user.bio.trim().is_empty() {
            bio = bio.child(ui_profile_card::bio_text(&user.bio, 14.0, 18.0, ctx.images, ctx.backend));
        }
        if let Some(roles) = roles_section(ctx) {
            bio = bio.child(section(&requiem_i18n::t!("full-profile-roles"), roles));
        }
        bio.child(section(
            &requiem_i18n::t!("full-profile-created-on"),
            member_since(ctx),
        ))
        .child(section(
            &requiem_i18n::t!("full-profile-note-only-visible-to-you"),
            note_field(ctx),
        ))
    };
    let tabs = div()
        .class(cls::BOT_TABS)
        .child(div().class(cls::BOT_TAB_BAR).child(bar))
        .child(content);
    div()
        .class(cls::BOT_INNER)
        .child(header)
        .child(div().class(cls::BOT_BODY).child(info).child(tabs))
}

#[allow(clippy::too_many_arguments)]
pub fn build_block_modal(
    user_id: Id<UserMarker>,
    name: &str,
    location: &'static str,
    open: ToggleState,
    overlays: &ProfileOverlays,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    queries: &QueryClient,
    http: Option<&HttpClient>,
) -> Node {
    let close = {
        let overlays = overlays.clone();
        move || overlays.close_block_confirm()
    };
    let backdrop = ModalBackdropWidget::new(open).with_on_click(close.clone());

    let profile = queries
        .peek::<UserProfile>(&profile_popout::profile_query_key_for(
            profile_popout::ProfileSurface::Modal,
            user_id,
            None,
        ))
        .or_else(|| queries.peek::<UserProfile>(&profile_popout::profile_query_key(user_id, None)));
    let (discriminator, avatar_hash) = profile
        .as_ref()
        .map(|p| (p.user.discriminator.clone(), p.user.avatar.clone()))
        .unwrap_or_else(|| ("0".to_string(), None));
    let mut props = ui::user_avatar_props(user_id, &discriminator, fonts);
    if let Some(image) = avatar_hash
        .as_deref()
        .and_then(|hash| images.get_or_decode_image(&backend.avatar_url(user_id, hash)))
    {
        props = props.image(image);
    }
    let badge = div()
        .class(cls::BLOCK_BADGE)
        .child(icon(DENY, TEXT_SECONDARY).class(cls::BLOCK_BADGE_ICON));
    let avatar = div()
        .class(cls::BLOCK_AVATAR)
        .child(props.build().class(cls::BLOCK_AVATAR_IMAGE))
        .child(badge);
    let title = TextProps::new(
        requiem_i18n::t!("full-profile-block-title", name = name),
        24.0_f32,
    )
    .weight(Weight::BOLD)
    .color(TEXT_STRONG)
    .center(true)
    .build();
    let subtitle = TextProps::styled(
        requiem_i18n::t!("full-profile-stop-direct-contact-and-limit-what").to_string(),
        TextVariant::TEXT_MD_MEDIUM,
    )
    .color(TEXT_DEFAULT)
    .center(true)
    .build();
    let heading = StackProps::vertical()
        .align(Align::Center)
        .gap(16)
        .full_width(false)
        .build()
        .child(avatar)
        .child(div().class(cls::BLOCK_TITLES).child(title).child(subtitle));

    let cell = |glyph: Icon, title: &str, line: &str, last: bool| {
        let texts = div()
            .class(cls::BLOCK_CELL_TEXT)
            .child(
                TextProps::styled(title, TextVariant::TEXT_MD_SEMIBOLD)
                    .color(TEXT_STRONG)
                    .build(),
            )
            .child(
                TextProps::styled(line, TextVariant::TEXT_XS_MEDIUM)
                    .color(TEXT_SECONDARY)
                    .build(),
            );
        let mut node = StackProps::horizontal()
            .align(Align::Center)
            .gap(16)
            .full_width(false)
            .padding(Padding::y(16))
            .build()
            .child(icon(glyph, TEXT_SECONDARY).class(cls::BLOCK_CELL_ICON))
            .child(texts);
        if !last {
            node = node.class(cls::BLOCK_CELL_DIVIDED);
        }
        node
    };
    let cells = div()
        .class(cls::BLOCK_CELLS)
        .child(cell(
            CIRCLE_CHECK,
            &requiem_i18n::t!("full-profile-stop-direct-messages"),
            &requiem_i18n::t!("full-profile-you-can-unhide-past-messages-if"),
            false,
        ))
        .child(cell(
            CIRCLE_CHECK,
            &requiem_i18n::t!("full-profile-limit-access-to-your-profile-and"),
            &requiem_i18n::t!("full-profile-they-won-t-see-your-mutual"),
            false,
        ))
        .child(cell(
            WARNING,
            &requiem_i18n::t!("full-profile-they-can-interact-with-you-in"),
            &requiem_i18n::t!("full-profile-if-you-join-a-voice-channel"),
            true,
        ));

    let ignore = {
        let (overlays, queries, http) = (overlays.clone(), queries.clone(), http.cloned());
        ButtonProps::default()
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Small)
            .label(requiem_i18n::t!("full-profile-view-details").to_string())
            .on_click(move || {
                overlays.close_block_confirm();
                let Some(http) = http.clone() else { return };
                queries.mutate(Vec::new(), move || async move {
                    relationships::ignore(&http, user_id, location).await
                });
            })
            .build()
    };
    let ignore_text = div()
        .class(cls::BLOCK_CELL_TEXT)
        .child(
            TextProps::styled(
                requiem_i18n::t!("full-profile-ignore-instead"),
                TextVariant::TEXT_MD_MEDIUM,
            )
            .color(TEXT_STRONG)
            .build(),
        )
        .child(
            TextProps::styled(
                requiem_i18n::t!("full-profile-get-space-from-them-without-letting").to_string(),
                TextVariant::TEXT_XS_MEDIUM,
            )
            .color(TEXT_SECONDARY)
            .build(),
        );
    let ignore_card = div().class(cls::BLOCK_IGNORE_CARD).child(
        div()
            .class(cls::BLOCK_IGNORE_ROW)
            .child(ignore_text)
            .child(ignore),
    );
    let other = StackProps::vertical()
        .full_width(false)
        .build()
        .child(
            TextProps::styled(
                requiem_i18n::t!("full-profile-too-much"),
                TextVariant::TEXT_SM_SEMIBOLD,
            )
            .color(TEXT_DEFAULT)
            .build(),
        )
        .child(ignore_card);
    let guide = TextLinkProps::default()
        .label(requiem_i18n::t!("full-profile-explore-feature-guide").to_string())
        .variant(TextLinkVariant::Anchor)
        .on_click(|| link::open_url(BLOCK_GUIDE_URL))
        .build();
    let content = scroll_view().class(cls::BLOCK_CONTENT).child(
        StackProps::vertical()
            .gap(16)
            .full_width(false)
            .padding(Padding::top(20))
            .build()
            .child(heading)
            .child(cells)
            .child(other)
            .child(
                StackProps::horizontal()
                    .justify(Justify::Center)
                    .gap(0)
                    .full_width(false)
                    .build()
                    .child(guide),
            ),
    );

    let block = {
        let (overlays, queries, http) = (overlays.clone(), queries.clone(), http.cloned());
        ButtonProps::default()
            .variant(ButtonVariant::CriticalPrimary)
            .label(requiem_i18n::t!("full-profile-block").to_string())
            .on_click(move || {
                overlays.close_block_confirm();
                let Some(http) = http.clone() else { return };
                queries.mutate(Vec::new(), move || async move {
                    relationships::block(&http, user_id, location).await
                });
            })
            .build()
    };
    let cancel = ButtonProps::default()
        .variant(ButtonVariant::Secondary)
        .label(requiem_i18n::t!("full-profile-cancel").to_string())
        .on_click(close)
        .build();
    let footer = div().class(cls::BLOCK_FOOTER).child(cancel).child(block);
    let panel = Node::new("div", ModalPanelWidget::new(open))
        .class(cls::BLOCK_PANEL)
        .child(content)
        .child(footer);
    Node::new("div", backdrop)
        .class(modal::MODAL_BACKDROP)
        .child(panel)
}

pub struct PluginProfile {
    pub banner: Option<Node>,

    pub icon: Option<Node>,
    pub acronym: String,
    pub name: String,
    pub subtitle: String,

    pub stats: Vec<(String, bool)>,
    pub badges: Vec<String>,

    pub body: Vec<Node>,

    pub tabs: Vec<(TabItem, Node)>,
    pub selected: String,
    pub on_tab: Rc<dyn Fn(String)>,
}

pub fn plugin_profile(spec: PluginProfile) -> Node {
    let banner = match spec.banner {
        Some(node) => node.class(cls::BANNER),
        None => div()
            .class(cls::BANNER)
            .style("background-color: var(--background-brand)"),
    };

    let label = TextProps::new(spec.acronym, 36.0_f32)
        .weight(Weight::MEDIUM)
        .color(Color::WHITE)
        .center(true)
        .line_clamp(1_u32)
        .build();
    let mut header = div()
        .class(cls::HEADER)
        .child(banner)
        .child(div().class(cls::SERVER_RING))
        .child(
            div()
                .class(cls::SERVER_ICON)
                .class(cls::SERVER_ACRONYM)
                .child(label),
        );
    if let Some(picture) = spec.icon {
        header = header.child(picture.class(cls::SERVER_ICON));
    }

    let name = TextProps::new(spec.name, 24.0_f32)
        .weight(Weight::SEMI_BOLD)
        .color(TEXT_DEFAULT)
        .build();

    let mut name_row = StackProps::horizontal()
        .align(Align::Center)
        .gap(8)
        .full_width(false)
        .build();
    for badge in &spec.badges {
        name_row = match badge.as_str() {
            "verified" => name_row.child(
                TooltipProps::default()
                    .text("Verified")
                    .build(channel_sidebar::verified_badge()),
            ),
            "partnered" => name_row.child(
                TooltipProps::default()
                    .text("Partnered")
                    .build(channel_sidebar::partnered_badge()),
            ),
            _ => name_row,
        };
    }
    let name_row = name_row.child(name);

    let mut username_row = div().class(cls::USERNAME_ROW);
    if !spec.subtitle.is_empty() {
        username_row = username_row.child(
            TextProps::styled(spec.subtitle, TextVariant::TEXT_SM_NORMAL)
                .color(TEXT_STRONG)
                .build(),
        );
    }

    let chips = spec
        .badges
        .into_iter()
        .filter(|b| b != "verified" && b != "partnered")
        .map(|badge| {
            div().class(cls::PLUGIN_BADGE).child(
                TextProps::styled(badge, TextVariant::TEXT_XS_NORMAL)
                    .color(TEXT_DEFAULT)
                    .build(),
            )
        });
    if !spec.stats.is_empty() {
        let stats = spec.stats.into_iter().map(|(text, online)| {
            let dot = div().class(cls::PLUGIN_STAT_DOT).class(if online {
                cls::PLUGIN_STAT_ONLINE
            } else {
                cls::PLUGIN_STAT_OFFLINE
            });
            div().class(cls::PLUGIN_STAT).child(dot).child(
                TextProps::styled(text, TextVariant::TEXT_SM_NORMAL)
                    .color(TEXT_STRONG)
                    .build(),
            )
        });
        username_row = username_row.child(div().class(cls::PLUGIN_STATS).children(stats));
    }
    username_row = username_row.children(chips);
    let names = StackProps::vertical()
        .gap(0)
        .full_width(false)
        .build()
        .child(name_row)
        .child(username_row);
    let body = scroll_view()
        .class(cls::BODY)
        .child(names)
        .children(spec.body);
    let card = div().class(cls::CARD).child(header).child(body);

    let items: Vec<TabItem> = spec.tabs.iter().map(|(item, _)| item.clone()).collect();
    let selected = items
        .iter()
        .find(|i| i.id == spec.selected)
        .or(items.first())
        .map(|i| i.id.clone())
        .unwrap_or_default();
    let panel = spec
        .tabs
        .into_iter()
        .find(|(item, _)| item.id == selected)
        .map(|(_, panel)| panel);
    let mut column = div().class(cls::TABS_COLUMN);
    if !items.is_empty() {
        let on_tab = spec.on_tab;
        let bar = TabBarProps::default()
            .items(items)
            .selected(selected)
            .on_select(Rc::new(move |id: String| on_tab(id)) as Rc<dyn Fn(String)>)
            .build();
        column = column.child(div().class(cls::TAB_BAR).child(bar));
    }
    if let Some(panel) = panel {
        column = column.child(div().class(cls::TAB_PANEL).child(panel));
    }
    div()
        .class(cls::INNER)
        .child(div().class(cls::COLUMNS).child(card).child(column))
}

pub fn plugin_profile_modal(content: Node, open: ToggleState, on_close: Rc<dyn Fn()>) -> Node {
    let dismiss = on_close.clone();
    let backdrop = ModalBackdropWidget::new(open).with_on_click(move || dismiss());
    let close = ButtonProps::default()
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Small)
        .icon(X_SMALL)
        .on_click(move || on_close())
        .build();
    let panel = Node::new("div", ModalPanelWidget::new(open))
        .class(cls::PANEL_BASE)
        .class(cls::PANEL)
        .child(content)
        .child(div().class(cls::CLOSE).child(close));
    Node::new("div", backdrop)
        .class(modal::MODAL_BACKDROP)
        .child(panel)
}
