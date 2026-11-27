use crate::paint;
use crate::perf;
use crate::reveal;
use crate::style::calc;
use slotmap::{Key, KeyData};
use std::slice;
use std::time::Instant;
use taffy::{
    AvailableSpace, BlockContext, CacheTree, Display, Layout, LayoutBlockContainer,
    LayoutFlexboxContainer, LayoutGridContainer, LayoutInput, LayoutOutput, LayoutPartialTree,
    NodeId, ResolveOrZero, RoundTree, RunMode, Size, Style, TraversePartialTree, TraverseTree,
    compute_block_layout, compute_cached_layout, compute_flexbox_layout, compute_grid_layout,
    compute_hidden_layout, compute_leaf_layout, compute_root_layout, round_layout,
};

use crate::widget::{WidgetId, WidgetTree};

pub(crate) fn node_id(id: WidgetId) -> NodeId {
    NodeId::from(id.data().as_ffi())
}

pub(crate) fn widget_id(node: NodeId) -> WidgetId {
    KeyData::from_ffi(u64::from(node)).into()
}

pub fn compute_layout(tree: &mut WidgetTree, available: Size<AvailableSpace>) {
    let space_key = |space: AvailableSpace| match space {
        AvailableSpace::Definite(px) => px.to_bits(),
        AvailableSpace::MinContent => u32::MAX - 1,
        AvailableSpace::MaxContent => u32::MAX,
    };
    let space = (space_key(available.width), space_key(available.height));
    if !tree.layout_dirty && tree.last_layout_space == Some(space) {
        return;
    }
    tree.layout_dirty = false;
    tree.last_layout_space = Some(space);
    let root = node_id(tree.root);
    compute_root_layout(tree, root, available);
    round_layout(tree, root);
}

pub fn publish_observed_sizes(tree: &mut WidgetTree) -> bool {
    let mut changed = reveal_requested(tree);
    for (id, node) in tree.nodes.iter() {
        if let Some(observer) = &node.size_observer {
            let size = (node.layout.size.width, node.layout.size.height);
            if observer.peek() != size {
                observer.set(size);
                changed = true;
            }
        }

        if let Some(observer) = &node.rect_observer {
            let rect = paint::absolute_rect(tree, id);
            if observer.peek() != rect {
                observer.set(rect);
                changed = true;
            }
        }
    }
    changed
}

fn reveal_requested(tree: &mut WidgetTree) -> bool {
    let pending: Vec<(WidgetId, f32)> = tree
        .nodes
        .iter()
        .filter_map(|(id, node)| {
            node.reveal
                .as_ref()
                .map(|(key, padding)| (id, key.clone(), *padding))
        })
        .filter(|(_, key, _)| reveal::take(key))
        .map(|(id, _, padding)| (id, padding))
        .collect();
    let mut moved = false;
    for (id, padding) in pending {
        let mut scroller = tree.nodes[id].parent;
        while let Some(parent) = scroller {
            if tree.nodes[parent].widget.scroll_offset().is_some() {
                break;
            }
            scroller = tree.nodes[parent].parent;
        }
        let Some(scroller) = scroller else { continue };
        let view = paint::absolute_rect(tree, scroller);
        let target = paint::absolute_rect(tree, id);
        let offset = tree.nodes[scroller]
            .widget
            .scroll_offset()
            .map_or(0.0, |(_, dy)| dy);
        let wanted = reveal::nearest_offset(
            offset,
            target.y - view.y,
            target.height,
            view.height,
            padding,
        );
        if wanted != offset {
            tree.nodes[scroller].widget.scroll_to_offset(wanted);
            moved = true;
        }
    }
    moved
}

impl WidgetTree {
    fn style_of(&self, node: NodeId) -> &Style {
        &self.nodes[widget_id(node)].layout_style
    }

    fn compute_node(
        &mut self,
        node: NodeId,
        inputs: LayoutInput,
        block_ctx: Option<&mut BlockContext<'_>>,
    ) -> LayoutOutput {
        if inputs.run_mode == RunMode::PerformHiddenLayout {
            return compute_hidden_layout(self, node);
        }
        compute_cached_layout(self, node, inputs, |tree, node, inputs| {
            let id = widget_id(node);
            let display = tree.nodes[id].layout_style.display;
            let has_children = !tree.nodes[id].children.is_empty();
            match (display, has_children) {
                (Display::None, _) => compute_hidden_layout(tree, node),
                (Display::Block, true) => compute_block_layout(tree, node, inputs, block_ctx),
                (Display::Flex, true) => compute_flexbox_layout(tree, node, inputs),
                (Display::Grid, true) => compute_grid_layout(tree, node, inputs),
                (_, false) => {
                    let widget_node = &tree.nodes[id];
                    let style = &widget_node.layout_style;
                    let widget = &widget_node.widget;
                    let profile = perf::profile_paint().then(|| {
                        format!(
                            "MEASURE {}.{}",
                            widget_node.tag,
                            widget_node.classes.first().map_or("", String::as_str)
                        )
                    });
                    let mut output = compute_leaf_layout(
                        inputs,
                        style,
                        calc::resolve,
                        |known, available| {
                            let started = Instant::now();
                            let size = widget.measure(known, available);
                            if let Some(name) = &profile {
                                perf::record_widget_paint(
                                    name.clone(),
                                    started.elapsed().as_secs_f64() * 1000.0,
                                );
                            }
                            size
                        },
                    );

                    let padding = style
                        .padding
                        .resolve_or_zero(inputs.parent_size.width, calc::resolve);
                    let border = style
                        .border
                        .resolve_or_zero(inputs.parent_size.width, calc::resolve);
                    let content_width = (output.size.width
                        - padding.left
                        - padding.right
                        - border.left
                        - border.right)
                        .max(0.0);
                    if let Some(baseline) = widget.first_baseline(content_width) {
                        output.first_baselines.y = Some(padding.top + border.top + baseline);
                    }
                    output
                }
            }
        })
    }
}

pub struct ChildIds<'a>(slice::Iter<'a, WidgetId>);

impl Iterator for ChildIds<'_> {
    type Item = NodeId;
    fn next(&mut self) -> Option<NodeId> {
        self.0.next().map(|&id| node_id(id))
    }
}

impl TraversePartialTree for WidgetTree {
    type ChildIter<'a>
        = ChildIds<'a>
    where
        Self: 'a;

    fn child_ids(&self, parent: NodeId) -> Self::ChildIter<'_> {
        ChildIds(self.nodes[widget_id(parent)].children.iter())
    }

    fn child_count(&self, parent: NodeId) -> usize {
        self.nodes[widget_id(parent)].children.len()
    }

    fn get_child_id(&self, parent: NodeId, index: usize) -> NodeId {
        node_id(self.nodes[widget_id(parent)].children[index])
    }
}

impl TraverseTree for WidgetTree {}

impl LayoutPartialTree for WidgetTree {
    type CoreContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;

    type CustomIdent = String;

    fn get_core_container_style(&self, node: NodeId) -> Self::CoreContainerStyle<'_> {
        self.style_of(node)
    }

    fn set_unrounded_layout(&mut self, node: NodeId, layout: &Layout) {
        self.nodes[widget_id(node)].unrounded_layout = *layout;
    }

    fn resolve_calc_value(&self, val: *const (), basis: f32) -> f32 {
        calc::resolve(val, basis)
    }

    fn compute_child_layout(&mut self, node: NodeId, inputs: LayoutInput) -> LayoutOutput {
        self.compute_node(node, inputs, None)
    }
}

impl CacheTree for WidgetTree {
    fn cache_get(&self, node: NodeId, input: &LayoutInput) -> Option<LayoutOutput> {
        self.nodes[widget_id(node)].layout_cache.get(input)
    }

    fn cache_store(&mut self, node: NodeId, input: &LayoutInput, output: LayoutOutput) {
        self.nodes[widget_id(node)]
            .layout_cache
            .store(input, output)
    }

    fn cache_clear(&mut self, node: NodeId) {
        self.nodes[widget_id(node)].layout_cache.clear();
    }
}

impl LayoutFlexboxContainer for WidgetTree {
    type FlexboxContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;
    type FlexboxItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    fn get_flexbox_container_style(&self, node: NodeId) -> Self::FlexboxContainerStyle<'_> {
        self.style_of(node)
    }

    fn get_flexbox_child_style(&self, child: NodeId) -> Self::FlexboxItemStyle<'_> {
        self.style_of(child)
    }
}

impl LayoutBlockContainer for WidgetTree {
    type BlockContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;
    type BlockItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    fn get_block_container_style(&self, node: NodeId) -> Self::BlockContainerStyle<'_> {
        self.style_of(node)
    }

    fn get_block_child_style(&self, child: NodeId) -> Self::BlockItemStyle<'_> {
        self.style_of(child)
    }

    fn compute_block_child_layout(
        &mut self,
        node: NodeId,
        inputs: LayoutInput,
        block_ctx: Option<&mut BlockContext<'_>>,
    ) -> LayoutOutput {
        self.compute_node(node, inputs, block_ctx)
    }
}

impl LayoutGridContainer for WidgetTree {
    type GridContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;
    type GridItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    fn get_grid_container_style(&self, node: NodeId) -> Self::GridContainerStyle<'_> {
        self.style_of(node)
    }

    fn get_grid_child_style(&self, child: NodeId) -> Self::GridItemStyle<'_> {
        self.style_of(child)
    }
}

impl RoundTree for WidgetTree {
    fn get_unrounded_layout(&self, node: NodeId) -> Layout {
        self.nodes[widget_id(node)].unrounded_layout
    }

    fn set_final_layout(&mut self, node: NodeId, layout: &Layout) {
        self.nodes[widget_id(node)].layout = *layout;
    }
}

#[cfg(test)]
mod tests {
    use crate::font_system;
    use std::cell::RefCell;
    use std::rc::Rc;

    use skia_safe::Color;
    use skia_safe::font_style::Weight;
    use taffy::{AlignItems, FlexDirection};

    use super::*;
    use crate::glyph_cache::GlyphCache;
    use crate::style::ComputedStyle;
    use crate::widget::primitives::{BoxWidget, TextWidget};

    #[test]
    fn baseline_alignment_uses_text_baselines() {
        let font_system = Rc::new(RefCell::new(font_system::make_font_system()));
        let glyph_cache = Rc::new(GlyphCache::new(1024 * 1024));
        let text = |content: &str, size: f32, line_height: f32| {
            TextWidget::new_weighted(
                content,
                size,
                Color::WHITE,
                font_system.clone(),
                glyph_cache.clone(),
                Weight::MEDIUM,
            )
            .with_line_height(line_height)
        };

        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let mut row_style = ComputedStyle::default();
        row_style.layout.flex_direction = FlexDirection::Row;
        row_style.layout.align_items = Some(AlignItems::BASELINE);
        let row = tree.add_child(tree.root, "div", Box::new(BoxWidget), row_style);
        let name = tree.add_child(
            row,
            "span",
            Box::new(text("Alice", 16.0, 22.0)),
            ComputedStyle::default(),
        );

        let time = tree.add_child(
            row,
            "span",
            Box::new(text("2:26 AM", 12.0, 14.0)),
            ComputedStyle::default(),
        );

        compute_layout(
            &mut tree,
            Size {
                width: AvailableSpace::Definite(400.0),
                height: AvailableSpace::Definite(100.0),
            },
        );

        let baseline = |id: WidgetId| {
            let widget = &tree.nodes[id].widget;
            tree.layout(id).location.y
                + widget
                    .first_baseline(tree.layout(id).size.width)
                    .expect("text has a baseline")
        };
        assert!(
            (baseline(name) - baseline(time)).abs() <= 1.0,
            "name baseline {} vs timestamp baseline {}",
            baseline(name),
            baseline(time)
        );
    }
}

#[cfg(test)]
mod overflow_tests {
    use crate::font_system;
    use std::cell::RefCell;
    use std::rc::Rc;

    use skia_safe::Color;
    use taffy::{Dimension, FlexDirection, Overflow, Point};

    use super::*;
    use crate::glyph_cache::GlyphCache;
    use crate::style::{ComputedStyle, TextLayoutStyle};
    use crate::widget::Widget;
    use crate::widget::primitives::{BoxWidget, TextWidget};

    #[test]
    fn overflow_hidden_text_shrinks_in_a_narrow_row() {
        let font_system = Rc::new(RefCell::new(font_system::make_font_system()));
        let glyph_cache = Rc::new(GlyphCache::new(1024 * 1024));
        let mut text = TextWidget::new(
            "a very long role name that will never fit in a narrow chip",
            12.0,
            Color::WHITE,
            font_system,
            glyph_cache,
        );
        text.apply_text_style(&TextLayoutStyle {
            nowrap: true,
            ellipsis: true,
            line_clamp: None,
        });

        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let mut row = ComputedStyle::default();
        row.layout.flex_direction = FlexDirection::Row;
        row.layout.size.width = Dimension::length(120.0);
        let row = tree.add_child(tree.root, "div", Box::new(BoxWidget), row);
        let mut name = ComputedStyle::default();
        name.layout.overflow = Point {
            x: Overflow::Hidden,
            y: Overflow::Hidden,
        };
        let name = tree.add_child(row, "span", Box::new(text), name);

        compute_layout(
            &mut tree,
            Size {
                width: AvailableSpace::Definite(400.0),
                height: AvailableSpace::Definite(100.0),
            },
        );
        assert!(
            tree.layout(name).size.width <= 120.0,
            "name is {}px in a 120px row",
            tree.layout(name).size.width
        );
    }
}

#[cfg(test)]
mod calc_tests {
    use taffy::{Dimension, FlexDirection};

    use super::*;
    use crate::style::ComputedStyle;
    use crate::style::values::{parse_dimension, parse_margin_token};
    use crate::widget::primitives::BoxWidget;

    #[test]
    fn calc_with_percentages_resolves_during_layout() {
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let mut row = ComputedStyle::default();
        row.layout.flex_direction = FlexDirection::Row;
        row.layout.size.width = Dimension::length(200.0);
        let row = tree.add_child(tree.root, "div", Box::new(BoxWidget), row);
        let mut child = ComputedStyle::default();
        child.layout.size.width = parse_dimension("calc(100% - 50px)").expect("calc width");
        child.layout.size.height = Dimension::length(10.0);
        child.layout.margin.left = parse_margin_token("calc(10% + 2px)").expect("calc margin");
        let child = tree.add_child(row, "div", Box::new(BoxWidget), child);

        compute_layout(
            &mut tree,
            Size {
                width: AvailableSpace::Definite(400.0),
                height: AvailableSpace::Definite(100.0),
            },
        );
        assert_eq!(tree.layout(child).size.width, 150.0);
        assert_eq!(tree.layout(child).location.x, 22.0);
    }
}

#[cfg(test)]
mod wrap_edge_tests {
    use crate::font_system;
    use crate::widget::primitives;
    use skia_safe::font_style::Weight as Font_styleWeight;
    use skia_safe::surfaces;
    use std::cell::RefCell;
    use std::collections::BTreeSet;
    use std::rc::Rc;

    use skia_safe::Color;
    use taffy::{Dimension, FlexDirection, LengthPercentage};

    use super::*;
    use crate::glyph_cache::GlyphCache;
    use crate::style::ComputedStyle;
    use crate::widget::primitives::{BoxWidget, TextWidget};

    #[test]
    fn text_beside_a_button_paints_the_lines_layout_reserved_room_for() {
        let font_system = Rc::new(RefCell::new(font_system::make_font_system()));
        let glyph_cache = Rc::new(GlyphCache::new(1024 * 1024));
        let body_text = "Please check your email and follow the instructions to verify your email. If you did not receive an email or if it expired, you can resend one.";
        let body = TextWidget::new_weighted(
            body_text,
            14.0,
            Color::WHITE,
            font_system,
            glyph_cache,
            Font_styleWeight::MEDIUM,
        );

        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let mut row = ComputedStyle::default();
        row.layout.flex_direction = FlexDirection::Row;
        row.layout.size.width = Dimension::percent(1.0);
        row.layout.gap = Size {
            width: LengthPercentage::length(8.3),
            height: LengthPercentage::length(0.0),
        };
        let row = tree.add_child(tree.root, "div", Box::new(BoxWidget), row);
        let mut icon = ComputedStyle::default();
        icon.layout.size = Size {
            width: Dimension::length(24.0),
            height: Dimension::length(24.0),
        };
        icon.layout.flex_shrink = 0.0;
        tree.add_child(row, "div", Box::new(BoxWidget), icon);
        let mut column = ComputedStyle::default();
        column.layout.flex_direction = FlexDirection::Column;
        column.layout.flex_grow = 1.0;
        column.layout.flex_shrink = 1.0;
        let column = tree.add_child(row, "div", Box::new(BoxWidget), column);
        let body = tree.add_child(column, "span", Box::new(body), ComputedStyle::default());
        let mut button = ComputedStyle::default();
        button.layout.size = Size {
            width: Dimension::length(171.7),
            height: Dimension::length(32.0),
        };
        button.layout.flex_shrink = 0.0;
        tree.add_child(row, "div", Box::new(BoxWidget), button);

        let mut mismatches = Vec::new();
        let mut heights = BTreeSet::new();
        for step in 0..=3200 {
            let width = 500.0 + step as f32 * 0.25;
            let mut root_style = tree.nodes[tree.root].layout_style.clone();
            root_style.size.width = Dimension::length(width);
            tree.set_layout_style(tree.root, root_style);
            compute_layout(
                &mut tree,
                Size {
                    width: AvailableSpace::Definite(width),
                    height: AvailableSpace::Definite(400.0),
                },
            );
            let laid_out = tree.layout(body).size;
            heights.insert(laid_out.height as i32);

            let mut surface =
                surfaces::raster_n32_premul((1400, 400)).expect("raster surface");
            paint::paint(&mut tree, surface.canvas());
            let painted_lines = primitives::painted_lines_of(
                "Please check your email and follow the",
            )
            .expect("body painted")
            .0;
            let reserved_lines = (laid_out.height / (14.0 * 1.2)).round() as usize;
            if painted_lines != reserved_lines {
                mismatches.push((width, laid_out.width, reserved_lines, painted_lines));
            }
        }
        assert!(
            heights.len() >= 2,
            "the sweep must cross the wrapping point, heights seen: {heights:?}"
        );
        assert!(
            mismatches.is_empty(),
            "(window width, text box width, reserved lines, painted lines): {mismatches:?}"
        );
    }
}
