use crate::shell;
use requiem_core::logo_video;
use requiem_core::rest;
use requiem_ui::components::ButtonVariant;
use requiem_ui::components::TextVariant;
use requiem_ui::components::{Align, StackProps};
use requiem_ui::font_system::TextFonts;
use requiem_ui::font_system;
use requiem_ui::theme;
use std::cell::RefCell;
use std::panic;
use std::process;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::runtime::Runtime;
use tokio::task;

use requiem_core::assets::{self, AssetsConfig, Progress, Stage};
use requiem_core::logo_video::LogoFrames;
use requiem_core::ui::logo::{Logo, LogoWidget};
use requiem_core::ui::{Fonts, TEXT_DEFAULT, TEXT_SECONDARY, TEXT_STRONG};
use requiem_ui::app::App;
use requiem_ui::app::{WindowAction, request_window_action};
use requiem_ui::components::{ButtonProps, TextProps, div};
use requiem_ui::control::section;
use requiem_ui::event::{EventResult, MouseButton, UiEvent};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};
use requiem_ui::style::{ComputedStyle, cascade};
use requiem_ui::widget::primitives::BoxWidget;
use requiem_ui::widget::{AbsoluteRect, Widget, WidgetTree};
use skia_safe::Canvas;
use skia_safe::font_style::Weight;
use winit::event_loop::EventLoopProxy;

use crate::shell::classes::splash as cls;

pub const FLAG: &str = "--assets-splash";

pub const EXIT_READY: i32 = 0;
pub const EXIT_QUIT: i32 = 1;
pub const EXIT_CONFIG: i32 = 2;
pub const EXIT_CRASH: i32 = 3;

const WINDOW_SIZE: (f32, f32) = (300.0, 350.0);

const LOGO_PX: u32 = 256;

pub enum SplashEvent {
    Logo(LogoFrames),
    Progress(Progress),

    Connecting,
    Failed(String),
    Done,
}

struct DragRoot;

impl Widget for DragRoot {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        BoxWidget.paint(canvas, rect, style);
    }

    fn handle_event(&mut self, event: &UiEvent, _rect: AbsoluteRect) -> EventResult {
        if let UiEvent::MouseDown {
            button: MouseButton::Left,
            ..
        } = event
        {
            request_window_action(WindowAction::Drag);
            return EventResult::Handled;
        }
        EventResult::Ignored
    }
}

#[derive(Clone)]
enum Phase {
    Working { label: String, fraction: f32 },
    Failed { message: String },
}

struct State {
    phase: Phase,
    logo: Option<Rc<Logo>>,
}

#[derive(Clone)]
struct Actions {
    retry: Rc<dyn Fn()>,
    quit: Rc<dyn Fn()>,
}

fn progress_phase(progress: Progress) -> Phase {
    let part = |done: usize, total: usize| {
        if total == 0 {
            1.0
        } else {
            done as f32 / total as f32
        }
    };

    let overall = match progress.stage {
        Stage::Manifest => 0.02,
        Stage::Chunks => 0.02 + 0.68 * part(progress.done, progress.total),
        Stage::Files => 0.70 + 0.18 * part(progress.done, progress.total),
        Stage::Fonts => 0.88 + 0.12 * part(progress.done, progress.total),
    };
    let label = match progress.stage {
        Stage::Manifest => requiem_i18n::t!("splash-starting"),
        Stage::Chunks | Stage::Files => {
            requiem_i18n::t!(
                "splash-downloading-assets",
                percent = (overall * 100.0).round()
            )
        }
        Stage::Fonts => requiem_i18n::t!("splash-preparing-fonts").to_string(),
    };
    Phase::Working {
        label,
        fraction: overall,
    }
}

fn build(state: &State, actions: &Actions, _config: &AssetsConfig) -> Node {
    let mut card = div().class(cls::CARD);
    match &state.phase {
        Phase::Working { label, fraction } => {
            if let Some(logo) = &state.logo {
                card = card.child(
                    Node::new(
                        "div",
                        LogoWidget {
                            logo: Some(logo.clone()),
                        },
                    )
                    .class(cls::LOGO),
                );
            }
            card = card.child(
                StackProps::vertical().align(Align::Center).build().child(
                    TextProps::styled(label.clone(), TextVariant::TEXT_SM_MEDIUM)
                        .line_height(((14.0_f32) * 1.35_f32).round())
                        .color(TEXT_SECONDARY)
                        .center(true)
                        .build(),
                ),
            );
            let percent = (fraction.clamp(0.0, 1.0) * 100.0).round();
            card = card.child(
                div().class(cls::BAR).child(
                    div()
                        .class(cls::BAR_FILL)
                        .style(format!("width: {percent}%;")),
                ),
            );
        }
        Phase::Failed { message } => {
            card = card
                .child(
                    div()
                        .class(cls::TEXT_BLOCK)
                        .child(
                            TextProps::styled(
                                requiem_i18n::t!("splash-couldn-t-start-discord"),
                                TextVariant::TEXT_MD_SEMIBOLD,
                            )
                            .line_height(((16.0_f32) * 1.35_f32).round())
                            .color(TEXT_STRONG)
                            .center(true)
                            .build(),
                        )
                        .child(
                            TextProps::new(
                                requiem_i18n::t!("splash-its-icons-and-fonts-couldn-t"),
                                13.0_f32,
                            )
                            .line_height(((13.0_f32) * 1.35_f32).round())
                            .weight(Weight::NORMAL)
                            .color(TEXT_DEFAULT)
                            .center(true)
                            .build(),
                        )
                        .child(
                            TextProps::new(message.clone(), 11.0_f32)
                                .line_height(((11.0_f32) * 1.35_f32).round())
                                .weight(Weight::NORMAL)
                                .color(TEXT_SECONDARY)
                                .center(true)
                                .build(),
                        ),
                )
                .child(
                    div()
                        .class(cls::ACTIONS)
                        .child(
                            ButtonProps::default()
                                .variant(ButtonVariant::Primary)
                                .label(requiem_i18n::t!("splash-retry").to_string())
                                .on_click({
                                    let retry = actions.retry.clone();
                                    move || retry()
                                })
                                .build(),
                        )
                        .child(
                            ButtonProps::default()
                                .variant(ButtonVariant::Secondary)
                                .label(requiem_i18n::t!("splash-quit").to_string())
                                .on_click({
                                    let quit = actions.quit.clone();
                                    move || quit()
                                })
                                .build(),
                        ),
                );
        }
    }
    card
}

async fn first_stage(
    http: &wreq::Client,
    config: &AssetsConfig,
) -> Result<
    (
        requiem_asset_store::Manifest,
        requiem_asset_store::ManifestSource,
        Option<LogoFrames>,
    ),
    String,
> {
    let (manifest, source) = requiem_asset_store::fetch_manifest(http, &config.store_config())
        .await
        .map_err(|e| e.to_string())?;
    let logo = load_logo(http, config, &manifest).await;
    Ok((manifest, source, logo))
}

async fn load_logo(
    http: &wreq::Client,
    config: &AssetsConfig,
    manifest: &requiem_asset_store::Manifest,
) -> Option<LogoFrames> {
    let path = match requiem_asset_store::fetch_file(
        http,
        &config.cache_dir,
        manifest,
        assets::LOGO_ASSET,
    )
    .await
    {
        Ok(path) => path,
        Err(e) => {
            log::warn!("splash: no logo animation: {e}");
            return None;
        }
    };
    let decoded = task::spawn_blocking(move || {
        logo_video::decode_logo_video(&path, LOGO_PX)
    })
    .await
    .ok()?;
    match decoded {
        Ok(logo) => Some(logo),
        Err(e) => {
            log::warn!("splash: could not decode the logo animation: {e}");
            None
        }
    }
}

async fn second_stage(
    http: wreq::Client,
    config: AssetsConfig,
    manifest: requiem_asset_store::Manifest,
    source: requiem_asset_store::ManifestSource,
    send: Arc<dyn Fn(SplashEvent) + Send + Sync>,
) {
    let progress = send.clone();
    match assets::prefetch(&http, &config, manifest, source, move |p| {
        progress(SplashEvent::Progress(p))
    })
    .await
    {
        Ok(()) => send(SplashEvent::Done),
        Err(e) => send(SplashEvent::Failed(e.to_string())),
    }
}

async fn attempt(
    http: wreq::Client,
    config: AssetsConfig,
    send: Arc<dyn Fn(SplashEvent) + Send + Sync>,
) {
    send(SplashEvent::Connecting);
    match first_stage(&http, &config).await {
        Ok((manifest, source, logo)) => {
            if let Some(logo) = logo {
                send(SplashEvent::Logo(logo));
            }
            second_stage(http, config, manifest, source, send).await;
        }
        Err(message) => send(SplashEvent::Failed(message)),
    }
}

pub fn run(config: AssetsConfig) -> i32 {
    panic::set_hook(Box::new(|info| {
        log::error!("splash crashed: {info}");
        process::exit(EXIT_CRASH);
    }));
    let runtime = Runtime::new().expect("failed to start tokio runtime");
    let http = match rest::build_client() {
        Ok(http) => http,
        Err(e) => {
            log::error!("splash: {e}");
            return EXIT_CONFIG;
        }
    };

    let first = runtime.block_on(first_stage(&http, &config));

    let fonts = Fonts {
        typeface: theme::make_typeface(Weight::MEDIUM),
        text_fonts: TextFonts::new(2 * 1024 * 1024),
    };
    font_system::install_ambient(fonts.text_fonts.clone());

    let (state, pending) = match first {
        Ok((manifest, source, logo)) => (
            State {
                phase: Phase::Working {
                    label: requiem_i18n::t!("splash-starting"),
                    fraction: 0.02,
                },
                logo: logo.and_then(Logo::from_frames).map(Rc::new),
            },
            Some((manifest, source)),
        ),
        Err(message) => (
            State {
                phase: Phase::Failed { message },
                logo: None,
            },
            None,
        ),
    };
    let state = Rc::new(RefCell::new(state));
    let tick: Signal<u64> = signal(0);

    let proxy_slot: Arc<Mutex<Option<EventLoopProxy<SplashEvent>>>> =
        Arc::new(Mutex::new(None));
    let send: Arc<dyn Fn(SplashEvent) + Send + Sync> = {
        let proxy_slot = proxy_slot.clone();
        Arc::new(move |event| {
            if let Some(proxy) = proxy_slot.lock().expect("proxy slot poisoned").as_ref() {
                let _ = proxy.send_event(event);
            }
        })
    };

    let actions = Actions {
        retry: {
            let (handle, http, config, send) = (
                runtime.handle().clone(),
                http.clone(),
                config.clone(),
                send.clone(),
            );
            Rc::new(move || {
                handle.spawn(attempt(http.clone(), config.clone(), send.clone()));
            })
        },
        quit: Rc::new(|| process::exit(EXIT_QUIT)),
    };

    let mut tree = WidgetTree::new(Box::new(DragRoot), ComputedStyle::default());
    let root = tree.root;
    tree.nodes[root].classes.push(cls::ROOT.to_string());
    let effect = {
        let (state, tick, actions, config) =
            (state.clone(), tick.clone(), actions.clone(), config.clone());
        section(&mut tree, root, move || {
            tick.get();
            build(&state.borrow(), &actions, &config)
        })
    };
    cascade::apply_stylesheet(&mut tree, &shell::load_theme());

    let on_event = {
        let (state, tick) = (state.clone(), tick.clone());
        move |event: SplashEvent, _tree: &mut WidgetTree| {
            {
                let mut state = state.borrow_mut();
                match event {
                    SplashEvent::Logo(frames) => {
                        state.logo = Logo::from_frames(frames).map(Rc::new)
                    }
                    SplashEvent::Progress(progress) => state.phase = progress_phase(progress),
                    SplashEvent::Connecting => {
                        state.phase = Phase::Working {
                            label: requiem_i18n::t!("splash-starting"),
                            fraction: 0.02,
                        }
                    }
                    SplashEvent::Failed(message) => state.phase = Phase::Failed { message },
                    SplashEvent::Done => process::exit(EXIT_READY),
                }
            }
            tick.update(|t| *t += 1);
        }
    };
    let app = App::new(&requiem_i18n::t!("splash-discord"), tree, on_event)
        .with_fixed_window_size(WINDOW_SIZE.0, WINDOW_SIZE.1)
        .with_frameless_window();
    *proxy_slot.lock().expect("proxy slot poisoned") = Some(app.proxy());

    if let Some((manifest, source)) = pending {
        runtime.spawn(second_stage(
            http.clone(),
            config.clone(),
            manifest,
            source,
            send.clone(),
        ));
    }

    let result = app.run();
    drop(effect);
    if let Err(e) = result {
        log::error!("splash: event loop failed: {e}");
    }

    EXIT_QUIT
}
