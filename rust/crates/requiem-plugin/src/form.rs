use crate::ui::{Node, SwitchProps, TextFieldProps, TextInputProps};

pub fn switch_row(
    label: impl Into<String>,
    description: Option<&str>,
    checked: bool,
    on_change: impl Fn(bool) + 'static,
) -> Node {
    let mut switch = SwitchProps::new(checked, on_change).label(label);
    if let Some(description) = description {
        switch = switch.description(description);
    }
    switch.build()
}

pub fn text_row(
    title: impl Into<String>,
    description: Option<&str>,
    value: &str,
    placeholder: &str,
    on_change: impl Fn(String) + 'static,
) -> Node {
    let mut field = TextFieldProps::new(
        title,
        TextInputProps::new()
            .text(value)
            .placeholder(placeholder)
            .on_change(on_change),
    );
    if let Some(description) = description {
        field = field.description(description);
    }
    field.build()
}

pub fn secret_row(
    title: impl Into<String>,
    description: Option<&str>,
    secret_key: &'static str,
) -> Node {
    let saved = matches!(crate::storage::get_secret(secret_key), Ok(Some(_)));
    let placeholder = if saved {
        "••••••••  (saved)"
    } else {
        ""
    };
    let input = TextInputProps::new()
        .password(true)
        .placeholder(placeholder)
        .on_change(move |value| {
            let result = if value.is_empty() {
                crate::storage::delete_secret(secret_key)
            } else {
                crate::storage::set_secret(secret_key, &value)
            };
            if let Err(err) = result {
                crate::log::error(&format!(
                    "saving secret {secret_key} failed: {}",
                    err.message
                ));
            }
        });
    let mut field = TextFieldProps::new(title, input);
    if let Some(description) = description {
        field = field.description(description);
    }
    field.build()
}
