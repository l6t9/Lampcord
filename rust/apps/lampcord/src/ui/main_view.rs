use gpui::{
    AnyElement, Bounds, ClickEvent, Context, ElementInputHandler, IntoElement, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ObjectFit, ParentElement, PathPromptOptions,
    Pixels, SharedString, Styled, TextAlign, TextRun, canvas, div, fill, img, list, point,
    prelude::*, px,
};

use crate::backend;
use crate::ui::debug_stats::{self, ImageKind};
use crate::ui::icons;
use crate::ui::shell::AppPage;
use crate::ui::theme::M3Theme;
use crate::ui::widgets::{
    avatar_circle, channel_icon, format_message_timestamp, format_time_hhmm, guild_icon_element,
    message_date_key, should_group_messages,
};
use crate::{BackendCommand, Root};

fn is_image_attachment(att: &backend::AttachmentView) -> bool {
    if let Some(ct) = att.content_type.as_deref() {
        if ct.starts_with("image/") {
            return true;
        }
    }
    let name = att.filename.to_ascii_lowercase();
    name.ends_with(".png")
        || name.ends_with(".jpg")
        || name.ends_with(".jpeg")
        || name.ends_with(".gif")
        || name.ends_with(".webp")
        || name.ends_with(".bmp")
        || name.ends_with(".avif")
}

fn format_file_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    let b = bytes as f64;
    if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

fn render_reply_preview(preview: &backend::ReplyPreviewView, theme: &M3Theme) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .mb(px(2.))
        .child(
            div()
                .w(px(28.))
                .h(px(8.))
                .mt(px(8.))
                .border_l_2()
                .border_t_2()
                .border_color(theme.outline_variant)
                .rounded_tl(px(4.)),
        )
        .child(
            div()
                .text_xs()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(theme.on_surface_variant)
                .child(SharedString::from(preview.author.clone())),
        )
        .child(
            div()
                .flex_1()
                .text_xs()
                .text_color(theme.on_surface_variant)
                .overflow_hidden()
                .child(SharedString::from(preview.content.clone())),
        )
}

fn render_attachments(
    atts: &[backend::AttachmentView],
    msg_id: u64,
    theme: &M3Theme,
) -> Vec<gpui::AnyElement> {
    let mut out = Vec::new();
    let mut idx = 0usize;
    let mut inline_images = 0usize;
    for att in atts.iter().take(6) {
        idx += 1;
        if is_image_attachment(att) {
            if inline_images < 2 {
                inline_images += 1;
                debug_stats::record_render_image_with_kind(Some(&att.url), ImageKind::Attachment);
                out.push(
                    div()
                        .id(SharedString::from(format!("att-wrap-{msg_id}-{idx}")))
                        .mt(px(4.))
                        .max_w(px(360.))
                        .rounded(px(8.))
                        .overflow_hidden()
                        .child(
                            img(SharedString::from(att.url.clone()))
                                .id(SharedString::from(format!("att-img-{msg_id}-{idx}")))
                                .max_w(px(360.))
                                .max_h(px(240.))
                                .object_fit(ObjectFit::Contain),
                        )
                        .into_any_element(),
                );
            } else {
                out.push(
                    div()
                        .id(SharedString::from(format!("att-file-{msg_id}-{idx}")))
                        .mt(px(4.))
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .px(px(12.))
                        .py(px(10.))
                        .rounded(px(8.))
                        .bg(theme.surface_container_high)
                        .border_1()
                        .border_color(theme.outline_variant)
                        .max_w(px(420.))
                        .child(
                            div()
                                .w(px(32.))
                                .h(px(32.))
                                .rounded(px(6.))
                                .bg(theme.surface_container_highest)
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_sm()
                                .child("🖼"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .flex_1()
                                .overflow_hidden()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(theme.primary)
                                        .overflow_hidden()
                                        .child(SharedString::from(att.filename.clone())),
                                )
                                .child(div().text_xs().text_color(theme.on_surface_variant).child(
                                    SharedString::from(format!(
                                        "Image • {}",
                                        format_file_size(att.size)
                                    )),
                                )),
                        )
                        .into_any_element(),
                );
            }
        } else {
            out.push(
                div()
                    .id(SharedString::from(format!("att-file-{msg_id}-{idx}")))
                    .mt(px(4.))
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .px(px(12.))
                    .py(px(10.))
                    .rounded(px(8.))
                    .bg(theme.surface_container_high)
                    .border_1()
                    .border_color(theme.outline_variant)
                    .max_w(px(420.))
                    .child(
                        div()
                            .w(px(32.))
                            .h(px(32.))
                            .rounded(px(6.))
                            .bg(theme.surface_container_highest)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_sm()
                            .child("📄"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .overflow_hidden()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.primary)
                                    .overflow_hidden()
                                    .child(SharedString::from(att.filename.clone())),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.on_surface_variant)
                                    .child(SharedString::from(format_file_size(att.size))),
                            ),
                    )
                    .into_any_element(),
            );
        }
    }
    if atts.len() > 6 {
        out.push(
            div()
                .mt(px(2.))
                .text_xs()
                .text_color(theme.on_surface_variant)
                .child(SharedString::from(format!(
                    "+{} more attachments",
                    atts.len() - 6
                )))
                .into_any_element(),
        );
    }
    out
}

fn render_embeds(
    embeds: &[backend::EmbedView],
    msg_id: u64,
    theme: &M3Theme,
) -> Vec<gpui::AnyElement> {
    let mut out = Vec::new();
    let mut idx = 0usize;
    let mut rendered_media = 0usize;
    for e in embeds.iter().take(2) {
        if e.title.is_none()
            && e.description.is_none()
            && e.image_url.is_none()
            && e.thumbnail_url.is_none()
            && e.author_name.is_none()
            && e.footer_text.is_none()
        {
            continue;
        }
        idx += 1;
        let accent = e
            .color
            .map(|c| {
                let r = ((c >> 16) & 0xFF) as u8;
                let g = ((c >> 8) & 0xFF) as u8;
                let b = (c & 0xFF) as u8;
                gpui::rgb(((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
            })
            .unwrap_or(theme.primary);

        let mut inner = div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .flex_1()
            .overflow_hidden();
        if let Some(author) = e.author_name.as_ref().filter(|s| !s.is_empty()) {
            inner = inner.child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.on_surface)
                    .child(SharedString::from(author.clone())),
            );
        }
        if let Some(title) = e.title.as_ref().filter(|s| !s.is_empty()) {
            inner = inner.child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.primary)
                    .child(SharedString::from(title.clone())),
            );
        }
        if let Some(desc) = e.description.as_ref().filter(|s| !s.is_empty()) {
            let mut d = desc.clone();
            if d.chars().count() > 240 {
                d = d.chars().take(240).collect::<String>() + "…";
            }
            inner = inner.child(
                div()
                    .text_xs()
                    .text_color(theme.on_surface_variant)
                    .child(SharedString::from(d)),
            );
        }

        let mut show_main_image = false;
        let mut show_thumb = false;
        if rendered_media < 1 {
            if e.image_url.as_ref().is_some_and(|s| !s.is_empty()) {
                show_main_image = true;
                rendered_media += 1;
            } else if e.thumbnail_url.as_ref().is_some_and(|s| !s.is_empty()) {
                show_thumb = true;
                rendered_media += 1;
            }
        }

        if show_main_image {
            if let Some(img_url) = e.image_url.as_ref().filter(|s| !s.is_empty()) {
                debug_stats::record_render_image_with_kind(Some(img_url), ImageKind::Embed);
                inner = inner.child(
                    div().mt(px(4.)).max_w(px(320.)).child(
                        img(SharedString::from(img_url.clone()))
                            .id(SharedString::from(format!("embed-img-{msg_id}-{idx}")))
                            .max_w(px(320.))
                            .max_h(px(200.))
                            .rounded(px(4.))
                            .object_fit(ObjectFit::Contain),
                    ),
                );
            }
        }
        if let Some(footer) = e.footer_text.as_ref().filter(|s| !s.is_empty()) {
            inner = inner.child(
                div()
                    .mt(px(2.))
                    .text_xs()
                    .text_color(theme.on_surface_variant)
                    .child(SharedString::from(footer.clone())),
            );
        }

        let mut content_row = div()
            .flex()
            .flex_row()
            .gap(px(12.))
            .items_start()
            .child(inner);
        if show_thumb {
            if let Some(thumb) = e.thumbnail_url.as_ref().filter(|s| !s.is_empty()) {
                debug_stats::record_render_image_with_kind(Some(thumb), ImageKind::Embed);
                content_row = content_row.child(
                    img(SharedString::from(thumb.clone()))
                        .id(SharedString::from(format!("embed-thumb-{msg_id}-{idx}")))
                        .w(px(64.))
                        .h(px(64.))
                        .rounded(px(4.))
                        .object_fit(ObjectFit::Cover),
                );
            }
        }

        out.push(
            div()
                .id(SharedString::from(format!("embed-{msg_id}-{idx}")))
                .mt(px(4.))
                .flex()
                .flex_row()
                .max_w(px(520.))
                .rounded(px(4.))
                .bg(theme.surface_container_high)
                .overflow_hidden()
                .child(div().w(px(4.)).flex_shrink_0().bg(accent))
                .child(div().flex_1().p(px(10.)).child(content_row))
                .into_any_element(),
        );
    }
    if embeds.len() > 2 {
        out.push(
            div()
                .mt(px(2.))
                .text_xs()
                .text_color(theme.on_surface_variant)
                .child(SharedString::from(format!(
                    "+{} more embeds",
                    embeds.len() - 2
                )))
                .into_any_element(),
        );
    }
    out
}

fn render_message_extras(m: &backend::MessageView, theme: &M3Theme) -> Vec<gpui::AnyElement> {
    let msg_id = u64::from(m.id);
    let mut extras: Vec<gpui::AnyElement> = Vec::new();
    extras.extend(render_attachments(&m.attachments, msg_id, theme));
    extras.extend(render_embeds(&m.embeds, msg_id, theme));

    if !m.sticker_names.is_empty() {
        let label = if m.sticker_names.len() == 1 {
            format!("Sticker: {}", m.sticker_names[0])
        } else {
            format!("Stickers: {}", m.sticker_names.join(", "))
        };
        extras.push(
            div()
                .mt(px(4.))
                .px(px(8.))
                .py(px(4.))
                .rounded(px(6.))
                .bg(theme.surface_container_high)
                .text_xs()
                .text_color(theme.on_surface_variant)
                .child(SharedString::from(label))
                .into_any_element(),
        );
    }

    if m.reaction_count > 0 {
        let label = if m.reaction_count == 1 {
            "1 reaction".to_owned()
        } else {
            format!("{} reactions", m.reaction_count)
        };
        extras.push(
            div()
                .id(SharedString::from(format!("reactions-{msg_id}")))
                .mt(px(4.))
                .flex()
                .child(
                    div()
                        .px(px(8.))
                        .py(px(3.))
                        .rounded(px(8.))
                        .bg(theme.surface_container_high)
                        .border_1()
                        .border_color(theme.outline_variant)
                        .text_xs()
                        .text_color(theme.on_surface_variant)
                        .child(SharedString::from(label)),
                )
                .into_any_element(),
        );
    }

    extras
}

impl Root {
    pub(crate) fn render_device_challenge(
        &mut self,
        cx: &mut Context<Self>,
        challenge: &backend::DeviceSessionChallenge,
        theme: &M3Theme,
    ) -> impl IntoElement {
        let (title, hint, is_password, allow_backup, message) = match challenge {
            backend::DeviceSessionChallenge::Password { message, .. } => (
                "Confirm with your password",
                "Enter your password",
                true,
                false,
                message.clone(),
            ),
            backend::DeviceSessionChallenge::Totp {
                allow_backup,
                message,
                ..
            } => (
                "Verify with your authenticator app",
                "Enter 6-digit authentication code",
                false,
                *allow_backup,
                message.clone(),
            ),
            backend::DeviceSessionChallenge::BackupCode { message, .. } => (
                "Verify with a backup code",
                "Enter backup code",
                false,
                false,
                message.clone(),
            ),
        };

        let display = if self.device_challenge_input.is_empty() {
            hint.to_owned()
        } else if is_password {
            "•".repeat(self.device_challenge_input.chars().count().min(48))
        } else {
            self.device_challenge_input.clone()
        };
        let display_color = if self.device_challenge_input.is_empty() {
            theme.on_surface_variant
        } else {
            theme.on_surface
        };

        let mut card = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .p(px(14.))
            .rounded(px(16.))
            .bg(theme.surface_container)
            .border_1()
            .border_color(theme.outline_variant)
            .child(div().text_sm().text_color(theme.on_surface).child(title));

        if let Some(msg) = message {
            card = card.child(
                div()
                    .text_xs()
                    .text_color(theme.on_surface_variant)
                    .child(SharedString::from(msg)),
            );
        }

        card = card.child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .px(px(12.))
                .py(px(10.))
                .rounded(px(12.))
                .bg(theme.surface_container_high)
                .border_1()
                .border_color(theme.primary)
                .child(div().text_xs().text_color(theme.primary).child(hint))
                .child(
                    div()
                        .text_sm()
                        .text_color(display_color)
                        .child(SharedString::from(display)),
                ),
        );

        let mut actions = div().flex().flex_row().gap(px(8.)).child(
            div()
                .id("device-challenge-submit")
                .px(px(12.))
                .py(px(8.))
                .rounded(px(16.))
                .bg(theme.primary)
                .text_color(theme.on_primary)
                .hover(|s| s.cursor_pointer())
                .child("Confirm")
                .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                    let input = this.device_challenge_input.trim().to_owned();
                    if input.is_empty() {
                        return;
                    }
                    let snapshot = this.backend.snapshot();
                    match snapshot.device_sessions.challenge {
                        Some(backend::DeviceSessionChallenge::Password { .. }) => {
                            this.backend
                                .send(BackendCommand::SubmitDeviceSessionPassword {
                                    password: input,
                                });
                        }
                        Some(backend::DeviceSessionChallenge::Totp { .. })
                        | Some(backend::DeviceSessionChallenge::BackupCode { .. }) => {
                            this.backend
                                .send(BackendCommand::SubmitDeviceSessionCode { code: input });
                        }
                        None => {}
                    }
                    this.device_challenge_input.clear();
                    cx.notify();
                })),
        );

        if allow_backup {
            actions = actions.child(
                div()
                    .id("device-challenge-backup")
                    .px(px(12.))
                    .py(px(8.))
                    .rounded(px(16.))
                    .bg(theme.surface_container_high)
                    .text_color(theme.on_surface)
                    .hover(|s| s.cursor_pointer())
                    .child("Use backup code")
                    .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                        this.device_challenge_input.clear();
                        this.backend
                            .send(BackendCommand::UseDeviceSessionBackupCode);
                        cx.notify();
                    })),
            );
        }

        actions = actions.child(
            div()
                .id("device-challenge-cancel")
                .px(px(12.))
                .py(px(8.))
                .rounded(px(16.))
                .bg(theme.surface_container_high)
                .text_color(theme.on_surface)
                .hover(|s| s.cursor_pointer())
                .child("Cancel")
                .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                    this.device_challenge_input.clear();
                    this.backend
                        .send(BackendCommand::CancelDeviceSessionChallenge);
                    cx.notify();
                })),
        );

        card.child(actions)
    }

    pub(crate) fn render_devices_panel(
        &mut self,
        cx: &mut Context<Self>,
        snapshot: &backend::Snapshot,
    ) -> impl IntoElement {
        let theme = self.theme.clone();
        let ds = &snapshot.device_sessions;

        let mut list = div().flex().flex_col().gap(px(8.)).w_full();
        if ds.loading {
            list = list.child(
                div()
                    .text_sm()
                    .text_color(theme.on_surface_variant)
                    .child("Loading sessions..."),
            );
        }
        if let Some(err) = ds.error.as_ref() {
            list = list.child(
                div()
                    .text_sm()
                    .text_color(theme.error)
                    .child(SharedString::from(err.clone())),
            );
        }
        if let Some(status) = ds.status.as_ref() {
            list = list.child(
                div()
                    .text_sm()
                    .text_color(theme.on_surface_variant)
                    .child(SharedString::from(status.clone())),
            );
        }
        if !ds.loading && ds.sessions.is_empty() && ds.error.is_none() {
            list = list.child(
                div()
                    .text_sm()
                    .text_color(theme.on_surface_variant)
                    .child("No sessions loaded."),
            );
        }

        for session in ds.sessions.iter().take(30) {
            let id_hash = session.id_hash.clone();
            let is_current = session.current;
            let mut meta = Vec::new();
            if let Some(loc) = session.location.as_ref() {
                meta.push(loc.clone());
            }
            if let Some(last) = session.last_used.as_ref() {
                meta.push(format!("Last used: {last}"));
            }
            meta.push(format!("Type: {}", session.icon));
            if is_current {
                meta.push("Current session".to_owned());
            }
            let meta_text = meta.join(" • ");
            let mut row = div()
                .id(SharedString::from(format!("device-{}", session.id_hash)))
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .gap(px(12.))
                .w_full()
                .p(px(12.))
                .rounded(px(16.))
                .bg(if is_current {
                    theme.secondary_container
                } else {
                    theme.surface_container
                })
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .flex_1()
                        .child(
                            div()
                                .text_sm()
                                .text_color(if is_current {
                                    theme.on_secondary_container
                                } else {
                                    theme.on_surface
                                })
                                .child(SharedString::from(session.name.clone())),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(if is_current {
                                    theme.on_secondary_container
                                } else {
                                    theme.on_surface_variant
                                })
                                .child(SharedString::from(meta_text)),
                        ),
                );

            if !is_current {
                row = row.child(
                    div()
                        .id(SharedString::from(format!(
                            "device-logout-{}",
                            session.id_hash
                        )))
                        .px(px(12.))
                        .py(px(8.))
                        .rounded(px(16.))
                        .bg(theme.error)
                        .text_color(theme.on_error)
                        .hover(|s| s.cursor_pointer())
                        .child("Log out")
                        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                            this.device_challenge_input.clear();
                            this.backend.send(BackendCommand::LogoutDeviceSessions {
                                id_hashes: vec![id_hash.clone()],
                            });
                            cx.notify();
                        })),
                );
            } else {
                row = row.child(
                    div()
                        .px(px(12.))
                        .py(px(8.))
                        .rounded(px(16.))
                        .bg(theme.surface_container_high)
                        .text_color(theme.on_surface_variant)
                        .child("This device"),
                );
            }

            list = list.child(row);
        }

        let mut panel = div()
            .absolute()
            .top(px(72.))
            .left(px(16.))
            .w(px(560.))
            .max_h(px(620.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .p(px(16.))
            .rounded(px(24.))
            .bg(theme.surface_container_low)
            .border_1()
            .border_color(theme.outline_variant)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_lg()
                            .text_color(theme.on_surface)
                            .child("Device sessions"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(8.))
                            .child(
                                div()
                                    .id("devices-refresh")
                                    .px(px(12.))
                                    .py(px(8.))
                                    .rounded(px(16.))
                                    .bg(theme.secondary_container)
                                    .text_color(theme.on_secondary_container)
                                    .hover(|s| s.cursor_pointer())
                                    .child("Refresh")
                                    .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                                        this.backend.send(BackendCommand::LoadDeviceSessions);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                div()
                                    .id("devices-close")
                                    .px(px(12.))
                                    .py(px(8.))
                                    .rounded(px(16.))
                                    .bg(theme.surface_container_high)
                                    .text_color(theme.on_surface)
                                    .hover(|s| s.cursor_pointer())
                                    .child("Close")
                                    .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                                        this.show_devices_panel = false;
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.on_surface_variant)
                    .child("Manage devices logged into your account. Some logouts require password or MFA verification."),
            );

        if let Some(challenge) = ds.challenge.as_ref() {
            panel = panel.child(self.render_device_challenge(cx, challenge, &theme));
        }

        panel.child(
            div()
                .flex()
                .flex_col()
                .gap(px(8.))
                .id("devices-list")
                .child(list),
        )
    }

    pub(crate) fn render_main(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
        snapshot: &backend::Snapshot,
    ) -> impl IntoElement {
        debug_stats::begin_render_frame();
        let theme = self.theme;

        if snapshot.selected_guild_id.is_some() && snapshot.selected_channel_id.is_none() {
            if let Some(first) = snapshot
                .channels
                .iter()
                .find(|c| c.kind != 4 && c.kind != 2 && c.kind != 13)
                .or_else(|| snapshot.channels.first())
            {
                let cid = first.id;
                self.backend.send(BackendCommand::SelectChannel {
                    channel_id: Some(cid),
                });
                self.backend.send(BackendCommand::FetchMessages {
                    channel_id: cid,
                    before: None,
                });
                self.requested_channel = Some(cid);
            }
        }
        if snapshot.selected_guild_id.is_none() && snapshot.selected_channel_id.is_none() {
            if let Some(first) = snapshot.channels.first() {
                let cid = first.id;
                self.backend.send(BackendCommand::SelectChannel {
                    channel_id: Some(cid),
                });
                self.backend.send(BackendCommand::FetchMessages {
                    channel_id: cid,
                    before: None,
                });
            }
        }
        let snapshot = self.backend.snapshot();

        let mut guild_rail = div()
            .id("guild-rail")
            .flex()
            .flex_col()
            .items_center()
            .w(px(72.))
            .h_full()
            .pt(px(12.))
            .pb(px(12.))
            .gap(px(8.))
            .overflow_y_scroll();

        let home_selected = snapshot.selected_guild_id.is_none();
        {
            let mut home_wrap = div()
                .id("guild-home-wrap")
                .relative()
                .w(px(72.))
                .h(px(48.))
                .flex()
                .items_center()
                .justify_center()
                .hover(|s| s.cursor_pointer())
                .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                    this.backend
                        .send(BackendCommand::SelectGuild { guild_id: None });
                    this.backend
                        .send(BackendCommand::SelectChannel { channel_id: None });
                    this.requested_channel = None;
                    cx.notify();
                }));

            let pill_h = if home_selected { 40.0 } else { 0.0 };
            if pill_h > 0.0 {
                home_wrap = home_wrap.child(
                    div()
                        .absolute()
                        .left(px(0.))
                        .w(px(4.))
                        .h(px(pill_h))
                        .rounded_r(px(4.))
                        .bg(theme.on_surface),
                );
            }

            let home_bg = if home_selected {
                theme.primary
            } else {
                theme.surface_container_high
            };
            let home_fg = if home_selected {
                theme.on_primary
            } else {
                theme.on_surface_variant
            };
            let home_radius = if home_selected { 16.0 } else { 24.0 };
            home_wrap = home_wrap.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(48.))
                    .h(px(48.))
                    .rounded(px(home_radius))
                    .bg(home_bg)
                    .text_color(home_fg)
                    .child(icons::rounded::Chat.element(24.0, home_fg)),
            );
            guild_rail = guild_rail.child(home_wrap);
        }

        guild_rail = guild_rail.child(
            div()
                .w(px(32.))
                .h(px(2.))
                .rounded(px(1.))
                .bg(theme.outline_variant),
        );

        {
            use lampcord_model::snowflake::Snowflake;
            use std::collections::{HashMap, HashSet};

            #[derive(Clone)]
            enum RailEntry {
                Guild(Snowflake),
                Folder {
                    id: i64,
                    guild_ids: Vec<Snowflake>,
                    name: Option<String>,
                    color: Option<u32>,
                },
            }

            let guild_map: HashMap<Snowflake, &backend::GuildView> =
                snapshot.guilds.iter().map(|g| (g.id, g)).collect();
            let mut seen: HashSet<Snowflake> = HashSet::new();
            let mut rail_entries: Vec<RailEntry> = Vec::new();

            for folder in snapshot.guild_folders.iter() {
                if folder.guild_ids.is_empty() {
                    continue;
                }
                if let Some(fid) = folder.id {
                    let mut ids = Vec::new();
                    for gid in &folder.guild_ids {
                        if guild_map.contains_key(gid) && !seen.contains(gid) {
                            seen.insert(*gid);
                            ids.push(*gid);
                        }
                    }
                    if !ids.is_empty() {
                        rail_entries.push(RailEntry::Folder {
                            id: fid,
                            guild_ids: ids,
                            name: folder.name.clone(),
                            color: folder.color,
                        });
                    }
                } else {
                    for gid in &folder.guild_ids {
                        if guild_map.contains_key(gid) && !seen.contains(gid) {
                            seen.insert(*gid);
                            rail_entries.push(RailEntry::Guild(*gid));
                        }
                    }
                }
            }
            for g in snapshot.guilds.iter() {
                if !seen.contains(&g.id) {
                    rail_entries.push(RailEntry::Guild(g.id));
                }
            }

            let render_guild_item = |g: &backend::GuildView,
                                     selected_guild_id: Option<Snowflake>,
                                     theme: &M3Theme,
                                     cx: &mut Context<Self>| {
                let gid = g.id;
                let selected = selected_guild_id == Some(g.id);
                let icon = guild_icon_element(g, selected, theme);
                let mut wrap = div()
                    .id(SharedString::from(format!("guild-wrap-{gid}")))
                    .relative()
                    .w(px(72.))
                    .h(px(48.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .hover(|s| s.cursor_pointer())
                    .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                        this.select_guild(gid, cx);
                    }));

                let pill_h = if selected {
                    40.0
                } else if g.unread || g.mention_count > 0 {
                    8.0
                } else {
                    0.0
                };
                if pill_h > 0.0 {
                    wrap = wrap.child(
                        div()
                            .absolute()
                            .left(px(0.))
                            .w(px(4.))
                            .h(px(pill_h))
                            .rounded_r(px(4.))
                            .bg(theme.on_surface),
                    );
                }

                let mut icon_holder = div().relative().child(icon);
                if g.mention_count > 0 {
                    let label = if g.mention_count > 99 {
                        "99+".to_owned()
                    } else {
                        g.mention_count.to_string()
                    };
                    icon_holder = icon_holder.child(
                        div()
                            .absolute()
                            .right(px(-4.))
                            .bottom(px(-4.))
                            .min_w(px(18.))
                            .h(px(18.))
                            .px(px(4.))
                            .rounded_full()
                            .bg(theme.error)
                            .text_color(theme.on_error)
                            .text_xs()
                            .font_weight(gpui::FontWeight::BOLD)
                            .flex()
                            .items_center()
                            .justify_center()
                            .border_2()
                            .border_color(theme.surface_container_lowest)
                            .child(SharedString::from(label)),
                    );
                }

                wrap = wrap.child(icon_holder);
                wrap
            };

            for entry in rail_entries {
                match entry {
                    RailEntry::Guild(gid) => {
                        if let Some(g) = guild_map.get(&gid).copied() {
                            guild_rail = guild_rail.child(render_guild_item(
                                g,
                                snapshot.selected_guild_id,
                                &theme,
                                cx,
                            ));
                        }
                    }
                    RailEntry::Folder {
                        id: folder_id,
                        guild_ids,
                        name: _name,
                        color,
                    } => {
                        let expanded = self.expanded_guild_folders.contains(&folder_id);
                        let mut folder_guilds: Vec<&backend::GuildView> = Vec::new();
                        for gid in &guild_ids {
                            if let Some(g) = guild_map.get(gid).copied() {
                                folder_guilds.push(g);
                            }
                        }
                        if folder_guilds.is_empty() {
                            continue;
                        }

                        let has_selected = folder_guilds
                            .iter()
                            .any(|g| snapshot.selected_guild_id == Some(g.id));
                        let total_mentions: u32 =
                            folder_guilds.iter().map(|g| g.mention_count).sum();
                        let any_unread = folder_guilds.iter().any(|g| g.unread);

                        let folder_color = color
                            .filter(|c| *c != 0)
                            .map(|c| gpui::rgb(c))
                            .unwrap_or(theme.primary);

                        if !expanded {
                            let mut mini_grid = div()
                                .flex()
                                .flex_wrap()
                                .items_center()
                                .justify_center()
                                .gap(px(2.))
                                .w(px(48.))
                                .h(px(48.))
                                .p(px(4.))
                                .rounded(px(16.))
                                .bg(folder_color.opacity(0.22));
                            for i in 0..4 {
                                if let Some(g) = folder_guilds.get(i) {
                                    let mini = div()
                                        .w(px(18.))
                                        .h(px(18.))
                                        .rounded(px(6.))
                                        .overflow_hidden()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .bg(theme.surface_container_high)
                                        .text_color(theme.on_surface_variant)
                                        .text_xs();
                                    let mini = if let Some(url) =
                                        g.icon_url.as_deref().filter(|u| !u.is_empty())
                                    {
                                        debug_stats::record_render_image_with_kind(
                                            Some(url),
                                            ImageKind::GuildIcon,
                                        );
                                        mini.child(
                                            img(SharedString::from(url.to_owned()))
                                                .size_full()
                                                .rounded(px(6.))
                                                .overflow_hidden()
                                                .object_fit(ObjectFit::Cover),
                                        )
                                    } else {
                                        let ch = g
                                            .name
                                            .chars()
                                            .find(|c| c.is_alphanumeric())
                                            .map(|c| c.to_uppercase().to_string())
                                            .unwrap_or_else(|| "#".to_owned());
                                        mini.child(SharedString::from(ch))
                                    };
                                    mini_grid = mini_grid.child(mini);
                                } else {
                                    mini_grid = mini_grid.child(
                                        div()
                                            .w(px(18.))
                                            .h(px(18.))
                                            .rounded(px(6.))
                                            .bg(theme.surface_container_low),
                                    );
                                }
                            }

                            let mut folder_wrap = div()
                                .id(SharedString::from(format!("guild-folder-{folder_id}")))
                                .relative()
                                .w(px(72.))
                                .h(px(48.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .hover(|s| s.cursor_pointer())
                                .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                                    if this.expanded_guild_folders.contains(&folder_id) {
                                        this.expanded_guild_folders.remove(&folder_id);
                                    } else {
                                        this.expanded_guild_folders.insert(folder_id);
                                    }
                                    cx.notify();
                                }));

                            let pill_h = if has_selected {
                                40.0
                            } else if any_unread || total_mentions > 0 {
                                8.0
                            } else {
                                0.0
                            };
                            if pill_h > 0.0 {
                                folder_wrap = folder_wrap.child(
                                    div()
                                        .absolute()
                                        .left(px(0.))
                                        .w(px(4.))
                                        .h(px(pill_h))
                                        .rounded_r(px(4.))
                                        .bg(theme.on_surface),
                                );
                            }

                            let mut icon_holder = div().relative().child(mini_grid);
                            if total_mentions > 0 {
                                let label = if total_mentions > 99 {
                                    "99+".to_owned()
                                } else {
                                    total_mentions.to_string()
                                };
                                icon_holder = icon_holder.child(
                                    div()
                                        .absolute()
                                        .right(px(-4.))
                                        .bottom(px(-4.))
                                        .min_w(px(18.))
                                        .h(px(18.))
                                        .px(px(4.))
                                        .rounded_full()
                                        .bg(theme.error)
                                        .text_color(theme.on_error)
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .border_2()
                                        .border_color(theme.surface_container_lowest)
                                        .child(SharedString::from(label)),
                                );
                            }

                            folder_wrap = folder_wrap.child(icon_holder);
                            guild_rail = guild_rail.child(folder_wrap);
                        } else {
                            let header_icon = if has_selected {
                                icons::rounded_filled::FolderOpen
                            } else {
                                icons::rounded::FolderOpen
                            };
                            let mut header = div()
                                .id(SharedString::from(format!("guild-folder-open-{folder_id}")))
                                .relative()
                                .w(px(72.))
                                .h(px(48.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .hover(|s| s.cursor_pointer())
                                .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                                    this.expanded_guild_folders.remove(&folder_id);
                                    cx.notify();
                                }));

                            if has_selected || any_unread || total_mentions > 0 {
                                let pill_h = if has_selected { 40.0 } else { 8.0 };
                                header = header.child(
                                    div()
                                        .absolute()
                                        .left(px(0.))
                                        .w(px(4.))
                                        .h(px(pill_h))
                                        .rounded_r(px(4.))
                                        .bg(theme.on_surface),
                                );
                            }

                            let mut header_icon_holder = div().relative().child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(48.))
                                    .h(px(48.))
                                    .rounded(px(16.))
                                    .bg(folder_color.opacity(0.22))
                                    .text_color(folder_color)
                                    .child(header_icon.element(24.0, folder_color)),
                            );
                            if total_mentions > 0 {
                                let label = if total_mentions > 99 {
                                    "99+".to_owned()
                                } else {
                                    total_mentions.to_string()
                                };
                                header_icon_holder = header_icon_holder.child(
                                    div()
                                        .absolute()
                                        .right(px(-4.))
                                        .bottom(px(-4.))
                                        .min_w(px(18.))
                                        .h(px(18.))
                                        .px(px(4.))
                                        .rounded_full()
                                        .bg(theme.error)
                                        .text_color(theme.on_error)
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .border_2()
                                        .border_color(theme.surface_container_lowest)
                                        .child(SharedString::from(label)),
                                );
                            }

                            header = header.child(header_icon_holder);
                            guild_rail = guild_rail.child(header);

                            let mut folder_body = div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(px(8.))
                                .py(px(4.))
                                .w(px(72.))
                                .bg(folder_color.opacity(0.08));
                            for g in folder_guilds {
                                folder_body = folder_body.child(render_guild_item(
                                    g,
                                    snapshot.selected_guild_id,
                                    &theme,
                                    cx,
                                ));
                            }
                            guild_rail = guild_rail.child(folder_body);
                        }
                    }
                }
            }
        }

        {
            guild_rail = guild_rail.child(
                div()
                    .id("guild-rail-add")
                    .w(px(72.))
                    .h(px(48.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(48.))
                            .h(px(48.))
                            .rounded_full()
                            .bg(theme.surface_variant)
                            .text_color(theme.primary)
                            .child(icons::rounded::Add.element(28.0, theme.primary)),
                    ),
            );
        }

        let sidebar_title = if let Some(gid) = snapshot.selected_guild_id {
            snapshot
                .guilds
                .iter()
                .find(|g| g.id == gid)
                .map(|g| g.name.clone())
                .unwrap_or_else(|| "Server".to_owned())
        } else {
            "Direct Messages".to_owned()
        };
        let is_dm_view = snapshot.selected_guild_id.is_none();

        let mut sidebar = div()
            .id("sidebar-channel-panel")
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.))
            .h_full()
            .rounded(px(16.))
            .overflow_hidden()
            .bg(theme.surface);

        if is_dm_view {
            sidebar = sidebar.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(48.))
                    .px(px(16.))
                    .border_b_1()
                    .border_color(theme.surface_container_lowest)
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .text_base()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.on_surface)
                            .child("Direct Messages"),
                    )
                    .child(
                        div()
                            .id("sidebar-switch-account")
                            .w(px(28.))
                            .h(px(28.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(6.))
                            .text_color(theme.on_surface_variant)
                            .hover(|s| s.cursor_pointer().bg(theme.surface_container_high))
                            .child(icons::rounded::Add.element(18.0, theme.on_surface_variant))
                            .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                                this.show_login = true;
                                this.login_explicitly_opened = true;
                                this.login.local_error = None;
                                cx.notify();
                            })),
                    ),
            );
            sidebar = sidebar.child(
                div().px(px(10.)).pt(px(10.)).pb(px(6.)).child(
                    div()
                        .id("dm-search-bar")
                        .h(px(28.))
                        .px(px(8.))
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .rounded(px(4.))
                        .bg(theme.surface_container_lowest)
                        .text_xs()
                        .text_color(theme.on_surface_variant)
                        .child(icons::rounded::Search.element(14.0, theme.on_surface_variant))
                        .child("Find or start a conversation"),
                ),
            );
        } else {
            sidebar = sidebar.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(48.))
                    .px(px(16.))
                    .border_b_1()
                    .border_color(theme.surface_container_lowest)
                    .hover(|s| s.bg(theme.surface_container).cursor_pointer())
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .text_sm()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme.on_surface)
                            .child(SharedString::from(sidebar_title.clone())),
                    )
                    .child(icons::rounded::ExpandMore.element(20.0, theme.on_surface_variant)),
            );
        }

        let mut channel_list = div()
            .id("channel-list")
            .flex()
            .flex_col()
            .gap(px(0.))
            .px(px(0.))
            .py(px(8.))
            .flex_1()
            .overflow_y_scroll();

        let make_badge = |count: u32, theme: &M3Theme| {
            let label = if count > 99 {
                "99+".to_owned()
            } else {
                count.to_string()
            };
            div()
                .min_w(px(16.))
                .h(px(16.))
                .px(px(4.))
                .rounded_full()
                .bg(theme.error)
                .text_color(theme.on_error)
                .text_xs()
                .font_weight(gpui::FontWeight::BOLD)
                .flex()
                .items_center()
                .justify_center()
                .child(SharedString::from(label))
        };

        if is_dm_view {
            channel_list = channel_list.child(
                div().px(px(8.)).py(px(1.)).child(
                    div()
                        .id("dm-nav-friends")
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .h(px(42.))
                        .px(px(8.))
                        .rounded(px(4.))
                        .hover(|s| s.bg(theme.surface_container).cursor_pointer())
                        .child(
                            div()
                                .w(px(32.))
                                .h(px(32.))
                                .rounded_full()
                                .bg(theme.surface_container_high)
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(theme.on_surface_variant)
                                .child(
                                    icons::rounded::Person.element(18.0, theme.on_surface_variant),
                                ),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(theme.on_surface)
                                .child("Friends"),
                        ),
                ),
            );
            channel_list = channel_list.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(16.))
                    .pt(px(12.))
                    .pb(px(4.))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.on_surface_variant)
                            .child("DIRECT MESSAGES"),
                    )
                    .child(
                        div()
                            .w(px(18.))
                            .h(px(18.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(theme.on_surface_variant)
                            .child(icons::rounded::Add.element(16.0, theme.on_surface_variant)),
                    ),
            );
        }

        if snapshot.channels.is_empty() {
            channel_list = channel_list.child(
                div()
                    .px(px(16.))
                    .py(px(16.))
                    .text_sm()
                    .text_color(theme.on_surface_variant)
                    .child(if is_dm_view {
                        "No conversations yet"
                    } else {
                        "No channels available"
                    }),
            );
        } else if is_dm_view {
            for c in snapshot.channels.iter().take(150) {
                let cid = c.id;
                let selected = snapshot.selected_channel_id == Some(c.id);
                let unread = c.unread && !selected;
                let name_color = if selected || c.unread {
                    theme.on_surface
                } else {
                    theme.on_surface_variant
                };
                let fallback = c
                    .name
                    .chars()
                    .find(|ch| ch.is_alphanumeric())
                    .map(|ch| ch.to_string())
                    .unwrap_or_else(|| "?".to_owned());

                let mut item = div()
                    .relative()
                    .w_full()
                    .h(px(42.))
                    .flex()
                    .items_center()
                    .py(px(1.));

                if unread {
                    item = item.child(
                        div()
                            .absolute()
                            .left(px(0.))
                            .w(px(4.))
                            .h(px(12.))
                            .rounded_r(px(2.))
                            .bg(theme.on_surface),
                    );
                }

                let mut row = div()
                    .id(SharedString::from(format!("dm-{cid}")))
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .w_full()
                    .h(px(42.))
                    .mx(px(8.))
                    .px(px(8.))
                    .rounded(px(4.))
                    .bg(if selected {
                        theme.surface_container_highest
                    } else {
                        gpui::rgba(0x00000000)
                    })
                    .hover(|s| {
                        s.bg(if selected {
                            theme.surface_container_highest
                        } else {
                            theme.surface_container
                        })
                        .cursor_pointer()
                    })
                    .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                        this.select_channel(cid, cx);
                    }))
                    .child(avatar_circle(
                        SharedString::from(format!("dm-av-{cid}")),
                        c.icon_url.as_deref(),
                        &fallback,
                        32.0,
                        &theme,
                    ))
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .text_sm()
                            .font_weight(if c.unread || selected {
                                gpui::FontWeight::SEMIBOLD
                            } else {
                                gpui::FontWeight::MEDIUM
                            })
                            .text_color(name_color)
                            .child(SharedString::from(c.name.clone())),
                    );

                if c.mention_count > 0 {
                    row = row.child(make_badge(c.mention_count, &theme));
                }
                item = item.child(row);
                channel_list = channel_list.child(item);
            }
        } else {
            fn is_voice_like(kind: u8) -> bool {
                matches!(kind, 2 | 13)
            }

            let mut render_channel_row = |mut channel_list: gpui::Stateful<gpui::Div>,
                                          c: &backend::ChannelView,
                                          indented: bool,
                                          theme: &M3Theme,
                                          cx: &mut Context<Self>|
             -> gpui::Stateful<gpui::Div> {
                let cid = c.id;
                let selected = snapshot.selected_channel_id == Some(c.id);
                let item_h = 36.0_f32;
                let unread = c.unread && !selected;
                let name_color = if selected || c.unread {
                    theme.on_surface
                } else {
                    theme.on_surface_variant
                };
                let icon_color = if selected || c.unread {
                    theme.on_surface_variant
                } else {
                    theme.outline
                };
                let icon = channel_icon(c.kind);

                let mut item = div()
                    .relative()
                    .w_full()
                    .h(px(item_h))
                    .flex()
                    .items_center()
                    .py(px(1.));

                if unread {
                    item = item.child(
                        div()
                            .absolute()
                            .left(px(0.))
                            .w(px(4.))
                            .h(px(12.))
                            .rounded_r(px(2.))
                            .bg(theme.on_surface),
                    );
                }

                let mut row = div()
                    .id(SharedString::from(format!("ch-{cid}")))
                    .flex()
                    .items_center()
                    .w_full()
                    .h(px(item_h))
                    .mx(px(8.))
                    .px(px(10.))
                    .rounded(px(4.))
                    .bg(if selected {
                        theme.surface_container_highest
                    } else {
                        gpui::rgba(0x00000000)
                    })
                    .hover(|s| {
                        s.bg(if selected {
                            theme.surface_container_highest
                        } else {
                            theme.surface_container
                        })
                        .cursor_pointer()
                    })
                    .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                        this.select_channel(cid, cx);
                    }))
                    .child(
                        div()
                            .w(px(24.))
                            .h(px(24.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(icon_color)
                            .child(icon.element(20.0, icon_color)),
                    )
                    .child(div().w(px(12.)))
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .text_base()
                            .font_weight(if c.unread && !selected {
                                gpui::FontWeight::BOLD
                            } else {
                                gpui::FontWeight::MEDIUM
                            })
                            .text_color(name_color)
                            .child(SharedString::from(c.name.clone())),
                    );

                if c.mention_count > 0 {
                    row = row.child(make_badge(c.mention_count, &theme));
                }

                item = item.child(row);
                channel_list.child(item)
            };

            let mut categories: Vec<&backend::ChannelView> =
                snapshot.channels.iter().filter(|c| c.kind == 4).collect();
            categories.sort_by(|a, b| a.position.cmp(&b.position).then_with(|| a.id.cmp(&b.id)));

            let mut root_channels: Vec<&backend::ChannelView> = snapshot
                .channels
                .iter()
                .filter(|c| c.kind != 4 && c.parent_id.is_none())
                .collect();
            root_channels.sort_by(|a, b| {
                is_voice_like(a.kind)
                    .cmp(&is_voice_like(b.kind))
                    .then_with(|| a.position.cmp(&b.position))
                    .then_with(|| a.id.cmp(&b.id))
            });

            let mut first_block = true;
            for c in root_channels {
                channel_list = render_channel_row(channel_list, c, false, &theme, cx);
                first_block = false;
            }

            for cat in categories {
                let cat_id = cat.id;
                let collapsed = self.collapsed_categories.contains(&cat_id);
                let mut children: Vec<&backend::ChannelView> = snapshot
                    .channels
                    .iter()
                    .filter(|c| c.kind != 4 && c.parent_id == Some(cat.id))
                    .collect();
                children.sort_by(|a, b| {
                    is_voice_like(a.kind)
                        .cmp(&is_voice_like(b.kind))
                        .then_with(|| a.position.cmp(&b.position))
                        .then_with(|| a.id.cmp(&b.id))
                });

                let mut category_header = div()
                    .id(SharedString::from(format!("cat-{cat_id}")))
                    .flex()
                    .flex_col()
                    .w_full()
                    .pt(if first_block { px(8.) } else { px(12.) });
                first_block = false;

                let header_color = theme.on_surface_variant;
                let chevron = if collapsed {
                    icons::rounded::ChevronRight
                } else {
                    icons::rounded::KeyboardArrowDown
                };
                let header_row = div()
                    .id(SharedString::from(format!("cat-header-{cat_id}")))
                    .flex()
                    .items_center()
                    .w_full()
                    .px(px(16.))
                    .py(px(2.))
                    .rounded(px(4.))
                    .hover(|s| s.cursor_pointer())
                    .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                        if this.collapsed_categories.contains(&cat_id) {
                            this.collapsed_categories.remove(&cat_id);
                        } else {
                            this.collapsed_categories.insert(cat_id);
                        }
                        cx.notify();
                    }))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(header_color)
                            .child(SharedString::from(cat.name.clone())),
                    )
                    .child(div().w(px(4.)))
                    .child(chevron.element(16.0, header_color));

                category_header = category_header.child(header_row);
                if !collapsed {
                    category_header = category_header.child(div().h(px(4.)));
                    channel_list = channel_list.child(category_header);
                    for c in children {
                        channel_list = render_channel_row(channel_list, c, true, &theme, cx);
                    }
                } else {
                    channel_list = channel_list.child(category_header);
                }
            }

            for c in snapshot.channels.iter() {
                if c.kind == 4 {
                    continue;
                }
                let is_root = c.parent_id.is_none();
                let in_known_category = c.parent_id.is_some_and(|pid| {
                    snapshot.channels.iter().any(|x| x.kind == 4 && x.id == pid)
                });
                if is_root || in_known_category {
                    continue;
                }
                channel_list =
                    render_channel_row(channel_list, c, c.parent_id.is_some(), &theme, cx);
            }
        }
        sidebar = sidebar.child(channel_list);

        let channel_panel = sidebar;

        let user_label = snapshot
            .current_user
            .clone()
            .unwrap_or_else(|| "Signed in".to_owned());
        let user_avatar_url = snapshot.current_user_avatar_url.clone();
        let account_panel = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .h(px(56.))
            .px(px(8.))
            .rounded(px(16.))
            .bg(theme.surface_container_lowest)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .flex_1()
                    .overflow_hidden()
                    .px(px(4.))
                    .py(px(4.))
                    .rounded(px(4.))
                    .hover(|s| s.bg(theme.surface_container_high).cursor_pointer())
                    .child(avatar_circle(
                        "sidebar-user-avatar",
                        user_avatar_url.as_deref(),
                        &user_label,
                        32.0,
                        &theme,
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .overflow_hidden()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.on_surface)
                                    .overflow_hidden()
                                    .child(SharedString::from(user_label.clone())),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.on_surface_variant)
                                    .child("Online"),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(2.))
                    .child(
                        div()
                            .id("user-btn-mic")
                            .w(px(32.))
                            .h(px(32.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(4.))
                            .text_color(theme.on_surface_variant)
                            .hover(|s| s.bg(theme.surface_container_high).cursor_pointer())
                            .child(icons::rounded::Mic.element(20.0, theme.on_surface_variant)),
                    )
                    .child(
                        div()
                            .id("user-btn-deafen")
                            .w(px(32.))
                            .h(px(32.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(4.))
                            .text_color(theme.on_surface_variant)
                            .hover(|s| s.bg(theme.surface_container_high).cursor_pointer())
                            .child(
                                icons::rounded::Headphones.element(20.0, theme.on_surface_variant),
                            ),
                    )
                    .child(
                        div()
                            .id("user-btn-settings")
                            .w(px(32.))
                            .h(px(32.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(4.))
                            .text_color(theme.on_surface_variant)
                            .hover(|s| s.bg(theme.surface_container_high).cursor_pointer())
                            .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                                this.page = AppPage::Settings;
                                cx.notify();
                            }))
                            .child(
                                icons::rounded::Settings.element(20.0, theme.on_surface_variant),
                            ),
                    ),
            );

        let selected_channel = snapshot
            .selected_channel_id
            .and_then(|cid| snapshot.channels.iter().find(|c| c.id == cid));
        let selected_channel_name = selected_channel.map(|c| c.name.clone()).unwrap_or_else(|| {
            if snapshot.selected_guild_id.is_some() {
                "Select a channel".to_owned()
            } else {
                "Select a conversation".to_owned()
            }
        });
        let channel_kind = selected_channel.map(|c| c.kind).unwrap_or(0);
        let is_group_dm = is_dm_view && selected_channel.map(|c| c.kind == 3).unwrap_or(false);

        #[derive(Clone)]
        enum MsgRow {
            LoadOlder {
                channel_id: u64,
                before: Option<u64>,
            },
            TopSpacer,
            DateSeparator(String),
            UnreadMarker,
            GroupedMessage(backend::MessageView),
            FullMessage(backend::MessageView),
            BottomSpacer,
        }

        let message_list: AnyElement = if snapshot.selected_channel_id.is_none() {
            div()
                .id("message-list")
                .flex()
                .flex_col()
                .flex_1()
                .px(px(0.))
                .py(px(0.))
                .overflow_y_scroll()
                .child(
                    div()
                        .flex()
                        .flex_1()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_xl()
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(theme.on_surface)
                                .child(if snapshot.selected_guild_id.is_some() {
                                    "Select a channel"
                                } else {
                                    "Select a conversation"
                                }),
                        )
                        .child(div().text_sm().text_color(theme.on_surface_variant).child(
                            if snapshot.selected_guild_id.is_some() {
                                "Choose a channel from the list to start chatting."
                            } else {
                                "Choose a DM or group from the sidebar."
                            },
                        )),
                )
                .into_any_element()
        } else if snapshot.messages.is_empty() {
            let welcome_title = if is_dm_view {
                selected_channel_name.clone()
            } else {
                format!("Welcome to #{selected_channel_name}!")
            };
            let welcome_sub = if is_dm_view {
                "This is the beginning of your direct message history.".to_owned()
            } else {
                format!("This is the start of the #{selected_channel_name} channel.")
            };

            let mut empty_state = div()
                .flex()
                .flex_1()
                .flex_col()
                .justify_end()
                .px(px(16.))
                .pb(px(24.))
                .gap(px(8.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(68.))
                        .h(px(68.))
                        .rounded_full()
                        .bg(theme.surface_container_highest)
                        .text_color(theme.on_surface)
                        .text_xl()
                        .font_weight(gpui::FontWeight::BOLD)
                        .child(if is_dm_view { "@" } else { "#" }),
                )
                .child(
                    div()
                        .text_xl()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme.on_surface)
                        .child(SharedString::from(welcome_title)),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.on_surface_variant)
                        .child(SharedString::from(welcome_sub)),
                );

            if let Some(channel_id) = snapshot.selected_channel_id {
                let backend = self.backend.clone();
                empty_state = empty_state.child(
                    div()
                        .id("load-messages-empty")
                        .mt(px(4.))
                        .w(px(140.))
                        .h(px(32.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(4.))
                        .bg(theme.surface_container_high)
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme.primary)
                        .hover(|s| s.cursor_pointer().bg(theme.secondary_container))
                        .child("Load messages")
                        .on_click(move |_e: &ClickEvent, _w, _cx| {
                            backend.send(BackendCommand::FetchMessages {
                                channel_id,
                                before: None,
                            });
                        }),
                );
            }

            div()
                .id("message-list")
                .flex()
                .flex_col()
                .flex_1()
                .px(px(0.))
                .py(px(0.))
                .overflow_y_scroll()
                .child(empty_state)
                .into_any_element()
        } else {
            let mut rows: Vec<MsgRow> = Vec::new();
            let channel_id = snapshot.selected_channel_id.unwrap();
            let before = snapshot.messages.first().map(|m| m.id);
            rows.push(MsgRow::LoadOlder { channel_id, before });
            rows.push(MsgRow::TopSpacer);

            let mut prev: Option<&backend::MessageView> = None;
            let mut prev_date: Option<String> = None;
            let mut unread_marker_inserted = snapshot.selected_channel_last_read_id.is_none();
            let last_read_id = snapshot.selected_channel_last_read_id;

            for m in snapshot.messages.iter() {
                let curr_date = message_date_key(&m.timestamp);
                if curr_date.is_some() && curr_date != prev_date {
                    let label = curr_date.clone().unwrap_or_default();
                    rows.push(MsgRow::DateSeparator(label));
                    prev_date = curr_date;
                    prev = None;
                }

                if !unread_marker_inserted {
                    if let Some(last_read) = last_read_id {
                        let mid = u64::from(m.id);
                        if mid > last_read {
                            rows.push(MsgRow::UnreadMarker);
                            unread_marker_inserted = true;
                            prev = None;
                        }
                    }
                }

                let grouped = prev.is_some_and(|p| should_group_messages(p, m));
                if grouped {
                    rows.push(MsgRow::GroupedMessage(m.clone()));
                } else {
                    rows.push(MsgRow::FullMessage(m.clone()));
                }
                prev = Some(m);
            }
            rows.push(MsgRow::BottomSpacer);

            let row_count = rows.len();
            let first_id = snapshot.messages.first().map(|m| u64::from(m.id));
            let last_id = snapshot.messages.last().map(|m| u64::from(m.id));
            let last_read = snapshot
                .selected_channel_last_read_id
                .map(|id| u64::from(id));
            let selected_cid = snapshot.selected_channel_id.map(|id| u64::from(id));

            let needs_update = self.message_list_channel_id != selected_cid
                || self.message_list_row_count != row_count
                || self.message_list_first_id != first_id
                || self.message_list_last_id != last_id
                || self.message_list_last_read != last_read;
            if needs_update {
                let channel_changed = self.message_list_channel_id != selected_cid;
                let was_at_end = self.message_list_state.is_scrolled_to_end().unwrap_or(true);
                let appended = !channel_changed
                    && self.message_list_first_id == first_id
                    && self.message_list_last_read == last_read
                    && row_count > self.message_list_row_count;

                if appended {
                    let added = row_count - self.message_list_row_count;
                    self.message_list_state.splice(
                        self.message_list_row_count..self.message_list_row_count,
                        added,
                    );
                } else {
                    self.message_list_state.reset(row_count);
                }

                self.message_list_channel_id = selected_cid;
                self.message_list_row_count = row_count;
                self.message_list_first_id = first_id;
                self.message_list_last_id = last_id;
                self.message_list_last_read = last_read;

                if channel_changed
                    || (appended && was_at_end)
                    || (was_at_end && self.message_list_last_id != last_id)
                {
                    self.message_list_state.scroll_to_end();
                }
            }

            let rows_for_list = rows;
            let backend = self.backend.clone();
            let list_state = self.message_list_state.clone();
            let root_entity = cx.entity();
            let current_user_id = snapshot.current_user_id;
            let selected_guild_id = snapshot.selected_guild_id;

            div()
                .id("message-list")
                .flex()
                .flex_col()
                .flex_1()
                .min_h(px(0.))
                .px(px(0.))
                .py(px(0.))
                .child(
                    list(list_state, move |ix, _window, cx| {
                        let Some(row) = rows_for_list.get(ix).cloned() else {
                            return div().into_any_element();
                        };
                        match row {
                            MsgRow::LoadOlder { channel_id, before } => {
                                let backend = backend.clone();
                                div()
                                    .flex()
                                    .justify_center()
                                    .pt(px(16.))
                                    .pb(px(4.))
                                    .child(
                                        div()
                                            .id(SharedString::from(format!(
                                                "load-older-{channel_id}"
                                            )))
                                            .px(px(12.))
                                            .py(px(6.))
                                            .rounded(px(16.))
                                            .bg(theme.surface_container_high)
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme.primary)
                                            .hover(|s| {
                                                s.cursor_pointer().bg(theme.secondary_container)
                                            })
                                            .child("Load older messages")
                                            .on_click(move |_e: &ClickEvent, _w, _cx| {
                                                if let Some(before) = before {
                                                    backend.send(BackendCommand::FetchMessages {
                                                        channel_id,
                                                        before: Some(before),
                                                    });
                                                }
                                            }),
                                    )
                                    .into_any_element()
                            }
                            MsgRow::TopSpacer => div().h(px(16.)).into_any_element(),
                            MsgRow::DateSeparator(label) => div()
                                .flex()
                                .items_center()
                                .gap(px(12.))
                                .px(px(16.))
                                .py(px(16.))
                                .child(
                                    div()
                                        .h(px(1.))
                                        .flex_1()
                                        .bg(theme.outline_variant.opacity(0.5)),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(theme.on_surface_variant)
                                        .child(SharedString::from(label)),
                                )
                                .child(
                                    div()
                                        .h(px(1.))
                                        .flex_1()
                                        .bg(theme.outline_variant.opacity(0.5)),
                                )
                                .into_any_element(),
                            MsgRow::UnreadMarker => div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .px(px(16.))
                                .py(px(8.))
                                .child(div().h(px(1.)).flex_1().bg(theme.error))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(theme.error)
                                        .child("NEW"),
                                )
                                .child(div().h(px(1.)).flex_1().bg(theme.error))
                                .into_any_element(),
                            MsgRow::GroupedMessage(m) => {
                                let mut content = m.content.clone();
                                if m.edited && !content.is_empty() {
                                    content.push_str(" (edited)");
                                }
                                let mut body =
                                    div().flex().flex_col().flex_1().min_w(px(0.)).gap(px(2.));
                                if !content.is_empty() {
                                    body = body.child(
                                        div()
                                            .text_sm()
                                            .text_color(theme.on_surface)
                                            .child(SharedString::from(content.clone())),
                                    );
                                }
                                for extra in render_message_extras(&m, &theme) {
                                    body = body.child(extra);
                                }
                                let short_ts = format_time_hhmm(&m.timestamp);
                                let can_edit =
                                    current_user_id.is_some_and(|uid| uid == m.author_id);
                                let reply_target = crate::ComposerReplyTarget {
                                    message_id: u64::from(m.id),
                                    channel_id: channel_id,
                                    guild_id: selected_guild_id.map(u64::from),
                                    author: m.author.clone(),
                                    content: if m.content.trim().is_empty() {
                                        "(attachment/message)".to_owned()
                                    } else {
                                        m.content.chars().take(120).collect::<String>()
                                    },
                                };
                                let edit_target = crate::ComposerEditTarget {
                                    message_id: u64::from(m.id),
                                    channel_id,
                                    original_content: m.content.clone(),
                                };
                                let mut actions =
                                    div().flex().items_center().gap(px(2.)).opacity(0.85);
                                {
                                    let root_entity = root_entity.clone();
                                    let reply_target = reply_target.clone();
                                    actions = actions.child(
                                        div()
                                            .id(SharedString::from(format!("msg-reply-{}", m.id)))
                                            .w(px(24.))
                                            .h(px(24.))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .rounded(px(4.))
                                            .hover(|s| {
                                                s.bg(theme.surface_container_high).cursor_pointer()
                                            })
                                            .child(
                                                icons::rounded::Reply
                                                    .element(16.0, theme.on_surface_variant),
                                            )
                                            .on_click(move |_e: &ClickEvent, _w, cx| {
                                                root_entity.update(cx, |root, cx| {
                                                    root.begin_reply(reply_target.clone());
                                                    cx.notify();
                                                });
                                            }),
                                    );
                                }
                                if can_edit {
                                    let root_entity = root_entity.clone();
                                    actions = actions.child(
                                        div()
                                            .id(SharedString::from(format!("msg-edit-{}", m.id)))
                                            .w(px(24.))
                                            .h(px(24.))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .rounded(px(4.))
                                            .hover(|s| {
                                                s.bg(theme.surface_container_high).cursor_pointer()
                                            })
                                            .child(
                                                icons::rounded::Edit
                                                    .element(16.0, theme.on_surface_variant),
                                            )
                                            .on_click(move |_e: &ClickEvent, _w, cx| {
                                                root_entity.update(cx, |root, cx| {
                                                    root.begin_edit(edit_target.clone());
                                                    cx.notify();
                                                });
                                            }),
                                    );
                                }
                                div()
                                    .id(SharedString::from(format!("msg-{}", m.id)))
                                    .flex()
                                    .flex_row()
                                    .items_start()
                                    .px(px(16.))
                                    .py(px(1.))
                                    .hover(|s| s.bg(theme.surface_container_low.opacity(0.4)))
                                    .child(
                                        div()
                                            .w(px(56.))
                                            .flex_shrink_0()
                                            .pt(px(2.))
                                            .flex()
                                            .justify_center()
                                            .text_xs()
                                            .text_color(theme.on_surface_variant.opacity(0.0))
                                            .child(SharedString::from(short_ts)),
                                    )
                                    .child(body)
                                    .child(actions)
                                    .into_any_element()
                            }
                            MsgRow::FullMessage(m) => {
                                let mut content = m.content.clone();
                                if m.edited && !content.is_empty() {
                                    content.push_str(" (edited)");
                                }
                                let ts = format_message_timestamp(&m.timestamp);
                                let author_name = m.author.clone();
                                let avatar_url = m.author_avatar_url.clone();

                                let mut body_col = div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .gap(px(2.))
                                    .overflow_hidden();

                                if let Some(preview) = m.reply_preview.as_ref() {
                                    body_col =
                                        body_col.child(render_reply_preview(preview, &theme));
                                } else if m.reply {
                                    body_col = body_col.child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.on_surface_variant)
                                            .child("↱ Reply"),
                                    );
                                }

                                body_col = body_col.child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_baseline()
                                        .gap(px(8.))
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                .text_color(theme.on_surface)
                                                .child(SharedString::from(author_name.clone())),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme.on_surface_variant)
                                                .child(SharedString::from(ts)),
                                        ),
                                );

                                if !content.is_empty() {
                                    body_col = body_col.child(
                                        div()
                                            .text_sm()
                                            .text_color(theme.on_surface)
                                            .child(SharedString::from(content)),
                                    );
                                }
                                for extra in render_message_extras(&m, &theme) {
                                    body_col = body_col.child(extra);
                                }

                                let can_edit =
                                    current_user_id.is_some_and(|uid| uid == m.author_id);
                                let reply_target = crate::ComposerReplyTarget {
                                    message_id: u64::from(m.id),
                                    channel_id,
                                    guild_id: selected_guild_id.map(u64::from),
                                    author: m.author.clone(),
                                    content: if m.content.trim().is_empty() {
                                        "(attachment/message)".to_owned()
                                    } else {
                                        m.content.chars().take(120).collect::<String>()
                                    },
                                };
                                let edit_target = crate::ComposerEditTarget {
                                    message_id: u64::from(m.id),
                                    channel_id,
                                    original_content: m.content.clone(),
                                };
                                let mut actions = div()
                                    .flex()
                                    .items_center()
                                    .gap(px(2.))
                                    .pt(px(2.))
                                    .opacity(0.85);
                                {
                                    let root_entity = root_entity.clone();
                                    let reply_target = reply_target.clone();
                                    actions = actions.child(
                                        div()
                                            .id(SharedString::from(format!("msg-reply-{}", m.id)))
                                            .w(px(24.))
                                            .h(px(24.))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .rounded(px(4.))
                                            .hover(|s| {
                                                s.bg(theme.surface_container_high).cursor_pointer()
                                            })
                                            .child(
                                                icons::rounded::Reply
                                                    .element(16.0, theme.on_surface_variant),
                                            )
                                            .on_click(move |_e: &ClickEvent, _w, cx| {
                                                root_entity.update(cx, |root, cx| {
                                                    root.begin_reply(reply_target.clone());
                                                    cx.notify();
                                                });
                                            }),
                                    );
                                }
                                if can_edit {
                                    let root_entity = root_entity.clone();
                                    actions = actions.child(
                                        div()
                                            .id(SharedString::from(format!("msg-edit-{}", m.id)))
                                            .w(px(24.))
                                            .h(px(24.))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .rounded(px(4.))
                                            .hover(|s| {
                                                s.bg(theme.surface_container_high).cursor_pointer()
                                            })
                                            .child(
                                                icons::rounded::Edit
                                                    .element(16.0, theme.on_surface_variant),
                                            )
                                            .on_click(move |_e: &ClickEvent, _w, cx| {
                                                root_entity.update(cx, |root, cx| {
                                                    root.begin_edit(edit_target.clone());
                                                    cx.notify();
                                                });
                                            }),
                                    );
                                }

                                div()
                                    .id(SharedString::from(format!("msg-{}", m.id)))
                                    .flex()
                                    .flex_row()
                                    .items_start()
                                    .px(px(16.))
                                    .pt(px(14.))
                                    .pb(px(2.))
                                    .hover(|s| s.bg(theme.surface_container_low.opacity(0.4)))
                                    .child(div().w(px(56.)).flex_shrink_0().child(avatar_circle(
                                        SharedString::from(format!("msg-av-{}", m.id)),
                                        avatar_url.as_deref(),
                                        &author_name,
                                        40.0,
                                        &theme,
                                    )))
                                    .child(body_col)
                                    .child(actions)
                                    .into_any_element()
                            }
                            MsgRow::BottomSpacer => div().h(px(24.)).into_any_element(),
                        }
                    })
                    .flex_1()
                    .w_full(),
                )
                .into_any_element()
        };

        let header_left = if is_dm_view {
            let mut row = div().flex().items_center().gap(px(10.));
            if let Some(ch) = selected_channel {
                row = row.child(avatar_circle(
                    "chat-header-avatar",
                    ch.icon_url.as_deref(),
                    &selected_channel_name,
                    24.0,
                    &theme,
                ));
            } else if is_group_dm {
                row = row.child(icons::rounded::Group.element(22.0, theme.on_surface_variant));
            } else {
                row = row.child(
                    div()
                        .text_lg()
                        .text_color(theme.on_surface_variant)
                        .child("@"),
                );
            }
            row.child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme.on_surface)
                    .child(SharedString::from(selected_channel_name.clone())),
            )
        } else {
            let header_icon = channel_icon(channel_kind);
            let mut left = div()
                .flex()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(theme.on_surface_variant)
                        .child(header_icon.element(22.0, theme.on_surface_variant)),
                )
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme.on_surface)
                        .child(SharedString::from(selected_channel_name.clone())),
                );
            if snapshot.selected_channel_id.is_some() {
                left = left
                    .child(div().w(px(1.)).h(px(20.)).bg(theme.outline_variant))
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.on_surface_variant)
                            .child("Channel"),
                    );
            }
            left
        };

        let mut main_col = div()
            .id("main-col")
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .min_w(px(0.))
            .rounded(px(16.))
            .overflow_hidden();

        let show_member_toggle = snapshot.selected_guild_id.is_some()
            && snapshot.selected_channel_id.is_some()
            && !matches!(selected_channel.map(|c| c.kind), Some(2) | Some(13));

        let mut header_actions = div()
            .flex()
            .items_center()
            .gap(px(4.))
            .text_color(theme.on_surface_variant);

        let mk_hdr_icon =
            |id: &'static str, icon: &'static icons::IconDef, active: bool, theme: &M3Theme| {
                let color = if active {
                    theme.on_surface
                } else {
                    theme.on_surface_variant
                };
                div()
                    .id(id)
                    .w(px(32.))
                    .h(px(32.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(4.))
                    .text_color(color)
                    .hover(|s| s.bg(theme.surface_container_high).cursor_pointer())
                    .child(icon.element(20.0, color))
            };

        header_actions = header_actions
            .child(mk_hdr_icon(
                "hdr-search",
                &icons::rounded::Search,
                false,
                &theme,
            ))
            .child(mk_hdr_icon(
                "hdr-pins",
                &icons::rounded::PushPin,
                false,
                &theme,
            ));
        if !is_dm_view {
            header_actions = header_actions.child(mk_hdr_icon(
                "hdr-threads",
                &icons::rounded::Forum,
                false,
                &theme,
            ));
        }
        if show_member_toggle {
            let color = if self.show_member_panel {
                theme.on_surface
            } else {
                theme.on_surface_variant
            };
            header_actions = header_actions.child(
                div()
                    .id("hdr-members-toggle")
                    .w(px(32.))
                    .h(px(32.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(4.))
                    .text_color(color)
                    .hover(|s| s.bg(theme.surface_container_high).cursor_pointer())
                    .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                        this.show_member_panel = !this.show_member_panel;
                        cx.notify();
                    }))
                    .child(icons::rounded::Group.element(20.0, color)),
            );
        }
        main_col = main_col.child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .h(px(48.))
                .px(px(16.))
                .border_b_1()
                .border_color(theme.surface_container_lowest)
                .child(header_left)
                .child(header_actions),
        );

        if let Some(err) = snapshot.error.clone() {
            main_col = main_col.child(
                div()
                    .px(px(16.))
                    .py(px(8.))
                    .bg(theme.error_container)
                    .text_color(theme.on_error_container)
                    .text_xs()
                    .child(SharedString::from(err)),
            );
        }

        main_col = main_col.child(message_list);

        let composer_enabled = snapshot.selected_channel_id.is_some();
        let has_draft = !self.composer.trim().is_empty() || !self.pending_attachments.is_empty();
        let editing_active = self
            .composer_edit
            .as_ref()
            .map(|e| {
                snapshot
                    .selected_channel_id
                    .map(|cid| cid == e.channel_id)
                    .unwrap_or(false)
            })
            .unwrap_or(false);
        let replying_active = self
            .composer_reply
            .as_ref()
            .map(|r| {
                snapshot
                    .selected_channel_id
                    .map(|cid| cid == r.channel_id)
                    .unwrap_or(false)
            })
            .unwrap_or(false);

        let can_send = composer_enabled
            && (if editing_active {
                !self.composer.trim().is_empty()
            } else {
                !self.composer.trim().is_empty()
            });

        let composer_placeholder = if snapshot.selected_channel_id.is_some() {
            if is_dm_view {
                format!("Message @{selected_channel_name}")
            } else {
                format!("Message #{selected_channel_name}")
            }
        } else {
            "Select a channel to message".to_owned()
        };

        let mut composer_wrap = div()
            .id("composer-wrap")
            .flex()
            .flex_col()
            .px(px(8.))
            .pt(px(2.))
            .pb(px(8.))
            .gap(px(6.));

        if replying_active {
            if let Some(reply) = self.composer_reply.clone() {
                composer_wrap = composer_wrap.child(
                    div()
                        .id("composer-reply-bar")
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(12.))
                        .py(px(6.))
                        .rounded_t(px(12.))
                        .rounded_b(px(4.))
                        .bg(theme.surface_container_high)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .min_w(px(0.))
                                .child(div().text_xs().text_color(theme.on_surface_variant).child(
                                    SharedString::from(format!("Replying to {}", reply.author)),
                                ))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.on_surface_variant.opacity(0.8))
                                        .overflow_hidden()
                                        .max_w(px(420.))
                                        .child(SharedString::from(reply.content)),
                                ),
                        )
                        .child(
                            div()
                                .id("composer-reply-cancel")
                                .w(px(24.))
                                .h(px(24.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded_full()
                                .hover(|s| s.bg(theme.surface_container_highest).cursor_pointer())
                                .child(
                                    icons::rounded::Close.element(16.0, theme.on_surface_variant),
                                )
                                .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                                    this.composer_reply = None;
                                    cx.notify();
                                })),
                        ),
                );
            }
        }

        if editing_active {
            if let Some(edit) = self.composer_edit.clone() {
                composer_wrap = composer_wrap.child(
                    div()
                        .id("composer-edit-bar")
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(12.))
                        .py(px(6.))
                        .rounded_t(px(12.))
                        .rounded_b(px(4.))
                        .bg(theme.surface_container_high)
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.primary)
                                .child("Editing message"),
                        )
                        .child(
                            div()
                                .id("composer-edit-cancel")
                                .w(px(24.))
                                .h(px(24.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded_full()
                                .hover(|s| s.bg(theme.surface_container_highest).cursor_pointer())
                                .child(
                                    icons::rounded::Close.element(16.0, theme.on_surface_variant),
                                )
                                .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                                    this.cancel_reply_or_edit();
                                    cx.notify();
                                })),
                        ),
                );
                let _ = edit;
            }
        }

        if !self.pending_attachments.is_empty() {
            let mut atts = div()
                .id("composer-attachments")
                .flex()
                .flex_row()
                .flex_wrap()
                .gap(px(8.))
                .px(px(4.));
            for (i, att) in self.pending_attachments.iter().cloned().enumerate() {
                atts = atts.child(
                    div()
                        .id(SharedString::from(format!("composer-att-{i}")))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .px(px(10.))
                        .py(px(6.))
                        .rounded(px(8.))
                        .bg(theme.surface_container_high)
                        .border_1()
                        .border_color(theme.outline_variant)
                        .child(icons::rounded::Description.element(16.0, theme.on_surface_variant))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.on_surface)
                                .child(SharedString::from(att.filename)),
                        )
                        .child(
                            div()
                                .id(SharedString::from(format!("composer-att-rm-{i}")))
                                .w(px(18.))
                                .h(px(18.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded_full()
                                .hover(|s| s.bg(theme.surface_container_highest).cursor_pointer())
                                .child(
                                    icons::rounded::Close.element(12.0, theme.on_surface_variant),
                                )
                                .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                                    if i < this.pending_attachments.len() {
                                        this.pending_attachments.remove(i);
                                        cx.notify();
                                    }
                                })),
                        ),
                );
            }
            composer_wrap = composer_wrap.child(atts);
        }

        let mut composer_row = div().flex().flex_row().items_end().gap(px(4.));

        composer_row = composer_row.child(
            div()
                .id("composer-attach")
                .w(px(44.))
                .h(px(44.))
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .bg(theme.surface_container_high)
                .text_color(theme.on_surface_variant)
                .when(composer_enabled, |d| {
                    d.hover(|s| s.cursor_pointer().bg(theme.surface_container_highest))
                })
                .on_click(cx.listener(|_this, _e: &ClickEvent, _w, cx| {
                    let receiver = cx.prompt_for_paths(PathPromptOptions {
                        files: true,
                        directories: false,
                        multiple: true,
                        prompt: Some("Select attachments".into()),
                    });
                    cx.spawn(async move |this, cx| {
                        let Ok(result) = receiver.await else {
                            return;
                        };
                        let Ok(Some(paths)) = result else {
                            return;
                        };
                        if paths.is_empty() {
                            return;
                        }
                        let Some(entity) = this.upgrade() else {
                            return;
                        };
                        let _ = cx.update_entity(&entity, |this: &mut Root, cx| {
                            for path in paths {
                                let filename = path
                                    .file_name()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("file")
                                    .to_owned();
                                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                                this.pending_attachments.push(crate::PendingAttachment {
                                    path,
                                    filename,
                                    size,
                                    content_type: None,
                                });
                            }
                            cx.notify();
                        });
                    })
                    .detach();
                }))
                .child(icons::rounded::Add.element(24.0, theme.on_surface_variant)),
        );

        let root_entity = cx.entity();
        let focus_for_click = self.focus_handle.clone();
        let focus_for_input = self.focus_handle.clone();
        let is_composer_focused = self.focus_handle.is_focused(window);
        let composer_text = self.composer.clone();
        let composer_selected_range = self.composer_selected_range.clone();
        let composer_selection_reversed = self.composer_selection_reversed;
        let composer_marked_range = self.composer_marked_range.clone();
        let composer_theme = theme;
        let placeholder_text = composer_placeholder.clone();
        let show_placeholder = composer_text.is_empty() && !is_composer_focused;

        let display_text = if show_placeholder {
            placeholder_text.clone()
        } else {
            composer_text.replace('\n', " ")
        };
        let display_selected_range = if show_placeholder {
            0..0
        } else {
            let display = display_text.clone();
            let start = composer_selected_range.start.min(display.len());
            let end = composer_selected_range.end.min(display.len());
            start..end
        };
        let display_marked_range = if show_placeholder {
            None
        } else {
            composer_marked_range.clone().map(|r| {
                let start = r.start.min(display_text.len());
                let end = r.end.min(display_text.len());
                start..end
            })
        };

        let input_canvas =
            canvas(
                move |bounds, _window, _cx| {
                    (
                        bounds,
                        display_text.clone(),
                        display_selected_range.clone(),
                        display_marked_range.clone(),
                        is_composer_focused,
                        show_placeholder,
                    )
                },
                {
                    let root_entity = root_entity.clone();
                    let focus_for_input = focus_for_input.clone();
                    move |bounds: Bounds<Pixels>,
                          prepaint: (
                        Bounds<Pixels>,
                        String,
                        std::ops::Range<usize>,
                        Option<std::ops::Range<usize>>,
                        bool,
                        bool,
                    ),
                          window,
                          cx| {
                        let (
                            _b,
                            display_text,
                            selected_range,
                            marked_range,
                            focused,
                            show_placeholder,
                        ) = prepaint;
                        window.handle_input(
                            &focus_for_input,
                            ElementInputHandler::new(bounds, root_entity.clone()),
                            cx,
                        );
                        root_entity.update(cx, |this, _cx| {
                            this.composer_last_bounds = Some(bounds);
                        });

                        if show_placeholder {
                            let style = window.text_style();
                            let run = TextRun {
                                len: display_text.len(),
                                font: style.font(),
                                color: composer_theme.on_surface_variant.opacity(0.6).into(),
                                background_color: None,
                                underline: None,
                                strikethrough: None,
                            };
                            let line = window.text_system().shape_line(
                                SharedString::from(display_text),
                                px(14.),
                                &[run],
                                None,
                            );
                            line.paint(
                                bounds.origin,
                                window.line_height(),
                                TextAlign::Left,
                                None,
                                window,
                                cx,
                            )
                            .ok();
                            return;
                        }

                        let font_size = px(14.);
                        let line_height = window.line_height();

                        // selection
                        if !selected_range.is_empty() {
                            let style = window.text_style();
                            let run = style.clone().to_run(display_text.len());
                            let line = window.text_system().shape_line(
                                SharedString::from(display_text.clone()),
                                font_size,
                                &[run],
                                None,
                            );
                            let x0 = line.x_for_index(selected_range.start);
                            let x1 = line.x_for_index(selected_range.end);
                            window.paint_quad(fill(
                                Bounds::from_corners(
                                    point(bounds.left() + x0, bounds.top()),
                                    point(bounds.left() + x1, bounds.top() + line_height),
                                ),
                                composer_theme.primary.opacity(0.25),
                            ));
                        }

                        // marked underline (IME composition)
                        if let Some(marked) = marked_range {
                            if !marked.is_empty() {
                                let style = window.text_style();
                                let run = style.clone().to_run(display_text.len());
                                let line = window.text_system().shape_line(
                                    SharedString::from(display_text.clone()),
                                    font_size,
                                    &[run],
                                    None,
                                );
                                let x0 = line.x_for_index(marked.start);
                                let x1 = line.x_for_index(marked.end);
                                window.paint_quad(fill(
                                    Bounds::from_corners(
                                        point(
                                            bounds.left() + x0,
                                            bounds.top() + line_height - px(2.),
                                        ),
                                        point(bounds.left() + x1, bounds.top() + line_height),
                                    ),
                                    composer_theme.primary,
                                ));
                            }
                        }

                        // text
                        let style = window.text_style();
                        let run = TextRun {
                            len: display_text.len(),
                            font: style.font(),
                            color: composer_theme.on_surface.into(),
                            background_color: None,
                            underline: None,
                            strikethrough: None,
                        };
                        let line = window.text_system().shape_line(
                            SharedString::from(display_text.clone()),
                            font_size,
                            &[run],
                            None,
                        );
                        line.paint(
                            bounds.origin,
                            line_height,
                            TextAlign::Left,
                            None,
                            window,
                            cx,
                        )
                        .ok();

                        // caret
                        if focused && selected_range.is_empty() {
                            let caret_x = line.x_for_index(selected_range.end);
                            window.paint_quad(fill(
                                Bounds::from_corners(
                                    point(bounds.left() + caret_x, bounds.top()),
                                    point(
                                        bounds.left() + caret_x + px(1.5),
                                        bounds.top() + line_height,
                                    ),
                                ),
                                composer_theme.primary,
                            ));
                        }
                    }
                },
            )
            .flex_1()
            .h(px(22.));

        let mut input_surface = div()
            .id("composer-box")
            .flex()
            .items_center()
            .flex_1()
            .min_h(px(44.))
            .max_h(px(200.))
            .px(px(16.))
            .py(px(10.))
            .rounded(px(16.))
            .bg(theme.surface_container_high)
            .overflow_hidden()
            .text_base()
            .cursor_text()
            .child(input_canvas)
            .child(
                div()
                    .id("composer-emoji")
                    .w(px(32.))
                    .h(px(32.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .text_color(theme.on_surface_variant)
                    .when(composer_enabled, |d| {
                        d.hover(|s| s.cursor_pointer().bg(theme.surface_container_highest))
                    })
                    .child(
                        icons::rounded::SentimentSatisfied.element(22.0, theme.on_surface_variant),
                    ),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                    window.focus(&focus_for_click, cx);
                    let byte = this.offset_from_point(event.position, window);
                    this.composer_move_to(byte);
                    this.composer_is_selecting = true;
                    cx.notify();
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, window, cx| {
                if this.composer_is_selecting && event.pressed_button == Some(MouseButton::Left) {
                    let byte = this.offset_from_point(event.position, window);
                    this.composer_select_to(byte);
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event: &MouseUpEvent, _window, cx| {
                    if this.composer_is_selecting {
                        this.composer_is_selecting = false;
                        cx.notify();
                    }
                }),
            );

        composer_row = composer_row.child(input_surface);

        if can_send || editing_active {
            composer_row = composer_row.child(div().w(px(4.)));
            composer_row = composer_row.child(
                div()
                    .id("composer-send")
                    .w(px(44.))
                    .h(px(44.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .bg(if can_send {
                        theme.primary
                    } else {
                        theme.surface_container_highest
                    })
                    .text_color(if can_send {
                        theme.on_primary
                    } else {
                        theme.on_surface_variant
                    })
                    .when(can_send, |d| d.hover(|s| s.cursor_pointer().opacity(0.92)))
                    .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                        this.send_composer(cx);
                        cx.notify();
                    }))
                    .child(icons::rounded::Send.element(
                        22.0,
                        if can_send {
                            theme.on_primary
                        } else {
                            theme.on_surface_variant
                        },
                    )),
            );
        }

        composer_wrap = composer_wrap.child(composer_row);
        main_col = main_col.child(composer_wrap);

        let show_member_panel = self.show_member_panel
            && snapshot.selected_guild_id.is_some()
            && snapshot.selected_channel_id.is_some()
            && !matches!(selected_channel.map(|c| c.kind), Some(2) | Some(13));

        let member_panel = if show_member_panel {
            let guild_name = snapshot
                .selected_guild_id
                .and_then(|gid| snapshot.guilds.iter().find(|g| g.id == gid))
                .map(|g| g.name.clone())
                .unwrap_or_else(|| "Server".to_owned());
            let member_count_label = snapshot
                .selected_guild_id
                .and_then(|gid| snapshot.guilds.iter().find(|g| g.id == gid))
                .and_then(|g| g.member_count)
                .map(|n| format!("Members — {n}"))
                .unwrap_or_else(|| "Members".to_owned());

            let mut list = div()
                .id("member-list")
                .flex()
                .flex_col()
                .flex_1()
                .overflow_y_scroll()
                .px(px(8.))
                .py(px(8.))
                .gap(px(2.));

            if let Some(name) = snapshot.current_user.clone() {
                list = list.child(
                    div()
                        .px(px(8.))
                        .pt(px(12.))
                        .pb(px(4.))
                        .text_xs()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme.on_surface_variant)
                        .child("ONLINE — 1"),
                );
                list = list.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .h(px(42.))
                        .px(px(8.))
                        .rounded(px(8.))
                        .hover(|s| s.bg(theme.surface_container_high))
                        .child(avatar_circle(
                            "member-self-avatar",
                            snapshot.current_user_avatar_url.as_deref(),
                            &name,
                            32.0,
                            &theme,
                        ))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(theme.on_surface)
                                        .child(SharedString::from(name)),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.on_surface_variant)
                                        .child("Online"),
                                ),
                        ),
                );
            }
            list = list.child(
                div()
                    .px(px(8.))
                    .pt(px(16.))
                    .text_xs()
                    .text_color(theme.on_surface_variant)
                    .child(SharedString::from(format!(
                        "Member list for {guild_name} will appear here when presence/member data is available."
                    ))),
            );

            Some(
                div()
                    .id("member-panel")
                    .flex()
                    .flex_col()
                    .w(px(240.))
                    .h_full()
                    .rounded(px(16.))
                    .overflow_hidden()
                    .bg(theme.background)
                    .child(
                        div()
                            .h(px(48.))
                            .px(px(16.))
                            .flex()
                            .items_center()
                            .border_b_1()
                            .border_color(theme.surface_container_lowest)
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.on_surface_variant)
                                    .child(SharedString::from(member_count_label)),
                            ),
                    )
                    .child(list),
            )
        } else {
            None
        };

        let sidebar_column = div()
            .id("sidebar-column")
            .flex()
            .flex_col()
            .w(px(312.))
            .h_full()
            .gap(px(8.))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_h(px(0.))
                    .child(guild_rail)
                    .child(channel_panel),
            )
            .child(account_panel);

        let mut main_row = div()
            .flex()
            .flex_row()
            .flex_1()
            .min_w(px(0.))
            .h_full()
            .gap(px(8.))
            .child(sidebar_column)
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_w(px(0.))
                    .h_full()
                    .rounded(px(16.))
                    .overflow_hidden()
                    .bg(theme.background)
                    .child(main_col),
            );

        if let Some(panel) = member_panel {
            main_row = main_row.child(panel);
        }

        let mut root = div()
            .id("main-root")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::on_key_down))
            .relative()
            .flex()
            .size_full()
            .p(px(8.))
            .bg(theme.surface_container_high)
            .text_color(theme.on_surface)
            .child(main_row);

        if self.show_devices_panel {
            root = root.child(self.render_devices_panel(cx, &snapshot));
        }

        if self.page == AppPage::Settings {
            root = root.child(self.render_settings_overlay(cx, &snapshot));
        }

        if self.show_debug_stats {
            let lines = debug_stats::format_overlay_lines();
            let mut panel = div()
                .id("debug-stats-overlay")
                .absolute()
                .top(px(12.))
                .right(px(12.))
                .w(px(420.))
                .max_h(px(320.))
                .flex()
                .flex_col()
                .gap(px(4.))
                .p(px(10.))
                .rounded(px(10.))
                .bg(gpui::hsla(0.0, 0.0, 0.05, 0.88))
                .border_1()
                .border_color(theme.outline_variant)
                .text_color(gpui::white())
                .text_xs()
                .overflow_hidden();
            for (i, line) in lines.into_iter().enumerate() {
                panel = panel.child(
                    div()
                        .id(SharedString::from(format!("dbg-line-{i}")))
                        .child(SharedString::from(line)),
                );
            }
            root = root.child(panel);
        }

        root
    }
}
