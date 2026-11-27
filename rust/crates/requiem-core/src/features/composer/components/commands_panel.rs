use crate::ui::built_in_commands;
use crate::ui::classes::commands_panel as cls;
use crate::ui::command_composer;
use crate::ui::slash_commands;
use requiem_ui::colors;
use requiem_ui::components::AvatarProps;
use requiem_ui::components;
use requiem_ui::widget::primitives::AvatarShape;
use requiem_ui::widget;
use std::cell::Cell;
use std::cell::RefCell;

use requiem_ui::colors::{TEXT_DEFAULT, TEXT_STRONG, TEXT_SUBTLE};
use requiem_ui::components::{
    Anchor, ButtonProps, ButtonVariant, PopoverAlign, PopoverProps, PopoverSide, ScrollViewProps,
    TextProps, TextVariant, div,
};
use requiem_ui::event::NamedKey;
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};

use crate::models::application_command::{CommandApp, CommandEntry};
use crate::models::id::marker::ChannelMarker;
use crate::models::{Channel, Id};
use crate::ui::Fonts;
use crate::ui::context_menus::MenuEnv;
use requiem_ui::icons::ANGLE_BRACKETS;

const ROW_HEIGHT: f32 = 56.0;
const HEADER_HEIGHT: f32 = 32.0;
const SECTION_GAP: f32 = 16.0;

const VIEWPORT: f32 = 404.0;

#[derive(Clone)]
enum Action {
    Begin(Box<CommandEntry>, Option<CommandApp>),
}

#[derive(Clone)]
struct Row {
    action: Action,
    title: String,
    description: String,
    source: String,
    required: Vec<String>,
    optional: usize,
}

struct Section {
    name: String,
    app: Option<CommandApp>,
    rows: Vec<Row>,
}

thread_local! {

    static QUERY: Signal<Option<(Id<ChannelMarker>, String)>> = signal(None);
    static SELECTED: Signal<usize> = signal(0);
    static SCROLL: Signal<f32> = signal(0.0);

    static SCROLL_NOW: Cell<f32> = const { Cell::new(0.0) };

    static SHOWN: RefCell<Vec<Row>> = const { RefCell::new(Vec::new()) };
}

pub fn set_text(channel: Id<ChannelMarker>, text: &str) {
    let next = text
        .strip_prefix('/')
        .map(|rest| (channel, rest.to_string()));
    let changed = QUERY.with(|q| q.peek() != next);
    if changed {
        QUERY.with(|q| q.set(next));
        SELECTED.with(|s| s.set(0));
        SCROLL.with(|s| s.set(0.0));
        SCROLL_NOW.with(|s| s.set(0.0));
    }
}

pub fn dismiss() {
    if QUERY.with(|q| q.peek()).is_some() {
        QUERY.with(|q| q.set(None));
    }
}

fn sections(env: &MenuEnv, channel: &Channel, query: &str) -> Vec<Section> {
    let query = query.to_lowercase();

    let matches = |name: &str| {
        let name = name.to_lowercase();
        name.starts_with(&query)
            || name.split(' ').any(|word| word.starts_with(&query))
            || name.contains(&format!(" {query}"))
    };
    let mut sections: Vec<Section> = Vec::new();
    if let Some(index) = slash_commands::index_for(env, channel) {
        let mut apps: Vec<&CommandApp> = index.applications.iter().collect();
        apps.sort_by_key(|app| app.name.to_lowercase());
        for app in apps {
            let mut entries: Vec<CommandEntry> = index
                .entries()
                .into_iter()
                .filter(|entry| entry.command.application_id == app.id && matches(&entry.display()))
                .collect();
            entries.sort_by_key(|entry| entry.display().to_lowercase());
            let rows: Vec<Row> = entries
                .into_iter()
                .map(|entry| Row {
                    title: format!("/{}", entry.display()),
                    description: entry.description.clone(),
                    source: app.name.clone(),
                    required: entry
                        .options
                        .iter()
                        .filter(|o| o.required && !slash_commands::is_structural(o))
                        .map(|o| o.name.clone())
                        .collect(),
                    optional: entry
                        .options
                        .iter()
                        .filter(|o| !o.required && !slash_commands::is_structural(o))
                        .count(),
                    action: Action::Begin(Box::new(entry), Some(app.clone())),
                })
                .collect();
            if !rows.is_empty() {
                sections.push(Section {
                    name: app.name.clone(),
                    app: Some(app.clone()),
                    rows,
                });
            }
        }
    }
    let mut built_in_entries = built_in_commands::entries(env, channel);
    built_in_entries.sort_by_key(|entry| entry.display().to_lowercase());
    let built_in: Vec<Row> = built_in_entries
        .into_iter()
        .filter(|entry| matches(&entry.display()))
        .map(|entry| Row {
            title: format!("/{}", entry.display()),
            description: entry.description.clone(),
            source: requiem_i18n::t!("commands-panel-built-in").to_string(),
            required: entry
                .options
                .iter()
                .filter(|o| o.required)
                .map(|o| o.name.clone())
                .collect(),
            optional: entry.options.iter().filter(|o| !o.required).count(),
            action: Action::Begin(Box::new(entry), None),
        })
        .collect();
    if !built_in.is_empty() {
        sections.push(Section {
            name: requiem_i18n::t!("commands-panel-built-in").to_string(),
            app: None,
            rows: built_in,
        });
    }
    sections
}

fn choose(env: &MenuEnv, channel: &Channel, row: &Row) {
    QUERY.with(|q| q.set(None));
    match &row.action {
        Action::Begin(entry, app) => {
            env.state.drafts.clear(channel.id);
            command_composer::begin(channel.id, (**entry).clone(), app.clone());
        }
    }
}

pub fn handle_key(env: &MenuEnv, channel: &Channel, key: NamedKey, shift: bool) -> bool {
    if QUERY
        .with(|q| q.peek())
        .is_none_or(|(id, _)| id != channel.id)
    {
        return false;
    }
    let rows = SHOWN.with(|s| s.borrow().clone());
    if rows.is_empty() {
        return false;
    }
    let selected = SELECTED.with(|s| s.peek()).min(rows.len() - 1);
    let move_to = |index: usize| {
        SELECTED.with(|s| s.set(index));
        let top = row_top(&rows_by_section(&rows), index);
        let current = SCROLL_NOW.with(|s| s.get());
        let wanted = if top < current {
            Some(top)
        } else if top + ROW_HEIGHT > current + VIEWPORT {
            Some(top + ROW_HEIGHT - VIEWPORT)
        } else {
            None
        };
        if let Some(wanted) = wanted {
            SCROLL_NOW.with(|s| s.set(wanted));
            SCROLL.with(|s| s.set(wanted));
        }
    };
    match key {
        NamedKey::ArrowDown => {
            move_to((selected + 1) % rows.len());
            true
        }
        NamedKey::ArrowUp => {
            move_to((selected + rows.len() - 1) % rows.len());
            true
        }
        NamedKey::Tab | NamedKey::Enter if !shift => {
            choose(env, channel, &rows[selected]);
            true
        }
        NamedKey::Escape => {
            dismiss();
            true
        }
        _ => false,
    }
}

fn rows_by_section(rows: &[Row]) -> Vec<usize> {
    let mut sizes: Vec<usize> = Vec::new();
    let mut last: Option<&str> = None;
    for row in rows {
        if last != Some(row.source.as_str()) {
            sizes.push(0);
            last = Some(row.source.as_str());
        }
        *sizes.last_mut().expect("just pushed") += 1;
    }
    sizes
}

fn row_top(sizes: &[usize], index: usize) -> f32 {
    let mut remaining = index;
    let mut top = 0.0;
    for size in sizes {
        top += HEADER_HEIGHT;
        if remaining < *size {
            return top + remaining as f32 * ROW_HEIGHT;
        }
        top += *size as f32 * ROW_HEIGHT + SECTION_GAP;
        remaining -= *size;
    }
    top
}

fn slash_glyph(class: &'static str) -> Node {
    components::icon(ANGLE_BRACKETS, TEXT_DEFAULT).class(class)
}

pub fn panel(
    env: &MenuEnv,
    channel: &Channel,
    fonts: &Fonts,
    frame: Signal<widget::AbsoluteRect>,
) -> Node {
    let (env, channel, fonts) = (env.clone(), channel.clone(), fonts.typeface.clone());
    Node::reactive(move || {
        let Some((id, query)) = QUERY.with(Signal::get) else {
            return div();
        };
        if id != channel.id {
            return div();
        }
        let sections = sections(&env, &channel, &query);
        let flat: Vec<Row> = sections
            .iter()
            .flat_map(|s| s.rows.iter().cloned())
            .collect();
        SHOWN.with(|s| *s.borrow_mut() = flat.clone());
        if flat.is_empty() {
            return div();
        }
        let selected = SELECTED.with(Signal::get).min(flat.len() - 1);
        let scroll = SCROLL.with(Signal::get);

        let icon_of = |app: &Option<CommandApp>, class: &'static str| -> Node {
            let mut props = AvatarProps::default()
                .fallback_background(colors::BRAND)
                .shape(AvatarShape::Circle)
                .typeface(fonts.clone());
            if let Some(image) = app
                .as_ref()
                .and_then(|app| {
                    Some(
                        env.backend
                            .app_icon_url(&app.id.to_string(), app.icon.as_deref()?),
                    )
                })
                .and_then(|url| env.images.get_or_decode_image(&url))
            {
                props = props.image(image);
            }
            props.build().class(class)
        };

        let mut rail = div().class(cls::CMD_RAIL);
        let mut list = div().class(cls::CMD_LIST);
        let mut index = 0;
        let mut first_in_section = Vec::new();
        for (section_index, section) in sections.iter().enumerate() {
            let owns_selection = (index..index + section.rows.len()).contains(&selected);
            first_in_section.push(index);
            if section_index > 0 && section.app.is_none() {
                rail = rail.child(div().class(cls::CMD_RAIL_DIVIDER));
            }
            let glyph = match &section.app {
                Some(_) => icon_of(&section.app, cls::CMD_RAIL_ICON),
                None => slash_glyph(cls::CMD_RAIL_GLYPH),
            };
            let mut item = div().class(cls::CMD_RAIL_ITEM).child(glyph);
            if owns_selection {
                item = item.class(cls::CMD_RAIL_ITEM_SELECTED);
            }
            let jump = index;
            rail = rail.child(
                ButtonProps::default()
                    .variant(ButtonVariant::Bare)
                    .on_click(move || {
                        SELECTED.with(|s| s.set(jump));
                        SCROLL.with(|s| s.set(0.0));
                    })
                    .build()
                    .child(item),
            );

            let header_icon = match &section.app {
                Some(_) => icon_of(&section.app, cls::CMD_SECTION_ICON),
                None => slash_glyph(cls::CMD_SECTION_ICON),
            };
            let mut block = div().class(cls::CMD_SECTION).child(
                div().class(cls::CMD_SECTION_HEADER).children([
                    header_icon,
                    TextProps::styled(section.name.clone(), TextVariant::TEXT_SM_SEMIBOLD)
                        .line_height(18.0_f32)
                        .color(TEXT_DEFAULT)
                        .line_clamp(1_u32)
                        .build(),
                ]),
            );
            for row in &section.rows {
                let is_selected = index == selected;
                let mut usage = div().class(cls::CMD_ROW_USAGE).child(
                    TextProps::styled(row.title.clone(), TextVariant::TEXT_MD_MEDIUM)
                        .line_height(20.0_f32)
                        .color(TEXT_STRONG)
                        .build(),
                );
                if is_selected {
                    for name in &row.required {
                        usage = usage.child(
                            div().class(cls::CMD_ROW_OPTION).child(
                                TextProps::styled(name.clone(), TextVariant::TEXT_SM_NORMAL)
                                    .line_height(18.0_f32)
                                    .color(TEXT_STRONG)
                                    .build(),
                            ),
                        );
                    }
                    if row.optional > 0 {
                        usage = usage.child(
                            TextProps::styled(
                                requiem_i18n::t!("commands-panel-optional", count = row.optional)
                                    .to_string(),
                                TextVariant::TEXT_SM_NORMAL,
                            )
                            .line_height(18.0_f32)
                            .color(TEXT_SUBTLE)
                            .build()
                            .class(cls::CMD_ROW_OPTIONAL),
                        );
                    }
                }
                let info = div().class(cls::CMD_ROW_INFO).children([
                    usage,
                    TextProps::styled(row.description.clone(), TextVariant::TEXT_XS_NORMAL)
                        .line_height(16.0_f32)
                        .color(TEXT_SUBTLE)
                        .line_clamp(1_u32)
                        .build()
                        .class(cls::CMD_ROW_DESCRIPTION),
                ]);
                let source = TextProps::styled(row.source.clone(), TextVariant::TEXT_XS_NORMAL)
                    .line_height(16.0_f32)
                    .color(TEXT_SUBTLE)
                    .line_clamp(1_u32)
                    .build()
                    .class(cls::CMD_ROW_SOURCE);
                let (pick_env, pick_channel, pick_row) =
                    (env.clone(), channel.clone(), row.clone());
                let mut button = ButtonProps::default()
                    .variant(ButtonVariant::Bare)
                    .extra_classes(vec![cls::CMD_ROW.to_string()])
                    .on_click(move || choose(&pick_env, &pick_channel, &pick_row))
                    .build()
                    .children([info, source]);
                if is_selected {
                    button = button.class(cls::CMD_ROW_SELECTED);
                }
                block = block.child(button);
                index += 1;
            }
            list = list.child(block);
        }

        let box_rect = frame.get();
        let target = widget::AbsoluteRect {
            x: box_rect.x + 1.0,
            width: (box_rect.width - 10.0).max(0.0),
            ..box_rect
        };
        let anchor = Anchor::new(target)
            .position(PopoverSide::Top)
            .align(PopoverAlign::Start)
            .spacing(6.0)
            .auto_invert(false)
            .nudge(false);
        let layer = PopoverProps::default()
            .anchor(anchor)
            .build()
            .class(cls::CMD_PANEL)
            .style(format!("width: {}px", target.width));
        let scroller = ScrollViewProps::default()
            .initial_offset_y(scroll)
            .on_scroll(|offset: f32, _max: f32| SCROLL_NOW.with(|s| s.set(offset)))
            .build()
            .class(cls::CMD_SCROLLER)
            .child(list);
        let _ = &first_in_section;
        layer.child(rail).child(scroller)
    })
}
