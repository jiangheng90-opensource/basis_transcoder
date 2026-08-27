// Core KTX2 transcode logic, shared by the inline Web Worker (src/basisWorker.js)
// and the Node test (test/transcode.test.mjs). Nothing in this module touches
// worker globals, so it can run anywhere.
//
// The Emscripten build is the official one, vendored from
// third_party/basis_universal/webgl/transcoder/build into the local
// `basis_transcoder` package by tools/build.js (UMD factory:
// BASIS(moduleArg) => Promise<Module>). The embind bindings used here are
// defined in third_party/basis_universal/webgl/transcoder/basis_wrappers.cpp
// (class KTX2File).

import BASIS from 'basis_transcoder';

// Creates a fresh Emscripten module and initializes the Basis transcoder
// tables (KTX2File asserts initializeBasis() was called first).
// moduleArgs is passed through to the factory, e.g. { wasmBinary } in Node or
// { locateFile } in the browser bundle.
export async function createBasisModule(moduleArgs = {}) {
    const Module = await BASIS(moduleArgs);
    Module.initializeBasis();
    return Module;
}

// Transcodes every mip level of a KTX2 Basis Universal texture.
//
// data:         Uint8Array with the whole .ktx2 file
// targetFormat: basist::transcoder_texture_format value (e.g. 13 = cTFRGBA32),
//               passed straight through to KTX2File.transcodeImage
//
// Returns {
//   width, height, levels, faces, layers,
//   hasAlpha, isEtc1s, isUastc, isSrgb, isHdr, isVideo,  // all 0/1 numbers
//   mips: [{ width, height, data: Uint8Array }]          // one entry per level 0..levels-1
// }
//
// Throws on anything the native side would report as Option::None: invalid or
// empty KTX2 data, HDR sources, ETC1S video (P-frames), startTranscoding or
// per-level transcode failures.
export function transcodeKtx2(Module, data, targetFormat) {
    if (!(data instanceof Uint8Array)) {
        data = new Uint8Array(data);
    }

    const file = new Module.KTX2File(data);
    try {
        // Note: upstream's ktx2_file constructor leaves m_is_valid set even
        // when ktx2_transcoder::init() fails, so a bogus header can still pass
        // isValid(); the width/levels check below catches those files.
        if (!file.isValid()) {
            throw new Error('basis_transcoder: invalid KTX2 data');
        }

        const width = file.getWidth();
        const height = file.getHeight();
        const levels = file.getLevels();
        const faces = file.getFaces();
        const layers = file.getLayers();

        // embind bools arrive as JS booleans; the contract wants 0/1 numbers.
        const hasAlpha = file.getHasAlpha() ? 1 : 0;
        const isEtc1s = file.isETC1S() ? 1 : 0;
        const isUastc = file.isUASTC() ? 1 : 0;
        // KTX2File.isSRGB() reads the DFD transfer function, matching the
        // native side's ktx2_transcoder::is_srgb().
        const isSrgb = file.isSRGB() ? 1 : 0;
        const isHdr = file.isHDR() ? 1 : 0;
        const isVideo = file.isVideo() ? 1 : 0;

        if (!width || !height || !levels) {
            throw new Error('basis_transcoder: empty or uninitialized KTX2 image');
        }
        if (isHdr) {
            throw new Error('basis_transcoder: HDR KTX2 textures are not supported');
        }
        if (isVideo) {
            throw new Error('basis_transcoder: KTX2 video (P-frames) is not supported');
        }

        if (!file.startTranscoding()) {
            throw new Error('basis_transcoder: startTranscoding failed');
        }

        // 2D textures only: layer 0, face 0 (same as the native transcode_level).
        const mips = [];
        for (let level = 0; level < levels; level++) {
            const info = file.getImageLevelInfo(level, 0, 0);
            const size = file.getImageTranscodedSizeInBytes(level, 0, 0, targetFormat);
            if (!size) {
                throw new Error(
                    'basis_transcoder: level ' + level +
                    ' cannot be transcoded to format ' + targetFormat
                );
            }

            const dst = new Uint8Array(size);
            // transcodeImage(dst, level, layer, face, format,
            //                getAlphaForOpaqueFormats, channel0, channel1)
            if (!file.transcodeImage(dst, level, 0, 0, targetFormat, 0, -1, -1)) {
                throw new Error(
                    'basis_transcoder: transcodeImage failed at level ' + level
                );
            }

            mips.push({ width: info.origWidth, height: info.origHeight, data: dst });
        }

        return {
            width, height, levels, faces, layers,
            hasAlpha, isEtc1s, isUastc, isSrgb, isHdr, isVideo,
            mips,
        };
    } finally {
        file.close();
        file.delete();
    }
}
