use crate::ui::attachments::FileKind;
use crate::ui::classes::message_embed as card;
use crate::ui::classes::message_list as rows;
use crate::ui::classes::pending_uploads;
use crate::ui::classes::pending_uploads as css;
use crate::ui::twemoji;
use crate::ui;
use crate::utils::format;
use crate::utils::time;
use requiem_ui::colors::{
    TEXT_ANCHOR, TEXT_DEFAULT, TEXT_MUTED, TEXT_SECONDARY, TEXT_STRONG, TEXT_SUBTLE,
};
use requiem_ui::components::ButtonVariant;
use requiem_ui::components::TextVariant;

use requiem_ui::components::{
    ButtonProps, MessageTextProps, ProgressBarProps, TextProps, div, icon, image, message_text,
};
use requiem_ui::node::Node;
use skia_safe::Color;
use skia_safe::font_style::Weight;

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::models::id::marker::{ChannelMarker, MessageMarker, UserMarker};
use crate::models::{Id, User};
use crate::ui::Fonts;
use crate::ui::attachments::{AttachmentsState, FailureNotice, PendingUpload, card_icon};
use requiem_ui::icons::{CHECKMARK_SMALL_BOLD, EYE, X_SMALL};

const GROUP_INTERVAL_MS: i64 = 7 * 60 * 1000;

pub type LastMessage = Option<(Id<UserMarker>, i64)>;

#[derive(Clone)]
pub struct PendingContext {
    pub user: Option<User>,
    pub last: LastMessage,
    pub images: ImageCache,
    pub backend: BackendConfig,
}

pub fn build_bottom(
    attachments: &AttachmentsState,
    channel: Id<ChannelMarker>,
    ctx: PendingContext,
    fonts: &Fonts,
) -> Node {
    let state = attachments.clone();
    let fonts = fonts.clone();
    Node::reactive(move || {
        let notices: Vec<FailureNotice> = state.notices.with(|list| {
            list.iter()
                .filter(|n| n.channel == channel && n.after.is_none())
                .cloned()
                .collect()
        });
        let uploads: Vec<PendingUpload> = state.uploads.with(|list| {
            list.iter()
                .filter(|u| u.channel == channel)
                .cloned()
                .collect()
        });
        let mut column = div().class(pending_uploads::PENDING_BOTTOM);
        for notice in &notices {
            column = column.child(notice_row(&state, notice));
        }

        let mut previous = if notices.is_empty() { ctx.last } else { None };
        for upload in &uploads {
            let user_id = ctx.user.as_ref().map(|u| u.id);
            let starts_group = match (previous, user_id) {
                (Some((author, at)), Some(me)) => {
                    author != me || (upload.created_ms - at).abs() > GROUP_INTERVAL_MS
                }
                _ => true,
            };
            column = column.child(pending_row(&state, upload, starts_group, &ctx, &fonts));
            previous = user_id.map(|me| (me, upload.created_ms));
        }
        column
    })
}

pub fn notices_after(
    attachments: &AttachmentsState,
    channel: Id<ChannelMarker>,
    message: Id<MessageMarker>,
) -> Vec<Node> {
    attachments
        .notices
        .with(|list| {
            list.iter()
                .filter(|n| n.channel == channel && n.after == Some(message))
                .cloned()
                .collect::<Vec<_>>()
        })
        .iter()
        .map(|n| notice_row(attachments, n))
        .collect()
}

fn pending_row(
    state: &AttachmentsState,
    upload: &PendingUpload,
    starts_group: bool,
    ctx: &PendingContext,
    fonts: &Fonts,
) -> Node {
    let mut row = div().class(rows::MESSAGE_ROW);
    let mut body = div().class(rows::MESSAGE_BODY);
    match ctx.user.as_ref().filter(|_| starts_group) {
        Some(user) => {
            row = row.class(rows::MESSAGE_ROW_START);
            let mut avatar = ui::user_avatar_props(user.id, &user.discriminator, fonts);
            if let Some(image) = user.avatar.as_deref().and_then(|hash| {
                ctx.images
                    .get_or_decode_image(&ctx.backend.avatar_url(user.id, hash))
            }) {
                avatar = avatar.image(image);
            }
            row = row.child(
                div()
                    .class(rows::MESSAGE_AVATAR_BTN)
                    .child(avatar.build().class(rows::MESSAGE_AVATAR)),
            );
            let name = TextProps::styled(user.username.clone(), TextVariant::TEXT_MD_MEDIUM)
                .line_height(22.0_f32)
                .color(TEXT_STRONG)
                .build();
            let time = chrono::DateTime::from_timestamp_millis(upload.created_ms)
                .map(|t| time::clock(&t.with_timezone(&chrono::Local)))
                .unwrap_or_default();
            body = body.child(
                div().class(rows::MESSAGE_HEADER).child(name).child(
                    TextProps::styled(time, TextVariant::TEXT_XS_MEDIUM)
                        .line_height(22.0_f32)
                        .color(TEXT_SUBTLE)
                        .build(),
                ),
            );
        }
        None => row = row.child(div().class(rows::MESSAGE_AVATAR_SPACER)),
    }
    if !upload.content.is_empty() {
        let text = message_text(MessageTextProps {
            content: upload.content.clone(),
            size: 16.0,
            color: TEXT_DEFAULT,

            emoji_provider: Some(twemoji::tile),
            line_height: Some(22.0),
            ..Default::default()
        });
        body = body.child(div().class(css::PENDING_CONTENT).child(text));
    }
    body = body.child(upload_card(state, upload));
    row.child(body)
}

fn upload_card(state: &AttachmentsState, upload: &PendingUpload) -> Node {
    let (title, kind) = match upload.files.as_slice() {
        [only] => (only.filename.clone(), only.kind),
        files => (
            requiem_i18n::t!("pending-uploads-uploading-files", count = files.len()),
            FileKind::Unknown,
        ),
    };
    let mut name_row = div().class(css::UPLOAD_NAME_ROW).child(
        TextProps::styled(title, TextVariant::TEXT_MD_NORMAL)
            .line_height(22.0_f32)
            .color(TEXT_STRONG)
            .build()
            .class(css::UPLOAD_NAME),
    );
    if upload.sent > 0 {
        let size: u64 = upload.files.iter().map(|f| f.size).sum();
        name_row = name_row.child(
            TextProps::styled(
                format!("— {}", format::file_size(size)),
                TextVariant::TEXT_MD_NORMAL,
            )
            .line_height(22.0_f32)
            .color(TEXT_MUTED)
            .build()
            .class(css::UPLOAD_SIZE),
        );
    }
    let fraction = if upload.total == 0 {
        1.0
    } else {
        (upload.sent as f32 / upload.total as f32).clamp(0.0, 1.0)
    };
    let track = ProgressBarProps::new(upload.nonce, fraction).build();
    let inner = div()
        .class(css::UPLOAD_INNER)
        .child(name_row)
        .child(div().class(css::UPLOAD_PROGRESS).child(track));
    let cancel = {
        let (backend, nonce) = (state.backend.clone(), upload.nonce);
        ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![css::UPLOAD_CANCEL.to_string()])
            .on_click(move || (backend.cancel)(nonce))
            .build()
            .child(icon(X_SMALL, TEXT_SECONDARY).class(css::UPLOAD_CANCEL_ICON))
    };
    div()
        .class(card::FILE_CARD)
        .class(card::FILE_CARD_FIRST)
        .child(image(card_icon(kind), 30.0, 40.0).class(card::FILE_CARD_ICON))
        .child(inner)
        .child(cancel)
}

thread_local! {
    static CLYDE_AVATAR: Option<skia_safe::Image> = ui::asset_image("image/clyde_avatar");
}

fn notice_row(state: &AttachmentsState, notice: &FailureNotice) -> Node {
    let avatar = CLYDE_AVATAR.with(Clone::clone);
    let avatar = div().class(rows::MESSAGE_AVATAR_BTN).child(
        image(avatar, 40.0, 40.0)
            .class(rows::MESSAGE_AVATAR)
            .class(css::NOTICE_AVATAR),
    );
    let name = TextProps::styled(
        requiem_i18n::t!("pending-uploads-clyde").to_string(),
        TextVariant::TEXT_MD_MEDIUM,
    )
    .line_height(22.0_f32)
    .color(TEXT_STRONG)
    .build();
    let app_tag = div()
        .class(css::APP_TAG)
        .child(icon(CHECKMARK_SMALL_BOLD, Color::WHITE).class(css::APP_TAG_ICON))
        .child(
            TextProps::new(requiem_i18n::t!("common-app-tag"), 12.8_f32)
                .line_height(15.0_f32)
                .weight(Weight::SEMI_BOLD)
                .color(Color::WHITE)
                .build(),
        );
    let header = div()
        .class(rows::MESSAGE_HEADER)
        .child(name)
        .child(app_tag)
        .child(
            TextProps::styled(notice.time.clone(), TextVariant::TEXT_XS_MEDIUM)
                .line_height(22.0_f32)
                .color(TEXT_SUBTLE)
                .build(),
        );
    let text = message_text(MessageTextProps {
        content: requiem_i18n::t!("upload-failed"),
        size: 16.0,
        color: TEXT_DEFAULT,
        line_height: Some(22.0),
        ..Default::default()
    });
    let dismiss = {
        let (state, id) = (state.clone(), notice.id);
        let label = TextProps::styled(
            requiem_i18n::t!("dismiss-message"),
            TextVariant::TEXT_XS_NORMAL,
        )
        .color(TEXT_ANCHOR)
        .build()
        .class(css::NOTICE_DISMISS);
        ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![css::NOTICE_DISMISS_BTN.to_string()])
            .on_click(move || state.dismiss_notice(id))
            .build()
            .child(label)
    };
    let footer = div()
        .class(css::NOTICE_FOOTER)
        .child(icon(EYE, TEXT_MUTED).class(css::NOTICE_FOOTER_ICON))
        .child(
            TextProps::styled(
                requiem_i18n::t!("pending-uploads-only-you-can-see-this").to_string(),
                TextVariant::TEXT_XS_NORMAL,
            )
            .color(TEXT_MUTED)
            .build(),
        )
        .child(dismiss);
    let body = div()
        .class(rows::MESSAGE_BODY)
        .child(header)
        .child(text)
        .child(footer);
    div()
        .class(rows::MESSAGE_ROW)
        .class(rows::MESSAGE_ROW_START)
        .class(css::NOTICE_ROW)
        .child(avatar)
        .child(body)
}
