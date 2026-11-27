use std::any::Any;
use std::cell::RefCell;
use std::env;
use std::fs;
use std::rc::Rc;

use requiem_tsx::{Actions, Host, Options, Value, compile, mount, ui_actions};
use requiem_ui::event::{MouseButton, UiEvent};
use requiem_ui::font_system::TextFonts;
use requiem_ui::signal::{Signal, flush};
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::primitives::{BoxWidget, TextWidget};
use requiem_ui::widget::{AbsoluteRect, WidgetId, WidgetTree};

const SOURCE: &str = r#"
import { Text } from "@requiem/ui";
export function Counter({ step, label }) {
  const [n, setN] = useState(0);
  const next = n + step;
  const tone = next > 5 ? "high" : "low";
  return (
    <div className={tone} style={`width: ${next}px`} onClick={() => setN(c => c + step)}>
      <Text variant="text-md/normal">{label}: {next}</Text>
      {next > 5 && <Text variant="text-md/bold" className="warn">big</Text>}
    </div>
  );
}
"#;

fn setup() -> (WidgetTree, WidgetId, Signal<Value>) {
    TextFonts::install_new(1024 * 1024);
    let module = compile(SOURCE, &Options::default()).expect("compiles");
    let host = Host::inert();
    let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
    let parent = tree.root;
    let (step, label) = (Signal::new(Value::Num(2.0)), Signal::new(Value::str("n")));
    let root = mount(
        &mut tree,
        parent,
        &module,
        "Counter",
        &[("step", step.clone()), ("label", label)],
        &host,
    )
    .expect("Counter");
    (tree, root, step)
}

fn click(tree: &mut WidgetTree, id: WidgetId) {
    let event = UiEvent::MouseDown {
        x: 0.0,
        y: 0.0,
        button: MouseButton::Left,
        shift: false,
    };
    let rect = AbsoluteRect::default();
    tree.nodes[id].widget.handle_event(&event, rect);
    let release = UiEvent::MouseUp {
        x: 0.0,
        y: 0.0,
        button: MouseButton::Left,
    };
    tree.nodes[id].widget.handle_event(&release, rect);
}

fn style(tree: &WidgetTree, id: WidgetId) -> String {
    tree.nodes[id].inline_style_source.clone().unwrap_or_default()
}

#[test]
fn a_derived_local_follows_state_and_props_although_the_component_ran_once() {
    let (mut tree, root, step) = setup();
    assert_eq!(style(&tree, root), "width: 2px");
    assert_eq!(tree.nodes[root].classes, ["low"]);

    click(&mut tree, root);
    flush(&mut tree);
    assert_eq!(style(&tree, root), "width: 4px", "state changed, derived `next` followed");

    step.set(Value::Num(4.0));
    flush(&mut tree);
    assert_eq!(style(&tree, root), "width: 6px", "a prop changed, derived `next` followed");
    assert_eq!(tree.nodes[root].classes, ["high"], "a derived of a derived followed too");
}

const FRAGMENTS: &str = r#"
import { Text } from "@requiem/ui";
export function Panel({ open, onPick }) {
  return (
    <div>
      <Text variant="text-md/normal" className="before">before</Text>
      {open ? (
        <>
          <Text variant="text-md/normal" className="one">one</Text>
          <Text variant="text-md/normal" className="two">two</Text>
        </>
      ) : null}
      <Child onPick={(value) => onPick(value)} />
      <Text variant="text-md/normal" className="after">after</Text>
    </div>
  );
}

function Child({ onPick }) {
  return <div className="child" onClick={() => onPick(7)} />;
}
"#;

#[test]
fn a_fragment_branch_mounts_all_its_elements_between_siblings() {
    TextFonts::install_new(1024 * 1024);
    let module = compile(FRAGMENTS, &Options::default()).expect("compiles");
    let host = Host::inert();
    let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
    let parent = tree.root;
    let open = Signal::new(Value::Bool(false));
    let on_pick = Signal::new(Value::Null);
    let root = mount(
        &mut tree,
        parent,
        &module,
        "Panel",
        &[("open", open.clone()), ("onPick", on_pick)],
        &host,
    )
    .expect("Panel");
    let classes = |tree: &WidgetTree| -> Vec<String> {
        tree.nodes[root]
            .children
            .iter()
            .map(|id| tree.nodes[*id].classes.first().cloned().unwrap_or_default())
            .collect()
    };
    assert_eq!(classes(&tree), ["before", "child", "after"]);

    open.set(Value::Bool(true));
    flush(&mut tree);
    assert_eq!(classes(&tree), ["before", "one", "two", "child", "after"]);

    open.set(Value::Bool(false));
    flush(&mut tree);
    assert_eq!(classes(&tree), ["before", "child", "after"]);
}

#[test]
fn a_function_prop_called_from_a_child_runs_the_parents_closure() {
    TextFonts::install_new(1024 * 1024);
    let source = r#"
import { Text } from "@requiem/ui";
export function Parent() {
  const [seen, setSeen] = useState(0);
  return (
    <div>
      <Text variant="text-md/normal">{seen}</Text>
      <Child onPick={(value) => setSeen(seen + value)} />
    </div>
  );
}
function Child({ onPick }) {
  return <div className="child" onClick={() => onPick(7)} />;
}
"#;
    let module = compile(source, &Options::default()).expect("compiles");
    let host = Host::inert();
    let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
    let parent = tree.root;
    let root = mount(&mut tree, parent, &module, "Parent", &[], &host).expect("Parent");
    let child = tree.nodes[root].children[1];
    click(&mut tree, child);
    flush(&mut tree);
    click(&mut tree, child);
    flush(&mut tree);
    let text = tree.nodes[root].children[0];
    let shown = (&*tree.nodes[text].widget as &dyn Any)
        .downcast_ref::<TextWidget>()
        .map(|widget| widget.text().to_string());
    assert_eq!(shown.as_deref(), Some("14"));
}

#[test]
fn translations_are_checked_and_follow_their_arguments() {
    TextFonts::install_new(1024 * 1024);
    let source = r#"
import { Text } from "@requiem/ui";
export function Heading({ n }) {
  return <Text variant="text-md/normal">{t("threads-browser-joined", { count: n })}</Text>;
}
"#;
    let module = compile(source, &Options::default()).expect("compiles");
    let host = Host::inert();
    let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
    let parent = tree.root;
    let n = Signal::new(Value::Num(3.0));
    let root = mount(&mut tree, parent, &module, "Heading", &[("n", n.clone())], &host)
        .expect("Heading");
    let shown = |tree: &WidgetTree| {
        (&*tree.nodes[root].widget as &dyn Any)
            .downcast_ref::<TextWidget>()
            .map(|widget| widget.text().to_string())
            .unwrap_or_default()
    };
    assert_eq!(shown(&tree), "3 joined threads");
    n.set(Value::Num(5.0));
    flush(&mut tree);
    assert_eq!(shown(&tree), "5 joined threads");

    let unknown = r#"import { Text } from "@requiem/ui";
export function A() { return <Text variant="text-md/normal">{t("no-such-key-anywhere")}</Text>; }"#;
    let errors = compile(unknown, &Options::default()).expect_err("rejected");
    assert_eq!(errors[0].message, "unknown translation key `no-such-key-anywhere`");
}

#[test]
fn translation_arguments_must_match_the_message() {
    let compile_body = |body: &str| {
        let source = format!(
            "import {{ Text }} from \"@requiem/ui\";\nexport function A({{ n }}) {{ return <Text variant=\"text-md/normal\">{body}</Text>; }}"
        );
        compile(&source, &Options::default())
            .err()
            .map(|errors| (errors[0].message.clone(), errors[0].help.clone()))
    };
    assert_eq!(
        compile_body("{t(\"threads-browser-joined\")}"),
        Some(("`threads-browser-joined` needs the argument `count`".to_string(), None))
    );
    assert_eq!(
        compile_body("{t(\"threads-browser-joined\", { count: n, extra: n })}"),
        Some((
            "`threads-browser-joined` has no argument `extra`".to_string(),
            Some("its arguments: count".to_string())
        ))
    );
    assert_eq!(
        compile_body("{t(\"threads-browser-title\", { count: n })}"),
        Some((
            "`threads-browser-title` has no argument `count`".to_string(),
            Some("this message takes no arguments".to_string())
        ))
    );
    assert_eq!(compile_body("{t(\"threads-browser-title\")}"), None);
}

#[test]
fn the_full_translation_declarations_can_be_written_out_for_tsc() {
    let Some(path) = env::var_os("REQUIEM_WRITE_I18N_TYPES") else {
        return;
    };
    let text = requiem_tsx::declarations(
        &Default::default(),
        &[],
        &requiem_tsx::StateQueries::new(),
        &[],
        requiem_i18n::catalog(),
    );
    fs::write(path, text).expect("write declarations");
}

#[test]
fn the_translation_declarations_type_the_key_and_its_arguments() {
    let subset: Vec<requiem_i18n::Message> = requiem_i18n::catalog()
        .iter()
        .filter(|message| {
            ["threads-browser-joined", "threads-browser-title"].contains(&message.key.as_str())
        })
        .cloned()
        .collect();
    let text = requiem_tsx::declarations(
        &Default::default(),
        &[],
        &requiem_tsx::StateQueries::new(),
        &[],
        &subset,
    );
    assert!(text.contains("\"threads-browser-joined\": { count: string | number };"), "{text}");
    assert!(text.contains("\"threads-browser-title\": undefined;"), "{text}");
    assert!(text.contains("declare function t<K extends keyof Translations>("), "{text}");
}

ui_actions! {
    #[derive(Debug, PartialEq)]
    enum Menu {
        open(x: f64, y: f64, label: String) => Open,
    }
}

#[test]
fn a_context_menu_handler_receives_the_pointer_position() {
    TextFonts::install_new(1024 * 1024);
    let source = r#"
export function Row({ label }) {
  return <div onContextMenu={(event) => actions.open(event.x, event.y, label)} />;
}
"#;
    let options = Options {
        actions: Menu::specs(),
        ..Options::default()
    };
    let module = compile(source, &options).expect("compiles");
    let seen = Rc::new(RefCell::new(Vec::new()));
    let sink = seen.clone();
    let host = Host::new(move |action: Menu| sink.borrow_mut().push(action));
    let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
    let parent = tree.root;
    let label = Signal::new(Value::str("row"));
    let root = mount(&mut tree, parent, &module, "Row", &[("label", label)], &host).expect("Row");
    let handler = tree.nodes[root].context_menu.clone().expect("a context menu handler");
    handler(12.0, 34.0);
    assert_eq!(
        *seen.borrow(),
        vec![Menu::Open {
            x: 12.0,
            y: 34.0,
            label: "row".into()
        }]
    );
}

#[test]
fn a_conditional_child_between_siblings_mounts_in_place() {
    let (mut tree, root, step) = setup();
    assert_eq!(tree.nodes[root].children.len(), 1);

    step.set(Value::Num(9.0));
    flush(&mut tree);
    let kids = tree.nodes[root].children.clone();
    assert_eq!(kids.len(), 2);
    assert_eq!(tree.nodes[kids[1]].classes, ["warn"]);

    step.set(Value::Num(1.0));
    flush(&mut tree);
    assert_eq!(tree.nodes[root].children.len(), 1);
}
