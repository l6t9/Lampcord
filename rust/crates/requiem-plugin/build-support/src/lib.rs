pub fn styles() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by cargo");
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR is set by cargo");
    requiem_css::process_crate(&manifest_dir, &out_dir);
    println!("cargo:rerun-if-changed=src");
}
