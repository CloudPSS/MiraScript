import test from 'ava';
import { REG_NUMBER } from '@mirascript/constants';
import { parse, toJSONSchema } from '@mirascript/typed';

const schema = (s: object) => ({ $schema: 'https://json-schema.org/draft/2020-12/schema', ...s });

test('function type JSON schema', (t) => {
    t.deepEqual(toJSONSchema(parse('fn(arg: number, ..rest: string) -> boolean')), schema({ not: true }));
    t.deepEqual(toJSONSchema(parse('fn() -> number')), schema({ not: true }));
    t.deepEqual(toJSONSchema(parse('fn(callback: fn(result: string) -> any)')), schema({ not: true }));
});

test('template type JSON schema', (t) => {
    for (const name of ['name', 'array', 'record', 'extern', 'any', 'unknown', 'name[]']) {
        t.deepEqual(
            toJSONSchema(parse(`"hello $(${name})"`)),
            schema({
                type: 'string',
                pattern: '^hello (.*?)$',
            }),
        );
    }
    t.deepEqual(
        toJSONSchema(parse('`value: $(string | number)`')),
        schema({
            type: 'string',
            pattern: `^value: (.*?)$`,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('`value: $(string & number)`')),
        schema({
            type: 'string',
            pattern: `^value: (.*?)$`,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('`value: $(boolean | number)`')),
        schema({
            type: 'string',
            pattern: `^value: (true|false|${REG_NUMBER.source})$`,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('`flag: $(true | false | nil)`')),
        schema({
            type: 'string',
            pattern: '^flag: (true|false)?$',
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('`prefix$(nil)suffix`')),
        schema({
            type: 'string',
            pattern: '^prefix()suffix$',
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('`prefix$(number | nil)suffix`')),
        schema({
            type: 'string',
            pattern: `^prefix(${REG_NUMBER.source})?suffix$`,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('`prefix$("x" | "y" | nil)suffix`')),
        schema({
            type: 'string',
            pattern: `^prefix(x|y)?suffix$`,
        }),
    );
    t.deepEqual(
        toJSONSchema(parse('`prefix$(boolean | "")suffix`')),
        schema({
            type: 'string',
            pattern: `^prefix(true|false)?suffix$`,
        }),
    );
});
