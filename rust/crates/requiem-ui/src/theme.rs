use crate::fonts;
use skia_safe::font_style::{Slant, Weight, Width};
use skia_safe::textlayout::TypefaceFontProvider;
use skia_safe::{Data, FontMgr, FontStyle, Typeface};

use crate::style::parser::{self, StyleRule};

pub const DEFAULT_CSS: &str = include_str!("../themes/default.css");

pub fn load_default() -> Vec<StyleRule> {
    parser::parse_stylesheet(DEFAULT_CSS)
}

pub const FONT_FAMILIES: &[&str] = &[
    "gg sans",
    "Noto Sans",
    "Helvetica Neue",
    "Helvetica",
    "Arial",
];

const GG_SANS_ALIAS: &str = "gg sans";

const GG_SANS_MONO_ALIAS: &str = "gg sans mono";

fn embedded_font_provider() -> TypefaceFontProvider {
    let parser = FontMgr::default();
    let mut provider = TypefaceFontProvider::new();
    for bytes in fonts::gg_sans() {
        if let Some(typeface) = parser.new_from_data(Data::new_copy(bytes), None) {
            provider.register_typeface(typeface, Some(GG_SANS_ALIAS));
        }
    }
    for bytes in fonts::gg_sans_mono() {
        if let Some(typeface) = parser.new_from_data(Data::new_copy(bytes), None) {
            provider.register_typeface(typeface, Some(GG_SANS_MONO_ALIAS));
        }
    }
    provider
}

pub fn make_mono_typeface(weight: Weight) -> Typeface {
    let style = FontStyle::new(weight, Width::NORMAL, Slant::Upright);
    embedded_font_provider()
        .match_family_style(GG_SANS_MONO_ALIAS, style)
        .unwrap_or_else(|| make_typeface(weight))
}

pub fn make_typeface(weight: Weight) -> Typeface {
    let style = FontStyle::new(weight, Width::NORMAL, Slant::Upright);
    let system = FontMgr::default();
    embedded_font_provider()
        .match_family_style(GG_SANS_ALIAS, style)
        .or_else(|| {
            FONT_FAMILIES
                .iter()
                .find_map(|family| system.match_family_style(*family, style))
        })
        .unwrap_or_else(|| {
            system
                .legacy_make_typeface(None, style)
                .expect("font manager returned no typeface even for its own platform default")
        })
}
