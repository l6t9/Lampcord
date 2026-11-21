use super::features::Feature;
use super::geometry::{self, Cubic, PointExt, distance};
use gpui::Point;
use std::f32::consts::PI;

#[derive(Clone, Copy, Debug)]
pub struct CornerRounding {
    pub radius: f32,
    pub smoothing: f32,
}

impl CornerRounding {
    pub fn unrounded() -> Self {
        Self {
            radius: 0.0,
            smoothing: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RoundedPolygon {
    pub features: Vec<Feature>,
    pub center: Point<f32>,
    pub cubics: Vec<Cubic>,
}

impl RoundedPolygon {
    pub fn new(features: Vec<Feature>, center: Point<f32>) -> Self {
        let mut cubics = Vec::new();
        let mut first_cubic: Option<Cubic> = None;
        let mut last_cubic: Option<Cubic> = None;

        let mut first_feature_split_start: Option<Vec<Cubic>> = None;
        let mut first_feature_split_end: Option<Vec<Cubic>> = None;

        if !features.is_empty() && features[0].cubics().len() == 3 {
            let center_cubic = features[0].cubics()[1];
            let (start, end) = center_cubic.split(0.5);
            first_feature_split_start = Some(vec![features[0].cubics()[0], start]);
            first_feature_split_end = Some(vec![end, features[0].cubics()[2]]);
        }

        for i in 0..=features.len() {
            let feature_cubics = if i == 0 && first_feature_split_end.is_some() {
                first_feature_split_end.as_ref().unwrap()
            } else if i == features.len() {
                if first_feature_split_start.is_some() {
                    first_feature_split_start.as_ref().unwrap()
                } else {
                    break;
                }
            } else {
                features[i].cubics()
            };

            for cubic in feature_cubics {
                if !cubic.zero_length() {
                    if let Some(lc) = last_cubic {
                        cubics.push(lc);
                    }
                    last_cubic = Some(*cubic);
                    if first_cubic.is_none() {
                        first_cubic = Some(*cubic);
                    }
                } else {
                    if let Some(mut lc) = last_cubic {
                        lc.anchor1 = cubic.anchor1;
                        last_cubic = Some(lc);
                    }
                }
            }
        }

        if let (Some(lc), Some(fc)) = (last_cubic, first_cubic) {
            cubics.push(Cubic::new(
                lc.anchor0.x,
                lc.anchor0.y,
                lc.control0.x,
                lc.control0.y,
                lc.control1.x,
                lc.control1.y,
                fc.anchor0.x,
                fc.anchor0.y,
            ));
        } else {
            cubics.push(Cubic::new(
                center.x, center.y, center.x, center.y, center.x, center.y, center.x, center.y,
            ));
        }

        Self {
            features,
            center,
            cubics,
        }
    }

    pub fn circle(num_vertices: usize, radius: f32, center_x: f32, center_y: f32) -> Self {
        let theta = PI / (num_vertices as f32);
        let polygon_radius = radius / theta.cos();
        Self::from_num_verts(
            num_vertices,
            polygon_radius,
            center_x,
            center_y,
            CornerRounding {
                radius,
                smoothing: 0.0,
            },
            None,
        )
    }

    pub fn from_num_verts(
        num_vertices: usize,
        radius: f32,
        center_x: f32,
        center_y: f32,
        rounding: CornerRounding,
        per_vertex_rounding: Option<Vec<CornerRounding>>,
    ) -> Self {
        let mut vertices = Vec::with_capacity(num_vertices * 2);
        for i in 0..num_vertices {
            let angle = (PI / num_vertices as f32) * 2.0 * (i as f32);
            let v = geometry::radial_to_cartesian(radius, angle);
            vertices.push(v.x + center_x);
            vertices.push(v.y + center_y);
        }
        Self::from_vertices(&vertices, rounding, per_vertex_rounding, center_x, center_y)
    }

    pub fn star(
        num_vertices_per_radius: usize,
        radius: f32,
        inner_radius: f32,
        rounding: CornerRounding,
        inner_rounding: Option<CornerRounding>,
        center_x: f32,
        center_y: f32,
    ) -> Self {
        let mut vertices = Vec::with_capacity(num_vertices_per_radius * 4);
        for i in 0..num_vertices_per_radius {
            let angle1 = (PI / num_vertices_per_radius as f32) * 2.0 * (i as f32);
            let v1 = geometry::radial_to_cartesian(radius, angle1);
            vertices.push(v1.x + center_x);
            vertices.push(v1.y + center_y);

            let angle2 = (PI / num_vertices_per_radius as f32) * (2.0 * (i as f32) + 1.0);
            let v2 = geometry::radial_to_cartesian(inner_radius, angle2);
            vertices.push(v2.x + center_x);
            vertices.push(v2.y + center_y);
        }

        let per_vertex = if let Some(ir) = inner_rounding {
            let mut pv = Vec::with_capacity(num_vertices_per_radius * 2);
            for _ in 0..num_vertices_per_radius {
                pv.push(rounding);
                pv.push(ir);
            }
            Some(pv)
        } else {
            None
        };

        Self::from_vertices(&vertices, rounding, per_vertex, center_x, center_y)
    }

    pub fn from_vertices(
        vertices: &[f32],
        rounding: CornerRounding,
        per_vertex_rounding: Option<Vec<CornerRounding>>,
        center_x: f32,
        center_y: f32,
    ) -> Self {
        let n = vertices.len() / 2;
        let mut rounded_corners = Vec::with_capacity(n);
        for i in 0..n {
            let vtx_rounding = if let Some(ref pv) = per_vertex_rounding {
                pv[i]
            } else {
                rounding
            };
            let prev_index = ((i + n - 1) % n) * 2;
            let next_index = ((i + 1) % n) * 2;
            rounded_corners.push(RoundedCorner::new(
                Point {
                    x: vertices[prev_index],
                    y: vertices[prev_index + 1],
                },
                Point {
                    x: vertices[i * 2],
                    y: vertices[i * 2 + 1],
                },
                Point {
                    x: vertices[next_index],
                    y: vertices[next_index + 1],
                },
                Some(vtx_rounding),
            ));
        }

        let mut cut_adjusts = Vec::with_capacity(n);
        for ix in 0..n {
            let expected_round_cut = rounded_corners[ix].expected_round_cut
                + rounded_corners[(ix + 1) % n].expected_round_cut;
            let expected_cut =
                rounded_corners[ix].expected_cut() + rounded_corners[(ix + 1) % n].expected_cut();
            let side_size = distance(
                vertices[ix * 2] - vertices[((ix + 1) % n) * 2],
                vertices[ix * 2 + 1] - vertices[((ix + 1) % n) * 2 + 1],
            );

            if expected_round_cut > side_size {
                cut_adjusts.push((side_size / expected_round_cut, 0.0));
            } else if expected_cut > side_size {
                cut_adjusts.push((
                    1.0,
                    (side_size - expected_round_cut) / (expected_cut - expected_round_cut),
                ));
            } else {
                cut_adjusts.push((1.0, 1.0));
            }
        }

        let mut corners = Vec::with_capacity(n);
        for i in 0..n {
            let (rc0, c0) = cut_adjusts[(i + n - 1) % n];
            let allowed_cut_0 = rounded_corners[i].expected_round_cut * rc0
                + (rounded_corners[i].expected_cut() - rounded_corners[i].expected_round_cut) * c0;

            let (rc1, c1) = cut_adjusts[i];
            let allowed_cut_1 = rounded_corners[i].expected_round_cut * rc1
                + (rounded_corners[i].expected_cut() - rounded_corners[i].expected_round_cut) * c1;

            corners.push(rounded_corners[i].get_cubics(allowed_cut_0, allowed_cut_1));
        }

        let mut temp_features = Vec::new();
        for i in 0..n {
            let prev_vtx_index = (i + n - 1) % n;
            let next_vtx_index = (i + 1) % n;
            let curr_vertex = Point {
                x: vertices[i * 2],
                y: vertices[i * 2 + 1],
            };
            let prev_vertex = Point {
                x: vertices[prev_vtx_index * 2],
                y: vertices[prev_vtx_index * 2 + 1],
            };
            let next_vertex = Point {
                x: vertices[next_vtx_index * 2],
                y: vertices[next_vtx_index * 2 + 1],
            };

            let d0 = Point {
                x: prev_vertex.x - curr_vertex.x,
                y: prev_vertex.y - curr_vertex.y,
            };
            let d1 = Point {
                x: next_vertex.x - curr_vertex.x,
                y: next_vertex.y - curr_vertex.y,
            };
            let convex = (d0.x * d1.y - d0.y * d1.x) < 0.0; // clockwise cross product

            temp_features.push(Feature::Corner {
                cubics: corners[i].clone(),
                convex,
            });

            temp_features.push(Feature::Edge {
                cubics: vec![Cubic::straight_line(
                    corners[i].last().unwrap().anchor1.x,
                    corners[i].last().unwrap().anchor1.y,
                    corners[(i + 1) % n].first().unwrap().anchor0.x,
                    corners[(i + 1) % n].first().unwrap().anchor0.y,
                )],
            });
        }

        Self::new(
            temp_features,
            Point {
                x: center_x,
                y: center_y,
            },
        )
    }
}

#[allow(dead_code)]
struct RoundedCorner {
    p0: Point<f32>,
    p1: Point<f32>,
    p2: Point<f32>,
    rounding: Option<CornerRounding>,
    d1: Point<f32>,
    d2: Point<f32>,
    corner_radius: f32,
    smoothing: f32,
    cos_angle: f32,
    sin_angle: f32,
    expected_round_cut: f32,
}

impl RoundedCorner {
    fn new(
        p0: Point<f32>,
        p1: Point<f32>,
        p2: Point<f32>,
        rounding: Option<CornerRounding>,
    ) -> Self {
        let v01 = Point {
            x: p0.x - p1.x,
            y: p0.y - p1.y,
        };
        let v21 = Point {
            x: p2.x - p1.x,
            y: p2.y - p1.y,
        };
        let d01 = v01.distance();
        let d21 = v21.distance();

        let (d1, d2, corner_radius, smoothing, cos_angle, sin_angle, expected_round_cut) =
            if d01 > 0.0 && d21 > 0.0 {
                let nd1 = Point {
                    x: v01.x / d01,
                    y: v01.y / d01,
                };
                let nd2 = Point {
                    x: v21.x / d21,
                    y: v21.y / d21,
                };
                let cr = rounding.as_ref().map(|r| r.radius).unwrap_or(0.0);
                let sm = rounding.as_ref().map(|r| r.smoothing).unwrap_or(0.0);
                let ca = nd1.dot_product(nd2);
                let sa = (1.0 - ca * ca).sqrt().max(0.0);
                let erc = if sa > 1e-3 { cr * (ca + 1.0) / sa } else { 0.0 };
                (nd1, nd2, cr, sm, ca, sa, erc)
            } else {
                (
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 0.0, y: 0.0 },
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                )
            };

        Self {
            p0,
            p1,
            p2,
            rounding,
            d1,
            d2,
            corner_radius,
            smoothing,
            cos_angle,
            sin_angle,
            expected_round_cut,
        }
    }

    fn expected_cut(&self) -> f32 {
        (1.0 + self.smoothing) * self.expected_round_cut
    }

    fn get_cubics(&self, allowed_cut_0: f32, allowed_cut_1: f32) -> Vec<Cubic> {
        let allowed_cut = allowed_cut_0.min(allowed_cut_1);
        if self.expected_round_cut < 1e-4 || allowed_cut < 1e-4 || self.corner_radius < 1e-4 {
            return vec![Cubic::straight_line(
                self.p1.x, self.p1.y, self.p1.x, self.p1.y,
            )];
        }

        let actual_round_cut = allowed_cut.min(self.expected_round_cut);

        let actual_smoothing_0 = if allowed_cut_0 > self.expected_cut() {
            self.smoothing
        } else if allowed_cut_0 > self.expected_round_cut {
            self.smoothing * (allowed_cut_0 - self.expected_round_cut)
                / (self.expected_cut() - self.expected_round_cut)
        } else {
            0.0
        };

        let actual_smoothing_1 = if allowed_cut_1 > self.expected_cut() {
            self.smoothing
        } else if allowed_cut_1 > self.expected_round_cut {
            self.smoothing * (allowed_cut_1 - self.expected_round_cut)
                / (self.expected_cut() - self.expected_round_cut)
        } else {
            0.0
        };

        let actual_r = self.corner_radius * actual_round_cut / self.expected_round_cut;
        let center_distance = (actual_r * actual_r + actual_round_cut * actual_round_cut).sqrt();
        let center_dir = Point {
            x: self.d1.x + self.d2.x,
            y: self.d1.y + self.d2.y,
        }
        .direction();
        let center = Point {
            x: self.p1.x + center_dir.x * center_distance,
            y: self.p1.y + center_dir.y * center_distance,
        };

        let circle_intersection_0 = Point {
            x: self.p1.x + self.d1.x * actual_round_cut,
            y: self.p1.y + self.d1.y * actual_round_cut,
        };
        let circle_intersection_2 = Point {
            x: self.p1.x + self.d2.x * actual_round_cut,
            y: self.p1.y + self.d2.y * actual_round_cut,
        };

        let flanking_0 = Self::compute_flanking_curve(
            actual_round_cut,
            actual_smoothing_0,
            self.p1,
            self.p0,
            circle_intersection_0,
            circle_intersection_2,
            center,
            actual_r,
        );

        let flanking_2 = Self::compute_flanking_curve(
            actual_round_cut,
            actual_smoothing_1,
            self.p1,
            self.p2,
            circle_intersection_2,
            circle_intersection_0,
            center,
            actual_r,
        )
        .reverse();

        vec![
            flanking_0,
            Cubic::circular_arc(
                center.x,
                center.y,
                flanking_0.anchor1.x,
                flanking_0.anchor1.y,
                flanking_2.anchor0.x,
                flanking_2.anchor0.y,
            ),
            flanking_2,
        ]
    }

    fn compute_flanking_curve(
        actual_round_cut: f32,
        actual_smoothing_values: f32,
        corner: Point<f32>,
        side_start: Point<f32>,
        circle_segment_intersection: Point<f32>,
        other_circle_segment_intersection: Point<f32>,
        circle_center: Point<f32>,
        actual_r: f32,
    ) -> Cubic {
        let side_direction = Point {
            x: side_start.x - corner.x,
            y: side_start.y - corner.y,
        }
        .direction();
        let curve_start = Point {
            x: corner.x + side_direction.x * actual_round_cut * (1.0 + actual_smoothing_values),
            y: corner.y + side_direction.y * actual_round_cut * (1.0 + actual_smoothing_values),
        };

        let p_mid = Point {
            x: (circle_segment_intersection.x + other_circle_segment_intersection.x) / 2.0,
            y: (circle_segment_intersection.y + other_circle_segment_intersection.y) / 2.0,
        };
        let p = geometry::interpolate_point(
            circle_segment_intersection,
            p_mid,
            actual_smoothing_values,
        );
        let curve_end_dir = Point {
            x: p.x - circle_center.x,
            y: p.y - circle_center.y,
        }
        .direction();
        let curve_end = Point {
            x: circle_center.x + curve_end_dir.x * actual_r,
            y: circle_center.y + curve_end_dir.y * actual_r,
        };

        let circle_tangent = Point {
            x: curve_end.x - circle_center.x,
            y: curve_end.y - circle_center.y,
        }
        .rotate90();

        let anchor_end =
            Self::line_intersection(side_start, side_direction, curve_end, circle_tangent)
                .unwrap_or(circle_segment_intersection);

        let anchor_start = Point {
            x: (curve_start.x + anchor_end.x * 2.0) / 3.0,
            y: (curve_start.y + anchor_end.y * 2.0) / 3.0,
        };

        Cubic::new(
            curve_start.x,
            curve_start.y,
            anchor_start.x,
            anchor_start.y,
            anchor_end.x,
            anchor_end.y,
            curve_end.x,
            curve_end.y,
        )
    }

    fn line_intersection(
        p0: Point<f32>,
        d0: Point<f32>,
        p1: Point<f32>,
        d1: Point<f32>,
    ) -> Option<Point<f32>> {
        let rotated_d1 = d1.rotate90();
        let den = d0.dot_product(rotated_d1);
        if den.abs() < 1e-4 {
            return None;
        }
        let num = Point {
            x: p1.x - p0.x,
            y: p1.y - p0.y,
        }
        .dot_product(rotated_d1);
        if den.abs() < 1e-4 * num.abs() {
            return None;
        }
        let k = num / den;
        Some(Point {
            x: p0.x + d0.x * k,
            y: p0.y + d0.y * k,
        })
    }
}

impl RoundedPolygon {
    pub fn transformed<F>(&self, mut f: F) -> Self
    where
        F: FnMut(f32, f32) -> (f32, f32) + Clone,
    {
        let features: Vec<Feature> = self
            .features
            .iter()
            .map(|feat| feat.transformed(f.clone()))
            .collect();
        let (cx, cy) = f(self.center.x, self.center.y);
        Self::new(features, Point { x: cx, y: cy })
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        let mut min_x = std::f32::MAX;
        let mut min_y = std::f32::MAX;
        let mut max_x = std::f32::MIN;
        let mut max_y = std::f32::MIN;
        for c in &self.cubics {
            min_x = min_x
                .min(c.anchor0.x)
                .min(c.control0.x)
                .min(c.control1.x)
                .min(c.anchor1.x);
            min_y = min_y
                .min(c.anchor0.y)
                .min(c.control0.y)
                .min(c.control1.y)
                .min(c.anchor1.y);
            max_x = max_x
                .max(c.anchor0.x)
                .max(c.control0.x)
                .max(c.control1.x)
                .max(c.anchor1.x);
            max_y = max_y
                .max(c.anchor0.y)
                .max(c.control0.y)
                .max(c.control1.y)
                .max(c.anchor1.y);
        }
        (min_x, min_y, max_x, max_y)
    }

    pub fn normalized(&self) -> Self {
        let (min_x, min_y, max_x, max_y) = self.bounds();
        let width = max_x - min_x;
        let height = max_y - min_y;
        let side = width.max(height);
        if side < 1e-4 {
            return self.clone();
        }
        let scale = 1.0 / side;
        let offset_x = -min_x * scale + (1.0 - width * scale) / 2.0;
        let offset_y = -min_y * scale + (1.0 - height * scale) / 2.0;
        self.transformed(|x, y| (x * scale + offset_x, y * scale + offset_y))
    }
}
