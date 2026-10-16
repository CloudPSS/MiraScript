import test from 'ava';
import { parse, toJSONSchema } from '@mirascript/typed';

const schema = (s: object) => ({ $schema: 'https://json-schema.org/draft/2020-12/schema', ...s });

test('tuple JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[number, string]')),
        schema({
            type: 'array',
            prefixItems: [{ type: 'number' }, { type: 'string' }],
            items: false,
            minItems: 2,
            maxItems: 2,
        }),
    );
});

test('tuple with rest element JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[number, ..string[]]')),
        schema({
            type: 'array',
            prefixItems: [{ type: 'number' }],
            items: { type: 'string' },
            minItems: 1,
        }),
    );
});

test('tuple with rest element in middle JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[number, ..string[], boolean]')),
        schema({
            type: 'array',
            prefixItems: [{ type: 'number' }],
            items: { anyOf: [{ type: 'string' }, { type: 'boolean' }] },
            minItems: 2,
        }),
    );
});

test('tuple with rest element at start JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[..number[], string]')),
        schema({
            type: 'array',
            items: { anyOf: [{ type: 'number' }, { type: 'string' }] },
            minItems: 1,
        }),
    );
});

test('tuple with multiple rest element JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[number, ..string[], ..boolean[], number]')),
        schema({
            type: 'array',
            prefixItems: [{ type: 'number' }],
            items: {
                anyOf: [{ type: 'string' }, { type: 'boolean' }, { type: 'number' }],
            },
            minItems: 2,
        }),
    );
});

test('tuple with bare rest element JSON schema (non-array)', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[number, ..string]')),
        schema({
            type: 'array',
            prefixItems: [{ type: 'number' }],
            items: true,
            minItems: 1,
        }),
    );
});

test('tuple with user-type rest element JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[number, ..MyType]')),
        schema({
            type: 'array',
            prefixItems: [{ type: 'number' }],
            items: true,
            minItems: 1,
        }),
    );
});

test('tuple with bare rest in middle JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[number, ..string, boolean]')),
        schema({
            type: 'array',
            prefixItems: [{ type: 'number' }],
            items: true,
            minItems: 2,
        }),
    );
});

test('single element tuple JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[number]')),
        schema({
            type: 'array',
            items: { type: 'number' },
            minItems: 1,
            maxItems: 1,
        }),
    );
});

test('empty tuple JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[]')),
        schema({
            type: 'array',
            items: false,
            minItems: 0,
            maxItems: 0,
        }),
    );
});

test('nested tuple JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[[number, string], boolean]')),
        schema({
            type: 'array',
            prefixItems: [
                {
                    type: 'array',
                    prefixItems: [{ type: 'number' }, { type: 'string' }],
                    items: false,
                    minItems: 2,
                    maxItems: 2,
                },
                { type: 'boolean' },
            ],
            items: false,
            minItems: 2,
            maxItems: 2,
        }),
    );
});

test('homogeneous tuples use one items schema and retain their length', (t) => {
    for (const type of ['[number, number]', '[number, ..[number]]', '[number | never, number]']) {
        t.deepEqual(
            toJSONSchema(parse(type)),
            schema({ type: 'array', items: { type: 'number' }, minItems: 2, maxItems: 2 }),
        );
    }
    t.deepEqual(
        toJSONSchema(parse('[[number, number], [number, number]]')),
        schema({
            type: 'array',
            items: { type: 'array', items: { type: 'number' }, minItems: 2, maxItems: 2 },
            minItems: 2,
            maxItems: 2,
        }),
    );
    for (const [type, items] of [
        ['[any, unknown]', true],
        ['[never, never]', false],
    ] as const) {
        t.deepEqual(toJSONSchema(parse(type)), schema({ type: 'array', items, minItems: 2, maxItems: 2 }));
    }
});

test('homogeneous tuple rests use one items schema without an upper bound', (t) => {
    for (const type of ['[number, ..number[]]', '[..number[], number]', '[..number[], number, ..number[]]']) {
        t.deepEqual(toJSONSchema(parse(type)), schema({ type: 'array', items: { type: 'number' }, minItems: 1 }));
        t.deepEqual(toJSONSchema(parse(type), { loose: true }), schema({ type: 'array', items: { type: 'number' } }));
    }
    t.deepEqual(toJSONSchema(parse('[..number[]]')), schema({ type: 'array', items: { type: 'number' }, minItems: 0 }));
});

test('tuple homogeneity distinguishes type keys containing separators', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[("a,literal:string:b" | "c"), ("a" | "b,literal:string:c")]')),
        schema({
            type: 'array',
            prefixItems: [{ enum: ['a,literal:string:b', 'c'] }, { enum: ['a', 'b,literal:string:c'] }],
            items: false,
            minItems: 2,
            maxItems: 2,
        }),
    );
    const result = toJSONSchema(parse('[("a:false:string:number,b": string), (a: number, b: string)]'));
    t.is((result.prefixItems as unknown[])?.length, 2);
    t.deepEqual((result.prefixItems as unknown[])[0], {
        type: 'object',
        properties: { 'a:false:string:number,b': { type: 'string' } },
        required: ['a:false:string:number,b'],
        additionalProperties: false,
    });
});

test('tuple homogeneity shares symbol identities across element types', (t) => {
    const a = Symbol('T');
    const b = Symbol('T');
    const field = (type: symbol) => ({ kind: 'record' as const, fields: [{ name: 'value', type }] });
    const distinct = toJSONSchema({ kind: 'tuple', elements: [{ type: field(a) }, { type: field(b) }] });
    t.is(distinct.prefixItems?.length, 2);
    const same = toJSONSchema({ kind: 'tuple', elements: [{ type: field(a) }, { type: field(a) }] });
    t.is(same.prefixItems, undefined);
    t.deepEqual(same.items, {
        type: 'object',
        properties: { value: true },
        required: ['value'],
        additionalProperties: false,
    });
});

test('loose tuples omit length constraints and allow arbitrary extra elements', (t) => {
    for (const type of ['[number, number]', '[number, string]']) {
        t.deepEqual(
            toJSONSchema(parse(type), { loose: true }),
            schema({
                type: 'array',
                prefixItems: [{ type: 'number' }, { type: type === '[number, number]' ? 'number' : 'string' }],
                items: true,
            }),
        );
    }
    t.deepEqual(toJSONSchema(parse('[]'), { loose: true }), schema({ type: 'array', items: true }));
    t.deepEqual(
        toJSONSchema(parse('[number, ..string[], boolean]'), { loose: true }),
        schema({
            type: 'array',
            prefixItems: [{ type: 'number' }],
            items: { anyOf: [{ type: 'string' }, { type: 'boolean' }] },
        }),
    );
});
