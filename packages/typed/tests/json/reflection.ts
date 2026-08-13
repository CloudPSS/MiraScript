import test from 'ava';
import { parse, toJSONSchema } from '@mirascript/typed';

const schema = (s: object) => ({ $schema: 'https://json-schema.org/draft/2020-12/schema', ...s });

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
