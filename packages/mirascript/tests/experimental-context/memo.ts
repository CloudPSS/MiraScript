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
