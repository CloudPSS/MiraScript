import { createHighlighterCore, type HighlighterCore } from '@shikijs/core';
import { INITIAL, type StateStack } from '@shikijs/vscode-textmate';
import { createOnigurumaEngine } from '@shikijs/engine-oniguruma';
import wasm from '@shikijs/engine-oniguruma/wasm-inlined';
import assert from 'node:assert/strict';
import { after, before, test } from 'node:test';
import { grammars } from '../scripts/grammar.ts';

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
    expectScope(tokens, '${', 'punctuation.definition.interpolation.begin.mira');
    expectScope(tokens, 'fn_call', 'entity.name.function.mira');
    expectScope(tokens, '$(', 'punctuation.definition.interpolation.begin.mira');
    expectScope(tokens, ':', 'punctuation.separator.format.mira');
    expectScope(tokens, '>8', 'string.unquoted.format.mira');
});

test('matches the exact interpolation width in verbatim strings', () => {
    for (const width of [1, 2, 3, 16]) {
        const ats = '@'.repeat(width);
        const dollars = '$'.repeat(width);
        const shorter = '$'.repeat(Math.max(1, width - 1));
        const tokens = tokenize(`${ats}"literal ${shorter}name ${dollars}name"${ats}`);
        expectScope(tokens, dollars, 'punctuation.definition.interpolation.begin.mira');
        expectScope(tokens, 'name', 'variable.other.mira', width === 1 ? 1 : 0);
        if (width > 1) {
            const literal = tokens.find((token) => token.text.includes(`${shorter}name`));
            assert.ok(literal!.scopes.includes('string.quoted.double.verbatim.mira'));
            assert.ok(!literal!.scopes.includes('meta.interpolation.simple.mira'));
        }
    }
});

test('highlights template text and embedded MiraScript', () => {
    const tokens = tokenize('Hello $name: ${ fn_call(1) }', 'mirascript-template');
    expectScope(tokens, 'Hello ', 'string.unquoted.template.mira');
    expectScope(tokens, '$', 'punctuation.definition.interpolation.begin.mira');
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
    expectScope(tokens, 'fn', 'storage.type.function.mira', 2);
    expectScope(tokens, 'data', 'variable.other.constant.emphasis.mira', 1);
    expectScope(tokens, 'f', 'entity.name.function.emphasis.mira');
    expectScope(tokens, 'value', 'variable.other.constant.emphasis.mira');
    expectScope(tokens, 'type', 'keyword.operator.expression.mira');
    expectScope(tokens, 'data', 'variable.other.mira', 2);
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
