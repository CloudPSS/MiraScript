import { createHighlighterCore, type HighlighterCore, type Grammar, type LanguageRegistration } from '@shikijs/core';
import { INITIAL, type StateStack } from '@shikijs/vscode-textmate';
import { createOnigurumaEngine } from '@shikijs/engine-oniguruma';
import { mirascript, mirascriptDoc, mirascriptTemplate } from '@mirascript/textmate';
import { languages, type IDisposable } from '../monaco-api.js';

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

const REGISTRATIONS: Record<string, LanguageRegistration> = {
    mirascript: mirascript,
    'mirascript-template': mirascriptTemplate,
    'mirascript-doc': mirascriptDoc,
};

/** Shared instance of highlighter. */
class HighlighterManager implements IDisposable {
    private highlighterPromise: Promise<HighlighterCore> | null = null;

    /**
     * Get the shared highlighter instance.
     */
    private async getHighlighter(): Promise<HighlighterCore> {
        this.highlighterPromise ??= createHighlighterCore({
            langs: Object.values(REGISTRATIONS),
            themes: [],
            engine: createOnigurumaEngine(async () => await import('@shikijs/engine-oniguruma/wasm-inlined')),
        });
        return this.highlighterPromise;
    }

    /** Get tokens provider factory of language */
    getTokensProviderFactory(languageId: string): languages.TokensProviderFactory {
        return {
            create: async () => {
                const highlighter = await this.getHighlighter();
                const grammar = highlighter.getLanguage(languageId);
                return new TokensProvider(grammar);
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
    constructor(private readonly grammar: Grammar) {}
    /** @inheritdoc */
    getInitialState(): StateStack {
        return INITIAL;
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
    const disposables = Object.keys(REGISTRATIONS).map((languageId) =>
        languages.registerTokensProviderFactory(languageId, manager.getTokensProviderFactory(languageId)),
    );
    disposables.push(manager);
    return disposables;
}
