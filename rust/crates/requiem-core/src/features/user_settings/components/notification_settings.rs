use chrono::TimeZone;
use crate::models::Permissions;
use crate::models;
use crate::rest::endpoints::user_guild_settings;
use crate::ui::classes::notification_settings;
use crate::ui::classes::notification_settings as cls;
use crate::ui::guild_settings;
use crate::ui;
use crate::utils::link;
use requiem_ui::colors;
use requiem_ui::signal as requiem_ui_signal;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use requiem_ui::components::{
    ButtonProps, ButtonVariant, ContextItem, FormItemProps, ModalProps, ModalSize, RadioGroupProps,
    RadioOption, SelectOption, SelectProps, SelectSearch, StackProps, SwitchProps, TextLinkProps,
    TextProps, TextVariant, checkbox_indicator, div, icon,
};
use requiem_ui::control::section;
use requiem_ui::node::Node;
use requiem_ui::signal::{Effect, Signal, signal};
use requiem_ui::widget::anim::{ToggleMemo, ToggleState};
use requiem_ui::widget::{WidgetId, WidgetTree};
use serde_json::{Value, json};

use crate::models::id::marker::{ChannelMarker, GuildMarker};
use crate::models::{Channel, Id};
use crate::state::{AppState, ChannelOverride, GuildSettings, Mute};
use crate::ui::context_menu as menu;
use crate::ui::dialog::Dialog;
use crate::ui::home::HomeEnv;
use requiem_ui::icons::CIRCLE_X;

const DURATIONS: [(i64, &str); 6] = [
    (900, "notification-settings-for-15-minutes"),
    (3600, "notification-settings-for-1-hour"),
    (10800, "notification-settings-for-3-hours"),
    (28800, "notification-settings-for-8-hours"),
    (86400, "notification-settings-for-24-hours"),
    (ALWAYS, "notification-settings-until-turned-back-on"),
];
const ALWAYS: i64 = -1;

const ALL: u8 = 0;
const MENTIONS: u8 = 1;
const NOTHING: u8 = 2;
const DEFAULT: u8 = 3;

const HIGHLIGHTS_DISABLED: u8 = 1;
const HIGHLIGHTS_ENABLED: u8 = 2;

const HIGHLIGHTS_ARTICLE: &str = "https://support.discord.com/hc/en-us/articles/5304469213079";

thread_local! {
    static DIALOG: Dialog = Dialog::new();
    static ENV: RefCell<Option<Rc<dyn Fn() -> HomeEnv>>> = const { RefCell::new(None) };

    static GUILD: Signal<Option<Id<GuildMarker>>> = signal(None);
    static PICKER_OPEN: Signal<bool> = signal(false);
    static PICKER_QUERY: Signal<Option<String>> = signal(None);
    static PICKER_ACTIVE: Signal<usize> = signal(0);
    static DURATION_OPEN: Signal<bool> = signal(false);

    static SWITCHES: RefCell<HashMap<&'static str, ToggleMemo>> = RefCell::new(HashMap::new());
}

fn env() -> Option<HomeEnv> {
    ENV.with(|e| e.borrow().clone()).map(|factory| factory())
}

fn save(guild_id: Id<GuildMarker>, changes: Value, local: impl FnOnce(&mut GuildSettings)) {
    let Some(env) = env() else { return };
    env.state.guild_settings.apply_local(guild_id, local);
    let (http, state) = (env.http.clone(), env.state.clone());
    guild_settings::run(
        &env.queries,
        move || async move {
            user_guild_settings::update(&http, guild_id, &changes).await
        },
        move |result| match result {
            Ok(entry) => state.guild_settings.replace(&entry),
            Err(_) => menu::notify_failure(&requiem_i18n::t!("notification-settings-save-failed")),
        },
    );
}

fn save_override(
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
    mut fields: Value,
    local: impl FnOnce(&mut ChannelOverride),
) {
    fields["channel_id"] = json!(channel_id.to_string());
    let changes = json!({ "channel_overrides": { channel_id.to_string(): fields } });
    save(guild_id, changes, move |settings| {
        local(settings.overrides.entry(channel_id).or_default())
    });
}

fn timed_mute(seconds: i64) -> (Value, Mute) {
    let end = (seconds > 0).then(|| chrono::Utc::now() + chrono::Duration::seconds(seconds));
    let config = json!({ "selected_time_window": seconds, "end_time": end.map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)) });
    (
        json!({ "muted": true, "mute_config": config }),
        Mute {
            muted: true,
            until_ms: end.map(|t| t.timestamp_millis()),
            window: Some(seconds),
        },
    )
}

pub fn mute_guild(guild_id: Id<GuildMarker>, seconds: Option<i64>) {
    match seconds {
        Some(seconds) => {
            let (changes, mute) = timed_mute(seconds);
            save(guild_id, changes, move |s| s.muted = mute);
        }
        None => save(guild_id, json!({ "muted": true }), |s| {
            s.muted = Mute {
                muted: true,
                ..Mute::default()
            }
        }),
    }
}

pub fn unmute_guild(guild_id: Id<GuildMarker>) {
    save(guild_id, json!({ "muted": false }), |s| {
        s.muted = Mute::default()
    });
}

pub fn set_guild_level(guild_id: Id<GuildMarker>, level: u8) {
    save(
        guild_id,
        json!({ "message_notifications": level }),
        move |s| s.message_notifications = level,
    );
}

fn set_suppress_everyone(guild_id: Id<GuildMarker>, on: bool) {
    save(guild_id, json!({ "suppress_everyone": on }), move |s| {
        s.suppress_everyone = on
    });
}

fn set_suppress_roles(guild_id: Id<GuildMarker>, on: bool) {
    save(guild_id, json!({ "suppress_roles": on }), move |s| {
        s.suppress_roles = on
    });
}

fn set_suppress_highlights(guild_id: Id<GuildMarker>, on: bool) {
    let value = if on {
        HIGHLIGHTS_DISABLED
    } else {
        HIGHLIGHTS_ENABLED
    };
    save(guild_id, json!({ "notify_highlights": value }), move |s| {
        s.notify_highlights = value
    });
}

fn set_mute_events(guild_id: Id<GuildMarker>, on: bool) {
    save(guild_id, json!({ "mute_scheduled_events": on }), move |s| {
        s.mute_scheduled_events = on
    });
}

fn set_mobile_push(guild_id: Id<GuildMarker>, on: bool) {
    save(guild_id, json!({ "mobile_push": on }), move |s| {
        s.mobile_push = on
    });
}

pub fn mute_channel(
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
    seconds: Option<i64>,
) {
    match seconds {
        Some(seconds) => {
            let (fields, mute) = timed_mute(seconds);
            save_override(guild_id, channel_id, fields, move |o| o.muted = mute);
        }
        None => save_override(guild_id, channel_id, json!({ "muted": true }), |o| {
            o.muted = Mute {
                muted: true,
                ..Mute::default()
            }
        }),
    }
}

pub fn unmute_channel(guild_id: Id<GuildMarker>, channel_id: Id<ChannelMarker>) {
    save_override(guild_id, channel_id, json!({ "muted": false }), |o| {
        o.muted = Mute::default()
    });
}

pub fn set_channel_level(guild_id: Id<GuildMarker>, channel_id: Id<ChannelMarker>, level: u8) {
    save_override(
        guild_id,
        channel_id,
        json!({ "message_notifications": level }),
        move |o| o.message_notifications = level,
    );
}

fn set_override_mute(
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
    level: u8,
    muted: bool,
) {
    save_override(
        guild_id,
        channel_id,
        json!({ "message_notifications": level, "muted": muted }),
        move |o| {
            o.message_notifications = level;
            o.muted = Mute {
                muted,
                ..Mute::default()
            };
        },
    );
}

fn remove_override(guild_id: Id<GuildMarker>, channel_id: Id<ChannelMarker>) {
    set_override_mute(guild_id, channel_id, DEFAULT, false);
}

fn guild_default(state: &AppState, guild_id: Id<GuildMarker>) -> u8 {
    state
        .guilds
        .find(guild_id)
        .map_or(ALL, |g| g.default_message_notifications)
}

fn level_label(level: u8) -> String {
    match level {
        MENTIONS => requiem_i18n::t!("notification-settings-only-mentions-plain"),
        NOTHING => requiem_i18n::t!("notification-settings-nothing"),
        _ => requiem_i18n::t!("notification-settings-all-messages"),
    }
}

fn muted_until(until_ms: i64) -> Option<String> {
    let time = chrono::Local.timestamp_millis_opt(until_ms).single()?;
    let text = if requiem_i18n::locale().starts_with("de") {
        time.format("%-d.%-m., %H:%M").to_string()
    } else {
        time.format("%-m/%-d, %-I:%M %p").to_string()
    };
    Some(requiem_i18n::t!(
        "notification-settings-muted-until",
        time = text
    ))
}

fn sidebar_order(state: &AppState, guild_id: Id<GuildMarker>) -> Vec<Channel> {
    let mut channels: Vec<Channel> = state
        .channels
        .by_guild
        .with(|all| all.get(&guild_id).cloned().unwrap_or_default());
    channels.retain(|c| {
        state.can(c, Permissions::VIEW_CHANNEL) && !matches!(c.kind, 10..=12)
    });
    channels.sort_by_key(|c| c.position.unwrap_or(0));
    let mut out: Vec<Channel> = channels
        .iter()
        .filter(|c| c.kind != 4 && c.parent_id.is_none())
        .cloned()
        .collect();
    for category in channels.iter().filter(|c| c.kind == 4) {
        out.push(category.clone());
        out.extend(
            channels
                .iter()
                .filter(|c| c.parent_id == Some(category.id))
                .cloned(),
        );
    }
    out
}

fn is_voice(channel: &Channel) -> bool {
    matches!(channel.kind, 2 | 13)
}

fn duration_items(on_pick: impl Fn(i64) + Clone + 'static) -> Vec<ContextItem> {
    DURATIONS
        .iter()
        .map(|&(seconds, key)| {
            let pick = on_pick.clone();
            ContextItem::action(requiem_i18n::t!(key), move || pick(seconds))
        })
        .collect()
}

pub fn guild_items(state: &AppState, guild_id: Id<GuildMarker>) -> Vec<ContextItem> {
    requiem_ui_signal::untrack(|| guild_items_now(state, guild_id))
}

fn guild_items_now(state: &AppState, guild_id: Id<GuildMarker>) -> Vec<ContextItem> {
    let mute = match state.guild_settings.guild_mute(guild_id) {
        Some(mute) => {
            let item = ContextItem::action(
                requiem_i18n::t!("notification-settings-unmute-server"),
                move || unmute_guild(guild_id),
            );
            match mute.until_ms.and_then(muted_until) {
                Some(until) => item.subtitle(until),
                None => item,
            }
        }
        None => ContextItem::action(
            requiem_i18n::t!("notification-settings-mute-server"),
            move || mute_guild(guild_id, None),
        )
        .submenu(move || duration_items(move |seconds| mute_guild(guild_id, Some(seconds))))
        .parent_action(),
    };
    let level = state
        .guild_settings
        .guild_level(guild_id, guild_default(state, guild_id));
    let state_for_menu = state.clone();
    let notifications = ContextItem::action(
        requiem_i18n::t!("context-menus-notification-settings"),
        move || open(guild_id),
    )
    .subtitle(level_label(level))
    .submenu(move || guild_notification_items(&state_for_menu, guild_id))
    .parent_action();
    vec![mute, notifications]
}

fn guild_notification_items(state: &AppState, guild_id: Id<GuildMarker>) -> Vec<ContextItem> {
    let settings = state.guild_settings.of(guild_id);
    let level = state
        .guild_settings
        .guild_level(guild_id, guild_default(state, guild_id));
    let muted = state.guild_settings.guild_mute(guild_id).is_some();
    let radio = |label: String, value: u8| {
        ContextItem::action(label, move || set_guild_level(guild_id, value))
            .radio(level == value)
            .keep_open()
            .markup()
    };
    let check = |label: String, on: bool, toggle: fn(Id<GuildMarker>, bool)| {
        ContextItem::action(label, move || toggle(guild_id, !on))
            .checkbox(on)
            .keep_open()
            .markup()
    };
    vec![
        radio(requiem_i18n::t!("notification-settings-all-messages"), ALL),
        radio(
            requiem_i18n::t!("notification-settings-only-mentions"),
            MENTIONS,
        ),
        radio(requiem_i18n::t!("notification-settings-nothing"), NOTHING),
        ContextItem::Separator,
        check(
            requiem_i18n::t!("notification-settings-suppress-everyone"),
            settings.suppress_everyone,
            set_suppress_everyone,
        ),
        check(
            requiem_i18n::t!("notification-settings-suppress-roles"),
            settings.suppress_roles,
            set_suppress_roles,
        ),
        check(
            requiem_i18n::t!("notification-settings-suppress-highlights"),
            muted || settings.notify_highlights == HIGHLIGHTS_DISABLED,
            set_suppress_highlights,
        ),
        check(
            requiem_i18n::t!("notification-settings-mute-events"),
            settings.mute_scheduled_events,
            set_mute_events,
        ),
        ContextItem::Separator,
        check(
            requiem_i18n::t!("notification-settings-mobile-push"),
            settings.mobile_push,
            set_mobile_push,
        ),
    ]
}

pub fn channel_items(
    state: &AppState,
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
) -> Vec<ContextItem> {
    requiem_ui_signal::untrack(|| channel_items_now(state, guild_id, channel_id))
}

fn channel_items_now(
    state: &AppState,
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
) -> Vec<ContextItem> {
    let Some((_, channel)) = ui::find_channel(state, channel_id) else {
        return Vec::new();
    };
    let category = channel.kind == 4;
    let mute = match state.guild_settings.channel_mute(guild_id, channel_id) {
        Some(mute) => {
            let label = if category {
                requiem_i18n::t!("notification-settings-unmute-category")
            } else {
                requiem_i18n::t!("notification-settings-unmute-channel")
            };
            let item = ContextItem::action(label, move || unmute_channel(guild_id, channel_id));
            match mute.until_ms.and_then(muted_until) {
                Some(until) => item.subtitle(until),
                None => item,
            }
        }
        None => {
            let label = if category {
                requiem_i18n::t!("notification-settings-mute-category")
            } else {
                requiem_i18n::t!("notification-settings-mute-channel")
            };
            ContextItem::action(label, move || mute_channel(guild_id, channel_id, None))
                .submenu(move || {
                    duration_items(move |seconds| mute_channel(guild_id, channel_id, Some(seconds)))
                })
                .parent_action()
        }
    };
    let effective = state.guild_settings.resolved_message_notifications(
        Some(guild_id),
        channel_id,
        channel.parent_id,
        guild_default(state, guild_id),
    );
    let state_for_menu = state.clone();
    let notifications = ContextItem::action(
        requiem_i18n::t!("context-menus-notification-settings"),
        || {},
    )
    .subtitle(level_label(effective))
    .submenu(move || channel_notification_items(&state_for_menu, guild_id, channel_id));
    vec![mute, notifications]
}

fn channel_notification_items(
    state: &AppState,
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
) -> Vec<ContextItem> {
    let Some((_, channel)) = ui::find_channel(state, channel_id) else {
        return Vec::new();
    };
    let own = state
        .guild_settings
        .channel_override(guild_id, channel_id)
        .message_notifications;
    let effective = state.guild_settings.resolved_message_notifications(
        Some(guild_id),
        channel_id,
        channel.parent_id,
        guild_default(state, guild_id),
    );
    let radio = |label: String, value: u8| {
        ContextItem::action(label, move || {
            set_channel_level(guild_id, channel_id, value)
        })
        .radio(own == value)
        .keep_open()
        .markup()
    };
    let default_label = if channel.parent_id.is_some() {
        requiem_i18n::t!("notification-settings-use-category-default")
    } else {
        requiem_i18n::t!("notification-settings-use-server-default")
    };

    let mut default = radio(default_label, DEFAULT);
    if own == DEFAULT {
        default = default.subtitle(level_label(effective));
    }
    let mut items = vec![default];
    if !is_voice(&channel) {
        items.push(radio(
            requiem_i18n::t!("notification-settings-all-messages"),
            ALL,
        ));
    }
    items.push(radio(
        requiem_i18n::t!("notification-settings-only-mentions"),
        MENTIONS,
    ));
    items.push(radio(
        requiem_i18n::t!("notification-settings-nothing"),
        NOTHING,
    ));
    items
}

pub fn open(guild_id: Id<GuildMarker>) {
    GUILD.with(|g| g.set(Some(guild_id)));
    close_pickers();
    SWITCHES.with(|s| s.borrow_mut().clear());
    DIALOG.with(Dialog::open);
}

fn close() {
    close_pickers();
    DIALOG.with(Dialog::close);
}

fn close_pickers() {
    PICKER_OPEN.with(|o| o.set(false));
    PICKER_QUERY.with(|q| q.set(None));
    PICKER_ACTIVE.with(|a| a.set(0));
    DURATION_OPEN.with(|o| o.set(false));
}

fn switch_state(key: &'static str, on: bool) -> ToggleState {
    SWITCHES.with(|s| {
        s.borrow_mut()
            .entry(key)
            .or_insert_with(|| ToggleMemo::new(on))
            .track(on)
    })
}

fn switch_field(
    key: &'static str,
    label: String,
    description: Option<String>,
    on: bool,
    disabled: bool,
    on_change: impl Fn(bool) + 'static,
) -> Node {
    let switch = SwitchProps::new(switch_state(key, on), on_change)
        .disabled(disabled)
        .build();
    let mut field = FormItemProps::new(label, switch)
        .horizontal(true)
        .markup(true);
    if let Some(description) = description {
        field = field.description(description);
    }
    field.build()
}

fn divider() -> Node {
    div().class(notification_settings::DIVIDER)
}

fn dialog(env: &HomeEnv) -> Node {
    let Some(guild_id) = GUILD.with(Signal::get) else {
        return div();
    };
    let Some(guild) = env.state.guilds.find(guild_id) else {
        return div();
    };
    let state = &env.state;
    let settings = state.guild_settings.of(guild_id);
    let mute = state.guild_settings.guild_mute(guild_id);
    let level = state
        .guild_settings
        .guild_level(guild_id, guild.default_message_notifications);

    let mut mute_stack = StackProps::vertical()
        .gap(20)
        .full_width(true)
        .build()
        .child(switch_field(
            "mute",
            requiem_i18n::t!(
                "notification-settings-mute-guild",
                name = guild.name.as_str()
            ),
            Some(requiem_i18n::t!("notification-settings-mute-guild-note")),
            mute.is_some(),
            false,
            move |on| {
                if on {
                    mute_guild(guild_id, None)
                } else {
                    unmute_guild(guild_id)
                }
            },
        ));
    if let Some(mute) = mute {
        let select = SelectProps::new(
            || DURATION_OPEN.with(|o| o.set(!o.peek())),
            move |value: String| {
                DURATION_OPEN.with(|o| o.set(false));
                if let Ok(seconds) = value.parse() {
                    mute_guild(guild_id, Some(seconds));
                }
            },
        )
        .options(
            DURATIONS
                .iter()
                .map(|&(seconds, key)| {
                    SelectOption::new(seconds.to_string(), requiem_i18n::t!(key))
                })
                .collect::<Vec<_>>(),
        )
        .selected(mute.window.unwrap_or(ALWAYS).to_string())
        .open(DURATION_OPEN.with(Signal::get))
        .build();
        let mut field = FormItemProps::new(
            requiem_i18n::t!("notification-settings-mute-duration"),
            select,
        );
        if let Some(until) = mute.until_ms.and_then(muted_until) {
            field = field.description(until);
        }
        mute_stack = mute_stack.child(field.build());
    }
    let mute_section = div()
        .class(cls::LARGE_FIRST)
        .child(mute_stack)
        .child(divider().class(cls::DIVIDER_SPACED));

    let option = |value: u8, key: &str| {
        RadioOption::new(
            value.to_string(),
            requiem_i18n::t!(key),
            ToggleState::new(level == value),
        )
        .markup()
    };
    let radios = RadioGroupProps::new(move |value: String| {
        if let Ok(value) = value.parse() {
            set_guild_level(guild_id, value);
        }
    })
    .options(vec![
        option(ALL, "notification-settings-all-messages"),
        option(MENTIONS, "notification-settings-only-mentions"),
        option(NOTHING, "notification-settings-nothing"),
    ])
    .build();
    let levels = div()
        .class(cls::SPACING)
        .child(
            div().class(cls::LARGE).child(
                FormItemProps::new(
                    requiem_i18n::t!("notification-settings-server-settings"),
                    radios,
                )
                .build(),
            ),
        )
        .child(divider());

    let highlights = StackProps::vertical()
        .gap(4)
        .full_width(true)
        .build()
        .child(switch_field(
            "highlights",
            requiem_i18n::t!("notification-settings-suppress-highlights"),
            Some(requiem_i18n::t!(
                "notification-settings-highlights-description"
            )),
            mute.is_some() || settings.notify_highlights == HIGHLIGHTS_DISABLED,
            mute.is_some(),
            move |on| set_suppress_highlights(guild_id, on),
        ))
        .child(
            TextLinkProps::new(
                requiem_i18n::t!("notification-settings-highlights-learn-more"),
                || link::open_url(HIGHLIGHTS_ARTICLE),
            )
            .small(true)
            .build(),
        );
    let switches = StackProps::vertical()
        .gap(16)
        .full_width(true)
        .build()
        .class(cls::LARGE)
        .children([
            switch_field(
                "everyone",
                requiem_i18n::t!("notification-settings-suppress-everyone"),
                None,
                settings.suppress_everyone,
                false,
                move |on| set_suppress_everyone(guild_id, on),
            ),
            divider(),
            switch_field(
                "roles",
                requiem_i18n::t!("notification-settings-suppress-roles"),
                None,
                settings.suppress_roles,
                false,
                move |on| set_suppress_roles(guild_id, on),
            ),
            divider(),
            highlights,
            divider(),
            switch_field(
                "events",
                requiem_i18n::t!("notification-settings-mute-events"),
                None,
                settings.mute_scheduled_events,
                false,
                move |on| set_mute_events(guild_id, on),
            ),
            divider(),
            switch_field(
                "push",
                requiem_i18n::t!("notification-settings-mobile-push"),
                None,
                settings.mobile_push,
                false,
                move |on| set_mobile_push(guild_id, on),
            ),
        ]);

    let order = sidebar_order(state, guild_id);
    let overridden: Vec<(Channel, ChannelOverride)> = order
        .iter()
        .filter_map(|c| {
            settings
                .overrides
                .get(&c.id)
                .filter(|o| o.is_set())
                .map(|o| (c.clone(), o.clone()))
        })
        .collect();
    let picker = override_picker(state, guild_id, &order, &overridden);
    let add = div().class(cls::LARGE).child(
        FormItemProps::new(
            requiem_i18n::t!("notification-settings-select-channel"),
            picker,
        )
        .description(requiem_i18n::t!("notification-settings-add-override"))
        .build(),
    );
    let table = override_table(state, guild_id, &guild, &overridden);

    let body = div()
        .class(cls::BODY)
        .children([mute_section, levels, switches, add, table]);
    let done = ButtonProps::new(
        ButtonVariant::Primary,
        requiem_i18n::t!("notification-settings-done"),
        close,
    )
    .full_width(true)
    .build();
    ModalProps::default()
        .size(ModalSize::Medium)
        .heading(requiem_i18n::t!("notification-settings-title"))
        .dismissable(true)
        .on_dismiss(close)
        .open(DIALOG.with(Dialog::toggle))
        .scroll_body(true)
        .body(body)
        .footer(vec![done])
        .build()
}

fn override_picker(
    state: &AppState,
    guild_id: Id<GuildMarker>,
    order: &[Channel],
    overridden: &[(Channel, ChannelOverride)],
) -> Node {
    let categories: HashMap<Id<ChannelMarker>, String> = order
        .iter()
        .filter(|c| c.kind == 4)
        .map(|c| (c.id, c.name.clone().unwrap_or_default()))
        .collect();
    let options: Vec<SelectOption> = order
        .iter()
        .filter(|c| !overridden.iter().any(|(o, _)| o.id == c.id))
        .map(|c| {
            let glyph = icon(
                ui::channel_icon(state, c),
                colors::TEXT_MUTED,
            )
            .class(cls::OPTION_ICON);
            let mut option =
                SelectOption::new(c.id.to_string(), c.name.clone().unwrap_or_default())
                    .with_leading(glyph);
            if let Some(category) = c.parent_id.and_then(|p| categories.get(&p)) {
                option = option.with_trailing(
                    TextProps::styled(category.to_uppercase(), TextVariant::TEXT_XS_SEMIBOLD)
                        .color(colors::TEXT_MUTED)
                        .line_clamp(1_u32)
                        .build(),
                );
            }
            option
        })
        .collect();
    let search = SelectSearch {
        query: PICKER_QUERY.with(Signal::get),
        active: PICKER_ACTIVE.with(Signal::get),
        on_query: Rc::new(|typed| {
            PICKER_QUERY.with(|q| q.set(Some(typed)));
            PICKER_ACTIVE.with(|a| a.set(0));
        }),
        on_active: Rc::new(|row| PICKER_ACTIVE.with(|a| a.set(row))),
    };
    let state = state.clone();
    SelectProps::new(
        || {
            let open = PICKER_OPEN.with(|o| o.peek());
            PICKER_OPEN.with(|o| o.set(!open));
            PICKER_QUERY.with(|q| q.set(None));
        },
        move |value: String| {
            PICKER_OPEN.with(|o| o.set(false));
            PICKER_QUERY.with(|q| q.set(None));
            let Some(channel_id) = value
                .parse::<u64>()
                .ok()
                .filter(|id| *id != 0)
                .map(Id::<ChannelMarker>::new)
            else {
                return;
            };
            let Some((_, channel)) = ui::find_channel(&state, channel_id) else {
                return;
            };
            let level = state.guild_settings.resolved_message_notifications(
                Some(guild_id),
                channel_id,
                channel.parent_id,
                guild_default(&state, guild_id),
            );
            set_override_mute(guild_id, channel_id, level, false);
        },
    )
    .search(search)
    .options(options)
    .open(PICKER_OPEN.with(Signal::get))
    .build()
}

fn override_table(
    state: &AppState,
    guild_id: Id<GuildMarker>,
    guild: &models::Guild,
    overridden: &[(Channel, ChannelOverride)],
) -> Node {
    let header = |text: String| {
        TextProps::styled(text, TextVariant::EYEBROW)
            .color(colors::TEXT_DEFAULT)
            .build()
    };
    let mut headers = div()
        .class(cls::HEADERS)
        .child(div().class(cls::HEADER_NAME).child(header(requiem_i18n::t!(
            "notification-settings-channel-or-category"
        ))));
    for key in [
        "notification-settings-column-all",
        "notification-settings-column-mentions",
        "notification-settings-column-nothing",
        "notification-settings-column-mute",
    ] {
        headers = headers.child(
            div()
                .class(cls::HEADER_OPTION)
                .child(header(requiem_i18n::t!(key))),
        );
    }
    let mut table = div().class(cls::TABLE).child(headers);
    if overridden.is_empty() {
        let text = TextProps::styled(
            requiem_i18n::t!("notification-settings-add-override"),
            TextVariant::TEXT_SM_SEMIBOLD,
        )
        .color(colors::TEXT_DEFAULT)
        .center(true)
        .build()
        .class(cls::LIGHTEN);
        return table.child(div().class(cls::PLACEHOLDER).child(text));
    }
    let categories: HashMap<Id<ChannelMarker>, String> = state.channels.by_guild.with(|all| {
        all.get(&guild_id)
            .map(|cs| {
                cs.iter()
                    .filter(|c| c.kind == 4)
                    .map(|c| (c.id, c.name.clone().unwrap_or_default()))
                    .collect()
            })
            .unwrap_or_default()
    });
    for (channel, entry) in overridden {
        let effective = state.guild_settings.resolved_message_notifications(
            Some(guild_id),
            channel.id,
            channel.parent_id,
            guild.default_message_notifications,
        );
        let level = if entry.message_notifications == DEFAULT {
            effective
        } else {
            entry.message_notifications
        };
        table = table.child(override_row(
            state,
            guild_id,
            channel,
            level,
            entry.muted.muted,
            categories
                .get(&channel.parent_id.unwrap_or(channel.id))
                .filter(|_| channel.parent_id.is_some())
                .cloned(),
        ));
    }
    table
}

fn override_row(
    state: &AppState,
    guild_id: Id<GuildMarker>,
    channel: &Channel,
    level: u8,
    muted: bool,
    category: Option<String>,
) -> Node {
    let channel_id = channel.id;
    let name = channel.name.clone().unwrap_or_default();
    let byline = category.unwrap_or_else(|| requiem_i18n::t!("notification-settings-no-category"));
    let names = div()
        .class(cls::NAME_TEXT)
        .child(
            TextProps::styled(name.clone(), TextVariant::TEXT_MD_SEMIBOLD)
                .color(colors::TEXT_STRONG)
                .line_clamp(1_u32)
                .build(),
        )
        .child(
            TextProps::styled(byline, TextVariant::TEXT_XS_NORMAL)
                .color(colors::TEXT_DEFAULT)
                .line_clamp(1_u32)
                .build()
                .class(cls::LIGHTEN),
        );
    let name_cell = div()
        .class(cls::NAME_CELL)
        .child(
            icon(
                ui::channel_icon(state, channel),
                colors::TEXT_MUTED,
            )
            .class(cls::ROW_ICON)
            .class(cls::LIGHTEN),
        )
        .child(names);

    let check = |on: bool, on_click: Box<dyn Fn()>, mute_column: bool| {
        let button = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::CHECK_BUTTON.to_string()])
            .on_click(on_click)
            .build()
            .child(checkbox_indicator(ToggleState::new(on)));
        let mut cell = div()
            .class(cls::CHECK_CELL)
            .child(div().class(if mute_column {
                cls::MUTE_SHADE
            } else {
                cls::COLUMN_LINE
            }));
        if muted && !mute_column {
            cell = cell.class(cls::CHECK_CELL_MUTED);
        }
        cell.child(button)
    };
    let mut boxes = div().class(cls::CHECKS);
    for value in [ALL, MENTIONS, NOTHING] {
        boxes = boxes.child(check(
            level == value,
            Box::new(move || set_channel_level(guild_id, channel_id, value)),
            false,
        ));
    }
    boxes = boxes.child(check(
        muted,
        Box::new(move || set_override_mute(guild_id, channel_id, level, !muted)),
        true,
    ));

    let remove = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::REMOVE.to_string()])
        .on_click(move || remove_override(guild_id, channel_id))
        .build()
        .child(icon(CIRCLE_X, colors::TEXT_SECONDARY).class(cls::REMOVE_ICON));
    div()
        .class(cls::ROW)
        .child(name_cell)
        .child(boxes)
        .child(remove)
}

pub fn mount(tree: &mut WidgetTree, parent: WidgetId, env: Rc<dyn Fn() -> HomeEnv>) -> Effect {
    ENV.with(|e| *e.borrow_mut() = Some(env.clone()));
    section(tree, parent, move || {
        if DIALOG.with(Dialog::is_mounted) {
            dialog(&env())
        } else {
            div()
        }
    })
}
