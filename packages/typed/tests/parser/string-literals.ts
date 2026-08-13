import test from 'ava';
import { parse, type FunctionType, type TemplateType } from '@mirascript/typed';

test('string literal type', (t) => {
    t.deepEqual(parse('"hello"'), { kind: 'literal', value: 'hello' });
});

test('string literal type with escaped characters', (t) => {
    t.deepEqual(parse(String.raw`'\''`), { kind: 'literal', value: "'" });
    t.deepEqual(parse('`\\n`'), { kind: 'literal', value: '\n' });
    t.deepEqual(parse(String.raw`'\"'`), { kind: 'literal', value: '"' });
    t.deepEqual(parse(String.raw`'\\'`), { kind: 'literal', value: '\\' });
    t.deepEqual(parse(String.raw`'\n'`), { kind: 'literal', value: '\n' });
    t.deepEqual(parse(String.raw`'\t'`), { kind: 'literal', value: '\t' });
    t.deepEqual(parse(String.raw`'\r'`), { kind: 'literal', value: '\r' });
    t.deepEqual(parse(String.raw`'\b'`), { kind: 'literal', value: '\b' });
    t.deepEqual(parse(String.raw`'\f'`), { kind: 'literal', value: '\f' });
    t.deepEqual(parse(String.raw`'\v'`), { kind: 'literal', value: '\v' });
    t.deepEqual(parse(String.raw`'\0'`), { kind: 'literal', value: '\0' });
    t.deepEqual(parse(String.raw`'\x41'`), { kind: 'literal', value: 'A' });
    t.deepEqual(parse(String.raw`'\u{41}'`), { kind: 'literal', value: 'A' });
    t.deepEqual(parse(String.raw`'\\u{41}'`), { kind: 'literal', value: String.raw`\u{41}` });
    t.deepEqual(parse(String.raw`"hello\nworld"`), { kind: 'literal', value: 'hello\nworld' });
    t.deepEqual(parse(String.raw`"hello\tworld"`), { kind: 'literal', value: 'hello\tworld' });
    t.deepEqual(parse(String.raw`"hello\\"`), { kind: 'literal', value: 'hello\\' });
    t.deepEqual(parse(String.raw`"hello\""`), { kind: 'literal', value: 'hello"' });
    t.deepEqual(parse(String.raw`"hello\x41world"`), { kind: 'literal', value: 'helloAworld' });
    t.deepEqual(parse(String.raw`"hello\u{41}world"`), { kind: 'literal', value: 'helloAworld' });
});

test('string literal type with invalid escape sequences', (t) => {
    t.throws(() => parse(String.raw`'\x4'`));
    t.throws(() => parse(String.raw`'\e'`));
    t.throws(() => parse(String.raw`'\u{4'`));
    t.throws(() => parse(String.raw`'\xff'`));
    t.throws(() => parse(String.raw`'\u{110000}'`));
    t.throws(() => parse(String.raw`'\u{D800}\u{DC00}'`));
    t.throws(() => parse(String.raw`'\x1'`));
    t.throws(() => parse(String.raw`'\x1g'`));
    t.throws(() => parse(String.raw`'\x'`));
});

test('string literal type with interpolation', (t) => {
    t.deepEqual(parse('`hello $(name)`'), {
        kind: 'template',
        parts: [{ kind: 'literal', value: 'hello ' }, 'name'],
    });
    t.deepEqual(parse('`hello $(name) world`'), {
        kind: 'template',
        parts: [{ kind: 'literal', value: 'hello ' }, 'name', { kind: 'literal', value: ' world' }],
    });
    t.deepEqual(parse('`hello $(name) world $( age )`'), {
        kind: 'template',
        parts: [{ kind: 'literal', value: 'hello ' }, 'name', { kind: 'literal', value: ' world ' }, 'age'],
    });
});

test('string literal type with bad interpolation', (t) => {
    t.throws(() => parse('`hello ${name}`'));
    t.throws(() => parse('`hello $name`'));
    t.throws(() => parse('`$`'));
    t.throws(() => parse('`$()`'));
    t.throws(() => parse('`$(x`'));
});

test('empty string literal type', (t) => {
    t.deepEqual(parse('""'), { kind: 'literal', value: '' });
});

test('single-quoted string literal type', (t) => {
    t.deepEqual(parse("'hello'"), { kind: 'literal', value: 'hello' });
});

test('backtick-quoted string literal type', (t) => {
    t.deepEqual(parse('`hello`'), { kind: 'literal', value: 'hello' });
});

test('string interpolation type with all quote styles', (t) => {
    t.deepEqual(parse('"hello $(name)"'), {
        kind: 'template',
        parts: [{ kind: 'literal', value: 'hello ' }, 'name'],
    });
    t.deepEqual(parse('"hello $(nil)"'), {
        kind: 'template',
        parts: [{ kind: 'literal', value: 'hello ' }, 'nil'],
    });
    t.deepEqual(parse("'hello $(name)'"), {
        kind: 'template',
        parts: [{ kind: 'literal', value: 'hello ' }, 'name'],
    });
    t.deepEqual(parse('`hello $(name)`'), {
        kind: 'template',
        parts: [{ kind: 'literal', value: 'hello ' }, 'name'],
    });
});

test('string interpolation type with escaped dollar', (t) => {
    t.deepEqual(parse(String.raw`"hello \$(name)"`), {
        kind: 'literal',
        value: 'hello $(name)',
    });
});

test('string interpolation type with complex type', (t) => {
    t.deepEqual(parse('`value: $(string | number)`'), {
        kind: 'template',
        parts: [
            { kind: 'literal', value: 'value: ' },
            { kind: 'union', types: ['string', 'number'] },
        ],
    });
});

test('string interpolation type with literal', (t) => {
    t.deepEqual(parse('`value: $(true)`'), {
        kind: 'template',
        parts: [
            { kind: 'literal', value: 'value: ' },
            { kind: 'literal', value: true },
        ],
    });
    t.deepEqual(parse('`value: $("x")$(`y`)`'), {
        kind: 'template',
        parts: [
            { kind: 'literal', value: 'value: ' },
            { kind: 'literal', value: 'x' },
            { kind: 'literal', value: 'y' },
        ],
    });
});

test('string interpolation type with generic function', (t) => {
    const result = parse('`callback: $(fn<T>(x: T) -> T)`') as TemplateType;
    t.is(result.kind, 'template');
    t.deepEqual(result.parts[0], { kind: 'literal', value: 'callback: ' });
    const fn = result.parts[1] as FunctionType;
    t.is(fn.kind, 'function');
    t.is(fn.typeParams!.length, 1);
    t.is(fn.params[0].type, fn.typeParams![0]);
    t.is(fn.returns, fn.typeParams![0]);
});
