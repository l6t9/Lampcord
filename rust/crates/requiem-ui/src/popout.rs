use crate::app;
use crate::classes::popout;
use crate::keymap;
use crate::scroll_animation;
use crate::style::cascade;
use std::cell::{Cell, RefCell};
use std::mem;
use std::rc::Rc;
use std::time::{Duration, Instant};
use winit::dpi::LogicalSize;
use winit::event::MouseButton as EventMouseButton;
use winit::event;

use skia_safe::Color;
use taffy::{AvailableSpace, Size as TaffySize};
use winit::event::{ElementState, Ime, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, ModifiersState};
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
use winit::window::{Window, WindowId};

use crate::event::{MouseButton, NamedKey, UiEvent};
use crate::gpu::SkiaGl;
use crate::node::Node;
use crate::signal;
use crate::style::ComputedStyle;
use crate::style::cascade::StyleIndex;
use crate::widget::WidgetTree;
use crate::widget::primitives::BoxWidget;
use crate::{layout, paint};

pub struct PopoutSpec {
    pub title: String,

    pub width: f32,
    pub height: f32,

    pub build: Rc<dyn Fn() -> Node>,

    pub on_close: Option<Box<dyn FnOnce()>>,
}

thread_local! {
    static NEXT_ID: Cell<u64> = const { Cell::new(1) };
    static OPEN_REQUESTS: RefCell<Vec<(u64, PopoutSpec)>> = const { RefCell::new(Vec::new()) };
    static CLOSE_REQUESTS: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}

pub fn open(spec: PopoutSpec) -> u64 {
    let id = NEXT_ID.with(|n| n.replace(n.get() + 1));
    OPEN_REQUESTS.with(|r| r.borrow_mut().push((id, spec)));
    paint::request_repaint();
    id
}

pub fn close(id: u64) {
    CLOSE_REQUESTS.with(|r| r.borrow_mut().push(id));
    paint::request_repaint();
}

pub(crate) fn take_open_requests() -> Vec<(u64, PopoutSpec)> {
    OPEN_REQUESTS.with(|r| mem::take(&mut *r.borrow_mut()))
}

pub(crate) fn take_close_requests() -> Vec<u64> {
    CLOSE_REQUESTS.with(|r| mem::take(&mut *r.borrow_mut()))
}

const SETTLE: Duration = Duration::from_millis(350);

pub(crate) struct PopoutWindow {
    pub id: u64,
    skia: SkiaGl,
    tree: WidgetTree,
    on_close: Option<Box<dyn FnOnce()>>,
    cursor: (f32, f32),
    modifiers: ModifiersState,

    rules_from: Option<*const StyleIndex>,
    redraw_until: Instant,
    occluded: bool,
}

pub(crate) enum Outcome {
    Continue,
    Close,
}

impl PopoutWindow {
    pub fn new(
        event_loop: &ActiveEventLoop,
        id: u64,
        spec: PopoutSpec,
        main_rules: Option<Rc<StyleIndex>>,
    ) -> Self {
        let attrs = Window::default_attributes()
            .with_title(&spec.title)
            .with_inner_size(LogicalSize::new(
                spec.width as f64,
                spec.height as f64,
            ))
            .with_min_inner_size(LogicalSize::new(240.0, 160.0));
        let skia = SkiaGl::new(event_loop, attrs);
        skia.gl.window.set_ime_allowed(true);

        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        tree.owner = id;
        tree.nodes[tree.root]
            .classes
            .push(popout::POPOUT_ROOT.to_string());
        let previous = signal::set_owner(id);
        let content = (spec.build)();
        let root = tree.root;
        tree.mount(root, content);
        signal::set_owner(previous);

        let mut window = Self {
            id,
            skia,
            tree,
            on_close: spec.on_close,
            cursor: (0.0, 0.0),
            modifiers: ModifiersState::empty(),
            rules_from: None,
            redraw_until: Instant::now() + SETTLE,
            occluded: false,
        };
        window.sync_rules(main_rules);
        window.skia.gl.window.request_redraw();
        window
    }

    pub fn window_id(&self) -> WindowId {
        self.skia.gl.window.id()
    }

    pub fn take_on_close(&mut self) -> Option<Box<dyn FnOnce()>> {
        self.on_close.take()
    }

    pub fn sync_rules(&mut self, main_rules: Option<Rc<StyleIndex>>) {
        let ptr = main_rules.as_ref().map(Rc::as_ptr);
        if ptr != self.rules_from {
            self.rules_from = ptr;
            self.tree.rules = main_rules;
            self.tree.invalidate_styles();
            self.touch();
        }
    }

    pub fn flush(&mut self) -> bool {
        let changed = signal::flush(&mut self.tree);
        if changed {
            self.touch();
        }
        changed
    }

    fn touch(&mut self) {
        self.redraw_until = Instant::now() + SETTLE;
        self.skia.gl.window.request_redraw();
    }

    fn scale(&self) -> f32 {
        self.skia.gl.window.scale_factor() as f32
    }

    pub fn handle_event(&mut self, event: WindowEvent) -> Outcome {
        let previous_owner = signal::set_owner(self.id);
        let outcome = self.handle_event_inner(event);
        signal::set_owner(previous_owner);
        outcome
    }

    fn handle_event_inner(&mut self, event: WindowEvent) -> Outcome {
        match event {
            WindowEvent::CloseRequested => return Outcome::Close,
            WindowEvent::Resized(size) => {
                self.skia.resize(size.width, size.height);
                self.tree.invalidate_styles();
                self.touch();
                if !self.occluded && size.width > 0 && size.height > 0 {
                    self.render();
                }
            }
            WindowEvent::Occluded(occluded) => {
                self.occluded = occluded;
                if !occluded {
                    self.touch();
                }
            }
            WindowEvent::RedrawRequested => {
                if !self.occluded {
                    self.render();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let scale = self.scale();
                self.cursor = (position.x as f32 / scale, position.y as f32 / scale);
                let before = self.tree.hovered_path.clone();
                paint::dispatch_event(
                    &mut self.tree,
                    &UiEvent::MouseMoved {
                        x: self.cursor.0,
                        y: self.cursor.1,
                    },
                );
                if self.tree.hovered_path != before {
                    self.touch();
                }
                self.refresh_cursor();
            }
            WindowEvent::CursorLeft { .. } => {
                paint::dispatch_event(&mut self.tree, &UiEvent::MouseMoved { x: -1.0, y: -1.0 });
                self.touch();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let button = match button {
                    EventMouseButton::Left => MouseButton::Left,
                    EventMouseButton::Right => MouseButton::Right,
                    EventMouseButton::Middle => MouseButton::Middle,
                    EventMouseButton::Back => MouseButton::Other(3),
                    EventMouseButton::Forward => MouseButton::Other(4),
                    EventMouseButton::Other(n) => MouseButton::Other(n),
                };
                let (x, y) = self.cursor;
                match state {
                    ElementState::Pressed => {
                        paint::dispatch_event(
                            &mut self.tree,
                            &UiEvent::MouseDown {
                                x,
                                y,
                                button,
                                shift: self.modifiers.shift_key(),
                            },
                        );
                    }
                    ElementState::Released => {
                        paint::dispatch_event(&mut self.tree, &UiEvent::MouseUp { x, y, button });
                        paint::clear_all_press(&mut self.tree);
                    }
                }
                self.touch();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (delta_x, delta_y, precise) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (
                        x * scroll_animation::WHEEL_NOTCH_PX,
                        y * scroll_animation::WHEEL_NOTCH_PX,
                        x.fract() != 0.0 || y.fract() != 0.0,
                    ),
                    MouseScrollDelta::PixelDelta(pos) => (
                        pos.x as f32 / self.scale(),
                        pos.y as f32 / self.scale(),
                        true,
                    ),
                };
                let (x, y) = self.cursor;
                paint::dispatch_event(
                    &mut self.tree,
                    &UiEvent::Scroll {
                        x,
                        y,
                        delta_x,
                        delta_y,
                        precise,
                    },
                );
                self.touch();
            }
            WindowEvent::ModifiersChanged(mods) => self.modifiers = mods.state(),
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } if key_event.state == ElementState::Pressed => {
                self.dispatch_key(&key_event);
                self.touch();
            }
            WindowEvent::Ime(Ime::Commit(text)) => {
                paint::dispatch_key_event(&mut self.tree, &UiEvent::TextInput { text });
                self.touch();
            }
            WindowEvent::Ime(Ime::Preedit(text, _)) => {
                paint::dispatch_key_event(&mut self.tree, &UiEvent::ImePreedit { text });
                self.touch();
            }
            WindowEvent::ScaleFactorChanged { .. } => self.touch(),
            _ => {}
        }
        Outcome::Continue
    }

    fn refresh_cursor(&mut self) {
        let icon = paint::cursor_at(&self.tree, self.cursor.0, self.cursor.1);
        self.skia
            .gl
            .window
            .set_cursor(app::map_cursor_icon(icon));
    }

    fn dispatch_key(&mut self, key_event: &event::KeyEvent) {
        let shift = self.modifiers.shift_key();
        let unmodified = key_event.key_without_modifiers();

        if self.modifiers.control_key() {
            if let Key::Character(ch) = &unmodified {
                let shortcut = keymap::ctrl_shortcut(ch.as_str(), shift);
                if let Some(key) = shortcut {
                    paint::dispatch_key_event(&mut self.tree, &UiEvent::KeyDown { key, shift });
                }
                return;
            }
        }

        let named = match &key_event.logical_key {
            Key::Named(key) => keymap::named_key(key),
            _ => None,
        };
        if let Some(key) = named {
            if key == NamedKey::Tab {
                paint::cycle_focus(&mut self.tree, shift);
            } else {
                let key =
                    app::modified_nav_key(key, self.modifiers.control_key(), false, false);
                paint::dispatch_key_event(&mut self.tree, &UiEvent::KeyDown { key, shift });
            }
            return;
        }
        if let Some(text) = &key_event.text {
            if !text.is_empty() && !text.chars().any(|c| c.is_control()) {
                paint::dispatch_key_event(
                    &mut self.tree,
                    &UiEvent::TextInput {
                        text: text.to_string(),
                    },
                );
            }
        }
    }

    fn render(&mut self) {
        self.skia.begin();
        let scale = self.scale();
        let size = self.skia.gl.window.inner_size();
        let (width, height) = (size.width as f32 / scale, size.height as f32 / scale);
        if width <= 0.0 || height <= 0.0 {
            return;
        }

        let saved = app::enter_render_context(scale, (width, height));
        let previous_owner = signal::set_owner(self.id);
        signal::flush(&mut self.tree);
        cascade::refresh_theme(&mut self.tree);
        cascade::recompute_if_dirty(&mut self.tree);
        layout::compute_layout(
            &mut self.tree,
            TaffySize {
                width: AvailableSpace::Definite(width),
                height: AvailableSpace::Definite(height),
            },
        );

        let clear = self.tree.nodes[self.tree.root]
            .style
            .background
            .unwrap_or(Color::BLACK);
        let canvas = self.skia.canvas();
        canvas.clear(clear);
        canvas.save();
        canvas.scale((scale, scale));
        paint::paint(&mut self.tree, canvas);
        canvas.restore();
        let animation = paint::take_animation();
        self.skia.present();
        signal::set_owner(previous_owner);
        app::leave_render_context(saved);

        if animation.full || !animation.regions.is_empty() || Instant::now() < self.redraw_until {
            self.skia.gl.window.request_redraw();
        }
    }
}
