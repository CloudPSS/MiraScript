import test from 'ava';
import { parse, toJSONSchema } from '@mirascript/typed';

const schema = (s: object) => ({ $schema: 'https://json-schema.org/draft/2020-12/schema', ...s });

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
            additionalProperties: { not: {} },
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
            additionalProperties: { not: {} },
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
                    additionalProperties: { not: {} },
                },
                {
                    type: 'object',
                    properties: { t: { type: 'string' }, y: { type: 'number' } },
                    required: ['t', 'y'],
                    additionalProperties: { not: {} },
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
                    additionalProperties: { not: {} },
                },
                {
                    type: 'object',
                    properties: { t: { type: 'string' }, y: { type: 'number' }, z: { type: 'boolean' } },
                    required: ['t', 'y', 'z'],
                    additionalProperties: { not: {} },
                },
                {
                    type: 'object',
                    properties: { t: { type: 'string' }, y: { type: 'number' }, w: { type: 'null' } },
                    required: ['t', 'y', 'w'],
                    additionalProperties: { not: {} },
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
                    additionalProperties: { not: {} },
                },
                {
                    type: 'object',
                    properties: {
                        a: { type: 'number' },
                        y: { type: 'string' },
                    },
                    required: ['a', 'y'],
                    additionalProperties: { not: {} },
                },
                {
                    type: 'object',
                    properties: {
                        b: { type: 'number' },
                        x: { type: 'string' },
                    },
                    required: ['b', 'x'],
                    additionalProperties: { not: {} },
                },
                {
                    type: 'object',
                    properties: {
                        b: { type: 'number' },
                        y: { type: 'string' },
                    },
                    required: ['b', 'y'],
                    additionalProperties: { not: {} },
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
            additionalProperties: { not: {} },
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
            additionalProperties: { not: {} },
        }),
    );
});
