use crate::net_profile;
use requiem_ui::signal::{Effect, Signal, create_effect};
use requiem_ui::widget::WidgetTree;

use crate::models::Id;
use crate::models::id::marker::ChannelMarker;
use crate::state::AppState;

pub fn track_displayed_channel(
    tree: &mut WidgetTree,
    state: &AppState,
    selected_channel: Signal<Option<Id<ChannelMarker>>>,
) -> Effect {
    let state = state.clone();
    create_effect(
        move |_| {
            let channel = super::displayed_channel(&state, selected_channel.get());

            net_profile::set_route(&match &channel {
                Some(channel) => match channel.guild_id {
                    Some(guild) => format!("/channels/{guild}/{}", channel.id),
                    None => format!("/channels/@me/{}", channel.id),
                },
                None => "/channels/@me".to_string(),
            });
            state
                .read_states
                .select_channel(channel.map(|channel| channel.id));
        },
        tree,
    )
}
