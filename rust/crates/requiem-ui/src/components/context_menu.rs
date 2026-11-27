use crate::app;
use crate::classes::context_menu as cls;
use crate::components;
use crate::event::NamedKey as K;
use crate::event;
use crate::font_system;
use crate::icon_handle;
use crate::icons;
use crate::widget::anim::ToggleState;
use crate::widget::primitives::RadioWidget;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use skia_safe::Color;
use skia_safe::font_style::Weight;

use super::{TextProps, div, icon, icon_with_hover, text};
use crate::font_system::TextFonts;
use crate::node::Node;
use crate::signal::{Signal, signal};
use crate::widget::primitives::{
    ButtonWidget, ContextBackdropWidget, ContextPanelWidget, SubmenuHostWidget, SubmenuPanelWidget,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconPosition {
    #[default]
    Trailing,

    Leading,
}

pub enum Trailing {
    None,

    Caret,

    Check(bool),

    Radio(bool),
}

pub struct ContextAction {
    pub label: String,
    pub subtitle: Option<String>,

    pub icon: Option<icon_handle::IconSpec>,

    /// Replaces `icon` on the leading side when set.
    pub leading: Option<Node>,

    pub icon_position: IconPosition,
    /// Red label. Does not change the click.
    pub danger: bool,

    pub disabled: bool,
    pub trailing: Trailing,
    pub on_click: Rc<dyn Fn()>,

    /// Built on hover. The row opens it instead of running `on_click`, unless `parent_action`.
    pub submenu: Option<Rc<dyn Fn() -> Vec<ContextItem>>>,

    /// Click runs `on_click` without dismissing the menu.
    pub keep_open: bool,

    /// `**bold**` in `label` renders as bold.
    pub markup: bool,

    /// A submenu row still runs `on_click` when chosen, instead of only opening the submenu.
    pub parent_action: bool,
}

pub enum ContextItem {
    Action(ContextAction),
    Separator,

    Header(String),

    Custom(Node),
}

impl ContextItem {
    pub fn action(label: impl Into<String>, on_click: impl Fn() + 'static) -> Self {
        ContextItem::Action(ContextAction {
            label: label.into(),
            subtitle: None,
            icon: None,
            leading: None,
            icon_position: IconPosition::Trailing,
            danger: false,
            disabled: false,
            trailing: Trailing::None,
            on_click: Rc::new(on_click),
            submenu: None,
            keep_open: false,
            markup: false,
            parent_action: false,
        })
    }

    pub fn unavailable(label: impl Into<String>) -> Self {
        Self::action(label, || {}).disabled()
    }

    fn edit(mut self, f: impl FnOnce(&mut ContextAction)) -> Self {
        if let ContextItem::Action(action) = &mut self {
            f(action);
        }
        self
    }

    pub fn icon_position(self, position: impl Into<IconPosition>) -> Self {
        let position = position.into();
        self.edit(move |a| a.icon_position = position)
    }

    pub fn leading(self, node: Node) -> Self {
        let mut node = Some(node);
        self.edit(move |a| a.leading = node.take())
    }

    pub fn icon(self, path: impl Into<icon_handle::IconSpec>) -> Self {
        self.edit(|a| a.icon = Some(path.into()))
    }

    pub fn subtitle(self, subtitle: impl Into<String>) -> Self {
        self.edit(|a| a.subtitle = Some(subtitle.into()))
    }

    pub fn danger(self) -> Self {
        self.edit(|a| a.danger = true)
    }

    pub fn disabled(self) -> Self {
        self.edit(|a| a.disabled = true)
    }

    pub fn caret(self) -> Self {
        self.edit(|a| a.trailing = Trailing::Caret)
    }

    pub fn submenu(self, items: impl Fn() -> Vec<ContextItem> + 'static) -> Self {
        let items: Rc<dyn Fn() -> Vec<ContextItem>> = Rc::new(items);
        self.edit(move |a| {
            a.trailing = Trailing::Caret;
            a.submenu = Some(items);
        })
    }

    pub fn header(label: impl Into<String>) -> Self {
        ContextItem::Header(label.into())
    }

    pub fn checkbox(self, checked: bool) -> Self {
        self.edit(|a| a.trailing = Trailing::Check(checked))
    }

    pub fn radio(self, selected: bool) -> Self {
        self.edit(|a| a.trailing = Trailing::Radio(selected))
    }

    pub fn keep_open(self) -> Self {
        self.edit(|a| a.keep_open = true)
    }

    pub fn markup(self) -> Self {
        self.edit(|a| a.markup = true)
    }

    pub fn parent_action(self) -> Self {
        self.edit(|a| a.parent_action = true)
    }
}

struct KeyRow {
    active: Signal<bool>,
    run: Option<Rc<dyn Fn()>>,

    submenu: Option<Signal<bool>>,
}

thread_local! {

    static KEY_ROWS: RefCell<Vec<KeyRow>> = const { RefCell::new(Vec::new()) };
    static CURSOR: Cell<Option<usize>> = const { Cell::new(None) };
}

fn set_cursor(rows: &[KeyRow], to: Option<usize>) {
    if let Some(old) = CURSOR.with(Cell::get) {
        if let Some(row) = rows.get(old) {
            row.active.set(false);
            if let Some(submenu) = &row.submenu {
                submenu.set(false);
            }
        }
    }
    CURSOR.with(|c| c.set(to));
    if let Some(row) = to.and_then(|i| rows.get(i)) {
        row.active.set(true);
    }
}

pub fn handle_key(key: event::NamedKey) -> bool {
    let mut chosen: Option<Rc<dyn Fn()>> = None;
    let used = KEY_ROWS.with(|rows| {
        let rows = rows.borrow();
        if rows.is_empty() {
            return false;
        }
        let cursor = CURSOR.with(Cell::get);
        let count = rows.len();
        match key {
            K::ArrowDown | K::ArrowUp => {
                let next = match (cursor, key) {
                    (None, K::ArrowDown) => 0,
                    (None, _) => count - 1,
                    (Some(i), K::ArrowDown) => (i + 1) % count,
                    (Some(i), _) => (i + count - 1) % count,
                };
                set_cursor(&rows, Some(next));
            }
            K::Home => set_cursor(&rows, Some(0)),
            K::End => set_cursor(&rows, Some(count - 1)),
            K::ArrowRight => {
                if let Some(submenu) = cursor
                    .and_then(|i| rows.get(i))
                    .and_then(|r| r.submenu.clone())
                {
                    submenu.set(true);
                }
            }
            K::ArrowLeft => {
                if let Some(submenu) = cursor
                    .and_then(|i| rows.get(i))
                    .and_then(|r| r.submenu.clone())
                {
                    submenu.set(false);
                }
            }
            K::Enter | K::Space => {
                if let Some(row) = cursor.and_then(|i| rows.get(i)) {
                    if let Some(submenu) = &row.submenu {
                        submenu.set(true);
                    } else {
                        chosen = row.run.clone();
                    }
                }
            }
            _ => return false,
        }
        true
    });
    if let Some(run) = chosen {
        run();
    }
    used
}

fn item_node(
    action: ContextAction,
    fonts: &TextFonts,
    on_dismiss: &Rc<dyn Fn()>,
    track: bool,
) -> Node {
    let mut label = TextProps {
        content: action.label,
        size: 14.0,
        weight: Weight::MEDIUM.into(),
        color: Color::from_rgb(0xfb, 0xfb, 0xfb),
        fonts: Some(fonts.clone()),
        ..Default::default()
    };
    if action.markup {
        label = label.bold_markup();
    }
    let label = text(label).class(cls::LABEL);
    let column = match action.subtitle {
        Some(subtitle) => {
            let subtitle = text(TextProps {
                content: subtitle,
                size: 12.0,
                color: Color::from_rgb(0xab, 0xac, 0xb2),
                fonts: Some(fonts.clone()),
                ..Default::default()
            })
            .class(cls::SUBTITLE);
            div().class(cls::TEXT_COLUMN).child(label).child(subtitle)
        }
        None => div().class(cls::TEXT_COLUMN).child(label),
    };

    let submenu = action.submenu.clone();
    let mut widget = ButtonWidget::new().with_focusable(false);
    let mut run_on_choose: Option<Rc<dyn Fn()>> = None;

    if !action.disabled && (submenu.is_none() || action.parent_action) {
        let (run, dismiss, keep_open) = (
            action.on_click.clone(),
            on_dismiss.clone(),
            action.keep_open,
        );
        let choose: Rc<dyn Fn()> = Rc::new(move || {
            if !keep_open {
                dismiss();
            }
            run();
        });
        run_on_choose = Some(choose.clone());
        widget = widget.with_on_click(move || choose());
    }
    let mut row = Node::new("button", widget).class(cls::ITEM);
    let key_active = (track && !action.disabled).then(|| signal(false));
    if let Some(active) = &key_active {
        row = row.bind_class(active.clone(), cls::ITEM_ACTIVE, "");
    }
    let muted_default = Color::from_rgb(0xab, 0xac, 0xb2);
    let strong_hover = Color::from_rgb(0xfb, 0xfb, 0xfb);
    let danger_color = Color::from_rgb(0xf8, 0x7e, 0x7a);
    let mut leading = action.leading;
    if action.icon_position == IconPosition::Leading && leading.is_none() {
        if let Some(path) = &action.icon {
            let glyph = if action.danger {
                icon(path, danger_color)
            } else {
                icon_with_hover(path, muted_default, strong_hover)
            };
            leading = Some(glyph.class(cls::ICON_GLYPH));
        }
    }
    let icon_is_leading = action.icon_position == IconPosition::Leading;
    if let Some(leading) = leading {
        row = row.child(div().class(cls::LEADING).child(leading));
    }
    row = row.child(column);
    if action.danger {
        row = row.class(cls::ITEM_DANGER);
    }
    if action.disabled {
        row = row.class(cls::ITEM_DISABLED);
    }

    let muted = Color::from_rgb(0x96, 0x97, 0x9e);
    let default = Color::from_rgb(0xab, 0xac, 0xb2);
    let strong = Color::from_rgb(0xfb, 0xfb, 0xfb);
    let danger = Color::from_rgb(0xf8, 0x7e, 0x7a);
    match action.trailing {
        Trailing::Caret => {
            row = row.child(
                div()
                    .class(cls::ICON)
                    .child(icon(icons::CHEVRON_SMALL_RIGHT, muted).class(cls::ICON_GLYPH)),
            )
        }
        Trailing::Check(checked) => {
            let mut boxed = div().class(cls::CHECKBOX);
            if checked {
                boxed = boxed.class(cls::CHECKBOX_CHECKED).child(
                    icon(icons::CHECKMARK_SMALL_BOLD, Color::WHITE).class(cls::CHECK_GLYPH),
                );
            }
            row = row.child(div().class(cls::ICON).child(boxed));
        }

        Trailing::Radio(selected) => {
            let ring = Node::new(
                "div",
                RadioWidget::new(ToggleState::new(
                    selected,
                )),
            )
            .class(cls::RADIO);
            row = row.child(div().class(cls::ICON).child(ring));
        }
        Trailing::None => {
            if let Some(path) = action.icon.as_ref().filter(|_| !icon_is_leading) {
                let glyph = if action.danger {
                    icon(path, danger)
                } else {
                    icon_with_hover(path, default, strong)
                };
                row = row.child(div().class(cls::ICON).child(glyph.class(cls::ICON_GLYPH)));
            }
        }
    }
    match submenu {
        Some(build) => {
            let (node, open) = with_submenu(row, build, fonts, on_dismiss);
            if let Some(active) = key_active {
                KEY_ROWS.with(|rows| {
                    rows.borrow_mut().push(KeyRow {
                        active,
                        run: None,
                        submenu: Some(open),
                    })
                });
            }
            node
        }
        None => {
            if let Some(active) = key_active {
                KEY_ROWS.with(|rows| {
                    rows.borrow_mut().push(KeyRow {
                        active,
                        run: run_on_choose,
                        submenu: None,
                    })
                });
            }
            row
        }
    }
}

fn with_submenu(
    row: Node,
    build: Rc<dyn Fn() -> Vec<ContextItem>>,
    fonts: &TextFonts,
    on_dismiss: &Rc<dyn Fn()>,
) -> (Node, Signal<bool>) {
    let host_rect = Rc::new(Cell::new(None));
    let open = signal(false);
    let host = {
        let open = open.clone();
        SubmenuHostWidget::new(host_rect.clone(), move |inside| {
            if open.peek() != inside {
                open.set(inside);
            }
        })
    };
    let (fonts, on_dismiss) = (fonts.clone(), on_dismiss.clone());
    let panel_open = open.clone();
    let panel = Node::reactive(move || {
        if !panel_open.get() {
            return div();
        }
        let rows = items_nodes(build(), &fonts, &on_dismiss, false);
        let panel = div().class(cls::SUBMENU).child(
            components::scroll_view()
                .class(cls::SUBMENU_SCROLL)
                .children(rows),
        );
        Node::new("div", SubmenuPanelWidget::new(host_rect.clone()))
            .class(cls::SUBMENU_BRIDGE)
            .child(panel)
    })
    .class(cls::SUBMENU_SLOT);
    (
        Node::new("div", host)
            .class(cls::ITEM_HOST)
            .child(row)
            .child(panel),
        open,
    )
}

fn items_nodes(
    items: Vec<ContextItem>,
    fonts: &TextFonts,
    on_dismiss: &Rc<dyn Fn()>,
    track: bool,
) -> Vec<Node> {
    items
        .into_iter()
        .map(|item| match item {
            ContextItem::Separator => div().class(cls::SEPARATOR),
            ContextItem::Header(label) => div().class(cls::HEADER).child(text(TextProps {
                content: label,
                size: 14.0,
                line_height: Some(20.0),
                weight: Weight::MEDIUM.into(),
                color: Color::from_rgb(0x96, 0x97, 0x9e),
                fonts: Some(fonts.clone()),
                ..Default::default()
            })),
            ContextItem::Custom(node) => node,
            ContextItem::Action(action) => item_node(action, fonts, on_dismiss, track),
        })
        .collect()
}

pub fn context_menu_panel(
    anchor: (f32, f32),
    items: Vec<ContextItem>,
    on_dismiss: &Rc<dyn Fn()>,
) -> Node {
    context_menu_panel_at(anchor, false, items, on_dismiss)
}

pub fn context_menu_panel_at(
    anchor: (f32, f32),
    above: bool,
    items: Vec<ContextItem>,
    on_dismiss: &Rc<dyn Fn()>,
) -> Node {
    let fonts = &font_system::resolve(None);

    let max_height = (app::current_window_size().1 - 2.0 * 8.0 - 2.0).max(120.0);

    KEY_ROWS.with(|rows| rows.borrow_mut().clear());
    CURSOR.with(|c| c.set(None));
    let scroller = components::scroll_view()
        .class(cls::SCROLLER)
        .style(format!("max-height: {max_height}px"))
        .children(items_nodes(items, fonts, on_dismiss, true));
    Node::new("div", ContextPanelWidget::new(anchor).above(above))
        .class(cls::PANEL)
        .child(scroller)
}

pub fn context_menu_layer(
    anchor: (f32, f32),
    items: Vec<ContextItem>,
    on_dismiss: Rc<dyn Fn()>,
) -> Node {
    context_menu_layer_at(anchor, false, items, on_dismiss)
}

pub fn context_menu_layer_at(
    anchor: (f32, f32),
    above: bool,
    items: Vec<ContextItem>,
    on_dismiss: Rc<dyn Fn()>,
) -> Node {
    let backdrop = {
        let dismiss = on_dismiss.clone();
        Node::new("div", ContextBackdropWidget::new(move || dismiss())).class(cls::BACKDROP)
    };
    div()
        .class(cls::LAYER)
        .child(backdrop)
        .child(context_menu_panel_at(anchor, above, items, &on_dismiss))
}
