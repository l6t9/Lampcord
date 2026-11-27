use std::time::{Duration, Instant};

use skia_safe::Color;

use super::ComputedStyle;
use super::transform::Transform;
use crate::widget::anim::lerp_color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionProperty {
    All,
    Opacity,
    BackgroundColor,
    BorderColor,
    Color,
    Transform,
}

impl TransitionProperty {
    fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "all" => Self::All,
            "opacity" => Self::Opacity,
            "background-color" | "background" => Self::BackgroundColor,
            "border-color" | "border" => Self::BorderColor,
            "color" | "fill" => Self::Color,
            "transform" => Self::Transform,
            _ => return None,
        })
    }

    fn covers(self, other: TransitionProperty) -> bool {
        self == TransitionProperty::All || self == other
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    CubicBezier(f32, f32, f32, f32),
}

impl Easing {
    pub const EASE: Easing = Easing::CubicBezier(0.25, 0.1, 0.25, 1.0);

    pub(super) fn parse(token: &str) -> Option<Self> {
        Some(match token {
            "linear" => Easing::Linear,
            "ease" => Easing::EASE,
            "ease-in" => Easing::CubicBezier(0.42, 0.0, 1.0, 1.0),
            "ease-out" => Easing::CubicBezier(0.0, 0.0, 0.58, 1.0),
            "ease-in-out" => Easing::CubicBezier(0.42, 0.0, 0.58, 1.0),
            _ => {
                let inner = token.strip_prefix("cubic-bezier(")?.strip_suffix(')')?;
                let n: Vec<f32> = inner
                    .split(',')
                    .filter_map(|v| v.trim().parse().ok())
                    .collect();
                if n.len() != 4 {
                    return None;
                }
                Easing::CubicBezier(n[0], n[1], n[2], n[3])
            }
        })
    }

    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::CubicBezier(x1, y1, x2, y2) => {
                let bez = |a: f32, b: f32, s: f32| {
                    let inv = 1.0 - s;
                    3.0 * inv * inv * s * a + 3.0 * inv * s * s * b + s * s * s
                };
                let dbez = |a: f32, b: f32, s: f32| {
                    let inv = 1.0 - s;
                    3.0 * inv * inv * a + 6.0 * inv * s * (b - a) + 3.0 * s * s * (1.0 - b)
                };
                let mut s = t;
                for _ in 0..8 {
                    let err = bez(x1, x2, s) - t;
                    if err.abs() < 1e-5 {
                        return bez(y1, y2, s);
                    }
                    let d = dbez(x1, x2, s);
                    if d.abs() < 1e-6 {
                        break;
                    }
                    s = (s - err / d).clamp(0.0, 1.0);
                }
                let (mut lo, mut hi) = (0.0_f32, 1.0_f32);
                for _ in 0..30 {
                    s = (lo + hi) / 2.0;
                    if bez(x1, x2, s) < t {
                        lo = s;
                    } else {
                        hi = s;
                    }
                }
                bez(y1, y2, s)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransitionSpec {
    pub property: TransitionProperty,
    pub duration: Duration,
    pub delay: Duration,
    pub easing: Easing,
}

pub fn parse(value: &str) -> Vec<TransitionSpec> {
    let value = value.trim();
    if value == "none" {
        return Vec::new();
    }
    split_top_level(value)
        .into_iter()
        .filter_map(|item| {
            let mut property = None;
            let mut times = Vec::new();
            let mut easing = Easing::EASE;
            for token in split_tokens(item) {
                if let Some(time) = parse_time(token) {
                    times.push(time);
                } else if let Some(e) = Easing::parse(token) {
                    easing = e;
                } else if property.is_none() {
                    property = Some(TransitionProperty::parse(token));
                }
            }
            let property = property.unwrap_or(Some(TransitionProperty::All))?;
            Some(TransitionSpec {
                property,
                duration: times.first().copied().unwrap_or(Duration::ZERO),
                delay: times.get(1).copied().unwrap_or(Duration::ZERO),
                easing,
            })
        })
        .collect()
}

pub(super) fn parse_time(token: &str) -> Option<Duration> {
    let (number, micros_per_unit) = match token.strip_suffix("ms") {
        Some(ms) => (ms, 1_000.0),
        None => (token.strip_suffix('s')?, 1_000_000.0),
    };
    number
        .parse::<f64>()
        .ok()
        .filter(|v| *v >= 0.0)
        .map(|v| Duration::from_micros((v * micros_per_unit).round() as u64))
}

pub(super) fn split_top_level(value: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let (mut depth, mut start) = (0, 0);
    for (i, c) in value.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(value[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(value[start..].trim());
    parts.into_iter().filter(|p| !p.is_empty()).collect()
}

pub(super) fn split_tokens(item: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let (mut depth, mut start) = (0, None);
    for (i, c) in item.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            c if c.is_whitespace() && depth == 0 => {
                if let Some(s) = start.take() {
                    tokens.push(&item[s..i]);
                }
                continue;
            }
            _ => {}
        }
        if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(s) = start {
        tokens.push(&item[s..]);
    }
    tokens
}

impl ComputedStyle {
    pub fn transition_for(&self, property: TransitionProperty) -> Option<&TransitionSpec> {
        self.transitions
            .iter()
            .rev()
            .find(|t| t.property.covers(property))
    }
}

#[derive(Clone, Copy, Debug)]
struct Anim<T> {
    from: T,
    to: T,
    start: Instant,
    spec: TransitionSpec,
}

impl<T: Copy> Anim<T> {
    fn progress(&self, now: Instant) -> Option<f32> {
        let elapsed = now.saturating_duration_since(self.start);
        if elapsed < self.spec.delay {
            return Some(0.0);
        }
        let t = (elapsed - self.spec.delay).as_secs_f32()
            / self.spec.duration.as_secs_f32().max(f32::EPSILON);
        (t < 1.0).then(|| self.spec.easing.apply(t))
    }
}

#[derive(Default)]
pub struct NodeTransitions {
    styled: bool,
    opacity: Option<Anim<f32>>,
    background: Option<Anim<Color>>,
    border: Option<Anim<Color>>,
    text: Option<Anim<Color>>,
    transform: Option<Anim<Transform>>,
}

fn opacity_of(style: &ComputedStyle) -> f32 {
    style.opacity.unwrap_or(1.0)
}

fn color_or_clear(color: Option<Color>, other: Option<Color>) -> Color {
    color.unwrap_or_else(|| other.map(|c| c.with_a(0)).unwrap_or(Color::TRANSPARENT))
}

impl NodeTransitions {
    pub(crate) fn update(&mut self, old: &ComputedStyle, new: &ComputedStyle) {
        if !self.styled {
            self.styled = true;
            return;
        }
        let now = Instant::now();

        let (old_o, new_o) = (opacity_of(old), opacity_of(new));
        if old_o != new_o {
            let current = self
                .opacity
                .and_then(|a| a.progress(now).map(|t| a.from + (a.to - a.from) * t))
                .unwrap_or(old_o);
            self.opacity = new
                .transition_for(TransitionProperty::Opacity)
                .filter(|s| !s.duration.is_zero())
                .map(|&spec| Anim {
                    from: current,
                    to: new_o,
                    start: now,
                    spec,
                });
        }

        let (old_t, new_t) = (
            old.transform.unwrap_or(Transform::IDENTITY),
            new.transform.unwrap_or(Transform::IDENTITY),
        );
        if old_t != new_t {
            let current = self
                .transform
                .and_then(|a| a.progress(now).map(|t| Transform::lerp(a.from, a.to, t)))
                .unwrap_or(old_t);
            self.transform = new
                .transition_for(TransitionProperty::Transform)
                .filter(|s| !s.duration.is_zero())
                .map(|&spec| Anim {
                    from: current,
                    to: new_t,
                    start: now,
                    spec,
                });
        }

        for (property, slot, old_c, new_c) in [
            (
                TransitionProperty::BackgroundColor,
                &mut self.background,
                old.background,
                new.background,
            ),
            (
                TransitionProperty::BorderColor,
                &mut self.border,
                old.border_color,
                new.border_color,
            ),
            (
                TransitionProperty::Color,
                &mut self.text,
                old.text_color,
                new.text_color,
            ),
        ] {
            if old_c == new_c {
                continue;
            }
            let target = color_or_clear(new_c, old_c);
            let current = slot
                .and_then(|a| a.progress(now).map(|t| lerp_color(a.from, a.to, t)))
                .unwrap_or_else(|| color_or_clear(old_c, new_c));
            *slot = new
                .transition_for(property)
                .filter(|s| !s.duration.is_zero())
                .map(|&spec| Anim {
                    from: current,
                    to: target,
                    start: now,
                    spec,
                });
        }
    }

    pub(crate) fn sample(&mut self, style: &ComputedStyle) -> Option<ComputedStyle> {
        if self.opacity.is_none()
            && self.background.is_none()
            && self.border.is_none()
            && self.text.is_none()
            && self.transform.is_none()
        {
            return None;
        }
        let now = Instant::now();
        let mut out = style.clone();
        let mut running = false;

        if let Some(a) = self.opacity {
            match a.progress(now) {
                Some(t) => {
                    running = true;
                    let o = a.from + (a.to - a.from) * t;
                    out.opacity = Some(o).filter(|o| *o < 1.0);
                }
                None => self.opacity = None,
            }
        }
        if let Some(a) = self.transform {
            match a.progress(now) {
                Some(t) => {
                    running = true;
                    out.transform =
                        Some(Transform::lerp(a.from, a.to, t)).filter(|t| !t.is_identity());
                }
                None => self.transform = None,
            }
        }
        for (slot, field) in [
            (&mut self.background, &mut out.background),
            (&mut self.border, &mut out.border_color),
            (&mut self.text, &mut out.text_color),
        ] {
            if let Some(a) = *slot {
                match a.progress(now) {
                    Some(t) => {
                        running = true;

                        *field = Some(lerp_color(a.from, a.to, t));
                    }
                    None => *slot = None,
                }
            }
        }
        running.then_some(out)
    }

    pub fn animates_background(style: &ComputedStyle) -> bool {
        style
            .transition_for(TransitionProperty::BackgroundColor)
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_discord_style_lists() {
        let t = parse("opacity 0.17s ease, color 50ms ease-in");
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].property, TransitionProperty::Opacity);
        assert_eq!(t[0].duration, Duration::from_millis(170));
        assert_eq!(t[1].property, TransitionProperty::Color);
        assert_eq!(t[1].duration, Duration::from_millis(50));
        assert_eq!(t[1].easing, Easing::CubicBezier(0.42, 0.0, 1.0, 1.0));
    }

    #[test]
    fn parses_delay_bezier_and_all() {
        let t = parse("all 0.2s cubic-bezier(0.4, 0, 0.2, 1) 0.3s");
        assert_eq!(t[0].property, TransitionProperty::All);
        assert_eq!(t[0].delay, Duration::from_millis(300));
        assert_eq!(t[0].easing, Easing::CubicBezier(0.4, 0.0, 0.2, 1.0));
        assert!(parse("none").is_empty());
        assert_eq!(
            parse("transform 0.2s")[0].property,
            TransitionProperty::Transform
        );
        assert!(parse("width 0.2s").is_empty());
    }

    #[test]
    fn easing_endpoints() {
        for e in [
            Easing::Linear,
            Easing::EASE,
            Easing::CubicBezier(0.0, 0.0, 0.58, 1.0),
        ] {
            assert!(e.apply(0.0).abs() < 1e-3);
            assert!((e.apply(1.0) - 1.0).abs() < 1e-3);
        }
        assert!((Easing::Linear.apply(0.25) - 0.25).abs() < 1e-6);
    }

    #[test]
    fn first_style_does_not_animate_then_changes_do() {
        let mut node = NodeTransitions::default();
        let hidden = ComputedStyle {
            opacity: Some(0.0),
            transitions: parse("opacity 1s linear"),
            ..Default::default()
        };
        node.update(&ComputedStyle::default(), &hidden);
        assert!(node.sample(&hidden).is_none());
        let shown = ComputedStyle {
            opacity: None,
            transitions: parse("opacity 1s linear"),
            ..Default::default()
        };
        node.update(&hidden, &shown);
        let mid = node.sample(&shown).expect("animating");
        assert!(mid.opacity.unwrap() < 0.1);
    }
}
