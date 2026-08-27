// Dedicated decode worker. Bundled inline (`?worker&inline`) by vite, so the
// final ES module spawns it from a Blob without any extra assets. All
// communication goes through this worker's own message channel — the host
// context's global `self.onmessage` is never touched (the host may itself be
// another Web Worker with its channel already in use).

import wasmUrl from 'basis_transcoder/basis_transcoder.wasm?url';
import { createBasisModule, transcodeKtx2 } from './transcode.js';

let modulePromise = null;

// Lazily creates the Emscripten module once per worker and reuses it for
// every subsequent decode. The wasm binary is the inlined data: URL.
function getModule() {
    if (!modulePromise) {
        modulePromise = createBasisModule({
            locateFile: (path) => (path.endsWith('.wasm') ? wasmUrl : path),
        });
    }
    return modulePromise;
}

self.onmessage = async (e) => {
    const { id, data, targetFormat } = e.data;

    try {
        const Module = await getModule();
        const result = transcodeKtx2(Module, data, targetFormat);
        self.postMessage(
            { id, success: true, result },
            result.mips.map((mip) => mip.data.buffer)
        );
    } catch (err) {
        self.postMessage({
            id,
            success: false,
            error: String(err && err.message ? err.message : err),
        });
    }
};
