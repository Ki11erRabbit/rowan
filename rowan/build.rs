use std::path::PathBuf;
use std::env;

fn main() {

    let workspace_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .to_path_buf();

    let runtime_target_dir = workspace_dir
        .join("target")
        .join("debug");


    println!("cargo:rerun-if-changed={}", runtime_target_dir.display());
    println!("cargo:rustc-link-search=native={}", runtime_target_dir.display());
    println!("cargo:rustc-link-lib=dylib=rowan_runtime");
}