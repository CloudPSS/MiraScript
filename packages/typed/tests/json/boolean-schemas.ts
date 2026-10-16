import test from 'ava';
import { parse, toJSONSchema } from '@mirascript/typed';

test('array items prefer boolean schemas', (t) => {
    for (const type of ['array', 'any[]', 'unknown[]', 'MyType[]', 'type(A)[]']) {
        t.is(toJSONSchema(parse(type)).items, true);
    }
    for (const type of ['never[]', 'array<fn() -> number>']) {
        t.is(toJSONSchema(parse(type)).items, false);
    }
    t.deepEqual(toJSONSchema(parse('true[]')).items, { const: true });
    t.deepEqual(toJSONSchema(parse('false[]')).items, { const: false });
});

test('record properties prefer boolean schemas and preserve literal constraints', (t) => {
    t.deepEqual(toJSONSchema(parse('(a: any, b?: never, c: MyType, d: fn(), e: true, f: false)')).properties, {
        a: true,
        b: false,
        c: true,
        d: false,
        e: { const: true },
        f: { const: false },
    });
    t.deepEqual(toJSONSchema(parse('record<"a" | "b", any>')).properties, { a: true, b: true });
    t.deepEqual(toJSONSchema(parse('record<"a", never>')).properties, { a: false });
    t.deepEqual(toJSONSchema(parse('(..record<"a", never>)')).properties, { a: false });
});

test('record values and rests prefer boolean schemas', (t) => {
    for (const [value, expected] of [
        ['any', true],
        ['never', false],
        ['fn()', false],
    ] as const) {
        t.is(toJSONSchema(parse(`record<${value}>`)).additionalProperties, expected);
        t.is(toJSONSchema(parse(`(a: number, ..record<${value}>)`)).additionalProperties, expected);
        t.deepEqual(Object.values(toJSONSchema(parse(`record<number, ${value}>`)).patternProperties ?? {}), [expected]);
    }
});

test('heterogeneous tuple prefix items prefer boolean schemas', (t) => {
    const result = toJSONSchema(parse('[any, never, true, false]'));
    t.deepEqual(result.prefixItems, [true, false, { const: true }, { const: false }]);
    t.is(result.items, false);
    t.is(result.minItems, 4);
    t.is(result.maxItems, 4);
});
