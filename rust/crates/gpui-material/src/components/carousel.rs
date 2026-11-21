//! Material 3 carousel (center-aligned multi-browse / hero layout).
//!
//! Items sit on *keylines*: one large focal item in the middle, then medium and small
//! items towards the edges, then extra-small and gone. As the carousel scrolls, every
//! item's **mask** interpolates between the keylines it travels across, while its content
//! keeps the large size and stays centered on the mask, so it is cropped from both
//! sides (the M3 "masking" effect, with a natural parallax). Masks use the extra-large
//! corner (28dp), capped at half their width.
//!
//! Sizing follows `MultiBrowseCarouselStrategy`: the small item is a third of the large
//! one clamped to 40..=56dp, medium items fill what's left between them.

use gpui::{AnyElement, IntoElement, ParentElement, Styled, div, px};

/// Layout metrics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarouselMetrics {
    /// Height of the carousel (and of every item).
    pub height: f32,
    /// Width of the large (focal) item.
    pub large_width: f32,
    /// Small item width bounds (M3: 40..=56dp).
    pub small_min: f32,
    /// See [`small_min`](Self::small_min).
    pub small_max: f32,
    /// Extra-small item, the last sliver before an item leaves (M3: 10dp).
    pub extra_small: f32,
    /// Space between items (M3: 8dp).
    pub gap: f32,
    /// Mask corner radius (M3 extra large: 28dp).
    pub corner: f32,
}

impl Default for CarouselMetrics {
    fn default() -> Self {
        Self {
            height: 200.0,
            large_width: 300.0,
            small_min: 40.0,
            small_max: 56.0,
            extra_small: 10.0,
            gap: 8.0,
            corner: 28.0,
        }
    }
}

/// Where one item is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarouselItem {
    /// Index into the caller's items.
    pub index: usize,
    /// Left edge of the mask, relative to the carousel.
    pub x: f32,
    /// Mask width.
    pub width: f32,
    /// Unmasked content width (the large item width).
    pub content_width: f32,
    /// Content offset inside the mask (negative: cropped on the left).
    pub content_x: f32,
    /// 1 on the focal keyline, 0 from the small keyline outwards.
    pub focus: f32,
    /// Corner radius for this mask.
    pub corner: f32,
}

/// Keylines for a given width: `(center, width)` from left to right, and the focal index.
#[derive(Clone, Debug, PartialEq)]
pub struct Keylines {
    /// Keyline centers and mask widths, left to right.
    pub lines: Vec<(f32, f32)>,
    /// Index of the large, focal keyline.
    pub focal: usize,
}

impl Keylines {
    /// Center-aligned keylines filling `width` with `m`.
    pub fn center_aligned(width: f32, m: &CarouselMetrics) -> Self {
        let large = m.large_width.min(width).max(m.small_min);
        let small = (large / 3.0).clamp(m.small_min, m.small_max.max(m.small_min));
        // one side: from the focal item outwards
        let mut side = Vec::new();
        let mut rem = (width - large) / 2.0 - m.gap;
        if rem > small {
            rem -= small + m.gap;
            while rem > large + m.gap {
                side.push(large);
                rem -= large + m.gap;
            }
            if rem >= small {
                side.push(rem);
            }
            side.push(small);
        } else if rem > m.extra_small {
            side.push(rem.max(m.extra_small));
        }
        side.push(m.extra_small);
        side.push(0.0);
        let mut widths: Vec<f32> = side.iter().rev().copied().collect();
        let focal = widths.len();
        widths.push(large);
        widths.extend(side.iter().copied());
        // centers: focal at width/2, neighbours packed with gaps
        let mut lines = vec![(0.0, 0.0); widths.len()];
        lines[focal] = (width / 2.0, large);
        for i in (0..focal).rev() {
            let (nc, nw) = lines[i + 1];
            let w = widths[i];
            lines[i] = (nc - nw / 2.0 - m.gap - w / 2.0, w);
        }
        for i in focal + 1..widths.len() {
            let (pc, pw) = lines[i - 1];
            let w = widths[i];
            lines[i] = (pc + pw / 2.0 + m.gap + w / 2.0, w);
        }
        Self { lines, focal }
    }

    /// Items to draw for `count` items scrolled to `scroll` (fractional item index at
    /// the focal keyline).
    pub fn arrange(&self, count: usize, scroll: f32, m: &CarouselMetrics) -> Vec<CarouselItem> {
        let mut out = Vec::new();
        let large = self.lines[self.focal].1;
        let small_k = self
            .lines
            .iter()
            .position(|l| l.1 > 0.0 && l.1 <= m.small_max + 0.5)
            .unwrap_or(0);
        let small_dist = (self.focal as f32 - small_k as f32).abs().max(1.0);
        for index in 0..count {
            let q = self.focal as f32 + index as f32 - scroll;
            if q < 0.0 || q > (self.lines.len() - 1) as f32 {
                continue;
            }
            let k = q.floor() as usize;
            let t = q - k as f32;
            let (c0, w0) = self.lines[k];
            let (c1, w1) = self.lines[(k + 1).min(self.lines.len() - 1)];
            let center = c0 + (c1 - c0) * t;
            let width = w0 + (w1 - w0) * t;
            if width < 0.5 {
                continue;
            }
            let focus = (1.0 - (q - self.focal as f32).abs() / small_dist).clamp(0.0, 1.0);
            out.push(CarouselItem {
                index,
                x: center - width / 2.0,
                width,
                content_width: large,
                content_x: (width - large) / 2.0,
                focus,
                corner: m.corner.min(width / 2.0),
            });
        }
        out
    }
}

/// Spring for the scroll position: snaps to whole items, M3 `CarouselSnapHelper` style.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarouselScroll {
    /// Current fractional index at the focal keyline.
    pub value: f32,
    /// Index the carousel is settling on.
    pub target: f32,
    velocity: f32,
}

impl CarouselScroll {
    /// Starts settled on `index`.
    #[must_use]
    pub fn new(index: usize) -> Self {
        Self {
            value: index as f32,
            target: index as f32,
            velocity: 0.0,
        }
    }

    /// Moves the target by `delta` items (wheel, arrow keys), clamped to `count`.
    pub fn nudge(&mut self, delta: i32, count: usize) {
        let max = count.saturating_sub(1) as f32;
        self.target = (self.target.round() + delta as f32).clamp(0.0, max);
    }

    /// Jumps the target to an item.
    pub fn go_to(&mut self, index: usize, count: usize) {
        self.target = (index.min(count.saturating_sub(1))) as f32;
    }

    /// The item the carousel shows (or is settling on).
    pub fn current(&self) -> usize {
        self.target.round().max(0.0) as usize
    }

    /// Advances the spring, returns true while moving.
    pub fn step(&mut self, dt: f32) -> bool {
        let dt = dt.clamp(0.0, 0.05);
        // near critically damped, ~250ms settle
        let (k, c) = (220.0, 2.0 * 220f32.sqrt());
        let a = -k * (self.value - self.target) - c * self.velocity;
        self.velocity += a * dt;
        self.value += self.velocity * dt;
        let moving = (self.value - self.target).abs() > 0.001 || self.velocity.abs() > 0.001;
        if !moving {
            self.value = self.target;
            self.velocity = 0.0;
        }
        moving
    }
}

/// Renders a carousel of `count` items: `content(index, item)` builds each item's
/// content at `item.content_width x metrics.height`; it is masked and positioned here.
pub fn render_carousel(
    width: f32,
    count: usize,
    scroll: f32,
    metrics: &CarouselMetrics,
    mut content: impl FnMut(usize, &CarouselItem) -> AnyElement,
) -> impl IntoElement {
    let keylines = Keylines::center_aligned(width, metrics);
    let items = keylines.arrange(count, scroll, metrics);
    let h = metrics.height;
    div()
        .relative()
        .w(px(width))
        .h(px(h))
        .overflow_hidden()
        .children(items.into_iter().map(move |item| {
            let body = content(item.index, &item);
            div()
                .absolute()
                .left(px(item.x))
                .top_0()
                .w(px(item.width))
                .h(px(h))
                .rounded(px(item.corner))
                .overflow_hidden()
                .child(
                    div()
                        .absolute()
                        .left(px(item.content_x))
                        .top_0()
                        .w(px(item.content_width))
                        .h(px(h))
                        .child(body),
                )
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m() -> CarouselMetrics {
        CarouselMetrics {
            height: 180.0,
            large_width: 320.0,
            ..Default::default()
        }
    }

    #[test]
    fn keylines_are_symmetric_and_centered() {
        let k = Keylines::center_aligned(1000.0, &m());
        let (c, w) = k.lines[k.focal];
        assert_eq!((c, w), (500.0, 320.0));
        let n = k.lines.len();
        for i in 0..k.focal {
            let (lc, lw) = k.lines[i];
            let (rc, rw) = k.lines[n - 1 - i];
            assert!((lw - rw).abs() < 0.01);
            assert!(((500.0 - lc) - (rc - 500.0)).abs() < 0.01);
        }
    }

    #[test]
    fn small_is_a_third_clamped() {
        let k = Keylines::center_aligned(1000.0, &m());
        assert!(k.lines.iter().any(|l| (l.1 - 56.0).abs() < 0.01));
    }

    #[test]
    fn focal_item_is_unmasked() {
        let km = m();
        let k = Keylines::center_aligned(1000.0, &km);
        let items = k.arrange(10, 3.0, &km);
        let focal = items.iter().find(|i| i.index == 3).unwrap();
        assert_eq!(focal.width, 320.0);
        assert_eq!(focal.content_x, 0.0);
        assert_eq!(focal.focus, 1.0);
        // neighbours are narrower and cropped from both sides
        let next = items.iter().find(|i| i.index == 4).unwrap();
        assert!(next.width < 320.0 && next.content_x < 0.0);
    }

    #[test]
    fn scroll_interpolates_and_settles() {
        let mut s = CarouselScroll::new(0);
        s.nudge(2, 10);
        let mut frames = 0;
        while s.step(0.016) {
            frames += 1;
            assert!(frames < 200);
        }
        assert_eq!(s.value, 2.0);
        assert_eq!(s.current(), 2);
    }
}
