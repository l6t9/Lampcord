use super::super::features::Feature;
use super::float_mapping::{DISTANCE_EPSILON, DoubleMapper, progress_distance, progress_in_range};
use gpui::Point;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct ProgressableFeature<'a> {
    pub progress: f32,
    pub feature: &'a Feature,
}

#[derive(Clone, Debug)]
struct DistanceVertex<'a> {
    distance: f32,
    f1: &'a ProgressableFeature<'a>,
    f2: &'a ProgressableFeature<'a>,
}

struct MappingHelper<'a> {
    mapping: Vec<(f32, f32)>,
    used_f1: HashSet<*const ProgressableFeature<'a>>,
    used_f2: HashSet<*const ProgressableFeature<'a>>,
}

impl<'a> MappingHelper<'a> {
    fn new() -> Self {
        Self {
            mapping: Vec::new(),
            used_f1: HashSet::new(),
            used_f2: HashSet::new(),
        }
    }

    fn add_mapping(&mut self, f1: &'a ProgressableFeature<'a>, f2: &'a ProgressableFeature<'a>) {
        let p1 = f1 as *const ProgressableFeature<'a>;
        let p2 = f2 as *const ProgressableFeature<'a>;

        if self.used_f1.contains(&p1) || self.used_f2.contains(&p2) {
            return;
        }

        let insertion_index = self.mapping.partition_point(|&(val, _)| val < f1.progress);
        let n = self.mapping.len();

        if n >= 1 {
            let before_idx = (insertion_index + n - 1) % n;
            let after_idx = insertion_index % n;
            let before1 = self.mapping[before_idx].0;
            let before2 = self.mapping[before_idx].1;
            let after1 = self.mapping[after_idx].0;
            let after2 = self.mapping[after_idx].1;

            if progress_distance(f1.progress, before1) < DISTANCE_EPSILON
                || progress_distance(f1.progress, after1) < DISTANCE_EPSILON
                || progress_distance(f2.progress, before2) < DISTANCE_EPSILON
                || progress_distance(f2.progress, after2) < DISTANCE_EPSILON
            {
                return;
            }

            if n > 1 && !progress_in_range(f2.progress, before2, after2) {
                return;
            }
        }

        self.mapping
            .insert(insertion_index, (f1.progress, f2.progress));
        self.used_f1.insert(p1);
        self.used_f2.insert(p2);
    }
}

pub fn feature_mapper(
    features1: &[ProgressableFeature],
    features2: &[ProgressableFeature],
) -> DoubleMapper {
    let filtered_features1: Vec<&ProgressableFeature> = features1
        .iter()
        .filter(|f| matches!(f.feature, Feature::Corner { .. }))
        .collect();

    let filtered_features2: Vec<&ProgressableFeature> = features2
        .iter()
        .filter(|f| matches!(f.feature, Feature::Corner { .. }))
        .collect();

    let feature_progress_mapping = do_mapping(&filtered_features1, &filtered_features2);
    DoubleMapper::new(&feature_progress_mapping)
}

fn do_mapping(
    features1: &[&ProgressableFeature],
    features2: &[&ProgressableFeature],
) -> Vec<(f32, f32)> {
    let mut distance_vertex_list = Vec::new();

    for &f1 in features1 {
        for &f2 in features2 {
            let d = feature_dist_squared(f1.feature, f2.feature);
            if d < f32::MAX {
                distance_vertex_list.push(DistanceVertex {
                    distance: d,
                    f1,
                    f2,
                });
            }
        }
    }

    distance_vertex_list.sort_by(|a, b| {
        a.distance
            .partial_cmp(&b.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if distance_vertex_list.is_empty() {
        return vec![(0.0, 0.0), (0.5, 0.5)];
    }

    if distance_vertex_list.len() == 1 {
        let p1 = distance_vertex_list[0].f1.progress;
        let p2 = distance_vertex_list[0].f2.progress;
        return vec![(p1, p2), ((p1 + 0.5) % 1.0, (p2 + 0.5) % 1.0)];
    }

    let mut helper = MappingHelper::new();
    for vertex in distance_vertex_list {
        helper.add_mapping(vertex.f1, vertex.f2);
    }

    helper.mapping
}

fn feature_dist_squared(f1: &Feature, f2: &Feature) -> f32 {
    if let (Feature::Corner { convex: c1, .. }, Feature::Corner { convex: c2, .. }) = (f1, f2) {
        if c1 != c2 {
            return f32::MAX;
        }
    }

    let p1 = feature_representative_point(f1);
    let p2 = feature_representative_point(f2);
    let dx = p1.x - p2.x;
    let dy = p1.y - p2.y;
    dx * dx + dy * dy
}

fn feature_representative_point(feature: &Feature) -> Point<f32> {
    let cubics = match feature {
        Feature::Corner { cubics, .. } => cubics,
        Feature::Edge { cubics, .. } => cubics,
    };

    if cubics.is_empty() {
        return Point::new(0.0, 0.0);
    }

    let first = &cubics[0];
    let last = &cubics[cubics.len() - 1];

    Point::new(
        (first.anchor0.x + last.anchor1.x) / 2.0,
        (first.anchor0.y + last.anchor1.y) / 2.0,
    )
}
