import test from 'ava';
import { compileSync, createVmContext, VmError, VmExtern, VmFunction, type VmValue } from '@mirascript/mirascript';
import { lib, wrapScript, serializeForDisplay, convert } from '@mirascript/mirascript/subtle';
import {
    AsyncRequiredError,
    ContextReentrancyError,
    ReplayMismatchError,
    runInContext,
} from '@mirascript/mirascript/experimental-context';
import { execute } from './_helpers.ts';

test('async completion timing and independent journals', async (t) => {
    const first = Promise.withResolvers<VmValue>();
    const second = Promise.withResolvers<VmValue>();
    const events = ['A'];
    let calls = 0;
    const load = VmFunction.async(() => (++calls === 1 ? first.promise : second.promise));
    const globals = createVmContext({ load });
    const a = execute('load()', globals).then((value) => {
        events.push(`a ${String(value)}`);
    });
    const b = execute('load()', globals).then((value) => {
        events.push(`b ${String(value)}`);
    });
    events.push('B');
    t.deepEqual(events, ['A', 'B']);
    second.resolve(2);
    await b;
    first.resolve(1);
    await a;
    t.deepEqual(events, ['A', 'B', 'b 2', 'a 1']);
    t.is(calls, 2);
});

test('thenable assimilation and VM object results', async (t) => {
    let calls = 0;
    const object = { answer: 42 };
    const load = VmFunction.async(() => ({
        // eslint-disable-next-line unicorn/no-thenable
        then(resolve) {
            calls++;
            return Promise.resolve(resolve?.(object));
        },
    }));
    t.is(await execute('load().answer', createVmContext({ load })), 42);
    t.is(calls, 1);
});

test('rejections and synchronous throws preserve errors', async (t) => {
    const error = new Error('load failed');
    for (const load of [
        VmFunction.async(() => Promise.reject(error)),
        VmFunction.async(() => {
            throw error;
        }),
    ]) {
        await t.throwsAsync(execute('load()', createVmContext({ load })), { is: error });
    }
    const load = VmFunction.async(() => Promise.reject(error));
    const caught = await t.throwsAsync(execute('load()', createVmContext(null, { load: () => load() })), {
        instanceOf: VmError,
    });
    t.is(caught?.cause, error);
    t.is(compileSync('1')(), 1);
});

test('caught memo errors and once return values are cached across replay', async (t) => {
    const error = new Error('read failed');
    let calls = 0;
    const read = VmFunction.memo(() => {
        calls++;
        throw error;
    });
    const value = { x: 1 };
    let writes = 0;
    const once = VmFunction.once(() => {
        writes++;
        return value;
    });
    const load = VmFunction.async(() => Promise.resolve(null));
    const script = wrapScript('', 'Script', () => {
        t.throws(() => read(), { is: error });
        t.is(once(), value);
        load();
        return value;
    });
    t.is(await new Promise((resolve, reject) => runInContext(script, null, resolve, reject)), value);
    t.is(calls, 1);
    t.is(writes, 1);
});

test('sequence changes and shorter replay paths fail', async (t) => {
    for (const shorten of [false, true]) {
        let first = true;
        const load = VmFunction.async(() => {
            first = false;
            return Promise.resolve(1);
        });
        const other = VmFunction.memo(() => 2);
        const script = wrapScript('', 'Script', () => {
            if (first) load();
            else if (!shorten) other();
            return null;
        });
        await t.throwsAsync(new Promise((resolve, reject) => runInContext(script, null, resolve, reject)), {
            instanceOf: ReplayMismatchError,
        });
    }
});

test('nested contexts and effects fail even when the host catches the error', async (t) => {
    const inner = VmFunction.memo(() => 1);
    for (const nested of [
        () => inner(),
        () =>
            runInContext(
                compileSync('1'),
                null,
                () => t.fail(),
                () => t.fail(),
            ),
    ]) {
        const outer = VmFunction.memo(() => {
            try {
                nested();
            } catch {
                /* Deliberately swallow the control error. */
            }
            return null;
        });
        await t.throwsAsync(execute('outer()', createVmContext({ outer })), { instanceOf: ContextReentrancyError });
    }
    t.is(await execute('2'), 2);
});

test('swallowing suspension cannot complete a pending computation', async (t) => {
    let calls = 0;
    const load = VmFunction.async(() => {
        calls++;
        return Promise.resolve(4);
    });
    const script = wrapScript('', 'Script', () => {
        try {
            return load();
        } catch {
            return 99;
        }
    });
    t.is(await new Promise((resolve, reject) => runInContext(script, null, resolve, reject)), 4);
    t.is(calls, 1);
});

test('conversion and display fallbacks propagate suspension and async-required errors', async (t) => {
    for (const convertValue of [(value: VmExtern) => convert.toString(value, 'fallback'), serializeForDisplay]) {
        const load = VmFunction.async(() => Promise.resolve('ready'));
        const value = new VmExtern({ toString: () => load() });
        t.throws(() => convertValue(value), { instanceOf: AsyncRequiredError });
        const script = wrapScript('', 'Script', () => convertValue(value));
        const result = await new Promise((resolve, reject) => runInContext(script, null, resolve, reject));
        t.true(String(result).includes('ready'));
    }
});

test('callback exceptions never invoke the other callback', (t) => {
    const error = new Error('callback');
    t.throws(
        () =>
            runInContext(
                compileSync('1'),
                null,
                () => {
                    throw error;
                },
                () => t.fail(),
            ),
        { is: error },
    );
    const script = wrapScript('', 'Script', () => {
        throw new Error('script');
    });
    t.throws(
        () =>
            runInContext(
                script,
                null,
                () => t.fail(),
                () => {
                    throw error;
                },
            ),
        { is: error },
    );
});

test('repeated deep suspension unwinds checkpoint depth', async (t) => {
    let calls = 0;
    const load = VmFunction.async(() => {
        calls++;
        return Promise.resolve(7);
    });
    const globals = createVmContext({ load });
    const script = compileSync('fn f(n) { if n == 0 { load() } else { f(n - 1) } }; f(80)');
    for (let i = 0; i < 4; i++) {
        t.is(await new Promise((resolve, reject) => runInContext(script, globals, resolve, reject)), 7);
    }
    t.is(calls, 4);
    t.is(compileSync('fn f(n) { if n == 0 { 9 } else { f(n - 1) } }; f(100)')(), 9);
});

test('missing and void VM results normalize to nil in scripts', async (t) => {
    const globals = createVmContext({
        read: VmFunction.memo((value) => value),
        write: VmFunction.once(() => {
            /* Returns void. */
        }),
        load: VmFunction.async(() => Promise.resolve(undefined)),
    });
    t.deepEqual(await execute('[read(), write(), load()]', globals), [null, null, null]);
});

test('arbitrary rejection values remain intact', async (t) => {
    for (const expected of [null, undefined, 0, 'failure']) {
        // Verify JavaScript permits non-Error rejection reasons at this boundary.
        // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors
        const globals = createVmContext({ load: VmFunction.async(() => Promise.reject(expected)) });
        let completed = false;
        await new Promise<void>((resolve) =>
            runInContext(
                compileSync('load()'),
                globals,
                () => {
                    completed = true;
                    resolve();
                },
                (error) => {
                    t.is(error, expected);
                    resolve();
                },
            ),
        );
        t.false(completed);
    }
});

test.serial('debug serializer cannot hide nested effects with its fallback', async (t) => {
    const { serializer } = lib.debug_print;
    const read = VmFunction.memo(() => 'nested');
    lib.debug_print.serializer = () => String(read());
    try {
        await t.throwsAsync(execute('debug_print("%s", 1)'), { instanceOf: ContextReentrancyError });
    } finally {
        lib.debug_print.serializer = serializer;
    }
});

test.serial('standard-library reads and logs are stable during replay', async (t) => {
    const originalNow = Date.now;
    const { logger } = lib.debug_print;
    const panicLogger = lib.panic.logger;
    t.teardown(() => {
        Date.now = originalNow;
        lib.debug_print.logger = logger;
        lib.panic.logger = panicLogger;
    });
    let time = 1000;
    Date.now = () => time;
    const logs: unknown[][] = [];
    lib.debug_print.logger = (...args) => {
        logs.push(args);
    };
    lib.panic.logger = (...args) => {
        logs.push(args);
    };
    const seen: unknown[] = [];
    const observe = VmFunction((value) => {
        seen.push(value);
    });
    const load = VmFunction.async(() => {
        time = 2000;
        return Promise.resolve(null);
    });
    const result = await execute(
        `
        let r = random();
        let a = to_timestamp();
        let b = to_datetime().second;
        let c = to_iso8601();
        observe([r, a, b, c]);
        debug_print("before");
        load();
        [a, b, c, to_timestamp(), to_timestamp(0)]
    `,
        createVmContext({ observe, load }),
    );
    t.deepEqual(seen[0], seen[1]);
    t.deepEqual(result, [1000, 1, '1970-01-01T00:00:01.000Z', 2000, 0]);
    t.is(logs.length, 1);
    await t.throwsAsync(execute('debug_print("kept"); load(); panic("failed")', createVmContext({ load })), {
        instanceOf: VmError,
    });
    t.is(logs.length, 3);
});
