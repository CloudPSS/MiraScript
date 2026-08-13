import test from 'ava';
import { parse } from '@mirascript/typed';

test('tuple type', (t) => {
    t.deepEqual(parse('[number, string]'), {
        kind: 'tuple',
        elements: [
            { type: 'number', spread: false },
            { type: 'string', spread: false },
        ],
    });
});

test('tuple type with rest element', (t) => {
    t.deepEqual(parse('[number, ..string[]]'), {
        kind: 'tuple',
        elements: [
            { type: 'number', spread: false },
            { type: { kind: 'array', element: 'string' }, spread: true },
        ],
    });
});

test('tuple type single element', (t) => {
    t.deepEqual(parse('[number]'), {
        kind: 'tuple',
        elements: [{ type: 'number', spread: false }],
    });
});

test('tuple type with trailing comma', (t) => {
    t.deepEqual(parse('[number,]'), {
        kind: 'tuple',
        elements: [{ type: 'number', spread: false }],
    });
});

test('tuple type with union elements', (t) => {
    t.deepEqual(parse('[string | number, boolean]'), {
        kind: 'tuple',
        elements: [
            { type: { kind: 'union', types: ['string', 'number'] }, spread: false },
            { type: 'boolean', spread: false },
        ],
    });
});

test('tuple type with nested tuple', (t) => {
    t.deepEqual(parse('[[number, string], boolean]'), {
        kind: 'tuple',
        elements: [
            {
                type: {
                    kind: 'tuple',
                    elements: [
                        { type: 'number', spread: false },
                        { type: 'string', spread: false },
                    ],
                },
                spread: false,
            },
            { type: 'boolean', spread: false },
        ],
    });
});

test('tuple type with function element', (t) => {
    t.deepEqual(parse('[fn(x: number) -> string, boolean]'), {
        kind: 'tuple',
        elements: [
            {
                type: {
                    kind: 'function',
                    params: [{ name: 'x', type: 'number' }],
                    returns: 'string',
                },
                spread: false,
            },
            { type: 'boolean', spread: false },
        ],
    });
});

test('empty tuple type', (t) => {
    t.deepEqual(parse('[]'), {
        kind: 'tuple',
        elements: [],
    });
});

test('tuple type with multiple rest elements', (t) => {
    t.deepEqual(parse('[..number[], ..string[]]'), {
        kind: 'tuple',
        elements: [
            { type: { kind: 'array', element: 'number' }, spread: true },
            { type: { kind: 'array', element: 'string' }, spread: true },
        ],
    });
});

test('tuple type with rest element in middle', (t) => {
    t.deepEqual(parse('[number, ..string[], boolean]'), {
        kind: 'tuple',
        elements: [
            { type: 'number', spread: false },
            { type: { kind: 'array', element: 'string' }, spread: true },
            { type: 'boolean', spread: false },
        ],
    });
});

test('tuple type with rest element at start', (t) => {
    t.deepEqual(parse('[..number[], string]'), {
        kind: 'tuple',
        elements: [
            { type: { kind: 'array', element: 'number' }, spread: true },
            { type: 'string', spread: false },
        ],
    });
});

test('tuple type with bare rest element (non-array)', (t) => {
    t.deepEqual(parse('[number, ..string]'), {
        kind: 'tuple',
        elements: [
            { type: 'number', spread: false },
            { type: 'string', spread: true },
        ],
    });
});

test('tuple type with user-type rest element', (t) => {
    t.deepEqual(parse('[number, ..MyType]'), {
        kind: 'tuple',
        elements: [
            { type: 'number', spread: false },
            { type: 'MyType', spread: true },
        ],
    });
});
