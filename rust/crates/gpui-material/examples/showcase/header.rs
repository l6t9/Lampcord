//! Showcase header: branding title & dark/light theme switch.

use gpui::{Context, IntoElement, ParentElement, Styled, div, prelude::*};
use gpui_material::{M3Icon, M3Palette};

use crate::Showcase;

impl Showcase {
    pub(crate) fn render_header(
        &self,
        pal: &M3Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex()
            .justify_between()
            .items_center()
            .pb_2()
            .border_b_1()
            .border_color(pal.outline_variant)
            .child(
                div()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(pal.on_surface)
                            .child("Material 3 Expressive"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(pal.on_surface_variant)
                            .child("Component Showcase • Damped Spring Physics & Dynamic Theming"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .px_3()
                    .py_1()
                    .rounded_full()
                    .bg(pal.secondary_container)
                    .text_color(pal.on_secondary_container)
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .cursor_pointer()
                    .child(if self.dark_theme {
                        M3Icon::LightMode.element(14.0, pal.on_secondary_container)
                    } else {
                        M3Icon::DarkMode.element(14.0, pal.on_secondary_container)
                    })
                    .child(if self.dark_theme {
                        "Light Theme"
                    } else {
                        "Dark Theme"
                    })
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.dark_theme = !this.dark_theme;
                            cx.notify();
                        }),
                    ),
            )
    }
}
