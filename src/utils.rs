//! Shared result types for the Basis Universal transcoder.

/// Target GPU texture format for transcoding.
///
/// Values mirror `basist::transcoder_texture_format` in the C++ library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TargetFormat {
    /// ETC1 RGB (alpha dropped). 8 bytes per 4x4 block.
    Etc1Rgb = 0,
    /// ETC2 RGBA. 16 bytes per 4x4 block.
    Etc2Rgba = 1,
    /// BC1 (DXT1) RGB. 8 bytes per 4x4 block.
    Bc1Rgb = 2,
    /// BC3 (DXT5) RGBA. 16 bytes per 4x4 block.
    Bc3Rgba = 3,
    /// BC4 single-channel (red). 8 bytes per 4x4 block.
    Bc4R = 4,
    /// BC5 two-channel (red, green). 16 bytes per 4x4 block.
    Bc5Rg = 5,
    /// BC7 RGBA. 16 bytes per 4x4 block.
    Bc7Rgba = 6,
    /// PVRTC1 4bpp RGB. 8 bytes per 4x4 block.
    Pvrtc1_4Rgb = 8,
    /// PVRTC1 4bpp RGBA. 8 bytes per 4x4 block.
    Pvrtc1_4Rgba = 9,
    /// ASTC 4x4 LDR RGBA. 16 bytes per 4x4 block.
    Astc4x4Rgba = 10,
    /// ATC RGB. 8 bytes per 4x4 block.
    AtcRgb = 11,
    /// ATC RGBA. 16 bytes per 4x4 block.
    AtcRgba = 12,
    /// Uncompressed 32-bit RGBA, raster order. 4 bytes per texel.
    Rgba32 = 13,
    /// Uncompressed 16-bit 5:6:5 RGB. 2 bytes per texel.
    Rgb565 = 14,
    /// Uncompressed 16-bit 5:6:5 BGR. 2 bytes per texel.
    Bgr565 = 15,
    /// Uncompressed 16-bit 4:4:4:4 RGBA. 2 bytes per texel.
    Rgba4444 = 16,
}

impl TargetFormat {
    /// The raw `basist::transcoder_texture_format` value.
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    /// Restores a target format from its raw value (worker transport).
    pub fn from_u32(value: u32) -> Option<Self> {
        Some(match value {
            0 => Self::Etc1Rgb,
            1 => Self::Etc2Rgba,
            2 => Self::Bc1Rgb,
            3 => Self::Bc3Rgba,
            4 => Self::Bc4R,
            5 => Self::Bc5Rg,
            6 => Self::Bc7Rgba,
            8 => Self::Pvrtc1_4Rgb,
            9 => Self::Pvrtc1_4Rgba,
            10 => Self::Astc4x4Rgba,
            11 => Self::AtcRgb,
            12 => Self::AtcRgba,
            13 => Self::Rgba32,
            14 => Self::Rgb565,
            15 => Self::Bgr565,
            16 => Self::Rgba4444,
            _ => return None,
        })
    }

    /// Whether the output is plain pixels rather than compressed blocks.
    pub fn is_uncompressed(self) -> bool {
        matches!(
            self,
            Self::Rgba32 | Self::Rgb565 | Self::Bgr565 | Self::Rgba4444
        )
    }
}

/// Basic information about a transcoded KTX2 texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureInfo {
    pub width: u32,
    pub height: u32,
    pub levels: u32,
    pub faces: u32,
    pub layers: u32,
    pub has_alpha: bool,
    /// Source codec is ETC1S (BasisLZ supercompressed).
    pub is_etc1s: bool,
    /// Source codec is UASTC LDR 4x4.
    pub is_uastc: bool,
    /// The DFD declares an sRGB transfer function.
    pub is_srgb: bool,
    /// Source codec is an HDR variant (cannot transcode to LDR targets).
    pub is_hdr: bool,
    /// ETC1S video (P-frames); stateless decoding is not possible.
    pub is_video: bool,
}

/// One transcoded mip level, tightly packed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelData {
    /// Logical width in texels (may not be a multiple of the block size).
    pub width: u32,
    /// Logical height in texels.
    pub height: u32,
    /// Transcoded bytes: blocks for compressed targets, pixels otherwise.
    pub data: Vec<u8>,
}

/// The result of transcoding a KTX2 Basis Universal texture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscodedTexture {
    pub info: TextureInfo,
    /// The format every level was transcoded to.
    pub format: TargetFormat,
    /// Mip level 0..levels, tightly packed, mip-major order.
    pub levels: Vec<LevelData>,
}
