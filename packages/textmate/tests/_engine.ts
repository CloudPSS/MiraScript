import { createHighlighter, type Highlighter } from 'shiki';
import { INITIAL, type StateStack } from 'shiki/textmate';
import test, { type ExecutionContext } from 'ava';
import { grammars } from '../src/index.ts';

let highlighter: Highlighter;

test.before(async () => {
    highlighter = await createHighlighter({
        langs: grammars,
        themes: [],
    });
});

test.after.always(() => highlighter.dispose());

/**
 * Tokenize complete source while preserving TextMate state across lines.
 */
export function tokenize(
    code: string,
    language = 'mirascript',
): Array<{ line: number; text: string; scopes: string[] }> {
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
export function expectScope(
    t: ExecutionContext,
    tokens: Array<{ text: string; scopes: string[] }>,
    text: string,
    scope: string,
    occurrence = 0,
): void {
    const matching = tokens.filter((token) => token.text === text);
    t.true(matching.length > occurrence, `Missing token ${JSON.stringify(text)} #${occurrence}`);
    t.true(
        matching[occurrence].scopes.includes(scope),
        `${JSON.stringify(text)} should include ${scope}; got ${matching[occurrence].scopes.join(', ')}`,
    );
}
