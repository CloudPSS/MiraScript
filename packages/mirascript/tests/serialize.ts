import test from 'ava';
import { compile, VmModule, type VmAny, type VmArray, type VmRecord, type VmValue } from '@mirascript/mirascript';
import { serialize, serializeString, serializePropName } from '@mirascript/mirascript/subtle';
import { VmExtern } from '../dist/index.js';

test('serializeString', (t) => {
    t.is(serializeString('Hello, World!'), `'Hello, World!'`);
    t.is(serializeString("He said, 'Hello!'\n"), String.raw`'He said, \'Hello!\'\n'`);
    t.is(serializeString('你好，世界！'), `'你好，世界！'`);
    t.is(serializeString('こんにちは、世界！'), `'こんにちは、世界！'`);
    t.is(serializeString('👋🌍'), `'👋🌍'`);
    t.is(serializeString('\0\u0001\n\r\t\b\f\v\\'), String.raw`'\0\x01\n\r\t\b\f\v\\'`);
    t.is(serializeString('\u202A\u{E0001}'), String.raw`'\u{202a}\u{e0001}'`);
    t.is(serializeString("'"), String.raw`'\''`);
    t.is(serializeString('"'), `'"'`);
    t.is(serializeString('`'), "'`'");
    t.is(serializeString('`\'"'), `'\`\\'"'`);
    t.is(serializeString('$a'), String.raw`'\$a'`);
    t.is(
        serializeString('\u000A\u000D\u001C\u001D\u0085\u2028\u2029\uFEFF'),
        `'\\n\\r\\x1c\\x1d\\u{85}\u2028\u2029\\u{feff}'`,
    );
    t.is(serializeString('\uDC00\uD800'), `'��'`); // broken surrogate
});

test('serializePropName', (t) => {
    t.is(serializePropName('simple'), 'simple');
    t.is(serializePropName('_simple'), '_simple');
    t.is(serializePropName('$simple'), '$simple');
    t.is(serializePropName('simple123'), 'simple123');
    t.is(serializePropName('$_simple123'), '$_simple123');
    t.is(serializePropName('123'), '123');
    t.is(serializePropName('0'), '0');
    t.is(serializePropName('00'), `'00'`);
    t.is(serializePropName('with space'), `'with space'`);
    t.is(serializePropName('with-hyphen'), `'with-hyphen'`);
    t.is(serializePropName('with.dot'), `'with.dot'`);
    t.is(serializePropName('你好'), `你好`);
    t.is(serializePropName('你好，世界！'), `'你好，世界！'`);
});

const serializeRoundTrip = test.macro<[value: VmAny, expected?: VmValue]>({
    exec: async (t, value, expected) => {
        const serialized = serialize(value);
        const deserialized = (await compile(serialized))();
        // bypass fast route
        const deserialized2 = (await compile(`return {${serialized}};`))();
        if (expected === undefined) {
            expected = value;
        }
        t.deepEqual(deserialized, expected);
        t.deepEqual(deserialized2, expected);
    },
    title: (providedTitle, value) => {
        const v = value == null || typeof value == 'number' ? String(value) : JSON.stringify(value) || String(value);
        if (!providedTitle) return v;
        return `${providedTitle}: ${v}`;
    },
});

for (const value of [null, true, false, Number.NaN, Infinity, -Infinity, +0, 0.1, 0.2, 0.3]) {
    test(`simple primitive`, serializeRoundTrip, value);
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
    test(`string`, serializeRoundTrip, value);
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
    test('roundtrip', serializeRoundTrip, value as VmAny);
}

for (const [value, expected] of [
    [undefined, null],
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
    [new VmExtern({}), null],
    [new VmModule('test', {}), null],
    [
        { e: new VmExtern({}), m: new VmModule('test', {}) },
        { e: null, m: null },
    ],
    [Object.assign([1, 2, 3], { x: 1 }), [1, 2, 3]],
]) {
    test('no roundtrip', serializeRoundTrip, value as VmAny, expected as VmAny);
}

const makeDeepArray = (depth: number): VmArray => {
    if (depth <= 0) return [];
    return [makeDeepArray(depth - 1)];
};
const makeDeepRecord = (depth: number): VmRecord => {
    if (depth <= 0) return {};
    return { d: makeDeepRecord(depth - 1) };
};

// TODO: fix stack overflow
for (const depth of [0, 1, 5, 10, 20, 100, 128]) {
    test(`deep array ${depth}`, serializeRoundTrip, makeDeepArray(depth));
    test(`deep record ${depth}`, serializeRoundTrip, makeDeepRecord(depth));
}

test('deep array maxDepth', (t) => t.deepEqual(serialize(makeDeepArray(256)), serialize(makeDeepArray(128))));
test('deep record maxDepth', (t) => t.deepEqual(serialize(makeDeepRecord(256)), serialize(makeDeepRecord(128))));
