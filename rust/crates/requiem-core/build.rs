use std::env;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set by cargo");
    requiem_css::process_crate(env!("CARGO_MANIFEST_DIR"), &out_dir);
    println!("cargo:rerun-if-changed=src");
}
