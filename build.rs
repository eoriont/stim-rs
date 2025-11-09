use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("cargo:rerun-if-changed=src/ffi/shim.cc");
    println!("cargo:rerun-if-changed=include/stim_rs/shim.h");
    println!("cargo:rerun-if-changed=stim/CMakeLists.txt");
    println!("cargo:rerun-if-changed=stim/src");
    println!("cargo:rerun-if-changed=stim/file_lists");
    println!("cargo:rerun-if-env-changed=STIM_RS_SIMD_WIDTH");

    let simd_width = env::var("STIM_RS_SIMD_WIDTH").ok();

    let mut cmake_config = cmake::Config::new("stim");
    cmake_config.profile(if cfg!(debug_assertions) {
        "Debug"
    } else {
        "Release"
    });
    cmake_config.define("BUILD_SHARED_LIBS", "OFF");
    if let Some(width) = simd_width.as_deref() {
        cmake_config.define("SIMD_WIDTH", width);
    }
    let dst = cmake_config.build();

    let lib_dir = dst.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=stim");

    let target = env::var("TARGET").unwrap();
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=dylib=c++");
    } else if !target.contains("msvc") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }

    let mut bridge = cxx_build::bridge("src/ffi.rs");
    bridge
        .file("src/ffi/shim.cc")
        .flag_if_supported("-std=c++20")
        .include("include")
        .include("stim/src");

    if let Some(width) = simd_width.as_deref() {
        bridge.define("STIM_RS_SIMD_WIDTH", Some(width));
    }

    bridge.compile("stimrs_ffi");
}
