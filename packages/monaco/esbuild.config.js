import esbuild from 'esbuild';
import packageJson from './package.json' with { type: 'json' };

const entryPoints = [...Object.values(packageJson.exports), ...Object.values(packageJson.imports)]
    .map((value) => {
        if ('@mira/development' in value) {
            return value['@mira/development'];
        }
        return undefined;
    })
    .filter(Boolean);

esbuild.build({
    sourcemap: true,
    sourcesContent: false,
    format: 'esm',
    entryPoints,
    outdir: './dist',
    target: 'esnext',
    bundle: true,
    packages: 'bundle',
    external: [...Object.keys(packageJson.dependencies || {}), ...Object.keys(packageJson.peerDependencies || {})],
    splitting: true,
});
