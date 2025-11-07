import esbuild from 'esbuild';
import packageJson from './package.json' with { type: 'json' };

const entryPoints = [...Object.values(packageJson.exports), ...Object.values(packageJson.imports)]
    .map((value) => {
        if ('@mirascript/dev' in value) {
            return value['@mirascript/dev'];
        }
        return undefined;
    })
    .filter(Boolean);

const external = Object.entries(packageJson.imports)
    .map(([key, value]) => {
        if ('node' in value) {
            return key;
        }
        return undefined;
    })
    .filter(Boolean);

esbuild.build({
    sourcemap: true,
    sourcesContent: false,
    format: 'esm',
    charset: 'utf8',
    entryPoints,
    outdir: './dist',
    target: 'esnext',
    bundle: true,
    packages: 'external',
    external,
    splitting: true,
});
