import test from 'ava';
import { parse, toJSONSchema } from '@mirascript/typed';

const schema = (s: object) => ({ $schema: 'https://json-schema.org/draft/2020-12/schema', ...s });

test('tuple JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[number, string]')),
        schema({
            type: 'array',
            prefixItems: [{ type: 'number' }, { type: 'string' }],
            items: { not: {} },
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
            items: {},
        }),
    );
});

test('tuple with user-type rest element JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[number, ..MyType]')),
        schema({
            type: 'array',
            prefixItems: [{ type: 'number' }],
            items: {},
        }),
    );
});

test('tuple with bare rest in middle JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[number, ..string, boolean]')),
        schema({
            type: 'array',
            prefixItems: [{ type: 'number' }],
            items: {},
        }),
    );
});

test('single element tuple JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[number]')),
        schema({
            type: 'array',
            prefixItems: [{ type: 'number' }],
            items: { not: {} },
        }),
    );
});

test('empty tuple JSON schema', (t) => {
    t.deepEqual(
        toJSONSchema(parse('[]')),
        schema({
            type: 'array',
            items: { not: {} },
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
                    items: { not: {} },
                },
                { type: 'boolean' },
            ],
            items: { not: {} },
        }),
    );
});
