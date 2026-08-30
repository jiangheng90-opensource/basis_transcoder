# basis_transcoder

Rust library for transcoding KTX2 [Basis Universal](https://github.com/BinomialLLC/basis_universal) textures (ETC1S / UASTC), with a unified API on native and WebAssembly (`wasm32-unknown-unknown`).

## How it works

- **Native**: `cxx` FFI over the vendored `basis_universal` transcoder (git submodule, `v2_50`). Only the transcoder is compiled (`transcoder/basisu_transcoder.cpp`); the encoder is not. zstd-supercompressed KTX2 levels are disabled (`BASISD_SUPPORT_KTX2_ZSTD=0`).
- **WASM**: the official prebuilt Emscripten transcoder (`basis_universal/webgl/transcoder`) is bundled by vite into self-contained ES modules (`javascript/index.es.js` + `javascript/core.es.js`): the wasm binary is inlined as a data URL. By default transcoding runs in a dedicated inline Web Worker (`transcode_ktx2`); `transcode_ktx2_local` runs the same transcode in the current context for hosts that already run inside their own worker. The bundles are committed and embedded into the Rust wasm via `include_str!`, then loaded at runtime through a Blob + dynamic `import()`. Consumers need no extra toolchain, file serving, or copy steps.

The two implementations expose the same API; the crate switches between them with `cfg(target_arch = "wasm32")`.

## API

```rust
use basis_transcoder::{TargetFormat, transcode_ktx2};

// Available on native and wasm32.
if let Some(texture) = transcode_ktx2(&ktx2_bytes, TargetFormat::Rgba32).await {
    println!("{}x{}, {} mip levels", texture.info.width, texture.info.height, texture.info.levels);
    for level in &texture.levels {
        // level.width, level.height, level.data (tightly packed)
    }
}
```

```rust
use basis_transcoder::{TargetFormat, transcode_ktx2_sync};

// Native only, synchronous.
let texture = transcode_ktx2_sync(&ktx2_bytes, TargetFormat::Bc7Rgba);
```

```rust
use basis_transcoder::{TargetFormat, transcode_ktx2_local};

// WASM only: transcodes in the CURRENT context — no dedicated worker is
// spawned. For hosts that already run inside their own Web Worker (e.g. a
// worker pool); on the main thread, prefer `transcode_ktx2`.
let texture = transcode_ktx2_local(&ktx2_bytes, TargetFormat::Rgba32).await;
```

- `TargetFormat` is a `repr(u32)` enum whose values match `basist::transcoder_texture_format` (e.g. `Rgba32 = 13`), with `as_u32`/`from_u32` round-trip helpers.
- `TranscodedTexture { info: TextureInfo, format, levels: Vec<LevelData> }` — one `LevelData { width, height, data }` per mip level.
- Decoding fails (`None`) for invalid input, HDR, and video textures.

## Limitations

- 2D textures only (layer 0, face 0).
- No zstd-supercompressed KTX2 levels.
- No HDR / video sources.

## Build

```sh
git submodule update --init   # third_party/basis_universal
cargo build
```

The native build compiles C++ with `cc`/`cxx-build`; a C++17 compiler is required (CI covers MSVC on Windows, clang on macOS, gcc on Ubuntu). No CMake needed.

## Rebuild the wasm bundle

Only needed when the upstream Emscripten artifacts change:

```sh
npm --prefix third_party/basis_transcoder_js ci
npm run build   # syncs vendored artifacts, vite build, writes javascript/index.es.js + core.es.js
```

## Verification

```sh
cargo test                 # native unit tests (testdata/texture_0.ktx2)
npm run test:js            # node tests for the bundled transcoder
cargo run --example decode_to_png -- testdata/texture_0.ktx2
                           # writes one PNG per mip level to output/ (gitignored)
```

## CI

GitHub Actions (`.github/workflows/ci.yml`): fmt/clippy/check + tests on ubuntu/macos/windows, JS bundle tests and rebuild, wasm32 check + wasm-pack build, release build.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
