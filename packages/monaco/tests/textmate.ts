import assert from 'node:assert/strict';
import test from 'node:test';
import { registerMonacoApi } from '../dist/index.js';
import { registerMiraScriptTokensProvider } from '../dist/basic/index.js';
import type { IDisposable, languages } from 'monaco-editor';

const providers = new Map<string, languages.TokensProvider>();
let providerDisposeCount = 0;
let themeMutationCount = 0;

/**
 * Minimal constructable Monaco API placeholder used by registration tests.
 */
function MonacoStub(): undefined {
    return undefined;
}

const monaco = {
    Uri: MonacoStub,
    Range: MonacoStub,
    Position: MonacoStub,
    CancellationTokenSource: MonacoStub,
    Emitter: MonacoStub,
    editor: {
        defineTheme() {
            themeMutationCount += 1;
        },
        setTheme() {
            themeMutationCount += 1;
        },
    },
    languages: {
        setTokensProvider(languageId: string, provider: languages.TokensProvider): IDisposable {
            providers.set(languageId, provider);
            return {
                dispose() {
                    providerDisposeCount += 1;
                    providers.delete(languageId);
                },
            };
        },
    },
};

registerMonacoApi(monaco as unknown as typeof import('monaco-editor'));

test('registers the three public language IDs without touching themes', async () => {
    const disposables = await registerMiraScriptTokensProvider();
    assert.deepEqual([...providers.keys()], ['mirascript', 'mirascript-template', 'mirascript-doc']);
    assert.equal(themeMutationCount, 0);

    for (const disposable of disposables) disposable.dispose();
    assert.equal(providerDisposeCount, 3);
});

test('preserves multiline TextMate state and falls back for long lines', async () => {
    const disposables = await registerMiraScriptTokensProvider();
    const provider = providers.get('mirascript')!;
    const initial = provider.getInitialState();
    const first = provider.tokenize('"prefix ${', initial);
    assert.ok(first.tokens.some((token) => token.startIndex === 0 && token.scopes.startsWith('string.')));
    assert.ok(first.tokens.some((token) => token.scopes === 'punctuation.definition.interpolation.begin.mira'));
    const second = provider.tokenize('value', first.endState);
    assert.ok(second.tokens.some((token) => token.scopes === 'variable.other.mira'));

    const longLine = provider.tokenize('x'.repeat(20000), second.endState);
    assert.deepEqual(longLine.tokens, [{ startIndex: 0, scopes: '' }]);
    assert.equal(longLine.endState, second.endState);

    for (const disposable of disposables) disposable.dispose();
});

test('can register and dispose another independent provider set', async () => {
    const disposables = await registerMiraScriptTokensProvider();
    assert.equal(providers.size, 3);
    for (const disposable of disposables) disposable.dispose();
    assert.equal(providers.size, 0);
});

test('exposes shared keyword and documentation scopes through Monaco', async () => {
    const disposables = await registerMiraScriptTokensProvider();
    const scopeAt = (provider: languages.TokensProvider, line: string, text: string) => {
        const result = provider.tokenize(line, provider.getInitialState());
        const offset = line.indexOf(text);
        return result.tokens.findLast((token) => token.startIndex <= offset)?.scopes;
    };

    const source = providers.get('mirascript')!;
    assert.equal(scopeAt(source, 'if (condition) call(condition)', 'if'), 'keyword.control.mira');
    assert.equal(scopeAt(source, 'if (condition) call(condition)', 'call'), 'entity.name.function.mira');
    assert.equal(scopeAt(source, 'type(value)', 'type'), 'keyword.operator.expression.mira');
    assert.equal(scopeAt(source, 'let type = value', 'type'), 'variable.other.mira');
    assert.equal(scopeAt(source, 'fn identity(value) { value }', 'value'), 'variable.other.constant.emphasis.mira');

    const doc = providers.get('mirascript-doc')!;
    assert.equal(scopeAt(doc, '(field) description', '(field)'), 'entity.name.label.mira');
    assert.equal(scopeAt(doc, 'let immutable', 'immutable'), 'variable.other.constant.mira');
    assert.equal(scopeAt(doc, 'let mut mutable', 'mutable'), 'variable.other.readwrite.mira');
    assert.equal(
        scopeAt(doc, 'fn transform<T>(value: record<string, T>) -> T[]', 'transform'),
        'entity.name.function.mira',
    );
    assert.equal(
        scopeAt(doc, 'fn transform<T>(value: record<string, T>) -> T[]', 'record'),
        'support.type.builtin.mira',
    );

    const signature = [
        '\0(global) fn map(',
        '  data: array | record,',
        '  f: fn(value: any, input: type(data)) -> any,',
        ') -> type(data)',
    ];
    let state = doc.getInitialState();
    const signatureTokens = signature.map((line) => {
        const result = doc.tokenize(line, state);
        state = result.endState;
        return result.tokens;
    });
    const signatureScopeAt = (lineIndex: number, text: string) => {
        const offset = signature[lineIndex].indexOf(text);
        return signatureTokens[lineIndex].findLast((token) => token.startIndex <= offset)?.scopes;
    };
    assert.equal(signatureScopeAt(0, 'fn'), 'keyword.declaration.function.mira');
    assert.equal(signatureScopeAt(1, 'data'), 'variable.other.constant.emphasis.mira');
    assert.equal(signatureScopeAt(2, 'f'), 'entity.name.function.emphasis.mira');
    assert.equal(signatureScopeAt(2, 'fn'), 'storage.type.function.mira');
    assert.equal(signatureScopeAt(2, 'type'), 'keyword.operator.expression.mira');

    for (const disposable of disposables) disposable.dispose();
});
