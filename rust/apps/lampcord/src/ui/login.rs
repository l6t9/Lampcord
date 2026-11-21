use gpui::{
    ClickEvent, Context, Div, FontWeight, IntoElement, KeyDownEvent, ParentElement, SharedString,
    Stateful, Styled, div, prelude::*, px, rgb, rgba,
};

use crate::Root;
use crate::backend::BackendCommand;
use crate::ui::theme::M3Theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoginStep {
    Credentials,
    Token,
    Qr,
    Mfa,
    Forgot,
    ForgotSent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoginField {
    Email,
    Password,
    Token,
    MfaCode,
    ForgotLogin,
}

#[derive(Clone, Debug, Default)]
pub struct LoginUiState {
    pub step: LoginStep,
    pub focus: Option<LoginField>,
    pub email: String,
    pub password: String,
    pub token: String,
    pub mfa_code: String,
    pub forgot_login: String,
    pub password_visible: bool,
    pub token_visible: bool,
    pub local_error: Option<String>,
    pub show_other_options: bool,
}

impl Default for LoginStep {
    fn default() -> Self {
        Self::Credentials
    }
}

pub const CAT_ASCII: &str = "          へ   ♡     ╱|、\n     ૮  -   ՛ )      (`   -  7\n       /   ⁻  ៸|       |、⁻〵\n 乀 (ˍ, ل ل        じしˍ,)ノ";

fn surface_card(theme: &M3Theme) -> Div {
    div()
        .flex()
        .flex_col()
        .w(px(480.))
        .max_w(px(480.))
        .p(px(24.))
        .gap(px(16.))
        .bg(theme.surface_container_low)
        .border_1()
        .border_color(theme.outline_variant)
        .rounded(px(28.))
}

fn field_container(theme: &M3Theme, focused: bool, error: bool) -> Div {
    let border = if error {
        theme.error
    } else if focused {
        theme.primary
    } else {
        theme.outline
    };
    let bg = if focused {
        theme.surface_container_high
    } else {
        theme.surface_container_low
    };
    div()
        .flex()
        .flex_col()
        .gap(px(4.))
        .w_full()
        .px(px(16.))
        .py(px(10.))
        .rounded(px(16.))
        .bg(bg)
        .border_2()
        .border_color(border)
}

fn label_text(text: impl Into<SharedString>, theme: &M3Theme, focused: bool, error: bool) -> Div {
    let color = if error {
        theme.error
    } else if focused {
        theme.primary
    } else {
        theme.on_surface_variant
    };
    div().text_xs().text_color(color).child(text.into())
}

fn value_text(
    text: String,
    placeholder: &str,
    theme: &M3Theme,
    masked: bool,
    focused: bool,
) -> Div {
    let has_value = !text.is_empty();
    let mut display = if !has_value {
        String::new()
    } else if masked {
        "•".repeat(text.chars().count().min(48))
    } else {
        text
    };

    let mut color = if has_value {
        theme.on_surface
    } else {
        theme.on_surface_variant
    };

    if focused {
        display.push('│');
        color = if has_value {
            theme.on_surface
        } else {
            theme.primary
        };
    } else if !has_value {
        display = placeholder.to_string();
        color = theme.on_surface_variant;
    }

    div()
        .text_base()
        .text_color(color)
        .child(SharedString::from(display))
}

fn primary_button(
    id: &'static str,
    label: &'static str,
    theme: &M3Theme,
    loading: bool,
) -> Stateful<Div> {
    let text = if loading { "Please wait..." } else { label };
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .w_full()
        .h(px(48.))
        .rounded(px(24.))
        .bg(theme.primary)
        .text_color(theme.on_primary)
        .hover(|s| {
            s.cursor_pointer()
                .bg(theme.primary_container)
                .text_color(theme.on_primary_container)
        })
        .child(SharedString::from(text))
}

fn tonal_button(id: &'static str, label: &'static str, theme: &M3Theme) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .w_full()
        .h(px(44.))
        .rounded(px(22.))
        .bg(theme.secondary_container)
        .text_color(theme.on_secondary_container)
        .hover(|s| s.cursor_pointer().bg(theme.surface_container_highest))
        .child(SharedString::from(label))
}

fn text_button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    theme: &M3Theme,
) -> Stateful<Div> {
    div()
        .id(id.into())
        .flex()
        .items_center()
        .justify_center()
        .px(px(12.))
        .py(px(8.))
        .rounded(px(20.))
        .text_color(theme.primary)
        .hover(|s| s.cursor_pointer().bg(theme.surface_container_high))
        .child(label.into())
}

fn outlined_button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    theme: &M3Theme,
) -> Stateful<Div> {
    div()
        .id(id.into())
        .flex()
        .items_center()
        .justify_center()
        .w_full()
        .h(px(40.))
        .rounded(px(20.))
        .border_1()
        .border_color(theme.outline)
        .text_color(theme.primary)
        .hover(|s| s.cursor_pointer().bg(theme.surface_container_high))
        .child(label.into())
}

pub fn step_title(step: LoginStep) -> (&'static str, &'static str) {
    match step {
        LoginStep::Credentials => ("Welcome back", "Log in to Lampcord"),
        LoginStep::Token => ("Log in with token", "Paste your Discord token to continue"),
        LoginStep::Qr => (
            "Log in with QR code",
            "Scan with the Discord mobile app to sign in",
        ),
        LoginStep::Mfa => (
            "Multi-factor authentication",
            "Enter your authentication code",
        ),
        LoginStep::Forgot => ("Forgot your password?", "Enter your email or phone number"),
        LoginStep::ForgotSent => ("Check your messages", "Password reset instructions"),
    }
}

pub fn render_login(
    root: &mut Root,
    cx: &mut Context<Root>,
    theme: &M3Theme,
    snapshot: &crate::backend::Snapshot,
) -> impl IntoElement {
    let login = root.login.clone();
    let step = login.step;
    let (title, subtitle) = step_title(step);
    let is_loading = snapshot.status.contains("Connecting")
        || snapshot.status.contains("handshake")
        || snapshot.status == "Ready"; // transient
    let backend_error = snapshot.error.clone();
    let display_error = login.local_error.clone().or(backend_error);
    let email_error = login
        .local_error
        .as_deref()
        .is_some_and(|e| e.to_lowercase().contains("email") || e.to_lowercase().contains("phone"));

    let mut card = surface_card(theme).child(
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(12.))
            .child(
                div()
                    .text_sm()
                    .text_color(theme.primary)
                    .child(SharedString::from(CAT_ASCII)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(4.))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.on_surface)
                            .child(title),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.on_surface_variant)
                            .child(subtitle),
                    ),
            ),
    );

    match step {
        LoginStep::Credentials => {
            let email_focused = login.focus == Some(LoginField::Email);
            let pw_focused = login.focus == Some(LoginField::Password);
            let email_val = login.email.clone();
            let pw_val = login.password.clone();
            let pw_visible = login.password_visible;

            card = card
                .child(
                    field_container(theme, email_focused, email_error)
                        .id("email-field")
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.surface_container_high))
                        .child(label_text(
                            "Email or phone number",
                            theme,
                            email_focused,
                            email_error,
                        ))
                        .child(value_text(
                            email_val,
                            "Enter email or phone number",
                            theme,
                            false,
                            email_focused,
                        ))
                        .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                            this.login.focus = Some(LoginField::Email);
                            cx.notify();
                        })),
                )
                .child(
                    field_container(theme, pw_focused, false)
                        .id("password-field")
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.surface_container_high))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .justify_between()
                                .items_center()
                                .child(label_text("Password", theme, pw_focused, false))
                                .child(
                                    div()
                                        .id("pw-toggle")
                                        .text_xs()
                                        .text_color(theme.primary)
                                        .hover(|s| s.cursor_pointer())
                                        .child(if pw_visible { "Hide" } else { "Show" })
                                        .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                                            this.login.password_visible =
                                                !this.login.password_visible;
                                            cx.notify();
                                        })),
                                ),
                        )
                        .child(value_text(
                            pw_val,
                            "Enter password",
                            theme,
                            !pw_visible,
                            pw_focused,
                        ))
                        .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                            this.login.focus = Some(LoginField::Password);
                            cx.notify();
                        })),
                )
                .child(
                    div().flex().justify_end().child(
                        div()
                            .id("forgot-link")
                            .text_xs()
                            .text_color(theme.primary)
                            .hover(|s| s.cursor_pointer())
                            .child("Forgot your password?")
                            .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                                this.backend
                                    .send(crate::backend::BackendCommand::ClearForgotPassword);
                                this.login.step = LoginStep::Forgot;
                                this.login.forgot_login = this.login.email.clone();
                                this.login.focus = Some(LoginField::ForgotLogin);
                                this.login.local_error = None;
                                cx.notify();
                            })),
                    ),
                );

            if let Some(err) = display_error {
                card = card.child(
                    div()
                        .text_sm()
                        .text_color(theme.error)
                        .child(SharedString::from(err)),
                );
            }

            let login_busy = snapshot.login_flow.stage == "busy";
            card = card
                .child(
                    primary_button(
                        "login-btn",
                        "Log In",
                        theme,
                        login_busy || (is_loading && snapshot.status.contains("Connecting")),
                    )
                    .on_click(cx.listener(
                        move |this, _e: &ClickEvent, _w, cx| {
                            if !login_busy {
                                this.submit_credentials_login(cx);
                            }
                        },
                    )),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(4.))
                        .child(
                            text_button("token-login-link", "Log in with token", theme).on_click(
                                cx.listener(|this, _e: &ClickEvent, _w, cx| {
                                    this.login.step = LoginStep::Token;
                                    this.login.focus = Some(LoginField::Token);
                                    this.login.local_error = None;
                                    cx.notify();
                                }),
                            ),
                        )
                        .child(
                            text_button("qr-login-link", "Log in with QR code", theme).on_click(
                                cx.listener(|this, _e: &ClickEvent, _w, cx| {
                                    this.login.step = LoginStep::Qr;
                                    this.login.local_error = None;
                                    this.backend
                                        .send(crate::backend::BackendCommand::StartRemoteAuth);
                                    cx.notify();
                                }),
                            ),
                        ),
                )
                .child(render_qr_panel(theme, &snapshot.remote_auth));
        }
        LoginStep::Token => {
            let token_focused = login.focus == Some(LoginField::Token);
            let token_val = login.token.clone();
            let token_visible = login.token_visible;

            card = card.child(
                field_container(theme, token_focused, false)
                    .id("token-field")
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.surface_container_high))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_between()
                            .items_center()
                            .child(label_text("Discord token", theme, token_focused, false))
                            .child(
                                div()
                                    .id("token-toggle")
                                    .text_xs()
                                    .text_color(theme.primary)
                                    .hover(|s| s.cursor_pointer())
                                    .child(if token_visible { "Hide" } else { "Show" })
                                    .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                                        this.login.token_visible = !this.login.token_visible;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(value_text(
                        token_val,
                        "Paste your token",
                        theme,
                        !token_visible,
                        token_focused,
                    ))
                    .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                        this.login.focus = Some(LoginField::Token);
                        cx.notify();
                    })),
            );

            if let Some(err) = display_error {
                card = card.child(
                    div()
                        .text_sm()
                        .text_color(theme.error)
                        .child(SharedString::from(err)),
                );
            }

            card = card
                .child(
                    primary_button("token-login-btn", "Log In", theme, false).on_click(
                        cx.listener(|this, _e: &ClickEvent, _w, cx| {
                            this.submit_token_login(cx);
                        }),
                    ),
                )
                .child(div().flex().justify_center().child(
                    text_button("back-credentials", "Back to credentials", theme).on_click(
                        cx.listener(|this, _e: &ClickEvent, _w, cx| {
                            this.login.step = LoginStep::Credentials;
                            this.login.focus = Some(LoginField::Email);
                            this.login.local_error = None;
                            cx.notify();
                        }),
                    ),
                ));
        }
        LoginStep::Qr => {
            if let Some(err) = display_error {
                card = card.child(
                    div()
                        .text_sm()
                        .text_color(theme.error)
                        .child(SharedString::from(err)),
                );
            }
            card = card
                .child(render_qr_panel(theme, &snapshot.remote_auth))
                .child(div().flex().justify_center().child(
                    text_button("back-credentials-qr", "Back to credentials", theme).on_click(
                        cx.listener(|this, _e: &ClickEvent, _w, cx| {
                            this.backend
                                .send(crate::backend::BackendCommand::StopRemoteAuth);
                            this.login.step = LoginStep::Credentials;
                            this.login.focus = Some(LoginField::Email);
                            this.login.local_error = None;
                            cx.notify();
                        }),
                    ),
                ));
        }
        LoginStep::Mfa => {
            let flow = &snapshot.login_flow;
            let busy = flow.stage == "busy" || snapshot.status.contains("Connecting");
            let selected = flow
                .selected_mfa_method
                .clone()
                .unwrap_or_else(|| "totp".to_owned());
            let code_focused = login.focus == Some(LoginField::MfaCode);
            let code_label = match selected.as_str() {
                "sms" => "SMS code",
                "backup" => "Backup code",
                _ => "Authentication code",
            };
            let mfa_error = login.local_error.clone().or_else(|| flow.error.clone());

            // Method selector if multiple methods available
            if flow.mfa_methods.len() > 1 {
                let mut methods_row = div().flex().flex_row().gap(px(8.)).flex_wrap();
                for method in &flow.mfa_methods {
                    let m = method.clone();
                    let is_selected = &selected == method;
                    let label = match method.as_str() {
                        "sms" => "SMS",
                        "backup" => "Backup code",
                        _ => "Authenticator",
                    };
                    let mut btn = div()
                        .id(SharedString::from(format!("mfa-method-{method}")))
                        .px(px(12.))
                        .py(px(8.))
                        .rounded(px(16.))
                        .text_sm()
                        .child(label);
                    if is_selected {
                        btn = btn
                            .bg(theme.secondary_container)
                            .text_color(theme.on_secondary_container);
                    } else {
                        btn = btn
                            .bg(theme.surface_container)
                            .text_color(theme.on_surface_variant)
                            .hover(|s| s.cursor_pointer().bg(theme.surface_container_high));
                        let m2 = m.clone();
                        btn = btn.on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                            this.backend
                                .send(crate::backend::BackendCommand::SetMfaMethod {
                                    method: m2.clone(),
                                });
                            this.login.mfa_code.clear();
                            this.login.local_error = None;
                            cx.notify();
                        }));
                    }
                    methods_row = methods_row.child(btn);
                }
                card = card.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.on_surface_variant)
                                .child("Verification method"),
                        )
                        .child(methods_row),
                );
            }

            if selected == "sms" {
                let sms_hint = flow
                    .sms_phone
                    .as_ref()
                    .map(|p| format!("Code sent to {p}"))
                    .unwrap_or_else(|| "We can send a code to your phone.".to_owned());
                card = card.child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.on_surface_variant)
                                .child(SharedString::from(sms_hint)),
                        )
                        .child(
                            text_button("resend-sms", "Send / Resend SMS", theme).on_click(
                                cx.listener(|this, _e: &ClickEvent, _w, cx| {
                                    this.backend
                                        .send(crate::backend::BackendCommand::ResendMfaSms);
                                    this.login.local_error = None;
                                    cx.notify();
                                }),
                            ),
                        ),
                );
            }

            let code_val = login.mfa_code.clone();
            card = card.child(
                field_container(theme, code_focused, mfa_error.is_some())
                    .id("mfa-code-field")
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.surface_container_high))
                    .child(label_text(
                        code_label,
                        theme,
                        code_focused,
                        mfa_error.is_some(),
                    ))
                    .child(value_text(
                        code_val,
                        "Enter code",
                        theme,
                        false,
                        code_focused,
                    ))
                    .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                        this.login.focus = Some(LoginField::MfaCode);
                        cx.notify();
                    })),
            );

            if let Some(err) = mfa_error {
                card = card.child(
                    div()
                        .text_sm()
                        .text_color(theme.error)
                        .child(SharedString::from(err)),
                );
            }

            card = card
                .child(
                    primary_button("mfa-submit-btn", "Verify", theme, busy).on_click(cx.listener(
                        move |this, _e: &ClickEvent, _w, cx| {
                            if !busy {
                                this.submit_mfa_login(cx);
                            }
                        },
                    )),
                )
                .child(div().flex().justify_center().child(
                    text_button("back-credentials-mfa", "Back to login", theme).on_click(
                        cx.listener(|this, _e: &ClickEvent, _w, cx| {
                            this.backend
                                .send(crate::backend::BackendCommand::CancelLoginFlow);
                            this.login.step = LoginStep::Credentials;
                            this.login.focus = Some(LoginField::Email);
                            this.login.local_error = None;
                            this.login.mfa_code.clear();
                            cx.notify();
                        }),
                    ),
                ));
        }
        LoginStep::Forgot => {
            let forgot = &snapshot.forgot_password;
            let busy = forgot.stage == "busy";
            let focused = login.focus == Some(LoginField::ForgotLogin);
            let value = login.forgot_login.clone();
            let forgot_error = login.local_error.clone().or_else(|| {
                if forgot.stage == "failed" || forgot.stage == "phone_required" {
                    forgot.message.clone()
                } else {
                    None
                }
            });

            card = card.child(
                field_container(theme, focused, forgot_error.is_some())
                    .id("forgot-login-field")
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.surface_container_high))
                    .child(label_text(
                        "Email or phone number",
                        theme,
                        focused,
                        forgot_error.is_some(),
                    ))
                    .child(value_text(
                        value,
                        "Enter email or phone number",
                        theme,
                        false,
                        focused,
                    ))
                    .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                        this.login.focus = Some(LoginField::ForgotLogin);
                        cx.notify();
                    })),
            );

            if let Some(err) = forgot_error {
                let color = if forgot.stage == "phone_required" {
                    theme.on_surface_variant
                } else {
                    theme.error
                };
                card = card.child(
                    div()
                        .text_sm()
                        .text_color(color)
                        .child(SharedString::from(err)),
                );
            }

            card = card
                .child(
                    primary_button("forgot-submit-btn", "Send reset instructions", theme, busy)
                        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                            if busy {
                                return;
                            }
                            let login_val = this.login.forgot_login.trim().to_owned();
                            if login_val.is_empty() {
                                this.login.local_error =
                                    Some("Please enter your email or phone number.".to_owned());
                                cx.notify();
                                return;
                            }
                            this.login.local_error = None;
                            this.backend
                                .send(crate::backend::BackendCommand::ForgotPassword {
                                    login: login_val,
                                });
                            cx.notify();
                        })),
                )
                .child(div().flex().justify_center().child(
                    text_button("forgot-back", "Back to login", theme).on_click(cx.listener(
                        |this, _e: &ClickEvent, _w, cx| {
                            this.backend
                                .send(crate::backend::BackendCommand::ClearForgotPassword);
                            this.login.step = LoginStep::Credentials;
                            this.login.focus = Some(LoginField::Email);
                            this.login.local_error = None;
                            cx.notify();
                        },
                    )),
                ));
        }
        LoginStep::ForgotSent => {
            let msg = snapshot.forgot_password.message.clone().unwrap_or_else(|| {
                "If an account exists for that email or phone number, we sent password reset instructions.".to_owned()
            });
            card = card
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.on_surface)
                        .child(SharedString::from(msg)),
                )
                .child(div().flex().justify_center().child(
                    text_button("forgot-done", "Back to login", theme).on_click(cx.listener(
                        |this, _e: &ClickEvent, _w, cx| {
                            this.backend
                                .send(crate::backend::BackendCommand::ClearForgotPassword);
                            this.login.step = LoginStep::Credentials;
                            this.login.focus = Some(LoginField::Email);
                            this.login.local_error = None;
                            cx.notify();
                        },
                    )),
                ));
        }
    }

    if !matches!(
        step,
        LoginStep::Mfa | LoginStep::Forgot | LoginStep::ForgotSent
    ) {
        card = card.child(render_saved_accounts(root, cx, theme, snapshot));
    }

    div()
        .id("login-root")
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .size_full()
        .bg(theme.background)
        .text_color(theme.on_background)
        .p(px(24.))
        .child(card)
}

fn render_qr_matrix(url: &str) -> Option<Div> {
    let code = qrcode::QrCode::new(url.as_bytes()).ok()?;
    let width = code.width();
    let colors = code.to_colors();
    let module = 4.0_f32; // px per module
    let quiet = 4_usize; // quiet zone modules
    let _total = width + quiet * 2;

    let mut grid = div()
        .flex()
        .flex_col()
        .bg(rgb(0xffffff))
        .p(px(module * quiet as f32));

    // Pre-render rows
    let mut idx = 0usize;
    for _y in 0..width {
        let mut row = div().flex().flex_row();
        for _x in 0..width {
            let dark = matches!(colors.get(idx), Some(qrcode::Color::Dark));
            idx += 1;
            row = row.child(div().w(px(module)).h(px(module)).bg(if dark {
                rgb(0x000000)
            } else {
                rgb(0xffffff)
            }));
        }
        grid = grid.child(row);
    }

    Some(
        div()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(0xffffff))
            .rounded(px(12.))
            .p(px(8.))
            .child(grid),
    )
}

fn render_qr_panel(theme: &M3Theme, remote_auth: &crate::backend::RemoteAuthView) -> Div {
    let status_text = if remote_auth.status.is_empty() {
        "Starting QR login...".to_owned()
    } else {
        remote_auth.status.clone()
    };

    let mut panel = div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(12.))
        .mt(px(8.))
        .w_full()
        .child(
            div()
                .text_sm()
                .text_color(theme.on_surface_variant)
                .child(SharedString::from(status_text)),
        );

    let qr_visual: Option<Div> = remote_auth.qr_url.as_deref().and_then(render_qr_matrix);

    let mut card = div()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(10.))
        .w(px(260.))
        .min_h(px(220.))
        .rounded(px(24.))
        .bg(rgba(0xffffffff))
        .border_1()
        .border_color(theme.outline_variant)
        .p(px(16.));

    if let Some(qr) = qr_visual {
        card = card
            .child(qr)
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x444746))
                    .child("Scan with the Discord mobile app"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x666666))
                    .max_w(px(220.))
                    .child(SharedString::from(
                        remote_auth
                            .qr_url
                            .clone()
                            .unwrap_or_else(|| "Waiting for QR URL...".to_owned()),
                    )),
            );
    } else {
        let detail = remote_auth
            .qr_url
            .clone()
            .or_else(|| remote_auth.scanned_user.clone())
            .or_else(|| remote_auth.error.clone());
        card = card
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x1f1f1f))
                    .child("Scan with Discord mobile app"),
            )
            .child(div().text_xs().text_color(rgb(0x444746)).child(
                if remote_auth.scanned_user.is_some() {
                    "Approve the login request on your phone."
                } else if remote_auth.error.is_some() {
                    "Try again or use token login."
                } else {
                    "Waiting for Discord remote auth gateway..."
                },
            ))
            .when_some(detail, |d, detail| {
                d.child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x1f1f1f))
                        .max_w(px(220.))
                        .child(SharedString::from(detail)),
                )
            });
    }

    panel = panel.child(card);

    if let Some(err) = remote_auth.error.as_ref() {
        if remote_auth.qr_url.is_none() && remote_auth.scanned_user.is_none() {
            panel = panel.child(
                div()
                    .text_sm()
                    .text_color(theme.error)
                    .child(SharedString::from(err.clone())),
            );
        }
    }

    panel
}

fn render_saved_accounts(
    _root: &mut Root,
    cx: &mut Context<Root>,
    theme: &M3Theme,
    snapshot: &crate::backend::Snapshot,
) -> Div {
    let mut section = div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .w_full()
        .mt(px(8.))
        .pt(px(12.))
        .border_t_1()
        .border_color(theme.outline_variant);

    section = section.child(
        div()
            .text_xs()
            .text_color(theme.on_surface_variant)
            .child("Saved accounts"),
    );

    if snapshot.saved_accounts.is_empty() {
        section = section.child(
            div()
                .text_sm()
                .text_color(theme.on_surface_variant)
                .child("No saved accounts yet"),
        );
    } else {
        for acct in snapshot.saved_accounts.iter().take(5) {
            let token = acct.token.clone();
            let label = acct
                .username
                .clone()
                .unwrap_or_else(|| format!("User {}", acct.user_id));
            section = section.child(
                div()
                    .id(SharedString::from(format!("saved-{}", acct.user_id)))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(14.))
                    .py(px(10.))
                    .rounded(px(16.))
                    .bg(theme.surface_container)
                    .hover(|s| s.cursor_pointer().bg(theme.surface_container_high))
                    .child(div().text_sm().text_color(theme.on_surface).child(label))
                    .child(div().text_xs().text_color(theme.primary).child("Connect"))
                    .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                        this.backend.send(BackendCommand::Connect {
                            token: token.clone(),
                        });
                        this.login.local_error = None;
                        cx.notify();
                    })),
            );
        }
    }
    section
}

pub fn handle_login_key(root: &mut Root, event: &KeyDownEvent, cx: &mut Context<Root>) -> bool {
    let key = event.keystroke.key.as_str();
    match key {
        "tab" => {
            root.login.focus = match (root.login.step, root.login.focus) {
                (LoginStep::Credentials, Some(LoginField::Email)) => Some(LoginField::Password),
                (LoginStep::Credentials, Some(LoginField::Password)) => Some(LoginField::Email),
                (LoginStep::Credentials, _) => Some(LoginField::Email),
                (LoginStep::Token, _) => Some(LoginField::Token),
                (LoginStep::Mfa, _) => Some(LoginField::MfaCode),
                (LoginStep::Forgot, _) => Some(LoginField::ForgotLogin),
                (LoginStep::ForgotSent, _) => None,
                (LoginStep::Qr, _) => None,
            };
            cx.notify();
            return true;
        }
        "enter" => {
            match root.login.step {
                LoginStep::Credentials => root.submit_credentials_login(cx),
                LoginStep::Token => root.submit_token_login(cx),
                LoginStep::Mfa => root.submit_mfa_login(cx),
                LoginStep::Forgot => {
                    let login_val = root.login.forgot_login.trim().to_owned();
                    if login_val.is_empty() {
                        root.login.local_error =
                            Some("Please enter your email or phone number.".to_owned());
                    } else {
                        root.login.local_error = None;
                        root.backend
                            .send(crate::backend::BackendCommand::ForgotPassword {
                                login: login_val,
                            });
                    }
                    cx.notify();
                }
                LoginStep::ForgotSent => {
                    root.backend
                        .send(crate::backend::BackendCommand::ClearForgotPassword);
                    root.login.step = LoginStep::Credentials;
                    root.login.focus = Some(LoginField::Email);
                    root.login.local_error = None;
                    cx.notify();
                }
                LoginStep::Qr => {}
            }
            return true;
        }
        "escape" => {
            if root.login.step != LoginStep::Credentials {
                if root.login.step == LoginStep::Qr {
                    root.backend
                        .send(crate::backend::BackendCommand::StopRemoteAuth);
                }
                if root.login.step == LoginStep::Mfa {
                    root.backend
                        .send(crate::backend::BackendCommand::CancelLoginFlow);
                }
                if matches!(root.login.step, LoginStep::Forgot | LoginStep::ForgotSent) {
                    root.backend
                        .send(crate::backend::BackendCommand::ClearForgotPassword);
                }
                root.login.step = LoginStep::Credentials;
                root.login.focus = Some(LoginField::Email);
                root.login.local_error = None;
                root.login.mfa_code.clear();
                cx.notify();
                return true;
            }
            root.login.focus = None;
            cx.notify();
            return true;
        }
        "backspace" => {
            match root.login.focus {
                Some(LoginField::Email) => {
                    root.login.email.pop();
                }
                Some(LoginField::Password) => {
                    root.login.password.pop();
                }
                Some(LoginField::Token) => {
                    root.login.token.pop();
                }
                Some(LoginField::MfaCode) => {
                    root.login.mfa_code.pop();
                }
                Some(LoginField::ForgotLogin) => {
                    root.login.forgot_login.pop();
                }
                None => return false,
            }
            root.login.local_error = None;
            cx.notify();
            return true;
        }
        "v" if event.keystroke.modifiers.control || event.keystroke.modifiers.platform => {
            if let Some(item) = cx.read_from_clipboard() {
                if let Some(text) = item.text() {
                    let text = text.replace('\n', "").replace('\r', "");
                    match root.login.focus {
                        Some(LoginField::Email) => root.login.email.push_str(&text),
                        Some(LoginField::Password) => root.login.password.push_str(&text),
                        Some(LoginField::Token) => root.login.token.push_str(&text),
                        Some(LoginField::MfaCode) => root.login.mfa_code.push_str(&text),
                        Some(LoginField::ForgotLogin) => root.login.forgot_login.push_str(&text),
                        None => return false,
                    }
                    root.login.local_error = None;
                    cx.notify();
                    return true;
                }
            }
            return false;
        }
        _ => {}
    }

    if let Some(text) = event.keystroke.key_char.as_ref() {
        if event.keystroke.modifiers.control || event.keystroke.modifiers.platform {
            return false;
        }
        match root.login.focus {
            Some(LoginField::Email) => root.login.email.push_str(text),
            Some(LoginField::Password) => root.login.password.push_str(text),
            Some(LoginField::Token) => root.login.token.push_str(text),
            Some(LoginField::MfaCode) => root.login.mfa_code.push_str(text),
            Some(LoginField::ForgotLogin) => root.login.forgot_login.push_str(text),
            None => return false,
        }
        root.login.local_error = None;
        cx.notify();
        return true;
    } else if key == "space" {
        if event.keystroke.modifiers.control || event.keystroke.modifiers.platform {
            return false;
        }
        match root.login.focus {
            Some(LoginField::Email) => root.login.email.push(' '),
            Some(LoginField::Password) => root.login.password.push(' '),
            Some(LoginField::MfaCode) => root.login.mfa_code.push(' '),
            Some(LoginField::ForgotLogin) => root.login.forgot_login.push(' '),
            Some(LoginField::Token) => {} // tokens usually don't have spaces
            None => return false,
        }
        root.login.local_error = None;
        cx.notify();
        return true;
    }

    false
}
