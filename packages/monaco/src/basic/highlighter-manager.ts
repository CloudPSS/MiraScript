import type { HighlighterCore } from '@shikijs/core';
import type { StateStack } from '@shikijs/vscode-textmate';

/** Shared instance of the highlighter */
let highlighterPromise: Promise<HighlighterCore> | null = null;
let INITIAL: StateStack;

/** Create an instance of the highlighter */
async function createHighlighter(): Promise<HighlighterCore> {
    const [
        { createHighlighterCore },
        { createJavaScriptRegexEngine },
        { INITIAL: initial },
        { mirascript, mirascriptDoc, mirascriptTemplate },
    ] = await Promise.all([
        import('@shikijs/core'),
        import('@shikijs/engine-javascript'),
        import('@shikijs/vscode-textmate'),
        import('@mirascript/textmate'),
    ]);
    INITIAL = initial;
    return await createHighlighterCore({
        langs: [mirascript, mirascriptDoc, mirascriptTemplate],
        themes: [],
        engine: createJavaScriptRegexEngine(),
    });
}

/** Get instance of the highlighter */
export async function getHighlighter(): Promise<HighlighterCore> {
    highlighterPromise ??= createHighlighter();
    return highlighterPromise;
}

/** Initial state stack */
export function getInitialState(): StateStack {
    return INITIAL;
}
