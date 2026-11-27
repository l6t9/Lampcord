use crate::value::Value;

pub trait FromValue: Sized {
    fn from_value(value: &Value) -> Option<Self>;
    fn ts() -> String;
}

impl FromValue for bool {
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    fn ts() -> String {
        "boolean".into()
    }
}

impl FromValue for f64 {
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Num(n) => Some(*n),
            _ => None,
        }
    }

    fn ts() -> String {
        "number".into()
    }
}

impl FromValue for i64 {
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Num(n) if n.fract() == 0.0 => Some(*n as i64),
            _ => None,
        }
    }

    fn ts() -> String {
        "number".into()
    }
}

impl FromValue for String {
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Str(text) => Some(text.to_string()),
            _ => None,
        }
    }

    fn ts() -> String {
        "string".into()
    }
}

impl<T: FromValue> FromValue for Option<T> {
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Null => Some(None),
            other => T::from_value(other).map(Some),
        }
    }

    fn ts() -> String {
        format!("{} | null", T::ts())
    }
}

impl<T: FromValue> FromValue for Vec<T> {
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::List(items) => items.iter().map(T::from_value).collect(),
            _ => None,
        }
    }

    fn ts() -> String {
        format!("({})[]", T::ts())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActionSpec {
    pub name: String,
    pub params: Vec<(String, String)>,
}

impl ActionSpec {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            params: Vec::new(),
        }
    }

    pub fn arg(mut self, name: &str, ts: impl Into<String>) -> Self {
        self.params.push((camel_case(name), ts.into()));
        self
    }

    pub fn signature(&self) -> String {
        let params: Vec<String> = self
            .params
            .iter()
            .map(|(name, ts)| format!("{name}: {ts}"))
            .collect();
        format!("{}({}): void", self.name, params.join(", "))
    }
}

pub trait Actions: Sized {
    fn specs() -> Vec<ActionSpec>;
    fn from_call(name: &str, args: &[Value]) -> Option<Self>;
}

fn camel_case(name: &str) -> String {
    let mut out = String::new();
    let mut upper = false;
    for c in name.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

#[macro_export]
macro_rules! ui_actions {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $( $js:ident ( $( $arg:ident : $ty:ty ),* $(,)? ) => $variant:ident ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $( $variant { $( $arg: $ty ),* } ),+
        }

        impl $crate::Actions for $name {
            fn specs() -> Vec<$crate::ActionSpec> {
                vec![
                    $(
                        $crate::ActionSpec::new(stringify!($js))
                            $( .arg(stringify!($arg), <$ty as $crate::FromValue>::ts()) )*
                    ),+
                ]
            }

            #[allow(unused_mut, unused_variables)]
            fn from_call(name: &str, args: &[$crate::Value]) -> Option<Self> {
                match name {
                    $(
                        stringify!($js) => {
                            let mut values = args.iter();
                            Some($name::$variant {
                                $( $arg: <$ty as $crate::FromValue>::from_value(values.next()?)? ),*
                            })
                        }
                    )+
                    _ => None,
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    ui_actions! {
        #[derive(Debug, PartialEq)]
        enum Sample {
            openThread(id: String, shift_key: bool) => OpenThread,
            create() => Create,
            tag(ids: Vec<i64>, parent: Option<String>) => Tag,
        }
    }

    #[test]
    fn calls_become_variants_with_typed_fields() {
        let open = Sample::from_call("openThread", &[Value::str("42"), Value::Bool(true)]);
        assert_eq!(
            open,
            Some(Sample::OpenThread {
                id: "42".into(),
                shift_key: true
            })
        );
        assert_eq!(Sample::from_call("create", &[]), Some(Sample::Create {}));
        let tag = Sample::from_call(
            "tag",
            &[Value::list([Value::Num(1.0), Value::Num(2.0)]), Value::Null],
        );
        assert_eq!(
            tag,
            Some(Sample::Tag {
                ids: vec![1, 2],
                parent: None
            })
        );
    }

    #[test]
    fn a_wrong_shape_or_name_is_not_a_variant() {
        assert_eq!(Sample::from_call("openThread", &[Value::Num(1.0), Value::Bool(true)]), None);
        assert_eq!(Sample::from_call("openThread", &[Value::str("1")]), None);
        assert_eq!(Sample::from_call("nope", &[]), None);
    }

    #[test]
    fn specs_carry_camel_cased_typescript_signatures() {
        let specs = Sample::specs();
        assert_eq!(
            specs[0].signature(),
            "openThread(id: string, shiftKey: boolean): void"
        );
        assert_eq!(specs[1].signature(), "create(): void");
        assert_eq!(
            specs[2].signature(),
            "tag(ids: (number)[], parent: string | null): void"
        );
    }
}
