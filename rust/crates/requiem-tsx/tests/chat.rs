use std::any::Any;

use requiem_tsx::{Host, Options, Value, compile, mount};
use requiem_ui::event::UiEvent;
use requiem_ui::font_system::TextFonts;
use requiem_ui::layout;
use requiem_ui::paint;
use requiem_ui::signal::{Signal, flush};
use requiem_ui::style::ComputedStyle;
use requiem_ui::style::{cascade, parser};
use requiem_ui::widget::primitives::{BoxWidget, TextWidget};
use requiem_ui::widget::{AbsoluteRect, ScrollChild, WidgetId, WidgetTree};
use taffy::{AvailableSpace, Size};

const SOURCE: &str = r#"
import { ScrollView, Text } from "@requiem/ui";
import { useState } from "react";

export function Chat({ items }) {
  const [position, setPosition] = useState({ pinned: true, key: null, offset: 0, height: 0 });
  const [measured, setMeasured] = useState(0);
  return (
    <div className="host">
      <Text variant="text-md/normal">{`measured ${measured}`}</Text>
      <ScrollView
        className="list"
        itemHeight="height"
        position={position}
        onPosition={(next) => setPosition(next)}
        onMeasure={(event) => setMeasured(event.heights[0])}
      >
        {items.map((item) => <Row key={item.id} item={item} />)}
      </ScrollView>
    </div>
  );
}

export function Row({ item }) {
  return (
    <div className="row">
      <Text variant="text-md/normal">{item.name}</Text>
    </div>
  );
}
"#;

const CSS: &str = "
.host { width: 200px; height: 340px; }
.list { width: 200px; height: 300px; }
.row { width: 100%; height: 50px; flex-shrink: 0; }
";

const WRONG_ESTIMATE: f64 = 40.0;
const ROW: f32 = 50.0;

fn items(count: usize) -> Value {
    items_estimated(count, f64::from(ROW))
}

fn items_estimated(count: usize, height: f64) -> Value {
    Value::list((0..count).map(|n| {
        Value::obj([
            ("id", Value::Num(n as f64)),
            ("name", Value::str(format!("row {n}"))),
            ("height", Value::Num(height)),
        ])
    }))
}

struct Chat {
    tree: WidgetTree,
    scroll: WidgetId,
    items: Signal<Value>,
}

fn setup(count: usize) -> Chat {
    setup_with(items(count))
}

fn setup_with(list: Value) -> Chat {
    TextFonts::install_new(1024 * 1024);
    let module = compile(SOURCE, &Options::default()).expect("compiles");
    let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
    let parent = tree.root;
    let items = Signal::new(list);
    let host = mount(
        &mut tree,
        parent,
        &module,
        "Chat",
        &[("items", items.clone())],
        &Host::inert(),
    )
    .expect("component");
    let scroll = tree.nodes[host].children[1];
    let mut chat = Chat { tree, scroll, items };
    chat.settle();
    chat
}

impl Chat {
    fn relayout(&mut self) {
        let rules = parser::parse_stylesheet(&format!("{}\n{CSS}", requiem_ui::BUNDLED_CSS));
        cascade::apply_stylesheet(&mut self.tree, &rules);
        layout::compute_layout(
            &mut self.tree,
            Size {
                width: AvailableSpace::Definite(200.0),
                height: AvailableSpace::Definite(340.0),
            },
        );
    }

    fn paint_step(&mut self) {
        let scroll = self.scroll;
        let content = paint::content_extent(&self.tree, scroll);
        let viewport = (200.0, 300.0);
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
            .update_scroll_bounds(viewport, content, &children);
    }

    fn settle(&mut self) {
        for _ in 0..4 {
            flush(&mut self.tree);
            self.relayout();
            self.paint_step();
        }
        flush(&mut self.tree);
        self.relayout();
    }

    fn wheel(&mut self, delta: f32) {
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
        let scroll = self.scroll;
        self.tree.nodes[scroll].widget.handle_event(&event, rect);
        self.settle();
    }

    fn rows(&self) -> Vec<String> {
        let mut out = Vec::new();
        for &child in &self.tree.nodes[self.scroll].children {
            if self.tree.nodes[child].dom_id.is_some() {
                collect(&self.tree, child, &mut out);
            }
        }
        out
    }

    fn offset(&self) -> f32 {
        self.tree.nodes[self.scroll]
            .widget
            .scroll_offset()
            .map_or(0.0, |(_, y)| y)
    }
}

fn collect(tree: &WidgetTree, id: WidgetId, out: &mut Vec<String>) {
    if let Some(text) = (&*tree.nodes[id].widget as &dyn Any).downcast_ref::<TextWidget>() {
        out.push(text.text().to_string());
    }
    for &child in &tree.nodes[id].children {
        collect(tree, child, out);
    }
}

fn label(tree: &WidgetTree, scroll: WidgetId) -> String {
    let host = tree.nodes[scroll].parent.expect("host");
    let mut out = Vec::new();
    collect(tree, tree.nodes[host].children[0], &mut out);
    out.concat()
}

#[test]
fn a_chat_list_starts_pinned_to_the_newest_rows() {
    let chat = setup(1000);
    let rows = chat.rows();
    assert_eq!(rows.last().map(String::as_str), Some("row 999"), "{rows:?}");
    assert!(!rows.contains(&"row 0".to_string()));
    assert!(rows.len() < 40, "{} rows mounted", rows.len());
    let expected_bottom = chat.tree.layout(chat.scroll).size.height;
    assert!(expected_bottom > 0.0);
    assert!(chat.offset() > 1000.0, "scrolled to the end: {}", chat.offset());
}

#[test]
fn scrolling_up_anchors_on_a_row_and_moves_the_window() {
    let mut chat = setup(1000);
    chat.wheel(30_000.0);
    let rows = chat.rows();
    assert!(!rows.contains(&"row 999".to_string()), "{rows:?}");
    assert!(rows.len() < 60, "{} rows mounted", rows.len());
    let first = rows.first().expect("rows");
    let number: usize = first.trim_start_matches("row ").parse().expect("row number");
    assert!(number > 300 && number < 900, "{first}");

    chat.wheel(1_000_000.0);
    let rows = chat.rows();
    assert_eq!(rows.first().map(String::as_str), Some("row 0"), "{rows:?}");
}

#[test]
fn a_new_row_keeps_a_pinned_list_at_the_bottom() {
    let mut chat = setup(200);
    assert_eq!(chat.rows().last().map(String::as_str), Some("row 199"));
    chat.items.set(items(201));
    chat.settle();
    assert_eq!(chat.rows().last().map(String::as_str), Some("row 200"));
    let end = chat.offset();
    chat.items.set(items(203));
    chat.settle();
    assert_eq!(chat.rows().last().map(String::as_str), Some("row 202"));
    assert!(chat.offset() > end);
}

#[test]
fn an_anchored_list_stays_on_its_row_when_rows_arrive_below() {
    let mut chat = setup(300);
    chat.wheel(3_000.0);
    let before = chat.rows();
    assert!(!before.contains(&"row 299".to_string()));
    let anchor_row = before[before.len() / 2].clone();
    chat.items.set(items(305));
    chat.settle();
    let after = chat.rows();
    assert!(after.contains(&anchor_row), "{anchor_row} still mounted: {after:?}");
    assert!(!after.contains(&"row 304".to_string()), "not dragged to the bottom");
}

#[test]
fn measured_row_heights_are_reported() {
    let chat = setup_with(items_estimated(50, WRONG_ESTIMATE));
    let text = label(&chat.tree, chat.scroll);
    assert_eq!(text, format!("measured {ROW}"));
}

#[test]
fn a_chat_scroll_view_needs_item_height() {
    let source = r#"
import { ScrollView, Text } from "@requiem/ui";
export function Bad({ position }) {
  return (
    <ScrollView position={position}>
      <Text variant="text-md/normal">x</Text>
    </ScrollView>
  );
}
"#;
    let Err(error) = compile(source, &Options::default()) else {
        panic!("rejected");
    };
    assert_eq!(error[0].message, "`position` needs `itemHeight` and no `itemGap`");
}
