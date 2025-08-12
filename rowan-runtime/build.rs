fn main() {
    if let Some(os) = std::env::var_os("CARGO_CFG_TARGET_OS") {
        if os == "macos" {
            println!("cargo:rustc-link-lib=framework=System");
            return;
        }
    }

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

