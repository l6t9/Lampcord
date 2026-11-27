use crate::rest::endpoints::search as endpoints_search;
use crate::ui::classes::search as classes_search;
use crate::ui::classes::search_filters as cls;
use crate::ui::search;
use crate::ui::search_state;
use crate::ui;
use crate::utils::time;
use requiem_ui::colors;
use requiem_ui::components::TextVariant;
use requiem_ui::components::icon;
use requiem_ui::components::{Align, StackProps};
use requiem_ui::components::{
    ButtonProps, FormItemProps, ModalProps, SelectOption, SelectProps, TextLinkProps, TextProps,
    div,
};
use requiem_ui::components::{ButtonVariant, ModalSize};
use requiem_ui::node::Node;
use requiem_ui::widget::anim::ToggleState;
use std::rc::Rc;

use crate::models::Channel;
use crate::state::AppState;

use crate::ui::TEXT_DEFAULT;
use crate::ui::search_state::{AUTHOR_TYPES, Answer, FilterKind, HAS_VALUES, SearchState};
use requiem_ui::icons::{CALENDAR, PLUS_MEDIUM, TRASH};

pub struct FilterChoices {
    pub example_user: Option<String>,

    pub users: Vec<(u64, String)>,
    pub channels: Vec<(u64, String)>,
}

pub fn choices(state: &AppState, channel: &Channel) -> FilterChoices {
    let c = search::candidates(state, channel);
    FilterChoices {
        example_user: state
            .users
            .current
            .with_untracked(|u| u.as_ref().map(|u| state.users.display_name(u))),
        users: c.users.into_iter().map(|u| (u.0, u.1)).collect(),
        channels: c.channels,
    }
}

fn toggle<T: PartialEq>(list: &mut Vec<T>, value: T) {
    match list.iter().position(|v| *v == value) {
        Some(i) => {
            list.remove(i);
        }
        None => list.push(value),
    }
}

#[allow(clippy::too_many_arguments)]
fn multi_select(
    key: &'static str,
    placeholder: &str,
    options: Vec<SelectOption>,
    selected: Vec<String>,
    search: &SearchState,
    on_toggle_value: impl Fn(String) + 'static,
) -> Node {
    let open = search.filters_select.get().as_deref() == Some(key);
    let toggler = search.clone();
    let selecter = search.clone();
    let clearer = search.clone();
    let clear_key = key;
    let mut props = SelectProps::new(
        move || {
            let now = toggler.filters_select.peek();
            toggler.filters_select.set(if now.as_deref() == Some(key) {
                None
            } else {
                Some(key.to_string())
            });
        },
        move |value: String| {
            on_toggle_value(value);

            selecter.filters_select.set(Some(key.to_string()));
        },
    )
    .options(options)
    .multiple(true)
    .selected_values(selected.clone())
    .placeholder(placeholder.to_string())
    .open(open)
    .compact(true);
    if !selected.is_empty() {
        props = props.on_clear(move || {
            let _ = clear_key;
            clearer.filters_draft.update(|d| match clear_key {
                "from" => d.from.clear(),
                "in" => d.in_channels.clear(),
                "has" => d.has.clear(),
                "mentions" => d.mentions.clear(),
                _ => d.author_types.clear(),
            });
        });
    }
    props.build()
}

fn field(title: &str, description: &str, control: Node) -> Node {
    div().class(cls::FIELD).child(
        FormItemProps::default()
            .title(title.to_string())
            .description(description.to_string())
            .children(control)
            .build(),
    )
}

fn answer_options(list: &[(u64, String)]) -> Vec<SelectOption> {
    list.iter()
        .map(|(id, name)| SelectOption::new(id.to_string(), name.clone()))
        .collect()
}

fn answer_for(list: &[(u64, String)], value: &str) -> Option<Answer> {
    list.iter()
        .find(|(id, _)| id.to_string() == value)
        .map(|(id, name)| Answer {
            value: id.to_string(),
            label: name.clone(),
        })
}

pub fn build_filters_modal(
    search: &SearchState,
    choices: &FilterChoices,
    open: ToggleState,
    apply: impl Fn() + Clone + 'static,
) -> Node {
    let draft = search.filters_draft.get();
    let _ = search.filters_tick.get();
    let dates = search.filters_dates.borrow().clone();

    let from_hint = requiem_i18n::t!(
        "search-filters-example",
        example = choices
            .example_user
            .clone()
            .or_else(|| choices.users.first().map(|u| u.1.clone()))
            .unwrap_or_else(|| requiem_i18n::t!("search-filters-example-user"))
    );
    let in_hint = requiem_i18n::t!(
        "search-filters-example",
        example = choices.channels.first().map_or_else(
            || requiem_i18n::t!("search-filters-example-channel"),
            |c| c.1.clone()
        )
    );

    let from = {
        let (s, users) = (search.clone(), choices.users.clone());
        multi_select(
            "from",
            from_hint.as_str(),
            answer_options(&choices.users),
            draft.from.iter().map(|a| a.value.clone()).collect(),
            search,
            move |value| {
                if let Some(answer) = answer_for(&users, &value) {
                    s.filters_draft.update(|d| toggle(&mut d.from, answer));
                }
            },
        )
    };
    let in_channels = {
        let (s, channels) = (search.clone(), choices.channels.clone());
        multi_select(
            "in",
            in_hint.as_str(),
            answer_options(&choices.channels),
            draft.in_channels.iter().map(|a| a.value.clone()).collect(),
            search,
            move |value| {
                if let Some(answer) = answer_for(&channels, &value) {
                    s.filters_draft
                        .update(|d| toggle(&mut d.in_channels, answer));
                }
            },
        )
    };
    let has = {
        let s = search.clone();
        multi_select(
            "has",
            &requiem_i18n::t!("search-filters-any-content"),
            HAS_VALUES
                .iter()
                .map(|v| SelectOption::new(*v, *v))
                .collect(),
            draft.has.clone(),
            search,
            move |value| s.filters_draft.update(|d| toggle(&mut d.has, value)),
        )
    };
    let mentions = {
        let (s, users) = (search.clone(), choices.users.clone());
        multi_select(
            "mentions",
            from_hint.as_str(),
            answer_options(&choices.users),
            draft.mentions.iter().map(|a| a.value.clone()).collect(),
            search,
            move |value| {
                if let Some(answer) = answer_for(&users, &value) {
                    s.filters_draft.update(|d| toggle(&mut d.mentions, answer));
                }
            },
        )
    };
    let author_types = {
        let s = search.clone();
        multi_select(
            "author",
            &requiem_i18n::t!("search-filters-ex-bot"),
            AUTHOR_TYPES
                .iter()
                .map(|v| SelectOption::new(*v, *v))
                .collect(),
            draft.author_types.clone(),
            search,
            move |value| {
                s.filters_draft
                    .update(|d| toggle(&mut d.author_types, value))
            },
        )
    };
    let pinned = {
        let (toggler, selecter) = (search.clone(), search.clone());
        let key = "pinned";
        SelectProps::new(
            move || {
                let now = toggler.filters_select.peek();
                toggler.filters_select.set(if now.as_deref() == Some(key) {
                    None
                } else {
                    Some(key.to_string())
                });
            },
            move |value: String| {
                selecter
                    .filters_draft
                    .update(|d| d.pinned = Some(value == "true"));
                selecter.filters_select.set(None);
            },
        )
        .options(vec![
            SelectOption::new("true", requiem_i18n::t!("search-filters-true")),
            SelectOption::new("false", requiem_i18n::t!("search-filters-false")),
        ])
        .selected(match draft.pinned {
            Some(true) => "true".to_string(),
            Some(false) => "false".to_string(),
            None => String::new(),
        })
        .placeholder(requiem_i18n::t!("search-filters-ex-true").to_string())
        .open(search.filters_select.get().as_deref() == Some(key))
        .compact(true)
        .build()
    };

    let mut date_column = StackProps::vertical().align(Align::Start).build();
    for (index, (kind, value)) in dates.iter().enumerate() {
        let kind_select = {
            let (toggler, selecter) = (search.clone(), search.clone());
            let key = format!("date-{index}");
            let open_key = key.clone();
            SelectProps::new(
                move || {
                    let now = toggler.filters_select.peek();
                    toggler
                        .filters_select
                        .set(if now.as_deref() == Some(key.as_str()) {
                            None
                        } else {
                            Some(key.clone())
                        });
                },
                move |picked: String| {
                    let kind = FilterKind::from_keyword(&picked).unwrap_or(FilterKind::Before);
                    if let Some(row) = selecter.filters_dates.borrow_mut().get_mut(index) {
                        row.0 = kind;
                    }
                    selecter.filters_select.set(None);
                    selecter.filters_tick.update(|n| *n += 1);
                },
            )
            .options(vec![
                SelectOption::new("before", requiem_i18n::t!("search-filters-before")),
                SelectOption::new("during", requiem_i18n::t!("search-filters-during")),
                SelectOption::new("after", requiem_i18n::t!("search-filters-after")),
            ])
            .selected(kind.keyword().to_string())
            .open(search.filters_select.get().as_deref() == Some(open_key.as_str()))
            .compact(true)
            .build()
            .class(cls::DATE_KIND)
        };

        let calendar_key = format!("date-cal-{index}");
        let toggler = search.clone();
        let toggle_key = calendar_key.clone();
        let shown = if value.is_empty() {
            requiem_i18n::t!("search-filters-select-a-date").to_string()
        } else {
            chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .map_or_else(|_| value.clone(), |d| time::date_medium(&d))
        };
        let color = if value.is_empty() {
            colors::INPUT_PLACEHOLDER
        } else {
            TEXT_DEFAULT
        };
        let input = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![
                cls::DATE_INPUT.to_string(),
                cls::DATE_FIELD.to_string(),
            ])
            .on_click(move || {
                let now = toggler.filters_select.peek();
                toggler
                    .filters_select
                    .set(if now.as_deref() == Some(toggle_key.as_str()) {
                        None
                    } else {
                        Some(toggle_key.clone())
                    });
            })
            .build()
            .child(
                TextProps::styled(shown, TextVariant::TEXT_MD_NORMAL)
                    .color(color)
                    .build()
                    .class(cls::DATE_FIELD_TEXT),
            )
            .child(icon(CALENDAR, ui::TEXT_SECONDARY).class(cls::DATE_FIELD_ICON));
        let picker = if search.filters_select.get().as_deref() == Some(calendar_key.as_str()) {
            let picked = search.clone();
            search::calendar_panel(
                search,
                Rc::new(move |date: String| {
                    if let Some(row) = picked.filters_dates.borrow_mut().get_mut(index) {
                        row.1 = date;
                    }
                    picked.filters_select.set(None);
                    picked.filters_tick.update(|n| *n += 1);
                }),
                classes_search::CALENDAR_IN_DIALOG,
            )
        } else {
            div()
        };
        let remover = search.clone();
        let remove = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::DATE_REMOVE.to_string()])
            .on_click(move || {
                let mut rows = remover.filters_dates.borrow_mut();
                if index < rows.len() {
                    rows.remove(index);
                }
                drop(rows);
                remover.filters_tick.update(|n| *n += 1);
            })
            .build()
            .child(icon(TRASH, ui::TEXT_SECONDARY).class(cls::DATE_REMOVE_ICON));
        date_column = date_column.child(
            div()
                .class(cls::DATE_ROW)
                .child(kind_select)
                .child(input)
                .child(remove)
                .child(picker),
        );
    }
    let add_date = {
        let s = search.clone();
        ButtonProps::default()
            .variant(ButtonVariant::Secondary)
            .label(requiem_i18n::t!("search-filters-add-date").to_string())
            .icon(PLUS_MEDIUM)
            .full_width(true)
            .on_click(move || {
                let today = chrono::Local::now()
                    .date_naive()
                    .format("%Y-%m-%d")
                    .to_string();
                s.filters_dates
                    .borrow_mut()
                    .push((FilterKind::Before, today));
                s.filters_tick.update(|n| *n += 1);
            })
            .build()
            .class(cls::ADD_DATE)
    };
    date_column = date_column.child(add_date);

    let body = div()
        .class(cls::BODY)
        .child(field(
            &requiem_i18n::t!("search-filters-from"),
            &requiem_i18n::t!("search-filters-sent-by-any-of-the-selected"),
            from,
        ))
        .child(field(
            &requiem_i18n::t!("search-filters-in"),
            &requiem_i18n::t!("search-filters-sent-in-any-of-the-selected"),
            in_channels,
        ))
        .child(field(
            &requiem_i18n::t!("search-filters-has"),
            &requiem_i18n::t!("search-filters-includes-any-of-the-selected-types"),
            has,
        ))
        .child(field(
            &requiem_i18n::t!("search-filters-mentions"),
            &requiem_i18n::t!("search-filters-mentions-any-of-the-selected-users"),
            mentions,
        ))
        .child(field(
            &requiem_i18n::t!("search-filters-date"),
            &requiem_i18n::t!("search-filters-when-the-message-was-sent"),
            date_column,
        ))
        .child(field(
            &requiem_i18n::t!("search-filters-author-type"),
            &requiem_i18n::t!("search-filters-sent-by-any-of-the-selected-2"),
            author_types,
        ))
        .child(field(
            &requiem_i18n::t!("search-filters-pinned"),
            &requiem_i18n::t!("search-filters-if-the-message-is-pinned-or"),
            pinned,
        ));

    let count = draft.count(&dates);
    let clear = {
        let s = search.clone();
        let label = if count > 0 {
            requiem_i18n::t!("search-filters-clear-count", count = count)
        } else {
            requiem_i18n::t!("search-filters-clear")
        };
        TextLinkProps::default()
            .label(label)
            .small(true)
            .disabled(count == 0)
            .on_click(move || {
                s.filters_draft.set(Default::default());
                s.filters_dates.borrow_mut().clear();
                s.filters_tick.update(|n| *n += 1);
            })
    }
    .build();
    let cancel = {
        let s = search.clone();
        ButtonProps::default()
            .variant(ButtonVariant::Secondary)
            .label(requiem_i18n::t!("search-filters-cancel").to_string())
            .on_click(move || s.close_filters())
            .build()
            .class(cls::FOOTER_BUTTON)
    };

    let changed = {
        let baseline = search.filters_baseline.borrow();
        baseline.0 != draft || baseline.1 != dates
    };
    let apply_button = ButtonProps::default()
        .variant(ButtonVariant::Primary)
        .label(requiem_i18n::t!("search-filters-apply-filters").to_string())
        .disabled(!changed)
        .on_click(apply)
        .build()
        .class(cls::FOOTER_BUTTON);

    let dismiss = {
        let s = search.clone();
        move || s.close_filters()
    };
    ModalProps::default()
        .size(ModalSize::Medium)
        .heading(requiem_i18n::t!("search-filters-filters"))
        .dismissable(true)
        .on_dismiss(dismiss)
        .open(open)
        .body(body)
        .footer_leading(clear)
        .footer(vec![cancel, apply_button])
        .build()
}

pub fn apply(search: &SearchState, scope: endpoints_search::SearchScope) {
    let draft = search.filters_draft.peek();
    let dates = search.filters_dates.borrow().clone();

    let query = draft.apply(&dates, &search.query().text);
    search.set_text(search_state::query_to_text(&query));
    search.close_filters();
    search.submit(scope, query);
}
