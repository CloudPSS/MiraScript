import fs from 'node:fs/promises';
import { builtinModules } from 'node:module';
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
    banner: {
        js: `(async () => {`,
    },
    footer: {
        js: `})();`,
    },
    loader: {
        '.ttf': 'dataurl',
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
    mainFields: ['exports', 'module', 'main'],
    banner: {
        js: /* js */ `import { createRequire as Ĩ } from 'node:module';const require = Ĩ(import.meta.url);const { dirname: __dirname, filename: __filename } = import.meta;`,
    },
    external: [
        'vscode',
        ...builtinModules.flatMap((m) => [m, `node:${m}`]),
        ...Object.keys(packageJson.dependencies || {}),
    ],
});
