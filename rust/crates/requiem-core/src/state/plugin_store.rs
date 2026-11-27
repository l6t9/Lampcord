use crate::models;
use crate::plugins;
use crate::state::member_list::MemberListRow;
use crate::ui::context_menu;
use requiem_ui::widget::primitives::PresenceStatus;
use std::cell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use requiem_ui::signal::{Signal, signal};

use super::{Action, AppState, GuildData, Store};
use crate::models::id::marker::{ChannelMarker, GuildMarker, MessageMarker};
use crate::models::{Channel, Guild, Id, Role};
use crate::plugins::{
    ByPlugin, GuildSnapshot, IncomingOutcome, PluginEvent, PluginInfo, Registrations,
    SurfaceRequest,
};
use requiem_ui::widget::anim::ToggleState;

#[derive(Clone, Debug)]
pub struct PluginSurface {
    pub plugin: String,
    pub view_id: String,
    pub title: String,

    pub size: String,

    pub view: Option<Result<plugins::ViewTree, String>>,
    pub open: ToggleState,
}

pub const MODAL_CONTEXT: &str = r#"{"surface":"modal"}"#;

#[derive(Clone, Debug)]
pub struct ComposerRestore {
    pub channel_id: Id<ChannelMarker>,
    pub content: String,
}

#[derive(Clone)]
pub struct PluginStore {
    pub overlays: Signal<Rc<HashMap<Id<MessageMarker>, IncomingOutcome>>>,

    pub show_original: Signal<HashSet<Id<MessageMarker>>>,
    pub list: Signal<Vec<PluginInfo>>,

    pub settings_views: Signal<HashMap<String, Result<plugins::ViewTree, String>>>,

    pub styles_revision: Signal<u64>,

    pub registrations: Signal<Rc<Vec<(String, Registrations)>>>,

    pub surfaces: Signal<Vec<PluginSurface>>,
    last_stylesheet: Rc<cell::RefCell<String>>,
}

impl Default for PluginStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginStore {
    pub fn new() -> Self {
        let list = plugins::host()
            .map(|host| host.list())
            .unwrap_or_default();
        Self {
            overlays: signal(Rc::new(HashMap::new())),
            show_original: signal(HashSet::new()),
            list: signal(list),
            settings_views: signal(HashMap::new()),
            styles_revision: signal(0),
            registrations: signal(Rc::new(Vec::new())),
            surfaces: signal(Vec::new()),
            last_stylesheet: Rc::default(),
        }
    }

    pub fn name_of(&self, id: &str) -> String {
        self.list
            .with_untracked(|list| list.iter().find(|p| p.id == id).map(|p| p.name.clone()))
            .unwrap_or_else(|| id.to_string())
    }

    pub fn refresh(&self) {
        let Some(host) = plugins::host() else {
            return;
        };
        let list = host.list();
        let enabled: HashSet<String> = list
            .iter()
            .filter(|p| p.enabled)
            .map(|p| p.id.clone())
            .collect();

        let stale = self.overlays.with_untracked(|o| {
            o.values()
                .any(|outcome| plugins_of(outcome).any(|id| !enabled.contains(id)))
        });
        if stale {
            self.overlays.update(|overlays| {
                let overlays = Rc::make_mut(overlays);
                overlays
                    .values_mut()
                    .for_each(|outcome| retain_plugins(outcome, |id| enabled.contains(id)));
                overlays.retain(|_, outcome| !outcome.is_unchanged());
            });
        }
        self.list.set(list);
        self.registrations.set(Rc::new(host.registrations()));
        let stylesheet = host.stylesheet();
        if *self.last_stylesheet.borrow() != stylesheet {
            *self.last_stylesheet.borrow_mut() = stylesheet;
            self.styles_revision.update(|n| *n += 1);
        }
    }

    pub fn close_surface(&self, plugin: &str, view_id: &str) {
        self.surfaces.update(|surfaces| {
            if let Some(surface) = surfaces
                .iter_mut()
                .find(|s| s.plugin == plugin && s.view_id == view_id)
            {
                surface.open = surface.open.set(false);
            }
        });
    }

    pub fn close_top_surface(&self) -> bool {
        let top = self.surfaces.with_untracked(|surfaces| {
            surfaces
                .iter()
                .rev()
                .find(|s| s.open.on)
                .map(|s| (s.plugin.clone(), s.view_id.clone()))
        });
        match top {
            Some((plugin, view_id)) => {
                self.close_surface(&plugin, &view_id);
                true
            }
            None => false,
        }
    }

    pub fn toggle_original(&self, message_id: Id<MessageMarker>) {
        self.show_original.update(|shown| {
            if !shown.remove(&message_id) {
                shown.insert(message_id);
            }
        });
    }

    pub fn apply_events(&self, events: Vec<PluginEvent>) -> Vec<ComposerRestore> {
        let mut restores = Vec::new();
        for event in events {
            match event {
                PluginEvent::Overlays(changed) => self.overlays.update(|overlays| {
                    let overlays = Rc::make_mut(overlays);
                    for (id, outcome) in changed {
                        overlays.insert(id, outcome);
                    }
                }),
                PluginEvent::OverlayUpdate { plugin, update } => self.overlays.update(|overlays| {
                    let overlays = Rc::make_mut(overlays);
                    match update {
                        requiem_plugin_host::OverlayUpdate::DisplayContent {
                            message_id,
                            content,
                            ..
                        } => {
                            let id = Id::new(message_id.get());
                            let outcome = overlays.entry(id).or_default();
                            match content {
                                Some(value) => {
                                    outcome.display_content = Some(ByPlugin { plugin, value })
                                }
                                None if outcome
                                    .display_content
                                    .as_ref()
                                    .is_some_and(|d| d.plugin == plugin) =>
                                {
                                    outcome.display_content = None
                                }
                                None => {}
                            }
                            if outcome.is_unchanged() {
                                overlays.remove(&id);
                            }
                        }
                        requiem_plugin_host::OverlayUpdate::Annotation {
                            message_id, text, ..
                        } => {
                            let id = Id::new(message_id.get());
                            let outcome = overlays.entry(id).or_default();
                            outcome.annotations.retain(|a| a.plugin != plugin);
                            if let Some(value) = text {
                                outcome.annotations.push(ByPlugin { plugin, value });
                            }
                            if outcome.is_unchanged() {
                                overlays.remove(&id);
                            }
                        }
                        requiem_plugin_host::OverlayUpdate::Hidden {
                            message_id, hidden, ..
                        } => {
                            let id = Id::new(message_id.get());
                            let outcome = overlays.entry(id).or_default();
                            if hidden {
                                outcome.hidden = Some(ByPlugin {
                                    plugin,
                                    value: String::new(),
                                });
                            } else if outcome.hidden.as_ref().is_some_and(|h| h.plugin == plugin) {
                                outcome.hidden = None;
                            }
                            if outcome.is_unchanged() {
                                overlays.remove(&id);
                            }
                        }
                        requiem_plugin_host::OverlayUpdate::Clear { message_id, .. } => {
                            let id = Id::new(message_id.get());
                            if let Some(outcome) = overlays.get_mut(&id) {
                                retain_plugins(outcome, |p| p != plugin);
                                if outcome.is_unchanged() {
                                    overlays.remove(&id);
                                }
                            }
                        }

                        _ => {}
                    }
                }),
                PluginEvent::StatusChanged => self.refresh(),
                PluginEvent::SendCancelled {
                    channel_id,
                    content,
                    plugin,
                    reason,
                } => {
                    context_menu::notify_failure(&requiem_i18n::t!(
                        "plugins-send-cancelled",
                        plugin = self.name_of(&plugin),
                        reason = reason
                    ));
                    restores.push(ComposerRestore {
                        channel_id,
                        content,
                    });
                }
                PluginEvent::SettingsView { plugin, view } => self.settings_views.update(|views| {
                    views.insert(plugin, view);
                }),
                PluginEvent::InstallFailed(error) => context_menu::notify_failure(
                    &requiem_i18n::t!("plugins-install-failed", error = error),
                ),
                PluginEvent::Toast { text, kind } => {
                    if kind == "failure" {
                        context_menu::notify_failure(&text);
                    } else {
                        context_menu::notify(&text);
                    }
                }
                PluginEvent::Surface(SurfaceRequest::OpenModal {
                    plugin,
                    view_id,
                    title,
                    size,
                }) => {
                    self.surfaces.update(|surfaces| {
                        surfaces.retain(|s| !(s.plugin == plugin && s.view_id == view_id));
                        surfaces.push(PluginSurface {
                            plugin: plugin.clone(),
                            view_id: view_id.clone(),
                            title,
                            size,
                            view: None,
                            open: ToggleState::new(false).set(true),
                        });
                    });
                    plugins::render_view(&plugin, &view_id, MODAL_CONTEXT);
                }
                PluginEvent::Surface(SurfaceRequest::CloseModal { plugin, view_id }) => {
                    self.close_surface(&plugin, &view_id)
                }
                PluginEvent::ViewRendered {
                    plugin,
                    view_id,
                    view,
                } => self.surfaces.update(|surfaces| {
                    if let Some(surface) = surfaces
                        .iter_mut()
                        .find(|s| s.plugin == plugin && s.view_id == view_id)
                    {
                        surface.view = Some(view);
                    }
                }),
                PluginEvent::OpenMedia { url } => {
                    (context_menu::actions().open_media)(url)
                }
                PluginEvent::OpenGuild { guild_id } => {
                    self.close_top_surface();
                    (context_menu::actions().open_guild)(Id::new(guild_id));
                }

                PluginEvent::OpenProfile { user_id, guild_id } => {
                    (context_menu::actions().open_profile)(
                        Id::new(user_id),
                        guild_id.map(Id::new),
                        false,
                    )
                }
                PluginEvent::RefreshView { plugin, view_id } => {
                    if self.surfaces.with_untracked(|surfaces| {
                        surfaces
                            .iter()
                            .any(|s| s.plugin == plugin && s.view_id == view_id && s.open.on)
                    }) {
                        plugins::render_view(&plugin, &view_id, MODAL_CONTEXT);
                    }
                }
                PluginEvent::SendModified { plugins, .. } => {
                    let names = plugins
                        .iter()
                        .map(|id| self.name_of(id))
                        .collect::<Vec<_>>()
                        .join(", ");
                    context_menu::notify(&requiem_i18n::t!(
                        "plugins-send-modified",
                        plugins = names
                    ));
                }
            }
        }
        restores
    }
}

pub fn user_data(user: &models::User, cdn: &str) -> requiem_plugin_host::UserData {
    requiem_plugin_host::UserData {
        id: requiem_plugin_host::UserId::new(user.id.get()),
        username: user.username.clone(),
        global_name: user.global_name.clone(),
        avatar_url: user.avatar.as_ref().map(|hash| {
            format!(
                "{}/avatars/{}/{hash}.png",
                cdn.trim_end_matches('/'),
                user.id
            )
        }),
        bot: user.bot,
        ..Default::default()
    }
}

fn channel_kind(kind: u8) -> &'static str {
    match kind {
        0 => "text",
        1 => "dm",
        2 => "voice",
        3 => "group-dm",
        4 => "category",
        5 => "announcement",
        10..=12 => "thread",
        13 => "stage",
        15 => "forum",
        16 => "media",
        _ => "other",
    }
}

pub fn snapshot(
    guild: &Guild,
    channels: &[Channel],
    roles: &[Role],
    emojis: &[models::CustomEmoji],
    counts: (Option<u32>, Option<u32>),
    members: Vec<requiem_plugin_host::MemberData>,
    cdn: &str,
) -> GuildSnapshot {
    let cdn = cdn.trim_end_matches('/');
    GuildSnapshot {
        guild: requiem_plugin_host::GuildData {
            id: requiem_plugin_host::GuildId::new(guild.id.get()),
            name: guild.name.clone(),
            icon_url: guild
                .icon
                .as_ref()
                .filter(|hash| !hash.is_empty())
                .map(|hash| format!("{cdn}/icons/{}/{hash}.png", guild.id)),
            banner_url: guild
                .banner
                .as_ref()
                .filter(|hash| !hash.is_empty())
                .map(|hash| format!("{cdn}/banners/{}/{hash}.png", guild.id)),
            owner_id: guild
                .owner_id
                .map_or(requiem_plugin_host::UserId::new(0), |id| {
                    requiem_plugin_host::UserId::new(id.get())
                }),
            member_count: counts.0,
            online_count: counts.1,
            features: guild.features.clone(),
            description: guild.description.clone().filter(|d| !d.trim().is_empty()),
            boost_tier: guild.premium_tier as u32,
            boost_count: guild.premium_subscription_count,
            splash_url: guild
                .splash
                .as_ref()
                .filter(|hash| !hash.is_empty())
                .map(|hash| format!("{cdn}/splashes/{}/{hash}.png", guild.id)),
            discovery_splash_url: guild
                .discovery_splash
                .as_ref()
                .filter(|hash| !hash.is_empty())
                .map(|hash| format!("{cdn}/discovery-splashes/{}/{hash}.png", guild.id)),
            emojis: emojis
                .iter()
                .map(|e| (e.name.clone(), format!("{cdn}/emojis/{}.png", e.id)))
                .collect(),
        },
        channels: channels
            .iter()
            .filter(|c| !c.is_obfuscated())
            .map(|c| requiem_plugin_host::ChannelData {
                id: requiem_plugin_host::ChannelId::new(c.id.get()),
                guild_id: c
                    .guild_id
                    .map(|g| requiem_plugin_host::GuildId::new(g.get())),
                name: c.name.clone(),
                kind: channel_kind(c.kind).to_string(),
                parent_id: c
                    .parent_id
                    .map(|p| requiem_plugin_host::ChannelId::new(p.get())),
                position: c.position,
                last_message_id: c
                    .last_message_id
                    .map(|m| requiem_plugin_host::MessageId::new(m.get())),
                ..Default::default()
            })
            .collect(),
        roles: roles
            .iter()
            .map(|r| requiem_plugin_host::RoleData {
                id: requiem_plugin_host::RoleId::new(r.id.get()),
                guild_id: requiem_plugin_host::GuildId::new(guild.id.get()),
                name: r.name.clone(),
                color: (r.color != 0).then_some(r.color as u32),
                position: r.position,
                hoist: r.hoist,
                mentionable: r.mentionable,
                permissions: r.permissions.names(),
                ..Default::default()
            })
            .collect(),
        members,
    }
}

pub fn refresh_guild(state: &AppState, guild_id: Id<GuildMarker>) {
    let Some(guild) = state.guilds.find(guild_id) else {
        return;
    };
    let channels = state
        .channels
        .by_guild
        .with_untracked(|by_guild| by_guild.get(&guild_id).cloned().unwrap_or_default());
    let roles = state.permissions.guild_roles(guild_id);
    let emojis = state
        .emojis
        .by_guild
        .with_untracked(|by_guild| by_guild.get(&guild_id).cloned().unwrap_or_default());
    let list = state.members.active_list(guild_id);
    let counts = (
        list.as_ref().map(|l| l.member_count as u32),
        list.as_ref().map(|l| l.online_count as u32),
    );
    let cdn = plugins::cdn_base();
    let mut users = Vec::new();
    let members: Vec<requiem_plugin_host::MemberData> = list
        .iter()
        .flat_map(|l| l.loaded_rows())
        .filter_map(|row| match row {
            MemberListRow::Member(m) => Some(m),
            _ => None,
        })
        .map(|listed| {
            let user = user_data(&listed.member.user, &cdn);
            users.push(user.clone());
            requiem_plugin_host::MemberData {
                guild_id: requiem_plugin_host::GuildId::new(guild_id.get()),
                user,
                nick: listed.member.nick.clone(),
                role_ids: listed
                    .member
                    .roles
                    .iter()
                    .map(|r| requiem_plugin_host::RoleId::new(r.get()))
                    .collect(),
                ..Default::default()
            }
        })
        .collect();

    let relationships = state.relationships.list.with_untracked(|list| {
        list.iter()
            .map(|r| requiem_plugin_host::RelationshipData {
                user: user_data(&r.user, &cdn),
                kind: if r.is_friend() {
                    "friend"
                } else if r.is_blocked() {
                    "blocked"
                } else if r.is_incoming() {
                    "incoming-request"
                } else if r.is_outgoing() {
                    "outgoing-request"
                } else {
                    "other"
                }
                .to_string(),
                nickname: r.nickname.clone(),
                since: None,
            })
            .collect::<Vec<_>>()
    });
    let presences: Vec<requiem_plugin_host::PresenceData> = list
        .iter()
        .flat_map(|l| l.loaded_rows())
        .filter_map(|row| match row {
            MemberListRow::Member(m) => {
                Some(requiem_plugin_host::PresenceData {
                    user_id: requiem_plugin_host::UserId::new(m.member.user.id.get()),
                    status: match m.status {
                        PresenceStatus::Online => "online",
                        PresenceStatus::Idle => "idle",
                        PresenceStatus::Dnd => "dnd",
                        PresenceStatus::Offline => "offline",
                    }
                    .to_string(),
                    ..Default::default()
                })
            }
            _ => None,
        })
        .collect();
    plugins::set_social(relationships, presences);
    plugins::remember_users(users);
    plugins::set_guild_snapshot(guild_id.get(), |cdn| {
        snapshot(&guild, &channels, &roles, &emojis, counts, members, cdn)
    });
    plugins::set_selected_guild(Some(guild_id.get()));
}

fn plugins_of(outcome: &IncomingOutcome) -> impl Iterator<Item = &str> {
    outcome
        .hidden
        .iter()
        .map(|h| h.plugin.as_str())
        .chain(outcome.display_content.iter().map(|d| d.plugin.as_str()))
        .chain(outcome.annotations.iter().map(|a| a.plugin.as_str()))
}

fn retain_plugins(outcome: &mut IncomingOutcome, keep: impl Fn(&str) -> bool) {
    if outcome.hidden.as_ref().is_some_and(|h| !keep(&h.plugin)) {
        outcome.hidden = None;
    }
    if outcome
        .display_content
        .as_ref()
        .is_some_and(|d| !keep(&d.plugin))
    {
        outcome.display_content = None;
    }
    outcome.annotations.retain(|a| keep(&a.plugin));
}

fn remember_guild(guild: &GuildData, channel_guilds: &mut HashMap<u64, u64>) {
    let guild_id = guild.guild.id.get();
    for channel in &guild.channels {
        channel_guilds.insert(channel.id.get(), guild_id);
    }
}

fn remember_snapshot(data: &GuildData) {
    if let Some(joined) = data.joined_at_ms {
        plugins::remember_joined(data.guild.id.get(), joined);
    }
    plugins::set_guild_snapshot(data.guild.id.get(), |cdn| {
        snapshot(
            &data.guild,
            &data.channels,
            &data.roles,
            &data.emojis,
            (None, None),
            Vec::new(),
            cdn,
        )
    });
}

impl Store for PluginStore {
    fn handle(&self, action: &Action) {
        match action {
            Action::ConnectionOpen { user, guilds, .. } => {
                plugins::update_context(|me, channel_guilds| {
                    *me = Some(user.id.get());
                    channel_guilds.clear();
                    guilds
                        .iter()
                        .for_each(|g| remember_guild(g, channel_guilds));
                });
                plugins::remember_users([user_data(user, &plugins::cdn_base())]);
                guilds.iter().for_each(remember_snapshot);
            }
            Action::GuildCreate(guild) => {
                plugins::update_context(|_, channel_guilds| {
                    remember_guild(guild, channel_guilds)
                });
                remember_snapshot(guild);
            }
            Action::GuildDelete(id) => plugins::remove_guild_snapshot(id.get()),
            Action::ChannelCreate(data) | Action::ChannelUpdate(data) => {
                let channel = data.channel();
                if let Some(guild_id) = channel.guild_id {
                    plugins::update_context(|_, channel_guilds| {
                        channel_guilds.insert(channel.id.get(), guild_id.get());
                    });
                }
            }
            _ => {}
        }
    }

    fn reset(&self) {
        self.overlays.set(Rc::new(HashMap::new()));
        self.show_original.set(HashSet::new());
        plugins::update_context(|me, channel_guilds| {
            *me = None;
            channel_guilds.clear();
        });
    }
}
