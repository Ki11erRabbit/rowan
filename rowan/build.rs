use std::env;
use std::path::PathBuf;

fn main() {
    let workspace_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .to_path_buf();

    let lib_name = "rowan_runtime";
    let lib_path = workspace_dir
        .join("target")
        .join("debug")
        .join(format!(
            "{}{}{}",
            lib_prefix(),
            lib_name,
            lib_suffix()
        ));

    // Tell Cargo to rerun the build if the actual .so file changes
    println!("cargo:rerun-if-changed={}", lib_path.display());

    println!("cargo:rustc-link-search=native={}", lib_path.parent().unwrap().display());
    println!("cargo:rustc-link-lib=dylib={}", lib_name);

    // Linking Libunwind
    println!("cargo:rustc-link-lib=unwind");
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    match target_arch.as_str() {
        "x86_64" => println!("cargo:rustc-link-lib=unwind-x86_64"),
        "aarch64" => println!("cargo:rustc-link-lib=unwind-aarch64"),
        "arm" => println!("cargo:rustc-link-lib=unwind-arm"),
        _ => panic!("Unsupported architecture: {}", target_arch),
    }
}

fn lib_prefix() -> &'static str {
    if cfg!(target_os = "windows") {
        ""
    } else {
        "lib"
    }
}

fn lib_suffix() -> &'static str {
    if cfg!(target_os = "windows") {
        ".dll"
    } else if cfg!(target_os = "macos") {
        ".dylib"
    } else {
        ".so"
    }
}
