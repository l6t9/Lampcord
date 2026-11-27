use crate::ui::classes::loading_screen;
use crate::ui::classes::loading_screen as css;
use crate::ui::emoji_button_icon;
use crate::ui::logo::Logo;
use crate::ui::logo;
use crate::utils::link as utils_link;
use requiem_ui::components::ButtonProps;
use requiem_ui::components::ButtonVariant;
use requiem_ui::components::TextVariant;
use requiem_ui::components::{Align, StackProps};
use requiem_ui::components::{MessageTextProps, icon_rotated, message_text};
use requiem_ui::icons::IconPath;
use requiem_ui::timer;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use requiem_ui::components::{TextProps, div, icon};
use requiem_ui::node::Node;
use requiem_ui::signal::{Effect, Signal, create_effect, signal};
use requiem_ui::widget::{WidgetId, WidgetTree};

use super::loading_tips::Piece;
use crate::ui::{TEXT_DEFAULT, TEXT_STRONG};
use requiem_ui::icons::{SCREEN_ARROW, USER_CIRCLE_STATUS, X_NEUTRAL};

const CONTAINER_FADE: Duration = Duration::from_millis(200);

const PROBLEMS_AFTER: Duration = Duration::from_secs(10);
const TWITTER_SUPPORT_URL: &str = "https://twitter.com/discord_support";
const STATUS_URL: &str = "https://discordstatus.com";

const TEXT_LINK: skia_safe::Color = skia_safe::Color::from_argb(255, 0x4d, 0x96, 0xee);

const LOGO_PX: u32 = 256;

fn random_tip() -> Vec<Piece> {
    let tips = super::loading_tips::tips();
    let index = emoji_button_icon::random_u32() as usize % tips.len();
    tips.into_iter().nth(index).unwrap_or_default()
}

fn key_label(key: &str) -> Option<String> {
    let mac = cfg!(target_os = "macos");
    Some(match key {
        "mod" => if mac { "\u{2318}" } else { "ctrl" }.to_string(),
        "ctrl" => if mac { "\u{2303}" } else { "ctrl" }.to_string(),
        "cmd" => "cmd".to_string(),
        "alt" => if mac { "\u{2325}" } else { "alt" }.to_string(),
        "shift" => if mac { "\u{21E7}" } else { "shift" }.to_string(),
        "return" | "enter" => if mac { "\u{23CE}" } else { "enter" }.to_string(),
        "esc" | "escape" => "esc".to_string(),
        "backspace" => if mac { "\u{232B}" } else { "backspace" }.to_string(),
        "up" | "down" => return None,
        other => other.to_string(),
    })
}

fn keybind(shortcut: &str) -> Node {
    let keys: Vec<Node> = shortcut
        .split('+')
        .map(|key| {
            let content = match key_label(key) {
                Some(label) => {
                    TextProps::styled(label.to_uppercase(), TextVariant::TEXT_XS_SEMIBOLD)
                        .line_height(12.0_f32)
                        .color(TEXT_STRONG)
                        .build()
                }
                None => icon_rotated(
                    SCREEN_ARROW,
                    TEXT_STRONG,
                    if key == "up" { 180.0 } else { 0.0 },
                )
                .class(css::KEYCAP_ARROW),
            };

            div()
                .class(css::KEYCAP)
                .child(div().class(css::KEYCAP_BAND))
                .child(content)
        })
        .collect();
    div().class(css::KEYBIND).children(keys)
}

fn tip_node(pieces: &[Piece]) -> Node {
    let word = |text: &str| {
        TextProps::styled(text.to_string(), TextVariant::TEXT_MD_MEDIUM)
            .color(TEXT_DEFAULT)
            .build()
    };
    let styled = |markdown: String| {
        message_text(MessageTextProps {
            content: markdown,
            size: 16.0,
            color: TEXT_DEFAULT,
            ..Default::default()
        })
    };
    let mut nodes: Vec<Node> = Vec::new();

    let mut glue_next = false;
    for piece in pieces {
        match piece {
            Piece::Text(text) => {
                let mut words = text
                    .split_whitespace()
                    .map(word)
                    .collect::<Vec<_>>()
                    .into_iter();
                if glue_next && !text.starts_with(char::is_whitespace) {
                    if let (Some(first), Some(previous)) = (words.next(), nodes.pop()) {
                        nodes.push(
                            StackProps::horizontal()
                                .align(Align::Center)
                                .gap(0)
                                .full_width(false)
                                .build()
                                .child(previous)
                                .child(first),
                        );
                    }
                }
                nodes.extend(words);
                glue_next = !text.ends_with(char::is_whitespace);
            }
            Piece::Bold(text) => {
                nodes.extend(text.split_whitespace().map(|w| styled(format!("**{w}**"))));
                glue_next = true;
            }
            Piece::Strike(text) => {
                nodes.extend(text.split_whitespace().map(|w| styled(format!("~~{w}~~"))));
                glue_next = true;
            }
            Piece::Keys(shortcut) => {
                nodes.push(keybind(shortcut));
                glue_next = true;
            }
        }
    }
    div().class(css::LOADING_TIP).children(nodes)
}

fn link(
    label: &str,
    icon_path: impl IconPath,
    url: &'static str,
    extra_class: Option<&'static str>,
) -> Node {
    let text = TextProps::styled(label.to_string(), TextVariant::TEXT_MD_MEDIUM)
        .line_height(16.0_f32)
        .color(TEXT_LINK)
        .build();
    let mut button = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![css::LOADING_LINK.to_string()])
        .on_click(move || utils_link::open_url(url))
        .build()
        .child(icon(icon_path, TEXT_LINK).class(css::LOADING_LINK_ICON))
        .child(text);
    if let Some(class) = extra_class {
        button = button.class(class);
    }
    button
}

fn build(fading: Signal<bool>, problems: Signal<bool>, tip: &[Piece]) -> Node {
    let title = TextProps::styled(
        requiem_i18n::t!("loading-screen-did-you-know").to_string(),
        TextVariant::TEXT_XS_SEMIBOLD,
    )
    .color(TEXT_STRONG)
    .center(true)
    .build()
    .class(css::LOADING_TIP_TITLE);
    let tip = tip_node(tip);
    let spinner = Node::new(
        "div",
        logo::LogoWidget {
            logo: Logo::from_store(LOGO_PX).map(Rc::new),
        },
    )
    .class(css::LOADING_SPINNER)
    .bind_class(fading.clone(), css::LOADING_SPINNER_FADE, "");
    let text = div()
        .class(css::LOADING_TEXT)
        .bind_class(fading.clone(), css::LOADING_TEXT_FADE, "")
        .child(title)
        .child(tip);

    let problems_text = TextProps::styled(
        requiem_i18n::t!("loading-screen-connection-problems-let-us-know").to_string(),
        TextVariant::TEXT_SM_MEDIUM,
    )
    .color(TEXT_DEFAULT)
    .center(true)
    .build()
    .class(css::LOADING_PROBLEMS_TEXT);
    let links = StackProps::horizontal()
        .align(Align::Center)
        .gap(0)
        .full_width(false)
        .build()
        .child(link(
            &requiem_i18n::t!("loading-screen-tweet-us"),
            X_NEUTRAL,
            TWITTER_SUPPORT_URL,
            None,
        ))
        .child(link(
            &requiem_i18n::t!("loading-screen-server-status"),
            USER_CIRCLE_STATUS,
            STATUS_URL,
            Some(css::LOADING_LINK_STATUS),
        ));
    let problems_box = div()
        .class(css::LOADING_PROBLEMS)
        .bind_class(problems, css::LOADING_PROBLEMS_IN, "")
        .child(problems_text)
        .child(links);

    div()
        .class(css::LOADING_SCREEN)
        .bind_class(fading, css::LOADING_SCREEN_FADE, "")
        .child(div().class(css::LOADING_CONTENT).child(spinner).child(text))
        .child(problems_box)
}

pub fn mount(tree: &mut WidgetTree, parent: WidgetId, ready: &Signal<bool>) -> Vec<Effect> {
    let fading = signal(ready.peek());
    let gone = signal(ready.peek());
    let problems = signal(false);
    let problems_timer: Rc<Cell<Option<timer::TimeoutHandle>>> =
        Rc::new(Cell::new(None));

    let effects = vec![create_effect(
        {
            let (ready, fading, gone, problems, problems_timer) = (
                ready.clone(),
                fading.clone(),
                gone.clone(),
                problems.clone(),
                problems_timer.clone(),
            );
            move |_tree| {
                if ready.get() {
                    if !fading.peek() {
                        fading.set(true);
                        if let Some(handle) = problems_timer.take() {
                            timer::clear_timeout(handle);
                        }
                        let (ready, gone) = (ready.clone(), gone.clone());
                        timer::set_timeout(CONTAINER_FADE, move || {
                            if ready.peek() {
                                gone.set(true);
                            }
                        });
                    }
                } else {
                    fading.set(false);
                    gone.set(false);
                    problems.set(false);
                    if let Some(handle) = problems_timer.take() {
                        timer::clear_timeout(handle);
                    }
                    let (ready, problems) = (ready.clone(), problems.clone());
                    problems_timer.set(Some(timer::set_timeout(
                        PROBLEMS_AFTER,
                        move || {
                            if !ready.peek() {
                                problems.set(true);
                            }
                        },
                    )));
                }
            }
        },
        tree,
    )];

    let tip = random_tip();
    let host_gone = gone.clone();
    let node = Node::reactive(move || {
        if gone.get() {
            return div();
        }
        build(fading.clone(), problems.clone(), &tip)
    });

    tree.mount(
        parent,
        node.class(loading_screen::LOADING_HOST)
            .bind_class(
                host_gone,
                loading_screen::LOADING_HOST_GONE,
                "",
            ),
    );
    effects
}

#[cfg(test)]
mod tests {

    use crate::test_assets;
    use crate::ui;
    use requiem_ui::components::ButtonProps;
    use requiem_ui::event::CursorIcon;
    use requiem_ui::font_system::TextFonts;
    use requiem_ui::layout;
    use requiem_ui::paint;
    use requiem_ui::signal as requiem_ui_signal;
    use requiem_ui::style::ComputedStyle;
    use requiem_ui::style::cascade;
    use requiem_ui::style::parser;
    use requiem_ui::widget::primitives::BoxWidget;
    use skia_safe::surfaces;
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::thread;
    use taffy::{AvailableSpace, Size};

    use super::*;

    fn install_fonts() {
        TextFonts::install_new(16 * 1024 * 1024);
    }

    #[test]
    fn renders_the_loading_screen() {
        let Ok(out) = env::var("LOADING_PREVIEW_DIR") else {
            return;
        };
        install_fonts();

        let _ = test_assets::install();
        let ready = signal(false);
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let root = tree.root;
        tree.nodes[root].classes.push("root".to_string());
        let _effects = mount(&mut tree, root, &ready);
        let theme = parser::parse_stylesheet(&format!(
            "{}\n{}\n.root {{ position: relative; width: 1400px; height: 900px; }}",
            requiem_ui::BUNDLED_CSS,
            ui::THEME_CSS
        ));
        cascade::apply_stylesheet(&mut tree, &theme);
        for (name, problems) in [("loading.png", false), ("loading_problems.png", true)] {
            if problems {
                for id in tree.nodes.keys().collect::<Vec<_>>() {
                    if tree.nodes[id]
                        .classes
                        .iter()
                        .any(|c| c.contains("loading-problems") && !c.contains("text"))
                    {
                        tree.nodes[id].classes.push(
                            loading_screen::LOADING_PROBLEMS_IN.to_string(),
                        );
                    }
                }
                cascade::apply_stylesheet(&mut tree, &theme);
            }
            layout::compute_layout(
                &mut tree,
                Size {
                    width: AvailableSpace::Definite(1400.0),
                    height: AvailableSpace::Definite(900.0),
                },
            );
            let mut warmup = surfaces::raster_n32_premul((1400, 900)).unwrap();

            paint::paint(&mut tree, warmup.canvas());
            thread::sleep(Duration::from_millis(300));
            let mut surface = surfaces::raster_n32_premul((1400, 900)).unwrap();
            paint::paint(&mut tree, surface.canvas());
            let png = surface
                .image_snapshot()
                .encode(None, skia_safe::EncodedImageFormat::PNG, 100)
                .unwrap();
            fs::write(Path::new(&out).join(name), png.as_bytes()).unwrap();
        }
    }

    #[test]
    fn renders_rich_tips() {
        let Ok(out) = env::var("LOADING_PREVIEW_DIR") else {
            return;
        };
        install_fonts();
        let tips = super::super::loading_tips::tips();
        let rich: Vec<&Vec<Piece>> = tips
            .iter()
            .filter(|t| t.iter().any(|p| !matches!(p, Piece::Text(_))))
            .collect();
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let root = tree.root;
        tree.nodes[root].classes.push("root".to_string());
        let column = tree.mount(
            root,
            div().style("display: flex; flex-direction: column; gap: 14px; padding: 16px"),
        );
        for tip in &rich {
            tree.mount(column, tip_node(tip));
        }
        let theme = parser::parse_stylesheet(&format!(
            "{}\n{}\n.root {{ position: relative; width: 340px; height: 1500px; background-color: #121214; }}",
            requiem_ui::BUNDLED_CSS,
            ui::THEME_CSS
        ));
        cascade::apply_stylesheet(&mut tree, &theme);
        layout::compute_layout(
            &mut tree,
            Size {
                width: AvailableSpace::Definite(340.0),
                height: AvailableSpace::Definite(1500.0),
            },
        );
        let mut surface = surfaces::raster_n32_premul((340, 1500)).unwrap();
        paint::paint(&mut tree, surface.canvas());
        let png = surface
            .image_snapshot()
            .encode(None, skia_safe::EncodedImageFormat::PNG, 100)
            .unwrap();
        fs::write(Path::new(&out).join("tips.png"), png.as_bytes()).unwrap();
    }

    fn screen_classes(tree: &WidgetTree) -> Option<Vec<String>> {
        tree.nodes.iter().map(|(_, n)| n.classes.clone()).find(|c| {
            c.iter()
                .any(|c| c.starts_with("loading_screen-loading-screen_"))
        })
    }

    #[test]
    fn fades_on_ready_and_returns_when_the_connection_resets() {
        install_fonts();
        let ready = signal(false);
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let root = tree.root;
        let _effects = mount(&mut tree, root, &ready);
        let fading = |tree: &WidgetTree| {
            screen_classes(tree)
                .expect("screen mounted")
                .iter()
                .any(|c| c.starts_with("loading_screen-loading-screen-fade_"))
        };

        assert!(screen_classes(&tree).is_some());
        assert!(!fading(&tree), "visible, not fading, before READY");
        ready.set(true);
        requiem_ui_signal::flush(&mut tree);
        assert!(fading(&tree), "READY starts the fade-out");

        ready.set(false);
        requiem_ui_signal::flush(&mut tree);
        assert!(!fading(&tree), "reconnecting shows the screen again");
    }

    #[test]
    fn a_finished_loading_screen_leaves_cursors_and_hover_alone() {
        let ready = signal(true);
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let root = tree.root;
        tree.nodes[root].classes.push("root".to_string());

        let button = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .on_click(|| {})
            .build()
            .style("width: 400px; height: 300px");
        tree.mount(root, button);
        let shifty = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .on_click_shift(|_| {})
            .build()
            .style("width: 400px; height: 40px");
        tree.mount(root, shifty);
        let _effects = mount(&mut tree, root, &ready);
        let theme = parser::parse_stylesheet(&format!(
            "{}\n{}\n.root {{ position: relative; width: 400px; height: 400px; }}",
            requiem_ui::BUNDLED_CSS,
            ui::THEME_CSS
        ));
        cascade::apply_stylesheet(&mut tree, &theme);
        layout::compute_layout(
            &mut tree,
            Size {
                width: AvailableSpace::Definite(400.0),
                height: AvailableSpace::Definite(400.0),
            },
        );
        assert_eq!(
            paint::cursor_at(&tree, 100.0, 100.0),
            CursorIcon::Pointer,
            "over a plain button"
        );
        assert_eq!(
            paint::cursor_at(&tree, 300.0, 20.0),
            CursorIcon::Pointer,
            "over a shift-click button (emoji tiles)"
        );
        assert_eq!(
            paint::cursor_at(&tree, 300.0, 200.0),
            CursorIcon::Default,
            "over nothing"
        );
    }

    #[test]
    fn mouse_reaches_the_app_after_the_fade_finishes() {
        install_fonts();
        let ready = signal(false);
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let root = tree.root;
        tree.nodes[root].classes.push("root".to_string());
        tree.mount(
            root,
            ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .on_click(|| {})
                .build()
                .style("width: 400px; height: 300px"),
        );
        let _effects = mount(&mut tree, root, &ready);
        let theme = parser::parse_stylesheet(&format!(
            "{}\n{}\n.root {{ position: relative; width: 400px; height: 400px; }}",
            requiem_ui::BUNDLED_CSS,
            ui::THEME_CSS
        ));
        cascade::apply_stylesheet(&mut tree, &theme);
        let size = Size {
            width: AvailableSpace::Definite(400.0),
            height: AvailableSpace::Definite(400.0),
        };
        layout::compute_layout(&mut tree, size);
        assert_eq!(
            paint::cursor_at(&tree, 100.0, 100.0),
            CursorIcon::Default,
            "the screen covers the app while loading"
        );

        ready.set(true);
        requiem_ui_signal::flush(&mut tree);
        thread::sleep(Duration::from_millis(250));
        assert!(timer::run_due());
        requiem_ui_signal::flush(&mut tree);

        cascade::apply_stylesheet(&mut tree, &theme);
        layout::compute_layout(&mut tree, size);
        assert_eq!(
            paint::cursor_at(&tree, 100.0, 100.0),
            CursorIcon::Pointer,
            "the app gets the mouse once the screen is gone"
        );
    }
}
