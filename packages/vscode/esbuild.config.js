import fs from 'node:fs/promises';
import esbuild from 'esbuild';
import packageJson from './package.json' with { type: 'json' };

await esbuild.build({
    sourcemap: true,
    sourcesContent: false,
    format: 'esm',
    charset: 'utf8',
    entryPoints: ['./media-src/*.ts'],
    minify: true,
    outdir: './media',
    target: 'esnext',
    platform: 'browser',
    bundle: true,
    packages: 'bundle',
    loader: {
        '.ttf': 'dataurl',
        '.wasm': 'dataurl',
    },
});

for await (const file of fs.glob('./media/*.js')) {
    const content = await fs.readFile(file, 'utf8');
    await fs.writeFile(file, content.replaceAll('import.meta', '({       })'), 'utf8');
}

await esbuild.build({
    sourcemap: true,
    format: 'esm',
    charset: 'utf8',
    entryPoints: ['./src/main.ts'],
    minify: false,
    outdir: './dist',
    target: 'esnext',
    bundle: true,
    packages: 'bundle',
    platform: 'node',
    external: ['vscode', ...Object.keys(packageJson.dependencies || {})],
});
