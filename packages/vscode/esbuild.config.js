import { builtinModules } from 'node:module';
import esbuild from 'esbuild';
import packageJson from './package.json' with { type: 'json' };

// Universal bundle
await esbuild.build({
    sourcemap: true,
    format: 'esm',
    charset: 'utf8',
    entryPoints: { main: './src/main.ts' },
    outExtension: { '.js': '.mjs' },
    minify: false,
    outdir: './dist',
    target: 'esnext',
    bundle: true,
    packages: 'bundle',
    platform: 'browser',
    conditions: [],
    external: [
        'vscode',
        ...builtinModules.flatMap((m) => [m, `node:${m}`]),
        ...Object.keys(packageJson.dependencies || {}),
    ],
});

// Web bundle
await esbuild.build({
    sourcemap: true,
    format: 'esm',
    charset: 'utf8',
    entryPoints: { browser: './src/browser.ts' },
    outExtension: { '.js': '.cjs' },
    minify: true,
    outdir: './dist',
    target: 'esnext',
    bundle: true,
    packages: 'bundle',
    platform: 'browser',
    conditions: ['web-extension'],
    external: ['vscode', ...builtinModules.flatMap((m) => [m, `node:${m}`])],
});
