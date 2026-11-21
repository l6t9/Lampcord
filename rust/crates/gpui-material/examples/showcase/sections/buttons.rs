//! Section 4: Button models & rendering.

use gpui::{Context, IntoElement, ParentElement, Styled, Window, div, prelude::*};
use gpui_material::{ButtonModel, ButtonShape, ButtonSize, ButtonVariant, M3Icon, M3Palette};

use crate::Showcase;

pub(crate) fn init_buttons() -> Vec<ButtonModel> {
    vec![
        // 0: Extra Small (32dp)
        ButtonModel::new(0, "XS Action")
            .size(ButtonSize::ExtraSmall)
            .variant(ButtonVariant::Filled)
            .icon(M3Icon::Button),
        // 1: Small (40dp)
        ButtonModel::new(1, "Small Action")
            .size(ButtonSize::Small)
            .variant(ButtonVariant::Filled)
            .icon(M3Icon::Add),
        // 2: Medium (56dp)
        ButtonModel::new(2, "Medium Action")
            .size(ButtonSize::Medium)
            .variant(ButtonVariant::Filled)
            .icon(M3Icon::Check),
        // 3: Large (96dp)
        ButtonModel::new(3, "Large Action")
            .size(ButtonSize::Large)
            .variant(ButtonVariant::Filled)
            .icon(M3Icon::All),
        // 4: Filled Variant
        ButtonModel::new(4, "Filled")
            .size(ButtonSize::Small)
            .variant(ButtonVariant::Filled),
        // 5: Filled Tonal Variant
        ButtonModel::new(5, "Tonal")
            .size(ButtonSize::Small)
            .variant(ButtonVariant::FilledTonal),
        // 6: Elevated Variant
        ButtonModel::new(6, "Elevated")
            .size(ButtonSize::Small)
            .variant(ButtonVariant::Elevated),
        // 7: Outlined Variant
        ButtonModel::new(7, "Outlined")
            .size(ButtonSize::Small)
            .variant(ButtonVariant::Outlined),
        // 8: Text Variant
        ButtonModel::new(8, "Text")
            .size(ButtonSize::Small)
            .variant(ButtonVariant::Text),
        // 9: Square Medium
        ButtonModel::new(9, "Square Medium")
            .size(ButtonSize::Medium)
            .variant(ButtonVariant::FilledTonal)
            .shape(ButtonShape::Square),
        // 10: Square Small
        ButtonModel::new(10, "Square Small")
            .size(ButtonSize::Small)
            .variant(ButtonVariant::Outlined)
            .shape(ButtonShape::Square),
        // 11: Disabled Filled
        ButtonModel::new(11, "Disabled Filled")
            .size(ButtonSize::Small)
            .variant(ButtonVariant::Filled)
            .enabled(false),
        // 12: Disabled Outlined
        ButtonModel::new(12, "Disabled Outlined")
            .size(ButtonSize::Small)
            .variant(ButtonVariant::Outlined)
            .enabled(false),
        // 13: Card Action: Buy tickets (Filled)
        ButtonModel::new(13, "Buy tickets")
            .size(ButtonSize::ExtraSmall)
            .variant(ButtonVariant::Filled)
            .icon(M3Icon::Ticket),
        // 14: Card Action: Details (Outlined)
        ButtonModel::new(14, "Details")
            .size(ButtonSize::ExtraSmall)
            .variant(ButtonVariant::Outlined)
            .icon(M3Icon::Info),
        // 15: Card Action: Play (Filled)
        ButtonModel::new(15, "Play")
            .size(ButtonSize::ExtraSmall)
            .variant(ButtonVariant::Filled)
            .icon(M3Icon::Play),
        // 16: Card Action: Download (Disabled Outlined)
        ButtonModel::new(16, "Download")
            .size(ButtonSize::ExtraSmall)
            .variant(ButtonVariant::Outlined)
            .icon(M3Icon::Download)
            .enabled(false),
    ]
}

impl Showcase {
    pub(crate) fn advance_buttons_springs(&mut self, window: &mut Window, dt: f32) {
        for btn in &mut self.buttons {
            btn.advance_springs(window, dt);
        }
    }

    pub(crate) fn press_button(&mut self, ix: usize, pressed: bool, cx: &mut Context<Self>) {
        if !self.buttons[ix].enabled {
            return;
        }
        self.buttons[ix].is_pressed = pressed;
        cx.notify();
    }

    pub(crate) fn hover_button(&mut self, ix: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.buttons[ix].enabled {
            return;
        }
        self.buttons[ix].is_hovered = hovered;
        cx.notify();
    }

    pub(crate) fn render_button(
        &self,
        ix: usize,
        pal: &M3Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let btn = &self.buttons[ix];
        let rendered = btn.render(pal);
        div()
            .id(("m3-btn-wrap", ix))
            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                this.hover_button(ix, *hovered, cx);
            }))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.press_button(ix, true, cx);
                }),
            )
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.press_button(ix, false, cx);
                }),
            )
            .on_mouse_up_out(
                gpui::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.press_button(ix, false, cx);
                }),
            )
            .child(rendered)
    }

    pub(crate) fn render_buttons_section(
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
                            .child("Buttons (Expressive with Native with_spring)"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(pal.on_surface_variant)
                            .child("Element-level spring corner morphing via gpui 0.3.4"),
                    ),
            )
            // Row 1: Sizes (XS, Small, Medium, Large)
            .child(
                div()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(pal.on_surface_variant)
                            .child(
                                "Sizes: Extra Small (32dp), Small (40dp), Medium (56dp), Large (96dp)",
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .py_2()
                            .child(self.render_button(0, pal, cx))
                            .child(self.render_button(1, pal, cx))
                            .child(self.render_button(2, pal, cx))
                            .child(self.render_button(3, pal, cx)),
                    ),
            )
            // Row 2: Variants (Filled, Tonal, Elevated, Outlined, Text)
            .child(
                div()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(pal.on_surface_variant)
                            .child(
                                "Variants: Filled, Filled Tonal, Elevated, Outlined, Text",
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .py_2()
                            .child(self.render_button(4, pal, cx))
                            .child(self.render_button(5, pal, cx))
                            .child(self.render_button(6, pal, cx))
                            .child(self.render_button(7, pal, cx))
                            .child(self.render_button(8, pal, cx)),
                    ),
            )
            // Row 3: Shapes (Round vs Square) & Disabled
            .child(
                div()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(pal.on_surface_variant)
                            .child("Shapes (Round vs Square) & Disabled"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .py_2()
                            .child(self.render_button(9, pal, cx))
                            .child(self.render_button(10, pal, cx))
                            .child(self.render_button(11, pal, cx))
                            .child(self.render_button(12, pal, cx)),
                    ),
            )
    }
}
