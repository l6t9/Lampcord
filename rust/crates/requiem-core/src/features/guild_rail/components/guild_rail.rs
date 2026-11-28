use crate::state::RtcState;
use crate::ui::classes::guild_rail;
use crate::ui::classes::guild_rail as cls;
use crate::ui::context_menu;
use crate::ui::context_menus;
use crate::ui::create_guild::Step;
use crate::ui::create_guild;
use crate::ui::navigation;
use crate::utils::guild;
use requiem_ui::colors::BRAND;
use requiem_ui::components::TextVariant;
use requiem_ui::components::{ButtonVariant, TooltipPosition};
use requiem_ui::paint as requiem_ui_paint;
use requiem_ui::widget::primitives::ImageWidget;
use std::cell::{Cell, RefCell};
use std::mem;
use std::rc::Rc;
use std::time::Duration;

use requiem_ui::components::{
    AvatarProps, ButtonProps, NumberBadgeProps, TextProps, TooltipProps, avatar, div, icon,
    number_badge, scroll_view, tooltip,
};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, create_effect};
use requiem_ui::timer;
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::anim::FloatTransition;
use requiem_ui::widget::primitives::AvatarShape;
use requiem_ui::widget::{AbsoluteRect, Widget, WidgetId, WidgetTree};
use skia_safe::{Canvas, Color, Image, Paint, Point, RRect, Rect};

use crate::config::BackendConfig;
use crate::images::ImageCache;
use crate::models::id::marker::{ChannelMarker, GuildMarker};
use crate::models::{Guild, Id};
use crate::state::{AppState, GuildFolder, RailEntry};
use crate::ui::{Fonts, TEXT_DEFAULT};
use requiem_ui::icons::{CIRCLE_PLUS, CLYDE, COMPASS, FOLDER, VOICE_NORMAL};

fn live_icon(
    images: &ImageCache,
    tick: &Signal<u64>,
    url: Option<String>,
    slot: &'static str,
    wrap_ready: bool,
    build: Rc<dyn Fn(Option<Image>) -> Node>,
) -> Node {
    let resolve: Rc<dyn Fn() -> Option<Image>> = {
        let (images, url) = (images.clone(), url.clone());
        Rc::new(move || url.as_deref().and_then(|url| images.get_or_decode_image(url)))
    };
    let image = resolve();
    if image.is_some() || url.is_none() {
        let node = build(image);
        return if wrap_ready {
            div().class(slot).child(node)
        } else {
            node
        };
    }
    let loaded = Signal::new(false);
    let slot_node = {
        let (loaded, resolve) = (loaded.clone(), resolve.clone());
        Node::reactive(move || {
            let _ = loaded.get();
            build(resolve())
        })
        .class(slot)
    };
    let tick = tick.clone();
    slot_node.on_mount(move |tree, id| {
        let (resolve, tick, loaded) = (resolve.clone(), tick.clone(), loaded.clone());
        let queued = Cell::new(false);
        let effect = create_effect(
            move |_| {
                if loaded.peek() || queued.get() {
                    return;
                }
                let _ = tick.get();
                if resolve().is_some() {
                    queued.set(true);
                    mark_ready(loaded.clone());
                }
            },
            tree,
        );
        tree.attach_effect(id, effect);
    })
}

const READY_BATCH: Duration = Duration::from_millis(150);

thread_local! {
    static READY_ICONS: RefCell<Vec<Signal<bool>>> = const { RefCell::new(Vec::new()) };
    static READY_FLUSH_PENDING: Cell<bool> = const { Cell::new(false) };
}

fn mark_ready(loaded: Signal<bool>) {
    READY_ICONS.with(|ready| ready.borrow_mut().push(loaded));
    if !READY_FLUSH_PENDING.with(|pending| pending.replace(true)) {
        timer::set_timeout(READY_BATCH, || {
            READY_FLUSH_PENDING.with(|pending| pending.set(false));
            for loaded in READY_ICONS.with(|ready| mem::take(&mut *ready.borrow_mut())) {
                loaded.set(true);
            }
        });
    }
}

pub fn build(
    tree: &mut WidgetTree,
    parent: WidgetId,
    state: &AppState,
    selected_guild_id: Option<Id<GuildMarker>>,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &BackendConfig,
    on_select_channel: &Rc<dyn Fn(Id<ChannelMarker>)>,
    on_open_home: &Rc<dyn Fn()>,
    tick: &Signal<u64>,
) {
    let rail = scroll_view().class(guild_rail::GUILD_RAIL);
    let rail_id = tree.mount(parent, rail);

    let home_icon = icon(CLYDE, TEXT_DEFAULT).class(guild_rail::HOME_ICON);
    let home_selected = selected_guild_id.is_none();
    let mut home_button = div()
        .class(guild_rail::HOME_BUTTON)
        .child(home_icon);
    if home_selected {
        home_button = home_button.class(guild_rail::HOME_BUTTON_SELECTED);
    }
    let on_open_home = on_open_home.clone();

    let home_pings: Rc<dyn Fn() -> u32> = {
        let (dm_mentions, relationships) =
            (state.read_states.dm_mentions(), state.relationships.clone());
        Rc::new(move || {
            dm_mentions.get()
                + relationships
                    .list
                    .with(|list| list.iter().filter(|r| r.is_incoming()).count() as u32)
        })
    };
    tree.mount(
        rail_id,
        rail_item(
            home_button,
            Some(home_selected),
            Some(state.read_states.dm_unread()),
            Some(home_pings),
            Some(Rc::new(move || on_open_home())),
            &requiem_i18n::t!("guild-rail-direct-messages"),
        ),
    );

    tree.mount(
        rail_id,
        div().class(guild_rail::GUILD_SEPARATOR),
    );

    let guilds = state.guilds.list.get();
    for entry in state.folders.arrange(&guilds) {
        let node = match entry {
            RailEntry::Guild(guild) => guild_item(
                state,
                images,
                backend,
                fonts,
                &guild,
                selected_guild_id,
                on_select_channel,
                tick,
            ),
            RailEntry::Folder { folder, guilds } => folder_group(
                state,
                images,
                backend,
                fonts,
                &folder,
                &guilds,
                selected_guild_id,
                on_select_channel,
                tick,
            ),
        };
        tree.mount(rail_id, node);
    }

    for (path, label, opens_create) in [
        (
            CIRCLE_PLUS,
            requiem_i18n::t!("guild-rail-add-a-server"),
            true,
        ),
        (COMPASS, requiem_i18n::t!("guild-rail-discover"), false),
    ] {
        let button = div()
            .class(guild_rail::ROUND_BUTTON)
            .child(
                icon(path, TEXT_DEFAULT).class(guild_rail::ROUND_BUTTON_ICON),
            );

        let on_click: Option<Rc<dyn Fn()>> = opens_create.then(|| {
            Rc::new(|| create_guild::open(Step::Templates))
                as Rc<dyn Fn()>
        });
        tree.mount(
            rail_id,
            rail_item(button, None, None, None, on_click, &label),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn guild_item(
    state: &AppState,
    images: &ImageCache,
    backend: &BackendConfig,
    fonts: &Fonts,
    guild: &Guild,
    selected_guild_id: Option<Id<GuildMarker>>,
    on_select_channel: &Rc<dyn Fn(Id<ChannelMarker>)>,
    tick: &Signal<u64>,
) -> Node {
    let url = guild
        .icon
        .as_deref()
        .map(|hash| backend.guild_icon_url(guild.id, hash));
    let selected = selected_guild_id == Some(guild.id);
    let (initial, typeface) = (guild::acronym(&guild.name), fonts.typeface.clone());
    let icon_node = live_icon(
        images,
        tick,
        url,
        guild_rail::GUILD_ICON,
        false,
        Rc::new(move |image| match image {
            Some(image) => avatar(AvatarProps {
                image: Some(image),
                fallback_initial: initial.clone(),
                fallback_background: BRAND,
                typeface: Some(typeface.clone()),
                shape: AvatarShape::Squircle,
                status_cutout: None,
                fallback_icon: None,
                image_binding: None,
                decoration: None,
            })
            .class(guild_rail::GUILD_ICON),
            None => {
                let mut tile = div().class(guild_rail::GUILD_FALLBACK).child(
                    TextProps::styled(initial.clone(), TextVariant::TEXT_MD_MEDIUM)
                        .color(TEXT_DEFAULT)
                        .build()
                        .class(guild_rail::GUILD_FALLBACK_TEXT),
                );
                if selected {
                    tile = tile.class(guild_rail::GUILD_FALLBACK_SELECTED);
                }
                tile
            }
        }),
    );
    let on_click = open_guild(state, guild.id, on_select_channel);
    let guild_pings: Rc<dyn Fn() -> u32> = {
        let badge = state.read_states.guild_badge(guild.id);
        Rc::new(move || badge.get().mentions)
    };
    let guild_id = guild.id;
    let item = rail_item(
        icon_node,
        Some(selected),
        Some(state.read_states.guild_unread(guild.id)),
        Some(guild_pings),
        Some(on_click),
        &guild.name,
    )
    .child(voice_badge(state, guild.id));
    item.on_context_menu({
        let state = state.clone();
        move |x, y| {
            let state = state.clone();
            context_menu::open(x, y, move || {
                context_menus::guild_items(&state, guild_id)
            });
        }
    })
}

fn open_guild(
    state: &AppState,
    guild_id: Id<GuildMarker>,
    on_select_channel: &Rc<dyn Fn(Id<ChannelMarker>)>,
) -> Rc<dyn Fn()> {
    let (on_select, state) = (on_select_channel.clone(), state.clone());
    Rc::new(move || {
        if let Some(channel) = navigation::channel_for_guild(&state, guild_id) {
            on_select(channel);
        }
    })
}

/// Folder color at 40% on the closed tile and 15% behind an open folder, as the web client
/// mixes it. A folder without a color keeps the stylesheet's subtle fill.
const CLOSED_TINT_ALPHA: u8 = 0x66;
const OPEN_TINT_ALPHA: u8 = 0x26;
const FOLDER_MINI_ICONS: usize = 4;

fn folder_color(folder: &GuildFolder) -> Option<Color> {
    folder
        .color
        .filter(|c| *c != 0)
        .map(|c| Color::from_rgb((c >> 16) as u8, (c >> 8) as u8, c as u8))
}

fn css_rgba(color: Color, alpha: u8) -> String {
    format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        color.r(),
        color.g(),
        color.b(),
        alpha
    )
}

/// A folder: closed it's one tile previewing its first servers, open it's a header over the
/// servers themselves, all on a tint of the folder's color. Both states are the same nodes, so
/// the web client's animation plays: header and preview slide past each other and the tint
/// fades over 150ms (stylesheet transitions) while the list grows (driven by the store).
#[allow(clippy::too_many_arguments)]
fn folder_group(
    state: &AppState,
    images: &ImageCache,
    backend: &BackendConfig,
    fonts: &Fonts,
    folder: &GuildFolder,
    guilds: &[Guild],
    selected_guild_id: Option<Id<GuildMarker>>,
    on_select_channel: &Rc<dyn Fn(Id<ChannelMarker>)>,
    tick: &Signal<u64>,
) -> Node {
    let Some(id) = folder.id else {
        return div();
    };
    let color = folder_color(folder);
    let tint = |alpha: u8| color.map(|c| format!("background-color: {}", css_rgba(c, alpha)));
    let label = folder.name.clone().unwrap_or_else(|| {
        guilds
            .iter()
            .map(|g| g.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    });
    let contains_selected = selected_guild_id.is_some_and(|s| guilds.iter().any(|g| g.id == s));
    let toggle: Rc<dyn Fn()> = {
        let folders = state.folders.clone();
        Rc::new(move || folders.toggle(id))
    };
    let ids: Vec<Id<GuildMarker>> = guilds.iter().map(|g| g.id).collect();
    let (open, list_style) = state.folders.motion(id, guilds.len());

    let mini: Vec<Node> = guilds
        .iter()
        .take(FOLDER_MINI_ICONS)
        .map(|guild| {
            let url = guild
                .icon
                .as_deref()
                .map(|hash| backend.guild_icon_url(guild.id, hash));
            let initial = guild::acronym(&guild.name);
            live_icon(
                images,
                tick,
                url,
                cls::FOLDER_MINI,
                true,
                Rc::new(move |image| match image {
                    Some(image) => Node::new("div", ImageWidget::new(Some(image)))
                        .style("width: 100%; height: 100%"),
                    None => div()
                        .class(cls::FOLDER_MINI_EMPTY)
                        .style("width: 100%; height: 100%")
                        .child(
                            TextProps::new(initial.clone(), 10.0_f32)
                                .line_height(20.0_f32)
                                .color(TEXT_DEFAULT)
                                .center(true)
                                .build()
                                .class(cls::FOLDER_MINI_TEXT),
                        ),
                }),
            )
        })
        .collect();
    let mut preview = div().class(cls::FOLDER_TILE).children(mini);
    if let Some(style) = tint(CLOSED_TINT_ALPHA) {
        preview = preview.style(&style);
    }
    let header = div().class(cls::FOLDER_HEADER).child(
        div()
            .class(cls::FOLDER_HEADER_BOX)
            .child(icon(FOLDER, color.unwrap_or(BRAND)).class(cls::FOLDER_HEADER_ICON)),
    );
    // The folder icon sits above the preview and the column rides up to show the preview
    // while closed.
    let column = div()
        .class(cls::FOLDER_COLUMN)
        .bind_class(open.clone(), cls::FOLDER_COLUMN_OPEN, "")
        .child(header)
        .child(preview);
    let viewport = div().class(cls::FOLDER_VIEWPORT).child(column);

    let mut background =
        div()
            .class(cls::FOLDER_TINT)
            .bind_class(open.clone(), cls::FOLDER_TINT_OPEN, "");
    if let Some(style) = tint(OPEN_TINT_ALPHA) {
        background = background.style(&style);
    }
    // The list is re-rendered as the store's tween moves its height. A section is what the
    // rest of the app drives from timers, and it keeps the list node itself out of the
    // folder's stable nodes, which the stylesheet transitions need.
    let list = {
        let (state, images, backend, fonts) = (
            state.clone(),
            images.clone(),
            backend.clone(),
            fonts.clone(),
        );
        let (guilds, on_select, tick) = (guilds.to_vec(), on_select_channel.clone(), tick.clone());
        Node::reactive(move || {
            div()
                .class(cls::FOLDER_LIST)
                .style(list_style.get())
                .children(
                    guilds
                        .iter()
                        .map(|guild| {
                            guild_item(
                                &state,
                                &images,
                                &backend,
                                &fonts,
                                guild,
                                selected_guild_id,
                                &on_select,
                                &tick,
                            )
                        })
                        .collect::<Vec<_>>(),
                )
        })
        .class(cls::FOLDER_LIST_HOST)
    };
    div()
        .class(cls::FOLDER_GROUP)
        .child(background)
        .child(folder_item(
            state,
            viewport,
            &ids,
            contains_selected,
            toggle,
            &label,
            open,
        ))
        .child(list)
}

/// The folder's own rail row. While closed it carries the pill and badges of everything
/// inside, so the pill takes the selected, unread and hover states for the whole folder; once
/// open, each server has its own and this one goes away.
fn folder_item(
    state: &AppState,
    content: Node,
    ids: &[Id<GuildMarker>],
    selected: bool,
    on_click: Rc<dyn Fn()>,
    label: &str,
    open: Signal<bool>,
) -> Node {
    let pill = {
        let (state, ids, open) = (state.clone(), ids.to_vec(), open.clone());
        Node::reactive(move || {
            let mut classes = vec![cls::RAIL_PILL.to_string()];
            if open.get() {
                classes.push(cls::RAIL_PILL_OFF.to_string());
            } else if selected {
                classes.push(cls::RAIL_PILL_SELECTED.to_string());
            } else if ids
                .iter()
                .any(|id| state.read_states.guild_unread(*id).get())
            {
                classes.push(cls::RAIL_PILL_UNREAD.to_string());
            }
            Node::new("div", RailPillWidget::new()).classes(classes)
        })
        .class(cls::RAIL_PILL_ANCHOR)
    };
    let pings: Rc<dyn Fn() -> u32> = {
        let (state, ids) = (state.clone(), ids.to_vec());
        Rc::new(move || {
            ids.iter()
                .map(|id| state.read_states.guild_badge(*id).get().mentions)
                .sum()
        })
    };
    let target = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .on_click(move || on_click())
        .build()
        .child(content);
    let badge = {
        let open = open.clone();
        Node::reactive(move || {
            let count = pings();
            if count == 0 || open.get() {
                return div();
            }
            div()
                .class(cls::RAIL_BADGE_RING)
                .child(number_badge(NumberBadgeProps::new(count)))
        })
        .class(cls::RAIL_BADGE_ANCHOR)
    };
    let (menu_state, menu_ids) = (state.clone(), ids.to_vec());
    div()
        .class(cls::RAIL_ITEM)
        .class(cls::FOLDER_ROW)
        .on_context_menu(move |x, y| {
            let (state, ids) = (menu_state.clone(), menu_ids.clone());
            context_menu::open(x, y, move || {
                context_menus::folder_items(&state, &ids)
            });
        })
        .children(vec![
            pill,
            tooltip(
                target,
                TooltipProps::default()
                    .text(label.to_string())
                    .position(TooltipPosition::Right)
                    .large(true)
                    .max_lines(4_u32),
            ),
            badge,
        ])
}

fn voice_badge(state: &AppState, guild_id: Id<GuildMarker>) -> Node {
    let state = state.clone();
    Node::reactive(move || {
        if !state.voice.guild_has_voice(guild_id) {
            return div();
        }
        let connected = state.voice.rtc.with(|rtc| {
            rtc.guild_id == Some(guild_id) && rtc.state != RtcState::Disconnected
        });
        let mut badge = div()
            .class(cls::RAIL_VOICE_BADGE)
            .child(icon(VOICE_NORMAL, Color::WHITE).class(cls::RAIL_VOICE_ICON));
        if connected {
            badge = badge.class(cls::RAIL_VOICE_BADGE_CONNECTED);
        }
        div().class(cls::RAIL_VOICE_RING).child(badge)
    })
    .class(cls::RAIL_BADGE_ANCHOR)
}

const RAIL_PILL_TRANSITION: Duration = Duration::from_millis(200);

fn rail_item(
    content: Node,
    pill: Option<bool>,
    unread: Option<Signal<bool>>,
    pings: Option<Rc<dyn Fn() -> u32>>,
    on_click: Option<Rc<dyn Fn()>>,
    label: &str,
) -> Node {
    let mut children = Vec::with_capacity(3);
    if let Some(selected) = pill {
        let mut pill_classes = vec![guild_rail::RAIL_PILL.to_string()];
        if selected {
            pill_classes.push(guild_rail::RAIL_PILL_SELECTED.to_string());
        }
        let mut pill_node = Node::new("div", RailPillWidget::new()).classes(pill_classes);
        if let Some(unread) = unread {
            pill_node =
                pill_node.bind_class(unread, guild_rail::RAIL_PILL_UNREAD, "");
        }
        children.push(pill_node);
    }
    let target = match on_click {
        Some(on_click) => ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .on_click(move || on_click())
            .build()
            .child(content),
        None => content,
    };

    children.push(tooltip(
        target,
        TooltipProps::default()
            .text(label.to_string())
            .position(TooltipPosition::Right)
            .large(true),
    ));
    if let Some(pings) = pings {
        children.push(
            Node::reactive(move || {
                let count = pings();
                if count == 0 {
                    return div();
                }
                div()
                    .class(guild_rail::RAIL_BADGE_RING)
                    .child(number_badge(NumberBadgeProps::new(count)))
            })
            .class(guild_rail::RAIL_BADGE_ANCHOR),
        );
    }
    div()
        .class(guild_rail::RAIL_ITEM)
        .children(children)
}

pub struct RailPillWidget {
    height: FloatTransition,

    alpha: FloatTransition,
    shift: FloatTransition,

    last_color: Cell<Color>,
}

impl Default for RailPillWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl RailPillWidget {
    pub fn new() -> Self {
        Self {
            height: FloatTransition::new(RAIL_PILL_TRANSITION),
            alpha: FloatTransition::new(RAIL_PILL_TRANSITION),
            shift: FloatTransition::new(RAIL_PILL_TRANSITION),
            last_color: Cell::new(Color::WHITE),
        }
    }
}

impl Widget for RailPillWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let Some(color) = style.background else {
            return;
        };
        if color.a() > 0 {
            self.last_color.set(color);
        }

        let wanted_alpha = f32::from(color.a());
        let wanted_shift = if color.a() > 0 { 0.0 } else { -4.0 };
        let alpha = self.alpha.animate(wanted_alpha);
        let shift = self.shift.animate(wanted_shift);
        let height = self.height.animate(rect.height);

        let top = rect.y + (rect.height - height) / 2.0;
        if (alpha - wanted_alpha).abs() > 0.5
            || (shift - wanted_shift).abs() > 0.001
            || (height - rect.height).abs() > 0.001
        {
            requiem_ui_paint::animate_region(
                canvas,
                Rect::from_xywh(rect.x - 4.0, rect.y - 26.0, rect.width + 8.0, 52.0),
            );
        }
        if alpha < 0.5 {
            return;
        }
        let color = self
            .last_color
            .get()
            .with_a(alpha.round().clamp(0.0, 255.0) as u8);
        let bounds = Rect::from_xywh(rect.x + shift, top, rect.width, height);

        let tl = style.corner_radius_top_left.unwrap_or(style.corner_radius);
        let tr = style.corner_radius_top_right.unwrap_or(style.corner_radius);
        let br = style
            .corner_radius_bottom_right
            .unwrap_or(style.corner_radius);
        let bl = style
            .corner_radius_bottom_left
            .unwrap_or(style.corner_radius);
        let radii = [
            Point::new(tl, tl),
            Point::new(tr, tr),
            Point::new(br, br),
            Point::new(bl, bl),
        ];

        let mut paint = Paint::default();
        paint.set_color(color);
        paint.set_anti_alias(true);
        canvas.draw_rrect(RRect::new_rect_radii(bounds, &radii), &paint);
    }
}

#[cfg(test)]
mod tests {
    use crate::gateway::events::GatewayEvent;
    use crate::ui::classes::guild_rail as cls;
    use crate::ui;
    use requiem_ui::font_system::TextFonts;
    use requiem_ui::font_system;
    use requiem_ui::layout;
    use requiem_ui::paint as requiem_ui_paint;
    use requiem_ui::signal;
    use requiem_ui::style::cascade;
    use requiem_ui::style::parser;
    use requiem_ui::theme as requiem_ui_theme;
    use requiem_ui::timer;
    use skia_safe::font_style::Weight;
    use std::mem;
    use std::thread;
    use std::time::Instant;
    use tokio::runtime::Runtime;

    use requiem_ui::style::ComputedStyle;
    use requiem_ui::widget::primitives::BoxWidget;
    use taffy::{AvailableSpace, Size};

    use super::*;
    use crate::state::wire::LegacyFolderWire;
    use crate::state::{Action, AppState};

    fn state_with_folder(folder_ids: &[u64]) -> AppState {
        let state = AppState::new();
        let guild = |id: u64, name: &str| {
            serde_json::json!({
                "id": id.to_string(),
                "properties": { "name": name, "icon": null, "owner_id": "1" },
                "channels": [],
                "emojis": [],
            })
        };
        let user = serde_json::json!({
            "id": "1", "username": "me", "discriminator": "0", "global_name": null, "avatar": null
        });
        state.apply_event(&GatewayEvent::new(
            "READY",
            serde_json::json!({ "user": user, "guilds": [
                guild(10, "Alpha"), guild(20, "Beta"), guild(30, "Gamma")
            ] }),
        ));
        state.dispatch(&Action::GuildFoldersUpdate(vec![LegacyFolderWire {
            id: Some(7),
            name: Some("Folder".into()),
            color: None,
            guild_ids: folder_ids.iter().map(|id| Id::new(*id)).collect(),
        }]));
        state
    }

    struct Laid {
        class: String,
        rect: [f32; 4],
        translate_y: f32,
    }

    const THEME_SPACE: Size<AvailableSpace> = Size {
        width: AvailableSpace::Definite(72.0),
        height: AvailableSpace::Definite(900.0),
    };

    fn theme() -> Vec<parser::StyleRule> {
        parser::parse_stylesheet(&format!(
            "{}\n{}",
            requiem_ui::BUNDLED_CSS,
            ui::THEME_CSS
        ))
    }

    /// Restyle (inline bindings included) and lay out again.
    fn relayout(tree: &mut WidgetTree, theme: &[parser::StyleRule]) {
        cascade::apply_stylesheet(tree, theme);
        layout::compute_layout(tree, THEME_SPACE);
    }

    fn rail_tree(state: &AppState, selected: Option<u64>) -> WidgetTree {
        let runtime = Runtime::new().unwrap();
        let images = ImageCache::new(runtime.handle().clone(), wreq::Client::new(), || {});
        let fonts = Fonts {
            typeface: requiem_ui_theme::make_typeface(Weight::MEDIUM),
            text_fonts: TextFonts::new(1024 * 1024),
        };
        font_system::install_ambient(fonts.text_fonts.clone());
        let backend = BackendConfig {
            rest_base_url: String::new(),
            gateway_url: String::new(),
            cdn_base_url: String::new(),
        };
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let root = tree.root;
        tree.nodes[root].classes.push("root".to_string());
        let on_select: Rc<dyn Fn(Id<ChannelMarker>)> = Rc::new(|_| {});
        let on_home: Rc<dyn Fn()> = Rc::new(|| {});
        build(
            &mut tree,
            root,
            state,
            selected.map(Id::new),
            &fonts,
            &images,
            &backend,
            &on_select,
            &on_home,
            &Signal::new(0),
        );
        relayout(&mut tree, &theme());
        // The runtime has to outlive the images it decodes for.
        mem::forget(runtime);
        tree
    }

    /// Every class in the rail with its rect relative to the rail and its translateY.
    fn rail(state: &AppState, selected: Option<u64>) -> Vec<Laid> {
        let tree = rail_tree(state, selected);
        laid_out(&tree)
    }

    fn laid_out(tree: &WidgetTree) -> Vec<Laid> {
        let rects = requiem_ui_paint::all_absolute_rects(tree);
        let rail = tree
            .nodes
            .iter()
            .find(|(_, n)| {
                n.classes
                    .iter()
                    .any(|c| c == guild_rail::GUILD_RAIL)
            })
            .map(|(id, _)| rects[&id])
            .expect("the rail is mounted");
        let mut out = Vec::new();
        let mut stack = vec![tree.root];
        while let Some(id) = stack.pop() {
            let r = rects[&id];
            let translate_y = tree.nodes[id]
                .style
                .transform
                .as_ref()
                .map_or(0.0, |t| t.translate_y);
            for class in &tree.nodes[id].classes {
                out.push(Laid {
                    class: class.clone(),
                    rect: [r.x - rail.x, r.y - rail.y, r.width, r.height],
                    translate_y,
                });
            }
            stack.extend(tree.nodes[id].children.iter().rev());
        }
        out
    }

    fn of<'a>(laid: &'a [Laid], class: &str) -> Vec<&'a Laid> {
        laid.iter().filter(|l| l.class == class).collect()
    }

    fn rects(laid: &[Laid], class: &str) -> Vec<[f32; 4]> {
        of(laid, class).iter().map(|l| l.rect).collect()
    }

    // Reference values from the web client: 40px icons on a 48px pitch, a 48px closed
    // folder tile at x=12, and an open folder whose list starts 4px under its header.
    #[test]
    fn a_closed_folder_is_a_48px_tile_between_40px_rows() {
        let laid = rail(&state_with_folder(&[10, 20]), None);
        assert_eq!(rects(&laid, cls::HOME_BUTTON), [[16.0, 0.0, 40.0, 40.0]]);
        assert_eq!(
            rects(&laid, cls::FOLDER_VIEWPORT),
            [[12.0, 57.0, 48.0, 48.0]]
        );

        // The folder icon sits above the preview and the column rides up to show it.
        let column = of(&laid, cls::FOLDER_COLUMN);
        assert_eq!(column[0].translate_y, -48.0);
        assert_eq!(rects(&laid, cls::FOLDER_HEADER), [[12.0, 57.0, 48.0, 48.0]]);
        assert_eq!(rects(&laid, cls::FOLDER_TILE), [[12.0, 105.0, 48.0, 48.0]]);

        let mini = rects(&laid, cls::FOLDER_MINI);
        assert_eq!(mini.len(), 2, "only the folder's two servers are previewed");
        assert_eq!(mini[0], [16.0, 109.0, 19.0, 19.0]);
        assert_eq!(mini[1], [37.0, 109.0, 19.0, 19.0]);

        let rows = rects(&laid, cls::RAIL_ITEM);
        let lone = rows
            .iter()
            .find(|r| r[3] > 0.0 && r[1] > 100.0 && r[1] < 150.0);
        assert_eq!(
            lone.map(|r| [r[1], r[3]]),
            Some([113.0, 40.0]),
            "the next row sits 8px under the 48px folder row, and the closed list takes no room"
        );
    }

    #[test]
    fn an_open_folder_lists_its_servers_four_pixels_under_the_header() {
        let state = state_with_folder(&[10, 20]);
        state.folders.toggle(7);
        let laid = rail(&state, None);
        assert_eq!(of(&laid, cls::FOLDER_COLUMN)[0].translate_y, 0.0);
        assert_eq!(rects(&laid, cls::FOLDER_GROUP), [[0.0, 57.0, 72.0, 144.0]]);
        assert_eq!(rects(&laid, cls::FOLDER_TINT), [[12.0, 57.0, 48.0, 144.0]]);
        assert_eq!(rects(&laid, cls::FOLDER_HEADER), [[12.0, 57.0, 48.0, 48.0]]);
        assert_eq!(
            rects(&laid, cls::FOLDER_HEADER_BOX),
            [[16.0, 61.0, 40.0, 40.0]]
        );
        assert_eq!(rects(&laid, cls::FOLDER_LIST), [[0.0, 105.0, 72.0, 96.0]]);
        let items: Vec<[f32; 2]> = rects(&laid, cls::RAIL_ITEM)
            .iter()
            .filter(|r| r[1] > 105.0 && r[1] < 201.0)
            .map(|r| [r[1], r[3]])
            .collect();
        assert_eq!(items, [[109.0, 40.0], [157.0, 40.0]]);
    }

    // A closed folder holding the open server wears the selected pill, full height like a
    // server's, and an open one hands the pill to the servers inside.
    #[test]
    fn the_closed_folder_takes_the_selected_pill_for_the_server_inside() {
        let state = state_with_folder(&[10, 20]);
        let closed = rail(&state, Some(20));
        // The server inside has a pill too, but its hidden list takes no room.
        let pills: Vec<[f32; 4]> = of(&closed, cls::RAIL_PILL_SELECTED)
            .iter()
            .map(|l| l.rect)
            .filter(|r| r[3] > 0.0)
            .collect();
        assert_eq!(
            pills,
            [[0.0, 61.0, 4.0, 40.0]],
            "one selected pill, on the folder row (57..105), 40px tall"
        );

        state.folders.toggle(7);
        let (_, list) = state.folders.motion(7, 2);
        let started = Instant::now();
        while list.get() != "height: 96px" && started.elapsed() < Duration::from_millis(500) {
            timer::run_due();
            thread::sleep(Duration::from_millis(4));
        }
        let open = rail(&state, Some(20));
        assert_eq!(
            of(&open, cls::RAIL_PILL_OFF).len(),
            1,
            "the folder's pill is off"
        );
        let selected: Vec<[f32; 4]> = of(&open, cls::RAIL_PILL_SELECTED)
            .iter()
            .map(|l| l.rect)
            .collect();
        assert_eq!(selected.len(), 1, "only the selected server's own pill");
        assert_eq!(selected[0][3], 40.0);
        assert!(
            selected[0][1] > 105.0,
            "it's down in the list, not on the folder"
        );
    }

    // The store drives the list's height through a bound style; once the timers have run
    // and the effects flushed, the laid out list has to have grown with it.
    #[test]
    fn opening_a_folder_grows_the_laid_out_list() {
        let state = state_with_folder(&[10, 20]);
        let mut tree = rail_tree(&state, None);
        let theme = theme();
        let height = |tree: &WidgetTree| {
            rects(&laid_out(tree), cls::FOLDER_LIST)
                .first()
                .map(|r| r[3])
                .unwrap()
        };
        assert_eq!(height(&tree), 0.0, "closed folders take no room");

        state.folders.toggle(7);
        let started = Instant::now();
        let mut seen = Vec::new();
        while started.elapsed() < Duration::from_millis(400) {
            timer::run_due();
            signal::flush(&mut tree);
            relayout(&mut tree, &theme);
            seen.push(height(&tree));
            if height(&tree) >= 96.0 {
                break;
            }
            thread::sleep(Duration::from_millis(4));
        }
        assert_eq!(seen.last(), Some(&96.0), "{seen:?}");
        assert!(seen.iter().any(|h| *h > 0.0 && *h < 96.0), "{seen:?}");
        let group = rects(&laid_out(&tree), cls::FOLDER_GROUP);
        assert_eq!(group[0][3], 144.0, "the whole group grew with its list");
    }

    #[test]
    fn the_closed_folder_shows_unread_for_any_server_inside() {
        let state = state_with_folder(&[10, 20]);
        state.read_states.guild_unread(Id::new(20)).set(true);
        let laid = rail(&state, None);
        let unread: Vec<[f32; 4]> = of(&laid, cls::RAIL_PILL_UNREAD)
            .iter()
            .map(|l| l.rect)
            .collect();
        assert!(
            unread.contains(&[0.0, 77.0, 4.0, 8.0]),
            "the folder row's 8px unread pill is centered on its 48px row: {unread:?}"
        );
    }

    #[test]
    fn the_rails_scrollable_height_ends_at_its_last_row() {
        let tree = rail_tree(&state_with_folder(&[10, 20]), None);
        let rail_id = tree
            .nodes
            .iter()
            .find(|(_, n)| n.classes.iter().any(|c| c == cls::GUILD_RAIL))
            .map(|(id, _)| id)
            .expect("the rail is mounted");
        let (_, extent) = requiem_ui_paint::content_extent(&tree, rail_id);
        let rects = requiem_ui_paint::all_absolute_rects(&tree);
        let rail = rects[&rail_id];
        let mut bottoms: Vec<(f32, String)> = tree
            .nodes
            .iter()
            .map(|(id, n)| {
                (
                    rects[&id].y + rects[&id].height - rail.y,
                    format!("{}.{}", n.tag, n.classes.join(".")),
                )
            })
            .collect();
        bottoms.sort_by(|a, b| b.0.total_cmp(&a.0));
        eprintln!("rail extent {extent}; lowest nodes: {:?}", &bottoms[..bottoms.len().min(6)]);
        let last_row = bottoms
            .iter()
            .find(|(_, name)| name.contains(cls::RAIL_ITEM))
            .map_or(0.0, |(bottom, _)| *bottom);
        assert!(extent <= last_row + 100.0, "the list is {extent} tall but its last row ends at {last_row}");
    }
}
