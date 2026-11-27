use crate::ui::classes::emoji_picker as cls;
use std::cell::Cell;
use crate::models;
use crate::state;
use crate::ui::classes::emoji_picker;
use crate::ui::context_menus;
use crate::ui::emoji_anim::EmojiImageWidget;
use crate::ui::emoji_anim;
use crate::ui::emoji_names;
use crate::ui::emoji_skin;
use crate::ui::emoji_style_menu;
use crate::ui::emoji_usage;
use crate::utils::guild;
use crate::utils::time;
use requiem_ui::components::ButtonVariant;
use requiem_ui::components::input_leading_icon;
use requiem_ui::control;
use requiem_ui::icons::Icon;
use requiem_ui::widget::primitives;
use skia_safe::font_style::Weight;
use std::cell::OnceCell;
use std::cell::RefCell;
use std::cmp;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::ptr;
use std::rc::Rc;
use std::sync::OnceLock;
use virtualization::CATEGORY_GAP_PX;
use virtualization::COLUMNS;
use virtualization::HEADER_ROW_HEIGHT;
use virtualization::TILE_ROW_HEIGHT;

use emojis::{Emoji, Group};
use skia_safe::Color;
use taffy::{AvailableSpace, Size};

use requiem_ui::components::{
    ButtonProps, PopoverProps, ScrollViewProps, TextInputProps, TextProps, div, icon, text,
};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::widget::Widget;
use requiem_ui::widget::primitives::ImageWidget;

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::models::id::marker::GuildMarker;
use crate::models::{CustomEmoji, Id};

use crate::ui::gif_picker::{self, GifPickerState};
use crate::ui::guild_rail::RailPillWidget;
use crate::ui::twemoji;
use crate::ui::{TEXT_DEFAULT, TEXT_SECONDARY, TEXT_STRONG};
use requiem_ui::icons::{
    BICYCLE, CHEVRON_SMALL_DOWN, CHEVRON_SMALL_RIGHT, CLOCK, EMOJI_SLIGHTLY_SMILING_FACE, FLAG,
    FOOD, GAME_CONTROLLER, HEART, LOCK, MAGNIFYING_GLASS, NATURE, OBJECT, TROPHY,
};

pub(super) fn emoji_glyph(emoji: &str, size_class: &'static str, text_size: f32) -> Node {
    emoji_glyph_with(emoji, size_class, text_size, twemoji::tile)
}

pub(super) fn emoji_glyph_exact(emoji: &str, size_class: &'static str, text_size: f32) -> Node {
    emoji_glyph_with(emoji, size_class, text_size, twemoji::exact_tile)
}

fn emoji_glyph_with(
    emoji: &str,
    size_class: &'static str,
    text_size: f32,
    tile: fn(&str) -> Option<(skia_safe::Image, (f32, f32, f32, f32))>,
) -> Node {
    match tile(emoji) {
        Some((image, (x, y, w, h))) => Node::new(
            "div",
            ImageWidget::new(Some(image.clone())).with_src_rect(x, y, w, h),
        )
        .class(size_class),
        None => text(TextProps {
            content: emoji.to_string(),
            size: text_size,
            color: Color::WHITE,
            ..Default::default()
        }),
    }
}

pub(super) fn matching_emoji(query: &str) -> impl Iterator<Item = &'static Emoji> + '_ {
    emojis::iter().filter(move |e| {
        e.name().to_lowercase().contains(query)
            || e.shortcode()
                .is_some_and(|s| s.to_lowercase().contains(query))
            || emoji_names::name_matches(e.as_str(), query)
    })
}

const EMOJI_LOOKUP_ROWS: usize = 10;

pub fn emoji_lookup(
    custom: Option<Rc<dyn Fn() -> Rc<Vec<GuildSection>>>>,
) -> primitives::DynamicAutocomplete {
    primitives::DynamicAutocomplete {
        trigger_char: ':',
        popup_label: requiem_i18n::t!("emoji-picker-emoji").to_string(),
        min_query_len: 2,
        search: Rc::new(move |query| {
            let mut matches = custom
                .as_ref()
                .map(|sections| custom_lookup_matches(&sections(), &query.to_lowercase()))
                .unwrap_or_default();
            matches.extend(emoji_lookup_matches(query));
            matches.truncate(EMOJI_LOOKUP_ROWS);
            matches
        }),
    }
}

fn custom_lookup_matches(
    sections: &[GuildSection],
    query: &str,
) -> Vec<primitives::DynamicMatch> {
    let usable = sections
        .iter()
        .flat_map(|s| s.emojis.iter())
        .filter(|t| !t.locked);
    let (mut prefix, mut rest) = (Vec::new(), Vec::new());
    for tile in usable {
        let name = tile.emoji.name.to_lowercase();
        let found = primitives::DynamicMatch {
            label: tile.emoji.name.clone(),
            glyph: tile.emoji.markup(),
        };
        if name.starts_with(query) {
            prefix.push(found);
        } else if name.contains(query) {
            rest.push(found);
        }
    }
    prefix.extend(rest);
    prefix
}

fn emoji_lookup_matches(query: &str) -> Vec<primitives::DynamicMatch> {
    emoji_names::lookup(&query.to_lowercase(), EMOJI_LOOKUP_ROWS)
        .into_iter()
        .map(|m| primitives::DynamicMatch {
            label: m.name.to_string(),
            glyph: m.emoji.to_string(),
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PickerCategory {
    People,
    Nature,
    Food,
    Activities,
    Travel,
    Objects,
    Symbols,
    Flags,

    Frequent,

    TopEmoji,

    Guild(Id<GuildMarker>),
}

pub const ALL_CATEGORIES: [PickerCategory; 8] = [
    PickerCategory::People,
    PickerCategory::Nature,
    PickerCategory::Food,
    PickerCategory::Activities,
    PickerCategory::Travel,
    PickerCategory::Objects,
    PickerCategory::Symbols,
    PickerCategory::Flags,
];

impl PickerCategory {
    fn groups(self) -> &'static [Group] {
        match self {
            PickerCategory::People => &[Group::SmileysAndEmotion, Group::PeopleAndBody],
            PickerCategory::Nature => &[Group::AnimalsAndNature],
            PickerCategory::Food => &[Group::FoodAndDrink],
            PickerCategory::Activities => &[Group::Activities],
            PickerCategory::Travel => &[Group::TravelAndPlaces],
            PickerCategory::Objects => &[Group::Objects],
            PickerCategory::Symbols => &[Group::Symbols],
            PickerCategory::Flags => &[Group::Flags],
            PickerCategory::Frequent | PickerCategory::TopEmoji | PickerCategory::Guild(_) => &[],
        }
    }

    fn label(self) -> String {
        match self {
            PickerCategory::People => requiem_i18n::t!("emoji-picker-people"),
            PickerCategory::Nature => requiem_i18n::t!("emoji-picker-nature"),
            PickerCategory::Food => requiem_i18n::t!("emoji-picker-food-drink"),
            PickerCategory::Activities => requiem_i18n::t!("emoji-picker-activities"),
            PickerCategory::Travel => requiem_i18n::t!("emoji-picker-travel-places"),
            PickerCategory::Objects => requiem_i18n::t!("emoji-picker-objects"),
            PickerCategory::Symbols => requiem_i18n::t!("emoji-picker-symbols"),
            PickerCategory::Flags => requiem_i18n::t!("emoji-picker-flags"),
            PickerCategory::Frequent => requiem_i18n::t!("emoji-picker-frequently-used"),
            PickerCategory::TopEmoji | PickerCategory::Guild(_) => String::new(),
        }
    }

    fn icon_path(self) -> Icon {
        match self {
            PickerCategory::People => EMOJI_SLIGHTLY_SMILING_FACE,
            PickerCategory::Nature => NATURE,
            PickerCategory::Food => FOOD,
            PickerCategory::Activities => GAME_CONTROLLER,
            PickerCategory::Travel => BICYCLE,
            PickerCategory::Objects => OBJECT,
            PickerCategory::Symbols => HEART,
            PickerCategory::Flags => FLAG,
            PickerCategory::Frequent => CLOCK,
            PickerCategory::TopEmoji => TROPHY,
            PickerCategory::Guild(_) => EMOJI_SLIGHTLY_SMILING_FACE,
        }
    }
}

#[derive(Clone)]
pub struct CustomTile {
    pub emoji: CustomEmoji,
    pub locked: bool,
}

#[derive(Clone)]
pub struct GuildSection {
    pub guild_id: Id<GuildMarker>,
    pub name: String,
    pub icon: Option<String>,
    pub emojis: Vec<CustomTile>,
}

#[derive(Clone)]
pub struct CustomEmojiSource {
    pub images: ImageCache,
    pub backend: BackendConfig,

    pub image_tick: Signal<u64>,

    pub on_pick_used: Option<Rc<dyn Fn(&str)>>,

    data: Rc<LazyEmoji>,
}

struct EmojiData {
    sections: Rc<Vec<GuildSection>>,
    frequent: Rc<Vec<Tile>>,
    top: Rc<Vec<CustomTile>>,
    top_label: String,
}

struct LazyEmoji {
    build: Box<dyn Fn() -> EmojiData>,
    cell: OnceCell<Rc<EmojiData>>,
}

impl LazyEmoji {
    fn get(&self) -> &Rc<EmojiData> {
        self.cell.get_or_init(|| Rc::new((self.build)()))
    }
}

const NEWEST_EMOJI: usize = 3;
const NEWEST_DAYS: u64 = 60;
const TOP_ROW: usize = 9;

impl CustomEmojiSource {
    pub fn new(
        env: &context_menus::MenuEnv,
        guild_id: Option<Id<GuildMarker>>,
        everywhere: bool,
        image_tick: &Signal<u64>,
    ) -> Self {
        let (used_env, build_env) = (env.clone(), env.clone());
        let build = move || {
            let env = &build_env;
            let sections = guild_sections(&env.state, guild_id, everywhere);
            let frequent = frequent_tiles(
                &emoji_usage::frequent_keys(
                    env,
                    emoji_usage::FREQUENT_LIMIT * 2,
                ),
                &sections,
            );
            let (top, top_label) = match guild_id
                .and_then(|id| sections.iter().find(|s| s.guild_id == id))
            {
                Some(section) => {
                    let top_ids = emoji_usage::top_emoji_ids(env, section.guild_id);
                    (
                        top_tiles(
                            section,
                            &top_ids,
                            time::snowflake_days_ago(NEWEST_DAYS),
                        ),
                        requiem_i18n::t!("emoji-picker-top-emoji-in", name = section.name.as_str()),
                    )
                }
                None => (Vec::new(), String::new()),
            };
            EmojiData {
                sections: Rc::new(sections),
                frequent: Rc::new(frequent),
                top: Rc::new(top),
                top_label,
            }
        };
        Self {
            images: env.images.clone(),
            backend: env.backend.clone(),
            image_tick: image_tick.clone(),
            on_pick_used: Some(Rc::new(move |picked| {
                emoji_usage::track(&used_env, picked)
            })),
            data: Rc::new(LazyEmoji {
                build: Box::new(build),
                cell: OnceCell::new(),
            }),
        }
    }

    pub fn sections(&self) -> Rc<Vec<GuildSection>> {
        self.data.get().sections.clone()
    }

    fn frequent(&self) -> Rc<Vec<Tile>> {
        self.data.get().frequent.clone()
    }

    fn top(&self) -> Rc<Vec<CustomTile>> {
        self.data.get().top.clone()
    }

    fn top_label(&self) -> String {
        self.data.get().top_label.clone()
    }

    pub fn lookup_sections(&self) -> Rc<dyn Fn() -> Rc<Vec<GuildSection>>> {
        let data = self.data.clone();
        Rc::new(move || data.get().sections.clone())
    }
}

fn frequent_tiles(keys: &[String], sections: &[GuildSection]) -> Vec<Tile> {
    let mut seen = HashSet::new();
    keys.iter()
        .filter_map(|key| match key.parse::<u64>() {
            Ok(id) => sections
                .iter()
                .flat_map(|s| s.emojis.iter())
                .find(|c| c.emoji.id.get() == id)
                .cloned()
                .map(Tile::Custom),
            Err(_) => {
                let emoji = emoji_names::emoji_for_name(key).and_then(emojis::get)?;
                let plain =
                    emojis::get(&emoji_skin::plain(emoji.as_str())).unwrap_or(emoji);
                seen.insert(plain.as_str()).then_some(Tile::Unicode(plain))
            }
        })
        .take(emoji_usage::FREQUENT_LIMIT)
        .collect()
}

fn top_tiles(section: &GuildSection, top_ids: &[u64], newest_from: u64) -> Vec<CustomTile> {
    let mut newest: Vec<&CustomTile> = section
        .emojis
        .iter()
        .filter(|c| c.emoji.id.get() >= newest_from)
        .collect();
    newest.sort_by_key(|c| cmp::Reverse(c.emoji.id.get()));
    newest.truncate(NEWEST_EMOJI);
    let is_newest = |id: u64| newest.iter().any(|c| c.emoji.id.get() == id);
    let top = top_ids
        .iter()
        .filter(|id| !is_newest(**id))
        .filter_map(|id| section.emojis.iter().find(|c| c.emoji.id.get() == *id));
    top.take(TOP_ROW - newest.len())
        .chain(newest.iter().copied())
        .cloned()
        .collect()
}

impl CustomEmojiSource {
    fn section(&self, guild_id: Id<GuildMarker>) -> Option<(String, Option<String>)> {
        self.data
            .get()
            .sections
            .iter()
            .find(|s| s.guild_id == guild_id)
            .map(|s| (s.name.clone(), s.icon.clone()))
    }
}

pub fn guild_sections(
    state: &state::AppState,
    current_guild: Option<Id<GuildMarker>>,
    can_use_elsewhere: bool,
) -> Vec<GuildSection> {
    state.emojis.by_guild.with_untracked(|by_guild| {
        state.guilds.list.with_untracked(|guilds| {
            let mut order: Vec<&models::Guild> = guilds.iter().collect();

            order.sort_by_key(|g| Some(g.id) != current_guild);
            order
                .into_iter()
                .filter_map(|guild| {
                    let emojis = by_guild.get(&guild.id).filter(|e| !e.is_empty())?;
                    let here = Some(guild.id) == current_guild;
                    let emojis = emojis
                        .iter()
                        .map(|e| CustomTile {
                            locked: !e.available
                                || !(here || can_use_elsewhere)
                                || (e.animated && !can_use_elsewhere),
                            emoji: e.clone(),
                        })
                        .collect();
                    Some(GuildSection {
                        guild_id: guild.id,
                        name: guild.name.clone(),
                        icon: guild.icon.clone(),
                        emojis,
                    })
                })
                .collect()
        })
    })
}

thread_local! {

    static CUSTOM_HOVERED: Signal<Option<u64>> = signal(None);
}

#[derive(Clone)]
pub enum Tile {
    Unicode(&'static Emoji),
    Custom(CustomTile),
}

impl Tile {
    fn key(&self) -> usize {
        match self {
            Tile::Unicode(e) => *e as *const Emoji as usize,
            Tile::Custom(c) => ((c.emoji.id.get() as usize) << 1) | 1,
        }
    }
}

fn occurrence_key(tile_key: usize, salt: usize) -> usize {
    tile_key.wrapping_mul(0x9E37_79B1).wrapping_add(salt)
}

fn guild_icon_node(
    source: &CustomEmojiSource,
    guild_id: Id<GuildMarker>,
    class: &'static str,
) -> Node {
    let Some((name, icon)) = source.section(guild_id) else {
        return div().class(class);
    };
    match icon.as_deref() {
        Some(hash) => {
            let url = source.backend.guild_icon_url(guild_id, hash);
            let initial = source.images.get_or_decode_image(&url);
            let images = source.images.clone();
            Node::new("div", ImageWidget::new(initial))
                .class(class)
                .bind_image(source.image_tick.clone(), move || {
                    images.get_or_decode_image(&url)
                })
        }
        None => div().class(class).child(text(TextProps {
            content: guild::acronym(&name),
            size: 9.0,
            color: TEXT_DEFAULT,
            weight: Weight::SEMI_BOLD.into(),
            ..Default::default()
        })),
    }
}

fn custom_emoji_node(source: &CustomEmojiSource, emoji: &CustomEmoji, class: &'static str) -> Node {
    Node::new(
        "div",
        EmojiImageWidget::new(
            emoji.id.get(),
            emoji.animated,
            &source.images,
            &source.backend,
        ),
    )
    .class(class)
}

fn tile_emoji_node(source: &CustomEmojiSource, emoji: &CustomEmoji, class: &'static str) -> Node {
    Node::new(
        "div",
        EmojiImageWidget::new(
            emoji.id.get(),
            emoji.animated,
            &source.images,
            &source.backend,
        )
        .frame_edge(emoji_anim::PICKER_FRAME_EDGE),
    )
    .class(class)
}

mod virtualization {
    use requiem_ui::signal as requiem_ui_signal;

    pub const COLUMNS: usize = 9;
    pub const TILE_ROW_HEIGHT: f32 = 48.0;
    pub const HEADER_ROW_HEIGHT: f32 = 32.0;

    pub const CATEGORY_GAP_PX: f32 = 12.0;

    pub const VIEWPORT_PX: f32 = 325.0;

    pub const OVERSCAN_PX: f32 = 480.0;

    pub const EDGE_PX: f32 = 160.0;

    thread_local! {

        pub static WINDOW_TICK: requiem_ui_signal::Signal<u64> = requiem_ui_signal::signal(0);
    }
}

struct EmojiRowPlaceholder {
    height: f32,
}

impl Widget for EmojiRowPlaceholder {
    fn measure(
        &self,
        _known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        Size {
            width: 0.0,
            height: self.height,
        }
    }
}

#[derive(Clone)]
pub struct EmojiPickerSignals {
    pub search: Signal<String>,

    pub scroll_offset: Signal<f32>,

    pub category_jump: Signal<Option<PickerCategory>>,

    pub hovered: Signal<Option<&'static Emoji>>,

    pub collapsed: Signal<Vec<PickerCategory>>,
}

impl EmojiPickerSignals {
    pub fn reset_for_open(&self) {
        if self.hovered.peek().is_some() {
            self.hovered.set(None);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PickerTab {
    Gif,
    Emoji,
}

#[derive(Clone)]
pub struct EmojiPickerState {
    pub open: Signal<bool>,
    pub tab: Signal<PickerTab>,
    pub signals: EmojiPickerSignals,
    pub gif: GifPickerState,

    pub on_insert: Rc<dyn Fn(String)>,

    pub on_insert_at: Rc<dyn Fn(&'static str, String)>,
}

pub fn build(
    state: &EmojiPickerState,
    images: &ImageCache,
    on_pick_emoji: Rc<dyn Fn(String, bool)>,
    on_pick_gif: Rc<dyn Fn(String)>,
    custom: Option<CustomEmojiSource>,
) -> Node {
    let tab = state.tab.get();

    let fresh = session::fresh();
    let nav = build_nav_row(&state.tab, &state.gif.on_open, tab);
    let content = match tab {
        PickerTab::Emoji => build_emoji_tab(&state.signals, on_pick_emoji, custom, fresh),
        PickerTab::Gif => gif_picker::build(&state.gif, images, on_pick_gif),
    };

    PopoverProps::default()
        .dom_id("emoji-picker-panel")
        .build()
        .class(emoji_picker::PICKER_PANEL)
        .child(nav)
        .child(content)
        .child(session_marker())
}

pub fn build_emoji_only(
    signals: &EmojiPickerSignals,
    on_pick_emoji: Rc<dyn Fn(String, bool)>,
    custom: Option<CustomEmojiSource>,
) -> Node {
    let fresh = session::fresh();
    PopoverProps::default()
        .build()
        .class(emoji_picker::PICKER_PANEL)
        .class(emoji_picker::PICKER_PANEL_EMOJI_ONLY)
        .child(build_emoji_tab(signals, on_pick_emoji, custom, fresh))
        .child(session_marker())
}

fn build_nav_row(
    tab_signal: &Signal<PickerTab>,
    on_open_gif_tab: &Rc<dyn Fn()>,
    active: PickerTab,
) -> Node {
    let mut row = div().class(emoji_picker::PICKER_NAV_ROW);

    let gif_tab_signal = tab_signal.clone();
    let on_open_gif_tab = on_open_gif_tab.clone();
    let mut gif_tab = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![
            emoji_picker::PICKER_NAV_TAB.to_string(),
        ])
        .on_click(move || {
            gif_tab_signal.set(PickerTab::Gif);
            on_open_gif_tab();
        })
        .build()
        .child(nav_tab_label(&requiem_i18n::t!("emoji-picker-gifs")));
    if active == PickerTab::Gif {
        gif_tab = gif_tab.class(emoji_picker::PICKER_NAV_TAB_ACTIVE);
    }

    let emoji_tab_signal = tab_signal.clone();
    let mut emoji_tab = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![
            emoji_picker::PICKER_NAV_TAB.to_string(),
        ])
        .on_click(move || emoji_tab_signal.set(PickerTab::Emoji))
        .build()
        .child(nav_tab_label(&requiem_i18n::t!("emoji-picker-emoji")));
    if active == PickerTab::Emoji {
        emoji_tab = emoji_tab.class(emoji_picker::PICKER_NAV_TAB_ACTIVE);
    }

    row = row.child(gif_tab).child(emoji_tab);
    row
}

fn nav_tab_label(content: &str) -> Node {
    text(TextProps {
        content: content.to_string(),
        size: 14.0,
        color: TEXT_DEFAULT,
        weight: Weight::SEMI_BOLD.into(),
        ..Default::default()
    })
    .class(emoji_picker::PICKER_NAV_TAB_LABEL)
}

fn build_emoji_tab(
    signals: &EmojiPickerSignals,
    on_pick: Rc<dyn Fn(String, bool)>,
    custom: Option<CustomEmojiSource>,
    autofocus: bool,
) -> Node {
    let search = signals.search.get();

    let placeholder = signal(String::new());
    let header = build_header(&signals.search, &search, &placeholder, autofocus);

    let body = {
        let signals = signals.clone();
        let (search, custom, placeholder) = (search.clone(), custom.clone(), placeholder.clone());
        Node::reactive(move || {
            let grid = build_grid(&signals, &search, on_pick.clone(), &custom);
            let rail = build_rail(&signals.category_jump, &grid.active_flags, &custom);

            let footer = {
                let hovered = signals.hovered.clone();
                let listed = grid.listed.clone();
                let first = grid.listed.first().cloned();
                let custom = custom.clone();
                let placeholder = placeholder.clone();
                Node::reactive(move || {
                    let custom_selected = CUSTOM_HOVERED.with(Signal::get).and_then(|id| {
                        listed
                            .iter()
                            .find(|t| matches!(t, Tile::Custom(c) if c.emoji.id.get() == id))
                            .cloned()
                    });
                    let unicode_selected = || {
                        hovered
                            .get()
                            .filter(|h| {
                                listed
                                    .iter()
                                    .any(|t| matches!(t, Tile::Unicode(e) if ptr::eq(*e, *h)))
                            })
                            .map(Tile::Unicode)
                    };
                    let selected = custom_selected
                        .or_else(unicode_selected)
                        .or_else(|| first.clone());
                    let label = selected.as_ref().map(tile_label).unwrap_or_default();
                    if placeholder.peek() != label {
                        placeholder.set(label);
                    }
                    build_footer(selected, &custom)
                })
            };

            let mut grid_column = div()
                .class(emoji_picker::PICKER_GRID_TAB_COLUMN)
                .child(grid.scroll);
            if let Some(sticky) = grid.sticky {
                grid_column = grid_column.child(sticky);
            }
            let grid_column = grid_column.child(footer);
            div()
                .class(emoji_picker::PICKER_BODY)
                .child(rail)
                .child(grid_column)
        })
        .style("flex-grow: 1; flex-shrink: 1; min-height: 0px; width: 100%")
    };

    div()
        .class(emoji_picker::PICKER_TAB_CONTENT)
        .child(header)
        .child(body)
}

fn build_header(
    search_signal: &Signal<String>,
    search: &str,
    placeholder: &Signal<String>,
    autofocus: bool,
) -> Node {
    let search_signal = search_signal.clone();
    search_header(
        &requiem_i18n::t!("emoji-picker-find-the-perfect-emoji"),
        search,
        move |text: &str| search_signal.set(text.to_string()),
        Some(placeholder.clone()),
        Some(emoji_style_menu::build()),
        autofocus,
    )
}

mod session {
    use std::cell::Cell;
    use std::time::{Duration, Instant};

    use requiem_ui::widget::Widget;

    thread_local! {
        static ALIVE: Cell<u32> = const { Cell::new(0) };
        static GONE_AT: Cell<Option<Instant>> = const { Cell::new(None) };
    }

    const REBUILD_GRACE: Duration = Duration::from_millis(250);

    pub fn fresh() -> bool {
        ALIVE.with(Cell::get) == 0
            && GONE_AT
                .with(Cell::get)
                .is_none_or(|gone| gone.elapsed() > REBUILD_GRACE)
    }

    pub struct Marker;

    impl Marker {
        pub fn new() -> Self {
            ALIVE.with(|alive| alive.set(alive.get() + 1));
            Marker
        }
    }

    impl Widget for Marker {}

    impl Drop for Marker {
        fn drop(&mut self) {
            let _ = ALIVE.try_with(|alive| {
                alive.set(alive.get().saturating_sub(1));
                if alive.get() == 0 {
                    let _ = GONE_AT.try_with(|gone| gone.set(Some(Instant::now())));
                }
            });
        }
    }
}

fn session_marker() -> Node {
    Node::new("div", session::Marker::new()).style("position: absolute; width: 0px; height: 0px")
}

const ACTIVE_CATEGORY_LOOKAHEAD_PX: f32 = 20.0;

fn tile_label(tile: &Tile) -> String {
    match tile {
        Tile::Custom(c) => format!(":{}:", c.emoji.name),
        Tile::Unicode(e) => emoji_label(e),
    }
}

fn emoji_label(emoji: &'static Emoji) -> String {
    let plain = emoji
        .with_skin_tone(emojis::SkinTone::Default)
        .unwrap_or(emoji);
    emoji_names::primary_name(plain.as_str())
        .or(plain.shortcode())
        .map(|s| format!(":{s}:"))
        .unwrap_or_else(|| plain.name().to_string())
}

pub fn search_header(
    placeholder: &str,
    search: &str,
    on_change: impl FnMut(&str) + 'static,
    live_placeholder: Option<Signal<String>>,
    trailing: Option<Node>,
    autofocus: bool,
) -> Node {
    let mut input = TextInputProps::new()
        .autofocus(autofocus)
        .text(search.to_string())
        .placeholder(placeholder.to_string())
        .leading(input_leading_icon(MAGNIFYING_GLASS))
        .on_change(on_change);
    if let Some(live) = live_placeholder {
        input = input.placeholder_live(live);
    }

    let input = input
        .build()
        .style("width: 0px; flex-grow: 1; flex-shrink: 1; min-width: 0px");
    let header = div()
        .class(emoji_picker::PICKER_HEADER)
        .child(input);
    match trailing {
        Some(trailing) => header.child(trailing),
        None => header,
    }
}

fn special_categories(
    custom: &Option<CustomEmojiSource>,
) -> impl Iterator<Item = PickerCategory> + '_ {
    let mut cats = Vec::new();
    if let Some(c) = custom {
        if !c.top().is_empty() {
            cats.push(PickerCategory::TopEmoji);
        }
        if !c.frequent().is_empty() {
            cats.push(PickerCategory::Frequent);
        }
        cats.extend(
            c.sections()
                .iter()
                .map(|s| PickerCategory::Guild(s.guild_id)),
        );
    }
    cats.into_iter()
}

fn build_rail(
    category_jump: &Signal<Option<PickerCategory>>,
    active_flags: &[(PickerCategory, Signal<bool>)],
    custom: &Option<CustomEmojiSource>,
) -> Node {
    let mut rail = ScrollViewProps::default().build().class(cls::PICKER_RAIL);
    for cat in special_categories(custom).chain(ALL_CATEGORIES) {
        let category_jump = category_jump.clone();
        let flag = active_flags
            .iter()
            .find(|(c, _)| *c == cat)
            .map(|(_, f)| f.clone())
            .unwrap_or_else(|| signal(false));

        let pill = Node::new("div", RailPillWidget::new())
            .class(cls::PICKER_RAIL_PILL)
            .bind_class(flag.clone(), cls::PICKER_RAIL_PILL_SELECTED, "");
        let item = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::PICKER_RAIL_ITEM.to_string()])
            .on_click(move || category_jump.set(Some(cat)))
            .build()
            .bind_class(flag, cls::PICKER_RAIL_ITEM_SELECTED, "")
            .child(pill);
        let item = match (cat, custom) {
            (PickerCategory::Guild(id), Some(source)) => {
                item.child(guild_icon_node(source, id, cls::PICKER_RAIL_GUILD_ICON))
            }
            _ => item.child(icon(cat.icon_path(), TEXT_SECONDARY).classes(vec![
                cls::PICKER_RAIL_ICON.to_string(),
                cls::PICKER_RAIL_ICON_COLOR.to_string(),
            ])),
        };
        rail = rail.child(item);
    }
    rail
}

enum RowKind {
    CategoryHeader(PickerCategory),

    Gap,

    Tiles(Vec<Tile>, usize),
}

struct Row {
    y: f32,
    height: f32,
    kind: RowKind,
}

fn layout_rows(
    by_category: &[(PickerCategory, Vec<Tile>)],
    collapsed: &[PickerCategory],
) -> (Vec<Row>, HashMap<PickerCategory, f32>, f32) {
    let mut rows = Vec::new();
    let mut category_y = HashMap::new();
    let mut y = 0.0f32;
    for (cat, emoji) in by_category {
        category_y.insert(*cat, y);
        rows.push(Row {
            y,
            height: HEADER_ROW_HEIGHT,
            kind: RowKind::CategoryHeader(*cat),
        });
        y += HEADER_ROW_HEIGHT;
        let tiles: &[Tile] = if collapsed.contains(cat) { &[] } else { emoji };
        for chunk in tiles.chunks(COLUMNS) {
            rows.push(Row {
                y,
                height: TILE_ROW_HEIGHT,
                kind: RowKind::Tiles(chunk.to_vec(), rows.len()),
            });
            y += TILE_ROW_HEIGHT;
        }

        rows.push(Row {
            y,
            height: CATEGORY_GAP_PX,
            kind: RowKind::Gap,
        });
        y += CATEGORY_GAP_PX;
    }
    (rows, category_y, y)
}

fn layout_search_rows(results: &[Tile]) -> (Vec<Row>, f32) {
    let mut rows = Vec::new();
    let mut y = 0.0f32;
    for chunk in results.chunks(COLUMNS) {
        rows.push(Row {
            y,
            height: TILE_ROW_HEIGHT,
            kind: RowKind::Tiles(chunk.to_vec(), rows.len()),
        });
        y += TILE_ROW_HEIGHT;
    }
    (rows, y)
}

struct GridLayout {
    id: u64,
    rows: Vec<Row>,
    category_y: HashMap<PickerCategory, f32>,
    total_height: f32,
    listed: Rc<Vec<Tile>>,

    selection: Rc<TileSelection>,

    selection_ready: Cell<bool>,
}

thread_local! {
    static NEXT_LAYOUT_ID: Cell<u64> = const { Cell::new(1) };
}

fn new_layout(
    rows: Vec<Row>,
    category_y: HashMap<PickerCategory, f32>,
    total_height: f32,
    listed: Vec<Tile>,
) -> Rc<GridLayout> {
    let id = NEXT_LAYOUT_ID.with(|n| {
        n.set(n.get() + 1);
        n.get()
    });
    Rc::new(GridLayout {
        id,
        rows,
        category_y,
        total_height,
        listed: Rc::new(listed),
        selection: Rc::new(TileSelection {
            current: Cell::new(None),
            flags: RefCell::new(HashMap::new()),
        }),
        selection_ready: Cell::new(false),
    })
}

struct GridKey {
    sections: Option<Rc<Vec<GuildSection>>>,
    frequent: Option<Rc<Vec<Tile>>>,
    top: Option<Rc<Vec<CustomTile>>>,
    collapsed: Vec<PickerCategory>,
}

fn same_rc<T>(a: &Option<Rc<T>>, b: &Option<Rc<T>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

thread_local! {

    static GRID_LAYOUT: RefCell<Option<(GridKey, Rc<GridLayout>)>> = const { RefCell::new(None) };
}

fn rows_are_reused() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| env::var("REQUIEM_ROW_REUSE").map_or(true, |v| v != "0"))
}

fn name_contains(name: &str, query: &str) -> bool {
    if name.is_ascii() && query.is_ascii() {
        return query.is_empty()
            || name
                .as_bytes()
                .windows(query.len())
                .any(|window| window.eq_ignore_ascii_case(query.as_bytes()));
    }
    name.to_lowercase().contains(query)
}

fn grid_layout(
    query: &str,
    collapsed: &[PickerCategory],
    custom: &Option<CustomEmojiSource>,
) -> Rc<GridLayout> {
    if !query.is_empty() {
        let guild_sections = custom.as_ref().map(|c| c.sections());
        let customs = guild_sections
            .iter()
            .flat_map(|l| l.iter())
            .flat_map(|s| s.emojis.iter())
            .filter(|c| name_contains(&c.emoji.name, query))
            .cloned()
            .map(Tile::Custom);
        let results: Vec<Tile> = customs
            .chain(matching_emoji(query).map(Tile::Unicode))
            .take(200)
            .collect();
        let (rows, total_height) = layout_search_rows(&results);
        return new_layout(rows, HashMap::new(), total_height, results);
    }
    let key = GridKey {
        sections: custom.as_ref().map(|c| c.sections()),
        frequent: custom.as_ref().map(|c| c.frequent()),
        top: custom.as_ref().map(|c| c.top()),
        collapsed: collapsed.to_vec(),
    };
    let cached = GRID_LAYOUT.with(|cache| {
        cache
            .borrow()
            .as_ref()
            .filter(|(k, _)| {
                same_rc(&k.sections, &key.sections)
                    && same_rc(&k.frequent, &key.frequent)
                    && same_rc(&k.top, &key.top)
                    && k.collapsed == key.collapsed
            })
            .map(|(_, layout)| layout.clone())
    });
    if let Some(layout) = cached {
        return layout;
    }

    let guild_sections = key.sections.iter().flat_map(|l| l.iter());
    let top = key.top.iter().filter(|t| !t.is_empty()).map(|t| {
        (
            PickerCategory::TopEmoji,
            t.iter().cloned().map(Tile::Custom).collect::<Vec<_>>(),
        )
    });
    let frequent = key
        .frequent
        .iter()
        .filter(|f| !f.is_empty())
        .map(|f| (PickerCategory::Frequent, (**f).clone()));
    let guilds = guild_sections.map(|s| {
        (
            PickerCategory::Guild(s.guild_id),
            s.emojis
                .iter()
                .cloned()
                .map(Tile::Custom)
                .collect::<Vec<_>>(),
        )
    });
    let unicode = ALL_CATEGORIES.iter().map(|&cat| {
        (
            cat,
            cat.groups()
                .iter()
                .flat_map(|g| g.emojis())
                .map(Tile::Unicode)
                .collect::<Vec<_>>(),
        )
    });
    let by_category: Vec<(PickerCategory, Vec<Tile>)> =
        top.chain(frequent).chain(guilds).chain(unicode).collect();
    let listed: Vec<Tile> = by_category
        .iter()
        .filter(|(cat, _)| !collapsed.contains(cat))
        .flat_map(|(_, tiles)| tiles.iter().cloned())
        .collect();
    let (rows, category_y, total_height) = layout_rows(&by_category, collapsed);
    let layout = new_layout(rows, category_y, total_height, listed);
    GRID_LAYOUT.with(|cache| *cache.borrow_mut() = Some((key, layout.clone())));
    layout
}

fn build_grid(
    signals: &EmojiPickerSignals,
    search: &str,
    on_pick: Rc<dyn Fn(String, bool)>,
    custom: &Option<CustomEmojiSource>,
) -> BuiltGrid {
    let query = search.trim().to_lowercase();
    let _ = virtualization::WINDOW_TICK.with(Signal::get);

    let on_pick: Rc<dyn Fn(String, bool)> =
        match custom.as_ref().and_then(|c| c.on_pick_used.clone()) {
            Some(used) => {
                let inner = on_pick;
                Rc::new(move |text, keep_open| {
                    used(&text);
                    inner(text, keep_open)
                })
            }
            None => on_pick,
        };

    let jump = signals.category_jump.get();

    let collapsed = signals.collapsed.get();
    let layout = grid_layout(&query, &collapsed, custom);
    let (rows, category_y, total_height, listed) = (
        &layout.rows,
        &layout.category_y,
        layout.total_height,
        layout.listed.clone(),
    );
    let hovered_custom = CUSTOM_HOVERED.with(|h| h.peek());
    let selected: Option<Tile> = hovered_custom
        .and_then(|id| {
            listed
                .iter()
                .find(|t| matches!(t, Tile::Custom(c) if c.emoji.id.get() == id))
                .cloned()
        })
        .or_else(|| {
            signals.hovered.peek().and_then(|h| {
                listed
                    .iter()
                    .find(|t| matches!(t, Tile::Unicode(e) if ptr::eq(*e, h)))
                    .cloned()
            })
        })
        .or_else(|| listed.first().cloned());

    let selection = layout.selection.clone();
    if !layout.selection_ready.replace(true) {
        selection.current.set(selected.as_ref().and_then(|sel| {
            rows.iter().find_map(|row| match &row.kind {
                RowKind::Tiles(tiles, salt) if tiles.iter().any(|t| t.key() == sel.key()) => {
                    Some(occurrence_key(sel.key(), *salt))
                }
                _ => None,
            })
        }));
    }

    let mut offset = signals.scroll_offset.peek();
    if let Some(cat) = jump {
        if let Some(&target_y) = category_y.get(&cat) {
            offset = target_y;
            signals.scroll_offset.set(offset);
        }
        signals.category_jump.set(None);
    }

    let mut starts: Vec<(PickerCategory, f32)> =
        category_y.iter().map(|(&cat, &y)| (cat, y)).collect();
    starts.sort_by(|a, b| {
        a.1.partial_cmp(&b.1)
            .expect("row Y offsets are always finite")
    });
    let category_at = move |starts: &[(PickerCategory, f32)], offset: f32| {
        starts
            .iter()
            .rev()
            .find(|(_, y)| *y <= offset)
            .map(|(cat, _)| *cat)
    };
    let active_category = if query.is_empty() {
        category_at(&starts, offset + ACTIVE_CATEGORY_LOOKAHEAD_PX)
    } else {
        None
    };
    let flag_categories: Vec<PickerCategory> =
        special_categories(custom).chain(ALL_CATEGORIES).collect();

    let sticky_category = if query.is_empty() {
        category_at(&starts, offset)
    } else {
        None
    };
    let sticky_flags: Rc<Vec<(PickerCategory, Signal<bool>)>> = Rc::new(
        flag_categories
            .iter()
            .map(|&cat| (cat, signal(sticky_category == Some(cat))))
            .collect(),
    );
    let active_flags: Rc<Vec<(PickerCategory, Signal<bool>)>> = Rc::new(
        flag_categories
            .iter()
            .map(|&cat| (cat, signal(active_category == Some(cat))))
            .collect(),
    );

    let window_start = (offset - virtualization::OVERSCAN_PX).max(0.0);
    let window_end = offset + virtualization::VIEWPORT_PX + virtualization::OVERSCAN_PX;

    let mut column = div().class(emoji_picker::PICKER_GRID_COLUMN);

    let visible_start = rows
        .iter()
        .position(|r| r.y + r.height >= window_start && r.y <= window_end)
        .unwrap_or(rows.len());
    let visible_end = rows
        .iter()
        .rposition(|r| r.y + r.height >= window_start && r.y <= window_end)
        .map_or(visible_start, |i| i + 1);

    if visible_start > 0 {
        let leading_height = rows.get(visible_start).map_or(total_height, |r| r.y);
        column = column.child(Node::new(
            "div",
            EmojiRowPlaceholder {
                height: leading_height,
            },
        ));
    }

    if rows.is_empty() {
        column = column.child(div().class(emoji_picker::PICKER_EMPTY));
    }

    for (offset_in_window, row) in rows[visible_start..visible_end].iter().enumerate() {
        let row_key = (1u64 << 63) | (visible_start + offset_in_window) as u64;
        column = column.child(
            if rows_are_reused() && control::retained_available(row_key, layout.id) {
                Node::reuse(row_key, layout.id)
            } else {
                build_row(row, signals, &collapsed, &selection, &on_pick, custom)
                    .retained(row_key, layout.id)
            },
        );
    }

    if visible_end < rows.len() {
        let trailing_start = rows.get(visible_end).map_or(total_height, |r| r.y);
        let trailing_height = (total_height - trailing_start).max(0.0);
        column = column.child(Node::new(
            "div",
            EmojiRowPlaceholder {
                height: trailing_height,
            },
        ));
    }

    let scroll_write = signals.scroll_offset.clone();
    let live_flags = active_flags.clone();
    let live_sticky = sticky_flags.clone();
    let live_starts = starts.clone();
    let searching = !query.is_empty();
    let (built_start, built_end) = (window_start, window_end.min(total_height));
    let rebuild_requested = Cell::new(false);
    let scroll = ScrollViewProps::default()
        .initial_offset_y(offset)
        .on_scroll(move |offset_y, _max_offset_y| {
            scroll_write.set(offset_y);
            let near_top = built_start > 0.0 && offset_y < built_start + virtualization::EDGE_PX;
            let near_bottom = built_end < total_height
                && offset_y + virtualization::VIEWPORT_PX > built_end - virtualization::EDGE_PX;
            if (near_top || near_bottom) && !rebuild_requested.replace(true) {
                virtualization::WINDOW_TICK.with(|tick| tick.update(|n| *n += 1));
            }
            if searching {
                return;
            }
            let pinned = category_at(&live_starts, offset_y);
            for (cat, flag) in live_sticky.iter() {
                let want = pinned == Some(*cat);
                if flag.peek() != want {
                    flag.set(want);
                }
            }
            let now = category_at(&live_starts, offset_y + ACTIVE_CATEGORY_LOOKAHEAD_PX);
            for (cat, flag) in live_flags.iter() {
                let want = now == Some(*cat);
                if flag.peek() != want {
                    flag.set(want);
                }
            }
        })
        .build()
        .class(emoji_picker::PICKER_GRID_SCROLL)
        .child(column);

    let sticky = (!searching).then(|| {
        let mut overlay = Node::new(
            "div",
            StickyOverlayWidget {
                scroll: signals.scroll_offset.clone(),
                starts: starts.iter().map(|(_, y)| *y).collect(),
            },
        )
        .class(emoji_picker::PICKER_STICKY_HEADER);
        for (cat, flag) in sticky_flags.iter() {
            let slot = div()
                .class(emoji_picker::PICKER_STICKY_SLOT)
                .bind_class(
                    flag.clone(),
                    "",
                    emoji_picker::PICKER_STICKY_SLOT_HIDDEN,
                )
                .child(category_header(*cat, &collapsed, signals, custom));
            overlay = overlay.child(slot);
        }
        overlay
    });

    BuiltGrid {
        scroll,
        sticky,
        active_flags: (*active_flags).clone(),
        listed,
    }
}

struct StickyOverlayWidget {
    scroll: Signal<f32>,

    starts: Vec<f32>,
}

impl Widget for StickyOverlayWidget {
    fn scroll_offset(&self) -> Option<(f32, f32)> {
        let offset = self.scroll.peek();
        let push = self
            .starts
            .iter()
            .find(|&&start| start > offset + 0.5)
            .map_or(0.0, |&next| {
                (HEADER_ROW_HEIGHT - (next - offset)).max(0.0)
            });
        Some((0.0, push))
    }
}

struct BuiltGrid {
    scroll: Node,
    sticky: Option<Node>,
    active_flags: Vec<(PickerCategory, Signal<bool>)>,

    listed: Rc<Vec<Tile>>,
}

struct TileSelection {
    current: Cell<Option<usize>>,
    flags: RefCell<HashMap<usize, Signal<bool>>>,
}

impl TileSelection {
    fn flag_for(&self, key: usize) -> Signal<bool> {
        let selected = self.current.get() == Some(key);
        self.flags
            .borrow_mut()
            .entry(key)
            .or_insert_with(|| signal(selected))
            .clone()
    }

    fn select(&self, key: usize) {
        if self.current.get() == Some(key) {
            return;
        }
        let flags = self.flags.borrow();
        if let Some(previous) = self.current.get().and_then(|c| flags.get(&c)) {
            previous.set(false);
        }
        if let Some(flag) = flags.get(&key) {
            flag.set(true);
        }
        self.current.set(Some(key));
    }
}

fn build_row(
    row: &Row,
    signals: &EmojiPickerSignals,
    collapsed: &[PickerCategory],
    selection: &Rc<TileSelection>,
    on_pick: &Rc<dyn Fn(String, bool)>,
    custom: &Option<CustomEmojiSource>,
) -> Node {
    match &row.kind {
        RowKind::CategoryHeader(cat) => category_header(*cat, collapsed, signals, custom),
        RowKind::Gap => Node::new("div", EmojiRowPlaceholder { height: row.height }),
        RowKind::Tiles(tiles, salt) => {
            let salt = *salt;
            let mut tile_row = div().class(cls::PICKER_TILE_ROW);
            for tile in tiles {
                tile_row = tile_row.child(match tile {
                    Tile::Unicode(e) => build_tile(
                        e,
                        salt,
                        selection.clone(),
                        signals.hovered.clone(),
                        on_pick.clone(),
                    ),
                    Tile::Custom(c) => build_custom_tile(
                        c,
                        salt,
                        selection.clone(),
                        signals.hovered.clone(),
                        on_pick.clone(),
                        custom,
                    ),
                });
            }
            tile_row
        }
    }
}

fn category_header(
    cat: PickerCategory,
    collapsed: &[PickerCategory],
    signals: &EmojiPickerSignals,
    custom: &Option<CustomEmojiSource>,
) -> Node {
    let chevron = if collapsed.contains(&cat) {
        CHEVRON_SMALL_RIGHT
    } else {
        CHEVRON_SMALL_DOWN
    };
    let label = match (cat, custom) {
        (PickerCategory::Guild(id), Some(source)) => {
            source.section(id).map(|(name, _)| name).unwrap_or_default()
        }
        (PickerCategory::TopEmoji, Some(source)) => source.top_label(),
        _ => cat.label().to_string(),
    };
    let collapsed_signal = signals.collapsed.clone();
    let button = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::PICKER_CATEGORY_HEADER_BUTTON.to_string()])
        .on_click(move || {
            let mut collapsed = collapsed_signal.peek();
            if let Some(i) = collapsed.iter().position(|c| *c == cat) {
                collapsed.remove(i);
            } else {
                collapsed.push(cat);
            }
            collapsed_signal.set(collapsed);
        })
        .build()
        .child(match (cat, custom) {
            (PickerCategory::Guild(id), Some(source)) => {
                guild_icon_node(source, id, cls::PICKER_CATEGORY_HEADER_ICON)
            }
            _ => icon(cat.icon_path(), TEXT_DEFAULT)
                .class(cls::PICKER_CATEGORY_HEADER_ICON)
                .class(cls::PICKER_CATEGORY_HEADER_COLOR),
        })
        .child(
            text(TextProps {
                content: label,
                size: 14.0,
                color: TEXT_DEFAULT,
                weight: Weight::SEMI_BOLD.into(),
                ..Default::default()
            })
            .class(cls::PICKER_CATEGORY_HEADER_LABEL)
            .class(cls::PICKER_CATEGORY_HEADER_COLOR),
        )
        .child(
            icon(chevron, TEXT_DEFAULT)
                .class(cls::PICKER_CATEGORY_HEADER_CHEVRON)
                .class(cls::PICKER_CATEGORY_HEADER_COLOR),
        );
    div().class(cls::PICKER_CATEGORY_HEADER).child(button)
}

fn build_custom_tile(
    tile: &CustomTile,
    salt: usize,
    selection: Rc<TileSelection>,
    hovered_signal: Signal<Option<&'static Emoji>>,
    on_pick: Rc<dyn Fn(String, bool)>,
    custom: &Option<CustomEmojiSource>,
) -> Node {
    let key = occurrence_key(Tile::Custom(tile.clone()).key(), salt);
    let selected_flag = selection.flag_for(key);
    let (id, markup, locked) = (tile.emoji.id.get(), tile.emoji.markup(), tile.locked);
    let classes = vec![cls::PICKER_TILE.to_string()];
    let button = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(classes)
        .on_click_shift(move |shift: bool| {
            if !locked {
                on_pick(markup.clone(), shift);
            }
        })
        .on_hover_change(move |now_hovered: bool| {
            if now_hovered {
                selection.select(key);
                CUSTOM_HOVERED.with(|h| h.set(Some(id)));
                if hovered_signal.peek().is_some() {
                    hovered_signal.set(None);
                }
            }
        })
        .build();
    let button = match custom {
        Some(source) => button.child(tile_emoji_node(source, &tile.emoji, cls::PICKER_TILE_GLYPH)),
        None => button,
    };

    let button = if locked {
        button.child(
            div()
                .class(cls::PICKER_TILE_LOCK)
                .child(icon(LOCK, TEXT_DEFAULT).class(cls::PICKER_TILE_LOCK_ICON)),
        )
    } else {
        button
    };
    button.bind_class(selected_flag, cls::PICKER_TILE_SELECTED, "")
}

fn build_tile(
    emoji: &'static Emoji,
    salt: usize,
    selection: Rc<TileSelection>,
    hovered_signal: Signal<Option<&'static Emoji>>,
    on_pick: Rc<dyn Fn(String, bool)>,
) -> Node {
    let key = occurrence_key(Tile::Unicode(emoji).key(), salt);
    let selected_flag = selection.flag_for(key);
    let tile = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![
            emoji_picker::PICKER_TILE.to_string(),
        ])
        .on_click_shift(move |shift: bool| on_pick(emoji.as_str().to_string(), shift))
        .on_hover_change(move |now_hovered: bool| {
            if now_hovered {
                selection.select(key);
                CUSTOM_HOVERED.with(|h| {
                    if h.peek().is_some() {
                        h.set(None);
                    }
                });
                if !hovered_signal
                    .peek()
                    .is_some_and(|current| ptr::eq(current, emoji))
                {
                    hovered_signal.set(Some(emoji));
                }
            }
        })
        .build()
        .child(emoji_glyph(
            emoji.as_str(),
            emoji_picker::PICKER_TILE_GLYPH,
            40.0,
        ));
    tile.bind_class(
        selected_flag,
        emoji_picker::PICKER_TILE_SELECTED,
        "",
    )
}

fn build_footer(selected: Option<Tile>, custom: &Option<CustomEmojiSource>) -> Node {
    let footer = div().class(emoji_picker::PICKER_FOOTER);
    let emoji = match selected {
        Some(Tile::Unicode(emoji)) => emoji,
        Some(Tile::Custom(tile)) => {
            let Some(source) = custom else { return footer };
            let guild = source
                .sections()
                .iter()
                .find(|s| s.emojis.iter().any(|e| e.emoji.id == tile.emoji.id))
                .map(|s| s.name.clone())
                .unwrap_or_default();
            let label = format!(":{}:", tile.emoji.name);
            return footer
                .child(custom_emoji_node(
                    source,
                    &tile.emoji,
                    emoji_picker::PICKER_FOOTER_GLYPH,
                ))
                .child(
                    div()
                        .class(emoji_picker::PICKER_FOOTER_TEXT)
                        .child(text(TextProps {
                            content: label,
                            size: 16.0,
                            color: TEXT_STRONG,
                            weight: Weight::SEMI_BOLD.into(),
                            ..Default::default()
                        }))
                        .child(
                            div()
                                .class(emoji_picker::PICKER_FOOTER_FROM)
                                .child(text(TextProps {
                                    content: requiem_i18n::t!("emoji-picker-from"),
                                    size: 12.0,
                                    color: TEXT_SECONDARY,
                                    ..Default::default()
                                }))
                                .child(text(TextProps {
                                    content: guild,
                                    size: 12.0,
                                    color: TEXT_SECONDARY,
                                    weight: Weight::SEMI_BOLD.into(),
                                    ..Default::default()
                                })),
                        ),
                );
        }
        None => return footer,
    };
    let label = emoji_label(emoji);
    footer
        .child(emoji_glyph(
            emoji.as_str(),
            emoji_picker::PICKER_FOOTER_GLYPH,
            24.0,
        ))
        .child(text(TextProps {
            content: label,
            size: 16.0,
            color: TEXT_STRONG,
            weight: Weight::SEMI_BOLD.into(),
            ..Default::default()
        }))
}

#[cfg(test)]
mod lookup_tests {
    use crate::gateway::events::GatewayEvent;
    use super::*;

    fn state_with_emojis() -> state::AppState {
        let state = state::AppState::new();
        let guild = |id: u64, name: &str, emojis: serde_json::Value| serde_json::json!({ "id": id.to_string(), "properties": { "name": name, "icon": null, "owner_id": "1" }, "channels": [], "emojis": emojis });
        let user = serde_json::json!({ "id": "1", "username": "me", "discriminator": "0", "global_name": null, "avatar": null });
        let event = GatewayEvent::new(
            "READY",
            serde_json::json!({ "user": user, "guilds": [
                guild(10, "Other", serde_json::json!([{ "id": "1", "name": "elsewhere" }])),
                guild(20, "Home", serde_json::json!([{ "id": "2", "name": "plain" }, { "id": "3", "name": "moving", "animated": true }, { "id": "4", "name": "gone", "available": false }])),
                guild(30, "Empty", serde_json::json!([])),
            ] }),
        );
        state.apply_event(&event);
        state
    }

    #[test]
    fn the_shortcode_lookup_inserts_the_plain_emoji() {
        let ear = emoji_lookup_matches("ear")
            .into_iter()
            .find(|m| m.label == "ear")
            .expect("`ear` is offered");
        assert_eq!(
            ear.glyph, "\u{1F442}",
            "no tone in what completing (or typing `:ear:`) inserts"
        );
        assert!(emoji_lookup_matches("thumb").iter().all(|m| {
            !m.glyph
                .chars()
                .any(|c| ('\u{1F3FB}'..='\u{1F3FF}').contains(&c))
        }));
    }

    #[test]
    fn a_toned_emoji_is_named_like_its_plain_one() {
        let toned = emojis::get("\u{1F44D}\u{1F3FD}").unwrap();
        assert_eq!(
            emoji_label(toned),
            emoji_label(emojis::get("\u{1F44D}").unwrap())
        );
        assert_eq!(emoji_label(toned), ":thumbsup:");
    }

    #[test]
    fn the_current_guild_leads_and_empty_guilds_are_left_out() {
        let sections = guild_sections(&state_with_emojis(), Some(Id::new(20)), false);
        assert_eq!(
            sections.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
            ["Home", "Other"]
        );
    }

    #[test]
    fn without_nitro_only_this_guilds_still_emoji_are_usable() {
        let sections = guild_sections(&state_with_emojis(), Some(Id::new(20)), false);
        let locked = |s: &GuildSection| {
            s.emojis
                .iter()
                .map(|t| (t.emoji.name.clone(), t.locked))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            locked(&sections[0]),
            [
                ("plain".to_string(), false),
                ("moving".to_string(), true),
                ("gone".to_string(), true)
            ]
        );
        assert_eq!(locked(&sections[1]), [("elsewhere".to_string(), true)]);
        let with_nitro = guild_sections(&state_with_emojis(), Some(Id::new(20)), true);
        assert_eq!(
            locked(&with_nitro[0]),
            [
                ("plain".to_string(), false),
                ("moving".to_string(), false),
                ("gone".to_string(), true)
            ],
            "an unavailable emoji stays locked"
        );
        assert_eq!(locked(&with_nitro[1]), [("elsewhere".to_string(), false)]);
    }

    #[test]
    fn typing_a_name_offers_usable_custom_emoji_first() {
        let sections = Rc::new(guild_sections(
            &state_with_emojis(),
            Some(Id::new(20)),
            false,
        ));
        let found = (emoji_lookup(Some(Rc::new(move || sections.clone()))).search)("pla");
        assert_eq!(found[0].label, "plain");
        assert_eq!(found[0].glyph, "<:plain:2>");

        let sections = Rc::new(guild_sections(
            &state_with_emojis(),
            Some(Id::new(20)),
            false,
        ));
        assert!(
            !(emoji_lookup(Some(Rc::new(move || sections.clone()))).search)("else")
                .iter()
                .any(|m| m.label == "elsewhere")
        );
    }

    #[test]
    fn typing_smi_lists_discords_names_prefix_matches_first() {
        let labels: Vec<String> = emoji_lookup_matches("smi")
            .into_iter()
            .map(|m| m.label)
            .collect();
        assert_eq!(labels.len(), EMOJI_LOOKUP_ROWS);
        assert_eq!(labels[0], "smile", "{labels:?}");
        assert!(
            labels.iter().all(|l| l.starts_with("smi")),
            "enough prefix matches to fill the list: {labels:?}"
        );

        let more: Vec<String> = emoji_lookup_matches("slight_smi")
            .into_iter()
            .map(|m| m.label)
            .collect();
        assert_eq!(
            more.first().map(String::as_str),
            Some("slight_smile"),
            "{more:?}"
        );
    }

    #[test]
    fn the_picker_searches_discords_names_too() {
        assert!(matching_emoji("slight_smile").any(|e| e.as_str() == "\u{1F642}"));
        assert!(matching_emoji("thumbsup").any(|e| e.as_str() == "\u{1F44D}"));
    }

    #[test]
    fn matches_carry_the_emoji_to_insert() {
        let fire = emoji_lookup_matches("fire")
            .into_iter()
            .find(|m| m.label == "fire")
            .expect("fire");
        assert_eq!(fire.glyph, "\u{1F525}");
        assert!(emoji_lookup_matches("zzzzzzzz").is_empty());
    }
}

#[cfg(test)]
mod search_tests {
    use super::name_contains;

    #[test]
    fn names_match_ignoring_case_without_lowercasing_them() {
        assert!(name_contains("PogChamp", "pog"));
        assert!(name_contains("PogChamp", "champ"));
        assert!(!name_contains("PogChamp", "sad"));
        assert!(name_contains("anything", ""));
        assert!(!name_contains("ab", "abc"));
        assert!(
            name_contains("ÄPFEL", "äpfel"),
            "non-ASCII names still match"
        );
    }
}
