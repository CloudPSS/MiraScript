import esbuild from 'esbuild';
import { parseArgs } from 'node:util';

const args = parseArgs({
    options: {
        watch: {
            type: 'boolean',
            default: false,
        },
    },
});

const context = await esbuild.context({
    sourcemap: true,
    sourcesContent: false,
    format: 'esm',
    charset: 'utf8',
    entryPoints: ['./src/index.ts'],
    outdir: './dist',
    target: 'esnext',
    bundle: true,
    packages: 'external',
    external: ['#package.json'],
    minify: false,
    treeShaking: true,
    splitting: true,
});

await context.rebuild();
if (args.values.watch) {
    await context.watch();
    process.on('SIGINT', async () => {
        await context.dispose();
        process.exit(0);
    });
} else {
    await context.dispose();
}
