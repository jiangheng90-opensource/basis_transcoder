// Node-side unit test for the core transcode path (no worker involved).
// Run with: npm test  (node --test test/)

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

import { createBasisModule, transcodeKtx2 } from '../src/transcode.js';

const WASM_PATH = fileURLToPath(import.meta.resolve('basis_transcoder/basis_transcoder.wasm'));
const KTX2_PATH = new URL('../../../testdata/texture_0.ktx2', import.meta.url);

// basist::transcoder_texture_format::cTFRGBA32 (see basisu_transcoder.h).
const CTF_RGBA32 = 13;

async function createTestModule() {
    const wasmBinary = readFileSync(WASM_PATH);
    return createBasisModule({ wasmBinary });
}

test('transcode texture_0.ktx2 to RGBA32', async () => {
    const Module = await createTestModule();

    // Cross-check the embind enum against the expected C++ value.
    assert.equal(Module.transcoder_texture_format.cTFRGBA32.value, CTF_RGBA32);

    const data = new Uint8Array(readFileSync(KTX2_PATH));
    const result = transcodeKtx2(Module, data, CTF_RGBA32);

    assert.equal(result.width, 16);
    assert.equal(result.height, 28);
    assert.equal(result.levels, 5);
    assert.equal(result.faces, 1);

    // Boolean fields must be 0/1 numbers.
    for (const key of ['hasAlpha', 'isEtc1s', 'isUastc', 'isSrgb', 'isHdr', 'isVideo']) {
        assert.ok(result[key] === 0 || result[key] === 1, `${key} must be 0 or 1`);
    }

    // One mip per level, dimensions halving each step (floor, min 1).
    assert.equal(result.mips.length, 5);
    let w = result.width;
    let h = result.height;
    for (let i = 0; i < result.mips.length; i++) {
        const mip = result.mips[i];
        assert.equal(mip.width, w, `mip ${i} width`);
        assert.equal(mip.height, h, `mip ${i} height`);
        assert.equal(mip.data.length, w * h * 4, `mip ${i} RGBA32 size`);
        w = Math.max(1, w >> 1);
        h = Math.max(1, h >> 1);
    }

    // The decoded pixels must not be a single solid color.
    const level0 = result.mips[0].data;
    const first = level0.subarray(0, 4).join(',');
    let allSame = true;
    for (let i = 4; i < level0.length; i += 4) {
        if (level0.subarray(i, i + 4).join(',') !== first) {
            allSame = false;
            break;
        }
    }
    assert.ok(!allSame, 'level 0 pixels are all identical');
});

test('invalid KTX2 data throws', async () => {
    const Module = await createTestModule();
    assert.throws(() =>
        transcodeKtx2(Module, new Uint8Array([1, 2, 3, 4]), CTF_RGBA32)
    );
});
