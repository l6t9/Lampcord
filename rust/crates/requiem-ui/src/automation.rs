use base64::Engine;
use base64::engine::general_purpose;
use crate::app::FileDrop;
use crate::app;
use crate::keymap::Chord;
use std::collections::HashMap;
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use serde_json::{Value, json};
use slotmap::Key;

use crate::event::{MouseButton, NamedKey, UiEvent};
use crate::paint;
use crate::widget::{AbsoluteRect, WidgetId, WidgetTree};

pub const DEFAULT_PORT: u16 = 9231;

pub fn configured_port() -> u16 {
    env::var("UI_AUTOMATION_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PORT)
}

type PendingRequest = (Value, Sender<Value>);

pub struct AutomationServer {
    rx: Receiver<PendingRequest>,
}

impl AutomationServer {
    pub fn start(port: u16) -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let listener = match TcpListener::bind(("127.0.0.1", port)) {
                Ok(listener) => listener,
                Err(err) => {
                    log::warn!(
                        "ui automation: failed to bind 127.0.0.1:{port}: {err} (automation disabled this run)"
                    );
                    return;
                }
            };
            log::info!("ui automation: listening on 127.0.0.1:{port}");
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                let tx = tx.clone();
                thread::spawn(move || handle_connection(stream, tx));
            }
        });
        Self { rx }
    }

    pub fn drain(
        &self,
        tree: &mut WidgetTree,
        mouse_down: &mut MouseDownHook,
    ) -> (Vec<PendingRequest>, bool) {
        let mut deferred = Vec::new();
        let mut executed = false;
        while let Ok((cmd, reply)) = self.rx.try_recv() {
            if cmd.get("cmd").and_then(Value::as_str) == Some("screenshot") {
                deferred.push((cmd, reply));
                continue;
            }
            let response = execute(tree, mouse_down, &cmd);
            let _ = reply.send(response);
            executed = true;
        }
        (deferred, executed)
    }
}

fn handle_connection(stream: TcpStream, tx: Sender<PendingRequest>) {
    let mut reader = match stream.try_clone() {
        Ok(clone) => BufReader::new(clone),
        Err(_) => return,
    };
    let mut writer = stream;
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => return,
            Ok(_) => {}
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Value>(trimmed) {
            Ok(cmd) => {
                let (reply_tx, reply_rx) = mpsc::channel();
                if tx.send((cmd, reply_tx)).is_err() {
                    return;
                }
                match reply_rx.recv() {
                    Ok(response) => response,
                    Err(_) => return,
                }
            }
            Err(err) => json!({"ok": false, "error": format!("invalid JSON: {err}")}),
        };
        if writeln!(writer, "{response}").is_err() {
            return;
        }
    }
}

fn parse_button(value: &Value) -> MouseButton {
    match value.get("button").and_then(Value::as_str) {
        Some("right") => MouseButton::Right,
        Some("middle") => MouseButton::Middle,
        _ => MouseButton::Left,
    }
}

fn parse_xy(value: &Value) -> Option<(f32, f32)> {
    let x = value.get("x")?.as_f64()? as f32;
    let y = value.get("y")?.as_f64()? as f32;
    Some((x, y))
}

fn parse_named_key(name: &str) -> Option<NamedKey> {
    Some(match name {
        "Backspace" => NamedKey::Backspace,
        "Delete" => NamedKey::Delete,
        "Enter" => NamedKey::Enter,
        "ArrowLeft" => NamedKey::ArrowLeft,
        "ArrowRight" => NamedKey::ArrowRight,
        "ArrowUp" => NamedKey::ArrowUp,
        "ArrowDown" => NamedKey::ArrowDown,
        "Home" => NamedKey::Home,
        "End" => NamedKey::End,
        "Escape" => NamedKey::Escape,
        "Tab" => NamedKey::Tab,
        "Space" => NamedKey::Space,
        "SelectAll" => NamedKey::SelectAll,
        "Copy" => NamedKey::Copy,
        "Cut" => NamedKey::Cut,
        "Paste" => NamedKey::Paste,
        "WordLeft" => NamedKey::WordLeft,
        "WordRight" => NamedKey::WordRight,
        "DeleteWordBack" => NamedKey::DeleteWordBack,
        "DeleteWordForward" => NamedKey::DeleteWordForward,
        "DocStart" => NamedKey::DocStart,
        "DocEnd" => NamedKey::DocEnd,
        "Undo" => NamedKey::Undo,
        "Redo" => NamedKey::Redo,
        _ => return None,
    })
}

fn dump_node(tree: &WidgetTree, id: WidgetId, rects: &HashMap<WidgetId, AbsoluteRect>) -> Value {
    let node = &tree.nodes[id];
    let rect = rects.get(&id).copied().unwrap_or_default();
    json!({
        "id": id.data().as_ffi(),
        "tag": node.tag,
        "dom_id": node.dom_id,
        "classes": node.classes,
        "rect": {"x": rect.x, "y": rect.y, "width": rect.width, "height": rect.height},
        "focused": tree.focused == Some(id),
        "children": node.children.iter().map(|&child| dump_node(tree, child, rects)).collect::<Vec<_>>(),
    })
}

pub type MouseDownHook = Option<Box<dyn FnMut(&mut WidgetTree, Option<WidgetId>)>>;

fn press(
    tree: &mut WidgetTree,
    mouse_down: &mut MouseDownHook,
    x: f32,
    y: f32,
    button: MouseButton,
    shift: bool,
) {
    let (_, handled) = paint::dispatch_event_with_handler(
        tree,
        &UiEvent::MouseDown {
            x,
            y,
            button,
            shift,
        },
    );
    if let Some(hook) = mouse_down {
        hook(tree, handled);
    }
}

fn execute(tree: &mut WidgetTree, mouse_down: &mut MouseDownHook, cmd: &Value) -> Value {
    let name = cmd.get("cmd").and_then(Value::as_str).unwrap_or("");
    match name {
        "mouse_move" => {
            let Some((x, y)) = parse_xy(cmd) else {
                return bad_args();
            };
            paint::dispatch_event(tree, &UiEvent::MouseMoved { x, y });
            json!({"ok": true})
        }
        "mouse_down" => {
            let Some((x, y)) = parse_xy(cmd) else {
                return bad_args();
            };
            let shift = cmd.get("shift").and_then(Value::as_bool).unwrap_or(false);
            press(tree, mouse_down, x, y, parse_button(cmd), shift);
            json!({"ok": true})
        }
        "mouse_up" => {
            let Some((x, y)) = parse_xy(cmd) else {
                return bad_args();
            };
            paint::dispatch_event(
                tree,
                &UiEvent::MouseUp {
                    x,
                    y,
                    button: parse_button(cmd),
                },
            );
            paint::clear_all_press(tree);
            json!({"ok": true})
        }

        "click" => {
            let Some((x, y)) = parse_xy(cmd) else {
                return bad_args();
            };
            let button = parse_button(cmd);
            let shift = cmd.get("shift").and_then(Value::as_bool).unwrap_or(false);
            paint::dispatch_event(tree, &UiEvent::MouseMoved { x, y });
            press(tree, mouse_down, x, y, button, shift);
            paint::dispatch_event(tree, &UiEvent::MouseUp { x, y, button });
            paint::clear_all_press(tree);
            json!({"ok": true})
        }
        "scroll" => {
            let Some((x, y)) = parse_xy(cmd) else {
                return bad_args();
            };
            let delta_x = cmd.get("delta_x").and_then(Value::as_f64).unwrap_or(0.0) as f32;
            let delta_y = cmd.get("delta_y").and_then(Value::as_f64).unwrap_or(0.0) as f32;

            let precise = !cmd.get("notch").and_then(Value::as_bool).unwrap_or(false);
            paint::dispatch_event(
                tree,
                &UiEvent::Scroll {
                    x,
                    y,
                    delta_x,
                    delta_y,
                    precise,
                },
            );
            json!({"ok": true})
        }
        "text" => {
            let Some(text) = cmd.get("text").and_then(Value::as_str) else {
                return bad_args();
            };
            let event = UiEvent::TextInput {
                text: text.to_string(),
            };
            if !app::run_text_fallback_hook(tree, &event) {
                paint::dispatch_key_event(tree, &event);
            }
            json!({"ok": true})
        }

        "modifiers" => {
            app::set_shift(cmd.get("shift").and_then(Value::as_bool).unwrap_or(false));
            json!({"ok": true})
        }
        "key" => {
            let Some(key_name) = cmd.get("key").and_then(Value::as_str) else {
                return bad_args();
            };
            let Some(key) = parse_named_key(key_name) else {
                return json!({"ok": false, "error": format!("unknown key: {key_name}")});
            };
            let shift = cmd.get("shift").and_then(Value::as_bool).unwrap_or(false);
            if key == NamedKey::Tab {
                paint::dispatch_tab(tree, shift);
            } else if !app::run_named_key_hook(tree, key) {
                paint::dispatch_key_event(tree, &UiEvent::KeyDown { key, shift });
            }
            json!({"ok": true})
        }

        "shortcut" => {
            let Some(bind) = cmd.get("chord").and_then(Value::as_str) else {
                return bad_args();
            };
            let Some(chord) = Chord::parse(bind) else {
                return json!({"ok": false, "error": format!("unparseable chord: {bind}")});
            };
            let handled = app::run_shortcut_hook(tree, &chord);
            json!({"ok": true, "handled": handled})
        }

        "file_drop" => {
            let shift = cmd.get("shift").and_then(Value::as_bool).unwrap_or(false);
            match cmd.get("phase").and_then(Value::as_str).unwrap_or("drop") {
                "hover" => app::run_file_drop_hook(tree, FileDrop::Hovered),
                "cancel" => app::run_file_drop_hook(tree, FileDrop::Cancelled),
                _ => {
                    let Some(paths) = cmd.get("paths").and_then(Value::as_array) else {
                        return json!({"ok": false, "error": "missing \"paths\""});
                    };
                    for path in paths.iter().filter_map(Value::as_str) {
                        app::run_file_drop_hook(
                            tree,
                            FileDrop::Dropped {
                                path: path.into(),
                                shift,
                            },
                        );
                    }
                }
            }
            json!({"ok": true})
        }
        "dump_tree" => {
            let rects = paint::all_absolute_rects(tree);
            json!({"ok": true, "tree": dump_node(tree, tree.root, &rects)})
        }
        "" => json!({"ok": false, "error": "missing \"cmd\""}),
        other => json!({"ok": false, "error": format!("unknown cmd: {other}")}),
    }
}

fn bad_args() -> Value {
    json!({"ok": false, "error": "missing/invalid x/y"})
}

pub fn reply_screenshot(png: Option<Vec<u8>>, reply: Sender<Value>) {
    let response = match png {
        Some(bytes) => {
            json!({"ok": true, "png_base64": general_purpose::STANDARD.encode(bytes)})
        }
        None => json!({"ok": false, "error": "screenshot encoding failed"}),
    };
    let _ = reply.send(response);
}
