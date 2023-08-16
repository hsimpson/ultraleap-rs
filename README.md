# ultraleap-rs

ultraleap-rs is a Rust wrapper for the [Ultraleap](https://www.ultraleap.com/) LeapC-SDK. It useses the C library provided by Ultraleap and wraps it in a Rust API.

Projects:

- ulraleap: the wrapper crate library which uses the LeapC-SDK
- cube: a simple example which uses ulraleap wrapper crate to control a cube in a window

To get this running you have to set the environment variable for the dynamic linker before running the example with `cargo run`:

MacOS:

```bash
export DYLD_LIBRARY_PATH=/Library/Application\ Support/Ultraleap/LeapSDK/lib:$DYLD_LIBRARY_PATH
```

Windows:

```bash
#TODO
```

Linux:

```bash
#TODO
```
