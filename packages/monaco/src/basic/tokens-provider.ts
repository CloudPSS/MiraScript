import type { HighlighterCore, Grammar } from '@shikijs/core';
import type { StateStack } from '@shikijs/vscode-textmate';
import { mirascript, mirascriptDoc, mirascriptTemplate } from '@mirascript/textmate';
import { languages, type IDisposable } from '../monaco-api.js';

const REGISTRATIONS = [mirascript, mirascriptTemplate, mirascriptDoc];

const TOKENIZE_MAX_LINE_LENGTH = 20000;
const TOKENIZE_TIME_LIMIT = 500;

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

/** Shared instance of highlighter. */
class HighlighterManager implements IDisposable {
    private highlighterPromise: Promise<HighlighterCore> | null = null;

    /**
     * Get the shared highlighter instance.
     */
    private async getHighlighter(): Promise<HighlighterCore> {
        if (this.highlighterPromise) return this.highlighterPromise;

        const [{ createHighlighterCore }, { createOnigurumaEngine }, wasm] = await Promise.all([
            import('@shikijs/core'),
            import('@shikijs/engine-oniguruma'),
            import('@shikijs/engine-oniguruma/wasm-inlined'),
        ]);
        this.highlighterPromise = createHighlighterCore({
            langs: REGISTRATIONS,
            themes: [],
            engine: createOnigurumaEngine(wasm),
        });
        return this.highlighterPromise;
    }

    /** Get tokens provider factory of language */
    getTokensProviderFactory(languageId: string): languages.TokensProviderFactory {
        return {
            create: async () => {
                const highlighter = await this.getHighlighter();
                const { INITIAL } = await import('@shikijs/vscode-textmate');
                const grammar = highlighter.getLanguage(languageId);
                return new TokensProvider(grammar, INITIAL);
            },
        };
    }
    /** @inheritdoc */
    dispose(): void {
        const promise = this.highlighterPromise;
        this.highlighterPromise = null;
        if (promise) {
            promise
                .then((highlighter) => highlighter.dispose())
                .catch(() => {
                    // eslint-disable-next-line no-console
                    console.error('Failed to dispose highlighter');
                });
        }
    }
}

/** A Monaco tokens provider that uses TextMate grammars. */
class TokensProvider implements languages.TokensProvider {
    constructor(
        private readonly grammar: Grammar,
        private readonly initialState: StateStack,
    ) {}
    /** @inheritdoc */
    getInitialState(): StateStack {
        return this.initialState;
    }
    /** @inheritdoc */
    tokenize(line: string, state: StateStack): languages.ILineTokens {
        if (line.length >= TOKENIZE_MAX_LINE_LENGTH) {
            // eslint-disable-next-line no-console
            console.warn(
                `MiraScript TextMate tokenization skipped for line exceeding ${TOKENIZE_MAX_LINE_LENGTH} characters: ${line.slice(0, 100)}`,
            );
            return {
                endState: state,
                tokens: [{ startIndex: 0, scopes: '' }],
            };
        }

        const result = this.grammar.tokenizeLine(line, state, TOKENIZE_TIME_LIMIT);
        if (result.stoppedEarly) {
            // eslint-disable-next-line no-console
            console.warn(`MiraScript TextMate tokenization timed out: ${line.slice(0, 100)}`);
        }
        return {
            endState: result.ruleStack,
            tokens: result.tokens.map((token) => ({
                startIndex: token.startIndex,
                scopes: tokenScope(token.scopes),
            })),
        };
    }
}

/** Register TextMate-backed token providers without changing Monaco themes. */
export function registerMiraScriptTokensProvider(): IDisposable[] {
    const manager = new HighlighterManager();
    const disposables = REGISTRATIONS.map((reg) =>
        languages.registerTokensProviderFactory(reg.name, manager.getTokensProviderFactory(reg.name)),
    );
    disposables.push(manager);
    return disposables;
}
