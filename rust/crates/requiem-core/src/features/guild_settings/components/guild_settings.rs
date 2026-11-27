use crate::rest::endpoints::users;
use crate::rest::endpoints;
use crate::rest;
use crate::state::Action;
use requiem_i18n::key;
use requiem_ui::timer;
use std::cell::Cell;
use std::collections;
use std::env;
use std::future::Future;
use std::mem;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::anim::ToggleState;

use crate::models::id::marker::{GuildMarker, RoleMarker, UserMarker};
use crate::models::{Guild, Id, Permissions, Role};
use crate::query::QueryClient;
use crate::rest::HttpClient;
use crate::rest::endpoints::guilds::{self, RoleEdit};
use crate::state::AppState;
use crate::ui::{PendingImage, PendingUpload};

pub const SERVER_PROFILE_PAGE: &str = requiem_i18n::key!("guild-settings-server-profile");
pub const INVITES_PAGE: &str = requiem_i18n::key!("guild-settings-invites");
pub const ROLES_PAGE: &str = requiem_i18n::key!("guild-settings-roles");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoleTab {
    Display,
    Permissions,
    Links,
    Members,
}

#[derive(Clone, Debug)]
pub struct RoleDraft {
    pub role_id: Id<RoleMarker>,
    pub saved: RoleEdit,
    pub current: RoleEdit,
}

impl RoleDraft {
    pub fn of(role: &Role) -> Self {
        let edit = RoleEdit::of(role);
        Self {
            role_id: role.id,
            saved: edit.clone(),
            current: edit,
        }
    }

    pub fn dirty(&self) -> bool {
        self.saved != self.current
    }

    pub fn reset(&mut self) {
        self.current = self.saved.clone();
    }
}

pub const BRAND_SWATCHES: [(&str, [u8; 3], [u8; 3]); 10] = [
    (
        requiem_i18n::key!("guild-settings-server-icon-color"),
        [87, 87, 89],
        [18, 18, 20],
    ),
    (
        requiem_i18n::key!("guild-settings-soul"),
        [255, 143, 230],
        [255, 28, 144],
    ),
    (
        requiem_i18n::key!("guild-settings-volcano"),
        [255, 135, 106],
        [232, 29, 30],
    ),
    (
        requiem_i18n::key!("guild-settings-marsh"),
        [255, 196, 113],
        [232, 110, 29],
    ),
    (
        requiem_i18n::key!("guild-settings-thunder"),
        [255, 255, 138],
        [232, 192, 47],
    ),
    (
        requiem_i18n::key!("guild-settings-rising"),
        [199, 134, 224],
        [113, 54, 138],
    ),
    (
        requiem_i18n::key!("guild-settings-zephyr"),
        [151, 246, 255],
        [2, 159, 252],
    ),
    (
        requiem_i18n::key!("guild-settings-cascade"),
        [179, 255, 255],
        [79, 226, 202],
    ),
    (
        requiem_i18n::key!("guild-settings-earth"),
        [147, 184, 88],
        [64, 102, 1],
    ),
    (
        requiem_i18n::key!("guild-settings-boulder"),
        [112, 112, 112],
        [39, 39, 39],
    ),
];

pub const TRAIT_SLOTS: usize = 5;

#[derive(Clone, Debug, Default)]
pub struct ProfileDraft {
    pub seeded: bool,
    pub name: String,
    pub saved_name: String,
    pub description: String,
    pub saved_description: String,
    pub traits: Vec<String>,
    pub saved_traits: Vec<String>,
    pub private: bool,
    pub saved_private: bool,

    pub brand: usize,
    pub saved_brand: usize,
    pub icon: PendingImage,

    pub traits_full: Vec<guilds::GuildTrait>,
    pub game_application_ids: Vec<String>,
}

impl ProfileDraft {
    fn of(guild: &Guild) -> Self {
        Self {
            name: guild.name.clone(),
            saved_name: guild.name.clone(),
            description: guild.description.clone().unwrap_or_default(),
            saved_description: guild.description.clone().unwrap_or_default(),
            traits: vec![String::new(); TRAIT_SLOTS],
            saved_traits: vec![String::new(); TRAIT_SLOTS],
            ..Default::default()
        }
    }

    pub fn dirty(&self) -> bool {
        self.seeded
            && (self.name.trim() != self.saved_name
                || self.description != self.saved_description
                || self.traits != self.saved_traits
                || self.private != self.saved_private
                || self.brand != self.saved_brand
                || self.icon.is_changed())
    }

    pub fn reset(&mut self) {
        self.name = self.saved_name.clone();
        self.description = self.saved_description.clone();
        self.traits = self.saved_traits.clone();
        self.private = self.saved_private;
        self.brand = self.saved_brand;
        self.icon = PendingImage::Unchanged;
    }
}

#[derive(Clone, Debug)]
pub enum Dialog {
    AddMembers(Role),
}

#[derive(Clone)]
pub struct GuildSettings {
    pub open: Signal<ToggleState>,
    pub guild: Signal<Option<Id<GuildMarker>>>,
    pub page: Signal<String>,

    pub role: Signal<Option<Id<RoleMarker>>>,
    pub tab: Signal<RoleTab>,
    pub draft: Signal<Option<RoleDraft>>,
    pub profile: Signal<ProfileDraft>,
    pub dialog: Signal<Option<Dialog>>,

    pub members: Signal<Option<Vec<Id<UserMarker>>>>,

    pub picked: Signal<Vec<Id<UserMarker>>>,

    pub epoch: Signal<u32>,

    pub error: Signal<Option<String>>,
}

thread_local! {
    static STATE: GuildSettings = GuildSettings {
        open: signal(ToggleState::new(false)),
        guild: signal(None),
        page: signal(SERVER_PROFILE_PAGE.to_string()),
        role: signal(None),
        tab: signal(RoleTab::Display),
        draft: signal(None),
        profile: signal(ProfileDraft::default()),
        dialog: signal(None),
        members: signal(None),
        picked: signal(Vec::new()),
        epoch: signal(0),
        error: signal(None),
    };
    static ROLE_REQUEST: Cell<u64> = const { Cell::new(0) };
}

pub fn state() -> GuildSettings {
    STATE.with(Clone::clone)
}

pub fn is_open() -> bool {
    STATE.with(|s| s.open.peek().on)
}

pub fn open(app: &AppState, guild_id: Id<GuildMarker>, page: &str) {
    let Some(guild) = app.guilds.find(guild_id) else {
        return;
    };
    if !can_access(app, &guild) {
        return;
    }
    let page = if can_see_page(app, &guild, page) {
        page
    } else {
        SERVER_PROFILE_PAGE
    };
    STATE.with(|s| {
        s.guild.set(Some(guild_id));
        s.page.set(page.to_string());
        s.role.set(None);
        s.draft.set(None);
        s.members.set(None);
        s.dialog.set(None);
        s.picked.set(Vec::new());
        s.error.set(None);
        s.profile.set(ProfileDraft::of(&guild));
        s.epoch.update(|n| *n += 1);
        s.open.update(|t| *t = t.set(true));
    });
}

pub fn close() {
    STATE.with(|s| s.open.update(|t| *t = t.set(false)));
}

pub fn go_to_page(page: &str) {
    STATE.with(|s| {
        if s.page.peek() != page {
            s.page.set(page.to_string());
            s.role.set(None);
            s.draft.set(None);
            s.error.set(None);
        }
    });
}

pub fn edit_role(role: &Role, guild_id: Id<GuildMarker>) {
    STATE.with(|s| {
        let everyone = role.id.get() == guild_id.get();
        s.role.set(Some(role.id));
        s.tab.set(if everyone {
            RoleTab::Permissions
        } else {
            RoleTab::Display
        });
        s.draft.set(Some(RoleDraft::of(role)));
        s.members.set(None);
        s.error.set(None);
        s.epoch.update(|n| *n += 1);
    });
}

pub fn back_to_roles() {
    STATE.with(|s| {
        s.role.set(None);
        s.draft.set(None);
        s.error.set(None);
    });
}

pub fn run<T, Fut>(
    queries: &QueryClient,
    work: impl FnOnce() -> Fut + Send + 'static,
    done: impl FnOnce(Result<T, String>) + 'static,
) where
    T: Send + 'static,
    Fut: Future<Output = Result<T, String>> + Send + 'static,
{
    let slot: Arc<Mutex<Option<Result<T, String>>>> = Arc::new(Mutex::new(None));
    let writer = slot.clone();
    queries.mutate(Vec::new(), move || async move {
        let result = work().await;
        *writer.lock().expect("result mutex poisoned") = Some(result);
        Ok::<(), String>(())
    });
    poll(slot, Box::new(done), 0);
}

fn poll<T: Send + 'static>(
    slot: Arc<Mutex<Option<Result<T, String>>>>,
    done: Box<dyn FnOnce(Result<T, String>)>,
    attempt: u32,
) {
    timer::set_timeout(Duration::from_millis(60), move || {
        let result = slot.lock().expect("result mutex poisoned").take();
        match result {
            Some(result) => done(result),
            None if attempt < 600 => poll(slot, done, attempt + 1),
            None => {}
        }
    });
}

pub fn save_role(queries: &QueryClient, http: &HttpClient, guild_id: Id<GuildMarker>) {
    let s = state();
    let Some(draft) = s.draft.peek() else { return };
    if !draft.dirty() {
        return;
    }
    let (http, edit, role_id) = (http.clone(), draft.current.clone(), draft.role_id);
    let sent = edit.clone();
    s.error.set(None);
    run(
        queries,
        move || async move { guilds::update_role(&http, guild_id, role_id, &edit).await },
        move |result| {
            let s = state();
            match result {
                Ok(_) => s.draft.update(|d| {
                    if let Some(d) = d.as_mut().filter(|d| d.role_id == role_id) {
                        d.saved = sent;
                    }
                }),
                Err(message) => s.error.set(Some(message)),
            }
        },
    );
}

pub fn create_role(
    queries: &QueryClient,
    http: &HttpClient,
    app: &AppState,
    guild_id: Id<GuildMarker>,
) {
    let http = http.clone();
    let app = app.clone();
    run(
        queries,
        move || async move { guilds::create_role(&http, guild_id).await },
        move |result| match result {
            Ok(role) => {
                app.dispatch(&Action::GuildRoleUpsert {
                    guild_id,
                    role: role.clone(),
                });
                edit_role(&role, guild_id);
            }
            Err(message) => state().error.set(Some(message)),
        },
    );
}

pub fn delete_role(
    queries: &QueryClient,
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
    role_id: Id<RoleMarker>,
) {
    let http = http.clone();
    run(
        queries,
        move || async move { guilds::delete_role(&http, guild_id, role_id).await },
        move |result| {
            let s = state();
            match result {
                Ok(()) => {
                    if s.role.peek() == Some(role_id) {
                        back_to_roles();
                    }
                }
                Err(message) => s.error.set(Some(message)),
            }
        },
    );
}

fn counts_key(guild_id: Id<GuildMarker>) -> String {
    format!("guild_role_counts:{guild_id}")
}

pub fn members_key(guild_id: Id<GuildMarker>, role_id: Id<RoleMarker>) -> String {
    format!("guild_role_members:{guild_id}:{role_id}")
}

pub fn role_counts(
    queries: &QueryClient,
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
) -> Option<Arc<collections::HashMap<String, u64>>> {
    let http = http.clone();
    queries
        .query_default::<collections::HashMap<String, u64>, _, rest::RestError>(
            &counts_key(guild_id),
            move || async move {
                let response = http
                    .request(
                        "GET /guilds/:id/roles/member-counts",
                        wreq::Method::GET,
                        &format!("/guilds/{guild_id}/roles/member-counts"),
                    )
                    .await?;
                Ok(response.json().await?)
            },
        )
        .data
}

pub fn role_members(
    queries: &QueryClient,
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
    role_id: Id<RoleMarker>,
) -> Option<Arc<Vec<Id<UserMarker>>>> {
    let http = http.clone();
    queries
        .query_default::<Vec<Id<UserMarker>>, _, rest::RestError>(
            &members_key(guild_id, role_id),
            move || async move { guilds::role_member_ids(&http, guild_id, role_id).await },
        )
        .data
}

pub fn add_members(
    queries: &QueryClient,
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
    role_id: Id<RoleMarker>,
    ids: Vec<Id<UserMarker>>,
) {
    let http = http.clone();
    let q = queries.clone();
    run(
        queries,
        move || async move { guilds::add_role_members(&http, guild_id, role_id, &ids).await },
        move |result| {
            let s = state();
            match result {
                Ok(()) => {
                    q.invalidate(&members_key(guild_id, role_id));
                    q.invalidate(&counts_key(guild_id));
                    s.dialog.set(None);
                    s.picked.set(Vec::new());
                }
                Err(message) => s.error.set(Some(message)),
            }
        },
    );
}

pub fn remove_member(
    queries: &QueryClient,
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
    role_id: Id<RoleMarker>,
    user_id: Id<UserMarker>,
) {
    let http = http.clone();
    let q = queries.clone();
    run(
        queries,
        move || async move {
            users::remove_member_role(&http, guild_id, user_id, role_id)
                .await
                .map_err(|e| endpoints::error_message(&e))
        },
        move |result| match result {
            Ok(()) => {
                q.invalidate(&members_key(guild_id, role_id));
                q.invalidate(&counts_key(guild_id));
            }
            Err(message) => state().error.set(Some(message)),
        },
    );
}

pub fn seed_profile(profile: &guilds::GuildProfile) {
    let s = state();
    if s.profile.peek().seeded {
        return;
    }
    s.profile.update(|d| {
        d.seeded = true;
        d.description = profile.description.clone().unwrap_or_default();
        d.saved_description = d.description.clone();
        let mut traits: Vec<String> = profile
            .traits
            .iter()
            .map(|t| t.label.clone())
            .take(TRAIT_SLOTS)
            .collect();
        traits.resize(TRAIT_SLOTS, String::new());
        d.traits = traits.clone();
        d.saved_traits = traits;
        d.traits_full = profile.traits.clone();
        d.private = profile.visibility == 2;
        d.saved_private = d.private;
        d.game_application_ids = profile.game_application_ids.clone();
    });
    s.epoch.update(|n| *n += 1);
}

pub fn profile_key(guild_id: Id<GuildMarker>) -> String {
    format!("guild_profile:{guild_id}")
}

pub fn profile(
    queries: &QueryClient,
    http: &HttpClient,
    guild_id: Id<GuildMarker>,
) -> Option<Arc<guilds::GuildProfile>> {
    let http = http.clone();
    queries
        .query_default::<guilds::GuildProfile, _, rest::RestError>(
            &profile_key(guild_id),
            move || async move {
                match guilds::get_guild_profile(&http, guild_id).await {
                    Err(_) if env::var_os("REQUIEM_SPOOF_PERMISSIONS").is_some() => {
                        Ok(guilds::GuildProfile {
                            member_count: 3,
                            online_count: 2,
                            ..Default::default()
                        })
                    }
                    other => other,
                }
            },
        )
        .data
}

pub fn save_profile(queries: &QueryClient, http: &HttpClient, guild_id: Id<GuildMarker>) {
    let s = state();
    let d = s.profile.peek();
    if !d.dirty() {
        return;
    }
    let icon = match &d.icon {
        PendingImage::Set(upload) => Some(upload.data_uri.to_string()),
        _ => None,
    };
    let traits: Vec<guilds::GuildTrait> = d
        .traits
        .iter()
        .enumerate()
        .filter(|(_, label)| !label.trim().is_empty())
        .map(|(index, label)| {
            let kept = d.traits_full.iter().find(|t| t.label == *label);
            guilds::GuildTrait {
                emoji_id: kept.and_then(|t| t.emoji_id),
                emoji_name: kept.and_then(|t| t.emoji_name.clone()),
                emoji_animated: kept.is_some_and(|t| t.emoji_animated),
                label: label.trim().to_string(),
                position: index as i32,
            }
        })
        .collect();
    let brand = (d.brand != 0).then(|| {
        let [r, g, b] = BRAND_SWATCHES[d.brand].2;
        format!("#{r:02x}{g:02x}{b:02x}")
    });
    let edit = guilds::GuildProfileEdit {
        name: d.name.trim().to_string(),
        description: d.description.clone(),
        icon,
        visibility: if d.private { 2 } else { 1 },
        brand_color_primary: brand,
        traits,
        game_application_ids: d.game_application_ids.clone(),
    };
    let http = http.clone();
    s.error.set(None);
    run(
        queries,
        move || async move { guilds::update_guild_profile(&http, guild_id, &edit).await },
        move |result| {
            let s = state();
            match result {
                Ok(()) => s.profile.update(|d| {
                    d.saved_name = d.name.trim().to_string();
                    d.saved_description = d.description.clone();
                    d.saved_traits = d.traits.clone();
                    d.saved_private = d.private;
                    d.saved_brand = d.brand;
                    d.icon = match mem::take(&mut d.icon) {
                        PendingImage::Set(upload) => PendingImage::Saved(upload),
                        other => other,
                    };
                }),
                Err(message) => s.error.set(Some(message)),
            }
        },
    );
}

pub fn pending_icon(profile: &ProfileDraft) -> Option<&PendingUpload> {
    profile.icon.shown()
}

pub fn can_grant(app: &AppState, guild: &Guild, flag: Permissions) -> bool {
    app.permissions.guild_permissions(guild).contains(flag)
}

pub fn everyone_id(guild_id: Id<GuildMarker>) -> Id<RoleMarker> {
    Id::new(guild_id.get())
}

pub const SETTINGS_ACCESS: &[Permissions] = &[
    Permissions::MANAGE_GUILD,
    Permissions::MANAGE_ROLES,
    Permissions::ADMINISTRATOR,
    Permissions::BAN_MEMBERS,
    Permissions::MANAGE_NICKNAMES,
    Permissions::MANAGE_GUILD_EXPRESSIONS,
    Permissions::MANAGE_WEBHOOKS,
    Permissions::VIEW_AUDIT_LOG,
    Permissions::KICK_MEMBERS,
    Permissions::MANAGE_EVENTS,
    Permissions::MANAGE_CHANNELS,
];

pub fn can_access(app: &AppState, guild: &Guild) -> bool {
    let bits = app.permissions.guild_permissions(guild);
    SETTINGS_ACCESS.iter().any(|flag| bits.contains(*flag))
}

pub fn can_see_page(app: &AppState, guild: &Guild, page: &str) -> bool {
    let Some(flag) = page_permission(page) else {
        return false;
    };
    can_grant(app, guild, flag)
}

#[allow(clippy::redundant_guards)]
fn page_permission(page: &str) -> Option<Permissions> {
    Some(match page {
        p if p == key!("guild-settings-emoji")
            || p == key!("guild-settings-stickers")
            || p == key!("guild-settings-soundboard") =>
        {
            Permissions::MANAGE_GUILD_EXPRESSIONS
        }
        p if p == key!("guild-settings-roles") => Permissions::MANAGE_ROLES,
        p if p == key!("guild-settings-audit-log") => Permissions::VIEW_AUDIT_LOG,
        p if p == key!("guild-settings-bans") => Permissions::BAN_MEMBERS,
        p if p == key!("guild-settings-server-profile")
            || p == key!("guild-settings-server-tag")
            || p == key!("guild-settings-engagement")
            || p == key!("guild-settings-boost-perks")
            || p == key!("guild-settings-invites")
            || p == key!("guild-settings-access")
            || p == key!("guild-settings-integrations")
            || p == key!("guild-settings-app-directory")
            || p == key!("guild-settings-safety-setup")
            || p == key!("guild-settings-automod")
            || p == key!("guild-settings-enable-community")
            || p == key!("guild-settings-server-template") =>
        {
            Permissions::MANAGE_GUILD
        }
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn role() -> Role {
        Role {
            id: Id::new(9),
            colors: Default::default(),
            name: "mods".into(),
            color: 0xff0000,
            position: 3,
            icon: None,
            unicode_emoji: None,
            managed: false,
            hoist: false,
            mentionable: false,
            permissions: Permissions::SEND_MESSAGES,
        }
    }

    #[test]
    fn a_role_draft_is_dirty_only_when_it_differs_and_reset_undoes_it() {
        let mut draft = RoleDraft::of(&role());
        assert!(!draft.dirty());
        draft.current.permissions = draft.current.permissions | Permissions::BAN_MEMBERS;
        assert!(draft.dirty());
        draft.reset();
        assert!(!draft.dirty());
        draft.current.name = "renamed".into();
        draft.current.name = "mods".into();
        assert!(
            !draft.dirty(),
            "editing back to the saved value is not a change"
        );
    }

    #[test]
    fn settings_pages_follow_the_permission_that_owns_them() {
        assert_eq!(
            page_permission(requiem_i18n::key!("guild-settings-roles")),
            Some(Permissions::MANAGE_ROLES)
        );
        assert_eq!(
            page_permission(requiem_i18n::key!("guild-settings-audit-log")),
            Some(Permissions::VIEW_AUDIT_LOG)
        );
        assert_eq!(
            page_permission(requiem_i18n::key!("guild-settings-bans")),
            Some(Permissions::BAN_MEMBERS)
        );
        assert_eq!(
            page_permission(requiem_i18n::key!("guild-settings-emoji")),
            Some(Permissions::MANAGE_GUILD_EXPRESSIONS)
        );
        assert_eq!(
            page_permission(requiem_i18n::key!("guild-settings-server-profile")),
            Some(Permissions::MANAGE_GUILD)
        );
        assert_eq!(page_permission("not-a-page"), None);
        assert!(SETTINGS_ACCESS.contains(&Permissions::VIEW_AUDIT_LOG));
        assert!(SETTINGS_ACCESS.contains(&Permissions::MANAGE_GUILD));
    }
}
