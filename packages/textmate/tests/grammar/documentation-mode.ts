import test from 'ava';
import { tokenize, expectScope } from '../_engine.ts';

test('highlights types', (t) => {
    const types = ['boolean', 'true', 'false', 'number', 'string', 'array', 'record', 'extern', 'any', 'nil'];
    const tokens = tokenize(types.map((type) => `fn _${type}(v: ${type}) -> ${type}`).join('\n'), 'mirascript-doc');
    for (const type of types) {
        expectScope(t, tokens, type, 'support.type.builtin.mira');
    }
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
    expectScope(t, tokens, 'extern', 'keyword.declaration.extern.mira');
    expectScope(t, tokens, 'function', 'keyword.js');
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
    expectScope(t, tokens, 'type', 'support.type.type.mira');
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
        t.true(token.scopes.includes('support.type.type.mira'), token.scopes.join(', '));
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
    expectScope(t, tokens, 'globalThis', 'variable.other.mira');
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
    expectScope(t, tokens, 'Window', 'entity.name.type.js');
    expectScope(t, tokens, 'CustomElementRegistry', 'entity.name.type.js');
    for (const token of tokens.filter((candidate) => candidate.text === 'function')) {
        t.true(token.scopes.includes('keyword.js'));
    }
    t.is(tokens.filter((token) => token.text === 'function').length, 5);
    for (const token of tokens.filter((candidate) => candidate.text === 'async')) {
        t.true(token.scopes.includes('keyword.js'));
    }
    t.is(tokens.filter((token) => token.text === 'async').length, 2);
    for (const token of tokens.filter((candidate) => candidate.text === '*')) {
        t.true(token.scopes.includes('keyword.operator.generator.js'));
    }
    t.is(tokens.filter((token) => token.text === '*').length, 2);
    expectScope(t, tokens, 'getValue', 'entity.name.function.js');
    expectScope(t, tokens, 'resolveValue', 'entity.name.function.js');
    expectScope(t, tokens, 'initializeValue', 'entity.name.function.js');
    expectScope(t, tokens, 'class', 'keyword.js', 0);
    expectScope(t, tokens, 'class', 'keyword.js', 1);
    expectScope(t, tokens, 'HTMLElement', 'entity.name.type.js');
    expectScope(t, tokens, ' x162 ', 'comment.block.mira');
    for (const delimiter of tokens.filter((token) => ['/*', '*/'].includes(token.text))) {
        t.true(delimiter.scopes.includes('comment.block.mira'), delimiter.scopes.join(', '));
    }
    for (const delimiter of tokens.filter((token) => ['<', '>'].includes(token.text))) {
        t.true(
            delimiter.scopes.some((scope) => scope.startsWith('punctuation.definition.tag.')),
            delimiter.scopes.join(', '),
        );
    }
});

test('highlights inline and comment-only documentation tags', (t) => {
    const tokens = tokenize(
        [
            '<module matrix / arbitrary name>',
            '<function global.to-string (value)>',
            '  <extern async function* request animation frame>  ',
            '/* <module matrix> */',
            '/*<function render>*/',
        ].join('\n'),
        'mirascript-doc',
    );
    expectScope(t, tokens, 'module', 'keyword.declaration.module.mira', 0);
    expectScope(t, tokens, 'module', 'keyword.declaration.module.mira', 1);
    expectScope(t, tokens, 'matrix / arbitrary name', 'entity.name.namespace.mira');
    expectScope(t, tokens, 'matrix', 'entity.name.namespace.mira');
    expectScope(t, tokens, 'function', 'keyword.declaration.function.mira', 0);
    expectScope(t, tokens, 'function', 'keyword.declaration.function.mira', 2);
    expectScope(t, tokens, 'global.to-string (value)', 'entity.name.function.mira');
    expectScope(t, tokens, 'render', 'entity.name.function.mira');
    expectScope(t, tokens, 'extern', 'keyword.declaration.extern.mira');
    expectScope(t, tokens, 'async', 'keyword.js');
    expectScope(t, tokens, 'function', 'keyword.js', 1);
    expectScope(t, tokens, '*', 'keyword.operator.generator.js');
    expectScope(t, tokens, 'request animation frame', 'entity.name.function.js');
    t.is(tokens.filter((token) => token.text === '<').length, 5);
    t.is(tokens.filter((token) => token.text === '>').length, 5);
    for (const delimiter of tokens.filter((token) => token.text === '<')) {
        t.true(delimiter.scopes.includes('punctuation.definition.tag.begin.mira'), delimiter.scopes.join(', '));
    }
    for (const delimiter of tokens.filter((token) => token.text === '>')) {
        t.true(delimiter.scopes.includes('punctuation.definition.tag.end.mira'), delimiter.scopes.join(', '));
    }
});

test('highlights multiple inline tags while keeping tag delimiters tight', (t) => {
    const tokens = tokenize(
        ['value = <module matrix>', '<extern Array(3)> [1, <extern Object>, [1, <extern Object>]]'].join('\n'),
        'mirascript-doc',
    );
    expectScope(t, tokens, 'module', 'keyword.declaration.module.mira');
    expectScope(t, tokens, 'matrix', 'entity.name.namespace.mira');
    expectScope(t, tokens, 'Array', 'entity.name.type.js');
    expectScope(t, tokens, '(', 'punctuation.section.parens.begin.mira');
    expectScope(t, tokens, '3', 'constant.numeric.mira');
    expectScope(t, tokens, ')', 'punctuation.section.parens.end.mira');
    expectScope(t, tokens, 'Object', 'entity.name.type.js', 0);
    expectScope(t, tokens, 'Object', 'entity.name.type.js', 1);
    t.is(tokens.filter((token) => token.text === 'extern').length, 3);
    t.true(tokens.some((token) => token.scopes.includes('meta.documentation.tag.mira')));

    const invalidTokens = tokenize(
        [
            '/* prefix <module matrix> */',
            '< module matrix>',
            '<module matrix >',
            '/* <module matrix> suffix */',
            '/*',
            '<module matrix>',
            '*/',
        ].join('\n'),
        'mirascript-doc',
    );
    t.false(
        invalidTokens.some((token) => token.scopes.includes('meta.documentation.tag.mira')),
        invalidTokens
            .map((token) => `${token.line}:${JSON.stringify(token.text)} ${token.scopes.join(', ')}`)
            .join('\n'),
    );
    t.false(
        invalidTokens.some((token) => token.scopes.some((scope) => scope.startsWith('punctuation.definition.tag.'))),
    );

    const sourceTokens = tokenize('<module matrix>\n/* <extern Navigator> */');
    t.false(sourceTokens.some((token) => token.scopes.includes('meta.documentation.tag.mira')));
    t.false(
        sourceTokens.some((token) => token.scopes.some((scope) => scope.startsWith('punctuation.definition.tag.'))),
    );
});

test('highlights extern tags while preserving surrounding document declarations', (t) => {
    const tokens = tokenize(
        [
            'let navigator = /* <extern Navigator> */ (',
            '  scheduling: /* <extern Scheduling> */,',
            '  getGamepads: /* <extern function> */,',
            ');',
            'let AbortController = /* <extern class AbortController> */;',
        ].join('\n'),
        'mirascript-doc',
    );
    expectScope(t, tokens, 'navigator', 'variable.other.constant.mira');
    expectScope(t, tokens, 'scheduling', 'variable.other.property.mira');
    expectScope(t, tokens, 'getGamepads', 'entity.name.function.mira');
    expectScope(t, tokens, 'AbortController', 'variable.other.constant.mira', 0);
    expectScope(t, tokens, 'extern', 'keyword.declaration.extern.mira');
    expectScope(t, tokens, 'Navigator', 'entity.name.type.js');
    expectScope(t, tokens, 'Scheduling', 'entity.name.type.js');
    expectScope(t, tokens, 'function', 'keyword.js');
    expectScope(t, tokens, 'class', 'keyword.js');
    expectScope(t, tokens, 'AbortController', 'entity.name.type.js', 1);
    for (const delimiter of tokens.filter((token) => ['/*', '*/'].includes(token.text))) {
        t.true(delimiter.scopes.includes('comment.block.mira'), delimiter.scopes.join(', '));
    }
});

test('highlights line-leading field declarations and infers callable and class field names', (t) => {
    const tokens = tokenize(
        [
            '(field) navigator: /* <extern Navigator> */',
            'scheduling: /* <extern Scheduling> */',
            'getGamepads: /* <extern function> */',
            'AbortController: /* <extern class AbortController> */;',
            '(field) requestAnimationFrame: /* <extern function> */;',
            '(field) 1: value',
            '(field) "invalid-name": value',
            '2: /* <extern function> */;',
            '"class-name": /* <extern class HTMLElement> */;',
        ].join('\n'),
        'mirascript-doc',
    );
    expectScope(t, tokens, '(field)', 'entity.name.label.mira');
    expectScope(t, tokens, 'navigator', 'variable.other.property.mira');
    expectScope(t, tokens, 'scheduling', 'variable.other.property.mira');
    expectScope(t, tokens, 'getGamepads', 'entity.name.function.mira');
    expectScope(t, tokens, 'AbortController', 'entity.name.type.mira', 0);
    expectScope(t, tokens, 'AbortController', 'entity.name.type.js', 1);
    expectScope(t, tokens, 'requestAnimationFrame', 'entity.name.function.mira');
    expectScope(t, tokens, '1', 'variable.other.property.mira');
    expectScope(t, tokens, '"invalid-name"', 'variable.other.property.mira');
    expectScope(t, tokens, '2', 'entity.name.function.mira');
    expectScope(t, tokens, '"class-name"', 'entity.name.type.mira');
    t.true(tokens.some((token) => token.scopes.includes('meta.documentation.field.mira')));

    const indented = tokenize(
        [
            '  (field) indented: /* <extern function> */',
            '  plain: /* <extern class HTMLElement> */',
            '  (global) value = /* <extern Window> */',
            '(field) legacy = /* <extern function> */',
            'legacy = /* <extern class HTMLElement> */',
        ].join('\n'),
        'mirascript-doc',
    );
    t.false(indented.some((token) => token.scopes.includes('meta.documentation.field.mira')));
    t.false(indented.some((token) => token.scopes.includes('meta.documentation.global-value.mira')));
});
