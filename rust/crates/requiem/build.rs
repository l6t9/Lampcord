use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set by cargo");
    requiem_css::process_crate(env!("CARGO_MANIFEST_DIR"), &out_dir);
    println!("cargo:rerun-if-changed=src");
    copy_ffmpeg_dlls(&out_dir);
}

fn copy_ffmpeg_dlls(out_dir: &str) {
    let Ok(ffmpeg_dir) = env::var("FFMPEG_DIR") else {
        return;
    };
    println!("cargo:rerun-if-env-changed=FFMPEG_DIR");

    let target_dir = Path::new(out_dir)
        .ancestors()
        .nth(3)
        .expect("OUT_DIR shallower than expected");

    let dll_dir = Path::new(&ffmpeg_dir).join("bin");
    let Ok(entries) = fs::read_dir(&dll_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("dll") {
            let dest = target_dir.join(path.file_name().unwrap());
            let _ = fs::copy(&path, &dest);
        }
    }
}
