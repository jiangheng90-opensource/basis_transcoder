use crate::utils::{LevelData, TargetFormat, TranscodedTexture};

#[cxx::bridge]
mod cpp {
    struct TextureInfo {
        width: u32,
        height: u32,
        levels: u32,
        faces: u32,
        layers: u32,
        has_alpha: u32,
        is_etc1s: u32,
        is_uastc: u32,
        is_srgb: u32,
        is_hdr: u32,
        is_video: u32,
    }

    unsafe extern "C++" {
        include!("transcoder_api.h");

        type Ktx2Transcoder;

        fn create_transcoder(data: &[u8]) -> UniquePtr<Ktx2Transcoder>;

        fn get_texture_info(transcoder: &Ktx2Transcoder, out: &mut TextureInfo) -> bool;

        fn get_level_output_size(
            transcoder: &Ktx2Transcoder,
            level: u32,
            format: u32,
            out_width: &mut u32,
            out_height: &mut u32,
        ) -> usize;

        unsafe fn transcode_level(
            transcoder: &Ktx2Transcoder,
            level: u32,
            format: u32,
            out_ptr: *mut u8,
            out_len: usize,
        ) -> bool;
    }
}

impl From<cpp::TextureInfo> for crate::TextureInfo {
    fn from(cpp: cpp::TextureInfo) -> Self {
        Self {
            width: cpp.width,
            height: cpp.height,
            levels: cpp.levels,
            faces: cpp.faces,
            layers: cpp.layers,
            has_alpha: cpp.has_alpha != 0,
            is_etc1s: cpp.is_etc1s != 0,
            is_uastc: cpp.is_uastc != 0,
            is_srgb: cpp.is_srgb != 0,
            is_hdr: cpp.is_hdr != 0,
            is_video: cpp.is_video != 0,
        }
    }
}

pub fn transcode_ktx2(data: &[u8], target: TargetFormat) -> Option<TranscodedTexture> {
    let transcoder = cpp::create_transcoder(data);
    if transcoder.is_null() {
        return None;
    }

    let mut cpp_info = cpp::TextureInfo {
        width: 0,
        height: 0,
        levels: 0,
        faces: 0,
        layers: 0,
        has_alpha: 0,
        is_etc1s: 0,
        is_uastc: 0,
        is_srgb: 0,
        is_hdr: 0,
        is_video: 0,
    };
    if !cpp::get_texture_info(&transcoder, &mut cpp_info) {
        return None;
    }

    let info: crate::TextureInfo = cpp_info.into();

    // HDR sources only transcode to HDR targets we don't expose; ETC1S video
    // frames carry cross-frame state and need ordered stateful decoding.
    if info.is_hdr || info.is_video {
        return None;
    }

    let mut levels = Vec::with_capacity(info.levels as usize);
    for level in 0..info.levels {
        let (mut width, mut height) = (0u32, 0u32);
        let size = cpp::get_level_output_size(&transcoder, level, target.as_u32(), &mut width, &mut height);
        if size == 0 {
            return None;
        }

        let mut buffer = vec![0u8; size];
        let ok = unsafe {
            cpp::transcode_level(&transcoder, level, target.as_u32(), buffer.as_mut_ptr(), buffer.len())
        };
        if !ok {
            return None;
        }

        levels.push(LevelData {
            width,
            height,
            data: buffer,
        });
    }

    Some(TranscodedTexture {
        info,
        format: target,
        levels,
    })
}
