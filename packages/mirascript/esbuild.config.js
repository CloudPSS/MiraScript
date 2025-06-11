import esbuild from 'esbuild';

esbuild.build({
    sourcemap: true,
    format: 'esm',
    entryPoints: ['./src/index.ts', './src/subtle.ts'],
    outdir: './dist',
    target: 'esnext',
    bundle: true,
    packages: 'external',
    splitting: true,
});
