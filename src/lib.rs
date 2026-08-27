//! # basis_transcoder
//!
//! A Rust library for transcoding KTX2 Basis Universal textures (ETC1S/UASTC)
//! to GPU formats, with native and WebAssembly support.
//!
//! - Native: the official Binomial `basis_universal` C++ transcoder, vendored
//!   under `third_party/basis_universal` and compiled via `cc`/cxx bridge.
//! - WASM: an Emscripten build of the same transcoder running in a Web Worker,
//!   bundled as an embedded JavaScript module.
//!
//! ## Example
//!
//! ```ignore
//! use basis_transcoder::{transcode_ktx2, TargetFormat};
//!
//! let data: &[u8] = /* your KTX2 encoded data */;
//! if let Some(texture) = transcode_ktx2(data, TargetFormat::Bc7Rgba).await {
//!     println!("{}x{}, {} mip levels", texture.info.width, texture.info.height, texture.levels.len());
//! }
//! ```

#[cfg(not(target_arch = "wasm32"))]
mod ffi;
pub mod utils;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub use utils::{LevelData, TargetFormat, TextureInfo, TranscodedTexture};

/// Transcodes a KTX2 Basis Universal texture asynchronously.
///
/// Every mip level is transcoded to `target` and returned tightly packed in
/// mip-major order. A fresh transcoder instance is created per call: the C++
/// `ktx2_transcoder` binds to the input bytes and cannot be reused across
/// files.
///
/// Returns `None` if the data is not a supported KTX2 Basis Universal texture,
/// the source codec cannot produce `target`, or the texture is HDR/video.
#[cfg(not(target_arch = "wasm32"))]
pub async fn transcode_ktx2(data: &[u8], target: TargetFormat) -> Option<TranscodedTexture> {
    ffi::transcode_ktx2(data, target)
}

/// Transcodes a KTX2 Basis Universal texture synchronously (native only).
#[cfg(not(target_arch = "wasm32"))]
pub fn transcode_ktx2_sync(data: &[u8], target: TargetFormat) -> Option<TranscodedTexture> {
    ffi::transcode_ktx2(data, target)
}

/// Transcodes a KTX2 Basis Universal texture asynchronously (WASM).
///
/// Uses the embedded JavaScript transcoder module running in a Web Worker.
#[cfg(target_arch = "wasm32")]
pub async fn transcode_ktx2(data: &[u8], target: TargetFormat) -> Option<TranscodedTexture> {
    wasm::transcode_ktx2_wasm_worker(data, target).await
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    // ETC1S (BasisLZ) texture atlas extracted from a real Cesium ion Japan 3D
    // city b3dm tile (asset 2602291).
    const SAMPLE: &[u8] = include_bytes!("../testdata/texture_0.ktx2");

    #[tokio::test]
    async fn test_transcode_etc1s_to_rgba32() {
        let texture = transcode_ktx2(SAMPLE, TargetFormat::Rgba32)
            .await
            .expect("transcode should succeed");

        assert!(texture.info.is_etc1s);
        assert!(texture.info.is_srgb);
        assert!(!texture.info.has_alpha);
        assert_eq!((texture.info.width, texture.info.height), (16, 28));
        assert_eq!(texture.info.levels, 5);

        let level0 = &texture.levels[0];
        assert_eq!((level0.width, level0.height), (16, 28));
        assert_eq!(level0.data.len(), 16 * 28 * 4);

        // Mip dimensions halve each level, clamped to 1 texel.
        for (i, level) in texture.levels.iter().enumerate() {
            let w = (16u32 >> i).max(1);
            let h = (28u32 >> i).max(1);
            assert_eq!((level.width, level.height), (w, h));
            assert_eq!(level.data.len(), (w * h * 4) as usize);
        }

        // The decode must not be a solid color.
        let first = &level0.data[0..4];
        assert!(level0.data.chunks_exact(4).any(|p| p != first));
    }

    #[tokio::test]
    async fn test_transcode_etc1s_to_bc7() {
        let texture = transcode_ktx2(SAMPLE, TargetFormat::Bc7Rgba)
            .await
            .expect("BC7 transcode should succeed");

        // 16x28 -> 4x7 blocks of 16 bytes each at level 0.
        let level0 = &texture.levels[0];
        assert_eq!(level0.data.len(), 4 * 7 * 16);
    }

    #[tokio::test]
    async fn test_reject_garbage() {
        assert!(transcode_ktx2(b"not a ktx2 file", TargetFormat::Rgba32).await.is_none());
        assert!(transcode_ktx2(&[], TargetFormat::Rgba32).await.is_none());
    }
}
