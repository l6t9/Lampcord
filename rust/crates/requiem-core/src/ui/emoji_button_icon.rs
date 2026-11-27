use requiem_ui::widget::primitives;
use skia_safe::color_filters;
use skia_safe::surfaces;
use std::cell::Cell;
use std::sync::OnceLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::{AbsoluteRect, Widget};
use skia_safe::{Canvas, Paint, Rect};

thread_local! {

    static RNG_STATE: Cell<u32> = Cell::new({
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0x9e3779b9);
        if nanos == 0 { 0x9e3779b9 } else { nanos }
    });
}

pub(crate) fn random_u32() -> u32 {
    RNG_STATE.with(|state| {
        let mut x = state.get();
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        state.set(x);
        x
    })
}

const COLS: u32 = 20;
const ROWS: u32 = 4;
const TILE_PX: f32 = 48.0;

const CROSSFADE_SECS: f32 = 0.15;

static COLORED: OnceLock<Option<skia_safe::Image>> = OnceLock::new();
static GREY: OnceLock<Option<skia_safe::Image>> = OnceLock::new();

fn colored_sprite() -> Option<&'static skia_safe::Image> {
    COLORED
        .get_or_init(|| super::asset_image("image/emoji_button_sprite"))
        .as_ref()
}

fn tinted(mask: &skia_safe::Image, color: skia_safe::Color) -> Option<skia_safe::Image> {
    let mut surface = surfaces::raster_n32_premul((mask.width(), mask.height()))?;
    let mut paint = Paint::default();
    paint.set_color_filter(color_filters::blend(
        color,
        skia_safe::BlendMode::SrcIn,
    ));
    surface.canvas().draw_image(mask, (0, 0), Some(&paint));
    Some(surface.image_snapshot())
}

fn grey_sprite() -> Option<&'static skia_safe::Image> {
    GREY.get_or_init(|| {
        super::asset_image("image/emoji_button_sprite_grey")
            .and_then(|mask| tinted(&mask, super::TEXT_SECONDARY))
    })
    .as_ref()
}

const TILE_COUNT: u32 = COLS * ROWS - 3;

fn random_tile() -> (u32, u32) {
    let index = random_u32() % TILE_COUNT;
    (index % COLS, index / COLS)
}

fn tile_rect(col: u32, row: u32) -> Rect {
    Rect::from_xywh(col as f32 * TILE_PX, row as f32 * TILE_PX, TILE_PX, TILE_PX)
}

thread_local! {
    /// The resting icon. Shared by every button instance so rebuilding the composer keeps it;
    /// it only changes once the pointer has hovered and left.
    static IDLE_TILE: Cell<(u32, u32)> = Cell::new(random_tile());
}

pub struct EmojiButtonIconWidget {
    hover_tile: Cell<(u32, u32)>,
    hover_started_at: Cell<Option<Instant>>,
}

impl EmojiButtonIconWidget {
    pub fn new() -> Self {
        Self {
            hover_tile: Cell::new(IDLE_TILE.with(Cell::get)),
            hover_started_at: Cell::new(None),
        }
    }
}

impl Default for EmojiButtonIconWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for EmojiButtonIconWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        if style.is_hovered && self.hover_started_at.get().is_none() {
            self.hover_started_at.set(Some(Instant::now()));
            self.hover_tile.set(random_tile());
        } else if !style.is_hovered
            && self.hover_started_at.take().is_some() {
                IDLE_TILE.with(|tile| tile.set(random_tile()));
            }

        let t = match self.hover_started_at.get() {
            Some(started) => (started.elapsed().as_secs_f32() / CROSSFADE_SECS).min(1.0),
            None => 0.0,
        };

        let dst = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);

        if t < 1.0 {
            if let Some(image) = grey_sprite() {
                let mut paint = Paint::default();
                paint.set_anti_alias(true);
                paint.set_alpha_f(1.0 - t);
                primitives::draw_atlas_tile(
                    canvas,
                    image,
                    {
                        let (col, row) = IDLE_TILE.with(Cell::get);
                        tile_rect(col, row)
                    },
                    dst,
                    &paint,
                );
            }
        }

        if t > 0.0 {
            if let Some(image) = colored_sprite() {
                let mut paint = Paint::default();
                paint.set_anti_alias(true);
                paint.set_alpha_f(t);
                let (col, row) = self.hover_tile.get();
                primitives::draw_atlas_tile(
                    canvas,
                    image,
                    tile_rect(col, row),
                    dst,
                    &paint,
                );
            }
        }
    }
}
