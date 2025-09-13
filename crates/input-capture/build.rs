use std::env;

fn main() {
    // Tell cargo to look for shared libraries in the specified directory
    println!(
        "cargo:rustc-link-search=native={}",
        env::var("OUT_DIR").unwrap()
    );

    // Tell cargo to tell rustc to link the C++ file
    println!("cargo:rustc-link-lib=static=set_device_data_format");

    // Link with DirectInput library
    println!("cargo:rustc-link-lib=dinput8");

    // Tell cargo to invalidate the built crate whenever the C++ file changes
    println!("cargo:rerun-if-changed=src/set_device_data_format.cpp");

    // Compile the C++ file
    cc::Build::new()
        .file("src/set_device_data_format.cpp")
        .cpp(true)
        .compile("set_device_data_format");
}
