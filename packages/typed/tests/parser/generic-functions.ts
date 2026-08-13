import test from 'ava';
import { parse, type FunctionType, type TemplateType } from '@mirascript/typed';

test('generic function type', (t) => {
    const result = parse('fn<T, U>(arg: T) -> U') as FunctionType;
    t.is(result.kind, 'function');
    t.is(result.typeParams?.length, 2);
    t.is(typeof result.typeParams![0], 'symbol');
    t.is(typeof result.typeParams![1], 'symbol');
    t.is(result.typeParams![0].description, 'T');
    t.is(result.typeParams![1].description, 'U');
    t.is(result.params[0].type, result.typeParams![0]);
    t.is(result.returns, result.typeParams![1]);
});

test('generic function type with single type parameter', (t) => {
    const result = parse('fn<T>(x: T) -> T') as FunctionType;
    t.is(result.kind, 'function');
    t.is(result.typeParams?.length, 1);
    t.is(typeof result.typeParams![0], 'symbol');
    t.is(result.typeParams![0].description, 'T');
    t.is(result.params[0].type, result.typeParams![0]);
    t.is(result.returns, result.typeParams![0]);
});

test('generic function type without parameters and return', (t) => {
    const result = parse('fn<T,>()') as FunctionType;
    t.is(result.kind, 'function');
    t.is(result.typeParams?.length, 1);
    t.is(typeof result.typeParams![0], 'symbol');
    t.is(result.params.length, 0);
});

test('nested generic function type', (t) => {
    const result = parse('fn<T>(callback: fn<U>(x: U) -> T) -> T') as FunctionType;
    t.is(result.kind, 'function');
    const outerT = result.typeParams![0];
    const innerFn = result.params[0].type as FunctionType;
    const innerU = innerFn.typeParams![0];
    t.is(outerT.description, 'T');
    t.is(innerU.description, 'U');
    t.is(innerFn.params[0].type, innerU);
    t.is(innerFn.returns, outerT);
    t.is(result.returns, outerT);
});

test('nested generic function with same name uses different symbols', (t) => {
    const result = parse('fn<T>(arg: T, callback: fn<T>(data: T))') as FunctionType;
    const outerT = result.typeParams![0];
    const innerFn = result.params[1].type as FunctionType;
    const innerT = innerFn.typeParams![0];
    t.not(outerT, innerT);
    t.is(result.params[0].type, outerT);
    t.is(innerFn.params[0].type, innerT);
});

test('complex generic function type', (t) => {
    const result = parse(
        'fn<T, U>(arg: record<T, U>, callback: fn<V>(data: V) -> "$(T | U | V)") -> T[] | U',
    ) as FunctionType;
    const outerT = result.typeParams![0];
    const outerU = result.typeParams![1];
    const outerArg = result.params[0].type;
    t.deepEqual(outerArg, { kind: 'record', key: outerT, value: outerU });
    const innerFn = result.params[1].type as FunctionType;
    const innerV = innerFn.typeParams![0];
    t.is(innerFn.params[0].type, innerV);
    const template = innerFn.returns as TemplateType;
    t.deepEqual(template, {
        kind: 'template',
        parts: [
            {
                kind: 'union',
                types: [outerT, outerU, innerV],
            },
        ],
    });
    t.deepEqual(result.returns, { kind: 'union', types: [{ kind: 'array', element: outerT }, outerU] });

    t.is(outerT.description, 'T');
    t.is(outerU.description, 'U');
    t.is(innerV.description, 'V');
});

test('rest parameter must be the last parameter', (t) => {
    t.throws(() => parse('fn(..a, b) -> string'));
    t.throws(() => parse('fn(a, ..b, c) -> string'));
    t.throws(() => parse('fn(..a, ..b) -> string'));
});

test('invalid syntax throws', (t) => {
    t.throws(() => parse(''));
    t.throws(() => parse('number['));
    t.throws(() => parse('fn'));
    t.throws(() => parse('fn | "12"'));
    t.throws(() => parse('fn(,)'));
    t.throws(() => parse('any<x>'));
    t.throws(() => parse('fn fn()'));
});
