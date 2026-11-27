#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub scale_x: f32,
    pub scale_y: f32,
    pub translate_x: f32,
    pub translate_y: f32,

    pub rotate_deg: f32,
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        scale_x: 1.0,
        scale_y: 1.0,
        translate_x: 0.0,
        translate_y: 0.0,
        rotate_deg: 0.0,
    };

    pub fn lerp(a: Transform, b: Transform, t: f32) -> Transform {
        let l = |x: f32, y: f32| x + (y - x) * t;
        Transform {
            scale_x: l(a.scale_x, b.scale_x),
            scale_y: l(a.scale_y, b.scale_y),
            translate_x: l(a.translate_x, b.translate_x),
            translate_y: l(a.translate_y, b.translate_y),
            rotate_deg: l(a.rotate_deg, b.rotate_deg),
        }
    }

    pub fn is_identity(&self) -> bool {
        *self == Self::IDENTITY
    }
}

pub fn parse(value: &str) -> Option<Transform> {
    let value = value.trim();
    if value == "none" {
        return Some(Transform::IDENTITY);
    }
    let num = |s: &str| s.parse::<f32>().ok();
    let px = |s: &str| {
        if s == "0" {
            Some(0.0)
        } else {
            s.strip_suffix("px").and_then(|v| v.parse::<f32>().ok())
        }
    };
    let deg = |s: &str| {
        s.strip_suffix("deg")
            .and_then(|v| v.trim().parse::<f32>().ok())
    };
    // Without perspective a rotation about X or Y only projects to a scale of
    // cos(angle) on the other axis, so rotateX(180deg) is an exact flip.
    let projected = |angle: f32| {
        let cos = angle.to_radians().cos();
        if (cos - cos.round()).abs() < 1e-6 {
            cos.round()
        } else {
            cos
        }
    };
    // Functions compose left to right like CSS: a translate after a scale or
    // rotate moves along the already scaled and rotated axes.
    let mut t = Transform::IDENTITY;
    let translate = |t: &mut Transform, x: f32, y: f32| {
        let (x, y) = (x * t.scale_x, y * t.scale_y);
        let (sin, cos) = t.rotate_deg.to_radians().sin_cos();
        t.translate_x += x * cos - y * sin;
        t.translate_y += x * sin + y * cos;
    };
    let mut rest = value;
    while !rest.is_empty() {
        let open = rest.find('(')?;
        let close = rest.find(')')?;
        let name = rest[..open].trim();
        let args: Vec<&str> = rest[open + 1..close].split(',').map(str::trim).collect();
        match name {
            "scale" => {
                let x = num(args[0])?;
                let y = match args.get(1) {
                    Some(a) => num(a)?,
                    None => x,
                };
                t.scale_x *= x;
                t.scale_y *= y;
            }
            "scaleX" => t.scale_x *= num(args[0])?,
            "scaleY" => t.scale_y *= num(args[0])?,
            "translate" | "translate3d" => {
                let x = px(args[0])?;
                let y = match args.get(1) {
                    Some(a) => px(a)?,
                    None => 0.0,
                };
                translate(&mut t, x, y);
            }
            "translateX" => translate(&mut t, px(args[0])?, 0.0),
            "translateY" => translate(&mut t, 0.0, px(args[0])?),
            "translateZ" => {}
            "rotate" => t.rotate_deg += deg(args[0])?,
            "rotateX" => t.scale_y *= projected(deg(args[0])?),
            "rotateY" => t.scale_x *= projected(deg(args[0])?),
            _ => return None,
        }
        rest = rest[close + 1..].trim_start();
    }
    Some(t)
}

pub fn parse_origin(value: &str) -> Option<(f32, f32)> {
    fn frac(s: &str) -> Option<(f32, Option<bool>)> {
        Some(match s {
            "left" => (0.0, Some(false)),
            "right" => (1.0, Some(false)),
            "top" => (0.0, Some(true)),
            "bottom" => (1.0, Some(true)),
            "center" => (0.5, None),
            _ => (s.strip_suffix('%')?.parse::<f32>().ok()? / 100.0, None),
        })
    }
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.as_slice() {
        [one] => {
            let (f, vertical) = frac(one)?;
            Some(if vertical == Some(true) {
                (0.5, f)
            } else {
                (f, 0.5)
            })
        }
        [a, b, ..] => {
            let (fa, va) = frac(a)?;
            let (fb, vb) = frac(b)?;

            Some(if va == Some(true) || vb == Some(false) {
                (fb, fa)
            } else {
                (fa, fb)
            })
        }
        [] => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mana_transforms() {
        assert_eq!(
            parse("scale(0.95)"),
            Some(Transform {
                scale_x: 0.95,
                scale_y: 0.95,
                ..Transform::IDENTITY
            })
        );
        assert_eq!(parse("translate3d(0px, -107%, 0px)"), None);
        assert_eq!(
            parse("translateY(2px) translateZ(0px)"),
            Some(Transform {
                translate_y: 2.0,
                ..Transform::IDENTITY
            })
        );
        assert_eq!(parse("none"), Some(Transform::IDENTITY));
        assert_eq!(
            parse("translate3d(-44px, -6px, 0) rotate(-30deg)"),
            Some(Transform {
                translate_x: -44.0,
                translate_y: -6.0,
                rotate_deg: -30.0,
                ..Transform::IDENTITY
            })
        );
        // Discord's inverted thread spine; Chrome computes
        // matrix3d(1,0,0,0, 0,-1,0,0, 0,0,-1,0, 0,9,0,1) for it.
        assert_eq!(
            parse("rotateX(180deg) translateY(-9px)"),
            Some(Transform {
                scale_y: -1.0,
                translate_y: 9.0,
                ..Transform::IDENTITY
            })
        );
        assert_eq!(
            parse("scale(2) translateX(3px)"),
            Some(Transform {
                scale_x: 2.0,
                scale_y: 2.0,
                translate_x: 6.0,
                ..Transform::IDENTITY
            })
        );
        assert_eq!(parse_origin("50% 100%"), Some((0.5, 1.0)));
        assert_eq!(parse_origin("bottom"), Some((0.5, 1.0)));
        assert_eq!(parse_origin("top left"), Some((0.0, 0.0)));
    }
}
