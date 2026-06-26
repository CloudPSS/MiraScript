import test from 'ava';
import { parse, type FunctionType, type TemplateType } from '../src/parser.ts';

test('primitive types', (t) => {
    t.is(parse('nil'), 'nil');
    t.is(parse('string'), 'string');
    t.is(parse('number'), 'number');
    t.is(parse('boolean'), 'boolean');
    t.is(parse('record'), 'record');
    t.is(parse('array'), 'array');
});

test('named type', (t) => {
    t.is(parse('MyType'), 'MyType');
});

test('unicode named type', (t) => {
    t.is(parse('类型'), '类型');
    t.is(parse('My类型'), 'My类型');
});

test('special character named type', (t) => {
    t.is(parse('_MyType'), '_MyType');
    t.is(parse('$MyType'), '$MyType');
    t.is(parse('@MyType'), '@MyType');
    t.is(parse('$_MyType'), '$_MyType');
    t.throws(() => parse('1MyType'));
    t.throws(() => parse('$@MyType'));
});

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
    t.is(fn.params[0]!.type, fn.typeParams![0]!);
    t.is(fn.returns, fn.typeParams![0]);
});

test('boolean literal types', (t) => {
    t.deepEqual(parse('true'), { kind: 'literal', value: true });
    t.deepEqual(parse('false'), { kind: 'literal', value: false });
});

test('array type', (t) => {
    t.deepEqual(parse('number[]'), { kind: 'array', element: 'number' });
});

test('nested array type', (t) => {
    t.deepEqual(parse('number[][]'), {
        kind: 'array',
        element: { kind: 'array', element: 'number' },
    });
});

test('array generic type', (t) => {
    t.deepEqual(parse('array<number>'), { kind: 'array', element: 'number' });
    t.deepEqual(parse('array<any,>'), { kind: 'array', element: 'any' });
    t.throws(() => parse('array<number, string>'));
});

test('record generic type', (t) => {
    t.deepEqual(parse('record<number>'), {
        kind: 'record',
        fields: [],
        value: 'number',
    });
    t.throws(() => parse('record<number, string>'));
});

test('union type', (t) => {
    t.deepEqual(parse('string | number'), {
        kind: 'union',
        types: ['string', 'number'],
    });
    t.deepEqual(parse('true | false'), {
        kind: 'union',
        types: [
            { kind: 'literal', value: true },
            { kind: 'literal', value: false },
        ],
    });
});

test('union type with leading pipe', (t) => {
    t.deepEqual(parse('| string | number'), {
        kind: 'union',
        types: ['string', 'number'],
    });
});

test('record type', (t) => {
    t.deepEqual(parse('(a: number, b: string)'), {
        kind: 'record',
        fields: [
            { name: 'a', optional: false, type: 'number' },
            { name: 'b', optional: false, type: 'string' },
        ],
    });
});

test('record type with optional field', (t) => {
    t.deepEqual(parse('(a?: number)'), {
        kind: 'record',
        fields: [{ name: 'a', optional: true, type: 'number' }],
    });
});

test('record type with string field name', (t) => {
    t.deepEqual(parse('("field-name": number)'), {
        kind: 'record',
        fields: [{ name: 'field-name', optional: false, type: 'number' }],
    });
});

test('record type with trailing comma', (t) => {
    t.deepEqual(parse('(a: number,)'), {
        kind: 'record',
        fields: [{ name: 'a', optional: false, type: 'number' }],
    });
});

test('complex nested type', (t) => {
    t.deepEqual(
        parse(
            '((stroke?: string, "stroke-width"?: number, fill?: string, text?: string, "font-size"?: number, "font-family"?: string) | ("on"|"off",))[]',
        ),
        {
            kind: 'array',
            element: {
                kind: 'union',
                types: [
                    {
                        kind: 'record',
                        fields: [
                            { name: 'stroke', optional: true, type: 'string' },
                            { name: 'stroke-width', optional: true, type: 'number' },
                            { name: 'fill', optional: true, type: 'string' },
                            { name: 'text', optional: true, type: 'string' },
                            { name: 'font-size', optional: true, type: 'number' },
                            { name: 'font-family', optional: true, type: 'string' },
                        ],
                    },
                    {
                        kind: 'record',
                        fields: [
                            {
                                name: '0',
                                type: {
                                    kind: 'union',
                                    types: [
                                        {
                                            kind: 'literal',
                                            value: 'on',
                                        },
                                        {
                                            kind: 'literal',
                                            value: 'off',
                                        },
                                    ],
                                },
                            },
                        ],
                    },
                ],
            },
        },
    );
});

test('empty record', (t) => {
    t.deepEqual(parse('()'), { kind: 'record', fields: [] });
});

test('anonymous record fields', (t) => {
    t.deepEqual(parse('(number, string)'), {
        kind: 'record',
        fields: [
            { name: '0', type: 'number' },
            { name: '1', type: 'string' },
        ],
    });
});

test('single anonymous record field requires trailing comma', (t) => {
    t.deepEqual(parse('(number,)'), {
        kind: 'record',
        fields: [{ name: '0', type: 'number' }],
    });
    t.is(parse('(number)'), 'number');
});

test('record type with bad field name', (t) => {
    t.throws(() => parse('(1a: number)'));
    t.throws(() => parse('("field$(name)": number)'));
});

test('function type with return', (t) => {
    t.deepEqual(parse('fn(arg: number, ..rest: string) -> boolean'), {
        kind: 'function',
        params: [
            { name: 'arg', type: 'number' },
            { name: 'rest', type: 'string', spread: true },
        ],
        returns: 'boolean',
    });
});

test('function type without return', (t) => {
    t.deepEqual(parse('fn(arg: number)'), {
        kind: 'function',
        params: [{ name: 'arg', type: 'number' }],
    });
});

test('function type without parameters', (t) => {
    t.deepEqual(parse('fn() -> number'), {
        kind: 'function',
        params: [],
        returns: 'number',
    });
});

test('function type with callback parameter', (t) => {
    t.deepEqual(parse('fn(arg: number, callback: fn(result: string, error: any) -> any)'), {
        kind: 'function',
        params: [
            { name: 'arg', type: 'number' },
            {
                name: 'callback',
                type: {
                    kind: 'function',
                    params: [
                        { name: 'result', type: 'string' },
                        { name: 'error', type: 'any' },
                    ],
                    returns: 'any',
                },
            },
        ],
    });
});

test('function type with trailing comma', (t) => {
    t.deepEqual(parse('fn(a: number,) -> string'), {
        kind: 'function',
        params: [{ name: 'a', type: 'number' }],
        returns: 'string',
    });
    t.deepEqual(parse('fn(..,)'), {
        kind: 'function',
        params: [{ name: '', type: { kind: 'array', element: 'any' }, spread: true }],
    });
    t.deepEqual(parse('fn(..rest,) -> string'), {
        kind: 'function',
        params: [{ name: 'rest', type: { kind: 'array', element: 'any' }, spread: true }],
        returns: 'string',
    });
});

test('function type with omitted param types', (t) => {
    t.deepEqual(parse('fn(a, b: number, c, ..d) -> string'), {
        kind: 'function',
        params: [
            { name: 'a', type: 'any' },
            { name: 'b', type: 'number' },
            { name: 'c', type: 'any' },
            { name: 'd', type: { kind: 'array', element: 'any' }, spread: true },
        ],
        returns: 'string',
    });
});

test('function type with omitted rest param name', (t) => {
    t.deepEqual(parse('fn(..) -> string'), {
        kind: 'function',
        params: [{ name: '', type: { kind: 'array', element: 'any' }, spread: true }],
        returns: 'string',
    });
});

test('generic function type', (t) => {
    const result = parse('fn<T, U>(arg: T) -> U') as FunctionType;
    t.is(result.kind, 'function');
    t.is(result.typeParams?.length, 2);
    t.is(typeof result.typeParams![0], 'symbol');
    t.is(typeof result.typeParams![1], 'symbol');
    t.is(result.typeParams![0]!.description, 'T');
    t.is(result.typeParams![1]!.description, 'U');
    t.is(result.params[0]!.type, result.typeParams![0]!);
    t.is(result.returns, result.typeParams![1]);
});

test('generic function type with single type parameter', (t) => {
    const result = parse('fn<T>(x: T) -> T') as FunctionType;
    t.is(result.kind, 'function');
    t.is(result.typeParams?.length, 1);
    t.is(typeof result.typeParams![0], 'symbol');
    t.is(result.typeParams![0]!.description, 'T');
    t.is(result.params[0]!.type, result.typeParams![0]!);
    t.is(result.returns, result.typeParams![0]);
});

test('generic function type without parameters and return', (t) => {
    const result = parse('fn<T,>()') as FunctionType;
    t.is(result.kind, 'function');
    t.is(result.typeParams?.length, 1);
    t.is(typeof result.typeParams![0], 'symbol');
    t.is(result.params.length, 0);
});

test('nested generic function type', (t) => {
    const result = parse('fn<T>(callback: fn<U>(x: U) -> T) -> T') as FunctionType;
    t.is(result.kind, 'function');
    const outerT = result.typeParams![0]!;
    const innerFn = result.params[0]!.type as FunctionType;
    const innerU = innerFn.typeParams![0]!;
    t.is(outerT.description, 'T');
    t.is(innerU.description, 'U');
    t.is(innerFn.params[0]!.type, innerU);
    t.is(innerFn.returns, outerT);
    t.is(result.returns, outerT);
});

test('nested generic function with same name uses different symbols', (t) => {
    const result = parse('fn<T>(arg: T, callback: fn<T>(data: T))') as FunctionType;
    const outerT = result.typeParams![0]!;
    const innerFn = result.params[1]!.type as FunctionType;
    const innerT = innerFn.typeParams![0]!;
    t.not(outerT, innerT);
    t.is(result.params[0]!.type, outerT);
    t.is(innerFn.params[0]!.type, innerT);
});

test('rest parameter must be the last parameter', (t) => {
    t.throws(() => parse('fn(..a, b) -> string'));
    t.throws(() => parse('fn(a, ..b, c) -> string'));
    t.throws(() => parse('fn(..a, ..b) -> string'));
});

test('invalid syntax throws', (t) => {
    t.throws(() => parse(''));
    t.throws(() => parse('number['));
    t.throws(() => parse('fn'));
    t.throws(() => parse('fn | "12"'));
    t.throws(() => parse('fn(,)'));
    t.throws(() => parse('any<x>'));
});

test('priority of types', (t) => {
    t.deepEqual(parse('string | number[]'), {
        kind: 'union',
        types: ['string', { kind: 'array', element: 'number' }],
    });
    t.deepEqual(parse('(string | number)[]'), {
        kind: 'array',
        element: {
            kind: 'union',
            types: ['string', 'number'],
        },
    });
    t.deepEqual(parse('fn() -> boolean | string[]'), {
        kind: 'function',
        params: [],
        returns: {
            kind: 'union',
            types: ['boolean', { kind: 'array', element: 'string' }],
        },
    });
    t.deepEqual(parse('fn() -> (boolean | string)[]'), {
        kind: 'function',
        params: [],
        returns: {
            kind: 'array',
            element: {
                kind: 'union',
                types: ['boolean', 'string'],
            },
        },
    });
    t.deepEqual(parse('(fn () -> boolean) | string[]'), {
        kind: 'union',
        types: [
            {
                kind: 'function',
                params: [],
                returns: 'boolean',
            },
            { kind: 'array', element: 'string' },
        ],
    });
    t.deepEqual(parse('fn() | string[]'), {
        kind: 'union',
        types: [
            {
                kind: 'function',
                params: [],
            },
            { kind: 'array', element: 'string' },
        ],
    });
    t.deepEqual(parse('fn(a: number) -> fn(b: boolean) -> string'), {
        kind: 'function',
        params: [{ name: 'a', type: 'number' }],
        returns: {
            kind: 'function',
            params: [{ name: 'b', type: 'boolean' }],
            returns: 'string',
        },
    });
});
