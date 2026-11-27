use crate::updater;
use requiem_updater::custom;
use requiem_updater::run;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use std::time::SystemTime;
use std::time;

use requiem_core::ui::context_menu::{ConfirmRequest, confirm};
use requiem_ui::colors::{TEXT_CRITICAL, TEXT_STRONG};
use requiem_ui::components::{
    AccordionProps, ButtonProps, ButtonVariant, FormItemProps, SwitchProps, TextInputProps,
    TextProps, TextVariant, div,
};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_updater::KeyEntry;
use requiem_updater::run::State;

use super::settings_page::{self, Switches, category, plain, rich, setting};
use crate::shell::classes::update_settings as cls;
use crate::updater::UpdateEvent;

const ARM_DELAY: Duration = Duration::from_secs(10);

#[derive(Clone)]
enum Add {
    Idle,
    Fetching,
    Failed(String),
    Review(Review),
}

#[derive(Clone)]
struct Review {
    base: String,
    host: String,
    keys: Vec<KeyEntry>,

    armed: bool,
}

struct Page {
    revision: Signal<u64>,
    switches: Switches,
    danger_open: Signal<bool>,
    add: Signal<Add>,

    url: RefCell<String>,
    typed: RefCell<String>,

    host_matches: Signal<bool>,
}

fn page() -> &'static Page {
    thread_local! {
        static PAGE: &'static Page = Box::leak(Box::new(Page {
            revision: signal(0),
            switches: Switches::default(),
            danger_open: signal(false),
            add: signal(Add::Idle),
            url: RefCell::new(String::new()),
            typed: RefCell::new(String::new()),
            host_matches: signal(false),
        }));
    }
    PAGE.with(|page| *page)
}

fn bump() {
    page().revision.update(|r| *r += 1);
}

fn reset_flow() {
    let page = page();
    page.typed.borrow_mut().clear();
    page.host_matches.set(false);
    page.add.set(Add::Idle);
}

pub fn on_event(event: UpdateEvent) {
    let page = page();
    match event {
        UpdateEvent::Keys { base, result } => {
            if !matches!(page.add.peek(), Add::Fetching) {
                return;
            }
            match result {
                Ok(keys) => {
                    page.typed.borrow_mut().clear();
                    page.host_matches.set(false);
                    page.add.set(Add::Review(Review {
                        host: custom::host_of(&base),
                        base,
                        keys,
                        armed: false,
                    }));
                    updater::arm_after(ARM_DELAY);
                }
                Err(reason) => page.add.set(Add::Failed(requiem_i18n::t!(
                    "update-settings-fetch-failed",
                    reason = reason
                ))),
            }
        }
        UpdateEvent::Armed => {
            if let Add::Review(review) = page.add.peek() {
                page.add.set(Add::Review(Review {
                    armed: true,
                    ..review
                }));
            }
        }
        UpdateEvent::State(_) => {}
    }
}

fn status(state: &State) -> String {
    match state {
        State::Idle => requiem_i18n::t!("update-settings-status-idle"),
        State::Checking => requiem_i18n::t!("update-settings-status-checking"),
        State::UpToDate => requiem_i18n::t!("update-settings-status-up-to-date"),
        State::Downloading { version, .. } => requiem_i18n::t!(
            "update-settings-status-downloading",
            version = version.clone()
        ),
        State::Ready(staged) => requiem_i18n::t!(
            "update-settings-status-ready",
            version = staged.version.clone()
        ),
        State::Failed(reason) => {
            requiem_i18n::t!("update-settings-status-failed", reason = reason.clone())
        }
    }
}

fn updates_category() -> Node {
    let db = requiem_db::Db::shared();
    let auto = run::auto_update(db);
    let mut switch = SwitchProps::new(
        page().switches.state("auto-update", auto),
        move |on: bool| {
            run::set_auto_update(requiem_db::Db::shared(), on);
            if let Some(handle) = updater::handle() {
                handle.set_auto(on);
            }
            bump();
        },
    )
    .label(requiem_i18n::t!("update-settings-auto"));
    switch.description_node = plain(requiem_i18n::t!("update-settings-auto-description"));

    let state = updater::state();
    let version_line = format!(
        "{} {}",
        requiem_i18n::t!("update-settings-version"),
        updater::current_version()
    );
    let check = ButtonProps::new(
        ButtonVariant::Secondary,
        requiem_i18n::t!("update-settings-check"),
        || {
            if let Some(handle) = updater::handle() {
                handle.check_now();
            }
        },
    )
    .build();

    let mut rows = vec![
        setting(switch.build()),
        setting(rich(version_line, None)),
        setting(rich(status(&state), None)),
    ];

    if cfg!(windows) {
        rows.push(setting(check));
    }
    category(Some(requiem_i18n::t!("settings-modal-app-updates")), rows)
}

fn custom_notice(host: String) -> Node {
    let restore = ButtonProps::new(
        ButtonVariant::CriticalSecondary,
        requiem_i18n::t!("update-settings-restore"),
        || {
            custom::clear(requiem_db::Db::shared());
            updater::source_changed();
            reset_flow();
            bump();
        },
    )
    .build();
    div()
        .class(cls::WARNING)
        .child(
            TextProps::styled(
                requiem_i18n::t!("update-settings-custom-badge", host = host),
                TextVariant::TEXT_MD_MEDIUM,
            )
            .color(TEXT_CRITICAL)
            .build(),
        )
        .child(restore)
}

fn warning_box() -> Node {
    div()
        .class(cls::WARNING)
        .child(
            TextProps::styled(
                requiem_i18n::t!("update-settings-warning-title"),
                TextVariant::TEXT_MD_MEDIUM,
            )
            .color(TEXT_CRITICAL)
            .build(),
        )
        .child(rich(requiem_i18n::t!("update-settings-warning-body"), None))
}

fn address_step(error: Option<String>) -> Node {
    let mut input = TextInputProps::new()
        .text(page().url.borrow().clone())
        .placeholder(requiem_i18n::t!("update-settings-server-placeholder"))
        .on_change(|text: &str| *page().url.borrow_mut() = text.to_string());
    if let Some(error) = error {
        input = input.error(error);
    }
    let field = FormItemProps::new(
        requiem_i18n::t!("update-settings-server-address"),
        input.build(),
    )
    .build();
    let next = ButtonProps::new(
        ButtonVariant::Secondary,
        requiem_i18n::t!("update-settings-continue"),
        || {
            let typed = page().url.borrow().clone();
            match custom::normalize_base(&typed) {
                Ok(base) => {
                    page().add.set(Add::Fetching);
                    updater::fetch_custom_keys(base);
                }
                Err(reason) => page().add.set(Add::Failed(reason)),
            }
        },
    )
    .build();
    div().class(cls::STEPS).child(field).child(next)
}

fn review_step(review: Review) -> Node {
    let fingerprints: Vec<Node> = review
        .keys
        .iter()
        .filter_map(|key| {
            key.verifying_key()
                .ok()
                .map(|k| (key.id.clone(), requiem_updater::fingerprint(&k)))
        })
        .map(|(id, print)| rich(format!("**{id}**  `{print}`"), None))
        .collect();
    let host = review.host.clone();
    let on_type = {
        let host = host.clone();
        move |text: &str| {
            *page().typed.borrow_mut() = text.to_string();
            let matches = text.trim() == host;
            if page().host_matches.peek() != matches {
                page().host_matches.set(matches);
            }
        }
    };
    let field = FormItemProps::new(
        requiem_i18n::t!("update-settings-type-host", host = host.clone()),
        TextInputProps::new()
            .text(page().typed.borrow().clone())
            .on_change(on_type)
            .build(),
    )
    .build();

    let ready = review.armed && page().host_matches.get();
    let use_it = {
        let review = review.clone();
        ButtonProps::new(
            ButtonVariant::CriticalPrimary,
            requiem_i18n::t!("update-settings-use-server"),
            move || {
                if !(review.armed && page().host_matches.peek()) {
                    return;
                }
                let review = review.clone();
                confirm(ConfirmRequest {
                    title: requiem_i18n::t!("update-settings-final-title"),
                    body: requiem_i18n::t!(
                        "update-settings-final-body",
                        host = review.host.clone()
                    ),
                    confirm_label: requiem_i18n::t!("update-settings-use-server"),
                    on_confirm: Rc::new(move || {
                        let confirmed_at = SystemTime::now()
                            .duration_since(time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        custom::set(
                            requiem_db::Db::shared(),
                            &custom::CustomSource {
                                base: review.base.clone(),
                                keys: review.keys.clone(),
                                confirmed_at,
                            },
                        );
                        updater::source_changed();
                        reset_flow();
                        bump();
                    }),
                    critical: true,
                    content: None,
                });
            },
        )
        .disabled(!ready)
        .build()
    };
    let cancel = ButtonProps::new(
        ButtonVariant::Secondary,
        requiem_i18n::t!("update-settings-cancel"),
        || {
            reset_flow();
            bump();
        },
    )
    .build();

    div()
        .class(cls::STEPS)
        .child(
            TextProps::styled(
                requiem_i18n::t!("update-settings-review-title", host = host),
                TextVariant::TEXT_MD_MEDIUM,
            )
            .color(TEXT_STRONG)
            .build(),
        )
        .child(div().class(cls::FINGERPRINTS).children(fingerprints))
        .child(rich(requiem_i18n::t!("update-settings-review-hint"), None))
        .child(field)
        .child(div().class(cls::ACTIONS).child(use_it).child(cancel))
}

fn danger_category() -> Node {
    let mut content = div().class(cls::STEPS).child(warning_box());
    content = content.child(match page().add.get() {
        Add::Idle => address_step(None),
        Add::Failed(reason) => address_step(Some(reason)),
        Add::Fetching => rich(requiem_i18n::t!("update-settings-fetching"), None),
        Add::Review(review) => review_step(review),
    });
    let accordion = AccordionProps {
        title_closed: requiem_i18n::t!("update-settings-danger-show"),
        title_open: requiem_i18n::t!("update-settings-danger-hide"),
        subtitle_closed: Some(requiem_i18n::t!("update-settings-danger-subtitle")),
        open: page().danger_open.get(),
        on_toggle: Rc::new(|| page().danger_open.update(|o| *o = !*o)),
        content,
    }
    .build();
    category(None, vec![accordion])
}

pub fn build_page() -> Node {
    let content = Node::reactive(move || {
        let _ = page().revision.get();
        let mut categories = vec![updates_category()];
        if let Some(custom) = custom::get(requiem_db::Db::shared()) {
            categories.push(category(
                None,
                vec![custom_notice(custom::host_of(
                    &custom.base,
                ))],
            ));
        }

        if cfg!(windows) {
            categories.push(danger_category());
        }
        settings_page::panel(categories)
    });
    settings_page::page_container(content)
}
