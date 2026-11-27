use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use usvg::tiny_skia_path;

use skia_safe::{Color, Matrix, Paint, PaintCap, PaintJoin, Path, PathBuilder, PathFillType};
use usvg::tiny_skia_path::PathSegment;

const TINT: (u8, u8, u8) = (0xfe, 0x00, 0xfe);

pub enum PartKind {
    Fill,
    Stroke {
        width: f32,
        cap: PaintCap,
        join: PaintJoin,
    },
}

pub struct SvgPart {
    pub path: Path,
    pub kind: PartKind,

    pub color: Option<Color>,

    pub opacity: f32,
}

pub struct ParsedSvg {
    pub width: f32,
    pub height: f32,
    pub parts: Vec<SvgPart>,
}

thread_local! {
    static CACHE: RefCell<HashMap<String, Option<Rc<ParsedSvg>>>> = RefCell::new(HashMap::new());
}

pub fn parse(svg: &str) -> Option<Rc<ParsedSvg>> {
    cached(svg, parse_uncached)
}

pub fn from_path_data(d: &str) -> Option<Rc<ParsedSvg>> {
    cached(d, |d| {
        let mut path = Path::from_svg(d)?;
        path.set_fill_type(PathFillType::EvenOdd);
        Some(ParsedSvg {
            width: 24.0,
            height: 24.0,
            parts: vec![SvgPart {
                path,
                kind: PartKind::Fill,
                color: None,
                opacity: 1.0,
            }],
        })
    })
}

fn cached(key: &str, make: impl FnOnce(&str) -> Option<ParsedSvg>) -> Option<Rc<ParsedSvg>> {
    CACHE.with(|cache| {
        if let Some(hit) = cache.borrow().get(key) {
            return hit.clone();
        }
        let parsed = make(key).map(Rc::new);
        cache.borrow_mut().insert(key.to_string(), parsed.clone());
        parsed
    })
}

impl ParsedSvg {
    pub fn draw(&self, canvas: &skia_safe::Canvas, tint: Color, alpha: f32) {
        for part in &self.parts {
            let mut paint = part.paint(tint);
            paint.set_alpha_f(paint.alpha_f() * alpha);
            canvas.draw_path(&part.path, &paint);
        }
    }

    pub fn draw_fitted(
        &self,
        canvas: &skia_safe::Canvas,
        rect: (f32, f32, f32, f32),
        tint: Color,
        alpha: f32,
    ) {
        let (x, y, w, h) = rect;
        let scale = (w / self.width).min(h / self.height);
        canvas.save();
        canvas.translate((
            x + (w - self.width * scale) / 2.0,
            y + (h - self.height * scale) / 2.0,
        ));
        canvas.scale((scale, scale));
        self.draw(canvas, tint, alpha);
        canvas.restore();
    }

    pub fn glyph_bounds(&self) -> skia_safe::Rect {
        let mut bounds: Option<skia_safe::Rect> = None;
        for part in &self.parts {
            let b = *part.path.bounds();
            bounds = Some(match bounds {
                Some(mut all) => {
                    all.join(b);
                    all
                }
                None => b,
            });
        }
        bounds.unwrap_or_else(|| skia_safe::Rect::from_wh(self.width, self.height))
    }
}

fn parse_uncached(svg: &str) -> Option<ParsedSvg> {
    let start = svg.find("<svg")? + "<svg".len();
    let (r, g, b) = TINT;
    let tinted = format!(
        "{} color=\"#{r:02x}{g:02x}{b:02x}\"{}",
        &svg[..start],
        &svg[start..]
    );
    let tree = match usvg::Tree::from_str(&tinted, &usvg::Options::default()) {
        Ok(tree) => tree,
        Err(err) => {
            log::warn!("icon svg did not parse: {err}");
            return None;
        }
    };
    let size = tree.size();
    let mut parts = Vec::new();
    collect(tree.root(), &mut parts);
    Some(ParsedSvg {
        width: size.width(),
        height: size.height(),
        parts,
    })
}

fn collect(group: &usvg::Group, parts: &mut Vec<SvgPart>) {
    for node in group.children() {
        match node {
            usvg::Node::Group(inner) => {
                if inner.clip_path().is_some() || inner.mask().is_some() {
                    log::debug!("icon svg: a clip path or mask is not drawn");
                }
                collect(inner, parts);
            }
            usvg::Node::Path(path) if path.is_visible() => {
                let skia = to_skia(path.data(), path.abs_transform());
                let fill_first = !matches!(path.paint_order(), usvg::PaintOrder::StrokeAndFill);
                let fill = path.fill().and_then(|fill| {
                    let (color, opacity) = paint(fill.paint(), fill.opacity().get())?;
                    let mut p = skia.clone();
                    p.set_fill_type(match fill.rule() {
                        usvg::FillRule::EvenOdd => PathFillType::EvenOdd,
                        usvg::FillRule::NonZero => PathFillType::Winding,
                    });
                    Some(SvgPart {
                        path: p,
                        kind: PartKind::Fill,
                        color,
                        opacity,
                    })
                });
                let stroke = path.stroke().and_then(|stroke| {
                    let (color, opacity) = paint(stroke.paint(), stroke.opacity().get())?;

                    let scale = path.abs_transform().get_scale().0;
                    Some(SvgPart {
                        path: skia.clone(),
                        kind: PartKind::Stroke {
                            width: stroke.width().get() * scale,
                            cap: match stroke.linecap() {
                                usvg::LineCap::Butt => PaintCap::Butt,
                                usvg::LineCap::Round => PaintCap::Round,
                                usvg::LineCap::Square => PaintCap::Square,
                            },
                            join: match stroke.linejoin() {
                                usvg::LineJoin::Round => PaintJoin::Round,
                                usvg::LineJoin::Bevel => PaintJoin::Bevel,
                                _ => PaintJoin::Miter,
                            },
                        },
                        color,
                        opacity,
                    })
                });
                if fill_first {
                    parts.extend(fill);
                    parts.extend(stroke);
                } else {
                    parts.extend(stroke);
                    parts.extend(fill);
                }
            }
            usvg::Node::Path(_) => {}
            _ => log::debug!("icon svg: an image or text node is not drawn"),
        }
    }
}

fn paint(paint: &usvg::Paint, opacity: f32) -> Option<(Option<Color>, f32)> {
    let usvg::Paint::Color(c) = paint else {
        return None;
    };
    if opacity <= 0.0 {
        return None;
    }
    Some(if (c.red, c.green, c.blue) == TINT {
        (None, opacity)
    } else {
        (Some(Color::from_rgb(c.red, c.green, c.blue)), opacity)
    })
}

fn to_skia(data: &tiny_skia_path::Path, transform: tiny_skia_path::Transform) -> Path {
    let mut builder = PathBuilder::new();
    for segment in data.segments() {
        match segment {
            PathSegment::MoveTo(p) => {
                builder.move_to((p.x, p.y));
            }
            PathSegment::LineTo(p) => {
                builder.line_to((p.x, p.y));
            }
            PathSegment::QuadTo(c, p) => {
                builder.quad_to((c.x, c.y), (p.x, p.y));
            }
            PathSegment::CubicTo(c1, c2, p) => {
                builder.cubic_to((c1.x, c1.y), (c2.x, c2.y), (p.x, p.y));
            }
            PathSegment::Close => {
                builder.close();
            }
        }
    }
    let path = builder.detach();
    let m = Matrix::new_all(
        transform.sx,
        transform.kx,
        transform.tx,
        transform.ky,
        transform.sy,
        transform.ty,
        0.0,
        0.0,
        1.0,
    );
    path.make_transform(&m)
}

impl SvgPart {
    pub fn paint(&self, tint: Color) -> Paint {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        let base = self.color.map(|c| c.with_a(255)).unwrap_or(tint);
        paint.set_color(base);
        paint.set_alpha_f(base.a() as f32 / 255.0 * self.opacity);
        match &self.kind {
            PartKind::Fill => {
                paint.set_style(skia_safe::PaintStyle::Fill);
            }
            PartKind::Stroke { width, cap, join } => {
                paint.set_style(skia_safe::PaintStyle::Stroke);
                paint.set_stroke_width(*width);
                paint.set_stroke_cap(*cap);
                paint.set_stroke_join(*join);
            }
        }
        paint
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tint_and_fixed_colors_are_told_apart() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="transparent"/><path fill="currentColor" fill-rule="evenodd" d="M12 23a11 11 0 1 0 0-22 11 11 0 0 0 0 22Z"/><rect x="1" y="1" width="4" height="4" fill="#ff0000"/></svg>"##;
        let parsed = parse(svg).expect("parses");
        assert_eq!((parsed.width, parsed.height), (24.0, 24.0));

        assert_eq!(parsed.parts.len(), 2);
        assert!(parsed.parts[0].color.is_none());
        assert_eq!(parsed.parts[0].path.fill_type(), PathFillType::EvenOdd);
        assert_eq!(parsed.parts[1].color, Some(Color::from_rgb(255, 0, 0)));
    }

    #[test]
    fn non_square_view_boxes_keep_their_size() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 20"><path fill="currentColor" d="M0 0h16v20H0z"/></svg>"#;
        let parsed = parse(svg).expect("parses");
        assert_eq!((parsed.width, parsed.height), (16.0, 20.0));
    }
}
