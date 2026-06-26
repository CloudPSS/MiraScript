import test from 'ava';
import { REG_NUMBER } from '@mirascript/constants';
import { parse } from '../src/parser.ts';
import { toJSONSchema } from '../src/json.ts';

test('primitive JSON schemas', (t) => {
    t.deepEqual(toJSONSchema(parse('string')), { type: 'string' });
    t.deepEqual(toJSONSchema(parse('number')), { type: 'number' });
    t.deepEqual(toJSONSchema(parse('boolean')), { type: 'boolean' });
    t.deepEqual(toJSONSchema(parse('nil')), { type: 'null' });
    t.deepEqual(toJSONSchema(parse('array')), { type: 'array' });
    t.deepEqual(toJSONSchema(parse('record')), { type: 'object' });
    t.deepEqual(toJSONSchema(parse('any')), {});
    t.deepEqual(toJSONSchema(parse('unknown')), {});
    t.deepEqual(toJSONSchema(parse('never')), {});
    t.deepEqual(toJSONSchema(parse('extern')), {});
});

test('named type JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('MyType')), {});
});

test('string literal JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('"hello"')), {
        type: 'string',
        const: 'hello',
    });
});

test('boolean literal JSON schemas', (t) => {
    t.deepEqual(toJSONSchema(parse('true')), {
        type: 'boolean',
        const: true,
    });
    t.deepEqual(toJSONSchema(parse('false')), {
        type: 'boolean',
        const: false,
    });
});

test('literal union JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('"on" | "off"')), {
        type: 'string',
        enum: ['on', 'off'],
    });
});

test('mixed literal union JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('"on" | true')), {
        enum: ['on', true],
    });
});

test('mixed union JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('| "private" | "public" | (search: string)')), {
        anyOf: [
            {
                type: 'object',
                properties: { search: { type: 'string' } },
                required: ['search'],
                additionalProperties: false,
            },
            { type: 'string', enum: ['private', 'public'] },
        ],
    });
});

test('array JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('number[]')), {
        type: 'array',
        items: { type: 'number' },
    });
});

test('array generic JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('array<number>')), {
        type: 'array',
        items: { type: 'number' },
    });
});

test('record generic JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('record<number>')), {
        type: 'object',
        additionalProperties: { type: 'number' },
    });
});

test('record key-value JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('record<string, number>')), {
        type: 'object',
        additionalProperties: { type: 'number' },
    });
    t.deepEqual(toJSONSchema(parse('record<number, boolean>')), {
        type: 'object',
        patternProperties: { [`^${REG_NUMBER.source}$`]: { type: 'boolean' } },
        additionalProperties: false,
    });
    t.deepEqual(toJSONSchema(parse('record<"id" | "name", boolean>')), {
        type: 'object',
        properties: { id: { type: 'boolean' }, name: { type: 'boolean' } },
        additionalProperties: false,
    });
    t.deepEqual(toJSONSchema(parse('record<"id" | "name" | boolean, boolean>')), {
        type: 'object',
        patternProperties: { '^id|name|true|false$': { type: 'boolean' } },
        additionalProperties: false,
    });
    t.deepEqual(toJSONSchema(parse('record<"id", boolean>')), {
        type: 'object',
        properties: { id: { type: 'boolean' } },
        additionalProperties: false,
    });
});

test('loose mode JSON schema allows arbitrary additional properties', (t) => {
    t.deepEqual(toJSONSchema(parse('(a: number)'), { loose: true }), {
        type: 'object',
        properties: { a: { type: 'number' } },
        additionalProperties: true,
    });
    t.deepEqual(toJSONSchema(parse('record<number>'), { loose: true }), {
        type: 'object',
        additionalProperties: { type: 'number' },
    });
    t.deepEqual(toJSONSchema(parse('record<number, boolean>'), { loose: true }), {
        type: 'object',
        patternProperties: { [`^${REG_NUMBER.source}$`]: { type: 'boolean' } },
        additionalProperties: true,
    });
    t.deepEqual(toJSONSchema(parse('record<"id" | "name", boolean>'), { loose: true }), {
        type: 'object',
        properties: { id: { type: 'boolean' }, name: { type: 'boolean' } },
        additionalProperties: true,
    });
});

test('union JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('string | number')), {
        anyOf: [{ type: 'string' }, { type: 'number' }],
    });
});

test('record JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('(a: number, b?: string)')), {
        type: 'object',
        properties: {
            a: { type: 'number' },
            b: { type: 'string' },
        },
        required: ['a'],
        additionalProperties: false,
    });
});

test('record with anonymous field JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('(number, string)')), {
        type: 'object',
        properties: {
            '0': { type: 'number' },
            '1': { type: 'string' },
        },
        required: ['0', '1'],
        additionalProperties: false,
    });
});

test('record with mixed anonymous and named fields JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('(number, b: string, c?: boolean, nil)')), {
        type: 'object',
        properties: {
            '0': { type: 'number' },
            b: { type: 'string' },
            c: { type: 'boolean' },
            '3': { type: 'null' },
        },
        required: ['0', 'b', '3'],
        additionalProperties: false,
    });
});

test('empty record JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('()')), {
        type: 'object',
        properties: {},
        additionalProperties: false,
    });
});

test('record with string field name JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('("field-name": number)')), {
        type: 'object',
        properties: {
            'field-name': { type: 'number' },
        },
        required: ['field-name'],
        additionalProperties: false,
    });
});

test('function type JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('fn(arg: number, ..rest: string) -> boolean')), {});
    t.deepEqual(toJSONSchema(parse('fn() -> number')), {});
    t.deepEqual(toJSONSchema(parse('fn(callback: fn(result: string) -> any)')), {});
});

test('template type JSON schema', (t) => {
    for (const name of ['name', 'array', 'record', 'extern', 'any', 'unknown', 'name[]']) {
        t.deepEqual(toJSONSchema(parse(`"hello $(${name})"`)), {
            type: 'string',
            pattern: '^hello (.*?)$',
        });
    }
    t.deepEqual(toJSONSchema(parse('`value: $(string | number)`')), {
        type: 'string',
        pattern: `^value: (.*?)$`,
    });
    t.deepEqual(toJSONSchema(parse('`value: $(boolean | number)`')), {
        type: 'string',
        pattern: `^value: (true|false|${REG_NUMBER.source})$`,
    });
    t.deepEqual(toJSONSchema(parse('`flag: $(true | false | nil)`')), {
        type: 'string',
        pattern: '^flag: (true|false)?$',
    });
    t.deepEqual(toJSONSchema(parse('`prefix$(nil)suffix`')), {
        type: 'string',
        pattern: '^prefix()suffix$',
    });
    t.deepEqual(toJSONSchema(parse('`prefix$(number | nil)suffix`')), {
        type: 'string',
        pattern: `^prefix(${REG_NUMBER.source})?suffix$`,
    });
    t.deepEqual(toJSONSchema(parse('`prefix$("x" | "y" | nil)suffix`')), {
        type: 'string',
        pattern: `^prefix(x|y)?suffix$`,
    });
    t.deepEqual(toJSONSchema(parse('`prefix$(boolean | "")suffix`')), {
        type: 'string',
        pattern: `^prefix(true|false)?suffix$`,
    });
});
