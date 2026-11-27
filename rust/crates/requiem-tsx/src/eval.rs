use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use requiem_i18n::{Args as I18nArgs, Value as I18nValue};
use requiem_ui::control;
use requiem_ui::signal::Signal;

use crate::ir::{BinOp, Expr, Method};
use crate::state::QueryFn;
use crate::value::{Closure, Func, Value};

pub type Frame = Vec<Signal<Value>>;

thread_local! {
    static QUERIES: RefCell<HashMap<String, QueryFn>> = RefCell::new(HashMap::new());
}

pub fn install_queries(queries: HashMap<String, QueryFn>) {
    QUERIES.with(|installed| *installed.borrow_mut() = queries);
}

fn query(path: &str, args: &[Value]) -> Value {
    let run = QUERIES.with(|installed| installed.borrow().get(path).cloned());
    run.map_or(Value::Null, |run| run(args))
}

pub fn eval(expr: &Expr, frame: &Frame) -> Value {
    match expr {
        Expr::Lit(value) => value.clone(),
        Expr::Closure { params, body } => Value::Func(Func(Rc::new(Closure {
            params: *params,
            body: body.clone(),
            frame: frame.clone(),
        }))),
        Expr::Translate { key, args } => {
            control::track_locale();
            let mut values = I18nArgs::new();
            for (name, expr) in args {
                values = values.with(name, match eval(expr, frame) {
                    Value::Num(n) => I18nValue::Number(n),
                    other => I18nValue::Text(other.to_child_text()),
                });
            }
            Value::str(requiem_i18n::tr(key, &values))
        }
        Expr::Query { path, args } => {
            let args: Vec<Value> = args.iter().map(|arg| eval(arg, frame)).collect();
            query(path, &args)
        }
        Expr::Var(index) => frame[*index].get(),
        Expr::Member(object, name) => eval(object, frame).member(name),
        Expr::Index(object, index) => eval(object, frame).index(&eval(index, frame)),
        Expr::Not(inner) => Value::Bool(!eval(inner, frame).truthy()),
        Expr::Neg(inner) => match eval(inner, frame) {
            Value::Num(n) => Value::Num(-n),
            _ => Value::Num(f64::NAN),
        },
        Expr::Binary(op, left, right) => binary(*op, eval(left, frame), eval(right, frame)),
        Expr::And(left, right) => {
            let left = eval(left, frame);
            if left.truthy() { eval(right, frame) } else { left }
        }
        Expr::Or(left, right) => {
            let left = eval(left, frame);
            if left.truthy() { left } else { eval(right, frame) }
        }
        Expr::Nullish(left, right) => match eval(left, frame) {
            Value::Null => eval(right, frame),
            other => other,
        },
        Expr::Cond(cond, then, otherwise) => {
            if eval(cond, frame).truthy() {
                eval(then, frame)
            } else {
                eval(otherwise, frame)
            }
        }
        Expr::Template(strings, exprs) => {
            let mut out = String::new();
            for (index, string) in strings.iter().enumerate() {
                out.push_str(string);
                if let Some(expr) = exprs.get(index) {
                    out.push_str(&eval(expr, frame).to_js_string());
                }
            }
            Value::str(out)
        }
        Expr::Method(object, method, args) => {
            let object = eval(object, frame);
            let args: Vec<Value> = args.iter().map(|arg| eval(arg, frame)).collect();
            call(&object, *method, &args)
        }
        Expr::List(items) => Value::list(items.iter().map(|item| eval(item, frame))),
        Expr::Obj(fields) => Value::Obj(
            fields
                .iter()
                .map(|(name, expr)| (name.clone(), eval(expr, frame)))
                .collect::<Vec<_>>()
                .into(),
        ),
    }
}

fn binary(op: BinOp, left: Value, right: Value) -> Value {
    match op {
        BinOp::Eq => Value::Bool(left == right),
        BinOp::Ne => Value::Bool(left != right),
        BinOp::Add => match (&left, &right) {
            (Value::Num(a), Value::Num(b)) => Value::Num(a + b),
            (Value::Str(_), _) | (_, Value::Str(_)) => {
                Value::str(format!("{}{}", left.to_js_string(), right.to_js_string()))
            }
            _ => Value::Num(f64::NAN),
        },
        BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => match (&left, &right) {
            (Value::Num(a), Value::Num(b)) => Value::Num(match op {
                BinOp::Sub => a - b,
                BinOp::Mul => a * b,
                BinOp::Div => a / b,
                _ => a % b,
            }),
            _ => Value::Num(f64::NAN),
        },
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            let ordering = match (&left, &right) {
                (Value::Num(a), Value::Num(b)) => a.partial_cmp(b),
                (Value::Str(a), Value::Str(b)) => Some(a.cmp(b)),
                _ => None,
            };
            Value::Bool(ordering.is_some_and(|ordering| match op {
                BinOp::Lt => ordering.is_lt(),
                BinOp::Le => ordering.is_le(),
                BinOp::Gt => ordering.is_gt(),
                _ => ordering.is_ge(),
            }))
        }
    }
}

fn call(object: &Value, method: Method, args: &[Value]) -> Value {
    match (method, object) {
        (Method::ToUpperCase, Value::Str(text)) => Value::str(text.to_uppercase()),
        (Method::ToLowerCase, Value::Str(text)) => Value::str(text.to_lowercase()),
        (Method::Trim, Value::Str(text)) => Value::str(text.trim()),
        (Method::Includes, Value::Str(text)) => match args.first() {
            Some(Value::Str(needle)) => Value::Bool(text.contains(&**needle)),
            _ => Value::Bool(false),
        },
        (Method::Includes, Value::List(items)) => {
            Value::Bool(args.first().is_some_and(|needle| items.contains(needle)))
        }
        (Method::Join, Value::List(items)) => {
            let separator = match args.first() {
                Some(Value::Str(separator)) => separator.to_string(),
                _ => ",".into(),
            };
            Value::str(
                items
                    .iter()
                    .map(|item| match item {
                        Value::Null => String::new(),
                        other => other.to_js_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(&separator),
            )
        }
        _ => Value::Null,
    }
}
