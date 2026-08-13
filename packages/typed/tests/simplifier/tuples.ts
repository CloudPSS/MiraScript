import test from 'ava';
import { parse, simplify } from '@mirascript/typed';

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
