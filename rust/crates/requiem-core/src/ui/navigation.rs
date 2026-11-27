use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use crate::models::id::marker::{ChannelMarker, GuildMarker};
use crate::models::{Channel, Id, Permissions};
use crate::state::AppState;

type ChannelId = Id<ChannelMarker>;
type GuildId = Id<GuildMarker>;

thread_local! {

    static LAST_CHANNEL: RefCell<HashMap<GuildId, ChannelId>> = RefCell::new(HashMap::new());

    static PREVIOUS_GUILD: Cell<Option<GuildId>> = const { Cell::new(None) };
    static CURRENT_GUILD: Cell<Option<GuildId>> = const { Cell::new(None) };
}

pub fn note_selection(state: &AppState, channel_id: ChannelId) {
    let guild = super::find_channel(state, channel_id).map(|(guild_id, _)| guild_id);
    if let Some(guild_id) = guild {
        LAST_CHANNEL.with(|m| m.borrow_mut().insert(guild_id, channel_id));
    }
    if CURRENT_GUILD.with(Cell::get) != guild {
        PREVIOUS_GUILD.with(|p| p.set(CURRENT_GUILD.with(Cell::get)));
        CURRENT_GUILD.with(|c| c.set(guild));
    }
}

pub fn note_home() {
    if CURRENT_GUILD.with(Cell::get).is_some() {
        PREVIOUS_GUILD.with(|p| p.set(CURRENT_GUILD.with(Cell::get)));
        CURRENT_GUILD.with(|c| c.set(None));
    }
}

pub fn reset() {
    LAST_CHANNEL.with(|m| m.borrow_mut().clear());
    PREVIOUS_GUILD.with(|p| p.set(None));
    CURRENT_GUILD.with(|c| c.set(None));
}

pub fn previous_guild() -> Option<GuildId> {
    PREVIOUS_GUILD.with(Cell::get)
}

pub fn channel_for_guild(state: &AppState, guild_id: GuildId) -> Option<ChannelId> {
    let remembered = LAST_CHANNEL.with(|m| m.borrow().get(&guild_id).copied());
    remembered
        .filter(|id| {
            super::find_channel(state, *id)
                .is_some_and(|(_, c)| state.can(&c, Permissions::VIEW_CHANNEL))
        })
        .or_else(|| super::default_channel(state, Some(guild_id)).map(|c| c.id))
}

pub fn guild_ids(state: &AppState) -> Vec<GuildId> {
    state
        .guilds
        .list
        .with_untracked(|guilds| guilds.iter().map(|g| g.id).collect())
}

pub fn guild_channels(state: &AppState, guild_id: GuildId) -> Vec<ChannelId> {
    state.channels.by_guild.with_untracked(|by_guild| {
        let Some(channels) = by_guild.get(&guild_id) else {
            return Vec::new();
        };
        let viewable = |c: &&Channel| state.can(c, Permissions::VIEW_CHANNEL);
        let mut loose: Vec<&Channel> = channels
            .iter()
            .filter(|c| c.kind != 4 && c.parent_id.is_none())
            .filter(viewable)
            .collect();
        loose.sort_by_key(|c| c.position.unwrap_or(0));
        let mut categories: Vec<&Channel> = channels.iter().filter(|c| c.kind == 4).collect();
        categories.sort_by_key(|c| c.position.unwrap_or(0));
        let mut ordered = loose;
        for category in categories {
            let mut children: Vec<&Channel> = channels
                .iter()
                .filter(|c| c.parent_id == Some(category.id))
                .filter(viewable)
                .collect();
            children.sort_by_key(|c| c.position.unwrap_or(0));
            ordered.extend(children);
        }
        ordered
            .into_iter()
            .filter(|c| matches!(c.kind, 0 | 5))
            .map(|c| c.id)
            .collect()
    })
}

pub fn dm_channels(state: &AppState) -> Vec<ChannelId> {
    state
        .dms
        .list
        .with_untracked(|dms| dms.iter().map(|d| d.id).collect())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Home,
    Guild(GuildId),
}

pub fn place(state: &AppState, selected: Option<ChannelId>) -> Place {
    match super::resolve_selected_guild(state, selected) {
        Some(guild_id) => Place::Guild(guild_id),
        None => Place::Home,
    }
}

fn channels_of(state: &AppState, place: Place) -> Vec<ChannelId> {
    match place {
        Place::Home => dm_channels(state),
        Place::Guild(guild_id) => guild_channels(state, guild_id),
    }
}

pub fn step_channel(
    state: &AppState,
    selected: Option<ChannelId>,
    step: isize,
) -> Option<ChannelId> {
    let list = channels_of(state, place(state, selected));
    step_in(&list, selected, step)
}

fn step_in(list: &[ChannelId], current: Option<ChannelId>, step: isize) -> Option<ChannelId> {
    if list.is_empty() {
        return None;
    }
    let len = list.len() as isize;
    let target = match current.and_then(|c| list.iter().position(|id| *id == c)) {
        Some(index) => (index as isize + step).rem_euclid(len),
        None if step > 0 => 0,
        None => len - 1,
    };
    list.get(target as usize).copied()
}

fn everything(state: &AppState) -> Vec<ChannelId> {
    let mut all = dm_channels(state);
    for guild_id in guild_ids(state) {
        all.extend(guild_channels(state, guild_id));
    }
    all
}

pub fn find_channel_where(
    state: &AppState,
    selected: Option<ChannelId>,
    step: isize,
    wanted: impl Fn(ChannelId) -> bool,
) -> Option<ChannelId> {
    let all = everything(state);
    if all.is_empty() {
        return None;
    }
    let len = all.len() as isize;
    let start = selected
        .and_then(|c| all.iter().position(|id| *id == c))
        .map_or(if step > 0 { -1 } else { len }, |i| i as isize);
    (1..=len)
        .map(|offset| all[(start + step * offset).rem_euclid(len) as usize])
        .find(|id| wanted(*id) && Some(*id) != selected)
}

pub fn next_unread(
    state: &AppState,
    selected: Option<ChannelId>,
    step: isize,
) -> Option<ChannelId> {
    find_channel_where(state, selected, step, |id| {
        let badge = state.read_states.badge(id).peek();
        badge.unread || badge.mentions > 0
    })
}

pub fn next_mention(
    state: &AppState,
    selected: Option<ChannelId>,
    step: isize,
) -> Option<ChannelId> {
    find_channel_where(state, selected, step, |id| {
        state.read_states.mention_count(id) > 0
    })
}

pub fn step_guild(state: &AppState, selected: Option<ChannelId>, step: isize) -> Option<GuildId> {
    let guilds = guild_ids(state);
    let slots = guilds.len() as isize + 1;
    let current = match place(state, selected) {
        Place::Home => 0,
        Place::Guild(id) => guilds
            .iter()
            .position(|g| *g == id)
            .map_or(0, |i| i as isize + 1),
    };
    let target = (current + step).rem_euclid(slots);
    (target > 0).then(|| guilds[target as usize - 1])
}

pub fn guild_slot(state: &AppState, n: usize) -> Result<Option<GuildId>, ()> {
    match n {
        0 => Err(()),
        1 => Ok(None),
        n => guild_ids(state).get(n - 2).copied().map(Some).ok_or(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(n: &[u64]) -> Vec<ChannelId> {
        n.iter().map(|n| Id::new(*n)).collect()
    }

    #[test]
    fn stepping_wraps_both_ways() {
        let list = ids(&[1, 2, 3]);
        assert_eq!(step_in(&list, Some(Id::new(3)), 1), Some(Id::new(1)));
        assert_eq!(step_in(&list, Some(Id::new(1)), -1), Some(Id::new(3)));
        assert_eq!(step_in(&list, Some(Id::new(2)), 1), Some(Id::new(3)));
    }

    #[test]
    fn stepping_from_nothing_starts_at_an_end() {
        let list = ids(&[1, 2, 3]);
        assert_eq!(step_in(&list, None, 1), Some(Id::new(1)));
        assert_eq!(step_in(&list, None, -1), Some(Id::new(3)));
        assert_eq!(step_in(&[], None, 1), None);
    }
}
