use crate::classes::copy_input as cls;
use crate::clipboard;
use crate::font_system;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use crate::colors::{INPUT_PLACEHOLDER, TEXT_DEFAULT};
use crate::node::Node;
use crate::signal::{Signal, signal};
use crate::timer::{TimeoutHandle, clear_timeout, set_timeout};
use crate::typography::TextVariant;
use crate::widget::primitives::TextInputWidget;

use super::button::{ButtonProps, ButtonVariant};
use super::container::div;
use super::text::TextProps;

const COPIED_FOR: Duration = Duration::from_millis(1000);

pub struct CopyInputProps {
    pub value: String,

    /// Called with the value after the clipboard write succeeds.
    pub on_copy: Option<Rc<dyn Fn(&str)>>,
    pub disabled: bool,
}

impl CopyInputProps {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            on_copy: None,
            disabled: false,
        }
    }

    pub fn on_copy(mut self, f: impl Fn(&str) + 'static) -> Self {
        self.on_copy = Some(Rc::new(f));
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn build(self) -> Node {
        copy_input(self)
    }
}

pub fn copy_input(props: CopyInputProps) -> Node {
    let copied: Signal<bool> = signal(false);
    let timer: Rc<Cell<Option<TimeoutHandle>>> = Rc::new(Cell::new(None));
    let CopyInputProps {
        value,
        on_copy,
        disabled,
    } = props;
    Node::reactive(move || {
        let success = copied.get();
        let fonts = font_system::resolve(None);
        let mut field = TextInputWidget::new(
            value.clone(),
            String::new(),
            16.0,
            TEXT_DEFAULT,
            INPUT_PLACEHOLDER,
            fonts.font_system.clone(),
            fonts.glyph_cache.clone(),
            false,
        )
        .with_read_only()
        .with_insets(10.0, 0.0);
        if disabled {
            field = field.with_disabled();
        }
        let label = if success {
            requiem_i18n::t!("ui-copied")
        } else {
            requiem_i18n::t!("ui-copy")
        };
        let (copy_value, copy_hook, copied, timer) = (
            value.clone(),
            on_copy.clone(),
            copied.clone(),
            timer.clone(),
        );
        let button = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![
                cls::BUTTON.to_string(),
                if success {
                    cls::BUTTON_GREEN
                } else {
                    cls::BUTTON_BRAND
                }
                .to_string(),
            ])
            .disabled(disabled)
            .on_click(move || {
                if !clipboard::set_text(copy_value.clone()) {
                    return;
                }
                copied.set(true);
                if let Some(previous) = timer.take() {
                    clear_timeout(previous);
                }
                let reset = copied.clone();
                timer.set(Some(set_timeout(COPIED_FOR, move || reset.set(false))));
                if let Some(hook) = &copy_hook {
                    hook(&copy_value);
                }
            })
            .build()
            .child(
                TextProps::styled(label, TextVariant::TEXT_SM_MEDIUM)
                    .color(skia_safe::Color::WHITE)
                    .line_height(16.0_f32)
                    .line_clamp(1_u32)
                    .build(),
            );
        div()
            .class(cls::FIELD)
            .class(if success {
                cls::FIELD_SUCCESS
            } else {
                cls::FIELD_DEFAULT
            })
            .focus_ring_host()
            .children([Node::new("div", field).class(cls::INPUT), button])
    })
}
