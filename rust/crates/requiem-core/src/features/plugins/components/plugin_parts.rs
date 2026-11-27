use std::rc::Rc;

use requiem_ui::colors::TEXT_SECONDARY;
use requiem_ui::components::{
    CheckboxProps, TextProps, TextVariant, TooltipPosition, TooltipProps, div, icon,
};
use requiem_ui::node::Node;
use requiem_ui::widget::anim::ToggleState;

use crate::plugins::PermissionInfo;
use crate::ui::classes::plugin_parts as cls;
use requiem_ui::icons::CIRCLE_INFORMATION;

pub fn label(permission: &str) -> String {
    if let Some(host) = permission.strip_prefix("network:") {
        return if host == "*" {
            requiem_i18n::t!("plugins-perm-network-any")
        } else {
            requiem_i18n::t!("plugins-perm-network-host", host = host)
        };
    }
    if let Some(route) = permission.strip_prefix("discord-api:") {
        return requiem_i18n::t!("plugins-perm-discord-api", route = route);
    }
    match permission {
        "messages.intercept-incoming" => requiem_i18n::t!("plugins-permission-intercept-incoming"),
        "messages.intercept-outgoing" => requiem_i18n::t!("plugins-permission-intercept-outgoing"),
        "guilds.read" => requiem_i18n::t!("plugins-perm-guilds-read"),
        "members.read" => requiem_i18n::t!("plugins-perm-members-read"),
        "presence.read" => requiem_i18n::t!("plugins-perm-presence-read"),
        "voice.read" => requiem_i18n::t!("plugins-perm-voice-read"),
        "relationships.read" => requiem_i18n::t!("plugins-perm-relationships-read"),
        "read-states.read" => requiem_i18n::t!("plugins-perm-read-states-read"),
        "read-states.write" => requiem_i18n::t!("plugins-perm-read-states-write"),
        "messages.read" => requiem_i18n::t!("plugins-perm-messages-read"),
        "attachments.read" => requiem_i18n::t!("plugins-perm-attachments-read"),
        "messages.send" => requiem_i18n::t!("plugins-perm-messages-send"),
        "messages.manage" => requiem_i18n::t!("plugins-perm-messages-manage"),
        "reactions" => requiem_i18n::t!("plugins-perm-reactions"),
        "composer" => requiem_i18n::t!("plugins-perm-composer"),
        "clipboard.write" => requiem_i18n::t!("plugins-perm-clipboard-write"),
        "clipboard.read" => requiem_i18n::t!("plugins-perm-clipboard-read"),
        "presence.write" => requiem_i18n::t!("plugins-perm-presence-write"),
        "voice.control" => requiem_i18n::t!("plugins-perm-voice-control"),
        "voice.moderate" => requiem_i18n::t!("plugins-perm-voice-moderate"),
        "notifications" => requiem_i18n::t!("plugins-perm-notifications"),
        "notifications.intercept" => requiem_i18n::t!("plugins-perm-notifications-intercept"),
        "audio.play" => requiem_i18n::t!("plugins-perm-audio-play"),
        "audio.replace" => requiem_i18n::t!("plugins-perm-audio-replace"),
        "audio.process" => requiem_i18n::t!("plugins-perm-audio-process"),
        "tweaks" => requiem_i18n::t!("plugins-perm-tweaks"),
        "files" => requiem_i18n::t!("plugins-perm-files"),
        "display.users" => requiem_i18n::t!("plugins-perm-display-users"),
        "display.channels" => requiem_i18n::t!("plugins-perm-display-channels"),
        "display.guilds" => requiem_i18n::t!("plugins-perm-display-guilds"),
        "ui.local-messages" => requiem_i18n::t!("plugins-perm-ui-local-messages"),
        "ui.modal" => requiem_i18n::t!("plugins-perm-ui-modal"),
        "ui.slots" => requiem_i18n::t!("plugins-perm-ui-slots"),
        "ui.pages" => requiem_i18n::t!("plugins-perm-ui-pages"),
        "ui.notices" => requiem_i18n::t!("plugins-perm-ui-notices"),
        "ui.overlay" => requiem_i18n::t!("plugins-perm-ui-overlay"),
        "data.raw" => requiem_i18n::t!("plugins-perm-data-raw"),
        "network.localhost" => requiem_i18n::t!("plugins-perm-network-localhost"),
        other => other.to_string(),
    }
}

pub fn row(
    permission: &PermissionInfo,
    state: ToggleState,
    on_change: Option<Rc<dyn Fn(bool)>>,
) -> Node {
    let disabled = on_change.is_none();
    let on_change = on_change.unwrap_or_else(|| Rc::new(|_| {}));
    let checkbox = CheckboxProps::new(state, move |on: bool| on_change(on))
        .label(label(&permission.name))
        .disabled(disabled)
        .build();
    let mut trailing = div().class(cls::TRAILING);
    if let Some(chip) = tier_chip(permission.tier) {
        trailing = trailing.child(chip);
    }
    if !permission.reason.trim().is_empty() {
        let info = icon(CIRCLE_INFORMATION, TEXT_SECONDARY).class(cls::INFO);
        trailing = trailing.child(
            TooltipProps::default()
                .text(permission.reason.clone())
                .position(TooltipPosition::Left)
                .build(info),
        );
    }
    div().class(cls::ROW).child(checkbox).child(trailing)
}

pub fn tier_chip(tier: &str) -> Option<Node> {
    let (text, color) = match tier {
        "sensitive" => (
            requiem_i18n::t!("plugins-tier-sensitive"),
            "var(--text-feedback-warning)",
        ),
        "dangerous" => (
            requiem_i18n::t!("plugins-tier-dangerous"),
            "var(--text-feedback-critical)",
        ),
        _ => return None,
    };
    Some(
        div().class(cls::CHIP).child(
            TextProps::styled(text, TextVariant::TEXT_XS_SEMIBOLD)
                .build()
                .style(format!("color: {color}")),
        ),
    )
}

pub fn tile(name: &str, large: bool) -> Node {
    let letter = name
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".to_string());
    let variant = if large {
        TextVariant::HEADING_LG_SEMIBOLD
    } else {
        TextVariant::TEXT_MD_SEMIBOLD
    };
    let mut tile = div().class(cls::TILE);
    if large {
        tile = tile.class(cls::TILE_LARGE);
    }
    tile.child(
        TextProps::styled(letter, variant)
            .build()
            .style("color: var(--text-strong)"),
    )
}

pub fn can_exfiltrate<'a>(granted: impl Iterator<Item = &'a str> + Clone) -> bool {
    granted
        .clone()
        .any(|p| p.starts_with("messages.read") || p.starts_with("messages.intercept"))
        && granted.clone().any(|p| p.starts_with("network:"))
}
