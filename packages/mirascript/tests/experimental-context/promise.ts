import test from 'ava';
import { compileSync, createVmContext, VmFunction } from '@mirascript/mirascript';
import { runInContext } from '@mirascript/mirascript/experimental-context';

test('runInContext returns value with sync function', (t) => {
    const result = runInContext(compileSync(`sin(42)`));
    t.is(result, Math.sin(42));
});

test('runInContext returns Promise with async function', async (t) => {
    const events: string[] = [];
    const result = runInContext(
        compileSync(`f()::sin()`),
        createVmContext({
            f: VmFunction.async(async () => {
                events.push('f enter');
                await Promise.resolve();
                events.push('f called');
                return 42;
            }),
        }),
    );
    t.true(result instanceof Promise);
    t.deepEqual(events, ['f enter']);
    t.is(await result, Math.sin(42));
    t.deepEqual(events, ['f enter', 'f called']);
});

test('runInContext reject one callback', (t) => {
    const s = compileSync('');
    // @ts-expect-error: runInContext expects 2 callbacks
    t.throws(() => runInContext(s, null, () => undefined), { instanceOf: TypeError });
    // @ts-expect-error: runInContext expects 2 callbacks
    t.throws(() => runInContext(s, null, undefined, () => undefined), { instanceOf: TypeError });
});
