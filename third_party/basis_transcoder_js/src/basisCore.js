// Pure in-context transcode entry — no worker. Bundled as `core.es.js` for
// hosts that already run inside their own Web Worker (e.g. a worker pool);
// the worker-hosted API stays in `src/index.js` (index.es.js).

import wasmUrl from 'basis_transcoder/basis_transcoder.wasm?url';
import { createBasisModule, transcodeKtx2 } from './transcode.js';

let modulePromise = null;

// Lazily creates the Emscripten module once per context and reuses it for
// every subsequent transcode. The wasm binary is the inlined data: URL.
// Safe to call concurrently.
export function initBasisTranscoder() {
    if (!modulePromise) {
        modulePromise = createBasisModule({
            locateFile: (path) => (path.endsWith('.wasm') ? wasmUrl : path),
        });
    }
    return modulePromise;
}

// Transcodes every mip level of a KTX2 Basis Universal texture in the
// current context. Returns the same shape as the worker-hosted
// `transcodeKtx2InWorker` (see src/index.js).
export async function transcodeKtx2Local(data, targetFormat) {
    const Module = await initBasisTranscoder();
    return transcodeKtx2(Module, data, targetFormat);
}
