use std::collections::HashMap;
use std::mem;
use std::rc::Rc;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{Duration, Instant};

use skia_safe::Color;

use super::cascade::apply_declaration;
use super::parser::Declaration;
use super::transform::Transform;
use super::transition::{Easing, parse_time, split_tokens, split_top_level};
use super::{ComputedStyle, values};
use crate::widget::anim::lerp_color;

#[derive(Clone, Debug)]
pub struct Frame {
    pub offset: f32,
    pub declarations: Vec<Declaration>,
}

#[derive(Clone, Debug, Default)]
pub struct Keyframes {
    pub frames: Vec<Frame>,
}

fn registry() -> &'static RwLock<HashMap<String, Arc<Keyframes>>> {
    static REGISTRY: OnceLock<RwLock<HashMap<String, Arc<Keyframes>>>> = OnceLock::new();
    REGISTRY.get_or_init(Default::default)
}

pub(crate) fn register(name: &str, mut keyframes: Keyframes) {
    keyframes
        .frames
        .sort_by(|a, b| a.offset.total_cmp(&b.offset));
    registry()
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .insert(name.to_string(), Arc::new(keyframes));
}

pub(crate) fn resolve_all(
    custom_properties: &HashMap<String, String>,
) -> HashMap<String, Rc<ResolvedKeyframes>> {
    registry()
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .map(|(name, keyframes)| {
            (
                name.clone(),
                Rc::new(ResolvedKeyframes::resolve(keyframes, custom_properties)),
            )
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct FrameValues {
    opacity: Option<f32>,
    background: Option<Color>,
    border_color: Option<Color>,
    text_color: Option<Color>,
    transform: Option<Transform>,
}

#[derive(Debug)]
pub struct ResolvedKeyframes {
    frames: Vec<(f32, FrameValues)>,
}

impl ResolvedKeyframes {
    fn resolve(keyframes: &Keyframes, custom_properties: &HashMap<String, String>) -> Self {
        let frames = keyframes
            .frames
            .iter()
            .map(|frame| {
                let mut scratch = ComputedStyle::default();
                let mut values = FrameValues::default();
                for decl in &frame.declarations {
                    apply_declaration(
                        &mut scratch,
                        &decl.name,
                        &values::resolve_var(&decl.raw_value, custom_properties),
                    );
                    match decl.name.as_str() {
                        "opacity" => values.opacity = Some(scratch.opacity.unwrap_or(1.0)),
                        "background-color" | "background" => values.background = scratch.background,
                        "border-color" => values.border_color = scratch.border_color,
                        "color" => values.text_color = scratch.text_color,
                        "transform" => {
                            values.transform =
                                Some(scratch.transform.unwrap_or(Transform::IDENTITY))
                        }
                        _ => {}
                    }
                }
                (frame.offset, values)
            })
            .collect();
        Self { frames }
    }

    fn spills(&self) -> bool {
        self.frames
            .iter()
            .any(|(_, v)| v.opacity.is_some() || v.transform.is_some())
    }

    fn value_at<T: Copy>(
        &self,
        p: f32,
        easing: Easing,
        base: T,
        get: impl Fn(&FrameValues) -> Option<T>,
        lerp: impl Fn(T, T, f32) -> T,
    ) -> Option<T> {
        let mut stops: Vec<(f32, T)> = self
            .frames
            .iter()
            .filter_map(|(offset, v)| get(v).map(|value| (*offset, value)))
            .collect();
        if stops.is_empty() {
            return None;
        }
        if stops[0].0 > 0.0 {
            stops.insert(0, (0.0, base));
        }
        if stops[stops.len() - 1].0 < 1.0 {
            stops.push((1.0, base));
        }

        let next = stops
            .iter()
            .position(|(offset, _)| *offset > p)
            .unwrap_or(stops.len() - 1);
        let prev = next.saturating_sub(1);
        let (from_offset, from) = stops[prev];
        let (to_offset, to) = stops[next];
        let span = to_offset - from_offset;
        if span <= f32::EPSILON {
            return Some(to);
        }
        Some(lerp(from, to, easing.apply((p - from_offset) / span)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Iterations {
    Count(f32),
    Infinite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillMode {
    None,
    Forwards,
    Backwards,
    Both,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationSpec {
    pub name: String,
    pub duration: Duration,
    pub delay: Duration,
    pub easing: Easing,
    pub iterations: Iterations,
    pub direction: Direction,
    pub fill: FillMode,
    pub paused: bool,
}

impl Default for AnimationSpec {
    fn default() -> Self {
        Self {
            name: String::new(),
            duration: Duration::ZERO,
            delay: Duration::ZERO,
            easing: Easing::EASE,
            iterations: Iterations::Count(1.0),
            direction: Direction::Normal,
            fill: FillMode::None,
            paused: false,
        }
    }
}

pub fn parse(value: &str) -> Vec<AnimationSpec> {
    let value = value.trim();
    if value == "none" {
        return Vec::new();
    }
    split_top_level(value)
        .into_iter()
        .filter_map(|item| {
            let mut spec = AnimationSpec::default();
            let mut times = Vec::new();
            let mut named = false;
            for token in split_tokens(item) {
                if let Some(time) = parse_time(token) {
                    times.push(time);
                } else if let Some(easing) = Easing::parse(token) {
                    spec.easing = easing;
                } else if token == "infinite" {
                    spec.iterations = Iterations::Infinite;
                } else if let Ok(count) = token.parse::<f32>() {
                    spec.iterations = Iterations::Count(count.max(0.0));
                } else if let Some(direction) = parse_direction(token) {
                    spec.direction = direction;
                } else if let Some(fill) = parse_fill(token) {
                    spec.fill = fill;
                } else if token == "paused" || token == "running" {
                    spec.paused = token == "paused";
                } else if !named {
                    spec.name = token.to_string();
                    named = true;
                }
            }
            spec.duration = times.first().copied().unwrap_or(Duration::ZERO);
            spec.delay = times.get(1).copied().unwrap_or(Duration::ZERO);
            named.then_some(spec)
        })
        .collect()
}

fn parse_direction(token: &str) -> Option<Direction> {
    Some(match token {
        "normal" => Direction::Normal,
        "reverse" => Direction::Reverse,
        "alternate" => Direction::Alternate,
        "alternate-reverse" => Direction::AlternateReverse,
        _ => return None,
    })
}

fn parse_fill(token: &str) -> Option<FillMode> {
    Some(match token {
        "none" => FillMode::None,
        "forwards" => FillMode::Forwards,
        "backwards" => FillMode::Backwards,
        "both" => FillMode::Both,
        _ => return None,
    })
}

pub fn apply_longhand(list: &mut Vec<AnimationSpec>, property: &str, value: &str) {
    let values = split_top_level(value.trim());
    if list.len() < values.len() {
        list.resize_with(values.len(), AnimationSpec::default);
    }
    for (spec, value) in list.iter_mut().zip(values) {
        match property {
            "animation-name" => {
                spec.name = if value == "none" {
                    String::new()
                } else {
                    value.to_string()
                }
            }
            "animation-duration" => spec.duration = parse_time(value).unwrap_or(spec.duration),
            "animation-delay" => spec.delay = parse_time(value).unwrap_or(spec.delay),
            "animation-timing-function" => {
                spec.easing = Easing::parse(value).unwrap_or(spec.easing)
            }
            "animation-iteration-count" => {
                spec.iterations = if value == "infinite" {
                    Iterations::Infinite
                } else {
                    value
                        .parse::<f32>()
                        .map(|c| Iterations::Count(c.max(0.0)))
                        .unwrap_or(spec.iterations)
                }
            }
            "animation-direction" => {
                spec.direction = parse_direction(value).unwrap_or(spec.direction)
            }
            "animation-fill-mode" => spec.fill = parse_fill(value).unwrap_or(spec.fill),
            "animation-play-state" => spec.paused = value == "paused",
            _ => {}
        }
    }
}

struct Running {
    spec: AnimationSpec,
    keyframes: Rc<ResolvedKeyframes>,
    start: Instant,
}

pub struct Sampled {
    pub style: ComputedStyle,

    pub spills: bool,

    pub running: bool,
}

#[derive(Default)]
pub struct NodeAnimations {
    running: Vec<Running>,
}

impl NodeAnimations {
    pub(crate) fn update(
        &mut self,
        specs: &[AnimationSpec],
        keyframes: &HashMap<String, Rc<ResolvedKeyframes>>,
    ) {
        if specs.is_empty() && self.running.is_empty() {
            return;
        }
        let now = Instant::now();
        let mut previous = mem::take(&mut self.running);
        for spec in specs
            .iter()
            .filter(|s| !s.name.is_empty() && !s.duration.is_zero())
        {
            let Some(found) = keyframes.get(&spec.name) else {
                continue;
            };

            let carried = previous
                .iter()
                .position(|r| {
                    r.spec.name == spec.name
                        && r.spec.duration == spec.duration
                        && r.spec.delay == spec.delay
                })
                .map(|i| previous.swap_remove(i));
            let start = carried.map_or(now, |r| r.start);
            self.running.push(Running {
                spec: spec.clone(),
                keyframes: found.clone(),
                start,
            });
        }
    }

    pub fn is_empty(&self) -> bool {
        self.running.is_empty()
    }

    pub(crate) fn sample(&mut self, style: &ComputedStyle) -> Option<Sampled> {
        if self.running.is_empty() {
            return None;
        }
        let now = Instant::now();
        let mut out = style.clone();
        let mut applied = false;
        let mut live = false;
        let mut spills = false;
        self.running.retain(|running| {
            let state = progress(&running.spec, now.saturating_duration_since(running.start));
            if matches!(state, State::Finished(_))
                && !matches!(running.spec.fill, FillMode::Forwards | FillMode::Both)
            {
                return false;
            }
            let p = match state {
                State::Before => {
                    (matches!(running.spec.fill, FillMode::Backwards | FillMode::Both))
                        .then(|| start_progress(&running.spec))
                }
                State::Running(p) => Some(p),
                State::Finished(p) => Some(p),
            };
            live |= !matches!(state, State::Finished(_));
            if let Some(p) = p {
                apply(&mut out, &running.keyframes, p, running.spec.easing);
                applied = true;
                spills |= running.keyframes.spills();
            }
            true
        });
        applied.then_some(Sampled {
            style: out,
            spills,
            running: live,
        })
    }
}

enum State {
    Before,

    Running(f32),

    Finished(f32),
}

fn start_progress(spec: &AnimationSpec) -> f32 {
    directed(spec.direction, 0, 0.0)
}

fn directed(direction: Direction, iteration: u64, fraction: f32) -> f32 {
    let reversed = match direction {
        Direction::Normal => false,
        Direction::Reverse => true,
        Direction::Alternate => iteration % 2 == 1,
        Direction::AlternateReverse => iteration.is_multiple_of(2),
    };
    if reversed { 1.0 - fraction } else { fraction }
}

fn progress(spec: &AnimationSpec, elapsed: Duration) -> State {
    if elapsed < spec.delay {
        return State::Before;
    }
    let t = (elapsed - spec.delay).as_secs_f64() / spec.duration.as_secs_f64().max(f64::EPSILON);
    if let Iterations::Count(count) = spec.iterations {
        let count = count as f64;
        if t >= count {
            let last = (count.ceil() as u64).saturating_sub(1);
            let fraction = if count.fract() == 0.0 {
                1.0
            } else {
                count.fract() as f32
            };
            return State::Finished(directed(spec.direction, last, fraction));
        }
    }
    let iteration = t.floor() as u64;
    State::Running(directed(spec.direction, iteration, t.fract() as f32))
}

fn apply(out: &mut ComputedStyle, keyframes: &ResolvedKeyframes, p: f32, easing: Easing) {
    let color_base = |own: Option<Color>, other: Option<Color>| {
        own.unwrap_or_else(|| other.map(|c| c.with_a(0)).unwrap_or(Color::TRANSPARENT))
    };
    let own = out.clone();
    if let Some(o) = keyframes.value_at(
        p,
        easing,
        own.opacity.unwrap_or(1.0),
        |v| v.opacity,
        |a, b, t| a + (b - a) * t,
    ) {
        out.opacity = Some(o).filter(|o| *o < 1.0);
    }
    if let Some(t) = keyframes.value_at(
        p,
        easing,
        own.transform.unwrap_or(Transform::IDENTITY),
        |v| v.transform,
        Transform::lerp,
    ) {
        out.transform = Some(t).filter(|t| !t.is_identity());
    }
    let mut colors = [
        (&mut out.background, own.background, 0),
        (&mut out.border_color, own.border_color, 1),
        (&mut out.text_color, own.text_color, 2),
    ];
    for (slot, base, which) in colors.iter_mut() {
        let get = |v: &FrameValues| match which {
            0 => v.background,
            1 => v.border_color,
            _ => v.text_color,
        };

        let other = keyframes.frames.iter().find_map(|(_, v)| get(v));
        if let Some(c) = keyframes.value_at(p, easing, color_base(*base, other), get, lerp_color) {
            **slot = Some(c);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use super::*;

    fn keyframes(name: &str, css: &str) -> Rc<ResolvedKeyframes> {
        let rules = super::super::parser::parse_stylesheet(css);
        assert!(rules.is_empty(), "keyframes are not style rules");
        resolve_all(&HashMap::new())
            .remove(name)
            .expect("registered")
    }

    #[test]
    fn parses_the_clients_shorthand() {
        let list = parse("1.6s ease-in-out 0s infinite normal none running pulse__1add6");
        assert_eq!(list.len(), 1);
        let spec = &list[0];
        assert_eq!(spec.name, "pulse__1add6");
        assert_eq!(spec.duration, Duration::from_millis(1600));
        assert_eq!(spec.delay, Duration::ZERO);
        assert_eq!(spec.easing, Easing::CubicBezier(0.42, 0.0, 0.58, 1.0));
        assert_eq!(spec.iterations, Iterations::Infinite);
        assert_eq!(spec.fill, FillMode::None);
        assert!(!spec.paused);
        assert!(parse("none").is_empty());
    }

    #[test]
    fn parses_counts_directions_and_longhands() {
        let spec = &parse("spin 2s linear 0.5s 3 alternate both")[0];
        assert_eq!(
            (spec.iterations, spec.direction, spec.fill, spec.delay),
            (
                Iterations::Count(3.0),
                Direction::Alternate,
                FillMode::Both,
                Duration::from_millis(500)
            )
        );
        let mut list = Vec::new();
        apply_longhand(&mut list, "animation-name", "a, b");
        apply_longhand(&mut list, "animation-duration", "1s, 2s");
        apply_longhand(&mut list, "animation-iteration-count", "infinite");
        assert_eq!(
            list.iter()
                .map(|s| (s.name.as_str(), s.duration))
                .collect::<Vec<_>>(),
            [("a", Duration::from_secs(1)), ("b", Duration::from_secs(2))]
        );
        assert_eq!(list[0].iterations, Iterations::Infinite);
        assert_eq!(list[1].iterations, Iterations::Count(1.0));
    }

    #[test]
    fn the_pulse_goes_bright_dim_bright() {
        let kf = keyframes(
            "test-pulse",
            "@keyframes test-pulse { 0%, 100% { background-color: #ff0000; } 50% { background-color: #0000ff; } }",
        );
        let at = |p: f32| {
            kf.value_at(
                p,
                Easing::Linear,
                Color::BLACK,
                |v| v.background,
                lerp_color,
            )
            .unwrap()
        };
        assert_eq!(at(0.0), Color::from_rgb(255, 0, 0));
        assert_eq!(at(0.5), Color::from_rgb(0, 0, 255));
        assert_eq!(at(1.0), Color::from_rgb(255, 0, 0));
        let quarter = at(0.25);
        assert!(
            quarter.r() > 100 && quarter.r() < 160 && quarter.b() > 100 && quarter.b() < 160,
            "{quarter:?}"
        );
    }

    #[test]
    fn a_missing_end_frame_falls_back_to_the_elements_own_value() {
        let kf = keyframes("test-fade", "@keyframes test-fade { 50% { opacity: 0; } }");
        let at = |p: f32| {
            kf.value_at(
                p,
                Easing::Linear,
                1.0,
                |v| v.opacity,
                |a, b, t| a + (b - a) * t,
            )
            .unwrap()
        };
        assert_eq!(at(0.0), 1.0);
        assert_eq!(at(0.5), 0.0);
        assert!((at(0.75) - 0.5).abs() < 1e-6);
        assert_eq!(at(1.0), 1.0);
    }

    #[test]
    fn timing_alternates_and_finishes() {
        let spec = AnimationSpec {
            duration: Duration::from_secs(1),
            iterations: Iterations::Count(2.0),
            direction: Direction::Alternate,
            ..Default::default()
        };
        let p = |ms: u64| match progress(&spec, Duration::from_millis(ms)) {
            State::Running(p) | State::Finished(p) => p,
            State::Before => -1.0,
        };
        assert!((p(250) - 0.25).abs() < 1e-5);
        assert!((p(1250) - 0.75).abs() < 1e-5);
        assert!(
            matches!(progress(&spec, Duration::from_millis(2500)), State::Finished(p) if p == 0.0)
        );
        let delayed = AnimationSpec {
            delay: Duration::from_secs(1),
            ..spec
        };
        assert!(matches!(
            progress(&delayed, Duration::from_millis(500)),
            State::Before
        ));
    }

    #[test]
    fn a_node_keeps_its_clock_across_restyles_and_drops_finished_animations() {
        let kf = keyframes(
            "test-clock",
            "@keyframes test-clock { from { opacity: 0; } to { opacity: 1; } }",
        );
        let map = HashMap::from([("test-clock".to_string(), kf)]);
        let mut node = NodeAnimations::default();
        let specs = parse("1s linear infinite test-clock");
        node.update(&specs, &map);
        let started = node.running[0].start;
        node.update(&specs, &map);
        assert_eq!(node.running[0].start, started);
        node.update(&[], &map);
        assert!(node.is_empty());

        let style = ComputedStyle::default();
        let once = parse("60ms linear test-clock");
        node.update(&once, &map);
        assert!(node.sample(&style).is_some());
        thread::sleep(Duration::from_millis(150));
        assert!(node.sample(&style).is_none());
        assert!(node.is_empty());
    }
}
