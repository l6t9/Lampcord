//! Section 3: Split button models, menu handlers & rendering.

use gpui::{Context, IntoElement, ParentElement, Styled, Window, deferred, div, prelude::*, px};
use gpui_material::{
    M3Palette, SplitButtonModel, SplitButtonSize, SplitButtonVariant, SplitIconKind,
    draw_chevron_icon, draw_split_icon,
};

use crate::Showcase;

pub(crate) fn init_split_buttons() -> Vec<SplitButtonModel> {
    vec![
        // 0: Filled (Save) with Check icon
        SplitButtonModel::new("Save")
            .variant(SplitButtonVariant::Filled)
            .size(SplitButtonSize::Medium)
            .icon(SplitIconKind::Check)
            .menu_items(["Save & Publish", "Save Draft", "Export"]),
        // 1: Tonal (Edit Document) with Edit icon
        SplitButtonModel::new("Edit Document")
            .variant(SplitButtonVariant::Tonal)
            .size(SplitButtonSize::Small)
            .icon(SplitIconKind::Edit)
            .menu_items(["Duplicate", "Rename", "Move"]),
        // 2: Elevated (Share Project) with Share icon
        SplitButtonModel::new("Share Project")
            .variant(SplitButtonVariant::Elevated)
            .size(SplitButtonSize::Small)
            .icon(SplitIconKind::Share)
            .menu_items(["Copy Link", "Email Invite", "Permissions"]),
        // 3: Outlined (Options)
        SplitButtonModel::new("Options")
            .variant(SplitButtonVariant::Outlined)
            .size(SplitButtonSize::Small)
            .menu_items(["Settings", "Shortcuts", "Reset"]),
        // 4: Extra Small (XS)
        SplitButtonModel::new("XS Action")
            .variant(SplitButtonVariant::Filled)
            .size(SplitButtonSize::ExtraSmall)
            .menu_items(["Action 1", "Action 2"]),
        // 5: Small Standard
        SplitButtonModel::new("Small Standard")
            .variant(SplitButtonVariant::Filled)
            .size(SplitButtonSize::Small)
            .menu_items(["Action 1", "Action 2"]),
        // 6: Medium Action
        SplitButtonModel::new("Medium Action")
            .variant(SplitButtonVariant::Tonal)
            .size(SplitButtonSize::Medium)
            .menu_items(["Action 1", "Action 2"]),
        // 7: Disabled Filled
        SplitButtonModel::new("Disabled Filled")
            .variant(SplitButtonVariant::Filled)
            .size(SplitButtonSize::Small)
            .enabled(false),
        // 8: Disabled Outlined
        SplitButtonModel::new("Disabled Outlined")
            .variant(SplitButtonVariant::Outlined)
            .size(SplitButtonSize::Small)
            .enabled(false),
    ]
}

impl Showcase {
    pub(crate) fn advance_split_buttons_springs(&mut self, window: &mut Window, dt: f32) {
        for btn in &mut self.split_buttons {
            btn.advance_springs(window, dt);
        }
    }

    pub(crate) fn press_split_leading(&mut self, ix: usize, pressed: bool, cx: &mut Context<Self>) {
        if !self.split_buttons[ix].enabled {
            return;
        }
        self.split_buttons[ix].is_leading_pressed = pressed;
        cx.notify();
    }

    pub(crate) fn hover_split_leading(&mut self, ix: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.split_buttons[ix].enabled {
            return;
        }
        self.split_buttons[ix].is_leading_hovered = hovered;
        cx.notify();
    }

    pub(crate) fn click_split_leading(&mut self, ix: usize, cx: &mut Context<Self>) {
        if !self.split_buttons[ix].enabled {
            return;
        }
        cx.notify();
    }

    pub(crate) fn press_split_trailing(
        &mut self,
        ix: usize,
        pressed: bool,
        cx: &mut Context<Self>,
    ) {
        if !self.split_buttons[ix].enabled {
            return;
        }
        self.split_buttons[ix].is_trailing_pressed = pressed;
        cx.notify();
    }

    pub(crate) fn hover_split_trailing(
        &mut self,
        ix: usize,
        hovered: bool,
        cx: &mut Context<Self>,
    ) {
        if !self.split_buttons[ix].enabled {
            return;
        }
        self.split_buttons[ix].is_trailing_hovered = hovered;
        cx.notify();
    }

    pub(crate) fn toggle_split_dropdown(&mut self, ix: usize, cx: &mut Context<Self>) {
        if !self.split_buttons[ix].enabled {
            return;
        }
        let was_open = self.split_buttons[ix].checked;
        for (i, b) in self.split_buttons.iter_mut().enumerate() {
            if i != ix {
                b.close_dropdown();
            }
        }
        if was_open {
            self.split_buttons[ix].close_dropdown();
        } else {
            self.split_buttons[ix].toggle_dropdown();
        }
        cx.notify();
    }

    pub(crate) fn close_split_dropdown(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.split_buttons[ix].close_dropdown();
        cx.notify();
    }

    pub(crate) fn select_split_menu_item(
        &mut self,
        ix: usize,
        item_ix: usize,
        cx: &mut Context<Self>,
    ) {
        self.split_buttons[ix].select_menu_item(item_ix);
        cx.notify();
    }

    pub(crate) fn render_split_button(
        &self,
        ix: usize,
        pal: &M3Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let btn = &self.split_buttons[ix];
        let metrics = btn.metrics();
        let colors = btn.resolve_colors(pal);
        let (l_tl, l_tr, l_br, l_bl) = btn.leading_corners();
        let (t_tl, t_tr, t_br, t_bl) = btn.trailing_corners();

        let trailing_w = (metrics.trailing_padding_start
            + metrics.trailing_icon_size
            + metrics.trailing_padding_end)
            .max(metrics.container_height * 0.75);

        div()
            .relative()
            .flex()
            .items_center()
            // Leading primary action button
            .child(
                div()
                    .id(("split-leading", ix))
                    .relative()
                    .h(px(metrics.container_height))
                    .w(px(btn.leading_width.pos))
                    .overflow_hidden()
                    .flex()
                    .items_center()
                    .justify_center()
                    .pl(px(metrics.leading_padding_start))
                    .pr(px(metrics.leading_padding_end))
                    .rounded_tl(px(l_tl))
                    .rounded_tr(px(l_tr))
                    .rounded_br(px(l_br))
                    .rounded_bl(px(l_bl))
                    .bg(if btn.enabled {
                        colors.container
                    } else {
                        colors.disabled_container
                    })
                    .when(btn.enabled && colors.has_shadow, |d| d.shadow_sm())
                    .when_some(
                        if btn.enabled {
                            colors.border
                        } else {
                            colors.disabled_border
                        },
                        |d, b| d.border_1().border_color(b),
                    )
                    .when(!btn.enabled, |d| d.cursor_not_allowed())
                    .when(btn.enabled, |d| d.cursor_pointer())
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.press_split_leading(ix, true, cx);
                        }),
                    )
                    .on_mouse_up(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.press_split_leading(ix, false, cx);
                            this.click_split_leading(ix, cx);
                        }),
                    )
                    .on_mouse_up_out(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.press_split_leading(ix, false, cx);
                        }),
                    )
                    .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                        this.hover_split_leading(ix, *hovered, cx);
                        if !*hovered {
                            this.press_split_leading(ix, false, cx);
                        }
                    }))
                    // State layer overlay
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .rounded_tl(px(l_tl))
                            .rounded_tr(px(l_tr))
                            .rounded_br(px(l_br))
                            .rounded_bl(px(l_bl))
                            .bg(colors.state_layer)
                            .opacity(btn.leading_state_layer.pos),
                    )
                    // Optional leading icon
                    .when(btn.icon != SplitIconKind::None, |d| {
                        let icon_sz = (metrics.container_height * 0.38).clamp(14.0, 24.0);
                        let icon_kind = btn.icon;
                        let icon_color = if btn.enabled {
                            colors.content
                        } else {
                            colors.disabled_content
                        };
                        d.child(
                            gpui::canvas(
                                move |bounds, _, _| (bounds, icon_kind, icon_color),
                                move |_, (bounds, k, c), window, _| {
                                    draw_split_icon(window, bounds, k, c);
                                },
                            )
                            .size(px(icon_sz))
                            .flex_shrink_0()
                            .mr_2(),
                        )
                    })
                    // Button Label
                    .child(
                        div()
                            .text_size(px(metrics.font_size))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .flex_shrink_0()
                            .whitespace_nowrap()
                            .text_color(if btn.enabled {
                                colors.content
                            } else {
                                colors.disabled_content
                            })
                            .child(btn.label.clone()),
                    ),
            )
            // 2dp spacing between buttons from M3E tokens
            .child(div().w(px(metrics.between_space)))
            // Trailing toggle/menu button
            .child(
                div()
                    .id(("split-trailing", ix))
                    .relative()
                    .h(px(metrics.container_height))
                    .w(px(trailing_w))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_tl(px(t_tl))
                    .rounded_tr(px(t_tr))
                    .rounded_br(px(t_br))
                    .rounded_bl(px(t_bl))
                    .bg(if btn.enabled {
                        colors.container
                    } else {
                        colors.disabled_container
                    })
                    .when(btn.enabled && colors.has_shadow, |d| d.shadow_sm())
                    .when_some(
                        if btn.enabled {
                            colors.border
                        } else {
                            colors.disabled_border
                        },
                        |d, b| d.border_1().border_color(b),
                    )
                    .when(!btn.enabled, |d| d.cursor_not_allowed())
                    .when(btn.enabled, |d| d.cursor_pointer())
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.press_split_trailing(ix, true, cx);
                        }),
                    )
                    .on_mouse_up(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.press_split_trailing(ix, false, cx);
                            this.toggle_split_dropdown(ix, cx);
                        }),
                    )
                    .on_mouse_up_out(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.press_split_trailing(ix, false, cx);
                        }),
                    )
                    .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                        this.hover_split_trailing(ix, *hovered, cx);
                        if !*hovered {
                            this.press_split_trailing(ix, false, cx);
                        }
                    }))
                    // State layer overlay
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .rounded_tl(px(t_tl))
                            .rounded_tr(px(t_tr))
                            .rounded_br(px(t_br))
                            .rounded_bl(px(t_bl))
                            .bg(colors.state_layer)
                            .opacity(btn.trailing_state_layer.pos),
                    )
                    // Animated vector chevron icon
                    .child({
                        let chevron_pos = btn.chevron_progress.pos;
                        let chevron_color = if btn.enabled {
                            colors.content
                        } else {
                            colors.disabled_content
                        };
                        gpui::canvas(
                            move |bounds, _, _| (bounds, chevron_pos, chevron_color),
                            move |_, (bounds, prog, col), window, _| {
                                draw_chevron_icon(window, bounds, prog, col);
                            },
                        )
                        .size(px(metrics.trailing_icon_size))
                    }),
            )
            // Anchored dropdown menu popover
            .when(
                (btn.checked || btn.menu_progress.pos > 0.01) && !btn.menu_items.is_empty(),
                |d| {
                    let anim = btn.menu_progress.pos.clamp(0.0, 1.0);
                    // Fast, snappy slide down
                    let slide_distance = 16.0;
                    let top_offset = metrics.container_height + 4.0 - (1.0 - anim) * slide_distance;
                    let opacity = (anim * 5.0).clamp(0.0, 1.0);
                    // Auto-scale modal width to match the split button container width exactly (or expand if items require)
                    let total_btn_w = btn.leading_width.pos + metrics.between_space + trailing_w;
                    let menu_w = total_btn_w.max(btn.menu_content_width());
                    d.child(
                        deferred(
                            div()
                                .id(("split-menu", ix))
                                .occlude()
                                .absolute()
                                .top(px(top_offset))
                                .left_0()
                                .w(px(menu_w))
                                .p(px(metrics.menu_item_padding_y * 0.75))
                                .when(btn.size == SplitButtonSize::ExtraSmall, |s| s.rounded_md())
                                .when(btn.size == SplitButtonSize::Small, |s| s.rounded_lg())
                                .when(btn.size == SplitButtonSize::Medium, |s| s.rounded_xl())
                                .when(btn.size >= SplitButtonSize::Large, |s| s.rounded_2xl())
                                .bg(pal.surface_container_highest)
                                .border_1()
                                .border_color(pal.outline_variant)
                                .shadow_lg()
                                .opacity(opacity)
                                .on_mouse_down_out(cx.listener(move |this, _, _, cx| {
                                    this.close_split_dropdown(ix, cx);
                                }))
                                .children(btn.menu_items.iter().enumerate().map(
                                    |(item_ix, item)| {
                                        let is_selected = btn.selected_index == Some(item_ix);
                                        div()
                                            .id(("split-menu-item", ix * 100 + item_ix))
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .px(px(metrics.menu_item_padding_x))
                                            .py(px(metrics.menu_item_padding_y))
                                            .when(btn.size == SplitButtonSize::ExtraSmall, |s| {
                                                s.rounded_sm()
                                            })
                                            .when(btn.size == SplitButtonSize::Small, |s| {
                                                s.rounded_md()
                                            })
                                            .when(btn.size >= SplitButtonSize::Medium, |s| {
                                                s.rounded_lg()
                                            })
                                            .cursor_pointer()
                                            .text_size(px(metrics.menu_font_size))
                                            .hover(|s| {
                                                s.bg(pal.secondary_container)
                                                    .text_color(pal.on_secondary_container)
                                            })
                                            .when(is_selected, |s| {
                                                s.font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .text_color(pal.primary)
                                            })
                                            .when(!is_selected, |s| s.text_color(pal.on_surface))
                                            .child(item.clone())
                                            .when(is_selected, |s| s.child("✓"))
                                            .on_mouse_down(
                                                gpui::MouseButton::Left,
                                                cx.listener(move |this, _, _, cx| {
                                                    this.select_split_menu_item(ix, item_ix, cx);
                                                }),
                                            )
                                    },
                                )),
                        )
                        .priority(100),
                    )
                },
            )
    }

    pub(crate) fn render_split_buttons_section(
        &self,
        pal: &M3Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex_col()
            .gap_4()
            .p_6()
            .rounded_2xl()
            .bg(gpui::Rgba {
                a: if self.dark_theme { 0.08 } else { 0.04 },
                ..pal.on_surface
            })
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_baseline()
                    .child(
                        div()
                            .text_base()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Split Buttons"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(pal.on_surface_variant)
                            .child("Material 3 Expressive • Damped Spring Shape Morphing"),
                    ),
            )
            // 1. Emphasis Variants (Interactive Dropdowns)
            .child(
                div()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(pal.on_surface_variant)
                            .child("Emphasis Variants (Interactive Dropdowns) • Hover/press inner corners (4px → 12px), chevron toggles menu"),
                    )
                    .child(
                        div()
                            .flex_col()
                            .gap_2()
                            .py_1()
                            // Filled
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .py_1()
                                    .child(
                                        div()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .child("Filled Split Button"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(pal.on_surface_variant)
                                                    .child("High emphasis • Primary container • Medium size (56dp)"),
                                            ),
                                    )
                                    .child(self.render_split_button(0, pal, cx)),
                            )
                            // Tonal
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .py_1()
                                    .child(
                                        div()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .child("Tonal Split Button"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(pal.on_surface_variant)
                                                    .child("Medium-high emphasis • SecondaryContainer • Medium size (56dp)"),
                                            ),
                                    )
                                    .child(self.render_split_button(1, pal, cx)),
                            )
                            // Elevated
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .py_1()
                                    .child(
                                        div()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .child("Elevated Split Button"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(pal.on_surface_variant)
                                                    .child("Elevation shadow • Primary content • Small size (40dp)"),
                                            ),
                                    )
                                    .child(self.render_split_button(2, pal, cx)),
                            )
                            // Outlined
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .py_1()
                                    .child(
                                        div()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .child("Outlined Split Button"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(pal.on_surface_variant)
                                                    .child("1px Outline border • Transparent container • Small size (40dp)"),
                                            ),
                                    )
                                    .child(self.render_split_button(3, pal, cx)),
                            ),
                    ),
            )
            // 2. Token Sizes Comparison
            .child(
                div()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(pal.on_surface_variant)
                            .child("Token Size Hierarchy: Extra Small (32dp), Small (40dp), Medium (56dp)"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_4()
                            .py_2()
                            // ExtraSmall (32dp)
                            .child(
                                div()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(pal.on_surface_variant)
                                            .child("Extra Small (32dp)"),
                                    )
                                    .child(self.render_split_button(4, pal, cx)),
                            )
                            // Small (40dp)
                            .child(
                                div()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(pal.on_surface_variant)
                                            .child("Small (40dp)"),
                                    )
                                    .child(self.render_split_button(5, pal, cx)),
                            )
                            // Medium (56dp)
                            .child(
                                div()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(pal.on_surface_variant)
                                            .child("Medium (56dp)"),
                                    )
                                    .child(self.render_split_button(6, pal, cx)),
                            ),
                    ),
            )
            // 3. Disabled States
            .child(
                div()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(pal.on_surface_variant)
                            .child("Disabled States (12% container opacity, 38% content opacity)"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_6()
                            .py_1()
                            .child(self.render_split_button(7, pal, cx))
                            .child(self.render_split_button(8, pal, cx)),
                    ),
            )
    }
}
