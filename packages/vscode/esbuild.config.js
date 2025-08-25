import esbuild from 'esbuild';
import packageJson from './package.json' with { type: 'json' };

esbuild.build({
    sourcemap: true,
    sourcesContent: false,
    format: 'iife',
    entryPoints: ['./media-src/*.ts'],
    minify: true,
    outdir: './media',
    target: 'esnext',
    bundle: true,
    packages: 'bundle',
    loader: {
        '.ttf': 'dataurl',
        '.wasm': 'dataurl',
    },
});

esbuild.build({
    sourcemap: true,
    format: 'esm',
    entryPoints: ['./src/main.ts'],
    minify: false,
    outdir: './dist',
    target: 'esnext',
    bundle: true,
    packages: 'bundle',
    external: ['vscode', ...Object.keys(packageJson.dependencies || {})],
    loader: {
        '.wasm': 'dataurl',
    },
});
