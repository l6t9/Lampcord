use std::any::Any;

use requiem_tsx::{Host, Options, Value, compile, mount};
use requiem_ui::event::UiEvent;
use requiem_ui::font_system::TextFonts;
use requiem_ui::layout;
use requiem_ui::signal::{Signal, flush};
use requiem_ui::style::ComputedStyle;
use requiem_ui::style::{cascade, parser};
use requiem_ui::widget::primitives::{BoxWidget, TextWidget};
use requiem_ui::widget::{AbsoluteRect, WidgetId, WidgetTree};
use taffy::{AvailableSpace, Size};

const SOURCE: &str = r#"
import { ScrollView, Text } from "@requiem/ui";

export function List({ items }) {
  return (
    <ScrollView className="list" itemHeight={50} itemGap={10}>
      {items.map((item) => <Row key={item.id} item={item} />)}
    </ScrollView>
  );
}

export function Row({ item }) {
  return (
    <div className="row">
      <Text variant="text-md/normal">{item.name}</Text>
    </div>
  );
}

export function Varied({ items }) {
  return (
    <ScrollView className="list" itemHeight="height">
      {items.map((item) => <Row key={item.id} item={item} />)}
    </ScrollView>
  );
}
"#;

const CSS: &str = "
.list { width: 200px; height: 300px; }
.row { width: 100%; height: 50px; flex-shrink: 0; }
";

fn items(count: usize) -> Value {
    Value::list((0..count).map(|n| {
        Value::obj([
            ("id", Value::Num(n as f64)),
            ("name", Value::str(format!("row {n}"))),
            ("height", Value::Num(if n % 2 == 0 { 50.0 } else { 30.0 })),
        ])
    }))
}

fn setup(component: &str, list: Value) -> (WidgetTree, WidgetId, Signal<Value>) {
    TextFonts::install_new(1024 * 1024);
    let module = compile(SOURCE, &Options::default()).expect("compiles");
    let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
    let parent = tree.root;
    let items = Signal::new(list);
    let scroll = mount(
        &mut tree,
        parent,
        &module,
        component,
        &[("items", items.clone())],
        &Host::inert(),
    )
    .expect("component");
    relayout(&mut tree);
    (tree, scroll, items)
}

fn relayout(tree: &mut WidgetTree) {
    let rules = parser::parse_stylesheet(&format!("{}\n{CSS}", requiem_ui::BUNDLED_CSS));
    cascade::apply_stylesheet(tree, &rules);
    layout::compute_layout(
        tree,
        Size {
            width: AvailableSpace::Definite(200.0),
            height: AvailableSpace::Definite(300.0),
        },
    );
}

fn column(tree: &WidgetTree, scroll: WidgetId) -> WidgetId {
    tree.nodes[scroll].children[0]
}

fn mounted(tree: &WidgetTree, scroll: WidgetId) -> Vec<String> {
    let mut names = Vec::new();
    for &child in &tree.nodes[column(tree, scroll)].children {
        collect(tree, child, &mut names);
    }
    names
}

fn collect(tree: &WidgetTree, id: WidgetId, out: &mut Vec<String>) {
    if let Some(text) = (&*tree.nodes[id].widget as &dyn Any).downcast_ref::<TextWidget>() {
        out.push(text.text().to_string());
    }
    for &child in &tree.nodes[id].children {
        collect(tree, child, out);
    }
}

fn spacers(tree: &WidgetTree, scroll: WidgetId) -> Vec<String> {
    let kids = &tree.nodes[column(tree, scroll)].children;
    [kids.first(), kids.last()]
        .into_iter()
        .flatten()
        .filter(|id| tree.nodes[**id].classes.is_empty() && tree.nodes[**id].children.is_empty())
        .map(|id| tree.nodes[*id].inline_style_source.clone().unwrap_or_default())
        .collect()
}

fn wheel(tree: &mut WidgetTree, scroll: WidgetId, delta: f32) {
    let event = UiEvent::Scroll {
        x: 10.0,
        y: 10.0,
        delta_x: 0.0,
        delta_y: delta,
        precise: true,
    };
    let rect = AbsoluteRect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 300.0,
    };
    tree.nodes[scroll].widget.handle_event(&event, rect);
    flush(tree);
}

#[test]
fn only_the_rows_near_the_viewport_are_mounted_and_spacers_stand_in_for_the_rest() {
    let (tree, scroll, _) = setup("List", items(1000));
    let shown = mounted(&tree, scroll);
    assert!(shown.len() < 30, "{} rows mounted", shown.len());
    assert_eq!(shown[0], "row 0");
    let spacers = spacers(&tree, scroll);
    assert_eq!(spacers.len(), 1, "{spacers:?}");
    let col = column(&tree, scroll);
    assert_eq!(
        tree.layout(column(&tree, scroll)).size.height,
        1000.0 * 60.0 - 10.0,
        "the column keeps its full content height instead of shrinking to the viewport"
    );
    let below_rows = 1000 - shown.len();
    assert_eq!(
        spacers[0],
        format!("height: {}px; flex-shrink: 0", below_rows * 60 - 10)
    );
}

#[test]
fn scrolling_moves_the_window_and_the_rows_that_stay_are_not_remounted() {
    let (mut tree, scroll, _) = setup("List", items(1000));
    let content = (200.0, 1000.0 * 60.0 - 10.0);
    tree.nodes[scroll]
        .widget
        .update_scroll_bounds((200.0, 300.0), content, &[]);

    wheel(&mut tree, scroll, -300.0);
    let near = mounted(&tree, scroll);
    assert_eq!(near[0], "row 0", "still inside the overscan: {near:?}");
    let kept = tree.nodes[column(&tree, scroll)].children[3];

    wheel(&mut tree, scroll, -3300.0);
    let shown = mounted(&tree, scroll);
    assert!(!shown.contains(&"row 0".to_string()), "{shown:?}");
    assert!(shown.contains(&"row 60".to_string()), "{shown:?}");
    assert!(shown.len() < 30);
    assert_eq!(spacers(&tree, scroll).len(), 2);
    assert!(
        !tree.nodes[column(&tree, scroll)].children.contains(&kept),
        "rows that left the window are unmounted"
    );

    wheel(&mut tree, scroll, 3600.0);
    let back = mounted(&tree, scroll);
    assert_eq!(back[0], "row 0");
    assert_eq!(spacers(&tree, scroll).len(), 1);
}

#[test]
fn a_changed_item_updates_its_mounted_row_in_place() {
    let (mut tree, scroll, list) = setup("List", items(40));
    let row = tree.nodes[column(&tree, scroll)].children[2];
    list.set(Value::list((0..40).map(|n| {
        Value::obj([
            ("id", Value::Num(n as f64)),
            ("name", Value::str(if n == 2 { "renamed".to_string() } else { format!("row {n}") })),
            ("height", Value::Num(50.0)),
        ])
    })));
    flush(&mut tree);
    assert_eq!(tree.nodes[column(&tree, scroll)].children[2], row);
    assert_eq!(mounted(&tree, scroll)[2], "renamed");
}

#[test]
fn row_heights_can_come_from_a_field_of_each_item() {
    let (tree, scroll, _) = setup("Varied", items(1000));
    let shown = mounted(&tree, scroll);
    let spacers = spacers(&tree, scroll);
    assert_eq!(spacers.len(), 1);
    let below: f32 = (shown.len()..1000)
        .map(|n| if n % 2 == 0 { 50.0 } else { 30.0 })
        .sum();
    assert_eq!(spacers[0], format!("height: {below}px; flex-shrink: 0"));
}

#[test]
fn a_windowed_scroll_view_must_hold_exactly_one_map() {
    let source = r#"
import { ScrollView, Text } from "@requiem/ui";
export function Bad() {
  return (
    <ScrollView itemHeight={50}>
      <Text variant="text-md/normal">x</Text>
    </ScrollView>
  );
}
"#;
    let Err(error) = compile(source, &Options::default()) else {
        panic!("rejected");
    };
    assert_eq!(error[0].message, "a ScrollView with `itemHeight` takes exactly one `.map`");
}
