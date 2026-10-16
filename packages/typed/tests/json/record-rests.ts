import test from 'ava';
import { REG_NUMBER } from '@mirascript/constants';
import { parse, toJSONSchema } from '@mirascript/typed';

const schema = (s: object) => ({ $schema: 'https://json-schema.org/draft/2020-12/schema', ...s });

test('record with rest field JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('(a: number, ..record<string, string>)')),
        schema({
            type: 'object',
            properties: {
                a: { type: 'number' },
            },
            required: ['a'],
            additionalProperties: { type: 'string' },
        }),
    );
    // Record literals are inlined
    t.deepEqual(
        toJSONSchema(parse('(a: number, ..(b: string, c?: boolean))')),
        schema({
            type: 'object',
            properties: {
                a: { type: 'number' },
                b: { type: 'string' },
                c: { type: 'boolean' },
            },
            required: ['a', 'b'],
            additionalProperties: false,
        }),
    );
    // Explicit fields take precedence over the rest record
    t.deepEqual(
        toJSONSchema(parse('(a: number, ..(a?: string, b: boolean))')),
        schema({
            type: 'object',
            properties: {
                a: { type: 'number' },
                b: { type: 'boolean' },
            },
            required: ['a', 'b'],
            additionalProperties: false,
        }),
    );
});

test('record with generic rest field JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('(a: number, ..record)')),
        schema({
            type: 'object',
            properties: {
                a: { type: 'number' },
            },
            required: ['a'],
            additionalProperties: true,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('(a: number, ..record<number, boolean>)')),
        schema({
            type: 'object',
            properties: {
                a: { type: 'number' },
            },
            required: ['a'],
            patternProperties: { [`^${REG_NUMBER.source}$`]: { type: 'boolean' } },
            additionalProperties: false,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('(a: number, ..record<"id", boolean>)')),
        schema({
            type: 'object',
            properties: {
                a: { type: 'number' },
                id: { type: 'boolean' },
            },
            required: ['a'],
            additionalProperties: false,
        }),
    );
});

test('record with non-record rest field JSON schema', (t) => {
    // Non-record rest types are treated as any, consistent with tuple spreads
    t.deepEqual(
        toJSONSchema(parse('(a: number, ..string)')),
        schema({
            type: 'object',
            properties: {
                a: { type: 'number' },
            },
            required: ['a'],
            additionalProperties: true,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('(a: number, ..MyType)')),
        schema({
            type: 'object',
            properties: {
                a: { type: 'number' },
            },
            required: ['a'],
            additionalProperties: true,
        }),
    );
});

test('record with rest field in loose mode JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('(a: number, ..(b: string))'), { loose: true }),
        schema({
            type: 'object',
            properties: {
                a: { type: 'number' },
                b: { type: 'string' },
            },
            additionalProperties: true,
        }),
    );
});

test('record rest union retains each record value constraint', (t) => {
    t.deepEqual(
        toJSONSchema(parse('(a: number, ..(record<string, string> | record<string, number>))')),
        schema({
            anyOf: [
                {
                    type: 'object',
                    properties: { a: { type: 'number' } },
                    required: ['a'],
                    additionalProperties: { type: 'string' },
                },
                {
                    type: 'object',
                    properties: { a: { type: 'number' } },
                    required: ['a'],
                    additionalProperties: { type: 'number' },
                },
            ],
        }),
    );
});

test('record rest intersection retains both value constraints', (t) => {
    t.deepEqual(
        toJSONSchema(parse('(a: number, ..record<string, string>) & (b: boolean, ..record<string, number>)')),
        schema({
            allOf: [
                {
                    type: 'object',
                    properties: { a: { type: 'number' }, b: { type: 'boolean' } },
                    required: ['a', 'b'],
                    additionalProperties: { type: 'string' },
                },
                {
                    type: 'object',
                    properties: { a: { type: 'number' }, b: { type: 'boolean' } },
                    required: ['a', 'b'],
                    additionalProperties: { type: 'number' },
                },
            ],
        }),
    );
});

test('record rest intersection keeps key restrictions from each member', (t) => {
    t.deepEqual(
        toJSONSchema(parse('(..(record<string, string> & record<number, number>))')),
        schema({
            allOf: [
                { type: 'object', properties: {}, additionalProperties: { type: 'string' } },
                {
                    type: 'object',
                    properties: {},
                    patternProperties: { [`^${REG_NUMBER.source}$`]: { type: 'number' } },
                    additionalProperties: false,
                },
            ],
        }),
    );
});

test('record rest pattern excludes explicitly named fields', (t) => {
    const result = toJSONSchema(parse('("1": number, ..record<number, string>)'));
    const pattern = Object.keys(result.patternProperties ?? {})[0];
    t.false(new RegExp(pattern).test('1'));
    t.true(new RegExp(pattern).test('2'));
    t.deepEqual(result.properties, { '1': { type: 'number' } });
});

test('record rest preserves property names inherited from Object', (t) => {
    const result = toJSONSchema(parse('(..record<"toString" | "constructor" | "__proto__", number>)'));
    t.deepEqual(result.properties, {
        toString: { type: 'number' },
        constructor: { type: 'number' },
        ['__proto__']: { type: 'number' },
    });
});
