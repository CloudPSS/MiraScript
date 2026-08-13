import test from 'ava';
import { parse } from '@mirascript/typed';

test('priority of types', (t) => {
    t.deepEqual(parse('string | number[]'), {
        kind: 'union',
        types: ['string', { kind: 'array', element: 'number' }],
    });
    t.deepEqual(parse('(string | number)[]'), {
        kind: 'array',
        element: {
            kind: 'union',
            types: ['string', 'number'],
        },
    });
    t.deepEqual(parse('fn() -> boolean | string[]'), {
        kind: 'function',
        params: [],
        returns: {
            kind: 'union',
            types: ['boolean', { kind: 'array', element: 'string' }],
        },
    });
    t.deepEqual(parse('string | number & boolean[]'), {
        kind: 'union',
        types: [
            'string',
            {
                kind: 'intersection',
                types: ['number', { kind: 'array', element: 'boolean' }],
            },
        ],
    });
    t.deepEqual(parse('(string | number) & boolean[]'), {
        kind: 'intersection',
        types: [
            {
                kind: 'union',
                types: ['string', 'number'],
            },
            { kind: 'array', element: 'boolean' },
        ],
    });
    t.deepEqual(parse('fn() -> (boolean | string)[]'), {
        kind: 'function',
        params: [],
        returns: {
            kind: 'array',
            element: {
                kind: 'union',
                types: ['boolean', 'string'],
            },
        },
    });
    t.deepEqual(parse('(fn () -> boolean) | string[]'), {
        kind: 'union',
        types: [
            {
                kind: 'function',
                params: [],
                returns: 'boolean',
            },
            { kind: 'array', element: 'string' },
        ],
    });
    t.deepEqual(parse('fn() | string[]'), {
        kind: 'union',
        types: [
            {
                kind: 'function',
                params: [],
            },
            { kind: 'array', element: 'string' },
        ],
    });
    t.deepEqual(parse('fn(a: number) -> fn(b: boolean) -> string'), {
        kind: 'function',
        params: [{ name: 'a', type: 'number' }],
        returns: {
            kind: 'function',
            params: [{ name: 'b', type: 'boolean' }],
            returns: 'string',
        },
    });
});

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
