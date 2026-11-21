use super::polygon::{CornerRounding, RoundedPolygon};
use std::f32::consts::PI;
pub struct MaterialShapes;

impl MaterialShapes {
    pub fn rotated(shape: RoundedPolygon, degrees: f32) -> RoundedPolygon {
        let radians = degrees * PI / 180.0;
        let cos_a = radians.cos();
        let sin_a = radians.sin();
        shape.transformed(|x, y| (x * cos_a - y * sin_a, x * sin_a + y * cos_a))
    }

    pub fn point_n_round(
        x: f32,
        y: f32,
        rounding: Option<CornerRounding>,
    ) -> (f32, f32, Option<CornerRounding>) {
        (x, y, rounding)
    }

    pub fn do_repeat(
        points: &[(f32, f32, Option<CornerRounding>)],
        reps: usize,
        center_x: f32,
        center_y: f32,
        mirroring: bool,
    ) -> Vec<(f32, f32, Option<CornerRounding>)> {
        let mut result = Vec::new();
        if mirroring {
            let mut angles = Vec::new();
            let mut distances = Vec::new();
            for p in points {
                let dx = p.0 - center_x;
                let dy = p.1 - center_y;
                angles.push(dy.atan2(dx) * 180.0 / PI);
                distances.push((dx * dx + dy * dy).sqrt());
            }
            let actual_reps = reps * 2;
            let section_angle = 360.0 / actual_reps as f32;
            for rep in 0..actual_reps {
                for index in 0..points.len() {
                    let i = if rep % 2 == 0 {
                        index
                    } else {
                        points.len() - 1 - index
                    };
                    if i > 0 || rep % 2 == 0 {
                        let a = if rep % 2 == 0 {
                            section_angle * (rep as f32) + angles[i]
                        } else {
                            section_angle * (rep as f32) + section_angle - angles[i]
                                + 2.0 * angles[0]
                        };
                        let rad = a * PI / 180.0;
                        let final_x = rad.cos() * distances[i] + center_x;
                        let final_y = rad.sin() * distances[i] + center_y;
                        result.push((final_x, final_y, points[i].2.clone()));
                    }
                }
            }
        } else {
            for rep in 0..reps {
                for i in 0..points.len() {
                    let angle = 360.0 / (reps as f32) * (rep as f32);
                    let rad = angle * PI / 180.0;
                    let dx = points[i].0 - center_x;
                    let dy = points[i].1 - center_y;
                    let new_x = dx * rad.cos() - dy * rad.sin() + center_x;
                    let new_y = dx * rad.sin() + dy * rad.cos() + center_y;
                    result.push((new_x, new_y, points[i].2.clone()));
                }
            }
        }
        result
    }

    pub fn custom_polygon(
        pnr: &[(f32, f32, Option<CornerRounding>)],
        reps: usize,
        center_x: f32,
        center_y: f32,
        mirroring: bool,
    ) -> RoundedPolygon {
        let actual = Self::do_repeat(pnr, reps, center_x, center_y, mirroring);
        let mut vertices = Vec::new();
        let mut roundings = Vec::new();
        for p in &actual {
            vertices.push(p.0);
            vertices.push(p.1);
            roundings.push(p.2.unwrap_or(CornerRounding {
                radius: 0.0,
                smoothing: 0.0,
            }));
        }
        RoundedPolygon::from_vertices(
            &vertices,
            CornerRounding {
                radius: 0.0,
                smoothing: 0.0,
            },
            Some(roundings),
            center_x,
            center_y,
        )
    }

    pub fn rectangle(
        width: f32,
        height: f32,
        rounding: CornerRounding,
        center_x: f32,
        center_y: f32,
    ) -> RoundedPolygon {
        let half_w = width / 2.0;
        let half_h = height / 2.0;
        let vertices = vec![
            center_x + half_w,
            center_y + half_h,
            center_x - half_w,
            center_y + half_h,
            center_x - half_w,
            center_y - half_h,
            center_x + half_w,
            center_y - half_h,
        ];
        RoundedPolygon::from_vertices(&vertices, rounding, None, center_x, center_y)
    }

    pub fn circle_raw(
        num_vertices: usize,
        radius: f32,
        center_x: f32,
        center_y: f32,
    ) -> RoundedPolygon {
        let theta = PI / (num_vertices as f32);
        let poly_radius = radius / theta.cos();
        RoundedPolygon::circle(num_vertices, poly_radius, center_x, center_y)
    }

    pub fn star_raw(
        num_vertices_per_radius: usize,
        radius: f32,
        inner_radius: f32,
        rounding: CornerRounding,
        inner_rounding: Option<CornerRounding>,
        center_x: f32,
        center_y: f32,
    ) -> RoundedPolygon {
        RoundedPolygon::star(
            num_vertices_per_radius,
            radius,
            inner_radius,
            rounding,
            inner_rounding,
            center_x,
            center_y,
        )
    }

    pub fn polygon(
        num_vertices: usize,
        radius: f32,
        rounding: CornerRounding,
        pvr: Option<Vec<CornerRounding>>,
        center_x: f32,
        center_y: f32,
    ) -> RoundedPolygon {
        RoundedPolygon::from_num_verts(num_vertices, radius, center_x, center_y, rounding, pvr)
    }

    pub fn rectangle_pvr(
        width: f32,
        height: f32,
        pvr: Option<Vec<CornerRounding>>,
        center_x: f32,
        center_y: f32,
    ) -> RoundedPolygon {
        let half_w = width / 2.0;
        let half_h = height / 2.0;
        let vertices = vec![
            center_x + half_w,
            center_y + half_h,
            center_x - half_w,
            center_y + half_h,
            center_x - half_w,
            center_y - half_h,
            center_x + half_w,
            center_y - half_h,
        ];
        RoundedPolygon::from_vertices(
            &vertices,
            CornerRounding {
                radius: 0.0,
                smoothing: 0.0,
            },
            pvr,
            center_x,
            center_y,
        )
    }

    pub fn circle() -> RoundedPolygon {
        Self::circle_raw(10, 1.0, 0.0, 0.0).normalized()
    }
    pub fn square() -> RoundedPolygon {
        Self::rectangle(
            1.0,
            1.0,
            CornerRounding {
                radius: 0.30,
                smoothing: 0.0,
            },
            0.0,
            0.0,
        )
        .normalized()
    }
    pub fn slanted() -> RoundedPolygon {
        Self::custom_polygon(
            &[
                Self::point_n_round(
                    0.926,
                    0.970,
                    Some(CornerRounding {
                        radius: 0.189,
                        smoothing: 0.811,
                    }),
                ),
                Self::point_n_round(
                    -0.021,
                    0.967,
                    Some(CornerRounding {
                        radius: 0.187,
                        smoothing: 0.057,
                    }),
                ),
            ],
            2,
            0.5,
            0.5,
            false,
        )
        .normalized()
    }
    pub fn arch() -> RoundedPolygon {
        let pvr = vec![
            CornerRounding {
                radius: 1.0,
                smoothing: 0.0,
            },
            CornerRounding {
                radius: 1.0,
                smoothing: 0.0,
            },
            CornerRounding {
                radius: 0.2,
                smoothing: 0.0,
            },
            CornerRounding {
                radius: 0.2,
                smoothing: 0.0,
            },
        ];
        let p = RoundedPolygon::from_vertices(
            &[1.0, 0.0, 0.0, 1.0, -1.0, 0.0, 0.0, -1.0],
            CornerRounding {
                radius: 0.0,
                smoothing: 0.0,
            },
            Some(pvr),
            0.0,
            0.0,
        );
        Self::rotated(p, -135.0).normalized()
    }
    pub fn fan() -> RoundedPolygon {
        let pvr = vec![
            CornerRounding {
                radius: 0.2,
                smoothing: 0.0,
            },
            CornerRounding {
                radius: 0.2,
                smoothing: 0.0,
            },
            CornerRounding {
                radius: 1.0,
                smoothing: 0.0,
            },
            CornerRounding {
                radius: 0.2,
                smoothing: 0.0,
            },
        ];
        let p = RoundedPolygon::from_vertices(
            &[1.0, 0.0, 0.0, 1.0, -1.0, 0.0, 0.0, -1.0],
            CornerRounding {
                radius: 0.0,
                smoothing: 0.0,
            },
            Some(pvr),
            0.0,
            0.0,
        );
        Self::rotated(p, -45.0).normalized()
    }
    pub fn arrow() -> RoundedPolygon {
        Self::custom_polygon(
            &[
                Self::point_n_round(
                    0.500,
                    0.892,
                    Some(CornerRounding {
                        radius: 0.313,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    -0.216,
                    1.050,
                    Some(CornerRounding {
                        radius: 0.207,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    0.499,
                    -0.160,
                    Some(CornerRounding {
                        radius: 0.215,
                        smoothing: 1.000,
                    }),
                ),
                Self::point_n_round(
                    1.225,
                    1.060,
                    Some(CornerRounding {
                        radius: 0.211,
                        smoothing: 0.0,
                    }),
                ),
            ],
            1,
            0.5,
            0.5,
            false,
        )
        .normalized()
    }
    pub fn semi_circle() -> RoundedPolygon {
        let pvr = vec![
            CornerRounding {
                radius: 0.2,
                smoothing: 0.0,
            },
            CornerRounding {
                radius: 0.2,
                smoothing: 0.0,
            },
            CornerRounding {
                radius: 1.0,
                smoothing: 0.0,
            },
            CornerRounding {
                radius: 1.0,
                smoothing: 0.0,
            },
        ];
        Self::rectangle_pvr(1.6, 1.0, Some(pvr), 0.0, 0.0).normalized()
    }
    pub fn oval() -> RoundedPolygon {
        Self::rectangle(
            1.0,
            0.7,
            CornerRounding {
                radius: 0.35,
                smoothing: 0.0,
            },
            0.0,
            0.0,
        )
        .normalized()
    }
    pub fn pill() -> RoundedPolygon {
        Self::rectangle(
            1.0,
            0.5,
            CornerRounding {
                radius: 0.25,
                smoothing: 0.0,
            },
            0.0,
            0.0,
        )
        .normalized()
    }
    pub fn triangle() -> RoundedPolygon {
        Self::rotated(
            Self::polygon(
                3,
                1.0,
                CornerRounding {
                    radius: 0.20,
                    smoothing: 0.0,
                },
                None,
                0.0,
                0.0,
            ),
            -90.0,
        )
        .normalized()
    }
    pub fn diamond() -> RoundedPolygon {
        Self::rotated(
            Self::polygon(
                4,
                1.0,
                CornerRounding {
                    radius: 0.30,
                    smoothing: 0.0,
                },
                None,
                0.0,
                0.0,
            ),
            -90.0,
        )
        .normalized()
    }
    pub fn clam_shell() -> RoundedPolygon {
        Self::custom_polygon(
            &[
                Self::point_n_round(
                    0.171,
                    0.841,
                    Some(CornerRounding {
                        radius: 0.159,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    -0.020,
                    0.500,
                    Some(CornerRounding {
                        radius: 0.140,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    0.170,
                    0.159,
                    Some(CornerRounding {
                        radius: 0.159,
                        smoothing: 0.0,
                    }),
                ),
            ],
            2,
            0.5,
            0.5,
            false,
        )
        .normalized()
    }
    pub fn pentagon() -> RoundedPolygon {
        Self::custom_polygon(
            &[
                Self::point_n_round(
                    0.500,
                    -0.009,
                    Some(CornerRounding {
                        radius: 0.172,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    1.030,
                    0.365,
                    Some(CornerRounding {
                        radius: 0.164,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    0.828,
                    0.970,
                    Some(CornerRounding {
                        radius: 0.169,
                        smoothing: 0.0,
                    }),
                ),
            ],
            1,
            0.5,
            0.5,
            true,
        )
        .normalized()
    }
    pub fn gem() -> RoundedPolygon {
        Self::custom_polygon(
            &[
                Self::point_n_round(
                    0.499,
                    1.023,
                    Some(CornerRounding {
                        radius: 0.241,
                        smoothing: 0.778,
                    }),
                ),
                Self::point_n_round(
                    -0.005,
                    0.792,
                    Some(CornerRounding {
                        radius: 0.208,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    0.073,
                    0.258,
                    Some(CornerRounding {
                        radius: 0.228,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    0.433,
                    -0.000,
                    Some(CornerRounding {
                        radius: 0.491,
                        smoothing: 0.0,
                    }),
                ),
            ],
            1,
            0.5,
            0.5,
            true,
        )
        .normalized()
    }
    pub fn sunny() -> RoundedPolygon {
        Self::star_raw(
            8,
            1.0,
            0.8,
            CornerRounding {
                radius: 0.15,
                smoothing: 0.0,
            },
            None,
            0.0,
            0.0,
        )
        .normalized()
    }
    pub fn very_sunny() -> RoundedPolygon {
        Self::custom_polygon(
            &[
                Self::point_n_round(
                    0.500,
                    1.080,
                    Some(CornerRounding {
                        radius: 0.085,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    0.358,
                    0.843,
                    Some(CornerRounding {
                        radius: 0.085,
                        smoothing: 0.0,
                    }),
                ),
            ],
            8,
            0.5,
            0.5,
            false,
        )
        .normalized()
    }
    pub fn cookie4sided() -> RoundedPolygon {
        Self::rotated(
            Self::star_raw(
                4,
                1.0,
                0.5,
                CornerRounding {
                    radius: 0.50,
                    smoothing: 0.50,
                },
                None,
                0.0,
                0.0,
            ),
            -45.0,
        )
        .normalized()
    }
    pub fn cookie6sided() -> RoundedPolygon {
        Self::rotated(
            Self::star_raw(
                6,
                1.0,
                0.75,
                CornerRounding {
                    radius: 0.50,
                    smoothing: 0.0,
                },
                None,
                0.0,
                0.0,
            ),
            -90.0,
        )
        .normalized()
    }
    pub fn cookie7sided() -> RoundedPolygon {
        Self::rotated(
            Self::star_raw(
                7,
                1.0,
                0.75,
                CornerRounding {
                    radius: 0.50,
                    smoothing: 0.0,
                },
                None,
                0.0,
                0.0,
            ),
            -90.0,
        )
        .normalized()
    }
    pub fn cookie9sided() -> RoundedPolygon {
        Self::rotated(
            Self::star_raw(
                9,
                1.0,
                0.8,
                CornerRounding {
                    radius: 0.50,
                    smoothing: 0.0,
                },
                None,
                0.0,
                0.0,
            ),
            -90.0,
        )
        .normalized()
    }
    pub fn cookie12sided() -> RoundedPolygon {
        Self::rotated(
            Self::star_raw(
                12,
                1.0,
                0.8,
                CornerRounding {
                    radius: 0.50,
                    smoothing: 0.0,
                },
                None,
                0.0,
                0.0,
            ),
            -90.0,
        )
        .normalized()
    }
    pub fn ghostish() -> RoundedPolygon {
        Self::custom_polygon(
            &[
                Self::point_n_round(
                    0.500,
                    0.0,
                    Some(CornerRounding {
                        radius: 1.000,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    1.0,
                    0.0,
                    Some(CornerRounding {
                        radius: 1.000,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    1.0,
                    1.140,
                    Some(CornerRounding {
                        radius: 0.254,
                        smoothing: 0.106,
                    }),
                ),
                Self::point_n_round(
                    0.575,
                    0.906,
                    Some(CornerRounding {
                        radius: 0.253,
                        smoothing: 0.0,
                    }),
                ),
            ],
            1,
            0.5,
            0.5,
            true,
        )
        .normalized()
    }
    pub fn clover4leaf() -> RoundedPolygon {
        Self::custom_polygon(
            &[
                Self::point_n_round(0.500, 0.074, None),
                Self::point_n_round(
                    0.725,
                    -0.099,
                    Some(CornerRounding {
                        radius: 0.476,
                        smoothing: 0.0,
                    }),
                ),
            ],
            4,
            0.5,
            0.5,
            true,
        )
        .normalized()
    }
    pub fn clover8leaf() -> RoundedPolygon {
        Self::custom_polygon(
            &[
                Self::point_n_round(0.500, 0.036, None),
                Self::point_n_round(
                    0.758,
                    -0.101,
                    Some(CornerRounding {
                        radius: 0.209,
                        smoothing: 0.0,
                    }),
                ),
            ],
            8,
            0.5,
            0.5,
            false,
        )
        .normalized()
    }
    pub fn burst() -> RoundedPolygon {
        Self::custom_polygon(
            &[
                Self::point_n_round(
                    0.500,
                    -0.006,
                    Some(CornerRounding {
                        radius: 0.006,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    0.592,
                    0.158,
                    Some(CornerRounding {
                        radius: 0.006,
                        smoothing: 0.0,
                    }),
                ),
            ],
            12,
            0.5,
            0.5,
            false,
        )
        .normalized()
    }
    pub fn soft_burst() -> RoundedPolygon {
        Self::custom_polygon(
            &[
                Self::point_n_round(
                    0.193,
                    0.277,
                    Some(CornerRounding {
                        radius: 0.053,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    0.176,
                    0.055,
                    Some(CornerRounding {
                        radius: 0.053,
                        smoothing: 0.0,
                    }),
                ),
            ],
            10,
            0.5,
            0.5,
            false,
        )
        .normalized()
    }
    pub fn boom() -> RoundedPolygon {
        Self::custom_polygon(
            &[
                Self::point_n_round(
                    0.457,
                    0.296,
                    Some(CornerRounding {
                        radius: 0.007,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    0.500,
                    -0.051,
                    Some(CornerRounding {
                        radius: 0.007,
                        smoothing: 0.0,
                    }),
                ),
            ],
            15,
            0.5,
            0.5,
            false,
        )
        .normalized()
    }
    pub fn soft_boom() -> RoundedPolygon {
        Self::custom_polygon(
            &[
                Self::point_n_round(0.733, 0.454, None),
                Self::point_n_round(
                    0.839,
                    0.437,
                    Some(CornerRounding {
                        radius: 0.532,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    0.949,
                    0.449,
                    Some(CornerRounding {
                        radius: 0.439,
                        smoothing: 1.000,
                    }),
                ),
                Self::point_n_round(
                    0.998,
                    0.478,
                    Some(CornerRounding {
                        radius: 0.174,
                        smoothing: 0.0,
                    }),
                ),
            ],
            16,
            0.5,
            0.5,
            true,
        )
        .normalized()
    }
    pub fn flower() -> RoundedPolygon {
        Self::custom_polygon(
            &[
                Self::point_n_round(0.370, 0.187, None),
                Self::point_n_round(
                    0.416,
                    0.049,
                    Some(CornerRounding {
                        radius: 0.381,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    0.479,
                    0.001,
                    Some(CornerRounding {
                        radius: 0.095,
                        smoothing: 0.0,
                    }),
                ),
            ],
            8,
            0.5,
            0.5,
            true,
        )
        .normalized()
    }
    pub fn puffy() -> RoundedPolygon {
        let shape = Self::custom_polygon(
            &[
                Self::point_n_round(0.500, 0.053, None),
                Self::point_n_round(
                    0.545,
                    -0.040,
                    Some(CornerRounding {
                        radius: 0.405,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    0.670,
                    -0.035,
                    Some(CornerRounding {
                        radius: 0.426,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    0.717,
                    0.066,
                    Some(CornerRounding {
                        radius: 0.574,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(0.722, 0.128, None),
                Self::point_n_round(
                    0.777,
                    0.002,
                    Some(CornerRounding {
                        radius: 0.360,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    0.914,
                    0.149,
                    Some(CornerRounding {
                        radius: 0.660,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    0.926,
                    0.289,
                    Some(CornerRounding {
                        radius: 0.660,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(0.881, 0.346, None),
                Self::point_n_round(
                    0.940,
                    0.344,
                    Some(CornerRounding {
                        radius: 0.126,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    1.003,
                    0.437,
                    Some(CornerRounding {
                        radius: 0.255,
                        smoothing: 0.0,
                    }),
                ),
            ],
            2,
            0.5,
            0.5,
            true,
        );
        shape.transformed(|x, y| (x, y * 0.742)).normalized()
    }
    pub fn puffy_diamond() -> RoundedPolygon {
        Self::custom_polygon(
            &[
                Self::point_n_round(
                    0.870,
                    0.130,
                    Some(CornerRounding {
                        radius: 0.146,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(0.818, 0.357, None),
                Self::point_n_round(
                    1.000,
                    0.332,
                    Some(CornerRounding {
                        radius: 0.853,
                        smoothing: 0.0,
                    }),
                ),
            ],
            4,
            0.5,
            0.5,
            true,
        )
        .normalized()
    }
    pub fn pixel_circle() -> RoundedPolygon {
        Self::custom_polygon(
            &[
                Self::point_n_round(0.500, 0.000, None),
                Self::point_n_round(0.704, 0.000, None),
                Self::point_n_round(0.704, 0.065, None),
                Self::point_n_round(0.843, 0.065, None),
                Self::point_n_round(0.843, 0.148, None),
                Self::point_n_round(0.926, 0.148, None),
                Self::point_n_round(0.926, 0.296, None),
                Self::point_n_round(1.000, 0.296, None),
            ],
            2,
            0.5,
            0.5,
            true,
        )
        .normalized()
    }
    pub fn pixel_triangle() -> RoundedPolygon {
        Self::custom_polygon(
            &[
                Self::point_n_round(0.110, 0.500, None),
                Self::point_n_round(0.113, 0.000, None),
                Self::point_n_round(0.287, 0.000, None),
                Self::point_n_round(0.287, 0.087, None),
                Self::point_n_round(0.421, 0.087, None),
                Self::point_n_round(0.421, 0.170, None),
                Self::point_n_round(0.560, 0.170, None),
                Self::point_n_round(0.560, 0.265, None),
                Self::point_n_round(0.674, 0.265, None),
                Self::point_n_round(0.675, 0.344, None),
                Self::point_n_round(0.789, 0.344, None),
                Self::point_n_round(0.789, 0.439, None),
                Self::point_n_round(0.888, 0.439, None),
            ],
            1,
            0.5,
            0.5,
            true,
        )
        .normalized()
    }
    pub fn bun() -> RoundedPolygon {
        Self::custom_polygon(
            &[
                Self::point_n_round(0.796, 0.500, None),
                Self::point_n_round(
                    0.853,
                    0.518,
                    Some(CornerRounding {
                        radius: 1.0,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    0.992,
                    0.631,
                    Some(CornerRounding {
                        radius: 1.0,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    0.968,
                    1.000,
                    Some(CornerRounding {
                        radius: 1.0,
                        smoothing: 0.0,
                    }),
                ),
            ],
            2,
            0.5,
            0.5,
            true,
        )
        .normalized()
    }
    pub fn heart() -> RoundedPolygon {
        Self::custom_polygon(
            &[
                Self::point_n_round(
                    0.500,
                    0.268,
                    Some(CornerRounding {
                        radius: 0.016,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    0.792,
                    -0.066,
                    Some(CornerRounding {
                        radius: 0.958,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    1.064,
                    0.276,
                    Some(CornerRounding {
                        radius: 1.000,
                        smoothing: 0.0,
                    }),
                ),
                Self::point_n_round(
                    0.501,
                    0.946,
                    Some(CornerRounding {
                        radius: 0.129,
                        smoothing: 0.0,
                    }),
                ),
            ],
            1,
            0.5,
            0.5,
            true,
        )
        .normalized()
    }
}
