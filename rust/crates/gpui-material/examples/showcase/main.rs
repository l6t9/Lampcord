//! Interactive visual showcase demonstrating Material 3 Expressive components.

mod header;
mod sections;

use gpui::{
    App, Application, Bounds, Context, IntoElement, ParentElement, Render, Styled, Window,
    WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use gpui_material::{
    ButtonModel, ButtonSize, ButtonVariant, CardModel, M3Icon, M3Palette, SplitButtonModel,
    SwitchModel, dark_scheme, light_scheme,
};
use gpui_platform::application;

use sections::{
    buttons::init_buttons,
    cards::init_cards,
    menus::{MenusState, init_menus},
    shapes::ShapePlaygroundModel,
    sliders::{SliderModel, init_sliders},
    split_buttons::init_split_buttons,
    switches::init_switches,
};

pub(crate) const DT: f32 = 1.0 / 60.0;

/// Available sections in the Material 3 Expressive showcase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShowcaseSection {
    All,
    Shapes,
    Sliders,
    Switches,
    SplitButtons,
    Buttons,
    Cards,
    Menus,
}

impl ShowcaseSection {
    pub(crate) const ALL: [Self; 8] = [
        Self::All,
        Self::Shapes,
        Self::Sliders,
        Self::Switches,
        Self::SplitButtons,
        Self::Buttons,
        Self::Cards,
        Self::Menus,
    ];

    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::All => "All Components",
            Self::Shapes => "35 Shapes & Morpher",
            Self::Sliders => "Sliders",
            Self::Switches => "Switches",
            Self::SplitButtons => "Split Buttons",
            Self::Buttons => "Buttons",
            Self::Cards => "Cards",
            Self::Menus => "Menus",
        }
    }

    pub(crate) fn short_name(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Shapes => "Shapes",
            Self::Sliders => "Sliders",
            Self::Switches => "Switches",
            Self::SplitButtons => "Split Buttons",
            Self::Buttons => "Buttons",
            Self::Cards => "Cards",
            Self::Menus => "Menus",
        }
    }

    pub(crate) fn icon(&self) -> M3Icon {
        match self {
            Self::All => M3Icon::All,
            Self::Shapes => M3Icon::Shapes,
            Self::Sliders => M3Icon::Sliders,
            Self::Switches => M3Icon::Switches,
            Self::SplitButtons => M3Icon::SplitButton,
            Self::Buttons => M3Icon::Button,
            Self::Cards => M3Icon::Cards,
            Self::Menus => M3Icon::Menu,
        }
    }

    pub(crate) fn index(&self) -> usize {
        match self {
            Self::All => 0,
            Self::Shapes => 1,
            Self::Sliders => 2,
            Self::Switches => 3,
            Self::SplitButtons => 4,
            Self::Buttons => 5,
            Self::Cards => 6,
            Self::Menus => 7,
        }
    }

    pub(crate) fn prev(&self) -> Option<Self> {
        let idx = self.index();
        if idx > 1 {
            Some(Self::ALL[idx - 1])
        } else {
            None
        }
    }

    pub(crate) fn next(&self) -> Option<Self> {
        let idx = self.index();
        if idx > 0 && idx + 1 < Self::ALL.len() {
            Some(Self::ALL[idx + 1])
        } else {
            None
        }
    }
}

/// Material 3 Expressive visual showcase application state.
pub(crate) struct Showcase {
    /// Whether the showcase is currently rendering in dark theme mode.
    pub(crate) dark_theme: bool,
    /// Currently focused showcase section.
    pub(crate) current_section: ShowcaseSection,
    /// 35 Expressive shapes and harmonic spring morpher playground model.
    pub(crate) shapes: ShapePlaygroundModel,
    /// Switch models demonstrating toggles and morphing vector icons.
    pub(crate) switches: Vec<SwitchModel>,
    /// Slider models demonstrating continuous and discrete stepped tracks.
    pub(crate) sliders: Vec<SliderModel>,
    /// Split button models demonstrating leading actions and flyout menus.
    pub(crate) split_buttons: Vec<SplitButtonModel>,
    /// Button models demonstrating sizes, variants, and shape morphing.
    pub(crate) buttons: Vec<ButtonModel>,
    /// Card models demonstrating containers, corner morphing, and media headers.
    pub(crate) cards: Vec<CardModel>,
    /// Menus showcase models and states.
    pub(crate) menus: MenusState,
    /// Previous section footer button model.
    pub(crate) prev_button: ButtonModel,
    /// Next section footer button model.
    pub(crate) next_button: ButtonModel,
}

impl Showcase {
    /// Render top navigation segmented bar to switch between showcase sections.
    fn render_navigation_bar(&self, pal: &M3Palette, cx: &Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap_3()
            .p_2()
            .rounded_2xl()
            .bg(gpui::Rgba {
                a: if self.dark_theme { 0.12 } else { 0.05 },
                ..pal.on_surface
            })
            .child(div().flex().flex_wrap().items_center().gap_1p5().children(
                ShowcaseSection::ALL.iter().copied().map(|sec| {
                    let is_active = self.current_section == sec;
                    let icon_color = if is_active {
                        pal.on_primary
                    } else {
                        pal.on_surface_variant
                    };
                    div()
                        .id(("nav-tab", sec.index()))
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_3()
                        .py_1p5()
                        .rounded_full()
                        .cursor_pointer()
                        .text_xs()
                        .when(is_active, |s| {
                            s.bg(pal.primary)
                                .text_color(pal.on_primary)
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .shadow_sm()
                        })
                        .when(!is_active, |s| {
                            s.bg(pal.surface_container_highest)
                                .text_color(pal.on_surface_variant)
                                .hover(|h| {
                                    h.bg(pal.secondary_container)
                                        .text_color(pal.on_secondary_container)
                                })
                        })
                        .child(sec.icon().element(14.0, icon_color))
                        .child(sec.name())
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                this.current_section = sec;
                                cx.notify();
                            }),
                        )
                }),
            ))
            .child(
                div().flex().items_center().gap_2().px_3().child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(pal.on_surface_variant)
                        .child(if self.current_section == ShowcaseSection::All {
                            "Full Showcase (All 6 Sections)".to_string()
                        } else {
                            format!("Section {} of 6", self.current_section.index())
                        }),
                ),
            )
    }

    /// Render previous / next section footer buttons.
    fn render_section_footer(&self, pal: &M3Palette, cx: &Context<Self>) -> impl IntoElement {
        if self.current_section == ShowcaseSection::All {
            return div()
                .flex()
                .justify_center()
                .items_center()
                .pt_4()
                .pb_4()
                .border_t_1()
                .border_color(pal.outline_variant)
                .child(div().text_xs().text_color(pal.on_surface_variant).child(
                    "Viewing All Components • Use tabs above to focus on individual sections",
                ));
        }

        let prev_sec = self.current_section.prev();
        let next_sec = self.current_section.next();
        div()
            .flex()
            .justify_between()
            .items_center()
            .pt_4()
            .pb_4()
            .border_t_1()
            .border_color(pal.outline_variant)
            .child(
                div()
                    .when_some(prev_sec, |d, prev| {
                        let mut btn = self.prev_button.clone();
                        btn.label = format!("Previous: {}", prev.short_name()).into();
                        let rendered = btn.render(pal);
                        d.child(
                            div()
                                .id("footer-prev-btn")
                                .on_hover(cx.listener(|this, hovered: &bool, _, cx| {
                                    this.prev_button.is_hovered = *hovered;
                                    cx.notify();
                                }))
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        this.prev_button.is_pressed = true;
                                        cx.notify();
                                    }),
                                )
                                .on_mouse_up(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        this.prev_button.is_pressed = false;
                                        this.current_section = prev;
                                        cx.notify();
                                    }),
                                )
                                .on_mouse_up_out(
                                    gpui::MouseButton::Left,
                                    cx.listener(|this, _, _, cx| {
                                        this.prev_button.is_pressed = false;
                                        cx.notify();
                                    }),
                                )
                                .child(rendered),
                        )
                    })
                    .when(prev_sec.is_none(), |d| d.child(div().size_0())),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(pal.on_surface_variant)
                    .child(format!(
                        "Section {} / 7 • {}",
                        self.current_section.index(),
                        self.current_section.name()
                    )),
            )
            .child(
                div()
                    .when_some(next_sec, |d, next| {
                        let mut btn = self.next_button.clone();
                        btn.label = format!("Next: {}", next.short_name()).into();
                        let rendered = btn.render(pal);
                        d.child(
                            div()
                                .id("footer-next-btn")
                                .on_hover(cx.listener(|this, hovered: &bool, _, cx| {
                                    this.next_button.is_hovered = *hovered;
                                    cx.notify();
                                }))
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        this.next_button.is_pressed = true;
                                        cx.notify();
                                    }),
                                )
                                .on_mouse_up(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        this.next_button.is_pressed = false;
                                        this.current_section = next;
                                        cx.notify();
                                    }),
                                )
                                .on_mouse_up_out(
                                    gpui::MouseButton::Left,
                                    cx.listener(|this, _, _, cx| {
                                        this.next_button.is_pressed = false;
                                        cx.notify();
                                    }),
                                )
                                .child(rendered),
                        )
                    })
                    .when(next_sec.is_none(), |d| d.child(div().size_0())),
            )
    }
}

impl Render for Showcase {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pal = if self.dark_theme {
            dark_scheme()
        } else {
            light_scheme()
        };

        // Advance active section's springs with fixed DT (matching showcase.rs.bak).
        // In GPUI, mouse move events trigger immediate renders (often 200Hz+).
        // If springs were stepped with raw wall-clock elapsed time (dt ~ 1ms),
        // the velocity displacement would be starved, causing sluggish dragging.
        // Stepping by DT = 1.0 / 60.0 keeps drag tracking snappy and responsive.
        self.prev_button.advance_springs(window, DT);
        self.next_button.advance_springs(window, DT);

        match self.current_section {
            ShowcaseSection::All => {
                self.advance_shapes_springs(window, DT);
                self.advance_sliders_springs(window, DT);
                self.advance_switches_springs(window, DT);
                self.advance_split_buttons_springs(window, DT);
                self.advance_buttons_springs(window, DT);
                self.advance_cards_springs(window, DT);
                self.advance_menus_springs(window, DT);
                self.advance_loading(window, DT);
            }
            ShowcaseSection::Shapes => {
                self.advance_shapes_springs(window, DT);
                self.advance_loading(window, DT);
            }
            ShowcaseSection::Sliders => self.advance_sliders_springs(window, DT),
            ShowcaseSection::Switches => self.advance_switches_springs(window, DT),
            ShowcaseSection::SplitButtons => self.advance_split_buttons_springs(window, DT),
            ShowcaseSection::Buttons => self.advance_buttons_springs(window, DT),
            ShowcaseSection::Cards => self.advance_cards_springs(window, DT),
            ShowcaseSection::Menus => self.advance_menus_springs(window, DT),
        }

        div()
            .id("showcase-scroll")
            .flex()
            .flex_col()
            .gap_6()
            .p_8()
            .size_full()
            .overflow_y_scroll()
            .bg(pal.surface)
            .text_color(pal.on_surface)
            .child(self.render_header(&pal, cx))
            .child(self.render_navigation_bar(&pal, cx))
            .child(match self.current_section {
                ShowcaseSection::All => div()
                    .flex()
                    .flex_col()
                    .gap_10()
                    .child(self.render_sliders_section(&pal, cx))
                    .child(self.render_switches_section(&pal, cx))
                    .child(self.render_split_buttons_section(&pal, cx))
                    .child(self.render_buttons_section(&pal, cx))
                    .child(self.render_cards_section(&pal, cx))
                    .child(self.render_menus_section(&pal, cx))
                    .child(self.render_shapes_section(&pal, cx))
                    .into_any_element(),
                ShowcaseSection::Shapes => self.render_shapes_section(&pal, cx).into_any_element(),
                ShowcaseSection::Sliders => {
                    self.render_sliders_section(&pal, cx).into_any_element()
                }
                ShowcaseSection::Switches => {
                    self.render_switches_section(&pal, cx).into_any_element()
                }
                ShowcaseSection::SplitButtons => self
                    .render_split_buttons_section(&pal, cx)
                    .into_any_element(),
                ShowcaseSection::Buttons => {
                    self.render_buttons_section(&pal, cx).into_any_element()
                }
                ShowcaseSection::Cards => self.render_cards_section(&pal, cx).into_any_element(),
                ShowcaseSection::Menus => self.render_menus_section(&pal, cx).into_any_element(),
            })
            .child(self.render_section_footer(&pal, cx))
    }
}

fn main() {
    // Temporary probe: GPUI_M3E_NO_A11Y=1 boots with AccessKit forcibly
    // disabled so we can measure what the a11y stack costs resident.
    let app = if std::env::var("GPUI_M3E_NO_A11Y").is_ok() {
        Application::new_inaccessible(gpui_platform::current_platform(false))
    } else {
        application()
    };
    app.run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(960.0), px(960.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| Showcase {
                    dark_theme: true,
                    current_section: ShowcaseSection::Sliders,
                    shapes: ShapePlaygroundModel::default(),
                    sliders: init_sliders(),
                    switches: init_switches(),
                    split_buttons: init_split_buttons(),
                    buttons: init_buttons(),
                    cards: init_cards(),
                    menus: init_menus(),
                    prev_button: ButtonModel::new(9901, "Previous")
                        .size(ButtonSize::Small)
                        .variant(ButtonVariant::FilledTonal)
                        .icon(M3Icon::ArrowBack),
                    next_button: ButtonModel::new(9902, "Next")
                        .size(ButtonSize::Small)
                        .variant(ButtonVariant::FilledTonal)
                        .trailing_icon(M3Icon::ArrowForward),
                })
            },
        )
        .expect("open window");
        cx.activate(true);
    });
}
