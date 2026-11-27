use std::env;
use std::fs::File;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process;

use requiem_update::{Error, ExtractRules, extract_zip};
use zip::write::SimpleFileOptions;

fn dir(name: &str) -> PathBuf {
    let d = env::temp_dir().join(format!(
        "requiem-update-stage-{name}-{}",
        process::id()
    ));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn rules() -> ExtractRules {
    ExtractRules {
        required_file: "app.exe".into(),
        forbidden_top: vec!["data".into()],
        max_files: 50,
        max_total: 1 << 20,
    }
}

fn zip_of(path: &PathBuf, entries: &[(&str, &[u8])]) {
    let mut w = zip::ZipWriter::new(File::create(path).unwrap());
    for (name, bytes) in entries {
        w.start_file(*name, SimpleFileOptions::default()).unwrap();
        w.write_all(bytes).unwrap();
    }
    w.finish().unwrap();
}

#[test]
fn a_release_zip_is_extracted_without_its_wrapper_folder() {
    let d = dir("ok");
    let z = d.join("r.zip");
    zip_of(
        &z,
        &[
            ("release-1.0/app.exe", b"exe"),
            ("release-1.0/avcodec.dll", b"dll"),
            ("release-1.0/sub/x.txt", b"x"),
        ],
    );
    let files = extract_zip(&z, &d.join("out"), &rules()).unwrap();
    assert_eq!(files.len(), 3);
    assert_eq!(fs::read(d.join("out/app.exe")).unwrap(), b"exe");
    assert_eq!(fs::read(d.join("out/sub/x.txt")).unwrap(), b"x");
    assert!(!d.join("out.partial").exists());
}

#[test]
fn a_flat_zip_is_extracted_as_is() {
    let d = dir("flat");
    let z = d.join("r.zip");
    zip_of(&z, &[("app.exe", b"exe"), ("a.dll", b"dll")]);
    extract_zip(&z, &d.join("out"), &rules()).unwrap();
    assert!(d.join("out/app.exe").is_file());
}

fn rejected(name: &str, entries: &[(&str, &[u8])], rules: ExtractRules) {
    let d = dir(name);
    let z = d.join("r.zip");
    zip_of(&z, entries);
    let err = extract_zip(&z, &d.join("out"), &rules).unwrap_err();
    assert!(matches!(err, Error::BadArchive(_)), "{name}: {err:?}");
    assert!(
        !d.join("out").exists() && !d.join("out.partial").exists(),
        "{name}: nothing may be left behind"
    );
}

#[test]
fn hostile_archives_are_rejected_and_leave_nothing_behind() {
    rejected(
        "traversal",
        &[("app.exe", b"x"), ("../evil.dll", b"x")],
        rules(),
    );
    rejected(
        "absolute",
        &[("app.exe", b"x"), ("/etc/evil", b"x")],
        rules(),
    );
    rejected(
        "drive",
        &[("app.exe", b"x"), ("C:/evil.dll", b"x")],
        rules(),
    );
    rejected(
        "stream",
        &[("app.exe", b"x"), ("a.dll:evil", b"x")],
        rules(),
    );
    rejected("device", &[("app.exe", b"x"), ("NUL.txt", b"x")], rules());
    rejected(
        "userdata",
        &[("app.exe", b"x"), ("data/app.db", b"x")],
        rules(),
    );
    rejected(
        "userdata-wrapped",
        &[("rel/app.exe", b"x"), ("rel/Data/app.db", b"x")],
        rules(),
    );
    rejected("no-exe", &[("other.dll", b"x")], rules());
    rejected(
        "too-many",
        &[("app.exe", b"x"), ("a", b"x"), ("b", b"x")],
        ExtractRules {
            max_files: 2,
            ..rules()
        },
    );
    rejected(
        "too-big",
        &[("app.exe", &[0u8; 4096])],
        ExtractRules {
            max_total: 1024,
            ..rules()
        },
    );
}

#[test]
fn an_existing_destination_is_refused() {
    let d = dir("exists");
    let z = d.join("r.zip");
    zip_of(&z, &[("app.exe", b"x")]);
    fs::create_dir_all(d.join("out")).unwrap();
    assert!(matches!(
        extract_zip(&z, &d.join("out"), &rules()),
        Err(Error::BadArchive(_))
    ));
}
