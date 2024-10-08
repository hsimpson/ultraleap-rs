use std::env;

fn main() {
    if env::consts::OS == "macos" {
        let dynamic_library_path = "/Applications/Ultraleap Hand Tracking.app/Contents/LeapSDK/lib";
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", dynamic_library_path);
    } else if env::consts::OS == "linux" {
        let dynamic_library_path = "unkonwn_linux";
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", dynamic_library_path);
    }
}
