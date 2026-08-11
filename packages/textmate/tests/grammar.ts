import test from 'ava';
import { mirascriptLanguage } from '../src/language.ts';
import { tokenize, expectScope } from './_engine.ts';

test('classifies declarations, parameters, properties, calls, and Unicode identifiers', (t) => {
    const tokens = tokenize('mod 数学 { fn 加法(mut 左, ..其余) { 对象.方法(左).属性 } }');
    expectScope(t, tokens, 'mod', 'keyword.control.module.mira');
    expectScope(t, tokens, '数学', 'entity.name.namespace.mira');
    expectScope(t, tokens, '加法', 'entity.name.function.mira');
    expectScope(t, tokens, '左', 'variable.emphasis.mira');
    expectScope(t, tokens, '其余', 'variable.other.constant.emphasis.mira');
    expectScope(t, tokens, '方法', 'entity.name.function.member.mira');
    expectScope(t, tokens, '属性', 'variable.other.property.mira');
});

test('classifies keyword and numeric families', (t) => {
    const tokens = tokenize('if true and value not in global { let @常量 = 0xCA_FE; 1.5e+2 }');
    expectScope(t, tokens, 'if', 'keyword.control.mira');
    expectScope(t, tokens, 'true', 'constant.language.mira');
    expectScope(t, tokens, 'and', 'keyword.operator.wordlike.mira');
    expectScope(t, tokens, 'not', 'keyword.operator.wordlike.mira');
    expectScope(t, tokens, 'in', 'keyword.operator.wordlike.mira');
    expectScope(t, tokens, 'global', 'variable.language.mira');
    expectScope(t, tokens, '@常量', 'variable.other.constant.mira');
    expectScope(t, tokens, '0xCA_FE', 'constant.numeric.hex.mira');
    expectScope(t, tokens, '1.5e+2', 'constant.numeric.float.mira');
});

test('does not classify control keywords followed by parentheses as functions', (t) => {
    const tokens = tokenize('if (condition) { case (value) { call(value) } }');
    expectScope(t, tokens, 'if', 'keyword.control.mira');
    expectScope(t, tokens, 'case', 'keyword.control.mira');
    expectScope(t, tokens, 'call', 'entity.name.function.mira');
    for (const keyword of ['if', 'case']) {
        const token = tokens.find((candidate) => candidate.text === keyword);
        t.false(token!.scopes.includes('entity.name.function.mira'));
    }
});

test('only classifies type as a keyword in its two contextual forms', (t) => {
    const tokens = tokenize('type(value); type Value; let type = 1; type + 1; object.type(); value::type();');
    expectScope(t, tokens, 'type', 'keyword.operator.expression.mira', 0);
    expectScope(t, tokens, 'type', 'keyword.operator.expression.mira', 1);
    expectScope(t, tokens, 'type', 'variable.other.mira', 2);
    expectScope(t, tokens, 'type', 'variable.other.mira', 3);
    expectScope(t, tokens, 'type', 'entity.name.function.member.mira', 4);
    expectScope(t, tokens, 'type', 'keyword.operator.expression.mira', 5);
});

test('handles nested interpolation and format strings', (t) => {
    const tokens = tokenize('"value ${ if ok { fn_call((1 + 2)) } } / $(value:>8[.]2f)"');
    expectScope(t, tokens, '${', 'punctuation.definition.template-expression.begin.mira');
    expectScope(t, tokens, 'fn_call', 'entity.name.function.mira');
    expectScope(t, tokens, '$(', 'punctuation.definition.template-expression.begin.mira');
    expectScope(t, tokens, ':', 'punctuation.separator.format.mira');
    expectScope(t, tokens, '>8', 'string.unquoted.format.mira');
});

test('matches the exact interpolation width in verbatim strings', (t) => {
    for (const width of [1, 2, 3, 16]) {
        const ats = '@'.repeat(width);
        const dollars = '$'.repeat(width);
        const shorter = '$'.repeat(Math.max(1, width - 1));
        const tokens = tokenize(`${ats}"literal ${shorter}name ${dollars}name"${ats}`);
        expectScope(t, tokens, dollars, 'punctuation.definition.template-expression.begin.mira');
        expectScope(t, tokens, 'name', 'variable.other.mira', width === 1 ? 1 : 0);
        if (width > 1) {
            const literal = tokens.find((token) => token.text.includes(`${shorter}name`));
            t.true(literal!.scopes.includes('string.quoted.double.verbatim.mira'));
            t.false(literal!.scopes.includes('meta.interpolation.simple.mira'));
        }
    }
});

test('separates documentation prefixes from bold and italic markup', (t) => {
    const tokens = tokenize(
        [
            '/**',
            ' * - *a*: 第一个操作数',
            ' * - **b**: 第二个操作数',
            ' * ```mirascript',
            ' * matrix.add([1, 2], [3, 4]) // [4, 6]',
            ' * ```',
            ' */',
        ].join('\n'),
    );
    expectScope(t, tokens, '*a*', 'markup.italic.documentation.mira');
    expectScope(t, tokens, '**b**', 'markup.bold.documentation.mira');
    for (const line of [1, 2, 3, 4, 5]) {
        const prefix = tokens.find((token) => token.line === line && token.text.trim() === '*');
        t.truthy(prefix, `Missing documentation prefix on line ${line}`);
        t.is(prefix!.scopes.at(-1), 'comment.block.documentation.mira');
    }
});

test('does not consume a documentation terminator as italic markup', (t) => {
    const tokens = tokenize('/**\n * *1*/\nlet x = 12;');
    expectScope(t, tokens, 'let', 'keyword.declaration.mira');
    expectScope(t, tokens, 'x', 'variable.other.mira');
    t.false(tokens.some((token) => token.text === '*1*' && token.scopes.includes('markup.italic.documentation.mira')));
});

test('ends a documentation fence before a prefixed comment terminator', (t) => {
    const tokens = tokenize('/**\n * ```mirascript\n * */\n * ```');
    const trailingLine = tokens.filter((token) => token.line === 3);
    t.true(trailingLine.length > 0);
    for (const token of trailingLine) {
        t.false(token.scopes.includes('comment.block.documentation.mira'), token.scopes.join(', '));
        t.false(token.scopes.includes('markup.fenced_code.block.mira'), token.scopes.join(', '));
    }
});

test('closes documentation before an embedded string can consume its terminator', (t) => {
    const tokens = tokenize("/**\n * ```mirascript\n * '*/\nlet recovered = 1;");
    expectScope(t, tokens, 'let', 'keyword.declaration.mira');
    expectScope(t, tokens, 'recovered', 'variable.other.mira');
    const recovered = tokens.find((token) => token.text === 'recovered');
    t.false(recovered!.scopes.includes('comment.block.documentation.mira'));
    t.false(recovered!.scopes.includes('markup.fenced_code.block.mira'));
});

test('preserves multiline embedded source state between safe documentation lines', (t) => {
    const tokens = tokenize("/**\n * ```mirascript\n * 'first\n * second'\n * ```\n */");
    expectScope(t, tokens, 'second', 'string.quoted.single.mira');
});

test('highlights documentation fences for the MiraScript name, aliases, and an omitted tag', (t) => {
    const tags = [undefined, mirascriptLanguage.name, ...(mirascriptLanguage.aliases ?? []), 'mIrAsCrIpT'];
    for (const tag of tags) {
        const tokens = tokenize(['/**', ` * \`\`\`${tag ?? ''}`, ' * matrix.identity(3)', ' * ```', ' */'].join('\n'));
        if (tag) expectScope(t, tokens, tag, 'fenced_code.block.language.mira');
        expectScope(t, tokens, 'identity', 'entity.name.function.member.mira');
    }
});

test('supports documentation fences with three or more backticks', (t) => {
    for (const [openingLength, closingLength] of [
        [3, 3],
        [4, 4],
        [8, 8],
        [12, 16],
    ] as const) {
        const opening = '`'.repeat(openingLength);
        const closing = '`'.repeat(closingLength);
        const tokens = tokenize(
            [
                '/**',
                ` * ${opening}mirascript`,
                ' * matrix.identity(3)',
                ` * ${closing}`,
                ' */',
                'let outside = 1;',
            ].join('\n'),
        );
        expectScope(t, tokens, 'identity', 'entity.name.function.member.mira');
        expectScope(t, tokens, 'outside', 'variable.other.mira');
    }
});

test('keeps unknown documentation fence tags as unparsed code blocks', (t) => {
    const tokens = tokenize(
        ['/**', ' * ````javascript', ' * let value = call(1);', ' * ````', ' */', 'let outside = 1;'].join('\n'),
    );
    expectScope(t, tokens, 'javascript', 'fenced_code.block.language.mira');
    const body = tokens.find((token) => token.line === 2 && token.text.includes('let value'));
    t.truthy(body);
    t.true(body!.scopes.includes('markup.fenced_code.block.mira'));
    t.true(body!.scopes.includes('markup.raw.block.mira'));
    t.false(body!.scopes.includes('keyword.declaration.mira'));
    t.false(body!.scopes.includes('entity.name.function.mira'));
    expectScope(t, tokens, 'outside', 'variable.other.mira');
});

test('highlights MiraScript fenced code inside documentation comments', (t) => {
    const tokens = tokenize(
        [
            '/**',
            ' * 创建一个单位矩阵',
            ' *',
            ' * - `..size`: 矩阵的维度',
            ' *',
            ' * ### 示例',
            ' * ```mirascript',
            ' * matrix.identity(3) // [[1, 0, 0], [0, 1, 0], [0, 0, 1]]',
            ' * ```',
            ' * @returns the matrix',
            ' */',
            'let outside = 2;',
        ].join('\n'),
    );
    expectScope(t, tokens, 'mirascript', 'fenced_code.block.language.mira');
    expectScope(t, tokens, 'identity', 'entity.name.function.member.mira');
    expectScope(t, tokens, '3', 'constant.numeric.float.mira');
    expectScope(t, tokens, '// [[1, 0, 0], [0, 1, 0], [0, 0, 1]]', 'comment.line.double-slash.mira');
    expectScope(t, tokens, '@returns', 'storage.type.class.documentation.mira');
    expectScope(t, tokens, 'let', 'keyword.declaration.mira');
    expectScope(t, tokens, 'outside', 'variable.other.mira');
});

test('ends an unterminated documentation fence at the comment boundary', (t) => {
    const tokens = tokenize('/**\n * ```mirascript\n * fn call() { nil }\n */\nlet recovered = 1;');
    expectScope(t, tokens, 'call', 'entity.name.function.mira');
    expectScope(t, tokens, 'let', 'keyword.declaration.mira');
    expectScope(t, tokens, 'recovered', 'variable.other.mira');
    const recovered = tokens.find((token) => token.text === 'recovered');
    t.false(recovered!.scopes.includes('comment.block.documentation.mira'));
});

test('highlights documentation comments inside documentation mode', (t) => {
    const tokens = tokenize(
        [
            '/**',
            ' * **bold** and *italic*',
            ' * ```mirascript',
            ' * matrix.identity(3)',
            ' * ```',
            ' */',
            'fn recovered(value: number) -> number',
        ].join('\n'),
        'mirascript-doc',
    );
    expectScope(t, tokens, '**bold**', 'markup.bold.documentation.mira');
    expectScope(t, tokens, '*italic*', 'markup.italic.documentation.mira');
    expectScope(t, tokens, 'identity', 'entity.name.function.member.mira');
    expectScope(t, tokens, 'recovered', 'entity.name.function.mira');
    expectScope(t, tokens, 'number', 'support.type.builtin.mira');
});

test('highlights template text and embedded MiraScript', (t) => {
    const tokens = tokenize('Hello $name: ${ fn_call(1) }', 'mirascript-template');
    expectScope(t, tokens, 'Hello ', 'string.unquoted.template.mira');
    expectScope(t, tokens, '$', 'punctuation.definition.template-expression.begin.mira');
    expectScope(t, tokens, 'name', 'variable.other.mira');
    expectScope(t, tokens, 'fn_call', 'entity.name.function.mira');
});

test('highlights generated documentation syntax', (t) => {
    const tokens = tokenize(
        [
            '\0(parameter) mut value',
            '\0(parameter) plain',
            '(field) description',
            'let immutable',
            'const @constant',
            'let mut mutable',
            'item: /* <extern function> */ fn(arg: number) -> string',
            'fn transform<T>(value: record<string, T>, callback: fn(result: T) -> boolean) -> T[] | nil',
            'reflected: type(MyValue)',
            '(field?: number, nested: (name: string))',
        ].join('\n'),
        'mirascript-doc',
    );
    expectScope(t, tokens, '(parameter)', 'entity.name.label.mira');
    expectScope(t, tokens, '(field)', 'entity.name.label.mira');
    expectScope(t, tokens, 'value', 'variable.emphasis.mira');
    expectScope(t, tokens, 'plain', 'variable.other.constant.emphasis.mira');
    expectScope(t, tokens, 'immutable', 'variable.other.constant.mira');
    expectScope(t, tokens, '@constant', 'variable.other.constant.mira');
    expectScope(t, tokens, 'mutable', 'variable.other.readwrite.mira');
    expectScope(t, tokens, 'extern', 'storage.modifier.extern.mira');
    expectScope(t, tokens, 'function', 'keyword.declaration.function.mira');
    expectScope(t, tokens, 'transform', 'entity.name.function.mira');
    expectScope(t, tokens, 'record', 'support.type.builtin.mira');
    expectScope(t, tokens, 'callback', 'entity.name.function.emphasis.mira');
    expectScope(t, tokens, 'boolean', 'support.type.builtin.mira');
    expectScope(t, tokens, 'MyValue', 'variable.other.mira');
    expectScope(t, tokens, 'field', 'variable.emphasis.mira');
    expectScope(t, tokens, 'nested', 'variable.emphasis.mira');
    expectScope(t, tokens, 'name', 'variable.other.property.mira');
    t.false(
        tokens.filter((token) => token.line === 9).some((token) => token.scopes.includes('entity.name.label.mira')),
    );
    expectScope(t, tokens, '->', 'keyword.operator.type.mira');
});

test('distinguishes doc declarations, globals, and nested function types', (t) => {
    const tokens = tokenize(
        [
            'mod matrix {',
            '  pub fn determinant(data: array | record) -> number',
            '}',
            '\0(global) mod matrix',
            '\0PI',
            '\0(global) PI',
            '\0(global) fn map(',
            '  data: array | record,',
            '  f: fn(value: any, key: number | string, input: type(data)) -> any,',
            ') -> type(data)',
        ].join('\n'),
        'mirascript-doc',
    );
    expectScope(t, tokens, 'mod', 'keyword.control.module.mira', 0);
    expectScope(t, tokens, 'matrix', 'entity.name.namespace.mira', 0);
    expectScope(t, tokens, 'pub', 'keyword.control.module.mira');
    expectScope(t, tokens, 'PI', 'variable.other.constant.mira', 0);
    expectScope(t, tokens, 'PI', 'variable.other.constant.mira', 1);
    expectScope(t, tokens, '(global)', 'entity.name.label.mira', 0);
    expectScope(t, tokens, 'fn', 'keyword.declaration.function.mira', 0);
    expectScope(t, tokens, 'fn', 'support.type.function.mira', 2);
    expectScope(t, tokens, 'data', 'variable.emphasis.mira', 1);
    expectScope(t, tokens, 'f', 'entity.name.function.emphasis.mira');
    expectScope(t, tokens, 'value', 'variable.emphasis.mira');
    expectScope(t, tokens, 'type', 'keyword.operator.expression.mira');
    expectScope(t, tokens, 'data', 'variable.other.mira', 2);
});

test('keeps nested types in unlabelled and module function signatures', (t) => {
    const tokens = tokenize(
        [
            'fn map(',
            '  data: array | record,',
            '  f: fn(value: any, key: number | string, input: type(data)) -> any,',
            ') -> type(data)',
            'mod matrix {',
            '  pub fn entrywise(',
            '    a: any | any[] | any[][],',
            '    b: any | any[] | any[][],',
            '    f: fn(a: any, b: any) -> any,',
            '  ) -> any | any[] | any[][];',
            '}',
        ].join('\n'),
        'mirascript-doc',
    );
    const declarationFns = tokens.filter((token) => token.text === 'fn' && (token.line === 0 || token.line === 5));
    const typeFns = tokens.filter((token) => token.text === 'fn' && (token.line === 2 || token.line === 8));
    t.is(declarationFns.length, 2);
    t.is(typeFns.length, 2);
    for (const token of declarationFns) {
        t.true(token.scopes.includes('keyword.declaration.function.mira'), token.scopes.join(', '));
        t.false(token.scopes.includes('support.type.function.mira'), token.scopes.join(', '));
    }
    for (const token of typeFns) {
        t.true(token.scopes.includes('support.type.function.mira'), token.scopes.join(', '));
        t.false(token.scopes.includes('keyword.declaration.function.mira'), token.scopes.join(', '));
    }
    for (const token of tokens.filter((token) => token.text === 'type')) {
        t.true(token.scopes.includes('keyword.operator.expression.mira'), token.scopes.join(', '));
    }
    t.is(tokens.filter((token) => token.text === 'type').length, 2);
    for (const token of tokens.filter((token) => token.text === 'data' && (token.line === 2 || token.line === 3))) {
        t.true(token.scopes.includes('variable.other.mira'), token.scopes.join(', '));
    }
    t.is(tokens.filter((token) => token.text === 'data' && (token.line === 2 || token.line === 3)).length, 2);
});

test('keeps tuple and array element types inside their type context', (t) => {
    const tokens = tokenize(
        [
            'fn size(matrix: any[][]) -> [number, number]',
            'fn identity(..size: [number] | [number, number]) -> number[][]',
        ].join('\n'),
        'mirascript-doc',
    );
    expectScope(t, tokens, 'matrix', 'variable.emphasis.mira');
    expectScope(t, tokens, 'size', 'variable.emphasis.mira', 1);
    for (const token of tokens.filter((candidate) => candidate.text === 'number')) {
        t.true(
            token.scopes.includes('support.type.builtin.mira'),
            `${JSON.stringify(token.text)} should stay a built-in type; got ${token.scopes.join(', ')}`,
        );
    }
    t.is(tokens.filter((token) => token.text === 'number').length, 6);
});

test('highlights serialized extern record values in documentation mode', (t) => {
    const tokens = tokenize(
        [
            '(global) globalThis = /* <extern Window> */ (',
            '  event: nil,',
            '  customElements: /* <extern CustomElementRegistry> */ (',
            '    define: /* <extern function> */,',
            '    get: /* <extern function getValue> */,',
            '    iterate: /* <extern function*> */,',
            '    resolve: /* <extern async function resolveValue> */,',
            '    initialize: /* <extern async function* initializeValue> */,',
            '    Constructor: /* <extern class> */,',
            '    Widget: /* <extern class HTMLElement> */',
            '  ),',
            '  ../* x162 */',
            ')',
        ].join('\n'),
        'mirascript-doc',
    );
    expectScope(t, tokens, '(global)', 'entity.name.label.mira');
    expectScope(t, tokens, 'globalThis', 'variable.other.constant.mira');
    expectScope(t, tokens, 'event', 'variable.other.property.mira');
    expectScope(t, tokens, 'nil', 'constant.language.mira');
    expectScope(t, tokens, 'customElements', 'variable.other.property.mira');
    expectScope(t, tokens, 'define', 'entity.name.function.mira');
    expectScope(t, tokens, 'get', 'entity.name.function.mira');
    expectScope(t, tokens, 'iterate', 'entity.name.function.mira');
    expectScope(t, tokens, 'resolve', 'entity.name.function.mira');
    expectScope(t, tokens, 'initialize', 'entity.name.function.mira');
    expectScope(t, tokens, 'Constructor', 'entity.name.type.mira');
    expectScope(t, tokens, 'Widget', 'entity.name.type.mira');
    expectScope(t, tokens, 'Window', 'entity.name.type.mira');
    expectScope(t, tokens, 'CustomElementRegistry', 'entity.name.type.mira');
    for (const token of tokens.filter((candidate) => candidate.text === 'function')) {
        t.true(token.scopes.includes('keyword.declaration.function.mira'));
    }
    t.is(tokens.filter((token) => token.text === 'function').length, 5);
    for (const token of tokens.filter((candidate) => candidate.text === 'async')) {
        t.true(token.scopes.includes('storage.modifier.async.mira'));
    }
    t.is(tokens.filter((token) => token.text === 'async').length, 2);
    for (const token of tokens.filter((candidate) => candidate.text === '*')) {
        t.true(token.scopes.includes('keyword.operator.generator.mira'));
    }
    t.is(tokens.filter((token) => token.text === '*').length, 2);
    expectScope(t, tokens, 'getValue', 'entity.name.function.mira');
    expectScope(t, tokens, 'resolveValue', 'entity.name.function.mira');
    expectScope(t, tokens, 'initializeValue', 'entity.name.function.mira');
    expectScope(t, tokens, 'class', 'keyword.declaration.class.mira', 0);
    expectScope(t, tokens, 'class', 'keyword.declaration.class.mira', 1);
    expectScope(t, tokens, 'HTMLElement', 'entity.name.type.mira');
    expectScope(t, tokens, ' x162 ', 'comment.block.mira');
    for (const delimiter of tokens.filter((token) => token.text === '/* <' || token.text === '> */')) {
        t.true(delimiter.scopes.includes('comment.block.mira'));
    }
});

test('marks invalid numeric and escape sequences', (t) => {
    const tokens = tokenize(String.raw`let a = 0xGG; let b = "\q";`);
    expectScope(t, tokens, '0xGG', 'invalid.illegal.numeric.mira');
    expectScope(t, tokens, String.raw`\q`, 'invalid.illegal.escape.mira');
});

test('keeps multiline rule state', (t) => {
    const tokens = tokenize('/* first\nsecond */\n"first ${\nvalue\n}"');
    expectScope(t, tokens, 'second ', 'comment.block.mira');
    expectScope(t, tokens, 'value', 'variable.other.mira');
});
