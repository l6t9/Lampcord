use crate::fonts;
use crate::perf;
use std::cell::RefCell;
use std::rc::Rc;

use cosmic_text::FontSystem;
use cosmic_text::fontdb::Weight;

use crate::glyph_cache::GlyphCache;

#[derive(Clone)]
pub struct TextFonts {
    pub font_system: Rc<RefCell<FontSystem>>,
    pub glyph_cache: Rc<GlyphCache>,
}

impl TextFonts {
    pub fn new(glyph_cache_bytes: u64) -> Self {
        let glyph_cache = Rc::new(GlyphCache::new(glyph_cache_bytes));
        let probe = glyph_cache.clone();
        perf::register_mem_probe(move || {
            let (n, bytes, images, pixels) = probe.stats();
            format!(
                "glyphs: {n} cached {} | {images} skia images {}",
                perf::mb(bytes as usize),
                perf::mb(pixels)
            )
        });
        Self {
            font_system: Rc::new(RefCell::new(make_font_system())),
            glyph_cache,
        }
    }

    pub fn install_new(glyph_cache_bytes: u64) -> Self {
        let fonts = Self::new(glyph_cache_bytes);
        install_ambient(fonts.clone());
        fonts
    }
}

thread_local! {
    static AMBIENT: RefCell<Option<TextFonts>> = const { RefCell::new(None) };
}

pub fn install_ambient(fonts: TextFonts) {
    AMBIENT.with(|a| *a.borrow_mut() = Some(fonts));
}

pub fn resolve(explicit: Option<TextFonts>) -> TextFonts {
    explicit.or_else(|| AMBIENT.with(|a| a.borrow().clone())).expect("no fonts: pass `fonts` explicitly or call requiem_ui::font_system::install_ambient at startup")
}

pub const GG_SANS_MONO_FAMILY: &str = "gg sans mono";

pub fn make_font_system() -> FontSystem {
    let mut system = FontSystem::new();
    let db = system.db_mut();
    for bytes in fonts::gg_sans()
        .iter()
        .chain(fonts::gg_sans_mono())
    {
        db.load_font_data(bytes.clone());
    }
    system
}

pub fn gg_sans_family(weight: Weight, italic: bool) -> &'static str {
    if !italic {
        return "gg sans";
    }
    match weight.0 {
        500 => "gg sans Medium",
        600 => "gg sans SemiBold",
        800 => "gg sans ExtraBold",

        _ => "gg sans Normal",
    }
}
