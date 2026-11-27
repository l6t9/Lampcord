use chrono::Datelike;
use chrono::NaiveDate;
use crate::query;
use crate::ui::classes::message_embed as classes_message_embed;
use crate::ui::classes::search as classes_search;
use crate::ui::classes::search as cls;
use crate::ui::context_menus;
use crate::ui::emoji_text;
use crate::ui::media_viewer;
use crate::ui::message_components;
use crate::ui::message_embed::ViewerLink;
use crate::ui::message_embed;
use crate::ui::message_list;
use crate::ui::search_art;
use crate::ui::search_state;
use crate::ui::twemoji;
use crate::ui;
use crate::utils::time;
use requiem_ui::colors::{INPUT_PLACEHOLDER, TEXT_MUTED, TEXT_SECONDARY};
use requiem_ui::components::TextVariant;
use requiem_ui::components::{Align, Padding, StackProps};
use requiem_ui::components::{ButtonSize, ButtonVariant};
use requiem_ui::event::NamedKey;
use requiem_ui::icons::Icon;
use requiem_ui::widget::primitives::MarkdownFeatures;
use requiem_ui::widget::primitives::PopoverPanelWidget;
use std::collections::HashSet;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::time::Duration;

use requiem_ui::components::{
    ButtonProps, MenuItem, MessageTextProps, ScrollViewProps, TextProps, div, icon, menu,
    message_text,
};
use requiem_ui::node::Node;
use requiem_ui::signal::Signal;
use requiem_ui::widget::primitives::{
    DecorationStyle, Decorator, PulsingEllipsisWidget, TextDecoration, TextInputWidget,
};
use skia_safe::Color;
use skia_safe::font_style::Weight;

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::models::id::marker::{ChannelMarker, UserMarker};
use crate::models::{Channel, Id, Message, Permissions};
use crate::query::QueryClient;
use crate::rest::HttpClient;
use crate::rest::endpoints::search::{
    self as search_api, PAGE_SIZE, SearchPage, SearchScope, SortMode,
};
use crate::state::AppState;

use crate::ui::search_state::{
    Answer, Candidates, FilterKind, ItemAction, NO_ROW, PageItem, PopoutItem, PopoutRow, RowIcon,
    SearchState, Segment, Submitted, answer_kind, parse, popout_rows, with_answer,
};
use crate::ui::{Fonts, TEXT_DEFAULT, TEXT_STRONG, TEXT_SUBTLE};
use requiem_ui::icons::{
    ARROW_ANGLE_RIGHT_UP, ARROWS_UP_DOWN, AT, ATTACHMENT, CHEVRON_SMALL_LEFT, CHEVRON_SMALL_RIGHT,
    EMBED, FILE, FILTERS_HORIZONTAL, FOLDER, IMAGE, LINK, MAGNIFYING_GLASS, POLLS, SOUNDBOARD,
    STICKER, TEXT, TRASH, USER, VIDEO, X_SMALL,
};

pub const BAR_DOM_ID: &str = "search-bar";
pub const POPOUT_DOM_ID: &str = "search-popout";

pub const SORT_MENU_DOM_ID: &str = "search-sort-menu";
pub const SORT_BUTTON_DOM_ID: &str = "search-sort-button";

pub fn scope_for(channel: &Channel) -> SearchScope {
    match channel.guild_id {
        Some(guild_id) => SearchScope::Guild(guild_id),
        None => SearchScope::Channel(channel.id),
    }
}

pub fn placeholder_for(state: &AppState, channel: &Channel) -> String {
    if let Some(dm) = state.dms.find(channel.id).filter(|dm| !dm.is_group()) {
        if let Some(user) = dm.recipients.first().and_then(|id| state.users.get(*id)) {
            return requiem_i18n::t!("search-search-server", name = user.username);
        }
    }
    match channel.guild_id.and_then(|id| {
        state
            .guilds
            .list
            .with(|guilds| guilds.iter().find(|g| g.id == id).map(|g| g.name.clone()))
    }) {
        Some(name) => requiem_i18n::t!("search-search-server", name = name),
        None => requiem_i18n::t!("search-search").to_string(),
    }
}

pub fn candidates(state: &AppState, channel: &Channel) -> Candidates {
    let mut users: Vec<(u64, String, String, String, Option<String>)> = Vec::new();
    let roster = channel
        .guild_id
        .map(|guild_id| {
            state.members.by_guild.with_untracked(|by_guild| {
                by_guild
                    .get(&guild_id)
                    .map(|m| m.values().cloned().collect::<Vec<_>>())
            })
        })
        .unwrap_or_default();
    match roster.filter(|r| !r.is_empty()) {
        Some(members) => {
            for member in members {
                let user = &member.user;
                let display = user.display_name(member.nick.as_deref());
                users.push((
                    user.id.get(),
                    display,
                    user.username.clone(),
                    user.discriminator.clone(),
                    user.avatar.clone(),
                ));
            }
        }
        None => state.messages.by_channel.with_untracked(|by_channel| {
            let mut seen = HashSet::new();
            for message in by_channel.get(&channel.id).into_iter().flatten() {
                if seen.insert(message.author.id) {
                    let user = state.users.resolve(&message.author);
                    let display = user
                        .global_name
                        .clone()
                        .unwrap_or_else(|| user.username.clone());
                    users.push((
                        user.id.get(),
                        display,
                        user.username.clone(),
                        user.discriminator.clone(),
                        user.avatar.clone(),
                    ));
                }
            }
        }),
    }
    users.sort_by_key(|u| u.1.to_lowercase());

    let channels = channel
        .guild_id
        .map(|guild_id| {
            state.channels.by_guild.with_untracked(|by_guild| {
                let mut list: Vec<&Channel> = by_guild
                    .get(&guild_id)
                    .into_iter()
                    .flatten()
                    .filter(|c| {
                        !matches!(c.kind, 2 | 4 | 13) && state.can(c, Permissions::VIEW_CHANNEL)
                    })
                    .collect();
                list.sort_by_key(|c| (c.position.unwrap_or(0), c.id.get()));
                list.into_iter()
                    .filter_map(|c| Some((c.id.get(), c.name.clone()?)))
                    .collect()
            })
        })
        .unwrap_or_default();
    Candidates { users, channels }
}

#[derive(Clone)]
struct ChipCtx {
    state: AppState,
    images: ImageCache,
    backend: BackendConfig,
}

impl ChipCtx {
    fn user(&self, id: u64) -> Option<(String, Option<String>)> {
        let user_id = Id::<UserMarker>::new(id);
        self.state.members.by_guild.with_untracked(|by_guild| {
            by_guild.values().find_map(|members| {
                members
                    .get(&user_id)
                    .map(|m| (m.user.discriminator.clone(), m.user.avatar.clone()))
            })
        })
    }

    fn avatar(&self, id: u64, fonts: &Fonts) -> Option<Node> {
        let (discriminator, hash) = self.user(id)?;
        let user_id = Id::<UserMarker>::new(id);
        let mut props = ui::user_avatar_props(user_id, &discriminator, fonts);
        if let Some(image) = hash.as_deref().and_then(|h| {
            self.images
                .get_or_decode_image(&self.backend.avatar_url(user_id, h))
        }) {
            props = props.image(image);
        }
        Some(props.build().class(cls::CHIP_AVATAR))
    }
}

#[derive(Clone)]
struct ActionEnv {
    search: SearchState,
    scope: Option<SearchScope>,
}

impl ActionEnv {
    fn run(&self, action: ItemAction, kind: Option<FilterKind>) {
        let search = &self.search;
        let text = search.text.peek();
        let parsed = search.parsed(&text);
        match action {
            ItemAction::AddFilter(kind) => {
                let mut next = text.trim_end().to_string();
                if !next.is_empty() {
                    next.push(' ');
                }
                next.push_str(&format!("{}:", kind.keyword()));
                search.show_filters.set(false);
                search.selected.set(NO_ROW);
                search.set_text(next);
            }
            ItemAction::Answer(answer) => {
                let next = match (parsed.pending(&text), kind) {
                    (Some((pending, keyword_end, _)), _) => {
                        with_answer(&text, Some(keyword_end), pending, &answer)
                    }

                    (None, Some(kind)) => with_answer(&text, None, kind, &answer),
                    (None, None) => text.clone(),
                };
                search.show_filters.set(false);
                search.selected.set(NO_ROW);
                search.set_text(next);
            }
            ItemAction::Submit => {
                if let Some(scope) = self.scope {
                    search.submit(scope, parsed.query(&text));
                }
            }
            ItemAction::History(index) => {
                let query = search.history.with_untracked(|h| h.get(index).cloned());
                if let (Some(query), Some(scope)) = (query, self.scope) {
                    search.load(&query);
                    search.submit(scope, query);
                }
            }
            ItemAction::ShowFilters => {
                search.show_filters.set(true);
                search.selected.set(NO_ROW);
            }
            ItemAction::MoreFilters => search.open_filters(),
        }
    }
}

fn is_active(search: &SearchState) -> bool {
    let text = search.text.peek();
    let parsed = search.parsed(&text);
    parsed.pending(&text).is_some() || !parsed.free_text(&text).is_empty()
}

fn effective_selected(selected: usize, active: bool, rows: usize) -> usize {
    match selected {
        NO_ROW if active && rows > 0 => 0,
        s if s < rows => s,
        _ => NO_ROW,
    }
}

pub fn build_bar(
    state: &AppState,
    channel: Option<&Channel>,
    search: &SearchState,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
) -> Node {
    let scope = channel.map(scope_for);
    let placeholder = channel
        .map(|c| placeholder_for(state, c))
        .unwrap_or_else(|| requiem_i18n::t!("search-search").to_string());
    let env = ActionEnv {
        search: search.clone(),
        scope,
    };
    if let Some(channel) = channel {
        *search.candidates.borrow_mut() = Rc::new(candidates(state, channel));
    }

    let bar = {
        let (search, fonts, env) = (search.clone(), fonts.clone(), env.clone());
        Node::reactive(move || build_bar_contents(&search, &fonts, &env, &placeholder))
            .class(cls::BAR_FILL)
    };

    let mut wrap = div().class(cls::BAR_WRAP).dom_id(BAR_DOM_ID).child(bar);
    if let Some(channel) = channel {
        let popout = {
            let (state, channel, search, fonts, env, images, backend) = (
                state.clone(),
                channel.clone(),
                search.clone(),
                fonts.clone(),
                env,
                images.clone(),
                backend.clone(),
            );
            Node::reactive(move || {
                build_popout(&state, &channel, &search, &fonts, &env, &images, &backend)
            })
        };
        wrap = wrap.child(popout);
    }
    wrap
}

fn build_bar_contents(
    search: &SearchState,
    fonts: &Fonts,
    env: &ActionEnv,
    placeholder: &str,
) -> Node {
    let open = search.popout_open.get();

    let _ = search.text_version.get();
    let has_content = search.has_text.get() || search.submitted.get().is_some();

    let chip = Color::from_rgb(0x2c, 0x2d, 0x32);
    let styles = vec![
        DecorationStyle {
            weight: 600,
            color: TEXT_STRONG,
            background: Some(chip),
            radii: [4.0, 0.0, 0.0, 4.0],
            pad_left: 4.0,
            pad_right: 0.0,
        },
        DecorationStyle {
            weight: 500,
            color: TEXT_STRONG,
            background: Some(chip),
            radii: [0.0, 4.0, 4.0, 0.0],
            pad_left: 0.0,
            pad_right: 4.0,
        },
    ];
    let candidates = search.candidates.clone();
    let decorator: Decorator = Rc::new(move |text: &str| {
        let candidates = candidates.borrow().clone();
        parse(text, &candidates)
            .decorations()
            .into_iter()
            .map(|(range, keyword)| TextDecoration {
                range,
                style: usize::from(!keyword),
            })
            .collect()
    });

    let field = {
        let (change_search, key_search) = (search.clone(), search.clone());
        let key_env = env.clone();
        let widget = TextInputWidget::new(
            search.text.peek(),
            placeholder.to_string(),
            14.0,
            TEXT_DEFAULT,
            INPUT_PLACEHOLDER,
            fonts.text_fonts.font_system.clone(),
            fonts.text_fonts.glyph_cache.clone(),
            open,
        )
        .with_insets(if search.has_text.peek() { 4.0 } else { 0.0 }, 0.0)
        .with_line_height(22.0)
        .with_decorator(styles, decorator)
        .with_on_change(move |text| {
            change_search.typed(text);
            change_search.show_filters.set(false);
            change_search.selected.set(NO_ROW);
            change_search.popout_open.set(true);
        })
        .with_key_hook(move |key, _shift, text| handle_key(&key_search, &key_env, key, text));
        let mut node = Node::new("div", widget).class(cls::INPUT);

        if open {
            node = node.wants_focus();
        }
        node
    };
    let editor = div().class(cls::EDITOR).child(field);

    let glyph = if has_content {
        let clear = search.clone();
        ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::ICON_WRAP.to_string()])
            .on_click(move || {
                clear.clear();
                clear.popout_open.set(true);
            })
            .build()
            .child(icon(X_SMALL, TEXT_SECONDARY).class(cls::CLEAR_GLYPH))
    } else {
        div()
            .class(cls::ICON_WRAP)
            .child(icon(MAGNIFYING_GLASS, TEXT_SECONDARY).class(cls::ICON_GLYPH))
    };

    div().class(cls::BAR).child(editor).child(glyph)
}

fn segment_nodes(segment: &Segment, fonts: &Fonts, chips: &ChipCtx) -> Vec<Node> {
    match segment {
        Segment::Text(text) => vec![
            TextProps::styled(text.clone(), TextVariant::TEXT_SM_MEDIUM)
                .line_height(22.0_f32)
                .color(TEXT_DEFAULT)
                .line_clamp(1_u32)
                .build()
                .class(cls::PLAIN_TEXT),
        ],
        Segment::Filter(token) => {
            let mut nodes = vec![
                div().class(cls::FILTER_CHIP).child(
                    TextProps::styled(
                        format!("{}:", token.kind.keyword()),
                        TextVariant::TEXT_SM_SEMIBOLD,
                    )
                    .color(TEXT_STRONG)
                    .line_clamp(1_u32)
                    .build(),
                ),
            ];
            if let Some(answer) = &token.answer {
                let avatar = matches!(token.kind, FilterKind::From | FilterKind::Mentions)
                    .then(|| {
                        answer
                            .value
                            .parse::<u64>()
                            .ok()
                            .and_then(|id| chips.avatar(id, fonts))
                    })
                    .flatten();
                let mut chip = div().class(cls::ANSWER_CHIP);
                chip = match avatar {
                    Some(avatar) => chip.child(avatar).child(
                        TextProps::styled(answer.label.clone(), TextVariant::TEXT_SM_MEDIUM)
                            .color(TEXT_STRONG)
                            .line_clamp(1_u32)
                            .build()
                            .class(cls::ANSWER_CHIP_TEXT_AFTER_AVATAR),
                    ),
                    None => chip.child(
                        TextProps::styled(answer.label.clone(), TextVariant::TEXT_SM_MEDIUM)
                            .color(TEXT_STRONG)
                            .line_clamp(1_u32)
                            .build()
                            .class(cls::ANSWER_CHIP_TEXT),
                    ),
                };
                nodes.push(chip);
            }
            nodes
        }
    }
}

fn handle_key(
    search: &SearchState,
    env: &ActionEnv,
    key: NamedKey,
    _text: &str,
) -> bool {
    let rows = search.actions.borrow().len();
    let active = is_active(search);
    match key {
        NamedKey::ArrowDown | NamedKey::ArrowUp if rows > 0 && search.popout_open.peek() => {
            let current = effective_selected(search.selected.peek(), active, rows);
            let next = match (current, key) {
                (NO_ROW, NamedKey::ArrowDown) => 0,
                (NO_ROW, _) => rows - 1,
                (c, NamedKey::ArrowDown) => (c + 1) % rows,
                (c, _) => (c + rows - 1) % rows,
            };
            search.selected.set(next);
            true
        }
        NamedKey::Enter => {
            let current = effective_selected(search.selected.peek(), active, rows);
            let picked = search.actions.borrow().get(current).cloned();
            match picked {
                Some((action, kind)) if search.popout_open.peek() => env.run(action, kind),

                _ if !search.query().is_empty() => env.run(ItemAction::Submit, None),
                _ => {}
            }
            true
        }
        NamedKey::Escape
            if search.popout_open.peek() => {
                search.popout_open.set(false);
                true
            }

        _ => false,
    }
}

fn row_icon_path(icon: RowIcon) -> Option<Icon> {
    Some(match icon {
        RowIcon::Search => MAGNIFYING_GLASS,
        RowIcon::More => FILTERS_HORIZONTAL,
        RowIcon::Channel => TEXT,
        RowIcon::User => return None,
        RowIcon::None => return None,
        RowIcon::Filter(kind) => match kind {
            FilterKind::From => USER,
            FilterKind::In => TEXT,
            FilterKind::Has => ATTACHMENT,
            FilterKind::Mentions => AT,
            _ => FILTERS_HORIZONTAL,
        },
        RowIcon::Has(value) => match value {
            "image" => IMAGE,
            "video" => VIDEO,
            "link" => LINK,
            "file" => FILE,
            "embed" => EMBED,
            "sound" => SOUNDBOARD,
            "poll" => POLLS,
            "sticker" => STICKER,
            _ => ARROW_ANGLE_RIGHT_UP,
        },
    })
}

fn build_popout(
    state: &AppState,
    channel: &Channel,
    search: &SearchState,
    fonts: &Fonts,
    env: &ActionEnv,
    images: &ImageCache,
    backend: &BackendConfig,
) -> Node {
    if !search.popout_open.get() {
        search.actions.borrow_mut().clear();
        return div();
    }
    *search.candidates.borrow_mut() = Rc::new(candidates(state, channel));
    let text = search.text.get();
    let parsed = search.parsed(&text);
    if parsed
        .pending(&text)
        .is_some_and(|(kind, _, _)| kind.is_date())
    {
        search.actions.borrow_mut().clear();
        let _ = search.calendar_month.get();
        let env = env.clone();
        let pick: Rc<dyn Fn(String)> = Rc::new(move |date: String| {
            env.run(
                ItemAction::Answer(Answer {
                    value: date.clone(),
                    label: date,
                }),
                None,
            )
        });
        return calendar_panel(search, pick, classes_search::CALENDAR_IN_BAR);
    }
    let show_filters = search.show_filters.get();
    let history = search.history.get();
    let rows = popout_rows(
        &text,
        &parsed,
        show_filters,
        &history,
        &search.candidates.borrow(),
    );
    if rows.is_empty() {
        search.actions.borrow_mut().clear();
        return div();
    }

    let actions: Vec<(ItemAction, Option<FilterKind>)> = rows
        .iter()
        .filter_map(|row| match row {
            PopoutRow::Item(item) => Some((item.action.clone(), answer_kind(item))),
            _ => None,
        })
        .collect();
    let selected = effective_selected(search.selected.get(), is_active(search), actions.len());
    *search.actions.borrow_mut() = actions;

    let chips = ChipCtx {
        state: state.clone(),
        images: images.clone(),
        backend: backend.clone(),
    };
    let mut panel = Node::new(
        "div",
        PopoverPanelWidget::new(),
    )
    .class(cls::POPOUT)
    .dom_id(POPOUT_DOM_ID);
    let mut index = 0;
    for row in &rows {
        panel = panel.child(match row {
            PopoutRow::Header {
                label,
                clear_history,
            } => header_node(label, *clear_history, search),
            PopoutRow::Divider => div().class(cls::DIVIDER),
            PopoutRow::Item(item) => {
                let node = item_node(item, index == selected, env, fonts, images, backend, &chips);
                index += 1;
                node
            }
        });
    }
    panel
}

fn header_node(label: &str, clear_history: bool, search: &SearchState) -> Node {
    let mut header = div().class(cls::GROUP_HEADER).child(
        TextProps::styled(label.to_string(), TextVariant::TEXT_XS_SEMIBOLD)
            .color(TEXT_SECONDARY)
            .line_clamp(1_u32)
            .build(),
    );
    if clear_history {
        let search = search.clone();
        header = header.child(
            ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![cls::CLEAR_HISTORY.to_string()])
                .on_click(move || search.history.set(Vec::new()))
                .build()
                .child(icon(TRASH, TEXT_SECONDARY).class(cls::CLEAR_HISTORY_GLYPH)),
        );
    }
    header
}

fn item_node(
    item: &PopoutItem,
    selected: bool,
    env: &ActionEnv,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    chips: &ChipCtx,
) -> Node {
    let mut content: Vec<Node> = Vec::new();

    if let Some((id, discriminator, avatar_hash)) = &item.avatar {
        let user_id = Id::<UserMarker>::new(*id);
        let mut avatar = ui::user_avatar_props(user_id, discriminator, fonts);
        if let Some(image) = avatar_hash
            .as_deref()
            .and_then(|hash| images.get_or_decode_image(&backend.avatar_url(user_id, hash)))
        {
            avatar = avatar.image(image);
        }
        content.push(avatar.build().class(cls::ITEM_AVATAR));
    } else if let Some(path) = row_icon_path(item.icon) {
        content.push(icon(path, TEXT_MUTED).class(cls::ITEM_ICON));
    }

    if let Some(query) = &item.query {
        let mut chip_row = div().class(cls::HISTORY_CHIPS);
        for segment in &query.segments {
            chip_row = chip_row.children(segment_nodes(segment, fonts, chips));
        }
        if !query.text.is_empty() {
            chip_row = chip_row.child(
                TextProps::styled(query.text.clone(), TextVariant::TEXT_SM_SEMIBOLD)
                    .color(TEXT_STRONG)
                    .line_clamp(1_u32)
                    .build(),
            );
        }
        let mut text = div().class(cls::ITEM_TEXT);
        if item.subtitle.is_some() {
            text = text.child(
                TextProps::styled(item.title.clone(), TextVariant::TEXT_SM_MEDIUM)
                    .color(TEXT_STRONG)
                    .line_clamp(1_u32)
                    .build(),
            );
            if let Some((prefix, rest)) = &item.subtitle {
                text = text.child(sub_row(prefix, rest));
            }
        } else {
            text = text.child(chip_row);
        }
        content.push(text);
    } else {
        let mut text = div().class(cls::ITEM_TEXT);
        let mut title = StackProps::horizontal()
            .align(Align::Center)
            .gap(4)
            .full_width(false)
            .build()
            .child(
                TextProps::new(item.title.clone(), 14.0_f32)
                    .weight(if item.subtitle.is_some() {
                        Weight::MEDIUM
                    } else {
                        Weight::SEMI_BOLD
                    })
                    .color(TEXT_STRONG)
                    .line_clamp(1_u32)
                    .build(),
            );
        if let Some(suffix) = &item.title_suffix {
            title = title.child(
                TextProps::styled(suffix.clone(), TextVariant::TEXT_SM_MEDIUM)
                    .color(TEXT_SUBTLE)
                    .line_clamp(1_u32)
                    .build(),
            );
        }
        text = text.child(title);
        if let Some((prefix, rest)) = &item.subtitle {
            text = text.child(sub_row(prefix, rest));
        }
        content.push(text);
    }

    let (env, action, kind) = (env.clone(), item.action.clone(), answer_kind(item));
    let mut row = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::ITEM.to_string()])
        .on_click(move || env.run(action.clone(), kind))
        .build()
        .children(content);
    if selected {
        row = row.class(cls::ITEM_SELECTED);
    }
    row
}

fn sub_row(prefix: &str, rest: &str) -> Node {
    let mut row = StackProps::horizontal()
        .align(Align::Center)
        .gap(4)
        .full_width(false)
        .build();
    if !prefix.is_empty() {
        row = row.child(
            TextProps::styled(prefix.to_string(), TextVariant::TEXT_SM_SEMIBOLD)
                .color(TEXT_SECONDARY)
                .line_clamp(1_u32)
                .build(),
        );
    }
    row.child(
        TextProps::styled(rest.to_string(), TextVariant::TEXT_SM_MEDIUM)
            .color(TEXT_MUTED)
            .line_clamp(1_u32)
            .build(),
    )
}

fn calendar_month(search: &SearchState) -> (i32, u32) {
    match search.calendar_month.get() {
        (0, 0) => {
            let today = chrono::Local::now().date_naive();
            (today.year(), today.month())
        }
        chosen => chosen,
    }
}

pub fn calendar_panel(search: &SearchState, on_pick: Rc<dyn Fn(String)>, placement: &str) -> Node {
    let (year, month) = calendar_month(search);
    let today = chrono::Local::now().date_naive();
    let Some(first) = NaiveDate::from_ymd_opt(year, month, 1) else {
        return div();
    };
    let this_month = (today.year(), today.month());

    let title = time::month_year_long(&first).to_uppercase();
    let shift = |delta: i32| {
        let search = search.clone();
        move || {
            let index = year * 12 + month as i32 - 1 + delta;
            search
                .calendar_month
                .set((index.div_euclid(12), index.rem_euclid(12) as u32 + 1));
        }
    };

    let arrow = |class: &str, path: Icon, on_click: Box<dyn Fn()>| {
        ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::CAL_ARROW.to_string(), class.to_string()])
            .on_click(on_click)
            .build()
            .child(icon(path, TEXT_STRONG).class(cls::CAL_ARROW_ICON))
    };
    let mut inner = div().class(cls::CAL_INNER).child(arrow(
        cls::CAL_ARROW_PREV,
        CHEVRON_SMALL_LEFT,
        Box::new(shift(-1)),
    ));
    if (year, month) < this_month {
        inner = inner.child(arrow(
            cls::CAL_ARROW_NEXT,
            CHEVRON_SMALL_RIGHT,
            Box::new(shift(1)),
        ));
    }

    let title_node = TextProps::new(title, 13.0_f32)
        .line_height(20.0_f32)
        .weight(Weight::SEMI_BOLD)
        .color(TEXT_STRONG)
        .center(true)
        .build()
        .class(cls::CAL_TITLE_TEXT);
    let mut names = div().class(cls::CAL_NAMES);
    for day in ["SU", "MO", "TU", "WE", "TH", "FR", "SA"] {
        names = names.child(
            div().class(cls::CAL_NAME).child(
                TextProps::new(day, 13.0_f32)
                    .line_height(13.0_f32)
                    .weight(Weight::MEDIUM)
                    .color(Color::from_argb(0x99, 0xfb, 0xfb, 0xfb))
                    .line_clamp(1_u32)
                    .build(),
            ),
        );
    }

    let mut grid = div().class(cls::CAL_GRID);
    let start = first - chrono::Duration::days(i64::from(first.weekday().num_days_from_sunday()));
    for week in 0..6i64 {
        let mut row = div().class(cls::CAL_WEEK);
        for weekday in 0..7i64 {
            let date = start + chrono::Duration::days(week * 7 + weekday);
            let in_month = date.month() == month;
            let disabled = date > today;
            let is_today = date == today;
            let faded = disabled || !in_month;
            let color = if faded {
                Color::from_argb(0x4d, 0xfb, 0xfb, 0xfb)
            } else {
                Color::from_rgb(0xfb, 0xfb, 0xfb)
            };
            let weight = if is_today {
                Weight::BOLD
            } else {
                Weight::MEDIUM
            };
            let label = TextProps::new(date.day().to_string(), 12.8_f32)
                .line_height(40.0_f32)
                .weight(weight)
                .color(color)
                .line_clamp(1_u32)
                .build();
            let mut cell = if disabled {
                div()
                    .class(cls::CAL_DAY)
                    .class(cls::CAL_DAY_DISABLED)
                    .child(label)
            } else {
                let (on_pick, value) = (on_pick.clone(), date.format("%Y-%m-%d").to_string());
                ButtonProps::default()
                    .variant(ButtonVariant::Bare)
                    .extra_classes(vec![cls::CAL_DAY.to_string()])
                    .on_click(move || on_pick(value.clone()))
                    .build()
                    .child(label)
            };
            if weekday == 6 {
                cell = cell.class(cls::CAL_DAY_LAST_COL);
            }
            if week == 5 {
                cell = cell.class(cls::CAL_DAY_LAST_ROW);
            }
            if week == 0 && weekday == 0 {
                cell = cell.class(cls::CAL_DAY_FIRST);
            }
            if is_today {
                cell = cell.child(div().class(cls::CAL_TODAY_BAR));
            }
            row = row.child(cell);
        }
        grid = grid.child(row);
    }
    let month_box = div()
        .class(cls::CAL_MONTH)
        .child(div().class(cls::CAL_TITLE).child(title_node))
        .child(names)
        .child(grid);
    Node::new(
        "div",
        PopoverPanelWidget::new(),
    )
    .class(cls::CALENDAR)
    .class(placement.to_string())
    .dom_id(POPOUT_DOM_ID)
    .child(inner.child(month_box))
}

pub struct ResultsEnv<'a> {
    pub state: &'a AppState,
    pub search: &'a SearchState,
    pub fonts: &'a Fonts,
    pub images: &'a ImageCache,
    pub backend: &'a BackendConfig,
    pub queries: &'a QueryClient,
    pub http: Option<&'a HttpClient>,
    pub media_viewer: &'a media_viewer::MediaViewerState,

    pub tick: &'a Signal<u64>,
    #[cfg(feature = "video")]
    pub video_ctx: Option<message_embed::VideoContext<'a>>,
}

const CARD_CONTENT_WIDTH: f32 = 288.0;

fn results_key(submitted: &Submitted, sort: SortMode, page: u32) -> String {
    let mut params = submitted.query.params();
    params.sort = sort;
    format!("search:{:?}:{params:?}:{page}", submitted.scope)
}

pub fn build_results(env: &ResultsEnv) -> Node {
    let search = env.search;
    let Some(submitted) = search.submitted.get() else {
        return div();
    };
    let sort = search.sort.get();
    let page = search.page.get();

    let mut params = submitted.query.params();
    params.sort = sort;
    let key = results_key(&submitted, sort, page);
    let result: Option<query::QueryResult<SearchPage>> = env.http.map(|http| {
        let (http, scope, fetch_params) = (http.clone(), submitted.scope, params.clone());
        env.queries.query(
            &key,
            Duration::from_secs(30),
            move || async move { search_api::search(&http, scope, &fetch_params, page).await },
        )
    });
    let loading = result
        .as_ref()
        .is_none_or(|r| r.data.is_none() && r.error.is_none());
    if loading {
        let _ = env.tick.get();
    }
    let data = result.as_ref().and_then(|r| r.data.clone());
    let failed = result.as_ref().is_some_and(|r| r.is_hard_error());

    let total = data.as_ref().map_or(0, |d| d.total_results);
    let mut panel = div()
        .class(cls::RESULTS)
        .child(build_results_header(env, &params, total, loading, failed));

    let body = match (&data, failed) {
        (Some(page_data), _) if page_data.total_results > 0 => {
            build_results_list(env, &submitted, page_data)
        }
        (Some(_), _) => empty_state(&requiem_i18n::t!(
            "search-we-searched-far-and-wide-unfortunately"
        )),
        (None, true) => empty_state(&requiem_i18n::t!(
            "search-something-went-wrong-please-try-searchin"
        )),
        (None, false) => div().class(cls::LOADING).child(Node::new(
            "div",
            PulsingEllipsisWidget::new(TEXT_DEFAULT, 1.0),
        )),
    };
    panel = panel.child(body);

    let pages = total.div_ceil(PAGE_SIZE);
    if pages > 1 {
        panel = panel.child(build_pager(env, pages, page));
    }
    panel
}

fn build_results_header(
    env: &ResultsEnv,
    params: &search_api::SearchParams,
    total: u32,
    loading: bool,
    failed: bool,
) -> Node {
    let search = env.search;
    let count_label = match (loading, failed, total) {
        (true, _, _) => String::new(),
        (_, true, _) | (_, _, 0) => requiem_i18n::t!("search-no-results").to_string(),
        (_, _, 1) => requiem_i18n::t!("search-1-result").to_string(),
        (_, _, n) => requiem_i18n::t!("search-n-results", count = n),
    };
    let filters = params.filter_count();
    let filters_label = if filters > 0 {
        requiem_i18n::t!("search-filters-count", count = filters)
    } else {
        requiem_i18n::t!("search-filters-filters")
    };
    let open_filters = search.clone();
    let filters_button = ButtonProps::default()
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Small)
        .label(filters_label)
        .icon(FILTERS_HORIZONTAL)
        .on_click(move || open_filters.open_filters())
        .build();

    let sort_open = search.sort_menu_open.clone();
    let sort_button = ButtonProps::default()
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Small)
        .label(requiem_i18n::t!("search-sort").to_string())
        .icon(ARROWS_UP_DOWN)
        .on_click({
            let sort_open = sort_open.clone();
            move || sort_open.set(!sort_open.peek())
        })
        .build()
        .dom_id(SORT_BUTTON_DOM_ID);

    let mut buttons = div()
        .class(cls::RESULTS_BUTTONS)
        .child(filters_button)
        .child(sort_button);
    if search.sort_menu_open.get() {
        buttons = buttons.child(build_sort_menu(env));
    }
    div()
        .class(cls::RESULTS_HEADER)
        .child(
            TextProps::styled(count_label, TextVariant::TEXT_MD_MEDIUM)
                .color(TEXT_STRONG)
                .line_clamp(1_u32)
                .build(),
        )
        .child(buttons)
}

fn build_sort_menu(env: &ResultsEnv) -> Node {
    let current = env.search.sort.get();
    let items = SortMode::ALL
        .into_iter()
        .map(|mode| {
            let search = env.search.clone();
            MenuItem::radio(mode.label(), mode == current, move || {
                search.sort.set(mode);
                search.page.set(1);
                search.sort_menu_open.set(false);
            })
        })
        .collect();
    menu(items).class(cls::SORT_MENU).dom_id(SORT_MENU_DOM_ID)
}

fn empty_state(message: &str) -> Node {
    let lines = TextProps::styled(message.to_string(), TextVariant::TEXT_MD_MEDIUM)
        .line_height(24.0_f32)
        .color(TEXT_DEFAULT)
        .center(true)
        .build()
        .class(cls::EMPTY_TEXT);
    div()
        .class(cls::EMPTY)
        .child(Node::new("div", search_art::NoResultsArt).class(cls::EMPTY_IMAGE))
        .child(lines)
}

fn build_results_list(env: &ResultsEnv, submitted: &Submitted, page: &SearchPage) -> Node {
    let terms = submitted.query.terms();
    let mut groups: BTreeMap<(i32, u64), (Option<Channel>, Vec<&Message>)> = BTreeMap::new();
    for message in page.hits() {
        let channel = ui::find_channel(env.state, message.channel_id)
            .map(|(_, c)| c)
            .or_else(|| {
                let mut found = None;
                env.state.channels.by_guild.with_untracked(|by_guild| {
                    found = by_guild
                        .values()
                        .flatten()
                        .find(|c| c.id == message.channel_id)
                        .cloned();
                });
                found
            });
        let order = channel
            .as_ref()
            .map_or(i32::MAX, |c| c.position.unwrap_or(0));
        groups
            .entry((order, message.channel_id.get()))
            .or_insert_with(|| (channel, Vec::new()))
            .1
            .push(message);
    }

    let mut list = StackProps::vertical()
        .gap(0)
        .padding(Padding::top(16).with_right(16).with_left(16))
        .build();
    for (_, (channel, messages)) in groups {
        let mut group = div().class(cls::GROUP);
        group = group.child(channel_header(
            env,
            channel.as_ref(),
            messages.first().map(|m| m.channel_id),
        ));
        for message in messages {
            group = group.child(build_card(env, channel.as_ref(), message, &terms));
        }
        list = list.child(group);
    }
    ScrollViewProps::default()
        .build()
        .class(cls::RESULTS_SCROLL)
        .child(list)
}

fn channel_header(
    env: &ResultsEnv,
    channel: Option<&Channel>,
    fallback_id: Option<Id<ChannelMarker>>,
) -> Node {
    let name = channel
        .and_then(|c| c.name.clone())
        .unwrap_or_else(|| fallback_id.map_or_else(String::new, |id| id.to_string()));
    let mut row = div()
        .class(cls::CHANNEL_NAME_ROW)
        .child(
            icon(
                channel.map_or(TEXT, |c| ui::channel_icon(env.state, c)),
                TEXT_STRONG,
            )
            .class(cls::CHANNEL_NAME_ICON),
        )
        .child(
            TextProps::styled(name, TextVariant::TEXT_MD_MEDIUM)
                .color(TEXT_STRONG)
                .line_clamp(1_u32)
                .build(),
        )
        .child(div().class(cls::CHANNEL_NAME_SPACER));

    let parent = channel.and_then(|c| c.parent_id).and_then(|parent_id| {
        env.state.channels.by_guild.with_untracked(|by_guild| {
            by_guild
                .values()
                .flatten()
                .find(|c| c.id == parent_id)
                .and_then(|c| c.name.clone())
        })
    });
    if let Some(parent) = parent {
        row = row.child(
            div()
                .class(cls::PARENT_NAME)
                .child(icon(FOLDER, TEXT_DEFAULT).class(cls::PARENT_ICON))
                .child(
                    TextProps::styled(parent, TextVariant::TEXT_XS_MEDIUM)
                        .color(TEXT_DEFAULT)
                        .line_clamp(1_u32)
                        .build(),
                ),
        );
    }
    row
}

fn mention_maps(
    env: &ResultsEnv,
    message: &Message,
    channel: Option<&Channel>,
) -> (Rc<HashMap<u64, String>>, Rc<HashMap<u64, String>>) {
    let mut mentions = HashMap::new();
    if let Some(guild_id) = channel.and_then(|c| c.guild_id) {
        env.state.members.by_guild.with_untracked(|by_guild| {
            for member in by_guild.get(&guild_id).into_iter().flat_map(|m| m.values()) {
                let display = member.user.display_name(member.nick.as_deref());
                mentions.insert(member.user.id.get(), display);
            }
        });
    }
    for user in &message.mentions {
        mentions
            .entry(user.id.get())
            .or_insert_with(|| env.state.users.display_name(user));
    }
    let channels = channel
        .and_then(|c| c.guild_id)
        .map(|guild_id| {
            env.state.channels.by_guild.with_untracked(|by_guild| {
                by_guild
                    .get(&guild_id)
                    .into_iter()
                    .flatten()
                    .filter(|c| env.state.can(c, Permissions::VIEW_CHANNEL))
                    .filter_map(|c| Some((c.id.get(), c.name.clone()?)))
                    .collect()
            })
        })
        .unwrap_or_default();
    (Rc::new(mentions), Rc::new(channels))
}

pub fn build_card(
    env: &ResultsEnv,
    channel: Option<&Channel>,
    message: &Message,
    terms: &[String],
) -> Node {
    let body = message_body(env, channel, message, terms, CARD_CONTENT_WIDTH, false);
    let (jump, channel_id, message_id) =
        (env.search.on_jump.clone(), message.channel_id, message.id);
    let on_click = {
        let jump = jump.clone();
        move || (jump.borrow())(channel_id, message_id)
    };
    let jump_button = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::JUMP.to_string()])
        .on_click(move || (jump.borrow())(channel_id, message_id))
        .build()
        .child(
            TextProps::styled(requiem_i18n::t!("search-jump"), TextVariant::TEXT_XS_MEDIUM)
                .color(TEXT_DEFAULT)
                .line_clamp(1_u32)
                .build(),
        );
    ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::CARD.to_string()])
        .on_click(on_click)
        .build()
        .child(body)
        .child(jump_button)
}

pub fn message_body(
    env: &ResultsEnv,
    channel: Option<&Channel>,
    message: &Message,
    terms: &[String],
    content_width: f32,
    compact: bool,
) -> Node {
    let fonts = env.fonts;
    let author = env.state.users.resolve(&message.author);
    let name = match channel {
        Some(channel) => message_list::author_display_name(env.state, channel, &author),
        None => env.state.users.display_name(&author),
    };

    let mut avatar_props =
        ui::user_avatar_props(message.author.id, &author.discriminator, fonts);
    if let Some(image) = author.avatar.as_deref().and_then(|hash| {
        env.images
            .get_or_decode_image(&env.backend.avatar_url(message.author.id, hash))
    }) {
        avatar_props = avatar_props.image(image);
    }
    let avatar = avatar_props.build().class(cls::CARD_AVATAR);

    let mut header = div().class(cls::CARD_HEADER).child(
        TextProps::styled(name, TextVariant::TEXT_MD_MEDIUM)
            .line_height(22.0_f32)
            .color(TEXT_STRONG)
            .line_clamp(1_u32)
            .build()
            .class(cls::CARD_USERNAME),
    );
    if let Some(timestamp) = message.format_header_timestamp() {
        header = header.child(
            TextProps::styled(timestamp, TextVariant::TEXT_XS_MEDIUM)
                .line_height(22.0_f32)
                .color(TEXT_SUBTLE)
                .line_clamp(1_u32)
                .build(),
        );
    }

    let mut body = div().class(cls::CARD_BODY);
    if !compact {
        body = body.child(avatar).child(header);
    }
    if !message.content.is_empty()
        && !message_embed::should_hide_content(&message.content, &message.embeds)
    {
        let (mentions, channels) = mention_maps(env, message, channel);
        body = body.child(message_text(MessageTextProps {
            content: message.content.clone(),
            size: 16.0,
            color: TEXT_DEFAULT,

            mentions,
            channels,
            emoji_provider: Some(twemoji::tile),
            custom_emoji: Some(emoji_text::resolver(env.images, env.backend)),
            line_height: Some(22.0),
            highlights: terms.to_vec(),
            markdown: Some(MarkdownFeatures::CHAT),
            ..Default::default()
        }));
    }

    let accessories: Vec<Node> = message_embed::with_width_cap(content_width, || {
        let menu_env = context_menus::MenuEnv {
            backend: env.backend.clone(),
            queries: env.queries.clone(),
            http: env.http.cloned(),
            images: env.images.clone(),
            on_save: env.media_viewer.on_save.clone(),
            own_user_id: env
                .state
                .users
                .current
                .with_untracked(|u| u.as_ref().map(|u| u.id)),
            state: env.state.clone(),
        };

        let viewer_link = ViewerLink::for_message(
            env.media_viewer,
            message,
            env.backend,
            message.format_header_timestamp(),
        )
        .with_file_actions(context_menus::file_actions(
            &menu_env, message, None,
        ));
        #[cfg(feature = "video")]
        let media = message_embed::attachment_images(
            &message.attachments,
            env.images,
            env.tick,
            env.video_ctx,
            Some(&viewer_link),
        );
        #[cfg(not(feature = "video"))]
        let media = message_embed::attachment_images(
            &message.attachments,
            env.images,
            env.tick,
            None,
            Some(&viewer_link),
        );
        let files = message_embed::file_cards(
            &message.attachments,
            !media.is_empty(),
            Some(&viewer_link),
            None,
        );
        #[cfg(feature = "video")]
        let embeds = message_embed::embed_cards(
            &message.embeds,
            env.images,
            env.backend,
            env.tick,
            env.video_ctx,
            Some(&viewer_link),
            None,
            message.id.get() ^ (1 << 61),
        );
        #[cfg(not(feature = "video"))]
        let embeds = message_embed::embed_cards(
            &message.embeds,
            env.images,
            env.backend,
            env.tick,
            None,
            Some(&viewer_link),
            None,
            message.id.get() ^ (1 << 61),
        );
        let components = message_components::cards(
            message,
            env.images,
            env.backend,
            env.tick,
            Some(&viewer_link),
            None,
        );
        media
            .into_iter()
            .chain(files)
            .chain(embeds)
            .chain(components)
            .collect()
    });
    if !accessories.is_empty() {
        body = body.child(
            div()
                .class(classes_message_embed::MESSAGE_ACCESSORIES)
                .children(accessories),
        );
    }
    body
}

fn build_pager(env: &ResultsEnv, pages: u32, page: u32) -> Node {
    let go = |target: u32| {
        let search = env.search.clone();
        move || {
            search.page.set(target);
        }
    };
    let back_disabled = page <= 1;
    let next_disabled = page >= pages;

    let mut back = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::PAGER_END.to_string()])
        .on_click(go(page.saturating_sub(1).max(1)))
        .disabled(back_disabled)
        .build();
    back = back.child(
        div()
            .class(cls::PAGER_END_INNER)
            .child(icon(CHEVRON_SMALL_LEFT, TEXT_STRONG).class(cls::PAGER_CARET_LEFT))
            .child(
                TextProps::styled(
                    requiem_i18n::t!("search-previous"),
                    TextVariant::TEXT_MD_SEMIBOLD,
                )
                .line_height(18.0_f32)
                .color(TEXT_STRONG)
                .line_clamp(1_u32)
                .build(),
            ),
    );
    if back_disabled {
        back = back.class(cls::PAGER_END_DISABLED);
    }
    let mut next = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![cls::PAGER_END.to_string()])
        .on_click(go((page + 1).min(pages)))
        .disabled(next_disabled)
        .build();
    next = next.child(
        div()
            .class(cls::PAGER_END_INNER)
            .child(
                TextProps::styled(
                    requiem_i18n::t!("search-next"),
                    TextVariant::TEXT_MD_SEMIBOLD,
                )
                .line_height(18.0_f32)
                .color(TEXT_STRONG)
                .line_clamp(1_u32)
                .build(),
            )
            .child(icon(CHEVRON_SMALL_RIGHT, TEXT_STRONG).class(cls::PAGER_CARET_RIGHT)),
    );
    if next_disabled {
        next = next.class(cls::PAGER_END_DISABLED);
    }

    let mut nav = div().class(cls::PAGER).child(back);
    for item in search_state::page_list(pages, page, 9) {
        nav = nav.child(match item {
            PageItem::Page(n) => {
                let active = n == page;
                let mut button = ButtonProps::default()
                    .variant(ButtonVariant::Bare)
                    .extra_classes(vec![cls::PAGER_PAGE.to_string()])
                    .on_click(go(n))
                    .build()
                    .child(
                        TextProps::styled(n.to_string(), TextVariant::TEXT_MD_SEMIBOLD)
                            .line_height(16.0_f32)
                            .color(if active { Color::WHITE } else { TEXT_STRONG })
                            .line_clamp(1_u32)
                            .build(),
                    );
                if active {
                    button = button.class(cls::PAGER_PAGE_ACTIVE);
                }
                button
            }
            PageItem::Gap => div().class(cls::PAGER_PAGE).child(
                TextProps::styled("\u{2026}", TextVariant::TEXT_MD_SEMIBOLD)
                    .color(TEXT_STRONG)
                    .line_clamp(1_u32)
                    .build(),
            ),
        });
    }
    nav.child(next)
}
