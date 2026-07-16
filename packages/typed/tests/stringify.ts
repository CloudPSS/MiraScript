import test, { type ExecutionContext } from 'ava';
import { parse } from '../dist/parser.js';
import { stringify } from '../dist/stringify.js';
import { simplify } from '../dist/simplifier.js';

function rt(typeStr: string, t: ExecutionContext) {
    const simplified = simplify(parse(typeStr));
    const str = stringify(simplified);
    const reparsed = simplify(parse(str));
    t.deepEqual(reparsed, simplified);
}

// ===========================================================================
// Primitives
// ===========================================================================

test('stringify primitives', (t) => {
    t.is(stringify(simplify(parse('nil'))), 'nil');
    t.is(stringify(simplify(parse('string'))), 'string');
    t.is(stringify(simplify(parse('number'))), 'number');
    t.is(stringify(simplify(parse('boolean'))), 'boolean');
    t.is(stringify(simplify(parse('record'))), 'record');
    t.is(stringify(simplify(parse('array'))), 'array');
    t.is(stringify(simplify(parse('any'))), 'any');
    t.is(stringify(simplify(parse('unknown'))), 'unknown');
    t.is(stringify(simplify(parse('never'))), 'never');
    t.is(stringify(simplify(parse('extern'))), 'extern');
});

test('stringify named type', (t) => {
    t.is(stringify(simplify(parse('MyType'))), 'MyType');
    t.is(stringify(simplify(parse('type'))), 'type');
});

test('stringify literals', (t) => {
    t.is(stringify(simplify(parse('true'))), 'true');
    t.is(stringify(simplify(parse('false'))), 'false');
    t.is(stringify(simplify(parse('"hello"'))), '"hello"');
    // single quotes normalize to double quotes after parse
    t.is(stringify(simplify(parse("'hello'"))), '"hello"');
});

// ===========================================================================
// Compound types
// ===========================================================================

test('stringify array', (t) => {
    t.is(stringify(simplify(parse('number[]'))), 'number[]');
    t.is(stringify(simplify(parse('array<number>'))), 'number[]');
});

test('stringify union roundtrip', (t) => {
    rt('string | number', t);
    rt('string | number | boolean', t);
});

test('stringify intersection roundtrip', (t) => {
    rt('string & number', t);
});

test('stringify record fields', (t) => {
    t.is(stringify(simplify(parse('()'))), '()');
    t.is(stringify(simplify(parse('(number,)'))), '(number,)');
    t.is(stringify(simplify(parse('(number, string)'))), '(number, string)');
    t.is(stringify(simplify(parse('(a: number)'))), '(a: number)');
    t.is(stringify(simplify(parse('(a: number, b: string)'))), '(a: number, b: string)');
    t.is(stringify(simplify(parse('(a?: number)'))), '(a?: number)');
});

test('stringify record generic', (t) => {
    t.is(stringify(simplify(parse('record<number>'))), 'record<number>');
    t.is(stringify(simplify(parse('record<string, number>'))), 'record<number>');
    t.is(stringify(simplify(parse('record<number, number>'))), 'record<number, number>');
    t.is(stringify(simplify(parse('record<"$(string)", number>'))), 'record<"$(string)", number>');
});

test('stringify function', (t) => {
    t.is(stringify(simplify(parse('fn()'))), 'fn()');
    t.is(stringify(simplify(parse('fn() -> number'))), 'fn() -> number');
    t.is(stringify(simplify(parse('fn(x: number)'))), 'fn(x: number)');
    t.is(stringify(simplify(parse('fn(x: number, y: string) -> boolean'))), 'fn(x: number, y: string) -> boolean');
    t.is(stringify(simplify(parse('fn(x: number, ..rest: string)'))), 'fn(x: number, ..rest: string)');
    t.is(stringify(simplify(parse('fn<T, U>(x: T, ..rest: U[])'))), 'fn<T, U>(x: T, ..rest: U[])');
});

test('stringify function in union roundtrip', (t) => {
    rt('(fn(value: any) -> boolean) | any', t);
});

test('stringify function return union roundtrip', (t) => {
    rt('fn(index: number) -> (string, any) | nil', t);
});

// ===========================================================================
// Tuple
// ===========================================================================

test('stringify tuple', (t) => {
    t.is(stringify(simplify(parse('[]'))), '[]');
    t.is(stringify(simplify(parse('[number]'))), '[number]');
    t.is(stringify(simplify(parse('[number, string]'))), '[number, string]');
    t.is(stringify(simplify(parse('[number, ..string[]]'))), '[number, ..string[]]');
});

// ===========================================================================
// Reflection
// ===========================================================================

test('stringify reflection roundtrip', (t) => {
    rt('type(MyVar)', t);
    rt('type(A) | string', t);
    rt('type(type) | type', t);
});

// ===========================================================================
// Template
// ===========================================================================

test('stringify template', (t) => {
    t.is(stringify(simplify(parse('`hello $(name)`'))), '`hello $(name)`');
});
