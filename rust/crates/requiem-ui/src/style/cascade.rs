use crate::event::CursorIcon;
use crate::perf::Phase;
use crate::perf;
use cssparser::ToCss;
use std::cell::RefCell;
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::mem;
use std::rc::Rc;
use super::ScrollbarKind;
use taffy::style::Dimension;

use selectors::context::{
    MatchingContext, MatchingForInvalidation, MatchingMode, NeedsSelectorFlags, QuirksMode,
    SelectorCaches,
};
use selectors::matching::matches_selector;
use selectors::parser::{Combinator, Component};
use taffy::{
    AlignItems, Display, FlexDirection, JustifyContent, LengthPercentage, LengthPercentageAuto,
    Overflow, Position, Rect as TaffyRect, Size as TaffySize,
};

use super::ComputedStyle;
use super::parser::StyleRule;
use super::selector_impl::{ElementRef, PseudoClass};
use super::values;
use crate::widget::{WidgetId, WidgetTree};

pub fn apply_stylesheet(tree: &mut WidgetTree, rules: &[StyleRule]) {
    tree.rules = Some(Rc::new(StyleIndex::new(rules.to_vec())));
    recompute(tree);
}

/// Theme tokens are resolved when the index is built. A live switch leaves the
/// parsed rules in place and only rebuilds the resolved values. Popouts share
/// this index, so the resolved values live in one cell every window reads.
pub fn refresh_theme(tree: &mut WidgetTree) -> bool {
    let Some(index) = tree.rules.clone() else {
        return false;
    };
    if !index.rebind_theme() {
        return false;
    }
    tree.invalidate_styles();
    true
}

struct ResolvedTheme {
    resolved: Vec<Vec<String>>,
    custom_properties: HashMap<String, String>,
    keyframes: HashMap<String, Rc<super::animation::ResolvedKeyframes>>,
    theme: String,
}

pub struct StyleIndex {
    rules: Vec<StyleRule>,
    resolved: RefCell<ResolvedTheme>,
    by_class: HashMap<String, Vec<SelectorRef>>,
    by_id: HashMap<String, Vec<SelectorRef>>,
    by_tag: HashMap<String, Vec<SelectorRef>>,

    universal: Vec<SelectorRef>,

    ancestor_interactive: bool,

    sibling_interactive: bool,
}

fn is_interactive(pc: &PseudoClass) -> bool {
    matches!(
        pc,
        PseudoClass::Hover | PseudoClass::Active | PseudoClass::Focus | PseudoClass::FocusWithin
    )
}

type SelectorRef = (u32, u32);

impl StyleIndex {
    fn new(rules: Vec<StyleRule>) -> Self {
        let custom_properties = custom_properties_for(&rules, &crate::theme_name());
        let keyframes = super::animation::resolve_all(&custom_properties);
        let resolved = rules
            .iter()
            .map(|rule| {
                rule.declarations
                    .iter()
                    .map(|decl| values::resolve_var(&decl.raw_value, &custom_properties))
                    .collect()
            })
            .collect();
        let mut by_class: HashMap<String, Vec<SelectorRef>> = HashMap::new();
        let mut by_id: HashMap<String, Vec<SelectorRef>> = HashMap::new();
        let mut by_tag: HashMap<String, Vec<SelectorRef>> = HashMap::new();
        let mut universal = Vec::new();
        let mut ancestor_interactive = false;
        let mut sibling_interactive = false;
        for (rule_idx, rule) in rules.iter().enumerate() {
            for (sel_idx, selector) in rule.selectors.slice().iter().enumerate() {
                let key = (rule_idx as u32, sel_idx as u32);
                let (mut class, mut id, mut tag) = (None, None, None);
                for component in selector.iter() {
                    match component {
                        Component::Class(name) => class = class.or(Some(name.0.clone())),
                        Component::ID(name) => id = id.or(Some(name.0.clone())),
                        Component::LocalName(local) => tag = tag.or(Some(local.name.0.clone())),
                        _ => {}
                    }
                }

                let mut reach = 0u8;
                for component in selector.iter_raw_match_order() {
                    match component {
                        Component::Combinator(c) => {
                            reach =
                                if matches!(*c, Combinator::NextSibling | Combinator::LaterSibling)
                                {
                                    2
                                } else {
                                    1
                                };
                        }
                        Component::NonTSPseudoClass(pc) if is_interactive(pc) => match reach {
                            1 => ancestor_interactive = true,
                            2 => sibling_interactive = true,
                            _ => {}
                        },
                        _ => {}
                    }
                }
                match (class, id, tag) {
                    (Some(class), _, _) => by_class.entry(class).or_default().push(key),
                    (None, Some(id), _) => by_id.entry(id).or_default().push(key),
                    (None, None, Some(tag)) => by_tag.entry(tag).or_default().push(key),
                    (None, None, None) => universal.push(key),
                }
            }
        }
        crate::install_theme_tokens(custom_properties.clone());
        StyleIndex {
            rules,
            resolved: RefCell::new(ResolvedTheme {
                resolved,
                custom_properties,
                keyframes,
                theme: crate::theme_name(),
            }),
            by_class,
            by_id,
            by_tag,
            universal,
            ancestor_interactive,
            sibling_interactive,
        }
    }

    fn rebind_theme(&self) -> bool {
        let theme = crate::theme_name();
        let mut slot = self.resolved.borrow_mut();
        if slot.theme == theme {
            return false;
        }
        let custom_properties = custom_properties_for(&self.rules, &theme);
        slot.keyframes = super::animation::resolve_all(&custom_properties);
        slot.resolved = self
            .rules
            .iter()
            .map(|rule| {
                rule.declarations
                    .iter()
                    .map(|decl| values::resolve_var(&decl.raw_value, &custom_properties))
                    .collect()
            })
            .collect();
        slot.custom_properties = custom_properties;
        slot.theme = theme;
        crate::install_theme_tokens(slot.custom_properties.clone());
        true
    }

    fn candidates(&self, tree: &WidgetTree, id: WidgetId, out: &mut Vec<SelectorRef>) {
        out.clear();
        let node = &tree.nodes[id];
        out.extend_from_slice(&self.universal);
        for class in &node.classes {
            if let Some(refs) = self.by_class.get(class) {
                out.extend_from_slice(refs);
            }
        }
        if let Some(refs) = node
            .dom_id
            .as_ref()
            .and_then(|dom_id| self.by_id.get(dom_id))
        {
            out.extend_from_slice(refs);
        }
        if let Some(refs) = self.by_tag.get(node.tag) {
            out.extend_from_slice(refs);
        }
        out.sort_unstable();
        out.dedup();
    }
}

pub(crate) fn recompute_if_dirty(tree: &mut WidgetTree) {
    if tree.style_dirty {
        recompute(tree);
    } else if !tree.interactive_changed.is_empty() {
        recompute_interactive(tree);
    }
}

pub(crate) fn recompute(tree: &mut WidgetTree) {
    tree.style_dirty = false;

    tree.interactive_changed.clear();
    let Some(index) = tree.rules.clone() else {
        return;
    };
    perf::time(Phase::Cascade, || recompute_all(tree, &index));
}

fn recompute_interactive(tree: &mut WidgetTree) {
    let Some(index) = tree.rules.clone() else {
        tree.interactive_changed.clear();
        return;
    };
    let changed = mem::take(&mut tree.interactive_changed);
    if changed.is_empty() {
        return;
    }

    if index.sibling_interactive {
        recompute_all(tree, &index);
        return;
    }
    let affected = collect_affected(tree, &index, &changed);
    perf::time(Phase::Cascade, || {
        recompute_nodes(tree, &index, &affected)
    });
}

fn collect_affected(tree: &WidgetTree, index: &StyleIndex, changed: &[WidgetId]) -> Vec<WidgetId> {
    let mut seen: HashSet<WidgetId> = HashSet::new();
    let mut out = Vec::new();
    let mut stack: Vec<WidgetId> = changed.to_vec();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        out.push(id);
        if index.ancestor_interactive {
            for &child in &tree.nodes[id].children {
                stack.push(child);
            }
        }
    }
    out
}

fn recompute_all(tree: &mut WidgetTree, index: &StyleIndex) {
    let ids: Vec<WidgetId> = tree.nodes.keys().collect();
    recompute_nodes(tree, index, &ids);
}

fn recompute_nodes(tree: &mut WidgetTree, index: &StyleIndex, ids: &[WidgetId]) {
    let mut computed_map = HashMap::with_capacity(ids.len());
    {
        let mut caches = SelectorCaches::default();
        let mut candidates = Vec::new();
        let mut matched_keys: Vec<SelectorRef> = Vec::new();

        let mut shared: HashMap<Vec<SelectorRef>, ComputedStyle> = HashMap::new();
        for &id in ids {
            let element = ElementRef { tree, id };
            index.candidates(tree, id, &mut candidates);
            matched_keys.clear();
            for &(rule_idx, sel_idx) in &candidates {
                let selector = &index.rules[rule_idx as usize].selectors.slice()[sel_idx as usize];
                let mut ctx = MatchingContext::new(
                    MatchingMode::Normal,
                    None,
                    &mut caches,
                    QuirksMode::NoQuirks,
                    NeedsSelectorFlags::No,
                    MatchingForInvalidation::No,
                );
                if matches_selector(selector, 0, None, &element, &mut ctx) {
                    matched_keys.push((rule_idx, sel_idx));
                }
            }

            let has_inline = !tree.nodes[id].inline_style.is_empty();
            let cached = if has_inline {
                None
            } else {
                shared.get(&matched_keys).cloned()
            };
            let mut computed = match cached {
                Some(style) => style,
                None => {
                    let mut matched: Vec<(u32, u32, usize)> = Vec::new();
                    for &(rule_idx, sel_idx) in &matched_keys {
                        let rule = &index.rules[rule_idx as usize];
                        let specificity = rule.selectors.slice()[sel_idx as usize].specificity();
                        matched.extend(
                            (0..rule.declarations.len())
                                .map(|decl_idx| (specificity, rule_idx, decl_idx)),
                        );
                    }
                    matched.sort_by_key(|&(specificity, rule_idx, _)| (specificity, rule_idx));

                    let mut computed = ComputedStyle::default();
                    let resolved = index.resolved.borrow();
                    for (_, rule_idx, decl_idx) in matched {
                        let decl = &index.rules[rule_idx as usize].declarations[decl_idx];
                        apply_declaration(
                            &mut computed,
                            &decl.name,
                            &resolved.resolved[rule_idx as usize][decl_idx],
                        );
                    }
                    if has_inline {
                        for decl in &tree.nodes[id].inline_style {
                            apply_declaration(
                                &mut computed,
                                &decl.name,
                                &values::resolve_var(&decl.raw_value, &resolved.custom_properties),
                            );
                        }
                    } else {
                        drop(resolved);
                        shared.insert(matched_keys.clone(), computed.clone());
                    }
                    computed
                }
            };
            computed.is_hovered = tree.hovered_path.contains(&id);
            computed_map.insert(id, computed);
        }
    }

    for (id, computed) in computed_map {
        let taffy_style = computed.to_taffy();

        let text_changed = tree.nodes[id]
            .widget
            .apply_text_style(&computed.text_layout);
        let node = &mut tree.nodes[id];
        node.transitions.update(&node.style, &computed);
        node.animations
            .update(&computed.animations, &index.resolved.borrow().keyframes);
        node.style = computed;
        tree.set_layout_style(id, taffy_style);
        if text_changed {
            tree.mark_dirty(id);
        }
    }
}

pub(crate) fn custom_properties(tree: &WidgetTree) -> Vec<(String, String)> {
    let Some(index) = tree.rules.clone() else {
        return Vec::new();
    };
    let resolved = index.resolved.borrow();
    let mut out: Vec<(String, String)> = resolved
        .custom_properties
        .iter()
        .map(|(name, value)| {
            (
                format!("--{name}"),
                values::resolve_var(value, &resolved.custom_properties),
            )
        })
        .collect();
    out.sort();
    out
}

pub struct MatchedRule {
    pub selector: String,
    pub specificity: u32,
    pub declarations: Vec<MatchedDeclaration>,
}

pub struct MatchedDeclaration {
    pub name: String,

    pub value: String,

    pub resolved: String,

    pub overridden: bool,
}

pub(crate) fn matched_rules(tree: &WidgetTree, id: WidgetId) -> Vec<MatchedRule> {
    let Some(index) = tree.rules.clone() else {
        return Vec::new();
    };
    let element = ElementRef { tree, id };
    let mut candidates = Vec::new();
    index.candidates(tree, id, &mut candidates);
    let mut caches = SelectorCaches::default();
    let mut matched: Vec<(u32, u32, u32)> = Vec::new();
    for &(rule_idx, sel_idx) in &candidates {
        let selector = &index.rules[rule_idx as usize].selectors.slice()[sel_idx as usize];
        let mut ctx = MatchingContext::new(
            MatchingMode::Normal,
            None,
            &mut caches,
            QuirksMode::NoQuirks,
            NeedsSelectorFlags::No,
            MatchingForInvalidation::No,
        );
        if matches_selector(selector, 0, None, &element, &mut ctx) {
            matched.push((selector.specificity(), rule_idx, sel_idx));
        }
    }
    matched.sort_by_key(|&(specificity, rule_idx, sel_idx)| {
        cmp::Reverse((specificity, rule_idx, sel_idx))
    });

    let mut seen: HashSet<String> = HashSet::new();
    let mut out = Vec::new();
    let resolved = index.resolved.borrow();
    let node = &tree.nodes[id];
    if !node.inline_style.is_empty() {
        let declarations = node
            .inline_style
            .iter()
            .rev()
            .map(|decl| MatchedDeclaration {
                overridden: !seen.insert(decl.name.clone()),
                name: decl.name.clone(),
                value: decl.raw_value.clone(),
                resolved: values::resolve_var(&decl.raw_value, &resolved.custom_properties),
            })
            .collect::<Vec<_>>();
        out.push(MatchedRule {
            selector: "element.style".into(),
            specificity: u32::MAX,
            declarations: declarations.into_iter().rev().collect(),
        });
    }
    for (specificity, rule_idx, sel_idx) in matched {
        let rule = &index.rules[rule_idx as usize];
        let selector = rule.selectors.slice()[sel_idx as usize].to_css_string();
        let declarations = rule
            .declarations
            .iter()
            .enumerate()
            .rev()
            .map(|(i, decl)| MatchedDeclaration {
                overridden: !seen.insert(decl.name.clone()),
                name: decl.name.clone(),
                value: decl.raw_value.clone(),
                resolved: resolved.resolved[rule_idx as usize][i].clone(),
            })
            .collect::<Vec<_>>();
        out.push(MatchedRule {
            selector,
            specificity,
            declarations: declarations.into_iter().rev().collect(),
        });
    }
    out
}

fn collect_custom_properties(rules: &[StyleRule]) -> HashMap<String, String> {
    let mut props = HashMap::new();
    for rule in rules {
        let theme = rule.selectors.slice().iter().any(|selector| {
            selector.iter().any(|component| {
                matches!(component, Component::Class(name) if name.0.starts_with("theme-"))
            })
        });
        if theme {
            continue;
        }
        for decl in &rule.declarations {
            if let Some(name) = decl.name.strip_prefix("--") {
                props.insert(name.to_string(), decl.raw_value.clone());
            }
        }
    }
    props
}

pub fn custom_properties_for(rules: &[StyleRule], theme: &str) -> HashMap<String, String> {
    let mut props = collect_custom_properties(rules);
    let class = format!("theme-{theme}");
    for rule in rules {
        let matches = rule.selectors.slice().iter().any(|selector| {
            selector
                .iter()
                .any(|component| matches!(component, Component::Class(name) if name.0 == class))
        });
        if !matches {
            continue;
        }
        for decl in &rule.declarations {
            if let Some(name) = decl.name.strip_prefix("--") {
                props.insert(name.to_string(), decl.raw_value.clone());
            }
        }
    }
    props
}

pub(super) fn apply_declaration(computed: &mut ComputedStyle, name: &str, value: &str) {
    match name {
        "background-color" => {
            if let Some(color) = values::parse_color(value) {
                computed.background = Some(color);
            }
        }
        "color" => {
            if let Some(color) = values::parse_color(value) {
                computed.text_color = Some(color);
            }
        }
        "text-decoration" => {
            computed.text_underline = value.trim() == "underline";
        }
        "white-space" => {
            computed.text_layout.nowrap = matches!(value.trim(), "nowrap" | "pre");
        }
        "text-overflow" => {
            computed.text_layout.ellipsis = value.trim() == "ellipsis";
        }
        "-webkit-line-clamp" | "line-clamp" => {
            computed.text_layout.line_clamp = value.trim().parse::<u32>().ok().filter(|n| *n > 0);
        }
        "corner-shape" => {
            computed.squircle = value.trim() == "squircle";
        }
        "border-radius" => {
            let parts: Vec<&str> = super::calc::split_tokens(value);
            match parts.as_slice() {
                [single] if single.trim() == "50%" => computed.corner_ellipse = true,
                [single] => {
                    computed.corner_ellipse = false;
                    if let Some(px) = values::parse_length_px(single) {
                        computed.corner_radius = px;
                    }
                }
                [top_left, top_right, bottom_right, bottom_left] => {
                    computed.corner_radius_top_left = values::parse_length_px(top_left);
                    computed.corner_radius_top_right = values::parse_length_px(top_right);
                    computed.corner_radius_bottom_right = values::parse_length_px(bottom_right);
                    computed.corner_radius_bottom_left = values::parse_length_px(bottom_left);
                }
                _ => {}
            }
        }
        "display" => match value {
            "none" => computed.layout.display = Display::None,
            "block" => computed.layout.display = Display::Block,

            "inline" => {
                computed.layout.display = Display::Flex;
                computed.layout.flex_grow = 0.0;
                computed.layout.flex_shrink = 1.0;
                computed.layout.flex_basis = Dimension::auto();
            }
            "flex" => computed.layout.display = Display::Flex,

            "grid" => computed.layout.display = Display::Grid,
            _ => {}
        },
        "flex-direction" => match value {
            "row" => computed.layout.flex_direction = FlexDirection::Row,
            "column" => computed.layout.flex_direction = FlexDirection::Column,
            "row-reverse" => computed.layout.flex_direction = FlexDirection::RowReverse,
            "column-reverse" => computed.layout.flex_direction = FlexDirection::ColumnReverse,
            _ => {}
        },
        "grid-template-columns" => {
            if let Some(cols) = values::parse_grid_template_columns(value) {
                computed.layout.grid_template_columns = cols;
            }
        }
        "overflow" => {
            if let Some((x, y)) = parse_overflow_shorthand(value) {
                computed.layout.overflow.x = x;
                computed.layout.overflow.y = y;
            }
        }
        "overflow-y" => {
            if let Some(overflow) = parse_overflow(value) {
                computed.layout.overflow.y = overflow;
            }
        }
        "overflow-x" => {
            if let Some(overflow) = parse_overflow(value) {
                computed.layout.overflow.x = overflow;
            }
        }
        "width" => {
            if let Some(dim) = values::parse_dimension(value) {
                computed.layout.size.width = dim;
            }
        }
        "height" => {
            if let Some(dim) = values::parse_dimension(value) {
                computed.layout.size.height = dim;
            }
        }
        "max-width" => {
            if let Some(dim) = values::parse_dimension(value) {
                computed.layout.max_size.width = dim;
            }
        }
        "max-height" => {
            if let Some(dim) = values::parse_dimension(value) {
                computed.layout.max_size.height = dim;
            }
        }

        "min-width" => {
            if let Some(dim) = values::parse_dimension(value) {
                computed.layout.min_size.width = dim;
            }
        }
        "min-height" => {
            if let Some(dim) = values::parse_dimension(value) {
                computed.layout.min_size.height = dim;
            }
        }
        "padding" => {
            if let Some(padding) = values::parse_padding_shorthand(value) {
                computed.layout.padding = padding;
            }
        }

        "padding-left" => {
            if let Some(px) = values::parse_length_px(value) {
                computed.layout.padding.left = LengthPercentage::length(px);
            }
        }
        "padding-right" => {
            if let Some(px) = values::parse_length_px(value) {
                computed.layout.padding.right = LengthPercentage::length(px);
            }
        }
        "padding-top" => {
            if let Some(px) = values::parse_length_px(value) {
                computed.layout.padding.top = LengthPercentage::length(px);
            }
        }
        "padding-bottom" => {
            if let Some(px) = values::parse_length_px(value) {
                computed.layout.padding.bottom = LengthPercentage::length(px);
            }
        }

        "margin" => {
            if let Some(margin) = values::parse_margin_shorthand(value) {
                computed.layout.margin = margin;
            }
        }
        "margin-left" => {
            if let Some(m) = values::parse_margin_token(value) {
                computed.layout.margin.left = m;
            }
        }
        "margin-right" => {
            if let Some(m) = values::parse_margin_token(value) {
                computed.layout.margin.right = m;
            }
        }
        "margin-top" => {
            if let Some(m) = values::parse_margin_token(value) {
                computed.layout.margin.top = m;
            }
        }
        "margin-bottom" => {
            if let Some(m) = values::parse_margin_token(value) {
                computed.layout.margin.bottom = m;
            }
        }
        "gap" => {
            if let Some(px) = values::parse_length_px(value) {
                computed.layout.gap = TaffySize {
                    width: LengthPercentage::length(px),
                    height: LengthPercentage::length(px),
                };
            }
        }

        "column-gap" => {
            if let Some(px) = values::parse_length_px(value) {
                computed.layout.gap.width = LengthPercentage::length(px);
            }
        }
        "row-gap" => {
            if let Some(px) = values::parse_length_px(value) {
                computed.layout.gap.height = LengthPercentage::length(px);
            }
        }
        "flex-wrap" => {
            computed.layout.flex_wrap = match value.trim() {
                "wrap" => taffy::FlexWrap::Wrap,
                "wrap-reverse" => taffy::FlexWrap::WrapReverse,
                _ => taffy::FlexWrap::NoWrap,
            };
        }
        "pointer-events" => {
            computed.pointer_events_none = value.trim() == "none";
        }
        "cursor" => {
            computed.cursor = match value.trim() {
                "pointer" => Some(CursorIcon::Pointer),
                "text" => Some(CursorIcon::Text),
                "zoom-in" => Some(CursorIcon::ZoomIn),
                "zoom-out" => Some(CursorIcon::ZoomOut),
                "grab" => Some(CursorIcon::Grab),
                "grabbing" => Some(CursorIcon::Grabbing),
                "not-allowed" => Some(CursorIcon::NotAllowed),
                "default" => Some(CursorIcon::Default),
                _ => None,
            };
        }
        "box-shadow" => {
            computed.box_shadows = values::parse_box_shadow(value);
        }
        "filter" => {
            computed.filter = values::parse_filter(value);
        }
        "mix-blend-mode" => {
            // `normal` and anything unknown mean no blend mode.
            computed.blend_mode = super::BlendMode::parse(value);
        }
        "transform" => {
            if let Some(t) = super::transform::parse(value) {
                computed.transform = Some(t).filter(|t| !t.is_identity());
            }
        }
        "transform-origin" => {
            computed.transform_origin = super::transform::parse_origin(value);
        }
        "transition" => {
            computed.transitions = super::transition::parse(value);
        }
        "animation" => {
            computed.animations = super::animation::parse(value);
        }

        "scrollbar-width" => {
            if let Some((kind, px)) = match value.trim() {
                "thin" => Some((ScrollbarKind::Thin, 8.0)),
                "auto" => Some((ScrollbarKind::Auto, 16.0)),
                "none" => Some((ScrollbarKind::None, 0.0)),
                _ => None,
            } {
                computed.scrollbar.kind = kind;
                computed.layout.scrollbar_width = px;
            }
        }
        "scrollbar-fade" => {
            computed.scrollbar.fade = value.trim() == "hover";
        }
        "animation-name"
        | "animation-duration"
        | "animation-delay"
        | "animation-timing-function"
        | "animation-iteration-count"
        | "animation-direction"
        | "animation-fill-mode"
        | "animation-play-state" => {
            super::animation::apply_longhand(&mut computed.animations, name, value);
        }
        "opacity" => {
            if let Ok(opacity) = value.parse::<f32>() {
                computed.opacity = Some(opacity.clamp(0.0, 1.0)).filter(|o| *o < 1.0);
            }
        }

        "flex" => {
            if let Some((grow, shrink, basis)) = values::parse_flex_shorthand(value) {
                computed.layout.flex_grow = grow;
                computed.layout.flex_shrink = shrink;
                computed.layout.flex_basis = basis;
            }
        }
        "flex-basis" => {
            if let Some(basis) = values::parse_dimension(value) {
                computed.layout.flex_basis = basis;
            }
        }
        "flex-grow" => {
            if let Ok(grow) = value.parse::<f32>() {
                computed.layout.flex_grow = grow;
            }
        }
        "flex-shrink" => {
            if let Ok(shrink) = value.parse::<f32>() {
                computed.layout.flex_shrink = shrink;
            }
        }
        "align-items" => {
            if let Some(align) = parse_align_items(value) {
                computed.layout.align_items = Some(align);
            }
        }
        "align-self" => {
            if let Some(align) = parse_align_items(value) {
                computed.layout.align_self = Some(align);
            }
        }
        "justify-content" => {
            if let Some(justify) = parse_justify_content(value) {
                computed.layout.justify_content = Some(justify);
            }
        }
        "position" => match value {
            "relative" => computed.layout.position = Position::Relative,
            "absolute" => computed.layout.position = Position::Absolute,
            _ => {}
        },
        "inset" => {
            if let Some(px) = values::parse_inset_shorthand(value) {
                let side = LengthPercentageAuto::length(px);
                computed.layout.inset = TaffyRect {
                    top: side,
                    right: side,
                    bottom: side,
                    left: side,
                };
            }
        }

        "top" => {
            if let Some(inset) = values::parse_inset_token(value) {
                computed.layout.inset.top = inset;
            }
        }
        "right" => {
            if let Some(inset) = values::parse_inset_token(value) {
                computed.layout.inset.right = inset;
            }
        }
        "bottom" => {
            if let Some(inset) = values::parse_inset_token(value) {
                computed.layout.inset.bottom = inset;
            }
        }
        "left" => {
            if let Some(inset) = values::parse_inset_token(value) {
                computed.layout.inset.left = inset;
            }
        }
        "border" => {
            if let Some((width, color)) = values::parse_border_shorthand(value) {
                computed.border_width = width;
                computed.border_color = Some(color);

                let side = LengthPercentage::length(width);
                computed.layout.border = taffy::Rect {
                    left: side,
                    right: side,
                    top: side,
                    bottom: side,
                };
                computed.border_dashed = value.split_whitespace().any(|token| token == "dashed");
            }
        }
        "border-bottom" => {
            if let Some((width, color)) = values::parse_border_shorthand(value) {
                computed.border_bottom_width = width;
                computed.border_bottom_color = Some(color);
                computed.layout.border.bottom = LengthPercentage::length(width);
            }
        }
        "border-top" => {
            if let Some((width, color)) = values::parse_border_shorthand(value) {
                computed.border_top_width = width;
                computed.border_top_color = Some(color);
                computed.layout.border.top = LengthPercentage::length(width);
            }
        }
        "border-left" => {
            if let Some((width, color)) = values::parse_border_shorthand(value) {
                computed.border_left_width = width;
                computed.border_left_color = Some(color);
                computed.layout.border.left = LengthPercentage::length(width);
            }
        }

        "border-right" => {
            if let Some((width, color)) = values::parse_border_shorthand(value) {
                computed.border_right_width = width;
                computed.border_right_color = Some(color);
                computed.layout.border.right = LengthPercentage::length(width);
            }
        }
        "border-top-left-radius" => {
            if let Some(px) = values::parse_length_px(value) {
                computed.corner_radius_top_left = Some(px);
            }
        }

        "border-top-right-radius" => {
            if let Some(px) = values::parse_length_px(value) {
                computed.corner_radius_top_right = Some(px);
            }
        }
        "border-bottom-left-radius" => {
            if let Some(px) = values::parse_length_px(value) {
                computed.corner_radius_bottom_left = Some(px);
            }
        }
        "border-bottom-right-radius" => {
            if let Some(px) = values::parse_length_px(value) {
                computed.corner_radius_bottom_right = Some(px);
            }
        }
        "outline-color" => {
            if let Some(color) = values::parse_color(value) {
                computed.outline_color = Some(color);
            }
        }
        "outline-width" => {
            if let Some(px) = values::parse_length_px(value) {
                computed.outline_width = Some(px);
            }
        }

        _ => {}
    }
}

/// `overflow: <x> [<y>]`: one value sets both axes, two set x then y. A value the engine
/// doesn't know drops the whole declaration, as in CSS.
fn parse_overflow_shorthand(value: &str) -> Option<(Overflow, Overflow)> {
    let mut parts = value.split_whitespace();
    let x = parse_overflow(parts.next()?)?;
    let y = match parts.next() {
        Some(part) => parse_overflow(part)?,
        None => x,
    };
    parts.next().is_none().then_some((x, y))
}

fn parse_overflow(value: &str) -> Option<Overflow> {
    match value {
        "visible" => Some(Overflow::Visible),
        "clip" => Some(Overflow::Clip),
        "hidden" => Some(Overflow::Hidden),
        "scroll" => Some(Overflow::Scroll),
        _ => None,
    }
}

fn parse_align_items(value: &str) -> Option<AlignItems> {
    match value {
        "flex-start" => Some(AlignItems::FLEX_START),
        "flex-end" => Some(AlignItems::FLEX_END),
        "start" => Some(AlignItems::START),
        "end" => Some(AlignItems::END),
        "center" => Some(AlignItems::CENTER),
        "baseline" => Some(AlignItems::BASELINE),
        "stretch" => Some(AlignItems::STRETCH),
        _ => None,
    }
}

fn parse_justify_content(value: &str) -> Option<JustifyContent> {
    match value {
        "flex-start" => Some(JustifyContent::FLEX_START),
        "flex-end" => Some(JustifyContent::FLEX_END),
        "start" => Some(JustifyContent::START),
        "end" => Some(JustifyContent::END),
        "center" => Some(JustifyContent::CENTER),
        "stretch" => Some(JustifyContent::STRETCH),
        "space-between" => Some(JustifyContent::SPACE_BETWEEN),
        "space-around" => Some(JustifyContent::SPACE_AROUND),
        "space-evenly" => Some(JustifyContent::SPACE_EVENLY),
        _ => None,
    }
}

#[cfg(test)]
mod hover_incremental_tests {
    use crate::style::parser;
    use super::*;
    use crate::widget::Widget;

    struct Plain;
    impl Widget for Plain {}

    #[test]
    fn hover_recomputes_the_changed_node_and_its_descendants() {
        let mut tree = WidgetTree::new(Box::new(Plain), ComputedStyle::default());
        let root = tree.root;
        let row = tree.add_child(root, "div", Box::new(Plain), ComputedStyle::default());
        let child = tree.add_child(row, "div", Box::new(Plain), ComputedStyle::default());
        tree.nodes[row].classes.push("row".to_string());
        tree.nodes[child].classes.push("child".to_string());

        let sheet = parser::parse_stylesheet(
            ".row:hover { background-color: #ff0000; } .row:hover .child { color: #00ff00; }",
        );
        apply_stylesheet(&mut tree, &sheet);

        assert_eq!(tree.nodes[row].style.background, None);
        assert_eq!(tree.nodes[child].style.text_color, None);

        tree.hovered_path.insert(row);
        tree.interactive_changed.push(row);
        recompute_interactive(&mut tree);

        assert_eq!(
            tree.nodes[row].style.background,
            Some(skia_safe::Color::from_rgb(255, 0, 0))
        );

        assert_eq!(
            tree.nodes[child].style.text_color,
            Some(skia_safe::Color::from_rgb(0, 255, 0))
        );
    }
}

#[cfg(test)]
mod overflow_tests {
    use super::*;

    #[test]
    fn the_shorthand_sets_both_axes_or_each_in_turn() {
        assert_eq!(
            parse_overflow_shorthand("hidden"),
            Some((Overflow::Hidden, Overflow::Hidden))
        );
        assert_eq!(
            parse_overflow_shorthand("hidden scroll"),
            Some((Overflow::Hidden, Overflow::Scroll))
        );
        assert_eq!(parse_overflow_shorthand("hidden nonsense"), None);
        assert_eq!(parse_overflow_shorthand("hidden hidden hidden"), None);
        assert_eq!(parse_overflow_shorthand(""), None);
    }

    #[test]
    fn overflow_hidden_clips_like_its_longhands() {
        let mut shorthand = ComputedStyle::default();
        apply_declaration(&mut shorthand, "overflow", "hidden");
        let mut longhands = ComputedStyle::default();
        apply_declaration(&mut longhands, "overflow-x", "hidden");
        apply_declaration(&mut longhands, "overflow-y", "hidden");
        assert_eq!(shorthand.layout.overflow, longhands.layout.overflow);
        assert_eq!(shorthand.layout.overflow.x, Overflow::Hidden);
        assert_eq!(shorthand.layout.overflow.y, Overflow::Hidden);
    }
}

#[cfg(test)]
mod justify_content_tests {
    use super::*;

    #[test]
    fn recognizes_every_real_justify_content_value() {
        assert_eq!(
            parse_justify_content("space-between"),
            Some(JustifyContent::SPACE_BETWEEN)
        );
        assert_eq!(
            parse_justify_content("space-around"),
            Some(JustifyContent::SPACE_AROUND)
        );
        assert_eq!(
            parse_justify_content("space-evenly"),
            Some(JustifyContent::SPACE_EVENLY)
        );
        assert_eq!(parse_justify_content("start"), Some(JustifyContent::START));
        assert_eq!(parse_justify_content("end"), Some(JustifyContent::END));
        assert_eq!(
            parse_justify_content("flex-start"),
            Some(JustifyContent::FLEX_START)
        );
        assert_eq!(
            parse_justify_content("flex-end"),
            Some(JustifyContent::FLEX_END)
        );
        assert_eq!(
            parse_justify_content("center"),
            Some(JustifyContent::CENTER)
        );
        assert_eq!(
            parse_justify_content("stretch"),
            Some(JustifyContent::STRETCH)
        );
        assert_eq!(parse_justify_content("not-a-real-value"), None);
    }

    #[test]
    fn align_items_recognizes_start_and_end_too() {
        assert_eq!(parse_align_items("start"), Some(AlignItems::START));
        assert_eq!(parse_align_items("end"), Some(AlignItems::END));
    }
}

#[cfg(test)]
mod theme_token_tests {
    use crate::style::parser;
    use crate::widget::primitives;
    use super::*;

    #[test]
    fn the_selected_theme_overrides_the_default_tokens() {
        let rules = parser::parse_stylesheet(
            ".theme-dark { --background-primary: #111111; } .theme-light { --background-primary: #eeeeee; }",
        );
        crate::set_theme_name("light");
        let light = custom_properties_for(&rules, &crate::theme_name());
        crate::set_theme_name("dark");
        let dark = custom_properties_for(&rules, &crate::theme_name());
        assert_eq!(
            light.get("background-primary").map(String::as_str),
            Some("#eeeeee")
        );
        assert_eq!(
            dark.get("background-primary").map(String::as_str),
            Some("#111111")
        );
    }

    #[test]
    fn switching_theme_rebinds_the_index_without_reparsing() {
        crate::set_theme_name("dark");
        let rules = parser::parse_stylesheet(
            ".box { background-color: var(--background-primary); } .theme-dark { --background-primary: #111111; } .theme-light { --background-primary: #eeeeee; }",
        );
        let mut tree = WidgetTree::new(
            Box::new(primitives::BoxWidget),
            ComputedStyle::default(),
        );
        let root = tree.root;
        tree.nodes[root].classes.push("box".into());
        apply_stylesheet(&mut tree, &rules);
        assert_eq!(
            tree.nodes[root].style.background,
            Some(skia_safe::Color::from_rgb(0x11, 0x11, 0x11))
        );
        crate::set_theme_name("light");
        assert!(refresh_theme(&mut tree));
        recompute(&mut tree);
        assert_eq!(
            tree.nodes[root].style.background,
            Some(skia_safe::Color::from_rgb(0xee, 0xee, 0xee))
        );
        assert!(!refresh_theme(&mut tree));
        assert_eq!(
            crate::theme_color("background-primary", skia_safe::Color::BLACK),
            skia_safe::Color::from_rgb(0xee, 0xee, 0xee)
        );
    }
}
