import test from 'ava';
import { parse } from '@mirascript/typed';

test('reflection type', (t) => {
    t.deepEqual(parse('type(MyVar)'), {
        kind: 'reflection',
        name: 'MyVar',
    });
});

test('reflection type with keyword-like name', (t) => {
    t.deepEqual(parse('type(string)'), {
        kind: 'reflection',
        name: 'string',
    });
});

test('reflection type in union', (t) => {
    t.deepEqual(parse('type(A) | string'), {
        kind: 'union',
        types: [{ kind: 'reflection', name: 'A' }, 'string'],
    });
});

test('tuples and reflection types together', (t) => {
    t.deepEqual(parse('[type(A), ..type(B)[]]'), {
        kind: 'tuple',
        elements: [
            { type: { kind: 'reflection', name: 'A' }, spread: false },
            { type: { kind: 'array', element: { kind: 'reflection', name: 'B' } }, spread: true },
        ],
    });
});

test('type as a regular named type', (t) => {
    t.is(parse('type'), 'type');
});

test('type as a regular named type in union', (t) => {
    t.deepEqual(parse('type | string'), {
        kind: 'union',
        types: ['type', 'string'],
    });
});

test('type as a regular named type in record generic', (t) => {
    t.deepEqual(parse('record<type, number>'), {
        kind: 'record',
        key: 'type',
        value: 'number',
    });
});

test('reflection of type itself', (t) => {
    t.deepEqual(parse('type(type)'), {
        kind: 'reflection',
        name: 'type',
    });
});

test('reflection of type in union', (t) => {
    t.deepEqual(parse('type(type) | string'), {
        kind: 'union',
        types: [{ kind: 'reflection', name: 'type' }, 'string'],
    });
});

test('type as a regular named type in function param', (t) => {
    t.deepEqual(parse('fn(x: type) -> type'), {
        kind: 'function',
        params: [{ name: 'x', type: 'type' }],
        returns: 'type',
    });
});
