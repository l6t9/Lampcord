use crate::config;
use crate::ui::classes::media_viewer as css;
use crate::ui;
use crate::utils::format;
use crate::utils::link as utils_link;
#[cfg(feature = "video")]
use crate::video;
use requiem_ui::app;
use requiem_ui::clipboard;
use requiem_ui::components::ButtonVariant;
use requiem_ui::components::StackProps;
use requiem_ui::components::TextVariant;
use requiem_ui::components::toast::ToastState;
use requiem_ui::icons::{Icon, IconPath};
use requiem_ui::paint as requiem_ui_paint;
use requiem_ui::signal as requiem_ui_signal;
use requiem_ui::timer;
use requiem_ui::widget::primitives;
use skia_safe::image::CachingHint;
#[cfg(feature = "video")]
use skia_safe::images;
use std::cell::{Cell, RefCell};
#[cfg(feature = "video")]
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

use requiem_ui::components::{
    ButtonProps, TextProps, TooltipPosition, TooltipProps, div, icon, tooltip,
};
use requiem_ui::control::section;
use requiem_ui::event::{CursorIcon, EventResult, MouseButton, UiEvent};
use requiem_ui::node::Node;
use requiem_ui::signal::{Effect, Signal, signal};
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::primitives::{ClickAreaWidget, ImageWidget};
use requiem_ui::widget::{AbsoluteRect, Widget, WidgetId, WidgetTree};
use skia_safe::{Canvas, Image, Paint, Rect};

use crate::images::ImageCache;
use crate::models::Id;
use crate::models::id::marker::UserMarker;
use crate::ui::{Fonts, TEXT_STRONG, TEXT_SUBTLE};
use requiem_ui::components::toast::ToastKind;
use requiem_ui::icons::{
    ARROW_LARGE_LEFT, ARROW_LARGE_RIGHT, CHEVRON_SMALL_RIGHT, COPY, DOWNLOAD, LINK,
    MAGNIFYING_GLASS_MINUS, MAGNIFYING_GLASS_PLUS, MORE_HORIZONTAL, WINDOW_LAUNCH, X_SMALL,
};

const ZOOM_SCALE: f32 = 2.5;

const OPEN_SCALE: f32 = 0.9;

const CLICK_SLOP_SQ: f32 = 400.0;

const FADE: Duration = Duration::from_millis(200);

#[derive(Clone, Debug, PartialEq)]
pub struct ViewerAuthor {
    pub id: Id<UserMarker>,
    pub discriminator: String,
    pub username: String,
    pub avatar_url: Option<String>,

    pub timestamp: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MediaItem {
    pub url: String,

    pub video_url: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,

    pub placeholder: Option<String>,
    pub filename: Option<String>,
    pub size: Option<u64>,
    pub alt: Option<String>,
    pub author: Option<ViewerAuthor>,

    pub is_attachment: bool,

    pub zoom_only: bool,
}

impl MediaItem {
    pub fn link(&self) -> &str {
        self.video_url.as_deref().unwrap_or(&self.url)
    }
}

pub fn scale_to_fit(w: f32, h: f32, max_w: f32, max_h: f32) -> (f32, f32) {
    if w == max_w && h == max_h {
        return (w, h);
    }
    let by_width = if w > max_w { max_w / w } else { 1.0 };
    let (w, h) = (
        (w * by_width).round().max(0.0),
        (h * by_width).round().max(0.0),
    );
    let by_height = if h > max_h { max_h / h } else { 1.0 };
    (
        (w * by_height).round().max(0.0),
        (h * by_height).round().max(0.0),
    )
}

pub fn fit_dims(multiple: bool, w: f32, h: f32, window: (f32, f32)) -> (f32, f32) {
    let (inner_w, inner_h) = window;
    let max_w = inner_w - if multiple { 76.0 } else { 24.0 } * 2.0;
    let max_h = inner_h - 176.0;
    if w <= 0.0 || h <= 0.0 {
        return (max_w.max(0.0), max_h.max(0.0));
    }
    let wide = scale_to_fit(w, h, max_w, max_h);
    let narrow = scale_to_fit(
        w,
        h,
        inner_w - 544.0,
        inner_h - if multiple { 88.0 } else { 36.0 } * 2.0,
    );
    if wide.0 >= narrow.0 { wide } else { narrow }
}

pub fn clamp_pan(
    size: (f32, f32),
    viewport: (f32, f32),
    offset: (f32, f32),
    delta: (f32, f32),
) -> (f32, f32) {
    let (width, height) = size;
    let (vw, vh) = viewport;
    let left = (vw - width) / 2.0 + offset.0;
    let top = (vh - height) / 2.0 + offset.1;
    let right = (vw + width) / 2.0 + offset.0;
    let bottom = (vh + height) / 2.0 + offset.1;
    let (mut x, mut y) = offset;
    if width > vw {
        x += delta.0;
        if left + delta.0 > 0.0 {
            x = (width - vw) / 2.0;
        }
        if right + delta.0 < vw {
            x = (vw - width) / 2.0;
        }
    }
    if height > vh {
        y += delta.1;
        if top + delta.1 > 0.0 {
            y = (height - vh) / 2.0;
        }
        if bottom + delta.1 < vh {
            y = (vh - height) / 2.0;
        }
    }
    (x, y)
}

#[derive(Clone, Copy, Debug)]
pub struct Spring {
    pub value: f32,
    velocity: f32,
    pub goal: f32,
}

impl Spring {
    const TENSION: f32 = 300.0;
    const FRICTION: f32 = 30.0;

    pub fn new(value: f32) -> Self {
        Self {
            value,
            velocity: 0.0,
            goal: value,
        }
    }

    pub fn set(&mut self, value: f32) {
        self.value = value;
        self.goal = value;
        self.velocity = 0.0;
    }

    pub fn start(&mut self, goal: f32) {
        self.goal = goal;
    }

    pub fn at_rest(&self) -> bool {
        (self.value - self.goal).abs() < 0.001 && self.velocity.abs() < 0.001
    }

    pub fn step(&mut self, dt: f32) {
        if self.at_rest() {
            self.value = self.goal;
            self.velocity = 0.0;
            return;
        }
        let mut remaining = dt.min(0.25);
        while remaining > 0.0 {
            let h = remaining.min(1.0 / 240.0);
            let acceleration =
                -Self::TENSION * (self.value - self.goal) - Self::FRICTION * self.velocity;
            self.velocity += acceleration * h;
            self.value += self.velocity * h;
            remaining -= h;
        }
        if self.at_rest() {
            self.value = self.goal;
            self.velocity = 0.0;
        }
    }
}

#[cfg(feature = "video")]
pub fn video_frames(videos: video::VideoCache) -> Rc<dyn Fn(&str) -> Option<Image>> {
    let cache: RefCell<HashMap<String, (u64, Image)>> =
        RefCell::new(HashMap::new());
    Rc::new(move |url| {
        let player = videos.get_or_open(url, true);
        if !player.is_playing() && !player.has_ended() && !player.has_failed() {
            player.play();
        }
        let version = player.frame_version();
        let mut cache = cache.borrow_mut();
        if cache.get(url).is_none_or(|(v, _)| *v != version) {
            if let Some(frame) = player.current_frame() {
                let info = skia_safe::ImageInfo::new(
                    (frame.width as i32, frame.height as i32),
                    skia_safe::ColorType::RGBA8888,
                    skia_safe::AlphaType::Unpremul,
                    None,
                );
                if let Some(image) = images::raster_from_data(
                    &info,
                    skia_safe::Data::new_copy(&frame.rgba),
                    frame.width as usize * 4,
                ) {
                    cache.insert(url.to_string(), (version, image));
                }
            }
        }
        cache.get(url).map(|(_, image)| image.clone())
    })
}

#[derive(Clone)]
pub struct ViewerSession {
    pub items: Rc<Vec<MediaItem>>,
}

#[derive(Clone)]
pub struct MediaViewerState {
    pub session: Signal<Option<ViewerSession>>,
    pub index: Signal<usize>,

    pub shown: Signal<bool>,
    pub zoomed: Signal<bool>,

    pub menu_open: Signal<bool>,

    pub details_open: Signal<bool>,

    details_left: Rc<Cell<bool>>,

    zoom_command: Rc<Cell<Option<bool>>>,

    opening: Rc<Cell<bool>>,

    pub on_save: Option<Rc<dyn Fn(Vec<u8>, String)>>,

    pub video_frame: Option<Rc<dyn Fn(&str) -> Option<Image>>>,

    pub toasts: ToastState,
}

impl MediaViewerState {
    pub fn with_toasts(mut self, toasts: ToastState) -> Self {
        self.toasts = toasts;
        self
    }

    pub fn with_video_frames(mut self, source: Rc<dyn Fn(&str) -> Option<Image>>) -> Self {
        self.video_frame = Some(source);
        self
    }

    pub fn new(on_save: Option<Rc<dyn Fn(Vec<u8>, String)>>) -> Self {
        Self {
            session: signal(None),
            index: signal(0),
            shown: signal(false),
            zoomed: signal(false),
            menu_open: signal(false),
            details_open: signal(false),
            details_left: Rc::new(Cell::new(false)),
            zoom_command: Rc::new(Cell::new(None)),
            opening: Rc::new(Cell::new(false)),
            on_save,
            video_frame: None,
            toasts: ToastState::new(),
        }
    }

    pub fn inert() -> Self {
        Self::new(None)
    }

    pub fn is_open(&self) -> bool {
        self.session.peek().is_some()
    }

    pub fn open(&self, items: Vec<MediaItem>, start: usize) {
        if items.is_empty() {
            return;
        }
        self.zoomed.set(false);
        self.menu_open.set(false);
        self.details_open.set(false);
        self.zoom_command.set(None);
        self.opening.set(true);
        self.index.set(start.min(items.len() - 1));
        self.shown.set(false);
        self.session.set(Some(ViewerSession {
            items: Rc::new(items),
        }));

        let shown = self.shown.clone();
        timer::set_timeout(Duration::from_millis(16), move || shown.set(true));
    }

    pub fn reset(&self) {
        self.menu_open.set(false);
        self.details_open.set(false);
        self.zoomed.set(false);
        self.shown.set(false);
        self.session.set(None);
    }

    pub fn close(&self) {
        if !self.is_open() || !self.shown.peek() {
            return;
        }
        self.menu_open.set(false);
        self.shown.set(false);
        self.zoom_command.set(Some(false));
        let (session, shown) = (self.session.clone(), self.shown.clone());
        timer::set_timeout(FADE, move || {
            if !shown.peek() {
                session.set(None);
            }
        });
    }

    pub fn step(&self, delta: i32) {
        let Some(session) = self.session.peek() else {
            return;
        };
        let count = session.items.len() as i32;
        if count < 2 {
            return;
        }
        let next = (self.index.peek() as i32 + delta).rem_euclid(count) as usize;
        self.menu_open.set(false);
        self.zoomed.set(false);
        self.index.set(next);
    }

    pub fn escape(&self) -> bool {
        if !self.is_open() {
            return false;
        }
        if self.menu_open.peek() {
            self.menu_open.set(false);
        } else {
            self.close();
        }
        true
    }

    fn set_zoomed(&self, zoomed: bool) {
        self.zoomed.set(zoomed);
        self.zoom_command.set(Some(zoomed));
    }
}

struct ZoomState {
    scale: Spring,
    x: Spring,
    y: Spring,
    zoomed: bool,
    dragging: bool,

    moved: bool,

    pressed: bool,
    press: (f32, f32),
    last: (f32, f32),
    last_frame: Option<Instant>,
}

struct ZoomImageWidget {
    state: MediaViewerState,
    images: ImageCache,
    item: MediaItem,
    multiple: bool,
    zoom: RefCell<ZoomState>,
}

impl ZoomImageWidget {
    fn new(state: &MediaViewerState, images: &ImageCache, item: MediaItem, multiple: bool) -> Self {
        let start = if state.opening.replace(false) {
            OPEN_SCALE
        } else {
            1.0
        };
        let mut scale = Spring::new(start);
        scale.start(1.0);
        Self {
            state: state.clone(),
            images: images.clone(),
            item,
            multiple,
            zoom: RefCell::new(ZoomState {
                scale,
                x: Spring::new(0.0),
                y: Spring::new(0.0),
                zoomed: false,
                dragging: false,
                moved: false,
                pressed: false,
                press: (0.0, 0.0),
                last: (0.0, 0.0),
                last_frame: None,
            }),
        }
    }

    fn decoded(&self) -> Option<Image> {
        if let (Some(url), Some(frames)) = (&self.item.video_url, &self.state.video_frame) {
            if let Some(frame) = frames(url) {
                return Some(frame);
            }
        }
        self.images.get_or_decode_image(&self.item.url)
    }

    fn fitted(&self, decoded: Option<&Image>) -> (f32, f32) {
        let (w, h) = match (self.item.width, self.item.height) {
            (Some(w), Some(h)) if w > 0 && h > 0 => (w as f32, h as f32),
            _ => decoded.map_or((0.0, 0.0), |i| (i.width() as f32, i.height() as f32)),
        };
        fit_dims(self.multiple, w, h, app::current_window_size())
    }

    fn screen_rect(&self, rect: AbsoluteRect) -> Rect {
        let zoom = self.zoom.borrow();
        let (w, h) = self.fitted(self.decoded().as_ref());
        let (cx, cy) = (
            rect.x + rect.width / 2.0 + zoom.x.value,
            rect.y + rect.height / 2.0 + zoom.y.value,
        );
        let (hw, hh) = (w * zoom.scale.value / 2.0, h * zoom.scale.value / 2.0);
        Rect::new(cx - hw, cy - hh, cx + hw, cy + hh)
    }

    fn contains(&self, rect: AbsoluteRect, x: f32, y: f32) -> bool {
        let r = self.screen_rect(rect);
        x >= r.left && x <= r.right && y >= r.top && y <= r.bottom
    }

    fn pan(&self, delta: (f32, f32)) {
        let (w, h) = self.fitted(self.decoded().as_ref());
        let mut zoom = self.zoom.borrow_mut();
        let goal = zoom.scale.goal;
        let (x, y) = clamp_pan(
            (w * goal, h * goal),
            app::current_window_size(),
            (zoom.x.goal, zoom.y.goal),
            delta,
        );
        zoom.x.set(x);
        zoom.y.set(y);
        drop(zoom);
        requiem_ui_paint::request_repaint();
    }
}

impl Widget for ZoomImageWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        if let Some(zoomed) = self.state.zoom_command.take() {
            let mut zoom = self.zoom.borrow_mut();
            zoom.zoomed = zoomed;
            zoom.scale.start(if zoomed { ZOOM_SCALE } else { 1.0 });
            if !zoomed {
                zoom.x.start(0.0);
                zoom.y.start(0.0);
            }
        }

        if !self.state.shown.peek() && self.state.is_open() {
            let mut zoom = self.zoom.borrow_mut();
            zoom.scale.start(OPEN_SCALE);
            zoom.x.start(0.0);
            zoom.y.start(0.0);
        }
        let decoded = self.decoded();
        let placeholder = if decoded.is_none() {
            self.item
                .placeholder
                .as_deref()
                .and_then(|hash| self.images.get_or_decode_placeholder(hash))
        } else {
            None
        };
        let image = decoded.clone().or(placeholder);
        let (w, h) = self.fitted(decoded.as_ref());

        let (scale, x, y, moving) = {
            let mut zoom = self.zoom.borrow_mut();
            let now = Instant::now();
            let dt = zoom
                .last_frame
                .map_or(1.0 / 60.0, |t| now.duration_since(t).as_secs_f32());
            zoom.last_frame = Some(now);
            zoom.scale.step(dt);
            zoom.x.step(dt);
            zoom.y.step(dt);
            (
                zoom.scale.value,
                zoom.x.value,
                zoom.y.value,
                !(zoom.scale.at_rest() && zoom.x.at_rest() && zoom.y.at_rest()),
            )
        };
        if moving {
            requiem_ui_paint::animate_region(
                canvas,
                Rect::from_xywh(rect.x, rect.y, rect.width, rect.height),
            );
        } else {
            self.zoom.borrow_mut().last_frame = None;
        }
        if self.item.video_url.is_some() && self.state.video_frame.is_some() {
            requiem_ui_paint::animate_region_on_wake(
                canvas,
                Rect::from_xywh(rect.x, rect.y, rect.width, rect.height),
            );
        }
        let Some(image) = image else { return };
        let (cx, cy) = (
            rect.x + rect.width / 2.0 + x,
            rect.y + rect.height / 2.0 + y,
        );
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        canvas.save();
        canvas.translate((cx, cy));
        canvas.scale((scale, scale));
        let dst = Rect::from_xywh(-w / 2.0, -h / 2.0, w, h);
        if moving {
            let sampling =
                primitives::sampling_for(canvas, image.width() as f32, w);
            canvas.draw_image_rect_with_sampling_options(&image, None, dst, sampling, &paint);
        } else {
            primitives::draw_image_filtered(canvas, &image, None, dst, &paint);
        }
        canvas.restore();
    }

    fn handle_event(&mut self, event: &UiEvent, rect: AbsoluteRect) -> EventResult {
        match event {
            UiEvent::MouseDown {
                x,
                y,
                button: MouseButton::Left,
                ..
            } => {
                let inside = self.contains(rect, *x, *y);
                let mut zoom = self.zoom.borrow_mut();
                zoom.pressed = inside;
                if inside && zoom.zoomed {
                    zoom.dragging = true;
                    zoom.moved = false;
                    zoom.press = (*x, *y);
                    zoom.last = (*x, *y);
                }
                if inside {
                    EventResult::Handled
                } else {
                    EventResult::Ignored
                }
            }
            UiEvent::MouseMoved { x, y } => {
                let delta = {
                    let mut zoom = self.zoom.borrow_mut();
                    if !zoom.dragging {
                        return EventResult::Ignored;
                    }
                    let delta = (x - zoom.last.0, y - zoom.last.1);
                    zoom.last = (*x, *y);

                    if (x - zoom.press.0).powi(2) + (y - zoom.press.1).powi(2) >= CLICK_SLOP_SQ {
                        zoom.moved = true;
                    }
                    delta
                };
                self.pan(delta);
                EventResult::Handled
            }
            UiEvent::MouseUp {
                x,
                y,
                button: MouseButton::Left,
            } => {
                let inside = self.contains(rect, *x, *y);
                let (zoomed, dragging, moved, pressed) = {
                    let mut zoom = self.zoom.borrow_mut();
                    let state = (zoom.zoomed, zoom.dragging, zoom.moved, zoom.pressed);
                    zoom.dragging = false;
                    zoom.pressed = false;
                    state
                };
                if !zoomed {
                    if !inside {
                        return EventResult::Ignored;
                    }

                    if !pressed {
                        return EventResult::Handled;
                    }

                    let window = app::current_window_size();
                    let (w, h) = self.fitted(self.decoded().as_ref());
                    let (tx, ty) = (x - window.0 / 2.0, y - window.1 / 2.0);
                    let (ox, oy) = clamp_pan(
                        (w * ZOOM_SCALE, h * ZOOM_SCALE),
                        window,
                        (0.0, 0.0),
                        (-tx * (ZOOM_SCALE - 1.0), -ty * (ZOOM_SCALE - 1.0)),
                    );
                    {
                        let mut zoom = self.zoom.borrow_mut();
                        zoom.zoomed = true;
                        zoom.scale.start(ZOOM_SCALE);
                        zoom.x.start(ox);
                        zoom.y.start(oy);
                    }
                    self.state.zoomed.set(true);
                    return EventResult::Handled;
                }
                if dragging && !moved {
                    {
                        let mut zoom = self.zoom.borrow_mut();
                        zoom.zoomed = false;
                        zoom.scale.start(1.0);
                        zoom.x.start(0.0);
                        zoom.y.start(0.0);
                    }
                    self.state.zoomed.set(false);
                }
                if inside || dragging {
                    EventResult::Handled
                } else {
                    EventResult::Ignored
                }
            }
            UiEvent::Scroll {
                x,
                y,
                delta_x,
                delta_y,
                ..
            } => {
                if !self.contains(rect, *x, *y) {
                    return EventResult::Ignored;
                }
                self.pan((*delta_x, *delta_y));
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }

    fn wants_pointer_capture(&self) -> bool {
        self.zoom.borrow().dragging
    }

    fn cursor_icon_at(&self, rect: AbsoluteRect, x: f32, y: f32) -> Option<CursorIcon> {
        if !self.contains(rect, x, y) {
            return None;
        }
        let zoom = self.zoom.borrow();

        Some(if zoom.dragging && zoom.moved {
            CursorIcon::Grabbing
        } else if zoom.zoomed {
            CursorIcon::ZoomOut
        } else {
            CursorIcon::ZoomIn
        })
    }
}

struct MenuProbe {
    left: Rc<Cell<bool>>,
}

impl Widget for MenuProbe {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        primitives::BoxWidget.paint(canvas, rect, style);
        let window = app::current_window_size();
        self.left.set(rect.x + rect.width + 4.0 + 188.0 > window.0);
    }
}

fn round_button(
    icon_path: impl IconPath,
    label: &str,
    positioned: &'static str,
    on_click: impl FnMut() + 'static,
) -> Node {
    let button = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![css::ROUND_BUTTON.to_string()])
        .on_click(on_click)
        .build()
        .child(
            div()
                .class(css::ROUND_BUTTON_INNER)
                .child(icon(icon_path, ui::TEXT_SECONDARY).class(css::ROUND_ICON)),
        );
    tooltip(
        button,
        TooltipProps::default()
            .text(label)
            .position(TooltipPosition::Bottom),
    )
    .class(positioned)
}

fn action_button(icon_path: impl IconPath, label: &str, on_click: impl FnMut() + 'static) -> Node {
    let button = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![css::ACTION_BUTTON.to_string()])
        .on_click(on_click)
        .build()
        .child(
            div()
                .class(css::ACTION_BUTTON_INNER)
                .child(icon(icon_path, ui::TEXT_SECONDARY).class(css::ACTION_ICON)),
        );
    tooltip(
        button,
        TooltipProps::default()
            .text(label)
            .position(TooltipPosition::Bottom),
    )
}

fn author_block(
    item: &MediaItem,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &config::BackendConfig,
) -> Option<Node> {
    let author = item.author.as_ref()?;
    let image = author
        .avatar_url
        .as_deref()
        .and_then(|url| images.get_or_decode_image(url));
    let _ = backend;
    let mut avatar_props = ui::user_avatar_props(author.id, &author.discriminator, fonts);
    if let Some(image) = image {
        avatar_props = avatar_props.image(image);
    }
    let avatar = avatar_props.build().class(css::AUTHOR_AVATAR);
    let name = TextProps::styled(author.username.clone(), TextVariant::TEXT_MD_MEDIUM)
        .line_height(22.0_f32)
        .color(TEXT_STRONG)
        .line_clamp(1_u32)
        .build()
        .class(css::AUTHOR_NAME);
    let mut column = div().class(css::AUTHOR).child(name);
    if let Some(timestamp) = &author.timestamp {
        column = column.child(
            TextProps::styled(timestamp.clone(), TextVariant::TEXT_XS_MEDIUM)
                .color(TEXT_SUBTLE)
                .build(),
        );
    }
    Some(div().class(css::AUTHOR_ROW).child(avatar).child(column))
}

fn is_http(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

fn menu_item(
    icon_path: Option<Icon>,
    label: &str,
    extra: Option<&'static str>,
    on_click: impl FnMut() + 'static,
) -> Node {
    let mut props = ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![css::MENU_ITEM.to_string()]);
    if let Some(extra) = extra {
        props = props.extra_classes(vec![css::MENU_ITEM.to_string(), extra.to_string()]);
    }
    let mut item = props.on_click(on_click).build();
    if let Some(path) = icon_path {
        item = item.child(icon(path, ui::TEXT_SECONDARY).class(css::MENU_ICON));
    }
    item.child(
        TextProps::styled(label.to_string(), TextVariant::TEXT_SM_MEDIUM)
            .color(TEXT_STRONG)
            .build(),
    )
}

fn more_menu(state: &MediaViewerState, item: &MediaItem, images: &ImageCache) -> Node {
    let mut menu = Node::new(
        "div",
        MenuProbe {
            left: state.details_left.clone(),
        },
    )
    .class(css::MENU);

    let mut first_group: Vec<Node> = Vec::new();

    let can_copy_image = item.video_url.is_none();
    if can_copy_image {
        let (images, url, state) = (images.clone(), item.url.clone(), state.clone());
        first_group.push(menu_item(
            Some(COPY),
            &requiem_i18n::t!("media-viewer-copy-image"),
            None,
            move || {
                state.menu_open.set(false);
                if images
                    .get_or_decode_image(&url)
                    .is_some_and(|image| copy_image(&image))
                {
                    state.toasts.show(
                        &requiem_i18n::t!("media-viewer-image-copied"),
                        ToastKind::Success,
                    );
                } else {
                    state.toasts.show(
                        &requiem_i18n::t!("media-viewer-unable-to-copy-image"),
                        ToastKind::Failure,
                    );
                }
            },
        ));
    }
    if is_http(item.link()) {
        let (url, state) = (item.link().to_string(), state.clone());
        first_group.push(menu_item(
            Some(LINK),
            &requiem_i18n::t!("media-viewer-copy-media-link"),
            None,
            move || {
                state.menu_open.set(false);
                if clipboard::set_text(url.clone()) {
                    state.toasts.show(
                        &requiem_i18n::t!("media-viewer-link-copied"),
                        ToastKind::Success,
                    );
                } else {
                    state.toasts.show(
                        &requiem_i18n::t!("media-viewer-unable-to-copy-link"),
                        ToastKind::Failure,
                    );
                }
            },
        ));
    }
    let has_first = !first_group.is_empty();
    menu = menu.children(first_group);

    if item.is_attachment {
        if has_first {
            menu = menu.child(div().class(css::MENU_SEPARATOR));
        }

        let hover = state.details_open.clone();
        let entry = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![css::MENU_ITEM.to_string()])
            .on_click(|| {})
            .build()
            .child(
                TextProps::styled(
                    requiem_i18n::t!("media-viewer-view-details").to_string(),
                    TextVariant::TEXT_SM_MEDIUM,
                )
                .color(TEXT_STRONG)
                .build()
                .class(css::MENU_LABEL_GROW),
            )
            .child(
                icon(CHEVRON_SMALL_RIGHT, ui::TEXT_SECONDARY).class(css::MENU_ICON_TRAILING),
            );
        let submenu_state = state.clone();
        let item = item.clone();
        let submenu = Node::reactive(move || {
            if !submenu_state.details_open.get() {
                return div();
            }
            let copy = |value: String| {
                let toasts = submenu_state.toasts.clone();
                move || {
                    if clipboard::set_text(value.clone()) {
                        toasts.show(
                            &requiem_i18n::t!("media-viewer-copied-to-clipboard"),
                            ToastKind::Success,
                        );
                    }
                }
            };
            let mut panel = div().class(css::SUBMENU);
            if let Some(name) = &item.filename {
                panel = panel.child(detail_row(
                    &requiem_i18n::t!("media-viewer-filename"),
                    name,
                    copy(name.clone()),
                ));
            }
            if let (Some(w), Some(h)) = (item.width, item.height) {
                let size = item
                    .size
                    .map(format::file_size)
                    .unwrap_or_default();
                let value = format!("{w}x{h} ({size})");
                panel = panel.child(detail_row(
                    &requiem_i18n::t!("media-viewer-size"),
                    &value,
                    copy(value.clone()),
                ));
            }
            if let Some(alt) = item.alt.as_ref().filter(|a| !a.is_empty()) {
                panel = panel.child(detail_row(
                    &requiem_i18n::t!("media-viewer-description-alt-text"),
                    alt,
                    copy(alt.clone()),
                ));
            }

            let host = div().class(css::SUBMENU_HOST);
            div()
                .class(if submenu_state.details_left.get() {
                    css::SUBMENU_HOST_LEFT
                } else {
                    css::SUBMENU_HOST_RIGHT
                })
                .child(host.child(panel))
        });

        let wrap = Node::new(
            "div",
            ClickAreaWidget::new().with_on_hover_path(move |hovered| {
                if hover.peek() != hovered {
                    hover.set(hovered);
                }
            }),
        )
        .class(css::MENU_ITEM_WRAP)
        .child(entry)
        .child(submenu);
        menu = menu.child(wrap);
    }
    menu
}

fn detail_row(label: &str, value: &str, on_click: impl FnMut() + 'static) -> Node {
    ButtonProps::default()
        .variant(ButtonVariant::Bare)
        .extra_classes(vec![css::DETAIL_ROW.to_string()])
        .on_click(on_click)
        .build()
        .child(
            TextProps::styled(label.to_string(), TextVariant::TEXT_SM_MEDIUM)
                .color(TEXT_STRONG)
                .build(),
        )
        .child(
            TextProps::styled(value.to_string(), TextVariant::TEXT_XS_NORMAL)
                .color(TEXT_SUBTLE)
                .line_clamp(2_u32)
                .build(),
        )
}

pub fn copy_image(image: &Image) -> bool {
    let (width, height) = (image.width() as usize, image.height() as usize);
    let info = skia_safe::ImageInfo::new(
        (width as i32, height as i32),
        skia_safe::ColorType::RGBA8888,
        skia_safe::AlphaType::Unpremul,
        None,
    );
    let mut pixels = vec![0u8; width * height * 4];
    image.read_pixels(
        &info,
        &mut pixels,
        width * 4,
        (0, 0),
        CachingHint::Disallow,
    ) && clipboard::set_image(width, height, pixels)
}

fn action_bar(state: &MediaViewerState, item: &MediaItem, images: &ImageCache) -> Node {
    let mut bar = div().class(css::ACTION_BAR);

    let zoom_state = state.clone();
    bar = bar.child(Node::reactive(move || {
        let zoomed = zoom_state.zoomed.get();
        let click_state = zoom_state.clone();
        action_button(
            if zoomed {
                MAGNIFYING_GLASS_MINUS
            } else {
                MAGNIFYING_GLASS_PLUS
            },
            &if zoomed {
                requiem_i18n::t!("media-viewer-zoom-out")
            } else {
                requiem_i18n::t!("media-viewer-zoom-in")
            },
            move || {
                let now = !click_state.zoomed.peek();
                click_state.set_zoomed(now);
            },
        )
    }));

    if item.zoom_only {
        return bar;
    }

    if let Some(on_save) = state.on_save.clone() {
        let default_name = if item.video_url.is_some() {
            "video.mp4"
        } else {
            "image.png"
        };
        let (images, url, filename) = (
            images.clone(),
            item.link().to_string(),
            item.filename
                .clone()
                .unwrap_or_else(|| default_name.to_string()),
        );
        let toasts = state.toasts.clone();
        bar = bar.child(action_button(
            DOWNLOAD,
            &requiem_i18n::t!("media-viewer-save-image"),
            move || match images.get_or_request(&url) {
                Some(bytes) => on_save(bytes.to_vec(), filename.clone()),
                None => toasts.show(
                    &requiem_i18n::t!("media-viewer-unable-to-save-image"),
                    ToastKind::Failure,
                ),
            },
        ));
    }
    if is_http(item.link()) {
        let url = item.link().to_string();
        bar = bar.child(action_button(
            WINDOW_LAUNCH,
            &requiem_i18n::t!("media-viewer-open-in-browser"),
            move || utils_link::open_url(&url),
        ));
    }

    let more_state = state.clone();
    let more_button = action_button(
        MORE_HORIZONTAL,
        &requiem_i18n::t!("media-viewer-more"),
        move || {
            let open = !more_state.menu_open.peek();
            more_state.details_open.set(false);
            more_state.menu_open.set(open);
        },
    );
    let menu_state = state.clone();
    let (menu_item_data, menu_images) = (item.clone(), images.clone());
    let menu = Node::reactive(move || {
        if !menu_state.menu_open.get() {
            return div();
        }
        div()
            .class(css::MENU_ANCHOR)
            .child(more_menu(&menu_state, &menu_item_data, &menu_images))
    });
    bar.child(div().class(css::MORE_WRAP).child(more_button).child(menu))
}

fn gallery(
    state: &MediaViewerState,
    items: &[MediaItem],
    index: usize,
    images: &ImageCache,
) -> Node {
    let mut row = StackProps::horizontal().gap(2).full_width(false).build();
    let last = items.len().saturating_sub(1);
    for (i, item) in items.iter().enumerate() {
        let active = i == index;
        let thumbnail = images.get_or_decode_image(&item.url).or_else(|| {
            item.placeholder
                .as_deref()
                .and_then(|hash| images.get_or_decode_placeholder(hash))
        });
        let (iw, ih) = thumbnail
            .as_ref()
            .map_or((40.0, 40.0), |t| (t.width() as f32, t.height() as f32));
        let (w, h) = scale_to_fit(iw, ih, 40.0, 40.0);
        let click_state = state.clone();
        let mut cell = div().class(css::GALLERY_ITEM);
        if i == 0 {
            cell = cell.class(css::GALLERY_ITEM_FIRST);
        }
        if i == last {
            cell = cell.class(css::GALLERY_ITEM_LAST);
        }
        if let Some(thumbnail) = thumbnail {
            cell = cell.child(
                Node::new(
                    "div",
                    ImageWidget::new(Some(thumbnail)).with_intrinsic_size(w.max(1.0), h.max(1.0)),
                )
                .class(css::GALLERY_IMAGE),
            );
        }
        if !active {
            cell = cell.child(div().class(css::GALLERY_OVERLAY));
        }
        let button = ButtonProps::default()
            .variant(ButtonVariant::Bare)
            .extra_classes(vec![css::GALLERY_BUTTON.to_string()])
            .on_click(move || {
                click_state.menu_open.set(false);
                click_state.zoomed.set(false);
                click_state.index.set(i);
            });
        row = row.child(button.build().child(cell));
    }
    row
}

fn build_view(
    state: &MediaViewerState,
    session: &ViewerSession,
    index: usize,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &config::BackendConfig,
) -> Node {
    let items = &session.items;
    let index = index.min(items.len() - 1);
    let item = items[index].clone();
    let multiple = items.len() > 1;

    let fade = state.shown.clone();
    let close_state = state.clone();
    let scrim = Node::new(
        "div",
        ClickAreaWidget::new().with_on_click({
            let s = close_state.clone();
            move || s.close()
        }),
    )
    .class(css::SCRIM)
    .bind_class(fade.clone(), css::SCRIM_SHOWN, "");

    let image = Node::new(
        "div",
        ZoomImageWidget::new(state, images, item.clone(), multiple),
    )
    .class(css::MEDIA);

    let zoomed = state.zoomed.clone();
    let fade_with_zoom = |node: Node| {
        node.class(css::CHROME_FADE)
            .bind_class(zoomed.clone(), css::CHROME_HIDDEN, "")
    };
    let mut top = div().class(css::TOP_BAR);
    if let Some(author) = author_block(&item, fonts, images, backend) {
        top = top.child(author);
    }
    let top = fade_with_zoom(
        top.child(
            Node::new("div", ClickAreaWidget::new())
                .class(css::ACTION_BAR_HOLDER)
                .child(action_bar(state, &item, images)),
        ),
    );
    let mut chrome: Vec<Node> = vec![top];
    if multiple {
        let (prev_state, next_state) = (state.clone(), state.clone());
        chrome.push(fade_with_zoom(round_button(
            ARROW_LARGE_LEFT,
            &requiem_i18n::t!("media-viewer-previous"),
            css::NAV_PREV,
            move || prev_state.step(-1),
        )));
        chrome.push(fade_with_zoom(round_button(
            ARROW_LARGE_RIGHT,
            &requiem_i18n::t!("media-viewer-next"),
            css::NAV_NEXT,
            move || next_state.step(1),
        )));
        chrome.push(fade_with_zoom(
            Node::new("div", ClickAreaWidget::new())
                .class(css::GALLERY_HOLDER)
                .child(gallery(state, items, index, images)),
        ));
    }

    let close_button = {
        let s = state.clone();
        round_button(
            X_SMALL,
            &requiem_i18n::t!("media-viewer-close"),
            css::CLOSE_BUTTON,
            move || s.close(),
        )
    };

    let modal = Node::new(
        "div",
        ClickAreaWidget::new().with_on_click({
            let s = state.clone();
            move || s.close()
        }),
    )
    .class(css::MODAL)
    .bind_class(fade, css::MODAL_SHOWN, "")
    .child(image)
    .children(chrome)
    .child(close_button);

    div().class(css::VIEWER).child(scrim).child(modal)
}

pub fn mount(
    tree: &mut WidgetTree,
    parent: WidgetId,
    state: &MediaViewerState,
    fonts: &Fonts,
    images: &ImageCache,
    backend: &config::BackendConfig,
) -> Vec<Effect> {
    let was_open = Rc::new(Cell::new(false));
    let session = state.session.clone();
    let blur = requiem_ui_signal::create_effect(
        move |tree| {
            let open = session.get().is_some();
            if open && !was_open.get() {
                if let Some(focused) = tree.focused.take() {
                    if let Some(node) = tree.nodes.get_mut(focused) {
                        node.widget.on_focus_change(false);
                    }
                }
            }
            was_open.set(open);
        },
        tree,
    );
    let (state, fonts, images, backend) = (
        state.clone(),
        fonts.clone(),
        images.clone(),
        backend.clone(),
    );
    let view = section(tree, parent, move || {
        let Some(session) = state.session.get() else {
            return div();
        };
        let index = state.index.get();
        build_view(&state, &session, index, &fonts, &images, &backend)
    });
    vec![blur, view]
}

#[cfg(test)]
mod tests {
    use requiem_ui::font_system::TextFonts;
    use requiem_ui::font_system;
    use requiem_ui::layout;
    use requiem_ui::paint as requiem_ui_paint;
    use requiem_ui::signal as requiem_ui_signal;
    use requiem_ui::style::ComputedStyle;
    use requiem_ui::style::cascade;
    use requiem_ui::style::parser;
    use requiem_ui::theme;
    use requiem_ui::widget::primitives::BoxWidget;
    use skia_safe::font_style::Weight;
    use skia_safe::surfaces;
    use std::env;
    use std::fs;
    use std::io::Read;
    use std::io::Write;
    use std::net::TcpListener;
    use std::path::Path;
    use std::thread;
    use super::*;
    use taffy::AvailableSpace;
    use taffy::Size;
    use tokio::runtime::Runtime;

    #[test]
    fn opening_takes_focus_off_the_composer() {
        let runtime = Runtime::new().unwrap();
        let images = ImageCache::new(runtime.handle().clone(), wreq::Client::new(), || {});
        let fonts = Fonts {
            typeface: theme::make_typeface(Weight::MEDIUM),
            text_fonts: TextFonts::new(1024 * 1024),
        };
        font_system::install_ambient(fonts.text_fonts.clone());
        let backend = config::BackendConfig {
            rest_base_url: String::new(),
            gateway_url: String::new(),
            cdn_base_url: String::new(),
        };
        let state = MediaViewerState::inert();
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let root = tree.root;
        let composer = tree.add_child(root, "div", Box::new(BoxWidget), ComputedStyle::default());
        tree.focused = Some(composer);
        let _effects = mount(&mut tree, root, &state, &fonts, &images, &backend);
        assert_eq!(
            tree.focused,
            Some(composer),
            "closed viewer leaves focus alone"
        );
        let item = MediaItem {
            video_url: None,
            url: "http://127.0.0.1:1/x.png".into(),
            width: Some(10),
            height: Some(10),
            placeholder: None,
            filename: None,
            size: None,
            alt: None,
            author: None,
            is_attachment: false,
            zoom_only: false,
        };
        state.open(vec![item], 0);
        requiem_ui_signal::flush(&mut tree);
        assert_eq!(tree.focused, None);
    }

    #[test]
    fn scale_to_fit_never_upscales_and_keeps_aspect() {
        assert_eq!(scale_to_fit(100.0, 50.0, 400.0, 400.0), (100.0, 50.0));
        assert_eq!(
            scale_to_fit(2000.0, 1000.0, 1000.0, 1000.0),
            (1000.0, 500.0)
        );
        assert_eq!(scale_to_fit(1000.0, 2000.0, 1000.0, 500.0), (250.0, 500.0));
    }

    #[test]
    fn fit_matches_the_clients_margins() {
        assert_eq!(
            fit_dims(false, 4000.0, 3000.0, (1400.0, 900.0)),
            (965.0, 724.0)
        );

        assert_eq!(
            fit_dims(false, 300.0, 200.0, (1400.0, 900.0)),
            (300.0, 200.0)
        );

        let (w, _) = fit_dims(true, 4000.0, 1000.0, (1400.0, 900.0));
        assert!(w <= 1400.0 - 152.0, "{w}");
    }

    #[test]
    fn pan_clamps_at_the_edges() {
        let offset = clamp_pan((2000.0, 500.0), (1000.0, 800.0), (0.0, 0.0), (900.0, 50.0));
        assert_eq!(offset.0, 500.0);
        assert_eq!(offset.1, 0.0, "it fits vertically, so no vertical pan");
        let offset = clamp_pan((2000.0, 500.0), (1000.0, 800.0), (0.0, 0.0), (-900.0, 0.0));
        assert_eq!(offset.0, -500.0);

        assert_eq!(
            clamp_pan((2000.0, 500.0), (1000.0, 800.0), (100.0, 0.0), (-30.0, 0.0)).0,
            70.0
        );
    }

    #[test]
    fn spring_settles_on_its_goal() {
        let mut spring = Spring::new(0.9);
        spring.start(1.0);
        for _ in 0..120 {
            spring.step(1.0 / 60.0);
        }
        assert!(spring.at_rest());
        assert_eq!(spring.value, 1.0);

        let mut spring = Spring::new(1.0);
        spring.start(2.5);
        let mut peak = 0.0_f32;
        for _ in 0..240 {
            spring.step(1.0 / 60.0);
            peak = peak.max(spring.value);
        }
        assert!(peak > 2.5 && peak < 2.6, "peak {peak}");
    }

    #[test]
    fn stepping_wraps_and_zooms_out() {
        timer::run_due();
        let state = MediaViewerState::inert();
        let item = |url: &str| MediaItem {
            video_url: None,
            url: url.into(),
            width: Some(10),
            height: Some(10),
            placeholder: None,
            filename: None,
            size: None,
            alt: None,
            author: None,
            is_attachment: true,
            zoom_only: false,
        };
        state.open(vec![item("a"), item("b"), item("c")], 0);
        state.step(-1);
        assert_eq!(
            state.index.peek(),
            2,
            "previous from the first wraps to the last"
        );
        state.step(1);
        assert_eq!(state.index.peek(), 0);
        state.zoomed.set(true);
        state.step(1);
        assert!(!state.zoomed.peek(), "changing image zooms back out");
        assert!(state.is_open());
    }

    fn serve_png(body: Vec<u8>) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let handle = thread::spawn(move || {
            listener.set_nonblocking(false).unwrap();
            for stream in listener.incoming().take(16) {
                let Ok(mut stream) = stream else { continue };
                let mut buffer = [0u8; 2048];
                let _ = stream.read(&mut buffer);
                let head = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: image/png\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&body);
            }
        });
        (base, handle)
    }

    fn test_png(width: i32, height: i32, tint: u8) -> Vec<u8> {
        let mut surface = surfaces::raster_n32_premul((width, height)).unwrap();
        let canvas = surface.canvas();
        canvas.clear(skia_safe::Color::from_rgb(40, 60, 90));
        for i in 0..24 {
            let mut paint = Paint::default();
            paint.set_color(skia_safe::Color::from_rgb(
                tint.wrapping_add((i * 9) as u8),
                120,
                (255 - i * 8) as u8,
            ));
            canvas.draw_rect(
                Rect::from_xywh(
                    i as f32 * width as f32 / 24.0,
                    (i % 5) as f32 * height as f32 / 5.0,
                    width as f32 / 24.0,
                    height as f32 / 3.0,
                ),
                &paint,
            );
        }
        surface
            .image_snapshot()
            .encode(None, skia_safe::EncodedImageFormat::PNG, 100)
            .unwrap()
            .as_bytes()
            .to_vec()
    }

    #[test]
    fn renders_the_viewer() {
        let Ok(out) = env::var("VIEWER_PREVIEW_DIR") else {
            return;
        };
        let (base, _server) = serve_png(test_png(1600, 1000, 30));
        let runtime = Runtime::new().unwrap();
        let images = ImageCache::new(runtime.handle().clone(), wreq::Client::new(), || {});
        let fonts = Fonts {
            typeface: theme::make_typeface(Weight::MEDIUM),
            text_fonts: TextFonts::new(16 * 1024 * 1024),
        };
        font_system::install_ambient(fonts.text_fonts.clone());
        let backend = config::BackendConfig {
            rest_base_url: String::new(),
            gateway_url: String::new(),
            cdn_base_url: String::new(),
        };
        let author = ViewerAuthor {
            id: Id::new(7),
            discriminator: "0".into(),
            username: "Wumpus".into(),
            avatar_url: None,
            timestamp: Some("3:29 PM".into()),
        };
        let item = |name: &str| MediaItem {
            video_url: None,
            url: format!("{base}/{name}.png"),
            width: Some(1600),
            height: Some(1000),
            placeholder: None,
            filename: Some(format!("{name}.png")),
            size: Some(523_000),
            alt: Some("A test card".into()),
            author: Some(author.clone()),
            is_attachment: true,
            zoom_only: false,
        };

        app::set_window_size(1400.0, 900.0);
        let state = MediaViewerState::new(Some(Rc::new(|_, _| {})));
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let root = tree.root;
        tree.nodes[root].classes.push("root".to_string());
        let _effects = mount(&mut tree, root, &state, &fonts, &images, &backend);
        let theme = parser::parse_stylesheet(&format!(
            "{}\n{}\n.root {{ position: relative; width: 1400px; height: 900px; background-color: #242429; }}",
            requiem_ui::BUNDLED_CSS,
            ui::THEME_CSS
        ));
        let size = Size {
            width: AvailableSpace::Definite(1400.0),
            height: AvailableSpace::Definite(900.0),
        };
        let render = |tree: &mut WidgetTree, name: &str| {
            requiem_ui_signal::flush(tree);
            cascade::apply_stylesheet(tree, &theme);
            layout::compute_layout(tree, size);
            let mut surface = surfaces::raster_n32_premul((1400, 900)).unwrap();
            requiem_ui_paint::paint(tree, surface.canvas());
            let png = surface
                .image_snapshot()
                .encode(None, skia_safe::EncodedImageFormat::PNG, 100)
                .unwrap();
            fs::write(Path::new(&out).join(name), png.as_bytes()).unwrap();
        };
        let settle = |tree: &mut WidgetTree| {
            for _ in 0..40 {
                thread::sleep(Duration::from_millis(25));
                timer::run_due();
                requiem_ui_signal::flush(tree);
            }
        };

        state.open(vec![item("a")], 0);
        render(&mut tree, "viewer_single_loading.png");
        settle(&mut tree);
        render(&mut tree, "viewer_single_start.png");
        settle(&mut tree);
        render(&mut tree, "viewer_single.png");

        requiem_ui_paint::dispatch_event(
            &mut tree,
            &UiEvent::MouseMoved {
                x: 1000.0,
                y: 400.0,
            },
        );
        requiem_ui_paint::dispatch_event(
            &mut tree,
            &UiEvent::MouseDown {
                x: 1000.0,
                y: 400.0,
                button: MouseButton::Left,
                shift: false,
            },
        );
        requiem_ui_paint::dispatch_event(
            &mut tree,
            &UiEvent::MouseUp {
                x: 1000.0,
                y: 400.0,
                button: MouseButton::Left,
            },
        );
        requiem_ui_signal::flush(&mut tree);
        assert!(state.zoomed.peek(), "a click on the image zooms in");
        for _ in 0..30 {
            thread::sleep(Duration::from_millis(20));
            render(&mut tree, "viewer_zoomed.png");
        }

        state.close();
        settle(&mut tree);
        state.open(vec![item("a"), item("b"), item("c")], 1);
        settle(&mut tree);
        settle(&mut tree);
        render(&mut tree, "viewer_gallery.png");
    }
}
