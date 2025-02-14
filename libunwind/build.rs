use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to look for shared libraries in the specified directory
    let var = env::var("LIBUNWIND_PATH").unwrap();
    println!("cargo:rustc-link-search={}", var);

    // Tell cargo to tell rustc to link the system libunwind
    // shared library.
    println!("cargo:rustc-link-lib=lunwind");

    //let libc = env::var("LIBC_INCLUDES_PATH").unwrap();
    //let gcc = env::var("GCC_INCLUDES_PATH").unwrap();
    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        //.clang_arg("-I/usr/local/include")
        //.clang_arg(format!("-I{}", gcc))
        //.clang_arg(format!("-I{}", libc))
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
