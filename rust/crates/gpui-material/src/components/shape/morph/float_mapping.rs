pub const DISTANCE_EPSILON: f32 = 1e-4;

/// Positive modulo for floating point numbers
pub fn positive_modulo(a: f32, b: f32) -> f32 {
    let m = a % b;
    if m < 0.0 { m + b } else { m }
}

/// Checks if the given progress is in the given progress range. Since progress
/// is in the [0..1) interval and wraps, there is a special case when
/// progressTo < progressFrom.
pub fn progress_in_range(progress: f32, progress_from: f32, progress_to: f32) -> bool {
    if progress_to >= progress_from {
        progress >= progress_from && progress <= progress_to
    } else {
        progress >= progress_from || progress <= progress_to
    }
}

/// Distance between two progress values. Since progress wraps around,
/// we consider a difference of 0.99 as a distance of 0.01.
pub fn progress_distance(p1: f32, p2: f32) -> f32 {
    let diff = (p1 - p2).abs();
    diff.min(1.0 - diff)
}

/// Validates that a list of progress values are all in range [0.0, 1.0)
/// and monotonically increasing, with exception of maybe one wrap-around.
pub fn validate_progress(p: &[f32]) {
    if p.len() < 2 {
        return;
    }

    let mut prev = *p.last().unwrap();
    let mut wraps = 0;

    for &curr in p {
        if curr < 0.0 || curr >= 1.0 {
            panic!("FloatMapping - Progress outside of range [0, 1): {}", curr);
        }

        if progress_distance(curr, prev) <= DISTANCE_EPSILON {
            panic!(
                "FloatMapping - Progress repeats a value: {} and {}",
                curr, prev
            );
        }

        if curr < prev {
            wraps += 1;
            if wraps > 1 {
                panic!("FloatMapping - Progress wraps more than once");
            }
        }
        prev = curr;
    }
}

/// Maps from one set of progress values to another using linear interpolation.
pub fn linear_map(x_values: &[f32], y_values: &[f32], x: f32) -> f32 {
    if x < 0.0 || x > 1.0 {
        panic!("Invalid progress value: {}", x);
    }

    // Exact match for 1.0 logic, though progress usually is < 1.0
    let x = if x >= 1.0 { 0.0 } else { x };

    let mut segment_start_index = 0;
    for i in 0..x_values.len() {
        let next_idx = (i + 1) % x_values.len();
        if progress_in_range(x, x_values[i], x_values[next_idx]) {
            segment_start_index = i;
            break;
        }
    }

    let segment_end_index = (segment_start_index + 1) % x_values.len();

    let segment_size_x = positive_modulo(
        x_values[segment_end_index] - x_values[segment_start_index],
        1.0,
    );
    let segment_size_y = positive_modulo(
        y_values[segment_end_index] - y_values[segment_start_index],
        1.0,
    );

    let position_in_segment = if segment_size_x < 0.001 {
        0.5
    } else {
        positive_modulo(x - x_values[segment_start_index], 1.0) / segment_size_x
    };

    positive_modulo(
        y_values[segment_start_index] + segment_size_y * position_in_segment,
        1.0,
    )
}

#[derive(Clone, Debug)]
pub struct DoubleMapper {
    source_values: Vec<f32>,
    target_values: Vec<f32>,
}

impl DoubleMapper {
    pub fn new(mappings: &[(f32, f32)]) -> Self {
        let mut source_values = Vec::with_capacity(mappings.len());
        let mut target_values = Vec::with_capacity(mappings.len());

        for &(s, t) in mappings {
            source_values.push(s);
            target_values.push(t);
        }

        validate_progress(&source_values);
        validate_progress(&target_values);

        Self {
            source_values,
            target_values,
        }
    }

    pub fn identity() -> Self {
        Self::new(&[(0.0, 0.0), (0.5, 0.5)])
    }

    pub fn map(&self, x: f32) -> f32 {
        linear_map(&self.source_values, &self.target_values, x)
    }

    pub fn map_back(&self, x: f32) -> f32 {
        linear_map(&self.target_values, &self.source_values, x)
    }
}
