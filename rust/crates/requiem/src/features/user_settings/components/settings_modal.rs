use requiem_core::ui::language_settings as ls;
use requiem_ui::components::FormItemProps;
use requiem_ui::components::RadioGroupProps;
use requiem_ui::components::RadioOption;
use requiem_ui::components::SelectOption;
use requiem_ui::components::SelectProps;
use requiem_ui::widget::anim::ToggleState;
use crate::features::voice::voice_settings;
use crate::shell::classes::profile_settings;
use crate::shell::classes::settings_modal;
use requiem_core::plugins;
use requiem_core::rest::endpoints::auth;
use requiem_core::ui::account_popout;
use requiem_core::ui::context_menu;
use requiem_core::ui::emoji_picker;
use requiem_core::ui::plugin_install;
use requiem_core::ui::plugins_settings;
use requiem_core::ui;
use requiem_core::utils::link;
use requiem_core::utils::prefs;
use requiem_core::utils::time::TimeFormat;
use requiem_ui::classes::modal;
use requiem_ui::colors::{TEXT_ANCHOR, TEXT_LINK, TEXT_MUTED};
use requiem_ui::components::ModalSize;
use requiem_ui::components::SwitchProps;
use requiem_ui::components::{Align, Justify, Padding, StackProps};
use requiem_ui::components::{BadgeProps, BadgeVariant, badge};
use requiem_ui::components::{HeadingLevel, TextVariant};
use requiem_ui::icons::Icon;
use requiem_ui::reveal;
use requiem_ui::scroll_animation;
use requiem_ui::signal as requiem_ui_signal;
use requiem_ui::widget::anim::ToggleMemo;
use std::cell::Cell;
use std::rc::Rc;

use requiem_ui::components::{
    ButtonProps, ButtonSize, ButtonVariant, ConfirmModalProps, HeadingProps, HelpMessageProps,
    ModalProps, ScrollViewProps, TextProps, div, help_message_warning, icon, modal_search_box,
    scroll_view, scroll_view_with,
};
use requiem_ui::node::Node;
use requiem_ui::signal::Signal;
use skia_safe::Color;

use requiem_core::config::BackendConfig;
use requiem_core::images::ImageCache;
use requiem_core::models::User;
use requiem_core::query::QueryClient;
use requiem_core::rest::endpoints::auth::{AuthSession, SessionsResponse};
use requiem_core::rest::{HttpClient, RestError};
use requiem_core::state::AppState;
use requiem_core::ui::{
    Fonts, SettingsSubPage, SettingsUiState, TEXT_DEFAULT, TEXT_SECONDARY, TEXT_STRONG,
};
use requiem_ui::icons::{
    ACCESSIBILITY, ANGLE_BRACKETS, APPS, ARROW_SMALL_LEFT, BELL, BOOST_GEM, CHEVRON_LARGE_RIGHT,
    CIRCLE_QUESTION, CREDIT_CARD, DOOR_EXIT, DOWNLOAD, GIFT, KEYBOARD, LANGUAGE, LINK, MICROPHONE,
    NITRO_WHEEL, PAINT_PALETTE, PUZZLE_PIECE, SCREEN, SHIELD_LOCK, SHIELD_USER, SUBSCRIPTION, USER,
    USER_PLATFORM, X_LARGE, X_SMALL,
};

const SESSIONS_QUERY_KEY: &str = "auth/sessions";

const LOG_OUT_LABEL: &str = requiem_i18n::key!("settings-modal-log-out");

struct NavItem {
    key: &'static str,
    icon_path: Icon,

    badge: bool,
}

const fn item(key: &'static str, icon_path: Icon) -> NavItem {
    NavItem {
        key,
        icon_path,
        badge: false,
    }
}

fn nav_sections() -> Vec<(Option<&'static str>, Vec<NavItem>)> {
    vec![
        (
            None,
            vec![
                item(requiem_i18n::key!("settings-modal-account"), USER),
                item(
                    requiem_i18n::key!("settings-modal-content-social"),
                    SHIELD_USER,
                ),
                item(
                    requiem_i18n::key!("settings-modal-data-privacy"),
                    SHIELD_LOCK,
                ),
                item(requiem_i18n::key!("settings-modal-authorized-apps"), APPS),
                item(requiem_i18n::key!("settings-modal-connections"), LINK),
                item(requiem_i18n::key!("settings-modal-notifications"), BELL),
            ],
        ),
        (
            Some(requiem_i18n::key!("settings-modal-billing")),
            vec![
                item(requiem_i18n::key!("settings-modal-nitro"), NITRO_WHEEL),
                item(requiem_i18n::key!("settings-modal-server-boost"), BOOST_GEM),
                item(
                    requiem_i18n::key!("settings-modal-subscriptions"),
                    SUBSCRIPTION,
                ),
                item(requiem_i18n::key!("settings-modal-gift-inventory"), GIFT),
                item(requiem_i18n::key!("settings-modal-billing"), CREDIT_CARD),
            ],
        ),
        (
            Some(requiem_i18n::key!("settings-modal-experience")),
            vec![
                item(requiem_i18n::key!("settings-modal-voice-video"), MICROPHONE),
                NavItem {
                    key: requiem_i18n::key!("settings-modal-appearance"),
                    icon_path: PAINT_PALETTE,
                    badge: true,
                },
                item(
                    requiem_i18n::key!("settings-modal-accessibility"),
                    ACCESSIBILITY,
                ),
                item(requiem_i18n::key!("settings-modal-system"), KEYBOARD),
                item(requiem_i18n::key!("settings-modal-language-time"), LANGUAGE),
            ],
        ),
        (
            Some(requiem_i18n::key!("settings-modal-activity")),
            vec![item(
                requiem_i18n::key!("settings-modal-activity-privacy"),
                USER_PLATFORM,
            )],
        ),
        (
            None,
            vec![
                item(requiem_i18n::key!("settings-modal-plugins"), PUZZLE_PIECE),
                item(requiem_i18n::key!("settings-modal-app-updates"), DOWNLOAD),
            ],
        ),
        (
            None,
            vec![item(
                requiem_i18n::key!("settings-modal-developer"),
                ANGLE_BRACKETS,
            )],
        ),
        (
            None,
            vec![NavItem {
                key: requiem_i18n::key!("settings-modal-log-out"),
                icon_path: DOOR_EXIT,
                badge: false,
            }],
        ),
    ]
}

const ACCOUNT_CATEGORY: &str = requiem_i18n::key!("settings-modal-account");
const ACCESSIBILITY_CATEGORY: &str = requiem_i18n::key!("settings-modal-accessibility");
const LANGUAGE_CATEGORY: &str = requiem_i18n::key!("settings-modal-language-time");
const DEVELOPER_CATEGORY: &str = requiem_i18n::key!("settings-modal-developer");
const PLUGINS_CATEGORY: &str = requiem_i18n::key!("settings-modal-plugins");
const APP_UPDATES_CATEGORY: &str = requiem_i18n::key!("settings-modal-app-updates");
const VOICE_CATEGORY: &str = requiem_i18n::key!("settings-modal-voice-video");
const NOTIFICATIONS_CATEGORY: &str = requiem_i18n::key!("settings-modal-notifications");
const APPEARANCE_CATEGORY: &str = requiem_i18n::key!("settings-modal-appearance");

const ACCOUNT_SUBNAV: &[&str] = &[
    requiem_i18n::key!("settings-modal-account-info"),
    requiem_i18n::key!("settings-modal-password-security"),
    requiem_i18n::key!("settings-modal-account-standing"),
    requiem_i18n::key!("settings-modal-family-center"),
];

#[allow(clippy::too_many_arguments)]
pub fn build_logout_confirm(
    open: ToggleState,
    settings_ui: &Signal<SettingsUiState>,
    on_log_out: Rc<dyn Fn()>,
) -> Node {
    let settings_ui = settings_ui.clone();
    let close = move || settings_ui.update(|ui| ui.logout_confirm = ui.logout_confirm.set(false));
    ConfirmModalProps::new(
        requiem_i18n::t!("settings-modal-log-out"),
        requiem_i18n::t!("settings-modal-log-out"),
        close,
        open,
    )
    .message(requiem_i18n::t!("settings-modal-are-you-sure-you-want-to"))
    .critical()
    .on_confirm(move || on_log_out())
    .build()
}

pub fn build(
    state: &AppState,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    open: ToggleState,
    on_dismiss: Rc<dyn Fn()>,
    settings_ui: &Signal<SettingsUiState>,
    queries: &QueryClient,
    http: Option<&HttpClient>,
    account_popout: &account_popout::AccountPopoutState,
    emoji_picker: &emoji_picker::EmojiPickerState,
) -> Node {
    let category = settings_ui.get().category;
    let sidebar = build_sidebar(state, fonts, images, backend, &category, settings_ui);

    let sub_page = settings_ui.get().sub_page;

    let page = match sub_page {
        Some(SettingsSubPage::LoggedInDevices) => build_devices_page(state, queries, http),
        None if category == super::profile_settings::PROFILES_CATEGORY => {
            super::profile_settings::build_page(
                state,
                fonts,
                images,
                backend,
                settings_ui,
                queries,
                http,
                account_popout,
                emoji_picker,
            )
        }
        None if category == ACCOUNT_CATEGORY => {
            build_account_page(state, settings_ui, queries, http)
        }
        None if category == ACCESSIBILITY_CATEGORY => build_accessibility_page(),
        None if category == APPEARANCE_CATEGORY => build_appearance_page(settings_ui),
        None if category == VOICE_CATEGORY => voice_settings::build_page(settings_ui),
        None if category == NOTIFICATIONS_CATEGORY => {
            super::notification_settings::build_page(settings_ui)
        }
        None if category == APP_UPDATES_CATEGORY => super::update_settings::build_page(),
        None if category == LANGUAGE_CATEGORY => build_language_page(),
        None if category == DEVELOPER_CATEGORY => build_developer_page(),
        None if category == PLUGINS_CATEGORY => div()
            .class(settings_modal::SETTINGS_ACCOUNT_PAGE)
            .class(settings_modal::SETTINGS_PAGE_WIDE)
            .child(plugins_settings::build_page(
                state,
                Some({
                    let install = settings_ui.peek().plugin_install;
                    Rc::new(move || pick_plugin_folder(&install))
                }),
            )),
        None => build_unimplemented_page(&category),
    };

    let content_page = if sub_page.is_none() && category == VOICE_CATEGORY {
        voice_settings::scroll_view(
            settings_modal::SETTINGS_CONTENT_SCROLL,
            page,
        )
    } else if sub_page.is_none() && category == NOTIFICATIONS_CATEGORY {
        voice_settings::on_leave();
        super::notification_settings::scroll_view(
            settings_modal::SETTINGS_CONTENT_SCROLL,
            page,
        )
    } else {
        voice_settings::on_leave();

        scroll_view()
            .class(settings_modal::SETTINGS_CONTENT_SCROLL)
            .dom_id(format!(
                "settings-content-{category}-{}",
                sub_page.is_some()
            ))
            .child(page)
    };

    let on_back: Option<Rc<dyn Fn()>> = sub_page.map(|_| {
        let settings_ui = settings_ui.clone();
        Rc::new(move || {
            settings_ui.update(|ui| ui.sub_page = None);
        }) as Rc<dyn Fn()>
    });

    let mut content = div()
        .class(settings_modal::SETTINGS_CONTENT)
        .children([
            build_content_header(&category, sub_page, on_back, on_dismiss.clone()),
            content_page,
        ]);

    if sub_page.is_none() && category == super::profile_settings::PROFILES_CATEGORY {
        content = content.child(
            div()
                .class(profile_settings::BAR_WRAP)
                .child(
                    super::profile_settings::build_bottom_bar(state, settings_ui, queries, http)
                        .class(profile_settings::BAR_HOST),
                ),
        );
    }
    let body = div()
        .class(settings_modal::SETTINGS_BODY)
        .children([sidebar, content]);

    ModalProps::default()
        .size(ModalSize::Bare)
        .panel_class(settings_modal::SETTINGS_MODAL.to_string())
        .header(false)
        .dismissable(true)
        .on_dismiss(move || on_dismiss())
        .open(open)
        .body(body)
        .build()
}

pub(super) fn build_content_header(
    title: &str,
    sub_page: Option<SettingsSubPage>,
    on_back: Option<Rc<dyn Fn()>>,
    on_dismiss: Rc<dyn Fn()>,
) -> Node {
    let mut title_children: Vec<Node> = Vec::new();
    if let Some(back) = on_back {
        let arrow = icon(ARROW_SMALL_LEFT, TEXT_SECONDARY)
            .class(settings_modal::SETTINGS_CLOSE_ICON);
        title_children.push(
            ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![
                    modal::MODAL_CLOSE_BTN.to_string(),
                    settings_modal::SETTINGS_CLOSE_BTN.to_string(),
                ])
                .on_click(move || back())
                .build()
                .child(arrow),
        );
    }
    match sub_page {
        Some(sub_page) => {
            title_children.push(
                TextProps::styled(requiem_i18n::text(title), TextVariant::TEXT_MD_NORMAL)
                    .color(TEXT_SECONDARY)
                    .build(),
            );
            title_children.push(
                TextProps::styled("/".to_string(), TextVariant::TEXT_MD_NORMAL)
                    .color(TEXT_SECONDARY)
                    .build(),
            );
            title_children.push(
                TextProps::styled(sub_page.title().to_string(), TextVariant::TEXT_MD_MEDIUM)
                    .color(TEXT_DEFAULT)
                    .build(),
            );
        }
        None => {
            title_children.push(
                TextProps::styled(requiem_i18n::text(title), TextVariant::TEXT_MD_MEDIUM)
                    .color(TEXT_STRONG)
                    .build(),
            );
        }
    }
    let title_row = div()
        .class(settings_modal::SETTINGS_CONTENT_TITLE)
        .children(title_children);

    let close_icon = icon(X_LARGE, TEXT_SECONDARY).classes([
        modal::MODAL_CLOSE_ICON,
        settings_modal::SETTINGS_CLOSE_ICON,
    ]);
    let close_btn = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![
            modal::MODAL_CLOSE_BTN.to_string(),
            settings_modal::SETTINGS_CLOSE_BTN.to_string(),
        ])
        .on_click(move || on_dismiss())
        .build()
        .child(close_icon);

    div()
        .class(settings_modal::SETTINGS_CONTENT_HEADER)
        .children([title_row, close_btn])
}

fn build_account_page(
    state: &AppState,
    settings_ui: &Signal<SettingsUiState>,
    queries: &QueryClient,
    http: Option<&HttpClient>,
) -> Node {
    let user = state.users.current.get();
    let user = user.as_ref();
    let username = user.map_or_else(|| "—".to_string(), |u| u.username.clone());

    let email = user.and_then(|u| u.email.clone());
    let phone = user.and_then(|u| u.phone.clone());
    let email_verified = user.and_then(|u| u.verified).unwrap_or(false);

    let revealed = settings_ui.get().email_revealed;

    let mut account_info_rows = Vec::new();

    if !email_verified {
        account_info_rows.push(build_unverified_email_banner());
    }
    account_info_rows.push(build_info_row(
        &requiem_i18n::t!("settings-modal-username"),
        &username,
        None,
        &requiem_i18n::t!("settings-modal-edit"),
        settings_modal::SETTINGS_FIELD_ROW_USERNAME,
    ));
    account_info_rows.push(build_email_row(email.as_deref(), revealed, settings_ui));
    account_info_rows.push(build_info_row(
        &requiem_i18n::t!("settings-modal-phone-number"),
        phone.as_deref().unwrap_or(&requiem_i18n::t!(
            "settings-modal-you-haven-t-added-a-phone"
        )),
        None,
        &if phone.is_some() {
            requiem_i18n::t!("settings-modal-edit")
        } else {
            requiem_i18n::t!("settings-modal-add")
        },
        settings_modal::SETTINGS_FIELD_ROW_PHONE,
    ));

    let account_info = build_section_group(
        &requiem_i18n::t!("settings-modal-account-info"),
        account_info_rows,
    );

    let mfa_enabled = user.and_then(|u| u.mfa_enabled).unwrap_or(false);
    let password_security = build_section_group(
        &requiem_i18n::t!("settings-modal-password-security"),
        vec![
            build_info_row(
                &requiem_i18n::t!("settings-modal-password"),
                "",
                None,
                &requiem_i18n::t!("settings-modal-edit"),
                settings_modal::SETTINGS_FIELD_ROW_PASSWORD,
            ),
            build_chevron_row(
                &requiem_i18n::t!("settings-modal-multi-factor-authentication"),
                None,
                &if mfa_enabled {
                    requiem_i18n::t!("settings-modal-manage")
                } else {
                    requiem_i18n::t!("settings-modal-set-up")
                },
            ),
            build_devices_row(queries, http, settings_ui),
        ],
    );

    let account_standing = build_account_standing_section();

    let family_center = build_section_group(
        &requiem_i18n::t!("settings-modal-family-center"),
        vec![build_chevron_row(
            &requiem_i18n::t!("settings-modal-set-up-family-center"),
            Some(&requiem_i18n::t!(
                "settings-modal-stay-informed-about-your-teen-s"
            )),
            "",
        )],
    );

    div()
        .class(settings_modal::SETTINGS_ACCOUNT_PAGE)
        .children([
            account_info,
            divider(),
            password_security,
            divider(),
            account_standing,
            divider(),
            family_center,
        ])
}

fn build_account_standing_section() -> Node {
    let leading = build_leading_icon(CIRCLE_QUESTION, TEXT_MUTED);
    let row = build_chevron_row_with_leading(
        Some(leading),
        &requiem_i18n::t!("settings-modal-loading"),
        Some(&requiem_i18n::t!(
            "settings-modal-loading-your-account-standing"
        )),
        "",
    );
    build_section_group(
        &requiem_i18n::t!("settings-modal-account-standing"),
        vec![row],
    )
}

fn build_leading_icon(path: Icon, color: Color) -> Node {
    let glyph = icon(path, color)
        .class(settings_modal::SETTINGS_INFO_ROW_LEADING_ICON);
    div()
        .class(settings_modal::SETTINGS_INFO_ROW_LEADING)
        .child(glyph)
}

fn divider() -> Node {
    div().class(settings_modal::SETTINGS_SECTION_DIVIDER)
}

fn build_section_group(title: &str, rows: Vec<Node>) -> Node {
    let heading_node = TextProps::new(title.to_string(), 24.0_f32)
        .color(TEXT_STRONG)
        .build()
        .class(settings_modal::SETTINGS_SECTION_HEADING);
    let heading_divider =
        div().class(settings_modal::SETTINGS_SECTION_HEADING_DIVIDER);
    let rows_container = div()
        .class(settings_modal::SETTINGS_SECTION_ROWS)
        .children(rows);
    StackProps::vertical()
        .gap(0)
        .full_width(false)
        .build()
        .children([heading_node, heading_divider, rows_container])
}

fn build_unverified_email_banner() -> Node {
    let title = TextProps::styled(
        requiem_i18n::t!("settings-modal-unverified-email").to_string(),
        TextVariant::TEXT_MD_MEDIUM,
    )
    .color(TEXT_STRONG)
    .build();

    let body = TextProps::styled(
        requiem_i18n::t!("settings-modal-please-check-your-email-and-follow").to_string(),
        TextVariant::TEXT_SM_MEDIUM,
    )
    .color(TEXT_DEFAULT)
    .build();
    let text_col = div()
        .class(settings_modal::SETTINGS_WARNING_TEXT)
        .children([title, body]);
    let resend_btn = ButtonProps::new(
        ButtonVariant::Secondary,
        requiem_i18n::t!("settings-modal-resend-verification-email"),
        || {},
    )
    .build();
    help_message_warning(
        HelpMessageProps::default().content(
            StackProps::horizontal()
                .align(Align::Center)
                .full_width(false)
                .build()
                .children([text_col, resend_btn]),
        ),
    )
}

fn build_info_row(
    label: &str,
    value: &str,
    link: Option<&str>,
    action_label: &str,
    column_class: &str,
) -> Node {
    let link = link.map(|label| (label, Rc::new(|| {}) as Rc<dyn Fn()>));
    build_info_row_with_link(label, value, link, action_label, column_class)
}

fn build_info_row_with_link(
    label: &str,
    value: &str,
    link: Option<(&str, Rc<dyn Fn()>)>,
    action_label: &str,
    column_class: &str,
) -> Node {
    let label_node = TextProps::styled(label.to_string(), TextVariant::TEXT_MD_MEDIUM)
        .color(TEXT_STRONG)
        .build();

    let mut value_group_children = Vec::new();
    if !value.is_empty() {
        value_group_children.push(
            TextProps::styled(value.to_string(), TextVariant::TEXT_MD_MEDIUM)
                .color(TEXT_DEFAULT)
                .build(),
        );
    }
    if let Some((link_label, on_click)) = link {
        value_group_children.push(build_link(link_label, on_click));
    }
    let value_group = StackProps::horizontal()
        .align(Align::Center)
        .gap(4)
        .full_width(false)
        .build()
        .children(value_group_children);

    let edit_btn = ButtonProps::default()
        .variant(ButtonVariant::Secondary)
        .label(action_label.to_string())
        .build();
    let control = StackProps::horizontal()
        .align(Align::Center)
        .justify(Justify::End)
        .gap(16)
        .full_width(false)
        .build()
        .children([value_group, edit_btn]);

    div()
        .classes(vec![
            settings_modal::SETTINGS_FIELD_ROW.to_string(),
            column_class.to_string(),
        ])
        .children([label_node, control])
}

fn build_email_row(
    email: Option<&str>,
    revealed: bool,
    settings_ui: &Signal<SettingsUiState>,
) -> Node {
    let Some(email) = email else {
        return build_info_row(
            &requiem_i18n::t!("settings-modal-email"),
            &requiem_i18n::t!("settings-modal-you-haven-t-added-an-email"),
            None,
            &requiem_i18n::t!("settings-modal-add"),
            settings_modal::SETTINGS_FIELD_ROW_EMAIL,
        );
    };

    let displayed = if revealed {
        email.to_string()
    } else {
        User::censor_email(email)
    };
    let link_label = if revealed {
        requiem_i18n::t!("settings-modal-hide")
    } else {
        requiem_i18n::t!("settings-modal-reveal")
    };

    let on_toggle: Rc<dyn Fn()> = {
        let settings_ui = settings_ui.clone();
        Rc::new(move || {
            settings_ui.update(|ui| ui.email_revealed = !ui.email_revealed);
        })
    };

    build_info_row_with_link(
        &requiem_i18n::t!("settings-modal-email"),
        &displayed,
        Some((link_label.as_str(), on_toggle)),
        &requiem_i18n::t!("settings-modal-edit"),
        settings_modal::SETTINGS_FIELD_ROW_EMAIL,
    )
}

fn build_devices_row(
    queries: &QueryClient,
    http: Option<&HttpClient>,
    settings_ui: &Signal<SettingsUiState>,
) -> Node {
    let trailing = match http {
        Some(http) => {
            let http = http.clone();
            let result =
                queries.query_default::<SessionsResponse, _, RestError>(
                    SESSIONS_QUERY_KEY,
                    move || async move {
                        auth::get_sessions(&http).await
                    },
                );
            match (result.data.as_deref(), result.is_loading) {
                (Some(sessions), _) => device_count_label(sessions.user_sessions.len()),
                (None, true) => String::new(),
                (None, false) => requiem_i18n::t!("settings-modal-manage").to_string(),
            }
        }

        None => match queries.peek::<SessionsResponse>(SESSIONS_QUERY_KEY) {
            Some(sessions) => device_count_label(sessions.user_sessions.len()),
            None => requiem_i18n::t!("settings-modal-manage").to_string(),
        },
    };

    let on_click: Rc<dyn Fn()> = {
        let settings_ui = settings_ui.clone();
        Rc::new(move || {
            settings_ui.update(|ui| ui.sub_page = Some(SettingsSubPage::LoggedInDevices));
        })
    };

    build_chevron_row_with(
        None,
        &requiem_i18n::t!("settings-modal-logged-in-devices"),
        None,
        &trailing,
        Some(on_click),
    )
}

fn build_devices_page(state: &AppState, queries: &QueryClient, http: Option<&HttpClient>) -> Node {
    let heading = TextProps::new(
        SettingsSubPage::LoggedInDevices.title().to_string(),
        24.0_f32,
    )
    .color(TEXT_STRONG)
    .build();
    let description = TextProps::styled(
        requiem_i18n::t!("settings-modal-these-are-all-the-devices-that").to_string(),
        TextVariant::TEXT_SM_NORMAL,
    )
    .color(TEXT_SECONDARY)
    .build();
    let intro = StackProps::vertical()
        .full_width(false)
        .build()
        .children([heading, description]);

    let mut children = vec![intro];

    let sessions = match http {
        Some(http) => {
            let http_for_query = http.clone();
            queries
                .query_default::<SessionsResponse, _, RestError>(
                    SESSIONS_QUERY_KEY,
                    move || async move {
                        auth::get_sessions(&http_for_query).await
                    },
                )
                .data
        }

        None => queries.peek::<SessionsResponse>(SESSIONS_QUERY_KEY),
    };

    match sessions {
        Some(sessions) => {
            let auth_session_id_hash = state.connection.auth_session_id_hash.get();
            let current_hash = auth_session_id_hash.as_deref();
            let (current, others): (Vec<_>, Vec<_>) = sessions
                .user_sessions
                .iter()
                .partition(|s| current_hash.is_some_and(|hash| hash == s.id_hash));

            if let Some(session) = current.first() {
                children.push(build_device_group(
                    &requiem_i18n::t!("settings-modal-current-device"),
                    vec![build_device_row(session, None)],
                ));
            }

            let note = build_older_devices_note();
            if others.is_empty() {
                children.push(note);
            } else {
                let mut rows: Vec<Node> = others
                    .iter()
                    .map(|session| {
                        let on_revoke =
                            revoke_handler(queries, http, vec![session.id_hash.clone()]);
                        build_device_row(session, Some(on_revoke))
                    })
                    .collect();
                rows.push(note);
                children.push(build_device_group(
                    &requiem_i18n::t!("settings-modal-other-devices"),
                    rows,
                ));
            }
            children.push(build_logout_all_section(queries, http, &sessions));
        }
        None => {
            children.push(
                TextProps::styled(
                    requiem_i18n::t!("settings-modal-loading-your-devices").to_string(),
                    TextVariant::TEXT_MD_NORMAL,
                )
                .color(TEXT_SECONDARY)
                .build(),
            );
        }
    }

    div()
        .class(settings_modal::SETTINGS_DEVICES_PAGE)
        .children(children)
}

fn revoke_handler(
    queries: &QueryClient,
    http: Option<&HttpClient>,
    hashes: Vec<String>,
) -> Rc<dyn Fn()> {
    let Some(http) = http.cloned() else {
        return Rc::new(|| {});
    };
    let queries = queries.clone();
    Rc::new(move || {
        let http = http.clone();
        let hashes = hashes.clone();
        queries.mutate(vec![SESSIONS_QUERY_KEY.to_string()], move || async move {
            auth::revoke_sessions(&http, &hashes).await
        });
    })
}

fn build_device_group(label: &str, rows: Vec<Node>) -> Node {
    let label_node = TextProps::styled(label.to_string(), TextVariant::TEXT_MD_SEMIBOLD)
        .color(TEXT_MUTED)
        .build();
    let list = StackProps::vertical()
        .gap(24)
        .full_width(false)
        .build()
        .children(rows);
    StackProps::vertical()
        .gap(24)
        .full_width(false)
        .build()
        .children([label_node, list])
}

fn build_device_row(session: &AuthSession, on_revoke: Option<Rc<dyn Fn()>>) -> Node {
    let icon_node = icon(SCREEN, TEXT_DEFAULT)
        .class(settings_modal::SETTINGS_DEVICE_ICON);

    let os = session
        .client_info
        .os
        .clone()
        .unwrap_or_else(|| requiem_i18n::t!("settings-modal-unknown").to_string());
    let mut title_children = vec![
        TextProps::styled(os, TextVariant::TEXT_MD_SEMIBOLD)
            .color(TEXT_STRONG)
            .build(),
    ];
    if let Some(platform) = session.client_info.platform.as_deref() {
        title_children.push(
            TextProps::styled("•".to_string(), TextVariant::TEXT_MD_MEDIUM)
                .color(TEXT_SECONDARY)
                .build(),
        );
        title_children.push(
            TextProps::styled(platform.to_string(), TextVariant::TEXT_MD_MEDIUM)
                .color(TEXT_SECONDARY)
                .build(),
        );
    }
    let title = StackProps::horizontal()
        .align(Align::Center)
        .full_width(false)
        .build()
        .children(title_children);

    let subtitle_text = session
        .client_info
        .location
        .clone()
        .unwrap_or_else(|| requiem_i18n::t!("settings-modal-unknown-location").to_string());
    let subtitle = TextProps::styled(subtitle_text, TextVariant::TEXT_SM_NORMAL)
        .color(TEXT_MUTED)
        .build();
    let stack = div()
        .class(settings_modal::SETTINGS_DEVICE_STACK)
        .children([title, subtitle]);

    let mut row_children = vec![icon_node, stack];
    if let Some(on_revoke) = on_revoke {
        let glyph = icon(X_SMALL, TEXT_SECONDARY)
            .class(settings_modal::SETTINGS_DEVICE_REVOKE_ICON);
        row_children.push(
            ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![
                    settings_modal::SETTINGS_DEVICE_REVOKE.to_string(),
                ])
                .on_click(move || on_revoke())
                .build()
                .child(glyph),
        );
    }

    div()
        .class(settings_modal::SETTINGS_DEVICE_ROW)
        .children(row_children)
}

fn build_older_devices_note() -> Node {
    let icon_node = icon(SCREEN, TEXT_SECONDARY)
        .class(settings_modal::SETTINGS_DEVICE_ICON);
    let title = TextProps::styled(
        requiem_i18n::t!("settings-modal-some-older-devices-may-not-be").to_string(),
        TextVariant::TEXT_MD_SEMIBOLD,
    )
    .color(TEXT_STRONG)
    .build();

    let lead = TextProps::styled(
        requiem_i18n::t!("settings-modal-to-log-them-out-please").to_string(),
        TextVariant::TEXT_SM_NORMAL,
    )
    .color(TEXT_MUTED)
    .build();
    let link = TextProps::styled(
        requiem_i18n::t!("settings-modal-change-your-password").to_string(),
        TextVariant::TEXT_SM_NORMAL,
    )
    .color(TEXT_ANCHOR)
    .build();
    let sub = StackProps::horizontal()
        .align(Align::Center)
        .gap(0)
        .full_width(false)
        .build()
        .children([lead, link]);
    let stack = div()
        .class(settings_modal::SETTINGS_DEVICE_STACK)
        .children([title, sub]);
    div()
        .class(settings_modal::SETTINGS_DEVICE_ROW)
        .children([icon_node, stack])
}

fn build_logout_all_section(
    queries: &QueryClient,
    http: Option<&HttpClient>,
    sessions: &SessionsResponse,
) -> Node {
    let title = TextProps::styled(
        requiem_i18n::t!("settings-modal-log-out-of-all-known-devices").to_string(),
        TextVariant::TEXT_MD_MEDIUM,
    )
    .color(TEXT_STRONG)
    .build();
    let body = TextProps::styled(
        requiem_i18n::t!("settings-modal-you-ll-have-to-log-back").to_string(),
        TextVariant::TEXT_SM_NORMAL,
    )
    .color(TEXT_MUTED)
    .build();

    let all_hashes: Vec<String> = sessions
        .user_sessions
        .iter()
        .map(|s| s.id_hash.clone())
        .collect();
    let on_click = revoke_handler(queries, http, all_hashes);

    let button_node = ButtonProps::default()
        .variant(ButtonVariant::CriticalPrimary)
        .size(ButtonSize::Small)
        .label(requiem_i18n::t!("settings-modal-log-out-all-known-devices").to_string())
        .on_click(move || on_click())
        .build();

    div()
        .class(settings_modal::SETTINGS_LOGOUT_ALL)
        .children([title, body, button_node])
}

fn device_count_label(count: usize) -> String {
    if count == 1 {
        requiem_i18n::t!("settings-modal-1-device").to_string()
    } else {
        requiem_i18n::t!("settings-modal-n-devices", count = count)
    }
}

thread_local! {

    static SMOOTH_SCROLLING: (Signal<bool>, Cell<ToggleState>) = {
        let on = scroll_animation::smooth_scrolling();
        (requiem_ui_signal::signal(on), Cell::new(ToggleState::new(on)))
    };
}

fn build_accessibility_page() -> Node {
    let switch = Node::reactive(move || {
        let (value, toggle) =
            SMOOTH_SCROLLING.with(|(value, toggle)| (value.clone(), toggle.get()));
        let on = value.get();
        let state = toggle.set(on);
        SMOOTH_SCROLLING.with(|(_, toggle)| toggle.set(state));
        SwitchProps::new(state, move |on: bool| {
            scroll_animation::set_smooth_scrolling(on);
            value.set(on);
        })
        .label(requiem_i18n::t!("settings-modal-smooth-scrolling").to_string())
        .description(requiem_i18n::t!("settings-modal-glide-when-you-scroll-with-the").to_string())
        .build()
    });
    let section = build_section_group(&requiem_i18n::t!("settings-modal-scrolling"), vec![switch]);
    div()
        .class(settings_modal::SETTINGS_ACCOUNT_PAGE)
        .child(section)
}

fn build_appearance_page(settings_ui: &Signal<SettingsUiState>) -> Node {
    let current = prefs::theme();
    let options: Vec<RadioOption> = [
        ("light", requiem_i18n::t!("settings-modal-theme-light")),
        ("ash", requiem_i18n::t!("settings-modal-theme-ash")),
        ("dark", requiem_i18n::t!("settings-modal-theme-dark")),
        ("onyx", requiem_i18n::t!("settings-modal-theme-onyx")),
    ]
    .into_iter()
    .map(|(value, label)| {
        RadioOption::new(
            value,
            label.to_string(),
            ToggleState::new(current == value),
        )
    })
    .collect();
    let settings_ui = settings_ui.clone();
    let group = RadioGroupProps::new(move |value: String| {
        prefs::set_theme(&value);
        requiem_ui::set_theme_name(match value.as_str() {
            "light" => "light",
            "ash" => "ash",
            "onyx" => "onyx",
            _ => "dark",
        });
        settings_ui.update(|_| {});
    })
    .label(requiem_i18n::t!("settings-modal-theme").to_string())
    .options(options)
    .build();
    div()
        .class(settings_modal::SETTINGS_ACCOUNT_PAGE)
        .child(group)
}

thread_local! {
    static DEVELOPER_MODE: (Signal<bool>, Cell<ToggleState>) = {
        let on = prefs::developer_mode();
        (requiem_ui_signal::signal(on), Cell::new(ToggleState::new(on)))
    };
}

fn build_developer_page() -> Node {
    let switch = Node::reactive(move || {
        let (value, toggle) = DEVELOPER_MODE.with(|(value, toggle)| (value.clone(), toggle.get()));
        let on = value.get();
        let state = toggle.set(on);
        DEVELOPER_MODE.with(|(_, toggle)| toggle.set(state));
        SwitchProps::new(state, move |on: bool| {
            prefs::set_developer_mode(on);
            value.set(on);
        })
        .label(requiem_i18n::t!("settings-modal-developer-mode").to_string())
        .description(requiem_i18n::t!("settings-modal-developer-mode-description").to_string())
        .build()
    });
    div()
        .class(settings_modal::SETTINGS_ACCOUNT_PAGE)
        .child(switch)
        .child(
            ButtonProps::new(
                ButtonVariant::Secondary,
                requiem_i18n::t!("settings-modal-open-logs"),
                || {
                    if let Some(dir) = prefs::log_dir() {
                        link::open_url(&dir.to_string_lossy());
                    }
                },
            )
            .build(),
        )
        .child(
            ButtonProps::new(
                ButtonVariant::Secondary,
                requiem_i18n::t!("settings-modal-export-logs"),
                || {
                    if let Some(path) = prefs::export_logs() {
                        link::open_url(&path.to_string_lossy());
                    }
                },
            )
            .build(),
        )
}

thread_local! {

    static TIME_FORMAT_TOGGLES: Vec<(TimeFormat, ToggleMemo)> = [TimeFormat::Auto, TimeFormat::Hour12, TimeFormat::Hour24].into_iter().map(|f| (f, ToggleMemo::new(false))).collect();
}

fn build_language_page() -> Node {
    let chosen = ls::chosen().get();
    let options: Vec<SelectOption> = ls::LANGUAGES
        .iter()
        .map(|language| {
            let english = TextProps::styled(
                requiem_i18n::text(language.english),
                TextVariant::TEXT_SM_NORMAL,
            )
            .color(TEXT_MUTED)
            .build();
            SelectOption::new(language.tag, language.native)
                .with_leading(ls::flag(language))
                .with_trailing(english)
        })
        .collect();
    let (open, picked) = (ls::open(), ls::open());
    let select = SelectProps::new(
        move || open.set(!open.peek()),
        move |tag: String| {
            picked.set(false);
            ls::choose(&tag);
        },
    )
    .options(options);
    let select = match ls::LANGUAGES.iter().find(|l| l.tag == chosen) {
        Some(language) => select.trigger_leading(ls::flag(language)).trigger_trailing(
            TextProps::styled(
                requiem_i18n::text(language.english),
                TextVariant::TEXT_SM_NORMAL,
            )
            .color(TEXT_MUTED)
            .build(),
        ),
        None => select,
    }
    .selected(chosen)
    .open(ls::open().get())
    .build();
    let language = FormItemProps::new(requiem_i18n::t!("settings-language-select"), select)
        .description(requiem_i18n::t!("settings-language-description"))
        .build();

    let format = ls::time_format().get();
    let radio = |value: TimeFormat, label: String| {
        let state = TIME_FORMAT_TOGGLES
            .with(|toggles| {
                toggles
                    .iter()
                    .find(|(f, _)| *f == value)
                    .map(|(_, memo)| memo.track(format == value))
            })
            .unwrap_or_else(|| ToggleState::new(format == value));
        RadioOption::new(value.key(), label, state)
    };
    let time_format = RadioGroupProps::new(|key: String| {
        if let Some(format) = TimeFormat::from_key(&key) {
            ls::choose_time_format(format);
        }
    })
    .label(requiem_i18n::t!("settings-language-time-format"))
    .options(vec![
        radio(
            TimeFormat::Auto,
            requiem_i18n::t!("settings-language-time-auto"),
        ),
        radio(
            TimeFormat::Hour12,
            requiem_i18n::t!("settings-language-time-12h"),
        ),
        radio(
            TimeFormat::Hour24,
            requiem_i18n::t!("settings-language-time-24h"),
        ),
    ])
    .build();
    div()
        .class(settings_modal::SETTINGS_ACCOUNT_PAGE)
        .class(settings_modal::SETTINGS_PAGE_WIDE)
        .child(
            StackProps::vertical()
                .gap(24)
                .build()
                .children([language, time_format]),
        )
}

fn pick_plugin_folder(install: &Signal<plugin_install::PluginInstallState>) {
    let Some(dir) = rfd::FileDialog::new()
        .set_title(requiem_i18n::t!("plugins-install-folder"))
        .pick_folder()
    else {
        return;
    };
    match plugins::preview_dir(&dir) {
        Ok(preview) => plugin_install::open(install, preview),
        Err(error) => context_menu::notify_failure(&requiem_i18n::t!(
            "plugins-install-failed",
            error = error
        )),
    }
}

fn build_unimplemented_page(category: &str) -> Node {
    let body = TextProps::styled(
        requiem_i18n::t!("settings-modal-this-settings-page-hasn-t-been").to_string(),
        TextVariant::TEXT_MD_NORMAL,
    )
    .color(TEXT_SECONDARY)
    .build();
    let section = build_section_group(&requiem_i18n::text(category), vec![body]);
    div()
        .class(settings_modal::SETTINGS_ACCOUNT_PAGE)
        .child(section)
}

fn build_link(label: &str, on_click: Rc<dyn Fn()>) -> Node {
    let label_node = TextProps::styled(label.to_string(), TextVariant::TEXT_MD_NORMAL)
        .color(TEXT_LINK)
        .build();
    ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .on_click(move || on_click())
        .build()
        .child(label_node)
}

fn build_chevron_row(label: &str, subtitle: Option<&str>, trailing: &str) -> Node {
    build_chevron_row_with_leading(None, label, subtitle, trailing)
}

fn build_chevron_row_with_leading(
    leading: Option<Node>,
    label: &str,
    subtitle: Option<&str>,
    trailing: &str,
) -> Node {
    build_chevron_row_with(leading, label, subtitle, trailing, None)
}

fn build_chevron_row_with(
    leading: Option<Node>,
    label: &str,
    subtitle: Option<&str>,
    trailing: &str,
    on_click: Option<Rc<dyn Fn()>>,
) -> Node {
    let label_node = TextProps::styled(label.to_string(), TextVariant::TEXT_MD_MEDIUM)
        .color(TEXT_STRONG)
        .build();
    let mut left_children = vec![label_node];
    if let Some(subtitle) = subtitle {
        left_children.push(
            TextProps::styled(subtitle.to_string(), TextVariant::TEXT_SM_NORMAL)
                .color(TEXT_SECONDARY)
                .build(),
        );
    }
    let left = div()
        .class(settings_modal::SETTINGS_INFO_ROW_LEFT_STACK)
        .children(left_children);

    let mut right_children = Vec::new();
    if !trailing.is_empty() {
        right_children.push(
            TextProps::styled(trailing.to_string(), TextVariant::TEXT_MD_MEDIUM)
                .color(TEXT_DEFAULT)
                .build(),
        );
    }
    right_children.push(
        icon(CHEVRON_LARGE_RIGHT, TEXT_SECONDARY)
            .class(settings_modal::SETTINGS_CHEVRON_ICON),
    );
    let right = div()
        .class(settings_modal::SETTINGS_INFO_ROW_RIGHT)
        .children(right_children);

    let mut row_children = Vec::with_capacity(3);
    row_children.extend(leading);
    row_children.push(left);
    row_children.push(right);

    let on_click = on_click.unwrap_or_else(|| Rc::new(|| {}));
    ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![
            settings_modal::SETTINGS_INFO_ROW.to_string(),
        ])
        .on_click(move || on_click())
        .build()
        .children(row_children)
}

fn build_sidebar(
    state: &AppState,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    category: &str,
    settings_ui: &Signal<SettingsUiState>,
) -> Node {
    let mut nav_children = Vec::new();
    let sections = nav_sections();
    let last_index = sections.len().saturating_sub(1);
    for (index, (label, items)) in sections.into_iter().enumerate() {
        nav_children.push(build_section(
            label,
            &items,
            index != last_index,
            category,
            settings_ui,
        ));
    }

    let nav_list = scroll_view_with(ScrollViewProps {
        top_fade: Some("background-base-lower".into()),
        ..Default::default()
    })
    .class(settings_modal::SETTINGS_NAV_LIST)
    .children(nav_children);
    let nav_list_wrapper = div()
        .class(settings_modal::SETTINGS_NAV_LIST_WRAPPER)
        .child(nav_list);

    div()
        .class(settings_modal::SETTINGS_SIDEBAR)
        .children([
            build_header(state, fonts, images, backend, category, settings_ui),
            build_search(),
            nav_list_wrapper,
        ])
}

fn build_header(
    state: &AppState,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    category: &str,
    settings_ui: &Signal<SettingsUiState>,
) -> Node {
    let Some(user) = state.users.current.get() else {
        return div();
    };
    let user = &user;

    let image = user.avatar.as_deref().and_then(|hash| {
        let url = backend.avatar_url(user.id, hash);
        images.get_or_decode_image(&url)
    });
    let display_name = user
        .global_name
        .clone()
        .unwrap_or_else(|| user.username.clone());

    let mut avatar_props = ui::user_avatar_props(user.id, &user.discriminator, fonts);
    if let Some(image) = image {
        avatar_props = avatar_props.image(image);
    }
    let avatar_node = avatar_props
        .build()
        .class(settings_modal::SETTINGS_AVATAR);

    let identity = StackProps::vertical()
        .gap(0)
        .full_width(false)
        .build()
        .children([
            TextProps::styled(display_name, TextVariant::TEXT_MD_MEDIUM)
                .color(TEXT_STRONG)
                .build(),
            TextProps::styled(
                requiem_i18n::t!("settings-modal-edit-profiles").to_string(),
                TextVariant::TEXT_SM_NORMAL,
            )
            .color(TEXT_MUTED)
            .build(),
        ]);

    let mut card_classes = vec![
        settings_modal::SETTINGS_NAV_ITEM.to_string(),
        settings_modal::SETTINGS_HEADER_CARD.to_string(),
    ];
    if category == super::profile_settings::PROFILES_CATEGORY {
        card_classes
            .push(settings_modal::SETTINGS_NAV_ITEM_ACTIVE.to_string());
    }
    let on_click = {
        let settings_ui = settings_ui.clone();
        move || {
            settings_ui.update(|ui| {
                if ui.category == super::profile_settings::PROFILES_CATEGORY
                    && ui.sub_page.is_none()
                {
                    return;
                }
                ui.category = super::profile_settings::PROFILES_CATEGORY.to_string();
                ui.sub_page = None;
                ui.email_revealed = false;
            });
        }
    };
    let card = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(card_classes)
        .on_click(on_click)
        .build()
        .children([avatar_node, identity]);
    StackProps::horizontal()
        .align(Align::Center)
        .gap(12)
        .full_width(false)
        .padding(Padding::x(16))
        .build()
        .child(card)
}

fn build_search() -> Node {
    div()
        .class(settings_modal::SETTINGS_SEARCH_WRAP)
        .child(modal_search_box(
            &requiem_i18n::t!("settings-modal-search"),
            TEXT_MUTED,
        ))
}

fn build_section(
    label: Option<&str>,
    items: &[NavItem],
    has_divider: bool,
    category: &str,
    settings_ui: &Signal<SettingsUiState>,
) -> Node {
    let mut children = Vec::with_capacity(items.len() + 1);

    if let Some(label) = label {
        let heading_node = HeadingProps::default()
            .level(HeadingLevel::H4)
            .variant(TextVariant::HEADING_SM_MEDIUM)
            .content(requiem_i18n::text(label))
            .color(TEXT_MUTED)
            .build();
        children.push(
            div()
                .class(settings_modal::SETTINGS_NAV_LABEL)
                .child(heading_node),
        );
    }

    for nav_item in items {
        children.push(build_nav_item(nav_item, category, settings_ui));
    }

    let mut section_classes =
        vec![settings_modal::SETTINGS_NAV_SECTION.to_string()];
    if has_divider {
        section_classes
            .push(settings_modal::SETTINGS_NAV_SECTION_DIVIDER.to_string());
    }
    div().classes(section_classes).children(children)
}

fn build_nav_item(
    nav_item: &NavItem,
    category: &str,
    settings_ui: &Signal<SettingsUiState>,
) -> Node {
    let is_active = nav_item.key == category;
    let text_color = if is_active { TEXT_STRONG } else { TEXT_MUTED };

    let icon_node = icon(nav_item.icon_path, text_color)
        .class(settings_modal::SETTINGS_NAV_ITEM_ICON);
    let label_node = TextProps::styled(
        requiem_i18n::text(nav_item.key),
        TextVariant::TEXT_MD_MEDIUM,
    )
    .color(text_color)
    .build()
    .class(settings_modal::SETTINGS_NAV_ITEM_LABEL);
    let content = StackProps::horizontal()
        .align(Align::Center)
        .gap(12)
        .full_width(false)
        .build()
        .children([icon_node, label_node]);

    let mut row_classes =
        vec![settings_modal::SETTINGS_NAV_ITEM.to_string()];
    if is_active {
        row_classes
            .push(settings_modal::SETTINGS_NAV_ITEM_ACTIVE.to_string());
    }
    if nav_item.key == LOG_OUT_LABEL {
        row_classes
            .push(settings_modal::SETTINGS_NAV_ITEM_DESTRUCTIVE.to_string());
    }

    let mut row_children = vec![content];
    if nav_item.badge {
        row_children.push(build_badge());
    }

    let target = nav_item.key;
    let is_action_row = target == LOG_OUT_LABEL;
    let on_click = {
        let settings_ui = settings_ui.clone();
        move || {
            if is_action_row {
                settings_ui.update(|ui| ui.logout_confirm = ui.logout_confirm.set(true));
                return;
            }
            settings_ui.update(|ui| {
                if ui.category == target {
                    return;
                }
                ui.category = target.to_string();

                voice_settings::reset_scroll();
                super::notification_settings::reset_scroll();
                reveal::request(SIDEBAR_REVEAL_KEY);

                ui.email_revealed = false;
            });
        }
    };

    let row = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(row_classes)
        .on_click(on_click)
        .build()
        .children(row_children);

    let item = if is_active && nav_item.key == ACCOUNT_CATEGORY {
        let active = ACCOUNT_SUBNAV
            .iter()
            .position(|&key| key == requiem_i18n::key!("settings-modal-account-info"))
            .unwrap_or(0);
        StackProps::vertical()
            .gap(0)
            .full_width(false)
            .build()
            .children([row, build_subnav(ACCOUNT_SUBNAV, active, None)])
    } else if is_active && nav_item.key == VOICE_CATEGORY {
        let settings_ui = settings_ui.clone();
        let subnav = Node::reactive(move || {
            let settings_ui = settings_ui.clone();
            let on_pick: Rc<dyn Fn(usize)> = Rc::new(move |index| {
                voice_settings::jump_to(index, || settings_ui.update(|_| {}))
            });
            build_subnav(
                &voice_settings::SUBNAV,
                voice_settings::active_subnav(),
                Some(on_pick),
            )
        });
        StackProps::vertical()
            .gap(0)
            .full_width(false)
            .build()
            .children([row, subnav])
    } else if is_active && nav_item.key == NOTIFICATIONS_CATEGORY {
        let settings_ui = settings_ui.clone();
        let subnav = Node::reactive(move || {
            let settings_ui = settings_ui.clone();
            let on_pick: Rc<dyn Fn(usize)> = Rc::new(move |index| {
                super::notification_settings::jump_to(index, || settings_ui.update(|_| {}))
            });
            build_subnav(
                &super::notification_settings::SUBNAV,
                super::notification_settings::active_subnav(),
                Some(on_pick),
            )
        });
        StackProps::vertical()
            .gap(0)
            .full_width(false)
            .build()
            .children([row, subnav])
    } else {
        row
    };

    if is_active {
        item.reveal_when_requested(SIDEBAR_REVEAL_KEY, 12.0)
    } else {
        item
    }
}

const SIDEBAR_REVEAL_KEY: &str = "settings-sidebar-active-item";

fn build_badge() -> Node {
    badge(
        BadgeProps::default()
            .text(requiem_i18n::t!("ui-new-badge"))
            .variant(BadgeVariant::Brand),
    )
}

fn subnav_thumb_offset_class(index: usize) -> &'static str {
    match index {
        0 => settings_modal::SETTINGS_SUBNAV_THUMB_0,
        1 => settings_modal::SETTINGS_SUBNAV_THUMB_1,
        2 => settings_modal::SETTINGS_SUBNAV_THUMB_2,
        3 => settings_modal::SETTINGS_SUBNAV_THUMB_3,
        4 => settings_modal::SETTINGS_SUBNAV_THUMB_4,
        _ => settings_modal::SETTINGS_SUBNAV_THUMB_5,
    }
}

fn build_subnav(keys: &[&'static str], active: usize, on_pick: Option<Rc<dyn Fn(usize)>>) -> Node {
    let mut children = vec![
        div().class(settings_modal::SETTINGS_SUBNAV_TRACK),
        div().classes([
            settings_modal::SETTINGS_SUBNAV_THUMB,
            subnav_thumb_offset_class(active),
        ]),
    ];
    children.extend(keys.iter().enumerate().map(|(index, &key)| {
        let color = if index == active {
            TEXT_STRONG
        } else {
            TEXT_MUTED
        };
        let row_text = TextProps::styled(requiem_i18n::text(key), TextVariant::TEXT_MD_NORMAL)
            .color(color)
            .build()
            .class(settings_modal::SETTINGS_SUBNAV_TEXT);
        match &on_pick {
            Some(on_pick) => {
                let on_pick = on_pick.clone();
                ButtonProps::default()
                    .variant(ButtonVariant::Bare)
                    .extra_classes(vec![
                        settings_modal::SETTINGS_SUBNAV_ROW.to_string(),
                    ])
                    .on_click(move || on_pick(index))
                    .build()
                    .child(row_text)
            }
            None => div()
                .class(settings_modal::SETTINGS_SUBNAV_ROW)
                .child(row_text),
        }
    }));

    div()
        .class(settings_modal::SETTINGS_SUBNAV)
        .children(children)
}

#[cfg(test)]
mod wrap_tests {
    use crate::shell;
    use requiem_core::ui::emoji_picker::EmojiPickerSignals;
    use requiem_core::ui::emoji_picker::EmojiPickerState;
    use requiem_core::ui::emoji_picker::PickerTab;
    use requiem_ui::font_system::TextFonts;
    use requiem_ui::font_system;
    use requiem_ui::layout;
    use requiem_ui::paint;
    use requiem_ui::style::cascade;
    use requiem_ui::theme;
    use requiem_ui::widget::primitives;
    use skia_safe::font_style::Weight;
    use skia_safe::surfaces;
    use std::collections::BTreeSet;
    use std::rc::Rc;
    use tokio::runtime::Runtime;

    use requiem_core::models::Id;
    use requiem_ui::signal::signal;
    use requiem_ui::style::ComputedStyle;
    use requiem_ui::widget::WidgetTree;
    use requiem_ui::widget::anim::ToggleState;
    use requiem_ui::widget::primitives::BoxWidget;
    use taffy::{AvailableSpace, Size};

    use super::*;

    fn fake_user() -> User {
        User {
            id: Id::new(999),
            username: "testing".to_string(),
            discriminator: "0".to_string(),
            global_name: None,
            avatar: None,
            email: Some("testing@example.com".to_string()),
            phone: None,
            verified: Some(false),
            mfa_enabled: Some(false),
            bot: false,
            premium_type: None,
            primary_guild: None,
        }
    }

    #[test]
    fn unverified_email_notice_paints_the_lines_layout_reserved() {
        const BODY: &str = "Please check your email and follow the";

        let runtime = Runtime::new().expect("tokio runtime");
        let fonts = Fonts {
            typeface: theme::make_typeface(Weight::MEDIUM),
            text_fonts: TextFonts::new(16 * 1024 * 1024),
        };
        font_system::install_ambient(fonts.text_fonts.clone());
        let backend = BackendConfig {
            rest_base_url: String::new(),
            gateway_url: String::new(),
            cdn_base_url: String::new(),
        };
        let images = ImageCache::new(runtime.handle().clone(), wreq::Client::new(), || {});
        let queries = QueryClient::new(runtime.handle().clone(), || {});
        let state = AppState::new();
        state.users.current.set(Some(fake_user()));
        let settings_ui = signal(SettingsUiState::default());

        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let root = tree.root;
        tree.nodes[root].classes.push("root".to_string());
        let emoji_picker = EmojiPickerState {
            open: signal(false),
            tab: signal(PickerTab::Emoji),
            signals: EmojiPickerSignals {
                search: signal(String::new()),
                scroll_offset: signal(0.0),
                category_jump: signal(None),
                hovered: signal(None),
                collapsed: signal(Vec::new()),
            },
            gif: ui::gif_picker::GifPickerState::inert(),
            on_insert: Rc::new(|_| {}),
            on_insert_at: Rc::new(|_, _| {}),
        };
        let account_popout = account_popout::AccountPopoutState::default();
        let modal = build(
            &state,
            &fonts,
            &images,
            &backend,
            ToggleState::new(true),
            Rc::new(|| {}),
            &settings_ui,
            &queries,
            None,
            &account_popout,
            &emoji_picker,
        );
        tree.mount(root, modal);
        cascade::apply_stylesheet(&mut tree, &shell::load_theme());

        let mut surface =
            surfaces::raster_n32_premul((2000, 1600)).expect("raster surface");

        let mut seen = BTreeSet::new();
        let mut problems = Vec::new();
        let mut check = |tree: &mut WidgetTree, width: f32, height: f32| {
            layout::compute_layout(
                tree,
                Size {
                    width: AvailableSpace::Definite(width),
                    height: AvailableSpace::Definite(height),
                },
            );
            paint::paint(tree, surface.canvas());
            let (lines, box_height) = primitives::painted_lines_of(BODY)
                .expect("the notice body was painted");
            let reserved_lines = (box_height / (14.0 * 1.2)).round() as usize;
            seen.insert((reserved_lines, lines));
            if lines != reserved_lines {
                problems.push((width, height, reserved_lines, lines));
            }
        };

        for step in (0..=100).chain((0..=100).rev()) {
            check(&mut tree, 1100.0 + step as f32 * 0.5, 900.0);
        }
        for step in (0..=100).chain((0..=100).rev()) {
            check(&mut tree, 1100.0, 600.0 + step as f32 * 0.5);
        }
        assert_eq!(
            seen.len(),
            1,
            "line counts (reserved, painted) seen while resizing: {seen:?}"
        );
        assert!(
            problems.is_empty(),
            "(width, height, reserved lines, painted lines): {problems:?}"
        );
    }
}

#[cfg(test)]
mod composer_fuzz_tests {

    use requiem_core::ui::emoji_picker as ui_emoji_picker;
    use requiem_core::ui::twemoji;
    use requiem_ui::event::MouseButton;
    use requiem_ui::event::{NamedKey, UiEvent};
    use requiem_ui::font_system;
    use requiem_ui::glyph_cache::GlyphCache;
    use requiem_ui::layout;
    use requiem_ui::paint;
    use requiem_ui::style::ComputedStyle;
    use requiem_ui::widget::primitives::{BoxWidget, MarkdownFeatures, TextInputWidget};
    use requiem_ui::widget::{AbsoluteRect, WidgetTree};
    use skia_safe::surfaces;
    use std::cell::RefCell;
    use std::panic;
    use std::rc::Rc;
    use taffy::{AvailableSpace, Size};

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
    }

    #[test]
    fn random_sessions_never_panic() {
        let font_system = Rc::new(RefCell::new(font_system::make_font_system()));
        let glyph_cache = Rc::new(GlyphCache::new(16 * 1024 * 1024));
        let widget = TextInputWidget::new(
            String::new(),
            "Message #general",
            16.0,
            skia_safe::Color::WHITE,
            skia_safe::Color::GRAY,
            font_system,
            glyph_cache,
            true,
        )
        .with_wrap()
        .with_markdown(MarkdownFeatures::ALL)
        .with_emoji_provider(twemoji::tile)
        .with_dynamic_autocomplete(ui_emoji_picker::emoji_lookup(None))
        .with_line_height(22.0)
        .with_vertical_padding(17.0)
        .with_insets(12.0, 12.0)
        .with_popup_outset(48.0, 8.0);
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let mut style = ComputedStyle::default();
        style.layout.size = Size {
            width: taffy::Dimension::length(500.0),
            height: taffy::Dimension::auto(),
        };
        style.layout.margin.top = taffy::LengthPercentageAuto::length(400.0);
        style.layout.margin.left = taffy::LengthPercentageAuto::length(60.0);
        let id = tree.add_child(tree.root, "div", Box::new(widget), style);
        tree.focused = Some(id);
        let mut surface = surfaces::raster_n32_premul((700, 700)).expect("surface");
        let chars = [
            ":",
            "s",
            "m",
            "i",
            "l",
            "e",
            " ",
            "\u{1F604}",
            "a",
            "\n",
            "*",
            "_",
            "`",
            "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}",
            "\u{1F3FB}",
        ];
        for seed in 1..=150u64 {
            let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
            let mut log: Vec<String> = Vec::new();
            for _ in 0..80 {
                let roll = rng.next() % 100;
                let event = if roll >= 90 {
                    let (x, y) = (
                        40.0 + (rng.next() % 600) as f32,
                        150.0 + (rng.next() % 350) as f32,
                    );
                    let mouse = if roll.is_multiple_of(2) {
                        UiEvent::MouseMoved { x, y }
                    } else {
                        UiEvent::MouseDown {
                            x,
                            y,
                            button: MouseButton::Left,
                            shift: false,
                        }
                    };
                    log.push(format!("{mouse:?}"));
                    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                        paint::dispatch_event(&mut tree, &mouse);
                        layout::compute_layout(
                            &mut tree,
                            Size {
                                width: AvailableSpace::Definite(700.0),
                                height: AvailableSpace::Definite(700.0),
                            },
                        );
                        paint::paint(&mut tree, surface.canvas());
                    }));
                    if let Err(panic) = result {
                        let msg = panic
                            .downcast_ref::<String>()
                            .cloned()
                            .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
                            .unwrap_or_default();
                        panic!("seed {seed} panicked: {msg}\nsteps: {log:?}");
                    }
                    continue;
                } else if roll < 45 {
                    let c = chars[(rng.next() % chars.len() as u64) as usize];
                    log.push(format!("type {c:?}"));
                    UiEvent::TextInput {
                        text: c.to_string(),
                    }
                } else {
                    let keys = [
                        NamedKey::Backspace,
                        NamedKey::Backspace,
                        NamedKey::Delete,
                        NamedKey::ArrowLeft,
                        NamedKey::ArrowRight,
                        NamedKey::ArrowUp,
                        NamedKey::ArrowDown,
                        NamedKey::Home,
                        NamedKey::End,
                        NamedKey::DeleteWordBack,
                        NamedKey::DeleteWordForward,
                        NamedKey::WordLeft,
                        NamedKey::Undo,
                        NamedKey::Redo,
                        NamedKey::Enter,
                        NamedKey::Escape,
                        NamedKey::SelectAll,
                    ];
                    let key = keys[(rng.next() % keys.len() as u64) as usize];
                    log.push(format!("{key:?}"));
                    UiEvent::KeyDown {
                        key,
                        shift: rng.next().is_multiple_of(4),
                    }
                };
                let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    paint::dispatch_key_event(&mut tree, &event);
                    layout::compute_layout(
                        &mut tree,
                        Size {
                            width: AvailableSpace::Definite(700.0),
                            height: AvailableSpace::Definite(700.0),
                        },
                    );
                    paint::paint(&mut tree, surface.canvas());
                }));
                if let Err(panic) = result {
                    let msg = panic
                        .downcast_ref::<String>()
                        .cloned()
                        .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
                        .unwrap_or_default();
                    panic!("seed {seed} panicked: {msg}\nsteps: {log:?}");
                }
            }
        }
        let _ = AbsoluteRect::default();
    }
}
