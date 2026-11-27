use requiem_ui::components::{
    ButtonProps, DisclosureProps, ModalProps, SelectOption, SelectProps, div,
};
use requiem_ui::components::{ButtonSize, ButtonVariant, ModalSize};
use requiem_ui::node::Node;
use requiem_ui::widget::anim::ToggleState;

use requiem_core::instances::{InstanceOverrides, derived_urls, is_valid_base_url};

use super::{AuthState, PICKER_DOM_ID, Screen};
use crate::shell::classes::auth_screen as cls;

pub(super) fn picker(state: &AuthState) -> Node {
    let store = state.instances.get();
    let open = state.picker_open.get();

    let options = store
        .all()
        .into_iter()
        .map(|i| SelectOption::new(i.id.clone(), i.name.clone()).with_description(i.display_host()))
        .collect::<Vec<_>>();

    let add = {
        let state = state.clone();
        ButtonProps::default()
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Small)
            .full_width(true)
            .label(requiem_i18n::t!("instance-add-an-instance").to_string())
            .on_click(move || {
                state.picker_open.set(false);
                open_add_modal(&state);
            })
            .build()
    };

    let toggle = {
        let picker_open = state.picker_open.clone();
        move || picker_open.update(|o| *o = !*o)
    };
    let pick = {
        let state = state.clone();
        move |id: String| {
            state.instances.update(|s| s.select(&id));
            state.picker_open.set(false);

            if matches!(state.screen.peek(), Screen::Mfa { .. }) {
                state.switch_screen(Screen::Login);
            } else {
                state.form.clear_errors();
            }
        }
    };

    let select = SelectProps::new(toggle, pick)
        .options(options)
        .selected(store.selected.clone())
        .open(open)
        .panel_above(true)
        .footer(vec![add])
        .build();
    div()
        .class(cls::INSTANCE_PICKER)
        .dom_id(PICKER_DOM_ID)
        .child(select)
}

fn open_add_modal(state: &AuthState) {
    state.add_form.reset(Some("name"));
    state.advanced_open.set(false);
    state.add_open.set(ToggleState::new(false).set(true));
}

fn close_add_modal(state: &AuthState) {
    state.add_open.update(|t| *t = t.set(false));
}

fn value(state: &AuthState, key: &str) -> String {
    state.add_form.trimmed(key)
}

fn save(state: &AuthState) {
    let (name, base_url) = (value(state, "name"), value(state, "base_url"));
    let form = &state.add_form;
    if !form.require(&["name", "base_url"]) {
        return;
    }
    if !is_valid_base_url(&base_url) {
        form.set_error("base_url", requiem_i18n::t!("instance-not-a-valid-url"));
        return;
    }
    let non_empty = |key: &str| Some(value(state, key)).filter(|v| !v.is_empty());
    let overrides = InstanceOverrides {
        api: non_empty("api"),
        gateway: non_empty("gateway"),
        cdn: non_empty("cdn"),
    };
    state.instances.update(|s| {
        s.add(name, base_url, overrides);
    });
    state.form.clear_errors();
    close_add_modal(state);
}

pub(super) fn add_modal(state: &AuthState, open: ToggleState) -> Node {
    let derived = derived_urls(&value(state, "base_url")).unwrap_or_default();
    let field = |key: &'static str, title: &str, placeholder: String| {
        let submit = {
            let state = state.clone();
            move || save(&state)
        };
        state
            .add_form
            .field(key, title, submit)
            .placeholder(placeholder)
    };

    let advanced_signal = state.advanced_open.clone();
    let advanced = DisclosureProps::new(move || advanced_signal.update(|o| *o = !*o))
        .title(requiem_i18n::t!("instance-advanced"))
        .description(requiem_i18n::t!("instance-override-the-api-gateway-or-cdn"))
        .open(state.advanced_open.get())
        .content(
            div()
                .class(cls::MODAL_FORM)
                .child(
                    field(
                        "api",
                        &requiem_i18n::t!("instance-api-url"),
                        derived
                            .api
                            .unwrap_or_else(|| "https://example.com/api/v9".into()),
                    )
                    .build(),
                )
                .child(
                    field(
                        "gateway",
                        &requiem_i18n::t!("instance-gateway-url"),
                        derived
                            .gateway
                            .unwrap_or_else(|| "wss://gateway.example.com".into()),
                    )
                    .build(),
                )
                .child(
                    field(
                        "cdn",
                        &requiem_i18n::t!("instance-cdn-url"),
                        derived
                            .cdn
                            .unwrap_or_else(|| "https://cdn.example.com".into()),
                    )
                    .build(),
                ),
        )
        .build();

    let body = div()
        .class(cls::MODAL_FORM)
        .child(
            field(
                "name",
                &requiem_i18n::t!("instance-name"),
                requiem_i18n::t!("instance-my-instance"),
            )
            .required(true)
            .build(),
        )
        .child(
            field(
                "base_url",
                &requiem_i18n::t!("instance-base-url"),
                "chat.example.com".into(),
            )
            .required(true)
            .description(requiem_i18n::t!(
                "instance-the-api-gateway-and-cdn-addresses"
            ))
            .build(),
        )
        .child(advanced);

    let close = {
        let state = state.clone();
        move || close_add_modal(&state)
    };
    let save_button = {
        let state = state.clone();
        ButtonProps::new(
            ButtonVariant::Primary,
            requiem_i18n::t!("instance-add-instance"),
            move || save(&state),
        )
        .build()
    };

    ModalProps::default()
        .size(ModalSize::Small)
        .heading(requiem_i18n::t!("instance-add-an-instance"))
        .dismissable(true)
        .on_dismiss(close.clone())
        .open(open)
        .body(body)
        .footer(vec![
            ButtonProps::new(
                ButtonVariant::Secondary,
                requiem_i18n::t!("instance-cancel"),
                close,
            )
            .build(),
            save_button,
        ])
        .build()
}
