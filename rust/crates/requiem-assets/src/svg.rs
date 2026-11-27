use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SvgIcon {
    pub view_box: String,
    pub paths: Vec<SvgPath>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<SvgElement>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SvgElement {
    pub tag: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<SvgNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SvgNode {
    Element(SvgElement),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SvgPath {
    pub d: String,
    pub fill_rule: Option<String>,
    pub clip_rule: Option<String>,
}

impl SvgIcon {
    pub fn to_svg(&self) -> Result<String, Error> {
        if let Some(issue) = self.issues.first() {
            return Err(Error::Unresolved(format!(
                "cannot be written as a whole SVG: {issue}"
            )));
        }
        let root = self
            .root
            .as_ref()
            .ok_or_else(|| Error::Unresolved("no markup was captured".into()))?;
        let mut out = String::new();
        write_element(root, &mut out, true);
        Ok(out)
    }
}

fn write_element(element: &SvgElement, out: &mut String, root: bool) {
    out.push('<');
    out.push_str(&element.tag);
    if root {
        out.push_str(" xmlns=\"http://www.w3.org/2000/svg\"");
    }
    for (name, value) in &element.attrs {
        out.push(' ');
        out.push_str(name);
        out.push_str("=\"");
        out.push_str(
            &value
                .replace('&', "&amp;")
                .replace('"', "&quot;")
                .replace('<', "&lt;"),
        );
        out.push('"');
    }
    if element.children.is_empty() {
        out.push_str("/>");
        return;
    }
    out.push('>');
    for child in &element.children {
        match child {
            SvgNode::Element(inner) => write_element(inner, out, false),
            SvgNode::Text(text) => out.push_str(&text.replace('&', "&amp;").replace('<', "&lt;")),
        }
    }
    out.push_str("</");
    out.push_str(&element.tag);
    out.push('>');
}

fn read_defaults(function: &Function<'_>, out: &mut HashMap<String, Option<String>>) {
    fn from_pattern(pattern: &BindingPattern<'_>, out: &mut HashMap<String, Option<String>>) {
        let BindingPattern::ObjectPattern(object) = pattern else {
            return;
        };
        for prop in &object.properties {
            let BindingPattern::AssignmentPattern(with_default) = &prop.value else {
                continue;
            };
            let BindingPattern::BindingIdentifier(local) = &with_default.left else {
                continue;
            };
            let default = match with_default.right.without_parentheses() {
                Expression::StringLiteral(lit) => Some(lit.value.to_string()),
                _ => None,
            };
            out.insert(local.name.to_string(), default);
        }
    }
    for param in &function.params.items {
        from_pattern(&param.pattern, out);
    }
    if let Some(body) = &function.body {
        for statement in &body.statements {
            if let Statement::VariableDeclaration(declaration) = statement {
                for d in &declaration.declarations {
                    from_pattern(&d.id, out);
                }
            }
        }
    }
}

pub fn summarize(source: &str) -> Result<SvgIcon, Error> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    if parsed.fatal_error {
        return Err(Error::Parse("unrecoverable syntax error".into()));
    }
    let mut collector = Collector::default();
    for statement in &parsed.program.body {
        if let Statement::FunctionDeclaration(function) = statement {
            read_defaults(function, &mut collector.defaults);
            break;
        }
    }
    collector.visit_program(&parsed.program);
    collector.finish()
}

pub(crate) fn summarize_function(function: &Function<'_>) -> Result<SvgIcon, Error> {
    let mut collector = Collector::default();
    read_defaults(function, &mut collector.defaults);
    if let Some(body) = &function.body {
        collector.visit_function_body(body);
    }
    collector.finish()
}

#[derive(Default)]
struct Collector {
    view_box: Option<String>,
    paths: Vec<SvgPath>,
    dynamic_path: bool,
    root: Option<SvgElement>,
    issues: Vec<String>,

    defaults: HashMap<String, Option<String>>,
}

impl Collector {
    fn finish(self) -> Result<SvgIcon, Error> {
        if self.dynamic_path {
            return Err(Error::Unresolved("a <path> has a non-literal `d`".into()));
        }
        let view_box = self
            .view_box
            .ok_or_else(|| Error::Unresolved("no <svg> with a literal viewBox".into()))?;

        if self.paths.is_empty()
            && self
                .root
                .as_ref()
                .is_none_or(|root| root.children.is_empty())
        {
            return Err(Error::Unresolved("no shapes".into()));
        }
        Ok(SvgIcon {
            view_box,
            paths: self.paths,
            root: self.root,
            issues: self.issues,
        })
    }
}

fn is_jsx_callee(callee: &Expression<'_>) -> bool {
    let name = match callee.without_parentheses() {
        Expression::SequenceExpression(seq) => match seq.expressions.last() {
            Some(Expression::StaticMemberExpression(m)) => m.property.name.as_str(),
            Some(Expression::Identifier(id)) => id.name.as_str(),
            _ => return false,
        },
        Expression::StaticMemberExpression(m) => m.property.name.as_str(),
        Expression::Identifier(id) => id.name.as_str(),
        _ => return false,
    };
    matches!(name, "jsx" | "jsxs")
}

fn prop_name<'a>(key: &'a PropertyKey<'_>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(s) => Some(s.value.as_str()),
        _ => None,
    }
}

impl<'a> Visit<'a> for Collector {
    fn visit_switch_statement(&mut self, switch: &SwitchStatement<'a>) {
        let Some(default) = switch.cases.iter().find(|case| case.test.is_none()) else {
            walk::walk_switch_statement(self, switch);
            return;
        };
        for statement in &default.consequent {
            self.visit_statement(statement);
        }
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_jsx_callee(&call.callee) {
            if let (Some(Argument::StringLiteral(tag)), Some(Argument::ObjectExpression(props))) =
                (call.arguments.first(), call.arguments.get(1))
            {
                self.element(tag.value.as_str(), props);
                if tag.value == "svg" && self.root.is_none() {
                    self.root = Some(build_element(call, true, &self.defaults, &mut self.issues));
                }
            }
        }
        walk::walk_call_expression(self, call);
    }
}

impl Collector {
    fn element(&mut self, tag: &str, props: &ObjectExpression<'_>) {
        let literal = |wanted: &str| -> Option<Option<String>> {
            props.properties.iter().find_map(|p| match p {
                ObjectPropertyKind::ObjectProperty(p) if prop_name(&p.key) == Some(wanted) => {
                    Some(match &p.value {
                        Expression::StringLiteral(s) => Some(s.value.to_string()),
                        _ => None,
                    })
                }
                _ => None,
            })
        };
        match tag {
            "svg" => {
                if self.view_box.is_none() {
                    if let Some(Some(vb)) = literal("viewBox") {
                        self.view_box = Some(vb);
                    }
                }
            }
            "path" => match literal("d") {
                Some(Some(d)) => self.paths.push(SvgPath {
                    d,
                    fill_rule: literal("fillRule").flatten(),
                    clip_rule: literal("clipRule").flatten(),
                }),
                _ => self.dynamic_path = true,
            },
            _ => {}
        }
    }
}

const CAMEL_ATTRS: &[&str] = &[
    "viewBox",
    "preserveAspectRatio",
    "gradientUnits",
    "gradientTransform",
    "clipPathUnits",
    "maskUnits",
    "maskContentUnits",
    "patternUnits",
    "patternContentUnits",
    "patternTransform",
    "spreadMethod",
    "stdDeviation",
    "filterUnits",
    "primitiveUnits",
    "markerUnits",
    "markerWidth",
    "markerHeight",
    "refX",
    "refY",
    "startOffset",
    "textLength",
    "baseFrequency",
    "numOctaves",
    "edgeMode",
    "kernelMatrix",
    "tableValues",
    "calcMode",
    "keyTimes",
    "keySplines",
    "repeatCount",
    "attributeName",
];

const PLUMBING_ATTRS: &[&str] = &[
    "className",
    "style",
    "ref",
    "key",
    "role",
    "focusable",
    "aria-hidden",
    "aria-label",
    "data-testid",
    "id_",
    "xmlns",
    "xmlnsXlink",
];

fn svg_attr_name(react: &str) -> String {
    if CAMEL_ATTRS.contains(&react) {
        return react.to_string();
    }
    if react == "xlinkHref" {
        return "xlink:href".into();
    }
    let mut out = String::new();
    for c in react.chars() {
        if c.is_ascii_uppercase() {
            out.push('-');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn number_text(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

fn build_element(
    call: &CallExpression<'_>,
    root: bool,
    defaults: &HashMap<String, Option<String>>,
    issues: &mut Vec<String>,
) -> SvgElement {
    let tag = match call.arguments.first() {
        Some(Argument::StringLiteral(tag)) => tag.value.to_string(),
        _ => {
            issues.push("an element whose tag is not a string literal".into());
            "g".into()
        }
    };
    let mut element = SvgElement {
        tag,
        attrs: Vec::new(),
        children: Vec::new(),
    };
    let Some(Argument::ObjectExpression(props)) = call.arguments.get(1) else {
        return element;
    };
    for property in &props.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        let Some(name) = prop_name(&property.key) else {
            continue;
        };
        if name == "children" {
            collect_children(&property.value, &mut element.children, defaults, issues);
            continue;
        }
        if PLUMBING_ATTRS.contains(&name) || (root && matches!(name, "width" | "height")) {
            continue;
        }
        let attr = svg_attr_name(name);
        let value = match property.value.without_parentheses() {
            Expression::StringLiteral(s) => s.value.to_string(),
            Expression::NumericLiteral(n) => number_text(n.value),
            Expression::BooleanLiteral(_) | Expression::NullLiteral(_) => continue,
            Expression::Identifier(id) if id.name == "undefined" => continue,

            expression
                if matches!(attr.as_str(), "fill" | "stroke" | "color" | "stop-color")
                    && is_color_param(expression) =>
            {
                param_name(expression)
                    .and_then(|name| defaults.get(name))
                    .cloned()
                    .flatten()
                    .unwrap_or_else(|| "currentColor".to_string())
            }
            _ => {
                issues.push(format!("<{}> `{name}` is computed", element.tag));
                continue;
            }
        };
        element.attrs.push((attr, value));
    }
    element
}

fn is_color_param(expression: &Expression<'_>) -> bool {
    match expression.without_parentheses() {
        Expression::Identifier(_) => true,
        Expression::StaticMemberExpression(member) => is_color_param(&member.object),
        Expression::ConditionalExpression(c) => {
            is_typeof_test(&c.test) && is_color_param(&c.consequent) && is_color_param(&c.alternate)
        }
        Expression::LogicalExpression(l) => {
            is_color_param(&l.left)
                && (is_color_param(&l.right)
                    || matches!(l.right.without_parentheses(), Expression::StringLiteral(s) if s.value == "currentColor"))
        }
        _ => false,
    }
}

fn param_name<'a>(expression: &'a Expression<'_>) -> Option<&'a str> {
    match expression.without_parentheses() {
        Expression::Identifier(id) => Some(id.name.as_str()),
        Expression::StaticMemberExpression(member) => param_name(&member.object),
        Expression::ConditionalExpression(c) => param_name(&c.consequent),
        Expression::LogicalExpression(l) => param_name(&l.left),
        _ => None,
    }
}

fn is_typeof_test(test: &Expression<'_>) -> bool {
    let Expression::BinaryExpression(b) = test.without_parentheses() else {
        return false;
    };
    let is_typeof = |e: &Expression<'_>| matches!(e.without_parentheses(), Expression::UnaryExpression(u) if u.operator == UnaryOperator::Typeof);
    let is_string =
        |e: &Expression<'_>| matches!(e.without_parentheses(), Expression::StringLiteral(_));
    (is_typeof(&b.left) && is_string(&b.right)) || (is_string(&b.left) && is_typeof(&b.right))
}

fn collect_children(
    expression: &Expression<'_>,
    out: &mut Vec<SvgNode>,
    defaults: &HashMap<String, Option<String>>,
    issues: &mut Vec<String>,
) {
    match expression.without_parentheses() {
        Expression::CallExpression(call) if is_jsx_callee(&call.callee) => out.push(
            SvgNode::Element(build_element(call, false, defaults, issues)),
        ),
        Expression::ArrayExpression(array) => {
            for item in &array.elements {
                match item.as_expression() {
                    Some(item) => collect_children(item, out, defaults, issues),
                    None => issues.push("a spread or hole among the children".into()),
                }
            }
        }
        Expression::StringLiteral(text) => out.push(SvgNode::Text(text.value.to_string())),
        Expression::NullLiteral(_) | Expression::BooleanLiteral(_) => {}
        _ => issues.push("a child that is not a literal element".into()),
    }
}

pub fn decode_string_literal(literal: &str) -> Result<String, Error> {
    let wrapped = format!("({literal})");
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &wrapped, SourceType::mjs()).parse();
    if parsed.fatal_error {
        return Err(Error::Parse("unrecoverable syntax error".into()));
    }
    match parsed.program.body.first() {
        Some(Statement::ExpressionStatement(s)) => match s.expression.without_parentheses() {
            Expression::StringLiteral(lit) => Ok(lit.value.to_string()),
            _ => Err(Error::Parse("not a string literal".into())),
        },
        _ => Err(Error::Parse("not a string literal".into())),
    }
}
