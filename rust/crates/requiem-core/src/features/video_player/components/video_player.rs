#![cfg(feature = "video")]

use crate::images;
use crate::ui::classes::video_player;
use crate::utils::format;
use requiem_ui::app;
use requiem_ui::colors::{BRAND, SURFACE_HIGH, TEXT_STRONG};
use requiem_ui::icons::IconPath;
use requiem_ui::paint as requiem_ui_paint;
use requiem_ui::theme;
use requiem_ui::widget::primitives;
use skia_safe::font_style::Weight;
use skia_safe::images as skia_safe_images;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

use requiem_ui::components::div;
use requiem_ui::event::{CursorIcon, EventResult, MouseButton, UiEvent};
use requiem_ui::node::Node;
use requiem_ui::signal::Signal;
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::{AbsoluteRect, Widget};
use skia_safe::{Canvas, Color, Contains, Font, Paint, Path, Point, RRect, Rect};

use crate::video::{VideoCache, VideoPlayer};
use requiem_ui::icons::{
    FULLSCREEN_ENTER, FULLSCREEN_EXIT, PAUSE, PLAY, REPLAY, VOICE_LOW, VOICE_NORMAL, VOICE_X,
};

const BAR_VISIBLE_H: f32 = 32.0;
const BAR_TOTAL_H: f32 = 42.0;
const BUTTON: f32 = 32.0;
const BUTTON_GAP: f32 = 8.0;
const ICON_PAD: f32 = 4.0;
const TIME_MARGIN: f32 = 4.0;
const TIME_SEPARATOR_MARGIN: f32 = 2.0;
const TIME_FONT_SIZE: f32 = 12.0;
const SEEK_INSET: f32 = 7.0;
const TRACK_H: f32 = 6.0;
const FAKE_EDGE: f32 = 3.0;
const GRABBER_D: f32 = 10.0;
const BUBBLE_H: f32 = 18.0;
const BUBBLE_PAD_X: f32 = 8.0;
const BUBBLE_OFFSET_Y: f32 = 28.0;
const BUBBLE_ARROW: f32 = 5.0;
const MIN_WIDTH_FOR_TIME: f32 = 250.0;
const VOLUME_PILL_W: f32 = 14.0;
const VOLUME_PILL_H: f32 = 88.0;
const VOLUME_PILL_INSET_X: f32 = 8.0;
const VOLUME_PILL_GAP: f32 = 4.0;
const VOLUME_TRACK_INSET_Y: f32 = 8.0;
const VOLUME_HOVER_W: f32 = 54.0;
const VOLUME_HOVER_H: f32 = 140.0;
const CENTER_BUTTON_D: f32 = 48.0;
const CENTER_ICON: f32 = 24.0;

const CONTROLS_TIMEOUT: Duration = Duration::from_millis(3000);
const CONTROLS_TIMEOUT_FULLSCREEN: Duration = Duration::from_millis(1000);
const VOLUME_HOVER_GRACE: Duration = Duration::from_millis(150);
const POP_DURATION: Duration = Duration::from_millis(400);
const GRABBER_TRANSITION_SECS: f32 = 0.25;

const VOLUME_CURVE_EXPONENT: f32 = 2.8;

const BAR_BG: Color = Color::from_argb(153, 0, 0, 0);
const CONTROL_ICON_HOVER: Color = Color::from_argb(255, 255, 255, 255);
const TRACK: Color = Color::from_argb(77, 202, 202, 205);
const BRAND_560: Color = Color::from_argb(255, 70, 84, 192);
const PREVIEW: Color = Color::from_argb(77, 255, 255, 255);
const VOLUME_PILL_BG: Color = Color::from_argb(173, 0, 0, 0);

thread_local! {

    static UI_STATES: RefCell<HashMap<String, Rc<SharedUi>>> = RefCell::new(HashMap::new());
    static FONTS: (skia_safe::Typeface, skia_safe::Typeface) = (
        theme::make_mono_typeface(Weight::NORMAL),
        theme::make_typeface(Weight::SEMI_BOLD),
    );
}

thread_local! {

    static VOLUME: Cell<f32> = const { Cell::new(1.0) };
}

#[derive(Default)]
struct ViewState {
    last_move: Cell<Option<Instant>>,

    controls_hidden: Cell<f32>,
}

#[derive(Default)]
struct SharedUi {
    embedded: ViewState,
    fullscreen: ViewState,
    has_clicked_play: Cell<bool>,

    wants_playing: Cell<bool>,
    muted: Cell<bool>,
}

fn shared_ui(url: &str) -> Rc<SharedUi> {
    UI_STATES.with(|m| m.borrow_mut().entry(url.to_string()).or_default().clone())
}

fn perceptual_to_amplitude(p: f32) -> f32 {
    p.clamp(0.0, 1.0).powf(VOLUME_CURVE_EXPONENT)
}

#[derive(Clone, Default)]
pub struct Poster {
    placeholder: Option<skia_safe::Image>,
    source: Option<(images::ImageCache, String)>,
    resolved: Rc<RefCell<Option<skia_safe::Image>>>,
}

impl Poster {
    pub fn new(
        images: &images::ImageCache,
        url: Option<String>,
        placeholder: Option<skia_safe::Image>,
    ) -> Self {
        Self {
            placeholder,
            source: url.map(|url| (images.clone(), url)),
            resolved: Rc::default(),
        }
    }

    fn image(&self) -> Option<skia_safe::Image> {
        if let Some(image) = self.resolved.borrow().clone() {
            return Some(image);
        }
        if let Some(image) = self
            .source
            .as_ref()
            .and_then(|(images, url)| images.get_or_decode_image(url))
        {
            *self.resolved.borrow_mut() = Some(image.clone());
            return Some(image);
        }
        self.placeholder.clone()
    }
}

#[derive(Clone)]
pub struct FullscreenVideoState {
    player: VideoPlayer,
    url: String,
    poster: Poster,
    intrinsic_w: u32,
    intrinsic_h: u32,
}

pub fn exit_fullscreen(fullscreen: &Signal<Option<FullscreenVideoState>>) {
    fullscreen.set(None);
    app::request_fullscreen(false);
}

enum Mode {
    Attachment {
        fullscreen: Signal<Option<FullscreenVideoState>>,
        videos: VideoCache,
    },
    Gifv {
        videos: VideoCache,
    },
    Fullscreen {
        fullscreen: Signal<Option<FullscreenVideoState>>,
    },
}

#[derive(Clone, Copy, PartialEq)]
enum Drag {
    None,
    Seek,
    Volume,
}

struct VideoWidget {
    url: String,
    poster: Poster,
    intrinsic_w: u32,
    intrinsic_h: u32,
    display_w: f32,
    display_h: f32,
    mode: Mode,
    ui: Rc<SharedUi>,

    player: RefCell<Option<VideoPlayer>>,
    decoded_frame: RefCell<Option<(u64, skia_safe::Image)>>,

    pointer: Cell<Option<(f32, f32)>>,
    pressed: Cell<bool>,
    drag: Cell<Drag>,
    drag_fraction: Cell<f32>,
    last_seek_fraction: Cell<f32>,
    seek_grabber_scale: Cell<f32>,
    volume_grabber_scale: Cell<f32>,
    volume_hover_left_at: Cell<Option<Instant>>,
    volume_hovered: Cell<bool>,
    pop: Cell<Option<(Instant, bool)>>,
    last_paint: Cell<Option<Instant>>,
}

struct Layout {
    bar: Rect,
    play_btn: Rect,
    time_origin: Option<(f32, f32)>,
    seek_hit: Rect,
    track: Rect,
    volume_btn: Rect,
    fullscreen_btn: Rect,
    volume_pill: Rect,
    volume_track: Rect,
    volume_hover: Rect,
}

fn time_font() -> Font {
    FONTS.with(|(mono, _)| Font::new(mono.clone(), TIME_FONT_SIZE))
}

fn bubble_font() -> Font {
    FONTS.with(|(_, semibold)| Font::new(semibold.clone(), TIME_FONT_SIZE))
}

fn time_strings(position: f64, duration: f64) -> (String, String) {
    let total = format::media_time(duration);
    let current = format!(
        "{:0>width$}",
        format::media_time(position),
        width = total.len()
    );
    (current, total)
}

fn draw_icon(
    canvas: &Canvas,
    icon: impl IconPath,
    color: Color,
    origin: (f32, f32),
    scale: f32,
    alpha: f32,
) {
    let Some(art) = icon.art() else { return };
    canvas.save();
    canvas.translate(origin);
    canvas.scale((scale, scale));
    art.draw(canvas, color, alpha);
    canvas.restore();
}

fn draw_button_icon(canvas: &Canvas, icon: impl IconPath, btn: Rect, hovered: bool, pressed: bool) {
    let color = if hovered {
        CONTROL_ICON_HOVER
    } else {
        TEXT_STRONG
    };
    let dy = if pressed { 1.0 } else { 0.0 };
    draw_icon(
        canvas,
        icon,
        color,
        (btn.left + ICON_PAD, btn.top + ICON_PAD + dy),
        1.0,
        1.0,
    );
}

fn fill_rrect(canvas: &Canvas, rect: Rect, radius: f32, color: Color) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color);
    canvas.draw_rrect(RRect::new_rect_xy(rect, radius, radius), &paint);
}

fn fill_circle(canvas: &Canvas, center: (f32, f32), radius: f32, color: Color) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color);
    canvas.draw_circle(center, radius, &paint);
}

fn approach(current: f32, target: f32, dt: f32, rate: f32) -> f32 {
    let step = dt * rate;
    if (target - current).abs() <= step {
        target
    } else {
        current + step * (target - current).signum()
    }
}

impl VideoWidget {
    fn new(
        url: &str,
        poster: Poster,
        intrinsic: (u32, u32),
        display: (f32, f32),
        mode: Mode,
        player: Option<VideoPlayer>,
    ) -> Self {
        if player.is_some() {
            if let Some(videos) = Self::cache_of(&mode) {
                videos.acquire(url);
            }
        }
        Self {
            url: url.to_string(),
            poster,
            intrinsic_w: intrinsic.0,
            intrinsic_h: intrinsic.1,
            display_w: display.0,
            display_h: display.1,
            mode,
            ui: shared_ui(url),
            player: RefCell::new(player),
            decoded_frame: RefCell::new(None),
            pointer: Cell::new(None),
            pressed: Cell::new(false),
            drag: Cell::new(Drag::None),
            drag_fraction: Cell::new(0.0),
            last_seek_fraction: Cell::new(-1.0),
            seek_grabber_scale: Cell::new(0.0),
            volume_grabber_scale: Cell::new(0.0),
            volume_hover_left_at: Cell::new(None),
            volume_hovered: Cell::new(false),
            pop: Cell::new(None),
            last_paint: Cell::new(None),
        }
    }

    fn cache_of(mode: &Mode) -> Option<&VideoCache> {
        match mode {
            Mode::Gifv { videos } => Some(videos),
            Mode::Attachment { .. } | Mode::Fullscreen { .. } => None,
        }
    }

    fn view(&self) -> &ViewState {
        if self.is_fullscreen() {
            &self.ui.fullscreen
        } else {
            &self.ui.embedded
        }
    }

    fn is_fullscreen(&self) -> bool {
        matches!(self.mode, Mode::Fullscreen { .. })
    }

    fn is_gifv(&self) -> bool {
        matches!(self.mode, Mode::Gifv { .. })
    }

    fn showing_poster(&self) -> bool {
        !self.is_fullscreen() && !self.ui.has_clicked_play.get()
    }

    fn player_for_playback(&self) -> Option<VideoPlayer> {
        if let Some(existing) = self.player.borrow().clone() {
            return Some(existing);
        }
        let player = match &self.mode {
            Mode::Attachment { videos, .. } => videos.get_or_open(&self.url, false),
            Mode::Gifv { videos } => videos.get_or_open(&self.url, true),
            Mode::Fullscreen { .. } => return None,
        };
        player.set_muted(self.ui.muted.get());
        if let Some(videos) = Self::cache_of(&self.mode) {
            videos.acquire(&self.url);
        }
        *self.player.borrow_mut() = Some(player.clone());
        Some(player)
    }

    fn duration(&self) -> f64 {
        self.player
            .borrow()
            .as_ref()
            .map_or(0.0, VideoPlayer::duration_secs)
    }

    fn has_ended(&self) -> bool {
        self.player
            .borrow()
            .as_ref()
            .is_some_and(|p| p.has_ended() && !p.is_playing())
    }

    fn set_playing(&self, playing: bool) {
        let Some(player) = self.player_for_playback() else {
            return;
        };
        let first_play = !self.ui.has_clicked_play.get();
        self.ui.has_clicked_play.set(true);
        if playing == self.ui.wants_playing.get() {
            return;
        }
        self.ui.wants_playing.set(playing);
        if playing {
            if player.has_ended() {
                player.seek(0.0);
            }
            player.play();
            self.view().last_move.set(Some(Instant::now()));
            if !first_play {
                self.pop.set(Some((Instant::now(), true)));
            }
        } else {
            player.pause();
            self.pop.set(Some((Instant::now(), false)));
        }
        requiem_ui_paint::request_repaint();
    }

    fn toggle_playing(&self) {
        self.set_playing(!self.ui.wants_playing.get());
    }

    fn controls_should_hide(&self, now: Instant) -> bool {
        if !self.ui.wants_playing.get() || self.drag.get() != Drag::None {
            return false;
        }
        let timeout = if self.is_fullscreen() {
            CONTROLS_TIMEOUT_FULLSCREEN
        } else {
            CONTROLS_TIMEOUT
        };
        self.view()
            .last_move
            .get()
            .is_none_or(|t| now.duration_since(t) > timeout)
    }

    fn controls_interactive(&self) -> bool {
        !self.showing_poster() && self.view().controls_hidden.get() < 0.99
    }

    fn layout(&self, rect: AbsoluteRect) -> Layout {
        let (left, right) = (rect.x, rect.x + rect.width);
        let bar_top =
            rect.y + rect.height - BAR_VISIBLE_H + self.view().controls_hidden.get() * BAR_TOTAL_H;
        let bar = Rect::from_xywh(left, bar_top, rect.width, BAR_TOTAL_H);
        let play_btn = Rect::from_xywh(left, bar_top, BUTTON, BUTTON);
        let mut x = play_btn.right + BUTTON_GAP;

        let time_origin = if rect.width > MIN_WIDTH_FOR_TIME {
            let origin = (x + TIME_MARGIN, bar_top + (BUTTON - TIME_FONT_SIZE) / 2.0);
            let (current, total) = time_strings(0.0, self.duration());
            let font = time_font();
            let w = font.measure_str(&current, None).0
                + font.measure_str("/", None).0
                + 2.0 * TIME_SEPARATOR_MARGIN
                + font.measure_str(&total, None).0;
            x += TIME_MARGIN + w + TIME_MARGIN;
            Some(origin)
        } else {
            None
        };

        let fullscreen_btn = Rect::from_xywh(right - BUTTON_GAP - BUTTON, bar_top, BUTTON, BUTTON);
        let volume_btn = Rect::from_xywh(fullscreen_btn.left - BUTTON, bar_top, BUTTON, BUTTON);
        let seek_left = x + SEEK_INSET;
        let seek_right = (volume_btn.left - SEEK_INSET).max(seek_left + 1.0);
        let seek_hit = Rect::from_ltrb(seek_left, bar_top, seek_right, bar_top + BUTTON);
        let track = Rect::from_xywh(
            seek_left,
            bar_top + (BUTTON - TRACK_H) / 2.0,
            seek_right - seek_left,
            TRACK_H,
        );

        let volume_pill = Rect::from_xywh(
            volume_btn.left + VOLUME_PILL_INSET_X,
            volume_btn.top - VOLUME_PILL_GAP - VOLUME_PILL_H,
            VOLUME_PILL_W,
            VOLUME_PILL_H,
        );
        let volume_track = Rect::from_ltrb(
            volume_pill.left + (VOLUME_PILL_W - TRACK_H) / 2.0,
            volume_pill.top + VOLUME_TRACK_INSET_Y,
            volume_pill.left + (VOLUME_PILL_W + TRACK_H) / 2.0,
            volume_pill.bottom - VOLUME_TRACK_INSET_Y,
        );
        let volume_hover = Rect::from_xywh(
            volume_pill.center_x() - VOLUME_HOVER_W / 2.0,
            volume_pill.bottom - VOLUME_HOVER_H,
            VOLUME_HOVER_W,
            VOLUME_HOVER_H,
        );

        Layout {
            bar,
            play_btn,
            time_origin,
            seek_hit,
            track,
            volume_btn,
            fullscreen_btn,
            volume_pill,
            volume_track,
            volume_hover,
        }
    }

    fn volume_slider_visible(&self, now: Instant) -> bool {
        self.drag.get() == Drag::Volume
            || self.volume_hovered.get()
            || self
                .volume_hover_left_at
                .get()
                .is_some_and(|t| now.duration_since(t) < VOLUME_HOVER_GRACE)
    }

    fn update_volume_hover(&self, layout: &Layout, point: Option<Point>) {
        let over = point.is_some_and(|p| {
            self.controls_interactive()
                && (layout.volume_btn.contains(p) || layout.volume_hover.contains(p))
        });
        if over != self.volume_hovered.get() {
            self.volume_hovered.set(over);
            self.volume_hover_left_at.set((!over).then(Instant::now));
        }
    }

    fn apply_volume_from_y(&self, y: f32, layout: &Layout) {
        let p = ((layout.volume_track.bottom - y) / layout.volume_track.height()).clamp(0.0, 1.0);
        VOLUME.with(|v| v.set(p));
        let muted = p == 0.0;
        if muted != self.ui.muted.get() {
            self.ui.muted.set(muted);
        }
        if let Some(player) = self.player.borrow().as_ref() {
            player.set_muted(muted);
        }
    }

    fn seek_fraction_at(x: f32, layout: &Layout) -> f32 {
        ((x - layout.track.left) / layout.track.width()).clamp(0.0, 1.0)
    }

    fn seek_to_fraction(&self, fraction: f32) {
        self.drag_fraction.set(fraction);
        if (fraction - self.last_seek_fraction.get()).abs() < 0.001 {
            return;
        }
        self.last_seek_fraction.set(fraction);
        let Some(player) = self.player.borrow().clone() else {
            return;
        };
        let duration = player.duration_secs();
        if duration > 0.0 {
            player.seek(fraction as f64 * duration);
        }
    }

    fn end_drag(&self) {
        match self.drag.replace(Drag::None) {
            Drag::Seek => {
                if self.ui.wants_playing.get() {
                    if let Some(player) = self.player.borrow().as_ref() {
                        player.play();
                    }
                }
                self.view().last_move.set(Some(Instant::now()));
            }
            Drag::Volume | Drag::None => {}
        }
        requiem_ui_paint::request_repaint();
    }

    fn toggle_fullscreen(&self) {
        match &self.mode {
            Mode::Attachment { fullscreen, .. } => {
                let Some(player) = self.player.borrow().clone() else {
                    return;
                };
                fullscreen.set(Some(FullscreenVideoState {
                    player,
                    url: self.url.clone(),
                    poster: self.poster.clone(),
                    intrinsic_w: self.intrinsic_w,
                    intrinsic_h: self.intrinsic_h,
                }));
                app::request_fullscreen(true);
            }
            Mode::Fullscreen { fullscreen } => exit_fullscreen(fullscreen),
            Mode::Gifv { .. } => {}
        }
    }

    fn current_image(&self) -> Option<skia_safe::Image> {
        let player = self.player.borrow().clone();
        let Some(player) = player else {
            return self.poster.image();
        };
        let version = player.frame_version();
        let mut cache = self.decoded_frame.borrow_mut();
        if cache.as_ref().is_none_or(|(v, _)| *v != version) {
            if let Some(frame) = player.current_frame() {
                let info = skia_safe::ImageInfo::new(
                    (frame.width as i32, frame.height as i32),
                    skia_safe::ColorType::RGBA8888,
                    skia_safe::AlphaType::Unpremul,
                    None,
                );
                if let Some(image) = skia_safe_images::raster_from_data(
                    &info,
                    skia_safe::Data::new_copy(&frame.rgba),
                    frame.width as usize * 4,
                ) {
                    *cache = Some((version, image));
                }
            }
        }
        cache
            .as_ref()
            .map(|(_, image)| image.clone())
            .or_else(|| self.poster.image())
    }

    fn letterbox(&self, rect: Rect, image: Option<&skia_safe::Image>) -> Rect {
        let (iw, ih) = image.map_or((self.intrinsic_w as f32, self.intrinsic_h as f32), |i| {
            (i.width() as f32, i.height() as f32)
        });
        let (iw, ih) = (iw.max(1.0), ih.max(1.0));
        let scale = (rect.width() / iw).min(rect.height() / ih);
        let (w, h) = (iw * scale, ih * scale);
        Rect::from_xywh(
            rect.left + (rect.width() - w) / 2.0,
            rect.top + (rect.height() - h) / 2.0,
            w,
            h,
        )
    }

    fn advance_animations(&self, now: Instant, layout_hint: Option<&Layout>) -> bool {
        let dt = self
            .last_paint
            .replace(Some(now))
            .map_or(0.0, |t| now.duration_since(t).as_secs_f32().min(0.1));
        let mut animating = false;

        if self.ui.wants_playing.get() && self.has_ended() {
            self.ui.wants_playing.set(false);
            self.pop.set(Some((now, false)));
        }

        let target = if self.controls_should_hide(now) {
            1.0
        } else {
            0.0
        };
        let hidden = self.view().controls_hidden.get();
        if !self.ui.wants_playing.get() {
            self.view().controls_hidden.set(target);
        } else if hidden != target {
            let next = hidden + (target - hidden) * (1.0 - (-dt * 14.0).exp());
            self.view()
                .controls_hidden
                .set(if (next - target).abs() < 0.002 {
                    target
                } else {
                    next
                });
            animating = true;
        }

        let pointer = self.pointer.get().map(|(x, y)| Point::new(x, y));
        let seek_hover = self.drag.get() == Drag::Seek
            || layout_hint
                .zip(pointer)
                .is_some_and(|(l, p)| self.controls_interactive() && l.seek_hit.contains(p));
        let volume_hover = self.drag.get() == Drag::Volume
            || layout_hint
                .zip(pointer)
                .is_some_and(|(l, p)| self.volume_slider_visible(now) && l.volume_pill.contains(p));
        for (cell, on) in [
            (&self.seek_grabber_scale, seek_hover),
            (&self.volume_grabber_scale, volume_hover),
        ] {
            let target = if on { 1.0 } else { 0.0 };
            if cell.get() != target {
                cell.set(approach(
                    cell.get(),
                    target,
                    dt,
                    1.0 / GRABBER_TRANSITION_SECS,
                ));
                animating = true;
            }
        }

        if self
            .volume_hover_left_at
            .get()
            .is_some_and(|t| now.duration_since(t) < VOLUME_HOVER_GRACE * 2)
        {
            animating = true;
        }
        if let Some((start, _)) = self.pop.get() {
            if now.duration_since(start) >= POP_DURATION {
                self.pop.set(None);
            }
            animating = true;
        }
        if self.ui.wants_playing.get() && target == 0.0 {
            animating = true;
        }
        animating
    }

    fn paint_pop(&self, canvas: &Canvas, center: (f32, f32), now: Instant) {
        let Some((start, play)) = self.pop.get() else {
            return;
        };
        let t = now.duration_since(start).as_secs_f32();
        let half = POP_DURATION.as_secs_f32() / 2.0;
        let opacity = 0.8
            * if t < half {
                t / half
            } else {
                (1.0 - (t - half) / half).max(0.0)
            };

        let scale = 1.0 + 1.5 * (1.0 - (-t * 2.5).exp());
        canvas.save_layer_alpha_f(None, opacity);
        canvas.translate(center);
        canvas.scale((scale, scale));
        fill_circle(canvas, (0.0, 0.0), CENTER_BUTTON_D / 2.0, BAR_BG);
        draw_icon(
            canvas,
            if play { PLAY } else { PAUSE },
            CONTROL_ICON_HOVER,
            (-CENTER_ICON / 2.0, -CENTER_ICON / 2.0),
            1.0,
            1.0,
        );
        canvas.restore();
    }

    fn paint_poster_button(&self, canvas: &Canvas, rect: AbsoluteRect) {
        let hovered = self.pointer.get().is_some();
        let dy = if self.pressed.get() { 1.0 } else { 0.0 };
        let center = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0 + dy);
        canvas.save_layer_alpha_f(None, if hovered { 0.8 } else { 0.6 });
        fill_circle(canvas, center, CENTER_BUTTON_D / 2.0, Color::BLACK);

        draw_icon(
            canvas,
            PLAY,
            Color::WHITE,
            (
                center.0 - CENTER_ICON / 2.0 + 1.0,
                center.1 - CENTER_ICON / 2.0,
            ),
            1.0,
            1.0,
        );
        canvas.restore();
    }

    fn paint_controls(&self, canvas: &Canvas, layout: &Layout, now: Instant) {
        let pointer = self.pointer.get().map(|(x, y)| Point::new(x, y));
        let over =
            |r: Rect| pointer.is_some_and(|p| r.contains(p)) && self.drag.get() == Drag::None;
        let pressed_on = |r: Rect| self.pressed.get() && over(r);

        let mut bg = Paint::default();
        bg.set_color(BAR_BG);
        canvas.draw_rect(layout.bar, &bg);

        let ended = self.has_ended();
        let duration = self.duration();
        let position = if self.drag.get() == Drag::Seek {
            self.drag_fraction.get() as f64 * duration
        } else if ended {
            duration
        } else {
            self.player
                .borrow()
                .as_ref()
                .map_or(0.0, VideoPlayer::position_secs)
                .min(duration.max(0.0))
        };

        let play_icon = if self.ui.wants_playing.get() {
            PAUSE
        } else if ended {
            REPLAY
        } else {
            PLAY
        };
        draw_button_icon(
            canvas,
            play_icon,
            layout.play_btn,
            over(layout.play_btn),
            pressed_on(layout.play_btn),
        );

        if let Some((x, top)) = layout.time_origin {
            let font = time_font();
            let (current, total) = time_strings(position, duration);
            let (_, metrics) = font.metrics();
            let baseline =
                top + (TIME_FONT_SIZE - (metrics.descent - metrics.ascent)) / 2.0 - metrics.ascent;
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_color(Color::WHITE);
            let mut cx = x;
            canvas.draw_str(&current, (cx, baseline), &font, &paint);
            cx += font.measure_str(&current, None).0 + TIME_SEPARATOR_MARGIN;
            canvas.draw_str("/", (cx, baseline), &font, &paint);
            cx += font.measure_str("/", None).0 + TIME_SEPARATOR_MARGIN;
            canvas.draw_str(&total, (cx, baseline), &font, &paint);
        }

        self.paint_seek_bar(canvas, layout, position, duration, pointer);

        let perceptual = VOLUME.with(Cell::get);
        let volume_icon = if self.ui.muted.get() || perceptual == 0.0 {
            VOICE_X
        } else if perceptual < 0.5 {
            VOICE_LOW
        } else {
            VOICE_NORMAL
        };
        draw_button_icon(
            canvas,
            volume_icon,
            layout.volume_btn,
            over(layout.volume_btn),
            pressed_on(layout.volume_btn),
        );
        let fs_icon = if self.is_fullscreen() {
            FULLSCREEN_EXIT
        } else {
            FULLSCREEN_ENTER
        };
        draw_button_icon(
            canvas,
            fs_icon,
            layout.fullscreen_btn,
            over(layout.fullscreen_btn),
            pressed_on(layout.fullscreen_btn),
        );

        if self.volume_slider_visible(now) {
            self.paint_volume_slider(canvas, layout, perceptual);
        }
    }

    fn paint_seek_bar(
        &self,
        canvas: &Canvas,
        layout: &Layout,
        position: f64,
        duration: f64,
        pointer: Option<Point>,
    ) {
        let track = layout.track;
        let dragging = self.drag.get() == Drag::Seek;
        let fraction = if dragging {
            self.drag_fraction.get()
        } else if duration > 0.0 {
            (position / duration).clamp(0.0, 1.0) as f32
        } else {
            0.0
        };
        let hover_fraction = pointer
            .filter(|p| !dragging && self.controls_interactive() && layout.seek_hit.contains(*p))
            .map(|p| Self::seek_fraction_at(p.x, layout));

        let edge = |w: f32| {
            Rect::from_xywh(
                track.left - FAKE_EDGE,
                track.top,
                w + 2.0 * FAKE_EDGE,
                track.height(),
            )
        };
        fill_rrect(canvas, edge(track.width()), FAKE_EDGE, TRACK);
        if let Some(h) = hover_fraction {
            fill_rrect(canvas, edge(track.width() * h), FAKE_EDGE, PREVIEW);
        }
        fill_rrect(canvas, edge(track.width() * fraction), FAKE_EDGE, BRAND);

        let scale = self.seek_grabber_scale.get();
        if scale > 0.0 {
            let color = if scale >= 1.0 || dragging || hover_fraction.is_some() {
                BRAND_560
            } else {
                BRAND
            };
            fill_circle(
                canvas,
                (track.left + track.width() * fraction, track.center_y()),
                GRABBER_D / 2.0 * scale,
                color,
            );
        }

        let bubble_fraction = if dragging {
            Some(fraction)
        } else {
            hover_fraction
        };
        if let Some(f) = bubble_fraction {
            let text = format::media_time(f as f64 * duration);
            let font = bubble_font();
            let text_w = font.measure_str(&text, None).0;
            let cx = track.left + track.width() * f;
            let bubble = Rect::from_xywh(
                cx - (text_w + 2.0 * BUBBLE_PAD_X) / 2.0,
                track.top - BUBBLE_OFFSET_Y,
                text_w + 2.0 * BUBBLE_PAD_X,
                BUBBLE_H,
            );
            fill_rrect(canvas, bubble, 3.0, Color::BLACK);
            let arrow = Path::polygon(
                &[
                    Point::new(cx - BUBBLE_ARROW, bubble.bottom),
                    Point::new(cx + BUBBLE_ARROW, bubble.bottom),
                    Point::new(cx, bubble.bottom + BUBBLE_ARROW),
                ],
                true,
                None,
                None,
            );
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_color(Color::BLACK);
            canvas.draw_path(&arrow, &paint);
            let (_, metrics) = font.metrics();
            let baseline =
                bubble.top + (BUBBLE_H - (metrics.descent - metrics.ascent)) / 2.0 - metrics.ascent;
            paint.set_color(TEXT_STRONG);
            canvas.draw_str(&text, (bubble.left + BUBBLE_PAD_X, baseline), &font, &paint);
        }
    }

    fn paint_volume_slider(&self, canvas: &Canvas, layout: &Layout, perceptual: f32) {
        fill_rrect(canvas, layout.volume_pill, 8.0, VOLUME_PILL_BG);
        let track = layout.volume_track;
        let value = if self.ui.muted.get() { 0.0 } else { perceptual };
        let edge = |h: f32| {
            Rect::from_ltrb(
                track.left,
                track.bottom - h - FAKE_EDGE,
                track.right,
                track.bottom + FAKE_EDGE,
            )
        };
        fill_rrect(canvas, edge(track.height()), FAKE_EDGE, TRACK);
        fill_rrect(canvas, edge(track.height() * value), FAKE_EDGE, BRAND);
        let scale = self.volume_grabber_scale.get();
        if scale > 0.0 {
            fill_circle(
                canvas,
                (track.center_x(), track.bottom - track.height() * value),
                GRABBER_D / 2.0 * scale,
                BRAND_560,
            );
        }
    }
}

impl Drop for VideoWidget {
    fn drop(&mut self) {
        if self.player.borrow().is_some() {
            if let Some(videos) = Self::cache_of(&self.mode) {
                videos.release(&self.url);
            }
        }
    }
}

impl Widget for VideoWidget {
    fn measure(
        &self,
        _known: taffy::Size<Option<f32>>,
        _available: taffy::Size<taffy::AvailableSpace>,
    ) -> taffy::Size<f32> {
        if self.is_fullscreen() {
            return taffy::Size::ZERO;
        }
        taffy::Size {
            width: self.display_w,
            height: self.display_h,
        }
    }

    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let now = Instant::now();
        if self.is_gifv() && self.player.borrow().is_none() {
            if let Some(player) = self.player_for_playback() {
                player.set_muted(true);
                player.play();
            }
        }
        let player = self.player.borrow().clone();
        if let Some(player) = &player {
            player.set_volume(perceptual_to_amplitude(VOLUME.with(Cell::get)));
        }

        let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
        canvas.save();
        if !self.is_fullscreen() && style.corner_radius > 0.0 {
            canvas.clip_rrect(
                RRect::new_rect_xy(bounds, style.corner_radius, style.corner_radius),
                None,
                true,
            );
        } else {
            canvas.clip_rect(bounds, None, true);
        }

        let image = self.current_image();
        match &image {
            Some(image) => {
                let dst = if self.is_fullscreen() {
                    self.letterbox(bounds, Some(image))
                } else {
                    bounds
                };
                let mut paint = Paint::default();
                paint.set_anti_alias(true);

                let sampling = primitives::sampling_for(
                    canvas,
                    image.width() as f32,
                    dst.width(),
                );
                canvas.draw_image_rect_with_sampling_options(image, None, dst, sampling, &paint);
            }
            None if !self.is_fullscreen() => {
                let mut bg = Paint::default();
                bg.set_color(SURFACE_HIGH);
                canvas.draw_rect(bounds, &bg);
            }
            None => {}
        }

        if self.is_gifv() {
            canvas.restore();
            if player.as_ref().is_some_and(VideoPlayer::is_playing) {
                requiem_ui_paint::animate_region_on_wake(
                    canvas,
                    Rect::from_xywh(rect.x, rect.y, rect.width, rect.height),
                );
            }
            return;
        }

        if self.showing_poster() {
            self.paint_poster_button(canvas, rect);
            canvas.restore();
            return;
        }

        let layout = self.layout(rect);
        let animating = self.advance_animations(now, Some(&layout));
        let layout = self.layout(rect);
        self.paint_pop(
            canvas,
            (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0),
            now,
        );
        if self.view().controls_hidden.get() < 1.0 {
            self.paint_controls(canvas, &layout, now);
        }
        canvas.restore();

        if animating {
            requiem_ui_paint::request_repaint();
        }
        if player.as_ref().is_some_and(VideoPlayer::is_playing) {
            requiem_ui_paint::animate_region_on_wake(
                canvas,
                Rect::from_xywh(rect.x, rect.y, rect.width, rect.height),
            );
        }
    }

    fn handle_event(&mut self, event: &UiEvent, rect: AbsoluteRect) -> EventResult {
        if self.is_gifv() {
            return EventResult::Ignored;
        }
        let layout = self.layout(rect);
        match *event {
            UiEvent::MouseMoved { x, y } => {
                if let Mode::Attachment { videos, .. } = &self.mode {
                    videos.preload(&self.url);
                }
                self.pointer.set(Some((x, y)));
                self.view().last_move.set(Some(Instant::now()));
                self.update_volume_hover(&layout, Some(Point::new(x, y)));
                match self.drag.get() {
                    Drag::Seek => self.seek_to_fraction(Self::seek_fraction_at(x, &layout)),
                    Drag::Volume => self.apply_volume_from_y(y, &layout),
                    Drag::None => {}
                }
                requiem_ui_paint::request_repaint();
                if self.drag.get() != Drag::None {
                    EventResult::Handled
                } else {
                    EventResult::Ignored
                }
            }
            UiEvent::MouseDown {
                x,
                y,
                button: MouseButton::Left,
                ..
            } => {
                let point = Point::new(x, y);
                self.pressed.set(true);
                requiem_ui_paint::request_repaint();
                if self.showing_poster() {
                    self.set_playing(true);
                    return EventResult::Handled;
                }
                let now = Instant::now();
                if self.controls_interactive()
                    && self.volume_slider_visible(now)
                    && layout.volume_pill.contains(point)
                {
                    self.drag.set(Drag::Volume);
                    self.apply_volume_from_y(y, &layout);
                    return EventResult::Handled;
                }
                if self.controls_interactive()
                    && Rect::from_ltrb(
                        layout.bar.left,
                        layout.bar.top,
                        layout.bar.right,
                        layout.bar.top + BAR_VISIBLE_H,
                    )
                    .contains(point)
                {
                    if layout.play_btn.contains(point) {
                        self.toggle_playing();
                    } else if layout.seek_hit.contains(point) {
                        self.drag.set(Drag::Seek);
                        self.last_seek_fraction.set(-1.0);
                        if self.ui.wants_playing.get() {
                            if let Some(player) = self.player.borrow().as_ref() {
                                player.pause();
                            }
                        }
                        self.seek_to_fraction(Self::seek_fraction_at(x, &layout));
                    } else if layout.volume_btn.contains(point) {
                        let muted = !self.ui.muted.get();
                        self.ui.muted.set(muted);
                        if let Some(player) = self.player.borrow().as_ref() {
                            player.set_muted(muted);
                        }
                    } else if layout.fullscreen_btn.contains(point) {
                        self.toggle_fullscreen();
                    }
                    return EventResult::Handled;
                }
                self.toggle_playing();
                EventResult::Handled
            }
            UiEvent::MouseUp { .. } => {
                self.pressed.set(false);
                if self.drag.get() != Drag::None {
                    self.end_drag();
                    return EventResult::Handled;
                }
                requiem_ui_paint::request_repaint();
                EventResult::Ignored
            }
            _ => EventResult::Ignored,
        }
    }

    fn wants_pointer_capture(&self) -> bool {
        self.drag.get() != Drag::None
    }

    fn clear_press(&mut self) {
        self.pressed.set(false);
        if self.drag.get() != Drag::None {
            self.end_drag();
        }
    }

    fn clear_hover(&mut self) {
        if self.drag.get() != Drag::None {
            return;
        }
        self.pointer.set(None);
        self.view().last_move.set(None);
        if self.volume_hovered.replace(false) {
            self.volume_hover_left_at.set(Some(Instant::now()));
        }
        requiem_ui_paint::request_repaint();
    }

    fn cursor_icon(&self) -> Option<CursorIcon> {
        if self.is_gifv() {
            return Some(CursorIcon::Pointer);
        }
        if self.view().controls_hidden.get() >= 1.0 {
            return Some(CursorIcon::Default);
        }
        Some(CursorIcon::Pointer)
    }
}

fn scaled_video_dims(width: u32, height: u32, max_w: f32, max_h: f32) -> (f32, f32) {
    let (w, h) = (width.max(1) as f32, height.max(1) as f32);
    let scale = (max_w / w).min(max_h / h).min(1.0);
    (w * scale, h * scale)
}

#[allow(clippy::too_many_arguments)]
pub fn attachment_video_node(
    url: &str,
    poster: Poster,
    width: Option<u32>,
    height: Option<u32>,
    videos: &VideoCache,
    max_w: f32,
    max_h: f32,
    fullscreen: &Signal<Option<FullscreenVideoState>>,
    class: &'static str,
) -> Node {
    let (iw, ih) = (width.unwrap_or(1280), height.unwrap_or(720));
    let display = scaled_video_dims(iw, ih, max_w, max_h);
    let widget = VideoWidget::new(
        url,
        poster,
        (iw, ih),
        display,
        Mode::Attachment {
            fullscreen: fullscreen.clone(),
            videos: videos.clone(),
        },
        videos.existing(url),
    );
    Node::new("div", widget).class(class)
}

#[allow(clippy::too_many_arguments)]
pub fn gifv_video_node(
    url: &str,
    poster: Poster,
    width: Option<u32>,
    height: Option<u32>,
    videos: &VideoCache,
    max_w: f32,
    max_h: f32,
    class: &'static str,
) -> Node {
    let (iw, ih) = (width.unwrap_or(1280), height.unwrap_or(720));
    let display = scaled_video_dims(iw, ih, max_w, max_h);
    let widget = VideoWidget::new(
        url,
        poster,
        (iw, ih),
        display,
        Mode::Gifv {
            videos: videos.clone(),
        },
        videos.existing(url),
    );
    widget.ui.muted.set(true);
    Node::new("div", widget).class(class)
}

pub fn fullscreen_overlay_node(
    state: &FullscreenVideoState,
    fullscreen: &Signal<Option<FullscreenVideoState>>,
) -> Node {
    let widget = VideoWidget::new(
        &state.url,
        state.poster.clone(),
        (state.intrinsic_w, state.intrinsic_h),
        (0.0, 0.0),
        Mode::Fullscreen {
            fullscreen: fullscreen.clone(),
        },
        Some(state.player.clone()),
    );
    widget.view().last_move.set(Some(Instant::now()));
    div()
        .class(video_player::VIDEO_FULLSCREEN_OVERLAY)
        .child(
            Node::new("div", widget)
                .class(video_player::VIDEO_FULLSCREEN_VIDEO),
        )
}
