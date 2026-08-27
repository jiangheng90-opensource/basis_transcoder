fn main() {
    if std::env::var("DOCS_RS").is_ok() {
        println!("cargo:warning=Skipping native build on docs.rs");
        return;
    }

    if std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default() == "wasm32" {
        println!("cargo:warning=Skipping build.rs on wasm32 target");
        return;
    }

    let mut build = cxx_build::bridge("src/ffi.rs");
    build
        .file("cpp/transcoder_api.cc")
        .file("third_party/basis_universal/transcoder/basisu_transcoder.cpp")
        .include("include")
        .include("third_party/basis_universal/transcoder")
        // Zstd-supercompressed KTX2 levels are not supported (avoids a zstd dependency).
        .define("BASISD_SUPPORT_KTX2_ZSTD", "0")
        // cc picks the right C++17 flag per compiler (/std:c++17 on MSVC,
        // -std=c++17 elsewhere).
        .std("c++17")
        .flag_if_supported("-fvisibility=hidden")
        .flag_if_supported("-fno-strict-aliasing")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-variable")
        .flag_if_supported("-Wno-unused-value")
        .flag_if_supported("-Wno-unused-local-typedefs");

    let target = std::env::var("TARGET").unwrap();
    if target.contains("apple-darwin") {
        build.flag("-mmacosx-version-min=15.5");
    }

    build.compile("basis_transcoder_api");

    println!("cargo:rerun-if-changed=cpp/transcoder_api.cc");
    println!("cargo:rerun-if-changed=include/transcoder_api.h");
    println!("cargo:rerun-if-changed=src/ffi.rs");
}
