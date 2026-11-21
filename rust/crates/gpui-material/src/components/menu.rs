//! Material 3 Expressive Menus component.
//!
//! Implements Material 3 Expressive menus adhering to official AndroidX
//! specifications and visual reference designs. Supports grouped cards,
//! unified cards with dividers, two-line layout with supporting text,
//! keyboard shortcuts, submenu chevrons, badges, spring-animated open/close
//! transitions, and spring state layers on items.

use gpui::{IntoElement, ParentElement, Rgba, SharedString, Styled, Window, div, prelude::*, px};

use crate::components::icon::M3Icon;
use crate::motion::{SpringParams, SpringState, spring_step};
use crate::theme::M3Palette;

/// Trailing content type for a menu item.
#[derive(Clone, Debug, PartialEq)]
pub enum MenuItemTrailing {
    /// No trailing accessory.
    None,
    /// Trailing vector icon (e.g. `Refresh`, `Settings`, `Help`, `MoreVert`).
    Icon(M3Icon),
    /// Keyboard shortcut string (e.g. `"⌘C"`).
    Shortcut(&'static str),
    /// Submenu navigation pointer chevron (`▸`).
    Chevron,
    /// Tag badge pill (e.g. `"New"`) with optional trailing arrow.
    Badge {
        /// Tag text.
        text: &'static str,
        /// Whether a small trailing chevron arrow is shown next to the badge.
        has_arrow: bool,
    },
}

/// A single interactive item inside a Material 3 Expressive Menu.
#[derive(Clone, Debug)]
pub struct MenuItem {
    /// Unique item identifier.
    pub id: usize,
    /// Primary label text.
    pub label: SharedString,
    /// Optional supporting text displayed below the label (two-line layout).
    pub supporting_text: Option<SharedString>,
    /// Optional leading vector icon.
    pub leading_icon: Option<M3Icon>,
    /// Optional trailing content.
    pub trailing: MenuItemTrailing,
    /// Whether the item is interactive.
    pub enabled: bool,
    /// Whether the item is actively selected (highlighted with pill background).
    pub selected: bool,
    /// Whether pointer is hovering over the item.
    pub is_hovered: bool,
    /// Whether pointer is pressing down on the item.
    pub is_pressed: bool,
    /// Animated spring state layer for hover and press overlay.
    pub state_layer: SpringState,
}

impl MenuItem {
    /// Creates a new enabled menu item with the given id and label.
    #[must_use]
    pub fn new(id: usize, label: impl Into<SharedString>) -> Self {
        Self {
            id,
            label: label.into(),
            supporting_text: None,
            leading_icon: None,
            trailing: MenuItemTrailing::None,
            enabled: true,
            selected: false,
            is_hovered: false,
            is_pressed: false,
            state_layer: SpringState::default(),
        }
    }

    /// Sets supporting text for a two-line menu item.
    #[must_use]
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    /// Sets a leading M3 vector icon.
    #[must_use]
    pub fn leading_icon(mut self, icon: M3Icon) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    /// Sets trailing content.
    #[must_use]
    pub fn trailing(mut self, trailing: MenuItemTrailing) -> Self {
        self.trailing = trailing;
        self
    }

    /// Sets a trailing vector icon (e.g. `Refresh`, `Settings`, `Help`, `MoreVert`).
    #[must_use]
    pub fn trailing_icon(mut self, icon: M3Icon) -> Self {
        self.trailing = MenuItemTrailing::Icon(icon);
        self
    }

    /// Sets a keyboard shortcut trailing label (e.g. `"⌘C"`).
    #[must_use]
    pub fn shortcut(mut self, shortcut: &'static str) -> Self {
        self.trailing = MenuItemTrailing::Shortcut(shortcut);
        self
    }

    /// Sets a trailing tag badge (e.g. `"New"`, with optional chevron arrow).
    #[must_use]
    pub fn badge(mut self, text: &'static str, has_arrow: bool) -> Self {
        self.trailing = MenuItemTrailing::Badge { text, has_arrow };
        self
    }

    /// Sets a trailing submenu chevron (`▸`).
    #[must_use]
    pub fn chevron(mut self) -> Self {
        self.trailing = MenuItemTrailing::Chevron;
        self
    }

    /// Sets selected state (rendered with rounded selection pill background).
    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Sets enabled state.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Target alpha for the state layer based on hover and press states.
    #[must_use]
    pub fn target_state_layer_alpha(&self) -> f32 {
        if !self.enabled {
            0.0
        } else if self.is_pressed {
            0.12
        } else if self.is_hovered {
            0.08
        } else {
            0.0
        }
    }

    /// Step spring state layer animation by `dt` seconds. Returns true if unsettled.
    pub fn step(&mut self, dt: f32) -> bool {
        let target = self.target_state_layer_alpha();
        let spring = SpringParams::expressive();
        self.state_layer = spring_step(spring, self.state_layer, target, dt);
        !self.state_layer.is_settled(target, 0.002, 0.005)
    }
}

/// A logical group of menu items, optionally with a section header.
#[derive(Clone, Debug)]
pub struct MenuGroup {
    /// Optional header label at the top of the group (callout 9).
    pub header: Option<SharedString>,
    /// Items belonging to this group.
    pub items: Vec<MenuItem>,
}

impl MenuGroup {
    /// Creates a new empty menu group.
    #[must_use]
    pub fn new() -> Self {
        Self {
            header: None,
            items: Vec::new(),
        }
    }

    /// Sets a section header label for this group.
    #[must_use]
    pub fn header(mut self, header: impl Into<SharedString>) -> Self {
        self.header = Some(header.into());
        self
    }

    /// Adds a single item to the group.
    #[must_use]
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Sets all items in this group.
    #[must_use]
    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }
}

impl Default for MenuGroup {
    fn default() -> Self {
        Self::new()
    }
}

/// Container display style for Material 3 Expressive menus.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuVariant {
    /// Separate stacked rounded cards with a vertical gap (callout 10).
    #[default]
    GroupedCards,
    /// Single unified rounded card with horizontal dividers between groups (callout 11).
    UnifiedCard,
}

#[inline]
fn srgb(r: u8, g: u8, b: u8) -> Rgba {
    Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}

/// Layout metrics matching Material 3 Expressive menu tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuMetrics {
    /// Outer container corner radius (24dp in M3 Expressive, callout 10/11).
    pub container_corner: f32,
    /// Inner container corner radius between grouped cards (8dp in M3 Expressive).
    pub inner_corner: f32,
    /// Selection highlight pill corner radius (12dp).
    pub item_corner: f32,
    /// Minimum height for single-line menu items (44dp).
    pub item_height_single: f32,
    /// Minimum height for two-line menu items with supporting text (60dp).
    pub item_height_double: f32,
    /// Container internal vertical padding.
    pub container_padding_y: f32,
    /// Container internal horizontal padding.
    pub container_padding_x: f32,
    /// Item horizontal padding.
    pub item_padding_x: f32,
    /// Item vertical padding.
    pub item_padding_y: f32,
    /// Gap between grouped cards in `GroupedCards` variant (callout 10).
    pub group_gap: f32,
    /// Icon size for leading and trailing icons (20dp).
    pub icon_size: f32,
    /// Space between leading icon and label.
    pub icon_label_space: f32,
    /// Typography font size for item label (14dp).
    pub label_font_size: f32,
    /// Typography font size for supporting text (12dp).
    pub supporting_font_size: f32,
    /// Typography font size for shortcut text (12dp).
    pub shortcut_font_size: f32,
    /// Typography font size for section header (12dp).
    pub header_font_size: f32,
    /// Minimum menu width.
    pub min_width: f32,
    /// Maximum menu width.
    pub max_width: f32,
}

impl Default for MenuMetrics {
    fn default() -> Self {
        Self {
            container_corner: 24.0,
            inner_corner: 6.0,
            item_corner: 12.0,
            item_height_single: 44.0,
            item_height_double: 60.0,
            container_padding_y: 4.0,
            container_padding_x: 6.0,
            item_padding_x: 12.0,
            item_padding_y: 6.0,
            group_gap: 4.0,
            icon_size: 20.0,
            icon_label_space: 12.0,
            label_font_size: 14.0,
            supporting_font_size: 12.0,
            shortcut_font_size: 12.0,
            header_font_size: 12.0,
            min_width: 200.0,
            max_width: 280.0,
        }
    }
}

/// Color tokens for Material 3 Expressive menus.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuColors {
    /// Container background surface color.
    pub container: Rgba,
    /// Selected item pill background container color (`secondary_container`).
    pub selected_container: Rgba,
    /// Label text color.
    pub text: Rgba,
    /// Selected label text color.
    pub selected_text: Rgba,
    /// Supporting text color.
    pub supporting_text: Rgba,
    /// Leading icon color.
    pub leading_icon: Rgba,
    /// Selected leading icon color.
    pub selected_leading_icon: Rgba,
    /// Trailing text/icon color.
    pub trailing_content: Rgba,
    /// Selected trailing text/icon color.
    pub selected_trailing_content: Rgba,
    /// Section header text color.
    pub header_text: Rgba,
    /// Divider line color.
    pub divider: Rgba,
    /// Hover and press state layer tint color.
    pub state_layer: Rgba,
    /// Badge background container color.
    pub badge_container: Rgba,
    /// Badge text color.
    pub badge_text: Rgba,
}

impl MenuColors {
    /// Resolves colors for the current theme palette.
    #[must_use]
    pub fn for_palette(p: &M3Palette) -> Self {
        let is_dark = p.surface.r < 0.2;
        Self {
            container: if is_dark {
                srgb(30, 26, 36)
            } else {
                p.surface_container_highest
            },
            selected_container: if is_dark {
                srgb(78, 55, 68)
            } else {
                p.secondary_container
            },
            text: p.on_surface,
            selected_text: if is_dark {
                srgb(245, 227, 231)
            } else {
                p.on_secondary_container
            },
            supporting_text: p.on_surface_variant,
            leading_icon: p.on_surface_variant,
            selected_leading_icon: if is_dark {
                srgb(245, 227, 231)
            } else {
                p.on_secondary_container
            },
            trailing_content: p.on_surface_variant,
            selected_trailing_content: if is_dark {
                srgb(245, 227, 231)
            } else {
                p.on_secondary_container
            },
            header_text: p.on_surface_variant,
            divider: p.outline_variant,
            state_layer: p.on_surface,
            badge_container: p.primary_container,
            badge_text: p.on_primary,
        }
    }
}

/// Interactive model for a Material 3 Expressive Menu.
#[derive(Clone, Debug)]
pub struct MenuModel {
    /// Logical groups of menu items.
    pub groups: Vec<MenuGroup>,
    /// Container presentation style (GroupedCards or UnifiedCard).
    pub variant: MenuVariant,
    /// Whether the menu is currently visible/open.
    pub is_open: bool,
    /// ID of the currently selected menu item, if any.
    pub selected_id: Option<usize>,
    /// Spring animation state for open/close scaling and opacity.
    pub open_progress: SpringState,
    /// Optional custom layout metrics.
    pub custom_metrics: Option<MenuMetrics>,
    /// Optional custom color tokens.
    pub custom_colors: Option<MenuColors>,
    /// Optional custom spring physics.
    pub custom_spring: Option<SpringParams>,
}

impl MenuModel {
    /// Creates a new menu model with the given groups.
    #[must_use]
    pub fn new(groups: impl IntoIterator<Item = MenuGroup>) -> Self {
        let groups_vec: Vec<MenuGroup> = groups.into_iter().collect();
        let initial_selected = groups_vec
            .iter()
            .flat_map(|g| &g.items)
            .find(|it| it.selected)
            .map(|it| it.id);

        Self {
            groups: groups_vec,
            variant: MenuVariant::GroupedCards,
            is_open: true,
            selected_id: initial_selected,
            open_progress: SpringState { pos: 1.0, vel: 0.0 },
            custom_metrics: None,
            custom_colors: None,
            custom_spring: None,
        }
    }

    /// Selects an item by its unique identifier, deselecting all other items.
    pub fn select(&mut self, id: usize) {
        self.selected_id = Some(id);
        for group in &mut self.groups {
            for item in &mut group.items {
                item.selected = item.id == id;
            }
        }
    }

    /// Sets or clears the selected item ID.
    pub fn select_optional(&mut self, id: Option<usize>) {
        self.selected_id = id;
        for group in &mut self.groups {
            for item in &mut group.items {
                item.selected = Some(item.id) == id;
            }
        }
    }

    /// Returns whether the item with `id` is currently selected.
    #[must_use]
    pub fn is_selected(&self, id: usize) -> bool {
        self.selected_id == Some(id)
    }

    /// Returns a reference to the currently selected item, if any.
    #[must_use]
    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.selected_id.and_then(|id| {
            for group in &self.groups {
                for item in &group.items {
                    if item.id == id {
                        return Some(item);
                    }
                }
            }
            None
        })
    }

    /// Sets the menu variant (GroupedCards or UnifiedCard).
    #[must_use]
    pub fn variant(mut self, variant: MenuVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets open or closed state.
    #[must_use]
    pub fn is_open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        if is_open {
            self.open_progress.pos = 1.0;
        } else {
            self.open_progress.pos = 0.0;
        }
        self
    }

    /// Sets custom layout metrics.
    #[must_use]
    pub fn metrics(mut self, metrics: MenuMetrics) -> Self {
        self.custom_metrics = Some(metrics);
        self
    }

    /// Sets custom color tokens.
    #[must_use]
    pub fn colors(mut self, colors: MenuColors) -> Self {
        self.custom_colors = Some(colors);
        self
    }

    /// Sets custom spring physics.
    #[must_use]
    pub fn spring(mut self, spring: SpringParams) -> Self {
        self.custom_spring = Some(spring);
        self
    }

    /// Toggles menu open/closed state.
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    /// Advances spring physics by `dt` seconds for menu transition and all item state layers.
    /// Returns `true` if any spring is still animating and needs another frame.
    pub fn step(&mut self, dt: f32) -> bool {
        let target = if self.is_open { 1.0 } else { 0.0 };
        let spring = self.custom_spring.unwrap_or_else(SpringParams::expressive);
        self.open_progress = spring_step(spring, self.open_progress, target, dt);
        let menu_animating = !self.open_progress.is_settled(target, 0.002, 0.005);

        let mut items_animating = false;
        for group in &mut self.groups {
            for item in &mut group.items {
                if item.step(dt) {
                    items_animating = true;
                }
            }
        }

        menu_animating || items_animating
    }

    /// Advance spring animations, requesting an animation frame if unsettled.
    pub fn advance_springs(&mut self, window: &mut Window, dt: f32) {
        if self.step(dt) {
            window.request_animation_frame();
        }
    }

    /// Computes `(top_left, top_right, bottom_right, bottom_left)` corner radii for a group card.
    #[must_use]
    pub fn group_card_radii(
        ix: usize,
        total: usize,
        metrics: &MenuMetrics,
    ) -> (f32, f32, f32, f32) {
        if total <= 1 {
            (
                metrics.container_corner,
                metrics.container_corner,
                metrics.container_corner,
                metrics.container_corner,
            )
        } else if ix == 0 {
            (
                metrics.container_corner,
                metrics.container_corner,
                metrics.inner_corner,
                metrics.inner_corner,
            )
        } else if ix == total - 1 {
            (
                metrics.inner_corner,
                metrics.inner_corner,
                metrics.container_corner,
                metrics.container_corner,
            )
        } else {
            (
                metrics.inner_corner,
                metrics.inner_corner,
                metrics.inner_corner,
                metrics.inner_corner,
            )
        }
    }

    /// Renders the menu as a GPUI element hierarchy according to variant, colors, and metrics.
    pub fn render(&self, pal: &M3Palette) -> impl IntoElement {
        let metrics = self.custom_metrics.unwrap_or_default();
        let colors = self
            .custom_colors
            .unwrap_or_else(|| MenuColors::for_palette(pal));

        let progress = self.open_progress.pos.clamp(0.0, 1.0);
        let opacity = progress;

        let root = div()
            .flex()
            .flex_col()
            .min_w(px(metrics.min_width))
            .max_w(px(metrics.max_width))
            .opacity(opacity)
            .when(!self.is_open && progress <= 0.001, |d| {
                d.invisible().size_0().overflow_hidden()
            });

        match self.variant {
            MenuVariant::GroupedCards => {
                // Stacked separate cards with group_gap (callout 10)
                let mut container = root.gap(px(metrics.group_gap));
                let total = self.groups.len();
                for (ix, group) in self.groups.iter().enumerate() {
                    let card = self.render_group_card(group, ix, total, &metrics, &colors, pal);
                    container = container.child(card);
                }
                container
            }
            MenuVariant::UnifiedCard => {
                // Single unified card with dividers between groups (callout 11)
                let mut card = div()
                    .flex()
                    .flex_col()
                    .bg(colors.container)
                    .rounded(px(metrics.container_corner))
                    .p(px(metrics.container_padding_y))
                    .shadow_md();

                for (ix, group) in self.groups.iter().enumerate() {
                    if ix > 0 {
                        card =
                            card.child(div().my(px(4.0)).mx(px(6.0)).h(px(1.0)).bg(colors.divider));
                    }
                    card = card.child(self.render_group_contents(group, &metrics, &colors, pal));
                }
                root.child(card)
            }
        }
    }

    /// Renders the menu with an interactive click callback attached to every item.
    pub fn render_with_handlers<V: 'static>(
        &self,
        pal: &M3Palette,
        cx: &Context<V>,
        on_click: impl Fn(&mut V, usize, &mut Context<V>) + 'static + Copy,
    ) -> impl IntoElement {
        let metrics = self.custom_metrics.unwrap_or_default();
        let colors = self
            .custom_colors
            .unwrap_or_else(|| MenuColors::for_palette(pal));

        let progress = self.open_progress.pos.clamp(0.0, 1.0);
        let opacity = progress;

        let root = div()
            .flex()
            .flex_col()
            .min_w(px(metrics.min_width))
            .max_w(px(metrics.max_width))
            .opacity(opacity)
            .when(!self.is_open && progress <= 0.001, |d| {
                d.invisible().size_0().overflow_hidden()
            });

        match self.variant {
            MenuVariant::GroupedCards => {
                let mut container = root.gap(px(metrics.group_gap));
                let total = self.groups.len();
                for (ix, group) in self.groups.iter().enumerate() {
                    let (tl, tr, br, bl) = Self::group_card_radii(ix, total, &metrics);
                    let card = div()
                        .flex()
                        .flex_col()
                        .bg(colors.container)
                        .rounded_tl(px(tl))
                        .rounded_tr(px(tr))
                        .rounded_br(px(br))
                        .rounded_bl(px(bl))
                        .p(px(metrics.container_padding_y))
                        .shadow_md()
                        .child(
                            self.render_group_with_handlers(group, &metrics, &colors, cx, on_click),
                        );
                    container = container.child(card);
                }
                container
            }
            MenuVariant::UnifiedCard => {
                let mut card = div()
                    .flex()
                    .flex_col()
                    .bg(colors.container)
                    .rounded(px(metrics.container_corner))
                    .p(px(metrics.container_padding_y))
                    .shadow_md();

                for (ix, group) in self.groups.iter().enumerate() {
                    if ix > 0 {
                        card =
                            card.child(div().my(px(4.0)).mx(px(6.0)).h(px(1.0)).bg(colors.divider));
                    }
                    card = card.child(
                        self.render_group_with_handlers(group, &metrics, &colors, cx, on_click),
                    );
                }
                root.child(card)
            }
        }
    }

    fn render_group_with_handlers<V: 'static>(
        &self,
        group: &MenuGroup,
        metrics: &MenuMetrics,
        colors: &MenuColors,
        cx: &Context<V>,
        on_click: impl Fn(&mut V, usize, &mut Context<V>) + 'static + Copy,
    ) -> impl IntoElement {
        let mut col = div().flex().flex_col().gap_0p5();

        if let Some(ref header) = group.header {
            col = col.child(
                div()
                    .px(px(metrics.item_padding_x))
                    .py_1()
                    .text_size(px(metrics.header_font_size))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(colors.header_text)
                    .child(header.clone()),
            );
        }

        for item in &group.items {
            let item_id = item.id;
            let item_element = self.render_item(item, metrics, colors);
            let interactive_item = div()
                .id(("m3-menu-item-wrap", item_id))
                .cursor_pointer()
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        on_click(this, item_id, cx);
                    }),
                )
                .child(item_element);
            col = col.child(interactive_item);
        }

        col
    }

    /// Renders a single group inside an isolated card (used for `GroupedCards` variant).
    fn render_group_card(
        &self,
        group: &MenuGroup,
        ix: usize,
        total: usize,
        metrics: &MenuMetrics,
        colors: &MenuColors,
        pal: &M3Palette,
    ) -> impl IntoElement {
        let (tl, tr, br, bl) = Self::group_card_radii(ix, total, metrics);
        div()
            .flex()
            .flex_col()
            .bg(colors.container)
            .rounded_tl(px(tl))
            .rounded_tr(px(tr))
            .rounded_br(px(br))
            .rounded_bl(px(bl))
            .p(px(metrics.container_padding_y))
            .shadow_md()
            .child(self.render_group_contents(group, metrics, colors, pal))
    }

    /// Renders the header and item list for a group.
    fn render_group_contents(
        &self,
        group: &MenuGroup,
        metrics: &MenuMetrics,
        colors: &MenuColors,
        _pal: &M3Palette,
    ) -> impl IntoElement {
        let mut col = div().flex().flex_col().gap_0p5();

        // Optional section header (callout 9)
        if let Some(ref header) = group.header {
            col = col.child(
                div()
                    .px(px(metrics.item_padding_x))
                    .py_1()
                    .text_size(px(metrics.header_font_size))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(colors.header_text)
                    .child(header.clone()),
            );
        }

        // Render each item
        for item in &group.items {
            col = col.child(self.render_item(item, metrics, colors));
        }

        col
    }

    /// Renders a single menu item with leading icon, label, supporting text, trailing content, and state layer.
    pub fn render_item(
        &self,
        item: &MenuItem,
        metrics: &MenuMetrics,
        colors: &MenuColors,
    ) -> impl IntoElement {
        let is_two_line = item.supporting_text.is_some();
        let min_h = if is_two_line {
            metrics.item_height_double
        } else {
            metrics.item_height_single
        };

        let (text_color, icon_color, trailing_color) = if item.selected {
            (
                colors.selected_text,
                colors.selected_leading_icon,
                colors.selected_trailing_content,
            )
        } else {
            (colors.text, colors.leading_icon, colors.trailing_content)
        };

        let bg_color = if item.selected {
            colors.selected_container
        } else {
            Rgba {
                a: 0.0,
                ..colors.container
            }
        };

        let mut row = div()
            .id(("m3-menu-item", item.id))
            .relative()
            .flex()
            .items_center()
            .justify_between()
            .min_h(px(min_h))
            .px(px(metrics.item_padding_x))
            .py(px(metrics.item_padding_y))
            .rounded(px(metrics.item_corner))
            .bg(bg_color)
            .cursor_pointer()
            .when(!item.selected, |d| {
                d.hover(|h| {
                    h.bg(Rgba {
                        a: 0.12,
                        ..colors.state_layer
                    })
                })
            })
            .active(|a| {
                a.bg(Rgba {
                    a: 0.22,
                    ..colors.state_layer
                })
            });

        // State layer overlay (spring-animated hover / press)
        let layer_alpha = item.state_layer.pos.clamp(0.0, 1.0);
        if layer_alpha > 0.001 {
            row = row.child(
                div()
                    .id(("m3-menu-item-layer", item.id))
                    .absolute()
                    .inset_0()
                    .rounded(px(metrics.item_corner))
                    .bg(colors.state_layer)
                    .opacity(layer_alpha),
            );
        }

        // Left side: leading icon + label + optional supporting text
        let mut left = div()
            .relative()
            .flex()
            .items_center()
            .gap(px(metrics.icon_label_space));

        if let Some(icon) = item.leading_icon {
            left = left.child(icon.element(metrics.icon_size, icon_color));
        }

        if let Some(ref sup) = item.supporting_text {
            // Two-line layout (callout 8)
            left = left.child(
                div()
                    .flex_col()
                    .child(
                        div()
                            .text_size(px(metrics.label_font_size))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(text_color)
                            .child(item.label.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(metrics.supporting_font_size))
                            .text_color(colors.supporting_text)
                            .child(sup.clone()),
                    ),
            );
        } else {
            left = left.child(
                div()
                    .text_size(px(metrics.label_font_size))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(text_color)
                    .child(item.label.clone()),
            );
        }

        // Right side: trailing content
        let right = match &item.trailing {
            MenuItemTrailing::None => div(),
            MenuItemTrailing::Icon(icon) => {
                div().child(icon.element(metrics.icon_size, trailing_color))
            }
            MenuItemTrailing::Shortcut(shortcut) => div()
                .text_size(px(metrics.shortcut_font_size))
                .font_weight(gpui::FontWeight::NORMAL)
                .text_color(trailing_color)
                .child(*shortcut),
            MenuItemTrailing::Chevron => {
                div().child(M3Icon::ChevronRight.element(16.0, trailing_color))
            }
            MenuItemTrailing::Badge { text, has_arrow } => {
                let mut badge_box = div().flex().items_center().gap_1().child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_full()
                        .bg(colors.badge_container)
                        .text_size(px(10.0))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(colors.badge_text)
                        .child(*text),
                );

                if *has_arrow {
                    badge_box = badge_box.child(M3Icon::ChevronRight.element(14.0, trailing_color));
                }
                badge_box
            }
        };

        row.child(left).child(right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_item_creation_and_defaults() {
        let item = MenuItem::new(1, "Copy")
            .shortcut("⌘C")
            .leading_icon(M3Icon::ContentCopy);

        assert_eq!(item.id, 1);
        assert_eq!(item.label.as_ref(), "Copy");
        assert_eq!(item.trailing, MenuItemTrailing::Shortcut("⌘C"));
        assert_eq!(item.leading_icon, Some(M3Icon::ContentCopy));
        assert!(item.enabled);
        assert!(!item.selected);
    }

    #[test]
    fn menu_group_and_variant() {
        let group = MenuGroup::new()
            .header("Actions")
            .item(MenuItem::new(1, "Edit").selected(true))
            .item(MenuItem::new(2, "Delete"));

        assert_eq!(group.items.len(), 2);
        assert!(group.items[0].selected);
        assert_eq!(group.header.as_deref(), Some("Actions"));

        let mut menu = MenuModel::new(vec![group]).variant(MenuVariant::UnifiedCard);
        assert_eq!(menu.variant, MenuVariant::UnifiedCard);
        assert!(menu.is_open);

        menu.toggle();
        assert!(!menu.is_open);
    }

    #[test]
    fn menu_spring_step() {
        let mut item = MenuItem::new(1, "Test");
        item.is_hovered = true;
        let running = item.step(1.0 / 60.0);
        assert!(running);
        assert!(item.state_layer.pos > 0.0);
    }

    #[test]
    fn menu_metrics_connected_corners() {
        let metrics = MenuMetrics::default();
        assert_eq!(metrics.container_corner, 24.0);
        assert_eq!(metrics.inner_corner, 6.0);
        assert_eq!(metrics.group_gap, 4.0);

        // 2 groups: top card has 24 top, 6 bottom
        let (tl, tr, br, bl) = MenuModel::group_card_radii(0, 2, &metrics);
        assert_eq!((tl, tr, br, bl), (24.0, 24.0, 6.0, 6.0));

        // 2 groups: bottom card has 6 top, 24 bottom
        let (tl, tr, br, bl) = MenuModel::group_card_radii(1, 2, &metrics);
        assert_eq!((tl, tr, br, bl), (6.0, 6.0, 24.0, 24.0));

        // 1 group: all corners are container_corner (24.0)
        let (tl, tr, br, bl) = MenuModel::group_card_radii(0, 1, &metrics);
        assert_eq!((tl, tr, br, bl), (24.0, 24.0, 24.0, 24.0));
    }

    #[test]
    fn check_flex_col() {
        use gpui::Styled;
        let mut d = gpui::div().flex().flex_col();
        let style = d.style();
        assert!(style.display.is_some());
    }
}
