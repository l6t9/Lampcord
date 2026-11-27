use crate::ui::classes::playground as css;
use requiem_ui::colors::{TEXT_DEFAULT, TEXT_SECONDARY, TEXT_SUBTLE};
use requiem_ui::components::TextLinkVariant;
use requiem_ui::components::{Align, StackProps};
use requiem_ui::components::{TextLinkProps, TextProps, div, icon, scroll_view, search_bar, text};
use requiem_ui::control::section;
use requiem_ui::node::Node;
use requiem_ui::signal::{Effect, Signal, signal};
use requiem_ui::widget::primitives::{ButtonWidget, ClickAreaWidget};
use requiem_ui::widget::{WidgetId, WidgetTree};
use skia_safe::Color;
use skia_safe::font_style::Weight;

use super::playground_stories::{self as stories, Story};
use crate::ui::TEXT_STRONG;
use requiem_ui::icons::{CHEVRON_SMALL_RIGHT, COPY, SETTINGS, X_SMALL};

pub struct Playground {
    pub name: &'static str,

    pub short: &'static str,
}

pub const PLAYGROUNDS: [Playground; 16] = [
    Playground {
        name: "Mana",
        short: "M",
    },
    Playground {
        name: "Void",
        short: "V",
    },
    Playground {
        name: "Payments",
        short: "P",
    },
    Playground {
        name: "Virtual Currency",
        short: "VC",
    },
    Playground {
        name: "Nitro",
        short: "N",
    },
    Playground {
        name: "Identity",
        short: "I",
    },
    Playground {
        name: "MFA",
        short: "M",
    },
    Playground {
        name: "Layout System",
        short: "LS",
    },
    Playground {
        name: "Video Player",
        short: "VP",
    },
    Playground {
        name: "Marketing",
        short: "M",
    },
    Playground {
        name: "Slayer Storefront",
        short: "SS",
    },
    Playground {
        name: "Collectibles Shop",
        short: "CS",
    },
    Playground {
        name: "Safety",
        short: "S",
    },
    Playground {
        name: "Core",
        short: "C",
    },
    Playground {
        name: "Server Hub",
        short: "SH",
    },
    Playground {
        name: "AI Visual Identity",
        short: "AVI",
    },
];

thread_local! {

    static OPEN: Signal<Option<&'static str>> = signal(None);

    static STORY: Signal<Option<&'static str>> = signal(None);
    static SEARCH: Signal<String> = signal(String::new());
}

pub fn open(name: &str) {
    let Some(playground) = PLAYGROUNDS.iter().find(|p| p.name == name) else {
        return;
    };
    STORY.with(|s| s.set(None));
    SEARCH.with(|s| s.set(String::new()));
    OPEN.with(|o| o.set(Some(playground.name)));
}

pub fn close() {
    OPEN.with(|o| o.set(None));
}

pub fn mount(tree: &mut WidgetTree, parent: WidgetId) -> Vec<Effect> {
    vec![section(tree, parent, move || {
        match OPEN.with(Signal::get) {
            Some(name) => layer(name),
            None => div(),
        }
    })]
}

fn layer(current: &'static str) -> Node {
    let content = div()
        .class(css::CONTENT)
        .child(header(current))
        .child(page(current));

    Node::new("div", ClickAreaWidget::new())
        .class(css::LAYER)
        .child(rail(current))
        .child(content)
}

fn rail(current: &'static str) -> Node {
    let mut rail = div().class(css::RAIL);
    for playground in &PLAYGROUNDS {
        let selected = playground.name == current;
        let label = text(TextProps {
            content: playground.short.to_string(),
            size: 18.0,
            line_height: Some(21.6),
            weight: Weight::MEDIUM.into(),
            color: if selected { Color::WHITE } else { TEXT_DEFAULT },

            ..Default::default()
        });
        let name = playground.name;
        let mut item = Node::new(
            "button",
            ButtonWidget::new()
                .with_focusable(false)
                .with_on_click(move || {
                    if name != current {
                        open(name)
                    }
                }),
        )
        .class(css::RAIL_ITEM)
        .child(label);
        if selected {
            item = item.class(css::RAIL_ITEM_SELECTED);
        }
        rail = rail.child(item);
    }
    rail
}

fn heading(content: &str) -> Node {
    text(TextProps {
        content: content.to_string(),
        size: 16.0,
        weight: Weight::MEDIUM.into(),
        color: TEXT_STRONG,
        ..Default::default()
    })
}

fn header(current: &'static str) -> Node {
    let crumbs = Node::reactive(move || {
        let mut crumbs = StackProps::horizontal()
            .align(Align::Center)
            .full_width(false)
            .build()
            .child(heading(current));
        if let Some(story) = STORY.with(Signal::get) {
            crumbs = crumbs
                .child(
                    icon(CHEVRON_SMALL_RIGHT, Color::from_argb(255, 0x96, 0x97, 0x9e))
                        .class(css::CRUMB_CHEVRON),
                )
                .child(heading(story))
                .child(div().class(css::CRUMB_DOT))
                .child(
                    TextLinkProps::default()
                        .label("Docs".to_string())
                        .variant(TextLinkVariant::Anchor)
                        .on_click(|| {})
                        .build(),
                )
                .child(icon(COPY, TEXT_SECONDARY).class(css::CRUMB_COPY));
        }
        crumbs
    });
    let icons = div()
        .class(css::HEADER_ICONS)
        .child(icon(SETTINGS, TEXT_SECONDARY).class(css::HEADER_ICON))
        .child(
            Node::new(
                "button",
                ButtonWidget::new()
                    .with_focusable(false)
                    .with_on_click(close),
            )
            .child(icon(X_SMALL, TEXT_SECONDARY).class(css::HEADER_ICON)),
        );
    div().class(css::HEADER).child(crumbs).child(icons)
}

fn page(current: &'static str) -> Node {
    let sidebar = scroll_view()
        .class(css::SIDEBAR)
        .child(search_bar("Search stories...", |query| {
            let query = query.to_string();
            SEARCH.with(|s| s.set(query));
        }))
        .child(nav(current));
    let main = Node::reactive(main_view).class(css::MAIN_HOST);
    div().class(css::PAGE).child(sidebar).child(main)
}

fn nav(current: &'static str) -> Node {
    Node::reactive(move || {
        let mut nav = div().class(css::NAV);
        if current != "Mana" {
            return nav;
        }
        let query = SEARCH.with(Signal::get).to_lowercase();
        let chosen = STORY.with(Signal::get);
        let mut first = true;
        for section in stories::SECTIONS {
            let matching: Vec<&Story> = section
                .stories
                .iter()
                .filter(|s| query.is_empty() || s.name.to_lowercase().contains(&query))
                .collect();
            if matching.is_empty() {
                continue;
            }
            if !first {
                nav = nav.child(div().class(css::NAV_SEPARATOR));
            }
            let header_class = if first {
                css::NAV_HEADER_FIRST
            } else {
                css::NAV_HEADER
            };
            first = false;
            nav = nav.child(div().class(header_class).child(text(TextProps {
                content: section.title.to_uppercase(),
                size: 12.0,
                weight: Weight::BOLD.into(),
                color: TEXT_SUBTLE,

                ..Default::default()
            })));
            for story in matching {
                let selected = chosen == Some(story.name);
                let name = story.name;
                let label = text(TextProps {
                    content: name.to_string(),
                    size: 16.0,
                    weight: Weight::MEDIUM.into(),
                    color: if selected {
                        TEXT_STRONG
                    } else {
                        TEXT_SECONDARY
                    },

                    ..Default::default()
                });
                let mut item = Node::new(
                    "button",
                    ButtonWidget::new()
                        .with_focusable(false)
                        .with_on_click(move || STORY.with(|s| s.set(Some(name)))),
                )
                .class(css::NAV_ITEM)
                .child(label);
                if selected {
                    item = item.class(css::NAV_ITEM_SELECTED);
                }
                nav = nav.child(item);
            }
        }
        nav
    })
}

fn main_view() -> Node {
    let Some(name) = STORY.with(Signal::get) else {
        return div().class(css::CANVAS);
    };
    let view = stories::find(name)
        .and_then(|story| story.build)
        .map(|build| build());
    let (content, controls) = match view {
        Some(view) => (view.canvas, view.controls),
        None => (
            text(TextProps {
                content: "This story isn't available in this client yet.".to_string(),
                size: 14.0,
                color: TEXT_SUBTLE,

                ..Default::default()
            }),
            Vec::new(),
        ),
    };
    let canvas = scroll_view().class(css::CANVAS).child(
        StackProps::vertical()
            .gap(0)
            .full_width(false)
            .build()
            .child(
                div().class(css::STORY_FRAME).child(
                    StackProps::horizontal()
                        .align(Align::Start)
                        .gap(16)
                        .wrap(true)
                        .full_width(false)
                        .build()
                        .child(content),
                ),
            ),
    );
    let controls = scroll_view().class(css::CONTROLS).child(
        StackProps::vertical()
            .gap(16)
            .full_width(false)
            .build()
            .children(controls),
    );
    div().class(css::MAIN).child(canvas).child(controls)
}
