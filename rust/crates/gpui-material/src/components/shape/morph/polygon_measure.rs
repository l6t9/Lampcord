use super::super::features::Feature;
use super::super::geometry::Cubic;
use super::super::polygon::RoundedPolygon;
use super::feature_mapping::ProgressableFeature;
use super::float_mapping::{DISTANCE_EPSILON, positive_modulo};
use gpui::Point;

#[derive(Clone, Debug)]
pub struct MeasuredCubic {
    pub cubic: Cubic,
    pub start_outline_progress: f32,
    pub end_outline_progress: f32,
    pub measured_size: f32,
}

impl MeasuredCubic {
    pub fn new(
        cubic: Cubic,
        start_outline_progress: f32,
        end_outline_progress: f32,
        measured_size: f32,
    ) -> Self {
        Self {
            cubic,
            start_outline_progress,
            end_outline_progress,
            measured_size,
        }
    }

    pub fn update_progress_range(&mut self, start_progress: f32, end_progress: f32) {
        if end_progress < start_progress {
            panic!("end_outline_progress must be >= start_outline_progress");
        }
        self.start_outline_progress = start_progress;
        self.end_outline_progress = end_progress;
    }

    pub fn cut_at_progress(
        &self,
        cut_outline_progress: f32,
        measurer: &LengthMeasurer,
    ) -> (MeasuredCubic, MeasuredCubic) {
        let bounded_cut_progress =
            cut_outline_progress.clamp(self.start_outline_progress, self.end_outline_progress);
        let outline_progress_size = self.end_outline_progress - self.start_outline_progress;
        let progress_from_start = bounded_cut_progress - self.start_outline_progress;

        let relative_progress = if outline_progress_size > 0.0 {
            progress_from_start / outline_progress_size
        } else {
            0.0
        };

        let t = measurer.find_cubic_cut_point(&self.cubic, relative_progress * self.measured_size);
        if t < 0.0 || t > 1.0 {
            panic!("Cubic cut point must be between 0 and 1");
        }

        let (c1, c2) = self.cubic.split(t);
        (
            MeasuredCubic::new(
                c1.clone(),
                self.start_outline_progress,
                bounded_cut_progress,
                measurer.measure_cubic(&c1),
            ),
            MeasuredCubic::new(
                c2.clone(),
                bounded_cut_progress,
                self.end_outline_progress,
                measurer.measure_cubic(&c2),
            ),
        )
    }
}

#[derive(Clone, Debug)]
pub struct LengthMeasurer {
    segments: usize,
}

impl LengthMeasurer {
    pub fn new() -> Self {
        Self { segments: 100 }
    }

    pub fn measure_cubic(&self, c: &Cubic) -> f32 {
        self.closest_progress_to(c, f32::INFINITY).1
    }

    pub fn find_cubic_cut_point(&self, c: &Cubic, m: f32) -> f32 {
        self.closest_progress_to(c, m).0
    }

    fn closest_progress_to(&self, cubic: &Cubic, threshold: f32) -> (f32, f32) {
        let mut total = 0.0;
        let mut remainder = threshold;
        let mut prev = Point::new(cubic.anchor0.x, cubic.anchor0.y);

        for i in 1..=self.segments {
            let progress = i as f32 / self.segments as f32;
            let point = cubic.point_on_curve(progress);
            let dx = point.x - prev.x;
            let dy = point.y - prev.y;
            let segment = (dx * dx + dy * dy).sqrt();

            if segment >= remainder {
                let p = progress - (1.0 - remainder / segment) / self.segments as f32;
                return (p, threshold);
            }

            remainder -= segment;
            total += segment;
            prev = point;
        }

        (1.0, total)
    }
}

#[derive(Clone, Debug)]
pub struct MeasuredPolygon<'a> {
    pub measurer: LengthMeasurer,
    pub features: Vec<ProgressableFeature<'a>>,
    pub cubics: Vec<MeasuredCubic>,
    pub outline_progress: Vec<f32>,
}

impl<'a> MeasuredPolygon<'a> {
    pub fn new(
        measurer: LengthMeasurer,
        features: Vec<ProgressableFeature<'a>>,
        cubics: &[Cubic],
        outline_progress: Vec<f32>,
    ) -> Self {
        if outline_progress.len() != cubics.len() + 1 {
            panic!("Outline progress size must be cubics size + 1");
        }
        if outline_progress.first().unwrap().abs() > f32::EPSILON {
            panic!("First outline progress value must be zero");
        }
        if (outline_progress.last().unwrap() - 1.0).abs() > f32::EPSILON {
            panic!("Last outline progress value must be one");
        }

        let mut start_outline_progress = 0.0;
        let mut measured_cubics = Vec::new();

        for i in 0..cubics.len() {
            if (outline_progress[i + 1] - outline_progress[i]) > DISTANCE_EPSILON {
                measured_cubics.push(MeasuredCubic::new(
                    cubics[i].clone(),
                    start_outline_progress,
                    outline_progress[i + 1],
                    measurer.measure_cubic(&cubics[i]),
                ));
                start_outline_progress = outline_progress[i + 1];
            }
        }

        if !measured_cubics.is_empty() {
            let last = measured_cubics.last_mut().unwrap();
            let start = last.start_outline_progress;
            last.update_progress_range(start, 1.0);
        } else {
            panic!("No cubics in measured polygon");
        }

        let mut actual_outline_progress = vec![0.0];
        for c in &measured_cubics {
            actual_outline_progress.push(c.end_outline_progress);
        }

        Self {
            measurer,
            features,
            cubics: measured_cubics,
            outline_progress: actual_outline_progress,
        }
    }

    pub fn cut_and_shift(&self, cutting_point: f32) -> Self {
        if cutting_point < 0.0 || cutting_point > 1.0 {
            panic!("Cutting point must be between 0 and 1");
        }
        if cutting_point < DISTANCE_EPSILON || cutting_point > 1.0 - DISTANCE_EPSILON {
            return self.clone();
        }

        let mut target_index = 0;
        for i in 0..self.cubics.len() {
            if cutting_point >= self.cubics[i].start_outline_progress
                && cutting_point <= self.cubics[i].end_outline_progress
            {
                target_index = i;
                break;
            }
        }

        let target = &self.cubics[target_index];
        let (b1, b2) = target.cut_at_progress(cutting_point, &self.measurer);

        let mut ret_cubics = Vec::new();
        ret_cubics.push(b2.cubic.clone());

        for i in 1..self.cubics.len() {
            ret_cubics.push(
                self.cubics[(i + target_index) % self.cubics.len()]
                    .cubic
                    .clone(),
            );
        }
        ret_cubics.push(b1.cubic.clone());

        let mut ret_outline_progress = Vec::with_capacity(self.cubics.len() + 2);
        for index in 0..(self.cubics.len() + 2) {
            if index == 0 {
                ret_outline_progress.push(0.0);
            } else if index == self.cubics.len() + 1 {
                ret_outline_progress.push(1.0);
            } else {
                let cubic_index = (target_index + index - 1) % self.cubics.len();
                ret_outline_progress.push(positive_modulo(
                    self.cubics[cubic_index].end_outline_progress - cutting_point,
                    1.0,
                ));
            }
        }

        let mut new_features = Vec::new();
        for f in &self.features {
            new_features.push(ProgressableFeature {
                progress: positive_modulo(f.progress - cutting_point, 1.0),
                feature: f.feature,
            });
        }

        Self::new(
            LengthMeasurer::new(),
            new_features,
            &ret_cubics,
            ret_outline_progress,
        )
    }

    pub fn measure_polygon(measurer: LengthMeasurer, polygon: &'a RoundedPolygon) -> Self {
        let mut cubics = Vec::new();
        let mut feature_to_cubic = Vec::new();

        for feature in &polygon.features {
            let feature_cubics = match feature {
                Feature::Corner { cubics, .. } => cubics,
                Feature::Edge { cubics } => cubics,
            };

            for (cubic_index, cubic) in feature_cubics.iter().enumerate() {
                if matches!(feature, Feature::Corner { .. })
                    && cubic_index == feature_cubics.len() / 2
                {
                    feature_to_cubic.push((feature, cubics.len()));
                }
                cubics.push(cubic.clone());
            }
        }

        let mut measures = vec![0.0];
        for cubic in &cubics {
            let measure = measurer.measure_cubic(cubic);
            if measure < 0.0 {
                panic!("Measured cubic must be >= 0");
            }
            measures.push(*measures.last().unwrap() + measure);
        }

        let total_measure = *measures.last().unwrap();
        let mut outline_progress = Vec::with_capacity(measures.len());
        for measure in measures {
            outline_progress.push(measure / total_measure);
        }

        let mut features = Vec::new();
        for (feature, idx) in feature_to_cubic {
            let progress = positive_modulo(
                (outline_progress[idx] + outline_progress[idx + 1]) / 2.0,
                1.0,
            );
            features.push(ProgressableFeature { progress, feature });
        }

        Self::new(measurer, features, &cubics, outline_progress)
    }
}
