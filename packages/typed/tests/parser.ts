import test from 'ava';
import { parse } from '../src/parser.ts';

test('primitive types', (t) => {
    t.is(parse('nil'), 'nil');
    t.is(parse('string'), 'string');
    t.is(parse('number'), 'number');
    t.is(parse('boolean'), 'boolean');
    t.is(parse('record'), 'record');
    t.is(parse('array'), 'array');
});

test('named type', (t) => {
    t.is(parse('MyType'), 'MyType');
});

test('unicode named type', (t) => {
    t.is(parse('类型'), '类型');
    t.is(parse('My类型'), 'My类型');
});

test('special character named type', (t) => {
    t.is(parse('_MyType'), '_MyType');
    t.is(parse('$MyType'), '$MyType');
    t.is(parse('@MyType'), '@MyType');
    t.is(parse('$_MyType'), '$_MyType');
    t.throws(() => parse('1MyType'));
    t.throws(() => parse('$@MyType'));
});

test('string literal type', (t) => {
    t.deepEqual(parse('"hello"'), { kind: 'literal', value: 'hello' });
});

test('string literal type with escaped characters', (t) => {
    t.deepEqual(parse(String.raw`'\''`), { kind: 'literal', value: "'" });
    t.deepEqual(parse('`\\n`'), { kind: 'literal', value: '\n' });
    t.deepEqual(parse(String.raw`'\"'`), { kind: 'literal', value: '"' });
    t.deepEqual(parse(String.raw`'\\'`), { kind: 'literal', value: '\\' });
    t.deepEqual(parse(String.raw`'\n'`), { kind: 'literal', value: '\n' });
    t.deepEqual(parse(String.raw`'\t'`), { kind: 'literal', value: '\t' });
    t.deepEqual(parse(String.raw`'\r'`), { kind: 'literal', value: '\r' });
    t.deepEqual(parse(String.raw`'\b'`), { kind: 'literal', value: '\b' });
    t.deepEqual(parse(String.raw`'\f'`), { kind: 'literal', value: '\f' });
    t.deepEqual(parse(String.raw`'\v'`), { kind: 'literal', value: '\v' });
    t.deepEqual(parse(String.raw`'\0'`), { kind: 'literal', value: '\0' });
    t.deepEqual(parse(String.raw`'\x41'`), { kind: 'literal', value: 'A' });
    t.deepEqual(parse(String.raw`'\u{41}'`), { kind: 'literal', value: 'A' });
    t.deepEqual(parse(String.raw`'\\u{41}'`), { kind: 'literal', value: String.raw`\u{41}` });
    t.deepEqual(parse(String.raw`"hello\nworld"`), { kind: 'literal', value: 'hello\nworld' });
    t.deepEqual(parse(String.raw`"hello\tworld"`), { kind: 'literal', value: 'hello\tworld' });
    t.deepEqual(parse(String.raw`"hello\\"`), { kind: 'literal', value: 'hello\\' });
    t.deepEqual(parse(String.raw`"hello\""`), { kind: 'literal', value: 'hello"' });
    t.deepEqual(parse(String.raw`"hello\x41world"`), { kind: 'literal', value: 'helloAworld' });
    t.deepEqual(parse(String.raw`"hello\u{41}world"`), { kind: 'literal', value: 'helloAworld' });
});

test('string literal type with invalid escape sequences', (t) => {
    t.throws(() => parse(String.raw`'\x4'`));
    t.throws(() => parse(String.raw`'\u{4'`));
    t.throws(() => parse(String.raw`'\xff'`));
    t.throws(() => parse(String.raw`'\u{110000}'`));
    t.throws(() => parse(String.raw`'\u{D800}\u{DC00}'`));
    t.throws(() => parse(String.raw`'\x1'`));
    t.throws(() => parse(String.raw`'\x1g'`));
    t.throws(() => parse(String.raw`'\x'`));
});

test('string literal type with interpolation', (t) => {
    t.throws(() => parse('`hello ${name}`'));
    t.throws(() => parse('`hello ${name} world`'));
    t.throws(() => parse('`hello ${name} world ${age}`'));
});

test('empty string literal type', (t) => {
    t.deepEqual(parse('""'), { kind: 'literal', value: '' });
});

test('single-quoted string literal type', (t) => {
    t.deepEqual(parse("'hello'"), { kind: 'literal', value: 'hello' });
});

test('backtick-quoted string literal type', (t) => {
    t.deepEqual(parse('`hello`'), { kind: 'literal', value: 'hello' });
});

test('boolean literal types', (t) => {
    t.deepEqual(parse('true'), { kind: 'literal', value: true });
    t.deepEqual(parse('false'), { kind: 'literal', value: false });
});

test('array type', (t) => {
    t.deepEqual(parse('number[]'), { kind: 'array', element: 'number' });
});

test('nested array type', (t) => {
    t.deepEqual(parse('number[][]'), {
        kind: 'array',
        element: { kind: 'array', element: 'number' },
    });
});

test('array generic type', (t) => {
    t.deepEqual(parse('array<number>'), { kind: 'array', element: 'number' });
});

test('record generic type', (t) => {
    t.deepEqual(parse('record<number>'), {
        kind: 'record',
        fields: [],
        value: 'number',
    });
});

test('union type', (t) => {
    t.deepEqual(parse('string | number'), {
        kind: 'union',
        types: ['string', 'number'],
    });
    t.deepEqual(parse('true | false'), {
        kind: 'union',
        types: [
            { kind: 'literal', value: true },
            { kind: 'literal', value: false },
        ],
    });
});

test('union type with leading pipe', (t) => {
    t.deepEqual(parse('| string | number'), {
        kind: 'union',
        types: ['string', 'number'],
    });
});

test('record type', (t) => {
    t.deepEqual(parse('(a: number, b: string)'), {
        kind: 'record',
        fields: [
            { name: 'a', optional: false, type: 'number' },
            { name: 'b', optional: false, type: 'string' },
        ],
    });
});

test('record type with optional field', (t) => {
    t.deepEqual(parse('(a?: number)'), {
        kind: 'record',
        fields: [{ name: 'a', optional: true, type: 'number' }],
    });
});

test('record type with string field name', (t) => {
    t.deepEqual(parse('("field-name": number)'), {
        kind: 'record',
        fields: [{ name: 'field-name', optional: false, type: 'number' }],
    });
});

test('record type with trailing comma', (t) => {
    t.deepEqual(parse('(a: number,)'), {
        kind: 'record',
        fields: [{ name: 'a', optional: false, type: 'number' }],
    });
});

test('complex nested type', (t) => {
    t.deepEqual(
        parse(
            '((stroke?: string, "stroke-width"?: number, fill?: string, text?: string, "font-size"?: number, "font-family"?: string) | ("on"|"off",))[]',
        ),
        {
            kind: 'array',
            element: {
                kind: 'union',
                types: [
                    {
                        kind: 'record',
                        fields: [
                            { name: 'stroke', optional: true, type: 'string' },
                            { name: 'stroke-width', optional: true, type: 'number' },
                            { name: 'fill', optional: true, type: 'string' },
                            { name: 'text', optional: true, type: 'string' },
                            { name: 'font-size', optional: true, type: 'number' },
                            { name: 'font-family', optional: true, type: 'string' },
                        ],
                    },
                    {
                        kind: 'record',
                        fields: [
                            {
                                name: '0',
                                type: {
                                    kind: 'union',
                                    types: [
                                        {
                                            kind: 'literal',
                                            value: 'on',
                                        },
                                        {
                                            kind: 'literal',
                                            value: 'off',
                                        },
                                    ],
                                },
                            },
                        ],
                    },
                ],
            },
        },
    );
});

test('empty record', (t) => {
    t.deepEqual(parse('()'), { kind: 'record', fields: [] });
});

test('anonymous record fields', (t) => {
    t.deepEqual(parse('(number, string)'), {
        kind: 'record',
        fields: [
            { name: '0', type: 'number' },
            { name: '1', type: 'string' },
        ],
    });
});

test('single anonymous record field requires trailing comma', (t) => {
    t.deepEqual(parse('(number,)'), {
        kind: 'record',
        fields: [{ name: '0', type: 'number' }],
    });
    t.is(parse('(number)'), 'number');
});

test('invalid syntax throws', (t) => {
    t.throws(() => parse(''));
    t.throws(() => parse('number['));
});
