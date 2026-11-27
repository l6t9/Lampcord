use crate::classes::toast as css;
use crate::icons;
use crate::timer;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;

use super::{TextProps, div, icon};
use crate::control::section;
use crate::node::Node;
use crate::signal::{Effect, Signal, signal};
use crate::widget::{WidgetId, WidgetTree};
use skia_safe::Color;

const DURATION: Duration = Duration::from_millis(3000);

const LEAVE: Duration = Duration::from_millis(300);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Failure,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub kind: ToastKind,
}

#[derive(Clone)]
pub struct ToastState {
    pub current: Signal<Option<Toast>>,

    pub shown: Signal<bool>,
    queue: Rc<RefCell<VecDeque<Toast>>>,
    next_id: Rc<Cell<u64>>,
}

impl Default for ToastState {
    fn default() -> Self {
        Self::new()
    }
}

impl ToastState {
    pub fn new() -> Self {
        Self {
            current: signal(None),
            shown: signal(false),
            queue: Rc::new(RefCell::new(VecDeque::new())),
            next_id: Rc::new(Cell::new(0)),
        }
    }

    pub fn show(&self, message: &str, kind: ToastKind) {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        let toast = Toast {
            id,
            message: message.to_string(),
            kind,
        };
        if self.current.peek().is_some() {
            self.queue.borrow_mut().push_back(toast);
        } else {
            self.start(toast);
        }
    }

    fn start(&self, toast: Toast) {
        self.shown.set(false);
        let id = toast.id;
        self.current.set(Some(toast));

        let shown = self.shown.clone();
        timer::set_timeout(Duration::from_millis(16), move || shown.set(true));
        let state = self.clone();
        timer::set_timeout(DURATION, move || {
            if state.current.peek().is_some_and(|t| t.id == id) {
                state.finish();
            }
        });
    }

    fn finish(&self) {
        self.shown.set(false);
        let state = self.clone();
        timer::set_timeout(LEAVE, move || {
            state.current.set(None);
            let next = state.queue.borrow_mut().pop_front();
            if let Some(next) = next {
                state.start(next);
            }
        });
    }

    pub fn reset(&self) {
        self.queue.borrow_mut().clear();
        self.shown.set(false);
        self.current.set(None);
    }
}

fn build(state: &ToastState, toast: &Toast) -> Node {
    let (kind_class, path, color) = match toast.kind {
        ToastKind::Success => (
            css::TOAST_SUCCESS,
            icons::CHECKMARK_LARGE,
            Color::from_rgb(0xc2, 0xfc, 0xd1),
        ),
        ToastKind::Failure => (
            css::TOAST_FAILURE,
            icons::X_LARGE,
            Color::from_rgb(0xff, 0xde, 0xde),
        ),
    };

    let icon_color = match toast.kind {
        ToastKind::Success => Color::from_rgb(0x24, 0xa6, 0x59),
        ToastKind::Failure => Color::from_rgb(0xff, 0xa1, 0x9c),
    };
    let text = TextProps::default()
        .content(toast.message.clone())
        .size(16.0_f32)
        .line_height(24.0_f32)
        .color(color)
        .build();
    let pill = div()
        .class(css::TOAST)
        .class(kind_class)
        .bind_class(state.shown.clone(), css::TOAST_SHOWN, "")
        .child(icon(path, icon_color).class(css::ICON))
        .child(text);
    div().class(css::TOAST_HOST).child(pill)
}

pub fn mount(tree: &mut WidgetTree, parent: WidgetId, state: &ToastState) -> Effect {
    let state = state.clone();
    section(tree, parent, move || match state.current.get() {
        Some(toast) => build(&state, &toast),
        None => div(),
    })
}

#[cfg(test)]
mod tests {
    use crate::app;
    use crate::font_system::TextFonts;
    use crate::layout;
    use crate::paint;
    use crate::signal as crate_signal;
    use crate::style::cascade;
    use crate::style::parser;
    use skia_safe::surfaces;
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::thread;
    use std::time::Instant;
    use super::*;
    use crate::style::ComputedStyle;
    use crate::widget::primitives::BoxWidget;
    use taffy::{AvailableSpace, Size};

    fn settle(tree: &mut WidgetTree, millis: u64) {
        let end = Instant::now() + Duration::from_millis(millis);
        while Instant::now() < end {
            thread::sleep(Duration::from_millis(20));
            timer::run_due();
            crate_signal::flush(tree);
        }
    }

    #[test]
    fn toasts_queue_one_at_a_time_and_render() {
        TextFonts::install_new(1024 * 1024);
        app::set_window_size(1400.0, 900.0);
        let state = ToastState::new();
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let root = tree.root;
        tree.nodes[root].classes.push("root".to_string());
        let _effect = mount(&mut tree, root, &state);
        let theme = parser::parse_stylesheet(&format!(
            "{}
.root {{ position: relative; width: 1400px; height: 900px; background-color: #242429; }}",
            crate::BUNDLED_CSS
        ));
        let size = Size {
            width: AvailableSpace::Definite(1400.0),
            height: AvailableSpace::Definite(900.0),
        };
        let render = |tree: &mut WidgetTree, name: &str| {
            crate_signal::flush(tree);
            cascade::apply_stylesheet(tree, &theme);
            layout::compute_layout(tree, size);
            if let Ok(dir) = env::var("TOAST_PREVIEW_DIR") {
                let mut surface = surfaces::raster_n32_premul((1400, 300)).unwrap();
                paint::paint(tree, surface.canvas());
                let png = surface
                    .image_snapshot()
                    .encode(None, skia_safe::EncodedImageFormat::PNG, 100)
                    .unwrap();
                fs::write(Path::new(&dir).join(name), png.as_bytes()).unwrap();
            }
        };

        state.show("Image Copied!", ToastKind::Success);
        state.show("Unable to copy link", ToastKind::Failure);
        settle(&mut tree, 500);
        render(&mut tree, "toast_success.png");
        assert_eq!(
            state.current.peek().map(|t| t.message),
            Some("Image Copied!".to_string())
        );
        assert!(state.shown.peek(), "slid in");

        settle(&mut tree, 3300);
        assert_eq!(
            state.current.peek().map(|t| t.message),
            Some("Unable to copy link".to_string()),
            "the queued one takes over after 3s"
        );
        settle(&mut tree, 500);
        render(&mut tree, "toast_failure.png");

        settle(&mut tree, 3600);
        assert!(state.current.peek().is_none(), "and goes away");
    }
}
