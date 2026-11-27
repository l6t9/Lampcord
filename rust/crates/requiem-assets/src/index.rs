use std::collections::HashMap;

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};

use crate::Error;
use crate::resolve;
use crate::svg::{self, SvgIcon};

#[derive(Debug, Clone)]
pub enum ExportTarget {
    Local(String),

    Member { binding: String, property: String },
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub start: u32,
    pub end: u32,

    pub icon: Option<Result<SvgIcon, String>>,
}

#[derive(Debug, Clone, Default)]
pub struct Module {
    pub id: String,
    pub start: u32,
    pub end: u32,
    pub requires: HashMap<String, String>,
    pub exports: HashMap<String, ExportTarget>,
    pub functions: HashMap<String, FunctionInfo>,

    pub asset_file: Option<String>,

    pub context: Option<Vec<(String, String)>>,

    pub json: Option<JsonText>,

    pub constants: HashMap<String, String>,

    pub param_defaults: Vec<(String, ParamDefault)>,

    pub path_literals: Vec<PathLiteral>,
}

#[derive(Debug, Clone)]
pub struct PathLiteral {
    pub start: u32,
    pub end: u32,

    pub head: Vec<String>,

    pub value: String,
}

#[derive(Debug, Clone)]
pub enum ParamDefault {
    Binding(String),

    File(String),
}

#[derive(Debug, Clone)]
pub struct JsonText {
    pub start: u32,
    pub end: u32,

    pub head: String,
}

#[derive(Debug, Default)]
pub struct ChunkIndex {
    pub modules: Vec<Module>,
}

pub fn index_chunk(source: &str) -> Result<ChunkIndex, Error> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    if parsed.fatal_error {
        return Err(Error::Parse("unrecoverable syntax error".into()));
    }
    let mut scanner = Scanner::default();
    scanner.visit_program(&parsed.program);
    Ok(ChunkIndex {
        modules: scanner.modules,
    })
}

#[derive(Default)]
struct Scanner {
    modules: Vec<Module>,
}

fn module_id(key: &PropertyKey<'_>) -> Option<String> {
    match key {
        PropertyKey::NumericLiteral(n) => Some(format!("{}", n.value as u64)),
        PropertyKey::StringLiteral(s) if s.value.bytes().all(|b| b.is_ascii_digit()) => {
            Some(s.value.to_string())
        }
        _ => None,
    }
}

impl<'a> Visit<'a> for Scanner {
    fn visit_object_property(&mut self, prop: &ObjectProperty<'a>) {
        if let Some(id) = module_id(&prop.key) {
            let body = match &prop.value {
                Expression::FunctionExpression(f) => f.body.as_deref(),
                Expression::ArrowFunctionExpression(a) => match &a.body {
                    ArrowFunctionBody::FunctionBody(b) => Some(&**b),
                    _ => None,
                },
                _ => None,
            };
            if let Some(body) = body {
                let span = prop.span();
                let mut module = Module {
                    id,
                    start: span.start,
                    end: span.end,
                    ..Default::default()
                };
                scan_module_body(body, &mut module);
                let mut literals = LiteralCollector::default();
                literals.visit_function_body(body);
                module.path_literals = literals.found;
                self.modules.push(module);

                return;
            }
        }
        walk::walk_object_property(self, prop);
    }
}

#[derive(Default)]
struct LiteralCollector {
    found: Vec<PathLiteral>,
}

impl<'a> Visit<'a> for LiteralCollector {
    fn visit_string_literal(&mut self, lit: &StringLiteral<'a>) {
        let value = lit.value.as_str();
        let mut chars = value.chars();
        let starts_with_move = matches!(chars.next(), Some('M' | 'm'));
        if value.len() >= 30
            && starts_with_move
            && value
                .chars()
                .all(|c| c.is_ascii_digit() || " .,-+eEaAcClLhHqQsStTvVzZmM".contains(c))
        {
            let mut head = resolve::path_tokens(value);
            head.truncate(14);
            self.found.push(PathLiteral {
                start: lit.span.start,
                end: lit.span.end,
                head,
                value: value.to_string(),
            });
        }
    }
}

fn scan_module_body(body: &FunctionBody<'_>, module: &mut Module) {
    for stmt in &body.statements {
        match stmt {
            Statement::VariableDeclaration(decl) => {
                for d in &decl.declarations {
                    let BindingPattern::BindingIdentifier(name) = &d.id else {
                        continue;
                    };
                    match &d.init {
                        Some(Expression::CallExpression(call)) => {
                            if let Some(id) = require_id(call) {
                                module.requires.insert(name.name.to_string(), id);
                            }
                        }
                        Some(Expression::ObjectExpression(obj)) => {
                            if let Some(table) = context_table(obj) {
                                module.context = Some(table);
                            }
                        }

                        Some(Expression::BinaryExpression(bin)) => {
                            if let (
                                Expression::StaticMemberExpression(m),
                                Expression::StringLiteral(lit),
                            ) = (&bin.left, &bin.right)
                            {
                                if m.property.name == "p" {
                                    module
                                        .constants
                                        .insert(name.name.to_string(), lit.value.to_string());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Statement::FunctionDeclaration(f) => {
                for param in &f.params.items {
                    collect_param_defaults(&param.pattern, &mut module.param_defaults);
                }

                if let Some(body) = &f.body {
                    for inner in &body.statements {
                        if let Statement::VariableDeclaration(decl) = inner {
                            for d in &decl.declarations {
                                collect_param_defaults(&d.id, &mut module.param_defaults);
                            }
                        }
                    }
                }
                if let Some(id) = &f.id {
                    let icon = match svg::summarize_function(f) {
                        Err(Error::Unresolved(msg)) if msg.contains("no <svg>") => None,
                        other => Some(other.map_err(|e| e.to_string())),
                    };
                    module.functions.insert(
                        id.name.to_string(),
                        FunctionInfo {
                            start: f.span.start,
                            end: f.span.end,
                            icon,
                        },
                    );
                }
            }
            Statement::ExpressionStatement(s) => scan_expression(&s.expression, module),
            _ => {}
        }
    }
}

fn collect_param_defaults(pattern: &BindingPattern<'_>, out: &mut Vec<(String, ParamDefault)>) {
    let BindingPattern::ObjectPattern(object) = pattern else {
        return;
    };
    for prop in &object.properties {
        let PropertyKey::StaticIdentifier(key) = &prop.key else {
            continue;
        };
        let BindingPattern::AssignmentPattern(with_default) = &prop.value else {
            continue;
        };
        match &with_default.right {
            Expression::Identifier(id) => out.push((
                key.name.to_string(),
                ParamDefault::Binding(id.name.to_string()),
            )),
            Expression::StringLiteral(lit) => {
                if let Some(file) = lit.value.strip_prefix("/assets/") {
                    out.push((key.name.to_string(), ParamDefault::File(file.to_string())));
                }
            }
            _ => {}
        }
    }
}

fn scan_expression(expr: &Expression<'_>, module: &mut Module) {
    match expr {
        Expression::SequenceExpression(seq) => {
            for e in &seq.expressions {
                scan_expression(e, module);
            }
        }
        Expression::CallExpression(call) => {
            if let Expression::StaticMemberExpression(callee) = &call.callee {
                if callee.property.name == "d" {
                    if let Some(Argument::ObjectExpression(map)) = call.arguments.get(1) {
                        read_export_map(map, module);
                    }
                }
            }
        }
        Expression::AssignmentExpression(assign) => {
            if let Expression::StringLiteral(s) = &assign.right {
                if let Some(file) = s.value.strip_prefix("/assets/") {
                    module.asset_file = Some(file.to_string());
                }
            }

            if let Expression::CallExpression(call) = &assign.right {
                if matches!(&call.callee, Expression::StaticMemberExpression(m) if m.property.name == "parse")
                {
                    if let Some(Argument::StringLiteral(text)) = call.arguments.first() {
                        if text.value.starts_with('{') {
                            module.json = Some(JsonText {
                                start: text.span.start,
                                end: text.span.end,
                                head: text.value.chars().take(600).collect(),
                            });
                        }
                    }
                }
            }

            if let Expression::BinaryExpression(bin) = &assign.right {
                if let Expression::StringLiteral(s) = &bin.right {
                    if matches!(&bin.left, Expression::StaticMemberExpression(m) if m.property.name == "p")
                    {
                        module.asset_file = Some(s.value.to_string());
                    }
                }
            }
        }
        _ => {}
    }
}

fn read_export_map(map: &ObjectExpression<'_>, module: &mut Module) {
    for prop in &map.properties {
        let ObjectPropertyKind::ObjectProperty(p) = prop else {
            continue;
        };
        let name = match &p.key {
            PropertyKey::StaticIdentifier(id) => id.name.to_string(),
            PropertyKey::StringLiteral(s) => s.value.to_string(),
            _ => continue,
        };
        let target_expr = match &p.value {
            Expression::ArrowFunctionExpression(a) => match &a.body {
                ArrowFunctionBody::FunctionBody(b) => match b.statements.first() {
                    Some(Statement::ExpressionStatement(s)) => Some(&s.expression),
                    _ => None,
                },
                _ => a.get_expression(),
            },
            _ => None,
        };
        let target = match target_expr.map(|e| e.without_parentheses()) {
            Some(Expression::Identifier(id)) => ExportTarget::Local(id.name.to_string()),
            Some(Expression::StaticMemberExpression(m)) => match &m.object {
                Expression::Identifier(obj) => ExportTarget::Member {
                    binding: obj.name.to_string(),
                    property: m.property.name.to_string(),
                },
                _ => continue,
            },
            _ => continue,
        };
        module.exports.insert(name, target);
    }
}

fn require_id(call: &CallExpression<'_>) -> Option<String> {
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if callee.name != "n" || call.arguments.len() != 1 {
        return None;
    }
    match call.arguments.first()? {
        Argument::NumericLiteral(n) => Some(format!("{}", n.value as u64)),
        _ => None,
    }
}

fn context_table(obj: &ObjectExpression<'_>) -> Option<Vec<(String, String)>> {
    let mut out = Vec::new();
    for prop in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(p) = prop else {
            return None;
        };
        let PropertyKey::StringLiteral(k) = &p.key else {
            return None;
        };
        let Expression::StringLiteral(v) = &p.value else {
            return None;
        };
        if !k.value.starts_with("./") {
            return None;
        }
        out.push((k.value.to_string(), v.value.to_string()));
    }
    (!out.is_empty()).then_some(out)
}
