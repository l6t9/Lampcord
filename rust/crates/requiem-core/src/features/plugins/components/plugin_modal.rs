use crate::plugins;
use crate::ui::full_profile;
use std::rc::Rc;

use requiem_ui::components::{ModalProps, ModalSize, TextProps, TextVariant, div};
use requiem_ui::node::Node;

use crate::state::{AppState, PLUGIN_MODAL_CONTEXT, PluginSurface};
use crate::ui::plugin_view::{self, ViewEnv};

pub fn current(state: &AppState) -> Option<PluginSurface> {
    state.plugins.surfaces.with(|surfaces| {
        surfaces
            .iter()
            .rev()
            .find(|s| s.open.on)
            .or_else(|| surfaces.last())
            .cloned()
    })
}

fn modal_size(name: &str) -> ModalSize {
    match name {
        "medium" => ModalSize::Medium,
        "large" => ModalSize::Large,
        "small" => ModalSize::Small,
        _ => ModalSize::Bare,
    }
}

pub fn build(surface: &PluginSurface, state: &AppState, env: &ViewEnv) -> Node {
    let dismiss: Rc<dyn Fn()> = {
        let (plugins, plugin, view_id) = (
            state.plugins.clone(),
            surface.plugin.clone(),
            surface.view_id.clone(),
        );
        Rc::new(move || plugins.close_surface(&plugin, &view_id))
    };
    let body = match &surface.view {
        None => TextProps::styled(
            requiem_i18n::t!("plugins-settings-loading"),
            TextVariant::TEXT_SM_NORMAL,
        )
        .build()
        .style("color: var(--text-muted)"),
        Some(Err(error)) => TextProps::styled(
            requiem_i18n::t!("plugins-settings-failed", error = error.clone()),
            TextVariant::TEXT_SM_NORMAL,
        )
        .build()
        .style("color: var(--text-feedback-critical)"),
        Some(Ok(tree)) => {
            let (plugin, view_id) = (surface.plugin.clone(), surface.view_id.clone());
            let on_action: plugin_view::OnAction = Rc::new(move |action, value, _| {
                plugins::view_action(&plugin, &view_id, action, value, PLUGIN_MODAL_CONTEXT)
            });
            plugin_view::render(&surface.plugin, tree, on_action, Some(env))
        }
    };

    if surface.size == "profile" {
        let content =
            body.style("flex-grow: 1; min-height: 0px; display: flex; flex-direction: column");
        return full_profile::plugin_profile_modal(content, surface.open, dismiss);
    }
    let on_dismiss = dismiss.clone();
    ModalProps::default()
        .size(modal_size(&surface.size))
        .header(false)
        .dismissable(true)
        .on_dismiss(move || on_dismiss())
        .open(surface.open)
        .body(div().child(body))
        .build()
}
