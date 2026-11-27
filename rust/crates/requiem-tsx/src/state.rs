use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use specta::datatype::{DataType, Reference};
use specta::{Type, Types};
use specta_typescript::Typescript;
use specta_typescript::semantic::Configuration;

use crate::components::TYPES_MODULE;
use crate::value::Value;

pub type QueryFn = Rc<dyn Fn(&[Value]) -> Value>;

#[derive(Clone, Debug, PartialEq)]
pub struct QuerySpec {
    pub path: String,
    pub params: Vec<(String, String)>,
    pub returns: String,
}

impl QuerySpec {
    pub fn signature(&self) -> String {
        let params: Vec<String> = self
            .params
            .iter()
            .map(|(name, ts)| format!("{name}: {ts}"))
            .collect();
        let name = self.path.rsplit('.').next().unwrap_or(&self.path);
        format!("{name}({}): {}", params.join(", "), self.returns)
    }
}

#[derive(Clone, Default)]
pub struct StateQueries {
    entries: Vec<(QuerySpec, QueryFn)>,
    types: Types,
    brands: Vec<String>,
}

impl StateQueries {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn brand(mut self, name: &str) -> Self {
        if !self.brands.iter().any(|known| known == name) {
            self.brands.push(name.to_string());
        }
        self
    }

    fn name_of<T: Type>(&mut self) -> String {
        match T::definition(&mut self.types) {
            DataType::Reference(Reference::Named(named)) => self
                .types
                .get(&named)
                .map(|ndt| ndt.name.to_string())
                .expect("a registered type has a name"),
            _ => panic!("state queries return named structs"),
        }
    }

    fn push(&mut self, path: &str, params: &[(&str, &str)], returns: String, run: QueryFn) {
        let spec = QuerySpec {
            path: path.to_string(),
            params: params
                .iter()
                .map(|(name, ts)| (name.to_string(), ts.to_string()))
                .collect(),
            returns,
        };
        self.entries.push((spec, run));
    }

    pub fn one<T: Type + Serialize + 'static>(
        mut self,
        path: &str,
        params: &[(&str, &str)],
        read: impl Fn(&[Value]) -> Option<T> + 'static,
    ) -> Self {
        let returns = format!("{} | null", self.name_of::<T>());
        let run: QueryFn = Rc::new(move |args| match read(args) {
            Some(item) => to_value(&item),
            None => Value::Null,
        });
        self.push(path, params, returns, run);
        self
    }

    pub fn scalar<T: Serialize + 'static>(
        mut self,
        path: &str,
        params: &[(&str, &str)],
        returns: &str,
        read: impl Fn(&[Value]) -> T + 'static,
    ) -> Self {
        let run: QueryFn = Rc::new(move |args| to_value(&read(args)));
        self.push(path, params, returns.to_string(), run);
        self
    }

    pub fn list<T: Type + Serialize + 'static>(
        mut self,
        path: &str,
        params: &[(&str, &str)],
        read: impl Fn(&[Value]) -> Vec<T> + 'static,
    ) -> Self {
        let returns = format!("{}[]", self.name_of::<T>());
        let run: QueryFn = Rc::new(move |args| {
            Value::list(read(args).iter().map(to_value))
        });
        self.push(path, params, returns, run);
        self
    }

    pub fn specs(&self) -> Vec<QuerySpec> {
        self.entries.iter().map(|(spec, _)| spec.clone()).collect()
    }

    pub fn runtime(&self) -> HashMap<String, QueryFn> {
        self.entries
            .iter()
            .map(|(spec, run)| (spec.path.clone(), run.clone()))
            .collect()
    }

    pub fn call(&self, path: &str, args: &[Value]) -> Option<Value> {
        self.entries
            .iter()
            .find(|(spec, _)| spec.path == path)
            .map(|(_, run)| run(args))
    }

    pub fn export<T: Type>(mut self) -> Self {
        self.types.register_mut::<T>();
        self
    }

    pub fn qualify(&self, text: &str) -> String {
        qualify_names(text, &self.shared().names)
    }

    fn shared(&self) -> Shared {
        let discovered: Arc<Mutex<BTreeSet<String>>> = Arc::default();
        let sink = discovered.clone();
        let types = Configuration::empty()
            .enable_lossless_floats()
            .apply_types(&self.types);
        let exported = Typescript::new()
            .header("")
            .branded_type_impl(move |_, branded| {
                if let Ok(mut found) = sink.lock() {
                    found.insert(branded.brand().to_string());
                }
                Ok(Cow::Owned(branded.brand().to_string()))
            })
            .export(&types, specta_serde::Format)
            .expect("shared types export");
        let mut brands: BTreeSet<String> = self.brands.iter().cloned().collect();
        if let Ok(found) = discovered.lock() {
            brands.extend(found.iter().cloned());
        }
        let mut names: Vec<String> = brands.iter().cloned().collect();
        names.extend(exported.lines().filter_map(|line| {
            let rest = line.trim_start().strip_prefix("export type ")?;
            let end = rest.find(|c: char| !c.is_alphanumeric() && c != '_')?;
            Some(rest[..end].to_string())
        }));
        Shared {
            exported,
            brands,
            names,
        }
    }

    pub fn declarations(&self) -> String {
        let (types, state) = self.declaration_parts();
        types + &state
    }

    pub fn declaration_parts(&self) -> (String, String) {
        if self.entries.is_empty() && self.brands.is_empty() && self.types.is_empty() {
            return (String::new(), String::new());
        }
        let shared = self.shared();
        let mut out = String::from("\ndeclare namespace Types {\n");
        for brand in &shared.brands {
            out.push_str(&format!(
                "  export type {brand} = string & {{ readonly __brand: \"{brand}\" }};\n"
            ));
        }
        for line in shared
            .exported
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with("//"))
        {
            out.push_str("  ");
            out.push_str(&line.replace('\t', "  "));
            out.push('\n');
        }
        out.push_str("}\n");
        out.push_str(&format!("\ndeclare module \"{TYPES_MODULE}\" {{\n"));
        for name in &shared.names {
            out.push_str(&format!("  export type {name} = Types.{name};\n"));
        }
        out.push_str("}\n");
        let mut state = String::new();
        if !self.entries.is_empty() {
            state.push_str("\ndeclare const state: ");
            let mut tree = Tree::default();
            for (spec, _) in &self.entries {
                let signature = qualify_names(&spec.signature(), &shared.names);
                tree.insert(spec.path.split('.').collect::<Vec<_>>().as_slice(), signature);
            }
            tree.render(&mut state, 0);
            state.push_str(";\n");
        }
        (out, state)
    }
}

struct Shared {
    exported: String,
    brands: BTreeSet<String>,
    names: Vec<String>,
}

fn qualify_names(text: &str, names: &[String]) -> String {
    let mut out = String::new();
    let mut chars = text.char_indices().peekable();
    let mut quoted = false;
    let mut previous = ' ';
    while let Some((start, c)) = chars.next() {
        if c == '"' {
            quoted = !quoted;
        }
        if !quoted && (c.is_alphabetic() || c == '_') && previous != '.' {
            let mut end = start + c.len_utf8();
            while let Some(&(at, next)) = chars.peek() {
                if next.is_alphanumeric() || next == '_' {
                    end = at + next.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
            let word = &text[start..end];
            if names.iter().any(|name| name == word) {
                out.push_str(&format!("Types.{word}"));
            } else {
                out.push_str(word);
            }
            previous = 'a';
            continue;
        }
        out.push(c);
        previous = c;
    }
    out
}

pub fn to_value<T: Serialize>(item: &T) -> Value {
    Value::from_json(serde_json::to_value(item).unwrap_or(serde_json::Value::Null))
}

#[derive(Default)]
struct Tree {
    leaves: BTreeMap<String, String>,
    branches: BTreeMap<String, Tree>,
}

impl Tree {
    fn insert(&mut self, path: &[&str], signature: String) {
        match path {
            [leaf] => {
                self.leaves.insert(leaf.to_string(), signature);
            }
            [head, rest @ ..] => self
                .branches
                .entry(head.to_string())
                .or_default()
                .insert(rest, signature),
            [] => {}
        }
    }

    fn render(&self, out: &mut String, depth: usize) {
        let pad = "  ".repeat(depth + 1);
        out.push_str("{\n");
        for signature in self.leaves.values() {
            out.push_str(&format!("{pad}{signature};\n"));
        }
        for (name, branch) in &self.branches {
            out.push_str(&format!("{pad}{name}: "));
            branch.render(out, depth + 1);
            out.push_str(";\n");
        }
        out.push_str(&"  ".repeat(depth));
        out.push('}');
    }
}
