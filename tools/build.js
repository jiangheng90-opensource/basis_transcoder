const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const rootDir = path.join(__dirname, '..');
const jsDir = path.join(rootDir, 'third_party', 'basis_transcoder_js');

// Upstream precompiled Emscripten artifacts (basis_universal is a nested git
// repo; never modify it, only read from it).
const upstreamDir = path.join(
    rootDir, 'third_party', 'basis_universal', 'webgl', 'transcoder', 'build'
);
// Local npm package (file: dependency of basis_transcoder_js) mirroring how
// draco_decoder_js consumes the draco3d package.
const vendorDir = path.join(jsDir, 'basis_transcoder');

// Sync the vendored artifacts so upstream rebuilds propagate on `npm run build`.
console.log('Syncing vendored transcoder artifacts...');
fs.mkdirSync(vendorDir, { recursive: true });
for (const name of ['basis_transcoder.js', 'basis_transcoder.wasm']) {
    fs.copyFileSync(path.join(upstreamDir, name), path.join(vendorDir, name));
}

// Source files
const srcJsFile = path.join(jsDir, 'dist', 'index.es.js');
const srcCoreFile = path.join(jsDir, 'dist', 'core.es.js');

// Destination files
const destJsFile = path.join(rootDir, 'javascript', 'index.es.js');
const destCoreFile = path.join(rootDir, 'javascript', 'core.es.js');

// Build basis_transcoder_js
console.log('Building basis_transcoder_js...');
execSync('npm run build', { cwd: jsDir, stdio: 'inherit' });

// Copy the bundles
console.log('Copying index.es.js...');
fs.copyFileSync(srcJsFile, destJsFile);
console.log('Copying core.es.js...');
fs.copyFileSync(srcCoreFile, destCoreFile);

// The Rust side (src/wasm.rs) embeds these bundles verbatim in a JS template
// literal. It escapes backslashes and backticks itself, but NOT "${", which
// would start a template interpolation in the eval'd code. Fail the build if
// the minifier emitted one (adjust terser options instead of shipping it).
for (const file of [destJsFile, destCoreFile]) {
    const bundle = fs.readFileSync(file, 'utf8');
    const bad = ['${', '`'].filter((s) => bundle.includes(s));
    if (bad.length > 0) {
        throw new Error(
            file + ' contains raw ' + bad.join(' and ') +
            ' sequence(s); adjust the vite/terser config so the bundle is safe to embed'
        );
    }
}

console.log('Copied index.es.js and core.es.js to javascript/ (embed-safe: OK)');
