use requiem_ui::widget::primitives;
use skia_safe::image::CachingHint;
use std::cell::RefCell;
use std::collections::HashMap;

use skia_safe::{AlphaType, Color, ColorType, Image, ImageInfo};

const PALETTE_SIZE: usize = 5;
const QUALITY: usize = 10;

thread_local! {
    static CACHE: RefCell<HashMap<String, Color>> = RefCell::new(HashMap::new());
}

pub fn primary_color(url: &str, image: &Image) -> Color {
    if let Some(color) = CACHE.with(|c| c.borrow().get(url).copied()) {
        return color;
    }
    let color = dominant_color(image);
    CACHE.with(|c| c.borrow_mut().insert(url.to_string(), color));
    color
}

pub fn default_avatar_primary_color(background: Color) -> Color {
    thread_local! {
        static DEFAULTS: RefCell<HashMap<u32, Color>> = RefCell::new(HashMap::new());
    }
    let key = (u32::from(background.a()) << 24)
        | (u32::from(background.r()) << 16)
        | (u32::from(background.g()) << 8)
        | u32::from(background.b());
    if let Some(color) = DEFAULTS.with(|d| d.borrow().get(&key).copied()) {
        return color;
    }
    let color = render_default_avatar(background)
        .map(|image| dominant_color(&image))
        .unwrap_or(background);
    DEFAULTS.with(|d| d.borrow_mut().insert(key, color));
    color
}

/// Discord's default avatar: the logo in white on `background`.
pub fn default_avatar_image(background: Color) -> Option<Image> {
    render_default_avatar(background)
}

fn render_default_avatar(background: Color) -> Option<Image> {
    primitives::render_fallback_avatar(background, super::DEFAULT_AVATAR_ICON, 256)
}

pub fn dominant_color(image: &Image) -> Color {
    let Some(rgba) = read_rgba(image) else {
        return Color::BLACK;
    };
    let pixels = sample_pixels(&rgba, QUALITY);
    match quantize(&pixels, PALETTE_SIZE).and_then(|palette| palette.first().copied()) {
        Some([r, g, b]) => Color::from_rgb(r, g, b),
        None => Color::BLACK,
    }
}

fn read_rgba(image: &Image) -> Option<Vec<u8>> {
    let (width, height) = (image.width(), image.height());
    if width <= 0 || height <= 0 {
        return None;
    }
    let info = ImageInfo::new(
        (width, height),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let row_bytes = width as usize * 4;
    let mut pixels = vec![0u8; row_bytes * height as usize];
    image
        .read_pixels(
            &info,
            &mut pixels,
            row_bytes,
            (0, 0),
            CachingHint::Allow,
        )
        .then_some(pixels)
}

fn sample_pixels(rgba: &[u8], quality: usize) -> Vec<[u8; 3]> {
    let count = rgba.len() / 4;
    (0..count)
        .step_by(quality)
        .map(|i| &rgba[i * 4..i * 4 + 4])
        .filter(|p| p[3] >= 125 && !(p[0] > 250 && p[1] > 250 && p[2] > 250))
        .map(|p| [p[0], p[1], p[2]])
        .collect()
}

const SIGBITS: u32 = 5;
const RSHIFT: u32 = 8 - SIGBITS;
const MAX_ITERATIONS: usize = 1000;
const FRACT_BY_POPULATIONS: f64 = 0.75;

const SIDE: usize = 1 << SIGBITS;

fn color_index(r: i64, g: i64, b: i64) -> usize {
    ((r << (2 * SIGBITS)) + (g << SIGBITS) + b) as usize
}

#[derive(Clone, Copy)]
struct VBox {
    r1: i64,
    r2: i64,
    g1: i64,
    g2: i64,
    b1: i64,
    b2: i64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Axis {
    R,
    G,
    B,
}

impl VBox {
    fn volume(&self) -> i64 {
        (self.r2 - self.r1 + 1) * (self.g2 - self.g1 + 1) * (self.b2 - self.b1 + 1)
    }

    fn count(&self, histo: &[u32]) -> i64 {
        let mut npix = 0i64;
        for i in self.r1..=self.r2 {
            for j in self.g1..=self.g2 {
                for k in self.b1..=self.b2 {
                    npix += i64::from(histo[color_index(i, j, k)]);
                }
            }
        }
        npix
    }

    fn avg(&self, histo: &[u32]) -> [u8; 3] {
        let mult = f64::from(1u32 << (8 - SIGBITS));
        let (mut ntot, mut rsum, mut gsum, mut bsum) = (0f64, 0f64, 0f64, 0f64);
        for i in self.r1..=self.r2 {
            for j in self.g1..=self.g2 {
                for k in self.b1..=self.b2 {
                    let hval = f64::from(histo[color_index(i, j, k)]);
                    ntot += hval;
                    rsum += hval * (i as f64 + 0.5) * mult;
                    gsum += hval * (j as f64 + 0.5) * mult;
                    bsum += hval * (k as f64 + 0.5) * mult;
                }
            }
        }

        if ntot > 0.0 {
            [
                (rsum / ntot) as u8,
                (gsum / ntot) as u8,
                (bsum / ntot) as u8,
            ]
        } else {
            let mid = |a: i64, b: i64| (mult * (a + b + 1) as f64 / 2.0) as u8;
            [
                mid(self.r1, self.r2),
                mid(self.g1, self.g2),
                mid(self.b1, self.b2),
            ]
        }
    }

    fn range(&self, axis: Axis) -> (i64, i64) {
        match axis {
            Axis::R => (self.r1, self.r2),
            Axis::G => (self.g1, self.g2),
            Axis::B => (self.b1, self.b2),
        }
    }

    fn set_range(&mut self, axis: Axis, lo: i64, hi: i64) {
        match axis {
            Axis::R => (self.r1, self.r2) = (lo, hi),
            Axis::G => (self.g1, self.g2) = (lo, hi),
            Axis::B => (self.b1, self.b2) = (lo, hi),
        }
    }
}

fn histogram(pixels: &[[u8; 3]]) -> Vec<u32> {
    let mut histo = vec![0u32; 1 << (3 * SIGBITS)];
    for p in pixels {
        let [r, g, b] = p.map(|c| i64::from(c >> RSHIFT));
        histo[color_index(r, g, b)] += 1;
    }
    histo
}

fn vbox_from_pixels(pixels: &[[u8; 3]]) -> VBox {
    let (mut rmin, mut rmax, mut gmin, mut gmax, mut bmin, mut bmax) =
        (1_000_000i64, 0i64, 1_000_000i64, 0i64, 1_000_000i64, 0i64);
    for p in pixels {
        let [r, g, b] = p.map(|c| i64::from(c >> RSHIFT));
        if r < rmin {
            rmin = r;
        } else if r > rmax {
            rmax = r;
        }
        if g < gmin {
            gmin = g;
        } else if g > gmax {
            gmax = g;
        }
        if b < bmin {
            bmin = b;
        } else if b > bmax {
            bmax = b;
        }
    }
    VBox {
        r1: rmin,
        r2: rmax,
        g1: gmin,
        g2: gmax,
        b1: bmin,
        b2: bmax,
    }
}

fn median_cut(histo: &[u32], vbox: &VBox) -> Option<(VBox, Option<VBox>)> {
    let count = vbox.count(histo);
    if count == 0 {
        return None;
    }
    let (rw, gw, bw) = (
        vbox.r2 - vbox.r1 + 1,
        vbox.g2 - vbox.g1 + 1,
        vbox.b2 - vbox.b1 + 1,
    );
    let maxw = rw.max(gw).max(bw);
    if count == 1 {
        return Some((*vbox, None));
    }
    let axis = if maxw == rw {
        Axis::R
    } else if maxw == gw {
        Axis::G
    } else {
        Axis::B
    };

    let mut partialsum: [Option<i64>; SIDE] = [None; SIDE];
    let mut total = 0i64;
    let (lo, hi) = vbox.range(axis);
    for i in lo..=hi {
        let mut sum = 0i64;
        match axis {
            Axis::R => {
                for j in vbox.g1..=vbox.g2 {
                    for k in vbox.b1..=vbox.b2 {
                        sum += i64::from(histo[color_index(i, j, k)]);
                    }
                }
            }
            Axis::G => {
                for j in vbox.r1..=vbox.r2 {
                    for k in vbox.b1..=vbox.b2 {
                        sum += i64::from(histo[color_index(j, i, k)]);
                    }
                }
            }
            Axis::B => {
                for j in vbox.r1..=vbox.r2 {
                    for k in vbox.g1..=vbox.g2 {
                        sum += i64::from(histo[color_index(j, k, i)]);
                    }
                }
            }
        }
        total += sum;
        partialsum[i as usize] = Some(total);
    }
    let lookaheadsum: [Option<i64>; SIDE] = partialsum.map(|d| d.map(|d| total - d));

    let truthy = |arr: &[Option<i64>; SIDE], index: i64| {
        index >= 0 && (index as usize) < SIDE && arr[index as usize].is_some_and(|v| v != 0)
    };

    for i in lo..=hi {
        if partialsum[i as usize].is_some_and(|p| p as f64 > total as f64 / 2.0) {
            let (left, right) = ((i - lo) as f64, (hi - i) as f64);
            let mut d2: i64 = if left <= right {
                (hi - 1).min((i as f64 + right / 2.0) as i64)
            } else {
                lo.max((i as f64 - 1.0 - left / 2.0) as i64)
            };

            while !truthy(&partialsum, d2) {
                d2 += 1;
            }
            let mut count2 = lookaheadsum
                .get(d2 as usize)
                .copied()
                .flatten()
                .unwrap_or(0);
            while count2 == 0 && truthy(&partialsum, d2 - 1) {
                d2 -= 1;
                count2 = lookaheadsum[d2 as usize].unwrap_or(0);
            }
            let (mut vbox1, mut vbox2) = (*vbox, *vbox);
            vbox1.set_range(axis, lo, d2);
            vbox2.set_range(axis, d2 + 1, hi);
            return Some((vbox1, Some(vbox2)));
        }
    }

    Some((*vbox, None))
}

struct PQueue<F: Fn(&VBox) -> i64> {
    contents: Vec<VBox>,
    sorted: bool,
    key: F,
}

impl<F: Fn(&VBox) -> i64> PQueue<F> {
    fn new(key: F) -> Self {
        Self {
            contents: Vec::new(),
            sorted: false,
            key,
        }
    }

    fn push(&mut self, vbox: VBox) {
        self.contents.push(vbox);
        self.sorted = false;
    }

    fn pop(&mut self) -> Option<VBox> {
        if !self.sorted {
            let key = &self.key;
            self.contents.sort_by_key(|v| key(v));
            self.sorted = true;
        }
        self.contents.pop()
    }

    fn size(&self) -> usize {
        self.contents.len()
    }
}

fn iterate<F: Fn(&VBox) -> i64>(queue: &mut PQueue<F>, histo: &[u32], target: f64) {
    let mut ncolors = 1usize;
    let mut niters = 0usize;
    while niters < MAX_ITERATIONS {
        let Some(vbox) = queue.pop() else { return };
        if vbox.count(histo) == 0 {
            queue.push(vbox);
            niters += 1;
            continue;
        }
        let Some((vbox1, vbox2)) = median_cut(histo, &vbox) else {
            return;
        };
        queue.push(vbox1);
        if let Some(vbox2) = vbox2 {
            queue.push(vbox2);
            ncolors += 1;
        }
        if ncolors as f64 >= target {
            return;
        }
        let over = niters > MAX_ITERATIONS;
        niters += 1;
        if over {
            return;
        }
    }
}

fn quantize(pixels: &[[u8; 3]], max_colors: usize) -> Option<Vec<[u8; 3]>> {
    if pixels.is_empty() || !(2..=256).contains(&max_colors) {
        return None;
    }
    let histo = histogram(pixels);
    let histo_ref = &histo;

    let mut by_count = PQueue::new(|v: &VBox| v.count(histo_ref));
    by_count.push(vbox_from_pixels(pixels));
    iterate(
        &mut by_count,
        &histo,
        FRACT_BY_POPULATIONS * max_colors as f64,
    );

    let mut by_product = PQueue::new(|v: &VBox| v.count(histo_ref) * v.volume());
    while let Some(vbox) = by_count.pop() {
        by_product.push(vbox);
    }
    let target = max_colors as f64 - by_product.size() as f64;
    iterate(&mut by_product, &histo, target);

    let mut palette = Vec::with_capacity(by_product.size());
    while let Some(vbox) = by_product.pop() {
        palette.push(vbox.avg(&histo));
    }
    Some(palette)
}

#[cfg(test)]
mod tests {
    use crate::test_assets;
    use crate::ui;
    use super::*;

    #[test]
    fn the_default_avatar_has_the_logo_on_its_background() {
        if !test_assets::install() {
            return;
        }
        let background = ui::DEFAULT_AVATAR_COLORS[0];
        let image = default_avatar_image(background).expect("default avatar renders");
        let rgba = read_rgba(&image).expect("pixels read");
        let white = rgba
            .chunks_exact(4)
            .filter(|p| p[0] > 240 && p[1] > 240 && p[2] > 240)
            .count();
        // The corner is background; the logo covers a real share of the middle.
        assert_eq!(&rgba[..3], &[background.r(), background.g(), background.b()]);
        assert!(white > rgba.len() / 4 / 20, "no logo drawn: {white} white pixels");
    }

    #[test]
    fn uniform_image() {
        let pixels = vec![[116u8, 124, 140]; 400];
        assert_eq!(
            quantize(&pixels, 5),
            Some(vec![
                [116, 124, 140],
                [120, 124, 140],
                [120, 124, 140],
                [120, 124, 140],
                [120, 124, 140]
            ])
        );
    }

    #[test]
    fn gray_field_with_light_edge_pixels() {
        let mut pixels = vec![[200u8, 205, 210]; 5];
        pixels.extend(vec![[116u8, 124, 140]; 900]);
        assert_eq!(
            quantize(&pixels, 5),
            Some(vec![
                [204, 204, 212],
                [116, 124, 140],
                [140, 124, 140],
                [136, 168, 176],
                [136, 124, 180]
            ])
        );
    }

    #[test]
    fn palette_order_is_count_times_volume() {
        let mut pixels = vec![[220u8, 30, 30]; 700];
        pixels.extend(vec![[30u8, 30, 220]; 300]);
        assert_eq!(
            quantize(&pixels, 5),
            Some(vec![
                [28, 28, 220],
                [220, 28, 28],
                [72, 28, 224],
                [172, 28, 128],
                [224, 28, 28]
            ])
        );
    }

    #[test]
    fn white_and_transparent_pixels_are_ignored() {
        let mut rgba = Vec::new();
        for i in 0..1000 {
            let px: [u8; 4] = match i % 3 {
                0 => [255, 255, 255, 255],
                1 => [10, 200, 30, 50],
                _ => [200, 40, 40, 255],
            };
            rgba.extend_from_slice(&px);
        }
        let pixels = sample_pixels(&rgba, 1);
        assert!(pixels.iter().all(|p| *p == [200, 40, 40]));
    }

    fn lcg_pixels(seed: u32) -> Vec<[u8; 3]> {
        let mut state = seed;
        let mut rand = move || {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            state
        };
        let n = 50 + rand() % 3000;
        let k = 1 + rand() % 6;
        let clusters: Vec<([u32; 3], u32)> = (0..k)
            .map(|_| ([rand() % 256, rand() % 256, rand() % 256], rand() % 61))
            .collect();
        (0..n)
            .map(|_| {
                let (center, spread) = clusters[(rand() % k) as usize];
                center.map(|v| {
                    (v as i64 + (rand() % (2 * spread + 1)) as i64 - spread as i64).clamp(0, 255)
                        as u8
                })
            })
            .collect()
    }

    #[test]
    fn matches_quantize_js_exactly() {
        let expected: Vec<Option<Vec<[u8; 3]>>> =
            serde_json::from_str(include_str!("avatar_color_quantize_js.json"))
                .expect("fixture parses");
        for (i, expected) in expected.into_iter().enumerate() {
            let seed = i as u32 + 1;
            assert_eq!(quantize(&lcg_pixels(seed), 5), expected, "seed {seed}");
        }
    }

    #[test]
    fn grey_default_avatar_matches_live_banner() {
        let color = default_avatar_primary_color(ui::DEFAULT_AVATAR_COLORS[1]);
        assert_eq!((color.r(), color.g(), color.b()), (116, 124, 140));
    }

    #[test]
    fn empty_input_has_no_palette() {
        assert!(quantize(&[], 5).is_none());
    }
}
