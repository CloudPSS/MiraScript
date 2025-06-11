import esbuild from 'esbuild';

esbuild.build({
    sourcemap: true,
    format: 'esm',
    entryPoints: ['./src/index.ts', './src/lsp/worker.ts'],
    outdir: './dist',
    target: 'esnext',
    bundle: true,
    packages: 'external',
    splitting: true,
});
