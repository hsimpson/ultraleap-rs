use std::env;

fn main() {
    let mut dynamic_library_path = "";
    if env::consts::OS == "macos" {
        dynamic_library_path = "/Library/Application Support/Ultraleap/LeapSDK/lib";
    } else if env::consts::OS == "window" {
        dynamic_library_path = "unkonwn_windows";
    } else if env::consts::OS == "linux" {
        dynamic_library_path = "unkonwn_linux";
    }

    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", dynamic_library_path);
}
