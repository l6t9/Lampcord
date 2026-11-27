use super::ModalSize;
use std::rc::Rc;

use crate::node::Node;
use crate::widget::anim::ToggleState;

use super::button::{ButtonProps, ButtonVariant};
use super::modal::ModalProps;
use super::text::TextProps;

pub struct ConfirmModalProps {
    pub title: String,

    pub message: Option<String>,

    pub subtitle: Option<String>,
    pub confirm_label: String,

    /// Visual of the confirm button. Does not change what confirm does.
    pub confirm_variant: ButtonVariant,

    /// `None` omits the cancel button. Dismiss still closes.
    pub cancel_label: Option<String>,

    /// Extra body under the message.
    pub content: Option<Node>,

    pub size: ModalSize,

    pub on_confirm: Rc<dyn Fn()>,

    pub on_dismiss: Rc<dyn Fn()>,
    /// Drives the open and close transition. Closing does not drop the node.
    pub open: ToggleState,
}

crate::builder_fields!(ConfirmModalProps {
    title: String,
    opt message: String,
    opt subtitle: String,
    confirm_label: String,
    confirm_variant: ButtonVariant,
    opt cancel_label: String,
    opt content: Node,
    size: ModalSize,
    open: ToggleState,
});

impl ConfirmModalProps {
    pub fn new(
        title: impl Into<String>,
        confirm_label: impl Into<String>,
        on_dismiss: impl Fn() + 'static,
        open: ToggleState,
    ) -> Self {
        Self {
            title: title.into(),
            message: None,
            subtitle: None,
            confirm_label: confirm_label.into(),
            confirm_variant: ButtonVariant::Primary,
            cancel_label: Some(requiem_i18n::t!("ui-cancel")),
            content: None,
            size: ModalSize::Small,
            on_confirm: Rc::new(|| {}),
            on_dismiss: Rc::new(on_dismiss),
            open,
        }
    }

    pub fn critical(mut self) -> Self {
        self.confirm_variant = ButtonVariant::CriticalPrimary;
        self
    }

    pub fn notice(mut self) -> Self {
        self.cancel_label = None;
        self
    }

    pub fn on_confirm(mut self, f: impl Fn() + 'static) -> Self {
        self.on_confirm = Rc::new(f);
        self
    }

    pub fn build(self) -> Node {
        confirm_modal(self)
    }
}

pub fn confirm_modal(props: ConfirmModalProps) -> Node {
    let ConfirmModalProps {
        title,
        message,
        subtitle,
        confirm_label,
        confirm_variant,
        cancel_label,
        content,
        size,
        on_confirm,
        on_dismiss,
        open,
    } = props;

    let confirm = {
        let (on_confirm, on_dismiss) = (on_confirm, on_dismiss.clone());
        ButtonProps::new(confirm_variant, confirm_label, move || {
            on_dismiss();
            on_confirm();
        })
        .build()
    };
    let mut footer = Vec::new();
    if let Some(label) = cancel_label {
        let on_dismiss = on_dismiss.clone();
        footer
            .push(ButtonProps::new(ButtonVariant::Secondary, label, move || on_dismiss()).build());
    }
    footer.push(confirm);

    let mut modal = ModalProps::default()
        .size(size)
        .heading(title)
        .dismissable(true)
        .on_dismiss({
            let on_dismiss = on_dismiss.clone();
            move || on_dismiss()
        });
    modal = modal.open(open).footer(footer);
    if let Some(subtitle) = subtitle {
        modal = modal.subtitle(subtitle);
    }
    match (message, content) {
        (Some(message), Some(content)) => {
            modal = modal.body(
                super::container::div()
                    .child(TextProps::new(message, 16.0).build())
                    .child(content),
            )
        }
        (Some(message), None) => modal = modal.body(TextProps::new(message, 16.0).build()),
        (None, Some(content)) => modal = modal.body(content),
        (None, None) => {}
    }
    modal.build()
}
