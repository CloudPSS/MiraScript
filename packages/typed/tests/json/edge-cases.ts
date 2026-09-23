import test from 'ava';
import { REG_NUMBER } from '@mirascript/constants';
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
