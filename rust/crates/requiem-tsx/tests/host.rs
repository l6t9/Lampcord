use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use requiem_tsx::{
    Actions, Host, HostComponents, Options, Source, StateQueries, Value, compile_project,
    declarations, mount, ui_actions,
};
use requiem_ui::components::TextProps;
use requiem_ui::font_system::TextFonts;
use requiem_ui::signal::{Signal, create_effect, flush};
use requiem_ui::style::ComputedStyle;
use requiem_ui::typography::TextVariant;
use requiem_ui::widget::primitives::{BoxWidget, TextWidget};
use requiem_ui::widget::{WidgetId, WidgetTree};

ui_actions! {
    #[derive(Debug, PartialEq)]
    enum Picked {
        picked(label: String) => Picked,
    }
}

fn badges() -> HostComponents {
    HostComponents::new().component(
        "Badge",
        &[
            ("label", "string", true),
            ("onPick", "(label: string) => void", false),
        ],
        |props| {
            let node = TextProps::styled(props.peek("label").to_child_text(), TextVariant::TEXT_SM_NORMAL)
                .build();
            let props = props.clone();
            node.on_mount(move |tree, id| {
                let props = props.clone();
                let effect = create_effect(
                    move |tree| {
                        let label = props.get("label").to_child_text();
                        if let Some(node) = tree.nodes.get_mut(id) {
                            if let Some(widget) =
                                (&mut *node.widget as &mut dyn Any).downcast_mut::<TextWidget>()
                            {
                                widget.set_text(label.clone());
                            }
                        }
                        tree.mark_dirty(id);
                        props.call("onPick", &[Value::str(label)]);
                    },
                    tree,
                );
                tree.attach_effect(id, effect);
            })
        },
    )
}

const SOURCE: &str = r#"
import { Badge } from "@requiem/app";
export function Page({ label }) {
  return <div><Badge label={label} onPick={(picked) => actions.picked(picked)} /></div>;
}
"#;

fn shown(tree: &WidgetTree, id: WidgetId) -> String {
    (&*tree.nodes[id].widget as &dyn Any)
        .downcast_ref::<TextWidget>()
        .map(|widget| widget.text().to_string())
        .unwrap_or_default()
}

#[test]
fn a_host_component_builds_a_node_follows_its_props_and_calls_back() {
    TextFonts::install_new(1024 * 1024);
    let components = badges();
    let options = Options {
        host: components.specs(),
        actions: Picked::specs(),
        ..Options::default()
    };
    let module = compile_project(&[Source::new("Page.tsx", SOURCE)], &options)
        .unwrap_or_else(|errors| panic!("{:?}", errors.iter().map(|e| e.diagnostic.to_string()).collect::<Vec<_>>()));

    let seen = Rc::new(RefCell::new(Vec::new()));
    let sink = seen.clone();
    let host = Host::new(move |action: Picked| sink.borrow_mut().push(action));
    host.serve_components(&components);
    let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
    let parent = tree.root;
    let label = Signal::new(Value::str("hello"));
    let root = mount(&mut tree, parent, &module, "Page", &[("label", label.clone())], &host)
        .expect("Page");
    let badge = tree.nodes[root].children[0];
    assert_eq!(shown(&tree, badge), "hello");

    label.set(Value::str("again"));
    flush(&mut tree);
    assert_eq!(shown(&tree, badge), "again");
    assert_eq!(
        *seen.borrow(),
        vec![
            Picked::Picked { label: "hello".into() },
            Picked::Picked { label: "again".into() },
        ]
    );
}

#[test]
fn host_components_are_checked_when_the_tsx_compiles() {
    let components = badges();
    let options = Options {
        host: components.specs(),
        ..Options::default()
    };
    let compile = |body: &str| {
        let source = format!("import {{ Badge }} from \"@requiem/app\";\nexport function Page() {{ return {body}; }}");
        compile_project(&[Source::new("Page.tsx", &source)], &options)
            .err()
            .map(|errors| errors[0].diagnostic.message.clone())
    };
    assert_eq!(compile("<Badge />").as_deref(), Some("<Badge> needs the prop `label`"));
    assert_eq!(
        compile("<Badge label=\"x\" colour=\"red\" />").as_deref(),
        Some("<Badge> has no prop `colour`")
    );
    assert_eq!(compile("<Badge label=\"x\" />"), None);
}

#[test]
fn the_declarations_describe_host_components() {
    let text = declarations(&Default::default(), &[], &StateQueries::new(), &badges().specs(), &[]);
    assert!(text.contains("declare module \"@requiem/app\" {"), "{text}");
    assert!(text.contains("label: string;"), "{text}");
    assert!(text.contains("onPick?: (label: string) => void;"), "{text}");
    assert!(text.contains("export function Badge(props: BadgeProps): JSX.Element;"), "{text}");
}
