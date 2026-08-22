import { builtinModules } from 'node:module';
import esbuild from 'esbuild';
import { definePlugin } from 'esbuild-plugin-define';

// Universal bundle
await esbuild.build({
    sourcemap: true,
    format: 'esm',
    charset: 'utf8',
    entryPoints: { main: './src/main.ts' },
    outExtension: { '.js': '.mjs' },
    minify: true,
    outdir: './dist',
    target: 'esnext',
    bundle: true,
    packages: 'bundle',
    platform: 'browser',
    plugins: [
        definePlugin({
            'navigator.userAgentData': '',
            'navigator.userAgent': 'Chrome/100',
        }),
    ],
    external: ['vscode', ...builtinModules.flatMap((m) => [m, `node:${m}`])],
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
