// Main-context entry point. Exposes the contract consumed by the Rust side
// (src/wasm.rs embeds this bundle and dynamic-imports it from a Blob):
//
//   transcodeKtx2InWorker(data: Uint8Array, targetFormat: number) =>
//     Promise<{
//       width, height, levels, faces, layers,
//       hasAlpha, isEtc1s, isUastc, isSrgb, isHdr, isVideo,
//       mips: [{ width, height, data: Uint8Array }],
//     }>
//
// Decoding happens in a single dedicated inline worker; requests are matched
// to responses by an incrementing id (same pattern as draco_decoder).

import createWorker from './basisWorker.js?worker&inline';

let worker = null;
let requestId = 0;
const callbacks = new Map();

function getWorker() {
    if (!worker) {
        worker = createWorker();
        worker.onmessage = (e) => {
            const { id, success, result, error } = e.data;
            const cb = callbacks.get(id);
            if (!cb) return;
            callbacks.delete(id);

            if (success) {
                cb.resolve(result);
            } else {
                cb.reject(new Error(error));
            }
        };
    }
    return worker;
}

export function transcodeKtx2InWorker(data, targetFormat) {
    return new Promise((resolve, reject) => {
        const id = requestId++;
        callbacks.set(id, { resolve, reject });

        getWorker().postMessage({ id, data, targetFormat }, [data.buffer]);
    });
}
