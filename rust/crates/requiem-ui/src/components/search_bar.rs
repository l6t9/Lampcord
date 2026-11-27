use crate::classes::search_bar as cls;
use crate::colors::{INPUT_PLACEHOLDER, TEXT_DEFAULT};
use crate::font_system;
use crate::icons;

use super::{div, icon};
use crate::node::Node;
use crate::widget::primitives::TextInputWidget;

pub fn search_bar(placeholder: &str, on_change: impl FnMut(&str) + 'static) -> Node {
    SearchBarProps::new(placeholder, on_change).build()
}

pub struct SearchBarProps {
    placeholder: String,
    text: String,
    on_change: Box<dyn FnMut(&str)>,
    on_submit: Option<Box<dyn FnMut(String)>>,
    autofocus: bool,
    clearable: bool,
}

impl SearchBarProps {
    pub fn new(placeholder: &str, on_change: impl FnMut(&str) + 'static) -> Self {
        Self {
            placeholder: placeholder.to_string(),
            text: String::new(),
            on_change: Box::new(on_change),
            on_submit: None,
            autofocus: false,
            clearable: false,
        }
    }

    /// The query already typed, for a bar rebuilt while open.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Enter.
    pub fn on_submit(mut self, on_submit: impl FnMut(String) + 'static) -> Self {
        self.on_submit = Some(Box::new(on_submit));
        self
    }

    pub fn autofocus(mut self, autofocus: bool) -> Self {
        self.autofocus = autofocus;
        self
    }

    /// An X that empties the query while it has text.
    pub fn clearable(mut self, clearable: bool) -> Self {
        self.clearable = clearable;
        self
    }

    pub fn build(self) -> Node {
        let fonts = &font_system::resolve(None);
        let mut widget = TextInputWidget::new(
            self.text,
            self.placeholder,
            14.0,
            TEXT_DEFAULT,
            INPUT_PLACEHOLDER,
            fonts.font_system.clone(),
            fonts.glyph_cache.clone(),
            false,
        )
        .with_on_change(self.on_change)
        .with_insets(0.0, 12.0);
        if let Some(on_submit) = self.on_submit {
            widget = widget.with_on_submit(on_submit);
        }
        if self.clearable {
            widget = widget.with_clear_button();
        }
        let mut input = Node::new("div", widget).class(cls::INPUT);
        if self.autofocus {
            input = input.wants_focus();
        }
        div()
            .class(cls::BAR)
            .focus_ring_host()
            .child(
                div().class(cls::GLYPH).child(
                    icon(icons::MAGNIFYING_GLASS, TEXT_DEFAULT).class(cls::GLYPH_ICON),
                ),
            )
            .child(input)
    }
}
