//! Signed Distance Field (SDF) evaluation for Material 3 Expressive shapes.
//!
//! Provides accurate, high-performance signed distance queries for all 35 official
//! Material 3 shapes. The SDF is negative inside the shape, zero on the perimeter,
//! and positive outside, with exact Euclidean metric scaling.

use super::{MaterialShape, SHAPE_SAMPLE_COUNT};
use std::sync::OnceLock;

pub(crate) const GRID_SIZE: usize = 96;
pub(crate) const EXTENT: f32 = 0.85;

static SHAPE_SDFS: [OnceLock<Box<[f32; GRID_SIZE * GRID_SIZE]>>; 35] =
    [const { OnceLock::new() }; 35];

/// Computes the signed distance of a 2D point `(x, y)` to the perimeter of a `MaterialShape`.
///
/// The shape is centered at `(0.0, 0.0)` with a normalized half-extent of approximately `0.5`.
/// Returns:
/// - `< 0.0`: strictly inside the shape
/// - `== 0.0`: on the perimeter
/// - `> 0.0`: outside the shape
pub fn shape_sdf(shape: MaterialShape, x: f32, y: f32) -> f32 {
    let grid = SHAPE_SDFS[shape as usize].get_or_init(|| generate_shape_sdf(shape));

    let cx = x.clamp(-EXTENT, EXTENT);
    let cy = y.clamp(-EXTENT, EXTENT);

    let fx = (cx + EXTENT) / (2.0 * EXTENT) * (GRID_SIZE - 1) as f32;
    let fy = (cy + EXTENT) / (2.0 * EXTENT) * (GRID_SIZE - 1) as f32;

    let ix = (fx.floor() as usize).min(GRID_SIZE - 2);
    let iy = (fy.floor() as usize).min(GRID_SIZE - 2);
    let tx = fx - ix as f32;
    let ty = fy - iy as f32;

    let d00 = grid[iy * GRID_SIZE + ix];
    let d10 = grid[iy * GRID_SIZE + ix + 1];
    let d01 = grid[(iy + 1) * GRID_SIZE + ix];
    let d11 = grid[(iy + 1) * GRID_SIZE + ix + 1];

    let d0 = d00 * (1.0 - tx) + d10 * tx;
    let d1 = d01 * (1.0 - tx) + d11 * tx;
    let sampled = d0 * (1.0 - ty) + d1 * ty;

    if x == cx && y == cy {
        sampled
    } else {
        let dx = x - cx;
        let dy = y - cy;
        sampled + (dx * dx + dy * dy).sqrt()
    }
}

fn generate_shape_sdf(shape: MaterialShape) -> Box<[f32; GRID_SIZE * GRID_SIZE]> {
    let raw_pts = shape.points();
    // Center the points around (0.0, 0.0)
    let mut poly = [(0.0f32, 0.0f32); SHAPE_SAMPLE_COUNT];
    for i in 0..SHAPE_SAMPLE_COUNT {
        poly[i] = (raw_pts[i][0] - 0.5, raw_pts[i][1] - 0.5);
    }

    // Precalculate segments: (p0, p1, v, len_sq, bbox)
    struct Segment {
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        vx: f32,
        vy: f32,
        len_sq: f32,
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
    }

    let mut segments = Vec::with_capacity(SHAPE_SAMPLE_COUNT);
    for i in 0..SHAPE_SAMPLE_COUNT {
        let p0 = poly[i];
        let p1 = poly[(i + 1) % SHAPE_SAMPLE_COUNT];
        let vx = p1.0 - p0.0;
        let vy = p1.1 - p0.1;
        let len_sq = vx * vx + vy * vy;
        let min_x = p0.0.min(p1.0);
        let max_x = p0.0.max(p1.0);
        let min_y = p0.1.min(p1.1);
        let max_y = p0.1.max(p1.1);
        segments.push(Segment {
            x0: p0.0,
            y0: p0.1,
            x1: p1.0,
            y1: p1.1,
            vx,
            vy,
            len_sq,
            min_x,
            max_x,
            min_y,
            max_y,
        });
    }

    let mut grid = Box::new([0.0f32; GRID_SIZE * GRID_SIZE]);

    for j in 0..GRID_SIZE {
        let py = -EXTENT + (j as f32 / (GRID_SIZE - 1) as f32) * (2.0 * EXTENT);
        for i in 0..GRID_SIZE {
            let px = -EXTENT + (i as f32 / (GRID_SIZE - 1) as f32) * (2.0 * EXTENT);

            // 1. Compute winding number to determine inside/outside
            let mut wn = 0i32;
            let mut min_d_sq = f32::MAX;

            for seg in &segments {
                // Winding number check
                if seg.y0 <= py {
                    if seg.y1 > py {
                        let cross =
                            (seg.x1 - seg.x0) * (py - seg.y0) - (px - seg.x0) * (seg.y1 - seg.y0);
                        if cross > 0.0 {
                            wn += 1;
                        }
                    }
                } else if seg.y1 <= py {
                    let cross =
                        (seg.x1 - seg.x0) * (py - seg.y0) - (px - seg.x0) * (seg.y1 - seg.y0);
                    if cross < 0.0 {
                        wn -= 1;
                    }
                }

                // Distance to segment
                // Quick bounding box rejection
                if min_d_sq < f32::MAX {
                    let r = min_d_sq.sqrt();
                    if px < seg.min_x - r
                        || px > seg.max_x + r
                        || py < seg.min_y - r
                        || py > seg.max_y + r
                    {
                        continue;
                    }
                }

                if seg.len_sq < 1e-8 {
                    let dx = px - seg.x0;
                    let dy = py - seg.y0;
                    min_d_sq = min_d_sq.min(dx * dx + dy * dy);
                } else {
                    let wx = px - seg.x0;
                    let wy = py - seg.y0;
                    let c1 = wx * seg.vx + wy * seg.vy;
                    let t = (c1 / seg.len_sq).clamp(0.0, 1.0);
                    let qx = seg.x0 + t * seg.vx;
                    let qy = seg.y0 + t * seg.vy;
                    let dx = px - qx;
                    let dy = py - qy;
                    let d_sq = dx * dx + dy * dy;
                    if d_sq < min_d_sq {
                        min_d_sq = d_sq;
                    }
                }
            }

            let dist = min_d_sq.sqrt();
            let is_inside = wn != 0;
            grid[j * GRID_SIZE + i] = if is_inside { -dist } else { dist };
        }
    }

    grid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_sdf_matches_analytical() {
        // Center of circle should be inside (negative radius)
        let d_center = shape_sdf(MaterialShape::Circle, 0.0, 0.0);
        assert!(
            d_center < -0.4,
            "Center of circle should be ~ -0.5, got {d_center}"
        );

        // Far outside should be positive
        let d_outside = shape_sdf(MaterialShape::Circle, 1.0, 0.0);
        assert!(
            d_outside > 0.4,
            "Outside (1.0, 0.0) should be > 0.4, got {d_outside}"
        );

        // Perimeter should be close to 0.0
        let d_edge = shape_sdf(MaterialShape::Circle, 0.5, 0.0);
        assert!(
            d_edge.abs() < 0.05,
            "Perimeter should be ~0.0, got {d_edge}"
        );
    }

    #[test]
    fn heart_sdf_signs() {
        // Center of heart (0.0, 0.0) should be inside
        let d_in = shape_sdf(MaterialShape::Heart, 0.0, 0.0);
        assert!(d_in < 0.0, "Center of heart should be inside, got {d_in}");

        // Far point should be outside
        let d_out = shape_sdf(MaterialShape::Heart, 1.2, 1.2);
        assert!(d_out > 0.5, "Far point should be outside, got {d_out}");
    }
}
