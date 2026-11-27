use requiem_core::ui::message_list;
use crate::shell::classes::guild_settings as classes_guild_settings;
use crate::shell::classes::guild_settings as c;
use crate::shell::classes::profile_settings as bar;
use crate::shell;
use requiem_core::rest::endpoints::guilds;
use requiem_core::ui::emoji_button_icon::EmojiButtonIconWidget;
use requiem_core::ui::message_list::PreviewMessage;
use requiem_core::ui::profile_image;
use requiem_core::ui;
use requiem_core::utils::guild;
use requiem_core::utils::time;
use requiem_ui::colors::{DEFAULT_ROLE, TEXT_CRITICAL, TEXT_SUBTLE};
use requiem_ui::components::TextVariant;
use requiem_ui::components::{Align, Justify, Padding, StackProps};
use requiem_ui::components::{ButtonSize, ButtonVariant, ModalSize};
use requiem_ui::components::{
    HelpMessageProps, IconPosition, TextLinkProps, line_clamp_class, text_area,
};
use requiem_ui::widget::primitives::ImageWidget;
use requiem_ui::widget::primitives::PopoverPanelWidget;
use skia_safe::gradient::Interpolation;
use skia_safe::gradient::shaders;
use std::cell::Cell;
use std::cell::RefCell;
use std::env;
use std::path::PathBuf;
use std::rc::Rc;

use requiem_ui::components::{
    ButtonProps, CheckboxProps, ColorPickerProps, FormItemProps, FormSectionProps, HeadingLevel,
    HeadingProps, InlineNoticeProps, ModalProps, SwitchProps, TabBarProps, TabItem, TextInputProps,
    TextProps, div, form_section, icon, rgb_color, scroll_view, search_bar,
};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::anim::ToggleState;
use requiem_ui::widget::primitives::{corner_radii, has_any_corner_radius};
use requiem_ui::widget::{AbsoluteRect, Widget};
use skia_safe::font_style::Weight;
use skia_safe::{Canvas, Color, Matrix, Paint};

use requiem_core::config::BackendConfig;
use requiem_core::images::ImageCache;
use requiem_core::models::id::marker::{GuildMarker, RoleMarker, UserMarker};
use requiem_core::models::{Guild, Id, Permissions, Role, RoleStyle};
use requiem_core::query::QueryClient;
use requiem_core::rest::HttpClient;
use requiem_core::rest::endpoints::guilds::RoleEdit;
use requiem_core::state::AppState;
use requiem_core::ui::context_menu::{self as menu, ConfirmRequest};
use requiem_core::ui::guild_settings::{self as gs, Dialog, RoleTab};
use requiem_core::ui::permission_spec;
use requiem_core::ui::{Fonts, PendingImage, TEXT_DEFAULT, TEXT_SECONDARY, TEXT_STRONG};
use requiem_ui::components::ContextItem;

use requiem_ui::colors::TEXT_MUTED;
use requiem_ui::icons::{
    ARROW_LARGE_LEFT, BOOST_GEM, CHECKMARK_LARGE, CHEVRON_LARGE_RIGHT, CIRCLE_X, DRAG, EYE_DROPPER,
    GROUP, IMAGE_PLUS, MORE_HORIZONTAL, PLUS_LARGE, SHIELD_USER, TRASH, USER, X_LARGE,
};

const NAV: &[(Option<&str>, &[&str])] = &[
    (
        None,
        &[
            requiem_i18n::key!("guild-settings-server-profile"),
            requiem_i18n::key!("guild-settings-server-tag"),
            requiem_i18n::key!("guild-settings-engagement"),
            requiem_i18n::key!("guild-settings-boost-perks"),
        ],
    ),
    (
        Some(requiem_i18n::key!("guild-settings-expression")),
        &[
            requiem_i18n::key!("guild-settings-emoji"),
            requiem_i18n::key!("guild-settings-stickers"),
            requiem_i18n::key!("guild-settings-soundboard"),
        ],
    ),
    (
        Some(requiem_i18n::key!("guild-settings-people")),
        &[
            requiem_i18n::key!("guild-settings-roles"),
            requiem_i18n::key!("guild-settings-invites"),
            requiem_i18n::key!("guild-settings-access"),
        ],
    ),
    (
        Some(requiem_i18n::key!("guild-settings-apps")),
        &[
            requiem_i18n::key!("guild-settings-integrations"),
            requiem_i18n::key!("guild-settings-app-directory"),
        ],
    ),
    (
        Some(requiem_i18n::key!("guild-settings-moderation")),
        &[
            requiem_i18n::key!("guild-settings-safety-setup"),
            requiem_i18n::key!("guild-settings-audit-log"),
            requiem_i18n::key!("guild-settings-bans"),
            requiem_i18n::key!("guild-settings-automod"),
        ],
    ),
    (
        None,
        &[requiem_i18n::key!("guild-settings-enable-community")],
    ),
    (
        None,
        &[requiem_i18n::key!("guild-settings-server-template")],
    ),
];

const DELETE_SERVER: &str = requiem_i18n::key!("guild-settings-delete-server");

thread_local! {

    static ROLE_QUERY: Signal<String> = signal(String::new());
    static PERMISSION_QUERY: Signal<String> = signal(String::new());
    static MEMBER_QUERY: Signal<String> = signal(String::new());
    static ADD_QUERY: Signal<String> = signal(String::new());

    static PICKED: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

#[derive(Clone)]
pub struct Env {
    pub state: AppState,
    pub fonts: Fonts,
    pub images: ImageCache,
    pub backend: BackendConfig,
    pub queries: QueryClient,
    pub http: HttpClient,
    pub tick: Signal<u64>,
}

fn role_color(role_color: i64) -> Color {
    if role_color == 0 {
        DEFAULT_ROLE
    } else {
        rgb_color(role_color as u32)
    }
}

fn solid(color: Color) -> String {
    format!(
        "background-color: #{:02x}{:02x}{:02x}",
        color.r(),
        color.g(),
        color.b()
    )
}

pub fn build(env: &Env, _open: ToggleState, on_dismiss: Rc<dyn Fn()>) -> Node {
    let s = gs::state();
    let Some(guild_id) = s.guild.get() else {
        return div();
    };
    let Some(guild) = env
        .state
        .guilds
        .list
        .with(|guilds| guilds.iter().find(|g| g.id == guild_id).cloned())
    else {
        return div();
    };
    if !gs::can_access(&env.state, &guild) {
        return div();
    }
    let mut page = s.page.get();
    if !gs::can_see_page(&env.state, &guild, &page) {
        page = gs::SERVER_PROFILE_PAGE.to_string();
    }
    let editing = s.role.get();
    let _ = s.epoch.get();

    let sidebar = div()
        .class(c::SIDEBAR_REGION)
        .child(build_nav(env, &guild, &page));
    let body = if page == gs::ROLES_PAGE {
        match editing {
            Some(role_id) => role_editor(env, &guild, role_id),
            None => content_scroll(roles_page(env, &guild)),
        }
    } else if page == gs::SERVER_PROFILE_PAGE {
        content_scroll(profile_page(env, &guild))
    } else {
        content_scroll(placeholder_page(&page))
    };
    let mut content = div()
        .class(c::CONTENT_REGION)
        .child(div().class(c::CONTENT_COLUMN).child(body));
    if let Some(bar) = save_bar(env, &guild, &page, editing.is_some()) {
        content = content.child(bar);
    }

    let overlay = !(page == gs::ROLES_PAGE && editing.is_some());
    content = content.child(close_button(on_dismiss, overlay));
    div().class(c::LAYER).children([sidebar, content])
}

fn content_scroll(page: Node) -> Node {
    scroll_view().class(c::CONTENT_SCROLL).child(page)
}

fn close_button(on_dismiss: Rc<dyn Fn()>, overlay: bool) -> Node {
    let ring = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![c::CLOSE_RING.to_string()])
        .on_click(move || on_dismiss())
        .build()
        .child(icon(X_LARGE, TEXT_SECONDARY).class(c::CLOSE_ICON));
    div()
        .class(if overlay { c::TOOLS_OVERLAY } else { c::TOOLS })
        .child(
            StackProps::vertical()
                .align(Align::Center)
                .gap(0)
                .full_width(false)
                .build()
                .children([
                    ring,
                    TextProps::styled(
                        requiem_i18n::t!("guild-settings-esc"),
                        TextVariant::TEXT_XS_SEMIBOLD,
                    )
                    .color(TEXT_SECONDARY)
                    .build()
                    .class(c::CLOSE_LABEL),
                ]),
        )
}

fn build_nav(env: &Env, guild: &Guild, page: &str) -> Node {
    let on_delete_server: Rc<dyn Fn()> = {
        let (env, guild_id) = (env.clone(), guild.id);
        Rc::new(move || confirm_delete_server(&env, guild_id))
    };
    let heading = |text: &str, first: bool| {
        div()
            .class(if first {
                c::NAV_HEADING_FIRST
            } else {
                c::NAV_HEADING
            })
            .child(
                TextProps::styled(text.to_uppercase(), TextVariant::TEXT_XS_BOLD)
                    .color(TEXT_SUBTLE)
                    .build(),
            )
    };

    let mut children = vec![heading(&guild.name, true)];
    for (index, (group, pages)) in NAV.iter().enumerate() {
        let pages: Vec<_> = pages
            .iter()
            .copied()
            .filter(|name| gs::can_see_page(&env.state, guild, name))
            .collect();
        if pages.is_empty() {
            continue;
        }
        if index > 0 && !children.is_empty() {
            children.push(div().class(c::NAV_SEPARATOR));
        }
        if let Some(group) = group {
            children.push(heading(&requiem_i18n::text(group), false));
        }
        for name in pages {
            children.push(nav_item(env, guild, name, name == page, false, &on_delete_server));
        }
    }
    let owner = env
        .state
        .users
        .current
        .with_untracked(|u| u.as_ref().map(|u| u.id))
        == guild.owner_id
        && guild.owner_id.is_some();
    if owner {
        children.push(div().class(c::NAV_SEPARATOR));
        children.push(nav_item(env, guild, DELETE_SERVER, false, true, &on_delete_server));
    }
    scroll_view().class(c::NAV).child(
        StackProps::vertical()
            .gap(0)
            .full_width(false)
            .padding(Padding::bottom(24))
            .build()
            .children(children),
    )
}

fn nav_item(
    env: &Env,
    guild: &Guild,
    name: &'static str,
    active: bool,
    danger: bool,
    on_delete_server: &Rc<dyn Fn()>,
) -> Node {
    let on_delete_server = on_delete_server.clone();
    let color = if danger {
        TEXT_CRITICAL
    } else if active {
        TEXT_STRONG
    } else {
        TEXT_SECONDARY
    };
    let mut classes = vec![c::NAV_ITEM.to_string()];
    if active {
        classes.push(c::NAV_ITEM_ACTIVE.to_string());
    }
    let (state, guild_id) = (env.state.clone(), guild.id);
    ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(classes)
        .on_click(move || {
            if name == DELETE_SERVER {
                on_delete_server();
            } else if state
                .guilds
                .find(guild_id)
                .as_ref()
                .is_some_and(|guild| gs::can_see_page(&state, guild, name))
            {
                gs::go_to_page(name);
            }
        })
        .build()
        .child(
            TextProps::styled(requiem_i18n::text(name), TextVariant::TEXT_MD_MEDIUM)
                .color(color)
                .build(),
        )
}

fn placeholder_page(page: &str) -> Node {
    div().class(c::PAGE).children([
        HeadingProps::new(HeadingLevel::H1, requiem_i18n::text(page))
            .variant(TextVariant::HEADING_LG_SEMIBOLD)
            .build()
            .class(classes_guild_settings::PAGE_TITLE),
        TextProps::styled(
            requiem_i18n::t!("guild-settings-this-settings-page-hasn-t-been"),
            TextVariant::TEXT_MD_NORMAL,
        )
        .color(TEXT_SECONDARY)
        .build(),
    ])
}

fn pick_icon() -> Option<ui::PendingUpload> {
    let picked = match env::var_os("REQUIEM_PICK_IMAGE") {
        Some(path) => Some(PathBuf::from(path)),
        None => {
            let picked = rfd::FileDialog::new()
                .set_title(requiem_i18n::t!("guild-settings-choose-an-icon"))
                .add_filter(
                    requiem_i18n::t!("dialogs-images"),
                    &["png", "jpg", "jpeg", "gif", "webp"],
                )
                .pick_file();
            shell::note_native_dialog_closed();
            picked
        }
    }?;
    match profile_image::prepare_guild_image(&picked, false) {
        Ok(upload) => Some(upload),
        Err(err) => {
            log::warn!("couldn't use {}: {err}", picked.display());
            None
        }
    }
}

struct RadialBox {
    from: Color,
    to: Color,
}

impl Widget for RadialBox {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let (w, h) = (rect.width, rect.height);
        if w <= 0.0 || h <= 0.0 {
            return;
        }

        let mut matrix = Matrix::new_identity();
        matrix.pre_translate((rect.x + 0.501 * w, rect.y + 1.2705 * h));
        matrix.pre_scale((1.0543 * w, 1.2705 * h), None);
        let colors: Vec<skia_safe::Color4f> = [self.from, self.from, self.to, self.to]
            .iter()
            .map(|c| skia_safe::Color4f::from(*c))
            .collect();
        let stops = [0.0, 0.2065, 0.8516, 1.0];
        let gradient = skia_safe::gradient::Gradient::new(
            skia_safe::gradient::Colors::new(
                &colors,
                Some(&stops[..]),
                skia_safe::TileMode::Clamp,
                None,
            ),
            Interpolation::default(),
        );
        let Some(shader) = shaders::radial_gradient(
            ((0.0, 0.0), 1.0),
            &gradient,
            Some(&matrix),
        ) else {
            return;
        };
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_shader(shader);
        let bounds = skia_safe::Rect::from_xywh(rect.x, rect.y, w, h);
        if has_any_corner_radius(style) {
            canvas.draw_rrect(
                skia_safe::RRect::new_rect_radii(bounds, &corner_radii(style)),
                &paint,
            );
        } else {
            canvas.draw_rect(bounds, &paint);
        }
    }
}

fn radial(swatch: usize) -> Node {
    let (_, from, to) = gs::BRAND_SWATCHES[swatch];
    Node::new(
        "div",
        RadialBox {
            from: Color::from_rgb(from[0], from[1], from[2]),
            to: Color::from_rgb(to[0], to[1], to[2]),
        },
    )
}

fn profile_page(env: &Env, guild: &Guild) -> Node {
    let f = &env.fonts;
    let s = gs::state();
    let draft = s.profile.clone();
    let can_edit = env
        .state
        .permissions
        .guild_permissions(guild)
        .contains(Permissions::MANAGE_GUILD);

    let profile = gs::profile(&env.queries, &env.http, guild.id);
    match &profile {
        Some(profile) => gs::seed_profile(profile),
        None => {
            let _ = env.tick.get();
            return div().class(c::PAGE).child(
                TextProps::styled(
                    requiem_i18n::t!("guild-settings-loading"),
                    TextVariant::TEXT_MD_NORMAL,
                )
                .color(TEXT_SECONDARY)
                .build(),
            );
        }
    }
    let profile = profile.expect("checked above");
    let d = draft.peek();

    let input_props = |text: String| TextInputProps::new().text(text).disabled(!can_edit);

    let name = {
        let draft = draft.clone();
        input_props(d.name.clone())
            .max_length(100_usize)
            .on_change(move |text: &str| draft.update(|d| d.name = text.to_string()))
            .build()
    };
    let name = FormItemProps::new(requiem_i18n::t!("guild-settings-name"), name).build();

    let icon_button = {
        let draft = draft.clone();
        ButtonProps::default()
            .variant(ButtonVariant::Primary)
            .size(ButtonSize::Small)
            .label(requiem_i18n::t!("guild-settings-change-server-icon").to_string())
            .disabled(!can_edit)
            .on_click(move || {
                if let Some(upload) = pick_icon() {
                    draft.update(|d| d.icon = PendingImage::Set(upload));
                }
            })
            .build()
    };
    let icon_field = FormItemProps::new(
        requiem_i18n::t!("guild-settings-icon"),
        StackProps::horizontal()
            .full_width(false)
            .build()
            .child(icon_button),
    )
    .description(requiem_i18n::t!(
        "guild-settings-we-recommend-an-image-of-at"
    ))
    .build();

    let banner = {
        let (draft, fonts) = (draft.clone(), f.clone());
        Node::reactive(move || {
            let chosen = draft.get().brand;
            let mut grid = StackProps::horizontal()
                .wrap(true)
                .full_width(false)
                .build();
            for (index, (name, ..)) in gs::BRAND_SWATCHES.iter().enumerate() {
                let draft = draft.clone();
                let mut swatch = div().class(c::SWATCH_CELL);
                swatch = swatch.child(radial(index).class(c::SWATCH_FILL));
                if index == chosen {
                    swatch = swatch.child(div().class(c::SWATCH_RING));
                }
                let mut button = ButtonProps::default()
                    .variant(ButtonVariant::Bare)
                    .extra_classes(vec![c::SWATCH_BUTTON.to_string()])
                    .disabled(!can_edit);
                if can_edit {
                    button = button.on_click(move || draft.update(|d| d.brand = index));
                }
                let _ = (name, &fonts);
                grid = grid.child(button.build().child(swatch));
            }
            FormItemProps::new(requiem_i18n::t!("guild-settings-banner"), grid).build()
        })
    };

    let traits = {
        let mut grid = StackProps::horizontal()
            .wrap(true)
            .full_width(false)
            .build();
        for index in 0..gs::TRAIT_SLOTS {
            let draft = draft.clone();
            let emoji = div().class(c::TRAIT_EMOJI).child(
                Node::new(
                    "span",
                    EmojiButtonIconWidget::new(),
                )
                .class(c::TRAIT_EMOJI_ICON),
            );
            let input = input_props(d.traits.get(index).cloned().unwrap_or_default())
                .max_length(24_usize)
                .leading(emoji)
                .on_change(move |text: &str| draft.update(|d| d.traits[index] = text.to_string()))
                .build();
            grid = grid.child(div().class(c::TRAIT_CELL).child(input));
        }
        FormItemProps::new(requiem_i18n::t!("guild-settings-traits"), grid)
            .description(requiem_i18n::t!("guild-settings-add-up-to-5-traits-to"))
            .build()
    };

    let description_field = {
        let draft = draft.clone();
        let area = text_area(
            input_props(d.description.clone())
                .placeholder(
                    requiem_i18n::t!("guild-settings-tell-the-world-a-bit-about").to_string(),
                )
                .max_length(300_usize)
                .rows(3_u32)
                .on_change(move |text: &str| draft.update(|d| d.description = text.to_string())),
        );
        FormItemProps::new(requiem_i18n::t!("guild-settings-description"), area)
            .description(requiem_i18n::t!(
                "guild-settings-how-did-your-server-get-started"
            ))
            .build()
    };

    let games = form_section(
        FormSectionProps::default()
            .title(requiem_i18n::t!("guild-settings-games").to_string())
            .description(
                requiem_i18n::t!("guild-settings-what-games-does-your-server-play").to_string(),
            )
            .children(vec![input_props(String::new()).disabled(true).build()]),
    );

    let private = {
        let draft = draft.clone();

        let toggle = Rc::new(Cell::new(ToggleState::new(draft.peek().private)));
        Node::reactive(move || {
            let on = draft.get().private;
            let state = toggle.get().set(on);
            toggle.set(state);
            let switch = {
                let draft = draft.clone();
                SwitchProps::new(state, move |on: bool| draft.update(|d| d.private = on))
                    .label(requiem_i18n::t!("guild-settings-private-profile").to_string())
                    .description(
                        requiem_i18n::t!("guild-settings-when-enabled-only-server-members-can")
                            .to_string(),
                    )
                    .disabled(!can_edit)
                    .build()
            };
            let mut unknown = unknown_server_card();
            if !on {
                unknown = unknown.class(c::HALF);
            }
            StackProps::horizontal()
                .gap(32)
                .full_width(false)
                .build()
                .children([
                    div().class(c::HALF_COLUMN).child(switch),
                    div().class(c::HALF_COLUMN).child(unknown),
                ])
        })
    };

    let divider = || div().class(c::PDIVIDER);
    let left = div().class(c::LEFT_COLUMN).children([
        StackProps::vertical().gap(0).build().children([
            HeadingProps::new(
                HeadingLevel::H1,
                requiem_i18n::t!("guild-settings-server-profile"),
            )
            .variant(TextVariant::HEADING_LG_SEMIBOLD)
            .build()
            .class(classes_guild_settings::PAGE_TITLE),
            TextProps::styled(
                requiem_i18n::t!("guild-settings-customize-how-your-server-appears-in"),
                TextVariant::TEXT_SM_MEDIUM,
            )
            .color(TEXT_DEFAULT)
            .build()
            .class(c::PROFILE_SUBTITLE),
        ]),
        name,
        divider(),
        icon_field,
        divider(),
        banner,
        divider(),
        traits,
        divider(),
        description_field,
        divider(),
        games,
        divider(),
        private,
    ]);
    let right = div()
        .class(c::RIGHT_COLUMN)
        .child(profile_preview(env, guild, &profile));
    StackProps::horizontal()
        .gap(40)
        .full_width(false)
        .build()
        .children([left, right])
}

fn unknown_server_card() -> Node {
    let avatar = div().class(c::PREVIEW_AVATAR_WRAP).child(
        div().class(c::PREVIEW_AVATAR_LETTERS).child(
            TextProps::styled("?", TextVariant::TEXT_SM_MEDIUM)
                .line_height(14.0_f32)
                .color(TEXT_DEFAULT)
                .build(),
        ),
    );
    div().class(c::PREVIEW_CARD).children([
        radial(0).class(c::PREVIEW_BANNER),
        div().class(c::PREVIEW_AVATAR_ROW).child(avatar),
        div().class(c::PREVIEW_HEADER).child(
            TextProps::styled(
                requiem_i18n::t!("guild-settings-private-server"),
                TextVariant::TEXT_LG_MEDIUM,
            )
            .color(TEXT_STRONG)
            .build(),
        ),
        div().class(c::PREVIEW_BODY).child(
            TextProps::styled(
                requiem_i18n::t!("guild-settings-the-server-has-limited-who-can"),
                TextVariant::TEXT_SM_NORMAL,
            )
            .color(TEXT_SECONDARY)
            .build(),
        ),
    ])
}

fn profile_preview(
    env: &Env,
    guild: &Guild,
    profile: &guilds::GuildProfile,
) -> Node {
    let (env, guild, draft) = (env.clone(), guild.clone(), gs::state().profile.clone());
    let (online, members) = (profile.online_count, profile.member_count);
    Node::reactive(move || {
        let d = draft.get();
        let icon_image = match d.icon.shown() {
            Some(upload) => Some(upload.preview.clone()),
            None => guild.icon.as_deref().and_then(|hash| {
                let image = env
                    .images
                    .get_or_decode_image(&env.backend.guild_icon_url(guild.id, hash));
                if image.is_none() {
                    let _ = env.tick.get();
                }
                image
            }),
        };
        let inner = match icon_image {
            Some(image) => Node::new(
                "div",
                ImageWidget::new(Some(image)),
            )
            .class(c::PREVIEW_AVATAR_IMAGE),
            None => div().class(c::PREVIEW_AVATAR_LETTERS).child(
                TextProps::styled(
                    guild::acronym(&d.name),
                    TextVariant::TEXT_SM_MEDIUM,
                )
                .line_height(14.0_f32)
                .color(TEXT_DEFAULT)
                .build(),
            ),
        };
        let dot = |color: Color| div().class(c::PREVIEW_DOT).style(solid(color));
        let count = |color: Color, text: String| {
            StackProps::horizontal()
                .align(Align::Center)
                .gap(4)
                .full_width(false)
                .build()
                .children([
                    dot(color),
                    TextProps::styled(&text, TextVariant::TEXT_SM_NORMAL)
                        .color(TEXT_SECONDARY)
                        .build(),
                ])
        };
        div().class(c::PREVIEW_CARD).children([
            radial(d.brand).class(c::PREVIEW_BANNER),
            div()
                .class(c::PREVIEW_AVATAR_ROW)
                .child(div().class(c::PREVIEW_AVATAR_WRAP).child(inner)),
            div().class(c::PREVIEW_HEADER_BLOCK).children([
                TextProps::styled(&d.name, TextVariant::TEXT_MD_SEMIBOLD)
                    .color(TEXT_STRONG)
                    .build(),
                StackProps::horizontal()
                    .full_width(false)
                    .build()
                    .children([
                        count(
                            Color::from_rgb(0x3b, 0xa5, 0x5c),
                            requiem_i18n::t!("guild-settings-preview-online", count = online),
                        ),
                        count(
                            Color::from_rgb(0x7f, 0x80, 0x8a),
                            requiem_i18n::t!("guild-settings-preview-members", count = members),
                        ),
                    ]),
                TextProps::styled(
                    requiem_i18n::t!(
                        "guild-settings-preview-established",
                        date = time::snowflake_month_year(guild.id.get())
                    ),
                    TextVariant::TEXT_SM_NORMAL,
                )
                .color(TEXT_SECONDARY)
                .build(),
            ]),
            div().class(c::PREVIEW_BODY).child(
                TextProps::styled(&d.description, TextVariant::TEXT_SM_NORMAL)
                    .color(TEXT_SECONDARY)
                    .build(),
            ),
        ])
    })
}

fn listed_roles(env: &Env, guild: &Guild) -> Vec<Role> {
    env.state
        .permissions
        .guild_roles(guild.id)
        .into_iter()
        .filter(|r| r.id.get() != guild.id.get())
        .collect()
}

fn roles_page(env: &Env, guild: &Guild) -> Node {
    if !env
        .state
        .permissions
        .guild_permissions(guild)
        .contains(Permissions::MANAGE_ROLES)
    {
        return div().class(c::PAGE).children([
            HeadingProps::new(HeadingLevel::H1, requiem_i18n::t!("guild-settings-roles"))
                .variant(TextVariant::HEADING_LG_SEMIBOLD)
                .build()
                .class(classes_guild_settings::PAGE_TITLE),
            InlineNoticeProps::new(requiem_i18n::t!(
                "guild-settings-you-need-the-manage-roles-permission"
            ))
            .build()
            .class(classes_guild_settings::NOTICE),
        ]);
    }
    let counts = gs::role_counts(&env.queries, &env.http, guild.id);
    if counts.is_none() {
        let _ = env.tick.get();
    }
    let roles = listed_roles(env, guild);
    let guild_id = guild.id;

    let everyone = env
        .state
        .permissions
        .guild_roles(guild.id)
        .into_iter()
        .find(|r| r.id.get() == guild.id.get());
    let default_card = {
        let icon_box = div()
            .class(c::DEFAULT_CARD_ICON)
            .child(icon(GROUP, TEXT_DEFAULT).class(c::MEMBERS_ICON));
        let text = div().class(c::DEFAULT_CARD_TEXT).children([
            TextProps::styled(
                requiem_i18n::t!("guild-settings-default-permissions"),
                TextVariant::TEXT_MD_MEDIUM,
            )
            .color(TEXT_STRONG)
            .build(),
            TextProps::styled(
                requiem_i18n::t!("guild-settings-everyone-applies-to-all-server-members"),
                TextVariant::TEXT_SM_NORMAL,
            )
            .color(TEXT_MUTED)
            .build(),
        ]);
        let chevron = icon(CHEVRON_LARGE_RIGHT, TEXT_SECONDARY).class(c::MEMBERS_ICON);
        let mut card = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![c::DEFAULT_CARD.to_string()]);
        if let Some(everyone) = everyone {
            card = card.on_click(move || gs::edit_role(&everyone, guild_id));
        }
        card.build().children([icon_box, text, chevron])
    };

    let search = {
        let query = ROLE_QUERY.with(Clone::clone);
        div().class(c::SEARCH_BOX).child(search_bar(
            &requiem_i18n::t!("guild-settings-search-roles"),
            move |text| query.set(text.to_string()),
        ))
    };
    let create = {
        let (queries, http, state) = (env.queries.clone(), env.http.clone(), env.state.clone());
        ButtonProps::default()
            .variant(ButtonVariant::Primary)
            .label(requiem_i18n::t!("guild-settings-create-role").to_string())
            .on_click(move || gs::create_role(&queries, &http, &state, guild_id))
            .build()
    };
    let help = TextProps::styled(
        requiem_i18n::t!("guild-settings-members-use-the-color-of-the"),
        TextVariant::TEXT_SM_NORMAL,
    )
    .color(TEXT_MUTED)
    .build()
    .class(c::HELP_TEXT);

    let table = {
        let (env, counts, roles, guild) =
            (env.clone(), counts.clone(), roles.clone(), guild.clone());
        Node::reactive(move || {
            let query = ROLE_QUERY.with(Signal::get).trim().to_lowercase();
            let shown: Vec<&Role> = roles
                .iter()
                .filter(|r| r.name.to_lowercase().contains(&query))
                .collect();
            let head = StackProps::horizontal()
                .align(Align::Center)
                .gap(0)
                .full_width(false)
                .padding(Padding::bottom(8))
                .build()
                .children([
                    div().class(c::COL_NAME).child(
                        TextProps::styled(
                            requiem_i18n::t!("guild-settings-roles-count", count = shown.len()),
                            TextVariant::TEXT_XS_BOLD,
                        )
                        .color(TEXT_SUBTLE)
                        .build(),
                    ),
                    div().class(c::COL_MEMBERS).child(
                        TextProps::styled(
                            requiem_i18n::t!("guild-settings-members-header"),
                            TextVariant::TEXT_XS_BOLD,
                        )
                        .color(TEXT_SUBTLE)
                        .build(),
                    ),
                    div().class(c::COL_ACTIONS),
                ]);
            let mut table = div().class(c::TABLE).child(head);
            for role in shown {
                let count = counts
                    .as_ref()
                    .and_then(|m| m.get(&role.id.to_string()).copied())
                    .unwrap_or(0);
                table = table.child(role_row(&env, &guild, role, count));
            }
            table
        })
    };

    div().class(c::PAGE).children([
        HeadingProps::new(HeadingLevel::H1, requiem_i18n::t!("guild-settings-roles"))
            .variant(TextVariant::HEADING_LG_SEMIBOLD)
            .build()
            .class(classes_guild_settings::PAGE_TITLE),
        TextProps::styled(
            requiem_i18n::t!("guild-settings-use-roles-to-group-your-server"),
            TextVariant::TEXT_SM_NORMAL,
        )
        .color(TEXT_MUTED)
        .build()
        .class(classes_guild_settings::PAGE_DESCRIPTION),
        default_card,
        div().class(c::SEARCH_ROW).children([search, create]),
        help,
        table,
    ])
}

fn role_row(env: &Env, guild: &Guild, role: &Role, members: u64) -> Node {
    let locked = !env.state.permissions.can_manage_role(guild, role);
    let color = role_color(role.color);
    let name = div().class(c::ROLE_NAME_CELL).children([
        icon(SHIELD_USER, color).class(c::ROLE_SHIELD),
        TextProps::styled(&role.name, TextVariant::TEXT_MD_MEDIUM)
            .color(TEXT_STRONG)
            .build()
            .class(line_clamp_class(1)),
    ]);
    let count = div().class(c::MEMBERS_CELL).children([
        TextProps::styled(members.to_string(), TextVariant::TEXT_MD_NORMAL)
            .color(TEXT_DEFAULT)
            .build(),
        icon(USER, TEXT_SECONDARY).class(c::MEMBERS_ICON),
    ]);

    let more = {
        let (role_for_menu, guild_for_menu, env_for_menu) =
            (role.clone(), guild.clone(), env.clone());
        ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![c::ICON_BUTTON.to_string()])
            .on_click_at(move |rect: AbsoluteRect| {
                let (role, guild, env) = (
                    role_for_menu.clone(),
                    guild_for_menu.clone(),
                    env_for_menu.clone(),
                );
                menu::open(rect.x, rect.y + rect.height, move || {
                    role_menu(&env, &guild, &role)
                })
            })
            .build()
            .child(icon(MORE_HORIZONTAL, TEXT_SECONDARY).class(c::ICON_BUTTON_ICON))
    };
    let drag = div()
        .class(c::DRAG_CELL)
        .child(icon(DRAG, TEXT_MUTED).class(c::MEMBERS_ICON));
    let buttons = div().class(c::ROW_BUTTONS).child(more);
    let (role_id, guild_id, role_for_click) = (role.id, guild.id, role.clone());
    let _ = role_id;
    ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![c::ROLE_ROW.to_string()])
        .on_click(move || gs::edit_role(&role_for_click, guild_id))
        .build()
        .children([
            drag,
            name,
            count,
            if locked {
                div().class(c::ROW_BUTTONS)
            } else {
                buttons
            },
        ])
}

fn role_menu(env: &Env, guild: &Guild, role: &Role) -> Vec<ContextItem> {
    let id = role.id.to_string();
    let mut items = vec![ContextItem::action(
        requiem_i18n::t!("guild-settings-copy-role-id"),
        move || menu::copy_with_toast(id.clone(), &requiem_i18n::t!("guild-settings-copied")),
    )];
    if env.state.permissions.can_manage_role(guild, role) && !role.managed {
        items.push(ContextItem::Separator);
        let (role, env, guild_id) = (role.clone(), env.clone(), guild.id);
        items.push(
            ContextItem::action(requiem_i18n::t!("guild-settings-delete"), move || {
                confirm_delete_role(&env, guild_id, &role)
            })
            .icon_position(IconPosition::Trailing)
            .icon(TRASH)
            .danger(),
        );
    }
    items
}

fn role_header_menu(env: &Env, guild: &Guild, role: &Role) -> Vec<ContextItem> {
    let mut items = vec![
        ContextItem::unavailable(requiem_i18n::t!("guild-settings-duplicate-role")),
        ContextItem::unavailable(requiem_i18n::t!("guild-settings-view-server-as-role")),
    ];
    if env.state.permissions.can_manage_role(guild, role) && !role.managed {
        items.push(ContextItem::Separator);
        let (role, env, guild_id) = (role.clone(), env.clone(), guild.id);
        items.push(
            ContextItem::action(requiem_i18n::t!("guild-settings-delete"), move || {
                confirm_delete_role(&env, guild_id, &role)
            })
            .icon_position(IconPosition::Trailing)
            .icon(TRASH)
            .danger(),
        );
    }
    items
}

fn confirm_delete_role(env: &Env, guild_id: Id<GuildMarker>, role: &Role) {
    let (queries, http, role_id, name) = (
        env.queries.clone(),
        env.http.clone(),
        role.id,
        role.name.clone(),
    );
    menu::confirm(ConfirmRequest {
        title: requiem_i18n::t!("guild-settings-delete-role").to_string(),
        body: requiem_i18n::t!("guild-settings-delete-role-body", name = name),
        confirm_label: requiem_i18n::t!("guild-settings-delete").to_string(),
        on_confirm: Rc::new(move || gs::delete_role(&queries, &http, guild_id, role_id)),
        critical: true,
        content: None,
    });
}

fn confirm_delete_server(env: &Env, guild_id: Id<GuildMarker>) {
    let Some(name) = env.state.guilds.find(guild_id).map(|g| g.name) else {
        return;
    };
    let (queries, http) = (env.queries.clone(), env.http.clone());
    menu::confirm(ConfirmRequest {
        title: requiem_i18n::t!("guild-settings-delete-server-title", name = name.as_str()),
        body: requiem_i18n::t!("guild-settings-delete-server-body", name = name),
        confirm_label: requiem_i18n::t!("guild-settings-delete-server").to_string(),
        on_confirm: Rc::new(move || {
            gs::close();
            let http = http.clone();
            gs::run(
                &queries,
                move || async move {
                    guilds::delete_guild(&http, guild_id).await
                },
                |result| {
                    if let Err(message) = result {
                        menu::notify_failure(&message);
                    }
                },
            );
        }),
        critical: true,
        content: None,
    });
}

fn role_editor(env: &Env, guild: &Guild, role_id: Id<RoleMarker>) -> Node {
    let s = gs::state();
    let all = env.state.permissions.guild_roles(guild.id);
    let Some(role) = all.iter().find(|r| r.id == role_id).cloned() else {
        gs::back_to_roles();
        return div();
    };
    if s.draft.peek().as_ref().map(|d| d.role_id) != Some(role_id) {
        s.draft.set(Some(gs::RoleDraft::of(&role)));
    }
    let locked = !env.state.permissions.can_manage_role(guild, &role);
    let everyone = role.id.get() == guild.id.get();

    let mut list = div().class(c::EDIT_SIDEBAR_LIST);
    let mut ordered: Vec<&Role> = all
        .iter()
        .filter(|r| r.id.get() != guild.id.get())
        .collect();
    ordered.extend(all.iter().filter(|r| r.id.get() == guild.id.get()));
    for other in ordered {
        let mut classes = vec![c::EDIT_ROLE_ROW.to_string()];
        if other.id == role_id {
            classes.push(c::EDIT_ROLE_ROW_ACTIVE.to_string());
        }
        let is_everyone = other.id.get() == guild.id.get();
        let dot = if is_everyone {
            div().class(c::EDIT_ROLE_DOT).style(solid(DEFAULT_ROLE))
        } else {
            div()
                .class(c::EDIT_ROLE_DOT)
                .style(solid(role_color(other.color)))
        };
        let (target, guild_id) = (other.clone(), guild.id);
        let name = if is_everyone {
            "@everyone".to_string()
        } else {
            other.name.clone()
        };
        list = list.child(
            ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(classes)
                .on_click(move || gs::edit_role(&target, guild_id))
                .build()
                .children([
                    dot,
                    TextProps::styled(&name, TextVariant::TEXT_MD_MEDIUM)
                        .color(if other.id == role_id {
                            TEXT_STRONG
                        } else {
                            TEXT_SECONDARY
                        })
                        .build()
                        .class(line_clamp_class(1)),
                ]),
        );
    }
    let back = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![c::BACK.to_string()])
        .on_click(gs::back_to_roles)
        .build()
        .children([
            icon(ARROW_LARGE_LEFT, TEXT_SECONDARY).class(c::MEMBERS_ICON),
            TextProps::styled(
                requiem_i18n::t!("guild-settings-back"),
                TextVariant::TEXT_XS_BOLD,
            )
            .color(TEXT_SECONDARY)
            .build(),
        ]);
    let add = {
        let (queries, http, state, guild_id) = (
            env.queries.clone(),
            env.http.clone(),
            env.state.clone(),
            guild.id,
        );
        ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![c::ICON_BUTTON_ICON.to_string()])
            .on_click(move || gs::create_role(&queries, &http, &state, guild_id))
            .build()
            .child(icon(PLUS_LARGE, TEXT_SECONDARY).class(c::ICON_BUTTON_ICON))
    };
    let sidebar = div().class(c::EDIT_SIDEBAR).children([
        StackProps::horizontal()
            .align(Align::Center)
            .justify(Justify::SpaceBetween)
            .gap(0)
            .full_width(false)
            .padding(
                Padding::top(60)
                    .with_right(16)
                    .with_bottom(16)
                    .with_left(24),
            )
            .build()
            .children([back, add]),
        scroll_view().class(c::EDIT_SIDEBAR_LIST).child(list),
    ]);

    let tab = s.tab.get();
    let tab = if everyone { RoleTab::Permissions } else { tab };

    let member_count = gs::role_members(&env.queries, &env.http, guild.id, role_id)
        .map(|m| m.len())
        .or_else(|| {
            gs::role_counts(&env.queries, &env.http, guild.id)
                .and_then(|counts| counts.get(&role_id.to_string()).map(|n| *n as usize))
        });
    let mut items = vec![];
    if !everyone {
        items.push(TabItem::new(
            "display",
            requiem_i18n::t!("guild-settings-display"),
        ));
    }
    items.push(TabItem::new(
        "permissions",
        requiem_i18n::t!("guild-settings-permissions"),
    ));
    if !everyone {
        items.push(TabItem::new(
            "links",
            requiem_i18n::t!("guild-settings-links"),
        ));
        items.push(TabItem::new(
            "members",
            requiem_i18n::t!(
                "guild-settings-manage-members",
                count = member_count.unwrap_or(0)
            ),
        ));
    }
    let selected = match tab {
        RoleTab::Display => "display",
        RoleTab::Permissions => "permissions",
        RoleTab::Links => "links",
        RoleTab::Members => "members",
    };
    let tabs = TabBarProps::default()
        .items(items)
        .selected(selected.to_string())
        .on_select(Rc::new(|id: String| {
            gs::state().tab.set(match id.as_str() {
                "display" => RoleTab::Display,
                "permissions" => RoleTab::Permissions,
                "links" => RoleTab::Links,
                _ => RoleTab::Members,
            })
        }) as Rc<dyn Fn(String)>)
        .roomy(true)
        .build();
    let title_name = if everyone {
        "@everyone".to_string()
    } else {
        role.name.clone()
    };

    let title = TextProps::styled(
        requiem_i18n::t!(
            "guild-settings-edit-role-title",
            name = title_name.to_uppercase()
        ),
        TextVariant::TEXT_MD_SEMIBOLD,
    )
    .line_height(25.0_f32)
    .color(TEXT_STRONG)
    .build()
    .class(line_clamp_class(1));
    let mut title_row = div().class(c::EDIT_TITLE_ROW).child(title);
    if !everyone {
        let (role_for_menu, guild_for_menu, env_for_menu) =
            (role.clone(), guild.clone(), env.clone());
        title_row = title_row.child(
            ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![c::EDIT_MORE.to_string()])
                .on_click_at(move |rect: AbsoluteRect| {
                    let (role, guild, env) = (
                        role_for_menu.clone(),
                        guild_for_menu.clone(),
                        env_for_menu.clone(),
                    );
                    menu::open(rect.x, rect.y + rect.height, move || {
                        role_header_menu(&env, &guild, &role)
                    })
                })
                .build()
                .child(icon(MORE_HORIZONTAL, TEXT_STRONG).class(c::EDIT_MORE_ICON)),
        );
    }
    let mut head = div().class(c::EDIT_HEAD).child(title_row);

    if role.managed {
        head = head.child(
            HelpMessageProps::default()
                .text(requiem_i18n::t!("guild-settings-managed-role-warning"))
                .build(),
        );
    }
    head = head.child(tabs);
    match tab {
        RoleTab::Permissions => {
            head = head.child(
                div()
                    .class(c::EDIT_SEARCH_PERMISSIONS)
                    .child(permissions_search()),
            )
        }
        RoleTab::Members => head = head.child(members_search(guild, &role, locked)),
        _ => {}
    }

    let content = match tab {
        RoleTab::Display => display_tab(env, guild, &role, locked),
        RoleTab::Permissions => permissions_tab(env, guild, locked),
        RoleTab::Links => StackProps::vertical().gap(0).build().child(
            TextProps::styled(
                requiem_i18n::t!("guild-settings-role-connections-aren-t-available-here"),
                TextVariant::TEXT_MD_NORMAL,
            )
            .color(TEXT_SECONDARY)
            .build(),
        ),
        RoleTab::Members => members_tab(env, guild, &role, locked),
    };
    let column = div()
        .class(c::EDIT_COLUMN)
        .children([head, scroll_view().class(c::EDIT_SCROLL).child(content)]);
    let main = div().class(c::EDIT_MAIN).child(column);
    div().class(c::EDIT).children([sidebar, main])
}

thread_local! {

    static COLOR_POPOUT: Signal<bool> = signal(false);
}

pub fn close_color_popout() -> bool {
    COLOR_POPOUT.with(|open| {
        let was_open = open.peek();
        if was_open {
            open.set(false);
        }
        was_open
    })
}

const PRESETS: [[u32; 10]; 2] = [
    [
        0x1abc9c, 0x2ecc71, 0x3498db, 0x9b59b6, 0xe91e63, 0xf1c40f, 0xe67e22, 0xe74c3c, 0x95a5a6,
        0x607d8b,
    ],
    [
        0x11806a, 0x1f8b4c, 0x206694, 0x71368a, 0xad1457, 0xc27c0e, 0xa84300, 0x992d22, 0x979c9f,
        0x546e7a,
    ],
];

fn role_color_control(locked: bool) -> Node {
    let draft = gs::state().draft.clone();
    Node::reactive(move || {
        let current = draft.get().map_or(0, |d| d.current.color);
        let set = {
            let draft = draft.clone();
            move |rgb: i64| {
                draft.update(|d| {
                    if let Some(d) = d {
                        d.current.color = rgb;
                    }
                })
            }
        };
        let is_preset = PRESETS.iter().flatten().any(|p| *p as i64 == current);
        let custom_selected = current != 0 && !is_preset;
        let swatch = |classes: Vec<String>,
                      fill: Option<Color>,
                      selected: bool,
                      on_click: Option<Box<dyn FnMut()>>|
         -> Node {
            let mut props = ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(classes);
            if let Some(on_click) = on_click.filter(|_| !locked) {
                props = props.on_click(on_click);
            }
            let mut node = props.build();
            if let Some(fill) = fill {
                node = node.style(solid(fill));
            }
            if locked {
                node = node.class(c::CP_DISABLED);
            }
            if selected {
                node = node.child(icon(CHECKMARK_LARGE, Color::WHITE).class(c::CP_CHECK));
            }
            node
        };

        let default = swatch(
            vec![c::CP_BIG.to_string()],
            Some(DEFAULT_ROLE),
            current == 0,
            Some({
                let set = set.clone();
                Box::new(move || set(0))
            }),
        );

        let toggle = Box::new(|| COLOR_POPOUT.with(|open| open.update(|o| *o = !*o)));
        let mut custom = swatch(
            vec![c::CP_BIG.to_string(), c::CP_CUSTOM.to_string()],
            custom_selected.then(|| role_color(current)),
            custom_selected,
            Some(toggle),
        );
        custom = custom.child(icon(EYE_DROPPER, Color::WHITE).class(c::CP_EYEDROPPER));
        let popout = {
            let (draft, set) = (draft.clone(), set.clone());
            Node::reactive(move || {
                if !COLOR_POPOUT.with(Signal::get) {
                    return div();
                }
                let shown = draft.peek().map_or(0, |d| d.current.color);

                let start = if shown == 0 { 0x000000 } else { shown as u32 };
                let set = set.clone();
                let picker = ColorPickerProps::default()
                    .rgb(start)
                    .plain(true)
                    .autofocus(true)
                    .on_change(Rc::new(move |rgb: u32| set(rgb as i64)) as Rc<dyn Fn(u32)>)
                    .build();
                Node::new(
                    "div",
                    PopoverPanelWidget::new(),
                )
                .dom_id("role-color-popout")
                .child(picker)
            })
            .class(c::CP_POPOUT_SLOT)
        };
        let custom_anchor = div().class(c::CP_ANCHOR).children([custom, popout]);

        let mut rows = StackProps::vertical().gap(10).full_width(false).build();
        for row in PRESETS {
            let mut line = StackProps::horizontal().gap(10).full_width(false).build();
            for preset in row {
                let rgb = preset as i64;
                let set = set.clone();
                line = line.child(swatch(
                    vec![c::CP_SMALL.to_string()],
                    Some(rgb_color(preset)),
                    current == rgb,
                    Some(Box::new(move || set(rgb))),
                ));
            }
            rows = rows.child(line);
        }
        StackProps::horizontal()
            .align(Align::Start)
            .gap(10)
            .full_width(false)
            .build()
            .children([default, custom_anchor, rows])
    })
}

#[derive(Clone)]
enum NameFill {
    Theme,
    Solid(Color),

    Gradient(Vec<Color>),
}

impl NameFill {
    fn of(edit: &RoleEdit) -> Self {
        let hex = |rgb: i64| rgb_color(rgb as u32);
        match (edit.color, edit.secondary, edit.tertiary) {
            (0, None, None) => NameFill::Theme,
            (primary, None, None) => NameFill::Solid(hex(primary)),
            (primary, secondary, tertiary) => {
                let mut colors = vec![hex(primary)];
                colors.extend(secondary.map(hex));
                colors.extend(tertiary.map(hex));
                colors.push(hex(primary));
                NameFill::Gradient(colors)
            }
        }
    }

    fn of_style(style: RoleStyle, edit: &RoleEdit) -> Self {
        match style {
            RoleStyle::Solid => NameFill::of(&RoleEdit {
                secondary: None,
                tertiary: None,
                ..edit.clone()
            }),
            other => {
                let (primary, secondary, tertiary) = other.defaults(0);
                NameFill::of(&RoleEdit {
                    color: primary,
                    secondary,
                    tertiary,
                    ..edit.clone()
                })
            }
        }
    }
}

fn preview_message(
    env: &Env,
    fill: &NameFill,
    username: Color,
    timestamp: Color,
    text: Color,
    light: bool,
) -> Node {
    let f = &env.fonts;
    let name = match fill {
        NameFill::Theme => TextProps::styled(
            requiem_i18n::t!("guild-settings-wumpus"),
            TextVariant::TEXT_MD_MEDIUM,
        )
        .line_height(22.0_f32)
        .color(username)
        .build(),
        NameFill::Solid(color) => TextProps::styled(
            requiem_i18n::t!("guild-settings-wumpus"),
            TextVariant::TEXT_MD_MEDIUM,
        )
        .line_height(22.0_f32)
        .color(*color)
        .build(),
        NameFill::Gradient(colors) => gradient_label(
            &requiem_i18n::t!("guild-settings-wumpus"),
            16.0,
            22.0,
            Weight::MEDIUM,
            colors.clone(),
        ),
    };

    let role_icon = ui::asset_svg(
        if light {
            "image/role_icon_preview_light"
        } else {
            "image/role_icon_preview_dark"
        },
        40,
        40,
    );
    message_list::preview_message(
        f,
        PreviewMessage {
            avatar: ui::asset_image("image/wumpus_avatar"),
            name,
            role_icon,
            time: time::clock_label(),
            timestamp_color: timestamp,
            text: &requiem_i18n::t!("guild-settings-rocks-are-really-old"),
            text_color: text,
        },
    )
}

fn gradient_label(
    text: &str,
    size: f32,
    line_height: f32,
    weight: Weight,
    colors: Vec<Color>,
) -> Node {
    TextProps::new(text.to_string(), size)
        .color(Color::WHITE)
        .weight(weight)
        .line_height(line_height)
        .gradient(colors)
        .gradient_tile(100.0_f32)
        .build()
}

fn role_style(env: &Env, guild: &Guild, locked: bool) -> Node {
    let enhanced = guild.has_feature("ENHANCED_ROLE_COLORS")
        || env::var_os("REQUIEM_SPOOF_BOOST").is_some();
    let draft = gs::state().draft.clone();
    let Some(current) = draft.get().map(|d| d.current.clone()) else {
        return div();
    };
    let selected_style = RoleStyle::of(current.secondary, current.tertiary);

    let card = |style: RoleStyle, name: &str| -> Node {
        let selected = style == selected_style;
        let disabled = locked || (!enhanced && style != RoleStyle::Solid);
        let mut classes = vec![c::RS_CARD.to_string()];
        if selected {
            classes.push(c::RS_CARD_SELECTED.to_string());
        }
        let mut props = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(classes);
        if !disabled {
            let draft = draft.clone();
            props = props.on_click(move || {
                draft.update(|d| {
                    if let Some(d) = d {
                        let (primary, secondary, tertiary) = style.defaults(d.current.color);

                        d.current.color = primary;
                        d.current.secondary = secondary;
                        d.current.tertiary = tertiary;
                    }
                })
            });
        }
        let mut button = props.build();
        if disabled {
            button = button.class(c::CP_DISABLED);
        }
        let fill = NameFill::of_style(style, &current);
        let crop = div().class(c::RS_PREVIEW).child(preview_message(
            env,
            &fill,
            Color::WHITE,
            Color::from_rgb(0x9d, 0x9e, 0xa5),
            Color::from_rgb(0xf3, 0xf3, 0xf4),
            false,
        ));
        let mut label_bar = div().class(c::RS_LABEL);
        if selected {
            label_bar = label_bar.class(c::RS_LABEL_SELECTED);
        }

        let label_bar = label_bar.child(
            TextProps::new(name, 13.3333_f32)
                .line_height(16.0_f32)
                .weight(Weight::MEDIUM)
                .color(if selected {
                    TEXT_STRONG
                } else {
                    TEXT_SECONDARY
                })
                .build(),
        );
        button.children([crop, label_bar])
    };
    let cards = StackProps::horizontal()
        .full_width(false)
        .padding(Padding::bottom(8))
        .build()
        .children([
            card(RoleStyle::Solid, &requiem_i18n::t!("guild-settings-solid")),
            card(
                RoleStyle::Gradient,
                &requiem_i18n::t!("guild-settings-gradient"),
            ),
            card(
                RoleStyle::Holographic,
                &requiem_i18n::t!("guild-settings-holographic"),
            ),
        ]);

    let mut parts = vec![cards];
    if !enhanced {
        let magical = vec![
            Color::from_rgb(0xa9, 0xc9, 0xff),
            Color::from_rgb(0xff, 0xbb, 0xec),
            Color::from_rgb(0xff, 0xc3, 0xa0),
            Color::from_rgb(0xa9, 0xc9, 0xff),
        ];
        let text = div().class(c::RS_UPSELL_TEXT).children([
            StackProps::horizontal()
                .gap(0)
                .full_width(false)
                .build()
                .children([
                    TextProps::styled(
                        requiem_i18n::t!("guild-settings-make-certain-roles"),
                        TextVariant::TEXT_SM_SEMIBOLD,
                    )
                    .color(TEXT_STRONG)
                    .build(),
                    gradient_label(
                        &requiem_i18n::t!("guild-settings-magical"),
                        14.0,
                        18.0,
                        Weight::SEMI_BOLD,
                        magical,
                    ),
                ]),
            TextProps::styled(
                requiem_i18n::t!("guild-settings-unlock-new-role-styles-with-boosting"),
                TextVariant::TEXT_XS_MEDIUM,
            )
            .color(TEXT_SECONDARY)
            .build(),
        ]);
        let button = ButtonProps::default()
            .variant(ButtonVariant::Expressive)
            .icon(BOOST_GEM)
            .label(requiem_i18n::t!("guild-settings-unlock-with-boosting").to_string())
            .build();
        parts.push(div().class(c::RS_UPSELL).children([text, button]));
    }
    FormItemProps::new(
        requiem_i18n::t!("guild-settings-role-style"),
        StackProps::vertical().gap(0).build().children(parts),
    )
    .build()
}

fn role_icon_section() -> Node {
    let pill = div().class(c::AVAILABILITY).children([
        icon(BOOST_GEM, Color::from_rgb(0xff, 0x73, 0xfa)).class(c::AVAILABILITY_ICON),
        TextProps::styled(
            requiem_i18n::t!("guild-settings-unlocked"),
            TextVariant::TEXT_XS_BOLD,
        )
        .color(TEXT_STRONG)
        .build(),
    ]);
    let title = div().class(c::ROLE_ICON_TITLE).children([
        TextProps::styled(
            requiem_i18n::t!("guild-settings-role-icon"),
            TextVariant::TEXT_MD_MEDIUM,
        )
        .color(TEXT_STRONG)
        .build(),
        pill,
    ]);
    let preview = div()
        .class(c::ROLE_ICON_PREVIEW)
        .child(icon(IMAGE_PLUS, TEXT_SECONDARY).class(c::ROLE_ICON_GLYPH));
    let choose = ButtonProps::default()
        .variant(ButtonVariant::Primary)
        .label(requiem_i18n::t!("guild-settings-choose-image").to_string())
        .build();
    div().class(c::ROLE_ICON).children([
        title,
        TextProps::styled(
            requiem_i18n::t!("guild-settings-upload-an-image-under-256-kb"),
            TextVariant::TEXT_SM_NORMAL,
        )
        .color(TEXT_DEFAULT)
        .build()
        .class(c::ROLE_ICON_TEXT),
        StackProps::horizontal()
            .align(Align::Center)
            .gap(16)
            .full_width(false)
            .build()
            .children([preview, choose]),
    ])
}

fn theme_previews(env: &Env, fill: &NameFill) -> Node {
    let rgb = |r: u8, g: u8, b: u8| Color::from_rgb(r, g, b);

    let themes = [
        (
            rgb(0x36, 0x37, 0x3e),
            rgb(0xff, 0xff, 0xff),
            rgb(0x9d, 0x9e, 0xa5),
            rgb(0xf3, 0xf3, 0xf4),
        ),
        (
            rgb(0x20, 0x20, 0x24),
            rgb(0xfb, 0xfb, 0xfb),
            rgb(0x81, 0x82, 0x8a),
            rgb(0xef, 0xef, 0xf1),
        ),
        (
            rgb(0x00, 0x00, 0x00),
            rgb(0xdc, 0xdc, 0xdf),
            rgb(0x73, 0x74, 0x7d),
            rgb(0xd4, 0xd5, 0xd8),
        ),
        (
            rgb(0xfb, 0xfb, 0xfb),
            rgb(0x28, 0x28, 0x2d),
            rgb(0x70, 0x71, 0x7a),
            rgb(0x2e, 0x2e, 0x34),
        ),
    ];
    let mut out = div().class(c::PV_CONTAINER);
    for (index, (bg, username, timestamp, text)) in themes.into_iter().enumerate() {
        out = out.child(
            div()
                .class(c::PV_ROW)
                .style(solid(bg))
                .child(preview_message(
                    env,
                    fill,
                    username,
                    timestamp,
                    text,
                    index == 3,
                )),
        );
    }
    out
}

fn display_tab(env: &Env, guild: &Guild, role: &Role, locked: bool) -> Node {
    let draft = gs::state().draft.clone();

    let name = {
        let draft = draft.clone();
        TextInputProps::new()
            .text(role.name.clone())
            .placeholder(requiem_i18n::t!("guild-settings-new-role").to_string())
            .max_length(100_usize)
            .disabled(locked)
            .on_change(move |text: &str| {
                let text = text.to_string();
                draft.update(|d| {
                    if let Some(d) = d {
                        d.current.name = text;
                    }
                });
            })
            .build()
    };

    let previews = {
        let (env, draft) = (env.clone(), draft.clone());
        Node::reactive(move || {
            let fill = draft
                .get()
                .map_or(NameFill::Theme, |d| NameFill::of(&d.current));
            StackProps::vertical()
                .gap(0)
                .build()
                .children([theme_previews(&env, &fill)])
        })
    };
    let style = {
        let (env, guild) = (env.clone(), guild.clone());
        Node::reactive(move || role_style(&env, &guild, locked))
    };

    let switch_field = |title: &str,
                        hint: Option<&str>,
                        get: fn(&gs::RoleDraft) -> bool,
                        set: fn(&mut gs::RoleDraft, bool)|
     -> Node {
        let (draft, title, hint) = (draft.clone(), title.to_string(), hint.map(str::to_string));
        let toggle = Rc::new(Cell::new(ToggleState::new(
            draft.peek().as_ref().is_some_and(get),
        )));
        Node::reactive(move || {
            let on = draft.get().as_ref().is_some_and(get);
            let state = toggle.get().set(on);
            toggle.set(state);
            let draft = draft.clone();
            let mut props = SwitchProps::new(state, move |on: bool| {
                draft.update(|d| {
                    if let Some(d) = d {
                        set(d, on);
                    }
                })
            })
            .label(title.clone())
            .disabled(locked);
            if let Some(hint) = &hint {
                props = props.description(hint.clone());
            }
            props.build()
        })
    };
    let hoist = switch_field(
        &requiem_i18n::t!("guild-settings-display-role-members-separately-from-onl"),
        None,
        |d| d.current.hoist,
        |d, on| d.current.hoist = on,
    );
    let mentionable = switch_field(
        &requiem_i18n::t!("guild-settings-allow-anyone-to-mention-this-role"),
        Some(&requiem_i18n::t!(
            "guild-settings-note-members-with-the-mention-everyone"
        )),
        |d| d.current.mentionable,
        |d, on| d.current.mentionable = on,
    );

    StackProps::vertical().gap(0).build().children([
        FormItemProps::new(requiem_i18n::t!("guild-settings-role-name"), name)
            .required(true)
            .build(),
        div().class(c::DIVIDER),
        style,
        div().class(c::COLOR_SECTION).child(
            FormItemProps::new(
                requiem_i18n::t!("guild-settings-role-color"),
                role_color_control(locked),
            )
            .description(requiem_i18n::t!(
                "guild-settings-members-use-the-color-of-the-2"
            ))
            .required(true)
            .build(),
        ),
        role_icon_section(),
        previews,
        div().class(c::DIVIDER),
        hoist,
        div().class(c::DIVIDER),
        mentionable,
    ])
}

fn permissions_search() -> Node {
    let query = PERMISSION_QUERY.with(Clone::clone);
    search_bar(
        &requiem_i18n::t!("guild-settings-search-permissions"),
        move |text| query.set(text.to_string()),
    )
}

fn permissions_tab(env: &Env, guild: &Guild, locked: bool) -> Node {
    let draft = gs::state().draft.clone();
    let clear = {
        let draft = draft.clone();
        Node::reactive(move || {
            let any = draft
                .get()
                .as_ref()
                .is_some_and(|d| d.current.permissions != Permissions::NONE);
            let draft = draft.clone();
            TextLinkProps::default()
                .label(requiem_i18n::t!("guild-settings-clear-permissions").to_string())
                .disabled(locked || !any)
                .on_click(move || {
                    draft.update(|d| {
                        if let Some(d) = d {
                            d.current.permissions = Permissions::NONE;
                        }
                    })
                })
                .build()
        })
    };

    let sections = {
        let (env, guild, draft) = (env.clone(), guild.clone(), draft.clone());
        Node::reactive(move || {
            let shown = permission_spec::filtered(&PERMISSION_QUERY.with(Signal::get));
            if shown.is_empty() {
                return StackProps::vertical()
                    .align(Align::Center)
                    .full_width(false)
                    .padding(Padding::top(48))
                    .build()
                    .child(
                        TextProps::styled(
                            requiem_i18n::t!("guild-settings-no-permissions-match-your-search"),
                            TextVariant::TEXT_SM_NORMAL,
                        )
                        .color(TEXT_MUTED)
                        .build(),
                    );
            }
            let mut out = StackProps::vertical().gap(0).full_width(false).build();
            for (section_index, (title, rows)) in shown.into_iter().enumerate() {
                let mut stack = StackProps::vertical().gap(24).full_width(false).build();
                for (index, row) in rows.iter().enumerate() {
                    if index > 0 {
                        stack = stack.child(div().class(c::PERM_DIVIDER));
                    }
                    let flag = row.flag;
                    let can_toggle = !locked && gs::can_grant(&env.state, &guild, flag);
                    let (draft, title, desc) = (draft.clone(), row.title, row.description);

                    let toggle = Rc::new(Cell::new(ToggleState::new(
                        draft
                            .peek()
                            .as_ref()
                            .is_some_and(|d| d.current.permissions.contains(flag)),
                    )));
                    stack = stack.child(Node::reactive(move || {
                        let on = draft
                            .get()
                            .as_ref()
                            .is_some_and(|d| d.current.permissions.contains(flag));
                        let state = toggle.get().set(on);
                        toggle.set(state);
                        let draft = draft.clone();
                        SwitchProps::new(state, move |on: bool| {
                            draft.update(|d| {
                                if let Some(d) = d {
                                    d.current.permissions = if on {
                                        d.current.permissions | flag
                                    } else {
                                        d.current.permissions.without(flag)
                                    };
                                }
                            })
                        })
                        .label(requiem_i18n::text(title))
                        .description(requiem_i18n::text(desc))
                        .disabled(!can_toggle)
                        .build()
                    }));
                }
                let legend =
                    TextProps::styled(requiem_i18n::text(title), TextVariant::TEXT_LG_MEDIUM)
                        .color(TEXT_STRONG)
                        .build()
                        .class(c::PERM_LEGEND);
                let mut section = StackProps::vertical()
                    .gap(0)
                    .full_width(false)
                    .build()
                    .children([legend, stack]);
                if section_index > 0 {
                    section = section.class(c::PERM_SECTION_NEXT);
                }
                out = out.child(section);
            }
            out
        })
    };
    div()
        .class(c::PERM_WRAPPER)
        .children([div().class(c::PERM_CLEAR).child(clear), sections])
}

fn members_search(_guild: &Guild, role: &Role, locked: bool) -> Node {
    let query = MEMBER_QUERY.with(Clone::clone);
    let add = {
        let role = role.clone();
        ButtonProps::default()
            .variant(ButtonVariant::Primary)
            .size(ButtonSize::Small)
            .label(requiem_i18n::t!("guild-settings-add-members").to_string())
            .disabled(locked)
            .on_click(move || {
                ADD_QUERY.with(|q| q.set(String::new()));
                gs::state().picked.set(Vec::new());
                gs::state()
                    .dialog
                    .set(Some(Dialog::AddMembers(role.clone())));
            })
            .build()
    };
    div().class(c::EDIT_SEARCH_MEMBERS).children([
        div().class(c::SEARCH_BOX).child(search_bar(
            &requiem_i18n::t!("guild-settings-search-members"),
            move |text| query.set(text.to_string()),
        )),
        add,
    ])
}

fn members_tab(env: &Env, guild: &Guild, role: &Role, locked: bool) -> Node {
    let ids = gs::role_members(&env.queries, &env.http, guild.id, role.id);
    if ids.is_none() {
        let _ = env.tick.get();
        return TextProps::styled(
            requiem_i18n::t!("guild-settings-loading-members"),
            TextVariant::TEXT_SM_SEMIBOLD,
        )
        .color(TEXT_MUTED)
        .build();
    }
    let ids = ids.map(|a| (*a).clone()).unwrap_or_default();
    let (env, guild_id, role_for_dialog, role_id) = (env.clone(), guild.id, role.clone(), role.id);
    Node::reactive(move || {
        let query = MEMBER_QUERY.with(Signal::get).trim().to_lowercase();
        let mut rows: Vec<(String, String, Id<UserMarker>, Option<String>, String)> = ids
            .iter()
            .filter_map(|id| member_identity(&env, guild_id, *id))
            .filter(|(name, username, ..)| {
                query.is_empty()
                    || name.to_lowercase().contains(&query)
                    || username.to_lowercase().contains(&query)
            })
            .collect();
        rows.sort_by_key(|r| r.0.to_lowercase());
        if rows.is_empty() {
            let mut text = div().class(c::MEMBER_EMPTY_TEXT).child(
                TextProps::styled(
                    requiem_i18n::t!("guild-settings-no-members-were-found"),
                    TextVariant::TEXT_SM_SEMIBOLD,
                )
                .color(TEXT_MUTED)
                .build(),
            );
            if !locked {
                let role = role_for_dialog.clone();
                text = text.child(
                    TextLinkProps::default()
                        .label(
                            requiem_i18n::t!("guild-settings-add-members-to-this-role").to_string(),
                        )
                        .on_click(move || {
                            ADD_QUERY.with(|q| q.set(String::new()));
                            gs::state().picked.set(Vec::new());
                            gs::state()
                                .dialog
                                .set(Some(Dialog::AddMembers(role.clone())));
                        })
                        .build(),
                );
            }
            return div()
                .class(c::MEMBER_ROW)
                .class(c::MEMBER_ROW_EMPTY)
                .children([icon(GROUP, TEXT_MUTED).class(c::MEMBER_EMPTY_ICON), text]);
        }
        let mut out = StackProps::vertical().gap(0).build();
        for (name, username, user_id, avatar, discriminator) in rows {
            out = out.child(member_row(
                &env,
                guild_id,
                role_id,
                user_id,
                &name,
                &username,
                avatar.as_deref(),
                &discriminator,
                locked,
            ));
        }
        out
    })
}

fn member_identity(
    env: &Env,
    guild_id: Id<GuildMarker>,
    id: Id<UserMarker>,
) -> Option<(String, String, Id<UserMarker>, Option<String>, String)> {
    let known = env
        .state
        .members
        .by_guild
        .with_untracked(|all| all.get(&guild_id).and_then(|m| m.get(&id)).cloned());
    let user = known
        .as_ref()
        .map(|m| m.user.clone())
        .or_else(|| env.state.users.get(id))?;
    let nick = known.and_then(|m| m.nick);
    let display = nick
        .or_else(|| user.global_name.clone())
        .unwrap_or_else(|| user.username.clone());
    Some((
        display,
        user.username.clone(),
        id,
        user.avatar.clone(),
        user.discriminator.clone(),
    ))
}

#[allow(clippy::too_many_arguments)]
fn member_row(
    env: &Env,
    guild_id: Id<GuildMarker>,
    role_id: Id<RoleMarker>,
    user_id: Id<UserMarker>,
    name: &str,
    username: &str,
    avatar: Option<&str>,
    discriminator: &str,
    locked: bool,
) -> Node {
    let f = &env.fonts;
    let mut avatar_props = ui::user_avatar_props(user_id, discriminator, f);
    if let Some(image) = avatar.and_then(|hash| {
        let image = env
            .images
            .get_or_decode_image(&env.backend.avatar_url(user_id, hash));
        if image.is_none() {
            let _ = env.tick.get();
        }
        image
    }) {
        avatar_props = avatar_props.image(image);
    }
    let name = TextProps::styled(name, TextVariant::TEXT_SM_SEMIBOLD)
        .color(TEXT_STRONG)
        .build()
        .class(c::MEMBER_NAME);
    let tag = TextProps::styled(username, TextVariant::TEXT_SM_NORMAL)
        .color(TEXT_SECONDARY)
        .build()
        .class(c::MEMBER_TAG);

    let remove = {
        let (queries, http) = (env.queries.clone(), env.http.clone());
        let mut button = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![c::MEMBER_REMOVE.to_string()]);
        if !locked {
            button = button
                .on_click(move || gs::remove_member(&queries, &http, guild_id, role_id, user_id));
        }
        button.build().child(
            icon(CIRCLE_X, if locked { TEXT_MUTED } else { TEXT_SECONDARY })
                .class(c::MEMBER_REMOVE_ICON),
        )
    };
    div().class(c::MEMBER_ROW).children([
        avatar_props.build().class(c::MEMBER_AVATAR),
        name,
        tag,
        div().class(c::MEMBER_REMOVE_WRAP).child(remove),
    ])
}

fn save_bar(env: &Env, guild: &Guild, page: &str, editing_role: bool) -> Option<Node> {
    let s = gs::state();
    let on_roles = page == gs::ROLES_PAGE && editing_role;
    let on_profile = page == gs::SERVER_PROFILE_PAGE;
    if !on_roles && !on_profile {
        return None;
    }
    let (env, guild_id, s) = (env.clone(), guild.id, s);
    let inner = Node::reactive(move || {
        let dirty = if on_roles {
            s.draft.get().as_ref().is_some_and(|d| d.dirty())
        } else {
            s.profile.get().dirty()
        };
        let error = s.error.get();
        if !dirty && error.is_none() {
            return div();
        }
        let mut out = StackProps::vertical().gap(0).build();
        if let Some(error) = error {
            out = out.child(
                TextProps::styled(&error, TextVariant::TEXT_SM_MEDIUM)
                    .color(TEXT_CRITICAL)
                    .build()
                    .class(c::BAR_ERROR),
            );
        }
        if !dirty {
            return out;
        }
        let message = TextProps::styled(
            requiem_i18n::t!("guild-settings-careful-you-have-unsaved-changes"),
            TextVariant::TEXT_MD_MEDIUM,
        )
        .color(Color::WHITE)
        .build();
        let reset = {
            let s = s.clone();
            TextLinkProps::default()
                .label(requiem_i18n::t!("guild-settings-reset").to_string())
                .on_click(move || {
                    if on_roles {
                        s.draft.update(|d| {
                            if let Some(d) = d {
                                d.reset();
                            }
                        });
                    } else {
                        s.profile.update(|d| d.reset());
                    }
                    s.error.set(None);

                    s.epoch.update(|n| *n += 1);
                })
                .build()
        };
        let save = {
            let (queries, http) = (env.queries.clone(), env.http.clone());
            ButtonProps::default()
                .variant(ButtonVariant::Active)
                .size(ButtonSize::Small)
                .label(requiem_i18n::t!("guild-settings-save-changes").to_string())
                .on_click(move || {
                    if on_roles {
                        gs::save_role(&queries, &http, guild_id)
                    } else {
                        gs::save_profile(&queries, &http, guild_id)
                    }
                })
                .build()
        };
        out.child(
            div().class(bar::BAR).children([
                message,
                StackProps::horizontal()
                    .align(Align::Center)
                    .full_width(false)
                    .build()
                    .children([reset, save]),
            ]),
        )
    });
    Some(div().class(c::BAR_HOST).child(inner.class(if on_roles {
        c::BAR_INNER
    } else {
        c::BAR_INNER_WIDE
    })))
}

pub fn open_dialogs(env: &Env) -> Option<Node> {
    let s = gs::state();
    let dialog = s.dialog.get()?;
    match dialog {
        Dialog::AddMembers(role) => {
            let guild_id = s.guild.peek()?;
            let held = gs::role_members(&env.queries, &env.http, guild_id, role.id)
                .map(|a| (*a).clone())
                .unwrap_or_default();
            let roster: Vec<(String, Id<UserMarker>)> =
                env.state.members.by_guild.with_untracked(|all| {
                    all.get(&guild_id)
                        .map(|m| {
                            m.values()
                                .filter(|m| !held.contains(&m.user.id))
                                .map(|m| {
                                    (
                                        m.nick
                                            .clone()
                                            .or_else(|| m.user.global_name.clone())
                                            .unwrap_or_else(|| m.user.username.clone()),
                                        m.user.id,
                                    )
                                })
                                .collect()
                        })
                        .unwrap_or_default()
                });
            let close = || {
                let s = gs::state();
                s.dialog.set(None);
                s.picked.set(Vec::new());
            };
            let query = ADD_QUERY.with(Clone::clone);
            let search = search_bar(
                &requiem_i18n::t!("guild-settings-search-members"),
                move |text| query.set(text.to_string()),
            );
            let picked = s.picked.clone();
            let list = {
                let picked = picked.clone();
                Node::reactive(move || {
                    let query = ADD_QUERY.with(Signal::get).trim().to_lowercase();
                    let chosen = picked.get();
                    let mut rows: Vec<&(String, Id<UserMarker>)> = roster
                        .iter()
                        .filter(|(name, _)| {
                            query.is_empty() || name.to_lowercase().contains(&query)
                        })
                        .collect();
                    rows.sort_by_key(|(name, _)| name.to_lowercase());
                    let mut out = div().class(c::ADD_LIST);
                    for (name, id) in rows {
                        let on = chosen.contains(id);
                        let (picked, id) = (picked.clone(), *id);
                        out = out.child(
                            ButtonProps::default()
                                .variant(ButtonVariant::Bare)
                                .extra_classes(vec![c::PICK_ROW.to_string()])
                                .on_click(move || {
                                    picked.update(|p| {
                                        if let Some(at) = p.iter().position(|x| *x == id) {
                                            p.remove(at);
                                        } else {
                                            p.push(id);
                                        }
                                    })
                                })
                                .build()
                                .children([
                                    div().class(c::PICK_NAMES).child(
                                        TextProps::styled(name, TextVariant::TEXT_MD_MEDIUM)
                                            .color(TEXT_STRONG)
                                            .build(),
                                    ),
                                    CheckboxProps::new(ToggleState::new(on), |_: bool| {}).build(),
                                ]),
                        );
                    }
                    out
                })
            };
            let body = div().class(c::FIELD).children([search, list]);
            let cancel = ButtonProps::default()
                .variant(ButtonVariant::Secondary)
                .label(requiem_i18n::t!("guild-settings-cancel").to_string())
                .on_click(close)
                .build();
            let add = {
                let (queries, http, picked, role_id) = (
                    env.queries.clone(),
                    env.http.clone(),
                    picked.clone(),
                    role.id,
                );
                ButtonProps::default()
                    .variant(ButtonVariant::Primary)
                    .label(requiem_i18n::t!("guild-settings-add").to_string())
                    .on_click(move || {
                        let ids = picked.peek();
                        if !ids.is_empty() {
                            gs::add_members(&queries, &http, guild_id, role_id, ids);
                        }
                    })
                    .build()
            };
            Some(
                ModalProps::default()
                    .size(ModalSize::Small)
                    .title(
                        TextProps::styled(
                            requiem_i18n::t!("guild-settings-add-members"),
                            TextVariant::TEXT_LG_SEMIBOLD,
                        )
                        .color(TEXT_STRONG)
                        .build(),
                    )
                    .subtitle(role.name.clone())
                    .dismissable(true)
                    .on_dismiss(move || {
                        let s = gs::state();
                        s.dialog.set(None);
                        s.picked.set(Vec::new());
                    })
                    .open(ToggleState::new(true))
                    .body(body)
                    .footer(vec![cancel, add])
                    .build()
                    .focus_scope(),
            )
        }
    }
}
