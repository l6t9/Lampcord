use crate::app;
use crate::perf;
use crate::selection;
use crate::style as crate_style;
use crate::widget::primitives;
use crate::widget;
use skia_safe::color_filters;
use skia_safe::image_filters;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::mem;
use std::time::Instant;
use taffy::Overflow;

use skia_safe::{Canvas, Color, Paint, PaintStyle, RRect, Rect};

use crate::event::{CursorIcon, EventResult, MouseButton, NamedKey, UiEvent};
use crate::widget::{AbsoluteRect, WidgetId, WidgetTree};

const DEFAULT_FOCUS_RING_COLOR: Color = Color::from_argb(255, 0x00, 0xb0, 0xf4);
const DEFAULT_FOCUS_RING_WIDTH: f32 = 4.0;

thread_local! {

    static ANIMATION: RefCell<Animation> = RefCell::new(Animation::default());
    static REPAINT_REQUESTED: Cell<bool> = const { Cell::new(false) };

    static DAMAGE_REQUESTED: RefCell<Vec<AbsoluteRect>> = const { RefCell::new(Vec::new()) };

    static LAYOUT_CLIP: Cell<Option<skia_safe::IRect>> = const { Cell::new(None) };

    static FOCUS_RING: Cell<Option<FocusRing>> = const { Cell::new(None) };
}

fn push_layout_clip(canvas: &Canvas, bounds: Rect) -> Option<skia_safe::IRect> {
    let device = to_device(canvas, bounds);
    LAYOUT_CLIP.with(|clip| {
        let previous = clip.get();
        let narrowed = match previous {
            Some(outer) => skia_safe::IRect::intersect(&outer, &device).unwrap_or_default(),
            None => device,
        };
        clip.set(Some(narrowed));
        previous
    })
}

fn pop_layout_clip(previous: Option<skia_safe::IRect>) {
    LAYOUT_CLIP.with(|clip| clip.set(previous));
}

fn to_device(canvas: &Canvas, rect: Rect) -> skia_safe::IRect {
    let (mapped, _) = canvas.local_to_device_as_3x3().map_rect(rect);
    skia_safe::IRect::from_ltrb(
        mapped.left.floor() as i32,
        mapped.top.floor() as i32,
        mapped.right.ceil() as i32,
        mapped.bottom.ceil() as i32,
    )
}

#[derive(Default)]
pub struct Animation {
    pub full: bool,

    pub regions: Vec<(skia_safe::IRect, Instant)>,

    pub wake_regions: Vec<skia_safe::IRect>,
}

pub fn request_repaint() {
    REPAINT_REQUESTED.with(|c| c.set(true));
}

pub fn take_repaint_request() -> bool {
    REPAINT_REQUESTED.with(|c| c.replace(false))
}

pub fn request_repaint_rect(rect: AbsoluteRect) {
    DAMAGE_REQUESTED.with(|d| d.borrow_mut().push(rect));
}

pub fn take_damage_requests() -> Vec<AbsoluteRect> {
    DAMAGE_REQUESTED.with(|d| mem::take(&mut *d.borrow_mut()))
}

pub fn keep_animating() {
    ANIMATION.with(|a| a.borrow_mut().full = true);
}

pub fn animate_region(canvas: &Canvas, rect: Rect) {
    animate_region_at(canvas, rect, Instant::now());
}

pub fn animate_region_at(canvas: &Canvas, rect: Rect, at: Instant) {
    if let Some(device) = visible_device_rect(canvas, rect) {
        ANIMATION.with(|a| a.borrow_mut().regions.push((device, at)));
    }
}

pub fn animate_region_on_wake(canvas: &Canvas, rect: Rect) {
    if let Some(device) = visible_device_rect(canvas, rect) {
        ANIMATION.with(|a| a.borrow_mut().wake_regions.push(device));
    }
}

fn visible_device_rect(canvas: &Canvas, rect: Rect) -> Option<skia_safe::IRect> {
    let mut device = to_device(canvas, rect);

    device.outset((2, 2));
    match LAYOUT_CLIP.with(Cell::get) {
        Some(clip) => skia_safe::IRect::intersect(&device, &clip),
        None => Some(device),
    }
}

pub fn take_animation() -> Animation {
    ANIMATION.with(|a| mem::take(&mut *a.borrow_mut()))
}

pub fn paint(tree: &mut WidgetTree, canvas: &Canvas) {
    let mut deferred = Vec::new();
    FOCUS_RING.with(|ring| ring.set(None));
    paint_node(
        tree,
        tree.root,
        0.0,
        app::current_top_inset(),
        canvas,
        &mut deferred,
    );

    let mut next = 0;
    while next < deferred.len() {
        let (id, x, y) = deferred[next];
        next += 1;
        paint_node(tree, id, x, y, canvas, &mut deferred);
    }

    if let Some(ring) = FOCUS_RING.with(Cell::take) {
        paint_focus_ring(canvas, ring);
    }
}

fn paint_box_shadows(canvas: &Canvas, rect: AbsoluteRect, style: &crate_style::ComputedStyle) {
    let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
    let radii = primitives::corner_radii(style);
    let rounded = primitives::has_any_corner_radius(style);
    let box_rrect = if rounded {
        RRect::new_rect_radii(bounds, &radii)
    } else {
        RRect::new_rect(bounds)
    };
    canvas.save();
    canvas.clip_rrect(box_rrect, skia_safe::ClipOp::Difference, true);
    for shadow in style
        .box_shadows
        .iter()
        .rev()
        .filter(|shadow| !shadow.inset)
    {
        let spread_bounds = bounds
            .with_offset((shadow.x, shadow.y))
            .with_outset((shadow.spread, shadow.spread));
        let spread_radii = radii.map(|r| {
            skia_safe::Point::new(
                (r.x + shadow.spread).max(0.0),
                (r.y + shadow.spread).max(0.0),
            )
        });
        let shape = if rounded {
            RRect::new_rect_radii(spread_bounds, &spread_radii)
        } else {
            RRect::new_rect(spread_bounds)
        };
        let mut paint = Paint::default();
        paint.set_color(shadow.color);
        paint.set_anti_alias(true);
        if shadow.blur > 0.0 {
            paint.set_mask_filter(skia_safe::MaskFilter::blur(
                skia_safe::BlurStyle::Normal,
                shadow.blur / 2.0,
                false,
            ));
        }
        canvas.draw_rrect(shape, &paint);
    }
    canvas.restore();
}

fn paint_inset_shadows(canvas: &Canvas, rect: AbsoluteRect, style: &crate_style::ComputedStyle) {
    let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height);
    let radii = primitives::corner_radii(style);
    let rounded = primitives::has_any_corner_radius(style);
    let box_rrect = if rounded {
        RRect::new_rect_radii(bounds, &radii)
    } else {
        RRect::new_rect(bounds)
    };
    canvas.save();
    canvas.clip_rrect(box_rrect, None, true);
    for shadow in style.box_shadows.iter().rev().filter(|shadow| shadow.inset) {
        let inner_bounds = bounds
            .with_offset((shadow.x, shadow.y))
            .with_inset((shadow.spread, shadow.spread));
        let inner_radii = radii.map(|r| {
            skia_safe::Point::new(
                (r.x - shadow.spread).max(0.0),
                (r.y - shadow.spread).max(0.0),
            )
        });
        let inner = if rounded {
            RRect::new_rect_radii(inner_bounds, &inner_radii)
        } else {
            RRect::new_rect(inner_bounds)
        };

        let reach = shadow.blur + shadow.spread.abs() + 1.0;
        let outer = RRect::new_rect(bounds.with_outset((reach, reach)));
        let mut paint = Paint::default();
        paint.set_color(shadow.color);
        paint.set_anti_alias(true);
        if shadow.blur > 0.0 {
            paint.set_mask_filter(skia_safe::MaskFilter::blur(
                skia_safe::BlurStyle::Normal,
                shadow.blur / 2.0,
                false,
            ));
        }
        canvas.draw_drrect(outer, inner, &paint);
    }
    canvas.restore();
}

fn top_layer_nodes(tree: &WidgetTree) -> Vec<(WidgetId, f32, f32)> {
    fn walk(
        tree: &WidgetTree,
        id: WidgetId,
        parent_x: f32,
        parent_y: f32,
        out: &mut Vec<(WidgetId, f32, f32)>,
    ) {
        if tree.nodes[id].widget.blocks_descendant_hit_test()
            || tree.nodes[id].style.pointer_events_none
        {
            return;
        }
        let abs_rect = abs_rect_of(tree, id, parent_x, parent_y);
        let (child_x, child_y) = match tree.nodes[id].widget.scroll_offset() {
            Some((dx, dy)) => (abs_rect.x - dx, abs_rect.y - dy),
            None => (abs_rect.x, abs_rect.y),
        };
        for &child in &tree.nodes[id].children {
            if tree.nodes[child].widget.is_top_layer() || tree.nodes[child].widget.hit_priority() {
                out.push((child, child_x, child_y));
            }
            walk(tree, child, child_x, child_y, out);
        }
    }
    let mut out = Vec::new();
    walk(
        tree,
        tree.root,
        0.0,
        app::current_top_inset(),
        &mut out,
    );

    // Callers test this list back to front. A window-covering dismiss backdrop is
    // mounted after the panel it dismisses, so left in place it would trap every
    // pointer event before the panel saw one. Scrims go first (tested last).
    let (mut scrims, rest): (Vec<_>, Vec<_>) = out.into_iter().partition(|&(id, ..)| {
        let widget = &tree.nodes[id].widget;
        widget.is_dismiss_backdrop() && widget.is_top_layer()
    });
    scrims.extend(rest);
    scrims
}

fn abs_rect_of(tree: &WidgetTree, id: WidgetId, parent_x: f32, parent_y: f32) -> AbsoluteRect {
    let node = &tree.nodes[id];
    let layout = &node.layout;

    if let Some((x, y)) = node
        .widget
        .anchored_position((layout.size.width, layout.size.height))
    {
        return AbsoluteRect {
            x,
            y,
            width: layout.size.width,
            height: layout.size.height,
        };
    }
    let (local_x, local_y) = node
        .widget
        .absolute_position_override()
        .unwrap_or((layout.location.x, layout.location.y));

    AbsoluteRect {
        x: parent_x + local_x,
        y: parent_y + local_y,
        width: layout.size.width,
        height: layout.size.height,
    }
}

pub fn content_extent(tree: &WidgetTree, id: WidgetId) -> (f32, f32) {
    fn subtree(tree: &WidgetTree, id: WidgetId) -> (f32, f32) {
        let own = tree.layout(id);
        let mut width = own.size.width;
        let mut height = own.size.height;
        // A clipped box contributes only itself: what it hides doesn't add scrollable room.
        if tree.nodes[id].widget.scroll_offset().is_none()
            && !overflow_clips(&tree.nodes[id].style)
        {
            for &child in &tree.nodes[id].children {
                // Tooltips and popouts are painted on the top layer, not in the scroll flow.
                if tree.nodes[child].widget.is_top_layer() {
                    continue;
                }
                let layout = tree.layout(child);
                let (child_width, child_height) = subtree(tree, child);
                width = width.max(layout.location.x + child_width);
                height = height.max(layout.location.y + child_height);
            }
        }
        (width, height)
    }

    let mut max_x: f32 = 0.0;
    let mut max_y: f32 = 0.0;
    for &child in &tree.nodes[id].children {
        if tree.nodes[child].widget.is_top_layer() {
            continue;
        }
        let layout = tree.layout(child);
        let (width, height) = subtree(tree, child);
        max_x = max_x.max(layout.location.x + width);
        max_y = max_y.max(layout.location.y + height);
    }
    let container_padding = tree.layout(id).padding;
    (
        max_x + container_padding.right,
        max_y + container_padding.bottom,
    )
}

pub fn all_absolute_rects(tree: &WidgetTree) -> HashMap<WidgetId, AbsoluteRect> {
    fn walk(
        tree: &WidgetTree,
        id: WidgetId,
        parent_x: f32,
        parent_y: f32,
        out: &mut HashMap<WidgetId, AbsoluteRect>,
    ) {
        let rect = abs_rect_of(tree, id, parent_x, parent_y);
        let origin = match tree.nodes[id].widget.scroll_offset() {
            Some((dx, dy)) => (rect.x - dx, rect.y - dy),
            None => (rect.x, rect.y),
        };
        out.insert(id, rect);
        for &child in &tree.nodes[id].children {
            walk(tree, child, origin.0, origin.1, out);
        }
    }

    let mut out = HashMap::new();
    walk(
        tree,
        tree.root,
        0.0,
        app::current_top_inset(),
        &mut out,
    );
    out
}

#[derive(Clone, Copy)]
struct FocusRing {
    matrix: skia_safe::Matrix,
    rect: AbsoluteRect,
    color: Option<Color>,
    width: Option<f32>,
    corner_radius: f32,

    from_host: bool,
}

fn paint_focus_ring(canvas: &Canvas, ring: FocusRing) {
    let FocusRing {
        matrix,
        rect,
        color,
        width,
        corner_radius,
        ..
    } = ring;
    let color = color.unwrap_or(DEFAULT_FOCUS_RING_COLOR);
    let width = width.unwrap_or(DEFAULT_FOCUS_RING_WIDTH);
    if width <= 0.0 {
        return;
    }
    canvas.save();
    canvas.reset_matrix();
    canvas.concat(&matrix);

    let mut paint = Paint::default();
    paint.set_color(color);
    paint.set_anti_alias(true);
    paint.set_style(PaintStyle::Stroke);
    paint.set_stroke_width(width);

    let half = width / 2.0;
    let bounds = Rect::from_xywh(rect.x, rect.y, rect.width, rect.height).with_outset((half, half));
    let radius = corner_radius + half;
    if radius > 0.0 {
        canvas.draw_rrect(RRect::new_rect_xy(bounds, radius, radius), &paint);
    } else {
        canvas.draw_rect(bounds, &paint);
    }
    canvas.restore();
}

fn paint_node(
    tree: &mut WidgetTree,
    id: WidgetId,
    parent_x: f32,
    parent_y: f32,
    canvas: &Canvas,
    deferred: &mut Vec<(WidgetId, f32, f32)>,
) {
    if is_display_none(tree, id) {
        return;
    }
    let abs_rect = abs_rect_of(tree, id, parent_x, parent_y);

    if tree.nodes[id].widget.scroll_offset().is_some() {
        let content = content_extent(tree, id);
        let viewport = (abs_rect.width, abs_rect.height);
        if tree.nodes[id].widget.wants_scroll_children() {
            let child_tops: Vec<(Option<String>, f32, f32)> = tree.nodes[id]
                .children
                .iter()
                .map(|&child| {
                    let layout = tree.layout(child);
                    (
                        tree.nodes[child].dom_id.clone(),
                        layout.location.y,
                        layout.size.height,
                    )
                })
                .collect();
            let children: Vec<widget::ScrollChild<'_>> = child_tops
                .iter()
                .map(|(key, top, height)| widget::ScrollChild {
                    key: key.as_deref(),
                    top: *top,
                    height: *height,
                })
                .collect();
            tree.nodes[id]
                .widget
                .update_scroll_bounds(viewport, content, &children);
        } else {
            tree.nodes[id]
                .widget
                .update_scroll_bounds(viewport, content, &[]);
        }
    }

    let animated = {
        let node = &mut tree.nodes[id];
        let mut sampled = node.transitions.sample(&node.style);
        if sampled.is_some() {
            keep_animating();
        }

        if let Some(animation) = node
            .animations
            .sample(sampled.as_ref().unwrap_or(&node.style))
        {
            if animation.running {
                if animation.spills {
                    keep_animating();
                } else {
                    animate_region(
                        canvas,
                        Rect::from_xywh(abs_rect.x, abs_rect.y, abs_rect.width, abs_rect.height),
                    );
                }
            }
            sampled = Some(animation.style);
        }
        sampled
    };
    let node = &tree.nodes[id];
    let style = animated.as_ref().unwrap_or(&node.style);

    if style.opacity.is_some_and(|o| o <= 0.001) {
        return;
    }
    let group = node.widget.paint_group_transform();

    // The element is painted on its own layer and composited with its blend mode on the way
    // out, so only it is blended with what is behind.
    let css_blend = style.blend_mode;
    if let Some(blend) = css_blend {
        let mut paint = Paint::default();
        paint.set_blend_mode(blend.to_skia());
        let bounds = Rect::from_xywh(abs_rect.x, abs_rect.y, abs_rect.width, abs_rect.height);
        canvas.save_layer(
            &skia_safe::canvas::SaveLayerRec::default()
                .bounds(&bounds)
                .paint(&paint),
        );
    }

    let css_opacity = style.opacity;
    if let Some(opacity) = css_opacity {
        canvas.save_layer_alpha_f(None, opacity);
    }

    let css_transform = style.transform;
    if let Some(t) = css_transform {
        let (fx, fy) = style.transform_origin.unwrap_or((0.5, 0.5));
        let (ox, oy) = (
            abs_rect.x + abs_rect.width * fx,
            abs_rect.y + abs_rect.height * fy,
        );
        canvas.save();
        canvas.translate((ox + t.translate_x, oy + t.translate_y));
        if t.rotate_deg != 0.0 {
            canvas.rotate(t.rotate_deg, None);
        }
        canvas.scale((t.scale_x, t.scale_y));
        canvas.translate((-ox, -oy));
    }

    if !style.box_shadows.is_empty() {
        paint_box_shadows(canvas, abs_rect, style);
    }

    let css_filter = node.style.filter;
    if let Some(filter) = css_filter {
        canvas.save();
        let bounds = Rect::from_xywh(abs_rect.x, abs_rect.y, abs_rect.width, abs_rect.height);
        if primitives::has_any_corner_radius(&node.style) {
            canvas.clip_rrect(
                RRect::new_rect_radii(
                    bounds,
                    &primitives::corner_radii(&node.style),
                ),
                None,
                true,
            );
        } else {
            canvas.clip_rect(bounds, None, true);
        }
        let mut layer_paint = Paint::default();
        if filter.blur > 0.0 {
            // CSS blurs treat everything past the element's edge as transparent, so the
            // element's own edges soften; clamping would keep them hard.
            layer_paint.set_image_filter(image_filters::blur(
                (filter.blur, filter.blur),
                skia_safe::TileMode::Decal,
                None,
                None,
            ));
        }
        if (filter.brightness - 1.0).abs() > f32::EPSILON {
            let b = filter.brightness;
            let matrix = skia_safe::ColorMatrix::new(
                b, 0.0, 0.0, 0.0, 0.0, 0.0, b, 0.0, 0.0, 0.0, 0.0, 0.0, b, 0.0, 0.0, 0.0, 0.0, 0.0,
                1.0, 0.0,
            );
            layer_paint.set_color_filter(color_filters::matrix(&matrix, None));
        }
        if let Some(shadow) = filter.drop_shadow {
            let sigma = shadow.blur / 2.0;
            let shadowed = image_filters::drop_shadow(
                (shadow.x, shadow.y),
                (sigma, sigma),
                shadow.color,
                None,
                layer_paint.image_filter(),
                None,
            );
            layer_paint.set_image_filter(shadowed);
        }
        canvas.save_layer(&skia_safe::canvas::SaveLayerRec::default().paint(&layer_paint));
    }

    if let Some((opacity, scale_x, scale_y, anchor_y)) = group {
        canvas.save_layer_alpha_f(None, opacity);
        let anchor_x_px = abs_rect.x + abs_rect.width / 2.0;
        let anchor_y_px = abs_rect.y + abs_rect.height * anchor_y;
        canvas.translate((anchor_x_px, anchor_y_px));
        canvas.scale((scale_x, scale_y));
        canvas.translate((-anchor_x_px, -anchor_y_px));
    }

    let node = &tree.nodes[id];
    if perf::profile_paint() {
        let started = Instant::now();
        node.widget
            .paint(canvas, abs_rect, animated.as_ref().unwrap_or(&node.style));
        let ms = started.elapsed().as_secs_f64() * 1000.0;
        if ms > 0.3 {
            perf::record_widget_paint(
                format!(
                    "{}.{}",
                    node.tag,
                    node.classes.first().map_or("", String::as_str)
                ),
                ms,
            );
        }
    } else {
        node.widget
            .paint(canvas, abs_rect, animated.as_ref().unwrap_or(&node.style));
    }
    if node.style.box_shadows.iter().any(|shadow| shadow.inset) {
        paint_inset_shadows(canvas, abs_rect, &node.style);
    }

    let scroll = node.widget.scroll_offset();

    let child_count = tree.nodes[id].children.len();

    let clip_bounds = Rect::from_xywh(abs_rect.x, abs_rect.y, abs_rect.width, abs_rect.height);
    let (child_x, child_y) = match scroll {
        Some((dx, dy)) => {
            canvas.save();
            canvas.clip_rect(
                Rect::from_xywh(abs_rect.x, abs_rect.y, abs_rect.width, abs_rect.height),
                None,
                None,
            );
            (abs_rect.x - dx, abs_rect.y - dy)
        }
        None => (abs_rect.x, abs_rect.y),
    };

    let overflow_clip = scroll.is_none() && overflow_clips(&tree.nodes[id].style);
    let outer_layout_clip =
        (scroll.is_some() || overflow_clip).then(|| push_layout_clip(canvas, clip_bounds));
    if overflow_clip {
        canvas.save();
        let bounds = Rect::from_xywh(abs_rect.x, abs_rect.y, abs_rect.width, abs_rect.height);
        let style = &tree.nodes[id].style;
        if primitives::has_any_corner_radius(style) {
            canvas.clip_rrect(
                RRect::new_rect_radii(bounds, &primitives::corner_radii(style)),
                None,
                true,
            );
        } else {
            canvas.clip_rect(bounds, None, true);
        }
    }

    let cull = scroll.is_some() || overflow_clip;
    for i in 0..child_count {
        let child = tree.nodes[id].children[i];
        if tree.nodes[child].widget.is_top_layer() {
            deferred.push((child, child_x, child_y));
            continue;
        }
        if cull && tree.nodes[child].style.transform.is_none() {
            let r = abs_rect_of(tree, child, child_x, child_y);
            if r.x > clip_bounds.right + 2.0
                || r.x + r.width < clip_bounds.left - 2.0
                || r.y > clip_bounds.bottom + 2.0
                || r.y + r.height < clip_bounds.top - 2.0
            {
                continue;
            }
        }
        paint_node(tree, child, child_x, child_y, canvas, deferred);
    }

    if scroll.is_some() || overflow_clip {
        canvas.restore();
    }
    if let Some(previous) = outer_layout_clip {
        pop_layout_clip(previous);
    }

    tree.nodes[id].widget.paint_overlay(canvas, abs_rect);

    if tree.focused == Some(id) && tree.focus_visible && !focus_hidden(tree, id) {
        let style = &tree.nodes[id].style;
        FOCUS_RING.with(|ring| {
            ring.set(Some(FocusRing {
                matrix: canvas.local_to_device_as_3x3(),
                rect: abs_rect,
                color: style.outline_color,
                width: style.outline_width,
                corner_radius: style.corner_radius,
                from_host: false,
            }))
        });
    }

    if tree.nodes[id].focus_ring_host
        && tree.focus_visible
        && tree.focused.is_some_and(|focused| {
            focused != id
                && tree
                    .nodes
                    .get(focused)
                    .is_some_and(|node| node.tag != "button")
                && is_ancestor(tree, id, focused)
        })
    {
        let style = &tree.nodes[id].style;
        let matrix = canvas.local_to_device_as_3x3();
        FOCUS_RING.with(|ring| {
            if let Some(inner) = ring.get().filter(|inner| !inner.from_host) {
                ring.set(Some(FocusRing {
                    matrix,
                    rect: abs_rect,
                    color: style.outline_color.or(inner.color),
                    width: style.outline_width.or(inner.width),
                    corner_radius: style.corner_radius,
                    from_host: true,
                }));
            }
        });
    }

    if group.is_some() {
        canvas.restore();
    }
    if css_filter.is_some() {
        canvas.restore();
        canvas.restore();
    }
    if css_transform.is_some() {
        canvas.restore();
    }
    if css_opacity.is_some() {
        canvas.restore();
    }
    if css_blend.is_some() {
        canvas.restore();
    }
}

pub fn clear_all_press(tree: &mut WidgetTree) {
    let ids: Vec<WidgetId> = tree.nodes.keys().collect();
    for id in ids {
        tree.nodes[id].widget.clear_press();
    }
}

thread_local! {
    static LAST_PRESS: Cell<(f32, f32)> = const { Cell::new((0.0, 0.0)) };
}

/// Where the most recent mouse press landed, in window coordinates. Lets a mouse-down
/// hook tell which widget was under the pointer when a backdrop took the event.
pub fn last_press() -> (f32, f32) {
    LAST_PRESS.with(Cell::get)
}

pub fn dispatch_event(tree: &mut WidgetTree, event: &UiEvent) -> EventResult {
    dispatch_event_with_handler(tree, event).0
}

pub fn dispatch_event_with_handler(
    tree: &mut WidgetTree,
    event: &UiEvent,
) -> (EventResult, Option<WidgetId>) {
    if matches!(event, UiEvent::MouseUp { .. }) && mem::take(&mut tree.swallow_release) {
        return (EventResult::Handled, None);
    }
    if matches!(event, UiEvent::MouseDown { .. }) {
        LAST_PRESS.with(|p| p.set(event.point()));
    }
    update_interactive_paths(tree, event);

    let captured = tree
        .pointer_capture
        .filter(|&id| tree.nodes.contains_key(id))
        .filter(|&id| match event {
            UiEvent::MouseMoved { .. } | UiEvent::MouseUp { .. } => true,

            UiEvent::MouseDown { .. } => tree.nodes[id]
                .widget
                .intercepts_pointer(event, absolute_rect(tree, id)),
            _ => false,
        });

    let handler = match captured {
        Some(id) => {
            let abs_rect = absolute_rect(tree, id);
            let result = tree.nodes[id].widget.handle_event(event, abs_rect);
            matches!(result, EventResult::Handled).then_some(id)
        }
        None => top_layer_nodes(tree)
            .into_iter()
            .rev()
            .find_map(|(id, x, y)| dispatch_node(tree, id, x, y, event))
            .or_else(|| {
                dispatch_node(tree, tree.root, 0.0, app::current_top_inset(), event)
            }),
    };
    if matches!(event, UiEvent::MouseDown { .. }) {
        tree.swallow_release =
            handler.is_some_and(|id| tree.nodes[id].widget.is_dismiss_backdrop());
    }

    if let Some(id) = handler {
        if tree.nodes[id].widget.relayout_after(event) {
            tree.mark_dirty(id);
        } else if matches!(event, UiEvent::MouseMoved { .. }) {
            let rect = absolute_rect(tree, id);
            request_repaint_rect(rect);
            if let Some(extra) = tree.nodes[id].widget.extra_hit_rect(rect) {
                request_repaint_rect(extra);
            }
        }
    }

    if matches!(event, UiEvent::MouseDown { .. }) {
        let wants_focus = handler.is_some_and(|id| {
            tree.nodes[id].widget.wants_focus()
                && tree
                    .focus_scope
                    .is_none_or(|scope| ancestor_chain(tree, id).contains(&scope))
        });
        let next = if wants_focus { handler } else { None };

        if !matches!(
            event,
            UiEvent::MouseDown {
                button: MouseButton::Right,
                ..
            }
        ) {
            set_focus(tree, next, false);
        }

        tree.pointer_capture = handler.filter(|&id| tree.nodes[id].widget.wants_pointer_capture());
    }

    if matches!(event, UiEvent::MouseUp { .. }) {
        tree.pointer_capture = tree.pointer_capture.filter(|&id| {
            tree.nodes
                .get(id)
                .is_some_and(|node| node.widget.wants_pointer_capture())
        });
    }

    if matches!(
        event,
        UiEvent::MouseDown { .. } | UiEvent::MouseMoved { .. } | UiEvent::MouseUp { .. }
    ) && selection::after_dispatch(tree, event, handler)
    {
        request_repaint();
    }

    (
        handler.map_or(EventResult::Ignored, |_| EventResult::Handled),
        handler,
    )
}

pub(crate) fn absolute_rect(tree: &WidgetTree, id: WidgetId) -> AbsoluteRect {
    let mut path = Vec::new();
    let mut current = Some(id);
    while let Some(node_id) = current {
        path.push(node_id);
        current = tree.nodes[node_id].parent;
    }
    path.reverse();

    let mut origin = (0.0, app::current_top_inset());
    let mut rect = AbsoluteRect::default();
    for node_id in path {
        rect = abs_rect_of(tree, node_id, origin.0, origin.1);
        origin = match tree.nodes[node_id].widget.scroll_offset() {
            Some((dx, dy)) => (rect.x - dx, rect.y - dy),
            None => (rect.x, rect.y),
        };
    }
    rect
}

fn focus_hidden(tree: &WidgetTree, id: WidgetId) -> bool {
    let mut current = tree.nodes.get(id).and_then(|node| node.parent);
    while let Some(parent) = current {
        let Some(node) = tree.nodes.get(parent) else {
            return false;
        };
        if node.widget.hides_descendant_focus() {
            return true;
        }
        current = node.parent;
    }
    false
}

fn focus_order(tree: &WidgetTree) -> Vec<WidgetId> {
    fn walk(tree: &WidgetTree, id: WidgetId, order: &mut Vec<WidgetId>) {
        if tree.nodes[id].widget.wants_focus() {
            order.push(id);
        }
        if tree.nodes[id].widget.hides_descendant_focus() {
            return;
        }
        for &child in &tree.nodes[id].children {
            walk(tree, child, order);
        }
    }

    let start = tree
        .focus_scope
        .filter(|&id| tree.nodes.contains_key(id))
        .unwrap_or(tree.root);
    let mut order = Vec::new();
    walk(tree, start, &mut order);
    order
}

fn tab_order(tree: &WidgetTree) -> Vec<WidgetId> {
    let mut order = focus_order(tree);
    order.retain(|&id| tree.focused == Some(id) || tree.nodes[id].widget.in_tab_order());
    order
}

pub fn focus_widget(tree: &mut WidgetTree, id: WidgetId) {
    set_focus(tree, Some(id), true);
    reveal(tree, id);
}

pub fn focus_index(tree: &WidgetTree) -> Option<usize> {
    let order = focus_order(tree);
    tree.focused
        .and_then(|id| order.iter().position(|&o| o == id))
}

pub fn restore_focus(tree: &mut WidgetTree, index: Option<usize>, visible: bool) {
    if let Some(index) = index {
        let order = focus_order(tree);
        if let Some(&id) = order.get(index) {
            set_focus(tree, Some(id), visible);
        }
    }
    apply_focus_claim(tree, true);
}

thread_local! {
    static FOCUS_CLAIM: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Moves focus to `dom_id` once a rebuild has mounted it. Restoring focus by
/// index after a rebuild would otherwise hand it back to the previously
/// focused widget.
pub fn claim_focus(dom_id: &str) {
    FOCUS_CLAIM.with(|claim| *claim.borrow_mut() = Some(dom_id.to_string()));
}

pub fn release_focus_claim() {
    FOCUS_CLAIM.with(|claim| *claim.borrow_mut() = None);
}

/// Moves focus to the claimed widget once it is mounted. A flush applies it
/// without releasing, since the restore that follows may still hand focus back
/// to the old widget; the restore applies it again and releases.
pub(crate) fn apply_focus_claim(tree: &mut WidgetTree, release: bool) {
    let Some(dom_id) = FOCUS_CLAIM.with(|claim| claim.borrow().clone()) else {
        return;
    };
    if focus_dom_id(tree, &dom_id) && release {
        release_focus_claim();
    }
}

pub fn clear_hidden_focus(tree: &mut WidgetTree) {
    if tree.focused.is_some_and(|id| focus_hidden(tree, id)) {
        set_focus(tree, None, false);
    }
}

pub fn cycle_focus(tree: &mut WidgetTree, backward: bool) {
    let order = tab_order(tree);
    if order.is_empty() {
        set_focus(tree, None, false);
        return;
    }

    let current_index = tree
        .focused
        .and_then(|id| order.iter().position(|&o| o == id));
    let next_index = match (current_index, backward) {
        (None, false) => 0,
        (None, true) => order.len() - 1,
        (Some(i), false) => (i + 1) % order.len(),
        (Some(i), true) => (i + order.len() - 1) % order.len(),
    };
    set_focus(tree, Some(order[next_index]), true);
    reveal(tree, order[next_index]);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub fn move_focus_direction(tree: &mut WidgetTree, direction: Direction) -> bool {
    let Some(current) = tree.focused.filter(|&id| tree.nodes.contains_key(id)) else {
        return false;
    };
    let rects = all_absolute_rects(tree);
    let Some(from) = rects.get(&current).copied() else {
        return false;
    };
    let center = |r: &AbsoluteRect| (r.x + r.width / 2.0, r.y + r.height / 2.0);
    let (fx, fy) = center(&from);

    let mut best: Option<(f32, WidgetId)> = None;
    for id in focus_order(tree) {
        if id == current {
            continue;
        }
        let Some(rect) = rects.get(&id) else { continue };
        if rect.width <= 0.0 || rect.height <= 0.0 {
            continue;
        }
        let (cx, cy) = center(rect);
        let (along, across, overlaps) = match direction {
            Direction::Down => (
                cy - fy,
                (cx - fx).abs(),
                rect.x < from.x + from.width && from.x < rect.x + rect.width,
            ),
            Direction::Up => (
                fy - cy,
                (cx - fx).abs(),
                rect.x < from.x + from.width && from.x < rect.x + rect.width,
            ),
            Direction::Right => (
                cx - fx,
                (cy - fy).abs(),
                rect.y < from.y + from.height && from.y < rect.y + rect.height,
            ),
            Direction::Left => (
                fx - cx,
                (cy - fy).abs(),
                rect.y < from.y + from.height && from.y < rect.y + rect.height,
            ),
        };

        if along <= 0.5 || (!overlaps && across > along + 16.0) {
            continue;
        }
        let score = along + across * if overlaps { 0.25 } else { 3.0 };
        if best.is_none_or(|(s, _)| score < s) {
            best = Some((score, id));
        }
    }
    let Some((_, target)) = best else {
        return false;
    };
    set_focus(tree, Some(target), true);
    reveal(tree, target);
    true
}

pub fn reveal(tree: &mut WidgetTree, id: WidgetId) {
    const MARGIN: f32 = 8.0;
    let mut ancestor = tree.nodes.get(id).and_then(|n| n.parent);
    while let Some(scroller) = ancestor {
        ancestor = tree.nodes[scroller].parent;
        if tree.nodes[scroller].widget.scroll_offset().is_none() {
            continue;
        }
        let viewport = absolute_rect(tree, scroller);
        let target = absolute_rect(tree, id);
        let delta_y = if target.y < viewport.y {
            viewport.y - target.y + MARGIN
        } else if target.y + target.height > viewport.y + viewport.height {
            -(target.y + target.height - (viewport.y + viewport.height) + MARGIN)
        } else {
            continue;
        };

        let delta_y = if target.height + 2.0 * MARGIN > viewport.height && delta_y < 0.0 {
            viewport.y - target.y + MARGIN
        } else {
            delta_y
        };
        let event = UiEvent::Scroll {
            x: viewport.x,
            y: viewport.y,
            delta_x: 0.0,
            delta_y,
            precise: true,
        };
        tree.nodes[scroller].widget.handle_event(&event, viewport);
        request_repaint();
    }
}

fn update_interactive_paths(tree: &mut WidgetTree, event: &UiEvent) {
    let new_hovered = match *event {
        UiEvent::MouseMoved { x, y } => Some(hit_path(tree, x, y)),
        _ => None,
    };
    let new_active = match *event {
        UiEvent::MouseDown {
            x,
            y,
            button: MouseButton::Left,
            ..
        } => Some(hit_path(tree, x, y)),
        UiEvent::MouseUp {
            button: MouseButton::Left,
            ..
        } => Some(HashSet::new()),
        _ => None,
    };

    if let Some(path) = new_hovered {
        if path != tree.hovered_path {
            let left: Vec<WidgetId> = tree.hovered_path.difference(&path).copied().collect();
            let entered: Vec<WidgetId> = path.difference(&tree.hovered_path).copied().collect();
            for &id in &left {
                if let Some(node) = tree.nodes.get_mut(id) {
                    node.widget.clear_hover();
                    node.widget.hover_path_changed(false);
                }
            }
            for &id in &entered {
                if let Some(node) = tree.nodes.get_mut(id) {
                    node.widget.hover_path_changed(true);
                }
            }

            tree.interactive_changed
                .extend(left.iter().copied().chain(entered.iter().copied()));
            tree.hovered_path = path;
        }
    }
    if let Some(path) = new_active {
        if path != tree.active_path {
            let left: Vec<WidgetId> = tree.active_path.difference(&path).copied().collect();
            let entered: Vec<WidgetId> = path.difference(&tree.active_path).copied().collect();
            for &id in &left {
                if let Some(node) = tree.nodes.get_mut(id) {
                    node.widget.press_path_changed(false);
                }
            }
            for &id in &entered {
                if let Some(node) = tree.nodes.get_mut(id) {
                    node.widget.press_path_changed(true);
                }
            }
            tree.interactive_changed
                .extend(left.iter().copied().chain(entered.iter().copied()));
            tree.active_path = path;
        }
    }
}

#[doc(hidden)]
pub fn hit_test_all(tree: &WidgetTree, x: f32, y: f32) -> Option<WidgetId> {
    top_layer_nodes(tree)
        .into_iter()
        .rev()
        .find_map(|(id, px, py)| hit_test(tree, id, px, py, x, y))
        .or_else(|| hit_test(tree, tree.root, 0.0, app::current_top_inset(), x, y))
}

/// True when what's under the point no longer matches the hovered set, as after a rebuild,
/// a scroll or a relayout moved content beneath a motionless pointer.
pub fn hover_out_of_date(tree: &WidgetTree, x: f32, y: f32) -> bool {
    hit_path(tree, x, y) != tree.hovered_path
}

fn hit_path(tree: &WidgetTree, x: f32, y: f32) -> HashSet<WidgetId> {
    hit_test_all(tree, x, y)
        .map(|id| ancestor_chain(tree, id))
        .unwrap_or_default()
}

pub fn cursor_at(tree: &WidgetTree, x: f32, y: f32) -> CursorIcon {
    let Some(mut id) = hit_test_all(tree, x, y) else {
        return CursorIcon::Default;
    };
    loop {
        let rect = absolute_rect(tree, id);
        if let Some(cursor) = tree.nodes[id]
            .widget
            .cursor_icon_at(rect, x, y)
            .or(tree.nodes[id].style.cursor)
        {
            return cursor;
        }
        match tree.nodes[id].parent {
            Some(parent) => id = parent,
            None => return CursorIcon::Default,
        }
    }
}

fn is_display_none(tree: &WidgetTree, id: WidgetId) -> bool {
    tree.nodes[id].style.layout.display == taffy::Display::None
}

fn clips_children(tree: &WidgetTree, id: WidgetId) -> bool {
    let node = &tree.nodes[id];
    node.widget.scroll_offset().is_some() || overflow_clips(&node.style)
}

fn overflow_clips(style: &crate_style::ComputedStyle) -> bool {
    let clips = |o: Overflow| matches!(o, Overflow::Hidden | Overflow::Clip);
    clips(style.layout.overflow.x) || clips(style.layout.overflow.y)
}

fn hit_test(
    tree: &WidgetTree,
    id: WidgetId,
    parent_x: f32,
    parent_y: f32,
    x: f32,
    y: f32,
) -> Option<WidgetId> {
    if is_display_none(tree, id) {
        return None;
    }

    if tree.nodes[id].widget.blocks_descendant_hit_test()
        || tree.nodes[id].style.pointer_events_none
    {
        return None;
    }

    let abs_rect = abs_rect_of(tree, id, parent_x, parent_y);

    let scroll = tree.nodes[id].widget.scroll_offset();
    if clips_children(tree, id) && !abs_rect.contains(x, y) {
        return None;
    }

    let (child_x, child_y) = match scroll {
        Some((dx, dy)) => (abs_rect.x - dx, abs_rect.y - dy),
        None => (abs_rect.x, abs_rect.y),
    };

    for &child in tree.nodes[id].children.iter().rev() {
        if tree.nodes[child].widget.is_top_layer() {
            continue;
        }
        if let Some(hit) = hit_test(tree, child, child_x, child_y, x, y) {
            return Some(hit);
        }
    }

    (abs_rect.contains(x, y)
        || tree.nodes[id]
            .widget
            .extra_hit_rect(abs_rect)
            .is_some_and(|extra| extra.contains(x, y)))
    .then_some(id)
}

fn is_ancestor(tree: &WidgetTree, ancestor: WidgetId, id: WidgetId) -> bool {
    let mut current = Some(id);
    while let Some(node_id) = current {
        if node_id == ancestor {
            return true;
        }
        current = tree.nodes.get(node_id).and_then(|node| node.parent);
    }
    false
}

fn ancestor_chain(tree: &WidgetTree, id: WidgetId) -> HashSet<WidgetId> {
    let mut path = HashSet::new();
    let mut current = Some(id);
    while let Some(node_id) = current {
        path.insert(node_id);
        current = tree.nodes[node_id].parent;
    }
    path
}

pub fn focus_by_dom_id(tree: &mut WidgetTree, dom_id: &str) -> bool {
    let Some(id) = find_by_dom_id(tree, dom_id) else {
        return false;
    };
    tree.focused = Some(id);
    refresh_focus_styles(tree);
    true
}

pub fn focus_dom_id(tree: &mut WidgetTree, dom_id: &str) -> bool {
    let Some(id) = find_by_dom_id(tree, dom_id) else {
        return false;
    };
    set_focus(tree, Some(id), false);
    true
}

pub fn find_by_dom_id(tree: &WidgetTree, dom_id: &str) -> Option<WidgetId> {
    tree.nodes
        .iter()
        .find(|(_, node)| node.dom_id.as_deref() == Some(dom_id))
        .map(|(id, _)| id)
}

pub fn rect_of_dom_id(tree: &WidgetTree, dom_id: &str) -> Option<AbsoluteRect> {
    find_by_dom_id(tree, dom_id).map(|id| absolute_rect(tree, id))
}

pub fn rect_of(tree: &WidgetTree, id: WidgetId) -> AbsoluteRect {
    absolute_rect(tree, id)
}

pub fn is_descendant_or_self(tree: &WidgetTree, id: WidgetId, ancestor_dom_id: &str) -> bool {
    let Some(ancestor_id) = find_by_dom_id(tree, ancestor_dom_id) else {
        return false;
    };
    ancestor_chain(tree, id).contains(&ancestor_id)
}

pub fn dispatch_tab(tree: &mut WidgetTree, backward: bool) {
    if !matches!(
        dispatch_key_to_focus(
            tree,
            &UiEvent::KeyDown {
                key: NamedKey::Tab,
                shift: backward
            }
        ),
        EventResult::Handled
    ) {
        cycle_focus(tree, backward);
    }
}

pub fn dispatch_key_event(tree: &mut WidgetTree, event: &UiEvent) -> EventResult {
    let result = dispatch_key_to_focus(tree, event);

    if matches!(result, EventResult::Ignored) {
        if let UiEvent::KeyDown {
            key:
                key @ (NamedKey::ArrowUp
                | NamedKey::ArrowDown
                | NamedKey::ArrowLeft
                | NamedKey::ArrowRight),
            shift: false,
        } = event
        {
            let direction = match key {
                NamedKey::ArrowUp => Direction::Up,
                NamedKey::ArrowDown => Direction::Down,
                NamedKey::ArrowLeft => Direction::Left,
                _ => Direction::Right,
            };
            if move_focus_direction(tree, direction) {
                return EventResult::Handled;
            }
        }
    }

    if matches!(result, EventResult::Ignored)
        && matches!(
            event,
            UiEvent::KeyDown {
                key: NamedKey::Copy,
                ..
            }
        )
        && selection::copy(tree)
    {
        return EventResult::Handled;
    }
    result
}

pub fn dismiss_top_popover(tree: &mut WidgetTree) -> bool {
    fn walk(
        tree: &WidgetTree,
        id: WidgetId,
        top: &mut Option<WidgetId>,
        deferred: &mut Vec<WidgetId>,
    ) {
        let node = &tree.nodes[id];
        if node.style.pointer_events_none || is_display_none(tree, id) {
            return;
        }
        if node.widget.is_dismiss_backdrop() {
            *top = Some(id);
        } else if node.widget.is_open_modal() {
            *top = None;
        }
        if node.widget.blocks_descendant_hit_test() {
            return;
        }
        for &child in &node.children {
            if tree.nodes[child].widget.is_top_layer() {
                deferred.push(child);
            } else {
                walk(tree, child, top, deferred);
            }
        }
    }
    let mut top = None;
    let mut deferred = Vec::new();
    walk(tree, tree.root, &mut top, &mut deferred);
    let mut next = 0;
    while next < deferred.len() {
        let id = deferred[next];
        next += 1;
        walk(tree, id, &mut top, &mut deferred);
    }
    top.is_some_and(|id| tree.nodes[id].widget.dismiss())
}

fn dispatch_key_to_focus(tree: &mut WidgetTree, event: &UiEvent) -> EventResult {
    let Some(id) = tree.focused else {
        return EventResult::Ignored;
    };
    if !tree.nodes.contains_key(id) {
        tree.focused = None;
        return EventResult::Ignored;
    }
    let rect = absolute_rect(tree, id);
    let result = tree.nodes[id].widget.handle_event(event, rect);

    if matches!(result, EventResult::Handled) && tree.nodes[id].widget.relayout_after(event) {
        tree.mark_dirty(id);
    }
    result
}

fn set_focus(tree: &mut WidgetTree, next: Option<WidgetId>, visible: bool) {
    if tree.focused == next {
        tree.focus_visible = next.is_some() && visible;
        return;
    }
    if let Some(prev) = tree.focused {
        if let Some(node) = tree.nodes.get_mut(prev) {
            node.widget.on_focus_change(false);
        }
    }
    if let Some(id) = next {
        tree.nodes[id].widget.on_focus_change(true);
    }
    tree.focused = next;
    tree.focus_visible = next.is_some() && visible;
    refresh_focus_styles(tree);
}

pub(crate) fn refresh_focus_styles(tree: &mut WidgetTree) {
    tree.invalidate_styles();
}

fn dispatch_node(
    tree: &mut WidgetTree,
    id: WidgetId,
    parent_x: f32,
    parent_y: f32,
    event: &UiEvent,
) -> Option<WidgetId> {
    let abs_rect = abs_rect_of(tree, id, parent_x, parent_y);
    let (x, y) = event.point();

    let scroll = tree.nodes[id].widget.scroll_offset();
    if clips_children(tree, id) && !abs_rect.contains(x, y) {
        return None;
    }
    let children = tree.nodes[id].children.clone();

    let (child_x, child_y) = match scroll {
        Some((dx, dy)) => (abs_rect.x - dx, abs_rect.y - dy),
        None => (abs_rect.x, abs_rect.y),
    };

    if tree.nodes[id].style.pointer_events_none {
        return None;
    }
    if abs_rect.contains(x, y)
        && tree.nodes[id].widget.intercepts_pointer(event, abs_rect)
        && matches!(
            tree.nodes[id].widget.handle_event(event, abs_rect),
            EventResult::Handled
        )
    {
        return Some(id);
    }
    if !tree.nodes[id].widget.blocks_descendant_hit_test() {
        let mut over: Vec<AbsoluteRect> = Vec::new();
        let presses = matches!(
            event,
            UiEvent::MouseDown { .. } | UiEvent::MouseUp { .. } | UiEvent::Scroll { .. }
        );
        for &child in children.iter().rev() {
            if tree.nodes[child].widget.is_dismiss_backdrop() && presses && !over.is_empty() {
                let backdrop = abs_rect_of(tree, child, child_x, child_y);

                if over
                    .iter()
                    .any(|rect| rect.width * rect.height < backdrop.width * backdrop.height - 1.0)
                {
                    return Some(child);
                }
            }
            if tree.nodes[child].widget.is_top_layer() {
                if presses && !tree.nodes[child].style.pointer_events_none {
                    let rect = abs_rect_of(tree, child, child_x, child_y);
                    if rect.contains(x, y) {
                        if let Some(handler) = dispatch_node(tree, child, child_x, child_y, event) {
                            return Some(handler);
                        }
                        over.push(rect);
                    }
                }
                continue;
            }
            if let Some(handler) = dispatch_node(tree, child, child_x, child_y, event) {
                return Some(handler);
            }
            if presses && !tree.nodes[child].style.pointer_events_none {
                let rect = abs_rect_of(tree, child, child_x, child_y);
                if rect.contains(x, y) {
                    over.push(rect);
                }
            }
        }
    }

    let widget = &tree.nodes[id].widget;
    if !abs_rect.contains(x, y)
        && !widget
            .extra_hit_rect(abs_rect)
            .is_some_and(|extra| extra.contains(x, y))
    {
        return None;
    }

    if matches!(
        event,
        UiEvent::MouseDown {
            button: MouseButton::Right,
            ..
        }
    ) {
        if let Some(handler) = tree.nodes[id].context_menu.clone() {
            handler(x, y);
            return Some(id);
        }
    }

    match tree.nodes[id].widget.handle_event(event, abs_rect) {
        EventResult::Handled => Some(id),

        EventResult::Ignored if tree.nodes[id].widget.traps_pointer() && event.is_pointer() => {
            Some(id)
        }
        EventResult::Ignored => None,
    }
}

#[cfg(test)]
mod focus_scope_tests {
    use super::*;
    use crate::style::ComputedStyle;
    use crate::widget::Widget;

    struct Focusable;
    impl Widget for Focusable {
        fn wants_focus(&self) -> bool {
            true
        }
    }

    struct Plain;
    impl Widget for Plain {}

    #[test]
    fn focus_order_is_confined_to_focus_scope_when_set() {
        let mut tree = WidgetTree::new(Box::new(Plain), ComputedStyle::default());
        let root = tree.root;
        let outside = tree.add_child(root, "div", Box::new(Focusable), ComputedStyle::default());
        let scope_root = tree.add_child(root, "div", Box::new(Plain), ComputedStyle::default());
        let inside = tree.add_child(
            scope_root,
            "div",
            Box::new(Focusable),
            ComputedStyle::default(),
        );

        assert_eq!(
            focus_order(&tree),
            vec![outside, inside],
            "unscoped: every focusable widget in the tree, background included"
        );

        tree.focus_scope = Some(scope_root);
        assert_eq!(
            focus_order(&tree),
            vec![inside],
            "scoped: only widgets inside the scope's own subtree"
        );
    }
}

#[cfg(test)]
mod scroll_clip_hit_test_tests {
    use super::*;
    use crate::event::MouseButton;
    use crate::layout;
    use crate::style::ComputedStyle;
    use crate::widget::Widget;
    use taffy::prelude::{FlexDirection, length};
    use taffy::style::{Display, Style as TaffyStyle};
    use taffy::{AvailableSpace, Size as TaffySize};

    struct Plain;
    impl Widget for Plain {}

    struct Clickable;
    impl Widget for Clickable {
        fn handle_event(&mut self, event: &UiEvent, _rect: AbsoluteRect) -> EventResult {
            match event {
                UiEvent::MouseDown { .. } => EventResult::Handled,
                _ => EventResult::Ignored,
            }
        }
    }

    struct ScrollStub(f32, f32);
    impl Widget for ScrollStub {
        fn scroll_offset(&self) -> Option<(f32, f32)> {
            Some((self.0, self.1))
        }
    }

    fn column_style(width: f32, height: f32) -> ComputedStyle {
        ComputedStyle {
            layout: TaffyStyle {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                size: TaffySize {
                    width: length(width),
                    height: length(height),
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn unshrinkable_style(width: f32, height: f32) -> ComputedStyle {
        ComputedStyle {
            layout: TaffyStyle {
                flex_shrink: 0.0,
                ..column_style(width, height).layout
            },
            ..Default::default()
        }
    }

    #[test]
    fn scrolled_out_of_view_item_does_not_shadow_content_above_the_scroll_container() {
        let mut tree = WidgetTree::new(Box::new(Plain), column_style(300.0, 300.0));
        let root = tree.root;

        let search = tree.add_child(root, "div", Box::new(Clickable), column_style(300.0, 50.0));
        let scroll_container = tree.add_child(
            root,
            "div",
            Box::new(ScrollStub(0.0, 300.0)),
            column_style(300.0, 250.0),
        );
        let item = tree.add_child(
            scroll_container,
            "div",
            Box::new(Clickable),
            unshrinkable_style(300.0, 400.0),
        );

        layout::compute_layout(
            &mut tree,
            TaffySize {
                width: AvailableSpace::Definite(300.0),
                height: AvailableSpace::Definite(300.0),
            },
        );

        let click_point = (10.0, 25.0);

        let container_rect = abs_rect_of(&tree, scroll_container, 0.0, 0.0);
        assert!(
            !container_rect.contains(click_point.0, click_point.1),
            "click point must be outside the scroll container's own rect"
        );
        let item_rect = abs_rect_of(&tree, item, container_rect.x, container_rect.y - 300.0);
        assert!(
            item_rect.contains(click_point.0, click_point.1),
            "the scrolled item's shifted rect must overlap the click point"
        );

        let hit = hit_test(&tree, root, 0.0, 0.0, click_point.0, click_point.1);
        assert_eq!(
            hit,
            Some(search),
            "the search box, not the scrolled-out-of-view item, should be hit"
        );

        let handler = dispatch_node(
            &mut tree,
            root,
            0.0,
            0.0,
            &UiEvent::MouseDown {
                x: click_point.0,
                y: click_point.1,
                button: MouseButton::Left,
                shift: false,
            },
        );
        assert_eq!(
            handler,
            Some(search),
            "the click should actually reach the search box, not the scrolled item"
        );
    }
}

#[cfg(test)]
mod dismiss_backdrop_tests {
    use crate::widget::primitives::ButtonWidget;
    use crate::widget::primitives::ClickAreaWidget;
    use crate::widget::primitives::TopLayerWidget;
    use super::*;
    use crate::event::MouseButton;
    use crate::layout;
    use crate::style::ComputedStyle;
    use crate::widget::Widget;
    use crate::widget::primitives::DismissBackdropWidget;
    use std::cell::Cell;
    use std::rc::Rc;
    use taffy::prelude::{FlexDirection, length};
    use taffy::style::{Display, Style as TaffyStyle};
    use taffy::{AvailableSpace, Size as TaffySize};

    struct Plain;
    impl Widget for Plain {}

    struct Panel;
    impl Widget for Panel {
        fn absolute_position_override(&self) -> Option<(f32, f32)> {
            Some((50.0, 50.0))
        }
    }

    fn style(width: f32, height: f32) -> ComputedStyle {
        ComputedStyle {
            layout: TaffyStyle {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                flex_shrink: 0.0,
                size: TaffySize {
                    width: length(width),
                    height: length(height),
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn click(tree: &mut WidgetTree, x: f32, y: f32) {
        for event in [
            UiEvent::MouseDown {
                x,
                y,
                button: MouseButton::Left,
                shift: false,
            },
            UiEvent::MouseUp {
                x,
                y,
                button: MouseButton::Left,
            },
        ] {
            dispatch_event(tree, &event);
        }
    }

    #[test]
    fn a_click_on_the_blank_part_of_the_panel_does_not_dismiss_but_one_outside_does() {
        let dismissed = Rc::new(Cell::new(0));
        let mut tree = WidgetTree::new(Box::new(Plain), style(300.0, 300.0));
        let root = tree.root;
        let counter = dismissed.clone();
        tree.add_child(
            root,
            "div",
            Box::new(DismissBackdropWidget::new(Rc::new(move || {
                counter.set(counter.get() + 1)
            }))),
            style(300.0, 300.0),
        );
        tree.add_child(root, "div", Box::new(Panel), style(100.0, 100.0));
        layout::compute_layout(
            &mut tree,
            TaffySize {
                width: AvailableSpace::Definite(300.0),
                height: AvailableSpace::Definite(300.0),
            },
        );

        click(&mut tree, 80.0, 80.0);
        assert_eq!(dismissed.get(), 0, "inside the panel");
        click(&mut tree, 250.0, 250.0);
        assert_eq!(dismissed.get(), 1, "outside the panel");
    }

    #[test]
    fn a_popover_panel_swallows_the_wheel_over_its_blank_space() {
        let wheel = |tree: &mut WidgetTree, x: f32, y: f32| {
            dispatch_event_with_handler(
                tree,
                &UiEvent::Scroll {
                    x,
                    y,
                    delta_x: 0.0,
                    delta_y: -40.0,
                    precise: true,
                },
            )
            .0
        };
        for (panel, swallowed) in [
            (
                Box::new(ClickAreaWidget::new().with_swallowed_scroll()) as Box<dyn Widget>,
                true,
            ),
            (Box::new(ClickAreaWidget::new()), false),
        ] {
            let mut tree = WidgetTree::new(Box::new(Plain), style(300.0, 300.0));
            let root = tree.root;
            tree.add_child(root, "div", panel, style(100.0, 100.0));
            layout::compute_layout(
                &mut tree,
                TaffySize {
                    width: AvailableSpace::Definite(300.0),
                    height: AvailableSpace::Definite(300.0),
                },
            );
            assert_eq!(
                wheel(&mut tree, 50.0, 50.0) == EventResult::Handled,
                swallowed
            );
            assert_eq!(
                wheel(&mut tree, 250.0, 250.0),
                EventResult::Ignored,
                "outside the panel"
            );
        }
    }

    #[test]
    fn a_button_inside_a_top_layer_panel_receives_the_press() {
        let clicked = Rc::new(Cell::new(0));
        let mut tree = WidgetTree::new(Box::new(Plain), style(300.0, 300.0));
        let root = tree.root;
        let layer = tree.add_child(root, "div", Box::new(TopLayerWidget), style(120.0, 80.0));
        let counter = clicked.clone();
        tree.add_child(
            layer,
            "div",
            Box::new(ButtonWidget::new().with_on_click(move || counter.set(counter.get() + 1))),
            style(40.0, 24.0),
        );
        layout::compute_layout(
            &mut tree,
            TaffySize {
                width: AvailableSpace::Definite(300.0),
                height: AvailableSpace::Definite(300.0),
            },
        );

        click(&mut tree, 10.0, 10.0);
        assert_eq!(clicked.get(), 1, "the press reaches the button");
    }
}

#[cfg(test)]
mod directional_focus_tests {
    use super::*;
    use crate::layout;
    use crate::style::ComputedStyle;
    use crate::widget::Widget;
    use taffy::prelude::{FlexDirection, length};
    use taffy::style::{Display, Style as TaffyStyle};
    use taffy::{AvailableSpace, Size as TaffySize};

    struct Plain;
    impl Widget for Plain {}

    struct Focusable;
    impl Widget for Focusable {
        fn wants_focus(&self) -> bool {
            true
        }
    }

    fn style(direction: FlexDirection, width: f32, height: f32) -> ComputedStyle {
        ComputedStyle {
            layout: TaffyStyle {
                display: Display::Flex,
                flex_direction: direction,
                flex_shrink: 0.0,
                size: TaffySize {
                    width: length(width),
                    height: length(height),
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn build() -> (WidgetTree, WidgetId, [WidgetId; 3]) {
        let mut tree = WidgetTree::new(Box::new(Plain), style(FlexDirection::Column, 300.0, 300.0));
        let root = tree.root;
        let header = tree.add_child(
            root,
            "div",
            Box::new(Focusable),
            style(FlexDirection::Row, 260.0, 30.0),
        );
        let row = tree.add_child(
            root,
            "div",
            Box::new(Plain),
            style(FlexDirection::Row, 300.0, 50.0),
        );
        let tiles = [0; 3].map(|_| {
            tree.add_child(
                row,
                "div",
                Box::new(Focusable),
                style(FlexDirection::Row, 50.0, 50.0),
            )
        });
        layout::compute_layout(
            &mut tree,
            TaffySize {
                width: AvailableSpace::Definite(300.0),
                height: AvailableSpace::Definite(300.0),
            },
        );
        (tree, header, tiles)
    }

    #[test]
    fn arrows_walk_a_row_and_stop_at_its_end() {
        let (mut tree, _, tiles) = build();
        tree.focused = Some(tiles[0]);
        assert!(move_focus_direction(&mut tree, Direction::Right));
        assert_eq!(tree.focused, Some(tiles[1]));
        assert!(move_focus_direction(&mut tree, Direction::Right));
        assert_eq!(tree.focused, Some(tiles[2]));

        assert!(!move_focus_direction(&mut tree, Direction::Right));
        assert_eq!(tree.focused, Some(tiles[2]));
        assert!(move_focus_direction(&mut tree, Direction::Left));
        assert_eq!(tree.focused, Some(tiles[1]));
    }

    #[test]
    fn up_and_down_move_between_the_header_and_the_row() {
        let (mut tree, header, tiles) = build();
        tree.focused = Some(header);
        assert!(move_focus_direction(&mut tree, Direction::Down));
        assert!(tiles.contains(&tree.focused.unwrap()));
        assert!(move_focus_direction(&mut tree, Direction::Up));
        assert_eq!(tree.focused, Some(header));
        assert!(
            !move_focus_direction(&mut tree, Direction::Up),
            "nothing above the header"
        );
    }

    #[test]
    fn nothing_focused_means_nowhere_to_move_from() {
        let (mut tree, _, _) = build();
        assert!(!move_focus_direction(&mut tree, Direction::Down));
    }
}

#[cfg(test)]
mod ellipse_paint_tests {
    use crate::style::cascade;
    use crate::style::parser;
    use skia_safe::surfaces;
    use super::*;
    use crate::layout;
    use crate::style::{ComputedStyle, Filter};
    use crate::widget::primitives::BoxWidget;
    use taffy::prelude::{FlexDirection, length};
    use taffy::style::{Display, Style as TaffyStyle};
    use taffy::{AvailableSpace, Size as TaffySize};

    fn sized(width: f32, height: f32) -> ComputedStyle {
        ComputedStyle {
            layout: TaffyStyle {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                flex_shrink: 0.0,
                size: TaffySize {
                    width: length(width),
                    height: length(height),
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// White 300x120 with a 200x80 black-at-52% node painted at its top-left.
    fn painted(ellipse: bool, blur: f32) -> skia_safe::Surface {
        let mut root_style = sized(300.0, 120.0);
        root_style.background = Some(Color::WHITE);
        let mut tree = WidgetTree::new(Box::new(BoxWidget), root_style);
        let mut style = sized(200.0, 80.0);
        style.background = Some(Color::from_argb(133, 0, 0, 0));
        style.corner_ellipse = ellipse;
        if blur > 0.0 {
            style.filter = Some(Filter {
                blur,
                brightness: 1.0,
                drop_shadow: None,
            });
        }
        let root = tree.root;
        tree.add_child(root, "div", Box::new(BoxWidget), style);
        layout::compute_layout(
            &mut tree,
            TaffySize {
                width: AvailableSpace::Definite(300.0),
                height: AvailableSpace::Definite(120.0),
            },
        );
        let mut surface = surfaces::raster_n32_premul((300, 120)).expect("surface");
        paint(&mut tree, surface.canvas());
        surface
    }

    fn red(surface: &mut skia_safe::Surface, x: i32, y: i32) -> i32 {
        let info = skia_safe::ImageInfo::new_n32_premul((1, 1), None);
        let mut pixel = [0u8; 4];
        surface.read_pixels(&info, &mut pixel, 4, (x, y));
        i32::from(pixel[2])
    }

    #[test]
    fn an_ellipse_leaves_its_corners_empty() {
        let mut surface = painted(true, 0.0);
        eprintln!("ellipse: corner {} center {}", red(&mut surface, 3, 3), red(&mut surface, 100, 40));
        assert!(red(&mut surface, 3, 3) > 250, "the corner is outside the oval");
        assert!(red(&mut surface, 100, 40) < 140, "the middle is covered");
        let mut rect = painted(false, 0.0);
        assert!(red(&mut rect, 3, 3) < 140, "without the flag it is a plain rectangle");
    }

    #[test]
    fn a_blur_filter_softens_the_edge() {
        let mut sharp = painted(true, 0.0);
        let mut blurred = painted(true, 10.0);
        // Just inside the oval's left tangent point.
        let (sharp_edge, blurred_edge) = (red(&mut sharp, 2, 40), red(&mut blurred, 2, 40));
        eprintln!("edge: sharp {sharp_edge} blurred {blurred_edge}");
        assert!(
            blurred_edge > sharp_edge + 25,
            "the blurred edge ({blurred_edge}) should be lighter than the sharp one ({sharp_edge})"
        );
    }

    #[test]
    fn the_headers_ellipse_css_reaches_the_node() {
        let css = ".ellipse { position: absolute; left: -25%; top: -45px; width: 150%; height: 98px; border-radius: 50%; background-color: #00000085; filter: blur(10px); mix-blend-mode: multiply; }";
        let rules = parser::parse_stylesheet(css);
        assert_eq!(rules.len(), 1, "the rule parses");
        assert_eq!(rules[0].declarations.len(), 9, "every declaration is kept");
        let mut tree = WidgetTree::new(Box::new(BoxWidget), sized(300.0, 120.0));
        let root = tree.root;
        let node = tree.add_child(root, "div", Box::new(BoxWidget), ComputedStyle::default());
        tree.nodes[node].classes.push("ellipse".to_string());
        cascade::apply_stylesheet(&mut tree, &rules);
        let style = &tree.nodes[node].style;
        eprintln!(
            "css: ellipse={} blur={:?} blend={:?} background={:?}",
            style.corner_ellipse,
            style.filter.map(|f| f.blur),
            style.blend_mode,
            style.background
        );
        assert!(style.corner_ellipse, "border-radius: 50% sets the ellipse");
        assert_eq!(style.filter.map(|f| f.blur), Some(10.0));
        assert!(style.blend_mode.is_some());
        assert!(style.background.is_some());
    }

    #[test]
    fn the_ellipse_sits_where_the_css_puts_it() {
        let css = ".host { position: relative; width: 240px; height: 130px; } .clip { position: absolute; left: 0px; top: 0px; width: 100%; height: 50px; overflow: hidden; } .ellipse { position: absolute; left: -25%; top: -45px; width: 150%; height: 98px; border-radius: 50%; background-color: #00000085; filter: blur(10px); mix-blend-mode: multiply; }";
        let rules = parser::parse_stylesheet(css);
        let mut tree = WidgetTree::new(Box::new(BoxWidget), sized(300.0, 200.0));
        let root = tree.root;
        let host = tree.add_child(root, "div", Box::new(BoxWidget), ComputedStyle::default());
        let clip = tree.add_child(host, "div", Box::new(BoxWidget), ComputedStyle::default());
        let ellipse = tree.add_child(clip, "div", Box::new(BoxWidget), ComputedStyle::default());
        tree.nodes[host].classes.push("host".to_string());
        tree.nodes[clip].classes.push("clip".to_string());
        tree.nodes[ellipse].classes.push("ellipse".to_string());
        cascade::apply_stylesheet(&mut tree, &rules);
        layout::compute_layout(
            &mut tree,
            TaffySize {
                width: AvailableSpace::Definite(300.0),
                height: AvailableSpace::Definite(200.0),
            },
        );
        let rect = abs_rect_of(&tree, ellipse, 0.0, 0.0);
        let clip_rect = abs_rect_of(&tree, clip, 0.0, 0.0);
        eprintln!("clip {clip_rect:?} ellipse {rect:?}");
        assert_eq!((clip_rect.width, clip_rect.height), (240.0, 50.0));
        assert_eq!((rect.x, rect.y), (-60.0, -45.0), "inset by a quarter and 45px up");
        assert_eq!((rect.width, rect.height), (360.0, 98.0));
    }

    #[test]
    fn the_blur_runs_past_the_ellipses_edge_and_fades_out() {
        let css = ".root { width: 400px; height: 200px; display: flex; flex-direction: column; background-color: #ffffff; } .blur { width: 360px; height: 158px; display: flex; flex-direction: column; padding: 30px 0px; filter: blur(10px); } .oval { width: 100%; height: 98px; flex-shrink: 0; border-radius: 50%; background-color: #00000085; }";
        let rules = parser::parse_stylesheet(css);
        let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
        let root = tree.root;
        tree.nodes[root].classes.push("root".to_string());
        let blur = tree.add_child(root, "div", Box::new(BoxWidget), ComputedStyle::default());
        let oval = tree.add_child(blur, "div", Box::new(BoxWidget), ComputedStyle::default());
        tree.nodes[blur].classes.push("blur".to_string());
        tree.nodes[oval].classes.push("oval".to_string());
        cascade::apply_stylesheet(&mut tree, &rules);
        layout::compute_layout(
            &mut tree,
            TaffySize {
                width: AvailableSpace::Definite(400.0),
                height: AvailableSpace::Definite(200.0),
            },
        );
        let mut surface = surfaces::raster_n32_premul((400, 200)).expect("surface");
        paint(&mut tree, surface.canvas());
        // The oval's centre column x = 180, its bottom edge at y = 128.
        let column: Vec<i32> = [100, 120, 128, 134, 140, 150, 158].iter().map(|&y| red(&mut surface, 180, y)).collect();
        eprintln!("down the middle (y 100,120,128,134,140,150,158): {column:?}");
        assert!(column[0] < 140, "inside the oval it is dark");
        assert!(column[3] < 250, "6px past the edge the blur still darkens");
        assert!(column[2] > column[0] && column[3] > column[2], "and it fades out going down");
        assert!(column[6] > 245, "30px past the edge it is clear");
    }

    #[test]
    fn a_clipped_boxs_hidden_children_add_no_scrollable_room() {
        let css = ".root { width: 100px; height: 100px; display: flex; flex-direction: column; } .clip { width: 100px; height: 10px; flex-shrink: 0; overflow: hidden; display: flex; flex-direction: column; } .tall { width: 100px; height: 300px; flex-shrink: 0; } .open { width: 100px; height: 10px; flex-shrink: 0; display: flex; flex-direction: column; }";
        let rules = parser::parse_stylesheet(css);
        let build = |clip_class: &str| {
            let mut tree = WidgetTree::new(Box::new(BoxWidget), ComputedStyle::default());
            let root = tree.root;
            tree.nodes[root].classes.push("root".to_string());
            let host = tree.add_child(root, "div", Box::new(BoxWidget), ComputedStyle::default());
            tree.nodes[host].classes.push(clip_class.to_string());
            let tall = tree.add_child(host, "div", Box::new(BoxWidget), ComputedStyle::default());
            tree.nodes[tall].classes.push("tall".to_string());
            cascade::apply_stylesheet(&mut tree, &rules);
            layout::compute_layout(
                &mut tree,
                TaffySize {
                    width: AvailableSpace::Definite(100.0),
                    height: AvailableSpace::Definite(100.0),
                },
            );
            let root = tree.root;
            content_extent(&tree, root)
        };
        assert_eq!(build("clip").1, 10.0, "a closed folder takes its own height only");
        assert_eq!(build("open").1, 300.0, "an unclipped overflow still counts");
    }
}
