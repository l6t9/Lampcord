use std::collections::BTreeMap;
use std::sync::OnceLock;

use fluent_syntax::ast::{
    Entry, Expression, InlineExpression, Pattern, PatternElement, VariantKey,
};
use fluent_syntax::parser;

use crate::{FALLBACK, LOCALES};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgKind {
    Number,
    Text,
    Any,
}

impl ArgKind {
    fn merge(self, other: ArgKind) -> ArgKind {
        match (self, other) {
            (a, b) if a == b => a,
            (ArgKind::Any, b) => b,
            (a, ArgKind::Any) => a,
            _ => ArgKind::Any,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arg {
    pub name: String,
    pub kind: ArgKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub key: String,
    pub args: Vec<Arg>,
}

const PLURAL_CATEGORIES: [&str; 6] = ["zero", "one", "two", "few", "many", "other"];

#[derive(Default)]
struct Found {
    args: BTreeMap<String, ArgKind>,
    references: Vec<String>,
}

impl Found {
    fn note(&mut self, name: &str, kind: ArgKind) {
        self.args
            .entry(name.to_string())
            .and_modify(|known| *known = known.merge(kind))
            .or_insert(kind);
    }
}

fn walk_pattern(pattern: &Pattern<&str>, found: &mut Found) {
    for element in &pattern.elements {
        if let PatternElement::Placeable { expression } = element {
            walk_expression(expression, found);
        }
    }
}

fn walk_expression(expression: &Expression<&str>, found: &mut Found) {
    match expression {
        Expression::Inline(inline) => walk_inline(inline, ArgKind::Any, found),
        Expression::Select { selector, variants } => {
            let numeric = variants.iter().all(|variant| match &variant.key {
                VariantKey::NumberLiteral { .. } => true,
                VariantKey::Identifier { name } => PLURAL_CATEGORIES.contains(name),
            });
            walk_inline(selector, if numeric { ArgKind::Number } else { ArgKind::Text }, found);
            for variant in variants {
                walk_pattern(&variant.value, found);
            }
        }
    }
}

fn walk_inline(inline: &InlineExpression<&str>, as_kind: ArgKind, found: &mut Found) {
    match inline {
        InlineExpression::VariableReference { id } => found.note(id.name, as_kind),
        InlineExpression::FunctionReference { id, arguments } => {
            let kind = if id.name == "NUMBER" { ArgKind::Number } else { ArgKind::Any };
            for positional in &arguments.positional {
                walk_inline(positional, kind, found);
            }
            for named in &arguments.named {
                walk_inline(&named.value, ArgKind::Any, found);
            }
        }
        InlineExpression::MessageReference { id, attribute } => {
            found.references.push(match attribute {
                Some(attribute) => format!("{}.{}", id.name, attribute.name),
                None => id.name.to_string(),
            });
        }
        InlineExpression::TermReference { arguments, .. } => {
            if let Some(arguments) = arguments {
                for named in &arguments.named {
                    walk_inline(&named.value, ArgKind::Any, found);
                }
            }
        }
        InlineExpression::Placeable { expression } => walk_expression(expression, found),
        InlineExpression::StringLiteral { .. } | InlineExpression::NumberLiteral { .. } => {}
    }
}

fn build() -> Vec<Message> {
    let sources = LOCALES
        .iter()
        .find(|(tag, _)| *tag == FALLBACK)
        .map_or(&[][..], |(_, sources)| *sources);
    let mut found: BTreeMap<String, Found> = BTreeMap::new();
    for source in sources {
        let resource = match parser::parse(*source) {
            Ok(resource) => resource,
            Err((resource, _)) => resource,
        };
        for entry in &resource.body {
            let Entry::Message(message) = entry else {
                continue;
            };
            if let Some(value) = &message.value {
                let mut entry_found = Found::default();
                walk_pattern(value, &mut entry_found);
                found.insert(message.id.name.to_string(), entry_found);
            }
            for attribute in &message.attributes {
                let mut entry_found = Found::default();
                walk_pattern(&attribute.value, &mut entry_found);
                found.insert(
                    format!("{}.{}", message.id.name, attribute.id.name),
                    entry_found,
                );
            }
        }
    }
    loop {
        let mut changed = false;
        let snapshot: BTreeMap<String, BTreeMap<String, ArgKind>> = found
            .iter()
            .map(|(key, entry)| (key.clone(), entry.args.clone()))
            .collect();
        for entry in found.values_mut() {
            for reference in entry.references.clone() {
                let Some(referenced) = snapshot.get(&reference) else {
                    continue;
                };
                for (name, kind) in referenced {
                    let before = entry.args.get(name).copied();
                    entry.note(name, *kind);
                    changed |= before != entry.args.get(name).copied();
                }
            }
        }
        if !changed {
            break;
        }
    }
    found
        .into_iter()
        .map(|(key, entry)| Message {
            key,
            args: entry
                .args
                .into_iter()
                .map(|(name, kind)| Arg { name, kind })
                .collect(),
        })
        .collect()
}

pub fn catalog() -> &'static [Message] {
    static CATALOG: OnceLock<Vec<Message>> = OnceLock::new();
    CATALOG.get_or_init(build)
}

pub fn message(key: &str) -> Option<&'static Message> {
    catalog().iter().find(|message| message.key == key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_placeables_accept_text_or_numbers() {
        let message = message("threads-browser-joined").expect("joined");
        assert_eq!(
            message.args,
            [Arg {
                name: "count".into(),
                kind: ArgKind::Any
            }]
        );
    }

    #[test]
    fn messages_without_variables_have_no_args() {
        assert!(message("threads-browser-title").expect("title").args.is_empty());
    }

    #[test]
    fn plural_selectors_are_numbers_and_named_selectors_text() {
        let source = "pluralized = { $n ->\n    [one] one thing\n   *[other] { $n } things\n}\nnamed = { $kind ->\n    [fruit] a fruit\n   *[other] something\n}\n";
        let (resource, _) = match parser::parse(source) {
            Ok(resource) => (resource, ()),
            Err((resource, _)) => (resource, ()),
        };
        let mut kinds = BTreeMap::new();
        for entry in &resource.body {
            if let Entry::Message(message) = entry {
                let mut found = Found::default();
                walk_pattern(message.value.as_ref().expect("value"), &mut found);
                kinds.insert(message.id.name.to_string(), found.args);
            }
        }
        assert_eq!(kinds["pluralized"]["n"], ArgKind::Number);
        assert_eq!(kinds["named"]["kind"], ArgKind::Text);
    }

    #[test]
    fn every_key_that_exists_at_runtime_is_in_the_catalogue() {
        assert!(catalog().len() > 100);
        assert!(catalog().iter().all(|message| crate::has(&message.key)));
    }
}
