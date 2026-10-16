import test from 'ava';
import { REG_NUMBER } from '@mirascript/constants';
import { parse, toJSONSchema } from '@mirascript/typed';

const schema = (s: object) => ({ $schema: 'https://json-schema.org/draft/2020-12/schema', ...s });

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
