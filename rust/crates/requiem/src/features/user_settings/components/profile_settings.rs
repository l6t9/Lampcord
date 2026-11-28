use crate::shell::classes::profile_settings as c;
use requiem_core::rest::endpoints::users;
use crate::shell;
use requiem_core::rest::endpoints::users::ProfileView;
use requiem_core::ui::devtools;
use requiem_core::ui::emoji_button_icon::EmojiButtonIconWidget;
use requiem_core::ui::message_list;
use requiem_core::ui;
use requiem_ui::components::TextVariant;
use requiem_ui::components::{Align, StackProps};
use requiem_ui::components::{AvatarStatusSize, TextLinkProps, avatar_with_status};
use requiem_ui::components::{ButtonSize, ButtonVariant};
use requiem_ui::components;
use requiem_ui::signal;
use requiem_ui::widget::primitives::PopoverPanelWidget;
use std::env;
use std::mem;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync;

use requiem_ui::colors::INPUT_PLACEHOLDER;
use requiem_ui::components::{
    ButtonProps, ColorPickerProps, FeatureTag, FormSectionProps, SelectOption, SelectProps,
    TabBarProps, TabItem, TextInputProps, TextProps, div, form_section, icon, premium_section_tags,
    rgb_color, tab_bar,
};
use requiem_ui::node::Node;
use requiem_ui::signal::Signal;
use requiem_ui::widget::primitives::PresenceStatus;
use skia_safe::Color;
use skia_safe::font_style::Weight;

use requiem_core::config::BackendConfig;
use requiem_core::images::ImageCache;
use requiem_core::models::id::marker::GuildMarker;
use requiem_core::models::{Id, User, UserProfile};
use requiem_core::query::QueryClient;
use requiem_core::rest::endpoints::users::{AccountUpdate, ProfileUpdate};
use requiem_core::rest::{HttpClient, RestError};
use requiem_core::state::AppState;
use requiem_core::ui::account_popout::AccountPopoutState;
use requiem_core::ui::custom_status::BubbleActions;
use requiem_core::ui::emoji_picker::{self, EmojiPickerState};
use requiem_core::ui::profile_card::{
    CardContainer, CardParts, CardStatus, ProfileCardView, footer_slot, profile_card,
};
use requiem_core::ui::profile_image::prepare_upload;
use requiem_core::ui::profile_popout::profile_query_key;
use requiem_core::ui::{
    CatalogKind, ColorTarget, Fonts, ImageTarget, NITRO_PRESETS as PRESETS, PendingImage,
    ProfileDialog, ProfileDraft, ProfileTab, ServerDraft, SettingsUiState, TEXT_DEFAULT,
    TEXT_SECONDARY, TEXT_STRONG,
};
use requiem_ui::icons::{HEART_OUTLINE, NITRO_WHEEL, PENCIL, REFRESH};

pub const PROFILES_CATEGORY: &str = requiem_i18n::key!("settings-modal-profiles");

const DEFAULT_BANNER_COLOR: i64 = 0x747c8c;

pub fn has_nitro(user: Option<&User>) -> bool {
    if devtools::spoof_nitro() {
        return true;
    }
    user.and_then(|u| u.premium_type).is_some_and(|t| t > 0)
}

fn solid(color: Color) -> String {
    format!(
        "background-color: #{:02x}{:02x}{:02x}",
        color.r(),
        color.g(),
        color.b()
    )
}

fn rgb(color: i64) -> Color {
    rgb_color(color as u32)
}

pub fn open_dialog(settings_ui: &Signal<SettingsUiState>, dialog: ProfileDialog) {
    settings_ui.peek().profile_dialog.update(|d| {
        d.dialog = Some(dialog);
        d.open = d.open.set(true);
    });
}

fn pick_image(settings_ui: &Signal<SettingsUiState>, target: ImageTarget, keep_animation: bool) {
    let title = match target {
        ImageTarget::Avatar => requiem_i18n::t!("profile-settings-choose-an-avatar"),
        ImageTarget::Banner => requiem_i18n::t!("profile-settings-choose-a-banner"),
    };

    let picked = match env::var_os("REQUIEM_PICK_IMAGE") {
        Some(path) => Some(PathBuf::from(path)),
        None => {
            let picked = rfd::FileDialog::new()
                .set_title(title)
                .add_filter(
                    requiem_i18n::t!("dialogs-images"),
                    &["png", "jpg", "jpeg", "gif", "webp"],
                )
                .pick_file();

            shell::note_native_dialog_closed();
            picked
        }
    };
    let Some(path) = picked else {
        return;
    };
    match prepare_upload(&path, target, keep_animation) {
        Ok(upload) => open_dialog(settings_ui, ProfileDialog::Image { target, upload }),
        Err(err) => log::warn!("couldn't use {}: {err}", path.display()),
    }
}

fn catalog_button(settings_ui: &Signal<SettingsUiState>, text: &str, kind: CatalogKind) -> Node {
    let settings_ui = settings_ui.clone();
    ButtonProps::default()
        .variant(ButtonVariant::Primary)
        .size(ButtonSize::Small)
        .label(text.to_string())
        .on_click(move || open_dialog(&settings_ui, ProfileDialog::Catalog(kind)))
        .build()
}

fn section(title: &str, tags: Vec<FeatureTag>, children: Vec<Node>, divider: bool) -> Node {
    form_section(
        FormSectionProps::default()
            .title(title.to_string())
            .tags(tags)
            .children(children)
            .divider(divider),
    )
}

fn target_color(d: &ProfileDraft, target: ColorTarget) -> i64 {
    match target {
        ColorTarget::Banner => d.accent_color.unwrap_or(DEFAULT_BANNER_COLOR),
        ColorTarget::ThemePrimary => d.theme_colors.unwrap_or(ProfileDraft::DEFAULT_THEME).0,
        ColorTarget::ThemeAccent => d.theme_colors.unwrap_or(ProfileDraft::DEFAULT_THEME).1,
        ColorTarget::PreviewPrimary => d.preview_colors().0,
        ColorTarget::PreviewAccent => d.preview_colors().1,
    }
}

fn apply_color(draft: &Signal<ProfileDraft>, target: ColorTarget, rgb: i64) {
    draft.update(|d| match target {
        ColorTarget::Banner => d.accent_color = Some(rgb),
        ColorTarget::ThemePrimary => {
            d.theme_colors = Some((rgb, d.theme_colors.unwrap_or(ProfileDraft::DEFAULT_THEME).1))
        }
        ColorTarget::ThemeAccent => {
            d.theme_colors = Some((d.theme_colors.unwrap_or(ProfileDraft::DEFAULT_THEME).0, rgb))
        }
        ColorTarget::PreviewPrimary => d.preview_theme = Some((rgb, d.preview_colors().1)),
        ColorTarget::PreviewAccent => d.preview_theme = Some((d.preview_colors().0, rgb)),
    });
}

const COLOR_SUGGESTIONS: [u32; 4] = [0xd9d9e0, 0x4eb25f, 0xb2914e, 0xb24ea1];

fn color_field(
    settings_ui: &Signal<SettingsUiState>,
    draft: &Signal<ProfileDraft>,
    target: ColorTarget,
    caption: Option<&str>,
) -> Node {
    let open = settings_ui.peek().color_popout;
    let fill = live(draft, move |d| {
        div()
            .class(c::SWATCH_FILL)
            .style(solid(rgb(target_color(d, target))))
    })
    .class(c::SWATCH_FILL_HOST);
    let pencil = icon(PENCIL, Color::WHITE).class(c::SWATCH_PENCIL);
    let toggle = {
        let open = open.clone();
        move || {
            open.update(|current| {
                *current = if *current == Some(target) {
                    None
                } else {
                    Some(target)
                }
            })
        }
    };
    let swatch = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![c::SWATCH.to_string()])
        .on_click(toggle)
        .build()
        .child(fill)
        .child(pencil);

    let popout = Node::reactive({
        let (open, draft) = (open.clone(), draft.clone());
        move || {
            if open.get() != Some(target) {
                return div();
            }
            let current = target_color(&draft.peek(), target) as u32;
            let apply = {
                let draft = draft.clone();
                Rc::new(move |rgb: u32| apply_color(&draft, target, rgb as i64)) as Rc<dyn Fn(u32)>
            };
            let mut suggested = vec![current];
            suggested.extend(COLOR_SUGGESTIONS);
            let picker = ColorPickerProps::default()
                .rgb(current)
                .suggested(suggested)
                .on_change(apply)
                .build();
            Node::new(
                "div",
                PopoverPanelWidget::new(),
            )
            .dom_id("color-popout")
            .child(picker)
        }
    })
    .class(c::COLOR_POPOUT_SLOT);

    let anchor = div().class(c::SWATCH_ANCHOR).children([swatch, popout]);
    match caption {
        Some(text) => StackProps::vertical()
            .align(Align::Center)
            .gap(4)
            .full_width(false)
            .build()
            .children([
                anchor,
                TextProps::styled(text, TextVariant::TEXT_XS_MEDIUM)
                    .color(TEXT_SECONDARY)
                    .build(),
            ]),
        None => anchor,
    }
}

fn live(draft: &Signal<ProfileDraft>, build: impl Fn(&ProfileDraft) -> Node + 'static) -> Node {
    let draft = draft.clone();
    Node::reactive(move || build(&draft.get()))
}

const BIO_LIMIT: usize = 190;

const BIO_INPUT_DOM_ID: &str = "bio-input";

fn build_bio(
    settings_ui: &Signal<SettingsUiState>,
    draft: &Signal<ProfileDraft>,
    emoji_picker: &EmojiPickerState,
    initial: &str,
) -> Node {
    let remaining = |text: &str| BIO_LIMIT.saturating_sub(text.chars().count()).to_string();
    let count = signal::signal(remaining(initial));
    let (bio_draft, count_for_change) = (draft.clone(), count.clone());
    let input = message_list::chat_input(TextInputProps::default())
        .text(initial.to_string())
        .placeholder(String::new())
        .font_size(16.0_f32)
        .color(TEXT_DEFAULT)
        .placeholder_color(INPUT_PLACEHOLDER)
        .max_length(BIO_LIMIT)
        .line_height(22.0_f32)
        .vertical_padding(16.0_f32)
        .insets((16.0_f32, 45.0_f32))
        .focus_border_color(Color::from_rgb(0x4d, 0x96, 0xee))
        .on_change(move |text: &str| {
            count_for_change.set(remaining(text));
            let text = text.to_string();
            bio_draft.update(|d| d.bio = text);
        })
        .build_area()
        .style("height: 138px")
        .dom_id(BIO_INPUT_DOM_ID);
    let count_node = TextProps::styled(count.peek(), TextVariant::TEXT_SM_SEMIBOLD)
        .color(TEXT_DEFAULT)
        .build()
        .bind_text(count)
        .class(c::BIO_COUNT);

    let open = settings_ui.peek().bio_emoji_open;
    let toggle = {
        let open = open.clone();
        move || open.set(!open.peek())
    };
    let button = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![c::BIO_EMOJI_BTN.to_string()])
        .on_click(toggle)
        .build()
        .child(
            Node::new(
                "span",
                EmojiButtonIconWidget::new(),
            )
            .class(c::BIO_EMOJI_ICON),
        );

    let picker = Node::reactive({
        let (open, emoji_picker) = (open.clone(), emoji_picker.clone());
        move || {
            if !open.get() {
                return div();
            }
            let (on_insert, close) = (emoji_picker.on_insert_at.clone(), open.clone());
            let on_pick = Rc::new(move |glyph: String, keep_open: bool| {
                on_insert(BIO_INPUT_DOM_ID, glyph);
                if !keep_open {
                    close.set(false);
                }
            });
            let panel = emoji_picker::build_emoji_only(&emoji_picker.signals, on_pick, None);
            let dismiss = open.clone();
            div().children([
                components::anchored_popover_backdrop(move || dismiss.set(false)),
                Node::new(
                    "div",
                    PopoverPanelWidget::new(),
                )
                .child(panel),
            ])
        }
    })
    .class(c::BIO_PICKER_SLOT);

    div()
        .class(c::BIO_WRAP)
        .children([input, count_node, button, picker])
}

#[derive(Clone)]
struct PreviewData {
    user: Option<User>,
    avatar: Option<skia_safe::Image>,
    banner: Option<skia_safe::Image>,
    fonts: Fonts,
    images: ImageCache,
    backend: BackendConfig,

    profile: Option<sync::Arc<UserProfile>>,

    state: AppState,

    account_popout: AccountPopoutState,
}

#[allow(clippy::too_many_arguments)]
pub fn build_page(
    state: &AppState,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    settings_ui: &Signal<SettingsUiState>,
    queries: &QueryClient,
    http: Option<&HttpClient>,
    account_popout: &AccountPopoutState,
    emoji_picker: &EmojiPickerState,
) -> Node {
    let ui = settings_ui.get();
    let user = state.users.current.get();
    let nitro = has_nitro(user.as_ref());
    let draft = ui.profile_draft.clone();

    let mut banner_hash = None;
    let mut fetched_profile = None;
    if let Some(user) = &user {
        let profile = match http {
            Some(http) => {
                let (http, id) = (http.clone(), user.id);
                queries
                    .query_default::<UserProfile, _, RestError>(
                        &profile_query_key(user.id, None),
                        move || async move {
                            users::get_user_profile(
                                &http,
                                id,
                                None,
                                ProfileView::Plain,
                            )
                            .await
                        },
                    )
                    .data
            }
            None => None,
        };
        banner_hash = profile.as_ref().and_then(|p| p.user.banner.clone());
        fetched_profile = profile.clone();
        if !draft.peek().seeded && (profile.is_some() || http.is_none()) {
            let name = user
                .global_name
                .clone()
                .unwrap_or_else(|| user.username.clone());
            let pronouns = profile
                .as_ref()
                .and_then(|p| p.user_profile.as_ref())
                .map(|m| m.pronouns.clone())
                .unwrap_or_default();
            let bio = profile
                .as_ref()
                .map(|p| p.user.bio.clone())
                .unwrap_or_default();
            let accent = profile.as_ref().and_then(|p| p.user.accent_color);
            let theme = profile
                .as_ref()
                .and_then(|p| p.user_profile.as_ref())
                .and_then(|m| m.theme_colors.as_ref())
                .and_then(|t| (t.len() == 2).then(|| (t[0], t[1])));
            draft.update(|d| d.seed(name, pronouns, bio, accent, theme));
        }
    }

    let data = PreviewData {
        avatar: user.as_ref().and_then(|u| {
            u.avatar
                .as_deref()
                .and_then(|hash| images.get_or_decode_image(&backend.avatar_url(u.id, hash)))
        }),
        banner: user.as_ref().and_then(|u| {
            banner_hash
                .as_deref()
                .and_then(|hash| images.get_or_decode_image(&backend.banner_url(u.id, hash)))
        }),
        user: user.clone(),
        fonts: fonts.clone(),
        images: images.clone(),
        backend: backend.clone(),
        profile: fetched_profile,
        state: state.clone(),
        account_popout: account_popout.clone(),
    };

    let tabs = tab_bar(
        TabBarProps::default()
            .items(vec![
                TabItem::new("main", requiem_i18n::t!("profile-settings-main-profile")),
                TabItem::new(
                    "guild",
                    requiem_i18n::t!("profile-settings-per-server-profiles"),
                ),
            ])
            .selected(
                if ui.profile_tab == ProfileTab::Main {
                    "main"
                } else {
                    "guild"
                }
                .to_string(),
            )
            .on_select({
                let settings_ui = settings_ui.clone();
                Rc::new(move |id: String| {
                    settings_ui.update(|ui| {
                        ui.profile_tab = if id == "main" {
                            ProfileTab::Main
                        } else {
                            ProfileTab::PerServer
                        };
                        ui.profile_server_select_open = false;
                    })
                }) as Rc<dyn Fn(String)>
            }),
    );

    let body = match ui.profile_tab {
        ProfileTab::Main => build_main_tab(settings_ui, &draft, &data, nitro, emoji_picker),
        ProfileTab::PerServer => {
            build_server_tab(state, settings_ui, &draft, &data, queries, http, &ui)
        }
    };
    div().class(c::PAGE).children([tabs, body])
}

fn input(
    draft: &Signal<ProfileDraft>,
    placeholder: &str,
    value: String,
    max: usize,
    on_change: Rc<dyn Fn(&mut ProfileDraft, String)>,
) -> Node {
    let draft = draft.clone();
    TextInputProps::new()
        .text(value)
        .placeholder(placeholder.to_string())
        .max_length(max)
        .on_change(move |text: &str| {
            let text = text.to_string();
            draft.update(|d| on_change(d, text));
        })
        .build()
}

fn build_main_tab(
    settings_ui: &Signal<SettingsUiState>,
    draft: &Signal<ProfileDraft>,
    data: &PreviewData,
    nitro: bool,
    emoji_picker: &EmojiPickerState,
) -> Node {
    let d = draft.peek();
    let promo = build_promo();

    let name_input = input(
        draft,
        "",
        d.display_name.clone(),
        32,
        Rc::new(|d, v| d.display_name = v),
    );
    let mut display_name = section(
        &requiem_i18n::t!("profile-settings-display-name"),
        vec![],
        vec![name_input],
        true,
    );
    if nitro {
        display_name = display_name.child(
            section(
                &requiem_i18n::t!("profile-settings-display-name-styles"),
                premium_section_tags(),
                vec![catalog_button(
                    settings_ui,
                    &requiem_i18n::t!("profile-settings-change-style"),
                    CatalogKind::DisplayNameStyle,
                )],
                false,
            )
            .class(c::SECTION_NESTED),
        );
    }

    let avatar_controls = live(draft, {
        let (settings_ui, draft, user_has_avatar) = (
            settings_ui.clone(),
            draft.clone(),
            data.user.as_ref().is_some_and(|u| u.avatar.is_some()),
        );
        move |d| {
            let mut controls = vec![{
                let settings_ui = settings_ui.clone();
                ButtonProps::new(
                    ButtonVariant::Primary,
                    requiem_i18n::t!("profile-settings-change-avatar"),
                    move || pick_image(&settings_ui, ImageTarget::Avatar, nitro),
                )
                .size(ButtonSize::Small)
                .build()
            }];
            if (user_has_avatar || d.avatar.shown().is_some())
                && !matches!(d.avatar, PendingImage::Removed)
            {
                let draft = draft.clone();
                controls.push(
                    ButtonProps::new(
                        ButtonVariant::Secondary,
                        requiem_i18n::t!("profile-settings-remove-avatar"),
                        move || draft.update(|d| d.avatar = PendingImage::Removed),
                    )
                    .size(ButtonSize::Small)
                    .build(),
                );
            }
            StackProps::horizontal()
                .align(Align::Center)
                .full_width(false)
                .build()
                .children(controls)
        }
    });

    let mut form = div().class(c::FORM).children([
        display_name,
        section(
            &requiem_i18n::t!("profile-settings-pronouns"),
            vec![],
            vec![input(
                draft,
                &requiem_i18n::t!("profile-settings-add-your-pronouns"),
                d.pronouns.clone(),
                40,
                Rc::new(|d, v| d.pronouns = v),
            )],
            true,
        ),
        section(
            &requiem_i18n::t!("profile-settings-profile-widgets"),
            vec![],
            vec![catalog_button(
                settings_ui,
                &requiem_i18n::t!("profile-settings-add-widgets"),
                CatalogKind::Widgets,
            )],
            true,
        ),
        section(
            &requiem_i18n::t!("profile-settings-avatar"),
            vec![],
            vec![avatar_controls],
            true,
        ),
        section(
            &requiem_i18n::t!("profile-settings-avatar-decoration"),
            vec![],
            vec![catalog_button(
                settings_ui,
                &requiem_i18n::t!("profile-settings-change-decoration"),
                CatalogKind::AvatarDecoration,
            )],
            true,
        ),
        section(
            &requiem_i18n::t!("profile-settings-nameplate"),
            vec![],
            vec![catalog_button(
                settings_ui,
                &requiem_i18n::t!("profile-settings-change-nameplate"),
                CatalogKind::Nameplate,
            )],
            true,
        ),
        section(
            &requiem_i18n::t!("profile-settings-profile-effect"),
            vec![],
            vec![catalog_button(
                settings_ui,
                &requiem_i18n::t!("profile-settings-change-effect"),
                CatalogKind::ProfileEffect,
            )],
            true,
        ),
        form_section(
            FormSectionProps::default()
                .title(requiem_i18n::t!("profile-settings-profile-frame").to_string())
                .tags(vec![FeatureTag::New])
                .description(
                    requiem_i18n::t!("profile-settings-complement-your-profile-with-all-new")
                        .to_string(),
                )
                .highlighted(true)
                .children(vec![catalog_button(
                    settings_ui,
                    &requiem_i18n::t!("profile-settings-change-frame"),
                    CatalogKind::ProfileFrame,
                )]),
        ),
    ]);

    if nitro {
        let banner_controls = live(draft, {
            let (settings_ui, draft, server_has_banner) =
                (settings_ui.clone(), draft.clone(), data.banner.is_some());
            move |d| {
                let mut controls = vec![{
                    let settings_ui = settings_ui.clone();
                    ButtonProps::new(
                        ButtonVariant::Primary,
                        requiem_i18n::t!("profile-settings-change-banner"),
                        move || pick_image(&settings_ui, ImageTarget::Banner, true),
                    )
                    .size(ButtonSize::Small)
                    .build()
                }];
                if (server_has_banner || d.banner.shown().is_some())
                    && !matches!(d.banner, PendingImage::Removed)
                {
                    let draft = draft.clone();
                    controls.push(
                        ButtonProps::new(
                            ButtonVariant::Secondary,
                            requiem_i18n::t!("profile-settings-remove-banner"),
                            move || draft.update(|d| d.banner = PendingImage::Removed),
                        )
                        .size(ButtonSize::Small)
                        .build(),
                    );
                }
                StackProps::horizontal()
                    .align(Align::Center)
                    .full_width(false)
                    .build()
                    .children(controls)
            }
        });
        let theme_swatches = StackProps::horizontal()
            .full_width(false)
            .build()
            .children([
                color_field(
                    settings_ui,
                    draft,
                    ColorTarget::ThemePrimary,
                    Some(&requiem_i18n::t!("profile-settings-primary")),
                ),
                color_field(
                    settings_ui,
                    draft,
                    ColorTarget::ThemeAccent,
                    Some(&requiem_i18n::t!("profile-settings-accent")),
                ),
            ]);
        form = form
            .child(section(
                &requiem_i18n::t!("profile-settings-profile-banner"),
                premium_section_tags(),
                vec![banner_controls],
                true,
            ))
            .child(section(
                &requiem_i18n::t!("profile-settings-profile-theme"),
                premium_section_tags(),
                vec![theme_swatches],
                true,
            ));
    } else {
        let banner_color = color_field(settings_ui, draft, ColorTarget::Banner, None);
        form = form.child(section(
            &requiem_i18n::t!("profile-settings-banner-color"),
            vec![],
            vec![banner_color],
            true,
        ));
    }

    form = form.child(form_section(
        FormSectionProps::default()
            .title(requiem_i18n::t!("profile-settings-bio").to_string())
            .description(
                requiem_i18n::t!("profile-settings-you-can-use-markdown-and-links").to_string(),
            )
            .children(vec![build_bio(settings_ui, draft, emoji_picker, &d.bio)]),
    ));

    let columns = div()
        .class(c::COLUMNS)
        .children([form, build_previews(draft, data, false)]);
    let mut page = StackProps::vertical()
        .gap(0)
        .full_width(false)
        .build()
        .children([promo, columns]);
    if !nitro {
        page = page.child(build_upsell(settings_ui, draft, data));
    }
    page
}

fn build_promo() -> Node {
    let copy = div().class(c::PROMO_COPY).children([
        TextProps::new(
            requiem_i18n::t!("profile-settings-create-your-dream-look"),
            20.0_f32,
        )
        .color(Color::WHITE)
        .weight(Weight::EXTRA_BOLD)
        .build(),
        TextProps::styled(
            requiem_i18n::t!("profile-settings-check-out-the-latest-avatar-decorations"),
            TextVariant::TEXT_SM_NORMAL,
        )
        .color(Color::WHITE)
        .build(),
    ]);
    let button = ButtonProps::default()
        .variant(ButtonVariant::OverlayPrimary)
        .label(requiem_i18n::t!("profile-settings-go-to-shop").to_string())
        .on_click(|| {})
        .build();
    div().class(c::PROMO).children([copy, button])
}

fn build_previews(draft: &Signal<ProfileDraft>, data: &PreviewData, per_server: bool) -> Node {
    let preview = Node::reactive({
        let (draft, data) = (draft.clone(), data.clone());
        move || build_preview(&draft.get(), &data, per_server, false)
    });
    let nameplate = Node::reactive({
        let (draft, data) = (draft.clone(), data.clone());
        move || build_nameplate(&draft.get(), &data, per_server)
    });
    div().class(c::PREVIEWS).children([
        StackProps::vertical().full_width(false).build().children([
            TextProps::styled(
                requiem_i18n::t!("profile-settings-preview"),
                TextVariant::TEXT_MD_MEDIUM,
            )
            .color(TEXT_STRONG)
            .build(),
            preview,
        ]),
        StackProps::vertical().full_width(false).build().children([
            TextProps::styled(
                requiem_i18n::t!("profile-settings-nameplate"),
                TextVariant::TEXT_MD_MEDIUM,
            )
            .color(TEXT_STRONG)
            .build(),
            nameplate,
        ]),
    ])
}

fn shown_name(draft: &ProfileDraft, data: &PreviewData, per_server: bool) -> String {
    let username = data
        .user
        .as_ref()
        .map(|u| u.username.clone())
        .unwrap_or_default();
    let display = if draft.display_name.trim().is_empty() {
        username
    } else {
        draft.display_name.clone()
    };
    if per_server && !draft.server.nick.trim().is_empty() {
        draft.server.nick.clone()
    } else {
        display
    }
}

fn avatar_node(draft: &ProfileDraft, data: &PreviewData, size: Option<AvatarStatusSize>) -> Node {
    let mut props = match &data.user {
        Some(u) => ui::user_avatar_props(u.id, &u.discriminator, &data.fonts),
        None => ui::user_avatar_props(Id::new(1), "0", &data.fonts),
    };
    let image = match &draft.avatar {
        PendingImage::Removed => None,
        pending => pending
            .shown()
            .map(|u| u.preview.clone())
            .or_else(|| data.avatar.clone()),
    };
    if let Some(image) = image {
        props = props.image(image);
    }
    match size {
        Some(size) => avatar_with_status(props, size, Some(PresenceStatus::Online)),
        None => props.build(),
    }
}

fn build_preview(draft: &ProfileDraft, data: &PreviewData, per_server: bool, sample: bool) -> Node {
    let Some(user) = &data.user else { return div() };
    let name = shown_name(draft, data, per_server);

    let preset = &PRESETS[draft.preview_preset % PRESETS.len()];
    let theme = if sample {
        Some(draft.preview_colors())
    } else {
        draft.theme_colors
    };
    let banner_color = rgb(if sample {
        theme.map_or(DEFAULT_BANNER_COLOR, |(p, _)| p)
    } else {
        draft
            .accent_color
            .unwrap_or(theme.map_or(DEFAULT_BANNER_COLOR, |(p, _)| p))
    });
    let banner_image = match &draft.banner {
        PendingImage::Removed => None,
        pending => pending
            .shown()
            .map(|u| u.preview.clone())
            .or_else(|| data.banner.clone()),
    }
    .filter(|_| !sample);
    let avatar = match &draft.avatar {
        PendingImage::Removed => None,
        pending => pending
            .shown()
            .map(|u| u.preview.clone())
            .or_else(|| data.avatar.clone()),
    };

    let custom = data.state.presence.custom_status.get();
    let status = data.state.presence.status.get().indicator();
    let account_popout = data.account_popout.clone();
    let actions = BubbleActions {
        on_open: {
            let (popout, custom) = (account_popout.clone(), custom.clone());
            Rc::new(move || popout.open_custom_status(custom.as_ref()))
        },
        on_clear: {
            let on_set = account_popout.on_set_custom_status.clone();
            Rc::new(move || on_set(None))
        },
    };
    let prompt = account_popout.prompt.peek();

    let view = ProfileCardView {
        user_id: user.id,
        discriminator: &user.discriminator,
        username: &user.username,
        display_name: &name,
        pronouns: &draft.pronouns,
        bio: &draft.bio,
        avatar,
        avatar_decoration: None,
        banner_image,
        banner_color,
        theme: theme.map(|(p, a)| (rgb(p), rgb(a))),
        name_color: sample.then(|| preset.name_color.map(rgb)).flatten(),
        status: Some(status),
        bot: false,
        badges_from: data.profile.as_deref(),
    };
    let footer = footer_slot(
        ButtonProps::default()
            .variant(ButtonVariant::Primary)
            .size(ButtonSize::Small)
            .label(requiem_i18n::t!("profile-settings-example-button").to_string())
            .full_width(true)
            .on_click(|| {})
            .build(),
    );
    profile_card(
        view,
        CardStatus {
            status: custom.as_ref(),
            actions: Some(&actions),
            prompt,
        },
        CardParts::default(),
        Some(footer),
        CardContainer::Inline,
        &data.fonts,
        &data.images,
        &data.backend,
    )
}

fn build_nameplate(draft: &ProfileDraft, data: &PreviewData, per_server: bool) -> Node {
    let avatar = avatar_node(draft, data, None).class(c::NAMEPLATE_AVATAR);
    div().class(c::NAMEPLATE).children([
        avatar,
        TextProps::styled(
            shown_name(draft, data, per_server),
            TextVariant::TEXT_MD_MEDIUM,
        )
        .color(TEXT_STRONG)
        .build(),
    ])
}

fn build_upsell(
    settings_ui: &Signal<SettingsUiState>,
    draft: &Signal<ProfileDraft>,
    data: &PreviewData,
) -> Node {
    let to_nitro = |text: &str| {
        let settings_ui = settings_ui.clone();
        ButtonProps::new(ButtonVariant::Primary, text, move || {
            settings_ui
                .update(|ui| ui.category = requiem_i18n::key!("settings-modal-nitro").to_string())
        })
        .size(ButtonSize::Small)
        .build()
    };
    let swatches = StackProps::horizontal()
        .full_width(false)
        .build()
        .children([
            color_field(
                settings_ui,
                draft,
                ColorTarget::PreviewPrimary,
                Some(&requiem_i18n::t!("profile-settings-primary")),
            ),
            color_field(
                settings_ui,
                draft,
                ColorTarget::PreviewAccent,
                Some(&requiem_i18n::t!("profile-settings-accent")),
            ),
        ]);
    let controls = div().class(c::UPSELL_CONTROLS).children([
        TextProps::new(requiem_i18n::t!("profile-settings-try-out-nitro"), 20.0_f32)
            .color(TEXT_STRONG)
            .weight(Weight::EXTRA_BOLD)
            .build(),
        TextProps::styled(
            requiem_i18n::t!("profile-settings-see-what-your-profile-could-look"),
            TextVariant::TEXT_SM_NORMAL,
        )
        .color(TEXT_DEFAULT)
        .build(),
        section(
            &requiem_i18n::t!("profile-settings-profile-theme"),
            vec![],
            vec![swatches],
            true,
        ),
        section(
            &requiem_i18n::t!("profile-settings-profile-banner"),
            vec![],
            vec![to_nitro(&requiem_i18n::t!(
                "profile-settings-change-banner"
            ))],
            true,
        ),
        section(
            &requiem_i18n::t!("profile-settings-animated-avatar"),
            vec![],
            vec![to_nitro(&requiem_i18n::t!(
                "profile-settings-add-animated-avatar"
            ))],
            true,
        ),
        section(
            &requiem_i18n::t!("profile-settings-display-name-styles"),
            vec![],
            vec![to_nitro(&requiem_i18n::t!("profile-settings-change-style"))],
            false,
        ),
        TextProps::styled(
            requiem_i18n::t!("profile-settings-nitro-also-includes-an-exclusive-nitro"),
            TextVariant::TEXT_SM_NORMAL,
        )
        .color(TEXT_DEFAULT)
        .build(),
    ]);

    let next_preset = {
        let draft = draft.clone();
        move || {
            draft.update(|d| {
                d.preview_preset = (d.preview_preset + 1) % PRESETS.len();
                d.preview_theme = None;
            })
        }
    };
    let preset_row = live(draft, {
        let next = next_preset.clone();
        move |d| {
            let preset = &PRESETS[d.preview_preset % PRESETS.len()];
            let shuffle = ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![c::PRESET_SHUFFLE.to_string()])
                .on_click(next.clone())
                .build()
                .child(icon(REFRESH, TEXT_SECONDARY).class(c::PRESET_SHUFFLE_ICON));
            let pill = ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![c::PRESET_PILL.to_string()])
                .on_click(next.clone())
                .build()
                .style(solid(rgb(preset.primary)))
                .child(
                    TextProps::styled(
                        requiem_i18n::text(preset.name).to_uppercase(),
                        TextVariant::TEXT_SM_BOLD,
                    )
                    .color(Color::WHITE)
                    .build(),
                );
            div().class(c::PRESET_ROW).children([shuffle, pill])
        }
    });
    let card = Node::reactive({
        let (draft, data) = (draft.clone(), data.clone());
        move || build_preview(&draft.get(), &data, false, true)
    });
    let preview = div().class(c::UPSELL_PREVIEW).children([
        TextProps::styled(
            requiem_i18n::t!("profile-settings-nitro-preview"),
            TextVariant::TEXT_SM_MEDIUM,
        )
        .color(TEXT_SECONDARY)
        .build(),
        preset_row,
        card,
    ]);
    div().class(c::UPSELL).children([controls, preview])
}

#[allow(clippy::too_many_arguments)]
fn build_server_tab(
    state: &AppState,
    settings_ui: &Signal<SettingsUiState>,
    draft: &Signal<ProfileDraft>,
    data: &PreviewData,
    queries: &QueryClient,
    http: Option<&HttpClient>,
    ui: &SettingsUiState,
) -> Node {
    let guilds = state.guilds.list.get();
    let chosen = draft.peek().server.guild_id;

    let options: Vec<SelectOption> = guilds
        .iter()
        .map(|g| SelectOption::new(g.id.to_string(), g.name.clone()))
        .collect();
    let select = SelectProps::new(
        {
            let settings_ui = settings_ui.clone();
            move || {
                settings_ui
                    .update(|ui| ui.profile_server_select_open = !ui.profile_server_select_open)
            }
        },
        {
            let (settings_ui, draft) = (settings_ui.clone(), draft.clone());
            move |value: String| {
                if let Ok(id) = value.parse::<u64>() {
                    draft.update(|d| {
                        d.server = ServerDraft {
                            guild_id: Some(Id::new(id)),
                            ..Default::default()
                        }
                    });
                }
                settings_ui.update(|ui| ui.profile_server_select_open = false);
            }
        },
    )
    .options(options)
    .placeholder(requiem_i18n::t!("profile-settings-choose-a-server").to_string())
    .open(ui.profile_server_select_open);
    let select = match chosen {
        Some(id) => select.selected(id.to_string()),
        None => select,
    }
    .build();
    let picker = section(
        &requiem_i18n::t!("profile-settings-server"),
        vec![],
        vec![div().class(c::FULL_WIDTH).child(select)],
        chosen.is_some(),
    );

    let Some(guild_id) = chosen else {
        let hint = TextProps::styled(
            requiem_i18n::t!("profile-settings-choose-a-server-to-change-how"),
            TextVariant::TEXT_SM_NORMAL,
        )
        .color(TEXT_SECONDARY)
        .build();
        return StackProps::vertical()
            .gap(0)
            .full_width(false)
            .build()
            .child(
                div()
                    .class(c::COLUMNS)
                    .child(div().class(c::FORM).children([picker, hint])),
            );
    };

    if let (Some(user), Some(http)) = (&data.user, http) {
        let (http, id) = (http.clone(), user.id);
        let profile = queries
            .query_default::<UserProfile, _, RestError>(
                &profile_query_key(user.id, Some(guild_id)),
                move || async move {
                    users::get_user_profile(
                        &http,
                        id,
                        Some(guild_id),
                        ProfileView::Plain,
                    )
                    .await
                },
            )
            .data;
        if let Some(profile) = profile.filter(|_| !draft.peek().server.seeded) {
            let nick = profile
                .guild_member
                .as_ref()
                .and_then(|m| m.nick.clone())
                .unwrap_or_default();
            draft.update(|d| {
                d.server = ServerDraft {
                    guild_id: Some(guild_id),
                    seeded: true,
                    nick: nick.clone(),
                    saved_nick: nick,
                }
            });
        }
    }

    let placeholder = shown_name(&draft.peek(), data, false);
    let nick_value = draft.peek().server.nick.clone();
    let nick = section(
        &requiem_i18n::t!("profile-settings-server-nickname"),
        vec![],
        vec![input(
            draft,
            &placeholder,
            nick_value,
            32,
            Rc::new(|d, v| d.server.nick = v),
        )],
        true,
    );
    let note = TextProps::styled(
        requiem_i18n::t!("profile-settings-this-instance-stores-a-nickname-per"),
        TextVariant::TEXT_SM_NORMAL,
    )
    .color(TEXT_SECONDARY)
    .build();
    let form = div().class(c::FORM).children([picker, nick, note]);
    StackProps::vertical()
        .gap(0)
        .full_width(false)
        .build()
        .child(
            div()
                .class(c::COLUMNS)
                .children([form, build_previews(draft, data, true)]),
        )
}

pub fn build_bottom_bar(
    state: &AppState,
    settings_ui: &Signal<SettingsUiState>,
    queries: &QueryClient,
    http: Option<&HttpClient>,
) -> Node {
    let nitro = has_nitro(state.users.current.get().as_ref());
    let draft = settings_ui.get().profile_draft;
    let (state, settings_ui, queries, http) = (
        state.clone(),
        settings_ui.clone(),
        queries.clone(),
        http.cloned(),
    );
    Node::reactive(move || {
        let d = draft.get();
        if d.dirty() {
            let message = TextProps::styled(
                requiem_i18n::t!("profile-settings-careful-you-have-unsaved-changes"),
                TextVariant::TEXT_MD_MEDIUM,
            )
            .color(Color::WHITE)
            .build();
            let reset = {
                let (draft, settings_ui) = (draft.clone(), settings_ui.clone());
                TextLinkProps::default()
                    .label(requiem_i18n::t!("profile-settings-reset").to_string())
                    .on_click(move || {
                        draft.update(|d| d.reset());

                        settings_ui.update(|ui| ui.profile_epoch += 1);
                    })
                    .build()
            };
            let save = {
                let (draft, state, queries, http) =
                    (draft.clone(), state.clone(), queries.clone(), http.clone());
                ButtonProps::default()
                    .variant(ButtonVariant::Active)
                    .size(ButtonSize::Small)
                    .label(requiem_i18n::t!("profile-settings-save-changes").to_string())
                    .on_click(move || save(&draft, &state, &queries, http.as_ref()))
                    .build()
            };
            div().class(c::BAR).children([
                message,
                StackProps::horizontal()
                    .align(Align::Center)
                    .full_width(false)
                    .build()
                    .children([reset, save]),
            ])
        } else if !nitro {
            let to_nitro = {
                let settings_ui = settings_ui.clone();
                move || {
                    settings_ui.update(|ui| {
                        ui.category = requiem_i18n::key!("settings-modal-nitro").to_string()
                    })
                }
            };
            let message = StackProps::horizontal()
                .gap(4)
                .full_width(false)
                .build()
                .children([
                    TextProps::styled(
                        requiem_i18n::t!("profile-settings-level-up-your-look-only-with"),
                        TextVariant::TEXT_MD_MEDIUM,
                    )
                    .color(TEXT_STRONG)
                    .build(),
                    TextProps::styled(
                        requiem_i18n::t!("profile-settings-nitro"),
                        TextVariant::TEXT_MD_MEDIUM,
                    )
                    .color(Color::from_rgb(0x4d, 0x96, 0xee))
                    .build(),
                ]);

            let heart = ButtonProps::default()
                .variant(ButtonVariant::OverlaySecondary)
                .icon(HEART_OUTLINE)
                .on_click(to_nitro.clone())
                .build();
            let try_it = ButtonProps::default()
                .variant(ButtonVariant::OverlayPrimary)
                .label(requiem_i18n::t!("profile-settings-try-it-out").to_string())
                .icon(NITRO_WHEEL)
                .on_click(to_nitro)
                .build();
            div().class(c::BAR).class(c::BAR_PROMPT).children([
                message,
                StackProps::horizontal()
                    .align(Align::Center)
                    .full_width(false)
                    .build()
                    .children([heart, try_it]),
            ])
        } else {
            div()
        }
    })
}

fn save(
    draft: &Signal<ProfileDraft>,
    state: &AppState,
    queries: &QueryClient,
    http: Option<&HttpClient>,
) {
    let d = draft.peek();
    let Some(http) = http.cloned() else { return };

    let account = AccountUpdate {
        display_name: (d.display_name != d.saved_display_name).then(|| d.display_name.clone()),
        avatar: match &d.avatar {
            PendingImage::Set(upload) => Some(Some(upload.data_uri.to_string())),
            PendingImage::Removed => Some(None),
            _ => None,
        },
    };
    let profile = ProfileUpdate {
        pronouns: (d.pronouns != d.saved_pronouns).then(|| d.pronouns.clone()),
        bio: (d.bio != d.saved_bio).then(|| d.bio.clone()),
        accent_color: (d.accent_color != d.saved_accent_color).then_some(d.accent_color),
        banner: match &d.banner {
            PendingImage::Set(upload) => Some(Some(upload.data_uri.to_string())),
            PendingImage::Removed => Some(None),
            _ => None,
        },
        theme_colors: (d.theme_colors != d.saved_theme_colors)
            .then(|| d.theme_colors.map(|(p, a)| [p, a])),
    };
    let nick = d
        .server
        .guild_id
        .filter(|_| d.server.seeded && d.server.nick != d.server.saved_nick)
        .map(|g| (g, d.server.nick.clone()));

    if let Some(name) = &account.display_name {
        state.users.current.update(|user| {
            if let Some(user) = user {
                user.global_name = if name.is_empty() {
                    None
                } else {
                    Some(name.clone())
                };
            }
        });
    }
    draft.update(|d| {
        d.saved_display_name = d.display_name.clone();
        d.saved_pronouns = d.pronouns.clone();
        d.saved_bio = d.bio.clone();
        d.saved_accent_color = d.accent_color;
        d.saved_theme_colors = d.theme_colors;
        d.server.saved_nick = d.server.nick.clone();
        for image in [&mut d.avatar, &mut d.banner] {
            *image = match mem::take(image) {
                PendingImage::Set(upload) => PendingImage::Saved(upload),
                PendingImage::Removed => PendingImage::Unchanged,
                other => other,
            };
        }
    });

    let me = state.users.current.peek().map(|u| u.id);
    let mut invalidates: Vec<String> = me
        .map(|id| profile_query_key(id, None))
        .into_iter()
        .collect();
    if let (Some(id), Some((guild, _))) = (me, &nick) {
        invalidates.push(profile_query_key(id, Some(*guild)));
    }
    queries.mutate(invalidates, move || async move {
        if !account.is_empty() {
            users::update_account(&http, &account).await?;
        }
        if !profile.is_empty() {
            users::update_profile(&http, &profile).await?;
        }
        if let Some((guild, nick)) = nick {
            let guild: Id<GuildMarker> = guild;
            users::update_member_nick(&http, guild, &nick).await?;
        }
        Ok::<(), RestError>(())
    });
}
