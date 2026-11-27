use crate::classes::number_badge as cls;
use skia_safe::Color;

use crate::font_system::TextFonts;
use crate::node::Node;
use crate::typography::TextVariant;

use super::container::div;
use super::text::TextProps;

#[derive(Default)]
pub struct NumberBadgeProps {
    /// Counts of 1000 and up draw as `Nk+`, capped at `9k+`.
    pub count: u32,

    pub fonts: Option<TextFonts>,
}

crate::builder_fields!(NumberBadgeProps {
    count: u32,
    opt fonts: TextFonts,
});

impl NumberBadgeProps {
    pub fn new(count: u32) -> Self {
        Self { count, fonts: None }
    }

    pub fn build(self) -> Node {
        number_badge(self)
    }
}

pub fn badge_width(count: u32) -> f32 {
    match count {
        0..=9 => 16.0,
        10..=99 => 22.0,
        _ => 30.0,
    }
}

pub fn badge_text(count: u32) -> String {
    if count < 1000 {
        count.to_string()
    } else {
        format!("{}k+", (count / 1000).min(9))
    }
}

pub fn number_badge(props: NumberBadgeProps) -> Node {
    let mut text =
        TextProps::styled(badge_text(props.count), TextVariant::EYEBROW).color(Color::WHITE);
    if let Some(fonts) = props.fonts {
        text = text.fonts(fonts);
    }
    div()
        .class(cls::NUMBER_BADGE)
        .style(format!("width: {}px", badge_width(props.count)))
        .child(text.build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_width_grows_with_the_digits() {
        assert_eq!(
            [1, 9, 10, 99, 100, 999, 5000].map(badge_width),
            [16.0, 16.0, 22.0, 22.0, 30.0, 30.0, 30.0]
        );
    }

    #[test]
    fn a_thousand_or_more_is_shown_as_thousands_capped_at_nine() {
        assert_eq!(
            [1, 999, 1000, 1999, 9000, 15000].map(badge_text),
            ["1", "999", "1k+", "1k+", "9k+", "9k+"]
        );
    }
}
