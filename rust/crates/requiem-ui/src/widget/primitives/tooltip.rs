use skia_safe::{Canvas, Paint, PaintStyle, PathBuilder, Point};

use crate::components::tooltip::TooltipState;
use crate::style::ComputedStyle;
use crate::widget::{AbsoluteRect, Widget};

pub struct TooltipAnchorWidget {
    state: TooltipState,
}

impl TooltipAnchorWidget {
    pub fn new(state: TooltipState) -> Self {
        Self { state }
    }
}

impl Widget for TooltipAnchorWidget {
    fn hover_path_changed(&mut self, inside: bool) {
        self.state.set_hovered(inside);
    }

    fn press_path_changed(&mut self, inside: bool) {
        if inside {
            self.state.pressed();
        }
    }
}

pub struct TopLayerWidget;

impl Widget for TopLayerWidget {
    fn is_top_layer(&self) -> bool {
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerEdge {
    Bottom,
    Top,
    Left,
    Right,
}

pub struct TooltipPointerWidget {
    edge: PointerEdge,
}

impl TooltipPointerWidget {
    pub fn new(edge: PointerEdge) -> Self {
        Self { edge }
    }
}

pub const CARET_WIDTH: f32 = 16.0;
pub const CARET_HEIGHT: f32 = 10.0;

impl Widget for TooltipPointerWidget {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, style: &ComputedStyle) {
        let degrees = match self.edge {
            PointerEdge::Bottom => 0.0,
            PointerEdge::Top => 180.0,
            PointerEdge::Left => 90.0,
            PointerEdge::Right => -90.0,
        };
        let (center_x, center_y) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
        canvas.save();
        canvas.translate((center_x, center_y));
        canvas.rotate(degrees, None);
        canvas.translate((-CARET_WIDTH / 2.0, -CARET_HEIGHT / 2.0));

        canvas.clip_rect(
            skia_safe::Rect::from_wh(CARET_WIDTH, CARET_HEIGHT),
            None,
            true,
        );

        let mut fill = PathBuilder::new();
        fill.move_to(Point::new(10.3426, 7.0715));
        fill.cubic_to(
            Point::new(9.14163, 8.57272),
            Point::new(6.85837, 8.57272),
            Point::new(5.65739, 7.0715),
        );
        fill.line_to(Point::new(0.0, -0.000244141));
        fill.line_to(Point::new(16.0, -0.000244141));
        fill.close();
        if let Some(color) = style.background {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.set_anti_alias(true);
            canvas.draw_path(&fill.detach(), &paint);
        }
        if let Some(color) = style.border_color {
            let mut stroke = PathBuilder::new();
            stroke.move_to(Point::new(9.93457, 6.84546));
            stroke.cubic_to(
                Point::new(8.93433, 8.06766),
                Point::new(7.06567, 8.06766),
                Point::new(6.06543, 6.84546),
            );
            stroke.line_to(Point::new(0.0546875, -0.500244));
            stroke.line_to(Point::new(15.9453, -0.500244));
            stroke.close();
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.set_anti_alias(true);
            paint.set_style(PaintStyle::Stroke);
            paint.set_stroke_width(1.0);
            canvas.draw_path(&stroke.detach(), &paint);
        }
        canvas.restore();
    }
}
