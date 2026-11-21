//! Section 5: Card models, media banners & composed action buttons.

use gpui::{Context, IntoElement, ParentElement, Styled, Window, div, prelude::*, px};
use gpui_material::{CardModel, CardVariant, M3Palette};

use crate::Showcase;

pub(crate) fn init_cards() -> Vec<CardModel> {
    vec![
        // 0: Filled Card
        CardModel::new(0)
            .variant(CardVariant::Filled)
            .title("Filled Card")
            .subhead("SurfaceContainerHighest")
            .body("Low-emphasis filled card providing subtle visual grouping."),
        // 1: Elevated Card
        CardModel::new(1)
            .variant(CardVariant::Elevated)
            .title("Elevated Card")
            .subhead("Surface + Level 1 Shadow")
            .body("Medium-emphasis container with physical elevation shadow."),
        // 2: Outlined Card
        CardModel::new(2)
            .variant(CardVariant::Outlined)
            .title("Outlined Card")
            .subhead("Surface + 1dp OutlineVariant")
            .body("High-emphasis boundary outline separating complex content."),
        // 3: Interactive Elevated Card
        CardModel::new(3)
            .variant(CardVariant::Elevated)
            .interactive(true)
            .title("Clickable Card")
            .subhead("Hover & Press Feedback")
            .body("Smooth spring corner morphing from 12dp to 8dp on press with state layer."),
        // 4: Interactive Filled Card
        CardModel::new(4)
            .variant(CardVariant::Filled)
            .interactive(true)
            .title("Tactile Feedback")
            .subhead("Expressive Physics")
            .body("Interactive filled card with state layer tint and damped harmonic springs."),
        // 5: Elevated Media Card with Action (from M3 Guidelines)
        CardModel::new(5)
            .variant(CardVariant::Elevated)
            .media(84.0, Some("🎈  Concert Tour"))
            .title("Glass Souls' World Tour")
            .subhead("From your recent favorites")
            .body("Tickets and venue details are live for worldwide presale access."),
        // 6: Outlined Podcast Card with Actions (from M3 Guidelines)
        CardModel::new(6)
            .variant(CardVariant::Outlined)
            .media(84.0, Some("🎙️  Podcast"))
            .title("90th minute")
            .subhead("Saved podcasts • 4.31 MB")
            .body("Tactical breakdown and analysis of stoppage-time drama."),
        // 7: Disabled Card with Inactive Action
        CardModel::new(7)
            .variant(CardVariant::Outlined)
            .enabled(false)
            .title("Archived Episode")
            .subhead("Unavailable Offline")
            .body("Disabled cards dim container content, outlines, and actions to 38% opacity."),
    ]
}

impl Showcase {
    pub(crate) fn advance_cards_springs(&mut self, window: &mut Window, dt: f32) {
        for card in &mut self.cards {
            card.advance_springs(window, dt);
        }
    }

    pub(crate) fn press_card(&mut self, ix: usize, pressed: bool, cx: &mut Context<Self>) {
        if !self.cards[ix].enabled || !self.cards[ix].interactive {
            return;
        }
        self.cards[ix].is_pressed = pressed;
        cx.notify();
    }

    pub(crate) fn hover_card(&mut self, ix: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.cards[ix].enabled || !self.cards[ix].interactive {
            return;
        }
        self.cards[ix].is_hovered = hovered;
        cx.notify();
    }

    pub(crate) fn render_card(
        &self,
        ix: usize,
        pal: &M3Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let card = &self.cards[ix];
        let content = match ix {
            5 => {
                // Card 5: Concert Tour with real ButtonModel (Button 13: "Buy tickets")
                card.render_content(pal).child(
                    div()
                        .px(px(card.metrics().padding_horizontal))
                        .pb(px(card.metrics().padding_vertical))
                        .flex()
                        .items_center()
                        .child(self.render_button(13, pal, cx)),
                )
            }
            6 => {
                // Card 6: Podcast with real ButtonModels (Button 14: "Details", Button 15: "Play")
                card.render_content(pal).child(
                    div()
                        .px(px(card.metrics().padding_horizontal))
                        .pb(px(card.metrics().padding_vertical))
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(self.render_button(14, pal, cx))
                        .child(self.render_button(15, pal, cx)),
                )
            }
            7 => {
                // Card 7: Archived Episode with real disabled ButtonModel (Button 16: "Download")
                card.render_content(pal).child(
                    div()
                        .px(px(card.metrics().padding_horizontal))
                        .pb(px(card.metrics().padding_vertical))
                        .flex()
                        .items_center()
                        .child(self.render_button(16, pal, cx)),
                )
            }
            _ => card.render_content(pal),
        };
        let card_element = card.render_with(pal, content);

        div()
            .id(("m3-card-wrap", ix))
            .w_full()
            .when(card.interactive, |d| {
                d.on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                    this.hover_card(ix, *hovered, cx);
                }))
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.press_card(ix, true, cx);
                    }),
                )
                .on_mouse_up(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.press_card(ix, false, cx);
                    }),
                )
                .on_mouse_up_out(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.press_card(ix, false, cx);
                    }),
                )
            })
            .child(card_element)
    }

    pub(crate) fn render_cards_section(
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
                            .child("Cards"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(pal.on_surface_variant)
                            .child("Filled, Elevated, Outlined & Tactile Corner Morphing"),
                    ),
            )
            // Row 1: The Three Variants (Filled, Elevated, Outlined)
            .child(
                div()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(pal.on_surface_variant)
                            .child("Variants: Filled (SurfaceContainerHighest), Elevated (Level 1 Shadow), Outlined (1dp Border)"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap_4()
                            .py_2()
                            .child(div().flex_1().child(self.render_card(0, pal, cx)))
                            .child(div().flex_1().child(self.render_card(1, pal, cx)))
                            .child(div().flex_1().child(self.render_card(2, pal, cx))),
                    ),
            )
            // Row 2: Interactive Cards (Corner Morphing 12dp -> 8dp & State Layer)
            .child(
                div()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(pal.on_surface_variant)
                            .child("Interactive Cards: Hover for State Layer / Shadow Lift, Press for Spring Corner Morph (12dp → 8dp)"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap_4()
                            .py_2()
                            .child(div().flex_1().child(self.render_card(3, pal, cx)))
                            .child(div().flex_1().child(self.render_card(4, pal, cx))),
                    ),
            )
            // Row 3: Media Header Cards & Disabled
            .child(
                div()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(pal.on_surface_variant)
                            .child("Media Containers, Action Buttons & Inactive States (38% Opacity)"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap_4()
                            .py_2()
                            .child(div().flex_1().child(self.render_card(5, pal, cx)))
                            .child(div().flex_1().child(self.render_card(6, pal, cx)))
                            .child(div().flex_1().child(self.render_card(7, pal, cx))),
                    ),
            )
    }
}
