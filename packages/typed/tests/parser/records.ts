import test from 'ava';
import { parse } from '@mirascript/typed';

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

test('record type with bad field name', (t) => {
    t.throws(() => parse('(1a: number)'));
    t.throws(() => parse('("field$(name)": number)'));
});

test('record type with rest field', (t) => {
    t.deepEqual(parse('(a: number, ..record<string, string>)'), {
        kind: 'record',
        fields: [{ name: 'a', optional: false, type: 'number' }],
        rest: { kind: 'record', key: 'string', value: 'string' },
    });
});

test('record type with only a rest field', (t) => {
    t.deepEqual(parse('(..MyType)'), {
        kind: 'record',
        fields: [],
        rest: 'MyType',
    });
});

test('record type with rest field of a record literal', (t) => {
    t.deepEqual(parse('(a: number, ..(b: string, c?: boolean))'), {
        kind: 'record',
        fields: [{ name: 'a', optional: false, type: 'number' }],
        rest: {
            kind: 'record',
            fields: [
                { name: 'b', optional: false, type: 'string' },
                { name: 'c', optional: true, type: 'boolean' },
            ],
        },
    });
});

test('record type with anonymous fields and rest field', (t) => {
    t.deepEqual(parse('(number, ..rule)'), {
        kind: 'record',
        fields: [{ name: '0', type: 'number' }],
        rest: 'rule',
    });
});

test('record type with trailing comma after rest field', (t) => {
    t.deepEqual(parse('(a: number, ..rule,)'), {
        kind: 'record',
        fields: [{ name: 'a', optional: false, type: 'number' }],
        rest: 'rule',
    });
});

test('record type with mispositioned rest field', (t) => {
    t.throws(() => parse('(..rule, a: number)'));
    t.throws(() => parse('(..rule, ..other)'));
    t.throws(() => parse('(..)'));
});

test('record rest resolves enclosing generic parameters', (t) => {
    const direct = parse('fn<T>(x: (..T)) -> T');
    if (typeof direct !== 'object' || direct.kind !== 'function') return t.fail('Expected a function');
    t.deepEqual(direct.params[0]?.type, { kind: 'record', fields: [], rest: direct.typeParams?.[0] });

    const nested = parse('fn<T>(x: (..(y: T))) -> T');
    if (typeof nested !== 'object' || nested.kind !== 'function') return t.fail('Expected a function');
    t.deepEqual(nested.params[0]?.type, {
        kind: 'record',
        fields: [],
        rest: { kind: 'record', fields: [{ name: 'y', optional: false, type: nested.typeParams?.[0] }] },
    });
});
