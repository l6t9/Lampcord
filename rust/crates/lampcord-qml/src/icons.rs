//! Serves the Compose client's Material Symbols to QML by name, from the
//! tables build.rs extracts out of `shared/.../ui/icons`.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        type IconPaths = super::IconPathsRust;

        /// SVG path on a 960 grid with y in -960..0, or "" if unknown.
        /// `filled` picks Icons.Filled over Icons.Rounded.
        #[qinvokable]
        fn path(self: &IconPaths, name: QString, filled: bool) -> QString;
    }
}

use std::collections::HashMap;
use std::sync::LazyLock;

use cxx_qt_lib::QString;

static FILLED: LazyLock<HashMap<String, String>> =
    LazyLock::new(|| parse(include_str!(concat!(env!("OUT_DIR"), "/icons_filled.json"))));
static ROUNDED: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    parse(include_str!(concat!(
        env!("OUT_DIR"),
        "/icons_rounded.json"
    )))
});

fn parse(table: &str) -> HashMap<String, String> {
    serde_json::from_str(table).expect("icon table is valid JSON")
}

#[derive(Default)]
pub struct IconPathsRust;

impl qobject::IconPaths {
    pub fn path(&self, name: QString, filled: bool) -> QString {
        let name = name.to_string();
        let table = if filled { &*FILLED } else { &*ROUNDED };
        table
            .get(&name)
            .or_else(|| ROUNDED.get(&name))
            .map(|path| QString::from(path.as_str()))
            .unwrap_or_default()
    }
}
