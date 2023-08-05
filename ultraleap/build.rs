use std::env;
use std::path::PathBuf;

fn main() {
    // This is the directory where the `c` library is located.
    let mut leapc_dir_path = "";
    if env::consts::OS == "macos" {
        leapc_dir_path = "/Library/Application Support/Ultraleap/LeapSDK";
    } else if env::consts::OS == "window" {
        leapc_dir_path = "unkonwn";
    }

    let leap_sdk_dir = PathBuf::from(leapc_dir_path)
        // Canonicalize the path as `rustc-link-search` requires an absolute
        // path.
        .canonicalize()
        .expect("cannot canonicalize path");

    // This is the path to the `c` headers file.
    let headers_path = leap_sdk_dir.join("include/LeapC.h");
    let headers_path_str = headers_path.to_str().expect("Path is not a valid string");

    // This is the path where the library is located.
    let lib_path = leap_sdk_dir.join("lib");

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search={}", lib_path.to_str().unwrap());

    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    println!("cargo:rustc-link-lib=LeapC");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=headers_path_str");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(headers_path_str)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
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
