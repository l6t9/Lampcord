use gpui::{
    ClickEvent, Context, IntoElement, ParentElement, SharedString, Styled, div, prelude::*, px,
    rgba,
};

use crate::ui::shell::{SettingsPage, empty_state, panel_card, section_title, settings_nav_item};
use crate::ui::theme::M3Theme;
use crate::{BackendCommand, Root};

impl Root {
    pub fn render_settings_page(
        &mut self,
        cx: &mut Context<Self>,
        snapshot: &crate::backend::Snapshot,
    ) -> impl IntoElement {
        let theme = self.theme;

        let mut nav = div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .w(px(240.))
            .p(px(12.))
            .bg(theme.surface_container_low)
            .border_r_1()
            .border_color(theme.outline_variant)
            .child(
                div()
                    .text_lg()
                    .text_color(theme.on_surface)
                    .px(px(8.))
                    .py(px(6.))
                    .child("Settings"),
            );

        for page in SettingsPage::all().iter().copied() {
            let selected = self.settings_page == page;
            nav = nav.child(
                settings_nav_item(
                    SharedString::from(format!("settings-nav-{page:?}")),
                    page,
                    selected,
                    &theme,
                )
                .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                    this.settings_page = page;
                    if page == SettingsPage::Devices {
                        this.backend.send(BackendCommand::LoadDeviceSessions);
                    }
                    cx.notify();
                })),
            );
        }

        nav = nav.child(div().flex_1()).child(
            div()
                .id("settings-back")
                .flex()
                .items_center()
                .justify_center()
                .px(px(14.))
                .py(px(10.))
                .rounded(px(20.))
                .bg(theme.primary)
                .text_color(theme.on_primary)
                .hover(|s| s.cursor_pointer())
                .child("Done")
                .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                    this.page = crate::ui::shell::AppPage::Main;
                    cx.notify();
                })),
        );

        let content = match self.settings_page {
            SettingsPage::Account => self
                .render_settings_account(snapshot, &theme)
                .into_any_element(),
            SettingsPage::Appearance => self.render_settings_appearance(&theme).into_any_element(),
            SettingsPage::VoiceAndVideo => self.render_settings_voice(&theme).into_any_element(),
            SettingsPage::Devices => self
                .render_settings_devices(cx, snapshot)
                .into_any_element(),
            SettingsPage::About => self
                .render_settings_about(snapshot, &theme)
                .into_any_element(),
        };

        div()
            .id("settings-root")
            .flex()
            .flex_row()
            .size_full()
            .bg(theme.surface)
            .text_color(theme.on_surface)
            .child(nav)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .p(px(24.))
                    .gap(px(16.))
                    .child(
                        div()
                            .text_xl()
                            .text_color(theme.on_surface)
                            .child(self.settings_page.label()),
                    )
                    .child(content),
            )
    }

    fn render_settings_account(
        &self,
        snapshot: &crate::backend::Snapshot,
        theme: &M3Theme,
    ) -> impl IntoElement {
        let mut card = panel_card(theme)
            .child(section_title("Current session", theme))
            .child(
                div()
                    .text_sm()
                    .text_color(theme.on_surface)
                    .child(SharedString::from(
                        snapshot
                            .current_user
                            .clone()
                            .unwrap_or_else(|| "Unknown user".to_owned()),
                    )),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.on_surface_variant)
                    .child(SharedString::from(format!("Status: {}", snapshot.status))),
            );

        if snapshot.saved_accounts.is_empty() {
            card = card.child(empty_state(
                "No saved accounts",
                "Accounts you sign into can appear here for quick switching.",
                theme,
            ));
        } else {
            let mut list = div().flex().flex_col().gap(px(8.));
            for acct in snapshot.saved_accounts.iter().take(20) {
                let label = acct
                    .username
                    .clone()
                    .unwrap_or_else(|| "Saved account".to_owned());
                list = list.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.))
                        .p(px(12.))
                        .rounded(px(14.))
                        .bg(theme.surface_container)
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.on_surface)
                                .child(SharedString::from(label)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.on_surface_variant)
                                .child(SharedString::from(acct.user_id.clone())),
                        ),
                );
            }
            card = card
                .child(section_title("Saved accounts", theme))
                .child(list);
        }

        card
    }

    fn render_settings_appearance(&self, theme: &M3Theme) -> impl IntoElement {
        panel_card(theme)
            .child(section_title("Theme", theme))
            .child(empty_state(
                "M3 theming in progress",
                "Lampcord is now using gpui-material M3 palette foundations. Theme mode, accent, and density controls will be wired here as settings state is ported.",
                theme,
            ))
            .child(section_title("Display", theme))
            .child(
                div()
                    .text_sm()
                    .text_color(theme.on_surface_variant)
                    .child("Message display, chat density, and accessibility appearance options will appear here."),
            )
    }

    fn render_settings_voice(&self, theme: &M3Theme) -> impl IntoElement {
        panel_card(theme)
            .child(section_title("Input / Output", theme))
            .child(empty_state(
                "Voice settings not yet wired",
                "Input/output device selection, volume, and voice processing options will be connected once voice services and settings storage are ported.",
                theme,
            ))
            .child(
                div()
                    .text_sm()
                    .text_color(theme.on_surface_variant)
                    .child("Planned: microphone/speaker selection, input/output levels, noise suppression, and push-to-talk options."),
            )
    }

    fn render_settings_about(
        &self,
        snapshot: &crate::backend::Snapshot,
        theme: &M3Theme,
    ) -> impl IntoElement {
        panel_card(theme)
            .child(section_title("Lampcord", theme))
            .child(
                div()
                    .text_sm()
                    .text_color(theme.on_surface)
                    .child("Rust + GPUI rewrite (M3 UI port in progress)"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.on_surface_variant)
                    .child(SharedString::from(format!(
                        "Gateway/session status: {}",
                        snapshot.status
                    ))),
            )
            .child(div().text_sm().text_color(theme.on_surface_variant).child(
                "Main UI, settings, and onboarding parity are being ported from the KMP client.",
            ))
    }
}

impl Root {
    pub fn render_settings_overlay(
        &mut self,
        cx: &mut Context<Self>,
        snapshot: &crate::backend::Snapshot,
    ) -> impl IntoElement {
        let theme = self.theme;

        div()
            .id("settings-overlay")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .p(px(16.))
            .child(
                div()
                    .id("settings-overlay-scrim")
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                    .bg(rgba(0x00000052))
                    .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                        this.page = crate::ui::shell::AppPage::Main;
                        this.device_challenge_input.clear();
                        this.backend
                            .send(BackendCommand::CancelDeviceSessionChallenge);
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .id("settings-overlay-panel")
                    .relative()
                    .w_full()
                    .h_full()
                    .max_w(px(1180.))
                    .max_h(px(800.))
                    .rounded(px(32.))
                    .overflow_hidden()
                    .bg(theme.surface)
                    .border_1()
                    .border_color(theme.outline_variant)
                    .child(self.render_settings_page(cx, snapshot)),
            )
    }

    pub fn render_settings_devices(
        &mut self,
        cx: &mut Context<Self>,
        snapshot: &crate::backend::Snapshot,
    ) -> impl IntoElement {
        let theme = self.theme;
        let ds = &snapshot.device_sessions;

        let mut body = panel_card(&theme)
            .child(section_title("Your sessions", &theme))
            .child(
                div().flex().flex_row().gap(px(8.)).child(
                    div()
                        .id("settings-devices-refresh")
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
                ),
            );

        if ds.loading {
            body = body.child(
                div()
                    .text_sm()
                    .text_color(theme.on_surface_variant)
                    .child("Loading sessions..."),
            );
        }
        if let Some(err) = ds.error.as_ref() {
            body = body.child(
                div()
                    .text_sm()
                    .text_color(theme.error)
                    .child(SharedString::from(err.clone())),
            );
        }
        if let Some(status) = ds.status.as_ref() {
            body = body.child(
                div()
                    .text_sm()
                    .text_color(theme.on_surface_variant)
                    .child(SharedString::from(status.clone())),
            );
        }

        if let Some(challenge) = ds.challenge.as_ref() {
            let (title, hint, is_password, allow_backup, message) = match challenge {
                crate::backend::DeviceSessionChallenge::Password { message, .. } => (
                    "Confirm with your password",
                    "Enter your password",
                    true,
                    false,
                    message.clone(),
                ),
                crate::backend::DeviceSessionChallenge::Totp {
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
                crate::backend::DeviceSessionChallenge::BackupCode { message, .. } => (
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
                    .id("settings-device-challenge-submit")
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
                            Some(crate::backend::DeviceSessionChallenge::Password { .. }) => {
                                this.backend
                                    .send(BackendCommand::SubmitDeviceSessionPassword {
                                        password: input,
                                    });
                            }
                            Some(crate::backend::DeviceSessionChallenge::Totp { .. })
                            | Some(crate::backend::DeviceSessionChallenge::BackupCode { .. }) => {
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
                        .id("settings-device-challenge-backup")
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
                    .id("settings-device-challenge-cancel")
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
            body = body.child(card.child(actions));
        }

        if !ds.loading && ds.sessions.is_empty() && ds.error.is_none() {
            body = body.child(empty_state(
                "No sessions loaded",
                "Refresh to load devices signed into your account.",
                &theme,
            ));
        } else {
            let mut list = div().flex().flex_col().gap(px(8.));
            for session in ds.sessions.iter().take(50) {
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
                    .id(SharedString::from(format!(
                        "settings-device-{}",
                        session.id_hash
                    )))
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
                                "settings-device-logout-{}",
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
            body = body.child(list);
        }

        body
    }
}
