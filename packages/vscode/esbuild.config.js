import esbuild from 'esbuild';

const entryPoints = ['./src/markdown-preview.ts'];

esbuild.build({
    sourcemap: true,
    format: 'iife',
    entryPoints,
    outdir: './dist',
    target: 'esnext',
    bundle: true,
    packages: 'bundle',
    loader: {
        '.ttf': 'dataurl',
        '.wasm': 'dataurl',
    },
});
