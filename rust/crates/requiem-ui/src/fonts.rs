use std::sync::OnceLock;

#[derive(Debug, Default, Clone)]
pub struct FontFiles {
    pub gg_sans: Vec<Vec<u8>>,

    pub gg_sans_mono: Vec<Vec<u8>>,
}

static FILES: OnceLock<FontFiles> = OnceLock::new();

pub fn install(files: FontFiles) -> bool {
    FILES.set(files).is_ok()
}

pub fn gg_sans() -> &'static [Vec<u8>] {
    FILES.get().map_or(&[], |f| &f.gg_sans)
}

pub fn gg_sans_mono() -> &'static [Vec<u8>] {
    FILES.get().map_or(&[], |f| &f.gg_sans_mono)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_installed_means_no_files_not_a_panic() {
        assert!(gg_sans().is_empty() && gg_sans_mono().is_empty());
    }
}
