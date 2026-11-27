use crate::instances::InstanceStore;
use crate::models;
use crate::ui::classes::quick_switcher;
use crate::ui::classes::quick_switcher as cls;
use crate::ui;
use crate::utils::guild;
use crate::utils::link;
use requiem_ui::components::ScrollViewProps;
use requiem_ui::components::TextVariant;
use requiem_ui::components::status_indicator;
use requiem_ui::components::{ButtonVariant, ModalSize};
use requiem_ui::icons::{Icon, TEXT};
use requiem_ui::paint as requiem_ui_paint;
use requiem_ui::style;
use requiem_ui::widget;
use std::cell::Cell;
use std::cell::RefCell;
use std::f32::consts;
use std::fs;
use std::path;
use std::rc::Rc;
use std::time::Instant;

use requiem_ui::components::{ButtonProps, ModalProps, TextInputProps, TextProps, div, icon};
use requiem_ui::control::section;
use requiem_ui::event::NamedKey;
use requiem_ui::node::Node;
use requiem_ui::signal::{Effect, Signal, signal};
use requiem_ui::widget::primitives::PresenceStatus;
use requiem_ui::widget::{WidgetId, WidgetTree};
use skia_safe::Color;
use skia_safe::font_style::Weight;

use crate::models::id::marker::ChannelMarker;
use crate::models::{Id, Permissions};
use crate::ui::dialog::Dialog;
use crate::ui::home::{HomeEnv, dm_title};
use crate::ui::{TEXT_DEFAULT, TEXT_SECONDARY, TEXT_SUBTLE};

const MAX_RESULTS: usize = 8;

/// Results list geometry, from the stylesheet.
const LIST_HEIGHT: f32 = 262.0;
const LIST_PADDING_TOP: f32 = 16.0;

thread_local! {
    static SCROLL_OFFSET: Cell<f32> = const { Cell::new(0.0) };
}

/// The scroll offset that keeps row `selected` in view. The list is rebuilt on
/// every selection change, so it restarts from where the user left it.
fn offset_for(selected: usize, has_header: bool) -> f32 {
    let top = LIST_PADDING_TOP
        + if has_header { ROW_HEIGHT } else { 0.0 }
        + selected as f32 * ROW_HEIGHT;
    let offset = SCROLL_OFFSET.with(Cell::get);
    let offset = if selected == 0 {
        0.0
    } else if top < offset {
        top
    } else if top + ROW_HEIGHT > offset + LIST_HEIGHT {
        top + ROW_HEIGHT - LIST_HEIGHT
    } else {
        offset
    };
    SCROLL_OFFSET.with(|o| o.set(offset));
    offset
}

const LEARN_MORE_URL: &str = "https://support.discord.com/hc/en-us/articles/115000070311";

thread_local! {
    static DIALOG: Dialog = Dialog::new();
    static QUERY: Signal<String> = signal(String::new());

    static HAS_QUERY: Signal<bool> = signal(false);
    static INDEX: Signal<usize> = signal(0);

    static LISTED: RefCell<Vec<Id<ChannelMarker>>> = const { RefCell::new(Vec::new()) };
}

const NO_RESULTS_PX: (u32, u32) = (200, 208);

thread_local! {
    static NO_RESULTS_IMAGE: Option<skia_safe::Image> = ui::asset_svg("image/no_results", NO_RESULTS_PX.0, NO_RESULTS_PX.1);
}

const ARROW_ASSETS: [(&str, u32, u32); 3] = [
    ("image/tutorial_arrow_1", 86, 36),
    ("image/tutorial_arrow_2", 96, 40),
    ("image/tutorial_arrow_3", 86, 36),
];

struct TutorialArrow {
    kind: usize,
    started: Instant,

    spring: Cell<(f32, f32, Option<Instant>)>,

    image: RefCell<Option<Option<skia_safe::Image>>>,
}

const ARROW_TOP: f32 = 118.0;
const ARROW_HALF: f32 = 9.0;
const FIRST_ROW_TOP: f32 = 178.0;
const ROW_HEIGHT: f32 = 34.0;
const MAX_TRAVEL: f32 = 250.0;
const SPRING_TENSION: f32 = 100.0;
const SPRING_FRICTION: f32 = 15.0;

impl TutorialArrow {
    fn new(kind: usize) -> Self {
        Self {
            kind,
            started: Instant::now(),
            spring: Cell::new((0.0, 0.0, None)),
            image: RefCell::new(None),
        }
    }

    fn target() -> f32 {
        if HAS_QUERY.with(|h| h.peek()) {
            let index = INDEX.with(|i| i.peek()) as f32;
            (FIRST_ROW_TOP + ROW_HEIGHT * index + ARROW_HALF - ARROW_TOP).clamp(0.0, MAX_TRAVEL)
        } else {
            0.0
        }
    }

    fn step(&self) -> f32 {
        let (mut pos, mut vel, last) = self.spring.get();
        let now = Instant::now();
        let mut remaining = last
            .map_or(0.0, |t| now.duration_since(t).as_secs_f32())
            .min(0.1);
        let target = Self::target();
        while remaining > 0.0 {
            let dt = remaining.min(0.004);
            vel += (-SPRING_TENSION * (pos - target) - SPRING_FRICTION * vel) * dt;
            pos += vel * dt;
            remaining -= dt;
        }
        self.spring.set((pos, vel, Some(now)));
        pos
    }
}

impl widget::Widget for TutorialArrow {
    fn paint(
        &self,
        canvas: &skia_safe::Canvas,
        rect: widget::AbsoluteRect,
        style: &style::ComputedStyle,
    ) {
        if style.opacity.is_some_and(|o| o <= 0.01) {
            return;
        }
        let (asset, w, h) = ARROW_ASSETS[self.kind];
        let travel = if self.kind == 0 { self.step() } else { 0.0 };
        let phase = (self.started.elapsed().as_secs_f32() / 0.82).fract();
        let nudge = 3.0 * (0.5 - 0.5 * (phase * consts::TAU).cos());
        let mut slot = self.image.borrow_mut();
        let Some(image) = slot
            .get_or_insert_with(|| ui::asset_svg(asset, w, h))
            .as_ref()
        else {
            return;
        };
        let dst =
            skia_safe::Rect::from_xywh(rect.x + nudge, rect.y + travel, rect.width, rect.height);
        let sampling = skia_safe::SamplingOptions::new(
            skia_safe::FilterMode::Linear,
            skia_safe::MipmapMode::Linear,
        );
        canvas.draw_image_rect_with_sampling_options(
            image,
            None,
            dst,
            sampling,
            &skia_safe::Paint::default(),
        );
        requiem_ui_paint::request_repaint();
    }
}

fn tutorial() -> Node {
    let message = |text: &str, class: &str| {
        TextProps::new(text.to_string(), 18.0_f32)
            .weight(Weight::MEDIUM)
            .color(Color::WHITE)
            .center(true)
            .build()
            .class(cls::MESSAGE)
            .class(class.to_string())
    };
    let arrow = |kind: usize, class: &str| {
        Node::new("div", TutorialArrow::new(kind))
            .class(cls::ARROW)
            .class(class.to_string())
    };
    let group = |right: bool| {
        let mut group = div().class(cls::ARROW_GROUP);
        if right {
            group = group.class(cls::ARROW_GROUP_RIGHT);
        }
        group.children([
            arrow(0, cls::HORIZONTAL),
            arrow(1, cls::DIAG1),
            arrow(2, cls::DIAG2),
        ])
    };
    let messages = div().class(cls::MESSAGES).children([
        message(
            &requiem_i18n::t!("quick-switcher-search-for-servers-channels-or-dms"),
            cls::SEARCH_MESSAGE,
        ),
        message(
            &requiem_i18n::t!("quick-switcher-select-a-result-and-press-enter"),
            cls::SELECT_MESSAGE,
        ),
    ]);
    div()
        .class(cls::TUTORIAL)
        .bind_class(HAS_QUERY.with(Clone::clone), cls::HAS_QUERY, cls::NO_QUERY)
        .children([group(false), group(true), messages])
}

struct NoResultsArt;

impl widget::Widget for NoResultsArt {
    fn paint(
        &self,
        canvas: &skia_safe::Canvas,
        rect: widget::AbsoluteRect,
        _style: &style::ComputedStyle,
    ) {
        NO_RESULTS_IMAGE.with(|image| {
            let Some(image) = image else { return };
            let dst = skia_safe::Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
            let sampling = skia_safe::SamplingOptions::new(
                skia_safe::FilterMode::Linear,
                skia_safe::MipmapMode::Linear,
            );
            canvas.draw_image_rect_with_sampling_options(
                image,
                None,
                dst,
                sampling,
                &skia_safe::Paint::default(),
            );
        });
    }
}

const USED_KEY: &str = "quick_switcher_used";

fn legacy_flag() -> Option<path::PathBuf> {
    InstanceStore::default_path()
        .and_then(|p| p.parent().map(|dir| dir.join("quick_switcher_used")))
}

thread_local! {
    static USED_BEFORE: Cell<Option<bool>> = const { Cell::new(None) };
}

fn used_before() -> bool {
    USED_BEFORE.with(|used| match used.get() {
        Some(value) => value,
        None => {
            let db = requiem_db::Db::shared();
            db.import_once("quick_switcher_used", |db| {
                if let Some(path) = legacy_flag().filter(|p| p.exists()) {
                    db.set_meta(USED_KEY, "1").map_err(|e| e.to_string())?;
                    let _ = fs::remove_file(path);
                }
                Ok(())
            });
            let value = db.meta(USED_KEY).is_some();
            used.set(Some(value));
            value
        }
    })
}

fn mark_used() {
    if used_before() {
        return;
    }
    USED_BEFORE.with(|used| used.set(Some(true)));
    let _ = requiem_db::Db::shared().set_meta(USED_KEY, "1");
}

pub fn is_open() -> bool {
    DIALOG.with(Dialog::is_open)
}

pub fn open() {
    QUERY.with(|q| q.set(String::new()));
    HAS_QUERY.with(|h| h.set(false));
    INDEX.with(|i| i.set(0));
    DIALOG.with(Dialog::open);
}

pub fn close() {
    DIALOG.with(Dialog::close);
}

enum Lead {
    User(models::User, PresenceStatus),

    Channel(Icon),

    Guild { name: String },
}

struct Entry {
    id: Id<ChannelMarker>,
    name: String,

    category: String,

    detail: String,
    lead: Lead,
}

fn entries(env: &HomeEnv, query: &str) -> Vec<Entry> {
    let (people, channels, servers, needle) = match query.chars().next() {
        Some('@') => (true, false, false, &query[1..]),
        Some('#') => (false, true, false, &query[1..]),
        Some('*') => (false, false, true, &query[1..]),
        _ => (true, true, !query.trim().is_empty(), query),
    };
    let needle = needle.trim().to_lowercase();
    let matches = |name: &str| needle.is_empty() || name.to_lowercase().contains(&needle);

    let mut out = Vec::new();
    let guilds = env.state.guilds.list.with_untracked(|g| g.clone());
    if servers {
        for guild in &guilds {
            if matches(&guild.name) {
                if let Some(channel) = ui::default_channel(&env.state, Some(guild.id)) {
                    out.push(Entry {
                        id: channel.id,
                        name: guild.name.clone(),
                        category: requiem_i18n::t!("quick-switcher-server"),
                        detail: String::new(),
                        lead: Lead::Guild {
                            name: guild.name.clone(),
                        },
                    });
                }
            }
        }
    }
    if people {
        for dm in env.state.dms.list.with_untracked(|l| l.clone()) {
            let name = dm_title(&env.state, &dm);
            let user = dm
                .recipients
                .first()
                .and_then(|id| env.state.users.get(*id));
            let username = user
                .as_ref()
                .filter(|_| !dm.is_group())
                .map(|u| u.username.clone())
                .unwrap_or_default();
            if matches(&name) || (!username.is_empty() && matches(&username)) {
                let lead = match user.filter(|_| !dm.is_group()) {
                    Some(user) => {
                        let status = env
                            .state
                            .user_presence
                            .by_user
                            .with_untracked(|all| all.get(&user.id).map(|p| p.status))
                            .unwrap_or_default();
                        Lead::User(user, status)
                    }
                    None => Lead::Channel(TEXT),
                };
                out.push(Entry {
                    id: dm.id,
                    name,
                    category: String::new(),
                    detail: username,
                    lead,
                });
            }
        }
    }
    if channels {
        env.state.channels.by_guild.with_untracked(|by_guild| {
            for guild in &guilds {
                let mut list: Vec<_> = by_guild
                    .get(&guild.id)
                    .into_iter()
                    .flatten()
                    .filter(|c| c.kind == 0 && env.state.can(c, Permissions::VIEW_CHANNEL))
                    .collect();
                list.sort_by_key(|c| c.position.unwrap_or(0));
                for channel in list {
                    let name = channel.name.clone().unwrap_or_default();
                    if matches(&name) {
                        out.push(Entry {
                            id: channel.id,
                            name,
                            category: requiem_i18n::t!("quick-switcher-text-channels"),
                            detail: guild.name.clone(),
                            lead: Lead::Channel(ui::channel_icon(&env.state, channel)),
                        });
                    }
                }
            }
        });
    }
    out.truncate(MAX_RESULTS);
    out
}

fn learn_more_link(label: &str, size: f32, weight: Weight) -> Node {
    let text = TextProps::new(label.to_string(), size)
        .weight(weight)
        .color(Color::from_rgb(0x4d, 0x96, 0xee))
        .build()
        .class(cls::LINK_TEXT);
    ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::LINK.to_string()])
        .on_click(|| link::open_url(LEARN_MORE_URL))
        .build()
        .child(text)
}

fn go(env: &HomeEnv, id: Id<ChannelMarker>) {
    mark_used();
    close();
    (env.on_select_channel)(id);
}

fn dialog(env: &HomeEnv) -> Node {
    let toggle = DIALOG.with(Dialog::toggle);

    let key_env = env.clone();
    let change_env = env.clone();
    let field = TextInputProps::new()
        .bare(true)
        .placeholder(requiem_i18n::t!(
            "quick-switcher-where-would-you-like-to-go"
        ))
        .font_size(22.0_f32)
        .insets((12.0_f32, 12.0_f32))
        .on_change(move |text: &str| {
            HAS_QUERY
                .with(|h| h.set(!text.trim().is_empty() && !entries(&change_env, text).is_empty()));
            QUERY.with(|q| q.set(text.to_string()));
            INDEX.with(|i| i.set(0));
        })
        .key_hook(move |key: NamedKey, _shift: bool, _text: &str| match key {
            NamedKey::Escape => {
                close();
                true
            }
            NamedKey::ArrowDown | NamedKey::ArrowUp => {
                let count = LISTED.with(|l| l.borrow().len());
                if count > 0 {
                    INDEX.with(|i| {
                        let next = if key == NamedKey::ArrowDown {
                            (i.peek() + 1) % count
                        } else {
                            (i.peek() + count - 1) % count
                        };
                        i.set(next);
                    });
                }
                true
            }
            NamedKey::Enter => {
                if let Some(id) = LISTED.with(|l| l.borrow().get(INDEX.with(|i| i.peek())).copied())
                {
                    go(&key_env, id);
                }
                true
            }
            _ => false,
        })
        .build()
        .class(quick_switcher::FIELD)
        .dom_id("quick-switcher-input")
        .wants_focus();

    let list_env = env.clone();
    let results = Node::reactive(move || results_list(&list_env));
    let tip = |text: &str, weight: Weight, color: Color| {
        TextProps::new(text.to_string(), 12.0_f32)
            .weight(weight)
            .color(color)
            .build()
    };
    let mut hint = div()
        .class(quick_switcher::PROTIP)
        .child(
            tip(
                &requiem_i18n::t!("quick-switcher-protip"),
                Weight::BOLD,
                Color::from_rgb(0x5e, 0xb4, 0x79),
            )
            .class(quick_switcher::PROTIP_LABEL),
        );
    hint = hint.child(
        tip(
            &requiem_i18n::t!("quick-switcher-start-searches-with"),
            Weight::NORMAL,
            TEXT_DEFAULT,
        )
        .class(quick_switcher::TIP_TEXT),
    );
    for symbol in ["@", "#", "!", "*", "$"] {
        hint = hint.child(
            div()
                .class(quick_switcher::SYMBOL)
                .child(tip(symbol, Weight::NORMAL, TEXT_DEFAULT)),
        );
    }
    hint = hint.child(
        tip(
            &requiem_i18n::t!("quick-switcher-to-narrow-results"),
            Weight::NORMAL,
            TEXT_DEFAULT,
        )
        .class(quick_switcher::TIP_TEXT),
    );
    hint = hint.child(learn_more_link(
        &requiem_i18n::t!("quick-switcher-learn-more"),
        12.0,
        Weight::NORMAL,
    ));

    let title = if used_before() { div() } else { tutorial() };
    let body = div()
        .class(quick_switcher::BODY)
        .children([title, field, results, hint]);
    ModalProps::default()
        .size(ModalSize::Medium)
        .header(false)
        .dismissable(true)
        .on_dismiss(close)
        .open(toggle)
        .body(body)
        .panel_class(quick_switcher::PANEL.to_string())
        .build()
}

fn results_list(env: &HomeEnv) -> Node {
    let _ = env.tick.get();
    let query = QUERY.with(Signal::get);
    let selected = INDEX.with(Signal::get);
    let found = entries(env, &query);
    LISTED.with(|l| *l.borrow_mut() = found.iter().map(|e| e.id).collect());
    let fonts = &env.fonts;

    let has_header = query.trim().is_empty() && !found.is_empty();
    let mut list = ScrollViewProps::default()
        .initial_offset_y(offset_for(selected, has_header))
        .on_scroll(|offset: f32, _max: f32| SCROLL_OFFSET.with(|o| o.set(offset)))
        .build()
        .class(cls::RESULTS);
    if has_header {
        list = list.child(
            TextProps::styled(
                requiem_i18n::t!("quick-switcher-previous-channels").to_string(),
                TextVariant::TEXT_XS_SEMIBOLD,
            )
            .color(TEXT_SECONDARY)
            .build()
            .class(cls::SECTION),
        );
    }
    for (i, entry) in found.iter().enumerate() {
        let mut classes = vec![cls::RESULT.to_string()];
        if i == selected {
            classes.push(cls::RESULT_SELECTED.to_string());
        }
        let (go_env, id) = (env.clone(), entry.id);
        let lead: Node = match &entry.lead {
            Lead::Channel(glyph) => icon(*glyph, TEXT_SUBTLE).class(cls::RESULT_ICON),
            Lead::Guild { name } => div().class(cls::GUILD_ICON).child(
                TextProps::styled(
                    guild::acronym(name),
                    TextVariant::TEXT_XXS_MEDIUM,
                )
                .color(TEXT_DEFAULT)
                .build(),
            ),
            Lead::User(user, status) => {
                let mut props = ui::user_avatar_props(user.id, &user.discriminator, fonts);
                if let Some(image) = user.avatar.as_deref().and_then(|hash| {
                    env.images
                        .get_or_decode_image(&env.backend.avatar_url(user.id, hash))
                }) {
                    props = props.image(image);
                }
                let mut wrap = div()
                    .class(cls::AVATAR_BOX)
                    .child(props.build().class(cls::AVATAR));
                if *status != PresenceStatus::Offline {
                    wrap = wrap
                        .child(div().class(cls::DOT_RING))
                        .child(status_indicator(*status).class(cls::DOT));
                }
                wrap
            }
        };
        let mut row = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(classes)
            .on_click(move || go(&go_env, id))
            .build();
        row = row.child(lead).child(
            TextProps::styled(entry.name.clone(), TextVariant::TEXT_MD_MEDIUM)
                .color(TEXT_SECONDARY)
                .line_clamp(1_u32)
                .build(),
        );
        if matches!(entry.lead, Lead::User(..)) && !entry.detail.is_empty() {
            row = row.child(
                TextProps::styled(entry.detail.clone(), TextVariant::TEXT_MD_NORMAL)
                    .color(TEXT_SECONDARY)
                    .line_clamp(1_u32)
                    .build()
                    .class(cls::USERNAME),
            );
        }
        if !entry.category.is_empty() {
            row = row.child(
                TextProps::styled(entry.category.clone(), TextVariant::TEXT_XXS_SEMIBOLD)
                    .color(Color::from_rgb(0x96, 0x97, 0x9e))
                    .build()
                    .class(cls::CATEGORY),
            );
        }
        row = row.child(div().class(cls::SPACER));
        if !entry.detail.is_empty() && !matches!(entry.lead, Lead::User(..)) {
            row = row.child(
                TextProps::styled(entry.detail.clone(), TextVariant::TEXT_SM_MEDIUM)
                    .color(TEXT_SECONDARY)
                    .line_clamp(1_u32)
                    .build(),
            );
        }
        list = list.child(row);
    }
    if found.is_empty() {
        let learn = learn_more_link(
            &requiem_i18n::t!("quick-switcher-learn-more-about-quick-switcher"),
            14.0,
            Weight::MEDIUM,
        );
        list = list.child(
            div().class(cls::EMPTY_STATE).children([
                Node::new("div", NoResultsArt).class(cls::EMPTY_ART),
                TextProps::styled(
                    requiem_i18n::t!("quick-switcher-can-t-seem-to-find-what").to_string(),
                    TextVariant::TEXT_MD_MEDIUM,
                )
                .color(TEXT_SUBTLE)
                .center(true)
                .build()
                .class(cls::EMPTY_NOTE),
                div().class(cls::EMPTY_CTA).child(learn),
            ]),
        );
    }
    list
}

pub fn mount(tree: &mut WidgetTree, parent: WidgetId, env: Rc<dyn Fn() -> HomeEnv>) -> Effect {
    section(tree, parent, move || {
        if DIALOG.with(Dialog::is_mounted) {
            dialog(&env())
        } else {
            div()
        }
    })
}
