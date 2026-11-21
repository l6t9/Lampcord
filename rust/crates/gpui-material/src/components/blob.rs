//! Liquid "blob" surfaces: rounded rectangles that melt into each other.
//!
//! Every [`BlobRect`] is a signed distance field (a rounded box with per-corner radii);
//! shapes closer than the smoothing radius merge through a circular smooth-min, so a
//! panel sliding out of a bar grows out of it like a drop of liquid, and a panel that
//! moves squashes and stretches along its velocity ([`DeformSpring`]). This follows the
//! blob shader of the Caelestia shell, evaluated on the CPU: the merged field is sampled
//! on a grid around each cluster of nearby shapes, its zero contour is traced with
//! marching squares, and the outline is filled as an antialiased path.
//!
//! A shape can also melt into a screen edge ([`BlobRect::melt_into`]): where it meets
//! the edge the field grows a concave fillet against it, so a dock standing on the
//! bottom of the screen, or a bar hugging its sides, curves into the edge. The fillet is
//! part of the field, so it follows the shape as it moves, squashes or slides behind the
//! edge (it shrinks as the shape sinks and wells up as it rises).
//!
//! Typical use from a canvas paint callback:
//!
//! ```rust,no_run
//! use gpui_material::blob::{paint_blobs, BlobRect};
//! # fn paint(window: &mut gpui::Window, bar: gpui::Bounds<gpui::Pixels>, popup: gpui::Bounds<gpui::Pixels>) {
//! let rects = [BlobRect::new(bar).radius(0.0), BlobRect::new(popup).radius(28.0)];
//! paint_blobs(window, &rects, 20.0, gpui::rgb(0x1d1b20).into());
//! # }
//! ```

use std::cell::RefCell;
use std::collections::HashMap;

use super::shape::MaterialShape;
use gpui::{Bounds, PathBuilder, Pixels, Point, Rgba, Window, point, px};

thread_local! {
    static FIELD_SCRATCH: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
    static COARSE_SCRATCH: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
}

/// A symmetric 2x2 deformation `[[m00, m01], [m01, m11]]` applied around a shape's
/// center: stretch along one axis, compress along the other.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Deform {
    /// Row 0, column 0.
    pub m00: f32,
    /// The off-diagonal term (the matrix is symmetric).
    pub m01: f32,
    /// Row 1, column 1.
    pub m11: f32,
}

impl Default for Deform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Deform {
    /// No deformation.
    pub const IDENTITY: Self = Self {
        m00: 1.0,
        m01: 0.0,
        m11: 1.0,
    };

    fn inverse(self) -> (f32, f32, f32) {
        let det = self.m00 * self.m11 - self.m01 * self.m01;
        if det.abs() < 1e-6 {
            return (1.0, 0.0, 1.0);
        }
        (self.m11 / det, -self.m01 / det, self.m00 / det)
    }

    /// Smallest eigenvalue, the SDF scale correction for the deformed shape.
    fn min_eigen(self) -> f32 {
        let tr = self.m00 + self.m11;
        let d = ((self.m00 - self.m11).powi(2) / 4.0 + self.m01 * self.m01).sqrt();
        tr / 2.0 - d
    }

    fn max_eigen(self) -> f32 {
        let tr = self.m00 + self.m11;
        let d = ((self.m00 - self.m11).powi(2) / 4.0 + self.m01 * self.m01).sqrt();
        tr / 2.0 + d
    }

    /// True when the deformation is visually imperceptible.
    pub fn is_identity(self) -> bool {
        (self.m00 - 1.0).abs() + self.m01.abs() + (self.m11 - 1.0).abs() < 0.004
    }
}

/// Velocity driven squash and stretch: an underdamped spring on each matrix component,
/// pulled towards a stretch along the direction of motion.
#[derive(Clone, Copy, Debug)]
pub struct DeformSpring {
    /// Current deformation.
    pub value: Deform,
    velocity: (f32, f32, f32),
    prev_center: Option<Point<Pixels>>,
    active: bool,
    /// Spring stiffness (default 200).
    pub stiffness: f32,
    /// Damping (default 16).
    pub damping: f32,
    /// Stretch per px/s of speed (default 0.0005 * 0.15, a panel-ish amount).
    pub deform_scale: f32,
}

impl Default for DeformSpring {
    fn default() -> Self {
        Self {
            value: Deform::IDENTITY,
            velocity: (0.0, 0.0, 0.0),
            prev_center: None,
            active: false,
            stiffness: 200.0,
            damping: 16.0,
            deform_scale: 0.000_075,
        }
    }
}

impl DeformSpring {
    /// A spring with a given deform scale (stretch per px/s).
    #[must_use]
    pub fn with_scale(deform_scale: f32) -> Self {
        Self {
            deform_scale,
            ..Self::default()
        }
    }

    /// True while the shape is still wobbling and needs frames.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Puts the spring at rest, for a shape that's no longer drawn: it only moves when
    /// stepped, so one left wobbling would otherwise ask for frames forever.
    pub fn rest(&mut self) {
        *self = Self {
            stiffness: self.stiffness,
            damping: self.damping,
            deform_scale: self.deform_scale,
            ..Self::default()
        };
    }

    /// Advances the spring given the shape's current center and the frame time.
    pub fn step(&mut self, center: Point<Pixels>, dt: f32) -> Deform {
        let Some(prev) = self.prev_center.replace(center) else {
            return self.value;
        };
        if dt < 0.001 {
            return self.value;
        }
        // long frames still make progress, clamped so the spring stays stable
        let dt = dt.min(0.05);
        let vx = f32::from(center.x - prev.x) / dt;
        let vy = f32::from(center.y - prev.y) / dt;
        let speed = (vx * vx + vy * vy).sqrt();
        if !self.active {
            if speed < 5.0 {
                return self.value;
            }
            self.active = true;
        }
        const MAX_STRETCH: f32 = 0.35;
        let (mut t00, mut t01, mut t11) = (1.0, 0.0, 1.0);
        if speed > 5.0 {
            let stretch = 1.0 + (speed * self.deform_scale).min(MAX_STRETCH);
            let compress = 1.0 / stretch;
            let (c, s) = (vx / speed, vy / speed);
            t00 = stretch * c * c + compress * s * s;
            t01 = (stretch - compress) * c * s;
            t11 = stretch * s * s + compress * c * c;
        }
        // implicit damping keeps the spring stable on slow frames
        let inv_damp = 1.0 / (1.0 + self.damping * dt);
        let k = self.stiffness;
        let v = &mut self.velocity;
        v.0 = (v.0 - k * (self.value.m00 - t00) * dt) * inv_damp;
        v.1 = (v.1 - k * (self.value.m01 - t01) * dt) * inv_damp;
        v.2 = (v.2 - k * (self.value.m11 - t11) * dt) * inv_damp;
        self.value.m00 += v.0 * dt;
        self.value.m01 += v.1 * dt;
        self.value.m11 += v.2 * dt;
        let vel = v.0.abs() + v.1.abs() + v.2.abs();
        if self.value.is_identity() && vel < 0.05 && speed < 5.0 {
            *self = Self {
                stiffness: self.stiffness,
                damping: self.damping,
                deform_scale: self.deform_scale,
                prev_center: Some(center),
                ..Self::default()
            };
        }
        self.value
    }
}

/// A screen edge a shape can melt into, at a coordinate along its axis: `Bottom(1080.0)`
/// is the line `y = 1080` with the screen above it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Edge {
    /// The line `x = at`, screen to its right.
    Left(f32),
    /// The line `y = at`, screen below it.
    Top(f32),
    /// The line `x = at`, screen to its left.
    Right(f32),
    /// The line `y = at`, screen above it.
    Bottom(f32),
}

impl Edge {
    /// Distance from the edge into the screen; negative past it.
    fn inside(self, x: f32, y: f32) -> f32 {
        match self {
            Edge::Left(a) => x - a,
            Edge::Top(a) => y - a,
            Edge::Right(a) => a - x,
            Edge::Bottom(a) => a - y,
        }
    }
}

/// One shape of a blob group.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlobRect {
    /// Layout bounds (before deformation).
    pub bounds: Bounds<Pixels>,
    /// Corner radii: top left, top right, bottom right, bottom left.
    pub radii: [f32; 4],
    /// Squash and stretch around the center.
    pub deform: Deform,
    /// Bit `i` set: don't smooth-merge with shape `i` of the same slice (shapes only
    /// touch with a hard union).
    pub exclude: u64,
    /// Edges the shape melts into, each with its fillet radius.
    pub melt: [Option<(Edge, f32)>; 4],
}

impl BlobRect {
    /// A shape with square corners.
    #[must_use]
    pub fn new(bounds: Bounds<Pixels>) -> Self {
        Self {
            bounds,
            radii: [0.0; 4],
            deform: Deform::IDENTITY,
            exclude: 0,
            melt: [None; 4],
        }
    }

    /// Same radius on every corner.
    #[must_use]
    pub fn radius(mut self, r: f32) -> Self {
        self.radii = [r; 4];
        self
    }

    /// Per-corner radii: top left, top right, bottom right, bottom left.
    #[must_use]
    pub fn radii(mut self, tl: f32, tr: f32, br: f32, bl: f32) -> Self {
        self.radii = [tl, tr, br, bl];
        self
    }

    /// Applies a deformation.
    #[must_use]
    pub fn deform(mut self, deform: Deform) -> Self {
        self.deform = deform;
        self
    }

    /// Hard union (no smooth merge) with the shape at `index`.
    #[must_use]
    pub fn exclude(mut self, index: usize) -> Self {
        self.exclude |= 1 << index.min(63);
        self
    }

    /// Melts the shape into `edge`: where it is within `radius` of the edge, a concave
    /// fillet of that radius joins the two, cut off at the edge itself. Up to four edges.
    #[must_use]
    pub fn melt_into(mut self, edge: Edge, radius: f32) -> Self {
        if let Some(slot) = self.melt.iter_mut().find(|m| m.is_none()) {
            *slot = Some((edge, radius.max(0.0)));
        }
        self
    }

    /// True when two shapes are visually identical (within subpixel/epsilon tolerance).
    pub fn approx_eq(&self, other: &Self) -> bool {
        let eps = 0.25f32;
        let bounds_close = (f32::from(self.bounds.origin.x) - f32::from(other.bounds.origin.x))
            .abs()
            <= eps
            && (f32::from(self.bounds.origin.y) - f32::from(other.bounds.origin.y)).abs() <= eps
            && (f32::from(self.bounds.size.width) - f32::from(other.bounds.size.width)).abs()
                <= eps
            && (f32::from(self.bounds.size.height) - f32::from(other.bounds.size.height)).abs()
                <= eps;
        if !bounds_close {
            return false;
        }
        for (a, b) in self.radii.iter().zip(&other.radii) {
            if (a - b).abs() > eps {
                return false;
            }
        }
        let deform_close = (self.deform.m00 - other.deform.m00).abs() <= 0.002
            && (self.deform.m01 - other.deform.m01).abs() <= 0.002
            && (self.deform.m11 - other.deform.m11).abs() <= 0.002;
        if !deform_close || self.exclude != other.exclude {
            return false;
        }
        self.melt == other.melt
    }

    fn prepared(&self) -> Prepared {
        let (x, y) = (
            f32::from(self.bounds.origin.x),
            f32::from(self.bounds.origin.y),
        );
        let (w, h) = (
            f32::from(self.bounds.size.width),
            f32::from(self.bounds.size.height),
        );
        let max_r = w.min(h) / 2.0;
        let r = self.radii.map(|r| r.clamp(0.0, max_r.max(0.0)));
        let stretch = self.deform.max_eigen().max(1.0);
        Prepared {
            kind: PreparedKind::Rect {
                hw: w / 2.0,
                hh: h / 2.0,
                r,
            },
            cx: x + w / 2.0,
            cy: y + h / 2.0,
            inv: self.deform.inverse(),
            min_eig: self.deform.min_eigen().max(0.01),
            ext_x: w / 2.0 * stretch,
            ext_y: h / 2.0 * stretch,
            exclude: self.exclude,
            melt: self.melt,
            reach: self.melt.iter().flatten().map(|m| m.1).fold(0.0, f32::max),
            empty: w < 0.5 || h < 0.5,
            color: None,
        }
    }
}

/// One Material 3 shape in a liquid blob group.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlobShape {
    /// Which Material 3 shape this blob is.
    pub shape: MaterialShape,
    /// Target shape being morphed into, if actively morphing.
    pub morph_to: Option<MaterialShape>,
    /// Progress of the morph animation from 0.0 to 1.0.
    pub morph_progress: f32,
    /// Center of the shape.
    pub center: Point<Pixels>,
    /// Radius (half-size) of the shape.
    pub radius: Pixels,
    /// Rotation in degrees.
    pub rotation: f32,
    /// Squash and stretch around the center.
    pub deform: Deform,
    /// Color of this individual shape blob.
    pub color: Option<Rgba>,
    /// Bit `i` set: don't smooth-merge with shape `i` of the same slice.
    pub exclude: u64,
    /// Edges the shape melts into, each with its fillet radius.
    pub melt: [Option<(Edge, f32)>; 4],
}

impl BlobShape {
    /// A new Material 3 shape blob with given shape, center, and radius.
    #[must_use]
    pub fn new(shape: MaterialShape, center: Point<Pixels>, radius: Pixels) -> Self {
        Self {
            shape,
            morph_to: None,
            morph_progress: 0.0,
            center,
            radius,
            rotation: 0.0,
            deform: Deform::IDENTITY,
            color: None,
            exclude: 0,
            melt: [None; 4],
        }
    }

    /// Sets a target shape to morph into with animated progress `t` in `[0.0, 1.0]`.
    #[must_use]
    pub fn morph_to(mut self, target: MaterialShape, progress: f32) -> Self {
        self.morph_to = Some(target);
        self.morph_progress = progress.clamp(0.0, 1.0);
        self
    }

    /// Construct a shape blob fitting inside layout `bounds`.
    #[must_use]
    pub fn from_bounds(shape: MaterialShape, bounds: Bounds<Pixels>) -> Self {
        let center = bounds.center();
        let radius = bounds.size.width.min(bounds.size.height) / 2.0;
        Self::new(shape, center, radius)
    }

    /// Rotates the shape by `degrees` clockwise.
    #[must_use]
    pub fn rotation(mut self, degrees: f32) -> Self {
        self.rotation = degrees;
        self
    }

    /// Applies a deformation.
    #[must_use]
    pub fn deform(mut self, deform: Deform) -> Self {
        self.deform = deform;
        self
    }

    /// Sets the color of this blob.
    #[must_use]
    pub fn color(mut self, color: Rgba) -> Self {
        self.color = Some(color);
        self
    }

    /// Hard union (no smooth merge) with the shape at `index`.
    #[must_use]
    pub fn exclude(mut self, index: usize) -> Self {
        self.exclude |= 1 << index.min(63);
        self
    }

    /// Melts the shape into `edge`.
    #[must_use]
    pub fn melt_into(mut self, edge: Edge, radius: f32) -> Self {
        if let Some(slot) = self.melt.iter_mut().find(|m| m.is_none()) {
            *slot = Some((edge, radius.max(0.0)));
        }
        self
    }

    /// True when two shape blobs are visually identical (within epsilon tolerance).
    pub fn approx_eq(&self, other: &Self) -> bool {
        let eps = 0.25f32;
        let center_close = (f32::from(self.center.x) - f32::from(other.center.x)).abs() <= eps
            && (f32::from(self.center.y) - f32::from(other.center.y)).abs() <= eps;
        let radius_close = (f32::from(self.radius) - f32::from(other.radius)).abs() <= eps;
        let rot_close = (self.rotation - other.rotation).abs() <= 0.5;
        let deform_close = (self.deform.m00 - other.deform.m00).abs() <= 0.002
            && (self.deform.m01 - other.deform.m01).abs() <= 0.002
            && (self.deform.m11 - other.deform.m11).abs() <= 0.002;
        center_close
            && radius_close
            && rot_close
            && deform_close
            && self.shape == other.shape
            && self.morph_to == other.morph_to
            && (self.morph_progress - other.morph_progress).abs() <= 0.01
            && self.exclude == other.exclude
            && self.melt == other.melt
            && self.color == other.color
    }

    fn prepared(&self) -> Prepared {
        let cx = f32::from(self.center.x);
        let cy = f32::from(self.center.y);
        let rad = f32::from(self.radius);
        let stretch = self.deform.max_eigen().max(1.0);
        let (sin_rot, cos_rot) = self.rotation.to_radians().sin_cos();
        Prepared {
            kind: PreparedKind::Shape {
                shape: self.shape,
                morph_to: self.morph_to,
                morph_t: self.morph_progress,
                radius: rad,
                sin_rot,
                cos_rot,
            },
            cx,
            cy,
            inv: self.deform.inverse(),
            min_eig: self.deform.min_eigen().max(0.01),
            // a shape fills its square; turned, the square's corners reach further out
            ext_x: rad * stretch * (sin_rot.abs() + cos_rot.abs()),
            ext_y: rad * stretch * (sin_rot.abs() + cos_rot.abs()),
            exclude: self.exclude,
            melt: self.melt,
            reach: self.melt.iter().flatten().map(|m| m.1).fold(0.0, f32::max),
            empty: rad < 0.5,
            color: self.color,
        }
    }
}

#[derive(Clone, Copy)]
enum PreparedKind {
    Rect {
        hw: f32,
        hh: f32,
        r: [f32; 4],
    },
    Shape {
        shape: MaterialShape,
        morph_to: Option<MaterialShape>,
        morph_t: f32,
        radius: f32,
        sin_rot: f32,
        cos_rot: f32,
    },
}

#[derive(Clone, Copy)]
struct Prepared {
    kind: PreparedKind,
    cx: f32,
    cy: f32,
    inv: (f32, f32, f32),
    min_eig: f32,
    ext_x: f32,
    ext_y: f32,
    exclude: u64,
    melt: [Option<(Edge, f32)>; 4],
    reach: f32,
    empty: bool,
    color: Option<Rgba>,
}

impl Prepared {
    /// Outline of the shape or deformed rounded box, grown by `iso`.
    fn outline(&self, iso: f32) -> Vec<Point<Pixels>> {
        let (i00, i01, i11) = self.inv;
        // forward deform = inverse of the stored inverse
        let det = i00 * i11 - i01 * i01;
        let (m00, m01, m11) = if det.abs() < 1e-6 {
            (1.0, 0.0, 1.0)
        } else {
            (i11 / det, -i01 / det, i00 / det)
        };

        match self.kind {
            PreparedKind::Rect { hw, hh, r } => {
                let grow = iso / self.min_eig;
                let (hw, hh) = (hw + grow, hh + grow);
                let mut pts = Vec::new();
                let corners = [
                    (-1.0f32, -1.0f32, std::f32::consts::PI, r[0]),
                    (1.0, -1.0, 1.5 * std::f32::consts::PI, r[1]),
                    (1.0, 1.0, 0.0, r[2]),
                    (-1.0, 1.0, 0.5 * std::f32::consts::PI, r[3]),
                ];
                for (sx, sy, a0, cr) in corners {
                    let cr = (cr + grow).max(0.0).min(hw.min(hh));
                    let (ccx, ccy) = (sx * (hw - cr), sy * (hh - cr));
                    let n = if cr < 0.5 {
                        1
                    } else {
                        ((cr / 2.0).ceil() as usize).clamp(4, 24)
                    };
                    for k in 0..=n {
                        let a = a0 + std::f32::consts::FRAC_PI_2 * k as f32 / n as f32;
                        let (qx, qy) = (ccx + cr * a.cos(), ccy + cr * a.sin());
                        pts.push(point(
                            px(self.cx + m00 * qx + m01 * qy),
                            px(self.cy + m01 * qx + m11 * qy),
                        ));
                    }
                }
                pts
            }
            PreparedKind::Shape {
                shape,
                morph_to,
                morph_t,
                radius,
                sin_rot,
                cos_rot,
            } => {
                let pts = if let Some(target) = morph_to {
                    let p1 = shape.polygon();
                    let p2 = target.polygon();
                    let morph = super::shape::Morph::new(&p1, &p2);
                    let cubics = morph.as_cubics(morph_t.clamp(0.0, 1.0));
                    let mut out = [[0.0f32; 2]; super::shape::SHAPE_SAMPLE_COUNT];
                    if !cubics.is_empty() {
                        let n = cubics.len();
                        for i in 0..super::shape::SHAPE_SAMPLE_COUNT {
                            let t_global = i as f32 / super::shape::SHAPE_SAMPLE_COUNT as f32;
                            let c_frac = t_global * (n as f32);
                            let idx = (c_frac.floor() as usize).min(n - 1);
                            let t = c_frac - idx as f32;
                            let pt = cubics[idx].point_on_curve(t);
                            out[i] = [pt.x, pt.y];
                        }
                    }
                    out
                } else {
                    *shape.points()
                };
                let grow = iso / self.min_eig;
                let diam = 2.0 * (radius + grow);
                let mut out = Vec::with_capacity(super::shape::SHAPE_SAMPLE_COUNT);
                for p in pts {
                    let lx = (p[0] - 0.5) * diam;
                    let ly = (p[1] - 0.5) * diam;
                    let rx = lx * cos_rot - ly * sin_rot;
                    let ry = lx * sin_rot + ly * cos_rot;
                    let fx = m00 * rx + m01 * ry;
                    let fy = m01 * rx + m11 * ry;
                    out.push(point(px(self.cx + fx), px(self.cy + fy)));
                }
                out
            }
        }
    }

    fn sdf(&self, px_: f32, py: f32) -> f32 {
        let (dx, dy) = (px_ - self.cx, py - self.cy);
        let (i00, i01, i11) = self.inv;
        let (qx, qy) = (i00 * dx + i01 * dy, i01 * dx + i11 * dy);
        match self.kind {
            PreparedKind::Rect { hw, hh, r } => {
                let r_corner = match (qx > 0.0, qy > 0.0) {
                    (false, false) => r[0],
                    (true, false) => r[1],
                    (true, true) => r[2],
                    (false, true) => r[3],
                };
                let ax = qx.abs() - hw + r_corner;
                let ay = qy.abs() - hh + r_corner;
                let outside = (ax.max(0.0).powi(2) + ay.max(0.0).powi(2)).sqrt();
                (ax.max(ay).min(0.0) + outside - r_corner) * self.min_eig
            }
            PreparedKind::Shape {
                shape,
                morph_to,
                morph_t,
                radius,
                sin_rot,
                cos_rot,
            } => {
                let rx = qx * cos_rot + qy * sin_rot;
                let ry = -qx * sin_rot + qy * cos_rot;
                let local_x = rx / (2.0 * radius.max(0.1));
                let local_y = ry / (2.0 * radius.max(0.1));
                let d0 = shape.sdf(local_x, local_y);
                let unit_d = if let Some(target) = morph_to {
                    let d1 = target.sdf(local_x, local_y);
                    d0 * (1.0 - morph_t) + d1 * morph_t
                } else {
                    d0
                };
                unit_d * (2.0 * radius) * self.min_eig
            }
        }
    }
}

/// Circular smooth minimum: a fillet of radius `k` between the two surfaces.
fn smin(a: f32, b: f32, k: f32) -> f32 {
    let ma = (k - a).max(0.0);
    let mb = (k - b).max(0.0);
    k.max(a.min(b)) - (ma * ma + mb * mb).sqrt()
}

fn merged_sdf(shapes: &[(usize, Prepared)], x: f32, y: f32, k: f32) -> f32 {
    let mut d = [f32::MAX; 64];
    let mut merged = f32::MAX;
    for (n, (_, s)) in shapes.iter().enumerate().take(64) {
        let reach = k.max(s.reach) * 1.5 + 4.0;
        if (x - s.cx).abs() > s.ext_x + reach || (y - s.cy).abs() > s.ext_y + reach {
            // far away: only matters as "outside"
            d[n] = f32::MAX;
            continue;
        }
        d[n] = s.sdf(x, y);
        merged = merged.min(d[n]);
        // fillets into the edges it melts into, only on the screen side of each edge
        for (edge, r) in s.melt.iter().flatten() {
            if d[n] >= *r {
                continue;
            }
            let de = edge.inside(x, y);
            if (0.0..*r).contains(&de) {
                merged = merged.min(smin(d[n], de, *r));
            }
        }
    }
    for i in 0..shapes.len().min(64) {
        if d[i] >= k {
            continue;
        }
        for j in i + 1..shapes.len().min(64) {
            if d[j] >= k {
                continue;
            }
            let (oi, oj) = (shapes[i].0, shapes[j].0);
            if shapes[i].1.exclude & (1 << oj.min(63)) != 0
                || shapes[j].1.exclude & (1 << oi.min(63)) != 0
            {
                continue;
            }
            merged = merged.min(smin(d[i], d[j], k));
        }
    }
    if merged == f32::MAX { 1e6 } else { merged }
}

/// True when two slices of [`BlobRect`] are visually identical (within epsilon tolerance).
pub fn rects_approx_eq(a: &[BlobRect], b: &[BlobRect]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.approx_eq(y))
}

/// Outlines of the merged shapes where the field equals `iso` (0 for the surface,
/// positive values for grown copies, e.g. for soft shadows). `step` is the grid size in
/// pixels; 2 gives smooth curves.
/// Computes contours for prepared shapes grouped into clusters.
fn prepared_contours(
    prepared: &[(usize, Prepared)],
    smoothing: f32,
    step: f32,
    iso: f32,
) -> Vec<(Vec<Vec<Point<Pixels>>>, Option<Rgba>)> {
    let k = smoothing.max(0.5);
    let step = step.max(0.5);
    let pad = |p: &Prepared| k.max(p.reach) * 1.5 + iso.max(0.0) + step * 2.0;
    // group shapes whose padded boxes touch, each cluster gets its own grid
    let boxes: Vec<(f32, f32, f32, f32)> = prepared
        .iter()
        .map(|(_, p)| {
            let pad = pad(p);
            (
                p.cx - p.ext_x - pad,
                p.cy - p.ext_y - pad,
                p.cx + p.ext_x + pad,
                p.cy + p.ext_y + pad,
            )
        })
        .collect();
    let mut cluster = (0..prepared.len()).collect::<Vec<_>>();
    fn find(c: &mut [usize], i: usize) -> usize {
        let mut i = i;
        while c[i] != i {
            c[i] = c[c[i]];
            i = c[i];
        }
        i
    }
    for i in 0..boxes.len() {
        for j in i + 1..boxes.len() {
            let (a, b) = (boxes[i], boxes[j]);
            if a.0 <= b.2 && b.0 <= a.2 && a.1 <= b.3 && b.1 <= a.3 {
                let (ri, rj) = (find(&mut cluster, i), find(&mut cluster, j));
                cluster[ri] = rj;
            }
        }
    }
    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..prepared.len() {
        let root = find(&mut cluster, i);
        groups.entry(root).or_default().push(i);
    }
    let mut out = Vec::new();
    for members in groups.values() {
        let colors: Vec<Rgba> = members
            .iter()
            .filter_map(|&idx| prepared[idx].1.color)
            .collect();
        let cluster_color = if colors.is_empty() {
            None
        } else {
            let mut r = 0.0;
            let mut g = 0.0;
            let mut b = 0.0;
            let mut a = 0.0;
            for c in &colors {
                r += c.r;
                g += c.g;
                b += c.b;
                a += c.a;
            }
            let n = colors.len() as f32;
            Some(Rgba {
                r: r / n,
                g: g / n,
                b: b / n,
                a: a / n,
            })
        };

        if let [only] = members.as_slice() {
            // Rectangles (e.g. OSD) have exact analytic outlines; shapes consistently march
            if matches!(prepared[*only].1.kind, PreparedKind::Rect { .. })
                && prepared[*only].1.reach <= 0.0
            {
                out.push((vec![prepared[*only].1.outline(iso)], cluster_color));
                continue;
            }
        }
        let shapes: Vec<(usize, Prepared)> = members.iter().map(|&i| prepared[i]).collect();
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for &i in members {
            let b = boxes[i];
            x0 = x0.min(b.0);
            y0 = y0.min(b.1);
            x1 = x1.max(b.2);
            y1 = y1.max(b.3);
        }
        x0 = (x0 / step).floor() * step;
        y0 = (y0 / step).floor() * step;
        let loops = march(
            &shapes,
            k,
            iso,
            x0,
            y0,
            ((x1 - x0) / step).ceil() as usize + 1,
            ((y1 - y0) / step).ceil() as usize + 1,
            step,
        );
        out.push((loops, cluster_color));
    }
    out
}

/// Outlines of the merged shapes where the field equals `iso` (0 for the surface,
/// positive values for grown copies, e.g. for soft shadows). `step` is the grid size in
/// pixels; 2 gives smooth curves.
pub fn blob_contours(
    rects: &[BlobRect],
    smoothing: f32,
    step: f32,
    iso: f32,
) -> Vec<Vec<Point<Pixels>>> {
    let prepared: Vec<(usize, Prepared)> = rects
        .iter()
        .enumerate()
        .map(|(i, r)| (i, r.prepared()))
        .filter(|(_, p)| !p.empty)
        .collect();
    prepared_contours(&prepared, smoothing, step, iso)
        .into_iter()
        .flat_map(|(loops, _)| loops)
        .collect()
}

/// Outlines of merged Material 3 shape blobs where the field equals `iso`.
pub fn blob_shape_contours(
    shapes: &[BlobShape],
    smoothing: f32,
    step: f32,
    iso: f32,
) -> Vec<Vec<Point<Pixels>>> {
    let prepared: Vec<(usize, Prepared)> = shapes
        .iter()
        .enumerate()
        .map(|(i, s)| (i, s.prepared()))
        .filter(|(_, p)| !p.empty)
        .collect();
    prepared_contours(&prepared, smoothing, step, iso)
        .into_iter()
        .flat_map(|(loops, _)| loops)
        .collect()
}

/// Outlines of merged Material 3 shape blobs with their cluster colors.
pub fn blob_shape_contours_colored(
    shapes: &[BlobShape],
    smoothing: f32,
    step: f32,
    iso: f32,
    default_color: Rgba,
) -> Vec<(Vec<Vec<Point<Pixels>>>, Rgba)> {
    let prepared: Vec<(usize, Prepared)> = shapes
        .iter()
        .enumerate()
        .map(|(i, s)| (i, s.prepared()))
        .filter(|(_, p)| !p.empty)
        .collect();
    prepared_contours(&prepared, smoothing, step, iso)
        .into_iter()
        .map(|(loops, c)| (loops, c.unwrap_or(default_color)))
        .collect()
}

/// The merged SDF on a `vw x vh` vertex grid. It is sampled on a coarse grid first and
/// only evaluated at full resolution in coarse cells the surface can pass through (the
/// field is 1-Lipschitz, so a cell whose corners are all farther than its diagonal from
/// the surface can't contain it); elsewhere the coarse sign is enough.
#[allow(clippy::too_many_arguments)]
fn with_sampled_field<R>(
    shapes: &[(usize, Prepared)],
    k: f32,
    iso: f32,
    x0: f32,
    y0: f32,
    vw: usize,
    vh: usize,
    step: f32,
    f_action: impl FnOnce(&[f32]) -> R,
) -> R {
    const C: usize = 4;
    let f = |i: usize, j: usize| {
        merged_sdf(shapes, x0 + i as f32 * step, y0 + j as f32 * step, k) - iso
    };
    let (cw, ch) = ((vw - 1).div_ceil(C) + 1, (vh - 1).div_ceil(C) + 1);
    let coarse_len = ch * cw;
    let safe = C as f32 * step * std::f32::consts::SQRT_2 * 1.25;

    COARSE_SCRATCH.with(|coarse_cell| {
        let mut coarse = coarse_cell.borrow_mut();
        coarse.clear();
        coarse.reserve(coarse_len);
        for n in 0..coarse_len {
            coarse.push(f(((n % cw) * C).min(vw - 1), ((n / cw) * C).min(vh - 1)));
        }

        FIELD_SCRATCH.with(|field_cell| {
            let mut field = field_cell.borrow_mut();
            let total = vw * vh;
            field.clear();
            field.resize(total, f32::NAN);

            for cj in 0..ch - 1 {
                for ci in 0..cw - 1 {
                    let corners = [
                        coarse[cj * cw + ci],
                        coarse[cj * cw + ci + 1],
                        coarse[(cj + 1) * cw + ci],
                        coarse[(cj + 1) * cw + ci + 1],
                    ];
                    if corners.iter().all(|v| v.abs() > safe) {
                        continue;
                    }
                    for j in cj * C..=((cj + 1) * C).min(vh - 1) {
                        for i in ci * C..=((ci + 1) * C).min(vw - 1) {
                            let n = j * vw + i;
                            if field[n].is_nan() {
                                field[n] = f(i, j);
                            }
                        }
                    }
                }
            }
            for j in 0..vh {
                for i in 0..vw {
                    let n = j * vw + i;
                    if field[n].is_nan() {
                        let (ci, cj) =
                            (((i + C / 2) / C).min(cw - 1), ((j + C / 2) / C).min(ch - 1));
                        field[n] = coarse[cj * cw + ci];
                    }
                }
            }
            f_action(&field)
        })
    })
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn sample_field(
    shapes: &[(usize, Prepared)],
    k: f32,
    iso: f32,
    x0: f32,
    y0: f32,
    vw: usize,
    vh: usize,
    step: f32,
) -> Vec<f32> {
    with_sampled_field(shapes, k, iso, x0, y0, vw, vh, step, |f| f.to_vec())
}

/// Marching squares over a `w x h` cell grid starting at `(x0, y0)`.
#[allow(clippy::too_many_arguments)]
fn march(
    shapes: &[(usize, Prepared)],
    k: f32,
    iso: f32,
    x0: f32,
    y0: f32,
    w: usize,
    h: usize,
    step: f32,
) -> Vec<Vec<Point<Pixels>>> {
    let (vw, vh) = (w + 1, h + 1);
    with_sampled_field(shapes, k, iso, x0, y0, vw, vh, step, |field| {
        let v = |i: usize, j: usize| field[j * vw + i];
        let h_id = |i: usize, j: usize| j * w + i;
        let v_id = |i: usize, j: usize| vw * vh + j * vw + i;
        let mut points: HashMap<usize, Point<Pixels>> = HashMap::new();
        let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
        let lerp = |a: f32, b: f32| (a / (a - b)).clamp(0.0, 1.0);
        for j in 0..h {
            for i in 0..w {
                let (a, b, c, d) = (v(i, j), v(i + 1, j), v(i + 1, j + 1), v(i, j + 1));
                let case = (a < 0.0) as u8
                    | ((b < 0.0) as u8) << 1
                    | ((c < 0.0) as u8) << 2
                    | ((d < 0.0) as u8) << 3;
                if case == 0 || case == 15 {
                    continue;
                }
                // edges: 0 top, 1 right, 2 bottom, 3 left
                let edge = |e: u8| -> usize {
                    match e {
                        0 => h_id(i, j),
                        1 => v_id(i + 1, j),
                        2 => h_id(i, j + 1),
                        _ => v_id(i, j),
                    }
                };
                let (fx, fy) = (x0 + i as f32 * step, y0 + j as f32 * step);
                let mut pt = |e: u8| {
                    let id = edge(e);
                    points.entry(id).or_insert_with(|| {
                        let (px_, py) = match e {
                            0 => (fx + lerp(a, b) * step, fy),
                            1 => (fx + step, fy + lerp(b, c) * step),
                            2 => (fx + lerp(d, c) * step, fy + step),
                            _ => (fx, fy + lerp(a, d) * step),
                        };
                        point(px(px_), px(py))
                    });
                    id
                };
                let center_in = (a + b + c + d) / 4.0 < 0.0;
                let segs: &[(u8, u8)] = match case {
                    1 | 14 => &[(3, 0)],
                    2 | 13 => &[(0, 1)],
                    3 | 12 => &[(3, 1)],
                    4 | 11 => &[(1, 2)],
                    6 | 9 => &[(0, 2)],
                    7 | 8 => &[(3, 2)],
                    5 => {
                        if center_in {
                            &[(0, 1), (2, 3)]
                        } else {
                            &[(3, 0), (1, 2)]
                        }
                    }
                    _ => {
                        // 10
                        if center_in {
                            &[(3, 0), (1, 2)]
                        } else {
                            &[(0, 1), (2, 3)]
                        }
                    }
                };
                for &(ea, eb) in segs {
                    let (ia, ib) = (pt(ea), pt(eb));
                    adj.entry(ia).or_default().push(ib);
                    adj.entry(ib).or_default().push(ia);
                }
            }
        }
        // walk the segment graph into closed loops
        let mut loops = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let keys: Vec<usize> = adj.keys().copied().collect();
        for start in keys {
            if visited.contains(&start) {
                continue;
            }
            let mut path = vec![start];
            visited.insert(start);
            let mut prev = usize::MAX;
            let mut cur = start;
            loop {
                let Some(next) = adj.get(&cur).and_then(|n| {
                    n.iter()
                        .copied()
                        .find(|&x| x != prev && !visited.contains(&x))
                        .or_else(|| {
                            n.iter()
                                .copied()
                                .find(|&x| x == start && prev != usize::MAX && x != prev)
                        })
                }) else {
                    break;
                };
                if next == start {
                    break;
                }
                visited.insert(next);
                path.push(next);
                prev = cur;
                cur = next;
            }
            if path.len() >= 3 {
                loops.push(
                    path.into_iter()
                        .filter_map(|id| points.get(&id).copied())
                        .collect(),
                );
            }
        }
        loops
    })
}

/// Fills the merged outline of `rects`. `smoothing` is the fillet radius where shapes
/// meet (20 is a good default).
pub fn paint_blobs(window: &mut Window, rects: &[BlobRect], smoothing: f32, color: Rgba) {
    paint_contours(window, &blob_contours(rects, smoothing, 2.0, 0.0), color);
}

/// Soft shadow under a blob group: a few grown, translucent copies of the outline.
pub fn paint_blob_shadow(
    window: &mut Window,
    rects: &[BlobRect],
    smoothing: f32,
    color: Rgba,
    spread: f32,
    offset_y: f32,
) {
    let layers = 4;
    let shifted: Vec<BlobRect> = rects
        .iter()
        .map(|r| {
            let mut r = *r;
            r.bounds.origin.y += px(offset_y);
            r
        })
        .collect();
    for n in (1..=layers).rev() {
        let iso = spread * n as f32 / layers as f32;
        let alpha = color.a / layers as f32;
        paint_contours(
            window,
            &blob_contours(&shifted, smoothing, 3.0, iso),
            Rgba { a: alpha, ..color },
        );
    }
}

/// Fills precomputed outlines, see [`blob_contours`].
pub fn paint_contours(window: &mut Window, contours: &[Vec<Point<Pixels>>], color: Rgba) {
    if contours.is_empty() {
        return;
    }
    let mut b = PathBuilder::fill();
    for c in contours {
        b.add_polygon(c, true);
    }
    if let Ok(path) = b.build() {
        window.paint_path(path, color);
    }
}

/// Fills the merged outline of Material 3 shape blobs. `smoothing` is the fillet radius where shapes meet.
pub fn paint_shape_blobs(window: &mut Window, shapes: &[BlobShape], smoothing: f32, color: Rgba) {
    paint_contours(
        window,
        &blob_shape_contours(shapes, smoothing, 2.0, 0.0),
        color,
    );
}

/// Fills merged Material 3 shape blobs, using each shape's / cluster's color.
pub fn paint_shape_blobs_colored(
    window: &mut Window,
    shapes: &[BlobShape],
    smoothing: f32,
    default_color: Rgba,
) {
    for (loops, color) in blob_shape_contours_colored(shapes, smoothing, 2.0, 0.0, default_color) {
        paint_contours(window, &loops, color);
    }
}

/// A floating blob in the [`LavaLamp`] fluid simulation.
#[derive(Clone, Debug)]
pub struct LavaBlob {
    /// Material 3 shape of this blob.
    pub shape: MaterialShape,
    /// Target shape being morphed into, if actively morphing.
    pub target_shape: Option<MaterialShape>,
    /// Progress of the morph animation from 0.0 to 1.0.
    pub morph_t: f32,
    /// Duration of the morph in seconds.
    pub morph_duration: f32,
    /// Depth layer index this blob belongs to.
    pub layer: usize,
    /// X coordinate in pixels.
    pub x: f32,
    /// Y coordinate in pixels.
    pub y: f32,
    /// Horizontal velocity in pixels/sec.
    pub vx: f32,
    /// Vertical velocity in pixels/sec.
    pub vy: f32,
    /// Base radius in pixels before scale factor.
    pub base_radius: f32,
    /// Effective radius in pixels.
    pub radius: f32,
    /// Rotation angle in degrees.
    pub rotation: f32,
    /// Rotational velocity in deg/sec.
    pub rot_speed: f32,
    /// Deformation spring driving velocity squash and stretch.
    pub deform: DeformSpring,
    /// Heat / buoyancy factor (-1.0 sinks, +1.0 rises).
    pub heat: f32,
    /// Color of this blob.
    pub color: Rgba,
    /// Phase offset for natural horizontal wandering.
    pub phase: f32,
    /// Time remaining until next morph evaluation in seconds.
    pub next_morph_in: f32,
}

/// A lava lamp simulation that drives fluidly floating and merging Material 3 shapes.
#[derive(Clone, Debug)]
pub struct LavaLamp {
    /// Active floating blobs.
    pub blobs: Vec<LavaBlob>,
    /// Simulation viewport width in pixels.
    pub width: f32,
    /// Simulation viewport height in pixels.
    pub height: f32,
    /// Fillet smoothing radius for merging shapes (default 32.0).
    pub smoothing: f32,
    /// Speed multiplier for the fluid simulation (default 1.0).
    pub speed: f32,
    /// Global scale multiplier for blob radii (default 1.0).
    pub scale: f32,
    /// Convection churn / turbulence intensity (0..100, default 50.0).
    pub churn: f32,
    /// Viscous fluid drag (0..1, default 0.2).
    pub drag: f32,
    /// Rotational spin multiplier (default 1.0).
    pub spin: f32,
    /// Maximum velocity magnitude cap (default 250.0).
    pub max_v: f32,
    /// Mouse cursor attraction / drag force multiplier (default 1.0).
    pub cursor_pull: f32,
    /// Mouse interaction influence radius (default 150.0).
    pub cursor_radius: f32,
    /// Number of separate depth layers (shapes don't merge across layers).
    pub layers: usize,
    /// Percentage probability (0..100) of random shape morphing.
    pub morph_chance: f32,
    /// Active pool of candidate Material 3 shapes.
    pub shape_pool: Vec<MaterialShape>,
    /// Accumulated simulation time in seconds.
    pub time: f32,
    /// Last known mouse position in simulation pixels: `(x, y)`.
    pub mouse_pos: Option<(f32, f32)>,
    /// Smoothed mouse velocity in pixels/second.
    pub mouse_vel: (f32, f32),
    /// Whether mouse button is currently held.
    pub mouse_down: bool,
    /// Index of blob currently grabbed/dragged by mouse, if any.
    pub dragged_blob: Option<usize>,
    /// Timestamp of last mouse movement in simulation time.
    pub last_mouse_time: f32,
}

impl LavaLamp {
    /// Initializes a new lava lamp simulation with floating Material 3 shapes across depth layers.
    pub fn new(
        width: f32,
        height: f32,
        count: usize,
        layers: usize,
        shapes: &[MaterialShape],
        colors: &[Rgba],
    ) -> Self {
        let num_layers = layers.max(1);
        let palette = if colors.is_empty() {
            vec![
                gpui::rgb(0x6750a4).into(),
                gpui::rgb(0x7d5260).into(),
                gpui::rgb(0x625b71).into(),
                gpui::rgb(0x9a82db).into(),
                gpui::rgb(0xb58392).into(),
            ]
        } else {
            colors.to_vec()
        };

        let default_shapes = [
            MaterialShape::Cookie9Sided,
            MaterialShape::Flower,
            MaterialShape::Heart,
            MaterialShape::Arch,
            MaterialShape::Sunny,
            MaterialShape::Clover4Leaf,
            MaterialShape::Gem,
            MaterialShape::Boom,
            MaterialShape::Pill,
            MaterialShape::Cookie4Sided,
            MaterialShape::Ghostish,
            MaterialShape::Diamond,
        ];
        let shape_pool = if shapes.is_empty() {
            &default_shapes[..]
        } else {
            shapes
        };

        let w = width.max(100.0);
        let h = height.max(100.0);
        let base_radius = (w.min(h) * 0.06).clamp(32.0, 85.0);

        let mut blobs = Vec::with_capacity(count);
        for i in 0..count {
            let frac = (i as f32 + 0.5) / count.max(1) as f32;
            let shape = shape_pool[i % shape_pool.len()];
            let layer = i % num_layers;
            let color = palette[layer % palette.len()];

            // Alternate starting positions: some in top half (cooling), some in bottom half (warming)
            let (init_y, heat) = if i % 2 == 0 {
                (h * (0.55 + 0.35 * (frac * 3.14159).sin()), 0.8)
            } else {
                (h * (0.1 + 0.35 * (frac * 3.14159).cos().abs()), -0.8)
            };

            let init_x = w * (0.12 + 0.76 * ((i as f32 * 1.618033).fract()));
            let radius_var = base_radius * (0.8 + 0.45 * ((i as f32 * 0.732).fract()));

            blobs.push(LavaBlob {
                shape,
                target_shape: None,
                morph_t: 0.0,
                morph_duration: 2.0,
                layer,
                x: init_x,
                y: init_y,
                vx: if i % 2 == 0 { 12.0 } else { -12.0 },
                vy: if heat > 0.0 { -25.0 } else { 25.0 },
                base_radius: radius_var,
                radius: radius_var,
                rotation: (i as f32 * 45.0) % 360.0,
                rot_speed: if i % 2 == 0 {
                    8.0 + (i as f32 * 3.0) % 15.0
                } else {
                    -8.0 - (i as f32 * 3.0) % 15.0
                },
                deform: DeformSpring::with_scale(0.00015),
                heat,
                color,
                phase: i as f32 * 1.3,
                next_morph_in: 3.0 + ((i as f32 * 1.7).fract()) * 6.0,
            });
        }

        Self {
            blobs,
            width: w,
            height: h,
            smoothing: 32.0,
            speed: 1.0,
            scale: 1.0,
            churn: 50.0,
            drag: 0.2,
            spin: 1.0,
            max_v: 250.0,
            cursor_pull: 1.0,
            cursor_radius: 150.0,
            layers: num_layers,
            morph_chance: 20.0,
            shape_pool: shape_pool.to_vec(),
            time: 0.0,
            mouse_pos: None,
            mouse_vel: (0.0, 0.0),
            mouse_down: false,
            dragged_blob: None,
            last_mouse_time: 0.0,
        }
    }

    /// Sets physics and animation parameters.
    pub fn set_params(
        &mut self,
        smoothing: f32,
        speed: f32,
        scale: f32,
        churn: f32,
        drag: f32,
        spin: f32,
        max_v: f32,
        cursor_pull: f32,
        cursor_radius: f32,
        morph_chance: f32,
    ) {
        self.smoothing = smoothing.max(0.0);
        self.speed = speed.clamp(0.1, 5.0);
        self.scale = scale.clamp(0.2, 3.0);
        self.churn = churn.clamp(0.0, 100.0);
        self.drag = drag.clamp(0.0, 1.0);
        self.spin = spin.clamp(0.0, 5.0);
        self.max_v = max_v.max(10.0);
        self.cursor_pull = cursor_pull.clamp(0.0, 5.0);
        self.cursor_radius = cursor_radius.clamp(10.0, 600.0);
        self.morph_chance = morph_chance.clamp(0.0, 100.0);

        for b in &mut self.blobs {
            b.radius = b.base_radius * self.scale;
        }
    }

    /// Updates viewport dimensions when the window or screen is resized.
    pub fn set_size(&mut self, width: f32, height: f32) {
        if (self.width - width).abs() > 1.0 || (self.height - height).abs() > 1.0 {
            self.width = width.max(100.0);
            self.height = height.max(100.0);
            for b in &mut self.blobs {
                b.x = b.x.clamp(b.radius, (self.width - b.radius).max(b.radius));
                b.y = b.y.clamp(b.radius, (self.height - b.radius).max(b.radius));
            }
        }
    }

    /// Updates mouse position and tracks smoothed cursor velocity for fluid wake and stirring.
    pub fn on_mouse_move(&mut self, x: f32, y: f32) {
        if let Some((px, py)) = self.mouse_pos {
            let dx = x - px;
            let dy = y - py;
            self.mouse_vel.0 = self.mouse_vel.0 * 0.4 + dx * 0.6;
            self.mouse_vel.1 = self.mouse_vel.1 * 0.4 + dy * 0.6;
        }
        self.mouse_pos = Some((x, y));
        self.last_mouse_time = self.time;
    }

    /// Handles mouse press: grabs a nearby blob if clicked, or emits a radial fluid shockwave.
    pub fn on_mouse_down(&mut self, x: f32, y: f32) {
        self.mouse_pos = Some((x, y));
        self.mouse_down = true;
        self.last_mouse_time = self.time;

        // Check if user clicked near any blob to drag it
        let mut clicked = None;
        let mut min_dist = f32::MAX;
        for (i, b) in self.blobs.iter().enumerate() {
            let d = ((b.x - x).powi(2) + (b.y - y).powi(2)).sqrt();
            if d < b.radius * 1.35 && d < min_dist {
                min_dist = d;
                clicked = Some(i);
            }
        }

        if let Some(idx) = clicked {
            self.dragged_blob = Some(idx);
            let b = &mut self.blobs[idx];
            b.vx = 0.0;
            b.vy = 0.0;
        } else {
            // Click in empty space sends a shockwave pushing nearby blobs outward
            let shock_rad = (self.cursor_radius * 2.2).max(100.0);
            for b in &mut self.blobs {
                let dx = b.x - x;
                let dy = b.y - y;
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                if dist < shock_rad {
                    let factor = (1.0 - dist / shock_rad).powi(2) * 160.0 * self.cursor_pull;
                    b.vx += (dx / dist) * factor;
                    b.vy += (dy / dist) * factor;
                }
            }
        }
    }

    /// Releases dragged blob, flinging it with accumulated cursor velocity.
    pub fn on_mouse_up(&mut self) {
        self.mouse_down = false;
        if let Some(idx) = self.dragged_blob.take() {
            if let Some(b) = self.blobs.get_mut(idx) {
                let fling_speed = (self.mouse_vel.0.powi(2) + self.mouse_vel.1.powi(2)).sqrt();
                if fling_speed > 1.0 {
                    b.vx =
                        (self.mouse_vel.0 * 8.0 * self.cursor_pull).clamp(-self.max_v, self.max_v);
                    b.vy =
                        (self.mouse_vel.1 * 8.0 * self.cursor_pull).clamp(-self.max_v, self.max_v);
                }
            }
        }
    }

    /// Clears mouse interaction when pointer leaves the window or monitor.
    pub fn on_mouse_leave(&mut self) {
        self.mouse_pos = None;
        self.mouse_vel = (0.0, 0.0);
        self.mouse_down = false;
        self.dragged_blob = None;
    }

    /// Advances the simulation by elapsed time `dt` seconds.
    pub fn step(&mut self, dt: f32) {
        let dt = dt.clamp(0.001, 0.05);
        let spd = self.speed.clamp(0.1, 5.0);
        let sim_dt = dt * spd;
        let w = self.width;
        let h = self.height;

        // Mouse velocity decay
        self.mouse_vel.0 *= (0.02f32).powf(dt);
        self.mouse_vel.1 *= (0.02f32).powf(dt);

        // Clear idle mouse after 2.5s if not holding mouse button
        if self.time - self.last_mouse_time > 2.5 && !self.mouse_down {
            self.mouse_pos = None;
        }

        // 0. Dragged blob tracking
        if let (Some(idx), Some((mx, my))) = (self.dragged_blob, self.mouse_pos) {
            if let Some(b) = self.blobs.get_mut(idx) {
                let pull = 16.0 * self.cursor_pull;
                b.vx += ((mx - b.x) * pull - b.vx) * (20.0 * dt).min(1.0);
                b.vy += ((my - b.y) * pull - b.vy) * (20.0 * dt).min(1.0);
                b.heat = if b.y > h * 0.5 { 0.8 } else { -0.8 };
            }
        }

        // Hover stirring & fluid wake
        if let Some((mx, my)) = self.mouse_pos {
            let mv_speed = (self.mouse_vel.0.powi(2) + self.mouse_vel.1.powi(2)).sqrt();
            for (i, b) in self.blobs.iter_mut().enumerate() {
                if Some(i) == self.dragged_blob {
                    continue;
                }
                let dx = b.x - mx;
                let dy = b.y - my;
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                let interact_rad = b.radius + self.cursor_radius;
                if dist < interact_rad {
                    let norm = 1.0 - dist / interact_rad;
                    // Deflection force away from mouse
                    let deflect = norm * 50.0 * self.cursor_pull * sim_dt;
                    b.vx += (dx / dist) * deflect;
                    b.vy += (dy / dist) * deflect;

                    // Wake / stirring from cursor motion
                    if mv_speed > 2.0 {
                        b.vx += self.mouse_vel.0 * norm * 1.4 * self.cursor_pull * sim_dt;
                        b.vy += self.mouse_vel.1 * norm * 1.4 * self.cursor_pull * sim_dt;
                        // Impart spin from stirring
                        let cross = (dx * self.mouse_vel.1 - dy * self.mouse_vel.0) / dist;
                        b.rot_speed = (b.rot_speed + cross * 0.8 * self.cursor_pull * sim_dt)
                            .clamp(-60.0, 60.0);
                    }
                }
            }
        }

        let churn_mult = (self.churn / 50.0).clamp(0.0, 3.0);
        let drag_mult = (1.0 - self.drag * sim_dt).clamp(0.0, 1.0);

        // Morph progression & cooldown update
        for (i, b) in self.blobs.iter_mut().enumerate() {
            if let Some(target) = b.target_shape {
                b.morph_t += sim_dt / b.morph_duration.max(0.1);
                if b.morph_t >= 1.0 {
                    b.shape = target;
                    b.target_shape = None;
                    b.morph_t = 0.0;
                    b.next_morph_in = 4.0 + ((self.time + b.phase * 5.3).sin().abs()) * 6.0;
                }
            } else if self.morph_chance > 0.0 && self.shape_pool.len() > 1 {
                b.next_morph_in -= sim_dt;
                if b.next_morph_in <= 0.0 {
                    b.next_morph_in = 3.5 + ((self.time + b.phase * 4.1).sin().abs()) * 4.0;
                    let hash = ((self.time * 17.13 + (i as f32) * 31.415 + b.x * 0.07)
                        .sin()
                        .abs()
                        * 1000.0)
                        % 100.0;
                    if hash < self.morph_chance {
                        let pick_idx = ((hash * 13.0) as usize) % self.shape_pool.len();
                        let cand = self.shape_pool[pick_idx];
                        if cand != b.shape {
                            b.target_shape = Some(cand);
                            b.morph_t = 0.0;
                            b.morph_duration = 2.0;
                        }
                    }
                }
            }
        }

        for b in &mut self.blobs {
            // 1. Buoyancy cycle: warm at bottom, cool at top
            let heat_rate = 1.2 * churn_mult.max(0.1);
            if b.y > h * 0.72 {
                b.heat = (b.heat + heat_rate * sim_dt).min(1.0);
            } else if b.y < h * 0.28 {
                b.heat = (b.heat - heat_rate * sim_dt).max(-1.0);
            }

            // Buoyancy acceleration
            let target_vy = -45.0 * b.heat * churn_mult;
            b.vy += (target_vy - b.vy) * 2.0 * sim_dt;

            // 2. Horizontal gentle wander
            let h_drift = (self.time * 0.6 + b.phase).sin() * 22.0 * churn_mult;
            b.vx += (h_drift - b.vx) * 1.5 * sim_dt;

            // 3. Fluid drag
            b.vx *= drag_mult;
            b.vy *= drag_mult;

            // 4. Max velocity clamp
            let v_sq = b.vx * b.vx + b.vy * b.vy;
            let max_v = self.max_v;
            if v_sq > max_v * max_v && v_sq > 0.0 {
                let v = v_sq.sqrt();
                b.vx = (b.vx / v) * max_v;
                b.vy = (b.vy / v) * max_v;
            }

            // 5. Soft wall containment
            let margin = b.radius + 15.0;
            if b.x < margin {
                b.vx += 45.0 * sim_dt;
            } else if b.x > w - margin {
                b.vx -= 45.0 * sim_dt;
            }

            if b.y < margin {
                b.vy = b.vy.abs() * 0.8;
                b.heat = -1.0;
            } else if b.y > h - margin {
                b.vy = -b.vy.abs() * 0.8;
                b.heat = 1.0;
            }

            // 6. Position & rotation integration
            b.x = (b.x + b.vx * sim_dt).clamp(b.radius, (w - b.radius).max(b.radius));
            b.y = (b.y + b.vy * sim_dt).clamp(b.radius, (h - b.radius).max(b.radius));
            b.rotation = (b.rotation + b.rot_speed * self.spin * sim_dt).rem_euclid(360.0);

            // 7. Spring deformation along velocity
            b.deform.step(point(px(b.x), px(b.y)), dt);
        }

        self.time += sim_dt;
    }

    /// Converts current simulation state into shapes grouped by layer: `Vec<(layer_index, Vec<BlobShape>, layer_color)>`.
    /// Shapes across different layers do not merge with each other.
    pub fn layer_blob_shapes(
        &self,
        default_palette: &[Rgba],
    ) -> Vec<(usize, Vec<BlobShape>, Rgba)> {
        let num_layers = self.layers.max(1);
        let mut layers_map: HashMap<usize, Vec<BlobShape>> = HashMap::new();
        for b in &self.blobs {
            let mut s = BlobShape::new(b.shape, point(px(b.x), px(b.y)), px(b.radius))
                .rotation(b.rotation)
                .deform(b.deform.value)
                .color(b.color);
            if let Some(target) = b.target_shape {
                s = s.morph_to(target, b.morph_t);
            }
            layers_map.entry(b.layer % num_layers).or_default().push(s);
        }

        let mut out = Vec::with_capacity(num_layers);
        for l in 0..num_layers {
            let shapes = layers_map.remove(&l).unwrap_or_default();
            let color = if !default_palette.is_empty() {
                default_palette[l % default_palette.len()]
            } else {
                gpui::rgb(0x6750a4).into()
            };
            out.push((l, shapes, color));
        }
        out
    }

    /// Converts current simulation state into a flat slice of renderable [`BlobShape`]s.
    pub fn blob_shapes(&self) -> Vec<BlobShape> {
        self.blobs
            .iter()
            .map(|b| {
                let mut s = BlobShape::new(b.shape, point(px(b.x), px(b.y)), px(b.radius))
                    .rotation(b.rotation)
                    .deform(b.deform.value)
                    .color(b.color);
                if let Some(target) = b.target_shape {
                    s = s.morph_to(target, b.morph_t);
                }
                s
            })
            .collect()
    }

    /// Paints the merged lava lamp shapes layer-by-layer onto the given GPUI `window`.
    pub fn paint(&self, window: &mut Window, default_palette: &[Rgba]) {
        for (_layer, shapes, color) in self.layer_blob_shapes(default_palette) {
            if !shapes.is_empty() {
                paint_shape_blobs(window, &shapes, self.smoothing, color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::size;

    #[test]
    fn shape_blobs_separate_make_separate_contours() {
        let s1 = BlobShape::new(MaterialShape::Heart, point(px(100.0), px(100.0)), px(30.0));
        let s2 = BlobShape::new(MaterialShape::Flower, point(px(300.0), px(100.0)), px(30.0));
        let c = blob_shape_contours(&[s1, s2], 20.0, 2.0, 0.0);
        assert_eq!(
            c.len(),
            2,
            "Distant shapes should produce separate contours"
        );
    }

    #[test]
    fn shape_blobs_close_merge_into_one() {
        // Two shapes 40px apart with radius 30px each (overlapping / touching with 25px smoothing): one merged contour
        let s1 = BlobShape::new(MaterialShape::Heart, point(px(100.0), px(100.0)), px(30.0));
        let s2 = BlobShape::new(MaterialShape::Flower, point(px(140.0), px(100.0)), px(30.0));
        let c = blob_shape_contours(&[s1, s2], 25.0, 2.0, 0.0);
        assert_eq!(
            c.len(),
            1,
            "Touching shapes should merge into one liquid outline"
        );
    }

    #[test]
    fn lavalamp_steps_and_advances() {
        let mut lamp = LavaLamp::new(
            800.0,
            600.0,
            4,
            2,
            &[MaterialShape::Heart, MaterialShape::Arch],
            &[],
        );
        let y0 = lamp.blobs[0].y;
        lamp.step(0.016);
        assert_ne!(lamp.blobs[0].y, y0);
        assert_eq!(lamp.blob_shapes().len(), 4);
    }

    #[test]
    fn lavalamp_layers_separated() {
        let lamp = LavaLamp::new(800.0, 600.0, 6, 3, &[MaterialShape::Heart], &[]);
        let layers = lamp.layer_blob_shapes(&[]);
        assert_eq!(layers.len(), 3);
        assert_eq!(layers[0].1.len(), 2);
        assert_eq!(layers[1].1.len(), 2);
        assert_eq!(layers[2].1.len(), 2);
    }

    #[test]
    fn lavalamp_mouse_interaction() {
        let mut lamp = LavaLamp::new(
            800.0,
            600.0,
            4,
            1,
            &[MaterialShape::Heart, MaterialShape::Arch],
            &[],
        );
        let b0_x = lamp.blobs[0].x;
        let b0_y = lamp.blobs[0].y;

        // 1. Mouse move stirs nearby blobs
        lamp.on_mouse_move(b0_x + 10.0, b0_y + 10.0);
        lamp.on_mouse_move(b0_x + 50.0, b0_y + 50.0);
        lamp.step(0.016);
        assert!(lamp.blobs[0].vx.abs() > 0.0 || lamp.blobs[0].vy.abs() > 0.0);

        // 2. Mouse down on a blob initiates drag
        lamp.on_mouse_down(lamp.blobs[0].x, lamp.blobs[0].y);
        assert_eq!(lamp.dragged_blob, Some(0));

        // 3. Moving mouse drags the blob
        lamp.on_mouse_move(350.0, 250.0);
        lamp.step(0.016);
        lamp.step(0.016);

        // 4. Mouse up flings it
        lamp.on_mouse_up();
        assert_eq!(lamp.dragged_blob, None);
    }

    fn rect(x: f32, y: f32, w: f32, h: f32) -> BlobRect {
        BlobRect::new(Bounds::new(point(px(x), px(y)), size(px(w), px(h))))
    }

    #[test]
    fn sdf_matches_box() {
        let p = rect(0.0, 0.0, 100.0, 50.0).prepared();
        assert!(p.sdf(50.0, 25.0) < 0.0);
        assert!((p.sdf(110.0, 25.0) - 10.0).abs() < 0.01);
        assert!((p.sdf(50.0, -5.0) - 5.0).abs() < 0.01);
    }

    #[test]
    fn rounded_corner_distance() {
        let p = rect(0.0, 0.0, 100.0, 100.0).radius(20.0).prepared();
        // the corner arc is centered at (20, 20)
        let d = p.sdf(0.0, 0.0);
        assert!((d - (20.0 * 2f32.sqrt() - 20.0)).abs() < 0.01);
    }

    #[test]
    fn separate_shapes_make_separate_loops() {
        let c = blob_contours(
            &[rect(0.0, 0.0, 40.0, 40.0), rect(200.0, 0.0, 40.0, 40.0)],
            10.0,
            2.0,
            0.0,
        );
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn close_shapes_merge_into_one() {
        // 8px apart, within the 20px smoothing radius: one outline
        let c = blob_contours(
            &[rect(0.0, 0.0, 100.0, 40.0), rect(30.0, 48.0, 40.0, 40.0)],
            20.0,
            2.0,
            0.0,
        );
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn excluded_shapes_do_not_bridge() {
        let c = blob_contours(
            &[
                rect(0.0, 0.0, 100.0, 40.0).exclude(1),
                rect(30.0, 48.0, 40.0, 40.0),
            ],
            20.0,
            2.0,
            0.0,
        );
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn melting_into_an_edge_fills_the_concave_corner() {
        // a 100x40 shape standing on y = 100, melting into it with 20px fillets
        let shape = rect(50.0, 60.0, 100.0, 40.0).melt_into(Edge::Bottom(100.0), 20.0);
        let s = shape.prepared();
        let field = |x: f32, y: f32| merged_sdf(&[(0, s)], x, y, 4.0);
        // just outside the corner, inside the fillet: filled
        assert!(field(46.0, 98.0) < 0.0);
        // beyond the fillet radius along the edge: empty
        assert!(field(25.0, 98.0) > 0.0);
        // past the edge: only the shape itself
        assert!(field(46.0, 101.0) > 0.0);
        // and it traces as one outline wider than the shape
        let c = blob_contours(&[shape], 4.0, 1.0, 0.0);
        assert_eq!(c.len(), 1);
        let min_x = c[0].iter().map(|p| f32::from(p.x)).fold(f32::MAX, f32::min);
        assert!(
            min_x < 40.0,
            "fillet should reach out past the shape, got {min_x}"
        );
    }

    #[test]
    fn a_shape_away_from_the_edge_is_untouched() {
        let shape = rect(50.0, 20.0, 100.0, 40.0).melt_into(Edge::Bottom(100.0), 20.0);
        let s = shape.prepared();
        assert!(merged_sdf(&[(0, s)], 46.0, 98.0, 4.0) > 0.0);
    }

    #[test]
    fn spring_settles() {
        let mut s = DeformSpring::with_scale(0.0005);
        let mut t = 0.0;
        for i in 0..20 {
            s.step(point(px(i as f32 * 20.0), px(0.0)), 0.016);
            t += 0.016;
        }
        assert!(!s.value.is_identity(), "moving shape stretches");
        for _ in 0..400 {
            s.step(point(px(400.0), px(0.0)), 0.016);
        }
        assert!(
            s.value.is_identity() && !s.is_active(),
            "settles after stopping ({t})"
        );
    }

    #[test]
    fn lone_outline_lies_on_the_surface() {
        let r = rect(10.0, 20.0, 120.0, 60.0)
            .radii(4.0, 12.0, 20.0, 0.0)
            .deform(Deform {
                m00: 1.2,
                m01: 0.1,
                m11: 0.85,
            });
        let p = r.prepared();
        for pt in p.outline(0.0) {
            assert!(p.sdf(f32::from(pt.x), f32::from(pt.y)).abs() < 0.05);
        }
    }

    #[test]
    fn coarse_sampling_matches_full_sampling() {
        let shapes: Vec<(usize, Prepared)> = [
            rect(0.0, 0.0, 200.0, 40.0).radius(20.0),
            rect(60.0, 38.0, 90.0, 120.0).radius(16.0),
        ]
        .iter()
        .enumerate()
        .map(|(i, r)| (i, r.prepared()))
        .collect();
        let (vw, vh, step) = (140, 110, 2.0);
        let field = sample_field(&shapes, 18.0, 0.0, -30.0, -30.0, vw, vh, step);
        for j in 0..vh {
            for i in 0..vw {
                let exact = merged_sdf(
                    &shapes,
                    -30.0 + i as f32 * step,
                    -30.0 + j as f32 * step,
                    18.0,
                );
                assert_eq!(
                    field[j * vw + i] < 0.0,
                    exact < 0.0,
                    "sign differs at {i},{j}"
                );
            }
        }
    }
}
