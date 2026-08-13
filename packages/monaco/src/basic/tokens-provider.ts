import type { HighlighterCore, Grammar } from '@shikijs/core';
import type { StateStack } from '@shikijs/vscode-textmate';
import { languages, type IDisposable } from '../monaco-api.js';
import { CONTRIBUTE_IDS } from '../contribute.js';

const TOKENIZE_MAX_LINE_LENGTH = 20000;
const TOKENIZE_TIME_LIMIT = 500;

const REMAP_PREFIXES: ReadonlyArray<[string, string]> = [
    ['constant.character.escape.', 'string.escape.'],
    ['constant.numeric.', 'number.'],
    ['constant.language.', 'keyword.'],
    ['support.variable.', 'variable.'],
    ['entity.name.type.', 'type.'],
    ['support.type.', 'type.'],
    ['entity.name.namespace.', 'namespace.'],
    ['entity.name.function.', 'function.'],
];

/** Remap scope */
function remapScope(scope: string): string {
    for (const [prefix, remap] of REMAP_PREFIXES) {
        if (scope.startsWith(prefix)) {
            return remap + scope.slice(prefix.length);
        }
    }
    return scope;
}

const STYLED_SCOPE_PREFIXES = [
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
/** Is styled scope */
function isStyledScope(scope: string): boolean {
    return STYLED_SCOPE_PREFIXES.some((prefix) => scope.startsWith(prefix));
}

/** Select the deepest scope that a native Monaco theme can style. */
function tokenScope(scopes: string[]): string {
    let embedded = scopes.findLastIndex((scope) => scope.startsWith('meta.embedded.'));
    if (embedded === -1) {
        embedded = 0;
    }
    let fallback;
    for (let index = scopes.length - 1; index >= embedded; index -= 1) {
        const scope = scopes[index]!;
        fallback ||= scope;
        if (isStyledScope(scope)) return remapScope(scope);
        if (scope.startsWith('meta.embedded.')) break;
    }
    return fallback ? remapScope(fallback) : 'source.mira';
}

/** Shared instance of highlighter. */
class HighlighterManager implements IDisposable {
    private highlighterPromise: Promise<HighlighterCore> | null = null;
    private INITIAL!: StateStack;
    /**
     * Load the shared highlighter instance.
     */
    private async loadHighlighter(): Promise<HighlighterCore> {
        const [
            { createHighlighterCore },
            { createJavaScriptRegexEngine },
            { INITIAL },
            { mirascript, mirascriptDoc, mirascriptTemplate },
        ] = await Promise.all([
            import('@shikijs/core'),
            import('@shikijs/engine-javascript'),
            import('@shikijs/vscode-textmate'),
            import('@mirascript/textmate'),
        ]);
        this.INITIAL = INITIAL;
        return await createHighlighterCore({
            langs: [mirascript, mirascriptDoc, mirascriptTemplate],
            themes: [],
            engine: createJavaScriptRegexEngine(),
        });
    }

    /**
     * Get the shared highlighter instance.
     */
    private async getHighlighter(): Promise<HighlighterCore> {
        this.highlighterPromise ??= this.loadHighlighter();
        return this.highlighterPromise;
    }

    /** Get tokens provider factory of language */
    getTokensProviderFactory(languageId: string): languages.TokensProviderFactory {
        return {
            create: async () => {
                const highlighter = await this.getHighlighter();
                const grammar = highlighter.getLanguage(languageId);
                return new TokensProvider(grammar, this.INITIAL);
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
        private readonly INITIAL: StateStack,
    ) {}
    /** @inheritdoc */
    getInitialState(): StateStack {
        return this.INITIAL;
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
    const disposables = CONTRIBUTE_IDS.map((id) =>
        languages.registerTokensProviderFactory(id, manager.getTokensProviderFactory(id)),
    );
    disposables.push(manager);
    return disposables;
}
