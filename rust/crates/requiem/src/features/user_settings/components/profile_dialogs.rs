use crate::shell::classes::profile_settings as c;
use requiem_core::rest::endpoints::users;
use requiem_core::ui;
use requiem_ui::components::{Align, StackProps};
use requiem_ui::components::{ButtonVariant, ModalSize};
use requiem_ui::components::{HeadingLevel, TextVariant};
use std::rc::Rc;

use requiem_ui::components::{
    AvatarStatusSize, ButtonProps, HeadingProps, ModalProps, TextProps, avatar_with_status, div,
    image,
};
use requiem_ui::node::Node;
use requiem_ui::signal::Signal;

use requiem_core::models::Id;
use requiem_core::query::QueryClient;
use requiem_core::rest::endpoints::users::CatalogItem;
use requiem_core::rest::{HttpClient, RestError};
use requiem_core::state::AppState;
use requiem_core::ui::{
    CatalogKind, Fonts, ImageTarget, PendingImage, PendingUpload, ProfileDialog,
    ProfileDialogState, SettingsUiState, TEXT_DEFAULT, TEXT_SECONDARY, TEXT_STRONG,
};

const CATALOG_QUERY_KEY: &str = "collectibles-categories";

pub fn build(
    dialog_state: &ProfileDialogState,
    dialog_signal: &Signal<ProfileDialogState>,
    settings_ui: &Signal<SettingsUiState>,
    state: &AppState,
    fonts: &Fonts,
    queries: &QueryClient,
    http: Option<&HttpClient>,
) -> Option<Node> {
    let dialog = dialog_state.dialog.clone()?;
    let open = dialog_state.open;
    let draft = settings_ui.peek().profile_draft;
    let close: Rc<dyn Fn()> = {
        let dialog_signal = dialog_signal.clone();
        Rc::new(move || dialog_signal.update(|d| d.open = d.open.set(false)))
    };
    let cancel = |label: &str| {
        let close = close.clone();
        ButtonProps::default()
            .variant(ButtonVariant::Secondary)
            .label(label.to_string())
            .on_click(move || close())
            .build()
    };

    let (title, body, footer) = match dialog {
        ProfileDialog::Image { target, upload } => {
            let body = image_preview(target, &upload, state, fonts);
            let apply = {
                let (close, draft) = (close.clone(), draft.clone());
                ButtonProps::default()
                    .variant(ButtonVariant::Primary)
                    .label(requiem_i18n::t!("profile-dialogs-apply").to_string())
                    .on_click(move || {
                        let upload = upload.clone();
                        draft.update(|d| match target {
                            ImageTarget::Avatar => d.avatar = PendingImage::Set(upload),
                            ImageTarget::Banner => d.banner = PendingImage::Set(upload),
                        });
                        close();
                    })
                    .build()
            };
            (
                if target == ImageTarget::Avatar {
                    requiem_i18n::t!("profile-dialogs-edit-image")
                } else {
                    requiem_i18n::t!("profile-dialogs-edit-banner")
                },
                body,
                vec![cancel(&requiem_i18n::t!("profile-dialogs-cancel")), apply],
            )
        }
        ProfileDialog::Catalog(kind) => (
            kind.title(),
            catalog_body(kind, queries, http),
            vec![cancel(&requiem_i18n::t!("profile-dialogs-close"))],
        ),
    };

    let heading = HeadingProps::default()
        .level(HeadingLevel::H1)
        .variant(TextVariant::HEADING_LG_SEMIBOLD)
        .content(title.to_string())
        .color(TEXT_STRONG)
        .build();
    let dismiss = close.clone();
    let node = ModalProps::default()
        .size(ModalSize::Small)
        .title(heading)
        .dismissable(true)
        .on_dismiss(move || dismiss())
        .open(open)
        .body(body)
        .footer(footer)
        .build();
    Some(node)
}

fn image_preview(
    target: ImageTarget,
    upload: &PendingUpload,
    state: &AppState,
    fonts: &Fonts,
) -> Node {
    let note = TextProps::styled(
        requiem_i18n::t!("profile-dialogs-the-largest-centered-area-of-the"),
        TextVariant::TEXT_SM_NORMAL,
    )
    .color(TEXT_SECONDARY)
    .build();
    let preview = match target {
        ImageTarget::Avatar => {
            let user = state.users.current.peek();
            let props = match &user {
                Some(u) => ui::user_avatar_props(u.id, &u.discriminator, fonts),
                None => ui::user_avatar_props(Id::new(1), "0", fonts),
            }
            .image(upload.preview.clone());
            avatar_with_status(props, AvatarStatusSize::Px120, None)
        }
        ImageTarget::Banner => {
            image(Some(upload.preview.clone()), 360.0, 144.0).class(c::DIALOG_BANNER)
        }
    };
    StackProps::vertical()
        .align(Align::Center)
        .gap(16)
        .full_width(false)
        .build()
        .children([preview, note])
}

fn catalog_body(kind: CatalogKind, queries: &QueryClient, http: Option<&HttpClient>) -> Node {
    let items: Option<Vec<CatalogItem>> = match (kind.collectible_type(), http) {
        (Some(wanted), Some(http)) => {
            let http = http.clone();
            queries
                .query_default::<Vec<CatalogItem>, _, RestError>(
                    CATALOG_QUERY_KEY,
                    move || async move {
                        users::list_catalog(&http).await
                    },
                )
                .data
                .map(|all| {
                    all.iter()
                        .filter(|item| item.kind == wanted)
                        .cloned()
                        .collect()
                })
        }
        _ => None,
    };
    match items {
        Some(items) if !items.is_empty() => {
            let tiles = items.iter().map(|item| {
                div().class(c::CATALOG_TILE).child(
                    TextProps::styled(&item.name, TextVariant::TEXT_SM_NORMAL)
                        .color(TEXT_DEFAULT)
                        .build(),
                )
            });
            StackProps::horizontal()
                .wrap(true)
                .full_width(false)
                .build()
                .children(tiles)
        }
        _ => StackProps::vertical()
            .align(Align::Center)
            .gap(16)
            .full_width(false)
            .build()
            .children([
                TextProps::styled(
                    requiem_i18n::t!("profile-dialogs-nothing-available-here-yet"),
                    TextVariant::TEXT_MD_NORMAL,
                )
                .color(TEXT_STRONG)
                .build(),
                TextProps::styled(
                    requiem_i18n::t!("profile-dialogs-this-server-doesn-t-offer-a"),
                    TextVariant::TEXT_SM_NORMAL,
                )
                .color(TEXT_SECONDARY)
                .build(),
            ]),
    }
}
