import { defineConfig } from 'vite';

// Bundles the official Emscripten transcoder build (vendored into the local
// `basis_transcoder` package by tools/build.js from
// third_party/basis_universal/webgl/transcoder/build) into a single
// self-contained ES module. The ~1MB wasm is inlined as a data: URL so the
// output has no external assets; the decoder runs in a dedicated inline
// Web Worker (`?worker&inline`).
export default defineConfig({
    build: {
        lib: {
            entry: {
                // Worker-hosted API (single dedicated worker, created lazily
                // on first call).
                index: 'src/index.js',
                // Pure in-context transcode API — no worker. Embedded by
                // hosts that already run inside their own worker.
                core: 'src/basisCore.js',
            },
            name: 'BasisTranscoderJS',
            fileName: (format, entryName) => `${entryName}.${format}.js`,
            formats: ['es'],
        },
        // Inline the wasm binary as a data: URL.
        assetsInlineLimit: 100 * 1024 * 1024,
        rollupOptions: {
            external: [],
            output: {
                assetFileNames: '[name][extname]',
            },
        },
        commonjsOptions: {
            // basis_transcoder.js is a UMD/CommonJS Emscripten artifact whose
            // real path (through the file: symlink) misses the default
            // node_modules-only include pattern.
            include: [/basis_transcoder/, /node_modules/],
        },
        minify: 'esbuild',
    },
    esbuild: {
        // The Rust side embeds the bundle in a JS template literal and only
        // escapes backslashes/backticks, so the output must not contain
        // template literals (`${` would break the embedding). esbuild lowers
        // them to plain string concatenation when the feature is disabled.
        supported: { 'template-literal': false },
        drop: ['console', 'debugger'],
        legalComments: 'none',
    },
    worker: {
        format: 'es',
    },
});
