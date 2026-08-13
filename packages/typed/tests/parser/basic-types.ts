import test from 'ava';
import { parse } from '@mirascript/typed';

test('primitive types', (t) => {
    t.is(parse('nil'), 'nil');
    t.is(parse('string'), 'string');
    t.is(parse('number'), 'number');
    t.is(parse('boolean'), 'boolean');
    t.is(parse('record'), 'record');
    t.is(parse('array'), 'array');
});

test('named type', (t) => {
    t.is(parse('MyType'), 'MyType');
});

test('unicode named type', (t) => {
    t.is(parse('类型'), '类型');
    t.is(parse('My类型'), 'My类型');
});

test('special character named type', (t) => {
    t.is(parse('_MyType'), '_MyType');
    t.is(parse('$MyType'), '$MyType');
    t.is(parse('@MyType'), '@MyType');
    t.is(parse('$_MyType'), '$_MyType');
    t.throws(() => parse('1MyType'));
    t.throws(() => parse('$@MyType'));
});

test('boolean literal types', (t) => {
    t.deepEqual(parse('true'), { kind: 'literal', value: true });
    t.deepEqual(parse('false'), { kind: 'literal', value: false });
});

test('array type', (t) => {
    t.deepEqual(parse('number[]'), { kind: 'array', element: 'number' });
});

test('nested array type', (t) => {
    t.deepEqual(parse('number[][]'), {
        kind: 'array',
        element: { kind: 'array', element: 'number' },
    });
});

test('array generic type', (t) => {
    t.deepEqual(parse('array<number>'), { kind: 'array', element: 'number' });
    t.deepEqual(parse('array<any,>'), { kind: 'array', element: 'any' });
    t.throws(() => parse('array<number, string>'));
});

test('record generic type', (t) => {
    t.deepEqual(parse('record<number>'), {
        kind: 'record',
        value: 'number',
    });
    t.deepEqual(parse('record<string, number>'), {
        kind: 'record',
        key: 'string',
        value: 'number',
    });
    t.deepEqual(parse('record<"id" | "name", boolean>'), {
        kind: 'record',
        key: {
            kind: 'union',
            types: [
                { kind: 'literal', value: 'id' },
                { kind: 'literal', value: 'name' },
            ],
        },
        value: 'boolean',
    });
    t.throws(() => parse('record<number, string, boolean>'));
});
