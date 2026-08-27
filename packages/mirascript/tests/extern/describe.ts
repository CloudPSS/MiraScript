import test from 'ava';
import { createVmContext, VmExtern } from '@mirascript/mirascript';
import { exec } from './_exec.ts';

test('describe extern', (t) => {
    t.is(new VmExtern({}, null).tag, 'Object');
    t.is(new VmExtern(Object.create(null), null).tag, 'Object: null prototype');
    t.is(new VmExtern([], null).tag, 'Array(0)');
    t.is(new VmExtern(() => 0, null).tag, 'function');
    // eslint-disable-next-line @typescript-eslint/require-await
    t.is(new VmExtern(async () => 0, null).tag, 'async function');
    t.is(
        new VmExtern(function* () {
            yield 0;
        }, null).tag,
        'function*',
    );
    t.is(
        // eslint-disable-next-line @typescript-eslint/require-await
        new VmExtern(async function* () {
            yield 0;
        }, null).tag,
        'async function*',
    );
    const a = class A {
        x = 1;
    };
    t.is(new VmExtern(new a(), null).tag, 'Object');
    t.is(new VmExtern(a, null).tag, 'class');
    Object.defineProperty(a, 'name', { value: 'ALongName' });
    t.is(new VmExtern(new a(), null).tag, 'ALongName');
    t.is(new VmExtern(a, null).tag, 'class ALongName');
    const ab = a.bind(null);
    t.is(new VmExtern(new ab(), null).tag, 'ALongName');
    t.is(new VmExtern(ab, null).tag, 'function');
    Object.defineProperty(a, 'name', { value: '' });
    t.is(new VmExtern(new a(), null).tag, 'Object');
    t.is(new VmExtern(a, null).tag, 'class');
    Object.defineProperty(a, 'displayName', { value: 'ADisplayName' });
    t.is(new VmExtern(new a(), null).tag, 'ADisplayName');
    t.is(new VmExtern(a, null).tag, 'class ADisplayName');
    // eslint-disable-next-line unicorn/consistent-function-scoping
    const f = function () {
        return 1;
    };
    t.is(new VmExtern(f, null).tag, 'class');
    f.prototype = undefined;
    t.is(new VmExtern(f, null).tag, 'function');
    f.prototype = null;
    t.is(new VmExtern(f, null).tag, 'class');
});

test('extern to_string', (t) => {
    const context = createVmContext(null, {
        ok: {
            toString() {
                return 'ok';
            },
        },
        fail: {
            toString() {
                throw new Error('fail');
            },
        },
        void: {
            toString: null,
        },
        bad: {
            toString: 123,
        },
        normal: {},
        arr: [
            1,
            2,
            3,
            /test/i,
            undefined,
            null,
            [1, 2, 3],
            {
                toString() {
                    return `Custom String`;
                },
            },
            // eslint-disable-next-line no-sparse-arrays
            ,
        ],
        fail_arr: [
            {
                toString() {
                    throw new Error('fail');
                },
            },
        ],
    });
    const e = exec(context);
    t.is(e('ok::to_string()'), 'ok');
    t.throws(() => e('fail::to_string()'), { message: 'Failed to convert value to string: <extern>' });
    t.is(e('fail::to_string(0)'), 0);
    t.is(e('void::to_string()'), '<extern Object>');
    t.is(e('bad::to_string()'), '<extern Object>');
    t.is(e('normal::to_string()'), '<extern Object>');
    t.is(e('arr::to_string()'), '1, 2, 3, /test/i, , nil, [1, 2, 3], Custom String, ');
    t.throws(() => e('fail_arr::to_string()'), { message: 'Failed to convert value to string: <extern>' });
    t.is(e('arr.3::to_string()'), '/test/i');
});

test('extern json', (t) => {
    const context = createVmContext(null, {
        obj: { a: 1, b: 2 },
        arr: [1, 2, 3],
        func: () => 0,
        func_json: Object.assign(() => 0, {
            toJSON: () => 'func_json',
        }),
        ok: {
            toJSON() {
                return { ok: true };
            },
        },
        fail: {
            toJSON() {
                throw new Error('fail');
            },
        },
        void: {
            toJSON: null,
        },
    });
    const e = exec(context);
    t.is(e('obj::to_json()'), JSON.stringify({ a: 1, b: 2 }));
    t.is(e('arr::to_json()'), JSON.stringify([1, 2, 3]));
    t.is(e('func::to_json()'), null);
    t.is(e('func_json::to_json()'), '"func_json"');
    t.is(e('ok::to_json()'), JSON.stringify({ ok: true }));
    t.throws(() => e('fail::to_json()'), { message: /^fail$/ });
    t.is(e('void::to_json()'), JSON.stringify({ toJSON: null }));
});
