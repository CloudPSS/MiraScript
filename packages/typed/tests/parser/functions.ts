import test from 'ava';
import { parse, type FunctionType } from '@mirascript/typed';

test('function type with return', (t) => {
    t.deepEqual(parse('fn(arg: number, ..rest: string) -> boolean'), {
        kind: 'function',
        params: [
            { name: 'arg', type: 'number' },
            { name: 'rest', type: 'string', spread: true },
        ],
        returns: 'boolean',
    });
});

test('function type with union types', (t) => {
    t.deepEqual(parse('fn(arg: number | string, ..rest: | string | string[]) -> | boolean | nil'), {
        kind: 'function',
        params: [
            { name: 'arg', type: { kind: 'union', types: ['number', 'string'] } },
            {
                name: 'rest',
                type: { kind: 'union', types: ['string', { kind: 'array', element: 'string' }] },
                spread: true,
            },
        ],
        returns: { kind: 'union', types: ['boolean', 'nil'] },
    });
});

test('function type without return', (t) => {
    t.deepEqual(parse('fn(arg: number)'), {
        kind: 'function',
        params: [{ name: 'arg', type: 'number' }],
    });
});

test('function type without parameters', (t) => {
    t.deepEqual(parse('fn() -> number'), {
        kind: 'function',
        params: [],
        returns: 'number',
    });
});

test('function type with callback parameter', (t) => {
    t.deepEqual(parse('fn(arg: number, callback: fn(result: string, error: any) -> any)'), {
        kind: 'function',
        params: [
            { name: 'arg', type: 'number' },
            {
                name: 'callback',
                type: {
                    kind: 'function',
                    params: [
                        { name: 'result', type: 'string' },
                        { name: 'error', type: 'any' },
                    ],
                    returns: 'any',
                },
            },
        ],
    });
});

test('function type with trailing comma', (t) => {
    t.deepEqual(parse('fn(a: number,) -> string'), {
        kind: 'function',
        params: [{ name: 'a', type: 'number' }],
        returns: 'string',
    });
    t.deepEqual(parse('fn(..,)'), {
        kind: 'function',
        params: [{ name: '', type: { kind: 'array', element: 'any' }, spread: true }],
    });
    t.deepEqual(parse('fn(..rest,) -> string'), {
        kind: 'function',
        params: [{ name: 'rest', type: { kind: 'array', element: 'any' }, spread: true }],
        returns: 'string',
    });
});

test('named function type at top level', (t) => {
    const result = parse('fn fnName<T>(arg: T) -> any') as FunctionType;
    t.is(result.kind, 'function');
    t.is(result.name, 'fnName');
    t.is(result.typeParams?.length, 1);
    t.is(result.params[0].name, 'arg');
    t.is(result.params[0].type, result.typeParams![0]);
    t.is(result.returns, 'any');
});

test('nested function type cannot have a name', (t) => {
    t.throws(() => parse('fn(callback: fn fnName(x: any) -> any)'));
    t.throws(() => parse('fn() -> fn fnName() -> any'));
});

test('function type with omitted param types', (t) => {
    t.deepEqual(parse('fn(a, b: number, c, ..d) -> string'), {
        kind: 'function',
        params: [
            { name: 'a', type: 'any' },
            { name: 'b', type: 'number' },
            { name: 'c', type: 'any' },
            { name: 'd', type: { kind: 'array', element: 'any' }, spread: true },
        ],
        returns: 'string',
    });
});

test('function type with omitted rest param name', (t) => {
    t.deepEqual(parse('fn(..) -> string'), {
        kind: 'function',
        params: [{ name: '', type: { kind: 'array', element: 'any' }, spread: true }],
        returns: 'string',
    });
});
