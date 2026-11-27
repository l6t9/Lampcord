use crate::plugins;
use crate::ui::plugin_view;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use requiem_ui::components::{
    Align, ButtonProps, ButtonSize, ButtonVariant, HeadingLevel, HeadingProps, Justify, StackProps,
    SwitchProps, TabBarProps, TabItem, TextProps, TextVariant, div,
};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::anim::{ToggleMemo, ToggleState};

use crate::plugins::{PluginInfo, PluginStatus};
use crate::state::AppState;
use crate::ui::classes::plugins_settings as cls;
use crate::ui::plugin_parts;

const TAB_PERMISSIONS: &str = "permissions";
const TAB_SETTINGS: &str = "settings";

thread_local! {

    static REQUESTED: RefCell<HashSet<String>> = RefCell::new(HashSet::new());

    static TOGGLES: RefCell<HashMap<String, ToggleMemo>> = RefCell::new(HashMap::new());

    static TABS: Signal<HashMap<String, String>> = signal(HashMap::new());
}

pub fn toggle(key: String, on: bool) -> ToggleState {
    TOGGLES.with(|toggles| {
        toggles
            .borrow_mut()
            .entry(key)
            .or_insert_with(|| ToggleMemo::new(on))
            .track(on)
    })
}

pub fn build_page(state: &AppState, on_install: Option<Rc<dyn Fn()>>) -> Node {
    REQUESTED.with(|requested| requested.borrow_mut().clear());
    state.plugins.refresh();
    let state = state.clone();
    let tabs = TABS.with(|tabs| tabs.clone());
    Node::reactive(move || {
        let plugins = state.plugins.list.get();
        let selected = tabs.get();
        let mut page = div().class(cls::PAGE);
        let mut intro = StackProps::horizontal()
            .justify(Justify::SpaceBetween)
            .align(Align::Center)
            .gap(16)
            .build()
            .child(
                TextProps::styled(
                    requiem_i18n::t!("plugins-intro"),
                    TextVariant::TEXT_SM_NORMAL,
                )
                .build()
                .style("color: var(--text-muted); flex-shrink: 1"),
            );
        if let Some(on_install) = &on_install {
            let on_install = on_install.clone();
            intro = intro.child(
                ButtonProps::new(
                    ButtonVariant::Primary,
                    requiem_i18n::t!("plugins-install-folder"),
                    move || on_install(),
                )
                .size(ButtonSize::Small)
                .build(),
            );
        }
        page = page.child(intro);
        if plugins.is_empty() {
            return page.child(
                TextProps::styled(
                    requiem_i18n::t!("plugins-none-installed"),
                    TextVariant::TEXT_MD_NORMAL,
                )
                .build()
                .style("color: var(--text-muted)"),
            );
        }
        let views = state.plugins.settings_views.get();
        for plugin in plugins.iter() {
            page = page.child(plugin_card(
                plugin,
                selected.get(&plugin.id).map(String::as_str),
                views.get(&plugin.id),
                &tabs,
            ));
        }
        page
    })
}

fn plugin_card(
    plugin: &PluginInfo,
    selected: Option<&str>,
    view: Option<&Result<plugins::ViewTree, String>>,
    tabs: &Signal<HashMap<String, String>>,
) -> Node {
    let id = plugin.id.clone();
    let has_page = plugin.enabled && plugin.has_settings;
    let tab = match selected {
        Some(TAB_SETTINGS) if has_page => TAB_SETTINGS,
        Some(TAB_PERMISSIONS) => TAB_PERMISSIONS,
        _ if has_page => TAB_SETTINGS,
        _ => TAB_PERMISSIONS,
    };

    let enabled_switch = {
        let id = id.clone();
        SwitchProps::new(
            toggle(format!("{id}/enabled"), plugin.enabled),
            move |on: bool| plugins::set_enabled(&id, on),
        )
        .build()
    };
    let mut name_row = StackProps::horizontal()
        .gap(8)
        .align(Align::Baseline)
        .build()
        .children([
            HeadingProps::new(HeadingLevel::H3, plugin.name.clone())
                .variant(TextVariant::HEADING_MD_SEMIBOLD)
                .build(),
            TextProps::styled(format!("v{}", plugin.version), TextVariant::TEXT_XS_NORMAL)
                .build()
                .style("color: var(--text-muted)"),
        ]);
    if let Some(chip) = highest_tier(plugin).and_then(plugin_parts::tier_chip) {
        name_row = name_row.child(chip);
    }
    let mut titles = div().class(cls::TITLES).child(name_row);
    if !plugin.authors.is_empty() {
        titles = titles.child(
            TextProps::styled(
                requiem_i18n::t!("plugins-by", authors = plugin.authors.join(", ")),
                TextVariant::TEXT_XS_NORMAL,
            )
            .build()
            .style("color: var(--text-muted)"),
        );
    }
    let header = div()
        .class(cls::CARD_HEADER)
        .child(plugin_parts::tile(&plugin.name, false))
        .child(titles)
        .child(enabled_switch);

    let mut card = div().class(cls::CARD).child(header);
    if !plugin.description.is_empty() {
        card = card.child(
            TextProps::styled(plugin.description.clone(), TextVariant::TEXT_SM_NORMAL)
                .build()
                .style("color: var(--text-default)"),
        );
    }
    let problem = match &plugin.status {
        PluginStatus::Failed(error) => Some(requiem_i18n::t!(
            "plugins-status-failed",
            error = error.clone()
        )),
        PluginStatus::Suspended(error) => Some(requiem_i18n::t!(
            "plugins-status-suspended",
            error = error.clone()
        )),
        _ => None,
    };
    if let Some(problem) = problem {
        card = card.child(
            TextProps::styled(problem, TextVariant::TEXT_SM_MEDIUM)
                .build()
                .style("color: var(--text-feedback-critical)"),
        );
    }

    let mut items = vec![TabItem::new(
        TAB_PERMISSIONS,
        requiem_i18n::t!("plugins-tab-permissions"),
    )];
    if has_page {
        items.push(TabItem::new(
            TAB_SETTINGS,
            requiem_i18n::t!("plugins-tab-settings"),
        ));
    }
    let tab_bar = {
        let (tabs, id) = (tabs.clone(), id.clone());
        TabBarProps::default()
            .items(items)
            .selected(tab.to_string())
            .on_select(Rc::new(move |tab: String| {
                let id = id.clone();
                tabs.update(|tabs| {
                    tabs.insert(id, tab);
                })
            }) as Rc<dyn Fn(String)>)
            .build()
    };
    card = card.child(tab_bar);

    let content = if tab == TAB_SETTINGS {
        settings_tab(&id, view)
    } else {
        permissions_tab(plugin)
    };
    card = card.child(div().class(cls::PANEL).child(content));

    let remove = {
        let id = id.clone();
        ButtonProps::new(
            ButtonVariant::CriticalSecondary,
            requiem_i18n::t!("plugins-remove"),
            move || plugins::remove(&id),
        )
        .size(ButtonSize::Small)
        .build()
    };
    card.child(div().class(cls::CARD_FOOTER).child(remove))
}

fn highest_tier(plugin: &PluginInfo) -> Option<&'static str> {
    let tiers = || plugin.permissions.iter().map(|p| p.tier);
    ["dangerous", "sensitive"]
        .into_iter()
        .find(|tier| tiers().any(|t| t == *tier))
}

fn permissions_tab(plugin: &PluginInfo) -> Node {
    let muted = |text: String| {
        TextProps::styled(text, TextVariant::TEXT_SM_NORMAL)
            .build()
            .style("color: var(--text-muted)")
    };
    let eyebrow = |text: String| {
        TextProps::styled(text, TextVariant::EYEBROW)
            .build()
            .style("color: var(--text-muted)")
    };
    let required: Vec<_> = plugin.permissions.iter().filter(|p| p.required).collect();
    let optional: Vec<_> = plugin.permissions.iter().filter(|p| !p.required).collect();
    let mut panel = div().class(cls::GROUPS);
    if required.is_empty() && optional.is_empty() {
        return panel.child(muted(requiem_i18n::t!("plugins-permissions-none")));
    }
    panel = panel.child(muted(requiem_i18n::t!(
        "plugins-permissions-summary",
        required = required.len(),
        optional = optional.len()
    )));
    if !required.is_empty() {
        let mut group = div()
            .class(cls::GROUP)
            .child(eyebrow(requiem_i18n::t!("plugins-permissions-required")));
        for permission in required {
            group = group.child(plugin_parts::row(permission, ToggleState::new(true), None));
        }
        panel = panel.child(group);
    }
    if !optional.is_empty() {
        let mut group = div()
            .class(cls::GROUP)
            .child(eyebrow(requiem_i18n::t!("plugins-permissions-optional")));
        for permission in optional {
            let (id, name) = (plugin.id.clone(), permission.name.clone());
            let state = toggle(format!("{id}/{name}"), permission.granted);
            let on_change: Rc<dyn Fn(bool)> =
                Rc::new(move |on| plugins::set_permission(&id, &name, on));
            group = group.child(plugin_parts::row(permission, state, Some(on_change)));
        }
        panel = panel.child(group);
    }
    panel
}

fn settings_tab(id: &str, view: Option<&Result<plugins::ViewTree, String>>) -> Node {
    match view {
        Some(Ok(tree)) => {
            let plugin_id = id.to_string();
            let on_action: plugin_view::OnAction =
                Rc::new(move |action, value, rerender| {
                    plugins::send_ui_action(&plugin_id, action, value, rerender)
                });
            plugin_view::render(id, tree, on_action, None)
        }
        Some(Err(error)) => TextProps::styled(
            requiem_i18n::t!("plugins-settings-failed", error = error.clone()),
            TextVariant::TEXT_SM_NORMAL,
        )
        .build()
        .style("color: var(--text-feedback-critical)"),
        None => {
            if REQUESTED.with(|requested| requested.borrow_mut().insert(id.to_string())) {
                plugins::request_settings_view(id);
            }
            TextProps::styled(
                requiem_i18n::t!("plugins-settings-loading"),
                TextVariant::TEXT_SM_NORMAL,
            )
            .build()
            .style("color: var(--text-muted)")
        }
    }
}
