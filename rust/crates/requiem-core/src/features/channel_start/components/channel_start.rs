use crate::ui::classes::channel_start as cls;
use crate::ui::profile_popout;
use crate::ui;
use requiem_ui::components::TextVariant;
use requiem_ui::components::{MessageTextProps, TextProps, div, icon, message_text};
use requiem_ui::node::Node;
use skia_safe::font_style::Weight;

use crate::models::{Channel, Permissions, Role};
use crate::state::AppState;
use crate::ui::message_list::ChatColors;
use crate::ui::profile_popout::{DEFAULT_ROLE_COLOR, accent_to_color};

pub fn build(state: &AppState, channel: &Channel, colors: &ChatColors) -> Option<Node> {
    let guild_id = channel.guild_id?;
    if !matches!(channel.kind, 0 | 5) {
        return None;
    }
    let name = channel.name.as_deref().unwrap_or("unknown");
    let permissions = state.channel_permissions(channel);

    let show_private = permissions.contains(Permissions::MANAGE_ROLES)
        && state.permissions.role_required(guild_id, channel);

    let glyph = ui::channel_icon_for(channel, show_private);
    let circle = div()
        .class(cls::ICON_CIRCLE)
        .child(icon(glyph, colors.text_strong).class(cls::ICON));

    let heading = TextProps::new(
        requiem_i18n::t!("channel-start-welcome", name = name),
        32.0_f32,
    )
    .weight(Weight::BOLD)
    .color(colors.text_strong)
    .build()
    .class(cls::HEADER);

    let description = if show_private {
        requiem_i18n::md!("channel-start-description-private", name = name)
    } else {
        requiem_i18n::md!("channel-start-description", name = name)
    };
    let description = message_text(MessageTextProps {
        content: description,
        size: 16.0,
        color: colors.text_default,
        line_height: Some(22.0),
        ..Default::default()
    })
    .class(cls::DESCRIPTION);

    let mut root = div()
        .class(cls::CONTAINER)
        .child(circle)
        .child(heading)
        .child(description);
    if show_private {
        let roles = state.permissions.roles_that_can_view(guild_id, channel);
        if !roles.is_empty() {
            root = root.child(
                div().class(cls::ROLES).children(
                    roles
                        .iter()
                        .map(role_label)
                        .collect::<Vec<_>>(),
                ),
            );
        }
    }
    Some(root)
}

fn role_label(role: &Role) -> Node {
    let color = if role.color == 0 {
        DEFAULT_ROLE_COLOR
    } else {
        accent_to_color(role.color)
    };
    let dot =
        Node::new("div", profile_popout::ColorBoxWidget { color }).class(cls::ROLE_DOT);
    let label = TextProps::styled(role.name.clone(), TextVariant::TEXT_XS_NORMAL)
        .color(ui::TEXT_DEFAULT)
        .line_clamp(1_u32)
        .build();
    div().class(cls::ROLE).child(dot).child(label)
}
