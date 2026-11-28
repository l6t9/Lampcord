use crate::ui;
use crate::ui::classes::member_list as member_list_cls;
use crate::ui::classes::dm_profile as cls;
use crate::models;
use crate::rest::endpoints::relationships::location;
use crate::ui::channel_sidebar;
use crate::ui::classes::profile_popout;
use crate::ui::profile_card;
use crate::ui::profile_popout::ProfileSurface;
use crate::utils::guild;
use crate::utils::time;
use requiem_ui::components::{
    AvatarStatusSize, ButtonProps, ButtonSize, ButtonVariant, ScrollViewProps, TextProps,
    TextVariant, TooltipProps, avatar_with_status, div, icon, popover_backdrop,
};
use requiem_ui::icons;
use requiem_ui::node::Node;
use requiem_ui::widget::primitives::{ImageWidget, PresenceStatus};
use skia_safe::font_style::Weight;

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::query::QueryClient;
use crate::rest::HttpClient;
use crate::rest::endpoints::relationships::FRIEND;
use crate::state::{AppState, DmChannel};
use crate::ui::member_list::MemberListState;
use crate::ui::profile_popout::{
    ColorBoxWidget, FullProfileTab, FullProfileTarget, ProfilePopoutState, badges_row,
    banner_buttons, banner_color, friend_menu, load_profile, more_menu, separator_dot,
};
use crate::ui::{Fonts, TEXT_SECONDARY, TEXT_STRONG, TEXT_SUBTLE, user_avatar_props};

#[allow(clippy::too_many_arguments)]
pub fn build(
    state: &AppState,
    dm: &DmChannel,
    ui: &MemberListState,
    popout: &ProfilePopoutState,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    queries: &QueryClient,
    http: Option<&HttpClient>,
) -> Node {
    if dm.is_group() {
        return build_group(state, dm, popout, fonts, images, backend);
    }
    let Some(user_id) = dm.recipients.first().copied() else {
        return div();
    };
    let panel = div().class(cls::PANEL);
    let Some(profile) = load_profile(
        queries,
        http,
        &state.users,
        user_id,
        None,
        ProfileSurface::Sidebar,
    ) else {
        return panel;
    };

    let banner_image =
        profile.user.banner.as_deref().and_then(|hash| {
            images.get_or_decode_image(&backend.banner_url(profile.user.id, hash))
        });
    let banner = match banner_image {
        Some(image) => Node::new(
            "div",
            ImageWidget::new(Some(image)).with_cover_centered(),
        )
        .class(cls::BANNER),
        None => {
            let color = banner_color(
                profile.user.id,
                &profile.user.discriminator,
                profile.user.accent_color,
                profile.user.avatar.as_deref(),
                images,
                backend,
            );
            Node::new("div", ColorBoxWidget { color }).class(cls::BANNER)
        }
    };

    let mut avatar_props = ui::with_avatar_decoration(
        user_avatar_props(profile.user.id, &profile.user.discriminator, fonts),
        profile.user.avatar_decoration_asset(),
        images,
        backend,
        80,
    );
    if let Some(image) = profile
        .user
        .avatar
        .as_deref()
        .and_then(|hash| images.get_or_decode_image(&backend.avatar_url(profile.user.id, hash)))
    {
        avatar_props = avatar_props.image(image);
    }
    let status = state.user_presence.of(user_id).status;
    let avatar = avatar_with_status(avatar_props, AvatarStatusSize::Px80, Some(status));

    let relationship = state.relationships.kind_of(user_id);
    let display_name = profile
        .user
        .global_name
        .clone()
        .unwrap_or_else(|| profile.user.username.clone());
    let header = div().class(cls::HEADER).children([
        banner,
        div().class(cls::AVATAR_RING),
        div().class(cls::AVATAR).child(avatar),
        banner_buttons(
            user_id,
            !profile.user.bot && relationship.is_none(),
            relationship == Some(FRIEND),
            queries,
            http,
            popout,
            location::PROFILE_SIDEBAR,
        ),
    ]);

    let name = TextProps::new(display_name.clone(), 20.0_f32)
        .weight(Weight::BOLD)
        .color(TEXT_STRONG)
        .line_clamp(1_u32)
        .build();
    let mut username_row = div().class(cls::USERNAME_ROW).child(
        TextProps::styled(profile.user.username.clone(), TextVariant::TEXT_SM_NORMAL)
            .color(TEXT_STRONG)
            .build(),
    );
    if let Some(pronouns) = profile
        .user_profile
        .as_ref()
        .map(|meta| meta.pronouns.trim())
        .filter(|p| !p.is_empty())
    {
        username_row = username_row.child(separator_dot(TEXT_STRONG)).child(
            TextProps::styled(pronouns.to_string(), TextVariant::TEXT_SM_NORMAL)
                .color(TEXT_STRONG)
                .line_clamp(1_u32)
                .build(),
        );
    }
    if let Some(badges) = badges_row(&profile, images, backend) {
        username_row = username_row.child(badges);
    }

    let note_overlays = popout.overlays.clone();
    let note = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![
            profile_popout::NOTE_BTN.to_string(),
        ])
        .on_click(move || {
            note_overlays.open_full_profile(FullProfileTarget {
                user_id,
                guild_id: None,
                focus_note: true,
                tab: FullProfileTab::First,
                server_guild_id: None,
            })
        })
        .build()
        .child(
            icon(icons::PAPER_PLUS, TEXT_SECONDARY)
                .class(profile_popout::NOTE_ICON),
        );
    let note = TooltipProps::default()
        .text(requiem_i18n::t!("full-profile-note-only-visible-to-you").to_string())
        .build(note);
    let name_row = div().class(cls::NAME_ROW).children([name, note]);
    let info = div().class(cls::INFO).children([name_row, username_row]);

    let section = |title: String, body: Node| {
        div().class(cls::SECTION).children([
            TextProps::styled(title, TextVariant::TEXT_XS_SEMIBOLD)
                .color(TEXT_STRONG)
                .build(),
            body,
        ])
    };
    let mut card = div().class(cls::CARD);
    if !profile.user.bio.trim().is_empty() {
        card = card.child(section(
            requiem_i18n::t!("full-profile-bio").to_string(),
            profile_card::bio_text(&profile.user.bio, 14.0, 18.0, images, backend),
        ));
    }
    card = card.child(section(
        requiem_i18n::t!("full-profile-member-since").to_string(),
        TextProps::styled(
            time::date_label(user_id.created_at_millis()),
            TextVariant::TEXT_SM_NORMAL,
        )
        .color(TEXT_STRONG)
        .build(),
    ));

    let mut body = div().class(cls::BODY).children([info, card]);
    let mutual_guilds: Vec<_> = profile
        .mutual_guilds
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter_map(|m| state.guilds.find(m.id))
        .collect();
    if !mutual_guilds.is_empty() {
        body = body.child(mutual_servers(&mutual_guilds, ui, popout, images, backend));
    }

    let open = popout.overlays.clone();
    let footer = div().class(cls::FOOTER).child(
        ButtonProps::default()
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Medium)
            .full_width(true)
            .label(requiem_i18n::t!("profile-popout-view-full-profile").to_string())
            .on_click(move || {
                open.open_full_profile(FullProfileTarget {
                    user_id,
                    guild_id: None,
                    focus_note: false,
                    tab: FullProfileTab::First,
                    server_guild_id: None,
                })
            })
            .build(),
    );

    let mut panel = panel.children([header, div().class(cls::SCROLL).child(body), footer]);
    if popout.overlays.friend_menu_open.get() {
        panel = panel.children(friend_menu(
            user_id,
            queries,
            http,
            popout,
            location::PROFILE_SIDEBAR,
        ));
    }
    if popout.overlays.more_menu_open.get() {
        let (open, close) = (
            popout.overlays.clone(),
            popout.overlays.more_menu_open.clone(),
        );
        let open_full = move |focus_note: bool| {
            open.more_menu_open.set(false);
            open.open_full_profile(FullProfileTarget {
                user_id,
                guild_id: None,
                focus_note,
                tab: FullProfileTab::First,
                server_guild_id: None,
            });
        };
        panel = panel.child(popover_backdrop(move || close.set(false)));
        panel = panel.child(
            more_menu(
                user_id,
                &display_name,
                relationship,
                open_full,
                queries,
                http,
                popout,
            )
            .class(profile_popout::MORE_MENU_POS),
        );
    }
    panel
}

fn mutual_servers(
    guilds: &[models::Guild],
    ui: &MemberListState,
    popout: &ProfilePopoutState,
    images: &ImageCache,
    backend: &BackendConfig,
) -> Node {
    let expanded = ui.dm_mutuals_open.get();
    let toggle = ui.dm_mutuals_open.clone();
    let chevron = if expanded {
        icons::CHEVRON_SMALL_DOWN
    } else {
        icons::CHEVRON_SMALL_RIGHT
    };
    let header = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::MUTUALS_HEADER.to_string()])
        .on_click(move || toggle.set(!toggle.peek()))
        .build()
        .children([
            TextProps::styled(
                requiem_i18n::t!("dm-profile-mutual-servers", count = guilds.len()),
                TextVariant::TEXT_SM_MEDIUM,
            )
            .color(TEXT_STRONG)
            .build()
            .class(cls::MUTUALS_TEXT),
            icon(chevron, TEXT_SECONDARY).class(cls::MUTUALS_CHEVRON),
        ]);
    let mut section = div().class(cls::MUTUALS).child(header);
    if expanded {
        let mut list = div().class(cls::MUTUALS_LIST);
        for guild in guilds {
            let (on_open, guild_id) = (popout.overlays.on_open_guild.clone(), guild.id);
            let mut tile = match guild.icon.as_deref().and_then(|hash| {
                images.get_or_decode_image(&backend.guild_icon_url(guild.id, hash))
            }) {
                Some(image) => {
                    Node::new("div", ImageWidget::new(Some(image))).class(cls::GUILD_ICON)
                }
                None => div()
                    .class(cls::GUILD_ICON)
                    .class(cls::GUILD_ACRONYM)
                    .child(
                        TextProps::new(guild::acronym(&guild.name), 14.0_f32)
                            .weight(Weight::MEDIUM)
                            .color(skia_safe::Color::WHITE)
                            .center(true)
                            .line_clamp(1_u32)
                            .build(),
                    ),
            };
            if guild.is_verified() {
                tile = tile.child(
                    div()
                        .class(cls::GUILD_BADGE)
                        .child(channel_sidebar::verified_badge()),
                );
            }
            list = list.child(
                ButtonProps::default()
                    .variant(ButtonVariant::Bare)
                    .extra_classes(vec![cls::GUILD_ROW.to_string()])
                    .on_click(move || on_open(guild_id))
                    .build()
                    .children([
                        tile,
                        TextProps::styled(guild.name.clone(), TextVariant::TEXT_MD_MEDIUM)
                            .color(TEXT_STRONG)
                            .line_clamp(1_u32)
                            .build()
                            .class(cls::GUILD_NAME),
                    ]),
            );
        }
        section = section.child(list);
    }
    section
}

fn build_group(
    state: &AppState,
    dm: &DmChannel,
    popout: &ProfilePopoutState,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
) -> Node {
    let me = state
        .users
        .current
        .with_untracked(|u| u.as_ref().map(|u| u.id));
    let mut ids = dm.recipients.clone();
    ids.extend(me.filter(|id| !ids.contains(id)));
    let label = TextProps::styled(
        requiem_i18n::t!(
            "member-list-group-header",
            name = requiem_i18n::t!("dm-profile-members"),
            count = ids.len()
        ),
        TextVariant::TEXT_SM_MEDIUM,
    )
    .color(TEXT_SUBTLE)
    .line_clamp(1_u32)
    .build()
    .class(member_list_cls::GROUP_LABEL);
    let mut column = div()
        .class(member_list_cls::LIST)
        .child(div().class(member_list_cls::GROUP).child(label));
    for id in ids {
        let Some(user) = state.users.get(id) else {
            continue;
        };
        let status = state.user_presence.of(id).status;
        let offline = status == PresenceStatus::Offline;
        let mut props = user_avatar_props(user.id, &user.discriminator, fonts);
        if let Some(image) = user
            .avatar
            .as_deref()
            .and_then(|hash| images.get_or_decode_image(&backend.avatar_url(user.id, hash)))
        {
            props = props.image(image);
        }

        let avatar =
            avatar_with_status(props, AvatarStatusSize::Px32, (!offline).then_some(status))
                .class(member_list_cls::AVATAR_WRAP);
        let name = TextProps::styled(state.users.display_name(&user), TextVariant::TEXT_MD_MEDIUM)
            .line_clamp(1_u32)
            .color(TEXT_SUBTLE)
            .build()
            .class(member_list_cls::NAME);
        let inner = div().class(member_list_cls::ROW_INNER).child(avatar).child(
            div()
                .class(member_list_cls::NAMES)
                .child(div().class(member_list_cls::NAME_ROW).child(name)),
        );
        let overlays = popout.overlays.clone();
        let mut row = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![member_list_cls::ROW.to_string()])
            .on_click(move || {
                overlays.open_full_profile(FullProfileTarget {
                    user_id: id,
                    guild_id: None,
                    focus_note: false,
                    tab: FullProfileTab::First,
                    server_guild_id: None,
                })
            })
            .build()
            .child(inner);
        if offline {
            row = row.class(member_list_cls::ROW_OFFLINE);
        }
        column = column.child(row);
    }
    div().class(member_list_cls::PANEL).child(
        ScrollViewProps::default()
            .build()
            .class(member_list_cls::SCROLL)
            .child(column),
    )
}
