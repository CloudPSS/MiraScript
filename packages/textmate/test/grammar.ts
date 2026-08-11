import { createHighlighterCore, type HighlighterCore } from '@shikijs/core';
import { INITIAL, type StateStack } from '@shikijs/vscode-textmate';
import { createOnigurumaEngine } from '@shikijs/engine-oniguruma';
import wasm from '@shikijs/engine-oniguruma/wasm-inlined';
import assert from 'node:assert/strict';
import { after, before, test } from 'node:test';
import { grammars } from '../scripts/index.ts';
import { mirascriptLanguage } from '../scripts/language.ts';

let highlighter: HighlighterCore;

before(async () => {
    highlighter = await createHighlighterCore({
        langs: grammars,
        themes: [],
        engine: createOnigurumaEngine(wasm),
    });
});

after(() => highlighter.dispose());

/**
 * Tokenize complete source while preserving TextMate state across lines.
 */
function tokenize(code: string, language = 'mirascript') {
    const grammar = highlighter.getLanguage(language);
    let state: StateStack = INITIAL;
    return code.split('\n').flatMap((line, lineIndex) => {
        const result = grammar.tokenizeLine(line, state);
        state = result.ruleStack;
        return result.tokens.map((token) => ({
            line: lineIndex,
            text: line.slice(token.startIndex, token.endIndex),
            scopes: token.scopes,
        }));
    });
}

/**
 * Assert that a selected textual token contains the expected scope.
 */
function expectScope(tokens: Array<{ text: string; scopes: string[] }>, text: string, scope: string, occurrence = 0) {
    const matching = tokens.filter((token) => token.text === text);
    assert.ok(matching.length > occurrence, `Missing token ${JSON.stringify(text)} #${occurrence}`);
    assert.ok(
        matching[occurrence].scopes.includes(scope),
        `${JSON.stringify(text)} should include ${scope}; got ${matching[occurrence].scopes.join(', ')}`,
    );
}

test('classifies declarations, parameters, properties, calls, and Unicode identifiers', () => {
    const tokens = tokenize('mod 数学 { fn 加法(mut 左, ..其余) { 对象.方法(左).属性 } }');
    expectScope(tokens, 'mod', 'keyword.control.module.mira');
    expectScope(tokens, '数学', 'entity.name.namespace.mira');
    expectScope(tokens, '加法', 'entity.name.function.mira');
    expectScope(tokens, '左', 'variable.emphasis.mira');
    expectScope(tokens, '其余', 'variable.other.constant.emphasis.mira');
    expectScope(tokens, '方法', 'entity.name.function.member.mira');
    expectScope(tokens, '属性', 'variable.other.property.mira');
});

test('classifies keyword and numeric families', () => {
    const tokens = tokenize('if true and value not in global { let @常量 = 0xCA_FE; 1.5e+2 }');
    expectScope(tokens, 'if', 'keyword.control.mira');
    expectScope(tokens, 'true', 'constant.language.mira');
    expectScope(tokens, 'and', 'keyword.operator.wordlike.mira');
    expectScope(tokens, 'not', 'keyword.operator.wordlike.mira');
    expectScope(tokens, 'in', 'keyword.operator.wordlike.mira');
    expectScope(tokens, 'global', 'variable.language.mira');
    expectScope(tokens, '@常量', 'variable.other.constant.mira');
    expectScope(tokens, '0xCA_FE', 'constant.numeric.hex.mira');
    expectScope(tokens, '1.5e+2', 'constant.numeric.float.mira');
});

test('does not classify control keywords followed by parentheses as functions', () => {
    const tokens = tokenize('if (condition) { case (value) { call(value) } }');
    expectScope(tokens, 'if', 'keyword.control.mira');
    expectScope(tokens, 'case', 'keyword.control.mira');
    expectScope(tokens, 'call', 'entity.name.function.mira');
    for (const keyword of ['if', 'case']) {
        const token = tokens.find((candidate) => candidate.text === keyword);
        assert.ok(!token!.scopes.includes('entity.name.function.mira'));
    }
});

test('only classifies type as a keyword in its two contextual forms', () => {
    const tokens = tokenize('type(value); type Value; let type = 1; type + 1; object.type(); value::type();');
    expectScope(tokens, 'type', 'keyword.operator.expression.mira', 0);
    expectScope(tokens, 'type', 'keyword.operator.expression.mira', 1);
    expectScope(tokens, 'type', 'variable.other.mira', 2);
    expectScope(tokens, 'type', 'variable.other.mira', 3);
    expectScope(tokens, 'type', 'entity.name.function.member.mira', 4);
    expectScope(tokens, 'type', 'keyword.operator.expression.mira', 5);
});

test('handles nested interpolation and format strings', () => {
    const tokens = tokenize('"value ${ if ok { fn_call((1 + 2)) } } / $(value:>8[.]2f)"');
    expectScope(tokens, '${', 'punctuation.definition.template-expression.begin.mira');
    expectScope(tokens, 'fn_call', 'entity.name.function.mira');
    expectScope(tokens, '$(', 'punctuation.definition.template-expression.begin.mira');
    expectScope(tokens, ':', 'punctuation.separator.format.mira');
    expectScope(tokens, '>8', 'string.unquoted.format.mira');
});

test('matches the exact interpolation width in verbatim strings', () => {
    for (const width of [1, 2, 3, 16]) {
        const ats = '@'.repeat(width);
        const dollars = '$'.repeat(width);
        const shorter = '$'.repeat(Math.max(1, width - 1));
        const tokens = tokenize(`${ats}"literal ${shorter}name ${dollars}name"${ats}`);
        expectScope(tokens, dollars, 'punctuation.definition.template-expression.begin.mira');
        expectScope(tokens, 'name', 'variable.other.mira', width === 1 ? 1 : 0);
        if (width > 1) {
            const literal = tokens.find((token) => token.text.includes(`${shorter}name`));
            assert.ok(literal!.scopes.includes('string.quoted.double.verbatim.mira'));
            assert.ok(!literal!.scopes.includes('meta.interpolation.simple.mira'));
        }
    }
});

test('uses the shared MiraScript language metadata', () => {
    assert.equal(grammars[0].name, mirascriptLanguage.name);
    assert.equal(grammars[0].scopeName, mirascriptLanguage.scopeName);
    assert.deepEqual(grammars[0].aliases, mirascriptLanguage.aliases);
});

test('separates documentation prefixes from bold and italic markup', () => {
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
    expectScope(tokens, '*a*', 'markup.italic.documentation.mira');
    expectScope(tokens, '**b**', 'markup.bold.documentation.mira');
    for (const line of [1, 2, 3, 4, 5]) {
        const prefix = tokens.find((token) => token.line === line && token.text.trim() === '*');
        assert.ok(prefix, `Missing documentation prefix on line ${line}`);
        assert.equal(prefix.scopes.at(-1), 'comment.block.documentation.mira');
    }
});

test('does not consume a documentation terminator as italic markup', () => {
    const tokens = tokenize('/**\n * *1*/\nlet x = 12;');
    expectScope(tokens, 'let', 'keyword.declaration.mira');
    expectScope(tokens, 'x', 'variable.other.mira');
    assert.ok(
        !tokens.some((token) => token.text === '*1*' && token.scopes.includes('markup.italic.documentation.mira')),
    );
});

test('ends a documentation fence before a prefixed comment terminator', () => {
    const tokens = tokenize('/**\n * ```mirascript\n * */\n * ```');
    const trailingLine = tokens.filter((token) => token.line === 3);
    assert.ok(trailingLine.length > 0);
    for (const token of trailingLine) {
        assert.ok(!token.scopes.includes('comment.block.documentation.mira'), token.scopes.join(', '));
        assert.ok(!token.scopes.includes('markup.fenced_code.block.mira'), token.scopes.join(', '));
    }
});

test('closes documentation before an embedded string can consume its terminator', () => {
    const tokens = tokenize("/**\n * ```mirascript\n * '*/\nlet recovered = 1;");
    expectScope(tokens, 'let', 'keyword.declaration.mira');
    expectScope(tokens, 'recovered', 'variable.other.mira');
    const recovered = tokens.find((token) => token.text === 'recovered');
    assert.ok(!recovered!.scopes.includes('comment.block.documentation.mira'));
    assert.ok(!recovered!.scopes.includes('markup.fenced_code.block.mira'));
});

test('preserves multiline embedded source state between safe documentation lines', () => {
    const tokens = tokenize("/**\n * ```mirascript\n * 'first\n * second'\n * ```\n */");
    expectScope(tokens, 'second', 'string.quoted.single.mira');
});

test('highlights documentation fences for the MiraScript name, aliases, and an omitted tag', () => {
    const tags = [undefined, mirascriptLanguage.name, ...(mirascriptLanguage.aliases ?? []), 'mIrAsCrIpT'];
    for (const tag of tags) {
        const tokens = tokenize(['/**', ` * \`\`\`${tag ?? ''}`, ' * matrix.identity(3)', ' * ```', ' */'].join('\n'));
        if (tag) expectScope(tokens, tag, 'fenced_code.block.language.mira');
        expectScope(tokens, 'identity', 'entity.name.function.member.mira');
    }
});

test('supports documentation fences with three or more backticks', () => {
    for (const [openingLength, closingLength] of [
        [3, 3],
        [4, 4],
        [8, 8],
        [12, 16],
    ]) {
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
        expectScope(tokens, 'identity', 'entity.name.function.member.mira');
        expectScope(tokens, 'outside', 'variable.other.mira');
    }
});

test('keeps unknown documentation fence tags as unparsed code blocks', () => {
    const tokens = tokenize(
        ['/**', ' * ````javascript', ' * let value = call(1);', ' * ````', ' */', 'let outside = 1;'].join('\n'),
    );
    expectScope(tokens, 'javascript', 'fenced_code.block.language.mira');
    const body = tokens.find((token) => token.line === 2 && token.text.includes('let value'));
    assert.ok(body);
    assert.ok(body.scopes.includes('markup.fenced_code.block.mira'));
    assert.ok(body.scopes.includes('markup.raw.block.mira'));
    assert.ok(!body.scopes.includes('keyword.declaration.mira'));
    assert.ok(!body.scopes.includes('entity.name.function.mira'));
    expectScope(tokens, 'outside', 'variable.other.mira');
});

test('highlights MiraScript fenced code inside documentation comments', () => {
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
    expectScope(tokens, 'mirascript', 'fenced_code.block.language.mira');
    expectScope(tokens, 'identity', 'entity.name.function.member.mira');
    expectScope(tokens, '3', 'constant.numeric.float.mira');
    expectScope(tokens, '// [[1, 0, 0], [0, 1, 0], [0, 0, 1]]', 'comment.line.double-slash.mira');
    expectScope(tokens, '@returns', 'storage.type.class.documentation.mira');
    expectScope(tokens, 'let', 'keyword.declaration.mira');
    expectScope(tokens, 'outside', 'variable.other.mira');
});

test('ends an unterminated documentation fence at the comment boundary', () => {
    const tokens = tokenize('/**\n * ```mirascript\n * fn call() { nil }\n */\nlet recovered = 1;');
    expectScope(tokens, 'call', 'entity.name.function.mira');
    expectScope(tokens, 'let', 'keyword.declaration.mira');
    expectScope(tokens, 'recovered', 'variable.other.mira');
    const recovered = tokens.find((token) => token.text === 'recovered');
    assert.ok(!recovered!.scopes.includes('comment.block.documentation.mira'));
});

test('highlights template text and embedded MiraScript', () => {
    const tokens = tokenize('Hello $name: ${ fn_call(1) }', 'mirascript-template');
    expectScope(tokens, 'Hello ', 'string.unquoted.template.mira');
    expectScope(tokens, '$', 'punctuation.definition.template-expression.begin.mira');
    expectScope(tokens, 'name', 'variable.other.mira');
    expectScope(tokens, 'fn_call', 'entity.name.function.mira');
});

test('highlights generated documentation syntax', () => {
    const tokens = tokenize(
        [
            '\0(parameter) mut value',
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
    expectScope(tokens, '(parameter)', 'entity.name.label.mira');
    expectScope(tokens, '(field)', 'entity.name.label.mira');
    expectScope(tokens, 'value', 'variable.emphasis.mira');
    expectScope(tokens, 'immutable', 'variable.other.constant.mira');
    expectScope(tokens, '@constant', 'variable.other.constant.mira');
    expectScope(tokens, 'mutable', 'variable.other.readwrite.mira');
    expectScope(tokens, 'extern', 'storage.modifier.extern.mira');
    expectScope(tokens, 'function', 'keyword.declaration.function.mira');
    expectScope(tokens, 'transform', 'entity.name.function.mira');
    expectScope(tokens, 'record', 'support.type.builtin.mira');
    expectScope(tokens, 'callback', 'entity.name.function.emphasis.mira');
    expectScope(tokens, 'boolean', 'support.type.builtin.mira');
    expectScope(tokens, 'MyValue', 'variable.other.mira');
    expectScope(tokens, 'field', 'variable.other.constant.emphasis.mira');
    assert.ok(
        !tokens.filter((token) => token.line === 8).some((token) => token.scopes.includes('entity.name.label.mira')),
    );
    expectScope(tokens, '->', 'keyword.operator.type.mira');
});

test('distinguishes doc declarations, globals, and nested function types', () => {
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
    expectScope(tokens, 'mod', 'keyword.control.module.mira', 0);
    expectScope(tokens, 'matrix', 'entity.name.namespace.mira', 0);
    expectScope(tokens, 'pub', 'keyword.control.module.mira');
    expectScope(tokens, 'PI', 'variable.other.constant.mira', 0);
    expectScope(tokens, 'PI', 'variable.other.constant.mira', 1);
    expectScope(tokens, '(global)', 'entity.name.label.mira', 0);
    expectScope(tokens, 'fn', 'keyword.declaration.function.mira', 0);
    expectScope(tokens, 'fn', 'support.type.function.mira', 2);
    expectScope(tokens, 'data', 'variable.other.constant.emphasis.mira', 1);
    expectScope(tokens, 'f', 'entity.name.function.emphasis.mira');
    expectScope(tokens, 'value', 'variable.other.constant.emphasis.mira');
    expectScope(tokens, 'type', 'keyword.operator.expression.mira');
    expectScope(tokens, 'data', 'variable.other.mira', 2);
});

test('keeps nested types in unlabelled and module function signatures', () => {
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
    assert.equal(declarationFns.length, 2);
    assert.equal(typeFns.length, 2);
    for (const token of declarationFns) {
        assert.ok(token.scopes.includes('keyword.declaration.function.mira'), token.scopes.join(', '));
        assert.ok(!token.scopes.includes('support.type.function.mira'), token.scopes.join(', '));
    }
    for (const token of typeFns) {
        assert.ok(token.scopes.includes('support.type.function.mira'), token.scopes.join(', '));
        assert.ok(!token.scopes.includes('keyword.declaration.function.mira'), token.scopes.join(', '));
    }
    for (const token of tokens.filter((token) => token.text === 'type')) {
        assert.ok(token.scopes.includes('keyword.operator.expression.mira'), token.scopes.join(', '));
    }
    assert.equal(tokens.filter((token) => token.text === 'type').length, 2);
    for (const token of tokens.filter((token) => token.text === 'data' && (token.line === 2 || token.line === 3))) {
        assert.ok(token.scopes.includes('variable.other.mira'), token.scopes.join(', '));
    }
    assert.equal(tokens.filter((token) => token.text === 'data' && (token.line === 2 || token.line === 3)).length, 2);
});

test('keeps tuple and array element types inside their type context', () => {
    const tokens = tokenize(
        [
            'fn size(matrix: any[][]) -> [number, number]',
            'fn identity(..size: [number] | [number, number]) -> number[][]',
        ].join('\n'),
        'mirascript-doc',
    );
    expectScope(tokens, 'matrix', 'variable.other.constant.emphasis.mira');
    expectScope(tokens, 'size', 'variable.other.constant.emphasis.mira', 1);
    for (const token of tokens.filter((candidate) => candidate.text === 'number')) {
        assert.ok(
            token.scopes.includes('support.type.builtin.mira'),
            `${JSON.stringify(token.text)} should stay a built-in type; got ${token.scopes.join(', ')}`,
        );
    }
    assert.equal(tokens.filter((token) => token.text === 'number').length, 6);
});

test('highlights serialized extern record values in documentation mode', () => {
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
    expectScope(tokens, '(global)', 'entity.name.label.mira');
    expectScope(tokens, 'globalThis', 'variable.other.constant.mira');
    expectScope(tokens, 'event', 'variable.other.property.mira');
    expectScope(tokens, 'nil', 'constant.language.mira');
    expectScope(tokens, 'customElements', 'variable.other.property.mira');
    expectScope(tokens, 'define', 'entity.name.function.mira');
    expectScope(tokens, 'get', 'entity.name.function.mira');
    expectScope(tokens, 'iterate', 'entity.name.function.mira');
    expectScope(tokens, 'resolve', 'entity.name.function.mira');
    expectScope(tokens, 'initialize', 'entity.name.function.mira');
    expectScope(tokens, 'Constructor', 'entity.name.type.mira');
    expectScope(tokens, 'Widget', 'entity.name.type.mira');
    expectScope(tokens, 'Window', 'entity.name.type.mira');
    expectScope(tokens, 'CustomElementRegistry', 'entity.name.type.mira');
    for (const token of tokens.filter((candidate) => candidate.text === 'function')) {
        assert.ok(token.scopes.includes('keyword.declaration.function.mira'));
    }
    assert.equal(tokens.filter((token) => token.text === 'function').length, 5);
    for (const token of tokens.filter((candidate) => candidate.text === 'async')) {
        assert.ok(token.scopes.includes('storage.modifier.async.mira'));
    }
    assert.equal(tokens.filter((token) => token.text === 'async').length, 2);
    for (const token of tokens.filter((candidate) => candidate.text === '*')) {
        assert.ok(token.scopes.includes('keyword.operator.generator.mira'));
    }
    assert.equal(tokens.filter((token) => token.text === '*').length, 2);
    expectScope(tokens, 'getValue', 'entity.name.function.mira');
    expectScope(tokens, 'resolveValue', 'entity.name.function.mira');
    expectScope(tokens, 'initializeValue', 'entity.name.function.mira');
    expectScope(tokens, 'class', 'keyword.declaration.class.mira', 0);
    expectScope(tokens, 'class', 'keyword.declaration.class.mira', 1);
    expectScope(tokens, 'HTMLElement', 'entity.name.type.mira');
    expectScope(tokens, ' x162 ', 'comment.block.mira');
    for (const delimiter of tokens.filter((token) => token.text === '/* <' || token.text === '> */')) {
        assert.ok(delimiter.scopes.includes('comment.block.mira'));
    }
});

test('marks invalid numeric and escape sequences', () => {
    const tokens = tokenize(String.raw`let a = 0xGG; let b = "\q";`);
    expectScope(tokens, '0xGG', 'invalid.illegal.numeric.mira');
    expectScope(tokens, String.raw`\q`, 'invalid.illegal.escape.mira');
});

test('keeps multiline rule state', () => {
    const tokens = tokenize('/* first\nsecond */\n"first ${\nvalue\n}"');
    expectScope(tokens, 'second ', 'comment.block.mira');
    expectScope(tokens, 'value', 'variable.other.mira');
});
