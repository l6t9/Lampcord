//! Material 3 Expressive vector icons rendered via GPUI canvas.
//!
//! Provides vector icons adhering to Google Material Symbols specifications,
//! drawn with zero raster assets using GPUI [`PathBuilder`].

use gpui::{Bounds, IntoElement, PathBuilder, Pixels, Rgba, Styled, Window, point, px, svg};

/// Material 3 Expressive icon identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum M3Icon {
    /// 2x2 grid representing all components or dashboard (Material `grid_view`).
    All,
    /// Composition of geometric shapes (circle, triangle, square) representing shapes (Material `category`).
    Shapes,
    /// Dual slider tracks with adjustment handles (Material `tune`).
    Sliders,
    /// Switch toggle pill with circular thumb (Material `toggle_on`).
    Switches,
    /// Split button container with vertical divider and dropdown chevron.
    SplitButton,
    /// Button container with inner touch indicator (Material `smart_button`).
    Button,
    /// Outlined card container with media header and content lines (Material `crop_portrait`).
    Cards,
    /// Left arrow for previous navigation (Material `arrow_back`).
    ArrowBack,
    /// Right arrow for next navigation (Material `arrow_forward`).
    ArrowForward,
    /// Triangle media action (Material `play_arrow`).
    Play,
    /// Tray and arrow down (Material `download`).
    Download,
    /// Plus symbol (Material `add`).
    Add,
    /// Checkmark symbol (Material `check`).
    Check,
    /// Information circle with 'i' (Material `info`).
    Info,
    /// Ticket voucher (Material `confirmation_number`).
    Ticket,
    /// Sun icon (Material `light_mode`).
    LightMode,
    /// Crescent moon icon (Material `dark_mode`).
    DarkMode,
    /// Menu / drawer navigation icon (Material `menu`).
    Menu,
    /// Eye icon for visibility (Material `visibility`).
    Visibility,
    /// Content copy / duplicate sheets icon (Material `content_copy`).
    ContentCopy,
    /// Edit / pencil icon (Material `edit`).
    Edit,
    /// Cloud icon (Material `cloud`).
    Cloud,
    /// Submenu right chevron pointer (Material `chevron_right`).
    ChevronRight,
    /// Three vertical dots overflow menu (Material `more_vert`).
    MoreVert,
    /// Circular reload / refresh arrow (Material `refresh`).
    Refresh,
    /// Settings gear / cog icon (Material `settings`).
    Settings,
    /// Help question circle icon (Material `help`).
    Help,
    /// User profile avatar icon (Material `person`).
    Person,
    /// Downward expansion chevron (Material `expand_more`).
    ExpandMore,
    /// Upward expansion chevron (Material `expand_less`).
    ExpandLess,
    /// Four-pointed diamond sparkle star (Material `spark` / Gemini symbol).
    Spark,
    /// RAM memory chip (Material `memory`).
    Memory,
    /// Processor / CPU load gauge (Material `planner_review` / `speed`).
    Cpu,
    /// Local Area Network / Ethernet tree (Material `lan`).
    Lan,
    /// Bluetooth symbol (Material `bluetooth`).
    Bluetooth,
    /// Microphone off / muted (Material `mic_off`).
    MicOff,
    /// Fullscreen toggle brackets (Material `fullscreen`).
    Fullscreen,
    /// Keyboard rectangle with keys (Material `keyboard`).
    Keyboard,
    /// Vinyl record / disc (Material `album`).
    Album,
    /// Two vertical pause bars (Material `pause`).
    Pause,
    /// Musical note glyph (Material `music_note`).
    MusicNote,
}

impl M3Icon {
    /// Draw this vector icon onto the GPUI window canvas within `bounds`.
    pub fn draw(&self, window: &mut Window, bounds: Bounds<Pixels>, color: Rgba) {
        let w = f32::from(bounds.size.width);
        let h = f32::from(bounds.size.height);
        if w < 1.0 || h < 1.0 {
            return;
        }
        let ox = bounds.left();
        let oy = bounds.top();
        let stroke_w = (w * 0.12).clamp(1.4, 3.0);

        let pt = |fx: f32, fy: f32| point(ox + px(w * fx), oy + px(h * fy));

        match self {
            M3Icon::All => {
                // 2x2 grid of rounded squares (Material 'grid_view')
                let mut b = PathBuilder::stroke(px(stroke_w));
                // Top-left
                b.move_to(pt(0.18, 0.18));
                b.line_to(pt(0.42, 0.18));
                b.line_to(pt(0.42, 0.42));
                b.line_to(pt(0.18, 0.42));
                b.line_to(pt(0.18, 0.18));
                // Top-right
                b.move_to(pt(0.58, 0.18));
                b.line_to(pt(0.82, 0.18));
                b.line_to(pt(0.82, 0.42));
                b.line_to(pt(0.58, 0.42));
                b.line_to(pt(0.58, 0.18));
                // Bottom-left
                b.move_to(pt(0.18, 0.58));
                b.line_to(pt(0.42, 0.58));
                b.line_to(pt(0.42, 0.82));
                b.line_to(pt(0.18, 0.82));
                b.line_to(pt(0.18, 0.58));
                // Bottom-right
                b.move_to(pt(0.58, 0.58));
                b.line_to(pt(0.82, 0.58));
                b.line_to(pt(0.82, 0.82));
                b.line_to(pt(0.58, 0.82));
                b.line_to(pt(0.58, 0.58));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Shapes => {
                // Circle at top, Triangle at bottom-left, Square at bottom-right (Material 'category')
                let mut b = PathBuilder::stroke(px(stroke_w));
                // Circle
                let cx = 0.50;
                let cy = 0.32;
                let r = 0.16;
                for i in 0..=12 {
                    let angle = i as f32 * std::f32::consts::TAU / 12.0;
                    let p = pt(cx + r * angle.cos(), cy + r * angle.sin());
                    if i == 0 {
                        b.move_to(p);
                    } else {
                        b.line_to(p);
                    }
                }
                // Triangle
                b.move_to(pt(0.25, 0.58));
                b.line_to(pt(0.10, 0.84));
                b.line_to(pt(0.40, 0.84));
                b.line_to(pt(0.25, 0.58));
                // Square
                b.move_to(pt(0.60, 0.60));
                b.line_to(pt(0.84, 0.60));
                b.line_to(pt(0.84, 0.84));
                b.line_to(pt(0.60, 0.84));
                b.line_to(pt(0.60, 0.60));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Sliders => {
                // Material 'tune': dual slider tracks with handles
                let mut b = PathBuilder::stroke(px(stroke_w));
                // Track 1
                b.move_to(pt(0.14, 0.36));
                b.line_to(pt(0.86, 0.36));
                // Notch 1
                b.move_to(pt(0.38, 0.22));
                b.line_to(pt(0.38, 0.50));
                // Track 2
                b.move_to(pt(0.14, 0.64));
                b.line_to(pt(0.86, 0.64));
                // Notch 2
                b.move_to(pt(0.62, 0.50));
                b.line_to(pt(0.62, 0.78));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Switches => {
                // Material 'toggle_on'
                let mut b = PathBuilder::stroke(px(stroke_w));
                // Pill outline
                b.move_to(pt(0.32, 0.28));
                b.line_to(pt(0.68, 0.28));
                for i in 0..=8 {
                    let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 8.0;
                    b.line_to(pt(0.68 + 0.22 * a.cos(), 0.50 + 0.22 * a.sin()));
                }
                b.line_to(pt(0.32, 0.72));
                for i in 0..=8 {
                    let a = std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 8.0;
                    b.line_to(pt(0.32 + 0.22 * a.cos(), 0.50 + 0.22 * a.sin()));
                }
                b.close();
                // Inner thumb circle on right
                for i in 0..=12 {
                    let a = i as f32 * std::f32::consts::TAU / 12.0;
                    let p = pt(0.68 + 0.12 * a.cos(), 0.50 + 0.12 * a.sin());
                    if i == 0 {
                        b.move_to(p);
                    } else {
                        b.line_to(p);
                    }
                }
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::SplitButton => {
                // Split button container with divider & dropdown chevron
                let mut b = PathBuilder::stroke(px(stroke_w));
                b.move_to(pt(0.12, 0.26));
                b.line_to(pt(0.88, 0.26));
                b.line_to(pt(0.88, 0.74));
                b.line_to(pt(0.12, 0.74));
                b.close();
                b.move_to(pt(0.64, 0.26));
                b.line_to(pt(0.64, 0.74));
                b.move_to(pt(0.72, 0.46));
                b.line_to(pt(0.77, 0.56));
                b.line_to(pt(0.82, 0.46));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Button => {
                // Material 'smart_button': rounded container with inner indicator
                let mut b = PathBuilder::stroke(px(stroke_w));
                b.move_to(pt(0.24, 0.26));
                b.line_to(pt(0.76, 0.26));
                for i in 0..=6 {
                    let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 6.0;
                    b.line_to(pt(0.76 + 0.14 * a.cos(), 0.50 + 0.24 * a.sin()));
                }
                b.line_to(pt(0.24, 0.74));
                for i in 0..=6 {
                    let a = std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 6.0;
                    b.line_to(pt(0.24 + 0.14 * a.cos(), 0.50 + 0.24 * a.sin()));
                }
                b.close();
                b.move_to(pt(0.38, 0.50));
                b.line_to(pt(0.62, 0.50));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Cards => {
                // Outlined card with header and content lines
                let mut b = PathBuilder::stroke(px(stroke_w));
                b.move_to(pt(0.20, 0.16));
                b.line_to(pt(0.80, 0.16));
                b.line_to(pt(0.80, 0.84));
                b.line_to(pt(0.20, 0.84));
                b.close();
                b.move_to(pt(0.20, 0.46));
                b.line_to(pt(0.80, 0.46));
                b.move_to(pt(0.30, 0.60));
                b.line_to(pt(0.70, 0.60));
                b.move_to(pt(0.30, 0.72));
                b.line_to(pt(0.55, 0.72));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::ArrowBack => {
                // Material 'arrow_back'
                let mut b = PathBuilder::stroke(px(stroke_w * 1.1));
                b.move_to(pt(0.82, 0.50));
                b.line_to(pt(0.18, 0.50));
                b.move_to(pt(0.44, 0.26));
                b.line_to(pt(0.18, 0.50));
                b.line_to(pt(0.44, 0.74));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::ArrowForward => {
                // Material 'arrow_forward'
                let mut b = PathBuilder::stroke(px(stroke_w * 1.1));
                b.move_to(pt(0.18, 0.50));
                b.line_to(pt(0.82, 0.50));
                b.move_to(pt(0.56, 0.26));
                b.line_to(pt(0.82, 0.50));
                b.line_to(pt(0.56, 0.74));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Play => {
                // Material 'play_arrow' (filled)
                let mut b = PathBuilder::fill();
                b.move_to(pt(0.30, 0.22));
                b.line_to(pt(0.80, 0.50));
                b.line_to(pt(0.30, 0.78));
                b.close();
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Download => {
                // Material 'download'
                let mut b = PathBuilder::stroke(px(stroke_w));
                b.move_to(pt(0.20, 0.66));
                b.line_to(pt(0.20, 0.82));
                b.line_to(pt(0.80, 0.82));
                b.line_to(pt(0.80, 0.66));
                b.move_to(pt(0.50, 0.18));
                b.line_to(pt(0.50, 0.62));
                b.move_to(pt(0.34, 0.46));
                b.line_to(pt(0.50, 0.62));
                b.line_to(pt(0.66, 0.46));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Add => {
                // Material 'add'
                let mut b = PathBuilder::stroke(px(stroke_w));
                b.move_to(pt(0.20, 0.50));
                b.line_to(pt(0.80, 0.50));
                b.move_to(pt(0.50, 0.20));
                b.line_to(pt(0.50, 0.80));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Check => {
                // Material 'check'
                let mut b = PathBuilder::stroke(px(stroke_w * 1.1));
                b.move_to(pt(0.18, 0.52));
                b.line_to(pt(0.42, 0.76));
                b.line_to(pt(0.82, 0.26));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Info => {
                // Material 'info'
                let mut b = PathBuilder::stroke(px(stroke_w));
                for i in 0..=16 {
                    let a = i as f32 * std::f32::consts::TAU / 16.0;
                    let p = pt(0.50 + 0.36 * a.cos(), 0.50 + 0.36 * a.sin());
                    if i == 0 {
                        b.move_to(p);
                    } else {
                        b.line_to(p);
                    }
                }
                b.move_to(pt(0.50, 0.30));
                b.line_to(pt(0.50, 0.34));
                b.move_to(pt(0.50, 0.46));
                b.line_to(pt(0.50, 0.70));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Ticket => {
                // Material 'confirmation_number'
                let mut b = PathBuilder::stroke(px(stroke_w));
                b.move_to(pt(0.18, 0.26));
                b.line_to(pt(0.82, 0.26));
                b.line_to(pt(0.82, 0.44));
                b.line_to(pt(0.74, 0.50));
                b.line_to(pt(0.82, 0.56));
                b.line_to(pt(0.82, 0.74));
                b.line_to(pt(0.18, 0.74));
                b.line_to(pt(0.18, 0.56));
                b.line_to(pt(0.26, 0.50));
                b.line_to(pt(0.18, 0.44));
                b.close();
                b.move_to(pt(0.40, 0.50));
                b.line_to(pt(0.60, 0.50));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::LightMode => {
                // Material 'light_mode'
                let mut b = PathBuilder::stroke(px(stroke_w));
                for i in 0..=12 {
                    let a = i as f32 * std::f32::consts::TAU / 12.0;
                    let p = pt(0.50 + 0.18 * a.cos(), 0.50 + 0.18 * a.sin());
                    if i == 0 {
                        b.move_to(p);
                    } else {
                        b.line_to(p);
                    }
                }
                for i in 0..8 {
                    let a = i as f32 * std::f32::consts::TAU / 8.0;
                    b.move_to(pt(0.50 + 0.26 * a.cos(), 0.50 + 0.26 * a.sin()));
                    b.line_to(pt(0.50 + 0.40 * a.cos(), 0.50 + 0.40 * a.sin()));
                }
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::DarkMode => {
                // Material 'dark_mode'
                let mut b = PathBuilder::stroke(px(stroke_w));
                b.move_to(pt(0.65, 0.20));
                for i in 0..=8 {
                    let a =
                        -std::f32::consts::PI * 0.4 + i as f32 * std::f32::consts::PI * 0.9 / 8.0;
                    b.line_to(pt(0.46 + 0.34 * a.cos(), 0.50 + 0.34 * a.sin()));
                }
                for i in (0..=8).rev() {
                    let a =
                        -std::f32::consts::PI * 0.3 + i as f32 * std::f32::consts::PI * 0.7 / 8.0;
                    b.line_to(pt(0.60 + 0.24 * a.cos(), 0.50 + 0.24 * a.sin()));
                }
                b.close();
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Menu => {
                // Material 'menu': three horizontal bars
                let mut b = PathBuilder::stroke(px(stroke_w));
                b.move_to(pt(0.18, 0.28));
                b.line_to(pt(0.82, 0.28));
                b.move_to(pt(0.18, 0.50));
                b.line_to(pt(0.82, 0.50));
                b.move_to(pt(0.18, 0.72));
                b.line_to(pt(0.82, 0.72));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Visibility => {
                // Material 'visibility': eye outline with pupil
                let mut b = PathBuilder::stroke(px(stroke_w));
                b.move_to(pt(0.14, 0.50));
                for i in 1..=8 {
                    let t = i as f32 / 8.0;
                    let x = 0.14 + 0.72 * t;
                    let y = 0.50 - 0.24 * (std::f32::consts::PI * t).sin();
                    b.line_to(pt(x, y));
                }
                for i in 1..=8 {
                    let t = 1.0 - i as f32 / 8.0;
                    let x = 0.14 + 0.72 * t;
                    let y = 0.50 + 0.24 * (std::f32::consts::PI * t).sin();
                    b.line_to(pt(x, y));
                }
                b.close();
                let pr = 0.11;
                for i in 0..=8 {
                    let a = i as f32 * std::f32::consts::TAU / 8.0;
                    let p = pt(0.50 + pr * a.cos(), 0.50 + pr * a.sin());
                    if i == 0 {
                        b.move_to(p);
                    } else {
                        b.line_to(p);
                    }
                }
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::ContentCopy => {
                // Material 'content_copy': front sheet and offset back sheet
                let mut b = PathBuilder::stroke(px(stroke_w));
                b.move_to(pt(0.32, 0.32));
                b.line_to(pt(0.82, 0.32));
                b.line_to(pt(0.82, 0.84));
                b.line_to(pt(0.32, 0.84));
                b.close();
                b.move_to(pt(0.18, 0.68));
                b.line_to(pt(0.18, 0.18));
                b.line_to(pt(0.68, 0.18));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Edit => {
                // Material 'edit': angled pencil with nib
                let mut b = PathBuilder::stroke(px(stroke_w));
                b.move_to(pt(0.24, 0.64));
                b.line_to(pt(0.64, 0.24));
                b.line_to(pt(0.76, 0.36));
                b.line_to(pt(0.36, 0.76));
                b.close();
                b.move_to(pt(0.24, 0.64));
                b.line_to(pt(0.16, 0.84));
                b.line_to(pt(0.36, 0.76));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Cloud => {
                // Material 'cloud': cloud outline with flat base
                let mut b = PathBuilder::stroke(px(stroke_w));
                b.move_to(pt(0.28, 0.72));
                b.line_to(pt(0.74, 0.72));
                for i in 0..=4 {
                    let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 4.0;
                    b.line_to(pt(0.74 + 0.12 * a.cos(), 0.60 + 0.12 * a.sin()));
                }
                for i in 0..=6 {
                    let a = -std::f32::consts::PI + i as f32 * std::f32::consts::PI / 6.0;
                    b.line_to(pt(0.50 + 0.19 * a.cos(), 0.45 + 0.19 * a.sin()));
                }
                for i in 0..=4 {
                    let a = -std::f32::consts::PI + i as f32 * std::f32::consts::PI / 4.0;
                    b.line_to(pt(0.28 + 0.12 * a.cos(), 0.60 + 0.12 * a.sin()));
                }
                b.close();
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::ChevronRight => {
                // Submenu right pointer '>'
                let mut b = PathBuilder::stroke(px(stroke_w));
                b.move_to(pt(0.38, 0.28));
                b.line_to(pt(0.62, 0.50));
                b.line_to(pt(0.38, 0.72));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::MoreVert => {
                // Three vertical dots (Material 'more_vert')
                let mut b = PathBuilder::stroke(px(stroke_w));
                for cy in [0.24, 0.50, 0.76] {
                    for i in 0..=6 {
                        let a = i as f32 * std::f32::consts::TAU / 6.0;
                        let p = pt(0.50 + 0.05 * a.cos(), cy + 0.05 * a.sin());
                        if i == 0 {
                            b.move_to(p);
                        } else {
                            b.line_to(p);
                        }
                    }
                }
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Refresh => {
                // Circular reload arrow (Material 'refresh')
                let mut b = PathBuilder::stroke(px(stroke_w));
                let r = 0.28;
                for i in 0..=12 {
                    let a =
                        0.6 * std::f32::consts::PI - i as f32 * 1.5 * std::f32::consts::PI / 12.0;
                    let p = pt(0.50 + r * a.cos(), 0.50 + r * a.sin());
                    if i == 0 {
                        b.move_to(p);
                    } else {
                        b.line_to(p);
                    }
                }
                b.move_to(pt(0.72, 0.22));
                b.line_to(pt(0.78, 0.36));
                b.line_to(pt(0.64, 0.38));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Settings => {
                // Gear / cog (Material 'settings')
                let mut b = PathBuilder::stroke(px(stroke_w));
                let num_teeth = 6;
                for i in 0..(num_teeth * 4) {
                    let a = i as f32 * std::f32::consts::TAU / (num_teeth as f32 * 4.0);
                    let r = if (i % 4 == 1) || (i % 4 == 2) {
                        0.36
                    } else {
                        0.28
                    };
                    let p = pt(0.50 + r * a.cos(), 0.50 + r * a.sin());
                    if i == 0 {
                        b.move_to(p);
                    } else {
                        b.line_to(p);
                    }
                }
                b.close();
                for i in 0..=8 {
                    let a = i as f32 * std::f32::consts::TAU / 8.0;
                    let p = pt(0.50 + 0.12 * a.cos(), 0.50 + 0.12 * a.sin());
                    if i == 0 {
                        b.move_to(p);
                    } else {
                        b.line_to(p);
                    }
                }
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Help => {
                // Question circle (Material 'help')
                let mut b = PathBuilder::stroke(px(stroke_w));
                for i in 0..=12 {
                    let a = i as f32 * std::f32::consts::TAU / 12.0;
                    let p = pt(0.50 + 0.36 * a.cos(), 0.50 + 0.36 * a.sin());
                    if i == 0 {
                        b.move_to(p);
                    } else {
                        b.line_to(p);
                    }
                }
                b.move_to(pt(0.40, 0.38));
                b.line_to(pt(0.44, 0.32));
                b.line_to(pt(0.56, 0.32));
                b.line_to(pt(0.58, 0.42));
                b.line_to(pt(0.50, 0.50));
                b.line_to(pt(0.50, 0.58));
                b.move_to(pt(0.49, 0.68));
                b.line_to(pt(0.51, 0.68));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Person => {
                // User profile avatar (Material 'person')
                let mut b = PathBuilder::stroke(px(stroke_w));
                let hr = 0.14;
                for i in 0..=10 {
                    let a = i as f32 * std::f32::consts::TAU / 10.0;
                    let p = pt(0.50 + hr * a.cos(), 0.32 + hr * a.sin());
                    if i == 0 {
                        b.move_to(p);
                    } else {
                        b.line_to(p);
                    }
                }
                b.move_to(pt(0.20, 0.82));
                b.line_to(pt(0.26, 0.62));
                b.line_to(pt(0.74, 0.62));
                b.line_to(pt(0.80, 0.82));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::ExpandMore => {
                // Downward expansion chevron 'v'
                let mut b = PathBuilder::stroke(px(stroke_w));
                b.move_to(pt(0.26, 0.38));
                b.line_to(pt(0.50, 0.62));
                b.line_to(pt(0.74, 0.38));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::ExpandLess => {
                // Upward expansion chevron '^'
                let mut b = PathBuilder::stroke(px(stroke_w));
                b.move_to(pt(0.26, 0.62));
                b.line_to(pt(0.50, 0.38));
                b.line_to(pt(0.74, 0.62));
                if let Ok(path) = b.build() {
                    window.paint_path(path, color);
                }
            }
            M3Icon::Spark
            | M3Icon::Memory
            | M3Icon::Cpu
            | M3Icon::Lan
            | M3Icon::Bluetooth
            | M3Icon::MicOff
            | M3Icon::Fullscreen
            | M3Icon::Keyboard
            | M3Icon::Album
            | M3Icon::Pause
            | M3Icon::MusicNote => {
                // Drawn via SVG element
            }
        }
    }

    /// Returns the raw UTF-8 bytes of the official Google Material Symbols SVG (24x24 viewBox).
    #[must_use]
    pub fn svg_bytes(&self) -> &'static [u8] {
        macro_rules! m3_svg {
            ($path:expr) => {
                concat!(
                    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24"><path fill="black" d=""#,
                    $path,
                    r#""/></svg>"#
                )
                .as_bytes()
            };
        }

        match self {
            M3Icon::All => m3_svg!("M4 8h4V4H4v4zm6 12h4v-4h-4v4zm-6 0h4v-4H4v4zm0-6h4v-4H4v4zm6 0h4v-4h-4v4zm6-10v4h4V4h-4zm-6 4h4V4h-4v4zm6 6h4v-4h-4v4zm0 6h4v-4h-4v4z"),
            M3Icon::Shapes => m3_svg!("M12 2l-5.5 9h11zM21.5 16a4.5 4.5 0 1 1-9 0 4.5 4.5 0 0 1 9 0zM3 13.5h8v8H3z"),
            M3Icon::Sliders => m3_svg!("M3 17v2h6v-2H3zM3 5v2h10V5H3zm10 16v-2h8v-2h-8v-2h-2v6h2zM7 9v2H3v2h4v2h2V9H7zm14 4v-2H11v2h10zm-6-4h2V7h4V5h-4V3h-2v6z"),
            M3Icon::Switches => m3_svg!("M17 7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h10c2.76 0 5-2.24 5-5s-2.24-5-5-5zm0 8c-1.66 0-3-1.34-3-3s1.34-3 3-3 3 1.34 3 3-1.34 3-3 3z"),
            M3Icon::SplitButton => m3_svg!("M14 4l2.29 2.29-2.88 2.88 1.42 1.42 2.88-2.88L20 10V4zm-4 0H4v6l2.29-2.29 4.71 4.7V20h2v-8.41l-5.29-5.3z"),
            M3Icon::Button => m3_svg!("M22 9v6c0 1.1-.9 2-2 2h-1l-1.66 4.3c-.22.56-.76.94-1.36.94-.97 0-1.69-.93-1.42-1.87L16 17H4c-1.1 0-2-.9-2-2V9c0-1.1.9-2 2-2h16c1.1 0 2 .9 2 2zm-2 0H4v6h16V9z"),
            M3Icon::Cards => m3_svg!("M20 3H4c-1.1 0-2 .9-2 2v6c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 10H4c-1.1 0-2 .9-2 2v4c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2v-4c0-1.1-.9-2-2-2z"),
            M3Icon::ArrowBack => m3_svg!("M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"),
            M3Icon::ArrowForward => m3_svg!("M12 4l-1.41 1.41L16.17 11H4v2h12.17l-5.58 5.59L12 20l8-8z"),
            M3Icon::Play => m3_svg!("M8 5v14l11-7z"),
            M3Icon::Download => m3_svg!("M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"),
            M3Icon::Add => m3_svg!("M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"),
            M3Icon::Check => m3_svg!("M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"),
            M3Icon::Info => m3_svg!("M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"),
            M3Icon::Ticket => m3_svg!("M22 10V6c0-1.11-.9-2-2-2H4c-1.1 0-1.99.89-1.99 2v4c1.1 0 1.99.9 1.99 2s-.89 2-2 2v4c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2v-4c-1.1 0-2-.9-2-2s.9-2 2-2zm-9 7.5h-2v-2h2v2zm0-4.5h-2v-2h2v2zm0-4.5h-2v-2h2v2z"),
            M3Icon::LightMode => m3_svg!("M12 7c-2.76 0-5 2.24-5 5s2.24 5 5 5 5-2.24 5-5-2.24-5-5-5zM2 13h2c.55 0 1-.45 1-1s-.45-1-1-1H2c-.55 0-1 .45-1 1s.45 1 1 1zm18 0h2c.55 0 1-.45 1-1s-.45-1-1-1h-2c-.55 0-1 .45-1 1s.45 1 1 1zM11 2v2c0 .55.45 1 1 1s1-.45 1-1V2c0-.55-.45-1-1-1s-1 .45-1 1zm0 18v2c0 .55.45 1 1 1s1-.45 1-1v-2c0-.55-.45-1-1-1s-1 .45-1 1zM5.99 4.58c-.39-.39-1.03-.39-1.41 0s-.39 1.03 0 1.41l1.06 1.06c.39.39 1.03.39 1.41 0s.39-1.03 0-1.41L5.99 4.58zm12.37 12.37c-.39-.39-1.03-.39-1.41 0s-.39 1.03 0 1.41l1.06 1.06c.39.39 1.03.39 1.41 0s.39-1.03 0-1.41l-1.06-1.06zm1.06-10.96c.39-.39.39-1.03 0-1.41s-1.03-.39-1.41 0l-1.06 1.06c-.39.39-.39 1.03 0 1.41s1.03.39 1.41 0l1.06-1.06zM7.05 18.36c.39-.39.39-1.03 0-1.41s-1.03-.39-1.41 0l-1.06 1.06c-.39.39-.39 1.03 0 1.41s1.03.39 1.41 0l1.06-1.06z"),
            M3Icon::DarkMode => m3_svg!("M12 3c-4.97 0-9 4.03-9 9s4.03 9 9 9 9-4.03 9-9c0-.46-.04-.92-.1-1.36-.98 1.37-2.58 2.26-4.4 2.26-2.98 0-5.4-2.42-5.4-5.4 0-1.81.89-3.42 2.26-4.4-.44-.06-.9-.1-1.36-.1z"),
            M3Icon::Menu => m3_svg!("M3 18h18v-2H3v2zm0-5h18v-2H3v2zm0-7v2h18V6H3z"),
            M3Icon::Visibility => m3_svg!("M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"),
            M3Icon::ContentCopy => m3_svg!("M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"),
            M3Icon::Edit => m3_svg!("M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zM20.71 7.04c.39-.39.39-1.02 0-1.41l-2.34-2.34c-.39-.39-1.02-.39-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"),
            M3Icon::Cloud => m3_svg!("M19.35 10.04C18.67 6.59 15.64 4 12 4 9.11 4 6.6 5.64 5.35 8.04 2.34 8.36 0 10.91 0 14c0 3.31 2.69 6 6 6h13c2.76 0 5-2.24 5-5 0-2.64-2.05-4.78-4.65-4.96z"),
            M3Icon::ChevronRight => m3_svg!("M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"),
            M3Icon::MoreVert => m3_svg!("M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z"),
            M3Icon::Refresh => m3_svg!("M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"),
            M3Icon::Settings => m3_svg!("M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"),
            M3Icon::Help => m3_svg!("M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 16h-2v-2h2v2zm1.07-7.75l-.9.92C12.45 11.9 12 12.5 12 14h-2v-.5c0-1.1.45-2.1 1.17-2.83l1.24-1.26c.37-.36.59-.86.59-1.41 0-1.1-.9-2-2-2s-2 .9-2 2H7c0-2.76 2.24-5 5-5s5 2.24 5 5c0 1.04-.42 1.99-1.07 2.75z"),
            M3Icon::Person => m3_svg!("M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"),
            M3Icon::ExpandMore => m3_svg!("M16.59 8.59L12 13.17 7.41 8.59 6 10l6 6 6-6z"),
            M3Icon::ExpandLess => m3_svg!("M12 8l-6 6 1.41 1.41L12 10.83l4.59 4.58L18 14z"),
            M3Icon::Spark => br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 381 381" width="24" height="24"><path fill="black" d="m 190.48 352.21 C 177.82 268.69 112.27 203.13 28.74 190.48 C 112.27 177.82 177.82 112.27 190.48 28.74 c 12.66 83.52 78.21 149.08 161.73 161.73 c -83.52 12.66 -149.08 78.21 -161.73 161.73 z"/></svg>"#,
            M3Icon::Memory => m3_svg!("M15 9H9v6h6V9zm-2 4h-2v-2h2v2zm8-2V9h-2V7c0-1.1-.9-2-2-2h-2V3h-2v2h-2V3H9v2H7c-1.1 0-2 .9-2 2v2H3v2h2v2H3v2h2v2c0 1.1.9 2 2 2h2v2h2v-2h2v2h2v-2h2c1.1 0 2-.9 2-2v-2h2v-2h-2v-2h2zm-4 6H7V7h10v10z"),
            M3Icon::Cpu => m3_svg!("M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 14h-2v-2h2v2zm0-4h-2V7h2v5z"),
            M3Icon::Lan => m3_svg!("M13 22h8v-7h-3v-4h-4V7h2V2H8v5h2v4H6v4H3v7h8v-7H8v-2h8v2h-3v5zM10 4h4v1h-4V4zM5 17h4v3H5v-3zm10 0h4v3h-4v-3z"),
            M3Icon::Bluetooth => m3_svg!("M17.71 7.71L12 2h-1v7.59L6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 11 14.41V22h1l5.71-5.71-4.3-4.29 4.3-4.29zM13 5.83l1.88 1.88L13 9.59V5.83zm1.88 10.46L13 18.17v-3.76l1.88 1.88z"),
            M3Icon::MicOff => m3_svg!("M19 11h-1.7c0 .74-.16 1.43-.43 2.05l1.23 1.23c.56-1 .9-2.14.9-3.28zm-4.02.17c0-.06.02-.11.02-.17V5c0-1.66-1.34-3-3-3S9 3.34 9 5v.18l5.98 5.99zM4.27 3L3 4.27l6.01 6.01V11c0 1.66 1.33 3 2.99 3 .22 0 .44-.03.65-.08l1.66 1.66c-.71.33-1.5.52-2.31.52-2.76 0-5.3-2.1-5.3-5.1H5c0 3.41 2.72 6.23 6 6.72V21h2v-3.28c.91-.13 1.77-.45 2.54-.9L19.73 21 21 19.73 4.27 3z"),
            M3Icon::Fullscreen => m3_svg!("M7 14H5v5h5v-2H7v-3zm-2-4h2V7h3V5H5v5zm12 7h-3v2h5v-5h-2v3zM14 5v2h3v3h2V5h-5z"),
            M3Icon::Keyboard => m3_svg!("M20 5H4c-1.1 0-1.99.9-1.99 2L2 17c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm-9 3h2v2h-2V8zm0 3h2v2h-2v-2zM8 8h2v2H8V8zm0 3h2v2H8v-2zm-1 2H5v-2h2v2zm0-3H5V8h2v2zm9 7H8v-2h8v2zm0-4h-2v-2h2v2zm0-3h-2V8h2v2zm3 3h-2v-2h2v2zm0-3h-2V8h2v2z"),
            M3Icon::Album => m3_svg!("M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 14.5c-2.49 0-4.5-2.01-4.5-4.5S9.51 7.5 12 7.5s4.5 2.01 4.5 4.5-2.01 4.5-4.5 4.5zm0-5.5c-.55 0-1 .45-1 1s.45 1 1 1 1-.45 1-1-.45-1-1-1z"),
            M3Icon::Pause => m3_svg!("M6 19h4V5H6v14zm8-14v14h4V5h-4z"),
            M3Icon::MusicNote => m3_svg!("M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"),
        }
    }

    /// Produce a GPUI native SVG element rendering the official Google Material Symbol at `size_px` with `color`.
    pub fn element(&self, size_px: f32, color: Rgba) -> impl IntoElement {
        svg()
            .data(self.svg_bytes())
            .size(px(size_px))
            .text_color(color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_variants_are_distinct() {
        let icons = [
            M3Icon::All,
            M3Icon::Shapes,
            M3Icon::Sliders,
            M3Icon::Switches,
            M3Icon::SplitButton,
            M3Icon::Button,
            M3Icon::Cards,
            M3Icon::ArrowBack,
            M3Icon::ArrowForward,
            M3Icon::Play,
            M3Icon::Download,
            M3Icon::Add,
            M3Icon::Check,
            M3Icon::Info,
            M3Icon::Ticket,
            M3Icon::LightMode,
            M3Icon::DarkMode,
            M3Icon::Menu,
            M3Icon::Visibility,
            M3Icon::ContentCopy,
            M3Icon::Edit,
            M3Icon::Cloud,
            M3Icon::ChevronRight,
            M3Icon::MoreVert,
            M3Icon::Refresh,
            M3Icon::Settings,
            M3Icon::Help,
            M3Icon::Person,
            M3Icon::ExpandMore,
            M3Icon::ExpandLess,
            M3Icon::Spark,
            M3Icon::Memory,
            M3Icon::Cpu,
            M3Icon::Lan,
            M3Icon::Bluetooth,
            M3Icon::MicOff,
            M3Icon::Fullscreen,
            M3Icon::Keyboard,
            M3Icon::Album,
            M3Icon::Pause,
            M3Icon::MusicNote,
        ];
        assert_eq!(icons.len(), 41);
        for (i, a) in icons.iter().enumerate() {
            for (j, b) in icons.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn icon_svg_bytes_are_valid_svg() {
        let icons = [
            M3Icon::All,
            M3Icon::Shapes,
            M3Icon::Sliders,
            M3Icon::Switches,
            M3Icon::SplitButton,
            M3Icon::Button,
            M3Icon::Cards,
            M3Icon::ArrowBack,
            M3Icon::ArrowForward,
            M3Icon::Play,
            M3Icon::Download,
            M3Icon::Add,
            M3Icon::Check,
            M3Icon::Info,
            M3Icon::Ticket,
            M3Icon::LightMode,
            M3Icon::DarkMode,
            M3Icon::Menu,
            M3Icon::Visibility,
            M3Icon::ContentCopy,
            M3Icon::Edit,
            M3Icon::Cloud,
            M3Icon::ChevronRight,
            M3Icon::MoreVert,
            M3Icon::Refresh,
            M3Icon::Settings,
            M3Icon::Help,
            M3Icon::Person,
            M3Icon::ExpandMore,
            M3Icon::ExpandLess,
            M3Icon::Spark,
            M3Icon::Memory,
            M3Icon::Cpu,
            M3Icon::Lan,
            M3Icon::Bluetooth,
            M3Icon::MicOff,
            M3Icon::Fullscreen,
            M3Icon::Keyboard,
            M3Icon::Album,
            M3Icon::Pause,
            M3Icon::MusicNote,
        ];
        for icon in icons {
            let bytes = icon.svg_bytes();
            assert!(bytes.starts_with(b"<svg"));
            assert!(bytes.ends_with(b"</svg>"));
            let text = std::str::from_utf8(bytes).expect("valid utf8 svg");
            assert!(text.contains("viewBox="));
            assert!(text.contains("<path fill=\"black\" d=\""));
        }
    }
}
