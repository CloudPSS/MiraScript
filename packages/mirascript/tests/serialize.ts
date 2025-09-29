import test from 'ava';
import { compile, type VmAny, type VmValue } from '@mirascript/mirascript';
import { serialize } from '@mirascript/mirascript/subtle';

const serializeRoundTrip = test.macro<[value: VmAny, expected?: VmValue]>({
    exec: async (t, value, expected) => {
        const serialized = serialize(value);
        const deserialized = (await compile(serialized))();
        // bypass fast route
        const deserialized2 = (await compile(`{${serialized}}`))();
        if (expected === undefined) {
            expected = value;
        }
        t.deepEqual(deserialized, expected);
        t.deepEqual(deserialized2, expected);
    },
    title: (providedTitle, value) => providedTitle || JSON.stringify(value) || String(value),
});

for (const value of [null, true, false, Number.NaN, Infinity, -Infinity, +0, 0.1, 0.2, 0.3]) {
    test(`simple primitive ${String(value)}`, serializeRoundTrip, value);
}

for (const [value, expected] of [[undefined, null]]) {
    test(`no roundtrip ${String(value)}`, serializeRoundTrip, value, expected);
}

for (const value of [
    '',
    ' ',
    'Hello, World!',
    '你好，世界！',
    'こんにちは、世界！',
    '👋🌍',
    '\0\u0001\n\r\t\b\f\v\\',
    '\u202A\u{E0001}',
    "'",
    '"',
    '`',
    '`\'"',
    '$a',
    '\u000A\u000D\u001C\u001D\u0085\u2028\u2029\uFEFF',
]) {
    test(`string ${serialize(value)}`, serializeRoundTrip, value);
}

test(`broken surrogate string`, serializeRoundTrip, '\uDC00\uD800', '\uFFFD\uFFFD');

for (const value of [
    0,
    [+0, -0],
    [],
    {},
    { 0: 1 },
    { '00': 1 },
    [1, 2, 3],
    { a: 1, b: 2, c: 3 },
    { a: 1, b: '2', c: true },
    { a: null, b: [null] },
    [Number.NaN, Infinity, -Infinity, +0, 0.1, 0.2, 0.3],
    { ...['x', 'y', ['z']] },
    // eslint-disable-next-line no-sparse-arrays
    { ...[, 'x', 'y', ['z']] },
    { 0: 'x', a: 'y', '\n': 'z', '': 'w' },
]) {
    test(serializeRoundTrip, value as VmAny);
}

for (const [value, expected] of [
    [
        { a: null, b: undefined },
        { a: null, b: null },
    ],
    // eslint-disable-next-line unicorn/no-new-array
    [new Array(1), [null]], // Array with a hole
    [() => 0, null],
    [
        [() => 0, /1/, new Date(0)],
        [null, {}, 0],
    ],
    [new Date(0), 0],
    [{ k: 'valueOf 1', valueOf: () => 1 }, 1],
    [{ k: 'valueOf []', valueOf: () => [1, 2, 3] }, [1, 2, 3]],
    [
        { k: 'valueOf undefined', valueOf: () => undefined },
        { k: 'valueOf undefined', valueOf: null },
    ],
    [{ k: 'valueOf null', valueOf: () => null }, null],
    [
        {
            k: 'valueOf this',
            valueOf() {
                return this;
            },
        },
        {
            k: 'valueOf this',
            valueOf: null,
        },
    ],
]) {
    test(serializeRoundTrip, value as VmAny, expected as VmAny);
}

test('array with props', serializeRoundTrip, Object.assign([1, 2, 3], { x: 1 }), [1, 2, 3]);
