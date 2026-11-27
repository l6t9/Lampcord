use std::collections::HashMap;

use requiem_ui::event::UiEvent;
use requiem_ui::font_system::{self, TextFonts};
use requiem_ui::layout;
use requiem_ui::paint;
use requiem_ui::signal::{flush, signal};
use requiem_ui::style::ComputedStyle;
use requiem_ui::style::{cascade, parser};
use requiem_ui::theme as requiem_ui_theme;
use requiem_ui::widget::primitives::BoxWidget;
use requiem_ui::widget::{AbsoluteRect, ScrollChild, WidgetId, WidgetTree};
use skia_safe::font_style::Weight;
use taffy::{AvailableSpace, Size};
use tokio::runtime::Runtime;

use super::*;
use crate::features::messages::components::attachments::AttachmentsBackend;
use crate::ui::profile_popout::ProfileOverlays;
use crate::models::{Channel, Message};
use crate::state::Action;
use crate::ui::emoji_picker::PickerTab;

const WIDTH: f32 = 700.0;
const HEIGHT: f32 = 600.0;

struct Chat {
    tree: WidgetTree,
    list: WidgetId,
    state: AppState,
    _runtime: Runtime,
}

fn channel() -> Channel {
    serde_json::from_value(serde_json::json!({
        "id": "100", "guild_id": "9", "type": 0, "name": "general"
    }))
    .expect("channel")
}

fn message(id: u64, author: u64, content: &str) -> Message {
    serde_json::from_value(serde_json::json!({
        "id": id.to_string(),
        "channel_id": "100",
        "author": {"id": author.to_string(), "username": format!("user{author}"), "discriminator": "0"},
        "content": content,
        "timestamp": "2026-10-27T10:00:00.000000+00:00"
    }))
    .expect("message")
}

fn setup(count: u64) -> Chat {
    let runtime = Runtime::new().expect("runtime");
    let images = ImageCache::new(runtime.handle().clone(), wreq::Client::new(), || {});
    let fonts = Fonts {
        typeface: requiem_ui_theme::make_typeface(Weight::MEDIUM),
        text_fonts: TextFonts::new(1024 * 1024),
    };
    font_system::install_ambient(fonts.text_fonts.clone());
    let backend = BackendConfig {
        rest_base_url: String::new(),
        gateway_url: String::new(),
        cdn_base_url: String::new(),
    };
    let state = AppState::new();
    state.channels.upsert(channel(), false);
    for n in 0..count {
        state.dispatch(&Action::MessageCreate(message(
            1_000_000 + n * 10,
            1 + n % 3,
            &format!("message number {n}"),
        )));
    }
    let profile_popout = ProfilePopoutState {
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
    };
    let attachments = AttachmentsState::new(AttachmentsBackend {
        pick_files: Rc::new(|_| {}),
        send: Rc::new(|_, _, _, _| {}),
        cancel: Rc::new(|_| {}),
        delete_upload: Rc::new(|_| {}),
    });
    install(ListCtx {
        state: state.clone(),
        fonts: fonts.clone(),
        images: images.clone(),
        backend: backend.clone(),
        colors: ChatColors::desktop(),
        profile_popout,
        queries: QueryClient::new(runtime.handle().clone(), || {}),
        attachments,
        media_viewer: media_viewer::MediaViewerState::new(Some(Rc::new(|_, _| {}))),
        measure_cache: Rc::new(MeasureCache::default()),
        images_ready: signal(0),
        message_scroll: signal(HashMap::new()),
        http: None,
        global_tick: signal(0),
        #[cfg(feature = "video")]
        videos: None,
    });
    ENTRY_HEIGHTS.with(|heights| heights.borrow_mut().clear());
    let tsx = tsx().expect("the message list compiles");
    let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
    let root = tree.root;
    let main = tree.mount(
        root,
        div()
            .class(message_list::MAIN_CONTENT)
            .style(format!("width: {WIDTH}px; height: {HEIGHT}px")),
    );
    let list = tree.mount(
        main,
        tsx.node("MessageList", vec![("channelId", Value::str("100"))])
            .class(message_list::MESSAGE_LIST_HOST),
    );
    let mut chat = Chat {
        tree,
        list,
        state,
        _runtime: runtime,
    };
    chat.settle();
    chat
}

impl Chat {
    fn scroll(&self) -> WidgetId {
        self.tree.nodes[self.list].children[0]
    }

    fn relayout(&mut self) {
        let rules = parser::parse_stylesheet(&format!("{}\n{}", requiem_ui::BUNDLED_CSS, ui::THEME_CSS));
        cascade::apply_stylesheet(&mut self.tree, &rules);
        layout::compute_layout(
            &mut self.tree,
            Size {
                width: AvailableSpace::Definite(WIDTH),
                height: AvailableSpace::Definite(HEIGHT),
            },
        );
    }

    fn paint_step(&mut self) {
        let scroll = self.scroll();
        let content = paint::content_extent(&self.tree, scroll);
        let rect = paint::all_absolute_rects(&self.tree)[&scroll];
        let tops: Vec<(Option<String>, f32, f32)> = self.tree.nodes[scroll]
            .children
            .iter()
            .map(|&child| {
                let layout = self.tree.layout(child);
                (
                    self.tree.nodes[child].dom_id.clone(),
                    layout.location.y,
                    layout.size.height,
                )
            })
            .collect();
        let children: Vec<ScrollChild<'_>> = tops
            .iter()
            .map(|(key, top, height)| ScrollChild {
                key: key.as_deref(),
                top: *top,
                height: *height,
            })
            .collect();
        self.tree.nodes[scroll]
            .widget
            .update_scroll_bounds((rect.width, rect.height), content, &children);
    }

    fn settle(&mut self) {
        for _ in 0..5 {
            flush(&mut self.tree);
            self.relayout();
            self.paint_step();
        }
        flush(&mut self.tree);
        self.relayout();
    }

    fn wheel(&mut self, delta: f32) {
        let rect = paint::all_absolute_rects(&self.tree)[&self.scroll()];
        let scroll = self.scroll();
        self.tree.nodes[scroll].widget.handle_event(
            &UiEvent::Scroll {
                x: rect.x + 10.0,
                y: rect.y + 10.0,
                delta_x: 0.0,
                delta_y: delta,
                precise: true,
            },
            AbsoluteRect::default(),
        );
        self.settle();
    }

    fn mounted_messages(&self) -> Vec<u64> {
        self.tree.nodes[self.scroll()]
            .children
            .iter()
            .filter_map(|&child| self.tree.nodes[child].dom_id.as_ref()?.parse().ok())
            .collect()
    }
}

#[test]
fn a_long_channel_mounts_only_the_newest_rows_and_starts_pinned() {
    let chat = setup(400);
    let mounted = chat.mounted_messages();
    let newest = 1_000_000 + 399 * 10;
    assert_eq!(mounted.last(), Some(&newest), "{mounted:?}");
    assert!(!mounted.contains(&1_000_000));
    assert!(mounted.len() < 60, "{} rows mounted of 400", mounted.len());
}

#[test]
fn a_new_message_appears_at_the_bottom_of_a_pinned_list() {
    let mut chat = setup(40);
    chat.state
        .dispatch(&Action::MessageCreate(message(2_000_000, 7, "brand new")));
    chat.settle();
    assert_eq!(chat.mounted_messages().last(), Some(&2_000_000));
}

#[test]
fn scrolling_far_up_moves_the_window_and_remembers_the_anchor() {
    let mut chat = setup(400);
    chat.wheel(8_000.0);
    let mounted = chat.mounted_messages();
    assert!(!mounted.contains(&(1_000_000 + 399 * 10)), "{mounted:?}");
    assert!(mounted.len() < 80, "{} rows", mounted.len());
    let anchored = ctx()
        .expect("ctx")
        .message_scroll
        .get()
        .get(&Id::new(100))
        .map(|scroll| scroll.position.clone());
    assert!(matches!(anchored, Some(ScrollPosition::Anchored(_))), "{anchored:?}");
}

#[test]
fn the_list_survives_an_outer_rebuild_and_keeps_following_the_store() {
    use requiem_ui::control::section;
    let runtime = Runtime::new().expect("runtime");
    let _ = &runtime;
    let chat = setup(10);
    let mut tree = chat.tree;
    tree.unmount(chat.list);
    let root = tree.root;
    let rebuilds = signal(0_u64);
    let counter = rebuilds.clone();
    let key = 100_u64 ^ 0x5453_5800_0000_0000;
    let _section = section(&mut tree, root, move || {
        let _ = counter.get();
        let list = if control::retained_available(key, TSX_LIST_VERSION) {
            Node::reuse(key, TSX_LIST_VERSION)
        } else {
            list_node(Id::new(100)).expect("list").retained(key, TSX_LIST_VERSION)
        };
        div().class(message_list::MAIN_CONTENT).child(list)
    });
    flush(&mut tree);
    rebuilds.update(|n| *n += 1);
    flush(&mut tree);
    rebuilds.update(|n| *n += 1);
    flush(&mut tree);

    chat.state
        .dispatch(&Action::MessageCreate(message(3_000_000, 5, "after the rebuilds")));
    flush(&mut tree);
    let found = tree
        .nodes
        .iter()
        .any(|(_, node)| node.dom_id.as_deref() == Some("3000000"));
    assert!(found, "the new message reached the list after outer rebuilds");
}

#[test]
fn history_that_arrives_after_the_list_mounted_shows_up() {
    let mut chat = setup(0);
    assert!(chat.mounted_messages().is_empty());
    let page: Vec<Message> = (0..30)
        .rev()
        .map(|n| message(1_000_000 + n * 10, 1 + n % 3, &format!("loaded later {n}")))
        .collect();
    chat.state.set_initial_history(Id::new(100), page);
    chat.settle();
    let mounted = chat.mounted_messages();
    assert_eq!(mounted.last(), Some(&(1_000_000 + 29 * 10)), "{mounted:?}");
}

#[test]
fn a_list_built_in_a_scratch_tree_comes_alive_when_it_reaches_the_real_tree() {
    let mut chat = setup(10);
    chat.tree.unmount(chat.list);
    let mut scratch = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
    scratch.defer_mount_hooks = true;
    let scratch_root = scratch.root;
    let main = scratch.mount(scratch_root, div().class(message_list::MAIN_CONTENT));
    let list = scratch.mount(main, list_node(Id::new(100)).expect("list"));
    assert!(
        scratch.nodes[list].children.is_empty(),
        "nothing is built in the scratch tree"
    );
    let node = ui::detach_only_child(scratch, scratch_root);

    let mut real = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
    let real_root = real.root;
    real.mount(real_root, node);
    flush(&mut real);
    chat.state
        .dispatch(&Action::MessageCreate(message(4_000_000, 5, "fresh")));
    flush(&mut real);
    let found = real
        .nodes
        .iter()
        .any(|(_, node)| node.dom_id.as_deref() == Some("4000000"));
    assert!(found, "the list follows the store in the real tree");
}
