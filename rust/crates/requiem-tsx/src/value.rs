use std::rc::Rc;

use requiem_ui::signal::Signal;

use crate::ir::Stmt;

#[derive(Debug)]
pub struct Closure {
    pub params: usize,
    pub body: Rc<Vec<Stmt>>,
    pub frame: Vec<Signal<Value>>,
}

#[derive(Clone, Debug)]
pub struct Func(pub Rc<Closure>);

impl PartialEq for Func {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Value {
    #[default]
    Null,
    Bool(bool),
    Num(f64),
    Str(Rc<str>),
    List(Rc<Vec<Value>>),
    Obj(Rc<Vec<(Rc<str>, Value)>>),
    Func(Func),
}

impl Value {
    pub fn str(text: impl AsRef<str>) -> Self {
        Value::Str(Rc::from(text.as_ref()))
    }

    pub fn list(items: impl IntoIterator<Item = Value>) -> Self {
        Value::List(Rc::new(items.into_iter().collect()))
    }

    pub fn obj<'a>(fields: impl IntoIterator<Item = (&'a str, Value)>) -> Self {
        Value::Obj(Rc::new(
            fields
                .into_iter()
                .map(|(name, value)| (Rc::from(name), value))
                .collect(),
        ))
    }

    pub fn from_json(json: serde_json::Value) -> Self {
        match json {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => Value::Num(n.as_f64().unwrap_or(f64::NAN)),
            serde_json::Value::String(s) => Value::str(s),
            serde_json::Value::Array(items) => Value::list(items.into_iter().map(Value::from_json)),
            serde_json::Value::Object(fields) => Value::Obj(Rc::new(
                fields
                    .into_iter()
                    .map(|(name, value)| (Rc::from(name.as_str()), Value::from_json(value)))
                    .collect(),
            )),
        }
    }

    pub fn truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Num(n) => *n != 0.0 && !n.is_nan(),
            Value::Str(s) => !s.is_empty(),
            Value::List(_) | Value::Obj(_) | Value::Func(_) => true,
        }
    }

    pub fn to_js_string(&self) -> String {
        match self {
            Value::Null => "null".into(),
            Value::Bool(b) => b.to_string(),
            Value::Num(n) => number_text(*n),
            Value::Str(s) => s.to_string(),
            Value::List(items) => items
                .iter()
                .map(|item| match item {
                    Value::Null => String::new(),
                    other => other.to_js_string(),
                })
                .collect::<Vec<_>>()
                .join(","),
            Value::Obj(_) => "[object Object]".into(),
            Value::Func(_) => "function".into(),
        }
    }

    pub fn to_child_text(&self) -> String {
        match self {
            Value::Null | Value::Bool(_) => String::new(),
            other => other.to_js_string(),
        }
    }

    pub fn as_f32(&self) -> f32 {
        match self {
            Value::Num(n) => *n as f32,
            _ => 0.0,
        }
    }

    pub fn member(&self, name: &str) -> Value {
        match self {
            Value::Obj(fields) => fields
                .iter()
                .find(|(field, _)| &**field == name)
                .map(|(_, value)| value.clone())
                .unwrap_or_default(),
            Value::List(items) if name == "length" => Value::Num(items.len() as f64),
            Value::Str(text) if name == "length" => Value::Num(text.chars().count() as f64),
            _ => Value::Null,
        }
    }

    pub fn index(&self, index: &Value) -> Value {
        match (self, index) {
            (Value::List(items), Value::Num(n)) if *n >= 0.0 && n.fract() == 0.0 => {
                items.get(*n as usize).cloned().unwrap_or_default()
            }
            (Value::Str(text), Value::Num(n)) if *n >= 0.0 && n.fract() == 0.0 => text
                .chars()
                .nth(*n as usize)
                .map(|c| Value::str(c.to_string()))
                .unwrap_or_default(),
            (Value::Obj(_), Value::Str(name)) => self.member(name),
            _ => Value::Null,
        }
    }
}

fn number_text(n: f64) -> String {
    if n.is_nan() {
        "NaN".into()
    } else if n.is_infinite() {
        if n > 0.0 { "Infinity" } else { "-Infinity" }.into()
    } else if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Num(n)
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Num(n as f64)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::str(s)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::str(s)
    }
}
