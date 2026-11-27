use requiem_core::instances;
use requiem_ui::colors::{TEXT_MUTED, TEXT_SECONDARY};
use requiem_ui::components::TextVariant;
use requiem_ui::components::{Align, Justify, StackProps};
use requiem_ui::components::{
    ButtonProps, CheckboxProps, ConfirmModalProps, FormItemProps, HeadingLevel, HeadingProps,
    SelectOption, SelectProps, TextLinkProps, TextProps, div, form_item, scroll_view,
};
use requiem_ui::components::{ButtonVariant, MessageTextProps, message_text};
use requiem_ui::node::Node;
use requiem_ui::widget::anim::ToggleState;
use skia_safe::Color;
use skia_safe::font_style::Weight;

use requiem_core::utils::time::{current_year, months_long};

use super::{
    AuthContext, AuthState, DOB_DOM_ID, Dob, DobPart, Screen, forgot_password, submit_login,
    submit_mfa, submit_register,
};
use crate::shell::classes::auth_screen as cls;

pub(super) fn build(state: &AuthState, ctx: &AuthContext) -> Node {
    let screen = state.screen.get();

    let _ = state.submitting.get();

    let content = match &screen {
        Screen::Login => login(state, ctx),
        Screen::Register => register(state, ctx),
        Screen::Mfa { ticket, instance } => mfa(state, ctx, ticket.clone(), instance.clone()),
    };
    scroll_view()
        .class(cls::SCROLL)
        .child(div().class(cls::CENTER).child(content))
}

fn login(state: &AuthState, ctx: &AuthContext) -> Node {
    let submit = {
        let (state, ctx) = (state.clone(), ctx.clone());
        move || submit_login(&state, &ctx)
    };
    let forgot = {
        let (state, ctx) = (state.clone(), ctx.clone());
        TextLinkProps::new(requiem_i18n::t!("forms-forgot-your-password"), move || {
            forgot_password(&state, &ctx)
        })
        .small(true)
        .build()
    };
    let register = {
        let state = state.clone();
        TextLinkProps::new(requiem_i18n::t!("forms-register"), move || {
            state.switch_screen(Screen::Register)
        })
        .small(true)
        .build()
    };

    let header = div()
        .class(cls::HEADER)
        .child(
            HeadingProps::new(HeadingLevel::H1, requiem_i18n::t!("forms-welcome-back"))
                .variant(TextVariant::HEADING_XL_SEMIBOLD)
                .build(),
        )
        .child(
            TextProps::styled(
                requiem_i18n::t!("forms-we-re-so-excited-to-see"),
                TextVariant::TEXT_MD_NORMAL,
            )
            .build(),
        );

    let form = div()
        .class(cls::FORM)
        .children(state.form.general_message())
        .child(
            state
                .form
                .field(
                    "login",
                    &requiem_i18n::t!("forms-email-or-phone-number"),
                    submit.clone(),
                )
                .required(true)
                .build(),
        )
        .child(
            div()
                .class(cls::PASSWORD_GROUP)
                .child(
                    state
                        .form
                        .field(
                            "password",
                            &requiem_i18n::t!("forms-password"),
                            submit.clone(),
                        )
                        .required(true)
                        .password(true)
                        .build(),
                )
                .child(
                    StackProps::horizontal()
                        .align(Align::Center)
                        .gap(4)
                        .full_width(false)
                        .build()
                        .child(forgot),
                ),
        )
        .child(
            div()
                .class(cls::ACTIONS)
                .child(
                    ButtonProps::new(
                        ButtonVariant::Primary,
                        requiem_i18n::t!("forms-log-in"),
                        submit,
                    )
                    .full_width(true)
                    .build(),
                )
                .child(
                    StackProps::horizontal()
                        .align(Align::Center)
                        .gap(4)
                        .full_width(false)
                        .build()
                        .child(
                            TextProps::styled(
                                requiem_i18n::t!("forms-need-an-account"),
                                TextVariant::TEXT_SM_NORMAL,
                            )
                            .color(TEXT_MUTED)
                            .build(),
                        )
                        .child(register),
                ),
        );

    let qr = div()
        .class(cls::QR)
        .child(
            div().class(cls::QR_CODE).child(
                TextProps::styled(
                    requiem_i18n::t!("forms-qr-login-isn-t-available-yet"),
                    TextVariant::TEXT_SM_NORMAL,
                )
                .color(Color::from_rgb(0x5c, 0x5e, 0x66))
                .center(true)
                .build(),
            ),
        )
        .child(
            HeadingProps::new(
                HeadingLevel::H1,
                requiem_i18n::t!("forms-log-in-with-qr-code"),
            )
            .variant(TextVariant::HEADING_XL_SEMIBOLD)
            .build(),
        )
        .child(
            TextProps::styled(
                requiem_i18n::t!("forms-scan-this-with-the-discord-mobile"),
                TextVariant::TEXT_MD_NORMAL,
            )
            .center(true)
            .strong(vec![(19..37, Weight::SEMI_BOLD)])
            .build(),
        );

    div().class(cls::BOX).class(cls::BOX_EXPANDED).child(
        StackProps::horizontal()
            .align(Align::Center)
            .gap(64)
            .build()
            .child(div().class(cls::MAIN).child(header).child(form))
            .child(qr),
    )
}

fn dob_select(
    state: &AuthState,
    part: DobPart,
    dob: Dob,
    open: Option<DobPart>,
    invalid: bool,
) -> Node {
    let (placeholder, options, selected): (&str, Vec<SelectOption>, Option<u32>) = match part {
        DobPart::Month => (
            &requiem_i18n::t!("forms-month"),
            months_long()
                .into_iter()
                .enumerate()
                .map(|(i, m)| SelectOption::new((i + 1).to_string(), m))
                .collect(),
            dob.month,
        ),
        DobPart::Day => (
            &requiem_i18n::t!("forms-day"),
            (1..=31)
                .map(|d| SelectOption::new(d.to_string(), d.to_string()))
                .collect(),
            dob.day,
        ),
        DobPart::Year => {
            let latest = current_year().saturating_sub(3);
            (
                &requiem_i18n::t!("forms-year"),
                (0..=150)
                    .map(|back| latest - back)
                    .map(|y| SelectOption::new(y.to_string(), y.to_string()))
                    .collect(),
                dob.year,
            )
        }
    };
    let dob_open = state.dob_open.clone();
    let toggle = move || dob_open.update(|o| *o = if *o == Some(part) { None } else { Some(part) });
    let (dob_signal, dob_open) = (state.dob.clone(), state.dob_open.clone());
    let pick = move |value: String| {
        let value = value.parse::<u32>().ok();
        dob_signal.update(|d| match part {
            DobPart::Month => d.month = value,
            DobPart::Day => d.day = value,
            DobPart::Year => d.year = value,
        });
        dob_open.set(None);
    };
    let mut props = SelectProps::new(toggle, pick)
        .options(options)
        .placeholder(placeholder.to_string())
        .open(open == Some(part))
        .invalid(invalid);
    if let Some(value) = selected {
        props = props.selected(value.to_string());
    }
    div()
        .class(if part == DobPart::Month {
            cls::DOB_MONTH
        } else {
            cls::DOB_REST
        })
        .child(props.build())
}

fn register(state: &AuthState, ctx: &AuthContext) -> Node {
    let submit = {
        let (state, ctx) = (state.clone(), ctx.clone());
        move || submit_register(&state, &ctx)
    };
    let (dob, dob_open) = (state.dob.get(), state.dob_open.get());
    let dob_invalid = state.form.errors().field("date_of_birth").is_some();

    let dob_row = StackProps::horizontal()
        .gap(10)
        .build()
        .dom_id(DOB_DOM_ID)
        .children(
            [DobPart::Month, DobPart::Day, DobPart::Year]
                .map(|part| dob_select(state, part, dob, dob_open, dob_invalid)),
        );

    let consent_line = message_text(MessageTextProps {
        content: requiem_i18n::md!("forms-consent"),
        size: 16.0,
        color: TEXT_SECONDARY,
        line_height: Some(20.0),
        ..Default::default()
    })
    .class(cls::CONSENT_LINE);
    let consent = state.consent.clone();
    let consent_box = CheckboxProps::new(state.consent.get(), move |on: bool| {
        consent.update(|t| *t = t.set(on))
    })
    .content(consent_line)
    .build()
    .class(cls::CONSENT);

    let login_link = {
        let state = state.clone();
        TextLinkProps::new(
            requiem_i18n::t!("forms-already-have-an-account-log-in"),
            move || state.switch_screen(Screen::Login),
        )
        .small(true)
        .build()
    };

    let form = div()
        .class(cls::FORM)
        .child(
            state
                .form
                .field("email", &requiem_i18n::t!("forms-email"), submit.clone())
                .required(true)
                .build(),
        )
        .child(
            state
                .form
                .field(
                    "global_name",
                    &requiem_i18n::t!("forms-display-name"),
                    submit.clone(),
                )
                .build(),
        )
        .child(
            state
                .form
                .field(
                    "username",
                    &requiem_i18n::t!("forms-username"),
                    submit.clone(),
                )
                .required(true)
                .build(),
        )
        .child(
            state
                .form
                .field(
                    "password",
                    &requiem_i18n::t!("forms-password"),
                    submit.clone(),
                )
                .required(true)
                .password(true)
                .build(),
        )
        .child(form_item(FormItemProps {
            title: Some(requiem_i18n::t!("forms-date-of-birth")),
            required: true,
            error: state.form.errors().field("date_of_birth"),
            children: dob_row,
            ..Default::default()
        }))
        .child(consent_box)
        .children(state.form.general_message())
        .child(
            ButtonProps::new(
                ButtonVariant::Primary,
                requiem_i18n::t!("forms-create-account"),
                submit,
            )
            .full_width(true)
            .build(),
        )
        .child(
            StackProps::horizontal()
                .align(Align::Center)
                .gap(4)
                .full_width(false)
                .build()
                .child(login_link),
        );

    div()
        .class(cls::BOX)
        .child(
            StackProps::vertical().align(Align::Center).build().child(
                HeadingProps::new(
                    HeadingLevel::H1,
                    requiem_i18n::t!("forms-create-an-account"),
                )
                .variant(TextVariant::HEADING_XL_SEMIBOLD)
                .build(),
            ),
        )
        .child(form)
}

fn mfa(
    state: &AuthState,
    ctx: &AuthContext,
    ticket: String,
    instance: instances::Instance,
) -> Node {
    let submit = {
        let (state, ctx) = (state.clone(), ctx.clone());
        move || submit_mfa(&state, &ctx, ticket.clone(), instance.clone())
    };
    let back = {
        let state = state.clone();
        TextLinkProps::new(requiem_i18n::t!("forms-go-back-to-login"), move || {
            state.switch_screen(Screen::Login)
        })
        .small(true)
        .build()
    };
    let header = div()
        .class(cls::HEADER)
        .child(
            HeadingProps::new(
                HeadingLevel::H1,
                requiem_i18n::t!("forms-multi-factor-authentication"),
            )
            .variant(TextVariant::HEADING_XL_SEMIBOLD)
            .build(),
        )
        .child(
            TextProps::styled(
                requiem_i18n::t!("forms-log-in-with-your-authenticator-app"),
                TextVariant::TEXT_MD_NORMAL,
            )
            .build(),
        );

    let form = div()
        .class(cls::FORM)
        .children(state.form.general_message())
        .child(
            state
                .form
                .field(
                    "code",
                    &requiem_i18n::t!("forms-enter-discord-auth-backup-code"),
                    submit.clone(),
                )
                .required(true)
                .build(),
        )
        .child(
            StackProps::vertical()
                .gap(12)
                .build()
                .child(
                    ButtonProps::new(
                        ButtonVariant::Primary,
                        requiem_i18n::t!("forms-log-in"),
                        submit,
                    )
                    .full_width(true)
                    .build(),
                )
                .child(
                    StackProps::horizontal()
                        .justify(Justify::Center)
                        .gap(0)
                        .build()
                        .child(back),
                ),
        );

    div().class(cls::BOX).child(header).child(form)
}

pub(super) fn reset_notice(state: &AuthState, open: ToggleState, login: String) -> Node {
    let notice = state.reset_notice.clone();
    ConfirmModalProps::new(
        requiem_i18n::t!("forms-instructions-sent"),
        requiem_i18n::t!("forms-okay"),
        move || notice.update(|(toggle, _)| *toggle = toggle.set(false)),
        open,
    )
    .message(requiem_i18n::t!(
        "forms-password-reset-sent",
        login = login.as_str()
    ))
    .notice()
    .build()
}
