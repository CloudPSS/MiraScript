import test from 'ava';
import { REG_NUMBER } from '@mirascript/constants';
import { parse, toJSONSchema } from '@mirascript/typed';

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
