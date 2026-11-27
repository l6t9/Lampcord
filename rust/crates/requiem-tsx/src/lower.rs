use std::collections::HashMap;
use std::rc::Rc;

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, ChainElement, ArrowFunctionBody, ArrowFunctionExpression, BindingPattern, Declaration,
    ExportDefaultDeclarationKind, Expression, Function, JSXAttributeItem, JSXAttributeName,
    JSXAttributeValue, JSXChild, JSXElement, JSXElementName, ObjectPropertyKind,
    PropertyKey, Statement, VariableDeclarationKind,
};
use oxc_ast::ast::{BinaryOperator, LogicalOperator, UnaryOperator};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};

use std::collections::{BTreeSet, HashSet};
use std::slice;
use std::sync::Mutex;

use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier, ImportOrExportKind};
use requiem_ui::components::{Align, ButtonSize, ButtonVariant, Direction, Justify};
use requiem_ui::icon_handle::Icon as IconHandle;
use requiem_ui::typography::TextVariant;

use crate::actions::ActionSpec;
use crate::host::HostSpec;
use crate::state::QuerySpec;
use crate::components::{
    BUTTON_SIZES, BUTTON_VARIANTS, STACK_ALIGNS, STACK_DIRECTIONS, STACK_JUSTIFIES, TEXT_COLORS,
    APP_MODULE, TYPES_MODULE, UI_COMPONENTS, UI_MODULE, icon_named, named, names, text_color,
};
use crate::css::CssModules;
use crate::diag::Diagnostic;
use crate::project::{Exported, FileIndex, ProjectIndex, Source, compile_project, resolve};
use crate::ir::{
    Attr, BinOp, ButtonEl, ChatSpec, ChevronEl, Child, Component, Decl, Div, Element, Expr, Handler, HostEl,
    IconEl, ItemHeight, Method, Part, ScrollEl, StackEl, Stmt, TextEl, UseEl, Window,
};
use crate::value::Value;

#[derive(Default)]
pub struct Options {
    pub actions: Vec<ActionSpec>,
    pub queries: Vec<QuerySpec>,
    pub host: Vec<HostSpec>,
    pub css_modules: CssModules,
}

#[derive(Clone, Debug)]
pub struct Module {
    pub components: Rc<Vec<Component>>,
}

impl Module {
    pub fn component(&self, name: &str) -> Option<&Component> {
        self.components
            .iter()
            .find(|c| c.name == name)
            .or_else(|| {
                self.components
                    .iter()
                    .find(|c| c.name.rsplit('/').next() == Some(name))
            })
    }
}

pub fn compile(source: &str, options: &Options) -> Result<Module, Vec<Diagnostic>> {
    compile_project(&[Source::new("main.tsx", source)], options)
        .map_err(|errors| errors.into_iter().map(|error| error.diagnostic).collect())
}

fn parse_errors(source: &str, parsed: &oxc_parser::ParserReturn) -> Vec<Diagnostic> {
    parsed
        .diagnostics
        .errors()
        .map(|error| {
            let (start, end) = error
                .labels
                .first()
                .map_or((0, 0), |label| (label.offset(), label.offset() + label.len()));
            Diagnostic::new(source, start, end, error.message.to_string())
        })
        .collect()
}

fn function_props(function: &Function) -> Vec<String> {
    let mut props = Vec::new();
    if let Some(BindingPattern::ObjectPattern(pattern)) =
        function.params.items.first().map(|param| &param.pattern)
    {
        for property in &pattern.properties {
            if let PropertyKey::StaticIdentifier(key) = &property.key {
                props.push(key.name.to_string());
            }
        }
    }
    props
}

pub(crate) fn scan_file(source: &str, path: &str) -> Result<FileIndex, Vec<Diagnostic>> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::tsx()).parse();
    if parsed.diagnostics.has_errors() {
        return Err(parse_errors(source, &parsed));
    }
    let mut file = FileIndex::default();
    let mut export = |name: &str, function: &Function, default: bool| {
        let found = Exported {
            key: Rc::from(format!("{path}/{name}")),
            props: function_props(function),
        };
        if default {
            file.exports.insert("default".to_string(), found.clone());
        }
        file.exports.insert(name.to_string(), found);
    };
    for statement in &parsed.program.body {
        match statement {
            Statement::ExportDeclaration(declaration) => match &declaration.declaration {
                Declaration::FunctionDeclaration(function) => {
                    if let Some(id) = &function.id {
                        export(id.name.as_str(), function, false);
                    }
                }
                Declaration::TSInterfaceDeclaration(interface) => {
                    file.types.insert(interface.id.name.to_string());
                }
                Declaration::TSTypeAliasDeclaration(alias) => {
                    file.types.insert(alias.id.name.to_string());
                }
                _ => {}
            },
            Statement::ExportDefaultDeclaration(declaration) => {
                if let ExportDefaultDeclarationKind::FunctionDeclaration(function) =
                    &declaration.declaration
                {
                    if let Some(id) = &function.id {
                        export(id.name.as_str(), function, true);
                    }
                }
            }
            _ => {}
        }
    }
    Ok(file)
}

pub(crate) fn lower_file(
    source: &str,
    path: &str,
    index: &ProjectIndex,
    options: &Options,
) -> Result<Vec<Component>, Vec<Diagnostic>> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::tsx()).parse();
    if parsed.diagnostics.has_errors() {
        return Err(parse_errors(source, &parsed));
    }
    let mut lowerer = Lowerer {
        source,
        file: path,
        index,
        options,
        diags: Vec::new(),
        ui_imports: HashSet::new(),
        app_imports: HashSet::new(),
        css_imports: HashMap::new(),
        signatures: HashMap::new(),
    };
    for statement in &parsed.program.body {
        lowerer.collect_signature(statement);
    }
    let mut components = Vec::new();
    for statement in &parsed.program.body {
        lowerer.top_level(statement, &mut components);
    }
    if lowerer.diags.is_empty() {
        Ok(components)
    } else {
        Err(lowerer.diags)
    }
}

#[derive(Default)]
struct Scope {
    vars: Vec<String>,
    setters: HashMap<String, usize>,
}

impl Scope {
    fn lookup(&self, name: &str) -> Option<usize> {
        self.vars.iter().rposition(|var| var == name)
    }
}

struct Lowerer<'s> {
    source: &'s str,
    file: &'s str,
    index: &'s ProjectIndex,
    options: &'s Options,
    diags: Vec<Diagnostic>,
    ui_imports: HashSet<String>,
    app_imports: HashSet<String>,
    css_imports: HashMap<String, String>,
    signatures: HashMap<String, Exported>,
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Div,
    Text,
    Button,
    Icon(IconHandle),
    Chevron,
    Scroll,
    Stack,
    Use,
    Host,
}

struct Raw<'x> {
    name: &'x str,
    name_span: Span,
    span: Span,
    value: Option<&'x JSXAttributeValue<'x>>,
}

type Lowered<T> = Option<T>;

impl Lowerer<'_> {
    fn error(&mut self, span: Span, message: impl Into<String>) {
        self.diags
            .push(Diagnostic::new(self.source, span.start, span.end, message));
    }

    fn error_help(&mut self, span: Span, message: impl Into<String>, help: impl Into<String>) {
        self.diags.push(
            Diagnostic::new(self.source, span.start, span.end, message).help(help),
        );
    }

    fn top_level(&mut self, statement: &Statement, components: &mut Vec<Component>) {
        match statement {
            Statement::ImportDeclaration(import) => self.import(import),
            Statement::ExportDeclaration(export) => match &export.declaration {
                Declaration::FunctionDeclaration(function) => {
                    components.extend(self.component(function));
                }
                Declaration::TSTypeAliasDeclaration(_) | Declaration::TSInterfaceDeclaration(_) => {}
                _ => self.error(export.span, "only `export function Component` is supported here"),
            },
            Statement::ExportDefaultDeclaration(export) => match &export.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
                    components.extend(self.component(function));
                }
                _ => self.error(export.span, "only `export default function Component` is supported here"),
            },
            Statement::FunctionDeclaration(function) => {
                components.extend(self.component(function));
            }
            Statement::TSTypeAliasDeclaration(_) | Statement::TSInterfaceDeclaration(_) => {}
            other => self.error_help(
                other.span(),
                "this top-level statement is not supported",
                "a UI module holds imports, type declarations and component functions",
            ),
        }
    }

    fn collect_signature(&mut self, statement: &Statement) {
        let function = match statement {
            Statement::FunctionDeclaration(function) => function,
            Statement::ExportDeclaration(export) => match &export.declaration {
                Declaration::FunctionDeclaration(function) => function,
                _ => return,
            },
            Statement::ExportDefaultDeclaration(export) => match &export.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(function) => function,
                _ => return,
            },
            _ => return,
        };
        let Some(name) = function.id.as_ref() else {
            return;
        };
        self.signatures.insert(
            name.name.to_string(),
            Exported {
                key: Rc::from(format!("{}/{}", self.file, name.name)),
                props: function_props(function),
            },
        );
    }

    fn relative_import(&mut self, import: &ImportDeclaration) {
        let source = import.source.value.as_str();
        if import.import_kind == ImportOrExportKind::Type {
            return;
        }
        let target = resolve(self.file, source);
        let Some(file) = self.index.files.get(&target) else {
            self.error_help(
                import.span,
                format!("cannot find module \"{source}\""),
                "relative imports name another UI source file in the project",
            );
            return;
        };
        let specifiers: &[ImportDeclarationSpecifier] =
            import.specifiers.as_ref().map_or(&[], |all| all.as_slice());
        for specifier in specifiers {
            let (local, wanted, span, type_only) = match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(named) => (
                    named.local.name.to_string(),
                    named.imported.name().to_string(),
                    named.span,
                    named.import_kind == ImportOrExportKind::Type,
                ),
                ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => (
                    default.local.name.to_string(),
                    "default".to_string(),
                    default.span,
                    false,
                ),
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace) => {
                    self.error(namespace.span, "namespace imports are not supported");
                    continue;
                }
            };
            if type_only {
                continue;
            }
            match file.exports.get(&wanted) {
                Some(found) => {
                    self.signatures.insert(local, found.clone());
                }
                None if file.types.contains(&wanted) => {}
                None => self.error_help(
                    span,
                    format!("\"{source}\" exports no component `{wanted}`"),
                    "import an exported function component or a type",
                ),
            }
        }
    }

    fn import(&mut self, import: &ImportDeclaration) {
        let source = import.source.value.as_str();
        let specifiers: &[ImportDeclarationSpecifier] =
            import.specifiers.as_ref().map_or(&[], |all| all.as_slice());
        if source == "react" || source == TYPES_MODULE {
            return;
        }
        if source == APP_MODULE {
            for specifier in specifiers {
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(named) => {
                        self.app_imports.insert(named.local.name.to_string());
                    }
                    other => self.error(other.span(), "import app components by name: { Avatar }"),
                }
            }
            return;
        }
        if source == UI_MODULE {
            for specifier in specifiers {
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(named) => {
                        self.ui_imports.insert(named.local.name.to_string());
                    }
                    other => self.error(other.span(), "import components by name: { Text, Button }"),
                }
            }
            return;
        }
        if let Some(file) = source.rsplit('/').next().and_then(|f| f.strip_suffix(".module.css")) {
            for specifier in specifiers {
                match specifier {
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                        if self.options.css_modules.module(file).is_none() {
                            self.error_help(
                                import.span,
                                format!("no CSS module named `{file}`"),
                                "the file must exist in the crate as <name>.module.css",
                            );
                        }
                        self.css_imports.insert(default.local.name.to_string(), file.to_string());
                    }
                    other => self.error(other.span(), "import a CSS module as a default: import cls from \"...\""),
                }
            }
            return;
        }
        if source.starts_with('.') {
            self.relative_import(import);
            return;
        }
        self.error_help(
            import.span,
            format!("cannot import from \"{source}\""),
            format!("UI sources import `react`, `{UI_MODULE}` and *.module.css files"),
        );
    }

    fn component(&mut self, function: &Function) -> Lowered<Component> {
        let name = function.id.as_ref()?.name.to_string();
        let diags_before = self.diags.len();
        if function.r#async || function.generator {
            self.error_help(
                function.span,
                "components cannot be async or generators",
                "load data in Rust and read it from `state`",
            );
            return None;
        }
        let mut scope = Scope::default();
        let mut props = Vec::new();
        match function.params.items.as_slice() {
            [] => {}
            [param] => match &param.pattern {
                BindingPattern::ObjectPattern(pattern) => {
                    if let Some(rest) = &pattern.rest {
                        self.error(rest.span, "rest props are not supported");
                    }
                    for property in &pattern.properties {
                        let (PropertyKey::StaticIdentifier(key), true) =
                            (&property.key, !property.computed)
                        else {
                            self.error(property.span, "props must be plain names");
                            continue;
                        };
                        let BindingPattern::BindingIdentifier(binding) = &property.value else {
                            self.error_help(
                                property.span,
                                "a prop cannot have a default or be destructured further",
                                "pass the default from the caller",
                            );
                            continue;
                        };
                        props.push(Rc::<str>::from(key.name.as_str()));
                        scope.vars.push(binding.name.to_string());
                    }
                }
                other => self.error_help(
                    other.span(),
                    "destructure props in the parameter list",
                    "write `function Name({ a, b }: Props)`; a `props` object is not available",
                ),
            },
            [_, extra, ..] => {
                self.error(extra.span, "a component takes a single props parameter")
            }
        }

        let body = function.body.as_ref()?;
        let mut decls = Vec::new();
        let mut root = None;
        for statement in &body.statements {
            match statement {
                Statement::VariableDeclaration(declaration) => {
                    if declaration.kind != VariableDeclarationKind::Const {
                        self.error_help(
                            declaration.span,
                            "components declare locals with `const`",
                            "mutable state is `useState`; reassignment is not supported",
                        );
                        continue;
                    }
                    for declarator in &declaration.declarations {
                        if let Some(decl) = self.declarator(declarator, &mut scope) {
                            decls.push(decl);
                        }
                    }
                }
                Statement::ReturnStatement(ret) => {
                    let Some(argument) = &ret.argument else {
                        self.error(ret.span, "a component returns an element");
                        continue;
                    };
                    match argument.get_inner_expression() {
                        Expression::JSXElement(element) => {
                            if let Some((element, _)) = self.element(element, &mut scope, false) {
                                root = Some(element);
                            }
                        }
                        other => self.error_help(
                            other.span(),
                            "a component returns a single element",
                            "wrap fragments and conditionals in a <div>; the root cannot appear and disappear",
                        ),
                    }
                }
                Statement::ExpressionStatement(expression) => {
                    self.statement_expression(expression.expression.get_inner_expression());
                }
                other => self.error_help(
                    other.span(),
                    "this statement is not supported in a component body",
                    "bodies hold `const` locals, `useState`, and one `return <jsx/>`; loops become `.map` in JSX",
                ),
            }
        }
        let root = match root {
            Some(root) => root,
            None => {
                if self.diags.len() == diags_before {
                    self.error(function.span, format!("`{name}` has no `return <element/>`"));
                }
                return None;
            }
        };
        Some(Component {
            name: format!("{}/{name}", self.file),
            props,
            decls,
            root,
        })
    }

    fn statement_expression(&mut self, expression: &Expression) {
        if let Expression::CallExpression(call) = expression {
            if let Expression::Identifier(callee) = call.callee.get_inner_expression() {
                if callee.name.starts_with("use") {
                    self.error_help(
                        call.span,
                        format!("`{}` is not part of the subset yet", callee.name),
                        "only `useState` is supported; derive values with `const` and let the runtime track reads",
                    );
                    return;
                }
            }
        }
        self.error(expression.span(), "a bare expression statement does nothing here");
    }

    fn declarator(
        &mut self,
        declarator: &oxc_ast::ast::VariableDeclarator,
        scope: &mut Scope,
    ) -> Lowered<Decl> {
        let Some(init) = &declarator.init else {
            self.error(declarator.span, "a local needs an initializer");
            return None;
        };
        match &declarator.id {
            BindingPattern::ArrayPattern(pattern) => {
                let is_use_state = matches!(
                    init.get_inner_expression(),
                    Expression::CallExpression(call)
                        if matches!(call.callee.get_inner_expression(),
                            Expression::Identifier(id) if id.name == "useState")
                );
                if !is_use_state {
                    self.error_help(
                        pattern.span,
                        "array destructuring is only supported for `useState`",
                        "index the array instead: `const first = list[0]`",
                    );
                    return None;
                }
                let Expression::CallExpression(call) = init.get_inner_expression() else {
                    return None;
                };
                let init = match call.arguments.as_slice() {
                    [] => Expr::Lit(Value::Null),
                    [argument] => {
                        let expression = self.argument_expression(argument)?;
                        self.expr(expression, scope)?
                    }
                    [_, extra, ..] => {
                        self.error(extra.span(), "useState takes at most one argument");
                        return None;
                    }
                };
                let names: Vec<Option<&oxc_ast::ast::BindingIdentifier>> = pattern
                    .elements
                    .iter()
                    .map(|element| match element {
                        Some(BindingPattern::BindingIdentifier(id)) => Some(&**id),
                        _ => None,
                    })
                    .collect();
                let (Some(Some(value)), setter) = (names.first(), names.get(1).copied().flatten())
                else {
                    self.error(pattern.span, "name the state value: `const [x, setX] = useState(..)`");
                    return None;
                };
                let slot = scope.vars.len();
                scope.vars.push(value.name.to_string());
                if let Some(setter) = setter {
                    scope.setters.insert(setter.name.to_string(), slot);
                }
                Some(Decl::State { init })
            }
            BindingPattern::BindingIdentifier(binding) => {
                let expr = self.expr(init, scope)?;
                scope.vars.push(binding.name.to_string());
                Some(Decl::Derived { expr })
            }
            other => {
                self.error_help(
                    other.span(),
                    "object destructuring of locals is not supported",
                    "read fields where you use them: `user.name`",
                );
                None
            }
        }
    }

    fn argument_expression<'a>(&mut self, argument: &'a Argument<'a>) -> Option<&'a Expression<'a>> {
        match argument.as_expression() {
            Some(expression) => Some(expression),
            None => {
                self.error(argument.span(), "spread arguments are not supported");
                None
            }
        }
    }

    fn translate(&mut self, call: &oxc_ast::ast::CallExpression, scope: &Scope) -> Lowered<Expr> {
        let key = match call.arguments.first().and_then(|argument| argument.as_expression()) {
            Some(expression) => match expression.get_inner_expression() {
                Expression::StringLiteral(text) => text.value.to_string(),
                other => {
                    self.error_help(
                        other.span(),
                        "the translation key must be a string literal",
                        "write t(\"threads-browser-title\")",
                    );
                    return None;
                }
            },
            None => {
                self.error(call.span, "t() needs a translation key");
                return None;
            }
        };
        let Some(message) = requiem_i18n::message(&key) else {
            self.error_help(
                call.span,
                format!("unknown translation key `{key}`"),
                "keys are the message ids in the locale files",
            );
            return None;
        };
        let mut args = Vec::new();
        match call.arguments.get(1).and_then(|argument| argument.as_expression()) {
            None => {}
            Some(expression) => match expression.get_inner_expression() {
                Expression::ObjectExpression(object) => {
                    for property in &object.properties {
                        let ObjectPropertyKind::ObjectProperty(property) = property else {
                            self.error(property.span(), "translation arguments cannot be spread");
                            return None;
                        };
                        let PropertyKey::StaticIdentifier(name) = &property.key else {
                            self.error(property.span, "translation argument names must be plain names");
                            return None;
                        };
                        args.push((intern(name.name.as_str()), self.expr(&property.value, scope)?));
                    }
                }
                other => {
                    self.error_help(
                        other.span(),
                        "translation arguments must be an object literal",
                        "write t(\"threads-browser-joined\", { count: joined.length })",
                    );
                    return None;
                }
            },
        }
        if call.arguments.len() > 2 {
            self.error(call.span, "t() takes a key and one object of arguments");
            return None;
        }
        let mut ok = true;
        for (name, _) in &args {
            if !message.args.iter().any(|arg| arg.name == *name) {
                let known: Vec<&str> = message.args.iter().map(|arg| arg.name.as_str()).collect();
                self.error_help(
                    call.span,
                    format!("`{key}` has no argument `{name}`"),
                    if known.is_empty() {
                        "this message takes no arguments".to_string()
                    } else {
                        format!("its arguments: {}", known.join(", "))
                    },
                );
                ok = false;
            }
        }
        for arg in &message.args {
            if !args.iter().any(|(name, _)| *name == arg.name) {
                self.error(call.span, format!("`{key}` needs the argument `{}`", arg.name));
                ok = false;
            }
        }
        if !ok {
            return None;
        }
        Some(Expr::Translate {
            key: Rc::from(key),
            args,
        })
    }

    fn query(
        &mut self,
        call: &oxc_ast::ast::CallExpression,
        path: &str,
        scope: &Scope,
    ) -> Lowered<Expr> {
        let Some(spec) = self.options.queries.iter().find(|spec| spec.path == path) else {
            let known: Vec<&str> = self
                .options
                .queries
                .iter()
                .map(|spec| spec.path.as_str())
                .collect();
            self.error_help(
                call.span,
                format!("`state.{path}` is not a state query"),
                format!("queries: {}", known.join(", ")),
            );
            return None;
        };
        if call.arguments.len() != spec.params.len() {
            let message = format!(
                "`state.{path}` takes {} argument{}, not {}",
                spec.params.len(),
                if spec.params.len() == 1 { "" } else { "s" },
                call.arguments.len()
            );
            let help = format!("signature: {}", spec.signature());
            self.error_help(call.span, message, help);
            return None;
        }
        let args = call
            .arguments
            .iter()
            .map(|argument| {
                let argument = self.argument_expression(argument)?;
                self.expr(argument, scope)
            })
            .collect::<Option<Vec<_>>>()?;
        Some(Expr::Query {
            path: Rc::from(path),
            args,
        })
    }

    fn css_class(&mut self, expression: &Expression) -> Option<Lowered<Expr>> {
        let (object, name, span) = match expression {
            Expression::StaticMemberExpression(member) => {
                (&member.object, member.property.name.to_string(), member.span)
            }
            Expression::ComputedMemberExpression(member) => match member.expression.get_inner_expression() {
                Expression::StringLiteral(text) => (&member.object, text.value.to_string(), member.span),
                _ => return None,
            },
            _ => return None,
        };
        let Expression::Identifier(id) = object.get_inner_expression() else {
            return None;
        };
        let stem = self.css_imports.get(id.name.as_str())?.clone();
        match self.options.css_modules.class(&stem, &name) {
            Some(hashed) => Some(Some(Expr::Lit(Value::str(hashed)))),
            None => {
                self.error_help(
                    span,
                    format!("`{stem}.module.css` has no class `{name}`"),
                    "add the class to the stylesheet, or fix the name",
                );
                Some(None)
            }
        }
    }

    fn expr(&mut self, expression: &Expression, scope: &Scope) -> Lowered<Expr> {
        let expression = expression.get_inner_expression();
        if let Some(result) = self.css_class(expression) {
            return result;
        }
        Some(match expression {
            Expression::BooleanLiteral(b) => Expr::Lit(Value::Bool(b.value)),
            Expression::NullLiteral(_) => Expr::Lit(Value::Null),
            Expression::NumericLiteral(n) => Expr::Lit(Value::Num(n.value)),
            Expression::StringLiteral(s) => Expr::Lit(Value::str(s.value.as_str())),
            Expression::TemplateLiteral(template) => {
                let strings = template
                    .quasis
                    .iter()
                    .map(|quasi| {
                        let text = quasi.value.cooked.as_ref().unwrap_or(&quasi.value.raw);
                        Rc::<str>::from(text.as_str())
                    })
                    .collect();
                let exprs = template
                    .expressions
                    .iter()
                    .map(|inner| self.expr(inner, scope))
                    .collect::<Option<Vec<_>>>()?;
                Expr::Template(strings, exprs)
            }
            Expression::Identifier(id) => {
                if id.name == "undefined" {
                    Expr::Lit(Value::Null)
                } else if let Some(index) = scope.lookup(id.name.as_str()) {
                    Expr::Var(index)
                } else if scope.setters.contains_key(id.name.as_str()) {
                    self.error_help(
                        id.span,
                        format!("`{}` is a state setter and can only be called from an event handler", id.name),
                        "move the call into `onClick={() => ...}`",
                    );
                    return None;
                } else {
                    self.error_help(
                        id.span,
                        format!("`{}` is not defined in this component", id.name),
                        "components see their props, their `useState` values and earlier `const` locals",
                    );
                    return None;
                }
            }
            Expression::StaticMemberExpression(member) => Expr::Member(
                Box::new(self.expr(&member.object, scope)?),
                Rc::from(member.property.name.as_str()),
            ),
            Expression::ComputedMemberExpression(member) => Expr::Index(
                Box::new(self.expr(&member.object, scope)?),
                Box::new(self.expr(&member.expression, scope)?),
            ),
            Expression::ChainExpression(chain) => match &chain.expression {
                ChainElement::StaticMemberExpression(member) => Expr::Member(
                    Box::new(self.expr(&member.object, scope)?),
                    Rc::from(member.property.name.as_str()),
                ),
                ChainElement::ComputedMemberExpression(member) => Expr::Index(
                    Box::new(self.expr(&member.object, scope)?),
                    Box::new(self.expr(&member.expression, scope)?),
                ),
                other => {
                    self.error(other.span(), "only optional member access is supported");
                    return None;
                }
            },
            Expression::UnaryExpression(unary) => {
                let inner = Box::new(self.expr(&unary.argument, scope)?);
                match unary.operator {
                    UnaryOperator::LogicalNot => Expr::Not(inner),
                    UnaryOperator::UnaryNegation => Expr::Neg(inner),
                    _ => {
                        self.error(unary.span, "this unary operator is not supported");
                        return None;
                    }
                }
            }
            Expression::BinaryExpression(binary) => {
                let op = match binary.operator {
                    BinaryOperator::Addition => BinOp::Add,
                    BinaryOperator::Subtraction => BinOp::Sub,
                    BinaryOperator::Multiplication => BinOp::Mul,
                    BinaryOperator::Division => BinOp::Div,
                    BinaryOperator::Remainder => BinOp::Rem,
                    BinaryOperator::StrictEquality => BinOp::Eq,
                    BinaryOperator::StrictInequality => BinOp::Ne,
                    BinaryOperator::LessThan => BinOp::Lt,
                    BinaryOperator::LessEqualThan => BinOp::Le,
                    BinaryOperator::GreaterThan => BinOp::Gt,
                    BinaryOperator::GreaterEqualThan => BinOp::Ge,
                    BinaryOperator::Equality | BinaryOperator::Inequality => {
                        self.error_help(
                            binary.span,
                            "loose equality is not supported",
                            "use `===` or `!==`",
                        );
                        return None;
                    }
                    _ => {
                        self.error(binary.span, "this operator is not supported");
                        return None;
                    }
                };
                Expr::Binary(
                    op,
                    Box::new(self.expr(&binary.left, scope)?),
                    Box::new(self.expr(&binary.right, scope)?),
                )
            }
            Expression::LogicalExpression(logical) => {
                let left = Box::new(self.expr(&logical.left, scope)?);
                let right = Box::new(self.expr(&logical.right, scope)?);
                match logical.operator {
                    LogicalOperator::And => Expr::And(left, right),
                    LogicalOperator::Or => Expr::Or(left, right),
                    LogicalOperator::Coalesce => Expr::Nullish(left, right),
                }
            }
            Expression::ConditionalExpression(cond) => Expr::Cond(
                Box::new(self.expr(&cond.test, scope)?),
                Box::new(self.expr(&cond.consequent, scope)?),
                Box::new(self.expr(&cond.alternate, scope)?),
            ),
            Expression::ArrayExpression(array) => {
                let mut items = Vec::new();
                for element in &array.elements {
                    match element {
                        ArrayExpressionElement::SpreadElement(spread) => {
                            self.error(spread.span, "spread is not supported");
                            return None;
                        }
                        ArrayExpressionElement::Elision(elision) => {
                            self.error(elision.span, "array holes are not supported");
                            return None;
                        }
                        other => items.push(self.expr(other.as_expression()?, scope)?),
                    }
                }
                Expr::List(items)
            }
            Expression::ObjectExpression(object) => {
                let mut fields = Vec::new();
                for property in &object.properties {
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        self.error(property.span(), "object spread is not supported");
                        return None;
                    };
                    let name = match (&property.key, property.computed) {
                        (PropertyKey::StaticIdentifier(id), false) => id.name.to_string(),
                        (PropertyKey::StringLiteral(s), false) => s.value.to_string(),
                        _ => {
                            self.error(property.span, "object keys must be plain names");
                            return None;
                        }
                    };
                    if property.method {
                        self.error(property.span, "methods are not supported");
                        return None;
                    }
                    fields.push((Rc::from(name), self.expr(&property.value, scope)?));
                }
                Expr::Obj(fields)
            }
            Expression::CallExpression(call) => {
                if matches!(call.callee.get_inner_expression(),
                    Expression::Identifier(id) if id.name == "t" && scope.lookup("t").is_none())
                {
                    return self.translate(call, scope);
                }
                if let Some(segments) = state_path(call.callee.get_inner_expression()) {
                    return self.query(call, &segments.join("."), scope);
                }
                if let Expression::StaticMemberExpression(member) = call.callee.get_inner_expression() {
                    let method = match member.property.name.as_str() {
                        "toUpperCase" => Some(Method::ToUpperCase),
                        "toLowerCase" => Some(Method::ToLowerCase),
                        "trim" => Some(Method::Trim),
                        "includes" => Some(Method::Includes),
                        "join" => Some(Method::Join),
                        _ => None,
                    };
                    if let Some(method) = method {
                        let args = call
                            .arguments
                            .iter()
                            .map(|argument| {
                                let argument = self.argument_expression(argument)?;
                                self.expr(argument, scope)
                            })
                            .collect::<Option<Vec<_>>>()?;
                        return Some(Expr::Method(
                            Box::new(self.expr(&member.object, scope)?),
                            method,
                            args,
                        ));
                    }
                }
                self.error_help(
                    call.span,
                    "this call is not supported",
                    "expressions may call .toUpperCase(), .toLowerCase(), .trim(), .includes() and .join(); mutations go through actions in event handlers",
                );
                return None;
            }
            Expression::AwaitExpression(await_expression) => {
                self.error_help(
                    await_expression.span,
                    "`await` is not supported",
                    "do the work in a Rust action and read the result from `state`",
                );
                return None;
            }
            Expression::AssignmentExpression(assignment) => {
                self.error_help(
                    assignment.span,
                    "assignment is not supported",
                    "call the state setter, or an action for anything in `state`",
                );
                return None;
            }
            Expression::UpdateExpression(update) => {
                self.error_help(
                    update.span,
                    "`++` and `--` are not supported",
                    "call the state setter: `setCount(count + 1)`",
                );
                return None;
            }
            Expression::NewExpression(new) => {
                self.error(new.span, "`new` is not supported");
                return None;
            }
            Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => {
                self.error_help(
                    expression.span(),
                    "functions are only allowed as `onClick` handlers and `.map` callbacks",
                    "inline the logic or move it into a Rust action",
                );
                return None;
            }
            Expression::JSXElement(_) | Expression::JSXFragment(_) => {
                self.error_help(
                    expression.span(),
                    "JSX is only allowed as the returned element, as a child, or as a `&&` / `?:` branch",
                    "return the element directly",
                );
                return None;
            }
            other => {
                self.error(other.span(), "this expression is not supported");
                return None;
            }
        })
    }

    fn element(
        &mut self,
        element: &JSXElement,
        scope: &mut Scope,
        keyed: bool,
    ) -> Lowered<(Element, Option<Expr>)> {
        let opening = &element.opening_element;
        let kind = match &opening.name {
            JSXElementName::Identifier(id) if id.name == "div" => Kind::Div,
            JSXElementName::Identifier(id) => {
                self.error_help(
                    id.span,
                    format!("<{}> is not supported", id.name),
                    "the only plain element is <div>; text is <Text> and buttons are <Button>",
                );
                return None;
            }
            JSXElementName::IdentifierReference(id) => {
                let name = id.name.as_str();
                if self.signatures.contains_key(name) {
                    Kind::Use
                } else if self.app_imports.contains(name) {
                    Kind::Host
                } else if !self.ui_imports.contains(name) {
                    self.error_help(
                        id.span,
                        format!("<{name}> is not an engine component"),
                        format!("import it first: import {{ Text, Button }} from \"{UI_MODULE}\""),
                    );
                    return None;
                } else {
                match name {
                    "Text" => Kind::Text,
                    "Button" => Kind::Button,
                    "ScrollView" => Kind::Scroll,
                    "Chevron" => Kind::Chevron,
                    "Stack" => Kind::Stack,
                    other if icon_named(other).is_some() => {
                        Kind::Icon(icon_named(other).unwrap_or(IconHandle::Named("")))
                    }
                    other => {
                        self.error_help(
                            id.span,
                            format!("<{other}> is not available yet"),
                            format!("available components: {}, and the <...Icon> components", UI_COMPONENTS.join(", ")),
                        );
                        return None;
                    }
                }
                }
            }
            other => {
                self.error(other.span(), "this element name is not supported");
                return None;
            }
        };

        let mut ok = true;
        let mut key = None;
        let mut raws = Vec::new();
        for item in &opening.attributes {
            let attribute = match item {
                JSXAttributeItem::Attribute(attribute) => attribute,
                JSXAttributeItem::SpreadAttribute(spread) => {
                    self.error(spread.span, "spread props are not supported");
                    ok = false;
                    continue;
                }
            };
            let JSXAttributeName::Identifier(name) = &attribute.name else {
                self.error(attribute.span, "namespaced attributes are not supported");
                ok = false;
                continue;
            };
            let raw = Raw {
                name: name.name.as_str(),
                name_span: name.span,
                span: attribute.span,
                value: attribute.value.as_ref(),
            };
            if raw.name == "key" {
                if !keyed {
                    self.error_help(
                        raw.name_span,
                        "`key` only belongs on the element a `.map` returns",
                        "remove it",
                    );
                    ok = false;
                    continue;
                }
                match self.attribute_expr(raw.value, raw.span, scope) {
                    Some(Attr::Dyn(expr)) => key = Some(expr),
                    Some(Attr::Static(text)) => key = Some(Expr::Lit(Value::str(&*text))),
                    _ => ok = false,
                }
                continue;
            }
            raws.push(raw);
        }

        let lowered = match kind {
            Kind::Div => self.div(element, &raws, scope),
            Kind::Text => self.text(element, &raws, scope),
            Kind::Button => self.button(element, &raws, scope),
            Kind::Icon(icon) => self.icon(icon, element, &raws, scope),
            Kind::Scroll => self.scroll(element, &raws, scope),
            Kind::Chevron => self.chevron(element, &raws, scope),
            Kind::Stack => self.stack(element, &raws, scope),
            Kind::Use => self.use_component(element, &raws, scope),
            Kind::Host => self.host_component(element, &raws, scope),
        };
        let lowered = lowered?;
        ok.then_some((lowered, key))
    }

    fn unsupported_attribute(&mut self, raw: &Raw, supported: &str) {
        self.error_help(
            raw.name_span,
            format!("attribute `{}` is not supported here", raw.name),
            format!("supported attributes: {supported}"),
        );
    }

    fn div(&mut self, element: &JSXElement, raws: &[Raw], scope: &mut Scope) -> Lowered<Element> {
        let (mut class, mut style, mut on_click) = (Attr::None, Attr::None, None);
        let mut on_context_menu = None;
        let mut ok = true;
        for raw in raws {
            match raw.name {
                "className" => match self.attribute_expr(raw.value, raw.span, scope) {
                    Some(attr) => class = attr,
                    None => ok = false,
                },
                "class" => {
                    self.error_help(raw.name_span, "`class` is not an attribute", "write `className`");
                    ok = false;
                }
                "style" => {
                    if let Some(JSXAttributeValue::ExpressionContainer(container)) = raw.value {
                        if matches!(
                            container.expression.as_expression().map(Expression::get_inner_expression),
                            Some(Expression::ObjectExpression(_))
                        ) {
                            self.error_help(
                                container.span,
                                "style objects are not supported yet",
                                "write the style as a CSS string: style=\"width: 10px\"",
                            );
                            ok = false;
                            continue;
                        }
                    }
                    match self.attribute_expr(raw.value, raw.span, scope) {
                        Some(attr) => style = attr,
                        None => ok = false,
                    }
                }
                "onClick" => match self.handler(raw.value, raw.span, scope, false) {
                    Some((body, _)) => on_click = Some(Rc::new(body)),
                    None => ok = false,
                },
                "onContextMenu" => match self.handler(raw.value, raw.span, scope, true) {
                    Some((body, event)) => {
                        on_context_menu = Some(Handler {
                            body: Rc::new(body),
                            event,
                        })
                    }
                    None => ok = false,
                },
                _ => {
                    self.unsupported_attribute(raw, "className, style, onClick, onContextMenu");
                    ok = false;
                }
            }
        }
        let mut children = Vec::new();
        self.children(&element.children, scope, &mut children);
        ok.then_some(Element::Div(Div {
            class,
            style,
            on_click,
            on_context_menu,
            children,
        }))
    }

    fn text(&mut self, element: &JSXElement, raws: &[Raw], scope: &mut Scope) -> Lowered<Element> {
        let mut ok = true;
        let mut variant = None;
        let mut color = text_color("text-default");
        let (mut line_clamp, mut center, mut class) = (None, false, None);
        for raw in raws {
            match raw.name {
                "variant" => {
                    let name = self.static_string(raw, scope)?;
                    match TextVariant::from_name(&name) {
                        Some(found) => variant = Some(found),
                        None => {
                            self.error_help(
                                raw.span,
                                format!("`{name}` is not a text variant"),
                                "variants are <text|heading>-<size>/<weight>, e.g. text-sm/semibold or heading-md/bold; display-sm/md/lg, eyebrow and code stand alone",
                            );
                            ok = false;
                        }
                    }
                }
                "color" => {
                    let name = self.static_string(raw, scope)?;
                    match text_color(&name) {
                        Some(found) => color = Some(found),
                        None => {
                            let known: Vec<&str> = TEXT_COLORS.iter().map(|(n, _)| *n).collect();
                            self.error_help(
                                raw.span,
                                format!("`{name}` is not a text color"),
                                format!("colors: {}", known.join(", ")),
                            );
                            ok = false;
                        }
                    }
                }
                "lineClamp" => match self.static_number(raw) {
                    Some(n) if n >= 1.0 && n.fract() == 0.0 => line_clamp = Some(n as u32),
                    _ => {
                        self.error(raw.span, "lineClamp must be a whole number of at least 1");
                        ok = false;
                    }
                },
                "center" => center = self.static_bool(raw)?,
                "className" => class = Some(Rc::from(self.static_string(raw, scope)?)),
                _ => {
                    self.unsupported_attribute(raw, "variant, color, lineClamp, center, className");
                    ok = false;
                }
            }
        }
        if variant.is_none() && ok {
            self.error_help(
                element.opening_element.span,
                "<Text> needs a `variant`",
                "e.g. variant=\"text-md/normal\"",
            );
            ok = false;
        }
        let parts = self.text_parts(&element.children, scope)?;
        ok.then_some(Element::Text(TextEl {
            variant: variant?,
            color: color?,
            line_clamp,
            center,
            class,
            parts,
        }))
    }

    fn button(&mut self, element: &JSXElement, raws: &[Raw], scope: &mut Scope) -> Lowered<Element> {
        let mut ok = true;
        let (mut variant, mut size) = (ButtonVariant::Primary, ButtonSize::Medium);
        let (mut class, mut on_click, mut on_click_event) = (None, None, false);
        let mut on_context_menu = None;
        let (mut disabled, mut full_width) = (false, false);
        for raw in raws {
            match raw.name {
                "variant" => {
                    let name = self.static_string(raw, scope)?;
                    match ButtonVariant::parse(&name) {
                        Some(found) => variant = found,
                        None => {
                            self.error_help(
                                raw.span,
                                format!("`{name}` is not a button variant"),
                                format!("variants: {}", BUTTON_VARIANTS.join(", ")),
                            );
                            ok = false;
                        }
                    }
                }
                "size" => match self.static_string(raw, scope)?.as_str() {
                    "small" => size = ButtonSize::Small,
                    "medium" => size = ButtonSize::Medium,
                    other => {
                        self.error_help(
                            raw.span,
                            format!("`{other}` is not a button size"),
                            format!("sizes: {}", BUTTON_SIZES.join(", ")),
                        );
                        ok = false;
                    }
                },
                "className" => class = Some(Rc::from(self.static_string(raw, scope)?)),
                "disabled" => disabled = self.static_bool(raw)?,
                "fullWidth" => full_width = self.static_bool(raw)?,
                "onClick" => match self.handler(raw.value, raw.span, scope, true) {
                    Some((body, event)) => {
                        on_click = Some(Rc::new(body));
                        on_click_event = event;
                    }
                    None => ok = false,
                },
                "onContextMenu" => match self.handler(raw.value, raw.span, scope, true) {
                    Some((body, event)) => {
                        on_context_menu = Some(Handler {
                            body: Rc::new(body),
                            event,
                        })
                    }
                    None => ok = false,
                },
                _ => {
                    self.unsupported_attribute(
                        raw,
                        "variant, size, className, disabled, fullWidth, onClick, onContextMenu",
                    );
                    ok = false;
                }
            }
        }
        let (mut label, mut children) = (Vec::new(), Vec::new());
        for child in &element.children {
            match child {
                JSXChild::Text(text) => {
                    let text = jsx_text(text.value.as_str());
                    if !text.is_empty() {
                        label.push(Part::Lit(Rc::from(text)));
                    }
                }
                JSXChild::ExpressionContainer(container) => {
                    let Some(expression) = container.expression.as_expression() else {
                        continue;
                    };
                    if renders_element(expression.get_inner_expression()) {
                        self.child_expression(expression, scope, &mut children);
                    } else {
                        match self.expr(expression, scope) {
                            Some(expr) => label.push(Part::Dyn(expr)),
                            None => ok = false,
                        }
                    }
                }
                other => self.children(slice::from_ref(other), scope, &mut children),
            }
        }
        if !label.is_empty() && !children.is_empty() {
            self.error_help(
                element.span,
                "a <Button> holds text or elements, not both",
                "put the text in its own <Text> beside the other elements",
            );
            ok = false;
        }
        ok.then_some(Element::Button(ButtonEl {
            variant,
            size,
            label,
            class,
            disabled,
            full_width,
            on_click,
            on_click_event,
            on_context_menu,
            children,
        }))
    }

    fn chevron(&mut self, element: &JSXElement, raws: &[Raw], scope: &mut Scope) -> Lowered<Element> {
        let mut ok = true;
        let (mut open, mut color, mut class) = (None, text_color("text-subtle"), None);
        let (mut width, mut height) = (None, None);
        for raw in raws {
            match raw.name {
                "open" => {
                    open = match raw.value {
                        None => Some(Expr::Lit(Value::Bool(true))),
                        Some(_) => match self.attribute_expr(raw.value, raw.span, scope) {
                            Some(Attr::Dyn(expr)) => Some(expr),
                            Some(Attr::Static(text)) => Some(Expr::Lit(Value::str(&*text))),
                            _ => {
                                ok = false;
                                None
                            }
                        },
                    };
                }
                "color" => {
                    let name = self.static_string(raw, scope)?;
                    match text_color(&name) {
                        Some(found) => color = Some(found),
                        None => {
                            self.error(raw.span, format!("`{name}` is not a color"));
                            ok = false;
                        }
                    }
                }
                "width" | "height" => match self.static_number(raw) {
                    Some(px) if px > 0.0 => {
                        if raw.name == "width" {
                            width = Some(px as f32);
                        } else {
                            height = Some(px as f32);
                        }
                    }
                    _ => {
                        self.error(raw.span, format!("`{}` must be a positive number of pixels", raw.name));
                        ok = false;
                    }
                },
                "className" => class = Some(Rc::from(self.static_string(raw, scope)?)),
                _ => {
                    self.unsupported_attribute(raw, "open, color, width, height, className");
                    ok = false;
                }
            }
        }
        if open.is_none() && ok {
            self.error_help(
                element.opening_element.span,
                "<Chevron> needs `open`",
                "e.g. open={expanded}",
            );
            ok = false;
        }
        if !element.children.is_empty() {
            self.error(element.span, "a chevron takes no children");
            ok = false;
        }
        ok.then_some(Element::Chevron(ChevronEl {
            open: open?,
            color: color?,
            width,
            height,
            class,
        }))
    }

    fn icon(
        &mut self,
        icon: IconHandle,
        element: &JSXElement,
        raws: &[Raw],
        scope: &mut Scope,
    ) -> Lowered<Element> {
        let mut ok = true;
        let (mut color, mut class) = (text_color("text-default"), None);
        let (mut width, mut height) = (None, None);
        for raw in raws {
            match raw.name {
                "color" => {
                    let name = self.static_string(raw, scope)?;
                    match text_color(&name) {
                        Some(found) => color = Some(found),
                        None => {
                            self.error(raw.span, format!("`{name}` is not a color"));
                            ok = false;
                        }
                    }
                }
                "width" | "height" => match self.static_number(raw) {
                    Some(px) if px > 0.0 => {
                        if raw.name == "width" {
                            width = Some(px as f32);
                        } else {
                            height = Some(px as f32);
                        }
                    }
                    _ => {
                        self.error(raw.span, format!("`{}` must be a positive number of pixels", raw.name));
                        ok = false;
                    }
                },
                "className" => class = Some(Rc::from(self.static_string(raw, scope)?)),
                _ => {
                    self.unsupported_attribute(raw, "color, width, height, className");
                    ok = false;
                }
            }
        }
        if !element.children.is_empty() {
            self.error(element.span, "an icon takes no children");
            ok = false;
        }
        ok.then_some(Element::Icon(IconEl {
            icon,
            color: color?,
            width,
            height,
            class,
        }))
    }

    fn stack(&mut self, element: &JSXElement, raws: &[Raw], scope: &mut Scope) -> Lowered<Element> {
        let mut ok = true;
        let (mut direction, mut align, mut justify) =
            (Direction::Vertical, Align::Stretch, Justify::Start);
        let (mut wrap, mut full_width, mut gap, mut padding, mut class) = (false, true, 8.0, 0.0, None);
        for raw in raws {
            match raw.name {
                "direction" => {
                    let name = self.static_string(raw, scope)?;
                    match named(STACK_DIRECTIONS, &name) {
                        Some(found) => direction = found,
                        None => {
                            self.error_help(
                                raw.span,
                                format!("`{name}` is not a stack direction"),
                                format!("directions: {}", names(STACK_DIRECTIONS).join(", ")),
                            );
                            ok = false;
                        }
                    }
                }
                "align" => {
                    let name = self.static_string(raw, scope)?;
                    match named(STACK_ALIGNS, &name) {
                        Some(found) => align = found,
                        None => {
                            self.error_help(
                                raw.span,
                                format!("`{name}` is not a stack alignment"),
                                format!("alignments: {}", names(STACK_ALIGNS).join(", ")),
                            );
                            ok = false;
                        }
                    }
                }
                "justify" => {
                    let name = self.static_string(raw, scope)?;
                    match named(STACK_JUSTIFIES, &name) {
                        Some(found) => justify = found,
                        None => {
                            self.error_help(
                                raw.span,
                                format!("`{name}` is not a stack justification"),
                                format!("justifications: {}", names(STACK_JUSTIFIES).join(", ")),
                            );
                            ok = false;
                        }
                    }
                }
                "wrap" => wrap = self.static_bool(raw)?,
                "fullWidth" => full_width = self.static_bool(raw)?,
                "gap" | "padding" => match self.static_number(raw) {
                    Some(px) if px >= 0.0 => {
                        if raw.name == "gap" {
                            gap = px as f32;
                        } else {
                            padding = px as f32;
                        }
                    }
                    _ => {
                        self.error(raw.span, format!("`{}` must be a number of pixels", raw.name));
                        ok = false;
                    }
                },
                "className" => class = Some(Rc::from(self.static_string(raw, scope)?)),
                _ => {
                    self.unsupported_attribute(
                        raw,
                        "direction, align, justify, wrap, gap, padding, fullWidth, className",
                    );
                    ok = false;
                }
            }
        }
        let mut children = Vec::new();
        self.children(&element.children, scope, &mut children);
        ok.then_some(Element::Stack(StackEl {
            direction,
            align,
            justify,
            wrap,
            gap,
            padding,
            full_width,
            class,
            children,
        }))
    }

    fn scroll(&mut self, element: &JSXElement, raws: &[Raw], scope: &mut Scope) -> Lowered<Element> {
        let mut ok = true;
        let mut class = None;
        let mut height = None;
        let mut gap = 0.0_f32;
        let mut initial_offset = None;
        let mut on_scroll = None;
        let mut on_near_end = None;
        let mut position = None;
        let mut pin_threshold = 16.0_f32;
        let mut on_position = None;
        let mut on_measure = None;
        for raw in raws {
            match raw.name {
                "className" => class = Some(Rc::from(self.static_string(raw, scope)?)),
                "position" => match self.prop_expr(raw, scope) {
                    Some(expr) => position = Some(expr),
                    None => ok = false,
                },
                "pinThreshold" => match self.static_number(raw) {
                    Some(px) if px >= 0.0 => pin_threshold = px as f32,
                    _ => {
                        self.error(raw.span, "`pinThreshold` must be a number of pixels");
                        ok = false;
                    }
                },
                "onPosition" => match self.handler(raw.value, raw.span, scope, true) {
                    Some((body, event)) => {
                        on_position = Some(Handler {
                            body: Rc::new(body),
                            event,
                        })
                    }
                    None => ok = false,
                },
                "onMeasure" => match self.handler(raw.value, raw.span, scope, true) {
                    Some((body, event)) => {
                        on_measure = Some(Handler {
                            body: Rc::new(body),
                            event,
                        })
                    }
                    None => ok = false,
                },
                "itemHeight" => match self.static_number(raw) {
                    Some(px) if px > 0.0 => height = Some(ItemHeight::Fixed(px as f32)),
                    Some(_) => {
                        self.error(raw.span, "`itemHeight` must be a positive number of pixels");
                        ok = false;
                    }
                    None => height = Some(ItemHeight::Field(Rc::from(self.static_string(raw, scope)?))),
                },
                "itemGap" => match self.static_number(raw) {
                    Some(px) if px >= 0.0 => gap = px as f32,
                    _ => {
                        self.error(raw.span, "`itemGap` must be a number of pixels");
                        ok = false;
                    }
                },
                "initialOffset" => match self.prop_expr(raw, scope) {
                    Some(expr) => initial_offset = Some(expr),
                    None => ok = false,
                },
                "onScroll" => match self.handler(raw.value, raw.span, scope, true) {
                    Some((body, event)) => {
                        on_scroll = Some(Handler {
                            body: Rc::new(body),
                            event,
                        })
                    }
                    None => ok = false,
                },
                "onNearEnd" => match self.handler(raw.value, raw.span, scope, false) {
                    Some((body, _)) => on_near_end = Some(Rc::new(body)),
                    None => ok = false,
                },
                _ => {
                    self.unsupported_attribute(
                        raw,
                        "className, itemHeight, itemGap, initialOffset, onScroll, onNearEnd, position, pinThreshold, onPosition, onMeasure",
                    );
                    ok = false;
                }
            }
        }
        let mut children = Vec::new();
        self.children(&element.children, scope, &mut children);
        let window = match height {
            Some(height) => {
                if !matches!(children.as_slice(), [Child::Each { .. }]) {
                    self.error_help(
                        element.span,
                        "a ScrollView with `itemHeight` takes exactly one `.map`",
                        "put the rows in one list: {items.map((item) => <Row key={item.id} />)}",
                    );
                    ok = false;
                }
                Some(Window { height, gap })
            }
            None => {
                if gap > 0.0 {
                    self.error(element.span, "`itemGap` needs `itemHeight`");
                    ok = false;
                }
                None
            }
        };
        let chat = match position {
            Some(position) => {
                if window.as_ref().is_none_or(|window| window.gap > 0.0) {
                    self.error_help(
                        element.span,
                        "`position` needs `itemHeight` and no `itemGap`",
                        "a chat list is windowed, and its rows carry their own spacing in `height`",
                    );
                    ok = false;
                }
                Some(ChatSpec {
                    position,
                    pin_threshold,
                    on_position,
                    on_measure,
                })
            }
            None => {
                if on_position.is_some() || on_measure.is_some() {
                    self.error(element.span, "`onPosition` and `onMeasure` need `position`");
                    ok = false;
                }
                None
            }
        };
        ok.then_some(Element::Scroll(ScrollEl {
            class,
            window,
            initial_offset,
            on_scroll,
            on_near_end,
            chat,
            children,
        }))
    }

    fn prop_expr(&mut self, raw: &Raw, scope: &mut Scope) -> Lowered<Expr> {
        let arrow = match raw.value {
            Some(JSXAttributeValue::ExpressionContainer(container)) => container
                .expression
                .as_expression()
                .map(Expression::get_inner_expression)
                .and_then(|expression| match expression {
                    Expression::ArrowFunctionExpression(arrow) => Some(arrow),
                    _ => None,
                }),
            _ => None,
        };
        if let Some(arrow) = arrow {
            return self.closure(arrow, scope);
        }
        match self.attribute_expr(raw.value, raw.span, scope)? {
            Attr::Static(text) => Some(Expr::Lit(Value::str(&*text))),
            Attr::Dyn(expr) => Some(expr),
            Attr::None => None,
        }
    }

    fn host_component(&mut self, element: &JSXElement, raws: &[Raw], scope: &mut Scope) -> Lowered<Element> {
        let JSXElementName::IdentifierReference(id) = &element.opening_element.name else {
            return None;
        };
        let name = id.name.to_string();
        let Some(spec) = self.options.host.iter().find(|spec| spec.name == name).cloned() else {
            self.error(id.span, format!("<{name}> is not registered by the app"));
            return None;
        };
        let mut ok = true;
        let mut props = Vec::new();
        for raw in raws {
            if !spec.props.iter().any(|prop| prop.name == raw.name) {
                let known: Vec<&str> = spec.props.iter().map(|prop| prop.name.as_str()).collect();
                self.error_help(
                    raw.name_span,
                    format!("<{name}> has no prop `{}`", raw.name),
                    format!("its props: {}", known.join(", ")),
                );
                ok = false;
                continue;
            }
            match self.prop_expr(raw, scope) {
                Some(expr) => props.push((Rc::<str>::from(raw.name), expr)),
                None => ok = false,
            }
        }
        for prop in spec.props.iter().filter(|prop| prop.required) {
            if !raws.iter().any(|raw| raw.name == prop.name) {
                self.error(
                    element.opening_element.span,
                    format!("<{name}> needs the prop `{}`", prop.name),
                );
                ok = false;
            }
        }
        if !element.children.is_empty() {
            self.error(element.span, format!("<{name}> takes no children"));
            ok = false;
        }
        ok.then_some(Element::Host(HostEl {
            name: Rc::from(name),
            props,
        }))
    }

    fn use_component(&mut self, element: &JSXElement, raws: &[Raw], scope: &mut Scope) -> Lowered<Element> {
        let JSXElementName::IdentifierReference(id) = &element.opening_element.name else {
            return None;
        };
        let name = id.name.to_string();
        let (key, declared) = match self.signatures.get(&name) {
            Some(found) => (found.key.clone(), found.props.clone()),
            None => (Rc::from(name.as_str()), Vec::new()),
        };
        let mut ok = true;
        let mut props = Vec::new();
        for raw in raws {
            if !declared.iter().any(|prop| prop == raw.name) {
                self.error_help(
                    raw.name_span,
                    format!("<{name}> has no prop `{}`", raw.name),
                    if declared.is_empty() {
                        format!("<{name}> takes no props")
                    } else {
                        format!("its props: {}", declared.join(", "))
                    },
                );
                ok = false;
                continue;
            }
            let Some(expr) = self.prop_expr(raw, scope) else {
                ok = false;
                continue;
            };
            props.push((Rc::<str>::from(raw.name), expr));
        }
        if !element.children.is_empty() {
            self.error_help(
                element.span,
                format!("<{name}> cannot take children yet"),
                "pass what it needs as props",
            );
            ok = false;
        }
        ok.then_some(Element::Use(UseEl { name: key, props }))
    }

    fn text_parts(&mut self, children: &[JSXChild], scope: &Scope) -> Lowered<Vec<Part>> {
        let mut parts = Vec::new();
        let mut ok = true;
        for child in children {
            match child {
                JSXChild::Text(text) => {
                    let text = jsx_text(text.value.as_str());
                    if !text.is_empty() {
                        parts.push(Part::Lit(Rc::from(text)));
                    }
                }
                JSXChild::ExpressionContainer(container) => {
                    if let Some(expression) = container.expression.as_expression() {
                        match self.expr(expression, scope) {
                            Some(expr) => parts.push(Part::Dyn(expr)),
                            None => ok = false,
                        }
                    }
                }
                other => {
                    self.error_help(
                        other.span(),
                        "<Text> holds text and {expressions} only",
                        "put elements beside the <Text>, not inside it",
                    );
                    ok = false;
                }
            }
        }
        ok.then_some(parts)
    }

    fn static_string(&mut self, raw: &Raw, scope: &Scope) -> Lowered<String> {
        match self.attribute_expr(raw.value, raw.span, scope)? {
            Attr::Static(text) => Some(text.to_string()),
            _ => {
                self.error_help(
                    raw.span,
                    format!("`{}` must be known at compile time", raw.name),
                    "use a string literal (or a CSS module class)",
                );
                None
            }
        }
    }

    fn static_bool(&mut self, raw: &Raw) -> Lowered<bool> {
        match raw.value {
            None => Some(true),
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                match container.expression.as_expression().map(Expression::get_inner_expression) {
                    Some(Expression::BooleanLiteral(b)) => Some(b.value),
                    _ => {
                        self.error(raw.span, format!("`{}` must be true or false", raw.name));
                        None
                    }
                }
            }
            _ => {
                self.error(raw.span, format!("`{}` must be true or false", raw.name));
                None
            }
        }
    }

    fn static_number(&mut self, raw: &Raw) -> Option<f64> {
        match raw.value {
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                match container.expression.as_expression().map(Expression::get_inner_expression) {
                    Some(Expression::NumericLiteral(n)) => Some(n.value),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn attribute_expr(
        &mut self,
        value: Option<&JSXAttributeValue>,
        span: Span,
        scope: &Scope,
    ) -> Lowered<Attr> {
        match value {
            Some(JSXAttributeValue::StringLiteral(text)) => {
                Some(Attr::Static(Rc::from(text.value.as_str())))
            }
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                let Some(expression) = container.expression.as_expression() else {
                    self.error(container.span, "an attribute needs a value");
                    return None;
                };
                match self.expr(expression, scope)? {
                    Expr::Lit(Value::Str(text)) => Some(Attr::Static(text)),
                    other => Some(Attr::Dyn(other)),
                }
            }
            _ => {
                self.error(span, "this attribute needs a string or an {expression}");
                None
            }
        }
    }

    fn handler(
        &mut self,
        value: Option<&JSXAttributeValue>,
        span: Span,
        scope: &mut Scope,
        takes_event: bool,
    ) -> Lowered<(Vec<Stmt>, bool)> {
        let Some(JSXAttributeValue::ExpressionContainer(container)) = value else {
            self.error(span, "`onClick` needs an arrow function: onClick={() => ...}");
            return None;
        };
        let expression = container.expression.as_expression()?.get_inner_expression();
        if let Expression::Identifier(id) = expression {
            let Some(slot) = scope.lookup(id.name.as_str()) else {
                self.error_help(
                    id.span,
                    format!("`{}` is not a function prop", id.name),
                    "pass a function prop down, or write an arrow function",
                );
                return None;
            };
            let call = Stmt::Call {
                callee: Expr::Var(slot),
                args: Vec::new(),
            };
            return Some((vec![call], false));
        }
        let Expression::ArrowFunctionExpression(arrow) = expression else {
            self.error_help(
                expression.span(),
                "`onClick` must be an arrow function or a function prop",
                "write onClick={() => setOpen(!open)}",
            );
            return None;
        };
        if arrow.r#async {
            self.error_help(arrow.span, "handlers cannot be async", "call an action; Rust does the async work");
            return None;
        }
        match arrow.params.items.as_slice() {
            [] => self.arrow_body(arrow, scope).map(|body| (body, false)),
            [param] if takes_event => {
                let BindingPattern::BindingIdentifier(name) = &param.pattern else {
                    self.error(param.span, "the event parameter must be a plain name");
                    return None;
                };
                scope.vars.push(name.name.to_string());
                let body = self.arrow_body(arrow, scope);
                scope.vars.pop();
                body.map(|body| (body, true))
            }
            [param, ..] => {
                self.error_help(
                    param.span,
                    "this handler cannot take an event",
                    "only a <Button> onClick receives one, with `shiftKey`",
                );
                None
            }
        }
    }

    fn closure(&mut self, arrow: &ArrowFunctionExpression, scope: &mut Scope) -> Lowered<Expr> {
        if arrow.r#async {
            self.error_help(arrow.span, "functions cannot be async", "call an action; Rust does the async work");
            return None;
        }
        let depth = scope.vars.len();
        for param in &arrow.params.items {
            let BindingPattern::BindingIdentifier(name) = &param.pattern else {
                scope.vars.truncate(depth);
                self.error(param.span, "parameters must be plain names");
                return None;
            };
            scope.vars.push(name.name.to_string());
        }
        let params = arrow.params.items.len();
        let body = self.arrow_body(arrow, scope);
        scope.vars.truncate(depth);
        Some(Expr::Closure {
            params,
            body: Rc::new(body?),
        })
    }

    fn arrow_body(&mut self, arrow: &ArrowFunctionExpression, scope: &mut Scope) -> Lowered<Vec<Stmt>> {
        let mut out = Vec::new();
        match &arrow.body {
            ArrowFunctionBody::FunctionBody(body) => {
                let mut ok = true;
                for statement in &body.statements {
                    ok &= self.stmt(statement, scope, &mut out);
                }
                ok.then_some(out)
            }
            expression => {
                out.push(self.call_stmt(expression.as_expression()?.get_inner_expression(), scope)?);
                Some(out)
            }
        }
    }

    fn stmt(&mut self, statement: &Statement, scope: &mut Scope, out: &mut Vec<Stmt>) -> bool {
        match statement {
            Statement::ExpressionStatement(expression) => {
                match self.call_stmt(expression.expression.get_inner_expression(), scope) {
                    Some(stmt) => {
                        out.push(stmt);
                        true
                    }
                    None => false,
                }
            }
            Statement::BlockStatement(block) => {
                let mut ok = true;
                for inner in &block.body {
                    ok &= self.stmt(inner, scope, out);
                }
                ok
            }
            Statement::IfStatement(branch) => {
                let Some(cond) = self.expr(&branch.test, scope) else {
                    return false;
                };
                let (mut then, mut otherwise) = (Vec::new(), Vec::new());
                let mut ok = self.stmt(&branch.consequent, scope, &mut then);
                if let Some(alternate) = &branch.alternate {
                    ok &= self.stmt(alternate, scope, &mut otherwise);
                }
                out.push(Stmt::If {
                    cond,
                    then,
                    otherwise,
                });
                ok
            }
            Statement::ForStatement(_)
            | Statement::ForInStatement(_)
            | Statement::ForOfStatement(_)
            | Statement::WhileStatement(_)
            | Statement::DoWhileStatement(_) => {
                self.error_help(
                    statement.span(),
                    "loops are not supported",
                    "render lists with `.map` in JSX; do bulk work in a Rust action",
                );
                false
            }
            Statement::TryStatement(_) | Statement::ThrowStatement(_) => {
                self.error_help(
                    statement.span(),
                    "`try`/`catch`/`throw` are not supported",
                    "errors are handled in Rust actions",
                );
                false
            }
            other => {
                self.error(other.span(), "this statement is not supported in a handler");
                false
            }
        }
    }

    fn call_stmt(&mut self, expression: &Expression, scope: &mut Scope) -> Lowered<Stmt> {
        let Expression::CallExpression(call) = expression else {
            self.error_help(
                expression.span(),
                "handlers may only call state setters and actions",
                "write `setX(value)` or `actions.name(args)`",
            );
            return None;
        };
        match call.callee.get_inner_expression() {
            Expression::Identifier(callee) => {
                let Some(&slot) = scope.setters.get(callee.name.as_str()) else {
                    let Some(variable) = scope.lookup(callee.name.as_str()) else {
                        self.error_help(
                            callee.span,
                            format!("`{}` is not a state setter, function prop or action", callee.name),
                            "handlers may call setters from `useState`, function props and `actions.*`",
                        );
                        return None;
                    };
                    let args = call
                        .arguments
                        .iter()
                        .map(|argument| {
                            let argument = self.argument_expression(argument)?;
                            self.expr(argument, scope)
                        })
                        .collect::<Option<Vec<_>>>()?;
                    return Some(Stmt::Call {
                        callee: Expr::Var(variable),
                        args,
                    });
                };
                let [argument] = call.arguments.as_slice() else {
                    self.error(call.span, "a state setter takes exactly one argument");
                    return None;
                };
                let argument = self.argument_expression(argument)?;
                if let Expression::ArrowFunctionExpression(update) = argument.get_inner_expression() {
                    let (Some(param), 1) = (update.params.items.first(), update.params.items.len())
                    else {
                        self.error(update.span, "an updater takes one parameter: `prev => ...`");
                        return None;
                    };
                    let BindingPattern::BindingIdentifier(name) = &param.pattern else {
                        self.error(param.span, "the updater parameter must be a plain name");
                        return None;
                    };
                    let Some(body) = arrow_expression(update) else {
                        self.error(update.span, "an updater is a single expression: `prev => prev + 1`");
                        return None;
                    };
                    let param_index = scope.vars.len();
                    scope.vars.push(name.name.to_string());
                    let value = self.expr(body, scope);
                    scope.vars.pop();
                    return Some(Stmt::Update {
                        slot,
                        param: param_index,
                        value: value?,
                    });
                }
                Some(Stmt::Set {
                    slot,
                    value: self.expr(argument, scope)?,
                })
            }
            Expression::StaticMemberExpression(member)
                if matches!(member.object.get_inner_expression(),
                    Expression::Identifier(id) if id.name == "actions") =>
            {
                let name = member.property.name.as_str();
                let Some(spec) = self.options.actions.iter().find(|known| known.name == name) else {
                    let known: Vec<&str> = self
                        .options
                        .actions
                        .iter()
                        .map(|known| known.name.as_str())
                        .collect();
                    self.error_help(
                        member.property.span,
                        format!("unknown action `{name}`"),
                        format!("registered actions: {}", known.join(", ")),
                    );
                    return None;
                };
                if call.arguments.len() != spec.params.len() {
                    let message = format!(
                        "`{name}` takes {} argument{}, not {}",
                        spec.params.len(),
                        if spec.params.len() == 1 { "" } else { "s" },
                        call.arguments.len()
                    );
                    let help = format!("signature: {}", spec.signature());
                    self.error_help(call.span, message, help);
                    return None;
                }
                let args = call
                    .arguments
                    .iter()
                    .map(|argument| {
                        let argument = self.argument_expression(argument)?;
                        self.expr(argument, scope)
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(Stmt::Action {
                    name: Rc::from(name),
                    args,
                })
            }
            other => {
                self.error_help(
                    other.span(),
                    "handlers may only call state setters and actions",
                    "write `setX(value)` or `actions.name(args)`",
                );
                None
            }
        }
    }

    fn children(&mut self, children: &[JSXChild], scope: &mut Scope, out: &mut Vec<Child>) {
        for child in children {
            match child {
                JSXChild::Text(text) => {
                    let shown = jsx_text(text.value.as_str());
                    if !shown.is_empty() {
                        self.error_help(
                            text.span,
                            "text must be inside <Text>",
                            "wrap it: <Text variant=\"text-md/normal\">...</Text>",
                        );
                    }
                }
                JSXChild::Element(element) => {
                    if let Some((element, _)) = self.element(element, scope, false) {
                        out.push(Child::Element(element));
                    }
                }
                JSXChild::Fragment(fragment) => self.children(&fragment.children, scope, out),
                JSXChild::ExpressionContainer(container) => {
                    if let Some(expression) = container.expression.as_expression() {
                        self.child_expression(expression, scope, out);
                    }
                }
                JSXChild::Spread(spread) => {
                    self.error(spread.span, "spread children are not supported");
                }
            }
        }
    }

    fn child_expression(&mut self, expression: &Expression, scope: &mut Scope, out: &mut Vec<Child>) {
        let expression = expression.get_inner_expression();
        match expression {
            Expression::LogicalExpression(logical)
                if logical.operator == LogicalOperator::And
                    && is_jsx(logical.right.get_inner_expression()) =>
            {
                let (Some(cond), Some(then)) = (
                    self.expr(&logical.left, scope),
                    self.branch(&logical.right, scope),
                ) else {
                    return;
                };
                out.push(Child::If {
                    cond,
                    then,
                    otherwise: Vec::new(),
                });
            }
            Expression::ConditionalExpression(cond)
                if is_jsx(cond.consequent.get_inner_expression())
                    || is_jsx(cond.alternate.get_inner_expression()) =>
            {
                let test = self.expr(&cond.test, scope);
                let consequent = self.branch(&cond.consequent, scope);
                let alternate = match cond.alternate.get_inner_expression() {
                    Expression::NullLiteral(_) => Some(Vec::new()),
                    other => self.branch(other, scope),
                };
                let (Some(test), Some(consequent), Some(alternate)) = (test, consequent, alternate)
                else {
                    return;
                };
                out.push(Child::If {
                    cond: test,
                    then: consequent,
                    otherwise: alternate,
                });
            }
            Expression::CallExpression(call)
                if matches!(call.callee.get_inner_expression(),
                    Expression::StaticMemberExpression(member) if member.property.name == "map") =>
            {
                if let Some(each) = self.each(call, scope) {
                    out.push(each);
                }
            }
            Expression::JSXElement(element) => {
                if let Some((element, _)) = self.element(element, scope, false) {
                    out.push(Child::Element(element));
                }
            }
            Expression::JSXFragment(fragment) => self.children(&fragment.children, scope, out),
            other => self.error_help(
                other.span(),
                "an expression cannot be a child here unless it renders an element",
                "text goes in <Text>; use `cond && <El/>`, `cond ? <A/> : <B/>` or `.map` for elements",
            ),
        }
    }

    fn branch(&mut self, expression: &Expression, scope: &mut Scope) -> Lowered<Vec<Element>> {
        match expression.get_inner_expression() {
            Expression::JSXElement(element) => {
                self.element(element, scope, false).map(|(element, _)| vec![element])
            }
            Expression::JSXFragment(fragment) => {
                let mut children = Vec::new();
                self.children(&fragment.children, scope, &mut children);
                let mut elements = Vec::new();
                for child in children {
                    match child {
                        Child::Element(element) => elements.push(element),
                        _ => {
                            self.error_help(
                                fragment.span,
                                "a fragment branch holds plain elements only",
                                "move the conditional or list out of the fragment",
                            );
                            return None;
                        }
                    }
                }
                Some(elements)
            }
            other => {
                self.error_help(
                    other.span(),
                    "a conditional branch must be an element or a fragment",
                    "use `null` for nothing",
                );
                None
            }
        }
    }

    fn each(&mut self, call: &oxc_ast::ast::CallExpression, scope: &mut Scope) -> Lowered<Child> {
        let Expression::StaticMemberExpression(member) = call.callee.get_inner_expression() else {
            return None;
        };
        let list = self.expr(&member.object, scope);
        let [argument] = call.arguments.as_slice() else {
            self.error(call.span, "`.map` takes one callback");
            return None;
        };
        let callback = self.argument_expression(argument)?.get_inner_expression();
        let Expression::ArrowFunctionExpression(arrow) = callback else {
            self.error_help(
                callback.span(),
                "the `.map` callback must be an inline arrow function",
                "write items.map(item => <div key={item.id}>...</div>)",
            );
            return None;
        };
        let [param] = arrow.params.items.as_slice() else {
            self.error_help(
                arrow.params.span,
                "the `.map` callback takes exactly one parameter, the item",
                "the index is not available; key rows by an id",
            );
            return None;
        };
        let BindingPattern::BindingIdentifier(item_name) = &param.pattern else {
            self.error_help(
                param.span,
                "the `.map` item must be a plain name",
                "destructure inside the row: `item.name`",
            );
            return None;
        };
        let body = arrow_expression(arrow);
        let Some(Expression::JSXElement(row)) = body.map(Expression::get_inner_expression) else {
            self.error_help(
                arrow.span,
                "the `.map` callback must return a single element",
                "return the row element directly",
            );
            return None;
        };

        let item = scope.vars.len();
        scope.vars.push(item_name.name.to_string());
        let lowered = self.element(row, scope, true);
        scope.vars.pop();
        let (body, key) = lowered?;
        let Some(key) = key else {
            self.error_help(
                row.opening_element.span,
                "rows in `.map` need a `key`",
                "add key={item.id}; a missing key is an error here, not a warning",
            );
            return None;
        };
        Some(Child::Each {
            list: list?,
            key,
            item,
            body: Box::new(body),
        })
    }
}

fn arrow_expression<'a, 'b>(arrow: &'b ArrowFunctionExpression<'a>) -> Option<&'b Expression<'a>> {
    match &arrow.body {
        ArrowFunctionBody::FunctionBody(body) => match body.statements.as_slice() {
            [Statement::ReturnStatement(ret)] => ret.argument.as_ref(),
            [Statement::ExpressionStatement(expression)] => Some(&expression.expression),
            _ => None,
        },
        expression => expression.as_expression(),
    }
}

fn renders_element(expression: &Expression) -> bool {
    match expression {
        Expression::JSXElement(_) | Expression::JSXFragment(_) => true,
        Expression::LogicalExpression(logical) => {
            logical.operator == LogicalOperator::And && is_jsx(logical.right.get_inner_expression())
        }
        Expression::ConditionalExpression(cond) => {
            is_jsx(cond.consequent.get_inner_expression())
                || is_jsx(cond.alternate.get_inner_expression())
        }
        Expression::CallExpression(call) => matches!(call.callee.get_inner_expression(),
            Expression::StaticMemberExpression(member) if member.property.name == "map"),
        _ => false,
    }
}

fn intern(name: &str) -> &'static str {
    static NAMES: Mutex<BTreeSet<&'static str>> = Mutex::new(BTreeSet::new());
    let Ok(mut names) = NAMES.lock() else {
        return Box::leak(name.to_string().into_boxed_str());
    };
    if let Some(known) = names.get(name) {
        return known;
    }
    let leaked: &'static str = Box::leak(name.to_string().into_boxed_str());
    names.insert(leaked);
    leaked
}

fn state_path(expression: &Expression) -> Option<Vec<String>> {
    match expression.get_inner_expression() {
        Expression::Identifier(id) if id.name == "state" => Some(Vec::new()),
        Expression::StaticMemberExpression(member) => {
            let mut path = state_path(&member.object)?;
            path.push(member.property.name.to_string());
            Some(path)
        }
        _ => None,
    }
}

fn is_jsx(expression: &Expression) -> bool {
    matches!(expression, Expression::JSXElement(_) | Expression::JSXFragment(_))
}

fn jsx_text(raw: &str) -> String {
    let raw = raw.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<&str> = raw.split('\n').collect();
    let last = lines.len() - 1;
    let mut kept = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let line = line.replace('\t', " ");
        let mut line = line.as_str();
        if index != 0 {
            line = line.trim_start_matches(' ');
        }
        if index != last {
            line = line.trim_end_matches(' ');
        }
        if !line.is_empty() {
            kept.push(line.to_string());
        }
    }
    kept.join(" ")
}
