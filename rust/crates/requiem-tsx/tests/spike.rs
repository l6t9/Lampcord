use std::any::Any;
use std::cell::RefCell;
use std::env;
use std::fs;
use std::rc::Rc;

use requiem_tsx::{
    Actions, CssModules, FromValue, Host, Options, Source, StateQueries, Value, compile_project,
    declarations, mount, ui_actions,
};
use requiem_ui::classes;
use requiem_ui::colors;
use requiem_ui::components::{
    Align, ButtonProps, ButtonSize, ButtonVariant, Padding, ScrollViewProps, Space, StackProps,
    TextProps, icon,
};
use requiem_ui::event::{MouseButton, UiEvent};
use requiem_ui::font_system::TextFonts;
use requiem_ui::icons;
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, flush};
use requiem_ui::style::ComputedStyle;
use requiem_ui::typography::TextVariant;
use requiem_ui::widget::primitives::{BoxWidget, ChevronWidget, TextWidget};
use requiem_ui::widget::{AbsoluteRect, WidgetId, WidgetTree};
use serde::Serialize;
use specta::Type;
use specta_typescript::branded;

branded!(#[derive(Clone, Debug, PartialEq, Serialize)] struct ThreadId(String));

impl FromValue for ThreadId {
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Str(text) => Some(ThreadId(text.to_string())),
            _ => None,
        }
    }

    fn ts() -> String {
        "ThreadId".into()
    }
}

ui_actions! {
    #[derive(Debug, PartialEq)]
    enum ThreadAction {
        openThread(id: ThreadId, shift_key: bool) => OpenThread,
        createThread() => CreateThread,
    }
}

const THREAD_LIST: &str = include_str!("fixtures/ThreadList.tsx");
const THREAD_ROW: &str = include_str!("fixtures/ThreadRow.tsx");
const THREAD_LIST_CSS: &str = include_str!("fixtures/ThreadList.module.css");
const THREAD_ROW_CSS: &str = include_str!("fixtures/ThreadRow.module.css");

fn sources() -> Vec<Source> {
    vec![
        Source::new("ThreadList.tsx", THREAD_LIST),
        Source::new("ThreadRow.tsx", THREAD_ROW),
    ]
}

fn css_modules() -> CssModules {
    let mut css = CssModules::default();
    for (stem, text) in [("ThreadList", THREAD_LIST_CSS), ("ThreadRow", THREAD_ROW_CSS)] {
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix('.') {
                let class: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                    .collect();
                css.insert(stem, &class, &format!("{stem}-{class}_test00"));
            }
        }
    }
    css
}

fn cls(class: &str) -> String {
    format!("ThreadList-{class}_test00")
}

fn row_cls(class: &str) -> String {
    format!("ThreadRow-{class}_test00")
}

fn options() -> Options {
    Options {
        actions: ThreadAction::specs(),
        css_modules: css_modules(),
        ..Options::default()
    }
}

fn tree() -> WidgetTree {
    TextFonts::install_new(1024 * 1024);
    WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default())
}

fn dump(tree: &WidgetTree, id: WidgetId) -> String {
    fn walk(tree: &WidgetTree, id: WidgetId, depth: usize, out: &mut String) {
        let node = &tree.nodes[id];
        out.push_str(&"  ".repeat(depth));
        out.push_str(node.tag);
        for class in &node.classes {
            out.push('.');
            out.push_str(class);
        }
        if let Some(style) = &node.inline_style_source {
            out.push_str(&format!(" [{style}]"));
        }
        if let Some(text) = (&*node.widget as &dyn Any).downcast_ref::<TextWidget>() {
            out.push_str(&format!(" {:?}", text.text()));
        }
        out.push('\n');
        for &child in &node.children {
            walk(tree, child, depth + 1, out);
        }
    }
    let mut out = String::new();
    walk(tree, id, 0, &mut out);
    out
}

fn click(tree: &mut WidgetTree, id: WidgetId) {
    click_with(tree, id, false);
}

fn click_with(tree: &mut WidgetTree, id: WidgetId, shift: bool) {
    let press = UiEvent::MouseDown {
        x: 0.0,
        y: 0.0,
        button: MouseButton::Left,
        shift,
    };
    let release = UiEvent::MouseUp {
        x: 0.0,
        y: 0.0,
        button: MouseButton::Left,
    };
    let rect = AbsoluteRect::default();
    tree.nodes[id].widget.handle_event(&press, rect);
    tree.nodes[id].widget.handle_event(&release, rect);
}

#[derive(Clone, Serialize, Type)]
struct Thread {
    id: ThreadId,
    name: &'static str,
    subtext: &'static str,
}

fn thread(id: u32, name: &'static str, subtext: &'static str) -> Thread {
    Thread {
        id: ThreadId(id.to_string()),
        name,
        subtext,
    }
}

fn to_value(threads: &[Thread]) -> Value {
    requiem_tsx::to_value(&threads)
}

fn shared_types() -> StateQueries {
    StateQueries::new().export::<Thread>()
}

fn rust_view(threads: &[Thread], expanded: bool) -> Node {
    let div = |class: &str| Node::new("div", BoxWidget).class(cls(class));
    let mut root = div("browser")
        .child(
            ButtonProps::default()
                .variant(ButtonVariant::Bare)
                .extra_classes(vec![cls("header")])
                .on_click(|| {})
                .build()
                .child(
                    StackProps::horizontal()
                        .align(Align::Center)
                        .build()
                        .child(
                            icon(icons::THREAD, colors::TEXT_SECONDARY)
                                .class(classes::icon::ROOT)
                                .style("width: 16px; height: 16px"),
                        )
                        .child(
                            TextProps::styled(
                                format!("Threads ({})", threads.len()),
                                TextVariant::HEADING_MD_SEMIBOLD,
                            )
                            .color(colors::TEXT_STRONG)
                            .build(),
                        )
                        .child(
                            Node::new(
                                "span",
                                ChevronWidget::new(
                                    icons::CHEVRON_LARGE_DOWN,
                                    colors::TEXT_SECONDARY,
                                    expanded,
                                ),
                            )
                            .class(classes::icon::ROOT),
                        ),
                ),
        )
        .child(
            ButtonProps::new(ButtonVariant::Primary, "New thread", || {})
                .size(ButtonSize::Small)
                .build(),
        )
        .child(
            ButtonProps::new(
                ButtonVariant::Secondary,
                if expanded { "Hide" } else { "Show" },
                || {},
            )
            .size(ButtonSize::Small)
            .build(),
        );
    if expanded {
        let mut rows = div("rows");
        for t in threads {
            rows = rows.child(
                ButtonProps::default()
                    .variant(ButtonVariant::Bare)
                    .extra_classes(vec![row_cls("row")])
                    .on_click(|| {})
                    .build()
                    .child(
                        Node::new("div", BoxWidget)
                            .class(row_cls("row-text"))
                            .child(
                                TextProps::styled(t.name, TextVariant::HEADING_MD_SEMIBOLD)
                                    .color(colors::TEXT_DEFAULT)
                                    .line_clamp(1_u32)
                                    .build(),
                            )
                            .child(
                                TextProps::styled(t.subtext, TextVariant::TEXT_SM_NORMAL)
                                    .color(colors::TEXT_SECONDARY)
                                    .line_clamp(1_u32)
                                    .build(),
                            ),
                    ),
            );
        }
        root = root.child(ScrollViewProps::default().build().child(rows));
    }
    if threads.is_empty() {
        root = root.child(
            StackProps::vertical()
                .align(Align::Center)
                .padding(Padding::all(Space(32.0)))
                .build()
                .child(
                    TextProps::styled("No threads yet", TextVariant::HEADING_XL_SEMIBOLD)
                        .color(colors::TEXT_STRONG)
                        .center(true)
                        .build(),
                ),
        );
    }
    root
}

fn rust_dump(threads: &[Thread], expanded: bool) -> String {
    let mut tree = tree();
    let root = tree.root;
    let id = tree.mount(root, rust_view(threads, expanded));
    dump(&tree, id)
}

struct Harness {
    tree: WidgetTree,
    root: WidgetId,
    threads: Signal<Value>,
    events: Rc<RefCell<Vec<ThreadAction>>>,
}

impl Harness {
    fn new(initial: &[Thread]) -> Self {
        let sources = sources();
        let module = compile_project(&sources, &options()).unwrap_or_else(|errors| {
            panic!(
                "{}",
                errors
                    .iter()
                    .map(|e| {
                        let text = &sources.iter().find(|s| s.path == e.path).expect("source").text;
                        e.diagnostic.render(&e.path, text)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        });
        let events = Rc::new(RefCell::new(Vec::new()));
        let sink = events.clone();
        let host = Host::new(move |action: ThreadAction| sink.borrow_mut().push(action));
        let mut tree = tree();
        let parent = tree.root;
        let threads = Signal::new(to_value(initial));
        let root = mount(
            &mut tree,
            parent,
            &module,
            "ThreadList",
            &[("threads", threads.clone())],
            &host,
        )
        .expect("ThreadList");
        Self {
            tree,
            root,
            threads,
            events,
        }
    }

    fn dump(&self) -> String {
        dump(&self.tree, self.root)
    }

    fn children(&self, id: WidgetId) -> Vec<WidgetId> {
        self.tree.nodes[id].children.clone()
    }

    fn rows(&self) -> Vec<WidgetId> {
        let scroll = self.children(self.root)[3];
        let stack = self.children(scroll)[0];
        self.children(stack)
    }

    fn header(&self) -> WidgetId {
        self.children(self.root)[0]
    }

    fn set(&mut self, threads: &[Thread]) {
        self.threads.set(to_value(threads));
        flush(&mut self.tree);
    }
}

fn three() -> Vec<Thread> {
    vec![
        thread(1, "release notes", "Ana: shipped"),
        thread(2, "old plans", "Started by Bo"),
        thread(3, "bugs", "Cy: repro attached"),
    ]
}

#[test]
fn the_first_render_matches_the_rust_view() {
    let h = Harness::new(&three());
    assert_eq!(h.dump(), rust_dump(&three(), true));
}

#[test]
fn clicking_the_header_folds_and_unfolds_the_rows() {
    let mut h = Harness::new(&three());
    let header = h.header();

    click(&mut h.tree, header);
    flush(&mut h.tree);
    assert_eq!(h.dump(), rust_dump(&three(), false));

    click(&mut h.tree, header);
    flush(&mut h.tree);
    assert_eq!(h.dump(), rust_dump(&three(), true));
}

#[test]
fn button_text_is_its_label_and_follows_state() {
    let mut h = Harness::new(&three());
    let toggle = h.children(h.root)[2];
    click(&mut h.tree, toggle);
    flush(&mut h.tree);
    assert_eq!(h.dump(), rust_dump(&three(), false));
    click(&mut h.tree, toggle);
    flush(&mut h.tree);
    assert_eq!(h.dump(), rust_dump(&three(), true));

    let create = h.children(h.root)[1];
    click(&mut h.tree, create);
    assert_eq!(*h.events.borrow(), vec![ThreadAction::CreateThread {}]);
}

#[test]
fn an_empty_list_shows_the_empty_state() {
    let mut h = Harness::new(&three());
    h.set(&[]);
    assert_eq!(h.dump(), rust_dump(&[], true));
    h.set(&three());
    assert_eq!(h.dump(), rust_dump(&three(), true));
}

#[test]
fn rows_are_added_removed_and_reordered_by_key_without_rebuilding_survivors() {
    let mut h = Harness::new(&three());
    let before = h.rows();

    let mut next = three();
    next.reverse();
    next.push(thread(4, "new thread", "Di: hello"));
    next.remove(1);
    h.set(&next);
    let after = h.rows();

    assert_eq!(h.dump(), rust_dump(&next, true));
    assert_eq!(after.len(), 3);
    assert_eq!(after[0], before[2]);
    assert_eq!(after[1], before[0]);
    assert!(!h.tree.nodes.contains_key(before[1]));
}

#[test]
fn a_row_whose_data_changed_updates_in_place() {
    let mut h = Harness::new(&three());
    let before = h.rows();

    let mut next = three();
    next[0].name = "renamed";
    next[0].subtext = "Ana: edited";
    h.set(&next);

    assert_eq!(h.rows(), before);
    assert_eq!(h.dump(), rust_dump(&next, true));
}

#[test]
fn a_row_click_reaches_the_action_with_that_rows_item() {
    let mut h = Harness::new(&three());
    let rows = h.rows();
    click(&mut h.tree, rows[2]);
    click_with(&mut h.tree, rows[0], true);
    assert_eq!(
        *h.events.borrow(),
        vec![
            ThreadAction::OpenThread {
                id: ThreadId("3".into()),
                shift_key: false
            },
            ThreadAction::OpenThread {
                id: ThreadId("1".into()),
                shift_key: true
            },
        ]
    );
}

#[test]
fn changing_one_row_leaves_the_others_text_alone() {
    let mut h = Harness::new(&three());
    let rows = h.rows();
    let name_of = |h: &Harness, row: WidgetId| {
        let stack = h.children(row)[0];
        let name = h.children(stack)[0];
        (&*h.tree.nodes[name].widget as &dyn Any)
            .downcast_ref::<TextWidget>()
            .map(|t| t.text().to_string())
            .unwrap_or_default()
    };
    let mut next = three();
    next[1].name = "archived plans";
    h.set(&next);
    assert_eq!(name_of(&h, rows[0]), "release notes");
    assert_eq!(name_of(&h, rows[1]), "archived plans");
    assert_eq!(name_of(&h, rows[2]), "bugs");
}

#[test]
fn the_checked_in_typescript_declarations_match_the_compiler() {
    let options = options();
    let generated = declarations(&options.css_modules, &options.actions, &shared_types(), &[], &[]);
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/requiem.d.ts");
    if env::var_os("REQUIEM_UPDATE_TYPES").is_some() {
        fs::write(path, &generated).expect("write requiem.d.ts");
    }
    let checked_in = fs::read_to_string(path).unwrap_or_default().replace("\r\n", "\n");
    assert_eq!(checked_in, generated, "run with REQUIEM_UPDATE_TYPES=1 to regenerate");
}
