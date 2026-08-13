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
