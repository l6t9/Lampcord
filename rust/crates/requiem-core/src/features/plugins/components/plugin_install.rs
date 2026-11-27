use crate::plugins;
use crate::ui::plugins_settings;
use std::collections::BTreeSet;
use std::rc::Rc;

use requiem_ui::components::{
    ButtonProps, ButtonVariant, HeadingLevel, HeadingProps, ModalProps, ModalSize, TextProps,
    TextVariant, div,
};
use requiem_ui::node::Node;
use requiem_ui::signal::Signal;
use requiem_ui::widget::anim::ToggleState;

use crate::plugins::PluginPreview;
use crate::ui::classes::plugin_install as cls;
use crate::ui::plugin_parts;

#[derive(Clone, Debug)]
pub struct PluginInstallState {
    pub preview: Option<PluginPreview>,

    pub chosen: BTreeSet<String>,
    pub open: ToggleState,
}

impl Default for PluginInstallState {
    fn default() -> Self {
        Self {
            preview: None,
            chosen: BTreeSet::new(),
            open: ToggleState::new(false),
        }
    }
}

pub fn open(signal: &Signal<PluginInstallState>, preview: PluginPreview) {
    let chosen = preview
        .permissions
        .iter()
        .filter(|p| !p.required && (p.granted || p.tier == "normal"))
        .map(|p| p.name.clone())
        .collect();
    signal.update(|state| {
        state.preview = Some(preview);
        state.chosen = chosen;
        state.open = state.open.set(true);
    });
}

pub fn close(signal: &Signal<PluginInstallState>) {
    signal.update(|state| state.open = state.open.set(false));
}

pub fn build(state: &PluginInstallState, signal: &Signal<PluginInstallState>) -> Option<Node> {
    let preview = state.preview.clone()?;
    let update = preview.installed_version.is_some();
    let subtle = |text: String, variant: TextVariant, color: &str| {
        TextProps::styled(text, variant)
            .build()
            .style(format!("color: {color}"))
    };

    let title = if update {
        requiem_i18n::t!("plugins-update-title", name = preview.name.clone())
    } else {
        requiem_i18n::t!("plugins-install-title", name = preview.name.clone())
    };
    let mut by = format!("v{}", preview.version);
    if !preview.authors.is_empty() {
        by = format!(
            "{by} · {}",
            requiem_i18n::t!("plugins-by", authors = preview.authors.join(", "))
        );
    }
    let mut hero = div()
        .class(cls::HERO)
        .child(plugin_parts::tile(&preview.name, true).class(cls::HERO_TILE))
        .child(
            HeadingProps::new(HeadingLevel::H1, title)
                .variant(TextVariant::HEADING_LG_SEMIBOLD)
                .build(),
        )
        .child(subtle(by, TextVariant::TEXT_SM_NORMAL, "var(--text-muted)"));
    if !preview.description.is_empty() {
        hero = hero.child(subtle(
            preview.description.clone(),
            TextVariant::TEXT_SM_NORMAL,
            "var(--text-default)",
        ));
    }

    let required: Vec<_> = preview.permissions.iter().filter(|p| p.required).collect();
    let optional: Vec<_> = preview.permissions.iter().filter(|p| !p.required).collect();

    let mut panel = div().class(cls::PANEL);
    if required.is_empty() && optional.is_empty() {
        panel = panel.child(subtle(
            requiem_i18n::t!("plugins-permissions-none"),
            TextVariant::TEXT_SM_NORMAL,
            "var(--text-muted)",
        ));
    }
    if !required.is_empty() {
        let mut group = div().class(cls::GROUP).child(subtle(
            requiem_i18n::t!("plugins-install-lead", name = preview.name.clone()),
            TextVariant::TEXT_SM_SEMIBOLD,
            "var(--text-strong)",
        ));
        for permission in &required {
            group = group.child(plugin_parts::row(permission, ToggleState::new(true), None));
        }
        panel = panel.child(group);
    }
    if !optional.is_empty() {
        if !required.is_empty() {
            panel = panel.child(div().class(cls::DIVIDER));
        }
        let mut group = div().class(cls::GROUP).child(subtle(
            requiem_i18n::t!("plugins-install-optional-lead"),
            TextVariant::TEXT_SM_SEMIBOLD,
            "var(--text-strong)",
        ));
        for permission in &optional {
            let on = state.chosen.contains(&permission.name);
            let name = permission.name.clone();
            let signal = signal.clone();
            let toggle =
                plugins_settings::toggle(format!("install/{}/{}", preview.id, name), on);
            let on_change: Rc<dyn Fn(bool)> = Rc::new(move |on| {
                signal.update(|state| {
                    if on {
                        state.chosen.insert(name.clone());
                    } else {
                        state.chosen.remove(&name);
                    }
                });
            });
            group = group.child(plugin_parts::row(permission, toggle, Some(on_change)));
        }
        panel = panel.child(group);
    }

    let granted = preview
        .permissions
        .iter()
        .filter(|p| p.required || state.chosen.contains(&p.name))
        .map(|p| p.name.as_str());
    let mut body = div().class(cls::DIALOG).child(hero).child(panel);
    if plugin_parts::can_exfiltrate(granted) {
        body = body.child(subtle(
            requiem_i18n::t!("plugins-install-warning"),
            TextVariant::TEXT_SM_MEDIUM,
            "var(--text-feedback-warning)",
        ));
    }
    body = body.child(subtle(
        requiem_i18n::t!("plugins-install-sandbox"),
        TextVariant::TEXT_XS_NORMAL,
        "var(--text-muted)",
    ));

    let cancel = {
        let signal = signal.clone();
        ButtonProps::new(
            ButtonVariant::Secondary,
            requiem_i18n::t!("plugins-install-cancel"),
            move || close(&signal),
        )
        .build()
    };
    let confirm = {
        let (signal, dir, chosen) = (
            signal.clone(),
            preview.dir.clone(),
            state.chosen.iter().cloned().collect::<Vec<_>>(),
        );
        let label = if update {
            requiem_i18n::t!("plugins-update-confirm")
        } else {
            requiem_i18n::t!("plugins-install-confirm")
        };
        ButtonProps::new(ButtonVariant::Primary, label, move || {
            plugins::install_dir(dir.clone(), chosen.clone());
            close(&signal);
        })
        .build()
    };
    let dismiss = {
        let signal = signal.clone();
        move || close(&signal)
    };
    Some(
        ModalProps::default()
            .size(ModalSize::Small)
            .header(false)
            .dismissable(true)
            .on_dismiss(dismiss)
            .open(state.open)
            .body(body)
            .footer(vec![cancel, confirm])
            .build(),
    )
}
