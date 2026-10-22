import { createHighlighterCore, type HighlighterCore } from '@shikijs/core';
import { createJavaScriptRegexEngine } from '@shikijs/engine-javascript';
import { mirascript, mirascriptDoc, mirascriptTemplate } from '@mirascript/textmate';

export { INITIAL } from '@shikijs/vscode-textmate';

/** Shared instance of the highlighter */
let highlighterPromise: Promise<HighlighterCore> | null = null;

/** Get instance of the highlighter */
export async function getHighlighter(): Promise<HighlighterCore> {
    highlighterPromise ??= createHighlighterCore({
        langs: [mirascript, mirascriptDoc, mirascriptTemplate],
        themes: [],
        engine: createJavaScriptRegexEngine(),
    });
    return highlighterPromise;
}
