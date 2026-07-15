import test from 'ava';
import { parse } from '../dist/parser.js';
import { simplify } from '../dist/simplifier.js';

test('simplify flattens unions and intersections', (t) => {
    t.deepEqual(simplify({ kind: 'union', types: ['string', { kind: 'union', types: ['number'] }] }), {
        kind: 'union',
        types: ['string', 'number'],
    });
    t.deepEqual(simplify({ kind: 'intersection', types: ['string', { kind: 'intersection', types: ['number'] }] }), {
        kind: 'intersection',
        types: ['string', 'number'],
    });
    t.is(simplify({ kind: 'union', types: ['string'] }), 'string');
    t.is(simplify({ kind: 'intersection', types: ['string'] }), 'string');
    t.is(simplify(parse('string | string')), 'string');
    t.is(simplify(parse('string & string')), 'string');
    t.deepEqual(simplify(parse('string & string & (a: string) & (a: string, b?: number)')), {
        kind: 'intersection',
        types: [
            {
                kind: 'record',
                fields: [
                    { name: 'a', optional: false, type: 'string' },
                    { name: 'b', optional: true, type: 'number' },
                ],
            },
            'string',
        ],
    });
});

test('simplify distributes intersections over unions and merges record fields', (t) => {
    t.deepEqual(simplify(parse('(t: string) & ((x: number) | (y: number))')), {
        kind: 'union',
        types: [
            {
                kind: 'record',
                fields: [
                    { name: 't', optional: false, type: 'string' },
                    { name: 'x', optional: false, type: 'number' },
                ],
            },
            {
                kind: 'record',
                fields: [
                    { name: 't', optional: false, type: 'string' },
                    { name: 'y', optional: false, type: 'number' },
                ],
            },
        ],
    });
    t.deepEqual(simplify(parse('(a: number) & (a?: string) & () & (a: false)')), {
        kind: 'record',
        fields: [
            {
                name: 'a',
                optional: false,
                type: {
                    kind: 'intersection',
                    types: ['number', 'string', { kind: 'literal', value: false }],
                },
            },
        ],
    });
    t.deepEqual(simplify(parse('(a: number) & string & (b: boolean)')), {
        kind: 'intersection',
        types: [
            {
                kind: 'record',
                fields: [
                    { name: 'a', optional: false, type: 'number' },
                    { name: 'b', optional: false, type: 'boolean' },
                ],
            },
            'string',
        ],
    });
});

test('simplify options can disable individual passes', (t) => {
    t.deepEqual(simplify(parse('string | (number | boolean)'), { flattenUnions: false }), {
        kind: 'union',
        types: ['string', { kind: 'union', types: ['number', 'boolean'] }],
    });
    t.deepEqual(simplify(parse('string & (number | boolean)'), { distributeIntersectionsOverUnions: false }), {
        kind: 'intersection',
        types: ['string', { kind: 'union', types: ['number', 'boolean'] }],
    });
    t.deepEqual(simplify(parse('(a: number) & (b: string)'), { mergeRecordIntersections: false }), {
        kind: 'intersection',
        types: [
            { kind: 'record', fields: [{ name: 'a', optional: false, type: 'number' }] },
            { kind: 'record', fields: [{ name: 'b', optional: false, type: 'string' }] },
        ],
    });
    t.deepEqual(simplify(parse('string | string'), { deduplicateUnions: false }), {
        kind: 'union',
        types: ['string', 'string'],
    });
    t.deepEqual(simplify(parse('string & string'), { deduplicateIntersections: false }), {
        kind: 'intersection',
        types: ['string', 'string'],
    });
});

test('simplify preserves tuple types', (t) => {
    t.deepEqual(simplify(parse('[number, string]')), {
        kind: 'tuple',
        elements: [
            { type: 'number', spread: false },
            { type: 'string', spread: false },
        ],
    });
    t.deepEqual(simplify(parse('[number, ..string[]]')), {
        kind: 'tuple',
        elements: [
            { type: 'number', spread: false },
            { type: { kind: 'array', element: 'string' }, spread: true },
        ],
    });
});

test('simplify expands tuple spread', (t) => {
    t.deepEqual(simplify(parse('[string, ..[string, number]]')), {
        kind: 'tuple',
        elements: [
            { type: 'string', spread: false },
            { type: 'string', spread: false },
            { type: 'number', spread: false },
        ],
    });
    t.deepEqual(simplify(parse('[string, ..[string, number], ..unknown, ..[], ..[boolean, "X"], ..number[]]')), {
        kind: 'tuple',
        elements: [
            { type: 'string', spread: false },
            { type: 'string', spread: false },
            { type: 'number', spread: false },
            { type: 'unknown', spread: true },
            { type: 'boolean', spread: false },
            { type: { kind: 'literal', value: 'X' }, spread: false },
            { type: { kind: 'array', element: 'number' }, spread: true },
        ],
    });
});

test('simplify expands nested tuple spread', (t) => {
    t.deepEqual(simplify(parse('[string, ..[string, ..[number, boolean]]]')), {
        kind: 'tuple',
        elements: [
            { type: 'string', spread: false },
            { type: 'string', spread: false },
            { type: 'number', spread: false },
            { type: 'boolean', spread: false },
        ],
    });
});

test('simplify expands tuple spread preserving inner rest', (t) => {
    t.deepEqual(simplify(parse('[string, ..[string, ..number[]]]')), {
        kind: 'tuple',
        elements: [
            { type: 'string', spread: false },
            { type: 'string', spread: false },
            { type: { kind: 'array', element: 'number' }, spread: true },
        ],
    });
});

test('simplify expandTupleSpreads option', (t) => {
    t.deepEqual(simplify(parse('[string, ..[string, number]]'), { expandTupleSpreads: false }), {
        kind: 'tuple',
        elements: [
            { type: 'string', spread: false },
            {
                type: {
                    kind: 'tuple',
                    elements: [
                        { type: 'string', spread: false },
                        { type: 'number', spread: false },
                    ],
                },
                spread: true,
            },
        ],
    });
});

test('simplify deduplicates tuple types', (t) => {
    t.deepEqual(
        simplify({
            kind: 'union',
            types: [
                {
                    kind: 'tuple',
                    elements: [
                        { type: 'number', spread: false },
                        { type: 'string', spread: false },
                    ],
                },
                {
                    kind: 'tuple',
                    elements: [
                        { type: 'number', spread: false },
                        { type: 'string', spread: false },
                    ],
                },
            ],
        }),
        {
            kind: 'tuple',
            elements: [
                { type: 'number', spread: false },
                { type: 'string', spread: false },
            ],
        },
    );
});

test('simplify preserves reflection types', (t) => {
    t.deepEqual(simplify(parse('type(MyVar)')), {
        kind: 'reflection',
        name: 'MyVar',
    });
    t.deepEqual(simplify(parse('type(MyVar) | string')), {
        kind: 'union',
        types: [{ kind: 'reflection', name: 'MyVar' }, 'string'],
    });
});

test('simplify top types in unions', (t) => {
    t.is(simplify(parse('unknown | string')), 'unknown');
    t.is(simplify(parse('any | string')), 'any');
    t.is(simplify(parse('string | never')), 'string');
    t.deepEqual(simplify(parse('(string | never) | number')), {
        kind: 'union',
        types: ['string', 'number'],
    });
    t.is(simplify(parse('never | never')), 'never');
    t.is(simplify(parse('never | any')), 'any');
    t.is(simplify(parse('never | unknown')), 'unknown');
    t.is(simplify(parse('unknown | unknown')), 'unknown');
    t.is(simplify(parse('unknown | any')), 'any');
    t.is(simplify(parse('any | any')), 'any');
});

test('simplify top types in intersections', (t) => {
    t.is(simplify(parse('never & string')), 'never');
    t.is(simplify(parse('string & unknown')), 'string');
    t.is(simplify(parse('string & any')), 'string');
    t.deepEqual(simplify(parse('(string & any) & number')), {
        kind: 'intersection',
        types: ['string', 'number'],
    });
    t.is(simplify(parse('never & never')), 'never');
    t.is(simplify(parse('never & any')), 'never');
    t.is(simplify(parse('never & unknown')), 'never');
    t.is(simplify(parse('unknown & unknown')), 'unknown');
    t.is(simplify(parse('unknown & any')), 'any');
    t.is(simplify(parse('any & any')), 'any');
});

test('simplify top types option as array', (t) => {
    // Only absorb unknown in unions
    t.is(simplify(parse('unknown | string'), { simplifyTopTypesInUnions: ['unknown'] }), 'unknown');
    t.deepEqual(simplify(parse('any | string'), { simplifyTopTypesInUnions: ['unknown'] }), {
        kind: 'union',
        types: ['any', 'string'],
    });
    // never is not in the list, so it stays — verify roundtrip
    t.deepEqual(simplify(parse('string | never'), { simplifyTopTypesInUnions: ['unknown'] }), {
        kind: 'union',
        types: ['string', 'never'],
    });

    // Only eliminate never in intersections
    t.is(simplify(parse('never & string'), { simplifyTopTypesInIntersections: ['never'] }), 'never');
    t.deepEqual(simplify(parse('string & unknown'), { simplifyTopTypesInIntersections: ['never'] }), {
        kind: 'intersection',
        types: ['string', 'unknown'],
    });
});

test('simplify top types option disabled', (t) => {
    // With false, top types are left untouched
    t.deepEqual(simplify(parse('unknown | string'), { simplifyTopTypesInUnions: false }), {
        kind: 'union',
        types: ['unknown', 'string'],
    });

    t.deepEqual(simplify(parse('never & string'), { simplifyTopTypesInIntersections: false }), {
        kind: 'intersection',
        types: ['never', 'string'],
    });
});

test('simplify normalize generic record', (t) => {
    // record<string, V> → record<V>
    t.deepEqual(simplify(parse('record<string, number>')), {
        kind: 'record',
        value: 'number',
    });
    // record<number, V> unchanged
    t.deepEqual(simplify(parse('record<number, boolean>')), {
        kind: 'record',
        key: 'number',
        value: 'boolean',
    });
});

test('simplify normalize generic array', (t) => {
    // array<any> → array
    t.is(simplify(parse('any[]')), 'array');
    t.is(simplify(parse('unknown[]')), 'array');
    // unchanged
    t.deepEqual(simplify(parse('number[]')), {
        kind: 'array',
        element: 'number',
    });
    t.deepEqual(simplify(parse('never[]')), {
        kind: 'array',
        element: 'never',
    });
});

test('simplify normalize options disabled', (t) => {
    t.deepEqual(simplify(parse('record<string, number>'), { normalizeGenericRecord: false }), {
        kind: 'record',
        key: 'string',
        value: 'number',
    });
    t.deepEqual(simplify(parse('array<any>'), { normalizeGenericArray: false }), {
        kind: 'array',
        element: 'any',
    });
});
