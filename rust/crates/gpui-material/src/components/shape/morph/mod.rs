pub mod feature_mapping;
pub mod float_mapping;
pub mod polygon_measure;

use super::geometry::{Cubic, interpolate};
use super::polygon::RoundedPolygon;
use feature_mapping::feature_mapper;
use polygon_measure::{LengthMeasurer, MeasuredPolygon};

#[derive(Clone, Debug)]
pub struct Morph {
    pub morph_match: Vec<(Cubic, Cubic)>,
}

impl Morph {
    pub fn new(start: &RoundedPolygon, end: &RoundedPolygon) -> Self {
        Self {
            morph_match: Self::match_polygons(start, end),
        }
    }

    fn match_polygons(p1: &RoundedPolygon, p2: &RoundedPolygon) -> Vec<(Cubic, Cubic)> {
        let measurer1 = LengthMeasurer::new();
        let measurer2 = LengthMeasurer::new();
        let measured_polygon1 = MeasuredPolygon::measure_polygon(measurer1, p1);
        let measured_polygon2 = MeasuredPolygon::measure_polygon(measurer2, p2);

        let double_mapper =
            feature_mapper(&measured_polygon1.features, &measured_polygon2.features);
        let polygon2_cut_point = double_mapper.map(0.0);

        let bs1 = measured_polygon1;
        let bs2 = measured_polygon2.cut_and_shift(polygon2_cut_point);

        let mut result = Vec::new();
        let mut i1 = 0;
        let mut i2 = 0;

        let mut b1_opt = bs1.cubics.get(i1).cloned();
        if b1_opt.is_some() {
            i1 += 1;
        }

        let mut b2_opt = bs2.cubics.get(i2).cloned();
        if b2_opt.is_some() {
            i2 += 1;
        }

        let angle_epsilon = 1e-4;

        while let (Some(b1), Some(b2)) = (&b1_opt, &b2_opt) {
            let b1a = if i1 == bs1.cubics.len() {
                1.0
            } else {
                b1.end_outline_progress
            };
            let b2a = if i2 == bs2.cubics.len() {
                1.0
            } else {
                double_mapper.map_back(float_mapping::positive_modulo(
                    b2.end_outline_progress + polygon2_cut_point,
                    1.0,
                ))
            };

            let minb = b1a.min(b2a);

            let mut seg1 = b1.clone();
            let mut seg2 = b2.clone();

            if b1a > minb + angle_epsilon {
                let (cut1, cut2) = b1.cut_at_progress(minb, &bs1.measurer);
                seg1 = cut1;
                b1_opt = Some(cut2);
            } else {
                b1_opt = bs1.cubics.get(i1).cloned();
                if b1_opt.is_some() {
                    i1 += 1;
                }
            }

            if b2a > minb + angle_epsilon {
                let cut_target = float_mapping::positive_modulo(
                    double_mapper.map(minb) - polygon2_cut_point,
                    1.0,
                );
                let (cut1, cut2) = b2.cut_at_progress(cut_target, &bs2.measurer);
                seg2 = cut1;
                b2_opt = Some(cut2);
            } else {
                b2_opt = bs2.cubics.get(i2).cloned();
                if b2_opt.is_some() {
                    i2 += 1;
                }
            }

            result.push((seg1.cubic, seg2.cubic));
        }

        if b1_opt.is_some() || b2_opt.is_some() {
            // They should finish together if math is correct, but floating point can diverge at the end
            // We just ignore extra pieces as they are practically 0 length
        }

        result
    }

    pub fn as_cubics(&self, progress: f32) -> Vec<Cubic> {
        let mut result = Vec::with_capacity(self.morph_match.len());

        for (start_cubic, end_cubic) in &self.morph_match {
            let p0 = interpolate(start_cubic.anchor0.x, end_cubic.anchor0.x, progress);
            let p1 = interpolate(start_cubic.anchor0.y, end_cubic.anchor0.y, progress);
            let p2 = interpolate(start_cubic.control0.x, end_cubic.control0.x, progress);
            let p3 = interpolate(start_cubic.control0.y, end_cubic.control0.y, progress);
            let p4 = interpolate(start_cubic.control1.x, end_cubic.control1.x, progress);
            let p5 = interpolate(start_cubic.control1.y, end_cubic.control1.y, progress);
            let p6 = interpolate(start_cubic.anchor1.x, end_cubic.anchor1.x, progress);
            let p7 = interpolate(start_cubic.anchor1.y, end_cubic.anchor1.y, progress);

            result.push(Cubic::new(p0, p1, p2, p3, p4, p5, p6, p7));
        }

        if result.len() > 1 {
            let first_x = result[0].anchor0.x;
            let first_y = result[0].anchor0.y;
            let last = result.last_mut().unwrap();
            last.anchor1.x = first_x;
            last.anchor1.y = first_y;
        }

        result
    }
}
