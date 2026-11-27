use crate::ui::classes::message_edit as cls;
use crate::ui::emoji_picker;
use requiem_ui::components::ButtonVariant;
use requiem_ui::components::TextVariant;
use requiem_ui::paint;
use std::cell::RefCell;
use std::rc::Rc;

use requiem_ui::colors::TEXT_ANCHOR;
use requiem_ui::components::{ButtonProps, TextInputProps, TextProps, div};
use requiem_ui::node::Node;
use requiem_ui::signal::{Signal, signal};

use crate::models::Id;
use crate::models::id::marker::{ChannelMarker, MessageMarker};
use crate::ui::message_list::{ChatColors, chat_input_with};

const INPUT_DOM_ID: &str = "message-edit-input";

thread_local! {
    static EDITING: Signal<Option<(Id<ChannelMarker>, Id<MessageMarker>)>> = signal(None);
    static DRAFT: RefCell<String> = const { RefCell::new(String::new()) };
}

pub fn start(channel: Id<ChannelMarker>, message: Id<MessageMarker>, content: &str) {
    DRAFT.with(|d| *d.borrow_mut() = content.to_string());
    EDITING.with(|e| e.set(Some((channel, message))));
    paint::claim_focus(INPUT_DOM_ID);
}

pub fn cancel() {
    paint::release_focus_claim();
    if is_active() {
        EDITING.with(|e| e.set(None));
        DRAFT.with(|d| d.borrow_mut().clear());
    }
}

pub fn is_active() -> bool {
    EDITING.with(|e| e.peek()).is_some()
}

pub fn target() -> Option<(Id<ChannelMarker>, Id<MessageMarker>)> {
    EDITING.with(Signal::get)
}

pub fn editor(
    colors: &ChatColors,
    custom: Option<emoji_picker::CustomEmojiSource>,
    on_save: Rc<dyn Fn(String)>,
) -> Node {
    let save_from_input = on_save.clone();
    let tick = custom.as_ref().map(|c| c.image_tick.clone());
    let input = chat_input_with(TextInputProps::default(), custom.as_ref())
        .text(DRAFT.with(|d| d.borrow().clone()))
        .font_size(16.0_f32)
        .line_height(22.0_f32)
        .vertical_padding(17.0_f32)
        .insets((0.0_f32, 0.0_f32))
        .color(colors.text_default)
        .placeholder_color(colors.text_placeholder)
        .focused(true)
        .autofocus(true)
        .wrap(true)
        .on_change(|text: &str| DRAFT.with(|d| *d.borrow_mut() = text.to_string()))
        .on_submit(move |content: String| save_from_input(content))
        .build()
        .dom_id(INPUT_DOM_ID)
        .class(cls::EDIT_INPUT);
    let input = match tick {
        Some(tick) => input.bind_image(tick, || None),
        None => input,
    };

    let piece = |content: &str, color| {
        TextProps::styled(content.to_string(), TextVariant::TEXT_XS_NORMAL)
            .line_height(12.0_f32)
            .color(color)
            .build()
    };
    let link = |label: &str, on_click: Box<dyn Fn()>| {
        ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![cls::EDIT_HINT_LINK.to_string()])
            .on_click(on_click)
            .build()
            .child(piece(label, TEXT_ANCHOR))
    };
    let hint = div()
        .class(cls::EDIT_HINT)
        .child(piece(
            &requiem_i18n::t!("message-edit-escape-to"),
            colors.text_default,
        ))
        .child(link(
            &requiem_i18n::t!("message-edit-cancel"),
            Box::new(cancel),
        ))
        .child(piece(
            &requiem_i18n::t!("message-edit-enter-to"),
            colors.text_default,
        ))
        .child(link(
            &requiem_i18n::t!("message-edit-save"),
            Box::new(move || on_save(DRAFT.with(|d| d.borrow().clone()))),
        ));

    div()
        .class(cls::EDIT)
        .child(div().class(cls::EDIT_BOX).child(input))
        .child(hint)
}
