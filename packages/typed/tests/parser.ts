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

test('string literal type', (t) => {
    t.deepEqual(parse('"hello"'), { kind: 'literal', value: 'hello' });
});

test('empty string literal type', (t) => {
    t.deepEqual(parse('""'), { kind: 'literal', value: '' });
});

test('single-quoted string literal type', (t) => {
    t.deepEqual(parse("'hello'"), { kind: 'literal', value: 'hello' });
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
