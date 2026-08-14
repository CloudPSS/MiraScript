import { Bench } from 'tinybench';
import { compile, compileSync, createVmContext } from '@mirascript/mirascript';

const bench = new Bench({
    name: 'simple benchmark',
    time: Number.parseInt(process.env['MIRASCRIPT_BENCH_TIME'] ?? '100', 10),
});
const source = `sin(x) + cos(y + PI / 2) + `.repeat(1) + '0';
const env = { x: 1, y: 2 };
const COMPILE_PARALLELISM = 1;

bench
    .add('compile', async () => {
        await Promise.all(Array.from({ length: COMPILE_PARALLELISM }).map(async () => compile(source)));
    })
    .add('compileSync', () => {
        Array.from({ length: COMPILE_PARALLELISM }).map(() => compileSync(source));
    });

bench.add('createVmContext', () => {
    return createVmContext(env);
});

const vmContext = createVmContext(env);
const script = compileSync(source);
bench.add('run', () => {
    return script(vmContext);
});

bench.add('nativeCompile', () => {
    // eslint-disable-next-line @typescript-eslint/no-implied-eval
    return new Function('context', `const {sin,cos,PI,x,y} = context; return ${source};`);
});
bench.add('nativeCreateContext', () => {
    return { __proto__: Math, ...env };
});
const nativeContext = { __proto__: Math, ...env };
// eslint-disable-next-line @typescript-eslint/no-implied-eval
const nativeScript = new Function('context', `const {sin,cos,PI,x,y} = context; return ${source};`) as (
    context: Record<string, unknown>,
) => unknown;

bench.add('nativeRun', () => {
    return nativeScript(nativeContext);
});

await bench.run();
// eslint-disable-next-line no-console
console.table(bench.table());
