#[cfg(debug_assertions)]
use crate::automation::AutomationServer;
#[cfg(debug_assertions)]
use crate::automation;
use crate::inspect;
use crate::keymap;
use crate::perf::Phase;
use crate::perf;
use crate::popout::Outcome;
use crate::popout::PopoutWindow;
use crate::popout;
use crate::scroll_animation;
use crate::style::cascade;
use crate::style::values;
use crate::theme;
use crate::timer;
use crate::waker::FrameWaker;
use crate::widget::primitives;
#[cfg(windows)]
use raw_window_handle::HasWindowHandle;
#[cfg(windows)]
use raw_window_handle::RawWindowHandle;
use skia_safe::font_style::Weight;
use skia_safe::graphics;
use skia_safe::region::RegionOp;
use std::cell::Cell;
use std::cell::RefCell;
use std::env;
use std::mem;
use std::path;
use std::process;
use std::sync::OnceLock;
use std::sync::atomic::AtomicIsize;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use winit::error;
use winit::event::MouseButton as EventMouseButton;
use winit::event::TouchPhase;
use winit::event as winit_event;
#[cfg(windows)]
use winit::platform::windows::WindowAttributesExtWindows;
use winit::window::CursorIcon;
use winit::window::CursorIcon as C;
use winit::window::Fullscreen;
use winit::window::ResizeDirection as D;
use winit::window::UserAttentionType;
use winit::window;

use skia_safe::{Color, Font};
use taffy::{AvailableSpace, Size as TaffySize};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, Ime, MouseScrollDelta, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{Key, ModifiersState};
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
use winit::window::{Window, WindowId};

use crate::event::{self, MouseButton, NamedKey, UiEvent};
use crate::gpu::SkiaGl;
use crate::keymap::Chord;
use crate::signal;
use crate::widget::{WidgetId, WidgetTree};
use crate::{layout, paint};

const TARGET_FRAME_SECONDS: f32 = 1.0 / 60.0;

const RESIZE_BORDER_PX: f32 = 6.0;

thread_local! {

    static CURRENT_SCALE_FACTOR: Cell<f32> = const { Cell::new(1.0) };

    static CURRENT_TOP_INSET: Cell<f32> = const { Cell::new(0.0) };

    static CURRENT_WINDOW_SIZE: Cell<(f32, f32)> = const { Cell::new((0.0, 0.0)) };

    static FULLSCREEN_REQUEST: Cell<Option<bool>> = const { Cell::new(None) };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowAction {
    Drag,
    Minimize,
    ToggleMaximize,
    Close,

    Focus,

    Attention,
}

thread_local! {
    static WINDOW_ACTION: Cell<Option<WindowAction>> = const { Cell::new(None) };
}

static NATIVE_WINDOW: AtomicIsize = AtomicIsize::new(0);

pub fn native_window_handle() -> Option<isize> {
    Some(NATIVE_WINDOW.load(Ordering::Acquire)).filter(|h| *h != 0)
}

pub fn request_window_action(action: WindowAction) {
    WINDOW_ACTION.with(|c| c.set(Some(action)));
}

type NamedKeyHook = Box<dyn FnMut(&mut WidgetTree, NamedKey) -> bool>;
type TextFallbackHook = Box<dyn FnMut(&mut WidgetTree, &UiEvent) -> bool>;
type ShortcutHook = Box<dyn FnMut(&mut WidgetTree, &Chord) -> bool>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileDrop {
    Hovered,

    Cancelled,

    Dropped {
        path: path::PathBuf,
        shift: bool,
    },
}

type FileDropHook = Box<dyn FnMut(&mut WidgetTree, FileDrop)>;

thread_local! {
    static SHIFT_HELD: Cell<bool> = const { Cell::new(false) };

    static SHIFT_DOWN: signal::Signal<bool> = signal::signal(false);
}

pub(crate) fn set_shift(shift: bool) {
    SHIFT_HELD.with(|c| c.set(shift));
    SHIFT_DOWN.with(|s| {
        if s.peek() != shift {
            s.set(shift);
        }
    });
}

pub fn shift_down() -> bool {
    SHIFT_DOWN.with(signal::Signal::get)
}

pub fn shift_held() -> bool {
    SHIFT_HELD.with(|c| c.get())
}

thread_local! {
    static FILE_DROP_HOOK: RefCell<Option<FileDropHook>> = const { RefCell::new(None) };
}

thread_local! {
    static TEXT_FALLBACK_HOOK: RefCell<Option<TextFallbackHook>> = const { RefCell::new(None) };
}

thread_local! {
    static SHORTCUT_HOOK: RefCell<Option<ShortcutHook>> = const { RefCell::new(None) };
}

pub(crate) fn run_shortcut_hook(tree: &mut WidgetTree, chord: &Chord) -> bool {
    SHORTCUT_HOOK.with(|h| {
        h.borrow_mut()
            .as_mut()
            .is_some_and(|hook| hook(tree, chord))
    })
}

thread_local! {
    static NAMED_KEY_HOOK: RefCell<Option<NamedKeyHook>> = const { RefCell::new(None) };
}

pub(crate) fn run_named_key_hook(tree: &mut WidgetTree, key: NamedKey) -> bool {
    if key == NamedKey::Escape && paint::dismiss_top_popover(tree) {
        return true;
    }
    NAMED_KEY_HOOK.with(|h| h.borrow_mut().as_mut().is_some_and(|hook| hook(tree, key)))
}

#[cfg(debug_assertions)]
pub(crate) fn run_text_fallback_hook(tree: &mut WidgetTree, event: &UiEvent) -> bool {
    let focused_takes_text = tree.focused.is_some_and(|id| {
        tree.nodes
            .get(id)
            .is_some_and(|n| n.widget.wants_text_input())
    });
    !focused_takes_text
        && TEXT_FALLBACK_HOOK.with(|h| {
            h.borrow_mut()
                .as_mut()
                .is_some_and(|hook| hook(tree, event))
        })
}

pub(crate) fn run_file_drop_hook(tree: &mut WidgetTree, drop: FileDrop) {
    FILE_DROP_HOOK.with(|h| {
        if let Some(hook) = h.borrow_mut().as_mut() {
            hook(tree, drop);
        }
    });
}

pub fn request_fullscreen(on: bool) {
    FULLSCREEN_REQUEST.with(|c| c.set(Some(on)));
}

pub fn current_scale_factor() -> f32 {
    CURRENT_SCALE_FACTOR.with(|c| c.get())
}

pub fn current_top_inset() -> f32 {
    CURRENT_TOP_INSET.with(|c| c.get())
}

#[doc(hidden)]
pub fn set_window_size(width: f32, height: f32) {
    CURRENT_WINDOW_SIZE.with(|c| c.set((width, height)));
}

thread_local! {

    static RESIZING_UNTIL: Cell<Option<Instant>> = const { Cell::new(None) };

    static RESIZE_REFINE: RefCell<Option<timer::TimeoutHandle>> = const { RefCell::new(None) };
}

const LIVE_RESIZE_WINDOW: Duration = Duration::from_millis(250);

pub fn live_resizing() -> bool {
    RESIZING_UNTIL.with(|c| c.get().is_some_and(|until| Instant::now() < until))
}

fn note_resized() {
    RESIZING_UNTIL.with(|c| c.set(Some(Instant::now() + LIVE_RESIZE_WINDOW)));
    RESIZE_REFINE.with(|handle| {
        if let Some(previous) = handle.borrow_mut().take() {
            timer::clear_timeout(previous);
        }
        *handle.borrow_mut() = Some(timer::set_timeout(
            LIVE_RESIZE_WINDOW + Duration::from_millis(20),
            paint::request_repaint,
        ));
    });
}

pub fn current_window_size() -> (f32, f32) {
    CURRENT_WINDOW_SIZE.with(|c| c.get())
}

const ZOOM_LEVELS: [f32; 11] = [
    0.50, 0.67, 0.75, 0.80, 0.90, 1.00, 1.10, 1.25, 1.50, 1.75, 2.00,
];

const DEFAULT_ZOOM: f32 = 1.00;

const FLING_SAMPLE_WINDOW: Duration = Duration::from_millis(100);

const MIN_FLING_VELOCITY: f32 = 80.0;

const FLING_DECAY_PER_SECOND: f32 = 0.02;

const DEFAULT_ANIMATION_SETTLE: Duration = Duration::from_millis(350);

const LIVE_RESIZE_SETTLE: Duration = Duration::from_millis(120);

pub struct App<T: 'static> {
    event_loop: EventLoop<T>,
    handler: AppHandler<T>,
}

impl<T: 'static> App<T> {
    pub fn new(
        title: &str,
        initial_tree: WidgetTree,
        on_user_event: impl FnMut(T, &mut WidgetTree) + 'static,
    ) -> Self {
        let event_loop = EventLoop::<T>::with_user_event()
            .build()
            .expect("failed to build winit event loop");
        Self::from_event_loop(title, initial_tree, on_user_event, event_loop)
    }

    fn from_event_loop(
        title: &str,
        initial_tree: WidgetTree,
        on_user_event: impl FnMut(T, &mut WidgetTree) + 'static,
        event_loop: EventLoop<T>,
    ) -> Self {
        let handler = AppHandler {
            title: title.to_string(),
            skia: None,
            tree: initial_tree,
            previous_frame_start: Instant::now(),
            cursor_pos: (0.0, 0.0),
            pointer_in_window: false,
            zoom: DEFAULT_ZOOM,
            active_touch: None,
            touch_velocity_samples: Vec::new(),
            fling_velocity_y: 0.0,
            fling_last_tick: None,
            modifiers: ModifiersState::empty(),
            current_cursor: event::CursorIcon::Default,
            custom_chrome: false,
            fixed_window_size: None,
            frameless: false,
            resize_edge: None,
            closing: false,
            defer_rebuilds: false,
            pending_events: Vec::new(),
            unflushed_user_events: None,
            on_user_event: Box::new(on_user_event),
            on_mouse_down_resolved: None,

            on_resize: None,
            on_focus: None,
            animation_settle: DEFAULT_ANIMATION_SETTLE,
            redraw_until: Instant::now() + DEFAULT_ANIMATION_SETTLE,
            occluded: false,
            live_resize_until: Instant::now(),
            full_repaint_until: Instant::now() + DEFAULT_ANIMATION_SETTLE,
            force_full_repaint: true,
            pending_regions: Vec::new(),
            presented_last_frame: false,
            frame_waker: FrameWaker::default(),
            wake_regions: Vec::new(),
            frame_interval: Duration::from_secs_f32(TARGET_FRAME_SECONDS),
            frame_count: 0,
            fps_window_start: Instant::now(),
            last_mem_log: Instant::now(),
            last_fps: 0.0,
            last_frame_ms: 0.0,

            debug_font: Font::from_typeface(
                theme::make_typeface(Weight::NORMAL),
                18.0,
            ),
            #[cfg(debug_assertions)]
            automation: AutomationServer::start(
                automation::configured_port(),
            ),
            popouts: Vec::new(),
        };

        Self {
            event_loop,
            handler,
        }
    }

    pub fn proxy(&self) -> EventLoopProxy<T> {
        self.event_loop.create_proxy()
    }

    pub fn with_frame_waker(mut self, waker: FrameWaker) -> Self {
        self.handler.frame_waker = waker;
        self
    }

    pub fn frame_waker(&self) -> FrameWaker {
        self.handler.frame_waker.clone()
    }

    pub fn with_animation_settle(mut self, duration: Duration) -> Self {
        self.handler.animation_settle = duration;
        self.handler.redraw_until = Instant::now() + duration;
        self
    }

    pub fn with_shortcut_hook(
        self,
        hook: impl FnMut(&mut WidgetTree, &Chord) -> bool + 'static,
    ) -> Self {
        SHORTCUT_HOOK.with(|h| *h.borrow_mut() = Some(Box::new(hook)));
        self
    }

    pub fn with_mouse_down_hook(
        mut self,
        hook: impl FnMut(&mut WidgetTree, Option<WidgetId>) + 'static,
    ) -> Self {
        self.handler.on_mouse_down_resolved = Some(Box::new(hook));
        self
    }

    pub fn with_custom_chrome(mut self) -> Self {
        self.handler.custom_chrome = true;
        self
    }

    pub fn with_fixed_window_size(mut self, width: f32, height: f32) -> Self {
        self.handler.fixed_window_size = Some((width, height));
        self
    }

    pub fn with_frameless_window(mut self) -> Self {
        self.handler.frameless = true;
        self
    }

    pub fn with_focus_hook(mut self, hook: impl FnMut(bool) + 'static) -> Self {
        self.handler.on_focus = Some(Box::new(hook));
        self
    }

    pub fn with_resize_hook(mut self, hook: impl FnMut(f32, f32) + 'static) -> Self {
        self.handler.on_resize = Some(Box::new(hook));
        self
    }

    pub fn with_named_key_hook(
        self,
        hook: impl FnMut(&mut WidgetTree, NamedKey) -> bool + 'static,
    ) -> Self {
        NAMED_KEY_HOOK.with(|h| *h.borrow_mut() = Some(Box::new(hook)));
        self
    }

    pub fn with_text_fallback_hook(
        self,
        hook: impl FnMut(&mut WidgetTree, &UiEvent) -> bool + 'static,
    ) -> Self {
        TEXT_FALLBACK_HOOK.with(|h| *h.borrow_mut() = Some(Box::new(hook)));
        self
    }

    pub fn with_file_drop_hook(
        self,
        hook: impl FnMut(&mut WidgetTree, FileDrop) + 'static,
    ) -> Self {
        FILE_DROP_HOOK.with(|h| *h.borrow_mut() = Some(Box::new(hook)));
        self
    }

    pub fn run(self) -> Result<(), error::EventLoopError> {
        let mut handler = self.handler;
        self.event_loop.run_app(&mut handler)
    }
}

struct AppHandler<T: 'static> {
    title: String,

    skia: Option<SkiaGl>,
    tree: WidgetTree,
    previous_frame_start: Instant,
    cursor_pos: (f32, f32),
    pointer_in_window: bool,

    zoom: f32,

    active_touch: Option<u64>,

    touch_velocity_samples: Vec<(Instant, f32)>,

    fling_velocity_y: f32,

    fling_last_tick: Option<Instant>,

    modifiers: ModifiersState,

    current_cursor: event::CursorIcon,

    custom_chrome: bool,

    fixed_window_size: Option<(f32, f32)>,

    frameless: bool,

    resize_edge: Option<window::ResizeDirection>,

    closing: bool,

    defer_rebuilds: bool,
    pending_events: Vec<T>,

    unflushed_user_events: Option<(Option<usize>, bool)>,
    on_user_event: Box<dyn FnMut(T, &mut WidgetTree)>,

    on_mouse_down_resolved: Option<Box<dyn FnMut(&mut WidgetTree, Option<WidgetId>)>>,

    on_resize: Option<Box<dyn FnMut(f32, f32)>>,

    on_focus: Option<Box<dyn FnMut(bool)>>,

    animation_settle: Duration,

    redraw_until: Instant,

    occluded: bool,

    live_resize_until: Instant,

    full_repaint_until: Instant,

    force_full_repaint: bool,

    pending_regions: Vec<(skia_safe::IRect, Instant)>,

    presented_last_frame: bool,

    frame_waker: FrameWaker,

    wake_regions: Vec<skia_safe::IRect>,

    frame_interval: Duration,

    frame_count: u32,
    fps_window_start: Instant,

    last_mem_log: Instant,

    last_fps: f64,

    last_frame_ms: f64,

    #[cfg_attr(not(any(debug_assertions, feature = "fps-overlay")), allow(dead_code))]
    debug_font: Font,

    #[cfg(debug_assertions)]
    automation: AutomationServer,

    popouts: Vec<PopoutWindow>,
}

impl<T: 'static> ApplicationHandler<T> for AppHandler<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.skia.is_some() {
            return;
        }

        let mut attrs = Window::default_attributes()
            .with_title(&self.title)
            .with_visible(false);
        if self.fixed_window_size.is_some() {
            attrs = attrs.with_resizable(false);
        }
        if self.custom_chrome || self.frameless {
            attrs = attrs.with_decorations(false);
            #[cfg(windows)]
            {
                attrs = attrs.with_undecorated_shadow(true);
            }
        }
        let skia = SkiaGl::new(event_loop, attrs);
        match self.fixed_window_size {
            Some((width, height)) => size_and_center_logical(&skia.gl.window, width, height),
            None => size_and_center_on_primary_monitor(&skia.gl.window),
        }
        skia.gl.window.set_visible(true);

        skia.gl.window.set_ime_allowed(true);
        #[cfg(windows)]
        {
            if let Ok(RawWindowHandle::Win32(handle)) =
                skia.gl.window.window_handle().map(|h| h.as_raw())
            {
                NATIVE_WINDOW.store(handle.hwnd.get(), Ordering::Release);
            }
        }
        self.frame_interval = monitor_frame_interval(&skia.gl.window);
        self.frame_waker.attach(Some(skia.gl.window.clone()));
        self.skia = Some(skia);

        self.arm_animation_window();
        self.schedule_next_wakeup(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if !self.closing {
            self.finish_user_events(event_loop);
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.frame_waker.attach(None);
        self.skia = None;
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        process::exit(0);
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        self.pump_popouts(event_loop);

        if self.closing {
            return;
        }
        if let StartCause::ResumeTimeReached { .. } = cause {
            if let Some(skia) = &self.skia {
                skia.gl.window.request_redraw();
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: T) {
        if self.closing {
            return;
        }

        if self.defer_rebuilds {
            self.pending_events.push(event);
            return;
        }

        if self.unflushed_user_events.is_none() {
            self.unflushed_user_events =
                Some((paint::focus_index(&self.tree), self.tree.focus_visible));
        }
        (self.on_user_event)(event, &mut self.tree);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        self.finish_user_events(event_loop);

        if self.popouts.iter().any(|p| p.window_id() == id) {
            self.popout_event(event_loop, id, event);
            return;
        }
        let Some(skia) = self.skia.as_ref() else {
            return;
        };
        if skia.gl.window.id() != id {
            return;
        }

        if self.closing {
            return;
        }

        if matches!(event, WindowEvent::CloseRequested) {
            self.closing = true;
            event_loop.exit();
            return;
        }

        let scroll_input = matches!(&event, WindowEvent::MouseWheel { .. });

        if !matches!(
            event,
            WindowEvent::CursorMoved { .. }
                | WindowEvent::RedrawRequested
                | WindowEvent::Occluded(_)
                | WindowEvent::Moved(_)
        ) {
            self.force_full_repaint = true;
        }

        match event {
            WindowEvent::Resized(size) => {
                self.skia
                    .as_mut()
                    .expect("resumed builds skia before any WindowEvent can arrive")
                    .resize(size.width, size.height);
                let scale_factor = self.scale_factor();
                if let Some(hook) = &mut self.on_resize {
                    hook(
                        size.width as f32 / scale_factor,
                        size.height as f32 / scale_factor,
                    );
                }

                note_resized();
                self.live_resize_until = Instant::now() + LIVE_RESIZE_SETTLE;
                if !self.occluded && size.width > 0 && size.height > 0 {
                    signal::flush(&mut self.tree);
                    self.force_full_repaint = true;
                    self.render();
                }
            }
            WindowEvent::Focused(focused) => {
                if let Some(hook) = &mut self.on_focus {
                    hook(focused);
                }
            }
            WindowEvent::Occluded(occluded) => {
                self.occluded = occluded;
                if !occluded {
                    self.arm_animation_window();
                }
            }
            WindowEvent::Moved(_) | WindowEvent::ScaleFactorChanged { .. } => {
                self.frame_interval = monitor_frame_interval(
                    &self
                        .skia
                        .as_ref()
                        .expect("resumed builds skia before any WindowEvent can arrive")
                        .gl
                        .window,
                );
            }
            WindowEvent::RedrawRequested => {
                if self.occluded {
                    return;
                }
                self.render();

                self.apply_window_action(event_loop);
            }
            WindowEvent::CursorEntered { .. } => self.pointer_in_window = true,
            WindowEvent::CursorLeft { .. } => self.pointer_in_window = false,
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer_in_window = true;
                let scale = self.scale_factor();
                self.cursor_pos = (position.x as f32 / scale, position.y as f32 / scale);
                self.resize_edge = self.resize_edge_at(position.x as f32, position.y as f32);
                if inspect::picking().peek() {
                    inspect::pick_hover(&self.tree, self.cursor_pos.0, self.cursor_pos.1);
                }

                let hovered_before = self.tree.hovered_path.clone();
                paint::dispatch_event(
                    &mut self.tree,
                    &UiEvent::MouseMoved {
                        x: self.cursor_pos.0,
                        y: self.cursor_pos.1,
                    },
                );
                if self.tree.hovered_path != hovered_before {
                    self.arm_bounded_window();
                }
                self.refresh_cursor();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let button = map_mouse_button(button);
                let (x, y) = self.cursor_pos;
                match state {
                    ElementState::Pressed => {
                        if button == MouseButton::Left
                            && inspect::picking().peek()
                            && inspect::pick_click(&self.tree, x, y)
                        {
                            return;
                        }
                        if let (Some(edge), MouseButton::Left) = (self.resize_edge, button) {
                            if let Some(skia) = self.skia.as_ref() {
                                let _ = skia.gl.window.drag_resize_window(edge);
                            }
                            return;
                        }
                        self.defer_rebuilds = true;
                        let (_, handled_id) = paint::dispatch_event_with_handler(
                            &mut self.tree,
                            &UiEvent::MouseDown {
                                x,
                                y,
                                button,
                                shift: self.modifiers.shift_key(),
                            },
                        );
                        if let Some(hook) = &mut self.on_mouse_down_resolved {
                            hook(&mut self.tree, handled_id);
                        }
                        self.apply_window_action(event_loop);
                    }
                    ElementState::Released => {
                        paint::dispatch_event(&mut self.tree, &UiEvent::MouseUp { x, y, button });
                        paint::clear_all_press(&mut self.tree);

                        self.stop_deferring_and_flush();
                        self.apply_window_action(event_loop);
                    }
                }
            }

            WindowEvent::Touch(touch) => {
                let scale = self.scale_factor();
                let (x, y) = (
                    touch.location.x as f32 / scale,
                    touch.location.y as f32 / scale,
                );
                match touch.phase {
                    TouchPhase::Started => {
                        if self.active_touch.is_some() {
                            return;
                        }
                        self.active_touch = Some(touch.id);
                        self.cursor_pos = (x, y);

                        self.fling_velocity_y = 0.0;
                        self.fling_last_tick = None;
                        self.touch_velocity_samples.clear();
                        self.touch_velocity_samples.push((Instant::now(), y));

                        paint::dispatch_event(&mut self.tree, &UiEvent::MouseMoved { x, y });
                        self.defer_rebuilds = true;

                        let (_, handled_id) = paint::dispatch_event_with_handler(
                            &mut self.tree,
                            &UiEvent::MouseDown {
                                x,
                                y,
                                button: MouseButton::Left,
                                shift: false,
                            },
                        );
                        if let Some(hook) = &mut self.on_mouse_down_resolved {
                            hook(&mut self.tree, handled_id);
                        }
                    }
                    TouchPhase::Moved => {
                        if self.active_touch != Some(touch.id) {
                            return;
                        }
                        let (_prev_x, prev_y) = self.cursor_pos;
                        self.cursor_pos = (x, y);

                        if self.tree.pointer_capture.is_some() {
                            let hovered_before = self.tree.hovered_path.clone();
                            paint::dispatch_event(&mut self.tree, &UiEvent::MouseMoved { x, y });
                            if self.tree.hovered_path != hovered_before {
                                self.arm_bounded_window();
                            }
                        }

                        let delta_y = y - prev_y;
                        if delta_y != 0.0 {
                            paint::dispatch_event(
                                &mut self.tree,
                                &UiEvent::Scroll {
                                    x,
                                    y,
                                    delta_x: 0.0,
                                    delta_y,
                                    precise: true,
                                },
                            );
                            self.arm_animation_window();
                        }

                        let now = Instant::now();
                        self.touch_velocity_samples.push((now, y));
                        self.touch_velocity_samples
                            .retain(|&(when, _)| now.duration_since(when) <= FLING_SAMPLE_WINDOW);
                    }
                    TouchPhase::Ended | TouchPhase::Cancelled => {
                        if self.active_touch != Some(touch.id) {
                            return;
                        }
                        self.active_touch = None;

                        paint::dispatch_event(
                            &mut self.tree,
                            &UiEvent::MouseUp {
                                x,
                                y,
                                button: MouseButton::Left,
                            },
                        );
                        paint::clear_all_press(&mut self.tree);
                        self.stop_deferring_and_flush();

                        if let (Some(&(oldest_t, oldest_y)), Some(&(newest_t, newest_y))) = (
                            self.touch_velocity_samples.first(),
                            self.touch_velocity_samples.last(),
                        ) {
                            let dt = newest_t.duration_since(oldest_t).as_secs_f32();
                            if dt > 0.0 {
                                let velocity = (newest_y - oldest_y) / dt;
                                if velocity.abs() >= MIN_FLING_VELOCITY {
                                    self.fling_velocity_y = velocity;
                                    self.fling_last_tick = Some(Instant::now());
                                    self.arm_animation_window();
                                }
                            }
                        }
                        self.touch_velocity_samples.clear();
                    }
                }
                self.refresh_cursor();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (delta_x, delta_y, precise) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (
                        x * scroll_animation::WHEEL_NOTCH_PX,
                        y * scroll_animation::WHEEL_NOTCH_PX,
                        x.fract() != 0.0 || y.fract() != 0.0,
                    ),

                    MouseScrollDelta::PixelDelta(pos) => {
                        let scale = self.scale_factor();
                        (pos.x as f32 / scale, pos.y as f32 / scale, true)
                    }
                };
                let (x, y) = self.cursor_pos;
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
            }
            WindowEvent::HoveredFile(_)
            | WindowEvent::HoveredFileCancelled
            | WindowEvent::DroppedFile(_) => {
                let drop = match event {
                    WindowEvent::HoveredFile(_) => FileDrop::Hovered,
                    WindowEvent::DroppedFile(path) => FileDrop::Dropped {
                        path,
                        shift: self.modifiers.shift_key(),
                    },
                    _ => FileDrop::Cancelled,
                };
                run_file_drop_hook(&mut self.tree, drop);
                self.arm_animation_window();
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
                set_shift(self.modifiers.shift_key());
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } if key_event.state == ElementState::Pressed => {
                self.dispatch_key(&key_event);

                self.arm_animation_window();
            }
            WindowEvent::Ime(ime) => {
                match ime {
                    Ime::Enabled => {}
                    Ime::Preedit(text, _cursor) => {
                        paint::dispatch_key_event(&mut self.tree, &UiEvent::ImePreedit { text });
                    }

                    Ime::Commit(text) => {
                        paint::dispatch_key_event(&mut self.tree, &UiEvent::TextInput { text });
                    }
                    Ime::Disabled => {
                        paint::dispatch_key_event(
                            &mut self.tree,
                            &UiEvent::ImePreedit {
                                text: String::new(),
                            },
                        );
                    }
                }

                self.arm_animation_window();
            }
            _ => {}
        }

        let changed = signal::flush(&mut self.tree);

        self.refresh_cursor();

        if changed {
            if scroll_input {
                self.arm_bounded_window();
            } else {
                self.arm_animation_window();
            }
        }
        self.schedule_next_wakeup(event_loop);
    }
}

pub(crate) fn enter_render_context(scale: f32, size: (f32, f32)) -> (f32, (f32, f32), (f32, f32)) {
    let saved = (
        CURRENT_SCALE_FACTOR.with(|c| c.replace(scale)),
        CURRENT_WINDOW_SIZE.with(|c| c.replace(size)),
        values::viewport(),
    );
    values::set_viewport(size.0, size.1);
    saved
}

pub(crate) fn leave_render_context(saved: (f32, (f32, f32), (f32, f32))) {
    CURRENT_SCALE_FACTOR.with(|c| c.set(saved.0));
    CURRENT_WINDOW_SIZE.with(|c| c.set(saved.1));
    values::set_viewport(saved.2.0, saved.2.1);
}

fn monitor_frame_interval(window: &Window) -> Duration {
    window
        .current_monitor()
        .and_then(|monitor| monitor.refresh_rate_millihertz())
        .filter(|&millihertz| millihertz >= 1000)
        .map_or(
            Duration::from_secs_f32(TARGET_FRAME_SECONDS),
            |millihertz| Duration::from_secs_f64(1000.0 / millihertz as f64),
        )
}

fn size_and_center_on_primary_monitor(window: &Window) {
    let Some(monitor) = window.primary_monitor() else {
        return;
    };
    let monitor_size = monitor.size();
    let width = monitor_size.width / 2;
    let height = width * 9 / 16;
    let x = monitor.position().x + ((monitor_size.width - width) / 2) as i32;
    let y = monitor.position().y + ((monitor_size.height - height) / 2) as i32;

    let _ = window.request_inner_size(PhysicalSize::new(width, height));
    window.set_outer_position(PhysicalPosition::new(x, y));
}

fn size_and_center_logical(window: &Window, width: f32, height: f32) {
    let scale = window.scale_factor() as f32;
    let (w, h) = (
        (width * scale).round() as u32,
        (height * scale).round() as u32,
    );
    let _ = window.request_inner_size(PhysicalSize::new(w, h));
    if let Some(monitor) = window.primary_monitor() {
        let size = monitor.size();
        let x = monitor.position().x + (size.width.saturating_sub(w) / 2) as i32;
        let y = monitor.position().y + (size.height.saturating_sub(h) / 2) as i32;
        window.set_outer_position(PhysicalPosition::new(x, y));
    }
}

pub(crate) fn map_cursor_icon(icon: event::CursorIcon) -> CursorIcon {
    match icon {
        event::CursorIcon::Default => CursorIcon::Default,
        event::CursorIcon::Pointer => CursorIcon::Pointer,
        event::CursorIcon::Text => CursorIcon::Text,
        event::CursorIcon::ZoomIn => CursorIcon::ZoomIn,
        event::CursorIcon::ZoomOut => CursorIcon::ZoomOut,
        event::CursorIcon::Grab => CursorIcon::Grab,
        event::CursorIcon::Grabbing => CursorIcon::Grabbing,
        event::CursorIcon::NotAllowed => CursorIcon::NotAllowed,
        event::CursorIcon::ColResize => CursorIcon::ColResize,
    }
}

fn map_mouse_button(button: winit_event::MouseButton) -> MouseButton {
    match button {
        EventMouseButton::Left => MouseButton::Left,
        EventMouseButton::Right => MouseButton::Right,
        EventMouseButton::Middle => MouseButton::Middle,
        EventMouseButton::Back => MouseButton::Other(u16::MAX - 1),
        EventMouseButton::Forward => MouseButton::Other(u16::MAX),
        EventMouseButton::Other(code) => MouseButton::Other(code),
    }
}

impl<T: 'static> AppHandler<T> {
    fn arm_bounded_window(&mut self) {
        self.redraw_until = Instant::now() + self.animation_settle;
        self.force_full_repaint = true;
    }

    fn arm_animation_window(&mut self) {
        let now = Instant::now();
        self.redraw_until = now + self.animation_settle;
        self.full_repaint_until = now + DEFAULT_ANIMATION_SETTLE.min(self.animation_settle);
    }

    fn schedule_next_wakeup(&mut self, event_loop: &ActiveEventLoop) {
        static IDLE_MEM_LOG: OnceLock<bool> = OnceLock::new();

        if paint::take_repaint_request() {
            self.arm_animation_window();
        }

        let damage = paint::take_damage_requests();
        if !damage.is_empty() {
            let scale = self.scale_factor();
            let now = Instant::now();
            for rect in damage {
                let mut device = skia_safe::IRect::from_ltrb(
                    (rect.x * scale).floor() as i32,
                    (rect.y * scale).floor() as i32,
                    ((rect.x + rect.width) * scale).ceil() as i32,
                    ((rect.y + rect.height) * scale).ceil() as i32,
                );
                device.outset((2, 2));
                self.pending_regions.push((device, now));
            }
        }
        self.log_memory();
        let gpu_cleanup_deadline = self
            .skia
            .as_mut()
            .and_then(|skia| skia.maintain_gpu_cache());

        let Some(skia) = self.skia.as_ref() else {
            event_loop.set_control_flow(ControlFlow::Wait);
            return;
        };

        #[cfg(debug_assertions)]
        let keep_polling = true;
        #[cfg(not(debug_assertions))]
        let keep_polling = false;

        let mem_deadline = IDLE_MEM_LOG
            .get_or_init(|| env::var_os("UI_MEM_LOG").is_some())
            .then(|| self.last_mem_log + Duration::from_secs(10));
        let now = Instant::now();
        let next_region = self.pending_regions.iter().map(|&(_, at)| at).min();
        let animating = keep_polling
            || now < self.redraw_until
            || self.force_full_repaint
            || next_region.is_some_and(|at| at <= now);
        if animating && !self.occluded {
            if skia.gl.vsync_active() && self.presented_last_frame {
                skia.gl.window.request_redraw();
                event_loop.set_control_flow(ControlFlow::Wait);
            } else {
                let next_frame = self.previous_frame_start + self.frame_interval;
                if now >= next_frame {
                    skia.gl.window.request_redraw();
                    event_loop.set_control_flow(ControlFlow::Wait);
                } else {
                    event_loop.set_control_flow(ControlFlow::WaitUntil(next_frame));
                }
            }
        } else if let Some(deadline) = [
            timer::next_deadline(),
            next_region.filter(|_| !self.occluded),
            mem_deadline,
            gpu_cleanup_deadline,
        ]
        .into_iter()
        .flatten()
        .min()
        {
            event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }

    fn scale_factor(&self) -> f32 {
        let dpi = self
            .skia
            .as_ref()
            .map(|skia| skia.gl.window.scale_factor() as f32)
            .unwrap_or(1.0);
        dpi * self.zoom
    }

    fn step_zoom(&mut self, step: i32) {
        let current = ZOOM_LEVELS
            .iter()
            .position(|level| (level - self.zoom).abs() < 0.001)
            .unwrap_or_else(|| {
                ZOOM_LEVELS
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| {
                        (*a - self.zoom).abs().total_cmp(&(*b - self.zoom).abs())
                    })
                    .map(|(i, _)| i)
                    .unwrap_or(0)
            });
        let next = (current as i32 + step).clamp(0, ZOOM_LEVELS.len() as i32 - 1) as usize;
        self.set_zoom(ZOOM_LEVELS[next]);
    }

    fn set_zoom(&mut self, zoom: f32) {
        let previous_scale = self.scale_factor();
        self.zoom = zoom;
        let ratio = previous_scale / self.scale_factor();
        self.cursor_pos = (self.cursor_pos.0 * ratio, self.cursor_pos.1 * ratio);
        self.refresh_cursor();
    }

    fn apply_window_action(&mut self, event_loop: &ActiveEventLoop) {
        let Some(action) = WINDOW_ACTION.with(|c| c.take()) else {
            return;
        };
        let Some(skia) = self.skia.as_ref() else {
            return;
        };
        let window = &skia.gl.window;
        match action {
            WindowAction::Drag => {
                let _ = window.drag_window();

                paint::clear_all_press(&mut self.tree);
            }
            WindowAction::Minimize => window.set_minimized(true),
            WindowAction::ToggleMaximize => window.set_maximized(!window.is_maximized()),
            WindowAction::Close => {
                self.closing = true;
                event_loop.exit();
            }
            WindowAction::Focus => {
                window.set_minimized(false);
                window.focus_window();
            }
            WindowAction::Attention => {
                window.request_user_attention(Some(UserAttentionType::Informational))
            }
        }
    }

    fn resize_edge_at(&self, x: f32, y: f32) -> Option<window::ResizeDirection> {
        if !self.custom_chrome {
            return None;
        }
        let window = &self.skia.as_ref()?.gl.window;
        if window.is_maximized() || window.fullscreen().is_some() {
            return None;
        }
        let size = window.inner_size();
        let border = RESIZE_BORDER_PX * window.scale_factor() as f32;
        let (w, h) = (size.width as f32, size.height as f32);
        let (left, right, top, bottom) = (x < border, x >= w - border, y < border, y >= h - border);
        Some(match (top, bottom, left, right) {
            (true, _, true, _) => D::NorthWest,
            (true, _, _, true) => D::NorthEast,
            (_, true, true, _) => D::SouthWest,
            (_, true, _, true) => D::SouthEast,
            (true, _, _, _) => D::North,
            (_, true, _, _) => D::South,
            (_, _, true, _) => D::West,
            (_, _, _, true) => D::East,
            _ => return None,
        })
    }

    fn refresh_cursor(&mut self) {
        if let Some(edge) = self.resize_edge {
            let icon = match edge {
                D::North | D::South => C::NsResize,
                D::East | D::West => C::EwResize,
                D::NorthWest | D::SouthEast => C::NwseResize,
                D::NorthEast | D::SouthWest => C::NeswResize,
            };
            if let Some(skia) = self.skia.as_ref() {
                skia.gl.window.set_cursor(icon);
            }

            self.current_cursor = event::CursorIcon::Text;
            return;
        }
        let wanted = paint::cursor_at(&self.tree, self.cursor_pos.0, self.cursor_pos.1);
        if wanted != self.current_cursor {
            self.current_cursor = wanted;

            if let Some(skia) = self.skia.as_ref() {
                skia.gl.window.set_cursor(map_cursor_icon(wanted));
            }
        }
    }

    fn flush_popouts(&mut self) -> bool {
        let mut any = false;
        for popout in &mut self.popouts {
            any |= popout.flush();
        }
        any
    }

    fn pump_popouts(&mut self, event_loop: &ActiveEventLoop) {
        for id in popout::take_close_requests() {
            if let Some(index) = self.popouts.iter().position(|p| p.id == id) {
                let mut popout = self.popouts.remove(index);
                if let Some(on_close) = popout.take_on_close() {
                    on_close();
                }

                drop(popout);
                if let Some(skia) = self.skia.as_mut() {
                    skia.begin();
                }
                signal::flush(&mut self.tree);
                self.arm_animation_window();
            }
        }
        for (id, spec) in popout::take_open_requests() {
            let rules = self.tree.rules.clone();
            self.popouts.push(PopoutWindow::new(
                event_loop, id, spec, rules,
            ));
            if let Some(skia) = self.skia.as_mut() {
                skia.begin();
            }
            self.arm_animation_window();
        }
    }

    fn popout_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let rules = self.tree.rules.clone();
        let Some(index) = self.popouts.iter().position(|p| p.window_id() == id) else {
            return;
        };
        self.popouts[index].sync_rules(rules);
        let was_redraw = matches!(event, WindowEvent::RedrawRequested);
        match self.popouts[index].handle_event(event) {
            Outcome::Close => {
                popout::close(self.popouts[index].id);
            }
            Outcome::Continue => {}
        }
        if !was_redraw {
            if signal::flush(&mut self.tree) {
                self.arm_animation_window();
            }
            self.flush_popouts();
        }
        self.pump_popouts(event_loop);
        self.schedule_next_wakeup(event_loop);
    }

    fn finish_user_events(&mut self, event_loop: &ActiveEventLoop) {
        let Some((focus_index, focus_visible)) = self.unflushed_user_events.take() else {
            return;
        };
        signal::flush(&mut self.tree);
        self.flush_popouts();
        paint::restore_focus(&mut self.tree, focus_index, focus_visible);
        paint::clear_hidden_focus(&mut self.tree);
        self.refresh_cursor();
        self.pump_popouts(event_loop);
        self.apply_window_action(event_loop);

        self.arm_bounded_window();
        self.schedule_next_wakeup(event_loop);
    }

    fn apply_user_event(&mut self, event: T) {
        let focus_index = paint::focus_index(&self.tree);
        let focus_visible = self.tree.focus_visible;
        (self.on_user_event)(event, &mut self.tree);

        signal::flush(&mut self.tree);
        self.flush_popouts();
        paint::restore_focus(&mut self.tree, focus_index, focus_visible);
        paint::clear_hidden_focus(&mut self.tree);
        self.refresh_cursor();
    }

    fn stop_deferring_and_flush(&mut self) {
        self.defer_rebuilds = false;
        if !self.pending_events.is_empty() {
            for event in mem::take(&mut self.pending_events) {
                self.apply_user_event(event);
            }

            self.arm_animation_window();
        }
    }

    fn dispatch_key(&mut self, key_event: &winit_event::KeyEvent) {
        let shift = self.modifiers.shift_key();

        let unmodified_key = key_event.key_without_modifiers();
        if self.modifiers.control_key() {
            if let Key::Character(ch) = &unmodified_key {
                let zoom_step = match ch.as_str() {
                    "+" | "=" => Some(1),
                    "-" | "_" => Some(-1),
                    _ => None,
                };
                if let Some(step) = zoom_step {
                    self.step_zoom(step);
                    return;
                }
                if ch.as_str() == "0" {
                    self.set_zoom(DEFAULT_ZOOM);
                    return;
                }

                let shortcut = if self.modifiers.alt_key() {
                    None
                } else {
                    keymap::ctrl_shortcut(ch.as_str(), shift)
                };
                if let Some(key) = shortcut {
                    let event = UiEvent::KeyDown { key, shift };
                    if key == NamedKey::Paste && self.route_text_fallback(&event) {
                        return;
                    }

                    if key == NamedKey::Copy && run_named_key_hook(&mut self.tree, key) {
                        return;
                    }
                    paint::dispatch_key_event(&mut self.tree, &event);
                    return;
                }
            }
        }

        if let Some(chord) =
            Chord::from_key(&unmodified_key, key_event.text.as_deref(), self.modifiers)
                .filter(Chord::is_global)
        {
            self.defer_rebuilds = true;
            let handled = run_shortcut_hook(&mut self.tree, &chord);
            self.stop_deferring_and_flush();
            if handled {
                return;
            }
        }

        let named = match &key_event.logical_key {
            Key::Named(key) => keymap::named_key(key),
            _ => None,
        };

        let named = named.map(|key| {
            let is_mac = cfg!(target_os = "macos");
            let word = if is_mac {
                self.modifiers.alt_key()
            } else {
                self.modifiers.control_key()
            };
            let super_key = is_mac && self.modifiers.super_key();
            modified_nav_key(key, word, super_key, is_mac)
        });

        if named == Some(NamedKey::Tab) {
            self.defer_rebuilds = true;
            paint::dispatch_tab(&mut self.tree, shift);
            self.stop_deferring_and_flush();
            return;
        }

        if let Some(key) = named {
            if run_named_key_hook(&mut self.tree, key) {
                return;
            }
            paint::dispatch_key_event(&mut self.tree, &UiEvent::KeyDown { key, shift });
            return;
        }

        if let Some(text) = &key_event.text {
            if !text.is_empty() && !text.chars().any(|c| c.is_control()) {
                let event = UiEvent::TextInput {
                    text: text.to_string(),
                };
                if !self.modifiers.super_key() && self.route_text_fallback(&event) {
                    return;
                }
                paint::dispatch_key_event(&mut self.tree, &event);
            }
        }
    }

    fn route_text_fallback(&mut self, event: &UiEvent) -> bool {
        let focused_takes_text = self.tree.focused.is_some_and(|id| {
            self.tree
                .nodes
                .get(id)
                .is_some_and(|n| n.widget.wants_text_input())
        });
        if focused_takes_text {
            return false;
        }
        self.defer_rebuilds = true;
        let tree = &mut self.tree;
        let handled = TEXT_FALLBACK_HOOK.with(|h| {
            h.borrow_mut()
                .as_mut()
                .is_some_and(|hook| hook(tree, event))
        });
        self.stop_deferring_and_flush();
        handled
    }

    fn render(&mut self) {
        self.previous_frame_start = Instant::now();
        let work_started = Instant::now();
        perf::set_frame_budget_ms(self.frame_interval.as_secs_f64() * 1000.0);

        if !self.popouts.is_empty() {
            self.skia
                .as_mut()
                .expect("render only ever called while resumed")
                .begin();
        }
        if timer::run_due() {
            signal::flush(&mut self.tree);
            self.flush_popouts();

            self.arm_bounded_window();
        }

        if inspect::take_stats_wanted() {
            let (gpu_resources, gpu_bytes, gpu_limit) = self
                .skia
                .as_ref()
                .map_or((0, 0, 0), |s| s.gpu_cache_stats());
            inspect::publish_stats(inspect::Stats {
                fps: self.last_fps,
                frame_ms: self.last_frame_ms,
                gpu_resources,
                gpu_bytes,
                gpu_limit,
                window: CURRENT_WINDOW_SIZE.with(|c| c.get()),
                scale: self.scale_factor(),
            });
            signal::flush(&mut self.tree);
            self.flush_popouts();
        }
        if inspect::has_requests() && inspect::service(&self.tree) {
            signal::flush(&mut self.tree);
            self.flush_popouts();
            self.arm_animation_window();
        }
        if inspect::take_highlight_dirty() {
            self.force_full_repaint = true;
        }

        #[cfg(debug_assertions)]
        let (pending_screenshots, automation_ran) = self
            .automation
            .drain(&mut self.tree, &mut self.on_mouse_down_resolved);
        #[cfg(debug_assertions)]
        if automation_ran {
            signal::flush(&mut self.tree);
            self.flush_popouts();
        }
        #[cfg(debug_assertions)]
        if automation_ran || !pending_screenshots.is_empty() {
            self.force_full_repaint = true;
        }

        if let Some(last_tick) = self.fling_last_tick {
            let now = Instant::now();
            let dt = now.duration_since(last_tick).as_secs_f32();
            if dt > 0.0 {
                let delta_y = self.fling_velocity_y * dt;
                let (x, y) = self.cursor_pos;
                paint::dispatch_event(
                    &mut self.tree,
                    &UiEvent::Scroll {
                        x,
                        y,
                        delta_x: 0.0,
                        delta_y,
                        precise: true,
                    },
                );
                self.fling_velocity_y *= FLING_DECAY_PER_SECOND.powf(dt);
                if self.fling_velocity_y.abs() < MIN_FLING_VELOCITY {
                    self.fling_velocity_y = 0.0;
                    self.fling_last_tick = None;
                } else {
                    self.fling_last_tick = Some(now);

                    self.arm_animation_window();
                }
            }
        }

        let scale_factor = self.scale_factor();
        let size = self
            .skia
            .as_mut()
            .expect("render only ever called while resumed")
            .gl
            .window
            .inner_size();

        CURRENT_SCALE_FACTOR.with(|c| c.set(scale_factor));

        let available_height_px = size.height as f32;

        CURRENT_TOP_INSET.with(|c| c.set(0.0));
        CURRENT_WINDOW_SIZE.with(|c| {
            c.set((
                size.width as f32 / scale_factor,
                available_height_px / scale_factor,
            ))
        });

        if values::set_viewport(
            size.width as f32 / scale_factor,
            available_height_px / scale_factor,
        ) {
            self.tree.invalidate_styles();
        }
        if cascade::refresh_theme(&mut self.tree) {
            self.force_full_repaint = true;
        }
        if self.tree.style_dirty {
            self.force_full_repaint = true;
        }
        cascade::recompute_if_dirty(&mut self.tree);
        let layout_space = TaffySize {
            width: AvailableSpace::Definite(size.width as f32 / scale_factor),
            height: AvailableSpace::Definite(available_height_px / scale_factor),
        };
        perf::time(Phase::Layout, || {
            layout::compute_layout(&mut self.tree, layout_space)
        });

        // Rebuilds, scrolling and relayout move content under a pointer that isn't moving, and
        // hover is otherwise only resolved on mouse moves. Re-resolve it on the settled layout.
        if self.pointer_in_window && self.tree.pointer_capture.is_none() {
            let (x, y) = self.cursor_pos;
            if paint::hover_out_of_date(&self.tree, x, y) {
                paint::dispatch_event(&mut self.tree, &UiEvent::MouseMoved { x, y });
                signal::flush(&mut self.tree);
                self.force_full_repaint = true;
                cascade::recompute_if_dirty(&mut self.tree);
                perf::time(Phase::Layout, || {
                    layout::compute_layout(&mut self.tree, layout_space)
                });
            }
        }

        for _ in 0..4 {
            if !layout::publish_observed_sizes(&mut self.tree) {
                break;
            }
            signal::flush(&mut self.tree);
            self.force_full_repaint = true;
            cascade::recompute_if_dirty(&mut self.tree);
            perf::time(Phase::Layout, || {
                layout::compute_layout(&mut self.tree, layout_space)
            });
        }

        let now = Instant::now();
        let mut due_regions = Vec::new();
        let woken = self.frame_waker.take_woken();
        if woken {
            due_regions.append(&mut self.wake_regions);
        }
        self.pending_regions.retain(|&(rect, at)| {
            if at <= now {
                due_regions.push(rect);
                false
            } else {
                true
            }
        });
        let carried_regions = mem::take(&mut self.pending_regions);
        let carried_wake_regions = mem::take(&mut self.wake_regions);
        let skia = self
            .skia
            .as_mut()
            .expect("render only ever called while resumed");
        let (canvas, cache_was_reset) = skia.frame_cache_canvas();
        let full = mem::take(&mut self.force_full_repaint)
            || now < self.full_repaint_until
            || cache_was_reset;
        if !full && due_regions.is_empty() {
            self.pending_regions = carried_regions;
            self.wake_regions = carried_wake_regions;
            self.presented_last_frame = false;
            return;
        }

        perf::count_frame(full);
        canvas.save();
        if !full {
            let mut damage = skia_safe::Region::new();
            for rect in &due_regions {
                damage.op_rect(*rect, RegionOp::Union);
            }
            canvas.clip_region(&damage, None);
        }

        let clear_color = self.tree.nodes[self.tree.root]
            .style
            .background
            .unwrap_or(Color::from_argb(255, 0, 0, 0));
        canvas.clear(clear_color);

        canvas.save();
        canvas.scale((scale_factor, scale_factor));
        perf::time(Phase::Paint, || {
            paint::clear_hidden_focus(&mut self.tree);
            paint::paint(&mut self.tree, canvas)
        });
        canvas.restore();
        canvas.restore();

        let animation = paint::take_animation();
        if animation.full {
            self.force_full_repaint = true;
        }

        let untouched = |rect: &skia_safe::IRect| {
            !full && !due_regions.iter().any(|due| skia_safe::IRect::intersects(due, rect))
        };
        self.pending_regions = carried_regions
            .into_iter()
            .filter(|(rect, _)| untouched(rect))
            .chain(animation.regions)
            .collect();
        self.wake_regions = carried_wake_regions
            .into_iter()
            .filter(untouched)
            .chain(animation.wake_regions)
            .collect();

        skia.blit_frame_cache();
        if inspect::has_highlight() {
            inspect::paint_highlight(
                &self.tree,
                skia.canvas(),
                &self.debug_font,
                scale_factor,
            );
        }
        #[cfg(any(debug_assertions, feature = "fps-overlay"))]
        let canvas = skia.canvas();

        #[cfg(any(debug_assertions, feature = "fps-overlay"))]
        draw_debug_overlay(
            canvas,
            &self.debug_font,
            self.last_fps,
            self.last_frame_ms,
            scale_factor,
        );

        #[cfg(debug_assertions)]
        if !pending_screenshots.is_empty() {
            let png = skia.screenshot_png();
            for (_, reply) in pending_screenshots {
                automation::reply_screenshot(png.clone(), reply);
            }
        }

        skia.gl
            .set_live_resize(Instant::now() < self.live_resize_until);
        perf::note_frame_work(work_started.elapsed().as_secs_f64() * 1000.0);
        perf::time(Phase::Present, || skia.present());
        self.presented_last_frame = true;

        if let Some(on) = FULLSCREEN_REQUEST.with(|c| c.take()) {
            let fullscreen = on.then_some(Fullscreen::Borderless(None));
            self.skia
                .as_ref()
                .expect("render only ever called while resumed")
                .gl
                .window
                .set_fullscreen(fullscreen);
        }

        self.last_frame_ms = self.previous_frame_start.elapsed().as_secs_f64() * 1000.0;
        self.update_fps();
    }

    fn update_fps(&mut self) {
        self.frame_count += 1;
        let elapsed = self.fps_window_start.elapsed();
        if elapsed >= Duration::from_secs(1) {
            self.last_fps = self.frame_count as f64 / elapsed.as_secs_f64();
            log::debug!(
                "render: {:.1} fps, last frame {:.2}ms (layout+paint+present)",
                self.last_fps,
                self.last_frame_ms
            );
            let perf = perf::take();
            log::debug!(
                target: "requiem_ui::perf",
                "per second: {} frames ({} full, {} partial) | cascade {}x {:.1}ms | flush {:.1}ms | layout {:.1}ms | paint {:.1}ms | present {:.1}ms | {} nodes | worst frame {:.1}ms, {} over the {:.1}ms budget",
                self.frame_count, perf.full_frames, perf.partial_frames, perf.cascade_count, perf.cascade_ms, perf.flush_ms, perf.layout_ms, perf.paint_ms, perf.present_ms, self.tree.nodes.len(), perf.worst_frame_ms, perf.over_budget_frames, self.frame_interval.as_secs_f64() * 1000.0
            );
            if let Some(report) = perf::take_widget_paint_report() {
                log::debug!(target: "requiem_ui::perf", "paint by widget (own time): {report}");
            }
            self.frame_count = 0;
            self.fps_window_start = Instant::now();
        }
    }

    fn log_memory(&mut self) {
        if self.last_mem_log.elapsed() < Duration::from_secs(10) {
            return;
        }
        self.last_mem_log = Instant::now();
        let mut parts = Vec::new();
        if let Some((working, private)) = perf::process_memory() {
            parts.push(format!(
                "process: working {} private {}",
                perf::mb(working),
                perf::mb(private)
            ));
        }
        if let Some(skia) = self.skia.as_ref() {
            let (count, bytes, limit) = skia.gpu_cache_stats();
            parts.push(format!(
                "skia gpu cache: {count} resources {} / {} (purgeable {})",
                perf::mb(bytes),
                perf::mb(limit),
                perf::mb(skia.gpu_cache_purgeable())
            ));
        }
        parts.push(format!(
            "skia font/image cpu cache: {} / {}",
            perf::mb(graphics::resource_cache_total_bytes_used()),
            perf::mb(graphics::resource_cache_total_bytes_limit())
        ));
        parts.push(format!(
            "rust heap: {}",
            perf::mb(perf::heap_bytes())
        ));
        parts.push(format!("widget nodes: {}", self.tree.nodes.len()));
        if let Some(win) = perf::win_heap_bytes() {
            parts.push(format!(
                "win heaps: {} (native = {})",
                perf::mb(win),
                perf::mb(win.saturating_sub(perf::heap_bytes()))
            ));
        }
        if let Some((image, mapped, private)) = perf::committed_by_type() {
            parts.push(format!(
                "committed: dll/exe {} mapped files {} private {}",
                perf::mb(image),
                perf::mb(mapped),
                perf::mb(private)
            ));
        }
        if let Some(resident) = perf::working_set_breakdown() {
            parts.push(resident);
        }
        if env::var_os("UI_GPU_DUMP").is_some() {
            if let Some(skia) = self.skia.as_mut() {
                parts.push(skia.gpu_dump());
            }
        }
        let (scaled_n, scaled_bytes) = primitives::scaled_cache_stats();
        parts.push(format!(
            "scaled images: {scaled_n} {}",
            perf::mb(scaled_bytes)
        ));
        parts.extend(perf::mem_probe_reports());
        log::info!(target: "requiem_ui::mem", "{}", parts.join(" | "));
    }
}

#[cfg(any(debug_assertions, feature = "fps-overlay"))]
fn draw_debug_overlay(
    canvas: &skia_safe::Canvas,
    font: &Font,
    fps: f64,
    frame_ms: f64,
    scale_factor: f32,
) {
    let text = format!("{fps:.1} fps | {frame_ms:.2}ms");

    canvas.save();
    canvas.scale((scale_factor, scale_factor));

    let (_, bounds) = font.measure_str(&text, None);
    let mut bg_paint = skia_safe::Paint::default();
    bg_paint.set_color(Color::from_argb(160, 0, 0, 0));
    canvas.draw_rect(
        skia_safe::Rect::from_xywh(8.0, 8.0, bounds.width() + 20.0, 36.0),
        &bg_paint,
    );

    let mut text_paint = skia_safe::Paint::default();
    text_paint.set_color(Color::from_argb(255, 0, 255, 0));
    text_paint.set_anti_alias(true);
    canvas.draw_str(&text, (18.0, 32.0), font, &text_paint);

    canvas.restore();
}

pub(crate) fn modified_nav_key(
    key: NamedKey,
    word: bool,
    super_key: bool,
    is_mac: bool,
) -> NamedKey {
    if super_key {
        return match key {
            NamedKey::ArrowLeft => NamedKey::Home,
            NamedKey::ArrowRight => NamedKey::End,
            NamedKey::ArrowUp => NamedKey::DocStart,
            NamedKey::ArrowDown => NamedKey::DocEnd,
            other => other,
        };
    }
    if !word {
        return key;
    }
    match key {
        NamedKey::ArrowLeft => NamedKey::WordLeft,
        NamedKey::ArrowRight => NamedKey::WordRight,
        NamedKey::Backspace => NamedKey::DeleteWordBack,
        NamedKey::Delete => NamedKey::DeleteWordForward,
        NamedKey::Home if !is_mac => NamedKey::DocStart,
        NamedKey::End if !is_mac => NamedKey::DocEnd,
        other => other,
    }
}

#[cfg(test)]
mod nav_key_tests {
    use super::*;

    #[test]
    fn word_modifier_maps_arrows_and_deletes() {
        assert_eq!(
            modified_nav_key(NamedKey::ArrowLeft, true, false, false),
            NamedKey::WordLeft
        );
        assert_eq!(
            modified_nav_key(NamedKey::ArrowRight, true, false, false),
            NamedKey::WordRight
        );
        assert_eq!(
            modified_nav_key(NamedKey::Backspace, true, false, false),
            NamedKey::DeleteWordBack
        );
        assert_eq!(
            modified_nav_key(NamedKey::Delete, true, false, false),
            NamedKey::DeleteWordForward
        );
        assert_eq!(
            modified_nav_key(NamedKey::Home, true, false, false),
            NamedKey::DocStart
        );
        assert_eq!(
            modified_nav_key(NamedKey::End, true, false, false),
            NamedKey::DocEnd
        );
        assert_eq!(
            modified_nav_key(NamedKey::Enter, true, false, false),
            NamedKey::Enter
        );
    }

    #[test]
    fn unmodified_keys_pass_through() {
        assert_eq!(
            modified_nav_key(NamedKey::ArrowLeft, false, false, false),
            NamedKey::ArrowLeft
        );
    }

    #[test]
    fn mac_uses_cmd_for_line_and_doc_ends() {
        assert_eq!(
            modified_nav_key(NamedKey::ArrowLeft, false, true, true),
            NamedKey::Home
        );
        assert_eq!(
            modified_nav_key(NamedKey::ArrowDown, false, true, true),
            NamedKey::DocEnd
        );

        assert_eq!(
            modified_nav_key(NamedKey::Home, true, false, true),
            NamedKey::Home
        );
    }
}
