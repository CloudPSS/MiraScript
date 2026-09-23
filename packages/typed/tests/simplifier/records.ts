import test from 'ava';
import { parse, simplify } from '@mirascript/typed';

test('simplify preserves record types', (t) => {
    t.deepEqual(simplify(parse('(a: number, b: string)')), {
        kind: 'record',
        fields: [
            { name: 'a', optional: false, type: 'number' },
            { name: 'b', optional: false, type: 'string' },
        ],
    });
    t.deepEqual(simplify(parse('(a: number, ..record<string, string>)')), {
        kind: 'record',
        fields: [{ name: 'a', optional: false, type: 'number' }],
        rest: { kind: 'record', value: 'string' },
    });
});

test('simplify expands record rest type', (t) => {
    t.deepEqual(simplify(parse('(a: number, ..(b: string, c?: boolean))')), {
        kind: 'record',
        fields: [
            { name: 'a', optional: false, type: 'number' },
            { name: 'b', optional: false, type: 'string' },
            { name: 'c', optional: true, type: 'boolean' },
        ],
    });
    // Explicit fields take precedence over the fields spread from the rest type
    t.deepEqual(simplify(parse('(a: number, ..(a?: string, b: boolean))')), {
        kind: 'record',
        fields: [
            { name: 'a', optional: false, type: 'number' },
            { name: 'b', optional: false, type: 'boolean' },
        ],
    });
    // Empty and nested record spreads
    t.deepEqual(simplify(parse('(a: number, ..())')), {
        kind: 'record',
        fields: [{ name: 'a', optional: false, type: 'number' }],
    });
    t.deepEqual(simplify(parse('(a: number, ..(b: string, ..(c: boolean)))')), {
        kind: 'record',
        fields: [
            { name: 'a', optional: false, type: 'number' },
            { name: 'b', optional: false, type: 'string' },
            { name: 'c', optional: false, type: 'boolean' },
        ],
    });
});

test('simplify preserves nested record rest type', (t) => {
    t.deepEqual(simplify(parse('(a: number, ..(b: string, ..record<string, boolean>))')), {
        kind: 'record',
        fields: [
            { name: 'a', optional: false, type: 'number' },
            { name: 'b', optional: false, type: 'string' },
        ],
        rest: { kind: 'record', value: 'boolean' },
    });
});

test('simplify expandRecordSpreads option', (t) => {
    t.deepEqual(simplify(parse('(a: number, ..(b: string))'), { expandRecordSpreads: false }), {
        kind: 'record',
        fields: [{ name: 'a', optional: false, type: 'number' }],
        rest: {
            kind: 'record',
            fields: [{ name: 'b', optional: false, type: 'string' }],
        },
    });
});

test('simplify merges record rest types in intersections', (t) => {
    t.deepEqual(simplify(parse('(a: number, ..record<string, string>) & (b: boolean)')), {
        kind: 'record',
        fields: [
            { name: 'a', optional: false, type: 'number' },
            { name: 'b', optional: false, type: 'boolean' },
        ],
        rest: { kind: 'record', value: 'string' },
    });
    // Rest types of all members are merged into one rest type
    t.deepEqual(simplify(parse('(a: number, ..(c: string)) & (b: boolean, ..(c: string, d?: number))')), {
        kind: 'record',
        fields: [
            { name: 'a', optional: false, type: 'number' },
            { name: 'c', optional: false, type: 'string' },
            { name: 'b', optional: false, type: 'boolean' },
            { name: 'd', optional: true, type: 'number' },
        ],
    });
    t.deepEqual(simplify(parse('(a: number, ..(c: string)) & (b: boolean, ..record<string, nil>)')), {
        kind: 'record',
        fields: [
            { name: 'a', optional: false, type: 'number' },
            { name: 'c', optional: false, type: 'string' },
            { name: 'b', optional: false, type: 'boolean' },
        ],
        rest: { kind: 'record', value: 'nil' },
    });
    t.deepEqual(simplify(parse('(a: number, ..record<string, string>) & (b: boolean, ..record<string, number>)')), {
        kind: 'record',
        fields: [
            { name: 'a', optional: false, type: 'number' },
            { name: 'b', optional: false, type: 'boolean' },
        ],
        rest: {
            kind: 'intersection',
            types: [
                { kind: 'record', value: 'string' },
                { kind: 'record', value: 'number' },
            ],
        },
    });
});

test('simplify deduplicates record rest types', (t) => {
    t.deepEqual(simplify(parse('(a: number, ..record<string, boolean>) | (a: number, ..record<string, string>)')), {
        kind: 'union',
        types: [
            {
                kind: 'record',
                fields: [{ name: 'a', optional: false, type: 'number' }],
                rest: { kind: 'record', value: 'boolean' },
            },
            {
                kind: 'record',
                fields: [{ name: 'a', optional: false, type: 'number' }],
                rest: { kind: 'record', value: 'string' },
            },
        ],
    });
    t.deepEqual(simplify(parse('(a: number, ..record<string, boolean>) | (a: number, ..record<string, boolean>)')), {
        kind: 'record',
        fields: [{ name: 'a', optional: false, type: 'number' }],
        rest: { kind: 'record', value: 'boolean' },
    });
});
