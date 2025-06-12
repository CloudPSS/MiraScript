import esbuild from 'esbuild';
import packageJson from './package.json' assert { type: 'json' };

esbuild.build({
    sourcemap: true,
    format: 'esm',
    entryPoints: ['./src/index.ts', './src/lsp/worker.ts'],
    outdir: './dist',
    target: 'esnext',
    bundle: true,
    packages: 'bundle',
    external: [...Object.keys(packageJson.dependencies || {}), ...Object.keys(packageJson.peerDependencies || {})],
    splitting: true,
});
