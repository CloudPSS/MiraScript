import test from 'ava';
import {
    compileSync,
    createVmContext,
    getVmFunctionInfo,
    isVmFunction,
    VmExtern,
    VmFunction,
} from '@mirascript/mirascript';
import { lib } from '@mirascript/mirascript/subtle';
import { AsyncRequiredError, runInContext } from '@mirascript/mirascript/experimental-context';
import { execute } from './_helpers.ts';

test('synchronous completion and callbacks outside the context', (t) => {
    const events = ['A'];
    runInContext(
        compileSync('42'),
        undefined,
        (value) => {
            events.push(`done ${String(value)}`);
            runInContext(
                compileSync('1'),
                null,
                (nested) => t.is(nested, 1),
                () => t.fail(),
            );
        },
        () => t.fail(),
    );
    events.push('B');
    t.deepEqual(events, ['A', 'done 42', 'B']);
});

test('basic check of wrapper functions', (t) => {
    t.throws(
        () =>
            // @ts-expect-error: VmFunction.memo expects a function, not a number
            VmFunction.memo(1),
        { instanceOf: TypeError },
    );
});

test('sync async-call failure never starts the implementation, including extern boundaries', (t) => {
    let calls = 0;
    const fn = VmFunction.async(() => {
        calls++;
        return Promise.resolve(1);
    });
    t.throws(() => fn(), { instanceOf: AsyncRequiredError });
    t.throws(() => compileSync('f()')(createVmContext({ f: fn })), { instanceOf: AsyncRequiredError });
    t.throws(() => new VmExtern(() => fn()).call([]), { instanceOf: AsyncRequiredError });
    t.is(calls, 0);
});

test('constructors preserve names, VM markers and all option sources', (t) => {
    const original = () => 3;
    const existing = VmFunction(original, { summary: 'original' });
    for (const factory of [VmFunction.memo, VmFunction.once]) {
        const wrapped = factory(existing);
        t.not(wrapped, existing);
        t.true(isVmFunction(wrapped));
        t.is(wrapped.name, 'original');
        t.is(wrapped(), 3);
        t.is(getVmFunctionInfo(factory(original, { name: 'renamed', summary: 'new' }))?.summary, 'new');
        t.is(factory(original, { name: 'renamed' }).name, 'renamed');
        t.is(getVmFunctionInfo(factory(original, existing))?.summary, 'original');
        t.is(getVmFunctionInfo(factory(original, lib.random))?.summary, lib.random.summary);
        t.is(getVmFunctionInfo(factory(original, lib.random))?.isLib, true);
    }
    const asynchronous = VmFunction.async(() => Promise.resolve(1), { name: 'load', summary: 'async' });
    t.true(isVmFunction(asynchronous));
    t.is(asynchronous.name, 'load');
    t.is(getVmFunctionInfo(asynchronous)?.summary, 'async');
});

test('memo and once retain synchronous values, undefined and exceptions', (t) => {
    t.is(
        VmFunction.once(() => {
            /* A void-returning effect. */
        })(),
        undefined,
    );
    const error = new Error('sync');
    t.throws(
        () =>
            VmFunction.memo(() => {
                throw error;
            })(),
        { is: error },
    );
});

test('managed effects run once per logical call through closures and higher-order functions', async (t) => {
    const reads: number[] = [];
    const writes: unknown[] = [];
    const loads: unknown[] = [];
    const context = createVmContext({
        read: VmFunction.memo(() => {
            reads.push(reads.length + 1);
            return reads.length;
        }),
        write: VmFunction.once((value) => {
            writes.push(value);
            return value;
        }),
        load: VmFunction.async((value) => {
            loads.push(value);
            return Promise.resolve(value);
        }),
    });
    const result = await execute(
        `
        let before = read();
        fn f(x) { write(x); load(x) }
        let values = map([1, 2, 3], f);
        [before, values, read()]
    `,
        context,
    );
    t.deepEqual(result, [1, [1, 2, 3], 2]);
    t.deepEqual(reads, [1, 2]);
    t.deepEqual(writes, [1, 2, 3]);
    t.deepEqual(loads, [1, 2, 3]);
});
