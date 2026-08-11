import { createHighlighterCore, type HighlighterCore } from 'shiki';
import { createJavaScriptRegexEngine } from 'shiki/engine-javascript.mjs';
import test from 'ava';
import { grammars } from '../src/index.ts';

test('grammars can be loaded', async (t) => {
    const highlighter = await createHighlighterCore({
        langs: grammars,
        themes: [],
        engine: createJavaScriptRegexEngine(),
    });
    const mirascript = highlighter.getLanguage('mirascript');
    t.truthy(mirascript);

    const mirascriptTemplate = highlighter.getLanguage('mirascript-template');
    t.truthy(mirascriptTemplate);

    const mirascriptDoc = highlighter.getLanguage('mirascript-doc');
    t.truthy(mirascriptDoc);

    highlighter.dispose();
});
