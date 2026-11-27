use crate::ui;
use requiem_ui::style::ComputedStyle;
use requiem_ui::widget::{AbsoluteRect, Widget};
use skia_safe::{Canvas, Paint, Rect};

pub struct NoResultsArt;

thread_local! {
    static IMAGE: Option<skia_safe::Image> = ui::asset_svg("image/search_no_results", 320, 320);
}

impl Widget for NoResultsArt {
    fn paint(&self, canvas: &Canvas, rect: AbsoluteRect, _style: &ComputedStyle) {
        IMAGE.with(|image| {
            let Some(image) = image else { return };
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            let sampling = skia_safe::SamplingOptions::new(
                skia_safe::FilterMode::Linear,
                skia_safe::MipmapMode::Linear,
            );
            canvas.draw_image_rect_with_sampling_options(
                image,
                None,
                Rect::from_xywh(rect.x, rect.y, rect.width, rect.height),
                sampling,
                &paint,
            );
        });
    }
}
