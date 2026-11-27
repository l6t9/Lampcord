use crate::keybinds;
use crate::ui::classes::shortcuts_dialog as cls;
use requiem_ui::components::ButtonVariant;
use requiem_ui::components::{
    ButtonProps, ModalProps, ModalSize, TextProps, TextVariant, div, scroll_view,
};
use requiem_ui::control::section;
use requiem_ui::node::Node;
use requiem_ui::signal::Effect;
use requiem_ui::widget::{WidgetId, WidgetTree};

use crate::keybinds::{ENTRIES, Group, display_groups};
use crate::ui::dialog::Dialog;
use crate::ui::{TEXT_STRONG, TEXT_SUBTLE};

thread_local! {
    static DIALOG: Dialog = Dialog::new();
}

pub fn is_open() -> bool {
    DIALOG.with(Dialog::is_open)
}

pub fn open() {
    DIALOG.with(Dialog::open);
}

pub fn close() {
    DIALOG.with(Dialog::close);
}

pub fn toggle() {
    if is_open() {
        close();
    } else {
        open();
    }
}

fn keycap(label: String) -> Node {
    div().class(cls::KEYCAP).child(
        TextProps::styled(label, TextVariant::TEXT_XS_SEMIBOLD)
            .color(TEXT_STRONG)
            .build(),
    )
}

fn bind(caps: Vec<String>) -> Node {
    div()
        .class(cls::BIND)
        .children(caps.into_iter().map(keycap))
}

fn entry(entry: &keybinds::Entry) -> Node {
    div()
        .class(cls::ENTRY)
        .child(
            TextProps::styled(
                requiem_i18n::text(entry.description),
                TextVariant::TEXT_MD_NORMAL,
            )
            .color(TEXT_STRONG)
            .build()
            .class(cls::DESCRIPTION),
        )
        .child(
            div()
                .class(cls::BINDS)
                .children(display_groups(entry.binds).into_iter().map(bind)),
        )
}

fn body() -> Node {
    let mut list = scroll_view().class(cls::LIST);
    for group in Group::ALL {
        let entries: Vec<&keybinds::Entry> =
            ENTRIES.iter().filter(|e| e.group == group).collect();
        if entries.is_empty() {
            continue;
        }
        let mut section = div().class(cls::GROUP).child(
            TextProps::styled(group.title(), TextVariant::TEXT_SM_SEMIBOLD)
                .color(TEXT_SUBTLE)
                .build()
                .class(cls::GROUP_TITLE),
        );
        if group == Group::Message {
            section = section.child(
                TextProps::styled(
                    requiem_i18n::t!("keybinds-messages-description"),
                    TextVariant::TEXT_SM_NORMAL,
                )
                .color(TEXT_SUBTLE)
                .build()
                .class(cls::GROUP_NOTE),
            );
        }
        list = list.child(section.children(entries.into_iter().map(entry)));
    }
    div().class(cls::BODY).child(list)
}

fn dialog() -> Node {
    let toggle_state = DIALOG.with(Dialog::toggle);
    let done = ButtonProps::default()
        .variant(ButtonVariant::Primary)
        .label(requiem_i18n::t!("keybinds-dialog-done").to_string())
        .on_click(close)
        .build();
    ModalProps::default()
        .size(ModalSize::Medium)
        .heading(requiem_i18n::t!("keybinds-dialog-title"))
        .dismissable(true)
        .on_dismiss(close)
        .open(toggle_state)
        .body(body())
        .footer(vec![done])
        .panel_class(cls::PANEL.to_string())
        .build()
}

pub fn mount(tree: &mut WidgetTree, parent: WidgetId) -> Effect {
    section(tree, parent, move || {
        if DIALOG.with(Dialog::is_mounted) {
            dialog()
        } else {
            div()
        }
    })
}
