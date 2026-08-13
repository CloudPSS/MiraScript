import test from 'ava';
import { parse, toJSONSchema } from '@mirascript/typed';

const schema = (s: object) => ({ $schema: 'https://json-schema.org/draft/2020-12/schema', ...s });

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
