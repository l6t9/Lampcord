use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};

use requiem_plugin::prelude::*;
use requiem_plugin::ui::{
    avatar, empty_state, icon, image, open_modal, profile, refresh, scroll, section, tab, toast,
};
use requiem_plugin::{
    Channel, Guild, Member, Role, actions, client, navigation, registry, slots, timers,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

requiem_plugin::classes!();
use classes::card as cls;

const ICON: &str = "CircleInformationIcon";
const MENU_ID: &str = "server-info";
const HEADER_SLOT: &str = "guild-header.menu";

const MAX_LOOKUPS: usize = 200;

const PER_TICK: usize = 3;
const TICK_MS: u64 = 150;

const TAB_ROLES: &str = "roles";
const TAB_IMAGES: &str = "images";
const TAB_FRIENDS: &str = "friends";
const TAB_BLOCKED: &str = "blocked";

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct Settings {
    show_boosts: bool,
    show_channels: bool,
    show_permissions: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            show_boosts: true,
            show_channels: true,
            show_permissions: true,
        }
    }
}

#[derive(Clone)]
struct Person {
    id: u64,
    name: String,
    avatar: Option<String>,
}

#[derive(Default)]
struct Social {
    friends: Vec<Person>,
    blocked: Vec<Person>,

    queue: VecDeque<(u64, bool)>,

    skipped: usize,

    saw_relationships: bool,
}

thread_local! {
    static TABS: RefCell<HashMap<u64, String>> = RefCell::new(HashMap::new());
    static SOCIAL: RefCell<HashMap<u64, Social>> = RefCell::new(HashMap::new());

    static EXPANDED: RefCell<HashSet<(u64, u64)>> = RefCell::new(HashSet::new());
}

struct ServerInfo;

impl Plugin for ServerInfo {
    fn init(_previous_version: Option<String>) -> Result<(), Error> {
        let item = registry::MenuItem::new("guild", MENU_ID, "Server Info");
        item.set_icon(ICON);
        registry::register_menu_item(item)?;

        let slot = slots::SlotItem::new(HEADER_SLOT, MENU_ID);
        slot.set_label("Server Info");
        slot.set_icon(ICON);
        if let Err(err) = slot.commit() {
            log::info(&format!(
                "no menu entry under the server name: {}",
                err.message
            ));
        }
        Ok(())
    }

    fn on_menu_item(menu: &str, item_id: &str, target: &str, _checked: Option<bool>) {
        if menu == "guild" && item_id == MENU_ID {
            open_profile(target);
        }
    }

    fn on_slot_action(slot: &str, item_id: &str, context: &str) {
        if slot == HEADER_SLOT && item_id == MENU_ID {
            open_profile(context);
        }
    }

    fn on_timer(id: &str) {
        if let Some(guild_id) = id
            .strip_prefix("social:")
            .and_then(|g| g.parse::<u64>().ok())
        {
            look_up_next(guild_id);
        }
    }

    fn view(view_id: &str, _context: &str) -> Node {
        match view_id
            .strip_prefix("guild:")
            .and_then(|id| id.parse::<u64>().ok())
            .and_then(client::get_guild)
        {
            Some(guild) => server_profile(&guild),
            None => div().child(
                TextProps::styled("This server isn't available.", TextVariant::TEXT_SM_NORMAL)
                    .color("var(--text-muted)")
                    .build(),
            ),
        }
    }

    fn settings_page() -> Node {
        let current = settings::load::<Settings>();
        div()
            .class(cls::PAGE)
            .child(form::switch_row(
                "Boosts",
                Some("Show the server's Boost level and count on its card."),
                current.show_boosts,
                |on| settings::update::<Settings>(|s| s.show_boosts = on),
            ))
            .child(form::switch_row(
                "Channels",
                Some("Show how many text channels, voice channels and categories there are."),
                current.show_channels,
                |on| settings::update::<Settings>(|s| s.show_channels = on),
            ))
            .child(form::switch_row(
                "Role permissions",
                Some("Show what each role may do on the Roles tab, not only its name."),
                current.show_permissions,
                |on| settings::update::<Settings>(|s| s.show_permissions = on),
            ))
    }
}

fn open_profile(context: &str) {
    let Some(guild_id) = serde_json::from_str::<Value>(context)
        .ok()
        .and_then(|v| v["guild-id"].as_str().and_then(|id| id.parse::<u64>().ok()))
    else {
        log::warn(&format!("no server in {context}"));
        return;
    };
    TABS.with(|tabs| tabs.borrow_mut().remove(&guild_id));
    EXPANDED.with(|open| open.borrow_mut().retain(|(guild, _)| *guild != guild_id));
    find_people(guild_id);
    let name = client::get_guild(guild_id)
        .map(|g| g.name())
        .unwrap_or_default();
    if let Err(err) = open_modal(&format!("guild:{guild_id}"), &name, "profile") {
        toast(
            &format!("Couldn't open Server Info: {}", err.message),
            "failure",
        );
    }
}

fn find_people(guild_id: u64) {
    let mut social = Social::default();
    for relationship in client::relationships() {
        social.saw_relationships = true;
        let kind = relationship.kind();
        let friend = kind == "friend";
        if !friend && kind != "blocked" {
            continue;
        }
        let user = relationship.user();
        let id = user.id();
        match client::get_member(guild_id, id) {
            Some(member) => add_person(&mut social, person(&member), friend),
            None => social.queue.push_back((id, friend)),
        }
    }
    if social.queue.len() > MAX_LOOKUPS {
        social.skipped = social.queue.len() - MAX_LOOKUPS;
        social.queue.truncate(MAX_LOOKUPS);
    }
    let busy = !social.queue.is_empty();
    SOCIAL.with(|all| all.borrow_mut().insert(guild_id, social));
    let timer = format!("social:{guild_id}");
    if busy {
        timers::set(&timer, TICK_MS, true);
    } else {
        timers::clear(&timer);
    }
}

fn look_up_next(guild_id: u64) {
    let timer = format!("social:{guild_id}");
    let batch: Vec<(u64, bool)> = SOCIAL.with(|all| {
        let mut all = all.borrow_mut();
        let Some(social) = all.get_mut(&guild_id) else {
            return Vec::new();
        };
        (0..PER_TICK)
            .filter_map(|_| social.queue.pop_front())
            .collect()
    });
    for (user_id, friend) in batch {
        match client::fetch_profile(user_id, Some(guild_id)) {
            Ok(found) => {
                if let Some(member) = found.member() {
                    SOCIAL.with(|all| {
                        if let Some(social) = all.borrow_mut().get_mut(&guild_id) {
                            add_person(social, person(&member), friend);
                        }
                    });
                }
            }

            Err(err) if err.code == "permission-denied" => SOCIAL.with(|all| {
                if let Some(social) = all.borrow_mut().get_mut(&guild_id) {
                    social.queue.clear();
                }
            }),

            Err(_) => {}
        }
    }
    if SOCIAL.with(|all| {
        all.borrow()
            .get(&guild_id)
            .is_none_or(|s| s.queue.is_empty())
    }) {
        timers::clear(&timer);
    }
    refresh(&format!("guild:{guild_id}"));
}

fn person(member: &Member) -> Person {
    let user = member.user();
    Person {
        id: user.id(),
        name: member.display_name(),
        avatar: user.avatar_url(80),
    }
}

fn add_person(social: &mut Social, person: Person, friend: bool) {
    let list = if friend {
        &mut social.friends
    } else {
        &mut social.blocked
    };
    if !list.iter().any(|p| p.id == person.id) {
        list.push(person);
        list.sort_by_key(|p| p.name.to_lowercase());
    }
}

fn server_profile(guild: &Guild) -> Node {
    let id = guild.id();
    let name = guild.name();
    let settings = settings::load::<Settings>();
    let selected = TABS
        .with(|tabs| tabs.borrow().get(&id).cloned())
        .unwrap_or_else(|| TAB_ROLES.to_string());
    let (friends, blocked, checking, skipped, saw_relationships) = SOCIAL.with(|all| {
        let all = all.borrow();
        match all.get(&id) {
            Some(s) => (
                s.friends.clone(),
                s.blocked.clone(),
                s.queue.len(),
                s.skipped,
                s.saw_relationships,
            ),
            None => (Vec::new(), Vec::new(), 0, 0, false),
        }
    });

    let mut card = profile(guild.name())
        .prop("stats", stats(guild))
        .prop("tab", selected)
        .on("tab", move |value| {
            if let Some(Value::String(tab)) = value {
                TABS.with(|tabs| tabs.borrow_mut().insert(id, tab));
            }
        });
    if let Some(url) = guild.banner_url(600) {
        card = card.prop("banner", url);

        if let Some(full) = guild.banner_url(2048) {
            card = card.prop("banner-full", full);
        }
    }
    if let Some(url) = guild.icon_url(256) {
        card = card.prop("icon", url);

        if let Some(full) = guild.icon_url(1024) {
            card = card.prop("icon-full", full);
        }
    }
    let badges = feature_badges(&guild.features());
    if !badges.is_empty() {
        card = card.prop("badges", badges);
    }

    if let Some(description) = guild.description() {
        card = card.child(
            TextProps::styled(description, TextVariant::TEXT_SM_NORMAL)
                .color("var(--text-default)")
                .build(),
        );
    }
    let created_ms = (id >> 22) as i64 + 1_420_070_400_000;

    let joined = client::current_user()
        .and_then(|me| client::get_member(id, me.id()))
        .and_then(|member| member.joined_at());
    card = card.child(since_row(created_ms, joined));
    card = card.child(section("Owner", owner(guild)));
    if settings.show_boosts {
        let (tier, boosts) = (guild.boost_tier(), guild.boost_count());
        let line = if boosts == 0 {
            "No boosts yet".to_string()
        } else {
            format!("{boosts} {}", if boosts == 1 { "boost" } else { "boosts" })
        };
        let text = TextProps::styled(line, TextVariant::TEXT_SM_NORMAL)
            .color("var(--text-default)")
            .build();

        card = card.child(section(
            "Server Boosts",
            if boosts == 0 {
                text
            } else {
                text.tooltip(format!("Level {tier}"))
            },
        ));
    }
    let channels = client::channels(id);
    if settings.show_channels {
        card = card.child(section("Channels", channel_counts(&channels)));
    }

    let id_row = div()
        .class(cls::ID_ROW)
        .tooltip("Click to copy")
        .on_click(move || copy(&id.to_string(), "Server ID copied"))
        .child(
            TextProps::styled(id.to_string(), TextVariant::TEXT_SM_NORMAL)
                .color("var(--text-default)")
                .build(),
        );
    card = card.child(section("Server ID", id_row));

    let busy = if checking > 0 { "…" } else { "" };
    let roles = client::roles(id);
    card = card.child(tab(
        TAB_ROLES,
        &count_label(roles.len(), "Role", "Roles", ""),
        roles_tab(id, roles, settings.show_permissions),
    ));
    card = card.child(tab(TAB_IMAGES, "Images", images_tab(guild)));
    card = card.child(tab(
        TAB_FRIENDS,
        &count_label(friends.len(), "Friend", "Friends", busy),
        people_tab(
            id,
            &name,
            &friends,
            "friends",
            checking,
            skipped,
            saw_relationships,
        ),
    ));
    card.child(tab(
        TAB_BLOCKED,
        &count_label(blocked.len(), "Blocked User", "Blocked Users", busy),
        people_tab(
            id,
            &name,
            &blocked,
            "blocked users",
            checking,
            skipped,
            saw_relationships,
        ),
    ))
}

fn stats(guild: &Guild) -> Value {
    let mut stats = Vec::new();
    if let Some(online) = guild.online_count() {
        stats.push(
            serde_json::json!({ "text": format!("{} Online", thousands(online)), "dot": "online" }),
        );
    }
    if let Some(members) = guild.member_count() {
        stats.push(serde_json::json!({ "text": format!("{} {}", thousands(members), if members == 1 { "Member" } else { "Members" }), "dot": "offline" }));
    }
    Value::Array(stats)
}

fn owner(guild: &Guild) -> Node {
    match client::get_member(guild.id(), guild.owner_id()) {
        Some(member) => {
            let user = member.user();
            let name = member.display_name();
            let picture = avatar(user.avatar_url(64).as_deref(), &name, 32, None);

            let (guild_id, owner_id) = (guild.id(), user.id());
            div()
                .class(cls::OWNER)
                .on_click(move || {
                    if let Err(err) = navigation::open_profile(owner_id, Some(guild_id)) {
                        log::warn(&format!("couldn't open the profile: {}", err.message));
                    }
                })
                .child(picture)
                .child(
                    TextProps::styled(name, TextVariant::TEXT_SM_MEDIUM)
                        .color("var(--text-default)")
                        .build(),
                )
        }
        None => muted(
            "Not available. Allow \"See server members and roles\" for Server Info, and open the member list.",
        ),
    }
}

const NOTABLE: [&str; 11] = [
    "administrator",
    "manage-guild",
    "manage-roles",
    "manage-channels",
    "kick-members",
    "ban-members",
    "moderate-members",
    "manage-messages",
    "mention-everyone",
    "manage-webhooks",
    "view-audit-log",
];

const CLOSED_CHIPS: usize = 4;

fn roles_tab(guild_id: u64, mut roles: Vec<Role>, show_permissions: bool) -> Node {
    roles.sort_by_key(|r| std::cmp::Reverse(r.position()));
    let mut list = div().class(cls::ROLES);
    for role in roles {
        let role_id = role.id();
        let permissions = role.permissions();
        let administrator = permissions.iter().any(|p| p == "administrator");
        let open = EXPANDED.with(|open| open.borrow().contains(&(guild_id, role_id)));
        let dot = match role.color() {
            Some(rgb) => div()
                .class(cls::CHIP_DOT)
                .style(format!("background-color: #{rgb:06x}")),
            None => div()
                .class(cls::CHIP_DOT)
                .style("background-color: var(--text-muted)"),
        };
        let summary = if administrator {
            "All permissions".to_string()
        } else if permissions.is_empty() {
            "No permissions".to_string()
        } else {
            format!(
                "{} {}",
                permissions.len(),
                if permissions.len() == 1 {
                    "permission"
                } else {
                    "permissions"
                }
            )
        };
        let head = div()
            .class(cls::ROLE_HEAD)
            .child(dot)
            .child(
                TextProps::styled(role.name(), TextVariant::TEXT_SM_MEDIUM)
                    .color("var(--text-default)")
                    .line_clamp(1)
                    .build()
                    .class(cls::ROLE_NAME),
            )
            .child(
                TextProps::styled(summary, TextVariant::TEXT_XS_NORMAL)
                    .color("var(--text-muted)")
                    .build(),
            );
        let mut row = div().class(cls::ROLE).child(head);
        if show_permissions && !permissions.is_empty() {
            let chips: Vec<Node> = if administrator {
                vec![permission_chip("Administrator", true)]
            } else if open {
                permissions
                    .iter()
                    .map(|p| permission_chip(&permission_label(p), false))
                    .collect()
            } else {
                let mut shown: Vec<&String> = NOTABLE
                    .iter()
                    .filter_map(|n| permissions.iter().find(|p| p == n))
                    .collect();
                for permission in &permissions {
                    if !shown.contains(&permission) {
                        shown.push(permission);
                    }
                }
                shown.truncate(CLOSED_CHIPS);
                let more = permissions.len() - shown.len();
                let mut chips: Vec<Node> = shown
                    .into_iter()
                    .map(|p| permission_chip(&permission_label(p), false))
                    .collect();
                if more > 0 {
                    chips.push(permission_chip(&format!("+{more}"), false));
                }
                chips
            };
            row = row.child(div().class(cls::PERMS).children(chips));
        }
        list = list.child(row.on_click(move || {
            EXPANDED.with(|open| {
                let mut open = open.borrow_mut();
                if !open.remove(&(guild_id, role_id)) {
                    open.insert((guild_id, role_id));
                }
            });
            refresh(&format!("guild:{guild_id}"));
        }));
    }
    list
}

fn permission_label(name: &str) -> String {
    match name {
        "manage-guild" => return "Manage Server".to_string(),
        "manage-guild-expressions" => return "Manage Expressions".to_string(),
        "create-guild-expressions" => return "Create Expressions".to_string(),
        "view-guild-insights" => return "View Server Insights".to_string(),
        "use-vad" => return "Use Voice Activity".to_string(),
        "stream" => return "Video".to_string(),
        _ => {}
    }
    name.split('-')
        .map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn permission_chip(label: &str, strong: bool) -> Node {
    let chip = div().class(cls::PERM).child(
        TextProps::styled(label.to_string(), TextVariant::TEXT_XS_NORMAL)
            .color(if strong {
                "var(--text-strong)"
            } else {
                "var(--text-default)"
            })
            .build(),
    );
    if strong {
        chip.class(cls::PERM_STRONG)
    } else {
        chip
    }
}

fn images_tab(guild: &Guild) -> Node {
    let kinds = [
        (
            "Invite Background",
            guild.splash_url(256),
            guild.splash_url(2048),
            (71, 40),
            4,
        ),
        (
            "Discovery Splash",
            guild.discovery_splash_url(256),
            guild.discovery_splash_url(2048),
            (71, 40),
            4,
        ),
    ];
    let mut list = div().class(cls::ROWS);
    for (name, small, big, (width, height), radius) in kinds {
        let mut row = div().class(cls::ROW);
        match (small, big) {
            (Some(small), Some(big)) => {
                row = row.child(
                    div().class(cls::THUMB).child(
                        image(small, width, height)
                            .prop("radius", radius)
                            .prop("open", big),
                    ),
                );
                row = row.child(
                    TextProps::styled(name.to_string(), TextVariant::TEXT_MD_MEDIUM)
                        .build()
                        .class(cls::ROW_NAME),
                );
            }
            _ => {
                let tile = div()
                    .class(cls::THUMB_EMPTY)
                    .class(if width == height {
                        cls::THUMB_EMPTY_SQUARE
                    } else {
                        cls::THUMB_EMPTY_WIDE
                    })
                    .child(icon("ImageIcon", None).class(cls::THUMB_EMPTY_ICON));
                row = row.child(div().class(cls::THUMB).child(tile));
                row = row.child(
                    TextProps::styled(name.to_string(), TextVariant::TEXT_MD_MEDIUM)
                        .build()
                        .class(cls::ROW_NAME),
                );
                row = row.child(muted("Not set"));
            }
        }
        list = list.child(row);
    }

    let emojis = guild.emojis(128);
    let emojis_big = guild.emojis(512);
    let emoji_count = emojis.len();
    let emoji_content = if emojis.is_empty() {
        muted("No custom emoji")
    } else {
        let mut grid = div().class(cls::EMOJIS);
        for ((name, small), (_, big)) in emojis.into_iter().zip(emojis_big) {
            grid = grid.child(
                image(small, 40, 40)
                    .prop("radius", 4)
                    .prop("open", big)
                    .tooltip(format!(":{name}:")),
            );
        }
        grid
    };
    scroll().child(
        div()
            .class(cls::GROUPS)
            .child(list)
            .child(section(&format!("Emoji ({emoji_count})"), emoji_content)),
    )
}

fn count_label(count: usize, one: &str, many: &str, busy: &str) -> String {
    match count {
        0 => format!("No {many}{busy}"),
        1 => format!("1 {one}{busy}"),
        n => format!("{n} {many}{busy}"),
    }
}

fn people_tab(
    guild_id: u64,
    server: &str,
    people: &[Person],
    what: &str,
    checking: usize,
    skipped: usize,
    saw_relationships: bool,
) -> Node {
    if people.is_empty() {
        return if checking > 0 {
            empty_state(
                &format!("Looking for {what}…"),
                &format!("{checking} left to check."),
            )
        } else if saw_relationships {
            empty_state(
                &format!("No {what} here yet"),
                &format!("None of your {what} are in {server}."),
            )
        } else {
            empty_state(
                "Allow Server Info to look",
                &format!(
                    "Allow \"See your friends and blocked users\" and \"See server members and roles\" to list your {what} here."
                ),
            )
        };
    }
    let mut rows = div().class(cls::ROWS);
    for person in people {
        let user_id = person.id;
        let status = client::get_presence(user_id).map(|p| p.status());
        let picture = avatar(
            person.avatar.as_deref(),
            &person.name,
            40,
            status.as_deref(),
        );
        let name = TextProps::styled(person.name.clone(), TextVariant::TEXT_MD_MEDIUM)
            .build()
            .class(cls::ROW_NAME);
        rows = rows.child(
            div()
                .class(cls::ROW)
                .child(picture)
                .child(name)
                .on_click(move || {
                    if let Err(err) = navigation::open_profile(user_id, Some(guild_id)) {
                        log::warn(&format!("couldn't open the profile: {}", err.message));
                    }
                }),
        );
    }
    if checking > 0 {
        rows = rows.child(muted(&format!("Still looking… {checking} left to check.")));
    }
    if skipped > 0 {
        rows = rows.child(muted(&format!("{skipped} more weren't checked.")));
    }
    scroll().child(rows)
}

fn muted(text: &str) -> Node {
    TextProps::styled(text.to_string(), TextVariant::TEXT_SM_NORMAL)
        .color("var(--text-muted)")
        .build()
}

fn feature_badges(features: &[String]) -> Vec<String> {
    ["verified", "partnered"]
        .iter()
        .filter(|key| features.iter().any(|f| f.eq_ignore_ascii_case(key)))
        .map(|key| key.to_string())
        .collect()
}

fn channel_counts(channels: &[Channel]) -> Node {
    let count = |kinds: &[&str]| {
        channels
            .iter()
            .filter(|c| kinds.contains(&c.kind().as_str()))
            .count()
    };
    let item = |glyph: &str, what: &str, n: usize| {
        div()
            .class(cls::COUNT)
            .tooltip(what)
            .child(icon(glyph, None).class(cls::COUNT_ICON))
            .child(
                TextProps::styled(n.to_string(), TextVariant::TEXT_SM_NORMAL)
                    .color("var(--text-default)")
                    .build(),
            )
    };
    div()
        .class(cls::COUNTS)
        .child(item(
            "TextIcon",
            "Text channels",
            count(&["text", "announcement", "forum", "media"]),
        ))
        .child(item(
            "VoiceNormalIcon",
            "Voice channels",
            count(&["voice", "stage"]),
        ))
        .child(item("FolderIcon", "Categories", count(&["category"])))
}

fn since_row(created_ms: i64, joined_ms: Option<i64>) -> Node {
    let date = |ms: i64, ago: bool| {
        TextProps::styled(date_text(ms), TextVariant::TEXT_SM_NORMAL)
            .color("var(--text-default)")
            .build()
            .tooltip(age_text(ms, ago))
    };
    let mut row = div()
        .class(cls::SINCE)
        .child(section("Created", date(created_ms, false)));
    if let Some(joined) = joined_ms {
        row = row.child(section("Joined", date(joined, true)));
    }
    row
}

fn date_text(ms: i64) -> String {
    let (year, month, day) = civil(ms.div_euclid(86_400_000));
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    format!("{} {day}, {year}", MONTHS[(month - 1) as usize])
}

fn age_text(ms: i64, ago: bool) -> String {
    let now_days = timers::now().div_euclid(86_400_000);
    let age_days = (now_days - ms.div_euclid(86_400_000)).max(0);
    let suffix = if ago { "ago" } else { "old" };
    if age_days >= 365 {
        let years = age_days / 365;
        format!(
            "{years} {} {suffix}",
            if years == 1 { "year" } else { "years" }
        )
    } else if age_days >= 30 {
        format!(
            "{} {} {suffix}",
            age_days / 30,
            if age_days / 30 == 1 {
                "month"
            } else {
                "months"
            }
        )
    } else if age_days >= 1 {
        format!(
            "{age_days} {} {suffix}",
            if age_days == 1 { "day" } else { "days" }
        )
    } else if ago {
        "today".to_string()
    } else {
        "created today".to_string()
    }
}

fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

fn thousands(n: u32) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

fn copy(text: &str, done: &str) {
    match actions::copy_to_clipboard(text) {
        Ok(()) => toast(done, "success"),
        Err(err) => toast(&err.message, "failure"),
    }
}

export_plugin!(ServerInfo);
