use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=locales");
    let root = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("locales");
    let mut locales: Vec<_> = fs::read_dir(&root)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .collect();
    locales.sort_by_key(|e| e.file_name());
    let mut out = String::from("pub const LOCALES: &[(&str, &[&str])] = &[\n");
    for locale in locales {
        let tag = locale.file_name().to_string_lossy().into_owned();
        let mut files: Vec<_> = fs::read_dir(locale.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|x| x == "ftl"))
            .collect();
        files.sort_by_key(|e| e.file_name());
        writeln!(out, "    ({tag:?}, &[").unwrap();
        for file in files {
            let relative = file
                .path()
                .strip_prefix(&root)
                .expect("locale file is under locales/")
                .to_string_lossy()
                .replace('\\', "/");
            writeln!(out, "        include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/locales/{relative}\")),")
                .unwrap();
        }
        out.push_str("    ]),\n");
    }
    out.push_str("];\n");
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("locales.rs"),
        out,
    )
    .unwrap();
}
