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
});
