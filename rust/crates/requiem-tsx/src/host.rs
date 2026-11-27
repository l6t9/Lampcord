use std::collections::HashMap;
use std::rc::Rc;

use requiem_ui::node::Node;
use requiem_ui::signal::Signal;

use crate::value::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct HostPropSpec {
    pub name: String,
    pub ts: String,
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HostSpec {
    pub name: String,
    pub props: Vec<HostPropSpec>,
}

#[derive(Clone)]
pub struct HostProps {
    values: Rc<HashMap<String, Signal<Value>>>,
    invoke: Rc<dyn Fn(&Value, &[Value])>,
}

impl HostProps {
    pub(crate) fn new(
        values: HashMap<String, Signal<Value>>,
        invoke: Rc<dyn Fn(&Value, &[Value])>,
    ) -> Self {
        Self {
            values: Rc::new(values),
            invoke,
        }
    }

    pub fn signal(&self, name: &str) -> Signal<Value> {
        self.values
            .get(name)
            .cloned()
            .unwrap_or_else(|| Signal::new(Value::Null))
    }

    pub fn get(&self, name: &str) -> Value {
        self.signal(name).get()
    }

    pub fn peek(&self, name: &str) -> Value {
        self.signal(name).peek()
    }

    pub fn call(&self, name: &str, args: &[Value]) {
        (self.invoke)(&self.signal(name).peek(), args);
    }
}

pub type BuildFn = Rc<dyn Fn(&HostProps) -> Node>;

#[derive(Clone, Default)]
pub struct HostComponents {
    entries: Vec<(HostSpec, BuildFn)>,
}

impl HostComponents {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn component(
        mut self,
        name: &str,
        props: &[(&str, &str, bool)],
        build: impl Fn(&HostProps) -> Node + 'static,
    ) -> Self {
        let spec = HostSpec {
            name: name.to_string(),
            props: props
                .iter()
                .map(|(name, ts, required)| HostPropSpec {
                    name: name.to_string(),
                    ts: ts.to_string(),
                    required: *required,
                })
                .collect(),
        };
        self.entries.push((spec, Rc::new(build)));
        self
    }

    pub fn specs(&self) -> Vec<HostSpec> {
        self.entries.iter().map(|(spec, _)| spec.clone()).collect()
    }

    pub fn runtime(&self) -> HashMap<String, BuildFn> {
        self.entries
            .iter()
            .map(|(spec, build)| (spec.name.clone(), build.clone()))
            .collect()
    }
}
