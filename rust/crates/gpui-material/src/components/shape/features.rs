use super::geometry::Cubic;

#[derive(Clone, Debug)]
pub enum Feature {
    Edge { cubics: Vec<Cubic> },
    Corner { cubics: Vec<Cubic>, convex: bool },
}

impl Feature {
    pub fn cubics(&self) -> &Vec<Cubic> {
        match self {
            Feature::Edge { cubics } => cubics,
            Feature::Corner { cubics, .. } => cubics,
        }
    }

    pub fn is_convex(&self) -> bool {
        match self {
            Feature::Edge { .. } => false,
            Feature::Corner { convex, .. } => *convex,
        }
    }

    pub fn is_concave(&self) -> bool {
        match self {
            Feature::Edge { .. } => false,
            Feature::Corner { convex, .. } => !*convex,
        }
    }

    pub fn is_edge(&self) -> bool {
        matches!(self, Feature::Edge { .. })
    }
}
impl Feature {
    pub fn transformed<F>(&self, f: F) -> Self
    where
        F: FnMut(f32, f32) -> (f32, f32) + Clone,
    {
        match self {
            Feature::Corner { convex, cubics } => Feature::Corner {
                convex: *convex,
                cubics: cubics.iter().map(|c| c.transformed(f.clone())).collect(),
            },
            Feature::Edge { cubics } => Feature::Edge {
                cubics: cubics.iter().map(|c| c.transformed(f.clone())).collect(),
            },
        }
    }
}
