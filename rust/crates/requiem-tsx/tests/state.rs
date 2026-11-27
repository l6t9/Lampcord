use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use requiem_tsx::{Host, Options, StateQueries, Value, compile, declarations, mount};
use requiem_ui::font_system::TextFonts;
use requiem_ui::signal::{Signal, flush};
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::primitives::{BoxWidget, TextWidget};
use requiem_ui::widget::{WidgetId, WidgetTree};
use serde::Serialize;
use specta::Type;

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
struct Person {
    id: String,
    display_name: Option<String>,
    is_bot: bool,
}

struct People {
    names: Rc<RefCell<HashMap<String, Signal<String>>>>,
    reads: Rc<RefCell<HashMap<String, u32>>>,
}

impl People {
    fn new(ids: &[&str]) -> Self {
        let names = ids
            .iter()
            .map(|id| (id.to_string(), Signal::new(format!("person {id}"))))
            .collect();
        Self {
            names: Rc::new(RefCell::new(names)),
            reads: Rc::default(),
        }
    }

    fn queries(&self) -> StateQueries {
        let (names, reads) = (self.names.clone(), self.reads.clone());
        let by_id = move |args: &[Value]| {
            let Some(Value::Str(id)) = args.first() else {
                return None;
            };
            *reads.borrow_mut().entry(id.to_string()).or_default() += 1;
            let name = names.borrow().get(&**id)?.get();
            Some(Person {
                id: id.to_string(),
                display_name: Some(name),
                is_bot: false,
            })
        };
        StateQueries::new().one("people.get", &[("id", "string")], by_id)
    }
}

const SOURCE: &str = r#"
import { Text } from "@requiem/ui";
export function Card({ id }) {
  const person = state.people.get(id);
  return <Text variant="text-md/normal">{person.displayName}</Text>;
}
"#;

fn setup(people: &People, ids: &[&str]) -> (WidgetTree, Vec<WidgetId>) {
    TextFonts::install_new(1024 * 1024);
    let queries = people.queries();
    let options = Options {
        queries: queries.specs(),
        ..Options::default()
    };
    let module = compile(SOURCE, &options).expect("compiles");
    let host = Host::inert();
    host.serve(&queries);
    let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
    let parent = tree.root;
    let roots = ids
        .iter()
        .map(|id| {
            mount(
                &mut tree,
                parent,
                &module,
                "Card",
                &[("id", Signal::new(Value::str(id)))],
                &host,
            )
            .expect("Card")
        })
        .collect();
    (tree, roots)
}

fn shown(tree: &WidgetTree, id: WidgetId) -> String {
    (&*tree.nodes[id].widget as &dyn Any)
        .downcast_ref::<TextWidget>()
        .map(|widget| widget.text().to_string())
        .unwrap_or_default()
}

#[test]
fn a_query_reads_the_store_and_follows_its_signal() {
    let people = People::new(&["1", "2"]);
    let (mut tree, roots) = setup(&people, &["1", "2"]);
    assert_eq!(shown(&tree, roots[0]), "person 1");
    assert_eq!(shown(&tree, roots[1]), "person 2");

    people.names.borrow()["1"].set("renamed".into());
    flush(&mut tree);
    assert_eq!(shown(&tree, roots[0]), "renamed");
    assert_eq!(shown(&tree, roots[1]), "person 2");
}

#[test]
fn a_write_to_one_key_reruns_only_the_readers_of_that_key() {
    let people = People::new(&["1", "2"]);
    let (mut tree, roots) = setup(&people, &["1", "2"]);
    let before = people.reads.borrow().clone();

    people.names.borrow()["1"].set("renamed".into());
    flush(&mut tree);
    let after = people.reads.borrow().clone();
    assert_eq!(after["1"] - before["1"], 1, "the changed key's reader reran once");
    assert_eq!(after["2"] - before["2"], 0, "the other key's reader did not");
    assert_eq!(shown(&tree, roots[0]), "renamed");
}

#[test]
fn queries_are_checked_when_the_tsx_compiles() {
    let people = People::new(&["1"]);
    let options = Options {
        queries: people.queries().specs(),
        ..Options::default()
    };
    let import = "import { Text } from \"@requiem/ui\"; ";
    let unknown = format!("{import}export function A() {{ return <Text variant=\"text-md/normal\">{{state.people.nope(1)}}</Text>; }}");
    let errors = compile(&unknown, &options).expect_err("rejected");
    assert_eq!(errors[0].message, "`state.people.nope` is not a state query");
    assert_eq!(errors[0].help.as_deref(), Some("queries: people.get"));

    let arity = format!("{import}export function A() {{ return <Text variant=\"text-md/normal\">{{state.people.get()}}</Text>; }}");
    let errors = compile(&arity, &options).expect_err("rejected");
    assert_eq!(errors[0].message, "`state.people.get` takes 1 argument, not 0");
    assert_eq!(errors[0].help.as_deref(), Some("signature: get(id: string): Person | null"));
}

#[test]
fn the_declarations_come_from_the_rust_types() {
    let people = People::new(&[]);
    let text = declarations(&Default::default(), &[], &people.queries(), &[], &[]);
    assert!(text.contains("declare namespace Types {"), "{text}");
    assert!(text.contains("declare module \"@requiem/types\" {"), "{text}");
    assert!(text.contains("export type Person = {"), "{text}");
    assert!(text.contains("displayName: string | null"), "{text}");
    assert!(text.contains("isBot: boolean"), "{text}");
    assert!(text.contains("get(id: string): Types.Person | null;"), "{text}");
    assert!(text.contains("export type Person = Types.Person;"), "{text}");
    assert!(!text.contains("import(\""), "{text}");
    assert!(!text.contains("//"), "{text}");
}
