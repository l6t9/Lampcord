use skia_safe::Color;

pub fn role_color(rgb: i64) -> Option<Color> {
    (rgb != 0).then(|| Color::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8))
}

pub fn from_id(id: u64) -> Color {
    let r = 70 + (id % 130) as u8;
    let g = 70 + ((id >> 8) % 130) as u8;
    let b = 90 + ((id >> 16) % 130) as u8;
    Color::from_argb(255, r, g, b)
}
