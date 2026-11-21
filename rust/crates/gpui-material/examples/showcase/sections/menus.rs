//! Section 7: Material 3 Expressive Menus showcase.
//!
//! Demonstrates Material 3 Expressive Menus matching AndroidX reference designs:
//! - Side-by-side presentation: Left (Icon Button trigger) and Right (Split Button trigger)
//! - Interactive selectable items with spring-animated hover state layers
//! - Switchable container styles: Grouped Cards (Style 1) and Unified Card (Style 2)
//! - Anchored dropdown menus with ample vertical clearance and chevron toggles

use gpui::{
    Context, IntoElement, ParentElement, SharedString, Styled, Window, div, prelude::*, px,
};
use gpui_material::{M3Icon, M3Palette, MenuGroup, MenuItem, MenuModel, MenuVariant};

use crate::Showcase;

/// State for the Menus showcase section.
pub(crate) struct MenusState {
    /// Left menu model (Icon Button anchored dropdown).
    pub(crate) standalone_menu: MenuModel,
    /// Whether the left icon button dropdown is open.
    pub(crate) icon_dropdown_open: bool,
    /// Currently selected action on the left menu.
    pub(crate) icon_action_label: SharedString,
    /// Right menu model (Split Button anchored dropdown).
    pub(crate) split_menu: MenuModel,
    /// Whether the right split button dropdown is open.
    pub(crate) split_dropdown_open: bool,
    /// Currently selected primary action on the split button.
    pub(crate) split_action_label: SharedString,
}

pub(crate) fn init_menus() -> MenusState {
    // 1. Left Menu: matching Reference 1 (Refresh, Settings, Help, More)
    let left_group = MenuGroup::new().items([
        MenuItem::new(1, "Refresh").trailing_icon(M3Icon::Refresh),
        MenuItem::new(2, "Settings").trailing_icon(M3Icon::Settings),
        MenuItem::new(3, "Help").trailing_icon(M3Icon::Help),
        MenuItem::new(4, "More").trailing_icon(M3Icon::MoreVert),
    ]);
    let mut standalone_menu = MenuModel::new([left_group]).variant(MenuVariant::UnifiedCard);
    standalone_menu.selected_id = None;
    standalone_menu.is_open = true;

    // 2. Right Menu: matching Reference 2 (Split Button with 2 Groups)
    let right_g1 = MenuGroup::new().items([
        MenuItem::new(101, "Item 1").leading_icon(M3Icon::Visibility),
        MenuItem::new(102, "Item 2")
            .leading_icon(M3Icon::ContentCopy)
            .shortcut("⌘C"),
        MenuItem::new(103, "Item 3")
            .leading_icon(M3Icon::Check)
            .selected(true),
    ]);
    let right_g2 = MenuGroup::new().items([
        MenuItem::new(104, "Item 4")
            .leading_icon(M3Icon::Person)
            .chevron(),
        MenuItem::new(105, "Item 5").leading_icon(M3Icon::Settings),
    ]);
    let mut split_menu = MenuModel::new([right_g1, right_g2]).variant(MenuVariant::GroupedCards);
    split_menu.select(103);
    split_menu.is_open = true;

    MenusState {
        standalone_menu,
        icon_dropdown_open: true,
        icon_action_label: "None".into(),
        split_menu,
        split_dropdown_open: true,
        split_action_label: "Item 3".into(),
    }
}

impl Showcase {
    /// Advances spring physics for all menus in the Menus showcase.
    pub(crate) fn advance_menus_springs(&mut self, window: &mut Window, dt: f32) {
        self.menus.standalone_menu.advance_springs(window, dt);
        self.menus.split_menu.advance_springs(window, dt);
    }

    /// Select an item in the left (icon button) menu.
    pub(crate) fn select_standalone_menu_item(&mut self, item_id: usize, cx: &mut Context<Self>) {
        self.menus.standalone_menu.select(item_id);
        if let Some(it) = self.menus.standalone_menu.selected_item() {
            self.menus.icon_action_label = it.label.clone();
        }
        cx.notify();
    }

    /// Toggle left icon button dropdown open or closed.
    pub(crate) fn toggle_icon_menu_dropdown(&mut self, cx: &mut Context<Self>) {
        self.menus.icon_dropdown_open = !self.menus.icon_dropdown_open;
        self.menus.standalone_menu.is_open = self.menus.icon_dropdown_open;
        cx.notify();
    }

    /// Select an item in the right (split button) dropdown menu.
    pub(crate) fn select_split_dropdown_item(
        &mut self,
        item_id: usize,
        label: SharedString,
        cx: &mut Context<Self>,
    ) {
        self.menus.split_menu.select(item_id);
        self.menus.split_action_label = label;
        cx.notify();
    }

    /// Toggle split button dropdown menu open or closed.
    pub(crate) fn toggle_split_menu_dropdown(&mut self, cx: &mut Context<Self>) {
        self.menus.split_dropdown_open = !self.menus.split_dropdown_open;
        self.menus.split_menu.is_open = self.menus.split_dropdown_open;
        cx.notify();
    }

    /// Renders the complete Menus showcase section.
    pub(crate) fn render_menus_section(
        &self,
        pal: &M3Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_grouped = self.menus.split_menu.variant == MenuVariant::GroupedCards;
        let left_selected = self.menus.icon_action_label.clone();
        let right_selected = self.menus.split_action_label.clone();

        let btn_bg = if self.dark_theme {
            gpui::Rgba {
                r: 215.0 / 255.0,
                g: 202.0 / 255.0,
                b: 237.0 / 255.0,
                a: 1.0,
            }
        } else {
            pal.secondary_container
        };
        let btn_fg = if self.dark_theme {
            gpui::Rgba {
                r: 56.0 / 255.0,
                g: 30.0 / 255.0,
                b: 80.0 / 255.0,
                a: 1.0,
            }
        } else {
            pal.on_secondary_container
        };

        div()
            .flex_col()
            .gap_6()
            .p_6()
            .rounded_2xl()
            .bg(gpui::Rgba {
                a: if self.dark_theme { 0.05 } else { 0.03 },
                ..pal.on_surface
            })
            // Section Header
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_between()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(M3Icon::Menu.element(22.0, pal.primary))
                                    .child(
                                        div()
                                            .text_lg()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(pal.on_surface)
                                            .child("Menus"),
                                    )
                                    .child(
                                        div()
                                            .px_2()
                                            .py_0p5()
                                            .rounded_full()
                                            .bg(pal.primary_container)
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(pal.on_primary)
                                            .child("AndroidX M3E"),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(pal.on_surface_variant)
                                    .child("Material 3 Expressive Menus · Grouped Cards (Callout 10) & Unified Card with Dividers (Callout 11)"),
                            ),
                    )
                    // Container Style Segmented Toggle
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .p_1()
                            .rounded_full()
                            .bg(pal.surface_container_highest)
                            // Grouped Cards button
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_full()
                                    .cursor_pointer()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .bg(if is_grouped {
                                        pal.secondary_container
                                    } else {
                                        gpui::Rgba { a: 0.0, ..pal.surface }
                                    })
                                    .text_color(if is_grouped {
                                        pal.on_secondary_container
                                    } else {
                                        pal.on_surface_variant
                                    })
                                    .child("Grouped Cards")
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.menus.standalone_menu.variant = MenuVariant::GroupedCards;
                                            this.menus.split_menu.variant = MenuVariant::GroupedCards;
                                            cx.notify();
                                        }),
                                    ),
                            )
                            // Unified Card button
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_full()
                                    .cursor_pointer()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .bg(if !is_grouped {
                                        pal.secondary_container
                                    } else {
                                        gpui::Rgba { a: 0.0, ..pal.surface }
                                    })
                                    .text_color(if !is_grouped {
                                        pal.on_secondary_container
                                    } else {
                                        pal.on_surface_variant
                                    })
                                    .child("Unified Card")
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.menus.standalone_menu.variant = MenuVariant::UnifiedCard;
                                            this.menus.split_menu.variant = MenuVariant::UnifiedCard;
                                            cx.notify();
                                        }),
                                    ),
                            ),
                    ),
            )
            // Centered Stage with BOTH Menus Side-by-Side:
            .child(
                div()
                    .flex()
                    .justify_center()
                    .items_start()
                    .gap_16()
                    .py_8()
                    .px_8()
                    .rounded_2xl()
                    .bg(if self.dark_theme {
                        gpui::Rgba {
                            r: 59.0 / 255.0,
                            g: 53.0 / 255.0,
                            b: 72.0 / 255.0,
                            a: 1.0,
                        }
                    } else {
                        gpui::Rgba {
                            r: 238.0 / 255.0,
                            g: 234.0 / 255.0,
                            b: 244.0 / 255.0,
                            a: 1.0,
                        }
                    })
                    .min_h(px(520.0))
                    // 1. Left Side: Icon Button Anchor (Reference 1)
                    .child(
                        div()
                            .flex_col()
                            .items_end()
                            .gap_3()
                            .w(px(220.0))
                            // Label / subtitle
                            .child(
                                div()
                                    .flex_col()
                                    .items_end()
                                    .gap_0p5()
                                    .w_full()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(pal.on_surface)
                                            .child("Icon Button Menu"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(pal.on_surface_variant)
                                            .child(format!("Selected: {}", left_selected)),
                                    ),
                            )
                            // Trigger Button: circular pill aligned above right edge of menu
                            .child(
                                div()
                                    .size_10()
                                    .rounded_full()
                                    .bg(btn_bg)
                                    .text_color(btn_fg)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .hover(|h| h.opacity(0.85))
                                    .child(M3Icon::MoreVert.element(20.0, btn_fg))
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.toggle_icon_menu_dropdown(cx);
                                        }),
                                    ),
                            )
                            // Left Menu (when open)
                            .when(self.menus.icon_dropdown_open, |d| {
                                d.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_end()
                                        .w_full()
                                        .child(
                                            self.menus.standalone_menu.render_with_handlers(
                                                pal,
                                                cx,
                                                |this, item_id, cx| {
                                                    this.select_standalone_menu_item(item_id, cx);
                                                },
                                            ),
                                        ),
                                )
                            }),
                    )
                    // 2. Right Side: Split Button Anchor (Reference 2)
                    .child(
                        div()
                            .flex_col()
                            .items_start()
                            .gap_3()
                            .w(px(260.0))
                            // Label / subtitle
                            .child(
                                div()
                                    .flex_col()
                                    .items_start()
                                    .gap_0p5()
                                    .w_full()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(pal.on_surface)
                                            .child("Split Button Menu"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(pal.on_surface_variant)
                                            .child(format!("Selected: {}", right_selected)),
                                    ),
                            )
                            // Split Button trigger
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    // Action button pill
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .h_10()
                                            .px_4()
                                            .rounded_full()
                                            .bg(btn_bg)
                                            .text_color(btn_fg)
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .cursor_pointer()
                                            .hover(|h| h.opacity(0.85))
                                            .child(M3Icon::Edit.element(16.0, btn_fg))
                                            .child(self.menus.split_action_label.clone())
                                            .on_mouse_down(
                                                gpui::MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.toggle_split_menu_dropdown(cx);
                                                }),
                                            ),
                                    )
                                    // Trailing chevron toggle button
                                    .child(
                                        div()
                                            .size_10()
                                            .rounded_full()
                                            .bg(btn_bg)
                                            .text_color(btn_fg)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .cursor_pointer()
                                            .hover(|h| h.opacity(0.85))
                                            .child(if self.menus.split_dropdown_open {
                                                M3Icon::ExpandLess.element(18.0, btn_fg)
                                            } else {
                                                M3Icon::ExpandMore.element(18.0, btn_fg)
                                            })
                                            .on_mouse_down(
                                                gpui::MouseButton::Left,
                                                cx.listener(|this, _, _, cx| {
                                                    this.toggle_split_menu_dropdown(cx);
                                                }),
                                            ),
                                    ),
                            )
                            // Right Menu (when open)
                            .when(self.menus.split_dropdown_open, |d| {
                                d.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_start()
                                        .w_full()
                                        .child(
                                            self.menus.split_menu.render_with_handlers(
                                                pal,
                                                cx,
                                                |this, item_id, cx| {
                                                    let label = this
                                                        .menus
                                                        .split_menu
                                                        .groups
                                                        .iter()
                                                        .flat_map(|g| &g.items)
                                                        .find(|it| it.id == item_id)
                                                        .map(|it| it.label.clone())
                                                        .unwrap_or_else(|| "Action".into());
                                                    this.select_split_dropdown_item(item_id, label, cx);
                                                },
                                            ),
                                        ),
                                )
                            }),
                    ),
            )
    }
}
