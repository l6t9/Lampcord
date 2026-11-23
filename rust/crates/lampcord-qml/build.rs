use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};
use std::fs;
use std::path::Path;

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

    let mut files = Vec::new();
    collect_qml(Path::new("qml"), &mut files);
    files.sort();

    // Theme holds the colour/type tokens and is read from everywhere.
    let qml_files = files.into_iter().map(|file| {
        let singleton = file.ends_with("/Theme.qml");
        QmlFile::from(file).singleton(singleton)
    });

    CxxQtBuilder::new_qml_module(QmlModule::new("Lampcord").qml_files(qml_files))
        .files(["src/backend.rs", "src/palette.rs"])
        .qt_module("Quick")
        .build();
}
