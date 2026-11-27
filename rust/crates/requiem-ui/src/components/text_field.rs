use crate::node::Node;

use super::form_item::{FormItemProps, form_item};
use super::text_input::{TextInputProps, text_input};

pub struct TextFieldProps {
    pub title: String,
    pub description: Option<String>,
    /// Draws the required mark. Does not validate.
    pub required: bool,

    /// Replaces the helper line and marks the input invalid.
    pub error: Option<String>,
    pub input: TextInputProps,
}

crate::builder_fields!(TextFieldProps {
    title: String,
    opt description: String,
    required: bool,
    error: Option<String>,
    input: TextInputProps,
});

impl TextFieldProps {
    pub fn new(title: impl Into<String>, input: TextInputProps) -> Self {
        Self {
            title: title.into(),
            description: None,
            required: false,
            error: None,
            input,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.input = self.input.placeholder(placeholder);
        self
    }

    pub fn password(mut self, on: bool) -> Self {
        self.input = self.input.password(on);
        self
    }

    pub fn build(self) -> Node {
        text_field(self)
    }
}

pub fn text_field(props: TextFieldProps) -> Node {
    let TextFieldProps {
        title,
        description,
        required,
        error,
        mut input,
    } = props;
    if error.is_some() {
        input.error = Some(String::new());
    }
    form_item(FormItemProps {
        title: Some(title),
        description,
        required,
        error,
        children: text_input(input),
        ..Default::default()
    })
}
