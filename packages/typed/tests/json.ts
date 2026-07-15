import test from 'ava';
import { REG_NUMBER } from '@mirascript/constants';
import { parse } from '../dist/parser.js';
import { toJSONSchema } from '../dist/json.js';

const schema = (s: object) => ({ $schema: 'https://json-schema.org/draft/2020-12/schema', ...s });

test('primitive JSON schemas', (t) => {
    t.deepEqual(toJSONSchema(parse('string')), schema({ type: 'string' }));
    t.deepEqual(toJSONSchema(parse('number')), schema({ type: 'number' }));
    t.deepEqual(toJSONSchema(parse('boolean')), schema({ type: 'boolean' }));
    t.deepEqual(toJSONSchema(parse('nil')), schema({ type: 'null' }));
    t.deepEqual(toJSONSchema(parse('array')), schema({ type: 'array', items: true }));
    t.deepEqual(toJSONSchema(parse('record')), schema({ type: 'object' }));
    t.deepEqual(toJSONSchema(parse('any')), schema({}));
    t.deepEqual(toJSONSchema(parse('unknown')), schema({}));
    t.deepEqual(toJSONSchema(parse('never')), schema({ not: true }));
    t.deepEqual(toJSONSchema(parse('extern')), schema({}));
});

test('named type JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('MyType')), schema({}));
});

test('string literal JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('"hello"')),
        schema({
            const: 'hello',
        }),
    );
});

test('boolean literal JSON schemas', (t) => {
    t.deepEqual(toJSONSchema(parse('true')), schema({ const: true }));
    t.deepEqual(toJSONSchema(parse('false')), schema({ const: false }));
});

test('literal union JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('"on" | "off"')), schema({ enum: ['on', 'off'] }));
});

test('mixed literal union JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('"on" | true')), schema({ enum: ['on', true] }));
});

test('preserve literal union and primitive types in mixed union JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('string | "on" | "off"')),
        schema({
            anyOf: [{ type: 'string' }, { enum: ['on', 'off'] }],
        }),
    );
});

test('mixed union JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('| "private" | "public" | (search: string)')),
        schema({
            anyOf: [
                {
                    type: 'object',
                    properties: { search: { type: 'string' } },
                    required: ['search'],
                    additionalProperties: false,
                },
                { enum: ['private', 'public'] },
            ],
        }),
    );
});

test('array JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('number[]')),
        schema({
            type: 'array',
            items: { type: 'number' },
        }),
    );
});

test('array generic JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('array<number>')),
        schema({
            type: 'array',
            items: { type: 'number' },
        }),
    );
});

test('record generic JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('record<number>')),
        schema({
            type: 'object',
            additionalProperties: { type: 'number' },
        }),
    );
});

test('record key-value JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('record<string, number>')),
        schema({
            type: 'object',
            additionalProperties: { type: 'number' },
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('record<number, boolean>')),
        schema({
            type: 'object',
            patternProperties: { [`^${REG_NUMBER.source}$`]: { type: 'boolean' } },
            additionalProperties: false,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('record<"id" | "name", boolean>')),
        schema({
            type: 'object',
            properties: { id: { type: 'boolean' }, name: { type: 'boolean' } },
            additionalProperties: false,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('record<"id" | "name" | boolean, boolean>')),
        schema({
            type: 'object',
            patternProperties: { '^id|name|true|false$': { type: 'boolean' } },
            additionalProperties: false,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('record<"id", boolean>')),
        schema({
            type: 'object',
            properties: { id: { type: 'boolean' } },
            additionalProperties: false,
        }),
    );
});

test('loose mode JSON schema allows arbitrary additional properties', (t) => {
    t.deepEqual(
        toJSONSchema(parse('(a: number)'), { loose: true }),
        schema({
            type: 'object',
            properties: { a: { type: 'number' } },
            additionalProperties: true,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('record<number>'), { loose: true }),
        schema({
            type: 'object',
            additionalProperties: { type: 'number' },
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('record<number, boolean>'), { loose: true }),
        schema({
            type: 'object',
            patternProperties: { [`^${REG_NUMBER.source}$`]: { type: 'boolean' } },
            additionalProperties: true,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('record<"id" | "name", boolean>'), { loose: true }),
        schema({
            type: 'object',
            properties: { id: { type: 'boolean' }, name: { type: 'boolean' } },
            additionalProperties: true,
        }),
    );
});

test('union JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('string | number')),
        schema({
            anyOf: [{ type: 'string' }, { type: 'number' }],
        }),
    );
});

test('intersection JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('string & "x"')),
        schema({
            allOf: [{ type: 'string' }, { const: 'x' }],
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('string & ("x" & "$(string)")')),
        schema({
            allOf: [{ type: 'string' }, { const: 'x' }, { type: 'string', pattern: '^(.*?)$' }],
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('(a: number) & (b: string)')),
        schema({
            type: 'object',
            properties: {
                a: { type: 'number' },
                b: { type: 'string' },
            },
            required: ['a', 'b'],
            additionalProperties: false,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('(a: number) & (a?: string) & () & (a: false)')),
        schema({
            type: 'object',
            properties: {
                a: {
                    allOf: [{ type: 'number' }, { type: 'string' }, { const: false }],
                },
            },
            required: ['a'],
            additionalProperties: false,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('(t: string) & ((x: number) | (y: number))')),
        schema({
            anyOf: [
                {
                    type: 'object',
                    properties: { t: { type: 'string' }, x: { type: 'number' } },
                    required: ['t', 'x'],
                    additionalProperties: false,
                },
                {
                    type: 'object',
                    properties: { t: { type: 'string' }, y: { type: 'number' } },
                    required: ['t', 'y'],
                    additionalProperties: false,
                },
            ],
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('(t: string) & ((x: number) | ((y: number) & ((z: boolean) | (w: nil))))')),
        schema({
            anyOf: [
                {
                    type: 'object',
                    properties: { t: { type: 'string' }, x: { type: 'number' } },
                    required: ['t', 'x'],
                    additionalProperties: false,
                },
                {
                    type: 'object',
                    properties: { t: { type: 'string' }, y: { type: 'number' }, z: { type: 'boolean' } },
                    required: ['t', 'y', 'z'],
                    additionalProperties: false,
                },
                {
                    type: 'object',
                    properties: { t: { type: 'string' }, y: { type: 'number' }, w: { type: 'null' } },
                    required: ['t', 'y', 'w'],
                    additionalProperties: false,
                },
            ],
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('((a: number) | (b: number)) & ((x: string) | (y: string))')),
        schema({
            anyOf: [
                {
                    type: 'object',
                    properties: {
                        a: { type: 'number' },
                        x: { type: 'string' },
                    },
                    required: ['a', 'x'],
                    additionalProperties: false,
                },
                {
                    type: 'object',
                    properties: {
                        a: { type: 'number' },
                        y: { type: 'string' },
                    },
                    required: ['a', 'y'],
                    additionalProperties: false,
                },
                {
                    type: 'object',
                    properties: {
                        b: { type: 'number' },
                        x: { type: 'string' },
                    },
                    required: ['b', 'x'],
                    additionalProperties: false,
                },
                {
                    type: 'object',
                    properties: {
                        b: { type: 'number' },
                        y: { type: 'string' },
                    },
                    required: ['b', 'y'],
                    additionalProperties: false,
                },
            ],
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('(number,) & (string,)')),
        schema({
            type: 'object',
            properties: {
                '0': {
                    allOf: [{ type: 'number' }, { type: 'string' }],
                },
            },
            required: ['0'],
            additionalProperties: false,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('(a?: number) & (a?: string)')),
        schema({
            type: 'object',
            properties: {
                a: {
                    allOf: [{ type: 'number' }, { type: 'string' }],
                },
            },
            additionalProperties: false,
        }),
    );
});

test('edge-case JSON schema branches', (t) => {
    // Generic symbols should map to unconstrained schema.
    t.deepEqual(toJSONSchema(Symbol('T')), schema({}));
    t.deepEqual(
        toJSONSchema({ kind: 'template', parts: [Symbol('T')] }),
        schema({
            type: 'string',
            pattern: '^(.*?)$',
        }),
    );

    // Internal AST edge case: single-member intersection should unwrap.
    t.deepEqual(toJSONSchema({ kind: 'intersection', types: ['string'] }), schema({ type: 'string' }));

    // Internal AST edge case: distributing over a one-member union keeps one branch.
    t.deepEqual(
        toJSONSchema({
            kind: 'intersection',
            types: [{ kind: 'union', types: ['string'] }, 'number'],
        }),
        schema({ allOf: [{ type: 'string' }, { type: 'number' }] }),
    );

    // Internal AST edge case: distribution produces a single flattened member.
    t.deepEqual(
        toJSONSchema({
            kind: 'intersection',
            types: [{ kind: 'union', types: [{ kind: 'intersection', types: ['string'] }] }],
        }),
        schema({ type: 'string' }),
    );
});

test('record JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('(a: number, b?: string)')),
        schema({
            type: 'object',
            properties: {
                a: { type: 'number' },
                b: { type: 'string' },
            },
            required: ['a'],
            additionalProperties: false,
        }),
    );
});

test('record with anonymous field JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('(number, string)')),
        schema({
            type: 'object',
            properties: {
                '0': { type: 'number' },
                '1': { type: 'string' },
            },
            required: ['0', '1'],
            additionalProperties: false,
        }),
    );
});

test('record with mixed anonymous and named fields JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('(number, b: string, c?: boolean, nil)')),
        schema({
            type: 'object',
            properties: {
                '0': { type: 'number' },
                b: { type: 'string' },
                c: { type: 'boolean' },
                '3': { type: 'null' },
            },
            required: ['0', 'b', '3'],
            additionalProperties: false,
        }),
    );
});

test('empty record JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('()')),
        schema({
            type: 'object',
            properties: {},
            additionalProperties: false,
        }),
    );
});

test('record with string field name JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('("field-name": number)')),
        schema({
            type: 'object',
            properties: {
                'field-name': { type: 'number' },
            },
            required: ['field-name'],
            additionalProperties: false,
        }),
    );
});

test('function type JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('fn(arg: number, ..rest: string) -> boolean')), schema({ not: true }));
    t.deepEqual(toJSONSchema(parse('fn() -> number')), schema({ not: true }));
    t.deepEqual(toJSONSchema(parse('fn(callback: fn(result: string) -> any)')), schema({ not: true }));
});

test('template type JSON schema', (t) => {
    for (const name of ['name', 'array', 'record', 'extern', 'any', 'unknown', 'name[]']) {
        t.deepEqual(
            toJSONSchema(parse(`"hello $(${name})"`)),
            schema({
                type: 'string',
                pattern: '^hello (.*?)$',
            }),
        );
    }
    t.deepEqual(
        toJSONSchema(parse('`value: $(string | number)`')),
        schema({
            type: 'string',
            pattern: `^value: (.*?)$`,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('`value: $(string & number)`')),
        schema({
            type: 'string',
            pattern: `^value: (.*?)$`,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('`value: $(boolean | number)`')),
        schema({
            type: 'string',
            pattern: `^value: (true|false|${REG_NUMBER.source})$`,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('`flag: $(true | false | nil)`')),
        schema({
            type: 'string',
            pattern: '^flag: (true|false)?$',
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('`prefix$(nil)suffix`')),
        schema({
            type: 'string',
            pattern: '^prefix()suffix$',
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('`prefix$(number | nil)suffix`')),
        schema({
            type: 'string',
            pattern: `^prefix(${REG_NUMBER.source})?suffix$`,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('`prefix$("x" | "y" | nil)suffix`')),
        schema({
            type: 'string',
            pattern: `^prefix(x|y)?suffix$`,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('`prefix$(boolean | "")suffix`')),
        schema({
            type: 'string',
            pattern: `^prefix(true|false)?suffix$`,
        }),
    );
});

test('tuple JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[number, string]')),
        schema({
            type: 'array',
            prefixItems: [{ type: 'number' }, { type: 'string' }],
            items: false,
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
        }),
    );
});

test('tuple with rest element at start JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[..number[], string]')),
        schema({
            type: 'array',
            items: { anyOf: [{ type: 'number' }, { type: 'string' }] },
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
        }),
    );
});

test('single element tuple JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[number]')),
        schema({
            type: 'array',
            prefixItems: [{ type: 'number' }],
            items: false,
        }),
    );
});

test('empty tuple JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[]')),
        schema({
            type: 'array',
            items: false,
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
                },
                { type: 'boolean' },
            ],
            items: false,
        }),
    );
});

test('reflection type JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('type(MyVar)')), schema({}));
});

test('reflection type in union JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('type(A) | string')), schema({}));
});

test('never union JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema({
            kind: 'union',
            types: [],
        }),
        schema({ not: true }),
    );
    t.deepEqual(
        toJSONSchema({
            kind: 'union',
            types: ['never', 'never'],
        }),
        schema({ not: true }),
    );
    t.deepEqual(
        toJSONSchema({
            kind: 'union',
            types: ['string', 'never'],
        }),
        schema({ type: 'string' }),
    );
});
