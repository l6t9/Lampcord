//! Section 2: Switch models & rendering.

use gpui::{Context, IntoElement, ParentElement, Styled, Window, div, prelude::*};
use gpui_material::{M3Palette, SwitchModel};

use crate::Showcase;

pub(crate) fn init_switches() -> Vec<SwitchModel> {
    vec![
        SwitchModel::new("Airplane Mode", true),
        SwitchModel::new("Bluetooth", false),
        SwitchModel::new("Haptics", true).show_icons(false),
        SwitchModel::new("Night Light", true).enabled(false),
        SwitchModel::new("Mobile Data", false).enabled(false),
    ]
}

impl Showcase {
    pub(crate) fn advance_switches_springs(&mut self, window: &mut Window, dt: f32) {
        for sw in &mut self.switches {
            sw.advance_springs(window, dt);
        }
    }

    pub(crate) fn toggle_switch(&mut self, ix: usize, cx: &mut Context<Self>) {
        if !self.switches[ix].enabled {
            return;
        }
        self.switches[ix].toggle();
        cx.notify();
    }

    pub(crate) fn press_switch(&mut self, ix: usize, pressed: bool, cx: &mut Context<Self>) {
        if !self.switches[ix].enabled {
            return;
        }
        self.switches[ix].is_pressed = pressed;
        cx.notify();
    }

    pub(crate) fn hover_switch(&mut self, ix: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.switches[ix].enabled {
            return;
        }
        self.switches[ix].is_hovered = hovered;
        cx.notify();
    }

    pub(crate) fn render_switches_section(
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
                            .child("Switches"),
                    )
                    .child(div().text_xs().text_color(pal.on_surface_variant).child(
                        "Material 3 Expressive • Damped Spring Morphing & Animated Vector Icons",
                    )),
            )
            .child(
                div()
                    .flex_col()
                    .gap_3()
                    .children(self.switches.iter().enumerate().map(|(ix, sw)| {
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
                                            .child(sw.label.clone()),
                                    )
                                    .child(
                                        div().text_xs().text_color(pal.on_surface_variant).child(
                                            if !sw.enabled {
                                                "Disabled"
                                            } else if sw.checked {
                                                "On"
                                            } else {
                                                "Off"
                                            },
                                        ),
                                    ),
                            )
                            .child(
                                div()
                                    .id(("m3-switch", ix))
                                    .when(sw.enabled, |d| d.cursor_pointer())
                                    .when(!sw.enabled, |d| d.cursor_not_allowed())
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.press_switch(ix, true, cx)
                                        }),
                                    )
                                    .on_mouse_up(
                                        gpui::MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.press_switch(ix, false, cx);
                                            this.toggle_switch(ix, cx);
                                        }),
                                    )
                                    .on_mouse_up_out(
                                        gpui::MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.press_switch(ix, false, cx)
                                        }),
                                    )
                                    .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                                        this.hover_switch(ix, *hovered, cx);
                                        if !*hovered {
                                            this.press_switch(ix, false, cx);
                                        }
                                    }))
                                    .child(sw.render(pal)),
                            )
                    })),
            )
    }
}
