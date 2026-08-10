import { createHighlighterCore } from '@shikijs/core';
import { createOnigurumaEngine } from '@shikijs/engine-oniguruma';
import wasm from '@shikijs/engine-oniguruma/wasm-inlined';
import { INITIAL, type StateStack } from '@shikijs/vscode-textmate';
import { mirascript, mirascriptDoc, mirascriptTemplate } from '@mirascript/textmate';
import { languages, type IDisposable } from '../monaco-api.js';

const TOKENIZE_MAX_LINE_LENGTH = 20000;
const TOKENIZE_TIME_LIMIT = 500;

/** Monaco tokenization state backed by a TextMate rule stack. */
class TextMateState implements languages.IState {
    constructor(readonly ruleStack: StateStack = INITIAL) {}

    /** Clone the immutable TextMate state wrapper. */
    clone(): TextMateState {
        return new TextMateState(this.ruleStack);
    }

    /** Compare the underlying TextMate rule stacks. */
    equals(other: languages.IState): boolean {
        return other instanceof TextMateState && this.ruleStack.equals(other.ruleStack);
    }
}

/** Select the deepest scope that a native Monaco theme can style. */
function tokenScope(scopes: string[]): string {
    const inInterpolation = scopes.some((scope) => scope.startsWith('meta.interpolation.'));
    let fallback = '';
    const styledScopePrefixes = [
        'invalid.',
        'comment.',
        'string.',
        'keyword.',
        'constant.',
        'variable.',
        'entity.',
        'storage.',
        'support.',
        'markup.',
    ];
    for (let index = scopes.length - 1; index >= 0; index -= 1) {
        const scope = scopes[index]!;
        if (scope === 'source.mira' || scope === 'source.mira.doc' || scope === 'text.miratpl') continue;
        if (scope.startsWith('meta.')) continue;
        fallback ||= scope;
        if (inInterpolation || styledScopePrefixes.some((prefix) => scope.startsWith(prefix))) return scope;
    }
    return fallback;
}

/** Register TextMate-backed token providers without changing Monaco themes. */
export async function registerMiraScriptTokensProvider(): Promise<IDisposable[]> {
    const registrations = [
        ['mirascript', mirascript],
        ['mirascript-template', mirascriptTemplate],
        ['mirascript-doc', mirascriptDoc],
    ] as const;
    const highlighter = await createHighlighterCore({
        langs: registrations.map(([, grammar]) => grammar),
        themes: [],
        engine: createOnigurumaEngine(wasm),
    });
    const disposables = registrations.map(([languageId]) => {
        const grammar = highlighter.getLanguage(languageId);
        return languages.setTokensProvider(languageId, {
            getInitialState: () => new TextMateState(),
            tokenize(line: string, state: TextMateState) {
                if (line.length >= TOKENIZE_MAX_LINE_LENGTH) {
                    return {
                        endState: state,
                        tokens: [{ startIndex: 0, scopes: '' }],
                    };
                }

                const result = grammar.tokenizeLine(line, state.ruleStack, TOKENIZE_TIME_LIMIT);
                if (result.stoppedEarly) {
                    // eslint-disable-next-line no-console
                    console.warn(`MiraScript TextMate tokenization timed out: ${line.slice(0, 100)}`);
                }
                return {
                    endState: new TextMateState(result.ruleStack),
                    tokens: result.tokens.map((token) => ({
                        startIndex: token.startIndex,
                        scopes: tokenScope(token.scopes),
                    })),
                };
            },
        });
    });
    disposables.push({ dispose: () => highlighter.dispose() });
    return disposables;
}
