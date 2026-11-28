use requiem::shell;
use requiem_core::rest::HttpClient;
use requiem_core::rest;
use requiem_core::video::VideoCache;
use requiem_ui::app;
use requiem_ui::components::toast::ToastState;
use requiem_ui::font_system::TextFonts;
use requiem_ui::font_system;
use requiem_ui::layout as requiem_ui_layout;
use requiem_ui::paint;
use requiem_ui::signal as requiem_ui_signal;
use requiem_ui::style::cascade;
use requiem_ui::style::parser;
use requiem_ui::text_cache::MeasureCache;
use requiem_ui::theme;
use requiem_ui::timer;
use requiem_ui::widget;
use skia_safe::font_style::Weight;
use skia_safe::surfaces;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

use requiem_core::ui;
use requiem_core::auth::AuthCoordinator;
use requiem_core::config::BackendConfig;
use requiem_core::images::ImageCache;
use requiem_core::models::id::marker::ChannelMarker;
use requiem_core::models::{Channel, Guild, Id, Message, User};
use requiem_core::query::QueryClient;
use requiem_core::rest::endpoints::favorite_gifs::FavoriteGifs;
use requiem_core::state::AppState;
use requiem_core::ui::attachments::AttachmentsState;
use requiem_core::ui::emoji_picker::{EmojiPickerSignals, EmojiPickerState, PickerTab};
use requiem_core::ui::gif_picker::{GifPickerSignals, GifPickerState, GifView};
use requiem_core::ui::media_viewer::MediaViewerState;
use requiem_core::ui::profile_popout::{ProfileOverlays, ProfilePopoutState};
use requiem_core::ui::{Fonts, RootContext, RootSignals, SettingsUiState};
use requiem_ui::event::{CursorIcon, MouseButton, UiEvent};
use requiem_ui::signal::signal;
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::anim::ToggleState;
use requiem_ui::widget::primitives::BoxWidget;
use requiem_ui::widget::{WidgetId, WidgetTree};
use taffy::{AvailableSpace, Size};

fn user(id: u64, name: &str) -> User {
    User {
        id: Id::new(id),
        username: name.into(),
        discriminator: "0".into(),
        global_name: Some(name.into()),
        avatar: None,
        email: None,
        phone: None,
        verified: None,
        mfa_enabled: None,
        bot: false,
        premium_type: None,
        primary_guild: None,
        avatar_decoration_data: None,
        collectibles: None,
        display_name_styles: None,
    }
}

fn message(id: u64, author: &User, extra: serde_json::Value) -> Message {
    let mut value = serde_json::json!({
        "id": id.to_string(), "channel_id": "11", "author": serde_json::to_value(author).unwrap(),
        "content": "", "timestamp": "2026-07-17T13:00:00.000Z", "mentions": [], "mention_everyone": false,
        "embeds": [], "attachments": [],
    });
    for (k, v) in extra.as_object().unwrap() {
        value[k] = v.clone();
    }
    serde_json::from_value(value).unwrap()
}

struct App {
    tree: WidgetTree,
    viewer: MediaViewerState,
    theme: Vec<parser::StyleRule>,
    _effects: Vec<requiem_ui_signal::Effect>,
    _runtime: Runtime,
}

fn boot(messages: Vec<Message>) -> App {
    let state = AppState::new();
    state.connection.ready.set(true);
    let me = user(999, "tester");
    state.users.current.set(Some(me));
    let guild = Guild::new(Id::new(1), "g", None, None);
    state.guilds.list.set(vec![guild.clone()]);
    state.channels.by_guild.update(|m| {
        m.insert(
            guild.id,
            vec![Channel {
                id: Id::new(11),
                guild_id: Some(Id::new(1)),
                kind: 0,
                name: Some("general".into()),
                position: Some(0),
                parent_id: None,
                permission_overwrites: Vec::new(),
                nsfw: false,
                flags: 0,
                last_message_id: None,
                owner_id: None,
                thread_metadata: None,
                message_count: 0,
                member_count: 0,
                total_message_sent: 0,
                member: None,
                newly_created: false,
            }],
        );
    });
    state.set_history(Id::new(11), messages);

    let fonts = Fonts {
        typeface: theme::make_typeface(Weight::MEDIUM),
        text_fonts: TextFonts::new(16 * 1024 * 1024),
    };
    font_system::install_ambient(fonts.text_fonts.clone());
    let backend = BackendConfig {
        rest_base_url: String::new(),
        gateway_url: String::new(),
        cdn_base_url: String::new(),
    };
    let runtime = Runtime::new().unwrap();
    let images = Rc::new(RefCell::new(Some(ImageCache::new(
        runtime.handle().clone(),
        wreq::Client::new(),
        || {},
    ))));
    let queries = Rc::new(RefCell::new(Some(QueryClient::new(
        runtime.handle().clone(),
        || {},
    ))));
    let http = HttpClient::new(
        backend.clone(),
        AuthCoordinator::new(backend.clone(), String::new()),
    )
    .unwrap();
    let (compose_tx, _rx) = mpsc::unbounded_channel();
    let videos = VideoCache::new(
        runtime.handle().clone(),
        rest::build_video_client().unwrap(),
    );
    let ctx = RootContext {
        fonts,
        backend,
        images,
        queries,
        http,
        compose_tx,
        measure_cache: Rc::new(MeasureCache::default()),
        videos,
    };

    let selected_channel = signal(Some(Id::<ChannelMarker>::new(11)));
    let global_tick = signal(0u64);
    let viewer = MediaViewerState::inert();
    let signals = RootSignals {
        selected_channel,
        member_list: ui::member_list::MemberListState::new(),
        search: ui::search_state::SearchState::new(),
        account_popout: ui::account_popout::AccountPopoutState::default(),
        on_select_channel: Rc::new(|_| {}),
        composer_focused: signal(false),
        composer_has_text: signal(false),
        composer_send_toggle: signal(ToggleState::new(false)),
        message_scroll: signal(HashMap::new()),
        mute_toggle: signal(ToggleState::new(false)),
        on_toggle_mute: Rc::new(|| {}),
        deafen_toggle: signal(ToggleState::new(false)),
        on_toggle_deafen: Rc::new(|| {}),
        settings_modal_open: signal(ToggleState::new(false)),
        on_open_settings: Rc::new(|| {}),
        on_close_settings: Rc::new(|| {}),
        on_log_out: Rc::new(|| {}),
        settings_ui: signal(SettingsUiState::default()),
        global_tick: global_tick.clone(),
        fullscreen_video: signal(None),
        attachments: AttachmentsState::inert(),
        media_viewer: viewer.clone(),
        toasts: ToastState::new(),
        emoji_picker: EmojiPickerState {
            open: signal(false),
            tab: signal(PickerTab::Emoji),
            signals: EmojiPickerSignals {
                search: signal(String::new()),
                scroll_offset: signal(0.0),
                category_jump: signal(None),
                hovered: signal(None),
                collapsed: signal(Vec::new()),
            },
            gif: GifPickerState {
                signals: GifPickerSignals {
                    search: signal(String::new()),
                    categories: signal(Vec::new()),
                    trending: signal(Vec::new()),
                    results: signal(Vec::new()),
                    favorites: signal(FavoriteGifs::default()),
                    view: signal(GifView::Browse),
                    image_tick: global_tick,
                },
                on_search: Rc::new(|_| {}),
                on_open: Rc::new(|| {}),
                on_save_favorites: Rc::new(|_| {}),
            },
            on_insert: Rc::new(|_| {}),
            on_insert_at: Rc::new(|_, _| {}),
        },
        profile_popout: ProfilePopoutState {
            open: signal(None),
            draft: signal(String::new()),
            emoji_open: signal(false),
            emoji_tab: signal(PickerTab::Emoji),
            emoji_search: signal(String::new()),
            emoji_scroll_offset: signal(0.0),
            emoji_category_jump: signal(None),
            emoji_hovered: signal(None),
            emoji_collapsed: signal(Vec::new()),
            role_picker_open: signal(false),
            window_resize_tick: signal(0),
            overlays: ProfileOverlays::new(),
        },
    };

    app::set_window_size(1400.0, 900.0);
    let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
    let effects = shell::mount_root(&mut tree, &state, &ctx, &signals);
    let theme = shell::load_theme();
    let mut app = App {
        tree,
        viewer,
        theme,
        _effects: effects,
        _runtime: runtime,
    };
    app.layout();
    app
}

impl App {
    fn layout(&mut self) {
        requiem_ui_signal::flush(&mut self.tree);
        cascade::apply_stylesheet(&mut self.tree, &self.theme);
        requiem_ui_layout::compute_layout(
            &mut self.tree,
            Size {
                width: AvailableSpace::Definite(1400.0),
                height: AvailableSpace::Definite(900.0),
            },
        );
    }

    fn find_center(&self, pred: impl Fn(&[String]) -> bool) -> Option<(f32, f32)> {
        fn walk(
            tree: &WidgetTree,
            rects: &HashMap<WidgetId, widget::AbsoluteRect>,
            id: WidgetId,
            pred: &dyn Fn(&[String]) -> bool,
            found: &mut Option<(f32, f32)>,
        ) {
            if found.is_some() {
                return;
            }
            if pred(&tree.nodes[id].classes) {
                let r = rects[&id];
                if r.width > 1.0 && r.height > 1.0 {
                    *found = Some((r.x + r.width / 2.0, r.y + r.height / 2.0));
                    return;
                }
            }
            for &child in &tree.nodes[id].children {
                walk(tree, rects, child, pred, found);
            }
        }

        let mut found = None;
        let rects = paint::all_absolute_rects(&self.tree);
        walk(&self.tree, &rects, self.tree.root, &pred, &mut found);
        found
    }

    fn click(&mut self, x: f32, y: f32) {
        paint::dispatch_event(&mut self.tree, &UiEvent::MouseMoved { x, y });
        self.layout();
        paint::dispatch_event(
            &mut self.tree,
            &UiEvent::MouseDown {
                x,
                y,
                button: MouseButton::Left,
                shift: false,
            },
        );
        paint::dispatch_event(
            &mut self.tree,
            &UiEvent::MouseUp {
                x,
                y,
                button: MouseButton::Left,
            },
        );
        self.layout();
    }
}

fn image_attachment(name: &str) -> serde_json::Value {
    serde_json::json!({ "id": name.bytes().map(u64::from).sum::<u64>().to_string(), "filename": name, "size": 1000, "content_type": "image/png", "width": 400, "height": 200, "url": format!("https://cdn.test/{name}") })
}

#[test]
fn clicking_an_attachment_image_in_the_real_message_list_opens_the_viewer() {
    let author = user(100, "alex");
    let mut app = boot(vec![message(
        1000,
        &author,
        serde_json::json!({ "content": "look", "attachments": [image_attachment("a.png")] }),
    )]);
    let (x, y) = app
        .find_center(|c| {
            c.iter()
                .any(|c| c == ui::classes::message_embed::ATTACHMENT_IMAGE)
        })
        .expect("the image is in the tree");
    paint::dispatch_event(&mut app.tree, &UiEvent::MouseMoved { x, y });
    app.layout();
    assert_eq!(
        paint::cursor_at(&app.tree, x, y),
        CursorIcon::Pointer,
        "cursor over the image"
    );
    app.click(x, y);
    assert!(app.viewer.is_open(), "the click opened the viewer");

    for _ in 0..10 {
        thread::sleep(Duration::from_millis(20));
        timer::run_due();
        app.layout();
    }
    let mut id = paint::hit_test_all(&app.tree, 700.0, 450.0);
    let mut on_top = false;
    while let Some(i) = id {
        on_top |= app.tree.nodes[i]
            .classes
            .iter()
            .any(|c| c.contains("media_viewer-viewer"));
        id = app.tree.nodes[i].parent;
    }
    assert!(on_top, "the viewer covers the app");
    assert!(app.viewer.shown.peek());
}

#[test]
fn the_release_of_the_opening_click_does_not_zoom() {
    let author = user(100, "alex");
    let mut app = boot(vec![message(
        1000,
        &author,
        serde_json::json!({ "attachments": [image_attachment("a.png")] }),
    )]);
    let (x, y) = app
        .find_center(|c| {
            c.iter()
                .any(|c| c == ui::classes::message_embed::ATTACHMENT_IMAGE)
        })
        .unwrap();
    paint::dispatch_event(&mut app.tree, &UiEvent::MouseMoved { x, y });
    app.layout();
    paint::dispatch_event(
        &mut app.tree,
        &UiEvent::MouseDown {
            x,
            y,
            button: MouseButton::Left,
            shift: false,
        },
    );
    settle(&mut app);
    assert!(app.viewer.is_open());

    paint::dispatch_event(&mut app.tree, &UiEvent::MouseMoved { x: 700.0, y: 450.0 });
    paint::dispatch_event(
        &mut app.tree,
        &UiEvent::MouseUp {
            x: 700.0,
            y: 450.0,
            button: MouseButton::Left,
        },
    );
    paint::clear_all_press(&mut app.tree);
    settle(&mut app);
    assert!(
        !app.viewer.zoomed.peek(),
        "the opening click's release must not zoom"
    );

    app.click(700.0, 450.0);
    settle(&mut app);
    assert!(app.viewer.zoomed.peek(), "a full click on the image zooms");
}

fn open_by_clicking_class(message: Message, class: &'static str) -> (bool, CursorIcon) {
    let mut app = boot(vec![message]);
    let (x, y) = app
        .find_center(|c| c.iter().any(|c| c == class))
        .expect("the media is in the tree");
    paint::dispatch_event(&mut app.tree, &UiEvent::MouseMoved { x, y });
    app.layout();
    let cursor = paint::cursor_at(&app.tree, x, y);
    app.click(x, y);
    (app.viewer.is_open(), cursor)
}

fn media(url: &str) -> serde_json::Value {
    serde_json::json!({ "url": url, "width": 400, "height": 200 })
}

#[test]
fn clicking_an_image_link_embed_opens_the_viewer() {
    let author = user(100, "alex");
    let embed = serde_json::json!({ "type": "image", "url": "https://x.test/a.png", "thumbnail": media("https://x.test/a.png") });
    let (open, cursor) = open_by_clicking_class(
        message(1000, &author, serde_json::json!({ "embeds": [embed] })),
        ui::classes::message_embed::ATTACHMENT_IMAGE,
    );
    assert_eq!(cursor, CursorIcon::Pointer);
    assert!(open);
}

#[test]
fn clicking_a_rich_embed_image_opens_the_viewer() {
    let author = user(100, "alex");
    let embed =
        serde_json::json!({ "type": "rich", "title": "t", "image": media("https://x.test/a.png") });
    let (open, cursor) = open_by_clicking_class(
        message(1000, &author, serde_json::json!({ "embeds": [embed] })),
        ui::classes::message_embed::EMBED_IMAGE,
    );
    assert_eq!(cursor, CursorIcon::Pointer);
    assert!(open);
}

#[test]
fn clicking_a_gif_embed_opens_the_viewer() {
    let author = user(100, "alex");
    let embed = serde_json::json!({ "type": "gifv", "url": "https://klipy.test/g", "thumbnail": media("https://x.test/g.png"), "video": media("https://x.test/g.mp4") });
    let (open, cursor) = open_by_clicking_class(
        message(1000, &author, serde_json::json!({ "embeds": [embed] })),
        ui::classes::message_embed::ATTACHMENT_IMAGE,
    );
    assert_eq!(cursor, CursorIcon::Pointer);
    assert!(open);
}

#[test]
fn a_spoiler_image_reveals_on_the_first_click_and_opens_on_the_second() {
    let author = user(100, "alex");
    let mut spoiler = image_attachment("secret.png");
    spoiler["flags"] = serde_json::json!(8);
    let mut app = boot(vec![message(
        1000,
        &author,
        serde_json::json!({ "attachments": [spoiler] }),
    )]);
    let (x, y) = app
        .find_center(|c| {
            c.iter()
                .any(|c| c == ui::classes::message_embed::ATTACHMENT_IMAGE)
        })
        .expect("the image is in the tree");

    paint::dispatch_event(&mut app.tree, &UiEvent::MouseMoved { x, y });
    app.layout();
    assert_eq!(
        paint::cursor_at(&app.tree, x, y),
        CursorIcon::Pointer,
        "the cover reads as clickable"
    );
    let snap = |app: &mut App, name: &str| {
        if let Ok(dir) = env::var("SPOILER_PREVIEW_DIR") {
            let mut surface = surfaces::raster_n32_premul((1400, 900)).unwrap();
            paint::paint(&mut app.tree, surface.canvas());
            let png = surface
                .image_snapshot()
                .encode(None, skia_safe::EncodedImageFormat::PNG, 100)
                .unwrap();
            fs::write(Path::new(&dir).join(name), png.as_bytes()).unwrap();
        }
    };
    snap(&mut app, "spoiler_hover.png");
    app.click(x, y);
    snap(&mut app, "spoiler_revealed.png");
    assert!(
        !app.viewer.is_open(),
        "the first click only lifts the spoiler"
    );
    app.click(x, y);
    assert!(
        app.viewer.is_open(),
        "once revealed it opens like any image"
    );
}

#[test]
fn only_the_image_itself_is_clickable() {
    let author = user(100, "alex");
    let mut app = boot(vec![message(
        1000,
        &author,
        serde_json::json!({ "attachments": [image_attachment("a.png")] }),
    )]);
    let (x, y) = app
        .find_center(|c| {
            c.iter()
                .any(|c| c == ui::classes::message_embed::ATTACHMENT_IMAGE)
        })
        .unwrap();

    app.click(x + 300.0, y);
    assert!(
        !app.viewer.is_open(),
        "clicking beside the image does nothing"
    );
    paint::dispatch_event(&mut app.tree, &UiEvent::MouseMoved { x: x + 300.0, y });
    app.layout();
    assert_ne!(
        paint::cursor_at(&app.tree, x + 300.0, y),
        CursorIcon::Pointer
    );
    app.click(x, y);
    assert!(app.viewer.is_open());
}

fn settle(app: &mut App) {
    for _ in 0..30 {
        thread::sleep(Duration::from_millis(20));
        timer::run_due();
        app.layout();
        let mut surface = surfaces::raster_n32_premul((1400, 900)).unwrap();
        paint::paint(&mut app.tree, surface.canvas());
    }
}

fn snap(app: &mut App, name: &str) {
    if let Ok(dir) = env::var("VIEWER_PREVIEW_DIR") {
        let mut surface = surfaces::raster_n32_premul((1400, 900)).unwrap();
        paint::paint(&mut app.tree, surface.canvas());
        let png = surface
            .image_snapshot()
            .encode(None, skia_safe::EncodedImageFormat::PNG, 100)
            .unwrap();
        fs::write(Path::new(&dir).join(name), png.as_bytes()).unwrap();
    }
}

fn hover(app: &mut App, x: f32, y: f32) -> CursorIcon {
    paint::dispatch_event(&mut app.tree, &UiEvent::MouseMoved { x, y });
    app.layout();
    paint::cursor_at(&app.tree, x, y)
}

#[test]
fn viewer_cursors_panning_and_view_details_flip() {
    let author = user(100, "alex");
    let mut app = boot(vec![message(
        1000,
        &author,
        serde_json::json!({ "attachments": [image_attachment("a.png")] }),
    )]);
    let (x, y) = app
        .find_center(|c| {
            c.iter()
                .any(|c| c == ui::classes::message_embed::ATTACHMENT_IMAGE)
        })
        .unwrap();
    app.click(x, y);
    settle(&mut app);
    assert_eq!(hover(&mut app, 700.0, 450.0), CursorIcon::ZoomIn);

    app.click(700.0, 450.0);
    settle(&mut app);
    assert!(app.viewer.zoomed.peek());
    assert_eq!(hover(&mut app, 700.0, 450.0), CursorIcon::ZoomOut);

    let down = |app: &mut App, x: f32, y: f32| {
        paint::dispatch_event(&mut app.tree, &UiEvent::MouseMoved { x, y });
        paint::dispatch_event(
            &mut app.tree,
            &UiEvent::MouseDown {
                x,
                y,
                button: MouseButton::Left,
                shift: false,
            },
        );
    };
    down(&mut app, 700.0, 450.0);
    paint::dispatch_event(&mut app.tree, &UiEvent::MouseMoved { x: 640.0, y: 430.0 });
    paint::dispatch_event(&mut app.tree, &UiEvent::MouseMoved { x: 700.0, y: 450.0 });
    assert_eq!(
        paint::cursor_at(&app.tree, 700.0, 450.0),
        CursorIcon::Grabbing
    );
    paint::dispatch_event(
        &mut app.tree,
        &UiEvent::MouseUp {
            x: 700.0,
            y: 450.0,
            button: MouseButton::Left,
        },
    );
    assert!(
        app.viewer.zoomed.peek(),
        "a drag that returns to its start is still not a click"
    );

    for i in 0..50 {
        let d = if i % 2 == 0 { -30.0 } else { 30.0 };
        down(&mut app, 700.0, 450.0);
        paint::dispatch_event(
            &mut app.tree,
            &UiEvent::MouseMoved {
                x: 700.0 + d,
                y: 450.0,
            },
        );
        paint::dispatch_event(
            &mut app.tree,
            &UiEvent::MouseUp {
                x: 700.0 + d,
                y: 450.0,
                button: MouseButton::Left,
            },
        );
    }
    assert!(app.viewer.zoomed.peek());

    app.click(700.0, 450.0);
    settle(&mut app);
    assert!(!app.viewer.zoomed.peek());

    app.viewer.menu_open.set(true);
    settle(&mut app);
    let (ix, iy) = app
        .find_center(|c| {
            c.iter()
                .any(|c| c == ui::classes::media_viewer::MENU_ITEM_WRAP)
        })
        .expect("View Details item");
    hover(&mut app, ix, iy);
    assert!(app.viewer.details_open.peek(), "hovering the item opens it");
    let panel_rect = |app: &App| {
        let rects = paint::all_absolute_rects(&app.tree);
        let panel = app
            .tree
            .nodes
            .iter()
            .find(|(_, n)| {
                n.classes
                    .iter()
                    .any(|c| c == ui::classes::media_viewer::SUBMENU)
            })
            .map(|(id, _)| id)
            .expect("submenu is mounted");
        rects[&panel]
    };
    let first = panel_rect(&app);
    snap(&mut app, "viewer_details.png");
    assert!(
        first.x >= 0.0 && first.x + first.width <= 1400.0,
        "on screen at once: {first:?}"
    );
    settle(&mut app);
    let later = panel_rect(&app);
    assert_eq!(
        (first.x, first.y),
        (later.x, later.y),
        "never moves after appearing"
    );
    let menu = app
        .find_center(|c| c.iter().any(|c| c.contains("media_viewer-menu_")))
        .unwrap();
    assert!(
        later.x + later.width < menu.0,
        "opens to the left of the menu: {later:?}"
    );

    hover(&mut app, later.x + later.width / 2.0, later.y + 20.0);
    assert!(
        app.viewer.details_open.peek(),
        "still open over the submenu"
    );
    hover(&mut app, later.x + later.width + 2.0, later.y + 20.0);
    assert!(
        app.viewer.details_open.peek(),
        "still open crossing the gap"
    );
    hover(&mut app, 300.0, 600.0);
    assert!(
        !app.viewer.details_open.peek(),
        "closed once the pointer leaves"
    );
}
