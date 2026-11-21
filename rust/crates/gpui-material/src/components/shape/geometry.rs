use gpui::Point;

pub fn interpolate(start: f32, stop: f32, fraction: f32) -> f32 {
    start + (stop - start) * fraction
}

pub fn interpolate_point(start: Point<f32>, stop: Point<f32>, fraction: f32) -> Point<f32> {
    Point {
        x: interpolate(start.x, stop.x, fraction),
        y: interpolate(start.y, stop.y, fraction),
    }
}

pub fn distance(dx: f32, dy: f32) -> f32 {
    (dx * dx + dy * dy).sqrt()
}

pub fn direction_vector(x: f32, y: f32) -> Point<f32> {
    let d = distance(x, y);
    if d > 0.0 {
        Point { x: x / d, y: y / d }
    } else {
        Point { x: 0.0, y: 0.0 }
    }
}

pub fn radial_to_cartesian(radius: f32, angle_radians: f32) -> Point<f32> {
    Point {
        x: radius * angle_radians.cos(),
        y: radius * angle_radians.sin(),
    }
}

pub trait PointExt {
    fn distance(&self) -> f32;
    fn distance_squared(&self) -> f32;
    fn dot_product(&self, other: Point<f32>) -> f32;
    fn rotate90(&self) -> Point<f32>;
    fn direction(&self) -> Point<f32>;
}

impl PointExt for Point<f32> {
    fn distance(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    fn distance_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }
    fn dot_product(&self, other: Point<f32>) -> f32 {
        self.x * other.x + self.y * other.y
    }
    fn rotate90(&self) -> Point<f32> {
        Point {
            x: -self.y,
            y: self.x,
        }
    }
    fn direction(&self) -> Point<f32> {
        direction_vector(self.x, self.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cubic {
    pub anchor0: Point<f32>,
    pub control0: Point<f32>,
    pub control1: Point<f32>,
    pub anchor1: Point<f32>,
}

impl Cubic {
    pub fn new(
        anchor0_x: f32,
        anchor0_y: f32,
        control0_x: f32,
        control0_y: f32,
        control1_x: f32,
        control1_y: f32,
        anchor1_x: f32,
        anchor1_y: f32,
    ) -> Self {
        Self {
            anchor0: Point {
                x: anchor0_x,
                y: anchor0_y,
            },
            control0: Point {
                x: control0_x,
                y: control0_y,
            },
            control1: Point {
                x: control1_x,
                y: control1_y,
            },
            anchor1: Point {
                x: anchor1_x,
                y: anchor1_y,
            },
        }
    }

    pub fn straight_line(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self::new(
            x0,
            y0,
            interpolate(x0, x1, 1.0 / 3.0),
            interpolate(y0, y1, 1.0 / 3.0),
            interpolate(x0, x1, 2.0 / 3.0),
            interpolate(y0, y1, 2.0 / 3.0),
            x1,
            y1,
        )
    }

    pub fn circular_arc(center_x: f32, center_y: f32, x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        let p0d = direction_vector(x0 - center_x, y0 - center_y);
        let p1d = direction_vector(x1 - center_x, y1 - center_y);
        let rotated_p0 = p0d.rotate90();
        let rotated_p1 = p1d.rotate90();
        let clockwise = rotated_p0.dot_product(Point {
            x: x1 - center_x,
            y: y1 - center_y,
        }) >= 0.0;
        let cosa = p0d.dot_product(p1d);
        if cosa > 0.999 {
            return Self::straight_line(x0, y0, x1, y1);
        }
        let k = distance(x0 - center_x, y0 - center_y)
            * (4.0 / 3.0)
            * ((2.0 * (1.0 - cosa)).sqrt() - (1.0 - cosa * cosa).sqrt())
            / (1.0 - cosa)
            * if clockwise { 1.0 } else { -1.0 };

        Self::new(
            x0,
            y0,
            x0 + rotated_p0.x * k,
            y0 + rotated_p0.y * k,
            x1 - rotated_p1.x * k,
            y1 - rotated_p1.y * k,
            x1,
            y1,
        )
    }

    pub fn reverse(&self) -> Self {
        Self {
            anchor0: self.anchor1,
            control0: self.control1,
            control1: self.control0,
            anchor1: self.anchor0,
        }
    }

    pub fn point_on_curve(&self, t: f32) -> Point<f32> {
        let u = 1.0 - t;
        Point {
            x: self.anchor0.x * (u * u * u)
                + self.control0.x * (3.0 * u * u * t)
                + self.control1.x * (3.0 * u * t * t)
                + self.anchor1.x * (t * t * t),
            y: self.anchor0.y * (u * u * u)
                + self.control0.y * (3.0 * u * u * t)
                + self.control1.y * (3.0 * u * t * t)
                + self.anchor1.y * (t * t * t),
        }
    }

    pub fn split(&self, t: f32) -> (Self, Self) {
        let u = 1.0 - t;
        let point_on_curve = self.point_on_curve(t);

        let c1 = Self::new(
            self.anchor0.x,
            self.anchor0.y,
            self.anchor0.x * u + self.control0.x * t,
            self.anchor0.y * u + self.control0.y * t,
            self.anchor0.x * (u * u) + self.control0.x * (2.0 * u * t) + self.control1.x * (t * t),
            self.anchor0.y * (u * u) + self.control0.y * (2.0 * u * t) + self.control1.y * (t * t),
            point_on_curve.x,
            point_on_curve.y,
        );

        let c2 = Self::new(
            point_on_curve.x,
            point_on_curve.y,
            self.control0.x * (u * u) + self.control1.x * (2.0 * u * t) + self.anchor1.x * (t * t),
            self.control0.y * (u * u) + self.control1.y * (2.0 * u * t) + self.anchor1.y * (t * t),
            self.control1.x * u + self.anchor1.x * t,
            self.control1.y * u + self.anchor1.y * t,
            self.anchor1.x,
            self.anchor1.y,
        );
        (c1, c2)
    }

    pub fn zero_length(&self) -> bool {
        let distance_epsilon = 1e-4;
        distance(
            self.anchor0.x - self.control0.x,
            self.anchor0.y - self.control0.y,
        ) < distance_epsilon
            && distance(
                self.control0.x - self.control1.x,
                self.control0.y - self.control1.y,
            ) < distance_epsilon
            && distance(
                self.control1.x - self.anchor1.x,
                self.control1.y - self.anchor1.y,
            ) < distance_epsilon
    }
}
impl Cubic {
    pub fn transformed<F>(&self, mut f: F) -> Self
    where
        F: FnMut(f32, f32) -> (f32, f32),
    {
        let (a0x, a0y) = f(self.anchor0.x, self.anchor0.y);
        let (c0x, c0y) = f(self.control0.x, self.control0.y);
        let (c1x, c1y) = f(self.control1.x, self.control1.y);
        let (a1x, a1y) = f(self.anchor1.x, self.anchor1.y);
        Self::new(a0x, a0y, c0x, c0y, c1x, c1y, a1x, a1y)
    }
}
