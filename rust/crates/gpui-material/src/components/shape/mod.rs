//! Material 3 Expressive Shapes & Morphing Engine.
//! Accurate Android 15/16 Jetpack Compose graphics-shapes implementation.

#![allow(missing_docs)]

use gpui::{Bounds, PathBuilder, Pixels, Point, Rgba, Window, point, px, size};

pub mod features;
pub mod geometry;
pub mod material_shapes;
pub mod morph;
pub mod polygon;
pub mod sdf;

pub use features::Feature;
pub use geometry::{
    Cubic, PointExt, direction_vector, distance, interpolate, interpolate_point,
    radial_to_cartesian,
};
pub use material_shapes::MaterialShapes;
pub use morph::Morph;
pub use polygon::{CornerRounding, RoundedPolygon};
pub use sdf::shape_sdf;

/// Number of perimeter sample points used to represent each normalized shape in point mode.
pub const SHAPE_SAMPLE_COUNT: usize = 180;

/// The 35 official Material 3 Expressive shapes from AndroidX.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialShape {
    /// The `Circle` shape.
    Circle,
    /// The `Square` shape.
    Square,
    /// The `Slanted` shape.
    Slanted,
    /// The `Arch` shape.
    Arch,
    /// The `Fan` shape.
    Fan,
    /// The `Arrow` shape.
    Arrow,
    /// The `SemiCircle` shape.
    SemiCircle,
    /// The `Oval` shape.
    Oval,
    /// The `Pill` shape.
    Pill,
    /// The `Triangle` shape.
    Triangle,
    /// The `Diamond` shape.
    Diamond,
    /// The `ClamShell` shape.
    ClamShell,
    /// The `Pentagon` shape.
    Pentagon,
    /// The `Gem` shape.
    Gem,
    /// The `Sunny` shape.
    Sunny,
    /// The `VerySunny` shape.
    VerySunny,
    /// The `Cookie4Sided` shape.
    Cookie4Sided,
    /// The `Cookie6Sided` shape.
    Cookie6Sided,
    /// The `Cookie7Sided` shape.
    Cookie7Sided,
    /// The `Cookie9Sided` shape.
    Cookie9Sided,
    /// The `Cookie12Sided` shape.
    Cookie12Sided,
    /// The `Ghostish` shape.
    Ghostish,
    /// The `Clover4Leaf` shape.
    Clover4Leaf,
    /// The `Clover8Leaf` shape.
    Clover8Leaf,
    /// The `Burst` shape.
    Burst,
    /// The `SoftBurst` shape.
    SoftBurst,
    /// The `Boom` shape.
    Boom,
    /// The `SoftBoom` shape.
    SoftBoom,
    /// The `Flower` shape.
    Flower,
    /// The `Puffy` shape.
    Puffy,
    /// The `PuffyDiamond` shape.
    PuffyDiamond,
    /// The `PixelCircle` shape.
    PixelCircle,
    /// The `PixelTriangle` shape.
    PixelTriangle,
    /// The `Bun` shape.
    Bun,
    /// The `Heart` shape.
    Heart,
}

impl MaterialShape {
    /// All 35 shapes in standard specification order.
    pub const ALL: &'static [MaterialShape] = &[
        MaterialShape::Circle,
        MaterialShape::Square,
        MaterialShape::Slanted,
        MaterialShape::Arch,
        MaterialShape::Fan,
        MaterialShape::Arrow,
        MaterialShape::SemiCircle,
        MaterialShape::Oval,
        MaterialShape::Pill,
        MaterialShape::Triangle,
        MaterialShape::Diamond,
        MaterialShape::ClamShell,
        MaterialShape::Pentagon,
        MaterialShape::Gem,
        MaterialShape::Sunny,
        MaterialShape::VerySunny,
        MaterialShape::Cookie4Sided,
        MaterialShape::Cookie6Sided,
        MaterialShape::Cookie7Sided,
        MaterialShape::Cookie9Sided,
        MaterialShape::Cookie12Sided,
        MaterialShape::Ghostish,
        MaterialShape::Clover4Leaf,
        MaterialShape::Clover8Leaf,
        MaterialShape::Burst,
        MaterialShape::SoftBurst,
        MaterialShape::Boom,
        MaterialShape::SoftBoom,
        MaterialShape::Flower,
        MaterialShape::Puffy,
        MaterialShape::PuffyDiamond,
        MaterialShape::PixelCircle,
        MaterialShape::PixelTriangle,
        MaterialShape::Bun,
        MaterialShape::Heart,
    ];

    /// Construct the official Android rounded polygon definition for this shape.
    pub fn polygon(&self) -> RoundedPolygon {
        match self {
            MaterialShape::Circle => MaterialShapes::circle(),
            MaterialShape::Square => MaterialShapes::square(),
            MaterialShape::Slanted => MaterialShapes::slanted(),
            MaterialShape::Arch => MaterialShapes::arch(),
            MaterialShape::Fan => MaterialShapes::fan(),
            MaterialShape::Arrow => MaterialShapes::arrow(),
            MaterialShape::SemiCircle => MaterialShapes::semi_circle(),
            MaterialShape::Oval => MaterialShapes::oval(),
            MaterialShape::Pill => MaterialShapes::pill(),
            MaterialShape::Triangle => MaterialShapes::triangle(),
            MaterialShape::Diamond => MaterialShapes::diamond(),
            MaterialShape::ClamShell => MaterialShapes::clam_shell(),
            MaterialShape::Pentagon => MaterialShapes::pentagon(),
            MaterialShape::Gem => MaterialShapes::gem(),
            MaterialShape::Sunny => MaterialShapes::sunny(),
            MaterialShape::VerySunny => MaterialShapes::very_sunny(),
            MaterialShape::Cookie4Sided => MaterialShapes::cookie4sided(),
            MaterialShape::Cookie6Sided => MaterialShapes::cookie6sided(),
            MaterialShape::Cookie7Sided => MaterialShapes::cookie7sided(),
            MaterialShape::Cookie9Sided => MaterialShapes::cookie9sided(),
            MaterialShape::Cookie12Sided => MaterialShapes::cookie12sided(),
            MaterialShape::Ghostish => MaterialShapes::ghostish(),
            MaterialShape::Clover4Leaf => MaterialShapes::clover4leaf(),
            MaterialShape::Clover8Leaf => MaterialShapes::clover8leaf(),
            MaterialShape::Burst => MaterialShapes::burst(),
            MaterialShape::SoftBurst => MaterialShapes::soft_burst(),
            MaterialShape::Boom => MaterialShapes::boom(),
            MaterialShape::SoftBoom => MaterialShapes::soft_boom(),
            MaterialShape::Flower => MaterialShapes::flower(),
            MaterialShape::Puffy => MaterialShapes::puffy(),
            MaterialShape::PuffyDiamond => MaterialShapes::puffy_diamond(),
            MaterialShape::PixelCircle => MaterialShapes::pixel_circle(),
            MaterialShape::PixelTriangle => MaterialShapes::pixel_triangle(),
            MaterialShape::Bun => MaterialShapes::bun(),
            MaterialShape::Heart => MaterialShapes::heart(),
        }
    }

    /// Full human-readable PascalCase display name.
    pub const fn name(&self) -> &'static str {
        match self {
            MaterialShape::Circle => "Circle",
            MaterialShape::Square => "Square",
            MaterialShape::Slanted => "Slanted",
            MaterialShape::Arch => "Arch",
            MaterialShape::Fan => "Fan",
            MaterialShape::Arrow => "Arrow",
            MaterialShape::SemiCircle => "SemiCircle",
            MaterialShape::Oval => "Oval",
            MaterialShape::Pill => "Pill",
            MaterialShape::Triangle => "Triangle",
            MaterialShape::Diamond => "Diamond",
            MaterialShape::ClamShell => "ClamShell",
            MaterialShape::Pentagon => "Pentagon",
            MaterialShape::Gem => "Gem",
            MaterialShape::Sunny => "Sunny",
            MaterialShape::VerySunny => "VerySunny",
            MaterialShape::Cookie4Sided => "Cookie4Sided",
            MaterialShape::Cookie6Sided => "Cookie6Sided",
            MaterialShape::Cookie7Sided => "Cookie7Sided",
            MaterialShape::Cookie9Sided => "Cookie9Sided",
            MaterialShape::Cookie12Sided => "Cookie12Sided",
            MaterialShape::Ghostish => "Ghostish",
            MaterialShape::Clover4Leaf => "Clover4Leaf",
            MaterialShape::Clover8Leaf => "Clover8Leaf",
            MaterialShape::Burst => "Burst",
            MaterialShape::SoftBurst => "SoftBurst",
            MaterialShape::Boom => "Boom",
            MaterialShape::SoftBoom => "SoftBoom",
            MaterialShape::Flower => "Flower",
            MaterialShape::Puffy => "Puffy",
            MaterialShape::PuffyDiamond => "PuffyDiamond",
            MaterialShape::PixelCircle => "PixelCircle",
            MaterialShape::PixelTriangle => "PixelTriangle",
            MaterialShape::Bun => "Bun",
            MaterialShape::Heart => "Heart",
        }
    }

    /// Canonical snake_case identifier matching Android resources.
    pub const fn short_name(&self) -> &'static str {
        match self {
            MaterialShape::Circle => "circle",
            MaterialShape::Square => "square",
            MaterialShape::Slanted => "slanted",
            MaterialShape::Arch => "arch",
            MaterialShape::Fan => "fan",
            MaterialShape::Arrow => "arrow",
            MaterialShape::SemiCircle => "semi_circle",
            MaterialShape::Oval => "oval",
            MaterialShape::Pill => "pill",
            MaterialShape::Triangle => "triangle",
            MaterialShape::Diamond => "diamond",
            MaterialShape::ClamShell => "clam_shell",
            MaterialShape::Pentagon => "pentagon",
            MaterialShape::Gem => "gem",
            MaterialShape::Sunny => "sunny",
            MaterialShape::VerySunny => "very_sunny",
            MaterialShape::Cookie4Sided => "cookie_4",
            MaterialShape::Cookie6Sided => "cookie_6",
            MaterialShape::Cookie7Sided => "cookie_7",
            MaterialShape::Cookie9Sided => "cookie_9",
            MaterialShape::Cookie12Sided => "cookie_12",
            MaterialShape::Ghostish => "ghostish",
            MaterialShape::Clover4Leaf => "clover_4",
            MaterialShape::Clover8Leaf => "clover_8",
            MaterialShape::Burst => "burst",
            MaterialShape::SoftBurst => "soft_burst",
            MaterialShape::Boom => "boom",
            MaterialShape::SoftBoom => "soft_boom",
            MaterialShape::Flower => "flower",
            MaterialShape::Puffy => "puffy",
            MaterialShape::PuffyDiamond => "puffy_diamond",
            MaterialShape::PixelCircle => "pixel_circle",
            MaterialShape::PixelTriangle => "pixel_triangle",
            MaterialShape::Bun => "bun",
            MaterialShape::Heart => "heart",
        }
    }

    /// Sample `SHAPE_SAMPLE_COUNT` points along the polygon perimeter for point-mode operations.
    pub fn points(&self) -> &'static [[f32; 2]; SHAPE_SAMPLE_COUNT] {
        static SHAPES_CACHE: std::sync::OnceLock<[[[f32; 2]; SHAPE_SAMPLE_COUNT]; 35]> =
            std::sync::OnceLock::new();
        let cache = SHAPES_CACHE.get_or_init(|| {
            let mut all = [[[0.0f32; 2]; SHAPE_SAMPLE_COUNT]; 35];
            for (i, shape) in MaterialShape::ALL.iter().enumerate() {
                all[i] = sample_polygon_points(&shape.polygon());
            }
            all
        });
        &cache[*self as usize]
    }

    /// Evaluates the signed distance field (SDF) of this shape at local normalized coordinate `(x, y)`.
    ///
    /// The shape is centered at `(0.0, 0.0)` with a normalized half-extent of ~0.5.
    /// Returns:
    /// - `< 0.0`: strictly inside the shape
    /// - `== 0.0`: on the perimeter
    /// - `> 0.0`: outside the shape
    pub fn sdf(&self, x: f32, y: f32) -> f32 {
        shape_sdf(*self, x, y)
    }
}

/// Sample `SHAPE_SAMPLE_COUNT` perimeter coordinates from a `RoundedPolygon`.
pub fn sample_polygon_points(poly: &RoundedPolygon) -> [[f32; 2]; SHAPE_SAMPLE_COUNT] {
    let mut out = [[0.0f32; 2]; SHAPE_SAMPLE_COUNT];
    if poly.cubics.is_empty() {
        return out;
    }
    let n = poly.cubics.len();
    for i in 0..SHAPE_SAMPLE_COUNT {
        let t_global = i as f32 / SHAPE_SAMPLE_COUNT as f32;
        let c_frac = t_global * (n as f32);
        let idx = (c_frac.floor() as usize).min(n - 1);
        let t = c_frac - idx as f32;
        let pt = poly.cubics[idx].point_on_curve(t);
        out[i] = [pt.x, pt.y];
    }
    out
}

/// Draw cubic Bézier segments onto GPUI `Window` within `bounds`.
pub fn draw_cubics(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    cubics: &[Cubic],
    rotation_deg: f32,
    color: Rgba,
) {
    let viewport = window.viewport_size();
    if bounds.right() < px(0.0)
        || bounds.left() > viewport.width
        || bounds.bottom() < px(0.0)
        || bounds.top() > viewport.height
    {
        return;
    }
    if cubics.is_empty() {
        return;
    }

    let center = bounds.center();
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let rot_rad = rotation_deg.to_radians();
    let (sin_r, cos_r) = rot_rad.sin_cos();

    let transform_pt = |p: Point<f32>| -> Point<Pixels> {
        let x = p.x - 0.5;
        let y = p.y - 0.5;
        let (rx, ry) = if rotation_deg.abs() < 1e-4 {
            (x, y)
        } else {
            (x * cos_r - y * sin_r, x * sin_r + y * cos_r)
        };
        point(center.x + px(rx * w), center.y + px(ry * h))
    };

    let mut builder = PathBuilder::fill();
    let p0 = transform_pt(cubics[0].anchor0);
    builder.move_to(p0);

    for c in cubics {
        let to = transform_pt(c.anchor1);
        let ctrl_a = transform_pt(c.control0);
        let ctrl_b = transform_pt(c.control1);
        builder.cubic_bezier_to(to, ctrl_a, ctrl_b);
    }
    builder.close();

    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

/// Draw a static Material 3 Expressive shape filled with the given color.
pub fn draw_shape(window: &mut Window, bounds: Bounds<Pixels>, shape: MaterialShape, color: Rgba) {
    draw_shape_rotated(window, bounds, shape, 0.0, color);
}

/// Draw a static Material 3 Expressive shape rotated by `rotation_deg` degrees clockwise.
pub fn draw_shape_rotated(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    shape: MaterialShape,
    rotation_deg: f32,
    color: Rgba,
) {
    let poly = shape.polygon();
    draw_cubics(window, bounds, &poly.cubics, rotation_deg, color);
}

/// Draw a `RoundedPolygon` directly.
pub fn draw_polygon(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    polygon: &RoundedPolygon,
    color: Rgba,
) {
    draw_cubics(window, bounds, &polygon.cubics, 0.0, color);
}

/// Draw an animated shape morph between `from` and `to` at progress `t` [0.0, 1.0].
pub fn draw_morph(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    from: MaterialShape,
    to: MaterialShape,
    progress: f32,
    color: Rgba,
) {
    if from == to {
        draw_shape(window, bounds, to, color);
        return;
    }
    let p1 = from.polygon();
    let p2 = to.polygon();
    let morph = Morph::new(&p1, &p2);
    let cubics = morph.as_cubics(progress.clamp(0.0, 1.0));
    draw_cubics(window, bounds, &cubics, 0.0, color);
}

/// Draw an interpolated morph between two shapes with separate monotonic morph progress and physical bounce scaling.
pub fn draw_shape_morph_with_bounce(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    from: MaterialShape,
    to: MaterialShape,
    morph_t: f32,
    spring_pos: f32,
    color: Rgba,
) {
    let scale = if (spring_pos - 1.0).abs() > 1e-4 {
        1.0 + (spring_pos - 1.0) * 0.12
    } else {
        1.0
    };
    let target_bounds = if (scale - 1.0).abs() > 1e-4 {
        let cx = bounds.left() + bounds.size.width / 2.0;
        let cy = bounds.top() + bounds.size.height / 2.0;
        let sw = bounds.size.width * scale;
        let sh = bounds.size.height * scale;
        Bounds {
            origin: point(cx - sw / 2.0, cy - sh / 2.0),
            size: size(sw, sh),
        }
    } else {
        bounds
    };
    draw_morph(window, target_bounds, from, to, morph_t, color);
}

/// Draw an interpolated morph between two sets of 180 normalized points with bounce scaling.
pub fn draw_morph_with_bounce(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    p0: &[[f32; 2]; SHAPE_SAMPLE_COUNT],
    p1: &[[f32; 2]; SHAPE_SAMPLE_COUNT],
    morph_t: f32,
    spring_pos: f32,
    color: Rgba,
) {
    let t = morph_t.clamp(0.0, 1.0);
    let scale = if (spring_pos - 1.0).abs() > 1e-4 {
        1.0 + (spring_pos - 1.0) * 0.12
    } else {
        1.0
    };

    let target_bounds = if (scale - 1.0).abs() > 1e-4 {
        let cx = bounds.left() + bounds.size.width / 2.0;
        let cy = bounds.top() + bounds.size.height / 2.0;
        let sw = bounds.size.width * scale;
        let sh = bounds.size.height * scale;
        Bounds {
            origin: point(cx - sw / 2.0, cy - sh / 2.0),
            size: size(sw, sh),
        }
    } else {
        bounds
    };

    if t >= 0.9999 {
        draw_points(window, target_bounds, p1, color);
        return;
    }
    if t <= 0.0001 {
        draw_points(window, target_bounds, p0, color);
        return;
    }
    let mut morphed = [[0.0f32; 2]; SHAPE_SAMPLE_COUNT];
    let one_minus_t = 1.0 - t;
    for i in 0..SHAPE_SAMPLE_COUNT {
        morphed[i][0] = one_minus_t * p0[i][0] + t * p1[i][0];
        morphed[i][1] = one_minus_t * p0[i][1] + t * p1[i][1];
    }
    draw_points(window, target_bounds, &morphed, color);
}

/// Draw an interpolated morph between two sets of 180 normalized points at progress t.
pub fn draw_morph_points(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    p0: &[[f32; 2]; SHAPE_SAMPLE_COUNT],
    p1: &[[f32; 2]; SHAPE_SAMPLE_COUNT],
    progress: f32,
    color: Rgba,
) {
    draw_morph_with_bounce(
        window,
        bounds,
        p0,
        p1,
        progress.clamp(0.0, 1.0),
        progress,
        color,
    );
}

/// Draw a closed polygon from 180 normalized unit coordinates [0, 1].
pub fn draw_points(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    points: &[[f32; 2]; SHAPE_SAMPLE_COUNT],
    color: Rgba,
) {
    let viewport = window.viewport_size();
    if bounds.right() < px(0.0)
        || bounds.left() > viewport.width
        || bounds.bottom() < px(0.0)
        || bounds.top() > viewport.height
    {
        return;
    }
    let w = f32::from(bounds.size.width);
    let h = f32::from(bounds.size.height);
    let ox = bounds.left();
    let oy = bounds.top();
    let mut builder = PathBuilder::fill();
    builder.move_to(point(ox + px(points[0][0] * w), oy + px(points[0][1] * h)));
    for pt in &points[1..] {
        builder.line_to(point(ox + px(pt[0] * w), oy + px(pt[1] * h)));
    }
    builder.close();
    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

/// Compute damping from stiffness and damping ratio.
#[inline]
#[must_use]
pub fn damping_from_ratio(stiffness: f32, damping_ratio: f32) -> f32 {
    2.0 * damping_ratio * stiffness.max(1e-4).sqrt()
}

/// Interpolate two normalized point sets and rotate the result about the unit center.
#[inline]
#[must_use]
pub fn lerp_and_rotate_points(
    p0: &[[f32; 2]; SHAPE_SAMPLE_COUNT],
    p1: &[[f32; 2]; SHAPE_SAMPLE_COUNT],
    progress: f32,
    angle_deg: f32,
) -> [[f32; 2]; SHAPE_SAMPLE_COUNT] {
    let t = progress.clamp(0.0, 1.0);
    let one_minus_t = 1.0 - t;
    let theta = angle_deg.to_radians();
    let (sin, cos) = theta.sin_cos();
    let mut out = [[0.0f32; 2]; SHAPE_SAMPLE_COUNT];
    for i in 0..SHAPE_SAMPLE_COUNT {
        let x = one_minus_t * p0[i][0] + t * p1[i][0] - 0.5;
        let y = one_minus_t * p0[i][1] + t * p1[i][1] - 0.5;
        out[i][0] = 0.5 + x * cos - y * sin;
        out[i][1] = 0.5 + x * sin + y * cos;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_35_shapes_present_and_unique() {
        assert_eq!(MaterialShape::ALL.len(), 35);
        for (i, s) in MaterialShape::ALL.iter().enumerate() {
            assert_eq!(*s as usize, i);
            assert!(!s.name().is_empty());
            assert!(!s.short_name().is_empty());
        }
    }

    #[test]
    fn all_35_shapes_generate_valid_polygons() {
        for s in MaterialShape::ALL {
            let poly = s.polygon();
            assert!(!poly.cubics.is_empty(), "Shape {:?} has no cubics", s);
            for (idx, c) in poly.cubics.iter().enumerate() {
                assert!(
                    !c.anchor0.x.is_nan() && !c.anchor0.y.is_nan(),
                    "{:?} cubic {} anchor0 is NaN",
                    s,
                    idx
                );
                assert!(
                    !c.control0.x.is_nan() && !c.control0.y.is_nan(),
                    "{:?} cubic {} control0 is NaN",
                    s,
                    idx
                );
                assert!(
                    !c.control1.x.is_nan() && !c.control1.y.is_nan(),
                    "{:?} cubic {} control1 is NaN",
                    s,
                    idx
                );
                assert!(
                    !c.anchor1.x.is_nan() && !c.anchor1.y.is_nan(),
                    "{:?} cubic {} anchor1 is NaN",
                    s,
                    idx
                );
            }
        }
    }

    #[test]
    fn shape_morph_matches_and_evaluates() {
        let s1 = MaterialShape::SoftBurst.polygon();
        let s2 = MaterialShape::Cookie9Sided.polygon();
        let morph = Morph::new(&s1, &s2);
        let cubics_mid = morph.as_cubics(0.5);
        assert!(!cubics_mid.is_empty());
        for c in &cubics_mid {
            assert!(!c.anchor0.x.is_nan());
            assert!(!c.anchor1.x.is_nan());
        }
    }

    #[test]
    fn all_shapes_morph_cleanly() {
        for s1 in MaterialShape::ALL {
            for s2 in MaterialShape::ALL {
                if s1 == s2 {
                    continue;
                }
                let p1 = s1.polygon();
                let p2 = s2.polygon();
                let morph = Morph::new(&p1, &p2);
                for step in [0.0, 0.25, 0.5, 0.75, 1.0] {
                    let cubics = morph.as_cubics(step);
                    for c in &cubics {
                        assert!(
                            !c.anchor0.x.is_nan() && !c.anchor0.y.is_nan(),
                            "NaN in morph {:?} -> {:?}",
                            s1,
                            s2
                        );
                        assert!(
                            !c.anchor1.x.is_nan() && !c.anchor1.y.is_nan(),
                            "NaN in morph {:?} -> {:?}",
                            s1,
                            s2
                        );
                    }
                }
            }
        }
    }
}
