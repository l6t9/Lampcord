#[doc(hidden)]
pub mod bindings {
    wit_bindgen::generate!({
        path: "../requiem-plugin-api/wit",
        world: "plugin",
        pub_export_macro: true,
        default_bindings_module: "requiem_plugin::bindings",
        export_macro_name: "__export_world",
    });
}

pub mod form;
pub mod ui;

use bindings::requiem::plugin as api;

pub use api::drafts::{
    Event, IncomingMessage, Notification, OutgoingEdit, OutgoingMessage, OutgoingPresence,
};
pub use api::model::{Attachment, Channel, Guild, Member, Message, Role, User};

pub use api::registry::{MenuState, Suggestion};
pub use api::types::{Error, Json, Snowflake, Timestamp, Verdict};
pub use api::{actions, client, navigation, registry, slots, timers};

pub trait Plugin {
    fn init(previous_version: Option<String>) -> Result<(), Error> {
        let _ = previous_version;
        Ok(())
    }

    fn shutdown() {}

    fn before_send(message: &OutgoingMessage) -> Verdict {
        let _ = message;
        Verdict::Pass
    }

    fn before_edit(edit: &OutgoingEdit) -> Verdict {
        let _ = edit;
        Verdict::Pass
    }

    fn transform(batch: &[IncomingMessage]) -> Vec<Verdict> {
        let _ = batch;
        Vec::new()
    }

    fn before_presence_update(presence: &OutgoingPresence) -> Verdict {
        let _ = presence;
        Verdict::Pass
    }

    fn before_notify(notification: &Notification) -> Verdict {
        let _ = notification;
        Verdict::Pass
    }

    fn settings_page() -> ui::Node {
        ui::div()
    }

    fn view(view_id: &str, context: &str) -> ui::Node {
        let _ = (view_id, context);
        ui::div()
    }

    fn on_event(event: Event) {
        let _ = event;
    }

    fn on_timer(id: &str) {
        let _ = id;
    }

    fn on_websocket(id: u32, kind: &str, data: Vec<u8>) {
        let _ = (id, kind, data);
    }

    fn on_command(name: &str, options: &str, context: &str) -> Result<(), String> {
        let _ = (name, options, context);
        Ok(())
    }

    fn on_menu_item(menu: &str, item_id: &str, target: &str, checked: Option<bool>) {
        let _ = (menu, item_id, target, checked);
    }

    fn on_slot_action(slot: &str, item_id: &str, context: &str) {
        let _ = (slot, item_id, context);
    }

    fn on_shortcut(id: &str) {
        let _ = id;
    }

    fn on_message_click(message: Message, button: &str, modifiers: &str) -> bool {
        let _ = (message, button, modifiers);
        false
    }

    fn context_menu(menu: &str, target: &str) -> Vec<MenuState> {
        let _ = (menu, target);
        Vec::new()
    }

    fn command_autocomplete(
        command: &str,
        option_name: &str,
        partial: &str,
        context: &str,
    ) -> Vec<Suggestion> {
        let _ = (command, option_name, partial, context);
        Vec::new()
    }

    fn autocomplete(trigger: &str, query: &str, context: &str) -> Vec<Suggestion> {
        let _ = (trigger, query, context);
        Vec::new()
    }

    fn search(provider: &str, query: &str) -> Vec<Suggestion> {
        let _ = (provider, query);
        Vec::new()
    }

    fn render_markdown(rule_id: &str, captures: Vec<String>, context: &str) -> ui::Node {
        let _ = (rule_id, captures, context);
        ui::div()
    }

    fn process_audio(stream: &str, sample_rate: u32, channels: u8, frame: Vec<f32>) -> Vec<f32> {
        let _ = (stream, sample_rate, channels);
        frame
    }
}

pub fn error(code: &str, message: impl Into<String>) -> Error {
    Error {
        code: code.to_string(),
        message: message.into(),
    }
}

pub mod log {
    use super::api::log::{Level, log};

    pub fn trace(message: &str) {
        log(Level::Trace, message);
    }
    pub fn debug(message: &str) {
        log(Level::Debug, message);
    }
    pub fn info(message: &str) {
        log(Level::Info, message);
    }
    pub fn warn(message: &str) {
        log(Level::Warn, message);
    }
    pub fn error(message: &str) {
        log(Level::Error, message);
    }
}

pub mod storage {
    pub use super::api::storage::{delete, delete_secret, get, get_secret, keys, set, set_secret};

    pub fn get_string(key: &str) -> Option<String> {
        get(key).and_then(|bytes| String::from_utf8(bytes).ok())
    }
}

pub mod settings {
    use serde::Serialize;
    use serde::de::DeserializeOwned;

    use super::{Error, storage};

    pub const KEY: &str = "settings";

    pub fn load<T: DeserializeOwned + Default>() -> T {
        storage::get(KEY)
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save<T: Serialize>(settings: &T) -> Result<(), Error> {
        let json =
            serde_json::to_vec(settings).map_err(|err| super::error("invalid", err.to_string()))?;
        storage::set(KEY, &json)
    }

    pub fn update<T: Serialize + DeserializeOwned + Default>(change: impl FnOnce(&mut T)) {
        let mut settings = load::<T>();
        change(&mut settings);
        if let Err(err) = save(&settings) {
            super::log::error(&format!("saving settings failed: {}", err.message));
        }
    }
}

pub mod overlay {
    pub use super::api::overlay::{clear, set_annotation, set_display_content};
}

#[macro_export]
macro_rules! classes {
    () => {
        #[allow(dead_code)]
        pub mod classes {
            include!(concat!(env!("OUT_DIR"), "/css_classes.rs"));
        }
    };
}

pub mod prelude {
    pub use super::ui::{
        Align, ButtonProps, ButtonSize, ButtonVariant, FormItemProps, HeadingLevel, HeadingProps,
        Justify, Node, Space, StackProps, SwitchProps, TextFieldProps, TextInputProps, TextProps,
        TextVariant, div, span,
    };
    pub use super::{
        Error, IncomingMessage, Message, OutgoingEdit, OutgoingMessage, Plugin, Snowflake, User,
        Verdict, error, export_plugin, form, log, overlay, settings, storage,
    };
}

#[doc(hidden)]
pub fn __render(view_id: &str, page: ui::Node) -> bindings::requiem::plugin::ui::ViewBuilder {
    ui::record(view_id, page)
}

#[doc(hidden)]
pub fn __on_ui_action(action_id: String, value: Option<String>) {
    ui::dispatch(&action_id, value);
}

#[macro_export]
macro_rules! export_plugin {
    ($plugin:ty) => {
        const _: () = {
            struct Exports;

            impl exports::lifecycle::Guest for Exports {
                fn init(previous_version: Option<String>) -> Result<(), $crate::Error> {
                    <$plugin as $crate::Plugin>::init(previous_version)
                }
                fn shutdown() {
                    <$plugin as $crate::Plugin>::shutdown()
                }
            }

            impl exports::outgoing::Guest for Exports {
                fn before_send(message: &$crate::OutgoingMessage) -> $crate::Verdict {
                    <$plugin as $crate::Plugin>::before_send(message)
                }
                fn before_edit(edit: &$crate::OutgoingEdit) -> $crate::Verdict {
                    <$plugin as $crate::Plugin>::before_edit(edit)
                }
                fn before_presence_update(presence: &$crate::OutgoingPresence) -> $crate::Verdict {
                    <$plugin as $crate::Plugin>::before_presence_update(presence)
                }
            }

            impl exports::incoming::Guest for Exports {
                fn transform(batch: Vec<$crate::IncomingMessage>) -> Vec<$crate::Verdict> {
                    <$plugin as $crate::Plugin>::transform(&batch)
                }
                fn before_notify(notification: &$crate::Notification) -> $crate::Verdict {
                    <$plugin as $crate::Plugin>::before_notify(notification)
                }
            }

            impl exports::events::Guest for Exports {
                fn on_event(event: $crate::Event) {
                    <$plugin as $crate::Plugin>::on_event(event)
                }
                fn on_timer(id: String) {
                    <$plugin as $crate::Plugin>::on_timer(&id)
                }
                fn on_websocket(id: u32, kind: String, data: Vec<u8>) {
                    <$plugin as $crate::Plugin>::on_websocket(id, &kind, data)
                }
            }

            impl exports::interactions::Guest for Exports {
                fn on_ui_action(_view_id: String, action_id: String, value: Option<$crate::Json>) {
                    $crate::__on_ui_action(action_id, value)
                }
                fn on_command(
                    name: String,
                    options: $crate::Json,
                    context: $crate::Json,
                ) -> Result<(), String> {
                    <$plugin as $crate::Plugin>::on_command(&name, &options, &context)
                }
                fn on_menu_item(
                    menu: String,
                    item_id: String,
                    target: $crate::Json,
                    checked: Option<bool>,
                ) {
                    <$plugin as $crate::Plugin>::on_menu_item(&menu, &item_id, &target, checked)
                }
                fn on_slot_action(slot: String, item_id: String, context: $crate::Json) {
                    <$plugin as $crate::Plugin>::on_slot_action(&slot, &item_id, &context)
                }
                fn on_shortcut(id: String) {
                    <$plugin as $crate::Plugin>::on_shortcut(&id)
                }
                fn on_message_click(
                    message: $crate::Message,
                    button: String,
                    modifiers: $crate::Json,
                ) -> bool {
                    <$plugin as $crate::Plugin>::on_message_click(message, &button, &modifiers)
                }
            }

            impl exports::views::Guest for Exports {
                fn render(
                    view_id: String,
                    context: $crate::Json,
                ) -> $crate::bindings::requiem::plugin::ui::ViewBuilder {
                    let page = if view_id == "settings" {
                        <$plugin as $crate::Plugin>::settings_page()
                    } else {
                        <$plugin as $crate::Plugin>::view(&view_id, &context)
                    };
                    $crate::__render(&view_id, page)
                }
            }

            impl exports::providers::Guest for Exports {
                fn context_menu(menu: String, target: $crate::Json) -> Vec<$crate::MenuState> {
                    <$plugin as $crate::Plugin>::context_menu(&menu, &target)
                }
                fn command_autocomplete(
                    command: String,
                    option_name: String,
                    partial: String,
                    context: $crate::Json,
                ) -> Vec<$crate::Suggestion> {
                    <$plugin as $crate::Plugin>::command_autocomplete(
                        &command,
                        &option_name,
                        &partial,
                        &context,
                    )
                }
                fn autocomplete(
                    trigger: String,
                    query: String,
                    context: $crate::Json,
                ) -> Vec<$crate::Suggestion> {
                    <$plugin as $crate::Plugin>::autocomplete(&trigger, &query, &context)
                }
                fn search(provider: String, query: String) -> Vec<$crate::Suggestion> {
                    <$plugin as $crate::Plugin>::search(&provider, &query)
                }
                fn render_markdown(
                    rule_id: String,
                    captures: Vec<String>,
                    context: $crate::Json,
                ) -> $crate::bindings::requiem::plugin::ui::ViewBuilder {
                    $crate::__render(
                        &rule_id,
                        <$plugin as $crate::Plugin>::render_markdown(&rule_id, captures, &context),
                    )
                }
                fn process_audio(
                    stream: String,
                    sample_rate: u32,
                    channels: u8,
                    frame: Vec<f32>,
                ) -> Vec<f32> {
                    <$plugin as $crate::Plugin>::process_audio(
                        &stream,
                        sample_rate,
                        channels,
                        frame,
                    )
                }
            }

    $crate::bindings::__export_world!(Exports with_types_in $crate::bindings);
        };
    };
}
