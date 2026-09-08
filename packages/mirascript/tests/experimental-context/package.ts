import test from 'ava';

test('published entry installs methods on the existing constructor only when imported', async (t) => {
    const main = await import('@mirascript/mirascript');
    const original = main.VmFunction;
    t.false('memo' in original);
    t.false('once' in original);
    t.false('async' in original);
    const { runInContext } = await import('@mirascript/mirascript/experimental-context');
    t.is(main.VmFunction, original);
    t.is(typeof original.memo, 'function');
    t.is(typeof original.once, 'function');
    t.is(typeof original.async, 'function');
    let starts = 0;
    let reads = 0;
    const context = main.createVmContext({
        read: original.memo(() => ++reads),
        load: original.async(() => {
            starts++;
            return Promise.resolve(5);
        }),
    });
    const value = await new Promise((resolve, reject) =>
        runInContext(main.compileSync('read() + load()'), context, resolve, reject),
    );
    t.is(value, 6);
    t.is(starts, 1);
    t.is(reads, 1);
});
