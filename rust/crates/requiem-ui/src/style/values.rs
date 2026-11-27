use std::cell::Cell;
use std::collections::HashMap;

use super::calc::{intern, parse_calc, split_tokens};
use skia_safe::Color;
use taffy::style_helpers::{FromFr, FromLength, TaffyAuto};
use taffy::{
    Dimension, GridTemplateComponent, LengthPercentage, LengthPercentageAuto, Rect,
    TrackSizingFunction,
};

pub fn resolve_var(raw: &str, custom_properties: &HashMap<String, String>) -> String {
    resolve_var_depth(raw.trim(), custom_properties, 0)
}

const MAX_VAR_DEPTH: usize = 8;

fn resolve_var_depth(
    raw: &str,
    custom_properties: &HashMap<String, String>,
    depth: usize,
) -> String {
    if depth > MAX_VAR_DEPTH || !raw.contains("var(") {
        return raw.to_string();
    }

    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(start) = rest.find("var(") {
        out.push_str(&rest[..start]);
        let body_start = start + "var(".len();

        let mut paren_depth = 1usize;
        let mut end = None;
        for (i, ch) in rest[body_start..].char_indices() {
            match ch {
                '(' => paren_depth += 1,
                ')' => {
                    paren_depth -= 1;
                    if paren_depth == 0 {
                        end = Some(body_start + i);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(end) = end else {
            out.push_str(&rest[start..]);
            return out;
        };

        let inner = &rest[body_start..end];
        let mut parts = inner.splitn(2, ',');
        let name = parts.next().unwrap_or("").trim();
        let name = name.strip_prefix("--").unwrap_or(name);
        let fallback = parts.next().map(str::trim);

        let substituted = match custom_properties.get(name) {
            Some(value) => value.trim().to_string(),
            None => fallback.unwrap_or_default().to_string(),
        };
        out.push_str(&resolve_var_depth(
            &substituted,
            custom_properties,
            depth + 1,
        ));
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    out
}

pub fn parse_box_shadow(value: &str) -> Vec<super::BoxShadow> {
    let value = value.trim();
    if value == "none" {
        return Vec::new();
    }
    value
        .split(',')
        .filter_map(|layer| {
            let mut lengths = Vec::new();
            let mut color = None;
            let mut inset = false;
            for token in layer.split_whitespace() {
                if token == "inset" {
                    inset = true;
                    continue;
                }
                if let Some(c) = parse_color(token) {
                    color = Some(c);
                } else {
                    lengths.push(if token == "0" {
                        0.0
                    } else {
                        parse_length_px(token)?
                    });
                }
            }
            if lengths.len() < 2 {
                return None;
            }
            Some(super::BoxShadow {
                x: lengths[0],
                y: lengths[1],
                blur: lengths.get(2).copied().unwrap_or(0.0),
                spread: lengths.get(3).copied().unwrap_or(0.0),
                color: color?,
                inset,
            })
        })
        .collect()
}

pub fn parse_filter(value: &str) -> Option<super::Filter> {
    let mut filter = super::Filter {
        blur: 0.0,
        brightness: 1.0,
        drop_shadow: None,
    };
    let mut any = false;
    let mut rest = value.trim();
    while let Some(open) = rest.find('(') {
        let name = rest[..open].trim();
        let close = rest[open..].find(')')? + open;
        let arg = rest[open + 1..close].trim();
        match name {
            "blur" => filter.blur = parse_length_px(arg)?,
            "brightness" => {
                filter.brightness = match arg.strip_suffix('%') {
                    Some(pct) => pct.trim().parse::<f32>().ok()? / 100.0,
                    None => arg.parse::<f32>().ok()?,
                }
            }
            "drop-shadow" => {
                let mut parts = arg.splitn(4, char::is_whitespace);
                let x = parse_length_px(parts.next()?)?;
                let y = parse_length_px(parts.next()?)?;
                let blur = parse_length_px(parts.next()?)?;
                let color = parse_color(parts.next()?)?;
                filter.drop_shadow = Some(super::DropShadow { x, y, blur, color });
            }
            _ => return None,
        }
        any = true;
        rest = &rest[close + 1..];
    }
    any.then_some(filter)
}

pub fn parse_color(value: &str) -> Option<Color> {
    let hex = value.trim().strip_prefix('#')?;
    let (r, g, b, a) = match hex.len() {
        6 => (
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
            255,
        ),
        8 => (
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
            u8::from_str_radix(&hex[6..8], 16).ok()?,
        ),
        _ => return None,
    };
    Some(Color::from_argb(a, r, g, b))
}

thread_local! {

    static VIEWPORT: Cell<(f32, f32)> = const { Cell::new((0.0, 0.0)) };

    static VIEWPORT_UNITS_USED: Cell<bool> = const { Cell::new(false) };
}

pub fn viewport() -> (f32, f32) {
    VIEWPORT.with(Cell::get)
}

pub fn set_viewport(width: f32, height: f32) -> bool {
    let changed = VIEWPORT.with(|v| v.replace((width, height))) != (width, height);
    changed && VIEWPORT_UNITS_USED.with(Cell::get)
}

fn parse_viewport_length(value: &str) -> Option<f32> {
    let split = value.find(|c: char| c.is_ascii_alphabetic())?;
    let (number, unit) = value.split_at(split);
    let unit = unit
        .strip_prefix(['s', 'l', 'd'])
        .filter(|u| matches!(*u, "vw" | "vh" | "vmin" | "vmax"))
        .unwrap_or(unit);
    let basis: fn((f32, f32)) -> f32 = match unit {
        "vw" => |(w, _)| w,
        "vh" => |(_, h)| h,
        "vmin" => |(w, h)| w.min(h),
        "vmax" => |(w, h)| w.max(h),
        _ => return None,
    };
    let n = number.trim().parse::<f32>().ok()?;
    VIEWPORT_UNITS_USED.with(|u| u.set(true));
    let viewport = VIEWPORT.with(Cell::get);
    (viewport.0 > 0.0 && viewport.1 > 0.0).then(|| n * basis(viewport) / 100.0)
}

pub fn parse_length_px(value: &str) -> Option<f32> {
    let value = value.trim();
    if let Some(px) = value.strip_suffix("px") {
        return px.trim().parse::<f32>().ok();
    }
    if value.starts_with("calc(") {
        return parse_calc(value)
            .filter(|lin| lin.pct == 0.0)
            .map(|lin| lin.px);
    }
    parse_viewport_length(value)
}

pub fn parse_padding_shorthand(value: &str) -> Option<Rect<LengthPercentage>> {
    let lengths: Vec<f32> = split_tokens(value)
        .into_iter()
        .map(parse_length_px)
        .collect::<Option<Vec<_>>>()?;

    let (top, right, bottom, left) = match lengths.as_slice() {
        [all] => (*all, *all, *all, *all),
        [vertical, horizontal] => (*vertical, *horizontal, *vertical, *horizontal),
        [top, horizontal, bottom] => (*top, *horizontal, *bottom, *horizontal),
        [top, right, bottom, left] => (*top, *right, *bottom, *left),
        _ => return None,
    };

    Some(Rect {
        top: LengthPercentage::length(top),
        right: LengthPercentage::length(right),
        bottom: LengthPercentage::length(bottom),
        left: LengthPercentage::length(left),
    })
}

pub fn parse_margin_token(token: &str) -> Option<LengthPercentageAuto> {
    if token.trim() == "auto" {
        return Some(LengthPercentageAuto::auto());
    }
    if let Some(lin) = parse_calc(token).filter(|lin| lin.pct != 0.0) {
        return Some(LengthPercentageAuto::calc(intern(lin)));
    }
    parse_length_px(token).map(LengthPercentageAuto::length)
}

pub fn parse_margin_shorthand(value: &str) -> Option<Rect<LengthPercentageAuto>> {
    let tokens: Vec<LengthPercentageAuto> = split_tokens(value)
        .into_iter()
        .map(parse_margin_token)
        .collect::<Option<Vec<_>>>()?;

    let (top, right, bottom, left) = match tokens.as_slice() {
        [all] => (*all, *all, *all, *all),
        [vertical, horizontal] => (*vertical, *horizontal, *vertical, *horizontal),
        [top, horizontal, bottom] => (*top, *horizontal, *bottom, *horizontal),
        [top, right, bottom, left] => (*top, *right, *bottom, *left),
        _ => return None,
    };

    Some(Rect {
        top,
        right,
        bottom,
        left,
    })
}

pub fn parse_border_shorthand(value: &str) -> Option<(f32, Color)> {
    let mut parts = split_tokens(value).into_iter();
    let width = parse_length_px(parts.next()?)?;
    let _style = parts.next()?;
    let color = parse_color(parts.next()?)?;
    Some((width, color))
}

pub fn parse_inset_token(value: &str) -> Option<LengthPercentageAuto> {
    let value = value.trim();
    if value == "auto" {
        return Some(LengthPercentageAuto::auto());
    }
    if let Some(pct) = value.strip_suffix('%') {
        return pct
            .trim()
            .parse::<f32>()
            .ok()
            .map(|p| LengthPercentageAuto::percent(p / 100.0));
    }
    if let Some(lin) = parse_calc(value).filter(|lin| lin.pct != 0.0) {
        return Some(LengthPercentageAuto::calc(intern(lin)));
    }
    parse_length_px(value).map(LengthPercentageAuto::length)
}

pub fn parse_inset_shorthand(value: &str) -> Option<f32> {
    parse_length_px(value)
}

pub fn parse_flex_shorthand(value: &str) -> Option<(f32, f32, Dimension)> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.as_slice() {
        ["none"] => Some((0.0, 0.0, Dimension::auto())),
        ["auto"] => Some((1.0, 1.0, Dimension::auto())),
        [grow] => grow
            .parse::<f32>()
            .ok()
            .map(|g| (g, 1.0, Dimension::length(0.0))),
        [grow, shrink] => match shrink.parse::<f32>() {
            Ok(s) => grow
                .parse::<f32>()
                .ok()
                .map(|g| (g, s, Dimension::length(0.0))),
            Err(_) => grow
                .parse::<f32>()
                .ok()
                .and_then(|g| parse_dimension(shrink).map(|b| (g, 1.0, b))),
        },
        [grow, shrink, basis] => {
            let (g, s) = (grow.parse::<f32>().ok()?, shrink.parse::<f32>().ok()?);
            parse_dimension(basis).map(|b| (g, s, b))
        }
        _ => None,
    }
}

pub fn parse_dimension(value: &str) -> Option<Dimension> {
    let value = value.trim();
    if value == "auto" {
        return Some(Dimension::auto());
    }
    if let Some(pct) = value.strip_suffix('%') {
        return pct
            .trim()
            .parse::<f32>()
            .ok()
            .map(|p| Dimension::percent(p / 100.0));
    }
    if let Some(lin) = parse_calc(value).filter(|lin| lin.pct != 0.0) {
        return Some(Dimension::calc(intern(lin)));
    }
    parse_length_px(value).map(Dimension::length)
}

fn parse_track_sizing_function(token: &str) -> Option<TrackSizingFunction> {
    let token = token.trim();
    if token == "auto" {
        return Some(TrackSizingFunction::AUTO);
    }
    if let Some(fr) = token.strip_suffix("fr") {
        return fr
            .trim()
            .parse::<f32>()
            .ok()
            .map(TrackSizingFunction::from_fr);
    }
    parse_length_px(token).map(TrackSizingFunction::from_length)
}

pub fn parse_grid_template_columns(value: &str) -> Option<Vec<GridTemplateComponent<String>>> {
    split_tokens(value)
        .into_iter()
        .map(|token| parse_track_sizing_function(token).map(GridTemplateComponent::Single))
        .collect::<Option<Vec<_>>>()
}

#[cfg(test)]
mod margin_shorthand_tests {
    use super::*;

    fn rect(top: f32, right: f32, bottom: f32, left: f32) -> Rect<LengthPercentageAuto> {
        Rect {
            top: LengthPercentageAuto::length(top),
            right: LengthPercentageAuto::length(right),
            bottom: LengthPercentageAuto::length(bottom),
            left: LengthPercentageAuto::length(left),
        }
    }

    #[test]
    fn one_value_applies_to_all_sides() {
        assert_eq!(
            parse_margin_shorthand("32px"),
            Some(rect(32.0, 32.0, 32.0, 32.0))
        );
    }

    #[test]
    fn two_values_are_vertical_then_horizontal() {
        assert_eq!(
            parse_margin_shorthand("8px 16px"),
            Some(rect(8.0, 16.0, 8.0, 16.0))
        );
    }

    #[test]
    fn three_values_are_top_horizontal_bottom() {
        assert_eq!(
            parse_margin_shorthand("1px 2px 3px"),
            Some(rect(1.0, 2.0, 3.0, 2.0))
        );
    }

    #[test]
    fn four_values_are_top_right_bottom_left() {
        assert_eq!(
            parse_margin_shorthand("1px 2px 3px 4px"),
            Some(rect(1.0, 2.0, 3.0, 4.0))
        );
    }

    #[test]
    fn rejects_unparseable_input() {
        assert_eq!(parse_margin_shorthand("not-a-value"), None);
        assert_eq!(parse_margin_shorthand("1px 2px 3px 4px 5px"), None);
    }

    #[test]
    fn flex_shorthand_expands_like_css() {
        assert_eq!(
            parse_flex_shorthand("1 1 auto"),
            Some((1.0, 1.0, Dimension::auto()))
        );
        assert_eq!(
            parse_flex_shorthand("1 1 0px"),
            Some((1.0, 1.0, Dimension::length(0.0)))
        );
        assert_eq!(
            parse_flex_shorthand("2"),
            Some((2.0, 1.0, Dimension::length(0.0)))
        );
        assert_eq!(
            parse_flex_shorthand("none"),
            Some((0.0, 0.0, Dimension::auto()))
        );
        assert_eq!(
            parse_flex_shorthand("1 80px"),
            Some((1.0, 1.0, Dimension::length(80.0)))
        );
        assert_eq!(parse_flex_shorthand("bogus"), None);
    }

    #[test]
    fn auto_is_accepted_per_side() {
        let rect = parse_margin_shorthand("0px auto").expect("should parse");
        assert_eq!(rect.top, LengthPercentageAuto::length(0.0));
        assert_eq!(rect.right, LengthPercentageAuto::auto());
        assert_eq!(rect.bottom, LengthPercentageAuto::length(0.0));
        assert_eq!(rect.left, LengthPercentageAuto::auto());
    }
}

#[cfg(test)]
mod grid_template_columns_tests {
    use super::*;

    fn tracks(cols: Vec<GridTemplateComponent<String>>) -> Vec<TrackSizingFunction> {
        cols.into_iter()
            .map(|c| match c {
                GridTemplateComponent::Single(track) => track,
                GridTemplateComponent::Repeat(_) => panic!("this parser never produces a repeat()"),
            })
            .collect()
    }

    #[test]
    fn parses_fixed_pixel_tracks() {
        let cols = parse_grid_template_columns("289px 383px").expect("should parse");
        assert_eq!(
            tracks(cols),
            vec![
                TrackSizingFunction::from_length(289.0_f32),
                TrackSizingFunction::from_length(383.0_f32)
            ]
        );
    }

    #[test]
    fn parses_auto_and_fr_tracks() {
        let cols = parse_grid_template_columns("auto 1fr").expect("should parse");
        assert_eq!(
            tracks(cols),
            vec![
                TrackSizingFunction::AUTO,
                TrackSizingFunction::from_fr(1.0_f32)
            ]
        );
    }

    #[test]
    fn rejects_unparseable_track() {
        assert!(parse_grid_template_columns("289px repeat(2, 1fr)").is_none());
    }
}

#[cfg(test)]
mod var_tests {
    use super::resolve_var;
    use std::collections::HashMap;

    fn props() -> HashMap<String, String> {
        HashMap::from([
            ("border-subtle".to_string(), "#94949c1f".to_string()),
            ("alias".to_string(), "var(--border-subtle)".to_string()),
        ])
    }

    #[test]
    fn whole_value() {
        assert_eq!(resolve_var("var(--border-subtle)", &props()), "#94949c1f");
    }

    #[test]
    fn inside_shorthand() {
        assert_eq!(
            resolve_var("1px solid var(--border-subtle)", &props()),
            "1px solid #94949c1f"
        );
    }

    #[test]
    fn token_referencing_token() {
        assert_eq!(resolve_var("var(--alias)", &props()), "#94949c1f");
    }

    #[test]
    fn nested_fallback() {
        assert_eq!(
            resolve_var("var(--missing, var(--border-subtle))", &props()),
            "#94949c1f"
        );
        assert_eq!(resolve_var("var(--missing, #fff)", &props()), "#fff");
    }

    #[test]
    fn no_var_is_untouched() {
        assert_eq!(resolve_var(" 8px ", &props()), "8px");
    }
}

#[cfg(test)]
mod filter_tests {
    use super::parse_filter;

    #[test]
    fn parses_blur_and_brightness() {
        let f = parse_filter("brightness(60%) blur(4px)").unwrap();
        assert_eq!((f.blur, f.brightness), (4.0, 0.6));
        assert!(parse_filter("none").is_none());
    }
}

#[cfg(test)]
mod viewport_tests {
    use super::*;

    #[test]
    fn viewport_units_resolve_against_the_viewport() {
        assert_eq!(
            parse_length_px("50vh"),
            None,
            "unknown viewport ignores the value"
        );
        assert!(!set_viewport(0.0, 0.0), "no change, nothing to re-cascade");
        assert!(
            set_viewport(1000.0, 800.0),
            "a units user exists, so a change re-cascades"
        );
        assert_eq!(parse_length_px("50vh"), Some(400.0));
        assert_eq!(parse_length_px("10vw"), Some(100.0));
        assert_eq!(parse_length_px("50dvh"), Some(400.0));
        assert_eq!(parse_length_px("100vmin"), Some(800.0));
        assert_eq!(parse_length_px("100vmax"), Some(1000.0));
        assert_eq!(parse_length_px("12px"), Some(12.0));
        assert_eq!(parse_length_px("12deg"), None);
        assert!(matches!(parse_dimension("50vh"), Some(d) if d == Dimension::length(400.0)));
        assert!(!set_viewport(1000.0, 800.0));
        assert!(set_viewport(1000.0, 600.0));
        assert_eq!(parse_length_px("50vh"), Some(300.0));
    }
}
