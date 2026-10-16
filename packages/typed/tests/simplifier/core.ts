import test from 'ava';
import { type FunctionType, parse, simplify } from '@mirascript/typed';

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

test('simplify dedup generics', (t) => {
    const ty = parse('fn <T>() -> (T | T & T)');
    const sty = simplify(ty) as FunctionType;
    t.deepEqual(sty, {
        kind: 'function',
        typeParams: [sty.returns],
        params: [],
        returns: sty.returns,
    });
});

test('simplify dedup preserves types with separators in names and literals', (t) => {
    for (const source of [
        '("a:false:string:number,b": string) | (a: number, b: string)',
        '["a,false:literal:string:b", "c"] | ["a", "b,false:literal:string:c"]',
    ]) {
        t.deepEqual(simplify(parse(source)), parse(source));
    }
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
