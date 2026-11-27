use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use crate::colors::TEXT_CRITICAL;
use crate::components::{TextFieldProps, TextInputProps, TextProps};
use crate::node::Node;
use crate::signal::{Signal, signal};

pub const REQUIRED_MESSAGE: &str = "This field is required";

#[derive(Clone, Default, PartialEq, Debug)]
pub struct FormErrors {
    pub fields: HashMap<String, String>,
    pub general: Option<String>,
}

impl FormErrors {
    pub fn with_general(message: impl Into<String>) -> Self {
        Self {
            fields: HashMap::new(),
            general: Some(message.into()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty() && self.general.is_none()
    }

    pub fn field(&self, name: &str) -> Option<String> {
        self.fields.get(name).cloned()
    }

    pub fn general_message(&self) -> Option<Node> {
        self.general.as_ref().map(|message| {
            TextProps::new(message.clone(), 14.0)
                .color(TEXT_CRITICAL)
                .build()
        })
    }
}

#[derive(Clone)]
pub struct Form {
    text: Rc<RefCell<HashMap<&'static str, String>>>,

    focus: Rc<Cell<Option<&'static str>>>,
    errors: Signal<FormErrors>,
}

impl Form {
    pub fn new(focus: Option<&'static str>) -> Self {
        Self {
            text: Rc::default(),
            focus: Rc::new(Cell::new(focus)),
            errors: signal(FormErrors::default()),
        }
    }

    pub fn value(&self, key: &str) -> String {
        self.text.borrow().get(key).cloned().unwrap_or_default()
    }

    pub fn trimmed(&self, key: &str) -> String {
        self.value(key).trim().to_string()
    }

    pub fn remove(&self, key: &str) {
        self.text.borrow_mut().remove(key);
    }

    pub fn focus_on(&self, key: &'static str) {
        self.focus.set(Some(key));
    }

    pub fn reset(&self, focus: Option<&'static str>) {
        self.text.borrow_mut().clear();
        self.focus.set(focus);
        self.errors.set(FormErrors::default());
    }

    pub fn errors(&self) -> FormErrors {
        self.errors.get()
    }

    pub fn set_errors(&self, errors: FormErrors) {
        self.errors.set(errors);
    }

    pub fn clear_errors(&self) {
        if !self.errors.peek().is_empty() {
            self.errors.set(FormErrors::default());
        }
    }

    pub fn set_error(&self, field: &str, message: impl Into<String>) {
        let message = message.into();
        self.errors.update(|e| {
            e.fields.insert(field.to_string(), message);
        });
    }

    pub fn set_general_error(&self, message: impl Into<String>) {
        self.errors.set(FormErrors::with_general(message));
    }

    pub fn require(&self, keys: &[&'static str]) -> bool {
        let missing: HashMap<String, String> = keys
            .iter()
            .filter(|k| self.trimmed(k).is_empty())
            .map(|k| (k.to_string(), REQUIRED_MESSAGE.to_string()))
            .collect();
        if missing.is_empty() {
            return true;
        }
        self.errors.set(FormErrors {
            fields: missing,
            general: None,
        });
        false
    }

    pub fn general_message(&self) -> Option<Node> {
        self.errors.get().general_message()
    }

    pub fn input(&self, key: &'static str) -> TextInputProps {
        let focused = self.focus.get() == Some(key);
        let (text, focus) = (self.text.clone(), self.focus.clone());
        TextInputProps::new()
            .text(self.value(key))
            .focused(focused)
            .autofocus(focused)
            .on_change(move |value: &str| {
                text.borrow_mut().insert(key, value.to_string());
            })
            .on_focus_change(move |is_focused: bool| {
                if is_focused {
                    focus.set(Some(key));
                }
            })
    }

    pub fn field(
        &self,
        key: &'static str,
        title: &str,
        on_submit: impl FnMut() + 'static,
    ) -> TextFieldProps {
        let mut on_submit = on_submit;
        TextFieldProps::new(
            title,
            self.input(key).on_submit(move |_: String| on_submit()),
        )
        .error(self.errors.get().field(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_marks_only_empty_fields() {
        let form = Form::new(None);
        form.text.borrow_mut().insert("a", "  ".into());
        form.text.borrow_mut().insert("b", "x".into());
        assert!(!form.require(&["a", "b", "c"]));
        let errors = form.errors();
        assert_eq!(errors.field("a").as_deref(), Some(REQUIRED_MESSAGE));
        assert_eq!(errors.field("c").as_deref(), Some(REQUIRED_MESSAGE));
        assert_eq!(errors.field("b"), None);
        assert!(form.require(&["b"]));
    }

    #[test]
    fn reset_clears_everything() {
        let form = Form::new(Some("a"));
        form.text.borrow_mut().insert("a", "x".into());
        form.set_general_error("nope");
        form.reset(Some("b"));
        assert_eq!(form.value("a"), "");
        assert!(form.errors().is_empty());
    }
}
