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
