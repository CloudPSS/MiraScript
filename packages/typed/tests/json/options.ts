import test from 'ava';
import { parse, toJSONSchema } from '@mirascript/typed';

test('$schema is included by default and when explicitly enabled', (t) => {
    const expected = { type: 'number', $schema: 'https://json-schema.org/draft/2020-12/schema' };
    t.deepEqual(toJSONSchema(parse('number')), expected);
    t.deepEqual(toJSONSchema(parse('number'), {}), expected);
    t.deepEqual(toJSONSchema(parse('number'), { $schema: true }), expected);
});

test('$schema can be omitted without changing schema constraints', (t) => {
    t.deepEqual(toJSONSchema(parse('number'), { $schema: false }), { type: 'number' });
    t.deepEqual(toJSONSchema(parse('any'), { $schema: false }), {});
    t.deepEqual(toJSONSchema(parse('never'), { $schema: false }), { not: {} });
    t.deepEqual(toJSONSchema(parse('(a: number[])'), { $schema: false, loose: true }), {
        type: 'object',
        properties: { a: { type: 'array', items: { type: 'number' } } },
        additionalProperties: true,
    });
});
