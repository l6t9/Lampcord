use crate::rest::endpoints::channels;
use crate::rest::endpoints::messages::send_location;
use crate::rest::endpoints::messages;
use crate::rest::endpoints::relationships;
use crate::ui::classes::invite_modal as cls;
use crate::ui::guild_settings;
use crate::ui::home;
use crate::ui;
use crate::utils::link;
use requiem_ui::colors;
use requiem_ui::components::AvatarProps;
use requiem_ui::components::ConfirmModalProps;
use requiem_ui::widget::primitives::PresenceStatus;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use requiem_ui::components::{
    AvatarStatusSize, ButtonProps, ButtonSize, ButtonVariant, CopyInputProps, FormItemProps,
    ModalProps, ModalSize, SelectOption, SelectProps, StackProps, SwitchProps, TextInputProps,
    TextLinkProps, TextLinkVariant, TextProps, TextVariant, avatar_with_status, div, icon,
    input_leading_icon,
};
use requiem_ui::control::section;
use requiem_ui::node::Node;
use requiem_ui::signal::{Effect, Signal, signal};
use requiem_ui::widget::anim::ToggleMemo;
use requiem_ui::widget::{WidgetId, WidgetTree};

use crate::models::id::marker::{ChannelMarker, GuildMarker, UserMarker};
use crate::models::{Channel, Id, Permissions, User};
use crate::rest::endpoints::invites::{self, InviteSettings, Validate};
use crate::state::DmChannel;
use crate::ui::dialog::Dialog;
use crate::ui::home::{HomeEnv, dm_title, user_name};
use crate::ui::{TEXT_DEFAULT, TEXT_MUTED};
use requiem_ui::icons::MAGNIFYING_GLASS;

pub mod location {

    pub const HEADER_BUTTON: &str = "Guild Header Invite Button";

    pub const HEADER: &str = "Guild Header";

    pub const CONTEXT_MENU: &str = "Guild Context Menu";

    pub const CHANNEL_CONTEXT_MENU: &str = "Channel Context Menu";
}

const MAX_ROWS: usize = 100;

const INVITES_HELP_URL: &str = "https://support.discord.com/hc/en-us/articles/208866998";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Main,
    Settings,
}

/// What the dialog is for, as the web client decides it: invites you can create, the server's
/// vanity link (shown as it is, with nothing to edit), or neither.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Create,
    Vanity,
    Denied,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Send {
    Sending,
    Sent,
}

thread_local! {
    static DIALOG: Dialog = Dialog::new();
    static ENV: RefCell<Option<Rc<dyn Fn() -> HomeEnv>>> = const { RefCell::new(None) };
    static SELECTED: RefCell<Option<Signal<Option<Id<ChannelMarker>>>>> = const { RefCell::new(None) };
    static GUILD: Cell<Option<Id<GuildMarker>>> = const { Cell::new(None) };
    static MODE: Cell<Mode> = const { Cell::new(Mode::Create) };
    static VANITY: RefCell<Option<String>> = const { RefCell::new(None) };

    static CHANNEL: RefCell<Option<Channel>> = const { RefCell::new(None) };
    static LOCATION: Cell<&'static str> = const { Cell::new(location::HEADER) };
    static PAGE: Signal<Page> = signal(Page::Main);
    static QUERY: Signal<String> = signal(String::new());

    static CODE: Signal<Option<String>> = signal(None);
    static ERROR: Signal<Option<String>> = signal(None);
    /// The server is at its invite limit, which has its own message.
    static LIMIT: Signal<bool> = signal(false);

    static SETTINGS: Signal<InviteSettings> = signal(InviteSettings::default());

    static DRAFT: Signal<InviteSettings> = signal(InviteSettings::default());
    static AGE_OPEN: Signal<bool> = signal(false);
    static USES_OPEN: Signal<bool> = signal(false);
    static TEMPORARY_TOGGLE: ToggleMemo = ToggleMemo::new(false);

    static SENDS: Signal<HashMap<u64, Send>> = signal(HashMap::new());

    static LAST_CODES: RefCell<HashMap<Id<ChannelMarker>, String>> = RefCell::new(HashMap::new());
}

pub fn is_open() -> bool {
    DIALOG.with(Dialog::is_open)
}

pub fn close() {
    AGE_OPEN.with(|o| o.set(false));
    USES_OPEN.with(|o| o.set(false));
    DIALOG.with(Dialog::close);
}

pub fn escape() {
    let select_open = AGE_OPEN.with(Signal::peek) || USES_OPEN.with(Signal::peek);
    if select_open {
        AGE_OPEN.with(|o| o.set(false));
        USES_OPEN.with(|o| o.set(false));
    } else {
        close();
    }
}

pub fn open(guild_id: Id<GuildMarker>, from: &'static str) {
    open_with(guild_id, None, from);
}

pub fn open_channel(guild_id: Id<GuildMarker>, channel_id: Id<ChannelMarker>) {
    open_with(guild_id, Some(channel_id), location::CHANNEL_CONTEXT_MENU);
}

fn open_with(guild_id: Id<GuildMarker>, explicit: Option<Id<ChannelMarker>>, from: &'static str) {
    let Some(env) = ENV.with(|e| e.borrow().clone()).map(|factory| factory()) else {
        return;
    };
    let channel = match explicit {
        Some(id) => ui::find_channel(&env.state, id).map(|(_, channel)| channel),
        None => target_channel(
            &env,
            guild_id,
            SELECTED.with(|s| s.borrow().as_ref().and_then(Signal::peek)),
        ),
    };
    let can_create = channel
        .as_ref()
        .is_some_and(|c| env.state.can(c, Permissions::CREATE_INSTANT_INVITE));
    let vanity = env
        .state
        .guilds
        .find(guild_id)
        .and_then(|guild| guild.vanity_url_code)
        .filter(|code| !code.is_empty());
    MODE.with(|m| {
        m.set(match (can_create, &vanity) {
            (true, _) => Mode::Create,
            (false, Some(_)) => Mode::Vanity,
            (false, None) => Mode::Denied,
        })
    });
    VANITY.with(|v| *v.borrow_mut() = vanity);
    GUILD.with(|g| g.set(Some(guild_id)));
    CHANNEL.with(|c| *c.borrow_mut() = channel.clone());
    LOCATION.with(|l| l.set(from));
    PAGE.with(|p| p.set(Page::Main));
    QUERY.with(|q| q.set(String::new()));
    CODE.with(|c| c.set(vanity_placeholder()));
    ERROR.with(|e| e.set(None));
    LIMIT.with(|l| l.set(false));
    SETTINGS.with(|s| s.set(InviteSettings::default()));
    SENDS.with(|s| s.set(HashMap::new()));
    let (guild_can, text_total, text_can) = {
        let guild = env.state.guilds.find(guild_id);
        let guild_can = guild.as_ref().is_some_and(|g| {
            env.state
                .permissions
                .guild_permissions(g)
                .contains(Permissions::CREATE_INSTANT_INVITE)
        });
        let texts: Vec<Channel> = env.state.channels.by_guild.with_untracked(|all| {
            all.get(&guild_id)
                .map(|list| list.iter().filter(|c| matches!(c.kind, 0 | 5)).cloned().collect())
                .unwrap_or_default()
        });
        let can = texts
            .iter()
            .filter(|c| env.state.can(c, Permissions::CREATE_INSTANT_INVITE))
            .count();
        (guild_can, texts.len(), can)
    };
    log::debug!(
        "invite modal: guild={guild_id} channel={:?} can_create={can_create} vanity={:?} \
         guild_level_create={guild_can} text_channels={text_total} invitable={text_can}",
        channel.as_ref().map(|c| c.id),
        VANITY.with(|v| v.borrow().clone())
    );
    DIALOG.with(Dialog::open);
    if let Some(channel) = channel.filter(|c| env.state.can(c, Permissions::CREATE_INSTANT_INVITE))
    {
        let validate = LAST_CODES.with(|codes| codes.borrow().get(&channel.id).cloned());
        request_invite(
            &env,
            channel.id,
            InviteSettings::default(),
            validate.map_or(ValidateOwned::Null, ValidateOwned::Code),
        );
    }
}

fn target_channel(
    env: &HomeEnv,
    guild_id: Id<GuildMarker>,
    selected: Option<Id<ChannelMarker>>,
) -> Option<Channel> {
    let state = &env.state;
    let current = selected
        .and_then(|id| ui::find_channel(state, id))
        .filter(|(guild, _)| *guild == guild_id)
        .map(|(_, channel)| channel);
    let current = current.and_then(|channel| match channel.kind {
        10..=12 => channel
            .parent_id
            .and_then(|parent| ui::find_channel(state, parent))
            .map(|(_, parent)| parent),
        _ => Some(channel),
    });
    // The open channel, if you can invite from it. Otherwise any channel you can: one channel's
    // overwrites shouldn't turn the whole server's invite button into the no-permission page.
    let viewable = current.filter(|c| state.can(c, Permissions::VIEW_CHANNEL));
    if let Some(channel) = viewable
        .clone()
        .filter(|c| state.can(c, Permissions::CREATE_INSTANT_INVITE))
    {
        return Some(channel);
    }
    let any = state.channels.by_guild.with_untracked(|by_guild| {
        let mut channels: Vec<&Channel> = by_guild
            .get(&guild_id)?
            .iter()
            .filter(|c| {
                matches!(c.kind, 0 | 5)
                    && state.can(
                        c,
                        Permissions::VIEW_CHANNEL | Permissions::CREATE_INSTANT_INVITE,
                    )
            })
            .collect();
        channels.sort_by_key(|c| c.position.unwrap_or(0));
        channels.first().map(|c| (*c).clone())
    });
    any.or(viewable)
}

enum ValidateOwned {
    Code(String),
    Null,
    Omit,
}

/// The vanity code stands in for the link until a real one arrives, and stays if none does
/// (the server's invite limit, a failed request).
fn vanity_placeholder() -> Option<String> {
    VANITY.with(|v| v.borrow().clone())
}

fn request_invite(
    env: &HomeEnv,
    channel_id: Id<ChannelMarker>,
    settings: InviteSettings,
    validate: ValidateOwned,
) {
    let (http, from) = (env.http.clone(), LOCATION.with(Cell::get));
    guild_settings::run(
        &env.queries,
        move || async move {
            let validate = match &validate {
                ValidateOwned::Code(code) => Validate::Code(Some(code)),
                ValidateOwned::Null => Validate::Code(None),
                ValidateOwned::Omit => Validate::Omit,
            };
            invites::create_invite(&http, channel_id, settings, validate, from).await
        },
        move |result| match result {
            Ok(invites::Created::TooMany) => LIMIT.with(|l| l.set(true)),
            Ok(invites::Created::Code(code)) => {
                LIMIT.with(|l| l.set(false));
                LAST_CODES.with(|codes| codes.borrow_mut().insert(channel_id, code.clone()));
                SETTINGS.with(|s| s.set(settings));
                CODE.with(|c| c.set(Some(code)));
                ERROR.with(|e| e.set(None));
            }
            Err(message) => ERROR.with(|e| e.set(Some(message))),
        },
    );
}

fn dialog(env: &HomeEnv) -> Node {
    match MODE.with(Cell::get) {
        Mode::Denied => return denied_page(),
        // The same dialog as when you can invite: the people list sends the vanity link, there is
        // just no editing of it.
        Mode::Vanity | Mode::Create => {}
    }
    match PAGE.with(Signal::get) {
        Page::Main => main_page(env),
        Page::Settings => settings_page(env),
    }
}

/// No permission to invite and no vanity link: only a notice.
fn denied_page() -> Node {
    ConfirmModalProps::new(
        requiem_i18n::t!("invite-modal-permission-required"),
        requiem_i18n::t!("invite-modal-got-it"),
        close,
        DIALOG.with(Dialog::toggle),
    )
    .message(requiem_i18n::t!("invite-modal-permission-required-body"))
    .notice()
    .build()
}

fn main_page(env: &HomeEnv) -> Node {
    let guild_name = GUILD
        .with(Cell::get)
        .and_then(|id| env.state.guilds.find(id))
        .map(|g| g.name)
        .unwrap_or_default();

    let search = TextInputProps::new()
        .placeholder(requiem_i18n::t!("invite-modal-search"))
        .leading(input_leading_icon(MAGNIFYING_GLASS))
        .clearable(true)
        .autofocus(true)
        .on_change(|text: &str| QUERY.with(|q| q.set(text.to_string())))
        .build();

    let list_env = env.clone();
    let list = Node::reactive(move || rows(&list_env)).class(cls::ROWS);

    let footer_env = env.clone();
    let preview = Node::reactive(move || link_section(&footer_env)).class(cls::PREVIEW);

    let mut props = ModalProps::default()
        .size(ModalSize::Medium)
        .heading(requiem_i18n::t!("invite-modal-title", name = guild_name))
        .dismissable(true)
        .on_dismiss(close)
        .open(DIALOG.with(Dialog::toggle))
        .controls(search)
        .list(list)
        .preview(preview);
    if let Some(subtitle) = CHANNEL
        .with(|c| c.borrow().clone())
        .map(|channel| subtitle(env, &channel))
    {
        props = props.subtitle_content(subtitle);
    }
    props.build()
}

fn subtitle(env: &HomeEnv, channel: &Channel) -> Node {
    let glyph = ui::channel_icon(&env.state, channel);
    let name = channel
        .name
        .clone()
        .unwrap_or_else(|| requiem_i18n::t!("invite-modal-no-access"));
    div().class(cls::SUBTITLE).children([
        TextProps::styled(
            requiem_i18n::t!("invite-modal-recipients-land-in"),
            TextVariant::TEXT_MD_NORMAL,
        )
        .color(ui::TEXT_SECONDARY)
        .build(),
        icon(glyph, ui::TEXT_SECONDARY).class(cls::SUBTITLE_ICON),
        TextProps::styled(name, TextVariant::TEXT_MD_NORMAL)
            .color(ui::TEXT_SECONDARY)
            .line_clamp(1_u32)
            .build(),
    ])
}

enum Row {
    User { user: User, member: bool },
    Group(DmChannel),
}

impl Row {
    fn key(&self) -> u64 {
        match self {
            Row::User { user, .. } => user.id.get(),
            Row::Group(dm) => dm.id.get(),
        }
    }
}

fn suggestions(env: &HomeEnv, query: &str) -> Vec<Row> {
    let state = &env.state;
    let guild = GUILD.with(Cell::get);
    let is_member = |id: Id<UserMarker>| guild.is_some_and(|g| state.members.is_member(g, id));
    let blocked = |id: Id<UserMarker>| {
        state.relationships.kind_of(id) == Some(relationships::BLOCKED)
    };
    let me = state
        .users
        .current
        .with_untracked(|u| u.as_ref().map(|u| u.id));
    let matches = |user: &User| {
        let name = user_name(state, user).to_lowercase();
        name.contains(query) || user.username.to_lowercase().contains(query)
    };
    let dms = state.dms.list.get();
    let friends: Vec<User> = state
        .relationships
        .list
        .get()
        .iter()
        .filter(|r| r.is_friend())
        .map(|r| r.user.clone())
        .collect();

    let mut shown: HashSet<Id<UserMarker>> = HashSet::new();
    let mut rows = Vec::new();
    let push_user = |rows: &mut Vec<Row>, shown: &mut HashSet<Id<UserMarker>>, user: User| {
        if Some(user.id) == me || user.bot || blocked(user.id) || shown.contains(&user.id) {
            return;
        }
        let member = is_member(user.id);
        if query.is_empty() && member {
            return;
        }
        shown.insert(user.id);
        rows.push(Row::User { user, member });
    };
    for dm in dms.iter() {
        if query.is_empty() && rows.len() >= MAX_ROWS {
            break;
        }
        if dm.is_group() {
            if query.is_empty() || dm_title(state, dm).to_lowercase().contains(query) {
                rows.push(Row::Group(dm.clone()));
            }
            continue;
        }
        if dm.last_message_id.is_none() {
            continue;
        }
        if let Some(user) = dm
            .recipients
            .first()
            .and_then(|id| state.users.get(*id))
            .filter(|u| query.is_empty() || matches(u))
        {
            push_user(&mut rows, &mut shown, user);
        }
    }
    for user in friends {
        if query.is_empty() && rows.len() >= MAX_ROWS {
            break;
        }
        if query.is_empty() || matches(&user) {
            push_user(&mut rows, &mut shown, user);
        }
    }
    rows
}

fn rows(env: &HomeEnv) -> Node {
    let _ = env.tick.get();
    let query = QUERY.with(Signal::get).trim().to_lowercase();
    let rows = suggestions(env, &query);
    let has_code = CODE.with(Signal::get).is_some();
    let sends = SENDS.with(Signal::get);
    // The people show as soon as we know them; sending waits for a link, but a link that fails
    // to arrive must not hide them.
    if rows.is_empty() {
        let message = requiem_i18n::t!("invite-modal-no-results");
        return div().class(cls::EMPTY).child(
            TextProps::styled(message, TextVariant::TEXT_MD_MEDIUM)
                .color(TEXT_MUTED)
                .build(),
        );
    }
    StackProps::vertical()
        .gap(0)
        .full_width(true)
        .build()
        .children(
            rows.into_iter()
                .map(|row| row_node(env, row, &sends, has_code)),
        )
}

fn row_node(env: &HomeEnv, row: Row, sends: &HashMap<u64, Send>, has_code: bool) -> Node {
    let key = row.key();
    let (avatar, name, sub, member) = match &row {
        Row::User { user, member } => {
            let status = env.state.user_presence.of(user.id).status;
            let status = (status != PresenceStatus::Offline)
                .then_some(status);
            (
                home::user_avatar(env, user, Some(AvatarStatusSize::Px32), status),
                user_name(&env.state, user),
                Some(user.username.clone()),
                *member,
            )
        }
        Row::Group(dm) => {
            let first = dm
                .recipients
                .first()
                .and_then(|id| env.state.users.get(*id));
            let avatar = match first {
                Some(user) => {
                    home::user_avatar(env, &user, Some(AvatarStatusSize::Px32), None)
                }
                None => avatar_with_status(
                    AvatarProps::default()
                        .typeface(env.fonts.typeface.clone()),
                    AvatarStatusSize::Px32,
                    None,
                ),
            };
            (avatar, dm_title(&env.state, dm), None, false)
        }
    };
    let mut labels = div().class(cls::LABELS).child(
        TextProps::styled(name, TextVariant::TEXT_MD_SEMIBOLD)
            .color(TEXT_DEFAULT)
            .line_clamp(1_u32)
            .build(),
    );
    if let Some(sub) = sub {
        labels = labels.child(
            TextProps::styled(sub, TextVariant::TEXT_XS_NORMAL)
                .color(TEXT_MUTED)
                .line_clamp(1_u32)
                .build(),
        );
    }
    let info = div()
        .class(cls::INFO)
        .children([avatar.class(cls::AVATAR), labels]);

    let action = if member {
        TextProps::styled(
            requiem_i18n::t!("invite-modal-member"),
            TextVariant::TEXT_SM_MEDIUM,
        )
        .color(TEXT_MUTED)
        .build()
    } else {
        let state = sends.get(&key).copied();
        let label = if state == Some(Send::Sent) {
            requiem_i18n::t!("invite-modal-sent")
        } else {
            requiem_i18n::t!("invite-modal-invite")
        };
        let env = env.clone();
        let target = match &row {
            Row::User { user, .. } => Target::User(user.id),
            Row::Group(dm) => Target::Channel(dm.id),
        };
        ButtonProps::new(ButtonVariant::Secondary, label, move || {
            send_invite(&env, key, target)
        })
        .size(ButtonSize::Small)
        .loading(state == Some(Send::Sending))
        .disabled(state == Some(Send::Sent) || !has_code)
        .build()
    };
    let mut node = div()
        .class(cls::ROW)
        .children([info, div().class(cls::ACTIONS).child(action)]);
    if member {
        node = node.class(cls::ROW_MEMBER);
    }
    node
}

#[derive(Clone, Copy)]
enum Target {
    User(Id<UserMarker>),
    Channel(Id<ChannelMarker>),
}

fn send_invite(env: &HomeEnv, key: u64, target: Target) {
    let Some(code) = CODE.with(Signal::peek) else {
        return;
    };
    if SENDS.with(|s| s.peek().contains_key(&key)) {
        return;
    }
    SENDS.with(|s| {
        s.update(|m| {
            m.insert(key, Send::Sending);
        })
    });
    let known_dm = match target {
        Target::User(user) => env.state.dms.with_user(user).map(|dm| dm.id),
        Target::Channel(channel) => Some(channel),
    };
    let http = env.http.clone();
    guild_settings::run(
        &env.queries,
        move || async move {
            let channel_id = match (known_dm, target) {
                (Some(channel), _) => channel,
                (None, Target::User(user)) => {
                    channels::create_dm(&http, user, None)
                        .await
                        .map_err(|e| e.to_string())?
                        .id
                }
                (None, Target::Channel(channel)) => channel,
            };
            let url = invites::invite_url(&code);
            messages::send_reply_from(
                &http,
                channel_id,
                &url,
                None,
                send_location::INVITE_SUGGESTION,
            )
            .await
            .map_err(|e| e.to_string())?;
            Ok::<(), String>(())
        },
        move |result| {
            SENDS.with(|s| {
                s.update(|m| match result {
                    Ok(()) => {
                        m.insert(key, Send::Sent);
                    }
                    Err(_) => {
                        m.remove(&key);
                    }
                })
            })
        },
    );
}

fn link_section(env: &HomeEnv) -> Node {
    let code = CODE.with(Signal::get);
    let can_invite = CHANNEL.with(|c| {
        c.borrow()
            .as_ref()
            .is_some_and(|channel| env.state.can(channel, Permissions::CREATE_INSTANT_INVITE))
    });
    let value = code.as_deref().map(invites::invite_url).unwrap_or_default();
    let mut stack = StackProps::vertical()
        .gap(16)
        .full_width(true)
        .build()
        .child(CopyInputProps::new(value).disabled(code.is_none()).build());
    if can_invite {
        let settings = SETTINGS.with(Signal::get);
        let edit = TextLinkProps::new(requiem_i18n::t!("invite-modal-edit-link"), open_settings)
            .variant(TextLinkVariant::Anchor)
            .inline_variant(TextVariant::TEXT_XS_NORMAL)
            .build();
        stack = stack.child(
            div().class(cls::EXPIRY).children([
                TextProps::styled(expiry_text(settings), TextVariant::TEXT_XS_NORMAL)
                    .color(TEXT_MUTED)
                    .build(),
                edit,
            ]),
        );
    }
    if LIMIT.with(Signal::get) {
        stack = stack.child(limit_message(env));
    } else if let Some(message) = ERROR.with(Signal::get) {
        stack = stack.child(
            TextProps::styled(message, TextVariant::TEXT_XS_NORMAL)
                .color(colors::TEXT_CRITICAL)
                .build(),
        );
    }
    FormItemProps::new(requiem_i18n::t!("invite-modal-send-link"), stack).build()
}

/// Web client parity: with Manage Server the message links to the invite list and the help
/// article, without it it points at admins.
fn limit_message(env: &HomeEnv) -> Node {
    let text = |content: String| {
        TextProps::styled(content, TextVariant::TEXT_XS_NORMAL)
            .color(colors::TEXT_CRITICAL)
            .build()
    };
    let guild = GUILD.with(Cell::get);
    let can_manage = guild.is_some_and(|guild| {
        env.state
            .guilds
            .find(guild)
            .is_some_and(|g| env.state.permissions.guild_permissions(&g).contains(Permissions::MANAGE_GUILD))
    });
    if !can_manage {
        return text(requiem_i18n::t!("invite-modal-limit-admins"));
    }
    let link = |label: String, on_click: Rc<dyn Fn()>| {
        TextLinkProps::new(label, move || on_click())
            .variant(TextLinkVariant::Anchor)
            .inline_variant(TextVariant::TEXT_XS_NORMAL)
            .build()
    };
    let state = env.state.clone();
    let list = link(
        requiem_i18n::t!("invite-modal-limit-view-invites"),
        Rc::new(move || {
            close();
            if let Some(guild) = guild {
                guild_settings::open(
                    &state,
                    guild,
                    guild_settings::INVITES_PAGE,
                );
            }
        }),
    );
    let help = link(
        requiem_i18n::t!("invite-modal-limit-learn-more"),
        Rc::new(|| link::open_url(INVITES_HELP_URL)),
    );
    div().class(cls::EXPIRY).children([
        text(requiem_i18n::t!("invite-modal-limit-reached")),
        list,
        help,
        text(requiem_i18n::t!("invite-modal-limit-how-invites-work")),
    ])
}

fn expiry_text(settings: InviteSettings) -> String {
    let uses = settings.max_uses;
    match settings.max_age {
        0 if uses == 0 => requiem_i18n::t!("invite-modal-never-expires"),
        0 => requiem_i18n::t!("invite-modal-expires-uses", uses = uses),
        1800 if uses == 0 => requiem_i18n::t!("invite-modal-expires-30-minutes"),
        1800 => requiem_i18n::t!("invite-modal-expires-30-minutes-uses", uses = uses),
        age if age < 86_400 => {
            let hours = age / 3600;
            if uses == 0 {
                requiem_i18n::t!("invite-modal-expires-hours", hours = hours)
            } else {
                requiem_i18n::t!(
                    "invite-modal-expires-hours-uses",
                    hours = hours,
                    uses = uses
                )
            }
        }
        age => {
            let days = age / 86_400;
            if uses == 0 {
                requiem_i18n::t!("invite-modal-expires-days", days = days)
            } else {
                requiem_i18n::t!("invite-modal-expires-days-uses", days = days, uses = uses)
            }
        }
    }
}

fn open_settings() {
    DRAFT.with(|d| d.set(SETTINGS.with(Signal::peek)));
    AGE_OPEN.with(|o| o.set(false));
    USES_OPEN.with(|o| o.set(false));
    PAGE.with(|p| p.set(Page::Settings));
}

const AGES: [u64; 7] = [1800, 3600, 21_600, 43_200, 86_400, 604_800, 0];
const USES: [u32; 7] = [0, 1, 5, 10, 25, 50, 100];

fn age_label(age: u64) -> String {
    match age {
        0 => requiem_i18n::t!("invite-modal-age-never"),
        1800 => requiem_i18n::t!("invite-modal-age-30-minutes"),
        age if age < 86_400 => requiem_i18n::t!("invite-modal-age-hours", hours = age / 3600),
        age => requiem_i18n::t!("invite-modal-age-days", days = age / 86_400),
    }
}

fn uses_label(uses: u32) -> String {
    if uses == 0 {
        requiem_i18n::t!("invite-modal-uses-unlimited")
    } else {
        requiem_i18n::t!("invite-modal-uses", uses = uses)
    }
}

fn settings_page(env: &HomeEnv) -> Node {
    let draft = DRAFT.with(Signal::get);
    let age_open = AGE_OPEN.with(Signal::get);
    let uses_open = USES_OPEN.with(Signal::get);

    let age = SelectProps::new(
        || AGE_OPEN.with(|o| o.set(!o.peek())),
        |value: String| {
            let age = value.parse().unwrap_or(604_800);
            DRAFT.with(|d| d.update(|s| s.max_age = age));
            AGE_OPEN.with(|o| o.set(false));
        },
    )
    .options(
        AGES.iter()
            .map(|&age| SelectOption::new(age.to_string(), age_label(age)))
            .collect::<Vec<_>>(),
    )
    .selected(draft.max_age.to_string())
    .open(age_open)
    .build();
    let uses = SelectProps::new(
        || USES_OPEN.with(|o| o.set(!o.peek())),
        |value: String| {
            let uses = value.parse().unwrap_or(0);
            DRAFT.with(|d| d.update(|s| s.max_uses = uses));
            USES_OPEN.with(|o| o.set(false));
        },
    )
    .options(
        USES.iter()
            .map(|&uses| SelectOption::new(uses.to_string(), uses_label(uses)))
            .collect::<Vec<_>>(),
    )
    .selected(draft.max_uses.to_string())
    .open(uses_open)
    .build();
    let temporary = SwitchProps::new(
        TEMPORARY_TOGGLE.with(|memo| memo.track(draft.temporary)),
        |on: bool| DRAFT.with(|d| d.update(|s| s.temporary = on)),
    )
    .build();

    let body = StackProps::vertical()
        .gap(20)
        .full_width(true)
        .build()
        .children([
            FormItemProps::new(requiem_i18n::t!("invite-modal-expire-after"), age).build(),
            FormItemProps::new(requiem_i18n::t!("invite-modal-max-uses"), uses).build(),
            FormItemProps::new(requiem_i18n::t!("invite-modal-temporary"), temporary)
                .description(requiem_i18n::t!("invite-modal-temporary-description"))
                .horizontal(true)
                .build(),
        ]);

    let cancel = ButtonProps::new(
        ButtonVariant::Secondary,
        requiem_i18n::t!("ui-cancel"),
        back_to_main,
    )
    .build();
    let generate_env = env.clone();
    let generate = ButtonProps::new(
        ButtonVariant::Primary,
        requiem_i18n::t!("invite-modal-generate"),
        move || generate(&generate_env),
    )
    .build();
    ModalProps::default()
        .size(ModalSize::Medium)
        .heading(requiem_i18n::t!("invite-modal-settings-title"))
        .dismissable(true)
        .on_dismiss(close)
        .open(DIALOG.with(Dialog::toggle))
        .body(body)
        .footer(vec![cancel, generate])
        .build()
}

fn back_to_main() {
    AGE_OPEN.with(|o| o.set(false));
    USES_OPEN.with(|o| o.set(false));
    PAGE.with(|p| p.set(Page::Main));
}

fn generate(env: &HomeEnv) {
    let settings = DRAFT.with(Signal::peek);
    if let Some(channel) = CHANNEL
        .with(|c| c.borrow().clone())
        .filter(|c| env.state.can(c, Permissions::CREATE_INSTANT_INVITE))
    {
        CODE.with(|c| c.set(vanity_placeholder()));
        request_invite(env, channel.id, settings, ValidateOwned::Omit);
    }
    back_to_main();
}

pub fn mount(
    tree: &mut WidgetTree,
    parent: WidgetId,
    env: Rc<dyn Fn() -> HomeEnv>,
    selected: Signal<Option<Id<ChannelMarker>>>,
) -> Effect {
    ENV.with(|e| *e.borrow_mut() = Some(env.clone()));
    SELECTED.with(|s| *s.borrow_mut() = Some(selected));
    section(tree, parent, move || {
        if DIALOG.with(Dialog::is_mounted) {
            dialog(&env())
        } else {
            div()
        }
    })
}
