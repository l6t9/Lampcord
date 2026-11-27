use std::any::Any;

use requiem_tsx::{FileDiagnostic, Host, Module, Options, Source, Value, compile_project, mount};
use requiem_ui::font_system::TextFonts;
use requiem_ui::signal::Signal;
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::primitives::{BoxWidget, TextWidget};
use requiem_ui::widget::{WidgetId, WidgetTree};

fn build(files: &[(&str, &str)]) -> Result<Module, Vec<FileDiagnostic>> {
    let sources: Vec<Source> = files.iter().map(|(path, text)| Source::new(path, text)).collect();
    compile_project(&sources, &Options::default())
}

fn errors(files: &[(&str, &str)]) -> Vec<(String, String)> {
    build(files)
        .expect_err("rejected")
        .into_iter()
        .map(|error| (error.path, error.diagnostic.message))
        .collect()
}

fn shown(tree: &WidgetTree, id: WidgetId) -> String {
    (&*tree.nodes[id].widget as &dyn Any)
        .downcast_ref::<TextWidget>()
        .map(|widget| widget.text().to_string())
        .unwrap_or_default()
}

const UI: &str = "import { Text } from \"@requiem/ui\";\n";

#[test]
fn a_named_import_resolves_to_the_component_in_the_other_file() {
    TextFonts::install_new(1024 * 1024);
    let row = format!("{UI}export function Row({{ label }}) {{ return <Text variant=\"text-md/normal\">{{label}}</Text>; }}");
    let list = format!(
        "{UI}import {{ Row }} from \"./Row\";\nexport function List() {{ return <div><Row label=\"hello\" /></div>; }}"
    );
    let module = build(&[("List.tsx", &list), ("Row.tsx", &row)]).expect("compiles");
    let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
    let parent = tree.root;
    let root = mount(&mut tree, parent, &module, "List", &[], &Host::inert()).expect("List");
    let text = tree.nodes[root].children[0];
    assert_eq!(shown(&tree, text), "hello");
}

#[test]
fn default_exports_directories_and_type_imports_resolve() {
    TextFonts::install_new(1024 * 1024);
    let card = format!(
        "{UI}import type {{ Item }} from \"../types\";\nexport default function Card({{ item }}: {{ item: Item }}) {{ return <Text variant=\"text-md/normal\">{{item.name}}</Text>; }}"
    );
    let types = "export interface Item { name: string }";
    let page = format!(
        "{UI}import Card from \"./cards/Card\";\nimport {{ Item }} from \"./types\";\nexport function Page({{ item }}: {{ item: Item }}) {{ return <div><Card item={{item}} /></div>; }}"
    );
    let module = build(&[
        ("pages/Page.tsx", &page),
        ("pages/cards/Card.tsx", &card),
        ("pages/types.ts", types),
    ])
    .expect("compiles");
    let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
    let parent = tree.root;
    let item = Signal::new(Value::obj([("name", "Ana".into())]));
    let root = mount(&mut tree, parent, &module, "Page", &[("item", item)], &Host::inert())
        .expect("Page");
    assert_eq!(shown(&tree, tree.nodes[root].children[0]), "Ana");
}

#[test]
fn two_files_may_define_a_component_with_the_same_name() {
    TextFonts::install_new(1024 * 1024);
    let a = format!("{UI}export function Label() {{ return <Text variant=\"text-md/normal\">from a</Text>; }}");
    let b = format!("{UI}export function Label() {{ return <Text variant=\"text-md/normal\">from b</Text>; }}");
    let page = format!(
        "{UI}import {{ Label }} from \"./a\";\nimport {{ Label as Other }} from \"./b\";\nexport function Page() {{ return <div><Label /><Other /></div>; }}"
    );
    let module = build(&[("Page.tsx", &page), ("a.tsx", &a), ("b.tsx", &b)]).expect("compiles");
    let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
    let parent = tree.root;
    let root = mount(&mut tree, parent, &module, "Page", &[], &Host::inert()).expect("Page");
    let kids = tree.nodes[root].children.clone();
    assert_eq!(shown(&tree, kids[0]), "from a");
    assert_eq!(shown(&tree, kids[1]), "from b");
}

#[test]
fn a_missing_module_or_export_is_reported_in_the_importing_file() {
    let row = format!("{UI}export function Row() {{ return <div />; }}");
    let missing = errors(&[("List.tsx", "import { Row } from \"./Nope\";"), ("Row.tsx", &row)]);
    assert_eq!(missing, [("List.tsx".to_string(), "cannot find module \"./Nope\"".to_string())]);

    let wrong = errors(&[("List.tsx", "import { Cell } from \"./Row\";"), ("Row.tsx", &row)]);
    assert_eq!(wrong[0].0, "List.tsx");
    assert!(wrong[0].1.contains("exports no component `Cell`"), "{wrong:?}");
}

#[test]
fn an_error_inside_an_imported_file_names_that_file() {
    let bad = format!("{UI}export function Row() {{ return <Text variant=\"nope\">x</Text>; }}");
    let list = format!(
        "{UI}import {{ Row }} from \"./Row\";\nexport function List() {{ return <div><Row /></div>; }}"
    );
    let found = errors(&[("List.tsx", &list), ("Row.tsx", &bad)]);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found.iter().all(|(path, _)| path == "Row.tsx"), "{found:?}");
    assert_eq!(found[0].0, "Row.tsx");
    assert!(found[0].1.contains("is not a text variant"));
}

#[test]
fn props_are_checked_against_the_imported_components_declaration() {
    let row = format!("{UI}export function Row({{ label }}) {{ return <Text variant=\"text-md/normal\">{{label}}</Text>; }}");
    let list = format!(
        "{UI}import {{ Row }} from \"./Row\";\nexport function List() {{ return <div><Row nope=\"x\" /></div>; }}"
    );
    let found = errors(&[("List.tsx", &list), ("Row.tsx", &row)]);
    assert_eq!(found, [("List.tsx".to_string(), "<Row> has no prop `nope`".to_string())]);
}
