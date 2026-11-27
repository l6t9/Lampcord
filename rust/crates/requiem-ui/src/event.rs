#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamedKey {
    Backspace,
    Delete,
    Enter,
    ArrowLeft,
    ArrowRight,

    ArrowUp,
    ArrowDown,
    Home,
    End,
    Escape,

    Tab,

    Space,

    SelectAll,

    Copy,

    Cut,

    Paste,

    WordLeft,

    WordRight,

    DeleteWordBack,

    DeleteWordForward,

    DocStart,

    DocEnd,

    Undo,

    Redo,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiEvent {
    MouseMoved {
        x: f32,
        y: f32,
    },

    MouseDown {
        x: f32,
        y: f32,
        button: MouseButton,
        shift: bool,
    },
    MouseUp {
        x: f32,
        y: f32,
        button: MouseButton,
    },

    Scroll {
        x: f32,
        y: f32,
        delta_x: f32,
        delta_y: f32,
        precise: bool,
    },

    TextInput {
        text: String,
    },

    KeyDown {
        key: NamedKey,
        shift: bool,
    },

    ImePreedit {
        text: String,
    },

    ImeSyncState {
        text: String,
        cursor: usize,
    },
}

impl UiEvent {
    pub fn point(&self) -> (f32, f32) {
        match *self {
            UiEvent::MouseMoved { x, y } => (x, y),
            UiEvent::MouseDown { x, y, .. } => (x, y),
            UiEvent::MouseUp { x, y, .. } => (x, y),
            UiEvent::Scroll { x, y, .. } => (x, y),
            UiEvent::TextInput { .. }
            | UiEvent::KeyDown { .. }
            | UiEvent::ImePreedit { .. }
            | UiEvent::ImeSyncState { .. } => {
                unreachable!("keyboard events are routed by focus, not hit-tested")
            }
        }
    }

    pub fn is_pointer(&self) -> bool {
        matches!(
            self,
            UiEvent::MouseMoved { .. }
                | UiEvent::MouseDown { .. }
                | UiEvent::MouseUp { .. }
                | UiEvent::Scroll { .. }
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventResult {
    Ignored,
    Handled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorIcon {
    Default,

    Pointer,

    Text,

    ZoomIn,

    ZoomOut,

    Grab,

    Grabbing,

    ColResize,

    NotAllowed,
}
