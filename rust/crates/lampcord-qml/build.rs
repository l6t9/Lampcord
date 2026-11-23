use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};
use std::fs;
use std::path::{Path, PathBuf};

/// The Material Symbols the Compose client draws, as `Name_Path` constants
/// on a 960 grid. `IconsRoundedFilled` is what Compose calls `Icons.Filled`.
const ICON_SOURCES: [(&str, &str); 2] = [
    ("icons_filled.json", "IconsRoundedFilled.kt"),
    ("icons_rounded.json", "IconsRounded.kt"),
];
const ICON_DIR: &str = "../../../shared/src/commonMain/kotlin/me/lampu/lampcord/shared/ui/icons";

/// Pulls every `const val X_Path = "..."` out of a Compose icon file into a
/// `{"X": "..."}` table, so QML gets the exact same shapes.
fn extract_icons(source: &Path, out: &Path) {
    let text = fs::read_to_string(source).unwrap_or_else(|e| panic!("reading {}: {e}", source.display()));
    let mut entries = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.trim().strip_prefix("internal const val ") else { continue };
        let Some((name, value)) = rest.split_once("_Path = \"") else { continue };
        let Some(path) = value.strip_suffix('"') else { continue };
        entries.push(format!("\"{name}\":\"{path}\""));
    }
    assert!(!entries.is_empty(), "no icon paths found in {}", source.display());
    fs::write(out, format!("{{{}}}", entries.join(","))).expect("writing icon table");
}

/// Every .qml file under `qml/`, relative to the crate root, so the module
/// picks up new components without editing this list.
fn collect_qml(dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_qml(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "qml") {
            out.push(path.to_string_lossy().replace('\\', "/"));
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=qml");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    for (table, source) in ICON_SOURCES {
        let source = Path::new(ICON_DIR).join(source);
        println!("cargo:rerun-if-changed={}", source.display());
        extract_icons(&source, &out_dir.join(table));
    }
    println!("cargo:rerun-if-changed=native");

    let mut files = Vec::new();
    collect_qml(Path::new("qml"), &mut files);
    files.sort();

    // Theme holds the colour/type tokens and is read from everywhere.
    let qml_files = files.into_iter().map(|file| {
        let singleton = file.ends_with("/Theme.qml");
        QmlFile::from(file).singleton(singleton)
    });

    CxxQtBuilder::new_qml_module(QmlModule::new("Lampcord").qml_files(qml_files))
        .files(["src/backend.rs", "src/emoji.rs", "src/icons.rs", "src/palette.rs", "src/surface.rs"])
        .cpp_file("native/surface_format.cpp")
        .include_dir("native")
        .qt_module("Quick")
        .build();
}
