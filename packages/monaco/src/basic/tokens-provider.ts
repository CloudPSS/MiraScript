import type { Grammar } from '@shikijs/core';
import type { StateStack } from '@shikijs/vscode-textmate';
import { languages, type IDisposable } from '../monaco-api.js';
import { CONTRIBUTE_IDS } from '../contribute.js';
import { getHighlighter, getInitialState } from './highlighter-manager.js';
import { textmateScopesToMonaco } from './textmate-to-monaco.js';

const TOKENIZE_MAX_LINE_LENGTH = 20000;
const TOKENIZE_TIME_LIMIT = 500;

/** A Monaco tokens provider that uses TextMate grammars. */
class TokensProvider implements languages.TokensProvider {
    constructor(private readonly grammar: Grammar) {}
    /** @inheritdoc */
    getInitialState = getInitialState;
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
                scopes: textmateScopesToMonaco(token.scopes),
            })),
        };
    }
}

/** Register TextMate-backed token providers without changing Monaco themes. */
export function registerMiraScriptTokensProvider(): IDisposable[] {
    const disposables = [
        CONTRIBUTE_IDS.mirascript,
        CONTRIBUTE_IDS.mirascriptDoc,
        CONTRIBUTE_IDS.mirascriptTemplate,
    ].map((id) =>
        languages.registerTokensProviderFactory(id, {
            create: async () => {
                const highlighter = await getHighlighter();
                const grammar = highlighter.getLanguage(id);
                return new TokensProvider(grammar);
            },
        }),
    );
    return disposables;
}
