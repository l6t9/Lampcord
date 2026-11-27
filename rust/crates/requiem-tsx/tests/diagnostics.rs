use requiem_tsx::{ActionSpec, Options, compile};

fn options() -> Options {
    Options {
        actions: vec![ActionSpec::new("send").arg("value", "boolean")],
        ..Options::default()
    }
}

#[test]
fn an_action_call_must_match_the_registered_arity() {
    let source = with_imports(
        "export function A() {\n  return <Button onClick={() => actions.send()}>Go</Button>;\n}",
    );
    let errors = compile(&source, &options()).expect_err("rejected");
    assert_eq!(errors[0].message, "`send` takes 1 argument, not 0");
    assert_eq!(errors[0].help.as_deref(), Some("signature: send(value: boolean): void"));
}

fn with_imports(source: &str) -> String {
    format!("import {{ Text, Button, ScrollView, Stack, ThreadIcon, XSmallIcon }} from \"@requiem/ui\"; {source}")
}

fn first_error(source: &str) -> (u32, String) {
    let errors = compile(&with_imports(source), &options()).expect_err("source should be rejected");
    (errors[0].line, errors[0].message.clone())
}

#[track_caller]
fn rejects(source: &str, line: u32, fragment: &str) {
    let (got_line, message) = first_error(source);
    assert!(
        message.contains(fragment),
        "expected {fragment:?} in {message:?}"
    );
    assert_eq!(got_line, line, "{message}");
}

#[test]
fn a_missing_key_is_an_error() {
    rejects(
        "export function A({ xs }) {\n  return <div>{xs.map(x => <div><Text variant=\"text-md/normal\">{x}</Text></div>)}</div>;\n}",
        2,
        "need a `key`",
    );
}

#[test]
fn async_and_await_are_rejected() {
    rejects(
        "export async function A() {\n  return <div/>;\n}",
        1,
        "async",
    );
    rejects(
        "export function A() {\n  const x = await load();\n  return <div/>;\n}",
        2,
        "`await`",
    );
}

#[test]
fn loops_are_rejected() {
    rejects(
        "export function A({ xs }) {\n  for (const x of xs) {}\n  return <div/>;\n}",
        2,
        "not supported in a component body",
    );
    rejects(
        "export function A() {\n  const [n, setN] = useState(0);\n  return <div onClick={() => { while (true) {} }}/>;\n}",
        3,
        "loops are not supported",
    );
}

#[test]
fn writes_are_rejected() {
    rejects(
        "export function A() {\n  const [n, setN] = useState(0);\n  return <div onClick={() => n++}/>;\n}",
        3,
        "handlers may only call",
    );
    rejects(
        "export function A() {\n  const x = (state.count = 1);\n  return <div/>;\n}",
        2,
        "assignment is not supported",
    );
}

#[test]
fn unknown_actions_are_rejected_with_the_registered_names() {
    let errors = compile(
        "export function A() {\n  return <div onClick={() => actions.nuke()}/>;\n}",
        &options(),
    )
    .expect_err("rejected");
    assert_eq!(errors[0].message, "unknown action `nuke`");
    assert_eq!(errors[0].help.as_deref(), Some("registered actions: send"));
}

#[test]
fn hooks_other_than_use_state_are_rejected() {
    rejects(
        "export function A() {\n  useEffect(() => {}, []);\n  return <div/>;\n}",
        2,
        "`useEffect` is not part of the subset",
    );
}

#[test]
fn loose_equality_new_and_unknown_names_are_rejected() {
    rejects(
        "export function A({ a }) {\n  return <Text variant=\"text-md/normal\">{a == 1 ? \"x\" : \"y\"}</Text>;\n}",
        2,
        "loose equality",
    );
    rejects(
        "export function A() {\n  const d = new Date();\n  return <div/>;\n}",
        2,
        "`new` is not supported",
    );
    rejects(
        "export function A() {\n  return <Text variant=\"text-md/normal\">{missing}</Text>;\n}",
        2,
        "`missing` is not defined",
    );
}

#[test]
fn a_props_object_is_rejected_in_favour_of_destructuring() {
    rejects(
        "export function A(props) {\n  return <Text variant=\"text-md/normal\">{props.a}</Text>;\n}",
        1,
        "destructure props",
    );
}

#[test]
fn a_setter_called_during_render_is_rejected() {
    rejects(
        "export function A() {\n  const [n, setN] = useState(0);\n  return <Text variant=\"text-md/normal\">{setN(1)}</Text>;\n}",
        3,
        "this call is not supported",
    );
}

#[test]
fn syntax_errors_come_from_the_parser_with_their_line() {
    rejects("export function A() {\n  return <div>;\n}", 3, "");
}

#[test]
fn the_rendered_diagnostic_quotes_the_line_and_underlines_the_construct() {
    let source = with_imports(
        "export function A() {
  return <Text variant=\"text-md/normal\">{new Date()}</Text>;
}",
    );
    let errors = compile(&source, &options()).expect_err("rejected");
    let text = errors[0].render("a.tsx", &source);
    let column = source.lines().nth(1).unwrap().find("new").unwrap() + 1;
    assert!(text.starts_with(&format!("a.tsx:2:{column}: `new` is not supported")), "{text}");
    assert!(text.contains("{new Date()}</Text>;"), "{text}");
    assert!(text.contains("^^^^^^^^^^"), "{text}");
}

#[test]
fn a_text_variant_must_be_kind_size_slash_weight() {
    rejects(
        "export function A() {
  return <Text variant=\"text-md-normal\">hi</Text>;
}",
        2,
        "`text-md-normal` is not a text variant",
    );
    rejects(
        "export function A() {
  return <Text variant=\"TEXT_MD_NORMAL\">hi</Text>;
}",
        2,
        "is not a text variant",
    );
    rejects(
        "export function A() {
  return <Text variant=\"label-md/normal\">hi</Text>;
}",
        2,
        "is not a text variant",
    );
    rejects(
        "export function A() {
  return <Text>hi</Text>;
}",
        2,
        "needs a `variant`",
    );
}

#[test]
fn text_colors_and_button_props_are_checked_against_their_tables() {
    rejects(
        "export function A() {
  return <Text variant=\"text-md/normal\" color=\"red\">hi</Text>;
}",
        2,
        "is not a text color",
    );
    rejects(
        "export function A() {
  return <Button variant=\"huge\" label=\"x\" />;
}",
        2,
        "is not a button variant",
    );
    rejects(
        "export function A() {
  return <Button size=\"tiny\" />;
}",
        2,
        "is not a button size",
    );
}

#[test]
fn text_only_lives_inside_text() {
    rejects(
        "export function A() {
  return <div>hello</div>;
}",
        2,
        "text must be inside <Text>",
    );
    rejects(
        "export function A({ n }) {
  return <div>{n}</div>;
}",
        2,
        "cannot be a child",
    );
    rejects(
        "export function A() {
  return <span>hi</span>;
}",
        2,
        "<span> is not supported",
    );
}

#[test]
fn components_must_be_imported_and_known() {
    let errors = compile(
        "export function A() {
  return <Text variant=\"text-md/normal\">hi</Text>;
}",
        &options(),
    )
    .expect_err("rejected");
    assert!(errors[0].message.contains("is not an engine component"), "{}", errors[0]);
    rejects(
        "export function A() {
  return <Chip />;
}",
        2,
        "is not an engine component",
    );
}

#[test]
fn css_module_classes_are_checked() {
    let mut css = requiem_tsx::CssModules::default();
    css.insert("card", "row-left", "card-row-left_x");
    let options = Options {
        css_modules: css,
        ..Options::default()
    };
    let ok = "import cls from \"./card.module.css\"; export function A() { return <div className={cls.rowLeft} />; }";
    assert!(compile(ok, &options).is_ok());
    let bad = "import cls from \"./card.module.css\";
export function A() {
  return <div className={cls.rowRight} />;
}";
    let errors = compile(bad, &options).expect_err("rejected");
    assert_eq!(errors[0].line, 3);
    assert_eq!(errors[0].message, "`card.module.css` has no class `rowRight`");
    let missing = "import cls from \"./nope.module.css\"; export function A() { return <div />; }";
    assert!(compile(missing, &options).expect_err("rejected")[0].message.contains("no CSS module named `nope`"));
}

#[test]
fn icons_are_components_named_after_the_icon() {
    rejects(
        "export function A() {
  return <ThreadIcon name=\"thread\" />;
}",
        2,
        "attribute `name` is not supported here",
    );
    rejects(
        "export function A() {
  return <ThreadIcon color=\"red\" />;
}",
        2,
        "is not a color",
    );
    rejects(
        "export function A() {\n  return <ThreadIcon width={0} />;\n}",
        2,
        "must be a positive number of pixels",
    );
    assert!(
        compile(
            &with_imports("export function A() { return <XSmallIcon width={16} height={16} color=\"text-muted\" />; }"),
            &options()
        )
        .is_ok()
    );
    let errors = compile(
        "import { NopeIcon } from \"@requiem/ui\"; export function A() { return <NopeIcon />; }",
        &options(),
    )
    .expect_err("rejected");
    assert!(errors[0].message.contains("is not available yet"), "{}", errors[0]);
}

#[test]
fn a_button_holds_text_or_elements_not_both() {
    rejects(
        "export function A() {\n  return <Button>Save<ThreadIcon /></Button>;\n}",
        2,
        "text or elements, not both",
    );
    assert!(
        compile(&with_imports("export function A() { return <Button>Save</Button>; }"), &options())
            .is_ok()
    );
}

#[test]
fn same_module_components_check_their_props() {
    let source = "export function A() {\n  return <div><B x={1} y={2} /></div>;\n}\nfunction B({ x }) {\n  return <Text variant=\"text-md/normal\">{x}</Text>;\n}";
    let errors = compile(&with_imports(source), &options()).expect_err("rejected");
    assert_eq!(errors[0].message, "<B> has no prop `y`");
    assert_eq!(errors[0].help.as_deref(), Some("its props: x"));
    rejects(
        "export function A() {\n  return <B>child</B>;\n}\nfunction B() {\n  return <div />;\n}",
        2,
        "cannot take children",
    );
}

#[test]
fn windows_line_endings_do_not_turn_layout_whitespace_into_text() {
    let source = "import { Text } from \"@requiem/ui\";\r\nexport function A() {\r\n  return (\r\n    <div>\r\n      <Text variant=\"text-md/normal\">hi</Text>\r\n    </div>\r\n  );\r\n}\r\n";
    assert!(compile(source, &options()).is_ok());
}

#[test]
fn stack_props_are_checked_against_their_tables() {
    rejects(
        "export function A() {
  return <Stack direction=\"diagonal\" />;
}",
        2,
        "is not a stack direction",
    );
    rejects(
        "export function A() {
  return <Stack align=\"middle\" />;
}",
        2,
        "is not a stack alignment",
    );
    rejects(
        "export function A() {
  return <Stack gap={-1} />;
}",
        2,
        "`gap` must be a number of pixels",
    );
    assert!(
        compile(
            &with_imports("export function A() { return <Stack direction=\"horizontal\" align=\"center\" gap={4} />; }"),
            &options()
        )
        .is_ok()
    );
}

#[test]
fn only_a_button_click_receives_an_event() {
    rejects(
        "export function A() {\n  return <div onClick={(event) => actions.send(event)} />;\n}",
        2,
        "this handler cannot take an event",
    );
    assert!(
        compile(
            &with_imports("export function A() { return <Button onClick={(event) => actions.send(event.shiftKey)}>Go</Button>; }"),
            &options()
        )
        .is_ok()
    );
}

#[test]
fn a_function_prop_must_exist_to_be_called_or_forwarded() {
    rejects(
        "export function A() {\n  return <Button onClick={nope}>Go</Button>;\n}",
        2,
        "`nope` is not a function prop",
    );
    rejects(
        "export function A() {\n  return <Button onClick={() => nope(1)}>Go</Button>;\n}",
        2,
        "is not a state setter, function prop or action",
    );
}

#[test]
fn props_may_be_typed_inline_or_by_a_named_interface() {
    let inline = "export function A({ n }: { n: number }) { return <Text variant=\"text-md/normal\">{n}</Text>; }";
    let named = "interface AProps { n: number }\nexport function A({ n }: AProps) { return <Text variant=\"text-md/normal\">{n}</Text>; }";
    assert!(compile(&with_imports(inline), &options()).is_ok());
    assert!(compile(&with_imports(named), &options()).is_ok());
}
