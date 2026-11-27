use skia_safe::font_style::Weight;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextVariant {
    pub size: f32,

    pub line_height: f32,
    pub weight: Weight,

    pub uppercase: bool,

    pub letter_spacing: f32,
}

macro_rules! variants {
    ($($name:ident = ($size:expr, $line_height:expr, $weight:expr, $uppercase:expr, $letter_spacing:expr)),+ $(,)?) => {
        impl TextVariant {
            $(pub const $name: TextVariant = TextVariant { size: $size, line_height: $line_height, weight: $weight, uppercase: $uppercase, letter_spacing: $letter_spacing };)+

            const ALL: &'static [(&'static str, TextVariant)] = &[$((stringify!($name), TextVariant::$name)),+];
        }
    };
}

variants! {
    TEXT_XXS_NORMAL = (10.0, 12.0, Weight::NORMAL, false, 0.0),
    TEXT_XXS_MEDIUM = (10.0, 12.0, Weight::MEDIUM, false, 0.0),
    TEXT_XXS_SEMIBOLD = (10.0, 12.0, Weight::SEMI_BOLD, false, 0.0),
    TEXT_XXS_BOLD = (10.0, 12.0, Weight::BOLD, false, 0.0),
    TEXT_XS_NORMAL = (12.0, 16.0, Weight::NORMAL, false, 0.0),
    TEXT_XS_MEDIUM = (12.0, 16.0, Weight::MEDIUM, false, 0.0),
    TEXT_XS_SEMIBOLD = (12.0, 16.0, Weight::SEMI_BOLD, false, 0.0),
    TEXT_XS_BOLD = (12.0, 16.0, Weight::BOLD, false, 0.0),
    TEXT_SM_NORMAL = (14.0, 18.0, Weight::NORMAL, false, 0.0),
    TEXT_SM_MEDIUM = (14.0, 18.0, Weight::MEDIUM, false, 0.0),
    TEXT_SM_SEMIBOLD = (14.0, 18.0, Weight::SEMI_BOLD, false, 0.0),
    TEXT_SM_BOLD = (14.0, 18.0, Weight::BOLD, false, 0.0),
    TEXT_MD_NORMAL = (16.0, 20.0, Weight::NORMAL, false, 0.0),
    TEXT_MD_MEDIUM = (16.0, 20.0, Weight::MEDIUM, false, 0.0),
    TEXT_MD_SEMIBOLD = (16.0, 20.0, Weight::SEMI_BOLD, false, 0.0),
    TEXT_MD_BOLD = (16.0, 20.0, Weight::BOLD, false, 0.0),
    TEXT_LG_NORMAL = (20.0, 24.0, Weight::NORMAL, false, 0.0),
    TEXT_LG_MEDIUM = (20.0, 24.0, Weight::MEDIUM, false, 0.0),
    TEXT_LG_SEMIBOLD = (20.0, 24.0, Weight::SEMI_BOLD, false, 0.0),
    TEXT_LG_BOLD = (20.0, 24.0, Weight::BOLD, false, 0.0),
    HEADING_SM_NORMAL = (14.0, 18.0, Weight::NORMAL, false, 0.0),
    HEADING_SM_MEDIUM = (14.0, 18.0, Weight::MEDIUM, false, 0.0),
    HEADING_SM_SEMIBOLD = (14.0, 18.0, Weight::SEMI_BOLD, false, 0.0),
    HEADING_SM_BOLD = (14.0, 18.0, Weight::BOLD, false, 0.0),
    HEADING_SM_EXTRABOLD = (14.0, 18.0, Weight::EXTRA_BOLD, false, 0.0),
    HEADING_MD_NORMAL = (16.0, 20.0, Weight::NORMAL, false, 0.0),
    HEADING_MD_MEDIUM = (16.0, 20.0, Weight::MEDIUM, false, 0.0),
    HEADING_MD_SEMIBOLD = (16.0, 20.0, Weight::SEMI_BOLD, false, 0.0),
    HEADING_MD_BOLD = (16.0, 20.0, Weight::BOLD, false, 0.0),
    HEADING_MD_EXTRABOLD = (16.0, 20.0, Weight::EXTRA_BOLD, false, 0.0),
    HEADING_LG_NORMAL = (20.0, 24.0, Weight::NORMAL, false, 0.0),
    HEADING_LG_MEDIUM = (20.0, 24.0, Weight::MEDIUM, false, 0.0),
    HEADING_LG_SEMIBOLD = (20.0, 24.0, Weight::SEMI_BOLD, false, 0.0),
    HEADING_LG_BOLD = (20.0, 24.0, Weight::BOLD, false, 0.0),
    HEADING_LG_EXTRABOLD = (20.0, 24.0, Weight::EXTRA_BOLD, false, 0.0),
    HEADING_XL_NORMAL = (24.0, 30.0, Weight::NORMAL, false, 0.0),
    HEADING_XL_MEDIUM = (24.0, 30.0, Weight::MEDIUM, false, 0.0),
    HEADING_XL_SEMIBOLD = (24.0, 30.0, Weight::SEMI_BOLD, false, 0.0),
    HEADING_XL_BOLD = (24.0, 30.0, Weight::BOLD, false, 0.0),
    HEADING_XL_EXTRABOLD = (24.0, 30.0, Weight::EXTRA_BOLD, false, 0.0),
    HEADING_XXL_NORMAL = (32.0, 40.0, Weight::NORMAL, false, 0.0),
    HEADING_XXL_MEDIUM = (32.0, 40.0, Weight::MEDIUM, false, 0.0),
    HEADING_XXL_SEMIBOLD = (32.0, 40.0, Weight::SEMI_BOLD, false, 0.0),
    HEADING_XXL_BOLD = (32.0, 40.0, Weight::BOLD, false, 0.0),
    HEADING_XXL_EXTRABOLD = (32.0, 40.0, Weight::EXTRA_BOLD, false, 0.0),
    DISPLAY_SM = (20.0, 20.0, Weight::EXTRA_BOLD, false, 0.0),
    DISPLAY_MD = (34.0, 36.0, Weight::EXTRA_BOLD, false, 0.0),
    DISPLAY_LG = (44.0, 42.0, Weight::EXTRA_BOLD, false, 0.0),
    EYEBROW = (12.0, 16.0, Weight::BOLD, true, 0.02),
    CODE = (12.0, 16.0, Weight::BOLD, false, 0.0),
}

impl TextVariant {
    pub fn named() -> Vec<(String, TextVariant)> {
        Self::ALL
            .iter()
            .map(|(ident, variant)| {
                let parts: Vec<&str> = ident.split('_').collect();
                let name = match parts.as_slice() {
                    [kind, size, weight] => format!("{kind}-{size}/{weight}"),
                    other => other.join("-"),
                };
                (name.to_lowercase(), *variant)
            })
            .collect()
    }

    pub fn from_name(name: &str) -> Option<TextVariant> {
        Self::named()
            .into_iter()
            .find_map(|(known, variant)| (known == name).then_some(variant))
    }

    pub fn line_height_for(size: f32) -> Option<f32> {
        if (size - size.round()).abs() > 0.01 {
            return None;
        }
        match size.round() as i32 {
            10 => Some(12.0),
            12 => Some(16.0),
            14 => Some(18.0),
            16 => Some(20.0),
            20 => Some(24.0),
            24 => Some(30.0),
            32 => Some(40.0),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_mana_css() {
        let v = TextVariant::TEXT_XXS_NORMAL;
        assert_eq!((v.size, v.line_height), (10.0, 12.0));
        let v = TextVariant::TEXT_XS_MEDIUM;
        assert_eq!((v.size, v.line_height), (12.0, 16.0));
        let v = TextVariant::TEXT_LG_SEMIBOLD;
        assert_eq!((v.size, v.line_height), (20.0, 24.0));
        let v = TextVariant::HEADING_XL_BOLD;
        assert_eq!((v.size, v.line_height), (24.0, 30.0));
        let v = TextVariant::HEADING_XXL_EXTRABOLD;
        assert_eq!(
            (v.size, v.line_height, v.weight),
            (32.0, 40.0, Weight::EXTRA_BOLD)
        );
        assert_eq!(TextVariant::TEXT_MD_SEMIBOLD.weight, Weight::SEMI_BOLD);
        assert!(TextVariant::EYEBROW.uppercase);
        assert_eq!(TextVariant::EYEBROW.letter_spacing, 0.02);
    }

    #[test]
    fn variants_are_named_kind_size_slash_weight() {
        assert_eq!(
            TextVariant::from_name("heading-md/semibold"),
            Some(TextVariant::HEADING_MD_SEMIBOLD)
        );
        assert_eq!(
            TextVariant::from_name("text-xxs/normal"),
            Some(TextVariant::TEXT_XXS_NORMAL)
        );
        assert_eq!(TextVariant::from_name("display-lg"), Some(TextVariant::DISPLAY_LG));
        assert_eq!(TextVariant::from_name("eyebrow"), Some(TextVariant::EYEBROW));
        assert_eq!(TextVariant::from_name("text-md-semibold"), None);
        assert_eq!(TextVariant::from_name("TEXT_MD_NORMAL"), None);
    }

    #[test]
    fn bare_sizes() {
        assert_eq!(TextVariant::line_height_for(14.0), Some(18.0));
        assert_eq!(TextVariant::line_height_for(13.0), None);
        assert_eq!(TextVariant::line_height_for(24.0), Some(30.0));
    }
}
